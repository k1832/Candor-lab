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
