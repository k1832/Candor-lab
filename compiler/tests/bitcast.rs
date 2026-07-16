//! `bitcast` gate (design 0016 §10) — same-width float<->int BIT reinterpretation.
//!
//! `bitcast T (e)` reinterprets the IDENTICAL bits of `e` as `T` (a same-byte-width
//! float<->integer pair). Unlike `conv` (which converts the numeric VALUE), bitcast
//! changes no bits, never faults, and is regime-independent. It is deterministic on
//! the shared x86-64 target, so every engine — the tree-walking oracle, the MIR
//! interpreter, Cranelift (no-opt and `-O2`), and LLVM `-O2` — produces
//! BIT-IDENTICAL results, observed via the `trace` channel (`{f32,f64}::to_bits`).
//!
//! Bitcast is precisely where a SPECIFIC NaN bit-pattern is preserved exactly:
//! there is no arithmetic to canonicalize the payload, so the gate asserts the exact
//! NaN bits survive a round trip (unlike a computed NaN, whose sign is
//! IEEE-unspecified — see `tests/floats.rs`).
//!
//! The `trace` channel is 64-bit, so a 32-bit result is widened with a
//! `conv i64 (bitcast u32 (..))`: bitcasting to the UNSIGNED 32-bit int makes the
//! widening zero-extend, matching host `f32::to_bits()` (a `u32`).

use std::path::Path;
use std::process::Command;

use candor::{
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

fn llvm_trace(src: &str, tag: &str) -> Option<Vec<i64>> {
    if !clang_available() {
        return None;
    }
    let dir = std::env::temp_dir();
    let srcp = dir.join(format!("candor-bitcast-{}-{}.cnr", std::process::id(), tag));
    let outp = dir.join(format!("candor-bitcast-{}-{}", std::process::id(), tag));
    std::fs::write(&srcp, src).unwrap();
    compile_path_llvm(Path::new(&srcp), &outp).expect("LLVM compile should succeed");
    let output = Command::new(&outp).output().expect("run compiled program");
    let _ = std::fs::remove_file(&srcp);
    let _ = std::fs::remove_file(&outp);
    let trace = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.trim().parse::<i64>().expect("trace line is an integer"))
        .collect();
    Some(trace)
}

/// Run `src` through every engine, assert they all agree bit-for-bit, return θ.
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

fn check_codes(src: &str) -> Vec<String> {
    match check_source_real(src) {
        Ok(diags) => diags.into_iter().map(|d| d.code).collect(),
        Err(d) => vec![d.code],
    }
}

// ---------------------------------------------------------------------------
// 1. Round trip — `bitcast f64 (bitcast i64 x) == x`, bit-identical everywhere.
// ---------------------------------------------------------------------------

#[test]
fn f64_i64_round_trip() {
    // A spread: normal, negative, +0.0, -0.0, huge, scientific.
    let src = "fn main() -> i64 { \
        let vs: [6]f64 = [123.456, -2.5, 0.0, -0.0, 1.0e300, 6.022e23]; \
        let mut i: i64 = 0; \
        while i < 6 { \
            let x: f64 = vs[conv usize (i)]; \
            let bits: i64 = bitcast i64 (x); \
            let back: f64 = bitcast f64 (bits); \
            trace(back); \
            i = i + 1; \
        } \
        return 0; }";
    let (_, trace) = all_engines(src, "rt64");
    let want: Vec<i64> = [123.456f64, -2.5, 0.0, -0.0, 1.0e300, 6.022e23]
        .iter()
        .map(|f| f.to_bits() as i64)
        .collect();
    assert_eq!(trace, want);
}

#[test]
fn f32_i32_round_trip() {
    let src = "fn main() -> i64 { \
        let vs: [5]f32 = [2.5f32, -7.25f32, 0.0f32, 1.0e30f32, 1.5e-20f32]; \
        let mut i: i64 = 0; \
        while i < 5 { \
            let x: f32 = vs[conv usize (i)]; \
            let bits: i32 = bitcast i32 (x); \
            let back: f32 = bitcast f32 (bits); \
            trace(back); \
            i = i + 1; \
        } \
        return 0; }";
    let (_, trace) = all_engines(src, "rt32");
    let want: Vec<i64> = [2.5f32, -7.25f32, 0.0f32, 1.0e30f32, 1.5e-20f32]
        .iter()
        .map(|f| f.to_bits() as i64)
        .collect();
    assert_eq!(trace, want);
}

// ---------------------------------------------------------------------------
// 2. Known bit patterns — exact IEEE encodings, both directions, f64 + f32.
// ---------------------------------------------------------------------------

#[test]
fn known_patterns_f64() {
    // bitcast i64 (1.0) == 0x3FF0000000000000 ; bitcast f64 (0x4000000000000000) == 2.0.
    let src = "fn main() -> i64 { \
        trace(bitcast i64 (1.0)); \
        let two: f64 = bitcast f64 (0x4000000000000000); trace(two); \
        trace(bitcast i64 (-2.0)); \
        return 0; }";
    let (_, trace) = all_engines(src, "kp64");
    assert_eq!(
        trace,
        vec![
            1.0f64.to_bits() as i64,     // 0x3FF0000000000000
            2.0f64.to_bits() as i64,     // 0x4000000000000000
            (-2.0f64).to_bits() as i64,  // 0xC000000000000000
        ]
    );
    assert_eq!(trace[0], 0x3FF0000000000000i64);
    assert_eq!(trace[1], 0x4000000000000000i64);
}

#[test]
fn known_patterns_f32() {
    // bitcast i32 (1.0f32) == 0x3F800000 ; bitcast f32 (0x3F800000) == 1.0f32.
    // 32-bit results are widened via `bitcast u32` so `conv i64` zero-extends.
    let src = "fn main() -> i64 { \
        trace(conv i64 (bitcast u32 (1.0f32))); \
        let one: f32 = bitcast f32 (0x3F800000); trace(one); \
        trace(conv i64 (bitcast u32 (-1.0f32))); \
        return 0; }";
    let (_, trace) = all_engines(src, "kp32");
    assert_eq!(
        trace,
        vec![
            1.0f32.to_bits() as i64,     // 0x3F800000
            1.0f32.to_bits() as i64,
            (-1.0f32).to_bits() as i64,  // 0xBF800000
        ]
    );
    assert_eq!(trace[0], 0x3F800000i64);
}

// ---------------------------------------------------------------------------
// 3. NaN bits preserved EXACTLY — the case bitcast uniquely handles (no
//    arithmetic, so the specific payload survives a round trip on every engine).
// ---------------------------------------------------------------------------

#[test]
fn nan_payload_survives_exactly() {
    // A specific quiet-NaN payload: 0x7FF8000000000001. Reinterpreted to f64 and
    // back to i64 WITHOUT any arithmetic, the exact bits (payload + sign) survive.
    let src = "fn main() -> i64 { \
        let n: f64 = bitcast f64 (0x7FF8000000000001); \
        let bits: i64 = bitcast i64 (n); \
        trace(bits); \
        return 0; }";
    let (_, trace) = all_engines(src, "nan64");
    assert_eq!(trace, vec![0x7FF8000000000001i64]);

    // f32 signalling-NaN payload 0x7FA00001 survives identically.
    let src32 = "fn main() -> i64 { \
        let n: f32 = bitcast f32 (0x7FA00001); \
        trace(conv i64 (bitcast u32 (n))); \
        return 0; }";
    let (_, trace32) = all_engines(src32, "nan32");
    assert_eq!(trace32, vec![0x7FA00001i64]);
}

// ---------------------------------------------------------------------------
// 4. Rejection — a different-width (or non-float<->int) bitcast is a check error.
// ---------------------------------------------------------------------------

#[test]
fn different_width_rejected() {
    // f64 (8 bytes) <-> i32 (4 bytes): different width, rejected at check time.
    let codes = check_codes("fn main() -> i64 { let v: i32 = 5i32; let f: f64 = bitcast f64 (v); return 0; }");
    assert!(codes.iter().any(|c| c == "E0714"), "expected E0714, got {codes:?}");

    let codes = check_codes("fn main() -> i64 { let x: f64 = 1.0; let v: i32 = bitcast i32 (x); return 0; }");
    assert!(codes.iter().any(|c| c == "E0714"), "expected E0714, got {codes:?}");

    // f32 (4 bytes) <-> i64 (8 bytes): different width, rejected.
    let codes = check_codes("fn main() -> i64 { let x: f32 = 1.0f32; let v: i64 = bitcast i64 (x); return 0; }");
    assert!(codes.iter().any(|c| c == "E0714"), "expected E0714, got {codes:?}");
}

#[test]
fn non_float_int_pair_rejected() {
    // i64 <-> u64 is NOT a bitcast (same width, but both integers — that is
    // `wrapping { conv }`); bitcast requires exactly one float side.
    let codes = check_codes("fn main() -> i64 { let v: u64 = 5u64; let w: i64 = bitcast i64 (v); return 0; }");
    assert!(codes.iter().any(|c| c == "E0714"), "expected E0714, got {codes:?}");

    // f64 <-> f32 is not a bitcast (different width, and both float) — use `conv`.
    let codes = check_codes("fn main() -> i64 { let x: f64 = 1.0; let y: f32 = bitcast f32 (x); return 0; }");
    assert!(codes.iter().any(|c| c == "E0714"), "expected E0714, got {codes:?}");
}

// ---------------------------------------------------------------------------
// 5. Regime independence — bitcast inside a `wrapping` block is unchanged and
//    still never faults (it is a pure reinterpret, exempt from the regime system).
// ---------------------------------------------------------------------------

#[test]
fn regime_independent() {
    let src = "fn main() -> i64 { \
        let x: f64 = -3.5; \
        let mut bits: i64 = 0; \
        wrapping { bits = bitcast i64 (x); } \
        let back: f64 = bitcast f64 (bits); \
        trace(back); \
        return 0; }";
    let (_, trace) = all_engines(src, "regime");
    assert_eq!(trace, vec![(-3.5f64).to_bits() as i64]);
}

// ---------------------------------------------------------------------------
// 6. The corpus fixture also runs through every native engine (aot/stage scan it).
// ---------------------------------------------------------------------------

#[test]
fn bitcast_fixture_runs() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/run/bitcast.cnr")).unwrap();
    let (ret, _trace) = all_engines(&src, "fixture");
    assert_eq!(ret, 1);
}
