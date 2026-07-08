# 09 — Memory Consistency Model

**Status: ADOPTED-PENDING.** The consistency model is **external proven art to be
adopted from the C/C++/Rust axis** (P18: "adopt proven art where proven art
suffices — the consistency model comes from the C/C++/Rust axis, where a decade
of committee work exists and novelty is risk without reward"). This chapter fixes
the **structure** of the adoption and records what MUST be decided before any
concurrency feature lands. No normative ordering clause is transcribed yet because
this edition ships no concurrency construct.

---

## 1. Position

1.1 The memory consistency model — the ordering semantics of atomics, unsafe
    code, and boundary-module interactions — SHALL be **adopted from the
    C/C++20-family model**, not invented (P18). The named source is the ISO
    C/C++20 memory model and its Rust adaptation.

1.2 Data-race freedom is guaranteed by the **ownership model** at compile time
    (P10, non-negotiable); adopting a consistency model does **not** replace that,
    and ownership-based DRF does **not** eliminate the need for a consistency
    model underneath unsafe code and atomics (P18).

---

## 2. Why the adoption is deferred-but-structured (P10 first-version posture)

2.1 This edition ships **no concurrency constructs** (P10: no `async`/`await`, no
    coloring-shaped mechanism, in the first stable version). There is therefore
    **no atomics surface and no cross-thread ordering to specify yet**; the
    adoption is deferred.

2.2 It is **structured, not open**: the fragments to adopt are enumerated (§3) and
    the decisions that MUST precede any concurrency feature are recorded (§4), so
    that when concurrency lands the model is transcribed, not designed from
    scratch.

---

## 3. Fragments to adopt (structure)

3.1 **SC-for-DRF baseline.** Sequentially-consistent semantics for data-race-free
    programs SHALL be the baseline guarantee, per the C/C++/Rust axis.

3.2 **Atomics — TBD with the concurrency design.** The atomic orderings
    (relaxed / acquire / release / acq-rel / seq-cst) and their fences are
    **deferred** and SHALL be adopted together with the concurrency design, from
    the same source.

3.3 **Unsafe-code and boundary-module interactions.** The ordering semantics an
    unsafe author and a boundary module may rely on SHALL be specified as part of
    the adoption, composed with the unsafe-code aliasing model (chapter 05 §6) and
    the fault window (chapter 06 §7).

---

## 4. What MUST be decided before any concurrency feature lands

4.1 Which concurrency primitives ship (P10 requires structured — scoped, joined,
    cancellable — primitives, and forbids a bidirectional type-transforming
    partition; NN#9).

4.2 The atomics surface and its orderings (§3.2), adopted from the named source.

4.3 The **composition of the fault window (chapter 06 §7, NN#20) with the adopted
    ordering** — the named-novel obligation; a concurrency feature SHALL NOT land
    before that composition is discharged (chapter 06 §7.5).

4.4 The **unsafe-code aliasing model (chapter 05 §6)** under the adopted ordering.

4.5 Whether any partition-free ergonomic model for high-concurrency I/O is proven
    compatible with P9 before it is added (P10; by amendment only).

**Acceptance criterion.** Discharged when this chapter transcribes the adopted
C/C++20-family clauses (SC-for-DRF, atomics, fences) and §4's decisions are made,
composed soundly with chapters 05 §6 and 06 §7 (chapter 99, obligation OBL-CONSIST).

**Gate:** blocks any concurrency feature and any atomics surface; does not block
the single-threaded core of this edition.
