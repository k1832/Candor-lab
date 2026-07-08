# 08 — Effects

**Status: NORMATIVE-DRAFT** (§§1–5, the `alloc` effect and its partition,
transcribed from design `0001-memory-model` §3.2 and §6) **+ SKELETON** (§6,
foreign-trust — no FFI in this edition). Rationale is in design 0001 and
philosophy P2/P9.

---

## 1. The tracked-effect set

1.1 The tracked-effect set is **closed and tiny**: **allocation** and, when FFI
    lands, **foreign trust** (P2/NN#19). Nothing else is a tracked effect in this
    edition. Growing the set is an amendment (NN#19), shipped with
    conservative-default migration.

1.2 A function that may allocate SHALL declare the `alloc` effect on its
    signature. Fallibility is not an effect (it is in the return type, P7);
    fault-potential is not tracked.

---

## 2. Upper-bound semantics

2.1 An effect marker is an **upper bound**: a signature MAY overstate, SHALL
    NEVER understate (NN#19). A function marked `alloc` that never allocates is
    permitted conservatism. Removing the marker is a non-breaking strengthening;
    adding it is a breaking change (chapter 00 §3.3).

---

## 3. The one-way partition

3.1 A non-`alloc` function **SHALL NOT** call an `alloc` function; an `alloc`
    function MAY call anything. This is the one-way partition with a **universal
    ground floor**: allocation-free code is callable from everywhere (P2).

3.2 The capability travels as an ordinary **value** (the allocator handle), not
    as a function-type transformation; one API serves both worlds by taking an
    allocator.

---

## 4. The allocation effect — both sides of the allocator

4.1 The following make the enclosing function `alloc`-marked. The **alloc side**:
    a `box`; a `clone` of a value that transitively bears a `Box`; a call to an
    `alloc`-marked function; an indirect call through an `alloc`-typed
    fn-pointer.

4.2 The **free side** (freeing is allocator work, equally forbidden in a
    non-`alloc` context): `unbox`; and **any scheduled drop of a `Box`-bearing
    value** — a scope-exit drop of a needs-drop box-bearing local, a
    reassignment-drop or `out`-drop of one, a box-bearing temporary dying at
    statement end, or a function-exit drop of an owned box-bearing parameter still
    live on some path. Consequently a function that receives a `Box` by value and
    **lets it die** is `alloc`-marked; a function that instead **moves it out**
    (returns or passes it on) is not.

4.3 A type whose `drop` **hook body** is `alloc`-effecting is **alloc-on-drop**
    (chapter 03 §6.4): a scheduled drop of it — or of any value transitively
    owning one — propagates the `alloc` requirement exactly like a `Box`.

---

## 5. Fn-pointer effect typing and indirect calls

5.1 A function-pointer type SHALL include its parameter modes **and** its effect
    marker. Assigning an `alloc`-marked function to a non-`alloc` fn-pointer type
    is **ill-formed** (it would understate the effect, NN#19); assigning a
    non-`alloc` function to an `alloc`-typed slot is permitted conservatism
    (§2.1).

5.2 An **indirect call takes its effect from the pointer's type**: a call through
    an `alloc`-typed pointer makes the caller `alloc`; a call through a
    non-`alloc`-typed pointer does not. There is **no** special case for any
    vtable — its fields carry the effect in their types, so every call through
    them follows this one general rule.

---

## 6. Foreign trust — SKELETON (P17)

**Status: SKELETON.** No FFI / boundary modules exist in this edition, so the
foreign-trust effect and its audit surface are empty and untested (design 0001
§9). This section records the obligation.

6.1 Every foreign call SHALL be unsafe in principle; safe code SHALL call a
    foreign function **only** through a declared **boundary module** (P17/NN#18)
    that localizes all FFI unsafety, attaches contracts (chapter 07) to foreign
    signatures, and is enumerable by the toolchain.

6.2 Most boundary contracts will be `assumed-proven` **trust declarations**, not
    verified facts (P17); the checkable value-level subset is checked, the rest is
    trust made visible, structured, and enumerable.

6.3 **Foreign trust SHALL join the closed effect set** (§1.1) when FFI lands, as
    a tracked effect on boundary-module signatures, shipped with conservative-
    default migration (NN#19).

6.4 **Acceptance criterion.** Discharged when the specification defines boundary
    modules, the foreign-trust effect's typing and partition, and the enumeration
    surface (chapter 99, obligation OBL-FFI).

**Gate:** blocks any FFI feature and the P17 audit story; does not block the
pure-Candor core.
