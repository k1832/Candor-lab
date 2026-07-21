# 06 — Faults

**Status: NORMATIVE-DRAFT** (§§1–8 — the prototype-validated fault model
(§§1–6), the single-threaded fault **window** (§7, discharged per ruling J1),
and observable effects (§8)) **+ SKELETON** (§7.5, the fault window's
**concurrency composition** — the NN#20 named-novel obligation, deferred with
atomics). Rationale is in design 0001; the soundness claim structure is in design
0003 §1; the fault-window proof artifact is
`docs/spec/drafts/fault-window-formalization.md`.

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

## 7. The imprecise fault window (P5, NN#20)

**Status: NORMATIVE-DRAFT (single-threaded window) + SKELETON (concurrency
composition).** The single-threaded fault-window core is formalized and
adversarially reviewed in `docs/spec/drafts/fault-window-formalization.md` — the
**proof artifact** this section cites. Its discharged results (Containment,
Prefix-determinism, NN#1/NN#5 preservation, collapse-to-precise) are stated
normatively in §7.4 and discharge OBL-WINDOW for the concurrency-free edition
(ruling J1, `docs/1.0-GATE-TRIAGE.md`; chapter 99). The **concurrency
composition** — the synchronization half of the §7.2 bound — is vacuous this
edition (no atomics ship, chapter 09) and remains SKELETON, deferred with the
atomics surface (§7.5). The prototype does not exercise the window (§3; no
optimizer).

7.1 **The invariant (P5).** For a given compilation target: fault-free executions
    are **deterministic across build modes** (same source + target + inputs ⇒
    identical observable behavior). A faulting execution is a **truncated**
    execution: the fault is **imprecise but inescapable** — no value derived from
    it becomes observable, delivery is guaranteed, and observable behavior is
    identical across build modes **up to a fault window** around the faulting
    operation, within which effects independent of it may or may not have retired;
    that window MAY differ between build modes.

7.2 **The window's bound.** A fault SHALL be delivered **no later than the next
    synchronization operation or externally visible effect** (an **observable
    effect**, §8) that follows the faulting operation in program order; nothing
    past that point executes. This edition ships no synchronization operation
    (chapter 09), so today the bound is the next **observable effect** (§8); the
    synchronization delimiter is reserved for the atomics edition (§7.5).

7.3 **Window collapse.** Where externally visible effects are dense (MMIO, DMA,
    shared-memory writes), the window collapses tightly and the model degenerates
    toward precise faulting (Bet 3); those paths are where scoped regimes (§4) and
    proven contracts (chapter 07) carry the performance load. At the dense limit
    the window has zero width and the semantics coincides with the precise trap of
    §3, of which precise trapping is the sound refinement (proof artifact §10).

7.4 **The discharged single-threaded core (NORMATIVE).** For every legal
    execution of a program on an input under this edition's single-threaded
    semantics (no atomics, no synchronization operation; §7.5), the fault window
    satisfies the following. Each is proved rigorous-informal in the proof
    artifact and is normative here:

    - **(a) Containment** (proof artifact Theorem 1). No observable effect (§8)
      that data- or control-depends on the faulting operation appears in the
      trace, and no observable effect at or after the window bound (§7.2) appears.
      The observable trace of a faulting run is therefore a program-order prefix
      of the fault-free observable sequence, truncated strictly before the bound,
      then delivery of the fault report (§6.2) under the declared fault policy
      (§6.1).
    - **(b) Prefix-determinism** (proof artifact Theorem 2). A fault-free
      execution produces the **identical complete** observable trace across all
      build modes. A faulting execution produces the **identical** observable
      trace across build modes: the deterministic observable prefix up to the last
      observable effect before the fault, then delivery of the fault report under
      the declared fault policy. The delivered fault is the
      **program-order-earliest** enabled fault — the same fault the precise trap
      (§3) delivers — so its **kind and source span are identical across build
      modes**. The fault report's **value context** (§6.2) is **advisory** and MAY
      vary with which window-interior, non-observable work retired.
    - **(c) NN#1 / NN#5 preserved** (proof artifact §9). The window is a bounded,
      **defined** nondeterminism, never undefined behavior (NN#1): no legal
      window-interior reordering makes a value derived from the fault observable,
      and none reaches an uninitialized read — initialization is the static
      all-paths property of §5.1 (NN#5), which the window's reordering license
      cannot perturb.

7.5 **Scope, the named-novel obligation, and the deferred concurrency composition
    (NN#20).** §§7.1–§7.4 are normative for **single-threaded** executions. The
    composition of imprecise-fault truncation with cross-thread ordering — the
    **synchronization half** of the §7.2 bound — is the **named-novel** part of
    NN#20 (novel semantics under an optimizing compiler; P5 says so, not sheltered
    by P18's "adopt proven art"). This edition ships no atomics or synchronization
    operation (chapter 09), so that half is **vacuous today**; its proof
    obligations (O1–O5, proof artifact §12) are **CONJECTURE**, deferred as a
    bundle with the atomics surface (chapter 09 §4). A concurrency feature SHALL
    NOT land before that composition is discharged.

    **Acceptance criterion (OBL-WINDOW, chapter 99).** The obligation is
    discharged when the fault model is formalized — **mechanized** (strongly
    preferred) or **at minimum rigorous-informal** — proving the §7.2 bound
    composes soundly with the adopted consistency model of chapter 09, preserving
    NN#1 and NN#5. Per ruling J1 the **single-threaded core is discharged**
    (rigorous-informal; the proof artifact), with mechanization **preferred, not
    required**; the concurrency composition above remains open.

**Gate:** NN#20's single-threaded core is discharged (J1), so §§7.1–§7.4 no
longer block an optimizing implementation of this concurrency-free edition on
fault-window grounds. The **concurrency-composition** gate — blocking any atomics
surface and the atomics edition's optimizing-implementation soundness claim —
stays closed until §7.5's O1–O5 conjecture is discharged (NN#20 remains a hard
pre-atomics gate).

---

## 8. Observable effects

**Status: NORMATIVE-DRAFT.** This section defines the term **observable effect**
for **any** execution — faulting or not — so the ordering guarantees the aliasing
model (chapter 05 §6.2.5, §6.3.1(c)) and the fault window (§7.2) rest on have a
single normative anchor rather than each restating it. Rationale: design 0001
§11.3 (MMIO) and §7 (the fault window).

8.1 **The observable effects of an execution are, and are only:**

- **MMIO accesses.** A read or write through a `rawptr` formed from an integer
  address (`addr_to_ptr[T]`, chapter 05 §2.4) that denotes memory outside the
  abstract machine's own storage — a device window (design 0001 §11.3).
- **Foreign / boundary calls.** A call across the foreign boundary (a boundary
  module, chapter 08 §6) — its I/O and any effect on foreign state. No FFI surface
  ships this edition (OBL-FFI), so this class is presently empty; it is enumerated
  now so a future boundary module inherits the ordering rule rather than
  discovering it.
- **`trace`.** The program's explicit output channel: `trace(x)` appends to the
  observable trace `θ`.
- **Program completion.** The program's termination and its externally visible
  result — the return value / exit status and, on a faulting execution, the
  structured fault report (§6.2).

8.2 **The ordering rule the optimizer owes.** Observable effects occur in
    **program order relative to one another.** A conforming optimizing
    implementation SHALL NOT **invent**, **duplicate**, **elide**, or **reorder**
    an observable effect — neither across another observable effect nor across a
    fault (§6.3, §7.2). This is the whole of the ordering the optimizer owes an
    execution: non-observable computation MAY be reordered, coalesced, or elided
    subject only to the fault window (§7) and the aliasing bound (chapter 05 §6.3).

8.3 **Spec-level requirement vs. conservative implementation (non-normative
    note).** Of the classes in §8.1, only *MMIO accesses* make a `rawptr` access
    observable at the spec level: an ordinary (non-MMIO) `rawptr` access carries no
    observable-ordering guarantee here, and the aliasing model permits optimizing
    it (chapter 05 §6.2.3). Because an implementation generally cannot distinguish
    an MMIO `rawptr` from an ordinary one, chapter 05 §6.2.5 **requires** it to
    treat **every** `rawptr` access as observable — a sound over-approximation that
    only adds ordering constraints. The shipping backends do exactly this: every
    `ptr_read` / `ptr_write` is marked observable in MIR (`mir/build.rs`
    `mark_last_observable`) and lowered as a barrier call, so `rawptr` is fully
    **volatile** today — stronger than §8.1 alone requires (Appendix 05-A).
