# Candor Bet 5 — Rust Baseline: Expression Parser

Idiomatic Rust baseline for the recursive-descent CalcLang parser defined by
`docs/basket/spec-parser.md`, one of the five Bet 5 validation-basket programs
(`docs/BET5_CRITERION.md` §2.4(d), §2.5).

## Provenance

- **Status:** COMMISSIONED. Written fresh against the frozen functional
  specification for this comparison. Per criterion §2.5 an independently sourced
  baseline is *preferred* over a commissioned one because a commissioned
  baseline is more exposed to unconscious shaping (§6.7); this baseline is
  commissioned, and that provenance is recorded here honestly.
- **Authored by:** Claude (Opus model family) session.
- **Date:** 2026-07-07.
- **Blindness:** authored reading only `docs/basket/spec-parser.md`,
  `docs/basket/README.md`, and `docs/BET5_CRITERION.md` §2.5. No Candor design
  docs, prototype code, or other basket specs were consulted.

## What it implements

- The §2 grammar with the mandated precedence and associativity: all binary
  operators left-associative, unary prefix (`-`, `!`) right-associative,
  left-associative postfix calls.
- The typed AST of §3 with a `[start, end)` byte span on every node.
- Errors returned as `{kind, offset}` values (§4) with exact byte offsets;
  never a panic on malformed input. Kinds: `E_UNEXPECTED_CHAR`,
  `E_UNEXPECTED_EOF`, `E_EXPECTED_EXPR`, `E_EXPECTED_RPAREN`,
  `E_TRAILING_INPUT` (see `ErrorKind`).
- The canonical S-expression serialization `S(root)` used for AST equality
  (§3.3).
- **Zero-copy (§1.4):** `Int`/`Ident` nodes hold a `&str` sub-slice borrowed
  from the input; no token text is copied into owned storage. The operator
  lexeme on `Unary`/`Binary` is a `&'static str`, not input-derived text.

## Layout

- `src/lib.rs` — lexer, recursive-descent parser, AST, and serialization.
- `tests/vectors.rs` — every numbered vector P1–P32, named by ID.

## How to run

```sh
cargo test          # runs all P1-P32 vectors
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```
