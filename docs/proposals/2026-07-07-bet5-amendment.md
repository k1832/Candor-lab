# Proposed §9 amendment: enactment of the Bet 5 verdict

**Status: PROPOSAL — not enacted.** Enactment is the deciding authority's act. Since Bet 5 and
P12 are tied to Non-Negotiable #14's stability gate, the philosophy's amendment rules apply in
full: this proposal is published for open comment before enactment, and objections are recorded
alongside the decision.

## What must be amended (not optional)

The pre-registered criterion computed **KILL** (docs/RESULTS.md). Philosophy §9: missed
pre-registered targets are enacted as amendments, not absorbed as silence. Appendix A gains a
ledger row either way.

## The evidence, in one paragraph

The bet's cognitive-load claim **won decisively**: 53% aggregate annotation reduction vs
idiomatic Rust across all five programs (range 29-79%), zero copy inflation, all five programs
completable, and the value-favorable programs landed exactly where the design predicted (parser
and arena near-zero valve). The bet's valve-rarity claim **failed as measured**: three of five
programs breached frozen valve-fraction ceilings. Two defect observations are published with the
verdict (§0.3): the fraction metrics punish Candor's 2-5x compactness (identical absolute valve
content in a smaller program yields a proportionally higher fraction; MMIO's breach is the
design's own prescribed two-function valve seam inside an eight-function program), and the
allocator's 0.63 valve-line fraction is the one breach likely to survive any denominator
correction.

## Options before the amender

**Option A — enact KILL; retire P12's value-first model.** The literal §7.2 path. Not
recommended: it discards a decisively confirmed load reduction on the strength of a metric the
record shows to be partly artifactual.

**Option B (recommended) — enact KILL as registered; re-register a corrected criterion; re-score
the frozen artifacts.** The v1 registration is dead and is never retrofitted. A successor
criterion is pre-registered openly BEFORE re-scoring, changing only the defective operationalization
(candidate: absolute valve content vs the spec's irreducible pointer work, or fractions normalized
by the Rust baseline's statement density), with its own kill thresholds. The existing ports and
baselines are already frozen and public, so re-scoring is mechanical and cannot be gamed by
re-porting. P12 survives only if the corrected registration passes; NN#14's stability gate stays
closed until then. The amendment also records the confirmed positive finding (the load claim) so
the philosophy's account of Bet 5 stays honest in both directions.

**Option C — Option B plus a P12 concession on allocator-class code.** The allocator breach
(0.63) likely survives correction; the amendment adds to P12 a named limit: value-first does not
carry allocator-class code (in-band metadata over raw memory), where the valve IS the program's
core. The design consequence worth recording with it: the ports' top friction — rawptr's lack of
safe field projection — inflates valve surface in exactly those programs, and a future design
round may shrink it (e.g., typed field access on rawptr as a narrower unsafe operation) without
touching the model's soundness.

## Recommendation

**C** — it is B plus honesty about the one result that isn't artifact. The friction record
(reborrow ceremony, rawptr field projection, no-tuples, E0304 false positives) transfers to the
design backlog regardless of the option chosen.
