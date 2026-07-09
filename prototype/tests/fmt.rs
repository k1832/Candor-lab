//! Formatter (P16/NN#11) validation: idempotence over the whole `.cnr` corpus
//! and a single-file semantic-preservation gate (spec 02 §9). The module-tree
//! semantic gate is covered by the existing corelib/modules/iteration run tests,
//! which execute the reformatted fixtures in place.

use candor_proto::{format_source_real, run_dir, run_source_real, RunResult};
use std::path::{Path, PathBuf};

fn all_cnr() -> Vec<PathBuf> {
    let root = format!("{}/tests/fixtures", env!("CARGO_MANIFEST_DIR"));
    let mut out = Vec::new();
    collect(Path::new(&root), &mut out);
    out.sort();
    out
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            collect(&p, out);
        } else if p.extension().map(|x| x == "cnr").unwrap_or(false) {
            out.push(p);
        }
    }
}

/// Every corpus file parses and formats without error.
#[test]
fn corpus_formats() {
    for f in all_cnr() {
        let src = std::fs::read_to_string(&f).unwrap();
        if let Err(d) = format_source_real(&src) {
            panic!("format failed for {}: {}", f.display(), d.to_json());
        }
    }
}

/// Idempotence: `format(format(x)) == format(x)` for every corpus file.
#[test]
fn idempotent() {
    for f in all_cnr() {
        let src = std::fs::read_to_string(&f).unwrap();
        let once = format_source_real(&src).unwrap();
        let twice = format_source_real(&once).unwrap();
        assert_eq!(once, twice, "not idempotent: {}", f.display());
    }
}

/// A comparable run outcome (θ = the trace vector, plus return / fault kind),
/// modulo spans; `None` for files that do not run as a standalone program.
fn outcome(src: &str) -> Option<Result<(i64, Vec<i64>), String>> {
    match run_source_real(src) {
        RunResult::Ok(run) => Some(Ok((run.ret, run.trace))),
        RunResult::Fault(fault) => Some(Err(format!("fault:{:?}", fault.kind))),
        RunResult::CheckErrors(_) | RunResult::ParseError(_) => None,
    }
}

/// Semantic-preservation gate (single-file): for every standalone-runnable file,
/// the reformatted source produces the identical (return, trace) / fault kind.
#[test]
fn semantic_preservation_single_file() {
    let mut runnable = 0;
    for f in all_cnr() {
        let src = std::fs::read_to_string(&f).unwrap();
        let before = match outcome(&src) {
            Some(o) => o,
            None => continue,
        };
        runnable += 1;
        let formatted = format_source_real(&src).unwrap();
        let after = outcome(&formatted).unwrap_or_else(|| {
            panic!("{}: ran before formatting but not after", f.display())
        });
        assert_eq!(before, after, "semantic drift after formatting {}", f.display());
    }
    assert!(runnable > 0, "no standalone-runnable fixtures exercised the gate");
}


fn dir_outcome(dir: &Path) -> Option<Result<(i64, Vec<i64>), String>> {
    match run_dir(dir) {
        RunResult::Ok(run) => Some(Ok((run.ret, run.trace))),
        RunResult::Fault(fault) => Some(Err(format!("fault:{:?}", fault.kind))),
        RunResult::CheckErrors(_) | RunResult::ParseError(_) => None,
    }
}

fn copy_tree(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src).unwrap().flatten() {
        let p = e.path();
        let target = dst.join(p.file_name().unwrap());
        if p.is_dir() {
            copy_tree(&p, &target);
        } else {
            std::fs::copy(&p, &target).unwrap();
        }
    }
}

fn format_tree_in_place(dir: &Path) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            format_tree_in_place(&p);
        } else if p.extension().map(|x| x == "cnr").unwrap_or(false) {
            let src = std::fs::read_to_string(&p).unwrap();
            let out = format_source_real(&src).unwrap();
            std::fs::write(&p, out).unwrap();
        }
    }
}

/// Semantic-preservation gate (module trees): every runnable `.cnr` module tree
/// (a directory whose `main.cnr` runs) produces the identical (return, trace) /
/// fault kind after every file in the tree is reformatted.
#[test]
fn semantic_preservation_module_trees() {
    let root = format!("{}/tests/fixtures", env!("CARGO_MANIFEST_DIR"));
    let tmp = std::env::temp_dir().join(format!("candor_fmt_gate_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    let mut checked = 0;
    for e in std::fs::read_dir(&root).unwrap().flatten() {
        let dir = e.path();
        if !dir.is_dir() {
            continue;
        }
        let before = match dir_outcome(&dir) {
            Some(o) => o,
            None => continue,
        };
        checked += 1;
        let mirror = tmp.join(dir.file_name().unwrap());
        let _ = std::fs::remove_dir_all(&mirror);
        copy_tree(&dir, &mirror);
        format_tree_in_place(&mirror);
        let after = dir_outcome(&mirror)
            .unwrap_or_else(|| panic!("{}: tree ran before formatting but not after", dir.display()));
        assert_eq!(before, after, "semantic drift after formatting tree {}", dir.display());
    }
    let _ = std::fs::remove_dir_all(&tmp);
    assert!(checked > 0, "no runnable module trees exercised the gate");
}
