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

use candor_proto::ast;
use candor_proto::interp::{Fault, FaultKind, Run};
use candor_proto::mir::{self, serial};
use candor_proto::resolve::Items;
use candor_proto::{check, generics, resolve, run_source_real, RunResult};

mod selfhost_modtree;
use selfhost_modtree::{run_module_tree, trace_text};

const LEXER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/lexer/lexer.cnr"));
const PARSER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/parser/parser.cnr"));
const MONO_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/mono/mono.cnr"));
const LOWER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/lower/lower.cnr"));

/// Integer kind-code map shared with `selfhost_interp` / the wire renderer.
fn fault_code(k: FaultKind) -> i64 {
    match k {
        FaultKind::Overflow => 0,
        FaultKind::DivByZero => 1,
        FaultKind::Assert => 2,
        FaultKind::Panic => 3,
        FaultKind::Bounds => 4,
        FaultKind::ConvLoss => 5,
        other => panic!("out-of-subset fault kind reached the gate: {other:?}"),
    }
}

fn dump_ok(run: &Run) -> String {
    let mut s = format!("RET {}\n", run.ret);
    for v in &run.trace {
        s.push_str(&format!("TRACE {v}\n"));
    }
    s
}
fn dump_fault(f: &Fault) -> String {
    format!("FAULT {} {} {}\n", fault_code(f.kind), f.span.start, f.span.end)
}

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
    let parsed = candor_proto::real::parse_source(src).expect("parse");
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

/// The scalar + control-flow + fault subset (the S1-era in-subset fixtures).
const CORPUS: &[(&str, Shape)] = &[
    ("arith.cnr", Ret),
    ("rem.cnr", Ret),
    ("ifelse.cnr", Ret),
    ("while_accum.cnr", Ret),
    ("loop_break.cnr", Ret),
    ("factorial.cnr", Ret),
    ("fib.cnr", Ret),
    ("shortcircuit.cnr", Ret),
    ("compare.cnr", Ret),
    ("bitwise.cnr", Ret),
    ("unary.cnr", Ret),
    ("assert_pass.cnr", Ret),
    ("trace_multi.cnr", Ret),
    ("width_i8.cnr", Ret),
    ("u64_value.cnr", Ret),
    ("overflow_i32.cnr", Fault),
    ("divzero.cnr", Fault),
    ("assert_fail.cnr", Fault),
    ("panic.cnr", Fault),
    ("width_i8_overflow.cnr", Fault),
    ("width_u8_overflow.cnr", Fault),
    ("u64_add_overflow.cnr", Fault),
    ("u64_sub_underflow.cnr", Fault),
    ("i64_mul_overflow.cnr", Fault),
    // S2 aggregates: structs + arrays (field offsets, strides, copyval, bounds).
    ("struct_field.cnr", Ret),
    ("nested_struct.cnr", Ret),
    ("field_assign.cnr", Ret),
    ("struct_param_ret.cnr", Ret),
    ("struct_mixed_width.cnr", Ret),
    ("array_index.cnr", Ret),
    ("array_repeat.cnr", Ret),
    ("index_assign.cnr", Ret),
    ("array_of_structs.cnr", Ret),
    ("struct_with_array.cnr", Ret),
    ("aggregate_mixed.cnr", Ret),
    ("array_bounds.cnr", Fault),
    // S3 MOVE/DROP schedule: Drop ops + move masks + drop-hooks-as-MIR-fns. The
    // trace-on-drop order is the load-bearing signal (reverse/LIFO, hook-then-fields,
    // move-suppression, partial-move-remainder).
    ("drop_single.cnr", Ret),
    ("drop_scope_order.cnr", Ret),
    ("drop_move_suppress.cnr", Ret),
    ("drop_partial_move.cnr", Ret),
    ("drop_move_return.cnr", Ret),
    ("drop_break.cnr", Ret),
    ("drop_nested.cnr", Ret),
    ("drop_param.cnr", Ret),
    // S4 ENUMS + MATCH: enum layout (tag@0/payload@8), the T_ENUMCTOR store, the
    // match tag-switch branch chain, payload binds, and the consuming-match /
    // tag-directed enum-drop L3 interaction (enum_drop_payload's TRACE order is
    // the load-bearing signal).
    ("enum_construct_match.cnr", Ret),
    ("match_wildcard.cnr", Ret),
    ("enum_multi_variant.cnr", Ret),
    ("match_bind_multi.cnr", Ret),
    ("enum_result_shape.cnr", Ret),
    ("enum_drop_payload.cnr", Ret),
    // S5/S6 box/alloc/rawptr/statics/CallIndirect + pointer intrinsics (L5).
    ("offsetof_first_field.cnr", Ret),
    ("offsetof_nonzero_field.cnr", Ret),
    ("ptr_roundtrip.cnr", Ret),
    ("cast_ptr_read.cnr", Ret),
    ("ptr_offset_stride.cnr", Ret),
    ("high_addr_roundtrip.cnr", Ret),
    ("page_boundary.cnr", Ret),
    ("enum_padding_copy.cnr", Ret),
    ("static_fnptr_indirect_call.cnr", Ret),
    ("alloc_abi.cnr", Ret),
    ("box_unbox_scalar.cnr", Ret),
    ("box_struct.cnr", Ret),
    ("unbox_path.cnr", Ret),
    ("boxresult_oom.cnr", Ret),
    ("box_drop_frees.cnr", Ret),
    ("nested_box.cnr", Ret),
    // L6 MILESTONE: the SYSTEMS CORPUS (five real programs), lowered to MIR by the
    // self-hosted lowering and executed byte-exact against the tree-walker oracle.
    ("11_3_mmio.cnr", Ret),
    ("11_1_allocator.cnr", Ret),
    ("11_2_scheduler.cnr", Ret),
    ("11_5_arena.cnr", Ret),
    ("11_4_parser.cnr", Ret),
    // ---- USER GENERICS via the monomorphizer pre-pass (mono.cnr), lowered to
    // MIR by the self-hosted lowering. Mirrors the interp G1 corpus.
    ("generics/mono3.cnr", Ret),
    ("generics/mixed.cnr", Ret),
    ("generics/nameval.cnr", Ret),
    ("generics/pair.cnr", Ret),
    ("generics/genenum.cnr", Ret),
    ("generics/arena.cnr", Ret),
    ("generics/gdrop_groundfloor.cnr", Ret),
    ("generics/gdrop.cnr", Ret),
    // T3: interface/impl method dispatch lowered to direct MIR Calls.
    ("generics/iface.cnr", Ret),
    ("generics/gimpl.cnr", Ret),
    ("generics/gbound.cnr", Ret),
    // L-std: std collections Vec/Map/String lowered to MIR CollectionOp. Mirrors
    // the interp S7 corpus; closes the generic/std self-hosting tail.
    ("string_build.cnr", Ret),
    ("vec_push_get_sum.cnr", Ret),
    ("vec_pop_opt.cnr", Ret),
    ("vec_struct_drop.cnr", Ret),
    ("map_insert_contains_get.cnr", Ret),
    ("vec_get_oob_fault.cnr", Fault),
    // F-LAYOUT-DRIFT regression: a Vec field inside a struct forces struct_size to
    // size the Vec (40, not 0), so the following scalar field's offset matches the
    // oracle. Before the ty_size fix this diverged.
    ("struct_with_vec.cnr", Ret),
];

fn read_fixture(rel: &str) -> String {
    let path = format!("{}/tests/fixtures/selfhost_interp/{}", env!("CARGO_MANIFEST_DIR"), rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn on_big_stack<F: FnOnce() + Send + 'static>(f: F) {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(f)
        .expect("spawn big-stack thread")
        .join()
        .expect("gate thread panicked");
}

#[test]
fn candor_lowering_execution_equal_to_oracle_over_scalar_subset() {
    on_big_stack(|| {
        let mut rets = 0usize;
        let mut faults = 0usize;
        for &(rel, shape) in CORPUS {
            let src = read_fixture(rel);
            let oracle = oracle_dump(&src);
            let mine = candor_dump(&src);
            assert_eq!(mine, oracle, "L1 lowering execution mismatch on {rel}");
            match shape {
                Ret => {
                    assert!(oracle.starts_with("RET "), "expected RET on {rel}, got {oracle:?}");
                    rets += 1;
                }
                Fault => {
                    assert!(oracle.starts_with("FAULT "), "expected FAULT on {rel}, got {oracle:?}");
                    faults += 1;
                }
            }
        }
        assert!(rets > 0 && faults > 0, "corpus must exercise both returns and faults");
        eprintln!(
            "selfhost lower (L1-L5): {}/{} fixtures lower -> deserialize -> interp byte-exact vs oracle ({} returns, {} faults)",
            CORPUS.len(),
            CORPUS.len(),
            rets,
            faults
        );
    });
}


