//! P15 migrator parity harness (design 0006 §5). For every living-corpus fixture
//! the migrator translates the throwaway `.cn` source to real `.cnr` syntax; the
//! test asserts the parity principle:
//!   (1) the migrated output parses under the real front-end with zero
//!       diagnostics beyond those the `.cn` original produced, and
//!   (2) check + run behavior is IDENTICAL — the same diagnostic code multiset,
//!       and the same run outcome/sentinel where the fixture is runnable.
//! It also pins the committed `.cnr` fixtures to the migrator's deterministic
//! output and spot-checks the `// MIGRATE:` markers and mechanical rewrites.

use candor_proto::{
    check_source, check_source_real, migrate_source, parse_source_real, run_source,
    run_source_real, RunResult,
};

const CHECK: &[&str] = &[
    "11_1_allocator",
    "11_2_scheduler",
    "11_3_mmio",
    "11_4_parser",
    "11_5_arena",
];
const RUN: &[&str] = &[
    "11_1_allocator",
    "11_2_scheduler",
    "11_3_mmio",
    "11_4_parser",
    "11_5_arena",
];

fn read(rel: &str) -> String {
    let path = format!("{}/tests/fixtures/{rel}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

/// Sorted diagnostic-code multiset from the throwaway checker.
fn throwaway_codes(src: &str) -> Vec<String> {
    let mut v: Vec<String> = match check_source(src) {
        Ok(diags) => diags.into_iter().map(|d| d.code).collect(),
        Err(parse) => vec![parse.code],
    };
    v.sort();
    v
}

/// Sorted diagnostic-code multiset from the real checker.
fn real_codes(src: &str) -> Vec<String> {
    let mut v: Vec<String> = match check_source_real(src) {
        Ok(diags) => diags.into_iter().map(|d| d.code).collect(),
        Err(parse) => vec![parse.code],
    };
    v.sort();
    v
}

/// A comparable reduction of a run outcome.
fn outcome(r: RunResult) -> (String, i64, String) {
    match r {
        RunResult::Ok(run) => ("ok".into(), run.ret, String::new()),
        RunResult::Fault(f) => ("fault".into(), 0, f.to_json()),
        RunResult::CheckErrors(d) => (
            "check-errors".into(),
            0,
            d.iter().map(|x| x.code.clone()).collect::<Vec<_>>().join(","),
        ),
        RunResult::ParseError(d) => ("parse-error".into(), 0, d.code),
    }
}

// ---- (1)+(2, check side): migrated output parses clean and matches the .cn ----

#[test]
fn migrated_check_fixtures_are_diagnostic_identical() {
    for name in CHECK {
        let cn = read(&format!("check/{name}.cn"));
        let migrated = migrate_source(&cn).unwrap_or_else(|d| panic!("{name}: migrate failed: {}", d.to_json()));
        // Parses under the real front-end with no parse error.
        parse_source_real(&migrated)
            .unwrap_or_else(|d| panic!("{name}.cnr failed to parse under real front-end: {}", d.to_json()));
        // Same diagnostic-code multiset as the throwaway original.
        let before = throwaway_codes(&cn);
        let after = real_codes(&migrated);
        assert_eq!(
            before, after,
            "{name}: migrated diagnostics differ from the .cn original (.cn={before:?} .cnr={after:?})"
        );
    }
}

// ---- (2, run side): migrated output runs to the identical outcome ----

#[test]
fn migrated_run_fixtures_run_identically() {
    for name in RUN {
        let cn = read(&format!("run/{name}.cn"));
        let migrated = migrate_source(&cn).unwrap_or_else(|d| panic!("{name}: migrate failed: {}", d.to_json()));
        parse_source_real(&migrated)
            .unwrap_or_else(|d| panic!("{name}.cnr failed to parse: {}", d.to_json()));
        let before = outcome(run_source(&cn));
        let after = outcome(run_source_real(&migrated));
        assert_eq!(
            before, after,
            "{name}: migrated run outcome differs from the .cn original"
        );
    }
}

// ---- the committed .cnr fixtures are the migrator's deterministic output ----

#[test]
fn committed_cnr_fixtures_match_migrator_output() {
    for (dir, names) in [("check", CHECK), ("run", RUN)] {
        for name in names {
            let cn = read(&format!("{dir}/{name}.cn"));
            let migrated = migrate_source(&cn).unwrap();
            let committed = read(&format!("{dir}/{name}.cnr"));
            assert_eq!(
                migrated, committed,
                "{dir}/{name}.cnr is stale; re-run `candor migrate {dir}/{name}.cn -o {dir}/{name}.cnr`"
            );
        }
    }
}

// ---- MIGRATE markers land only on the detectable author-assisted sites ----

#[test]
fn migrate_markers_flag_boxresult_ladders() {
    // The parser fixture has two nested two-arm BoxResult matches.
    let parser = migrate_source(&read("check/11_4_parser.cn")).unwrap();
    assert_eq!(
        parser.matches("// MIGRATE:").count(),
        2,
        "check/11_4_parser should carry two MIGRATE markers"
    );
    // The mmio fixture has matches, but none on BoxResult -> no markers.
    let mmio = migrate_source(&read("check/11_3_mmio.cn")).unwrap();
    assert_eq!(
        mmio.matches("// MIGRATE:").count(),
        0,
        "check/11_3_mmio has no BoxResult ladder, so no MIGRATE marker"
    );
    // Runnable parser has the two in parse_expr plus one in main.
    let run_parser = migrate_source(&read("run/11_4_parser.cn")).unwrap();
    assert_eq!(run_parser.matches("// MIGRATE:").count(), 3);
}

// ---- mechanical-row spot checks on the emitted text ----

#[test]
fn mechanical_rewrites_are_applied() {
    let arena = migrate_source(&read("check/11_5_arena.cn")).unwrap();
    // case dropped; conv parens dropped; read auto-deref; write `.*` kept.
    assert!(arena.contains("let i: u32 = ar.count;"), "read auto-deref");
    assert!(arena.contains("ar.*.mem[conv usize i] = n;"), "write `.*` + conv without parens");
    assert!(arena.contains("Node::leaf(v) => return v,"), "`case` removed from arms");
    // reborrow collapse: `write (deref ar)` -> `ar`.
    assert!(arena.contains("fold_consts(ar, l)"), "reborrow collapse");
    assert!(!arena.contains("deref"), "no residual `deref` keyword");
    assert!(!arena.contains("case "), "no residual `case` keyword");
    assert!(!arena.contains("slice"), "no residual `slice` keyword");

    let parser = migrate_source(&read("check/11_4_parser.cn")).unwrap();
    // slice type keyword rewritten to `[u8]`.
    assert!(parser.contains("s: [u8]"), "slice u8 -> [u8]");
}
