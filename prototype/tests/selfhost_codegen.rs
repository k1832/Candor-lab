//! The N1 gate: the FIRST Candor NATIVE code generator. `selfhost/codegen/codegen.cnr`
//! (composed with the `lexer` + `parser` modules) walks the parser's AST arena for
//! each in-subset scalar fixture and EMITS x86-64 assembly TEXT (AT&T syntax)
//! through the `trace` byte sink. The harness reconstructs that `.s` text, links it
//! with the static C runtime (`src/backend/aot_runtime.c`) via the system `cc` into
//! a REAL ELF process, runs it, and asserts its observable outcome — θ (stdout
//! trace), the process exit byte, and the `(kind, span)` fault JSON on stderr — is
//! byte-exact to the tree-walking oracle (`run_source_real`). Passing proves Candor
//! can emit a runnable native artifact for the scalar subset.
//!
//! Like the AOT gate, this FAILS LOUDLY if `cc` is unavailable: a code generator
//! that cannot produce a runnable artifact is not verifiable.

use std::path::{Path, PathBuf};
use std::process::Command;

use candor_proto::interp::{Fault, FaultKind};
use candor_proto::{run_source, run_source_real, RunResult};

mod selfhost_modtree;
use selfhost_modtree::{run_module_tree, trace_text};

const LEXER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/lexer/lexer.cnr"));
const PARSER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/parser/parser.cnr"));
const CODEGEN_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/codegen/codegen.cnr"));
const LAYOUT_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/layout/layout.cnr"));

#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    Ok { exit: u8, trace: Vec<i64> },
    Fault { kind: String, start: usize, end: usize },
}

fn kind_str(k: FaultKind) -> String {
    use FaultKind::*;
    match k {
        Overflow => "overflow",
        DivByZero => "div_by_zero",
        Bounds => "bounds",
        ConvLoss => "conv_loss",
        Assert => "assert",
        Requires => "requires",
        Ensures => "ensures",
        Panic => "panic",
        BadPointer => "bad_pointer",
        NoForeignRuntime => "no_foreign_runtime",
    }
    .to_string()
}

fn oracle_fault(f: &Fault) -> Outcome {
    Outcome::Fault { kind: kind_str(f.kind), start: f.span.start, end: f.span.end }
}

fn oracle_src(src: &str, real: bool) -> Outcome {
    let r = if real { run_source_real(src) } else { run_source(src) };
    match r {
        RunResult::Ok(run) => Outcome::Ok { exit: run.ret as u8, trace: run.trace },
        RunResult::Fault(f) => oracle_fault(&f),
        RunResult::CheckErrors(d) => {
            panic!("fixture has check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("fixture parse error: {}", d.to_json()),
    }
}

/// Extract a JSON field `"<key>":<digits>` or `"<key>":"<str>"`.
fn field<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let pat = format!("\"{key}\":");
    let i = json.find(&pat)? + pat.len();
    let rest = &json[i..];
    if let Some(stripped) = rest.strip_prefix('"') {
        let end = stripped.find('"')?;
        Some(&stripped[..end])
    } else {
        let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
        Some(&rest[..end])
    }
}

/// Generate the root `main.cnr`: lex the embedded source, then run codegen_dump
/// (which emits the `.s` text, one byte per `trace`).
fn candor_main(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut m = String::from(
        "use lexer::{Buf, mk, lex};\nuse codegen::{codegen_dump};\n\nfn main() -> i64 {\n",
    );
    m.push_str(&format!("    let src: [{}]u8 = [", bytes.len()));
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            m.push_str(", ");
        }
        m.push_str(&format!("{b}u8"));
    }
    m.push_str("];\n");
    m.push_str("    let mut buf: Buf = Buf { toks: [mk(0, 0usize, 0usize); 49152], n: 0usize };\n");
    m.push_str("    let cnt: usize = lex(slice_of(src), write buf);\n");
    m.push_str("    codegen_dump(slice_of(src), read buf);\n");
    m.push_str("    return conv i64 cnt;\n}\n");
    m
}

/// Run codegen.cnr over `src` in the tree-walker and return the emitted `.s` text.
fn candor_asm(src: &str) -> String {
    let main = candor_main(src);
    let modules = [
        ("lexer.cnr", LEXER_SRC),
        ("parser.cnr", PARSER_SRC),
        ("layout.cnr", LAYOUT_SRC),
        ("codegen.cnr", CODEGEN_SRC),
    ];
    match run_module_tree(&modules, &main) {
        RunResult::Ok(run) => trace_text(&run),
        RunResult::Fault(f) => panic!("codegen.cnr faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!(
            "codegen.cnr has check errors: {:?}",
            d.iter().map(|x| &x.code).collect::<Vec<_>>()
        ),
        RunResult::ParseError(d) => panic!("codegen.cnr parse error: {}", d.to_json()),
    }
}

fn runtime_c() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/backend/aot_runtime.c")
}

fn cc_available() -> bool {
    Command::new("cc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Assemble+link the Candor-emitted `.s` with the C runtime, run it, translate
/// (exit, stdout, stderr) into an `Outcome`.
fn native_outcome(asm: &str, tag: &str) -> Result<Outcome, String> {
    let dir = std::env::temp_dir().join(format!("candor-codegen-{}-{}", std::process::id(), tag));
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir: {e}"))?;
    let spath = dir.join("prog.s");
    let out = dir.join("prog");
    std::fs::write(&spath, asm).map_err(|e| format!("write asm: {e}"))?;

    let status = Command::new("cc")
        .arg("-no-pie")
        .arg(&spath)
        .arg(runtime_c())
        .arg("-pthread")
        .arg("-o")
        .arg(&out)
        .output()
        .map_err(|e| format!("cc invocation failed: {e}"))?;
    if !status.status.success() {
        let _ = std::fs::remove_dir_all(&dir);
        return Err(format!(
            "cc failed:\n{}\n--- asm ---\n{asm}",
            String::from_utf8_lossy(&status.stderr)
        ));
    }
    let output = Command::new(&out)
        .output()
        .map_err(|e| format!("could not run compiled program: {e}"))?;
    let _ = std::fs::remove_dir_all(&dir);

    let code = output.status.code();
    let stderr = String::from_utf8_lossy(&output.stderr);
    // A fault is exit 2 WITH the (kind, span) JSON on stderr; a plain program that
    // returns 2 also exits 2 but writes nothing to stderr (INV: disambiguate).
    if code == Some(2) && stderr.contains("\"kind\"") {
        let kind = field(&stderr, "kind").ok_or("no kind in fault JSON")?.to_string();
        let start: usize = field(&stderr, "start").ok_or("no start")?.parse().map_err(|_| "bad start")?;
        let end: usize = field(&stderr, "end").ok_or("no end")?.parse().map_err(|_| "bad end")?;
        return Ok(Outcome::Fault { kind, start, end });
    }
    let exit = code.ok_or("compiled process killed by signal")? as u8;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trace: Vec<i64> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.trim().parse::<i64>().expect("trace line is an integer"))
        .collect();
    Ok(Outcome::Ok { exit, trace })
}

fn assert_native_eq_oracle(src: &str, real: bool, tag: &str) {
    let o = oracle_src(src, real);
    let asm = candor_asm(src);
    let a = native_outcome(&asm, tag).expect("assemble+link+run should succeed");
    assert_eq!(a, o, "native codegen vs oracle divergence for:\n{src}\n--- asm ---\n{asm}");
}

// ---------------------------------------------------------------------------
// The scalar OK slice (traces + exit codes) and the fault axes (kind + span).
// Sources are the tree-walker's `real` syntax (`.cnr`), the same corpus the L1
// lowering gate exercises, plus the AOT gate's scalar OK/FAULT programs.
// ---------------------------------------------------------------------------

const OK: &[&str] = &[
    "fn main() -> i64 { let a: i64 = 20; let b: i64 = 22; trace(a); trace(b); return a + b; }",
    "fn fib(n: i64) -> i64 { if n < 2 { return n; } return fib(n - 1) + fib(n - 2); } fn main() -> i64 { let r: i64 = fib(10); trace(r); return r; }",
    "fn main() -> i64 { let mut s: i64 = 0; let mut i: i64 = 0; while i < 5 { s = s + i; trace(i); i = i + 1; } trace(s); return s; }",
    "fn main() -> i64 { let a: u8 = 12u8; let b: u8 = 10u8; trace(conv i64 (a & b)); trace(conv i64 (a | b)); trace(conv i64 (a ^ b)); return 0; }",
    "fn main() -> i64 { let a: i64 = 6i64; let b: i64 = 7i64; return a * b + 100i64 - 42i64 / 2i64; }",
    "fn main() -> i64 { let a: i64 = 17i64; return a % 5i64; }",
    "fn main() -> i64 { let x: i64 = 3i64; if x > 5i64 { return 1i64; } else if x > 2i64 { return 2i64; } else { return 3i64; } }",
    "fn main() -> i64 { let mut sum: i64 = 0i64; let mut i: i64 = 1i64; while i <= 5i64 { sum = sum + i; i = i + 1i64; } trace(sum); return sum; }",
    "fn main() -> i64 { let mut i: i64 = 0i64; let mut sum: i64 = 0i64; loop { i = i + 1i64; if i > 10i64 { break; } if i % 2i64 == 0i64 { continue; } sum = sum + i; } return sum; }",
    "fn fact(n: i64) -> i64 { if n <= 1i64 { return 1i64; } return n * fact(n - 1i64); } fn main() -> i64 { return fact(5i64); }",
    "fn main() -> i64 { let z: i64 = 0i64; let mut n: i64 = 0i64; if (z == 0i64) || (10i64 / z == 1i64) { n = n + 1i64; } if (z != 0i64) && (10i64 / z == 1i64) { n = n + 100i64; } return n; }",
    "fn main() -> i64 { let a: i64 = 5i64; let b: i64 = 7i64; let mut n: i64 = 0i64; if a < b { n = n + 1i64; } if a <= 5i64 { n = n + 10i64; } if b > a { n = n + 100i64; } if a >= 5i64 { n = n + 1000i64; } if a == 5i64 { n = n + 10000i64; } if a != b { n = n + 100000i64; } return n; }",
    "fn main() -> i64 { let a: i64 = 12i64; let b: i64 = 10i64; let c: i64 = (a & b) | (a ^ b); let d: i64 = (1i64 << 4i64) + (256i64 >> 2i64); trace(c); trace(d); return c + d; }",
    "fn main() -> i64 { let x: i64 = 5i64; let y: i64 = -x; let b: bool = false; if !b { return y - 2i64; } return y; }",
    "fn main() -> i64 { let x: i64 = 4i64; assert(x == 4i64); return x; }",
    "fn main() -> i64 { trace(10i64); trace(20i64); trace(30i64); return 0i64; }",
    "fn main() -> i64 { let x: i8 = 100i8 + 20i8; if x == 120i8 { return 1i64; } return 0i64; }",
    "fn main() -> i64 { let a: u64 = 18446744073709551615u64; trace(a); return 0i64; }",
];

const FAULTS: &[&str] = &[
    "fn main() -> i64 { let a: i32 = 2147483647i32; let b: i32 = a + 1i32; return conv i64 (b); }",
    "fn main() -> i64 { let z: i64 = 0; let q: i64 = 10 / z; return q; }",
    "fn main() -> i64 { let a: i64 = 300; let b: i8 = conv i8 (a); return 0; }",
    "fn main() -> i64 { let x: i64 = 3; assert(x > 10); return 0; }",
    "fn main() -> i64 { panic(\"boom\"); return 0; }",
    "fn main() -> i64 { let a: u8 = 1u8; let b: u8 = a << 9u8; return 0; }",
    "fn main() -> i64 { let x: i8 = 100i8 + 50i8; return 0i64; }",
    "fn main() -> i64 { let x: u8 = 200u8 + 100u8; return 0i64; }",
    "fn main() -> i64 { let a: u64 = 10000000000000000000u64; let b: u64 = 9000000000000000000u64; let c: u64 = a + b; return 0i64; }",
    "fn main() -> i64 { let a: u64 = 3u64; let b: u64 = 5u64; let c: u64 = a - b; return 0i64; }",
    "fn main() -> i64 { let a: i64 = 4000000000000i64; let b: i64 = 4000000000i64; let c: i64 = a * b; return c; }",
];

fn on_big_stack<F: FnOnce() + Send + 'static>(f: F) {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(f)
        .expect("spawn big-stack thread")
        .join()
        .expect("gate thread panicked");
}

#[test]
fn candor_native_codegen_equal_to_oracle_over_scalar_subset() {
    assert!(cc_available(), "cc/linker unavailable: cannot assemble+link the emitted .s");
    on_big_stack(|| {
        for (i, src) in OK.iter().enumerate() {
            assert_native_eq_oracle(src, true, &format!("ok{i}"));
        }
        let mut faults = 0usize;
        for (i, src) in FAULTS.iter().enumerate() {
            let o = oracle_src(src, true);
            assert!(matches!(o, Outcome::Fault { .. }), "expected a fault:\n{src}");
            assert_native_eq_oracle(src, true, &format!("fault{i}"));
            faults += 1;
        }
        assert!(faults >= 6, "expected the full scalar fault axes");
        eprintln!(
            "selfhost codegen (N1): {} scalar programs codegen -> assemble -> link -> run byte-exact vs oracle ({} OK, {} faults)",
            OK.len() + FAULTS.len(),
            OK.len(),
            FAULTS.len()
        );
    });
}

// ---------------------------------------------------------------------------
// N2: flat aggregates (structs + arrays). The S2 aggregate fixtures — struct
// field read/write, nested structs, struct by-value params/returns, array
// literals (listed + repeat), index read/write, arrays-of-structs / structs-of-
// arrays, and the array Bounds fault (kind + span byte-exact). Each codegen ->
// .s -> cc + aot_runtime.c -> run, compared to run_source_real.
// ---------------------------------------------------------------------------

macro_rules! agg_fixture {
    ($konst:ident, $file:literal) => {
        const $konst: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/selfhost_interp/",
            $file
        ));
    };
}
agg_fixture!(STRUCT_FIELD, "struct_field.cnr");
agg_fixture!(NESTED_STRUCT, "nested_struct.cnr");
agg_fixture!(FIELD_ASSIGN, "field_assign.cnr");
agg_fixture!(STRUCT_PARAM_RET, "struct_param_ret.cnr");
agg_fixture!(STRUCT_MIXED_WIDTH, "struct_mixed_width.cnr");
agg_fixture!(ARRAY_INDEX, "array_index.cnr");
agg_fixture!(ARRAY_REPEAT, "array_repeat.cnr");
agg_fixture!(INDEX_ASSIGN, "index_assign.cnr");
agg_fixture!(ARRAY_OF_STRUCTS, "array_of_structs.cnr");
agg_fixture!(STRUCT_WITH_ARRAY, "struct_with_array.cnr");
agg_fixture!(AGGREGATE_MIXED, "aggregate_mixed.cnr");
agg_fixture!(ARRAY_BOUNDS, "array_bounds.cnr");

const AGG_OK: &[&str] = &[
    STRUCT_FIELD,
    NESTED_STRUCT,
    FIELD_ASSIGN,
    STRUCT_PARAM_RET,
    STRUCT_MIXED_WIDTH,
    ARRAY_INDEX,
    ARRAY_REPEAT,
    INDEX_ASSIGN,
    ARRAY_OF_STRUCTS,
    STRUCT_WITH_ARRAY,
    AGGREGATE_MIXED,
];

#[test]
fn candor_native_codegen_equal_to_oracle_over_aggregate_subset() {
    assert!(cc_available(), "cc/linker unavailable: cannot assemble+link the emitted .s");
    on_big_stack(|| {
        for (i, src) in AGG_OK.iter().enumerate() {
            assert_native_eq_oracle(src, true, &format!("agg_ok{i}"));
        }
        // The array Bounds fault: kind + span (the base array's span) byte-exact.
        let o = oracle_src(ARRAY_BOUNDS, true);
        assert!(matches!(o, Outcome::Fault { .. }), "expected a bounds fault:\n{ARRAY_BOUNDS}");
        assert_native_eq_oracle(ARRAY_BOUNDS, true, "agg_bounds");
        eprintln!(
            "selfhost codegen (N2): {} flat-aggregate programs codegen -> assemble -> link -> run byte-exact vs oracle ({} OK, 1 bounds fault)",
            AGG_OK.len() + 1,
            AGG_OK.len()
        );
    });
}

// ---------------------------------------------------------------------------
// N3: the MOVE/DROP SCHEDULE with trace-on-drop. The S3 drop fixtures mirror
// lower.cnr's L3 ownership/schedule: owned needs-drop locals drop at scope exit /
// return / loop-exit in REVERSE declaration order, a struct runs its drop hook
// (a `call` to the emitted `cnr_drophk_<Name>` asm fn, self address in %rdi) then
// its needs-drop fields in reverse; a (whole-/partial-)moved path is not dropped;
// a fault aborts without dropping. The trace-on-drop ORDER is the load-bearing
// signal. Each codegen -> .s -> cc + aot_runtime.c -> run, vs run_source_real.
// ---------------------------------------------------------------------------

agg_fixture!(DROP_SINGLE, "drop_single.cnr");
agg_fixture!(DROP_SCOPE_ORDER, "drop_scope_order.cnr");
agg_fixture!(DROP_MOVE_SUPPRESS, "drop_move_suppress.cnr");
agg_fixture!(DROP_PARTIAL_MOVE, "drop_partial_move.cnr");
agg_fixture!(DROP_MOVE_RETURN, "drop_move_return.cnr");
agg_fixture!(DROP_BREAK, "drop_break.cnr");
agg_fixture!(DROP_NESTED, "drop_nested.cnr");
agg_fixture!(DROP_PARAM, "drop_param.cnr");

const DROP_OK: &[&str] = &[
    DROP_SINGLE,
    DROP_SCOPE_ORDER,
    DROP_MOVE_SUPPRESS,
    DROP_PARTIAL_MOVE,
    DROP_MOVE_RETURN,
    DROP_BREAK,
    DROP_NESTED,
    DROP_PARAM,
];

#[test]
fn candor_native_codegen_equal_to_oracle_over_drop_subset() {
    assert!(cc_available(), "cc/linker unavailable: cannot assemble+link the emitted .s");
    on_big_stack(|| {
        for (i, src) in DROP_OK.iter().enumerate() {
            assert_native_eq_oracle(src, true, &format!("drop_ok{i}"));
        }
        eprintln!(
            "selfhost codegen (N3): {} drop-schedule programs codegen -> assemble -> link -> run byte-exact vs oracle (trace-on-drop order load-bearing)",
            DROP_OK.len()
        );
    });
}

// ---------------------------------------------------------------------------
// N4: ENUMS + MATCH. The S4 enum fixtures mirror lower.cnr's L4: enum construction
// (u64 tag@0 + payload store/copy), the match tag-switch jump chain (first-match,
// payload binds), and the consuming-match / tag-directed enum drop N3 interaction.
// enum_drop_payload's trace-on-drop ORDER is load-bearing. Each codegen -> .s -> cc +
// aot_runtime.c -> run, compared byte-exact to run_source_real.
// ---------------------------------------------------------------------------

agg_fixture!(ENUM_CONSTRUCT_MATCH, "enum_construct_match.cnr");
agg_fixture!(MATCH_WILDCARD, "match_wildcard.cnr");
agg_fixture!(ENUM_MULTI_VARIANT, "enum_multi_variant.cnr");
agg_fixture!(MATCH_BIND_MULTI, "match_bind_multi.cnr");
agg_fixture!(ENUM_RESULT_SHAPE, "enum_result_shape.cnr");
agg_fixture!(ENUM_DROP_PAYLOAD, "enum_drop_payload.cnr");

const ENUM_OK: &[&str] = &[
    ENUM_CONSTRUCT_MATCH,
    MATCH_WILDCARD,
    ENUM_MULTI_VARIANT,
    MATCH_BIND_MULTI,
    ENUM_RESULT_SHAPE,
    ENUM_DROP_PAYLOAD,
];

#[test]
fn candor_native_codegen_equal_to_oracle_over_enum_subset() {
    assert!(cc_available(), "cc/linker unavailable: cannot assemble+link the emitted .s");
    on_big_stack(|| {
        for (i, src) in ENUM_OK.iter().enumerate() {
            assert_native_eq_oracle(src, true, &format!("enum_ok{i}"));
        }
        eprintln!(
            "selfhost codegen (N4): {} enum/match programs codegen -> assemble -> link -> run byte-exact vs oracle (tag-switch jump chain + tag-directed enum drop)",
            ENUM_OK.len()
        );
    });
}

// ---------------------------------------------------------------------------
// N5: BOX/ALLOCATOR ABI + rawptr/fnptr surface. The S5/S6a fixtures mirror
// lower.cnr's L5: rawptr/fnptr scalars + the pointer intrinsics (address model,
// MEM_BASE), statics + fn-ptr values + indirect `call *reg`, the alloc vtable
// INDIRECT call for box/unbox/BoxResult, and alloc-on-drop (pointee-then-free).
// Each codegen -> .s -> cc + aot_runtime.c -> run, byte-exact vs run_source_real.
// ---------------------------------------------------------------------------

agg_fixture!(N5_STATIC_FNPTR, "static_fnptr_indirect_call.cnr");
agg_fixture!(N5_PTR_ROUNDTRIP, "ptr_roundtrip.cnr");
agg_fixture!(N5_CAST_PTR_READ, "cast_ptr_read.cnr");
agg_fixture!(N5_ALLOC_ABI, "alloc_abi.cnr");
agg_fixture!(N5_BOX_UNBOX_SCALAR, "box_unbox_scalar.cnr");
agg_fixture!(N5_BOX_STRUCT, "box_struct.cnr");
agg_fixture!(N5_UNBOX_PATH, "unbox_path.cnr");
agg_fixture!(N5_BOXRESULT_OOM, "boxresult_oom.cnr");
agg_fixture!(N5_BOX_DROP_FREES, "box_drop_frees.cnr");
agg_fixture!(N5_NESTED_BOX, "nested_box.cnr");
agg_fixture!(N5_HIGH_ADDR, "high_addr_roundtrip.cnr");
agg_fixture!(N5_OFFSETOF_FIRST, "offsetof_first_field.cnr");
agg_fixture!(N5_OFFSETOF_NONZERO, "offsetof_nonzero_field.cnr");
agg_fixture!(N5_PTR_OFFSET_STRIDE, "ptr_offset_stride.cnr");
agg_fixture!(N5_ENUM_PADDING, "enum_padding_copy.cnr");
agg_fixture!(N5_PAGE_BOUNDARY, "page_boundary.cnr");

const BOX_OK: &[(&str, &str)] = &[
    ("static_fnptr_indirect_call", N5_STATIC_FNPTR),
    ("ptr_roundtrip", N5_PTR_ROUNDTRIP),
    ("cast_ptr_read", N5_CAST_PTR_READ),
    ("alloc_abi", N5_ALLOC_ABI),
    ("box_unbox_scalar", N5_BOX_UNBOX_SCALAR),
    ("box_struct", N5_BOX_STRUCT),
    ("unbox_path", N5_UNBOX_PATH),
    ("boxresult_oom", N5_BOXRESULT_OOM),
    ("box_drop_frees", N5_BOX_DROP_FREES),
    ("nested_box", N5_NESTED_BOX),
    ("high_addr_roundtrip", N5_HIGH_ADDR),
    ("offsetof_first_field", N5_OFFSETOF_FIRST),
    ("offsetof_nonzero_field", N5_OFFSETOF_NONZERO),
    ("ptr_offset_stride", N5_PTR_OFFSET_STRIDE),
    ("enum_padding_copy", N5_ENUM_PADDING),
    ("page_boundary", N5_PAGE_BOUNDARY),
];

#[test]
fn candor_native_codegen_equal_to_oracle_over_box_subset() {
    assert!(cc_available(), "cc/linker unavailable: cannot assemble+link the emitted .s");
    on_big_stack(|| {
        for (tag, src) in BOX_OK.iter() {
            assert_native_eq_oracle(src, true, tag);
        }
        eprintln!(
            "selfhost codegen (N5): {} box/rawptr/fnptr/static programs codegen -> assemble -> link -> run byte-exact vs oracle",
            BOX_OK.len()
        );
    });
}

// ---------------------------------------------------------------------------
// N6 — THE MILESTONE: the systems corpus native-compiles. The five programs the
// interp (S6b) and lower (L6) arcs already run — an allocator/pool, a scheduler
// intrusive list, MMIO device state, a recursive-descent parser over a byte-
// string, and a Box-array arena — each codegen -> .s -> cc + aot_runtime.c -> run,
// byte-exact (exit / trace / fault) vs run_source_real. Proves Candor compiles
// the corpus to a native x86-64 executable with NO Rust in the compile path.
// ---------------------------------------------------------------------------

fn corpus_src(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/run").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {name}: {e}"))
}

fn run_corpus(name: &str, tag: &str) {
    assert!(cc_available(), "cc/linker unavailable: cannot assemble+link the emitted .s");
    let name = name.to_string();
    let tag = tag.to_string();
    on_big_stack(move || {
        let src = corpus_src(&name);
        assert_native_eq_oracle(&src, true, &tag);
        eprintln!("selfhost codegen (N6): {name} native-compiles byte-exact vs oracle");
    });
}

#[test]
fn n6_11_1_allocator() { run_corpus("11_1_allocator.cnr", "n6_alloc"); }
#[test]
fn n6_11_2_scheduler() { run_corpus("11_2_scheduler.cnr", "n6_sched"); }
#[test]
fn n6_11_3_mmio() { run_corpus("11_3_mmio.cnr", "n6_mmio"); }
#[test]
fn n6_11_4_parser() { run_corpus("11_4_parser.cnr", "n6_parser"); }
#[test]
fn n6_11_5_arena() { run_corpus("11_5_arena.cnr", "n6_arena"); }

