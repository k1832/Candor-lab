# 10 — Generics and Interface Bounds

**Status: NORMATIVE-DRAFT.** Transcription of design `0007-generics-and-bounds`
(the `interface`/bound system, definition-site checking, coherence, and the
instantiation strategy), with the single associated-type member reopened by
design `0009` §2 (its projection and iteration use are chapter 12). Rationale is
in designs 0007/0009; this chapter states rules only, in **clauses**, not error
codes (the non-normative appendix maps them). **Discharges OBL-GENERICS** and
**satisfies OBL-GENERIC-BRACKET** (chapter 99).

---

## 1. Generic items and parameter lists

1.1 The generic items are `fn`, `struct`, `enum`, and `interface`; each MAY
    declare a bracketed parameter list **immediately after its name**
    (`fn id[T]`, `struct List[T]`, `enum Result[T, E]`, `interface From[E]`). The
    type-argument delimiter SHALL be `[…]`; `<…>` SHALL NOT be one, so `>>` stays
    the shift operator (NN#13; OBL-GENERIC-BRACKET; design 0007 §6.1).

1.2 **Declaration versus use.** A bracket immediately following an item's name in
    its **declaration** *introduces* binders; a bracket following a name in
    **type or expression position** (`List[u8]`, `Box[T]`) *applies* arguments.
    Position alone decides, with **no symbol table**; a use `Name[…]` is uniformly
    "apply arguments to `Name`", and `Name`'s genericity and arity are checker
    facts (design 0007 §6.1.1; NN#13).

1.3 **Region versus type parameter.** In a declaration list, `region r` declares
    a **region variable** (chapter 04 §7) and a **bare identifier** `T` declares a
    **type parameter**; a region SHALL always wear the `region` keyword, a bare
    bracketed identifier is never a region. A list MAY mix them
    (`fn choose[region r, T](a: read[r] T, b: read[r] T, …) -> read[r] T`). This
    `region` form **supersedes** the bare-`[r]` list of chapters 02 §4 / 04 §7 for
    generic items (design 0007 §6.1.1; the coordinated ch02/04 grammar update is a
    recorded gap, chapter 99).

1.4 **Compact default = no `region` declaration.** Chapter 04 §7.3's compact
    default applies iff the list declares no `region` parameter; type parameters
    do not suppress it (`fn first[T](s: read T) -> read T` needs no region). Only
    a `region` declaration turns it off (design 0007 §6.1.1).

---

## 2. Type parameters, opacity, and the `copy` bound

2.1 A **bound** is written `[T: I]`, conjoined with `+` (`[T: Ord + copy]`).
    There SHALL be **no `where` clause** (P3). The formatter emits `+` for
    multiple bounds and the comma only between distinct parameters (design 0007
    §6.4).

2.2 An **unbounded** type parameter is fully **opaque**: its values may be moved,
    dropped, `read`/`write`-borrowed, stored, and pattern-bound through a borrow,
    and nothing else. Reading a field of an opaque `T` or calling an undeclared
    method on it is a **definition-site error** (design 0007 §3, §2.1).

2.3 A type argument ranges over an owned/value or `rawptr` type — **never a
    borrow type**. `read U`, `write U`, `[U]`, `write [U]` SHALL NOT be type
    arguments (chapter 04 §8 lifted to the generic layer); the checker rejects
    this as a bound-conformance failure at instantiation (design 0007 §3.5).

2.4 **`copy` is the one built-in bound** — a structural capability spelled in
    bound position for uniformity, and the only capability whose presence changes
    body-checking (§5) (design 0007 §3.1).

---

## 3. Interfaces

3.1 An `interface` is a **named set of method signatures** with **at most one
    associated type**; it has no associated constants and no default bodies. The
    keyword SHALL be `interface`, never `trait` (design 0007 §1.2, §6.4; the
    associated type is design 0009 §2, its projection chapter 12).

3.2 A method signature carries **everything a caller must know** (P2): each
    parameter's mode, the return type, region variables where a borrow is
    returned, and its **effect marker** (`alloc` or its absence). It SHALL NOT
    understate its effect (NN#19).

3.3 **Self modes / receiver semantics.** A self-carrying method is invoked as
    `recv.method(args)`. A **`read self`** or **`write self`** receiver
    **borrows** the receiver place (not consumed); a **`take self`** receiver
    **consumes** it through the same syntax (design 0007 stage-2 ruling — the
    consuming case chapter 12's `Iter::next` depends on).

3.4 **Self-less associated functions.** A method MAY omit the self receiver
    (`From::from` requires it); it is invoked through its **interface path**, not
    a receiver (design 0007 stage-1 ruling 2/4).

3.5 **`Self`** names the impl target in a signature, resolved at the impl — a
    vocabulary word, not an associated type (design 0007 §7.1).

3.6 **Effect markers are uniform per impl.** A bounded `T: I` calling `x.m(…)`
    **inherits `m`'s declared marker** (caller is `alloc`-marked, else a
    definition-site effect error). A non-`alloc` interface method SHALL NOT
    allocate in **any** impl. The marker is the interface's, uniform across every
    impl (design 0007 §4.1; chapter 08).

3.7 **Borrow-returning interface methods.** Chapter 04 §7.3's compact default
    counts **`self` as the one borrow-in**, so `fn first(read self) -> read Elem`
    needs no region variable. A method adding a second borrow parameter alongside
    `self` SHALL declare region variables (design 0007 §3.5).

---

## 4. Impls and conformance

4.1 An **impl** attaches an interface to a type (`impl I for T`), optionally
    generic and bounded (`impl[T: I2] I for List[T]`). It is an ordinary module
    item under chapter 11's visibility and placement rules.

4.2 **Every generic impl type parameter SHALL appear in the target type** — the
    only sound monomorphization driver absent a use-site impl shape (design 0007
    stage-3 ruling; prototype `E1016`).

4.3 **Conformance axes.** Each interface method an impl provides SHALL, at the
    impl's **definition site**, carry the **same signature** as the interface's
    after substituting `Self` → target, interface type parameters → impl
    arguments, and associated type → impl binding, on **every** axis: (a) self
    receiver presence and mode; (b) parameter count; (c) each parameter mode;
    (d) each parameter type; (e) return type; (f) an **exact effect-marker match**
    (§3.6). Region conformance is subsumed by borrow-kind matching on (c)–(e)
    (regions are not independently declarable on interface/impl methods).
    Divergence on any axis is a definition-site error (design 0007 stage-3;
    prototype `E1021`–`E1026`).

4.4 **Extra impl methods are rejected** — one interface, one shape; this edition
    grows no inherent-method concept (design 0007 stage-3; prototype `E1014`).

---

## 5. The memory-model interaction (opaque conservatism)

5.1 **Governing rule.** For each chapter 03/04 analysis, an opaque `T` takes the
    **most conservative** rule; a bound relaxes it only when it proves the
    relaxation sound for **every** instantiation (design 0007 §3).

5.2 **Copyability.** An opaque `T` is **not `copy`** (uses move). `T: copy`
    relaxes it (uses copy) and every instantiation SHALL supply a `copy` type
    (design 0007 §3.1).

5.3 **Move / granularity.** Move checking is uniform with chapter 03 §7; an
    opaque `T` is a **single indivisible place** — no namable fields, so no
    partial move into it and no field-disjointness reasoning on it (design 0007
    §3.2).

5.4 **Drop scheduling.** An opaque `T` is assumed **needs-drop**: a live
    `T`-local at scope exit is scheduled for drop and its init state SHALL be
    path-independent at drop points (chapter 03 §7.5). `T: copy` relaxes it
    (`copy` ⟹ drop-inert) (design 0007 §3.3).

5.5 **Alloc-on-drop of an opaque `T`.** A generic that **owns a `T` and lets it
    drop** SHALL be `alloc`-marked (chapter 03 §6.2's upper bound over an owner
    that could be `Box`-bearing); one that only borrows `T`, or moves `T` out
    without dropping it, SHALL NOT be so marked. The marker is **fixed once,
    conservatively, at the definition site** for **every** instance (including
    drop-inert ones) and **never re-derived at codegen** (design 0007 §3.4, §5.2);
    the over-statement at drop-inert instances is permitted conservatism (chapter
    00 §3.3).

5.6 **Borrows of `T`** (`read T`, `write T`) are checked as borrows of any
    nominal type — XOR, NLL-lite ranges, reborrows, region provenance — all at
    whole-place granularity (design 0007 §3.5).

5.7 **`rawptr T` is unchanged**: it stays compiler-known; a generic MAY hold one
    (inert in safe code) and act on it only in a valve (design 0007 §3.5).

---

## 6. Definition-site checking

6.1 **The NN#10 guarantee.** A generic is type-, move-, loan-, and
    effect-checked **completely at its definition**, against its bounds only; the
    sole operations on a `T`-value are §5's memory-model operations and the
    bounds' declared methods. **Instantiation never re-checks the body** (design
    0007 §2.1).

6.2 **Conformance error, not type error (the NN#10 distinction).** Instantiation
    checks exactly **bound conformance** — a property of the **argument**,
    decidable from the callee's signature alone, impl found locally (§7). A
    failure is a **use-site error attributable to the caller's argument**, like
    passing a `bool` for a `u8`; it is **not** a body-internal type error. NN#10
    holds precisely because a bad argument against a visible bound is the caller's
    local error. This distinction is normative and SHALL NOT be blurred (design
    0007 §2.1).

6.3 An associated-type projection `T::Item` (chapter 12 §1) inside a generic body
    denotes an **opaque projected type**, checked exactly as an opaque `T` (§5);
    its concrete choice is substituted only at instantiation (design 0009 §2.2).

---

## 7. Coherence

7.1 **Orphan rule (module granularity).** An `impl I for T` MAY be declared
    **only in `T`'s module or `I`'s module**. For a **generic interface** the
    placement referent is the interface's **declaration** module, independent of
    instantiation (design 0007 §2.3; the module is the coherence unit, chapter
    11).

7.2 **The uniqueness key is the instantiated interface** `(I[args…], T)`, not the
    bare `I`: `impl From[E1] for T` and `impl From[E3] for T` coexist (distinct
    keys), a second `impl From[E1] for T` is a duplicate compile error. Placement
    is keyed on the *declaration*, coherence on the *instantiation* (design 0007
    §2.3, §7.1).

7.3 **Unifiability rejection.** Two impl heads whose targets **unify** are a
    compile-time duplicate — at most one impl per `(instantiated interface,
    type)`. There is **no impl lattice, no negative reasoning, no specialization
    order** (overlapping/blanket impls refused), so coherence is a lookup, not a
    solver (design 0007 §1.1, §2.3).

7.4 **Finding an impl is a two-place lookup** over names reachable in the module
    DAG, no global scan (P20). This is the smallest coherence that makes NN#10
    hold: "`T` satisfies `I`" has a single locally findable answer (design 0007
    §2.3).

7.5 **Soundness role and builtin scalars.** This coherence is the precondition of
    the dispatch-consistency invariant (chapter 13 §2): it makes `resolve` single-
    valued, so at most one legal impl exists per `(instantiated interface, type)`
    per linked program. A **builtin scalar** target has no defining module; chapter
    13 §4.2 states where its impls are legal.

---

## 8. Cross-type `?`

8.1 Cross-type `?` (chapter 02 §6.5 parked it) is expressed by one library
    interface `interface From[E] { fn from(e: E) -> Self }` (self-less; `Self` =
    the impl target) (design 0007 §7.1).

8.2 For `expr?` with result-shaped `R1` (non-`ok` payload `E1`) inside a function
    returning result-shaped `R2` (non-`ok` payload `E2 ≠ E1`), `?` SHALL return
    `R2`'s non-`ok` variant built from `E2::from(e1)`, **provided
    `impl From[E1] for E2` exists** (found by §7). No impl is a **definition-site
    error at the `?` site**. Same-type `?` (the `E1 = E2` case) calls no `from`.

8.3 By §7.2 one error type MAY absorb several sources
    (`impl From[IoError] for AppError`, `impl From[ParseError] for AppError` —
    distinct keys, both legal in `AppError`'s module); `?` selects the impl whose
    source matches `expr`'s payload (design 0007 §7.1).

8.4 **Cross-type `?` inherits `From::from`'s declared effect** (§3.6): an `alloc`
    `from` makes the `?` allocating, a definition-site effect error in an unmarked
    function. `?` adds no effect of its own (design 0007 §7.1).

---

## 9. Instantiation, inference, and naming a generic as a value

9.1 **Body-local type-argument inference is allowed** (`push(list, x)` infers `T`
    from `x`); **no inference crosses a signature** (P2/NN#17) — bounds and
    argument type are both local (design 0007 §2.2).

9.2 **Expected-type and fn-pointer inference.** A type parameter in a value
    parameter's type SHALL be inferable from the argument, **including from a
    fn-pointer argument's return type** (`U` in `f: fn(T) -> U`); expected-type
    hints from annotations resolve payload-less variant construction; an
    unsuffixed integer-literal argument infers `i64` (design 0007 stage-1 ruling
    3; design 0009 §5.3 completes this — the `E1002` gap).

9.3 **No call-site turbofish.** A user generic call SHALL take no explicit
    type-argument bracket (`foo[Bar](x)` collides with indexing, unresolvable
    without a symbol table). Where a type parameter appears in no value argument
    (the `ptr_null[T]` shape), the type is written in **type position** (a binding
    annotation or return type), unambiguous by §1.2. Keyword-led intrinsics keep
    their bracket (design 0007 §6.2; reopenable as OBL-GENERIC-TURBOFISH).

9.4 **Naming a generic as a value — `name::[T]`.** A generic function used as a
    **value** (not called) names one instantiation with `::[T]`
    (`let f: fn(u8) -> u8 = id::[u8];`); it appears only in value positions
    (bindings, fn-pointer slots, vtable fields), so the indexing collision cannot
    arise, and the stored value is thereafter a concrete fn-pointer. The `::[`
    digraph is NN#13-clean (one-token check after the reserved `::`) (design 0007
    §6.2.1).

---

## 10. Monomorphization and determinism

10.1 **Monomorphization is the deterministic, documented, only strategy this
     edition** (P11): each distinct type-argument tuple produces a distinct
     concrete instance carrying the substituted types' concrete cost, no
     shared-code indirection, no hidden dispatch, predictable from source. The
     source-level override is deferred (OBL-GENERIC-STRATEGY) (design 0007 §5.1).

10.2 **Check-once, instantiate-cached.** The body is checked exactly once at its
     definition; instantiation is **codegen, never re-analysis**, keyed by the
     instantiation tuple. An instance's `alloc`-ness is fixed at the definition
     (§5.5), so **effects are never re-derived at codegen** (design 0007 §5.2;
     chapter 11 §7 carries the cross-module artifact).

10.3 **Polymorphic recursion — decidable case is a definition-site error.** A
     body directly instantiating itself (or a mutually-recursive partner) with a
     **syntactically growing** argument (`T ↦ Wrap[T]`) is non-terminating,
     decidable from the body, and SHALL be reported **once at the definition
     site** — never at instantiation (design 0007 §5.1.1).

10.4 **The undecidable remainder is a documented resource bound, not a type
     error.** Indirect or value-dependent instantiation chains are backstopped by
     a **deterministic, documented monomorphization depth limit**; crossing it
     aborts the build with a **resource diagnostic** naming the instantiation
     stack — the memory-exhaustion category, **not** a type error, so NN#10's
     letter holds. It is recorded as a real, deterministic resource cliff, not
     claimed away (design 0007 §5.1.1).

10.5 **`sizeof(T)` is post-monomorphization only**: an opaque `T` has no known
     size at the definition site, so `sizeof(T)` SHALL NOT be used there where a
     compile-time constant is required; it is legal only once `T` is concrete
     (design 0007 §5.1.1).

---

## 11. Compiler-known parametric types

11.1 `[T]` / `write [T]` slices and `[N]T` arrays **stay built-in syntax**;
     `rawptr T` **stays compiler-known**; `Box T` is re-spelled **`Box[T]`** and
     **stays compiler-known** (`box`/`unbox`/`clone`/deref compiler-blessed).
     `BoxResult[T]` becomes an **ordinary library generic enum**
     (`enum BoxResult[T] { ok Boxed(Box[T]), OutOfMemory }`), its `ok` marker
     surviving substitution and driving `?` (design 0007 §6.3).

---

## Appendix 10-A (non-normative) — clause-to-prototype-diagnostic map

Informative; the specification speaks in clauses.

| Spec clause | Prototype diagnostic |
|-------------|----------------------|
| 4.2 generic impl param absent from target | E1016 |
| 4.3 impl-method conformance axes (a)–(f) | E1021–E1026 |
| 4.4 extra impl method | E1014 |
| 9.2 type parameter not inferable | E1002 |
| 5.4 / 03 §7.5 conditional-init at a generic drop point | E0309 |
