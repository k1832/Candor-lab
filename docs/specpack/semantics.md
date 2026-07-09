# Candor semantics — rules a generator must not violate

Source: spec 03–08 + designs 0001/0004/0005/0007/0009 + MEMORY.md friction
records. Ordered by **observed agent friction first** (the rules ports/fixtures
actually tripped), then by topic. Codes in parentheses point to `diagnostics.md`.

## Top-5 most-tripped rules (fix these before anything else)

1. **Reborrow discipline: implicit at calls, explicit `.*` on write-paths.**
   Passing a borrow you already hold to a `read`/`write` parameter is a *reborrow*,
   written **bare** (`f(b)`, not `f(write b.*)`). But a bare exclusive-borrow
   parameter used as a value is a **move**: `advance(pos)` where `pos: write usize`
   MOVES `pos` (use-after-move next line). To keep it, reborrow: `advance(write
   pos.*)` for write, `peek(read pos.*)` for read. Every **write** through a
   borrow needs an explicit `.*` on the path (`a.*.total = ...`, E0713); **reads**
   auto-deref (`a.total`). Walking a `Box` chain reads through both peels:
   `read (tail.*.*)`. (spec 04 §2–3, 0005; MEMORY: "§11's own examples leaned on
   an implicit reborrow the model forbids".)

2. **A consuming `match` over a `Box`/drop-bearing enum must `return` from EVERY
   arm (E0302).** Arms that move payloads out and arms that don't disagree on
   partial-move state at the match join — even in a `unit` function with nothing
   used afterward. Put `return ...;` at the end of every arm. (spec 03 §7.3;
   MEMORY: parser serializer/span-walker.)

3. **E0309: init state must be path-independent at every drop point.** For a
   needs-drop place (has a `drop` hook, or transitively contains a drop-hooked
   type or a `Box`), it must be initialized on ALL paths reaching a drop point, or
   uninitialized/moved on all. Drop points: scope exit (incl. `return`/`break`/
   `continue`/block end), whole-binding reassignment, passing as an `out` arg. Any
   new control-flow construct that exits a function must reach the CFG as a genuine
   return. (spec 03 §7.5; MEMORY: "new-construct soundness gaps cluster at exit
   points".)

4. **`alloc` propagates through the FREE side too — letting a `Box` die is
   allocation.** A non-`alloc` function that receives a `Box`-bearing value by
   value and lets it drop is rejected (E0401): the scope-exit drop frees the box,
   which is allocator work. A function that instead moves it out (returns/forwards)
   is not `alloc`. A `drop` hook whose body allocates makes the type *alloc-on-drop*
   — every drop of it propagates `alloc`. (spec 08 §4; corpus `list.cnr`.)

5. **Move-then-reassign and use-after-move.** After a value is moved (into a call,
   another binding, a return), the source place is uninitialized and reading it is
   E0301. Building a chain with `l = f(.., l)` (RHS consumes `l`, then rebinds it)
   is a move-tracking hazard; prefer distinct bindings `l1 = f(.., l0)`. A move
   binding in a `match` partial-moves the scrutinee. (spec 03 §3.3; MEMORY.)

## Move & copy (spec 03)
- Every pass/assign is exactly a **copy** (copy types: flat bit-copy, source stays
  valid) or a **move** (source becomes uninitialized, runs no user code). No third
  option, no implicit deep copy. Move is the default for every non-`copy` type.
- Duplicate a non-`copy` value only with `clone place` (structural; cloning a
  `Box` allocates through its stored handle → `alloc`). A user type with a `rawptr`
  field is NOT cloneable.
- **Partial move** of one struct field is allowed ONLY if the type has no `drop`
  hook (E0303). Array-element move only for `copy` elements. No conditional move
  divergence — move state must agree at every join (E0302). Moving a non-`copy`
  value out through a path containing a `deref`/index is ill-formed (E0310); the
  defined `Box`-pointee extraction is `unbox`.

## Borrows: XOR & reborrows (spec 04)
- **XOR aliasing**: at every point, for every place, live loans are EITHER any
  number of shared borrows XOR exactly one exclusive — never both, never two
  exclusives (E0801). A move/write/reassign conflicts with ANY live loan (E0802/
  E0803). Reading conflicts with a live exclusive loan (E0804).
- Place granularity: `p` and `p.f` overlap; disjoint fields `p.f`/`p.g` don't;
  any `a[i]` covers the whole array `a`.
- **Write through a shared borrow is illegal** (E0809): a deref on a write path
  must peel an exclusive borrow (or a `Box`) at every step.
- Reborrow lifetimes: an exclusive reborrow suspends the parent for its live
  range; a shared reborrow freezes the parent to shared. Live ranges are
  body-local (NLL-lite), not lexical. **No two-phase borrows** (E0805): a call
  reserving `write v` while `read v[0]` computes an argument is rejected.
- **Borrows are never a storage class** (E0201): no borrow/slice field. Stored
  inter-object links are `Box`, a handle/index integer, or a `rawptr` (the valve).
- Region provenance crosses the signature explicitly (NN#17). Compact default:
  exactly one borrow-in + a borrow return → the return derives from it, no region
  variable. Two+ borrow params returning a borrow → region variables MANDATORY
  (E0808/E0807). A borrow of a local/`take` param can't be returned (E0806).

## `out` parameters (spec 04 §6.7)
- Callee SHALL assign the slot on EVERY normal-return path (E0305) and SHALL NOT
  read it before first assignment (E0306). Caller keeps ownership. A pre-init `out`
  slot is dropped at the call site before the call. If the callee faults before
  assigning, the slot stays uninitialized and isn't dropped. Arg carries the
  mandatory `out` marker (E0307/E0308) and must be a place (E0705); `out` produces
  an exclusive loan conflicting with other overlapping args (E0805).

## Valve rules: unsafe & raw pointers (spec 05)
- Raw-pointer create/offset/cast/deref ops require an `unsafe` block (E0501).
- `unsafe` REQUIRES a non-empty **justification string** (E0502/P0003): `unsafe
  "why this is sound" { ... }`. Presence is enforced, not truth; every region is
  enumerable.
- `unsafe` grants EXACTLY ONE new power (raw-pointer ops). Move/borrow/overflow/
  bounds checking on safe values still apply inside it.
- Holding/copying/comparing a `rawptr` is safe (it is `copy`); a `rawptr` field is
  inert in safe code. `field_ptr(p, f)` and `is_null`/`offsetof`/`sizeof` are safe.
- Inside `unsafe` the author owns: pointer validity/init, not creating two owners
  via `ptr_read`, and handle/pointee liveness for the life of every copy.

## Effects: the `alloc` partition (spec 08)
- **Ground floor**: a non-`alloc` function may not call an `alloc` function
  (E0401); an `alloc` function may call anything. One-way partition; allocation-free
  code is callable everywhere (interrupt/no_std). The capability travels as a VALUE
  (the `Alloc` handle), never a function-type transformation.
- `alloc`-triggering: `box`; `clone` of a `Box`-bearing value; a call to an
  `alloc` fn; an indirect call through an `alloc`-typed fn-pointer; `unbox`; and
  ANY scheduled drop of a `Box`-bearing or alloc-on-drop value (see top-5 #4).
- Effects are **upper bounds**: overstating (`alloc` that never allocates) is legal
  conservatism; understating is forbidden. Assigning an `alloc` fn to a non-`alloc`
  fn-pointer slot is ill-formed (E0402); the reverse is legal. An indirect call
  takes its effect from the POINTER's type — no vtable special case.

## Contracts (spec 07)
- `requires`/`ensures`/`assert` are executable, analyzable, and **read-only**
  (E0708 family): inside a clause you may NOT move a non-`copy` value, take a
  `write` borrow, pass `out`, or call anything taking a non-copy `take`/`write`/
  `out` arg. Reads, `read` borrows, copy-`take` calls are fine. `ensures` may name
  `result` (only there, E0702).
- `?` may NOT appear inside a contract clause (E0708) — it is control flow, not a
  read.
- The **optimizer NEVER assumes a contract holds** (all levels, incl.
  `assumed-proven`): a wrong contract yields wrong VALUES, never UB. This edition
  implements only the `enforced` level (violation = fault).

## Generics (designs 0007/0009)
- Bounds `[T: I + copy]`; unbounded `T` is fully **opaque**: movable, droppable,
  borrowable, storable — and NOTHING else. No field access on `T`, no method not
  declared by a bound, no `conv T`/arithmetic on `T` (no `Num` bound exists), no
  `sizeof(T)` at the definition site.
- Checked once at the definition site (NN#10): a generic that compiles cannot
  produce a body-internal type error at instantiation; only bound conformance is
  checked at the call (E1006–E1008). Monomorphization is the only strategy;
  polymorphic recursion with a growing argument is a def-site error (E1020).
- **A generic impl's type parameters must appear in the target type** (E1016):
  `impl[T] I for List[T]` OK; a `T` used only in method bodies is rejected.
- An opaque `T` is assumed needs-drop, so a generic that owns-and-drops a bare `T`
  is `alloc`-marked even at drop-inert instantiations (the opaque-drop tax); `T:
  copy` relaxes this (copy ⟹ drop-inert). Interface methods carry effect markers;
  a bounded method call inherits the declared effect. Impl method signatures must
  match the interface exactly (self mode, arity, param modes/types, return, effect
  marker — E1021–E1026); extra impl methods are rejected (E1014).
- No type parameter over a borrow type (E1006). No associated-type bounds/GATs/
  defaults, no specialization/blanket impls, no HKT, no super-interfaces.

## The `?` operator (spec 02 §6.5, 0007 §7.1)
- `expr?` on a **result-shaped** enum (exactly one `ok`-marked variant): unwrap the
  `ok` payload, else `return` the value unchanged. Well-formed only where the
  enclosing function returns a compatible result type (E0711/E0712). Same-type `?`
  needs no conversion; cross-type `?` desugars to `E2::from(e1)` via an `impl
  From[E1] for E2` and inherits `from`'s effect marker.
- **Impl note**: `?` (same-type and cross-type via `From`) works in single-file
  AND multi-module programs (fixed 2026-07-08; the corelib_question fixture is the
  regression test).
- `static` bindings are IMMUTABLE (spec 03 §10): assigning, `write`-borrowing, or
  passing one as `out` is ill-formed (E0311). Reads and `read` borrows are fine.
- Faults (overflow, div-by-zero, bounds, conv-loss, failed `assert`/`requires`/
  `ensures`, `panic`) are bugs, not values — they route through the root fault
  policy and truncate execution. Expected failure is an **error value** in the
  return type (a result-shaped enum), never a fault. Default arithmetic is checked
  (overflow faults); `wrapping {}`/`saturating {}` are scoped, textual-only regimes
  (no dynamic scope across calls). Unchecked overflow exists only inside `unsafe`.
- A non-unit function must return on all paths (E0810); indexing is always
  bounds-checked. Uninitialized reads are impossible (E0304).

- **`out T` is an owned place, not a borrow**: an `out` parameter binds a caller-owned, initially-uninitialized slot; assign it directly (`p = v`), never `p.*` — `out T` is not a borrow type and takes no deref. The callee must assign on every normal-return path before returning.
