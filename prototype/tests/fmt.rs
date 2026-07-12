//! Design 0011/0013 formatting foundation: `fmt_i64` (decimal rendering of a
//! signed 64-bit integer, total including `i64::MIN`) and the `Show` value-
//! rendering convention (a 0009-style interface composing through a `T: Show`
//! bound). The corelib source is the self-contained image
//! `fixtures/std_fmt.cnr` (the `corelib_flat` pattern; `String` std is single-
//! file because it is a MIR-interp-only CollectionOp, excluded from the backend
//! corpus — see that file's header). These tests append a `main` to those
//! definitions and OBSERVE the produced `String` at runtime.

use candor_proto::diag::Severity;
use candor_proto::{check_source_real, run_source_real, RunResult};

fn source() -> String {
    let path = format!("{}/tests/fixtures/std_fmt.cnr", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

// Append a `main` (with the bump-allocator setup) to the formatting definitions.
fn wrap(body: &str) -> String {
    format!(
        "{}\nfn main() alloc -> i64 {{\n  let mut bs: Bump = with_window(16777216, 1048576);\n  let al: Alloc = mk_alloc(write bs);\n{body}\n}}",
        source()
    )
}

fn run_ret(body: &str) -> i64 {
    match run_source_real(&wrap(body)) {
        RunResult::Ok(r) => r.ret,
        RunResult::Fault(f) => panic!("unexpected fault: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!("check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        RunResult::ParseError(d) => panic!("parse error: {}", d.to_json()),
    }
}

fn run_trace(body: &str) -> Vec<i64> {
    match run_source_real(&wrap(body)) {
        RunResult::Ok(r) => r.trace,
        RunResult::Fault(f) => panic!("unexpected fault: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!("check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        RunResult::ParseError(d) => panic!("parse error: {}", d.to_json()),
    }
}

// Build `fmt_i64(al, <n>)`, compare its `as_str` view against the exact decimal
// literal (str equality is bytewise, tests/text.rs). Returns 1 on a byte-match.
fn fmt_eq(n: &str, expect: &str) -> i64 {
    run_ret(&format!(
        "let s: String = fmt_i64(read al, {n}); if as_str(read s) == \"{expect}\" {{ return 1; }} return 0;"
    ))
}

// ---- the corelib image checks clean ----------------------------------------

#[test]
fn image_checks_clean() {
    // The formatting source + a `main` exercising the whole convention (fmt_i64,
    // Show, and the generic `T: Show` composition) type-checks with no errors.
    let src = wrap("let s: String = fmt_i64(read al, 0); return conv i64 len(as_str(read s));");
    let diags = check_source_real(&src).unwrap_or_else(|p| panic!("parse error: {}", p.to_json()));
    let errs: Vec<_> = diags.iter().filter(|d| d.severity == Severity::Error).map(|d| &d.code).collect();
    assert!(errs.is_empty(), "std_fmt.cnr should check clean, got {errs:?}");
}

// ---- fmt_i64: the decimal primitive ----------------------------------------

#[test]
fn fmt_i64_zero() {
    assert_eq!(fmt_eq("0", "0"), 1);
}

#[test]
fn fmt_i64_positive() {
    assert_eq!(fmt_eq("42", "42"), 1);
}

#[test]
fn fmt_i64_negative() {
    assert_eq!(fmt_eq("0 - 42", "-42"), 1);
}

#[test]
fn fmt_i64_max() {
    assert_eq!(fmt_eq("9223372036854775807", "9223372036854775807"), 1);
}

#[test]
fn fmt_i64_min() {
    // i64::MIN: negating it overflows, so the digits are produced in the negative
    // domain. This is the load-bearing edge case.
    assert_eq!(fmt_eq("-9223372036854775808", "-9223372036854775808"), 1);
}

#[test]
fn fmt_i64_min_not_a_fault() {
    // A wrong MIN handling would trip Candor's arithmetic-overflow fault rather
    // than return; asserting a clean run to `len==20` guards that path directly.
    assert_eq!(
        run_ret("let s: String = fmt_i64(read al, -9223372036854775808); return conv i64 len(as_str(read s));"),
        20 // '-' + 19 digits
    );
}

#[test]
fn fmt_i64_negative_bytes_traced() {
    // Observe the produced String byte-by-byte: -42 -> '-','4','2' == 45,52,50.
    let bytes = run_trace(
        "let s: String = fmt_i64(read al, 0 - 42); let v: str = as_str(read s); \
         let mut i: usize = 0; while i < len(v) { trace(conv i64 v[i]); i = i + 1; } return 0;",
    );
    assert_eq!(bytes, vec![45, 52, 50]);
}

// ---- Show: the value-rendering convention ----------------------------------

#[test]
fn show_leaf_renders_via_fmt_i64() {
    // The `ShowInt` witness delegates to `fmt_i64`.
    assert_eq!(
        run_ret("let x: ShowInt = ShowInt { val: 7 }; let s: String = x.to_string(al); if as_str(read s) == \"7\" { return 1; } return 0;"),
        1
    );
}

#[test]
fn show_composes_through_bound_some() {
    // `impl[T: Show] Show for Opt[T]` renders "Some(<inner>)" by calling the
    // payload's own `to_string` — composition through the `T: Show` bound.
    assert_eq!(
        run_ret("let x: Opt[ShowInt] = Opt::Some(ShowInt { val: 42 }); let s: String = x.to_string(al); if as_str(read s) == \"Some(42)\" { return 1; } return 0;"),
        1
    );
}

#[test]
fn show_composes_through_bound_none() {
    assert_eq!(
        run_ret("let x: Opt[ShowInt] = Opt::None; let s: String = x.to_string(al); if as_str(read s) == \"None\" { return 1; } return 0;"),
        1
    );
}

#[test]
fn show_through_generic_fn_bound() {
    // A def-site `T: Show` bound: `show_it` calls `to_string` on an opaque `T`,
    // resolved only through the bound (§2.1). Nested `Opt[ShowInt]` exercises the
    // bound composing twice.
    assert_eq!(
        run_ret("let x: ShowInt = ShowInt { val: 5 }; let s: String = show_it(read x, al); if as_str(read s) == \"5\" { return 1; } return 0;"),
        1
    );
    assert_eq!(
        run_ret("let x: Opt[ShowInt] = Opt::Some(ShowInt { val: 9 }); let s: String = show_it(read x, al); if as_str(read s) == \"Some(9)\" { return 1; } return 0;"),
        1
    );
}
