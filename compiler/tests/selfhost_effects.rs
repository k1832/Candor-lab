//! The oracle gate for the SIXTH self-hosted slice: the Candor EFFECT
//! (alloc-partition) analysis and MATCH EXHAUSTIVENESS check — the two remaining
//! self-contained checker analyses that need no CFG/liveness machinery — that
//! extend slices 4-5's move/init/loan analysis in the same
//! `selfhost/analyses/analyses.cnr` (composed after `selfhost/lexer/lexer.cnr`
//! and `selfhost/parser/parser.cnr`). Its canonical diagnostic dump (one
//! `EXXXX START END` line per diagnostic, sorted by (code,start,end)) is asserted
//! byte-equal to the Rust oracle's diagnostics for the SAME source, FILTERED to
//! the families this slice covers (E0401/E0402 effects from `check/effects.rs`,
//! E0601 exhaustiveness from `check/patterns.rs`) and rendered in the identical
//! canonical schema.
//!
//! COVERAGE / BOUNDARY (reported honestly).
//!  * MATCHED (code + span) against the oracle:
//!    - E0401 for the ALLOCATING OPERATIONS whose span the arena now carries:
//!      a CALL to an `alloc`-marked fn / `box` / `unbox` (call span), and
//!      `clone` of a box-bearing value (clone span). The oracle records only the
//!      FIRST such site per function; the Candor walk emits it once, post-order,
//!      matching that ordering.
//!    - E0601 for a non-exhaustive match on an enum (span = the whole match expr,
//!      which the parser now records on the `T_MATCH` node). A wildcard/binding
//!      arm makes the match exhaustive (the oracle's catch-all rule).
//!  * OUT OF SUBSET (deferred, reported):
//!    - E0401 from the IMPLICIT drop of a Box-owning param/local at scope exit —
//!      the oracle span is the synthetic block `join_span` the span-lean arena
//!      does not carry (the effect analog of slice 4's E0302/E0309). The
//!      `unbox`/`clone` cases exercise the freeing/allocating effect with a
//!      matchable span instead.
//!    - E0402 (an `alloc` fn assigned to a non-`alloc` fn-pointer slot) — needs
//!      fn-pointer-TYPE tracking the span-lean arena does not seed.
//!    - Scalar/bool match exhaustiveness: the oracle raises E0603 ("not an enum"),
//!      NOT an E0601 coverage obligation, so matching a scalar is an ERROR rather
//!      than an exhaustiveness case; the Candor analysis likewise emits no E0601
//!      for a non-enum scrutinee (E0603 is out of this slice's covered set).
//!  * BOX-PROPERTY APPROXIMATION: a type is box-bearing iff it transitively
//!    contains `Box`/`BoxResult` (through arrays and struct/enum fields); generic
//!    `Vec[T]`/`String` and unbounded projections — which the oracle also treats
//!    box-bearing — are NOT flagged. The fixtures use explicit `Box`.
//!
//! Harness shape reuses slice 5 (`selfhost_loans.rs`): a generated root
//! `main.cnr` `use`s the `lexer`/`analyses` modules, embeds the fixture source
//! as a `[N]u8` literal, lexes then analyze-dumps; the lexer + parser + analyses
//! tree is loaded with `run_dir` on a 256 MiB thread, and the dump reconstructed
//! from `Run.trace` compared to the filtered+sorted oracle rendering.

use candor::check_source_real;
use candor::RunResult;

mod selfhost_modtree;
use selfhost_modtree::{on_big_stack, run_module_tree, trace_text};

const LEXER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../selfhost/lexer/lexer.cnr"));
const PARSER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../selfhost/parser/parser.cnr"));
const ANALYSES_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../selfhost/analyses/analyses.cnr"));

/// The families the Candor effect/exhaustiveness analysis matches this slice.
const COVERED: &[&str] = &["E0401", "E0402", "E0601"];

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

/// (fixture, expected covered-diagnostic count) — a redundant, human-auditable
/// check that the gate exercises real diagnostics, not two empty sets.
const CORPUS: &[(&str, usize)] = &[
    // positive: effect-clean + exhaustive (empty covered-diag set)
    ("tests/fixtures/selfhost_effects/pos_effect.cnr", 0),
    ("tests/fixtures/selfhost_effects/pos_wild.cnr", 0),
    // negative — E0401 (effect partition): call to alloc fn, unbox (frees a box),
    // and clone of an owned box-bearing value
    ("tests/fixtures/selfhost_effects/neg_call_alloc.cnr", 1),
    ("tests/fixtures/selfhost_effects/neg_unbox.cnr", 1),
    ("tests/fixtures/selfhost_effects/neg_clone_box.cnr", 1),
    // negative — E0601 (exhaustiveness)
    ("tests/fixtures/selfhost_effects/neg_match.cnr", 1),
    // negative — both families in one program
    ("tests/fixtures/selfhost_effects/neg_mixed.cnr", 2),
];

fn read_fixture(rel: &str) -> String {
    let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

#[test]
fn candor_effect_exhaustiveness_diagnostics_equal_to_oracle_over_corpus() {
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
            "selfhost effects: E0401/E0601 diagnostic PARITY on {}/{} fixtures, {} covered diagnostics matched",
            passed,
            CORPUS.len(),
            total_diags
        );
    });
}
