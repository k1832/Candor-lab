# 05 — Unsafe Code and Pointers

**Status: NORMATIVE-DRAFT** (§§1–5, the valve and its audit line, transcribed from
design `0001-memory-model` §4 and design `0004` `field_ptr`) **+ SKELETON** (§6,
the unsafe-code aliasing model — P18's named mandatory obligation). Rationale is
in designs 0001 and 0004.

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

## 6. The unsafe-code aliasing model — SKELETON (P18, mandatory scope)

**Status: SKELETON.** This section records the obligation and its open questions
precisely. It is P18's named mandatory-scope item: *what unsafe code may assume
about the references and pointers it touches, and — the load-bearing dual — what
an OPTIMIZER may assume about memory an unsafe region touches.* No optimizer
exists in the prototype (chapter 06 §3), so no answer has been forced yet; this
is the cheapest point to record the questions.

6.1 **Obligation.** The specification SHALL, before stability, define the
    aliasing model for unsafe code: the guarantees an optimizer may rely on
    across `rawptr` operations and across the safe/unsafe boundary, and the
    obligations an unsafe author must meet to keep NN#1 (no UB in safe code)
    sound underneath the safe aliasing rules of chapter 04.

6.2 **Recorded open question (materialization).** May unsafe code **materialize a
    place** (form a borrow, or otherwise re-enter the tracked ownership/borrow
    system) from a `rawptr` **while a loan on the underlying storage is live**?
    The prototype's posture is *author's-responsibility* — the checker does not
    track raw pointers (§2.1), so it neither permits nor forbids this at the type
    level. The specification MUST eventually state what the **optimizer** may
    assume when it cannot see such a materialization — i.e. whether a raw write
    through an aliasing `rawptr` may invalidate an optimizer's assumptions derived
    from a live safe borrow.

6.3 **Cautionary art to study (P18).** Rust's decade-long effort on a formal
    aliasing model for unsafe code (Stacked Borrows, then Tree Borrows) is the
    named cautionary prior art: a model retrofitted **after** the language shipped,
    at large cost. Candor SHALL study it and, per P18, adopt proven art where it
    suffices and name any residual novelty as tracked pre-stability work.

6.4 **Acceptance criterion.** The aliasing model is discharged when the
    specification states, for every unsafe operation of §2, the optimizer
    assumptions it preserves or breaks, composed consistently with the memory
    consistency model of chapter 09; mechanized where feasible, rigorous-informal
    at minimum (chapter 99, obligation OBL-ALIAS).

**Gate:** blocks any optimizing implementation's soundness claim and any
stability commitment. Does not block the prototype (no optimizer).
