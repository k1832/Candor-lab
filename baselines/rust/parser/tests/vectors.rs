//! Frozen test vectors P1-P32 from `docs/basket/spec-parser.md` §4.
//! Each test is named by its vector ID.

use candor_parser_baseline::{parse, parse_ast, ErrorKind, Node};

/// Assert a valid input serializes to `expected` (spec §3.4) and that every
/// leaf node's span selects exactly its token text (the §3.5 span/substring
/// check, i.e. vector P31).
fn assert_valid(input: &str, expected: &str) {
    let bytes = input.as_bytes();
    let ast = parse_ast(bytes).expect("expected a valid parse");
    assert_eq!(ast.serialize(), expected, "S(root) mismatch for {input:?}");

    let mut leaves = Vec::new();
    ast.collect_leaves(&mut leaves);
    for (start, end, text) in leaves {
        let substring = std::str::from_utf8(&bytes[start..end]).unwrap();
        assert_eq!(
            substring, text,
            "span/substring check (P31) failed for {input:?}"
        );
    }
}

fn assert_error(input: &str, kind: ErrorKind, offset: usize) {
    let err = parse(input.as_bytes()).expect_err("expected an error");
    assert_eq!(err.kind, kind, "error kind mismatch for {input:?}");
    assert_eq!(err.offset, offset, "error offset mismatch for {input:?}");
}

// --- Valid: precedence & associativity ------------------------------------

#[test]
fn p1_int() {
    assert_valid("42", "42");
    let ast = parse_ast(b"42").unwrap();
    assert_eq!(ast.span(), (0, 2)); // root span [0,2)
}

#[test]
fn p2_ident() {
    assert_valid("foo", "foo");
}

#[test]
fn p3_add() {
    assert_valid("1+2", "(+ 1 2)");
}

#[test]
fn p4_add_binds_looser_than_mul() {
    assert_valid("1 + 2 * 3", "(+ 1 (* 2 3))");
}

#[test]
fn p5_mul_binds_tighter_than_add() {
    assert_valid("1 * 2 + 3", "(+ (* 1 2) 3)");
}

#[test]
fn p6_parens_override() {
    assert_valid("(1 + 2) * 3", "(* (+ 1 2) 3)");
}

#[test]
fn p7_sub_left_assoc() {
    assert_valid("1 - 2 - 3", "(- (- 1 2) 3)");
}

#[test]
fn p8_unary_minus() {
    assert_valid("-a", "(u- a)");
    let ast = parse_ast(b"-a").unwrap();
    assert_eq!(ast.span(), (0, 2)); // root span [0,2)
}

#[test]
fn p9_unary_right_assoc() {
    assert_valid("--a", "(u- (u- a))");
}

#[test]
fn p10_unary_not_with_and() {
    assert_valid("!a && b", "(&& (u! a) b)");
}

#[test]
fn p11_comparison_binds_tighter_than_equality() {
    assert_valid("a < b == c", "(== (< a b) c)");
}

#[test]
fn p12_and_binds_tighter_than_or() {
    assert_valid("a || b && c", "(|| a (&& b c))");
}

#[test]
fn p13_call_two_args() {
    assert_valid("f(1, 2)", "(call f 1 2)");
}

#[test]
fn p14_call_zero_args() {
    assert_valid("f()", "(call f)");
}

#[test]
fn p15_chained_call_left_assoc() {
    assert_valid("f(1)(2)", "(call (call f 1) 2)");
}

#[test]
fn p16_mixed_precedence() {
    assert_valid("1 + 2 * 3 - 4 / 2", "(- (+ 1 (* 2 3)) (/ 4 2))");
}

#[test]
fn p17_percent_same_level_as_mul() {
    assert_valid("a % b * c", "(* (% a b) c)");
}

#[test]
fn p18_whitespace_spans() {
    let input = "  1\n+\t2 ";
    assert_valid(input, "(+ 1 2)");
    let ast = parse_ast(input.as_bytes()).unwrap();
    let mut leaves = Vec::new();
    ast.collect_leaves(&mut leaves);
    assert_eq!(leaves[0], (2, 3, "1")); // Int `1` span [2,3)
    assert_eq!(leaves[1], (6, 7, "2")); // Int `2` span [6,7)
}

#[test]
fn p19_nested_parens() {
    assert_valid("((((5))))", "5");
}

#[test]
fn p20_comparison_and_equality_chain() {
    assert_valid("a >= b != c <= d", "(!= (>= a b) (<= c d))");
}

// --- Invalid: kind + position ---------------------------------------------

#[test]
fn p21_empty_input() {
    assert_error("", ErrorKind::UnexpectedEof, 0);
}

#[test]
fn p22_operand_expected_after_plus() {
    assert_error("1 +", ErrorKind::UnexpectedEof, 3);
}

#[test]
fn p23_rparen_where_primary_required() {
    assert_error("1 + )", ErrorKind::ExpectedExpr, 4);
}

#[test]
fn p24_missing_rparen() {
    assert_error("(1 + 2", ErrorKind::ExpectedRparen, 6);
}

#[test]
fn p25_trailing_token() {
    assert_error("1 2", ErrorKind::TrailingInput, 2);
}

#[test]
fn p26_unexpected_char() {
    assert_error("@", ErrorKind::UnexpectedChar, 0);
}

#[test]
fn p27_lex_error_before_trailing_input() {
    assert_error("1 + 2 @", ErrorKind::UnexpectedChar, 6);
}

#[test]
fn p28_no_trailing_comma_in_args() {
    assert_error("f(1,)", ErrorKind::ExpectedExpr, 4);
}

#[test]
fn p29_rparen_at_start() {
    assert_error(")", ErrorKind::ExpectedExpr, 0);
}

#[test]
fn p30_operand_expected_after_and() {
    assert_error("a && ", ErrorKind::UnexpectedEof, 5);
}

// --- Structural cross-checks ----------------------------------------------

/// P31: for every valid vector the §3.5 span/substring check holds for all
/// leaf nodes. `assert_valid` performs this check inline; here we assert it
/// explicitly across the full valid corpus.
#[test]
fn p31_span_substring_all_valid_vectors() {
    let corpus = [
        "42",
        "foo",
        "1+2",
        "1 + 2 * 3",
        "1 * 2 + 3",
        "(1 + 2) * 3",
        "1 - 2 - 3",
        "-a",
        "--a",
        "!a && b",
        "a < b == c",
        "a || b && c",
        "f(1, 2)",
        "f()",
        "f(1)(2)",
        "1 + 2 * 3 - 4 / 2",
        "a % b * c",
        "  1\n+\t2 ",
        "((((5))))",
        "a >= b != c <= d",
    ];
    for input in corpus {
        let bytes = input.as_bytes();
        let ast = parse_ast(bytes).expect("valid corpus entry");
        let mut leaves = Vec::new();
        ast.collect_leaves(&mut leaves);
        for (start, end, text) in leaves {
            assert_eq!(
                &bytes[start..end],
                text.as_bytes(),
                "P31 failed for {input:?}"
            );
        }
    }
}

/// P32: parsing is deterministic; the same input yields the same result.
#[test]
fn p32_determinism() {
    let inputs = ["1 + 2 * 3", "f(1)(2)", "1 + )", "@"];
    for input in inputs {
        let first = parse(input.as_bytes());
        for _ in 0..8 {
            assert_eq!(
                parse(input.as_bytes()),
                first,
                "non-deterministic for {input:?}"
            );
        }
    }
}

/// The op field carries the operator lexeme (spec §3.1); a couple of direct
/// AST-shape assertions to guard against serialization masking a wrong tree.
#[test]
fn ast_shape_binary() {
    let ast = parse_ast(b"1+2").unwrap();
    match ast {
        Node::Binary { op, .. } => assert_eq!(op, "+"),
        other => panic!("expected Binary, got {other:?}"),
    }
}
