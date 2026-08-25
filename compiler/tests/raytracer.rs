//! Ray-tracer corpus twin gate (design 0016) — the differential story behind
//! examples/15_raytracer.cnr.
//!
//! The fixture (`tests/fixtures/run/raytracer.cnr`) renders the example's exact
//! scene at 64x48 — four spheres (one mirror) on a checkerboard plane, a
//! directional sun with hard shadows, 2x2 supersampling, iterative reflection
//! (depth cap 4), sqrt-gamma output — using only IEEE-754 f64 `+ - * /`,
//! ordered comparison, int<->f64 `conv`, and the correctly-rounded `sqrt`
//! intrinsic. All of those are bit-deterministic on the shared x86-64 target,
//! so every engine — the tree-walking oracle, the MIR interpreter, the
//! Cranelift native backend (no-opt and `-O2`), and the LLVM `-O2` backend —
//! must produce the SAME image, byte for byte. The fixture traces the RGB
//! bytes of three probe pixels (sky, mirror sphere, shadowed floor — chosen so
//! a shading regression names its own cause) plus a checksum folded over all
//! 9,216 pixel bytes; this test pins the exact values and requires all five
//! engines to agree. The run/ corpus auto-scans (tests/stage_a, stage_b,
//! stage_d, aot, llvm) re-run the fixture as well, so the render is also gated
//! through the linked-ELF paths.
//!
//! KNOWN TOOLCHAIN BUG, found while building this tracer and deliberately
//! routed around: a float->int `conv` to a SUB-32-BIT target (`conv u8`,
//! `conv u16`, `conv i8` of an f64) panics both Cranelift backends (JIT and
//! AOT) inside cranelift-codegen's x64 emitter — `eval_float_conv`
//! (src/backend/lower.rs) asks `fcvt_to_{s,u}int_sat` for an I8/I16 result,
//! which the x64 ISA lowering cannot emit ("internal error: entered
//! unreachable code", cranelift-codegen-0.132.3 src/isa/x64/inst/emit.rs:1247).
//! Minimal reproduction:
//!
//! ```text
//! fn main() -> i64 { let a: f64 = 6.4; let b: u8 = conv u8 (a); return conv i64 (b); }
//! ```
//!
//! The tree-walker and MIR interpreter execute it fine (saturating narrow),
//! so this is an engine-parity hole, not a semantics question. Until it is
//! fixed, float->int `conv` in corpus code targets i32/i64 and narrows in the
//! integer domain (clamp, then int->int `conv`) — `to_byte` in the fixture is
//! the shape.

use std::path::Path;
use std::process::Command;

use candor::{
    compile_path_llvm, run_source_real, run_source_real_mir, run_source_real_native,
    run_source_real_native_opt, MirRunResult, RunResult,
};

/// The pinned trace: probe pixels in row-major visit order — sky (5,5),
/// mirror sphere (32,20), shadowed floor (53,33) — each as R,G,B bytes, then
/// the checksum folded over every pixel byte of the 64x48 render.
const WANT_TRACE: [i64; 10] = [201, 219, 251, 177, 201, 241, 77, 81, 87, -2339203916046775192];
const WANT_RET: i64 = 42;

fn fixture() -> String {
    let path = format!("{}/tests/fixtures/run/raytracer.cnr", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn oracle(src: &str) -> (i64, Vec<i64>) {
    match run_source_real(src) {
        RunResult::Ok(r) => (r.ret, r.trace),
        RunResult::Fault(f) => panic!("oracle faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => {
            panic!("oracle check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("oracle parse error: {}", d.to_json()),
    }
}

fn mir_out(r: MirRunResult, label: &str) -> (i64, Vec<i64>) {
    match r {
        MirRunResult::Ok(r) => (r.ret, r.trace),
        MirRunResult::Fault(f) => panic!("{label} faulted: {}", f.to_json()),
        MirRunResult::Unsupported(e) => panic!("{label} unsupported: {e}"),
        MirRunResult::CheckErrors(d) => {
            panic!("{label} check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        MirRunResult::ParseError(d) => panic!("{label} parse error: {}", d.to_json()),
    }
}

fn clang_available() -> bool {
    Command::new("clang").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

/// The LLVM `-O2` process's traced values, or `None` when clang is absent.
fn llvm_trace(src: &str) -> Option<Vec<i64>> {
    if !clang_available() {
        return None;
    }
    let dir = std::env::temp_dir();
    let srcp = dir.join(format!("candor-raytracer-{}.cnr", std::process::id()));
    let outp = dir.join(format!("candor-raytracer-{}", std::process::id()));
    std::fs::write(&srcp, src).unwrap();
    compile_path_llvm(Path::new(&srcp), &outp).expect("LLVM compile should succeed");
    let output = Command::new(&outp).output().expect("run compiled raytracer");
    let _ = std::fs::remove_file(&srcp);
    let _ = std::fs::remove_file(&outp);
    let trace = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.trim().parse::<i64>().expect("trace line is an integer"))
        .collect();
    Some(trace)
}

#[test]
fn raytracer_image_pinned_all_engines() {
    let src = fixture();
    let (o_ret, o_trace) = oracle(&src);
    assert_eq!(o_trace, WANT_TRACE, "oracle probe pixels / checksum diverged from the pinned image");
    assert_eq!(o_ret, WANT_RET);

    for (label, r) in [
        ("mir", run_source_real_mir(&src)),
        ("native-noopt", run_source_real_native(&src)),
        ("native-opt", run_source_real_native_opt(&src)),
    ] {
        let (ret, trace) = mir_out(r, label);
        assert_eq!(trace, o_trace, "{label} image diverged from oracle");
        assert_eq!(ret, o_ret, "{label} ret diverged from oracle");
    }

    if let Some(l_trace) = llvm_trace(&src) {
        assert_eq!(l_trace, o_trace, "llvm image diverged from oracle");
    }
}
