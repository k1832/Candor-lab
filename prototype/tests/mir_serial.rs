//! The MIR serialization boundary gate (self-lowering L0): every corpus fixture is
//! lowered to MIR by the EXISTING Rust lowering, pushed through the text wire
//! (`mir::serial::serialize` -> `deserialize`), then run by the precise MIR
//! interpreter. The deserialized MIR's execution dump (RET / TRACE / FAULT, in the
//! `selfhost_interp` schema) must be byte-exact to the tree-walking oracle
//! (`run_source_real`). This proves the wire losslessly carries real MIR and that
//! `mir::interp` over deserialized MIR matches the oracle — the whole boundary,
//! validated on real MIR, before any Candor lowering (L1) emits into it.
//!
//! Three facts are asserted per fixture:
//!   1. Wire round-trip idempotence: `serialize(deserialize(serialize(p)))` equals
//!      `serialize(p)` (no field drifts across the boundary).
//!   2. Deserialized MIR == oracle (RET/TRACE/FAULT byte-exact — fault identity is
//!      kind + span.start + span.end).
//!   3. In-memory MIR == oracle (the boundary changed nothing: original ==
//!      deserialized == oracle).

use std::collections::HashMap;

use candor_proto::ast;
use candor_proto::interp::{Fault, FaultKind, Run};
use candor_proto::mir::{self, serial};
use candor_proto::resolve::Items;
use candor_proto::{check, diag, generics, real, resolve, run_source_real, RunResult};

/// Integer kind-code map shared with `selfhost_interp` — the same rendering schema.
fn fault_code(k: FaultKind) -> i64 {
    match k {
        FaultKind::Overflow => 0,
        FaultKind::DivByZero => 1,
        FaultKind::Assert => 2,
        FaultKind::Panic => 3,
        FaultKind::Bounds => 4,
        FaultKind::ConvLoss => 5,
        FaultKind::Requires => 6,
        FaultKind::Ensures => 7,
        FaultKind::BadPointer => 8,
        FaultKind::NoForeignRuntime => 9,
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

/// The tree-walking oracle's dump for `src`, in the canonical schema.
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

/// Build the MIR program plus the runtime `items`/`consts` for `src`, exactly as
/// the MIR engine's `lower_and_run` does — the `items`/`consts` are derived from
/// SOURCE (never carried in the wire), so the harness rebuilds them here.
fn lower_from_source(src: &str) -> (mir::MirProgram, Items, HashMap<String, u64>) {
    let program = real::parse_source(src).expect("parse");
    assert!(!generics::is_generic_program(&program), "corpus fixture unexpectedly generic");
    let diags = check::check_program_real(&program);
    assert!(
        !diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)),
        "fixture failed the checker"
    );
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
    let mp = mir::lower_checked(&program, &items).expect("lower to MIR");
    (mp, items, consts)
}

fn run_dump(mp: &mir::MirProgram, items: &Items, consts: &HashMap<String, u64>) -> String {
    match mir::interp::run(mp, items, consts) {
        Ok(run) => dump_ok(&run),
        Err(f) => dump_fault(&f),
    }
}

/// (fixture, expected shape) — the same corpus the self-interp gate uses, from
/// scalar through the systems corpus, so the boundary is proven on real MIR that
/// exercises fault edges, spans, aggregates, enums, boxes, fn-ptrs, statics, drops.
#[derive(Clone, Copy, PartialEq)]
enum Shape {
    Ret,
    Fault,
}
use Shape::*;

const CORPUS: &[(&str, Shape)] = &[
    // scalar core + every scalar fault kind (spans + fault identity)
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
    // aggregates: structs + arrays (field offsets, strides, copyval)
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
    // move/drop schedule (Drop move masks)
    ("drop_single.cnr", Ret),
    ("drop_scope_order.cnr", Ret),
    ("drop_move_suppress.cnr", Ret),
    ("drop_partial_move.cnr", Ret),
    ("drop_move_return.cnr", Ret),
    ("drop_break.cnr", Ret),
    ("drop_nested.cnr", Ret),
    ("drop_param.cnr", Ret),
    // enums + match
    ("enum_construct_match.cnr", Ret),
    ("match_wildcard.cnr", Ret),
    ("enum_multi_variant.cnr", Ret),
    ("match_bind_multi.cnr", Ret),
    ("enum_result_shape.cnr", Ret),
    ("enum_drop_payload.cnr", Ret),
    // allocator ABI: fn-ptrs table, statics, indirect calls, structural Alloc
    ("static_fnptr_indirect_call.cnr", Ret),
    ("ptr_roundtrip.cnr", Ret),
    ("cast_ptr_read.cnr", Ret),
    ("alloc_abi.cnr", Ret),
    // box / BoxResult / unbox / alloc-on-drop (drop_hooks referencing fn ids)
    ("box_unbox_scalar.cnr", Ret),
    ("box_struct.cnr", Ret),
    ("unbox_path.cnr", Ret),
    ("boxresult_oom.cnr", Ret),
    ("box_drop_frees.cnr", Ret),
    ("nested_box.cnr", Ret),
    // paged memory + pointer intrinsics
    ("high_addr_roundtrip.cnr", Ret),
    ("offsetof_first_field.cnr", Ret),
    ("offsetof_nonzero_field.cnr", Ret),
    ("ptr_offset_stride.cnr", Ret),
    ("enum_padding_copy.cnr", Ret),
    ("page_boundary.cnr", Ret),
    // the systems corpus: five real programs
    ("11_3_mmio.cnr", Ret),
    ("11_1_allocator.cnr", Ret),
    ("11_2_scheduler.cnr", Ret),
    ("11_5_arena.cnr", Ret),
    ("11_4_parser.cnr", Ret),
];

fn read_fixture(rel: &str) -> String {
    let path = format!("{}/tests/fixtures/selfhost_interp/{}", env!("CARGO_MANIFEST_DIR"), rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

/// The compiler-known collection corpus (Vec/Map/String, PROPOSAL-selfhost-
/// ergonomics / design 0013), lowered as MIR intrinsics: the SAME serialize ->
/// deserialize -> `mir::interp` round-trip the systems corpus proves, now over
/// programs the MIR path previously could not represent. Each must run byte-exact
/// (RET / TRACE / FAULT) to the tree-walking oracle, which already runs collections.
const COLLECTION_CORPUS: &[(&str, Shape)] = &[
    ("vec_push_get_sum.cnr", Ret),
    ("vec_pop_opt.cnr", Ret),
    ("vec_struct_drop.cnr", Ret),
    ("map_insert_contains_get.cnr", Ret),
    ("string_build.cnr", Ret),
    ("vec_get_oob_fault.cnr", Fault),
];

fn read_fixture_lower(rel: &str) -> String {
    let path = format!("{}/tests/fixtures/selfhost_lower/{}", env!("CARGO_MANIFEST_DIR"), rel);
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
fn mir_serialization_boundary_execution_equal_to_oracle() {
    on_big_stack(|| {
        let mut rets = 0usize;
        let mut faults = 0usize;
        for &(rel, shape) in CORPUS {
            let src = read_fixture(rel);
            let oracle = oracle_dump(&src);

            let (mp, items, consts) = lower_from_source(&src);

            // (1) wire round-trip idempotence: no field drifts across the boundary.
            let wire1 = serial::serialize(&mp);
            let mp2 = serial::deserialize(&wire1)
                .unwrap_or_else(|e| panic!("deserialize failed on {rel}: {e}"));
            let wire2 = serial::serialize(&mp2);
            assert_eq!(wire1, wire2, "wire not round-trip idempotent on {rel}");

            // (2) deserialized MIR runs byte-exact to the oracle.
            let via_wire = run_dump(&mp2, &items, &consts);
            assert_eq!(via_wire, oracle, "deserialized MIR diverges from oracle on {rel}");

            // (3) the boundary changed nothing: in-memory MIR == deserialized == oracle.
            let in_memory = run_dump(&mp, &items, &consts);
            assert_eq!(in_memory, via_wire, "boundary changed execution on {rel}");

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
            "mir serial boundary: {} fixtures round-trip byte-exact vs oracle ({} returns, {} faults)",
            CORPUS.len(),
            rets,
            faults
        );
    });
}

#[test]
fn mir_collection_boundary_execution_equal_to_oracle() {
    on_big_stack(|| {
        let mut rets = 0usize;
        let mut faults = 0usize;
        for &(rel, shape) in COLLECTION_CORPUS {
            let src = read_fixture_lower(rel);
            let oracle = oracle_dump(&src);

            let (mp, items, consts) = lower_from_source(&src);

            // (1) wire round-trip idempotence: the new collection ops carry losslessly.
            let wire1 = serial::serialize(&mp);
            let mp2 = serial::deserialize(&wire1)
                .unwrap_or_else(|e| panic!("deserialize failed on {rel}: {e}"));
            let wire2 = serial::serialize(&mp2);
            assert_eq!(wire1, wire2, "wire not round-trip idempotent on {rel}");

            // (2) deserialized MIR runs byte-exact to the oracle (which runs collections).
            let via_wire = run_dump(&mp2, &items, &consts);
            assert_eq!(via_wire, oracle, "deserialized collection MIR diverges from oracle on {rel}");

            // (3) the boundary changed nothing: in-memory == deserialized == oracle.
            let in_memory = run_dump(&mp, &items, &consts);
            assert_eq!(in_memory, via_wire, "boundary changed execution on {rel}");

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
        assert!(rets > 0 && faults > 0, "collection corpus must exercise both returns and faults");
        eprintln!(
            "mir collection boundary: {} fixtures round-trip byte-exact vs oracle ({} returns, {} faults)",
            COLLECTION_CORPUS.len(),
            rets,
            faults
        );
    });
}
