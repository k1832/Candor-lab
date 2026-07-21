# 05 — Unsafe Code and Pointers

**Status: NORMATIVE-DRAFT.** §§1–5 (the valve and its audit line) are transcribed
from design `0001-memory-model` §4 and design `0004` `field_ptr`; §6 (the
unsafe-code aliasing model) discharges P18's named mandatory obligation OBL-ALIAS
(discharged for the single-threaded edition; the adversarial review's repair
condition is met — chapter 99, and
`docs/reviews/05-aliasing-model-review.md`). Rationale is in designs 0001 and 0004;
the shipping backends' actual assumptions for §6 are recorded in the non-normative
Appendix 05-A.

---

## 1. The unsafe region (the valve)

1.1 Raw-pointer operations SHALL be permitted **only** inside an `unsafe` region.

1.2 An `unsafe` region SHALL carry a **mandatory, non-empty justification**
    string. A conforming implementation SHALL enforce the justification's
    **presence** (not its truth) and SHALL record every unsafe region with its
    justification and source span, so the toolchain can enumerate everything a
    program trusts (P1/P17).

1.3 `unsafe` is a **block**, not a modifier: its extent is syntactic and
    greppable.

1.4 `unsafe` grants **exactly one** new power: raw-pointer operations (§2). It
    SHALL NOT disable move checking, borrow checking, overflow checking, or
    bounds checking on safe values; every rule of chapters 03, 04, and 06 that is
    not a raw-pointer operation SHALL still apply inside an `unsafe` block.

---

## 2. Raw pointers and the audit line

2.1 A `rawptr T` is an address. It may be null and is **not** tracked by the
    borrow checker or the ownership system.

2.2 **Holding, moving, copying, and comparing** a `rawptr` value is **safe** (it
    is a `copy` scalar; a `rawptr` field in a struct is inert in safe code).

2.3 **Every operation that creates, offsets, casts, or dereferences** a `rawptr`
    SHALL require an `unsafe` region. This is the audit line: every line that
    gives a raw pointer meaning is inside `unsafe`; safe code may shuffle
    addresses but SHALL NOT act on them.

2.4 The unsafe-gated operations are: `addr_of` / `addr_of_mut` (address of a
    place), `ptr_read` / `ptr_write` (bitwise read/store — `ptr_write` SHALL NOT
    drop the old value), `ptr_offset` (pointer arithmetic by element count),
    `ptr_null[T]` (null of element type `T`), `ptr_to_addr` / `addr_to_ptr[T]`
    (pointer↔integer, for MMIO and fixed addresses), and `cast_ptr[U]`
    (reinterpret element type). Target element types SHALL be written explicitly;
    there is no implicit pointer-type inference and no `as` cast (P2).

2.5 `ptr_read` yields an owned value by bit copy; the dereference forms SHALL NOT
    drop or move-track. The author is responsible for not creating two owners of a
    move-only value.

---

## 3. Safe pointer queries

3.1 `is_null(p)`, `offsetof(Type, field)`, and `sizeof`/`alignof` are **safe**:
    they give a pointer no meaning and confer no ability to act on memory.

3.2 **`field_ptr(p, f)`** (design 0004) is **safe**. It computes a field's
    address as `address(p) + offsetof(StructT, f)` unconditionally, performs no
    dereference and no null check, and yields an inert address. It is address
    arithmetic with a static proof, not free arithmetic, and does not move the
    audit line: dereferencing the result still requires `unsafe`.

3.3 `field_ptr(p, f)` is **well-formed only when** `p` has type `rawptr StructT`
    for a compiler-known struct and `f` is a **statically known field** of
    `StructT`; otherwise it is ill-formed (prototype E0510, a well-formedness
    diagnostic, not an unsafety gate).

---

## 4. Obligations transferred to the author

4.1 Inside `unsafe`, the following are the **author's declared responsibility**,
    carried by the justification string and **not** checked by the
    implementation: (a) the validity and initialization of every dereferenced raw
    pointer; (b) not creating two owners of a moved value via `ptr_read`; (c) the
    liveness of any state a raw pointer or allocator handle refers to for the
    lifetime of every copy of that handle and every `Box` it serves (chapter 08).

4.2 The safety guarantees of chapters 03, 04, and 06 hold for the **safe
    fragment** and say **nothing** about a bad raw-pointer operation. That
    transfer of obligation is the valve's meaning (P1).

---

## 5. Scope of the valve set

5.1 This edition ships **exactly one valve**: `unsafe` + raw pointers. It does
    **not** ship a checked-runtime interior-mutability cell (a `RefCell`-analog).
    A valve occurrence count measured under this specification is therefore an
    **upper bound** on real-language valve occurrence (design 0001 §4.3). Adding
    a checked-runtime alternative is a future design question (chapter 99).

---

## 6. The unsafe-code aliasing model

**Status: NORMATIVE-DRAFT.** Discharges OBL-ALIAS (P18's named mandatory-scope
item). This section states the **conservative** aliasing model: what each pointer
kind may alias (§6.2), the **upper bound** on what a conforming optimizing
implementation MAY assume (§6.3), the obligation an unsafe region carries to keep
that bound sound (§6.4), and the adopted-art position and future-tightening path
(§6.5). Rationale is in design 0001 §4; the shipping backends' actual assumptions
are the non-normative Appendix 05-A.

### 6.1 Posture

6.1.1 The model is an **upper bound.** §6.3 defines the **most** a conforming
    optimizing implementation MAY assume about aliasing. A conforming
    implementation MAY assume **less** — up to and including assuming nothing and
    reordering no memory access — and the shipping backends do (Appendix 05-A).
    Assuming **more** than §6.3 grants is a specification violation. This is the
    effects-as-upper-bounds asymmetry (P2) applied to the optimizer: the spec caps
    the license; a conforming implementation lives at or below the cap.

6.1.2 **No contract-derived assumption.** Nothing an optimizer may assume is
    derived from a contract (P8; chapter 07 §4). Contracts are checks, never an
    input to aliasing analysis.

6.1.3 **Composition.** The model composes with the fault model (chapter 06, P5)
    and the memory consistency model (chapter 09): an aliasing assumption SHALL
    NOT license reordering an access across a fault edge or an observable that
    those chapters forbid (chapter 06 §6.3, §7.2).

### 6.2 The pointer taxonomy — what each kind may alias

Every pointer-typed access is one of the following kinds. Its **aliasing set** is
what other accesses may touch the same storage.

6.2.1 **Shared borrow (`read T`).** While a `read` borrow's loan is live, the
    checker's XOR rule (chapter 04 §4.1) guarantees the borrowed place is **not
    written through any safe path**. Multiple live `read` borrows of one place may
    coexist and alias one another; none may write. **Aliasing set:** other `read`
    borrows of the same place, and any `rawptr` (§6.2.3).

6.2.2 **Exclusive borrow (`write T`).** While a `write` borrow's loan is live, the
    checker's XOR rule guarantees the borrowed place is accessed — read or written
    — through **that borrow only** on every safe path (chapter 04 §4.1). Within the
    safe fragment, two distinct live `write` borrows therefore never alias, and a
    live `write` borrow never aliases a live `read` borrow. **Aliasing set:** itself
    only, within the safe fragment; and any `rawptr` (§6.2.3).

6.2.3 **Raw pointer (`rawptr T`).** A `rawptr` is untracked (§2.1). A `rawptr`
    access MAY alias **any** storage — any live `read` or `write` borrow, any box,
    any static, any other `rawptr`, any MMIO window. It is unsafe code's escape
    hatch, and it is the **only** construct that can alias a live borrow.
    **Aliasing set:** everything.

6.2.4 **Box / allocator-returned memory (chapter 08).** A `Box T` uniquely owns
    its pointee while owned; safe code reaches the pointee only through the box, or
    through a borrow reborrowed from it (which then obeys §6.2.1–6.2.2).
    **Aliasing set:** as the borrow reborrowed from it; and any `rawptr`.

6.2.5 **MMIO address (`addr_to_ptr[T](a)`).** A `rawptr` formed from an integer
    address (§2.4) denotes memory outside the abstract machine's own storage — a
    device window (design 0001 §11.3). Its accesses are **observable effects**
    (chapter 06 §8): they SHALL NOT be invented, duplicated, elided as dead, or
    reordered across one another or across any other observable effect or fault
    (the ordering rule of chapter 06 §8.2).
    **Aliasing set:** everything (it is a `rawptr`), plus this observable-ordering
    guarantee. A conforming implementation that cannot distinguish an MMIO `rawptr`
    from an ordinary one SHALL treat **every** `rawptr` access as observable — a
    sound over-approximation (Appendix 05-A records that the backends do exactly
    this today).

### 6.3 The optimizer's license — the upper bound

6.3.1 A conforming optimizing implementation MAY assume the following, **and no
    more**:

  (a) **Safe-borrow non-aliasing.** Two accesses reaching storage through
      **distinct live `write` borrows**, or through a live `write` borrow and a
      live `read` borrow, or through a live `read` borrow and a direct safe write,
      do not alias — **justified solely by the checker's XOR loans** (chapter 04
      §4.1), never by any other means. Equivalently: within a `write` borrow's live
      range its pointee is modified only through that borrow — *absent an aliasing
      `rawptr`* (§6.4).

  (b) **No `rawptr` assumption.** A `rawptr` access (§6.2.3) may alias any storage.
      The optimizer SHALL treat a `rawptr` load as reading, and a `rawptr` store as
      writing, **unknown memory** that may include any live borrow's pointee, any
      box, and any static. It gains **no** non-aliasing assumption from the
      presence, type, or provenance of a `rawptr`.

  (c) **Observable ordering.** Accesses a conforming implementation treats as
      observable — MMIO, and any `rawptr` access it does not prove non-observable
      (§6.2.5) — SHALL NOT be invented, duplicated, elided, or reordered across one
      another or across faults/other observable effects (the ordering rule of
      chapter 06 §8.2, which defines the observable-effect set for any execution).

  (d) **No contract-derived assumption** (§6.1.2, P8).

6.3.2 Clause (a) is the **only** non-aliasing assumption the model grants, and it
    rests entirely on the safe checker. Because a `rawptr` may alias a live borrow
    (§6.2.3), (a) is sound **only** in the absence of an aliasing `rawptr` access —
    and the checker cannot see raw pointers (§2.1). The obligation that no such
    aliasing exists is therefore carried by unsafe code (§6.4), not by the
    optimizer.

### 6.4 Obligations of an unsafe region

6.4.1 The checker guarantees §6.3.1(a) only over the **safe fragment**. A `rawptr`
    can materialize a second access path to storage a live borrow views, which the
    checker never sees (§2.1). Keeping §6.3.1(a) sound is therefore an **author
    obligation**, carried by the `unsafe` justification (§1.2, §4.1):

    > While a `read` or `write` borrow of a place is live, unsafe code SHALL NOT
    > **write** that place's storage through an aliasing `rawptr` (nor otherwise
    > modify it other than through a live `write` borrow). Violating this is
    > undefined behavior **in unsafe code** — the one aliasing fact the checker
    > cannot check (§4.1).

    A `rawptr` **read** aliasing a live borrow is **not** UB in this model: the only
    non-aliasing the optimizer is granted (§6.3.1(a)) is non-aliasing of
    *modification*, which a read cannot falsify. This is a deliberate narrowing from
    a Rust-style model, where forming or reading through a pointer aliasing a live
    reference is itself UB (§6.5.1).

6.4.2 This rule constrains the **unsafe author**, not a safe program. No safe
    program can violate it, because safe code cannot create a `rawptr` (§2.3). So
    NN#1 (no UB in safe code) is preserved: §6.4.1's UB is reachable only from
    inside `unsafe`, where §4.2's transfer of obligation already applies.

6.4.3 **What unsafe code may rely on in return.** Because a conforming
    implementation SHALL NOT assume more than §6.3, unsafe code MAY rely on the
    optimizer **not** doing the following: (a) it will not invent, duplicate, elide,
    or reorder an MMIO/observable access (§6.2.5, §6.3.1(c)); (b) it will not treat
    a `rawptr` access as non-aliasing with a borrow (§6.3.1(b)) — a `rawptr` write
    is always assumed able to modify any borrow's pointee, box, or static; (c) it
    will not derive any assumption from a contract (§6.1.2). A `rawptr` access
    ordered in the program is ordered in the execution relative to observables and
    faults, per chapters 06 and 09.

### 6.5 Adopted-art position (P18)

6.5.1 The model adopts the **C-style conservative** rule for raw pointers — *a raw
    pointer may alias anything* (§6.2.3) — and **explicitly does not adopt** a
    Rust-style pervasive-reference aliasing-UB model (Stacked Borrows / Tree
    Borrows) for `rawptr`. Rust's model exists because `&`/`&mut` are the language's
    *pervasive* reference types, whose non-aliasing must be exploited for
    performance and therefore made UB to violate. Candor's `rawptr` is a **visible,
    greppable valve** (§1, §2.3), not a pervasive reference type: the safe borrows
    carry the performance-relevant non-aliasing (§6.3.1(a)) and the valve stays
    conservative. The single narrow aliasing-UB rule Candor states (§6.4.1) is the
    **coarse** obligation "do not alias a live borrow," **not** a per-access
    provenance/retag discipline — the model tracks no pointer provenance.

6.5.2 Rust's decade-long retrofit of an unsafe-aliasing model **after** shipping
    (Stacked, then Tree Borrows) is the studied cautionary prior art (P18): the cost
    of that ordering is precisely why this model is stated **before** an optimizer
    relies on it. Candor states the conservative model now and defers the tightening.

6.5.3 **Future tightening path.** A future edition MAY grant the optimizer more than
    §6.3 — for example, lowering `write` borrows with a `noalias` attribute
    (borrow-based non-aliasing exploited across opaque calls), or optimizing
    non-MMIO `rawptr` accesses. Because that would **narrow** the set of sound
    programs (unsafe code sound today could become UB) or change observable
    optimization, it is a **breaking change** and SHALL ship only by amendment with
    an automatic migrator story (P15/NN#14; chapter 00 §3.2) and a restatement of
    §6.3/§6.4. **In particular, a borrow-based `noalias` tightening WITHDRAWS
    §6.4.1's `rawptr`-*read* carve-out.** LLVM `noalias` promises the optimizer that
    a `noalias` pointee is neither written **nor read** through any pointer not
    derived from that borrow; so under such an amendment a foreign **read** through a
    `rawptr` aliasing a live borrow — explicitly **not** UB today (§6.4.1) — becomes
    UB too. That amendment is therefore a **restatement of §6.4's aliasing
    obligation by amendment** (the read carve-out withdrawn, so both reads and
    writes through an aliasing `rawptr` are UB), not merely a narrowing of
    otherwise-sound programs, and its migrator SHALL flag the newly-UB foreign
    reads. Until such an amendment, §6.3 is the ceiling and §6.4.1 is the only
    aliasing obligation on unsafe code.

**Gate discharged.** OBL-ALIAS's acceptance criterion — the optimizer assumptions
each unsafe operation of §2 preserves or breaks, composed with chapter 09, with
Rust's Stacked/Tree Borrows studied as cautionary art — is met, rigorous-informal
(chapter 99, OBL-ALIAS). The gate on an optimizing implementation's soundness claim
is: assume no more than §6.3.

---

## Appendix 05-A (non-normative) — the shipping backends' actual assumptions

Informative; the normative content is §6. This records **what the shipping backends
assume today**, verified by reading the emitters, substantiating §6.1.1: every
backend assumes strictly **less** than §6.3 grants (permitted conservatism).

- **MIR optimizer** (`compiler/src/mir/opt.rs`). The only MIR pass is dead-local
  elimination of pure, non-fault-capable `τ`-assignments whose destination local is
  dead. It never removes an observable (rawptr/MMIO/`trace`), never a fault-capable
  op, and makes **no** aliasing assumption — no redundant-load elimination across
  pointers, no reordering. It assumes far less than §6.3.1(a).

- **Cranelift** (`compiler/src/backend/lower.rs`). Every load/store uses the default
  `MemFlags::new()` — no `readonly`, `notrap`, or aliasing-region flag, and no
  `noalias`. At `opt_level=none` nothing is reordered; when Stage D flips
  `opt_level=speed`, the egraph runs but every rawptr/MMIO/fault access is a barrier
  **call** it will not reorder past or elide. No non-aliasing assumption is emitted.

- **LLVM** (`compiler/src/backend/llvm.rs`). Emits **no** `!tbaa`, `noalias`,
  `!alias.scope`, or `!invariant.load` metadata (grep of the emitter: none). All
  memory lives in one flat arena addressed by `inttoptr(MEM_BASE + off)`, so
  `clang -O2`'s own alias analysis cannot prove any two accesses non-aliasing (they
  share one base). rawptr/MMIO accesses are `rt_mmio_load`/`rt_mmio_store` calls
  (opaque barriers `-O2` will not reorder or elide). The only function attribute
  emitted is `noreturn`. ROADMAP records per-object allocas + TBAA as **deferred**.

- **rawptr treated as volatile today.** Every `ptr_read`/`ptr_write` through a value
  of type `rawptr` (not a borrow or box) is marked **observable** in MIR
  (`compiler/src/mir/build.rs`, `mark_last_observable`) and lowered as a barrier call
  on both backends. Today rawptr is thus treated as fully **volatile** — stronger
  than §6.2.3 requires (the model would permit optimizing a non-MMIO rawptr access;
  the backends do not).

**Net.** No backend emits an aliasing assumption a `rawptr` could falsify, so
§6.4.1's obligation is currently un-exercised. It is stated now so that a future
backend which begins deriving §6.3.1(a) borrow-non-aliasing inherits the obligation
rather than discovering it — P18's "budgeted, not discovered."
