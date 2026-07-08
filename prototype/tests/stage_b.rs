//! Stage B gate (design 0010 §5): the NATIVE (Cranelift JIT, no-opt, whole-
//! program) artifact must reproduce the oracle's semantic trace `(k, s, θ)` on
//! the full runnable corpus, including fault identity `f★` (kind + span). This is
//! the first compiled-vs-interpreted differential test. `native(..)` compiles the
//! whole MirProgram in-process and runs it; `oracle(..)` is the tree-walking
//! reference. Out-of-subset programs surface as `Unsupported` (reported, never a
//! silent skip). Runs on at least one hosted target (x86-64/aarch64 Linux).

use candor_proto::interp::{Fault, Run};
use candor_proto::{run_source, run_source_real, MirRunResult, RunResult};

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

/// The native (Cranelift JIT) engine's outcome. `Ok(None)` = out-of-subset; `Err` = a non-run result.
fn native(src: &str, real: bool) -> Result<Option<Outcome>, ()> {
    let r = if real {
        candor_proto::run_source_real_native(src)
    } else {
        candor_proto::run_source_native(src)
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
    match native(src, real) {
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
fn gate_native_in_subset_corpus() {
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
fn gate_native_fault_injection_axis() {
    for (src, real) in FAULTS {
        let o = oracle(src, *real).expect("faulting program should run");
        assert!(matches!(o, Outcome::Fault { .. }), "expected a fault:\n{src}");
        assert_equal(src, *real);
    }
}

// ---------------------------------------------------------------------------
// 2b. Aggregates over the shared memory substrate (Stage A2): structs, fixed
//     arrays, borrows/deref, and the drop schedule — full (k, s, θ) equality.
// ---------------------------------------------------------------------------

const AGGREGATES: &[(&str, bool)] = &[
    // struct construction + field access
    ("struct P { x: i64, y: i64 } fn main() -> i64 { let p: P = P { x: 40, y: 2 }; return p.x + p.y; }", false),
    // struct behind a borrow, deref + field (the parity shape)
    ("struct P { x: i64, y: i64 } fn get(p: read P) -> i64 { return (deref p).x - (deref p).y; } fn main() -> i64 { let p: P = P { x: 7, y: 5 }; return get(read p); }", false),
    // nested struct
    ("struct Inner { v: i64 } struct Outer { a: Inner, b: i64 } fn main() -> i64 { let o: Outer = Outer { a: Inner { v: 10 }, b: 32 }; return o.a.v + o.b; }", false),
    // fixed array literal + in-bounds index
    ("fn main() -> i64 { let a: [3]i64 = [10, 20, 30]; let i: usize = 2; return a[i]; }", false),
    // array element write through an index place
    ("fn main() -> i64 { let mut a: [3]i64 = [1, 1, 1]; let i: usize = 1; a[i] = 40; return a[0] + a[i] + a[2]; }", false),
    // array-repeat construction
    ("fn main() -> i64 { let a: [4]i64 = [11; 4]; let i: usize = 3; return a[i]; }", false),
];

const ENUMS: &[(&str, bool)] = &[
    // enum construction + match with a copy payload bind (real syntax)
    ("enum Opt { None, Some(i64) } fn main() -> i64 { let a: Opt = Opt::Some(5); let b: Opt = Opt::None; let x: i64 = match a { Opt::None => 0, Opt::Some(v) => v }; let y: i64 = match b { Opt::None => 7, Opt::Some(v) => v }; return x + y; }", true),
    // match returning through arms that `return`
    ("enum E { A(i64), B } fn pick(e: E) -> i64 { match e { E::A(n) => { return n; } E::B => { return 100; } } } fn main() -> i64 { return pick(E::A(42)) + pick(E::B); }", true),
    // enum by-value parameter (drop-inert) + wildcard arm
    ("enum Col { R, G, Bl } fn code(c: Col) -> i64 { return match c { Col::R => 1, Col::G => 2, _ => 3 }; } fn main() -> i64 { return code(Col::R) + code(Col::G) + code(Col::Bl); }", true),
];

#[test]
fn gate_native_enums_match() {
    for (src, real) in ENUMS {
        assert_equal(src, *real);
    }
}

#[test]
fn gate_native_aggregates() {
    for (src, real) in AGGREGATES {
        assert_equal(src, *real);
    }
}

// The bounds fault (design 0010 §5): an out-of-range index delivers `Bounds` at
// the index op's span — a fault axis A1 could not express (no arrays).
const AGGREGATE_FAULTS: &[(&str, bool)] = &[
    ("fn main() -> i64 { let a: [3]i64 = [1, 2, 3]; let i: usize = 5; return a[i]; }", false),
    ("fn main() -> i64 { let a: [2]i64 = [7, 8]; let mut i: usize = 0; let mut s: i64 = 0; while i <= 2 { s = s + a[i]; i = i + 1; } return s; }", false),
];

// The `?`-adjacent fault axis (design 0010 §5/§7): a fault delivered inside a
// function invoked with `?` must carry the identical `(k, s)` in both engines —
// the second axis A1 could not express (no result enums / `?`).
const QUESTION_FAULTS: &[(&str, bool)] = &[
    // overflow inside a `?`-called function (the fault precedes the `?` unwrap)
    ("enum R { ok Val(i64), Err } fn step(x: i32) -> R { let y: i32 = x + 1i32; return R::Val(conv i64 (y)); } fn run(x: i32) -> R { let a: i64 = step(x)?; return R::Val(a); } fn main() -> i64 { let r: R = run(2147483647i32); return match r { R::Val(v) => v, R::Err => 0 - 1 }; }", true),
    // division-by-zero inside a `?`-called function
    ("enum R { ok Val(i64), Err } fn dv(n: i64, d: i64) -> R { let q: i64 = n / d; return R::Val(q); } fn run() -> R { let a: i64 = dv(10, 0)?; return R::Val(a); } fn main() -> i64 { return match run() { R::Val(v) => v, R::Err => 0 - 1 }; }", true),
];

#[test]
fn gate_native_question_fault_axis() {
    for (src, real) in QUESTION_FAULTS {
        let o = oracle(src, *real).expect("faulting program should run");
        assert!(matches!(o, Outcome::Fault { .. }), "expected a fault:\n{src}");
        assert_equal(src, *real);
    }
}

#[test]
fn gate_native_aggregate_fault_axis() {
    for (src, real) in AGGREGATE_FAULTS {
        let o = oracle(src, *real).expect("faulting program should run");
        assert!(matches!(o, Outcome::Fault { kind, .. } if kind == "Bounds"), "expected a Bounds fault:\n{src}");
        assert_equal(src, *real);
    }
}

// ---------------------------------------------------------------------------
// 3. Gate A's closure (design 0010 §5): EVERY runnable fixture — the five §11
//    `.cn` run fixtures and their five `.cnr` twins, the corelib module tree and
//    its flat single-file form, the parity pair, and every generics / real /
//    iteration fixture — is asserted `(k, s, θ)`-equal across the two engines,
//    AND the classification asserts ZERO out-of-subset runnable fixtures. There
//    is no honest coverage boundary left: the MIR is a faithful carrier of the
//    full runnable corpus. (Directory trees `corelib` and `corelib_question` are
//    included; `.cn`/`.cnr` single files under run/parity/real/generics too.)
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
fn gate_native_full_corpus_equality() {
    let mut equal = 0usize;
    let mut out_subset: Vec<String> = Vec::new();
    let mut not_run = 0usize;
    let mut diffs: Vec<String> = Vec::new();

    // Single-file fixtures (run/parity/real/generics) plus the flat corelib.
    let mut files = fixture_files();
    files.push(dir().join("corelib_flat.cnr"));
    for path in files {
        let src = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let real = path.extension().map(|x| x == "cnr").unwrap_or(false);
        let o = match oracle(&src, real) {
            Some(o) => o,
            None => {
                not_run += 1;
                continue;
            }
        };
        match native(&src, real) {
            Ok(Some(m)) => {
                if m == o {
                    equal += 1;
                } else {
                    diffs.push(format!("{}: mir={m:?} oracle={o:?}", path.display()));
                }
            }
            Ok(None) => out_subset.push(path.display().to_string()),
            Err(()) => not_run += 1,
        }
    }

    // Module-tree fixtures (design 0008): the corelib seed and the cross-type `?`
    // seed, each run as a directory through both engines.
    for name in ["corelib", "corelib_question"] {
        let d = dir().join(name);
        if !d.is_dir() {
            continue;
        }
        let o = match candor_proto::run_dir(&d) {
            RunResult::Ok(r) => Some(ok(r)),
            RunResult::Fault(f) => Some(faulted(f)),
            _ => None,
        };
        let o = match o {
            Some(o) => o,
            None => {
                not_run += 1;
                continue;
            }
        };
        match candor_proto::run_dir_native(&d) {
            MirRunResult::Ok(r) => {
                if ok(r) == o { equal += 1; } else { diffs.push(format!("{name}: DIFF")); }
            }
            MirRunResult::Fault(f) => {
                if faulted(f) == o { equal += 1; } else { diffs.push(format!("{name}: DIFF")); }
            }
            MirRunResult::Unsupported(_) => out_subset.push(name.to_string()),
            _ => not_run += 1,
        }
    }

    eprintln!(
        "GATE A CLOSED: {equal} runnable fixtures (k, s, θ)-equal; out-of-subset={}  not-runnable={not_run}",
        out_subset.len()
    );
    assert!(diffs.is_empty(), "semantic-trace divergences:\n{}", diffs.join("\n"));
    // Gate A's closure statement: NO runnable fixture is out of the MIR subset.
    assert!(
        out_subset.is_empty(),
        "Gate A not closed: {} runnable fixtures still out-of-subset:\n{}",
        out_subset.len(),
        out_subset.join("\n")
    );
    assert!(equal >= 30, "expected the full runnable corpus (>=30 fixtures), got {equal}");
}

