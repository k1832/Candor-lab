# 04 — Ownership and Borrows

**Status: NORMATIVE-DRAFT.** Complete transcription of design `0001-memory-model`
§2 (borrowing), §3 (signatures), and §5.2 (slices as borrows), as amended by
design `0005` (implicit call-site reborrow). Rationale is in designs 0001 and
0005; this chapter states rules only. It speaks in **clauses**, not error codes;
the non-normative appendix maps clauses to the prototype's diagnostic codes.

---

## 1. Loans and the two borrow kinds

1.1 A **borrow** is produced by a keyword operator on a place. A **shared borrow**
    (`read place`, type `read T`) is read-only, aliasable, and `copy`. An
    **exclusive borrow** (`write place`, type `write T`) is read-write,
    unique, and **moves** (is not `copy`). An exclusive borrow REQUIRES a mutable
    place.

1.2 A **loan** is the restriction a borrow places on the storage it views. Every
    borrow expression creates a loan on its place, tagged shared or exclusive,
    in scope over the borrow's live range (§4).

---

## 2. Dereference and reborrow

2.1 `b.*` is a **place** denoting the borrowed storage. On the right of `=`
    it reads (copying if `copy`, or serving as a place to reborrow); on the left
    it writes, and a deref-write SHALL be permitted **only** through an exclusive
    borrow (or a `Box`). Every `.*` on the path to a written place SHALL peel
    an **exclusive** borrow (or a `Box`), never a shared one — including a shared
    deref reached via autoderef or nested under an exclusive one.

2.2 A read through `.*` SHALL NOT move the pointee out: moving a non-`copy`
    value out through any place whose path contains a `.*` or an index is
    **ill-formed** (chapter 03 §3.3; the defined extraction of a `Box` pointee is
    `unbox`, chapter 08).

2.3 A borrow of a place through another borrow is a **reborrow**. An exclusive
    reborrow `write b.*` yields a fresh exclusive borrow constrained not to
    outlive `b`; a shared reborrow `read b.*` yields a fresh shared borrow
    of the pointee (legal even when `b` is exclusive).

2.4 **The reborrow rule (both directions).** For the reborrow's live range: an
    **exclusive reborrow SHALL suspend the parent entirely** (while live, `b` may
    not be read, written, or further reborrowed); a **shared reborrow SHALL
    freeze the parent to shared** (while live, the pointee may be read or
    re-shared through `b`, but `b` may not write and may not be reborrowed
    exclusively). Each restriction lasts exactly the reborrow's live range.

---

## 3. Implicit call-site reborrow (design 0005)

3.1 In argument position for a parameter of mode `read` or `write`, an argument
    that is a **place already denoting a borrow** whose pointee type and
    shareability admit the parameter's mode is a **reborrow**, not a move,
    governed by §2.4.

3.2 The admission rule: `write`-mode parameter with an exclusive source yields an
    exclusive reborrow; `read`-mode parameter with an exclusive-or-shared source
    yields a shared reborrow; `write`-mode parameter with a **shared** source is
    **ill-formed** (cannot reborrow exclusive from shared).

3.3 Lending **owned** storage to a `read`/`write`-mode parameter SHALL keep its
    keyword (`f(write x)`, `f(read x)`); passing an owned place bare to a mode
    parameter is ill-formed. Passing a borrow-typed value to a by-value (`take`)
    parameter is the value gear (a shared borrow copies, an exclusive borrow
    moves), bare and unkeyworded.

3.4 The explicit spellings `read b.*` / `write b.*` remain accepted
    input; in the real toolchain the canonical formatter (NN#11) normalizes them
    to the bare form in reborrow-argument position (a P16 obligation, chapter 99).

---

## 4. The aliasing rule (XOR) and access classification

4.1 **XOR.** At every program point, for every place, the set of **live** loans
    (§5) reaching it SHALL satisfy: **either** any number of shared borrows,
    **xor** exactly one exclusive borrow — never both, never two exclusives.

4.2 Consequently: while an exclusive loan of a place is live, the place SHALL be
    accessed **only** through that borrow; while any shared loan of a place is
    live, the place MAY be read directly and re-shared but SHALL NOT be written or
    exclusively borrowed.

4.3 **Moves and writes are exclusive accesses.** A move out of a place (whole or
    partial) and a direct write or reassignment to a place SHALL each conflict
    with **any** live loan — shared or exclusive — on that place or any
    overlapping place.

4.4 **Place granularity.** Borrows are tracked at place granularity. Overlapping
    places (e.g. `p` and `p.f`) conflict; **disjoint fields** (`p.f` and `p.g`) do
    **not** conflict. Any index `a[i]` is treated as covering the **whole array**
    `a` (no index-sensitive disjointness in this edition; a sound
    over-approximation).

---

## 5. Borrow duration — observable acceptance rules

The following are stated as **observable acceptance rules** over borrow **live
ranges**, not as a mandated algorithm. A conforming implementation MAY compute
live ranges by any means (the prototype uses body-local non-lexical liveness,
"NLL-lite"), provided it accepts exactly the programs these rules accept.

5.1 A borrow's **live range** is body-local and **not** tied to lexical blocks.
    A borrow (and any reborrow taken from it) is **live** at a point P iff some
    path from P reaches a **use** of it — a `.*`, a pass-by-borrow, a store of
    it, or a reborrow of it — without first passing through a redefinition of the
    binding that holds it.

5.2 A loan is **in scope** exactly over the live range of the borrow value(s)
    carrying it, reborrows included (a reborrow's loan extends the parent's
    obligation, §2.4).

5.3 A program SHALL be accepted with respect to aliasing iff, at every point, for
    every place, the in-scope loans satisfy §4.1 under the access classification
    of §4.2–§4.3.

5.4 **No two-phase borrows** in this edition. A pattern reserving an exclusive
    borrow while a shared borrow is briefly used to compute an argument (e.g.
    `push(write v, read v[0])`) SHALL be rejected. This is an accepted
    over-approximation (chapter 03/04 rationale; design 0001 §2.3).

---

## 6. Signature modes

6.1 A parameter is written `name: MODE Type`. There are **four** modes; omitting
    the mode means `take`. This ordering is the value-first bet made visible
    (P12).

6.2 **`take`** (default): the argument is moved in (or copied, if `copy`); the
    callee receives an owned value it may mutate, move, or drop; ownership
    transfers **in**.

6.3 **`read`**: the argument is borrowed shared; the caller retains ownership; the
    callee gets a shared borrow (may read and re-share; SHALL NOT mutate or move).

6.4 **`write`**: the argument is borrowed exclusively; the caller retains
    ownership and SHALL NOT touch the place during the call; the callee gets an
    exclusive borrow (may read and mutate; SHALL NOT move the pointee out).

6.5 **`out`**: the caller passes a place it owns; the callee receives a slot it
    **SHALL initialize on every normal-return path** and **SHALL NOT read before
    its first assignment**; the caller keeps ownership.

6.6 Return values (`-> T`) **move out**; RVO is a permissible optimization but is
    semantically a move, never a hidden copy.

6.7 **`out` rules.** (a) `out place` produces an **exclusive loan** on the place
    for the call, conflicting by §4 with any other argument in the same call that
    touches an overlapping place. (b) A pre-initialized `out` slot has its current
    value **dropped at the call site before the call** (chapter 03 §6.8, §7.5).
    (c) The callee SHALL leave the slot definitely assigned on every
    normal-return path; repeated assignment is permitted; if the callee faults
    before assigning, the slot stays uninitialized and is not dropped by the
    caller.

6.8 **Passing borrow-typed arguments.** A slice or borrow-typed value is passed
    **by value**, not re-annotated with a mode: a shared borrow / shared slice
    (`[T]`) copies in; an exclusive borrow / exclusive slice (`write [T]`) moves
    in, or is reborrowed at the call site (§3). A `read`/`write` mode written on a
    parameter whose type is already a borrow kind (a borrow, `[T]`, or `write [T]`)
    is **ill-formed**. (A
    `write usize` parameter is well-formed: `usize` is not a borrow kind.)

---

## 7. Regions and the compact default

7.1 When a function returns a borrow, the caller SHALL be able to know **which
    input** that borrow derives from **without inference** (NN#17). Region
    relationships that cross the signature SHALL be written in the signature;
    borrows **inside** a body are inferred body-locally (§5).

7.2 **Region variables** are declared after the function name in a bracketed
    declaration list, each wearing the `region` keyword
    (`fn f[region r](a: read[r] T) -> read[r] T`; design 0007 §6.1.1, chapter 10
    §1.3), and attach to borrow parameters and borrow returns by the region tag
    `[r]`; the returned borrow derives from the
    region-tagged parameter. The checker SHALL verify, body-locally, that the
    returned borrow's provenance is reachable through the tagged parameter.

7.3 **Compact default.** If a function has **exactly one** borrow parameter and
    returns a borrow, and no region variables are written, the returned borrow
    **SHALL be defined** to derive from that sole borrow parameter (a syntactic
    rule applied by inspection, not inference).

7.4 **The rare case carries the weight.** If a function has **two or more** borrow
    parameters and returns a borrow, region variables SHALL be **mandatory**; an
    unannotated borrow return SHALL be rejected. Ambiguity SHALL NOT be silently
    resolved.

7.5 A returned borrow whose provenance is a **local** (or an owned `take`
    parameter) SHALL be rejected: a borrow SHALL NOT outlive the body it was born
    in. This is checked body-locally.

---

## 8. Storage restriction on borrows

8.1 A struct or enum field **SHALL NOT** have a borrow type (shared or exclusive
    borrow, `[T]`, or `write [T]`). Owned values and `rawptr T` fields are
    permitted.

8.2 Borrows are a gear for **passing and computing**, first-class as parameters,
    locals, and return values — **never a storage class**. A stored inter-object
    reference SHALL be an owning relation (`Box`, nested value), a handle/index
    (a `copy` integer into a slice or arena), or a `rawptr` (the valve,
    chapter 05).

---

## 9. Pattern bindings

9.1 The binding mode of each payload variable in a `match` pattern is fixed by
    **how the scrutinee is held**, mirroring the gears.

9.2 **Owned scrutinee.** Each named payload is taken by the value gear: it moves
    out of the payload sub-place, or copies if the payload type is `copy`. A move
    binding partial-moves the scrutinee under chapter 03 §7 (rejected when the
    scrutinee type has a `drop` hook; move states SHALL agree at joins).
    Unnamed payloads stay owned by the scrutinee and drop with it.

9.3 **Borrowed scrutinee.** Reached through a shared borrow, each named payload
    binds as a shared borrow of its sub-place; through an exclusive borrow, as an
    exclusive borrow. These are ordinary loans (§4, §5); matching SHALL NOT move
    out of a borrowed scrutinee. A `copy` payload MAY instead be read out as an
    owned copy at the match head, ending that payload's loan there.

9.4 **Nested patterns** compose these rules one level at a time.

9.5 A `match` SHALL be **exhaustive**.

---

## Appendix 4-A (non-normative) — clause-to-prototype-diagnostic map

This mapping is informative. The specification speaks in clauses; the prototype
checker (`compiler/src/check/`, design 0003) emits E-numbered diagnostics. The
map aids implementers and the diagnostic taxonomy (P4/P19); it is not normative.

| Spec clause | Prototype diagnostic |
|-------------|----------------------|
| 03 §3.3 use after move | E0301 |
| 03 §7.3 move-agreement at joins | E0302 |
| 03 §7.4 partial move of drop-hooked type | E0303 |
| 03 §7.6 / 06 §5 uninitialized read | E0304 |
| 06 §5 `out` slot not assigned / read early | E0305 / E0306 |
| 03 §7.5 conditional-init at a drop point | E0309 |
| 04 §2.2 non-copy move through deref/index | E0310 |
| 03 §10.2 write to immutable static | E0311 |
| 04 §6.7 effect/alloc partition (see ch08) | E0401 / E0402 |
| 05 §2 unsafe-gated raw-pointer op | E0501 / E0502 |
| 05 §3.3 `field_ptr` well-formedness | E0510 |
| 09-pattern exhaustiveness (04 §9.5) | E0601 |
| 04 §9 pattern arity / wrong-variant | E0604 / E0605 |
| 07 read-only contract clause | E0708 |
| 04 §4 Borrow-vs-Borrow conflict | E0801 |
| 04 §4.3 Move-vs-loan conflict | E0802 |
| 04 §4.3 Write/`out`-vs-loan conflict | E0803 |
| 04 §4.2 Read-vs-exclusive-loan conflict | E0804 |
| 04 §6.7 same-call overlap (no two-phase) | E0805 |
| 04 §7.2/§7.4/§7.5 return-provenance | E0806 / E0807 / E0808 |
| 04 §2.1 write through shared borrow | E0809 |
| 06 §6 all-paths-return | E0810 |
