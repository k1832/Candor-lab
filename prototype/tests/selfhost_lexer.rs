//! The oracle gate for the FIRST self-hosted slice: a Candor lexer
//! (`selfhost/lexer/lexer.cnr`) is run on the tree-walker over each real-syntax
//! corpus fixture, and its canonical token dump is asserted byte-equal to the
//! Rust oracle lexer's (`src/real/lexer.rs`) dump of the same source. Passing
//! this gate is token-stream EQUALITY between the two lexers.
//!
//! Harness interface (documented in the .cnr header): the Candor lexer is a
//! library module (`lex` + `dump`); this harness loads it as a module tree with
//! a generated root `main.cnr` that `use`s the lexer, embeds the fixture's source
//! bytes as a `[N]u8` literal, calls `lex` into a fixed `Buf`, and calls `dump`,
//! which emits each output BYTE of the canonical text through the built-in
//! `trace` sink. We reconstruct the text from `Run.trace` and compare it to the
//! oracle's byte-identical rendering. Loading (via `run_dir`) dogfoods the
//! stage-1 module system (design 0008) instead of string-concatenating sources.

use candor_proto::real::token::{RTok, RToken};
use candor_proto::token::ScalarTy;
use candor_proto::RunResult;

mod selfhost_modtree;
use selfhost_modtree::{run_module_tree, trace_text};

const LEXER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/selfhost/lexer/lexer.cnr"));

fn suf_code(s: Option<ScalarTy>) -> u32 {
    match s {
        None => 0,
        Some(ScalarTy::I8) => 1,
        Some(ScalarTy::I16) => 2,
        Some(ScalarTy::I32) => 3,
        Some(ScalarTy::I64) => 4,
        Some(ScalarTy::Isize) => 5,
        Some(ScalarTy::U8) => 6,
        Some(ScalarTy::U16) => 7,
        Some(ScalarTy::U32) => 8,
        Some(ScalarTy::U64) => 9,
        Some(ScalarTy::Usize) => 10,
        Some(ScalarTy::Bool) | Some(ScalarTy::Unit) => unreachable!("not a suffix"),
    }
}

fn punct_code(k: &RTok) -> u32 {
    match k {
        RTok::LParen => 10,
        RTok::RParen => 11,
        RTok::LBrace => 12,
        RTok::RBrace => 13,
        RTok::LBracket => 14,
        RTok::RBracket => 15,
        RTok::Comma => 16,
        RTok::Dot => 17,
        RTok::DotStar => 18,
        RTok::Semi => 19,
        RTok::Colon => 20,
        RTok::ColonColon => 21,
        RTok::Arrow => 22,
        RTok::FatArrow => 23,
        RTok::Question => 24,
        RTok::Eq => 25,
        RTok::EqEq => 26,
        RTok::Ne => 27,
        RTok::Lt => 28,
        RTok::Le => 29,
        RTok::Gt => 30,
        RTok::Ge => 31,
        RTok::Plus => 32,
        RTok::Minus => 33,
        RTok::Star => 34,
        RTok::Slash => 35,
        RTok::Percent => 36,
        RTok::Amp => 37,
        RTok::Pipe => 38,
        RTok::Caret => 39,
        RTok::Tilde => 40,
        RTok::Shl => 41,
        RTok::Shr => 42,
        RTok::AmpAmp => 43,
        RTok::PipePipe => 44,
        RTok::Bang => 45,
        other => panic!("not a punct token: {other:?}"),
    }
}

/// Render the oracle's token stream in the canonical dump form the .cnr lexer
/// produces (see the .cnr header for the format).
fn oracle_dump(src: &str) -> String {
    let toks: Vec<RToken> = candor_proto::real::lexer::lex(src).expect("oracle lexes the fixture");
    let bytes = src.as_bytes();
    let mut out = String::new();
    for t in &toks {
        let s = t.span.start;
        let e = t.span.end;
        match &t.kind {
            RTok::Ident(_) => {
                out.push_str(&format!("1 {s} {e} "));
                out.push_str(std::str::from_utf8(&bytes[s..e]).unwrap());
            }
            RTok::Kw(_) => {
                out.push_str(&format!("2 {s} {e} "));
                out.push_str(std::str::from_utf8(&bytes[s..e]).unwrap());
            }
            RTok::Scalar(_) => {
                out.push_str(&format!("3 {s} {e} "));
                out.push_str(std::str::from_utf8(&bytes[s..e]).unwrap());
            }
            RTok::Int { value, suffix } => {
                out.push_str(&format!("4 {s} {e} {value} {}", suf_code(*suffix)));
            }
            RTok::Str(buf) => {
                out.push_str(&format!("5 {s} {e} {}", buf.len()));
                for b in buf.as_bytes() {
                    out.push_str(&format!(" {b}"));
                }
            }
            RTok::Bytes(buf) => {
                out.push_str(&format!("6 {s} {e} {}", buf.len()));
                for b in buf.as_bytes() {
                    out.push_str(&format!(" {b}"));
                }
            }
            RTok::Eof => out.push_str(&format!("99 {s} {e}")),
            other => out.push_str(&format!("{} {s} {e}", punct_code(other))),
        }
        out.push('\n');
    }
    out
}

/// Generate the root `main.cnr`: it `use`s the lexer module's `Buf`/`mk`/`lex`/
/// `dump`, embeds `src` as a byte-array literal, lexes it, and dumps the stream.
fn candor_main(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut m = String::from("use lexer::{Buf, mk, lex, dump};\n\nfn main() -> i64 {\n");
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
    m.push_str("    dump(slice_of(src), read buf);\n");
    m.push_str("    return conv i64 cnt;\n}\n");
    m
}

/// Run the Candor lexer on `src` as a `lexer` + `main` module tree, returning the
/// reconstructed canonical dump.
fn candor_dump(src: &str) -> String {
    let main = candor_main(src);
    match run_module_tree(&[("lexer.cnr", LEXER_SRC)], &main) {
        RunResult::Ok(run) => trace_text(&run),
        RunResult::Fault(f) => panic!("candor lexer faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!(
            "candor lexer check errors: {:?}",
            d.iter().map(|x| &x.code).collect::<Vec<_>>()
        ),
        RunResult::ParseError(d) => panic!("candor lexer parse error: {}", d.to_json()),
    }
}

/// The real-syntax corpus the Candor lexer is gated against.
const CORPUS: &[&str] = &[
    "tests/fixtures/real/bits.cnr",
    "tests/fixtures/real/propagate.cnr",
    "tests/fixtures/real/slices.cnr",
    "tests/fixtures/generics/arena.cnr",
    "tests/fixtures/generics/fromq.cnr",
    "tests/fixtures/generics/gbound.cnr",
    "tests/fixtures/generics/gdrop.cnr",
    "tests/fixtures/generics/gdrop_groundfloor.cnr",
    "tests/fixtures/generics/genenum.cnr",
    "tests/fixtures/generics/gfromq.cnr",
    "tests/fixtures/generics/gimpl.cnr",
    "tests/fixtures/generics/iface.cnr",
    "tests/fixtures/generics/mixed.cnr",
    "tests/fixtures/generics/mono3.cnr",
    "tests/fixtures/generics/nameval.cnr",
    "tests/fixtures/generics/pair.cnr",
];

fn read_fixture(rel: &str) -> String {
    let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

#[test]
fn candor_lexer_token_equal_to_oracle_over_corpus() {
    let mut passed = Vec::new();
    for &rel in CORPUS {
        let src = read_fixture(rel);
        let oracle = oracle_dump(&src);
        let mine = candor_dump(&src);
        assert_eq!(mine, oracle, "token stream mismatch on {rel}");
        passed.push(rel);
    }
    assert_eq!(passed.len(), CORPUS.len());
    eprintln!(
        "selfhost lexer: token-equality PASS on {}/{} corpus fixtures",
        passed.len(),
        CORPUS.len()
    );
}

/// A focused unit over the operator/edge-token set (maximal munch, `.*`, `::`,
/// hex + suffix, the bitwise set, `?`), independent of the corpus.
#[test]
fn candor_lexer_token_equal_on_operator_probe() {
    let src = "a::b .* -> => == != <= >= << >> && || ! ~ & | ^ ? 0x1F 42u8 0 255usize -1";
    assert_eq!(candor_dump(src), oracle_dump(src), "operator probe mismatch");
}
