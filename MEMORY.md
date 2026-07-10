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

- **Impl/interface conformance and the reassign double-drop were both real; fixed
  2026-07-08.** (1) Interface method-set matching (E1014/E1015) existed but SIGNATURE
  conformance did not — an impl could diverge on self mode, param count/mode/type,
  return, or effect marker and be accepted. Now checked in `resolve_impl`
  (`check_impl_conformance`, E1021-E1026) by substituting Self->target, iface
  params->iface args, Self::Assoc->the impl binding, then comparing UNDER-LOWERED
  param types + modes and borrow-wrapped returns. Interface/impl methods cannot
  declare region variables in the grammar, so region conformance is subsumed by
  borrow-kind matching (no independent region-divergence test is writable). Extra
  impl methods stay REJECTED (E1014) — one interface, one shape. (2) The assign path
  dropped the old value BEFORE evaluating the RHS, so `lst = push(a, lst)` freed the
  old chain then re-embedded/re-freed it — double free, masked by the no-op bump
  free. Fix: evaluate RHS first, then drop-old only if `place_owned` (which already
  consults the move mask) holds, using the local's LIVE mask not `MoveMask::default()`.
  A counting allocator (trace +5555/-5555) unmasks it in tests.

- **Match the oracle's fault SPAN by simulating its `cur_span` threading at
  lowering time, then bake the exact `(k,s)` into each MIR fault edge.** The
  tree-walker's fault span is whatever `self.cur_span` last held: arithmetic
  resets it to the *whole-binary-op* span before the check (overflow/div0 =
  op span); conv/neg/`assert`/`requires`/bounds deliver at the *operand's
  trailing* span (conv-loss = operand span, `assert(a<b)` = `b`'s span, bounds =
  the base-array span). The MIR lowerer threads its own `cur_span` identically and
  stamps the edge — no runtime span-threading needed, and the gate calibrates it.
  Verified empirically before writing (probe the oracle's fault JSON), not by
  reading alone (Stage A MIR, 2026-07-08).

- **A value model (typed i128) beats reusing flat `interp::mem::Mem` for a
  scalar-core alternate engine — the Stage-A gate tests `(k,s,θ)`, not bytes.**
  0010 says "reuse the memory module is preferable", but that's load-bearing only
  once rawptr/MMIO/Box/aggregate layout enter; for the scalar/control/contract
  core a value model reproduces θ exactly at a fraction of the risk. Report the
  substrate decision explicitly; reserve Mem for the aggregate extension.

- **Lower AST→MIR per-construct in the oracle's evaluation order and let
  out-of-subset constructs return `Unsupported`; point the gate at the WHOLE
  corpus and it self-reports coverage.** Treating `Unsupported` as
  "out-of-subset, not a failure" turns the differential harness into an honest,
  automatic coverage meter (found 2 real fixtures — `bits`, `mono3` — in-subset
  for free, incl. the generic→monomorphize→scalar path).

- **Cranelift `opt_level=speed` turns a saturating-clamp `select` on i128 into
  `smax.i128`/`smin.i128`, which its x86 backend has NO ISLE lowering for.** Only
  the OPTIMIZER (egraph) exposes it — at `opt_level=none` the `select` lowers fine;
  the failure surfaces as a *compile error*, not a trace divergence. Fix: keep the
  min/max operands at i64 width (compare in i128, but `select` the already-fitted
  i64 value) so no i128 min/max pattern forms. When flipping the native optimizer
  on, run the FULL corpus under opt — codegen gaps hide behind `opt_level=none`
  (Stage D, 2026-07-09).

- **Marking rawptr/MMIO observables and lowering them as runtime-hook CALLS is the
  honest F1 barrier on Cranelift — it has no volatile MemFlag.** A call with a side
  effect is a barrier the egraph will not reorder past, coalesce, or DCE, so
  INV-OBS-ORDER/INV-FAULT-ID hold by lowering discipline at `opt_level=speed`. In
  the MIR, `let` bindings lower to `Store` into named locals (not dead `Assign`
  temps), so an R1 dead-local pass must treat a BARE-ROOT `Store` as a def to fire
  at all; projected stores (`.f`/`[i]`/`deref`) keep their root live. Fault-capable
  rvalues (checked arith, `Div`/`Rem`/shift, index, checked conv) are NEVER
  DCE-eligible — their fault is a potential observable (Stage D, 2026-07-09).

- **Freestanding (no-libc) linking needs the flat buffer at a fixed VA without
  bloating the binary — a NOBITS section + `--section-start` does it.** The
  Cranelift object bakes `MEM_BASE` (0x200000000000) as an absolute `movabs`
  constant for every load/store, so the flat region must live at that exact VA.
  A normal `.bss` array lands near 0x4xxxxx (wrong), and forcing a named section
  with a zero-init array can turn PROGBITS (256 MiB on disk). The fix: declare the
  region in inline asm as `.section ...,"aw",@nobits` + `.skip`, address it ONLY
  through the absolute `MEM_BASE` constant (a `-no-pie` PC-relative ref to a 32 TiB
  VA does not fit — R_X86_64_PC32 truncation), and pin it with
  `-Wl,--section-start=.candor_flat=0x200000000000`. No mmap, no linker-script
  file, ~9 KB binary, kernel zero-fills the LOAD segment. The freestanding AOT
  path reuses the SAME emitted object as the hosted one — only the runtime C file
  and link flags differ (`src/backend/freestanding_runtime.c`, object.rs
  `link_freestanding`).

- **The design pipeline's first outright rejection came at design twelve
  (concurrency), and the rework held.** The (c)-verdict pattern: the reviewer
  constructed a race from the draft's own flagship example (Alloc copy-out
  through a shared borrow - copy types hiding rawptr are the laundering
  channel). The fix that held: gate borrow REFERENTS transitively including
  through copy fields, with fn pointers as a portable leaf. Concurrency
  reviews must always attack copy-out-from-behind-borrows first.

- **A compiler-known std collection has a fixed 9-site integration checklist.**
  Adding `Map` (mirroring `Vec`/`String`) touched exactly: `types.rs`
  (`needs_drop`, `bears_box`, `box_subpaths` — the three that drive alloc-on-drop
  / E0401; `is_copy`/`is_portable` need NO change, their `App`-no-registered-generic
  default already yields non-copy/non-portable), `interp/layout.rs`
  (`size_of`+`align_of`), `interp/eval.rs` (`resolve_ty` keeps it an `App`, the
  `drop_value` arm, the shared `len` builtin, the builtin dispatch + op fns),
  `generics.rs` (`rewrite_ty`+`type_to_ast_kind` keep it an `App` through
  monomorphization), and `check/expr.rs` (`arg0_is_*` router + `check_builtin`
  arms). The resolver needs nothing — unknown call names are treated as builtins
  resolved by the checker (`ast.rs` comment). Overloaded op names (`get`) are
  disambiguated by mutually-exclusive `arg0_is_X` guards, so arm order between
  collections does not matter. Grep `"Vec"` across `src` to find every site.

- **`read`/`write` param mode on a borrow-kind type (`str`, `[u8]`, any view) is
  E0203 — pass views by value.** A `fn f(word: read str)` is ill-formed; `str` is
  already a view, so it is `fn f(word: str)`. Bit the Map keyword-classify
  demonstrator (2026-07-10). Owned collections take `read`/`write` (they are not
  borrow-kind); their contained views do not.

- **Compiler-known std types must be identified STRUCTURALLY, never by name —
  and the same predicate must hold in BOTH the interpreter and the checker.**
  The allocator handle `Alloc`/`AllocVtable` are detected by shape (vtable =
  struct with `alloc`+`free` fn-ptr fields; handle = struct whose `vt` is a
  `rawptr` to it), because the module loader qualifies a user `struct Alloc` to
  `analyses::Alloc` and any literal `n == "Alloc"` test then fails. finding-F1
  fixed this in `interp/eval.rs` but left the identical name test live in
  `check/expr.rs` (the `vec_new`/`string_new`/`map_new` arms), so a module tree
  defining its own `Alloc` was rejected with a spurious E0703 before it ever ran
  (OBL-SELFHOST-MOD-F1, 2026-07-10). Lesson: when you special-case a
  compiler-known type by name in ONE stage, grep the other stages for the same
  name test — the checker and interpreter must agree on the structural identity,
  or module qualification desyncs them.

- **A `write`/`read` borrow PARAMETER is a reference — using it (passing bare,
  field-read) REBORROWS, it never MOVES. A move analysis must not conflate the
  loan-analysis "non-copy exclusive access" flag with "moves on use."** The
  self-host move/init analysis classified a `write C` param as non-copy
  (`loc_copy == 0`) — right for the loan sense (E0804 exclusive access) but it
  also drove `place_use` to mark the param MOVED, so the next bare pass/field-read
  was a false E0301 use-after-move (1× on lexer.cnr, 22× on checker.cnr; the Rust
  oracle emitted ∅). Fix: a separate `loc_ref` flag (read/write params +
  borrow-typed bindings) gates the move — only an OWNED non-copy value moves, a
  reference reborrows. Lesson: "can this place be moved" (ownership) and "does
  this place hold an exclusive borrow" (loan kind) are DIFFERENT properties; a
  borrow reference is non-copy for aliasing yet un-movable — track them apart, or
  every reborrowed `write` param reads as a use-after-move (2026-07-10).

- **Immutable literal storage must be interned, or a literal-heavy program's
  static region grows into the stack and corrupts live data.** The interpreter's
  `str_literal` (`src/interp/eval.rs`) `static_alloc`ed fresh bytes on EVERY
  evaluation of a `b"..."`/`"..."` literal, never dedup/reclaim. Checking the
  literal-heavy `analyses.cnr` did ~18845 literal evals; static grew from 0x1000
  past `STACK_BASE` 0x100000 and overwrote `main`'s embedded `src: [u8]` — reads
  of `src[9173]` flipped 110→115→116 mid-run (the "byte-22600 boundary" red
  herring: a symptom, not a checker gap). Fix: content-addressed interning
  (`literal_cache: HashMap<Vec<u8>,u64>`), sound because literals are immutable
  (observationally invariant — four-engine equivalence stayed green). Lesson: a
  "checker can't resolve past offset N" symptom on a large embedded-source gate is
  often a runtime memory-model leak, not a front-end bug — check static/stack
  collision before extending the checker (2026-07-10).
