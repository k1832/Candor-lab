# 0018 — Coherence and dispatch soundness obligation

**Status: RATIFIED (deciding authority, 2026-07-17), after adversarial review #1 + repairs.**
Adversarial review #1 returned **RATIFY-WITH-REPAIRS**; repairs 1–6 were applied (see
the Appendix ledger), and the deciding authority ratified the repaired document. This
document establishes a pre-stability soundness obligation — a *specification* obligation
NN#20's spec already owns, plus a validation gate — and a narrow residual-risk 1.0 gate.
The §9 open-comment step for the NN-adjacent 1.0-gate extension is satisfied by this
repository's public record (the draft, the review, and the repairs were published in
full before enactment; objections, had any been recorded, would appear in the ledger).
The commitments below are now binding pre-stability work.

**Date:** 2026-07-17
**Philosophy hooks:** **NN#1** (no undefined behavior in safe code — the invariant
this subsystem broke twice, §2.1 and §2.2), **P11/NN#10** (public generics checked at their
definition site; coherence and associated items are the "real, irreducible
complexity" P11 pays for), **P18/NN#20** (a normative specification, independent of the
compiler, *defines the semantics* — and a spec that defines the semantics must define
dispatch; P18 adopts proven art where it suffices and the orphan rule is "the standard
orphan rule" (0007 §2.3), so the obligation here is *specification*, not the novel
formalization NN#20 reserves for the fault model), **Bet 5** (the validation-prototype-plus-pre-registered-kill-
criterion methodology this borrows wholesale), **P20** (CI-gated, pre-registered,
measurable release criteria, or the claim is withdrawn by amendment), **NN#14/P15**
(a 1.0 gate scoped to the *residual design-flaw* risk a migrator cannot save; routine
dispatch fixes are ordinary post-1.0 bug fixes and an ambiguous call is itself
mechanically disambiguable under an edition).
Subordinate to `LANG_PHILOSOPHY.md` and to designs 0007 (generics and bounds),
0009 (iteration and associated types), and 0015 (borrowed iteration), which define
the coherence/dispatch model this proposes to specify and validate. Where they
conflict, the higher document wins and this one changes. It operationalizes delta #1
of `PHILOSOPHY_RETROSPECTIVE.md` ("add a validation bet for the trait/coherence/
dispatch system, at Bet 5's tier").

**What this is.** A proposal to close a structural asymmetry in the philosophy. The
document makes soundness priority #1 and defends it with a full apparatus aimed at
the **memory model** — Bet 5's validation prototype and pre-registered kill
criterion, NN#14's 1.0 gate, NN#20's mandatory fault-model formalization. It gives
the **coherence and dispatch subsystem** — interface resolution, the orphan rule,
associated-type normalization, and method dispatch — far less, though NN#1 depends on
it exactly as much. This proposes three commitments: (1) **specify** the
dispatch-consistency invariant (`dispatch = resolve`), the orphan rule, and projection
normalization in the normative spec — discharging scope NN#20's spec already owns,
since a spec that "defines the semantics" must define dispatch; this is *specification
of adopted art*, not the novel formalization NN#20 reserves for the fault model (§4.4);
(2) a differential-plus-adversarial validation campaign with a pre-registered kill
criterion, CI-gated like P20's targets, whose load-bearing new check is a
**`dispatch = resolve` + expected-output gate** (§5.2 gate (d)); (3) a 1.0 gate scoped
narrowly to the **residual design-flaw** risk the campaign might surface — the case
where the coherence *rules themselves* are wrong and no migrator can save them — not to
implementation bugs, which are ordinary post-1.0 fixes.

**What this is not.** **Not a redesign of the trait system.** The retrospective's
finding is that the *design* held and the *plumbing* broke; this validates the
existing designs 0007/0009/0015, it does not reopen them (§8). Not a claim that
differential engine agreement is a soundness proof — §6 concedes exactly where it is
not, and that concession is the reason commitment 1 exists. Not a novel-formalization
promise: the coherence rules are adopted art ("the standard orphan rule", 0007 §2.3),
so commitment 1 is *specification* of what NN#20's spec must already contain, not the
fault-model-tier mechanization NN#20 reserves for genuine novelty (§4.4). Not a gate on
routine soundness bugs — fixing UB in an implementation breaks no correct program and
is an ordinary post-1.0 fix; the 1.0 gate (§7) is scoped only to the residual risk that
a coherence *rule itself* is unspecifiable or not checker-runtime-consistent. Not a
change to the memory model, fault model, effect set, or coherence *rules* themselves —
those are fixed by 0001/0007/0009/0015 and, if this validation ever contradicts one,
§8.2 names the branch that reopens it, evidence first.

---

## 1. Problem — the asymmetry, and where NN#1 actually broke

### 1.1 The philosophy defended one soundness-critical subsystem and left the other bare

The priority order puts **soundness** first and admits no exceptions (§2.1, NN#1).
The document then spends its entire soundness-validation apparatus on the memory
model:

- **A validation prototype with a pre-registered kill criterion** (Bet 5): the
  memory-model core, ported against an adversarial basket, measured, published
  either way.
- **A 1.0 gate** (NN#14, P15): "no stability commitment before Bet 5's pre-registered
  verdict," justified because "a memory-model rework is precisely the break no tool
  can migrate."
- **Mandatory pre-stability formalization** (NN#20, P18): the novel fault semantics
  "formalized as mandatory pre-stability work," because "'the compiler is the spec'
  is how 'no UB in safe code' decays from theorem to folklore, and NN#1 deserves
  better than folklore."

The coherence and dispatch subsystem receives none of these. P11 names its
complexity precisely — "declared capabilities, associated items, coherence rules —
real, irreducible complexity, a meaningful fraction of what makes Rust's traits
heavy" — and pays it deliberately. But P11 treats that complexity as a *simplicity*
cost (a P6-budget expenditure), not as a *soundness* risk. There is no normative
spec of what dispatch must do, no validation prototype that stresses it, no kill
criterion, and no gate. The subsystem the philosophy filed under "complex" is
missing every defense the philosophy built for the subsystem it filed under "risky."

### 1.2 The asymmetry is unjustified on the philosophy's own terms

Two properties made the memory model deserve its apparatus. The coherence/dispatch
subsystem has both, to the same degree.

- **Equally soundness-critical (NN#1).** Definition-site checking (P11) is *sound
  only if* "does `T` satisfy `I`?" has a single, locally findable answer and the impl
  the checker resolves is the impl that runs (0007 §2.3 states this dependency
  explicitly: "definition-site checking is sound only if 'T satisfies I' has a single
  … answer"). A dispatch that runs a different impl than the one type-checked is a
  type-confusion hole — undefined behavior in safe code, an NN#1 violation, with no
  memory-model bug anywhere near it.
- **The migration trap is narrower than the memory model's — but real at the
  residual.** The memory model earns its 1.0 gate because its rework "cannot be
  mechanically migrated." Dispatch and coherence are *not* unmigratable to the same
  degree: an ambiguous `x.tag()` can be mechanically rewritten to a disambiguated form
  under an edition (P15), and a routine implementation-soundness fix breaks no correct
  program at all. So the honest claim is narrower than the memory model's: what a
  migrator *cannot* save is a change to the coherence *rules themselves* — an orphan
  rule that turns out not to be both locally checkable and dispatch-consistent, or a
  projection rule with no deterministic normal form. Those re-resolve calls across the
  entire trained corpus with no over-approximation available (P15's test), the way a
  changed ownership discipline would. That residual — not dispatch fixes in general —
  is what §7's gate is scoped to.

Same soundness load; a narrower but real migration trap at the residual; far less of
the defense. That gap is what this document closes.

### 1.3 The empirical case: it did not merely *could* break — it *broke*

The asymmetry is not a hypothetical. The first real implementation — a self-hosting
compiler across five execution engines (tree-walker, MIR interpreter, Cranelift,
LLVM, AOT) — produced undefined behavior from safe, well-typed code **twice in this
subsystem** (§2.1, §2.2), sitting in the generics/coherence/dispatch plumbing, never in
the value/borrow model that got the validation:

> "the value/borrow model is the one thing that did not break … But safe,
> well-typed code produced undefined behavior three times this build, all in the
> machinery the design treated as merely 'complex' rather than 'risky' … The memory
> model was doubted rigorously and held; the trait machinery was assumed and broke."
> — `PHILOSOPHY_RETROSPECTIVE.md`

Read that count exactly. Of the retrospective's three build-time UBs, **two are in this
subsystem** — the projection that failed to normalize (§2.1) and the dispatch key
computed wrong (§2.2) — and the third is the canonical formatter rewriting `read v.*` to
`v`, which is a *formatter* defect, not coherence or dispatch, and is not counted here.
The subsystem's other two findings — the impl-for-scalar coherence review (§2.3) and the
two module-tree qualification gaps (§2.4) — are real but are *not* UBs. So the motivating
tally is precise: **two in-subsystem UBs, plus a coherence review and two qualification
gaps that are not UBs.** The document does not silently swap the formatter UB in for the
review-and-dogfooding catches to reach a rounder number.

The through-line is the whole argument: **doubt was allocated by the philosophy's
risk labels, and the labels were wrong.** The memory model was doubted and held; the
trait machinery was assumed and broke. This document reallocates the doubt.

---

## 2. The evidence, concretely (the motivation, not decoration)

All five findings below are from the one real build; all are in the trait plumbing. Of
the five, **two are UBs** — §2.1 (projection) and §2.2 (the dispatch key), the ones
that type-checked and then miscompiled; the other three (§2.3, §2.4) are a coherence
review and two qualification gaps, real defects but not UBs. The retrospective's third
build-time UB, the canonical formatter, is a formatter defect and is out of this
subsystem's scope (§1.3). All five were caught **reactively** — by the five-engine
differential gate *or* by an adversarial coherence review after the fact — never by a
proactive spec or a pre-registered stress campaign, because none existed.

### 2.1 Associated-type projection failed to normalize through a concrete base

A projection (`I::Item`, 0009 §2.2) that did not normalize to its concrete leaf
through a concrete base type produced a **miscompile of well-typed safe code** — and,
tellingly, a *different* miscompile per engine: the tree-walker returned garbage, the
MIR interpreter panicked, the native backend **segfaulted**. One safe program, three
distinct failures. A segfault from safe code is a naked NN#1 violation.

This one **diverged across engines**, so the differential gate caught it. Hold that
fact against §2.2, which did not.

### 2.2 Method dispatch keyed on `(target, method)`, not the resolved interface

Dispatch was keyed on the pair `(receiver type, method name)` rather than on the
*interface the checker resolved*. So a call the checker resolved as `A::tag` — having
type-checked the program against `A`'s signature — ran `B::tag` on **every engine**.
The checker approved a program the runtime then executed as a different method; with
differing return types between `A::tag` and `B::tag`, that is type confusion the
type-checker *blessed* — a memory-safety hole.

**This is the load-bearing example of the whole document.** Because all five engines
shared the same wrong dispatch key, *all five agreed* — they all ran `B::tag`. The
differential gate compares engines against each other; it sees no divergence when
every engine is wrong the same way, so it was **blind to this bug**. It was caught
by an adversarial coherence review, not by the gate. §6 builds the agreement-vs-
soundness argument on this case: it is the concrete proof that byte-identical
five-engine agreement is *consistency*, not *soundness*.

### 2.3 impl-for-scalar coherence required a dedicated review to confirm

Whether two divergent `impl Ord for i64` could coexist — the orphan rule's core
promise that a `(interface, type)` pair has at most one impl (0007 §2.3) — was not
self-evidently guaranteed by the implementation and required a **dedicated coherence
review** to confirm it held for scalar target types. The property is a one-line
design rule; that confirming it took a review is the point. Coherence uniqueness is
asserted in the design and was not, until reviewed, an *observed* property of the
compiler.

### 2.4 Two module-tree impl-qualification gaps

Two separate gaps let a bounded generic impl **check clean in a single file but fail
in a module tree**: first a type-parameter-bounds qualification gap, then an
associated-type-binding qualification gap. Both were surfaced only by self-hosting —
the largest program in the language exercising bounded impls across a real module DAG
at a scale no single-file fixture reached (`PHILOSOPHY_RETROSPECTIVE.md`: "dogfooding
at scale … finds soundness … obligations a curated test suite structurally cannot").
A curated suite would not have found them; an adversarial corpus built to stress
module-tree qualification would have.

### 2.5 What the five findings establish together

- The failure surface is the whole subsystem, not one construct: **resolution**
  (2.2), the **orphan rule** (2.3), **projection normalization** (2.1), and
  **module-tree qualification** (2.4) each broke or needed defending.
- The severity axis is **reject-vs-miscompile** (`PHILOSOPHY_RETROSPECTIVE.md`): 2.1
  and 2.2 *type-checked and then miscompiled* — the subsystem's two NN#1 violations,
  where a clean rejection would have been a mere limitation; 2.3 and 2.4 are real
  defects but not UBs. This is the exact soundness surface the
  formalization (§4) must pin down: a construct the compiler cannot fully handle must
  be *rejected at check time*, never dispatched wrong.
- Detection was **reactive and partial**: differential agreement caught the divergent
  bug (2.1) and was blind to the agree-but-wrong bug (2.2); dogfooding caught the
  module-tree bugs (2.4) by luck of scale; two required a human coherence review
  (2.2, 2.3). None was caught by a standing, pre-registered soundness campaign,
  because there was none to catch them.

---

## 3. What is proposed — three commitments

The memory model's apparatus is the reference point, but the mapping is *not*
one-for-one — commitment 1 is a **specification** obligation NN#20's spec already owns,
not fault-model-tier formalization, and commitment 3 is scoped to a **residual
design-flaw** risk, not to the memory model's blanket unmigratability:

| Memory model has | Coherence/dispatch gets (proposed) |
|---|---|
| NN#20: fault model **formalized** (novel semantics) in the normative spec | **Commitment 1** (§4): **specify** the `dispatch = resolve` invariant, the orphan rule, and projection normalization in the spec — discharging scope NN#20 already owns for *adopted* art, not novel formalization (§4.4) |
| Bet 5: validation prototype + **pre-registered kill criterion** | **Commitment 2** (§5): a differential + adversarial campaign, CI-gated, with gate (d) (`dispatch = resolve` + expected-output) and a pre-registered kill condition carrying a mechanical design-kill limb |
| NN#14: no 1.0 before the verdict | **Commitment 3** (§7): no 1.0 before this campaign passes — scoped to the residual risk that a coherence *rule itself* is wrong, the only part a migrator cannot save |

The three are ordered by dependency, exactly as the memory model's are: the spec
(commitment 1) is what makes the campaign (commitment 2) a *soundness* check rather
than a consistency check (§6), and the campaign's passing verdict is what the gate
(commitment 3) waits on.

---

## 4. Commitment 1 — specify the coherence and dispatch model in the normative spec

NN#20 already requires "a normative specification, independent of the compiler, [that]
defines the semantics." Dispatch and coherence *are* semantics — a spec that defines the
semantics must define what a call resolves to and dispatches to — so this commitment
does not add an obligation at the fault-model tier; it **discharges scope NN#20's spec
already owns** and that the build left unwritten. The coherence rules here are *adopted
art*, not novelty: 0007 §2.3 calls its rule "the standard orphan rule," and P18 adopts
"proven art where proven art suffices," reserving *formalization as mandatory
pre-stability work* for "where this language's own commitments require novelty" (the
fault window). By that logic the borrow and type checkers would qualify for the
fault-model tier too, and they do not — they are specified, not formalized as novel
semantics. The obligation here is therefore **specification**; this document names no
genuine novelty in the dispatch semantics that would demand more. What is unchanged is
the reason to write it down: "the compiler is the coherence model" is the folklore NN#20
rejects, and NN#1 rests on this subsystem, so its semantics belongs in the spec, tested
against, not left in the dispatch table.

Three properties must be stated precisely. Each is the normative form of a §2 failure.

### 4.1 The load-bearing invariant: checker-runtime dispatch consistency

> **For every method call, the interface and impl the checker resolves is the impl
> that every execution engine dispatches.**

State it as a commuting obligation between the two artifacts that must never
disagree: let `resolve(call)` be the `(interface, impl)` the type-checker selects and
against whose signature it type-checks the program, and let `dispatch(call)` be the
impl the runtime executes. The invariant is `dispatch(call) = resolve(call)` for
every call in every well-typed program, on every engine. §2.2 is exactly the
statement that the compiler computed `dispatch` from `(receiver type, method name)`
while `resolve` had selected on the interface — so the two functions disagreed, and
the disagreement was a type-confusion hole the checker had approved. The spec names
the invariant so that a compiler which violates it violates the *spec*, catchable
without waiting for two engines to happen to disagree (they did not, §2.2).

The invariant's consequence for an unfinished compiler, stated as a spec rule
because §2.5 makes it the soundness surface: **a call whose resolved impl the
compiler cannot faithfully dispatch must be rejected at check time, never dispatched
to a different impl.** Reject-don't-miscompile is a spec obligation on dispatch, not
an implementation courtesy.

### 4.2 The orphan rule: which impls are legal

The spec states, normatively, which impls may exist, such that **two divergent impls
for one `(interface, type)` pair cannot coexist** — the uniqueness that makes
`resolve` single-valued and therefore makes `dispatch = resolve` even *possible* to
guarantee. The rule already exists in prose in 0007 §2.3 (placement keyed on the
interface's declaration module; uniqueness keyed on the *instantiated* interface
`(I[args…], T)`; no blanket, overlapping, or specialized impls). Commitment 1 lifts
it from a design-doc rule to a normative spec invariant with a stated soundness role:
coherence uniqueness is the precondition of dispatch consistency, and §2.3 showed it
was an *asserted* property that took a review to confirm was an *observed* one. The
spec makes it the thing conformance is tested against.

### 4.3 Associated-type normalization: projections resolve deterministically

The spec states that **every associated-type projection normalizes to a single
concrete leaf, deterministically, or is cleanly rejected** — never silently
mis-normalized into a miscompile (§2.1). 0009 §2.2 already argues single-valuedness
(at most one `impl Iter for C`, so `C::Item` has no second impl to disambiguate
against); commitment 1 states the *normalization* obligation normatively: through any
chain of concrete bases, projection reaches its leaf or the program is rejected at
check time. The determinism clause is what turns §2.1's three-different-failures into
a single defined answer or a single defined rejection.

### 4.4 What "specify" means here — stated honestly, no overclaim

The obligation is *specification*, not the novel formalization NN#20 mandates for the
fault window (§4). Following NN#20's own posture ("mechanized formalization … strongly
preferred; at minimum, the specification is the arbiter and the compiler is its
subject") and 0003's ("a rigorous informal argument, not a machine-checked proof"):

- **The obligation: a normative prose specification** of §4.1–§4.3 — the invariant, the
  orphan rule, and the normalization obligation — written independently of the compiler,
  such that the compiler is tested against it and a divergence is a compiler bug, not a
  spec revision. This is the floor and the ask; it is what a spec that "defines the
  semantics" (NN#20) already owes for dispatch.
- **Optional, not promised: a mechanized model** of resolution and dispatch (a small
  executable relation `resolve`/`dispatch` and a checked statement that they agree). For
  adopted art this is a *nicety that would tighten the oracle* (§6.4), not a
  fault-model-tier requirement; the honest floor is the prose invariant, and this
  document does not claim mechanization it has not scoped. Whether it is feasible at
  acceptable cost is itself a finding the campaign (§5) can report.

The distinction matters for §6: the prose spec plus the expected-output oracle (§5.2
gate (d)) is what moves the validation from a consistency check toward a soundness
check, and §6.3 states precisely why, and the residual it leaves.

---

## 5. Commitment 2 — a differential + adversarial validation campaign, CI-gated, pre-registered

Mirroring Bet 5's validation prototype (a throwaway artifact exercising an
adversarially-chosen basket, measured against a criterion frozen in advance) and
P20's discipline (measurable targets tracked in CI as release criteria, or the claim
is withdrawn by amendment).

### 5.1 The corpus

A **trait-heavy corpus**, deliberately chosen to concentrate exactly the pressure
§2 broke under, in two parts:

- **The standard library as the natural-load anchor** — iterators (`Iter`,
  `Indexed`, `RefIndexed`, the adapter chains of 0009 §9), `Ord` and comparison,
  collections, and the bounded generic impls the corelib already carries. This is the
  trait system under its intended workload, at self-hosting scale, which §2.4 shows is
  where module-tree qualification bugs actually surface.
- **Adversarially-authored coherence stress cases** — the analog of Bet 5's
  adversarial basket, built to attack the subsystem where it broke: cross-interface
  dispatch (a type carrying two interfaces with a same-named method and *differing
  return types* — the §2.2 shape, which is the case differential agreement is blind
  to and which therefore *must* be in the corpus), projection through deep concrete
  bases (§2.1), orphan/uniqueness edge cases including impl-for-scalar (§2.3), and
  bounded generic impls exercised across module trees (§2.4).

The adversarial half is load-bearing and is also this document's sharpest honesty
problem: unlike Bet 5's basket, which humans *ported* from fixed external programs,
these cases are *authored* to stress the system, by a party who can see the compiler.
§5.4 and §10 name what that costs the campaign's independence.

### 5.2 The gates — dispatched across all five engines, CI-gated

For every program in the corpus, run under all five engines, gated in CI (P20):

- **(a) Consistency gate.** Every trait method call dispatches **byte-identically on
  all five engines.** This is the differential gate that caught §2.1; it catches
  *divergence* — and, as §2.2 proved, nothing else.
- **(b) Coherence/orphan gate.** Every coherence or orphan violation is **rejected at
  check time, never miscompiled** — the reject-don't-miscompile rule (§4.1) applied to
  illegal impls. A stress case that should be rejected and instead type-checks is a
  gate failure regardless of what it then runs.
- **(c) Normalization gate.** Every associated-type projection **normalizes to its
  concrete leaf or is cleanly rejected** (§4.3) — never a silent mis-normalization.
- **(d) Dispatch-consistency gate — the one that catches §2.2.** For every trait method
  call, the compiler **emits `resolve(call)`** — the `(interface, impl)` the checker
  selected and type-checked against — and the campaign asserts, on **each engine**, that
  the impl actually **`dispatch`ed equals that emitted `resolve`**. This operationalizes
  the §4.1 invariant `dispatch = resolve` directly, so §2.2 (all five engines running
  `B::tag` where the checker resolved `A::tag`) is a **red gate**, not a green pass.
  **And:** each corpus program carries **author-declared expected-output assertions**,
  checked on every engine — a *partial external oracle* (the thing Bet 5 had via ported
  programs) that pins the *right* answer independently of the compiler's own `resolve`,
  so a checker that resolves *and* dispatches wrong the same way still fails against the
  declared output.

Gates (a)–(c) are the checks the build's apparatus (plus §4's spec) can express, but
**none of them catches §2.2**: it is not a divergence (gate a), not an illegal impl
(gate b), and not a projection (gate c). Gate (d) is the limb that operationalizes the
theory of §6 — without it, a §2.2-shaped corpus program passes every other gate green
while running the wrong impl. Gates (b)–(d) are checks *against the spec* (§4), which is
why commitment 1 must precede commitment 2. Gate (d)'s two halves close different gaps:
the `dispatch = resolve` half catches **dispatch-drift-from-resolution** (one real
compiler artifact against another), and the expected-output half catches **wrong
resolution** to the extent the declared outputs cover it — §6.4 states the residual
honestly.

### 5.3 The pre-registered kill condition — with a mechanical limb

Frozen before the campaign runs, in a registration document at
`docs/BET5_CRITERION2.md`'s tier (structure, freeze discipline, publish-either-way,
data-aware-conflict honesty if authored after any measurement is visible). It has two
limbs — a **mechanical** one that fires without adjudication, and a **judgment** one:

> **Mechanical design-kill (automatic).** The model is KILLED if the campaign exhibits
> **one call for which two distinct legal impls are both validly selectable under the
> resolution rule** — an ambiguity `resolve` cannot single-value, so the §4.1 invariant
> `dispatch = resolve` is *unsatisfiable by any implementation*. That is a defect of the
> rule, not the plumbing, and it fires on the corpus artifact alone, with no
> "implementation fix" reading available. (Equivalently: a projection the rule admits
> with no deterministic normal form, or an orphan configuration the rule permits under
> which two divergent impls coexist for one `(interface, type)` pair.)
>
> **Judgment design-kill.** The model is also KILLED if the campaign surfaces a
> soundness divergence that requires a DESIGN change rather than an implementation fix,
> or if the coherence rules cannot otherwise be made checker-runtime-consistent (§4.1).

The model is then reworked before 1.0. The mechanical limb exists because "a divergence
needing a *design* change" is otherwise pure judgment the authority can always narrate
as "implementation fix" — all five §2 cases *were* so narrated, correctly. Pre-registering
an ambiguity the resolution rule cannot resolve gives the criterion teeth that do not
depend on that adjudication. The **design-vs-implementation line** remains the spine of
the judgment limb, and it is drawn where the retrospective drew it: §2's five findings
were *implementation* fixes — the design's dispatch-on-resolved-interface rule (0007) and
single-valued-projection rule (0009) were correct, and the plumbing was made to match
them. A *kill* is the opposite finding: making the plumbing sound requires changing the
*rule*. That is the finding the campaign exists to be able to return, and pre-registering
it — mechanical limb first — is what stops the line being redrawn after the data is seen
(§5.4). A pass is not "no bugs"; a pass is "every bug the campaign found was an
implementation fix against an unchanged design, the mechanical limb never fired, and all
four gates hold green in CI."

### 5.4 Pre-registration honesty (the Bet 5 discipline, inherited with its residual)

The campaign inherits Bet 5's methodology *and* its named weaknesses. The kill
condition is frozen before the run; the results are published whatever they are (a
kill enacted as a §9 amendment, a pass recorded as the verdict that unblocks the 1.0
gate). The residual is stated, not dissolved: the adversarial corpus is authored, not
ported, so a corpus author who can see the compiler can under-stress the exact case
the compiler happens to handle — the construct-validity threat Bet 5 named for its
data-aware successor (`docs/BET5_CRITERION2.md` §0). Mitigations are the same and no
stronger: freeze the corpus and the gates before the run, subject the corpus to an
adversarial review briefed to hunt *for the missing stress case* (not just to check
the present ones), and keep the whole corpus and every engine's output in the public
record. These reduce the threat; they do not eliminate it, and §10 keeps it visible.

---

## 6. The hardest open question — is five-engine agreement soundness, or only consistency?

This is the attack the document most needs to survive, so it is answered in the body,
not deferred to a footnote.

### 6.1 The question, stated at full strength

The differential gate's guarantee is that five independent engines produce
byte-identical output. The retrospective calls this "the operational counterpart to
the spec." But five engines agreeing proves only that they *agree* — it is a
*consistency* property. **All five can agree on the same wrong answer.** And this is
not hypothetical: §2.2 is exactly that case. Every engine shared the dispatch key
`(receiver type, method name)`, so every engine ran `B::tag`, so all five agreed —
and the program was type-confused. The differential gate was blind to it. If the
strongest tool in the build can be unanimously, consistently wrong, then "the five
engines agree" is not a soundness proof, and a campaign built only on gate (a) (§5.2)
would inherit that blindness.

### 6.2 The answer: differential agreement catches divergence; gate (d) catches agree-but-wrong

The two failure modes are different, and they need different catchers:

- **Divergence** — engines disagree (§2.1: garbage, panic, segfault from one
  program). Differential agreement catches this directly: any disagreement is a
  red flag. Gate (a) owns it.
- **Agreement-but-wrong** — engines agree on an answer that violates the intended
  semantics (§2.2: all five run the wrong impl). Differential agreement is
  *structurally blind* to this, because there is no divergence to see. Nothing that
  compares engines *to each other* can catch it.

What catches agree-but-wrong is **gate (d)** (§5.2), and it does so in two distinct ways
that must not be conflated:

- **`dispatch = resolve` catches dispatch-drift-from-resolution.** `resolve` and
  `dispatch` are *two real compiler artifacts*; asserting they agree is genuine signal —
  it does not ask "do the engines agree?" but "does the impl every engine ran equal the
  impl the checker resolved?" In §2.2 that answer is *no* even though the engines agreed,
  so this half catches the exact bug the differential gate missed. This is the §4.1
  invariant, operationalized.
- **Expected-output assertions catch wrong resolution — partially.** Because `resolve` is
  *compiler-internal*, the first half cannot catch a checker that resolves *and*
  dispatches wrong the same way (the §2.2 hazard displaced up one level). The
  author-declared expected outputs are a *partial external oracle* — the thing Bet 5 had
  via ported programs — that pins the right answer independently of the compiler, so a
  program's declared output failing on every engine catches wrong resolution on the calls
  the outputs cover.

Gates (b) and (c) likewise check the two rules beneath the invariant (§4.2, §4.3).
Together with gate (d) they are the soundness-directed checks where gate (a) is only a
consistency check.

### 6.3 Therefore: the spec invariant plus gate (d) is what moves the campaign toward soundness

This is the load-bearing link between the commitments, and it must be stated without
overclaiming. **Commitment 2 with only gates (a)–(c) is a consistency campaign** — five
engines gated to agree (plus two spec-rule checks), still blind to the §2.2 class,
because none of (a)–(c) catches a call that runs the wrong impl with every engine
agreeing. **Gate (d) is what operationalizes the theory:** the `dispatch = resolve` half
turns dispatch-drift from an undetectable class into a gate failure, and the
**author-declared expected-output** half — not prose plus reviewer judgment — is what
materially reduces the "agree-but-wrong" residual by supplying an oracle external to the
compiler. Differential agreement remains necessary — it is the cheap, strong catcher of
divergence, and it caught §2.1 — but it is not sufficient. The invariant is enforced by a
*gate*, not by prose and review; prose and review alone do not close the "agree-but-wrong"
class, and this document no longer claims they do. What partially closes it is the
expected-output oracle, and §6.4 states the residual that remains.

### 6.4 The residual limit, conceded

Formalization upgrades the check but does not make it a proof, and the document says
so rather than overclaiming. Two residuals:

- **Oracle circularity — reduced by the expected-output oracle, not dissolved.** The
  `dispatch = resolve` half of gate (d) computes `resolve` with the *same checker* whose
  resolution is under test, so it catches dispatch-drift but **not wrong resolution**: a
  checker that resolves *and* dispatches wrong the same way escapes it (the §2.2 hazard
  displaced up one level). What reduces this is the **author-declared expected-output**
  half of gate (d) — a partial *external* oracle, independent of the compiler's
  `resolve`, that catches wrong resolution on the calls its outputs cover. It is
  *partial*: outputs are authored, cover only exercised call shapes, and a resolution
  wrong in a way the declared output cannot observe still escapes. The prose spec read by
  a reviewer narrows it further ("a reviewer confirms the compiler's resolution matches
  the written invariant"), and the optional mechanized `resolve`/`dispatch` relation
  (§4.4) would narrow it further still — but neither is a machine-checked entailment on
  the general case, so the residual is real and named. The document does **not** claim
  prose plus review closes "agree-but-wrong" in general.
- **Corpus coverage.** The campaign proves the invariant holds *on the corpus*, not
  universally. An agree-but-wrong bug on a call shape no stress case exercises is
  caught by neither gate (a) (no divergence) nor gates (b)–(d) (not run). This is
  Bet 5's "provisional confirmation on this basket" (`docs/BET5_CRITERION2.md` §6.5),
  inherited exactly: the verdict is scoped to the corpus, "provisional" and "on this
  corpus" are load-bearing, and §5.4's adversarial corpus review is the (partial)
  defense.

The honest summary: the spec invariant plus gate (d)'s two oracles move the campaign
from *consistency-checking* toward *soundness-checking against a written invariant and a
partial external oracle, on a corpus*, with a named oracle-independence residual and a
named coverage residual. That is a strictly stronger
claim than "the engines agree," and it is not "the model is proven sound." Both halves
are stated because the document's own philosophy forbids letting the first
masquerade as the second (the header warning: a named cost is not a managed cost).

---

## 7. Commitment 3 — the 1.0 gate, scoped to the residual design-flaw risk (the NN#14 analog)

> **No stability commitment (no 1.0) may precede a passing verdict of the
> coherence/dispatch validation campaign (§5) — where a *pass* is defined by §5.3 (no
> mechanical or judgment design-kill). The gate exists for one thing a migrator cannot
> save: a coherence *rule itself* found unspecifiable or not checker-runtime-consistent.
> It does not gate routine soundness fixes.**

This is NN#14 restated for this subsystem, but scoped honestly to what NN#14's own logic
(P15) actually reaches. Two things it does *not* reach:

- **Routine implementation-soundness fixes are ordinary post-1.0 bug fixes.** Fixing UB
  in the implementation breaks no *correct* program — a program that relied on the buggy
  behavior was never well-defined — so such a fix ships post-1.0 with no migrator and
  needs no gate. §2's two UBs were exactly this class.
- **Ambiguous dispatch is mechanically migratable.** Unlike a changed ownership
  discipline, an ambiguous `x.tag()` can be rewritten to a disambiguated form under an
  edition (P15), so even a resolution change is not automatically the unmigratable break
  the memory model's is.

What *is* unmigratable — and what this gate is for — is the **residual design-flaw**
case: a coherence rule that cannot be made both locally checkable and dispatch-consistent,
or a projection rule with no deterministic normal form (the §5.3 kill categories). If a
rule like that ships, no over-approximation and no disambiguating rewrite saves the
corpus, the way none saves a changed ownership discipline (P15's test). That residual is
the only part of this subsystem that carries the memory model's migration trap, and the
doc's own evidence rates it **low** — the design held; the plumbing broke. The gate is
therefore proportionate: a thin, pre-registered stop on a low-probability but
unrecoverable outcome, not a broad hold on the subsystem. Concretely: NN#14 currently
gates 1.0 on Bet 5's verdict; this adds the coherence/dispatch campaign's verdict as a
second, independent precondition, where a failing verdict means specifically a §5.3
design-kill. Both must pass; neither substitutes for the other.

Stated as the amendment it is (§9): this extends NN#14 to read, in effect, "no stability
commitment before Bet 5's verdict *and* before the coherence/dispatch validation
campaign's passing verdict (§5.3)." That extension is the Non-Negotiable change this
document exists to propose, and it takes the §9 slow path (§9 below).

---

## 8. Scope guard — this validates, it does not redesign

### 8.1 The claim: the design held, the plumbing broke

The retrospective is explicit and this document does not exceed it: "a sound design
is a hypothesis about the compiler, not a guarantee of one — and the hypothesis was
false in the generics/coherence/dispatch plumbing." Every §2 finding was an
*implementation* defect against a *correct* design rule — dispatch was fixed to key
on the resolved interface that 0007 already specified, projection normalization was
fixed to reach the leaf that 0009 already required. This is a validation campaign for
designs 0007/0009/0015, not a proposal to reopen them. Commitment 1 *writes down* the
invariant those designs assume; it does not change it. Commitment 2 *tests* the
compiler against it; it does not redesign it.

### 8.2 If validation reveals a genuine design flaw: the evidence-first rework branch, named

The scope guard is honest only if it admits the branch where it is wrong. The kill
condition (§5.3) *is* that branch: a soundness divergence requiring a design change,
or coherence rules that cannot be made checker-runtime-consistent, is a finding
against the *design*, and the pre-registration exists precisely so that finding can be
returned rather than defined away. If it is returned, the response is **not** a
blanket redesign; it is a **targeted, evidence-first rework** of the specific rule the
campaign broke — the same discipline the retrospective drew from the Bet 5 KILL
("measure where the valve concentrates and attack those specific ergonomics — not to
argue about defaults in the abstract"). The rework is scoped to the failing rule,
driven by the campaign's evidence, and runs through the normal design pipeline
(draft → adversarial review → ruling). This document does **not** assume that branch
is unreached — the retrospective's evidence is that the design held — but it names the
branch so the scope guard is a falsifiable claim, not an assumption.

### 8.3 The honest seam: was §2.2 plumbing, or an under-specified design?

The sharpest attack on the scope guard (and §10 flags it): §2.2's dispatch-keying bug
can be read as a *design silence* — 0007 specified dispatch on the resolved interface
in its coherence discussion, but did the design *state, normatively and up front*,
that the dispatch key is the resolved `(interface, impl)` and never `(type, method)`?
If not, the bug lived in a gap the design left, which is closer to under-specification
than to pure plumbing. The document's answer is that this is not a redesign trigger
but is *exactly the argument for commitment 1*: the rule was correct but *unwritten as
a normative invariant*, and §2.2 is what an unwritten invariant costs. Specification
(§4.1) closes the seam by writing the invariant down normatively; it does not change
the rule, so it is validation-of-the-design, not redesign. The seam is conceded, not
hidden — it is the difference between "the design was wrong" (a redesign, not claimed)
and "the design was right but unspecified" (a specification, claimed).

---

## 9. Governance path — a Non-Negotiable amendment on the §9 slow path

This document adds a pre-stability soundness obligation (commitments 1–2) and extends
a Non-Negotiable 1.0 gate (commitment 3, extending NN#14). Under `LANG_PHILOSOPHY.md`
§9, that is an amendment to the Non-Negotiables, and the authority's power over it is
"deliberately slowed": the proposal, its rationale, and its evidence **must be
published for open comment for a defined period before enactment, and the enacted
amendment records the objections alongside the decision.**

What the amendment changes, recorded per §9's requirement that an amendment (a) name
what it changes, (b) state the evidence that overcame the burden of proof, and (c)
append to Appendix A's ledger:

- **(a) What it changes.** It extends **NN#14** (adds the coherence/dispatch campaign
  verdict as a second 1.0 precondition, scoped to the §5.3 residual design-flaw risk,
  §7). It **does not** raise coherence to NN#20's novel-formalization tier: NN#20's spec
  already "defines the semantics," and dispatch is semantics, so specifying the
  `dispatch = resolve` invariant, the orphan rule, and projection normalization
  *discharges* scope NN#20 already owns rather than enlarging it — the coherence rules
  are adopted art ("the standard orphan rule", 0007 §2.3), not the novelty NN#20 reserves
  formalization for. It adds a validation obligation in the tier of Bet 5's prototype. It
  changes **no** coherence or dispatch *rule* — 0007/0009/0015 stand (§8).
- **(b) The evidence.** The **two** in-subsystem NN#1 violations (§2.1 projection, §2.2
  the dispatch key), plus a coherence review (§2.3) and two module-tree qualification
  gaps (§2.4) that are real but not UBs — all in the trait plumbing, none caught by a
  standing soundness apparatus because none existed; the retrospective's finding
  (delta #1) that the subsystem got none of the memory model's defenses despite carrying
  the same soundness load (its migration trap is narrower — only the residual design-flaw
  case is unmigratable, §7); and the §6 argument that differential agreement alone is
  consistency, not soundness.
- **(c) The ledger.** On enactment, a row in `LANG_PHILOSOPHY.md` Appendix A records
  the change, this document, the evidence, and the open-comment objections.

Until that path completes, this document is **DRAFT — NOT RATIFIED** and binds
nothing.

---

## 10. Where an adversarial reviewer will push (pre-empted, not hidden)

Per the house norm of stating attacks before a reviewer does:

- **"Your soundness check is still circular."** Gate (d)'s `dispatch = resolve` half
  computes `resolve` with the same checker under test, so it catches dispatch-drift but
  not wrong resolution (§6.4). Conceded — and the reply is *not* "prose plus review fixes
  it": what materially reduces the circularity is gate (d)'s **author-declared
  expected-output** oracle, a partial *external* check, with the prose spec and the
  optional mechanized relation narrowing it further. It is reduced, not eliminated; this
  remains the document's weakest joint and it is named as such.
- **"The pre-registration is weaker than Bet 5's."** Bet 5 ported fixed external
  programs; this *authors* its adversarial corpus, so a corpus author who can see the
  compiler can omit the one stress case it fails — construct-invalidity by omission
  (§5.4). Two things narrow the gap toward Bet 5's: gate (d)'s **author-declared expected
  outputs** are a partial *external* oracle of the kind Bet 5 had via ported programs,
  and the §5.3 **mechanical** design-kill limb fires without the adjudication a motivated
  authority could shade. The residual mitigations (freeze-first, adversarial corpus review
  briefed to hunt the *missing* case, public record) are real but weaker than an
  externally-fixed basket, and the *judgment* kill limb remains shadeable. Conceded and
  mitigated, not solved.
- **"This is a redesign wearing a validation costume."** §2.2 reads as a design
  under-specification, so formalizing the invariant is arguably changing the design,
  not validating it (§8.3). The reply — the rule was right but unwritten, so writing it
  is formalization not redesign — is a real distinction but a *fine* one, and a
  reviewer may reasonably press whether commitment 1 quietly settles design questions
  0007/0009 left open. The evidence-first rework branch (§8.2) is the honest escape
  hatch if it does.
- **Lesser pushes, flagged:** whether the five-engine differential apparatus is even a
  ratified project commitment or an artifact of one build (commitment 2 leans on it and
  it is itself only a retrospective *proposal*, delta #2); whether commitments 1–2
  respect the P6 "when you add, remove" budget (answered as NN#20 answered it: a
  soundness obligation is a stability-gate cost, not a language-surface feature); and
  whether "byte-identical dispatch on all five engines" is even well-defined across a
  tree-walker and an AOT backend for every observable (it is defined for method-call
  results and return values; timing and address nondeterminism are out of scope, as
  P5's declared-nondeterminism carve-out already establishes).

---

## 11. Rejected alternatives

- **Do nothing — let differential execution carry NN#1 for this subsystem.**
  Rejected: §6 shows the five-engine gate is blind to unanimous error, and §2.2 is a
  real instance the gate missed. Differential agreement is necessary and insufficient;
  leaning on it alone is leaning on the tool that already failed to catch the
  load-bearing bug.
- **Only formalize (commitment 1), no validation campaign.** Rejected: a spec with no
  campaign gating it against the compiler is the "compiler is the spec" folklore NN#20
  rejects, in reverse — a spec nobody tests the implementation against is as inert as
  no spec. NN#20 pairs formalization with mandatory validation; so does this.
- **Only validate (commitment 2), no formalization.** Rejected: §6.3 is the whole
  argument — without the spec invariant, the campaign is a consistency check blind to
  agree-but-wrong (§2.2). Validation without formalization cannot catch the class of
  bug that most needs catching.
- **Redesign the trait system pre-emptively.** Rejected: the evidence (retrospective,
  §8.1) is that the design held and the plumbing broke. Redesigning a design that has
  not been shown to fail is speculative rework against P6 and against the evidence;
  §8.2's evidence-first branch reopens a rule only if the campaign shows that *specific*
  rule fails.
- **Fold this into Bet 5's criterion / the memory-model campaign.** Rejected:
  different subsystem, different failure surface (dispatch vs ownership), different
  corpus (trait-heavy vs value/borrow-heavy). NN#14 gets a second, independent
  precondition (§7), not a widened first one; conflating them would let a memory-model
  pass mask a dispatch failure.
- **Gate 1.0 on differential agreement alone, no separate campaign.** Rejected: same
  reason as the first alternative, escalated to the gate — gating stability on a
  consistency property that §2.2 shows can be unanimously wrong would put a known-blind
  check on the critical path and call it a soundness gate.

---

## 12. Consequences and costs (debts, not absolutions)

- **A new pre-stability obligation on the critical path.** Commitments 1–2 are work
  that must complete before 1.0 (§7). Per §8's Sequencing discipline in the
  philosophy, this is a stability-gate cost, not a founding-day one — but it is a real
  cost: a spec section, a frozen corpus, a CI gate across five engines, and an
  adversarial review, all before stability. Named, not waved.
- **The oracle-independence residual is unpaid at the prose minimum.** §6.4: the prose
  spec leaves a reviewer-judgment circularity that only the unpromised mechanized model
  closes. Shipping commitment 1 at its honest floor (prose) means the soundness check
  is "against a written invariant a reviewer confirms," not "machine-checked" — a real
  gap between what "soundness campaign" connotes and what the minimum delivers.
- **The verdict is provisional and corpus-scoped.** §6.4: a pass is "on this corpus,"
  inheriting Bet 5's exact limit. An agree-but-wrong bug outside the corpus is caught
  by nothing. The corpus is a floor under confidence, not a proof of its absence.
- **The judgment kill limb is still a judgment call; the mechanical limb is not.**
  §5.3/§10: whether a given failure "requires a design change" is adjudicated, and a
  motivated adjudicator could shade it toward "implementation fix" to avoid a kill. The
  §5.3 **mechanical** limb (two legal impls both selectable for one call) removes that
  discretion for at least one design-kill category; the judgment limb remains
  adjudicated, constrained by pre-registration and public record but not mechanized.
- **Dependence on five-engine infrastructure that is not itself ratified.**
  Commitment 2's gate (a) presumes ≥2 (here 5) independent engines maintained in
  lockstep — infrastructure the retrospective proposes (delta #2) but the philosophy
  has not yet adopted. If that infrastructure is not committed, commitment 2's
  consistency gate weakens; the dependency is a debt this document does not itself
  discharge.
- **It buys soundness confidence, not soundness.** The whole document, stated once
  plainly: it makes the coherence/dispatch subsystem *doubted with rigor* rather than
  *assumed*, closing the asymmetry §1 opens. It does not make the subsystem *proven
  sound*. "Doubted and held" is a stronger place than "assumed and broke" — it is the
  place the memory model already occupies — and it is not "proven." The header
  warning applies to this document as to the philosophy: a named cost is a borne cost,
  not a managed one.


---

## Appendix — review and repair ledger

| # | Date | Event |
|---|---|---|
| 1 | 2026-07-17 | **Adversarial review #1: RATIFY-WITH-REPAIRS** (six findings). Repairs 1–6 applied to this document (below). |

**Repairs applied (review #1):**

1. **Gate (d) added (§5.2, §6.2–§6.3).** Gates (a)/(b)/(c) did not catch §2.2; added
   gate (d) — per-call `dispatch = resolve` emission checked on every engine, plus
   author-declared expected-output assertions — so the §4.1 invariant is enforced by a
   gate, not by prose + review.
2. **Oracle circularity stated honestly (§5.4, §6.2–§6.4, §10).** `dispatch = resolve`
   catches dispatch-drift (two real compiler artifacts); the *expected-output* oracle,
   not prose + review, is what partially closes "agree-but-wrong"; residual named.
3. **Unmigratability de-inflated; §7 rescoped (§1.2, §7).** Dropped "as unmigratable as
   the memory model" — dispatch is mechanically disambiguable under an edition (P15) and
   routine UB fixes need no gate; §7 now gates only the residual design-flaw risk.
4. **Commitment 1 recast to "specify" (§3, §4, §4.4, §9).** Coherence is adopted art
   ("the standard orphan rule", 0007 §2.3); the obligation *discharges* scope NN#20's
   spec already owns, not novel formalization at the fault-model tier.
5. **Count corrected (§1.3, §2, §9).** Exact tally: **two** in-subsystem UBs (projection,
   dispatch key), plus a coherence review and two qualification gaps that are not UBs;
   the retrospective's third UB (the formatter) is out of subsystem and not counted.
6. **Mechanical kill limb added (§5.3, §10, §12).** Pre-registered automatic
   design-kill: two distinct legal impls both selectable for one call under the
   resolution rule — fires without adjudication.

**Status:** repairs applied; **RATIFIED by the deciding authority, 2026-07-17** (the §9
open-comment step satisfied by the public record of draft + review + repairs prior to
enactment). The full adversarial review #1 — its six findings, the accepted repairs, and
what genuinely holds — is recorded at
[docs/reviews/0018-coherence-dispatch-soundness-review.md](../reviews/0018-coherence-dispatch-soundness-review.md).
Commitments 1–3 are now binding pre-stability work: the spec section (Commitment 1) and
the gate-(d) validation campaign (Commitment 2) are the implementation queue.

**Commitment 1 status (2026-07-20): LANDED.** The dispatch-consistency invariant
(§4.1), the reject-don't-miscompile rule (§4.1), the orphan/uniqueness rule with its
builtin-scalar case and linked-program soundness role (§4.2), and associated-type
projection normalization (§4.3) are specified normatively in
[docs/spec/13-coherence-and-dispatch.md](../spec/13-coherence-and-dispatch.md)
(chapter 13, §§1–5), with cross-reference clauses added to chapters 10 §7.5 and 12
§1.5 and the chapter registered in `docs/spec/00-front.md`. Commitment 2 (the gate-(d)
campaign) remains the open implementation queue item.
