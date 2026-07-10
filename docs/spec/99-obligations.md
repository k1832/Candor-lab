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
