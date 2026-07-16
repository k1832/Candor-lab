# Adversarial review — Bet 6 pre-registered measurement criterion (DRAFT)

**Draft:** [docs/BET6_CRITERION.md](../BET6_CRITERION.md) (DRAFT-0, 2026-07-16)
**Date:** 2026-07-16
**Reviewer:** adversarial pass (fresh context), mandate: is this a valid, falsifiable,
adequately-powered pre-registered test of Bet 6 — or does it measure the wrong thing?
**Verdict:** **NOT-YET-A-VALID-TEST.** The machinery (sampling, freeze discipline, the
corpus→eval scoring wall) is sound, but the committed measurement *axis* does not test Bet 6
as P19 states it. Two SINK-class findings; the fix is a deciding-authority reframe, not a patch.

The disposition is the deciding authority's; this document records the review pending that ruling.

---

## F1 — Construct validity: the in-context axis measures prompt-conditioning, not manufactured competence. Severity: SINK

P19 (LANG_PHYLOSOPHY.md L79-80, §249) frames "manufactured" competence as competence **built
into the model** via the project's spec-grounded **synthetic corpus and fine-tuning tokens** — the
counterfactual to competence "awaited from organic corpus accretion," beating the incumbents'
*training-corpus-scale* barrier (L375). The draft (§3.1, §4.1) instead measures kibitokens of spec
pack placed **in the context** of an **unchanged** model — §4.1 openly substitutes this because
"the prototype has no fine-tuning pipeline." That is retrieval / prompt-conditioning: a weaker,
different claim.

**Killer detail:** §4.4.6 / §6.2c define the cross-release trend at the **fixed apparatus rung B3**
while the *model* varies across releases — so a rising `r_k(B3)` is **pure frontier-model
improvement**, which the project did not manufacture, yet is labeled "the manufacture gets more
efficient" (§3.1). To isolate manufacturing you must **hold the model fixed and vary/improve the
project's corpus**; the draft does the reverse. A CONFIRM on this axis would not support Bet 6.
As written, §1.1/§6 would let an in-context CONFIRM "be scored as a Bet 6 verdict" — the laundering
pre-registration exists to prevent.

**Repair (authority's choice):** either (a) **gate the Bet 6 verdict run on a corpus-trained /
fine-tuned model**, so the budget axis is fine-tuning tokens as P19 names — the lab cannot self-serve
this today; or (b) explicitly **narrow the registered claim** to "in-context spec-pack adaptability"
and record that P19-manufacturing stays **untested** until the fine-tuning pipeline exists.

## F2 — Falsifiability teeth are hollow on this axis. Severity: SINK

Lift `L = r(B3) − r(B0)` (§6.3a) is near-tautologically positive: a coherent spec pack in context
obviously helps a capable model, so that clause essentially cannot fire. §6.3(b) is satisfied by
general model improvement (F1). The only KILL with real teeth — corpus-quality collapse §6.3(c) —
presupposes a *generator distribution* that does not exist in the in-context regime, and §6.5 defers
its definition/threshold "to ratification" while the harness scores no idiom quality. So every
quantitative KILL clause is near-unfireable and the qualitative one is unbuilt; with §6.4 VOID-for-
saturation routing failures away from KILL, the criterion trends **can-only-confirm**.
**Repair:** fold into F1's reframe; if kept, operationalize §6.5 before freeze with a concrete
degenerate-diversity metric, and add a KILL clause the in-context axis can actually trip.

## F3 — Ceiling thresholds asserted, and frozen against a non-existent, headroom-selected task set. Severity: REPAIR

The 0.85 / 0.98 guards (§6.4) trace to nothing but "proposed" — weaker than BET5_CRITERION2 §0.2(b),
which derived every threshold from a carried margin or named principle with the derivation shown.
Worse, the verdict rests entirely on a "harder, non-saturating, independently-authored task set" that
**does not exist** (§9.2, "the single largest blocker"), and admitting a task *because* B3 sits below
ceiling (§9.2) pre-selects for headroom — guaranteeing room for the near-certain in-context lift
(feeds F2). Unlike Bet 5, which froze a ruler over *already-frozen* artifacts, this freezes a ruler
over objects authored *after* the freeze. **Repair:** tie ceiling to the §5.1 resolvable half-width;
defer freezing §6.4 / task identity until the non-saturating set exists.

## F4 — Budget axis confounds volume with content-kind. Severity: REPAIR

The rungs add *different* content (grammar → +semantics → +idioms → +exemplars), yet §4.4.4 fits an
OLS "rate per kibitoken" slope treating them as a fungible continuous budget. A positive slope may be
one rung's content being essential, not a budget relationship. **Repair:** report per-rung lift; drop
the fungible-budget OLS framing (or state the confound explicitly).

## F5 — Release-slope power. Severity: REPAIR

A 3-point OLS slope CI is nearly uninformative; §6.2c's "CI excludes a negative slope" over 3
releases is either unmeetable or fragile. **Repair:** require more releases, or make the release test
a sign test rather than a slope-CI.

## F6 — Adaptation-material provenance unguarded. Severity: NIT/REPAIR

The family-bias gate (§5.4) constrains *anchor* authorship vs the model-under-test, but the spec pack
/ exemplars (B4) — the manufactured thing itself — have unconstrained family provenance; §11
acknowledges the residual but doesn't record adaptation-material authorship. **Repair:** attest
adaptation-material provenance per run.

---

## What genuinely holds

- The **S≥5 sampling** upgrade with a **cluster-bootstrap CI on the correct unit** (tasks as clusters)
  is right and fixes the prior rounds' N=1 defect.
- The **B0 zero-budget counterfactual / lift construction** cleanly isolates the *in-context* claim
  (it just isn't the Bet 6 claim — F1).
- Scoring is delegated to the built `candor` oracle; the **corpus→eval scoring wall** (§5.3) and the
  **never-model-generated anchors** (§5.2) are airtight *on the scored side*. The leak is not the
  grader — it is the **independent variable**.
- Freeze instant / amendment lock / ledger / authority-enacted verdict mirror BET5_CRITERION2.

## The one thing the authority must resolve before ratifying

**Decide what claim is being registered.** In-context spec-pack conditioning of an unchanged model
is not P19's "manufactured competence." So either (a) **gate the Bet 6 verdict run on a
fine-tuning / corpus-trained model** (the axis P19 names — not lab-self-servable today), or (b)
**re-title the registration as an "in-context adaptability" proxy** that explicitly *cannot* confirm
or kill Bet 6 / P19 on its own. Ratifying as-is would let a near-guaranteed in-context lift, riding
frontier-model improvement, be enacted as a Bet 6 confirmation.

**Implication for the campaign:** a *graduation-tier Bet 6 verdict* is not runnable in-lab until a
fine-tuning / corpus-training pipeline exists (a capability the lab does not have), independent of the
already-known operator dependencies (cross-family model access, a harder non-saturating task set).
