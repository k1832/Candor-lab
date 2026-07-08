# DRAFT — Fault-Window Formalization (OBL-WINDOW, NN#20)

**Status: SPEC-TIER DRAFT — NOT NORMATIVE.** A rigorous-informal formalization of
the imprecise fault window of chapter 06 §7, discharging the single-threaded core
of OBL-WINDOW (chapter 99). Mechanization is preferred by P18 and is out of scope
for this draft (§13). This document is the *arbiter's argument*, written to be
attacked: every load-bearing step is stated so a reviewer can contest it.

**Hooks:** P5 (the invariant and its bound); NN#20 (named-novel, mandatory
pre-stability formalization); NN#1 (no UB in safe code); NN#5 (no uninitialized
reads); Bet 3 (imprecision restores scheduling, collapses where effects are
dense); P4 (diagnostics as first-class); chapter 06 §3 (precise trap as sound
refinement); chapter 09 (adopted consistency model, currently no-concurrency).

---

## 1. Scope, and what is vacuous today

1.1 This edition ships **no concurrency construct, no atomics surface, and no
    synchronization operation** (chapter 09 §2.1; P10 first-version posture). The
    §7.2 bound names *two* delimiters — "the next **synchronization operation** or
    **externally visible effect**." The **synchronization half is therefore
    VACUOUS today**: there are no synchronization operations for a fault to be
    ordered before. This draft formalizes the **single-threaded core** honestly,
    and structures the definitions (§3, §12) so the synchronization half becomes
    *non-vacuous by extension*, not by redesign, when atomics land.

1.2 Consequently the acceptance clause of OBL-WINDOW that reads "composes soundly
    with the adopted consistency model (chapter 09)" is, today, a composition with
    a consistency model that *specifies no cross-thread ordering*. This draft
    discharges that composition **in its current (degenerate) content** and marks
    the future non-degenerate content as CONJECTURE (§12, §14).

1.3 The claim of this draft is about **semantics** — the set of observable
    behaviors the language *permits*. It is **not** a claim that any compiler
    emits only permitted behaviors; that is the separate compiler-correctness
    obligation (§13.2), out of scope here.

---

## 2. The observable substrate (enumerated honestly)

2.1 An **observable event** (equivalently *externally visible effect*) is an
    action whose result escapes the abstract machine's private, race-free store —
    something an agent outside the single Candor thread can witness. In the
    **current** edition the observable events are, exhaustively:

  - **`mmio_w(a, v)` / `mmio_r(a) ⇒ v`** — a volatile store/load through a raw
    pointer formed by `addr_to_ptr[T]` to a fixed address `a`, performed inside
    `unsafe` (chapter 05 §2.4). This is the sole *program-authored* externally
    visible effect this edition has.
  - **`fault(k, s, c)`** — delivery of the structured, machine-readable fault
    report (kind `k`, source span `s`, value context `c`) at a fault (chapter 06
    §6.2). Delivery is itself observable *and* is the truncation point.
  - **`halt`** — program termination under the root-declared fault policy
    (abort / halt-and-log / handler entry; chapter 06 §6.1), or normal return to
    the environment.

2.2 The following are **NOT** observable events, and this is load-bearing:

  - Ordinary loads/stores to owned or borrowed **in-model** storage. Under
    single-thread execution with ownership-guaranteed DRF (chapter 09 §1.2) and no
    aliasing outside the valve, no external agent can witness them. They are
    internal (`τ`) steps.
  - A `drop` (chapter 03 §6) is observable **only** to the extent its hook body
    performs an observable event (an `mmio_w`); the schedule itself is internal.

2.3 **Empty today, reserved for extension:** `ffi_call(...)` (FFI does not exist,
    chapter 08 §6) and `sync(o)` (no synchronization operation exists, §1.1). Both
    are externally visible when they arrive; the definitions below quantify over
    the observable-event set `Obs` so that adding them widens `Obs` without
    changing the theorems' statements.

---

## 3. The formal core (small-step, single thread)

3.1 **Configurations.** A configuration is `C = ⟨k, σ, θ⟩` where `k` is the
    control continuation (the remaining computation, ordered by program order §4),
    `σ` is the store (a partial map from places to values carrying a static
    initialization tag per chapter 03 §7.6), and `θ ∈ Obs*` is the finite trace of
    observable events emitted so far. `Obs = { mmio_w, mmio_r, fault, halt }`
    today (§2), extended monotonically later.

3.2 **Transitions.** The reference (precise) semantics is a labelled relation
    `C —ℓ→ C'` with `ℓ ∈ Obs ∪ {τ}`. A `τ`-step performs internal computation or
    an in-model memory access and appends nothing to `θ`. An `ℓ ∈ Obs` step
    appends `ℓ` to `θ`. The **observable trace** of a run is `tr = θ` at
    termination (the `Obs`-subsequence of the label stream).

3.3 **Faulting steps.** An operation `op` on input state is *enabled-faulting*
    when it meets a defined fault condition of chapter 06 §2 (Overflow, DivByZero,
    Bounds, ConvLoss, Assert/Requires/Ensures, Panic). Its reference transition is
    `⟨k, σ, θ⟩ —fault(k,s,c)→ ⟨halted, σ, θ·fault·halt⟩`: it **produces no value**
    (chapter 06 §6.3 — no value derived from the faulting operation becomes
    observable), writes no place, and truncates. Write `⊥` for its (absent)
    result.

3.4 **Program order and data dependence.**
    - **Program order** `≤po` is the total order on operation occurrences fixed by
      the source's *defined* evaluation order (chapter 06 §5.2). It is a property
      of the program text and input, not of a build mode.
    - `op₂` **data-depends** on `op₁` (`op₁ → op₂`) iff `op₂` reads a place `op₁`
      writes, transitively via def-use through `σ`. Dependence implies order:
      `op₁ → op₂ ⇒ op₁ <po op₂` (you cannot consume a value not yet produced).
    - An operation is **fault-independent of `f`** iff it is not reachable from `f`
      in `→` (it neither reads `f`'s result — which is `⊥` — nor is
      control-dependent on `f`'s completion).

3.5 **The precise reference run.** For a program `P` and input `i`, the reference
    relation is *deterministic* (defined evaluation order, no declared
    nondeterminism on the observable substrate; allocation-address nondeterminism
    of chapter 06 §5.3 does not reach `Obs`). Call its observable trace
    `tr★(P,i)`, and if it faults, call the po-earliest enabled-faulting operation
    `f★(P,i)` — the **precise fault**. `f★` equals the fault a zero-width-window
    (precise-trap) implementation delivers (chapter 06 §3).

---

## 4. The fault window

4.1 **Definition (window).** Fix a run of `P` on `i` in which `f` faults. Let
    `e⁻` be the `≤po`-greatest observable event with `e⁻ <po f` (or the start of
    the program if none), and `e⁺` the `≤po`-least observable event with
    `f <po e⁺` (or program end if none). The **fault window** is
    `W(f) = { op : e⁻ <po op <po e⁺ }` — the maximal po-neighbourhood of `f` that
    contains **no** observable event other than the fault itself.

4.2 **The bound (P5, §7.2).** `e⁺` is the **bound**: the fault is delivered no
    later than `e⁺`, and **nothing at or after `e⁺` in program order executes**.
    In particular `e⁺` itself does **not** retire — letting it retire would make an
    effect past the fault observable, contradicting truncation.

4.3 **Structural consequence.** Because `e⁺` is the *first* observable event after
    `f`, the window `W(f)` contains **no observable events**. Every operation in
    `W(f)` is a `τ`-step (internal computation or in-model memory access). Hence
    the "effects independent of it [that] may or may not have retired" of P5 §7.1
    are exactly the **non-observable** operations of `W(f)`: their retirement is
    invisible to `tr`. This is why the window can "differ between build modes"
    (P5) while leaving the observable trace of a faulting run *fully determined*
    (§9) — the divergence lives entirely in non-observable internal state (and, at
    most, in the diagnostic value-context `c`, §6.4).

---

## 5. The reordering license, stated precisely

5.1 A **build mode** is any implementation whose observable behaviour is drawn
    from the **legal-execution relation** `⇒R` defined by closing the reference
    relation §3 under the following rewrites, and no others:

  - **(R1) Reorder / hoist / fuse independent effects.** Two operations that are
    pairwise data-independent (`¬(op₁ → op₂) ∧ ¬(op₂ → op₁)`) and both
    fault-independent of any fault they straddle MAY be executed in either order,
    coalesced, or eliminated if dead. **A data-dependent pair keeps its `≤po`
    order** (the optimizer respects `→`); this is the standard soundness side
    condition and it is what makes Containment (§7) hold.

  - **(R2) Vary window-interior retirement.** For a faulting run, any subset of
    the `τ`-operations of `W(f)` MAY have retired at delivery; the complement MAY
    be dropped by truncation. Because `W(f)` has no observable events (§4.3), R2
    changes only non-observable state.

  - **(R3) Detect a fault late.** The *detection point* of `f` MAY be moved to any
    point in `W(f)` (batched/vectorized checking, hoisted arithmetic), provided
    delivery still precedes `e⁺`. R3 is the freedom Bet 3 buys.

5.2 **What the license forbids.** No rewrite may (a) move a fault-*dependent*
    operation before its producing fault (R1 side condition), (b) let any
    observable event `≥po e⁺` retire (R2/§4.2), (c) suppress delivery when a fault
    is enabled (inescapability, §8), or (d) read a place not statically proven
    initialized (§9.2 — no rewrite touches the static init property).

---

## 6. Decision forced: which fault fires

6.1 **The question.** Let two or more operations in a single window `W` be
    enabled-faulting on input `i` (no observable event separates them, by §4.3).
    R3 lets a build detect any of them first. **Which fault is delivered** — which
    `(k, s, c)` appears in `θ`? The philosophy does not pin this (P5); the
    formalization must. Two rules:

6.2 **Option A — program-order-first.** The delivered fault SHALL be `f★` (§3.5):
    the `≤po`-earliest enabled-faulting operation. Equivalently, *every* legal
    execution delivers the same fault the precise-trap refinement (chapter 06 §3)
    delivers.
    - *Benefit.* The **entire** observable trace of a faulting run — prefix **and**
      fault identity — becomes deterministic across build modes (§9, strengthened
      form). Debug (precise) and release (optimized) report the *same* fault,
      which is exactly what the regenerate-and-test loop (P5 rationale) and the
      diagnostic loop (P4) require.
    - *Cost.* A fused/vectorized check that observes "some lane faulted" must
      additionally recover the po-earliest faulting element (an index-of-first
      reduction) or fall back to scalar replay of `W` to locate it. **Crucially
      this cost is paid only on the fault path** — the cold, about-to-truncate,
      bug path — never on the hot no-fault path, which needs only a cheap "any
      fault?" OR-reduction. And it forbids **no** transformation: scalar replay on
      the deterministic input always recovers `f★`.

6.3 **Option B — any-enabled.** The delivered fault MAY be any operation
    enabled-faulting within `W`.
    - *Benefit.* No cold-path first-index recovery; the build reports whichever
      fault is cheapest to detect. Marginal, because the hot path is identical to
      Option A (both need only "any fault?").
    - *Cost.* The fault identity `(k, s, c)` becomes **nondeterministic across
      build modes**: debug and release can report different faults for the same
      `(P, i)`. This is an observable divergence (the `fault` label is in `Obs`),
      it undercuts P5's rationale directly ("the tested artifact must behave like
      the shipped one"), and it degrades P4's loop.

6.4 **Recommendation: Option A (program-order-first).** The priority order settles
    it: source-declared semantics (Priority 2) and diagnostic determinism (P4)
    outrank the optimizer's scheduling freedom (Priority 4-adjacent), and — the
    decisive point — Option A's cost is **cold-path-only and forbids no
    optimization**, so it buys full observable determinism for nearly nothing. The
    residual under A: the *value-context* `c` (register/variable snapshot) MAY
    still vary with which independent `τ`-work retired (R2); this draft rules `c`
    a **diagnostic, best-effort** field, not part of the semantic trace equality
    of §9 — the *kind* `k` and *span* `s` are pinned, `c` is advisory. Reviewers
    should attack §6.4 first (see §14).

---

## 7. Theorem 1 — Containment

7.1 **Claim.** For every legal execution `E ∈ ⇒R` of `(P, i)` that delivers a
    fault `f`: (a) **no observable event data-dependent on `f` appears in
    `tr(E)`**, and (b) **no observable event `≥po e⁺` appears in `tr(E)`**. Hence
    `tr(E)` is a prefix of the fault-free program-order observable sequence,
    truncated strictly before `e⁺`, followed by `fault·halt`.

7.2 **Proof sketch.**
    - *(b)* By §4.2 nothing at or after `e⁺` executes; R2 (§5.1) may only vary
      *window-interior* `τ`-operations and may never let an observable event
      `≥po e⁺` retire (§5.2b). So no observable event `≥po e⁺` is appended to `θ`.
    - *(a)* Suppose an observable `g` with `f → g` (transitively) appears. By
      §3.4, dependence implies `f <po g`. `g` is observable, so `g` is not in
      `W(f)` (§4.3); being `>po f` and observable, `g ≥po e⁺`. By (b), `g` does
      not appear — contradiction. The single load-bearing step is R1's side
      condition (§5.1): the optimizer never reorders `g` before `f`, so
      dependence-implies-order survives every legal rewrite. **Attack here:** if a
      rewrite could break `f → g ⇒ f <po g` (e.g. speculative execution of `g`
      before `f` resolves), (a) fails; the claim rests on R1 forbidding exactly
      that.
    - *No value escapes.* `f` writes no place and yields `⊥` (§3.3, chapter 06
      §6.3); so even a *non-observable* consumer of `f` computes on no real value
      and, being `≥po e⁺` if observable, is truncated. ∎(sketch)

---

## 8. Theorem 2 — Prefix-determinism

8.1 **Claim (fault-free).** If the reference run `tr★(P,i)` is fault-free, then
    **every** legal `E ∈ ⇒R` produces the identical complete observable trace
    `tr(E) = tr★(P,i)`.

8.2 **Claim (faulting).** If the reference run faults at `f★` with bound `e⁺`,
    then every legal `E` that faults produces the identical observable trace:
    the observable events `≤po e⁻` (the deterministic prefix), followed by
    `fault(k★, s★, ·)·halt` under Option A (§6). Legal executions may differ
    **only** in non-observable window-interior state (R2) and in the advisory
    value-context `c` (§6.4).

8.3 **Proof sketch.**
    - *Fault-free.* With no fault, R2/R3 do not apply; only R1 (reorder
      *independent* effects) is available. Observable events that are pairwise
      independent commute, but on the single-thread observable substrate the only
      program-authored observable events are `mmio_w/mmio_r` to fixed addresses;
      two `mmio` accesses to *distinct* addresses are independent and their
      relative order is not externally distinguishable at those distinct
      addresses, while two accesses that could alias are data-dependent (§3.4) and
      R1 keeps their order. Hence every legal reorder yields the same `tr`. The
      **load-bearing assumption**: MMIO observability is *per-address*, so
      reordering independent (distinct-address) accesses is not observable. Attack
      here if a target's MMIO fabric imposes cross-address ordering (then those
      accesses are *not* independent and must be modelled as dependent — a target
      parameter, chapter 06 §7.3 window-collapse territory).
    - *Faulting.* By Theorem 1 the trace is `[obs ≤po e⁻] · fault · halt`. The
      prefix `[obs ≤po e⁻]` is a fault-free observable segment, hence determined
      by the fault-free argument above. The fault identity is `f★` by Option A
      (§6.2). The only freedom left is R2 over `W(f)`'s `τ`-operations, which by
      §4.3 append nothing to `θ`. Therefore `tr(E)` is identical across builds. ∎
      (sketch)

8.4 **Remark (the strengthening Option A buys).** Under Option B, 8.2 weakens to
    "identical up to `e⁻`, fault identity unconstrained." Option A upgrades it to
    "identical including fault identity." This is the precise sense in which the
    precise-trap prototype (chapter 06 §3) and any optimizing build produce the
    **same observable trace**, differing only in invisible internal state.

---

## 9. NN#1 and NN#5 preserved

9.1 **NN#1 (no UB in safe code).** `⇒R` assigns every configuration a defined,
    non-empty set of successor traces; truncation (`halt`) is a defined terminal;
    no rewrite yields an "undefined" successor. By Theorem 1 no value derived from
    `f` becomes observable. The only nondeterminism is (i) window-interior
    retirement (invisible) and, under Option A, none over fault identity. Safe
    code therefore never exhibits UB *through the window*. The window is a
    bounded, defined nondeterminism, not an undefined one. ∎(sketch)

9.2 **NN#5 (no uninitialized reads).** Initialization is a **static all-paths
    property** (chapter 03 §7.6): every place is provably initialized before every
    read on every control-flow path, independent of dynamic timing. Every legal
    rewrite (R1–R3) either reorders/drops independent operations or moves the fault
    detection point; none changes *which reads are init-dominated* (that is a
    static fact), and the set of reads any legal `E` executes is a subset of the
    reference run's po-reads up to truncation, each statically init-dominated.
    Hence no legal execution reads uninitialized storage. **The window is the sole
    license against strict ordering (chapter 06 §5.2), and this shows the license
    never reaches an uninitialized read.** ∎(sketch)

---

## 10. Collapse property (precise faulting as the degenerate limit)

10.1 **Claim.** As observable-event **density → 1** (every operation is, or is
     immediately followed in po by, an observable event — the MMIO/DMA/shared-
     memory-dense regime of P5 and Bet 3), `W(f) → ∅` (zero width) and `⇒R`
     collapses to the precise-trap semantics of chapter 06 §3.

10.2 **Proof sketch.** If every operation is immediately po-followed by an
     observable event, then for a fault at `f` the next observable `e⁺` is the
     operation immediately po-after `f`, so `e⁻ = ` the observable just before `f`
     and `W(f) = { op : e⁻ <po op <po e⁺ } = ∅` (no operation lies strictly
     between two adjacent observables around `f`). With `W(f)` empty: R2 has no
     `τ`-operations to vary, R3 has no slack to move the detection point, and R1
     is constrained by the dense dependence/ordering of adjacent observables.
     The only legal trace is `[obs ≤po e⁻] · fault · halt` with delivery *at* `f`
     — a **zero-width window**, i.e. the precise trap. This is exactly chapter 06
     §3's sound refinement (a strict subset of P5-legal behaviours), recovered as
     the `density → 1` limit. Where effects are dense, the optimizer's freedom
     (R1–R3) shrinks to nothing and the model degenerates to precise faulting —
     P5's and Bet 3's stated behaviour, here derived rather than asserted. ∎
     (sketch)

10.3 **Corollary (prototype coherence).** The prototype interpreter (no optimizer,
     precise trap; chapter 06 §3.2) is the `density → 1` degenerate case of this
     semantics *and*, under Option A, delivers the same fault identity `f★` as
     every optimizing build at any density. The prototype is thus a sound
     refinement at every density, not merely the dense one.

---

## 11. Summary of the single-threaded result

The single-threaded core of P5's invariant is established (rigorous-informal):
Thm 1 (Containment), Thm 2 (Prefix-determinism), §9 (NN#1/NN#5), §10 (collapse to
precise faulting). Discharge accounting is in §14.

---

## 12. Structuring the non-vacuous (concurrency) obligation

12.1 When atomics land (chapter 09 §4), `Obs` gains `sync(o)` and the §7.2 bound's
     *synchronization half* becomes live. The definitions above are written to
     extend, not be redone: `≤po` becomes **per-thread program order**; the window
     bound `e⁺` becomes the po-least element that is *either* an observable event
     *or* a **synchronization operation** in that thread.

12.2 **Intended rule (CONJECTURE).** A fault SHALL be delivered **before any
     subsequent synchronization operation of the faulting thread RETIRES** — i.e.
     before the fault-thread's next release/fence/seq-cst op takes effect — so that
     **no other thread observes post-window state through a synchronizes-with
     edge**. Truncation is thus enforced not only along per-thread po but along the
     inter-thread happens-before/`synchronizes-with` relation of the adopted
     C/C++20 model (chapter 09 §3.1, SC-for-DRF).

12.3 **Proof obligations this imposes (all currently OPEN):**
     - **(O1)** Define `W(f)` against `synchronizes-with`/happens-before, not only
       per-thread po; show `e⁺` is well-defined under all legal interleavings.
     - **(O2)** *Cross-thread containment*: no thread observes a value data-
       dependent on `f` via any `sw` edge (Theorem 1 lifted across threads).
     - **(O3)** *Truncation under interleaving*: no observable effect ordered
       happens-after the fault-thread's bounding sync retires.
     - **(O4)** *SC-for-DRF preservation*: fault-free executions still enjoy the
       adopted model's guarantees (Theorem 2 lifted); the window perturbs only
       faulting executions.
     - **(O5)** *Composition with OBL-ALIAS* (chapter 05 §6): the aliasing model's
       optimizer assumptions must not license a rewrite that violates O2/O3
       across the safe/unsafe boundary.

12.4 O1–O5 are the **named novelty** (P5, NN#20): imprecise-fault truncation
     composed with an adopted consistency model under an optimizing compiler.
     This draft does **not** discharge them; it fixes their statements so the
     future work transcribes a proof rather than inventing a target.

---

## 13. What remains unestablished (honesty section)

13.1 **Concurrency composition — CONJECTURE.** §12.2's delivery-before-sync-retires
     rule and obligations O1–O5 are unproven. They are the genuinely novel core
     and cannot be discharged before the atomics surface exists (chapter 09 §4.2).

13.2 **Compiler-correctness gap.** This formalizes the *semantics* — the legal-
     execution relation `⇒R` and the behaviours it permits. It does **not** prove
     any compiler emits only members of `⇒R`. A correctness (simulation/refinement)
     result — every compiled execution is a legal execution — is a separate,
     larger obligation not attempted here.

13.3 **Mechanization owed.** P18 strongly prefers a **mechanized** formalization;
     this is rigorous-informal. A mechanized `⇒R` (executable small-step relation
     and machine-checked Theorems 1–2, §9, §10) remains owed and is the natural
     next artifact. The proof sketches above are structured for mechanization
     (explicit relations, explicit side conditions).

13.4 **Modelling assumptions a reviewer inherits.** (i) MMIO observability is
     per-address (§8.3) — false on fabrics with cross-address ordering, which must
     then be modelled as dependence. (ii) Allocation-address nondeterminism
     (chapter 06 §5.3) does not reach `Obs` — true only while addresses are not
     leaked to an observable channel; leaking an address via `mmio_w` is an
     `unsafe`-authored observable and is the author's declared responsibility
     (chapter 05 §4). (iii) `drop` is observable only via hook-body effects (§2.2).

---

## 14. Acceptance-criterion check against OBL-WINDOW (chapter 99)

OBL-WINDOW acceptance: *"the fault model is formalized as mechanized (strongly
preferred) or at minimum rigorous-informal, proving the §7.2 window bound composes
soundly with the adopted consistency model (chapter 09), preserving NN#1 and
NN#5."*

**Discharged by this draft (rigorous-informal, single-threaded):**
- The §7.2 window bound is defined (§4) and the truncation/containment it names is
  proved-sketch (Theorem 1, §7).
- Prefix-determinism (§8) establishes the P5 invariant's fault-free and faulting
  clauses for the single-threaded core, with the recommended which-fault ruling
  (§6) strengthening it to full observable determinism.
- Sound composition with chapter 09 **in its current content**: chapter 09
  specifies no cross-thread ordering (§1.1), so "composition with the adopted
  model" today is the single-threaded degenerate case, which §7–§10 cover.
- NN#1 (§9.1) and NN#5 (§9.2) preserved.
- Collapse to the precise-trap refinement (§10) ties the semantics to chapter 06
  §3 and the prototype.

**NOT discharged (remaining OBL-WINDOW debt):**
- **Concurrency composition** — the non-vacuous synchronization half: O1–O5,
  §12.2's rule, all CONJECTURE (§13.1). This is the named-novel heart of NN#20 and
  is blocked on the atomics surface (chapter 09 §4).
- **Mechanization** — P18's preferred form (§13.3).
- **Compiler correctness** — out of scope by construction (§13.2).

**Verdict.** OBL-WINDOW is **partially discharged**: the rigorous-informal
single-threaded core is delivered; the concurrency composition and the mechanized
form remain open and are here given precise, attackable proof-obligation
statements (§12) rather than a hand-wave. The obligation's *gate* (no optimizing-
implementation soundness claim, no "1.0") stays **closed** until §13.1's
conjecture is discharged.

---

## 15. Where a reviewer should attack first

1. **§6.4 — the value-context ruling.** Making `c` advisory (not part of trace
   equality) is the softest normative move; if `c` must be deterministic, Option A
   costs more than claimed.
2. **§7.2 (a) / §5.1 R1 side condition.** Containment rests entirely on the
   optimizer never reordering a fault-*dependent* observable before its fault. A
   speculative-execution counterexample would break it.
3. **§8.3 — per-address MMIO observability.** The fault-free determinism proof
   assumes distinct-address MMIO accesses are independent; targets with ordered
   MMIO fabrics violate the premise.
4. **§12.2 — the delivery-before-sync-retires CONJECTURE.** The whole future
   novelty; contest whether it is even satisfiable under SC-for-DRF with an
   optimizing compiler.
