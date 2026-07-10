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
use selfhost_modtree::{run_module_tree, trace_text};

const LEXER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/lexer/lexer.cnr"));
const PARSER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/parser/parser.cnr"));
const CHECKER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/checker/checker.cnr"));

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
fn candor_main(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut m = String::from(
        "use lexer::{Buf, mk, lex};\nuse checker::{check_dump};\n\nfn main() -> i64 {\n",
    );
    m.push_str(&format!("    let src: [{}]u8 = [", bytes.len()));
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            m.push_str(", ");
        }
        m.push_str(&format!("{b}u8"));
    }
    m.push_str("];\n");
    m.push_str("    let mut buf: Buf = Buf { toks: [mk(0, 0usize, 0usize); 1024], n: 0usize };\n");
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
