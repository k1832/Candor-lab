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
use selfhost_modtree::{run_module_tree, trace_text};

const LEXER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/lexer/lexer.cnr"));
const PARSER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/parser/parser.cnr"));
const ANALYSES_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/analyses/analyses.cnr"));

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
    m.push_str("    let mut buf: Buf = Buf { toks: [mk(0, 0usize, 0usize); 4096], n: 0usize };\n");
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

fn on_big_stack<F: FnOnce() + Send + 'static>(f: F) {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(f)
        .expect("spawn big-stack thread")
        .join()
        .expect("gate thread panicked");
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
