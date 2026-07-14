# Project memory

Durable lessons from past working sessions on Candor. Read before starting
non-trivial work; append a lesson when something worth remembering surfaces.
One lesson per entry, one-line summary first.

## Lessons

- **Use `cargo nextest run --profile fast` for the inner dev loop (~2.3 s).** Plain
  `cargo test` runs the 44 test binaries sequentially (~15 min); `cargo nextest run`
  parallelizes all tests in one process pool → ~3.2 min full suite (gated by the
  self-host corpus integration tests, each a couple of minutes of interpreted Candor
  over the corpus — that's the floor without optimizing the build). The `fast` nextest
  profile (`prototype/.config/nextest.toml`) drops the slow integration binaries
  (self-host, aot/llvm/stage native gates, freestanding, concurrency_native, golden)
  for a ~2.3 s / 518-test edit-check loop; CI still runs the full default profile.
  Deliberately did NOT add `[profile.test] opt-level` — it speeds the interpreted
  self-host tests but slows every compile, and the fast profile already fixes the
  loop pain. nextest is process-per-test, and the suite is green under it with no
  serial group (the foreign-io/shell-out tests are already process-isolated). Do NOT
  lower the llvm gate fixtures to `-O0` to save clang time — the whole point of that
  gate is that `clang -O2` preserves observable semantics (LLVM S6). (2026-07-12).

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

- **LLVM-S3 (drop schedule + trace-on-drop): the ENTIRE schedule is already baked
  into the MIR — the only new emitter work is drop-glue expansion, and there are NO
  runtime drop flags.** The S1/S2 "MIR bakes it in" lesson holds strongest here.
  `mir::build` already places each `Drop` at scope exit / early-return / break in
  reverse declaration order and stamps a STATIC move mask (`moved: Vec<Vec<String>>`,
  the field-name sub-paths moved out at that point); conditional-move correctness is
  compile-time, and the checker forbids inconsistent move state at CFG joins (E0302),
  so every drop site has ONE deterministic mask (design 0010 §2 INV-DROP). So the
  LLVM backend only needed `emit_drop`/`drop_enum`/`needs_drop` as an op-for-op mirror
  of `lower::emit_drop`: struct = fire the `drop` hook (`call cnf_<drop Name>(addr)` —
  the hook body, already an ordinary MIR fn, is where the observable `trace` lives)
  THEN fields reverse; array = elements reverse; enum = load u64 tag@0, per droppable
  variant `icmp eq` -> per-variant block dropping payload reverse -> merge; pruned by
  the same `is_moved`/`partially`/`prefix` mask predicates (a partial move skips the
  struct's whole-value hook). Box/BoxResult drop needs the allocator's vtable `free`
  (S4) — reject precisely. TRAP when authoring fixtures: an enum `match` that
  move-binds a needs-drop payload with a sibling arm that doesn't is E0302 (the
  MEMORY enum trap) — never reaches codegen; use a struct/array/enum-scope drop
  instead. The value-drop corpus fixtures (`generics/gdrop*.cnr`) are in-subset; the
  Box corpus (`11_1_allocator`/`11_5_arena`/`11_4_parser`) rejects BEFORE any drop, on
  the allocator-handle `CopyVal`. (LLVM-S3, 2026-07-12).

- **Adding a new native backend behind the MIR seam = walk MIR + mirror `lower.rs`
  op-for-op, because `mir::build` carries almost all the structure.** The whole LLVM
  backend (S0-S6, `src/backend/llvm.rs`, ~2100 lines) confirmed this at every slice:
  match is already lowered to a tag `Load` + `icmp`/`Branch` chain; drop schedules are
  pre-placed with static move masks (no runtime flags); struct/enum/array field offsets
  are baked into `Proj::Field`; box/unbox/rawptr are `BoxOp`/`UnboxOp`/observable
  `Load`/`Store`; a fn-as-value is `Const(id)` indexing `prog.fn_ptrs`; extern is a
  `Call` keying `items.externs`; spawn/scope are explicit statements. So each slice
  reduced to emitting the LLVM twin of what Cranelift's `lower.rs` emits — the emitter
  never re-derives semantics. **The one genuinely new decision** was the two-tier value
  model (`classify_tiers`): address-never-taken scalars -> `alloca i64` (mem2reg -> SSA
  registers, the real perf win under clang -O2); aggregates + address-taken -> flat
  `rt_stack_alloc`+`inttoptr(MEM_BASE+off)` (correct but optimizer-opaque — the
  permanent flat-arena aliasing ceiling). **Correctness traps that bite every backend:**
  NEVER emit `nsw`/`nuw` (LLVM deletes overflow checks as UB — use `llvm.*.with.overflow`
  + guarded div); keep `rt_*` as bare external declares (they're the optimization
  barriers that preserve trace/fault observability under -O2). CollectionOp (Vec/Map/
  String) is out-of-subset in Cranelift TOO — a shared corpus boundary, not a per-backend
  gap. Equivalence is transitive-through-one-oracle (each engine == tree-walker), not an
  all-pairs diff — state it that way, don't overclaim. Next backend (ARM/etc.) should
  start from this same walk-MIR-mirror-lower.rs playbook. (LLVM S0-S6 arc, 2026-07-12).

- **The generic check path silently drops per-fn side-reports (`foreign_report`) that the
  concrete path returns.** `check_program_real_foreign` returned `Vec::new()` for any
  generic program, so `candor audit` reported empty foreign discharge/reach for a boundary
  module that was ALSO generic — teeth lost exactly where a generic I/O wrapper lives. The
  info is NOT lost pre-monomorphization: the trust boundary is a source-level property, and
  `check_generic_program_own` already runs `check_fn_with_sig` on every fn (concrete, generic
  def-site, impl method, hook), each pushing its `ForeignFnInfo`. The def-site inner
  `Checker`s just dropped their `foreign_report` while propagating `diags`/`insts`/`shapes`.
  Fix = propagate it too + return it (new `GenericForeignCheck`/`check_generic_program_foreign`).
  LESSON: when a report is computed per-`check_fn` and one path forks into inner `Checker`s,
  audit that EVERY inner-checker field the outer needs is propagated back, not just diags.
  (P17 audit-generic, 2026-07-13).

- **Dropping a self-contained `.cnr` into `tests/fixtures/run/` auto-enlists it in
  every native gate.** `tests/stage_d.rs` (four-engine: oracle·MIR·native-noopt·
  native-opt), `tests/aot.rs` (Cranelift ELF), and `tests/llvm.rs` (clang -O2 ELF)
  all scan `run/parity/real/generics` — so one fixture proves an allocator/box
  program on six engines for free. But those corpus gates only assert cross-engine
  AGREEMENT with the tree-walker oracle, NOT a specific value; to prove SEMANTICS
  (e.g. that a freed block was reused) you must add an explicit test asserting the
  RET (e.g. `tests/freelist.rs` via `run_source_real{,_mir,_native,_native_opt}`),
  because a broken program that returns the wrong value consistently still passes
  the agreement gates. Also: `box(a, expr)` fails MIR type-inference on an inline
  arithmetic payload ("cannot infer box value type") — bind it first
  (`let p: i64 = ...; box(read a, p)`). And adding a file to the corelib module
  tree bumps `tests/stage_c.rs`'s hardcoded module count (8→9). (2026-07-13).

- **An ADDRESS-ORDERED intrusive free list gives both-sided coalescing with no
  boundary-tag footer.** For the `FreeBlock { next: rawptr u8, size: usize }`
  rawptr model, keeping the free list sorted by address (insert-in-order in
  `free`) makes forward coalescing (merge `cur` when `addr + cap == addr(cur)`)
  AND backward coalescing (extend `prev` when `addr(prev) + prev.size == addr`)
  fall out of the single insertion walk — no per-block footer, no header
  redesign, layout unchanged. Splitting stays compatible: the remainder takes the
  split block's slot (its address is > cur and < the successor), preserving order
  and the "no two free blocks physically adjacent" invariant. A merge fires only
  on an EXACT byte-span boundary using the identical `block_span` rounding alloc
  used, so it can only ever join physically-adjacent blocks (no overlap, no gap).
  All addressing via `ptr_to_addr`/`addr_to_ptr` — no new primitive, lowers
  natively like the existing MVP. (FREELIST-ALLOC split+coalesce, 2026-07-13).

- **Native collection gates for needs_drop elements must drain the collection empty
  at scope end (until collection-drop lands).** The MIR/tree interp's `drop_value`
  has a `Type::App("Vec"/"Map")` arm that drops each live element + frees the buffer
  at scope end, but native `emit_drop`/`needs_drop`/`walk_glue` have NO `Type::App`
  arm yet (collection-drop is a separate later slice) — a live collection leaks
  natively. That leak is observable-invisible ONLY if no element drop fires at scope
  end. So a byte-exact native drop gate (e.g. Vec `set` drop-on-overwrite with a
  hook-bearing element) must pop/consume every remaining element so the collection is
  empty when it goes out of scope; otherwise interp fires the surviving hooks and
  native doesn't → trace divergence. Drop-on-overwrite itself (VecSet → `emit_drop`
  of the old element) IS native and byte-exact for hook-bearing/non-Box elements; a
  `Vec<Box>` overwrite would silently skip the Box free because `walk_glue` lacks a
  `Vec` arm to collect the element glue — fold both into the collection-drop slice.
  Also: the LLVM backend tiers wordy locals to registers, so a `CollectionOp` result
  read through `place_addr` (e.g. VecGet's `read elem` borrow dst, or a VecPush/Set
  value operand) must be marked Tier-F in `classify_tiers`; Cranelift stack-allocates
  every local so it needs no such change (native collections slice #3, 2026-07-13).

- **Native collection drop = the interp `drop_value` mirror, gated by the counting
  allocator's `live` balance.** `emit_drop`/`needs_drop`/`walk_glue` (both
  `backend/lower.rs` + `backend/llvm.rs`) needed a `Type::App("Vec"|"Map")` +
  `Type::Named("String")` arm to free the buffer (and drop live elements/values, free
  Map keys) at scope end — the `String` arm MUST precede the generic-struct arm since
  `String` is a synthesized nominal struct. The move mask already suppresses a
  moved-out collection's drop (`mir::build::emit_drop` emits no `Drop` when the whole
  value is moved, same as Box) — no drop flags needed. Prove free/no-leak cheaply by
  running the fixture over the COUNTING BUMP allocator (`tests/vec.rs` ALLOC:
  `live = allocs - frees`) and returning `bs.live` — the aot/stage_d/llvm corpus gates
  auto-scan `tests/fixtures/run/*.cnr` and compare native to the oracle, so `live != 0`
  (leak) or `< 0` (double free) diverges and fails. Prove REUSE with the reclaiming
  free-list + a tight window + a many-iteration build-and-drop loop (a leak OOM-faults
  `vec_reserve`). GOTCHA: the CLI binary is `target/debug/candor` (NOT `candor-proto`,
  which is a stale leftover) — `cargo run --bin candor` or nextest, never the stale
  path. LLVM loop bodies that call `emit_drop` need the loop-carried `phi` next-value
  computed in a dedicated back-edge block so the `phi` predecessor stays a concrete
  label (the map_grow rehash pattern); Cranelift uses `BlockArg` params and is
  immune. This CLOSED the native-collections arc (S5, 2026-07-13).

- **The native str-view family is complete: `str_from` (validated → `Utf8Res`) and
  `substr` (char-boundary slice → str) both landed native, joining the already-native
  `str_from_unchecked`/`as_bytes`.** Both are new top-level `StatementKind`s
  (`StrFrom`/`Substr`, twins of the existing `Subslice`), NOT `CollOp`s — they have no
  5-word collection header. `str_from` needs a real UTF-8 validation LOOP in each
  backend; `substr` is `Subslice` + two char-boundary byte-checks (no loop). Plumb a
  new StatementKind through SIX sites: `mir/mod.rs` (def), `mir/build.rs`
  (`is_builtin` + `lower_builtin_into` — and `lower_scrutinee` needs a
  `builtin_static_ret` arm so `match str_from(x)` types its scrutinee, else "match on
  an indirect call"), `mir/interp.rs` (handler), `mir/opt.rs` (`stmt_uses` liveness;
  non-Assign/Store kinds are never DCE'd so no removal logic), `mir/serial.rs`
  (ser+deser round-trip), and BOTH backends (`backend/lower.rs` Cranelift +
  `backend/llvm.rs` LLVM emit, plus llvm `classify_tiers` must mark dst+src Tier-F
  since they go through `place_addr`). The MIR interp mirrors the tree-walker by
  calling `std::str::from_utf8().valid_up_to()` directly (byte-exact, no hand loop);
  only the two NATIVE backends hand-roll the scan. Key facts that made it byte-exact:
  (1) Rust's `run_utf8_validation` sets `valid_up_to = old_offset` (the START of the
  bad sequence) for EVERY failure class, so the native loop's single `invalid` exit
  just carries the current `index` — no per-byte sub-offset tracking. (2) The 3-byte
  second-byte range collapses to `lo = (b0==0xE0)?0xA0:0x80, hi = (b0==0xED)?0x9F:0xBF`
  and 4-byte to `lo = (b0==0xF0)?0x90:0x80, hi = (b0==0xF4)?0x8F:0xBF` — one select
  pair each, covering overlong+surrogate+range. (3) width via lead class 0xC2..=0xDF
  / 0xE0..=0xEF / 0xF0..=0xF4 (else invalid lead) then an upfront `index+w<=len`
  presence check is offset-equivalent to Rust's per-byte `next!()`. GOTCHA (cost me a
  Cranelift verifier round): in a multi-block loop, a value computed in one branch
  (e.g. `p1 = idx+1` in the width-2 arm, or the `iconst 1/2/3/4` step constants) must
  NOT be reused in a sibling branch it doesn't dominate — "uses value from
  non-dominating inst". Hoist the shared step constants to the entry block (they then
  dominate all) and recompute each branch's byte offsets LOCALLY. LLVM's textual phi
  forgives forward-refs so the same structure with a single back-edge (`adv1..adv4 ->
  head` phi) just works. `substr`'s boundary check needs a guarded load (skip when
  `i==0||i==len` so `i==len` never reads past the run) — a small diamond (Cranelift
  `BlockArg` / LLVM `phi i1`); mind Cranelift `icmp` returns `I8`, so the skip-path
  const must be `iconst.i8 0`, not the I64 `iconst` helper. `substr` faults reuse
  `FaultKind::Bounds` at the CALL span (`lower_builtin_into`'s `self.cur_span` == the
  tree-walker's `bi_substr(args, span)` span) for both the out-of-range and
  non-boundary cases; the gate checks kind+span only, so the two distinct messages are
  fine. One `tests/fixtures/run/str_view.cnr` (self-contained `fn main() -> i64`, no
  allocator) auto-enlists in all six corpus engines; flip the interp-only `text.rs`
  str_from/substr cases to all-engine via `run_ret_all`/`run_fault_all` (oracle·MIR·
  native-noopt·native-opt; LLVM via the fixture). NOTE `str` byte-INDEXING (`s[i]`) is
  still NOT native — MIR `Index` lowering handles Array/Slice only, not `Type::Str`
  ("index of non-array"); a `str_from`-then-use test must read the recovered view via
  `as_bytes(s)[i]` (native slice index), not `s[i]`. (str-view native, 2026-07-13).
- **Loan provenance closes over ALL borrow-kind returns, not just `borrow`/`borrow_mut`.**
  A user fn returning a VIEW (`slice T`/`slice_mut T`/`str`/`[u8]`) is a borrow-kind
  return exactly like a `read`/`write` borrow return: the view aliases the source's
  backing. But four checker gates spelled `matches!(sig.ret, Type::Borrow(_)
  | Type::BorrowMut(_))` while `region_source_indices`/`is_borrow_param` already used
  `is_borrow_kind()` for PARAMS — so view returns silently shed the argument loan.
  Repro (check-clean, stale/freed read on native, sound answer differs): `fn view_of(
  s: read String) -> [u8] { return as_bytes(as_str(read s)); }` then caller `append`s
  (grow = free-old) while the returned `vb` is live. Fix: generalize all four gates to
  `sig.ret.is_borrow_kind()` — `ret_is_borrow` (mod.rs:763, → E0806 return-provenance),
  `check_signature_regions` (mod.rs:1059, → E0807 on 2+ borrow-param view returns),
  `check_user_call` return-extension (expr.rs:1402), `borrow_provenance` user-call
  branch (expr.rs:920) — plus `carries_borrow`'s user-Call arm (stmt.rs). Do NOT add a
  SEPARATE view-provenance analysis: views obey the same "return directly from the
  param, laundering through a local is E0806" rule borrows do (verified: borrow launder
  through a local is already E0806). Zero corpus false positives (707 nextest green);
  the one view-returning fixture (`slices.cnr` `first[region r](s: [i64]) -> [i64]`,
  single slice param) stays clean because `is_borrow_param` marks slice params `is_bp`.
  (loan-provenance function-return views, 2026-07-13).

- The loan-provenance return-extension (LOAN-COPY / STR-VIEW fix) covered only
  FREE-fn (`Ident`-callee) borrow returns; INTERFACE-METHOD (`Field`-callee) borrow
  returns shed the receiver's loan — a stale-borrow hole the completeness sweep
  missed because `arena_get` is called as a free fn, never a method. It bit design
  0015's `for read` end-to-end: the desugar emits `__c.get_ref(__i)` (a method call),
  so `let esc = c.get_ref(0); write c` (the escape §5 forbids) checked CLEAN. Two
  places both needed the SAME return-extension the free-fn path already had, applied
  to the method-call shape: `check_iface_method_call` (generics.rs) must carry the
  receiver's loan when a `read`/`write self` method returns a borrow deriving (compact
  default, single borrow-in) from the receiver, AND `carries_borrow` (stmt.rs, made
  `&mut self` + `method_returns_borrow`) must admit that method call so the landing
  binding anchors it. Lesson: when adding "a borrow return keeps its source loan,"
  cover BOTH callee shapes (`Ident` and `Field`) — they go through different check
  paths (`check_user_call` vs `check_iface_method_call`). Also: `for read`'s escape
  soundness for the LOOP-LOCAL cursor is conservative — assigning the yield to an
  outer local is rejected outright (design 0015 §7 open-Q1 non-escape fallback), sound
  but rejects escape-without-later-mutation; transitively threading `__c`'s binding
  loan through the method return caused a spurious loop self-conflict, so the simpler
  direct-receiver-loan carry (reject any escape) is what shipped. And: a `Vec` user
  `get_ref` returning `get(read self.v, i)` fails E0806 (`get`'s return not recognized
  by `borrow_provenance`); use the `arena_get` place-reborrow `read self.slot` instead.
  (method-return loan-provenance + for-read borrowed iteration, 2026-07-14).

- **WASM interpreter M0 (decode-and-run spine, in Candor) — five Candor-authoring
  facts, one spec correction.** Built `tests/fixtures/wasm/interp.cnr` +
  `tests/wasm.rs`: byte cursor (`Reader { pos }`) + real LEB128 (unsigned
  `read_u32`, signed `read_i64` with the continuation-bit loop + 0x40 sign-
  extend), an id-dispatched section walk, and an opcode-dispatched eval loop over
  a fixed `[256]i64` value stack. i32-in-i64 rep: an i32 is held SIGN-EXTENDED
  from bit 31; every arith result re-normalized by `(x << 32) >> 32` in
  `wrapping{}`. Authoring facts that bit: (1) `wrapping { }` is a STATEMENT block,
  not an expression — `let s = wrapping { a+b };` is a parse error; assign an
  outer `let mut` inside the block (`wrapping { s = a+b; }`). (2) Candor `match`
  has NO integer-literal patterns (PatKind = Wildcard|Binding|Variant only), so
  opcode/section dispatch is an if/else-if chain, not `match`. (3) `result` and
  `out` are reserved keywords — never binding names (rename to `acc`/`norm`). (4)
  Forwarding a `write S` PARAM to another fn reborrows when passed BARE (`f(r)`),
  NOT `f(write r)` (that double-borrows: E0703 `borrow_mut borrow_mut`); but a
  `let mut` OWNER passes `write r`, and a param's field write needs the deref
  `r.*.pos = ..` while an owner's is bare `r.pos = ..`. (5) `panic("..")` →
  FaultKind::Panic and `assert(c)` → Assert both lower on all engines (kind+span
  match; message may differ) — use them for decode/eval error paths. SPEC
  CORRECTION: the M0 brief says "34-byte module" but its own byte listing is 30
  bytes of valid wasm (8 header + 7 type + 4 func + 11 code); used 30. The Rust
  harness re-encodes the same module independently (`wasm_add_module`, its own
  signed-LEB) and asserts byte-equality to the spec bytes — two independent LEB
  impls agreeing (Rust encode vs Candor decode) is the real correctness signal,
  not a hardcoded 42. Runs on all 6 engines: dropping the file into
  `tests/fixtures/run/` auto-enlists it in aot/stage_d/llvm; `tests/wasm.rs`
  asserts the VALUE 42 (+ 10+20, 1000+(-500), i32 wrap, malformed-magic Panic) on
  oracle·MIR·native-noopt·native-opt. Keep the run/ copy byte-identical to the
  canonical (a drift-guard test). (WASM M0, 2026-07-14).

- **WASM interpreter M1 (multi-function decode + locals/frames + full i32/i64
  numeric + `call`/recursion + structured control) — the load-bearing lesson is
  that a RECURSIVE eval loop overflows the tree-walking oracle's HOST stack at
  shallow WASM depth (fib(10) SIGABRTs), so `call` must run on an EXPLICIT
  activation stack.** Built on M0 (`tests/fixtures/wasm/interp.cnr` + `tests/wasm.rs`):
  section walk now collects Type arities, Function type-indices, the Export
  section ("main" entry), and Code bodies (byte range + summed local count) into
  a fixed-capacity `Module`; frames hold params-then-zeroed-locals in a `[32]i64`;
  full numeric via `eval_unop`/`eval_binop` leaf helpers (clz/ctz/popcount
  hand-rolled over `u64` shifts; div/rem trap on zero and signed INT_MIN/-1;
  unsigned ops mask to width for i32, `wrapping { conv u64 }` for i64);
  structured control via a per-frame label stack (`do_branch`: loop target →
  jump to start + keep label; block/if target → `scan_forward_exits` past the
  matching `end` + pop; blocktype arities via single-byte `read_blocktype`; `if`
  uses `skip_to_else_or_end`). Facts that bit, in priority order: (1) HOST-STACK:
  a plain recursive `eval_call` runs fine on the CLI (main thread, 8 MB) up to
  fib(20) on tree/mir/native, but SIGABRTs in `cargo nextest` at fib(10) on the
  ORACLE — nextest test threads have a smaller stack and the tree-walker's
  per-Candor-call host frame is heavy, so WASM-recursion-depth × that frame
  overflows. Cranelift NO-OPT also gave `eval_call` a huge frame (the ~90-branch
  numeric chain), overflowing native ~fib(15) even after shrinking arrays.
  Definitive fix = an explicit activation stack: a non-recursive `exec` driver
  keeps the working frame in locals and snapshots callers into a `[256]Act`
  array on `call`, restoring on return — HOST recursion is O(1), so fib(20) runs
  on every engine. Factoring numeric dispatch into leaf helpers helped but was
  NOT sufficient; the explicit stack is what mattered. (2) `conv u64`/`conv i64`
  across the sign boundary is CHECKED — `conv u64 (negative i64)` faults
  `conv_loss`; every signed↔unsigned reinterpret must sit inside `wrapping { }`.
  (3) A struct with array fields that you assign around (snapshot into / restore
  out of an array slot) must be a `copy struct` — moving a non-copy value out of
  `saved[fp]` or `f = s.frame` is E0310/E0301; `copy struct Frame`/`copy struct
  Act` (all-scalar/array-of-copy fields) makes assignment copy. (4) Structured
  forward branches need one immediate-aware skipper (`skip_immediates`) so a
  `block/loop/if` forward scan never mistakes an operand byte for an opcode; the
  scan counts `end`s at depth 0 to resolve a forward branch out of N enclosing
  blocks, and `br` to a `loop` jumps backward to a recorded start_pos.
  (5) blocktype kept single-byte (0x40 / valtype) — multi-byte type-index
  blocktypes are rejected, fine for hand-encoded MVP modules. Gate: fib(10)==55
  (+0,1,7,15), sum(10)==55 (loop+br_if), div_s/div_u/rem/shr_s/shr_u/rotl/clz/
  popcnt/unsigned-compare/i64 ops each vs known values, br_table target+default,
  and divide-by-zero/INT_MIN÷-1 → Panic, all byte-exact on oracle·MIR·native-
  noopt·native-opt (encoder asserted byte-equal to hand-listed fib/sum spec).
  732 nextest green (incl. selfhost gates over the enlarged interp.cnr + the
  AOT/stage_d corpus copy), clippy clean. M2 next: linear memory load/store +
  bounds trap over a `Vec[u8]`. (WASM M1, 2026-07-14).

- **WASM interpreter M2 (linear memory: memory/data section decode + load/store +
  memory.size/grow over a native `Vec[u8]`) — the load-bearing fact is that the
  Vec builtins take `self` by an EXPLICIT borrow expression, so forwarding a
  BORROWED `Vec` PARAM to `get`/`set`/`push`/`len` needs a DEREF-reborrow
  (`set(write mem.*, ..)`, `get(read mem.*, ..)`), not bare and not `write mem`.**
  Built on M1: `decode_module` now also decodes the Memory section (id 5: flags +
  min pages LEB, opt max) and Data section (id 11: active segments — flag 0,
  `i32.const N; end` offset via `read_const_offset`, byte vector; stored as
  (offset, byte-range) in fixed `[16]` arrays). `run_module` instantiates the
  linear memory as a native `Vec[u8]` over the SAME free-list allocator that
  `vec_native.cnr` uses (`with_window(16MB, 8MB)` + `mk_alloc`), pushes
  `min_pages*65536` zero bytes, copies each active data segment in (bounds-checked
  → panic/trap), then runs `exec(bytes, read m, write mem)`. Load/store leaf
  helpers (`do_load`/`do_store`/`mem_load_bytes`/`mem_store_bytes`/`sign_extend`,
  factored OUT of `exec` like the M1 numeric helpers to keep its frame small):
  memarg = align LEB (ignored) + offset LEB; effective addr = `(popped_i32 &
  0xffffffff) + offset` (unsigned usize); bounds check `eaddr + size > len(mem)` →
  `panic("out of bounds memory access")` (FaultKind::Panic, engine-consistent);
  little-endian assemble/disassemble in `wrapping{}`; sign-extend via
  `(v<<(64-bits))>>(64-bits)`, zero-extend by leaving the raw; i32 results
  re-normalized with `as_i32`. memory.size (0x3f) reads the reserved memidx byte
  and pushes `len/65536`; memory.grow (0x40) pushes `delta*65536` zeros (respecting
  a declared max), returns OLD pages or -1. Facts that bit, in priority order:
  (1) BORROW-FORWARDING: `set`/`push`/`get`/`len` type-check arg0 as `Use::Value`
  over an explicit `write v`/`read v` borrow, so an OWNER (`let mut`) passes
  `write v`/`read v`, but a `write Vec[u8]` PARAM must forward a DEREF-reborrow
  `write mem.*`/`read mem.*` — bare `set(mem,..)` MOVES the param (E0301), and
  `set(write mem,..)` double-borrows (E0703 `borrow_mut borrow_mut`); this differs
  from the M1 struct-param rule (bare reborrows) precisely because the Vec builtins
  consume an explicit borrow arg. (2) Using a native `Vec[u8]` forces the whole run
  path to carry the `alloc` effect (`run_module`/`exec`/`main` + the store/grow
  helpers) and an allocator handle; zero-init is per-byte `push`, ~0.37s for one
  64KiB page on the tree-walker (debug), acceptable at the test scale (min 1 page)
  but O(page_bytes) — a bulk `Vec`-of-length constructor would remove that cost
  (roadmap: no `vec_with_len`/repeat intrinsic exists today). (3) `conv u8`/`conv
  usize` are CHECKED — mask the assembled byte to `& 0xffi64` and the address to
  `& 0xffffffffi64` (nonneg) before converting, inside `wrapping{}` for the shifts.
  (4) `skip_immediates` must learn the memory ops (0x28-0x3e: 2 LEBs; 0x3f/0x40:
  1 reserved byte) or a structured-control forward-scan over a block containing a
  load/store mistakes an operand for an opcode. Gate (byte-exact on
  oracle·MIR·native-noopt·native-opt, + AOT/LLVM/stage_d over the corpus copy):
  i32/i64 store→load round-trip, width+sign/zero (load8_s→-1/load8_u→255, 16-bit,
  i64.load8_s→-1, i64.load32_s→-1), little-endian order (store 0x04030201 →
  load8_u@0=1, @3=4), data-segment init (0xDEADBEEF placed + read back), OOB
  load/store/data-segment → Panic, memory.size/grow (old-count return, size
  reflects growth, new region reads zero, max-exceeded → -1); encoder asserted
  byte-equal to a hand-listed memory.size module + a data-segment module. 739
  nextest green (+7 M2 tests over M1's 732; full + fast profiles), clippy clean,
  run/ copy byte-identical. M3 next: read a real `.wasm` off disk (read_into +
  read_all_bytes) and gate output byte-exact vs a wasmtime/node reference over a
  module corpus. (WASM M2, 2026-07-14).

- **WASM interpreter M3 (real `.wasm` off disk + a DIFFERENTIAL gate vs the
  independent `wasmi` spec reference) — three load-bearing facts: (1) the Candor
  interp is NON-VALIDATING, so a type-incorrect module RUNS on it but a spec
  reference REJECTS it — a differential corpus must be well-typed (an i64 compare
  yields an i32, so its function result is I32 not I64; an i64.store8 takes an i64
  value, not an i32.const); the two encoder bugs this surfaced are exactly what a
  real reference catches that a hand-written expected value cannot. (2) Reading a
  module off disk into a `Vec[u8]` needs a free-list window DISJOINT from the
  [16MiB,24MiB) one `run_module` carves for the module's linear memory — overlap
  is silent until the Vec's Drop walks a free-chain the interp's memory clobbered;
  the file reader uses [32MiB,40MiB). (3) There is NO `Vec[u8] -> [u8]` builtin,
  so bridge the read bytes through a fixed stack array + `subslice(slice_of(arr),
  0, n)` (run_module's `[u8]` signature — the byte-source-decoupled decode — is
  unchanged).**
  A. `read_all_bytes(a: read Alloc, fd) -> VecIoResult` (the raw-bytes sibling of
  `read_to_string`): loop `read_into` a fixed `[64]u8` stack buffer, `push` the `n`
  read bytes onto a growable `Vec[u8]` until `Ok(0)` (EOF); `?` propagates read
  errors via an identity `From[IoError]`. `run_wasm_file.cnr` = the std::io
  boundary prefix + the canonical interpreter VERBATIM (above the harness split) +
  the file-run tail; a drift-guard test (`file_run_fixture_reuses_canonical_interp`)
  keeps it in sync with `interp.cnr` (same discipline as the run/ corpus copy). Its
  `main` opens a fixed `mod.wasm`, `read_all_bytes`, then decodes+runs it. Proven
  reading a REAL 74-byte fib(10) module (> the 64-byte buffer, so the loop + Vec
  growth genuinely run) off disk → 55 on the tree-walker AND the MIR engine (via the
  foreign_io shims, `tests/wasm.rs`), AND on a native Cranelift binary AND a
  clang -O2 binary calling REAL libc open/read/close (`tests/aot.rs`,
  `tests/llvm.rs`, cwd = the temp dir holding `mod.wasm`).
  B. `wasmi` 1.1.0 added as a `[dev-dependencies]` (pure-Rust, no C deps, fetched +
  built offline; the library keeps its own deps). `wasmi_run` compiles+instantiates
  +calls exported `main`, normalizing an i32 result by sign-extension to match the
  interp's i32-in-i64 representation, and maps any validate/instantiate/trap error
  to Err. Six differential tests assert the Candor tree-walker result == wasmi over
  a corpus covering: i32 arith (add/sub/mul/div_s/div_u/rem_s/rem_u/and/or/xor/shl/
  shr_s/shr_u/rotl/rotr) × 6 operand pairs; i32 unary (clz/ctz/popcnt/eqz) +
  compares (eq/ne/lt/gt/le/ge, signed+unsigned); the full i64 set (arith, shifts,
  rotl/rotr, clz/ctz/popcnt, compares); control flow + recursion (fib, sum-loop,
  br_table); linear memory (i32/i64 store→load at every width, little-endian, data
  segments, memory.grow); and TRAP-EQUIVALENCE (div/rem by zero i32+i64, signed
  div overflow, OOB load/store, OOB data segment, unreachable — each traps in BOTH).
  The hand-asserted M0-M2 tests stay (a fast cross-engine check); the differential
  corpus runs on the tree-walker only (the task's "tree-walker at least") to keep
  the suite fast — cross-engine (MIR + Cranelift + clang) agreement over the same
  instruction classes is already gated by the M0-M2 `run_ret_all` tests + the
  file-run gate. 750 nextest green (+11 over M2's 739: 9 wasm + 1 aot + 1 llvm; full
  + fast profiles), clippy clean, run/ copy byte-identical. WASM MVP capstone done;
  post-MVP options: host imports / a WASI-lite print, f32/f64 floats (needs the
  language float prerequisite), a JIT-to-LLVM. (WASM M3, 2026-07-14).

- **WASM M4 — host imports + a WASI-lite `print` (real output through the I/O
  boundary). Three load-bearing lessons.** (1) THE FUNCTION INDEX SPACE: imported
  functions occupy the LOWEST func indices, so `K` func-imports shift every defined
  function to index `K + j`. The decode fix: the Import section (id 2) records each
  func import at index `nimports` (its type arities into the SAME per-function
  arrays), and the Function (id 3) + Code (id 10) sections store defined metadata at
  `nimports + i`. `call N` then routes `N < K` to a host handler and `N >= K` to a
  defined body with NO other change (the export entry index already lives in the
  full space). Only func imports bump `K`; table/memory/global imports decode-and-
  skip without consuming a func index. Gated by ONE module that imports print_str
  (0) + print_i32 (1) and defines helper (2) + main (3), where main calls import 0,
  then defined helper 2, which calls import 1 — every index-space case at once.
  (2) WHERE THE HOST LAYER LIVES (the drift-guard forces this): the pure `interp.cnr`
  reusable section CANNOT hold the std::io boundary — `tests/wasm.rs` compiles it
  standalone on every engine incl. native, so an `extern sys_write` would be an
  unresolved-symbol link error even if never called. So the host dispatch BUFFERS
  into a threaded `Vec[u8]` inside the pure exec (`host_call` → `print_i32` decimal /
  `print_str` copies LINEAR MEMORY, bounds-checked → TRAP on OOB), and the boundary
  fixture (`run_wasm_file.cnr`) FLUSHES the buffer to real stdout via `write_all`
  after the run. Buffering-then-flushing yields byte-identical captured stdout while
  keeping the reusable decode+exec extern-free and byte-identical across interp.cnr /
  its run/ copy / the file fixture. (3) `out` IS A CANDOR KEYWORD — name the host
  buffer `hout` (a `write Vec[u8]` param re-borrows as `write hout.*`, not `write
  hout`). The file main juggles THREE disjoint free-list windows: run_module's linear
  memory [16,24) MiB, the file bytes [32,40), the host-output buffer [48,56) (256 MiB
  arena, so all fit). The gate asserts captured stdout == expected on the tree-walker
  + MIR shims AND on real-libc AOT + clang-O2 binaries ("hello, wasm\n42\n"), an OOB
  print_str TRAPS on both interp engines, and — the print path IS wasmi-differential
  — a wasmi `Linker` with the SAME `env.print_i32`/`print_str` host funcs over a
  captured `Store<Vec<u8>>` buffer (print_str reads the EXPORTED memory, Errs on OOB)
  produces output byte-equal to Candor's. Modules that use print_str must EXPORT
  "memory" so wasmi's host can reach it (the Candor interp reads mem directly and does
  not care). Post-MVP remaining: f32/f64 floats (needs a language float prerequisite),
  a JIT-to-LLVM. (WASM M4, 2026-07-14).

## Adding a new scalar type (e.g. `f64`) — the full cross-engine checklist + traps
Adding `f64` (design 0016) touched ~18 files across every layer. The mechanical
spine: add the `ScalarTy` variant, then let `cargo build` enumerate every
non-exhaustive match (scalar_name/scalar_size/scalar_kw in serial+emit, scalar_range,
the four `ty_range` copies in eval/mir-interp/lower/llvm, and ~15 `ExprKind` matches
once you add the literal node). Key design/impl facts worth reusing:
- **Value repr: carry the f64 as its `to_bits()` pattern in the existing i128/i64
  "register" model + 8 bytes in flat memory** — every engine already stores scalars
  this way, so loads/stores/call-ABI need ZERO change. Only the float OPS bit-cast in
  (Cranelift `bitcast`, LLVM `bitcast i64→double`), compute native IEEE, bit-cast out.
  Make `ty_range(f64)` UNSIGNED-64 so no read_int sign-extends the pattern.
- **`is_integer()` must EXCLUDE the new type** — it gates arithmetic typing, `expect_integer`,
  literal-suffix validity, and the regime/overflow machinery. Floats being non-integer is
  what keeps them regime-exempt AND out of every integer path (no int test perturbed).
- **Observe f64 bits across all five engines via `trace`, not the return value.** The
  LLVM/AOT gate runs a separate PROCESS whose exit code is 8 bits; the reliable shared
  channel is θ (the printed trace). `trace(f64)` emits the bit pattern as i64. But `trace`
  forces `expected=i64` on its arg, which would force INT math on a float arith arg —
  probe the arg's static type and pass `expected=f64` (both build.rs and eval.rs).
- **TRAP: a computed NaN's SIGN BIT is IEEE-unspecified.** LLVM `-O2` constant-folds
  `0.0/0.0` to `+NaN` (0x7FF8…) while x86 runtime `divsd` yields `-NaN` (0xFFF8…). Gate
  finite/inf/conv results to EXACT bits, but gate NaN by BEHAVIOUR (comparison outcomes).
  The four in-process engines agree on the NaN bits (all runtime); only the folding LLVM
  process differs.
- **TRAP: do NOT widen a serialized MIR Rvalue's field set casually.** Adding `from: ScalarTy`
  to `Rvalue::Conv`'s WIRE broke the self-host lowerer's byte-exact MIR (it emits the old
  `(conv <to> …)`). Recover the source scalar from the OPERAND (`operand_sty(v)`) instead.
  For INV-CHECK uniformity, a checked f64→int conv keeps an INERT ConvLoss edge (saturating
  never takes it) rather than fault=None (which the invariant, lacking the source type,
  can't distinguish from a missing edge on an int→int conv).
- **f64→int must saturate to the EXACT target width** (`fcvt_to_*int_sat(I8/I16/I32/I64)` /
  `llvm.fpto*i.sat.iN`), then sign/zero-extend to the i64 register — saturating to i64 first
  then narrowing would wrap (300.0→u8 must be 255, not 44). Matches Rust `as`.
- Newton's-method sqrt(n) converges to ONE ULP short of the correctly-rounded library `sqrt`
  — assert loop results against the reproduced loop in host Rust, not against `f64::sqrt`.

- **`fmt_f64` (float->String) is `f64`-only-arithmetic bounded to ~15 significant
  digits — 17-digit round-trip is unreachable without a bitcast/bignum.** The
  formatter (in `tests/fixtures/std_fmt.cnr`, joining `fmt_i64`) normalizes `|x|`
  to a decimal exponent by comparing against a repeated-`*10` power-of-ten table,
  then forms the significand with ONE scaled multiply/divide kept `< 10^15 < 2^53`
  (an exact integer) — a digit-by-digit `*10` loop drifts and was rejected. This
  round-trips every `f64` nearest a ≤15-sig-digit decimal in `[1e-15, 1e39)`
  (30M-sample + Rust `f64::from_str` gate in `tests/fmt.rs`); 16-17-sig-fig values
  (raw `sqrt(2)`, `0.1+0.2`) provably cannot round-trip because a 17th correct
  decimal digit needs an integer past `2^53`. AUTHORING TRAP: `conv usize (a - b)`
  pushes the unsigned target INTO the subtraction, so a negative intermediate
  faults `conv_loss` — compute the index into an `i32` local first, then
  `conv usize (local)`. Cross-engine byte gate can't use the MIR-only `String`, so
  a String-free twin (`tests/fixtures/run/fmt_f64_trace.cnr`) traces the ASCII
  bytes and the corpus gates prove all five engines agree. (2026-07-14).

- **Integer-literal `match` patterns: the shared `PatKind` enum forces
  completeness edits in ~13 sites, and integer exhaustiveness is a HARD
  catch-all requirement (not variant enumeration).** Added `PatKind::IntLit
  { value, negative, suffix }` (`prototype/src/ast.rs`, with `int_pat_value`
  helper). An integer match can never enumerate 2^N values, so exhaustiveness is
  redefined: an integer-scrutinee match is exhaustive IFF it has a `_`/binding
  arm, else `E0601` — required for soundness (an unmatched value is UB). The
  checker (`check/expr.rs check_int_match`) dispatches off the `None` arm of
  `resolve_enum` when the scrutinee is `Type::Scalar(is_integer)`; it also does
  coherence (`E0606`: int pattern on enum scrutinee / variant pattern on int
  scrutinee / suffix mismatch), range-check (`E0709`) and duplicate-literal
  (`E0602`, a dead arm) checks. Lowering is a compare-and-branch chain reusing
  existing MIR ops (`Cmp Eq` + `Branch`), so BOTH native backends (Cranelift
  no-opt/opt, LLVM) needed ZERO change — confirmed by the auto-discovered
  `run/` corpus gates. The trap: a new `PatKind` variant breaks every exhaustive
  `match &pat.kind` — build/canon, real/emit+fmt, modules (rewrite_pattern),
  interp (eval_match + pat_matches + bind_pattern + bind_sub), mir/build (both
  the enum arm dispatch AND the sub-pattern match), AND the `tests/`
  selfhost_parser S-expr renderer. Let the compiler find them (2 rounds of
  `cargo build`), don't grep-and-guess. Skipped rewriting the WASM interp's
  opcode dispatch: its `if/else-if` chains mix ranges (`op >= 0x28 && op <=
  0x3e`) and OR-combined tests that literal-only match can't express, and the
  run/ copy must stay byte-identical to the drift-guard source — disproportionate
  risk for no functional gain. (2026-07-14).

- **Adding `f32` to complete the float family (mirror `f64`) needs NO new width
  tag — the `ScalarTy` enum already distinguishes the two, and it is carried at
  every op site.** `f32` is 4 bytes / align 4, carried as its `to_bits()` 32-bit
  pattern ZERO-extended in the same i128/i64 register model (`ty_range(f32)` =
  unsigned-32, so no path sign-extends it). The register/load-store machinery is
  fully parametric over `ty_range` + `Layout::scalar_size`, so `f32` "just works"
  once those two return `(32,false)` / `4`; only the float decode/encode branches
  (which bit-cast to a native single) plus the literal grammar are genuinely new.
  `Operand::Const(_, ScalarTy)` and the `ty: ScalarTy` on MIR `Bin`/`Un`/`Conv`
  already carry the width; `Cmp` recovers it from its operands (both the same float
  type, checker-guaranteed). Add a `ty: ScalarTy` field to `ast::FloatLit` + the
  real `Float` token for the literal width; the Conv wire format stays UNCHANGED
  (recover source/target from operand/`to`) — do NOT add a `from` field, it breaks
  self-host MIR byte-parity (same trap the f64 slice hit). Literal grammar chosen:
  an `f32` *suffix* on a FLOAT-form literal (`1.5f32`); `10f32` (integer form) and
  any non-`f32` suffix are lex errors — keeps the integer-suffix space integer-only.
  Parametrize the interps with free `float_arith`/`float_cmp`/`float_neg`/
  `float_conv(bits, from, to)` helpers over the width instead of duplicating the
  f64 branches. THE TRAP: the tree-walker `Eq`/`Ne` arm computes an `equal` bool the
  caller flips for `!=` (`res = if Eq {equal} else {!equal}`); feeding
  `float_cmp(op,…)` (which returns the `!=` result for `Ne`) into `equal`
  DOUBLE-NEGATES `!=`. Pass `BinOp::Eq` to get equality; the ordered branch
  (Lt/Le/Gt/Ge, assigned straight to `res`) takes `op` directly and is fine. The
  MIR Cmp (takes `op` directly) was already correct, so the f64-oracle `!=`
  regression pinpointed the bug. Backend bit-casts differ by width: Cranelift
  `ireduce I32`+`bitcast F32` / `bitcast I32`+`uextend`; LLVM `trunc`+`bitcast
  float` / `bitcast`+`zext`; `fpromote`/`fdemote` (Cranelift) = `fpext`/`fptrunc`
  (LLVM) for f32↔f64. `mir/opt.rs` is DCE-only (type-agnostic) — nothing to do.
  New `tests/fixtures/run/floats_f32.cnr` auto-joins the aot/stage native corpus
  gates for free. Full nextest 808 green, clippy clean. (2026-07-14).

- **WASM-interp differential over floats: run the big value matrices on the
  tree-walker ORACLE only; keep cross-engine (all 4) to a small hand-picked set.**
  The interp is a ~1400-line Candor program; running ONE module through the native
  (Cranelift no-opt + -O2) engines recompiles the whole interp, so a full-matrix
  test that called `run_ret_all` (4 engines) per module hit **1246 s** for the
  `wasm` binary. The M3 corpus already established the split: large differential
  matrices vs wasmi on the oracle; a separate compact `*_cross_engine_agreement`
  test proves byte-identical across tree-walker/MIR/Cranelift. Float NaN needs
  NaN-aware compare everywhere (non-NaN bit-exact, NaN by is-nan — a computed NaN's
  sign is IEEE-unspecified across a folding compiler vs. runtime). Also trim compare
  matrices (6 ops x 9x9 x 2 widths in one test = ~180 s of tree-walking). (2026-07-14)

- **WASM float trunc TRAPS where Candor's `conv` SATURATES — range-check in the
  float domain against exact representable bounds, don't trust `conv`.** WASM MVP
  `i{N}.trunc_f*_s/u` trap on NaN/out-of-range; Candor `conv i{N} <float>` saturates
  (0016 section 5). Derive bounds as powers of two so f32 representability is exact:
  f64->i32_s traps iff `x >= 2^31 || x <= -(2^31+1)`; f32->i32_s iff `x >= 2^31 ||
  x < -2^31` (f32 has no value between -2^31 and -2^31-1). All four rounding ops
  are bit-exact without an intrinsic (trunc via in-range int round-trip + copysign;
  nearest via the add/sub-2^52 magic under IEEE round-nearest-even); only `sqrt`
  genuinely needs a `sqrt` intrinsic and is deferred. A bare int literal on the
  bitcast side takes the float's same-width UNSIGNED int, and high-bit patterns
  (e.g. `0x8000000000000000`) need an explicit `u64` suffix. (2026-07-14)
