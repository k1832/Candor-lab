//! Single-precision (`f32`) gate (design 0016).
//!
//! IEEE-754 binary32 is deterministic on the shared x86-64 target, so every
//! engine — the tree-walking oracle, the MIR interpreter, the Cranelift native
//! backend (no-opt and `-O2`), and the LLVM `-O2` backend — must produce
//! BIT-IDENTICAL results. Each test computes an `f32`, observes its exact 32-bit
//! pattern via `trace` (the channel all five engines share — `f32::to_bits`
//! zero-extended into the i64 word), and asserts every engine agrees and matches
//! the value computed in host Rust `f32`.
//!
//! NaN is the single exception (as for `f64`): the SIGN of a computed NaN is
//! IEEE-unspecified and differs between a constant-folding compiler and runtime
//! hardware, so NaN is gated by BEHAVIOUR (comparison outcomes), not by its bits.

use std::path::Path;
use std::process::Command;

use candor_proto::{
    check_source_real, compile_path_llvm, run_source_real, run_source_real_mir,
    run_source_real_native, run_source_real_native_opt, MirRunResult, RunResult,
};

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
    let srcp = dir.join(format!("candor-f32-{}-{}.cnr", std::process::id(), tag));
    let outp = dir.join(format!("candor-f32-{}-{}", std::process::id(), tag));
    std::fs::write(&srcp, src).unwrap();
    compile_path_llvm(Path::new(&srcp), &outp).expect("LLVM compile should succeed");
    let output = Command::new(&outp).output().expect("run compiled f32 program");
    let _ = std::fs::remove_file(&srcp);
    let _ = std::fs::remove_file(&outp);
    let trace = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.trim().parse::<i64>().expect("trace line is an integer"))
        .collect();
    Some(trace)
}

/// Run `src` through every engine, assert they all agree bit-for-bit, and return
/// the oracle's `(ret, trace)`.
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

/// The 32-bit pattern of an `f32` as it appears on the `trace` channel (zero-
/// extended into the i64 word).
fn f32_word(f: f32) -> i64 {
    f.to_bits() as i64
}

/// Assert every engine computes the traced `f32` results to the exact bit patterns
/// `expected` (host Rust `f32::to_bits`), and returns `ret_expected`.
fn assert_bits(src: &str, tag: &str, ret_expected: i64, expected: &[f32]) {
    let (ret, trace) = all_engines(src, tag);
    let want: Vec<i64> = expected.iter().map(|f| f32_word(*f)).collect();
    assert_eq!(trace, want, "traced bits mismatch for:\n{src}");
    assert_eq!(ret, ret_expected, "ret mismatch for:\n{src}");
}

fn check_codes(src: &str) -> Vec<String> {
    match check_source_real(src) {
        Ok(diags) => diags.into_iter().map(|d| d.code).collect(),
        Err(d) => vec![d.code],
    }
}

// ---------------------------------------------------------------------------
// 1. Arithmetic + precedence — exact bits across all engines.
// ---------------------------------------------------------------------------

#[test]
fn add_sub_mul_div_exact_bits() {
    assert_bits(
        "fn main() -> i64 { \
         let a: f32 = 1.5f32; let b: f32 = 2.25f32; \
         let s: f32 = a + b; trace(s); \
         let d: f32 = b - a; trace(d); \
         let m: f32 = a * b; trace(m); \
         let q: f32 = b / a; trace(q); \
         return 0; }",
        "arith",
        0,
        &[1.5f32 + 2.25, 2.25f32 - 1.5, 1.5f32 * 2.25, 2.25f32 / 1.5],
    );
}

#[test]
fn precedence_and_neg() {
    assert_bits(
        "fn main() -> i64 { \
         let a: f32 = 2.0f32; let b: f32 = 3.0f32; let c: f32 = 4.0f32; \
         let r: f32 = a + b * c; trace(r); \
         let n: f32 = -r; trace(n); \
         return 0; }",
        "prec",
        0,
        &[2.0f32 + 3.0 * 4.0, -(2.0f32 + 3.0 * 4.0)],
    );
}

#[test]
fn accumulated_rounding() {
    // 0.1 + 0.2 != 0.3 exactly in f32 either — bit-identical everywhere.
    assert_bits(
        "fn main() -> i64 { let a: f32 = 0.1f32; let b: f32 = 0.2f32; let r: f32 = a + b; trace(r); return 0; }",
        "round",
        0,
        &[0.1f32 + 0.2f32],
    );
}

// ---------------------------------------------------------------------------
// 2. Comparisons (ordered) — including negatives.
// ---------------------------------------------------------------------------

#[test]
fn ordered_comparisons() {
    let src = "fn main() -> i64 { \
        let a: f32 = -1.5f32; let b: f32 = 2.5f32; \
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
// 3. inf — deterministic bits (overflow / divide-by-zero do NOT fault).
// ---------------------------------------------------------------------------

#[test]
fn infinities_exact_bits() {
    assert_bits(
        "fn main() -> i64 { \
         let z: f32 = 0.0f32; let o: f32 = 1.0f32; \
         let pinf: f32 = o / z; trace(pinf); \
         let ninf: f32 = (-o) / z; trace(ninf); \
         return 0; }",
        "inf",
        0,
        &[f32::INFINITY, f32::NEG_INFINITY],
    );
}

// ---------------------------------------------------------------------------
// 4. NaN — gated by BEHAVIOUR (its sign bit is IEEE-unspecified across engines).
// ---------------------------------------------------------------------------

#[test]
fn nan_comparisons_are_ieee() {
    let src = "fn main() -> i64 { \
        let z: f32 = 0.0f32; let n: f32 = z / z; let o: f32 = 1.0f32; \
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
// 5. Conversions — int<->f32 and f32<->f64.
// ---------------------------------------------------------------------------

#[test]
fn int_to_f32_exact() {
    assert_bits(
        "fn main() -> i64 { \
         let i: i64 = 10; let f: f32 = conv f32 (i); trace(f); \
         let j: i64 = -7; let g: f32 = conv f32 (j); trace(g); \
         return 0; }",
        "i2f",
        0,
        &[10.0f32, -7.0f32],
    );
}

#[test]
fn f32_to_int_truncates_toward_zero() {
    let src = "fn main() -> i64 { \
        let a: f32 = 3.9f32; trace(conv i64 (a)); \
        let b: f32 = -3.9f32; trace(conv i64 (b)); \
        return conv i64 (a); }";
    let (ret, trace) = all_engines(src, "f2i");
    assert_eq!(trace, vec![3, -3]);
    assert_eq!(ret, 3);
}

#[test]
fn f32_to_int_saturates() {
    // +inf -> i32::MAX, -inf -> i32::MIN, and a huge finite -> i32::MAX (Rust `as`).
    let src = "fn main() -> i64 { \
        let z: f32 = 0.0f32; let o: f32 = 1.0f32; \
        let pinf: f32 = o / z; trace(conv i64 (conv i32 (pinf))); \
        let ninf: f32 = (-o) / z; trace(conv i64 (conv i32 (ninf))); \
        let big: f32 = 1.0e18f32; trace(conv i64 (conv i32 (big))); \
        return 0; }";
    let (_, trace) = all_engines(src, "sat");
    assert_eq!(trace, vec![i32::MAX as i64, i32::MIN as i64, i32::MAX as i64]);
}

#[test]
fn int_f32_round_trip() {
    let src = "fn main() -> i64 { \
        let i: i64 = 1234; let f: f32 = conv f32 (i); let back: i64 = conv i64 (f); \
        trace(back); return back; }";
    let (ret, trace) = all_engines(src, "rt");
    assert_eq!(trace, vec![1234]);
    assert_eq!(ret, 1234);
}

#[test]
fn f32_to_f64_widening_is_exact() {
    // Widening f32 -> f64 is exact: the f64 equals the f32 value promoted.
    let src = "fn main() -> i64 { \
        let a: f32 = 0.1f32; let w: f64 = conv f64 (a); trace(w); \
        return 0; }";
    let (_, trace) = all_engines(src, "widen");
    assert_eq!(trace, vec![(0.1f32 as f64).to_bits() as i64]);
}

#[test]
fn f64_to_f32_narrowing_rounds_and_loses_precision() {
    // 0.1 as f64 is NOT representable in f32; narrowing rounds, so the
    // round-trip f64 -> f32 -> f64 differs from the original f64 bits.
    let src = "fn main() -> i64 { \
        let d: f64 = 0.1; let n: f32 = conv f32 (d); trace(n); \
        let back: f64 = conv f64 (n); trace(back); \
        return 0; }";
    let (_, trace) = all_engines(src, "narrow");
    let narrowed = 0.1f64 as f32;
    assert_eq!(trace, vec![narrowed.to_bits() as i64, (narrowed as f64).to_bits() as i64]);
    // The narrowed-then-widened value differs from the original f64 (precision loss).
    assert_ne!((narrowed as f64).to_bits(), 0.1f64.to_bits());
}

// ---------------------------------------------------------------------------
// 6. A numeric loop program — Newton's-method sqrt + dot product/average, in f32.
// ---------------------------------------------------------------------------

#[test]
fn newton_sqrt_and_dot_product() {
    let src = "fn main() -> i64 { \
        let mut x: f32 = 1.0f32; let two: f32 = 2.0f32; let half: f32 = 0.5f32; \
        let mut i: i64 = 0; \
        while i < 8 { x = half * (x + two / x); i = i + 1; } \
        trace(x); \
        let mut dot: f32 = 0.0f32; let mut a: f32 = 1.0f32; let mut b: f32 = 4.0f32; let mut j: i64 = 0; \
        while j < 3 { dot = dot + a * b; a = a + 1.0f32; b = b + 1.0f32; j = j + 1; } \
        trace(dot); \
        let avg: f32 = dot / conv f32 (j); trace(avg); \
        return conv i64 (x); }";
    let mut x = 1.0f32;
    for _ in 0..8 {
        x = 0.5 * (x + 2.0 / x);
    }
    let mut dot = 0.0f32;
    let (mut a, mut b) = (1.0f32, 4.0f32);
    for _ in 0..3 {
        dot += a * b;
        a += 1.0;
        b += 1.0;
    }
    let avg = dot / 3.0f32;
    assert_bits(src, "newton", 1, &[x, dot, avg]);
    assert_eq!(dot, 32.0);
}

#[test]
fn floats_f32_fixture_runs() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/run/floats_f32.cnr")).unwrap();
    let (ret, trace) = all_engines(&src, "fixture");
    assert_eq!(ret, 1);
    let mut x = 1.0f32;
    for _ in 0..8 {
        x = 0.5 * (x + 2.0 / x);
    }
    let avg = 32.0f32 / 3.0f32;
    assert_eq!(
        trace,
        vec![
            x.to_bits() as i64,
            32.0f32.to_bits() as i64,
            avg.to_bits() as i64,
            (avg as f64).to_bits() as i64,
        ]
    );
}

// ---------------------------------------------------------------------------
// 7. Negative checks — no implicit int<->f32 / f32<->f64, no f32 `%`, suffix rules.
// ---------------------------------------------------------------------------

#[test]
fn mixed_f32_int_arithmetic_rejected() {
    let codes = check_codes("fn main() -> i64 { let a: f32 = 1.0f32; let b: i64 = 2; let c: f32 = a + b; return 0; }");
    assert!(codes.iter().any(|c| c == "E0703"), "expected E0703, got {codes:?}");
}

#[test]
fn mixed_f32_f64_arithmetic_rejected() {
    // No implicit f32<->f64 promotion: the two float widths do not mix.
    let codes = check_codes("fn main() -> i64 { let a: f32 = 1.0f32; let b: f64 = 2.0; let c: f32 = a + b; return 0; }");
    assert!(codes.iter().any(|c| c == "E0703"), "expected E0703, got {codes:?}");
}

#[test]
fn f32_remainder_rejected() {
    let codes = check_codes("fn main() -> i64 { let a: f32 = 5.0f32; let b: f32 = 2.0f32; let c: f32 = a % b; return 0; }");
    assert!(codes.iter().any(|c| c == "E0703"), "expected E0703, got {codes:?}");
}

#[test]
fn f32_suffix_requires_float_form() {
    // The `f32` suffix attaches only to a float-form literal; `10f32` (integer
    // form) is rejected as an invalid integer-literal suffix.
    match check_source_real("fn main() -> i64 { let a: f32 = 10f32; return 0; }") {
        Ok(_) => panic!("expected a lex error for an integer-form f32 literal"),
        Err(d) => assert_eq!(d.code, "L0005"),
    }
}

#[test]
fn only_f32_is_a_valid_float_suffix() {
    // A float form may carry the `f32` suffix, but no other (e.g. `1.5f64`).
    match check_source_real("fn main() -> i64 { let a: f64 = 1.5f16; return 0; }") {
        Ok(_) => panic!("expected a lex error for an unknown float suffix"),
        Err(d) => assert_eq!(d.code, "L0005"),
    }
}
