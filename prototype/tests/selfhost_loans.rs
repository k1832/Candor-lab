//! The oracle gate for the FIFTH self-hosted slice: the Candor LOAN (borrow)
//! analysis — the XOR conflict core of the borrow checker — that extends slice
//! 4's move/init analysis in the same `selfhost/analyses/analyses.cnr` (composed
//! after `selfhost/lexer/lexer.cnr` and `selfhost/parser/parser.cnr`). Its
//! canonical diagnostic dump (one `E08XX START END` line per diagnostic, sorted
//! by (code,start,end)) is asserted byte-equal to the Rust oracle's diagnostics
//! for the SAME source, FILTERED to the loan families the Candor analysis covers
//! (E0801 conflicting borrow, E0802 move-while-borrowed, E0803 write-while-
//! borrowed, E0804 read-while-exclusively-borrowed) and rendered in the identical
//! canonical schema.
//!
//! COVERAGE / BOUNDARY (reported honestly).
//!  * MATCHED (code + span) against the oracle's `loans.rs` XOR conflict scan:
//!    E0801, E0802, E0804 carry the ACCESS site's BARE-IDENTIFIER span (the
//!    borrowed/moved/read place), which the span-lean slice-2 arena stores
//!    verbatim in the `T_ID` node, exactly like slice 4's E0301/E0304. E0803
//!    (write-while-borrowed) carries the whole assignment STATEMENT span, which
//!    the parser now populates into the `T_ASSIGN` node's existing `p0/p1` pair
//!    (leftmost-token-start .. semicolon-end); the span-free S-expr dump is
//!    unchanged. A write conflicts with a live loan of EITHER kind.
//!  * OUT OF SUBSET on the span-lean boundary (the loan analog of slice 4's
//!    E0302/E0309): E0809 (write-through-shared) — its oracle span is this same
//!    whole-statement span (now carried), but emitting it needs per-binding
//!    borrow-TYPE tracking this slice does not perform, so it stays deferred.
//!  * LIVENESS: a RESTRICTED-LEXICAL approximation — a loan is in scope from its
//!    `let` to the end of its enclosing block. This is CONSERVATIVE vs the
//!    oracle's backward NLL-lite liveness (live to the binding's last use), so
//!    the fixtures are restricted to the lexical==NLL region: every borrow
//!    binding is used at/after each conflict point and there is no dead-borrow-
//!    then-reuse; `pos_block` exercises a loan that DROPS at a block exit so a
//!    later move is clean under BOTH liveness models.
//!
//! Harness shape reuses slice 4 (`selfhost_analyses.rs`): a generated root
//! `main.cnr` `use`s the `lexer`/`analyses` modules, embeds the fixture source
//! as a `[N]u8` literal, lexes then analyze-dumps; the lexer + parser + analyses
//! tree is loaded with `run_dir` on a 256 MiB thread, and the dump reconstructed
//! from `Run.trace` compared to the filtered+sorted oracle rendering. The
//! generated `main` is `alloc` (the analysis's Vecs).

use candor_proto::check_source_real;
use candor_proto::RunResult;

mod selfhost_modtree;
use selfhost_modtree::{run_module_tree, trace_text};

const LEXER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/lexer/lexer.cnr"));
const PARSER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/parser/parser.cnr"));
const ANALYSES_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/analyses/analyses.cnr"));

/// The loan families the Candor analysis matches against the oracle this slice.
const COVERED: &[&str] = &["E0801", "E0802", "E0803", "E0804"];

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
    m.push_str("    let mut buf: Buf = Buf { toks: [mk(0, 0usize, 0usize); 8192], n: 0usize };\n");
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
    // negative — E0803 write-while-borrowed (excl loan, then shared loan)
    ("tests/fixtures/selfhost_loans/neg_write_excl.cnr", 1),
    ("tests/fixtures/selfhost_loans/neg_write_shared.cnr", 1),
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
            "selfhost loans: XOR loan diagnostic PARITY (E0801/E0802/E0803/E0804) on {}/{} fixtures, {} covered diagnostics matched",
            passed,
            CORPUS.len(),
            total_diags
        );
    });
}
