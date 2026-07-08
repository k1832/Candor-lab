//! Stage A gate (design 0010 §5): the precise MIR interpreter must reproduce the
//! tree-walking oracle's **semantic trace** `(k, s, θ)` — fault kind `k`, source
//! span `s`, and observable trace `θ` (here: `trace` values) plus exit status —
//! on every runnable program *within the Stage-A MIR subset*. The init-byte guard
//! and the advisory value-context `c` are excluded as non-semantic (§5/F6).
//!
//! Coverage boundary: the current MIR subset is the non-generic scalar/boolean
//! core (arithmetic in all three regimes and every scalar fault kind, comparisons,
//! `&&`/`||`, `if`/`while`/`loop`, `return`/`break`/`continue`, `assert`/`panic`,
//! `requires`/`ensures`, `trace`, and value-parameter user calls). Anything else
//! lowers to `Unsupported`, which this harness records as out-of-subset — never a
//! silent pass. Aggregates/boxes/rawptr/slices/generics (and thus the §11 basket
//! run fixtures, corelib, and generics fixtures) are out of this subset.

use candor_proto::interp::{Fault, Run};
use candor_proto::{
    check, mir, resolve, run_source, run_source_real, MirRunResult, RunResult,
};

/// The comparable semantic outcome: `(k, s, θ)` + exit status.
#[derive(Debug, PartialEq, Eq)]
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

/// The oracle's outcome, or `None` if the program does not run (parse/check error).
fn oracle(src: &str, real: bool) -> Option<Outcome> {
    let r = if real { run_source_real(src) } else { run_source(src) };
    match r {
        RunResult::Ok(run) => Some(ok(run)),
        RunResult::Fault(f) => Some(faulted(f)),
        _ => None,
    }
}

/// The MIR engine's outcome. `Ok(None)` = out-of-subset; `Err` = a non-run result.
fn mir(src: &str, real: bool) -> Result<Option<Outcome>, ()> {
    let r = if real {
        candor_proto::run_source_real_mir(src)
    } else {
        candor_proto::run_source_mir(src)
    };
    match r {
        MirRunResult::Ok(run) => Ok(Some(ok(run))),
        MirRunResult::Fault(f) => Ok(Some(faulted(f))),
        MirRunResult::Unsupported(_) => Ok(None),
        _ => Err(()),
    }
}

/// Assert `(k, s, θ)` equality for an in-subset program (panics on divergence or
/// if the program is unexpectedly out-of-subset).
fn assert_equal(src: &str, real: bool) {
    let o = oracle(src, real).expect("oracle should run this program");
    match mir(src, real) {
        Ok(Some(m)) => assert_eq!(m, o, "semantic-trace divergence for:\n{src}"),
        Ok(None) => panic!("expected in-subset, got Unsupported:\n{src}"),
        Err(()) => panic!("MIR engine produced a non-run result:\n{src}"),
    }
}

// ---------------------------------------------------------------------------
// 1. The in-subset corpus: full (k, s, θ) equality.
// ---------------------------------------------------------------------------

const CORPUS: &[(&str, bool)] = &[
    // arithmetic + regimes
    ("fn main() -> i64 { let a: i64 = 20; let b: i64 = 22; return a + b; }", false),
    ("fn main() -> i64 { let a: i64 = 7; let b: i64 = 3; trace(a % b); trace(a / b); trace(a * b); return a - b; }", false),
    ("fn main() -> i64 { let a: i32 = 2147483647i32; wrapping { let b: i32 = a + 1i32; trace(conv i64 (b)); } return 0; }", false),
    ("fn main() -> i64 { saturating { let a: i8 = 100i8; let b: i8 = a + 100i8; trace(conv i64 (b)); } return 0; }", false),
    ("fn main() -> i64 { let a: u8 = 200u8; wrapping { let b: u8 = a + 100u8; trace(conv i64 (b)); } return 0; }", false),
    // bitwise / shift / bitnot (design 0006 — real `.cnr` syntax only)
    ("fn main() -> i64 { let a: u8 = 12u8; let b: u8 = 10u8; trace(conv i64 (a & b)); trace(conv i64 (a | b)); trace(conv i64 (a ^ b)); return 0; }", true),
    ("fn main() -> i64 { let a: u8 = 1u8; trace(conv i64 (a << 3u8)); let b: u8 = 240u8; trace(conv i64 (b >> 2u8)); return 0; }", true),
    ("fn main() -> i64 { let a: u8 = 5u8; trace(conv i64 (~a)); return 0; }", true),
    ("fn main() -> i64 { wrapping { let a: u8 = 1u8; trace(conv i64 (a << 9u8)); } return 0; }", true),
    // comparisons + booleans + short-circuit
    ("fn main() -> i64 { let x: i64 = 3; if x < 10 { trace(1); } else { trace(2); } return 0; }", false),
    ("fn main() -> i64 { let x: bool = true; let y: bool = false; if x && y { trace(1); } if x || y { trace(2); } return 0; }", false),
    ("fn main() -> i64 { let a: i64 = 5; let b: i64 = 5; if a == b { trace(1); } if a != b { trace(0); } if a <= b { trace(2); } if a >= b { trace(3); } return 0; }", false),
    ("fn main() -> i64 { let n: i64 = 0; if !(n > 0) { trace(99); } return 0; }", false),
    // control flow
    ("fn main() -> i64 { let mut s: i64 = 0; let mut i: i64 = 0; while i < 5 { s = s + i; i = i + 1; } trace(s); return s; }", false),
    ("fn main() -> i64 { let mut i: i64 = 0; loop { if i >= 3 { break; } trace(i); i = i + 1; } return i; }", false),
    ("fn main() -> i64 { let mut i: i64 = 0; let mut acc: i64 = 0; loop { i = i + 1; if i > 10 { break; } if i % 2 == 0 { continue; } acc = acc + i; } trace(acc); return acc; }", false),
    // calls + recursion + contracts (passing)
    ("fn add(a: i64, b: i64) -> i64 { return a + b; } fn main() -> i64 { let r: i64 = add(40, 2); trace(r); return r; }", false),
    ("fn fib(n: i64) -> i64 { if n < 2 { return n; } return fib(n - 1) + fib(n - 2); } fn main() -> i64 { let r: i64 = fib(10); trace(r); return r; }", false),
    ("fn good(x: i64) requires(x > 0) ensures(result > x) -> i64 { return x + 1; } fn main() -> i64 { return good(41); }", false),
    ("fn fact(n: i64) -> i64 { if n <= 1 { return 1; } return n * fact(n - 1); } fn main() -> i64 { let r: i64 = fact(5); trace(r); return r; }", false),
    // conv (non-faulting)
    ("fn main() -> i64 { let a: i64 = 100; let b: i8 = conv i8 (a); trace(conv i64 (b)); return 0; }", false),
    ("fn main() -> i64 { let a: i32 = 0i32 - 5i32; let b: i64 = conv i64 (a); trace(b); return b; }", false),
];

#[test]
fn gate_in_subset_corpus() {
    for (src, real) in CORPUS {
        assert_equal(src, *real);
    }
}

// ---------------------------------------------------------------------------
// 2. The fault-injection axis (design 0010 §4/§5): deliberately-faulting programs
//    must deliver an identical fault identity `(k, s)`. (Bounds and `?`-adjacent
//    faults require arrays/enums, which are out of the current MIR subset.)
// ---------------------------------------------------------------------------

const FAULTS: &[(&str, bool)] = &[
    // overflow (checked arithmetic — INV-CHECK)
    ("fn main() -> i64 { let a: i32 = 2147483647i32; let b: i32 = a + 1i32; return conv i64 (b); }", false),
    // division by zero
    ("fn main() -> i64 { let z: i64 = 0; let q: i64 = 10 / z; return q; }", false),
    // conversion loss
    ("fn main() -> i64 { let a: i64 = 300; let b: i8 = conv i8 (a); return 0; }", false),
    // assert
    ("fn main() -> i64 { let x: i64 = 3; assert(x > 10); return 0; }", false),
    // requires
    ("fn need(x: i64) requires(x > 0) -> i64 { return x; } fn main() -> i64 { return need(0 - 1); }", false),
    // ensures
    ("fn bad() ensures(result > 0) -> i64 { return 0 - 5; } fn main() -> i64 { return bad(); }", false),
    // panic
    ("fn main() -> i64 { panic(\"boom\"); return 0; }", false),
    // shift-amount overflow (checked — real `.cnr` syntax)
    ("fn main() -> i64 { let a: u8 = 1u8; let b: u8 = a << 9u8; return 0; }", true),
];

#[test]
fn gate_fault_injection_axis() {
    for (src, real) in FAULTS {
        let o = oracle(src, *real).expect("faulting program should run");
        assert!(matches!(o, Outcome::Fault { .. }), "expected a fault:\n{src}");
        assert_equal(src, *real);
    }
}

// ---------------------------------------------------------------------------
// 3. Corpus classification: run every runnable fixture through both engines,
//    classify in/out of subset, and assert every in-subset one matches. Prints a
//    coverage summary (visible with `--nocapture`). Out-of-subset is NOT a
//    failure — it is the honest coverage boundary.
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
fn gate_corpus_classification() {
    let mut in_subset = 0usize;
    let mut out_subset = 0usize;
    let mut not_run = 0usize;
    let mut diffs: Vec<String> = Vec::new();

    // Single-file fixtures.
    for path in fixture_files() {
        let src = std::fs::read_to_string(&path).unwrap();
        let real = path.extension().map(|x| x == "cnr").unwrap_or(false);
        let o = match oracle(&src, real) {
            Some(o) => o,
            None => {
                not_run += 1;
                continue;
            }
        };
        match mir(&src, real) {
            Ok(Some(m)) => {
                if m == o {
                    in_subset += 1;
                } else {
                    diffs.push(format!("{}: mir={m:?} oracle={o:?}", path.display()));
                }
            }
            Ok(None) => out_subset += 1,
            Err(()) => not_run += 1,
        }
    }

    // The corelib module tree (design 0008), run as a directory.
    let corelib = dir().join("corelib");
    if corelib.is_dir() {
        let o = match candor_proto::run_dir(&corelib) {
            RunResult::Ok(r) => Some(ok(r)),
            RunResult::Fault(f) => Some(faulted(f)),
            _ => None,
        };
        if let Some(o) = o {
            match candor_proto::run_dir_mir(&corelib) {
                MirRunResult::Ok(r) => {
                    if ok(r) == o { in_subset += 1; } else { diffs.push("corelib: DIFF".into()); }
                }
                MirRunResult::Fault(f) => {
                    if faulted(f) == o { in_subset += 1; } else { diffs.push("corelib: DIFF".into()); }
                }
                MirRunResult::Unsupported(_) => out_subset += 1,
                _ => not_run += 1,
            }
        }
    }

    eprintln!(
        "STAGE-A corpus coverage: in-subset(match)={in_subset}  out-of-subset={out_subset}  not-runnable={not_run}"
    );
    assert!(diffs.is_empty(), "semantic-trace divergences:\n{}", diffs.join("\n"));
}

// ---------------------------------------------------------------------------
// 4. MIR invariants (design 0010 §2), as an explicit test axis.
// ---------------------------------------------------------------------------

fn lower(src: &str) -> mir::MirProgram {
    let program = candor_proto::parse_source(src).expect("parse");
    let mut diags = Vec::new();
    let _ = check::check_program(&program);
    let items = resolve::resolve_program(&program, &mut diags);
    mir::lower_checked(&program, &items).expect("lower")
}

#[test]
fn inv_check_default_arith_carries_fault_edge_wrapping_does_not() {
    // A checked add carries its fault edge; the same add under `wrapping` is a
    // distinct, non-faulting op (INV-CHECK).
    let mp = lower(
        "fn main() -> i64 { let a: i32 = 1i32; let b: i32 = a + 1i32; wrapping { let c: i32 = a + 1i32; } return 0; }",
    );
    let f = mp.get("main").unwrap();
    let mut checked_with_edge = 0;
    let mut wrapping_without_edge = 0;
    for b in &f.blocks {
        for s in &b.stmts {
            if let mir::StatementKind::Assign(_, mir::Rvalue::Bin { op: candor_proto::ast::BinOp::Add, regime, fault, .. }) = &s.kind {
                match regime {
                    mir::Regime::Checked => {
                        assert!(fault.is_some(), "checked add must carry its fault edge");
                        checked_with_edge += 1;
                    }
                    mir::Regime::Wrapping => {
                        assert!(fault.is_none(), "wrapping add must not carry an overflow edge");
                        wrapping_without_edge += 1;
                    }
                    _ => {}
                }
            }
        }
    }
    assert_eq!(checked_with_edge, 1);
    assert_eq!(wrapping_without_edge, 1);
    // check_invariants runs inside lower_checked; run it again explicitly.
    mir::check_invariants(f);
}

#[test]
fn inv_precise_replay_policy_on_every_fn() {
    let mp = lower("fn helper(x: i64) -> i64 { return x + 1; } fn main() -> i64 { return helper(1); }");
    for f in &mp.fns {
        assert_eq!(f.replay, mir::ReplayPolicy::Precise);
        mir::check_invariants(f);
    }
}
