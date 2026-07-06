# Spec: Recursive-Descent Parser (`spec-parser.md`)

**Status:** FROZEN on hash. Authored blind to Candor design docs (see README).
**Source obligation:** `BET5_CRITERION.md` §2.4(d). Restates and sharpens; never
weakens. This is a deliberately value-favorable workload.

---

## 1. Purpose & required features

1.1 The program is a **recursive-descent parser** for the fixed non-trivial
expression grammar of §2 ("CalcLang"), producing a **typed AST** (§3).

1.2 The parser MUST implement **operator precedence and associativity** exactly as
§2.4 specifies. A precedence-flat or right-associative shortcut detectably fails.

1.3 The parser MUST **report the first error as a value** — a `{kind, offset}`
pair (§4) — with a **byte offset source position**, never a crash/panic.

1.4 The parser MUST operate over a **slice/view of the input** (zero-copy):
identifier and integer AST nodes reference the input by `(offset, length)` /
sub-slice; the parser MUST NOT copy token text into owned strings. This is
observable via §3.5 (serialization reproduces the exact input substring) and is a
required property, not an optimization.

1.5 Input is a byte string. Bytes are interpreted as ASCII; non-ASCII bytes
outside string/identifier rules are `E_UNEXPECTED_CHAR`.

---

## 2. Grammar (binding)

2.1 **Lexical.** Whitespace (`0x20` space, `0x09` tab, `0x0A` LF, `0x0D` CR) is
skipped and never a token. Tokens:
- `INT` := `[0-9]+` (one or more ASCII digits; the value's magnitude is not
  checked — see Non-goals).
- `IDENT` := `[A-Za-z_][A-Za-z0-9_]*`.
- Punctuation/operators: `+ - * / % ! ( ) ,` and the two-character tokens
  `<= >= == != && ||`, and the one-character `< >`. Two-character tokens are
  matched maximally (`<=` before `<`).
- Any other byte where a token is demanded is `E_UNEXPECTED_CHAR` at its offset.

2.2 **Syntax** (precedence low → high; all binary operators **left-associative**,
unary prefix **right-associative**):
```
expr    := or
or      := and        ( "||"                    and        )*
and     := equality   ( "&&"                    equality   )*
equality:= comparison ( ("==" | "!=")           comparison )*
comparison := additive( ("<" | "<=" | ">" | ">=") additive )*
additive := mul       ( ("+" | "-")             mul        )*
mul     := unary      ( ("*" | "/" | "%")       unary      )*
unary   := ("-" | "!") unary | postfix
postfix := primary    ( "(" args ")" )*        # left-associative call
primary := INT | IDENT | "(" expr ")"
args    := ε | expr ( "," expr )*              # no trailing comma
```

2.3 The top level parses **one** `expr` and then requires end-of-input; any
remaining token is `E_TRAILING_INPUT` at that token's offset.

---

## 3. Typed AST & canonical form

3.1 **Node kinds:** `Int(slice)`, `Ident(slice)`, `Unary(op, operand)`,
`Binary(op, left, right)`, `Call(callee, args[])`. `op` is the operator lexeme.

3.2 **Spans.** Every node carries a byte span `[start, end)` into the input:
`Int`/`Ident` = the token span; `Unary` = `[op.start, operand.end)`; `Binary` =
`[left.start, right.end)`; `Call` = `[callee.start, close_paren.end)`.

3.3 **Canonical serialization** `S(node)` (used for AST-equality across
implementations):
- `Int` → the literal digits verbatim (e.g. `42`).
- `Ident` → the identifier verbatim (e.g. `foo`).
- `Unary` → `(u<op> S(operand))`, i.e. unary minus is `u-`, unary not is `u!`.
- `Binary` → `(<op> S(left) S(right))`.
- `Call` → `(call S(callee) S(arg1) ... S(argN))`; zero args → `(call S(callee))`.
Tokens in `S(...)` are space-separated; there is exactly one canonical string per
AST.

3.4 **AST equality** = string equality of `S(root)`. Two implementations parsing
the same valid input MUST produce the same `S(root)`.

3.5 **Zero-copy check.** For every `Int`/`Ident` node, the substring of the input
at the node's span MUST equal the token text emitted in `S`. This makes the
slice-referencing requirement (1.4) observable.

---

## 4. Observable behavior & frozen test vectors

Result is `Ok(S(root))` or `Err{kind, offset}`. Only the **first** error is
reported. `offset` is a 0-based byte index; end-of-input errors use
`offset == len(input)`. Error kinds: `E_UNEXPECTED_CHAR`, `E_UNEXPECTED_EOF`,
`E_EXPECTED_EXPR`, `E_EXPECTED_RPAREN`, `E_TRAILING_INPUT`.

### Valid — precedence & associativity
- **P1** `42` → `42`; root span `[0,2)`.
- **P2** `foo` → `foo`.
- **P3** `1+2` → `(+ 1 2)`.
- **P4** `1 + 2 * 3` → `(+ 1 (* 2 3))`.
- **P5** `1 * 2 + 3` → `(+ (* 1 2) 3)`.
- **P6** `(1 + 2) * 3` → `(* (+ 1 2) 3)`.
- **P7** `1 - 2 - 3` → `(- (- 1 2) 3)` (left-assoc).
- **P8** `-a` → `(u- a)`; root span `[0,2)`.
- **P9** `--a` → `(u- (u- a))` (unary right-assoc; `--` lexes as two `-`).
- **P10** `!a && b` → `(&& (u! a) b)`.
- **P11** `a < b == c` → `(== (< a b) c)` (comparison binds tighter than `==`).
- **P12** `a || b && c` → `(|| a (&& b c))` (`&&` tighter than `||`).
- **P13** `f(1, 2)` → `(call f 1 2)`.
- **P14** `f()` → `(call f)`.
- **P15** `f(1)(2)` → `(call (call f 1) 2)` (chained-call left-assoc).
- **P16** `1 + 2 * 3 - 4 / 2` → `(- (+ 1 (* 2 3)) (/ 4 2))`.
- **P17** `a % b * c` → `(* (% a b) c)` (`%` same level as `*`, left-assoc).
- **P18** `  1\n+\t2 ` (leading/inner/trailing whitespace) → `(+ 1 2)`; the `Int`
  `1` node span is `[2,3)` and the `Int` `2` span is `[5,6)` (positions ignore
  skipped whitespace).
- **P19** `((((5))))` → `5`.
- **P20** `a >= b != c <= d` → `(!= (>= a b) (<= c d))`.

### Invalid — kind + position
- **P21** `` (empty) → `E_UNEXPECTED_EOF` at offset `0`.
- **P22** `1 +` → `E_UNEXPECTED_EOF` at offset `3` (operand expected after `+`).
- **P23** `1 + )` → `E_EXPECTED_EXPR` at offset `4` (`)` where a primary is
  required).
- **P24** `(1 + 2` → `E_EXPECTED_RPAREN` at offset `6` (`)` expected, EOF).
- **P25** `1 2` → `E_TRAILING_INPUT` at offset `2` (second token after a complete
  expression).
- **P26** `@` → `E_UNEXPECTED_CHAR` at offset `0`.
- **P27** `1 + 2 @` → `E_UNEXPECTED_CHAR` at offset `6` (lex error surfaces when
  the trailing token is demanded, before trailing-input is diagnosed).
- **P28** `f(1,)` → `E_EXPECTED_EXPR` at offset `4` (no trailing comma; `)` where
  an argument expression is required).
- **P29** `)` → `E_EXPECTED_EXPR` at offset `0`.
- **P30** `a && ` → `E_UNEXPECTED_EOF` at offset `5`.

### Structural cross-checks
- **P31.** For every valid vector, re-parsing `S(root)` is **not** required, but
  the §3.5 zero-copy substring check MUST hold for all leaf nodes.
- **P32.** Parsing is deterministic: the same input yields the same
  `Ok`/`Err` result on every run and every conforming implementation.

---

## 5. Non-goals

5.1 **Evaluation / semantic analysis** (computing the expression's value, name
resolution, type-checking) is NOT required.
5.2 **Integer-magnitude / overflow checking** of `INT` literals is NOT a parser
concern; a literal is any digit run.
5.3 **Error recovery / multiple-error reporting** is NOT required; the parser
stops at and reports the first error only.
5.4 **Unicode / non-ASCII identifiers, floats, string literals, comments** are out
of scope; the grammar is exactly §2.
5.5 **Performance** (throughput, allocation count) is NOT graded, only AST
equality, error `{kind, offset}` correctness, and the zero-copy property
(criterion §8.2).
5.6 Building a **CST / preserving whitespace and comments** is out of scope; only
the typed AST of §3 is produced.
