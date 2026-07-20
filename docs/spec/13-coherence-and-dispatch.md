# 13 — Coherence and Dispatch Soundness

**Status: NORMATIVE-DRAFT.** Transcription of design
`0018-coherence-dispatch-soundness` §4 (the ratified specification obligation),
resting on the coherence rules of design `0007-generics-and-bounds` §2.3 and the
associated-type projection rules of design `0009-iteration-and-associated-types`
§2.2. Rationale is in design 0018; this chapter states rules only, in **clauses**,
not error codes (the non-normative appendix maps them). It builds on chapter 10
(interfaces, impls, coherence §7) and chapter 12 (associated-type projection §1),
and states the **soundness invariant** those chapters' rules exist to guarantee.
**Discharges Commitment 1 of design 0018.**

This chapter is the direct guard of NN#1 (no undefined behavior in safe code) for
the interface subsystem: definition-site checking (chapter 10 §6) is sound only if
the impl the checker resolves is the impl the runtime executes.

---

## 1. Resolution and dispatch

1.1 **Resolution.** For every interface-method call, the type-checker selects
    **exactly one `(interface, impl)` pair** — the *resolved* interface and impl —
    and type-checks the program against that impl's signature. The selection is:
    - in a **generic context**, fixed by the receiver parameter's declared bound
      (chapter 10 §6.1); and
    - for a **direct call** on a concrete receiver, the **deterministic
      first-matching legal impl** providing the method for the receiver's type.

    This selection is a **checker fact recorded on the call** and is normative:
    the program's meaning is defined against the resolved impl (design 0018 §4.1).

1.2 **Dispatch.** *Dispatch* is the impl a conforming execution engine actually
    executes for that call. This chapter's invariant governs the relationship
    between resolution (a check-time fact) and dispatch (a run-time fact) on
    **every** conforming engine (design 0018 §4.1).

---

## 2. The dispatch-consistency invariant

2.1 **The invariant.** For **every** interface-method call in **every** well-formed
    program, on **every** conforming execution engine, the impl dispatched SHALL be
    the impl resolved:

    > **dispatch(call) = resolve(call).**

    An implementation whose runtime executes an impl other than the one the checker
    resolved and type-checked against **violates this specification** — even if
    every engine executes the *same* wrong impl and no two engines disagree
    (design 0018 §4.1, §6.2). This is a soundness obligation, not an implementation
    courtesy: because the checker approved the program against the resolved impl's
    signature, dispatching a different impl is a type-confusion hole in safe code
    (NN#1).

2.2 **The dispatch key is the resolved interface.** Dispatch selection SHALL be
    keyed on the **(target type, resolved interface, method)** triple, **never** on
    **(target type, method)** alone. When a receiver type carries **two interfaces
    with a same-named method** (chapter 10 §4.4 forbids two impls of *one* interface
    adding the method, but distinct interfaces may share a method name), the call
    SHALL dispatch the impl of the interface the checker resolved (§1.1), including
    when the two methods differ in return type (design 0018 §4.1).

2.3 **Single-valued resolution is the precondition.** The invariant is guaranteeable
    only because §4 makes `resolve(call)` **single-valued** — at most one legal impl
    per `(interface, type)` pair in a linked program. Where resolution is not
    single-valued, the invariant is unsatisfiable by any implementation (design 0018
    §4.2, §5.3).

---

## 3. Reject, do not miscompile

3.1 **The rule.** Any construct a conforming implementation does not **fully and
    faithfully** support SHALL be **rejected at check time**. A construct that
    type-checks and is then miscompiled is a specification violation **regardless of
    the construct's feature or support status**: "not yet supported" is a licence to
    *reject*, never to *miscompile* (design 0018 §4.1, §2.5).

3.2 **Applied to dispatch.** In particular, a call whose resolved impl (§1.1) an
    implementation cannot faithfully dispatch SHALL be **rejected at check time**,
    never dispatched to a **different** impl. Reject-don't-miscompile is a normative
    obligation on dispatch (design 0018 §4.1).

---

## 4. Coherence as the soundness precondition

The orphan and uniqueness rules of **chapter 10 §7** are the coherence discipline
that makes `resolve` single-valued (§2.3). This section states their **soundness
role** and completes the builtin-scalar case chapter 10 §7 leaves implicit.

4.1 **Legal placement (the orphan rule).** An `impl I for T` is legal **only** in
    `T`'s defining module, `I`'s defining module, or the **program root** — the
    two-place placement of chapter 10 §7.1, with the root admitted for a
    self-contained program (design 0007 §2.3).

4.2 **Builtin scalar targets.** A **builtin scalar type** `T` (e.g. `i64`) has **no
    defining module**. An `impl I for T` for such a `T` is therefore legal **only**
    in **`I`'s defining module** or in a **self-contained program root**; every other
    module — any dependency, and even a module tree's non-owner root — SHALL reject
    it. Consequently, since an interface is owned by exactly one module, **at most one
    legal `impl I for T` for a builtin scalar can exist per linked program**: two
    packages cannot each bless a divergent one (design 0018 §4.2; design 0007 §2.3).

4.3 **Uniqueness and the linked-program guarantee.** Two impls of the **same
    interface for the same type** — two impl heads that **unify** — SHALL be rejected
    (chapter 10 §7.2–§7.3). Therefore, across a **linked program**, **at most one
    legal impl exists for each `(instantiated interface, type)` pair**, so
    `resolve(call)` is single-valued (§2.3). This uniqueness is the property
    conformance is tested against, not merely an asserted design rule (design 0018
    §4.2).

---

## 5. Associated-type projection normalization

Chapter 12 §1 establishes that a projection `P::Assoc` is single-valued (at most one
`impl I for C`, so no disambiguation is needed) and that it is opaque at a definition
site. This section states the **normalization obligation** at instantiation.

5.1 **Deterministic normalization to a concrete leaf.** A projection `P::Assoc`
    whose base `P` is bound to a **concrete type** SHALL normalize to a **single
    concrete leaf type**, deterministically — the associated-type binding the unique
    `impl I for P` fixes (chapter 12 §1.1) (design 0009 §2.2).

5.2 **Transitivity with a termination bound.** Normalization SHALL proceed
    **transitively**: when an impl's associated-type binding is itself a projection
    (an adapter whose `type Item = I::Item` over a further concrete base), the base is
    concretized and the projection resolved again, so an **adapter-over-adapter chain
    reaches the underlying leaf**. This transitive resolution SHALL be bounded by a
    **fixed, documented depth**; a chain that exceeds the bound (a malformed or cyclic
    binding) is treated as **unresolvable** and handled by §5.3 — it SHALL NOT
    diverge (design 0009 §2.2; design 0018 §4.3).

5.3 **Reject, never leak.** A projection over a concrete base that does **not**
    normalize to a concrete leaf (including one that exceeds the §5.2 bound) SHALL be
    **rejected at check time**. An **unresolved projection SHALL NOT reach codegen or
    dispatch** — there is no silent mis-normalization and no unresolved projected type
    in a compiled program. A projection over a base still **opaque** at a definition
    site (an unbound parameter) is not an error: it stays symbolic per chapter 12
    §1.2 and is normalized only once the base is made concrete (design 0018 §4.3;
    design 0009 §2.2).

---

## Appendix 13-A (non-normative) — clause-to-prototype-diagnostic map and mechanism

Informative; the specification speaks in clauses.

| Spec clause | Prototype diagnostic / mechanism |
|-------------|----------------------------------|
| 4.2 orphan placement (incl. scalar) | E1013 |
| 4.3 overlapping / unifiable impl heads | E1009 |
| 5.3 unresolvable projection | check-time rejection (no codegen) |

**Mechanism (non-normative).** The prototype realizes the §2 invariant by having the
type-checker **record the resolved interface on each method call** and carry it
unchanged through monomorphization onto the call node; every execution engine keys
dispatch on the recorded `(target, interface, method)` triple, falling back to the
same deterministic first-matching-impl table when a direct call carries no recorded
interface. The §5 termination bound is a fixed projection-reduction depth limit
beyond which a projection is treated as unresolvable and rejected. These are
implementation facts about the prototype; the normative content is §§1–5.
