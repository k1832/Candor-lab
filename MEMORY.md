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

- **A place-walker that can't root an expression must DESCEND to the expression's
  non-place base — never re-dispatch the same node to the general expr checker,
  or you get infinite mutual recursion.** The self-host `borrow_place`, on a field
  access whose base isn't rooted at a local (`f().field`, base is a call), had
  `root_of` return 0 and fell through to `chk_expr` on the SAME T_FIELD node →
  `chk_expr`↔`borrow_place` cycled forever (frame depth past 60000, overflowed even
  a 1 GiB stack). Surfaced only by parser.cnr (the largest self-host module) — its
  size wasn't the cause, its `f().field` shape was. Fix (analyses.cnr:439): the
  not-rooted branch descends the place chain to its non-place base and walks THAT.
  Lesson: place recursion terminates on the base expression, so peel to the base;
  re-handing the whole node back is a non-decreasing recursive call (2026-07-10).

- **Detecting integer overflow without a wider type: run the op in `wrapping {}`
  then decide overflow separately.** The self-host scalar interpreter must match
  the Rust oracle's overflow faults, but the self-host language has no `i128` to
  widen into (the oracle uses i128, `eval.rs` ty_range/fit). Technique: compute the
  raw result in a `wrapping { }` block (which cannot fault), then test overflow by
  arithmetic identity — signed add/sub by operand-sign logic, signed mul by
  `wrapped/a != b` (special-case MIN*-1), unsigned add by carry (`wrapped < a`),
  unsigned sub by borrow (`a < b`), unsigned mul by division re-check; narrow types
  (i8..i32/u8..u32) compute in the i64 base and range-check against the type
  min/max. u64↔i64 reinterpretation via `wrapping { conv }`. Verified byte-exact vs
  the oracle. This is the standard trick for any Candor numeric code that can't
  reach for a wider type (self-interp S1, 2026-07-10). See [[self-host-analyses]].

- **Building a size/align/offset layout table from the self-host parser's arena
  type-nodes: scalar keywords are `T_SC`, not `T_NAMED`.** For the S2 self-interp
  (structs+arrays over flat byte memory) I needed `ty_size`/`ty_align`/field-offset
  driven by arena type-nodes. The trap: `parse_type` tags a SCALAR keyword (`i64`,
  `bool`, …) as `T_SC`, and `T_NAMED` is only user structs/enums; an array is
  `T_ARRAY` (`.a` length, `.b` element) and a struct VALUE can carry its `T_STRUCT`
  decl node directly. Gating scalar detection on `T_NAMED` alone made every scalar
  local resolve as a zero-width "aggregate" and read back its ADDRESS instead of its
  value (arith fixture returned 143 vs 121). Fix: treat `T_SC`+`T_NAMED` uniformly —
  `sc_width(p0,p1)` on the span decides scalar-vs-struct. Layout mirrors
  src/interp/layout.rs: fields in DECLARED order at natural alignment, struct size
  rounded to max-field alignment, array stride = round_up(size,align). Because
  addresses never surface in the RET/TRACE/FAULT dump, the byte arena's base/size are
  free choices — reset the bump per block AND per call to bound loops/recursion
  (self-interp S2, 2026-07-10). See [[self-host-analyses]].

- **Self-interp enums: a scrutinee's type reaches `match` in TWO shapes, enum
  drop is tag-directed, and by-value match arms must not diverge in move state.**
  For S4 (enums+match) three points recur in every later enum slice (S5 BoxResult
  reuses all of it): (1) `enum_of_ty` must accept BOTH a `T_NAMED` local/param type
  AND a bare `T_ENUM` decl node — a constructor result carries the decl node
  directly in `cur_ty`, a local carries the named type; route both or a `match` on
  a freshly-constructed enum silently mis-resolves. Enum layout mirrors
  src/interp/layout.rs: `{tag:u64@0, payload@8}`, payload laid out struct-style from
  offset 8, size `round_up(8+max_payload,8)`, align 8 always, tag = 0-based variant
  index. (2) Enums carry NO drop hook, so enum drop is the WHOLE story of reading
  tag@0, selecting the active variant, and dropping its payload fields in reverse
  honoring S3's one-level move-mask (`drop_variant_rev`, dual of `drop_fields_rev`) —
  S3's `needs_drop`/`drop_value` knew nothing of enums and must be extended. A bound
  non-copy payload is aliased in place and its field marked moved on the scrutinee's
  DIRECT-LOCAL root (`cur_org==2`); copy payloads are byte-copied and are fully
  S3-independent. (3) FIXTURE-AUTHORING TRAP: the real checker rejects a by-value
  `match` where one arm binds+moves a payload and a sibling arm does not — E0302
  "inconsistent move state at join". Make every arm move consistently (all bind a
  payload, or use a single wildcard arm) or the fixture never reaches the interp.
  (self-interp S4, 2026-07-10). See [[self-host-analyses]].

- **Self-interp S5a (allocator-ABI foundation): a fn-ptr value is the fn's ARENA
  NODE ID, rawptr ops need `unsafe`, and rawptr/fnptr must be 8-byte scalars
  everywhere including `fn_ret_width`.** Porting the box prerequisites: (1) The
  self-interp has no fn-table, so fn-name-as-value yields the fn's parser node id
  and an indirect call is just `call_fn(pp,src,e,fnid)` — this differs numerically
  from the oracle's `fn_id_of` index, but a fn-ptr value is NEVER surfaced as a
  RET/TRACE observable, so dumps stay byte-exact. Likewise absolute addresses
  differ (16384-byte SMALL arena vs the oracle's 256 MiB space) yet only VALUES
  are observable — surface a value written/read THROUGH a pointer, never an address
  (no `ptr_to_addr` in-subset), and both engines agree. (2) Classify `T_RAWPTR`(17)
  and `T_FNPTR`(22) as an 8-byte usize scalar in `scalar_width` AND `ty_size`/
  `ty_align` AND `fn_ret_width` — the last is easy to miss: a `-> rawptr u8` fn
  whose return width stays 0 delivers its result on the aggregate channel (`cur_w
  == 0`) and the caller mis-reads it. To let `ptr_read` recover a pointee type,
  make a scalar load carry the place/local type in `cur_ty` (harmless — plain
  scalars ignore it) and read the pointee from `.a` of any rawptr-shaped node
  (`rawptr_inner` over T_RAWPTR/T_CASTPTR/T_ADDRTOPTR/T_PTRNULL). (3) FIXTURE TRAP:
  raw-pointer ops (addr_of/ptr_read/ptr_write/cast_ptr/addr_to_ptr/ptr_null; NOT
  is_null) are E0501 outside an `unsafe "why" {}` block, so fixtures need real
  unsafe blocks and the interp must handle `T_UNSAFE` (run the body). `unsafe`
  types as `unit` and yields no value — surface results via `return` from inside or
  by assigning an outer `let mut`. And `out` is a reserved keyword — never a
  binding name. (4) Identify Alloc/AllocVtable STRUCTURALLY at startup (vtable =
  struct with `alloc`+`free` fn-ptr fields; handle = struct whose `vt` is a rawptr
  to it), mirroring the oracle's `Interp::new`, NOT the checker (which only uses
  Alloc as its own prelude, no predicate) — this is the S5b box seam.
  (self-interp S5a, 2026-07-11). See [[self-host-analyses]].

- **Self-interp S5b (the heap): synthesize `BoxResult` as a real enum at startup
  to REUSE match/drop, and save/restore the return register around every drop
  loop because alloc-on-drop's `free` is a nested call.** Two load-bearing points
  for the box slice: (1) The parser emits no enum/variant nodes for `BoxResult T`,
  so at startup (where `write P` is in hand) APPEND, per `T_BOXRESULT` annotation
  node, a `{boxed(Box T), oom}` shape — a `T_BOX` payload node, two `T_VARIANT`
  nodes whose name spans point at the `boxed`/`oom` BYTES scanned from source (so a
  pattern name compares equal via `name_eq`), and a `T_ENUM`, linked back through
  the `T_BOXRESULT` node's spare `.c` field. Route `enum_of_ty(T_BOXRESULT) -> .c`
  and the entire S4 `match`/`drop_variant_rev`/enum-size machinery works UNCHANGED;
  only `T_BOX` needs new arms (drop_box + is_copy/needs_drop/size/align). Box's own
  result type node rides a new `cur_exp_ty` register set by the enclosing `let`, so
  box results must be let-bound with an explicit `BoxResult T` annotation (which
  also dodges the E0302 partial-move-at-join a fall-through `match` raises). (2)
  ALLOC-ON-DROP HAZARD: `drop_box` frees through the vtable via a real INDIRECT
  `call_fn`, executed DURING a drop; that nested call overwrites the self-interp's
  SHARED `ret_val`/`ret_w` registers with the freed fn's unit return. A `return v`
  inside a `match` arm whose Box frees on the way out silently returned 0 (the
  `.*`-read was correct — only the return register was clobbered). Fix: save/restore
  `ret_val`/`ret_w` alongside `cur_val`/`cur_w`/`cur_ty` around EVERY drop loop
  (`eval_match`, `exec_block`, `call_fn`, `exec_stmt` temps). Any interpreter with a
  register-based return and alloc-on-drop has this seam — a drop that runs a
  function after a `return` must not leak into the pending return value. Drop order
  is pointee-then-free, and it recurses (Box of a Box). (self-interp S5b,
  2026-07-11). See [[self-host-analyses]].

- **Self-interp S6a (paged memory + pointer intrinsics): page the byte store so
  the oracle need only init touched frames, and two authoring traps — untyped
  `0` in an array-repeat and no `conv` in the interpreted subset.** The systems
  corpus uses fixed addresses to ~16.9 MiB; a dense 17 MiB `[N]u8` in `E` is
  memory-feasible but INIT-infeasible because the oracle running interp.cnr
  initializes an array-repeat with a guarded per-byte move (~18M for 17 MiB). Fix:
  a PAGED store — `pages` (a small 128×4096 frame pool), `pagedir` (page-number →
  slot, sentinel -1, covering 0..32 MiB), `page_bump`; `xlate(addr)` = `page =
  addr>>12`, `off = addr&4095` (shift+mask, no division), binding+ZEROING a frame
  on first touch. Route every accessor byte through `xlate` (per-byte, so cross-
  page load/store/copy is automatic). Zero-on-page-alloc is load-bearing: a whole-
  value copy of a padded enum reads the variant's unwritten tail, which must read 0
  to match the oracle's init-byte guard. TWO TRAPS that cost time: (1) an array-
  repeat element `[0 - 1i32; N]` with an UNTYPED `0` mis-types the element and trips
  the oracle's uninit guard (bad_pointer read) at construction — write `[0i32 -
  1i32; N]` so the element is cleanly i32 (used for the pagedir -1 fill). (2) The
  self-interp does NOT implement `conv` in the INTERPRETED program (no T_CONV arm;
  it falls through to 0), so a fixture's `conv i64 <usize>` silently yields 0 —
  fixtures must branch/compare against typed literals instead of casting. The
  self-host bump bases stay low and diverge from the oracle's STACK_BASE; only
  values are observable, so fixed high addresses never collide with sub-MiB locals.
  The dropped `init` bitmap was write-only in the self-host (no guard). Cost: paging
  adds ~0.12 s/fixture (~25%), the oracle's per-fixture 512 KiB `pages` zero-init.
  (self-interp S6a, 2026-07-11). See [[self-host-analyses]].

- **Self-lowering L4 (enums+match): a scalar move/copy-bind must emit `Store(place
  local, Load)` not `Assign(local, Load)`, and enum-payload moves need a SYNTHETIC
  `_i` path segment in the move mask.** Two byte-exactness points for the `match`
  lowering in `selfhost/lower/lower.cnr` (both reused by L5 BoxResult): (1) build.rs
  copy-binds a wordy payload with `StatementKind::Store(Place::local(l), Load{..})`,
  NOT `Assign(l, Load)` — the two execute identically (a local scalar store), so the
  execution gate stays green either way, but `serialize(mir::build …)` carries the
  extra `(place l (proj))` nesting; matching it is what makes the wire byte-identical
  modulo spans. (2) A consuming match that move-binds a non-copy payload marks the
  scrutinee's `_i` sub-path moved so its tag-directed enum `Drop` prunes it (no
  double-drop); that path segment is the SYNTHETIC name `_i` (build.rs
  `format!("_{i}")`), not a source-byte span, so the move-mask side table needs a
  parallel `syn` flag (>=0 = payload index, emitted as `"_N"`; -1 = a real source
  span) — `seg_is_prefix` must compare the syn tag, or two synthetic `_0`/`_1` with
  empty spans compare equal via `name_eq`. VERIFICATION METHOD (reusable): the gate is
  EXECUTION equality (RET/TRACE/FAULT), but to confirm structural fidelity, whitespace-
  tokenize both wires and count diffs where BOTH tokens are numeric (`trim '-' && all
  ascii_digit`, incl. digits glued to trailing `)`) = inert spans; any non-numeric diff
  = a real structural gap. After the Store fix, all 6 S4 fixtures showed 0 structural
  diffs. Enums carry no drop hook — enum drop is WHOLLY the interp's tag-directed
  `drop_value`; the lowerer emits one `Drop` per needs-drop enum local (self-lower L4,
  2026-07-11).
