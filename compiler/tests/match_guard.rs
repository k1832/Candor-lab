//! Guarded `match` arms (design 0001 §8.2, extended): an optional `if EXPR`
//! between a pattern and its `=>`, applying to every pattern kind. A guarded arm
//! fires only when the pattern matches AND the guard is true; a false guard falls
//! through to test the later arms. These gates cover: functional dispatch of int
//! and enum guarded matches byte-exact across the oracle / MIR / native no-opt /
//! opt engines, including guard-false fall-through and two same-pattern guarded
//! arms; the exhaustiveness rule (a guarded catch-all/covering arm does NOT make
//! a match exhaustive); and guard typing/scope (a non-bool guard is rejected, a
//! pattern binding resolves inside the guard).

use candor::{
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

// ---- functional: integer guards, incl. guard-false fall-through, all engines ----

#[test]
fn int_guard_dispatch_with_fallthrough() {
    let src = "fn classify(n: i64) -> i64 { \
        return match n { \
            x if x < 0 => 1, \
            0 => 2, \
            x if x > 100 => 3, \
            _ => 4 \
        }; } \
        fn main() -> i64 { \
        trace(classify(0i64 - 5i64)); trace(classify(0)); \
        trace(classify(500)); trace(classify(50)); \
        return classify(0i64 - 5i64) + classify(0) + classify(500) + classify(50); }";
    let o = assert_four_engines(src);
    assert_eq!(o.ret, 10); // 1 + 2 + 3 + 4
    assert_eq!(o.trace, vec![1, 2, 3, 4]);
}

// ---- functional: two guarded arms, SAME pattern, first fails then second fires ----

#[test]
fn two_same_pattern_guards_fallthrough_and_retry() {
    let src = "fn pick(n: i64) -> i64 { \
        return match n { \
            7 if n > 10 => 100, \
            7 if n == 7 => 200, \
            _ => 0 \
        }; } \
        fn main() -> i64 { return pick(7) + pick(3); }";
    let o = assert_four_engines(src);
    assert_eq!(o.ret, 200); // 7 -> 200 (first guard false, second true), 3 -> 0
}

// ---- functional: enum guards + payload bindings, guard-false Some fall-through ----

#[test]
fn enum_guard_binding_fallthrough() {
    let src = "enum Opt { Some(i64), None } \
        fn describe(o: Opt) -> i64 { \
        return match o { \
            Opt::Some(v) if v > 10 => v * 2, \
            Opt::Some(v) => v, \
            Opt::None => 0 \
        }; } \
        fn main() -> i64 { \
        return describe(Opt::Some(20i64)) + describe(Opt::Some(5i64)) + describe(Opt::None); }";
    let o = assert_four_engines(src);
    assert_eq!(o.ret, 45); // 40 + 5 + 0
}

// ---- functional: the disk fixtures, byte-exact across engines ----

#[test]
fn run_fixture_match_guard_int() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/run/match_guard_int.cnr"
    ))
    .unwrap();
    let o = assert_four_engines(&src);
    assert_eq!(o.ret, 210);
}

#[test]
fn run_fixture_match_guard_enum() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/run/match_guard_enum.cnr"
    ))
    .unwrap();
    let o = assert_four_engines(&src);
    assert_eq!(o.ret, 45);
}

// ---- exhaustiveness: a guarded catch-all does NOT make an int match exhaustive ----

#[test]
fn int_guarded_catchall_is_non_exhaustive() {
    let src = "fn f(n: i64) -> i64 { return match n { x if x > 0 => 1, _ if false => 2 }; } \
        fn main() -> i64 { return f(1); }";
    assert_eq!(check_codes(src), vec!["E0601"]);
}

#[test]
fn int_guarded_catchall_plus_unguarded_checks_clean() {
    let src = "fn f(n: i64) -> i64 { return match n { x if x > 0 => 1, _ => 2 }; } \
        fn main() -> i64 { return f(1); }";
    assert!(matches!(run_source_real(src), RunResult::Ok(_)));
}

// ---- exhaustiveness: an enum variant only covered by a GUARDED arm is non-exhaustive ----

#[test]
fn enum_guarded_only_variant_is_non_exhaustive() {
    let src = "enum Opt { Some(i64), None } \
        fn f(o: Opt) -> i64 { return match o { Opt::Some(v) => v, Opt::None if false => 0 }; } \
        fn main() -> i64 { return f(Opt::None); }";
    assert_eq!(check_codes(src), vec!["E0601"]);
}

#[test]
fn enum_guarded_variant_plus_unguarded_checks_clean() {
    let src = "enum Opt { Some(i64), None } \
        fn f(o: Opt) -> i64 { \
        return match o { Opt::Some(v) if v > 0 => v, Opt::Some(v) => v, Opt::None => 0 }; } \
        fn main() -> i64 { return f(Opt::Some(3i64)); }";
    assert!(matches!(run_source_real(src), RunResult::Ok(_)));
}

// ---- overlap: a GUARDED earlier arm does NOT make a same-pattern later arm dead ----

#[test]
fn guarded_arm_does_not_shadow_later_same_pattern() {
    // Without the guard-aware overlap rule this would flag the second `0` arm as
    // E0602-unreachable. The guard on the first `0` arm keeps the second reachable.
    let src = "fn f(n: i64) -> i64 { return match n { 0 if n == 1 => 1, 0 => 2, _ => 3 }; } \
        fn main() -> i64 { return f(0); }";
    assert!(matches!(run_source_real(src), RunResult::Ok(_)));
    // And an UNGUARDED earlier arm still shadows a later overlapping one.
    let dead = "fn g(n: i64) -> i64 { return match n { 0 => 1, 0 if n == 0 => 2, _ => 3 }; } \
        fn main() -> i64 { return g(0); }";
    assert_eq!(check_codes(dead), vec!["E0602"]);
}

// ---- guard typing/scope ----

#[test]
fn non_bool_guard_rejected() {
    let src = "fn f(n: i64) -> i64 { return match n { x if x => 1, _ => 0 }; } \
        fn main() -> i64 { return f(1); }";
    assert_eq!(check_codes(src), vec!["E0703"]);
}

#[test]
fn binding_in_guard_resolves() {
    // The payload binding `v` referenced inside the guard resolves and types.
    let src = "enum Opt { Some(i64), None } \
        fn f(o: Opt) -> i64 { return match o { Opt::Some(v) if v > 0 => v, _ => 0 }; } \
        fn main() -> i64 { return f(Opt::Some(9i64)); }";
    let o = assert_four_engines(src);
    assert_eq!(o.ret, 9);
}

// ---- the formatter round-trips a guarded arm ----

#[test]
fn formatter_roundtrips_guard() {
    let src = "fn f(n: i64) -> i64 {\n    return match n {\n        x if x < 0 => 1,\n        _ => 0,\n    };\n}\n";
    let once = candor::format_source_real(src).expect("format ok");
    let twice = candor::format_source_real(&once).expect("format idempotent");
    assert_eq!(once, twice, "formatter must be idempotent on guarded arms");
    assert!(once.contains("x if x < 0 =>"), "guard preserved: {once}");
}
