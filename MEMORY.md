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

- **Module qualification breaks every compiler-known-name lookup by string.**
  The stdlib seed surfaced four bugs whose shared root was the module tree
  qualifying names (`Alloc` -> `std::alloc::Alloc`, `From` -> `core::res::From`)
  while the interpreter/checker still matched bare strings: box/unbox field
  offsets (F1), and the `?` From-impl/interface lookup (F2). Fix by identifying
  compiler-known types STRUCTURALLY (Alloc = the struct whose `vt` field points
  at the {alloc,free} fn-ptr vtable) or by BASE NAME (`rsplit("::").next()`),
  never by a hardcoded qualified string. When adding any lang-item lookup, ask
  first how it survives qualification. (2026-07-08)

- **Niladic generic constructs need expected-type inference at three sites.**
  A value giving no type evidence — `nil()`, `Node::Nil`, `List::nil()` — can
  only pin its type parameter from the EXPECTED type. The checker already had
  the `expected_ty` hint plumbed through `check_against`, but two paths dropped
  it: a generic struct literal resolved its own args from the hint yet never
  folded them back into the substitution map before substituting FIELD expected
  types (F3), and a generic CALL only unified from value args, never from the
  return type against `expected_ty` (F4). When a construct can appear with zero
  value-argument evidence, wire the expected type into its inference and fold
  resolved args back before any nested substitution. (2026-07-08)

- **The init-analysis fixpoint must iterate in reverse-postorder, not block
  order.** A `loop { match { arm => if c { break } } other => break }` (the
  `for`-desugar shape) made the definite-assignment fixpoint OSCILLATE between
  `Init` and `MaybeInit` and never converge: a back-edge continuation block whose
  only predecessor is a HIGHER-numbered block seeded itself from `entry`
  (bottom/Uninit) on pass 1, poisoning the loop header. Iterating the fixpoint in
  RPO (back-edges are the only backward edges) fixed it (init.rs, 2026-07-08).
  Any new control-flow construct that adds back-edges — test convergence, not just
  correctness of the transfer functions.

- **Reassigning a variable that was moved into the RHS call double-drops it.**
  `lst = cons(a, v, lst)` (RHS consumes `lst`, then rebinds it) drops the OLD
  `lst` at the reassignment even though it was already moved into `cons` — a
  latent interpreter double-drop, masked everywhere because bump-`free` is a
  no-op and no drop-hooked value had been reassigned-through-a-move. A `List` of
  drop-hooked items built with `l = f(.., l)` traces each element ~n times.
  Build with DISTINCT bindings (`l1 = f(.., l0)`) to avoid it; the bug itself is
  a pre-existing move-tracking gap in the interpreter's assignment drop, not the
  `for`-desugar (2026-07-08).
