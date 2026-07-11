//! Trait-generics front-end SMOKE gate (slice 1 of the trait-generics arc): the
//! self-hosted lexer + parser (`selfhost/lexer/lexer.cnr`,
//! `selfhost/parser/parser.cnr`) are run over the five deferred trait-generic
//! fixtures -- which use `interface`/`impl`/`for` syntax OUT OF SUBSET for the
//! byte-exact AST-dump gate -- to prove they now LEX and PARSE with no fault or
//! parse error. The generated root `main` embeds a fixture, lexes it, then calls
//! the non-dumping `parse_count`, returning the arena node count; a successful
//! run with a non-trivial count means the interface/impl nodes are in the arena.
//!
//! This slice lands nothing runnable on its own (mono/interp/lower dispatch of
//! interface/impl is deferred to later slices), so this is the verification that
//! the parse support works.

use candor_proto::RunResult;

mod selfhost_modtree;
use selfhost_modtree::run_module_tree;

const LEXER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/lexer/lexer.cnr"));
const PARSER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/parser/parser.cnr"));

/// The five trait-generic fixtures blocked on interface/impl parsing. Each uses
/// an `interface` decl and an `impl ... for ...` block (some generic).
const TRAIT_FIXTURES: &[&str] = &[
    "tests/fixtures/generics/iface.cnr",
    "tests/fixtures/generics/gimpl.cnr",
    "tests/fixtures/generics/gbound.cnr",
    "tests/fixtures/generics/fromq.cnr",
    "tests/fixtures/generics/gfromq.cnr",
];

fn read_fixture(rel: &str) -> String {
    let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

/// Generate the root `main.cnr`: `use`s the lexer's `Buf`/`mk`/`lex` and the
/// parser's `parse_count`, embeds `src`, lexes then parse-counts it, returning the
/// arena node count.
fn candor_main(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut m = String::from(
        "use lexer::{Buf, mk, lex};\nuse parser::{parse_count};\n\nfn main() -> i64 {\n",
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
    m.push_str("    let _cnt: usize = lex(slice_of(src), write buf);\n");
    m.push_str("    let nc: usize = parse_count(slice_of(src), read buf);\n");
    m.push_str("    return conv i64 nc;\n}\n");
    m
}

fn parse_node_count(src: &str) -> i64 {
    let main = candor_main(src);
    match run_module_tree(&[("lexer.cnr", LEXER_SRC), ("parser.cnr", PARSER_SRC)], &main) {
        RunResult::Ok(run) => run.ret,
        RunResult::Fault(f) => panic!("self-host parser faulted on trait fixture: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!(
            "self-host parser tree check errors: {:?}",
            d.iter().map(|x| &x.code).collect::<Vec<_>>()
        ),
        RunResult::ParseError(d) => {
            panic!("self-host parser parse error on trait fixture: {}", d.to_json())
        }
    }
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
fn selfhost_parser_parses_trait_generic_fixtures() {
    on_big_stack(|| {
        for &rel in TRAIT_FIXTURES {
            let src = read_fixture(rel);
            let count = parse_node_count(&src);
            // A completed run (no fault / parse error) with a non-trivial arena means
            // the interface/impl nodes were built. The arena fills from index 1, so a
            // genuine parse of these multi-item fixtures lands far more than a handful.
            assert!(
                count > 20,
                "trait fixture {rel} parsed to only {count} arena nodes; expected a full parse"
            );
            eprintln!("selfhost traitparse: {rel} -> {count} arena nodes (no parse error)");
        }
    });
}
