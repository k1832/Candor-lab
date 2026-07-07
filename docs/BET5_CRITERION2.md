# Bet 5 — Successor Pre-Registered Kill Criterion (v2 of the registration; SUCCEEDS BET5_CRITERION.md)

**Artifact type:** Scientific pre-registration. Not a design document, not a pitch.
It fixes, in the open, the corrected conditions under which Bet 5's *ordering claim* is declared
**killed** on the frozen validation basket. It is the successor required by `LANG_PHYLOSOPHY.md`
§3 (Bet 5 verdict clause, v4→v4.1) after the first registration (`BET5_CRITERION.md`, the *dead v1
registration*) computed KILL with published operationalization defects (`docs/RESULTS.md` §0.3, the
final-verdict block, and the measurement-observation block).

**Status:** PRE-REGISTERED — *data-aware* (see §0). Not yet frozen.
**Registration version:** SUCCESSOR-1.
**Date:** 2026-07-07.
**Governing document:** `LANG_PHYLOSOPHY.md` v4.1 (normative; this sits at the design-document tier
under §9 and binds the re-scoring, not the philosophy).
**Adjudicating authority:** the single deciding authority in `GOVERNANCE.md` (philosophy §9),
currently k1832 — same authority, same open-comment discipline as v1 (§6.5/§6.7 there).
**Predecessor:** `BET5_CRITERION.md` (v2 of that document), frozen at instant `e22ca51`, **never
modified**. This document does not edit it; it *succeeds* it, changing only the operationalization the
v1 defect analysis identified.

---

## 0. Honesty preamble — this registration is written after the data is public

0.1 **The conflict, named plainly.** Unlike v1, this document is authored *after* the artifacts it
will score exist and after their measurements are public (`docs/measurements/`, `docs/RESULTS.md`,
`docs/ADJUDICATIONS.md`). The author can see, at drafting time, roughly what any candidate metric
would compute. That is a real threat to the pre-registration's meaning, and no amount of wording
dissolves it. It is stated here, at the top, where every reader starts — as the philosophy's own
header demands of admitted costs: a named conflict is a borne cost, not a managed one.

0.2 **The legitimacy basis is four constraints, not a claim of innocence.**

- **(a) Minimal change.** This registration changes **only** the valve-ambiency operationalization
  (v1's M2 and M3). Everything else is **carried over unchanged and is binding here by reference**:
  the claim under test (v1 §1), the validation artifact and basket (v1 §2), the frozen unit/
  classification table and all counting scripts (v1 §3.1–3.5), the annotation metric **M1** (v1 §4.1),
  the combined-load metric **M1b** (v1 §4.1b), the copy-blow-up metric **M4** (v1 §4.4), the
  completability gate **M5** (v1 §4.5), the supplementary M6/M7 status, the R14 implementation/harness
  split, the frozen Rust baselines and their recorded valve fractions (v1 §6.6), and the adjudication
  mechanics (blind classification order, §6.5 open-comment, §6.7 solo-project honesty). Their computed
  results carry over **as already computed** (§6.1). The surface that changes is exactly the surface
  v1's own §0.3 defect analysis flagged, and no larger.

- **(b) Thresholds derived by principle, not chosen freely.** Every threshold below traces either to a
  **v1 margin carried forward** or to a **named principle**, with the derivation shown inline (§4).
  No number in this document's gating metric is picked to land a verdict; each is a v1 constant or a
  P12/P13/finding-4 consequence. Where v1's *base* was defective (the fraction denominator), the base
  is corrected and the *margin* (e.g. 1.25×) is carried unchanged onto the corrected base — that carry
  is stated explicitly each time.

- **(c) Adversarial review before freeze.** This document is reviewed by a session briefed to hunt
  specifically for outcome-reverse-engineering — to ask of every choice "does this pick land a verdict
  the author already knows?" Its findings are recorded in the §0.5 ledger before the freeze instant,
  exactly as v1's review findings were.

- **(d) It binds future re-validations; it is not single-use.** These rules apply, unmodified, to any
  future extension of the basket or re-port under the same claim: if a sixth program is added, or a
  program is re-ported, the corrected valve metric and decision rule here govern it without
  re-registration. The document is written to be reused, not to score one frozen set once.

0.3 **What this document must not do, and how it is checked.** It must be written **metric-first**: the
corrected valve verdict is **not computed anywhere in this document** and **no expected overall
verdict is stated**. The re-scoring (§7) is a *separate public act performed after this document
freezes*. The adversarial reviewer (c) judges whether the author held metric-first discipline despite
knowing the data. The re-scoring outputs — not this text — carry the numbers. **(Amended per review finding 2.)** Appendix A now discloses, as a normative anti-laundering measure, the per-program valve-metric values on the exact unit that the §7 re-scoring will confirm — the rejected-alternative and reference tables the reviewer required be published rather than hidden. That appendix states **per-program metric outcomes only**; the **combined overall verdict remains uncomputed in this document** and is the post-freeze re-scoring act, preserving the metric-first discipline this section demands.

0.4 **After the freeze, nothing here changes.** Like v1 §0.3: a defect found in *this* criterion after
its freeze is itself published and the philosophy is amended under §9; this registration is never
silently retrofitted to a result already seen.

0.5 **Amendment ledger (pre-freeze changes only).**

| # | Date | Section | Change | Rationale | Enacted by |
|---|------|---------|--------|-----------|------------|
| 0 | 2026-07-07 | all | Initial draft of the successor registration | Philosophy §3 (v4.1) mandates a data-aware successor changing only the defective operationalization | deciding authority (k1832) |
| 1 | 2026-07-07 | §4.1 | Operative unit set to the exact `valve_statements` intersection now emitted by both counters (`unit_ext_version` 2); the draft's false `per_site`/"no new metric" claim corrected; the freeze-before-ratification requirement discharged; VSE demoted to a reported-only cross-check | Review finding 1 (accepted, option (b)), `docs/reviews/2026-07-07-criterion2-review-1.md` | deciding authority (k1832) |
| 2 | 2026-07-07 | Appendix A | Added the normative appendix disclosing the outcome tables (exact unit, rejected raw-absolute-lines alternative, v1 reference, VSE cross-check) | Review finding 2 (accepted) — non-disclosure was the draft's worst self-inflicted wound | deciding authority (k1832) |
| 3 | 2026-07-07 | §4.3, §6.2(d), §6.3, §7.2 | V2 zero-baseline floor re-derived on the statement unit (`valve_statements / logical_statements`), carrying v1's 0.15/0.08 margins; decision-rule and field-mapping references updated to match | Review finding 3 (accepted) — the floor must not revert to the condemned line-fraction unit | deciding authority (k1832) |
| 4 | 2026-07-07 | §4.5 | stdlib-substitutable keyed solely to the frozen spec's required features (a spec §2.4 required-feature valve is never substitutable); the soft "prototype lacks a stdlib" limb deleted | Review finding 4 (accepted) | deciding authority (k1832) |
| 5 | 2026-07-07 | §5.2 | Allocator-class carve-out restated as a KILL→mandatory-review reversal under §6.4, never a silent pass | Review finding 5 (accepted) | deciding authority (k1832) |

---

## 1. Freeze instant and change lock (this document)

1.1 **Change lock.** Thresholds, definitions, and the decision rule here may change **only before the
re-scoring (§7) is published**; after that instant this document is **frozen** — nothing added,
removed, loosened, tightened, or re-normalized (v1 §0.1 discipline, relocated to the only freeze point
a data-aware registration has: the mechanical re-scoring is the act this must not be retrofitted to).
Any pre-freeze change is a numbered §0.5 ledger row (old/new/date/rationale/enactor) by the deciding
authority; adversarial-review findings (§0.2c) enter the same way; unrecorded changes have no force.
The carried-over v1 artifacts (§0.2a) are frozen *there*; this lock governs only §4 (valve metric),
§5 (carve-out), and §6 (decision rule).

---

## 2. The claim under test (unchanged from v1 §1)

2.1 Bet 5: **value-first defaults lower cognitive load for real systems code.** The only delta on
trial is the **default-gear ordering** — value first, borrowing second, valves third (P12; v1 §1.2);
body inference and compact signatures are Rust's too and are not on trial. The valve half, in the
philosophy's words: valves stay **"rare in occurrence even where critical in function."** v1
operationalized "rare in occurrence" as a *fraction* of the program; §3 records why that was
defective and §4 replaces it. The load half (M1/M1b/M4) and completability (M5) are unchanged and
already computed. Killing is enacted as a §9 amendment, never worked around (P12).

---

## 3. The three defects being corrected (from `RESULTS.md` §0.3 and the measurement observation)

3.1 **Defect (i) — the fraction denominator punishes density.** v1 measured
`valve_line_fraction = valve_lines / total_lines`. Candor implements the same frozen spec in **2–5×
fewer lines** than idiomatic Rust, so the denominator shrinks and *identical or lower absolute valve
content yields a higher fraction*: the two allocators carry comparable absolute valve content (Candor
112 valve lines, Rust 179) yet Candor's fraction is 0.6292 vs 0.1929 "because Candor needed far less
safe scaffolding" (`RESULTS.md`). The fraction conflates "the valve is the program" with "the
non-valve code got lean."

3.2 **Defect (ii) — small-N function counts make the function-fraction ceiling degenerate.** v1's M2
also gated on `valve_function_fraction`. MMIO's port realizes the design's *own prescribed ideal*
architecture — two one-pointer-op register accessors (`reg_read`/`reg_write`) — in an 8-function
program, giving 2/8 = 0.2500 against a 0.20 ceiling. Ruling R28: "no honest refactor reduces the
fraction … the ideal valve architecture fails the function-fraction ceiling purely because the program
is 8 functions." A ceiling that a program fails *by being small and well-factored* measures program
size, not valve ambiency.

3.3 **Defect (iii) — stdlib asymmetry.** The validation prototype ships "no standard library beyond
slices" (v1 §2.1). The Candor parser's *only* valve is bump-allocator infrastructure it must hand-roll
to hold its AST; the idiomatic Rust baseline obtains the same capability from `std` (`Box`, `Vec`)
**invisibly, at zero valve cost** (baseline parser valve fraction 0.0000). Candor is charged for the
prototype's deliberate absence of a facility the production language (P9) would provide as
subtractively layered stdlib. v1 had no rule for this; §4.5 adds one, principled and symmetric.

---

## 4. The corrected valve metric (replaces v1 M2 and M3)

4.1 **Unit — valve statements (the exact intersection), now emitted by the frozen counters.**
v1's own finding 4 established the governing principle: *do not compare across two non-commensurable
lexers; compare at the shared normalized unit of the logical statement* (an AST-derived node). v1
applied this to annotation (M1) but left valves counted in **physical lines** — the residue of the
old ruler, and the seat of defect (i). The correction carries finding 4 into the valve metric.

- **Operative unit — valve statements (direct, exact).** A logical statement is a *valve statement*
  iff its AST-node byte span **intersects** any valve region's span without strictly enclosing it —
  i.e. statements inside, or partly inside, a valve region (`unsafe` blocks / `unsafe fn` bodies /
  raw-pointer regions, per the frozen unit table's valve-region rules), while the enclosing fn or
  block that merely *wraps* a valve is excluded. `V = valve_statements`.
- **Correction of the draft's false claim (review finding 1).** The predecessor draft asserted this
  intersection was "a mechanical query over the same frozen artifacts … adding no new metric," on the
  ground that "the frozen counter already emits the per-site valve spans (`per_site`)." **That was
  wrong:** `per_site` holds **annotation** sites, not valve regions; the counters' `valve_spans` were
  internal and never serialized; and no per-statement spans existed. The exact unit therefore
  required new counter code. That code has now been **written, reviewed, tested, and frozen before
  ratification** (finding 1, option (b)): both counters emit an additive `valve_statements` field
  under `unit_ext_version` "2" (`prototype/src/count.rs`, `tools/rust-count/src/lib.rs`), with every
  `table_version` "1" field unchanged and recomputed identically. The freeze-before-ratification
  requirement finding 1 imposed is **discharged by this work**; the re-measured counts live in
  `docs/measurements/{ports,baselines}/<prog>.v2.json` and are tabulated in Appendix A.
- **VSE demoted to a reported-only cross-check (finding 1).** The old fallback
  `VSE = valve_lines × (logical_statements / total_lines)` is **no longer operative** — the direct
  count supersedes it. VSE survives in Appendix A **only** as a reported cross-check, with its named
  limitation intact: it assumes valve regions share the whole-program statement density, and where
  the baseline's valve is concentrated in a dense region (e.g. the scheduler's vendored intrusive
  list) it **over-estimates** the ratio. Appendix A shows exactly this — VSE puts the scheduler at
  1.807 (a spurious KILL) where the exact count is 0.528.
- **Why not raw valve lines.** Rejected for the *same reason v1 rejected raw token fractions* (finding
  4): a Candor line packs ~2–3× the statements of a Rust line, so raw valve-line comparison is
  incommensurable and — because Candor is the denser language — *flatters* the bet. Appendix A
  quantifies the flattery: on raw valve lines the allocator is 0.626 (a comfortable pass), while on
  the density-corrected statement unit it is 1.116 (a WARN). The unit is chosen for methodological
  consistency with v1, not for the direction it happens to push — it in fact pushes *against* the bet
  on the hardest program.

4.2 **Primary gating metric V1 — spec-relative valve content (comparative, all non-carve-out
programs).** Both languages implement the **same frozen functional spec** (v1 §2.3), so the idiomatic
Rust baseline fixes how much valve work the spec inherently demands (this is the "valve share of
SPEC-MANDATED pointer work" candidate, operationalized through the same-spec baseline rather than a
hand-drawn denominator). Per program with `V_rust > 0` and not carved out under §5:

    R_valve = V_candor / V_rust        (V per §4.1; per-program, no aggregation across programs)

- **KILL** if `R_valve > 1.25`.
- **WARN** if `R_valve > 1.00`.

  *Derivation.* The **1.25× margin is carried unchanged from v1 §4.2's home-ground ceiling**
  (`1.25 × valve_line_rust`) — v1's own chosen answer to "how much more valve than idiomatic Rust is
  tolerable where the bet is hardest." v1 applied it only to home-ground and only on the defective
  fraction base; the correction applies the *same margin* to the *corrected content base* and, because
  same-spec baseline comparison already normalizes for how pointer-rich each workload is, applies it
  **uniformly** rather than splitting home-ground from value-favorable. The **1.00× WARN is carried
  from v1 §4.3 (M3)**, which WARNed at `valve_line_candor > valve_line_rust` (parity) — carrying more
  spec-normalized valve content than idiomatic Rust is the same "actively counterproductive where it
  matters" signal, now on the corrected base. This operationalizes "rare in occurrence even where
  critical in function" **better than a raw fraction did**: it asks whether value-first *forces more
  valve than the specification inherently needs* (controlling for both program density and workload
  pointer-richness, because the baseline implements the identical spec), instead of asking whether the
  valve is a large share of a program that value-first made lean — the exact conflation of defect (i).

4.3 **Zero-baseline floor V2 — absolute value-favorable ceiling (for `V_rust = 0` programs).** When
the idiomatic Rust baseline carries **zero** valve statements, `R_valve` is undefined and the spec
inherently demands no pointer work; any Candor valve is "extra." The principled floor is **v1's own
value-favorable absolute ceiling, its margin carried over unchanged** (v1 §4.2), re-based onto the
corrected statement unit (finding 3) so the floor does **not** revert to the condemned line-fraction
ruler: on the non-stdlib-substitutable valve content (§4.5), as a **valve-statement fraction**
`valve_statements / logical_statements`,

- **KILL** if the fraction `> 0.15`;
- **WARN** if the fraction `> 0.08`.

  *Derivation.* The **0.15 / 0.08 margins are v1's value-favorable thresholds verbatim**, not
  re-chosen; only their *base* moves from valve-lines to valve-statements, exactly as defect (i) and
  review finding 3 require (a zero-baseline floor must not smuggle the condemned line-fraction unit
  back in). A value-favorable program is exactly where v1 set an *absolute* rarity bar, and a program
  whose same-spec Rust needs no valve at all is value-favorable by revealed demand. Appendix A
  confirms the re-based floor changes **no** zero-baseline verdict: parser (`9/311 = 0.0289`) and
  arena (`0/111 = 0.0000`) both pass, as under v1.

4.4 **Function-fraction — retired to reported-only (corrects defect (ii)).** `valve_function_fraction`
is **removed from all gating** and reported for continuity only. *Derivation.* This mirrors v1's own
move of demoting non-commensurable/size-sensitive measures to reported-not-gating (finding 4; v1 §8.3,
§8.4) and is compelled by ruling R28's finding that the ideal architecture fails it purely by
function count. Valve *prevalence* is already governed by V1/V2 on the corrected content unit; a
second gate that penalizes small, well-factored programs measures size, not ambiency. The fraction is
still reported so the record is complete.

4.5 **stdlib-substitutable regions — carve-out for prototype-absent-stdlib valves (corrects defect
(iii)), mirroring v1's cell-substitutable mechanic.** A valve region may be tagged
**stdlib-substitutable** iff **both** hold: (1) it does **not** implement any feature the frozen
functional spec **requires** of the program (spec §2.4); **and** (2) its idiomatic Rust baseline
obtains the equivalent capability from `std`. The definition is keyed **solely to the frozen spec's
required-feature list** (review finding 4), so it is principled and not stretchable:

- A valve that implements a **spec §2.4 required feature is never stdlib-substitutable** — full stop.
  The allocator's free-list pointer work (spec §2.4a) and the scheduler's intrusive linkage (spec
  §2.4b) are required deliverables and can never be tagged, whatever argument is made about libraries.
- **Only** a valve **outside** the spec's required-feature set — pure infrastructure the port
  hand-rolls because the prototype ships no standard library (v1 §2.1) and whose capability `std`
  supplies directly, e.g. a parser's bump arena holding its AST — is eligible.
- The soft "only because the prototype lacks a stdlib" limb of the predecessor draft is **deleted**:
  eligibility is no longer argued from what production Candor *would* provide (a stretchable
  counterfactual), but decided by the frozen spec's required-feature list, which is fixed.

  *Mechanic (carried from v1 §4.2 cell-substitutable, unchanged).* Port authors tag; the adjudicator
  confirms or rejects each tag with recorded public reasoning under §6.5's open-comment discipline.
  The gating metric (V1/V2) is always computed on the **full** valve count first. **If — and only if —
  excluding adjudicator-confirmed stdlib-substitutable regions would reverse a KILL or WARN, the
  outcome is a mandatory §9 review** (which may still escalate), never a silent pass. This keeps the
  metric's teeth while ensuring the prototype's deliberate stdlib omission cannot, by itself, kill the
  bet on a program whose only valve is a facility production Candor ships. *Derivation.* Pure re-use of
  v1's upper-bound-honesty mechanic (v1 §4.2), re-pointed from "checked-runtime alternative existed" to
  "std facility existed"; both correct the same class of error — a valve the *production* language
  would not require in user code inflating the *prototype's* count.

4.6 **Rejected alternatives for the valve metric (rationale is the product).**
- **Raw absolute valve lines vs Rust (no density normalization)** — *rejected*, §4.1: incommensurable
  lexer artifact, and it flatters the denser language (Candor). Choosing it would be reverse-engineering
  toward the bet.
- **Density-normalized fraction `candor_fraction × (candor_stmts / rust_stmts)`** — *rejected*:
  scaling a program's fraction by a cross-program size ratio is dimensionally ad hoc and *flatters* the
  leaner language — correcting defect (i) in the direction that helps the bet, the choice this
  registration must not make. §4.2 corrects defect (i) by controlling for the spec, not by a size
  multiplier.
- **Keeping the fixed 0.40 home-ground absolute floor (v1 §4.2)** — *rejected*: its job was to catch
  "the valve is the program," which P12 v4.1 has now *conceded* for allocator-class code (§5); keeping
  it would re-litigate a conceded point (double jeopardy) on the very fraction base defect (i)
  discredits.
- **Keeping any function-fraction gate** — *rejected*, §4.4 / R28.

---

## 5. The allocator-class carve-out (per P12's v4.1 concession)

5.1 **What the concession is.** P12 v4.1 records a named limit: **value-first does not carry
allocator-class code — "programs whose core is in-band metadata over raw memory (free lists threaded
through the blocks they describe) … the valve is the program's spine."** The philosophy has already
paid this finding into the record as an amendment.

5.2 **Consequence for scoring — decision.** For a program the adjudicator confirms **allocator-class**
under P12's definition, the valve metric (V1/V2) is **still computed and reported in full**, but a
would-be valve **KILL is reversed to a mandatory §9 review under §6.4 — never a silent pass** (review
finding 5). *Argument:* the philosophy has **already conceded** the valve is that program's spine;
killing Bet 5 again on the identical, already-amended finding is **double jeopardy** (§9 enacts a
verdict once). The reversal removes only the *automatic* KILL: the valve outcome does not vanish — it
routes to the recorded mandatory review of §6.4, which may still escalate to KILL with stated reasons,
and any WARN it raises still counts toward §6.4's triggers. The carve-out is **valve-only**: LOAD
metrics **M1, M1b, M4** and the gate **M5 still apply in full** to allocator-class programs — the bet
can still KILL there on load or incompletion. It removes only the double-counted *automatic* valve
KILL, never converting a measured breach into a silent pass.

5.3 **Scope of the class — decided narrowly, to preserve falsifiability.** The carve-out attaches to
programs the philosophy has **conceded** are allocator-class. P12 concedes **the allocator** by name
("the measured allocator's lines"). **Any other program enters the carve-out only by an explicit §6.5
adjudication** that it meets P12's "in-band metadata over raw memory" definition, published with
reasoning and open to comment; absent that ruling its valve metric **gates normally.** In particular
the intrusive scheduler's valve **gates by default** and is carved out only if the adjudicator rules,
publicly, that embedded-linkage intrusive structures fall within P12's conceded class.

  *Argument, and the alternative recorded.* The **alternative** is a pure class reading: auto-carve
  *every* program matching "in-band metadata over raw memory," which — since the intrusive scheduler
  threads linkage through its entities — would carve out *both* home-ground programs' valves
  automatically. **Rejected** for two reasons. (1) **Falsifiability:** auto-carving both hard programs
  would leave the valve metric unable to KILL on any home-ground workload, and combined with the
  already-passed carried-over LOAD metrics that risks a criterion that *cannot* fail — precisely the
  reverse-engineering hazard §0 exists to guard against. (2) **Fidelity to the concession:** P12
  conceded the allocator specifically; extending the concession to a second program is a *new*
  judgment that belongs in the open adjudication record, not presumed by this registration to hit a
  number. The narrow reading keeps the bet genuinely falsifiable on home ground and puts the burden of
  proof on any expansion of the carve-out, where §9 puts it.

---

## 6. Decision rule (corrected valve metric combined with the carried-over metrics)

6.1 **Carried-over results, stated as already computed (not recomputed here).** From the frozen
artifacts (`RESULTS.md`), binding here unchanged:
- **M5 (completability):** all five programs completed in Candor — **pass** (carried).
- **M1 (annotation load):** worse-of aggregate ratio **0.474** vs KILL 0.90 / WARN 0.75 — **pass,
  no WARN** (carried).
- **M1b (combined load):** ratio **0.472** vs KILL 1.00 / WARN 0.85 — **pass, no WARN** (carried;
  zero value copies).
- **M4 (copy blow-up):** 0 vs 1 copies — **no WARN** (carried).

  These are inputs, not this document's subject. The **only** computation this registration newly
  specifies is the corrected valve metric (§4–§5); it is performed in §7's public re-scoring, **not
  here.**

6.2 **KILL (bet's ordering claim fails; §9 amendment mandatory)** if **any** of:
- (a) any basket program not completed in Candor — **M5** (carried: does not fire);
- (b) **M1** aggregate KILL — `AGG_candor > 0.90 × AGG_rust`, worse of weighted/mean (carried: does
  not fire);
- (c) **M1b** aggregate KILL — `AGG_combined_candor > AGG_combined_rust`, worse of weighted/mean
  (carried: does not fire);
- (d) any **per-program valve KILL** (§4): for a non-carve-out program with `V_rust > 0`,
  `R_valve > 1.25` (V1); for a `V_rust = 0` program, non-stdlib-substitutable **valve-statement
  fraction** `valve_statements / logical_statements > 0.15` (V2). These are **absolute per-program
  floors** that strong results elsewhere cannot
  offset — subject only to the stdlib-substitutable reversal path (§4.5) and the allocator-class
  carve-out (§5), under which the affected program's valve outcome becomes a mandatory §9 review
  rather than an automatic KILL.

6.3 **Anti-masking (v1 §5.2 spirit, preserved).** Aggregates (M1, M1b) use the **worse** of
statement-weighted and unweighted mean. Per-program valve floors stand independently — no averaging,
no basket subsetting, no "4-of-5." Value-favorable programs with `V_rust > 0` gate on the **worse** of
{V1 comparative ratio, V2 absolute 0.15 valve-statement ceiling}, so a program cannot pass a
comparative bar while carrying an absolutely un-rare valve.

6.4 **Mandatory §9 review, no KILL.** Count WARN triggers across **M1, M1b, M4, and the corrected
valve metric (V1/V2)** — the function-fraction feeds no count (§4.4); M6/M7 remain supplementary
(v1 §4.6–4.7):
- **Any** WARN on the **allocator or scheduler** (home-ground sensitivity, philosophy §3) →
  mandatory review;
- **Two or more** WARN triggers in total → mandatory review;
- A carve-out or stdlib-substitutable **reversal** outcome (§4.5, §5) → mandatory review.
- A mandatory review **cannot silently pass**: the authority produces a recorded §0.5 ledger ruling —
  *proceed*, *re-scope the design*, or *escalate to KILL* — with reasoning and any dissent.

6.5 **Provisional confirmation.** If no KILL fires, fewer than two WARNs fire in total, and neither
home-ground program triggers any WARN, the *ordering claim* is **PROVISIONALLY CONFIRMED on this
basket** (scope-limited exactly as v1 §5.4; a pass licenses the syntax freeze per philosophy §3/§8.5
and nothing more). "Provisional" and "on this basket" are load-bearing.

---

## 7. Re-scoring procedure (mechanical, public, computed after this document freezes)

7.1 **Inputs — frozen only.** The frozen per-program JSONs in `docs/measurements/baselines/` and
`docs/measurements/ports/`, and, where the direct valve-statement unit (§4.1) is used, a **re-run of
the frozen counting scripts on the frozen artifacts at their recorded commits** (baselines `b689860`;
ports at their recorded port commits). **No new ports, no baseline changes, no re-porting, no new
programs.**

7.2 **Field mapping (mechanical).** Per program, from the JSONs:

    V_candor / V_rust (exact unit):        valve_statements        (from <prog>.v2.json)
    V2 valve-statement fraction (§4.3):    valve_statements / logical_statements
    baseline V_rust > 0 ?                  from baselines/<prog>.v2.json valve_statements
    VSE cross-check (reported only, §4.1): valve.lines × logical_statements / valve.total_lines

  The operative unit (§4.1) is the `valve_statements` field the extended counters emit
  (`unit_ext_version` "2") over the frozen artifacts; §7.1's re-run reproduces the `.v2.json`
  measurements. VSE is computed only for the reported Appendix-A cross-check.

7.3 **Procedure.** (1) Adjudicate any stdlib-substitutable tags (§4.5) and any allocator-class
classifications beyond the allocator (§5.3), each under §6.5 open-comment, in `docs/ADJUDICATIONS.md`.
(2) Compute `V_candor`, `V_rust`, `R_valve` per program (or V2 for `V_rust = 0`) on the full count.
(3) Apply carve-out (§5) and reversal (§4.5) paths. (4) Combine with carried-over M1/M1b/M4/M5 per §6.
(5) Publish the full computation — every intermediate value, the unit used, per-program verdicts — as
the act that freezes this document (§1.1).

7.4 **This document stops short of executing §7.3.** The numbers are produced by the public re-scoring,
not asserted here (§0.3). No overall verdict is stated in this registration.

---

## 8. Publication, ledger, and §9 obligations

8.1 **Published either way** (philosophy §3, §7; v1 §7): the re-scoring output, the unit used, all
adjudications, and the §0.5 ledger are published whether the result is KILL, mandatory review, or
provisional confirmation.

8.2 **Verdict enacted as a §9 amendment** (philosophy §9; v1 §7.2):
- **KILL:** amendment naming Bet 5 and P12, stating the evidence, appended to the philosophy's
  Appendix A ledger.
- **PROVISIONAL CONFIRMATION:** recorded as the verdict that unblocks the syntax freeze (philosophy
  §3, §8.5), scope preserved.
- **MANDATORY REVIEW:** the authority's recorded §0.5 ruling is the enacted outcome.

8.3 **Binding scope (from §0.2d).** These rules govern any future basket extension or re-port under
the same claim, unmodified. NN#14's stability gate stays closed until a re-scoring under this
registration produces a non-KILL verdict.

8.4 **Independence residual (v1 §6.7, plus this document's own conflict).** Full third-party
independence is unavailable to a solo project and not claimed. Mitigations: v1's (session-blind
carried artifacts, blind classification order, §6.5 open-comment, public record) plus this document's
data-aware preamble (§0) and its dedicated reverse-engineering review (§0.2c). These reduce but do not
eliminate the §0.1 conflict; naming the residual is itself required (philosophy header).

---

## 9. Explicitly rejected alternatives (beyond §4.6)

9.1 **Re-porting or re-baselining** — *rejected*: artifacts are frozen and public; any artifact
authored after the verdict is known is unauditable. The correction is to the *ruler*, on *frozen
objects* (§7.1). **9.2 Editing v1 in place** — *rejected*: v1 is frozen and never modified (philosophy
§3); this succeeds it, the dead registration stays readable. **9.3 No zero-baseline rule** —
*rejected*: parser and arena (`V_rust = 0`) need a defined floor; §4.3 carries one from v1 rather than
inventing one under data-aware conditions. **9.4 Auto-carving all pointer-rich programs** — *rejected*,
§5.3: risks an un-failable criterion and over-reads a concession made for the allocator specifically.

---

## Appendix A (normative) — disclosed outcome tables on the exact unit (review finding 2)

Review finding 2 requires the rejected-alternative and reference numbers be **disclosed, not hidden**;
finding 1(b) requires them on the **exact valve-statement unit** now that the counters emit it
(§4.1). These tables are that disclosure. They are computed from the frozen `.v2.json`
re-measurements (`docs/measurements/{ports,baselines}/<prog>.v2.json`) produced by the extended
counters (`unit_ext_version` "2") over the frozen artifacts. **They state the per-program valve-metric
outcomes the §7 re-scoring will confirm; they deliberately do NOT state the combined overall verdict**
(the valve metric composed with M1/M1b/M4/M5, the WARN-count, and the carve-out / mandatory-review
routing), which remains the post-freeze re-scoring act (§0.3, §7.4). The formal re-scoring happens
after ratification.

**A.1 — Operative successor metric (exact unit; `V = valve_statements`).**

| Program | V_candor | V_rust | R_valve = V_c / V_r | Metric | Verdict (V1/V2) | Routing |
|---|---|---|---|---|---|---|
| Allocator | 96 | 86 | **1.116** | V1 | **WARN** (>1.00, ≤1.25) | home-ground WARN → mandatory §6.4 review; also allocator-class carve-out → no auto-KILL (§5) |
| Scheduler | 47 | 89 | 0.528 | V1 | pass | gates normally (§5.3); no WARN |
| MMIO | 3 | 6 | 0.500 | V1 | pass | — |
| Parser | 9 | 0 | `V_rust = 0` → V2 | V2 | pass (`9/311 = 0.0289` ≤ 0.08) | — |
| Arena | 0 | 0 | `V_rust = 0` → V2 | V2 | pass (`0/111 = 0.0000`) | — |

On the exact unit **no program triggers an automatic valve KILL**; the sole valve WARN (allocator,
home-ground) routes to a mandatory §6.4 review. This is a materially different outcome from v1's three
valve KILLs (A.3) and is the direct effect of correcting defect (i): the allocator carries **1.116×**
— not v1's 3.26× line-fraction — the idiomatic-Rust valve *content* once density is removed.

**A.2 — Rejected raw-absolute-valve-lines alternative (no density normalization; §4.6).** Same 1.25
KILL / 1.00 WARN margins applied to raw valve *lines*; zero-baseline programs fall to v1's absolute
line-fraction floor.

| Program | lines_candor | lines_rust | ratio | Verdict |
|---|---|---|---|---|
| Allocator | 112 | 179 | 0.626 | pass |
| Scheduler | 89 | 159 | 0.560 | pass |
| MMIO | 6 | 11 | 0.545 | pass |
| Parser | 15 | 0 | line-frac `0.0408` | pass |
| Arena | 0 | 0 | line-frac `0.0000` | pass |

The rejected raw-lines alternative **passes every program** — it would wave through even the allocator
(0.626) that the operative exact unit flags (1.116). Choosing raw lines would have been
reverse-engineering *toward* the bet; the exact unit was chosen instead, and it is **harsher on the
hardest program**. This is the disclosure finding 2 says exonerates the design's honesty: the
drafter's unit cuts against the bet exactly where a rejected alternative would not.

**A.3 — v1 verdict, for reference (frozen line/function-fraction ruler; `RESULTS.md`).**

| Program | v1 metric | v1 value | v1 verdict |
|---|---|---|---|
| Allocator | valve-line fraction | 0.6292 > 0.40 | KILL |
| Scheduler | valve-line fraction | 0.4120 > 0.40 | KILL |
| MMIO | valve-function fraction | 0.2500 > 0.20 | KILL |
| Parser | valve-line fraction | 0.0408 | pass |
| Arena | valve-line fraction | 0.0000 | pass |

**A.4 — VSE cross-check (reported-only; §4.1).** `VSE = valve_lines × logical_statements / total_lines`.

| Program | VSE_candor | VSE_rust | VSE ratio | vs exact R_valve (A.1) |
|---|---|---|---|---|
| Allocator | 101.93 | 57.67 | 1.767 | 1.116 |
| Scheduler | 71.28 | 39.45 | **1.807** | 0.528 |
| MMIO | 4.36 | 4.02 | 1.084 | 0.500 |

VSE's density assumption fails wherever the baseline's valve is concentrated in a dense region: it
over-states the scheduler at **1.807** (a spurious KILL) where the exact intersection is **0.528**,
because Rust's scheduler valve lives in a dense vendored intrusive list rather than spread at
whole-program density. This is precisely why finding 1(b) required the exact counter and demoted VSE
(§4.1). The pre-ratification review transcript's figure of "R_valve 1.807 (scheduler re-KILL)" was
that VSE estimate; it does **not** survive the exact count and is **superseded** by A.1. What survives
the correction, and is the honest disclosure finding 2 demands, is that the operative exact unit is
harsher than every rejected alternative on the hardest program (allocator WARN vs raw-lines pass)
while refusing to manufacture a KILL the frozen artifacts do not support.
