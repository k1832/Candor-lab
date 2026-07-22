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

// ===========================================================================
// Canonical formatter (P16 / NN#11) coverage — differential semantic gates +
// per-construct golden output. House rule (docs/TESTING.md): a soundness gate
// must demonstrably reject the bug it guards; goldens pin the exact byte form.
// The formatter shipped a SEMANTICS-CHANGING bug this cycle (the reborrow
// collapse); the corpus gate below is that bug's standing generalization.
// ===========================================================================

use candor::{run_source_real_mir, MirRunResult};

// ---- differential comparators (the gate's teeth) ---------------------------

// Sorted (severity, code) diagnostic set — the property the reborrow bug broke
// (formatting introduced an E0301 the original never had). Spans are excluded
// because byte offsets legitimately shift under formatting.
fn fmt_diag_set(src: &str) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = check_source_real(src)
        .map(|ds| ds.iter().map(|d| (format!("{:?}", d.severity), d.code.clone())).collect())
        .unwrap_or_else(|d| vec![("Parse".into(), d.code.clone())]);
    v.sort();
    v
}

// Behavioral signature under the tree-walker: return value + trace, or a fault's
// kind + pre-fault trace, or the sorted check/parse error codes.
fn fmt_run_sig(src: &str) -> String {
    match run_source_real(src) {
        RunResult::Ok(r) => format!("ok:{}:{:?}", r.ret, r.trace),
        RunResult::Fault(f) => format!("fault:{:?}:{:?}", f.kind, f.trace),
        RunResult::CheckErrors(d) => {
            let mut c: Vec<_> = d.iter().map(|x| x.code.clone()).collect();
            c.sort();
            format!("check:{c:?}")
        }
        RunResult::ParseError(d) => format!("parse:{}", d.code),
    }
}

// Behavioral signature under the MIR engine. `None` means the program is outside
// the MIR subset (`Unsupported` — a documented coverage boundary, not a
// divergence); callers only compare when BOTH sides are `Some`.
fn fmt_mir_sig(src: &str) -> Option<String> {
    match run_source_real_mir(src) {
        MirRunResult::Ok(r) => Some(format!("ok:{}:{:?}", r.ret, r.trace)),
        MirRunResult::Fault(f) => Some(format!("fault:{:?}:{:?}", f.kind, f.trace)),
        MirRunResult::CheckErrors(d) => {
            let mut c: Vec<_> = d.iter().map(|x| x.code.clone()).collect();
            c.sort();
            Some(format!("check:{c:?}"))
        }
        MirRunResult::ParseError(d) => Some(format!("parse:{}", d.code)),
        MirRunResult::Unsupported(_) => None,
    }
}

// Two sources are behaviorally identical iff they check to the same diagnostics,
// run to the same tree-walker signature, and (when both are MIR-supported) run to
// the same MIR signature. This is exactly the property formatting must preserve.
fn behaves_identically(a: &str, b: &str) -> bool {
    if fmt_diag_set(a) != fmt_diag_set(b) {
        return false;
    }
    if fmt_run_sig(a) != fmt_run_sig(b) {
        return false;
    }
    if let (Some(x), Some(y)) = (fmt_mir_sig(a), fmt_mir_sig(b)) {
        if x != y {
            return false;
        }
    }
    true
}

// The formatter no longer panics on any run-corpus fixture: `emit_expr_bare`
// now handles `ExprKind::FloatLit` (fmt.rs). The set is pinned EMPTY so that a
// newly-crashing fixture trips `corpus_format_panic_set_is_pinned` rather than
// silently regressing. Float fixtures now flow through the full semantic gate.
const FMT_FLOAT_PANIC_FIXTURES: &[&str] = &[];

// No full-tree fixture panics the formatter any more (FloatLit is handled), so
// the whole-tree idempotency scan now checks every float fixture too.
const FMT_FLOAT_PANIC_ANY: &[&str] = &[];

// ---- the load-bearing corpus-wide SEMANTIC gate ----------------------------

#[test]
fn corpus_format_preserves_behavior() {
    // For every runnable `.cnr` fixture: formatting must parse, be idempotent,
    // check to the same diagnostics, and run byte-identically under the tree-
    // walker (and MIR where both support it). This is the reborrow bug's
    // generalization made a standing gate over the whole run corpus.
    let dir = format!("{}/tests/fixtures/run", env!("CARGO_MANIFEST_DIR"));
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().into_owned()))
        .filter(|n| n.ends_with(".cnr"))
        .collect();
    names.sort();

    let mut checked = 0;
    let mut failures = Vec::new();
    for n in &names {
        if FMT_FLOAT_PANIC_FIXTURES.contains(&n.as_str()) {
            continue; // guarded separately by corpus_format_panic_set_is_pinned
        }
        let src = std::fs::read_to_string(format!("{dir}/{n}")).unwrap();
        let formatted = match candor::format_source_real(&src) {
            Ok(f) => f,
            Err(d) => {
                failures.push(format!("{n}: format failed: {}", d.code));
                continue;
            }
        };
        checked += 1;
        if candor::format_source_real(&formatted).unwrap() != formatted {
            failures.push(format!("{n}: not idempotent"));
        }
        if fmt_diag_set(&src) != fmt_diag_set(&formatted) {
            failures.push(format!(
                "{n}: diagnostics changed {:?} -> {:?}",
                fmt_diag_set(&src),
                fmt_diag_set(&formatted)
            ));
        }
        if fmt_run_sig(&src) != fmt_run_sig(&formatted) {
            failures.push(format!(
                "{n}: tree-walker run changed {} -> {}",
                fmt_run_sig(&src),
                fmt_run_sig(&formatted)
            ));
        }
        if let (Some(a), Some(b)) = (fmt_mir_sig(&src), fmt_mir_sig(&formatted)) {
            if a != b {
                failures.push(format!("{n}: MIR run changed {a} -> {b}"));
            }
        }
    }
    assert!(failures.is_empty(), "corpus behavior drift:\n{}", failures.join("\n"));
    assert!(checked >= 25, "expected a substantial corpus, only checked {checked}");
}

// ---- corpus-wide idempotency over the whole fixture tree -------------------

#[test]
fn corpus_format_is_idempotent_and_reparses() {
    // Every `.cnr` across the fixture tree (check/, run/, corelib/, http/, net/,
    // selfhost/, wasm/, ...) that parses as real syntax must format idempotently
    // and re-parse. The float-literal fixtures are pinned separately.
    let root = format!("{}/tests/fixtures", env!("CARGO_MANIFEST_DIR"));
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().map(|x| x == "cnr").unwrap_or(false) {
                    out.push(p);
                }
            }
        }
    }
    let mut files = Vec::new();
    walk(std::path::Path::new(&root), &mut files);
    files.sort();

    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let mut checked = 0;
    let mut failures = Vec::new();
    for p in &files {
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        let src = std::fs::read_to_string(p).unwrap();
        let panicked = std::panic::catch_unwind(|| candor::format_source_real(&src)).is_err();
        if panicked {
            if !FMT_FLOAT_PANIC_ANY.contains(&name.as_str()) {
                failures.push(format!("{}: UNEXPECTED format panic", p.display()));
            }
            continue;
        }
        let formatted = match candor::format_source_real(&src) {
            Ok(f) => f,
            Err(_) => continue, // not real syntax (e.g. throwaway-only or neg fixture)
        };
        checked += 1;
        match candor::format_source_real(&formatted) {
            Ok(twice) if twice == formatted => {}
            Ok(_) => failures.push(format!("{}: not idempotent", p.display())),
            Err(d) => failures.push(format!("{}: formatted output fails to reparse: {}", p.display(), d.code)),
        }
    }
    std::panic::set_hook(prev);
    assert!(failures.is_empty(), "idempotency drift:\n{}", failures.join("\n"));
    assert!(checked >= 100, "expected a broad corpus, only formatted {checked}");
}

#[test]
fn corpus_format_panic_set_is_pinned() {
    // Pin the KNOWN float-literal panic set exactly. If a listed fixture stops
    // panicking (bug fixed -> remove it here) or a new fixture starts panicking,
    // this fails and forces attention rather than silently hiding the bug.
    let dir = format!("{}/tests/fixtures/run", env!("CARGO_MANIFEST_DIR"));
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let mut actual: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().into_owned()))
        .filter(|n| n.ends_with(".cnr"))
        .filter(|n| {
            let src = std::fs::read_to_string(format!("{dir}/{n}")).unwrap();
            std::panic::catch_unwind(|| candor::format_source_real(&src)).is_err()
        })
        .collect();
    std::panic::set_hook(prev);
    actual.sort();
    let mut expected: Vec<String> = FMT_FLOAT_PANIC_FIXTURES.iter().map(|s| s.to_string()).collect();
    expected.sort();
    assert_eq!(actual, expected, "run-corpus formatter panic set drifted");
}

// ---- sensitivity: the gate must REJECT behavior-altering output ------------

const REBORROW_SRC: &str = "\
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

#[test]
fn semantic_gate_rejects_behavior_altering_format() {
    // Anti-cheat (docs/TESTING.md): prove the comparator can fire. Feeding it a
    // deliberately behavior-altered "formatted" output must be REJECTED.
    assert!(behaves_identically(REBORROW_SRC, REBORROW_SRC), "identical inputs must match");

    // A diagnostic-altering "format" (an invalid field access) must be rejected:
    // the diagnostic set diverges (clean -> E0107).
    let diag_altered = REBORROW_SRC.replace("return c.val;", "return c.nope;");
    assert_ne!(fmt_diag_set(REBORROW_SRC), fmt_diag_set(&diag_altered));
    assert!(!behaves_identically(REBORROW_SRC, &diag_altered), "gate must reject a diagnostic change");

    // A run-value-altering "format" must be rejected: bump returns 6 -> 10.
    let run_altered = REBORROW_SRC.replace("val: 5", "val: 9");
    assert_ne!(fmt_run_sig(REBORROW_SRC), fmt_run_sig(&run_altered));
    assert!(!behaves_identically(REBORROW_SRC, &run_altered), "gate must reject a changed run value");
}

// ---- targeted per-construct golden output ----------------------------------

// Assert the exact canonical bytes AND idempotency for `src`.
fn golden(src: &str, expected: &str) {
    let out = candor::format_source_real(src).unwrap_or_else(|d| panic!("format failed: {}", d.to_json()));
    assert_eq!(out, expected, "golden mismatch");
    let twice = candor::format_source_real(&out).unwrap_or_else(|d| panic!("reparse failed: {}", d.to_json()));
    assert_eq!(twice, out, "not idempotent");
}

#[test]
fn golden_struct_enum_s6() {
    golden("struct  Point{ y:i64,x:i64 }", "struct Point {\n    y: i64,\n    x: i64,\n}\n");
    golden("struct  E{}", "struct E {}\n");
    golden("enum Color{Red,Green(i64),ok Ok}", "enum Color {\n    Red,\n    Green(i64),\n    ok Ok,\n}\n");
    golden("enum Void {}", "enum Void {}\n");
}

#[test]
fn golden_drop_hook() {
    golden(
        "struct R { h: i64 } drop(write self) { trace(self.h); }",
        "struct R {\n    h: i64,\n} drop(write self) {\n    trace(self.h);\n}\n",
    );
}

#[test]
fn golden_generics_bounds_and_region_order_s4() {
    golden("fn id[T: Show + Clone](x: T) -> T { return x; }", "fn id[T: Show + Clone](x: T) -> T {\n    return x;\n}\n");
    // S4: region parameters are reordered before type parameters.
    golden("fn f[T, region r](x: read[r] T) -> read[r] T { return x; }", "fn f[region r, T](x: read[r] T) -> read[r] T {\n    return x;\n}\n");
}

#[test]
fn golden_interface_and_impl_assoc() {
    golden(
        "interface Iter { type Item; fn next(write self) alloc -> Opt[i64]; }",
        "interface Iter {\n    type Item;\n    fn next(write self) alloc -> Opt[i64];\n}\n",
    );
    golden(
        "impl[T] Iter for Vec[T] { type Item = T; fn next(write self) -> i64 { return 0; } }",
        "impl[T] Iter for Vec[T] {\n    type Item = T;\n    fn next(write self) -> i64 {\n        return 0;\n    }\n}\n",
    );
    golden(
        "impl Show[i64] for Foo { fn to_string(read self, al: Alloc) alloc -> String { return mk(al); } }",
        "impl Show[i64] for Foo {\n    fn to_string(read self, al: Alloc) alloc -> String {\n        return mk(al);\n    }\n}\n",
    );
}

#[test]
fn golden_extern_and_trust_s5() {
    // S5: an extern block's members are implicitly `foreign` (no marker emitted).
    golden(
        "extern \"C\" { fn puts(s: rawptr u8) -> i32; fn abort(); }",
        "extern \"C\" {\n    fn puts(s: rawptr u8) -> i32;\n    fn abort();\n}\n",
    );
    golden(
        "extern \"C\" { fn raw() -> i32 trust \"vetted\" { nonnull(x), aligned }; }",
        "extern \"C\" {\n    fn raw() -> i32\n        trust \"vetted\" {\n            nonnull(x),\n            aligned,\n        };\n}\n",
    );
    // S5: the `foreign` effect on a free fn IS kept.
    golden("fn c_puts(s: rawptr u8) foreign -> i32 { return 0i32; }", "fn c_puts(s: rawptr u8) foreign -> i32 {\n    return 0i32;\n}\n");
}

#[test]
fn golden_export_static_use() {
    golden("export \"C\" fn add(a: i64, b: i64) -> i64 = my_add;", "export \"C\" fn add(a: i64, b: i64) -> i64 = my_add;\n");
    golden("static N: i64 = 3 + 4;", "static N: i64 = 3 + 4;\n");
    golden("static A: [4]i64 = [1, 2, 3, 4];", "static A: [4]i64 = [1, 2, 3, 4];\n");
    golden("use  a::b::{ C , D };\nfn main() -> i64 { return 0; }", "use a::b::{C, D};\nfn main() -> i64 {\n    return 0;\n}\n");
    golden("use core::mem;\nfn main() -> i64 { return 0; }", "use core::mem;\nfn main() -> i64 {\n    return 0;\n}\n");
}

#[test]
fn golden_sig_tail_order_s3() {
    // S3: markers then contracts, in canonical order regardless of source order.
    golden(
        "fn f(x: i64) ensures(result > 0) foreign requires(x > 0) alloc -> i64 { return x; }",
        "fn f(x: i64) alloc foreign requires(x > 0) ensures(result > 0) -> i64 {\n    return x;\n}\n",
    );
}

#[test]
fn golden_types_zoo() {
    golden(
        "fn t(a: [i64], b: write [u8], c: read Foo, d: rawptr u8, e: Box i64, f: BoxResult i64, g: Map[i64, str], h: fn(i64) alloc -> i64) -> i64 { return 0; }",
        "fn t(a: [i64], b: write [u8], c: read Foo, d: rawptr u8, e: Box i64, f: BoxResult i64, g: Map[i64, str], h: fn(i64) alloc -> i64) -> i64 {\n    return 0;\n}\n",
    );
    golden("fn p(x: T::Item) -> i64 { return 0; }", "fn p(x: T::Item) -> i64 {\n    return 0;\n}\n");
}

#[test]
fn golden_expressions() {
    // conv/bitcast/clone drop the redundant parens around their operand.
    golden(
        "fn c(x: i32) -> i64 { let y: i64 = conv i64 (x); let z: u32 = bitcast u32 (x); let w: i32 = clone (x); return y; }",
        "fn c(x: i32) -> i64 {\n    let y: i64 = conv i64 x;\n    let z: u32 = bitcast u32 x;\n    let w: i32 = clone x;\n    return y;\n}\n",
    );
    golden(
        "fn q(v: Vec[i64], r: read Foo) alloc -> i64 { let a: i64 = foo()?; let b: i64 = r.field; let c: i64 = v[0]; return a; }",
        "fn q(v: Vec[i64], r: read Foo) alloc -> i64 {\n    let a: i64 = foo()?;\n    let b: i64 = r.field;\n    let c: i64 = v[0];\n    return a;\n}\n",
    );
    golden(
        "fn g() -> i64 { let x: Opt[i64] = Opt::Some(3); let y: i64 = size::[i64]; return 0; }",
        "fn g() -> i64 {\n    let x: Opt[i64] = Opt::Some(3);\n    let y: i64 = size::[i64];\n    return 0;\n}\n",
    );
    golden(
        "fn s() -> i64 { let p: Point = Point{x:1,y:2}; let a: [i64] = [1,2,3]; let r: [i64] = [0; 8]; return 0; }",
        "fn s() -> i64 {\n    let p: Point = Point { x: 1, y: 2 };\n    let a: [i64] = [1, 2, 3];\n    let r: [i64] = [0; 8];\n    return 0;\n}\n",
    );
    golden(
        "fn pi(p: rawptr u8) -> i64 { let a: rawptr i64 = cast_ptr[i64](p); let b: usize = offsetof(Foo, bar); let c: rawptr i64 = field_ptr(p, bar); let dd: usize = sizeof(i64); let e: usize = alignof(i64); let n: rawptr u8 = ptr_null[u8](); return 0; }",
        "fn pi(p: rawptr u8) -> i64 {\n    let a: rawptr i64 = cast_ptr[i64](p);\n    let b: usize = offsetof(Foo, bar);\n    let c: rawptr i64 = field_ptr(p, bar);\n    let dd: usize = sizeof(i64);\n    let e: usize = alignof(i64);\n    let n: rawptr u8 = ptr_null[u8]();\n    return 0;\n}\n",
    );
    golden("fn ap() -> i64 { assert(1 < 2); panic(\"boom\"); return 0; }", "fn ap() -> i64 {\n    assert(1 < 2);\n    panic(\"boom\");\n    return 0;\n}\n");
    golden("fn nb() -> i64 { let b: str = b\"raw\"; let n: i64 = -5; return n; }", "fn nb() -> i64 {\n    let b: str = b\"raw\";\n    let n: i64 = -5;\n    return n;\n}\n");
}

#[test]
fn golden_precedence_paren_removal_9_3() {
    // §9.3: redundant parens dropped; precedence-necessary ones kept.
    golden(
        "fn pp(a: i64, b: i64, c: i64) -> i64 { let x: i64 = (a + b) * c; let y: i64 = a + (b * c); let z: i64 = ((a)); return x + y + z; }",
        "fn pp(a: i64, b: i64, c: i64) -> i64 {\n    let x: i64 = (a + b) * c;\n    let y: i64 = a + b * c;\n    let z: i64 = a;\n    return x + y + z;\n}\n",
    );
}

#[test]
fn golden_control_flow() {
    golden(
        "fn ie(x: i64) -> i64 { if x < 0 { return 0; } else if x == 0 { return 1; } else { return 2; } }",
        "fn ie(x: i64) -> i64 {\n    if x < 0 {\n        return 0;\n    } else if x == 0 {\n        return 1;\n    } else {\n        return 2;\n    }\n}\n",
    );
    golden(
        "fn ls() -> i64 { loop { break; } scope { let x: i64 = 1; } while false { continue; } return 0; }",
        "fn ls() -> i64 {\n    loop {\n        break;\n    }\n    scope {\n        let x: i64 = 1;\n    }\n    while false {\n        continue;\n    }\n    return 0;\n}\n",
    );
    golden(
        "fn fr(v: Vec[i64]) -> i64 { for read x in v { trace(x); } for y in v { trace(y); } return 0; }",
        "fn fr(v: Vec[i64]) -> i64 {\n    for read x in v {\n        trace(x);\n    }\n    for y in v {\n        trace(y);\n    }\n    return 0;\n}\n",
    );
    golden(
        "fn ws(a: i32) -> i64 { wrapping { let b: i32 = a + 1i32; } saturating { let c: i32 = a + 1i32; } return 0; }",
        "fn ws(a: i32) -> i64 {\n    wrapping {\n        let b: i32 = a + 1i32;\n    }\n    saturating {\n        let c: i32 = a + 1i32;\n    }\n    return 0;\n}\n",
    );
    golden(
        "fn u(p: rawptr i64) -> i64 { unsafe \"deref vetted\" { let x: i64 = p.*; } return 0; }",
        "fn u(p: rawptr i64) -> i64 {\n    unsafe \"deref vetted\" {\n        let x: i64 = p.*;\n    }\n    return 0;\n}\n",
    );
}

#[test]
fn golden_match_patterns() {
    golden(
        "fn m(n: i64) -> i64 { return match n { 0 => 1, 1..=5 => 2, 6..10 => 3, x if x > 100 => 4, _ => 0 }; }",
        "fn m(n: i64) -> i64 {\n    return match n {\n        0 => 1,\n        1..=5 => 2,\n        6..10 => 3,\n        x if x > 100 => 4,\n        _ => 0,\n    };\n}\n",
    );
    golden(
        "enum E { A(i64), B } fn m(e: E) -> i64 { return match e { E::A(x) => x, E::B => 0 }; }",
        "enum E {\n    A(i64),\n    B,\n}\nfn m(e: E) -> i64 {\n    return match e {\n        E::A(x) => x,\n        E::B => 0,\n    };\n}\n",
    );
    golden(
        "fn m(n: i32) -> i64 { return match n { -5i32 => 1, 0i32..=10i32 => 2, _ => 0 }; }",
        "fn m(n: i32) -> i64 {\n    return match n {\n        -5i32 => 1,\n        0i32..=10i32 => 2,\n        _ => 0,\n    };\n}\n",
    );
    // A struct-literal scrutinee IS parenthesized (disambiguation kept).
    golden(
        "struct P { x: i64 } fn main() -> i64 { return match (P { x: 1 }) { _ => 0 }; }",
        "struct P {\n    x: i64,\n}\nfn main() -> i64 {\n    return match (P { x: 1 }) {\n        _ => 0,\n    };\n}\n",
    );
}

#[test]
fn golden_comments_and_blank_lines_s1_s2() {
    // S2: standalone comment on its own line; trailing comment kept on the node.
    golden(
        "fn cm() -> i64 {\n    // standalone\n    let x: i64 = 1; // trailing\n    return x;\n}\n",
        "fn cm() -> i64 {\n    // standalone\n    let x: i64 = 1; // trailing\n    return x;\n}\n",
    );
    // S1: runs of blank lines collapse to a single blank.
    golden(
        "fn bl() -> i64 {\n    let a: i64 = 1;\n\n\n    let b: i64 = 2;\n    return a;\n}\n",
        "fn bl() -> i64 {\n    let a: i64 = 1;\n\n    let b: i64 = 2;\n    return a;\n}\n",
    );
    // S2: standalone comments at the top level, in source order.
    golden(
        "// header\nstruct S { a: i64 }\n// between\nfn f() -> i64 { return 0; }\n",
        "// header\nstruct S {\n    a: i64,\n}\n// between\nfn f() -> i64 {\n    return 0;\n}\n",
    );
}

#[test]
fn golden_string_escapes() {
    golden(
        "fn e() -> i64 { let s: str = \"tab\\tnl\\nq\\\"bs\\\\\"; return 0; }",
        "fn e() -> i64 {\n    let s: str = \"tab\\tnl\\nq\\\"bs\\\\\";\n    return 0;\n}\n",
    );
}

// ---- targeted RUNNABLE formatted-vs-original behavioral equality -----------

#[test]
fn formatted_runnable_programs_behave_identically() {
    // Self-contained, deliberately non-canonical programs: format them and assert
    // the canonical form checks + runs identically (the property, per construct).
    let programs = [
        "fn main() -> i64 { let a: i64 = 2; let b: i64 = 3; let c: i64 = 4; return (a + b) * c + ((a)); }",
        "fn main() -> i64 { let x: i64 = 5; if (x > 0) { return 1; } else { return 0; } }",
        "fn main() -> i64 { let mut s: i64 = 0; let mut i: i64 = 0; while i < 5 { s = s + i; i = i + 1; } return s; }",
        "enum E { A(i64), B } fn main() -> i64 { let e: E = E::A(7); return match e { E::A(x) => x, E::B => 0 }; }",
        "fn g(v: out i64) -> i64 { v = 5; return 0; } fn main() -> i64 { let mut a: i64 = 0; let r: i64 = g(out a); return a; }",
        "fn main() -> i64 { let a: i32 = 2147483647i32; wrapping { let b: i32 = a + 1i32; return conv i64 b; } return 0; }",
        "fn main() -> i64 { let t: bool = true; if !t { return 0; } return 1; }",
    ];
    for src in programs {
        let formatted = candor::format_source_real(src).unwrap_or_else(|d| panic!("format failed: {}", d.to_json()));
        assert_eq!(
            candor::format_source_real(&formatted).unwrap(),
            formatted,
            "not idempotent: {formatted}"
        );
        assert!(
            behaves_identically(src, &formatted),
            "formatting changed behavior:\nsrc  = {}\nfmt  = {}\nsrun = {} frun = {}",
            src,
            formatted,
            fmt_run_sig(src),
            fmt_run_sig(&formatted)
        );
    }
}

// ---- fixed-bug regressions (P16 findings) ----------------------------------
// The two formatter bugs found this cycle -- a panic on float literals and a
// struct literal losing its disambiguating parens in a control-flow head -- are
// fixed; these pin the correct behavior so it cannot silently regress.

#[test]
fn float_literals_format_and_round_trip() {
    let src = "fn main() -> i64 { let x: f64 = 1.5; let y: f64 = x + 2.25; return 0; }";
    let out = candor::format_source_real(src).expect("float literals must format, not panic");
    assert!(out.contains("1.5") && out.contains("2.25"), "float lexemes preserved: {out}");
    let twice = candor::format_source_real(&out).expect("formatted float source must reparse");
    assert_eq!(twice, out, "float formatting must be idempotent");
}

#[test]
fn struct_literal_in_head_keeps_parens() {
    let src = "struct P { x: i64 }\nfn main() -> i64 {\n    if (P { x: 1 }).x == 1 {\n        return 7;\n    }\n    return 0;\n}\n";
    let out = candor::format_source_real(src).expect("format ok");
    // Correct expectation: formatted output must re-parse and preserve behavior.
    assert!(candor::format_source_real(&out).is_ok(), "formatted output must reparse: {out}");
    assert!(behaves_identically(src, &out), "formatting must preserve behavior");
}

#[test]
fn golden_remaining_edge_paths() {
    // write/read return borrows.
    golden("fn f(c: write Cell) -> write Cell { return write c.*; }", "fn f(c: write Cell) -> write Cell {\n    return write c.*;\n}\n");
    golden("fn f(c: read Cell) -> read Cell { return read c.*; }", "fn f(c: read Cell) -> read Cell {\n    return read c.*;\n}\n");
    // empty struct literal.
    golden("struct E {} fn m() -> i64 { let x: E = E {}; return 0; }", "struct E {}\nfn m() -> i64 {\n    let x: E = E {};\n    return 0;\n}\n");
    // multiple interface args and multiple region params (list separators).
    golden("impl Conv[i64, u8] for Foo { fn go(read self) -> i64 { return 0; } }", "impl Conv[i64, u8] for Foo {\n    fn go(read self) -> i64 {\n        return 0;\n    }\n}\n");
    golden("fn f[region r, region s](x: read[r] i64, y: read[s] i64) -> i64 { return 0; }", "fn f[region r, region s](x: read[r] i64, y: read[s] i64) -> i64 {\n    return 0;\n}\n");
    // multiple generic value type-args.
    golden("fn m() -> i64 { let y: i64 = foo::[i64, u8]; return 0; }", "fn m() -> i64 {\n    let y: i64 = foo::[i64, u8];\n    return 0;\n}\n");
    // negative range bounds in a pattern.
    golden("fn m(n: i32) -> i64 { return match n { -5i32..=-1i32 => 1, _ => 0 }; }", "fn m(n: i32) -> i64 {\n    return match n {\n        -5i32..=-1i32 => 1,\n        _ => 0,\n    };\n}\n");
    // an impl method with no receiver and no params (empty-params branch).
    golden("impl Maker for Foo { fn make() -> i64 { return 0; } }", "impl Maker for Foo {\n    fn make() -> i64 {\n        return 0;\n    }\n}\n");
    // a `write T` type in field position.
    golden("struct S { f: write Cell } fn m() -> i64 { return 0; }", "struct S {\n    f: write Cell,\n}\nfn m() -> i64 {\n    return 0;\n}\n");
    // `spawn CALLEE(ARGS);` statement.
    golden("fn work() -> i64 { return 1; } fn main() alloc -> i64 { spawn work(); return 0; }", "fn work() -> i64 {\n    return 1;\n}\nfn main() alloc -> i64 {\n    spawn work();\n    return 0;\n}\n");
}
