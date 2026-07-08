# 06 — Faults

**Status: NORMATIVE-DRAFT** (§§1–6, the prototype-validated fault model,
transcribed from design `0001-memory-model` §7–§8 and design `0003` §2.6)
**+ SKELETON** (§7, the imprecise fault **window** — P5's bound and the NN#20
formalization obligation). Rationale is in design 0001; the soundness claim
structure is in design 0003 §1.

---

## 1. The fault vocabulary (P7)

1.1 A **fault** is a bug manifesting: an arithmetic fault, a violated `enforced`
    contract, a failed assertion, or an explicit panic are **all faults** and
    route through one policy (§6). Faults are distinct from **errors**, which are
    ordinary values in return types (P7; sum types, chapter 03 §1.2). A fallible
    function returns its failure as a value; it does **not** fault for expected
    failure.

---

## 2. Defined fault conditions

2.1 The defined fault conditions of the safe language are, and are only:

- **Overflow** — `+ - * /` and negation overflowing the default arithmetic
  regime, including negation of the minimum signed value.
- **DivByZero** — division or remainder by zero.
- **Bounds** — an array/slice index or `subslice` range out of bounds
  (`0 <= i < len`; `lo <= hi <= len`).
- **ConvLoss** — a lossy `conv` narrowing under the default regime (chapter 03
  §9.3).
- **Assert / Requires / Ensures** — a failed `assert`, precondition, or
  postcondition at the `enforced` level (chapter 07).
- **Panic** — an explicit `panic`, unconditionally.

2.2 Indexing SHALL **always** be bounds-checked; there is no unchecked indexing
    outside a valve, and inside a valve indexing is via raw pointers, not slice
    indexing (NN#1: no side door).

2.3 A raw-pointer dereference inside `unsafe` is **not** a defined fault
    condition; a bad `ptr_read`/`ptr_write` is the author's declared
    responsibility (chapter 05 §4). An implementation MAY best-effort trap an
    out-of-model address, but such a trap is outside the safe-language fault set.

---

## 3. Precise trap as a sound refinement

3.1 Delivering a fault **immediately, at the faulting operation** (a zero-width
    fault window) is a **sound refinement** of the P5 imprecise semantics (§7): it
    lies within P5's bound and produces a strict subset of P5-legal observable
    behaviors.

3.2 A non-optimizing implementation (e.g. the prototype interpreter) MAY trap
    precisely. An optimizing implementation MAY widen the window up to P5's bound
    (§7); a program valid under precise trapping SHALL remain valid there.

---

## 4. Arithmetic regimes

4.1 Arithmetic regimes are **scoped and source-declared** (P5). The default
    regime is **checked** (overflow faults). `wrapping { ... }` and
    `saturating { ... }` are block-level regions that change the overflow and
    `conv`-loss behavior of the arithmetic **lexically inside them only**, to
    two's-complement wrap or clamp-to-bound respectively.

4.2 A regime block applies **textually only**. It SHALL NOT change the regime of
    any function it calls; a callee runs under its own regime. The block is a
    purely syntactic, greppable region with **no dynamic scope across calls**.

4.3 **Unchecked arithmetic** (overflow as undefined) SHALL NOT exist in safe
    code; it exists **only** inside an `unsafe` region (NN#1/NN#4). This edition's
    safe regimes are exactly checked, wrapping, and saturating.

---

## 5. No uninitialized reads; defined evaluation order

5.1 **No value SHALL ever be read while uninitialized** (NN#5). This is a
    compile-time guarantee (chapter 03 §7.6), not a runtime check: every place is
    provably initialized before every read on every path.

5.2 Evaluation order is **defined**. In a faulting execution, the fault window
    (§7) is the sole, bounded license against strict ordering.

5.3 Nondeterminism exists **only where explicitly declared** (allocation
    addresses, hash iteration order, and kin — P5).

---

## 6. Fault policy and delivery

6.1 Every program or embedded image declares, at its root, a single **fault
    policy** — abort, halt-and-log, or a user-supplied handler (P7). Unwinding is
    not required machinery.

6.2 On a fault, an implementation SHALL emit a **structured, machine-readable
    fault report** (kind, source span, value context) as well as human prose
    (P4).

6.3 A fault is a **truncated execution**: no value derived from the faulting
    operation SHALL ever become observable, and (under the prototype's abort
    policy) no drops run after a fault. The two "fall off with no value" paths —
    a non-exhaustive `match` and a non-unit function running off its end — SHALL
    be static errors, so no accepted program reaches a use of a value a skipped
    fault or missing arm would have had to produce (chapter 04 §9.5; all-paths-
    return).

---

## 7. The imprecise fault window — SKELETON (P5, NN#20)

**Status: SKELETON.** This section records P5's invariant and the mandatory
pre-stability formalization obligation NN#20. The prototype does not exercise the
window (§3; no optimizer), so it is the cheapest point to fix the position.

7.1 **The invariant (P5, normative position).** For a given compilation target:
    fault-free executions are **deterministic across build modes** (same source +
    target + inputs ⇒ identical observable behavior). A faulting execution is a
    **truncated** execution: the fault is **imprecise but inescapable** — no value
    derived from it becomes observable, delivery is guaranteed, and observable
    behavior is identical across build modes **up to a fault window** around the
    faulting operation, within which effects independent of it may or may not have
    retired; that window MAY differ between build modes.

7.2 **The window's bound (normative position).** A fault SHALL be delivered **no
    later than the next synchronization operation or externally visible effect**
    that follows the faulting operation in program order; nothing past that point
    executes.

7.3 **Window collapse (normative position).** Where externally visible effects
    are dense (MMIO, DMA, shared-memory writes), the window collapses tightly and
    the model degenerates toward precise faulting (Bet 3); those paths are where
    scoped regimes (§4) and proven contracts (chapter 07) carry the performance
    load.

7.4 **The named-novel obligation (NN#20).** The composition of imprecise-fault
    truncation with the adopted memory consistency model (chapter 09) under an
    optimizing compiler is **novel semantics** (P5 says so; it is not sheltered
    under P18's "adopt proven art"). Formalizing it is **mandatory pre-stability
    work**, in the same validation tier as Bet 5's artifact.

7.5 **Acceptance criterion (NN#20).** The obligation is discharged when the fault
    model is formalized as **mechanized** (strongly preferred) or **at minimum a
    rigorous-informal** composition with the adopted consistency model of chapter
    09 — precisely: a stated proof (mechanized or rigorous-informal) that the
    window bound of §7.2 composes soundly with chapter 09's ordering, preserving
    NN#1 and NN#5 (chapter 99, obligation OBL-WINDOW).

**Gate:** blocks any optimizing implementation's soundness claim and any
stability commitment (NN#20 is named a hard pre-stability gate).
