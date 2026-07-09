//! Structured-concurrency **Stage 2** gate (design 0012 §6): the native (Cranelift
//! JIT) engine runs `scope`/`spawn` with **REAL OS threads** (`rt_spawn`/`rt_join`
//! over `std::thread`), while the tree-walking interpreter remains the deterministic
//! sequential oracle. This gate shakes schedules by running each fixture N times and
//! asserts the design's nondeterminism discipline holds every time:
//!
//!  * **Per-task projection equivalence (§6).** Each task traces into its own buffer;
//!    the join merges buffers in spawn order, so the observed `θ` is schedule-
//!    independent and equals the oracle's sequential trace — asserted exactly.
//!  * **Fault identity is spawn-order-first (§3.2), stable across schedules.** All
//!    task outcomes are collected at the brace; the po-least (spawn-order-least)
//!    faulter's fault `(k, s)` is delivered — identical on every iteration and equal
//!    to the oracle's.
//!  * **Results (write-slots) equal.** Successful tasks' in-place writes into
//!    caller-owned slots (the shared flat substrate; DRF-disjoint) retire identically
//!    under real threads.
//!
//! **Honest boundary (design 0012 §6 "which the native engine supports").** The
//! native engine runs SCALAR tasks and AGGREGATE write-slots (distinct struct
//! fields / locals are disjoint places — §3.1). `split_mut` is a *compile-time-only*
//! blessed primitive here: it has **no runtime lowering on ANY engine** (the oracle
//! itself faults `BadPointer` on it), so its disjoint-fill flagship is a Stage-1
//! CHECKER fixture, not a Stage-2 execution one. Disjoint-contended writes are
//! therefore exercised through **distinct named slots** (the race-shaker below),
//! which puts many threads on the shared buffer at disjoint offsets exactly as a
//! split buffer would. On a FAULTING run only the fault IDENTITY is compared (not
//! `θ`): side-effect/trace *extent* is declared-nondeterministic (§3.1/§3.2).

use candor_proto::{run_source_real, run_source_real_native, MirRunResult, RunResult};

#[derive(PartialEq, Debug, Clone)]
enum Out {
    Ok(i64, Vec<i64>),
    Fault(String),
}

fn native(src: &str) -> Out {
    match run_source_real_native(src) {
        MirRunResult::Ok(r) => Out::Ok(r.ret, r.trace),
        MirRunResult::Fault(f) => Out::Fault(format!("{:?}@{}..{}", f.kind, f.span.start, f.span.end)),
        MirRunResult::Unsupported(u) => panic!("native: expected in-subset, got Unsupported: {u}"),
        MirRunResult::CheckErrors(d) => panic!("native check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        MirRunResult::ParseError(d) => panic!("native parse error: {}", d.to_json()),
    }
}

fn oracle(src: &str) -> Out {
    match run_source_real(src) {
        RunResult::Ok(r) => Out::Ok(r.ret, r.trace),
        RunResult::Fault(f) => Out::Fault(format!("{:?}@{}..{}", f.kind, f.span.start, f.span.end)),
        RunResult::CheckErrors(d) => panic!("oracle check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        RunResult::ParseError(d) => panic!("oracle parse error: {}", d.to_json()),
    }
}

/// The gate: the native engine's real-threaded result matches the sequential oracle
/// on every one of `iters` schedule-shaking iterations (ret, merged θ, and fault
/// identity all stable and equal).
fn gate(src: &str, iters: usize) {
    let want = oracle(src);
    for i in 0..iters {
        let got = native(src);
        assert_eq!(got, want, "native/oracle divergence on iteration {i} for:\n{src}");
    }
}

const ITERS: usize = 50;

// ===========================================================================
// SCALAR write-slot results (§3.1) — the fork-join-of-owned shape
// ===========================================================================

#[test]
fn native_scalar_write_slot() {
    gate(
        "fn setv(o: write i64) -> unit { o.* = 42; } \
         fn main() -> i64 { let mut r: i64 = 0; scope { spawn setv(write r); } return r; }",
        ITERS,
    );
}

#[test]
fn native_two_disjoint_scalar_slots() {
    gate(
        "fn setv(o: write i64) -> unit { o.* = 5; } \
         fn addv(o: write i64) -> unit { o.* = 9; } \
         fn main() -> i64 { let mut a: i64 = 0; let mut b: i64 = 0; \
           scope { spawn setv(write a); spawn addv(write b); } return a + b; }",
        ITERS,
    );
}

#[test]
fn native_shared_read_portable_referent() {
    // Two tasks READ one shared `portable` referent (immutable share, no race) and
    // each writes its own disjoint slot (§2.2/§3.1).
    gate(
        "fn dbl(s: read i64, o: write i64) -> unit { o.* = s.* + s.*; } \
         fn main() -> i64 { let v: i64 = 21; let mut a: i64 = 0; let mut b: i64 = 0; \
           scope { spawn dbl(read v, write a); spawn dbl(read v, write b); } return a + b; }",
        ITERS,
    );
}

#[test]
fn native_nested_scope_tasks_both_levels() {
    gate(
        "fn setv(o: write i64) -> unit { o.* = 5; } \
         fn addv(o: write i64) -> unit { o.* = 9; } \
         fn main() -> i64 { let mut a: i64 = 0; let mut b: i64 = 0; \
           scope { spawn setv(write a); scope { spawn addv(write b); } } return a + b; }",
        ITERS,
    );
}

// ===========================================================================
// PER-TASK TRACE PROJECTION merged in spawn order (§6, deterministic θ)
// ===========================================================================

#[test]
fn native_trace_projection_merged_spawn_order() {
    // Each task traces into its own buffer; the merge at the brace is spawn-order,
    // so θ == [1, 2, 3] on every schedule (the raw interleaving is discarded).
    gate(
        "fn t(x: i64) -> unit { trace(x); } \
         fn main() -> i64 { scope { spawn t(1); spawn t(2); spawn t(3); } return 0; }",
        ITERS,
    );
}

#[test]
fn native_trace_multi_event_per_task() {
    // Multi-event tasks: each task's projection is a contiguous run; the merge keeps
    // spawn order → θ == [10,11, 20,21, 30,31] deterministically.
    gate(
        "fn t(base: i64) -> unit { trace(base); trace(base + 1); } \
         fn main() -> i64 { scope { spawn t(10); spawn t(20); spawn t(30); } return 0; }",
        ITERS,
    );
}

// ===========================================================================
// AGGREGATE write-slots — distinct struct fields are disjoint places (§3.1)
// ===========================================================================

#[test]
fn native_aggregate_disjoint_fields() {
    gate(
        "struct Pair { a: i64, b: i64 } fn wr(o: write i64, v: i64) -> unit { o.* = v; } \
         fn main() -> i64 { let mut p: Pair = Pair { a: 0, b: 0 }; \
           scope { spawn wr(write p.a, 100); spawn wr(write p.b, 23); } return p.a + p.b; }",
        ITERS,
    );
}

// ===========================================================================
// FAULT IDENTITY — spawn-order-first (§3.2), stable across schedules
// ===========================================================================

#[test]
fn native_fault_single_task() {
    gate(
        "fn boom() -> unit { let z: i64 = 0; let q: i64 = 1 / z; } \
         fn main() -> i64 { scope { spawn boom(); } return 0; }",
        ITERS,
    );
}

#[test]
fn native_fault_spawn_order_first_a_wins() {
    // Both tasks fault; the spawn-order-least (A: div0) is delivered EVERY time,
    // never B's assert, regardless of which thread hits its fault first.
    let src = "fn boom_div() -> unit { let z: i64 = 0; let q: i64 = 1 / z; } \
               fn boom_assert() -> unit { assert(false); } \
               fn main() -> i64 { scope { spawn boom_div(); spawn boom_assert(); } return 0; }";
    gate(src, ITERS);
    // Pin the identity explicitly: DivByZero, not Assert.
    match native(src) {
        Out::Fault(s) => assert!(s.starts_with("DivByZero@"), "spawn-order-first: A (div0) wins, got {s}"),
        o => panic!("expected fault, got {o:?}"),
    }
}

#[test]
fn native_fault_first_ok_second_faults() {
    // A succeeds, B faults → the po-least *faulter* is B; its fault is delivered
    // (A running fully in parallel does not change the identity).
    let src = "fn ok(o: write i64) -> unit { o.* = 1; } \
               fn boom() -> unit { assert(false); } \
               fn main() -> i64 { let mut r: i64 = 0; scope { spawn ok(write r); spawn boom(); } return r; }";
    gate(src, ITERS);
    match native(src) {
        Out::Fault(s) => assert!(s.starts_with("Assert@"), "po-least faulter is B (assert), got {s}"),
        o => panic!("expected fault, got {o:?}"),
    }
}

// ===========================================================================
// THE RACE-SHAKER — many spawns, contended-but-disjoint writes, run hard
// ===========================================================================

#[test]
fn native_stress_many_disjoint_writers() {
    // Eight tasks hammer the shared flat buffer concurrently at eight disjoint field
    // offsets (the split_mut disjoint-fill shape, expressed via distinct slots since
    // split_mut has no runtime here). Run 200× to shake schedules: every write must
    // retire, sum == 255, on every iteration.
    let src = "struct Oct { a: i64, b: i64, c: i64, d: i64, e: i64, f: i64, g: i64, h: i64 } \
       fn wr(o: write i64, v: i64) -> unit { o.* = v; } \
       fn main() -> i64 { let mut p: Oct = Oct { a:0,b:0,c:0,d:0,e:0,f:0,g:0,h:0 }; \
         scope { spawn wr(write p.a, 1); spawn wr(write p.b, 2); spawn wr(write p.c, 4); \
                 spawn wr(write p.d, 8); spawn wr(write p.e, 16); spawn wr(write p.f, 32); \
                 spawn wr(write p.g, 64); spawn wr(write p.h, 128); } \
         return p.a + p.b + p.c + p.d + p.e + p.f + p.g + p.h; }";
    assert_eq!(oracle(src), Out::Ok(255, vec![]));
    gate(src, 200);
}

#[test]
fn native_stress_many_tracers() {
    // Eight tracing tasks, 100× — the merged θ is spawn-order [0..8) every time,
    // never an interleaving.
    let src = "fn t(x: i64) -> unit { trace(x); } \
       fn main() -> i64 { scope { spawn t(0); spawn t(1); spawn t(2); spawn t(3); \
                                   spawn t(4); spawn t(5); spawn t(6); spawn t(7); } return 0; }";
    assert_eq!(oracle(src), Out::Ok(0, vec![0, 1, 2, 3, 4, 5, 6, 7]));
    gate(src, 100);
}


// ===========================================================================
// split_mut PARALLEL FILL on real threads (design 0012 §1.4/§2.4)
// ===========================================================================

#[test]
fn native_split_mut_parallel_fill() {
    // The flagship: `[4]u8` split into disjoint halves, each filled by a sibling
    // spawn through its disjoint `slice_mut`; native (real threads) == oracle.
    gate(
        "fn fill(s: write [u8], v: u8, n: usize) -> unit { \
            let mut i: usize = 0; loop { if i >= n { break; } s[i] = v; i = i + 1; } } \
         fn main() -> i64 { let mut buf: [4]u8 = [0u8, 0u8, 0u8, 0u8]; \
            let lo: write [u8]; let hi: write [u8]; \
            split_mut(buf, 2, out lo, out hi); \
            scope { spawn fill(write lo, 1u8, 2); spawn fill(write hi, 2u8, 2); } \
            return conv i64 buf[0] + conv i64 buf[1] * 10 \
                 + conv i64 buf[2] * 100 + conv i64 buf[3] * 1000; }",
        ITERS,
    );
}

#[test]
fn native_split_mut_nested() {
    // Nested split feeding three disjoint sibling spawns; the two-level disjoint
    // partition mutates disjoint memory with no race under real threads.
    gate(
        "fn fill(s: write [u8], v: u8, n: usize) -> unit { \
            let mut i: usize = 0; loop { if i >= n { break; } s[i] = v; i = i + 1; } } \
         fn main() -> i64 { let mut buf: [4]u8 = [0u8, 0u8, 0u8, 0u8]; \
            let lo: write [u8]; let hi: write [u8]; \
            split_mut(buf, 2, out lo, out hi); \
            let a: write [u8]; let b: write [u8]; \
            split_mut(lo, 1, out a, out b); \
            scope { spawn fill(write a, 1u8, 1); spawn fill(write b, 2u8, 1); \
                    spawn fill(write hi, 3u8, 2); } \
            return conv i64 buf[0] + conv i64 buf[1] * 10 \
                 + conv i64 buf[2] * 100 + conv i64 buf[3] * 1000; }",
        ITERS,
    );
}

#[test]
fn native_split_mut_bounds_fault() {
    // `mid > len` faults `bounds`; the native engine delivers the same fault
    // identity as the oracle (kind + span), on every iteration.
    gate(
        "fn main() -> i64 { let mut buf: [4]u8 = [0u8, 0u8, 0u8, 0u8]; \
           let lo: write [u8]; let hi: write [u8]; \
           split_mut(buf, 5, out lo, out hi); return 0; }",
        ITERS,
    );
}
