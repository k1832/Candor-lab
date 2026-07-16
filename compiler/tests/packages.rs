//! Manifested-package directory builds — design 0017 §1 / the 2026-07-15
//! erratum to design 0008 §2.4 (the `src/` module-root relocation).
//!
//! A directory carrying a `candor.toml` roots its module tree at `src/`: a
//! binary's entry is `fn main` in `src/main.cnr`, a library's public root is
//! `src/<name>.cnr`. A manifest-less bare directory (covered by `modules.rs`)
//! keeps 0008's directory-root `main.cnr` unchanged.

use std::path::PathBuf;

use candor_proto::{check_dir, run_dir, RunResult};

fn dir(name: &str) -> PathBuf {
    PathBuf::from(format!("{}/tests/fixtures/packages/{name}", env!("CARGO_MANIFEST_DIR")))
}

fn codes(name: &str) -> Vec<String> {
    match check_dir(&dir(name)) {
        Ok(diags) => diags.into_iter().map(|d| d.code).collect(),
        Err(d) => vec![d.code],
    }
}

// ---- a manifested binary package: rooted at src/, entry in src/main.cnr -----

#[test]
fn bin_package_checks_clean() {
    assert!(codes("bin_pkg").is_empty(), "bin_pkg should check clean, got {:?}", codes("bin_pkg"));
}

#[test]
fn bin_package_runs_via_src_root() {
    match run_dir(&dir("bin_pkg")) {
        RunResult::Ok(r) => assert_eq!(r.ret, 42),
        other => panic!("bin_pkg did not run: {}", describe(other)),
    }
}

// ---- a manifested library package: public root src/<name>.cnr, no main ------

#[test]
fn lib_package_checks_clean() {
    assert!(codes("lib_pkg").is_empty(), "lib_pkg should check clean, got {:?}", codes("lib_pkg"));
}

// ---- a manifested package missing its declared entry file -------------------

#[test]
fn missing_entry_is_rejected() {
    assert!(
        codes("bad_missing_entry").contains(&"E0906".to_string()),
        "want E0906 (missing src/main.cnr), got {:?}",
        codes("bad_missing_entry"),
    );
}

fn describe(r: RunResult) -> String {
    match r {
        RunResult::Ok(run) => format!("ok({})", run.ret),
        RunResult::Fault(f) => format!("fault: {}", f.to_json()),
        RunResult::CheckErrors(d) => {
            format!("check-errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => format!("parse-error: {}", d.to_json()),
    }
}

// ---------------------------------------------------------------------------
// Cross-package dependency resolution (design 0017 §5/§6/§7). Each test stages
// the needed sibling packages into a fresh temp dir (path deps resolve relative
// to the depending manifest) so no `candor.lock` is written into the fixtures.
// ---------------------------------------------------------------------------

use std::sync::atomic::{AtomicU64, Ordering};
use candor_proto::{build, compile_path, modules, resolve_pkg, run_dir_mir, run_dir_native, MirRunResult};

static STAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Copy each named package fixture into a fresh temp dir as siblings; return the
/// temp root. The depending package addresses the others by `../<name>`.
fn stage(names: &[&str]) -> PathBuf {
    let n = STAGE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir().join(format!("candor_pkg_{}_{}", std::process::id(), n));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    for name in names {
        copy_tree(&dir(name), &root.join(name));
    }
    root
}

fn copy_tree(src: &std::path::Path, dst: &std::path::Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let p = entry.path();
        let name = entry.file_name();
        // Never carry a stray lockfile or cache into the pristine copy.
        if name == "candor.lock" || name.to_str() == Some(".candor-cache") {
            continue;
        }
        let target = dst.join(&name);
        if p.is_dir() {
            copy_tree(&p, &target);
        } else {
            std::fs::copy(&p, &target).unwrap();
        }
    }
}

fn native_ret(r: MirRunResult) -> i64 {
    match r {
        MirRunResult::Ok(run) => run.ret,
        MirRunResult::Fault(f) => panic!("native fault: {}", f.to_json()),
        MirRunResult::CheckErrors(d) => panic!("native check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        MirRunResult::ParseError(d) => panic!("native parse error: {}", d.to_json()),
        MirRunResult::Unsupported(m) => panic!("native unsupported: {m}"),
    }
}

/// Run a staged package through every engine and assert a byte-exact return.
fn assert_all_engines(pkg_dir: &std::path::Path, want: i64) {
    match run_dir(pkg_dir) {
        RunResult::Ok(r) => assert_eq!(r.ret, want, "tree-walker"),
        other => panic!("tree-walker did not run: {}", describe(other)),
    }
    assert_eq!(native_ret(run_dir_mir(pkg_dir)), want, "MIR engine");
    assert_eq!(native_ret(run_dir_native(pkg_dir)), want, "native engine");
    let out = pkg_dir.join("prog.out");
    compile_path(pkg_dir, &out).unwrap_or_else(|e| panic!("compile failed: {e}"));
    let status = std::process::Command::new(&out).status().unwrap();
    assert_eq!(status.code(), Some((want & 0xff) as i32), "AOT executable exit");
}

// ---- the milestone: `app` depends on `b` (path), builds and runs -----------

#[test]
fn app_depends_on_b_builds_and_runs_across_engines() {
    let root = stage(&["app", "b"]);
    let app = root.join("app");
    assert!(check_dir(&app).unwrap().is_empty(), "app should check clean: {:?}", codes_at(&app));
    // b::value() = helper::secret() (23) + 100 = 123.
    assert_all_engines(&app, 123);
}

// ---------------------------------------------------------------------------
// NN#16 reproducibility gate: the pkgid is derived from a package's CONTENT
// (design 0017 §5/§6 content hash), never its filesystem path. Byte-identical
// source staged under two different absolute directories must yield identical
// pkgids — and therefore identical mangled symbols and artifacts. This fails
// against the old absolute-path hash and pins the Non-Negotiable that the build
// path can never leak into a compiled artifact.
// ---------------------------------------------------------------------------

#[test]
fn pkgid_is_path_independent() {
    // Two independent stagings live at two distinct absolute temp roots.
    let root_a = stage(&["app", "b"]);
    let root_b = stage(&["app", "b"]);
    assert_ne!(root_a, root_b, "the two stagings must occupy different paths");

    let res_a = resolve_pkg::resolve(&root_a.join("app")).unwrap();
    let res_b = resolve_pkg::resolve(&root_b.join("app")).unwrap();

    let ids_a: std::collections::BTreeMap<String, String> =
        res_a.packages.iter().map(|p| (p.name.clone(), p.pkgid.clone())).collect();
    let ids_b: std::collections::BTreeMap<String, String> =
        res_b.packages.iter().map(|p| (p.name.clone(), p.pkgid.clone())).collect();
    assert_eq!(ids_a, ids_b, "identical source at different paths must produce identical pkgids (NN#16)");

    // The pkgid is the leading segment of every mangled symbol, so the whole
    // mangled module universe must match too: the artifact carries no build path.
    let univ_a: std::collections::BTreeSet<String> =
        modules::discover_multi(&res_a).unwrap().into_iter().map(|(path, _, _)| path.join("::")).collect();
    let univ_b: std::collections::BTreeSet<String> =
        modules::discover_multi(&res_b).unwrap().into_iter().map(|(path, _, _)| path.join("::")).collect();
    assert_eq!(univ_a, univ_b, "the mangled module universe must be path-independent");
}

// ---------------------------------------------------------------------------
// NN#16 for candor.lock's `[package.source]`: a path-dependency source is
// recorded RELATIVE to the root package dir (design 0017 §6), never as an
// absolute machine path. Byte-identical source staged under two different
// absolute roots must produce a byte-identical lock — moving the tree preserves
// every path. This fails against the old absolute-path recording.
// ---------------------------------------------------------------------------

#[test]
fn lock_source_is_path_independent() {
    let root_a = stage(&["app", "b"]);
    let root_b = stage(&["app", "b"]);
    assert_ne!(root_a, root_b, "the two stagings must occupy different paths");

    resolve_pkg::resolve(&root_a.join("app")).unwrap();
    resolve_pkg::resolve(&root_b.join("app")).unwrap();

    let lock_a = std::fs::read_to_string(root_a.join("app/candor.lock")).unwrap();
    let lock_b = std::fs::read_to_string(root_b.join("app/candor.lock")).unwrap();
    assert_eq!(lock_a, lock_b, "identical source at different paths must produce a byte-identical candor.lock (NN#16)");

    // No absolute machine path leaks into the lock's `[package.source]`.
    assert!(!lock_a.contains("/home/"), "lock must embed no absolute path: {lock_a}");
    assert!(!lock_a.contains(root_a.to_str().unwrap()), "lock must not embed the temp-root prefix");
    // The root package is `"."`; the sibling path dep is `../b` (relative to root).
    assert!(lock_a.contains(r#"path = ".""#), "the root package's source is recorded as \".\": {lock_a}");
    assert!(lock_a.contains(r#"path = "../b""#), "the sibling path dep is recorded relative to root: {lock_a}");
}

// ---------------------------------------------------------------------------
// The incremental build resolves cross-package dependencies (design 0017
// Open-Q4): `candor build` on a package-with-deps discovers EVERY package's
// modules under its pkgid — the same multi-package universe check/run/compile
// build — and the two-hash cache invalidates correctly across the boundary.
// ---------------------------------------------------------------------------

/// The set of module paths a `candor build` report covers.
fn built_paths(r: &build::BuildReport) -> std::collections::BTreeSet<String> {
    r.modules.iter().map(|m| m.path.clone()).collect()
}

#[test]
fn incremental_build_covers_dependency_and_universe_matches_check() {
    let root = stage(&["app", "b"]);
    let app = root.join("app");

    let report = build::build_dir(&app).expect("incremental build must not hard-error");
    assert!(report.ok(), "incremental build clean: {:?}", report.diags.iter().map(|d| &d.code).collect::<Vec<_>>());

    // Anti-divergence: the incremental universe is exactly the resolver's shared
    // pkgid-prefixed discovery — the same module set build_tree (check/run/compile)
    // builds. The two paths cannot diverge because they share `discover_multi`.
    let res = resolve_pkg::resolve(&app).unwrap();
    let want: std::collections::BTreeSet<String> =
        modules::discover_multi(&res).unwrap().into_iter().map(|(path, _, _)| path.join("::")).collect();
    assert_eq!(built_paths(&report), want, "incremental universe must equal build_tree's discovery");

    // b's modules appear under b's pkgid — the dependency is not silently dropped.
    let b = res.packages.iter().find(|p| p.name == "b").expect("b resolved");
    let b_prefix = format!("{}::", b.pkgid);
    assert!(
        built_paths(&report).iter().any(|p| p.starts_with(&b_prefix)),
        "b's modules must be in the incremental universe: {:?}",
        built_paths(&report),
    );
    // Consistency with the merge path: check sees the same clean build.
    assert!(check_dir(&app).unwrap().is_empty(), "check agrees the build is clean");
}

#[test]
fn incremental_build_invalidates_across_package_boundary() {
    let root = stage(&["app", "b"]);
    let app = root.join("app");
    let res = resolve_pkg::resolve(&app).unwrap();
    let b = res.packages.iter().find(|p| p.name == "b").expect("b resolved");
    let helper_path = format!("{}::helper", b.pkgid);
    let main_path = format!("{}::main", res.root_pkg().pkgid);

    let first = build::build_dir(&app).unwrap();
    assert!(first.ok() && first.reused().is_empty(), "first build reuses nothing");

    // A no-change rebuild reuses EVERYTHING, including the dependency's modules.
    let second = build::build_dir(&app).unwrap();
    assert!(second.ok());
    assert!(second.checked().is_empty(), "no-change rebuild re-analyzed {:?}", second.checked());
    assert!(
        second.reused().contains(&helper_path),
        "a dependency module is reused on a no-change rebuild: {:?}",
        second.reused(),
    );

    // Edit the dependency's `helper` module BODY (a cross-boundary source change).
    // Its signature is unchanged, so only `helper` re-analyzes; `b` (sig stable)
    // and app's `main` (which does not depend on helper) stay reused — the same
    // two-hash zero-downstream behavior an in-package edit gets (design 0008 §2).
    std::fs::write(root.join("b/src/helper.cnr"), "pub fn secret() -> i64 {
    return 24;
}
").unwrap();
    let third = build::build_dir(&app).unwrap();
    assert!(third.ok(), "rebuild after dep edit clean: {:?}", third.diags.iter().map(|d| &d.code).collect::<Vec<_>>());
    assert_eq!(third.checked(), vec![helper_path.clone()], "only the changed dep module re-analyzes");
    assert!(third.reused().contains(&main_path), "app's main stays reused across the boundary: {:?}", third.reused());
}

#[test]
fn incremental_build_of_dependency_free_package_takes_single_package_path() {
    // A manifest-carrying but dependency-free package resolves nothing and keeps
    // the historical single-package universe (bare module paths, no pkgid, no
    // lock) — the incremental build is byte-for-byte unchanged from before.
    let root = stage(&["bin_pkg"]);
    let pkg = root.join("bin_pkg");
    let report = build::build_dir(&pkg).expect("build must not hard-error");
    assert!(report.ok(), "dep-free package builds clean: {:?}", report.diags.iter().map(|d| &d.code).collect::<Vec<_>>());
    assert!(!pkg.join("candor.lock").exists(), "a dep-free package resolves nothing (single-package path)");
    assert!(
        report.modules.iter().all(|m| !m.path.contains('#')),
        "single-package modules keep bare paths (no pkgid): {:?}",
        report.modules.iter().map(|m| &m.path).collect::<Vec<_>>(),
    );
}

// ---- the visibility wall: a b item not re-exported from b's public root -----

#[test]
fn cross_package_private_item_is_walled() {
    let root = stage(&["vis_app", "b"]);
    let cs = codes_at(&root.join("vis_app"));
    assert!(cs.contains(&"E0903".to_string()), "want E0903 boundary error, got {cs:?}");
}

// ---- diamond: dia_c reached via dia_b and directly is unified once ----------

#[test]
fn diamond_unifies_shared_dependency() {
    let root = stage(&["dia_app", "dia_b", "dia_c"]);
    let app = root.join("dia_app");
    assert!(check_dir(&app).unwrap().is_empty(), "diamond should check clean: {:?}", codes_at(&app));
    // mid() = base()+1 (8) + base() (7) = 15.
    assert_all_engines(&app, 15);
    // `dia_c` is unified to a single build node: three packages resolved, not four.
    let res = resolve_pkg::resolve(&app).unwrap();
    assert_eq!(res.packages.len(), 3, "dia_c must be deduped to one node");
    assert_eq!(res.packages.iter().filter(|p| p.name == "dia_c").count(), 1);
}

// ---- divergent diamond: the same name via two sources is a hard conflict ----

#[test]
fn divergent_diamond_is_a_hard_conflict() {
    let root = stage(&["div_app", "divc1", "divc2"]);
    let cs = codes_at(&root.join("div_app"));
    assert!(cs.contains(&"E0923".to_string()), "want E0923 source conflict, got {cs:?}");
}

// ---- cycle: a -> b -> a is a package-level cycle ---------------------------

#[test]
fn package_cycle_is_rejected() {
    let root = stage(&["cyc_a", "cyc_b"]);
    let cs = codes_at(&root.join("cyc_a"));
    assert!(cs.contains(&"E0927".to_string()), "want E0927 package cycle, got {cs:?}");
}

// ---- name collision: a dep local name equal to a top-level src module -------

#[test]
fn dep_name_module_collision_is_rejected() {
    let root = stage(&["coll_app", "b"]);
    let cs = codes_at(&root.join("coll_app"));
    assert!(cs.contains(&"E0930".to_string()), "want E0930 collision, got {cs:?}");
}

#[test]
fn alias_resolves_the_collision() {
    let root = stage(&["coll_ok", "b"]);
    let app = root.join("coll_ok");
    assert!(check_dir(&app).unwrap().is_empty(), "aliased dep should check clean: {:?}", codes_at(&app));
    // mylib(b)::value() (123) + local b::thing() (1) = 124.
    assert_all_engines(&app, 124);
}

// ---- lockfile: written with the resolved set + content hashes, then reused --

#[test]
fn lockfile_is_written_then_reused() {
    let root = stage(&["app", "b"]);
    let app = root.join("app");

    let first = resolve_pkg::resolve(&app).unwrap();
    assert!(!first.lock_reused, "first resolve creates the lock");
    assert!(app.join("candor.lock").exists(), "candor.lock must be written");
    assert_eq!(first.packages.len(), 2);
    for p in &first.packages {
        assert!(!p.content_hash.is_empty(), "every locked package records a content hash");
    }

    let second = resolve_pkg::resolve(&app).unwrap();
    assert!(second.lock_reused, "a present, consistent lock is reused verbatim");
}

// ---- per-package trust summary in the lock + the supply-chain delta ---------
// design 0017 §8: each lock entry records counts of boundary modules / foreign
// externs / `unsafe` regions (reusing the impl #4 audit walk) so that bumping a
// dependency surfaces its trust delta in `git diff candor.lock`. Enumerate-only,
// not gating (gating is Open-Q1).

#[test]
fn lock_records_per_package_trust_summary_and_tracks_the_delta() {
    // Growing b's surface trips the trust-delta gate (design 0017 §8, Open-Q1);
    // accept it so this test can observe the *recorded* delta. Hold the env lock
    // for the same reason the gate tests below do.
    let _guard = TRUST_ENV_LOCK.lock().unwrap();
    let root = stage(&["audit_app", "audit_b", "audit_c"]);
    let app = root.join("audit_app");

    // Gate: `app` -> `b` (a foreign extern + an `unsafe` region) records b's
    // CORRECT counts — matching b's actual boundary externs + unsafe regions —
    // and the root package records its own (inert) summary too.
    resolve_pkg::resolve(&app).unwrap();
    assert_eq!(lock_trust(&app, "audit_app"), (0, 0, 0), "the root's own summary");
    assert_eq!(lock_trust(&app, "b"), (1, 1, 1), "b: 1 boundary module, 1 extern, 1 unsafe");
    assert_eq!(lock_trust(&app, "c"), (1, 1, 1), "transitive c is summarized too");

    // The delta is the point: add one `unsafe` region to b, re-resolve, and b's
    // recorded `unsafe_regions` increments (1 -> 2) — the visible supply-chain
    // delta a reviewer sees in the lock diff. The re-resolve grows the surface, so
    // accept the delta to let the lock update through.
    let (_, _, before) = lock_trust(&app, "b");
    add_unsafe_region_to_b(&root.join("audit_b"));
    std::env::set_var("CANDOR_ACCEPT_TRUST_DELTA", "1");
    let res = resolve_pkg::resolve(&app);
    std::env::remove_var("CANDOR_ACCEPT_TRUST_DELTA");
    res.unwrap();
    let (_, _, after) = lock_trust(&app, "b");
    assert_eq!(after, before + 1, "b's recorded unsafe_regions must track the new region");
}

#[test]
fn dependency_free_package_writes_no_lock() {
    // A manifest-carrying but dependency-free package resolves nothing and takes
    // the single-package path — no `candor.lock`, unchanged from before this slice.
    let root = stage(&["audit_c"]);
    let c = root.join("audit_c");
    assert!(check_dir(&c).unwrap().is_empty(), "dep-free package checks clean: {:?}", codes_at(&c));
    assert!(!c.join("candor.lock").exists(), "a dep-free package writes no lock");
}

/// The `(boundary_modules, externs, unsafe_regions)` recorded for package `name`
/// in the root's `candor.lock` — the per-package trust summary (design 0017 §8).
fn lock_trust(app: &std::path::Path, name: &str) -> (i64, i64, i64) {
    let text = std::fs::read_to_string(app.join("candor.lock")).expect("candor.lock written");
    let doc: toml::Value = toml::from_str(&text).expect("candor.lock is TOML");
    let entry = doc["package"]
        .as_array()
        .expect("package array")
        .iter()
        .find(|p| p["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("no lock entry for `{name}`:\n{text}"));
    let t = &entry["trust"];
    let n = |k: &str| t[k].as_integer().unwrap_or_else(|| panic!("trust.{k} missing:\n{text}"));
    (n("boundary_modules"), n("externs"), n("unsafe_regions"))
}

/// Append one more `unsafe` region to the `audit_b` fixture's public root, growing
/// its trust surface by exactly one region (a pure source edit; only parsing feeds
/// the trust count, so it need not type-check).
fn add_unsafe_region_to_b(pkg_dir: &std::path::Path) {
    let f = pkg_dir.join("src/b.cnr");
    let mut src = std::fs::read_to_string(&f).unwrap();
    src.push_str(
        "\npub fn b_read_again(p: rawptr u8) -> usize {\n    \
         unsafe \"a second region added to grow the trust surface\" {\n        \
         return b_native_read(p);\n    }\n}\n",
    );
    std::fs::write(&f, src).unwrap();
}

// ---- whole-graph audit: a dependency's foreign + unsafe surface is aggregated
// and attributed to it (design 0017 §8; review F1b) --------------------------
//
// `audit_app` -> `b` -> `c`: `candor audit audit_app` walks the WHOLE resolved
// graph. Each package's trust surface is enumerated and tagged with its name,
// version, and source, so a dependency's `foreign` externs and `unsafe` regions
// are visible and traceable to it — a dep (and a dep-of-a-dep) cannot hide I/O.

#[test]
fn audit_aggregates_dependency_trust_surface_across_graph() {
    let root = stage(&["audit_app", "audit_b", "audit_c"]);
    let app = root.join("audit_app");
    let got = candor_proto::audit::audit_path(&app).expect("graph audit succeeds");
    let doc: serde_json::Value = serde_json::from_str(&got).expect("audit emits JSON");

    let packages = doc["packages"].as_array().expect("packages array");
    assert_eq!(packages.len(), 3, "root + b + c are all enumerated:\n{got}");
    let find = |name: &str| {
        packages
            .iter()
            .find(|p| p["package"] == name)
            .unwrap_or_else(|| panic!("missing package `{name}`:\n{got}"))
    };

    // The root package is attributed and flagged; its surface is also the report's
    // top level (the backward-compatible single-package shape).
    let app_pkg = find("audit_app");
    assert_eq!(app_pkg["is_root"], true);
    assert_eq!(doc["boundary_modules"], app_pkg["boundary_modules"]);
    assert_eq!(doc["summary"], app_pkg["summary"]);

    // b: its `foreign` extern AND its `unsafe` region, attributed to b
    // (name + version + source).
    let b = find("b");
    assert_eq!(b["is_root"], false);
    assert_eq!(b["version"], "0.2.0");
    assert_eq!(b["source"]["kind"], "path");
    assert!(
        b["source"]["path"].as_str().unwrap().ends_with("audit_b"),
        "b's source is its canonical path:\n{got}"
    );
    assert!(has_foreign_extern(b, "b_native_read"), "b's foreign extern:\n{got}");
    assert!(has_unsafe_fn(b, "b::b_read"), "b's unsafe region:\n{got}");
    assert_eq!(b["summary"]["externs"], 1);
    assert_eq!(b["summary"]["unsafe_regions"], 1);

    // Transitive depth: c is a dependency of b, yet its surface still appears,
    // attributed to c — a dep-of-a-dep cannot hide.
    let c = find("c");
    assert_eq!(c["is_root"], false);
    assert_eq!(c["version"], "0.3.0");
    assert!(c["source"]["path"].as_str().unwrap().ends_with("audit_c"), "c source:\n{got}");
    assert!(has_foreign_extern(c, "c_native_write"), "c's foreign extern:\n{got}");
    assert!(has_unsafe_fn(c, "c::c_write"), "c's unsafe region:\n{got}");
}

// ---- backward compat: a dependency-free package audits with the unchanged
// single-package shape (no `packages` layer) --------------------------------

#[test]
fn audit_of_dependency_free_package_is_single_package_shape() {
    let root = stage(&["audit_c"]);
    let got = candor_proto::audit::audit_path(&root.join("audit_c")).expect("audit succeeds");
    let doc: serde_json::Value = serde_json::from_str(&got).expect("audit emits JSON");
    assert!(doc.get("packages").is_none(), "no graph layer for a dep-free package:\n{got}");
    assert_eq!(doc["summary"]["externs"], 1);
    assert_eq!(doc["summary"]["unsafe_regions"], 1);
}

/// Whether any of `pkg`'s boundary modules declares a `foreign` extern named `name`.
fn has_foreign_extern(pkg: &serde_json::Value, name: &str) -> bool {
    pkg["boundary_modules"].as_array().unwrap().iter().any(|m| {
        m["externs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["name"] == name && e["foreign"] == true)
    })
}

/// Whether `pkg` records an `unsafe` region in the module-qualified function `func`.
fn has_unsafe_fn(pkg: &serde_json::Value, func: &str) -> bool {
    pkg["unsafe_regions"].as_array().unwrap().iter().any(|u| u["function"] == func)
}

fn codes_at(dir: &std::path::Path) -> Vec<String> {
    match check_dir(dir) {
        Ok(diags) => diags.into_iter().map(|d| d.code).collect(),
        Err(d) => vec![d.code],
    }
}

// ---------------------------------------------------------------------------
// Git dependency fetch (design 0017 §4/§6). Hermetic + offline: each test builds
// a LOCAL git repo in a temp dir (no network), points `CANDOR_CACHE_DIR` at an
// isolated temp cache, and depends on it via `{ git = <file url>, rev = <sha> }`.
// If `git` cannot be spawned (a locked-down sandbox), the tests skip cleanly.
// ---------------------------------------------------------------------------

use std::process::Command;
use std::sync::Mutex;

// `CANDOR_CACHE_DIR` is process-global; serialize the git tests so a set_var in
// one never races another (a no-op under nextest's process-per-test, a guard
// under the plain `cargo test` harness).
static GIT_ENV_LOCK: Mutex<()> = Mutex::new(());

fn git_available() -> bool {
    Command::new("git").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

/// Run `git` in `repo` with a self-contained identity (so a commit succeeds even
/// with no global git config), failing loudly on error.
fn git(repo: &std::path::Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["-c", "user.name=candor-test", "-c", "user.email=test@candor.invalid", "-c", "commit.gpgsign=false"])
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Stage `fixture` as a fresh local git repo committed on branch `main`; return
/// its (repo path, `file://` url, commit sha).
fn init_git_repo(name: &str, fixture: &str) -> (PathBuf, String, String) {
    let n = STAGE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let repo = std::env::temp_dir().join(format!("candor_gitrepo_{}_{}_{}", std::process::id(), name, n));
    let _ = std::fs::remove_dir_all(&repo);
    copy_tree(&dir(fixture), &repo);
    git(&repo, &["init", "-q", "-b", "main"]);
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "package"]);
    let out = Command::new("git").arg("-C").arg(&repo).args(["rev-parse", "HEAD"]).output().expect("git rev-parse");
    let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let url = format!("file://{}", repo.display());
    (repo, url, sha)
}

/// Create an `app` package whose single dependency `b` is the given git source.
fn stage_git_app(cache: &std::path::Path, dep_toml: &str) -> PathBuf {
    let n = STAGE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let app = std::env::temp_dir().join(format!("candor_gitapp_{}_{}", std::process::id(), n));
    let _ = std::fs::remove_dir_all(&app);
    std::fs::create_dir_all(app.join("src")).unwrap();
    std::fs::write(
        app.join("candor.toml"),
        format!("[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2026\"\n\n[dependencies]\n{dep_toml}\n"),
    )
    .unwrap();
    std::fs::write(app.join("src/main.cnr"), "use b::{value};\n\nfn main() -> i64 {\n    return value();\n}\n").unwrap();
    let _ = cache; // cache root is passed via env by the caller.
    app
}

fn fresh_cache() -> PathBuf {
    let n = STAGE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let cache = std::env::temp_dir().join(format!("candor_gitcache_{}_{}", std::process::id(), n));
    let _ = std::fs::remove_dir_all(&cache);
    cache
}

// ---- the milestone: `app` depends on `b` via git, fetches, builds+runs, and a
// second build reuses the content-addressed cache (no re-clone) ---------------

#[test]
fn git_dependency_fetches_builds_reuses_cache_and_locks_sha() {
    if !git_available() {
        eprintln!("SKIP git_dependency_*: `git` is not spawnable in this sandbox");
        return;
    }
    let _guard = GIT_ENV_LOCK.lock().unwrap();

    let (repo, url, sha) = init_git_repo("b", "b");
    let cache = fresh_cache();
    std::env::set_var("CANDOR_CACHE_DIR", &cache);

    let app = stage_git_app(&cache, &format!("b = {{ git = \"{url}\", rev = \"{sha}\" }}"));

    // First build: fetches b into the temp cache and runs across every engine.
    assert!(check_dir(&app).unwrap().is_empty(), "git app should check clean: {:?}", codes_at(&app));
    assert_all_engines(&app, 123);

    // The checkout landed in the content-addressed cache, keyed by url + sha.
    let src_cache = cache.join("git-src");
    let checkout = std::fs::read_dir(&src_cache)
        .expect("git-src cache exists")
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().ends_with(&sha))
        .expect("a checkout keyed by the resolved sha");
    assert!(checkout.path().join("candor.toml").is_file(), "checkout is a pristine package");
    assert!(!checkout.path().join(".git").exists(), "checkout is plain read-only source (no .git)");

    // candor.lock records b's git url + resolved sha + a content hash.
    let lock = std::fs::read_to_string(app.join("candor.lock")).expect("candor.lock written");
    assert!(lock.contains(&url), "lock records the git url:\n{lock}");
    assert!(lock.contains(&sha), "lock records the resolved sha:\n{lock}");
    assert!(lock.contains("content_hash"), "lock records a content hash:\n{lock}");

    // Second build REUSES the cache: delete the source repo AND the mirror db, so
    // only the content-addressed checkout remains. A successful build proves no
    // re-clone/re-fetch happened.
    std::fs::remove_dir_all(&repo).unwrap();
    std::fs::remove_dir_all(cache.join("git-db")).unwrap();
    match run_dir(&app) {
        RunResult::Ok(r) => assert_eq!(r.ret, 123, "second build reuses the cached checkout"),
        other => panic!("cache reuse build did not run: {}", describe(other)),
    }

    std::env::remove_var("CANDOR_CACHE_DIR");
}

// ---- a tag/branch dependency resolves to and locks the underlying commit sha --

#[test]
fn git_dependency_tag_resolves_to_locked_sha() {
    if !git_available() {
        eprintln!("SKIP git_dependency_*: `git` is not spawnable in this sandbox");
        return;
    }
    let _guard = GIT_ENV_LOCK.lock().unwrap();

    let (repo, url, sha) = init_git_repo("b", "b");
    git(&repo, &["tag", "v1.0"]);
    let cache = fresh_cache();
    std::env::set_var("CANDOR_CACHE_DIR", &cache);

    // The manifest writes the tag for convenience; resolution goes through the tag
    // and the lock pins to the commit sha it points at (design 0017 §4).
    let app = stage_git_app(&cache, &format!("b = {{ git = \"{url}\", rev = \"{sha}\", tag = \"v1.0\" }}"));
    assert!(check_dir(&app).unwrap().is_empty(), "tag-pinned app checks clean: {:?}", codes_at(&app));
    match run_dir(&app) {
        RunResult::Ok(r) => assert_eq!(r.ret, 123),
        other => panic!("tag-pinned app did not run: {}", describe(other)),
    }
    let lock = std::fs::read_to_string(app.join("candor.lock")).expect("candor.lock written");
    assert!(lock.contains(&sha), "the tag resolved to the pinned commit sha in the lock:\n{lock}");

    let _ = repo;
    std::env::remove_var("CANDOR_CACHE_DIR");
}

// ---- a bad git source fails with a clear diagnostic, not a panic ------------

#[test]
fn git_dependency_bad_source_is_a_clean_error() {
    if !git_available() {
        eprintln!("SKIP git_dependency_*: `git` is not spawnable in this sandbox");
        return;
    }
    let _guard = GIT_ENV_LOCK.lock().unwrap();

    let cache = fresh_cache();
    std::env::set_var("CANDOR_CACHE_DIR", &cache);
    let missing = std::env::temp_dir().join(format!("candor_nope_{}", std::process::id()));
    let app = stage_git_app(&cache, &format!("b = {{ git = \"file://{}\", rev = \"{}\" }}", missing.display(), "0".repeat(40)));
    let cs = codes_at(&app);
    assert!(cs.contains(&"E0932".to_string()), "want E0932 clone failure, got {cs:?}");

    std::env::remove_var("CANDOR_CACHE_DIR");
}

// ---- whole-graph audit surfaces a git dependency's trust surface ------------

#[test]
fn git_dependency_audit_surfaces_trust_surface() {
    if !git_available() {
        eprintln!("SKIP git_dependency_*: `git` is not spawnable in this sandbox");
        return;
    }
    let _guard = GIT_ENV_LOCK.lock().unwrap();

    // `audit_c` is a dependency-free package with a `foreign` extern and an
    // `unsafe` region — a self-contained trust surface to fetch over git.
    let (_repo, url, sha) = init_git_repo("audit_c", "audit_c");
    let cache = fresh_cache();
    std::env::set_var("CANDOR_CACHE_DIR", &cache);

    let n = STAGE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let app = std::env::temp_dir().join(format!("candor_gitauditapp_{}_{}", std::process::id(), n));
    let _ = std::fs::remove_dir_all(&app);
    std::fs::create_dir_all(app.join("src")).unwrap();
    std::fs::write(
        app.join("candor.toml"),
        format!("[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2026\"\n\n[dependencies]\nc = {{ git = \"{url}\", rev = \"{sha}\" }}\n"),
    )
    .unwrap();
    std::fs::write(app.join("src/main.cnr"), "fn main() -> i64 {\n    return 0;\n}\n").unwrap();

    let got = candor_proto::audit::audit_path(&app).expect("git-dep graph audit succeeds");
    let doc: serde_json::Value = serde_json::from_str(&got).expect("audit emits JSON");
    let packages = doc["packages"].as_array().expect("packages array");
    let c = packages
        .iter()
        .find(|p| p["package"] == "c")
        .unwrap_or_else(|| panic!("git dep `c` enumerated:\n{got}"));
    assert_eq!(c["source"]["kind"], "git", "c is attributed to its git source:\n{got}");
    assert_eq!(c["source"]["url"], url, "git source url:\n{got}");
    assert_eq!(c["source"]["rev"], sha, "git source pinned sha:\n{got}");
    assert!(has_foreign_extern(c, "c_native_write"), "git dep's foreign extern surfaces:\n{got}");
    assert!(has_unsafe_fn(c, "c::c_write"), "git dep's unsafe region surfaces:\n{got}");

    std::env::remove_var("CANDOR_CACHE_DIR");
}

// ---------------------------------------------------------------------------
// F5 (review 0017 §8): freestanding composition rejects any transitive
// `boundary`/`foreign` surface (0011 §5) and any transitive `std` import (0008
// §5) across the resolved graph, gated on the ROOT package's `freestanding`
// claim. The check runs post-resolution over the pinned package set and reuses
// the audit's structural boundary enumeration (single source of truth).
// ---------------------------------------------------------------------------

/// The hard error a failed resolution/composition raises (its `Err` diagnostic),
/// or `None` if the package resolved and checked. Only a resolver-level failure
/// (E0935 here) surfaces as `Err`; per-module check diagnostics surface as `Ok`.
fn resolve_error(dir: &std::path::Path) -> Option<candor_proto::diag::Diag> {
    check_dir(dir).err()
}

// The load-bearing F5 escape (`blink -> hal`): the freestanding root pulls in a
// dependency whose `boundary` module declares a `foreign` extern that is never
// called. Resolution must fail, naming the offending package + the extern.
#[test]
fn freestanding_rejects_transitive_declared_but_uncalled_foreign_extern() {
    let root = stage(&["blink", "hal"]);
    let blink = root.join("blink");
    let diag = resolve_error(&blink)
        .expect("freestanding blink over hal's foreign surface must be rejected");
    assert_eq!(diag.code, "E0935", "want the freestanding-composition code, got {diag:?}");
    let text = format!("{} {}", diag.message, diag.to_json());
    assert!(text.contains("hal"), "diagnostic must name the offending package `hal`: {text}");
    assert!(
        text.contains("hal_write"),
        "diagnostic must name the declared-but-uncalled extern `hal_write`: {text}"
    );
}

// A transitive `std` import (0008 §5): the `std` package appears in a
// freestanding root's resolved graph. Rejected with the same code.
#[test]
fn freestanding_rejects_transitive_std_import() {
    let root = stage(&["fs_std", "stdpkg"]);
    let app = root.join("fs_std");
    let diag =
        resolve_error(&app).expect("freestanding fs_std importing the `std` package must be rejected");
    assert_eq!(diag.code, "E0935", "want the freestanding-composition code, got {diag:?}");
    assert!(
        format!("{} {}", diag.message, diag.to_json()).contains("std"),
        "diagnostic must name the `std` package: {diag:?}"
    );
}

// Regression guard: a freestanding root over only pure, core-only dependencies
// (no boundary/foreign, no std) composes clean and runs across every engine.
#[test]
fn freestanding_over_pure_deps_composes_clean() {
    let root = stage(&["fs_ok", "purelib"]);
    let app = root.join("fs_ok");
    assert!(
        check_dir(&app).unwrap().is_empty(),
        "legit freestanding composition must check clean: {:?}",
        codes_at(&app)
    );
    assert_all_engines(&app, 42);
}

// A NORMAL (non-freestanding) root over the same foreign-boundary dependency
// must build exactly as before — the composition check is gated on the root's
// flag and must not touch it.
#[test]
fn non_freestanding_root_over_foreign_dep_is_unaffected() {
    let root = stage(&["hosted_app", "hal"]);
    let app = root.join("hosted_app");
    assert!(
        check_dir(&app).unwrap().is_empty(),
        "non-freestanding root over a foreign dep must check clean: {:?}",
        codes_at(&app)
    );
    assert_eq!(native_ret(run_dir_mir(&app)), 0, "non-freestanding root still runs");
}

// ---- the trust-delta gate on lock updates (design 0017 §8, Open-Q1) ---------
// A lock update that GROWS a dependency's foreign/`unsafe` surface (a per-dep
// count increase, or a new dependency carrying nonzero surface) is a hard error
// (E0936) naming the offending dep + the delta, and must NOT overwrite the
// reviewed lock. Initial creation, a no-change rebuild, a shrunk/equal surface,
// and a new pure dependency are not gated. `CANDOR_ACCEPT_TRUST_DELTA` overrides.

// The override is a process-global env var; serialize the gate tests that depend
// on its state (the grow/new-foreign tests need it UNSET; the override test sets
// then clears it) so a set_var in one never races another. A no-op under
// nextest's process-per-test, a guard under the plain `cargo test` harness — the
// same pattern the git tests use for `CANDOR_CACHE_DIR`.
static TRUST_ENV_LOCK: Mutex<()> = Mutex::new(());

/// Create a fresh binary `gate_app` package depending on the given `(dep-key,
/// sibling-dir)` path dependencies; return its directory. Rewriting the manifest
/// later re-points the graph (adding/removing a dependency between resolves).
fn scaffold_app(root: &std::path::Path, deps: &[(&str, &str)]) -> PathBuf {
    let app = root.join("gate_app");
    std::fs::create_dir_all(app.join("src")).unwrap();
    std::fs::write(app.join("src/main.cnr"), "fn main() -> i64 {\n    return 0;\n}\n").unwrap();
    write_app_manifest(&app, deps);
    app
}

/// (Re)write `gate_app`'s manifest with the given path dependencies.
fn write_app_manifest(app: &std::path::Path, deps: &[(&str, &str)]) {
    let mut s = String::from("[package]\nname = \"gate_app\"\nversion = \"0.1.0\"\nedition = \"2026\"\n\n[dependencies]\n");
    for (key, sib) in deps {
        s.push_str(&format!("{key} = {{ path = \"../{sib}\" }}\n"));
    }
    std::fs::write(app.join("candor.toml"), s).unwrap();
}

// LOAD-BEARING: growing a dependency's surface fails the lock update with E0936
// (naming the dep + the delta) and leaves the reviewed lock byte-for-byte intact.
#[test]
fn trust_growth_fails_the_lock_update_and_leaves_the_lock_intact() {
    let _guard = TRUST_ENV_LOCK.lock().unwrap();
    let root = stage(&["audit_app", "audit_b", "audit_c"]);
    let app = root.join("audit_app");

    // Baseline: establish the reviewed lock (b: 1 unsafe region).
    resolve_pkg::resolve(&app).unwrap();
    let baseline = std::fs::read_to_string(app.join("candor.lock")).unwrap();

    // Grow b's surface by one `unsafe` region and re-resolve.
    add_unsafe_region_to_b(&root.join("audit_b"));
    let err = resolve_pkg::resolve(&app).err().expect("the grown surface must be gated");

    assert_eq!(err.code, "E0936", "want the trust-delta gate code, got {err:?}");
    assert!(err.message.contains("b:"), "must name the offending dep `b`: {}", err.message);
    assert!(
        err.message.contains("unsafe_regions 1 -> 2"),
        "must show the per-dep delta: {}",
        err.message,
    );
    assert!(
        err.message.contains("CANDOR_ACCEPT_TRUST_DELTA"),
        "must document the override in the help text: {}",
        err.message,
    );
    assert_eq!(
        std::fs::read_to_string(app.join("candor.lock")).unwrap(),
        baseline,
        "the gate must NOT overwrite the reviewed lock on failure",
    );
}

// Accepting the delta via the override lets the update proceed and rewrites the
// lock with the grown counts.
#[test]
fn accepting_the_trust_delta_rewrites_the_lock() {
    let _guard = TRUST_ENV_LOCK.lock().unwrap();
    let root = stage(&["audit_app", "audit_b", "audit_c"]);
    let app = root.join("audit_app");

    resolve_pkg::resolve(&app).unwrap();
    add_unsafe_region_to_b(&root.join("audit_b"));

    std::env::set_var("CANDOR_ACCEPT_TRUST_DELTA", "1");
    let res = resolve_pkg::resolve(&app);
    std::env::remove_var("CANDOR_ACCEPT_TRUST_DELTA");
    res.expect("the override must let the grown lock update proceed");

    assert_eq!(lock_trust(&app, "b"), (1, 1, 2), "the grown surface is now recorded in the lock");
}

// A NEW dependency carrying foreign/`unsafe` surface is gated (more attack
// surface than the committed lock vouched for).
#[test]
fn a_new_dependency_with_foreign_surface_is_gated() {
    let _guard = TRUST_ENV_LOCK.lock().unwrap();
    let root = stage(&["purelib", "audit_b", "audit_c"]);
    let app = scaffold_app(&root, &[("purelib", "purelib")]);

    // Baseline: a pure graph (app + purelib), zero surface.
    resolve_pkg::resolve(&app).unwrap();
    let baseline = std::fs::read_to_string(app.join("candor.lock")).unwrap();

    // Add `b` (a foreign extern + an `unsafe` region) as a new dependency.
    write_app_manifest(&app, &[("purelib", "purelib"), ("b", "audit_b")]);
    let err = resolve_pkg::resolve(&app).err().expect("a new foreign dep must be gated");

    assert_eq!(err.code, "E0936", "a new foreign dep must be gated, got {err:?}");
    assert!(err.message.contains("b (new dependency)"), "must name the new dep: {}", err.message);
    assert_eq!(
        std::fs::read_to_string(app.join("candor.lock")).unwrap(),
        baseline,
        "lock untouched on the gated new-dep update",
    );
}

// A NEW dependency with zero foreign/`unsafe` surface is NOT gated — it adds no
// new trust.
#[test]
fn a_new_dependency_with_no_foreign_surface_is_not_gated() {
    let root = stage(&["purelib", "lib_pkg"]);
    let app = scaffold_app(&root, &[("purelib", "purelib")]);

    resolve_pkg::resolve(&app).unwrap();
    write_app_manifest(&app, &[("purelib", "purelib"), ("lib_pkg", "lib_pkg")]);
    resolve_pkg::resolve(&app).expect("a pure new dependency must not be gated");

    assert_eq!(lock_trust(&app, "lib_pkg"), (0, 0, 0), "the new pure dep is recorded with zero surface");
}

// A SHRINKING (or equal) surface is not gated: a dependency whose counts drop
// re-resolves clean and the lock is rewritten with the smaller counts.
#[test]
fn a_shrinking_trust_surface_is_not_gated() {
    let root = stage(&["audit_app", "audit_b", "audit_c"]);
    let app = root.join("audit_app");
    let b_src = root.join("audit_b/src/b.cnr");
    let original = std::fs::read_to_string(&b_src).unwrap();

    // Baseline with a grown surface (b: 2 unsafe regions) — an initial creation,
    // so writing it is not gated.
    add_unsafe_region_to_b(&root.join("audit_b"));
    resolve_pkg::resolve(&app).unwrap();
    assert_eq!(lock_trust(&app, "b"), (1, 1, 2), "baseline records the grown surface");

    // Shrink b back to one region and re-resolve: a drop is not gated.
    std::fs::write(&b_src, &original).unwrap();
    resolve_pkg::resolve(&app).expect("a shrinking surface must not be gated");
    assert_eq!(lock_trust(&app, "b"), (1, 1, 1), "the shrunk surface is rewritten");
}

// Regression guards: initial creation (no lock on disk) and a no-change rebuild
// (fresh == on-disk) never reach the gate.
#[test]
fn initial_creation_and_no_change_rebuild_are_not_gated() {
    let root = stage(&["audit_app", "audit_b", "audit_c"]);
    let app = root.join("audit_app");

    // Initial creation: the staged copy carries no lock — establishing the
    // baseline writes it and is not gated.
    let first = resolve_pkg::resolve(&app).unwrap();
    assert!(!first.lock_reused, "the initial resolve writes a fresh lock");
    assert!(app.join("candor.lock").exists(), "the baseline lock is written");

    // No-change rebuild: the fresh set equals the on-disk lock — reused, never
    // reaching the gate.
    let second = resolve_pkg::resolve(&app).unwrap();
    assert!(second.lock_reused, "an unchanged re-resolve reuses the lock");
}
