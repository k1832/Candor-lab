# Project memory

Durable lessons from past working sessions on Candor. Read before starting
non-trivial work; append a lesson when something worth remembering surfaces.
One lesson per entry, one-line summary first.

## Lessons

- **The philosophy dictates sequencing — check §8 before planning.** The
  critical path (Bet 5 prototype → semantic core/spec skeleton → minimal
  toolchain → breadth) is normative, not advisory. No stability commitment
  before Bet 5's verdict (NN#14).

- **Spend LLM effort where decisions are irreversible.** Owner's stated
  preference (2026-07-06): high effort on the Bet 5 kill criterion,
  memory-model design, and adversarial reviews; medium/light on scaffolding
  and mechanical work. Adversarial review of every major design doc is the
  established quality pattern (the philosophy itself went v1→v4 that way).

- **The document pipeline is: draft → adversarial review → dispositions →
  revision, each step a separate commit.** Review findings live in
  `docs/reviews/` with the deciding authority's disposition recorded inline
  per finding; the revision agent implements dispositions exactly (they are
  decisions, not suggestions). This caught 3 blockers in the kill criterion
  and 2 soundness holes in the memory model on 2026-07-06 — keep it.

- **Agents drafting in parallel can bake in stale facts.** The criterion
  draft claimed GOVERNANCE.md didn't exist because it was written in
  parallel with the scaffolding commit. Check parallel-drafted docs for
  references to repo state and fix before review.
