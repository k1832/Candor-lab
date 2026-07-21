//! The L1 gate: the FIRST Candor lowering. `selfhost/lower/lower.cnr` (composed
//! with the `lexer` and `parser` modules) lowers each in-subset fixture to MIR and
//! EMITS the L0 wire format (src/mir/serial.rs) through the `trace` byte sink. The
//! harness reconstructs that wire text, `deserialize`s it Rust-side into a real
//! `MirProgram`, rebuilds the runtime `items`/`consts` from the SAME source (never
//! carried in the wire), runs the precise MIR interpreter, and renders RET/TRACE/
//! FAULT. That dump must be byte-exact to the tree-walking oracle (`run_source_real`)
//! for the same source. Passing is EXECUTION equality — return value, trace
//! sequence, and fault identity (kind + span) — between Candor-emitted MIR and the
//! oracle. This proves Candor can emit an executable control-flow graph.

use std::collections::HashMap;

use candor::ast;
use candor::mir::{self, serial};
use candor::resolve::Items;
use candor::{check, generics, resolve, run_source_real, RunResult};

mod selfhost_modtree;
use selfhost_modtree::{dump_fault, dump_ok, on_big_stack, run_module_tree, trace_text};

const LEXER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../selfhost/lexer/lexer.cnr"));
const PARSER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../selfhost/parser/parser.cnr"));
const LAYOUT_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../selfhost/layout/layout.cnr"));
const MONO_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../selfhost/mono/mono.cnr"));
const LOWER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../selfhost/lower/lower.cnr"));

/// The tree-walking oracle's dump for `src`.
fn oracle_dump(src: &str) -> String {
    match run_source_real(src) {
        RunResult::Ok(run) => dump_ok(&run),
        RunResult::Fault(f) => dump_fault(&f),
        RunResult::CheckErrors(d) => {
            panic!("fixture has check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("fixture parse error: {}", d.to_json()),
    }
}

/// The runtime `items`/`consts` for `src` (from SOURCE; the wire carries neither).
fn items_and_consts(src: &str) -> (Items, HashMap<String, u64>) {
    let parsed = candor::real::parse_source(src).expect("parse");
    // For a generic fixture, the concrete `items` (struct/enum layouts, mangled
    // instance names) come from the MONOMORPHIZED program -- the same pre-pass the
    // self-hosted lowering runs -- exactly as the oracle's `run_generic` does.
    let program = if generics::is_generic_program(&parsed) {
        let (_d, insts, shapes) = check::check_generic_program(&parsed, true);
        generics::monomorphize(&parsed, &insts, &shapes).program
    } else {
        parsed
    };
    let mut resolve_diags = Vec::new();
    let items = resolve::resolve_program(&program, &mut resolve_diags);
    let mut consts = HashMap::new();
    for it in &program.items {
        if let ast::Item::Static(st) = it {
            if let ast::ExprKind::IntLit { value, .. } = &st.value.kind {
                consts.insert(st.name.clone(), *value);
            }
        }
    }
    (items, consts)
}

/// Generate the root `main.cnr`: `use` the lexer + lower, embed `src`, lex, then
/// emit the wire through `lower_dump` (each byte a `trace` value).
fn candor_main(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut m = String::from(
        "use lexer::{Buf, mk, lex};\nuse lower::{lower_dump};\n\nfn main() -> i64 {\n",
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
    m.push_str("    lower_dump(slice_of(src), read buf);\n");
    m.push_str("    return conv i64 cnt;\n}\n");
    m
}

/// Run `lower.cnr` over `src` and return the emitted L0 wire text.
fn candor_wire(src: &str) -> String {
    let main = candor_main(src);
    let modules = [
        ("lexer.cnr", LEXER_SRC),
        ("parser.cnr", PARSER_SRC),
        ("mono.cnr", MONO_SRC),
        ("layout.cnr", LAYOUT_SRC),
        ("lower.cnr", LOWER_SRC),
    ];
    match run_module_tree(&modules, &main) {
        RunResult::Ok(run) => trace_text(&run),
        RunResult::Fault(f) => panic!("lower.cnr faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!(
            "lower.cnr has check errors: {:?}",
            d.iter().map(|x| &x.code).collect::<Vec<_>>()
        ),
        RunResult::ParseError(d) => panic!("lower.cnr parse error: {}", d.to_json()),
    }
}

/// Deserialize the Candor-emitted wire, run the precise MIR interpreter, render.
fn candor_dump(src: &str) -> String {
    let wire = candor_wire(src);
    let mp = serial::deserialize(&wire).unwrap_or_else(|e| panic!("deserialize failed: {e}\n{wire}"));
    let (items, consts) = items_and_consts(src);
    match mir::interp::run(&mp, &items, &consts) {
        Ok(run) => dump_ok(&run),
        Err(f) => dump_fault(&f),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Shape {
    Ret,
    Fault,
}
use Shape::*;

fn read_fixture(rel: &str) -> String {
    let path = format!("{}/tests/fixtures/selfhost_interp/{}", env!("CARGO_MANIFEST_DIR"), rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

/// Lower one corpus program with the self-hosted lowering, deserialize its wire
/// to a real `MirProgram`, run the precise MIR interpreter, and assert the dump
/// is byte-exact to the tree-walking oracle for the same source -- plus that the
/// result has the declared shape (a real `RET` or `FAULT`). Each corpus program
/// is its own `#[test]` (see `corpus!`) so nextest runs them as parallel
/// processes; the wall clock is the slowest single program, not the serial sum.
fn check_program(rel: &'static str, shape: Shape) {
    on_big_stack(move || {
        let src = read_fixture(rel);
        let oracle = oracle_dump(&src);
        let mine = candor_dump(&src);
        assert_eq!(mine, oracle, "L1 lowering execution mismatch on {rel}");
        match shape {
            Ret => assert!(
                oracle.starts_with("RET "),
                "expected RET on {rel}, got {oracle:?}"
            ),
            Fault => assert!(
                oracle.starts_with("FAULT "),
                "expected FAULT on {rel}, got {oracle:?}"
            ),
        }
    });
}

/// One `#[test]` per corpus program, so nextest parallelizes the corpus across
/// processes instead of running it as a single serial loop.
macro_rules! corpus {
    ($($name:ident : $rel:literal => $shape:ident),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                check_program($rel, $shape);
            }
        )*
    };
}

corpus! {
    arith: "arith.cnr" => Ret,
    rem: "rem.cnr" => Ret,
    ifelse: "ifelse.cnr" => Ret,
    while_accum: "while_accum.cnr" => Ret,
    loop_break: "loop_break.cnr" => Ret,
    factorial: "factorial.cnr" => Ret,
    fib: "fib.cnr" => Ret,
    shortcircuit: "shortcircuit.cnr" => Ret,
    compare: "compare.cnr" => Ret,
    bitwise: "bitwise.cnr" => Ret,
    unary: "unary.cnr" => Ret,
    assert_pass: "assert_pass.cnr" => Ret,
    trace_multi: "trace_multi.cnr" => Ret,
    width_i8: "width_i8.cnr" => Ret,
    u64_value: "u64_value.cnr" => Ret,
    overflow_i32: "overflow_i32.cnr" => Fault,
    divzero: "divzero.cnr" => Fault,
    assert_fail: "assert_fail.cnr" => Fault,
    panic: "panic.cnr" => Fault,
    width_i8_overflow: "width_i8_overflow.cnr" => Fault,
    width_u8_overflow: "width_u8_overflow.cnr" => Fault,
    u64_add_overflow: "u64_add_overflow.cnr" => Fault,
    u64_sub_underflow: "u64_sub_underflow.cnr" => Fault,
    i64_mul_overflow: "i64_mul_overflow.cnr" => Fault,
    // S2 aggregates: structs + arrays (field offsets, strides, copyval, bounds).
    struct_field: "struct_field.cnr" => Ret,
    nested_struct: "nested_struct.cnr" => Ret,
    field_assign: "field_assign.cnr" => Ret,
    struct_param_ret: "struct_param_ret.cnr" => Ret,
    struct_mixed_width: "struct_mixed_width.cnr" => Ret,
    array_index: "array_index.cnr" => Ret,
    array_repeat: "array_repeat.cnr" => Ret,
    index_assign: "index_assign.cnr" => Ret,
    array_of_structs: "array_of_structs.cnr" => Ret,
    struct_with_array: "struct_with_array.cnr" => Ret,
    aggregate_mixed: "aggregate_mixed.cnr" => Ret,
    array_bounds: "array_bounds.cnr" => Fault,
    // S3 MOVE/DROP schedule: Drop ops + move masks + drop-hooks-as-MIR-fns. The
    // trace-on-drop order is the load-bearing signal (reverse/LIFO, hook-then-fields,
    // move-suppression, partial-move-remainder).
    drop_single: "drop_single.cnr" => Ret,
    drop_scope_order: "drop_scope_order.cnr" => Ret,
    drop_move_suppress: "drop_move_suppress.cnr" => Ret,
    drop_partial_move: "drop_partial_move.cnr" => Ret,
    drop_move_return: "drop_move_return.cnr" => Ret,
    drop_break: "drop_break.cnr" => Ret,
    drop_nested: "drop_nested.cnr" => Ret,
    drop_param: "drop_param.cnr" => Ret,
    // S4 ENUMS + MATCH: enum layout (tag@0/payload@8), the T_ENUMCTOR store, the
    // match tag-switch branch chain, payload binds, and the consuming-match /
    // tag-directed enum-drop L3 interaction (enum_drop_payload's TRACE order is
    // the load-bearing signal).
    enum_construct_match: "enum_construct_match.cnr" => Ret,
    match_wildcard: "match_wildcard.cnr" => Ret,
    enum_multi_variant: "enum_multi_variant.cnr" => Ret,
    match_bind_multi: "match_bind_multi.cnr" => Ret,
    enum_result_shape: "enum_result_shape.cnr" => Ret,
    enum_drop_payload: "enum_drop_payload.cnr" => Ret,
    // S5/S6 box/alloc/rawptr/statics/CallIndirect + pointer intrinsics (L5).
    offsetof_first_field: "offsetof_first_field.cnr" => Ret,
    offsetof_nonzero_field: "offsetof_nonzero_field.cnr" => Ret,
    ptr_roundtrip: "ptr_roundtrip.cnr" => Ret,
    cast_ptr_read: "cast_ptr_read.cnr" => Ret,
    ptr_offset_stride: "ptr_offset_stride.cnr" => Ret,
    high_addr_roundtrip: "high_addr_roundtrip.cnr" => Ret,
    page_boundary: "page_boundary.cnr" => Ret,
    enum_padding_copy: "enum_padding_copy.cnr" => Ret,
    static_fnptr_indirect_call: "static_fnptr_indirect_call.cnr" => Ret,
    alloc_abi: "alloc_abi.cnr" => Ret,
    box_unbox_scalar: "box_unbox_scalar.cnr" => Ret,
    box_struct: "box_struct.cnr" => Ret,
    unbox_path: "unbox_path.cnr" => Ret,
    boxresult_oom: "boxresult_oom.cnr" => Ret,
    box_drop_frees: "box_drop_frees.cnr" => Ret,
    nested_box: "nested_box.cnr" => Ret,
    // L6 MILESTONE: the SYSTEMS CORPUS (five real programs), lowered to MIR by the
    // self-hosted lowering and executed byte-exact against the tree-walker oracle.
    p_11_3_mmio: "11_3_mmio.cnr" => Ret,
    p_11_1_allocator: "11_1_allocator.cnr" => Ret,
    p_11_2_scheduler: "11_2_scheduler.cnr" => Ret,
    p_11_5_arena: "11_5_arena.cnr" => Ret,
    p_11_4_parser: "11_4_parser.cnr" => Ret,
    // ---- USER GENERICS via the monomorphizer pre-pass (mono.cnr), lowered to
    // MIR by the self-hosted lowering. Mirrors the interp G1 corpus.
    generics_mono3: "generics/mono3.cnr" => Ret,
    generics_mixed: "generics/mixed.cnr" => Ret,
    generics_nameval: "generics/nameval.cnr" => Ret,
    generics_pair: "generics/pair.cnr" => Ret,
    generics_genenum: "generics/genenum.cnr" => Ret,
    generics_arena: "generics/arena.cnr" => Ret,
    generics_gdrop_groundfloor: "generics/gdrop_groundfloor.cnr" => Ret,
    generics_gdrop: "generics/gdrop.cnr" => Ret,
    // T3: interface/impl method dispatch lowered to direct MIR Calls.
    generics_iface: "generics/iface.cnr" => Ret,
    generics_gimpl: "generics/gimpl.cnr" => Ret,
    generics_gbound: "generics/gbound.cnr" => Ret,
    // T5: the `?` operator + `From` widening lowered to a MIR ok/err CFG.
    generics_fromq: "generics/fromq.cnr" => Ret,
    generics_gfromq: "generics/gfromq.cnr" => Ret,
    // L-std: std collections Vec/Map/String lowered to MIR CollectionOp. Mirrors
    // the interp S7 corpus; closes the generic/std self-hosting tail.
    string_build: "string_build.cnr" => Ret,
    vec_push_get_sum: "vec_push_get_sum.cnr" => Ret,
    vec_pop_opt: "vec_pop_opt.cnr" => Ret,
    vec_struct_drop: "vec_struct_drop.cnr" => Ret,
    map_insert_contains_get: "map_insert_contains_get.cnr" => Ret,
    vec_get_oob_fault: "vec_get_oob_fault.cnr" => Fault,
    // F-LAYOUT-DRIFT regression: a Vec field inside a struct forces struct_size to
    // size the Vec (40, not 0), so the following scalar field's offset matches the
    // oracle. Before the ty_size fix this diverged.
    struct_with_vec: "struct_with_vec.cnr" => Ret,
}
