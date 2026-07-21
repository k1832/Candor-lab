# Testing the Candor compiler

House rules for the `compiler/` test suite. This is a description of conventions
already in force, not a proposal.

## Differential byte-exactness

The suite's backbone is differential testing against an oracle: a program is run
through two or more independent engines and their observable output is asserted
**byte-exact** equal. The engines are the tree-walking reference interpreter
(`run_source_real`), the MIR interpreter, the two Cranelift native backends
(no-opt / opt), and — when `clang` is present — the LLVM backend. The canonical
observable is the execution dump: `RET <i64>`, a `TRACE <i64>` line per traced
value, and `FAULT <kindcode> <span.start> <span.end>` for a runtime fault
(including the pre-fault trace — see `selfhost_modtree::dump_fault`). Equality is
over that exact text, so any divergence in return value, trace sequence, or fault
identity is caught.

The self-hosting gates extend this: a Candor-written tool (lexer, parser,
checker, interpreter, lowering) processes each corpus program, and its dump is
asserted byte-exact to the Rust reference for the same source. Passing is
*execution equality between two engines*, not merely "it ran".

## Fast and full profiles

nextest runs every test in its own process across a parallel pool, so wall clock
is the slowest single test, not the sum of the binaries.

- `cargo nextest run` — the full suite; this is what CI runs.
- `cargo nextest run --profile fast` — the inner dev loop; drops the slow
  integration binaries (self-host, native AOT/LLVM/stage gates, freestanding,
  native concurrency, golden). It changes nothing that runs in CI. See
  `compiler/.config/nextest.toml`.

Prefer per-program `#[test]`s over one test that loops a corpus: nextest can only
parallelize across tests, so a single looping test pins the wall clock to the
whole corpus run in one process. The self-host `selfhost_lower` / `selfhost_interp`
gates emit one `#[test]` per corpus program (via a `corpus!` macro) for this
reason.

## The gate-verification rule

**Every soundness GATE must demonstrably fail on the bug it guards.** A gate that
has never been shown to go red on its target regression is unproven — it may be
asserting something the bug does not touch. Before relying on a gate, show it
fail on the violation, one of two ways:

1. **Live regression (manual, not in CI).** Reintroduce the bug in the compiler,
   confirm the gate goes red, then revert. Kept out of CI because it edits the
   compiler.
2. **Structural sensitivity (mechanized, in CI).** Where the gate's decision
   reduces to a testable pure function (a comparator or checker), feed that
   function the exact bad input the bug produces and assert it returns failure —
   and feed it a good input and assert it accepts, so the rejection is not
   vacuous. This is the in-CI stand-in when (1) can't run.

### Precedents

- **fmt write-borrow reborrow (`tests/fmt.rs::fmt_preserves_read_reborrow_of_write_borrow`).**
  The formatter must not collapse `read c.*` (a non-moving reborrow of a `write`
  borrow) to a bare `c` (a move) — the old reborrow-collapse silently did, making
  a later use fail E0301. The gate is sensitivity-by-construction: it feeds the
  exact source the bug broke to the real `format_source_real` and asserts the
  output still contains `peek(read c.*)`. If the collapse regresses, that
  assertion fails.

- **dispatch gate (d) resolve/dispatch drift (`tests/dispatch_gate.rs`).** The
  §2.2 regression keys dispatch on `(target, method)` instead of the resolved
  interface, so an engine dispatches a different impl than the checker resolved
  ("all engines agree on the WRONG impl"). The live-regression check edits the
  compiler, so it can't run in CI; the structural stand-in extracts the gate's
  comparator (`dispatch_matches_resolve`) and `gate_d_comparator_rejects_dispatch_resolve_drift`
  feeds it the exact drift the bug produces (resolved `{A}`, dispatched `{B}`),
  asserting it rejects — and accepts an agreeing pair.
