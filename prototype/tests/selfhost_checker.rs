//! The oracle gate for the THIRD self-hosted slice: a Candor TYPE-CHECKER core
//! (`selfhost/checker/checker.cnr`, loaded as a module tree with the `lexer` and
//! `parser` modules) is run on the tree-walker over each corpus
//! fixture. Its canonical diagnostic dump (one `E010X START END` line per
//! diagnostic, sorted by (code,start,end)) is asserted byte-equal to the Rust
//! oracle checker's diagnostics for the SAME source, FILTERED to the code
//! families the Candor checker covers this slice (E0102 unknown type, E0103
//! unknown name) and rendered in the identical canonical schema.
//!
//! Passing this gate is DIAGNOSTIC equality (code + span) between the two
//! checkers over the covered subset. Value-type-level codes whose oracle span is
//! a COMPOSITE expression range (E0703/E0706/E0107/E0108/E0605/E0709) are OUT OF
//! SUBSET this slice: the span-lean slice-2 arena does not carry those spans.
//! Harness shape reuses slice 2: a generated root `main.cnr` `use`s the
//! `lexer`/`parser`/`checker` modules, embeds the fixture source as a `[N]u8`
//! literal, lexes then parse+check+dumps; the tree is loaded with `run_dir`
//! (dogfooding the module system) and the dump reconstructed from `Run.trace`,
//! compared to the filtered+sorted oracle rendering.

use candor_proto::check_source_real;
use candor_proto::RunResult;

mod selfhost_modtree;
use selfhost_modtree::{check_module_tree, run_module_tree, trace_text};

const LEXER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/lexer/lexer.cnr"));
const PARSER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/parser/parser.cnr"));
const CHECKER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/checker/checker.cnr"));
const ANALYSES_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/analyses/analyses.cnr"));

/// The code families the Candor checker covers this slice.
const COVERED: &[&str] = &["E0102", "E0103"];

/// Render the oracle's diagnostics in the canonical dump schema, filtered to the
/// covered families and sorted by (code, start, end) — the exact schema the
/// Candor checker emits through `trace`.
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
/// and the checker module's `check_dump`, embeds `src`, lexes then check-dumps it.
/// `main` is `alloc` because `check_dump` builds Map-backed symbol tables (the
/// allocator-explicit `map_new`), whose effect is viral through the call.
fn candor_main(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut m = String::from(
        "use lexer::{Buf, mk, lex};\nuse checker::{check_dump};\n\nfn main() alloc -> i64 {\n",
    );
    m.push_str(&format!("    let src: [{}]u8 = [", bytes.len()));
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            m.push_str(", ");
        }
        m.push_str(&format!("{b}u8"));
    }
    m.push_str("];\n");
    m.push_str("    let mut buf: Buf = Buf { toks: [mk(0, 0usize, 0usize); 32768], n: 0usize };\n");
    m.push_str("    let cnt: usize = lex(slice_of(src), write buf);\n");
    m.push_str("    check_dump(slice_of(src), read buf);\n");
    m.push_str("    return conv i64 cnt;\n}\n");
    m
}

fn candor_dump(src: &str) -> String {
    let main = candor_main(src);
    let modules = [
        ("lexer.cnr", LEXER_SRC),
        ("parser.cnr", PARSER_SRC),
        ("checker.cnr", CHECKER_SRC),
    ];
    match run_module_tree(&modules, &main) {
        RunResult::Ok(run) => trace_text(&run),
        RunResult::Fault(f) => panic!("candor checker faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!(
            "candor checker (the .cnr program) has check errors: {:?}",
            d.iter().map(|x| &x.code).collect::<Vec<_>>()
        ),
        RunResult::ParseError(d) => panic!("candor checker parse error: {}", d.to_json()),
    }
}

/// (fixture, expected covered-diagnostic count) — the count is a redundant,
/// human-auditable check that the gate is exercising real diagnostics, not just
/// matching two empty sets.
const CORPUS: &[(&str, usize)] = &[
    // positive: type-check clean over the covered families (empty diag set)
    ("tests/fixtures/selfhost_check/pos_basic.cnr", 0),
    ("tests/fixtures/selfhost_check/pos_builtins.cnr", 0),
    ("tests/fixtures/selfhost_check/pos_scopes.cnr", 0),
    // negative: specific covered diagnostics
    ("tests/fixtures/selfhost_check/neg_unknown_name.cnr", 2),
    ("tests/fixtures/selfhost_check/neg_unknown_type.cnr", 4),
    ("tests/fixtures/selfhost_check/neg_mixed.cnr", 2),
    ("tests/fixtures/selfhost_check/neg_scope_leak.cnr", 1),
];

fn read_fixture(rel: &str) -> String {
    let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn on_big_stack<F: FnOnce() + Send + 'static>(f: F) {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(f)
        .expect("spawn big-stack thread")
        .join()
        .expect("gate thread panicked");
}

#[test]
fn candor_checker_diagnostics_equal_to_oracle_over_corpus() {
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
            "selfhost checker: diagnostic PARITY (E0102/E0103) on {}/{} fixtures, {} covered diagnostics matched",
            passed,
            CORPUS.len(),
            total_diags
        );
    });
}

/// FIXPOINT GATE (first self-checking sub-goal): run the self-hosted checker over
/// the self-hosted LEXER's own source and assert its covered-diagnostic set
/// (E0102/E0103) is EMPTY — byte-equal to the Rust oracle, which also emits
/// nothing on this valid source (so both sides are the empty set by construction).
/// This proves the self-check loop end-to-end on real self-host source.
#[test]
fn candor_checker_checks_lexer_source_clean_fixpoint() {
    on_big_stack(|| {
        // Teeth: a deliberately-broken variant (a param of an unknown type) MUST be
        // flagged E0102, so the clean assertion below cannot pass vacuously.
        let broken = format!("{LEXER_SRC}\nfn zz_smoke(x: Nonexistent) -> unit {{ return; }}\n");
        let broken_dump = candor_dump(&broken);
        assert!(
            broken_dump.contains("E0102"),
            "negative smoke: broken lexer source must be flagged E0102, got {broken_dump:?}"
        );

        // Clean: the real lexer.cnr checks clean over the covered families, byte-equal
        // to the oracle (empty set on this valid source).
        let oracle = oracle_dump(LEXER_SRC);
        let mine = candor_dump(LEXER_SRC);
        assert_eq!(mine, oracle, "self-host checker diverged from oracle on lexer.cnr");
        assert!(
            mine.is_empty(),
            "self-host checker emitted covered diagnostics on clean lexer.cnr: {mine:?}"
        );
    });
}

/// Render the MODULE-AWARE oracle's diagnostics (from `check_dir` over the
/// lexer+parser+checker tree, which resolves `use` imports before checking) in
/// the same canonical schema as `oracle_dump`, filtered to the covered families.
fn module_oracle_dump(src: &str) -> String {
    let main = candor_main(src);
    let modules = [
        ("lexer.cnr", LEXER_SRC),
        ("parser.cnr", PARSER_SRC),
        ("checker.cnr", CHECKER_SRC),
    ];
    let diags = check_module_tree(&modules, &main).expect("oracle checks the module tree");
    let mut rows: Vec<(String, usize, usize)> = diags
        .iter()
        .filter(|d| COVERED.contains(&d.code.as_str()))
        .map(|d| (d.code.clone(), d.span.start, d.span.end))
        .collect();
    rows.sort();
    let mut out = String::new();
    for (code, a, b) in rows {
        out.push_str(&format!("{code} {a} {b}\n"));
    }
    out
}

/// IMPORT-RESOLUTION ISOLATION GATE (the payoff): the self-hosted checker checks
/// the self-hosted CHECKER's own source -- a NON-LEAF module that imports names
/// from BOTH the `lexer` and `parser` modules -- IN ISOLATION, resolving those
/// imports itself from its `use` decls (the parser's `T_USE` nodes -> the
/// checker's imported-name registration). It emits an EMPTY covered-diagnostic
/// (E0102/E0103) set, byte-equal to the MODULE-AWARE reference oracle.
///
/// The naive single-file oracle (`check_source_real`, no import resolution) flags
/// every imported type as E0102 -- asserted below -- so this gate proves real
/// import resolution, not a leaf module trivially checking clean (contrast the
/// `lexer.cnr` fixpoint gate above, whose leaf module has nothing to resolve).
#[test]
fn candor_checker_checks_checker_source_clean_via_import_resolution() {
    on_big_stack(|| {
        // The imports really are load-bearing: without resolution the non-leaf
        // module's imported TYPES (Node, P, Buf, ...) are unknown -> E0102. This
        // makes the clean assertion below meaningful, not a leaf-module tautology.
        let naive = check_source_real(CHECKER_SRC).expect("oracle parses checker.cnr");
        let naive_unknown_types =
            naive.iter().filter(|d| d.code == "E0102").count();
        assert!(
            naive_unknown_types > 0,
            "single-file check must flag the unresolved imported types (E0102)"
        );

        // Teeth: a deliberately-broken variant (a param of an unknown type) MUST be
        // flagged E0102, so the clean assertion below cannot pass vacuously.
        let broken = format!("{CHECKER_SRC}\nfn zz_smoke(x: Nonexistent) -> unit {{ return; }}\n");
        let broken_dump = candor_dump(&broken);
        assert!(
            broken_dump.contains("E0102"),
            "negative smoke: broken checker source must be flagged E0102, got {broken_dump:?}"
        );

        // Clean: the self-host checker, resolving checker.cnr's `use lexer::{..}` /
        // `use parser::{..}` imports itself, emits an EMPTY covered set -- byte-equal
        // to the module-aware oracle (both resolve the imports and find it clean).
        let oracle = module_oracle_dump(CHECKER_SRC);
        let mine = candor_dump(CHECKER_SRC);
        assert_eq!(
            mine, oracle,
            "self-host checker diverged from the module-aware oracle on checker.cnr"
        );
        assert!(
            mine.is_empty(),
            "self-host checker emitted covered diagnostics on clean checker.cnr: {mine:?}"
        );
    });
}

/// Like `module_oracle_dump`, but over an explicit module tree -- so a checked
/// module that lives BEYOND the lexer+parser+checker set (here `analyses.cnr`,
/// which imports from lexer+parser) is a REAL member of the tree and has its own
/// imports resolved by the reference checker.
fn module_oracle_dump_tree(modules: &[(&str, &str)], src: &str) -> String {
    let main = candor_main(src);
    let diags = check_module_tree(modules, &main).expect("oracle checks the module tree");
    let mut rows: Vec<(String, usize, usize)> = diags
        .iter()
        .filter(|d| COVERED.contains(&d.code.as_str()))
        .map(|d| (d.code.clone(), d.span.start, d.span.end))
        .collect();
    rows.sort();
    let mut out = String::new();
    for (code, a, b) in rows {
        out.push_str(&format!("{code} {a} {b}\n"));
    }
    out
}

/// IMPORT-RESOLUTION ISOLATION GATE (third module): the self-hosted checker checks
/// the self-hosted ANALYSES core's own source -- a NON-LEAF module importing ~70
/// names from the `lexer` and `parser` modules AND exercising the Vec/collection
/// builtins `get`/`set`/`vec_new` -- IN ISOLATION, resolving those imports itself
/// from its `use` decls. It emits an EMPTY covered-diagnostic (E0102/E0103) set,
/// byte-equal to the MODULE-AWARE reference oracle over the real
/// lexer+parser+checker+analyses tree.
///
/// This is the FOURTH module under the self-check name-resolution fixpoint, and
/// the first to use the collection builtins -- so the gate proves both the builtin
/// table extension (`get`/`set`/`vec_new`) and that name resolution scales past the
/// checker.cnr size without the static-storage collision the interpreter had.
#[test]
fn candor_checker_checks_analyses_source_clean_via_import_resolution() {
    on_big_stack(|| {
        // The imports are load-bearing: without resolution the non-leaf module's
        // imported TYPES (Node, P, Buf, ...) are unknown -> E0102, so the clean
        // assertion below is meaningful, not a leaf-module tautology.
        let naive = check_source_real(ANALYSES_SRC).expect("oracle parses analyses.cnr");
        let naive_unknown_types = naive.iter().filter(|d| d.code == "E0102").count();
        assert!(
            naive_unknown_types > 0,
            "single-file check must flag the unresolved imported types (E0102)"
        );

        // Teeth: a deliberately-broken variant (a param of an unknown type) MUST be
        // flagged E0102, so the clean assertion below cannot pass vacuously.
        let broken = format!("{ANALYSES_SRC}\nfn zz_smoke(x: Nonexistent) -> unit {{ return; }}\n");
        let broken_dump = candor_dump(&broken);
        assert!(
            broken_dump.contains("E0102"),
            "negative smoke: broken analyses source must be flagged E0102, got {broken_dump:?}"
        );

        // Clean: the self-host checker, resolving analyses.cnr's imports itself,
        // emits an EMPTY covered set -- byte-equal to the module-aware oracle over
        // a tree that INCLUDES analyses.cnr (so the reference resolves its imports).
        let modules = [
            ("lexer.cnr", LEXER_SRC),
            ("parser.cnr", PARSER_SRC),
            ("checker.cnr", CHECKER_SRC),
            ("analyses.cnr", ANALYSES_SRC),
        ];
        let oracle = module_oracle_dump_tree(&modules, ANALYSES_SRC);
        let mine = candor_dump(ANALYSES_SRC);
        assert_eq!(
            mine, oracle,
            "self-host checker diverged from the module-aware oracle on analyses.cnr"
        );
        assert!(
            mine.is_empty(),
            "self-host checker emitted covered diagnostics on clean analyses.cnr: {mine:?}"
        );
    });
}

/// IMPORT-RESOLUTION ISOLATION GATE (parser.cnr -- the LARGEST self-host module).
/// The self-hosted checker checks the self-hosted PARSER's own source IN ISOLATION,
/// resolving its `use lexer::{Tok, Buf, span_eq, ...}` imports itself, and emits an
/// EMPTY covered-diagnostic (E0102/E0103) set, byte-equal to the MODULE-AWARE
/// reference oracle over the lexer+parser+checker tree.
///
/// parser.cnr is ~19703 tokens / ~11272 self-host nodes -- 2.6x checker.cnr and the
/// module that previously could not embed at all (the interpreter's literal-leak
/// static-storage collision corrupted its 77.7 KB `[N]u8` literal, yielding 11378
/// self-host tokens vs the oracle's 19703). With that leak fixed (content-addressed
/// literal interning) the embedding is byte-exact, so the last module comes under
/// the name-res self-check. This is the FIFTH module and the LARGEST.
#[test]
fn candor_checker_checks_parser_source_clean_via_import_resolution() {
    on_big_stack(|| {
        // The imports are load-bearing: without resolution parser.cnr's imported
        // TYPES (Tok, Buf) are unknown -> E0102, so the clean assertion below is
        // meaningful, not a leaf-module tautology.
        let naive = check_source_real(PARSER_SRC).expect("oracle parses parser.cnr");
        let naive_unknown_types = naive.iter().filter(|d| d.code == "E0102").count();
        assert!(
            naive_unknown_types > 0,
            "single-file check must flag the unresolved imported types (E0102)"
        );

        // Teeth: a deliberately-broken variant (a param of an unknown type) MUST be
        // flagged E0102, so the clean assertion below cannot pass vacuously.
        let broken = format!("{PARSER_SRC}\nfn zz_smoke(x: Nonexistent) -> unit {{ return; }}\n");
        let broken_dump = candor_dump(&broken);
        assert!(
            broken_dump.contains("E0102"),
            "negative smoke: broken parser source must be flagged E0102, got {broken_dump:?}"
        );

        // Clean: the self-host checker, resolving parser.cnr's `use lexer::{..}`
        // imports itself, emits an EMPTY covered set -- byte-equal to the
        // module-aware oracle over the lexer+parser+checker tree (which contains
        // parser.cnr, so the reference resolves its imports).
        let oracle = module_oracle_dump(PARSER_SRC);
        let mine = candor_dump(PARSER_SRC);
        assert_eq!(
            mine, oracle,
            "self-host checker diverged from the module-aware oracle on parser.cnr"
        );
        assert!(
            mine.is_empty(),
            "self-host checker emitted covered diagnostics on clean parser.cnr: {mine:?}"
        );
    });
}
