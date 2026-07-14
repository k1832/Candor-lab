//! Floating-point (`f64`) gate (design 0016).
//!
//! IEEE-754 `f64` is deterministic on the shared x86-64 target, so every engine —
//! the tree-walking oracle, the precise MIR interpreter, the Cranelift native
//! backend (no-opt and `-O2`), and the LLVM `-O2` backend — must produce
//! BIT-IDENTICAL results. Each test computes an `f64`, observes its exact 64-bit
//! pattern via `trace` (the one channel all five engines share — `f64::to_bits`
//! reinterpreted as `i64`), and asserts every engine agrees and matches the value
//! computed in host Rust.
//!
//! NaN is the single exception: the SIGN of a computed NaN (`0.0/0.0`) is
//! IEEE-unspecified and differs between a constant-folding compiler (LLVM `-O2`
//! folds to `+NaN`) and x86 runtime hardware (`-NaN`). NaN is therefore gated by
//! BEHAVIOUR (IEEE comparison outcomes), not by its exact bit pattern.

use std::path::Path;
use std::process::Command;

use candor_proto::{
    compile_path_llvm, run_source_real, run_source_real_mir, run_source_real_native,
    run_source_real_native_opt, MirRunResult, RunResult,
};

/// (ret, trace) of the tree-walking oracle.
fn oracle(src: &str) -> (i64, Vec<i64>) {
    match run_source_real(src) {
        RunResult::Ok(r) => (r.ret, r.trace),
        RunResult::Fault(f) => panic!("oracle faulted: {}\n{src}", f.to_json()),
        RunResult::CheckErrors(d) => {
            panic!("oracle check errors: {:?}\n{src}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("oracle parse error: {}\n{src}", d.to_json()),
    }
}

fn mir_out(r: MirRunResult, label: &str, src: &str) -> (i64, Vec<i64>) {
    match r {
        MirRunResult::Ok(r) => (r.ret, r.trace),
        MirRunResult::Fault(f) => panic!("{label} faulted: {}\n{src}", f.to_json()),
        MirRunResult::Unsupported(e) => panic!("{label} unsupported: {e}\n{src}"),
        MirRunResult::CheckErrors(d) => panic!("{label} check errors: {:?}\n{src}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        MirRunResult::ParseError(d) => panic!("{label} parse error: {}\n{src}", d.to_json()),
    }
}

fn clang_available() -> bool {
    Command::new("clang").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

/// The LLVM `-O2` process's traced result (θ), or `None` when clang is absent.
fn llvm_trace(src: &str, tag: &str) -> Option<Vec<i64>> {
    if !clang_available() {
        return None;
    }
    let dir = std::env::temp_dir();
    let srcp = dir.join(format!("candor-floats-{}-{}.cnr", std::process::id(), tag));
    let outp = dir.join(format!("candor-floats-{}-{}", std::process::id(), tag));
    std::fs::write(&srcp, src).unwrap();
    compile_path_llvm(Path::new(&srcp), &outp).expect("LLVM compile should succeed");
    let output = Command::new(&outp).output().expect("run compiled float program");
    let _ = std::fs::remove_file(&srcp);
    let _ = std::fs::remove_file(&outp);
    let trace = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.trim().parse::<i64>().expect("trace line is an integer"))
        .collect();
    Some(trace)
}

/// Run `src` through every engine, assert they all agree bit-for-bit (`ret` for the
/// four in-process engines; `trace`/θ for all five including LLVM), and return the
/// oracle's `(ret, trace)`.
fn all_engines(src: &str, tag: &str) -> (i64, Vec<i64>) {
    let (o_ret, o_trace) = oracle(src);
    let (m_ret, m_trace) = mir_out(run_source_real_mir(src), "mir", src);
    let (n_ret, n_trace) = mir_out(run_source_real_native(src), "native-noopt", src);
    let (p_ret, p_trace) = mir_out(run_source_real_native_opt(src), "native-opt", src);

    for (label, ret, trace) in [
        ("mir", m_ret, &m_trace),
        ("native-noopt", n_ret, &n_trace),
        ("native-opt", p_ret, &p_trace),
    ] {
        assert_eq!(ret, o_ret, "{label} ret diverged from oracle for:\n{src}");
        assert_eq!(trace, &o_trace, "{label} trace diverged from oracle for:\n{src}");
    }
    if let Some(l_trace) = llvm_trace(src, tag) {
        assert_eq!(l_trace, o_trace, "llvm trace diverged from oracle for:\n{src}");
    }
    (o_ret, o_trace)
}

/// Assert every engine computes the traced `f64` results to the exact bit patterns
/// `expected` (host Rust `f64::to_bits`), and returns `ret_expected`.
fn assert_bits(src: &str, tag: &str, ret_expected: i64, expected: &[f64]) {
    let (ret, trace) = all_engines(src, tag);
    let want: Vec<i64> = expected.iter().map(|f| f.to_bits() as i64).collect();
    assert_eq!(trace, want, "traced bits mismatch for:\n{src}");
    assert_eq!(ret, ret_expected, "ret mismatch for:\n{src}");
}

// ---------------------------------------------------------------------------
// 1. Arithmetic + precedence — exact bits across all engines.
// ---------------------------------------------------------------------------

#[test]
fn add_sub_mul_div_exact_bits() {
    assert_bits(
        "fn main() -> i64 { \
         let a: f64 = 1.5; let b: f64 = 2.25; \
         let s: f64 = a + b; trace(s); \
         let d: f64 = b - a; trace(d); \
         let m: f64 = a * b; trace(m); \
         let q: f64 = b / a; trace(q); \
         return 0; }",
        "arith",
        0,
        &[1.5 + 2.25, 2.25 - 1.5, 1.5 * 2.25, 2.25 / 1.5],
    );
}

#[test]
fn precedence_and_neg() {
    // `a + b * c` binds `*` first; unary neg is IEEE sign flip.
    assert_bits(
        "fn main() -> i64 { \
         let a: f64 = 2.0; let b: f64 = 3.0; let c: f64 = 4.0; \
         let r: f64 = a + b * c; trace(r); \
         let n: f64 = -r; trace(n); \
         return 0; }",
        "prec",
        0,
        &[2.0 + 3.0 * 4.0, -(2.0 + 3.0 * 4.0)],
    );
}

#[test]
fn accumulated_rounding() {
    // 0.1 + 0.2 != 0.3 exactly — the canonical rounding case, bit-identical everywhere.
    assert_bits(
        "fn main() -> i64 { let a: f64 = 0.1; let b: f64 = 0.2; let r: f64 = a + b; trace(r); return 0; }",
        "round",
        0,
        &[0.1 + 0.2],
    );
}

// ---------------------------------------------------------------------------
// 2. Comparisons (ordered) — including negatives.
// ---------------------------------------------------------------------------

#[test]
fn ordered_comparisons() {
    let src = "fn main() -> i64 { \
        let a: f64 = -1.5; let b: f64 = 2.5; \
        if a < b { trace(1); } else { trace(0); } \
        if b <= b { trace(1); } else { trace(0); } \
        if a > b { trace(1); } else { trace(0); } \
        if b >= a { trace(1); } else { trace(0); } \
        if a == a { trace(1); } else { trace(0); } \
        if a != b { trace(1); } else { trace(0); } \
        return 0; }";
    let (_, trace) = all_engines(src, "cmp");
    assert_eq!(trace, vec![1, 1, 0, 1, 1, 1]);
}

// ---------------------------------------------------------------------------
// 3. inf — deterministic bits (overflow / divide-by-zero do NOT fault: design 0016).
// ---------------------------------------------------------------------------

#[test]
fn infinities_exact_bits() {
    assert_bits(
        "fn main() -> i64 { \
         let z: f64 = 0.0; let o: f64 = 1.0; \
         let pinf: f64 = o / z; trace(pinf); \
         let ninf: f64 = (-o) / z; trace(ninf); \
         return 0; }",
        "inf",
        0,
        &[f64::INFINITY, f64::NEG_INFINITY],
    );
}

// ---------------------------------------------------------------------------
// 4. NaN — gated by BEHAVIOUR (its sign bit is IEEE-unspecified across engines).
// ---------------------------------------------------------------------------

#[test]
fn nan_comparisons_are_ieee() {
    // NaN != NaN is true; every ordered/`==` comparison with NaN is false.
    let src = "fn main() -> i64 { \
        let z: f64 = 0.0; let n: f64 = z / z; let o: f64 = 1.0; \
        if n != n { trace(1); } else { trace(0); } \
        if n == n { trace(1); } else { trace(0); } \
        if n < o { trace(1); } else { trace(0); } \
        if n > o { trace(1); } else { trace(0); } \
        if n <= n { trace(1); } else { trace(0); } \
        return 0; }";
    let (_, trace) = all_engines(src, "nan");
    assert_eq!(trace, vec![1, 0, 0, 0, 0]);
}

// ---------------------------------------------------------------------------
// 5. Conversions — int->f64 (exact) and f64->int (truncate toward zero, saturating).
// ---------------------------------------------------------------------------

#[test]
fn int_to_f64_exact() {
    assert_bits(
        "fn main() -> i64 { \
         let i: i64 = 10; let f: f64 = conv f64 (i); trace(f); \
         let j: i64 = -7; let g: f64 = conv f64 (j); trace(g); \
         return 0; }",
        "i2f",
        0,
        &[10.0f64, -7.0f64],
    );
}

#[test]
fn f64_to_int_truncates_toward_zero() {
    // 3.9 -> 3, -3.9 -> -3 (toward zero, not floor).
    let src = "fn main() -> i64 { \
        let a: f64 = 3.9; trace(conv i64 (a)); \
        let b: f64 = -3.9; trace(conv i64 (b)); \
        return conv i64 (a); }";
    let (ret, trace) = all_engines(src, "f2i");
    assert_eq!(trace, vec![3, -3]);
    assert_eq!(ret, 3);
}

#[test]
fn f64_to_int_saturates() {
    // +inf -> i32::MAX, -inf -> i32::MIN, and a huge finite -> i32::MAX (Rust `as`).
    // `conv i32` saturates the float; widen to i64 for the 8-byte `trace` channel.
    let src = "fn main() -> i64 { \
        let z: f64 = 0.0; let o: f64 = 1.0; \
        let pinf: f64 = o / z; trace(conv i64 (conv i32 (pinf))); \
        let ninf: f64 = (-o) / z; trace(conv i64 (conv i32 (ninf))); \
        let big: f64 = 1.0e18; trace(conv i64 (conv i32 (big))); \
        return 0; }";
    let (_, trace) = all_engines(src, "sat");
    assert_eq!(trace, vec![i32::MAX as i64, i32::MIN as i64, i32::MAX as i64]);
}

#[test]
fn int_f64_round_trip() {
    // 1234 -> f64 -> i64 recovers 1234 exactly.
    let src = "fn main() -> i64 { \
        let i: i64 = 1234; let f: f64 = conv f64 (i); let back: i64 = conv i64 (f); \
        trace(back); return back; }";
    let (ret, trace) = all_engines(src, "rt");
    assert_eq!(trace, vec![1234]);
    assert_eq!(ret, 1234);
}

// ---------------------------------------------------------------------------
// 6. A numeric loop program — Newton's-method sqrt + dot product/average.
// ---------------------------------------------------------------------------

#[test]
fn newton_sqrt_and_dot_product() {
    let src = "fn main() -> i64 { \
        let mut x: f64 = 1.0; let two: f64 = 2.0; let half: f64 = 0.5; \
        let mut i: i64 = 0; \
        while i < 8 { x = half * (x + two / x); i = i + 1; } \
        trace(x); \
        let mut dot: f64 = 0.0; let mut a: f64 = 1.0; let mut b: f64 = 4.0; let mut j: i64 = 0; \
        while j < 3 { dot = dot + a * b; a = a + 1.0; b = b + 1.0; j = j + 1; } \
        trace(dot); \
        let avg: f64 = dot / conv f64 (j); trace(avg); \
        return conv i64 (x); }";
    // Reproduce the exact computation in host Rust.
    let mut x = 1.0f64;
    for _ in 0..8 {
        x = 0.5 * (x + 2.0 / x);
    }
    let mut dot = 0.0f64;
    let (mut a, mut b) = (1.0f64, 4.0f64);
    for _ in 0..3 {
        dot += a * b;
        a += 1.0;
        b += 1.0;
    }
    let avg = dot / 3.0f64;
    assert_bits(src, "newton", 1, &[x, dot, avg]);
    assert_eq!(dot, 32.0);
}

// ---------------------------------------------------------------------------
// 7. The corpus fixture also runs through every engine (aot/stage_b auto-scan it),
//    but pin its sentinel here too.
// ---------------------------------------------------------------------------

#[test]
fn floats_fixture_runs() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/run/floats.cnr")).unwrap();
    let (ret, trace) = all_engines(&src, "fixture");
    assert_eq!(ret, 1);
    // Reproduce the fixture's Newton-8 sqrt(2) exactly (it is NOT the library
    // `sqrt`, which is correctly rounded — Newton stops one ULP short).
    let mut x = 1.0f64;
    for _ in 0..8 {
        x = 0.5 * (x + 2.0 / x);
    }
    assert_eq!(
        trace,
        vec![x.to_bits() as i64, 32.0f64.to_bits() as i64, (32.0f64 / 3.0).to_bits() as i64]
    );
}

// ---------------------------------------------------------------------------
// 8. Negative checks — no implicit int<->float, no float `%`, no float suffix.
// ---------------------------------------------------------------------------

use candor_proto::check_source_real;

fn check_codes(src: &str) -> Vec<String> {
    match check_source_real(src) {
        Ok(diags) => diags.into_iter().map(|d| d.code).collect(),
        Err(d) => vec![d.code],
    }
}

#[test]
fn mixed_int_float_arithmetic_rejected() {
    // No implicit promotion: `f64 + i64` is a type error (design 0016 §5).
    let codes = check_codes("fn main() -> i64 { let a: f64 = 1.0; let b: i64 = 2; let c: f64 = a + b; return 0; }");
    assert!(codes.iter().any(|c| c == "E0703"), "expected E0703, got {codes:?}");
}

#[test]
fn float_remainder_rejected() {
    // `%` is integer-only; no `Rem` on f64 (design 0016 §2).
    let codes = check_codes("fn main() -> i64 { let a: f64 = 5.0; let b: f64 = 2.0; let c: f64 = a % b; return 0; }");
    assert!(codes.iter().any(|c| c == "E0703"), "expected E0703, got {codes:?}");
}

#[test]
fn float_literal_takes_no_suffix() {
    // `1.5f64` is a lexer error — a float literal carries no type suffix.
    match check_source_real("fn main() -> i64 { let a: f64 = 1.5f64; return 0; }") {
        Ok(_) => panic!("expected a parse/lex error for a suffixed float literal"),
        Err(d) => assert_eq!(d.code, "L0005"),
    }
}

#[test]
fn bare_dot_float_requires_digit_both_sides() {
    // `.5` is not a float literal (needs a digit before `.`); `5.` needs one after.
    // `.5` lexes `.` then `5`, so it is a parse error in expression position.
    assert!(!check_codes("fn main() -> i64 { let a: f64 = .5; return 0; }").is_empty());
}
