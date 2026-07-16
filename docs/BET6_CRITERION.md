# Bet 6 — Pre-Registered Measurement Criterion (DRAFT — NOT RATIFIED)

> **DRAFT — NOT RATIFIED.** This is a *proposal* for the Bet 6 pre-registration. It has **no
> force** until it survives adversarial review (§0.2c) and is ratified and frozen by the deciding
> authority (`GOVERNANCE.md`; philosophy §9). Nothing here is binding, no threshold here is final,
> and **no measurement may be scored as a Bet 6 verdict against this document while it carries this
> banner.** It is written to be attacked: a reviewer's job is to find the choice that lets Bet 6
> confirm itself, and every such choice is a defect to be fixed before ratification, not after.

**Artifact type:** Scientific pre-registration (draft). Not a design document, not a pitch. It fixes,
in the open, the conditions under which **Bet 6 / P19 — "model competence can be MANUFACTURED"** is
declared confirmed, refuted, or killed on the external anchors, and *nothing else* (philosophy §3, Bet
6 falsification clause; P19).

**Status:** DRAFT. Awaiting adversarial review #1 and deciding-authority ratification. Modeled on the
frozen `docs/BET5_CRITERION2.md` for structure, freeze discipline, and pre-registration language.
**Registration version:** DRAFT-0.
**Date:** 2026-07-16.
**Governing document:** `LANG_PHYLOSOPHY.md` v4.2 (normative; Bet 6, P19, P4/P13/P16). This sits at
the design-document tier under §9 and binds the measurement, not the philosophy.
**Adjudicating authority:** the single deciding authority in `GOVERNANCE.md` (philosophy §9),
currently k1832.
**Sibling registration:** `docs/BET5_CRITERION2.md` (RATIFIED AND FROZEN 2026-07-08). Bet 6 shares
Bet 5's external ground truth — the human-ported basket — and its freeze-manifest discipline
(`docs/FREEZE_MANIFEST.md`), but tests a *different* claim with a *different* metric shape (a slope,
not a per-basket verdict).

---

## 0. Status, honesty preamble, and ledger

### 0.1 This is a DRAFT written data-aware, and that is disclosed at the top

Three prior competence measurements are already public and saturated at ceiling
(`docs/measurements/eval/2026-07-09-{round1,round2,round2-graduation}.json` and their READMEs:
12/12, then 23/23, slope delta 0). This draft is therefore authored *knowing the seed and graduation
task sets saturate for a strong model*. That is a real threat to a pre-registration's meaning and is
stated here, where every reader starts (philosophy header: an admitted cost is a borne cost). The
mitigations are the same four `BET5_CRITERION2.md` §0.2 relied on — minimal, principled, adversarially
reviewed, reusable — plus one specific to this bet: **the saturation is not a nuisance to be worked
around but the exact failure this criterion is designed to detect.** A single 100% point is a floor,
not a slope (P19: "metrics are slope and efficiency … absolute parity is a long-term consequence, not
a launch criterion"). The metric below is built so that the prior ceiling results **cannot** be scored
as a confirmation.

### 0.2 The legitimacy basis is four constraints, not a claim of innocence

- **(a) Metric-first.** No verdict is computed in this document. It defines the aggregation; the
  scoring is a separate public act performed after this document freezes (§7). No expected slope value
  is stated.
- **(b) Thresholds derived by principle.** Every threshold below traces to a named principle (P19's
  positive-slope and anti-circularity clauses; the ceiling-guard the prior rounds compel; standard
  statistical power), with the derivation shown inline. No number is chosen to land a verdict.
- **(c) Adversarial review before freeze.** This draft is reviewed by a session briefed to hunt
  specifically for outcome-reverse-engineering — "does this pick let Bet 6 confirm itself?" Findings
  enter the §0.4 ledger before the freeze instant, exactly as `BET5_CRITERION2.md` §0.2(c) required.
- **(d) It binds future re-runs; it is not single-use.** Once ratified, these rules govern every
  competence measurement scored as a Bet 6 verdict — new releases, harder task sets, new budget
  rungs — without re-registration. It is written to be reused.

### 0.3 What this document must not do

It must not state, imply, or reverse-engineer a slope value. The graduation-tier verdict run (§7) is
performed after this document freezes; the numbers it produces — not this text — carry the outcome.
The reviewer (0.2c) judges whether the author held metric-first discipline despite knowing the seed
sets saturate.

### 0.4 Amendment ledger (pre-freeze changes only)

| # | Date | Section | Change | Rationale | Enacted by |
|---|------|---------|--------|-----------|------------|
| 0 | 2026-07-16 | all | Initial DRAFT of the Bet 6 pre-registration | Philosophy §3 (Bet 6) mandates a pre-registered, published kill criterion on the external anchors, held to Bet 5's standard; it is the missing artifact blocking the graduation-tier campaign | DRAFT — pending review + ratification |
| — | 2026-07-16 | §3/§4/§6 | Adversarial review #1 ([reviews/BET6-criterion-review.md](reviews/BET6-criterion-review.md)) returned **NOT-YET-A-VALID-TEST**: two SINKs — the committed **in-context** budget axis measures prompt-conditioning of an unchanged model, not P19's *fine-tuning-manufactured* competence (F1), and the KILL clauses are near-unfireable on that axis (F2). Machinery (sampling, scoring wall, freeze discipline) is sound. | **BLOCKED on a deciding-authority reframe** before any further work: (a) gate the verdict run on a fine-tuning/corpus-trained model (not lab-self-servable), or (b) re-title as an "in-context adaptability" proxy that cannot confirm/kill Bet 6. Not ratified. | Pending authority ruling |

*(Rows are appended by the deciding authority on accepted review findings; adversarial-review
dispositions enter here before the freeze instant. Unrecorded changes have no force.)*

---

## 1. Freeze instant and change lock

1.1 **Change lock (proposed).** Thresholds, definitions, the budget ladder, the task-set identity, and
the decision rule here may change **only before the first graduation-tier VERDICT run (§7) is
published**. After that instant this document is **frozen** — nothing added, removed, loosened, or
re-normalized (the `BET5_CRITERION2.md` §1.1 discipline, relocated to the only freeze point a
data-aware registration has: the mechanical verdict run this must not be retrofitted to). Any pre-freeze
change is a numbered §0.4 ledger row.

1.2 **Ratification precondition.** This document does not self-ratify. It becomes binding only when the
deciding authority records ratification (after review #1) and enters its hash into
`docs/FREEZE_MANIFEST.md`, as `BET5_CRITERION2.md` did (manifest §"Successor registration").

---

## 2. The claim under test

2.1 **Bet 6:** **model competence in Candor can be MANUFACTURED** — deliberately built by the project's
adaptation apparatus (spec pack, exemplars, and eventually a synthetic corpus and fine-tuning), rather
than awaited from organic corpus accretion (philosophy §3 Bet 6; P19; the §1 thesis consequence that
"model competence in this language is a deliverable of the project, not an emergent hope").

2.2 **The outcome measure.** First-attempt correctness on **external anchors** — the human-ported
Bet 5 basket and independently authored problems, **never self-generated tests grading self-generated
code** (P19). Correctness is the harness's stage classification (`tools/eval-harness/src/score.rs`):
a task **passes** iff the submission `check`s clean and (for a `run`/`generate` anchor) emits the
frozen sentinel; otherwise it fails at a classified stage (`parse`/`check`/`run`/`wrong-sentinel`/
`missing`). This is already built and automatic; the scoring is delegated wholly to the
`candor-proto` oracle, never re-implemented.

2.3 **Why a slope, not a rate.** The metric is **NOT** a single pass-rate. Bet 6's falsification clause
(§3) demands "a competence *slope* that is positive across releases; a flat slope … kills the bet." A
100% point proves the artifacts *can* be written, not that competence was *manufactured*: a model that
already scores 100% may carry the competence from pre-training, in which case the project's adaptation
apparatus manufactured nothing. **Manufactured competence is a CAUSAL claim about the project's own
adaptation material**, and the metric must isolate that causal component. §3–§4 do so.

2.4 **Scope honesty.** Confirmation is scope-limited to the frozen anchors and the model releases
actually swept — exactly as `BET5_CRITERION2.md` §6.5's "provisional … on this basket." Bet 6 is not
"Candor models beat Rust models"; P19 forbids that reading ("raw comparisons against incumbents
confound design with corpus scale and will read as failure for years regardless of merit"). The slope
is **within-language**, never Candor-vs-incumbent (§3.2).

---

## 3. The slope axis (the independent variable)

3.1 **Committed axis: correctness per unit of ADAPTATION BUDGET, within-language, swept across model
releases.** The independent variable is the *amount of project-authored adaptation effort* supplied to
the model; the dependent variable is first-attempt correctness on the frozen anchors (§2.2); and the
whole adaptation curve is re-measured on successive model releases. Bet 6 is confirmed by the curve
**rising with budget** (competence is caused by the project's material) **and improving across
releases** (the manufacture gets more efficient over time); it is refuted by a flat curve or a
non-improving release trend (§6).

3.2 **Why this axis over the alternatives** (P19's corpus-scale-confound caution is the deciding
constraint):

- **Rejected: raw model-capability / cross-model comparison** (score model A vs model B, or Candor-
  trained vs Rust-trained). P19 rejects it directly: it "confounds design with corpus scale and will
  read as failure for years regardless of merit." A cross-model rate measures the models' pre-training,
  not Candor's manufacturing. Used only *within* the family-bias controls (§5), never as the metric.
- **Rejected: task-difficulty as the swept axis** (score a fixed model over an easy→hard task ladder).
  This measures the *task library's* difficulty calibration, not the project's adaptation apparatus. It
  is a necessary **control** (the ceiling guard, §6.4, needs a difficulty spread) but not the claim:
  "manufactured" is about the *adaptation material*, not the problems.
- **Committed: adaptation budget, within-language, across releases.** This is the only axis whose
  slope is *caused by the thing Bet 6 claims* — the project's deliberately built adaptation material —
  and it is within-language by construction, honoring P19's confound caution. It is the axis P19 names
  verbatim: "repair and generation correctness *per unit of adaptation budget*."

3.3 **The curve, and its load-bearing anchor point.** At a fixed release, sweep the adaptation budget
across a fixed ladder of rungs (§4.2) and score the anchors at each rung. The resulting
**adaptation curve** `rate = f(budget)` is the object of measurement. Its **zero-budget anchor** —
the model scored with *no* project adaptation material (task statement only, no spec pack, no
exemplars) — is load-bearing: it is the counterfactual "what competence existed *without* Candor's
manufacturing." The metric of interest is the curve's **lift above that anchor**, not its absolute
height (§4.3). This is what makes the prior ceiling results non-confirming: a model that scores high at
zero budget has had *nothing manufactured for it*.

---

## 4. Adaptation-budget accounting (currently unbuilt — defined here to spec)

4.1 **The unit.** One unit of adaptation budget is **one kibitoken (1,024 tokens) of project-authored
adaptation material placed in the model's context before its attempt**, counted under a **fixed,
declared reference tokenizer** (named at ratification and hashed into the freeze manifest, so the unit
is model-agnostic and stable across model families). "Adaptation material" is the spec pack
(`docs/specpack/`) and any exemplars the project supplies — the deliberately built competence
substrate. It **excludes** the task `statement` and `prompt_material` intrinsic to the problem (that is
the problem, not the manufacturing) and excludes the model's own chain of thought. A second,
commensurable budget axis is defined for the repair loop: **feedback budget `B_fb`** = the number of
repair rounds, each round = one machine-readable diagnostic (`feedback_diagnostic`) fed back and
re-scored (the harness `--round` mechanism). Rationale: P19 names the unit as "fine-tuning tokens /
examples"; the prototype has no fine-tuning pipeline (corpus-gen feeds training only and is walled off,
§5.3), so the realizable, measurable, model-agnostic budget for the current apparatus is
**in-context adaptation tokens** plus **feedback rounds**. When a fine-tuning pipeline exists,
fine-tuning tokens enter as a third budget axis under the same "kibitokens of project material"
unit — the definition is chosen to extend without re-registration (§0.2d).

4.2 **The budget ladder (frozen at ratification).** A fixed, ordered set of rungs, each a named subset
of the adaptation material with a measured `B_ctx` (in kibitokens under the reference tokenizer):

| Rung | Adaptation material supplied | Purpose |
|---|---|---|
| **B0** | none (task statement only) | the zero-budget counterfactual anchor (§3.3) |
| **B1** | grammar summary only | minimal syntactic adaptation |
| **B2** | grammar + semantics summary | + rule adaptation |
| **B3** | full spec pack (grammar + semantics + idioms + diagnostics) | the manufactured apparatus as it stands |
| **B4** | full spec pack + worked exemplars | the apparatus + few-shot |

The exact file→rung mapping and each rung's measured `B_ctx` are frozen at ratification and hashed.
Rungs are strictly nested (each ⊇ the previous) so the curve is monotone in material and the
finite-difference slope (§4.4) is well defined.

4.3 **Correctness-per-budget — the two derived quantities.** Let `r(b)` be the first-attempt rate
(§2.2, §5.1 sampling) at rung `b`. Define:

- **Lift** `L = r(B3) − r(B0)` — the competence attributable to the standing manufactured apparatus
  above the zero-budget counterfactual. This is the primary "was anything manufactured" quantity.
- **Efficiency** `E(b→b') = (r(b') − r(b)) / (B_ctx(b') − B_ctx(b))` — marginal rate gained per
  kibitoken of added material between adjacent rungs. P19's "improvement per corpus token," realized on
  the in-context budget.

4.4 **The slope computation (`report.rs` must be extended to this spec).** `report.rs` today computes a
**per-round rate only** (`Report::build` → `first_attempt_rate = passed/total`); no slope, delta, lift,
or error bar exists anywhere. Define a new aggregation `SlopeReport` that ingests a set of per-run
`Report`s, each tagged with a coordinate `(release_id, rung, B_ctx, B_fb)` and produced under the §5.1
sampling, and computes:

  1. **Per-rung rate with error bars.** For rung `b`: `r(b) = passes / (T · S)` over `T` tasks × `S`
     samples; report a **cluster-bootstrap** 95% CI resampling *tasks* (then samples within task) —
     because samples of one task are not independent, task is the cluster. Report a Wilson score
     interval as a cross-check.
  2. **The adaptation curve** `{(B_ctx(b), r(b), CI)}` per release.
  3. **Lift** `L` (§4.3) with a CI on the *difference* `r(B3) − r(B0)` via the same cluster bootstrap
     (paired over tasks).
  4. **Budget-slope** — the ordinary-least-squares slope of `r(b)` on `B_ctx(b)` across rungs at a
     fixed release, with a bootstrap CI; and the per-segment efficiencies `E(b→b')`.
  5. **Repair sub-slope** — `r` as a function of `B_fb` (round-over-round), the existing round-1→
     round-2 delta generalized, with a CI.
  6. **Cross-release delta** — for the fixed apparatus rung B3: `Δ_release = r_{k+1}(B3) − r_k(B3)`,
     and the change in lift `L_{k+1} − L_k`, each with a bootstrap CI; and the OLS slope of `r_k(B3)`
     on release index `k` across the swept releases (the "positive across releases" quantity, P19).

All six are pure functions of the tagged `Report` set; the harness stays model-free (it aggregates
scored JSON, it does not drive a model). This is the spec the build must satisfy.

---

## 5. Sampling, and anti-circularity / anti-family-bias safeguards (pre-conditions)

5.1 **N and samples-per-task.** The prior runs were **N = 1** (one submission per task) — underpowered:
a rate with no error bar cannot carry a slope. Pre-registered minimums for a VERDICT run:

- **S ≥ 5 samples per task** per `(release, rung)` cell, drawn at a **fixed non-zero decoding
  temperature** (declared and frozen), so the rate captures the model's stochasticity rather than a
  single lucky/unlucky draw.
- **T = the frozen anchor set** (§8), which must be a **harder, non-saturating** set (§6.4, §9), not
  the 23-task ceiling set. With `T·S ≥ 115` trials per cell, the per-rung Wilson half-width at
  `r ≈ 0.5` is ≈ ±0.09 — the pre-registered floor on resolvable lift; a smaller detectable effect
  requires more samples, stated so power is a *derived* number, not an afterthought.

5.2 **Anti-circularity (pre-condition, from the harness rule).** Anchors are authored **from the frozen
basket specs and the spec pack**, and are **never model-generated** (`tools/eval-harness/` README, the
Bet 6 anti-circularity note; P19). The oracle (`candor-proto`) filters for internal consistency; the
*anchors* are what make a pass mean "matches an externally fixed intent." A run whose anchors were
generated by any model — including corpus-gen output — is **void** for a Bet 6 verdict.

5.3 **The corpus/eval wall (pre-condition).** `tools/corpus-gen/` produces **training** material only
and is **strictly walled off** from the eval anchors (its README §"the circularity guard": "The corpus
is TRAINING material. It is never an evaluation anchor," one-directional separation). A verdict run
must attest that no anchor, battery, or exemplar was sourced from the corpus pipeline. Adaptation
budget (§4) *may* include corpus/fine-tuning material as an input **to the model**, but never as an
anchor **scoring** the model — the wall is between what trains and what grades.

5.4 **Anti-family-bias (pre-condition — the graduation README's own caveat, elevated to a gate).** The
`2026-07-09-round2-graduation` README records that a task set "authored within the same broad model
family as the model-under-test may systematically underestimate difficulty for that family," and names
the needed controls. This criterion **binds** them:

- The **model-under-test must be cross-family** relative to the task authors — i.e. tasks authored by
  (or the frozen basket ported by) a party independent of the model whose competence is scored. Same-
  family author-and-subject is **exploratory only**, never a verdict (§9). This mirrors
  `BET5_CRITERION2.md`'s freeze-manifest ruling that the Candor ports may not be authored by the same
  model family that produced the baselines (`FREEZE_MANIFEST.md` step ii).
- The **zero-budget anchor B0** (§3.3) is itself an anti-family-bias control: if a cross-family model
  already passes at B0, the "competence" is pre-trained transfer, not manufactured, and the lift `L`
  correctly reads zero.

5.5 **Why cross-family authorship matters.** If the same family authors the tasks and takes the test,
a ceiling result is unfalsifiable — it cannot distinguish "the apparatus manufactured competence" from
"the tasks are easy for this family's priors." Cross-family authorship is what makes a *pass* mean the
apparatus generalized beyond the distribution that wrote the anchors.

---

## 6. Thresholds and verdict rule

6.1 **Pre-conditions gate the verdict (checked first).** A run is scored as a **VERDICT** only if all
§9 operator preconditions hold (cross-family subject, non-saturating frozen task set, S ≥ 5, declared
tokenizer + temperature, ≥ the required release count). A run failing any is **EXPLORATORY** — recorded
and published, but it may not confirm, refute, or kill Bet 6. The prior three runs are exploratory by
this rule.

6.2 **CONFIRM (provisional, on these anchors and releases)** iff **all** hold:
- **(a) Manufactured lift is real:** `L = r(B3) − r(B0)` is significantly positive — its cluster-
  bootstrap 95% CI excludes 0 (§4.4). Something was manufactured above the zero-budget counterfactual.
- **(b) Budget-slope is positive:** the OLS budget-slope (§4.4.4) is positive with a CI excluding 0 —
  competence rises with the project's adaptation material, not by accident of one rung.
- **(c) Release trend is non-negative and positively sloped:** across ≥ 3 swept releases, the OLS slope
  of `r_k(B3)` on release index (or of lift `L_k`) is ≥ 0 with the CI excluding a *negative* slope, and
  positive on at least one of {rate-at-B3, lift, efficiency}. This is P19's "positive across releases."
- **(d) No corpus collapse:** the corpus-quality / distribution guard (§6.5) does not fire.

6.3 **REFUTE / KILL (Bet 6 fails; §9 amendment mandatory)** if **any** hold on a VERDICT run:
- **(a) Flat budget-slope with a non-saturating set:** `L`'s CI includes 0 **and** the ceiling guard
  (§6.4) confirms the set is not saturated — i.e. there is headroom the apparatus fails to fill.
  Competence is not manufactured: adding the project's material changes nothing.
- **(b) Flat or negative release trend** across ≥ 3 releases (the P19 flat-slope kill).
- **(c) Corpus-quality collapse** toward the generator's own distribution (§6.5) — P19's second named
  kill.

6.4 **Ceiling guard (the prior-rounds failure, made detectable and disqualifying).** A run whose frozen
task set is **saturated** cannot produce a verdict. Saturation is declared if **either** the zero-budget
anchor `r(B0)` for the frontier model-under-test is already ≥ a pre-registered ceiling (proposed **0.85**
— the apparatus has little room to add value) **or** the full-apparatus rate `r(B3)` is at ceiling
(≥ **0.98**) with `L` below the §5.1 resolvable half-width (no measurable lift because there is no
headroom). A saturated run is **VOID for a verdict** (not a KILL — absence of headroom is a task-library
defect, not evidence against the bet) and routes to the §9 mandatory action "commission a harder
independently-authored set." This is precisely the disposition the three 2026-07-09 runs earn: ceiling,
therefore void-for-verdict, therefore exploratory.

6.5 **Corpus-quality / distribution-collapse guard.** P19 names "corpus-quality collapse toward the
generator's own distribution" as a kill. Operationalized as a pre-condition check independent of the
pass-rate: if the model-under-test's *passing* submissions converge on a narrow set of near-identical
templates (a degenerate-diversity signal, defined and thresholded at ratification) rather than spanning
the anchors' intended idiom space, the apparent competence is corpus-distribution echo, and §6.3(c)
fires. Idiom quality is otherwise **not** scored by the harness (its README §"what this seed does NOT
measure"); this guard is the minimum distribution check the kill clause requires, not a full idiom
grader.

6.6 **Mandatory review (no auto-verdict), mirroring `BET5_CRITERION2.md` §6.4.** A VERDICT run that is
neither a clean CONFIRM nor a clean KILL — e.g. positive lift but a flat release trend over only 2
releases, or a fired corpus guard with an otherwise-positive slope — routes to a **mandatory §9 review**
that cannot silently pass: the authority records a ruling — *provisionally confirm*, *commission harder
anchors / more releases*, or *escalate to KILL* — with reasoning and any dissent, entered in the ledger.

6.7 **KILL / re-open rule.** A KILL is enacted as a §9 amendment naming Bet 6 and P19, stating the
evidence (the flat/negative slope and the non-saturation proof), appended to the philosophy's
Appendix A. Bet 6 **re-opens** — a new VERDICT run may be scored — only when a *materially different*
input arrives under the frozen rules: a new model release, or a harder independently-authored anchor
set that passes the §6.4 non-saturation check. Re-running the *same* releases on the *same* anchors
after a KILL is not a re-open (it cannot change the frozen inputs), exactly as `BET5_CRITERION2.md`
§9.1 forbids re-porting frozen objects to chase a known verdict.

---

## 7. Verdict-run procedure (mechanical, public, computed after this document freezes)

7.1 **Inputs — frozen only.** The frozen anchor set and budget ladder (§8), a built `candor-proto`
oracle at a recorded commit, the declared reference tokenizer, and the per-`(release, rung)` submission
sets produced by the operator driving each model over each rung's adaptation material (§9). No anchor
authored after the verdict is known; no post-hoc rung added.

7.2 **Procedure.** (1) For each `(release, rung)` cell, the operator collects S ≥ 5 submissions per task
and runs `eval-harness score` per submission, tagging the resulting `Report` with `(release_id, rung,
B_ctx, B_fb)`. (2) The extended `SlopeReport` aggregation (§4.4) computes per-rung rates + CIs, the
adaptation curve, lift, budget-slope, efficiencies, repair sub-slope, and cross-release delta. (3) The
ceiling guard (§6.4) and corpus guard (§6.5) are evaluated. (4) The §6 decision rule is applied. (5) The
full computation — every rate, CI, slope, the tokenizer + temperature used, per-cell task tables — is
published as the act that freezes this document (§1.1).

7.3 **This document stops short of executing §7.2.** The numbers are produced by the public verdict run,
not asserted here (§0.3). No slope value and no verdict is stated in this registration.

---

## 8. What gets frozen, and when

8.1 **Frozen at ratification** (hashes into `docs/FREEZE_MANIFEST.md`, a new "Bet 6" section mirroring
the Bet 5 sections):
- this criterion (`docs/BET6_CRITERION.md`) at its ratification hash;
- the **anchor task set** — the frozen `tools/eval-harness/tasks/*.json` used for the verdict, plus any
  harder independently-authored additions (§9), each hashed; the anchors remain authored from the frozen
  basket specs, never model-generated (§5.2);
- the **budget ladder** (§4.2): the file→rung mapping and each rung's measured `B_ctx`;
- the **reference tokenizer** identity and the decoding temperature (§4.1, §5.1).

8.2 **Frozen per verdict run** (hashed as produced): every per-`(release, rung)` results `Report` JSON,
the aggregated `SlopeReport` JSON, and the run README stating model releases, tokenizer, temperature,
S, and the pre-condition attestations (§5.2–§5.4). Published whether the outcome is CONFIRM, KILL,
mandatory review, or VOID/exploratory (philosophy §3, §7; `BET5_CRITERION2.md` §8.1).

8.3 **Never frozen retroactively.** A defect found in this criterion after its freeze is itself
published and the philosophy amended under §9; the registration is never silently retrofitted to a
result already seen (`BET5_CRITERION2.md` §0.4).

---

## 9. Operator preconditions (gates the lab cannot self-serve)

These are the conditions that separate a graduation-tier **VERDICT** run from an **EXPLORATORY** one
(§6.1). They exist because the lab, running solo, cannot manufacture them for itself — and a verdict
scored without them would be exactly the family-biased, ceiling-saturated non-result the prior three
rounds already are.

9.1 **Cross-family model access.** At least one model-under-test from a family **different** from the
party that authored the anchors / ported the basket (§5.4). Without it, a ceiling result is
unfalsifiable and the run is exploratory only.

9.2 **A harder, independently-authored, non-saturating task set.** The current 23-task set saturates
(§6.4); a verdict needs a set on which the frontier model-under-test is demonstrably **below ceiling**
at B3 and **well below** ceiling at B0 — with headroom for the apparatus to fill. Authored by a party
independent of the model-under-test's family, from the frozen basket specs (§5.2). This is the single
largest blocker the graduation campaign faces and cannot be satisfied by re-running the existing set.

9.3 **A release series.** ≥ 3 model releases (or a genuine release ladder) to compute the "positive
across releases" trend (§6.2c). With only 2, the best available outcome is a mandatory-review provisional
read (§6.6), never a clean confirm.

9.4 **Sampling and accounting fixtures.** S ≥ 5 samples per task at a fixed declared temperature (§5.1);
a fixed reference tokenizer for the budget unit (§4.1) — both declared and frozen at ratification.

9.5 **The built slope aggregation.** `report.rs`'s `SlopeReport` extension (§4.4) exists and is tested
before a verdict run — the per-round rate the harness computes today is not sufficient to score any of
the §6 clauses.

---

## 10. Explicitly rejected alternatives (rationale is the product)

10.1 **A single pass-rate threshold** (e.g. "≥ 90% first-attempt confirms") — *rejected*, §2.3: a rate
is a floor, not a slope; it cannot distinguish manufactured competence from pre-trained transfer, and
the prior 100% runs would spuriously "confirm."

10.2 **Cross-model / Candor-vs-Rust comparison as the metric** — *rejected*, §3.2: P19 forbids it
(corpus-scale confound). Used only inside the family-bias controls.

10.3 **Task-difficulty as the swept axis** — *rejected*, §3.2: measures the task library, not the
adaptation apparatus. Retained as the ceiling-guard control, not the claim.

10.4 **Dropping the zero-budget anchor B0** — *rejected*, §3.3: without the counterfactual, lift is
undefined and a ceiling result cannot be told apart from a manufactured one — the exact circularity
this bet must avoid.

10.5 **Scoring the existing 23-task set as a verdict** — *rejected*, §6.4/§9.2: it saturates; a
saturated set is void for a verdict by construction.

10.6 **N = 1 sampling** — *rejected*, §5.1: no error bar, no slope. Underpowered by the prior rounds'
own admission.

---

## 11. Independence residual

Full third-party independence is unavailable to a solo project and not claimed (`BET5_CRITERION2.md`
§8.4). The same model family that orchestrates this project also assists its reviews; the criterion
constrains only the *scored* relationship — anchors independent of the model-under-test's family
(§5.4), corpus walled from eval (§5.3), anchors never model-generated (§5.2) — and names the residual
rather than claiming it away (philosophy header). These reduce but do not eliminate the §0.1 conflict.
