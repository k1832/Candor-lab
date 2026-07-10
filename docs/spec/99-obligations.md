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

### Slice 3 addendum — the type checker (name resolution + type-error core in Candor, 2026-07-10)

Writing the self-hosted CHECKER (`prototype/selfhost/checker/checker.cnr`, composed after
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

Writing the self-hosted MOVE/INIT analysis (`prototype/selfhost/analyses/analyses.cnr`,
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
`prototype/tests/selfhost_checker.rs` (`candor_checker_checks_lexer_source_clean_fixpoint`), reusing
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
work, still region-free. Prototype: RefIndexed wired for Vec; a user impl should typecheck via
the compact default (not yet exercised end-to-end).

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
instead of concatenating sources. Payoff gate added to `prototype/tests/selfhost_checker.rs`
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
`prototype/tests/selfhost_analyses.rs` (`candor_analyses_check_lexer_source_clean_fixpoint`,
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
`prototype/tests/selfhost_checker.rs::candor_checker_checks_analyses_source_clean_via_import_resolution`
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
`prototype/tests/selfhost_analyses.rs::candor_analyses_check_analyses_source_clean_fixpoint`
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

The first self-interpreting slice: `prototype/selfhost/interp/interp.cnr`, a tree-walking
SCALAR interpreter WRITTEN IN CANDOR, executes an in-subset Candor program directly over the
self-hosted parser's Node arena and reproduces the Rust reference interpreter
(`prototype/src/interp/`) BYTE-EXACT — same `main` return, same `trace` sequence, same fault
identity (kind + span). Gated by EXECUTION equality in `prototype/tests/selfhost_interp.rs`
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

The self-interpreter (`prototype/selfhost/interp/interp.cnr`) gains a FLAT BYTE-MEMORY model
plus STRUCTS and ARRAYS, extended to match the Rust reference (`prototype/src/interp/`) byte-exact
on aggregate programs. Same gate (`prototype/tests/selfhost_interp.rs`, EXECUTION equality vs
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

The self-interpreter (`prototype/selfhost/interp/interp.cnr`) gains the MOVE/DROP SCHEDULE —
the observable-trace crux of the language — reproducing the Rust reference (`src/interp/eval.rs`)
byte-exact on drop-bearing programs. Same gate (`prototype/tests/selfhost_interp.rs`, EXECUTION
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

The self-interpreter (`prototype/selfhost/interp/interp.cnr`) gains ENUM VALUES and `match`,
reproducing the Rust reference (`src/interp/{layout,eval}.rs`) byte-exact. Same gate
(`prototype/tests/selfhost_interp.rs`, EXECUTION equality vs `run_source_real`): the corpus grows
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

The self-interpreter (`prototype/selfhost/interp/interp.cnr`) gains the machinery `box` needs
but NOT box itself (that is S5b): `rawptr`/`fn`-ptr as scalar values, top-level `static`
evaluation, fn-name-as-value, INDIRECT calls through a fn-ptr, structural `Alloc`/`AllocVtable`
identification, and the minimal raw-pointer surface. Same gate
(`prototype/tests/selfhost_interp.rs`, EXECUTION equality vs `run_source_real`): the corpus grows
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

The self-interpreter (`prototype/selfhost/interp/interp.cnr`) gains THE HEAP on top of S5a's
allocator-ABI foundation: `box`/`unbox`, the compiler-known `BoxResult` enum, `.*` Box-deref
(and field/index auto-deref THROUGH a Box), and alloc-on-drop. Same gate
(`prototype/tests/selfhost_interp.rs`, EXECUTION equality vs `run_source_real`): the corpus grows
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

The self-interpreter (`prototype/selfhost/interp/interp.cnr`) swaps its flat 16 KiB byte arena for a
PAGED backing store and adds the three pointer intrinsics the systems corpus needs — the
INFRASTRUCTURE for that corpus; the five corpus programs (11_1..11_5) are S6b, a separate slice.
Same gate (`prototype/tests/selfhost_interp.rs`, EXECUTION equality vs `run_source_real`): the corpus
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

The self-hosted Candor interpreter (`prototype/selfhost/interp/interp.cnr`) now EXECUTES all five
systems-corpus programs — Candor's hardest real programs — each byte-exact against the Rust reference
(`run_source_real`). Same gate (`prototype/tests/selfhost_interp.rs`, EXECUTION equality): the corpus
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
byte-exact, clippy clean). Four review-specified cleanups to `prototype/selfhost/interp/interp.cnr`:
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
- EMBED: the self-host lexer produces 25736 tokens (< the 32768 buffer, ~7k headroom) and the parse
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
- `prototype/tests/selfhost_checker.rs::candor_checker_checks_interp_source_clean_via_import_resolution`
  — the self-host CHECKER name-resolves interp.cnr clean (empty E0102/E0103), byte-equal to the
  module-aware oracle over the real lexer+parser+checker+interp tree. Teeth: the naive single-file
  check flags the unresolved imports E0102 (>0), and an injected unknown-type param fires E0102.
- `prototype/tests/selfhost_analyses.rs::candor_analyses_check_interp_source_clean_fixpoint`
  — the self-host ANALYSES (move/init E0301/E0304, loans E0801-4, effect E0401, exhaustiveness E0601)
  emits an EMPTY covered set over interp.cnr, byte-equal to the module-aware oracle. Teeth: an
  injected use-after-move fires E0301.

Both pass CLEAN, oracle-matched, exactly as the probe predicted — no construct in interp.cnr fails to
check clean. NO arena bump needed (25736 tokens < the 32768 token buffer / node arena, ~7k headroom).
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

- WIRE FORMAT (`prototype/src/mir/serial.rs`): a canonical, deterministic, human-readable
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

- GATE (`prototype/tests/mir_serial.rs`): for each corpus fixture — lower source to MIR
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
of the Rust reference lowering (`prototype/src/mir/build.rs`).

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

- GATE (`prototype/tests/selfhost_lower.rs`): for each in-subset fixture — run `lower.cnr`
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
aggregate half of the Rust reference lowering (`prototype/src/mir/build.rs`): struct/array
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

- GATE (`prototype/tests/selfhost_lower.rs`, S2 fixtures added to the existing corpus): each
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
