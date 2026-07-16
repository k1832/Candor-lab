//! Shared harness support for the module-tree self-host oracle gates (lexer,
//! parser, checker). Each gate materializes its self-host `.cnr` modules plus a
//! per-fixture generated root `main.cnr` into a unique temp directory, then runs
//! the tree through the module loader (`run_dir`) — dogfooding the stage-1 module
//! system (design 0008) rather than string-concatenating the sources.

#![allow(dead_code)]

use std::sync::atomic::{AtomicU64, Ordering};

use candor::{check_dir, diag::Diag, run_dir, RunResult};

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

/// Materialize `modules` plus the generated `main.cnr` into a fresh temp dir and
/// run the MODULE-AWARE reference checker (`check_dir`) over the tree, returning
/// its diagnostics. Unlike the single-file `check_source_real`, this resolves the
/// tree's `use` imports before checking, so a non-leaf module's imported names are
/// known -- the correct oracle for the import-resolution isolation gate.
pub fn check_module_tree(modules: &[(&str, &str)], main_cnr: &str) -> Result<Vec<Diag>, Diag> {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("selfhost_check_{}_{id}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp module dir");
    for (name, src) in modules {
        std::fs::write(dir.join(name), src).unwrap_or_else(|e| panic!("write {name}: {e}"));
    }
    std::fs::write(dir.join("main.cnr"), main_cnr).expect("write main.cnr");
    let result = check_dir(&dir);
    let _ = std::fs::remove_dir_all(&dir);
    result
}

/// Reconstruct the canonical dump text a self-host slice emits through the
/// built-in `trace` sink (each traced value is one output byte).
pub fn trace_text(run: &candor::interp::Run) -> String {
    let bytes: Vec<u8> = run.trace.iter().map(|&v| v as u8).collect();
    String::from_utf8(bytes).expect("dump is ASCII")
}

use candor::interp::{Fault, FaultKind, Run};

/// Canonical fault kind-code map — the schema every self-host differential gate
/// renders `FAULT kind span` with. Total over `FaultKind` (codes 0..=9, matching
/// the interp/backend kind codes) so the mapping cannot drift between gates.
pub fn fault_code(k: FaultKind) -> i64 {
    match k {
        FaultKind::Overflow => 0,
        FaultKind::DivByZero => 1,
        FaultKind::Assert => 2,
        FaultKind::Panic => 3,
        FaultKind::Bounds => 4,
        FaultKind::ConvLoss => 5,
        FaultKind::Requires => 6,
        FaultKind::Ensures => 7,
        FaultKind::BadPointer => 8,
        FaultKind::NoForeignRuntime => 9,
    }
}

/// Render a non-faulting run in the canonical dump schema (`RET`, then a `TRACE`
/// line per traced value).
pub fn dump_ok(run: &Run) -> String {
    let mut s = format!("RET {}\n", run.ret);
    for v in &run.trace {
        s.push_str(&format!("TRACE {v}\n"));
    }
    s
}

/// Render a fault in the canonical dump schema: the trace emitted BEFORE the
/// fault (`TRACE` lines, threaded into the fault at the run boundary) followed by
/// `FAULT kind span.start span.end`. Including the pre-fault trace closes the
/// F-FAULT-TRACE blind spot — two engines that trace differently before an
/// identical fault now diverge in the dump.
pub fn dump_fault(f: &Fault) -> String {
    let mut s = String::new();
    for v in &f.trace {
        s.push_str(&format!("TRACE {v}\n"));
    }
    s.push_str(&format!("FAULT {} {} {}\n", fault_code(f.kind), f.span.start, f.span.end));
    s
}

/// Run `f` on a 256 MiB stack. The self-host tools recurse deeply (parser and
/// checker over the largest modules), overflowing the default test-thread stack.
pub fn on_big_stack<F: FnOnce() + Send + 'static>(f: F) {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(f)
        .expect("spawn big-stack thread")
        .join()
        .expect("gate thread panicked");
}
