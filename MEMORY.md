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

- **Every checker soundness hole found so far lived at a SEAM, never inside one
  analysis.** S1: pattern-bindings × call-loans; drop flags: init-analysis ×
  interpreter drop points; nested hooks: partial-move × projection depth;
  free-effect: effects × interpreter drop sites; E0310: move classification ×
  opaque places (checker/interpreter divergence). When reviewing or extending
  the checker, attack interactions between analyses and checker-vs-interpreter
  agreement first; single-analysis internals have held up.

- **§11's own example code leans on an implicit reborrow the model forbids.**
  In 11.4 `peek(s, pos)`/`advance(pos)` pass a bare exclusive-borrow param
  (`pos: write usize`), which the memory model says *moves* it (reborrow needs
  the explicit `write (deref pos)`). So a use-after-move follows on the next
  cursor use. The Stage-2 checkable fixtures adapt those two call sites to
  `read (deref pos)`/`write (deref pos)` (commented ADAPTED). If Stage 3/4 ever
  auto-reborrows at call sites, revisit these adaptations.

- **Simulating volatile MMIO needs a seam on the ACCESS, not the value; fn-pointer
  hooks provide it cleanly.** The mmio port keeps reg_read/reg_write as real
  one-ptr-op valves over a live register window and attaches the device model via
  fn-pointer fields in the handle (on_write after the store, on_read BEFORE the
  load, depositing the driven value). This keeps the measured section free of
  simulation code and standalone-checkable (R14). §6.1's vtable machinery
  (fn-pointer fields, enums-in-structs through ptr_read/ptr_write, loop/break)
  all worked first try — verify capabilities with tiny scratch programs before
  committing to an architecture.

- **Consuming matches over Box-bearing enums must return from every arm
  (E0302).** A visitor-style `match` that moves payloads out of an owned
  enum cannot fall through the match join — arms that move and arms that
  don't disagree on partial-move state (§1.6 rule 1) even in a unit function
  with nothing used afterwards. Put `return;` at the end of every arm (the
  §11.4 fixture's all-arms-return shape is load-bearing, not style). Found
  in the parser port's serializer/span-walker, 2026-07-07.

- **The region-based valve metric counts the `unsafe` block statement itself
  plus each inner statement.** A one-return unsafe block = 2 valve statements,
  so deleting t_link's block in the scheduler re-port moved the count 47->45
  ("≈1" in the a-priori ruling was block-granularity; both removed statements
  attribute to t_link). Verified with a minimal fixture via
  `candor-proto count`, 2026-07-08.

- **The counter counts an unsafe block's statement AND its inner statements.**
  Eliminating a one-statement unsafe block removes 2 valve statements, not 1
  (block + inner). State a-priori predictions in the metric's exact unit, or
  reconcile granularities explicitly when recording outcomes (scheduler re-port,
  2026-07-08: predicted ~1 at block granularity, measured 2 at statement
  granularity).

- **The migrate-by-AST-reemission pattern is the P15 workhorse.** The
  throwaway-to-real migrator parses with the old front-end and pretty-prints
  the shared AST in canonical new syntax - semantic fidelity by construction,
  validated by a parity harness (identical diagnostics + run sentinels).
  Author-assisted rows get // MIGRATE: markers, never silent transforms.
  Reuse this shape for every future edition migrator.

- **New-construct soundness gaps cluster at exit points.** The ? operator's
  stage-1 gap (unmodeled early return) was the same class as E0309's history:
  any construct that exits a function must reach the CFG as a genuine Return
  so drop checks, ensures re-emission, and move state fire. Check exit-point
  modeling FIRST when adding control flow.
