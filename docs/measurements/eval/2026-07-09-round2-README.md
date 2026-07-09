# Second P19 competence measurement (graduation tier) — 2026-07-09

Same Sonnet-class model discipline, spec pack only, all 23 tasks (8 seed + 15 graduation:
compositions, basket-derived sub-programs, hard multi-error repairs, one adversarial-semantics
task). **Result: 23/23 first attempt; no round-2 needed; slope delta 0.**

Reading — double-edged, recorded honestly:
- **For Bet 6:** strong early support. A 630-line hand-distilled spec pack manufactured enough
  competence for a Sonnet-class model to write, blind and first-try, intrusive rawptr structures
  with container_of, a self-contained allocator vtable, cross-type ? via From, and multi-error
  repairs — and to pass the adversarial task (bare + faults at 255; only wrapping awareness
  passes), i.e. rule internalization, not just syntax. The P4/P13 legibility investment appears
  to transfer.
- **Measurement caveat:** the graduation tier, authored within the same broad model family as the
  model-under-test, may systematically underestimate difficulty for that family. A ceiling result
  here does NOT establish the tasks are hard in general. The needed controls: (1) a cross-family
  model-under-test; (2) tasks authored by an independent party; (3) harder compositions (whole
  basket programs, not sub-problems). Until then this is a positive floor, not a slope.
- One near-miss (not a failure): `out`-parameter deref was resolvable only by inference; the pack
  now states it directly (semantics.md).

The harness and tasks are sound (reference-verified, controls fail as designed); the limit is task
difficulty relative to strong models, which is a task-library problem, tracked for scale-up.
