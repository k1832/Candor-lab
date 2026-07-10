//! The oracle gate for the FIFTH self-hosted slice: the Candor LOAN (borrow)
//! analysis — the XOR conflict core of the borrow checker — that extends slice
//! 4's move/init analysis in the same `selfhost/analyses/analyses.cnr` (composed
//! after `selfhost/lexer/lexer.cnr` and `selfhost/parser/parser.cnr`). Its
//! canonical diagnostic dump (one `E08XX START END` line per diagnostic, sorted
//! by (code,start,end)) is asserted byte-equal to the Rust oracle's diagnostics
//! for the SAME source, FILTERED to the loan families the Candor analysis covers
//! (E0801 conflicting borrow, E0802 move-while-borrowed, E0804 read-while-
//! exclusively-borrowed) and rendered in the identical canonical schema.
//!
//! COVERAGE / BOUNDARY (reported honestly).
//!  * MATCHED (code + span) against the oracle's `loans.rs` XOR conflict scan:
//!    E0801, E0802, E0804 — all three carry the ACCESS site's BARE-IDENTIFIER
//!    span (the borrowed/moved/read place), which the span-lean slice-2 arena
//!    stores verbatim in the `T_ID` node, exactly like slice 4's E0301/E0304.
//!  * OUT OF SUBSET on the span-lean boundary (the loan analog of slice 4's
//!    E0302/E0309): E0803 (write-while-borrowed) whose oracle span is the whole
//!    assignment STATEMENT (the arena carries no statement span), and E0809
//!    (write-through-shared) which is a TYPE-level check in `expr.rs` with a
//!    composite prefix/statement span AND needs per-binding borrow-TYPE tracking.
//!  * LIVENESS: a RESTRICTED-LEXICAL approximation — a loan is in scope from its
//!    `let` to the end of its enclosing block. This is CONSERVATIVE vs the
//!    oracle's backward NLL-lite liveness (live to the binding's last use), so
//!    the fixtures are restricted to the lexical==NLL region: every borrow
//!    binding is used at/after each conflict point and there is no dead-borrow-
//!    then-reuse; `pos_block` exercises a loan that DROPS at a block exit so a
//!    later move is clean under BOTH liveness models.
//!
//! Harness shape reuses slice 4 (`selfhost_analyses.rs`): embed the fixture
//! source as a `[N]u8` literal, lex + parse + analyze + dump via the concatenated
//! `.cnr` program on a 256 MiB thread, compare to the filtered+sorted oracle
//! rendering. The generated `main` is `alloc` (the analysis's Vecs).

use candor_proto::check_source_real;
use candor_proto::{run_source_real, RunResult};

const LEXER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/lexer/lexer.cnr"));
const PARSER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/parser/parser.cnr"));
const ANALYSES_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/analyses/analyses.cnr"));

/// The loan families the Candor analysis matches against the oracle this slice.
const COVERED: &[&str] = &["E0801", "E0802", "E0804"];

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
    prog.push_str(ANALYSES_SRC);
    prog.push_str("\nfn main() alloc -> i64 {\n");
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
    prog.push_str("    analyze_dump(slice_of(src), read buf);\n");
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
        RunResult::Fault(f) => panic!("candor analysis faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!(
            "candor analysis (the .cnr program) has check errors: {:?}",
            d.iter().map(|x| &x.code).collect::<Vec<_>>()
        ),
        RunResult::ParseError(d) => panic!("candor analysis parse error: {}", d.to_json()),
    }
}

/// (fixture, expected covered-diagnostic count) — a redundant, human-auditable
/// check that the gate exercises real diagnostics, not two empty sets.
const CORPUS: &[(&str, usize)] = &[
    // positive: borrow-clean over the covered families (empty diag set)
    ("tests/fixtures/selfhost_loans/pos_basic.cnr", 0),
    ("tests/fixtures/selfhost_loans/pos_shared.cnr", 0),
    ("tests/fixtures/selfhost_loans/pos_block.cnr", 0),
    // negative — E0801 conflicting (two exclusive) borrows
    ("tests/fixtures/selfhost_loans/neg_two_excl.cnr", 1),
    // negative — E0802 move-while-borrowed
    ("tests/fixtures/selfhost_loans/neg_move_borrowed.cnr", 1),
    // negative — E0804 read-while-exclusively-borrowed
    ("tests/fixtures/selfhost_loans/neg_read_excl.cnr", 1),
    // negative — E0801 + E0802 (twice, one per live loan) in one program
    ("tests/fixtures/selfhost_loans/neg_mixed.cnr", 3),
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
fn candor_loan_diagnostics_equal_to_oracle_over_corpus() {
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
            "selfhost loans: XOR loan diagnostic PARITY (E0801/E0802/E0804) on {}/{} fixtures, {} covered diagnostics matched",
            passed,
            CORPUS.len(),
            total_diags
        );
    });
}
