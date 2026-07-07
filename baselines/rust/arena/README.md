# Arena-Based Compiler Pass — Idiomatic Rust Baseline

Bet 5 validation basket, program (e): arena-based IR transformation pass.
Implements the frozen functional specification `docs/basket/spec-arena.md`
(constant folding plus the algebraic identities of §3), producing new IR in a
fresh arena.

## Provenance

- **Type:** **Commissioned** baseline — written fresh for this comparison
  against the frozen spec, not independently sourced. Per BET5_CRITERION.md §2.5
  an independently sourced baseline is *preferred*; this one is commissioned and
  is more exposed to unconscious shaping. Recorded honestly for the adjudicator.
- **Author / model family:** Claude Opus (Anthropic) session.
- **Date:** 2026-07-07.
- **Blindness:** authored reading only `docs/basket/spec-arena.md`,
  `docs/basket/README.md`, and `docs/BET5_CRITERION.md` §2.5. No Candor design
  docs, prototype, or other specs were read.
- **Dependencies:** none. The arena is a small hand-written typed-index arena
  (`Vec<Node>` + `NodeId(usize)`); no external crate is used.

## Design

- `Arena` is a region allocator (`Vec<Node>`). `alloc` appends and returns a
  stable `NodeId`; `reset` releases every node at once (whole-arena reclamation,
  spec §1.3/§2.4) and leaves the arena reusable. There is no per-node freeing.
- `Node` operands are `NodeId`s into the same arena (cross-node references,
  spec §2.6).
- `fold(src, root)` runs a single post-order pass, copying and simplifying each
  node into a **fresh** `dst` arena; `src` is left unmodified and `dst` is fully
  independent (spec §3.4). Within each node the §3.2 rules run in order,
  first-applicable-wins.
- Arithmetic is two's-complement 64-bit wrapping (`wrapping_add/sub/mul/neg`);
  `Div` truncates toward zero via `wrapping_div`, which also yields the sole
  overflow case `i64::MIN / -1 = i64::MIN`. Division by a constant zero is left
  unfolded.

## Layout

- `src/lib.rs` — arena, IR, `fold`, canonical `serialize` (§3.3), `eval` (§3.5).
- `tests/vectors.rs` — the frozen suite AR1–AR29, one `#[test]` per vector ID,
  including the wrapping/boundary vectors (AR19–AR21) and the arena-invariant
  vectors (AR25 independence, AR26 no-dangling, AR27 semantic equivalence,
  AR28 whole-release reuse, AR29 determinism).

## How to run

```sh
cargo test        # runs all 29 frozen vectors (AR1-AR29)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```
