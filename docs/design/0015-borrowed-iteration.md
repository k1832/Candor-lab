# 0015 — Borrowed-element iteration (RefIndexed)

**Status:** ACCEPTED and IMPLEMENTED (2026-07-14). Adversarially reviewed (verdict:
survives with repairs; see [the review](../reviews/0015-borrowed-iteration-review.md));
the soundness prerequisite it surfaced (the loan-provenance UAF family) is **fixed and
audited**; §5/§7 revised accordingly. `for read x in read coll` (the §4 recommendation)
is implemented end-to-end over a user `impl RefIndexed`, byte-exact on all five engines
(tree-walker, MIR, Cranelift, LLVM, AOT). Implementing §4 required COMPLETING one piece of
the inherited machinery the §5 escape argument rests on: the reborrow-through-call-return
loan-provenance discipline, fixed for FREE-fn returns by the four `soundness:` commits, did
NOT cover METHOD-call returns — the exact shape the desugar emits (`__c.get_ref(__i)`); the
§5 hinge / §7 open-Q1 was therefore live for the loop's method call. It is now closed (the
method-call return carries the receiver's loan; ledger `OBL-ITER-BORROW (shared branch)`,
2026-07-14). Escapes of the yield are rejected; the loop-local-cursor escape is caught
conservatively (§7 open-Q1's documented non-escape fallback). See §8.
**Date:** 2026-07-13
**Philosophy hooks:** **P12/Bet 5** (value-first: this is the *borrow* gear reaching
into iteration, and the whole difficulty is doing it without regressing the
value-first bet), **P6** (budget: adds one library interface and one `for`
surface variant, removes nothing — a named tension), **P4/P9** (the `for`-by-ref
desugar is statement-level, greppable, visibly costed), **P3** (one loop family;
this is a third operand/pattern mode of the *existing* `for`, not a new loop),
**NN#17** (nothing crosses a signature by inference — the yielded borrow is tied
to the `read self` receiver, not to an inferred lifetime parameter), **P2**
(local verifiability — the loan that keeps the yield sound is the one already
visible on the loop). Subordinate to `LANG_PHYLOSOPHY.md` and to designs 0001
(memory model — respected once, unchanged) and 0007/0009 (the interface and
iteration systems this extends). Where they conflict, the higher document wins
and this one changes.

**What this is.** The design for **borrowed-element iteration**: walking a
collection by *shared reference* to each element (`read Item`) rather than by
owned copy (`Indexed`, 0009 §3.2) or by consumption (`Iter`, 0009 §3.1). It
proposes to admit the `RefIndexed` protocol already *declared but unimplemented*
in the corelib seed (`compiler/tests/fixtures/corelib/core/iter.cnr`) and to
specify its `for`-by-ref desugar, its checker discipline, and its soundness. It
takes up the **region-free branch** of the gate 0009 §9 set on **OBL-ITER-BORROW**:
"0007 §3.5's region-parameterized-type question reopened, **or a region-free
borrowed-yield model found**." This document argues the second is found; the first
stays refused.

**What this is not.** Not first-class lifetimes, GATs, or region-parameterized
associated types (0007 §1.1 / §3.5, 0009 §2.3 — all stay refused). Not mutating
iteration (`for write x` yielding `write Item`) — a symmetric extension is noted
(§7) but not decided here. Not by-borrow `List` iteration (a borrow-typed cursor
into a pointer chain, 0009 §3.4 — still refused: a chain has no `usize` index).
Not a change to the memory model, effect set, coherence, or borrow discipline —
all fixed by 0001/0007 and respected once.

---

## 1. Problem statement

Candor has two working iteration protocols (0009 §3):

- **`Iter`** (0009 §3.1) — *consuming*: `next(take self) -> IterStep[Item, Self]`.
  It **moves** the collection and yields **owned** items. `for x in coll` over a
  `List[T]` destroys `list`.
- **`Indexed`** (0009 §3.2) — *borrow-copy*: `at(read self, i) -> Opt[Item]` where
  the impl requires `Item: copy`. It borrows the collection `read` for the loop
  and yields a **copy** of each element. `for x in read coll` over an `Arena[T]`
  touches every element without consuming — but only by copying it, and only when
  `Item` is `copy`.

Neither covers the case where you want to **touch each element in place, by shared
reference, without copying it or consuming the collection**:

- **`Vec[T]` / `Map[V]` by `read`.** Iterating a `Vec[BigStruct]` to read one field
  of each element must today either consume the vector (`Iter`) or copy every
  element out (`Indexed`, and only if `BigStruct: copy`). A non-`copy` element
  (anything owning a `Box`, a nested collection, a `drop`-hooked resource) cannot
  be walked non-destructively **at all** — 0009 §7's first named debt.

- **A streaming `Lines: Iter` that yields borrowed `str`.** A line reader that owns
  a buffer wants to yield a `read str` view into that buffer per line. Under `Iter`
  it can only yield an **owned `String`**, copying every line off the buffer it
  already holds — a pure-overhead copy the borrowed form removes (the ergonomic +
  performance case named in the round mandate; 0013 owns the `str`/`String` split).

- **General.** Any "look at each element" pass (sum a field, find a max, print) over
  a non-`copy` element type is unwritable in the value/borrow gears without this.
  0009 §3.4 recorded exactly this as the *named cut* and filed it as
  **OBL-ITER-BORROW**; 0009 §7 lists it first among the round's accepted debts.

The capability is missing, and its absence is the difference between "iterate by
value/copy only" and "iterate the way most read-only passes actually want to."

---

## 2. Constraints from the memory model

A borrowed *yield* is hard precisely because it touches the rules the memory model
exists to enforce. The design must respect all of the following, none of which it
may change (0001/0007 outrank this document):

1. **No borrow-typed fields** (0001 §3.4; 0007 §3.5). A struct or enum field may
   not have a borrow type (`read`/`write` borrow, `slice`, `slice_mut`). So an
   iterator **may not store a borrow of its collection** — Rust's
   `struct Iter<'a>{ coll: &'a C }` is unwritable. This is the *decisive* rule:
   any design whose iterator object holds the collection borrow is dead on arrival.

2. **No type parameter ranges over a borrow** (0007 §3.5). A type argument ranges
   over owned/value and `rawptr` types, **never** `read T`/`write T`/`[T]`. So
   `Opt[read Item]` — an `Opt` instantiated with a borrow — is **ill-formed**: it
   would put a borrow in the `Some` variant's payload, which is both a borrow-typed
   field (rule 1) and a type parameter over a borrow (this rule). This single
   constraint kills the "obvious" signature (§4, option b) and forces the shape of
   the one that survives.

3. **XOR loans** (0001 §2.2). At every point, for every place: any number of shared
   borrows **xor** exactly one exclusive borrow. While a shared loan on the
   collection is live, the collection may be read and re-shared but **not written,
   not moved, not exclusively borrowed** (0001 §2.2's move/write clause). This is
   the mechanism that must prevent mutation-during-borrow (iterator invalidation).

4. **No first-class lifetimes; regions have compact defaults** (0001 §3.3; NN#17).
   There is no lifetime *parameter* a caller supplies. A function returning a borrow
   ties it to an input by the **compact default**: *one borrow-in, one borrow-out ⇒
   the return derives from that sole borrow parameter*, no region variable written.
   A borrow whose provenance is a **local is rejected** — a borrow may not outlive
   the body it was born in (0001 §3.3). Nothing about regions crosses a signature by
   inference (NN#17).

5. **The borrow must not escape its live range, and the collection stays
   read-locked while any yielded borrow is live.** Combined consequence of 1+3+4:
   the yielded `read Item` is a reborrow whose provenance is the collection; its
   live range must sit **inside** the collection loan's live range; and for as long
   as it (or anything derived from it) lives, the collection is frozen to shared.

6. **NLL-lite, lifetime-variable-free checking** (0001 §2.3). The checker is
   backward liveness of borrow bindings plus a conflict scan (`compiler/src/check/
   loans.rs`) — no lifetime solver, no cross-function region unification. Whatever
   this design demands of the checker must fit that shape: *a loan held over a live
   range, and a conflict scan*, nothing more.

The design below is the one shape that satisfies 1–6 simultaneously without adding
any of the refused machinery (GATs, region parameters on types, closures).

---

## 3. Design space survey

Four viable shapes, each measured against §2's constraints, its `for`-desugar, its
checker obligation, and its soundness story.

### 3.1 Option A — `RefIndexed`: indexed-by-reference (count + get_ref)

The shape already declared in the corelib seed:

```
interface RefIndexed {
    type Item;                                   // one associated type (0009 §2.3 intact)
    fn count(read self) -> usize;                // the loop bound
    fn get_ref(read self, i: usize) -> read Item; // a shared reborrow of element i
}
```

**How it types without first-class lifetimes.** `get_ref` is region-free by 0001
§3.3's **compact default**: its sole borrow-in is `read self`, its sole borrow-out
is `read Item`, so the returned reborrow is *defined* to derive from `self` — no
region variable is written, none is *storable* (rule 1). This is not a new rule:
`get_ref` is **structurally identical to `arena_get` in 0001 §11.5** —
`fn arena_get(ar: read Arena, i: u32) -> read Node` — a borrow-returning accessor
that already checks clean on all three engines. `get_ref` generalizes it from a
concrete `read Node` to the projected `read Item` (0009 §2.2 treats `T::Item` as an
opaque borrowable type). The yielded borrow is tied to the **`read self` receiver
loan**, which *is* the collection loan the loop holds — that is the whole trick, and
it needs no lifetime parameter.

**Why `count` + `get_ref` and not one method.** A single `at_ref(read self, i) ->
Opt[read Item]` is **unwritable** (§2 rule 2): `Opt[read Item]` instantiates `Opt`'s
`Some(T)` payload with `T = read Item`, a borrow-typed variant payload (rule 1) and
a type parameter over a borrow (rule 2). The split dodges it exactly: `count`
returns an **owned `usize`** (`copy`, no borrow), giving the loop its bound; and
`get_ref` returns the **bare `read Item`** — a borrow returned directly, never
wrapped in a sum. No sum type ever contains a borrow. This is the load-bearing
design move, and it is why the interface spends **two methods** (0009 §2.3 caps
associated *types* at one, not method count — `Iter` and `Indexed` each happen to
use one; `RefIndexed` uses two, and the cap is untouched).

**No `Item: copy` requirement.** Unlike `Indexed`, `RefIndexed` places **no**
copyability bound on `Item` — yielding a borrow is exactly what lets non-`copy`
elements be walked (the whole point). `count`/`get_ref` are both non-`alloc`
(returning a `usize` and reborrowing allocate nothing), so `for read x in read coll`
is a **ground-floor, interrupt-callable** walk, on the same effect tier as `Indexed`
(0009 §3.3).

**The `for`-by-ref desugar** (§4 spells it fully): `for read x in read coll { BODY }`
lowers to a `loop` that holds a loop-local `read` borrow of `coll` across the whole
loop, calls `count` once for the bound, and each turn binds `x` to a fresh reborrow
`__c.get_ref(__i)`.

**Checker obligation.** (i) The loop-local `__c = read coll` places a shared loan on
`coll` live across the loop (used by `count` + every `get_ref`) — the *same* loan
`Indexed` already relies on (0009 §4.3). (ii) The return of `get_ref` is tracked as
a **reborrow of `__c`** (0001 §3.3 default), so `x` carries `__c`'s loan on `coll`.
(iii) The XOR conflict scan (loans.rs `conflict_scan`) then forbids any `write`,
move, or exclusive borrow of `coll` in `BODY` while the loop loan is live. All three
are existing machinery.

**Soundness story** (full argument in §5). The yielded borrow *is* a reborrow of the
collection loan; its live range sits inside that loan's; the loan freezes the
collection to shared for the loop. No use-after-free (the borrow can't outlive its
provenance without keeping the loan live), no mutation-during-borrow (XOR).

### 3.2 Option B — `at_ref(read self, i) -> Opt[read Item]`

The naive one-method mirror of `Indexed`. **Rejected outright** by §2 rule 2:
`Opt[read Item]` is a borrow inside an enum payload — a borrow-typed field (rule 1)
and a type argument over a borrow (rule 2). It cannot even be *spelled* in the type
system, let alone checked. This is the single cleanest illustration of why the
problem is hard and why option A's `count`/`get_ref` split is forced rather than
stylistic. Recorded in §6.

### 3.3 Option C — internal iteration: `for_each(read self, body: fn(read Item))`

A higher-order form: the collection drives, calling a supplied `body` once per
element with a `read Item` reborrow that lives only for the duration of that call.
**Soundness is trivial** — the borrow is a parameter passed *down* into `body`'s
frame and never surfaces as a stored or returned value (0001 §3.4's "borrows are a
gear for passing, not storing," honored perfectly). No loan escapes.

**Rejected** (§6) on three grounds, each fatal for backing the `for` statement:

1. **It cannot express a mutating body without closures.** A real loop body mutates
   outer state (`let mut s = 0; for read x in read v { s = s + x.len; }`). `body` is
   a **bare fn-pointer** (0009 §5: capturing closures are refused, OBL-GENERICS-CLOSURE);
   it cannot touch `s`. Threading `s` as an explicit `ctx` (0009 §5.2) turns every
   loop body into a hand-written fold — the exact friction 0009 §5 deferred, now
   imposed on *all* by-ref iteration.
2. **Control flow cannot cross the fn-pointer boundary.** `break`, `continue`,
   `return`, and `?` in a `for` body are ordinary control flow (0009 §4.1). Across a
   `for_each` fn-pointer they are impossible — you cannot `break` out of a callee.
   This breaks P4 (no hidden control flow) and simply fails to implement `for`.
3. **It is a different surface.** It is not the syntax-directed `for x in OPERAND`;
   it does not compose with the existing `for` desugar (0009 §4.2).

`for_each` is a fine *library* combinator to offer later, but it cannot be the
protocol the `for` statement lowers to.

### 3.4 Option D — a cursor/streaming associated-state shape

A `Cursor` associated type with `advance(write cursor) -> read Item`, mirroring how
`Iter` threads `Self`. **Rejected** (§6): a cursor that can yield a `read Item` must
*hold* a borrow of the collection (to reborrow from it), which is a **borrow-typed
field** (§2 rule 1) → a region-parameterized cursor type → precisely the refusal the
whole design routes around. Strip the borrow to make it storable and the cursor
collapses to a `usize` index — i.e. it *becomes* option A, renamed. There is no
region-free cursor that yields borrows and is also storable; the two requirements are
contradictory under 0001 §3.4.

### 3.5 Option E (for completeness) — region-parameterized associated type / GAT

`type Item[r]` — the Rust `LendingIterator` answer. **Rejected** (§6): it needs
first-class lifetimes + generic associated types, both refused (0007 §1.1, 0009
§2.3), and drags region parameters across every signature that names the iterator —
the value-first antithesis (0001 §3.4 rationale, P12). This is the "big hammer"
0009 §9 named as the *other* branch of the OBL-ITER-BORROW gate; the region-free
option A is offered precisely so this branch stays closed.

---

## 4. Recommendation

**Adopt option A: the `RefIndexed` protocol as declared in the corelib seed, backing
a `for read x in read coll` variant of the existing `for` statement.** It is the only
shape that (a) yields borrows of non-`copy` elements, (b) stores no borrow, (c) needs
no first-class lifetime or region parameter, and (d) reduces its soundness entirely to
already-checked machinery (`arena_get`'s §3.3 default return + `Indexed`'s loop loan).

### 4.1 Protocol signatures

```
interface RefIndexed {
    type Item;
    fn count(read self) -> usize;
    fn get_ref(read self, i: usize) -> read Item;
}
```

- `Item` is unconstrained (no `copy` bound) — non-`copy` elements are the case.
- Both methods are **non-`alloc`** (ground floor, 0009 §3.3).
- `get_ref`'s `read Item` return is region-free by the 0001 §3.3 compact default
  (sole borrow-in `read self`, sole borrow-out `read Item`); the impl's return
  provenance must be reachable through `self` — 0007 §3.5 F9 "interface compact
  default and exact impl region match," the identical rule `arena_get` satisfies
  (0001 §11.5).
- An impl author's contract: `get_ref(i)` for `i < count()` returns a live borrow of
  element `i`; `get_ref` out of range **faults** (like `Indexed::at`'s in-range test,
  0009 §8) — the loop never calls it out of range, so the fault is a defensive floor,
  not a control path.

### 4.2 The `for`-by-ref desugar (statement-level, visibly costed — P4/P9)

Selection is **syntax-directed** by the *pattern* mode, disambiguating cleanly from
`Indexed` (both take a `read` operand):

- `for x in read coll { … }` → `Indexed` (copy each item into `x`) — 0009 §4.1.
- `for read x in read coll { … }` → `RefIndexed` (`x` is a `read Item` reborrow).

The `read` on the *pattern* is the new token; the operand is `read coll` in both
cases. NN#13-clean: the parser reads `for read PATTERN in OPERAND` without a symbol
table (the leading `read` after `for` marks a by-ref binding), and never needs to
know whether the operand is `Indexed` or `RefIndexed` — a checker fact (0009 §4.4).

Desugar:

```
// for read x in read coll { BODY }  ==>
let __c = read coll;                     // loop-local READ borrow, live across the whole loop
let __n: usize = __c.count();            // bound computed once
let mut __i: usize = 0;
loop {
    if __i >= __n { break; }
    let read x = __c.get_ref(__i);       // x : read Item — a reborrow of element __i, dies each turn
    { BODY }
    __i = __i + 1u;
}
```

- `__c` is a `read` borrow (drop-inert, `copy`); `__n` and `__i` are `usize`
  (`copy`, drop-inert). **No owned iterator state crosses any edge**, so the desugar
  needs **none** of the `Iter` desugar's exit-edge sink-move accounting (0009 §4.2) —
  `break`/`return`/`?` from `BODY` are ordinary, exactly as in the `Indexed` desugar
  (0009 §4.2).
- `x`'s binding is fresh **per iteration** and its live range is confined to that
  turn's `BODY` (it is redefined next turn). This is what keeps the yielded borrow's
  range inside `__c`'s (§5).
- Cost is visible (P4): a bound query, a per-step index compare, and a per-step
  `get_ref` (whose own in-range check the impl runs) — the same guard+bounds shape
  0009 §7 already named for `Indexed`, minus the copy.

Rendered as source on request (P4/P6 transparency), like the other two desugars.

### 4.3 Checker changes (the loan discipline)

Small, and almost entirely *reuse*:

1. **Parse** the `for read PATTERN in OPERAND` variant (one grammar arm on the
   existing `for`, 0009 §4.4). *(Grammar surface, OBL-GRAM.)*
2. **Protocol selection**: pattern `read x` + operand `read coll` ⇒ require
   `typeof(coll): RefIndexed`; lower per §4.2. *(Mirrors 0009 §4.1's mode-directed
   selection.)*
3. **Loan checking — inherited, not new.** `__c = read coll` creates a shared loan
   on `coll` anchored to `__c`'s binding, in scope over `__c`'s live range (loans.rs
   `Anchor::Binding`) — identical to `Indexed`'s loop loan (0009 §4.3). The one fact
   the checker must already get right (and does, for `arena_get`): **the return of
   `get_ref(__i)` is a reborrow of `__c`**, so the loan on `coll` is carried by `x`
   (0001 §2.3 step 3: reborrows extend the parent's obligation). The existing
   `conflict_scan` then rejects any `write coll` / move / exclusive borrow in `BODY`
   (E0803/E0802/E0801) while the loop loan is live — the same E0303-class diagnostic
   `Indexed` produces (0009 §4.3), monomorphized over the concrete collection like
   any other loan check (0007 stage-2).

No new aliasing rule, no lifetime variable, no region solver. The novelty is confined
to the desugar target and the protocol-selection arm; the safety is old.

### 4.4 Interp / backend implications (small)

**A borrowed yield is an address.** `get_ref` returns a `read Item`, which at runtime
is a pointer — the identical representation the interpreter already uses for every
`read` parameter and for `arena_get`'s `read Node` return (verified on the
tree-walker, MIR, and native engines). `x` is an ordinary borrow local. There is
**no new runtime value kind, no new calling convention, no new drop behavior** (a
`read` borrow drops trivially). The interp/backend cost is essentially zero beyond
lowering the new desugar — the effort is entirely in the checker (parse + select +
inherited loan check).

### 4.5 Composition with the owned adapters

Honest and partial. The owned adapter family (`fold`/`map`/`filter`, `tests/
adapters.rs`) is built on `Iter` — each adapter **stores its inner iterator in a
struct field** and threads `Self` through `next`. `RefIndexed` **cannot** join that
family as a stored-field lazy adapter, because a by-ref adapter would need to store
the *collection borrow* to reborrow from it — a borrow field (§2 rule 1, refused). So:

- `RefIndexed` backs the **`for` statement**, which holds the collection loan on the
  *loop's own stack* (not in a stored iterator). This is the primary, supported use.
- **By-ref lazy adapter chains** (`v.iter_ref().map(…).filter(…)`) are **not**
  expressible as stored-field adapters — the collection borrow has nowhere to live.
  A by-ref transform is instead written as an internal `for read` loop, or the
  element is copied into the owned `Iter` pipeline where copy is acceptable.
- An eager by-ref combinator taking `read coll` + a `usize` cursor by *parameter*
  (never storing the borrow) is expressible — but that is just `RefIndexed`'s own
  shape spelled at a call boundary, not a new stored adapter. Noted as available, not
  as a parallel adapter family.

This asymmetry is a real cost (§ Consequences), and a direct consequence of the
no-borrow-field rule biting exactly where 0009 predicted.

---

## 5. Soundness argument

The claim: `for read x in read coll` can produce **neither a use-after-free nor a
mutation-during-shared-borrow**. Both reduce to the XOR-loan mechanism (0001 §2.2)
over the loop-local collection loan, via already-checked cases.

**Setup.** The desugar creates one shared loan `L` on `coll`, anchored to `__c`,
whose live range (NLL-lite, 0001 §2.3) spans the whole loop: `count` uses `__c` at
the top and `get_ref` uses it every turn, so `__c` is live from `let __c` to the
final `get_ref`. By 0001 §3.3's compact default, each `x = __c.get_ref(__i)` is a
**shared reborrow of `__c`**, so `x` carries `L` (0001 §2.3 step 3: a reborrow's loan
extends the parent's obligation).

**(1) No mutation-during-borrow (iterator invalidation).** The guarantee comes from
`L`'s **span across the whole loop** — `L` is anchored to `__c`, and `__c` is live from
`let __c` (its use in `count`) through the final `get_ref`, so by backward liveness
(0001 §2.3) `L` covers every iteration. (It is `L`'s span that protects the loop, **not**
a per-iteration reborrow: each `x = __c.get_ref(__i)` carries a loan rooted at `__c`
that dies at turn end — that per-turn loan is not what freezes `coll`; `L` is. The
earlier framing that leaned on "each `get_ref` return reborrow carrying `L`" is
corrected here per the adversarial review's F3.) While `L` is live, XOR (0001 §2.2)
forbids any exclusive access to `coll` or any overlapping place: a direct `write`, a
`write coll` argument, a move out, or an exclusive reborrow. Any such access in `BODY`
overlaps `L` (loans.rs `overlaps` + `judge`) and is rejected — E0803 (write), E0802
(move), or E0801 (conflicting borrow). This is the *same* guarantee `Indexed` already
gives (0009 §4.3's E0303 example). The collection is therefore **read-locked (unmutated)
for the entire loop**, so no element the yielded borrow views can be reallocated, moved,
or overwritten while the borrow is live.

**(2) No use-after-free.** `x` is a reborrow whose provenance is `coll` (through
`__c`). Two cases:

  - **`x` does not escape `BODY`** (the overwhelming case). `x` is redefined each
    turn, so its live range is confined to that iteration, which is strictly inside
    `L`'s live range. `x` is dropped (trivially — a `read` borrow) at turn end while
    `coll` is still fully live and loaned. No dangling.

  - **`BODY` tries to make `x` escape** (assign it to an outer binding, return it,
    store it in a field). Each route is closed by an *existing* rule, with no new
    check: storing in a struct/enum field is 0001 §3.4 (no borrow fields); returning
    it fails the provenance check — `x`'s provenance is `coll`, and if `coll` is a
    local, "a borrow whose provenance is a local is rejected" (0001 §3.3), while if
    `coll` is a borrow parameter the return is governed by the function's own region
    signature, not the loop; assigning it to an outer local extends the live range of
    a value carrying `L`, which **keeps `L` live past the loop** — and `L` then
    continues to forbid mutating or moving `coll` for as long as the escaped borrow
    lives, i.e. the collection stays frozen exactly as long as a borrow of it is
    reachable. In every route the borrow either cannot escape, or drags its
    read-lock on `coll` with it — so it can never observe freed or overwritten
    storage.

    **This escape argument was UNSOUND when this doc was first drafted** — the
    adversarial review (F1) showed the implemented loan model *shed* a borrow's loan
    when it was copied/aliased/returned, so an escaped `x` did **not** keep `L` live and
    a real safe-code use-after-free followed. That was a **pre-existing checker defect,
    not specific to `for read`** (reachable through plain `let`, slice copies, `String`
    views, and view-returning functions alike). It has since been **fixed** (ledger
    `LOAN-COPY-UAF` / `STR-VIEW-UAF` and the four `soundness:` commits of 2026-07-13):
    every borrow-kind value — a `read`/`write` borrow, a `slice`/`slice_mut`, a `str`/
    `[u8]` view — now uniformly threads its source loan through copies, aliases, pattern
    bindings, view retypes, and function returns. With that fix landed the escape
    argument above is **valid as stated**, so `for read` needs **no** special
    non-escape rule on `x` (0015's open-Q1 fallback is not required); its soundness
    rests on the now-audited general loan-provenance discipline.

**Reduction to checked ground.** `get_ref`'s *definition* is checked once, at its
definition site, as a borrow-returning accessor deriving its return from `read self`
— structurally `arena_get` (0001 §11.5), which the checker already accepts on all
three engines. The *use* site (the desugar) is `Indexed`'s loop loan (0009 §4.3, also
already accepted) plus ordinary reborrow tracking through a call return (the same
tracking `arena_get`'s callers exercise). The design introduces **no aliasing rule
the checker does not already run**; its soundness is the composition of two
independently-verified pieces.

**The one load-bearing checker fact** the argument rests on, called out for the
reviewer: *the return of `get_ref(__i)` must be tracked as a reborrow of `__c` (not
as a fresh, provenance-free borrow), so that escaping `x` keeps `L` live.* If the
checker instead treated `get_ref`'s return as unanchored, case (2)'s escape route
would be unsound. This is the §3.3 compact-default behavior and is exercised by
`arena_get` today — but it is the hinge, and §7 flags verifying it end-to-end for the
loop case as a gating obligation.

---

## 6. Rejected alternatives (philosophy §8.6)

- **Option B — `at_ref(read self, i) -> Opt[read Item]`.** Rejected (§3.2): a borrow
  inside an enum payload, violating both "no borrow fields" (0001 §3.4) and "no type
  parameter over a borrow" (0007 §3.5). Not even spellable. Its impossibility is what
  *forces* the `count` + `get_ref` split.

- **Option C — internal iteration `for_each(read self, body: fn(read Item))`.**
  Rejected (§3.3): sound but cannot back the `for` statement — a bare fn-pointer body
  cannot mutate outer state without the closures 0009 §5 refused, and `break`/
  `return`/`?` cannot cross the fn-pointer boundary (P4). Fine as a future library
  combinator; not the loop protocol.

- **Option D — a borrow-yielding cursor / associated state type.** Rejected (§3.4):
  a cursor that yields borrows must hold the collection borrow — a borrow field
  (0001 §3.4) → a region-typed cursor. Strip the borrow and it *is* option A
  (a `usize` index). No region-free storable borrow-yielding cursor exists.

- **Option E — region-parameterized associated type / GAT (`type Item[r]`).**
  Rejected (§3.5): needs first-class lifetimes + GATs (both refused, 0007 §1.1 /
  §3.5, 0009 §2.3) and spreads region parameters across every signature naming the
  iterator — the value-first antithesis (P12, 0001 §3.4). This is the OBL-ITER-BORROW
  gate's *other* branch (0009 §9); option A exists to keep it closed.

- **A `copy` bound on `RefIndexed::Item` (making it just `Indexed` in disguise).**
  Rejected: the entire motivating case is **non-`copy`** elements (§1). A `copy`
  bound would make `RefIndexed` strictly redundant with `Indexed` (0009 §3.2), which
  already copies `copy` items. `RefIndexed` earns its slot only by covering the
  non-`copy` case `Indexed` cannot.

- **Overloading `Indexed` with a by-ref method instead of a new interface.**
  Rejected: two associated interfaces with distinct method sets and distinct
  yield disciplines are clearer than one interface with a copy path and a borrow path
  (P3 — one canonical shape per protocol); and an impl may reasonably provide one and
  not the other (a `copy`-element `Arena` wants `Indexed`; a `Box`-element `Vec` wants
  `RefIndexed`). Separate interfaces keyed by `(I, C)` (0007 §2.3) keep coherence a
  lookup.

---

## 7. Open questions / risks

Flagged explicitly for the adversarial reviewer and the deciding authority — the
parts least nailed down.

1. **Does the checker anchor `get_ref`'s returned borrow to `__c` through the
   desugared call?** (The §5 hinge.) — **RESOLVED (2026-07-13).** When drafted this was
   the top open risk, and the adversarial review confirmed it was *not* holding: the
   loan model shed a borrow's loan on copy/alias/return, so an escaping `x` did not keep
   `L` live — a real safe-code UAF (F1). It was a pre-existing checker defect, since
   fixed across the whole borrow-kind family (the four `soundness:` commits; ledger
   `LOAN-COPY-UAF`/`STR-VIEW-UAF`): a returned/aliased borrow now provably extends its
   source loan, and a dedicated completeness sweep found no remaining route (match
   bindings, aggregate transit, view returns, rawptr, loop-carried all closed or
   unsafe-gated). The escape route in §5 case (2) is now sound, and the non-escape
   fallback rule is **not** needed. The remaining `for read`-specific work is
   implementation (the desugar + protocol-selection arm), not this soundness hinge.

2. **Interaction of a scope-exit drop of `coll` with a still-live escaped borrow.**
   loans.rs classifies `Access::ScopeExit` as `AccessKind::None` — a drop point is
   "neither a use nor a def of a loan." If an escaped `x` (case (2)) keeps `L` live
   past a point where `coll`'s owner goes out of scope, is the drop-vs-live-loan
   conflict caught? For non-escaping `x` (the normal case) this never arises. But the
   escape case needs the model to guarantee `coll` cannot be dropped while a borrow of
   it lives — and the current scope-exit classification suggests that guarantee may
   live elsewhere (or may be an existing gap). Worth confirming this is not a
   pre-existing soundness assumption `RefIndexed` is the first to actually stress.

3. **Mutating by-ref iteration (`for write x` / `get_mut(write self, i) -> write
   Item`) — deferred, but is the deferral clean?** A symmetric exclusive protocol
   looks region-free too (each turn's single exclusive reborrow is the only live one,
   and 0001 §2.2 tracks index access at whole-collection granularity, so there is no
   index-disjointness obligation). It would finally make the *other* half of
   OBL-ITER-BORROW (mutating iteration, 0009 §3.4) writable without swap/replace. But
   it interacts with the whole-collection-granularity loan (every `get_mut` is an
   exclusive access to *all* of `coll`, so successive turns must fully serialize —
   fine for a loop, but worth stating) and with `Item: copy`-free move-out concerns.
   **Not decided here**; flagged as the natural next increment if `RefIndexed` lands.

4. **`Lines`-style streaming borrowed yield is not `RefIndexed`.** `RefIndexed`
   requires a `usize`-indexable collection with a cheap `get_ref(i)` and a known
   `count` (0009 §7's "cheap `at`" idiom obligation applies identically). A streaming
   line reader has no random-access index and no up-front count — it is an `Iter`, and
   yielding a *borrowed* `str` from an `Iter::next` is a **different, harder problem**
   (the borrow would point into buffer state the successor iterator owns — squarely
   the borrow-field wall again). So the §1 `Lines` motivation is **only partially**
   served: index-backed collections (`Vec`, `Arena`, `Map`-over-dense-keys) get
   borrowed iteration; streaming sources do **not**, and remain owned-yield only.
   This should be stated as a scope limit if the design is accepted, and it re-raises
   whether a separate borrowed-streaming protocol is eventually needed (a second
   basket-grade case, per 0007 §1.1's bar).

   **RULED (deciding authority, 2026-07-16): owned-`String` per line is the TERMINAL
   answer; no borrowed-streaming protocol is commissioned.** `RefIndexed` already serves
   the common non-`copy` *indexed* walk; a streaming borrowed yield would be a fresh
   basket-grade design (re-solving the borrow-field wall and advance-vs-live-view
   freezing) for the rarer case, and the per-line alloc+copy is an efficiency footnote,
   not a soundness gap. `read_line`/`Lines` yield an owned `String`; the borrowed-
   streaming protocol is closed, not deferred. If a future profile shows the per-line
   copy is a real bottleneck, this ruling has a falsifiable re-open trigger.

5. **`count`/`get_ref` consistency is an unchecked impl contract.** Nothing forces
   `get_ref(i)` to be valid for all `i < count()`, nor `count()` to be stable across
   the loop (it is called once, so a mutating `count` cannot even be observed — but an
   impl whose `get_ref` faults below `count` is a bug the checker cannot catch). This
   is the same idiom-obligation class as `Indexed`'s "cheap `at`" and O(n²) cliff
   (0009 §7) — consistent with P4 (visible cost, author's responsibility), but worth
   the reviewer confirming it introduces no *soundness* obligation (it does not: an
   out-of-range `get_ref` faults, it does not read freed memory).

---

## 8. Obligations

- **OBL-ITER-BORROW (0009 §9) — shared branch DISCHARGED (region-free), 2026-07-14.**
  This design takes the region-free branch of the gate for **shared** borrowed
  iteration over index-backed collections, now implemented end-to-end (`for read x in
  read coll` over a user `impl RefIndexed`, byte-exact on all five engines; the
  method-return loan-provenance completion that makes §5's escape argument hold in the
  implementation is landed — ledger `OBL-ITER-BORROW (shared branch)`, 2026-07-14). It
  does **not** discharge: mutating iteration (open question 3), by-borrow `List`/
  pointer-chain iteration (no `usize` index — still refused, 0009 §3.4), or borrowed
  *streaming* yield (open question 4). The obligation narrows to those; the shared
  branch is closed.
- **OBL-GRAM.** The `for read PATTERN in OPERAND` grammar arm joins the surface.
- **Refused, kept refused:** first-class lifetimes, region-parameterized types, GATs,
  closures — none are added (0007 §1.1/§3.5, 0009 §2.3/§5).
