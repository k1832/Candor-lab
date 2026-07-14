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

// ---- M1 encoder: multi-function modules, locals, control flow -------------
//
// Same discipline as M0: an INDEPENDENT Rust encoder builds real `.wasm`, and
// the fib/sum modules are asserted byte-equal to a hand-listed spec below. The
// arithmetic/control modules reuse those anchored primitives (LEB, section, vec)
// and are checked against KNOWN values on every engine — nothing is hardcoded.

const I32: u8 = 0x7f;
const I64: u8 = 0x7e;

/// Unsigned LEB128 of `v`, appended to `out`.
fn leb_u32(mut v: u32, out: &mut Vec<u8>) {
    loop {
        let byte = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

/// Signed LEB128 of an `i64` (i64.const operands), appended to `out`.
fn leb_i64(mut v: i64, out: &mut Vec<u8>) {
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

fn wasm_section(m: &mut Vec<u8>, id: u8, payload: &[u8]) {
    m.push(id);
    leb_u32(payload.len() as u32, m);
    m.extend_from_slice(payload);
}

fn wasm_vec(items: &[Vec<u8>]) -> Vec<u8> {
    let mut v = Vec::new();
    leb_u32(items.len() as u32, &mut v);
    for it in items {
        v.extend_from_slice(it);
    }
    v
}

struct Func {
    locals: Vec<(u32, u8)>,
    body: Vec<u8>,
}

/// Assemble a real module from functypes, function type-indices, exports, and
/// bodies — Type(1), Function(3), Export(7), Code(10) in ascending id order.
fn wasm_program(
    types: &[(Vec<u8>, Vec<u8>)],
    funcs: &[u32],
    exports: &[(&str, u8, u32)],
    codes: &[Func],
) -> Vec<u8> {
    let mut m: Vec<u8> = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

    let tvec = wasm_vec(
        &types
            .iter()
            .map(|(p, r)| {
                let mut t = vec![0x60];
                t.extend(wasm_vec(&p.iter().map(|v| vec![*v]).collect::<Vec<_>>()));
                t.extend(wasm_vec(&r.iter().map(|v| vec![*v]).collect::<Vec<_>>()));
                t
            })
            .collect::<Vec<_>>(),
    );
    wasm_section(&mut m, 0x01, &tvec);

    let fvec = wasm_vec(
        &funcs
            .iter()
            .map(|t| {
                let mut b = Vec::new();
                leb_u32(*t, &mut b);
                b
            })
            .collect::<Vec<_>>(),
    );
    wasm_section(&mut m, 0x03, &fvec);

    if !exports.is_empty() {
        let evec = wasm_vec(
            &exports
                .iter()
                .map(|(name, kind, idx)| {
                    let mut e = wasm_vec(&name.bytes().map(|c| vec![c]).collect::<Vec<_>>());
                    e.push(*kind);
                    leb_u32(*idx, &mut e);
                    e
                })
                .collect::<Vec<_>>(),
        );
        wasm_section(&mut m, 0x07, &evec);
    }

    let cvec = wasm_vec(
        &codes
            .iter()
            .map(|f| {
                let mut body = wasm_vec(
                    &f.locals
                        .iter()
                        .map(|(c, v)| {
                            let mut b = Vec::new();
                            leb_u32(*c, &mut b);
                            b.push(*v);
                            b
                        })
                        .collect::<Vec<_>>(),
                );
                body.extend_from_slice(&f.body);
                let mut c = Vec::new();
                leb_u32(body.len() as u32, &mut c);
                c.extend_from_slice(&body);
                c
            })
            .collect::<Vec<_>>(),
    );
    wasm_section(&mut m, 0x0a, &cvec);
    m
}

fn c32(n: i32) -> Vec<u8> {
    let mut b = vec![0x41];
    leb_i32(n, &mut b);
    b
}

fn c64(n: i64) -> Vec<u8> {
    let mut b = vec![0x42];
    leb_i64(n, &mut b);
    b
}

/// A module of one exported `main : () -> result` with the given body/locals.
fn one_func(result: u8, body: Vec<u8>, locals: Vec<(u32, u8)>) -> Vec<u8> {
    wasm_program(&[(vec![], vec![result])], &[0], &[("main", 0, 0)], &[Func { locals, body }])
}

fn arith32(a: i32, b: i32, op: u8) -> Vec<u8> {
    let mut body = c32(a);
    body.extend(c32(b));
    body.push(op);
    body.push(0x0b);
    one_func(I32, body, vec![])
}

fn arith64(a: i64, b: i64, op: u8) -> Vec<u8> {
    let mut body = c64(a);
    body.extend(c64(b));
    body.push(op);
    body.push(0x0b);
    one_func(I64, body, vec![])
}

fn un32(a: i32, op: u8) -> Vec<u8> {
    let mut body = c32(a);
    body.push(op);
    body.push(0x0b);
    one_func(I32, body, vec![])
}

/// `fib(n)` via `if`(result i32) + `call` + `i32.add`/`i32.sub` + `i32.lt_s`.
/// func 0 = fib(param i32)->i32; func 1 = main()->i32 pushing `n` then calling.
fn fib_module(n: i32) -> Vec<u8> {
    let fib_body = vec![
        0x20, 0x00, 0x41, 0x02, 0x48, // local.get 0; i32.const 2; i32.lt_s
        0x04, 0x7f, // if (result i32)
        0x20, 0x00, // then: local.get 0
        0x05, // else
        0x20, 0x00, 0x41, 0x01, 0x6b, 0x10, 0x00, // fib(n-1)
        0x20, 0x00, 0x41, 0x02, 0x6b, 0x10, 0x00, // fib(n-2)
        0x6a, // i32.add
        0x0b, // end if
        0x0b, // end func
    ];
    let mut main_body = c32(n);
    main_body.extend([0x10, 0x00, 0x0b]); // call 0; end
    wasm_program(
        &[(vec![I32], vec![I32]), (vec![], vec![I32])],
        &[0, 1],
        &[("main", 0, 1)],
        &[Func { locals: vec![], body: fib_body }, Func { locals: vec![], body: main_body }],
    )
}

/// `sum(n) = 1+2+...+n` via `loop`+`br_if`+`block`+`local`s (i, acc).
fn sum_module(n: i32) -> Vec<u8> {
    let sum_body = vec![
        0x41, 0x01, 0x21, 0x01, // i = 1
        0x02, 0x40, // block
        0x03, 0x40, // loop
        0x20, 0x01, 0x20, 0x00, 0x4a, // local.get i; local.get n; i32.gt_s
        0x0d, 0x01, // br_if 1 (exit block when i>n)
        0x20, 0x02, 0x20, 0x01, 0x6a, 0x21, 0x02, // acc += i
        0x20, 0x01, 0x41, 0x01, 0x6a, 0x21, 0x01, // i += 1
        0x0c, 0x00, // br 0 (loop)
        0x0b, // end loop
        0x0b, // end block
        0x20, 0x02, // local.get acc
        0x0b, // end func
    ];
    let mut main_body = c32(n);
    main_body.extend([0x10, 0x00, 0x0b]);
    wasm_program(
        &[(vec![I32], vec![I32]), (vec![], vec![I32])],
        &[0, 1],
        &[("main", 0, 1)],
        &[Func { locals: vec![(2, I32)], body: sum_body }, Func { locals: vec![], body: main_body }],
    )
}

/// `br_table`: selector 0 -> block $inner exit (returns 100), else -> default
/// (returns 200). Exercises target selection and default.
fn brtable_module(sel: i32) -> Vec<u8> {
    let mut body = vec![
        0x02, 0x40, // block $outer
        0x02, 0x40, // block $inner
    ];
    body.extend(c32(sel));
    body.extend([0x0e, 0x01, 0x00, 0x01]); // br_table 0 (default 1)
    body.push(0x0b); // end inner
    body.extend(c32(100));
    body.push(0x0f); // return
    body.push(0x0b); // end outer
    body.extend(c32(200));
    body.push(0x0f); // return
    body.push(0x0b); // end func
    one_func(I32, body, vec![])
}

fn divzero_module() -> Vec<u8> {
    let mut body = c32(10);
    body.extend(c32(0));
    body.push(0x6d); // i32.div_s
    body.push(0x0b);
    one_func(I32, body, vec![])
}

// ---- M1 gate: recursion, loops, full numeric, control flow, traps ----------

/// Hand-listed spec bytes anchoring the encoder (two independent byte listings
/// must agree, exactly as M0 anchors `wasm_add_module`).
const FIB10_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, // \0asm, version 1
    0x01, 0x0a, 0x02, 0x60, 0x01, 0x7f, 0x01, 0x7f, 0x60, 0x00, 0x01, 0x7f, // Type: 2 functypes
    0x03, 0x03, 0x02, 0x00, 0x01, // Function: func0:type0, func1:type1
    0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x01, // Export "main" -> func1
    0x0a, 0x25, 0x02, 0x1c, 0x00, 0x20, 0x00, 0x41, 0x02, 0x48, 0x04, 0x7f, 0x20, 0x00, 0x05, 0x20,
    0x00, 0x41, 0x01, 0x6b, 0x10, 0x00, 0x20, 0x00, 0x41, 0x02, 0x6b, 0x10, 0x00, 0x6a, 0x0b, 0x0b,
    0x06, 0x00, 0x41, 0x0a, 0x10, 0x00, 0x0b, // Code: fib + main(10)
];

const SUM10_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x0a, 0x02, 0x60, 0x01, 0x7f, 0x01, 0x7f,
    0x60, 0x00, 0x01, 0x7f, 0x03, 0x03, 0x02, 0x00, 0x01, 0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69,
    0x6e, 0x00, 0x01, 0x0a, 0x30, 0x02, 0x27, 0x01, 0x02, 0x7f, 0x41, 0x01, 0x21, 0x01, 0x02, 0x40,
    0x03, 0x40, 0x20, 0x01, 0x20, 0x00, 0x4a, 0x0d, 0x01, 0x20, 0x02, 0x20, 0x01, 0x6a, 0x21, 0x02,
    0x20, 0x01, 0x41, 0x01, 0x6a, 0x21, 0x01, 0x0c, 0x00, 0x0b, 0x0b, 0x20, 0x02, 0x0b, 0x06, 0x00,
    0x41, 0x0a, 0x10, 0x00, 0x0b,
];

#[test]
fn m1_encoder_matches_spec_bytes() {
    assert_eq!(fib_module(10), FIB10_MODULE);
    assert_eq!(sum_module(10), SUM10_MODULE);
}

#[test]
fn recursive_fib() {
    assert_eq!(run_ret_all(&program(&fib_module(0))), 0);
    assert_eq!(run_ret_all(&program(&fib_module(1))), 1);
    assert_eq!(run_ret_all(&program(&fib_module(7))), 13);
    assert_eq!(run_ret_all(&program(&fib_module(10))), 55);
    assert_eq!(run_ret_all(&program(&fib_module(15))), 610);
}

#[test]
fn loop_sum() {
    assert_eq!(run_ret_all(&program(&sum_module(1))), 1);
    assert_eq!(run_ret_all(&program(&sum_module(10))), 55);
    assert_eq!(run_ret_all(&program(&sum_module(100))), 5050);
}

#[test]
fn i32_div_rem_signed_unsigned() {
    // div_s truncates toward zero; div_u treats operands as unsigned 32-bit.
    assert_eq!(run_ret_all(&program(&arith32(-7, 2, 0x6d))), -3); // div_s
    assert_eq!(run_ret_all(&program(&arith32(-1, 2, 0x6e))), 2147483647); // div_u 0xffffffff/2
    assert_eq!(run_ret_all(&program(&arith32(-7, 2, 0x6f))), -1); // rem_s
    assert_eq!(run_ret_all(&program(&arith32(-1, 3, 0x70))), 0); // rem_u 0xffffffff%3
    assert_eq!(run_ret_all(&program(&arith32(i32::MIN, -1, 0x6f))), 0); // rem_s INT_MIN/-1 = 0
}

#[test]
fn i32_shift_rotate() {
    assert_eq!(run_ret_all(&program(&arith32(-8, 1, 0x75))), -4); // shr_s
    assert_eq!(run_ret_all(&program(&arith32(-8, 1, 0x76))), 2147483644); // shr_u
    assert_eq!(run_ret_all(&program(&arith32(1, 33, 0x74))), 2); // shl, count mod 32 = 1
    assert_eq!(run_ret_all(&program(&arith32(0x40000000, 2, 0x77))), 1); // rotl
    assert_eq!(run_ret_all(&program(&arith32(1, 1, 0x78))), i32::MIN as i64); // rotr 1 by 1
}

#[test]
fn i32_bitcount_and_unsigned_compare() {
    assert_eq!(run_ret_all(&program(&un32(1, 0x67))), 31); // clz
    assert_eq!(run_ret_all(&program(&un32(0, 0x68))), 32); // ctz(0)
    assert_eq!(run_ret_all(&program(&un32(8, 0x68))), 3); // ctz(8)
    assert_eq!(run_ret_all(&program(&un32(0xff, 0x69))), 8); // popcnt
    assert_eq!(run_ret_all(&program(&un32(-1, 0x69))), 32); // popcnt(0xffffffff)
    assert_eq!(run_ret_all(&program(&arith32(-5, 3, 0x49))), 0); // lt_u: 0xfffffffb < 3? no
    assert_eq!(run_ret_all(&program(&arith32(-5, 3, 0x4b))), 1); // gt_u: yes
}

#[test]
fn i64_numeric() {
    assert_eq!(run_ret_all(&program(&arith64(0x100000000, 3, 0x7e))), 12884901888); // mul 2^32*3
    assert_eq!(run_ret_all(&program(&arith64(-1, 60, 0x88))), 15); // shr_u 0xff..>>60
    assert_eq!(run_ret_all(&program(&arith64(-9, 4, 0x7f))), -2); // div_s
    assert_eq!(run_ret_all(&program(&arith64(-1, 10, 0x80))), 1844674407370955161); // div_u
    assert_eq!(run_ret_all(&program(&arith64(1, 63, 0x86))), i64::MIN); // shl 1<<63
    assert_eq!(run_ret_all(&program(&arith64(-1, 0, 0x54))), 0); // lt_u 0xff..< 0? no
    assert_eq!(run_ret_all(&program(&arith64(1, 4, 0x89))), 16); // rotl 1 by 4
}

#[test]
fn br_table_selects_target() {
    assert_eq!(run_ret_all(&program(&brtable_module(0))), 100);
    assert_eq!(run_ret_all(&program(&brtable_module(1))), 200);
    assert_eq!(run_ret_all(&program(&brtable_module(5))), 200); // default
}

#[test]
fn divide_by_zero_traps() {
    assert_eq!(run_fault_all(&program(&divzero_module())), FaultKind::Panic);
    // div_u by zero and rem by zero also trap.
    assert_eq!(run_fault_all(&program(&arith32(1, 0, 0x6e))), FaultKind::Panic);
    assert_eq!(run_fault_all(&program(&arith32(1, 0, 0x70))), FaultKind::Panic);
    // signed div overflow INT_MIN / -1 traps.
    assert_eq!(run_fault_all(&program(&arith32(i32::MIN, -1, 0x6d))), FaultKind::Panic);
}
