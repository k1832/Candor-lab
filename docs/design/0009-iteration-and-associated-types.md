# 0009 — Iteration and associated types

**Status:** draft
**Date:** 2026-07-08
**Prototype:** implemented (generics stage 3 — §9's stages 1–3 shipped: `for`/`type`/`I::Item` parse, the associated-type member + projection, `Iter`/`Indexed` + the two desugars, the corelib port, and the E1002 completion). Stage plan §9. Builds on 0007's stage-1+2 core
(interfaces, impls, coherence, monomorphization, generic drop hooks) and on the
corelib seed (`compiler/tests/fixtures/corelib/`), whose measured friction is
this round's mandate.
**Philosophy hooks:** **P11** (definition-site checking; a generic that compiles
cannot fail at instantiation — NN#10 — extended intact to associated types),
**P6** (small core; *the budget is fixed — adding requires removing*; this round
adds without removing (§1.3) — a named P6 tension the deciding authority accepts on the seed's measured pressure, keeping every adjacent refusal),
**P4/P9** (no hidden allocation or control flow — the `for` desugar is
statement-level, greppable, visibly costed, §4), **P10** (iteration carries the
effect marker like any method and adds no coloring, §3.3), **P3** (one loop
family; iteration reuses `Opt` and the `loop` desugar target), **NN#13** (`for` /
`type` / `I::Item` tokenize and parse without a symbol table, §4.4), **P12/Bet 5**
(value-first: by-value/by-copy iteration first; capturing closures refused this
edition, §5). Subordinate to `LANG_PHYLOSOPHY.md` and to designs 0001 (the memory
model respected once, for every instantiation) and 0007 (the interface system this
extends). Where they conflict, the higher document wins and this one changes.

**What this is.** The round OBL-GENERICS-ITER reserved: the machinery that makes
the seed's unwritable APIs writable — `for`-iteration and `Opt::map` — at the
least P6 budget the now-concrete pressure demands. It adopts a minimal
associated-type form (§2), two by-value iteration protocols with their `for`
desugar (§3–4), and a capture-free ruling on higher-order code (§5). It discharges
**OBL-GENERICS-ITER**.

**What this is not.** Not closures (refused with a staged plan, §5). Not
borrowed-item or mutating iteration (deferred — 0007 §3.5's collision is real and
named, §3.4). Not a change to the memory model, effect set, coherence, or borrow
discipline — all fixed by 0001/0007 and respected once.

**Revision history.** 2026-07-08 — revised per adversarial review #1 of design 0009
(`docs/reviews/2026-07-08-design-0009-review-1.md`): `Iter::next` alloc-marked and the
ground-floor claim moved to `Indexed` (§3.1/§3.3/§8); the break-edge E0309 sink-move in
the consuming desugar (§4.2/§4.3); the for-operand parses as `ExprNoStruct` (§4.4); §1.3
budget rewritten honestly (addition without removal, the reserve fiction deleted); `for`/`in`
made contextual keywords with the migration residual walked (§4.4/§9); §5.4 cross-references
qualified to 0007; and the `Indexed` per-step guard+bounds cost named (§7).

---

## 1. The pressure cases, honestly

0007 §1.1 refused associated types "argued from the one case that wants them" and
set the reopen condition: *the pressure cases must exist.* The seed produced them.
Refusal-first, per P6: this section states the friction and the smallest machinery
that clears it — nothing more is authorized below.

### 1.1 `for x in coll {}` — the associated-type case

The seed walks collections by hand: `main.cnr` reads an `Arena` with a manual
index ladder (`get(read ar,0)+get(read ar,1)+…`) and computes a `List` length by
open-coded recursion. There is no `for` (0006 §7, 0007 §7.2 deferred it) because
iteration needs a protocol whose *yielded type varies per implementer* — an
interface with a **type member**.

**Why a type *parameter* will not serve.** `interface Iter[Item] { fn next(write
self) -> Opt[Item] }` fails on coherence: 0007 §2.3 keys uniqueness on the
*instantiated* interface, so one type may `impl Iter[u8]` *and* `impl Iter[u32]` —
two legal keys — and `it.next()` then has no single return type. Iteration demands
a **functional dependency**: the item type is *determined by* the iterator type,
one per implementer. That is exactly an associated type and not a type parameter.
This is the basket-grade case 0007 §1.1 required.

### 1.2 `Opt::map` — the higher-order case, dissected

`core/opt.cnr` records `map[T,U](Opt[T], fn(T)->U) -> Opt[U]` as the corelib's
headline friction. It is **two** bites, only one about higher-order values:

- **Inference (E1002).** The prototype cannot infer `U` from a fn-pointer
  argument's *return* type, and there is no turbofish (0007 §6.2). But `U` appears
  in a value parameter's type — the return of `f: fn(T)->U` — so inferring it is
  ordinary body-local unification (0007 §2.2, as `push(list,x)` infers `T`).
  **E1002 is a prototype completeness gap, not a language refusal** (§5.3); closing
  it needs *no new surface*.
- **The drop tax.** `map` **forwards** its `T` into `f(v)` — moves it out, never
  drops it — and 0007 §3.4 exempts "moves `T` out … without dropping it," so a
  forwarding `map` is *not* `alloc`-taxed; a `[T: copy, U: copy]` `map` is
  drop-inert regardless (0007 §3.3), like the seed's `unwrap_or[T: copy]`.

So `map` needs **neither closures nor new machinery** — only completed inference.
What a bare `fn(T)->U` cannot express is a transform that *carries state* (a
running sum, a base pointer, an allocator). That is the closure question, answered
capture-free (§5): state travels as an **explicit context value** — P2's
allocator-as-value idiom (0007 §8.3) generalized, adding no surface.

### 1.3 The budget: an addition without a removal, named as such

P6: *when you add, remove.* This round does not remove — it adds, and says so.
There is no pre-authorized "reserve" to draw down: 0007 §1.1 *refused* associated
types and *recorded a reopen condition* (the pressure cases must exist); a recorded
reopen is a debt to be argued, not a paid-up credit. The seed supplied the pressure
(§1.1–1.2), and this round spends budget against it. Counting the additions honestly:

- **the associated type** (`type Item`, §2) — one new type-member construct;
- **the `for` statement** (§4) — one new surface form;
- **`for` / `in`** (§4.4) — two contextual keywords;
- **`Iter` and `Indexed`** (§3) — two library interfaces;
- **`IterStep`** (§3.1) — one library enum.

That is a real P6 cost, not a wash. The deciding authority accepts it as a **named
P6 tension**: the measured reading-friction of the seed's hand-coded ladders and
recursion (§1.1) is judged to outweigh the surface added, and the tension is recorded
here rather than dissolved by a fiction. The discipline is honored where it still can
be — by *keeping every adjacent refusal* (§2.3): bounds on associated types, GATs,
defaults, and more-than-one member per interface all stay refused.

---

## 2. Associated types, minimal form

### 2.1 The construct

An `interface` declares **at most one associated type**, introduced by `type`:

```
interface Iter {
    type Item;                              // the one member — no bound, no default
    fn next(take self) -> IterStep[Item, Self];
}
```

`Item` is a type member the *impl* fixes and the interface's method signatures may
mention. It is not a type parameter (§1.1): it is determined by the impl, not
supplied by the user.

### 2.2 Projection `I::Item`, and why it needs no qualifier

Inside `fn f[T: Iter](it: T)`, the element type is **`T::Item`** — a path-form
projection (`::` is the path separator, 0008 §3; NN#13-clean, §4.4). It denotes an
**opaque projected type**: like an unbounded parameter (0007 §3) it is movable,
droppable, borrowable, storable, and *nothing else*. Definition-site checking
treats `T::Item` exactly as an opaque `T` (0007 §2.1): the body is checked once,
against this projected-opaque; instantiation substitutes the impl's concrete
choice. NN#10 holds unchanged.

**No `<T as Iter>::Item` disambiguation is ever needed** — a dividend of 0007's
no-overlap rule. Because at most one `impl Iter for C` exists (0007 §2.3), `C::Item`
is single-valued: no second impl to disambiguate against. Rust's heaviest
associated-type syntax exists to name *which* impl; 0007's coherence already
answered that, so the projection stays a bare path.

### 2.3 What stays refused (the refusals kept intact)

Each is a debt, reopenable by amendment when a program calls it in.

- **No bound on an associated type** (`type Item: copy`): a `where`-clause over a
  projection — a second bound-spelling (P3) and a constraint graph the lookup
  would solve. A generic needing a copyable item pushes the capability to the impl.
- **No generic associated types** (`type Item[X]`): higher-kinded machinery
  (0007 §1.1's HKT refusal renamed). No basket case.
- **No associated-type default** (`type Item = u8` in the *interface*): private
  codegen in a public signature — comptime's job, not the bound system's.
- **No more than one member per interface**: two reintroduce the `<T as I>::A`
  naming pressure. One covers iteration; the two iteration interfaces (§3) each
  spend exactly one. A second member demands its own basket case.

**Coherence is unchanged.** The member is fixed by the impl and keyed by its
`(I, C)` pair — no lattice, no negative reasoning, no specialization order (0007
§2.3). Orphan placement and the module-DAG lookup stand as written.

---

## 3. The two iteration protocols

Iteration hits two 0001/0007 walls at once — the collision, not taste, sets the
protocol count:

1. **No borrow-typed fields** (0001 §3.4; 0007 §3.5): an iterator may not *store* a
   borrow of its collection, so Rust's `struct Iter<'a>{coll:&'a C}` is unwritable
   — it would force a region parameter onto the iterator type.
2. **No swap/replace** (OBL-GENERICS-ITER evidence; E0303/E0310): an iterator
   cannot move an owned element *out through a `write self` borrow* and overwrite
   `self` with the successor — so Rust's `fn next(&mut self)` yielding owned items
   is also unwritable.

These two together are *why* 0007 deferred iteration. This design routes around
both rather than adding swap/replace or region-typed iterators (both larger than
the case warrants). Two small interfaces result, each one associated type,
**selected by the operand's borrow mode** (lexically visible, NN#13):

### 3.1 Consuming — `Iter` (operand moved, always `alloc`)

```
enum IterStep[T, S] { ok More(T, S), Done }      // library generic; the seed's `Popped` shape

interface Iter {
    type Item;
    fn next(take self) alloc -> IterStep[Item, Self];  // consume self; return item + successor
}
```

`next` **consumes** `self` and returns the `Item` alongside the **successor
iterator** (`Self`) — a functional step dodging swap/replace exactly as the seed's
proven `pop_front(l: List[T]) -> Popped[T]` does (`std/list.cnr`). No borrow is
stored, no element moves through a borrow. `Self` names the impl target (0007 §7.1
vocabulary). `List[T]` impls this by generalizing `pop_front`; `for x in list`
moves `list`.

**`next` is `alloc`-marked, period (0007's uniform-marker rule, 0007 §4.1).** The
headline consuming impl — `List::next` — must `unbox` its tail (§8), which allocates;
a non-`alloc` interface method may allocate in *no* impl (0007 §4.1), so admitting
`List` forces the marker onto the interface, and 0007 refused the effect polymorphism
(a per-impl `alloc` variable, OBL-GENERIC-EFFECT) that would let one impl escape it.
The marker is therefore uniform and unconditional: **every** consuming `for` is
`alloc`, even over a `copy` element whose successor allocates nothing. Consuming
iteration is thus *never* the no-alloc ground floor — that role is `Indexed`'s (§3.2,
§3.3).

### 3.2 Borrow-copy — `Indexed` (operand borrowed, items copied)

```
interface Indexed {
    type Item;
    fn at(read self, i: usize) -> Opt[Item];     // Some(copy) in range, None past the end
}
```

`at` takes `read self` and a **plain `usize` cursor the loop owns** — not an
associated state type, not stored in any iterator; it lives on the desugared
loop's stack (§4.2). The collection is held only by a **loop-local `read` borrow**,
never a field, so 0001 §3.4 holds. `Item` is copied out (the impl requires
`Item: copy`, as `Arena`'s `get[T: copy]` already does). `for x in read arena`
borrows and yields copies without consuming. **`at` carries no effect marker
(non-`alloc`):** copying a `copy` `Item` out of a `read self` allocates nothing, so
`for x in read coll` over an `Indexed` collection is non-`alloc` and interrupt-callable
— the ground-floor iteration protocol (§3.3). This form carries the loan-soundness
story (§4.3).

### 3.3 Effects: no coloring, and the effect tier splits the protocols (P10)

`Iter::next` and `Indexed::at` carry the effect marker like any method (0007 §4.1):
the `for` inherits its step's marker, so an allocating `for` in an unmarked function
is a definition-site effect error, identical to a bare method call. There is no
iteration-specific effect rule and no transformed calling convention: the `for` is
sugar over `loop` + method calls, so it colors nothing (P10, NN#9).

But the two protocols sit on **different effect tiers**, and this is fixed, not
incidental: `Iter::next` is `alloc` uniformly (§3.1 — the consuming case is never
ground-floor), while `Indexed::at` is non-`alloc` uniformly (§3.2 — a copy-out over a
`read` borrow allocates nothing). The **ground-floor, interrupt-callable iteration
protocol is `Indexed`** (copy items, non-`alloc`); consuming iteration is not it.

**The effect axis strengthens the two-protocol argument (P3).** §3's split was argued
from operand mode (moved vs. borrowed); the effect tier is a *second, independent*
reason the count is two. A single unified iteration interface would carry **one**
uniform effect marker across every impl (0007 §4.1): mark it `alloc` and every
ground-floor copy walk (`Arena`) is taxed off the no-alloc floor; mark it non-`alloc`
and the allocating consuming impl (`List`, which `unbox`es) is inexpressible (0007 §4.1
bars allocation under an unmarked method). The *only* way one protocol could span both
tiers is a per-impl effect variable — **effect polymorphism, which 0007 refused**
(0007 §1.1, OBL-GENERIC-EFFECT). So the protocols differ not merely by operand mode but
by effect tier, and unifying them would require exactly the machinery 0007 declined.
The two-protocol shape is forced twice over; this is not a patch on the P3 argument but
a strengthening of it.

### 3.4 The named cut

- **Borrowed-item iteration** — `for x in read coll` yielding `read Item` (a
  reference, not a copy) — is **deferred**. `Item` ranging over a borrow is 0007
  §3.5's refusal; yielding `read Item` needs a region-parameterized associated
  type. `Indexed` copies instead (free for `copy`, unavailable otherwise). The
  exact collision 0007 §7.2 predicted. **OBL-ITER-BORROW.**
- **Mutating iteration** — `for x in write coll` yielding `write Item` — deferred
  for the same reason plus swap/replace's absence. **OBL-ITER-BORROW.**
- **`List` by shared borrow** — a non-consuming chain walk needs a *cursor that is
  a borrow of the chain* (a borrow field, refused, 0001 §3.4). `List` iterates only
  by consumption (§3.1); the seed's `length` stays hand-written. **OBL-ITER-BORROW.**

---

## 4. The `for` loop

### 4.1 Canonical form

```
for PATTERN in OPERAND { STATEMENTS }
```

The **operand's borrow mode selects the protocol**, syntax-directed and greppable:

- `for x in coll { … }` — `coll` owned; requires `typeof(coll): Iter`; **moves** it.
- `for x in read coll { … }` — requires `typeof(coll): Indexed`; **borrows** `coll`
  `read` for the loop's extent; `x` is a copied `Item`.

`x` is any irrefutable pattern (0002); `break`/`continue` work as in `loop`/`while`
(P3 — same vocabulary).

**Why `for` earns a second loop member (P3/P13).** `loop`/`while` express
condition-driven repetition; `for` expresses protocol-driven traversal, and
open-coding it (the seed's ladder and hand-recursion) costs the reader the tokens
P13 optimizes and re-derives termination each time. `for x in read arena` states
"traverse every element, once, in order" in four tokens the reviewer need not
verify; the manual ladder needs a paragraph they must.

### 4.2 Desugaring — statement-level, visibly costed (P4/P9)

The desugar is documented, statement-level, and allocation-free in itself (any
allocation is the impl's marked, inherited `next`/`at`, §3.3). Consuming:

```
// for x in coll { BODY }  ==>
let mut __it = coll;                     // coll MOVED; __it owns all iterator state
loop {
    match __it.next() {                  // next() takes self: CONSUMES __it every turn
        IterStep::More(x, __rest) => { __it = __rest; BODY* }
        IterStep::Done => { break; }     // __it already moved out by next(): UNINITIALIZED here
    }
}
// after the loop, __it is uninitialized on EVERY path (the exit-edge accounting below)
```

**Why `next` consuming `__it` is load-bearing — the exit-edge accounting (E0309).**
`next(take self)` moves `__it` out on every turn; `More` restores it (`__it = __rest`),
`Done` does not. So on the `Done` edge — the only edge that falls through to the
post-loop point without a `break` — `__it` is uninitialized. 0001's drop scheduling
requires init state to be **path-independent** at any drop point (E0309), so *every*
edge reaching post-loop must agree `__it` is uninitialized. The desugar makes each
exit edge consume `__it`:

- **`Done`** consumes `__it` via `next` (the move is not restored) — uninitialized.
- **`break`** edges do *not* naturally consume `__it` (a `break` inside `BODY` fires
  just after `__it = __rest`, when `__it` is live), so `BODY*` is `BODY` with every
  `break` targeting this loop rewritten to a **synthesized sink-move** that consumes
  `__it` first:

  ```
  { let __sink = __it; break; }          // move __it into __sink, which drops as break unwinds
  ```

- **`return` / `?`** edges leave the function, not the loop; `__it` is a live local
  there and is dropped at its ordinary **per-path drop point** by 0001's drop schedule
  — it never reaches post-loop, so it needs no sink-move.

Post-loop, `__it` is therefore uniformly uninitialized on `Done` and on every rewritten
`break`. Path-independent, **zero new rules** — the sink-move is an ordinary move into
a scoped binding, and E0309 is the *existing* check.

**The non-example (the reviewer's `has_positive`).** A `for` whose body breaks early is
exactly the collision. Source:

```
fn has_positive(list: List[i64]) alloc -> bool {   // alloc: List::next unboxes (§3.1)
    let mut found: bool = false;
    for x in list {
        if x > 0 { found = true; break; }
    }
    return found;
}
```

Desugared with the E0309-satisfying sink-move:

```
fn has_positive(list: List[i64]) alloc -> bool {
    let mut found: bool = false;
    let mut __it = list;
    loop {
        match __it.next() {
            IterStep::More(x, __rest) => {
                __it = __rest;
                if x > 0 { found = true; { let __sink = __it; break; } }  // sink-move, then break
            }
            IterStep::Done => { break; }                                  // __it consumed by next()
        }
    }
    return found;                        // __it uninitialized on BOTH the Done and the break path
}
```

Without the sink-move the `break` edge would carry `__it` *initialized* (just set to
`__rest`) into a post-loop point the `Done` edge reaches *uninitialized* — the join
disagrees and 0001 reports **E0309**. The synthesized sink-move consumes `__it` on the
break edge, restoring path-independence.

Borrow-copy:

```
// for x in read coll { BODY }  ==>
let __c = read coll;                     // loop-local READ borrow, live across the whole loop
let mut __i: usize = 0;
loop {
    match __c.at(__i) {
        Opt::Some(x) => { __i = __i + 1u; BODY }
        Opt::None => { break; }
    }
}
```

The borrow-copy loop needs **no** exit-edge rewrite: `__c` is a `read` borrow (dropped
trivially) and `__i` is a `usize` (`copy`, drop-inert), so no owned iterator state
crosses the `break` — the E0309 concern is specific to `Iter`'s owned `__it`.

No hidden allocation, no hidden control flow: `loop`, `match`, the move or borrow, the
increment, and the sink-move are ordinary constructs the reader can cost (P4). Rendered
as source on request (P4/P6 transparency).

### 4.3 Loans and the iterator-invalidation soundness story

Iterator invalidation is prevented by the **existing loan machinery**, no new rule
— one argument per protocol:

- **Consuming: by the move.** `for x in coll` *moves* `coll` into `__it`, so `coll`
  is uninitialized after (0001 §1.6) and any mention of it in `BODY` is a
  use-after-move error. The collection is gone; invalidation is structurally
  impossible. `__it` itself is consumed on every exit edge (§4.2's sink-move
  accounting), so no live iterator survives to invalidate post-loop either.
- **Borrow-copy: by the XOR loan.** `__c = read coll` is a **loop-local borrow whose
  live range spans the loop** (0001 §2.3 NLL-lite: used by `at` every iteration).
  By XOR (0001 §2.2) a `write` of `coll` cannot coexist with that live `read` loan:

  ```
  for x in read arena {
      let ok: bool = push(write arena, x);   // E0303: `write arena` conflicts with the
      // …                                    //        live `read` loan held by the loop
  }
  ```

  Iterator invalidation caught at compile time by the loan discipline that already
  exists (0001 §2.2/§2.3), monomorphized over the concrete `Arena[T]` like any
  other loan check (0007 stage-2). `__c`'s region is the loop scope, spelled by
  0001 §3.3's compact default (`read self` is `at`'s sole borrow-in; 0007 §3.5 F9)
  — no region variable written, none *storable*, which is exactly why the borrow
  lives on the loop's stack and not in a field.

The design adds no aliasing rule; it inherits one. Novelty is confined to the
desugar target; the safety is old.

### 4.4 NN#13 walk

- **`for` / `in`** — **contextual keywords** (like `type`, below; consistent with the
  round's grammar discipline), not hard reserved words. `for PATTERN in OPERAND {`
  tokenizes without a symbol table: `for` in statement-leading position begins the loop
  and `in` separates pattern from operand; elsewhere both remain ordinary identifiers
  (the residual is walked in §9's migration note). No production competes. **The
  operand parses as `ExprNoStruct`** — the same restriction 0006 §0.7 places on the
  `if`/`while` condition, extended here to the `for` operand: a bare struct literal is
  excluded so the `{` that opens the loop body is never misread as the start of a
  struct value. The excluded case is a `Range`-style literal, which must be
  parenthesized:

  ```
  for x in Range { lo: 0, hi: n } { … }     // REJECTED: `{ lo: 0, … }` reads as the loop body
  for x in (Range { lo: 0, hi: n }) { … }   // OK: parenthesized, then the body `{`
  ```

  The parser need not know whether the operand is `Iter` or `Indexed` (a checker fact,
  like 0007's `Name[…]` arity).
- **`type Item;`** — inside `interface`/`impl` bodies only, `type` is a
  **contextual keyword** (declaration in an interface, assignment `type Item = u8;`
  in an impl). Outside, `type` is a plain identifier (no top-level alias exists), so
  nothing global is reserved and no program breaks; one-token lookahead within the
  body disambiguates, under the two-token ceiling (0002 §10).
- **`T::Item`** — a path (0008 §3) in type position, head a type-name token, no
  bracket (the 0007 §6.2 leading/following rule is untouched). `::` is never an
  expression operator, so no indexing/arithmetic misread; the final segment
  resolves to the associated type by `T`'s bound — position-resolved like
  `Enum::Variant` (0001 §8.2).

---

## 5. The closure question, resolved

**Ruling: full capturing closures are refused this edition; higher-order code is
capture-free — a fn-pointer plus an explicit context value threaded by hand — and
`Opt::map` ships on that plus completed inference, adding no surface.**

### 5.1 What full closures would cost (why not now)

A capturing closure bundles code with an *environment* that touches the **entire**
memory model:

- **Borrow captures force region-typed closures.** Capturing `read x` stores a
  borrow — a borrow-typed field — dragging a region parameter onto the synthesized
  closure type and every signature holding it: precisely 0007 §3.5 / 0001 §3.4's
  refusal, the machinery the value-first bet exists to avoid.
- **Move captures force anonymous owned aggregates with drop glue** — move
  checking, `copy`-or-not, alloc-on-drop propagation (0007 §3.4), *and* an
  anonymous existential type that must be boxed or returned `impl Trait`-style
  (HKT-adjacent, 0007 §1.1) to be stored.
- **Capture-mode inference fights P2/NN#17** — inferring modes at a value that
  crosses a signature; spelling them is ceremony a fn-pointer avoids.

A large addition — a fraction of a second borrow system — for a case (§1.2) that
does not need it. P6's default is no, and the priority order agrees: closures buy
item-7 ergonomics at an item-3 and item-5 cost.

### 5.2 The capture-free subset that ships instead

Higher-order operations take a **fn-pointer plus an explicit context parameter**,
threaded by value — P2's allocator-as-value pattern (0007 §8.3) generalized from
allocators to arbitrary transforms:

```
fn map[T, U](o: Opt[T], f: fn(T) -> U) -> Opt[U] {
    match o { Opt::Some(v) => Opt::Some(f(v)), Opt::None => Opt::None }
}
fn map_ctx[T, U, C](o: Opt[T], ctx: read C, f: fn(read C, T) -> U) -> Opt[U] {
    match o { Opt::Some(v) => Opt::Some(f(ctx, v)), Opt::None => Opt::None }
}
```

`map_ctx`'s `ctx` *is* the environment — an explicit, value-threaded, region-free
parameter the reviewer sees; state (an accumulator, a base pointer, an `Alloc`)
travels as a value, not a hidden capture. `f` forwards `v: T`, so 0007 §3.4 does
not tax `map` (§1.2). **No language surface is added** — fn-pointers, modes, and
effect markers on fn-pointer types already exist (0001 §6.1) — honoring "when you
add, remove" by adding nothing.

### 5.3 The one thing to complete (not design)

**RESOLVED (2026-07-12).** `map` was uncallable only via E1002 (§1.2): the
prototype did not infer `U` from `f`'s return. This was *within* 0007 §2.2's
body-local inference — an **implementation obligation**, not a design choice, with
no surface attached. Closed by extending the call-site unifier (`unify` in
`src/check/generics.rs`) to descend `fn(..)->..` formal parameters structurally,
binding a type parameter from the actual fn-pointer argument's parameter and
return types — the natural completion of the existing pass, no new engine. `map`
now checks clean and runs to `42` on the tree-walker, MIR, and native engines
(`opt_map_end_to_end`).

### 5.4 Staged plan and gate

**OBL-GENERICS-CLOSURE.** Full capturing closures are deferred, not dropped.
Reopen requires **both**: (a) a basket-grade case where the capture-free threading
cost (§5.2) is *measured* as the dominant reading-friction or expressiveness
failure — the bar 0007 §1.1 set for associated types, now met by iteration and to
be met by closures before they ship; **and** (b) a capture model compatible with
**0007 §3.5** — **move/copy captures only**, closure type a synthesized owned aggregate
with ordinary drop glue and **0007 §3.4** alloc-on-drop, **borrow captures remaining
refused** (they are the region-typed-closure machinery). Until both hold,
capture-free is the one way (P3).

---

## 6. Rejected alternatives

- **Type parameter `Iter[Item]` — rejected** (§1.1): coherence permits multiple
  `Iter[·]` impls per type, so the return is ambiguous. Iteration needs the
  functional dependency an associated type gives.
- **One Rust-style `Iterator` storing the collection borrow — rejected** (§3): a
  borrow field (0001 §3.4) forcing a region-typed iterator (0007 §3.5), *and*
  `write self` yielding owned items needs swap/replace. Two smaller protocols cost
  less than either addition.
- **Adding swap/replace to make `next(write self)` work — rejected**: a
  memory-model primitive (0001's turf) added for one protocol shape that
  `next(take self)` already serves via the seed's proven `pop_front`. Recorded as a
  standing memory-model question if a broader case appears.
- **Region-parameterized iterators (borrowed items) — rejected this edition**
  (§3.4): the value-first bet's antithesis; deferred as OBL-ITER-BORROW.
- **Full capturing closures — rejected** (§5): a whole-memory-model addition for a
  capture-free case; OBL-GENERICS-CLOSURE gates the reopen.
- **Bounds on associated types / GATs / defaults / multiple members — rejected**
  (§2.3): the refusals kept intact, each reopenable by its own basket case.
- **`for` as keyword-free library sugar — rejected** (§4.1): the open-coded walk
  is the friction P13 measures; one traversal keyword earns its slot.

## 7. Consequences and costs (debts, not absolutions)

- **Borrowed-item and by-borrow `List` iteration are unavailable** (§3.4,
  OBL-ITER-BORROW): iterating large non-`copy` elements without consuming the
  collection, or walking a `List` non-destructively, is unwritable — `Indexed`
  copies (free for `copy`, impossible otherwise), `Iter` consumes, the seed's
  `length` stays hand-written. 0001 §3.4's no-borrow-field rule biting where Bet 5
  predicted.
- **Two iteration interfaces, not one** (§3): forced by two independent 0001 walls,
  each spending only its one member.
- **Higher-order code threads state by hand** (§5.2): `map_ctx`'s explicit `ctx`
  is more tokens than a capture — the deliberate P2 trade (visible value over
  hidden environment), the debt OBL-GENERICS-CLOSURE may call in.
- **`Indexed` implies random access** (§3.2): its `usize` cursor over an
  O(n)-indexed structure is an O(n²) walk — a cliff the *impl author* must not
  create (the interface documents "cheap `at`"); no checker enforces it, an idiom
  obligation consistent with P4.
- **`Indexed` costs a guard + a bounds check per step** (§4.2): each turn runs the
  loop's termination guard (the `Opt::None` match on `at`) *and* the impl's in-range
  test (`i >= count`, §8), where the seed's manual index ladder paid a *single* static
  check for the whole fixed-length unrolling. A real per-element cost, not a wash — but
  a **visible** one: the reader sees the `for` and knows the loop guards and
  bounds-checks every step, exactly P4's visible-cost test (formatter and reviewer read
  the traversal and its cost in the same four tokens). The ladder hid the same
  termination reasoning in prose the reviewer had to reconstruct; the `for` names it.
  Cost paid in cycles, bought back in legibility.
- **The associated type is still real complexity** (P11's concession): the claim is
  only that it is the *smallest* form the pressure case demands, every adjacent
  growth (§2.3) refused.

## 8. Worked examples

**Arena traversal replaces the index ladder:**

```
impl[T: copy] Indexed for Arena[T] {
    type Item = T;
    fn at(read self, i: usize) -> Opt[T] {
        if i >= conv usize self.count { return Opt::None; }
        return Opt::Some(self.mem[i]);          // copy out; T: copy, no borrow, no region
    }
}
fn sum(ar: read Arena[i64]) -> i64 {
    let mut s: i64 = 0;
    for x in read ar { s = s + x; }             // read loan on `ar` live across the body
    return s;
}
```

Mutating `ar` inside the `for` is E0303 (§4.3). Non-consuming, copy-yielding,
**ground-floor** (non-`alloc`, interrupt-callable — the ground-floor iteration
protocol, §3.3), one associated type.

**`List` consumption replaces open-coded recursion:**

```
impl[T] Iter for List[T] {
    type Item = T;
    fn next(take self) alloc -> IterStep[T, List[T]] {              // alloc: matches the interface (§3.1)
        match self {
            List::Nil => IterStep::Done,
            List::Cons(x, tail) => IterStep::More(x, unbox(tail)),   // pop_front, generalized
        }
    }
}
// for x in list { … }  moves `list`; `list` is unusable in the body (use-after-move safety).
```

`next` is `alloc` (it `unbox`es, §3.3), so `for x in list` inherits `alloc` — a
`for` over a `List` in an unmarked function is a definition-site effect error at
the loop, exactly the P2 / 0007 §3.4 partition.

---

## 9. Prototype stage plan and obligations

**Stage plan:**
- **Stage 1 — parse.** `for PATTERN in OPERAND { }`; `type Ident;` in interface
  bodies and `type Ident = Type;` in impl bodies; `T::Item` projection in type
  position. NN#13 walks (§4.4) as parser tests.
- **Stage 2 — check + desugar.** The associated-type member and single-valued
  projection (§2.2); interfaces `Iter`/`Indexed` and `IterStep`; the two desugars
  (§4.2) lowering to existing `loop`/`match`; loan/move soundness reusing 0001
  §2.2/§2.3 and 0007 stage-2 ripple checks (§4.3) — *no new aliasing rule*; effect
  inheritance through the desugar (§3.3).
- **Stage 3 — corelib port.** `Arena: Indexed` with a traversal replacing the
  index ladder; `List: Iter` replacing hand-recursion; `Opt::map`/`map_ctx`
  written capture-free (§5.2); close E1002 (§5.3) as its own commit; record the
  before/after friction as the seed round did.

**Obligations (recorded here, not edited into their home documents):**
- **OBL-GENERICS-ITER — discharged**: the reserve is drawn (§2), `for` specified
  (§4), `Opt::map` made writable capture-free (§5). Its two residuals become the
  new obligations below.
- **OBL-ITER-BORROW (new).** Borrowed-item and mutating iteration, and
  non-consuming pointer-chain iteration, deferred (§3.4). Gate: 0007 §3.5's
  region-parameterized-type question reopened, or a region-free borrowed-yield
  model found.
- **OBL-GENERICS-CLOSURE (new).** Full capturing closures deferred with §5.4's
  two-part gate.
- **Iterator adapters (2026-07-13) — LANDED (capture-free); fully-generic
  adapters RESOLVED (2026-07-13).**
  The std+I/O milestone's headline feature, built additively on §3.1's `Iter`
  protocol (`next(take self)` returning `IterStep::More(item, successor)`; the
  adapter is moved through each step and the successor is threaded back). All
  transforms are capture-free bare fn pointers (§5.2), referenced by name and
  stored in an adapter's struct field (as the allocator vtable stores
  `bump_alloc`). *Landed, checked clean + run to exact values on the tree-walker,
  MIR, and native engines* (`tests/adapters.rs`): an eager `fold[T, A]` (generic
  over element and accumulator) driving `List::next`; a lazy `MapIter[T, U]` and
  `FilterIter[T]`, each a generic struct that `impl Iter`, over `List`; and a
  `list -> filter(even) -> map(double) -> fold(sum)` chain that composes lazily
  (one fused pass, no intermediate list) to `12`. `filter`'s predicate is
  `fn(read T) -> bool` — it must BORROW the item, since a by-value predicate
  moves it and E0301 then bars yielding it.
  *Fully generic adapters over an arbitrary `I: Iter` inner now check and run.*
  A `struct MapIter[I, U] { f: fn(I::Item) -> U, .. }` that itself `impl Iter`,
  and `gmap`/`gfold[I: Iter, ..](it: I, f: fn(.., I::Item) -> ..)`, check clean
  and run to exact values on all three engines (`generic_inner_adapter_over_any_iter`,
  = 20). The prior BLOCK was a projection-normalization *ordering* gap in the
  checker: a concrete `fn(i64) -> U` argument/field would not unify against the
  projection `I::Item` (E0703) because call-site inference resolved `I::Item` to
  the impl's binding only AFTER argument/field mode-checking. Fixed surgically in
  `src/check/generics.rs`: `normalize_projections` injects `I::Item -> i64` into
  the inference map (via `resolve_proj`, mirroring the monomorphizer's already-
  correct `generics::resolve_proj`) BEFORE the arg/field checks, in both
  `check_generic_call` (moving the resolution above the arg loop) and
  `check_generic_struct_lit`. The concrete `I` is already known at that point
  (pinned by the other value arguments / the expected-type annotation), so this
  is a normalization reorder, NOT a new constraint solver. Monomorphization and
  lowering already resolved projections, so no compiler surface below the checker
  changed. The old pin `generic_inner_adapter_field_is_rejected` flipped to the
  passing `generic_inner_adapter_over_any_iter`. Remaining: capturing closures
  (OBL-GENERICS-CLOSURE) still gate stateful transforms.
- **Prototype inference note — RESOLVED (2026-07-12).** E1002 (infer a type
  parameter from a fn-pointer argument's *return* type) was a completeness gap
  within 0007 §2.2; closed by extending `unify` to descend `fn(..)->..` formal
  parameters (§5.3) — no surface change. A genuinely uninferable parameter still
  correctly raises E1002.
- **Forced on 0007 (refusal ledger, when 0007 is next revised):** §1.1's
  associated-type refusal → *reopened and minimally admitted* (one member,
  §2.3 sub-refusals retained); §7.2's `for`/iteration deferral → *discharged*
  (§3–4); §7.4's operator overloading stays deferred (0009 does not take it).
- **Forced on the spec (recorded, not edited):** the `for` statement and
  `type`/`I::Item` grammar join OBL-GRAM's surface; the two iteration interfaces
  and their by-value discipline join OBL-GENERICS's stdlib-successor scope.

**Migration: no reserved-word break.** `for`, `in`, and `type` are **contextual**
keywords (§4.4): each is a keyword only in its grammatical position — `for`/`in` in the
for-statement header, `type` inside interface/impl bodies — and an ordinary identifier
everywhere else. So no program breaks merely by *containing* an identifier named `for`,
`in`, or `type`; the new library interfaces and the new statement form are purely
additive (P15's additive path).

**The residual, walked.** One narrow case is not free: a variable named `for` or `in`
used **in the position where the for-statement grammar now commits**. A statement that
begins with the token `for` is parsed as a loop, so a pre-existing statement-leading use
of a variable `for` (e.g. `for = next_id();` or `for + 1;`) would misparse; a variable
`in` sitting where the header expects the `in` separator collides likewise. That is the
whole residual — it bites only an identifier both named `for`/`in` *and* appearing in the
one position the keyword now claims; any other use (a field, a call argument, an `in` not
between a `for`-pattern and its operand) is untouched. The corelib seed uses no such
names, so the measured migration cost is zero; the residual is recorded, not claimed away.
