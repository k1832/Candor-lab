//! Native collection drop-free gate (closes the native-collections arc): native
//! `emit_drop` must FREE a String/Vec/Map buffer (and drop its live elements /
//! values, and free a Map's owned keys) at scope end, mirroring the interpreter's
//! `alloc_on_drop`. Each fixture is proven byte-identical under the tree-walking
//! oracle, the MIR interpreter, and the Cranelift native backend (no-opt + opt);
//! the LLVM `clang -O2` engine covers the same fixtures transitively through
//! `tests/llvm.rs`'s full-corpus fifth-engine gate (they live in `fixtures/run/`).

use candor_proto::interp::Run;
use candor_proto::{
    run_source_real, run_source_real_mir, run_source_real_native, run_source_real_native_opt,
    MirRunResult, RunResult,
};

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/run/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn oracle(src: &str) -> Run {
    match run_source_real(src) {
        RunResult::Ok(r) => r,
        RunResult::Fault(f) => panic!("oracle faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => {
            panic!("oracle check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("oracle parse error: {}", d.to_json()),
    }
}

fn mir_run(r: MirRunResult, label: &str) -> Run {
    match r {
        MirRunResult::Ok(run) => run,
        MirRunResult::Fault(f) => panic!("{label} faulted: {}", f.to_json()),
        MirRunResult::Unsupported(m) => panic!("{label} unsupported: {m}"),
        MirRunResult::CheckErrors(d) => {
            panic!("{label} check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        MirRunResult::ParseError(d) => panic!("{label} parse error: {}", d.to_json()),
    }
}

/// Assert every non-LLVM engine reproduces the oracle's `(ret, trace)` byte-exact,
/// and that the oracle itself matches `expected_ret` / `expected_trace`.
fn gate(name: &str, expected_ret: i64, expected_trace: &[i64]) {
    let src = fixture(name);
    let o = oracle(&src);
    assert_eq!(o.ret, expected_ret, "{name}: oracle ret");
    assert_eq!(o.trace, expected_trace, "{name}: oracle trace");
    for (label, r) in [
        ("mir", run_source_real_mir(&src)),
        ("native-noopt", run_source_real_native(&src)),
        ("native-opt", run_source_real_native_opt(&src)),
    ] {
        let run = mir_run(r, label);
        assert_eq!(run.ret, o.ret, "{name}: {label} ret diverged from oracle");
        assert_eq!(run.trace, o.trace, "{name}: {label} trace diverged from oracle");
    }
}

// A grown String/Vec/Map dropped at scope end returns the counting allocator to
// balance (every buffer + Map key freed exactly once).
#[test]
fn collection_drop_frees_buffers_all_engines() {
    gate("coll_drop_balance.cnr", 0, &[]);
}

// Vec[Box]/Map[Box]: the element/value drop frees each inner Box before the
// buffer is freed — the allocator returns to balance.
#[test]
fn collection_drop_frees_box_elements_all_engines() {
    gate("coll_drop_box_elems.cnr", 0, &[]);
}

// A `new`'d collection that never allocates a buffer (buf == 0) drops cleanly
// with no free (the buf!=0 guard) — no fault, no double free.
#[test]
fn empty_collection_drops_without_free_all_engines() {
    gate("coll_drop_empty.cnr", 0, &[]);
}

// A collection moved into a callee is dropped once by the callee; the caller-side
// drop is suppressed by the move mask (a double free would drive `live` negative).
#[test]
fn moved_out_collection_not_double_freed_all_engines() {
    gate("coll_drop_moved.cnr", 0, &[]);
}

// A reclaiming free-list with a 2048-byte window services 200 build-and-drop
// cycles only because each dropped Vec buffer is returned and reused.
#[test]
fn collection_buffer_reused_after_drop_all_engines() {
    gate("coll_drop_reuse.cnr", 21700, &[]);
}

// A drop-hooked Vec element traces its id; the Vec drop must run them in reverse
// index order (5,4,3,2,1), identical across engines.
#[test]
fn vec_element_drop_order_all_engines() {
    gate("coll_drop_order_vec.cnr", 0, &[5, 4, 3, 2, 1]);
}

// A drop-hooked Map value traces its id; the Map drop must run every occupied
// slot's value (slot order), identical across engines. The order is hash-derived,
// so it is asserted against the oracle's observed order rather than sorted away.
#[test]
fn map_value_drop_order_all_engines() {
    gate("coll_drop_order_map.cnr", 0, &[33, 44, 11, 22]);
}
