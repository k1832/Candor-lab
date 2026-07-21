# 99 — Obligations Tracker

**Status: NORMATIVE (process).** Every SKELETON and ADOPTED-PENDING item in this
specification is tracked here with its **philosophy hook**, its **gate** (what it
blocks), and its **acceptance criterion** (what discharges it). An obligation is
a debt, not an absolution (philosophy header warning): listing it does not manage
it. None of these open items licenses undefined behavior in safe code (NN#1); each
chapter's stated obligations bind now.

---

## 1. Pre-stability hard gates (named in the philosophy)

### OBL-WINDOW — imprecise fault-window formalization
- **Chapter:** 06 §7 (NORMATIVE-DRAFT single-threaded window; §7.5 concurrency
  composition SKELETON).
- **Status:** **discharged** (single-threaded edition, 2026-07-21; ruling J1,
  `docs/1.0-GATE-TRIAGE.md`). The rigorous-informal single-threaded core
  (Containment, Prefix-determinism, NN#1/NN#5 preservation, collapse-to-precise)
  is promoted into chapter 06 §7.4; mechanization is **preferred, not required**
  (J1). Concurrency composition (O1–O5) deferred as a bundle with the atomics
  surface (OBL-CONSIST). See the status update below.
- **Hook:** P5 (the fault-window bound); **NN#20** (named-novel, mandatory
  pre-stability formalization); philosophy §8 critical path; NN#1/NN#5.
- **Gate:** blocks any optimizing implementation's soundness claim and any
  stability commitment ("1.0"). The prototype (no optimizer, precise trap) is not
  blocked.
- **Acceptance:** the fault model is formalized as **mechanized** (strongly
  preferred) or **at minimum rigorous-informal**, proving the §7.2 window bound
  composes soundly with the adopted consistency model (chapter 09), preserving
  NN#1 and NN#5.

### OBL-ALIAS — unsafe-code aliasing model
- **Chapter:** 05 §6 (NORMATIVE-DRAFT).
- **Status:** **discharged** (single-threaded edition, 2026-07-21; atomics
  composition deferred with OBL-CONSIST). The adversarial review's one repair
  condition is met — see the status update below.
- **Hook:** **P18** (named mandatory spec scope); NN#1 (which quietly rests on it).
- **Gate:** blocks any optimizing implementation's soundness claim and any
  stability commitment.
- **Acceptance:** the spec states, for every unsafe operation of chapter 05 §2,
  the optimizer assumptions it preserves or breaks — notably the materialization
  question (former 05 §6.2, now the §6.4 author obligation) — composed with chapter
  09; Rust's Stacked/Tree Borrows studied as cautionary art (05 §6.5); mechanized
  where feasible, rigorous-informal at minimum.

### OBL-CONSIST — memory consistency model adoption
- **Chapter:** 09 (ADOPTED-PENDING).
- **Hook:** **P18** (adopt proven art — C/C++/Rust axis); P10 (first-version
  no-concurrency posture).
- **Gate:** blocks any concurrency feature and any atomics surface.
- **Acceptance:** the adopted C/C++20-family clauses (SC-for-DRF, atomics, fences)
  are transcribed and chapter 09 §4's decisions made, composed soundly with
  OBL-ALIAS and OBL-WINDOW.

---

## 2. Feature-gated obligations

### OBL-CONTRACT — `audit` and `assumed-proven` full specification
- **Chapter:** 07 §5 (SKELETON). The optimizer-never-assumes rule (07 §4) is
  **already normative** at all three levels; only the levels' machinery is open.
- **Hook:** **P8** (contract levels; the settled optimizer rule).
- **Gate:** blocks shipping `audit`/`assumed-proven`; `enforced` is fully
  specified and unblocked.
- **Acceptance:** the spec defines the structured recording format for `audit`
  violations and the enumeration surface for `assumed-proven` contracts.

### OBL-FFI — boundary modules and the foreign-trust effect
- **Chapter:** 08 §6 (SKELETON).
- **Hook:** **P17/NN#18** (foreign boundary as audit surface); **NN#19** (closed
  effect set grows only by amendment with conservative-default migration).
- **Gate:** blocks any FFI feature and the P17 audit story; the pure-Candor core
  is unblocked.
- **Acceptance:** the spec defines boundary modules, the foreign-trust effect's
  typing and one-way partition, and the toolchain enumeration surface.
- **Update (2026-07-09):** the boundary-module *marker* now has a SKELETON home in
  chapter 11 §10.1 (file-level `boundary`, part of the interface artifact,
  `candor audit --boundaries`); the FFI *content* remains deferred to design 0011
  and this obligation.

### OBL-GENERICS — user-defined generics
- **Chapter:** 03 §1.5 (excluded this edition).
- **Hook:** **P11** (definition-site-checked generics with predictable
  instantiation).
- **Gate:** blocks generic user types/functions; the compiler-known parametric
  types (`Box`, `slice`, `[N]T`, ...) are unblocked.
- **Acceptance:** the spec defines the interface/bound system, definition-site
  checking, coherence, and the documented, stable-within-edition instantiation
  strategy.
- **Discharged (2026-07-09):** chapter 10 promoted to NORMATIVE-DRAFT — the
  interface/bound system (the `copy` built-in, `[T: I]`/`+`, self modes, self-less
  associated functions, the single associated-type member, uniform per-impl effect
  markers), impls and the six conformance axes, coherence (module-granularity
  orphan rule, instantiated-interface uniqueness key, unifiability rejection,
  params-in-target), definition-site checking (opaque conservatism, the
  conformance-error-not-type-error distinction), instantiation/inference
  (expected-type + fn-pointer, `name::[T]`, no call-site turbofish), and
  monomorphization determinism with the depth-limit resource bound are transcribed
  from design 0007 (with the associated-type member of 0009 §2). Acceptance met.

### OBL-TEXT — the text-type budget
- **Chapter:** 03 §8.3 (byte slices only this edition).
- **Hook:** **P3** (one canonical way; the named string-sprawl stress point).
- **Gate:** blocks any owning/interop text type beyond `[u8]` / `[N]u8`.
- **Acceptance:** the spec resolves text under P3 with a defended budget (one
  universal view type; minimum owning/interop forms, each by recorded
  justification).

### OBL-GENERIC-BRACKET — the generic-bracket token constraint
- **Chapter:** design 0006 §3 and §7 (a constraint on the future P11 round),
  recorded here at spec tier per review #1 disposition 12.
- **Hook:** **P11** (generics, deferred); **NN#13** (tokenize/parse without a
  symbol table).
- **Gate:** binds the eventual generics design — its type-argument bracketing
  **must not** be `<>`, so that `>>` stays unambiguously the right-shift operator
  (design 0006 added `<<`/`>>`; a `<>` overload would reintroduce the C++
  `>>`-vs-close-angle-bracket parse hazard NN#13 forbids). No `<…>` generic syntax
  may be adopted.
- **Acceptance:** OBL-GENERICS's design uses a bracketing token other than `<>`
  for type arguments, preserving the design-0006 shift tokenization; discharged
  when OBL-GENERICS is discharged consistent with this constraint.
- **Discharged (2026-07-09):** design 0007 §6.1 adopts `[…]` type-argument
  brackets, never `<>` (`>>` stays the shift operator); transcribed in chapter 10
  §1.1. Discharged with OBL-GENERICS.

---

## 3. Skeleton-chapter obligations (bind to design 0006)

### OBL-LEX — real-language lexical structure
- **Chapter:** 01 (SKELETON). **Hook:** **NN#13** (tokenize without a symbol
  table); P13; NN#11 (formatter is the only form).
- **Gate:** blocks any stability commitment; the abstract-construct semantic core
  (03–08) is unblocked.
- **Acceptance:** design 0006's token inventory is transcribed, satisfying §1's
  obligations; the prototype fixture (design 0002) is superseded, not codified.
- **Discharged (2026-07-08):** chapter 01 promoted to NORMATIVE-DRAFT — design
  0006's token inventory (keywords with hard/contextual status, the full bitwise
  set, the `.*` single-token rule, integer-literal suffix and bare-over-range-
  error rules, string/comment/identifier lexis), maximal-munch rules and their
  documented boundaries, whitespace-insignificance, and the NN#13 tokenization-
  without-context clause (01 §6) are transcribed. Acceptance met.

### OBL-GRAM — real-language grammar
- **Chapter:** 02 (SKELETON). **Hook:** **NN#13** (parse without a symbol table);
  **NN#17/P2** (nothing crosses a signature by inference).
- **Gate:** blocks any stability commitment; the semantic core is unblocked.
- **Acceptance:** design 0006's productions are transcribed, realizing complete
  signatures (modes, regions, effects, contracts) and the permissive-parse /
  strict-check separation.
- **Discharged (2026-07-08):** chapter 02 promoted to NORMATIVE-DRAFT — the
  complete real EBNF (items with the `ok` marker, types incl. `read[r] [T]` and
  `[N]T`, signatures with take-by-omission/regions/`alloc`/contracts, statements,
  expressions with the normative Rust-precedence table, `conv` scalar-keyword
  production, `?`, the negative-literal fold, patterns, the match-arm boundary
  rule, `unsafe`/regime blocks, `field_ptr`), the two-token-lookahead ceiling and
  NN#13 walks (02 §10), and the canonical-form clauses (02 §9) are transcribed.
  Acceptance met.

---

## 4. Real-toolchain (non-language) obligations

### OBL-FMT-REBORROW — canonical formatter normalizes explicit reborrow
- **Chapter:** 04 §3.4. **Hook:** **P16/NN#11** (one canonical form); **P3**;
  design 0005 P3 disposition.
- **Gate:** blocks the "exactly one reborrow spelling in argument position"
  guarantee in the real toolchain; the prototype has no formatter and accepts both
  forms.
- **Acceptance:** the shipped, **type-aware** formatter rewrites
  `f(write b.*)` / `f(read b.*)` to `f(b)` exactly where the argument
  fills a matching `read`/`write`-mode parameter, leaving `take`-mode borrow
  arguments and non-place arguments alone.

---

## 5. Obligation count

**Twelve tracked obligations:** three pre-stability hard gates (OBL-WINDOW,
OBL-ALIAS, OBL-CONSIST), four feature gates (OBL-CONTRACT, OBL-FFI, OBL-GENERICS,
OBL-TEXT), one P11-round token constraint (OBL-GENERIC-BRACKET), two
skeleton-chapter gates (OBL-LEX, OBL-GRAM), one real-toolchain gate
(OBL-FMT-REBORROW), and one design-0001 hole found by transcription
(OBL-SLICE-REGION). The two skeleton-chapter gates (OBL-LEX, OBL-GRAM) are
**discharged** (2026-07-08) by the NORMATIVE-DRAFT promotion of chapters 01/02.

Of these, **OBL-WINDOW, OBL-ALIAS, and OBL-CONSIST are the philosophy-named
pre-stability tier** (P18, NN#20): no "1.0" precedes their discharge (chapter 00
§3.4). OBL-ALIAS is **discharged** for the no-concurrency edition (2026-07-21; chapter 05
§6 NORMATIVE-DRAFT; the review's one repair condition is met, below), its atomics
composition deferred with OBL-CONSIST. OBL-WINDOW is likewise **discharged** for
the single-threaded edition (2026-07-21; chapter 06 §7 single-threaded window
NORMATIVE-DRAFT; ruling J1), its concurrency composition (O1–O5) deferred with
OBL-CONSIST; mechanization preferred-not-required (J1).

## OBL-SLICE-REGION (found by transcription, 2026-07-08)

Chapter 04's transcription exposed a design-0001 hole, confirmed against the prototype: slice
parameters count toward §3.3's borrow-parameter test (E0807 fires) but no region variable can be
spelled on a slice type, so a function with one slice + one other borrow parameter returning a
borrow is unwritable. Ruling queued with the 0006 revision: region tags become spellable on
slice-typed parameters and returns (real-syntax spelling decided in 0006; the prototype gains the
throwaway form); 0001 §3.3 amended to state the counting rule it already implements.
Gate: blocks chapter 04's NORMATIVE promotion for the affected clause.
Resolution (2026-07-08, review #1): real-syntax spelling `read[r] [T]` /
`write[r] [T]` adopted in design 0006 §2.2; 0001 §3.3 amended to state the
counting rule (slice parameters count as borrow parameters); prototype stopgap
`slice[r] T` / `slice_mut[r] T` added.

## OBL-ALIAS status update (2026-07-21)

Chapter 05 §6 promoted from SKELETON to NORMATIVE-DRAFT, discharging OBL-ALIAS
rigorous-informal (**discharged-pending-review**; deciding-authority ruling J2,
`docs/1.0-GATE-TRIAGE.md`, made this REQUIRED pre-1.0 because the shipping
Cranelift/LLVM backends optimize). The section states, as an upper bound (P2): the
pointer taxonomy and each kind's aliasing set (§6.2), the most a conforming
optimizer MAY assume (§6.3 — safe-borrow non-aliasing from the checker's XOR only;
no `rawptr` assumption; observable ordering for MMIO/rawptr; nothing from a
contract, P8), the one aliasing UB the checker cannot check and `unsafe` therefore
carries (§6.4 — no `rawptr` aliasing a live `write`/`read` borrow), and the
adopted-art position (§6.5 — C-style "rawptr may alias anything," explicitly **not**
a Rust Stacked/Tree-Borrows model for the valve; future noalias tightening is a
breaking edition change requiring a migrator, P15/NN#14).

Verified for honesty (non-normative Appendix 05-A): the backends assume strictly
**less** than §6.3 grants — no `!tbaa`/`noalias`/`!alias.scope` in the LLVM emitter,
default `MemFlags::new()` (no `noalias`/`readonly`) in Cranelift, the MIR pass only
elides dead pure non-faulting `τ`-steps, and every `rawptr` deref is marked
observable and lowered as a barrier call (fully volatile). No backend emits an
assumption a `rawptr` could falsify, so §6.4.1's obligation is un-exercised today;
it is stated now so a future noalias-deriving backend inherits it, not discovers it.

**Review + discharge (2026-07-21).** Adversarially reviewed
(`docs/reviews/05-aliasing-model-review.md`): verdict SOUND-WITH-REPAIRS, one
REPAIR required before dropping "discharged-pending-review" — "observable" was
undefined for non-faulting executions. Repaired: chapter 06 §8 now defines
**observable effect** normatively for **any** execution (MMIO accesses, foreign /
boundary calls, `trace`, program completion) with the program-order-among-
observables rule the optimizer owes (§8.2); chapter 05 §6.2.5 / §6.3.1(c) anchor to
it instead of the fault-window clauses. The §6.5.3 future-`noalias` seam was
tightened to state that a borrow-based `noalias` tightening **withdraws** §6.4.1's
`rawptr`-read carve-out (foreign reads through an aliasing `rawptr` become UB, as
LLVM `noalias` requires) — a restatement of §6.4 by amendment with a migrator, not
merely a narrowing of sound programs. OBL-ALIAS is thus **discharged** for the
single-threaded edition.

Open (deferred with the concurrency edition): the composition with atomics and the
full chapter-09 consistency model (OBL-CONSIST); mechanization (preferred, not
required). The single-threaded, no-atomics edition is discharged.

## OBL-WINDOW status update (2026-07-08)

The rigorous-informal single-threaded core is drafted, adversarially reviewed (three
theorem-breaks found and repaired: docs/reviews/2026-07-08-fault-window-review-1.md), and
revised (docs/spec/drafts/fault-window-formalization.md). Discharged: Containment,
Prefix-determinism, NN#1/NN#5 preservation, and the collapse-to-precise limit under a sound
effect-order-total reordering license. Open: concurrency composition (stated as conjecture with
obligations O1-O5), mechanization, compiler correctness, and the fault-torn multi-store
transaction residual (deferred to the volatile-access design).

## OBL-WINDOW status update (2026-07-21)

Chapter 06 §7's single-threaded fault window is promoted from SKELETON to
NORMATIVE-DRAFT, discharging OBL-WINDOW for the concurrency-free edition (ruling
J1, `docs/1.0-GATE-TRIAGE.md`). The draft's discharged core — Containment
(Theorem 1), Prefix-determinism (Theorem 2, program-order-first fault identity),
NN#1/NN#5 preservation, and collapse-to-precise — is stated normatively in
chapter 06 §7.4; `docs/spec/drafts/fault-window-formalization.md` is retained as
the **proof artifact** the chapter cites. Per J1, **mechanization is preferred,
not required** for this edition. The §7.2 window bound is now a normative
property, not a draft claim.

Open (deferred as a bundle with the atomics edition): the **concurrency
composition** — the synchronization half of the §7.2 bound — and its obligations
O1–O5 (draft §12), all CONJECTURE and blocked on the atomics surface (chapter 09
§4, OBL-CONSIST); mechanization (preferred); compiler-correctness (out of scope
by construction, draft §13.2); and the fault-torn multi-store transaction
residual (deferred to the volatile-access design). Chapter 06 §7.5 records the
scope; chapter 09 §4.3 gates any concurrency feature on the composition's
discharge.

## OBL-MINMAX-INTRINSICS (found by editor-support work, 2026-07-08)

Spec 01 §2.3 lists min_of/max_of as normative intrinsics; the real front-end defers them (stage-1
report) and the lexer table omits them. Resolve by implementing the two intrinsics (trivial
compile-time constants) or downgrading the spec entry to reserved; the grammar highlights them
per spec meanwhile. Gate: chapter 01 NORMATIVE promotion for that clause.

Resolution/re-scope (2026-07-09): the prototype front-end never implemented
min_of/max_of and the negative-literal fold (chapter 02 §6.6) covers the
programmatic-bound use. Spec 01 §2.3 is **downgraded to reserved-but-not-
implemented** (the honest state); the tokens stay reserved so the spelling is
available. The obligation stays **open, re-scoped**: implement or drop
min_of/max_of at the **real-toolchain gate**. Chapter 01's NORMATIVE promotion
for that clause no longer blocks on an unimplemented feature.

## OBL-GENERICS-ITER evidence update (stdlib seed, 2026-07-08)

The seed gathered the concrete friction: (a) no closures/higher-order inference makes Opt::map
unwritable (E1002: U uninferable from a fn-pointer return; no turbofish; the opaque-drop alloc
tax compounds it); (b) no swap/replace plus E0303 plus struct-only drop hooks make "mutable
container with a user drop hook" mutually exclusive - containers free structurally or hooks live
on wrapper cells. Both feed the iteration/associated-types design round.

**E1002 RESOLVED (2026-07-12).** Friction (a)'s inference half is closed: the call-site
unifier (`unify`, `src/check/generics.rs`) now descends `fn(..)->..` formal parameters, so `U`
in `map[T,U](Opt[T], fn(T)->U) -> Opt[U]` is inferred from the fn-pointer argument's return
type. `Opt::map` checks clean and runs on the tree-walker, MIR, and native engines
(`opt_map_end_to_end`); a genuinely uninferable parameter still raises E1002. The remaining
higher-order friction is OBL-GENERICS-CLOSURE (capturing closures), not inference. This
discharges the STD-FMT F2 / design 0009 §5.3 inference obligation.


---

## 6. Generics, modules, and iteration — discharges and new obligations (designs 0007/0008/0009)

### OBL-GENERICS-ITER — iteration and associated types
- **Discharged (2026-07-09):** design 0009 supplied the pressure cases 0007 §1.1
  required; the minimal associated type (chapter 12 §1), the two by-value protocols
  `Iter`/`Indexed` with their `for` desugar (chapter 12 §§2–4), and capture-free
  higher-order code (chapter 12 §6.2) are transcribed. Its two residuals become
  OBL-ITER-BORROW and OBL-GENERICS-CLOSURE below.

### OBL-MODULES-ARTIFACT — the P20 interface-artifact / incrementality machinery
- **Chapter:** 11 §§7–10 (SKELETON). **Hook:** **P20** (signature-bounded
  incremental invalidation); NN#17; P17 (boundary); P9 (layering).
- **Gate:** blocks the P20 incrementality guarantee — interface artifacts, the
  signature/codegen two-hash tiers, the schema/toolchain salt, `inline`, `pub use`
  facades and external reachability, `foo.cd`-beside-`foo/` directory bodies, the
  boundary-marker semantics, package identity, and core/std layering. The shipped
  multi-file subset (chapter 11 §§1–6: file=module, `use`, `pub`, acyclic DAG,
  `::` lookahead) is unblocked.
- **Acceptance:** the spec transcribes the interface-artifact format, the two-hash
  invalidation tiers with the schema salt, `pub use` external reachability,
  directory bodies, and the manifest/layering rules; the boundary FFI content
  composes with OBL-FFI.

### OBL-ITER-PIN (found by transcription, 2026-07-09) — the `for` library-shape mechanism
- **Chapter:** 12 §2.3 (recorded in-place). **Hook:** **P11/P3** (the `for` desugar's
  library targets; one canonical way).
- **Gate:** blocks a complete normative account of how `for` resolves its targets.
- **Found:** design 0009 fixes the interface *shapes* (`Iter`/`Indexed`/`IterStep`)
  but does **not** specify the *resolution mechanism* — compiler-known lang-items on
  a blessed `core` path versus ordinary in-scope name lookup — and it uses
  `Opt[T]`/`Opt::Some`/`Opt::None` without defining `Opt` in designs 0007–0009.
  Chapter 12 §2.3 adopts "well-known `core` interfaces, shapes fixed by the spec"
  and records the mechanism as unestablished.
- **Acceptance:** the name-resolution design fixes the `core` well-known-item
  mechanism (or a chosen alternative), and `Opt`'s normative shape is established.

### OBL-REGION-KEYWORD-GRAM (found by transcription, 2026-07-09) — region-keyword grammar coordination
- **Chapter:** 02 §4, 04 §7, 01 (recorded). **Hook:** **NN#13/P2** (grammar
  coherence); design 0007 §6.1.1.
- **Gate:** blocks chapter 02/04 internal consistency with chapter 10.
- **Found:** design 0007 §6.1.1's `region r` keyword form supersedes the bare-`[r]`
  region list, and is transcribed in chapter 10 §1.3, but chapter 02 §4
  (`Regions`/`Mode`) and chapter 04 §7 still spell the bare `[r]` form; chapter 01
  has no contextual-keyword entry for `for`/`in`/`type` or a status for `region`.
  These three chapters made only the authorized single for-statement edit to
  chapter 02 (§8.3); the region-keyword and lexical-keyword updates are recorded,
  not resolved.
- **Acceptance:** chapters 01/02/04 updated to the `region r` form and the
  contextual-keyword inventory (`for`, `in`, `type`).
- **Discharged (2026-07-09):** chapters 01/02/04 updated to the `region r`
  declaration form — chapter 02 §4's `Regions` production and new clause 4.6,
  chapter 04 §7.2 — and chapter 01 §2.5's contextual-keyword inventory extended
  with `for`, `in`, `type`, `region` (`ok` already present). The region **use**
  tag keeps the bare `[r]` on borrow types; only declarations wear the keyword.
  Acceptance met.

### OBL-ITER-BORROW (new, design 0009 §3.4)
- **Chapter:** 12 §6.1 (SKELETON pointer). **Hook:** **P12/P11** (value-first; the
  no-borrow-field rule of chapter 04 §8 / chapter 10 §2.3).
- **Gate:** blocks borrowed-item iteration (`read Item`), mutating iteration
  (`write coll`), and non-consuming pointer-chain (`List`) iteration.
- **Acceptance:** chapter 10 §2.3's region-parameterized-type question reopened, or
  a region-free borrowed-yield model found.

### OBL-GENERICS-CLOSURE (new, design 0009 §5.4)
- **Chapter:** 12 §6.2. **Hook:** **P12/P2** (visible value over hidden environment).
- **Gate:** blocks full capturing closures; capture-free threading (fn-pointer +
  explicit `ctx`) is the shipped surface.
- **Acceptance:** BOTH (a) a basket-grade measured case where capture-free threading
  is the dominant reading-friction or expressiveness failure, AND (b) a capture
  model compatible with chapter 10 §2.3 — move/copy captures only, closure a
  synthesized owned aggregate with ordinary drop glue and alloc-on-drop, borrow
  captures remaining refused.

### OBL-GENERIC-STRATEGY (new, design 0007 §5.3)
- **Chapter:** 10 §10.1. **Hook:** **P11** (strategy overridable where cost demands).
- **Gate:** blocks the source-level instantiation-strategy override; the
  monomorphization default is unblocked.
- **Acceptance (trigger):** a measured code-size regression on the basket (or its
  stdlib successor) attributable to monomorphization bloat.

### OBL-GENERIC-TURBOFISH (new, design 0007 §6.2)
- **Chapter:** 10 §9.3. **Hook:** **P11/NN#13** (generics without a symbol table).
- **Gate:** blocks call-site explicit type arguments; inference + type-position
  annotations cover the shipped surface.
- **Acceptance:** a basket case where inference + annotations cannot cover, resolved
  by a keyword-led turbofish preserving the NN#13 tokenization.

### OBL-GENERIC-EFFECT (new, design 0007 §3.4)
- **Chapter:** 10 §5.5; chapter 12 §5.2. **Hook:** **P2/NN#19** (the closed,
  non-transforming effect set).
- **Gate:** blocks per-instantiation effect polymorphism; conservative def-site
  alloc-marking (the opaque-drop tax) is the shipped rule.
- **Acceptance:** a measured, real opaque-drop `alloc` tax on the basket justifies a
  per-instantiation `alloc` variable (so e.g. `Container[u8]::drop` is provably
  non-alloc).

### Prototype inference note — E1002 (design 0009 §5.3)
- **Chapter:** 10 §9.2. Inferring a type parameter from a fn-pointer argument's
  *return* type is a completeness gap **within** design 0007 §2.2 body-local
  inference — an implementation obligation with **no surface change**, to close as
  the corelib port's own commit.

## Obligation count update (2026-07-09)

The generics/modules/iteration round **discharges three** prior obligations
(OBL-GENERICS, OBL-GENERIC-BRACKET, OBL-GENERICS-ITER) and **adds eight** tracked
obligations: OBL-MODULES-ARTIFACT (one SKELETON-chapter gate for chapter 11
§§7–10), OBL-ITER-BORROW, OBL-GENERICS-CLOSURE, OBL-GENERIC-STRATEGY,
OBL-GENERIC-TURBOFISH, OBL-GENERIC-EFFECT (five feature/expressiveness gates), and
**two found by transcription** — OBL-ITER-PIN (the `for` library-shape resolution
mechanism, and `Opt` undefined) and OBL-REGION-KEYWORD-GRAM (the `region r`
grammar coordination across chapters 01/02/04). The E1002 inference note is an
implementation obligation with no surface. The philosophy-named pre-stability tier
(OBL-WINDOW, OBL-ALIAS, OBL-CONSIST) is unchanged; no "1.0" precedes its discharge
(chapter 00 §3.4).

## Spec-maintenance pass (2026-07-09)

Chapters 03/04 re-spelled from the throwaway forms (`borrow T`/`borrow_mut T`/
`slice T`/`slice_mut T`/`deref b`) into the real syntax (`read T`/`write T`/
`[T]`/`write [T]`/`.*`; design 0006), semantics untouched — chapter 04 §3's
reborrow content already cited design 0005 and was unchanged. **OBL-REGION-
KEYWORD-GRAM discharged** and **OBL-MINMAX-INTRINSICS re-scoped** (open, deferred
to the real-toolchain gate) per the entries above. The OBL-SLICE-REGION ledger
keeps the retired prototype spelling `slice[r] T` / `slice_mut[r] T` as a
historical record, not a live spelling.

## OBL-TEXT-RESULT (found by text implementation, 2026-07-09)

Design 0013's str_from must return a Result carrying a str (a borrow), but §3.4/E0201 bans a
borrow in an enum payload. The prototype resolved it with a compiler-known transient
Utf8Res { ok Valid(str), Invalid(usize) } destructured immediately. A genuine tension: either
§3.4 gains a borrow-in-payload-if-immediately-consumed relaxation (analysis-heavy), or fallible
validation-returning-a-view is always compiler-known (the current honest state), or str_from
returns the offset and the caller re-forms the view. Design decision deferred; the prototype's
transient-type approach is sound meanwhile. Gate: a general library str_from.

## OBL-SELFHOST-ERGO (self-hosting friction, 2026-07-09)

Writing a real Candor program (the self-hosted lexer) surfaced six ergonomic frictions, ranked
by how often they bit: (1) a [u8]/str view cannot be a struct field without a region variable
(0001 §3.4), so source is threaded through every helper instead of held in a Lexer struct - the
biggest structural tax; (2) no growable Vec forces fixed arrays + count with a hard cap; (3) no
owned String without an Alloc handle makes text output a manual byte-by-byte itoa; (4) no
match-on-bytes/no map makes keyword classification a linear span_eq ladder (55 branches); (5) out
is a hard keyword the self-lexer must dodge as an identifier; (6) write-path buf.*.f[i]= vs
read-path buf.f deref asymmetry is error-prone. Items (1)/(3) point at the region-bearing-fields
and text-in-core questions; (2) at a std Vec; (4) at a std map + the deferred Chars/byte-match.
This is the P19/dogfooding signal the self-hosting arc exists to produce.
### Slice 2 addendum — the parser (recursive descent in Candor, 2026-07-10)

Writing the self-hosted PARSER surfaced seven MORE frictions, distinct from the lexer's (a
recursive tree-builder stresses different edges than an iterative scanner), ranked by bite: (1)
no sum type / no Box-recursive AST — a self-referential tree must be a fixed `[N]Node` ARENA
addressed by `u32`, with every child edge a `u32` slot and every list an intrusive `nx` sibling
chain (the arena-basket idiom); this is the dominant structural tax and caps tree size. (2) ONE
return value per fn — `decl_brackets` (regions+tparams) and mode/type parsing (mode+region+type)
must smuggle their multiple results through SCRATCH FIELDS on the parser struct, an aliasing-prone
side channel `out`-params would fix. (3) `match` has no integer/literal patterns (only
wildcard/binding/variant), so the AST-tag DISPATCH in the dumper is a ~100-branch `if nd.tag == T_X`
else-if ladder instead of a `match` — the same wart the lexer's keyword ladder hit, now on tags.
(4) DEEP recursion overflows the default (~2 MiB) test-thread stack: the precedence ladder is ~13
fn-frames deep per primary and each interpreter frame is heavy, so the gate must run on a 256 MiB
thread (the iterative lexer never hit this — a recursive-descent program pays a runtime-stack tax
the tree-walker amplifies). (5) hard keywords cannot be locals — `drop` as a variable name is a
parse error (renamed to `drophook`); the keyword set leaks into the self-author's identifier space,
compounding the slice-1 `out` friction. (6) no generics-over-borrow and no fn-pointers taking a
`write` param — the Rust oracle's one `binary_left(next_fn, ops)` helper cannot be written, so all
~10 precedence levels are hand-expanded into near-identical functions. (7) `p.*.f = g(p, ...)`
(store into a field of the same `write`-borrowed struct from a call that also borrows it) is a
two-phase-borrow hazard, forcing a `let tmp = g(p, ..); p.*.f = tmp;` temp at EVERY arena-alloc
site — ubiquitous ceremony. Items (1)/(2) point at a language sum-type/AST story and `out`-value
ergonomics; (3) at literal match patterns; (4) at guaranteed tail/heap recursion or a bigger
default stack; (6) at higher-order over borrows. What WORKED cleanly: reusing the lexer's helpers
by CONCATENATION (span_eq, emit_*), threading `(write P, [u8], read Buf)` through mutual recursion
via automatic write-param reborrow, and a SPAN-FREE canonical S-expression as the differential
target (excluding spans sidesteps span-arithmetic divergence and keeps the gate on tree SHAPE).

## OBL-SELFHOST-ERGO friction #1 (source-threading) — RATIFIED as accepted cost, 2026-07-10

Ruled by high-effort deliberation (docs/design/PROPOSAL-selfhost-ergonomics.md, Candidate C
disposition): 0001 §3.4's no-borrow-fields rule stands; threading ambient read-only views through
pass call-trees is the accepted essential cost of the value-first model (the region-parameterized
alternative is the complexity Bet 5 traded away, and reopening it re-blinds the frozen Bet 5 valve
accounting). Re-open trigger: a slice whose pass-context tuple grows to 4-5 threaded views on the
majority of signatures. OBL-ITER-BORROW and OBL-TEXT-CHARS re-routed to region-free discharge
paths; OBL-TEXT-RESULT left open as a separate smaller question.

### Slice 3 addendum — the type checker (name resolution + type-error core in Candor, 2026-07-10)

Writing the self-hosted CHECKER (`selfhost/checker/checker.cnr`, composed after
lexer + parser; gated by DIAGNOSTIC equality vs the Rust oracle over a corpus) surfaced three
findings, the first of which is the load-bearing datum the Candidate-C ruling named as its
re-open trigger.

(1) **CONTEXT-TUPLE SIZE — the C-ruling re-open trigger is NOT tripped: the checker reaches
THREE threaded views, of which only TWO are read-views.** Every pass/walker signature threads
exactly `(read P arena, [u8] src, write C state)` — 9 of the file's functions, i.e. the majority.
Two are read-only VIEWS (the parser's node arena `P`, and the source bytes `src` that §3.4 forbids
as a struct field — friction #1); the third is a single MUTABLE aggregate `C`. Crucially the count
stays at three ONLY because ALL mutable checker state — the locals/scope stack, the diagnostic
buffer, AND the item-table root — is folded into the ONE struct `C` (owned fixed arrays, which
structs MAY hold). Had those been threaded as separate params (locals, diags, item-table), the
tuple would be FIVE and the trigger would fire. So the essential read-view floor a value-first
checker pays is 2 (arena + source), not 4-5; state aggregation absorbs the rest. The "item table"
needed no new structure at all: struct/enum/fn/static decls are resolved by walking the parser's
item `nx`-chain (O(n), n tiny) from a root index stored in `C`. VERDICT for the C decision: the
hardest slice so far threads 2 read-views on the majority of signatures — under the 4-5 trigger.
The ratified source-threading cost holds; no re-open warranted by this slice.

> **Perf addendum (2026-07-10):** the checker's name resolution is now **Map-backed** (std hash `Map`, O(1) `contains`). One setup pass (`build_symbols`) registers imported names + declared struct/enum names into a `known_types` Map and imported names + declared fn/static names into a `known_values` Map; `is_type_known`/`is_value_known` answer by `contains` instead of the former per-occurrence O(names×items) `nx`-chain + import-chain scans. SET semantics answer identically to the scan (duplicate names are idempotent), so diagnostics are byte-exact; the `C` struct gained the two `Map` fields and `check_dump` an `Alloc` handle (a bump prelude, dropped with `C`). Output-invariant; parser.cnr self-check gate 47s→36s debug. Locals stay a linear stack (few, scoped); the analyses slice's own scans are the next lever.

(2) **The span-lean slice-2 arena confines the checker to NAME-TOKEN-span diagnostics.** Slice 2
excluded spans from its canonical AST (differential target = tree shape), so the arena `Node`
stores only NAME-token spans (`p0/p1`) for identifiers/types/fields and NO span at all for
integer/bool literals. But the oracle's diagnostics for the VALUE-type-level codes carry COMPOSITE
spans = `span_from(lo)` = (leftmost-token-start … rightmost-token-end) of the whole enclosing
expr/decl (E0703 mismatch, E0706 arg-count, E0107 field, E0108/E0605 enum, E0709 literal-range).
The arena cannot reproduce these without re-deriving a per-node `(start,end)` by walking to
leftmost/rightmost tokens — and even that fails for E0709 because literals store no offset. So
diagnostic-equality is provable ONLY for the codes whose oracle span coincides with a single
stored name token: **E0102** (unknown type) and **E0103** (unknown name), both matched exactly
(codes + spans) on 7/7 corpus fixtures (3 positive/clean, 4 negative), 9 diagnostics. This is the
honest boundary: a sound name-resolution + item-table core that matches the oracle, not broad
value-type coverage that would drift on spans. GATE to unlock the rest: give the arena a per-node
`(start,end)` span pair (2 usizes/node — cheap, but reverses slice 2's span-free decision) OR store
literal spans; either turns the composite-span code families (E0703/E0706/E0107/E0108/E0605/E0709)
into gate-checkable targets.

(3) **The `write`-param double-reborrow trap (49 errors at once).** Passing a `write C` PARAMETER
onward as `write c` does NOT auto-reborrow — it write-borrows the borrow, yielding
`borrow_mut borrow_mut C` and an E0703 at EVERY call site. A `write` param must be passed BARE
(`c`) to auto-reborrow; `write c` is correct ONLY for an owned local (the entry fn). This is the
write-path dual of slice 2's `p.*.f = g(p,…)` two-phase hazard and §11.4's `read (deref c)` rule:
the surface gives no cue that bare-vs-`write` flips meaning between owned locals and write params,
and the failure is a wall of identical type-mismatch errors, not a targeted one. A reborrow lint,
or an explicit reborrow operator, would remove the trap. What WORKED cleanly: reusing the lexer's
`span_eq`/`emit_*`/`trace` sink and the parser's `P`/`Node`/`T_*` arena by CONCATENATION; a
canonical `CODE start end` dump sorted by (code,start,end) so emission order need not match the
oracle's traversal; and filtering the oracle to the covered code families so the differential
harness self-reports coverage honestly (positive fixtures = empty covered-set, negatives = the
specific codes).

### Slice 4 addendum — the move/init core of the borrow checker (in Candor, 2026-07-10)

Writing the self-hosted MOVE/INIT analysis (`selfhost/analyses/analyses.cnr`,
composed after lexer + parser; gated by DIAGNOSTIC equality vs the Rust oracle's `init.rs`
over `dataflow.rs`) surfaced four findings. This is the hardest slice — the borrow checker is
state-heavy (per-place move state + init state) — so the CONTEXT-TUPLE datum was the load-bearing
question.

(1) **CONTEXT-TUPLE SIZE — the C-ruling 4-5 re-open trigger is NOT tripped even for the
state-heaviest analysis: THREE threaded views, exactly like slice 3.** Every walk/dataflow
signature threads `(read P arena, [u8] src, write A state)`. The two read-views are the same
essential floor (the arena, and the §3.4-forbidden `src`); the third is one MUTABLE aggregate `A`
that folds ALL of the analysis's state — the per-place flow lattice (`st[]` array), the locals
table (name spans + copy flags), the item-table root, AND the growable diagnostic buffer. The
BORROW CHECKER'S STATE-HEAVINESS DOES NOT FORCE 4-5 VIEWS: per-place state aggregates into one
struct exactly as slice 3's `C` did. The value-snapshot the branch JOIN needs (clone the flow
state at each if/match, meet, restore) is done by copying the `st[]` FIELD out of `A` as a plain
`[256]i32` value — no extra threaded view, no separate state param. So the ratified source-threading
cost holds; the hardest slice confirms the 2-read-view floor rather than breaching it. NOTE: had the
diagnostic Vec forced a threaded `read Alloc` (see #3), that would have added a 4th view — but Vec's
allocator is needed only at CONSTRUCTION (in the entry fn), never in the walk, so the walk stays at 3.

(2) **The span-lean arena confines move/init to BARE-IDENTIFIER-use spans: E0301 + E0304 matched;
E0302 + E0309 out of subset (same boundary as slice 3's E0703).** The oracle's use-of-moved
(E0301) and read-before-init (E0304) diagnostics attach to the USE SITE; for a bare identifier that
is the `T_ID` node's stored `p0/p1`, matched EXACTLY (codes + spans) on 11/11 corpus fixtures
(3 positive/clean, 8 negative), 10 covered diagnostics. But (a) a FIELD/index use (`x.a` of a moved
`x`) carries the whole-expr composite span (96..99) the arena does not store — only the root `T_ID`
(96..97) — so field-use diagnostics are excluded and fixtures use bare-id uses; (b) the move-JOIN
family is synthetic-span: E0302 (move-join disagreement) spans the block `join_span`, E0309
(needs-drop maybe-init at a drop point) spans the scope-exit/reassign statement — neither carried by
the span-lean arena. The dataflow JOIN is STILL COMPUTED (Init∪Moved→Moved, Init∪Uninit→MaybeInit),
so E0301/E0304 stay correct across if/match branches and a conditional move followed by a use is
caught; only the join-anchored diagnostics are not EMITTED. Same GATE to unlock as slice 3: a
per-node `(start,end)` span pair reverses the slice-2 span-free decision and turns E0302/E0309 (and
field-use E0301/E0304) into gate-checkable targets.

(3) **Vec's first self-hosting customer: it fits the append-only diagnostic buffer but is the WRONG
tool for the value-snapshotted flow state, and its allocator-explicitness (P9) is a viral tax.**
Measured, not guessed. Vec (`vec_new`/`push`/`get`/`set`/`len`) works and grows past initial
capacity without re-passing the allocator — so it retired the diagnostic buffer's fixed 512-cap.
But three frictions: (a) `vec_new(a: read Alloc)` needs an `Alloc` handle, which in a standalone
program means a ~20-line bump-allocator prelude over a reserved flat-memory window with two `unsafe`
valves — the analysis carries its own allocator; (b) `push` is `alloc`-effecting, so EVERY function
in the walk that can emit a diagnostic becomes virally `alloc`-marked (~14 fns here) and `main`
becomes `alloc`; (c) `get` returns a `borrow T`, so a scalar read is `get(v,i).*` and arithmetic on
two `get`s is an E0703 borrow-arithmetic error — an insertion sort over the diag Vec is noticeably
more ceremony than over an array. Crucially, the per-place FLOW STATE stayed a FIXED `[256]i32`
array precisely because a Vec CANNOT be cheaply cloned at every branch join (the borrow checker's own
state wants cheap value-copy semantics, which arrays give and an owning, move-only, alloc-marked Vec
does not — a nice irony for the tool that enforces those semantics). VERDICT: Vec earns its place for
unbounded append-only OUTPUT; fixed arrays remain correct for snapshotted dataflow state; the P9
allocator-explicitness is the price and it is viral through the effect system.

(4) **Loop-carried move hazards + owned-scrutinee match-moves are the deferred boundary (slice 5 /
loans).** The structured walk mirrors the oracle's forward dataflow for straight-line code and
if/match joins, but handles `while`/`loop` with a SINGLE body-pass + a conservative post-loop join
(pre-loop ∪ body-out) instead of the oracle's RPO FIXPOINT. This matches the oracle on loops with no
loop-carried move (positive loops, loops whose body only reads initialized values) but MISSES the
second E0301 the fixpoint reports when a non-copy move inside a body is re-executed next iteration
(measured: oracle 2 diags, ours 1). Match scrutinees are treated as non-consuming inspections (fine
for copy scrutinees; an owned non-copy scrutinee whose arm bindings MOVE it is deferred). The LOAN
family (E0801-E0809: XOR conflict, move/write-while-borrowed, backward loan-liveness) is deferred
whole to slice 5 — correctly matching a small move/init core beats drifting on loans, exactly the
compiler-stage discipline. What WORKED cleanly: reusing the lexer/parser `emit_*`/`span_eq`/`Node`
arena by concatenation; folding all state into one `A` so branch snapshots are a single array copy;
and reading the `copy` property directly off the parser's `T_STRUCT/T_ENUM` `op` flag + type nodes
(the `is_copy` recursion) so move-vs-copy classification needs no separate type-resolution pass.

### Slice 5 addendum — composite-span enrichment (step 3 of self-checking self-hosting, 2026-07-10)

Revisiting the "span-lean arena" boundary (slice-4 addendum #2, slice-5 loans boundary) with the
goal of retaining the sub-spans the deferred composite-span families need. KEY CORRECTION to the
earlier GATE framing: the `Node` already carries a general `(p0, p1)` usize span pair per node — the
STRUCTURAL nodes (`T_ASSIGN`, `T_IF`, `T_BLOCK`, `T_LOOP`, `T_WHILE`) simply left it ZERO, and the
S-expr dump renders `p0/p1` only for name/literal content, never for these nodes. So enriching a
structural node's span is a POPULATE, not a widen: no field added, the `[2048]Node` cap and `u32`
edges are untouched, and the parser AST-S-expr and token dumps stay byte-exact (verified:
selfhost_parser green).

Family-by-family, the honest split (the blocker for the still-deferred three is ANALYSIS capability,
not span retention):

* **E0803 (write-while-borrowed) — DISCHARGED, oracle-matched.** Oracle span = the whole assignment
  STATEMENT = `span_from(lo)` = (leftmost-token-start .. semicolon-end); the parser now populates
  exactly this into `T_ASSIGN.p0/p1`. The existing loan machinery already tracks the in-scope loan
  set, so `chk_stmt`'s assignment arm adds a WRITE loan-check (`acc == 2`) on the LHS root that emits
  E0803 against a live loan of EITHER kind (unlike E0804, which needs an exclusive loan). Matched
  byte-exact (code + span) vs the oracle on 2 new fixtures (`neg_write_excl`, `neg_write_shared`),
  1 diagnostic each; the selfhost_loans COVERED set now includes E0803.

* **E0809 (write-through-shared) — span ENABLED, analysis DEFERRED.** Its oracle span is the SAME
  whole-statement span, now carried on `T_ASSIGN.p0/p1` for free. But emitting it requires deciding
  whether the LHS deref-path peels a SHARED (`read`) borrow — per-binding borrow-TYPE tracking this
  slice's root-granularity loan model does not perform. Span retained (capability exists); emission
  deferred until the loan model tracks each binding's borrow kind/type.

* **E0302 (move-join disagreement) — DEFERRED (analysis).** Oracle span = the enclosing `if`/`match`/
  `loop` construct's `join_span` = the whole-construct span, populatable into `T_IF/T_LOOP/T_WHILE`
  `p0/p1` (T_MATCH already carries it) via the same existing pair. NOT populated here: the blocker is
  emission, not span — the analysis computes the dataflow JOIN but does not compare per-place
  branch-end states and emit the disagreement diagnostic. Populating the span with no consumer would
  be speculative dead code, so it is left for the slice that adds join-disagreement emission.

* **E0309 (needs-drop maybe-init at a drop point) — DEFERRED (analysis).** Oracle span = the
  enclosing block span (`b.span`), populatable into `T_BLOCK.p0/p1`. Blocker is analysis, not span:
  the slice tracks no needs-drop/`drop`-hook type property and models no scope-exit drop points, so
  the MaybeInit-at-drop condition is never computed. Deferred until the analysis gains drop-point
  modeling.

### Fixpoint gate — the checker checks the LEXER's own source clean (first self-check sub-goal, 2026-07-10)

First end-to-end turn of the self-check loop on real self-host source: the self-hosted checker
(`selfhost/checker/checker.cnr`, E0102 unknown-type / E0103 unknown-value) run OVER
`selfhost/lexer/lexer.cnr` itself emits an EMPTY covered-diagnostic set — byte-equal to the Rust
oracle, which also emits nothing on this valid source. `lexer.cnr` is the right first target: a LEAF
module (no `use`, no `Vec`), so every name it uses is locally declared and no builtin is missing —
the checker resolves it CLEAN on the first try, with no false positive to fix. Gate added to
`compiler/tests/selfhost_checker.rs` (`candor_checker_checks_lexer_source_clean_fixpoint`), reusing
the slice-3 module-tree harness; a negative smoke (a param of an unknown type appended to the source)
is asserted to be flagged E0102, so the clean assertion is non-vacuous.

The one real blocker was ARENA CAPACITY: the fixed self-host arenas were sized for tiny fixtures, not
a 434-line file. Measured need for `lexer.cnr`: 3690 tokens, 2389 parser nodes. Both `Buf.toks`
(`lexer.cnr`) and `P.nodes` (`parser.cnr`) grown 1024/2048 -> **4096** (next power-of-two above the
3690-token max; node arena gets ample margin). The arena model is UNCHANGED ([N]Node, u32 edges,
region-free view threading). Every coupled site moved together to keep the type-level array sizes
consistent: the `Buf.toks`/`P.nodes` struct fields, the three `[nnew(0); 4096]` P-literals
(parser/checker/analyses `*.cnr`), and the six harness-generated `main.cnr` Buf-literals
(selfhost_{lexer,parser,checker,analyses,effects,loans}.rs). All prior selfhost oracle gates stay
byte-exact green under the larger arenas. Gate runtime ~6.5s (two tree-walk lex+parse+check passes
over the 434-line file); the Map/symbol-table perf enabler remains the known lever if this grows.

## OBL-ITER-BORROW — DISCHARGED via the region-free path, 2026-07-10

The candidate-C ruling re-routed this to its region-free branch; that branch WORKED, vindicating
the ruling's bet that borrowed iteration needs no region-parameterized types. Shipped as a THIRD
iteration protocol RefIndexed { type Item; count(read self); get_ref(read self, i) -> read Item }:
region-free because get_ref has a single borrow-in (read self) and single borrow-out, so 0001
§3.3's compact default fixes provenance to self with no region variable and nothing stored. The
literal next_ref(write self, coll) -> Opt[read Item] shape was shown UNVIABLE (two borrow-params
force mandatory regions; a borrow in Opt's payload is E0201) - dissolved by folding the cursor
into the loop and splitting count() out so get_ref returns a bare read Item. P3: three protocols
selected by two visible syntactic axes (operand mode + binding mode), forced not sprawl. Loan
lifetime: the loop's read loan spans the body, so mutation-during-iteration is E0801 by existing
machinery. for write x (mutating yield) extends the same compact-default argument and is future
work, still region-free. Prototype: RefIndexed wired for Vec (tree-walker only); a user impl now typechecks AND
runs end-to-end on all five engines via the compact default — see the 2026-07-14 shared-branch entry
(which also fixes the method-return loan-provenance gap design 0015 §5's escape argument rested on).

## OBL-TEXT-CHARS — DISCHARGED via the value-gear path, 2026-07-10

The second candidate-C consequence, also discharged region-free-and-C-free (the ruling's bet pays
off a SECOND time). char_at(s: str, pos: usize) -> CharStep{cp: u32, next: usize} is a value-gear
decoder: a compiler-known copy struct of owned values, no iterator struct, no borrow field, no
alloc marker - the position threads through step.next exactly as the self-hosted lexer threads its
scan cursor. char_count(s) -> usize (O(n)) ships with it, discharging 0013's deferred op. No
for-sugar (P6: the primitive + while is the honest minimal surface; a Chars value earns no slot).
Ill-formed UTF-8 or pos==len faults (Bounds family) - a valid str never hits it (0013 §4
guarantee); a str_from_unchecked forgery yields wrong values, never UB. Tree-walk oracle only
(str ops are oracle-only per the text stage). Both features candidate C would have unlocked
(borrowed iteration, char iteration) are now proven buildable WITHOUT region-parameterized types -
the ruling vindicated constructively, twice.


## OBL-SELFHOST-MOD — self-host module-tree split (step 2 of self-checking self-hosting, 2026-07-10)

Modularizing the self-host compiler — the one substantial Candor program that did not exercise
modules — into a real `use`/`pub` tree loaded by the stage-1 module system (design 0008) DOGFOODED
that system and surfaced its ergonomic edges. The oracle gates stayed byte-exact (573 green): a
pure source-organization refactor. COMPLETE (2026-07-10): ALL SIX slices — lexer / parser / checker
AND analyses / loans / effects — now load as a module tree (each harness materializes the flat
`lexer.cnr` + `parser.cnr` + the slice's `.cnr` + a generated root `main.cnr` and runs it via
`run_dir`). The blocker below (OBL-SELFHOST-MOD-F1) is DISCHARGED, so analyses/loans/effects — which
define their own `struct Alloc` and use the Vec-backed diagnostic buffer — now load modular too;
`analyses.cnr` imports the lexer (5 names) and parser (66 names, incl. `Node`) by NAMED `use` and
exports `pub fn analyze_dump`. Full suite 576 green (573 + the 3 new checker tests below).

What WORKED — the shared-type graph crosses module boundaries cleanly via NAMED imports:
`pub struct Tok`/`Buf` (lexer) and `pub struct Node`/`P` (parser) are `use`d and used as TYPES in
type position (params, fields, array elems, borrows) — the loader's rewriter qualifies `Named`/
`App`/`Proj`/`Array`/`Borrow` type nodes; `pub fn` cross-module calls resolve as values; the
`pub static` T_* / PF_* AST-tag vocabulary crosses as value idents. The merge (every non-root item
mangled to `module::name`) is output-invariant, and name clashes that the flat concatenation had to
avoid (checker's and analyses' duplicate `name_eq`/`loc_push`/…) become NON-issues under mangling.

Frictions found (module ergonomics):

(1) **No glob import (`use m::*`) and no `pub use` re-export (design 0008 §6, stage-1 deferred) force
every cross-module name to be enumerated by hand.** checker's `use parser::{…}` lists 65 names and
analyses' 66 — almost entirely the ~60-entry `T_*` tag vocabulary the consumers read off the arena.
A facade/prelude module that re-exports the tag set is impossible without `pub use`; a glob would
collapse each list to one line. This is the dominant verbosity tax of the split.

(2) **file=module / directory=namespace maps `selfhost/lexer/lexer.cnr` to the module `lexer::lexer`,
not `lexer`.** Clean single-segment module names (`lexer`, `parser`) require FLAT files in one
directory, and the `foo.cnr`-beside-`foo/`-directory body merge is stage-1-deferred, so the harness
copies the sources FLAT into a per-fixture temp dir (with the generated `main.cnr`) rather than
pointing the loader at the existing nested `selfhost/*/` layout.

(3) **The entry convention (`fn main` must live in the root `main.cnr`) means the modular self-host
source is not a standalone checkable tree** — it is a LIBRARY tree with no root, so it is loadable
only with a driver `main.cnr` supplied (here, per-fixture, embedding the corpus source as a `[N]u8`
literal). `check_dir` on the library alone reports E0905.

### OBL-SELFHOST-MOD-F1 — the checker's builtin-`Alloc` name test breaks under module qualification (DISCHARGED 2026-07-10)

`src/check/expr.rs` types the allocator-explicit builtins `vec_new` / `string_new` / `map_new` by
requiring their argument to be `Type::Named(n) if n == "Alloc"` — a LITERAL name test. The module
loader qualifies a user-defined `struct Alloc` to `analyses::Alloc`, so the checker rejects it with
E0703 ("`vec_new` expects `read Alloc`, found `borrow analyses::Alloc`") — five times for the
analyses slice's Vec-backed diagnostic buffer — BEFORE the interpreter runs. The interpreter side
(`src/interp/eval.rs`) ALREADY identifies the `Alloc`/`AllocVtable` structs STRUCTURALLY (its own
"finding F1", added precisely so box/unbox survive module-qualified names), but that fix was never
applied to the checker. So this is NOT a module-system type-graph limit — the shared-type graph
crosses fine — but a compiler incompleteness: finding-F1 was discharged in the interpreter and left
open in the checker. Consequence: any module tree that both defines its own `struct Alloc` and calls
`vec_new`/`string_new`/`map_new` cannot load, which blocks the analyses (move/init + loans) and
effects slices (all three share the Vec-based diagnostic buffer). RESOLUTION (2026-07-10). `src/check/expr.rs` now identifies the allocator handle STRUCTURALLY,
mirroring `eval.rs`'s finding-F1 detection, via one helper `is_alloc_handle(&self, ty: &Type) -> bool`
(with `is_alloc_vtable`) called from all three arms (`vec_new`/`string_new`/`map_new`) in place of the
`n == "Alloc"` name test: the handle is a struct whose `vt` field is a `rawptr` to the vtable struct,
and the vtable is the struct with fn-ptr fields `alloc` and `free` — looked up through the checker's
own `self.items.lookup_struct`. The check stays borrow-peeled (the arg is `read Alloc`). The bare
single-file `struct Alloc` satisfies the same predicate, so single-file and module-qualified paths
pass through ONE test — no name special-case. A focused checker test proves it (tests/check.rs):
`vec_new`/`map_new` on a RENAMED handle (`struct Handle`, `vt: rawptr Vt`) type-check with no E0703,
and a non-handle struct passed to `vec_new` still gets E0703. With the checker fixed, the
analyses/loans/effects harnesses were converted from `include_str!` concatenation to the module-tree
loader (`run_module_tree`), completing the split — all six slices are now modular.

### Import resolution — the self-host CHECKER checks a NON-LEAF module clean in isolation (2026-07-10)

Step 3-continued of self-checking self-hosting: `use`/`pub` import resolution added to the
self-host front-end so a module can be checked IN ISOLATION with its imported names resolved,
instead of concatenating sources. Payoff gate added to `compiler/tests/selfhost_checker.rs`
(`candor_checker_checks_checker_source_clean_via_import_resolution`): the self-host checker checks
`selfhost/checker/checker.cnr` — a NON-LEAF module that imports names from BOTH the `lexer` and
`parser` modules (`use lexer::{Buf, span_eq, ...}; use parser::{Node, P, T_FN, ...}`) — and emits
an EMPTY E0102/E0103 set, byte-equal to the MODULE-AWARE reference oracle (`check_dir` over the
lexer+parser+checker tree). Contrast the leaf-module `lexer.cnr` fixpoint gate: that module has no
imports, so it checks clean trivially; checker.cnr resolves ~70 imported names to check clean.

ORACLE-TOKEN-PARITY RESOLUTION (the subtlety resolved before coding): the Rust reference lexer
(`src/real/token.rs::real_keyword_from_str`) does NOT list `use`/`pub` — they lex as IDENTIFIERS,
and the reference PARSER recognizes them CONTEXTUALLY at item-leading position (`at_ident("pub")`/
`at_ident("use")`, collecting `use` decls into the `mod_uses` side channel, dropping `pub` into a
discarded `_vis` vector). So the self-host lexer needed NO change — it already leaves `use`/`pub` as
IDENT (kind 1), matching the reference byte-exact; inventing keyword tokens would have BROKEN token
parity. Resolution lives at the PARSER level from identifier tokens (via the existing `ids` helper).

PARSER: `use path::{a, b};` / `use path::name;` parses into a `T_USE` node whose `a` edge heads a
T_NAME chain of the imported binding names; `pub` is consumed as a visibility marker and dropped.
The `use` nodes are collected into a SEPARATE chain stored in a new `P.uses` field (mirroring the
reference's `mod_uses` side channel), NOT the item chain the AST S-expr dump walks — so the parser
dump stays byte-exact against the oracle, which likewise excludes `use` decls from `Program.items`
(the oracle's `_uses` is discarded by the S-expr renderer). No reference-dump change was needed.

CHECKER: after `parse_program`, `P.uses` is read into `C.uhead`; `is_type_known` and
`is_value_known` each scan the imported-name chain (`is_imported`), registering every imported name
as BOTH a known type and a known value (sound for the clean-source no-false-positive goal; the
span-lean arena does not record which module a name is a type-vs-value in). `pub fn`/`pub struct`
still register their own names normally (visibility is cross-module only).

BLOCKER — parser.cnr is NOT isolation-checkable via this harness (harness embedding ceiling, NOT a
front-end limit and NOT related to import resolution): the module-tree harness embeds the checked
source as a `[N]u8` array literal in a generated `main.cnr`. checker.cnr (19.5 KB -> ~107 KB main)
embeds and lexes byte-exact; parser.cnr (77.7 KB -> ~465 KB main) is CORRUPTED by the real
front-end when it parses that giant literal — the interpreter sees len==77660 but the embedded bytes
diverge from the file at byte 1748 (self-host lexer yields 11378 tokens vs the oracle's 19703), so
its self-host parse derails. checker.cnr is the correct isolation target regardless: it is a genuine
non-leaf module importing from two modules and exercises richer import resolution than parser.cnr
(which imports only from lexer). Reaching parser.cnr-scale isolation needs a harness that feeds
source without a giant array literal (out of scope here).

ARENA CAPACITY: `checker.cnr` measures 4011 tokens / 2432 self-host arena nodes; `lexer.cnr` 3690 /
2389. Both `Buf.toks` (lexer.cnr) and `P.nodes` (parser.cnr) grown 4096 -> **8192** (comfortable ~2x
token / ~3.4x node headroom over the ~4k-token self-host files that are checked in isolation),
sized to the measured target rather than the unreachable parser.cnr. Every coupled site moved
together: the `Buf.toks`/`P.nodes` struct fields, the three `[nnew(0); 8192]` P-literals
(parser/checker/analyses `*.cnr`), and the six harness-generated `main.cnr` Buf-literals
(selfhost_{lexer,parser,checker,analyses,effects,loans}.rs). A new `P.uses: u32` field threads the
use-chain head; all three P-literals updated. The arena model is UNCHANGED ([N]Node, u32 edges,
region-free view threading). All prior selfhost oracle gates stay byte-exact green under the larger
arenas + the new field. Isolation-gate runtime ~7.7s (lex+parse+check of the 4011-token file, twice,
with the checker's linear name scans); the Map/symbol-table adoption remains the planned perf lever.

### Fixpoint gate — the self-hosted ANALYSES core accepts the self-host source (step 3 deepened, 2026-07-10)

The deepest self-check turn so far: the self-hosted borrow-checker ANALYSES core
(`selfhost/analyses/analyses.cnr`, `analyze_dump` — move/init E0301/E0304, loans
E0801-E0804, effect E0401, exhaustiveness E0601) run OVER real self-host source
(`lexer.cnr` and `checker.cnr`) now emits an EMPTY covered-diagnostic set, byte-equal
to the module-aware Rust oracle (`check_dir` over the real lexer+parser+analyses[+checker]
tree, which runs the full Rust check pipeline and is likewise ∅). Two gates added to
`compiler/tests/selfhost_analyses.rs` (`candor_analyses_check_lexer_source_clean_fixpoint`,
`candor_analyses_check_checker_source_clean_fixpoint`), each with a use-after-move injected
into a copy of the checked source asserted to fire E0301 — so the clean set is non-vacuous.
Runtime ~7.5s for both (two analyze passes over ~3.7k/4.0k-token files). checker.cnr is the
right deep target: a non-leaf module, but the analysis needs no import resolution (it runs
dataflow over the parsed AST and treats unresolved names as non-locals). Full suite 580 green.

FALSE POSITIVE FOUND AND FIXED (the real find — the scoping prediction did NOT hold ∅ on the
first run). The unfixed analysis emitted E0301 (use-after-move) 1× on lexer.cnr and 22× on
checker.cnr, all on `write`-borrow PARAMETERS (`c: write C`, `buf: write Buf`) passed onward
bare to a call or field-read after such a pass. Root cause (`analyses.cnr` `param_copy` +
`place_use`): a `write` param was classified non-copy (`loc_copy == 0`), correct for the LOAN
analysis's exclusive-access sense but wrong for the MOVE analysis — `place_use` ctx-0 then marked
the param MOVED (`st = 2`), so the next use was use-after-move. A borrow reference cannot be moved
out of; passing a `write`/`read` param bare AUTO-REBORROWS. Minimal fix: a new per-local
`loc_ref` flag (set for read/write params and borrow-typed bindings via `is_ref_binding`); in
`place_use` the `st = 2` move is now guarded by `loc_ref == 0`, so only an OWNED non-copy value
moves, a reference reborrows. The `loan_check` access (acc) is unchanged, so no loan diagnostic
shifts. All negative fixtures stay byte-exact (selfhost_analyses/loans/effects green in isolation):
the loan fixtures move only OWNED `let mut x` locals, never a reference, so no true positive is
silenced.

### OBL-SELFHOST-ANALYSES-NAMERES (DISCHARGED 2026-07-10) — analyses.cnr is name-res-clean under the self-host CHECKER

Item scoped for this slice: extend the self-host CHECKER's E0102/E0103 clean gate to a FOURTH
module, `analyses.cnr`, if it embeds byte-exact. MEASURED: analyses.cnr embeds BYTE-EXACT (7666
self-host tokens == 7666 oracle tokens; 4662 parser nodes; both under the 8192 arenas — NOT the
77.7 KB parser.cnr embedding ceiling, which corrupted at token-count divergence). BUT the
self-host checker (`checker.cnr`) does NOT check it clean: 142 spurious E0102/E0103 where the
module-aware oracle emits ∅. Two causes, BOTH requiring an extension of the CHECKER slice (not the
analyses slice that is this arc's subject):
(1) CONFIRMED — `checker.cnr`'s `is_builtin` lacks the Vec/allocator vocabulary `get` / `set` /
`vec_new` / `map_new` that analyses.cnr uses but lexer/parser/checker.cnr themselves never do; those
callees flag E0103 from their first use (line 204).
(2) OBSERVED, not root-caused — imported/item names (`Node`, the `T_*` statics, local fns) resolve
correctly through a hard boundary at analyses.cnr byte 22600 (line 560, a `let rn: Node` in
`chk_expr`'s `T_OUT` arm) and fail after it. It is NOT a checker fixed-buffer capacity limit (growing
the `[512]` locals AND diagnostic arrays to `[8192]` left the count at exactly 142) and NOT the
embedding ceiling; it points at a self-host parser/checker interaction on an analyses.cnr-specific
construct that the small parser fixtures do not cover, needing deeper (interpreter-level) debugging.
DEFERRED: the analyses.cnr name-res gate is not added; discharging it means teaching checker.cnr the
Vec builtin set and resolving cause (2). This is a capability the checker slice deliberately omitted
(it needed only the builtins lexer/parser use), so per the slice discipline it is logged, not hacked.
Gate to unlock: extend `checker.cnr`'s builtin table + isolate the line-560 resolution boundary.

RESOLUTION (2026-07-10). analyses.cnr now checks CLEAN (∅ E0102/E0103) under the self-host checker,
byte-equal to the module-aware oracle. Gate added:
`compiler/tests/selfhost_checker.rs::candor_checker_checks_analyses_source_clean_via_import_resolution`
(the FOURTH module under the name-res fixpoint, first to exercise the collection builtins). Teeth
held: the naive single-file check flags the unresolved imports E0102 (>0), and an injected
unknown-typed param flags E0102 — so the clean set is non-vacuous. Two causes, as predicted, but
cause (2) was NOT a checker gap:

(1) `is_builtin` gap — DISCHARGED. Added exactly `get` / `set` / `vec_new` to `checker.cnr`'s
`is_builtin` (the three collection builtins analyses.cnr actually calls; `map_new` is unused here, so
NOT added — P6-minimal). All three are genuine builtins in the Rust reference `check_builtin`
(`src/check/expr.rs`), so the self-host checker's builtin notion still matches the language.

(2) The "byte-22600 boundary" was NOT a checker name-resolution gap — the checker's name resolution
was already CORRECT. ROOT-CAUSED to an INTERPRETER memory-model leak. `str_literal` (the evaluator
of every `b"..."` / `"..."` literal, `src/interp/eval.rs`) `static_alloc`s FRESH storage on EVERY
evaluation and never dedups or reclaims it; static storage grows monotonically from `STATIC_BASE`
(0x1000) toward `STACK_BASE` (0x100000), where `main`'s embedded `src: [u8]` source array is
allocated. Checking analyses.cnr evaluates the checker's byte-literal vocabulary (span_eq / is_builtin
/ is_type_known) so many times that `static_bump` crosses 0x100000 after ~18845 literal allocations
and `write_bytes` begins overwriting `src` — corrupting the very source the checker is name-resolving.
The "byte ~22600" boundary is exactly the traversal point at which static crossed the stack; the
values that leaked into `src` were literal bytes ('n'→'s'→'t'). lexer.cnr / checker.cnr check clean
only because they evaluate fewer literals and never cross the line. Confirmed by instrumenting
`str_literal` (18845 allocations at/above STACK_BASE) and by probing `src[9173]` reading 110→115→116
transiently deep in the traversal. FIX: content-addressed literal interning — a
`literal_cache: HashMap<Vec<u8>, u64>` on the interpreter maps identical literal bytes to one static
allocation. Literals are immutable read-only views, so deduping is sound; it bounds static growth to
the few KB of distinct literals, far below the stack. This is a latent memory-safety fix for ANY
literal-heavy program, not just the self-host checker. All 580 prior tests stay byte-exact green.

BONUS — analyses.cnr also passes ANALYSES-clean now. Gate added:
`compiler/tests/selfhost_analyses.rs::candor_analyses_check_analyses_source_clean_fixpoint`
(the analysis accepts its OWN source; injected use-after-move teeth fire E0301). It tripped TWO NEW
false-positive patterns (10× spurious E0301), both `ty_copy` mis-classifications fixed minimally:
(A) T_ARRAY copy-ness recursed on `nd.a` — but the parser stores `nd.a` = size expr, `nd.b` = element
type, so `[256]i32` (the `snap` match-state snapshot) was read as the copy-ness of the size literal
→ non-copy → a by-value assign was a false move. Fixed: recurse on `nd.b`.
(B) an imported struct type (`Node`, read from the arena as `let pm: Node` and passed by value to
`param_copy`) has no decl in the analysis's local item table, so `ty_copy`'s `find_decl` miss defaulted
it non-copy → the by-value pass was a false move. Fixed: an unresolvable/imported named type is treated
as copy (non-moving) — parity with the oracle, which resolves the import; LOCAL types still resolve via
`find_decl`, so move teeth (the MOVE_SMOKE injected use-after-move on a local struct) are unchanged.
Both fixes verified by full oracle parity across lexer.cnr / checker.cnr / analyses.cnr clean gates +
the negative corpus (no lost teeth). Full suite 582 green (580 + 2 new gates), clippy clean. No arena
change (all targets < 8192, unchanged). Isolation runtimes (release): selfhost_checker 2.9s,
selfhost_analyses 2.9s.

### CAPSTONE — parser.cnr under the self-check: all four self-host modules self-check clean (2026-07-10)

The last deferred self-host module, `parser.cnr`, is now under BOTH self-check gates
(name-res E0102/E0103 and analyses E0301/E0304/E0401/E0601/E0801-4), oracle-matched
with teeth. All four self-host modules — lexer.cnr, checker.cnr, analyses.cnr,
parser.cnr — self-check clean.

STEP 1 — THE "HARNESS EMBEDDING CEILING" WAS THE INTERPRETER LITERAL LEAK, NOW GONE.
The prior BLOCKER entry (parser.cnr "CORRUPTED by the real front-end", self-host lexer
yielding 11378 tokens vs the oracle's 19703, bytes diverging at byte 1748) was measured
BEFORE the content-addressed literal-interning fix (commit fde0a92). RE-MEASURED
post-fix: parser.cnr embeds BYTE-EXACT — the self-host lexer over the embedded 77.7 KB
`[N]u8` literal yields 19703 tokens == the oracle's 19703 (self-host parse: 11272 arena
nodes). The corruption was exactly the static-storage-crosses-STACK_BASE leak overwriting
`main`'s embedded `src[]`, same signature as the analyses.cnr byte-22600 boundary; bounding
static growth removed it. There was NO real front-end / array-literal / recursion ceiling.
The BLOCKER above is DISCHARGED.

STEP 2 — ARENAS 8192 -> 32768. parser.cnr's measured 19703 tokens exceed the 8192
`Buf.toks`/`P.nodes` arenas; grown to 32768 (next power of two above 19703, ~1.66x token /
~2.9x node headroom over the largest module). Every coupled site moved together: the two
struct fields (`Buf.toks` lexer.cnr, `P.nodes` parser.cnr), the three `[nnew(0); 32768]`
P-literals (parser/checker/analyses `*.cnr`), and the six harness Buf-literals
(selfhost_{lexer,parser,checker,analyses,effects,loans}.rs); the `P.uses` field retained.
Arena model unchanged ([N]Node, u32 edges, region-free threading). All prior oracle gates
stay byte-exact green under the larger arenas.

STEP 3 — GATES + ONE REAL FALSE POSITIVE FOUND ON THE LARGEST MODULE.
`candor_checker_checks_parser_source_clean_via_import_resolution` (selfhost_checker.rs):
parser.cnr name-resolves clean (∅ E0102/E0103), resolving its `use lexer::{Tok, Buf, ...}`
imports itself, byte-equal to the module-aware oracle; single-file + injected-unknown teeth.
This gate passed clean on the first run (no checker false positive).
`candor_analyses_check_parser_source_clean_fixpoint` (selfhost_analyses.rs): parser.cnr is
analyses-clean over ALL families (∅), oracle-matched; injected-use-after-move teeth fire E0301.
This gate FIRST tripped an UNBOUNDED-RECURSION false positive — a genuine latent bug in
`analyses.cnr` surfaced only by parser.cnr (the other three modules lack the construct):
`borrow_place` (analyses.cnr), on a field access whose base is NOT rooted at a local
(`f().field` — a projection off a call temporary, which parser.cnr uses), had `root_of`
return 0 and fell through to `chk_expr(operand)` on the SAME T_FIELD node, which re-dispatched
to `borrow_place` on that node — an infinite `chk_expr <-> borrow_place` cycle (confirmed by
frame-depth instrumentation: climbing unbounded past 60000 frames, overflowing even a 1 GiB
release stack). MINIMAL FIX: `borrow_place`'s not-rooted-at-a-local branch now DESCENDS the
place chain to its non-place base and walks THAT base expression (plus index sub-exprs),
instead of re-dispatching the whole place node. Semantics preserved for genuine non-place
operands (a bare call still walks once, as before); the loop is broken because a field-of-
non-place walks its base call, not itself. No teeth weakened, no negative fixture regressed —
all analyses/loans/effects gates byte-exact green (the negatives move only bare-id locals).

RUNTIMES (debug): selfhost_checker 46s (5 gates incl. parser), selfhost_analyses 38s (5 gates
incl. parser). parser.cnr's gates dominate (~2.6x checker.cnr; the tree-walk lex+parse+check
of the 19703-token file). Both run on the existing 256 MiB gate thread — no `#[ignore]`, no
release-only gate. The Map/symbol-table adoption remains the planned perf lever. Full suite
584 green (582 + 2 new gates), clippy clean. NOTE: the loans (E0801-4) and effects (E0401)
gates run the SAME `analyses.cnr`/`analyze_dump` over their own corpora (there are no separate
loans/effects `.cnr` modules), so the single `borrow_place` fix covers them too — all
selfhost_{lexer,parser,checker,analyses,loans,effects} harnesses stay byte-exact green.

### S1 — self-INTERPRETING Candor: a scalar tree-walker in Candor (first slice, 2026-07-10)

The first self-interpreting slice: `selfhost/interp/interp.cnr`, a tree-walking
SCALAR interpreter WRITTEN IN CANDOR, executes an in-subset Candor program directly over the
self-hosted parser's Node arena and reproduces the Rust reference interpreter
(`compiler/src/interp/`) BYTE-EXACT — same `main` return, same `trace` sequence, same fault
identity (kind + span). Gated by EXECUTION equality in `compiler/tests/selfhost_interp.rs`
against `run_source_real` over a 24-fixture in-subset corpus (15 returns, 9 faults).

SCOPE (STRICT MVP, P6). Integer (`i8..i64`/`u8..u64`/`usize`/`isize`) + `bool` scalars only;
annotated `let`/`let mut` + assignment; `if`/`else`; `while`; `loop`+`break`/`continue`;
`return`; `+ - * / %` with Overflow/DivByZero; comparisons; `&&`/`||` short-circuit; bitwise
`& | ^` + shifts `<< >>`; unary `-`/`!`; `trace`; `assert`; `panic`; direct non-generic free-fn
calls (by-value scalars) with recursion. OUT OF SUBSET (later slices S2+): structs/arrays/
pointers/Box/heap/match/enums/str/slices/generics/conv/contracts/`?`.

VALUE MODEL. A tagged scalar is one `i64` bit-slot plus a width code (1=i8..5=isize, 6=u8..
10=usize, 11=bool) — NO flat byte memory. Width propagates from `let` annotations, literal
suffixes, and fn return types the way the oracle threads `expected`. Locals are a flat stack
keyed by name span with a `cur_base` frame floor; recursion rides interp.cnr's OWN Candor call
stack (each `call_fn` frame's locals restore the caller). The running `cur_span` is tracked to
mirror eval.rs exactly, so `assert` reports the LAST leaf evaluated in its condition (not the
whole condition) and arithmetic faults carry the composite binary span.

i64/u64 OVERFLOW WITHOUT i128 (scoping blocker #3, DISCHARGED). The language has no `i128`, but
the oracle detects overflow by widening to `i128`. Since interp.cnr is NOT self-checked this
slice, it uses the FULL language: the raw op is computed inside a `wrapping { }` block (which
cannot fault), then overflow is decided WITHOUT a wider type — signed add/sub by sign logic
(operands same/diff sign vs result sign), signed mul by a division re-check (`wrapped / a != b`,
with the `MIN * -1` case special-cased), unsigned add by carry (`wrapped < a`), unsigned sub by
borrow (`a < b`), unsigned mul by `wrapped / a != b`; narrow types compute in the 64-bit base and
range-check against the type's min/max. u64<->i64 bit reinterpretation uses `wrapping { conv }`
(no transmute). Verified byte-exact against the oracle on i32/i8/u8 overflow, u64 add-carry,
u64 sub-borrow, and i64 mul-overflow fixtures.

FAULT-SPAN STAMP (reusable, DUMP-INVARIANT). `mk_bin`/literal/unary/panic nodes were span-lean
(`p0/p1` zeroed). The parser now stamps `T_BIN` (lhs-start..rhs-end, matching the Rust
`span_from(lo)`), `T_INT`/`T_NEGINT`/`T_BOOL`/`T_UNARY` (token spans), and `T_PANIC` (keyword..`)`).
The parser S-expr dump renders `p0/p1` only for name/string content, never for these operator/
literal tags, so all stamps are DUMP-INVARIANT — the lexer/parser/checker/analyses oracle gates
stay byte-exact. This also pays down the deferred composite-span diagnostics of earlier slices.

WHAT IS GATED / NOT. Gated: interp.cnr's EXECUTION behavior (ret/trace/fault) byte-equal to the
oracle. NOT gated this slice: interp.cnr under the self-CHECK gates (E0102/E0103 name-res and the
analyses families) — it is new, larger surface, deferred to a later concern like the earlier
modules were. No in-subset construct was un-matchable; the whole MVP subset reproduces byte-exact.

### S2 — flat byte-memory + structs and arrays in the self-interpreter (second slice, 2026-07-10)

The self-interpreter (`selfhost/interp/interp.cnr`) gains a FLAT BYTE-MEMORY model
plus STRUCTS and ARRAYS, extended to match the Rust reference (`compiler/src/interp/`) byte-exact
on aggregate programs. Same gate (`compiler/tests/selfhost_interp.rs`, EXECUTION equality vs
`run_source_real`): the corpus grows from 24 to 36 fixtures (34 returns, 2 faults) — the 12 new
S2 fixtures cover struct literal + field read, nested struct, field assignment, struct as by-value
fn param + return, mixed-width struct layout (bool/i8/i64 padding), array literal + index read,
array repeat `[e; N]`, index assignment, array of structs, struct containing array, an aggregate
`Bounds` fault (`a[len]`), and a mixed struct/array program with ordered `trace`s.

FLAT MEMORY MODEL (S1's value model CONVERGED onto memory). `E` now carries a byte arena
`mem: [N]u8` + an `init: [N]u8` written-byte bitmap + a `stack_bump` (ported from src/interp/mem.rs;
STATIC/STACK bases are free choices since addresses never surface in the RET/TRACE/FAULT dump, so
the arena is sized small — 16 KiB — with headroom, reset per block AND per call to bound loops and
recursion). Locals/params/aggregates live at bump-allocated addresses; a scalar local is
stored/loaded by (address, width) via little-endian `mem_store`/`mem_load` (sign/zero-extended to
mirror the oracle's read_int), REPLACING S1's pure `i64` slot. Scalar arithmetic still flows through
the `cur_val`/`cur_w` registers (byte-exact); an aggregate value is carried as an ADDRESS —
`cur_w == 0` marks it, `cur_val` is the address, `cur_ty` its type node. Faithfulness invariant:
all 24 S1 fixtures pass byte-exact THROUGH the new model. `init` is marked on every write (the
uninit-guard structure is kept), but the guard's BadPointer fault stays a later (pointer) slice.

TYPE/LAYOUT TABLE EXTRACTED FROM THE ARENA AST (scoping blocker #1, DISCHARGED). A type descriptor
is an arena node: scalar keywords are `T_SC` (NOT `T_NAMED` — the transcription trap that first
read every scalar local as an aggregate address), user structs are `T_NAMED`, arrays are `T_ARRAY`
(`.a` length, `.b` element), and a struct VALUE carries its `T_STRUCT` decl node directly.
`ty_size`/`ty_align` mirror src/interp/layout.rs: scalars by width (bool/i8/u8=1, i16/u16=2,
i32/u32=4, else 8); arrays = round_up(size,align) stride × len (literal `[N]` lengths only —
named/`static` lengths need a const table, deferred); structs walk the `T_SFIELD` chain in DECLARED
order at natural alignment, size rounded up to the struct's alignment (= max field alignment).
Field offsets are computed by the same running-offset walk; field-init evaluation follows declared
order (observable when a field value calls `trace`).

STRUCT + ARRAY OPS. Struct literals write each field at its offset (whole-slot zero-fill first, so
padding counts as initialized); field read/assign compute `base + field_offset`; `let`/by-value
param/return copy the bytes (aggregate returns land in a caller-owned `ret_slot` reserved below the
frame's reset point, so they survive the stack_bump reset). Array literals (`[a, b, ...]` listed and
`[e; N]` repeat), index read/assign compute `base + i*stride`; the `Bounds` fault (`i >= len`, shared
harness code 4) reports the base array expression's span, matching eval.rs's eval_place cur_span
threading exactly. Nested structs, arrays of structs, and structs containing arrays all landed.

COPY-ONLY / DROP-DEFERRED BOUNDARY. S2 stays on COPY aggregates (`copy struct`s and arrays of copy
scalars), so there is NO drop obligation to model — `trace` output is purely explicit `trace(x)`
calls, never a drop side-effect. The move/drop schedule (and trace-on-drop) is S3. Also deferred:
named/`static` array lengths, `conv`, borrows/`read`/`write` params, and field/index off a bare call
result (`make().f`) — S2 fixtures assign to a local first. interp.cnr remains NOT self-checked.

### S3 — MOVE/DROP schedule with trace-on-drop in the self-interpreter (third slice, 2026-07-10)

The self-interpreter (`selfhost/interp/interp.cnr`) gains the MOVE/DROP SCHEDULE —
the observable-trace crux of the language — reproducing the Rust reference (`src/interp/eval.rs`)
byte-exact on drop-bearing programs. Same gate (`compiler/tests/selfhost_interp.rs`, EXECUTION
equality vs `run_source_real`): the corpus grows from 36 to 44 fixtures (42 returns, 2 faults) —
the 8 new S3 fixtures cover single drop, reverse-order scope drops, move-suppresses-drop, a
one-level partial move (drop the un-moved field only), move-out via return (drop in the caller,
not at callee exit), a break-path drop, nested-aggregate order (outer hook then inner-field hook,
reverse), and by-value param drop at the callee's param-scope exit. Confined to interp.cnr — NO
parser change; the drop-hook block was already parsed into `T_STRUCT.c` and the `copy` marker into
`T_STRUCT.op` by slice 2's parser. All 36 prior interp fixtures and every lexer/parser/checker/
analyses/loans/effects self-check gate stay byte-exact (585 suite green, 0 failing; clippy clean).

STATIC-FACT DROP SCHEDULE COLLECTED DURING THE WALK (mirrors mod.rs:36-45). No runtime drop flags:
the interpreter recomputes ownership as it walks (it does NOT read the analyses' move facts) and
drops exactly the statically-owned places at each scope/stmt/param-scope exit in §1.5 reverse/LIFO
order. Machinery added to `E`: a per-local `loc_owns` flag (true on `let`/owned param, false on the
hook's borrowed `self`); a flat move-mask side table (`mv_local`/`mv_field`, field = -1 = whole
local moved) covering whole-value suppression and ONE-LEVEL partial move; and a non-copy temp stack
(`tmp_*`) for values materialized mid-statement. `is_copy`/`needs_drop` are ported from
`src/types.rs` (a struct is `copy` iff its `T_STRUCT.op` marker is set, it has no hook, and every
field is copy; it NEEDS drop iff it has a hook or transitively holds a needs-drop field). EVERY S3
addition is gated behind `!is_copy` / `needs_drop`, so S1's scalars and S2's `copy` aggregates are
drop-inert and produce byte-identical output — the faithfulness invariant across all 36 prior
fixtures.

TRACE-ON-DROP FIRES AS AN ORDINARY SELF-HOST PROGRAM (mirrors run_drop_hook eval.rs:3421-3438).
`drop_value` walks the type: a struct runs its hook FIRST (whole value) unless partially moved,
THEN drops fields in reverse declared order skipping moved fields; an array drops elements reverse;
scalars/copy are inert. `run_drop_hook` saves/restores `cur_base`/`loc_n`/`stack_bump` like a call
frame and binds `self` DIRECTLY to the value's address (`loc_addr = addr`, `loc_ty = the T_STRUCT
node`, `owns = false`) — simpler than the oracle's pointer indirection because interp.cnr already
carries aggregates by address — so `trace(self.id)` inside the hook resolves through the existing
field-read path and appends to the same `trace_out` sink, getting nested-hook trace order right.
Since interp.cnr is not self-checked it cannot spell the reserved `self` as a binding via source;
`self`'s name span is found by scanning `src` once for the `s e l f` bytes (a hook program always
contains it), and `find_local`'s byte-comparison matches the hook body's `self` tokens against it.

CONSUME-COMPLETENESS + ABORT-NO-DROP (the two easiest regressions, both honored). Move suppression
is a `consume` at EVERY move-out site of the S3 subset, recorded via a `cur_org` register the value
producer sets (temp / whole-place / one-level field-place): let-init-from-a-place, assignment RHS
(which also drops the OLD target value first, after the RHS, honoring its mask — preventing a
double free when the RHS moved the target out), struct-literal field init, by-value aggregate
argument, and the return operand (do_return moves into the result slot then consumes its origin, so
unwinding scopes and the param-scope see it moved and skip it). A MISSING consume double-drops; the
subset is covered exhaustively (`?` and deeper-than-one-level partials are out of subset — the
oracle's remaining consume sites at 1278/1289/1351/1946/1983 are match/enum/`?` paths this slice
does not reach). Abort semantics (eval.rs:3183): a `Ctl::Fault` returns from `exec_block`/
`exec_stmt`/`call_fn` WITHOUT running scope/temp/param drops (`if st != 1` guards every drop loop);
only Break/Continue/Return/normal fall through to the drop. Block/statement VALUES are preserved by
saving/restoring the `cur_val`/`cur_w`/`cur_ty` registers around each drop loop (a no-op for S1/S2).

RESTRICTIONS / DEFERRED (OBL-SELFHOST-INTERP-DROP). (1) ONE-LEVEL PARTIAL MOVE only: the move-mask
side table records a moved field by index on its direct-local root, so `a.f` moved out is handled,
but a deeper `a.f.g` partial move is not tracked (it would need a path-vector mask like the oracle's
`MoveMask`); deferred until a fixture needs it. (2) Non-copy ARRAYS are not drop-tracked as temps
(array literals carry no element type in `cur_ty`), so an array-of-drop-values materialized as a
bare temp would leak its drop — out of the S3 subset (fixtures use struct aggregates). (3) Borrow
(`read`/`write`) params are assumed by-value-owned (`owns = 1`); a `read Noisy` param would be
wrongly dropped — out of subset (all S3 params are by-value). (4) Field/index REASSIGNMENT drops
its old value only for direct-local (`T_ID`) targets, matching the oracle's `place_is_local_direct`
gate for the tested cases; a `x.f = ...` old-value drop is best-effort. interp.cnr remains NOT
self-checked.

### S4 — ENUMS and MATCH in the self-interpreter (fourth slice, 2026-07-10)

The self-interpreter (`selfhost/interp/interp.cnr`) gains ENUM VALUES and `match`,
reproducing the Rust reference (`src/interp/{layout,eval}.rs`) byte-exact. Same gate
(`compiler/tests/selfhost_interp.rs`, EXECUTION equality vs `run_source_real`): the corpus grows
from 44 to 50 fixtures (all returns; one also traces a drop schedule). Confined to interp.cnr — NO
parser change; slice-2's parser already emits `T_ENUM`/`T_VARIANT`, `T_ENUMCTOR`, `T_MATCH`/`T_ARM`,
and the `T_WILD`/`T_BIND`/`T_PVARIANT` pattern nodes. All 44 prior interp fixtures and every
lexer/parser/checker/analyses/loans/effects self-check gate stay byte-exact (585 suite green, 0
failing; clippy clean).

ENUM LAYOUT (mirrors src/interp/layout.rs). `{tag: u64 @0, payload @8}`: the payload is laid out
struct-style (declared order, natural alignment) from offset 8; enum size = `round_up(8 +
max-padded-payload, 8)`, enum align = 8 always; the tag is the variant's 0-based declared index.
`find_enum`/`enum_of_ty` mirror `find_struct`/`struct_of_ty`, and `enum_of_ty` resolves BOTH a
`T_NAMED` local/param type and a `T_ENUM` decl node carried directly in `cur_ty` by a constructor
result (the R1 scrutinee-ambiguity: a scrutinee's `cur_ty` is one or the other). The payload chain
is a raw type-node `nx`-chain, so `payload_size`/`variant_payload_off` generalize S2's struct-field
layout over a chain whose elements ARE the field types. `ty_size`/`ty_align`, `is_copy`, and
`needs_drop` extend their `T_NAMED` (and direct-`T_ENUM`) paths to enums: an enum is `copy` iff its
`T_ENUM.op` marker is set and every variant payload is copy; it NEEDS drop iff any variant payload
needs drop (enums carry no drop hook of their own).

CONSTRUCTION + MATCH (mirrors eval.rs ~2876/2948). `T_ENUMCTOR` resolves the enum by name and the
variant by name over the `T_VARIANT` chain, allocates the enum size, writes the tag as u64 @0, then
stores/copies each payload arg at its `variant_payload_off` (consuming a non-copy arg's origin, and
registering the whole value as a non-copy temp exactly like a struct literal). `T_MATCH` evaluates
the scrutinee to its enum place, reads the tag, selects the FIRST source-order arm whose pattern
matches (a `T_PVARIANT` by variant-name equality; `T_WILD` and a bare `T_BIND` match any), then runs
the arm body in a fresh scope with the same save/restore of `loc_n`/`stack_bump` and reverse-order
scope-drop as `exec_block`, propagating the arm's status code and leaving its value in the register
(R5). Pattern binding pulls each payload sub from the active variant's offsets: a COPY payload is
byte-copied to a fresh bump slot and owned; a NON-COPY payload is ALIASED in place and the
scrutinee's payload field is marked moved in S3's one-level field-granular move-mask (`mv_push(root,
i)`), so the scrutinee's later drop skips it (R3). ENUM DROP is tag-directed and is S4's alone (S3
did not know enums): `drop_value` reads tag @0, resolves the active variant, and drops that variant's
payload fields in reverse honoring the move-mask (`drop_variant_rev`, the dual of `drop_fields_rev`);
this is the whole of enum drop since enums have no hook. The COPY-payload match core is fully
S3-INDEPENDENT — copy payloads are byte-copied and never touch the mask or a drop point.

RESTRICTIONS / DEFERRED (OBL-SELFHOST-INTERP-ENUM). (1) OWNED scrutinees only: the scrutinee is
peeled no further than an owned enum place, so a borrowed/boxed scrutinee (the oracle's
`peel_scrutinee` Borrow/BorrowMut/Box arms, which bind sub-patterns by borrow) is out of subset
(R2) — mirrors the checker slice's owned-scrutinee-match boundary. (2) NESTED variant sub-patterns
are not bound (a `T_PVARIANT` sub inside a variant pattern) — the oracle faults on them too. (3)
Whole-scrutinee `T_BIND` binds nothing (the oracle no-ops it as well). (4) The non-copy partial-move
mark keys on the scrutinee's DIRECT-LOCAL root (`cur_org == 2`), matching S3's one-level mask; a
constructor-temp scrutinee's partial move is not tracked (out of subset — all fixtures match a
local). (5) BoxResult (compiler-synthesized, Box-payload variants) is DEFERRED to S5, which reuses
this enum layout + tag-directed drop. interp.cnr remains NOT self-checked.

### S5a — ALLOCATOR-ABI FOUNDATION in the self-interpreter (fifth slice, part a, 2026-07-11)

The self-interpreter (`selfhost/interp/interp.cnr`) gains the machinery `box` needs
but NOT box itself (that is S5b): `rawptr`/`fn`-ptr as scalar values, top-level `static`
evaluation, fn-name-as-value, INDIRECT calls through a fn-ptr, structural `Alloc`/`AllocVtable`
identification, and the minimal raw-pointer surface. Same gate
(`compiler/tests/selfhost_interp.rs`, EXECUTION equality vs `run_source_real`): the corpus grows
from 50 to 54 fixtures (all returns). Confined to interp.cnr — NO parser change; the parser
already emits `T_RAWPTR`/`T_FNPTR` types, `T_STATIC`, `T_UNSAFE`, and the `T_CASTPTR`/`T_ADDRTOPTR`/
`T_PTRNULL` intrinsic nodes. All 50 prior interp fixtures and every lexer/parser/checker/analyses/
loans/effects self-check gate stay byte-exact (585 suite green, 0 failing; clippy clean).

RAWPTR / FNPTR AS 8-BYTE SCALARS (the R3 prerequisite). `scalar_width` classifies `T_RAWPTR`(17)
and `T_FNPTR`(22) as an 8-byte, 8-aligned `usize`-coded scalar (and `ty_size`/`ty_align`/
`fn_ret_width` follow); without this, a `rawptr u8`/`usize`/fn-ptr argument to an indirect call
mis-routes as an aggregate in `call_fn`. A rawptr VALUE is a plain `u64` address into the flat
model memory; a fn-ptr VALUE is the callee fn's ARENA NODE ID (the self-interp has no fn-table
layer, so an indirect call is just `call_fn(pp, src, e, fnid)`) — this differs numerically from the
oracle's `fn_id_of` index, but a fn-ptr value is never surfaced as an observable, so dumps still
match byte-exact. A scalar load now carries the place/local's type node in `cur_ty` (harmless for
plain scalars, which ignore it) so `ptr_read` can recover a rawptr's pointee type via `rawptr_inner`
(the pointee lives in `.a` of any of `T_RAWPTR`/`T_CASTPTR`/`T_ADDRTOPTR`/`T_PTRNULL`).

FN-NAME-AS-VALUE + STATICS + INDIRECT CALL (mirrors src/interp/eval.rs:722-728, 270-303, 1604-1610).
`T_ID` resolves a bare name as local, THEN static, THEN function: a fn hit yields the fn's node id
as an 8-byte usize (fn-name-as-value); `eval_place` gains the same static fallback so `addr_of` of a
static works. `run_statics` is a two-phase pre-pass (phase 1 reserves each `static`'s bump storage;
phase 2 evaluates every initializer into its slot, resetting the bump between statics so only the
static storage below `static_top` survives into `main`), run before `main` so a vtable static a
fn-ptr program reads is already populated. `eval_call` resolves the callee as a declared fn
(`find_fn`) or, on a miss, evaluates the callee EXPRESSION to a fn-ptr value and calls that node id —
the oracle's indirect path; the arg loop then routes by the resolved fn's declared param types.

STRUCTURAL Alloc IDENTIFICATION (mirrors src/interp/eval.rs:236-248, NEVER by name). At startup,
like the oracle's `Interp::new`, `find_alloc_vtable_struct` finds the struct with fn-ptr fields
`alloc` AND `free`, and `find_alloc_struct` finds the struct whose `vt` field is a `rawptr` to that
vtable; the two struct ids are stored on `E` as the ABI seam. (The self-host checker uses
`Alloc`/`AllocVtable` as its own allocation prelude but carries no structural predicate — that
predicate lives in the oracle, which this mirrors.) S5a computes+stores them; S5b's box/unbox READ
them and resolve `ctx`/`vt`/`alloc`/`free` offsets via `field_off` — the documented S5b hook.

MINIMAL RAWPTR SURFACE (mirrors eval.rs builtins + intrinsic exprs). `T_CASTPTR`/`T_ADDRTOPTR`/
`T_PTRNULL` are address-value producers (reinterpret / usize-to-ptr over the SMALL arena / null=0);
`addr_of`/`addr_of_mut`, `ptr_read`, `ptr_write`, `is_null` are `T_CALL` builtins dispatched by
callee name. `ptr_read` loads a scalar pointee into the register or copies an aggregate pointee to a
fresh bump slot (registering a non-copy temp like a call return); `ptr_write` stores a scalar or
`mem_copy`s an aggregate through the address (consuming a non-copy source). `T_UNSAFE` runs its body
block (the ops are the same primitives, just check-gated). Fixtures: `static_fnptr_indirect_call`
(fn-name-as-value + static eval + indirect call through a fn-ptr-valued local and static field),
`ptr_roundtrip` (addr_of/ptr_write/ptr_read a local + is_null/ptr_null; the return proves the
pointer aliases the local), `cast_ptr_read` (cast_ptr as address-reinterpret then a 1-byte
ptr_read), and `alloc_abi` (a full bump `AllocVtable`/`Alloc`/`Bump` mirroring analyses.cnr:
main reads the `alloc` fn-ptr out of the vtable and INDIRECT-CALLs `alloc(ctx,size,align)` twice
over the internal `[8192,16384)` window, proving ctx threading, the bump advancing, and usable
in-arena pointers — the whole ABI foundation end-to-end, WITHOUT box).

DEFERRED to S5b (clean seams left, none implemented here): `box`/`unbox`/`BoxResult` and
Box-deref/alloc-on-drop. Absolute addresses differ from the oracle (16384-byte SMALL arena vs the
oracle's 256 MiB space; DENSE/high-address memory is S6), but only VALUES are observable so dumps
stay byte-exact. interp.cnr remains NOT self-checked (it uses the full language, incl. `unsafe` and
the raw-pointer intrinsics).

### S5b — BOX / BoxResult / unbox / Box-deref + alloc-on-drop in the self-interpreter (fifth slice, part b, 2026-07-11)

The self-interpreter (`selfhost/interp/interp.cnr`) gains THE HEAP on top of S5a's
allocator-ABI foundation: `box`/`unbox`, the compiler-known `BoxResult` enum, `.*` Box-deref
(and field/index auto-deref THROUGH a Box), and alloc-on-drop. Same gate
(`compiler/tests/selfhost_interp.rs`, EXECUTION equality vs `run_source_real`): the corpus grows
from 54 to 60 fixtures (all returns). Confined to interp.cnr — NO parser change (the parser already
emits `T_BOX`/`T_BOXRESULT` type nodes, `box`/`unbox` as `T_CALL` builtins, and `.*` as
`T_PREFIX`/`PF_DEREF`). All 54 prior interp fixtures and every lexer/parser/checker/analyses/loans/
effects self-check gate stay byte-exact (585 suite green, 0 failing; clippy clean).

LAYOUT (mirrors src/interp/layout.rs). `Box T` = `{ptr@0, ctx@8, vt@16}`, size 24, align 8;
`BoxResult T` = the 2-variant enum `{boxed(Box T), oom}`, size 32 (round_up(8 + 24, 8)), align 8,
tag@0, the Box payload @8. `ty_size`/`ty_align` return 24/8 and 32/8 for `T_BOX`/`T_BOXRESULT`;
`is_copy` is false and `needs_drop` is true for both (wired into S3's schedule and S4's enum
machinery).

SYNTHETIC BoxResult ENUM (the reuse mechanism). The parser never emits enum/variant nodes for
`BoxResult T`, so at STARTUP (`synth_boxresults`, where `write P` is available) we APPEND, for every
`T_BOXRESULT` annotation node, a `{boxed(Box T), oom}` shape: a `T_BOX` payload node, two
`T_VARIANT` nodes (their name spans point at the `boxed`/`oom` BYTES scanned from the source, so a
`match` pattern name compares equal), and a `T_ENUM`, linked back through the `T_BOXRESULT` node's
`.c`. `enum_of_ty` routes a `T_BOXRESULT` to this synthetic enum — so `match`, enum-size and
enum-drop (`eval_match`/`drop_variant_rev`) reuse the S4 machinery UNCHANGED; only `T_BOX` needs new
handling (a `drop_box` arm, is_copy/needs_drop/size/align cases).

box(alloc, v) (mirrors src/interp/eval.rs bi_box). Reads `ctx`/`vt` from the Alloc handle (arg0,
`field_off_lit` name-agnostic off `alloc_sid`), sizes `v` (arg1), reads the `alloc` fn-ptr out of
the vtable and INDIRECT-CALLs `alloc(ctx,size,align)` via `call_fn`. On null → OOM: drop+consume `v`,
tag 1; else MOVE `v` into the heap slot and build the Box {ptr,ctx,vt} in the boxed payload (tag 0).
The result's `T_BOXRESULT` node rides a new `cur_exp_ty` register set by the enclosing `let` (the
same node the synthetic enum is linked from) — so a fixture must let-bind the box result with an
explicit `BoxResult T` annotation (which also dodges the E0302 partial-move-at-join the checker
raises on a fall-through `match`). unbox(box) reads the Box, MOVES the pointee bytes into a fresh
slot, FREES the storage (`call_free`), and CONSUMES the box origin (frees AND consumes). `.*` deref
is a `T_PREFIX`/`PF_DEREF` place: read ptr@0 and become a read-through PLACE of the inner type (does
NOT free); `peel_box_place` auto-derefs field/index bases through Box layers.

ALLOC-ON-DROP + THE SHARED-RETURN-REGISTER TRAP. `drop_box` (dispatched from `drop_value` for a
`T_BOX`) drops the pointee FIRST (recursive inner-type drop — a Box of a Box recurses) THEN frees
through the vtable — pointee-then-free order is load-bearing. Because `free` is a real INDIRECT call
executed DURING a drop, it clobbers the self-interp's shared `ret_val`/`ret_w` registers with the
freed fn's unit return; a `return v` inside a `match` arm whose Box drops on the way out lost `v`.
Fix: save/restore `ret_val`/`ret_w` (alongside `cur_val`/`cur_w`/`cur_ty`) around EVERY drop loop
(`eval_match`, `exec_block`, `call_fn`, `exec_stmt` temps) — any drop that can invoke `free` after a
`return`. Fixtures: `box_unbox_scalar` (box+match+`.*`-read), `box_struct` (24-byte Box + field
auto-deref `bx.x` and `bx.*.y`), `unbox_path` (move-out + tracing free + consume ordering),
`boxresult_oom` (zero-headroom window forces the oom arm; the un-boxed value is dropped+consumed),
`box_drop_frees` (the acceptance signal: hook-then-free order on Box drop), `nested_box`
(`Box (Box T)` — recursive `drop_box`, boxed-payload-is-Box).

The self-interpreter now executes S1 scalars + S2 structs/arrays + S3 move/drop + S4 enums/match +
S5 the heap. The systems corpus is now gated ONLY on S6 (DENSE high-address memory model) plus the
`offsetof`/`ptr_offset`/`ptr_to_addr` intrinsics — the last self-interp gap before the systems
programs run. interp.cnr remains NOT self-checked (it uses the full language, incl. `unsafe` and the
raw-pointer intrinsics).

### S6a — PAGED memory model + the three pointer intrinsics (sixth slice, part a, 2026-07-11)

The self-interpreter (`selfhost/interp/interp.cnr`) swaps its flat 16 KiB byte arena for a
PAGED backing store and adds the three pointer intrinsics the systems corpus needs — the
INFRASTRUCTURE for that corpus; the five corpus programs (11_1..11_5) are S6b, a separate slice.
Same gate (`compiler/tests/selfhost_interp.rs`, EXECUTION equality vs `run_source_real`): the corpus
grows from 60 to 66 fixtures (all returns). Confined to interp.cnr — NO parser change (the parser
already emits `T_OFFSETOF`, and `ptr_offset`/`ptr_to_addr` are `T_CALL` builtins). All 60 prior
interp fixtures stay byte-exact through the paged model (the migration-without-regression check), and
every lexer/parser/checker/analyses/loans/effects self-check gate stays green (585+ suite, 0 failing;
clippy clean).

WHY PAGED. The systems corpus uses fixed addresses up to ~16.9 MiB (0x1013800), unreachable in the
16 KiB arena. A dense 17 MiB array is memory-feasible but INIT-time-infeasible: the oracle running
interp.cnr initializes an `[N]u8` array-repeat with a guarded per-byte move (~18M for 17 MiB). Paging
allocates+zeroes only TOUCHED pages, so E stays small (a 128-frame pool = 512 KiB) while addressing a
sparse 32 MiB space.

THE MODEL. `E` gains `pages:[524288]u8` (128 × 4096-byte frames), `pagedir:[8192]i32` (page-number →
pool slot, sentinel -1, covering 0..32 MiB), and `page_bump:usize`. `xlate(e, addr)`: `page = addr
>> 12`, `off = addr & 4095` (4096 = 2^12, so shift+mask, no division); on first touch of a page it
binds the next `page_bump` slot and ZEROES that frame; returns `slot*4096 + off`. `mem_load`/
`mem_store`/`mem_copy`/`zero_slot` route every byte through `xlate` — translating each byte
independently, so a load/store/copy SPANNING a page boundary (a struct straddling 4096) is correct.
`bump_alloc` is unchanged (it hands out addresses; xlate maps them lazily). ZERO-ON-PAGE-ALLOC is
load-bearing: a whole-value copy of a padded enum (a `leaf(i64)` variant of a 24-byte `Node` writes
only bytes 0..15) reads the unwritten tail, which must read 0 to match the oracle (whose init-byte
guard would otherwise fault). The self-host bump bases stay LOW (stack_bump from 8) and diverge from
the oracle's STACK_BASE — fine, only values are observable; fixed high addresses (≥2 MiB) never
collide with sub-MiB bump locals.

INIT ARRAY DROPPED. The vestigial `init:[16384]u8` (the oracle's uninit-guard bitmap, ported but
WRITE-ONLY in the self-host — it was never read; the self-host `mem_load` has no guard) is removed
along with the flat `mem`, eliminating an array and its per-write marking.

THE THREE INTRINSICS. `offsetof(T, field)` → `T_OFFSETOF` expr (node `.a` = struct type, `[.p0,.p1)`
= field name-span): resolve the struct via `struct_of_ty`, return the field's byte offset by name-
span from the S2 layout table (`field_off`) as a usize. `ptr_offset(p, n)` → name-dispatched `T_CALL`
builtin: base = the 8-byte address in `p`; stride = `ty_size(inner of p's rawptr type)`; result =
`base + n*stride` (`n` is isize, may be negative), returning a rawptr of the same inner. `ptr_to_addr
(p)` → builtin: the pointer's address as a plain usize (a rawptr scalar already carries its address,
so only the type changes). `unbox` was S5b; `field_ptr`/`container_of`/`sizeof` are unused by the
corpus and NOT added.

MIGRATION FIX. An array-repeat element `[0 - 1i32; N]` (untyped `0`) mis-types the element in the
oracle and trips its init-byte guard; `[0i32 - 1i32; N]` types cleanly — used for the `pagedir`
sentinel fill. FIXTURES (each byte-exact vs `run_source_real`): `high_addr_roundtrip` (u32
write/read + ptr_to_addr at a fixed 3 MiB address — paged high-address + MMIO access), `offsetof_
first_field` (offset 0) and `offsetof_nonzero_field` (a padded later field at 8 — offsetof mirrors
layout alignment), `ptr_offset_stride` (index a `[3]Pt` by pointer arithmetic — stride = size_of(Pt)
= 16), `enum_padding_copy` (whole-move a padded `Node::leaf` twice — proves zero-on-page-alloc
satisfies the guard, the load-bearing correctness for S6b's 11_5), `page_boundary` (an i64 straddling
a 4096 boundary with non-zero bytes in both halves — cross-page store/load). NOTE: interp.cnr does
not implement `conv` in the INTERPRETED program, so the fixtures branch/compare instead of casting.
RUNTIME: paging adds ~0.12 s/fixture (~25%), dominated by the oracle's per-fixture 512 KiB `pages`
zero-init; correctness landed, cost noted.

The self-interpreter now executes S1 scalars + S2 structs/arrays + S3 move/drop + S4 enums/match +
S5 the heap + S6a paged memory & pointer intrinsics. The systems corpus (11_1..11_5) is now gated
ONLY on S6b — the corpus programs themselves. interp.cnr remains NOT self-checked.

### S6b — THE MILESTONE: the self-interpreter RUNS the five systems-corpus programs (sixth slice, part b, 2026-07-11)

The self-hosted Candor interpreter (`selfhost/interp/interp.cnr`) now EXECUTES all five
systems-corpus programs — Candor's hardest real programs — each byte-exact against the Rust reference
(`run_source_real`). Same gate (`compiler/tests/selfhost_interp.rs`, EXECUTION equality): the corpus
grows from 66 to 71 fixtures (61 returns, 10 faults). Confined to interp.cnr + the gate list (plus one
harness line mapping `ConvLoss`) — NO parser change (the parser already emits every needed node). All
66 prior interp fixtures stay byte-exact, every lexer/parser/checker/analyses self-check gate stays
green (585 suite, 0 failing; clippy clean).

THE FIVE (each RET oracle-matched): `11_3_mmio` RET 42 (enums/match + `addr_to_ptr` MMIO at a fixed 3
MiB window), `11_1_allocator` RET 1234 (a pool free-list allocator, `box` matched directly), `11_2_
scheduler` RET 42 (a rawptr intrusive list with a hand-written `container_of` = `cast_ptr` +
`ptr_offset` + `conv isize offsetof`), `11_5_arena` RET 5 (a `Box [4096]Node` arena, ~96 KiB, + a
recursive `fold_consts`), `11_4_parser` RET 17 (a recursive-descent expression parser over a `b"..."`
byte-slice, enums with `Box` payloads, `box`/`unbox`).

WHAT S6b IMPLEMENTED (each a real interp fix mirroring the oracle, not a fixture special-case):
- `conv` (`T_CONV`, the #1 blocker, unimplemented before). Mirrors `src/interp/eval.rs::eval_conv`:
  read the source integer at its declared signedness, range-check against the TARGET scalar's
  `(min,max)` (unsigned magnitudes compared via u64); in range keeps the bit pattern at the new width,
  out-of-range in the checked regime is a `ConvLoss` fault (`FK_CONVLOSS = 5`, added to the harness's
  kind map; the corpus stays in range). All five use `conv`; 11_2 also needs it inside `container_of`.
- BORROW parameters (`read`/`write` params, e.g. `state: write Bump`, `a: read Alloc`). The parser
  UNWRAPS a borrow param — it stores the inner type in `T_PARAM.a` and the borrow kind in `T_PARAM.op`
  (1 = read, 2 = write) — so the interp never saw a borrow node and copied the POINTEE by value (the
  `mk_alloc`/`pool_handle` `ctx = &st` bug). `synth_param_borrows` gives each such param a real
  `T_BORROW`/`T_BORROWMUT` node (rewriting `.a`), so it is stored as an 8-byte pointer scalar;
  `PF_READ`/`PF_WRITE` expressions (`write st`) yield a place ADDRESS; `peel_box_place` now peels
  borrows as well as boxes, so `param.*`/`param.field` deref correctly.
- SLICES + byte-strings (11_4): `[u8]`/`[T]` (`T_SLICE`/`T_SLICEMUT`) size 16 / align 8; `b"..."`
  (`T_BYTES`) decodes its bytes into memory and materializes a `{ptr,len}` fat pointer (escape
  decoding shared with the lexer's `decoded_len`); `len(s)` reads the length field; slice indexing
  reads `ptr`/`len` and bounds-faults, distinct from array indexing.
- The compiler-known `BoxResult` shape WITHOUT an annotation (11_1/11_5 `match box(...)` directly, and
  11_4's `BoxResult::boxed(lhs)`/`::oom` ctors). A generic `{boxed(Box _), oom}` scaffold routes such
  values; the boxed inner type rides `e.gen_inner`, set at each `box` from the value's type — a
  synthesized `T_SC` node for a scalar value (`box(a, 1234)`), a synthesized `[N]T` node for an
  array-rep value (`box(al, [Node::leaf(0); 4096])`, so it can be sized). `box`/`unbox`/deref/drop
  consult `gen_inner` when the box node is the generic one.

NOTHING NEEDED S7+. Every one of the five is monomorphic and lands entirely within the S1–S6 value
model + these additions; no generics/monomorphization, no `for`/iterators, no `Vec`/`Map`, no `?`
were required. RUNTIME: the gate runs ~82 s (the oracle running interp.cnr over the corpus, dominated
by 11_5's ~96 KiB arena build + the recursive drop of 4096 copy `Node`s, and 11_4's recursive parse/
eval); correctness landed, cost noted. The self-hosted interpreter now executes Candor's full
systems-programming surface — allocators, intrusive lists, MMIO, an arena, and a recursive-descent
parser — matching the reference engine byte-for-byte. interp.cnr remains NOT self-checked.

### interp.cnr consolidation + self-check probe (2026-07-11)

CONSOLIDATION (output-invariant; all 71 interp fixtures + every self-check gate + the whole suite
byte-exact, clippy clean). Four review-specified cleanups to `selfhost/interp/interp.cnr`:
- The identical drop-owned-locals loop — save the five value registers, drop owned not-moved locals
  `[base, loc_n)` in reverse, restore the registers — was hand-copied in `exec_block`, `call_fn`, and
  `eval_match`. Factored into `fn drop_owned_locals_from(pp, src, e, base) -> i32` (returns 1 if a
  drop faulted); all three sites now call it. The `ret_val`/`ret_w` half of that save IS the S5b
  register-trap fix (a `free` run mid-drop clobbers the shared return register); it now lives in ONE
  place, so a future S7 drop caller can't silently reintroduce the bug. `exec_stmt`'s temp-drop keeps
  its own save/restore: its loop iterates the temp table (`tmp_live/tmp_addr/tmp_ty` over
  `[tmp_base, tmp_n)`, killing live temps), a structurally different loop from the owned-locals
  reverse drop, and the only shareable part (the 5-register save) would need a carrier struct /
  multi-value return that adds more machinery than the duplication it removes.
- Removed three dead `use parser` imports (`T_RET`, `T_ARM`, `T_NAME`) — each appeared only in the
  `use` list, never referenced.
- `eval_place`'s `T_ID` fallthrough (after a local-miss AND static-miss) re-read `loc_addr[loc_n]`,
  the out-of-frame sentinel slot — a silent wrong answer. Now faults explicitly (`FK_PANIC`): an
  unresolved name is an internal invariant violation. Unreached by the corpus.
- `xlate` (paged translate) gained a capacity guard: an address ≥ 32 MiB overran `pagedir:[8192]i32`
  and exhausting the 128-frame `pages` pool overran `pages:[524288]u8`, both silent OOB. Now signals
  a `FK_BOUNDS` fault (the nearest oracle bad-address kind) on `page >= 8192` or pool exhaustion.
  Defensive for S7's Vec/Map; the corpus stays in bounds, so no fixture output changes.

SELF-CHECK PROBE — the "NOT self-checked (uses `unsafe` and the raw-pointer intrinsics)" blocker
above is STALE. interp.cnr's OWN code uses no `unsafe`, no raw-ptr intrinsic calls, no `self`, no
generics/Map/Vec; its heavy constructs (`wrapping{}`, `conv`) are already self-checked in
parser/analyses.cnr. Running interp.cnr through the SAME machinery that gates the other modules:
- EMBED: the self-host lexer produces 25736 tokens at the time (< the 32768 buffer; NOTE after the later I-std collection port interp.cnr is ~32712 tokens, only ~56 under the cap) and the parse
  fits the 32768-node arena with no fault — NO arena bump needed.
- CHECKER (name-res, E0102/E0103): the self-host checker resolves interp.cnr's `use lexer::{..}` /
  `use parser::{..}` imports itself and emits ZERO E0102/E0103, byte-equal to the module-aware oracle
  over lexer+parser+checker+interp (also 0). The naive single-file oracle (no import resolution)
  flags 202 E0102 + 222 E0103, so the imports are load-bearing and really resolved.
- ANALYSES (move/init/loans/effects/exhaustiveness): the self-host analyses emits ZERO covered
  diagnostics (E0301/E0304/E0401/E0601/E0801-4), byte-equal to the module-aware oracle (0). The
  predicted E0302 partial-move-at-join false-positive class does NOT surface — E0302/E0309 are
  out-of-subset for the analyses core, and the FULL Rust oracle already accepts interp.cnr (it
  compiles and runs under the execution gate).
READ: self-checking the interpreter is a NEAR MILESTONE, not a project. Landing it is essentially
adding interp.cnr as another module under the existing checker + analyses fixpoint gates (same shape
as parser.cnr's gate, with a use-after-move teeth smoke) — no arena bump, no new builtins. (Probe was
run-and-report; no probe test committed.)

### Self-check fixpoint COMPLETE — interp.cnr now self-checks (2026-07-11)

The self-check probe above is now a COMMITTED gate with teeth. Two gates added, mirroring the
existing per-module fixpoint gates (same shape as the parser.cnr/analyses.cnr gates):
- `compiler/tests/selfhost_checker.rs::candor_checker_checks_interp_source_clean_via_import_resolution`
  — the self-host CHECKER name-resolves interp.cnr clean (empty E0102/E0103), byte-equal to the
  module-aware oracle over the real lexer+parser+checker+interp tree. Teeth: the naive single-file
  check flags the unresolved imports E0102 (>0), and an injected unknown-type param fires E0102.
- `compiler/tests/selfhost_analyses.rs::candor_analyses_check_interp_source_clean_fixpoint`
  — the self-host ANALYSES (move/init E0301/E0304, loans E0801-4, effect E0401, exhaustiveness E0601)
  emits an EMPTY covered set over interp.cnr, byte-equal to the module-aware oracle. Teeth: an
  injected use-after-move fires E0301.

Both pass CLEAN, oracle-matched, exactly as the probe predicted — no construct in interp.cnr fails to
check clean. NO arena bump needed at the time (25736 tokens; NOTE now ~32712 after the I-std collection port -- ~56-token margin, see the token-cap constraint in ROADMAP).
This completes the self-check fixpoint across ALL FIVE self-host modules: the lexer, parser, checker,
analyses core, AND the interpreter that executes them all name-resolve clean under the self-host
checker and pass the self-host move/init/loan/effect/exhaustiveness analyses — each byte-equal to the
Rust oracle. Runtime: the interp checker gate ~46s, the interp analyses gate ~49s (debug); the
Map-in-checker perf lever remains if these grow. All 71 interp fixtures + every prior self-check gate
stay byte-exact; clippy clean.


### Self-lowering L0 — the MIR serialization boundary (2026-07-11)

The enabler for the self-lowering tier: a MIR program can now cross from a serialized
TEXT form into the Rust MIR interpreter, gated EXECUTIONALLY against the tree-walking
oracle. Built SOLO, Rust-side only — NO Candor lowering yet (that is L1+). The boundary
is proven on REAL MIR produced by the existing Rust lowering (`mir::build`), so it is
validated before any Candor code depends on it.

- WIRE FORMAT (`compiler/src/mir/serial.rs`): a canonical, deterministic, human-readable
  S-expression text for a whole `MirProgram`. Atoms are bare keywords/decimal integers;
  names and string literals are `"…"`-quoted with `\\ \" \n \t` escapes; whitespace-
  insensitive on read, canonically indented on write. Deliberately simple to EMIT — the
  L1 Candor lowering emits exactly this text. It faithfully carries everything the MIR
  interpreter reads from `MirProgram`: every fn/block/statement/terminator/rvalue/operand/
  place/proj, each `LocalDecl` (type + name + drop_obligation), the full recursive `Type`
  (incl. `FnPtr` params/alloc/foreign, arrays/slices/box/rawptr), fault edges as
  `(fedge KIND start end)` — spans are LOAD-BEARING (the FAULT comparison checks span
  identity), projection offsets + index stride/len/span/slice, the `Drop` move masks
  (`(moved (path …) …)`), the `fn_ptrs` table (order = id, load-bearing for indirect
  calls), `drop_hooks` (sorted by key for determinism), and the `statics` table. The
  derived `fn_index` (name → position in `fns`) and the runtime `items`/`consts` are NOT
  on the wire: `fn_index` is rebuilt on load; `items`/`consts` are derived from the fixture
  SOURCE by the Rust front-end, exactly as `lower_and_run` does (the harness rebuilds them).

- SERIALIZER/DESERIALIZER: `serialize(prog) -> String` / `deserialize(s) -> Result<MirProgram, String>`,
  round-trip-exact (`serialize(deserialize(serialize(p))) == serialize(p)`).

- GATE (`compiler/tests/mir_serial.rs`): for each corpus fixture — lower source to MIR
  via `mir::build`, serialize → deserialize → run via `mir::interp::run(prog, rebuilt
  items, consts)`, render RET/TRACE/FAULT in the `selfhost_interp` schema, and assert
  byte-exact to `run_source_real` (the oracle). Three facts per fixture: (1) wire round-trip
  idempotence; (2) deserialized MIR == oracle; (3) in-memory MIR == deserialized == oracle
  (the boundary changed nothing).

- PROVEN: all 71 corpus fixtures round-trip byte-exact vs the oracle (61 returns,
  10 faults) — the same corpus the self-interp gate uses: scalar core + every scalar fault
  kind (spans/fault identity), aggregates (struct/array field offsets, strides, copyval),
  move/drop schedule (Drop move masks), enums + match, allocator ABI (fn_ptrs table,
  statics, indirect calls, structural Alloc), box/BoxResult/unbox + alloc-on-drop
  (drop_hooks → fn ids), paged memory + pointer intrinsics, and the five-program systems
  corpus (11_1..11_5). NOTHING deferred — every MIR construct the existing Rust lowering
  emits round-trips. This is the boundary L1's Candor lowering must EMIT INTO: emit this
  exact wire text (carrying spans + fault edges + move masks + fn_ptrs + drop_hooks +
  statics), and the Rust MIR interpreter runs it identically to the oracle.

Additive: every existing gate stays green; new gate green in isolation; clippy clean.

### Self-lowering L1 — the FIRST Candor lowering: scalar + control-flow → MIR (2026-07-11)

The MVP of the self-lowering arc: a Candor program (`selfhost/lower/lower.cnr`, composed
with the `lexer` + `parser` modules) reads a Candor program from the self-host parser's
Node arena, lowers the SCALAR + CONTROL-FLOW subset to MIR, and EMITS the L0 wire text
(the exact `mir::serial` format), so the Rust `deserialize` accepts it and `mir::interp`
runs it. Built SOLO. This proves Candor can emit an executable control-flow graph — the
first time a Candor-authored lowering feeds the MIR interpreter. It ports the scalar half
of the Rust reference lowering (`compiler/src/mir/build.rs`).

- SCOPE (scalar + control flow ONLY): integer/bool scalars; `let`/assignment; `if`/`else`;
  `while`; `loop` + `break`/`continue`; `return`; arithmetic `+ - * / %` in the Checked/
  Wrapping/Saturating regimes with the Overflow/DivByZero fault edges; comparisons;
  `&&`/`||` (short-circuit → branch blocks); bitwise/shift; unary neg/not; `trace(x)`;
  `assert`/`panic` (fault edges); direct non-generic fn calls with scalar params + return;
  `conv` (ConvLoss edge). DEFERRED to L2+: structs/arrays/enums/box/pointers/drop, `match`,
  aggregate params/returns, `?`, statics, contracts (`requires`/`ensures`), borrow params.

- IR-BUILDER (`struct M`): a flat local table (each `LocalDecl` is a scalar width — L1 is
  scalar-only, so drop_obligation is always false and there is no layout table) and a basic-
  block table. Each statement's and terminator's wire text is rendered ONCE, at emit time,
  into a byte POOL as a chunk; a block links its statement chunks intrusively (a block is
  built OUT of creation order via `switch_to`, so the text is buffered and assembled in
  block-id order at emit time). `new_local`/`new_block`/`emit_temp` mirror build.rs's
  allocation order.

- CFG FLATTENER: structured control flow → basic blocks + Goto/Branch/Return/Fault
  terminators, porting build.rs's `lower_if`/`lower_while`/`lower_loop` block shapes and a
  loop stack of `(continue_bb, break_bb)` for `break`/`continue`; join blocks; `&&`/`||`
  lower to a result local + rhs/short/join branch blocks. `reachable` tracking replicates
  build.rs's `terminate`/`emit` no-op-when-unreachable, so post-`return`/`break` code is
  correctly dropped.

- WIDTH INFERENCE: `lit_width`/`concretize` mirror build.rs's `int_type`/`concretize` — a
  literal's width is its suffix, else the integer `expected` width threaded down, else i64;
  fn return/param widths come from the arena type nodes (`T_RET`→`.a`, `T_PARAM`→`.a`).

- FAULT-EDGE SPANS (load-bearing — the FAULT comparison checks span identity): a threaded
  `cur_span` (set to each node's span at `lower_value` entry, and to the op-node span for a
  fallible bin) supplies the exact `(kind, span)` the oracle delivers — bin overflow/div at
  the T_BIN node span (div-by-zero is delivered at run time off the same `Bin.span`; the
  edge kind is always `overflow`, matching build.rs), neg-overflow at the operand span,
  conv-loss at the operand span, `assert` at the condition's trailing span, `panic` at the
  T_PANIC node span.

- WIRE EMISSION: the whole `MirProgram` text — `(mir (fns …) (drop_hooks) (fn_ptrs …)
  (statics))` — is emitted through the built-in `trace` byte sink (each traced value is one
  output byte; the harness rebuilds the string), matching `serial.rs`'s atom encoding
  (bare keywords/decimals, `"…"`-quoted names). `fn_ptrs` lists every fn name in program
  order; `drop_hooks`/`statics` are empty in the scalar subset. Because the port follows
  build.rs's `new_local`/`new_block` order faithfully, the emitted wire is byte-IDENTICAL to
  `serialize(mir::build …)` on 19/24 fixtures.

- GATE (`compiler/tests/selfhost_lower.rs`): for each in-subset fixture — run `lower.cnr`
  in the tree-walker over the embedded source to produce the wire text, `deserialize` it
  Rust-side, rebuild `items`/`consts` from the same SOURCE, `mir::interp::run`, render
  RET/TRACE/FAULT, and assert byte-exact to `run_source_real` (the oracle). PROVEN:
  EXECUTION byte-exact on all 24 scalar-subset fixtures (15 returns, 9 faults) — arith,
  rem, ifelse, while_accum, loop_break, factorial, fib, shortcircuit, compare, bitwise,
  unary, assert_pass, trace_multi, width_i8, u64_value; and the fault fixtures overflow_i32,
  divzero, assert_fail, panic, width_i8_overflow, width_u8_overflow, u64_add_overflow,
  u64_sub_underflow, i64_mul_overflow. Every faulting op reaches the oracle's exact
  fault identity (kind + span).

- FINDING (OBL-L1-TRACE-SPAN, benign): the emitted wire is byte-identical to
  `serialize(mir::build …)` on 19/24 fixtures; the 5 that differ (while_accum, fib, bitwise,
  trace_multi, u64_value) differ ONLY in the source SPAN of `trace(x)` STATEMENTS — off by
  one byte, because the self-host parser's `T_CALL` node does not record the call's closing-
  paren end that the Rust AST call span includes. Statement spans are INERT for execution
  (only fault-edge spans surface in RET/TRACE/FAULT), so all 24 still execute byte-exact.
  This is a self-host-parser span-representation gap, NOT a lowering gap — recorded, not
  faked or special-cased.

Additive: every existing gate stays green; new gate green in isolation; clippy clean.

### Self-lowering L2 — STRUCTS and ARRAYS: flat aggregates → MIR (2026-07-11)

L2 extends `selfhost/lower/lower.cnr` from scalars to flat COPY aggregates, porting the
aggregate half of the Rust reference lowering (`compiler/src/mir/build.rs`): struct/array
literals, field/element access + assignment, and by-value struct params/returns. Built SOLO.
DEFERRED to L3+: drop schedule (L3), enums/`match` (L4), box/pointers (L5), slices, `?`,
statics, contracts, borrow params.

- LAYOUT TABLE (copied into lower.cnr, adapted from `selfhost/interp/interp.cnr` +
  `src/mir/layout.rs`): `ty_size`/`ty_align`/`struct_size`/`struct_align`/`field_off`/
  `field_ty`/`array_len_of`/`stride_of`, pure over the Node arena + the item-list `head`
  (not interp's `write E` — the E-coupled functions are NOT imported; per-module duplication
  is the accepted idiom). Declared-order fields at natural alignment; struct size rounded to
  max field alignment; array stride = `round_up(elem_size, elem_align)`. Widths→byte sizes
  via `w_size`. These MUST agree with the Rust interp's `lay()` (which sizes locals from
  `items`), and do: field offsets and strides are byte-exact vs `serialize(mir::build …)`.

- PLACES + PROJECTIONS: an lvalue lowers to `(place <root> (proj <entries>))`. `emit_projs`
  walks `T_ID`/`T_FIELD`/`T_INDEX`, appending `(field <offset> <fieldty>)` and
  `(index <op> <stride> <len> <sp0> <sp1> false)` in base-first order (matching build.rs's
  `Place.proj` order and serial.rs's `proj_to` encoding). `pool_ty`/`emit_ty` render the
  Type s-expr (`(scalar w)` / `(array <elem> (litlen N))` / `(named "S")`) — load-bearing:
  the interp derives a Store's width from the leaf `Proj::Field.ty`, and a local's byte size
  from its `LocalDecl` type, so aggregate locals carry their real type node (`new_local_tn`).

- BUFFERS: destination places (`let`/`return`/assign targets and each aggregate sub-field)
  build into a persistent `dbuf` (proj-entry text), so they survive the value lowering that
  follows; that value lowering renders its own READ places directly to the pool inline — the
  two never alias, which is what lets a struct-literal fill emit field-read values (`Vec2 { x:
  u.x + v.x, … }`) without clobbering the destination prefix. `dbuf` appends reuse the pool
  primitives as scratch then relocate the bytes (`reloc_to_dbuf`).

- RVALUES/STATEMENTS (byte-exact vs build.rs): struct literal → per-declared-field `Store`
  into `dst.field(off)` (recursing for nested structs); array literal → per-element `Store`
  into `dst.index(i, stride, len)`; array-repeat `[e;N]` → eval `e` once into a temp, then N
  `CopyVal`s into each slot; scalar field/element READ → `Load` from the projected place into
  a temp; field/element ASSIGN → `Store` to the projected place.

- BOUNDS FAULT EDGE (load-bearing): `Proj::Index` carries `kind Bounds` implicitly and the
  BASE expression's span (build.rs threads `cur_span` to the base after lowering the index),
  captured in `emit_projs` right after the base recursion. `array_bounds` (`a[3]`) faults
  `Bounds` at span 61–62 (the `a`) byte-exact vs the oracle. Index subexpressions must be a
  constant or a simple local in the L2 subset (so the place renders with no mid-statement
  emission) — true for every S2 fixture.

- BY-VALUE STRUCT PARAM/RETURN ABI (caller-owned return slot, mirroring build.rs): an
  aggregate ARG is passed as the ADDRESS of its place — `assign <rawptr tmp> (ref <place>)`,
  operand `(oplocal tmp)` — and the callee copies it into its param slot (`is_wordy` false →
  `copy_bytes`). A struct-returning call materializes via `assign <rawptr tmp> (call …)`
  (the call yields the return-slot address) then `copyval <dst> (place <tmp> (proj (deref
  <ty>))) <ty>`; the callee lowers `return <struct>` straight into `_0`.

- GATE (`compiler/tests/selfhost_lower.rs`, S2 fixtures added to the existing corpus): each
  runs `lower.cnr` in the tree-walker → wire → `deserialize` → `mir::interp::run` → byte-exact
  RET/TRACE/FAULT vs `run_source_real`. PROVEN EXECUTION byte-exact on all 12 S2 aggregate
  fixtures (11 returns + 1 fault): struct_field, nested_struct, field_assign, struct_param_ret,
  struct_mixed_width, array_index, array_repeat, index_assign, array_of_structs,
  struct_with_array, aggregate_mixed, and array_bounds (the Bounds fault). All L1 scalar
  fixtures remain byte-exact.

- FINDING (extends OBL-L1-TRACE-SPAN, benign): the emitted wire's field offsets, index
  strides, `Proj` chains, field types, local aggregate types, the bounds fault span, and all
  aggregate statement/rvalue shapes are byte-identical to `serialize(mir::build …)`. The ONLY
  wire diffs are INERT span-only differences on non-observable statements (and on in-bounds
  array-literal index projections and the `trace` STATEMENT span — the traced VALUE, not the
  span, is what RET/TRACE/FAULT compares), rooted in the self-host parser recording
  name-only/zero spans for `T_FIELD`/`T_INDEX`/`T_STRUCTLIT` nodes where the Rust AST carries
  full-expression spans. The one load-bearing span (the `array_bounds` fault) matches. This is
  a self-host-parser span-representation gap, NOT a lowering gap — recorded, not faked.

Additive: full suite 589/589 green (0 failing); new S2 gate green in isolation; clippy clean.


---

## OBL-L3-DROP-SCHEDULE — self-lowering, third slice: the MOVE/DROP schedule to explicit MIR (2026-07-11)

L3 extends `selfhost/lower/lower.cnr` from drop-inert aggregates to the full drop
schedule: it emits explicit MIR `Drop` statements with static move masks and lowers
each struct `drop` hook to a MIR function, mirroring `src/mir/build.rs`.

- MOVE TRACKING (the compile-time analog of build.rs's `moves` map + `mark_moved`):
  a per-local list of moved field-name paths, recorded as moves are lowered — a
  non-copy whole-aggregate copy (`let b = a;`, `p.a`), and a by-value non-copy `take`
  argument. `collect_path` walks an Ident/Field chain to a (root local, segment) path
  (one-level partial move suffices, matching interp.cnr's S3 subset); `reassign_reown`
  clears overlapping rows on reassignment. Marking is gated on `!is_copy` (ported from
  interp.cnr), so S1 scalars and S2 `copy` aggregates record nothing.

- DROP EMISSION (mirrors `pop_scope_with_drops`/`emit_return_drops`/`emit_loop_exit_drops`):
  drop-scopes track owned locals in declaration order (params scope + each block
  scope). At scope exit, at `return`, and at `break`/`continue` (down to the loop's
  captured scope depth), each owned, needs-drop, not-(whole-)moved local emits
  `(stmt (drop <local> (moved <field-paths>)) 0 0 false)` in REVERSE declaration order,
  carrying its move mask so the interp drops only the un-moved sub-paths. Borrows /
  pointers and copy/scalar locals emit nothing (`needs_drop` ported from interp.cnr,
  also driving each local's `drop_obligation` wire flag). Abort-no-drop is honored: the
  fault path is unreachable, so no `Drop` is emitted on it.

- DROP HOOKS AS MIR FNS (mirrors build.rs's hook lowering + `drop_hooks` registration):
  each `struct … drop(write self) { … }` lowers to an ordinary MIR fn `<drop Name>`
  (`self` = a drop-inert `borrowmut` local; the body reuses the L1/L2 machinery, its
  `self.field` reads auto-deref the borrow). The fn is registered in the `drop_hooks`
  table `(hook "Name" "<drop Name>")` and the `fn_ptrs` list; the interp calls it when
  dropping that struct (hook FIRST, then fields reverse — the recursion lives in the
  interp's `drop_value`, so `drop_nested` emits ONE `Drop` for the outer local).

- GATE (`compiler/tests/selfhost_lower.rs`, 8 S3 fixtures added to the corpus): each
  runs `lower.cnr` → wire → `deserialize` → `mir::interp::run` → byte-exact RET/TRACE/
  FAULT vs `run_source_real`. PROVEN EXECUTION byte-exact on all 8 — the trace-on-drop
  ORDER is the load-bearing signal: drop_single (one hook), drop_scope_order (reverse
  3/2/1), drop_move_suppress (moved source not dropped), drop_partial_move (drop the
  un-moved field b only: 1 then 2), drop_move_return (moved into return, dropped in the
  caller: 42), drop_break (loop-exit drop: 100 then 2), drop_nested (outer hook then
  inner-field hook, reverse: 2 then 1), drop_param (by-value struct-literal arg
  materialized + dropped at the callee's param-scope exit: 3). All 36 prior L1+L2
  fixtures remain byte-exact.

- FINDING (extends OBL-L2-AGG-SPAN, benign): the emitted `Drop` ops, their `(moved …)`
  masks, the `drop_hooks` table, the `fn_ptrs` order, the hook fns' locals/blocks, and
  every local's `drop_obligation` flag are byte-identical to `serialize(mir::build …)`
  (0 structural diffs across all 8 fixtures). The ONLY wire diffs remain the same INERT
  span-only differences the L1/L2 field-read lowering already exhibits (the self-host
  parser records name-only spans for `T_FIELD`/`T_INDEX` where the Rust AST carries
  full-expression spans) — none load-bearing (no S3 fixture faults). No lowering gap.

- DEFERRED: deeper-than-one-level partial moves (out of the interp's S3 subset too);
  enum/`Box` drop glue (L4/L5b); `drop_hooks` table SORTING (build.rs sorts by struct
  name; lower.cnr emits in item order — the interp reads the table into a HashMap, so
  execution is order-independent, and the S3 fixtures' item order already coincides with
  sorted order, hence 0 wire diff here).

Additive: new S3 gate green in isolation (44/44 lower fixtures); clippy clean.


---

## OBL-L4-ENUM-MATCH — self-lowering, fourth slice: ENUMS and MATCH to MIR (2026-07-11)

L4 extends `selfhost/lower/lower.cnr` from structs/arrays to user ENUMS and `match`:
enum layout, the `T_ENUMCTOR` tag+payload store, and the `match` tag-switch branch
chain, plus the consuming-match / tag-directed enum-drop interaction with L3. Ports
`src/mir/build.rs` `lower_enum_ctor`/`lower_match`/`lower_match_arm` and reuses the
S4 enum layout of `selfhost/interp/interp.cnr`. Plain user enums only — `BoxResult`
(a synthesized enum with a Box payload) is L5.

- ENUM LAYOUT (ported from interp.cnr / `src/interp/layout.rs`): `{tag:u64@0,
  payload@8}`; the payload is laid out struct-style (declared order, natural
  alignment) from offset 8; `enum_size = round_up(8 + max padded-payload, 8)`, align
  8 always; tag = the variant's 0-based DECLARED index. The payload chain is a raw
  type-node `nx`-chain (each node IS a field's type), so it reuses the L2 field-walk.
  `find_enum`/`enum_id_of`/`enum_size`/`variant_by_index`/`variant_index_by_name`/
  `variant_payload_off`/`variant_payload_ty` are added, and `ty_size`/`ty_align`/
  `needs_drop_ty`/`is_copy_ty` route `T_NAMED`-that-is-an-enum (and bare `T_ENUM`)
  through them.

- T_ENUMCTOR (mirrors build.rs `lower_enum_ctor`): allocate nothing new — write into
  the destination place (a `let`/return slot). Store the variant tag as a `u64`
  const at field 0, then `lower_into` each argument at its `variant_payload_off`. A
  scalar payload stores; an aggregate payload (e.g. a struct-literal `Noisy { … }`)
  recurses through the existing L2 aggregate machinery.

- T_MATCH branch-chain CFG (mirrors build.rs `lower_match`, statement position): read
  the tag ONCE into a `u64` temp (`(load (place root (proj … (field 0 (scalar
  u64)))) (scalar u64))`), create the `join` block FIRST, then per VARIANT arm emit a
  `Cmp eq (oplocal tag) (const idx u64)` into a bool temp, `Branch` into a fresh
  `arm_bb` else a fresh `next_bb`, lower the arm in `arm_bb`, `Goto join`, then
  `switch_to next_bb` (the fall-through test chain). A WILDCARD/BINDING arm is the
  unconditional tail (bind nothing, `Goto join`, stop). A non-exhaustive tail faults
  (`Panic`), mirroring the oracle's "no matching arm" (unreachable for a checked
  exhaustive match, but emitted). Block-creation order (join, then arm/next per
  variant) and the Cmp/Branch/Load shapes match build.rs BYTE-FOR-BYTE.

- PAYLOAD BINDS (mirrors build.rs `lower_match_arm`): each arm binds its sub-patterns
  in a fresh drop-scope (so the arm-scope drop fires AFTER the body, before `Goto
  join`). `T_WILD` binds nothing; `T_BIND` binds by payload index. A COPY payload
  copies — a wordy scalar via `(store (place loc (proj)) (load <payload place> ty))`,
  a non-wordy aggregate via `CopyVal`. A NON-COPY payload of an OWNED scrutinee
  MOVE-binds: `CopyVal` into a fresh local + mark the scrutinee's `_i` sub-path moved.
  (Nested variant sub-patterns are out of subset — flagged E-unsupported, none
  needed.)

- CONSUMING-MATCH + ENUM DROP (the L3 interaction): the move-mask gains a SYNTHETIC
  `_i` segment kind (`mv_seg_syn`), emitted in the `Drop` op's `(moved (path "_i"))`
  mask, so a bound-and-moved payload is pruned from the scrutinee's later drop. Enums
  carry NO hook: `needs_drop_ty` returns true iff a variant payload needs drop, and
  the scrutinee's `Drop` is resolved TAG-DIRECTED by the interp (`drop_enum` reads the
  live tag, drops the active variant's payloads in reverse, honoring the move mask) —
  the lowerer emits ONE `Drop` per needs-drop enum local; the tag-directed resolution
  lives in the interp, unchanged.

- GATE (`compiler/tests/selfhost_lower.rs`, 6 S4 fixtures added to the corpus): each
  runs `lower.cnr` → wire → `deserialize` → `mir::interp::run` → byte-exact RET/TRACE/
  FAULT vs `run_source_real`. PROVEN EXECUTION byte-exact on all 6 — enum_construct_match
  (tag write + tag-switch + scalar payload bind + branch chain), match_wildcard (unit
  variants + wildcard tail), enum_multi_variant (mixed-width payload offsets i16@8/
  i64@16), match_bind_multi (three-payload binds → TRACE 10/20/30), enum_result_shape
  (enum by-value return via the caller-return-slot ABI, then matched), and
  enum_drop_payload (the load-bearing drop-order signal: TRACE 1, then the moved
  payload's hook 7 at arm-scope exit, then 2, then the un-consumed `some(Noisy)`
  scrutinee's tag-directed drop 8; the `Two::a`-moved `_0` is pruned so it does NOT
  double-drop). All 44 prior L1+L2+L3 fixtures remain byte-exact (50 total).

- FINDING (extends OBL-L3-DROP-SCHEDULE, benign): the emitted enum tag store, payload
  offsets/types, the tag-read Load, the Cmp/Branch chain, arm block IDs, payload-bind
  Stores/CopyVals, the `(moved (path "_i"))` synthetic paths, and every enum local's
  `drop_obligation` flag are byte-identical to `serialize(mir::build …)` (0 structural
  diffs across all 6 fixtures). The ONLY wire diffs remain the SAME inert span-only
  differences the L1/L2/L3 lowering exhibits (the self-host parser records name-only/
  glued spans where the Rust AST carries full-expression spans) — none load-bearing
  (no S4 fixture faults). No branch-chain, payload-bind, or consuming-drop wire gap.

- DEFERRED: value-producing `match` (`let x = match … { => expr }`, arm result into a
  dst place) — build.rs supports it via `lower_into`, but no S4 gate fixture exercises
  it (all six are statement-position with block bodies), so it is left unimplemented
  rather than shipped untested; a place-with-index/field or call scrutinee (fixtures
  use plain-local scrutinees); nested variant sub-patterns and guards (out of the
  interp's S4 subset too); `BoxResult` (L5b — a synthesized enum over a Box payload).

Additive: new S4 gate green in isolation (50/50 lower fixtures); full suite 589/589
green (0 failing); clippy clean.

## OBL-L5-BOX-RAWPTR — self-lowering, fifth slice: BOX/ALLOC ABI + rawptr/fnptr/statics/CallIndirect to MIR (2026-07-11)

L5 extends `selfhost/lower/lower.cnr` from enums/`match` to the pointer + allocation
surface: the rawptr/fnptr scalar operands and pointer intrinsics; top-level `static`
items; a fn name used as a value + indirect (`CallIndirect`) calls; the BOX/ALLOCATOR
ABI (`box`/`unbox`/`.*`-deref, the synthesized `BoxResult` enum) and alloc-on-drop.
This is the last infrastructure slice before the L6 systems-corpus milestone. Ports
`src/mir/build.rs`'s box/rawptr/static/indirect-call half; matches `src/mir/serial.rs`.

- RAWPTR/FNPTR SCALARS: 8-byte wordy operands. `ty_size`/`ty_align`/`needs_drop`/
  `is_copy` gain T_RAWPTR/T_FNPTR/T_BORROW(MUT) (8/8, no drop, copy), T_BOX (24/8,
  drop, not copy), T_BOXRESULT (32/8, drop, not copy), T_SLICE(MUT) (16/8). Type
  rendering (`pool_ty`/`emit_ty`) gains `(rawptr …)`, `(box …)`, `(boxresult …)`,
  `(fnptr <alloc> <foreign> <ret> (params ((<mode> <ty>) …)))`, borrow/slice, and
  the `(scalar unit)` fix. A new `is_wordy_tn` (scalar|rawptr|fnptr|borrow) routes
  `lower_into` to `Store (use …)` for pointer-wordy locals as build.rs does.

- POINTER INTRINSICS (byte-exact vs build.rs): `addr_of`/`addr_of_mut` → `Ref(place)`;
  `ptr_read`/`ptr_write` → `Load`/`Store` through a `(deref <inner>)` place, marked
  observable (INV-OBS-ORDER) iff the pointer is a rawptr; `cast_ptr`/`addr_to_ptr`/
  `ptr_null` → a `Use` reinterpret into a `(rawptr T)` temp; `is_null` → `IsNull`;
  `ptr_offset` → `PtrArith { base, index, stride = size_of(inner) }`; `ptr_to_addr`
  → `Use`; `offsetof` → a `(const <field-offset> usize)` via the L2 layout table.
  The pointee type + rawptr-ness of a pointer value are threaded on the result
  (`res_pointee`/`res_israw`) so a non-place pointer arg (e.g. `cast_ptr[T](ctx)`)
  resolves its pointee — the fix that unblocked the allocator's write-back.

- STATICS: each `static NAME: T = value;` lowers to an `<init NAME>()` fn returning
  the value into `_0` (emitted between drop hooks and user fns); a `(static "NAME"
  <ty> "<init NAME>")` row joins the wire `statics` table; a static read is modeled
  as `*(&STATIC)` — a `StaticAddr` temp + `Deref` place (`place_base_prep` pre-emits
  the `staticaddr` assign before the consuming statement).

- FN-PTR VALUES + CALLINDIRECT: a fn name as a value → `(const <id> u64)` where `id`
  is its fn-pointer id, assigned in program order over drop hooks / static inits /
  fns (`fn_ptr_id_of`, matching build.rs `reg`). The `fn_ptrs` table adds `<init …>`
  entries in item order. An indirect call (callee a fn-ptr local) → `CallIndirect`
  through the fn-pointer's declared param modes (`lower_args_fnptr`).

- BOX/UNBOX/BOXRESULT (reusing L4's enum layout/ctor/match/drop): `box(alloc,v)` →
  `alloc_addr_operand` (a `Ref` of the owned handle) + `materialize_place(v)` +
  `BoxOp { dst, inner, result_ty, alloc, value }`. `unbox(b)` → `UnboxOp` + whole-b
  move; `.*`/`bx.field` auto-deref a Box place (Box.ptr@0 ⇒ a plain `(deref …)`, so
  no interp distinction needed). `BoxResult T` is the synthesized 2-variant enum
  (boxed(Box T)@8 = 0, oom = 1): `lower_match_full` detects a `T_BOXRESULT` scrutinee
  and binds `bx` as a move-bind of the Box at offset 8, marking `_0` moved. A Box
  local (new `loc_box` flag: type = pointee node, renders `(box …)`, `drop_obligation`
  = true) and a BoxResult local both carry drop obligations; the interp resolves the
  `Drop` as pointee-then-free (alloc-on-drop) and enum-tag-routed BoxResult drop — the
  lowering emits only the `Drop`, no memory model.

- L4 DEFERRAL NOW IMPLEMENTED: value-producing `match` (`lower_match_dst` threads a
  dst place; each arm body lowers via `lower_into` to it, matching build.rs). It is
  not exercised by an L5 gate fixture (all box-matches bind a plain-local `BoxResult`
  in statement position) but is wired for the L6 corpus. STILL DEFERRED: a non-plain-
  local scrutinee (`match box(…)` / matching a call result — materialize to a temp
  Place first); no gate fixture needs it and it is added when an L6 program requires
  it.

- GATE (`compiler/tests/selfhost_lower.rs`, 16 S5/S6 fixtures added): each runs
  `lower.cnr` → wire → `deserialize` → `mir::interp::run` → byte-exact RET/TRACE/FAULT
  vs `run_source_real`. PROVEN EXECUTION byte-exact on all 16 — offsetof_first_field,
  offsetof_nonzero_field, ptr_roundtrip, cast_ptr_read, ptr_offset_stride,
  high_addr_roundtrip, page_boundary, enum_padding_copy, static_fnptr_indirect_call,
  alloc_abi, box_unbox_scalar, box_struct, unbox_path, boxresult_oom, box_drop_frees,
  nested_box. All 50 prior L1–L4 fixtures remain byte-exact (66 total).

- FINDING (extends OBL-L4-ENUM-MATCH, benign): 12/16 fixtures are 0 structural diffs
  vs `serialize(mir::build …)`; the emitted rawptr/box/fnptr/static/CallIndirect ops,
  the BoxOp/UnboxOp shapes, the fn-ptr-id constants, the `fn_ptrs`/`statics` tables,
  and every box/boxresult local's `drop_obligation` are byte-identical. The ONLY wire
  diffs are the SAME inert span-only differences (self-host parser records name-only/
  glued spans) PLUS, in 4 box/enum fixtures, the span on the UNREACHABLE
  non-exhaustive-`match` `(fedge panic …)` terminator (build.rs threads its `cur_span`;
  the lowering uses the match node span). That terminator is dead code for every
  exhaustive fixture — never delivered — so RET/TRACE/FAULT stay byte-exact. No box,
  rawptr, static, fn-ptr, CallIndirect, or alloc-on-drop op/wire gap was found; the L6
  systems corpus (11_1..11_5) is now unblocked.

Additive: new S5/S6 gate green in isolation (66/66 lower fixtures); full suite 589/589
green (0 failing); clippy clean.

## OBL-L6-SYSTEMS-CORPUS — THE MIR MILESTONE: the self-hosted lowering compiles all five systems-corpus programs to MIR (2026-07-11)

L6 is the milestone: `selfhost/lower/lower.cnr` now lowers ALL FIVE systems-corpus
programs to MIR wire text that the Rust MIR interpreter executes byte-exact
(RET/TRACE/FAULT) against the tree-walking oracle (`run_source_real`). The
`selfhost_lower` gate grows from 66 to 71 fixtures. Method per program: get
lower.cnr's wire, Rust-`deserialize` it, run `mir::interp`, diff against the
oracle AND a structural diff against `serialize(mir::build(source))` to localize
each divergence, then fix minimally mirroring `src/mir/build.rs`.

Per program (all PASS byte-exact):
- 11_3_mmio (RET 42) — enums/match + addr_to_ptr + a `write Uart` borrow param.
- 11_1_allocator (RET 1234) — box+match, `read a` borrow-value alloc handle.
- 11_2_scheduler (RET 42) — rawptr intrusive list, offsetof/ptr_offset container_of.
- 11_5_arena (RET 5) — `Box [4096]Node` arena, borrow return, deep recursion.
- 11_4_parser (RET 17) — `[u8]`/`b"…"`/`len`/slice-index + enums + Box + recursion.

Features implemented in lower.cnr, mirroring build.rs:
- BORROW PARAMS (`read`/`write T`): a param with mode 1/2 becomes a word-sized
  Borrow/BorrowMut local (loc_ptr 2=borrowmut, 3=borrow); a `read x`/`write x`
  value is `(ref (place x …))` into a borrow temp (build.rs `lower_borrow`); at a
  call it passes by value. Field/index/`.*` on a borrow use a DEFERRED peel
  (`plc_borrow`, mirroring the `plc_box` Box auto-deref) so `d.*.field` derefs the
  borrow ONCE — the earlier eager-deref-at-ident double-derefed. Wordy (rawptr/
  fnptr/borrow) params now pass BY VALUE in `lower_args` (was: aggregate-`ref`).
- NON-PLACE (materialized) MATCH SCRUTINEE (the L5 deferral): a pre-pass
  (`prep_synth`, run while the parse arena is still writable) records each
  non-place `match` scrutinee's type node — synthesizing a `BoxResult`/scalar/
  array/`named` node for `box(…)`, or reusing the callee return node for a call —
  then `lower_match_full` materializes the value into a temp (owned) or evaluates
  the borrow address (borrowed), and a `vscrut_*` short-circuit in
  resolve_place_root/emit_projs/collect_path/render_place_field renders every
  scrutinee sub-place off that temp.
- BORROW RETURN (`-> read T`): `prep_synth` synthesizes the Borrow node; lower_fn
  and lower_call size `_0`/the call result as a word borrow.
- BYTE-STRINGS + SLICES: `b"…"` (T_BYTES) / string (T_STR) materialize a `[u8]`
  slice header {f0=`straddr`, f8=`decoded_len`}; `len(slice)` loads f8 (array →
  const); a slice index emits `(index … 0 … true)` (bounds from the header).
- Box-typed FIELDS auto-deref on further index/field (`ar.*.mem[i]` where
  `mem: Box [N]Node`): `emit_projs`/`resolve_place_tn` peel a `T_BOX` leaf.
- Aggregate array-repeat element (`[Node::leaf(0); 4096]`): the element temp is an
  aggregate local, copyval'd into each slot (was: scalar-element only).
- Computed array index hoisting: `s[conv usize i]` — the index expression can emit
  its own statement chunk, which must land in the block, not inside the place
  bytes; `prep_place` pre-lowers every index to an operand cache consumed by
  `emit_projs`.
- Synthetic `BoxResult` CTOR construction: `BoxResult::oom` = tag 1;
  `BoxResult::boxed(x)` = tag 0 + copyval-move the Box payload at offset 8.
- Per-function pool grown 64 KiB → 1 MiB (4096 array-repeat copyvals overflowed it).

Nothing needed beyond the L1–L6 arc: all five programs are monomorphic and use only
features present in `src/mir/build.rs` (interp.cnr already ran them). No lowering
gap forced a `#[ignore]`. Runtime: the `selfhost_lower` gate ~131s (11_5's 4096-Node
arena and 11_4's parser dominate); each systems program lowers+runs in a few s
except 11_5 (~75s under the tree-walker).

Additive: `selfhost_lower` 71/71 byte-exact; `selfhost_interp` 71/71 unchanged;
all self-check + mir_serial gates green; full suite green (0 failing); clippy clean.

## OBL-G1-MONOMORPHIZER — the shared generic pre-pass: user generics resolved to concrete instances for self-interpret (2026-07-11)

The first slice of the generic/std self-hosting tail: a shared Candor MONOMORPHIZER
(`selfhost/mono/mono.cnr`, `use parser`), an engine-independent pre-pass
mirroring the Rust oracle `src/generics.rs`. It resolves USER generics
(`fn[T]`/`struct[T]`/`enum[T]`) into concrete instances over the parse arena, wired
into the self-host interpreter (`interp.cnr`) so the tree-walker runs generic
programs unchanged. Gated byte-exact (RET/TRACE/FAULT) against the Rust
`monomorphize → tree-walker` oracle. The systems corpus is monomorphic, so this
tail is cleanly separable from the S1–S6 interpreter milestone; L-gen (self-lowering)
will reuse this same pass.

STRUCTURE (mono.cnr, ~770 lines):
- DECL TABLES: a decl is generic iff its type-param list (`T_FN.a`/`T_STRUCT.a`/
  `T_ENUM.a`) is non-empty; `find_gfn`/`find_gstruct`/`find_genum` locate them by
  name span, monomorphic fns are the discovery roots.
- DISCOVERY + INFERENCE: the walk resolves each generic reference. Type args are
  recovered by `unify(param-or-ret pattern, concrete)` — unwrapping borrows/`T_RET`,
  binding a `T_NAMED` type-param, recursing `App` args pairwise. The primary signal
  is RETURN-type-vs-EXPECTED (a `let x: TY` annotation); a minimal `ty_of`
  (local-var env + `read`/paren) supplies ARG types for the residual cases
  (e.g. `unwrap_or(a, 0)` unifies `Opt[T]` against the local `a: Opt[i64]`).
  `T_GENERICVAL` (`f::[T]`) takes its args explicitly.
- WORKLIST/FIXPOINT/DEDUP/DEPTH: instances are emitted EAGERLY and recursively;
  each is registered in a dedup table BEFORE its body is cloned (cycle break), so a
  reached (decl, type-args) is emitted once. Identity is (orig decl, type-args)
  compared STRUCTURALLY (`same_type`) — exactly the equivalence class
  `inst_fn_name`/`mangle_ty` encode, so dedup matches the oracle without
  materializing the mangled string. `MONO_DEPTH_LIMIT = 64` backstops runaway
  instantiation (design 0007 §5.1.1).
- ARENA CLONE-WITH-SUBST: `clone_subst` deep-clones a decl subtree into fresh arena
  nodes, replacing each type-parameter `T_NAMED` with a fresh copy of the concrete
  type node (preserving the occurrence's sibling link). Cloned param/ret/field types
  then `resolve_type` (emit + stash nested nominal instances). Vec/Map/String stay
  `App` (no user generic decl → left untouched, resolved by the type table).
- CALL-SITE REWRITE (the span-based-naming reality): the interp resolves names by
  SOURCE SPAN, so a synthesized instance name is unaddressable. Instead each generic
  REFERENCE node carries the RESOLVED INSTANCE NODE ID in a spare arena field and the
  interp's lookups prefer it: `T_CALL.c`/`T_GENERICVAL.c` (fn instance),
  `T_STRUCTLIT.b` (struct instance), `T_ENUMCTOR.c` (enum instance), `T_APP.b`
  (a type reference → struct/enum instance). Dispatch is by node id, so instance
  identity is the structural type-key above rather than a literal mangled name.

INTERP WIRING (interp.cnr): `interp_dump` runs `mono_program(write p, src, head)`
after `parse_program` and before the synth passes/tree-walk, so the walker sees the
concrete arena. Five surgical, additive edits: a `T_APP && b != 0` redirect in
`struct_of_ty`/`enum_of_ty`/`ty_size`/`ty_align` (routing an instance type ref to its
concrete node), the stash reads in `eval_call`/`eval_struct_lit`/`eval_enum_ctor`, and
a `T_GENERICVAL` case (its value is the resolved instance's node id, a fn-ptr).
MONOMORPHIC programs are UNAFFECTED: `mono_program` is a no-op when no decl is
generic (all 71 existing interp fixtures stay byte-exact), and every stash defaults
to 0 (the pre-mono lookup path).

RUNS via mono → interp, byte-exact vs the oracle (8 of the 13 generic fixtures):
`mono3` (fn[T] at three types), `pair` (struct[T] + swap/mk), `genenum` (enum[T] +
match), `arena` (struct[T: copy] with `[N]T` field), `gdrop` (generic drop hook,
LIFO hook-then-field trace), `mixed` (region + type param list), `nameval`
(`f::[T]` named-as-value + indirect call), `gdrop_groundfloor` (a generic-instance
type named in a MONOMORPHIC fn's param — its annotation is stashed too).

DEFERRED (5 fixtures — a FRONT-END gap, NOT a monomorphizer gap): `iface`, `gimpl`,
`gbound` (interface/impl + method dispatch), `fromq`, `gfromq` (cross-type `?`/`From`).
The self-host PARSER does not parse `interface`/`impl` items
(`parse_item` handles only struct/enum/fn/static) and the interp has no method
dispatch, `?`, or `From` machinery; mono correctly no-ops on them (it clones the
generic struct/enum, but the method call resolves to nothing → RET 0 ≠ oracle).
Reaching these needs impl/interface parsing + mangled impl-method dispatch + the
`?`/`From` runtime — far beyond this slice. mono's mangling scheme (structural
identity) is already the right foundation for the impl-method dispatch names L-gen
and a future impl slice will need.

Additive: `selfhost_interp` 79/79 byte-exact (71 monomorphic unchanged + 8 generics);
self-check (checker E0102/E0103) and analyses fixpoints over `interp.cnr` stay green
(mono.cnr added to those oracle trees so interp's `use mono` resolves; mono.cnr is
check-clean); lower/mir_serial gates unaffected; whole suite green. mono.cnr does not
self-check this slice (it runs in the tree-walker, which requires it to type-check;
no fixpoint gate is claimed for it).

## OBL-L-GEN — the monomorphizer wired into the self-hosted MIR lowering: user generics compiled to MIR (2026-07-11)

The LOWER-tier counterpart of OBL-G1: the shared `mono.cnr` pre-pass is now wired
into the self-hosted MIR lowering (`selfhost/lower/lower.cnr`), so the
self-host compiles USER generics (`fn[T]`/`struct[T]`/`enum[T]`) to MIR wire text,
byte-exact vs the `monomorphize → build.rs → mir::interp` oracle. `lower_dump` runs
`mono_program` right after `parse_program`, then lowers the CONCRETE arena.

Mirrors interp's G1 wiring, adapted to a STATIC artifact keyed by NAMES (not runtime
node-id dispatch):
- LAYOUT REDIRECT: `struct_id_of`/`enum_id_of`/`ty_size`/`ty_align` follow
  `T_APP.b != 0` to the stashed concrete instance, so offsets/sizes/strides over a
  generic type are computed against its monomorphic instance.
- CALL-SITE STASH READS: `T_CALL.c` (fn instance), `T_STRUCTLIT.b` (struct instance),
  `T_ENUMCTOR.c` (enum instance), `T_APP.b` (type ref), and `T_GENERICVAL.c` (a
  `name::[T]` fn value → its fn-pointer id) drive lowering to the right instance.
- MANGLED INSTANCE NAMES: a MIR program has no struct table — `mir::interp` sizes
  `(named N)` via `items`, which the gate now builds from the MONOMORPHIZED program
  (mangled `id$i64`, `Pair$i64`, `Opt$i64`, …). So the wire must emit those exact
  names. `mono.cnr` now stashes each instance's concrete type-arg tuple as a cons
  chain in the instance node's `ival` (inert to the tree-walk interp, which never
  reads a decl's `ival`); lower reconstructs `base$<mangle_ty(arg)>` per arg
  (mirroring `src/generics.rs` `inst_fn_name`/`mangle_ty`) for fn defs, call callees,
  `(named …)` type refs, drop-hook names + the `drop_hooks` table, and the `fn_ptrs`
  list. Generic DECLS (`T_*.a != 0`) are skipped in every emission loop and the
  fn-pointer id count (they are parametric — not lowerable callables/types).

LOWER-SIDE GAP FOUND + FIXED (a real, pre-existing latent bug the `arena` fixture
first exercised): `lower_agg_copy` rendered a by-value aggregate read of an indexed
place (`return ar.mem[conv usize i]`) WITHOUT the `prep_place` pre-pass, so a
statement-producing index operand (`conv usize i`) was re-lowered by `emit_projs`
INTO the middle of the place projection — malformed wire (`unknown proj tag stmt`).
Fixed minimally by pre-lowering the static base + index operands (`place_base_prep`
+ `prep_place`) before rendering, exactly as `lower_place_value` already does.

ALL 8 of the G1-landed generic fixtures now LOWER → `mir::interp`, byte-exact vs the
oracle: `mono3` (plain `fn[T]` at three type args), `mixed` (region + type params →
`choose$i64`; regions excluded from the mangle), `nameval` (`dbl::[i64]` named-as-
value + indirect call), `pair` (`struct[T]` + value-arg inference + App field access),
`genenum` (`enum[T]` + match), `arena` (`struct[T: copy]` with `[N]T` field),
`gdrop_groundfloor` (a generic-instance type in a monomorphic fn's param), and `gdrop`
(a generic-struct drop hook — `Wrap$Noisy`'s hook runs then drops its `Noisy` field).

DEFERRED (same 5 as G1 — a FRONT-END gap, not a lowering gap): `iface`, `gimpl`,
`gbound`, `fromq`, `gfromq` need interface/impl parsing + mangled impl-method dispatch
+ `?`/`From`. Out of scope here.

Additive: `selfhost_lower` 79/79 byte-exact (71 monomorphic UNCHANGED — `mono_program`
is a no-op with no generics, every redirect/stash defaults to 0 — plus 8 generics);
`mono.cnr`'s `ival` stash is inert to `selfhost_interp` (79/79 still green); clippy
clean; whole suite green. Runtime: `selfhost_lower` ~140s.

## OBL-L-STD — std collections lowered to MIR CollectionOp: the generic/std self-hosting tail is COMPLETE (2026-07-11)

The FINAL slice of the generic/std tail: the self-hosted MIR lowering
(`selfhost/lower/lower.cnr`) now lowers the compiler-known `Vec[T]` /
`Map[V]` / `String` builtins to `StatementKind::CollectionOp { dst, op }`, so Candor
COMPILES collection programs to MIR (not just interprets them). With user generics
(L-gen) already lowering, Candor now compiles ANY in-subset program — user generics
AND std collections — to MIR.

Mirrors `src/mir/build.rs` `lower_collection_agg` / `lower_collection_value`, emitting
`serial.rs`'s `collop_to` wire byte-for-byte (accepted by the Rust `collop_from`
deserializer):
- AGGREGATE CONSTRUCTORS (in `lower_into`, before the user-fn agg path): `vec_new`/
  `map_new`/`string_new` → `CollOp::New { alloc }` (alloc = the `read Alloc` address,
  same operand build.rs `alloc_addr_operand` produces); `pop` on a Vec → `VecPop`
  into the `Opt[T]` destination; `as_str` on a String → `StringAsStr` into the `str`
  destination.
- VALUE OPS (in `lower_call`, before the user-fn lookup — receiver-type-gated, so a
  same-named user fn wins when arg0 is not a collection): `push`/`get`/`insert`/
  `contains`/`append`. Each emits the receiver ADDRESS operand (`(ref …)` of the
  `read`/`write` receiver), the element/value type, materialized value/key places
  (by-value args → a temp filled by `lower_into`; str/`[u8]` keys → a `{ptr@0,len@8}`
  view), and the fault-relevant span = the ARG span (per P0's finding), while the
  statement span stays the call span. `get`'s `read T` borrow result feeds the
  `get(…).*` deref: a `PF_DEREF` over a collection-`get` call lowers the call to a
  borrow local, then a `Load` through it (mirroring build.rs deref-of-call-result).
- `len` on `Vec[T]`/`Map[V]` → the offset-8 length word via a deref + field-8 load
  (build.rs), and on `str` via the fat-pointer field-8 load.
- TYPE-TABLE SPECIAL-CASE (the interp.cnr App-40 / String-40 analogue): `emit_ty`/
  `pool_ty` render an un-monomorphized `T_APP` (`b == 0`, i.e. Vec/Map) as
  `(app "Vec"/"Map" <elem>)` and the `str` keyword (parsed as `T_NAMED "str"`) as
  `(str)`; `needs_drop_ty` returns true and `is_copy_ty` false for Vec/Map/String
  (owning, allocator-bearing) — so a collection LOCAL is a drop obligation and gets a
  statically-scheduled `Drop`. COLLECTION DROP is the existing `Drop` op: the interp
  resolves the local's `(app "Vec" (named "E"))` type and runs each element's hook in
  reverse (vec_struct_drop's reverse element-drop trace is the load-bearing signal).

All 6 collection fixtures LOWER → `mir::interp`, byte-exact RET/TRACE/FAULT vs the
tree-walking oracle: `string_build` (push/append/as_str/len + buffer free on drop),
`vec_push_get_sum` (vec_new/push/get/len), `vec_pop_opt` (pop → `Opt` match),
`vec_struct_drop` (the 4→8 realloc raw-move then reverse per-element hook drops),
`map_insert_contains_get` (FNV-1a insert/contains/get + owned-key copies + rehash),
and `vec_get_oob_fault` (bounds fault at the arg span). No CollectionOp wire gap
found — the Rust deserializer accepts every emitted collop.

Additive: `selfhost_lower` 85/85 byte-exact (79 prior — the monomorphic + user-generic
corpus — UNCHANGED, plus the 6 collections); `mir_serial` 2/2 green (the reference MIR
boundary unaffected); clippy clean. Runtime: `selfhost_lower` ~154s, `mir_serial` ~2s.
No edits to `interp.cnr` / `mono.cnr` / `src/mir/*`. The generic/std self-hosting tail
is COMPLETE: Candor compiles user generics + std collections to MIR.

## OBL-QUALITY-REVIEW — findings from the cross-model quality audit (Fable 5 on Opus 4.8's work, 2026-07-11)

Four fresh-context quality reviewers graded the ~30h self-hosting arc: code GOOD-with-caveats,
Rust STRONG, docs MOSTLY-HONEST, architecture SOUND-with-bounded-debt. Genuineness was already
verified separately. Findings, with disposition:

**Fixing now (latent correctness + the acute blocker):**
- **F-LAYOUT-DRIFT:** lower.cnr's ty_size returns 0 for Vec/Map/String/str/Box (interp.cnr's does
  not) -- masked today because lower routes collections separately, but a `struct { v: Vec[i64] }`
  field would mis-size the struct. LATENT correctness bug.
- **F-MONO-SILENT:** mono.cnr ty_of returns 0 for literals/field/call; an un-inferred type-param
  leaves T_NAMED unsubstituted with NO diagnostic (silent-wrong). Fail loud instead.
- **F-FIND-NEEDLE:** synth_boxresults/synth_generic_boxresult scan the source for "boxed"/"oom"
  substrings WITHOUT a bounds guard (unlike synth_scw) -> OOB read if a BoxResult program lacks
  them.
- **F-ARENA-CAP:** interp.cnr self-checks with ~56 tokens under the [32768] arena -- already
  blocking the next interp feature. Raise the arena (stopgap) now.

**Deferred (documented debt, not blocking; fix when the trigger fires):**
- **F-VECSET-ORDER:** build.rs materializes Vec::set's value BEFORE the bounds check; the oracle
  checks bounds THEN evaluates. Divergence only if a faulting set has a side-effecting value arg;
  UNTESTED (no `.set(` fixture). Fix + add a fixture when Vec::set is exercised.
- **F-FAULT-TRACE:** the differential harness compares FAULT kind+span but NOT the pre-fault trace
  sequence (Fault carries no trace). So "byte-exact including trace" holds only for NON-faulting
  runs. Full fix: thread accumulated trace into Fault and compare. Until then the claim is scoped
  to non-faulting runs.
- **F-DESERIALIZE-ARITY:** src/mir/serial.rs deserialize indexes args positionally with no arity
  guard -> panics (not Err) on malformed wire. Matters now that Candor emits the wire. Add arity
  checks or document the trust boundary.
- **F-LAYOUT-EXTRACT:** ty_size/field_off/enum layout is triplicated (interp/lower/Rust). Extract a
  shared layout.cnr BEFORE a 4th consumer (trait-generics front-end or native codegen) appears.
- **F-FOUR-ENGINE-CLAIM:** the "matching the tree-walker is transitively matching native" framing
  is feature-level, not program-level -- the 11_* corpus does not itself run through the
  four-engine gate. Soften the doc language, or add 11_* to the four-engine gate.
- **F-TEST-DEDUP:** fault_code/dump_ok/on_big_stack/CORPUS are copy-pasted across 5+ gate files and
  fault_code has drifted (6 vs 10 arms). Hoist into selfhost_modtree/mod.rs.

### Addressed — OBL-QUALITY-REVIEW fix-now batch (Opus 4.8, 2026-07-11)

The four fix-now findings are resolved; each verified per-gate in isolation, byte-exact.

- **F-LAYOUT-DRIFT — FIXED.** `selfhost/lower/lower.cnr` `ty_size`/`ty_align` now handle the
  compiler-known collections exactly as `interp.cnr`/`src/interp/layout.rs`: `Vec`/`Map`/`String`
  = 40 (align 8), `str`/`T_STR`/slice = 16 (align 8), `Box` = 24, `BoxResult` = 32. Regression
  fixture `tests/fixtures/selfhost_interp/struct_with_vec.cnr` (`struct Holder { v: Vec[i64],
  tag: i64 }`, returns `offsetof(Holder, tag)`): the Vec field forces `field_off` to size the Vec,
  so `tag` lands at 40, matching the oracle. Pre-fix lower sized the Vec at 0 -> `tag`@0 -> RET 0
  vs oracle RET 40 (would FAIL). Added to `selfhost_lower` (86/86) and `selfhost_interp` CORPUS.
- **F-MONO-SILENT — FIXED (fail-loud).** `mono.cnr` `clone_subst` now distinguishes an unbound
  type-param (a `T_NAMED` whose name IS in the active env but bound to 0) from a plain nominal via
  a new `env_has` helper, and `panic("mono: unbound type parameter")` on the unbound case instead
  of silently emitting an unsubstituted `T_NAMED`. The 8 generic fixtures all infer, so the path is
  never hit and they stay byte-exact. (No dedicated uninferable fixture: the front-end rejects the
  obvious constructions before mono runs; the fail-loud mechanism + the 8 passing fixtures cover it.)
- **F-FIND-NEEDLE — FIXED.** `interp.cnr` `synth_boxresults`/`synth_generic_boxresult` now bound the
  `find_needle` result: when "boxed"/"oom" is absent the synthesized variant span is clamped to an
  in-bounds empty `[0,0)` span (mirroring `synth_scw`'s `pos < len(src)` guard) instead of a span
  running past the buffer. Byte-exact (e.g. `box_drop_frees.cnr` has no "oom" substring and never
  read that span anyway).
- **F-ARENA-CAP — FIXED (stopgap).** Node arena / token buffer raised 32768 -> **49152** (+50%,
  ~16k headroom over interp.cnr's ~32712 tokens). Grew every coupled site: `parser.cnr` `P.nodes`
  `[N]Node` + `lexer.cnr` `Buf.toks` `[N]Tok` array types; the `[nnew(0); N]` P-literals in
  parser/checker/analyses/interp/lower (mono allocates no P); the `[mk(..); N]` Buf literals in
  tests/selfhost_{lexer,parser,checker,analyses,effects,loans,interp,lower}.rs; and the stale
  "~56-token margin" doc comments in selfhost_checker.rs / selfhost_analyses.rs. Node is 64 bytes,
  so `[49152]Node` = 3 MiB per P frame. COUPLED CONSEQUENCE: the bigger stack arrays pushed the
  interp memory-model's simulated stack (grows up from 1 MiB) into the checker/analyses tools' own
  bump-allocator window (was [16 MiB, 32 MiB)), corrupting their symbol Maps -> spurious E0103 on
  interp.cnr's second-half fns. Fixed by moving that tool-internal window to [96 MiB, 240 MiB)
  (checker.cnr / analyses.cnr `check_dump`/entry; ~80 MiB stack headroom, 144 MiB Map space, all
  under 256 MiB MAX_ADDR; addresses never appear in the diagnostic dump, so dump-invariant). The
  checker/analyses self-checks (which lex interp.cnr into the arena) then pass without overflow.
  The real fix (module-split interp.cnr / Vec-backed arena) stays deferred
  (F-LAYOUT-EXTRACT / token-cap obligation). Capacity-only; every gate byte-exact.

### Addressed — F-LAYOUT-EXTRACT (Opus 4.8, 2026-07-11)

**F-LAYOUT-EXTRACT — DONE.** The triplication-in-waiting is retired: the layout /
type-property core now lives ONCE in the new module `selfhost/layout/layout.cnr`
and both consumers `use` it. FEASIBILITY (assessed before extracting): the layout
functions are PURE over `(read P pp, [u8] src, u32 head, ...)` — interp.cnr's
copies took `write E` but only ever READ `e.head` (via `find_struct`/`find_enum`,
which just walk the item `nx`-chain), and lower.cnr's copies already threaded a raw
`head: u32`. No genuine coupling to mutable interpreter/lowering state — layout is a
read-only computation over the arena — so the two signatures unify cleanly on
`head`. `layout.cnr` (~460 lines) holds the single source of truth for the
Vec/Map/String=40, str/slice=16, Box=24, BoxResult=32 (align 8) ABI plus
`ty_size`/`ty_align`/`struct_size`/`struct_align`/`field_off`/`field_off_lit`/
`field_ty`/enum layout (`enum_size`/`payload_size`/`variant_*`)/`struct_of_ty`/
`enum_of_ty`/`is_copy`/`needs_drop`/collection recognizers, with its own small
scalar utils (the per-module concatenation idiom). `interp.cnr` and `lower.cnr`
`use layout::{...}`, deleted their copies, and adapted call sites (interp threads
`e.head`; lower already threaded `head`, so only its divergently-named calls —
`is_copy_ty`/`struct_id_of`/`tn_is_vec`/… — were renamed to the canonical names).
HEADROOM: interp.cnr 4015 -> 3654 lines, lower.cnr 5297 -> 4948 (~360 / ~348 lines
out of each), widening the interp.cnr arena/token headroom under the [49152] cap
(F-ARENA-CAP) by the corresponding ~3k tokens — the room native N2 needs. `is_copy`
and `needs_drop` were verified behaviorally identical between the two prior copies
(interp's `scalar_width` treats pointers as word scalars; lower's handled them via
explicit tag arms — same result), so the unified bodies are output-invariant.
`layout.cnr` JOINS the self-check module tree (added to selfhost_checker.rs /
selfhost_analyses.rs interp trees exactly as mono.cnr was); interp.cnr still
self-checks CLEAN through the checker + analyses fixpoints, now resolving its
`use layout` imports. codegen (N1, scalar) is untouched — it grows no layout copy
until N2, when it too will `use layout`. Every gate byte-exact green: selfhost_interp
(28s), selfhost_lower (59s), selfhost_codegen (4s), selfhost_checker (6/6), and
selfhost_analyses (6/6) each in isolation; clippy clean.

### Addressed — OBL-QUALITY-REVIEW deferred batch (Opus 4.8, 2026-07-12)

Four of the deferred findings are resolved; each verified per-gate in isolation, byte-exact.

- **F-VECSET-ORDER — FIXED.** `src/mir/build.rs` (the `"set" if is_vec` arm) now mirrors
  `bi_vec_set`'s order: it emits a `VecGet` bounds-probe (read len, eval index, fault `Bounds`
  at the index span; slot-borrow discarded) BEFORE `materialize_place` for the value arg, then
  the `VecSet` store. So an out-of-bounds `set` faults WITHOUT running the value's side effects,
  matching the oracle. Reuses the already-serialized, fault-capable `VecGet` op (never DCE'd) —
  no new MIR surface. Fixture `tests/fixtures/selfhost_lower/vec_set_oob_fault.cnr`
  (`set(write v, 5, tapped(7))`, `tapped` traces 99, `v` has len 1) is gated through
  `mir_serial` COLLECTION_CORPUS: pre-fix the MIR ran the value first (dump `TRACE 99\nFAULT`),
  the oracle only `FAULT`; post-fix both are `FAULT 4 <index-span>` byte-exact. `mir_serial` 8/8.

- **F-DESERIALIZE-ARITY — FIXED.** `src/mir/serial.rs` adds a bounds-checked accessor
  `arg(args, i) -> Result<&Sexp, String>` and routes every positional index in the `*_from`
  deserializers through it (185 sites), so a parseable-but-truncated wire returns a descriptive
  `Err("wire truncated: need arg #N, have M")` instead of an index-OOB panic — the `Result`
  contract made honest now a Candor-emitted wire feeds `deserialize`. Well-formed wires index
  the same slots, so every existing round-trip still succeeds. Unit test
  `mir::serial::tests::truncated_wire_errs_not_panics` feeds `(mir (fns (fn "main" 0)) …)` and
  asserts a graceful `Err`.

- **F-TEST-DEDUP — FIXED.** The copy-pasted helpers are hoisted into
  `tests/selfhost_modtree/mod.rs` (`pub fn` `fault_code`, `dump_ok`, `dump_fault`,
  `on_big_stack`); the gate files `use` them and deleted their local copies (10 gates for
  `on_big_stack`; `fault_code`/`dump_ok`/`dump_fault` in lower/interp/mir_serial). The DRIFTED
  `fault_code` (6-arm, `panic!` fallback in lower/interp) is reconciled to the canonical
  10-arm total map (Overflow=0 … NoForeignRuntime=9, matching the `FaultKind` enum + interp
  kind codes); the subset gates never hit the extra arms, so byte-exactness holds. Per-gate
  CORPUS lists stay gate-specific (not shared). Every gate compiled warning-clean and stayed
  byte-exact green.

- **F-FAULT-TRACE — FIXED (implemented, not scoped).** Assessed as a clean, contained addition:
  every fault propagates to a single run boundary in each engine with no internal catch, and
  only `Fault::new` constructs the struct literal. `Fault` gains `pub trace: Vec<i64>`
  (`src/interp/mod.rs`); the tree-walker (`eval.rs` `run_main` via a new `attach_trace`) and the
  MIR interp (`mir/interp.rs` `run` boundary) thread the accumulated pre-fault trace into the
  escaping fault. The shared `dump_fault` now renders those `TRACE` lines before `FAULT`, so the
  differential harness compares the pre-fault trace, not just kind+span. Fixture
  `tests/fixtures/selfhost_interp/trace_then_fault.cnr` (traces 7, 11, THEN overflows) added to
  `mir_serial` CORPUS: both engines emit `TRACE 7\nTRACE 11\nFAULT 0 …` byte-exact, proving the
  pre-fault trace is now compared (empty for every prior fault fixture, so all stay green). The
  `Fault`-shape harness assertions relaxed from `starts_with("FAULT ")` to `contains("FAULT ")`.

The remaining deferred findings (F-LAYOUT-EXTRACT [done above], F-FOUR-ENGINE-CLAIM) are
unaddressed by this batch.

## T1 — trait-generics front-end: `interface`/`impl` parsing in the self-host parser (2026-07-11)

The FIRST slice of the trait-generics arc, closing the FRONT-END gap flagged by OBL-G1 /
OBL-L-GEN (the 5 deferred fixtures `iface`/`gimpl`/`gbound`/`fromq`/`gfromq` failed only
because the self-host PARSER did not parse `interface`/`impl` items). This slice adds that
parsing to `selfhost/parser/parser.cnr` SOLO; it lands nothing runnable alone — mono +
interp + lower dispatch of impl methods / `?` / `From` stays DEFERRED to the T2+ slices —
but it unblocks them.

TOKENIZATION: `interface`, `impl`, and `for` are CONTEXTUAL identifiers (kind-1 tokens),
not hard keywords — exactly as the Rust reference lexer treats them (`src/real/token.rs`
has no such keywords; `parser.rs` dispatches on `at_ident("interface"/"impl"/"for")`). The
self-host lexer is UNCHANGED, so the lexer gate's byte-exact token stream is preserved; the
new item forms are recognized by the parser's existing `ids(src, t, b"...")` contextual
check, mirroring how `use`/`pub` already work.

NODE DESIGN (arena tags): `T_INTERFACE` (6), `T_IMPL` (7), `T_METHODSIG` (108).
- `T_INTERFACE`: name span in p0/p1, `a` = type-param chain (free via `parse_decl_brackets`),
  `b` = `T_METHODSIG` chain.
- `T_METHODSIG`: name span, `op` = self-receiver mode (0 take / 1 read / 2 write, −1 = no
  receiver → associated fn like `From::from`), `suf` = alloc flag, `a` = non-self params,
  `e` = return type.
- `T_IMPL`: iface-name span, `a` = type-params, `b` = iface type-arg chain, `c` = target
  type, `d` = impl-method (`T_FN`) chain. Each impl method is a normal `T_FN` with a
  SYNTHETIC `self` param prepended (a `Self`-placeholder `T_NAMED` type), mirroring the
  reference's `parse_impl_method` (parser.rs:387-396).
New parse fns: `parse_self_mode`, `parse_method_params`, `parse_method_sig`,
`parse_interface`, `parse_impl_method`, `parse_impl`; `parse_item` dispatches
`interface`/`impl` before the struct/enum/fn/static match. The self-receiver mode is carried
in LOCALS (not a new `P` field) deliberately: `P` is imported by checker.cnr/analyses.cnr, so
a new field would have broken their `P{}` literals — the change stays contained to parser.cnr.
Associated-type members (`type Item;` / `type Item = T;`) are NOT parsed (no fixture needs
them this slice); deferred.

DUMP: the AST-dump `emit_node` gains NO cases for the new tags — the parser gate's corpus
has no interface/impl fixtures, so those nodes are never traversed and the dump stays
byte-exact; adding emit cases would be speculative (the oracle renderer has no canonical form
for out-of-subset items to match against).

SMOKE GATE (`tests/selfhost_traitparse.rs`): a new `pub fn parse_count(src, buf) -> usize`
(non-dumping entry point) lets the harness lex+parse each of the 5 trait fixtures through the
self-host lexer+parser and return the arena node count; the gate asserts a completed run (no
fault / parse-error / check-error) with a non-trivial count, proving the interface/impl nodes
land in the arena.

VERIFICATION (all in isolation): `selfhost_parser` (AST-dump byte-exact) green — existing
fixtures unaffected; `selfhost_checker` green including
`candor_checker_checks_parser_source_clean_via_import_resolution` (parser.cnr self-checks
name-res-clean under the self-host checker); `selfhost_analyses` green including
`candor_analyses_check_parser_source_clean_fixpoint` (move/init-clean); the new
`selfhost_traitparse` smoke gate green (all 5 fixtures parse); clippy clean.

DEFERRED to later slices (unchanged): mono impl-method mangling/dispatch, interp method
dispatch + `?`/`From` runtime, and the MIR lowering of impl methods (T2/T3+). This slice is
purely the parser front-end that those slices build on.

## T2 — trait-generics: impl-method monomorphization + interp method dispatch (2026-07-11)

The tall-pole slice: teach `selfhost/mono/mono.cnr` to emit/instantiate impl methods and
`selfhost/interp/interp.cnr` to dispatch `recv.m(args)` method calls, landing `iface`,
`gimpl`, and `gbound` RUNNING on the interp tier (RET 42 / 40 / 105, byte-exact to
`run_source_real`). Mirrors the Rust oracle: dispatch resolves at EVAL time from the
receiver's static nominal (`src/interp/eval.rs:1571-1601`), and impl methods are emitted as
concrete fns by `monomorphize` (`src/generics.rs`, `emit_impl`/`emit_impl_instance`).

MONO (impl-method emission + generic-impl instantiation):
- `fixup_concrete_impls` walks non-generic `T_IMPL` nodes and, per method, retargets the
  synthetic `self` param — the parser spans its placeholder type on the self-MODE keyword
  (`read`/`write`/…), so `fix_impl_selves` sets the param's type to the impl target and
  retargets its NAME span to `self` bytes (the interp binds locals by byte content, so `self`
  in the body then resolves).
- Generic impls (`impl[T] … for Wrap[T]`) instantiate at each reached concrete target:
  `emit_struct_inst` snapshots the struct's concrete type-args (into `M.sarg`) and, after
  emitting the instance `sid`, calls `instantiate_impls_for_struct`. That finds each generic
  `T_IMPL` whose App target head matches, recovers the impl's type-param bindings by matching
  its target App args against the captured args, `clone_subst`s the target + method chain
  (reusing the existing subst/dedup infra), stashes the concrete struct instance on the
  cloned target, fixes `self`, and appends a CONCRETE (`a == 0`) `T_IMPL` to the item chain.
  The `Self` placeholder resolves to the impl's target type. Existing generic fixtures are
  untouched (no `T_IMPL` → both passes are no-ops).

INTERP (dispatch table + T_FIELD-callee path):
- `build_dispatch` (startup, after mono) walks concrete `T_IMPL` nodes and records
  `(target struct-instance node id, method-name span) -> method fn node id`. Keying by NODE
  ID (not a mangled name — synthesized names are unaddressable in the self-host interp)
  disambiguates instantiations; generic templates (`a != 0`) are skipped.
- `eval_call` routes a `T_FIELD` callee to `eval_method_call`: `eval_place(base)` +
  `peel_box_place` type the receiver to a struct instance (`struct_of_ty`), `dispatch_lookup`
  finds the method fn, and the receiver is bound as `self` — a `read`/`write` self is a
  borrow (post-`synth_param_borrows`), so the receiver ADDRESS is passed as an 8-byte pointer
  scalar exactly as `call_fn` expects a borrow param. NESTED dispatch (`gbound`'s
  `self.inner.show()`) falls out for free: `eval_place` types `self.inner` to `Cell` after
  T=Cell substitution, and `show` dispatches on `Cell`.

VERIFICATION (all in isolation): `selfhost_interp` green — the 8 prior generic fixtures + all
S1–S6 fixtures + `iface`/`gimpl`/`gbound` byte-exact to the oracle; `selfhost_checker` +
`selfhost_analyses` green (mono.cnr/interp.cnr self-check clean via import resolution);
`selfhost_lower` + `mir_serial` green; clippy clean. interp.cnr stays comfortably under the
49152 node-arena cap.

DEFERRED (unchanged): MIR lowering of impl methods (T3), and `?`/`From` runtime (T4). No
bound checking in interp/mono — `[T: Weighable]`/`[T: Show]` is enforced only by the Rust
oracle; after instantiation the method call resolves by the concrete nominal.

## T5 — trait-generics, FINAL slice: the `?` operator + `From` widening lowered to a MIR ok/err CFG — the trait-generics arc is COMPLETE (2026-07-11)

The last slice of the trait-generics arc: teach the self-hosted lowering
(`selfhost/lower/lower.cnr`) to lower `expr?` (`T_TRY`) + cross-type
`From` widening to a MIR control-flow graph, landing `fromq` and `gfromq`
COMPILING on the lower tier (both RET 7, byte-exact to `run_source_real`). Mirrors
`selfhost/interp/interp.cnr`'s T4 `eval_try`/`eval_try_from`/`find_from_fn` and the
Rust oracle `src/mir/build.rs` `lower_try`/`lower_try_from`, emitting MIR instead of
walking.

LOWERING (`lower_try` in the `lower_value` dispatch, gated on `T_TRY`):
- Materialize the operand call into a fresh enum temp (reusing `lower_into` — the
  existing agg-call return-slot ABI), read the tag at field 0, `cmp eq` against the
  ok-variant index (`T_VARIANT.op == 1`), and `Branch` to an ok block / err block
  with a join block, exactly as `build.rs` orders them.
- OK arm: load the (word-sized) ok payload as the `?` expression's value and `Goto`
  the join; the enclosing `let` store then runs in the join block. Aggregate ok
  payloads stay out of subset (`build.rs` `unsupported`).
- ERR arm: `lower_try_from` resolves the `From` impl from the concrete `T_IMPL`
  blocks — `find_from_impl` keys (iface name `From`, target enum == the return
  error payload e2, iface-arg-0 enum == the operand error payload e1), mirroring
  interp's `find_from_fn` — then passes e1 to `from` (by value if word-sized, else
  by address via `(ref …)`), calls it through the return-slot ABI, and builds the
  return enum in `_0` (tag = the return error variant, payload = the widened e2
  copied from the call result). `emit_return_drops` + `Return` early-return the
  widened error (INV-DROP). Same-error-type `?` (`reid == eid`) is a plain
  `copyval` of the whole operand into `_0` + return, no widening.
- The gate compares EXECUTION (RET/TRACE/FAULT via deserialize -> `mir::interp`),
  not wire text, so non-faulting statement spans are free; the From resolution uses
  the same `enum_id_of` layout view as interp, and `gfromq`'s generic `From` impl is
  already instantiated concretely by `mono.cnr` (interp T4 proved it), so the lower
  side only emits the `?`/From CFG.

VERIFICATION (all in isolation): `selfhost_lower` green — `fromq`/`gfromq` added to
the CORPUS, both byte-exact, and ALL prior lower fixtures (the 8 user-generic +
`iface`/`gimpl`/`gbound` + the systems + std corpora) UNCHANGED byte-exact;
`mir_serial` green (unaffected — it exercises the Rust `build.rs` lowering); whole
suite 591 passed / 0 failed; clippy clean. No edits to `interp.cnr` / `mono.cnr` /
`src/mir/*` / the self-host parser — the change is contained to `lower.cnr` (+ its
CORPUS). Runtime: `selfhost_lower` ~176s, `mir_serial` ~2s.

THE TRAIT-GENERICS ARC IS COMPLETE: all 13 generic fixtures — the 8 user-generic
(`mono3`, `mixed`, `nameval`, `pair`, `genenum`, `arena`, `gdrop_groundfloor`,
`gdrop`) + `iface`/`gimpl`/`gbound` (interface/impl method dispatch) +
`fromq`/`gfromq` (`?`/`From`) — now RUN on the interp tier AND COMPILE on the lower
tier, byte-exact to the Rust oracle on both.

## N1 — the native self-compile tier's FIRST slice: a Candor code generator that emits x86-64 assembly TEXT for the scalar subset, assembled+linked+run byte-exact vs the oracle (2026-07-11)

The first slice of the native self-compile tier (the true bootstrap): a new
Candor module `selfhost/codegen/codegen.cnr` walks the parser's AST
arena — mirroring `selfhost/lower/lower.cnr`'s scalar-subset walk and its
`cur`-span / fault-edge threading — and EMITS x86-64 assembly TEXT (AT&T syntax)
through the same `trace` byte sink `lower_dump` uses for its MIR wire. The system
`cc` assembles+links the reconstructed `.s` with the UNCHANGED C runtime
(`src/backend/aot_runtime.c`) into a REAL ELF process whose observable outcome —
θ (stdout trace), the exit byte, and the `(kind, span)` fault JSON on stderr — is
asserted byte-exact to the tree-walking oracle (`run_source_real`). Gated by
`compiler/tests/selfhost_codegen.rs`.

SUBSET (same as lower.cnr's L1): integer (i8..i64/u8..u64/isize/usize) + bool
scalars; `let`/`let mut` + assignment; `if`/`else if`/`else`; `while`; `loop` +
`break`/`continue`; `return`; arithmetic `+ - * / %` in the Checked/Wrapping/
Saturating regimes with the Overflow/DivByZero fault edges; comparisons; `&&`/`||`
short-circuit; bitwise/shift; unary neg/not/bitnot; `trace(x)` (-> `rt_trace`);
`assert`/`panic` (-> `rt_fault` Assert/Panic); direct non-generic scalar fn calls +
recursion; `conv` (with the ConvLoss edge). Structs/arrays/enums/box/pointers/drop
stay out of subset (later slices).

MODEL (`codegen.cnr`):
- AST WALK, not a MIR parser: `gen_value` dispatches on node tag exactly like
  `lower_value`; `gen_if`/`gen_while`/`gen_loop` mirror the CFG shape and keep a
  `(continue_lbl, break_lbl)` loop stack; width inference reuses lower.cnr's
  `lit_width`/`concretize`/`scalar_width`.
- ALL-ON-STACK, NO register allocator: each local + each temp gets an 8-byte stack
  slot holding the CANONICAL i64 value (sign/zero-extended per scalar type, as
  `load_scalar` produces). Values evaluate into `%rax`, spilling to / reloading
  from slots; every op canonicalizes its result (`movsbq`/`movzbq`/… into width).
- SysV frame per fn: `pushq %rbp; movq %rsp,%rbp; subq $frame,%rsp`. `%rsp` never
  moves after the prologue (temps are `%rbp`-relative slots, not pushes), so every
  `call` sees a 16-byte-aligned `%rsp` — the classic alignment bug is structurally
  absent. The frame size is known only after the body is walked, so the body is
  buffered in a byte pool and flushed after the streamed prologue. `main` compiles
  to `cnr_main`; a `.globl candor_entry` alias tail-`jmp`s to it (the runtime's
  entry, same 0-arg `-> i64` ABI).
- FAULT edges (INV-CHECK): after each checked op an explicit test + `jCC` skips a
  fault stub that does `call rt_fault(kind, span.start, span.end)` — signed
  overflow via `jo`, u64 carry via `jc`, u64 mul via `mulq`+`jc`, narrow widths via
  an explicit `[min,max]` range compare, `div/rem` guarding divisor==0 (and signed
  type-MIN/-1). The rt_fault kind codes are the BACKEND codes
  (`src/backend/lower.rs::kind_code` == `aot_runtime.c::kind_name`: overflow=0,
  div_by_zero=1, conv_loss=3, assert=4, panic=7) — NOT lower.cnr's wire codes. The
  spans are the same node spans lower.cnr/the oracle use (binary-expr span for
  arith/shift; operand span for neg/conv; comparison-RHS span for assert;
  `panic(...)` span for panic — confirmed against `run_source_real`).

GATE (`tests/selfhost_codegen.rs`, cloning tests/aot.rs + tests/selfhost_lower.rs):
per scalar fixture — run codegen.cnr in the tree-walker over the embedded source
(module tree lexer+parser+codegen + a generated `main` that lexes then calls
`codegen_dump`), reconstruct the `.s` text via the `trace` byte sink, then
`cc -no-pie prog.s src/backend/aot_runtime.c -pthread -o prog`, run it, and compare
(exit byte, stdout trace, stderr fault-JSON) to the oracle. Fault vs plain-exit-2
is disambiguated by the presence of the `"kind"` JSON on stderr (a program that
returns 2 also exits 2). FAILS LOUDLY if `cc` is unavailable.

VERIFICATION (all in isolation): `selfhost_codegen` green — 29 scalar programs
(18 OK: let/trace/return, fib recursion, while accumulate, loop/break/continue,
factorial, short-circuit incl. the div-by-zero-guarded `||`, all six comparisons,
bitwise+shift, unary neg/not, assert-pass, multi-trace, i8-width add, u64-max
trace; 11 faults: i32 overflow, div-by-zero, conv-loss, assert, panic, u8 shift
overflow, i8/u8 width overflow, u64 add/sub overflow, i64 mul overflow) codegen ->
assemble -> link -> run byte-exact to `run_source_real`. Existing suites
UNCHANGED: `aot` 6/6 and `selfhost_lower` 1/1 green; whole suite 592 passed / 0
failed (591 baseline + this gate); clippy clean. NO edits to
`interp.cnr`/`lower.cnr`/`mono.cnr`/`src/backend/*` — the change is contained to the
new `codegen.cnr` + its gate. Runtime: `selfhost_codegen` ~19s.

DEFERRED (later native slices): a register allocator (the all-on-stack model is the
MVP), and structs/arrays/enums/box/pointers/drop/concurrency codegen (N2–N5).

## N2 — the native self-compile tier's SECOND slice: flat aggregates (structs + arrays) codegen, byte-exact vs the oracle (2026-07-12)

The second native slice extends `selfhost/codegen/codegen.cnr` to emit
x86-64 asm for FLAT (copy) STRUCTS and ARRAYS, matching the tree-walking oracle
(`run_source_real`) byte-exact. Gated by `compiler/tests/selfhost_codegen.rs`
(the new `..._over_aggregate_subset` test); the N1 scalar gate stays green.

MEMORY MODEL SHIFT (mirroring `src/backend/lower.rs`): N1's scalars stayed in
native `%rbp` stack slots (no address is observed). AGGREGATES instead live in the
FLAT MEMORY region: an aggregate local/temp is `rt_stack_alloc(size, align)`'d and
its VALUE flows as a Candor OFFSET (`%rax`), exactly as an aggregate MIR place does.
A scalar leaf at a place is loaded/stored at `MEM_BASE + offset` — a
`movabsq $0x200000000000` folded into a scratch reg, then a sized mov through it,
the asm twin of `lower.rs::host_addr`. An aggregate local keeps BOTH a `%rbp` slot
(holding its 8-byte offset, spilled/reloaded as a scalar) and its flat storage.

CONSTRUCTS:
- Struct literal: `emit_alloc` then write each DECL-ORDER field at `field_off`
  (scalar -> sized store; aggregate field -> `rt_copy`). Field read `s.f` resolves
  a flat offset (`gen_place_addr`, base offset + `field_off`) then a sized load;
  field assign stores there. Nested structs recurse.
- Arrays: `[a,b,...]` and `[e;N]` (unrolled at codegen time; count is the type's
  static `array_len`). Index read/assign resolve `base + i*stride`; the Bounds
  fault (`i >= len`) emits `cmpq $n,%rax; jb ok; call rt_fault(2 /*bounds*/, s, e)`.
- Struct by-value fn param/return: an aggregate ARG is passed as its offset
  (pointer); the callee `rt_stack_alloc`s its own local and `rt_copy`s the bytes in
  (by-value, mirroring `lower.rs`'s aggregate-param prologue). An aggregate RETURN
  leaves the result's offset in `%rax`; the caller `rt_copy`s from it. All copies
  are byte copies (`rt_copy`) — sound because N2 is COPY aggregates (no drop yet).

BOUNDS SPAN (byte-exact): the oracle's `eval_place` evaluates the index, then the
BASE place, then bounds-checks with `cur_span` — so the reported span is the BASE
ROOT's span, NOT the index's (confirmed via `run_source_real`: `a[3]` faults at
`a`'s span `61..62`, not `3`). `gen_place_addr` mirrors the eval order, so the
`cur_p0/cur_p1` left by the base resolution IS the fault span.

LAYOUT REUSE: codegen `use`s `layout::{ty_size, ty_align, struct_size,
struct_align, field_off, field_ty, struct_of_ty, array_len_of}` for ALL
sizes/offsets — no layout logic is copied into codegen.cnr. The one derived value
is the array element STRIDE = `ty_size([N]T) / array_len` (layout exposes no
`stride_of`; computed here from exported sizes — see GAP).

GATE / VERIFICATION (isolation): `selfhost_codegen` green — N2: 12 flat-aggregate
programs (11 OK: struct_field, nested_struct, field_assign, struct_param_ret,
struct_mixed_width, array_index, array_repeat, index_assign, array_of_structs,
struct_with_array, aggregate_mixed; + 1 Bounds fault: array_bounds) codegen ->
assemble -> link -> run byte-exact to `run_source_real`; N1: 29 scalar programs
stay byte-exact. The gate's module tree now also materializes `layout.cnr`.
clippy clean. NO edits to `interp.cnr`/`lower.cnr`/`mono.cnr`/`layout.cnr`/
`src/backend/*` — contained to `codegen.cnr` + its gate.

GAP (reported, not worked around): `layout.cnr` exposes no `stride_of(ty)` =
`round_up(ty_size, ty_align)`; codegen derives the array stride as
`ty_size([N]T)/array_len` from exported functions instead of duplicating the
round-up. A future `pub fn stride_of` in layout.cnr would let index/array codegen
(and any element-stride consumer) call it directly.

DEFERRED (later native slices): enums/box/pointers, DROP glue for non-copy
aggregates, slices/collections, and a register allocator.

## N3 — the native self-compile tier's THIRD slice: the MOVE/DROP SCHEDULE with trace-on-drop, byte-exact vs the oracle (2026-07-12)

The third native slice extends `selfhost/codegen/codegen.cnr` to emit
the DROP SCHEDULE for non-copy aggregates as x86-64 asm, matching the tree-walking
oracle (`run_source_real`) byte-exact. It MIRRORS `selfhost/lower/lower.cnr`'s L3
ownership/schedule logic (proven on the same fixtures) but emits a `call` to a
drop-hook asm fn instead of a MIR `Drop` op. Gated by the new
`..._over_drop_subset` test in `compiler/tests/selfhost_codegen.rs`; the N1
scalar and N2 aggregate gates stay green.

MOVE MASK (mirrors lower.cnr `collect_path`/`mark_moved`/`whole_moved`): a
per-function side-table of moved (sub-)paths. A non-copy value is marked moved at
let-init-from-a-place, a by-value aggregate arg, and a return operand
(`is_copy` from layout.cnr gates it). Paths are (owned-local id, one-level field
segment); `whole_moved` (no segments) suppresses the whole local's drop,
`field_moved` (one segment) suppresses just that field in the struct recursion —
matching the one-level partial move L3 supports.

DROP EMISSION (mirrors lower.cnr `scope_pop_drops`/`emit_return_drops`/
`emit_loop_exit_drops`): owned needs-drop locals are registered per drop-scope in
declaration order (their `%rbp` slot holds the value's flat offset). Drops fire at
scope exit, `return` (all live scopes), and `break`/`continue` (down to the loop's
captured scope depth), each in REVERSE declaration order, pruned by the move mask.
`emit_drop_at` runs a struct's drop hook FIRST (whole value) then drops its
needs-drop fields in reverse (skipping moved ones), recursing for nested structs.
ABORT-NO-DROP is automatic: a fault `call`s `rt_fault` (which exits) inline, so the
scope-exit drop asm emitted after the body is simply never reached on the fault
edge. `return` spills the return value (%rax), runs the return-drops (which clobber
%rax via hook calls), then reloads it — the returned value itself is move-suppressed.

DROP HOOKS AS ASM FNS: each `struct S { .. } drop(write self) { .. }` is emitted as
a normal asm fn `cnr_drophk_S` reusing the N1/N2 body codegen (the hook body is a
plain block that `trace`s `self.field`). `self` is a BORROW: its slot holds the
flat OFFSET passed in %rdi (NO by-value copy), so `self.field` reads the caller's
value in place. Dropping a value `call`s `cnr_drophk_S` with the value's flat
offset (its self address) in %rdi.

GATE / VERIFICATION (isolation): `selfhost_codegen` green — N3: 8 drop-schedule
programs codegen -> assemble -> link -> run byte-exact to `run_source_real`, with
the trace-on-drop ORDER load-bearing: drop_single ([7]), drop_scope_order
(reverse [3,2,1]), drop_move_suppress (moved local pruned, [5]), drop_partial_move
(one-level field move, [1,2]), drop_move_return (returned value pruned, [42]),
drop_break (loop-exit drop, [100,2]), drop_nested (hook-first-then-fields-reverse,
[2,1]), drop_param (owned aggregate param dropped at scope exit, exit 9 trace [3]).
N1 (29 scalar) + N2 (12 aggregate) stay byte-exact. clippy clean. NO edits to
`interp.cnr`/`lower.cnr`/`mono.cnr`/`layout.cnr`/`src/backend/*` — contained to
`codegen.cnr` + its gate. `aot_runtime.c` reused UNCHANGED (drop needs no new
runtime helper; the hook call is a plain SysV `call`).

DEFERRED (later native slices): drop for arrays-of-drop / enums / box / collections
(no N3 fixture exercises them), reassignment-drop of an owned aggregate (the move
mask reassign path is unexercised by the corpus), and a register allocator.

## N4 — the native self-compile tier's FOURTH slice: ENUMS + MATCH codegen (tag-switch jump chain + tag-directed enum drop), byte-exact vs the oracle (2026-07-12)

The fourth native slice extends `selfhost/codegen/codegen.cnr` to emit
ENUM CONSTRUCTION and MATCH as x86-64 asm, matching the tree-walking oracle
(`run_source_real`) byte-exact. It MIRRORS `selfhost/lower/lower.cnr`'s L4
(`lower_enum_ctor`/`lower_match_full`/`lower_match_arm`) but lowers directly to
machine instructions. Gated by the new `..._over_enum_subset` test in
`compiler/tests/selfhost_codegen.rs`; the N1 scalar, N2 aggregate, and N3 drop
gates stay green.

ENUM CONSTRUCTION (`gen_enum_ctor`, mirrors `lower_enum_ctor`): `T_ENUMCTOR`
resolves the enum by `enum_of_ty` and the variant index by name, `rt_stack_alloc`s
`enum_size` bytes (align 8), writes the u64 tag @0 (the variant's declared index),
then stores each scalar payload / byte-copies each aggregate payload at its
`variant_payload_off`. It leaves the enum's flat offset in %rax (the N2 aggregate
convention), so let/return/arg copy the full enum size (padded tail zeroed by
`rt_stack_alloc`). All enum layout comes from `layout.cnr` (`enum_size`,
`variant_by_index`, `variant_index_by_name`, `variant_payload_off`,
`variant_payload_ty`, `enum_of_ty`) — no layout logic copied into codegen.cnr.

MATCH (`gen_match`/`gen_match_arm`, mirrors `lower_match_full`/`lower_match_arm`):
resolve the scrutinee place to its flat offset, read the tag @0 once, then a per-arm
test chain — a `T_PVARIANT` arm `cmpq $idx`s the tag and `jne`s past its arm block
(first-match, source order); a `T_WILD`/`T_BIND` arm is the unconditional tail. Each
arm jumps to the shared join; a non-exhaustive tail faults (Panic). Payload
sub-patterns bind in a fresh drop-scope: a scalar copies via `emit_load_flat` at
`scrut+off`; an aggregate payload byte-copies (`rt_copy`) into a fresh flat-memory
local. `T_WILD` binds nothing; nested variant sub-patterns are out of subset (the
oracle faults on them too).

CONSUMING-MATCH + TAG-DIRECTED ENUM DROP (the N3 interaction): a bound NON-COPY
payload additionally marks the scrutinee's `_i` payload path moved (a new synthetic
index segment `mv_seg_syn` in the move mask, the dual of N3's name-keyed
`field_moved` — `payload_moved`) and registers the bound local for drop, so it drops
at arm-scope exit and the scrutinee's own drop prunes that field. `emit_drop_at`
gains an enum branch (`emit_enum_drop`): read the tag @0, then a runtime switch that,
for the active variant, drops its needs-drop payload fields in REVERSE (skipping
`payload_moved` fields), recursing through `emit_drop_at`. Enums carry no drop hook,
so this is the whole of enum drop — the asm dual of interp `drop_variant_rev`.

GATE / VERIFICATION (isolation): `selfhost_codegen` green — N4: 6 enum/match
programs codegen -> assemble -> link -> run byte-exact to `run_source_real`:
enum_construct_match (construct + tag-switch + scalar payload bind, RET 5),
match_wildcard (default-arm tail, RET 2), enum_multi_variant (mixed payload
shapes/offsets `two(i16,i64)`, RET 100), match_bind_multi (3 scalar binds, trace
[10,20,30]), enum_result_shape (enum by-value return from a fn, RET 35), and — the
load-bearing drop-order case — enum_drop_payload (trace [1,7,2,8]: e1's `a(n)` arm
moves the Noisy payload into `n` which drops at arm-scope exit [7], the consumed
field pruned from e1's drop; e2's `_` arm leaves the payload, dropped tag-directed at
return [8]). N1 (29 scalar) + N2 (12 aggregate) + N3 (8 drop) stay byte-exact.
clippy clean. NO edits to `interp.cnr`/`lower.cnr`/`mono.cnr`/`layout.cnr`/
`src/backend/*` — contained to `codegen.cnr` + its gate. `aot_runtime.c` reused
UNCHANGED (enum construction/match/drop need no new runtime helper).

DEFERRED (later native slices): value-producing match into a result slot and
whole-scrutinee `T_BIND` arms (no N4 fixture exercises them; `gen_match` leaves the
arm result in %rax and, like lower.cnr, does not bind a whole-scrutinee name),
materialized (call/ctor) scrutinees, and BoxResult/`?` (N5).

## N5 — the native self-compile tier's FIFTH slice: the BOX/ALLOCATOR ABI + the rawptr/fnptr surface + statics, byte-exact vs the oracle (2026-07-12)

The fifth native slice extends `selfhost/codegen/codegen.cnr` to emit
x86-64 asm for the whole Box/alloc/rawptr/fnptr surface, reproducing the
tree-walking oracle (`run_source_real`) byte-exact. Gated by the new
`..._over_box_subset` test in `compiler/tests/selfhost_codegen.rs`; N1-N4 stay
green. This is the last infrastructure slice before the systems-corpus native
milestone (N6).

ADDRESS MODEL (mirroring `src/backend/lower.rs` + the interp oracle): a rawptr is
an 8-byte scalar whose value is the CANDOR FLAT OFFSET (not a host address); a
fn-ptr is the callee's code address. `scalar_width` now returns `W_USIZE` for
`rawptr`/`fnptr`/`borrow`, so pointers flow as word scalars. Through-pointer
access computes `MEM_BASE+offset` and loads/stores inline (the same addressing as
the N2 flat aggregates) — observably identical to the reference's `rt_mmio_*`
barrier calls, so no new runtime helper is needed. `addr_of`/`addr_of_mut` return
a place's flat offset; `ptr_read`/`ptr_write` load/store the pointee (scalar =
load/store, aggregate = `rt_copy`); `cast_ptr`/`addr_to_ptr` reinterpret;
`ptr_null`/`is_null` are 0 / (v==0); `ptr_offset` is base + n*size_of(inner)
(wrapping); `ptr_to_addr` is identity; `offsetof` is a layout-derived field-offset
const. A SCALAR local whose address is taken is FLAT-ALLOCATED (its slot holds the
offset), so a through-pointer write aliases its real storage — detected by an
arena scan for `addr_of`/`addr_of_mut` targets.

BOX / BOXRESULT (24 / 32 bytes, reusing `layout.cnr`): the synthetic
`{ boxed(Box T), oom }` enum is appended per `BoxResult T` annotation (mirroring
interp `synth_boxresults`), so `enum_of_ty` routes a `T_BOXRESULT` to it and the
N4 enum/match machinery is reused unchanged. `box(al, v)` reads `ctx`/`vt` from the
Alloc handle (name-agnostic `field_off_lit`), INDIRECT-CALLs the vtable `alloc`
fn-ptr (`call *reg`) for [size,align]; null -> `BoxResult::oom` (drop+consume v),
else move v into the pointee + build `Box {ptr,ctx,vt}` inside `boxed`. `unbox`
moves the pointee out, indirect-calls `free`, and consumes the box. `.*` /
field-access AUTO-DEREF through a Box (load `ptr@0`, re-base). The Alloc handle /
vtable are STRUCTURALLY identified (alloc+free fn-ptr fields / a `vt` rawptr),
never by name.

STATICS + FN-PTR VALUES + INDIRECT CALLS: each `static` gets a `.data` slot
(`cnr_stat<i>`) holding its flat offset; `cnr_static_init` (called from
`candor_entry` before `cnr_main`) rt_stack_allocs the storage and runs the
initializer into it. A fn NAME used as a value lowers to its `cnr_<name>` code
address (`leaq ... (%rip)`); an indirect call `fp(args)` loads the fn-ptr and
`call *%rax` with SysV arg regs.

ALLOC-ON-DROP: `emit_drop_at` routes a `T_BOX` to `drop_box` (if `ptr!=0`, drop
the pointee recursively THEN indirect-call `free` — pointee-then-free order); a
`T_BOXRESULT` routes through the N4 tag-directed enum drop to its `boxed` Box.
Nested Boxes free bottom-up.

GATE / VERIFICATION (isolation): `selfhost_codegen` green — N5: 16
box/rawptr/fnptr/static programs (static_fnptr_indirect_call, ptr_roundtrip,
cast_ptr_read, alloc_abi, box_unbox_scalar, box_struct, unbox_path, boxresult_oom,
box_drop_frees, nested_box, high_addr_roundtrip, offsetof_first_field,
offsetof_nonzero_field, ptr_offset_stride, enum_padding_copy, page_boundary)
codegen -> assemble -> link -> run byte-exact to `run_source_real`. N1 (29 scalar)
+ N2 (12 aggregate) + N3 (8 drop) + N4 (6 enum) stay byte-exact. clippy clean. The
`aot_runtime.c` mmap region (`MAX_ADDR` = 256 MiB) covers the high-address
fixtures (3 MiB) with room for the systems corpus (~16.9 MiB); NO runtime change.
NO edits to `interp.cnr`/`lower.cnr`/`mono.cnr`/`layout.cnr`/`src/backend/*` —
contained to `codegen.cnr` + its gate.

DEFERRED: value-producing match into a result slot and non-plain-local (call/ctor)
scrutinees — NO N5 fixture exercises them (every box/enum match is
statement-position with an explicit-`return`/assign arm over a plain-local
scrutinee), so per the no-speculative-generality rule they are NOT added here.

## N6 — THE MILESTONE: Candor native-compiles all five systems-corpus programs to real x86-64 executables, byte-exact vs the oracle (no Rust in the compile path) (2026-07-12)

The true-bootstrap milestone: `selfhost/codegen/codegen.cnr` (composed
with the Candor `lexer`+`parser`+`layout` modules, run in the tree-walker) now
emits x86-64 asm for the FULL systems corpus, and each program `codegen -> cc +
aot_runtime.c -> run` reproduces `run_source_real` byte-exact (exit / θ trace /
fault JSON). Gated by five `n6_11_*` tests in
`compiler/tests/selfhost_codegen.rs`. The five programs — an allocator/pool
(`11_1`, RET 1234), a scheduler intrusive list (`11_2`, 42), MMIO device state
(`11_3`, 42), a recursive-descent parser over a byte-string (`11_4`, 17), and a
`Box [4096]Node` arena (`11_5`, 5) — are the SAME corpus the interp (S6b) and
lower (L6) arcs run; the reference native backend (`src/backend/lower.rs`)
compiles them too. NOTHING beyond asm codegen was needed.

FEATURES / BUGS the real programs surfaced (each a real codegen fix mirroring
`lower.cnr` L6 + `backend/lower.rs`, not a fixture special-case):
- BORROW PARAMS by address (`11_1/2/3/4/5`): a `read`/`write T` param carries its
  mode in `T_PARAM.op` with `.a` = the *bare* pointee type, so codegen was copying
  it BY VALUE (16-byte Uart copy) — the borrow-param-by-value bug. `synth_borrow_params`
  now wraps such a param's type in a synthetic `T_BORROW`/`T_BORROWMUT`, so it flows
  as a word-sized pointer (the pointee's flat offset). `peel_box_addr` / `PF_DEREF`
  no longer over-dereference a borrow (only a Box loads `ptr@0`; a borrow's value IS
  the pointee address).
- VALUE-PRODUCING MATCH (`11_3` `let s: State = match ev {...}`): `gen_match` split
  into `gen_match_core` + a value form (`gen_match_value`) that materializes each
  arm's aggregate result into a fresh flat temp.
- NON-PLACE (materialized) SCRUTINEE (`11_1/4/5` `match box(...)` / `match parse_expr(...)`):
  a `prep_match_synth` pre-pass records each non-place scrutinee's type (a synthetic
  `BoxResult` for `box(...)`; the callee's declared return type for a call — a borrow
  return like `arena_get -> read Node` held as the pointee address), and
  `gen_match_scrut` materializes owned values / uses the borrow address in place.
- BYTE-STRINGS + SLICES (`11_4`): a `[u8]` is a `{ptr@0,len@8}` fat pointer; `gen_bytes`
  decodes `b"..."` into a fresh flat blob and builds the header; slice indexing bounds-
  checks against `len@8` and offsets through `ptr@0`; `len(slice)` loads `len@8`.
- RECURSIVE-TYPE DROP GLUE (`11_4` `enum Expr { add(Box Expr, Box Expr) ...}`): inline
  drop expansion recurses forever on a self-referential enum. Enum drops with no
  partial-move info now route through per-enum drop-glue fns `cnr_dropenum_<eid>`
  (emitted once via a worklist), so a `Box Expr`-inside-`Expr` drop recurses at RUNTIME
  via a `call`. Enum/struct/array-lit ctor payloads now `mark_moved` non-copy operands
  (matching `lower_agg_copy`), so moved values' drops are pruned.
- CALL-RETURNS-RAWPTR pointee (`11_2` `ptr_read(task_of(lp))`): `ptr_type_node` resolves
  a call's pointee from the callee's declared return type (container_of).
- The SysV/stack pole HELD: the pre-existing all-on-stack frame model (rbp-relative
  slots, 16-byte-aligned `rsp` at every `call`, only caller-saved regs) already
  survives the corpus's deep/nested recursion (`11_4` parser, `11_5` arena) — no
  change was required there.

GATE / VERIFICATION (isolation): `selfhost_codegen` green — all 10 tests: N6's five
corpus programs native-compile byte-exact, and N1 (29 scalar) + N2 (12 aggregate) +
N3 (8 drop) + N4 (6 enum) + N5 (16 box/rawptr) stay byte-exact. Full suite: 601
passed, 0 failed. clippy clean. `aot_runtime.c` reused UNCHANGED (the 256 MiB mmap
covers `11_5`'s ~16.8 MiB arena). NO edits to
`interp.cnr`/`lower.cnr`/`mono.cnr`/`layout.cnr`/`src/backend/*` — contained to
`codegen.cnr` + its gate. Gate runtime ~27 s (dominated by `11_5`'s 4096-element
array literal; the emitted `.s` assembles/links/runs without choking `cc`).

## OBL-NG1-GENERICS — native self-compile: USER GENERICS + method dispatch reach codegen.cnr (2026-07-12)

NG1 brings USER generics (`fn[T]`/`struct[T]`/`enum[T]`) and impl-method dispatch to
the Candor NATIVE code generator `selfhost/codegen/codegen.cnr`, so the generic
fixtures native-compile byte-exact. It mirrors the interp/lower wiring of the shared
monomorphizer — no new type machinery, just the same pre-pass + stashes lowered to asm.

- MONO INTO CODEGEN: `codegen_dump` now runs `mono_program(write p, src, head)` right
  after parse (as `lower_dump` does), so it emits the MONOMORPHIZED arena — concrete
  fn/struct/enum instances appended to the item chain and generic references rewritten
  to carry their resolved instance node id. The fn-emission loop skips generic decls
  (`T_FN` with `a != 0`) and emits every concrete instance (`a == 0`). Call sites read
  mono's stashes: `T_CALL.c` (concrete callee fn), `T_GENERICVAL.c` (`name::[T]` value).
- THE T_APP REDIRECT COMES FREE: `layout.cnr`'s `struct_of_ty`/`enum_of_ty`/`ty_size`/
  `ty_align` already redirect a `T_APP` type node with `b != 0` to its stashed concrete
  instance, and codegen routes ALL layout through `layout.cnr`, so a generic type ref
  resolves with no codegen-side change.
- SYMBOL MANGLING (codegen-internal): a generic instance's fn symbol is `cnr_<name>.<nodeid>`
  (the `.<nodeid>` tag only when `ival != 0`, i.e. mono's type-arg cons chain is present);
  a period cannot occur in a Candor identifier, so an instance symbol never collides with
  a plain fn's. Drop hooks likewise gain a `.<sid>` tag for generic struct instances
  (`cnr_drophk_<Name>.<sid>`). The names are private labels (only `candor_entry` + the
  `rt_*` runtime are the ABI), so the mangling need not match lower's `$`-mangled MIR
  names — def and call sites just agree.
- METHOD DISPATCH (mirrors lower T3): `build_dispatch_cg` builds a table from every
  concrete (`a == 0`) `T_IMPL` — key `(target struct-instance node, method-name span)`
  -> method fn node; generic impl templates (`a != 0`) are skipped (mono has
  instantiated them). Impl methods emit as free asm fns `cnr_meth.<fnid>`. A
  T_FIELD-callee `T_CALL` lowers to `gen_method_call`: resolve the receiver place's
  nominal (`gen_place_addr` leaves its flat OFFSET in `%rax`, peel borrow/box for the
  nominal), look up `(sid, method)`, then `call cnr_meth.<fnid>` passing the receiver's
  address as the leading `self` arg. codegen already hands borrow params by address
  (N6), and `gen_place_addr`'s flat offset IS `self`'s address whether the base is a
  value place or an already-borrow local — so no borrow/non-borrow split is needed.

GATE / VERIFICATION (isolation): `selfhost_codegen` green — 11 tests, incl. the new
`candor_native_codegen_equal_to_oracle_over_generics` (11 fixtures: the 8 plain —
`mono3`, `pair`, `genenum`, `arena`, `gdrop`, `gdrop_groundfloor`, `mixed`, `nameval`
— plus `iface`, `gimpl`, `gbound`), each codegen -> `.s` -> `cc` + `aot_runtime.c` ->
run, byte-exact (exit / trace / fault) vs `run_source_real`. All N1-N6 codegen
fixtures stay byte-exact. The gate now composes `mono.cnr` into the module tree.
clippy clean. `aot_runtime.c` reused UNCHANGED. NO edits to
`interp.cnr`/`lower.cnr`/`mono.cnr`/`layout.cnr`/`src/backend/*` — contained to
`codegen.cnr` + its gate. `?`/`From` (fromq/gfromq) and collections are OUT of scope
(NG2 + native-deferred). NG1 + NG2 close user generics on the native tier.

## OBL-NG2-TRYFROM — native self-compile: the `?` operator + `From`-widening reach codegen.cnr — user generics fully native-compile (all 13 generic fixtures) (2026-07-12)

NG2 lowers the `?` operator (`T_TRY`) with `From`-widening to x86-64 asm in
`selfhost/codegen/codegen.cnr`, so `fromq`/`gfromq` native-compile byte-exact. This
closes user generics on the native tier: all 13 generic fixtures now codegen ->
assemble -> link -> run byte-exact vs the oracle. It mirrors `lower.cnr` T5
(`lower_try`/`lower_try_from`/`find_from_impl`) and `interp.cnr` T4
(`eval_try`/`find_from_fn`) — no new type machinery, reusing N4's enum layout + NG1's
mono + `cnr_meth.<fnid>` dispatch.

- OK/ERR DISCRIMINATION (via the ok-marked variant): a result-shaped enum marks its ok
  variant with `T_VARIANT.op == 1`. `gen_try` materializes the operand into a fresh enum
  temp (`gen_agg` leaves its flat OFFSET in a slot), reads `tag@0`, and `cmpq`s the ok
  index (`ok_variant_index_cg`). The ok arm loads the word-sized ok payload at its
  `variant_payload_off` as the `?` value and joins; the err arm early-returns.
- FROM RESOLUTION (`find_from_impl_cg` mirrors lower/interp): scan concrete (`a == 0`)
  `T_IMPL` blocks whose iface name span is `From`, target enum (`nd.c`) == the return
  error payload's enum (e2), and iface arg-0 enum (`nd.b`) == the operand error payload's
  enum (e1); `from_method_of_cg` picks its `from` method node. mono has already
  instantiated a generic `impl[T] From[..] for ..[T]` (gfromq) into a concrete `T_IMPL`,
  so the same scan finds it with no generic-specific code.
- ERR-ARM ASM (`gen_try_from`): call `from(e1)` over the established impl-method ABI —
  e1 by value when word-sized, else by address (its flat offset = `s_local + e1off`) in
  `%rdi` — via `call cnr_meth.<from_fn>` (the same symbol NG1's dispatch emits, so a
  mono-instantiated `from` matches its definition). The call returns e2's flat offset
  (aggregate return convention). Build the enclosing fn's return enum: `rt_stack_alloc`
  its `enum_size`, store `tag = err_variant_index_cg(reid)` at `@0`, `rt_copy` the widened
  e2 into `variant_payload_off`, leave its offset in `%rax`. Same-error-type `?` (no
  widening, `reid == eid`) leaves the operand offset in `%rax` unchanged (early-return the
  operand directly). Then the N3 return sequence: spill, `emit_return_drops`, reload,
  `leave`/`ret`.

GATE / VERIFICATION (isolation): `selfhost_codegen` green — 11 tests, the generics test
now covering 13 fixtures (NG1's 11 + `fromq` + `gfromq`), each codegen -> `.s` -> `cc` +
`aot_runtime.c` -> run, byte-exact (exit / trace / fault) vs `run_source_real`
(fromq/gfromq exit 7). All N1-N6 + NG1 codegen fixtures stay byte-exact. clippy clean.
`aot_runtime.c` reused UNCHANGED. NO edits to
`interp.cnr`/`lower.cnr`/`mono.cnr`/`layout.cnr`/`src/backend/*` — contained to
`codegen.cnr` + its gate. USER GENERICS ARE NOW FULLY NATIVE-COMPILED: all 13 generic
fixtures reproduce the oracle byte-exact with no Rust in the compile path.

- **F-FOUR-ENGINE-CLAIM: ADDRESSED (2026-07-12).** Design 0014 §2.2 now states the
  transitivity honestly as FEATURE-level, not program-level: the four-engine gate covers
  tests/fixtures/{run,parity,real,generics}+corelib, not 11_1..11_5 directly, so the
  engines are proven to agree on the CONSTRUCTS the corpus uses (enough for the claim),
  not on those exact five programs. Softened the "transitively matching native" wording.
  OBL-QUALITY-REVIEW is now fully cleared.

## OBL-N-REGALLOC — native self-compile: a register-allocator MVP for `codegen.cnr` (within-expression operand pairs in callee-saved registers) (2026-07-12)

`selfhost/codegen/codegen.cnr` gained an OPERAND-REGISTER FILE so the hot
left-operand spill/reload keeps the left operand ALIVE in a callee-saved register
across evaluating the right operand, instead of a `%rbp` temp slot. This is a
behavior-preserving optimization of the emitted asm only: all 89 codegen fixtures
stay byte-exact (same exit / trace / fault) vs `run_source_real`. The five
callee-saved regs `%rbx/%r12/%r13/%r14/%r15` were completely unused by codegen and
are preserved across every `call` (runtime helpers, `cnr_*`, drop hooks, `call
*reg`) and disjoint from every scratch the flat-mem helpers/arg conventions touch,
which is why the MVP is non-invasive.

- REGISTER FILE (`spill_reg`/`reload_reg`, beside `spill`/`reload_rax`): a LIFO
  stack of the 5 regs with a depth counter (`rf_depth`). `spill_reg` parks `%rax`
  in the next free reg and returns a NEGATIVE handle `-(reg+1)` when the file is
  armed and depth < 5; otherwise it FALLS BACK to `spill` and returns the (>= 0)
  slot handle. `reload_reg` moves a reg handle's value back to `%rax` and pops
  (LIFO), or reloads the slot. Because the fallback is always correct, the file
  being armed or not only changes performance, never behavior.
- CONVERTED SITES (exactly the three straight-line left-operand pairs): `gen_two`,
  `gen_binary` (arith/bitwise/shift), `gen_cmp` — the only uses of the local `ls`.
  Every other spill site (aggregates, calls, drops, box/alloc, `gen_return`'s
  spill-across-`emit_return_drops`, `gen_short_circuit`, place resolution) is left
  on the slot path unchanged.
- FIXED SAVE GROUP via `%rbp`-relative SLOTS (not push/pop, so `emit_epilogue`'s
  `movq %rbp,%rsp; popq %rbp` stays alignment-agnostic): a per-fn used-flag
  (`rf_used`) is decided UP FRONT by a cheap AST pre-pass (`subtree_uses_regfile`)
  BEFORE the walk. If armed, `gen_fn` reserves save slots 0..4 (folded into
  `slot_max`/the frame), `emit_fn_streaming` saves the 5 regs after `subq $frame`,
  and `emit_epilogue` restores them at EVERY exit. `rf_used`/`rf_depth` are reset
  in all four function generators; only `gen_fn` arms the file (hooks/glue/static-
  init keep the slot baseline).
- INVARIANTS: (1) LIFO balance — spill_reg/reload_reg are perfectly nested inside a
  single `gen_*` call, so the reg handle encodes the exact reg and the pop always
  matches. (2) The used-flag is a per-fn CONSTANT known before any epilogue is
  emitted, which is required because a loop back-edge can reach an early `return`
  after a register was clobbered — a walk-time flag would miss that restore. (3) A
  fault label from a right-operand sub-eval falling between a `spill_reg` and its
  `reload_reg` is safe: `fault_unless` is a forward branch over non-returning code
  (`rt_fault` never returns), so no live alternate path reaches the reload with the
  reg clobbered; drops clobber only caller-saved regs.

GATE / VERIFICATION (isolation): `selfhost_codegen` green — 12 tests (the 11 fixture
tests, all 89 fixtures byte-exact across N1-N6 + generics, plus a new
`candor_native_codegen_uses_operand_register_file` spot-check asserting fib's emitted
`.s` actually parks the outer `+`'s left operand in `%rbx`
(`movq %rax, %rbx`/`movq %rbx, %rax`) and saves/restores the group
(`movq %rbx, -8(%rbp)`/`movq -8(%rbp), %rbx`), distinguishing "regalloc runs" from
"dead code"). clippy clean. Contained to `codegen.cnr` + its gate; NO edits to
`interp.cnr`/`lower.cnr`/`mono.cnr`/`layout.cnr`/`src/backend/*`. The self-hosted
native codegen now does real register allocation for scalar operand pairs.


## LLVM-S0 — the first OPTIMIZED native backend: MIR -> textual LLVM-IR built by `clang -O2`, byte-exact vs the oracle (2026-07-12)

A new Rust-side production backend `src/backend/llvm.rs` emits TEXTUAL LLVM-IR from
the same checked MIR the Cranelift backend consumes, for the scalar + control-flow
subset, built by `clang -O2` and linked against the SAME static C runtime
(`aot_runtime.c`, reused UNCHANGED) as the Cranelift AOT object. It mirrors
`backend::lower`'s SEMANTICS in `.ll` text; it reuses no Cranelift code. This is the
first optimized native code Candor produces via LLVM — the earlier native tiers run
at `opt_level=none` (Cranelift) or emit naive asm (self-host codegen).

- VALUE MODEL (Tier-R, the perf win): each scalar local is an `alloca i64` in the
  entry block; a use is a `load`, a definition a `store`. No local's address is
  taken and there are no aggregates, so `clang -O2`'s mem2reg promotes every slot to
  an SSA register. Scalars NEVER route through the flat MEM_BASE buffer /
  rt_stack_alloc (that would defeat mem2reg). Values are canonicalized (trunc/sext/
  zext per the local's width) exactly as `lower::canon` does; the slot always holds
  the canonical i64.
- CHECKED ARITH (INV-CHECK): checked add/sub/mul use `llvm.{s,u}{add,sub,mul}.
  with.overflow.iN` -> `extractvalue` result + overflow bit -> `br i1` to a fault
  block. NEVER `add nsw`/`mul nsw` (LLVM would delete the overflow check as
  UB-unreachable). Conv/neg range checks are explicit `icmp` in i128 against the
  target min/max. Div/Rem guard divisor==0, then guard signed MIN/-1 via a
  `select`ed safe divisor so `sdiv` never sees UB — op-for-op with
  `lower::range_or_fit` / the div path. Wrapping = plain iN wrapping ops; Saturating
  = i128 compare + select-clamp.
- FAULT BLOCKS + rt_* (INV-FAULT-ID / INV-OBS-ORDER): each fault edge is `call void
  @rt_fault(i32 kind_code, i32 s, i32 e)` then `unreachable`; `rt_fault` is declared
  `noreturn`. `rt_trace`/`rt_fault` are BARE external declares (no `readnone`/
  `memory(none)`), so `-O2` preserves trace order and fault points (external
  side-effecting calls are optimization barriers). The kind-code is the stable
  `lower::kind_code` map, reused directly.
- ENTRY: `define i64 @candor_entry()` calls `@cnf_main` and returns its i64 (or 0
  for a non-i64 main), mirroring `lower::lower_entry` for the scalar subset (no
  statics/strings). Candor fn symbols are prefixed `cnf_` (as `object.rs` does) and
  emitted `internal` so `-O2` can inline; `candor_entry` is external. The full
  requires/ensures predicate-region + shared-final-return block structure of
  `lower::lower_fn` is replicated so contract faults land byte-exact.
- WIRING: `compile_path_llvm(path, out)` and `emit_llvm_ir(path)` in `lib.rs` beside
  `compile_path`; `candor compile --backend=llvm x.cnr -o prog` in `main.rs`. The
  clang invocation reuses `aot_runtime.c` verbatim: `clang -O2 <file>.ll
  <aot_runtime.c> -o out -no-pie -pthread`.

GATE / VERIFICATION (isolation): `tests/llvm.rs` green — 3 tests. `gate_llvm_fault_
axes` (8 axes: overflow, div-by-zero, conv-loss, assert, requires, ensures, panic,
shift-overflow) and `gate_llvm_ok_slice` (arithmetic, fib recursion, while-loop sum,
bitwise) compile via `clang -O2`, run as standalone processes, and compare (exit
byte, stdout trace, stderr fault-JSON) byte-exact to the tree-walking oracle
(`run_source`/`run_source_real`). `perf_mem2reg_promotes_locals` proves the
optimization is REAL: the emitted `.ll` uses `alloca i64` per local, and `clang -O2
-S -emit-llvm` output contains NO `alloca` (mem2reg promoted every slot to a
register). clippy clean. Additive: `aot`/self-host gates untouched; `aot_runtime.c`
reused UNCHANGED; no new crate dependency (format! + shelling clang). GAP: the
array-bounds and enum/`?` axes of `tests/aot.rs`'s FAULTS array are OUT of the S0
scalar subset (aggregates/enums) and are correctly rejected by the emitter with a
precise "out of LLVM-S0 subset" error, not faked — deferred to a later slice.

## LLVM-S1 — structs + arrays: the two-tier value model on the LLVM backend (2026-07-12)

Extends `src/backend/llvm.rs` from the S0 scalar subset to STRUCTS and ARRAYS by
introducing the two-tier value model that mirrors LLVM's own "is the address taken?"
question, then mirrors `backend::lower`'s flat-aggregate SEMANTICS in `.ll` text. Same
static C runtime (`aot_runtime.c`, still UNCHANGED); no Cranelift code reused.

- TWO-TIER CLASSIFICATION (`classify_tiers`, a trivial per-fn MIR scan): each local is
  Tier-R or Tier-F. **Tier-R (register)** = a word-sized (`is_wordy`) scalar/pointer
  whose address is NEVER taken -> `alloca i64` (the S0 model; mem2reg promotes it to an
  SSA register — the S0 perf win is PRESERVED verbatim: a scalar-only program emits the
  identical alloca-per-local IR, no `rt_stack_alloc`). **Tier-F (flat)** = an aggregate
  (`!is_wordy`) OR any local whose address is taken — the root of a `Ref`, or of a
  `CopyVal` src/dst, whose projection does not begin with a `Deref` (a leading `Deref`
  reads the local's pointer *value*, not its address). A Tier-F local lives in the flat
  MEM_BASE buffer via `%off<id> = call rt_stack_alloc(size, align)` at fn entry; its
  stable MEM_BASE-relative offset IS the Candor "address" that `Ref`/`Index`/borrows
  pass around.
- FLAT ACCESS (mirrors `lower::host_addr`/`load_scalar`/`store_scalar`): a scalar leaf
  is `%p = inttoptr i64 (add MEM_BASE, %off) to ptr` then a typed `load`/`store i{bits}`
  (sext/zext to the canonical i64); a whole aggregate is `call rt_copy(dst, src, len)`.
  Aggregate loads/stores through `inttoptr` are correct but optimizer-opaque — the
  documented flat-arena ceiling (making aggregates fast is a later ABI project).
- PLACE PROJECTIONS (`place_addr`, op-for-op with `lower::place_addr`): `Field{offset}`
  -> `add`; `Deref` -> `load` the pointer; `Index{stride,len,span}` -> the bounds check
  (`icmp uge i64 %i, %len` -> `br` to a `Bounds` fault block, kind + span byte-exact)
  then `%off = add base, (mul %i, stride)`. Offsets/strides/lens are already baked in
  the MIR by the front end; only aggregate sizes/alignments come from `Layout`
  (`emit_ll` now threads `items`/`consts`). Slice headers `{ptr@0,len@8}` handled too.
- AGGREGATE ABI (mirrors `lower`): struct/array literals store each scalar leaf at its
  field/element offset (`Store` with projections) and byte-copy aggregate leaves
  (`CopyVal`); array-repeat `[e;N]` copies one temp N times. A by-value struct param
  arrives as the caller's candor offset and is `rt_copy`-ed into the callee's Tier-F
  slot; a struct return delivers the candor offset of `_0`'s slot (convention B), which
  the caller `CopyVal`s from through a Tier-R pointer temp. `Rvalue::Ref` of a place is
  its candor offset (an i64). A drop-inert aggregate's `Drop` is a no-op (gated on
  `LocalDecl::drop_obligation`); a needs-drop value stays out of subset.
- GATE / VERIFICATION (isolation): `tests/llvm.rs` green — 5 tests. New:
  `gate_llvm_aggregates` (struct literal/field-read/field-assign, nested struct,
  by-value struct param + struct return, array listed/repeat literal, array index read +
  index-assign, array-of-structs, sub-word u8 elements) and
  `gate_llvm_aggregate_bounds_fault` (array-index OOB, incl. array-of-struct) compile
  via `clang -O2`, run as standalone processes, and compare (exit / trace / fault-JSON)
  byte-exact to the oracle. The array-bounds fault — the S0 GAP — is now IN subset and
  matches the oracle's `bounds` kind + span. All S0 fixtures + `perf_mem2reg_promotes_
  locals` stay green (scalars remain Tier-R). `aot`/self-host gates untouched;
  `aot_runtime.c` reused UNCHANGED; clippy clean. Out-of-S1 ops (enum/box/collections/
  concurrency, needs-drop) still REJECTED precisely ("out of LLVM-S1 subset"), never
  faked. NEXT: S2 (enums + statics).

## LLVM-S2 — tagged-union ENUMS + STATIC/CONST data on the LLVM backend (2026-07-12)

Extends `src/backend/llvm.rs` from the S0+S1 subset to TAGGED-UNION ENUMS and the
STATIC/CONST data region — purely additive, mirroring `backend::lower`'s SEMANTICS in
`.ll` text. Same static C runtime (`aot_runtime.c`, still UNCHANGED); no Cranelift
code reused; no `nsw`/`nuw`.

- ENUM LAYOUT (from the shared `interp::layout`, the same source `lower` uses): a
  tagged union = a `u64` tag at offset 0 (`ENUM_TAG=8`) followed by the payload union
  sized to the largest variant (`enum_size`); payload field `i` of a variant sits at
  `ENUM_TAG + lay_fields(payload)[i].offset`. An enum is `!is_wordy`, so `classify_
  tiers` makes every enum local Tier-F — it lives in the flat MEM_BASE buffer via
  `rt_stack_alloc`, exactly like a struct/array. NO new emitter code was needed for the
  enum value model: the MIR front end already bakes the tag/payload byte offsets into
  `Proj::Field`, so the S1 flat-aggregate path (`place_addr`, `Store`, `CopyVal`,
  sub-word trunc/sext/zext canonicalization) covers enums verbatim.
- CONSTRUCTION: `EnumCtor` lowers (in MIR) to a `Store` of the tag constant at offset 0
  plus a `Store`/`CopyVal` of each payload field at its offset — ordinary S1 stores.
- MATCH: MIR lowers `match` to a tag read (`Load` of the `u64` at offset 0) followed by
  an `icmp eq`/`Branch` chain, one test per `Variant` arm, with the wildcard/`Binding`
  arm as the unconditional default and a `Panic` fault on a non-exhaustive fall-through
  — so the LLVM emitter's existing `Cmp`+`Branch` lowering reproduces the switch and the
  exhaustiveness/default behavior byte-for-byte; each arm binds its pattern fields as
  payload-offset field reads/borrows (copy-bind `Load`, move-bind/borrow-bind via the S1
  paths). Enum-payload sub-word scalars are canonicalized like any S1 leaf.
- STATIC/CONST DATA (the new emitter work): `layout_statics_strings` reproduces the
  Cranelift driver's bump arithmetic (`STATIC_BASE`, statics in program order each
  aligned, then interned string bytes) so the `StaticAddr`/`StrAddr` constants baked
  into function bodies agree with what the entry prologue writes. `Rvalue::StaticAddr`/
  `Rvalue::StrAddr` now emit the baked Candor address as an immediate `u64` (a `static`
  place is modeled by the front end as `*(&STATIC)` — a Tier-R pointer temp holding the
  address, then a `Deref` — so `place_addr`'s existing leading-`Deref` path reads through
  it). Integer `const`s stay immediate (`Operand::Const`); the `consts` map only feeds
  `Layout` array lengths. `candor_entry`'s PROLOGUE now mirrors `lower::lower_entry`:
  copy each string literal's bytes into the flat buffer at `MEM_BASE + addr + i`
  (`inttoptr`+`store i8`), then run each static initializer `cnf_<init_fn>()` and deliver
  its result to the static's slot (wordy: store the low `size` bytes; aggregate:
  `rt_copy` from the returned candor offset), then call `main`. Compiler-synthesized body
  names (e.g. a static's `"<init G>"` initializer) are not valid bare LLVM identifiers,
  so every `cnf_*` symbol is now emitted QUOTED (`@"cnf_..."`).
- GATE / VERIFICATION (isolation): `tests/llvm.rs` green — 7 tests. New:
  `gate_llvm_enums_and_statics` (enum construct + `match` payload bind; multi-variant +
  wildcard/default arm; the real-syntax `propagate` golden — struct-in-enum `?` chain;
  enum-in-struct; static scalar/struct/array reads; string-literal `b"..."` byte data)
  and `gate_llvm_enum_fault_axis` (a div-by-zero raised through an enum-returning `?`
  chain — `div_by_zero` kind + span byte-exact) compile via `clang -O2`, run as
  standalone processes, and compare (exit / trace / fault-JSON) byte-exact to the oracle.
  All S0+S1 fixtures + `perf_mem2reg_promotes_locals` stay green; `tests/aot.rs` (6
  tests) untouched and green; `aot_runtime.c` reused UNCHANGED; clippy clean. Out-of-S2
  ops (box/alloc/rawptr, needs-drop, collections, concurrency) still REJECTED precisely
  ("out of LLVM-S2 subset"), never faked. NEXT: S3 (drop — the tallest pole).

## LLVM-S3 — the MOVE/DROP SCHEDULE with trace-on-drop on the LLVM backend (2026-07-12)

Extends `src/backend/llvm.rs` from S0+S1+S2 to the deterministic-destruction
contract: every needs-drop value runs its drop glue at the scheduled point, with
trace-on-drop firing in `backend::lower`'s exact order — purely additive, same
static C runtime (`aot_runtime.c`, still UNCHANGED), no Cranelift code reused, no
`nsw`/`nuw`.

- ALREADY-IN-MIR vs NEW EMITTER WORK (the S1/S2 lesson holds, strongly). The
  ENTIRE drop SCHEDULE is baked into the MIR by the front end (`mir::build`): each
  `Drop` statement is already placed at scope exit / early `return` / `break` in
  REVERSE declaration order, and carries a STATIC move mask (`moved: Vec<Vec<String>>`)
  — the field-name sub-paths already moved out at that point. There is NO runtime
  drop flag anywhere (design 0010 §2 INV-DROP): conditional-move correctness is a
  compile-time mask, and the checker forbids inconsistent move state at CFG joins
  (E0302), so every drop site has one deterministic mask. The NEW emitter work is
  only the drop-GLUE expansion at each `Drop`: recurse the static type + mask into
  the matching LLVM, op-for-op with `lower::emit_drop`.
- DROP-GLUE MODEL (`FnEmit::emit_drop`/`drop_enum`, mirror of `lower::emit_drop`/
  `drop_enum`). A needs-drop local is non-wordy -> Tier-F, so its flat candor offset
  `%off{id}` is the drop address. STRUCT: if not partially-moved, fire the `drop`
  hook (`call cnf_<drop Name>(i64 addr)` — the user hook body, already emitted as an
  ordinary MIR fn, is where the observable `trace` lives) THEN drop each field in
  reverse declaration order. ARRAY: drop each element in reverse index order (stride
  = round_up(size,align)). ENUM: load the u64 tag at offset 0, then per variant with
  a droppable payload emit `icmp eq tag, idx` -> branch to a per-variant block that
  drops the payload fields in reverse, all joining a `merge` block (variants with no
  droppable payload emit no test). Recurses; `needs_drop` mirrors `lower::needs_drop`.
- DROP-FLAG / CONDITIONAL-DROP MECHANISM (no runtime flag; static mask). `emit_drop`
  is pruned by the same `is_moved`/`partially`/`prefix` predicates `lower` uses: a
  path fully covered by the mask is skipped (a moved value never double-drops), a
  strictly-deeper mask entry marks a struct partially-moved (skip its whole-value
  hook, drop only the still-owned fields). A value consumed by a call is moved out
  and pruned from the caller's schedule, dropped exactly once inside the callee's
  param scope; early-return drops fire on every control path.
- OUT OF S3 (rejected precisely, never faked). A drop that recurses through
  `Box`/`BoxResult` needs the allocator's `free` through the vtable — that is S4;
  `emit_drop` rejects it with "out of LLVM-S3 subset: drop through Box/alloc". The
  corpus fixtures still out-of-subset for this (and the surrounding Alloc/rawptr
  machinery) are `11_1_allocator`, `11_5_arena`, `11_4_parser` (all reject on the
  allocator-handle `CopyVal`, before any drop). The pure-VALUE drop/trace corpus
  fixtures ARE in subset: `generics/gdrop.cnr` (generic struct hook fires BEFORE the
  field drop) and `generics/gdrop_groundfloor.cnr` (move-into-callee param drop).
- GATE / VERIFICATION (isolation): `tests/llvm.rs` green — 9 tests. New:
  `gate_llvm_drop_trace` (9 fixtures: single-struct hook, two-value reverse order,
  conditional-move no-double-drop + param drop, nested struct inner-to-outer, array
  reverse, enum active-variant payload, enum non-droppable variant, early-return on
  both paths, partial-move hook-skip) and `gate_llvm_drop_trace_corpus` (the two
  generics value-drop fixtures) compile via `clang -O2`, run standalone, and compare
  (exit / TRACE / fault-JSON) byte-exact to the oracle — the TRACE order is the
  correctness axis. All S0+S1+S2 fixtures + `perf_mem2reg_promotes_locals` stay
  green; `tests/aot.rs` (6 tests) untouched and green; `aot_runtime.c` reused
  UNCHANGED; clippy clean. NEXT: S4 (Box/alloc/rawptr — drop through the allocator).

## LLVM-S4 — HEAP ALLOCATION: Box[T], the allocator ABI, rawptr load/store, and drop-through-Box (2026-07-12)

Extends `src/backend/llvm.rs` from S0..S3 to the systems corpus: heap allocation
through a Candor allocator handle, raw-pointer memory access, and the free-on-drop
that completes the arena drop story — purely additive, same static C runtime
(`aot_runtime.c`, still UNCHANGED), no Cranelift code reused, no `nsw`/`nuw`.

- ALREADY-IN-MIR vs NEW EMITTER WORK (the S1/S2/S3 lesson holds). `mir::build` bakes
  the whole shape: `box(a, v)` is a `BoxOp { dst, inner_ty, result_ty, alloc, value }`
  statement, `unbox(b)` an `UnboxOp { dst, inner_ty, boxed }`, `ptr_read`/`ptr_write`
  through a `rawptr` are `Load`/`Store`/`CopyVal` with `observable = true`, and a fn
  named as a value is `Operand::Const(id, u64)` — a `u64` index into `prog.fn_ptrs`.
  The allocator handle is NOT a C `rt_alloc`; it is a Candor `Alloc { ctx, vt }` whose
  `vt` points at an `AllocVtable { alloc, free }` of fn-pointer ids (design 0010 §5).
  NEW emitter work: the fn-pointer dispatch table, the vtable-indirect alloc/free
  calls, the box/unbox/subslice payload moves, observable rawptr/MMIO barriers, and
  the per-Box-pointee drop-glue functions.
- BOX MODEL (`FnEmit::box_op`/`unbox_op`, mirror of `lower`). `box`: load `ctx`/`vt`
  from the `Alloc` handle at the `alloc` operand's address, load the `alloc` fn-ptr id
  from the vtable, call it `(ctx, size, align)` -> a flat block address (0 == OOM). On
  OOM: drop the payload, write the `oom` tag. On success: `rt_copy` the payload into
  the block, then build the boxed arm `{tag, ptr@8, ctx@16, vt@24}`. A `Box` value is
  the 24-byte `{ptr@0, ctx@8, vt@16}`; `bx.*` is a `Load`/place through a `Deref` of
  `ptr@0`. `unbox`: `rt_copy` the pointee into `dst`, then free the block.
- ALLOCATOR-ABI THREADING (the CopyVal that used to reject). The handle is passed by
  borrow: the `alloc` operand is a wordy local holding the `Alloc` struct's flat
  address; `operand()` loads it, and `box`/`unbox`/`free` read `ctx`/`vt` and the
  vtable fn-ptr ids relative to it. A fn-ptr id `i` dispatches through the emitted
  module global `@cn_fnptr_table = [N x ptr]` (slot `i` = `@cnf_<fn_ptrs[i]>`), the
  textual-IR twin of `object.rs`'s `cn_fnptr_table` data object: `getelementptr` the
  slot, `load ptr`, indirect `call`. The observable `CopyVal`/`Load`/`Store` reading
  the handle-backing (`ptr_read(cast_ptr[Pool](ctx))`) now lower normally (aggregate
  copy -> `rt_copy` barrier), removing the S3 reject.
- RAWPTR LOAD/STORE (INV-OBS-ORDER). An `observable` scalar `Assign(Load)` /`Store`
  through a `rawptr` lowers to a barrier CALL `rt_mmio_load(addr,size)` /
  `rt_mmio_store(addr,val,size)` (declared bare, so `-O2` never reorders/elides the
  device access), canonicalized like an S1 scalar; an observable aggregate `CopyVal`
  falls through to the `rt_copy` barrier. `IsNull`/`PtrArith` rvalues and the
  `CallIndirect` rvalue join the emitter. `rt_mmio_load`/`rt_mmio_store` already exist
  in `aot_runtime.c` (reused UNCHANGED).
- DROP-THROUGH-BOX (free-on-drop completes the arena story). `emit_drop`'s Box/BoxResult
  reject is replaced: `Box(inner)` -> `drop_box` (guard `ptr != 0`; recursively drop the
  pointee through its glue fn, then `free(ctx, vt, ptr, size, align)`); `BoxResult` ->
  `drop_enum`. A recursive Box type drops through a synthesized per-pointee glue fn
  `@__drop_glue_<i>(addr)` (`collect_glue_types` from `lower`), so recursion terminates
  at the runtime null guard — never infinite compile-time unrolling. The free / pointee
  hook is observable exactly where `lower` makes it so.
- GATE / VERIFICATION (isolation): `tests/llvm.rs` green — 11 tests. New:
  `gate_llvm_systems_corpus` (all five design 0001 §11 programs: `11_1_allocator` pool
  free-list, `11_2_scheduler` intrusive-list `container_of`, `11_3_mmio` device
  register FSM, `11_4_parser` recursive-descent over a bump arena with recursive
  `Box Expr`, `11_5_arena` `Box [4096]Node` arena) and `gate_llvm_drop_through_box` (a
  drop-hooked `Res` boxed then DROPPED, so free-on-drop fires the pointee hook: trace
  ends `... 77`) compile via `clang -O2`, run standalone, and compare (exit / TRACE /
  fault-JSON) byte-exact to the oracle. ALL FIVE corpus programs pass. All S0..S3
  fixtures + `perf_mem2reg_promotes_locals` stay green; `tests/aot.rs` (6 tests)
  untouched and green; `aot_runtime.c` reused UNCHANGED; clippy clean. Out-of-S4 ops
  (FFI/extern, concurrency/spawn) still REJECTED precisely ("out of LLVM-S4 subset"),
  never faked. NEXT: S5 (FFI + concurrency).

## LLVM-S5 — EXTERN/FFI + STRUCTURED CONCURRENCY on the LLVM backend (2026-07-12)

Extends `src/backend/llvm.rs` from S0..S4 to the WHOLE native language surface:
boundary `extern` (real libc I/O across the audited trust boundary) and design 0012
Stage-2 structured concurrency (`spawn`/`scope` on real OS threads). Purely additive,
same static C runtime (`aot_runtime.c`, still UNCHANGED — no runtime symbol added or
modified), no Cranelift code reused, no `nsw`/`nuw`. After S5 the only construct still
out of subset is the `Vec`/`Map`/`String` collection intrinsics — MIR-interp only in
`lower` too (native codegen of their alloc/hash bodies is a shared forward dependency),
so the LLVM backend now covers exactly what the Cranelift backend covers.

- ALREADY-IN-MIR vs NEW EMITTER WORK (the S1..S4 lesson holds). `mir::build` bakes the
  shape; the emitter mirrors `lower` op-for-op. A boundary `extern` call is an ordinary
  `Rvalue::Call { func, args }` whose `func` is a key in `items.externs` (NOT a `cnf_`
  body) — the emitter branches on that. `spawn`/`scope` are dedicated statements
  `Spawn { func, args }` / `ScopeBegin` / `ScopeEnd` (previously the S4 reject). NEW
  emitter work: the imported-C-symbol declares, the C-ABI call marshalling, the
  scope/spawn runtime declares, and the `rt_spawn` marshalling (function-address +
  padded i64 args).
- FFI MODEL (`FnEmit::extern_call`, mirror of `lower::lower_extern_call`). Each
  `extern` is declared as an IMPORTED C symbol at its SysV C-ABI signature via
  `c_abi_llty` (a pointer word or a full 64-bit scalar -> `i64`; a narrower scalar ->
  `i32`; a unit return -> `void`), keyed by `c_symbol_name` (`sys_read`->`read`, …).
  The call marshals per the C ABI: a `rawptr`/borrow argument is a flat-buffer OFFSET
  translated to the REAL host pointer `MEM_BASE + offset` that libc needs; a <=32-bit
  scalar is `trunc`ed to its `i32` register width; a sub-word return is `sext`/`zext`ed
  back to the canonical i64 word. The imported symbol resolves to REAL libc in the
  linked binary — the standalone-binary trust boundary (the extern carries the
  assumed-proven TRUST; the safe `pub` wrappers carry the enforced value contracts).
  The declares are bare (no `readnone`), so `-O2` preserves I/O ordering.
- CONCURRENCY MODEL (mirror of `lower`'s `Spawn`/`ScopeBegin`/`ScopeEnd`). `spawn
  CALLEE(args)` emits `call void @rt_spawn(i64 ptrtoint (ptr @cnf_CALLEE to i64), i64
  argc, a0..a5)`: the task fn's ADDRESS (address-taken, so `-O2` keeps its C calling
  convention and never elides it), `argc`, and the args MARSHALLED ON THE PARENT
  THREAD — each a single i64 (a scalar value, or an aggregate/slice's flat address —
  the same operand ABI as an ordinary call), padded to `MAX_SPAWN_ARGS` (6). The
  runtime creates a real pthread that casts the address back to a fn-ptr of the right
  arity and calls it (shared-nothing/message-passing: only the marshalled words cross;
  race freedom is a COMPILE-TIME guarantee already checked). `scope { … }` brackets
  emit `rt_scope_begin` / `rt_scope_end`; the closing brace JOINS every task in SPAWN
  ORDER, MERGES each task's per-thread trace fragment into the parent's trace (so `θ`
  is DETERMINISTIC regardless of the OS-thread interleaving — this is the mechanism
  that makes the cross-thread trace byte-exact), and re-delivers the spawn-order-first
  fault on the parent thread. `rt_spawn`/`rt_scope_begin`/`rt_scope_end` already exist
  in `aot_runtime.c` (reused UNCHANGED). The LLVM gate mirrors `tests/aot.rs`'s
  concurrency mechanism exactly (compare the compiled binary against the sequential
  oracle, whose inline spawn-order run yields the same deterministic θ + fault).
- GATE / VERIFICATION (isolation): `tests/llvm.rs` green — 14 tests (11 from S0..S4 +
  3 new). `gate_llvm_concurrency` runs the three design-0012 fixtures the AOT gate
  carries (the parallel-fill flagship via `split_mut` + two disjoint `spawn`s -> exit
  byte 2211; per-task trace projection merged in spawn order -> θ == [100, 3, 4, 200];
  a spawned div-by-zero task whose fault the join re-delivers -> exit 2 + (kind, span)).
  `gate_llvm_native_io_real_libc` builds the std_io demonstrator with `clang -O2` into
  a standalone binary that calls REAL libc open/read/write/close (no shim registry),
  run with the fixture dir as cwd: exit byte + stdout bytes byte-exact vs the
  shim-backed interpreter (uppercased 17-byte fixture, `HELLO, CANDOR IO\n`).
  `gate_llvm_native_io_open_error` opens a missing file -> the `Fail` arm's negative
  exit byte, matched. All S0..S4 fixtures + `perf_mem2reg_promotes_locals` stay green;
  `tests/aot.rs` (6 tests) untouched and green; `aot_runtime.c` reused UNCHANGED;
  clippy clean. The LLVM backend is now a fully-covering native engine, ready for S6
  (wiring it in as the 5th Stage-D proven-equivalent engine over the whole corpus).

## LLVM-S6 — the CAPSTONE: LLVM wired in as the FIFTH proven-equivalent Stage-D engine over the whole corpus (2026-07-12)

The LLVM backend (`src/backend/llvm.rs`, frozen at S5 — NO codegen change in this slice)
joins the P5 cross-mode determinism proof as a co-equal fifth engine. Purely additive:
every existing test stays byte-exact green; `aot_runtime.c` reused UNCHANGED.

- WHAT THE FOUR-ENGINE PROOF ACTUALLY IS (honest structure — no overclaim). The P5
  equivalence is NOT a single pairwise N-way diff; it is TRANSITIVE-THROUGH-A-SHARED
  ORACLE. `tests/stage_d.rs` runs the corpus through the four IN-PROCESS engines —
  tree-walking oracle (`run_source`), MIR interpreter, native-noopt (Cranelift JIT),
  native-opt (MIR R1 rewrite + Cranelift `opt_level=speed`) — and asserts each of
  mir/noopt/opt `== oracle` (`assert_eq!(engine, oracle)`), i.e. three per-engine
  byte-exact-vs-oracle comparisons sharing one oracle; equality among the engines is
  transitive through that pivot. `tests/aot.rs` adds a fifth comparison of the SAME
  shape: the Cranelift linked-ELF process vs the SAME oracle over the SAME corpus. The
  observable compared is `(k, s, θ)` = fault (kind, span) OR (exit byte, printed trace);
  for the ELF engines "ret" is the Unix low exit byte (`ret as u8`), the standard fair
  comparison aot.rs already makes.
- HOW LLVM IS ADDED (the honest 5th member — byte-exact vs the same oracle). Because the
  proof is transitive-through-oracle, the honest wiring is a sixth comparison of that
  same shape, not a fabricated unified diff. New test `gate_llvm_full_corpus_fifth_engine`
  in `tests/llvm.rs` enumerates the IDENTICAL corpus `tests/aot.rs`/`tests/stage_d.rs`
  close on — every `run`/`parity`/`real`/`generics` fixture (`.cn`+`.cnr`), the flat
  `corelib_flat.cnr`, and the `corelib`/`corelib_question` module trees — compiles each
  through `compile_path_llvm` (clang -O2 ELF), runs the standalone process, and asserts
  its `(k, s, θ)` byte-exact against the SAME tree-walking oracle. LLVM thus enters the
  equivalence set exactly as the Cranelift-ELF engine did: co-equal via the shared oracle.
- COVERAGE (fair, nothing narrowed). The gate reports `31 runnable fixtures clang-O2 ==
  oracle; not-runnable=0` — the SAME 31 fixtures `tests/stage_d.rs` (`STAGE D: 31 …
  4-engine … equal`) and `tests/aot.rs` (`AOT GATE: 31 … == oracle`) close on. The
  four-engine corpus contains NO CollectionOp program: `Vec`/`Map`/`String` are
  MIR-interp-only in BOTH the Cranelift MIR-lowering and the LLVM backend, and live in
  the separate `tests/vec`·`map`·`text` suites, never in these directories (grep of the
  corpus finds the word "collection" only in `corelib/core/iter.cnr` comments). So the
  LLVM engine's coverage == the Cranelift engine's coverage == the whole four-engine
  corpus — a fair equivalence with the one legitimate, shared exclusion, logged not faked.
- WHAT FIVE-ENGINE EQUIVALENCE NOW PROVES. For all 31 corpus programs, five independent
  Stage-D backends — tree-walking interpreter, MIR interpreter, Cranelift JIT (no-opt),
  Cranelift optimized (JIT + R1) / Cranelift linked-ELF, and now the LLVM `clang -O2`
  linked-ELF — produce the byte-identical observable `(k, s, θ)`: identical return/exit
  byte, identical printed trace `θ`, and identical fault identity `f★` = (kind, span).
  Because every engine is verified byte-exact against ONE shared tree-walking oracle,
  their mutual agreement is transitive through that oracle (not a direct all-pairs diff).
  This does NOT prove equivalence outside the shared subset (CollectionOp is excluded
  from all engines alike) nor a whole-program formal refinement; it is an empirical,
  corpus-bounded cross-backend determinism guarantee — a fifth, differently-built
  optimizing native codegen (LLVM) preserves the exact observable semantics.
- GATE / VERIFICATION (isolation). `tests/llvm.rs` green — 15 tests (14 from S0..S5 + the
  new fifth-engine corpus gate). `tests/stage_d.rs` (5 tests) and `tests/aot.rs` (6 tests)
  untouched and green. No change to `src/backend/llvm.rs` semantics, the Cranelift backend,
  the self-host modules, or `aot_runtime.c`. No `nsw`/`nuw`. clippy clean.

## STD-FMT — the formatting foundation: `fmt_i64` + the `Show` value-rendering convention (2026-07-12)

The first slice of the "real std + I/O" milestone and the language's first
value-rendering convention. Purely additive: `fmt_i64` (decimal rendering of a
signed 64-bit integer) and a `Show` interface (the Display analog), both pure
Candor over the design-0013 `String` primitives — NO compiler/MIR/checker
change. Source: `tests/fixtures/std_fmt.cnr` — a self-contained single-file
corelib image (the `corelib_flat` pattern). It is NOT a `corelib/` module tree:
it uses the design-0013 `String` (a MIR-interp-only CollectionOp), and the
corelib tree is compiled by the AOT/LLVM/Stage-D native gates, which exclude
CollectionOp — so `String` std stays single-file, exactly like `text.rs`/`vec.rs`/
`map.rs`. Runtime observation: `tests/fmt.rs`, which appends a `main`.

- THE PRIMITIVE SURFACE (built on, not assumed). `string_new(a: read Alloc) ->
  String`; `push(write String, c)` appends ONE Unicode scalar (an i64 codepoint,
  UTF-8-encoded, `enforced requires(is_scalar_value)`); `append(write String,
  s: str)` appends a view; `as_str(read String) -> str`. A digit `d` is emitted
  as the scalar `48 + d` ('0'..='9'); the sign is the scalar `45` ('-').
- `fmt_i64` AND THE `i64::MIN` EDGE (load-bearing). Digits are produced in the
  NEGATIVE domain, so the whole value is never negated: `i64::MIN` has no positive
  counterpart (negating it overflows -> a Candor arithmetic fault), so a
  non-negative `n` is reflected to `0 - n` (safe) and a negative `n` passes
  through unchanged — `-i64::MIN` is never formed. For `m <= 0`, `m % 10 in
  -9..=0` (truncated division), so `48 - (m % 10)` is the low digit's code and
  `m / 10` carries. Digits go low-first into a fixed `[20]i64` buffer (an i64 is
  <= 19 digits), then pushed high-to-low — no String reversal. A fixed buffer,
  not a `write String` helper, because the interpreter's String-intrinsic
  dispatch (`arg0_is_string`) only recognizes a String argument that is a
  directly-owned local (or `read`/`write` of one), NOT one reached through a
  borrow parameter; so every `push` must target `fmt_i64`'s own owned local.
- THE `Show` CONVENTION. `interface Show { fn to_string(read self, a: Alloc)
  alloc -> String; }` — an ordinary 0009-style interface returning a freshly-built
  owning String (region-free: the result derives from `a`, not from `self`). It
  threads `Alloc` BY VALUE: `Alloc` is a `copy` handle meant to pass by value,
  which ALSO sidesteps a checker quirk — a `read`-mode interface-method parameter
  is double-borrowed at the call site (the param's decl type already carries the
  borrow and `check_arg_mode` re-adds one -> `borrow borrow Alloc`), a path never
  exercised before (existing interface methods take only `self` or scalar params).
  Witnessed by `impl Show for ShowInt` (a leaf i64, delegating to `fmt_i64`).
- COMPOSITION THROUGH A `T: Show` BOUND (single-file only, parallel to F2). The
  generic `impl[T: Show] Show for Opt[T]` rendering "Some(<inner>)"/"None" via the
  payload's own `to_string`, and a `show_it[T: Show]` free function, check and RUN
  single-file but do NOT resolve under the module-tree checker (E1002: "no method
  `to_string` on type parameter `T`" — a bound-method call inside a generic impl
  across the tree, unexercised until now). So, exactly as cross-type `?` is proven
  single-file (corelib finding F2), the `T: Show` composition — with the whole
  convention — lives in the single-file image `std_fmt.cnr`, exercised by
  `tests/fmt.rs` (`show_composes_through_bound_{some,none}`,
  `show_through_generic_fn_bound`).
- GATE (isolation). `tests/fmt.rs` green — 12 tests: an `image_checks_clean`
  guard on `std_fmt.cnr`; `fmt_i64` on 0/positive/negative/`i64::MIN`/`i64::MAX`
  (str-equality on the `as_str` view) + a clean-run/`len==20` guard that MIN does
  not fault + a byte-trace of `-42` -> `[45, 52, 50]`; `Show` leaf + the `T: Show`
  composition. `tests/corelib.rs` (13), `tests/text.rs` (36), and the AOT/LLVM/
  Stage-D native gates untouched and green (nothing added to the backend corpus).
  clippy clean. No backend, self-host, `aot_runtime.c`, MIR, or checker change.
- NEXT SLICE (DELIVERED). `fmt`/`Show` are wired through `std_io` (the libc
  `write` path) in STD-IO-PRINT below: the non-generic `println_i64` and the
  generic `print_show[T: Show]` both produce real captured stdout, byte-for-byte
  identical on the tree-walker and the MIR engine.

## STD-IO-PRINT — the println path: `print_str`/`println_i64` wiring `fmt_i64` through `std_io` to real captured stdout (2026-07-12)

The second slice of "real std + I/O": the `fmt_i64` formatting primitive (STD-FMT)
wired through the `std_io` libc-`write` boundary (`fixtures/std_io/main.cnr`,
`write_all(fd: i32, s: read [u8]) -> IoResult` over the `sys_write` extern) to
produce REAL, host-captured stdout on the `foreign_io` shims. Source:
`tests/fixtures/std_print.cnr` (the print layer) composed with the `std_io` module
prefix (minus its `main`) and `fmt_i64`/`Show` from `std_fmt.cnr`; observed by
`tests/print.rs` on the host-backed shims. A follow-up carries TWO surgical
compiler fixes (below) — an `as_bytes` MIR lowering and a monomorphizer that keeps
`extern`/`export` — that make the whole path (including the generic `print_show`)
run byte-for-byte on the tree-walker AND the MIR engine.

- THE LAYER (capture-free, discharges nothing). `print_str(s: str) -> IoResult`
  retypes the view to bytes (`as_bytes`) and calls `write_all(stdout(), .)`.
  `println_i64(a: Alloc, n: i64) alloc -> IoResult` = `fmt_i64` into a fresh owned
  local `s`, then `push(write s, 10)` (the trailing newline pushed onto `s`'s OWN
  owned local — the STD-FMT owned-String rule), then `write_all` of `s`'s byte
  view. `Alloc` threads BY VALUE (the `copy` handle; a `read Alloc` param would
  double-borrow at `fmt_i64(read a, .)` -> `borrow borrow Alloc`, E0703). String
  text reaches `write_all` as `[u8]` via `as_bytes(as_str(read s))`, each step
  bound to a local (a call-expression used directly as a place does not lower).
- GATE (isolation). `tests/print.rs` green — 7 tests via the captured-stdout
  shims: an `image_checks_clean` guard; `println_i64` over 0/positive/negative/
  `i64::MIN` asserting the EXACT bytes `"0\n42\n-42\n-9223372036854775808\n"`;
  `print_str` verbatim (no newline); `print_str`+`println_i64` composing
  (`"n=7\n"`); `println_both_engines_byte_parity` (the whole println path on the
  tree-walker AND the MIR engine, byte-for-byte identical); and
  `print_show_through_boundary_renders_bytes` (the generic path — see below).
  `tests/fmt.rs` (12), `tests/std_io.rs` (13), and the AOT/LLVM/Stage-D native
  gates (which consume the same monomorphized program) untouched and green.
- BOTH-ENGINE PARITY — `as_bytes` NOW LOWERS ON MIR. `write_all` requires `[u8]`,
  and the only bridge from a built `String`'s text to `[u8]` is `as_bytes`
  (str -> [u8]). `as_bytes` is now recognized in `is_builtin` (`src/mir/build.rs`)
  and lowered in `lower_builtin_into` as a FREE retype: a `str`'s `{ptr@0, len@8}`
  fat pointer IS its `[u8]` byte view (design 0013 §1.3), so it copies the 16-byte
  header verbatim (`CopyVal`), mirroring the tree-walker's `move_bytes` — MIR-interp
  only, since the native backends already reject CollectionOp `String`/`as_str`.
  The former `println_mir_boundary_is_unsupported_as_bytes` pin is flipped to the
  positive `println_both_engines_byte_parity`.
- GENERIC PATH — `print_show[T: Show]` NOW SURVIVES THE `extern` BOUNDARY.
  `generics::monomorphize` (`src/generics.rs`) previously DROPPED
  `Item::Extern`/`Item::Export` via its item loop's `_ => {}` catch-all, so the
  instant the `Show` interface made the image "generic" the `sys_*` externs
  vanished and `write_all` faulted `Panic("unknown name \`sys_write\`")`. The loop
  now carries `Item::Extern`/`Item::Export` through UNCHANGED (they are never
  generic and need no substitution; not re-added elsewhere, so no duplication;
  the native gates already handle externs). `print_show[T: Show]` — rendering a
  `Show` leaf (`ShowInt` -> `"42"`) and a `T: Show` composition (`Opt[ShowInt]` ->
  `"Some(9)"`) straight to stdout — now runs byte-for-byte on both engines
  (`print_show_through_boundary_renders_bytes`).
- NEXT SLICE. Add the trailing-newline `println_show[T: Show]` and route
  `print_str`/`println_i64` and `print_show` into a single `Display`-style print
  surface; carry the `T: Show` module-tree resolution (STD-FMT finding F2 / E1002)
  so the convention leaves the single-file image.

## SHOW-DISPLAY — `println_show` unifies the print surface; collection `Show` blocked by the impl/coherence surface (2026-07-12)

The `Show`-broadening slice (STD-IO-PRINT "NEXT SLICE"): add the unified
`println_show[T: Show]` and extend the `Show` rendering convention to the `String`/
`Vec`/`Map` collection builtins. `println_show` LANDED; every collection-`Show`
goal is BLOCKED by an existing compiler restriction (no compiler surface was added
— design 0007 §2.3 / the CollectionOp builtins are the ground truth), so those are
reported here with evidence for a follow-up compiler slice to authorize.

- LANDED — `println_show[T: Show]`. The `Show` analog of `println_i64`:
  `x.to_string(a)` into a fresh owned local `s`, `push(write s, 10)` (the trailing
  newline onto `s`'s OWN owned local — the STD-FMT owned-String rule), then
  `write_all(stdout(), as_bytes(as_str(read s)))`. Source: the new generic print
  layer `tests/fixtures/std_print_show.cnr` (holding `print_show` + `println_show`,
  the `T: Show` sibling of the non-generic `std_print.cnr`); `tests/print.rs`
  `show_image` now composes that fixture (the inline `PRINT_SHOW` const is gone,
  test 7 unchanged). It runs the SAME generic-through-boundary path proven by
  STD-IO-PRINT (the `Show` interface makes the image generic; the extern survives
  mono; `as_bytes` lowers on MIR), so both engines produce identical bytes.
- GATE (isolation). `tests/print.rs` green — 9 tests: the 8 prior plus
  `println_show_string_both_engines`, asserting `impl Show for String` renders
  `"hello\n"` byte-for-byte on the tree-walker AND the MIR engine (the coherence-fix
  payoff). `tests/fmt.rs` (12), `tests/std_io.rs` (13), `tests/generics.rs` (44 —
  +2 coherence tests below), and the AOT/LLVM/Stage-D native gates green
  (byte-exact corpus unchanged — the fix only NARROWS overlap detection).
- LANDED (2026-07-12 coherence fix) — `impl Show for String` (design point 1).
  Two independent blockers, both resolved:
  (1) COHERENCE. `heads_overlap` (`src/generics.rs`) compared only the interface args
  and the target's TYPE ARGUMENTS, never the target CONSTRUCTOR NAME, so any two
  bare-nominal impls of one interface (both zero target args) falsely "overlapped" —
  `impl Show for String` collided with `impl Show for ShowInt` -> spurious `E1009`.
  FIX: `heads_overlap` now also requires the target constructor NAME to match
  (`atgt != btgt` short-circuits to no-overlap); the `Head` tuple carries the target
  name, and the E1009 caller passes `e.target`/`target`. Distinct nominals no longer
  overlap; SAME-head cases are UNCHANGED (a generic-vs-concrete `W[T]` vs `W[i64]`
  and a duplicate `A` twice still unify -> still `E1009`), so the narrowing opens no
  soundness hole (regression tests: `tests/generics.rs`
  `distinct_target_impls_of_one_interface_coexist_and_dispatch`,
  `generic_and_concrete_impls_on_same_head_overlap_are_rejected`,
  `duplicate_impl_is_rejected`).
  (2) OWNED-STRING-THROUGH-BORROW. `to_string(read self)` gets `self` as a `read`
  borrow, whose static type is `Borrow(String)`; the String intrinsic dispatch
  (`arg0_is_string`, checker + tree-walker) only recognizes a `Named("String")`, so
  `as_str(read self)` is `E0103 unknown name as_str` on BOTH engines. RESOLVED IN
  THE FIXTURE (no compiler change): `as_str(read self.*)` — the explicit deref names
  the underlying `String` place directly (static type `String`), which both engines'
  dispatch and `string_base` (deref of the borrow value) accept. The rendered target
  `s` remains this fn's own owned local (the STD-FMT owned-String build rule).
- BLOCKED — `impl[T: Show] Show for Vec[T]` (design point 2), on THREE independent
  restrictions: (1) `Vec`/`Map` are compiler-known CollectionOps, not registered
  nominals, so an impl target `Vec[T]` is `E0102 unknown type Vec`
  (`resolve_tables`, `src/generics.rs`; only `String` is a registered struct).
  (2) The sanctioned free-fn fallback `vec_show[T: Show](v: read Vec[T], a: Alloc)`
  ALSO fails — in a generic signature `Vec[T]` is `E1004 \`Vec\` is not a generic
  type` (`resolve_gty`: `Vec` is not in `generic_types`), so a `Vec` cannot be
  abstracted over its element type at all; only a CONCRETE `Vec[ShowInt]` param
  resolves. (3) RESOLVED (2026-07-12) — the tree-walker's `expr_static_ty`
  (`src/interp/eval.rs`) now resolves a CALL receiver to the callee's declared
  return type (free fn / nominal-dispatched method / `extern`; post-mono the sig
  is already concrete, so no substitution here), and the existing deref case
  recurses into it, so a method on a call-shaped receiver (`get(read v, i).*` or a
  bare `f(x)`) dispatches. Only an indirect fn-pointer call still has no
  statically-known callee (unchanged; it never worked). The CHECKER never had this
  gap — its method dispatch resolves the receiver via the full `check_expr` walk,
  which types a call as `sig.ret`. Proven by `method_dispatches_on_call_shaped_receiver`
  (tests/generics.rs), both engines. (A loop-carried owning `String` local also
  trips `E0309` unless pre-initialized before the loop.) So `Vec` STILL cannot
  render through `Show` without compiler work — but only on (1)/(2) below, not (3).
- BLOCKED (as predicted) — `Show for Map` (design point 3). The `Map[V]` CollectionOp
  surface is `map_new`/`insert`/`contains`/`get`/`len` only (`CollOp` in
  `src/mir/mod.rs`; `src/check/expr.rs`) — there is NO key iteration / `keys()`
  primitive, so entries cannot be enumerated to render. Skipped with that reason;
  no `keys()` primitive was invented.
- NEXT SLICE (compiler, needs authorization). Coherence step (a) — fix
  `heads_overlap` to compare the target constructor name — is DONE (above; unblocked
  `Show for String`). Remaining, to broaden `Show` to the collections: (b) make
  `Vec`/`Map` impl-able and element-abstractable (register them for impl targets and
  in `generic_types`, or add a blessed generic-collection impl path). Step (c) —
  tree-walker dispatch of a method on a call-shaped receiver (`get(..).*`) — is
  DONE (see BLOCKED (3) above; `expr_static_ty` gained the call-return-type case).
  Then carry the `T: Show` module-tree resolution (STD-FMT finding F2 / E1002) so
  the whole convention leaves the single-file image.

## MONO-SPAN-KEY — the cross-module monomorphization-key collision (memory-safety fix, 2026-07-12)

The REAL blocker for generics/`Show` leaving the single-file image — mis-attributed
to E1002 by the STD-FMT slice above. The whole-program monomorphizer's *shape map*
(`check::generics` records a per-node `Shape`; `generics::monomorphize` replays it to
lower each generic call/ctor/pattern to its concrete instance) was keyed by a bare
`span.start: usize`. But design 0008 merges every module's AST into one program
**without rebasing spans** (`modules::build_tree` just `extend`s items), and each
`.cnr` file is parsed with byte offsets numbered from ~0. So two nodes in DIFFERENT
modules routinely share a `span.start` and COLLIDE in the map (`insert` overwrites) —
a generic instantiation then resolves to the WRONG shape. Confirmed with a fixture of
two sibling modules whose `Opt::Some`/`unwrap_or` sites are byte-aligned: `ga`'s
`Opt[i64]` was monomorphized as `gb`'s `Opt[i32]` (an 8-byte payload read as 4 bytes),
returning a truncated/garbage value that even DIFFERED between the tree-walker
(`3567587348`) and MIR (`-727379948`) — an uninitialized/mis-sized-memory fault, hence
memory-safety, not a mere wrong answer.

- FIX (surgical, both replay sites + all five record sites). The key is now
  `pub type ShapeKey = (usize, usize)` = `(top-level item index in the merged program,
  node span.start)` (`src/generics.rs`). The item index disambiguates modules; within
  one item's file `span.start` stays unique (the exact invariant the single-file path
  already relied on), so the key is GENUINELY globally collision-free — NOT a wider hash
  that could still alias. The checker threads `cur_item` (set per top-level item, inherited
  by every def-site sub-checker and drop-hook check) so each recorded shape is keyed by its
  owning item; the monomorphizer threads `cur_item` too (set before rewriting each concrete
  item, each generic-fn/type instance via a name->item-index table, and each app-impl
  instance via a stored `def_item`), so an emitted instance replays its body under the
  DEF's own key namespace — matching how all instances already shared one parametric shape.
  Spans themselves are untouched (fault reporting, the `mir::mod` observable-order invariant,
  and the byte-exact self-host MIR differential all still see raw per-file `span.start`).
- UNIQUENESS ARGUMENT. Collision requires equal `(item, start)`. Equal `item` => same
  top-level item => same source file (0008: file = module) => `start` is the per-file byte
  offset, unique across that file's nodes (single-file monomorphization is correct today,
  which is precisely this property). Distinct `item` can never share a key regardless of
  span overlap. Hence no two distinct recording sites map to one key.
- GATE. New regression `tests/generics.rs::cross_module_generic_instantiation_no_span_collision`
  (fixture `tests/fixtures/modules/generic_span_collision`) — the byte-aligned two-module
  case — checks clean and returns `1000000000020` on tree-walker + MIR + native + native-opt.
  Full `cargo nextest` green incl. the self-host suite (33/33 — the self-host modules ARE a
  multi-module generic program, the exact scenario) and the AOT/LLVM/Stage-D native gates.
  clippy clean. No span, backend, or self-host source change.
- UNBLOCKS. Generics — and the `Show`/`fmt` convention — can now leave the single-file
  image and resolve across a module tree (the STD-FMT F2 / SHOW-DISPLAY "next slice" goal;
  the E1002 attribution there was the surface symptom, this was the cause). STILL OPEN and
  independent: the `impl Show for Vec[T]` / element-abstractable-`Vec` impl-target surface
  (SHOW-DISPLAY BLOCKED (1)/(2) — `Vec`/`Map` are CollectionOps, not impl-able nominals).

## STD-IO-ERROR — the I/O error story: Res/Opt combinators + a Res-typed IoError, `?` in-tree confirmed (2026-07-13)

The "real std + I/O" slice for error handling. Purely additive: the six seed
Res/Opt combinators (capture-free fn-pointer args, OBL-GENERICS-CLOSURE), a
structured `IoError` enum for `std_io`, and the in-tree `?` confirmation with the
stale-comment fix. No compiler, MIR, checker, backend, or `aot_runtime.c` change.

- RES/OPT COMBINATORS (`corelib/core/opt.cnr`, `core/res.cnr`). Added `and_then`
  to `Opt`; `map`/`map_err`/`unwrap_or`/`ok`/`ok_or` to `Res`. `ok_or` (an
  `Opt -> Res` bridge) lives in `core::res`, NOT `core::opt`, because `core::opt`
  is the ground floor and must not import `core::res`; the acyclic-DAG rule
  (design 0008 §3, enforced E0904) permits only the `res -> opt` edge, so both
  `Opt <-> Res` bridges (`ok`, `ok_or`) sit in `res`. `map`/`map_err`/`and_then`
  FORWARD every payload and stay non-`alloc` (ground floor); `ok`/`unwrap_or`/
  `ok_or` DISCARD a bare generic `E` (`ok_or` drops the eager `e: E` on the `Some`
  path), which is the design 0007 §3.4 opaque-drop `alloc` tax — so those three
  carry `alloc` (verified: without it, E0401). NO faulting `unwrap` was added: it
  is not in the required set and there is no clean surface fault primitive; the
  safe `unwrap_or` is the shipped form.
- COMBINATOR TEST (`corelib.rs::res_opt_combinators_run_to_sentinel`). A flattened
  image `corelib_combinators.cnr` drives `map`/`map_err`/`unwrap_or`/`ok`/`ok_or`
  to an exact sentinel (452) on ALL FOUR engines (tree-walker, MIR, native,
  native-opt). `tree_checks_clean` proves the same combinators check clean in the
  real seed tree. NEW BACKEND GAP (reported, not patched): `and_then`'s
  `f: fn(T) -> Opt[U]` is an INDIRECT (fn-pointer) call returning an AGGREGATE,
  which `mir/build.rs` rejects ("indirect/unknown aggregate call"), so `and_then`
  runs on the tree-walker only — `map`/`map_err`'s scalar-returning `f` lower on
  every engine. Pinned by `and_then_runs_on_tree_walker_mir_native_gap` (asserts
  MIR `Unsupported`); if the gap is closed the pin flips.
- RES-TYPED IOERROR (`std_io/main.cnr`). `IoError` is a structured ENUM,
  `Errno(i32)`, replacing the bare-`i32` failure payload. Chosen over named errno
  variants because the trust-clause externs expose only the syscall RESULT, never
  C `errno`; a NotFound/PermissionDenied mapping would need to read `errno`, which
  would change the externs (forbidden). `Errno(i32)` is native-safe (LLVM S2
  scalar enum, no `String`/allocator). Wrappers now return `IoResult {ok Ok(usize),
  Err(IoError)}`. Cross-type `?` widening `IoError -> AppErr` via `From` (design
  0007 §7.1) is proven ACROSS the I/O boundary on BOTH engines with captured I/O
  (`std_io.rs::write_app_question_*` / `open_app_question_*`): `?` unwraps
  `IoResult::Ok` and, on `Err`, widens into `AppResult::Err(AppErr::Io(..))` and
  early-returns. Cross-ENUM `?` (two distinct result enums, not one generic `Res`)
  composes on tree-walker/MIR/native — `try_from_conversion` keys on the non-`ok`
  PAYLOAD types, so it does not require a shared result enum.
- FORK (deciding authority) — the AUDITED std_io module stays NON-GENERIC.
  Part 2 literally asks the wrappers to return the GENERIC `Res[T, IoError]`. Doing
  so turns the boundary module generic, and `check::check_program_real_foreign`
  (`src/check/mod.rs:230`) routes any generic program through
  `generics::check_generic_program` and DISCARDS the (already-computed)
  `ForeignFnInfo` report — so `candor audit` loses its per-function foreign
  discharge/propagation report (`audit_enumerates_io_externs_trust_and_discharge`
  goes red: no "discharges foreign"). The `foreign_report` IS populated in the
  generic path (`check_fn` on the concrete wrappers) and only dropped by the
  return tuple, so exposing it is a small plumbing change — but that is a COMPILER
  change, deferred to the deciding authority per "do not touch the compiler unless
  a genuine primitive is missing (then STOP + report — do NOT build it)". Until
  then the audited wrappers use the non-generic `IoResult` (audit-green, full
  teeth) and the generic `Res`-typed `?`-widening is proven in the (non-audited)
  test probes. To ship generic `Res[T, IoError]` wrappers, authorize plumbing
  `foreign_report` through `check_generic_program`.
- `?` IN-TREE — E0712 NARROWED, stale comment fixed. The `res.cnr:10-16` comment
  claimed E0712 fired for ALL in-tree `?`; that is STALE. `src/check/expr.rs`
  `check_try` (~:1003-1033) resolves same-type `?` (Named==Named / App==App) and
  cross-type `?` (`try_from_conversion` via a `From` impl matched by base name
  across the tree's qualified names); E0712 now fires ONLY when neither holds — the
  enclosing return error type differs AND no `impl From[E1] for E2` is in scope.
  The comment is rewritten to cite this. Confirmed by: multi-file same+cross `?`
  (`corelib_question` / `question_operator_across_tree`, already green post
  MONO-SPAN-KEY), single-file cross-type (`cross_type_question_works_single_file`),
  and the NEW narrowed-negative `question_without_return_or_from_is_e0712`.
- GATE. `tests/corelib.rs` (16), `tests/std_io.rs` (16) green; the std_io AOT +
  LLVM native gates (`gate_{aot,llvm}_native_io_{real_libc,open_error}`) rebuild
  the new `IoResult`/`IoError` demonstrator byte-exact (exit + stdout). Full
  `cargo nextest` green; clippy clean. The allocator direction remains the
  separate deciding-authority fork.

## P17-AUDIT-GENERIC — `candor audit` keeps full teeth on generic boundary modules (2026-07-13)

RESOLVES the STD-IO-ERROR "FORK (deciding authority)" note above: the plumbing
it deferred is now done. `check::check_program_real_foreign` (`src/check/mod.rs`)
previously routed any generic program through `generics::check_generic_program`
and returned `Vec::new()` for the `ForeignFnInfo` effect-reach — so a boundary
module that was ALSO generic (any `interface`/`impl`/generic `fn`) reported an
EMPTY foreign discharge/propagation to `candor audit`, losing its teeth exactly
where a generic `Res[T, IoError]` I/O wrapper would live.
- WHAT `foreign_report` IS: one `ForeignFnInfo { name, boundary, discharges,
  propagates }` per checked function — the design 0011 §2 discharge decision,
  which depends only on source-level facts (the fn's `boundary`-module status,
  its `foreign` mark, and the ground-source `extern` calls in its body). It is a
  SOURCE-level property, independent of monomorphization.
- FIX (same fidelity, not weaker): the generic checker already runs
  `check_fn_with_sig` on every function — concrete fns, generic def-site fns,
  impl methods, and drop hooks — each pushing its `ForeignFnInfo`. Those inner
  def-site `Checker`s dropped their `foreign_report`; they now propagate it back
  to the outer checker exactly as they already propagate `diags`/`insts`/`shapes`.
  `check_generic_program_own` returns the report (`GenericForeignCheck`), and
  `check_program_real_foreign` threads it through the new
  `check_generic_program_foreign` instead of substituting empty. The discharge
  decision is computed by the IDENTICAL `ForeignEffect::resolve` on the IDENTICAL
  per-fn accumulator, so a generic wrapper's entry is bit-for-bit the entry the
  non-generic path would produce — no info is lost pre-monomorphization.
- REGRESSION: `tests/foreign.rs::audit_generic_boundary_keeps_effect_reach` over
  `tests/fixtures/ffi_audit_generic/` — a boundary module with a GENERIC
  discharging wrapper (`safe_len[T]`) and a GENERIC propagating one
  (`thin_raw[T] foreign`); asserts the audit names both externs, their trust
  predicates (`valid_nul_terminated`/`no_retain`), classifies `main::safe_len`
  as "discharges foreign" and `main::thin_raw` as "propagates foreign
  (undischarged)", and counts one undischarged wrapper. Pre-fix this reported an
  empty effect-reach.
- PAYOFF DEFERRED: making a real `std_io` wrapper generic is left untouched — the
  audited `std_io` module changes codegen/monomorphization and feeds the AOT/LLVM
  native byte-exact gates, so flipping it generic balloons scope beyond this fix.
  The regression fixture already proves a generic boundary module keeps full
  teeth. The allocator direction remains the separate deciding-authority fork.
- GATE. Full `cargo nextest` green (652 tests, incl. `tests/foreign.rs`,
  `tests/std_io.rs`, generics, and the native gates); clippy clean.

## FREELIST-ALLOC — a reclaiming first-fit free-list allocator on the existing vtable (2026-07-13)

The first allocator that RECLAIMS: a fixed-window first-fit free-list `std`
module (`tests/fixtures/corelib/std/freelist.cnr`), a NEW impl on the EXISTING
`std::alloc` vtable (design 0001 §6.1) — NO ABI change. Structural sibling of
`std::bump`: same caller-provided window `[base, end)`, state in `ctx`, vtable
construction, and the same liveness `unsafe` justification — but `free` reclaims
and `alloc` reuses.
- STATE (`ctx`): `FreeList { next: usize, end: usize, head: rawptr u8 }` — the
  bump frontier, the window end, and the free-list head (null when empty).
- FREE BLOCK via the rawptr valve (design 0001 §3.4, forbidding a borrow-typed
  field): each freed block stores a `FreeBlock { next: rawptr u8, size: usize }`
  header IN ITS OWN memory (`ptr_write`/`ptr_read` of the header struct). Minimal,
  justified rawptr manipulation exactly as §4.2 intends.
- `alloc`: round the request up to a header-sized, `align`-ed span (`block_span`,
  applied identically by `free` so the recorded capacity matches); first-fit walk
  of the ADDRESS-ORDERED free list, unlinking the first block whose stored size
  fits. SPLITTING (landed): when the fitting block's excess >= MIN_SPLIT (32 = a
  16-byte `FreeBlock` header + a 16-byte minimum block) the front `need`-sized
  piece is handed out and a NEW `FreeBlock` header for the trailing remainder is
  written into the block's own tail and takes the block's slot in the ordered list
  (remainder addr is > cur and < the successor, so order + non-adjacency hold);
  below MIN_SPLIT the whole block is returned (bounded internal fragmentation). On
  no fit, carve from the bump frontier; if the frontier would pass `end`, return
  null (BoxResult::oom contract, §6.2).
- `free`: keep the free list ADDRESS-ORDERED. Walk to the insertion point (`prev`
  = last block below the freed address, `cur` = first above), then COALESCE
  (landed, FORWARD + BACKWARD): forward-merge `cur` when it begins exactly at
  `addr + cap`, backward-merge into `prev` when `prev` ends exactly at `addr`;
  otherwise splice the (possibly forward-merged) block in at its ordered position.
  Address ordering both makes first-fit deterministic and gives both-sided
  coalescing with NO boundary-tag footer or header redesign — the `FreeBlock
  { next, size }` layout is unchanged. A merge fires ONLY on an exact byte-span
  boundary (`addr + size == neighbour addr`), where sizes are the identical
  `block_span` rounding used at alloc, so a merge can only ever join two
  PHYSICALLY-adjacent free blocks — never overlap live memory nor bridge a gap.
  The invariant "no two free blocks are physically adjacent" is established on the
  empty list and preserved by every free (merges on contact) and every split
  (remainder stays strictly inside the old block's span).
- HANDLE: `mk_alloc(state: write FreeList) -> Alloc` builds
  `Alloc { ctx: addr_of_mut(state), vt: addr_of(FREELIST_VT) }` inside the §6.1
  liveness-obligation `unsafe` justification, mirroring bump.
- NATIVE: the rawptr-threaded free list LOWERS NATIVELY — no gap. `ptr_read`/
  `ptr_write` of the two-field `FreeBlock` (with its rawptr field) from a heap
  block, and of the three-field `FreeList` from `ctx`, lower on every engine
  (same shape as `11_1_allocator`'s Pool, already native). One MIR nit surfaced:
  `box`'s payload type must be inferable from a `let`-bound local, not an inline
  arithmetic expression (`let payload: i64 = i * 10;` then `box(read a, payload)`).
- GATE (all engines): `tests/freelist.rs` — REUSE (`freelist_reuse.cnr`: a 16-byte
  one-block window serves five sequential boxes only because drop reclaims and
  alloc reuses; bump-only would OOM on box #2 — RET 100), OOM (`freelist_oom.cnr`:
  zero-headroom window takes the `BoxResult::oom` arm — RET 42), and a non-box
  direct `alloc`/`free` drive (alloc two, free one, alloc again reuses the freed
  address while the live block is untouched — RET 111), SPLITTING
  (`freelist_split.cnr`: a 160-byte window carved into one block then freed with
  the frontier exhausted; 9 sixteen-byte allocations succeed by splitting that one
  block, where a no-split allocator yields 1 then OOMs — RET 9), and COALESCING
  (`freelist_coalesce.cnr`: a 192-byte window fragmented into three adjacent
  64-byte blocks, freed ends-then-middle so freeing the middle merges both sides
  into one 192-byte block; the subsequent 192-byte alloc succeeds and its span is
  written end-to-end, where without coalescing it OOMs — RET 111). All asserted
  byte-exact on the tree-walking oracle, the MIR interpreter, and Cranelift native
  no-opt + opt. The box fixtures AND the split/coalesce fixtures live in
  `tests/fixtures/run/`, so the Cranelift-AOT ELF (`tests/aot.rs`) and LLVM
  clang-`-O2` ELF (`tests/llvm.rs`) full-corpus gates and the four-engine
  `tests/stage_d.rs` gate cover them transitively — six engines agree.
- `realloc` LANDED (see REALLOC-ABI below, 2026-07-16): the `std::alloc` vtable
  now carries a third slot `realloc(ctx, ptr, old_size, new_size, align) -> ptr`;
  the freelist grows in place (bump-frontier extend or adjacent-free absorb, with
  tail-split) else moves+copies. DEFERRED: best-fit — splitting + forward/backward
  coalescing landed, so the allocator no longer fragments under churn; first-fit
  (not best-fit) is retained as the simplest policy the split/coalesce tests prove
  non-fragmenting.
- The corelib module tree grew 8 -> 9 modules (`std::freelist` discovered +
  checked clean by the module-tree checker); `tests/stage_c.rs`'s incremental-
  build module counts updated 8 -> 9 accordingly.
- Full `cargo nextest` green (657 tests); clippy clean. This is the foundation
  for native growable collections + buffered I/O.

## NATIVE-COLLECTIONS-S1 — `String` (New / Append / AsStr) landed native on BOTH backends (2026-07-13)

The first native collection slice (design 0013, Path A: native intrinsic lowering).
The three non-UTF-8 `String` `CollectionOp` arms — `New`, `StringAppend`,
`StringAsStr` — are lowered INLINE in both native backends, mirroring
`mir::interp::collection_op` byte-for-byte (the MIR interpreter is the native
oracle, `src/mir/mod.rs`). Purely additive: no change to the `String` representation
(the 5-word header `{buf@0, len@8, cap@16, ctx@24, vt@32}`, `src/resolve.rs`), the
checker, or the MIR. `aot_runtime.c` reused UNCHANGED — NO new runtime symbol.

- WHAT LANDED (`src/backend/lower.rs` Cranelift + `src/backend/llvm.rs` LLVM, in
  lockstep). `collection_op` handles `New` (write `buf=0/len=0/cap=0` and `ctx`/`vt`
  read from the `Alloc` handle exactly as `box_op` does), `StringAsStr` (write the
  16-byte `{buf, len}` fat pointer — the same fat pointer the S1/S5 str/slice
  machinery already carries), and `StringAppend` (`string_reserve(need)` then
  `rt_copy` the view's `len` bytes into `buf+len`, bump `len`). The shared
  `string_reserve` growth helper is the reusable substrate: when `len+need > cap`,
  `newcap = (len+need).max(cap*2).max(8)` (byte-exact with the interp cap-growth
  formula), `alloc` a new buffer through the carried `Alloc` vtable (the `box_op`
  alloc path), `rt_copy` the existing `len` bytes over, `free` the old buffer, then
  update `buf`/`cap`. OOM (a null alloc return) FAULTS `Panic` at the append span,
  matching the interp. No `nsw`/`nuw` (wrapping `add`/`mul`; `.max` via `icmp`+
  `select`). New runtime-length `rt_copy`/`free`/`alloc` helper variants were added
  (the buffer size is dynamic, unlike `box_op`'s compile-time size) — pure IR
  emitters, no C-runtime symbol.
- GATE (`tests/string_native.rs` + `tests/fixtures/run/string_native.cnr`):
  `string_new` over the reclaiming free-list allocator, then empty (`len 0`), a
  single sub-cap append (`"hi"`), and four appends crossing the initial cap
  (`5 -> 10 -> 15 -> 20` bytes, forcing `string_reserve` growth more than once);
  content read back through `as_str` + `as_bytes` byte index (a native `str` has no
  direct index surface). Asserted BYTE-IDENTICAL — trace `[0, 2, 104, 105, 20, 97,
  107, 116]`, ret `20` — under the oracle, the MIR interpreter, and Cranelift native
  no-opt + opt. The fixture lives in `tests/fixtures/run/`, so `tests/aot.rs`
  (Cranelift ELF), `tests/llvm.rs` (clang-`-O2` ELF fifth-engine), and
  `tests/stage_d.rs` (four-engine) full-corpus gates cover it transitively — six
  engines agree.
- REUSE-READY: `New` and `string_reserve` are the `Vec`/`Map` substrate (identical
  5-word header + alloc-copy-free growth; the interp's `vec_reserve` differs only in
  `stride`/`align`/`.max(4)` and `map_reserve` in rehash). This slice is
  deliberately the three non-UTF-8 arms only.
- DEFERRED (next slices, in order): String `push` (UTF-8 scalar encode +
  `is_scalar_value` `Requires` backstop), then `Vec` (`push`/`pop`/`get`/`set`),
  then `Map` (FNV-1a + linear-probe `insert`/`contains`/`get`). GAP: a native
  `String`'s buffer is NOT yet freed on scope-end drop (the interp/`eval` special-
  case it; native `emit_drop` treats `String` as a plain struct = no-op) — a leak,
  not a divergence: it is observable-independent (nothing inspects allocator
  liveness after the final drop), and the gate is built to not depend on it. The
  drop-free (`alloc_on_drop` for `String`/`Vec`/`Map` in `emit_drop`) should land
  with the collection-drop slice.
- Full `cargo nextest` green (658 tests, was 657 + this gate); `--profile fast`
  green (538); `tests/text.rs` (36) / fmt / print / std_io interp gates unchanged;
  clippy clean.


## NATIVE-COLLECTIONS-S2 — `String::push` (UTF-8 encode + scalar-value backstop) landed native on BOTH backends → `String` is now FULLY native (2026-07-13)

The second native collection slice (design 0013, Path A). The one remaining `String`
`CollectionOp` arm — `StringPush`, the sole UTF-8 op — is lowered INLINE in both
native backends, mirroring `mir::interp::collection_op`'s `StringPush` (and its
`utf8_encode_scalar`) byte-for-byte. Purely additive: no change to the `String`
representation, the checker, or the MIR. `aot_runtime.c` reused UNCHANGED — NO new
runtime symbol (the encode is inline branches, not an `rt_utf8_encode` call).

- WHAT LANDED (`src/backend/lower.rs` Cranelift + `src/backend/llvm.rs` LLVM, in
  lockstep). `collection_op`'s `StringPush` arm: (1) validate the `u32` scalar —
  reject a surrogate (`0xD800..=0xDFFF`) or out-of-range (`> 0x10FFFF`) code point,
  exactly as `utf8_encode_scalar`, FAULTING `Requires` at the push call span
  (kind-code 5 + `span.start`/`span.end` through the shared `emit_fault`); (2)
  compute the UTF-8 length via a `select` chain (`<0x80 → 1`, `<0x800 → 2`,
  `<0x10000 → 3`, else 4); (3) `string_reserve(enc_len)` — REUSING the S1 growth
  helper unchanged; (4) branch per width to write the 1–4 encoded bytes at `buf+len`
  (lead byte `prefix | (c >> shift)`, continuation bytes `0x80 | ((c >> shift) &
  0x3F)`, bit-for-bit with `char::encode_utf8`); (5) bump `len` by `enc_len`. No
  `nsw`/`nuw` (wrapping `add`/`lshr`/`and`/`or`; `.max` via `icmp`+`select`). The
  encode is emitted INLINE in both backends (new `store_byte`/`utf8_lead`/`utf8_cont`
  IR-emitter helpers) — `aot_runtime.c` untouched.
- `String` IS NOW FULLY NATIVE: `New` / `StringAppend` / `StringAsStr` (S1) +
  `StringPush` (S2) all lower inline in both backends; no `String` op is MIR-interp-
  only anymore.
- GATE (`tests/string_native.rs` + `tests/fixtures/run/string_native.cnr`): the
  fixture pushes scalars encoding to 1/2/3/4 bytes — `'A'` (65 → `65`), `'é'` (233 →
  `195 169`), `'€'` (0x20AC → `226 130 172`), `'😀'` (0x1F600 → `240 159 152 128`) —
  the last push crossing a `string_reserve` growth (cap 8 → 16), read back through
  `as_str` + `as_bytes` byte index. Asserted BYTE-IDENTICAL under the oracle, MIR,
  and Cranelift native no-opt + opt (extended trace tail `10, 65, 195, 169, 226,
  130, 172, 240, 159, 152, 128`); the LLVM `-O2` fifth engine covers it transitively
  through `tests/llvm.rs`'s full-corpus gate (the fixture lives in `run/`). A FAULT
  gate (`string_push_non_scalar_faults_requires_all_engines`) pushes a surrogate
  (`0xD800`) and an out-of-range scalar (`0x110000`) and asserts the `Requires` fault
  kind AND span byte-exact across the oracle, MIR, and Cranelift no-opt + opt.
- GAP (unchanged from S1, still pending): a native `String`'s buffer is NOT yet freed
  on scope-end drop (native `emit_drop` treats `String` as a plain struct = no-op) —
  a leak, not a divergence (observable-independent; the gate does not depend on
  allocator liveness). The drop-free (`alloc_on_drop` for `String`/`Vec`/`Map` in
  `emit_drop`) should land with the collection-drop slice.
- DEFERRED (next slices, in order): `Vec` (same 5-word header + `string_reserve`-shape
  growth, adding `stride` + drop-on-overwrite: `push`/`pop`/`get`/`set`), then `Map`
  (FNV-1a + linear-probe `insert`/`contains`/`get`), then collection-drop-free.
- Full `cargo nextest` green (659 tests, was 658 + this fault gate); `--profile fast`
  green (539); `tests/text.rs` (36) / fmt / print / std_io interp gates unchanged;
  the aot / llvm / stage_d native corpus gates green; clippy clean.


## NATIVE-COLLECTIONS-S3 — `Vec[T]` (push/pop/get/set/len + drop-on-overwrite) landed native on BOTH backends over the shared substrate (2026-07-13)

The third native collection slice (design 0013, Path A). Every `Vec` `CollectionOp`
arm is now lowered INLINE in both native backends (`src/backend/lower.rs` Cranelift +
`src/backend/llvm.rs` LLVM-text), mirroring `mir::interp::collection_op` /
`vec_reserve` byte-for-byte over the same 5-word header `{buf@0,len@8,cap@16,ctx@24,
vt@32}` the S1/S2 `String` slices established.
- REUSED SUBSTRATE: `New` (the shared empty-header init) is unchanged; a new
  `vec_reserve` mirrors `string_reserve` but scales every byte size by the element
  `stride = round_up(size_of(elem), align_of(elem))` and grows `newcap =
  (len+need).max(cap*2).max(4)` (vs String's `.max(8)`, byte stride 1), allocating
  `newcap*stride` / freeing `cap*stride` / copying `len*stride` through the same
  `Alloc` vtable (alloc-new + rt_copy + free-old; no realloc). OOM faults `Panic`.
- OPS: `VecPush` (`vec_reserve(len+1)`, write the element at `buf+len*stride` via
  `rt_copy` for the exact `size_of(elem)` bytes — scalar OR struct — then bump `len`);
  `VecGet` (bounds-check `idx < len`, write the slot borrow `buf+idx*stride` to the
  `read elem` dst); `VecSet` (bounds-check, DROP the overwritten element via
  `emit_drop(slot, elem)` BEFORE the move, then `rt_copy` the new value);
  `VecPop` (empty → write `Opt::None` discriminant; else decrement `len`, write
  `Opt::Some` + `rt_copy` the last element into the `Some` payload — the Opt indices
  and payload offset are compile-time layout). `len` on a `Vec` is unchanged (an
  offset-8 word Load, not a `CollOp`). No `nsw`/`nuw`; `.max` via `icmp`+`select`.
- DROP-ON-OVERWRITE LANDED: `VecSet` runs the old element's drop glue (reusing
  `emit_drop`, the byte-exact twin of the interp's `drop_value`) in the correct order.
  For a hook-bearing struct element this fires the hook directly (`callref`, no glue
  table needed) and is proven byte-exact across all engines.
- BOUNDS FAULT: `idx >= len` faults `Bounds` at the index arg's span — the SAME
  `emit_fault`/`fault_if` the array/String faults use — byte-identical kind AND span.
- LLVM TIERING: `classify_tiers` now marks a `CollectionOp`'s result dst and its
  `VecPush`/`VecSet` value operand Tier-F (they are read/written through place
  ADDRESSES via `place_addr`, like Box/Subslice operands), so a wordy `read elem`
  borrow dst gets addressable flat storage. Cranelift stack-allocates every local, so
  it needed no tiering change. `aot_runtime.c` untouched.
- GATES: `tests/vec_native.rs` + `tests/fixtures/run/vec_native.cnr` (auto-scanned by
  the aot / stage_d four-engine and llvm fifth-engine corpus gates). The fixture,
  over the reclaiming FREE-LIST allocator, pushes a `Vec[i64]` past cap 0→4→8 (two
  `vec_reserve` growths), then `get`/`set`/`len`/`pop` (trace `6,10,60,99,60,5,5`),
  and a `Vec[Pair]` (stride 16 > 8, `rt_copy` element moves) with `push`/`get`/`set`
  (trace `1,10,30,40`), ret 5 — asserted BYTE-IDENTICAL under the oracle, MIR, and
  Cranelift no-opt + opt; LLVM `-O2` covers it transitively. A FAULT gate
  (`vec_get_set_out_of_bounds_faults_bounds_all_engines`) drives an OOB `get` AND
  `set` (index 5 on a 1-element Vec) and asserts the `Bounds` kind AND span byte-exact
  across the oracle, MIR, and Cranelift no-opt + opt. A DROP gate
  (`vec_set_drops_overwritten_element_all_engines` + `tests/fixtures/run/
  vec_drop_overwrite.cnr`) drains the Vec empty so the (deferred) scope-end collection
  drop is a no-op, isolating the overwrite drop: `set` over E{1} then draining E{2},
  E{9} traces `1,100,2,200,9` byte-identical across all five engines.
- GAP (deferred to the collection-drop slice, unchanged from S1/S2): a live `Vec`'s
  buffer + remaining elements are NOT freed/dropped on scope-end drop (native
  `emit_drop`/`needs_drop`/`walk_glue` have no `Type::App` arm; the interp's
  `drop_value` DOES via its `Vec` arm). This is a leak, not an observable divergence,
  as long as the fixture's element drops are not observed at scope end — hence every
  drop gate drains its Vec empty. Two sub-items fold into the NEXT slice: (1) the
  `Type::App("Vec")` arm in native `emit_drop`/`needs_drop` (per-element reverse drop
  + buffer free through the carried vtable); (2) `walk_glue` needs a `Vec` arm so a
  `Vec[Box[T]]` element's Box drop-glue is collected (today only hook-bearing / non-
  Box element drops are reachable at a `VecSet` overwrite — a `Vec<Box>` overwrite
  would silently skip the Box free; not exercised by any current fixture).
- Full `cargo nextest` green; `--profile fast` green; `tests/vec.rs` (interp) unchanged
  and now additionally native-runnable; text/fmt/print/std_io interp gates unchanged;
  the aot / llvm / stage_d native corpus gates green (they auto-scan the two new
  `run/` fixtures); clippy clean.
- DEFERRED (next slices, in order): `Map` (FNV-1a hash + linear-probe
  `insert`/`contains`/`get`) — LANDED, see NATIVE-COLLECTIONS-S4 below; then
  collection-drop-free (`alloc_on_drop` for `String`/`Vec`/`Map` in `emit_drop`).


## NATIVE-COLLECTIONS-S4 — `Map[V]` (Insert / Contains / Get) landed native on BOTH backends (2026-07-13)

The fourth and LAST collection slice. The three hash-`Map` `CollectionOp` arms —
`MapInsert`, `MapContains`, `MapGet` — are lowered INLINE in both native backends,
mirroring `mir::interp::collection_op`'s Map arms byte-for-byte (the MIR
interpreter is the native oracle). Purely additive: no change to the `Map`
representation (the same 5-word header `{buf@0, len@8, cap@16, ctx@24, vt@32}`,
bucket buffer, `Alloc` handle), the checker, or the MIR. `aot_runtime.c` reused
UNCHANGED — NO new runtime symbol (the FNV hash + probe are inline emission, like
the String UTF-8 and Vec stride arithmetic). This makes ALL THREE collections
(`String`, `Vec`, `Map`) native on both backends.

- BACKING LAYOUT (mirrored exactly): open-addressed bucket buffer, entry stride
  `round_up(24 + size_of(V), 8)`, each bucket `{state@0 (0=empty/1=occupied),
  keyptr@8, keylen@16, value@24}`. `cap` = bucket count (a power of two: initial 8,
  then x2). `len` at header offset 8 is the live entry count — `len(read m)` is an
  offset-8 Load, NOT a `CollOp` (unchanged). No tombstones (no `remove` op ships).
- FNV-1a HASH (mirrored bit-for-bit vs `mir::interp::map_hash`): 64-bit, offset
  basis `0xcbf29ce484222325`, prime `0x100000001b3`, folding each key byte `h =
  (h ^ byte) * prime` (wrapping). Emitted as an inline byte loop. The LLVM basis is
  written as its signed-i64 two's-complement (`-3750763034362895579`) — identical
  64 bits, xor/mul being width-agnostic. Verified identical under `clang -O2`.
- LINEAR PROBE (mirrored bit-for-bit vs `map_find`/`map_find_empty`): start index
  `hash & (cap-1)`, step `(idx+1) & (cap-1)`, stop at the first empty bucket
  (`state==0`). A bucket matches iff `keylen == klen` AND the `klen` key bytes are
  equal (an inline `mem_eq` byte loop). Same probe order in insert, lookup, and the
  rehash re-insert, so it matches the interp's observable placement.
- OPS: `MapInsert` — hash+probe (`map_find`); if PRESENT, DROP the displaced value
  via `emit_drop(slot+24, V)` BEFORE the move then `rt_copy` the new value (value
  drop-on-overwrite, the exact `VecSet` pattern); if ABSENT, `map_reserve` (grow +
  rehash at load factor 3/4), then own a heap byte-copy of the key via the carried
  `Alloc` (`alloc(klen,1)` + `rt_copy` of the key bytes; OOM faults `Panic`), write
  `state=1/keyptr/keylen`, `rt_copy` the value, bump `len`. `MapContains` — hash+
  probe, store the membership bool (1-byte `Bool`, matching the interp's
  `write_int(.., Bool)`). `MapGet` — hash+probe; ABSENT faults `Bounds` at the key
  arg's span; PRESENT writes the `slot+24` value borrow to the `read V` dst.
- REHASH (mirrored bit-for-bit vs `map_reserve`): grows when `!(cap!=0 &&
  (len+1)*4 <= cap*3)`; `newcap = cap==0 ? 8 : cap*2`; alloc the new buffer, ZERO it
  (an inline 8-byte-word loop — the bucket `state` words must read empty; the
  vtable does not zero), RE-INSERT every live old entry by re-probing into the new
  buffer (`map_find_empty`), then free the old buffer. Re-probe order matches the
  interp, so post-rehash placement is byte-identical.
- LLVM LOWERING NOTE: the probe / hash / mem_eq / rehash loops are emitted as
  textual-IR blocks with `phi` induction variables (forward-referenced loop-carried
  temps), each loop entered through a named pre-block so the `phi` has a concrete
  predecessor label. `classify_tiers` now marks a `Map` op's `key` (and `MapInsert`'s
  `value`) operand Tier-F — they are read through place ADDRESSES via `place_addr`.
  Cranelift uses `BlockArg` block parameters for the same induction variables and
  stack-allocates every local, so it needed no tiering change.
- GATES: `tests/map_native.rs` + `tests/fixtures/run/map_native.cnr` (auto-scanned
  by the aot / stage_d four-engine and llvm fifth-engine corpus gates). The fixture,
  over the reclaiming FREE-LIST allocator, inserts 8 keys chosen to COLLIDE in the
  cap-8 probe chain (`a/i/y` at bucket 4, `h/x` at bucket 7) so the probe loop is
  genuinely exercised, crosses the 3/4 load factor on the 7th distinct insert to
  trigger a GROWTH REHASH to cap 16 (where `i/y` and `h/x` still collide), OVERWRITES
  an existing key (`i`), then `get`/`contains`/`len` — trace
  `8,8,1,99,3,4,5,6,7,8,1,0`, ret 8, asserted BYTE-IDENTICAL under the oracle, MIR,
  and Cranelift no-opt + opt; LLVM `-O2` covers it transitively. A FAULT gate
  (`map_get_missing_faults_bounds_all_engines`) drives a `get` on an absent key and
  asserts the `Bounds` kind AND span byte-exact across the oracle, MIR, and Cranelift
  no-opt + opt.
- VALUE DROP-ON-OVERWRITE: LANDED in code in both backends (the `emit_drop` before
  the value move, the byte-exact twin of the interp's `drop_value` in `MapInsert`,
  structurally identical to the cross-engine-gated `VecSet` path). It is covered by
  `tests/map.rs`'s oracle test `map_overwrite_drops_old_value_once`. A cross-engine
  NATIVE gate for it is NOT feasible this slice: a hook-bearing value that survives
  in the Map is dropped at scope end by the interp (its `drop_value` `Map` arm) but
  NOT by native `emit_drop` (deferred; see GAP), and a `Map` has no `remove`/`pop` to
  drain the survivors — so a tracing survivor would diverge at scope end. Folds into
  the collection-drop slice (S5).
- GAP (deferred to the collection-drop slice, unchanged from S1/S2/S3): a live
  `Map`'s bucket buffer + owned key byte-copies + remaining values are NOT
  freed/dropped on scope-end drop (native `emit_drop`/`needs_drop` have no
  `Type::App("Map")` arm; the interp's `drop_value` DOES). This is a leak, not an
  observable divergence, as long as the value type needs no drop (the `map_native`
  fixture uses `i64` values) — so the scope-end `Map` drop is observationally a
  no-op in every engine. The `Type::App("Map")` arm in native `emit_drop`/
  `needs_drop`/`walk_glue` (free each key copy + drop each live value + free the
  buffer through the carried vtable) folds into S5, together with the `Vec`/`String`
  buffer frees.
- Full `cargo nextest` green; `--profile fast` green; `tests/map.rs` (interp,
  incl. collision/rehash/overwrite-drop) unchanged and now additionally native-
  runnable; text/fmt/print/std_io/vec/string interp gates unchanged; the aot / llvm /
  stage_d native corpus gates green (they auto-scan the new `run/` fixture); clippy
  clean.
- DEFERRED (next slice): collection-drop-free — the `alloc_on_drop` `Type::App`
  arms for `String`/`Vec`/`Map` in native `emit_drop`/`needs_drop`/`walk_glue`,
  freeing each collection's heap buffer (and, for `Map`, its owned key copies) and
  running per-element/per-value drop glue at scope end, so a live collection no
  longer leaks and a hook-bearing collection's scope-end drops become cross-engine
  observable.

## NATIVE-COLLECTIONS-S5 — collection DROP-FREE: native `emit_drop` frees String/Vec/Map buffers (and drops live elements/values, frees Map keys) at scope end → the native-collections arc CLOSES (2026-07-13)

The fifth and CLOSING collection slice — the deferred GAP from S1–S4. Native
`emit_drop` / `needs_drop` / `walk_glue` gain the compiler-known collection arm in
BOTH backends (`src/backend/lower.rs`, `src/backend/llvm.rs`), so a live
`String`/`Vec`/`Map` frees its heap buffer (and drops its remaining elements /
values, and frees a `Map`'s owned key byte-copies) at the schedule's drop site —
BYTE-EXACT with the interpreter's `alloc_on_drop` (`mir::interp::drop_value`, the
oracle). Purely additive: no representation, checker, MIR, or `aot_runtime.c`
change — only backend `emit_drop` lowering + tests. This closes the arc: `String`,
`Vec`, `Map` are now FULLY native AND memory-owning on both backends.

- THE ARM (mirror of `mir::interp::drop_value`, byte-exact order + free args):
  `needs_drop` and the standalone `type_needs_drop` return true for
  `Type::App("Vec"|"Map")` and `Type::Named("String")` (each owns a buffer); the
  `String` arm PRECEDES the generic-struct arm (`String` is a synthesized nominal
  struct). `emit_drop` dispatches to `drop_vec`/`drop_map`/`drop_string`.
  - `Vec[T]`: if `buf != 0`, drop each live element in REVERSE index order
    (`(0..len).rev()`, guarded by `needs_drop(T)`), at `buf + i*stride`
    (`stride = round_up(size_of T, align_of T)`), then
    `free(buf, cap*stride, align_of T)`.
  - `Map[V]`: if `buf != 0`, for each slot `0..cap` FORWARD, if occupied
    (`state==1`) free its owned key bytes (`free(keyptr, keylen, 1)`) THEN drop its
    value at `slot+24` (`emit_drop V`); then `free(buf, cap*stride, 8)`
    (`stride = round_up(24 + size_of V, 8)`). The key free runs for every occupied
    slot regardless of `V` (POD keys), matching the interp.
  - `String`: if `buf != 0`, `free(buf, cap, 1)` — bytes are POD, no element drops.
  Every free goes through the SAME carried `Alloc` vtable handle the collection
  allocated with (`ctx@24`, `vt@32`), reusing the box-op `call_free`/`call_free_val`
  path. No `nsw`/`nuw` on any index/offset arithmetic.
- MEMORY SAFETY (the free happens EXACTLY once, only for a real buffer):
  - The `buf != 0` guard means a `new`'d-but-never-allocated collection frees
    nothing (the allocator was never called) — matches the interp.
  - Element/value drops PRECEDE the buffer free (so a `Vec[Box]`/`Map[Box]` frees
    each inner Box before releasing the block it lived in).
  - A MOVED-OUT collection is not double-freed: `mir::build::emit_drop` emits NO
    `Drop` statement when the whole value is moved (empty path in the move mask),
    exactly as for `Box` — so the caller-side drop is already suppressed; the callee
    that now owns the value drops it. No drop flags. Tested explicitly.
- `walk_glue`/`type_needs_drop` gained `Type::App("Vec"|"Map")` arms (recurse into
  the element/value types) so a `Box[Vec[..]]` pointee gets drop glue and a
  `Vec[Box[T]]`/`Map[Box[T]]` whose inner box itself needs drop reaches that glue.
  A plain `Vec[Box[i64]]` needs no glue (the element `emit_drop` → `drop_box` frees
  the box inline; `i64` needs no glue), so this is the general, not the common, path.
- LLVM LOWERING NOTE: the element/slot loops are textual-IR blocks with `phi`
  induction variables entered through a named pre-block; the loop-carried next value
  is computed in a dedicated back-edge block so the `phi` predecessor is a concrete
  label even though the body's `emit_drop` may open its own blocks. Cranelift uses
  `BlockArg` block parameters for the same loops. No tiering change (drop reads flat
  addresses already available).
- GATES (all byte-identical under oracle / MIR / Cranelift no-opt + opt, and LLVM
  `-O2` transitively via the auto-scanned `run/` fixtures — `tests/collection_drop.rs`
  + `tests/fixtures/run/coll_drop_*.cnr`):
  - `coll_drop_balance` — a grown String + Vec + Map dropped at scope end return the
    counting allocator to `live == 0` (every buffer + Map key freed once).
  - `coll_drop_box_elems` — `Vec[Box i64]` / `Map[Box i64]`: each inner Box freed by
    the element/value drop before the buffer → `live == 0`.
  - `coll_drop_empty` — `new`'d-but-unallocated (`buf == 0`) collections drop with NO
    free, no fault, no double-free → `live == 0`.
  - `coll_drop_moved` — a Vec/String/Map moved into a callee is dropped once by the
    callee; the move mask suppresses the caller drop → `live == 0` (a double free
    would drive it negative).
  - `coll_drop_reuse` — a reclaiming FREE-LIST with a 2048-byte window services 200
    build-and-drop `Vec[i64]` cycles (each ~128-byte buffer) ONLY because every
    dropped buffer is returned and reused (`ret 21700`); a leak OOM-faults
    `vec_reserve` within a dozen iterations.
  - `coll_drop_order_vec` — a drop-hooked element traces its id; the Vec drop runs
    them in reverse index order (`trace 5,4,3,2,1`).
  - `coll_drop_order_map` — a drop-hooked value traces its id; the Map drop runs every
    occupied slot's value in slot order (`trace 33,44,11,22`), asserted against the
    oracle's observed order.
  Teeth verified: neutering the `emit_drop` collection arm makes `coll_drop_balance`
  return `live == 11`, `coll_drop_reuse` OOM-panic, and `coll_drop_order_vec` lose its
  trace — each diverging from the oracle.
- Full `cargo nextest` green (671 tests, incl. the self-host interp/lower/checker/
  analyses/codegen tiers and the aot/llvm/stage_d native corpus gates that auto-scan
  the 7 new fixtures); `--profile fast` green; the S1–S4 native gates
  (string/vec/map_native) and the text/vec/map interp gates unchanged; clippy clean.
- GAP CLEARED: the S1/S2/S3/S4 deferred collection-drop leak is closed. Native
  collections now fully own their memory — allocate AND free through the carried
  `Alloc`, byte-exact with the interpreter, on both the Cranelift and LLVM backends.
  Nothing deferred from this slice.

## STD-IO-FILE — whole-file String I/O over the std_io boundary: `read_file`/`read_to_string`/`write_str`, with the read path pinned tree-walker-only (2026-07-13)

The native-`String` payoff for I/O: read a whole file into a growable `String`,
write a `String` out, over the existing `std_io` boundary. Purely additive — no
extern/trust-clause, compiler, MIR, checker, backend, or `aot_runtime.c` change.
The three functions live as a probe prelude (`FILE_API`) in `tests/std_io.rs`, NOT
baked into the audited `main.cnr` module, for the lowering reason below.

- THE THREE FNS. `read_to_string(a: read Alloc, fd: i32) alloc -> StrIoResult`
  loops `read_into` into a fixed `[64]u8` stack buffer: `Ok(0)` is EOF (return the
  owned String), `Ok(n)` appends `buf[0..n]`, `Err` propagates via `?`. Growth is
  the native `String` append (`string_reserve`). `read_file(a, path) alloc ->
  StrIoResult` = `open_read(path)? -> read_to_string(a, fd)? -> close(fd)`.
  `write_str(fd, s: read String) -> IoResult` = `write_all(fd, as_bytes(as_str(read
  s.*)))`.
- THE BOUNDED STR-VIEW PRIMITIVE. Appending exactly the n read bytes needs a
  `[u8] -> str` reinterpret: `subslice(slice_of(buf), 0, n)` yields the bounded
  `[u8]` (design 0013 byte-view; the `subslice` builtin), then `str_from_unchecked`
  reinterprets it to a `str` (`append` requires a `str`). `str_from_unchecked`, NOT
  the checked `str_from`, is REQUIRED: a multibyte char may straddle a read
  boundary, so per-chunk UTF-8 validation would wrongly reject a whole that is
  valid. It is sound here — `append` only COPIES bytes (`bi_string_append` never
  re-validates) and the partial `str` is never exposed; the assembled String is
  valid UTF-8 iff the source is. Carries an `unsafe` justification (P1).
- REPORTED NATIVE GAP (read path is tree-walker only). `str_from_unchecked`,
  `str_from`, and `substr` are NOT in `mir::build::is_builtin`, so any program
  containing them is rejected at MIR/native lowering ("indirect/unknown aggregate
  call") — and MIR lowering is whole-program/eager, so a single such fn taints the
  whole image. This is the same class as STD-IO-ERROR's `and_then` aggregate-call
  gap. Consequence: `read_to_string`/`read_file` run on the TREE-WALKER only.
  Pinned by `read_to_string_mir_native_gap_is_unsupported` (asserts MIR
  `Unsupported`); if a native `[u8] -> str` (or a byte-append-to-String) intrinsic
  ever lands, the pin flips and the read tests promote to MIR/native. Per the
  standing rule, NO compiler surface was added to force it.
- `write_str` IS FULLY NATIVE. It uses only `as_str`/`as_bytes` (free retypes,
  both in the native path) + `write_all`, no str-view constructor, so it lowers on
  the MIR/native backends. Proven on BOTH engines by
  `write_str_writes_string_bytes_both_engines` (tree + MIR, captured stdout).
- fd LEAK ON THE ERROR PATH (noted). In `read_file`, a `read_to_string` error
  `?`-early-returns BEFORE `close`, leaking the fd. Accepted for this slice:
  closing on the error arm fights the owned-String-through-`?` flow; a clean close
  needs a match (not `?`) or a defer/scope-guard the edition lacks.
- RESULT TYPE. `read_to_string`/`read_file` return `StrIoResult { ok Ok(String),
  Err(IoError) }`, a String-carrying sibling of the module's non-generic `IoResult`
  (the audited module stays NON-GENERIC — see the STD-IO-ERROR fork). `?` moves
  `IoError` across it via an identity `impl From[IoError] for IoError`.
- GATE. `tests/std_io.rs` now 22 (16 prior + 6 new): round-trip (build a String,
  write it, read it back byte-equal via stdin->read_to_string->write_str->stdout),
  multi-fill (~300-byte multi-byte content > the 64-byte buffer forces the loop +
  repeated String growth, reassembled exactly), read_file (real open+read+close of
  a temp file), the error path (`read_file` of a missing path -> `Err` via `?`),
  the `write_str` both-engines proof, and the MIR-gap pin. Full `cargo nextest`
  green incl. the std_io AOT/LLVM native gates; clippy clean.

NEXT: a lines iterator + a buffered reader/writer. A native `[u8] -> str` (or a
byte-append-to-String) intrinsic would let the read path go native.

## STD-IO-FILE-NATIVE — the read path goes native: `str_from_unchecked` lowered as a pure retype, plus aggregate-`?` and block-arm aggregate init (2026-07-13)

Flips STD-IO-FILE's "REPORTED NATIVE GAP": `read_to_string`/`read_file` now run on
the tree-walker AND the MIR engine AND both native backends (Cranelift + LLVM),
byte-exact. Backend/MIR lowering only — no checker/semantic/extern change.

- STR-VIEW RETYPE. `str_from_unchecked` ([u8] -> str) is now in
  `mir::build::is_builtin` and lowers exactly like `as_bytes` (the str -> [u8]
  mirror): a `[u8]` and a `str` share the same 16-byte `{ptr@0, len@8}` fat pointer
  (design 0013 §4), so the unsafe (validation-skipping) variant just `CopyVal`s the
  16-byte header into the `str` destination. Backend-agnostic: `CopyVal` already
  lowers on both native backends, so no lower.rs/llvm.rs arm was needed (same as
  `as_bytes`).
- TWO ENABLING GAPS the read path also hit (whole-program eager lowering means
  `read_file` taints the image even when `main` only calls `read_to_string`):
  (1) `?` with an AGGREGATE ok-payload — `read_to_string(a, fd)?` unwraps a
  `StrIoResult` carrying an owned `String`. `lower_try` now moves an aggregate
  ok-payload into the caller's destination via `CopyVal` (the enum temp is never
  dropped, so it is a move, mirroring the tree-walker's payload-address reuse);
  word payloads are unchanged. (2) A BLOCK match-arm in aggregate position — the
  diverging `StrIoResult::Err(e) => { return ...; }` arm is now lowered by
  `lower_into` via `lower_block` (mirroring the value-position handling); a
  value-producing arm stays a bare expression, so `dst` is left unwritten only on
  the diverging path.
- GATES. `read_to_string_round_trip_byte_equal_tree_and_mir` (tests/std_io.rs)
  replaces the old `read_to_string_mir_native_gap_is_unsupported` pin: the stdin ->
  read_to_string -> String -> write_str -> stdout round-trip is byte-exact on the
  tree-walker AND the MIR engine through the same real shims. `gate_aot_native_read_file_string`
  (tests/aot.rs) and `gate_llvm_native_read_file_string` (tests/llvm.rs) compile a
  new self-contained fixture (`tests/fixtures/std_io_readpath/read_file_native.cnr`:
  the extern module + FILE_API + a `read_file`-of-a-real-file main) to a real ELF /
  clang -O2 binary calling real libc, run with the file's dir as cwd, and assert the
  stdout + exit byte equal the shim oracle. The fixture lives OUTSIDE the audited
  `fixtures/std_io` dir so `candor audit`'s extern count stays 4.
- DEFERRED (unchanged, tree-walker-only). `str_from` (the CHECKED [u8] -> Utf8Res:
  a full UTF-8 validation byte scan returning `Valid(str)`/`Invalid(offset)`) and
  `substr` (a char-boundary str slice with a `Bounds` fault on out-of-bounds OR a
  non-boundary offset) are NOT lowered: both need substantial new multi-block
  fault-edge / byte-scan machinery emitted byte-exact across the MIR interp AND
  both native backends (a dedicated `Substr`-style op or an `rt_` validation
  symbol), which balloons well past the read-path deliverable. Only
  `str_from_unchecked` (the read path's actual need) was landed; their text.rs
  tree-walker tests stay green.
- Full `cargo nextest` green (679 tests) incl. the std_io/text/native gates;
  clippy clean.

NEXT: the deferred `str_from`/`substr` native lowering; then a lines iterator + a
buffered reader/writer (the next file-I/O slice).

## STD-IO-FILE-LINES — native line-oriented file I/O: `split_lines`/`read_lines` over the native String/Vec (2026-07-13)

Rounds out the file-I/O layer with line splitting, fully native (tree-walker · MIR
· Cranelift · LLVM). Purely additive — no extern/trust-clause, compiler, MIR,
checker, backend, or `aot_runtime.c` change; built entirely on the already-native
`String`/`Vec[T]`/`str_from_unchecked` intrinsics.

- `split_lines(a: read Alloc, s: read String) alloc -> Vec[String]` byte-scans the
  String's UTF-8 bytes (via `as_bytes(as_str(read s.*))`) for the ASCII newline
  (10). '\n' is single-byte, so the scan needs no char-boundary logic: each line
  span `[start, end)` is a bounded `subslice` cut only at a newline (never inside a
  multibyte char), so it is itself valid UTF-8 — reinterpreted with
  `str_from_unchecked` and `append`ed into the line's OWN owned `String` local
  before `push`ing it into the result `Vec[String]` (the owned-String-through-borrow
  discipline). CONVENTION (tested): a trailing newline does NOT emit a final empty
  line (`"a\nb\n"` -> `["a","b"]`, `"a\nb"` -> `["a","b"]`); an interior empty line
  IS preserved (`"a\n\nb"` -> `["a","","b"]`); the empty string yields an EMPTY Vec.
- `read_lines(a: read Alloc, path: read [u8]) alloc -> LinesIoResult` = `read_file(a,
  path)?` (aggregate-`?` propagating `IoError`, the STD-IO-FILE-NATIVE path) then
  `split_lines(a, read s)`. `LinesIoResult { ok Ok(Vec[String]), Err(IoError) }` is a
  Vec-carrying sibling of the module's non-generic `IoResult` (the audited module
  stays NON-GENERIC), the `?` widening via the identity `From[IoError] for IoError`.
- TWO CHECKER/INTERP QUIRKS were originally worked around in this fixture; both are
  now FIXED at the compiler level (see COLLECTION-OP-ERGONOMICS below), and
  `sum_line_lengths` in the fixture was flipped to the DIRECT forms to prove it:
  (1) re-borrowing an already-`read Vec` param — `len(read v)` / `get(read v, i)` on a
  `v: read Vec[T]` param — now dispatches identically to the bare `v`; (2)
  `as_str`/`as_bytes` chained on a `get(v,i).*` result (`as_str(read get(read v, i).*)`)
  now type-checks and runs on all engines without the `fn(s: read String)` helper.
- GATES. `tests/lines.rs` (`split_lines_all_cases_all_engines`) drives the
  `tests/fixtures/run/split_lines.cnr` fixture — the five split cases plus a `fold`
  compose (sum of `["hello","world","!"]` line lengths = 11 over the 0009 Iter
  protocol) — asserting the exact 23-entry trace + ret byte-identical on the oracle,
  MIR, and Cranelift no-opt/opt; the fixture is auto-scanned by the AOT ELF corpus
  (`gate_aot_full_corpus`) and the LLVM `clang -O2` corpus
  (`gate_llvm_full_corpus_fifth_engine`), so all FIVE engines gate it. `read_lines`
  runs on the tree-walker AND MIR through the real shims
  (`read_lines_splits_file_into_line_vec_tree_and_mir`, tests/std_io.rs) and as a
  linked Cranelift / clang -O2 binary calling real libc
  (`gate_aot_native_read_lines`, `gate_llvm_native_read_lines`) over the
  self-contained `tests/fixtures/std_io_readpath/read_lines_native.cnr`: read a real
  4-line multibyte >64-byte file, split, write each line back newline-terminated ==
  the file body byte-for-byte, ret == 4.
- Full `cargo nextest` green (683 tests) incl. the std_io/text/adapters/native gates;
  clippy clean.

NEXT: the deferred `str_from`/`substr` native lowering; a streaming `Lines: Iter`
adapter (blocked on 0009 RefIndexed borrowed-yield — a Vec-backed iterator must yield
each line by borrow); a buffered reader/writer.

## COLLECTION-OP-ERGONOMICS — builtin-call return typing + re-borrow dispatch (2026-07-13)

Two general collection-op friction bugs that the file-I/O and collection slices
repeatedly worked around in-fixture, now fixed at the compiler level. Both are
DISPATCH/STATIC-TYPING fixes only — no collection-op MIR/semantics change, no
borrow-checking or soundness change; the checker's builtin result types are the
single source of truth and were NOT altered (the interp and MIR static-typers now
MIRROR them).

- (A) BUILTIN-CALL RETURN TYPING. A method/field/deref chain on a builtin
  collection-op result (`as_str(get(v,i).*)`, `as_bytes(as_str(read get(v,i).*))`)
  faulted at runtime ("unknown name `as_str`" on the tree-walker; "indirect/unknown
  aggregate call" / "unsupported place" on MIR/native) because neither static-typer
  could type a builtin `get`/`as_str`/... call's return, so the chain never resolved to
  a builtin. The checker ALREADY typed these (its `arg0_is_*` peels through
  `synth_arg_type`), but the tree-walker's `expr_static_ty` (`src/interp/eval.rs`) and
  the MIR builder's `static_ty` (`src/mir/build.rs`) returned `None` for a builtin
  call. FIX: each gained a `builtin_static_ret(name, args)` that mirrors the checker's
  `check_builtin` result types (`get(Vec[T],_) -> read T`, `get(Map[V],_) -> read V`,
  `as_str`/`str_from_unchecked`/`substr` -> `str`, `str_from` -> `Utf8Res`, `char_at` ->
  `CharStep`, `len`/`char_count` -> `usize`, `as_bytes` -> `[u8]`, `pop` -> `Opt`,
  `unbox`/ptr ops, ...), gated by the same arg0 collection guards so a same-named user
  fn still resolves. The MIR agg/value collection lowering also gained `collection_base`,
  which addresses a by-value collection place (`get(v,i).*`) by reference so the
  receiver is a single pointer to the collection.

- (B) RE-BORROW DISPATCH on an already-borrowed collection param. With `v: read
  Vec[T]`, `len(read v)` / `get(read v, i)` mis-dispatched vs the direct `len(v)` /
  `get(v,i)`: the checker's `arg0_is_*` recognition (via `synth_arg_type`, which peels
  ONE borrow) left the re-borrow as `&&C` and failed the receiver test (E0103 "unknown
  name"); the interp/MIR addressing then read one indirection short. FIX: the checker's
  `arg0_collection_ty` peels EVERY leading borrow (a collection is never itself a
  borrow); the interp's `deref_borrows` follows the whole borrow chain to the base
  address (`vec_base`/`map_base`/`string_base`/`bi_len`); and the MIR's
  `collection_base` collapses each extra borrow layer with a `Load`. Borrow-checking is
  unchanged — a re-borrow of a `read` borrow is still validated as before; only the
  DISPATCH recognition and base addressing were corrected.

- GATE. `tests/fixtures/run/split_lines.cnr` `sum_line_lengths` was flipped from the
  `fn(s: read String)` helper + direct-pass workaround to `as_bytes(as_str(read
  get(read v, i).*))` with `len(read v)` / `get(read v, i)` — exercising BOTH fixes in
  one chain, byte-identical across all five engines (`split_lines_all_cases_all_engines`
  drives tree, MIR, Cranelift no-opt/opt; the AOT ELF and LLVM `clang -O2` corpora
  auto-scan the fixture). Full `cargo nextest` green (683 tests); clippy clean.

STILL DEFERRED (unchanged by this fix): `str_from`/`substr` native lowering; a
streaming `Lines: Iter` adapter (0009 RefIndexed borrowed-yield); a buffered
reader/writer. Also orthogonal and still pre-existing: `len(<call returning a slice>)`
inline (e.g. `len(as_bytes(x))`) — the non-collection `len` path lowers its arg as a
place, so bind the slice to a local first (as the fixtures already do).

## STD-IO-BUFFERED — streaming buffered I/O: `BufReader.read_line` (owned per-line Strings, cross-refill assembly) + `BufWriter` (accumulate + threshold flush) (2026-07-13)

Adds the streaming buffered layer over the whole-file layer, fully native
(tree-walker · MIR · Cranelift · LLVM). Purely additive — no extern/trust-clause,
compiler, MIR, checker, backend, or `aot_runtime.c` change; built entirely on the
already-native `String`/`str_from_unchecked`/`subslice`/`slice_of[_mut]`/`read_into`/
`write_all` intrinsics.

- `BufReader { fd: i32, buf: [64]u8, pos: usize, filled: usize }` with
  `buf_reader(fd) -> BufReader`. `read_line(a: read Alloc, r: write BufReader) alloc
  -> LineIoResult` scans `buf[pos..filled]` for '\n' (10); on a hit at k the owned
  line is `buf[pos..k]` (a `subslice` cut only at the ASCII newline, valid UTF-8) and
  `pos` advances past the newline. On a miss it appends `buf[pos..filled]` to the
  owned accumulator `String` and REFILLS (`read_into` -> `pos=0, filled=n`): `Ok(0)`
  is EOF (return the accumulator if non-empty, else `None`), `Ok(n)` continues the
  scan — so a line SPANNING multiple refills (a line longer than 64, and lines
  straddling a refill boundary) is assembled correctly. A final line without a
  trailing '\n' is still returned; a trailing '\n' yields no extra empty line;
  interior empty lines are kept — MATCHING `split_lines`' convention. Owned-String
  yield (not a borrowed str-view) sidesteps the 0009 RefIndexed borrowed-yield gap; a
  streaming `Lines: Iter` stays blocked on it. Result: `LineIoResult { ok Ok(Opt),
  Err(IoError) }` over `enum Opt { Some(String), None }` (a monomorphic Opt in the
  vec_native idiom; the audited module stays NON-GENERIC), `?` propagating `IoError`.
- `BufWriter { fd: i32, al: Alloc, buf: String }` with `buf_writer(a: Alloc, fd) alloc
  -> BufWriter`. `bw_write_str(w: write BufWriter, s: str) alloc -> IoResult` appends
  into the owned `buf`, auto-flushing once it reaches 4096 bytes; `bw_flush(w: write
  BufWriter) alloc -> IoResult` does `write_all(fd, as_bytes(as_str(read w.buf)))` and
  RESETS the accumulator. String has NO in-place clear/reset primitive, so the reset
  is a fresh `w.*.buf = string_new(w.al)` (the old String's heap freed by the field
  reassignment's drop) — the `Alloc` is stored in the struct for exactly this. If a
  `string_clear`/`string_truncate` primitive is later added it would avoid the
  per-flush realloc; not needed for correctness.
- MUTABLE `write BufReader`/`write BufWriter` STATE ACROSS CALLS worked CLEANLY — no
  borrow/checker workaround. Field reads (`r.fd`, `r.buf[k]`, `r.pos`, `r.filled`),
  field-array slicing through the borrow (`slice_of(r.buf)`, `slice_of_mut(r.buf)`,
  `subslice(slice_of(r.buf), ..)`), and field writes (`r.*.pos = ..`, `w.*.buf =
  string_new(..)`) all check and lower on every engine, matching the arena's
  `write Arena` field-mutation pattern (11_5_arena).
- ONE PRE-EXISTING NATIVE LIMITATION hit (not new, already deferred under
  COLLECTION-OP-ERGONOMICS): the non-collection `len` lowers its argument as a PLACE,
  so `len(as_bytes(as_str(read w.buf)))` (a `len` of a temporary slice) is "unsupported
  place" on the native backends. FIX in-fixture (idiomatic, as the read path already
  does): bind the byte view to a local first — `let view: [u8] = as_bytes(as_str(read
  w.buf)); if len(view) >= 4096usize { .. }`.
- GATES. `tests/std_io.rs`: `buf_reader_read_line_streams_large_file_tree_and_mir`
  (>4096-byte multibyte file with a 200-byte line spanning many 64-byte refills, an
  interior empty line, trailing newline — reconstruction byte-exact + line count on
  tree AND MIR), `buf_reader_read_line_edge_cases_tree_and_mir` (empty file -> None
  immediately / no trailing newline / interior empty / single line no newline), and
  `buf_writer_round_trip_crosses_flush_threshold_both_engines` (300*20+4 = 6004 buffered
  bytes crossing the auto-flush threshold, flushed in order, byte-exact on tree AND
  MIR). Linked Cranelift and clang -O2 binaries calling real libc over the
  self-contained `tests/fixtures/std_io_readpath/buf_io_native.cnr`
  (`gate_aot_native_buf_io`, `gate_llvm_native_buf_io`): read a real >4096-byte file
  line by line through `read_line`, write each line back newline-terminated through the
  buffered `BufWriter` (auto-flushing) == the file body byte-for-byte, ret == line
  count. Full `cargo nextest` green (688 tests, +5); clippy clean.

NEXT: a streaming `Lines: Iter` adapter (blocked on 0009 RefIndexed borrowed-yield —
must yield each line by borrow); the `String` realloc ABI / an in-place
`string_clear` (would drop BufWriter's per-flush realloc); `str[i]` byte indexing and
the deferred `str_from`/`substr` native lowering.

## STR-INDEX-NATIVE + NON-PLACE-LEN — `str[i]` byte-indexing lowers, and non-collection `len` accepts a non-place (temporary) fat-pointer argument (2026-07-13)

Two native-lowering completeness gaps closed; MIR-lowering only — no checker,
collection-op, or fault-identity change. Byte-exact with the tree-walker across
tree · MIR · Cranelift (no-opt + opt) · LLVM -O2.

- (A) NON-PLACE `len`. The non-collection `len` (slice/array/str fat-pointer
  length, NOT the Vec/Map/String CollectionOp header read) lowered its argument
  AS A PLACE — reading the length word at offset 8 of the header in memory — so a
  temporary (`len(as_bytes(x))`, `len(<inline call returning a slice/str>)`) had
  no address and hit "unsupported place". FIX (`src/mir/build.rs`, the `"len"`
  value builtin): when the arg is NOT a place (new `is_place_expr` helper,
  mirroring `materialize_place`'s place arms), take its static type and
  `materialize_place` it into a fresh temp local, then read the length word from
  THAT temp — exactly what the tree-walker's `bi_len` does (`eval_value` first,
  then read offset 8). A place arg keeps the unchanged `lower_place` path. The
  collection `len` (offset-8 header word) is untouched. Flipped the in-fixture
  workarounds proving the direct form lowers + runs byte-exact: `tests/std_io.rs`
  `bw_write_str` now `if len(as_bytes(as_str(read w.buf))) >= 4096usize` (no
  `view` binding — proven on tree + MIR via
  `buf_writer_round_trip_crosses_flush_threshold_both_engines`), and `tests/text.rs`
  `as_bytes_is_free_retype` is now inline `len(as_bytes("hello"))` on ALL engines
  (`run_ret_all`), plus a focused `len_of_inline_slice_call`
  (`len(as_bytes(substr("hello",1,4)))` == 3) on all engines.

- (B) `str[i]` BYTE-INDEXING. The `Index` place lowering handled
  `Type::Array`/`Type::Slice`/`SliceMut` but not `Type::Str`, so `s[i]` was
  interp-only. A `str` is a `{ptr@0, len@8}` fat pointer with stride-1 `u8`
  elements, so the new `Type::Str` arm reuses the slice-index path verbatim
  (`Proj::Index { stride: 1, slice: true, span }`) — the SAME `Bounds` fault
  (kind + span) the slice-index path emits — yielding a `u8`. The three MIR
  engines' post-index element-type match (`src/mir/interp.rs`,
  `src/backend/lower.rs`, `src/backend/llvm.rs`) gained a `Type::Str =>
  Scalar(U8)` arm; the tree-walker (`src/interp/eval.rs`) already handled
  `str[i]` and is the oracle. GATES (`tests/text.rs`): `str_index_reads_bytes`
  (in-range `s[0]`,`s[2]` -> 97+99 == 196, all engines via `run_ret_all`),
  `str_index_out_of_bounds_faults` (`s[3]` -> `Bounds`, kind + span, all engines
  via `run_fault_all`); `str_from_then_use` now byte-indexes the recovered `str`
  directly (`s[2]`) instead of `as_bytes(s)[2]`. Corpus fixture
  `tests/fixtures/run/str_index.cnr` (loop-indexed `"candor"` bytes, folded ==
  631) is auto-scanned by the Cranelift + LLVM full-corpus gates
  (`gate_aot_full_corpus`, `gate_llvm_full_corpus_fifth_engine`,
  `gate_full_corpus_four_engine`).

- Full `cargo nextest` green (691 tests, +3); clippy clean; all native gates
  (aot / llvm / stage_a/b/d corpus, text, std_io, vec/map/string_native) pass.

NEXT (unchanged remaining debt): a streaming `Lines: Iter` adapter (blocked on
0009 RefIndexed borrowed-yield — must yield each line by borrow); the `String`
realloc ABI / an in-place `string_clear`/`string_truncate` (would drop
BufWriter's per-flush realloc); the deferred `str_from`/`substr` are already
native, `str[i]` and non-place `len` are now landed.

## LOAN-COPY-UAF — a borrow copied into a binding shed its loan: a safe-code use-after-free (memory-safety fix, 2026-07-13)

The 0015 borrowed-iteration review's finding F1 (SINK-grade, pre-existing) verified a
safe-code use-after-free the Stage-3 borrow checker ACCEPTED: a borrow copied into a new
binding shed its loan, so a later move/free/write of the borrowed place went unflagged.
Repro (`docs/reviews/0015-loan-copy-uaf-repro.cn`): `let b = read (deref bx); let c = b;
let owned = unbox(bx); use_it(read (deref c));` — `candor check` exited 0, `candor run`
faulted reading a freed slot.

ROOT CAUSE. Loans are created at each borrow (`record_borrow`, `src/check/mod.rs`) as
transient loans marked *carried*, then anchored to the landing binding's live range at the
`let`/assign site (`stmt.rs`). The anchor decision consulted `carries_borrow`, a syntactic
whitelist that returned `false` for a bare identifier — so `let c = b` anchored NOTHING, and
`b`'s own loan on `bx` died at `b`'s last use (the copy). `c` then aliased into `bx` with no
loan restricting it, and `unbox(bx)` (a move of the box) conflicted with nothing. NOTE the
deref-root anchoring was NOT part of the hole here: `read (deref bx)` canonicalizes to root
`bx` (deref collapses to the box binding, `dataflow.rs` `canonical`), which is exactly the
place that must stay frozen — `b`'s loan was correctly on `bx`; it was just shed on copy.

FIX (a real loan-propagation correction, not a repro special-case). A borrow value read out
of a place as a plain value ALIASES the source's borrow, so the source loan(s) must extend to
wherever the value lands. New `propagate_place_loans` (`src/check/mod.rs`), called from the
place-read arm of `check_expr` for `Use::Value` (`src/check/expr.rs`): when the value is a
`read`/`write` borrow, it records fresh transient copies (same place, kind, span) of every
loan anchored to the source binding and marks them carried. `carries_borrow` (`stmt.rs`) is
extended to recognize a bare place already holding a `read`/`write` borrow, so the existing
anchor path fastens the propagated loan onto the new binding's live range. Propagation is
transitive (`b -> c -> d`) and composes with call return-extension (a copy of a returned
borrow keeps the underlying argument loan). Deliberately kept to `Type::Borrow`/`BorrowMut`:
the anchor stays gated by `carries_borrow` (a syntactic/type test), so a `deref`-through to a
pointee copy or a non-borrow-returning builtin argument (e.g. `len(read a.dcode)`) that leaves
a stale carried loan is NOT anchored — this is what avoids false positives across the corpus.

GATES. `docs/reviews/0015-loan-copy-uaf-repro.cn` now fails `candor check` with E0802 ("cannot
move out of `bx` while it is borrowed"). Eight regression tests in `tests/loans.rs`
(`loan_copy_*`): copy-then-move-out (box + local), chained-copy-then-move (transitive),
copy-then-write (E0803), exclusive-copy-then-read-source (E0804), return-a-copied-borrow-of-a-
local (E0806), copy-of-a-return-extended-borrow-then-write (E0803), plus an NLL positive
(copy dies before the source is rewritten → clean). Full `cargo nextest` green (691 + 8 =
699), including the loans / selfhost_loans oracle, selfhost checker/analyses, generics, and the
native collections/allocator corpus; `--profile fast` green; clippy clean. No existing test
changed verdict — no valid borrow-copy/reborrow pattern was newly rejected.

RESIDUAL — RESOLVED (2026-07-13, verified repro + scoped fix). Slice values (`Type::Slice`/
`SliceMut`) are `copy` and alias their backing the same way, and a `let s2 = s;` copy shed the
loan identically. A VERIFIED safe-code repro (`docs/reviews/0015-slice-copy-uaf-repro.cnr`):
`box` a `[4]i64`, `let s = slice_of(bx.*); let s2 = s; let owned = unbox(bx); return s2[0];`
— pre-fix `candor check` exited 0 and the tree oracle faulted reading the freed box slot; the
same program without the `let s2 = s;` copy was correctly rejected E0802, so the copy was the
shed. FIX: `propagate_place_loans` (`src/check/mod.rs`) and the `carries_borrow` bare-place gate
(`src/check/stmt.rs`) are extended from `Type::Borrow`/`BorrowMut` to also cover `Type::Slice`/
`SliceMut` — a copied slice now records transient copies of its source's loans and anchors them
to the new binding's live range, exactly as the borrow copy does. The repro now fails check
E0802; four `slice_copy_*` regressions in `tests/loans.rs` (exclusive-copy-then-read-source
E0804, shared-copy-then-write-source E0803, chained/transitive copy E0804, plus an NLL positive
that stays clean) each checked CLEAN pre-fix. Full `cargo nextest` green (699 + 4 = 703), no
existing test changed verdict — the heavy slice corpus (vec / vec_native / std_io / selfhost
checker/analyses/interp/lower / stage_a-d native gates) stays green, so no valid slice code was
newly rejected. Scoped deliberately to `Slice`/`SliceMut`; a separate observation (NOT this
sub-class): `as_str`/`as_bytes`/`substr` views of a native `String` record no loan at
all (they retype through `arg0`/`Use::Value`), so a String realloc while a view is live is
accepted even without any copy. That path is NOW VERIFIED AND CLOSED — see STR-VIEW-UAF below.

## STR-VIEW-UAF — a `str`/`[u8]` view of a native String recorded no loan: a safe-code use-after-free (memory-safety fix, 2026-07-13)

The retype builtins `as_str`/`as_bytes`/`substr`/`str_from_unchecked` produce a `str`/`[u8]`
VIEW aliasing a native `String`'s heap buffer, but recorded NO loan tying the view to its
source: they retype through `arg0`/`Use::Value`, and neither `carries_borrow` nor
`propagate_place_loans` recognized the view types, so the transient `read`-loan on the source
died at the end of the retype statement. A `push`/`append` that forces a `string_reserve`
GROWTH (alloc-new + copy + FREE-old), or a move/drop of the source, while the view was live
was ACCEPTED — the view then dangled into the freed old buffer.

VERIFIED REPRO (check-clean + stale-read on EVERY engine, not merely a latent hole). Over the
reclaiming free-list allocator: `append(write s, "abcde")` (buf B0 = "abcde"), `let v = as_str(read s);
let vb = as_bytes(v);` (vb -> B0), read `vb[0]` = 97 (`'a'`), then `append(write s, "fghij")`
crosses cap 8 -> grow: alloc B1, copy, FREE B0. Reading `vb[0]` again returns 0 — the free-list
`free` wrote its `FreeBlock{next,size}` header into the freed B0, overwriting the string bytes the
view still points at. `candor check` exited 0; the return value `before*1000 + after` was 97000
(a sound program yields 97097) identically on the tree-walker, the MIR interpreter, and the
Cranelift native engine. This is a stale-read UAF (untracked view provenance), independent of any
copy — the view carried no loan in the first place.

FIX (record the source loan on the view, mirroring the LOAN-COPY / SLICE-COPY mechanism).
`carries_borrow` (`src/check/stmt.rs`) now recognizes the four retype calls
(`as_str`/`as_bytes`/`substr`/`str_from_unchecked`) and a bare place of type `str`, and
`propagate_place_loans` (`src/check/mod.rs`) is extended to `Type::Str`. For `as_str(read s)`
the `read s` already records a shared loan on `s` and marks it carried; recognizing `as_str`
anchors that loan to the view binding's live range. For the chaining cases
(`as_bytes(v)`/`substr(v, ..)`/`str_from_unchecked(b)`) the argument view carries the source
loan, which propagates through the retype to the result. NLL (backward liveness) releases the
view's loan at the binding's LAST use, so the pervasive make-view-use-done pattern is unaffected;
only a mutate/realloc/move/drop of the source WHILE a view is still live is rejected.

GATES. The repro now fails `candor check` E0801 (the grow's `write s` conflicts with the view's
live shared borrow of `s`). Four regressions in `tests/loans.rs` (real-syntax, `str_view_*` /
`substr_view_*`): as_bytes-of-as_str view then realloc (E0801), move-source-while-view-live
(E0802), substr-chained view then realloc (E0801, proving transitive propagation), plus an NLL
positive (view dies before the grow -> clean). Full `cargo nextest` green (703 + 4 = 707),
including the heavy `as_bytes`/`as_str` corpus (text, string_native, std_io, lines, buf_io, fmt,
print) and the selfhost + native (stage_a-d, *_native) gates; `--profile fast` green; clippy
clean. No existing test changed verdict — no valid make-view-use-done pattern was newly rejected.
This closes the last known loan-tracking gap before 0015 resumes.

## OBL-ITER-BORROW (shared branch) — `for read x in read coll` implemented end-to-end + the method-return loan-provenance gap it exposed (2026-07-14)

Discharges OBL-ITER-BORROW's SHARED, region-free branch as an implementation, not just a design:
`for read x in read coll` — the `RefIndexed` protocol (`count(read self) -> usize` +
`get_ref(read self, i) -> read Item`, `Item` unconstrained) behind design 0015 §4 — now runs
END-TO-END over a USER `impl RefIndexed`, byte-exact on all five engines (tree-walker, MIR,
Cranelift no-opt/opt, `clang -O2` LLVM, AOT). The desugar (already present, `src/real/parser.rs`
`desugar_ref_indexed`) is one arm on the existing `for`, mode-directed by the PATTERN's `read`
(NN#13-clean), lowering to the §4.2 loop; protocol selection reuses the general interface-method
resolution, so a user `impl RefIndexed for Bag` resolves `count`/`get_ref` to free fns that lower
on every backend with NO new runtime value kind (a borrowed yield is an address — §4.4). The
demonstration walks a Vec-of-`Box`-free, DROP-hooked (hence non-`copy`) element and reads a field
of each through the `read Item` reborrow — the capability `Indexed` (copy-out `at`, needs
`Item: copy`) and `Iter` (consuming) cannot express — each element dropped exactly once
(`tests/fixtures/run/refindexed_native.cnr`, driven by `tests/refindexed.rs`).

FINDING + FIX (a real loan-provenance completion, not a repro special-case). Design 0015 §5's escape
argument rests on ONE load-bearing checker fact: `get_ref(__i)`'s return is tracked as a reborrow of
the receiver, so an escaped yield keeps the collection loan live (§5 case 2 / open-Q1, marked
RESOLVED by the four `soundness:` commits). That was TRUE for FREE-fn borrow returns
(`get0(read c)`) but NOT for METHOD-call returns (`c.get_ref(i)`) — and the desugar emits a method
call. `carries_borrow`/`check_user_call`'s return-borrow loan extension (the LOAN-COPY / STR-VIEW
fix) only handled `Ident`-callee (free-fn) calls; the `Field`-callee (interface-method) path in
`check_iface_method_call` cleared the receiver's loan. So `let esc = c.get_ref(0); write c` — the
exact escape §5 forbids — checked CLEAN (a stale-borrow hole the completeness sweep missed, because
`arena_get` is called as a free fn, never as a method). Fix: `check_iface_method_call`
(`src/check/generics.rs`) now carries the receiver's loan when a `read`/`write self` method returns
a borrow deriving (compact default, single borrow-in) from the receiver, and `carries_borrow`
(`src/check/stmt.rs`, now `&mut self` + `method_returns_borrow`) admits such a method call — the
SAME return-extension a free-fn borrow return already got, applied to the method-call shape (no new
aliasing rule; 0001 §2.3 step 3). With it, mutation-during-iteration is rejected (E0801, via the
loop's own `read` loan — unchanged) AND any escape of the yield is rejected (E0801/E0803): a
directly-escaped `read W` keeps its source-loan on the collection live; the loop-local-cursor
escape is caught conservatively (assigning the yield to an outer local is rejected — design 0015 §7
open-Q1's documented non-escape fallback, sound). The normal (non-escaping) `for read` body — which
only READS through the yield — is unaffected.

GATES. `cargo nextest` full green (712 -> 715: +3 `refindexed.rs` — the all-engine functional walk,
mutation-during-iteration rejected, escaped-yield rejected); `--profile fast` green; clippy clean.
The new fixture auto-joins the four/five-engine corpus (llvm/aot/stage_d) — all green. The selfhost
loan-diagnostics oracle stayed byte-exact: the method-return loan extension rejected NO valid
existing borrow-returning-method call across the whole corpus. `for write x` (mutating yield) and
borrowed streaming remain future work; the obligation narrows to the shared branch, discharged.

DOC-vs-IMPL NOTE. A `Vec[T]` user `get_ref` returning `get(read self.v, i)` (the collection-op
surface's borrowing accessor) does NOT check: `get`'s returned borrow is not recognized by
`borrow_provenance` (`src/check/expr.rs`) as deriving from its receiver (E0806), so the demonstration
uses the `arena_get`-shaped place reborrow `return read self.slot;` instead — the compact-default
accessor design 0015 §4.1 names. The Vec-wired `count`/`get_ref` from the original discharge
(commit 3176fb7) remains tree-walker-only (method-call collection ops don't lower to MIR); the user
impl is the cross-engine path.

## FLOATS-S1 — `f64` (IEEE-754 binary64) landed end-to-end across ALL FIVE engines, bit-identical (2026-07-14)

CONTEXT. Candor had NO floating point. This slice adds ONE scalar type, `f64`, per a new design
note (`docs/design/0016-floats.md`). Decisions pinned there: (1) **f64 first** (f32 deferred);
(2) **IEEE semantics, regime-EXEMPT** — `+ - * /` and unary `-` never fault (overflow → ±inf,
`x/0.0` → ±inf/NaN), and a `wrapping{}`/`saturating{}` block does NOT change float behaviour (0006
§2.4's fault machinery is integer-only — confirmed reading); (3) **literal grammar** — a numeric
literal is `f64` iff it has a `.` (digit on BOTH sides: `0.5` yes, `.5`/`5.` no) or an exponent
(`1e10`, `1.0e-5`); always the concrete type `f64` (no flexible float-lit, no suffix); over-range
magnitude → ±inf; (4) **NaN/inf** flow through; IEEE comparisons (NaN != NaN true, all ordered/`==`
with NaN false); (5) **conversions** — `conv f64 <int>` rounds (exact `|x|<2^53`), `conv i{N} <f64>`
truncates toward zero, **saturating** (NaN→0, out-of-range clamps; Rust `as` = Cranelift
`fcvt_to_*int_sat` = LLVM `llvm.fpto*i.sat`), never faults, regime-exempt.

IMPLEMENTATION. Every layer: real lexer (float tokens), `token::ScalarTy::F64` + `is_integer`
(now excludes f64) / `is_float`, `ast::ExprKind::FloatLit{bits}`, checker (`unify_arith` for
`+-*/`+ordered-cmp accepting f64, Neg on f64, numeric `conv`, `%`/mixed-int-float REJECTED), both
interpreters, MIR (`build`/`interp`/`serial`/`opt`; float `Bin`/`Un`/`Cmp` carry `ScalarTy::F64` and
NO fault edge — INV-CHECK relaxed to skip float ops / int→f64 conv), and BOTH native backends
(Cranelift `fadd/fsub/fmul/fdiv/fcmp/fneg` + `fcvt_from_{s,u}int`/`fcvt_to_*int_sat` via `bitcast`;
LLVM `fadd`/…/`fneg`/`fcmp`/`sitofp`/`uitofp`/`fptosi.sat`/`fptoui.sat` via `bitcast`).

VALUE REPR. An `f64` is carried as its `to_bits()` 64-bit pattern (zero-extended) in the shared
i128/i64 "register" model and 8 bytes in flat memory (byte-identical to the IEEE encoding). Each
float op bit-casts to a native `double`, computes with real IEEE arithmetic, and bit-casts back —
nothing in load/store/call-ABI changes. `ty_range(f64)` is unsigned-64 so no path sign-extends the
pattern. The Conv wire format is UNCHANGED (source type recovered from the operand, not a new field)
— self-host lowering parity preserved. `main` returning `f64` reports its 64-bit word (its bits);
`trace(f64)` observes the bit pattern (the one channel all five engines share).

TWO TRAPS. (1) The Conv wire needed NO new field: adding a `from: ScalarTy` broke the self-host
lowerer's byte-exact MIR (it emits the old `(conv <to> …)`); recover the source scalar from the
operand instead. A checked `f64`→int conv keeps an INERT ConvLoss edge (never taken — saturating)
so INV-CHECK stays uniform. (2) A computed **NaN's sign bit is IEEE-UNSPECIFIED**: LLVM `-O2`
constant-folds `0.0/0.0` to `+NaN` (0x7FF8…) while x86 runtime `divsd` yields `-NaN` (0xFFF8…). So
the gate asserts finite/inf results and conversions to EXACT bits across all five engines, but NaN
by BEHAVIOUR (comparison outcomes) only.

GATES. `cargo nextest` full green; `--profile fast` green; clippy clean. New `tests/floats.rs`
(16 tests: exact-bits arithmetic/precedence/rounding, ordered comparisons, ±inf exact bits, NaN
behaviour, int↔f64 conv incl. saturation + round-trip, a Newton's-sqrt + dot-product loop, and 4
negative checks) asserts BIT-IDENTICAL across tree-walker / MIR / Cranelift no-opt / Cranelift -O2 /
LLVM -O2 (`f64::to_bits`). A new `tests/fixtures/run/floats.cnr` auto-joins the aot/stage_b native
corpus. Existing integer arithmetic / regime / conv tests unperturbed (f64 excluded from
`is_integer`, so no int typing/regime path changed). NEXT: f32; `fmt_f64` float FORMATTING (a
separate slice); WASM float opcodes (now unblocked); math functions (`sqrt`/`sin`/…); the full
NaN-payload / signaling-NaN edge cases; a flexible float-literal type.

## FLOATS-S2 — `f32` (IEEE-754 binary32) landed end-to-end, bit-identical across ALL FIVE engines (2026-07-14)

CONTEXT. Completes the float family: adds `f32` mirroring the `f64` slice (FLOATS-S1) at single
precision. Design pinned in `docs/design/0016-floats.md` §9. Everything in §2 (regime-EXEMPT IEEE),
§4 (NaN/inf), §6 (value repr) carries over unchanged; only the two `f32`-specific decisions are new.

DECISIONS. (1) **Literal grammar — an `f32` *suffix* on a float form.** `f64` stays suffix-free (a
`.`/exponent literal is `f64`); `f32` is spelled by adding `f32` to a *float-form* literal:
`1.5f32`, `1.0e-5f32`. The suffix attaches ONLY to a float form — `10f32` (integer form) is a lex
error (`L0005`), keeping the integer-suffix space integer-only; write `10.0f32`. `f32` is the ONLY
float suffix; any other (incl. `f64`) on a float form is rejected. Over-range magnitude → `±inf`
(`str::parse::<f32>`). Still no flexible float-lit type. (2) **Conversions.** No implicit promotion
between `f32`/`f64`/int (`f32 + f64` rejected exactly like `f64 + i64`). `conv f32 <int>` rounds;
`conv f64 <f32>` widens EXACTLY (`fpromote`/`fpext`); `conv f32 <f64>` narrows with rounding, may →
`±inf` (`fdemote`/`fptrunc`); `conv i{N} <f32>` truncates toward zero, SATURATING (NaN→0, clamp;
Rust `as` = Cranelift `fcvt_to_*int_sat` = LLVM `llvm.fpto*i.sat.iN.f32`); `f32`↔`f32` identity.
All IEEE, regime-exempt, never fault.

VALUE REPR / HOW f32 IS DISTINGUISHED. An `f32` is carried as its `f32::to_bits()` 32-bit pattern
**zero-extended** into the shared i128/i64 register slot and **4 bytes** in flat memory;
`ty_range(f32)` is unsigned-32 so it is never sign-extended. **No new width tag was needed anywhere.**
The existing `ScalarTy` tag already distinguishes `F32` from `F64` and is carried at every op site
that needs it — `Operand::Const(_, ScalarTy)` and the `ty: ScalarTy` on MIR `Bin`/`Un`/`Conv` — while
`Cmp` recovers the width from its operands (both are the same float type, checker-guaranteed).
`ast::ExprKind::FloatLit` and the real `Float` token gained a `ty: ScalarTy` field so the literal
width survives to lowering; the Conv wire format is UNCHANGED (source/target from operand/`to`),
preserving self-host MIR parity.

IMPLEMENTATION. Same layers as FLOATS-S1: real lexer (`f32`-suffix on `finish_float`),
`token::ScalarTy::F32` + `is_float`, `FloatLit{bits,ty}`, checker (`unify_arith` now requires the
SAME float type — rejects `f32`+`f64`; Neg/conv/trace over any float; `%` still rejected), both
interpreters (parametric `float_arith`/`float_cmp`/`float_neg`/`float_conv` over the width), MIR
`build`/`interp`/`serial`, and both backends (Cranelift `as_float`/`float_bits` via
`ireduce`+`bitcast F32` / `uextend`, `fpromote`/`fdemote`, `fcvt_*` at F32; LLVM `trunc`+`bitcast float`
/ `bitcast`+`zext`, `fpext`/`fptrunc`, `sitofp`/`fptosi.sat.*.f32`). `opt` is DCE-only (type-agnostic),
untouched. f64 and integer paths byte-unchanged.

ONE TRAP. The tree-walker `Eq`/`Ne` branch computes an `equal` bool that the caller flips for `!=`
(`res = if Eq {equal} else {!equal}`). Routing `float_cmp(op, …)` (which itself returns the `!=`
result for `Ne`) into `equal` DOUBLE-NEGATES `!=`. Fix: pass `BinOp::Eq` to get equality and let the
outer flip handle `!=`. Caught because the f64 `!=` oracle regressed (`1.0 != 2.0` → false) — the MIR
Cmp (which takes `op` directly) was already correct, so the divergence pinpointed the oracle bug.

GATES. `cargo nextest` full: **808 passed / 0 failed** (incl. all f64, integer arithmetic/regime,
self-host lexer/parser, native aot/stage, and wasm gates); `--profile fast` 678 green; clippy clean.
New `tests/floats_f32.rs` (19): exact-bits arithmetic/precedence/rounding, ordered comparisons, ±inf
exact bits, NaN by BEHAVIOUR, int↔f32 conv + saturation + round-trip, `f32`↔`f64` widening (exact) +
narrowing (precision loss asserted), a Newton-sqrt + dot-product loop, and negative checks
(`f32`+`f64`, `f32`+int, `f32 %`, `10f32`, `1.5f16` all rejected) — BIT-IDENTICAL across tree-walker /
MIR / Cranelift no-opt / Cranelift -O2 / LLVM -O2 via `f32::to_bits`. New
`tests/fixtures/run/floats_f32.cnr` auto-joins the aot/stage native corpus. NEXT: **WASM float
opcodes** (`f32` + `f64` loads/stores/arith — now that both float types exist); math functions
(`sqrt`/…); the full NaN-payload / signaling-NaN edge cases; a flexible float-literal type.

## FLOATS-S3 — `bitcast` (same-width float<->int BIT reinterpretation) landed end-to-end, bit-identical across ALL FIVE engines (2026-07-14)

CONTEXT. `conv` (FLOATS-S1/S2) converts the numeric VALUE (`conv i64 (1.0)` == `1`).
This slice adds `bitcast`, which reinterprets the identical BITS as a same-width type
(`bitcast i64 (1.0)` == `0x3FF0000000000000`). It unblocks the WASM float opcodes
(their interp holds floats as `i64` stack bits and must reinterpret them to do
arithmetic) and is generally useful (float hashing/serialization, WASM `reinterpret`).
Design pinned in `docs/design/0016-floats.md` §10.

DECISIONS. (1) **Spelling — a `bitcast <ty>` keyword**, uniform with `conv`: prototype
grammar `bitcast T ( e )`, real/surface grammar the paren-free `bitcast ScalarKw
<operand>` (exactly as `conv`). Chosen over intrinsic fns — one parse slot / precedence
/ emit path, no per-type builtin family. (2) **Semantics — same-BYTE-WIDTH float<->int
only.** Exactly one side a float, the other an integer of the same byte width: f64<->
{i64,u64,isize,usize}, f32<->{i32,u32}. REJECTED at check time (`E0714`): different
width (f64<->i32), both-integer (i64<->u64 — that is `wrapping{conv}`), both-float
(f64<->f32), non-scalar/bool/unit. (3) **Total + regime-independent.** Bitcast NEVER
faults (no fault edge) and a bitcast inside `wrapping{}`/`saturating{}` is unchanged —
a pure reinterpret has no overflow notion. (4) A bare `{integer}` on the int side takes
the float's same-width UNSIGNED int so its full pattern survives (a high-bit pattern
like `0xFFF8…` needs an explicit `u64` suffix; a suffix-less literal is `i64`-range-
checked).

IMPLEMENTATION. Every layer as a sibling of `conv` but a distinct, simpler op:
`ast::ExprKind::Bitcast{ty,expr}`, real+prototype lexer keyword (`bitcast`), both
parsers, checker `check_bitcast` (§10.2 pair rule, new code E0714), both interpreters
(`eval_bitcast` / MIR arm), and a **NEW MIR rvalue `Bitcast{to,v}`** (build/interp/
serial/opt) — NOT a `conv` "reinterpret" mode: `conv` carries `regime`+a `fault` edge
(INV-CHECK) a total bitcast would only null out, whereas `Bitcast` carries neither and
falls through INV-CHECK like a float op. Wire form `(bitcast <to> <operand>)` is
ADDITIVE — existing `conv` MIR + self-host lowering parity untouched. Both backends:
Cranelift `canon`, LLVM `canon` (a NO-OP at 64 bits, `ireduce`/`trunc`+`sext`/`zext`
at 32) — the raw register-level reinterpret, pointedly NOT `fcvt`/`fpto*i`/`*tofp`.

VALUE REPR. Floats are already carried as their `to_bits()` pattern in the shared
i128/i64 register and byte-identically in flat memory (S1/S2 §6/§9.3), so a bitcast
changes NO bits — it only re-canonicalizes the operand's held pattern to the target
width/signedness (`fit_bits(x,to)` in the interps; `canon` in both backends). Nothing
in load/store/call-ABI changes.

NaN. Bitcast is the case a SPECIFIC NaN payload survives exactly: no arithmetic
canonicalizes it, so `bitcast i64 (bitcast f64 (0x7FF8000000000001))` reproduces
`0x7FF8000000000001` bit-for-bit on all five engines — asserted by EXACT bits (vs.
computed NaN, gated by behaviour in tests/floats.rs).

TWO TRAPS. (1) The tree-walker `trace(x)` reads a NON-float arg at I64 (8-byte) width,
so tracing a bare `i32` result faults `bad_pointer` (reads past its 4-byte slot) — a
PRE-EXISTING trace behaviour (plain/`conv`/`bitcast` i32 all fault identically). The
gate widens 32-bit results with `conv i64 (bitcast u32 (..))`; bitcasting to the
UNSIGNED u32 makes the widening zero-extend, matching host `f32::to_bits()` (a u32) —
`bitcast i32` would sign-extend and diverge. (2) The real array-type syntax is `[N]T`
(not `[T; N]`) and `wrapping{}` is a STATEMENT block (the reinterpret goes inside as
`bits = bitcast i64 (x);`, not `let b = wrapping { bitcast .. }`).

GATES. `cargo nextest` full green (incl. all f64/f32, integer arithmetic/regime, self-
host lexer/parser/lower/interp, native aot/stage/llvm, and wasm gates); `--profile
fast` green; clippy clean. New `tests/bitcast.rs` (9 tests): f64<->i64 + f32<->i32
round trips over a spread; known encodings (0x3FF0.../0x4000... for f64, 0x3F800000 for
f32); the EXACT-NaN-payload round trip; regime-independence (bitcast in `wrapping{}`);
and E0714 rejections (different-width, both-integer, both-float) — BIT-IDENTICAL across
tree-walker / MIR / Cranelift no-opt / Cranelift -O2 / LLVM -O2. New
`tests/fixtures/run/bitcast.cnr` auto-joins the aot/stage native corpus. Existing conv
value-conversion semantics UNCHANGED. NEXT: **WASM float opcodes** (now unblocked — the
interp can reinterpret its i64-held stack bits as f32/f64); math functions
(`sqrt`/…); the full NaN-payload / signaling-NaN edge cases; a flexible float-lit type.


## WASM-FLOAT — the WebAssembly float ISA landed in the Candor-written interpreter, gated == wasmi + bit-exact across the Candor engines (2026-07-14)

CONTEXT. The M0-M4 WASM interpreter (`compiler/tests/fixtures/wasm/interp.cnr`,
written IN Candor: LEB128 byte cursor, id-dispatched section walk, non-recursive
`exec` over a `[256]i64` operand stack + `[256]Act` activation stack, integer ISA /
structured control / linear memory / host imports) carried only integers. FLOATS-S1/
S2/S3 gave Candor `f64`/`f32` + `bitcast`; this slice implements the WASM MVP float
opcodes on top. Floats RIDE THE i64 OPERAND STACK as their IEEE bit pattern (f64 as
its 64-bit `to_bits`, f32 zero-extended into the low word, mirroring `f32.load`/
`f32.const`); each op reinterprets the slot with `bitcast` AT THE BOUNDARY, does real
IEEE math with Candor `f32`/`f64` ops, and `bitcast`s the result back. The Candor
COMPILER is untouched — this is pure interpreter (Candor) + harness (Rust) work.

SCOPE (implemented). const (f32/f64, raw LE bytes); load/store (f32.load 0x2a /
f64.load 0x2b / f32.store 0x38 / f64.store 0x39, little-endian bytes <-> the bit slot,
bounds-trapped via the M2 memory path); arithmetic add/sub/mul/div (both widths);
abs/neg (sign-bit ops on the slot — exact for -0/NaN); copysign (bit splice); min/max
(WASM rules: NaN in -> NaN out; min(-0,+0)=-0, max=+0 via OR/AND of the sign-carrying
patterns); ceil/floor/trunc/nearest (ALL FOUR bit-exact, no intrinsic: trunc via an
in-range int round-trip + sign restore; nearest via the add/sub-2^52 magic under the
IEEE round-nearest-even default; ceil/floor off trunc ± 1); comparisons eq/ne/lt/gt/le/
ge (push i32 0/1, IEEE); conversions — trapping trunc i32/i64.trunc_f32/f64_s/u,
convert i32/i64_s/u -> f32/f64, f32.demote_f64 / f64.promote_f32, and the four
reinterpret ops (a same-slot width re-canonicalization). Type/blocktype decode already
accepted f32(0x7d)/f64(0x7c) valtypes (they are counted, not validated; blocktype
`>= 0x7c` yields arity 1), and `skip_immediates` learned f32.const=4 / f64.const=8 raw
immediate bytes so the control-flow forward scans don't misread a float const.

KEY SEMANTICS. (1) **trunc TRAPS vs Candor's SATURATING conv.** WASM MVP
`i{N}.trunc_f*_s/u` TRAP on NaN or out-of-range; Candor's `conv i{N} <float>` SATURATES
(0016 §5). So the interp RANGE-CHECKS in the float domain against the exact
representable bounds (e.g. f64->i32_s traps iff `x >= 2^31 || x <= -(2^31+1)`; f32->i32_s
iff `x >= 2^31 || x < -2^31`, since f32 has no representable value between -2^31 and
-2^31-1) THEN uses `conv` (in range -> exact truncation), rather than trusting `conv`'s
saturation. (2) **NaN sign is unspecified** for arithmetic-generated NaNs (0016 §4/§9.3,
and WASM's canonical arithmetic NaN): the gate compares NaN results by IS-NAN, non-NaN
by EXACT bits.

DEFERRED (next chain links, reported precisely). **`f32.sqrt` (0x91) / `f64.sqrt`
(0x9f)** — correctly-rounded IEEE sqrt needs a Candor `sqrt` INTRINSIC; a Newton
iteration is not bit-identical to hardware sqrt, so the interp TRAPS on these rather
than shipping a wrong answer (do NOT ship a non-bit-exact float op). **trunc_sat**
(the saturating `0xFC`-prefixed truncations) — a later opcode set, not implemented. No
rounding op was deferred: all of ceil/floor/trunc/nearest are bit-exact here.

GATES. Extended `compiler/tests/wasm.rs` with a wasmi (1.1.0) differential over float
modules: each module runs through the Candor interp AND wasmi with results asserted
equal — non-NaN float results BIT-EXACT (`to_bits`), NaN results by IS-NAN, and the
trapping trunc conversions trap in BOTH. Coverage: f32/f64 arithmetic, min/max (incl.
NaN and -0/+0), abs/neg/copysign, all four rounding ops, comparisons, int<->float
conversions, promote/demote, reinterpret, f32/f64 load/store round-trip, the trunc
TRAP (out-of-range/NaN), and a mixed int+float module (compute f64 -> trunc to i32 ->
combine). The large matrices run on the tree-walker oracle (like the M3 corpus); a
`float_cross_engine_agreement` test asserts the interpreter is BYTE-IDENTICAL across
all four Candor engines (tree-walker / MIR / Cranelift no-opt / -O2) over a
representative float set — non-NaN bit-exact, NaN by is-nan. `tests/fixtures/run/
wasm_interp.cnr` stays byte-identical to the canonical `interp.cnr` (drift guard), and
`run_wasm_file.cnr` regenerated to embed the updated interp. `cargo nextest run` (full)
+ `--profile fast` green; clippy clean. NEXT: a Candor `sqrt` (and broader math)
INTRINSIC to unblock `f*.sqrt`; then trunc_sat; the full NaN-payload / signaling-NaN
edges.


## SQRT — a correctly-rounded `sqrt` intrinsic + `f32.sqrt`/`f64.sqrt` wired into the WASM interp (2026-07-14)

CONTEXT. Candor had f32/f64 arithmetic but NO square root; a Newton/Heron iteration
converges one ULP short of correctly-rounded IEEE sqrt, so the WASM float slice
DEFERRED `f32.sqrt` (0x91) / `f64.sqrt` (0x9f) — the interp TRAPPED rather than ship a
non-bit-exact op. This slice adds a `sqrt` intrinsic that lowers to each backend's
NATIVE square root (bit-identical to hardware/IEEE, deterministic — unlike an
arithmetic NaN's sign) and closes that deferral.

PART A — the intrinsic. SURFACE: a compiler-known builtin `sqrt(x)` (an ordinary
call, no new keyword — like `len`/`is_null`), OVERLOADED by the argument's float type
(`f32 -> f32`, `f64 -> f64`); a single name works because the checker resolves
builtins by argument type. It is a pure, TOTAL, non-faulting unary float->float op:
`sqrt` of a negative is NaN (IEEE, NOT a fault), `sqrt(-0.0) == -0.0`, `sqrt(0.0) ==
0.0`, `sqrt(+inf) == +inf`; being non-faulting it is regime-independent and carries no
fault edge (design 0016 §11). IMPLEMENTATION: a dedicated `Rvalue::Sqrt { ty, v }`
(parallel to `Bitcast`), threaded through checker (`check/expr.rs check_builtin`), both
interpreters (tree-walker `eval_builtin` + `mir/interp.rs`, via a `float_sqrt` helper
over Rust's correctly-rounded `f{32,64}::sqrt`), `mir/build.rs` (`is_builtin` +
`lower_builtin_value` + `builtin_static_ret`), `mir/serial.rs` (wire round-trip),
`mir/opt.rs` (pure & non-faulting -> DCE-eligible when dead). Both native backends emit
the NATIVE sqrt — Cranelift's `sqrt` instruction (`sqrtsd`/`sqrtss`) and LLVM's
`@llvm.sqrt.f64`/`@llvm.sqrt.f32` intrinsic (confirmed in the emitted `.ll`), NOT a
software approximation: reinterpret the register bit pattern as a float, take native
sqrt, reinterpret back. The front-end (lexer/parser/AST) is UNTOUCHED — `sqrt` is a
Call, not an ExprKind, so the AST-level files that `bitcast` needed are not involved.
GATE: `tests/sqrt.rs` + `tests/fixtures/run/{sqrt,sqrt_f32}.cnr` (auto-enlisted in the
aot/stage_d/llvm corpus gates) assert BIT-IDENTICAL results across all five engines
(tree-walker / MIR / Cranelift no-opt / -O2 / LLVM -O2) for f32 AND f64: known values
(`sqrt(4.0) == 2.0`, `sqrt(2.0)` to exact bits, `sqrt(0.0)`, `sqrt(-0.0) == -0.0`,
`sqrt(1e100)`), a round-trip `sqrt(x)*sqrt(x)`, and a negative -> NaN gated by IS-NAN
behaviour (NaN sign IEEE-unspecified across a folding compiler vs runtime) proving it
is a VALUE not a fault. (A corpus fixture must not RETURN 2 — exit code 2 is the
real-ELF fault signal — so the fixtures return `21 * sqrt(4.0) == 42`.)

PART B — WASM `f32.sqrt`/`f64.sqrt` wired in. In `tests/fixtures/wasm/interp.cnr`
`eval_funop`, the deferred-trap for 0x91/0x9f is replaced with: bitcast the operand
slot to the float, call the Candor `sqrt` intrinsic, bitcast back (mirroring the
existing ceil/floor rounding ops). The drift-guard copies stay in lock-step — the run/
corpus copy (`tests/fixtures/run/wasm_interp.cnr`) is byte-IDENTICAL, and the file-run
fixture (`tests/fixtures/wasm/run_wasm_file.cnr`) embeds the same reusable interp
verbatim. GATE: `tests/wasm.rs` extends the wasmi (1.1.0) differential to sqrt modules
— `diff_float_sqrt` runs f32.sqrt/f64.sqrt over perfect squares, irrational roots
(the case Newton gets wrong), zeros, inf, and NaN-producing negatives, asserting
Candor-interp == wasmi BIT-EXACT for non-NaN and NaN by IS-NAN; `diff_vector_length`
gates a `sqrt(x*x + y*y)` length (hypot 3,4 -> 5; 5,12 -> 13) vs wasmi; and
`float_cross_engine_agreement` adds sqrt to the small all-four-engine set.

ALL existing tests green: `cargo nextest run` (full, incl. self-host corpus, the
aot/llvm/stage_d native gates, freestanding, concurrency, the WASM M0-M4 suite + the
float differential) + `--profile fast` (704 tests) + clippy clean. The WASM
interpreter's remaining post-MVP gaps (for the next chain links): globals (import +
defined `global.get`/`global.set`), tables + `call_indirect`, the `trunc_sat`
(0xFC-prefixed saturating) conversions, and the full NaN-payload / signaling-NaN edges.

## WASM-GTC — globals + tables/`call_indirect` land in the Candor-written WASM interpreter, closing the core MVP; gated == wasmi incl. trap-equivalence + byte-exact across the Candor engines (2026-07-14)

CONTEXT. The WASM interpreter (`compiler/tests/fixtures/wasm/interp.cnr`, written IN
Candor) ran M0-M5 (decode+eval, control flow, linear memory, host imports, the full
float ISA). The two remaining CORE MVP features were GLOBALS and TABLES +
`call_indirect` — the machinery a compiled language needs for function pointers /
vtables / virtual dispatch. This slice adds both, keeping the non-recursive `exec`
architecture, the `wrapping{}` iN semantics, the function INDEX SPACE (imports low),
and the drift guards.

PART A — GLOBALS. `decode_module` gains the **Global section (id 6)**: each global is
a valtype + a mutability byte + a constant init expr (`i32/i64/f32/f64.const` or
`global.get` of an earlier/imported global; `; end`). **Imported globals** (import
kind 0x03) are decoded and occupy the LOW global indices — same index-space pattern as
functions — so a defined global's index is shifted past them (MVP has no host global,
so an imported global's runtime value defaults to 0 and its mutability byte is
recorded). Globals hold their i32/i64/f32/f64 value as a bit pattern in an i64 slot
(like the operand stack). `exec` carries a single `gvals: [64]i64` initialized from the
module's decoded globals and SHARED across every activation (a callee's `global.set` is
visible on return — globals are instance state, not per-frame). New opcodes:
**`global.get` (0x23)** pushes `gvals[x]`; **`global.set` (0x24)** pops into `gvals[x]`
(the module is assumed valid — wasmi rejects a set to an immutable global at
validation — so no runtime mutability check is needed).

PART B — TABLES + `call_indirect`. `decode_module` gains the **Table section (id 4)**
(funcref elem type 0x70 + limits; one table in MVP) stored as a fixed `tbl: [256]i64`
of function indices with -1 = null/uninitialized, plus `tbl_size` (the declared min),
and the **Element section (id 9)** (active segments: table 0, a const offset, a vector
of function indices) filling the table at instantiation. **`call_indirect` (0x11)**
reads a TYPE index + a table index byte (0x00 in MVP), pops the i32 table slot, then
applies the three spec TRAP conditions EXACTLY per spec BEFORE invoking: (1) slot
`>= tbl_size` -> trap (OUT OF BOUNDS), (2) slot entry `< 0` -> trap (NULL /
uninitialized), (3) the stored function's actual signature must MATCH the call site's
declared type -> trap on mismatch (the indirect-call SAFETY property a compiled vtable
relies on). The signature check is STRUCTURAL: each functype is folded at decode into a
collision-free `type_key` (valtypes 0x7c..0x7f -> 1..4, a separator between params and
results), stored per type and referenced per function (`ftype`); two types match iff
their (params, results) valtype sequences match — exactly wasmi's dedup'd equality, no
arity-only shortcut. On no trap the callee is invoked identically to `call` (0x10) —
the two opcodes now share one `exec` branch that resolves `callee` (direct immediate vs
table lookup) then runs the common host/defined dispatch, respecting the function index
space.

GATES. `compiler/tests/wasm.rs` extends the wasmi (1.1.0) differential (M6): globals —
a mutable global a function increments and returns, an immutable read, an i64
round-trip, two-globals set/get, all `== wasmi`, and an imported-global index-space test
(a defined global at index 1 after an imported global at index 0) byte-exact across the
four Candor engines; `call_indirect` — a mini vtable (index selects x+100 / x*2 / x-1)
`== wasmi` and byte-exact across engines, plus the TRAP cases as trap-equivalence (OOB
table index, NULL slot, and a SIGNATURE MISMATCH where the call site declares a
`(i32,i32)->i32` against a stored `(i32)->i32` — a valid module that traps at RUNTIME in
BOTH Candor and wasmi). Value modules run on all four Candor engines via `run_ret_all`
(tree-walker / MIR / Cranelift no-opt / -O2). The drift guards stay in lock-step: the
run/ corpus copy (`tests/fixtures/run/wasm_interp.cnr`) is byte-IDENTICAL to the
canonical `interp.cnr`, and the file-run fixture (`tests/fixtures/wasm/run_wasm_file.cnr`)
was regenerated to embed the updated interp verbatim.

ALL tests green: `cargo nextest run` (full, incl. self-host corpus, the aot/llvm/stage_d
native gates that compile the run/ corpus copy, freestanding, concurrency, and the WASM
M0-M6 suite = 57 wasm tests) 840 passed; `--profile fast` 710 passed; clippy clean. The
Candor compiler was NOT changed. The WASM interpreter now covers the CORE MVP — it can
run genuinely compiled modules that dispatch through function pointers / vtables.
DEFERRED (next chain links, reported precisely): `trunc_sat` (0xFC-prefixed saturating
float->int) conversions; the full NaN-payload / signaling-NaN edges; reference-type / GC
proposals, multi-table, and bulk-memory (`memory.copy`/`fill`, passive segments) — all
non-MVP. Running real clang/rustc `.wasm` output additionally needs the WASI import
surface those toolchains target (only a WASI-lite `print_i32`/`print_str` is wired), so
the interpreter is MVP-complete but not yet a drop-in host for arbitrary compiled wasm.

## Design 0017 (packages) ACCEPTED — `candor audit` extension queued (2026-07-15)

Design 0017 (packages, manifests, dependency resolution) ACCEPTED with review
repairs applied (`docs/reviews/0017-packages-review.md`; also issued a dated erratum
to design 0008 §2.4 for the `src/` root move, review F6a). New obligation, queued as
the **first packaging implementation slice**: extend `candor audit` to enumerate
`unsafe` regions + all `assumed-proven` contracts **graph-wide** (not only the
`boundary` foreign surface it walks today, `compiler/src/audit.rs`) — this both
meets philosophy §7's success criterion, which the shipping tool does not yet meet
(a pre-existing local-audit gap, review F1b), and is the prerequisite for 0017 §8's
whole-graph trust aggregation. Relates to OBL-FFI and OBL-CONTRACT (the
`assumed-proven` enumeration surface). Trust-delta *gating* on lock updates stays a
first-1.0 need (0017 Open-Q1), not 0.x.

## OBL-0017-MANIFEST — packaging slice 1: the `candor.toml` manifest parser + closed schema (2026-07-15)

First code slice of design 0017 (packages). Lands the manifest **data model +
parse + validate** only — no resolver, no lockfile, no multi-package build driver
(later slices). `compiler/src/manifest.rs`: `Manifest`/`Package`/`Version`/`Lib`/
`Bin`/`Dependency`/`Source` model 0017 §2's schema; `parse_manifest(text)` and
`load_manifest(dir)` (the latter returns `Ok(None)` for a manifest-less directory —
today's degenerate bare package, 0017 §1/§6, left unchanged). The `toml` crate
(v1, serde-integrated) is a normal prototype dependency.

- **Closed schema (0017 §2, load-bearing):** an unknown key is an *error*. The
  top-level and `[package]`/`[[bin]]` stanzas deserialize with serde
  `deny_unknown_fields`; the toml error names the offending/missing key precisely
  (M0002).
- **Validation:** name/alias identity charset (M0100/M0110/M0203 — lowercase-alnum
  + `_`/`-`, leading letter), semver triple `major.minor.patch` (M0101), the one
  legal `edition = "2026"` else "unknown edition" (M0102, 0017 §3), and dependency
  sources (path non-empty; git requires a non-empty `rev`; M0200/M0202).
- **Forward-compatible source (0017 §4):** a `[dependencies]` value is read as a
  source-selecting inline table; the *kind* is chosen by which recognized key is
  present, so a future `{ registry = … }` source slots in as a new branch without
  disturbing existing manifests — and an unknown kind today is a precise
  "unknown source kind" error (M0201), not silent acceptance.
- **Wiring:** a `candor manifest <dir_or_file>` debug command parses + prints the
  manifest as JSON; the parse API is exposed for tests.
- **Gate:** `compiler/tests/manifest.rs` (18 tests) — a full valid manifest
  (name/version/edition/freestanding + `[lib]` + two `[[bin]]` + path/git/aliased
  deps) plus each rejection (unknown package key, unknown top-level table, bad
  version, unknown edition, missing required field, git-without-rev, empty path,
  unknown/registry source kind, ill-formed alias). Full `cargo nextest run` green
  (892 tests), clippy clean.

Next packaging slices: the `src/` module-root support (0008 §2.4 erratum) +
cross-package resolver + `candor.lock` lockfile, then the multi-package build
driver (0017 §6/§7).

## OBL-0017-RESOLVE — packaging slice: cross-package resolver + `candor.lock` + package-qualified `use` — "A depends on B, build them together" (2026-07-15)

The milestone slice of design 0017: a manifested package's `[dependencies]` are
resolved, linked, and built with it into one program, so **package A can depend on
package B (path source) and build+run together**. Implements 0017 §5/§6/§7 and the
F2 injective-pkgid repair; reuses 0008's visibility + acyclicity + merged-`Program`
model unchanged (§8, no new linking/visibility mechanism).

- **Resolver (`compiler/src/resolve_pkg.rs`).** From the root package it
  transitively loads each dependency's `candor.toml` (`load_manifest`) and builds
  the pinned set. **Path deps** resolve relative to the depending manifest's dir
  and canonicalize; **single-version unification** dedups a package reached via the
  **same** canonical source to one node and rejects the **same name via different
  sources** with a hard conflict naming both request paths (E0923, §6). The
  **package graph must be acyclic** — a transitive self-dependency is E0927 (§8).
  **Git deps are a clean deferral:** they error (E0924, "not yet fetched") rather
  than fetch; the resolver is shaped so a git source slots in as a resolved
  directory without disturbing the path-dep milestone.
- **`candor.lock` (§6).** TOML at the root package: per resolved package `name`,
  `version`, `edition`, exact source (canonicalized path; git url+rev when git
  lands), and a **content hash** over its `src/` sources. Written on resolution; a
  present lock consistent with the manifest is **reused verbatim** (not rewritten,
  so its bytes are stable). Written only when dependencies exist, so a manifest-less
  or dependency-free package's directory is untouched.
- **Cross-package `use` (§5).** `use b::{item}` — the first segment is a
  `[dependencies]` **local** name; the remainder resolves as filesystem position in
  that dependency's `src/` root. **What crosses is exactly 0008 §3
  external-reachability:** only the dependency's public-root `pub` surface crosses;
  a package-internal item (not re-exported from the public root) is walled with the
  existing E0903-class error. The **name-collision rule** — dep local names vs the
  root's top-level `src/` module names must be disjoint (E0930) — with the
  `package = "…"` alias as the fix.
- **Injective pkgid mangling (§5, review F2).** Every item's mangled name is
  prefixed with an injective `<name>#<source-hash>` pkgid, **including the root
  package's own items**, so a local module can never collide with a transitive
  dependency's package name and two same-named modules from different packages never
  merge into one DAG node (securing acyclicity, F6b). The `#` separator cannot occur
  in a source identifier, so the pkgid segment cannot alias any module name. **The
  pkgid is applied only in a multi-package build:** a manifest-less or
  dependency-free package takes the unchanged single-package merge path, and the
  root entry `fn main` keeps the bare global `main` in both, so **single-package /
  manifest-less builds are observably identical** (verified: self-host byte-exact
  gates + native/wasm corpus green).
- **Build driver (§7).** `modules::build_tree` routes a dependency-having package to
  `build_tree_multi`: discover each package's `src/` tree under its pkgid prefix,
  resolve cross-package `use`, and merge into one pkgid-qualified `Program` fed
  unchanged to the checker/interp/codegen. So `candor check`/`run`/`compile` on a
  manifested package with dependencies works across every engine.
- **Gate (`compiler/tests/packages.rs`, +8 tests, 12 total).** `app`→`b` builds and runs to
  **123 byte-exact across tree-walker/MIR/native/AOT**; the visibility wall
  (E0903), diamond unification (`dia_c` deduped once), divergent-diamond conflict
  (E0923), package cycle (E0927), name collision (E0930) + alias fix, and the
  lockfile (written with content hashes, then reused). Full `cargo nextest run`
  green (904 tests) incl. all self-host + native + wasm; `--profile fast` green
  (773); clippy clean.

**Deferred (reported):** git-dependency fetch + content-addressed cache (E0924
error stands in); the incremental `candor build` (Stage C, `build/mod.rs`) does not
yet resolve deps — it builds only the root `src/` tree, so cross-package `use` there
is unresolved (the resolver/merge lands in `build_tree`, used by
check/run/compile). The `candor audit` whole-graph trust aggregation (0017 §8 /
F1b) is a separate queued slice, not this one.

## OBL-0017-AUDIT-GRAPH — packaging slice: `candor audit` walks the WHOLE resolved dependency graph (2026-07-15)

Closes the cross-package half of review F1b (0017 §8): `candor audit` on a
manifested package that declares dependencies now enumerates every package's
trust surface across the **whole resolved graph** and attributes each finding to
its package + version + source (P16 provenance). A dependency's `foreign` externs
and `unsafe` regions are visible and traceable to it — a dep (or a dep-of-a-dep)
cannot hide I/O from the consumer's audit. Composes the two landed pieces (it
re-implements neither): the resolver (`resolve_pkg::resolve`, OBL-0017-RESOLVE)
for the pinned package set + provenance, and the per-package audit walk
(`audit.rs`, boundary externs/exports + effect-reach + `unsafe` regions).

- **Detection (`compiler/src/audit.rs`).** `audit_path` routes to the graph
  aggregation only when the target is a directory with a `candor.toml` that
  declares ≥1 dependency; a bare file, a manifest-less directory, or a
  dependency-free package takes the **unchanged** single-package walk
  (`audit_program`, the former `audit_path` body, refactored to return the report
  struct).
- **Aggregation + attribution.** For each package in the pinned set (root + every
  transitive path dependency) the per-package walk runs over that package's `src/`
  tree; the result is tagged with the package `name`, `version`, and `source`
  (canonical path; git url+rev when git lands). Additive JSON schema: the top-level
  `boundary_modules`/`effect_reach`/`unsafe_regions`/`summary` remain the **root
  package's** local surface (the exact single-package shape, so existing readers
  and the `ffi_audit`/`bump.cnr` goldens are unaffected); a new `packages` array
  carries the per-package attribution layer.
- **Git deps** keep erroring cleanly (E0924, "not yet fetched") via the reused
  resolver — the git surface audits once git-fetch lands.
- **Gate (`compiler/tests/packages.rs`, +2 tests).** `audit_app` → `b` → `c`:
  `candor audit audit_app` enumerates b's `foreign` extern (`b_native_read`) AND
  its `unsafe` region (`b::b_read`), each attributed to package `b` v0.2.0 at its
  canonical `audit_b` source; transitive `c`'s surface (`c_native_write`,
  `c::c_write`) surfaces attributed to `c` v0.3.0. A dependency-free package audits
  with the unchanged single-package shape (no `packages` layer). Full
  `cargo nextest run` green (906 tests, +2) incl. all self-host + native + wasm
  gates; `--profile fast` green; clippy clean. The `ffi_audit`/`bump.cnr` single-
  program audits in `tests/foreign.rs` are byte-unchanged.

**Deferred (reported):** the per-dependency **trust summary in `candor.lock`**
(0017 §8 — counts of boundary modules / externs / `unsafe` regions per resolved
dep, so a lock update surfaces a trust delta). Adding it would couple the resolver
to the audit walk (the lock is written inside `resolve()`, which must stay
audit-free) and would perturb the lock's `PartialEq` reuse check; the required
deliverable is the audit-command aggregation, which is independent and complete.
A *gating* trust-delta diff on lock updates remains a first-1.0 need (0017 Open-Q1).

## OBL-0017-GIT-FETCH — packaging slice: git dependencies are real — fetch into a content-addressed cache, build like a path dep (2026-07-15)

Makes `{ git = "URL", rev = "<sha>" }` dependencies real (design 0017 §4/§6):
the resolver **fetches** a git dependency into a toolchain-managed,
content-addressed cache and then treats its checkout exactly like a path
dependency — same transitive-load, unification, merge, build, and audit paths.
Replaces the E0924 "not yet fetched" deferral (OBL-0017-RESOLVE). Reproducibility
comes from the lock's pinned commit sha, never from checked-in copies.

- **Fetch (`compiler/src/pkg_fetch.rs`).** Shells out to the system `git`. Cache
  layout under a root that defaults to a per-user cache dir and is **overridable by
  `CANDOR_CACHE_DIR`** (so tests use an isolated temp cache, never a real one):
  `git-db/<url-hash>/` is a bare mirror of the url (for resolving refs);
  `git-src/<url-hash>-<sha>/` is a **pristine checkout at the exact commit** with
  its `.git` removed — plain read-only source that survives mirror eviction.
  Content-addressed by (url, resolved sha): a bare full-sha `rev` whose checkout is
  already cached returns with **no git invocation at all** (the reuse fast path); a
  moving ref forces mirror consultation. Mirror clone and checkout are built in a
  unique sibling temp dir then **atomically renamed** into place, so a partial
  fetch is never observed and concurrent resolves cannot corrupt the cache.
- **Tag/branch → sha (done, not deferred).** A `tag`/`branch` written in the
  manifest is resolved against the mirror (`git rev-parse <ref>^{commit}`) to its
  full commit sha; that resolved sha is what the lock records, so the build pins to
  the commit even when the manifest names a moving ref (design 0017 §4).
- **Integration (`compiler/src/resolve_pkg.rs`).** `resolve_source` returns the
  package dir + its `ResolvedSource`; a git dep's checkout dir feeds the existing
  BFS/unification/merge unchanged. Two deps at the same (url, sha) resolve to the
  same content-addressed dir and unify to one build node; the same name via a
  different sha/url is the existing E0923 hard conflict. Package identity/pkgid and
  the `src/`-content hash use the fetched checkout (the F2 scheme, unchanged).
- **Lockfile (§6).** A git package's record is `{ git = <url>, rev = <resolved
  sha>, content_hash }`. On a second resolve the fast-path reuse yields the same
  resolved sha + content hash, so a present lock is **reused verbatim**.
- **Errors (not panics).** `git` unspawnable → E0931 (clear "is git installed?");
  clone failure (bad/unreachable url) → E0932; a rev/tag/branch that resolves to no
  commit → E0933; cache-dir/publish failure → E0934. Each is prefixed with the
  offending dependency's local name.
- **Whole-graph audit over a git dep.** Unchanged `audit.rs` — because git deps now
  resolve, a git dependency's `foreign`/`unsafe` surface is enumerated and
  attributed to it (name + version + **git source: url + resolved sha**) through the
  reused resolver (review F1b / §8).
- **Gate (`compiler/tests/packages.rs`, +4 tests, HERMETIC + offline).** Each test
  builds a **local** git repo in a temp dir (no network) and points
  `CANDOR_CACHE_DIR` at a temp cache; if `git` cannot be spawned the tests skip
  cleanly with a reason. `app` → `b` via `{ git = file://…, rev = <sha> }` fetches
  into the temp cache and builds+runs to **123 byte-exact across
  tree-walker/MIR/native/AOT**; the checkout is a pristine `.git`-free package;
  `candor.lock` records b's git url + resolved sha + content hash; a **second build
  reuses the cache** — proven by deleting both the source repo and the mirror db,
  after which the build still succeeds from the content-addressed checkout alone. A
  `tag` dep resolves to and locks the underlying commit sha. A bad git url fails
  E0932 (clean diagnostic). `candor audit` over a git-fetched dep enumerates its
  `foreign` extern + `unsafe` region attributed to the git source. Full
  `cargo nextest run` green incl. path-dep + self-host + native + wasm; clippy clean.

**Deferred (reported):** trust-delta **gating** on lock updates (0017 Open-Q1,
first-1.0) and the incremental `candor build` (Stage C) resolving deps — both
unchanged by this slice. A moving-ref build does not re-fetch a present mirror
(rev-pinned reproducibility; re-resolution is a manifest-change action, §6).


## REALLOC-ABI — a `realloc` slot on the allocator vtable; native grow paths rewired (2026-07-16)

Ratified fork (deciding authority, 2026-07-16). `AllocVtable` grew a third slot
`realloc(ctx: rawptr u8, ptr, old_size, new_size, align) alloc -> rawptr u8`
(same ctx/align convention as `alloc`/`free`; `{alloc, free}` unchanged).

- **Implementors.** `std::freelist` grows IN PLACE when it can — extend the bump
  frontier when the block ends there, else absorb the physically-adjacent free
  block beginning exactly at `ptr + old_span` (reusing the coalescing invariants,
  splitting its tail when the remainder is >= MIN_SPLIT) — otherwise `alloc(new)`
  + copy + `free(old)`. Shrink keeps the block in place. `std::bump` cannot
  reclaim, so it always `alloc(new)` + copy + `free(old)` (a no-op free for a real
  bump — old space leaked, bump semantics); this uses the arena's own
  `bump_alloc`/`bump_free`, so an instrumented `bump_free` (the `live`-counter
  fixtures) stays balanced exactly as the prior alloc-copy-free grow did. `pool`
  (11_1 demo) gets an alloc+copy+free `realloc` for vtable completeness. Best-fit
  stays DEFERRED.
- **Native grow paths rewired.** `String`/`Vec` growth in all four grow lowerings
  (tree-walk `interp/eval.rs`, `mir/interp.rs`, Cranelift `backend/lower.rs`, LLVM
  `backend/llvm.rs`) now calls `realloc` instead of manual alloc-new + copy +
  free-old (a first grow with a null buffer still calls `alloc`). `old_size` is
  passed as the LIVE byte count (`len`/`len*stride`), not capacity: this bounds
  the copy to initialized bytes (the interpreter's init-byte guard faults on
  reading the never-written `[len, cap)` tail — capacity would read uninitialized
  memory), and at a `Vec` grow `len == cap` so the freelist's span/frontier logic
  stays exact. `Map` growth is NOT rewired — a hash-table rehash reorganizes slots
  by capacity and needs a fresh zeroed buffer distinct from the old one during
  migration, which a byte-preserving `realloc` cannot express; it keeps
  alloc-new(zeroed) + rehash + free-old.
- **Soundness UNCHANGED.** The grow-invalidates-views loan rule (STR-VIEW-UAF /
  0015 F1 residual, obligations above) already forbids holding a borrowed view
  across a grow statically, independent of whether `realloc` moved the buffer, so
  `realloc` introduces no new UAF and no loan rule was weakened.
- **Gate.** Byte-exact across all five engines (tree-walk, MIR interp, Cranelift
  no-opt/opt, LLVM `-O2`): all `String`/`Vec`/`Map` + freelist + std_io growth
  fixtures stay green and identical, plus a focused `realloc` gate
  (`tests/fixtures/run/freelist_realloc.cnr`, `freelist.rs`) that direct-drives
  BOTH physical paths — frontier in-place grow (same pointer) and move-copy with
  the following block occupied — asserting the payload survives byte-for-byte.
  Every `AllocVtable` construction site + implementor across corelib, fixtures,
  and the self-host sources (`analyses.cnr`/`checker.cnr`) carries the complete
  three-slot vtable; the grow-free 0001 §11 demo fixtures (`11_1_allocator`,
  `11_4_parser`, `11_5_arena`) keep their two-slot vtables (never exercise a grow;
  their golden item-counts and `.cn`/migrator pairing are unchanged). Full
  `cargo nextest run` green (929 tests) incl. self-host interp+lower corpus, AOT,
  LLVM fifth-engine, wasm; clippy `--all-targets` clean.

## OBL-EDITION-REHEARSAL — P15 edition/migrator machinery exercised end-to-end (1.0-gate item 1) (2026-07-21)

1.0-gate item 1 (`docs/1.0-GATE-TRIAGE.md` row 1): P15's promise that "the
evolution mechanism works before it is relied on." Before this slice the manifest
had a required, validated `edition` field with **only `"2026"` legal**, and the
edition never reached the front-end — no second edition, no retained old
front-end, no breaking-change migrator had ever run. This slice proves the
machinery with a **REHEARSAL** edition (`"2027-rehearsal"`, `manifest::REHEARSAL_EDITION`),
deliberately synthetic and clearly marked so nobody mistakes it for a shipped
language change; it can be retired once a real edition relies on the machinery.

- **The plumbing the design assumed but did not have.** The `edition` field
  validated but was **never threaded to the parser** (`resolve_dir_root`,
  `resolve_pkg`, and `build_dir` all loaded the manifest yet parsed with a fixed
  front-end). This slice adds that plumbing: `manifest::Edition` flows from each
  package's manifest through the module builders (`modules.rs`, `build/mod.rs`),
  the resolver (`resolve_pkg.rs`), and the audit walk (`audit.rs`) into the real
  lexer (`real::lexer::lex_in` / `real::token::real_keyword_from_str`). The
  **parser is edition-agnostic** — both keyword spellings lex to the same token —
  so the fork lives in exactly one place (the lexer keyword table).
- **The breaking change (surface-only, byte-identical semantics, 0017 F4).** In
  `2027-rehearsal` the mutability keyword `mut` is respelled `mutable`; `mut` is
  demoted to an ordinary identifier. (a) The 2026 front-end still accepts `mut`
  (old editions keep compiling), (b) the 2027-rehearsal front-end accepts ONLY
  `mutable` (it IS a breaking change), (c) both spellings map to `RKw::Mut` — the
  same AST — so the program means the same thing across the boundary.
- **The automatic migrator.** `candor migrate-edition <pkg-dir>` (and
  `candor::migrate_edition_dir`): a token-driven, formatting-preserving rewrite of
  every `.cnr` under `src/` (`mut` keyword tokens -> `mutable`, leaving `mut` in
  comments/strings/identifiers untouched) plus a line-oriented manifest edition
  bump. Fully automatic and **idempotent** (an already-migrated package is a
  reported no-op). Honest scope note: a shipped keyword rename would also need to
  alpha-rename any existing identifier equal to the new keyword; the rehearsal
  fixtures use none, so the migrator does not.
- **Verified byte-identical across engines.** The migrated (2027) package checks
  clean and returns identically to the original (2026) on the tree-walker, the MIR
  interpreter, the Cranelift native engine, and the AOT executable
  (`tests/editions.rs`).
- **Cross-edition linking, both directions (0017 F4).** A 2026 app depending on a
  2027-rehearsal library (using `mutable`) and a 2027-rehearsal app (using
  `mutable`) depending on a 2026 library (using `mut`) both link, check clean, and
  run across every engine — each package parsed under its own manifest edition, the
  merged interface artifact edition-agnostic.
- **Gate.** `tests/editions.rs` (9 tests, CI-gated with the suite): old edition
  compiles+runs; new edition rejects the old spelling and accepts only the new;
  old edition rejects the new spelling; unknown edition still errors (M0102);
  migrator is automatic + idempotent + byte-identical across engines; migrate
  source is idempotent; cross-edition deps link both directions. Full
  `cargo nextest run` green; clippy `--all-targets` clean. ROADMAP left to the
  deciding authority.
