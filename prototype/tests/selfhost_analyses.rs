//! The oracle gate for the FOURTH self-hosted slice: a Candor MOVE / INIT
//! (definite-assignment) analysis — the sound core of the borrow checker
//! (`selfhost/analyses/analyses.cnr`, composed after `selfhost/lexer/lexer.cnr`
//! and `selfhost/parser/parser.cnr`) — is run on the tree-walker over each
//! corpus fixture. Its canonical diagnostic dump (one `E03XX START END` line per
//! diagnostic, sorted by (code,start,end)) is asserted byte-equal to the Rust
//! oracle checker's diagnostics for the SAME source, FILTERED to the move/init
//! families the Candor analysis covers this slice (E0301 use-after-move, E0304
//! read-before-init) and rendered in the identical canonical schema.
//!
//! Passing this gate is DIAGNOSTIC equality (code + span) between the two
//! analyses over the covered subset — the move/init family whose oracle span is
//! a BARE-IDENTIFIER use site the span-lean slice-2 arena carries verbatim. The
//! move-JOIN-anchored codes E0302 (join_span) and E0309 (scope-exit/reassign
//! span) are OUT OF SUBSET: their oracle spans are synthetic block spans the
//! arena does not carry — the same span-lean boundary slice 3 hit for E0703 etc.
//! The dataflow JOIN is still computed, so E0301/E0304 stay correct across
//! if/match branches. Loans (E0801-E0809) are deferred to slice 5.
//!
//! Harness shape reuses slice 3 (`selfhost_checker.rs`): a generated root
//! `main.cnr` `use`s the `lexer`/`analyses` modules, embeds the fixture source
//! as a `[N]u8` literal, lexes then analyze-dumps; the lexer + parser + analyses
//! tree is loaded with `run_dir` (dogfooding the module system) and the dump
//! reconstructed from `Run.trace`, compared to the filtered+sorted oracle
//! rendering. The generated `main` is `alloc` because the analysis's growable
//! diagnostic buffer is a `Vec` (its first self-hosting customer).

use candor_proto::check_source_real;
use candor_proto::RunResult;

mod selfhost_modtree;
use selfhost_modtree::{check_module_tree, on_big_stack, run_module_tree, trace_text};

const LEXER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/lexer/lexer.cnr"));
const PARSER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/parser/parser.cnr"));
const LAYOUT_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/layout/layout.cnr"));
const ANALYSES_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/analyses/analyses.cnr"));
const CHECKER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/checker/checker.cnr"));
const INTERP_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/interp/interp.cnr"));

/// The code families the Candor move/init analysis covers this slice.
const COVERED: &[&str] = &["E0301", "E0304"];

/// Render the oracle's diagnostics in the canonical dump schema, filtered to the
/// covered families and sorted by (code, start, end) — the exact schema the
/// Candor analysis emits through `trace`.
fn oracle_dump(src: &str) -> String {
    let diags = check_source_real(src).expect("oracle parses the fixture");
    let mut rows: Vec<(String, usize, usize)> = diags
        .iter()
        .filter(|d| COVERED.contains(&d.code.as_str()))
        .map(|d| (d.code.clone(), d.span.start, d.span.end))
        .collect();
    rows.sort();
    let mut s = String::new();
    for (code, a, b) in rows {
        s.push_str(&format!("{code} {a} {b}\n"));
    }
    s
}

/// Generate the root `main.cnr`: it `use`s the lexer module's `Buf`/`mk`/`lex`
/// and the analyses module's `analyze_dump`, embeds `src`, lexes then
/// analyze-dumps it. It is `alloc` because `analyze_dump`'s growable diagnostic
/// buffer is a `Vec`.
fn candor_main(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut m = String::from(
        "use lexer::{Buf, mk, lex};\nuse analyses::{analyze_dump};\n\nfn main() alloc -> i64 {\n",
    );
    m.push_str(&format!("    let src: [{}]u8 = [", bytes.len()));
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            m.push_str(", ");
        }
        m.push_str(&format!("{b}u8"));
    }
    m.push_str("];\n");
    m.push_str("    let mut buf: Buf = Buf { toks: [mk(0, 0usize, 0usize); 49152], n: 0usize };\n");
    m.push_str("    let cnt: usize = lex(slice_of(src), write buf);\n");
    m.push_str("    analyze_dump(slice_of(src), read buf);\n");
    m.push_str("    return conv i64 cnt;\n}\n");
    m
}

fn candor_dump(src: &str) -> String {
    let main = candor_main(src);
    let modules = [
        ("lexer.cnr", LEXER_SRC),
        ("parser.cnr", PARSER_SRC),
        ("analyses.cnr", ANALYSES_SRC),
    ];
    match run_module_tree(&modules, &main) {
        RunResult::Ok(run) => trace_text(&run),
        RunResult::Fault(f) => panic!("candor analysis faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!(
            "candor analysis (the .cnr program) has check errors: {:?}",
            d.iter().map(|x| &x.code).collect::<Vec<_>>()
        ),
        RunResult::ParseError(d) => panic!("candor analysis parse error: {}", d.to_json()),
    }
}

/// (fixture, expected covered-diagnostic count) — the count is a redundant,
/// human-auditable check that the gate is exercising real diagnostics, not just
/// matching two empty sets.
const CORPUS: &[(&str, usize)] = &[
    // positive: move/init-clean over the covered families (empty diag set)
    ("tests/fixtures/selfhost_analyses/pos_basic.cnr", 0),
    ("tests/fixtures/selfhost_analyses/pos_loop.cnr", 0),
    ("tests/fixtures/selfhost_analyses/pos_branch.cnr", 0),
    // negative — E0301 use-after-move (bare-identifier use spans)
    ("tests/fixtures/selfhost_analyses/neg_use_after_move.cnr", 1),
    ("tests/fixtures/selfhost_analyses/neg_call_move.cnr", 1),
    ("tests/fixtures/selfhost_analyses/neg_join_move.cnr", 1),
    ("tests/fixtures/selfhost_analyses/neg_both_branch.cnr", 1),
    ("tests/fixtures/selfhost_analyses/neg_two_moves.cnr", 2),
    // negative — E0304 read-before-init
    ("tests/fixtures/selfhost_analyses/neg_read_uninit.cnr", 1),
    ("tests/fixtures/selfhost_analyses/neg_maybe_init.cnr", 1),
    // negative — both families in one program
    ("tests/fixtures/selfhost_analyses/neg_mixed.cnr", 2),
];

fn read_fixture(rel: &str) -> String {
    let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

#[test]
fn candor_move_init_diagnostics_equal_to_oracle_over_corpus() {
    on_big_stack(|| {
        let mut passed = 0usize;
        let mut total_diags = 0usize;
        for &(rel, want_n) in CORPUS {
            let src = read_fixture(rel);
            let oracle = oracle_dump(&src);
            let mine = candor_dump(&src);
            assert_eq!(mine, oracle, "diagnostic mismatch on {rel}");
            let n = oracle.lines().count();
            assert_eq!(n, want_n, "unexpected covered-diag count on {rel}");
            total_diags += n;
            passed += 1;
        }
        assert_eq!(passed, CORPUS.len());
        eprintln!(
            "selfhost analyses: move/init diagnostic PARITY (E0301/E0304) on {}/{} fixtures, {} covered diagnostics matched",
            passed,
            CORPUS.len(),
            total_diags
        );
    });
}

// ---- ANALYSES-CLEAN FIXPOINT GATES (self-checking self-hosting, step 3) -------
//
// The full set of diagnostic families the self-hosted ANALYSES core (`analyze_dump`)
// emits: move/init (E0301/E0304), loans (E0801-E0804), effect (E0401), and match
// exhaustiveness (E0601). `candor_dump` returns the analysis's RAW output over
// ALL of these; the oracle side is filtered to the same union.
const FULL_COVERED: &[&str] = &[
    "E0301", "E0304", "E0401", "E0601", "E0801", "E0802", "E0803", "E0804",
];

/// Run the MODULE-AWARE reference checker (`check_dir` via `check_module_tree`)
/// over the real self-host module tree -- which runs the FULL Rust check pipeline
/// (move/init/loans/effects/exhaustiveness) over every module, the ground truth --
/// and render its diagnostics for the checked module in the canonical dump schema,
/// filtered to the analyses families. `modules` must CONTAIN the checked module so
/// its diagnostics are real (non-vacuous), not just the generated `main`.
fn module_oracle_full(modules: &[(&str, &str)], src: &str) -> String {
    let main = candor_main(src);
    let diags = check_module_tree(modules, &main).expect("oracle checks the module tree");
    let mut rows: Vec<(String, usize, usize)> = diags
        .iter()
        .filter(|d| FULL_COVERED.contains(&d.code.as_str()))
        .map(|d| (d.code.clone(), d.span.start, d.span.end))
        .collect();
    rows.sort();
    let mut out = String::new();
    for (code, a, b) in rows {
        out.push_str(&format!("{code} {a} {b}\n"));
    }
    out
}

/// A self-contained use-after-move injected into a checked module to give the
/// clean assertions TEETH: the self-host analysis MUST flag it E0301, so an empty
/// clean dump cannot pass vacuously (e.g. from the analysis silently no-op-ing).
const MOVE_SMOKE: &str = concat!(
    "\nstruct ZzSmoke { a: i64, b: i64 }\n",
    "fn zz_smoke() -> ZzSmoke {\n",
    "    let x: ZzSmoke = ZzSmoke { a: 0, b: 0 };\n",
    "    let y: ZzSmoke = x;\n",
    "    return x;\n",
    "}\n",
);

/// FIXPOINT GATE: the self-hosted ANALYSES core, run over the self-hosted LEXER's
/// own source, emits an EMPTY covered-diagnostic set across ALL its families --
/// byte-equal to the module-aware Rust oracle over the real lexer+parser+analyses
/// tree (also empty, since the source is analyses-clean). This is a far deeper
/// self-check than name resolution: it proves the move/init, loan, effect, and
/// exhaustiveness cores accept the self-host source.
#[test]
fn candor_analyses_check_lexer_source_clean_fixpoint() {
    on_big_stack(|| {
        // Teeth: a use-after-move injected into the lexer source MUST fire E0301.
        let broken = format!("{LEXER_SRC}{MOVE_SMOKE}");
        let broken_dump = candor_dump(&broken);
        assert!(
            broken_dump.contains("E0301"),
            "negative smoke: injected use-after-move must be flagged E0301, got {broken_dump:?}"
        );

        // Clean: the real lexer.cnr is analyses-clean over EVERY covered family,
        // byte-equal to the module-aware oracle (empty on this valid source).
        let modules = [
            ("lexer.cnr", LEXER_SRC),
            ("parser.cnr", PARSER_SRC),
            ("analyses.cnr", ANALYSES_SRC),
        ];
        let oracle = module_oracle_full(&modules, LEXER_SRC);
        let mine = candor_dump(LEXER_SRC);
        assert_eq!(mine, oracle, "self-host analyses diverged from the oracle on lexer.cnr");
        assert!(
            mine.is_empty(),
            "self-host analyses emitted diagnostics on analyses-clean lexer.cnr: {mine:?}"
        );
    });
}

/// FIXPOINT GATE: the self-hosted ANALYSES core, run over the self-hosted CHECKER's
/// own source (a NON-LEAF module importing ~70 names from lexer+parser), emits an
/// EMPTY covered-diagnostic set across ALL its families -- byte-equal to the
/// module-aware Rust oracle over the real lexer+parser+analyses+checker tree.
/// The analysis needs no import resolution: it runs dataflow over the parsed AST
/// and treats unresolved names as non-locals, so imported names never false-fire.
#[test]
fn candor_analyses_check_checker_source_clean_fixpoint() {
    on_big_stack(|| {
        // Teeth: a use-after-move injected into the checker source MUST fire E0301.
        let broken = format!("{CHECKER_SRC}{MOVE_SMOKE}");
        let broken_dump = candor_dump(&broken);
        assert!(
            broken_dump.contains("E0301"),
            "negative smoke: injected use-after-move must be flagged E0301, got {broken_dump:?}"
        );

        // Clean: the real checker.cnr is analyses-clean over EVERY covered family.
        // The oracle tree INCLUDES checker.cnr so the reference checks the real
        // module (ground truth), not just the generated main.
        let modules = [
            ("lexer.cnr", LEXER_SRC),
            ("parser.cnr", PARSER_SRC),
            ("analyses.cnr", ANALYSES_SRC),
            ("checker.cnr", CHECKER_SRC),
        ];
        let oracle = module_oracle_full(&modules, CHECKER_SRC);
        let mine = candor_dump(CHECKER_SRC);
        assert_eq!(mine, oracle, "self-host analyses diverged from the oracle on checker.cnr");
        assert!(
            mine.is_empty(),
            "self-host analyses emitted diagnostics on analyses-clean checker.cnr: {mine:?}"
        );
    });
}

/// FIXPOINT GATE: the self-hosted ANALYSES core, run over ITS OWN source
/// (`analyses.cnr`, a non-leaf module importing ~70 names from lexer+parser and the
/// first self-host module to use the Vec-backed diagnostic buffer), emits an EMPTY
/// covered-diagnostic set across ALL its families -- byte-equal to the module-aware
/// Rust oracle over the real lexer+parser+analyses tree. This closes the analyses
/// self-check over the third module: the analysis accepts the very source that
/// implements it.
#[test]
fn candor_analyses_check_analyses_source_clean_fixpoint() {
    on_big_stack(|| {
        // Teeth: a use-after-move injected into the analyses source MUST fire E0301.
        let broken = format!("{ANALYSES_SRC}{MOVE_SMOKE}");
        let broken_dump = candor_dump(&broken);
        assert!(
            broken_dump.contains("E0301"),
            "negative smoke: injected use-after-move must be flagged E0301, got {broken_dump:?}"
        );

        // Clean: the real analyses.cnr is analyses-clean over EVERY covered family.
        // The oracle tree already contains analyses.cnr, so the reference checks the
        // real module (ground truth), not just the generated main.
        let modules = [
            ("lexer.cnr", LEXER_SRC),
            ("parser.cnr", PARSER_SRC),
            ("analyses.cnr", ANALYSES_SRC),
        ];
        let oracle = module_oracle_full(&modules, ANALYSES_SRC);
        let mine = candor_dump(ANALYSES_SRC);
        assert_eq!(mine, oracle, "self-host analyses diverged from the oracle on analyses.cnr");
        assert!(
            mine.is_empty(),
            "self-host analyses emitted diagnostics on analyses-clean analyses.cnr: {mine:?}"
        );
    });
}

/// FIXPOINT GATE: the self-hosted ANALYSES core, run over the self-hosted PARSER's
/// own source (`parser.cnr`, the LARGEST self-host module -- ~19703 tokens /
/// ~11272 self-host nodes, 2.6x checker.cnr), emits an EMPTY covered-diagnostic set
/// across ALL its families -- byte-equal to the module-aware Rust oracle over the
/// real lexer+parser+analyses tree. parser.cnr is the module that previously could
/// not embed (the interpreter literal-leak corrupted its 77.7 KB `[N]u8` literal);
/// with that leak fixed it embeds byte-exact, bringing the last module under the
/// analyses self-check. This is the recursive-descent tree-builder analysing itself.
#[test]
fn candor_analyses_check_parser_source_clean_fixpoint() {
    on_big_stack(|| {
        // Teeth: a use-after-move injected into the parser source MUST fire E0301.
        let broken = format!("{PARSER_SRC}{MOVE_SMOKE}");
        let broken_dump = candor_dump(&broken);
        assert!(
            broken_dump.contains("E0301"),
            "negative smoke: injected use-after-move must be flagged E0301, got {broken_dump:?}"
        );

        // Clean: the real parser.cnr is analyses-clean over EVERY covered family.
        // The oracle tree already contains parser.cnr, so the reference checks the
        // real module (ground truth), not just the generated main.
        let modules = [
            ("lexer.cnr", LEXER_SRC),
            ("parser.cnr", PARSER_SRC),
            ("analyses.cnr", ANALYSES_SRC),
        ];
        let oracle = module_oracle_full(&modules, PARSER_SRC);
        let mine = candor_dump(PARSER_SRC);
        assert_eq!(mine, oracle, "self-host analyses diverged from the oracle on parser.cnr");
        assert!(
            mine.is_empty(),
            "self-host analyses emitted diagnostics on analyses-clean parser.cnr: {mine:?}"
        );
    });
}

/// FIXPOINT GATE: the self-hosted ANALYSES core, run over the self-hosted
/// INTERPRETER's own source (`interp.cnr`, the LARGEST self-host module --
/// ~3721 lines / ~32712 tokens, under the 49152 arena cap raised by F-ARENA-CAP),
/// emits an EMPTY covered-diagnostic set across ALL
/// its families -- byte-equal to the module-aware Rust oracle over the real
/// lexer+parser+analyses+interp tree. This is the SIXTH -- and final -- module
/// under the analyses self-check, closing the fixpoint: every self-host module,
/// including the interpreter that executes them, passes its own move/init, loan,
/// effect, and exhaustiveness analyses.
#[test]
fn candor_analyses_check_interp_source_clean_fixpoint() {
    on_big_stack(|| {
        // Teeth: a use-after-move injected into the interp source MUST fire E0301.
        let broken = format!("{INTERP_SRC}{MOVE_SMOKE}");
        let broken_dump = candor_dump(&broken);
        assert!(
            broken_dump.contains("E0301"),
            "negative smoke: injected use-after-move must be flagged E0301, got {broken_dump:?}"
        );

        // Clean: the real interp.cnr is analyses-clean over EVERY covered family.
        // The oracle tree INCLUDES interp.cnr so the reference checks the real
        // module (ground truth), not just the generated main.
        let modules = [
            ("lexer.cnr", LEXER_SRC),
            ("parser.cnr", PARSER_SRC),
            ("analyses.cnr", ANALYSES_SRC),
            ("layout.cnr", LAYOUT_SRC),
            ("interp.cnr", INTERP_SRC),
        ];
        let oracle = module_oracle_full(&modules, INTERP_SRC);
        let mine = candor_dump(INTERP_SRC);
        assert_eq!(mine, oracle, "self-host analyses diverged from the oracle on interp.cnr");
        assert!(
            mine.is_empty(),
            "self-host analyses emitted diagnostics on analyses-clean interp.cnr: {mine:?}"
        );
    });
}
