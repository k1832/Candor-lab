# 12 — Iteration and Associated Types

**Status: NORMATIVE-DRAFT.** Transcription of design
`0009-iteration-and-associated-types`: the associated-type projection rules, the
two by-value iteration protocols, the `for` statement and its grammar hooks, and
the desugar stated as **observable semantics** (not a mandated algorithm).
Rationale is in design 0009; this chapter states rules only. It builds on
chapter 10 (interfaces, opacity, coherence) and **discharges OBL-GENERICS-ITER**
(chapter 99).

---

## 1. Associated-type projection

1.1 An interface's single associated type (chapter 10 §3.1) is introduced by the
    contextual keyword **`type`** in the interface body (`type Item;` — no bound,
    no default) and fixed by each impl (`type Item = u8;`). It is determined **by
    the impl**, not supplied by the user (design 0009 §2.1).

1.2 **Projection `T::Item`.** Inside `fn f[T: I](…)`, the associated type is the
    path-form projection `T::Item` (`::` the path separator, chapter 11 §4). It
    denotes an **opaque projected type** — like an unbounded parameter (chapter 10
    §2.2) it is movable, droppable, borrowable, storable, and nothing else — and
    definition-site checking treats it exactly as an opaque `T` (design 0009 §2.2;
    chapter 10 §6.3).

1.3 **The projection needs no `<T as I>::Item` qualifier.** Because at most one
    `impl I for C` exists (chapter 10 §7.3), `C::Item` is **single-valued** — no
    second impl to disambiguate against. The projection SHALL stay a bare path
    (design 0009 §2.2).

1.4 **The refusals kept intact** (each a debt, reopenable by amendment): **no
    bound on an associated type**, **no generic associated types**, **no
    associated-type default in the interface**, and **no more than one member per
    interface** (design 0009 §2.3). Coherence is unchanged — the member is fixed
    by the impl and keyed by its `(I, C)` pair (chapter 10 §7).

1.5 **Normalization at instantiation.** When the base of a projection is made
    concrete, `T::Item` SHALL normalize to a single concrete leaf — transitively
    through adapter-over-adapter bindings, under a fixed depth bound — or the
    program is rejected at check time; no unresolved projection reaches codegen.
    Chapter 13 §5 states this obligation normatively (design 0009 §2.2).

---

## 2. The two iteration protocols as library definitions

2.1 Iteration is defined over **two library interfaces**, each with exactly one
    associated type, selected by the operand's borrow mode (design 0009 §3):

        enum IterStep[T, S] { ok More(T, S), Done }

        interface Iter {                                   // consuming; operand moved
            type Item;
            fn next(take self) alloc -> IterStep[Item, Self];
        }

        interface Indexed {                                // borrow-copy; operand borrowed
            type Item;
            fn at(read self, i: usize) -> Opt[Item];       // Some(copy) in range, None past end
        }

2.2 **`Iter::next` consumes `self`** (chapter 10 §3.3) and returns the item
    alongside the **successor iterator**, a functional step that stores no borrow
    and moves no element through a borrow. **`Indexed::at`** takes `read self` and
    a plain `usize` cursor the loop owns (not stored in any iterator), copying the
    item out (the impl requires `Item: copy`) (design 0009 §3.1–§3.2).

2.3 **How the spec pins these library shapes (the mechanism, stated honestly).**
    The `for` statement is defined by **desugaring to method calls on the
    interfaces named `Iter` and `Indexed` and the enums `IterStep`/`Opt`**, and
    the spec pins the shapes by **fixing the exact member signatures of §2.1**: a
    conforming program's `for` binds to interfaces of these canonical names,
    resolved as **`core` library items** (chapter 11 §10.3), that have exactly
    these signatures. **Recorded honestly:** design 0009 fixes the *shapes* but
    does **not** specify the *resolution mechanism* (compiler-known lang-items on a
    blessed `core` path versus ordinary in-scope name lookup), and it uses
    `Opt[T]`/`Opt::Some`/`Opt::None` without defining `Opt` in designs 0007–0009.
    Both are transcription gaps (chapter 99, OBL-ITER-PIN); this chapter records
    the mechanism it adopts (well-known `core` interfaces, shapes fixed here) and
    the fact that the designs did not establish it.

---

## 3. The `for` statement

3.1 **Canonical form** (design 0009 §4.1):

        for PATTERN in OPERAND { STATEMENTS }

    `PATTERN` is any irrefutable pattern; `break`/`continue` work as in
    `loop`/`while` (P3 — same vocabulary).

3.2 **Grammar hooks (coordinated with chapter 02).** `for` and `in` are
    **contextual keywords**: keywords only in the for-statement header (`for` in
    statement-leading position, `in` separating pattern from operand), ordinary
    identifiers everywhere else. The **operand parses as `ExprNoStruct`** (chapter
    02 §8.2) — a bare struct literal is excluded so the `{` opening the loop body
    is never misread as a struct value; a `Range`-style literal operand SHALL be
    parenthesized. The parser need not know whether the operand is `Iter` or
    `Indexed` (a checker fact, like `Name[…]` arity) (design 0009 §4.4; chapter 02
    §5, §8).

3.3 **Protocol selection is syntax-directed by the operand's borrow mode:**
    - `for x in coll { … }` — `coll` **owned**; requires `typeof(coll): Iter`;
      **moves** `coll`.
    - `for x in read coll { … }` — requires `typeof(coll): Indexed`; **borrows**
      `coll` `read` for the loop's extent; `x` is a **copied** `Item`.

    (design 0009 §4.1). Borrowed-item (`read Item`) and mutating (`write coll`)
    iteration are the named cut (§6).

---

## 4. The desugar as observable semantics

The `for` statement is **sugar over `loop` + `match` + method calls**; a
conforming implementation MAY lower it by any means that yields exactly the
observable semantics below (no hidden allocation and no hidden control flow — any
allocation is the impl's marked `next`/`at`, §5; P4/P9). The desugar is
rendered as source on request.

4.1 **Consuming (`Iter`).** `coll` is **moved** into a fresh iterator local; each
    turn calls `next` (which **consumes** the iterator), binding `More(x, rest)`
    to run the body with the successor restored, or `Done` to exit (design 0009
    §4.2).

4.2 **Exit-edge consumption and E0309-path-independence.** Because `next(take
    self)` moves the iterator local out every turn and only the `More` edge
    restores it, on the `Done` edge the iterator is **uninitialized**. Chapter 03
    §7.5 requires init state to be **path-independent at any drop point**, so
    **every** exit edge reaching the post-loop point SHALL agree the iterator is
    uninitialized. The observable rule:
    - the **`Done`** edge leaves the iterator consumed (uninitialized);
    - a **`break`** targeting the loop SHALL **consume the iterator first** (a
      synthesized sink-move `{ let __sink = it; break; }`), since a naked `break`
      would otherwise carry it initialized into a point the `Done` edge reaches
      uninitialized — a join disagreement that is exactly **E0309**;
    - **`return`/`?`** edges leave the function, never reaching post-loop; the
      iterator is a live local there and drops at its ordinary per-path drop point.

    This is **zero new rules** — the sink-move is an ordinary scoped move and
    E0309 is the existing check (design 0009 §4.2).

4.3 **Borrow-copy (`Indexed`) needs no exit-edge rewrite.** The loop holds only a
    **loop-local `read` borrow** of the collection and a `usize` cursor (both
    drop-inert / trivially dropped), so no owned iterator state crosses a `break`;
    the E0309 concern is specific to `Iter`'s owned iterator (design 0009 §4.2).

4.4 **The loan story — inherited, not added.** Iterator invalidation is prevented
    by the **existing** loan machinery (chapter 04), one argument per protocol:
    - **Consuming: by the move.** `for x in coll` **moves** `coll`, so any mention
      of `coll` in the body is a use-after-move error; the collection is gone,
      invalidation is structurally impossible.
    - **Borrow-copy: by the XOR loan.** `read coll` is a **loop-local borrow whose
      live range spans the loop** (chapter 04 §5, NLL-lite), so by XOR (chapter 04
      §4) a `write` of `coll` inside the body conflicts with the live `read` loan
      and is rejected — iterator invalidation caught at compile time. Its region is
      the loop scope, spelled by the compact default (`read self` is `at`'s sole
      borrow-in, chapter 10 §3.7); none is storable, which is why the borrow lives
      on the loop's stack, not in a field (design 0009 §4.3).

    The design adds **no aliasing rule**; novelty is confined to the desugar
    target, the safety is old (design 0009 §4.3).

---

## 5. Effects — the two protocols on different tiers

5.1 The `for` **inherits its step method's effect marker** (chapter 10 §3.6): an
    allocating `for` in an unmarked function is a definition-site effect error,
    identical to a bare method call. There is **no iteration-specific effect rule
    and no transformed calling convention** — the `for` colors nothing (P10;
    design 0009 §3.3).

5.2 **`Iter::next` is `alloc`-marked, uniformly and unconditionally** (chapter 10
    §4.3's exact-marker rule): the headline consuming impl (`List::next`) must
    `unbox` its tail, which allocates, and a non-`alloc` interface method may
    allocate in no impl (chapter 10 §3.6); effect polymorphism that would let one
    impl escape it is refused (OBL-GENERIC-EFFECT). So **every** consuming `for` is
    `alloc`, even over a `copy` element — consuming iteration is **never** the
    no-alloc ground floor (design 0009 §3.1, §3.3).

5.3 **`Indexed::at` carries no effect marker (non-`alloc`)**: copying a `copy`
    `Item` out of a `read self` allocates nothing, so `for x in read coll` over an
    `Indexed` collection is non-`alloc` and interrupt-callable — the **ground-floor
    iteration protocol** (design 0009 §3.2, §3.3).

5.4 The effect tier is a **second, independent** reason the protocol count is two
    (the first is operand mode, §2): a single unified interface would carry one
    uniform marker and could span both tiers only via the refused effect
    polymorphism (design 0009 §3.3).

---

## 6. The named cut — SKELETON pointer

6.1 **Borrowed-item iteration** (`for x in read coll` yielding `read Item`),
    **mutating iteration** (`for x in write coll` yielding `write Item`), and
    **non-consuming pointer-chain iteration** (a `List` walk by shared borrow) are
    **deferred**: each needs either a region-parameterized associated type (a
    borrow ranging over a type parameter, refused by chapter 10 §2.3) or a
    swap/replace primitive (absent). `Indexed` copies instead (free for `copy`,
    unavailable otherwise); `List` iterates only by consumption. Tracked as
    **OBL-ITER-BORROW** (chapter 99; design 0009 §3.4).

6.2 **Higher-order code is capture-free this edition**: full capturing closures
    are refused (they would force region-typed closures or anonymous drop-glue
    aggregates); state travels as an **explicit context value** threaded by hand
    (a fn-pointer plus a `ctx` parameter), adding no language surface. Tracked as
    **OBL-GENERICS-CLOSURE** (chapter 99; design 0009 §5).
