//! Integer-literal `match` patterns (design 0001 §8.2, extended). A literal-arm
//! match over an integer scrutinee compares-and-branches to the matching arm or
//! the required `_`/binding catch-all. These gates cover: functional dispatch
//! (u8 byte + i64/i32) byte-exact across the oracle / MIR / native no-opt / opt
//! engines in both value-producing and statement forms; the exhaustiveness rule
//! (a literal-only int match is rejected without a catch-all); the duplicate-
//! literal diagnostic; and pattern-kind coherence with the scrutinee.

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

// ---- functional: byte (u8) dispatch, value-producing form, all engines ----

#[test]
fn u8_byte_dispatch_value_form() {
    let src = "fn op(b: u8) -> i64 { \
        return match b { 0x41u8 => 10, 0x6au8 => 20, 0x6bu8 => 30, _ => 0 }; } \
        fn main() -> i64 { \
        return op(0x41u8) + op(0x6au8) + op(0x6bu8) + op(0xffu8); }";
    let o = assert_four_engines(src);
    assert_eq!(o.ret, 60); // 10 + 20 + 30 + 0
}

// ---- functional: i64 dispatch, statement form + a binding catch-all ----

#[test]
fn i64_statement_form_with_binding() {
    let src = "fn classify(n: i64) -> i64 { \
        let mut r: i64 = 0; \
        match n { 0 => { r = 1; }, 10 => { r = 100; }, v => { r = v * 2; } } \
        return r; } \
        fn main() -> i64 { \
        trace(classify(0)); trace(classify(10)); trace(classify(7)); \
        return classify(0) + classify(10) + classify(7); }";
    let o = assert_four_engines(src);
    assert_eq!(o.ret, 115); // 1 + 100 + 14
    assert_eq!(o.trace, vec![1, 100, 14]);
}

// ---- functional: i32 scrutinee incl. a negative-literal pattern ----

#[test]
fn i32_dispatch_with_negative_literal() {
    let src = "fn sign(n: i32) -> i64 { \
        return match n { -1i32 => 100, 0i32 => 200, 1i32 => 300, _ => 999 }; } \
        fn main() -> i64 { \
        return sign(0i32 - 1i32) + sign(0i32) + sign(1i32) + sign(42i32); }";
    let o = assert_four_engines(src);
    assert_eq!(o.ret, 1599); // 100 + 200 + 300 + 999
}

// ---- functional: the disk fixture, byte-exact across engines ----

#[test]
fn run_fixture_match_int() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/run/match_int.cnr"
    ))
    .unwrap();
    let o = assert_four_engines(&src);
    assert_eq!(o.ret, 113);
}

// ---- exhaustiveness: a literal-only int match with no catch-all is rejected ----

#[test]
fn non_exhaustive_int_match_rejected() {
    let src = "fn f(b: u8) -> i64 { return match b { 0x41u8 => 1, 0x6au8 => 2 }; } \
        fn main() -> i64 { return f(0x41u8); }";
    assert_eq!(check_codes(src), vec!["E0601"]);
}

#[test]
fn int_match_with_wildcard_checks_clean() {
    let src = "fn f(b: u8) -> i64 { return match b { 0x41u8 => 1, _ => 2 }; } \
        fn main() -> i64 { return f(0x41u8); }";
    assert!(matches!(run_source_real(src), RunResult::Ok(_)));
}

// ---- duplicate literal is a dead arm ----

#[test]
fn duplicate_literal_arm_rejected() {
    let src = "fn f(b: u8) -> i64 { return match b { 0x41u8 => 1, 0x41u8 => 2, _ => 0 }; } \
        fn main() -> i64 { return f(0x41u8); }";
    assert_eq!(check_codes(src), vec!["E0602"]);
}

// ---- coherence: literal vs. variant patterns must match the scrutinee kind ----

#[test]
fn int_literal_on_enum_scrutinee_rejected() {
    let src = "enum E { A, B } \
        fn f(e: E) -> i64 { return match e { 0 => 1, _ => 2 }; } \
        fn main() -> i64 { return f(E::A); }";
    assert_eq!(check_codes(src), vec!["E0606"]);
}

#[test]
fn variant_pattern_on_int_scrutinee_rejected() {
    let src = "enum E { A, B } \
        fn f(n: u8) -> i64 { return match n { E::A => 1, _ => 2 }; } \
        fn main() -> i64 { return f(0u8); }";
    assert_eq!(check_codes(src), vec!["E0606"]);
}

#[test]
fn literal_suffix_mismatch_rejected() {
    let src = "fn f(n: i32) -> i64 { return match n { 5u8 => 1, _ => 2 }; } \
        fn main() -> i64 { return f(5i32); }";
    assert_eq!(check_codes(src), vec!["E0606"]);
}

#[test]
fn out_of_range_literal_rejected() {
    let src = "fn f(n: u8) -> i64 { return match n { 300 => 1, _ => 2 }; } \
        fn main() -> i64 { return f(0u8); }";
    assert_eq!(check_codes(src), vec!["E0709"]);
}

// ---- the formatter round-trips an integer-literal pattern ----

#[test]
fn formatter_roundtrips_int_patterns() {
    let src = "fn f(b: u8) -> i64 {\n    return match b {\n        0x41u8 => 1,\n        -5i32 => 2,\n        _ => 0,\n    };\n}\n";
    let once = candor_proto::format_source_real(src).expect("format ok");
    let twice = candor_proto::format_source_real(&once).expect("format idempotent");
    assert_eq!(once, twice, "formatter must be idempotent on int patterns");
    assert!(once.contains("0x41u8") || once.contains("65u8"), "literal preserved: {once}");
    assert!(once.contains("-5i32"), "negative literal preserved: {once}");
}
