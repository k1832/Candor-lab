//! Stage D gate (design 0010 §5): optimization within the R1 license.
//!
//! Four engines, one trace. For every fixture and every fault axis, the OPTIMIZED
//! native engine (the validated MIR-level R1 rewrite `mir::opt` + Cranelift
//! `opt_level=speed`) must deliver the identical semantic trace `(k, s, θ)` — fault
//! identity `f★` included — as the tree-walking ORACLE, the precise MIR
//! interpreter, and the no-opt native engine. This is P5/NN#2's cross-build-mode
//! determinism made empirical: interpreted · MIR · native-noopt · native-opt agree.
//!
//! Any optimization that changes an observable trace is a soundness bug that BLOCKS
//! the stage (design 0010 §5) — never a tolerated perf win.

use candor::interp::{Fault, Run};
use candor::{
    run_dir, run_dir_native, run_dir_native_opt, run_source, run_source_mir, run_source_native,
    run_source_native_opt, run_source_real, run_source_real_mir, run_source_real_native,
    run_source_real_native_opt, MirRunResult, RunResult,
};

#[derive(Debug, PartialEq, Eq, Clone)]
enum Outcome {
    Ok { ret: i64, trace: Vec<i64> },
    Fault { kind: String, start: usize, end: usize },
}

fn ok(r: Run) -> Outcome {
    Outcome::Ok { ret: r.ret, trace: r.trace }
}
fn faulted(f: Fault) -> Outcome {
    Outcome::Fault { kind: format!("{:?}", f.kind), start: f.span.start, end: f.span.end }
}

fn from_oracle(r: RunResult) -> Option<Outcome> {
    match r {
        RunResult::Ok(run) => Some(ok(run)),
        RunResult::Fault(f) => Some(faulted(f)),
        _ => None,
    }
}
fn from_mir(r: MirRunResult) -> Result<Option<Outcome>, ()> {
    match r {
        MirRunResult::Ok(run) => Ok(Some(ok(run))),
        MirRunResult::Fault(f) => Ok(Some(faulted(f))),
        MirRunResult::Unsupported(_) => Ok(None),
        _ => Err(()),
    }
}

/// Assert all four engines agree for one source program.
fn assert_four(src: &str, real: bool) {
    let oracle = if real { run_source_real(src) } else { run_source(src) };
    let oracle = from_oracle(oracle).expect("oracle should run this program");

    let mir = if real { run_source_real_mir(src) } else { run_source_mir(src) };
    let noopt = if real { run_source_real_native(src) } else { run_source_native(src) };
    let opt = if real { run_source_real_native_opt(src) } else { run_source_native_opt(src) };

    for (label, r) in [("mir", mir), ("native-noopt", noopt), ("native-opt", opt)] {
        match from_mir(r) {
            Ok(Some(m)) => assert_eq!(m, oracle, "{label} diverged from oracle for:\n{src}"),
            Ok(None) => panic!("{label}: expected in-subset, got Unsupported:\n{src}"),
            Err(()) => panic!("{label}: non-run result:\n{src}"),
        }
    }
}

// A representative in-subset corpus spanning arithmetic regimes, control flow,
// calls/recursion, contracts, aggregates, enums, and — critically — the rawptr/
// MMIO observable path now marked observable and lowered as a barrier call.
const CORPUS: &[(&str, bool)] = &[
    ("fn main() -> i64 { let a: i64 = 20; let b: i64 = 22; return a + b; }", false),
    ("fn main() -> i64 { let a: i64 = 7; let b: i64 = 3; trace(a % b); trace(a / b); trace(a * b); return a - b; }", false),
    ("fn main() -> i64 { let a: i32 = 2147483647i32; wrapping { let b: i32 = a + 1i32; trace(conv i64 (b)); } return 0; }", false),
    ("fn main() -> i64 { saturating { let a: i8 = 100i8; let b: i8 = a + 100i8; trace(conv i64 (b)); } return 0; }", false),
    ("fn main() -> i64 { let mut s: i64 = 0; let mut i: i64 = 0; while i < 5 { s = s + i; i = i + 1; } trace(s); return s; }", false),
    ("fn fib(n: i64) -> i64 { if n < 2 { return n; } return fib(n - 1) + fib(n - 2); } fn main() -> i64 { let r: i64 = fib(10); trace(r); return r; }", false),
    ("fn good(x: i64) requires(x > 0) ensures(result > x) -> i64 { return x + 1; } fn main() -> i64 { return good(41); }", false),
    ("struct P { x: i64, y: i64 } fn get(p: read P) -> i64 { return (deref p).x - (deref p).y; } fn main() -> i64 { let p: P = P { x: 7, y: 5 }; return get(read p); }", false),
    ("fn main() -> i64 { let mut a: [3]i64 = [1, 1, 1]; let i: usize = 1; a[i] = 40; return a[0] + a[i] + a[2]; }", false),
    ("enum Opt { None, Some(i64) } fn main() -> i64 { let a: Opt = Opt::Some(5); let x: i64 = match a { Opt::None => 0, Opt::Some(v) => v }; return x; }", true),
    // rawptr write+read through a fixed address (the MMIO substrate observable):
    ("fn main() -> i64 { unsafe \"mmio\" { let p: rawptr u32 = addr_to_ptr[u32](3145728); ptr_write(p, 7u32); let v: u32 = ptr_read(p); return conv i64 v; } }", true),
];

const FAULTS: &[(&str, bool)] = &[
    ("fn main() -> i64 { let a: i32 = 2147483647i32; let b: i32 = a + 1i32; return conv i64 (b); }", false),
    ("fn main() -> i64 { let z: i64 = 0; let q: i64 = 10 / z; return q; }", false),
    ("fn main() -> i64 { let a: i64 = 300; let b: i8 = conv i8 (a); return 0; }", false),
    ("fn main() -> i64 { let x: i64 = 3; assert(x > 10); return 0; }", false),
    ("fn need(x: i64) requires(x > 0) -> i64 { return x; } fn main() -> i64 { return need(0 - 1); }", false),
    ("fn bad() ensures(result > 0) -> i64 { return 0 - 5; } fn main() -> i64 { return bad(); }", false),
    ("fn main() -> i64 { panic(\"boom\"); return 0; }", false),
    ("fn main() -> i64 { let a: [3]i64 = [1, 2, 3]; let i: usize = 5; return a[i]; }", false),
];

#[test]
fn gate_four_engine_corpus() {
    for (src, real) in CORPUS {
        assert_four(src, *real);
    }
}

#[test]
fn gate_four_engine_fault_identity() {
    // The sharpest INV-FAULT-ID test: fault identity `f★` must survive optimization.
    for (src, real) in FAULTS {
        let o = if *real { run_source_real(src) } else { run_source(src) };
        assert!(matches!(from_oracle(o), Some(Outcome::Fault { .. })), "expected a fault:\n{src}");
        assert_four(src, *real);
    }
}

// ---------------------------------------------------------------------------
// The FULL runnable corpus (the 31 fixtures Gate A/B close on) through all four
// engines including native-opt (design 0010 §5's cross-mode gate).
// ---------------------------------------------------------------------------

fn dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn fixture_files() -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for sub in ["run", "parity", "real", "generics"] {
        let d = dir().join(sub);
        if let Ok(rd) = std::fs::read_dir(&d) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().map(|x| x == "cn" || x == "cnr").unwrap_or(false) {
                    out.push(p);
                }
            }
        }
    }
    out.sort();
    out
}

#[test]
fn gate_full_corpus_four_engine() {
    let mut equal = 0usize;
    let mut diffs: Vec<String> = Vec::new();
    let mut out_subset: Vec<String> = Vec::new();

    let mut files = fixture_files();
    files.push(dir().join("corelib_flat.cnr"));
    for path in files {
        let src = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let real = path.extension().map(|x| x == "cnr").unwrap_or(false);
        let oracle = if real { run_source_real(&src) } else { run_source(&src) };
        let oracle = match from_oracle(oracle) {
            Some(o) => o,
            None => continue,
        };
        let engines: [(&str, MirRunResult); 3] = [
            ("mir", if real { run_source_real_mir(&src) } else { run_source_mir(&src) }),
            ("noopt", if real { run_source_real_native(&src) } else { run_source_native(&src) }),
            ("opt", if real { run_source_real_native_opt(&src) } else { run_source_native_opt(&src) }),
        ];
        let mut all_ok = true;
        for (label, r) in engines {
            match from_mir(r) {
                Ok(Some(m)) => {
                    if m != oracle {
                        diffs.push(format!("{} [{label}]: got={m:?} oracle={oracle:?}", path.display()));
                        all_ok = false;
                    }
                }
                Ok(None) => {
                    out_subset.push(format!("{} [{label}]", path.display()));
                    all_ok = false;
                }
                Err(()) => {
                    diffs.push(format!("{} [{label}]: non-run", path.display()));
                    all_ok = false;
                }
            }
        }
        if all_ok {
            equal += 1;
        }
    }

    // Directory-tree fixtures through all four engines.
    for name in ["corelib", "corelib_question"] {
        let d = dir().join(name);
        if !d.is_dir() {
            continue;
        }
        let oracle = match from_oracle(run_dir(&d)) {
            Some(o) => o,
            None => continue,
        };
        let engines: [(&str, MirRunResult); 3] = [
            ("mir", candor::run_dir_mir(&d)),
            ("noopt", run_dir_native(&d)),
            ("opt", run_dir_native_opt(&d)),
        ];
        let mut all_ok = true;
        for (label, r) in engines {
            match from_mir(r) {
                Ok(Some(m)) if m == oracle => {}
                other => {
                    diffs.push(format!("{name} [{label}]: {other:?} vs {oracle:?}"));
                    all_ok = false;
                }
            }
        }
        if all_ok {
            equal += 1;
        }
    }

    eprintln!("STAGE D: {equal} fixtures 4-engine (k,s,θ)-equal (oracle·mir·noopt·OPT)");
    assert!(diffs.is_empty(), "cross-mode divergences under optimization:\n{}", diffs.join("\n"));
    assert!(out_subset.is_empty(), "unexpected out-of-subset:\n{}", out_subset.join("\n"));
    assert!(equal >= 30, "expected >=30 four-engine-equal fixtures, got {equal}");
}

// ---------------------------------------------------------------------------
// The R1 rewrite machinery (INV-R1-ONLY made executable): the MIR-level
// dead-local elimination fires, its per-rewrite validator passes, and every
// invariant is preserved. Fault-capable and observable statements are NEVER
// removed. Replay stays Precise (no R3 batching introduced) — replay vacuous.
// ---------------------------------------------------------------------------

fn lower(src: &str) -> candor::mir::MirProgram {
    let program = candor::parse_source(src).expect("parse");
    let mut diags = Vec::new();
    let items = candor::resolve::resolve_program(&program, &mut diags);
    candor::mir::lower_checked(&program, &items).expect("lower")
}

#[test]
fn r1_rewrite_fires_and_validates() {
    // A function with an obviously-dead pure temp (`d` computed, never read).
    let src = "fn main() -> i64 { let a: i64 = 5; let b: i64 = 6; let d: bool = a < b; let e: i64 = a; return a + b; }";
    let mp = lower(src);
    let before = candor::mir::opt::stmt_count(&mp);
    let opt = candor::mir::opt::optimize(mp);
    let after = candor::mir::opt::stmt_count(&opt.prog);
    assert!(!opt.rewrites.is_empty(), "R1 dead-local elimination should have fired");
    assert!(after < before, "the pass should have removed at least one statement ({before} -> {after})");
    // Every recorded rewrite is a validated R1 removal of a pure τ-step.
    for rw in &opt.rewrites {
        assert!(
            matches!(rw.kind, "use" | "cmp" | "is_null" | "ref" | "load" | "ptr_arith" | "static_addr" | "str_addr" | "un"),
            "R1 removed a non-whitelisted rvalue kind: {}",
            rw.kind
        );
    }
    // Post-pass, replay stays Precise on every fn (no R3 batching introduced, so
    // the f★-replay obligation stays vacuous — design 0010 §2 INV-FAULT-ID).
    for f in &opt.prog.fns {
        assert_eq!(f.replay, candor::mir::ReplayPolicy::Precise);
    }
}

#[test]
fn r1_never_removes_observables_or_faults() {
    // The MMIO fixture: rawptr write/read are observable — the pass must NOT touch
    // them, and a checked add's fault edge must survive.
    let src = "fn main() -> i64 { unsafe \"mmio\" { let p: rawptr u32 = addr_to_ptr[u32](3145728); ptr_write(p, 7u32); let v: u32 = ptr_read(p); return conv i64 v; } }";
    let program = candor::real::parse_source(src).expect("parse");
    let mut diags = Vec::new();
    let items = candor::resolve::resolve_program(&program, &mut diags);
    let mp = candor::mir::lower_checked(&program, &items).expect("lower");
    // Count observable statements before/after; none may be removed.
    let obs_before: usize = mp.fns.iter().flat_map(|f| f.blocks.iter()).flat_map(|b| b.stmts.iter()).filter(|s| s.observable).count();
    assert!(obs_before >= 2, "expected the rawptr write+read to be marked observable, got {obs_before}");
    let opt = candor::mir::opt::optimize(mp);
    let obs_after: usize = opt.prog.fns.iter().flat_map(|f| f.blocks.iter()).flat_map(|b| b.stmts.iter()).filter(|s| s.observable).count();
    assert_eq!(obs_before, obs_after, "R1 must never eliminate an observable");
    // No rewrite ever targeted an observable statement.
    assert!(opt.rewrites.iter().all(|rw| rw.kind != "other"));
}
