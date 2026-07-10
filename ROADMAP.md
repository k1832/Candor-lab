# Candor roadmap

Rulings by the deciding authority (2026-07-09) on publication and self-hosting; the maturation
queue below is ordered, not dated.

## Publication staging

1. **This repository is the lab, permanently.** The full experiment record — philosophy,
   adversarial reviews, the frozen Bet 5 experiment, designs, gates — is the project's
   distinctive asset and is never diluted into a product repo.
2. **The 0.x preview ships from a separate distribution repository** created at the packaging
   milestone: the toolchain (renamed `candor`), spec, stdlib, editor support, getting-started.
   Explicitly unstable; this repo linked as the design record.
3. **1.0 is the stability gate, not a date:** P15's edition/migrator promises live, P20's
   pre-registered compile-time targets ratified in CI, the spec's obligations ledger clear of
   pre-stability items (NN#20 mechanization decision included). NN#14's Bet 5 condition is
   already satisfied.

## Self-hosting

The compiler is Rust and remains the bootstrap/reference implementation permanently. A
self-hosted compiler is the project's ultimate dogfood (a compiler is the basket's own home
ground), its largest P19 corpus, and its credibility proof — gated on std, not the language:

1. The P3 text-type budget design (the named deferred obligation).
2. An I/O layer as a boundary module over libc (what the P17 boundary exists for).
3. Then port the CHECKER first — highest value, its own domain — with the Rust implementation
   as the differential oracle, per the house methodology.

## Maturation queue (ordered)

- Graduation-tier eval campaign (the first slope-capable measurement).
- Toolchain packaging: candor-proto → candor, install story, the distribution repo (publication
  step 2 above).
- Text-type budget design (P3's named obligation; gates self-hosting and real std growth).
- I/O boundary module (gates self-hosting).
- Bare-metal target (blocked locally on qemu; the freestanding proof stands meanwhile).
- LLVM second backend behind the MIR seam and differential gate.
- Corpus scale-up; per-edition regeneration.
- Self-hosted checker (after text + I/O).
- Stability-gate proceedings (1.0) when the checklist clears.

## Self-hosting arc — STARTED 2026-07-09

Ruling: port onto the interpreter first (AOT extern lowering deferred; a self-hosted checker runs
on the tree-walker, which is a validated engine). Order and gates:
1. **Lexer + parser** (this slice): a Candor .cnr program that lexes and parses Candor source to
   a canonical AST S-expression, gated by S-expr equality against the Rust front-end over the
   whole corpus (the differential methodology; the Rust parser is the oracle).
2. Type checker for the scalar+aggregate core, oracle = the Rust checker's diagnostics.
3. The analyses (move/init, loans, effects), each oracle-gated.
4. Generics, concurrency, text — the frontier.
The Rust implementation remains the permanent bootstrap and oracle. Each slice is a driver: the
Candor port reads source via the std/io module, emits its result, the harness diffs it.

## Self-hosting target RULED: SELF-CHECKING, 2026-07-10

Deciding authority chose the self-checking tier: the Candor-written checker checks its own
source, oracle-matched, culminating in the fixpoint gate — run checker.cnr / analyses.cnr on the
self-host .cnr compiler source and assert the diagnostic set equals the Rust oracle's.
Self-interpreting and true native bootstrap are named as the horizon, NOT targeted now (they need
the str/collection runtime and a backend ported to Candor — prototype-stretching).

**Honesty boundary:** the self-hosted compiler MAY depend on a small set of compiler-known
primitives (Vec, str, Alloc, CharStep) — the runtime/language split every real language has
(cf. Rust core intrinsics), not cheating. Self-checking does not require them in-language.

**Path (targeted enablers pulled by demand, not a speculative std suite):**
1. A std hash map / symbol table — name resolution (item tables, scopes, the keyword ladder) is
   the checker's hot path, currently a linear scan; every remaining slice consumes it.
2. Richer AST spans in the self-host parser — the REAL blocker: the span-lean arena defers
   composite-span diagnostics (E0302/E0309/E0803/E0809); fuller spans unblock matching the oracle
   on all codes.
3. Extend self-host checker coverage to the feature set the self-host SOURCE uses (only that).
4. The fixpoint gate: the self-host checker checks the self-host source, oracle-matched.
