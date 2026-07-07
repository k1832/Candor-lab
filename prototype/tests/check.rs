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
    assert_clean(
        "struct S { a: Box i64, b: Box i64 } fn t(x: Box i64) -> unit { } \
         fn f(s: S, n: Box i64) -> unit { t(s.a); s.a = n; t(s.a); }",
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
