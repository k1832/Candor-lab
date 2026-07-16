//! Correctly-rounded `sqrt` intrinsic gate (design 0016 §11).
//!
//! IEEE square root is correctly-rounded and deterministic on the shared x86-64
//! target, so every engine — the tree-walking oracle, the MIR interpreter, the
//! Cranelift native backend (no-opt and `-O2`), and the LLVM `-O2` backend — must
//! produce BIT-IDENTICAL results. Each test computes a `sqrt` and observes its
//! exact bit pattern via `trace` (the channel all five engines share), asserting
//! every engine agrees and matches the value computed in host Rust (`f*::sqrt`,
//! itself correctly-rounded). Both backends emit the NATIVE sqrt (Cranelift's
//! `sqrt` instruction / the `llvm.sqrt` intrinsic), never a software approximation.
//!
//! A negative argument yields NaN, whose SIGN is IEEE-unspecified across a
//! constant-folding compiler vs runtime hardware — so the negative case is gated
//! by IS-NAN behaviour (a comparison outcome), not by its exact bit pattern.

use std::path::Path;
use std::process::Command;

use candor::{
    compile_path_llvm, run_source_real, run_source_real_mir, run_source_real_native,
    run_source_real_native_opt, MirRunResult, RunResult,
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
        MirRunResult::CheckErrors(d) => {
            panic!("{label} check errors: {:?}\n{src}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
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
    let srcp = dir.join(format!("candor-sqrt-{}-{}.cnr", std::process::id(), tag));
    let outp = dir.join(format!("candor-sqrt-{}-{}", std::process::id(), tag));
    std::fs::write(&srcp, src).unwrap();
    compile_path_llvm(Path::new(&srcp), &outp).expect("LLVM compile should succeed");
    let output = Command::new(&outp).output().expect("run compiled sqrt program");
    let _ = std::fs::remove_file(&srcp);
    let _ = std::fs::remove_file(&outp);
    let trace = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.trim().parse::<i64>().expect("trace line is an integer"))
        .collect();
    Some(trace)
}

/// Run `src` through every engine, assert they agree bit-for-bit (`ret` for the
/// four in-process engines; `trace` for all five including LLVM), return the
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

// ---------------------------------------------------------------------------
// f64 — known values, exact bits (negative -> NaN by behaviour).
// ---------------------------------------------------------------------------

#[test]
fn f64_known_values_exact_bits() {
    let src = std::fs::read_to_string("tests/fixtures/run/sqrt.cnr").unwrap();
    let (ret, trace) = all_engines(&src, "f64");
    let rt = 2.0f64.sqrt() * 2.0f64.sqrt();
    let want: Vec<i64> = vec![
        (2.0f64).to_bits() as i64,        // sqrt(4.0) == 2.0
        (2.0f64.sqrt()).to_bits() as i64, // sqrt(2.0), exact bits
        (0.0f64).to_bits() as i64,        // sqrt(0.0) == 0.0
        (-0.0f64).to_bits() as i64,       // sqrt(-0.0) == -0.0
        (1e100f64.sqrt()).to_bits() as i64,
        1,                                // sqrt(-1.0) is NaN (behaviour)
        rt.to_bits() as i64,              // round-trip sqrt(x)*sqrt(x)
    ];
    assert_eq!(trace, want, "f64 sqrt bits mismatch");
    assert_eq!(ret, 42, "sqrt(4.0)==2.0 -> 21*2 == 42");
    // sqrt(4.0) is exactly 2.0.
    assert_eq!(trace[0], 2.0f64.to_bits() as i64);
    // -0.0 is a distinct bit pattern (sign bit set), not +0.0.
    assert_ne!(trace[3], trace[2]);
}

// ---------------------------------------------------------------------------
// f32 — known values, exact bits.
// ---------------------------------------------------------------------------

#[test]
fn f32_known_values_exact_bits() {
    let src = std::fs::read_to_string("tests/fixtures/run/sqrt_f32.cnr").unwrap();
    let (ret, trace) = all_engines(&src, "f32");
    let rt = 2.0f32.sqrt() * 2.0f32.sqrt();
    let want: Vec<i64> = vec![
        (2.0f32).to_bits() as i64,
        (2.0f32.sqrt()).to_bits() as i64,
        (0.0f32).to_bits() as i64,
        (-0.0f32).to_bits() as i64,
        (1e30f32.sqrt()).to_bits() as i64,
        1,
        rt.to_bits() as i64,
    ];
    assert_eq!(trace, want, "f32 sqrt bits mismatch");
    assert_eq!(ret, 42);
    assert_ne!(trace[3], trace[2]);
}

// ---------------------------------------------------------------------------
// Negative -> NaN is a value, not a fault (total op); IEEE comparison outcomes.
// ---------------------------------------------------------------------------

#[test]
fn negative_sqrt_is_nan_not_a_fault() {
    // sqrt of a negative is NaN (not a fault). NaN != NaN is true; every ordered
    // comparison with it is false.
    let src = "fn main() -> i64 { \
        let neg: f64 = -2.0; let n: f64 = sqrt(neg); let o: f64 = 1.0; \
        if n != n { trace(1); } else { trace(0); } \
        if n == n { trace(1); } else { trace(0); } \
        if n < o { trace(1); } else { trace(0); } \
        return 0; }";
    let (_, trace) = all_engines(src, "nan_neg");
    assert_eq!(trace, vec![1, 0, 0]);
}
