//! Shared harness support for the module-tree self-host oracle gates (lexer,
//! parser, checker). Each gate materializes its self-host `.cnr` modules plus a
//! per-fixture generated root `main.cnr` into a unique temp directory, then runs
//! the tree through the module loader (`run_dir`) — dogfooding the stage-1 module
//! system (design 0008) rather than string-concatenating the sources.

#![allow(dead_code)]

use std::sync::atomic::{AtomicU64, Ordering};

use candor_proto::{run_dir, RunResult};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// Write `modules` (each `(file_name, source)`) plus the generated `main.cnr`
/// into a fresh temp directory under `CARGO_TARGET_TMPDIR`, load and run it via
/// the module system, then remove the directory. The unique per-call name keeps
/// parallel gate threads and repeated fixtures isolated.
pub fn run_module_tree(modules: &[(&str, &str)], main_cnr: &str) -> RunResult {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("selfhost_tree_{}_{id}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp module dir");
    for (name, src) in modules {
        std::fs::write(dir.join(name), src).unwrap_or_else(|e| panic!("write {name}: {e}"));
    }
    std::fs::write(dir.join("main.cnr"), main_cnr).expect("write main.cnr");
    let result = run_dir(&dir);
    let _ = std::fs::remove_dir_all(&dir);
    result
}

/// Reconstruct the canonical dump text a self-host slice emits through the
/// built-in `trace` sink (each traced value is one output byte).
pub fn trace_text(run: &candor_proto::interp::Run) -> String {
    let bytes: Vec<u8> = run.trace.iter().map(|&v| v as u8).collect();
    String::from_utf8(bytes).expect("dump is ASCII")
}
