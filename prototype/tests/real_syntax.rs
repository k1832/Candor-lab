//! Real-syntax (`.cnr`) front-end tests (design 0006; spec 01/02):
//! a golden suite (check + run) over programs exercising every new construct, a
//! set of parser/checker negatives for the spec's error clauses, and a parity
//! test asserting a program written in both syntaxes runs identically.

use candor_proto::diag::Severity;
use candor_proto::{
    check_source_real, parse_source_real, run_source, run_source_real, RunResult,
};

fn fixture(rel: &str) -> String {
    let path = format!("{}/tests/fixtures/{rel}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn run_real_ret(rel: &str) -> i64 {
    match run_source_real(&fixture(rel)) {
        RunResult::Ok(r) => r.ret,
        RunResult::Fault(f) => panic!("{rel} faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => {
            panic!("{rel} check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("{rel} parse error: {}", d.to_json()),
    }
}

/// The error codes a real-syntax program reports (parse error surfaces as one).
fn real_error_codes(src: &str) -> Vec<String> {
    match check_source_real(src) {
        Ok(diags) => diags
            .into_iter()
            .filter(|d| d.severity == Severity::Error)
            .map(|d| d.code)
            .collect(),
        Err(parse) => vec![parse.code],
    }
}

// ---- golden suite: check clean + run to the asserted sentinel ----

#[test]
fn golden_bits_checks_and_runs() {
    let src = fixture("real/bits.cnr");
    assert!(check_source_real(&src).unwrap().is_empty(), "bits.cnr should check clean");
    assert_eq!(run_real_ret("real/bits.cnr"), 781);
}

#[test]
fn golden_propagate_checks_and_runs() {
    let src = fixture("real/propagate.cnr");
    assert!(
        check_source_real(&src).unwrap().is_empty(),
        "propagate.cnr should check clean, got {:?}",
        check_source_real(&src).unwrap()
    );
    assert_eq!(run_real_ret("real/propagate.cnr"), 60);
}

#[test]
fn golden_slices_checks_and_runs() {
    let src = fixture("real/slices.cnr");
    assert!(check_source_real(&src).unwrap().is_empty(), "slices.cnr should check clean");
    assert_eq!(run_real_ret("real/slices.cnr"), 34);
}

// ---- the golden programs parse into the shared AST ----

#[test]
fn golden_programs_parse() {
    for f in ["real/bits.cnr", "real/propagate.cnr", "real/slices.cnr"] {
        let p = parse_source_real(&fixture(f)).unwrap_or_else(|d| panic!("{f}: {}", d.to_json()));
        assert!(!p.items.is_empty());
    }
}

// ---- negatives for the spec's error clauses ----

#[test]
fn neg_bare_over_range_literal_is_compile_error() {
    // spec 01 §3.3: a bare unsigned over-range literal is rejected at compile
    // time (never a runtime fault).
    let codes = real_error_codes("fn main() -> i64 { let x: u64 = 9223372036854775808; return 0; }");
    assert!(codes.contains(&"E0709".to_string()), "expected E0709, got {codes:?}");
}

#[test]
fn neg_conv_non_scalar_target_is_parse_error() {
    // design 0006 §2.4 / spec 02 §6.4: the `conv` target must be a scalar keyword.
    let codes = real_error_codes("struct P { x: i64 } fn main() -> i64 { let y: i64 = conv P 3; return 0; }");
    assert!(codes.contains(&"P0007".to_string()), "expected P0007, got {codes:?}");
}

#[test]
fn neg_write_through_borrow_without_explicit_deref() {
    // spec 02 §6.3: a write through a borrow must keep the deref explicit (`.*`).
    let src = "struct P { x: i64 } fn f(p: write P) -> unit { p.x = 5; } fn main() -> i64 { return 0; }";
    let codes = real_error_codes(src);
    assert!(codes.contains(&"E0713".to_string()), "expected E0713, got {codes:?}");
}

#[test]
fn neg_write_through_borrow_with_explicit_deref_is_ok() {
    let src = "struct P { x: i64 } fn f(p: write P) -> unit { p.*.x = 5; } fn main() -> i64 { return 0; }";
    let codes = real_error_codes(src);
    assert!(!codes.contains(&"E0713".to_string()), "explicit `.*` write must not error: {codes:?}");
}

#[test]
fn neg_more_than_one_ok_marker() {
    // spec 02 §2.2: at most one variant may be `ok`-marked.
    let src = "enum E { ok A(i64), ok B(i64) } fn main() -> i64 { return 0; }";
    let codes = real_error_codes(src);
    assert!(codes.contains(&"E0109".to_string()), "expected E0109, got {codes:?}");
}

#[test]
fn neg_comparison_is_non_associative() {
    // spec 02 §6.1: `a < b < c` is a parse error.
    let codes = real_error_codes("fn main() -> i64 { let x: bool = 1 < 2 < 3; return 0; }");
    assert!(codes.contains(&"P0006".to_string()), "expected P0006, got {codes:?}");
}

#[test]
fn neg_conv_constant_loss_is_error_but_folds_in_regime() {
    // design 0006 §2.4: constant-known `conv` loss is a compile error in the
    // default regime, but folds inside a `wrapping`/`saturating` block.
    let bad = real_error_codes("fn main() -> i64 { let x: u8 = conv u8 300; return 0; }");
    assert!(bad.contains(&"E0710".to_string()), "expected E0710, got {bad:?}");
    let good = real_error_codes(
        "fn main() -> i64 { let mut r: u8 = 0; wrapping { r = conv u8 300; } return 0; }",
    );
    assert!(!good.contains(&"E0710".to_string()), "regime fold must not error: {good:?}");
}

// ---- oversize shift faults at runtime (default regime) ----

#[test]
fn oversize_shift_faults() {
    let src = "fn main() -> i64 { let n: i64 = 64; return 1 << n; }";
    match run_source_real(src) {
        RunResult::Fault(_) => {}
        other => panic!("expected a shift fault, got a different outcome: {:?}", matches_kind(&other)),
    }
}

fn matches_kind(r: &RunResult) -> &'static str {
    match r {
        RunResult::Ok(_) => "ok",
        RunResult::Fault(_) => "fault",
        RunResult::CheckErrors(_) => "check-errors",
        RunResult::ParseError(_) => "parse-error",
    }
}

// ---- parity: the same program in both syntaxes runs identically ----

#[test]
fn parity_same_result_both_syntaxes() {
    let throwaway = match run_source(&fixture("parity/point.cn")) {
        RunResult::Ok(r) => r.ret,
        other => panic!("throwaway parity run failed: {}", matches_kind(&other)),
    };
    let real = run_real_ret("parity/point.cnr");
    assert_eq!(throwaway, real, "parity mismatch: .cn={throwaway} .cnr={real}");
    assert_eq!(real, 42);
}
