# 03 — Types and Values

**Status: NORMATIVE-DRAFT.** Complete transcription of design `0001-memory-model`
§1 (value model), §5 (slices and arrays), and §8 (type-system minimum) into
normative clauses. Rationale is in design 0001; this chapter states rules only.

---

## 1. The type universe

1.1 The scalar types are the sized integers `i8 i16 i32 i64 isize` (signed) and
    `u8 u16 u32 u64 usize` (unsigned), together with `bool` and `unit`. The
    widths of `isize` and `usize` are **target-defined** and SHALL be queryable
    per target as constants (P5); no type has an undefined width.

1.2 The aggregate types are the nominal `struct` (named fields), the nominal
    tagged-sum `enum` (variants with zero or more positional payloads), and the
    fixed array `[N]T` for a compile-time-constant length `N`.

1.3 The reference types are the shared borrow `read T`, the exclusive borrow
    `write T`, the shared slice `[T]`, and the exclusive slice
    `write [T]`. Their access discipline is normative in chapter 04.

1.4 The pointer type is `rawptr T` (chapter 05). The heap-owning type is `Box T`,
    and `BoxResult T` is the compiler-known sum `enum { boxed(Box T), oom }`
    (chapter 08). Function-pointer types carry their parameter modes **and** their
    effect marker (chapter 08).

1.5 A conforming program in this edition **SHALL NOT** declare generic types or
    functions; the only parametric types are the compiler-known `[N]T`, `[T]`,
    `write [T]`, `rawptr T`, `Box T`, and `BoxResult T` (design 0001 §8.3;
    user-defined generics are a future obligation, chapter 99).

---

## 2. Values, ownership, and places

2.1 A **value** is a fully-initialized instance of a type occupying storage.

2.2 Every value SHALL have, at every program point, **exactly one owner**: a
    local binding, a struct field, an array-element slot, or the pointee slot of
    a `Box`. There is **no shared ownership** in safe code.

2.3 A **place** is an expression denoting storage that holds or will hold a
    value: a local, a field access `p.f`, an index `a[i]`, or a dereference
    `b.*`. Places are what a program borrows, moves out of, and assigns to.

---

## 3. Move and copy

3.1 Passing or assigning a value from a place SHALL do exactly one of two things,
    determined **solely by the value's type**: a **copy** or a **move**. There is
    no third option; in particular there is **no implicit deep copy**.

3.2 A **copy** leaves the source place valid and holding an equal value. A copy
    SHALL be a flat, dependency-free bit copy of known, bounded cost, and is
    permitted only for `copy` types (§4).

3.3 A **move** transfers ownership; the source place becomes **invalid**
    (uninitialized) and SHALL NOT be read until reassigned. A move SHALL run no
    user code and SHALL NOT deep-copy. Move is the default for every non-`copy`
    type.

3.4 Producing an independent duplicate of a non-`copy` value SHALL require the
    explicit `clone` operator (§5).

---

## 4. Copyability

4.1 `copy` is a structural, checker-computed property, requestable by the author
    only where it is cheap.

4.2 The following are `copy`: every sized integer, `bool`, `unit`, every
    `rawptr T`, every shared borrow (`read T`), and the shared slice `[T]`.

4.3 An exclusive borrow (`write T`), an exclusive slice (`write [T]`), a `Box T`, and every
    owning aggregate lacking the `copy` marker **SHALL move** (are not `copy`).

4.4 A fixed array `[N]T` is `copy` **iff** `T` is `copy`.

4.5 A `struct` or `enum` is `copy` **iff** the author writes the `copy` marker on
    its declaration **and** every field/variant payload type is `copy` **and** the
    type has no `drop` hook (§6). The marker is opt-in; copyability SHALL NOT be
    an implicit consequence of field shapes.

---

## 5. `clone`

5.1 `clone place` SHALL produce an owned, independent duplicate of the value at
    `place`, defined **structurally**: recursively clone each field or element.

5.2 `clone` of a `Box T` SHALL copy the stored `Alloc` handle, allocate a fresh
    box through that stored handle, and clone the pointee; it is therefore an
    `alloc`-effecting operation (chapter 08). Cloning any value bearing `Box`es
    SHALL allocate through **each `Box`'s own stored handle**.

5.3 A **user** type that contains a `rawptr` field is **not** cloneable; such a
    value SHALL be duplicated only by hand inside a valve (chapter 05). The
    compiler-known `Box T` is the sole exception (§5.2).

5.4 `clone` on a `copy`-typed place is permitted and equals a copy.

---

## 6. Destruction and drop order

6.1 A value is **destroyed** (dropped) when its owner ceases to exist.
    Destruction is fully deterministic and part of the source-declared semantics
    (P5).

6.2 A type MAY declare **at most one** `drop(write self) { ... }` hook. It SHALL
    run **before** the value's fields are destroyed, and SHALL run **exactly
    once** per live value.

6.3 A `drop` hook body is **ordinary checked code**, analyzed as a synthetic
    `fn drop(self: write StructT) -> unit`: every rule of this specification
    (chapters 03–08) holds inside it. The hook SHALL NOT move the value out of
    `self`, nor move any field out of it via a deref-path move.

6.4 If a `drop` hook body is `alloc`-effecting, the type is **alloc-on-drop**:
    every scheduled drop of it propagates the `alloc` effect to the enclosing
    function (chapter 08).

6.5 After the hook (or immediately, if there is none), fields SHALL be destroyed
    in **reverse field-declaration order**; array elements from **highest index to
    lowest**; enum payloads by the same reverse rule.

6.6 A local binding SHALL be destroyed at the **end of its enclosing block**;
    locals in one block SHALL be destroyed in **reverse order of first
    initialization** (LIFO). A temporary SHALL be destroyed at the end of the full
    statement that created it.

6.7 A place that has been **moved out of SHALL NOT be destroyed** at scope end.
    Move state is a static per-point property (§7); no runtime drop flag is
    required.

6.8 Reassigning a place SHALL first destroy the value it currently holds (if
    any), then store the new one.

---

## 7. Partial moves and static move/init state

7.1 Moving a **single field** out of a struct is permitted **only when the type
    has no `drop` hook**. The remaining fields stay owned and are destroyed
    individually at scope end; the moved-out field is not.

7.2 Moving one **element** out of an array by a constant index is permitted
    **only for `copy`** element types; a non-`copy` element move is ill-formed
    (the place model tracks a single index at whole-array granularity).

7.3 **No conditional move divergence.** At every control-flow join, each place
    SHALL have the **same move state** on all incoming paths.

7.4 **No partial move out of a `drop`-hooked type.**

7.5 **No conditional initialization divergence at a drop point.** At any drop
    point of a **needs-drop** place — a place whose type has a `drop` hook, or
    transitively contains a drop-hooked type or a `Box` — the place's
    initialization state SHALL be path-independent (initialized on all incoming
    paths, or uninitialized/moved on all). The drop points are: scope exit
    (including via `return`/`break`/`continue` or block end), whole-binding
    reassignment, and passing the place as an `out` argument. Drop-inert types
    (scalars, `copy` aggregates, `rawptr`, and aggregates of them) are **exempt**.

7.6 By §7.3–§7.5, the drop schedule of every needs-drop value SHALL be a static
    fact; no read of any value SHALL ever observe uninitialized or moved-from
    storage (NN#5; chapter 06 §5).

---

## 8. Arrays, slices, and text

8.1 `[N]T` is a fixed-size, contiguous, owned block of `N` values of `T`. It
    moves, or copies iff `T` is `copy` (§4.4). `a[i]` is a place. Indexing SHALL
    be bounds-checked (chapter 06).

8.2 A `[T]` is a shared borrow of a contiguous run of `T`; a `write [T]` is
    an exclusive borrow of one. Slices obey the borrow rules of chapter 04. They
    SHALL NOT be struct fields (chapter 04 §8). Slice range operations
    (`subslice`) SHALL be bounds-checked (chapter 06).

8.3 There is **no string type** in this edition. Text is `[u8]` (borrowed) or
    `[N]u8` / `Box`-of-bytes (owned). String and byte literals produce `[u8]`
    viewing read-only static storage. (The text-type budget under P3 is a future
    design obligation, chapter 99.)

---

## 9. Integer conversion

9.1 There are **no implicit conversions** (P2). Converting between integer types
    SHALL be written explicitly (`conv T (e)` in the prototype fixture).

9.2 A **widening** conversion is always value-preserving.

9.3 A **narrowing or sign-changing** conversion SHALL, under the default
    regime, **fault on value loss** (chapter 06); inside a `wrapping` region it
    SHALL truncate (two's-complement); inside a `saturating` region it SHALL
    clamp to the nearest representable value. These are the same three regimes,
    selected the same textual way, as arithmetic (chapter 06 §2).

9.4 Changing a **pointer's** element type is not a `conv`; it is an unsafe
    operation (chapter 05).

---

## 10. Statics

10.1 A top-level `static NAME: Type = Expr;` declares a program-lifetime value.

10.2 Statics are **immutable**: assigning to a static, taking an exclusive
     (`write`) borrow of one, or passing one as an `out` argument SHALL be
     ill-formed. Reading and shared (`read`) borrowing are permitted. (A mutable
     global is a future design question — concurrency, P9 — not part of this
     edition.)
