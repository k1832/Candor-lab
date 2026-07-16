//! Module-tree (multi-file `.cnr`) tests — design 0008 stage 1.
//!
//! Exercises the filesystem→module mapping, `use` resolution, `pub`/private
//! visibility, the nested-namespace path, and the two negative paths (import
//! cycle, unresolved import). Single-file behavior is covered by the rest of the
//! suite and is untouched here.

use std::path::PathBuf;

use candor_proto::{check_dir, run_dir, RunResult};

fn dir(name: &str) -> PathBuf {
    PathBuf::from(format!("{}/tests/fixtures/modules/{name}", env!("CARGO_MANIFEST_DIR")))
}

fn codes(name: &str) -> Vec<String> {
    match check_dir(&dir(name)) {
        Ok(diags) => diags.into_iter().map(|d| d.code).collect(),
        Err(d) => vec![d.code],
    }
}

// ---- positive: use, pub/private, nested namespace, alias call --------------

#[test]
fn ok_tree_checks_clean() {
    assert!(codes("ok_tree").is_empty(), "ok_tree should check clean, got {:?}", codes("ok_tree"));
}

#[test]
fn ok_tree_runs_to_sentinel() {
    match run_dir(&dir("ok_tree")) {
        RunResult::Ok(r) => assert_eq!(r.ret, 13),
        other => panic!("ok_tree did not run: {}", describe(other)),
    }
}

// ---- negative: importing a private (non-`pub`) item ------------------------

#[test]
fn private_item_is_rejected() {
    assert!(codes("bad_private").contains(&"E0903".to_string()), "want E0903, got {:?}", codes("bad_private"));
}

// ---- negative: an import cycle ---------------------------------------------

#[test]
fn import_cycle_is_rejected() {
    assert!(codes("bad_cycle").contains(&"E0904".to_string()), "want E0904, got {:?}", codes("bad_cycle"));
}

// ---- negative: importing a name the module does not export -----------------

#[test]
fn unresolved_import_is_rejected() {
    assert!(codes("bad_unresolved").contains(&"E0902".to_string()), "want E0902, got {:?}", codes("bad_unresolved"));
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
