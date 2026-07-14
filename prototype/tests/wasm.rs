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
        "{}\nfn main() alloc -> i64 {{\n    let data: [{}]u8 = [{}];\n    let mut st: FreeList = with_window(33554432usize, 8388608usize);\n    let a: Alloc = mk_alloc(write st);\n    let mut hout: Vec[u8] = vec_new(read a);\n    return run_module(slice_of(data), write hout);\n}}\n",
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

// ---- M2 encoder: linear memory (memory/data sections, load/store) ----------
//
// Same discipline as M0/M1: an INDEPENDENT Rust encoder builds the real `.wasm`
// bytes, two modules (memory.size + a data segment) are asserted byte-equal to a
// hand-listed spec below, and every result is checked against a KNOWN value on
// all four engines. The decode+eval genuinely reads/writes the `Vec[u8]` memory
// little-endian and bounds-traps — nothing is hardcoded.

/// A memarg (align, offset). `align` is the natural log2(size); semantics ignore
/// it, but the encoder emits the canonical value so the byte anchor is meaningful.
fn mop(op: u8, align: u32, offset: u32) -> Vec<u8> {
    let mut b = vec![op];
    leb_u32(align, &mut b);
    leb_u32(offset, &mut b);
    b
}

/// One exported `main : () -> result` over a memory of `min_pages` (opt `max`),
/// with optional active data segments (each `(offset, bytes)`), given body/locals.
/// Sections: Type(1) Function(3) Memory(5) Export(7) Code(10) Data(11).
fn wasm_mem_module(
    result: u8,
    min_pages: u32,
    max_pages: Option<u32>,
    data: &[(u32, Vec<u8>)],
    locals: Vec<(u32, u8)>,
    body: Vec<u8>,
) -> Vec<u8> {
    let mut m: Vec<u8> = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    // Type: 1 functype () -> (result).
    wasm_section(&mut m, 0x01, &[0x01, 0x60, 0x00, 0x01, result]);
    // Function: 1 func, type index 0.
    wasm_section(&mut m, 0x03, &[0x01, 0x00]);
    // Memory: 1 memory (flags + min pages [+ max pages]).
    let mut mem = vec![0x01u8];
    match max_pages {
        Some(mx) => {
            mem.push(0x01);
            leb_u32(min_pages, &mut mem);
            leb_u32(mx, &mut mem);
        }
        None => {
            mem.push(0x00);
            leb_u32(min_pages, &mut mem);
        }
    }
    wasm_section(&mut m, 0x05, &mem);
    // Export "main" -> func 0.
    wasm_section(&mut m, 0x07, &[0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00]);
    // Code: one body (local decls + instructions).
    let mut bd: Vec<u8> = Vec::new();
    leb_u32(locals.len() as u32, &mut bd);
    for (c, v) in &locals {
        leb_u32(*c, &mut bd);
        bd.push(*v);
    }
    bd.extend_from_slice(&body);
    let mut code: Vec<u8> = Vec::new();
    leb_u32(1, &mut code);
    leb_u32(bd.len() as u32, &mut code);
    code.extend_from_slice(&bd);
    wasm_section(&mut m, 0x0a, &code);
    // Data: active segments (flag 0, memidx 0, `i32.const off; end`, byte vector).
    if !data.is_empty() {
        let mut dv: Vec<u8> = Vec::new();
        leb_u32(data.len() as u32, &mut dv);
        for (off, bytes) in data {
            dv.push(0x00);
            dv.push(0x41);
            leb_i32(*off as i32, &mut dv);
            dv.push(0x0b);
            leb_u32(bytes.len() as u32, &mut dv);
            dv.extend_from_slice(bytes);
        }
        wasm_section(&mut m, 0x0b, &dv);
    }
    m
}

/// Store `val` (i32) at address 0, then load it back with `load_op` at `load_off`.
fn store_then_load(store_op: u8, store_al: u32, load_op: u8, load_al: u32, load_off: u32, val: i32, result: u8) -> Vec<u8> {
    let mut body = c32(0);
    body.extend(c32(val));
    body.extend(mop(store_op, store_al, 0));
    body.extend(c32(0));
    body.extend(mop(load_op, load_al, load_off));
    body.push(0x0b);
    wasm_mem_module(result, 1, None, &[], vec![], body)
}

// ---- M2 spec anchors: two hand-listed modules (encoder must reproduce) ------

/// `memory.size` over a 2-page memory -> 2. Anchors the Memory(5) section decode.
const MEMSIZE2_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, // \0asm, version 1
    0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, // Type: () -> (i32)
    0x03, 0x02, 0x01, 0x00, // Function: func 0 : type 0
    0x05, 0x03, 0x01, 0x00, 0x02, // Memory: 1 mem, flags 0, min 2 pages
    0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00, // Export "main" -> func 0
    0x0a, 0x06, 0x01, 0x04, 0x00, 0x3f, 0x00, 0x0b, // Code: memory.size; end
];

/// A data segment `01 02 03 04` at offset 0, then `i32.load` at 0 -> 0x04030201
/// (little-endian). Anchors both the Memory(5) and Data(11) section decode.
const DATA_MODULE: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
    0x03, 0x02, 0x01, 0x00,
    0x05, 0x03, 0x01, 0x00, 0x01, // Memory: 1 page
    0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
    0x0a, 0x09, 0x01, 0x07, 0x00, 0x41, 0x00, 0x28, 0x02, 0x00, 0x0b, // i32.const 0; i32.load; end
    0x0b, 0x0a, 0x01, 0x00, 0x41, 0x00, 0x0b, 0x04, 0x01, 0x02, 0x03, 0x04, // Data: [1,2,3,4] @ 0
];

#[test]
fn m2_encoder_matches_spec_bytes() {
    let msize = {
        let mut body = vec![0x3f, 0x00];
        body.push(0x0b);
        wasm_mem_module(I32, 2, None, &[], vec![], body)
    };
    assert_eq!(msize, MEMSIZE2_MODULE);

    let dmod = {
        let mut body = c32(0);
        body.extend(mop(0x28, 2, 0));
        body.push(0x0b);
        wasm_mem_module(I32, 1, None, &[(0, vec![0x01, 0x02, 0x03, 0x04])], vec![], body)
    };
    assert_eq!(dmod, DATA_MODULE);
}

// ---- M2 gate: round-trip, width+sign/zero+endianness, data, OOB, grow -------

#[test]
fn store_load_roundtrip() {
    // i32 round-trip: store a value, load it back.
    let m = {
        let mut body = c32(16);
        body.extend(c32(0x12345678));
        body.extend(mop(0x36, 2, 0)); // i32.store
        body.extend(c32(16));
        body.extend(mop(0x28, 2, 0)); // i32.load
        body.push(0x0b);
        wasm_mem_module(I32, 1, None, &[], vec![], body)
    };
    assert_eq!(run_ret_all(&program(&m)), 0x12345678);

    // i64 round-trip through a full 8-byte access.
    let m64 = {
        let mut body = c32(32);
        body.extend(c64(0x0102030405060708));
        body.extend(mop(0x37, 3, 0)); // i64.store
        body.extend(c32(32));
        body.extend(mop(0x29, 3, 0)); // i64.load
        body.push(0x0b);
        wasm_mem_module(I64, 1, None, &[], vec![], body)
    };
    assert_eq!(run_ret_all(&program(&m64)), 0x0102030405060708);
}

#[test]
fn load_width_sign_zero_extension() {
    // Store byte 0xFF, then i32.load8_s -> -1, i32.load8_u -> 255.
    assert_eq!(run_ret_all(&program(&store_then_load(0x3a, 0, 0x2c, 0, 0, 0xff, I32))), -1);
    assert_eq!(run_ret_all(&program(&store_then_load(0x3a, 0, 0x2d, 0, 0, 0xff, I32))), 255);
    // Store 0xFFFF, then i32.load16_s -> -1, i32.load16_u -> 65535.
    assert_eq!(run_ret_all(&program(&store_then_load(0x3b, 1, 0x2e, 1, 0, 0xffff, I32))), -1);
    assert_eq!(run_ret_all(&program(&store_then_load(0x3b, 1, 0x2f, 1, 0, 0xffff, I32))), 65535);
    // i64 width forms: store byte 0xFF, i64.load8_s -> -1 (full 64-bit sign fill),
    // i64.load8_u -> 255; i64.load32_s of 0xFFFFFFFF -> -1.
    assert_eq!(run_ret_all(&program(&store_then_load(0x3c, 0, 0x30, 0, 0, 0xff, I64))), -1);
    assert_eq!(run_ret_all(&program(&store_then_load(0x3c, 0, 0x31, 0, 0, 0xff, I64))), 255);
    let m32s = {
        let mut body = c32(0);
        body.extend(c64(0xffffffff));
        body.extend(mop(0x3e, 2, 0)); // i64.store32 (low 4 bytes = 0xffffffff)
        body.extend(c32(0));
        body.extend(mop(0x34, 2, 0)); // i64.load32_s
        body.push(0x0b);
        wasm_mem_module(I64, 1, None, &[], vec![], body)
    };
    assert_eq!(run_ret_all(&program(&m32s)), -1);
}

#[test]
fn little_endian_byte_order() {
    // Store 0x04030201 as i32 at 0; the LOW byte sits at offset 0.
    assert_eq!(run_ret_all(&program(&store_then_load(0x36, 2, 0x2d, 0, 0, 0x04030201, I32))), 0x01);
    // ... the byte at offset 3 is the HIGH byte 0x04.
    assert_eq!(run_ret_all(&program(&store_then_load(0x36, 2, 0x2d, 0, 3, 0x04030201, I32))), 0x04);
}

#[test]
fn data_segment_init() {
    // A data segment places 0xEF 0xBE 0xAD 0xDE at offset 8; i32.load at 8 reads
    // them little-endian as 0xDEADBEEF.
    let m = {
        let mut body = c32(8);
        body.extend(mop(0x28, 2, 0)); // i32.load
        body.push(0x0b);
        wasm_mem_module(I32, 1, None, &[(8, vec![0xef, 0xbe, 0xad, 0xde])], vec![], body)
    };
    assert_eq!(run_ret_all(&program(&m)), 0xdeadbeefu32 as i32 as i64);
    // A single byte from the segment via load8_u confirms placement/order.
    let mb = {
        let mut body = c32(8);
        body.extend(mop(0x2d, 0, 2)); // i32.load8_u at offset 2 -> 0xad
        body.push(0x0b);
        wasm_mem_module(I32, 1, None, &[(8, vec![0xef, 0xbe, 0xad, 0xde])], vec![], body)
    };
    assert_eq!(run_ret_all(&program(&mb)), 0xad);
}

#[test]
fn out_of_bounds_traps() {
    // Load at address 65536 (one page) faults: 65536 + 4 > 65536.
    let load_oob = {
        let mut body = c32(65536);
        body.extend(mop(0x28, 2, 0));
        body.push(0x0b);
        wasm_mem_module(I32, 1, None, &[], vec![], body)
    };
    assert_eq!(run_fault_all(&program(&load_oob)), FaultKind::Panic);
    // Store past the end faults too: store an i32 at 65534 -> 65534 + 4 > 65536.
    let store_oob = {
        let mut body = c32(65534);
        body.extend(c32(1));
        body.extend(mop(0x36, 2, 0));
        body.push(0x0b);
        wasm_mem_module(I32, 1, None, &[], vec![], body)
    };
    assert_eq!(run_fault_all(&program(&store_oob)), FaultKind::Panic);
    // An out-of-range active data segment is a module/instantiation trap.
    let data_oob = wasm_mem_module(I32, 1, None, &[(65534, vec![0x01, 0x02, 0x03, 0x04])], vec![], {
        let mut b = c32(0);
        b.push(0x0b);
        b
    });
    assert_eq!(run_fault_all(&program(&data_oob)), FaultKind::Panic);
}

#[test]
fn memory_size_and_grow() {
    // memory.size over 2 pages -> 2.
    let size2 = {
        let mut body = vec![0x3f, 0x00];
        body.push(0x0b);
        wasm_mem_module(I32, 2, None, &[], vec![], body)
    };
    assert_eq!(run_ret_all(&program(&size2)), 2);

    // memory.grow by 3 returns the OLD page count (1).
    let grow_old = {
        let mut body = c32(3);
        body.extend([0x40, 0x00]); // memory.grow
        body.push(0x0b);
        wasm_mem_module(I32, 1, None, &[], vec![], body)
    };
    assert_eq!(run_ret_all(&program(&grow_old)), 1);

    // grow by 2, then memory.size (3), added -> 1 + 3 = 4.
    let grow_then_size = {
        let mut body = c32(2);
        body.extend([0x40, 0x00]);
        body.extend([0x3f, 0x00]);
        body.push(0x6a); // i32.add
        body.push(0x0b);
        wasm_mem_module(I32, 1, None, &[], vec![], body)
    };
    assert_eq!(run_ret_all(&program(&grow_then_size)), 4);

    // A load into the newly-grown region reads zero. Grow by 1, stash the old
    // count in a local, then i32.load8_u at 70000 (in the new page) -> 0.
    let grow_zero = {
        let mut body = c32(1);
        body.extend([0x40, 0x00]);
        body.extend([0x21, 0x00]); // local.set 0 (discard old count)
        body.extend(c32(70000));
        body.extend(mop(0x2d, 0, 0)); // i32.load8_u
        body.push(0x0b);
        wasm_mem_module(I32, 1, None, &[], vec![(1, I32)], body)
    };
    assert_eq!(run_ret_all(&program(&grow_zero)), 0);

    // grow beyond a declared max fails, pushing -1.
    let grow_fail = {
        let mut body = c32(5);
        body.extend([0x40, 0x00]);
        body.push(0x0b);
        wasm_mem_module(I32, 1, Some(2), &[], vec![], body)
    };
    assert_eq!(run_ret_all(&program(&grow_fail)), -1);
}

// ===========================================================================
// M3 — real `.wasm` off disk + a DIFFERENTIAL gate vs an independent reference
// ===========================================================================
//
// Two deliverables:
//   A. The interpreter runs a module read from a FILE (`read_all_bytes` loops
//      `read_into` a fixed buffer into a growable `Vec[u8]`), on the interp
//      engines (foreign_io shims here) and native/real-libc (tests/aot.rs +
//      tests/llvm.rs). Proves it runs actual `.wasm` off disk, not just embedded
//      bytes.
//   B. A differential corpus: the SAME real bytes through (a) the Candor
//      interpreter and (b) `wasmi` (a pure-Rust, spec-compliant reference), with
//      the results asserted EQUAL — values AND trap-equivalence. Upgrades the gate
//      from "matches a hand-written value" to "matches an independent spec impl".

use candor_proto::foreign_io;
use std::sync::{Mutex, MutexGuard};

// foreign_io state is process-global; serialize the file-run tests (plain
// `cargo test` shares a process; nextest isolates per test, so this is a no-op
// there but keeps `cargo test` correct).
static IO_GUARD: Mutex<()> = Mutex::new(());
fn io_lock() -> MutexGuard<'static, ()> {
    IO_GUARD.lock().unwrap_or_else(|e| e.into_inner())
}

const FILE_RUN_SRC: &str = include_str!("fixtures/wasm/run_wasm_file.cnr");
const STD_IO_SRC: &str = include_str!("fixtures/std_io/main.cnr");

fn std_io_prefix() -> &'static str {
    let idx = STD_IO_SRC.find("fn main").expect("std_io fixture has a main");
    &STD_IO_SRC[..idx]
}

// ---- A. drift guard: the file-run fixture reuses the canonical interpreter ---

#[test]
fn file_run_fixture_reuses_canonical_interp() {
    // run_wasm_file.cnr must be the std::io boundary prefix + the canonical
    // interpreter (verbatim, above the harness split) + the file-run tail. If
    // interp.cnr changes above the split, this fails until the fixture is
    // regenerated — the same drift discipline as the run/ corpus copy.
    assert!(
        FILE_RUN_SRC.starts_with(std_io_prefix()),
        "run_wasm_file.cnr must open with the std::io boundary prefix"
    );
    assert!(
        FILE_RUN_SRC.contains(interp_fns().trim_end()),
        "run_wasm_file.cnr drifted from the canonical interp.cnr (regenerate it)"
    );
    assert!(
        FILE_RUN_SRC.contains("fn read_all_bytes(a: read Alloc, fd: i32)"),
        "run_wasm_file.cnr must define read_all_bytes"
    );
}

// ---- A. the interpreter runs a real `.wasm` FILE off disk (interp engines) ---

/// Run the file-run program with `mod.wasm` = `bytes` in a temp dir, on the
/// tree-walker AND the MIR engine (via the foreign_io shims, like tests/std_io.rs).
/// Returns (tree_ret, mir_ret).
fn run_wasm_file(bytes: &[u8], tag: &str) -> (i64, i64) {
    let dir = std::env::temp_dir().join(format!("candor-wasm-file-{}-{}", tag, std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("mod.wasm"), bytes).unwrap();

    foreign_io::reset();
    foreign_io::set_root(&dir);
    foreign_io::register_std_io();
    let tw = match run_source_real(FILE_RUN_SRC) {
        RunResult::Ok(r) => r.ret,
        RunResult::Fault(f) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: tree-walker faulted: {}", f.to_json());
        }
        RunResult::CheckErrors(d) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>());
        }
        RunResult::ParseError(d) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: parse error: {}", d.to_json());
        }
    };
    foreign_io::unregister_std_io();

    foreign_io::reset();
    foreign_io::set_root(&dir);
    foreign_io::register_std_io();
    let mir = match run_source_real_mir(FILE_RUN_SRC) {
        MirRunResult::Ok(r) => r.ret,
        MirRunResult::Fault(f) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: MIR faulted: {}", f.to_json());
        }
        MirRunResult::Unsupported(m) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: MIR unsupported: {m}");
        }
        MirRunResult::CheckErrors(d) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: MIR check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>());
        }
        MirRunResult::ParseError(d) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: MIR parse error: {}", d.to_json());
        }
    };
    foreign_io::unregister_std_io();
    let _ = std::fs::remove_dir_all(&dir);
    (tw, mir)
}

#[test]
fn file_run_reads_real_wasm_off_disk() {
    let _g = io_lock();
    // A 74-byte module (> the 64-byte read buffer, so read_all_bytes genuinely
    // loops and grows the Vec): fib(10) = 55, read off disk and executed.
    let bytes = fib_module(10);
    assert_eq!(bytes, FIB10_MODULE, "the module written to disk is the anchored fib(10)");
    assert!(bytes.len() > 64, "must exceed the read buffer to exercise the loop");
    let (tw, mir) = run_wasm_file(&bytes, "fib10");
    assert_eq!(tw, 55, "tree-walker ran fib(10) off disk");
    assert_eq!(mir, 55, "MIR ran fib(10) off disk");
}

#[test]
fn file_run_matches_embedded_across_modules() {
    let _g = io_lock();
    // Reading off disk must agree with the embedded-bytes path across a spread of
    // modules (arith, i64, memory data-segment). The file read yields the exact
    // module bytes, so the two paths compute the same result.
    for (tag, bytes, expected) in [
        ("sum100", sum_module(100), 5050i64),
        ("mul", arith32(6, 7, 0x6c), 42),
        ("i64", arith64(0x100000000, 3, 0x7e), 12884901888),
        ("data", {
            let mut body = c32(8);
            body.extend(mop(0x28, 2, 0));
            body.push(0x0b);
            wasm_mem_module(I32, 1, None, &[(8, vec![0xef, 0xbe, 0xad, 0xde])], vec![], body)
        }, 0xdeadbeefu32 as i32 as i64),
    ] {
        let (tw, mir) = run_wasm_file(&bytes, tag);
        assert_eq!(tw, expected, "{tag}: tree-walker off disk");
        assert_eq!(mir, expected, "{tag}: MIR off disk");
        assert_eq!(run_ret_all(&program(&bytes)), expected, "{tag}: embedded path agrees");
    }
}

// ---- B. the DIFFERENTIAL gate against wasmi (independent spec reference) -----

/// Run `bytes` through the wasmi reference interpreter. `Ok(i64)` normalizes an
/// i32 result by sign-extension (matching the Candor interp's i32-in-i64 result
/// representation); `Err` on any validation / instantiation / trap error, so a
/// Candor trap and a wasmi error are compared as equivalent.
fn wasmi_run(bytes: &[u8]) -> Result<i64, String> {
    use wasmi::{Engine, Linker, Module, Store, Val};
    let engine = Engine::default();
    let module = Module::new(&engine, bytes).map_err(|e| format!("compile: {e}"))?;
    let mut store = Store::new(&engine, ());
    let linker = <Linker<()>>::new(&engine);
    let instance =
        linker.instantiate_and_start(&mut store, &module).map_err(|e| format!("instantiate: {e}"))?;
    let func = instance.get_func(&store, "main").ok_or("no `main` export")?;
    let nres = func.ty(&store).results().len();
    let mut out = vec![Val::I32(0); nres];
    func.call(&mut store, &[], &mut out).map_err(|e| format!("trap: {e}"))?;
    match out.first() {
        Some(Val::I32(x)) => Ok(*x as i64),
        Some(Val::I64(x)) => Ok(*x),
        other => Err(format!("unexpected result: {other:?}")),
    }
}

/// The oracle (tree-walker) result, for the differential corpus. The corpus is
/// large, so it runs on the tree-walker only (the task's "on the tree-walker at
/// least"); cross-engine (MIR + Cranelift native) agreement over the same
/// instruction classes is already gated by the hand-asserted M0-M2 tests above
/// (`run_ret_all`) and the file-run test.
fn run_ret_oracle(src: &str) -> i64 {
    match run_source_real(src) {
        RunResult::Ok(r) => r.ret,
        RunResult::Fault(f) => panic!("oracle faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => {
            panic!("check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("parse error: {}", d.to_json()),
    }
}

fn run_fault_oracle(src: &str) -> FaultKind {
    match run_source_real(src) {
        RunResult::Fault(f) => f.kind,
        RunResult::Ok(r) => panic!("expected fault, got ret {}", r.ret),
        RunResult::CheckErrors(d) => {
            panic!("expected fault, got check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("expected fault, got parse error: {}", d.to_json()),
    }
}

/// The Candor interpreter and wasmi must agree on the value.
fn assert_diff(bytes: &[u8], label: &str) {
    let candor = run_ret_oracle(&program(bytes));
    match wasmi_run(bytes) {
        Ok(w) => assert_eq!(candor, w, "{label}: candor={candor} != wasmi={w}"),
        Err(e) => panic!("{label}: candor returned {candor} but wasmi did not: {e}"),
    }
}

/// Trap-equivalence: the module traps in Candor (Panic) AND errors in wasmi.
fn assert_diff_trap(bytes: &[u8], label: &str) {
    assert_eq!(run_fault_oracle(&program(bytes)), FaultKind::Panic, "{label}: candor should trap");
    if let Ok(w) = wasmi_run(bytes) {
        panic!("{label}: candor trapped but wasmi returned {w}");
    }
}

#[test]
fn diff_i32_arithmetic() {
    for (op, name) in [
        (0x6au8, "add"), (0x6b, "sub"), (0x6c, "mul"), (0x6d, "div_s"), (0x6e, "div_u"),
        (0x6f, "rem_s"), (0x70, "rem_u"), (0x71, "and"), (0x72, "or"), (0x73, "xor"),
        (0x74, "shl"), (0x75, "shr_s"), (0x76, "shr_u"), (0x77, "rotl"), (0x78, "rotr"),
    ] {
        for (a, b) in [(-7i32, 2i32), (100, 7), (1, 33), (0x40000000, 3), (-1, 3), (i32::MIN, 5)] {
            assert_diff(&arith32(a, b, op), &format!("i32.{name}({a},{b})"));
        }
    }
}

#[test]
fn diff_i32_unary_and_compares() {
    for (op, name) in [(0x67u8, "clz"), (0x68, "ctz"), (0x69, "popcnt"), (0x45, "eqz")] {
        for a in [0i32, 1, 8, 0xff, -1, i32::MIN, 0x00ff00ff] {
            assert_diff(&un32(a, op), &format!("i32.{name}({a})"));
        }
    }
    for (op, name) in [
        (0x46u8, "eq"), (0x47, "ne"), (0x48, "lt_s"), (0x49, "lt_u"), (0x4a, "gt_s"),
        (0x4b, "gt_u"), (0x4c, "le_s"), (0x4d, "le_u"), (0x4e, "ge_s"), (0x4f, "ge_u"),
    ] {
        for (a, b) in [(-5i32, 3i32), (3, 3), (7, 2), (-1, -2), (i32::MIN, i32::MAX)] {
            assert_diff(&arith32(a, b, op), &format!("i32.{name}({a},{b})"));
        }
    }
}

#[test]
fn diff_i64_arithmetic_and_compares() {
    for (op, name) in [
        (0x7cu8, "add"), (0x7d, "sub"), (0x7e, "mul"), (0x7f, "div_s"), (0x80, "div_u"),
        (0x81, "rem_s"), (0x82, "rem_u"), (0x83, "and"), (0x84, "or"), (0x85, "xor"),
        (0x86, "shl"), (0x87, "shr_s"), (0x88, "shr_u"), (0x89, "rotl"), (0x8a, "rotr"),
    ] {
        for (a, b) in [
            (0x100000000i64, 3i64), (-9, 4), (-1, 10), (1, 63), (i64::MIN, 7), (-1, 60),
        ] {
            assert_diff(&arith64(a, b, op), &format!("i64.{name}({a},{b})"));
        }
    }
    // i64 comparisons consume two i64 operands but PRODUCE an i32 — the module's
    // result type must be I32 (a mismatch that the non-validating Candor interp
    // runs but wasmi correctly rejects, so the corpus must be well-typed).
    for (op, name) in [
        (0x51u8, "eq"), (0x52, "ne"), (0x53, "lt_s"), (0x54, "lt_u"), (0x55, "gt_s"),
        (0x56, "gt_u"), (0x57, "le_s"), (0x58, "le_u"), (0x59, "ge_s"), (0x5a, "ge_u"),
    ] {
        for (a, b) in [(-1i64, 0i64), (5, 5), (7, 2), (i64::MIN, i64::MAX)] {
            let mut body = c64(a);
            body.extend(c64(b));
            body.push(op);
            body.push(0x0b);
            assert_diff(&one_func(I32, body, vec![]), &format!("i64.{name}({a},{b})"));
        }
    }
    // i64 unary bit-counts and eqz.
    for (op, name) in [(0x79u8, "clz"), (0x7a, "ctz"), (0x7b, "popcnt")] {
        for a in [0i64, 1, 0xff, -1, i64::MIN] {
            let mut body = c64(a);
            body.push(op);
            body.push(0x0b);
            assert_diff(&one_func(I64, body, vec![]), &format!("i64.{name}({a})"));
        }
    }
}

#[test]
fn diff_control_flow_and_recursion() {
    for n in [0i32, 1, 5, 7, 10, 15] {
        assert_diff(&fib_module(n), &format!("fib({n})"));
    }
    for n in [1i32, 5, 10, 100] {
        assert_diff(&sum_module(n), &format!("sum({n})"));
    }
    for sel in [0i32, 1, 2, 5, -1] {
        assert_diff(&brtable_module(sel), &format!("br_table({sel})"));
    }
}

#[test]
fn diff_linear_memory() {
    // store/load round-trip at various widths (little-endian), sign/zero extension.
    assert_diff(&store_then_load(0x36, 2, 0x28, 2, 0, 0x12345678, I32), "i32.store/load");
    assert_diff(&store_then_load(0x3a, 0, 0x2c, 0, 0, 0xff, I32), "i32.load8_s");
    assert_diff(&store_then_load(0x3a, 0, 0x2d, 0, 0, 0xff, I32), "i32.load8_u");
    assert_diff(&store_then_load(0x3b, 1, 0x2e, 1, 0, 0xffff, I32), "i32.load16_s");
    assert_diff(&store_then_load(0x3b, 1, 0x2f, 1, 0, 0xffff, I32), "i32.load16_u");
    // i64.load8_s: store an i64 value with i64.store8 (the value operand must be
    // i64, so store_then_load's i32.const value would be ill-typed for wasmi).
    let i64_load8s = {
        let mut body = c32(0);
        body.extend(c64(0xff));
        body.extend(mop(0x3c, 0, 0));
        body.extend(c32(0));
        body.extend(mop(0x30, 0, 0));
        body.push(0x0b);
        wasm_mem_module(I64, 1, None, &[], vec![], body)
    };
    assert_diff(&i64_load8s, "i64.load8_s");
    assert_diff(&store_then_load(0x36, 2, 0x2d, 0, 3, 0x04030201, I32), "little-endian byte 3");

    // i64 full-width round-trip.
    let m64 = {
        let mut body = c32(32);
        body.extend(c64(0x0102030405060708));
        body.extend(mop(0x37, 3, 0));
        body.extend(c32(32));
        body.extend(mop(0x29, 3, 0));
        body.push(0x0b);
        wasm_mem_module(I64, 1, None, &[], vec![], body)
    };
    assert_diff(&m64, "i64.store/load");

    // data segment init read little-endian.
    let data = {
        let mut body = c32(8);
        body.extend(mop(0x28, 2, 0));
        body.push(0x0b);
        wasm_mem_module(I32, 1, None, &[(8, vec![0xef, 0xbe, 0xad, 0xde])], vec![], body)
    };
    assert_diff(&data, "data-segment i32.load");

    // memory.grow returns the old page count; grow-then-size adds.
    let grow_then_size = {
        let mut body = c32(2);
        body.extend([0x40, 0x00]);
        body.extend([0x3f, 0x00]);
        body.push(0x6a);
        body.push(0x0b);
        wasm_mem_module(I32, 1, None, &[], vec![], body)
    };
    assert_diff(&grow_then_size, "memory.grow+size");
    // grow beyond a declared max returns -1.
    let grow_fail = {
        let mut body = c32(5);
        body.extend([0x40, 0x00]);
        body.push(0x0b);
        wasm_mem_module(I32, 1, Some(2), &[], vec![], body)
    };
    assert_diff(&grow_fail, "memory.grow-fail");
}

#[test]
fn diff_traps_are_equivalent() {
    // divide-by-zero (i32 + i64, div + rem, signed + unsigned).
    assert_diff_trap(&arith32(10, 0, 0x6d), "i32.div_s/0");
    assert_diff_trap(&arith32(10, 0, 0x6e), "i32.div_u/0");
    assert_diff_trap(&arith32(10, 0, 0x6f), "i32.rem_s/0");
    assert_diff_trap(&arith32(10, 0, 0x70), "i32.rem_u/0");
    assert_diff_trap(&arith64(10, 0, 0x7f), "i64.div_s/0");
    assert_diff_trap(&arith64(10, 0, 0x80), "i64.div_u/0");
    // signed division overflow INT_MIN / -1.
    assert_diff_trap(&arith32(i32::MIN, -1, 0x6d), "i32.div_s overflow");
    assert_diff_trap(&arith64(i64::MIN, -1, 0x7f), "i64.div_s overflow");
    // out-of-bounds load and store.
    let load_oob = {
        let mut body = c32(65536);
        body.extend(mop(0x28, 2, 0));
        body.push(0x0b);
        wasm_mem_module(I32, 1, None, &[], vec![], body)
    };
    assert_diff_trap(&load_oob, "load OOB");
    let store_oob = {
        let mut body = c32(65534);
        body.extend(c32(1));
        body.extend(mop(0x36, 2, 0));
        body.push(0x0b);
        wasm_mem_module(I32, 1, None, &[], vec![], body)
    };
    assert_diff_trap(&store_oob, "store OOB");
    // active data segment out of bounds (an instantiation trap).
    let data_oob = wasm_mem_module(I32, 1, None, &[(65534, vec![1, 2, 3, 4])], vec![], {
        let mut b = c32(0);
        b.push(0x0b);
        b
    });
    assert_diff_trap(&data_oob, "data segment OOB");
    // unreachable.
    let unreachable = one_func(I32, vec![0x00, 0x0b], vec![]);
    assert_diff_trap(&unreachable, "unreachable");
}

// ===========================================================================
// M4 — HOST IMPORTS + a WASI-lite `print` (real output through the I/O boundary)
// ===========================================================================
//
// Imported functions occupy the LOWEST function indices, so a module importing K
// functions addresses its defined bodies at [K, K+n). The encoder below builds
// real `.wasm` with an Import section (id 2) and calls that mix host imports
// (`call N`, N < K) with defined functions (`call N`, N >= K). The interpreter
// resolves an import's (module, field) name to a host handler: `env.print_i32`
// writes the argument's decimal + newline; `env.print_str(ptr,len)` copies `len`
// bytes of LINEAR MEMORY out through std::io `write_all`. The gate asserts the
// CAPTURED STDOUT equals the expected bytes on the interp engines (foreign_io
// shims) and, via wasmi host imports over a captured buffer, that Candor's
// captured output == wasmi's captured output for the print modules.

/// A functype `(params) -> (results)`.
fn functype(params: &[u8], results: &[u8]) -> Vec<u8> {
    let mut t = vec![0x60];
    t.extend(wasm_vec(&params.iter().map(|v| vec![*v]).collect::<Vec<_>>()));
    t.extend(wasm_vec(&results.iter().map(|v| vec![*v]).collect::<Vec<_>>()));
    t
}

/// One import entry: module name, field name, then the descriptor bytes.
fn import_entry(module: &str, field: &str, desc: &[u8]) -> Vec<u8> {
    let mut e = wasm_vec(&module.bytes().map(|c| vec![c]).collect::<Vec<_>>());
    e.extend(wasm_vec(&field.bytes().map(|c| vec![c]).collect::<Vec<_>>()));
    e.extend_from_slice(desc);
    e
}

/// One code body: local declarations + instruction bytes, size-prefixed.
fn code_body(locals: &[(u32, u8)], body: &[u8]) -> Vec<u8> {
    let mut bd = wasm_vec(
        &locals
            .iter()
            .map(|(c, v)| {
                let mut b = Vec::new();
                leb_u32(*c, &mut b);
                b.push(*v);
                b
            })
            .collect::<Vec<_>>(),
    );
    bd.extend_from_slice(body);
    let mut c = Vec::new();
    leb_u32(bd.len() as u32, &mut c);
    c.extend_from_slice(&bd);
    c
}

/// An active data segment `(offset, bytes)` in Data(11) form.
fn data_seg(off: u32, bytes: &[u8]) -> Vec<u8> {
    let mut d = vec![0x00u8, 0x41u8];
    leb_i32(off as i32, &mut d);
    d.push(0x0b);
    leb_u32(bytes.len() as u32, &mut d);
    d.extend_from_slice(bytes);
    d
}

const MAGIC: [u8; 8] = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

/// The main M4 gate module: imports BOTH `env.print_str` (funcidx 0) and
/// `env.print_i32` (funcidx 1), defines a `helper` (funcidx 2) and `main`
/// (funcidx 3). `main` calls the print_str IMPORT (`call 0`), then the DEFINED
/// helper (`call 2`), which in turn calls the print_i32 IMPORT (`call 1`). This
/// exercises every index-space case in one module. Expected stdout:
/// `"hello, wasm\n42\n"`.
fn print_mixed_module() -> Vec<u8> {
    let mut m = MAGIC.to_vec();
    let types = wasm_vec(&[
        functype(&[I32, I32], &[]), // t0: print_str
        functype(&[I32], &[]),      // t1: print_i32
        functype(&[], &[]),         // t2: helper / main
    ]);
    wasm_section(&mut m, 0x01, &types);
    let imports = wasm_vec(&[
        import_entry("env", "print_str", &[0x00, 0x00]),
        import_entry("env", "print_i32", &[0x00, 0x01]),
    ]);
    wasm_section(&mut m, 0x02, &imports);
    wasm_section(&mut m, 0x03, &wasm_vec(&[vec![0x02], vec![0x02]])); // helper, main : t2
    wasm_section(&mut m, 0x05, &[0x01, 0x00, 0x01]); // 1 memory, min 1 page
    // Export "main" -> func 3, "memory" -> mem 0 (wasmi's host reads the export).
    let exports = wasm_vec(&[
        {
            let mut e = wasm_vec(&"main".bytes().map(|c| vec![c]).collect::<Vec<_>>());
            e.push(0x00);
            leb_u32(3, &mut e);
            e
        },
        {
            let mut e = wasm_vec(&"memory".bytes().map(|c| vec![c]).collect::<Vec<_>>());
            e.push(0x02);
            leb_u32(0, &mut e);
            e
        },
    ]);
    wasm_section(&mut m, 0x07, &exports);
    let helper = {
        let mut b = c32(42);
        b.extend([0x10, 0x01, 0x0b]); // call 1 (print_i32 import); end
        b
    };
    let main = {
        let mut b = c32(0); // ptr
        b.extend(c32(12)); // len of "hello, wasm\n"
        b.extend([0x10, 0x00]); // call 0 (print_str import)
        b.extend([0x10, 0x02]); // call 2 (defined helper)
        b.push(0x0b); // end
        b
    };
    wasm_section(&mut m, 0x0a, &wasm_vec(&[code_body(&[], &helper), code_body(&[], &main)]));
    wasm_section(&mut m, 0x0b, &wasm_vec(&[data_seg(0, b"hello, wasm\n")]));
    m
}

/// `env.print_i32`-only module (funcidx 0 = import, funcidx 1 = main). `main`
/// prints 42, then -7, then 0 — decimal formatting incl. sign and zero.
/// Expected stdout: `"42\n-7\n0\n"`.
fn print_i32_module() -> Vec<u8> {
    let mut m = MAGIC.to_vec();
    let types = wasm_vec(&[functype(&[I32], &[]), functype(&[], &[])]);
    wasm_section(&mut m, 0x01, &types);
    wasm_section(&mut m, 0x02, &wasm_vec(&[import_entry("env", "print_i32", &[0x00, 0x00])]));
    wasm_section(&mut m, 0x03, &wasm_vec(&[vec![0x01]])); // main : t1
    let exports = wasm_vec(&[{
        let mut e = wasm_vec(&"main".bytes().map(|c| vec![c]).collect::<Vec<_>>());
        e.push(0x00);
        leb_u32(1, &mut e);
        e
    }]);
    wasm_section(&mut m, 0x07, &exports);
    let main = {
        let mut b = c32(42);
        b.extend([0x10, 0x00]); // call 0 (print_i32)
        b.extend(c32(-7));
        b.extend([0x10, 0x00]);
        b.extend(c32(0));
        b.extend([0x10, 0x00]);
        b.push(0x0b);
        b
    };
    wasm_section(&mut m, 0x0a, &wasm_vec(&[code_body(&[], &main)]));
    m
}

/// `env.print_str`-only module. `main` writes `msg` from `ptr` (a data segment).
/// With `ptr`/`len` in bounds it prints the string; the OOB variant traps.
fn print_str_module(ptr: i32, len: i32, msg: &[u8]) -> Vec<u8> {
    let mut m = MAGIC.to_vec();
    let types = wasm_vec(&[functype(&[I32, I32], &[]), functype(&[], &[])]);
    wasm_section(&mut m, 0x01, &types);
    wasm_section(&mut m, 0x02, &wasm_vec(&[import_entry("env", "print_str", &[0x00, 0x00])]));
    wasm_section(&mut m, 0x03, &wasm_vec(&[vec![0x01]])); // main : t1
    wasm_section(&mut m, 0x05, &[0x01, 0x00, 0x01]); // 1 memory, min 1 page
    let exports = wasm_vec(&[
        {
            let mut e = wasm_vec(&"main".bytes().map(|c| vec![c]).collect::<Vec<_>>());
            e.push(0x00);
            leb_u32(1, &mut e);
            e
        },
        {
            let mut e = wasm_vec(&"memory".bytes().map(|c| vec![c]).collect::<Vec<_>>());
            e.push(0x02);
            leb_u32(0, &mut e);
            e
        },
    ]);
    wasm_section(&mut m, 0x07, &exports);
    let main = {
        let mut b = c32(ptr);
        b.extend(c32(len));
        b.extend([0x10, 0x00]); // call 0 (print_str)
        b.push(0x0b);
        b
    };
    wasm_section(&mut m, 0x0a, &wasm_vec(&[code_body(&[], &main)]));
    wasm_section(&mut m, 0x0b, &wasm_vec(&[data_seg(0, msg)]));
    m
}

/// Run the file-run fixture with `mod.wasm` = `bytes`, capturing STDOUT on the
/// tree-walker AND the MIR engine (foreign_io shims). Returns
/// (tw_ret, tw_stdout, mir_ret, mir_stdout).
fn run_wasm_file_stdout(bytes: &[u8], tag: &str) -> (i64, Vec<u8>, i64, Vec<u8>) {
    let dir =
        std::env::temp_dir().join(format!("candor-wasm-print-{}-{}", tag, std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("mod.wasm"), bytes).unwrap();

    foreign_io::reset();
    foreign_io::set_root(&dir);
    foreign_io::register_std_io();
    let tw = match run_source_real(FILE_RUN_SRC) {
        RunResult::Ok(r) => r.ret,
        RunResult::Fault(f) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: tree-walker faulted: {}", f.to_json());
        }
        RunResult::CheckErrors(d) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>());
        }
        RunResult::ParseError(d) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: parse error: {}", d.to_json());
        }
    };
    let tw_out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();

    foreign_io::reset();
    foreign_io::set_root(&dir);
    foreign_io::register_std_io();
    let mir = match run_source_real_mir(FILE_RUN_SRC) {
        MirRunResult::Ok(r) => r.ret,
        MirRunResult::Fault(f) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: MIR faulted: {}", f.to_json());
        }
        MirRunResult::Unsupported(msg) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: MIR unsupported: {msg}");
        }
        MirRunResult::CheckErrors(d) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: MIR check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>());
        }
        MirRunResult::ParseError(d) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: MIR parse error: {}", d.to_json());
        }
    };
    let mir_out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();

    let _ = std::fs::remove_dir_all(&dir);
    (tw, tw_out, mir, mir_out)
}

/// The file-run fixture must TRAP on `bytes` on both interp engines (used for the
/// OOB `print_str`). Returns the two fault kinds.
fn run_wasm_file_fault(bytes: &[u8], tag: &str) -> (FaultKind, FaultKind) {
    let dir = std::env::temp_dir()
        .join(format!("candor-wasm-print-oob-{}-{}", tag, std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("mod.wasm"), bytes).unwrap();

    foreign_io::reset();
    foreign_io::set_root(&dir);
    foreign_io::register_std_io();
    let tw = match run_source_real(FILE_RUN_SRC) {
        RunResult::Fault(f) => f.kind,
        RunResult::Ok(r) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: tree-walker expected a trap, got ret {}", r.ret);
        }
        RunResult::CheckErrors(d) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>());
        }
        RunResult::ParseError(d) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: parse error: {}", d.to_json());
        }
    };
    foreign_io::unregister_std_io();

    foreign_io::reset();
    foreign_io::set_root(&dir);
    foreign_io::register_std_io();
    let mir = match run_source_real_mir(FILE_RUN_SRC) {
        MirRunResult::Fault(f) => f.kind,
        MirRunResult::Ok(r) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: MIR expected a trap, got ret {}", r.ret);
        }
        MirRunResult::Unsupported(msg) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: MIR unsupported: {msg}");
        }
        MirRunResult::CheckErrors(d) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: MIR check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>());
        }
        MirRunResult::ParseError(d) => {
            foreign_io::unregister_std_io();
            panic!("{tag}: MIR parse error: {}", d.to_json());
        }
    };
    foreign_io::unregister_std_io();

    let _ = std::fs::remove_dir_all(&dir);
    (tw, mir)
}

/// Run `bytes` through wasmi with the SAME host imports registered against a
/// captured buffer (`env.print_i32`/`env.print_str`), returning the captured
/// output — the reference for the print-path differential. `print_str` reads the
/// exported linear memory and traps (Err) on an out-of-range span.
fn wasmi_run_print(bytes: &[u8]) -> Result<Vec<u8>, String> {
    use wasmi::{Caller, Engine, Error, Extern, Linker, Module, Store};
    let engine = Engine::default();
    let module = Module::new(&engine, bytes).map_err(|e| format!("compile: {e}"))?;
    let mut store: Store<Vec<u8>> = Store::new(&engine, Vec::new());
    let mut linker = <Linker<Vec<u8>>>::new(&engine);
    linker
        .func_wrap("env", "print_i32", |mut caller: Caller<'_, Vec<u8>>, x: i32| {
            let mut s = format!("{x}\n").into_bytes();
            caller.data_mut().append(&mut s);
        })
        .map_err(|e| format!("link print_i32: {e}"))?;
    linker
        .func_wrap(
            "env",
            "print_str",
            |mut caller: Caller<'_, Vec<u8>>, ptr: i32, len: i32| -> Result<(), Error> {
                let mem = match caller.get_export("memory") {
                    Some(Extern::Memory(m)) => m,
                    _ => return Err(Error::new("print_str: no memory export")),
                };
                let start = ptr as usize;
                let end = start
                    .checked_add(len as usize)
                    .ok_or_else(|| Error::new("print_str: length overflow"))?;
                let chunk = {
                    let data = mem.data(&caller);
                    if end > data.len() {
                        return Err(Error::new("print_str: out of bounds"));
                    }
                    data[start..end].to_vec()
                };
                caller.data_mut().extend_from_slice(&chunk);
                Ok(())
            },
        )
        .map_err(|e| format!("link print_str: {e}"))?;
    let instance =
        linker.instantiate_and_start(&mut store, &module).map_err(|e| format!("instantiate: {e}"))?;
    let func = instance.get_func(&store, "main").ok_or("no `main` export")?;
    func.call(&mut store, &[], &mut []).map_err(|e| format!("trap: {e}"))?;
    Ok(store.into_data())
}

#[test]
fn m4_mixed_import_and_defined_calls_right_target() {
    let _g = io_lock();
    // ONE module that imports two host funcs (indices 0,1) and defines two bodies
    // (indices 2,3): main (3) calls the print_str import (0), then the defined
    // helper (2), which calls the print_i32 import (1). Getting the index space
    // wrong would call the wrong target and change (or crash) the output.
    let bytes = print_mixed_module();
    let expected = b"hello, wasm\n42\n".to_vec();
    let (tw, tw_out, mir, mir_out) = run_wasm_file_stdout(&bytes, "mixed");
    assert_eq!(tw, 0, "main returns nothing -> 0");
    assert_eq!(mir, 0);
    assert_eq!(tw_out, expected, "tree-walker captured stdout");
    assert_eq!(mir_out, expected, "MIR captured stdout");
    // Differential: the SAME module through wasmi with the same host imports.
    let w = wasmi_run_print(&bytes).expect("wasmi runs the mixed print module");
    assert_eq!(w, expected, "wasmi captured stdout");
    assert_eq!(tw_out, w, "candor-interp captured stdout == wasmi captured stdout");
}

#[test]
fn m4_print_i32_decimal() {
    let _g = io_lock();
    let bytes = print_i32_module();
    let expected = b"42\n-7\n0\n".to_vec();
    let (_, tw_out, _, mir_out) = run_wasm_file_stdout(&bytes, "pi32");
    assert_eq!(tw_out, expected, "tree-walker print_i32 decimal/sign/zero");
    assert_eq!(mir_out, expected, "MIR print_i32");
    let w = wasmi_run_print(&bytes).expect("wasmi runs print_i32 module");
    assert_eq!(w, expected, "wasmi print_i32");
    assert_eq!(tw_out, w, "candor == wasmi (print_i32)");
}

#[test]
fn m4_print_str_reads_linear_memory() {
    let _g = io_lock();
    // print_str genuinely reads the string out of the Vec[u8] linear memory: the
    // data segment places the bytes, and a sub-slice (offset 7, len 5) proves it
    // reads the requested window, not a hardcoded string.
    let full = print_str_module(0, 12, b"hello, wasm\n");
    let (_, tw_out, _, mir_out) = run_wasm_file_stdout(&full, "pstr_full");
    assert_eq!(tw_out, b"hello, wasm\n".to_vec());
    assert_eq!(mir_out, b"hello, wasm\n".to_vec());
    assert_eq!(wasmi_run_print(&full).unwrap(), b"hello, wasm\n".to_vec());
    assert_eq!(tw_out, wasmi_run_print(&full).unwrap(), "candor == wasmi (print_str)");

    let sub = print_str_module(7, 5, b"hello, wasm\n"); // "wasm\n"
    let (_, tw_sub, _, mir_sub) = run_wasm_file_stdout(&sub, "pstr_sub");
    assert_eq!(tw_sub, b"wasm\n".to_vec(), "reads the requested memory window");
    assert_eq!(mir_sub, b"wasm\n".to_vec());
    assert_eq!(wasmi_run_print(&sub).unwrap(), b"wasm\n".to_vec());
    assert_eq!(tw_sub, wasmi_run_print(&sub).unwrap());
}

#[test]
fn m4_print_str_out_of_bounds_traps() {
    let _g = io_lock();
    // ptr 65530 + len 12 = 65542 > 65536 (one page): print_str's bounds check
    // TRAPS on both interp engines, and wasmi's host errors too.
    let oob = print_str_module(65530, 12, b"hello, wasm\n");
    let (tw, mir) = run_wasm_file_fault(&oob, "pstr_oob");
    assert_eq!(tw, FaultKind::Panic, "tree-walker OOB print_str traps");
    assert_eq!(mir, FaultKind::Panic, "MIR OOB print_str traps");
    assert!(wasmi_run_print(&oob).is_err(), "wasmi host errors on OOB print_str");
}