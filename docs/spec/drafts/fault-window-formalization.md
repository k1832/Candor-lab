# DRAFT — Fault-Window Formalization (OBL-WINDOW, NN#20)

**Status: SPEC-TIER DRAFT — NOT NORMATIVE.** A rigorous-informal formalization of
the imprecise fault window of chapter 06 §7, discharging the single-threaded core
of OBL-WINDOW (chapter 99). Mechanization is preferred by P18 and is out of scope
for this draft (§13). This document is the *arbiter's argument*, written to be
attacked: every load-bearing step is stated so a reviewer can contest it.

**Promotion note (2026-07-21).** The discharged single-threaded core here —
Containment (Theorem 1, §7), Prefix-determinism (Theorem 2, §8), NN#1/NN#5
preservation (§9), and collapse-to-precise (§10) — was **promoted into the
normative specification, chapter 06 §7.4**, on 2026-07-21 per ruling J1
(`docs/1.0-GATE-TRIAGE.md`): the single-threaded core discharges NN#20 for the
concurrency-free edition, mechanization **preferred, not required**. This document
is retained as the **proof artifact** chapter 06 §7 cites; O1–O5 (§12) and the
concurrency composition (§13.1) remain **CONJECTURE** for the atomics edition. The
§14 verdict below predates J1: the gate it records as "closed" is, per J1, the
**concurrency-composition** gate — the single-threaded gate is discharged.

**Hooks:** P5 (the invariant and its bound); NN#20 (named-novel, mandatory
pre-stability formalization); NN#1 (no UB in safe code); NN#5 (no uninitialized
reads); Bet 3 (imprecision restores scheduling, collapses where effects are
dense); P4 (diagnostics as first-class); chapter 06 §3 (precise trap as sound
refinement); chapter 09 (adopted consistency model, currently no-concurrency).

**Revision history.** *2026-07-08* — amended per adversarial review #1
(`docs/reviews/2026-07-08-fault-window-review-1.md`, all nine dispositions
accepted). The review found the draft UNSOUND as originally written: R1 licensed
reordering observables and fault-capable ops as if they were internal `τ`-steps,
breaking three theorems. Amendments applied here: R1 restricted to `τ`-steps with
observable events made effect-order-total (§5.1, §8.3); R1 window constraint added —
no fault-capable op reorders before its `e⁻` (§5.1, resolving the R1/R3
contradiction); control-dependence defined and folded into `→` (§3.4); dependence
restated as static def-use on `f`'s output place (§3.3–§3.4); O2 rescoped to
happens-before AND relaxed visibility with racy channels assigned to the unsafe
author (§12.3); store-granularity/torn-transaction note added (§8.3); the
`f★`-recovery/replay obligation made load-bearing (§6.5). The three review
counterexamples appear as worked non-examples N1–N3 (§7.3). Theorem 2 fault-free is
now the trivial consequence of effect-order-total observables, replacing the invalid
per-address aliasing proof.

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
    result. The place `op` would have written is its **output place** `out(op)` —
    a *static* location (chapter 03 §7.6), named by the program text independent of
    whether `op` dynamically faults. Dependence (§3.4) is defined over `out(op)`
    **statically**; **truncation is the dynamic fact** — that `op` actually faulted
    on this `(P, i)` — layered on top of the static def-use structure, not part of
    it.

3.4 **Program order, dependence, control-dependence.**
    - **Program order** `≤po` is the total order on operation occurrences fixed by
      the source's *defined* evaluation order (chapter 06 §5.2). It is a property
      of the program text and input, not of a build mode.
    - **Static data-dependence.** `op₂` **data-depends** on `op₁`
      (`op₁ →data op₂`) iff `op₂` names, among its input places, the **output
      place** `out(op₁)` that `op₁` writes — a *static* def-use relation over the
      program's places (chapter 03 §7.6), transitively closed. An observable's
      dependence on a faulting `f` is thus read off `out(f)` **statically**, before
      knowing whether `f` faults on `(P, i)`; whether it *does* fault is the
      separate dynamic fact of §3.3.
    - **Control-dependence.** Over the CFG of `P` (basic blocks, branch/switch
      terminators, a unique exit), operation `op₂` in block `Y` **control-depends**
      on branch `op₁` in block `X` (`op₁ →ctrl op₂`) iff (Ferrante–Ottenstein–
      Warren): `X` has ≥2 CFG successors, `Y` **post-dominates** some successor of
      `X`, and `Y` does **not** post-dominate `X`. Intuitively `op₁`'s outcome
      decides whether `op₂` executes. A branch that tests a value data-depends on
      that value; hence control flow gated on `f`'s result (which is `⊥`) is
      captured by `→ctrl` composed with `→data` and is **not** launderable through
      the CFG (non-example N3, §7.3).
    - **Combined dependence.** `→ = (→data ∪ →ctrl)⁺` (transitive closure).
      Dependence implies order: `op₁ → op₂ ⇒ op₁ <po op₂` (you cannot consume a
      value — or take a branch on a value — not yet produced).
    - An operation is **fault-independent of `f`** iff it is **not reachable from
      `f` in `→`**: it neither reads `out(f)` (whose dynamic content is `⊥`), nor is
      control-dependent on `f`'s completion, nor transitively depends on either.
      Speculating an observable across a control edge reachable from `f` in `→` is
      therefore **not** fault-independent, and R1 (§5.1) forbids hoisting it before
      `f`.

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

  - **(R1) Reorder / hoist / fuse / eliminate independent *internal* steps.** Two
    *internal* (`τ`) operations that are pairwise dependence-independent
    (`¬(op₁ → op₂) ∧ ¬(op₂ → op₁)` under the combined data-and-control dependence
    `→` of §3.4) and both fault-independent of any fault they straddle MAY be
    executed in either order, coalesced, or eliminated if dead. **This license
    applies to `τ`-steps only.** Observable events are **effect-order-total**
    (§8.3): they are **never reordered, coalesced, or eliminated** — every legal
    execution emits the events of `Obs` in full program order. Two side conditions
    bind R1: **(i)** a dependent pair keeps its `≤po` order — the optimizer respects
    `→`, *data and control* (this is what makes Containment §7 hold); and **(ii) the
    window constraint** — **no fault-capable operation may be reordered before its
    `e⁻`** (the po-greatest observable preceding it, §4.1). A potentially-faulting op
    stays within its own window `[e⁻, e⁺)`; it may not be hoisted past an earlier
    observable, so a fault can never suppress an observable that program-order-
    precedes it.

  - **(R2) Vary window-interior retirement.** For a faulting run, any subset of
    the `τ`-operations of `W(f)` MAY have retired at delivery; the complement MAY
    be dropped by truncation. Because `W(f)` has no observable events (§4.3), R2
    changes only non-observable state.

  - **(R3) Detect a fault late.** The *detection point* of `f` MAY be moved to any
    point in `W(f)` (batched/vectorized checking, hoisted arithmetic), provided
    delivery still precedes `e⁺`. R3 is the freedom Bet 3 buys.

  - **(R1/R3 boundary — resolved.)** R3 moves the *detection* point of a fault
    *within* its window `W(f)` (never earlier than `e⁻`, never at/after `e⁺`); R1's
    window constraint (ii) forbids moving the *fault-capable operation itself*
    before `e⁻`. The earlier draft let R1 reorder fault-capable ops freely, which
    could hoist a fault before an observable that program-order-precedes it and
    suppress that observable — a direct contradiction with R3's "delivery within the
    window." Constraint (ii) removes the contradiction: R1 and R3 now both keep
    every fault strictly inside `[e⁻, e⁺)`.

5.2 **What the license forbids.** No rewrite may (a) move any operation *reachable
    from a fault `f` in `→`* — data- or control-dependent — before `f`, nor move a
    fault-capable op before its `e⁻` (R1 side conditions i–ii); (b) reorder,
    coalesce, or eliminate any observable event, which are effect-order-total
    (R1/§8.3); (c) let any observable event `≥po e⁺` retire (R2/§4.2); (d) suppress
    delivery when a fault is enabled (inescapability, §8); or (e) read a place not
    statically proven initialized (§9.2 — no rewrite touches the static init
    property).

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

6.5 **The f★-recovery obligation (load-bearing).** Under Option A the recovery of
    the po-earliest fault is a **proof-load-bearing obligation**, not a mere cost:
    whenever a fused/vectorized/reordered build detects "some fault occurred in `W`"
    without knowing *which* op is `f★`, it MUST recover `f★` before delivery. The
    **replay origin** is the **last retired observable** — `e⁻` (§4.1): the build
    re-executes the deterministic segment `(e⁻, f★]` in program order to identify
    the `≤po`-earliest enabled-faulting op, then delivers `(k★, s★, ·)`. Replaying
    from `e⁻` (rather than from program start) is sound because everything `≤po e⁻`
    is an already-committed deterministic prefix (§8.2) and `W(f)` contains no
    observable events (§4.3), so replay re-derives no observable and re-emits
    nothing to `θ`. This obligation is what discharges "same fault identity across
    builds" (§8.2, §10.3); a build that cannot replay from `e⁻` cannot claim Option
    A conformance.

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
    - *(a)* Suppose an observable `g` reachable from `f` in `→` (data- **or**
      control-dependent, §3.4) appears. By §3.4 dependence implies `f <po g`. `g` is
      observable, so `g ∉ W(f)` (§4.3); being `>po f` and observable, `g ≥po e⁺`. By
      (b), `g` does not appear — contradiction. Two load-bearing steps, both now
      discharged by amended R1 (§5.1): the optimizer never reorders a fault-reachable
      `g` before `f` (side condition i, extended to control-dependence), and
      observable events are **effect-order-total**, so `g` can be neither hoisted nor
      synthesized across a control edge (the laundering route). The
      speculative-execution and control-laundering breaks the earlier draft admitted
      are now blocked as worked non-examples N1 and N3 (§7.3).
    - *No value escapes.* `f` writes no place and yields `⊥` (§3.3, chapter 06
      §6.3); so even a *non-observable* consumer of `f` computes on no real value
      and, being `≥po e⁺` if observable, is truncated. ∎(sketch)

7.3 **Worked non-examples (the three review counterexamples, each blocked).** Each
    is a rewrite an earlier draft admitted; the amended rule that rejects it is named
    on one line.

    - **(N1) Suppressed observable — a fault hoisted before its `e⁻`.**
      `mmio_w(p, 1); x = a / b`, with `a / b` reordered before the store: the fault
      would truncate before `mmio_w` retires, deleting an observable that
      program-order-precedes it. **Blocked by R1 constraint (ii)** (§5.1): a
      fault-capable op may not be reordered before its `e⁻`, which here is the store.

    - **(N2) MMIO reorder / DCE / per-address aliasing.**
      Two distinct-address stores `mmio_w(p, 1); mmio_w(q, 2)` swapped, or a "dead"
      `mmio_w` eliminated, or aliasing `p ≡ q` mis-modelled as independent.
      **Blocked by effect-order-total R1** (§5.1, §8.3): observable events are never
      reordered, coalesced, or eliminated — they keep full program order
      unconditionally, so no per-address-independence premise is needed or used.

    - **(N3) Control-flow laundering of a fault-dependent observable.**
      `x = a / b; y = 0; if x > 0 { y = 1 }; mmio_w(p, y)` — `y` is only
      *data*-independent of `f = a / b`, so a data-only `→` would let `mmio_w(p, y)`
      hoist before `f`, leaking post-fault behaviour through the CFG. **Blocked by
      control-dependence folded into `→`** (§3.4): the chain
      `f →data (x > 0) →ctrl (y = 1) →data mmio_w(p, y)` makes the store reachable
      from `f` in `→`, hence fault-*dependent*, and R1 side condition (i) forbids
      hoisting it before `f`.

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
    - *Fault-free (now trivial).* Observable events are **effect-order-total**
      (§5.1 R1): no legal rewrite reorders, coalesces, or eliminates any member of
      `Obs`, and R1 touches `τ`-steps only. Therefore every legal `E ∈ ⇒R` emits
      exactly the reference program-order observable sequence — `tr(E) = tr★(P, i)`
      — **by construction**. This is the honest, now-trivial proof; it makes **no**
      premise about MMIO aliasing or per-address independence. The earlier draft's
      invalid step (reordering distinct-address accesses and eliminating "dead"
      observables under R1) is rejected as non-example N2 (§7.3). `τ`-reordering
      under R1 remains invisible to `tr` because `τ`-steps append nothing to `θ`
      (§3.2).
    - *Store-granularity observability.* An observable is emitted at **individual
      store granularity**: each `mmio_w(a, v)` is one indivisible observable event.
      The model provides **no multi-store atomicity** — a logical device transaction
      spanning several `mmio_w`s is a *sequence* of observables, and a fault may fall
      between them, truncating (§4.2) after some stores and before others. Such a
      **torn multi-store device transaction is P5-legal**, and is flagged for the
      eventual volatile-access / atomic-MMIO design (§13.4); this draft does not
      promise transactional device writes.
    - *Faulting.* By Theorem 1 the trace is `[obs ≤po e⁻] · fault · halt`. The
      prefix `[obs ≤po e⁻]` is a fault-free observable segment, hence determined by
      the effect-order-total argument above. The fault identity is `f★` by Option A
      (§6.2), recovered via the f★-recovery obligation (§6.5, replay from `e⁻`). The
      only freedom left is R2 over `W(f)`'s `τ`-operations, which by §4.3 append
      nothing to `θ`. Therefore `tr(E)` is identical across builds. ∎ (sketch)

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
     - **(O2)** *Cross-thread containment*: no thread observes a value dependent on
       `f` (data- **or** control-, §3.4) via any **happens-before** edge, **and**
       none via **relaxed-atomic visibility** (inter-thread ordering weaker than
       `sw`, e.g. `memory_order_relaxed` publication) — Theorem 1 lifted across
       threads over *both* the hb and the relaxed-visibility relations. **Racy or
       unsafe channels are out of scope of this guarantee**: a data race or a
       raw-pointer cross-thread share carries no ordering the model can bound, and
       post-window leakage through it is the **unsafe author's declared
       responsibility** per §13.4 (chapter 05 §4), not a containment defect.
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

13.4 **Modelling assumptions a reviewer inherits.** (i) Observability is at
     **individual-store granularity** with **no multi-store atomicity** (§8.3): a
     logical device transaction spanning several `mmio_w`s may be **torn** by a
     fault between stores (P5-legal), deferred to the eventual volatile-access /
     atomic-MMIO design. The earlier "per-address MMIO observability" assumption is
     **withdrawn** — the fault-free proof (§8.3) now rests on observable events being
     effect-order-total, not on cross-address independence, so ordered-MMIO fabrics
     need no special treatment. (ii) Allocation-address nondeterminism (chapter 06
     §5.3) does not reach `Obs` — true only while addresses are not leaked to an
     observable channel; leaking an address via `mmio_w` is an `unsafe`-authored
     observable and is the author's declared responsibility (chapter 05 §4). (iii)
     `drop` is observable only via hook-body effects (§2.2). (iv) **Racy / unsafe
     cross-thread channels** (data races, raw-pointer sharing) carry no
     model-bounded ordering; post-window leakage through them is assigned to the
     **unsafe author** (chapter 05 §4), and cross-thread containment (§12.3 O2) is
     claimed only over happens-before and relaxed-atomic visibility, not over racy
     channels.

---

## 14. Acceptance-criterion check against OBL-WINDOW (chapter 99)

OBL-WINDOW acceptance: *"the fault model is formalized as mechanized (strongly
preferred) or at minimum rigorous-informal, proving the §7.2 window bound composes
soundly with the adopted consistency model (chapter 09), preserving NN#1 and
NN#5."*

**Discharged by this draft (rigorous-informal, single-threaded):**
- The reordering license `⇒R` is now **sound** as amended (review #1, §revision-
  history): R1 is restricted to `τ`-steps, observable events are effect-order-total,
  and the window constraint keeps every fault inside `[e⁻, e⁺)` so a fault can never
  suppress an earlier observable. What is discharged is the **single-threaded core**,
  now resting on a **sound reordering license** rather than the originally-unsound
  one flagged by the review.
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
2. **§7.2 (a) / §5.1 R1 side conditions.** Containment rests on the optimizer never
   reordering a fault-reachable observable (data **or** control, §3.4) before its
   fault, and on observables being effect-order-total. The three review
   counterexamples (suppressed observable, MMIO reorder/DCE, control-flow
   laundering) are now blocked as worked non-examples N1–N3 (§7.3); attack whether
   any *fourth* laundering route survives the amended `→`.
3. **§8.3 — store-granularity / torn transactions.** The per-address-MMIO premise
   is withdrawn (§13.4 i); the fault-free proof is now effect-order-total and
   trivial. The residual soft spot is store-granularity: the model permits
   fault-torn multi-store device transactions (§8.3), deferred to the
   volatile-access design — contest whether P5 should instead promise transactional
   device writes.
4. **§12.2 — the delivery-before-sync-retires CONJECTURE.** The whole future
   novelty; contest whether it is even satisfiable under SC-for-DRF with an
   optimizing compiler.
