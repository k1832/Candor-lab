# Feasibility assessment: restoring the fmt reborrow collapse (backlog item)

*Fresh-context assessment, 2026-07-25. Question: should the reborrow collapse
removed from the canonical formatter in d4e0fb4 be restored in a type-aware
form? Verdict first, then the findings that outlived verification.*

## Verdict: KEEP IT REMOVED. Backlog item closed as won't-do.

Neither a full type-aware collapse nor a syntactic subset clears the
risk/reward bar. The collapse only ever bought brevity (`read b.*` -> `b`); the
cases where it is safe are exactly the cases that were never broken, and the
uncollapsed form is arguably clearer. The costs:

- **Full type-aware collapse:** the formatter is parse-only by design — a pure
  deterministic function of one file, able to format code that does not
  compile. Routing checker types in requires running resolve+check inside fmt,
  a span-keyed type table that does not exist (`Expr` has no id), whole-module-
  tree assembly for imports, and "conditional fidelity" (the same file
  formatting differently depending on whether its project type-checks), which
  breaks the corpus idempotency gates. Large blast radius for a cosmetic win.
- **Syntactic subset** (collapse only `read (b.*)` where `b` is *declared*
  `read` in scope): sound, but needs a new lexical scope tracker in the
  emitter with shadowing/capture handling — real bug surface to shorten the
  cases that were never dangerous.

When a collapse IS sound: exactly when `b`'s type is a shared/`read` borrow
(`Type::Borrow` is Copy — `compiler/src/types.rs`); a `write` borrow is never
Copy, so bare `b` moves it. The formatter cannot see this distinction, which is
the whole story.

## Finding that mattered more than the question: the migrator still collapsed

The assessment found the **P15 migrator** (`compiler/src/real/emit.rs`,
`emit_borrow_op`) still performed the exact collapse d4e0fb4 removed from fmt.

**Empirical narrowing (recorded because the initial claim was broader):** the
assessment predicted the fmt failure mode (E0301 after a call-argument
collapse). Testing against the current checker showed call-argument position is
now an *implicit-reborrow* position — `peek(c)` with `c: write Cell` passed to
a `read` parameter checks clean, so the collapse was legal there (this changed
with the reborrow-ceremony redesign, one of the Bet-5 successor commitments).
The genuine breakage is **return position**: `return read (deref c);` in a
function returning `read Cell` migrated to `return c;`, which fails E0703
(found `borrow_mut`, expected `borrow`) while the original checks clean.
Binding position would also break, but the throwaway dialect cannot spell
borrow-typed lets, so return position is the reachable case.

**Fix (same commit series as this memo):** the collapse is removed from the
migrator, mirroring d4e0fb4; `read (deref b)` re-spells as `read b.*`. Pinned
by the `reborrow_return` fixture (check + run parity), committed
11_4_parser/11_5_arena `.cnr` fixtures regenerated, stale doc comments in
emit.rs and fmt.rs corrected (fmt's module doc still listed the collapse among
the canonical rules and claimed it was shared with the migrator).

**Second finding, from the pre-commit adversarial review of that fix:** the
precedence tables in BOTH the migrator and the formatter still classified a
borrow-of-deref at postfix level — a leftover tuned to the removed collapse.
Consequence: a borrow used as a field/index *base* lost its load-bearing
parens. In the migrator this would have been a new regression exposed by the
collapse removal (`(read (deref c)).v` -> `read c.*.v`, E0703); in the
formatter it was a LIVE bug shipped since d4e0fb4 — `candor fmt` reformatted
the working `(read c.*).v` into the borrow-broken `read c.*.v`. Both `expr_bp`
branches now return prefix level unconditionally; pinned by
`fmt_preserves_parens_around_borrow_used_as_base` and the `base_v` function in
the check-side `reborrow_return` fixture. The initial re-spell claim ("sound in
every position") was therefore too broad as stated: it holds only with the
precedence fix, which restores the grouping parens. Position audit after both
fixes: call-argument (implicit reborrow, collapse was legal), return and
binding (mistranslated, fixed by removal), postfix base (mistranslated by the
leftover precedence, fixed), each verified empirically.

## Standing observations for the record

- The corpus behavior gate (`corpus_format_preserves_behavior`) now catches a
  naive fmt collapse reintroduction — but by luck of corpus content
  (`wasm_interp.cnr` carries write-borrow reborrows), not by construction. The
  write-borrow reborrow shape now also exists as the named `reborrow_return`
  run fixture.
- The gate's run comparator checks `ret` + `trace` only; a semantics change
  preserving both plus diagnostics would be invisible. Acceptable residual for
  a formatter with no remaining rewrite rules of this class.
- If a future canonicalization mandate forces the collapse back, the only
  defensible path is the declared-`read`-only syntactic subset, gated by new
  fixtures: a write-param reborrow that must not collapse, shadowing where an
  inner `write b` hides an outer `read b`, and a region-parametric read
  reborrow in a storing context.
