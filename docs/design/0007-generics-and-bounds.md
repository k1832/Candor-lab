# 0007 — Generics and interface bounds

**Status:** draft
**Date:** 2026-07-08
**Prototype:** stage-1 + stage-2 core shipped in `prototype/`. Stage 1: parse; opaque def-site checking §3; `[T]`-bracket params/bounds/`interface`/concrete `impl`; value-argument inference §2.2 and `name::[T]` §6.2.1; monomorphization §5.1 with the depth backstop §5.1.1; coherence + module-granularity orphan §2.3; cross-type `?` via `From` §7.1. Stage 2 (this edition's core, complete): generic `impl`s (`impl[T] I for List[T]`, incl. bounded `impl[T: I2] …`) — def-site-checked once with opaque `T`, coherence by head-unification overlap (§2.3, two impl heads that unify are a compile-time duplicate), method/`?` dispatch monomorphized per reached target instance; generic-struct `drop` hooks — checked once with opaque `T`, def-site-fixed alloc-on-drop for all instances (§3.4 F5), run by the interpreter in the static drop schedule; and the ripple checks (E0303/E0310/E0401 and the loan machinery) over instantiated generic aggregates. Still deferred: associated types/iteration (OBL-GENERICS-ITER), the source-level strategy override (OBL-GENERIC-STRATEGY), call-site turbofish (OBL-GENERIC-TURBOFISH), and effect polymorphism (OBL-GENERIC-EFFECT).
**Philosophy hooks:** **P11** (public generics checked completely at their
*definition site* against declared interface bounds; a generic that compiles cannot
fail to type-check at instantiation — NN#10; instantiation strategy is a
deterministic, documented, source-overridable default, never an invisible cliff),
**P6** (small core; comptime is *not* the public-generics mechanism; the budget is
fixed — "keep the bound system as small as coherence allows and reject expressive
growth by default"), **P20** (definition-site checking is a compile-speed
*architecture* — checked once, instantiation is cached codegen), **P2/NN#17**
(nothing crosses a signature by inference; effects and bounds are written),
**NN#13** (the grammar parses without a symbol table — OBL-GENERIC-BRACKET), **P3**
(one construct per concept). Subordinate to `LANG_PHYLOSOPHY.md` and to designs 0001
(the memory model this must respect) and 0006 (the syntax whose deferrals this
discharges). Where they conflict, the higher document wins and this one changes.

**Revision history.** 2026-07-08 — revised per joint adversarial review #1 of
designs 0007/0008 (`docs/reviews/2026-07-08-design-0007-0008-review-1.md`): F1/F11
region-vs-type declaration keyword and the declaration-vs-use bracket rule (§6.1);
F2 polymorphic-recursion def-site error plus a monomorphization depth limit (§5.1);
F3 `name::[T]` generic-function-value spelling (§6.2); F4 instantiated-interface
coherence key (§2.3, §7.1); F5 def-site-fixed drop-glue effects, §5.2 corrected
(§3.4, §5.2); F7 cross-type `?` effect inheritance (§7.1); F8 conv-with-type-param
unwritable (§8.5); F9 interface compact default and exact impl region match (§3.5);
F10 `sizeof(T)` discharged by scope (§5.1).

**What this is.** The definition-site-checked polymorphism P11 mandates: the
`interface` construct and its bounds, their copyability/effect/loan interactions
with 0001, the instantiation strategy, and the delimiter under the no-`<>`
constraint. It discharges **OBL-GENERICS** and satisfies **OBL-GENERIC-BRACKET**.
This is where the P6 budget is spent "in preference to almost anywhere else"; it is
written to spend as little of it as the basket's measured pressure demands.

**What this is not.** Not a stdlib design (containers, iteration, the text budget
are named-and-deferred, §7). Not a change to the memory model, fault model, effect
set, or borrow discipline — those are fixed by 0001, and this design must be *sound
for every instantiation while respecting them once*.

---

## 1. Scope discipline first — what this refuses, and the minimum it keeps

P11 spends the P6 budget here, then orders: "keep the bound system as small as
coherence allows and reject expressive growth by default." So this section is
refusal-first. Each refusal is argued from the budget or from a *basket-grade
absence of pressure* — never taste.

### 1.1 The refusals

- **No specialization / overlapping impls.** These need an impl *lattice* with
  negative reasoning — the heaviest coherence machinery and the root of the C++
  template-wizardry the philosophy §6 declines. No basket program expresses "in
  general" vs "for this type." Refused; consequence: at most **one impl of a given
  interface for a given type** (§2.3), which makes coherence a lookup, not a solver.
- **No higher-kinded types.** Abstracting over a type *constructor* (`F[_]`) is a
  large, verification-hostile jump against P2 and has no basket case. Refused.
- **No const generics beyond array sizes.** `[N]T`'s compile-time length is the
  *entire* const-parametricity the memory model needs (0001 §5.1). General const
  parameters have no case and entangle comptime values with public signatures (P6).
  Refused; `[N]T` stays built-in syntax, the sole compile-time-constant parameter.
- **No associated types (this edition) — argued from the one case that wants them.**
  Exactly one construct demands an associated type: iteration (`Iterator.Item`).
  Iteration is deferred (§7.2) because it *also* needs iteration over borrowed items,
  colliding with §3.5. The case that would make associated types basket-grade is
  itself undesigned, so adding them now is speculation. Refused; re-opened with
  iteration (OBL-GENERICS-ITER). Methods may still take/return already-bound
  parameters; they simply introduce no new type member.
- **No interface inheritance / super-interfaces.** `interface B: A` turns bound
  checking into a graph walk and re-grows a hierarchy P6 does without. A two-
  capability bound writes both (`[T: Ord + Hash]`). Refused; re-open on measured
  redundancy.
- **No blanket impls.** `impl I for all T` makes "does `T` implement `I`?" a global
  inference question. No basket case, and its absence is load-bearing for §2.3.
  Refused.
- **No effect polymorphism.** A generic's `alloc`-ness is not a type-parameterized
  effect variable (§4.2 pays conservatism instead); parameterizing the effect over
  `T` re-grows the coloring machinery P2 kept to one closed, non-transforming
  effect. Refused.
- **No type parameter over a borrow type.** A type argument ranges over owned/value
  and `rawptr` types, never `read T` / `write T` / `[T]` / `write [T]` — 0001 §3.4
  ("borrows are for passing and computing, never for storing") lifted to the generic
  layer (§3.5). Refused.

### 1.2 The minimum viable bound system

- **The construct is `interface`** (the P13 naming call, §6.4): a named set of
  **method signatures** — no associated types, no associated constants, no default
  bodies (a default body is private codegen, comptime's job, not public generics).
- **A method signature carries everything a caller must know** (P2): parameter modes
  (`take`/`read`/`write`/`out`), return type, region variables where a borrow is
  returned (0001 §3.3), and its **effect marker** (`alloc` or absence, §4.1).
  Interfaces never lie any more than `fn`s do.
- **Bounds are written `[T: I]`**, conjoined with `+` (`[T: Ord + copy]`, §6.4). An
  unbounded parameter is fully **opaque**: movable, droppable, borrowable, storable,
  nothing else (§3).
- **One built-in bound, `copy`** (§3.1): a structural capability, not an interface,
  spelled in bound position for uniformity.
- **Generic items:** `fn`, `struct`, `enum`, and `interface` may declare a bracketed
  type-parameter list (§6). An interface may be generic (`From[E]`, §7.1) — a type
  *parameter*, still no associated *type*.

Everything below is how this surface interacts with what already exists.

---

## 2. Definition-site checking and coherence

### 2.1 The NN#10 guarantee, precisely

A generic is type-checked completely at its definition, against its bounds and
nothing else. Inside `fn f[T: I](x: T)` the only operations on a `T`-value are (a)
the memory-model operations every type supports — move, `copy` iff `T: copy`, drop,
`read`/`write`-borrow, store, pattern-bind through a borrow — and (b) the methods `I`
declares. Calling an undeclared method, or reading a field of an opaque `T` (it has
no *known* fields), is a **definition-site error**, reported once against the
generic's own text.

**Instantiation therefore never re-checks the body.** Proven well-typed for *every*
conforming type, the body cannot surface a new type error when a concrete conforming
type is substituted — the anti-C++-template property.

**What instantiation checks, and why it is not a violation.** Instantiating `f[u8]`
checks one thing: *does the argument satisfy the declared bounds?* This is **bound
conformance** — a property of the *argument*, decidable from the *callee's signature
alone* (P2), with the impl found locally (§2.3). A conformance failure is a use-site
error attributable to the caller's argument, categorically like passing a `bool`
where a `u8` is demanded — never a spelunk into the generic's body. NN#10
("instantiation can never produce a type error") means precisely *no body-internal
error at instantiation*; a bad argument against a visible bound is the caller's
local error. This distinction is the whole reconciliation and is stated so it is not
later blurred.

### 2.2 Body-local inference of type arguments is allowed

Locality is defined at the *signature line* (P2). Inferring a call's type arguments
from its value arguments inside a body is ordinary body-local inference and is
encouraged: `push(list, x)` infers `T` from `x`. No inference crosses a signature —
the callee's bounds and the argument's type are both local. Explicit type arguments
are needed only when `T` appears in no value parameter (the `ptr_null[T]()` shape);
§6.2 decides where those may be written.

### 2.3 Coherence — the orphan rule, minus the machinery

An **impl** attaches an interface to a type: `impl I for MyType { … }`. The single
rule:

> An `impl I for T` may be declared **only in the module that defines `T`, or the
> module that defines `I`** (the standard orphan rule).

**For a generic interface, the placement referent is the interface's *declaration*
module** (where `interface I[…]` is written), independent of any instantiation: an
`impl From[E1] for T` may be declared in `T`'s module or in `From`'s declaration
module, whatever `E1` is. **The uniqueness key, by contrast, is the *instantiated*
interface** `(I[args…], T)` (next bullet) — placement keyed on the interface's
declaration, coherence on its instantiation.

Why this suffices without Rust's coherence engine:

- **Finding an impl is a two-place lookup, not a search.** Both candidate homes are
  named by the type expression itself and reachable in the module DAG (P20) — no
  global scan; coherence is per-module over explicit names (P20 invalidation intact).
- **Uniqueness is free** because there are no blanket or overlapping impls (§1.1):
  each `(I[args…], T)` pair has at most one impl — the coherence key is the
  **instantiated** interface, so `impl From[E1] for T` and `impl From[E3] for T`
  coexist (distinct keys), while a second `impl From[E1] for T` is a duplicate and a
  compile error, and there
  is **no impl lattice, no negative reasoning, no specialization order** — the three
  pieces that make Rust coherence heavy, all absent because §1.1 refused what needs
  them.
- **No orphan means no cross-crate incoherence:** two unrelated libraries cannot
  both impl `I` for `T` unless one defines `I` or `T`, and then the other (owning
  neither) cannot — so the "two crates, conflicting impls, clash at link" hazard
  cannot arise.

This is the smallest coherence that makes NN#10 hold: definition-site checking is
sound only if "T satisfies I" has a single, locally findable answer, and the orphan
rule delivers exactly that and no more.

---

## 3. The memory-model interaction (the careful part)

Governing principle: for each analysis of 0001, an opaque `T` takes that analysis's
**most conservative rule**, and a bound *relaxes* it only when the bound *proves* the
relaxation sound for all conforming instantiations. Each analysis in turn.

### 3.1 Copyability — non-`copy` default, `copy` opt-in

An opaque `T` is **not `copy`** (0001 §1.3): every use of a `T`-place is a **move**,
duplication needs `clone` (not available generically this edition — the basket needs
no generic `clone`). Sound, because move-only discipline is valid for every type
(treating a copyable value as move-only under-approximates its capability).

`T: copy` **relaxes** it: uses become copies, and every instantiation must supply a
`copy` type (a bound-conformance check, §2.1). This is the arena's need made precise
(§8.2). `copy` is the only capability whose *presence* changes body-checking
outcomes, which is why it is the one built-in bound.

### 3.2 Move checking

Uniform with 0001 §1.6 over opaque `T`: move on use (unless `copy`), source
uninitialized after, partial-move and join-agreement rules as written. An opaque `T`
is a **single indivisible place** — no namable fields (§2.1), so no partial-move
*into* it and no field-disjointness reasoning on it. Strictly simpler than the
concrete case, and sound (fewer disjointness facts under-approximates the loans).

### 3.3 Drop scheduling

An opaque `T` is assumed **needs-drop** (0001 §1.6 rule 3 / §7.4): it could be a
drop-hooked or `Box`-bearing type, so a live `T`-local at scope exit is scheduled for
drop, and its init state must be path-independent at drop points (E0309). Sound and
cheap: a drop-inert instantiation monomorphizes the scheduled drop to a no-op —
assuming needs-drop only ever over-schedules nothing, never under-schedules a real
destructor. `T: copy` relaxes it (`copy` ⟹ no `drop` hook, cannot bear a `Box`, 0001
§1.3 ⟹ drop-inert, and E0309 lifts for that `T`).

### 3.4 Alloc-on-drop of an opaque `T` (the sharpest interaction)

`Box`-bearing and alloc-on-drop types propagate `alloc` through their *drop* (0001
§6.2/§6.3). An opaque `T` could be alloc-on-drop, so soundness for every
instantiation demands the conservative rule — and this design takes the one that does
**not** grow the effect system:

> A generic function that **owns a `T` and lets it drop** is `alloc`-marked — 0001
> §6.2's upper-bound rule applied to an opaque owner. A generic that only *borrows*
> `T`, or *moves `T` out* (returns/forwards/stores-into-a-returned-aggregate) without
> dropping it, is **not** alloc-marked.

This never under-states (NN#19) and grows nothing: it is the existing "a function
that receives a `Box` and lets it die is `alloc`-marked" rule with "an opaque `T` we
cannot prove is not `Box`-like" in the `Box`'s place. **The cost, named:** at a
drop-inert instantiation (`u8`) the marker over-states (permitted conservatism, P2 —
effects are upper bounds) and bars that instance from the no-alloc ground floor.
Blast radius is small: a container's own `drop` already frees its `Box` backing and
is `alloc`-marked regardless of `T`, so `T`'s drop-effect rides inside an already-
`alloc` op; the marker bites only a pure stack aggregate that owns-and-drops a bare
`T`. This marker is fixed **once at the definition** and
stands for every instance; it is **not** re-derived per concrete type at codegen
(§5.2).

Rejected: **effect polymorphism** (a per-instantiation `alloc` variable so
`Container[u8]::drop` is provably non-alloc). Refused (§1.1) — it re-grows the closed-
effect/coloring machinery P2 fought to keep minimal, for a tax §8.2 shows is bounded.
Recorded as the escape hatch if the basket measures the tax as real (OBL-GENERIC-
EFFECT).

### 3.5 Borrows of `T`, and `T` in valve positions

- **Borrows of opaque `T`** (`read T`, `write T`) are checked exactly as borrows of
  any nominal type: XOR (0001 §2.2), NLL-lite live ranges (§2.3), reborrows, region
  provenance (§3.3) — all at whole-place granularity (no field-disjointness, §3.2).
  A generic returning a borrow of a borrowed `T` parameter uses the same region
  variables and compact default as any function (0001 §3.3).
- **Interface methods returning a borrow: `self` is the sole borrow-in (F9).** For
  an interface method that returns a borrow, 0001 §3.3's compact default counts
  **`self` as the one borrow-in**, so `fn first(read self) -> read Elem` needs no
  region variable and returns a borrow of `self`. A method that adds a second
  borrow parameter alongside `self` must declare region variables (no default). An
  **impl's method must match the interface's region signature exactly** — the same
  region variables in the same positions — because a caller reasons from the
  interface's declared regions (P2); an impl that renamed or widened them would lie
  about provenance.
- **A type argument never ranges over a borrow type.** A generic `[T]` binds an
  owned/value or `rawptr` type — never `read U`, `write U`, `[U]`, `write [U]`. This
  is 0001 §3.4 (no borrow-typed fields, to avoid region parameters on types) lifted
  intact: `List[read U]` would put a borrow in a stored field and force a region
  parameter onto `List` and every signature mentioning it — the machinery whose
  absence *is* the value-first bet. The checker enforces this at instantiation as an
  ordinary bound-conformance failure (§2.1): "a borrow type is not a legal type
  argument."
- **`rawptr T` reconciled.** `rawptr T` (0001 §4.2) is the *pre-existing* generic — a
  compiler-known type parametric over an opaque element, its parameter used only
  inside valves. User generics generalize the *pattern*, but `rawptr` stays compiler-
  known (§6.3): its ops are unsafe intrinsics that bit-copy `T`, not methods on a
  user interface. A generic may hold `rawptr T` (inert in safe code) and act on it in
  `unsafe`, exactly as a concrete type does — the valve discipline is unchanged by
  genericity.

---

## 4. Effects

### 4.1 Interface method signatures carry effect markers — they must

Signatures never understate (P2/NN#19). A method that may allocate is declared
`alloc`:

```
interface Sink {
    fn push(write self, v: u8) alloc -> unit    // may allocate (grow a buffer)
}
```

A generic bounded `T: Sink` calling `x.push(b)` **inherits the method's declared
effect**: the caller is `alloc`-marked, or the call is a definition-site effect error
if it is unmarked. A non-`alloc` method may not allocate in *any* impl (an allocating
body under an unmarked method is a definition-site error on the impl — the same
upper-bound rule). This is the one-way partition (0001 §3.2) carried through the
bound: the ground floor stays callable from everywhere because a non-`alloc` bound
method is callable from non-`alloc` generic code.

### 4.2 Composition with fn-pointers and the drop rule

Fn-pointer types already carry parameter modes *and* the effect marker (0001 §6.1);
interface methods carry the same, and the two compose without a special case: an impl
method installed behind a vtable field keeps its marker, and an indirect call takes
its effect from the pointer's type (0001 §6.1) — there is no "interface-dispatch"
effect rule distinct from the fn-pointer rule. The alloc-on-drop-of-`T` rule (§3.4)
is the *only* effect subtlety generics add, and it is a conservative application of an
existing rule, not a new one.

---

## 5. Instantiation strategy (P11's second half)

### 5.1 Monomorphization is the deterministic documented default

The default and only strategy this edition is **monomorphization**: each distinct
type-argument tuple produces a distinct fully-concrete instance. Deterministic and
documented per target (P11): a reader predicts from the source exactly which
instances exist and that each carries the concrete cost of the substituted types — no
shared-code indirection, no hidden dispatch, no cliff the programmer cannot see
(Priority 4 applied to the compiler's own strategy, P11).

### 5.1.1 Polymorphic recursion and the termination backstop (F2, F10)

Monomorphization is a fixed point over the instantiations a program reaches. A
generic that instantiates **itself with a strictly larger type argument** —
`fn f[T](x: T) { … f(wrap(x)) … }`, whose recursive call *infers* the type argument
`Wrap[T]` (§6.2, no call-site turbofish) — has no fixed point: the instantiation set
is infinite. Two cases, kept honest:

- **Decidable direct self-instantiation is a definition-site error.** When a
  generic's body directly instantiates itself (or a mutually-recursive partner)
  with a syntactically **growing** argument — the argument nests the parameter under
  a type constructor, `T ↦ Wrap[T]` — non-termination is decidable from the body
  alone and is reported **once, at the definition site**, never surfaced at
  instantiation. NN#10's letter holds.
- **The undecidable remainder is backstopped by a documented depth limit.**
  Indirect or value-dependent instantiation chains are not decidable in general, so
  a **deterministic, documented monomorphization depth limit** halts them. This is
  classified honestly as a **compile *resource* limit**, not a type error: crossing
  it aborts the build with a resource diagnostic naming the instantiation stack —
  the same category as exhausting memory, **not** a TYPE error at instantiation, so
  NN#10's letter ("instantiation produces no body-internal type error") is
  preserved. **The residual, named:** the depth limit is a deterministic-but-real
  compile cliff for the undecidable tail; NN#10's *spirit* (no invisible
  instantiation cliff) is honored by determinism and documentation, but a resource
  cliff is a resource cliff, recorded as such rather than claimed away.

**`sizeof(T)` is discharged by scope (F10).** An opaque `T` has no known size at the
definition site, so `sizeof(T)` is not a definition-site compile-time constant — but
the prototype **does not implement generics at all**, so no `sizeof(T)` over an
opaque `T` is written in prototype code today. When generics land, `sizeof(T)` is
legal only **post-monomorphization**, where `T` is concrete and its size is known;
at the definition site it may not be used where a compile-time constant is required.
No prototype-scope change is needed; the obligation is scoped forward with generics.

### 5.2 The P20 check-once / instantiate-cached story

Definition-site checking is the compile-speed architecture P20 names: the body is
type-, move-, loan-, and effect-checked **exactly once**, at its definition, against
its bounds. Instantiation is **codegen, never re-analysis** — substitute concrete
types, emit or reuse a cache keyed by the instantiation tuple within a module-DAG-
invalidated build. A signature change rechecks; a body change re-codegens its
instances but re-checks nothing downstream (P20 signature-bounded invalidation). An
instance's `alloc`-ness is **not** resolved per-instance at codegen: §3.4 fixes
generic drop-glue alloc-ness **once, conservatively, at the definition site** (a
`T`-dropping generic is `alloc`-marked for *every* instance, including drop-inert
ones such as `Wrap[u8]`). Effects are therefore **never re-derived at codegen** —
codegen only lowers the already-resolved def-site effect to machine code, so it
never reopens the body. The check-once architecture is total: types, moves, loans,
*and effects* are all settled at the definition.

### 5.3 The source-level override — deferred, obligation recorded

P11 requires the strategy be *overridable* "where cost control demands" (shared-code /
dictionary-passing to bound monomorphization bloat). No basket program shows that
code-size pressure, and shared-code polymorphism is substantial machinery with P2
implications (its cost is *less* locally predictable). P11's text requires the
*default* be deterministic-and-documented (it is) and override exist *where cost
demands* — so it is **deferred with a recorded trigger**, not dropped on principle:

> **OBL-GENERIC-STRATEGY.** The source-level instantiation-strategy override (P11) is
> deferred. Trigger: a measured code-size regression on the basket (or its stdlib
> successor) attributable to monomorphization bloat. Until then the default is the
> only strategy, satisfying P11's determinism clause; the override clause is a
> standing obligation, not a silent drop.

---

## 6. Syntax under the no-`<>` constraint

### 6.1 The delimiter: square brackets `[…]`

Type parameters and arguments are delimited by **`[…]`**: `fn id[T](x: T)`,
`struct List[T] { … }`, `enum Result[T, E] { … }`, bound `[T: Ord + copy]`. This
satisfies **OBL-GENERIC-BRACKET** (not `<>`, so `>>` stays the shift operator 0006
added) and it is **not new**: the prototype already spells explicit type arguments
with brackets on its compiler-known intrinsics — `ptr_null[T]()`, `cast_ptr[U](p)`,
`addr_to_ptr[T](a)` (0001 §4.2), retained by 0006. Reusing that bracket for user
generics is P3 (one spelling for "type arguments"), not a Rust reflex.

### 6.1.1 Declaration lists mix region and type parameters — keyword-disambiguated (F1, F11)

Before generics, a bracket after a function name already declared **region
variables** (0001 §3.3: `fn pick[r](…)`); this design now also declares **type
parameters** in that same bracket (`fn id[T]`). Bare `[r]` and `[T]` are lexically
identical, so a mixed list needs a rule. Two numbered rules govern every generic
bracket:

1. **Declaration vs. use (the F11 rule, stated explicitly).** A bracket
   **immediately following an item's *name* in its declaration** — `fn choose[…]`,
   `struct List[…]`, `enum Result[…]`, `interface From[…]` — is a
   **parameter-declaration** list that *introduces* binders. A bracket following a
   **name in type or expression position** — `List[u8]`, `From[E1]`, `Box[T]` — is a
   **use** list that *applies* arguments. Declaration introduces, use applies; the
   defining-occurrence-vs-referring-occurrence position decides, with no symbol table
   (§6.2's leading-vs-following machinery handles the use side).
2. **Region vs. type parameter inside a declaration list.** A parameter written
   **`region r`** declares a **region variable** (0001 §3.3); a **bare identifier**
   `T` declares a **type parameter**. Regions always wear the `region` keyword; a
   bare bracketed identifier is *never* a region. So

   ```
   fn choose[region r, T](a: read[r] T, b: read[r] T, pick_a: bool) -> read[r] T
   ```

   declares one region variable `r` and one type parameter `T`, and the returned
   borrow's provenance is `r` (explicit, per 0001 §3.3).
3. **Compact-default trigger restated as "no region declarations."** 0001 §3.3's
   compact default (a lone borrow-in/borrow-out needs no region variable) triggers
   when the declaration list declares **no `region` parameter**. Type parameters do
   **not** suppress it: `fn first[T](s: read T) -> read T` keeps the compact default
   and needs no region. Only a `region` declaration turns the default off, and then
   every returned-borrow provenance is written — the rare signature carries the
   annotation weight (P12).

### 6.2 NN#13 — the grammar still parses without a symbol table

- **Type position — clean, by leading-vs-following bracket.** A **leading** `[` in
  type position is a slice/array (`[T]`, `[N]T` — 0001 §5, 0006 §2.2). A bracket
  **following a type-name token** is a type-argument list: `List[T]`, `Map[K, V]`,
  `Box[T]`. They never compete — one leads, one follows — so a one-token check ("did
  a type-name precede this `[`?") decides, no symbol table. Nesting closes `]]`,
  never `>>`, so no shift hazard exists (the point of the constraint). The parser
  need not know whether `List` *is* generic — `Name[…]` is uniformly "apply type
  arguments to `Name`"; arity is a checker fact (NN#13-clean, as `Enum::Variant` is
  position-resolved, 0001 §8.2).
- **Expression position — the indexing collision, resolved by refusing call-site
  turbofish.** `foo[i]` is **indexing**, and `foo[T](x)` (explicit type args) is
  lexically identical — `IDENT [ … ] ( … )` also matches "index `foo`, call the
  result," legal when `foo[i]` holds a fn-pointer. Choosing between them needs to
  know whether the bracket holds a *type* or a *value* (`foo[Bar]`: type or
  variable?) — a symbol-table question NN#13 forbids. This design **removes the
  ambiguous construct**: a user generic call takes **no explicit type-argument
  bracket at the call site**. Where the parameters appear in value arguments they are
  inferred body-locally (§2.2); where they do not (the `ptr_null[T]` shape) the type
  is written in **type position** — a binding annotation (`let p: List[u8] = new();`)
  or return type — which is unambiguous by the leading-vs-following rule. The
  intrinsics keep their bracket because they are **keyword-led** (`cast_ptr`,
  `ptr_null`, … are reserved tokens the parser knows take a type argument), so no
  user-identifier ambiguity arises. This is a deliberate expressiveness cut
  (contestable-call #1): small (a type-annotated binding always covers it) and the
  price of a symbol-table-free grammar without `<>` or reopening `::` (variant-
  reserved). If the successor basket shows inference + annotations cannot cover a
  case, a keyword-led turbofish is the recorded re-open (OBL-GENERIC-TURBOFISH).

### 6.2.1 Naming a generic function as a *value* — `name::[T]` (F3)

§6.2 removes the call-site type-argument bracket, but a generic function used **as a
value** — not called — still needs to name *one* instantiation: a `let` binding of a
function, a fn-pointer slot, and a vtable field each store a concrete function, and a
generic is not one until its type arguments are fixed. That instantiation is spelled
with the **`::[T]`** digraph:

```
let f: fn(u8) -> u8 = id::[u8];                    // let-binding a generic fn as a value
let slot: fn(read Node) -> u32 = weigh::[Node];    // fn-pointer slot
```

- **The `::[` digraph is unambiguous (NN#13-clean).** `::` is already the reserved
  path separator (0008 §3), never an expression operator, so `name::[` cannot be read
  as "index `name`" (that is `name[`) nor as a path segment (`::` here is followed by
  `[`, not an identifier or `{`). A one-token check after `::` — is the next token
  `[`? — separates the generic-value spelling from a module path, no symbol table.
- **This is *not* the call-site turbofish §6.2 refused.** It appears only in **value**
  positions (binding, pointer/vtable install), never on a call, so the
  `foo[i]`-indexing collision cannot arise.
- **It covers let-bindings, fn-pointer slots, and vtable fields uniformly:** installing
  an impl method or a generic function behind a vtable slot names the instantiation
  with `::[…]`, and the stored value is an ordinary concrete fn-pointer thereafter (its
  effect marker rides on the pointer type, §4.2). Rare, keyword-free, NN#13-clean.

### 6.3 How the compiler-known parametric types migrate

| Type | This edition | Disposition |
|---|---|---|
| `[T]` / `write [T]` slice | built-in syntax | **stays built-in** — special grammar, region tags `read[r] [T]`, ground-floor primitive the memory model is defined over |
| `[N]T` array | built-in syntax | **stays built-in** — `N` is the sole const parameter (§1.1) |
| `rawptr T` | compiler-known | **stays compiler-known** — ops are unsafe intrinsics, not interface methods (§3.5) |
| `Box T` → **`Box[T]`** | compiler-known | **stays compiler-known**, re-spelled `Box[T]`. Deref-to-pointee and `box`/`unbox`/clone stay compiler-blessed; a user `Deref`/`Clone` interface is the expressive growth P11 refuses by default |
| `BoxResult T` → **`BoxResult[T]`** | compiler-known sum | **becomes an ordinary library generic enum** — `enum BoxResult[T] { ok Boxed(Box[T]), OutOfMemory }`. No special powers (a plain result-shaped sum; 0006's `ok`-marker `?` already operates on it generically), so it needs no compiler blessing once user generics exist |

**Stdlib shape this determines:** slices, arrays, `rawptr`, and `Box` stay in the
compiler; `BoxResult` and every future container (`List`, `Map`, `Arena[T]`, §8.1)
become library generics. `Box` is the deliberate hold-back — the one owning
abstraction whose special powers are worth compiler-knowledge over a `Deref` interface
this edition.

### 6.4 The construct name and bound spelling (P13 calls)

- **`interface`, not `trait`.** Same argument 0006 used to reject `as`: a token that
  imports a false prior is a Bet 6 hazard. "trait" carries the Rust prior of
  associated types, blanket impls, coherence lattices, default bodies — *precisely
  the machinery §1.1 refuses*. "interface" signals "a named set of method signatures
  a type satisfies," which is what ships, and P11 itself already says "interface
  bounds." Chosen.
- **`[T: I]` and `[T: A + B]`.** One bound spelling: `:` introduces the bound, `+`
  conjoins capabilities on one parameter; **no `where` clause** (a second spelling
  for the same fact, against P3). The formatter (NN#11) emits `+` for multi-bound
  parameters, the comma form only to separate distinct parameters (`[K: Hash, V]`).

---

## 7. What this unblocks, and what stays deferred

### 7.1 Cross-type `?` — designed here (the flagship unblock)

0006 §2.4 shipped same-type `?` and parked cross-type propagation on "needs a `From`-
style trait — P11." It is now expressible with a single-method generic interface:

```
interface From[E] {
    fn from(e: E) -> Self          // Self = the implementing type; not an associated type
}
```

`Self` names the impl target in a method signature (the one new piece of vocabulary,
resolved at the impl — not an associated type, §1.1). Cross-type `?` desugars: for
`expr?` where `expr` has result-shaped `R1` (0006's `ok`-marker) with non-`ok`
payload `E1`, inside a function returning result-shaped `R2` with non-`ok` payload
`E2 ≠ E1`, the operator returns `R2`'s non-`ok` variant built from `E2::from(e1)` —
**provided `impl From[E1] for E2` exists**, found by the §2.3 orphan rule (in `E2`'s
module, which the error-defining crate owns). No impl ⟹ a definition-site error at
the `?` site, attributable locally. Same-type `?` (0006) is the `E1 = E2` case,
needing no impl. This is the entire cross-type story, and it grows the interface
system by nothing (one library interface).

**Coherence over the *instantiated* interface (F4).** The coherence key is
`(From[E1], E2)` — the *instantiated* interface, not the bare `From` (§2.3) — so a
single error type absorbs several sources without clash:

```
impl From[IoError]    for AppError { fn from(e: IoError)    -> Self { … } }
impl From[ParseError] for AppError { fn from(e: ParseError) -> Self { … } }
```

These are two distinct keys, both legal in `AppError`'s module (which owns the
placement referent — `From`'s *declaration* lives in `core`, `AppError`'s module owns
`AppError`, §2.3). `expr?` selects the impl whose source type matches `expr`'s
non-`ok` payload. A second `impl From[IoError] for AppError` would be a duplicate key
and a compile error; `From[IoError]` and `From[ParseError]` never are.

**Cross-type `?` inherits `From::from`'s declared effect — disclosed (F7).** `expr?`
desugars to a **call** to `E2::from(e1)`, so it inherits `From::from`'s **declared**
effect marker exactly as any bounded method call does (§4.1/§4.2). The marker is the
interface's, an upper bound uniform across impls (§4.1): the non-`alloc` signature
above makes every cross-type `?` non-`alloc`; an **allocation-permitting** error
interface instead declares

```
interface From[E] { fn from(e: E) alloc -> Self }   // conversions may allocate
```

and then a cross-type `?` is an **allocating** operation for the caller, so using it
in an **unmarked** function is a **definition-site effect error**:

```
fn load(s: read [u8]) -> AppResult {     // unmarked: no alloc
    let n = decode(s)?;                   // ERROR at ?: from() is alloc, load is not
    ok(n)                                //   (def-site effect error, reported here)
}
```

Mark `load` `alloc` to fix it — identical to a direct `x.from(...)` call. Same-type
`?` (0006, `E1 = E2`) calls no `from` and inherits no effect. `?` is sugar over a
method call; the call's effect is the method's, nothing added.

### 7.2 `for` / iteration — deferred to the stdlib round (with the reason)

Iteration needs an `Iterator` interface, which needs (a) an **associated type**
`Item` (§1.1 refuses this edition) and (b) iteration over **borrowed** items,
colliding with §3.5 ("no type parameter over a borrow"). Both are real, both must not
be prejudged, and 0006 already deferred `for` on the "iterators need generics" ground.
Deferred; recorded as **OBL-GENERICS-ITER** — the round that re-opens associated types
and the borrowed-item question together, with iteration as the basket-grade case that
finally makes them non-speculative.

### 7.3 The P3 text budget — deferred (unchanged)

Owning/interop text forms beyond `[u8]` (OBL-TEXT) are a P3 call, not a generics one.
Generics *enable* a library owning-string type (over `Box`-of-bytes), but which forms
exist and why is OBL-TEXT's budget. Deferred, unchanged; this document only removes
the "no generics" blocker.

### 7.4 User containers and operator overloading

User generic containers are unblocked (§8.1). Operator overloading (`Add`/`Index`
interfaces) stays **deferred with `for`** — the same "which interfaces does the stdlib
bless" question, no basket case beyond iteration/indexing; reopening it here is
speculative growth (P6). Recorded under OBL-GENERICS-ITER's scope.

---

## 8. Worked examples, rejected alternatives, and costs

### 8.1 The parser's `Args` cons list becomes `List[T]`

The clearest no-generics pressure (parser README note 4): call-argument lists are a
**hand-rolled per-element-type cons list** `enum Args { Nil, Cons(Box Ast, Box Args) }`
with its own `must_box_args` helper and a *duplicated* recursive walk
`arg_leaf_spans`. Under this design, one generic type:

```
enum List[T] { Nil, Cons(Box[T], Box[List[T]]) }

fn push_front[T](a: read Alloc, head: T, tail: Box[List[T]]) alloc
        -> BoxResult[List[T]] {
    box(a, List::Cons(head, tail))         // T moves in; Box[T], Box[List[T]] owned
}
```

`List[Ast]` replaces `Args` and every future per-type list. `push_front` is `alloc`-
marked because it calls `box` (unconditional, §4.1) — *not* by §3.4 (it moves `head`
in and drops no `T`). The duplicated walk collapses into one generic walk. The
definition-site check runs once on `List[T]`; `List[Ast]` is codegen (§5.2). NN#10
holds — nothing in `List`'s body can fail for `T = Ast`.

### 8.2 A generic arena `Arena[T]`, and the `copy` bound

The arena port (README note 6) is pure value+index gear *because* `Node` is `copy`:
"`arena_get` returns an owned copy — no borrow returns, no region annotations."
Generalized:

```
struct Arena[T: copy] { mem: Box[[4096]T], count: u32 }   // T: copy is load-bearing

fn arena_get[T: copy](ar: read Arena[T], i: u32) -> T {   // returns an owned copy
    ar.mem[conv usize i]                                   // copy out; no borrow, no region
}
```

`T: copy` is exactly §3.1: it relaxes the opaque default so `arena_get` copies the
element out and returns by value — no returned borrow, no region variable, matching
the port's shape. `Arena[T]`'s own `drop` frees its `Box` and is `alloc`-marked
regardless of `T`; the §3.4 opaque-drop tax does **not** additionally bite, because
`T: copy` ⟹ drop-inert (§3.3). The generic arena over a `copy` node carries no
ground-floor surprise — the copyability-bound design paying off where the basket felt
it.

### 8.3 The `AllocVtable` stays concrete (the deliberate non-migration)

`Alloc`/`AllocVtable` (0001 §6.1) *could* become `interface Allocator { … }` with
`fn f[A: Allocator](a: A)`. It is **not** migrated, on direct philosophy authority
(P2): "allocator-as-parameter as the single polymorphism mechanism (no dual trait
hierarchies) … the capability travels as an ordinary value … not a function-type
transformation." An `Allocator` bound would monomorphize a separate copy of every
allocating function per allocator type — the trait-hierarchy fan-out P2 names as the
no_std ecosystem-gravity risk. The `Alloc` **copy value handle** (a struct of `alloc`-
typed fn-pointers) is dynamic dispatch *by value*: one type, callable everywhere,
storable in a `Box`, threaded as a parameter — the ground floor stays one type wide.
Generics **coexist with** the concrete handle rather than subsuming it; this is the one
place the design chooses a value-vtable over a bound *on principle*, recorded as such.

### 8.4 Rejected alternatives (cross-referenced to their arguments)

- **`<>` brackets — rejected** (§6.1, OBL-GENERIC-BRACKET/NN#13): reintroduces the
  C++ `>>`-vs-close-angle parse hazard. Square brackets chosen.
- **Call-site turbofish for user generics — rejected** (§6.2): indexing collision,
  unresolvable without a symbol table. Inference + annotations cover it (OBL-GENERIC-
  TURBOFISH re-open).
- **`trait` naming — rejected** (§6.4): imports the Rust prior of the refused
  machinery (Bet 6 hazard). `interface` chosen.
- **Effect polymorphism — rejected** (§3.4): re-grows coloring machinery for a bounded
  tax. Upper-bound conservatism chosen (OBL-GENERIC-EFFECT re-open).
- **Type parameters over borrow types — rejected** (§3.5): drags region parameters
  onto every generic type, the value-first bet's antithesis.
- **Associated types this edition — rejected** (§1.1): the demanding case (iteration)
  is deferred, so they are speculation now (OBL-GENERICS-ITER re-open).
- **Specialization, blanket impls, HKT, super-interfaces, general const generics —
  rejected** (§1.1): coherence- or verification-heavy machinery with no basket
  pressure.
- **Migrating `Box` to a `Deref`/`Clone`-interface library type — rejected** (§6.3):
  a `Deref` interface is the growth P11 refuses; `Box` stays compiler-known,
  `BoxResult` becomes a library generic.
- **An `interface Allocator` bound — rejected** (§8.3): the concrete `Alloc` value
  handle is P2's mandated single allocator-polymorphism mechanism.

### 8.5 Consequences and costs (debts, not absolutions — the header warning)

- **The opaque-drop `alloc` tax** (§3.4): generic stack aggregates owning-and-dropping
  a bare `T` are `alloc`-marked even at drop-inert instantiations. Bounded, named, not
  dissolved.
- **No call-site turbofish** (§6.2): an inference-defeating, annotation-awkward case is
  unwritable until OBL-GENERIC-TURBOFISH.
- **No generic numeric `conv`** (§8, F8): `conv`'s target is a **scalar-type
  keyword** (0006 §6), never a type parameter, and there is **no `Num`/arithmetic
  bound** to make an opaque `T` numeric (the same refusal as the no-`Num`-bound
  cut). So `conv T x` over an opaque `T` is **unwritable by design** — generic code
  cannot convert between an abstract `T` and a scalar. Recorded, consistent with the
  bound system's refusal to grow an arithmetic hierarchy.
- **No generic over borrows** (§3.5): no abstraction over `read U` items — a real cut
  that also blocks part of iteration (§7.2).
- **Monomorphization-only** (§5.3): code-size blowup has no override until OBL-GENERIC-
  STRATEGY; predictable cost is preserved, code size is not bounded.
- **The bound system is still real complexity** (P11's own concession): `interface`,
  impls, coherence, bound conformance, and the copyability/effect/loan interactions
  above are a meaningful fraction of what makes Rust's traits heavy. The claim is only
  that it is the *smallest coherent* such system for the basket — every §1.1 omission
  is a debt some future program may call in, by amendment, in the open.

**Stage-1 implementation rulings (2026-07-08, deciding authority, surfaced by the prototype):**
(1) §6.1.1's `region` keyword supersedes bare-`[r]` lists; the migrator emits `region r` and this
doc is the migration guidance. (2) Interface methods MAY be self-less associated functions —
`From::from` requires it; §1.2's self-carrying description is the common case, not a rule.
(3) Inference defaults: an unsuffixed integer literal argument infers `i64`; expected-type hints
from annotations resolve payload-less variant construction. (4) Interface-method call spelling is
receiver syntax `recv.method(args)`; self-less associated functions are invoked through their
interface path (stage 1: only internally by `?`).

**Stage-2 rulings (2026-07-08):** a generic impl's type parameters must appear in the target type
(E1016) — the only sound monomorphization driver absent use-site impl shapes; receiver lowering
for `recv.method()` follows the method's self mode — **`read self` / `write self` receivers borrow
the receiver place (they are not consumed); a `take self` method consumes the receiver place through
the same receiver syntax** (the earlier "owned receivers borrow, not consumed" wording is corrected:
it held only for read/write self, and `take self` is exactly the consuming case 0009's `Iter::next`
desugar depends on — see `docs/reviews/2026-07-08-design-0009-review-1.md` disp. 4); generic result
enums keep their `ok` marker through substitution.
