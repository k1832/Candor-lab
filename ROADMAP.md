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

**Fixpoint scoping RESULT (2026-07-10):** the gate is ~a handful of small slices, not N. The
self-host source is structurally clean where it matters (zero persistent loans; no
box/unbox/clone/match/enums), so the deferred error codes (E0302/E0309/E0809/E0401/E0601) are
UNREACHABLE over it — out of scope for self-checking (they matter only for negative error
fixtures). The gate reduces to "the checker traverses the whole source without false-rejecting,
oracle-matched (∅)." First sub-goal DONE: the self-host checker checks lexer.cnr clean (leaf
module, arena growth only).

**Fixpoint path RULED: HONEST import-resolution (deciding authority, 2026-07-10)** — over the
pragmatic concatenate-into-one-blob alternative. Each module is checked IN ISOLATION with its
`use` imports resolved, faithful to the module tree we built (not a concatenated blob) and with a
smaller max input (parser.cnr ~20.7k tokens vs a ~42k blob → better tree-walker perf). Cost: a
real feature (use/pub in the self-host lexer/parser/checker) plus possibly adopting the std Map
into the checker's currently-linear name scan for runtime. Slice order: import-resolution (the
gate) → per-module arena growth → per-module clean gates (parser/checker/analyses) →
analyses-clean (E0301/E0304, empirically inspected for move/init false positives) → Map-in-checker
perf if needed.

**Scheduled — modularize the self-host source (dogfood the module system).** The four
self-host `.cnr` files use zero `use` imports; the oracle harness composes them by
`include_str!` concatenation (convenience, not a language limit — modules stage 1 works,
proven by `tests/fixtures/modules/ok_tree/`). The self-host compiler is the one substantial
Candor program that does NOT exercise modules. Split it into a proper `use`/`pub` tree
(lexer / parser / checker / analyses as real modules sharing `Tok`/`Node`/state via named
imports), teaching the oracle harness to load a module tree instead of concatenating. Expected
to surface real stage-1 module-ergonomics obligations (cross-module type refs need named
imports not alias paths; no `pub use` re-exports) — that friction is valuable signal, logged to
99-obligations.md. Serialized: touches `selfhost/` + every oracle harness, so it runs SOLO,
after the Map enabler and outside any compiler-crate agent's window. Slot: after step 1 (Map)
lands, before the step-3 coverage slices — a split, better-navigable source eases those.

## Self-interpreting — sliced plan of record (2026-07-10)

The next self-hosting tier: a tree-walking interpreter written in Candor
(`selfhost/interp/interp.cnr`) that EXECUTES Candor programs, oracle-matched
(byte-exact `Run{ret,trace}` + fault identity) against the Rust `src/interp/`.
Built in slices; each is solo/serial (writes the crate). Scoped by two read-only
design passes (S3 drop semantics; blockers A/B/C).

- **S1 DONE** — scalar interp (ints/bool, control flow, arithmetic+faults, calls,
  recursion). Overflow without i128 via wrapping-then-decide. 24 fixtures.
- **S2 DONE** — flat byte-memory model + structs/arrays. Value model converged onto
  addressed memory (S1 fixtures still byte-exact). 36 fixtures.
- **S3** — move/drop schedule + trace-on-drop. DOABLE BEFORE HEAP: the drop trace
  comes from a user `drop(write self)` hook calling `trace`, no Box needed. Reverse/
  LIFO order, hook-then-fields, move-suppresses-drop, fault aborts without dropping.
  Recomputes ownership during the walk (not from analyses). Extends the type table
  with is_copy/needs_drop + struct->drophook lookup. 8 fixtures to author.
- **S4** — match/enums (RESEQUENCED before Box: every Box systems fixture matches
  BoxResult). Tag@0/payload@8.
- **S5** — Box/BoxResult + allocator ABI ({ptr,ctx,vt}=24, structural Alloc discovery).
- **S6** — rawptr/fnptr/MMIO + pointer intrinsics + the dense [N]u8 Mem (blocker C,
  ~80 lines, 4-8 MiB, no paging — corpus max address is 3 MiB).
  **MILESTONE: after S6 all five systems-corpus programs run** (11_1..11_5
  allocator/scheduler/mmio/parser/arena) — the non-generic, non-container corpus.
- **S7** — slices/str + std Vec/Map/String (fat pointers=16, Vec/Map=40).
- **S8** — the monomorphizer (blocker B, ~700-900 lines). Gates ONLY the generic
  library tail; the systems corpus is monomorphic, so B is descopable indefinitely.
- **S9** — conv/contracts + whole-corpus close-out.

**Blocker verdicts (design memo):** A (type/layout table) is the TALLEST POLE —
the self-host checker does no type inference, so A must BUILD annotation-directed
type synthesis (`ty_of`), ~1000 lines grown incrementally across S2-S7, with no
oracle to differentially test against. B (monomorphizer) is late and optional.
C (address space) is nearly free (dense arena, no paging). ~6 slices / ~3500-4500
lines of Candor remain to the whole corpus; the systems-corpus milestone (S6) is
much nearer.
