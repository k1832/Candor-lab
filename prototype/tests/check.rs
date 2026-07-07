//! Stage 2 checker tests: negative snippets asserting a specific diagnostic
//! code, and positive snippets asserting a clean check.

use candor_proto::check_source;

fn codes(src: &str) -> Vec<String> {
    let diags = check_source(src).expect("parse ok");
    diags.into_iter().map(|d| d.code).collect()
}

fn assert_has(src: &str, code: &str) {
    let cs = codes(src);
    assert!(
        cs.iter().any(|c| c == code),
        "expected `{code}` for:\n{src}\ngot {cs:?}"
    );
}

fn assert_clean(src: &str) {
    let cs = codes(src);
    assert!(cs.is_empty(), "expected clean for:\n{src}\ngot {cs:?}");
}

// ----- negative -----------------------------------------------------------

#[test]
fn use_after_move() {
    assert_has(
        "fn use2(b: Box i64) -> unit { } fn f(b: Box i64) -> unit { use2(b); use2(b); }",
        "E0301",
    );
}

#[test]
fn move_state_join_disagreement() {
    assert_has(
        "fn use1(b: Box i64) -> unit { } fn f(b: Box i64, c: bool) -> unit { if c { use1(b); } }",
        "E0302",
    );
}

#[test]
fn partial_move_of_drop_hooked() {
    assert_has(
        "struct H { x: Box i64 } drop(write self) { } fn t(b: Box i64) -> unit { } \
         fn f(h: H) -> unit { t(h.x); }",
        "E0303",
    );
}

#[test]
fn read_before_init() {
    assert_has("fn f() -> i64 { let x: i64; return x; }", "E0304");
}

#[test]
fn out_not_assigned_on_return_path() {
    assert_has("fn f(x: out i64) -> unit { }", "E0305");
    assert_has(
        "fn f(x: out i64, c: bool) -> unit { if c { x = 1; } }",
        "E0305",
    );
}

#[test]
fn out_read_before_first_assignment() {
    assert_has("fn f(x: out i64) -> unit { let y: i64 = x; x = 0; }", "E0306");
}

#[test]
fn non_alloc_calls_alloc() {
    assert_has("fn a() alloc -> unit { } fn b() -> unit { a(); }", "E0401");
}

#[test]
fn clone_of_box_bearing_requires_alloc() {
    // Non-alloc function performing a box-bearing clone: E0401.
    assert_has(
        "enum Tree { leaf(i64), node(Box Tree) } \
         fn dup(t: read Tree) -> Tree { return clone (deref t); }",
        "E0401",
    );
}

#[test]
fn alloc_fn_assigned_to_non_alloc_fn_ptr() {
    assert_has("fn a() alloc -> unit { } static P: fn() -> unit = a;", "E0402");
}

#[test]
fn borrow_typed_struct_field() {
    assert_has("struct Bad { r: borrow i64 }", "E0201");
}

#[test]
fn slice_typed_struct_field() {
    assert_has("struct Bad { s: slice u8 }", "E0201");
}

#[test]
fn rawptr_op_outside_unsafe() {
    assert_has("fn f(p: rawptr i64) -> i64 { return ptr_read(p); }", "E0501");
}

#[test]
fn copy_marker_on_non_copy_field() {
    assert_has("copy struct Bad { b: Box i64 }", "E0202");
}

#[test]
fn non_exhaustive_match() {
    assert_has(
        "enum E { a, b } fn f(e: E) -> unit { match e { case E::a => panic(\"x\") } }",
        "E0601",
    );
}

#[test]
fn move_binding_from_borrowed_scrutinee() {
    // Design §8.2.1 (Stage 3): a payload bound from a `read` scrutinee is a
    // shared *borrow* of the sub-place (`borrow Box i64`), not an owned `Box`.
    // Passing it to a by-value `Box i64` parameter is therefore a type error
    // (the old conservative E0602 "non-movable" rule is gone).
    assert_has(
        "enum E { v(Box i64) } fn takeb(b: Box i64) -> unit { } \
         fn f(e: read E) -> unit { match e { case E::v(b) => takeb(b) } }",
        "E0703",
    );
}

#[test]
fn conv_on_non_integer() {
    assert_has("fn f() -> i32 { return conv i32 (true); }", "E0701");
}

#[test]
fn modes_on_borrow_kind_param() {
    assert_has("fn f(s: read slice u8) -> unit { }", "E0203");
}

#[test]
fn result_outside_ensures() {
    assert_has("fn f() -> i64 { return result; }", "E0702");
}

// ----- positive -----------------------------------------------------------

#[test]
fn copy_semantics_use_after_copy() {
    assert_clean(
        "copy struct P { x: i64 } fn use1(p: P) -> unit { } \
         fn f(p: P) -> unit { use1(p); use1(p); }",
    );
}

#[test]
fn partial_move_then_reassign_then_use() {
    // `t` receives a `Box` by value and lets it die (free), and `f` frees `s.b`
    // (never moved out) at scope exit -- both are allocator-effecting, so both
    // carry the `alloc` marker (finding 4 of 2026-07-07; §6.2/§6.3). The partial
    // move / reassign / re-use pattern itself stays accepted.
    assert_clean(
        "struct S { a: Box i64, b: Box i64 } fn t(x: Box i64) alloc -> unit { } \
         fn f(s: S, n: Box i64) alloc -> unit { t(s.a); s.a = n; t(s.a); }",
    );
}

#[test]
fn loop_reassignment_of_out_slot() {
    assert_clean("fn f(x: out i64) -> unit { loop { x = 0; break; } }");
}

#[test]
fn clone_of_box_bearing_propagates_alloc() {
    assert_clean(
        "enum Tree { leaf(i64), node(Box Tree) } \
         fn dup(t: read Tree) alloc -> Tree { return clone (deref t); }",
    );
}

#[test]
fn result_inside_ensures_ok() {
    assert_clean("fn f() ensures(result == 0) -> i64 { return 0; }");
}

// ----- loop definite-assignment / move regression (dataflow fixpoint fix) -----
// A forward must-analysis must join only over *computed* predecessors: a loop
// back-edge that is still pessimistically `Uninit` on the first pass must not
// degrade a value initialized before the loop to possibly-uninitialized.

#[test]
fn while_reads_loop_invariant_local_ok() {
    // (a) `x` is initialized before the loop and read in the body.
    assert_clean(
        "fn main() -> i64 { let x: i64 = 5; let mut i: i64 = 0; \
         while i < 3 { i = i + x; } return i; }",
    );
}

#[test]
fn loop_with_break_reads_invariant_ok() {
    // (b) same, using `loop { ... break; }`.
    assert_clean(
        "fn main() -> i64 { let x: i64 = 5; let mut i: i64 = 0; \
         loop { i = i + x; if i >= 3 { break; } } return i; }",
    );
}

#[test]
fn loop_reads_conditionally_initialized_still_uninit() {
    // (c) `x` is initialized only on the `if` branch before the loop: a genuine
    // uninit path remains, so the read in the body is still E0304 (the fix must
    // not paper over a real definite-assignment hole).
    assert_has(
        "fn f(c: bool) -> i64 { let x: i64; if c { x = 5; } let mut i: i64 = 0; \
         while i < 3 { i = i + x; } return i; }",
        "E0304",
    );
}

#[test]
fn value_moved_in_loop_body_used_next_iteration_still_error() {
    // (d) a non-copy value moved inside the body is used again on the next
    // iteration: the back-edge join sees moved-vs-live and the use-after-move
    // fires. Loop-carried move errors must still be caught.
    assert_has(
        "fn sink(b: Box i64) -> unit { } \
         fn f(b: Box i64) -> unit { while true { sink(b); } }",
        "E0301",
    );
}

#[test]
fn value_moved_in_loop_used_after_loop_still_error() {
    // (d, variant) moved inside the body, then used after the loop.
    assert_has(
        "fn sink(b: Box i64) -> unit { } \
         fn f(b: Box i64) -> unit { let mut i: i64 = 0; \
         while i < 1 { sink(b); i = i + 1; } sink(b); }",
        "E0301",
    );
}

#[test]
fn loop_accumulator_then_returned_ok() {
    // (e) `let mut` accumulated across iterations and returned.
    assert_clean(
        "fn main() -> i64 { let mut acc: i64 = 0; let mut i: i64 = 0; \
         while i < 5 { acc = acc + i; i = i + 1; } return acc; }",
    );
}

#[test]
fn nested_loops_read_outer_invariant_ok() {
    // (f) the inner loop reads a value initialized before the outer loop.
    assert_clean(
        "fn main() -> i64 { let x: i64 = 5; let mut i: i64 = 0; \
         while i < 3 { let mut j: i64 = 0; while j < 2 { i = i + x; j = j + 1; } } \
         return i; }",
    );
}

// ----- C1: unsafe justification must be non-empty (§4.1) -------------------

#[test]
fn unsafe_empty_justification_rejected() {
    assert_has(
        "fn f() -> i64 { unsafe \"\" { let p: rawptr i64 = ptr_null[i64](); } return 0; }",
        "E0502",
    );
}

#[test]
fn unsafe_whitespace_justification_accepted() {
    assert_clean(
        "fn f() -> i64 { let x: i64 = 5; unsafe \"   \" { let p: rawptr i64 = addr_of(x); } return 0; }",
    );
}

// ----- D-A: `out` is the mandatory call-site marker for out-args (§3.1) ----

#[test]
fn out_arg_with_marker_accepted() {
    assert_clean("fn g(a: out i64) -> unit { a = 1; } fn f() -> i64 { let mut x: i64; g(out x); return x; }");
}

#[test]
fn out_arg_without_marker_rejected() {
    assert_has(
        "fn g(a: out i64) -> unit { a = 1; } fn f() -> i64 { let mut x: i64; g(x); return x; }",
        "E0307",
    );
}

#[test]
fn out_marker_on_non_out_param_rejected() {
    assert_has(
        "fn g(a: write i64) -> unit { (deref a) = 1; } fn f() -> i64 { let mut x: i64 = 0; g(out x); return x; }",
        "E0308",
    );
}

// ----- E0309: scope-exit path-independence for needs-drop places ----------
// The dual of §1.6's move-join rule (finding 2026-07-07): at a needs-drop
// place's drop point its initialization must be path-independent, else the
// interpreter would decide the drop from a runtime flag.

/// A drop-hooked struct `R` plus `mk`/`sink` helpers used by the E0309 tests.
const RDROP: &str = "struct R { v: i64 } drop(write self) { trace((deref self).v); } \
                     fn mk(n: i64) -> R { return R { v: n }; } \
                     fn sink(r: R) -> unit { trace(r.v); } ";

#[test]
fn e0309_conditional_init_of_drop_hooked_rejected() {
    // The finding's exact repro: initialized on one path, not the other, live to
    // scope exit — the drop would be a runtime decision.
    let src = format!("{RDROP}fn f(c: bool) -> unit {{ let x: R; if c {{ x = mk(7); }} return; }}");
    assert_has(&src, "E0309");
    // Same shape falling off the end of the body (no explicit `return`).
    let src2 = format!("{RDROP}fn f(c: bool) -> unit {{ let x: R; if c {{ x = mk(7); }} }}");
    assert_has(&src2, "E0309");
}

#[test]
fn e0309_scalar_and_copy_struct_exempt() {
    // A drop-inert scalar: MaybeInit at scope exit stays legal.
    assert_clean("fn f(c: bool) -> unit { let x: i64; if c { x = 7; } return; }");
    // A copy aggregate is drop-inert too.
    assert_clean(
        "copy struct P { a: i64 } fn f(c: bool) -> unit { let x: P; if c { x = P { a: 7 }; } return; }",
    );
}

#[test]
fn e0309_box_bearing_type_rejected() {
    // Needs-drop transitively via a `Box` field (no drop hook). `clone` produces
    // it without moving `src`, so the only diagnostic is the scope-exit one.
    assert_has(
        "struct BB { b: Box i64 } \
         fn f(c: bool, src: BB) alloc -> unit { let x: BB; if c { x = clone src; } return; }",
        "E0309",
    );
}

#[test]
fn e0309_initialized_on_both_branches_ok() {
    let src = format!(
        "{RDROP}fn f(c: bool) -> unit {{ let x: R; if c {{ x = mk(7); }} else {{ x = mk(8); }} sink(x); }}"
    );
    assert_clean(&src);
}

#[test]
fn e0309_join_coherent_consume_variants_ok() {
    // Consumed on one path, uninitialized on the other: both paths leave the
    // place drop-free at scope exit (Moved / Uninit), which is path-independent.
    let a = format!("{RDROP}fn f(c: bool) -> unit {{ let x: R; if c {{ x = mk(7); sink(x); }} return; }}");
    assert_clean(&a);
    // Initialized-and-consumed on both paths: Moved on both, still path-independent.
    let b = format!(
        "{RDROP}fn f(c: bool) -> unit {{ let x: R; if c {{ x = mk(7); sink(x); }} else {{ x = mk(8); sink(x); }} return; }}"
    );
    assert_clean(&b);
}

#[test]
fn e0309_early_return_before_scope_exit_ok() {
    // Initialized then consumed-and-returned on one path; on the other it is
    // (re)initialized and consumed. State agrees at each *actual* scope exit.
    let src = format!(
        "{RDROP}fn f(c: bool) -> unit {{ let x: R; if c {{ x = mk(7); sink(x); return; }} x = mk(8); sink(x); return; }}"
    );
    assert_clean(&src);
}

#[test]
fn e0309_loop_conditional_init_then_break_ok() {
    // A back-edge case: `x` is fresh each iteration. On the break path it is
    // Uninit (never dropped); on the back-edge it is initialized-and-consumed
    // (Moved). Both scope exits are path-independent, so it is accepted.
    let src = format!(
        "{RDROP}fn g(n: i64) -> unit {{ let mut i: i64 = 0; \
         loop {{ let x: R; if i >= n {{ break; }} x = mk(i); sink(x); i = i + 1; }} return; }}"
    );
    assert_clean(&src);
}

#[test]
fn e0309_loop_backedge_maybe_init_rejected() {
    // The unsound shape the rule must catch on a back-edge: `x` is MaybeInit when
    // the loop body falls through to the header (its scope exit / drop point).
    let src = format!(
        "{RDROP}fn g(c: bool) -> unit {{ loop {{ let x: R; if c {{ x = mk(7); }} if c {{ break; }} }} return; }}"
    );
    assert_has(&src, "E0309");
}

#[test]
fn e0309_move_join_disagreement_still_e0302() {
    // The move dimension (§1.6 rule 1) is unchanged: a value live on one path and
    // moved on another is still E0302, independent of the new scope-exit rule.
    let src = format!("{RDROP}fn f(c: bool) -> unit {{ let x: R = mk(1); if c {{ sink(x); }} return; }}");
    assert_has(&src, "E0302");
}

// ==========================================================================
// 2026-07-07 soundness review #1 — the five accepted findings, as regression
// tests. Repros and controls from `target/scratch-review/` (f1..f4, c1..c6).
// ==========================================================================

// ---- Finding 1: return-borrow provenance is TOTAL (E0806) ----------------

#[test]
fn f2_borrow_of_local_through_match_rejected() {
    // The decisive repro: a borrow of a local laundered through a `match` must
    // not escape the region check. Provenance recurses into every arm.
    assert_has(
        "enum Sel { a, b } \
         fn pick(s: Sel) -> borrow i64 { let x: i64 = 5; \
         return match s { case Sel::a => read x, case Sel::b => read x, }; }",
        "E0806",
    );
}

#[test]
fn f2b_borrow_of_local_direct_still_rejected() {
    // The direct control stays rejected.
    assert_has(
        "fn pick() -> borrow i64 { let x: i64 = 5; return read x; }",
        "E0806",
    );
}

#[test]
fn borrow_of_param_through_match_accepted() {
    // The legal counterpart: each arm derives from the sole borrow parameter, so
    // the launder is region-legal and accepted (provenance total, not blanket).
    assert_clean(
        "struct T { v: i64 } enum Sel { a, b } \
         fn pick(t: read T, s: Sel) -> borrow i64 { \
         return match s { case Sel::a => read (deref t).v, \
                          case Sel::b => read (deref t).v, }; }",
    );
}

// ---- Finding 2: reassignment & out-argument drop points (E0309) ----------

#[test]
fn f1_reassignment_of_maybe_init_needs_drop_rejected() {
    // Whole-binding reassignment of a needs-drop place that is MaybeInit: the
    // old value's drop would be a runtime decision (the drop flag the resolution
    // claimed to remove).
    assert_has(
        "struct R { v: i64 } drop(write self) { trace((deref self).v); } \
         fn mk(n: i64) -> R { return R { v: n }; } \
         fn f(c: bool) -> unit { let x: R; if c { x = mk(1); } x = mk(2); return; }",
        "E0309",
    );
}

#[test]
fn f1b_scope_exit_of_maybe_init_needs_drop_still_rejected() {
    // The original scope-exit drop point stays rejected (the first resolution).
    assert_has(
        "struct R { v: i64 } drop(write self) { trace((deref self).v); } \
         fn mk(n: i64) -> R { return R { v: n }; } \
         fn f(c: bool) -> unit { let x: R; if c { x = mk(1); } return; }",
        "E0309",
    );
}

#[test]
fn out_arg_drop_point_of_maybe_init_needs_drop_rejected() {
    // Passing a MaybeInit needs-drop place as `out` drops its old value at the
    // call — the same drop point as a reassignment (finding 2, out path).
    assert_has(
        "struct R { v: i64 } drop(write self) { trace((deref self).v); } \
         fn mk(n: i64) -> R { return R { v: n }; } \
         fn fill(o: out R) -> unit { o = mk(9); } \
         fn f(c: bool) -> unit { let x: R; if c { x = mk(1); } fill(out x); return; }",
        "E0309",
    );
}

#[test]
fn reassignment_of_definitely_init_needs_drop_ok() {
    // Reassigning a place that is Init on every path is a static drop — accepted.
    assert_clean(
        "struct R { v: i64 } drop(write self) { trace((deref self).v); } \
         fn mk(n: i64) -> R { return R { v: n }; } \
         fn f() -> unit { let mut x: R = mk(1); x = mk(2); return; }",
    );
}

// ---- Finding 3: nested partial move out of a drop-hooked struct (E0303) ---

#[test]
fn f4_nested_partial_move_through_hooked_intermediate_rejected() {
    // `Outer` has no hook but the intermediate `Outer.a : A` does — moving
    // `outer.a.leaf` out would skip `A`'s hook. Every proper prefix is walked.
    assert_has(
        "struct Leaf { v: i64 } struct A { leaf: Leaf } drop(write self) { trace(1); } \
         struct Outer { a: A } \
         fn f() -> i64 { let outer: Outer = Outer { a: A { leaf: Leaf { v: 1 } } }; \
         let l: Leaf = outer.a.leaf; return l.v; }",
        "E0303",
    );
}

#[test]
fn f4b_direct_partial_move_of_hooked_still_rejected() {
    // The direct (root is the hooked struct) control stays rejected.
    assert_has(
        "struct Leaf { v: i64 } struct A { leaf: Leaf } drop(write self) { trace(1); } \
         fn f() -> i64 { let a: A = A { leaf: Leaf { v: 1 } }; let l: Leaf = a.leaf; return l.v; }",
        "E0303",
    );
}

#[test]
fn legal_partial_move_no_hook_on_path_accepted() {
    // No `drop` hook anywhere on the projection path: the partial move stays
    // legal (the rule is non-vacuous, not blanket).
    assert_clean(
        "struct Leaf { v: i64 } struct A { leaf: Leaf } struct Outer { a: A } \
         fn f() -> i64 { let outer: Outer = Outer { a: A { leaf: Leaf { v: 1 } } }; \
         let l: Leaf = outer.a.leaf; return l.v; }",
    );
}

// ---- Finding 4: the free side of the alloc effect (E0401) -----------------

#[test]
fn f3_unbox_in_nonalloc_rejected() {
    // `unbox` frees the box storage — allocator work.
    assert_has(
        "struct T { v: i64 } fn unboxes(b: Box T) -> i64 { return unbox(b).v; }",
        "E0401",
    );
}

#[test]
fn f3_box_param_dropped_in_nonalloc_rejected() {
    // A `Box` received by value and let die (dropped at function exit) frees.
    assert_has(
        "struct T { v: i64 } fn frees(b: Box T) -> unit { return; }",
        "E0401",
    );
}

#[test]
fn f3_box_free_accepted_when_marked_alloc() {
    // The same two functions with the honest `alloc` marker check clean.
    assert_clean(
        "struct T { v: i64 } \
         fn unboxes(b: Box T) alloc -> i64 { return unbox(b).v; } \
         fn frees(b: Box T) alloc -> unit { return; }",
    );
}

#[test]
fn box_reassignment_drop_is_alloc() {
    // Reassigning an initialized `Box` binding frees the old box.
    assert_has(
        "fn f(b: Box i64, n: Box i64) -> unit { b = n; return; }",
        "E0401",
    );
}

#[test]
fn drop_hooked_nonbox_param_not_alloc() {
    // Precision: a drop hook that owns no `Box` does not allocate — dropping it
    // must not force the marker (only a `Box` drop frees).
    assert_clean(
        "struct R { v: i64 } drop(write self) { trace((deref self).v); } \
         fn f(r: R) -> unit { return; }",
    );
}

#[test]
fn box_param_moved_out_not_alloc() {
    // Precision: a box passed straight through (moved out, never dropped here)
    // does not allocate — the disposition's "lets it die", not conservatism.
    assert_clean("fn pass(b: Box i64) -> Box i64 { return b; }");
}

// ---- c1..c6 coverage controls (core machinery survived the attack) --------
// c1 (E0807), c3 (E0401 clone), c4 (E0701), c5 (E0805), c6/c2 (E0802) are
// already covered by existing tests (loans.rs / check.rs). The one new code is
// c2's E0401 via `unbox`, covered by `f3_unbox_in_nonalloc_rejected` above.

#[test]
fn c1_two_borrow_params_return_needs_region() {
    assert_has(
        "struct T { v: i64 } \
         fn f[r](a: write[r] T, b: write T) -> borrow_mut i64 { return write (deref b).v; }",
        "E0807",
    );
}

// ---- Soundness review #2 (2026-07-07): opaque non-copy moves rejected (E0310)
// A non-copy value moved out of a place whose projection path crosses a `deref`
// or index is rejected. Copy reads through the same paths are unaffected. See
// docs/reviews/2026-07-07-soundness-review-2.md.

fn assert_lacks(src: &str, code: &str) {
    let cs = codes(src);
    assert!(
        !cs.iter().any(|c| c == code),
        "expected NO `{code}` for:\n{src}\ngot {cs:?}"
    );
}

#[test]
fn r2_partial_move_through_box_deref_rejected() {
    // G_partial: the shape the re-review used; formerly a false-E0401 (finding
    // 2), now the opaque-move error and no spurious free.
    let src = "struct Leaf { v: i64 } struct A { leaf: Leaf } drop(write self) { trace(1); } \
         struct W { a: A } \
         fn f(bx: Box W) alloc -> i64 { let l: Leaf = (deref bx).a.leaf; return l.v; }";
    assert_has(src, "E0310");
    assert_lacks(src, "E0401");
}

#[test]
fn r2_double_drop_run_shape_rejected() {
    // G7/G8: move a non-copy field out through the box deref, then `unbox` — the
    // double-drop the re-review demonstrated. Rejected at the opaque move.
    assert_has(
        "struct Inner { v: i64 } struct W { inner: Inner } \
         fn trigger(bx: Box W) alloc -> unit { let taken: Inner = (deref bx).inner; \
         let w: W = unbox(bx); return; }",
        "E0310",
    );
}

#[test]
fn r2_use_after_partial_box_move_rejected() {
    // G6: partial move through box deref then whole `unbox`.
    assert_has(
        "struct Leaf { v: i64 } struct A { leaf: Leaf } drop(write self) { trace(1); } \
         struct W { a: A } \
         fn f(bx: Box W) alloc -> i64 { let l1: Leaf = (deref bx).a.leaf; \
         let w: W = unbox(bx); return l1.v; }",
        "E0310",
    );
}

#[test]
fn r2_move_through_write_borrow_deref_rejected() {
    // K2: no Box involved — a non-copy field moved out through an exclusive
    // borrow's deref would hollow out the lender. Rejected.
    assert_has(
        "struct Cell { p: rawptr i64 } \
         drop(write self) { unsafe \"x\" { let o: i64 = ptr_read((deref self).p); ptr_write((deref self).p, o + 1); } } \
         struct W { c: Cell } \
         fn steal(w: write W) -> Cell { let taken: Cell = (deref w).c; return taken; }",
        "E0310",
    );
}

#[test]
fn r2_array_element_hooked_move_rejected() {
    // H: a drop-hooked element moved out by constant index — index granularity
    // is beyond the prototype's place model, so the non-copy element move is
    // rejected (0001 §1.6 narrowed to copy element types).
    assert_has(
        "struct A { v: i64 } drop(write self) { trace(1); } \
         fn f() -> i64 { let arr: [2]A = [A { v: 1 }, A { v: 2 }]; let x: A = arr[0]; return x.v; }",
        "E0310",
    );
}

#[test]
fn r2_owned_scrutinee_through_deref_match_rejected() {
    // The match-head form: an owned scrutinee reached through a `deref` whose
    // arms move-bind a non-copy payload is the same illegal opaque move.
    assert_has(
        "struct P { v: i64 } enum E { some(P), none } \
         fn f(bx: Box E) alloc -> i64 { \
         match deref bx { case E::some(p) => { return p.v; } case E::none => { return 0; } } }",
        "E0310",
    );
}

#[test]
fn r2_direct_partial_move_control_still_e0303() {
    // G3: the direct (field-only) partial move of a drop-hooked struct keeps its
    // existing code — the new rule does not swallow the direct rule.
    let src = "struct Leaf { v: i64 } struct A { leaf: Leaf } drop(write self) { trace(1); } \
         struct W { a: A } \
         fn f() -> i64 { let w: W = W { a: A { leaf: Leaf { v: 5 } } }; let l: Leaf = w.a.leaf; return l.v; }";
    assert_has(src, "E0303");
    assert_lacks(src, "E0310");
}

#[test]
fn r2_move_box_then_use_control_still_e0301() {
    // I: moving a whole `Box` binding then using it stays a use-after-move.
    assert_has(
        "fn sink(b: Box i64) -> unit { return; } \
         fn f(b: Box i64) alloc -> unit { sink(b); let v: i64 = deref b; return; }",
        "E0301",
    );
}

#[test]
fn r2_box_forwarded_out_still_clean() {
    // P1: moving a whole box out is fine (no opaque move, no free here).
    assert_clean("fn forward(b: Box i64) -> Box i64 { return b; }");
}

#[test]
fn r2_box_dropped_in_nonalloc_still_e0401() {
    // P2: alloc-precision unchanged — a box let die in a non-alloc fn frees.
    assert_has("fn eat(b: Box i64) -> unit { return; }", "E0401");
}

#[test]
fn r2_copy_reads_through_deref_and_index_accepted() {
    // Positive: reading a `copy` value through `deref` or index still copies and
    // stays accepted everywhere — the ruling touches only non-copy moves.
    assert_clean(
        "struct S { v: i64 } \
         fn rd(s: read S) -> i64 { return (deref s).v; } \
         fn arr_rd() -> i64 { let a: [3]i64 = [1, 2, 3]; let x: i64 = a[0]; return x; } \
         fn box_rd(b: Box i64) alloc -> i64 { let v: i64 = deref b; return unbox(b); }",
    );
}

// ---- Soundness review #3 (2026-07-07): contracts read-only + ensures dataflow,
// and immutable statics --------------------------------------------------------

#[test]
fn ensures_read_of_body_moved_param_is_e0301() {
    // Fix 1(b): `ensures` is analyzed against the post-body state at each return.
    // The body moved `x`, so the clause's read of `x.v` is the ordinary E0301.
    assert_has(
        "struct R { v: i64 } \
         fn f(x: R) ensures(x.v == 7) -> i64 { let y: R = x; return y.v; }",
        "E0301",
    );
}

#[test]
fn ensures_unbox_then_deref_is_e0301() {
    // The body `unbox`es the Box param; the clause dereferences the freed box —
    // a use-after-free that is now the ordinary use-of-moved diagnostic.
    assert_has(
        "struct R { v: i64 } \
         fn f(bx: Box R) ensures((deref bx).v == 7) alloc -> i64 { let r: R = unbox(bx); return r.v; }",
        "E0301",
    );
}

#[test]
fn ensures_reading_live_param_is_clean() {
    // Control: a clause reading a still-live param passes the dataflow check.
    assert_clean("struct R { v: i64 } fn f(x: R) ensures(x.v == 7) -> i64 { return x.v; }");
}

#[test]
fn contract_write_borrow_is_e0708() {
    // Fix 1(a) read-only rule: a `write`-borrow inside a contract is rejected.
    assert_has(
        "fn g(p: write i64) -> bool { return true; } \
         fn f(y: i64) ensures(g(write y)) -> i64 { return y; }",
        "E0708",
    );
}

#[test]
fn contract_out_arg_is_e0708() {
    // Read-only rule: an `out` argument inside a contract is rejected.
    assert_has(
        "fn g(p: out i64) -> bool { p = 0; return true; } \
         fn f() -> i64 { let mut r: i64; assert(g(out r)); return r; }",
        "E0708",
    );
}

#[test]
fn contract_call_taking_by_take_is_e0708() {
    // Read-only rule: a call taking a non-copy argument by `take` inside a
    // contract consumes it and is rejected.
    assert_has(
        "struct R { v: i64 } \
         fn consume(r: R) -> bool { return r.v == 0; } \
         fn f(x: R) ensures(consume(x)) -> i64 { return 0; }",
        "E0708",
    );
}

#[test]
fn contract_read_borrow_and_copy_take_are_clean() {
    // Read-only rule permits reads, `read`-borrows, and copy-`take` calls.
    assert_clean(
        "fn peek(p: read i64) -> bool { return (deref p) == 0; } \
         fn dbl(n: i64) -> bool { return n == n; } \
         fn f(y: i64) ensures(peek(read y)) requires(dbl(y)) -> i64 { return y; }",
    );
}

#[test]
fn static_assignment_is_e0311() {
    // Fix 2: statics are immutable.
    assert_has(
        "static COUNTER: i64 = 0; fn f() -> i64 { COUNTER = 5; return COUNTER; }",
        "E0311",
    );
}

#[test]
fn static_write_borrow_is_e0311() {
    assert_has(
        "static COUNTER: i64 = 0; \
         fn g(p: write i64) -> unit { return; } \
         fn f() -> i64 { g(write COUNTER); return COUNTER; }",
        "E0311",
    );
}

#[test]
fn static_out_arg_is_e0311() {
    assert_has(
        "static COUNTER: i64 = 0; \
         fn g(p: out i64) -> unit { p = 1; return; } \
         fn f() -> i64 { g(out COUNTER); return COUNTER; }",
        "E0311",
    );
}

#[test]
fn static_read_and_read_borrow_are_clean() {
    // Reading and `read`-borrowing a static stay legal.
    assert_clean(
        "static COUNTER: i64 = 7; \
         fn g(p: read i64) -> i64 { return deref p; } \
         fn f() -> i64 { return g(read COUNTER) + COUNTER; }",
    );
}
