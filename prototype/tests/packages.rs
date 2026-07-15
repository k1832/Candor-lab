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
use candor_proto::{compile_path, resolve_pkg, run_dir_mir, run_dir_native, MirRunResult};

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
