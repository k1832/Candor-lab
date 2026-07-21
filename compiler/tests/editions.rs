//! Edition + migrator gate (1.0-gate item 1, docs/1.0-GATE-TRIAGE.md row 1) — the
//! P15 promise that "the evolution mechanism works before it is relied on".
//!
//! Exercises the machinery end-to-end with the REHEARSAL edition transition
//! (`2026` -> `2027-rehearsal`), whose one synthetic breaking change is a keyword
//! rename (`mut` -> `mutable`) with byte-identical semantics (0017 F4). This is
//! NOT a shipped language change (see `manifest::REHEARSAL_EDITION`); it exists to
//! prove the edition/migrator machinery works before a real edition relies on it.
//!
//! What it pins:
//!   * the old edition still compiles + runs (retained old-spelling front-end),
//!   * the new edition rejects the old spelling and accepts only the new one
//!     (it IS a breaking change), and the old edition rejects the new spelling,
//!   * the migrator is automatic, idempotent, and byte-identical across engines,
//!   * cross-edition dependencies link + run in BOTH directions (0017 F4),
//!   * an unknown edition still errors.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use candor::{
    check_dir, compile_path, migrate_edition_dir, migrate_edition_source, run_dir, run_dir_mir,
    run_dir_native, MirRunResult, RunResult,
};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A fresh, empty temp root unique to this process + call.
fn fresh_root(tag: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir().join(format!("candor_ed_{}_{}_{}", std::process::id(), tag, n));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

/// Write a package (manifest + `src/` files) at `dir`. `extra_toml` appends stanzas
/// (`[lib]`, `[dependencies]`, ...) after `[package]`.
fn write_pkg(dir: &Path, name: &str, edition: &str, extra_toml: &str, files: &[(&str, &str)]) {
    std::fs::create_dir_all(dir.join("src")).unwrap();
    let toml = format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"{edition}\"\n{extra_toml}"
    );
    std::fs::write(dir.join("candor.toml"), toml).unwrap();
    for (rel, body) in files {
        let path = dir.join("src").join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }
}

fn codes(dir: &Path) -> Vec<String> {
    match check_dir(dir) {
        Ok(diags) => diags.into_iter().map(|d| d.code).collect(),
        Err(d) => vec![d.code],
    }
}

fn mir_ret(r: MirRunResult) -> i64 {
    match r {
        MirRunResult::Ok(run) => run.ret,
        MirRunResult::Fault(f) => panic!("engine fault: {}", f.to_json()),
        MirRunResult::CheckErrors(d) => {
            panic!("engine check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        MirRunResult::ParseError(d) => panic!("engine parse error: {}", d.to_json()),
        MirRunResult::Unsupported(m) => panic!("engine unsupported: {m}"),
    }
}

fn tree_ret(dir: &Path) -> i64 {
    match run_dir(dir) {
        RunResult::Ok(r) => r.ret,
        RunResult::Fault(f) => panic!("tree fault: {}", f.to_json()),
        RunResult::CheckErrors(d) => {
            panic!("tree check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("tree parse error: {}", d.to_json()),
    }
}

/// Every engine's return for a package: (tree, MIR, native, AOT). The load-bearing
/// cross-edition invariant is that these agree — surface-only edition changes are
/// byte-identical semantics (0017 F4 / NN#14).
fn all_engine_returns(dir: &Path) -> (i64, i64, i64, i64) {
    let tree = tree_ret(dir);
    let mir = mir_ret(run_dir_mir(dir));
    let native = mir_ret(run_dir_native(dir));
    let out = dir.join("prog.out");
    compile_path(dir, &out).unwrap_or_else(|e| panic!("AOT compile failed: {e}"));
    let status = std::process::Command::new(&out).status().unwrap();
    let aot = status.code().expect("AOT executable exited") as i64;
    (tree, mir, native, aot)
}

/// Assert every engine returns `want` (AOT compares the low byte, the process exit).
fn assert_all_engines(dir: &Path, want: i64) {
    let (tree, mir, native, aot) = all_engine_returns(dir);
    assert_eq!(tree, want, "tree-walker");
    assert_eq!(mir, want, "MIR engine");
    assert_eq!(native, want, "native engine");
    assert_eq!(aot, want & 0xff, "AOT executable exit");
}

const MUT_MAIN_2026: &str =
    "fn main() -> i64 {\n    let mut x: i64 = 40;\n    x = x + 2;\n    return x;\n}\n";
const MUTABLE_MAIN_2027: &str =
    "fn main() -> i64 {\n    let mutable x: i64 = 40;\n    x = x + 2;\n    return x;\n}\n";

// ---- old edition still compiles + runs (retained old-spelling front-end) ----

#[test]
fn old_edition_compiles_and_runs_across_engines() {
    let root = fresh_root("old");
    write_pkg(&root, "old", "2026", "", &[("main.cnr", MUT_MAIN_2026)]);
    assert!(codes(&root).is_empty(), "2026 `mut` must check clean, got {:?}", codes(&root));
    assert_all_engines(&root, 42);
}

// ---- new edition accepts ONLY the new spelling (it IS a breaking change) -----

#[test]
fn new_edition_accepts_new_spelling() {
    let root = fresh_root("new_ok");
    write_pkg(&root, "newok", "2027-rehearsal", "", &[("main.cnr", MUTABLE_MAIN_2027)]);
    assert!(codes(&root).is_empty(), "2027 `mutable` must check clean, got {:?}", codes(&root));
    assert_all_engines(&root, 42);
}

#[test]
fn new_edition_rejects_old_spelling() {
    // The breaking change: `mut` is not a keyword in 2027-rehearsal, so `let mut x`
    // is a parse error (a bare `let <name=mut>` then an unexpected `x`).
    let root = fresh_root("new_bad");
    write_pkg(&root, "newbad", "2027-rehearsal", "", &[("main.cnr", MUT_MAIN_2026)]);
    let cs = codes(&root);
    assert!(
        cs.iter().any(|c| c == "P0001"),
        "2027 must reject the old `mut` spelling with a parse error, got {cs:?}",
    );
}

#[test]
fn old_edition_rejects_new_spelling() {
    // The fork is real in both directions: `mutable` is an ordinary identifier
    // under 2026, so `let mutable x` fails to parse there.
    let root = fresh_root("old_bad");
    write_pkg(&root, "oldbad", "2026", "", &[("main.cnr", MUTABLE_MAIN_2027)]);
    let cs = codes(&root);
    assert!(
        cs.iter().any(|c| c == "P0001"),
        "2026 must reject the new `mutable` spelling with a parse error, got {cs:?}",
    );
}

// ---- an unknown edition still errors -----------------------------------------

#[test]
fn unknown_edition_is_rejected() {
    let root = fresh_root("unk");
    write_pkg(&root, "unk", "1999", "", &[("main.cnr", "fn main() -> i64 {\n    return 0;\n}\n")]);
    let cs = codes(&root);
    assert!(cs.iter().any(|c| c == "M0102"), "unknown edition must be M0102, got {cs:?}");
}

// ---- the migrator: automatic, idempotent, byte-identical across engines -------

#[test]
fn migrator_is_automatic_idempotent_and_byte_identical() {
    let root = fresh_root("mig");
    // A 2026 package whose `mut` keyword AND a `mut` inside a comment coexist, to
    // pin that the token-driven migrator rewrites only the keyword.
    let src = "fn main() -> i64 {\n    // keep this mut word in the comment\n    let mut x: i64 = 40;\n    x = x + 2;\n    return x;\n}\n";
    write_pkg(&root, "mig", "2026", "", &[("main.cnr", src)]);

    // Baseline: every engine agrees before migration.
    let before = all_engine_returns(&root);
    assert_eq!(before, (42, 42, 42, 42 & 0xff), "2026 baseline across engines");

    // Migrate: automatic, reports the bump + the one rewritten file.
    let m = migrate_edition_dir(&root).expect("migration succeeds");
    assert!(m.manifest_bumped, "the manifest edition is bumped");
    assert_eq!(m.files_rewritten.len(), 1, "exactly the one source file is rewritten");

    // The manifest now declares the rehearsal edition (other bytes preserved).
    let manifest = std::fs::read_to_string(root.join("candor.toml")).unwrap();
    assert!(manifest.contains("edition = \"2027-rehearsal\""), "manifest bumped: {manifest}");
    assert!(manifest.contains("name = \"mig\""), "manifest otherwise preserved: {manifest}");

    // The source now uses the new spelling; the comment's `mut` is untouched.
    let migrated = std::fs::read_to_string(root.join("src/main.cnr")).unwrap();
    let expected = "fn main() -> i64 {\n    // keep this mut word in the comment\n    let mutable x: i64 = 40;\n    x = x + 2;\n    return x;\n}\n";
    assert_eq!(migrated, expected, "byte-exact migrator output (comment `mut` preserved)");

    // Byte-identical semantics: the migrated (2027) package returns the same on
    // every engine as the original (2026) package did.
    assert!(codes(&root).is_empty(), "migrated package checks clean, got {:?}", codes(&root));
    let after = all_engine_returns(&root);
    assert_eq!(after, before, "every engine returns identically after migration (0017 F4)");

    // Idempotent: a second migration is a no-op, and the bytes are unchanged.
    let again = migrate_edition_dir(&root).expect("re-migration succeeds");
    assert!(!again.manifest_bumped, "re-run does not bump the manifest");
    assert!(again.files_rewritten.is_empty(), "re-run rewrites nothing");
    assert_eq!(
        std::fs::read_to_string(root.join("src/main.cnr")).unwrap(),
        expected,
        "the source is byte-identical after an idempotent re-run",
    );
}

#[test]
fn migrate_edition_source_is_idempotent() {
    // Applying the source rewrite twice equals applying it once (the keyword is
    // already respelled and lexes as an identifier under 2026 on the second pass).
    let once = migrate_edition_source(MUT_MAIN_2026).unwrap();
    let twice = migrate_edition_source(&once).unwrap();
    assert_eq!(once, MUTABLE_MAIN_2027);
    assert_eq!(twice, once, "migrate_edition_source is idempotent");
}

// ---- cross-edition dependencies link + run in BOTH directions (0017 F4) -------

#[test]
fn cross_edition_2026_app_depends_on_2027_lib() {
    let root = fresh_root("xA");
    // 2027-rehearsal library, using the new `mutable` spelling in its body.
    write_pkg(
        &root.join("mlib"),
        "mlib",
        "2027-rehearsal",
        "\n[lib]\n",
        &[("mlib.cnr", "pub fn contribute() -> i64 {\n    let mutable acc: i64 = 0;\n    acc = acc + 100;\n    return acc;\n}\n")],
    );
    // 2026 application using the old `mut` spelling, depending on the 2027 lib.
    write_pkg(
        &root.join("app"),
        "app",
        "2026",
        "\n[dependencies]\nmlib = { path = \"../mlib\" }\n",
        &[("main.cnr", "use mlib::{contribute};\n\nfn main() -> i64 {\n    let mut x: i64 = contribute();\n    x = x + 23;\n    return x;\n}\n")],
    );
    let app = root.join("app");
    assert!(check_dir(&app).unwrap().is_empty(), "cross-edition A checks clean: {:?}", codes(&app));
    // contribute() (100, parsed under 2027) + 23 (parsed under 2026) = 123.
    assert_all_engines(&app, 123);
}

#[test]
fn cross_edition_2027_app_depends_on_2026_lib() {
    let root = fresh_root("xB");
    // 2026 library, using the old `mut` spelling.
    write_pkg(
        &root.join("nlib"),
        "nlib",
        "2026",
        "\n[lib]\n",
        &[("nlib.cnr", "pub fn base() -> i64 {\n    let mut b: i64 = 0;\n    b = b + 42;\n    return b;\n}\n")],
    );
    // 2027-rehearsal application using the new `mutable` spelling, over the 2026 lib.
    write_pkg(
        &root.join("app"),
        "app",
        "2027-rehearsal",
        "\n[dependencies]\nnlib = { path = \"../nlib\" }\n",
        &[("main.cnr", "use nlib::{base};\n\nfn main() -> i64 {\n    let mutable y: i64 = base();\n    y = y + 1;\n    return y;\n}\n")],
    );
    let app = root.join("app");
    assert!(check_dir(&app).unwrap().is_empty(), "cross-edition B checks clean: {:?}", codes(&app));
    // base() (42, parsed under 2026) + 1 (parsed under 2027) = 43.
    assert_all_engines(&app, 43);
}
