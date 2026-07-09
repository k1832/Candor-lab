//! Structured-concurrency Stage-1 tests (design 0012): the unified spawn gate
//! (E12xx), scope-length loans (reusing the XOR scan's E08xx), the `split_mut`
//! blessed primitive, the sequential-oracle execution, spawn-order-first fault
//! delivery, and tree-walker == MIR engine equality on scalar concurrency
//! fixtures (extending the Stage-A gate set).

use candor_proto::interp::{Fault, FaultKind};
use candor_proto::{
    check_source_real, run_source_real, run_source_real_mir, MirRunResult, RunResult,
};

fn codes(src: &str) -> Vec<String> {
    check_source_real(src).expect("parse ok").into_iter().map(|d| d.code).collect()
}
fn assert_has(src: &str, code: &str) {
    let cs = codes(src);
    assert!(cs.iter().any(|c| c == code), "expected `{code}` for:\n{src}\ngot {cs:?}");
}
fn assert_clean(src: &str) {
    let cs = codes(src);
    assert!(cs.is_empty(), "expected clean for:\n{src}\ngot {cs:?}");
}
fn run_ret(src: &str) -> i64 {
    match run_source_real(src) {
        RunResult::Ok(r) => r.ret,
        RunResult::Fault(f) => panic!("faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!("check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        RunResult::ParseError(d) => panic!("parse error: {}", d.to_json()),
    }
}
fn run_fault(src: &str) -> Fault {
    match run_source_real(src) {
        RunResult::Fault(f) => f,
        RunResult::Ok(r) => panic!("expected fault, got ret {}", r.ret),
        RunResult::CheckErrors(d) => panic!("check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        RunResult::ParseError(d) => panic!("parse error: {}", d.to_json()),
    }
}

// ===========================================================================
// GATE NEGATIVES (§2.1)
// ===========================================================================

#[test]
fn gate_take_nonportable_rawptr() {
    // A `take` of a struct hiding a `rawptr` is rejected (E1202).
    assert_has(
        "struct Handle { p: rawptr u8 } fn w(h: Handle) -> unit { } \
         fn f(h: Handle) -> unit { scope { spawn w(h); } }",
        "E1202",
    );
}

#[test]
fn gate_alloc_read_copyout_race() {
    // THE review race: a `copy` struct hiding a `rawptr` crossed as a `read`
    // borrow could be copied out from behind the shared borrow (§2.3). Rejected
    // on the referent (E1203) — the laundering channel never opens.
    assert_has(
        "copy struct Handle { p: rawptr u8 } fn w(h: read Handle) -> unit { } \
         fn f(h: Handle) -> unit { scope { spawn w(read h); } }",
        "E1203",
    );
}

#[test]
fn gate_box_hidden_rawptr_referent() {
    // A `Box` whose pointee hides a `rawptr`, crossed as a shared borrow: the
    // walk descends through the Box and rejects (E1203).
    assert_has(
        "struct Node { p: rawptr u8 } fn w(b: read Box Node) -> unit { } \
         fn f(b: Box Node) -> unit { scope { spawn w(read b); } }",
        "E1203",
    );
}

#[test]
fn gate_opaque_t_take_unbounded() {
    // An unbounded generic `T` is not `portable`: a `take` across a spawn at the
    // def-site is rejected (E1202).
    assert_has(
        "fn sink[T](x: T) -> unit { } \
         fn caller[T](x: T) -> unit { scope { spawn sink(x); } }",
        "E1202",
    );
}

#[test]
fn gate_out_across_spawn_rejected() {
    assert_has(
        "fn w(o: out i64) -> unit { } \
         fn f() -> unit { let mut r: i64 = 0; scope { spawn w(out r); } }",
        "E1204",
    );
}

// ===========================================================================
// GATE POSITIVES (§2.1/§2.2)
// ===========================================================================

#[test]
fn gate_take_portable_ok() {
    assert_clean(
        "fn w(x: i64) -> unit { } fn f() -> unit { let v: i64 = 3; scope { spawn w(v); } }",
    );
}

#[test]
fn gate_shared_read_portable_referent_ok() {
    assert_clean(
        "fn r(s: read i64) -> unit { } \
         fn f() -> unit { let v: i64 = 3; scope { spawn r(read v); } }",
    );
}

#[test]
fn gate_fnptr_leaf_vtable_shared_read_ok() {
    // A fn-pointer is a portable LEAF (the walk does not descend its signature),
    // so sharing a vtable-bearing struct read-only across a spawn is legal.
    assert_clean(
        "struct VtHolder { f: fn() -> unit } fn use_vt(h: read VtHolder) -> unit { } \
         fn f(h: VtHolder) -> unit { scope { spawn use_vt(read h); } }",
    );
}

#[test]
fn gate_portable_bounded_generic_spawn_ok() {
    // A `[T: portable]` bound satisfies the referent gate at the def site; using
    // read-borrows keeps the body free of any owned-`T` drop (so no `alloc`).
    assert_clean(
        "fn sink[T](x: read T) -> unit { } \
         fn caller[T: portable](x: read T) -> unit { scope { spawn sink(x); } }",
    );
}

#[test]
fn gate_write_slot_disjoint_sibling_fields_ok() {
    // Distinct named fields are disjoint places (§3.1): two tasks writing sibling
    // slots do not conflict.
    assert_clean(
        "struct Pair { a: i64, b: i64 } fn wr(o: write i64) -> unit { } \
         fn f() -> unit { let mut p: Pair = Pair { a: 0, b: 0 }; \
           scope { spawn wr(write p.a); spawn wr(write p.b); } }",
    );
}

#[test]
fn gate_nested_scopes_ok() {
    assert_clean(
        "fn w(x: i64) -> unit { } \
         fn f() -> unit { let a: i64 = 1; let b: i64 = 2; \
           scope { spawn w(a); scope { spawn w(b); } } }",
    );
}

#[test]
fn split_mut_parallel_fill_ok() {
    // The flagship: a [u8] exclusively split into two disjoint halves, each
    // filled by a sibling spawn — accepted (§1.4/§2.4).
    assert_clean(
        "fn fill(s: write [u8]) -> unit { } \
         fn f() -> unit { let mut buf: [4]u8 = [0u8, 0u8, 0u8, 0u8]; \
           let lo: write [u8]; let hi: write [u8]; \
           split_mut(buf, 2, out lo, out hi); \
           scope { spawn fill(write lo); spawn fill(write hi); } }",
    );
}

// ===========================================================================
// SCOPE-LENGTH LOAN NEGATIVES (§1.2, reusing the XOR scan)
// ===========================================================================

#[test]
fn loan_parent_writes_read_borrowed_data() {
    assert_has(
        "fn r(s: read i64) -> unit { } \
         fn f() -> unit { let mut d: i64 = 0; scope { spawn r(read d); d = 5; } }",
        "E0803",
    );
}

#[test]
fn loan_two_spawns_write_same_place() {
    assert_has(
        "fn w(o: write i64) -> unit { } \
         fn f() -> unit { let mut d: i64 = 0; scope { spawn w(write d); spawn w(write d); } }",
        "E0801",
    );
}

#[test]
fn loan_third_access_to_split_mut_parent() {
    assert_has(
        "fn fill(s: write [u8]) -> unit { } \
         fn f() -> unit { let mut buf: [4]u8 = [0u8, 0u8, 0u8, 0u8]; \
           let lo: write [u8]; let hi: write [u8]; \
           split_mut(buf, 2, out lo, out hi); \
           let z: u8 = buf[0]; \
           scope { spawn fill(write lo); spawn fill(write hi); } }",
        "E0804",
    );
}

// ===========================================================================
// SPAWN-OUTSIDE-SCOPE and CONTRACT rejection
// ===========================================================================

#[test]
fn spawn_outside_scope_rejected() {
    assert_has(
        "fn w(x: i64) -> unit { } fn f() -> unit { spawn w(3); }",
        "E1201",
    );
}

// ===========================================================================
// EXECUTION (sequential oracle §6) + FAULT-AT-JOIN (§3.2)
// ===========================================================================

#[test]
fn exec_write_slot_result() {
    // The task writes its result through a `write`-borrowed slot; the parent reads
    // it after the join. Sequential oracle runs the task at the spawn point.
    assert_eq!(
        run_ret("fn setv(o: write i64) -> unit { o.* = 42; } \
                 fn main() -> i64 { let mut r: i64 = 0; scope { spawn setv(write r); } return r; }"),
        42,
    );
}

#[test]
fn exec_alloc_crosses_spawn() {
    // A spawned `alloc`-callee makes the enclosing function `alloc` (§2.5): the
    // non-`alloc` container is rejected (E0401).
    assert_has(
        "fn mk(x: i64) alloc -> unit { } \
         fn f() -> unit { scope { spawn mk(3); } }",
        "E0401",
    );
}

#[test]
fn fault_at_join_spawn_order_first() {
    // Two faulting tasks: the spawn-order-least task's fault is delivered (§3.2).
    // In the sequential schedule this is naturally the first fault encountered.
    let f = run_fault(
        "fn boom_div() -> unit { let z: i64 = 0; let q: i64 = 1 / z; } \
         fn boom_assert() -> unit { assert(false); } \
         fn main() -> i64 { scope { spawn boom_div(); spawn boom_assert(); } return 0; }",
    );
    assert_eq!(f.kind, FaultKind::DivByZero, "spawn-order-first: task A (div0) wins over B (assert)");
}

// ===========================================================================
// ENGINE EQUALITY: tree-walker == MIR on scalar concurrency fixtures
// ===========================================================================

fn assert_engines_equal(src: &str) {
    let oracle = match run_source_real(src) {
        RunResult::Ok(r) => Ok(r.ret),
        RunResult::Fault(f) => Err(format!("{:?}@{}", f.kind, f.span.start)),
        RunResult::CheckErrors(d) => panic!("oracle check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        RunResult::ParseError(d) => panic!("oracle parse error: {}", d.to_json()),
    };
    let m = match run_source_real_mir(src) {
        MirRunResult::Ok(r) => Ok(r.ret),
        MirRunResult::Fault(f) => Err(format!("{:?}@{}", f.kind, f.span.start)),
        MirRunResult::Unsupported(u) => panic!("expected in-subset, got Unsupported: {u}"),
        MirRunResult::CheckErrors(d) => panic!("MIR check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        MirRunResult::ParseError(d) => panic!("MIR parse error: {}", d.to_json()),
    };
    assert_eq!(m, oracle, "engine divergence for:\n{src}");
}

#[test]
fn engine_equality_fork_join_owned() {
    assert_engines_equal(
        "fn setv(o: write i64) -> unit { o.* = 7; } \
         fn main() -> i64 { let mut r: i64 = 0; scope { spawn setv(write r); } return r; }",
    );
}

#[test]
fn engine_equality_shared_read_and_nested() {
    assert_engines_equal(
        "fn setv(o: write i64) -> unit { o.* = 5; } \
         fn addv(o: write i64) -> unit { o.* = 9; } \
         fn main() -> i64 { let mut a: i64 = 0; let mut b: i64 = 0; \
           scope { spawn setv(write a); scope { spawn addv(write b); } } return a + b; }",
    );
}

#[test]
fn engine_equality_fault_at_join() {
    assert_engines_equal(
        "fn boom() -> unit { let z: i64 = 0; let q: i64 = 1 / z; } \
         fn main() -> i64 { scope { spawn boom(); } return 0; }",
    );
}
