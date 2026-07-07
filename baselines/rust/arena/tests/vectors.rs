//! Frozen test suite from spec-arena.md §4 (AR1-AR29).
//!
//! Each numbered vector is one `#[test]`, named by its vector ID. The source IR
//! is built directly in an arena (spec §5.6) via a tiny S-expression reader that
//! mirrors the canonical form of §3.3, so each vector reads like the spec.

use candor_arena::{eval, fold, serialize, Arena, Node, NodeId};

/// Build the IR described by canonical S-expression `s` into `arena`, returning
/// its root. Test-only convenience; the library itself does no parsing.
fn build(arena: &mut Arena, s: &str) -> NodeId {
    let spaced = s.replace('(', " ( ").replace(')', " ) ");
    let tokens: Vec<&str> = spaced.split_whitespace().collect();
    let mut pos = 0;
    let id = parse(arena, &tokens, &mut pos);
    assert_eq!(pos, tokens.len(), "trailing tokens in {s:?}");
    id
}

fn parse(arena: &mut Arena, tokens: &[&str], pos: &mut usize) -> NodeId {
    assert_eq!(tokens[*pos], "(", "expected '('");
    *pos += 1;
    let tag = tokens[*pos];
    *pos += 1;
    let node = match tag {
        "c" => {
            let v: i64 = tokens[*pos].parse().unwrap();
            *pos += 1;
            Node::Const(v)
        }
        "v" => {
            let v: u32 = tokens[*pos].parse().unwrap();
            *pos += 1;
            Node::Var(v)
        }
        "neg" => {
            let a = parse(arena, tokens, pos);
            Node::Neg(a)
        }
        "+" | "-" | "*" | "/" => {
            let a = parse(arena, tokens, pos);
            let b = parse(arena, tokens, pos);
            match tag {
                "+" => Node::Add(a, b),
                "-" => Node::Sub(a, b),
                "*" => Node::Mul(a, b),
                _ => Node::Div(a, b),
            }
        }
        other => panic!("unknown tag {other:?}"),
    };
    assert_eq!(tokens[*pos], ")", "expected ')'");
    *pos += 1;
    arena.alloc(node)
}

/// Build `input`, fold it, and assert the folded IR serializes to `expected`.
fn check(input: &str, expected: &str) {
    let mut src = Arena::new();
    let root = build(&mut src, input);
    let (dst, new_root) = fold(&src, root);
    assert_eq!(serialize(&dst, new_root), expected);
}

// --- Constant folding ---

#[test]
fn ar1() {
    check("(+ (c 1) (c 2))", "(c 3)");
}

#[test]
fn ar2() {
    check("(* (c 3) (c 4))", "(c 12)");
}

#[test]
fn ar3() {
    check("(- (c 10) (c 4))", "(c 6)");
}

#[test]
fn ar4() {
    check("(neg (c 5))", "(c -5)");
}

#[test]
fn ar9() {
    check("(/ (c 8) (c 2))", "(c 4)");
}

#[test]
fn ar10() {
    check("(/ (c 7) (c 2))", "(c 3)");
}

#[test]
fn ar11() {
    check("(/ (c -7) (c 2))", "(c -3)");
}

// --- Algebraic identities ---

#[test]
fn ar5() {
    check("(+ (v 0) (c 0))", "(v 0)");
}

#[test]
fn ar6() {
    check("(* (v 0) (c 1))", "(v 0)");
}

#[test]
fn ar7() {
    check("(* (v 0) (c 0))", "(c 0)");
}

#[test]
fn ar8() {
    check("(/ (v 0) (c 1))", "(v 0)");
}

#[test]
fn ar13() {
    check("(neg (neg (v 0)))", "(v 0)");
}

#[test]
fn ar14() {
    check("(- (c 0) (v 0))", "(neg (v 0))");
}

#[test]
fn ar23() {
    check("(* (c 1) (* (c 1) (v 0)))", "(v 0)");
}

// --- Div-by-zero & wrapping boundaries ---

#[test]
fn ar12() {
    check("(/ (c 5) (c 0))", "(/ (c 5) (c 0))");
}

#[test]
fn ar24() {
    check("(/ (v 0) (v 1))", "(/ (v 0) (v 1))");
}

#[test]
fn ar19() {
    check("(* (c 9223372036854775807) (c 2))", "(c -2)");
}

#[test]
fn ar20() {
    check(
        "(- (c -9223372036854775808) (c 1))",
        "(c 9223372036854775807)",
    );
}

#[test]
fn ar21() {
    check(
        "(/ (c -9223372036854775808) (c -1))",
        "(c -9223372036854775808)",
    );
}

// --- Nested & cascading ---

#[test]
fn ar15() {
    check("(+ (* (c 2) (c 3)) (v 0))", "(+ (c 6) (v 0))");
}

#[test]
fn ar16() {
    check("(+ (* (v 0) (c 0)) (c 5))", "(c 5)");
}

#[test]
fn ar17() {
    check("(* (+ (c 1) (c 1)) (+ (v 0) (c 0)))", "(* (c 2) (v 0))");
}

#[test]
fn ar22() {
    check("(+ (+ (c 1) (c 2)) (+ (c 3) (c 4)))", "(c 10)");
}

#[test]
fn ar18() {
    check("(+ (v 0) (v 1))", "(+ (v 0) (v 1))");
}

// --- Structural / arena invariants ---

/// AR25: independence (§3.4). After fold, resetting `src` must not change the
/// serialization of the result: `dst` holds its own copy of every node.
#[test]
fn ar25() {
    let mut src = Arena::new();
    let root = build(&mut src, "(+ (* (c 2) (c 3)) (v 0))");
    let (dst, new_root) = fold(&src, root);
    src.reset();
    assert_eq!(serialize(&dst, new_root), "(+ (c 6) (v 0))");
}

/// AR26: no dangling (§3.4). Every NodeId reachable from `new_root` resolves
/// within `dst` (its index is a live index of `dst`).
#[test]
fn ar26() {
    let mut src = Arena::new();
    let root = build(&mut src, "(* (+ (c 1) (c 1)) (+ (v 0) (c 0)))");
    let (dst, new_root) = fold(&src, root);

    // Collect every id reachable from new_root and confirm each resolves in dst.
    let mut stack = vec![new_root];
    while let Some(id) = stack.pop() {
        assert!(id.index() < dst.len(), "id {id:?} dangles outside dst");
        match dst.get(id) {
            Node::Const(_) | Node::Var(_) => {}
            Node::Add(a, b) | Node::Sub(a, b) | Node::Mul(a, b) | Node::Div(a, b) => {
                stack.push(a);
                stack.push(b);
            }
            Node::Neg(a) => stack.push(a),
        }
    }
    assert_eq!(serialize(&dst, new_root), "(* (c 2) (v 0))");
}

/// AR27: semantic equivalence (§3.5). For AR15 with v0 = 7, both source and
/// result evaluate to 13.
#[test]
fn ar27() {
    let mut src = Arena::new();
    let root = build(&mut src, "(+ (* (c 2) (c 3)) (v 0))");
    let (dst, new_root) = fold(&src, root);
    let assign = |v: u32| -> i64 {
        assert_eq!(v, 0);
        7
    };
    assert_eq!(eval(&src, root, &assign), 13);
    assert_eq!(eval(&dst, new_root, &assign), 13);
}

/// AR28: whole-release reuse (§3.3/2.4). Build AR22, fold, reset the source
/// arena, then reuse it to build AR2 and fold again.
#[test]
fn ar28() {
    let mut src = Arena::new();
    let root22 = build(&mut src, "(+ (+ (c 1) (c 2)) (+ (c 3) (c 4)))");
    let (d1, r1) = fold(&src, root22);
    assert_eq!(serialize(&d1, r1), "(c 10)");

    src.reset();
    assert!(src.is_empty());

    let root2 = build(&mut src, "(* (c 3) (c 4))");
    let (d2, r2) = fold(&src, root2);
    assert_eq!(serialize(&d2, r2), "(c 12)");
}

/// AR29: determinism (§3.6). Re-running a vector yields a byte-identical result.
#[test]
fn ar29() {
    let run = || {
        let mut src = Arena::new();
        let root = build(&mut src, "(+ (* (c 2) (c 3)) (v 0))");
        let (dst, new_root) = fold(&src, root);
        serialize(&dst, new_root)
    };
    assert_eq!(run(), run());
    assert_eq!(run(), "(+ (c 6) (v 0))");
}
