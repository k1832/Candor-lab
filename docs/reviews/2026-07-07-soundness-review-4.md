# Independent retest #4 of design 0003 — PASSED

**Date:** 2026-07-07
**Reviewer:** fourth fresh independent session. **Verdict: (a) ACCEPTABLE for freeze step (i)
as-is.** No accept-invalid found; no asserted-not-argued claim; no text amendment required.

What was verified:
- All retest-#3 fixes hold at every boundary attacked (25 new adversarial programs): per-return
  `ensures` analysis correctly rejects if the param is moved on *any* return path and accepts
  valid multi-return shapes; the alloc effect propagates out of contract clauses (a non-alloc fn
  with an allocating `requires` is E0401 — the correct reading of 0001 §3.2); E0708 is tight at
  direct, call, indirect-call, and static-read boundaries; E0311 covers all mutation forms;
  no loan hazard is reachable from `ensures` position (body borrows are NLL-dead at returns);
  pass-2 diagnostic truncation hides no real errors; regime blocks cannot syntactically occur in
  contract conditions.
- All prior repro sets (reviews #1–#3) re-confirmed rejecting; full suite green (211/0).
- 0003's §0 history is complete and faithful; §2.7's mechanism claims all check against code;
  the suite-green invariant replaced the rotting tally.

Recorded observations (non-blocking):
- Returning a `read`-borrow of a static is rejected E0806 with a cosmetically wrong "local"
  message — sound reject-valid (the prototype has no 'static region); noted so future reviewers
  need not re-derive it.
- Finding 2's confirmation that contract clauses propagate the alloc effect is now load-bearing
  precedent: contracts are ordinary expressions to every analysis.

## Consequence: freeze step (i) is COMPLETE

Per BET5_CRITERION.md §3.7 and §6.2 step (i), all required artifacts now exist, are adversarially
reviewed, and are hashed in [docs/FREEZE_MANIFEST.md](../FREEZE_MANIFEST.md): the criterion (v2),
the unit table + both counting tools, the checker-soundness argument (reviewed, retested to a
pass on the fourth round), and the prototype whose syntax is now frozen per §3.1.

Review-loop tally for the soundness artifact: 4 independent sessions, 12 findings, 3 failed
retests, 9 new error codes/rules across the loop (E0309, E0310, E0311, E0708, E0806-totality,
E0303-prefix-walk, alloc-free-side, reassignment/out drop points, read-only contracts) — the
philosophy's "a repair that fails its retest teaches more than one that passes" borne out three
times over.
