# Adversarial review — 0018 Coherence and dispatch soundness obligation

**Design:** [docs/design/0018-coherence-dispatch-soundness.md](../design/0018-coherence-dispatch-soundness.md)
**Date:** 2026-07-17
**Reviewer:** adversarial pass (fresh context), mandate: break it — is this obligation
well-founded and proportionate, or does it overclaim, over-reach, or propose a
validation that would not catch the bug it exists for?
**Verdict:** **RATIFY-WITH-REPAIRS.** The asymmetry is real and well-evidenced
(0007 §2.3 states def-site checking "is sound only if 'T satisfies I' has a single
answer" — coherence *is* NN#1-load-bearing), it did break, and it got no proactive
apparatus. The direction is right and the draft is unusually self-aware. But one
operational gap is disqualifying as-drafted, and three claims over-reach. Repairs
1–6 have been applied to the draft; dispositions recorded inline below.

Dispositions are the deciding authority's; recorded per finding once made.

---

## F1 — The campaign has no gate for its own load-bearing bug. Severity: SINK-as-drafted → REPAIR

§5.2's three gates — (a) engine-agreement, (b) orphan-violation-rejection, (c)
projection-normalization — do **not** catch §2.2 (dispatch ran `B::tag` while the
checker resolved `A::tag`, with *all engines agreeing* on the wrong impl): it is not
a divergence (a), not an illegal impl (b), not a projection (c). So a §2.2-shaped
corpus program — which §5.1 says *must* be in the corpus — would pass all three gates
green. The theory (§6's `dispatch = resolve` invariant) is never operationalized in
§5. Without this, the campaign cannot catch the very bug it exists for.
**Repair:** add **gate (d)** — the compiler emits, per trait method call, the `resolve`
result (interface/impl the checker selected), and the campaign checks it equals the
`dispatch` executed on each engine; plus author-declared expected-output assertions
per corpus program.
**Disposition:** ACCEPTED — repair applied (§5.2 gate (d); §6.2/§6.3 now enforce the
invariant via the gate and state plainly that (a)/(b)/(c) miss §2.2).

## F2 — Oracle circularity: named honestly, not hollow, but the draft under-uses its own fix. Severity: REPAIR

`resolve` and `dispatch` are two genuinely distinct compiler artifacts, so the
invariant adds real signal (it caught §2.2's drift where `resolve` was right) — the
"soundness, not just consistency" claim stands *in principle*. But the checker's
`resolve` is compiler-internal, so gate (d) alone catches dispatch-drift-from-
resolution, **not wrong resolution**. Bet 5 had external ground truth (ported
programs); §5.4 concedes this campaign lacks it — yet **author-declared expected
outputs** (a partial external oracle) would materially reduce the circularity and
were never adopted.
**Repair:** fold expected-output oracles into gate (d); stop framing prose + review
as closing "agree-but-wrong" in general — name the residual honestly.
**Disposition:** ACCEPTED — repair applied (expected-output oracle in gate (d);
residual named in §5.4/§6.2–§6.4/§10; overclaim dropped).

## F3 — Unmigratability overclaimed; the NN#14 analogy is weaker than stated. Severity: REPAIR

Ownership has no syntactic escape, so a memory-model rework is unmigratable.
Dispatch/coherence does have one: an ambiguous `x.tag()` can be mechanically rewritten
to a disambiguated form under an edition (P15). So "exactly as unmigratable as the
memory model" is false for the annotation-escapable subset. Deeper: routine
implementation-soundness fixes are ordinary post-1.0 bug fixes (fixing UB breaks no
correct program) and need no gate. The 1.0 gate is proportionate only to the
*residual design-flaw* risk — the case where the coherence *rules themselves* are
wrong — which the draft's own evidence rates low.
**Repair:** drop "identical unmigratable break"; scope §7 to the residual design-flaw
risk; acknowledge edition-disambiguation and that UB fixes need no gate.
**Disposition:** ACCEPTED — repair applied (§1.2/§7 rescoped; §7 §"1.0 gate" now
targets the residual risk a migrator cannot save).

## F4 — Mis-reads NN#20/P18: coherence is adopted art, not novelty. Severity: REPAIR

P18 mandates *formalization/mechanization* only where "the language's own commitments
require novelty" (the fault window), and says "adopt proven art where proven art
suffices." 0007 §2.3 calls its rule "the standard orphan rule"; the design is built
on being deliberately non-novel. So Commitment 1's push to "formalize (mechanized
preferred) at NN#20's tier" over-elevates adopted art (by that logic the borrow/type
checkers would qualify too). And NN#20's spec must already "define the semantics,"
which includes dispatch — so *specifying* it is arguably already owed, making a new-NN
framing partly redundant.
**Repair:** recast Commitment 1 as "**specify** the dispatch semantics — discharging
scope NN#20's spec already owns"; reserve "formalize/mechanize" for genuine novelty.
**Disposition:** ACCEPTED — repair applied (§3/§4/§4.4/§9 recast to "specify (discharge
NN#20)"; coherence named as adopted art, no novelty claimed).

## F5 — The empirical spine over-attributes by one. Severity: REPAIR

The retrospective's three build-time UBs were projection, the dispatch-key bug, and
**the canonical formatter rewriting `read v.*` → `v`** — the formatter is not
coherence/dispatch. So "undefined behavior three times … every one in the generics/
coherence/dispatch plumbing" is wrong by one, and §2's "five findings" silently swaps
the formatter for the review/dogfooding catches (§2.3/§2.4 are not UBs). For an
NN-adjacent amendment the motivating count must be exact.
**Repair:** state the precise tally — **two in-subsystem UBs (projection, dispatch-key)**,
plus a coherence review (impl-for-scalar) and two module-tree qualification gaps,
which are not UBs.
**Disposition:** ACCEPTED — repair applied (§1.3/§2/§9 corrected; formatter excluded).

## F6 — The kill criterion has no mechanical limb. Severity: REPAIR

Bet 5's criterion computed KILL mechanically from frozen thresholds; 0018's "a
divergence needing a *design* change" is pure judgment the same authority can always
narrate as "implementation fix" (all five historical cases were). It structurally
trends toward "fix the impl and continue."
**Repair:** pre-register at least one **automatic** design-kill category — e.g., "two
distinct legal impls are both validly selectable for one call under the resolution
rule" (an ambiguity the rule cannot resolve) fires without adjudication.
**Disposition:** ACCEPTED — repair applied (§5.3/§10/§12 mechanical auto-design-kill limb).

---

## What genuinely holds (strengthens the proposal)

- The **asymmetry is real and well-evidenced**: coherence is NN#1-load-bearing (0007
  §2.3), it broke, and — unlike the memory model — it got no spec, no validation
  prototype, no kill criterion.
- **`dispatch = resolve` is the right invariant**, and (with gate (d)) it is now
  operationalized, not merely asserted.
- A **standing, pre-registered spec-plus-campaign beats reactive catching** by the
  differential apparatus.
- The draft **pre-empts its own hardest attacks** (oracle circularity, agreement-vs-
  soundness, validate-vs-redesign) rather than hiding them.

## Net effect of the repairs

The proposal's ask shrank from **a grand new Non-Negotiable** (formalize at the fault-
model tier + a blanket unmigratable 1.0 gate) to a more honest and proportionate one:
**specify the dispatch semantics NN#20's spec already owes + a real validation gate
(gate (d): `dispatch = resolve` + expected-output oracles) + a 1.0 gate scoped narrowly
to the residual design-flaw risk a migrator cannot save.** The deciding authority
should ratify the repaired draft, with F1 (the dispatch-consistency gate) as the
load-bearing addition without which the campaign is theater.
