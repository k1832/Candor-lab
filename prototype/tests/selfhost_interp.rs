//! The oracle gate for the FIRST self-interpreting slice: a Candor SCALAR
//! interpreter (`selfhost/interp/interp.cnr`, loaded as a module tree with the
//! `lexer` and `parser` modules) executes each in-subset fixture program. Its
//! canonical execution dump -- `RET <i64>` then a `TRACE <i64>` line per traced
//! value, OR a single `FAULT <kindcode> <p0> <p1>` -- is asserted byte-equal to
//! the Rust reference interpreter's result for the SAME source, rendered in the
//! identical schema. Passing this gate is EXECUTION equality (return value, trace
//! sequence, and fault identity: kind + span) between the two engines.
//!
//! Harness shape reuses the checker slice: a generated root `main.cnr` `use`s the
//! `lexer`/`interp` modules, embeds the fixture source as a `[N]u8` literal, lexes
//! then interp-dumps it; the tree is loaded with `run_dir` (dogfooding the module
//! system) and the dump reconstructed from `Run.trace`, compared to the oracle
//! rendering from `run_source_real`.

use candor_proto::interp::FaultKind;
use candor_proto::{run_source_real, RunResult};

mod selfhost_modtree;
use selfhost_modtree::{run_module_tree, trace_text};

const LEXER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/lexer/lexer.cnr"));
const PARSER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/parser/parser.cnr"));
const INTERP_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/interp/interp.cnr"));

/// Integer kind-code map shared with the Candor interpreter's `FK_*` statics. Only
/// the four faults reachable in the scalar MVP subset appear here.
fn fault_code(k: FaultKind) -> i64 {
    match k {
        FaultKind::Overflow => 0,
        FaultKind::DivByZero => 1,
        FaultKind::Assert => 2,
        FaultKind::Panic => 3,
        FaultKind::Bounds => 4,
        other => panic!("out-of-subset fault kind reached the gate: {other:?}"),
    }
}

/// Render the Rust reference interpreter's result in the canonical dump schema.
fn oracle_dump(src: &str) -> String {
    match run_source_real(src) {
        RunResult::Ok(run) => {
            let mut s = format!("RET {}\n", run.ret);
            for v in &run.trace {
                s.push_str(&format!("TRACE {v}\n"));
            }
            s
        }
        RunResult::Fault(f) => {
            format!("FAULT {} {} {}\n", fault_code(f.kind), f.span.start, f.span.end)
        }
        RunResult::CheckErrors(d) => panic!(
            "fixture has check errors (out of subset?): {:?}",
            d.iter().map(|x| &x.code).collect::<Vec<_>>()
        ),
        RunResult::ParseError(d) => panic!("fixture parse error: {}", d.to_json()),
    }
}

/// Generate the root `main.cnr`: it `use`s the lexer's `Buf`/`mk`/`lex` and the
/// interp's `interp_dump`, embeds `src`, lexes then interp-dumps it.
fn candor_main(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut m = String::from(
        "use lexer::{Buf, mk, lex};\nuse interp::{interp_dump};\n\nfn main() -> i64 {\n",
    );
    m.push_str(&format!("    let src: [{}]u8 = [", bytes.len()));
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            m.push_str(", ");
        }
        m.push_str(&format!("{b}u8"));
    }
    m.push_str("];\n");
    m.push_str("    let mut buf: Buf = Buf { toks: [mk(0, 0usize, 0usize); 32768], n: 0usize };\n");
    m.push_str("    let cnt: usize = lex(slice_of(src), write buf);\n");
    m.push_str("    interp_dump(slice_of(src), read buf);\n");
    m.push_str("    return conv i64 cnt;\n}\n");
    m
}

fn candor_dump(src: &str) -> String {
    let main = candor_main(src);
    let modules = [
        ("lexer.cnr", LEXER_SRC),
        ("parser.cnr", PARSER_SRC),
        ("interp.cnr", INTERP_SRC),
    ];
    match run_module_tree(&modules, &main) {
        RunResult::Ok(run) => trace_text(&run),
        RunResult::Fault(f) => panic!("candor interp (the .cnr program) faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!(
            "candor interp (the .cnr program) has check errors: {:?}",
            d.iter().map(|x| &x.code).collect::<Vec<_>>()
        ),
        RunResult::ParseError(d) => panic!("candor interp parse error: {}", d.to_json()),
    }
}

/// (fixture, expected dump shape) — `Ret` for a value/trace result, `Fault` for a
/// runtime fault. A redundant, human-auditable check that the gate is exercising
/// real returns and real faults, not just matching two coincidentally-equal texts.
#[derive(Clone, Copy, PartialEq)]
enum Shape {
    Ret,
    Fault,
}
use Shape::*;

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
    // ---- S2: structs + arrays over the flat byte-memory model ----
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
    // ---- S3: MOVE/DROP schedule with trace-on-drop ----
    ("drop_single.cnr", Ret),
    ("drop_scope_order.cnr", Ret),
    ("drop_move_suppress.cnr", Ret),
    ("drop_partial_move.cnr", Ret),
    ("drop_move_return.cnr", Ret),
    ("drop_break.cnr", Ret),
    ("drop_nested.cnr", Ret),
    ("drop_param.cnr", Ret),
    // ---- S4: ENUMS + MATCH ----
    ("enum_construct_match.cnr", Ret),
    ("match_wildcard.cnr", Ret),
    ("enum_multi_variant.cnr", Ret),
    ("match_bind_multi.cnr", Ret),
    ("enum_result_shape.cnr", Ret),
    ("enum_drop_payload.cnr", Ret),
    // ---- S5a: ALLOCATOR-ABI FOUNDATION (rawptr/fnptr scalars, statics,
    // fn-name-as-value, indirect calls, structural Alloc, minimal rawptr surface) ----
    ("static_fnptr_indirect_call.cnr", Ret),
    ("ptr_roundtrip.cnr", Ret),
    ("cast_ptr_read.cnr", Ret),
    ("alloc_abi.cnr", Ret),
    // ---- S5b: BOX / BoxResult / unbox / Box-deref + alloc-on-drop ----
    ("box_unbox_scalar.cnr", Ret),
    ("box_struct.cnr", Ret),
    ("unbox_path.cnr", Ret),
    ("boxresult_oom.cnr", Ret),
    ("box_drop_frees.cnr", Ret),
    ("nested_box.cnr", Ret),
    // ---- S6a: PAGED memory model + pointer intrinsics (offsetof / ptr_offset /
    // ptr_to_addr). Infrastructure for the systems corpus (the corpus is S6b). ----
    ("high_addr_roundtrip.cnr", Ret),
    ("offsetof_first_field.cnr", Ret),
    ("offsetof_nonzero_field.cnr", Ret),
    ("ptr_offset_stride.cnr", Ret),
    ("enum_padding_copy.cnr", Ret),
    ("page_boundary.cnr", Ret),
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
fn candor_interp_execution_equal_to_oracle_over_corpus() {
    on_big_stack(|| {
        let mut passed = 0usize;
        let mut faults = 0usize;
        let mut rets = 0usize;
        for &(rel, shape) in CORPUS {
            let src = read_fixture(rel);
            let oracle = oracle_dump(&src);
            let mine = candor_dump(&src);
            assert_eq!(mine, oracle, "execution mismatch on {rel}");
            match shape {
                Ret => {
                    assert!(oracle.starts_with("RET "), "expected a RET result on {rel}, got {oracle:?}");
                    rets += 1;
                }
                Fault => {
                    assert!(oracle.starts_with("FAULT "), "expected a FAULT result on {rel}, got {oracle:?}");
                    faults += 1;
                }
            }
            passed += 1;
        }
        assert_eq!(passed, CORPUS.len());
        assert!(rets > 0 && faults > 0, "corpus must exercise both returns and faults");
        eprintln!(
            "selfhost interp: EXECUTION PARITY on {}/{} fixtures ({} returns, {} faults) byte-equal to the Rust oracle",
            passed,
            CORPUS.len(),
            rets,
            faults
        );
    });
}
