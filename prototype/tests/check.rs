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
