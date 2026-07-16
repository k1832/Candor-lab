//! Design 0011/0013 formatting foundation: `fmt_i64` (decimal rendering of a
//! signed 64-bit integer, total including `i64::MIN`) and the `Show` value-
//! rendering convention (a 0009-style interface composing through a `T: Show`
//! bound). The corelib source is the self-contained image
//! `fixtures/std_fmt.cnr` (the `corelib_flat` pattern; `String` std is single-
//! file because it is a MIR-interp-only CollectionOp, excluded from the backend
//! corpus — see that file's header). These tests append a `main` to those
//! definitions and OBSERVE the produced `String` at runtime.

use candor::diag::Severity;
use candor::{check_source_real, run_source_real, RunResult};

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

// ---- fmt_f64: decimal rendering of an f64 (design 0016 float formatting) -------

// Build `fmt_f64(al, <lit>)` and compare its `as_str` view against `expect`.
fn fmt_f64_eq(lit: &str, expect: &str) -> i64 {
    run_ret(&format!(
        "let s: String = fmt_f64(read al, {lit}); if as_str(read s) == \"{expect}\" {{ return 1; }} return 0;"
    ))
}

// Reconstruct the exact string `fmt_f64` produces for `lit` by tracing its bytes.
fn fmt_f64_str(lit: &str) -> String {
    let bytes = run_trace(&format!(
        "let s: String = fmt_f64(read al, {lit}); let v: str = as_str(read s); \
         let mut i: usize = 0; while i < len(v) {{ trace(conv i64 v[i]); i = i + 1; }} return 0;"
    ));
    bytes.iter().map(|&b| b as u8 as char).collect()
}

#[test]
fn fmt_f64_known_values() {
    // Byte-exact strings for the documented format (fixed-point, <=15 sig digits,
    // trailing zeros stripped; specials NaN/inf/-inf/0).
    for (lit, expect) in [
        ("1.5", "1.5"),
        ("-2.25", "-2.25"),
        ("0.0", "0"),
        ("10.0", "10"),
        ("3.14159", "3.14159"),
        ("100.5", "100.5"),
        ("0.1", "0.1"),
        ("-0.001", "-0.001"),
        ("-3.5", "-3.5"),
        ("42.0", "42"),
        ("1e10", "10000000000"),
        ("1e-7", "0.0000001"),
        ("123456.789012345", "123456.789012345"),
        ("6.022e23", "602200000000000000000000"),
        ("1.0 / 0.0", "inf"),
        ("(-1.0) / 0.0", "-inf"),
        ("0.0 / 0.0", "NaN"),
        // -0.0 collapses to "0" (the sign of zero is dropped, documented).
        ("-0.0", "0"),
    ] {
        assert_eq!(fmt_f64_eq(lit, expect), 1, "fmt_f64({lit}) should be {expect:?}");
    }
}

#[test]
fn fmt_f64_round_trips_covered_range() {
    // The correctness anti-cheat: format via the Candor fmt_f64, parse the string
    // back with Rust `f64::from_str`, and assert identical bits. Covers the
    // documented guaranteed domain: <=15 significant digits, magnitude in
    // [1e-15, 1e39) -- small, large, fractional, negative, both exponent ends.
    let lits = [
        "1.5", "-2.25", "0.0", "10.0", "3.14159", "100.5", "0.1", "-0.001",
        "1e10", "1e-7", "123456.789012345", "6.022e23", "0.5", "0.25", "-7.125",
        "42.0", "1000000.0", "0.000123", "2.5e-14", "9.87e30",
        "999999999999999.0", "-3.5", "5e38", "5e-15", "-0.0009765625",
    ];
    for lit in lits {
        // Parse the same literal in Rust for the expected bits (both use IEEE
        // round-to-nearest, so Candor and Rust agree on the literal's f64).
        let expected: f64 = lit.replace(' ', "").parse().expect("test literal parses");
        let s = fmt_f64_str(lit);
        let parsed: f64 = s.parse().unwrap_or_else(|e| panic!("fmt_f64({lit}) = {s:?} did not parse: {e}"));
        assert_eq!(
            parsed.to_bits(),
            expected.to_bits(),
            "fmt_f64({lit}) = {s:?} parsed to {parsed} (bits differ from {expected})"
        );
    }
}

#[test]
fn fmt_f64_high_precision_values_are_documented_15_digits() {
    // The documented LIMIT: values needing 16-17 significant digits (a 17th correct
    // digit is beyond f64's exact 2^53 integer range with f64-only arithmetic) are
    // rendered to 15 significant digits and therefore do NOT round-trip. Pinning
    // the exact output keeps this behaviour honest and observed, not hand-waved.
    assert_eq!(fmt_f64_str("2.0 / 3.0"), "0.666666666666667");
    // 0.666666666666667 parses to a DIFFERENT f64 than the true 2.0/3.0.
    let two_thirds = 2.0f64 / 3.0f64;
    let parsed: f64 = "0.666666666666667".parse().unwrap();
    assert_ne!(parsed.to_bits(), two_thirds.to_bits());
}

#[test]
fn show_float_renders_via_fmt_f64() {
    // `ShowFloat` witness delegates to `fmt_f64` (parallels `ShowInt`).
    assert_eq!(
        run_ret("let x: ShowFloat = ShowFloat { val: 1.5 }; let s: String = x.to_string(al); if as_str(read s) == \"1.5\" { return 1; } return 0;"),
        1
    );
}

#[test]
fn show_float_composes_through_bound() {
    // `impl[T: Show] Show for Opt[T]` renders "Some(<f64>)" through the T: Show bound.
    assert_eq!(
        run_ret("let x: Opt[ShowFloat] = Opt::Some(ShowFloat { val: -2.25 }); let s: String = x.to_string(al); if as_str(read s) == \"Some(-2.25)\" { return 1; } return 0;"),
        1
    );
}

// ---- formatter must NOT collapse `read <write-borrow>.*` (semantics gate) -----

#[test]
fn fmt_preserves_read_reborrow_of_write_borrow() {
    // `read (c.*)` over a `write c` borrow is a non-moving read-reborrow, while
    // bare `c` MOVES the write borrow (a later use then fails E0301). Lacking a
    // type table, the formatter cannot tell a read borrow (collapse safe) from a
    // write borrow (collapse unsound), so it must keep `read c.*` verbatim -- only
    // the redundant parens drop. This is the exact case the old reborrow collapse
    // silently broke.
    let src = "\
struct Cell { val: i64 }
fn peek(c: read Cell) -> i64 {
    return c.val;
}
fn bump(c: write Cell) -> i64 {
    let x: i64 = peek(read c.*);
    c.*.val = x + 1;
    return c.*.val;
}
fn main() -> i64 {
    let mut cell: Cell = Cell { val: 5 };
    return bump(write cell);
}
";
    let formatted = candor::format_source_real(src).expect("format ok");
    assert!(
        formatted.contains("peek(read c.*)"),
        "reborrow of a write borrow must not collapse to a move:\n{formatted}"
    );
    // Idempotent, and the formatted output still type-checks and runs correctly.
    let twice = candor::format_source_real(&formatted).expect("format idempotent");
    assert_eq!(twice, formatted, "formatter must be idempotent");
    match run_source_real(&formatted) {
        RunResult::Ok(r) => assert_eq!(r.ret, 6, "formatted output must run correctly"),
        RunResult::Fault(f) => panic!("formatted output faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!(
            "formatted output failed to check: {:?}",
            d.iter().map(|x| &x.code).collect::<Vec<_>>()
        ),
        RunResult::ParseError(d) => panic!("formatted output parse error: {}", d.to_json()),
    }
}
