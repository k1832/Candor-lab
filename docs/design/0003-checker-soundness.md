# 0003 — Checker Soundness Argument (Bet 5 Validation Prototype)

**Status:** draft
**Date:** 2026-07-07
**Philosophy hooks:** NN#1 (no UB in safe code), NN#5 (no reads of uninitialized memory),
NN#7 (no hidden allocation), P1 (unsafety explicit/local/auditable), P12 (values-first
memory model), P18 (semantics defined by a spec, not the compiler — served here in miniature).
**Subordinate to** `LANG_PHYLOSOPHY.md` and to design `0001-memory-model.md`; where either
outranks this document and conflicts, this document is the artifact that changes.

**Revision note (2026-07-08).** §2.2, §2.5, §3 (#4), and §0 amended to agree with accepted designs 0004 (`field_ptr` + E0510) and 0005 (implicit call-site reborrow), per `docs/reviews/2026-07-08-design-0004-0005-review-1.md`; the freeze hash is invalidated and a fresh-session re-review is scheduled with the implementation verification (§0).

**Why this document exists.** `docs/BET5_CRITERION.md` §3.7 and freeze step (i) make a written,
independently reviewed soundness argument a *precondition of admissibility*: "counts produced by a
checker without that reviewed argument are inadmissible." The reason is measurement, not ceremony —
a permissive checker mechanically lowers the annotation, valve, and copy counts Bet 5 turns on, so a
count is only meaningful if the checker that accepted the code was sound. This is the argument. It is
a **rigorous informal argument, not a machine-checked proof** (P18's honest posture): the prototype
is throwaway and the mechanized formalization P18 mandates is later, stability-gated work.

---

## 0. A hole found while writing this argument (read first)

Writing the "no runtime drop flags" discharge (§2.1) surfaced a defect. **It is not a violation of
the safety claims (a)–(e) of §1 — the interpreter executes memory-safely — but it refutes an
auxiliary claim design 0001 §1.5 makes and this document was asked to discharge, so it is recorded
prominently per the project's honesty norm.**

Design 0001 §1.5 asserts "the interpreter needs **no runtime drop flags**" and §10.6 rejects
"conditional partial moves via runtime drop flags" specifically to keep the drop schedule static.
The stated mechanism is §1.6's rule that *move* state must agree at every join. That rule is
enforced (E0302) and does make the **move** dimension path-independent. But **conditional
*initialization* is not a move**, and the checker permits a local that is initialized on one branch
and uninitialized on another provided it is never read afterward (§7.4: `MaybeInit` is legal until
read; `join_st` in `check/dataflow.rs` returns `MaybeInit` with `disagree=false` for `Init`/`Uninit`).
Such a local's drop obligation at scope exit **is** path-dependent, and the interpreter resolves it
by consulting a per-value runtime ownership mask (`MoveMask`, `interp/eval.rs`) — which is a drop
flag in exactly the sense §1.5/§10.6 claim to avoid.

Confirmed end-to-end against the built prototype:

```
struct R { v: i64 } drop(write self) { trace((deref self).v); }
fn f(c: bool) -> unit { let x: R; if c { x = mk(7); } return; }   // ACCEPTED, no diagnostics
```

`f(true)` drops `R` (trace `[7]`); `f(false)` does not (no trace). The drop decision reads the
runtime mask; it is not a static fact of the program point. Whether this is fixed by **correcting
design 0001 §1.5** (concede the interpreter carries an initialization-flag for the conditional-init
case) or by **adding a checker rule** that rejects a local owned-on-some-path-only at scope exit
(making drop truly static, consistent with §10.6's intent) is for the independent reviewer and the
deciding authority to adjudicate. It does not, on its own, make any accepted program memory-unsafe.

**RESOLVED (2026-07-07, in the strict direction; `docs/reviews/2026-07-07-drop-flag-finding.md`).**
The deciding authority accepted the finding and chose the checker rule over amending §1.5. The checker
now enforces the **dual of §1.6's move-join rule** (0001 §1.6 rule 3, §7.4): at a place's drop point
(any scope exit, including via `return`/`break`/`continue`), a **needs-drop** place — one whose type
has a `drop` hook or transitively contains a drop-hooked type or a `Box` — whose initialization state
is path-dependent (`MaybeInit`) is rejected as **E0309**, while drop-inert types stay exempt. Move
state must still agree (E0302, unchanged), so the drop schedule for needs-drop values is now fully
static in both the move and the initialization dimension; the interpreter's `MoveMask` consultation at
scope exit is a mechanism for exempt types and an internal debug assertion for needs-drop ones, never a
semantic drop-flag decision. §1.5/§10.6's "no runtime drop flags" claim therefore holds as written for
safe code, and §2.1's discharge is complete. The example above (`f(c)`) is now a checker error.

**EXTENDED (2026-07-07, soundness review #1 finding 2; `docs/reviews/2026-07-07-soundness-review-1.md`).**
The re-review found the first resolution *incomplete*: it closed the **scope-exit** drop point but not the
**reassignment** drop point, though §1.5 defines a reassignment as also dropping the value the place
currently holds. A needs-drop place `MaybeInit` at a *whole-binding reassignment* was still accepted, and the
interpreter still consulted the runtime `MoveMask` to decide the old value's drop — the same drop flag, at a
second drop point. The deciding authority accepted the finding in the strict direction again: **E0309 now
covers the reassignment drop point too** (and the caller-side `out`-argument drop point, which §3.1 makes the
same drop site), with a message distinguishing "at reassignment"/"as `out`" from "at scope exit". Design 0001
§1.6 rule 3 now lists **both** drop points. With this extension the "no runtime drop flags" claim holds for
*every* drop point of a needs-drop value — scope exit (via `return`/`break`/`continue` or block end) and
reassignment/`out`. The full history is kept visible per the ledger discipline: the mechanism claim was found
false (§0 top), resolved for the move dimension and the scope-exit init dimension (RESOLVED), then completed
for the reassignment/`out` init dimension (this note).

**RETEST FAILED, THEN RESOLVED (2026-07-07, soundness re-review #2; `docs/reviews/2026-07-07-soundness-review-2.md`).**
The retest of the finding-1/2 repair (freeze step (i)) failed in a fresh session: it found a **new accept-invalid**
adjacent to review #1 finding 3. `init.rs::apply` treated a **move of an opaque place** (one whose projection
crosses `Proj::Deref` or `Proj::Index`) as a plain read of the root — it never set `Moved` and never ran the
drop-hooked-partial check — while the interpreter divergently marked the whole root moved. A non-copy value could
therefore be moved out through a `deref` (`let taken = (deref bx).inner;`) and the box then `unbox`-ed, running a
run-exactly-once drop hook **twice** (double-free, NN#1); the same held through a `write` borrow with no `Box`, and
for a drop-hooked array element by constant index (bypassing §1.6). A sibling symptom was a false **E0401** (a
non-`alloc` function partially moving through a box deref reported as "frees" though the runtime leaks). Root cause:
design 0001 never defined a move of a non-copy value out through a `deref`. **RESOLVED in the strict direction** by
the deciding authority: **moving a non-copy value out of any place whose path contains a `deref` or index is
rejected (new checker error E0310)** — `unbox` is the defined `Box` extraction, a borrow's deref was only ever a
copy-or-reborrow, and array index-granularity is beyond the place model (0001 §1.6 narrowed to `copy` element types;
§2.1 gains the deref clarification). The checker now also marks the whole root moved at the opaque move, so it and the
interpreter agree on the (rejected) program's move state, and the interpreter's whole-root opaque-move path is
unreachable for accepted programs (a `debug_assert` records this). §2.1/§2.4 corrected; §3 conservatism #5 corrected
(it had described the interpreter's mark-root-moved as `init.rs` behavior). A retest #3 in a fresh session follows;
counts remain inadmissible until it passes.

Retest #3 (fresh session, 2026-07-07) **failed** with a new accept-invalid at the one seam no prior
review touched: **contracts × dataflow**. `check_fn` type-checked `ensures` with `cur = None` — off the
CFG — so a clause's reads/moves/borrows reached **no** analysis, while the interpreter evaluated the
clause after the body and before drops. A function that `unbox`-ed its `Box` param and whose `ensures`
dereferenced that param checked clean and read freed heap at runtime; the same held for reads of any
body-moved (incl. drop-hooked) param. `requires` was unaffected (checked in the entry block). This
falsified claims (a)/(c) for contract expressions. A sibling observation: assignment to a `static` was
accepted and silently mutated it. **RESOLVED** by the deciding authority: **(1) contract clauses are
read-only** — no moves, `write`-borrows, `out` arguments, or consuming/mutating calls inside a
`requires`/`ensures`/`assert` condition (new checker error **E0708**); **(2) `ensures` accesses are
analyzed against the post-body state at every normal return** — a read of a moved/consumed place is the
ordinary **E0301**, as if written at the return; **(3) statics are immutable** — assignment, `write`-borrow,
or `out`-passing of a static is a new checker error **E0311**. Design 0001 §7.3 gains rules (1)/(2) and
§8.2 gains rule (3); the new §2.7 sub-argument below discharges the contract boundary. A retest #4 in a
fresh session follows; counts remain inadmissible until it passes.

**HASH TRIPWIRE FIRED (2026-07-08; designs 0004/0005, `docs/reviews/2026-07-08-design-0004-0005-review-1.md`).** Design 0005 (implicit call-site reborrow) contradicts this document's §3 conservatism #4 as hashed ("explicit call-site reborrows required; no implicit call-site reborrow"), and design 0004 adds a new safe op, `field_ptr`, to the §2.5 boundary. Both are soundness-relevant changes to the accept/reject boundary, so the freeze-step-(i) hash is invalidated exactly as the tripwire (Consequences and costs) intends. Applied in this change series: §3 #4 rewritten, the implicit-reborrow **desugaring rule** added to §2.2, and `field_ptr` + its **E0510** well-formedness rule added to the §2.5 safe carve-out. Per the review's disposition (findings 3, 5), **a fresh-session re-review of this document is scheduled as part of the implementation verification pass, before the scheduler re-port** — counts stay inadmissible until that re-review passes, the same standing rule the retests above enforce.

---

## 1. The claim, stated precisely and honestly

Let *P* be a program the checker accepts with zero error diagnostics, and let *E(P)* be its execution
by the prototype interpreter (`interp/`). Soundness is the conjunction, over all such *P* and all
inputs, of the following, **everywhere outside `unsafe` regions**:

- **(a) No uninitialized reads (NN#5).** No typed read in *E(P)* observes storage that has not been
  written on the path taken to that read.
- **(b) XOR aliasing (NN#1/§2.2).** No place is accessed in violation of the shared-xor-exclusive
  borrow discipline: while an exclusive loan of a place is live, the place is touched only through
  it; while a shared loan is live, the place is not written or exclusively re-borrowed.
- **(c) No use after move/drop (§1.2/§1.5).** No value is read, borrowed, or moved after its owner
  was moved out or dropped.
- **(d) No allocation in a non-`alloc` context (NN#7/§6.3).** A function whose signature lacks the
  `alloc` effect performs no allocation in *E(P)*.
- **(e) Faults are delivered, not stepped over (§7).** On the defined fault conditions (overflow,
  div-by-zero, bounds, conv-loss, failed `assert`/`requires`/`ensures`, `panic`), *E(P)* halts at the
  faulting operation rather than producing a value derived from it.

**Soundness is with respect to the prototype interpreter's semantics** as the operative model of
design 0001 (P18: the spec, not the compiler, is the arbiter — but the prototype has no independent
mechanized spec, so the interpreter *is* the model here, and this is stated so it is not mistaken for
more).

**What is NOT claimed.**

1. **Nothing inside `unsafe`.** Raw-pointer operations (`ptr_read`/`ptr_write`/`ptr_offset`/
   `addr_of`/`cast_ptr`/`addr_to_ptr`) are unchecked by construction (§4.2). Their correctness is the
   author's declared obligation carried in the justification string; that transfer *is* the valve's
   meaning. Claims (a)–(e) hold for the safe fragment; they say nothing about a bad `ptr_read`.
2. **No machine-checked proof.** This is an informal argument keyed to the implementation, reviewed
   by a human/LLM session, not a mechanized theorem.
3. **No claim beyond the interpreter's model.** The prototype has no optimizer and traps precisely
   (§7.1); it does not exercise P5's fault window. Soundness here is soundness against *this* model.
4. **The §0 auxiliary-claim ("no runtime drop flags")** is now *discharged* for the safe fragment, at every
   drop point of a needs-drop value (scope exit and reassignment/`out`), following soundness review #1 findings
   1–2; see §0 and §4. The residual is the general informality of the argument, not this claim.

---

## 2. The argument, per analysis

The checker runs in stages (`check/mod.rs::check_fn`): resolve → type check → **Stage 2** forward
definite-assignment + move analysis (`check/init.rs` over the CFG in `check/dataflow.rs`) → **Stage
3** backward loan-liveness + conflict scan (`check/loans.rs`), same-call overlap, all-paths-return →
the `alloc` partition (`check/effects.rs`). Loans and access classifications are produced during type
checking (`check/expr.rs`, `check/stmt.rs`, `check/patterns.rs`) and consumed by both dataflow passes
over the *same* CFG.

### 2.1 Definite assignment + move (serving (a), (c), NN#5)

`check/init.rs::analyze` is a forward **must**-analysis over the place-state lattice `St = {Init,
Uninit, Moved, MaybeInit}` with per-local place trees (`dataflow.rs::Tree`) that split on touched
fields for partial-move tracking (§1.6).

- **Reads require `Init`.** `apply` classifies each `Access`; `Read`/`Borrow`/`Move` call
  `require_init`, which emits **E0301** (use of moved value) on `Moved` and **E0304** (possibly
  uninitialized) on `Uninit`/`MaybeInit`. An opaque place (through `deref`/index, `!is_direct`) is
  required-init at its *root*, conservatively. This is (a) and the read half of (c): no accepted
  program reaches a read of a non-`Init` place.
- **A non-copy move out of an opaque place is rejected (E0310).** A `copy` value read through
  `deref`/index is an `Access::Read` (it copies), so a `Move` whose place is opaque is a genuine
  non-copy move out through a `deref` or index. `apply` rejects it as **E0310** (ruling of soundness
  review #2, 2026-07-07): through a borrow such a move would hollow out the lender's value (§2.1's
  deref read was only ever copy-or-reborrow); through a `Box` the defined extraction is `unbox`; for
  arrays index-granular move tracking is beyond the place model (§3 conservatism 5). The checker also
  marks the whole root moved at that point, matching the interpreter, so the two agree on the move
  state of the (now-rejected) program — closing the divergence that produced the double-drop and the
  false-free of review #2 findings 1–2. This makes the interpreter's whole-root opaque-move path
  (`mark_place_moved`) unreachable for any accepted program (a `debug_assert` records the invariant).
- **TOP-initialization of unvisited edges is correct at fixpoint.** A forward must-analysis must meet
  only over predecessors whose out-state is computed; `incoming` filters predecessors by `reach &&
  visited`. A not-yet-visited edge (notably a loop back-edge on the first pass) contributes identity
  (TOP), not bottom (`Uninit`) — otherwise a value initialized before a loop would be falsely
  degraded to `MaybeInit` inside the body (the **E0304** regression this guards). Soundness of the
  optimism: it is confined to the *fixpoint iteration*; the separate **reporting pass** runs after
  `visited[*]` is all-true for reachable blocks, so it meets over **every** reachable predecessor. A
  genuinely uninitialized path therefore still surfaces as `MaybeInit`/`Uninit` and still triggers
  E0304 — the optimism accelerates convergence without hiding a real uninit path.
- **Move agreement at joins is enforced (E0302).** `join_st` flags `disagree` when a live value meets
  a moved-out one (`Init`↔`Moved`, `MaybeInit`↔`Moved`), reported as **E0302** at the join span. This
  is §1.6 rule 1: a place moved on one incoming path and live on another is rejected. Empirically
  load-bearing — `let x = R{..}; if c { sink(x); }` is rejected E0302.
- **Partial move out of a `drop`-hooked struct is rejected (E0303).** `emit_place_action` →
  `is_drop_hooked_partial` marks a field-move of a `has_drop` struct; `apply` emits **E0303** (§1.6
  rule 2). D-B from verification #1 confirms hooks attach to structs only, so the rule is
  non-vacuously scoped.
- **`out` obligations reuse this pass.** Out-params enter `Uninit` (`check_fn`); reaching a
  `Return`/`FallThrough` with a non-`Init` out-slot is **E0305**; reading one before assignment is
  **E0306** (§3.1/§7.4).

**What the drop-scheduling discharge does and does not buy.** Because move state agrees at every join
(E0302), the set of *moved* places at each program point is path-independent, and the interpreter can
collect its drop schedule from the moves it performs rather than from a per-move runtime flag — this
half of §1.5's claim holds. **It does not extend to conditional initialization** (§0): `Init`↔`Uninit`
at a join is `MaybeInit` with no disagreement, is legal absent a later read, and yields a
path-dependent drop obligation the interpreter resolves with the runtime `MoveMask`. The "no runtime
drop flags" claim is therefore discharged only for the move dimension, not in general.

### 2.2 Loan machinery (serving (b), the borrow half of (c))

Stage 3 is NLL-lite: backward liveness of borrow-carrying bindings plus a conflict scan.

- **Backward liveness (`loans.rs::Liveness`).** `transfer` computes `live_before = (live_after \ def)
  ∪ use`: `Decl` and whole-binding `Assign` (empty projection) *kill* the root; every `Read`/`Borrow`/
  `OutArg`/`Move` and every write *through* a borrow (`Assign` with a non-empty projection) is a *use*.
  A loan anchored to binding *X* is in scope precisely where *X* is live-after the accessing point.
  This is design §2.3 step 2 over a finite lattice, one worklist pass.
- **Loan creation and anchoring (`check/mod.rs`, `check/expr.rs`, `check/stmt.rs`).** Every borrow
  expression creates a loan on the **conflict-granularity** place (`Place::canonical`) tagged
  shared/exclusive, initially `Anchor::Temp` and marked *carried* by the expression value. When the
  value lands in a `let`/assignment binding, `anchor_carried` rebinds its loans to `Anchor::Binding(name)`
  so the loan lives over that binding's live range (§2.3 step 3). Distinct cases:
  - **Reborrow parent-anchoring.** `write (deref b)` / `read (deref b)` borrows a place whose
    `canonical()` collapses the `deref` to the root binding `b`, so the loan restricts **the parent
    binding** `b` for the reborrow's live range (§2.1: an exclusive reborrow suspends the parent, a
    shared reborrow freezes it to shared). Chained reborrows are covered transitively, each anchoring
    on its immediate parent.
  - **Implicit-reborrow desugaring (design 0005).** When an argument is a **place that already
    denotes a borrow** and it fills a `read`/`write`-mode parameter whose pointee type and
    shareability admit the mode, the checker **inserts a reborrow node** — `read`/`write` of `deref
    place` — as a syntactic desugaring at the front of the pipeline, then runs liveness + the conflict
    scan unchanged. Bare `b` to such a parameter is a **reborrow, not a move**; the loan it creates is
    anchored exactly as the explicit form above (parent-binding restriction over the reborrow's live
    range). No new lattice state and no coercion pass: one rule keyed on (argument is a borrow-typed
    place) × (parameter mode is `read`/`write`) × (pointee admits the mode). The §2.1 use-after-move
    diagnostic that previously fired on bare `b` has nothing to fire on. Fresh borrows of owned storage
    (`f(write x)`) are unaffected — they still wear the keyword.
  - **Call-argument loans.** `check_user_call`/`check_call` capture per-argument carried loans into a
    `call_group`; these are `Temp` loans checked by `same_call_overlaps` (below), not by the liveness
    scan — a call argument's borrow is live only for the call.
  - **`out`-loans.** `check_out_arg` creates an **exclusive** `Temp` loan on the slot for the call, so
    `f(out x, write x)` / `f(out x, read x)` collide by same-call overlap (§3.1, closing the
    two-paths-to-one-slot hole).
  - **Return-extension, named path.** When a call returns a borrow, `region_source_indices` picks the
    argument(s) the return derives from — the region-tagged params, or the sole borrow param under the
    compact default (§3.3) — and `check_user_call` re-carries those arguments' loans as the value's
    loans. Landing in a binding anchors them, so writing the source while the returned borrow is live
    is **E0803** (test `returned_borrow_extends_arg_loan`).
  - **Return-extension, inline-scrutinee path (the S1 fix).** Verification #1 found that a compact-
    default borrow-returning call used *inline as a match scrutinee*, with a non-copy payload bound as
    a **borrow** binding, dropped its return-extended argument loans at the call — letting an arm
    reassign/write/move the argument while the binding aliased it (demonstrated returning overwritten
    `999`). The fix (`check_match`): the scrutinee's carried loans are captured (`scrut_carried =
    take_carried()`) *before* the match head and, for each derived `BorrowShared`/`BorrowExcl`
    binding, re-anchored as a fresh loan on the same place over that binding's live range
    (`record_binding_loan`, `check/expr.rs` ~line 1160) — the same treatment the named-local path gets
    via `anchor_carried`. All four verifier repro shapes (reassign, write-mode call, the §11.5 arena
    shape with a non-copy payload, reborrow-of-reborrow) are now negative tests (E0803/E0801) and the
    copy-payload control (read out at the match head, loan ends there) is a positive test — see
    `tests/loans.rs`. This mechanism is covered explicitly because it is the fix the freeze depends on.
- **The conflict scan (`loans.rs::conflict_scan`, `judge`).** At every action, for every loan whose
  anchor binding is live-after and whose place `overlaps` the access, `judge` applies §2.2: a direct
  **Read** conflicts only with a live *exclusive* loan (**E0804**); a **Borrow** conflicts unless both
  are shared (**E0801**); and — closing the §2.2 hole class — a **Move** (**E0802**) and a **Write**/
  `out`-init (**E0803**) each conflict with *any* live loan, shared or exclusive. Classifying moves and
  writes as exclusive accesses is what stops a move out of, or a store to, storage a live borrow still
  views (the §2.2 rejected-program `let b = read x; let y = x;`).
- **Place overlap and canonicalization (`dataflow.rs`).** `canonical` collapses any `deref` to the
  root (a borrow/box through a pointer restricts the pointer's binding) and **truncates at the first
  index** (any `a[i]` covers the whole array `a` — no index-sensitive disjointness), while keeping
  distinct fields distinct. `overlaps` is prefix containment on the field path: `p` overlaps `p.f`;
  `p.f` and `p.g` do not. Truncating indices is a sound over-approximation — it can only *add*
  conflicts, never miss one.
- **Same-call overlap (`same_call_overlaps`, E0805).** Within one `call_group`, two argument loans on
  overlapping places that are not both shared are rejected: the prototype has no two-phase borrows, so
  `push(write v, read v[0])` is a false positive it deliberately reports rather than accepts.
- **All-paths-return (E0810).** A non-unit function with any reachable `FallThrough` terminator is
  rejected, so a returned borrow's provenance check (below) is not bypassed by falling off the end.
- **Return provenance (E0806/E0807/E0808), now TOTAL (review #1 finding 1).** `check_return_provenance`
  rejects returning a borrow of a local or of an owned (`take`) parameter, and enforces that an explicitly
  region-tagged return derives from the region's parameter — a borrow may not outlive its body (§3.3),
  checked body-locally because no region crosses a signature (NN#17). The walk is **total**: it recurses
  into **every `match` arm** and **`if` branch** (each tail must independently pass the region check) and,
  crucially, an **unrecognized** borrow-producing shape is **rejected** (provenance `None` ⇒ E0806), never
  skipped. The prior code returned on `None` for `Match`/`Block` shapes, so a borrow of a local laundered
  through a `match` escaped E0806 and the interpreter read a torn-down stack slot — the review's decisive
  accept-invalid repro (`fn pick(s) -> borrow i64 { let x; return match s { .. => read x } }`). Block tails
  carry no borrow (a block evaluates to unit/never — no tail expression, `check_block_value`), so they are
  handled as such, not skipped. The repro and its controls are regression tests (`tests/check.rs`).

### 2.3 Pattern bindings (serving (b), (c))

`check/patterns.rs` derives each payload binding's mode from how the scrutinee is held
(`HoldMode`→`BindMode`, §8.2.1), and `check_match` emits the corresponding CFG actions:

- **Owned scrutinee, copy payload → copy-read-out.** `is_copy` payloads bind `BindMode::Copy`
  (`Access::Read`, no loan, no move). This is sound because a `copy` read produces an independent
  value and consumes nothing — the scrutinee is untouched, matching §8.2.1 and the §11.5 arena case
  (every payload a `copy` scalar, so the shared loan on the scrutinee ends at the match head, freeing a
  later exclusive reborrow).
- **Owned scrutinee, non-copy payload → move.** Binds `BindMode::Move`; `check_match` emits an
  `Access::Move` on the scrutinee place (partial move under §1.6), so E0302/E0303 and the move-agreement
  machinery apply — the §11.4 parser case (moving `Box Expr` children out of an owned `BoxResult`).
- **Borrowed scrutinee → borrow binding carrying a loan.** `BorrowShared`/`BorrowExcl` bindings carry a
  loan on the scrutinee sub-place via `record_binding_loan`, in scope over the binding's live range —
  so the borrowed payload is subject to the full §2.2 scan and matching never moves out of a borrowed
  scrutinee. The inline-scrutinee re-anchoring of §2.2 is the temporary-scrutinee case of this rule.
- **Exhaustiveness (E0601), arity (E0605), wrong-variant (E0604/E0108).** Enforced by
  `check_exhaustive`/`analyze_pattern`; exhaustiveness matters for (a)/(e) because a non-exhaustive
  match would otherwise fall through with no value.

### 2.4 Effects (serving (d), NN#7)

`check/effects.rs` tracks one boolean. `note_alloc` records the first allocation site. The enumeration is
now **both sides of the allocator** (review #1 finding 4): the *alloc* side — `box`, `clone` of a
`bears_box` value, a call to an `alloc`-marked function, an indirect call through an `alloc`-typed
fn-pointer — **and the *free* side** — `unbox` (frees the box storage), and any **drop of a `Box`-bearing
value the checker schedules**: a scope-exit drop of a needs-drop box-bearing local, a reassignment-drop or
`out`-drop of one, a box-bearing temporary dying at statement end, and a function-exit drop of an owned
box-bearing parameter still live on some path. The box-drop sites are detected precisely by
`check/init.rs::analyze` against the final move state (a box moved out on every path frees nothing and is
*not* marked; a partially-moved struct is checked field-granularly, so a live box field is caught even when
a sibling was moved), and fed back through `note_alloc`. A partial move *through a `Box` deref* is no longer
a box-drop site: it is now rejected outright as an opaque move (E0310, §2.1), and the checker marks the box
moved, so the false-free that review #2 finding 2 measured (a non-`alloc` function reported as "frees" though
the runtime leaks) no longer arises — the checker and interpreter agree the box is consumed. `AllocEffect::finish` emits **E0401** if a
non-`alloc` function has any such site, naming the drop site (P4). Function-pointer types carry the effect in the
type (`types::FnPtrTy`), and `assignable`/`check_against` reject assigning an `alloc` function to a
non-`alloc` pointer (**E0402**) while permitting the reverse (upper-bound conservatism, §6.1). There is
no vtable special case: `AllocVtable` fields are `alloc`-typed, so every indirect call through them is
`alloc` by the one general rule. This makes a non-`alloc` context allocation-free: the only allocating
operations are the enumerated ones, each of which forces the marker.

### 2.5 The unsafe boundary (bounding what (a)–(e) cover)

`require_unsafe` (**E0501**) gates every operation that gives a raw pointer meaning
(`ptr_read`/`ptr_write`/`ptr_offset`/`addr_of`/`addr_of_mut`/`cast_ptr`/`addr_to_ptr`/`ptr_null`),
while holding/moving/copying/comparing a `rawptr` and `is_null`/`offsetof`/`field_ptr` stay safe
(§4.2). **`field_ptr(p, f)` (design 0004) is a new safe carve-out** in this enumeration: it computes a
field's address by static offset and gives the pointer no meaning (no read, no write, no borrow, no
fault of its own), so it joins `offsetof`/`is_null` on the safe side of E0501 rather than being gated.
It carries its own **well-formedness rule, checker error E0510**: `p` must have type `rawptr StructT`
for a compiler-known struct and `f` must be a statically known field of `StructT`, so the op cannot
name a non-field or apply to a non-struct pointee — the restriction to a static field of the pointee
is what keeps it address-arithmetic-with-a-proof, not free pointer arithmetic. E0510 is a
well-formedness diagnostic, not an unsafety gate; the §1 "What is NOT claimed" safe-carve-out list
gains `field_ptr` as one more safe entry, not one more proof obligation. `unsafe`
is a block with a mandatory non-empty justification (**E0502**, closing verification #1 C1). The
boundary is syntactically narrow: `set_in_unsafe` toggles a flag only across the block body, and the
block grants *only* raw-pointer power — move, borrow, overflow, and bounds checking still run inside
`unsafe` (they are not raw-pointer operations). The obligations that transfer to the author are
exactly: validity/initialization of every dereferenced raw pointer; not creating two owners of a
moved value via `ptr_read`; and the liveness of `ctx`/`vt` behind every `Alloc` handle and `Box`
(§6.1, stated in `pool_handle`'s justification in §11.1). The checker enforces the *boundary* (where
unsafety may appear and that it is greppable and justified), not the *truth* of these obligations —
that is what P1 buys and what claims (a)–(e) explicitly exclude.

### 2.6 Fault delivery (discharging (e)) — review #1 finding 5

Claim (e) is that every *defined fault condition* halts execution at the faulting operation rather than
producing a value derived from it. It is discharged by enumerating the fault sites and showing the checker
never suppresses one.

- **The fault sites are closed and enumerated** (`interp/mod.rs::FaultKind`, raised in `interp/eval.rs`):
  the enum has **nine** variants and every one is named here. Eight are in scope of claim (e): arithmetic
  **`Overflow`** (incl. **negation** of `INT_MIN`) and integer **`DivByZero`** (division/remainder); array/slice
  **`Bounds`**; **`ConvLoss`** (lossy `conv` narrowing); and a failed **`Assert`**, **`Requires`**, or **`Ensures`**,
  and an explicit **`Panic`**. Each is raised at the operation and propagated as `Ctl::Fault`, unwinding to the
  top without yielding the operation's would-be result — the interpreter has no path that both raises a fault
  and returns a derived value.
- **The ninth variant, `BadPointer`, is outside claim (e)'s scope — named and placed, not omitted.** It is
  raised only at **raw-pointer** operations (an address beyond the memory model on `ptr_read`/`ptr_write`/
  `ptr_offset`), plus an **init-byte guard** diagnostic aid that traps a read of never-written storage. Both are
  **unsafe-only** sites: claims (a)–(e) hold for the safe fragment and explicitly exclude raw-pointer meaning
  (§2.5), so `BadPointer` is not a *defined fault condition* of the safe language that (e) ranges over — it is a
  best-effort trap on unsafe operations and a debugging guard, deliberately outside (e). Naming it completes the
  enumeration (closed by construction, not by assertion; review #2 finding 3).
- **The checker never *skips* a fault.** There is exactly one construct that changes fault behavior, and it
  **redefines** the operation rather than skipping the check: the lexically-scoped arithmetic **regimes**
  `wrapping { .. }` / `saturating { .. }` (`ExprKind::Wrapping`/`Saturating`; `interp/eval.rs::regime_block`).
  Inside them, overflow and conv-loss are **total, defined results** (two's-complement wrap, or clamp to the
  type bound) — not an un-checked operation with UB, but a *different specified semantics* chosen by the
  author in a greppable block. Every *other* fault (div-by-zero, bounds, assert/requires/ensures, panic) still
  fires inside a regime block; only overflow/conv-loss are redefined, and only lexically. This is redefinition,
  not suppression.
- **`unsafe` is excluded, by construction and by the claim.** Raw-pointer operations are unchecked (§2.5);
  claims (a)–(e) hold for the safe fragment only. A regime block grants no such power — move, borrow, bounds,
  div-by-zero, and the non-redefined faults all still run inside it.
- **The two fall-off-with-no-value paths are closed.** A fault-relevant hole would be a program that reaches a
  point *needing a value* without producing one: a non-exhaustive `match` (falls through with no arm) and a
  non-unit function that runs off its end. Both are static errors — **E0601** exhaustiveness
  (`check/patterns.rs`) and **E0810** all-paths-return (`check/loans.rs`) — so no accepted program reaches a
  use of a value that a skipped fault or a missing arm would have had to produce. This is why (a)/(e) lean on
  exhaustiveness and all-paths-return (cross-referenced from §2.3).

The discharge is bounded by the same model caveat as the rest (§1): faults are defined against the prototype
interpreter's trapping semantics, which has no optimizer and does not exercise P5's fault window.

### 2.7 The contract boundary (serving (a), (c)) — review #3

A contract clause is a place where source expressions are *evaluated at runtime* (P8: enforced level). Two
seams therefore have to be closed for claims (a) (no use-after-free) and (c) (no use-after-move) to hold for
those expressions as much as for the body.

- **Read-only rule (`check/*`, E0708).** A `requires`/`ensures`/`assert` condition may not move a non-`copy`
  value, take a `write`-borrow, pass an `out` argument, or call a function that takes any argument by `take`
  (non-copy), `write`, or `out`. This is enforced at the same classification points the body uses:
  `emit_place_action` flags a non-copy `Move`, an exclusive `Borrow`, or an `OutArg`; the call sites
  (`check_user_call`, the indirect-call arm, `check_out_arg`) flag a consuming callee and an `out` place.
  Reads, `read`-borrows, and copy-`take` calls pass. The rationale is P8: a check that consumes or mutates
  the state it inspects is incoherent, and (relevant here) a clause that *moved* a value would leave the
  interpreter's post-body drop schedule inconsistent with the checker's. Restricting clauses to reads makes
  their meaning stable across a future non-interpreting implementation.
- **Return-point dataflow (`check_fn`, `init.rs`, E0301).** Instead of checking `ensures` off the CFG, the
  checker re-emits each clause's accesses into **every normal-return block** (`Term::Return`/`FallThrough`),
  where they are analyzed by the same definite-assignment+move pass (§2.1) against the move/init state the
  body leaves at that return. A clause that reads a param the body moved, or dereferences a `Box` the body
  `unbox`-ed, is then the ordinary **E0301** — carrying a note that the access is in a contract. The analysis
  is done **once per return block against that block's own out-state**, not against a meet of the return
  states. That is the precise reading of the runtime semantics (the clause runs at each return, over exactly
  that return's state) and it is sound: a place moved on the path to a given return is `Moved` in that
  block's state, so its clause read is rejected; a place still live there is `Init` and accepted. A meet would
  be sound only paired with an initialized-on-all-return-paths demand, which would additionally reject clauses
  that are well-defined at every actual return — strictly more conservative for no soundness gain. Because the
  read-only rule forbids clause moves, an *accepted* clause emits only reads and `read`-borrows, so the
  re-emission never perturbs the body's own move/init fixpoint (reads do not change state). The interpreter is
  unchanged: it still evaluates each clause at the return before drops — but an accepted program can no longer
  make that evaluation touch moved or freed state. This restores (a)/(c) for contract expressions and turns
  review #2's demanded disclosure of the gap into a discharge. Regression and control tests
  (moved-param read, `unbox`-then-deref, live-param positive, the three read-only-rule rejections, and the
  static-immutability set) live in `tests/check.rs`/`tests/run.rs`.

---

## 3. Known conservatisms (reject-valid; harmless to soundness; relevant to measurement bias)

Each rejects some sound programs. All are safe over-approximations, but each can push a basket author
toward an extra binding, a clone, or a valve, so each is a potential source of *measurement* bias to
record for Bet 5.

1. **No two-phase borrows** (§2.3, E0805). `push(write v, read v[0])`-shaped nested calls are rejected.
2. **Index-covers-array** (`canonical` truncates at the first index). `a[0]` and `a[1]` are treated as
   overlapping; disjoint-index mutation is rejected.
3. **One-level / parent-anchored reborrow provenance.** A reborrow loan restricts its immediate parent
   binding (canonical root), not the ultimate origin; deep aliasing is approximated transitively rather
   than tracked with fine provenance.
4. **Call-site reborrow of a held borrow is implicit** (§2.1, as amended by design 0005;
   `docs/reviews/2026-07-08-design-0004-0005-review-1.md`). A bare held borrow passed to a
   `read`/`write`-mode parameter **reborrows** (it does not move); the checker inserts the reborrow
   node by the desugaring rule of §2.2. This *retires* the former conservatism (explicit
   `write (deref b)`/`read (deref b)` required at every call site, which pushed authors toward
   ceremony) rather than adding one — it is kept in this list because the earlier hashed text asserted
   the opposite ("no implicit call-site reborrow"), and the change is what fired this document's
   re-review tripwire (§0). Fresh borrows of owned storage still wear the keyword (`f(write x)`).
5. **Opaque places required-init at the root; no non-copy move through indirection.** A place through
   `deref`/index is required initialized at its whole *root* (finer partial init states through indirection
   are not tracked). A *non-copy* move out of such a place is not a conservatism but a rejected shape
   (**E0310**, §2.1/§2.4; ruling of review #2): only `copy` reads pass through a `deref`/index, and the
   checker marks the whole root moved to agree with the interpreter. (Before review #2 this entry claimed
   `init.rs` *tracked* the opaque move by marking the root moved — it did not; only the interpreter's
   `mark_place_moved` did, and the checker silently accepted the move. That divergence was the accept-invalid
   hole; the entry is corrected here.)
6. **Compact-default provenance only for the sole borrow parameter.** A borrow return from two-plus
   borrow params with no region variable is rejected (E0807) rather than inferred.
7. **`copy` is opt-in and structural.** A cheap all-scalar struct without the `copy` marker moves;
   this is by design (§1.3) but is a reject-valid-style friction worth noting for annotation counts.

---

## 4. Known gaps and threats to validity

Stated specifically, not reassuringly.

- **The "no runtime drop flags" claim is now discharged at every drop point (§0, review #1 findings 1–2).**
  What was the thinnest part of the argument is resolved in the strict direction: E0309 rejects a needs-drop
  place that is `MaybeInit` at *any* of its drop points — scope exit **and** reassignment/`out` — so the drop
  schedule for needs-drop values is static in both the move and the initialization dimension, and the
  interpreter's `MoveMask` consultation is a mechanism for exempt types and a debug assertion for needs-drop
  ones, never a semantic drop-flag decision. The residual is the general informality of the argument, not a
  known unsound drop path.
- **Soundness is asserted against the interpreter, which is itself unverified.** There is no
  independent model; `interp/` *is* the semantics. A bug in the interpreter (e.g. in `MoveMask`
  bookkeeping, layout, or the init-byte guard) could make an "accepted, safe" program misbehave without
  the checker being at fault. The init-byte guard is explicitly a *diagnostic aid*, not language
  semantics (`interp/mod.rs`), so it is not a backstop the checker's soundness may lean on.
- **Liveness precision at the accessing point.** The scan uses `live_after(b,i)`; the exact treatment
  of an access that coincides with a borrow's last use or its creation point is subtle. The test suite
  exercises the boundary cases (E0801–E0805) and they pass, but this is argued by testing, not by a
  proof that `live_after` is the correct point in every CFG shape.
- **`clone` effect rests on `bears_box` reachability.** `bears_box` walks named types with a cycle
  guard; a box reachable only through a type shape the walk mis-handles would under-mark the effect.
  Reviewed as correct for the basket's shapes, not proven for all.
- **Regions are checked by expression-shape provenance** (`borrow_provenance`), a syntactic walk — but the
  walk is now **total** (review #1 finding 1, §2.2). It recurses into every `match` arm and `if` branch, and
  an unrecognized borrow-producing shape is **rejected** (`None` ⇒ E0806), not skipped. The earlier draft
  rated the skip a "completeness caveat"; the review correctly re-rated it an accept-invalid memory-unsafety
  (a returned borrow of a local escaping through a `match`), and it is fixed: an unrecognized shape can now
  only cause a *reject-valid* (a sound borrow the syntactic walk fails to recognize), never an accept-invalid.
  That residual conservatism is the honest caveat here.
- **The argument is informal.** Per P18 and §1, this is not mechanized; the residual risk is exactly
  the gap between a reviewed informal argument and a proof.

---

## 5. The §11 standing verification obligation

Design 0001 §11 makes every worked example a fixture and requires re-verification before freeze step
(i). The five basket fixtures pass the full checker and run under the interpreter:

- `tests/check_fixtures.rs` — all five (`11_1_allocator`, `11_2_scheduler`, `11_3_mmio`,
  `11_4_parser`, `11_5_arena`) check with **zero diagnostics**.
- `tests/run_golden.rs` — all five execute to completion under the interpreter.

Standing invariant: the full suite (`cargo test`) is **green — every test passes, zero failures** — and is
kept so at each freeze step; the live count is whatever CI and the repo report, not a number pinned in this
hashed document (which would only rot). The suite spans the unit tests, the `check` negative/positive
snippets (including the review #1 regression sets — total return-provenance, reassignment/`out` drop
points, nested partial move, the box-drop/`unbox` free side — the review #2 opaque-move E0310 set, and the
review #3 contract-boundary and static-immutability sets, each with positive controls), the five
`check_fixtures` and five `run_golden` basket fixtures, and the `count`, `golden`, `loans`, and `run`
suites. This discharges the §11 obligation for the accept/reject boundary and at runtime; it does not, and
cannot, substitute for the argument above — passing fixtures show the checker accepts what it should, not
that it rejects everything unsound. (The `11_4_parser` runnable fixture gained an `alloc` marker on
`eval_owned`, which `unbox`es the owned AST — the free side of finding 4.)

---

## Consequences and costs

- The admissibility precondition (§3.7) is met **only if** the independent reviewer accepts this
  argument *and* dispositions the §0 finding. Until then, counts remain inadmissible by the criterion's
  own rule — which is the correct, conservative default.
- The §0 finding is a debt, not an absolution: if the resolution is "correct design 0001 §1.5," then
  design 0001's own §10.6 rejection of "runtime drop flags" must be re-read as applying to moves only,
  and that asymmetry should be recorded where a future reader starts.
- This document is written to be hashed at freeze step (i). Any later change to the checker's
  soundness-relevant behavior invalidates the hash and requires re-review — the intended tripwire.
