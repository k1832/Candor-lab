//! Bet 5 Candor counter tests (frozen unit table, table_version 1).
//! Each asserts exact counts on a small `.cn` snippet.

use candor_proto::count::Counts;
use candor_proto::count_source;

fn counts(src: &str) -> Counts {
    count_source(src).expect("parse ok")
}

#[test]
fn read_param_is_one_class_a() {
    let c = counts("fn f(x: read i64) -> unit { }");
    assert_eq!(c.annotation_units.a, 1);
    assert_eq!(c.annotation_units.b, 0);
    assert_eq!(c.annotation_units.d, 0);
    assert_eq!(c.logical_statements, 1);
    assert_eq!(c.annotation_units.per_site[0].kind, "read_param");
}

#[test]
fn write_param_is_one_class_a() {
    let c = counts("fn f(x: write i64) -> unit { }");
    assert_eq!(c.annotation_units.a, 1);
    assert_eq!(c.annotation_units.per_site[0].kind, "write_param");
}

#[test]
fn out_param_is_one_class_a() {
    let c = counts("fn f(x: out i64) -> unit { }");
    assert_eq!(c.annotation_units.a, 1);
    assert_eq!(c.annotation_units.per_site[0].kind, "out_param");
}

#[test]
fn take_value_param_is_zero_annotation() {
    let c = counts("fn f(x: i64) -> unit { }");
    assert_eq!(c.annotation_units.a, 0);
    assert_eq!(c.annotation_units.b, 0);
    assert_eq!(c.annotation_units.c, 0);
    assert_eq!(c.annotation_units.d, 0);
}

#[test]
fn slice_and_slice_mut_params_are_class_a() {
    let c = counts("fn f(s: slice i64) -> unit { } fn g(s: slice_mut i64) -> unit { }");
    assert_eq!(c.annotation_units.a, 2);
    assert!(c
        .annotation_units
        .per_site
        .iter()
        .all(|s| s.kind == "slice_param"));
}

#[test]
fn regions_declaration_and_attachments() {
    // decl [r] + attach on `a` + attach on return = 3 class-(b); reads + borrow
    // return = 3 class-(a). `b` carries no region.
    let c = counts("fn pick[r](a: read[r] i64, b: read i64) -> read[r] i64 { }");
    assert_eq!(c.annotation_units.a, 3);
    assert_eq!(c.annotation_units.b, 3);
    assert_eq!(c.annotation_units.c, 0);
}

#[test]
fn unsafe_block_and_rawptr_param_are_class_d() {
    let c = counts("fn f(p: rawptr u8) -> unit { unsafe \"x\" { let y: i64 = 0; } }");
    assert_eq!(c.annotation_units.d, 2); // rawptr param + unsafe block
    assert_eq!(c.annotation_units.a, 0); // rawptr is not a borrow
    assert_eq!(c.valve.functions, 1);
    assert_eq!(c.valve.total_functions, 1);
    assert_eq!(c.logical_statements, 3); // fn + unsafe-stmt + inner let
}

#[test]
fn rawptr_struct_field_is_class_d_not_a_function() {
    let c = counts("struct P { head: rawptr u8, n: usize }");
    assert_eq!(c.annotation_units.d, 1);
    assert_eq!(c.logical_statements, 1);
    assert_eq!(c.valve.total_functions, 0);
    assert_eq!(c.valve.functions, 0);
    assert_eq!(c.valve.lines, 1); // the rawptr field line is a valve line
}

#[test]
fn clone_is_one_value_copy_unit() {
    let c = counts("fn f(x: i64) -> i64 { return clone x; }");
    assert_eq!(c.value_copy_units, 1);
    assert_eq!(c.annotation_units.a, 0);
    assert_eq!(c.logical_statements, 2); // fn + `return clone x;` stmt
}

#[test]
fn nested_statement_counting() {
    let c = counts("fn f() -> unit { let a = 1; let b = 2; if true { let c = 3; } }");
    // fn + let a + let b + if-stmt + inner let c = 5
    assert_eq!(c.logical_statements, 5);
}

#[test]
fn valve_line_and_function_fractions() {
    let src = "fn f() -> unit {\n    let a = 1;\n    unsafe \"x\" {\n        let b = 2;\n    }\n}\n";
    let c = counts(src);
    assert_eq!(c.valve.total_functions, 1);
    assert_eq!(c.valve.functions, 1);
    assert_eq!(c.valve.total_lines, 6); // 6 code lines
    assert_eq!(c.valve.lines, 3); // unsafe block spans lines 3..5
}

#[test]
fn valve_function_fraction_partial() {
    let c = counts("fn a() -> unit { } fn b(p: rawptr u8) -> unit { unsafe \"j\" { } }");
    assert_eq!(c.valve.total_functions, 2);
    assert_eq!(c.valve.functions, 1);
}

#[test]
fn drop_hook_is_function_like_with_write_self() {
    let c = counts("struct S { x: i64 } drop(write self) { let y = 1; }");
    assert_eq!(c.annotation_units.a, 1); // the `write self` receiver
    assert_eq!(c.annotation_units.per_site[0].kind, "drop_self");
    assert_eq!(c.valve.total_functions, 1); // the hook is function-like
    assert_eq!(c.logical_statements, 3); // struct + drop-hook decl + inner let
}

#[test]
fn deterministic_sites_sorted_by_span() {
    let c = counts("fn f(a: read i64, b: write i64) -> unit { }");
    let starts: Vec<usize> = c.annotation_units.per_site.iter().map(|s| s.start).collect();
    let mut sorted = starts.clone();
    sorted.sort();
    assert_eq!(starts, sorted);
    assert_eq!(c.annotation_units.a, 2);
}

#[test]
fn valve_statements_inside_unsafe_block_counted() {
    // fn + let a + unsafe-stmt + let b + let c = 5 logical statements.
    // Valve statements: the unsafe-block statement (its span coincides with the
    // valve region — partly-inside, not strictly enclosing) plus the two inner
    // lets = 3. The outer `let a` and the enclosing fn are NOT counted.
    let c = counts("fn f() -> unit { let a = 1; unsafe \"x\" { let b = 2; let c = 3; } }");
    assert_eq!(c.logical_statements, 5);
    assert_eq!(c.valve_statements, 3);
    assert_eq!(c.unit_ext_version, "2");
    assert_eq!(c.table_version, "1");
}

#[test]
fn valve_statements_exclude_fn_and_siblings_outside_unsafe() {
    // Statements in a fn that contains a valve but sit OUTSIDE the unsafe block
    // are not counted; neither is the enclosing fn declaration.
    let c = counts("fn outer() -> unit { let before = 0; unsafe \"y\" { let inside = 1; } let after = 2; }");
    assert_eq!(c.logical_statements, 5); // fn + before + unsafe + inside + after
    assert_eq!(c.valve_statements, 2); // unsafe-stmt + inside
}

#[test]
fn valve_statements_empty_unsafe_block_counts_only_the_unsafe_statement() {
    // The unsafe-block statement is partly inside the valve (span-coincident);
    // it counts even with an empty body. The enclosing fn does not.
    let c = counts("fn f() -> unit { unsafe \"z\" { } }");
    assert_eq!(c.logical_statements, 2); // fn + unsafe-stmt
    assert_eq!(c.valve_statements, 1);
}

#[test]
fn valve_statements_rawptr_declaration_is_not_a_valve_statement() {
    // A struct with a rawptr field opens a valve region on the type node, but the
    // declaration statement strictly encloses it, so it is not a valve statement.
    let c = counts("struct P { head: rawptr u8, n: usize }");
    assert_eq!(c.logical_statements, 1);
    assert_eq!(c.valve_statements, 0);
    // A rawptr param + empty body: the fn strictly encloses the valve span.
    let c2 = counts("fn f(p: rawptr u8) -> unit { }");
    assert_eq!(c2.logical_statements, 1);
    assert_eq!(c2.valve_statements, 0);
}

#[test]
fn cross_counter_fixture_valve_statements() {
    // Shared cross-counter fixture: the paired Candor (.cn) and Rust (.rs) files
    // express the same shapes, and both counters must report the SAME
    // valve_statements. Hand-computed = 6 (per-fn derivation in the fixture
    // header). This is the Candor half; tools/rust-count asserts 6 on the .rs.
    let path = format!(
        "{}/tests/fixtures/count/cross_counter.cn",
        env!("CARGO_MANIFEST_DIR")
    );
    let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let c = counts(&src);
    assert_eq!(c.valve_statements, 6);
    assert_eq!(c.unit_ext_version, "2");
}

#[test]
fn field_ptr_region_rule_preserved_outside_and_inside_unsafe() {
    // Design 0004: `field_ptr` is a SAFE op and opens NO valve region. The frozen
    // region rule (BET5_CRITERION2 §4.1) is preserved BOTH ways:
    //   (1) a `field_ptr` statement OUTSIDE any `unsafe` region contributes ZERO
    //       valve statements; and
    //   (2) the SAME statement INSIDE an `unsafe` block still counts exactly as
    //       its enclosing region does (like any statement there).
    // rust-count has no `field_ptr` token, so this Candor-side test suffices and
    // the shared cross-counter fixture is not extended.
    let outside = counts(
        "struct T { a: i64, b: i64 } \
         fn f(p: rawptr T) -> unit { let q: rawptr i64 = field_ptr(p, b); }",
    );
    // struct + fn + let = 3 logical statements. The only valve region is the
    // `rawptr T` parameter type node; the fn strictly encloses it and the body
    // `let` does not intersect it — so the field_ptr statement is NOT a valve
    // statement.
    assert_eq!(outside.logical_statements, 3);
    assert_eq!(outside.valve_statements, 0);

    let inside = counts(
        "struct T { a: i64, b: i64 } \
         fn g(p: rawptr T) -> unit { unsafe \"x\" { let q: rawptr i64 = field_ptr(p, b); } }",
    );
    // struct + fn + unsafe-stmt + inner let = 4 logical statements. The `unsafe`
    // block opens a valve region; the unsafe-block statement and the inner
    // field_ptr `let` both intersect it → 2 valve statements. `field_ptr` still
    // contributes no region of its own — the count comes entirely from `unsafe`.
    assert_eq!(inside.logical_statements, 4);
    assert_eq!(inside.valve_statements, 2);
}
