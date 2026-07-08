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
- **Chapter:** 06 §7 (SKELETON).
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
- **Chapter:** 05 §6 (SKELETON).
- **Hook:** **P18** (named mandatory spec scope); NN#1 (which quietly rests on it).
- **Gate:** blocks any optimizing implementation's soundness claim and any
  stability commitment.
- **Acceptance:** the spec states, for every unsafe operation of chapter 05 §2,
  the optimizer assumptions it preserves or breaks — notably the materialization
  question (05 §6.2) — composed with chapter 09; Rust's Stacked/Tree Borrows
  studied as cautionary art (05 §6.3); mechanized where feasible, rigorous-
  informal at minimum.

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

### OBL-GENERICS — user-defined generics
- **Chapter:** 03 §1.5 (excluded this edition).
- **Hook:** **P11** (definition-site-checked generics with predictable
  instantiation).
- **Gate:** blocks generic user types/functions; the compiler-known parametric
  types (`Box`, `slice`, `[N]T`, ...) are unblocked.
- **Acceptance:** the spec defines the interface/bound system, definition-site
  checking, coherence, and the documented, stable-within-edition instantiation
  strategy.

### OBL-TEXT — the text-type budget
- **Chapter:** 03 §8.3 (byte slices only this edition).
- **Hook:** **P3** (one canonical way; the named string-sprawl stress point).
- **Gate:** blocks any owning/interop text type beyond `slice u8` / `[N]u8`.
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
  `f(write (deref b))` / `f(read (deref b))` to `f(b)` exactly where the argument
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
§3.4).

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

## OBL-WINDOW status update (2026-07-08)

The rigorous-informal single-threaded core is drafted, adversarially reviewed (three
theorem-breaks found and repaired: docs/reviews/2026-07-08-fault-window-review-1.md), and
revised (docs/spec/drafts/fault-window-formalization.md). Discharged: Containment,
Prefix-determinism, NN#1/NN#5 preservation, and the collapse-to-precise limit under a sound
effect-order-total reordering license. Open: concurrency composition (stated as conjecture with
obligations O1-O5), mechanization, compiler correctness, and the fault-torn multi-store
transaction residual (deferred to the volatile-access design).

## OBL-MINMAX-INTRINSICS (found by editor-support work, 2026-07-08)

Spec 01 §2.3 lists min_of/max_of as normative intrinsics; the real front-end defers them (stage-1
report) and the lexer table omits them. Resolve by implementing the two intrinsics (trivial
compile-time constants) or downgrading the spec entry to reserved; the grammar highlights them
per spec meanwhile. Gate: chapter 01 NORMATIVE promotion for that clause.

## OBL-GENERICS-ITER evidence update (stdlib seed, 2026-07-08)

The seed gathered the concrete friction: (a) no closures/higher-order inference makes Opt::map
unwritable (E1002: U uninferable from a fn-pointer return; no turbofish; the opaque-drop alloc
tax compounds it); (b) no swap/replace plus E0303 plus struct-only drop hooks make "mutable
container with a user drop hook" mutually exclusive - containers free structurally or hooks live
on wrapper cells. Both feed the iteration/associated-types design round.
