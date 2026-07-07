# 0003 — Checker Soundness Argument (Bet 5 Validation Prototype)

**Status:** draft
**Date:** 2026-07-07
**Philosophy hooks:** NN#1 (no UB in safe code), NN#5 (no reads of uninitialized memory),
NN#7 (no hidden allocation), P1 (unsafety explicit/local/auditable), P12 (values-first
memory model), P18 (semantics defined by a spec, not the compiler — served here in miniature).
**Subordinate to** `LANG_PHYLOSOPHY.md` and to design `0001-memory-model.md`; where either
outranks this document and conflicts, this document is the artifact that changes.

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
4. **The §0 auxiliary-claim hole** ("no runtime drop flags") is explicitly *not* discharged; see §0
   and §4.

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
- **Return provenance (E0806/E0807/E0808).** `check_return_provenance` rejects returning a borrow of a
  local or of an owned (`take`) parameter, and enforces that an explicitly region-tagged return
  derives from the region's parameter — a borrow may not outlive its body (§3.3), checked body-locally
  because no region crosses a signature (NN#17).

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

`check/effects.rs` tracks one boolean. `note_alloc` records the first allocation site — `box`
(`check/expr.rs` ~827), `clone` of a `bears_box` value (~60), a call to an `alloc`-marked function
(~731), and an indirect call through an `alloc`-typed fn-pointer (~689). `AllocEffect::finish` emits
**E0401** if a non-`alloc` function has any such site. Function-pointer types carry the effect in the
type (`types::FnPtrTy`), and `assignable`/`check_against` reject assigning an `alloc` function to a
non-`alloc` pointer (**E0402**) while permitting the reverse (upper-bound conservatism, §6.1). There is
no vtable special case: `AllocVtable` fields are `alloc`-typed, so every indirect call through them is
`alloc` by the one general rule. This makes a non-`alloc` context allocation-free: the only allocating
operations are the enumerated ones, each of which forces the marker.

### 2.5 The unsafe boundary (bounding what (a)–(e) cover)

`require_unsafe` (**E0501**) gates every operation that gives a raw pointer meaning
(`ptr_read`/`ptr_write`/`ptr_offset`/`addr_of`/`addr_of_mut`/`cast_ptr`/`addr_to_ptr`/`ptr_null`),
while holding/moving/copying/comparing a `rawptr` and `is_null`/`offsetof` stay safe (§4.2). `unsafe`
is a block with a mandatory non-empty justification (**E0502**, closing verification #1 C1). The
boundary is syntactically narrow: `set_in_unsafe` toggles a flag only across the block body, and the
block grants *only* raw-pointer power — move, borrow, overflow, and bounds checking still run inside
`unsafe` (they are not raw-pointer operations). The obligations that transfer to the author are
exactly: validity/initialization of every dereferenced raw pointer; not creating two owners of a
moved value via `ptr_read`; and the liveness of `ctx`/`vt` behind every `Alloc` handle and `Box`
(§6.1, stated in `pool_handle`'s justification in §11.1). The checker enforces the *boundary* (where
unsafety may appear and that it is greppable and justified), not the *truth* of these obligations —
that is what P1 buys and what claims (a)–(e) explicitly exclude.

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
4. **Explicit call-site reborrows required** (§2.1). Passing a held exclusive borrow bare is a move; a
   reborrow must be spelled `write (deref b)`/`read (deref b)`. No implicit call-site reborrow.
5. **Opaque places required-init at the root.** A place through `deref`/index is required initialized at
   its whole root, and a move through `deref`/index marks the whole root moved (`init.rs::apply`,
   `mark_place_moved`) — finer partial states through indirection are not tracked.
6. **Compact-default provenance only for the sole borrow parameter.** A borrow return from two-plus
   borrow params with no region variable is rejected (E0807) rather than inferred.
7. **`copy` is opt-in and structural.** A cheap all-scalar struct without the `copy` marker moves;
   this is by design (§1.3) but is a reject-valid-style friction worth noting for annotation counts.

---

## 4. Known gaps and threats to validity

Stated specifically, not reassuringly.

- **The "no runtime drop flags" claim is refuted for conditional initialization (§0).** This is the
  thinnest part of the whole argument. The safety claims (a)–(e) survive — the interpreter's
  `MoveMask`-guarded drop is correct — but design 0001 §1.5/§10.6's *mechanism* claim does not hold as
  written, and the discharge the task requested succeeds only for the move dimension. Adjudication
  (correct the doc, or add a checker rule rejecting owned-on-some-path-only locals at scope exit) is
  deferred to the independent review.
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
- **Regions are checked by expression-shape provenance** (`borrow_provenance`), a syntactic walk. A
  borrow laundered through a shape the walk does not recognize would return `None` (no provenance) and
  escape the E0806/E0808 check. No such shape is known in the basket; this is a completeness caveat.
- **The argument is informal.** Per P18 and §1, this is not mechanized; the residual risk is exactly
  the gap between a reviewed informal argument and a proof.

---

## 5. The §11 standing verification obligation

Design 0001 §11 makes every worked example a fixture and requires re-verification before freeze step
(i). The five basket fixtures pass the full checker and run under the interpreter:

- `tests/check_fixtures.rs` — all five (`11_1_allocator`, `11_2_scheduler`, `11_3_mmio`,
  `11_4_parser`, `11_5_arena`) check with **zero diagnostics**.
- `tests/run_golden.rs` — all five execute to completion under the interpreter.

Observed: the full suite (`cargo test`) is green — 33 unit + 35 `check` + 5 `check_fixtures` + 6
`golden` + 30 `loans` (including the four S1 negative tests and the copy-payload positive control) +
32 `run` + 5 `run_golden`, 0 failures. This discharges the §11 obligation for the accept/reject
boundary and at runtime; it does not, and cannot, substitute for the argument above — passing
fixtures show the checker accepts what it should, not that it rejects everything unsound.

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
