//! The oracle gate for the THIRD self-hosted slice: a Candor TYPE-CHECKER core
//! (`selfhost/checker/checker.cnr`, composed after `selfhost/lexer/lexer.cnr` and
//! `selfhost/parser/parser.cnr`) is run on the tree-walker over each corpus
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
//! Harness shape reuses slice 2: embed the fixture source as a `[N]u8` literal,
//! lex via lexer.cnr, parse+check+dump via checker.cnr, reconstruct the dump
//! from `Run.trace`, compare to the filtered+sorted oracle rendering.

use candor_proto::check_source_real;
use candor_proto::{run_source_real, RunResult};

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

fn candor_program(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut prog = String::from(LEXER_SRC);
    prog.push('\n');
    prog.push_str(PARSER_SRC);
    prog.push('\n');
    prog.push_str(CHECKER_SRC);
    prog.push_str("\nfn main() -> i64 {\n");
    prog.push_str(&format!("    let src: [{}]u8 = [", bytes.len()));
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            prog.push_str(", ");
        }
        prog.push_str(&format!("{b}u8"));
    }
    prog.push_str("];\n");
    prog.push_str("    let mut buf: Buf = Buf { toks: [mk(0, 0usize, 0usize); 1024], n: 0usize };\n");
    prog.push_str("    let cnt: usize = lex(slice_of(src), write buf);\n");
    prog.push_str("    check_dump(slice_of(src), read buf);\n");
    prog.push_str("    return conv i64 cnt;\n}\n");
    prog
}

fn candor_dump(src: &str) -> String {
    let prog = candor_program(src);
    match run_source_real(&prog) {
        RunResult::Ok(run) => {
            let bytes: Vec<u8> = run.trace.iter().map(|&v| v as u8).collect();
            String::from_utf8(bytes).expect("dump is ASCII")
        }
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
