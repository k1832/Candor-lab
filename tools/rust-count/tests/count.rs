//! Bet 5 Rust counter tests (frozen unit table, table_version 1).
//! Each asserts exact counts on a small Rust snippet.

use rust_count::{count_str, Counts};

fn counts(src: &str) -> Counts {
    count_str(src).expect("parse ok")
}

#[test]
fn shared_ref_param_is_one_class_a() {
    let c = counts("fn f(x: &i64) {}");
    assert_eq!(c.annotation_units.a, 1);
    assert_eq!(c.annotation_units.b, 0);
    assert_eq!(c.annotation_units.d, 0);
    assert_eq!(c.logical_statements, 1);
    assert_eq!(c.annotation_units.per_site[0].kind, "ref");
}

#[test]
fn mut_ref_param_is_one_class_a() {
    let c = counts("fn f(x: &mut i64) {}");
    assert_eq!(c.annotation_units.a, 1);
    assert_eq!(c.annotation_units.per_site[0].kind, "ref_mut");
}

#[test]
fn plain_value_param_is_zero_annotation() {
    let c = counts("fn f(x: i64) {}");
    assert_eq!(c.annotation_units.a, 0);
    assert_eq!(c.annotation_units.b, 0);
    assert_eq!(c.annotation_units.c, 0);
    assert_eq!(c.annotation_units.d, 0);
}

#[test]
fn self_receivers_count_class_a() {
    let c = counts("struct S; impl S { fn a(&self) {} fn b(&mut self) {} }");
    assert_eq!(c.annotation_units.a, 2);
    assert_eq!(c.valve.total_functions, 2);
    assert_eq!(c.logical_statements, 3); // struct + 2 methods; impl block not counted
}

#[test]
fn lifetime_decl_attach_and_return() {
    let c = counts("fn pick<'a>(a: &'a i64, b: &i64) -> &'a i64 { a }");
    assert_eq!(c.annotation_units.a, 3); // &'a a, &b, &'a return
    assert_eq!(c.annotation_units.b, 3); // decl + attach on a + attach on return
    assert_eq!(c.annotation_units.c, 0);
}

#[test]
fn static_and_elided_lifetimes_cost_zero() {
    let c = counts("fn f(x: &i64) -> &'static str { \"\" }");
    assert_eq!(c.annotation_units.a, 2); // two references
    assert_eq!(c.annotation_units.b, 0); // 'static and elided both cost zero
}

#[test]
fn raw_pointer_field_is_class_d() {
    let c = counts("struct P { head: *mut u8, n: usize }");
    assert_eq!(c.annotation_units.d, 1);
    assert_eq!(c.logical_statements, 1);
    assert_eq!(c.valve.total_functions, 0);
    assert_eq!(c.valve.functions, 0);
    assert_eq!(c.valve.lines, 1);
}

#[test]
fn cell_family_field_is_class_d() {
    let c = counts("struct S { c: std::cell::RefCell<i64> }");
    assert_eq!(c.annotation_units.d, 1);
    assert_eq!(c.annotation_units.per_site.iter().filter(|s| s.kind == "cell").count(), 1);
}

#[test]
fn unsafe_block_and_raw_pointer_param_are_class_d() {
    let c = counts("fn f(p: *mut u8) { unsafe { let _y = 1; } }");
    assert_eq!(c.annotation_units.d, 2); // rawptr param + unsafe block
    assert_eq!(c.annotation_units.a, 0);
    assert_eq!(c.valve.functions, 1);
    assert_eq!(c.valve.total_functions, 1);
    assert_eq!(c.logical_statements, 3); // fn + unsafe-stmt + inner let
}

#[test]
fn unsafe_fn_is_class_d_and_valve_function() {
    let c = counts("unsafe fn f() { let _x = 1; }");
    assert_eq!(c.annotation_units.d, 1);
    assert_eq!(c.annotation_units.per_site[0].kind, "unsafe_fn");
    assert_eq!(c.valve.functions, 1);
    assert_eq!(c.valve.total_functions, 1);
}

#[test]
fn clone_and_to_owned_are_value_copy_units() {
    let c = counts("fn f(x: &Vec<u8>) -> Vec<u8> { let y = x.clone(); y.to_owned() }");
    assert_eq!(c.value_copy_units, 2);
    assert_eq!(c.annotation_units.a, 1); // the &Vec param
    assert_eq!(c.logical_statements, 3); // fn + let y + tail expr
}

#[test]
fn nested_statement_counting() {
    let c = counts("fn f() { let a = 1; if true { let b = 2; } }");
    assert_eq!(c.logical_statements, 4); // fn + let a + if-stmt + inner let b
}

#[test]
fn valve_line_and_function_fractions() {
    let src = "fn f() {\n    let a = 1;\n    unsafe {\n        let b = 2;\n    }\n}\n";
    let c = counts(src);
    assert_eq!(c.valve.total_functions, 1);
    assert_eq!(c.valve.functions, 1);
    assert_eq!(c.valve.total_lines, 6);
    assert_eq!(c.valve.lines, 3); // unsafe block spans lines 3..5
}

#[test]
fn valve_function_fraction_partial() {
    let c = counts("fn a() {}\nfn b(p: *mut u8) { unsafe {} }");
    assert_eq!(c.valve.total_functions, 2);
    assert_eq!(c.valve.functions, 1);
}

#[test]
fn deterministic_sites_sorted_by_span() {
    let c = counts("fn f(a: &i64, b: &mut i64) {}");
    let starts: Vec<usize> = c.annotation_units.per_site.iter().map(|s| s.start).collect();
    let mut sorted = starts.clone();
    sorted.sort();
    assert_eq!(starts, sorted);
    assert_eq!(c.annotation_units.a, 2);
}

#[test]
fn valve_statements_inside_unsafe_block_counted() {
    // fn + let a + unsafe-stmt + let b + let c = 5 logical statements.
    // Valve statements: the unsafe-block statement (span-coincident with the
    // valve, partly-inside not strictly enclosing) plus the two inner lets = 3.
    // The outer `let a` and the enclosing fn are NOT counted.
    let c = counts("fn f() { let a = 1; unsafe { let b = 2; let c = 3; } }");
    assert_eq!(c.logical_statements, 5);
    assert_eq!(c.valve_statements, 3);
    assert_eq!(c.unit_ext_version, "2");
    assert_eq!(c.table_version, "1");
}

#[test]
fn valve_statements_exclude_fn_and_siblings_outside_unsafe() {
    // A statement in a fn that contains a valve but sits OUTSIDE the unsafe block
    // is not counted; neither is the enclosing fn nor the wrapping `if`.
    let c = counts("fn f() { if true { unsafe { let z = 1; } } let w = 9; }");
    assert_eq!(c.logical_statements, 5); // fn + if + unsafe + z + w
    assert_eq!(c.valve_statements, 2); // unsafe-stmt + z
}

#[test]
fn valve_statements_unsafe_fn_body_is_all_valve() {
    // An `unsafe fn` makes its whole body a valve region: the two body statements
    // (and the coincident fn declaration) are valve statements.
    let c = counts("unsafe fn f() { let x = 1; let y = 2; }");
    assert_eq!(c.logical_statements, 3);
    assert_eq!(c.valve_statements, 3);
}

#[test]
fn valve_statements_rawptr_declaration_is_not_a_valve_statement() {
    // A raw-pointer field opens a valve region on the type node, but the
    // declaration statement strictly encloses it, so it is not a valve statement.
    let c = counts("struct P { head: *mut u8, n: usize }");
    assert_eq!(c.logical_statements, 1);
    assert_eq!(c.valve_statements, 0);
}
