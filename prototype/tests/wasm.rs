//! M0 of a WebAssembly interpreter written IN Candor (the decode-and-run spine).
//!
//! The interpreter (`tests/fixtures/wasm/interp.cnr`) decodes a REAL `.wasm`
//! binary module — magic + version, an id-dispatched section walk, real LEB128
//! — and runs its single function on a value stack, in the ordinary Candor
//! subset. So it runs on the tree-walking oracle, the MIR interpreter, and both
//! Cranelift native modes, and this gate asserts they agree on the result.
//!
//! `wasm_add_module` is an INDEPENDENT Rust encoder of the same one-function
//! `(result i32)` add module. Two independent LEB128 implementations (Rust
//! encode, Candor decode) agreeing on the answer is the real correctness signal:
//! the M0 module is asserted byte-identical to the spec's bytes, and larger
//! (multi-byte, negative) operands round-trip too — nothing is hardcoded to 42.

use candor_proto::interp::FaultKind;
use candor_proto::{
    run_source_real, run_source_real_mir, run_source_real_native, run_source_real_native_opt,
    MirRunResult, RunResult,
};

// The reusable interpreter functions (everything above the harness-split line in
// interp.cnr). The file's own `fn main` runs the M0 module; here we append our
// own `main` to feed the decoder other real modules.
const INTERP_SRC: &str = include_str!("fixtures/wasm/interp.cnr");
const SPLIT: &str = "// === HARNESS SPLIT:";

fn interp_fns() -> &'static str {
    INTERP_SRC.split(SPLIT).next().expect("interp.cnr must contain the harness-split sentinel")
}

// ---- an independent real-`.wasm` encoder ----------------------------------

/// Signed LEB128 of `v`, appended to `out`.
fn leb_i32(mut v: i32, out: &mut Vec<u8>) {
    loop {
        let byte = (v & 0x7f) as u8;
        v >>= 7;
        let done = (v == 0 && byte & 0x40 == 0) || (v == -1 && byte & 0x40 != 0);
        out.push(if done { byte } else { byte | 0x80 });
        if done {
            return;
        }
    }
}

/// The real binary of `(module (func (result i32) i32.const a i32.const b i32.add))`.
/// Section sizes are single-byte LEB (all our modules are well under 128 bytes).
fn wasm_add_module(a: i32, b: i32) -> Vec<u8> {
    let mut m: Vec<u8> = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    let section = |m: &mut Vec<u8>, id: u8, payload: &[u8]| {
        m.push(id);
        m.push(payload.len() as u8);
        m.extend_from_slice(payload);
    };
    // Type: 1 functype, 0 params, 1 result i32 (0x7f).
    section(&mut m, 0x01, &[0x01, 0x60, 0x00, 0x01, 0x7f]);
    // Function: 1 func, type index 0.
    section(&mut m, 0x03, &[0x01, 0x00]);
    // Code: 1 body — 0 locals, i32.const a, i32.const b, i32.add, end.
    let mut body: Vec<u8> = vec![0x00];
    body.push(0x41);
    leb_i32(a, &mut body);
    body.push(0x41);
    leb_i32(b, &mut body);
    body.push(0x6a);
    body.push(0x0b);
    let mut code = vec![0x01, body.len() as u8];
    code.extend_from_slice(&body);
    section(&mut m, 0x0a, &code);
    m
}

/// Build a full Candor program: the interpreter functions plus a `main` that
/// embeds `bytes` as a fixed `[N]u8` and decodes+runs them.
fn program(bytes: &[u8]) -> String {
    let mut lits = String::new();
    for (i, byte) in bytes.iter().enumerate() {
        if i > 0 {
            lits.push_str(", ");
        }
        lits.push_str(&format!("0x{byte:02x}u8"));
    }
    format!(
        "{}\nfn main() -> i64 {{\n    let data: [{}]u8 = [{}];\n    return run_module(slice_of(data));\n}}\n",
        interp_fns(),
        bytes.len(),
        lits,
    )
}

// ---- all-engine drivers (oracle · MIR · native-noopt · native-opt) ---------

fn run_ret_all(src: &str) -> i64 {
    let o = match run_source_real(src) {
        RunResult::Ok(r) => r,
        RunResult::Fault(f) => panic!("oracle faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => {
            panic!("check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("parse error: {}", d.to_json()),
    };
    for (label, r) in [
        ("mir", run_source_real_mir(src)),
        ("native-noopt", run_source_real_native(src)),
        ("native-opt", run_source_real_native_opt(src)),
    ] {
        match r {
            MirRunResult::Ok(run) => assert_eq!(run.ret, o.ret, "{label} diverged from oracle"),
            MirRunResult::Fault(f) => panic!("{label} faulted: {}", f.to_json()),
            MirRunResult::Unsupported(m) => panic!("{label} unsupported: {m}"),
            MirRunResult::CheckErrors(d) => {
                panic!("{label} check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
            }
            MirRunResult::ParseError(d) => panic!("{label} parse error: {}", d.to_json()),
        }
    }
    o.ret
}

fn run_fault_all(src: &str) -> FaultKind {
    let of = match run_source_real(src) {
        RunResult::Fault(f) => f,
        RunResult::Ok(r) => panic!("expected fault, got ret {}", r.ret),
        RunResult::CheckErrors(d) => {
            panic!("expected fault, got check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("expected fault, got parse error: {}", d.to_json()),
    };
    for (label, r) in [
        ("mir", run_source_real_mir(src)),
        ("native-noopt", run_source_real_native(src)),
        ("native-opt", run_source_real_native_opt(src)),
    ] {
        match r {
            MirRunResult::Fault(f) => {
                assert_eq!(f.kind, of.kind, "{label} fault kind diverged from oracle")
            }
            MirRunResult::Ok(run) => panic!("{label}: expected fault, got ret {}", run.ret),
            MirRunResult::Unsupported(m) => panic!("{label} unsupported: {m}"),
            MirRunResult::CheckErrors(d) => {
                panic!("{label} check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
            }
            MirRunResult::ParseError(d) => panic!("{label} parse error: {}", d.to_json()),
        }
    }
    of.kind
}

// ---- the M0 gate -----------------------------------------------------------

/// The exact bytes from the M0 spec (30 bytes: the "34" in the brief miscounts
/// the listing, which is itself valid `.wasm`).
const M0_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, // \0asm, version 1
    0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, // Type: () -> (i32)
    0x03, 0x02, 0x01, 0x00, // Function: func 0 : type 0
    0x0a, 0x09, 0x01, 0x07, 0x00, 0x41, 0x28, 0x41, 0x02, 0x6a, 0x0b, // Code: 40 + 2
];

#[test]
fn encoder_matches_spec_bytes() {
    // The independent Rust encoder reproduces the spec's module exactly.
    assert_eq!(wasm_add_module(40, 2), M0_MODULE);
}

#[test]
fn standalone_interp_returns_42() {
    // interp.cnr as shipped (its own `main` embeds the M0 module).
    assert_eq!(run_ret_all(INTERP_SRC), 42);
}

#[test]
fn m0_module_decodes_and_evals_to_42() {
    assert_eq!(run_ret_all(&program(M0_MODULE)), 42);
}

#[test]
fn different_const_add() {
    // A distinct module: 10 + 20 = 30 (single-byte operands).
    assert_eq!(run_ret_all(&program(&wasm_add_module(10, 20))), 30);
}

#[test]
fn multibyte_and_negative_leb() {
    // 1000 and -500 both need multi-byte signed LEB; 1000 + (-500) = 500.
    // Exercises the continuation-bit loop AND sign extension in the decoder.
    assert_eq!(run_ret_all(&program(&wasm_add_module(1000, -500))), 500);
}

#[test]
fn i32_add_wraps() {
    // i32::MAX + 1 wraps to i32::MIN under WASM's mod-2^32 add.
    assert_eq!(run_ret_all(&program(&wasm_add_module(i32::MAX, 1))), i32::MIN as i64);
}

#[test]
fn malformed_magic_faults() {
    let mut bad = M0_MODULE.to_vec();
    bad[0] = 0xff; // corrupt the first magic byte
    assert_eq!(run_fault_all(&program(&bad)), FaultKind::Panic);
}

#[test]
fn corpus_copy_matches_canonical() {
    // The `run/` corpus copy (auto-enlisted in the AOT/LLVM/stage_d gates) must
    // stay byte-identical to the canonical interpreter this harness runs.
    assert_eq!(
        include_str!("fixtures/wasm/interp.cnr"),
        include_str!("fixtures/run/wasm_interp.cnr"),
        "tests/fixtures/run/wasm_interp.cnr drifted from tests/fixtures/wasm/interp.cnr"
    );
}
