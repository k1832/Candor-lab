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
