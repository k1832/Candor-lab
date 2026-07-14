//! Integer-range `match` patterns (design 0001 §8.2, extended). A range arm
//! (`lo..=hi` inclusive, `lo..hi` half-open) over an integer scrutinee lowers to
//! a bounds test (`lo <= x` then `x <= hi` / `x < hi`) branching to the arm body,
//! reusing only existing MIR ops. These gates cover: functional dispatch mixing
//! literals, ranges, and a `_`/binding catch-all — byte-exact across the oracle /
//! MIR / native no-opt / opt engines in value-producing and statement forms, at
//! range edges and interiors — plus the diagnostics: `lo > hi` (E0715),
//! overlapping arms (E0602: literal-in-range, range-in-range), and the
//! non-exhaustive rejection of a range-only match with no catch-all (E0601).

use candor_proto::{
    run_source_real, run_source_real_mir, run_source_real_native, run_source_real_native_opt,
    MirRunResult, RunResult,
};

#[derive(Debug, PartialEq, Eq)]
struct Obs {
    ret: i64,
    trace: Vec<i64>,
}

fn oracle(src: &str) -> Obs {
    match run_source_real(src) {
        RunResult::Ok(r) => Obs { ret: r.ret, trace: r.trace },
        other => panic!("oracle did not run: {other:?}", other = discl(&other)),
    }
}

fn discl(r: &RunResult) -> String {
    match r {
        RunResult::Ok(_) => "ok".into(),
        RunResult::Fault(f) => format!("fault {}", f.to_json()),
        RunResult::CheckErrors(d) => {
            format!("check errors {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => format!("parse error {}", d.to_json()),
    }
}

fn mir_obs(r: MirRunResult) -> Obs {
    match r {
        MirRunResult::Ok(run) => Obs { ret: run.ret, trace: run.trace },
        MirRunResult::Unsupported(e) => panic!("MIR-engine reported out-of-subset: {e}"),
        MirRunResult::Fault(f) => panic!("MIR-engine faulted: {}", f.to_json()),
        MirRunResult::CheckErrors(d) => {
            panic!("MIR check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        MirRunResult::ParseError(d) => panic!("MIR parse error: {}", d.to_json()),
    }
}

/// Assert the oracle, MIR, native no-opt, and native-opt engines all agree.
fn assert_four_engines(src: &str) -> Obs {
    let o = oracle(src);
    for (label, r) in [
        ("mir", run_source_real_mir(src)),
        ("native-noopt", run_source_real_native(src)),
        ("native-opt", run_source_real_native_opt(src)),
    ] {
        assert_eq!(mir_obs(r), o, "{label} diverged from oracle for:\n{src}");
    }
    o
}

fn check_codes(src: &str) -> Vec<String> {
    match run_source_real(src) {
        RunResult::CheckErrors(d) => d.iter().map(|x| x.code.clone()).collect(),
        other => panic!("expected check errors, got {}", discl(&other)),
    }
}

// ---- functional: mixed literal + inclusive-range + `_`, u8, all engines ----

#[test]
fn u8_mixed_literal_range_wildcard() {
    // A WASM-style contiguous opcode group as an inclusive range, flanked by
    // single-literal arms and a `_` fall-through. Probe below, at both edges,
    // interior, above the range, and the isolated literals.
    let src = "fn op(b: u8) -> i64 { \
        return match b { \
            0x00u8 => 1, 0x28u8..=0x3eu8 => 2, 0x41u8 => 3, _ => 0 }; } \
        fn main() -> i64 { \
        return op(0x00u8) + op(0x27u8) + op(0x28u8) + op(0x3eu8) \
             + op(0x30u8) + op(0x3fu8) + op(0x41u8) + op(0xffu8); }";
    let o = assert_four_engines(src);
    // 0x00->1, 0x27 (just below)->0, 0x28 edge->2, 0x3e edge->2,
    // 0x30 interior->2, 0x3f (just above)->0, 0x41->3, 0xff->0 = 10
    assert_eq!(o.ret, 10);
}

// ---- functional: i64 half-open + inclusive + binding, statement form ----

#[test]
fn i64_half_open_and_inclusive_statement_form() {
    let src = "fn classify(n: i64) -> i64 { \
        let mut r: i64 = 0; \
        match n { 0..10 => { r = 1; }, 10..=20 => { r = 2; }, v => { r = v; } } \
        return r; } \
        fn main() -> i64 { \
        trace(classify(0)); trace(classify(9)); trace(classify(10)); \
        trace(classify(20)); trace(classify(21)); \
        return classify(0) + classify(9) + classify(10) + classify(20) + classify(21); }";
    let o = assert_four_engines(src);
    // 0->[0,10)->1, 9->1, 10->[10,20]->2, 20->2, 21->binding->21 = 27
    assert_eq!(o.ret, 27);
    assert_eq!(o.trace, vec![1, 1, 2, 2, 21]);
}

// ---- functional: negative-bounded range over a signed scrutinee ----

#[test]
fn i32_negative_range() {
    let src = "fn f(n: i32) -> i64 { \
        return match n { -5i32..=-1i32 => 1, 0i32 => 2, 1i32..=5i32 => 3, _ => 0 }; } \
        fn main() -> i64 { \
        return f(0i32 - 5i32) + f(0i32 - 1i32) + f(0i32) + f(1i32) + f(5i32) + f(6i32); }";
    let o = assert_four_engines(src);
    // -5->1, -1->1, 0->2, 1->3, 5->3, 6->0 = 10
    assert_eq!(o.ret, 10);
}

// ---- functional: the disk fixture, byte-exact across engines ----

#[test]
fn run_fixture_match_range() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/run/match_range.cnr"
    ))
    .unwrap();
    let o = assert_four_engines(&src);
    assert_eq!(o.ret, 116);
}

// ---- diagnostics: `lo > hi` (and empty half-open) rejected ----

#[test]
fn inclusive_lo_gt_hi_rejected() {
    let src = "fn f(b: u8) -> i64 { return match b { 5u8..=1u8 => 1, _ => 0 }; } \
        fn main() -> i64 { return f(0u8); }";
    assert_eq!(check_codes(src), vec!["E0715"]);
}

#[test]
fn half_open_empty_rejected() {
    // `5..5` is empty (matches nothing); requiring `lo < hi` flags it.
    let src = "fn f(b: u8) -> i64 { return match b { 5u8..5u8 => 1, _ => 0 }; } \
        fn main() -> i64 { return f(0u8); }";
    assert_eq!(check_codes(src), vec!["E0715"]);
}

// ---- diagnostics: overlapping arms are unreachable (E0602) ----

#[test]
fn overlapping_ranges_rejected() {
    let src = "fn f(b: u8) -> i64 { return match b { 0u8..=10u8 => 1, 5u8..=15u8 => 2, _ => 0 }; } \
        fn main() -> i64 { return f(0u8); }";
    assert_eq!(check_codes(src), vec!["E0602"]);
}

#[test]
fn literal_inside_prior_range_rejected() {
    let src = "fn f(b: u8) -> i64 { return match b { 0u8..=10u8 => 1, 5u8 => 2, _ => 0 }; } \
        fn main() -> i64 { return f(0u8); }";
    assert_eq!(check_codes(src), vec!["E0602"]);
}

#[test]
fn range_covering_prior_literal_rejected() {
    let src = "fn f(b: u8) -> i64 { return match b { 5u8 => 1, 0u8..=10u8 => 2, _ => 0 }; } \
        fn main() -> i64 { return f(0u8); }";
    assert_eq!(check_codes(src), vec!["E0602"]);
}

#[test]
fn adjacent_ranges_are_not_overlap() {
    // [0,9] and [10,20] touch but do not overlap — no false-positive E0602.
    let src = "fn f(b: u8) -> i64 { return match b { 0u8..=9u8 => 1, 10u8..=20u8 => 2, _ => 0 }; } \
        fn main() -> i64 { return f(0u8); }";
    assert!(matches!(run_source_real(src), RunResult::Ok(_)));
}

// ---- exhaustiveness: a range-only match with no catch-all is non-exhaustive ----

#[test]
fn non_exhaustive_range_match_rejected() {
    let src = "fn f(b: u8) -> i64 { return match b { 0u8..=10u8 => 1, 20u8..=30u8 => 2 }; } \
        fn main() -> i64 { return f(0u8); }";
    assert_eq!(check_codes(src), vec!["E0601"]);
}

// ---- diagnostics: an out-of-range endpoint is rejected ----

#[test]
fn out_of_range_endpoint_rejected() {
    let src = "fn f(b: u8) -> i64 { return match b { 0u8..=300u8 => 1, _ => 0 }; } \
        fn main() -> i64 { return f(0u8); }";
    assert_eq!(check_codes(src), vec!["E0709"]);
}

// ---- the formatter round-trips a range pattern ----

#[test]
fn formatter_roundtrips_range_patterns() {
    let src = "fn f(b: u8) -> i64 {\n    return match b {\n        0x28u8..=0x3eu8 => 1,\n        0u8..10u8 => 2,\n        _ => 0,\n    };\n}\n";
    let once = candor_proto::format_source_real(src).expect("format ok");
    let twice = candor_proto::format_source_real(&once).expect("format idempotent");
    assert_eq!(once, twice, "formatter must be idempotent on range patterns");
    assert!(once.contains("..="), "inclusive range preserved: {once}");
    assert!(once.contains(".."), "half-open range preserved: {once}");
}
