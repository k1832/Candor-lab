# 0014 — The self-hosted interpreter: Candor executing Candor

**Status:** draft (records a reached milestone — the public record of a completed
artifact, per philosophy §8.6's "document what was built and why").
**Date:** 2026-07-11
**Philosophy hooks:** P19/Bet 6 (dogfooding as the correctness signal — the
self-hosting arc exists to produce it); P6 (the strict per-slice MVP subset —
scope is a budget, added only when a fixture forces it); P4/P7 (the observable
semantics are `Run{ret,trace}` and fault *identity*, not internal state); P5
(precise faults — kind + span, reproduced byte-exact); P9 (the allocator-explicit
ABI the interpreter must honour structurally); P2/NN#17 (nothing crosses a
signature by inference — the type/layout table is annotation-directed, no
inference). Builds on 0001 (the memory model, §1.5 drop order, §4.2 memory, §5–§8
the interpreter semantics), 0010 §5 (the four-engine equivalence gate this rides),
0013 (`str`/`[u8]` and the collection stage plan that S7 would need). Mirrors the
Rust reference interpreter `prototype/src/interp/{mod,eval,layout,mem}.rs`.
**Prototype:** `prototype/selfhost/interp/interp.cnr` (~3100 lines of Candor),
composed with the `lexer` and `parser` self-host modules by named `use`, gated by
`prototype/tests/selfhost_interp.rs` (execution equality vs `run_source_real`) over
a 71-fixture corpus (61 returns, 10 faults) that includes all five systems-corpus
programs (11_1..11_5). Slices S1–S6b are recorded in chapter 99 (the obligations
tracker) as the authoritative slice-by-slice ledger; this document is their
synthesis. The std/generic tail (S7–S9) is deferred and cleanly separable (§7).

## 1. Purpose and place in the self-hosting arc

Self-*interpreting* is the tier where a Candor program **executes** Candor
programs. `interp.cnr` is a tree-walking interpreter, written in Candor, that
walks the self-hosted parser's `Node` arena and produces the same observable
result — `main`'s return value, the ordered `trace` log, and any precise fault —
as the Rust reference interpreter it mirrors.

This is the third tier of the arc, and its position is deliberate:

- **Self-lexing / self-parsing** proved Candor can *recognize* its own syntax.
- **Self-checking** (the lexer/parser/checker/analyses modules checking their own
  source, oracle-matched — chapter 99, the fixpoint gates) proved Candor can
  *reason about* its own static semantics: name resolution, move/init, loans,
  effects.
- **Self-interpreting** proves Candor can express its own *dynamic* semantics —
  the runtime meaning of the language: arithmetic and its overflow faults, the
  flat byte-memory model, struct/array/enum layout, the move/drop schedule with
  observable trace-on-drop, `match`, the `box`/allocator ABI, raw pointers and
  MMIO. It is the tier *after* self-checking (which decides whether a program is
  well-formed) and *before* self-lowering and self-compiling (which would turn a
  well-formed program into MIR, then native code — §8).

The credibility claim is narrow and strong: **Candor can state its own execution
semantics, in Candor, precisely enough that the statement is behaviourally
indistinguishable from the reference.** A language whose runtime meaning can only
be pinned down in another language has not fully specified itself; this artifact
closes that gap for the whole systems-programming surface. It is also the
project's densest dogfood to date — writing an interpreter exercises nearly every
language feature at once (aggregates, pointers, unsafe, the drop schedule), and
§6 records what that pressure flushed out.

## 2. The differential methodology: what correctness *is*

Correctness here is not "passes a hand-written expectation." It is defined
**differentially**: for every fixture in the corpus, the observable dump produced
by running the program *through `interp.cnr`* must be **byte-identical** to the
dump produced by running the same program through the Rust reference interpreter
(`run_source_real`). The gate lives in `prototype/tests/selfhost_interp.rs`
(`candor_interp_execution_equal_to_oracle_over_corpus`) and asserts
`assert_eq!(mine, oracle)` per fixture.

### 2.1 The observable schema (RET / TRACE / FAULT)

The dump is a canonical, whitespace-lean rendering of the two things 0001 makes
observable — `Run{ret, trace}` (mod.rs:98–104) and a precise `Fault{kind, span}`
(mod.rs:60–95):

- a successful run emits `RET <i64>` followed by one `TRACE <i64>` line per traced
  value, in drop/trace order (`interp.cnr` §`interp_dump`, lines 3085–3096);
- a faulting run emits a single `FAULT <kindcode> <p0> <p1>` line — the fault's
  **kind** and its **source span** (lines 3074–3083).

Nothing else is observable. Absolute addresses, the stack-bump cursor, page-pool
slots, fn-ptr encodings — all are free implementation choices, precisely because
they never surface in RET/TRACE/FAULT. This is what lets `interp.cnr` diverge
wildly from the oracle internally (a 128-frame paged store vs the oracle's 256 MiB
dense space; a fn-ptr encoded as an arena node id vs the oracle's `fn_id_of`
index) while staying byte-exact on the dump.

Fault *identity* — not merely "it faulted" — is the sharp part. The six shared
fault codes (`FK_OVERFLOW=0`, `FK_DIVZERO=1`, `FK_ASSERT=2`, `FK_PANIC=3`,
`FK_BOUNDS=4`, `FK_CONVLOSS=5`, interp.cnr:33–38, mapped from the oracle's
`FaultKind`) plus the exact source span mean an off-by-one in *which leaf* an
`assert` blames, or *which token* an overflow reports, is a hard test failure.
The interpreter therefore threads a running `cur_p0/cur_p1` span through every
evaluation step to reproduce the oracle's `span_from(lo)` composite spans exactly.

### 2.2 Riding the four-engine equivalence

The reference interpreter is not just *an* implementation; it is the anchor of the
prototype's **four-engine equivalence gate** (design 0010 §5): the tree-walking
interpreter (`run_source_real`), the MIR interpreter (Stage A), the native
Cranelift JIT (Stage B), and the optimized native engine (Stage D) are all held to
semantic-trace equality on every corpus program (`prototype/src/lib.rs`, "the
four-engine gate proves the trace survives," lib.rs:333). The tree-walker is the
reference the other three are proven equal to.

The consequence is load-bearing for the credibility claim: **matching the
tree-walker is, transitively, matching native execution.** `interp.cnr` needs to
be differentially tested against only *one* engine (the cheapest, most direct one),
and the existing gate guarantees that agreement with the tree-walker is agreement
with the optimized native backend. The self-interpreter did not have to re-prove
equivalence across engines; it inherited it.

### 2.3 The self-interpreter is gated on execution, not self-checked

One honest boundary, restated in every slice: **`interp.cnr` is gated on its
EXECUTION behaviour, not run through the self-host checker.** It uses the *full*
language (including `unsafe`, raw-pointer intrinsics, and constructs the self-host
checker does not yet cover) precisely because it is a program the *oracle* runs,
not one the self-host front-end must accept. Self-checking `interp.cnr` itself is a
separate, later concern (§8) — the same staging the earlier self-host modules used.

## 3. The value model: from tagged scalars to flat paged memory

The single most consequential design movement across the slices is the
**convergence of the value model onto flat, addressed byte memory** — and it was
deliberately *not* done up front.

- **S1 (scalars).** A value is a tagged scalar: one `i64` bit-slot plus a **width
  code** (1=i8..5=isize, 6=u8..10=usize, 11=bool). No memory. Width propagates
  from `let` annotations, literal suffixes, and fn return types the way the oracle
  threads `expected` (`lit_width`, interp.cnr:539). This was enough for the whole
  scalar/control-flow/arithmetic subset and let S1 land without any layout table.

- **S2 (flat memory).** The scalar model **converges onto memory**: every local,
  parameter, and aggregate now lives at a bump-allocated **address**, and a scalar
  is stored/loaded by `(address, width)` via little-endian `mem_store`/`mem_load`
  (interp.cnr:580–616), sign/zero-extended to mirror the oracle's `read_int`. A
  scalar value still flows through the `cur_val`/`cur_w` registers byte-exact; an
  **aggregate value is carried as an address** — `cur_w == 0` marks it, `cur_val`
  *is* the address, and `cur_ty` its type node (the comment at interp.cnr:1954–1956
  states this register discipline). The faithfulness invariant — all 24 S1 fixtures
  pass byte-exact *through* the new model — is what made the convergence safe.

- **S6a (paged memory).** The flat arena becomes a **paged backing store**:
  `pages: [524288]u8` (128 × 4096-byte frames), a `pagedir: [8192]i32` mapping a
  12-bit page number to a pool slot (sentinel −1 = untouched), and a `page_bump`
  (interp.cnr:75–78). `xlate` (interp.cnr:560) maps a linear address to a backing
  offset — `page = addr >> 12`, `off = addr & 4095` — binding and **zeroing** a
  frame on first touch. Every byte routes through `xlate` independently, so a
  load/store/copy that straddles a page boundary is correct (§4).

**Why flat memory was necessary, and why paged over dense.** A tagged-value
interpreter cannot express the language's pointer surface: `rawptr` is a `u64`
address into a real memory (0001 §5, mod.rs memory-model header), `box` hands back
an address plus its owning `Alloc` handle, and MMIO reads a *fixed* address. None
of that has meaning without addressable memory. So flat memory was not an
optimization — it was the enabling substrate for S5/S6. Paging, in turn, was
forced by a subtle interpreter-over-interpreter cost: the systems corpus uses fixed
addresses up to ~16.9 MiB, and a *dense* `[N]u8` of that size is memory-feasible
but **initialization-time-infeasible** — the oracle running `interp.cnr` would
have to execute an `[N]u8` array-repeat with a guarded per-byte move (~18M
iterations under the tree-walker) just to zero it. Paging allocates and zeroes only
*touched* pages, keeping `E` small (a 512 KiB pool) while addressing a sparse
32 MiB space. **Zero-on-page-alloc is itself load-bearing:** a whole-value copy of
a padded enum reads the variant's unwritten tail, which must read 0 to match the
oracle's initialized-byte guard (interp.cnr:559 comment).

## 4. Architecture

### 4.1 One region-free state struct, threaded

All interpreter state lives in a single `struct E` (interp.cnr:54–116), threaded
through every function as `(read P arena, [u8] src, write E)`. This is the same
context-tuple shape the self-host checker and analyses settled on (chapter 99's
2-read-view floor: the parser's `Node` arena and the `src` bytes that 0001 §3.4
forbids as a struct field, plus one mutable aggregate). Folding *all* mutable
runtime state into one owned struct — locals, the paged store, the move-mask, the
temp stack, statics, the drop registers — is what keeps the thread at three views:
`E` holds fixed owned arrays (`loc_*[1024]`, `pages[524288]`, `mv_*[512]`,
`tmp_*[256]`, `static_*[64]`), which structs may hold, so no additional view is
threaded. The model is region-free by construction — no borrow-typed field anywhere
(0001 §3.4), the accepted value-first cost the whole self-host source pays.

### 4.2 The three dispatchers over the arena's `T_*` tags

The interpreter is three mutually-recursive dispatchers over the parser's arena-tag
`Node`s, each an `if nd.tag == T_X` ladder (the language has no integer `match`, so
tag dispatch is a ladder — chapter 99's recorded parser-slice friction):

- **`eval_expr(pp, src, e, i, exp_w) -> i32`** (interp.cnr:1957) — evaluates an
  expression node, leaving a scalar in `cur_val`/`cur_w` or an aggregate address in
  `cur_val` (`cur_w == 0`). `exp_w` carries the expected width down (the oracle's
  `expected` threading).
- **`exec_stmt`/`exec_block`** (interp.cnr:2785/2886) — execute statements and
  blocks, owning the scope-exit drop schedule (§4.4).
- **`eval_place(pp, src, e, i) -> i32`** (interp.cnr:1458) — resolves an
  l-value to `(plc_addr, plc_w, plc_ty)`: locals, then statics; struct fields via
  the layout table; array/slice indices with the `Bounds` fault; and `.*` deref,
  auto-peeling `Box`/borrow layers (`peel_box_place`, interp.cnr:1443).

`call_fn` (interp.cnr:2691) runs a function: arguments arrive in `E.arg_*`,
by-value params are copied into fresh param slots, an aggregate return is delivered
through a caller-owned `ret_slot` reserved *below* the frame's bump-reset point so
it survives. Recursion rides `interp.cnr`'s **own** Candor call stack — each
`call_fn` frame saves and restores `loc_n`, `cur_base`, and `stack_bump`.

### 4.3 Status-code control flow (0/1/2/3/4)

There is no exception mechanism, so control flow is carried by an **`i32` status
code** returned from every dispatcher (interp.cnr:1954): **0** normal, **1** fault,
**2** return, **3** break, **4** continue. Each dispatcher inspects the code from
its callee and either propagates it or acts on it — a loop turns 3 into a normal
exit and swallows 4; `call_fn` turns 2 into a delivered return value; **1 aborts
all the way out**. This last is the abort-no-drop rule (§4.4).

### 4.4 The drop schedule: a static fact, recomputed during the walk

The observable heart of the language is the **drop/trace schedule**, and its design
mirrors the oracle's key decision (mod.rs:36–45): because 0001 §1.6 forbids
conditional move divergence and proves move state agrees at every join, the set of
live drop obligations at each scope/statement exit is **path-independent — a static
fact**. The interpreter therefore carries *no runtime drop flag*. It **recomputes
ownership as it walks** (it does not read the analyses pass's move facts) and, at
each block/statement/param-scope exit, drops exactly the statically-owned,
not-moved, needs-drop places in 0001 §1.5 reverse/LIFO order.

The machinery: a per-local `loc_owns` flag; a flat **move-mask** side table
(`mv_local`/`mv_field`, field −1 = whole-local moved) covering whole-value
suppression and one-level partial moves; and a **non-copy temp stack** (`tmp_*`)
for mid-statement values. `is_copy`/`needs_drop` (interp.cnr:830/868) are ported
from `src/types.rs`, and **every** drop-schedule addition is gated behind them, so
scalars and `copy` aggregates are drop-inert and byte-identical to the pre-drop
slices. `drop_value` (interp.cnr:1192) runs a struct's hook first (whole value,
unless partially moved), then fields in reverse; `run_drop_hook` (interp.cnr:1107)
binds `self` to the value's address and executes the user hook block, so
`trace(self.id)` fires through the ordinary field-read path into the same
`trace_out` sink. **Abort semantics** (0001, eval.rs:3183): a status-1 fault
returns *without* running any scope/temp/param drops — every drop loop is guarded
by `if st != 1`.

## 5. The slices: scope, mechanism, and the decisions

Each slice was solo and serial (it writes the crate), scoped strictly under P6 —
added only what a fixture forced. The scope/mechanism table below is a synthesis;
chapter 99's S1–S6b entries are the authoritative record.

### S1 — scalar tree-walker
**Scope:** integer + bool scalars, `let`/assignment, `if`/`while`/`loop`/`break`/
`continue`/`return`, `+ - * / %` with Overflow/DivByZero, comparisons, `&&`/`||`
short-circuit, bitwise + shifts, unary, `trace`/`assert`/`panic`, direct calls with
recursion. 24 fixtures.
**Decision — overflow without `i128` (wrapping-then-decide).** The language has no
`i128`, but the oracle detects overflow by widening to `i128`. Since `interp.cnr`
is not self-checked it uses the full language: compute the raw op inside a
`wrapping { }` block (which cannot fault), then decide overflow *without* a wider
type — signed add/sub by sign logic (interp.cnr:1325–1336), signed mul by a
division re-check with `MIN * -1` special-cased (interp.cnr:1338–1346), unsigned
add by carry (`wrapped < a`), unsigned sub by borrow (`a < b`), unsigned mul by
`wrapped / a != b` (`arith_unsigned`, interp.cnr:1354). u64↔i64 bit reinterpretation
is `wrapping { conv }` — no transmute (`to_u64`/`to_i64b`, interp.cnr:119–128).

### S2 — flat byte-memory + structs and arrays
**Scope:** the flat memory model (§3), a type/layout table, struct literals + field
read/assign + nested structs, struct by-value params/returns, array literals and
repeat, index read/assign, the `Bounds` fault. 36 fixtures.
**Decision — the type/layout table is extracted from the arena AST, annotation-
directed.** A type descriptor *is* an arena node: a scalar keyword is `T_SC` (**not**
`T_NAMED` — the transcription trap that first read every scalar local as an
aggregate address), a user struct is `T_NAMED`, an array is `T_ARRAY`.
`ty_size`/`ty_align` (interp.cnr:698/725) mirror `layout.rs` exactly: scalars by
width, arrays as `round_up(size,align)` stride × len, structs in **declared order**
at natural alignment rounded to the struct's alignment. There is no inference
(NN#17): every layout follows from a written annotation or a struct decl.
**Decision — copy-only, drop-deferred.** S2 stays on `copy` aggregates, so there is
no drop obligation to model yet; `trace` output is purely explicit calls. This kept
S2 orthogonal to the drop schedule (S3).

### S3 — move/drop schedule + trace-on-drop
**Scope:** the drop schedule of §4.4 — reverse/LIFO scope drops, move-suppresses-
drop, one-level partial move, move-out-via-return dropping in the caller, break-path
drops, by-value param drop. 8 fixtures.
**Decision — recompute the schedule during the walk, no runtime flags** (§4.4).
**Decision — one-level partial move only.** The move-mask records a moved field by
index on its direct-local root; a deeper `a.f.g` partial move would need a
path-vector mask (the oracle's `MoveMask`) and is deferred until a fixture needs it.
This is the honest MVP: cover the subset the corpus exercises, not the general case.

### S4 — enums and match
**Scope:** enum values, construction, `match` with variant/wildcard/binding
patterns, tag-directed enum drop. 50 fixtures total.
**Decision — enum layout `{tag: u64 @0, payload @8}`** (interp.cnr:918–950),
mirroring `layout.rs`: payload laid out struct-style from offset 8, enum size
`round_up(8 + max-padded-payload, 8)`, align 8 always, tag = the variant's 0-based
declared index. The payload chain is a raw type-node `nx`-chain, so
`payload_size`/`variant_payload_off` **generalize** S2's struct-field layout over a
chain whose elements *are* the field types — reuse, not a parallel mechanism.
**Decision — owned scrutinees only; non-copy payload bindings alias and mark the
move-mask.** A copy payload is byte-copied to a fresh slot; a non-copy payload is
aliased in place and the scrutinee's payload field is marked moved
(`mv_push(root, i)`), so the scrutinee's later drop skips it. Borrowed/boxed
scrutinee peeling is deferred (matches the checker slice's owned-scrutinee boundary).

### S5 — Box / BoxResult / allocator ABI, and the heap
**Scope (S5a, foundation):** `rawptr`/`fn`-ptr as 8-byte scalars, top-level `static`
evaluation, fn-name-as-value, indirect calls through a fn-ptr, the minimal
raw-pointer surface (`addr_of`, `ptr_read`, `ptr_write`, `is_null`, `cast_ptr`,
`addr_to_ptr`, `ptr_null`). **Scope (S5b, the heap):** `box`/`unbox`, the
compiler-known `BoxResult`, `.*` Box-deref, alloc-on-drop. 60 fixtures total.
**Decision — structural Alloc identification, never by name** (interp.cnr:448–506,
mirroring eval.rs:236–248). The `AllocVtable` is *the struct carrying fn-ptr fields
`alloc` AND `free`*; the `Alloc` handle is *the struct whose `vt` field is a `rawptr`
to that vtable*. Computed once at startup and stored as the ABI seam. This is the
P9 allocator-explicit ABI honoured *structurally* — box/unbox resolve
`ctx`/`vt`/`alloc`/`free` by field offset (`field_off_lit`, name-agnostic,
interp.cnr:792), so the ABI does not depend on how the module tree qualifies the
allocator type names.
**Decision — synthesize `BoxResult` from the S4 enum machinery.** The parser never
emits enum/variant nodes for the compiler-known `BoxResult T`. So at startup
`synth_boxresults` (interp.cnr:240) *appends* a real `{boxed(Box T), oom}` enum to
the arena — a `T_BOX` payload node, two `T_VARIANT` nodes whose name spans point at
the `boxed`/`oom` bytes scanned from source (so a `match` pattern name compares
equal), and a `T_ENUM` linked from the `T_BOXRESULT` node's `.c`. `enum_of_ty`
routes a `T_BOXRESULT` to this synthetic enum, so **`match`, enum-size, and
enum-drop reuse the S4 machinery unchanged** — only `T_BOX` (layout
`{ptr@0, ctx@8, vt@16}`, 24 bytes) needs a new `drop_box` arm. `box`
(`bi_box`, interp.cnr:2350) reads `ctx`/`vt`, sizes the value, indirect-calls
`alloc(ctx,size,align)` through the vtable, and on null takes the `oom` arm
(drop+consume the value, tag 1); else moves the value into the heap slot and builds
the Box (tag 0). Drop order is **pointee-then-free**, load-bearing (interp.cnr:1239).

### S6 — paged memory, pointer intrinsics, and the corpus (the milestone)
**Scope (S6a):** the paged store (§3) and the three intrinsics the corpus needs —
`offsetof(T, field)`, `ptr_offset(p, n)`, `ptr_to_addr(p)`. **Scope (S6b):** the
five systems-corpus programs themselves. 71 fixtures total (61 returns, 10 faults).
**Decision — paged, not dense** (§3): dense is initialization-time-infeasible under
the interpreter-over-interpreter.
**Decision — `conv` mirrors `eval_conv` exactly** (interp.cnr:1897, the #1 S6b
blocker): read the source integer at its declared signedness, range-check against
the *target* scalar's `(min,max)` (unsigned magnitudes via u64), keep the bit
pattern in range, `ConvLoss` fault out of range in the checked regime.
**Decision — borrow parameters get real borrow nodes.** The parser *unwraps* a
`read`/`write` param — inner type in `T_PARAM.a`, borrow kind in `T_PARAM.op` — so
the interp never saw a borrow node and copied the *pointee by value* (§6).
`synth_param_borrows` (interp.cnr:369) rewrites each such param's `.a` to a real
`T_BORROW`/`T_BORROWMUT` node, so it is stored as an 8-byte pointer and `param.*`
derefs correctly.
**Decision — a generic `BoxResult` scaffold** for `box(...)`/`BoxResult::*` used
*without* an annotated `BoxResult T` in scope (`synth_generic_boxresult`,
interp.cnr:275): the boxed inner type rides `e.gen_inner`, set at each `box` from
the value's type (a synthesized `T_SC` for a scalar, a `[N]T` for an array-rep).

**The five, each RET oracle-matched:** `11_1_allocator` (pool free-list, RET 1234),
`11_2_scheduler` (rawptr intrusive list + hand-written `container_of`, RET 42),
`11_3_mmio` (enums/match + fixed-address MMIO, RET 42), `11_4_parser` (recursive-
descent expression parser over a `b"..."` slice, enums with Box payloads, RET 17),
`11_5_arena` (a `Box [4096]Node` arena + recursive `fold_consts`, RET 5).

## 6. What dogfooding caught

Fixtures are written to a mental model; **real programs are written to reality.**
Each real artifact — the self-host source itself, and the five corpus programs —
surfaced a genuine bug that the targeted fixtures had missed. This is the P19/Bet-6
payoff the arc exists to produce, and it is worth recording in full.

- **Borrow-param-by-value: an allocator read its own `ctx` through a *copy*.** The
  parser unwraps a `read`/`write` param, so the interpreter never saw a borrow node
  and stored a `state: write Bump` parameter as a *by-value copy of the `Bump`
  struct* rather than an 8-byte pointer. The `mk_alloc`/`pool_handle` code set
  `ctx = &st`, but `st` was the copy — so the allocator threaded a pointer into a
  stack frame that had already been reset. No small fixture had a borrow-typed
  aggregate parameter feeding an allocator; the 11_1 allocator did. Fixed by
  `synth_param_borrows` giving every borrow param a real `T_BORROW`/`T_BORROWMUT`
  node (S6b; interp.cnr:369).

- **The `ret_val`/alloc-on-drop register clobber.** Because `free` is a *real
  indirect call* executed *during* a drop, it overwrites the shared
  `ret_val`/`ret_w` registers with the freed function's unit return. A
  `return v` inside a `match` arm whose `Box` drops on the way out therefore lost
  `v` — the register held the freed callee's return, not the program's. No fixture
  before S5b combined a value-returning path with an alloc-on-drop on the same exit.
  Fixed by saving/restoring `ret_val`/`ret_w` (alongside `cur_val`/`cur_w`/`cur_ty`)
  around **every** drop loop — `eval_match`, `exec_block`, `call_fn`, `exec_stmt`
  temps (interp.cnr:1231–1237, and the save/restore blocks at 1850–1867, 2751–2771).

- **The literal static-region leak (a latent memory-safety bug in the *reference*
  interpreter).** Surfaced not by the self-interpreter but by running the
  *literal-heavy* self-host source (`analyses.cnr`) through the oracle during the
  self-*check* dogfood: the oracle's `str` literal handling `static_alloc`'d fresh
  storage on **every** literal evaluation and never reclaimed it, so static storage
  grew past `STACK_BASE` and overwrote `main`'s embedded `src[]` mid-run
  (`src[9173]` flipped). Presented as "142 spurious diagnostics past byte 22600,"
  it was not a checker gap at all — it was a memory-model leak in the reference the
  self-interpreter mirrors. Fixed with content-addressed literal interning
  (immutable literals dedup); four-engine equivalence stayed green (commit
  `fde0a92`). A latent safety bug that only a literal-dense real program could flush.

- **The array-copy misclassification.** Two flavours, both from real source. (a) In
  the self-host `ty_copy`, `T_ARRAY` copyability recursed on the *size child*
  instead of the *element type*, so `[256]i32` was wrongly classed non-copy (also
  `fde0a92`). (b) In S6a, an untyped array-repeat element `[0 - 1i32; N]`
  mis-types the element in the oracle and trips its initialized-byte guard;
  `[0i32 - 1i32; N]` types cleanly (used for the `pagedir` sentinel fill). Both are
  the same lesson: element-type classification of aggregates is easy to get subtly
  wrong, and only an aggregate-dense real program exercises the corner.

The through-line: **fixtures test the model you have; corpus programs test the
model you *forgot you assumed*.** Every one of these was a mismatch the differential
gate turned into a hard, localized failure — the methodology's payoff.

## 7. Scope boundary: what runs, what is deferred

**What runs (the milestone).** The self-interpreter executes the entire
**systems-programming surface**: scalars and control flow, flat and paged byte
memory, structs/arrays/enums, the move/drop schedule with trace-on-drop, `match`,
the `Box`/allocator ABI (fn-ptr indirect calls, structural `Alloc` discovery),
raw pointers, MMIO, and `conv`. Concretely: all five systems-corpus programs
(11_1..11_5) plus 66 targeted fixtures, 71 total, each byte-exact against the
reference.

**What is deferred (the std/generic tail).** Three slices remain, and none is
needed by the systems corpus:

- **S7** — slices/`str` beyond the corpus's use + the std collections `Vec`/`Map`/
  `String` (fat pointers = 16 bytes; `Vec`/`Map` = 40).
- **S8** — the **monomorphizer** (chapter 99's blocker B, ~700–900 lines). This is
  the substantial one: user-defined generics require instantiation before the
  tree-walker sees a monomorphic program.
- **S9** — the `conv`-family / contracts close-out (`conv` itself landed early, in
  S6b).

**Why the tail is cleanly separable, precisely.** Chapter 99's S6b entry confirms
it empirically: **every one of the five systems programs is monomorphic** and lands
entirely within the S1–S6 value model plus S6b's additions — *no generics, no
monomorphization, no `for`/iterators, no `Vec`/`Map`, no `?`* were required. The
deferred tail is therefore not a gap *in* the milestone; it is a *different*
milestone (a self-hosted standard library), gated on machinery (the monomorphizer)
the systems corpus structurally does not touch. The boundary is honest: the claim
is "the self-interpreter runs Candor's systems-programming corpus byte-exact," not
"the self-interpreter runs all of Candor." The std/generic surface is future work,
and S8 in particular is independently descopable.

## 8. Forward look

The interpreter tier has proven the language can express its own **execution**
semantics. The horizon (not a plan — a direction) has two branches, recorded in
ROADMAP's post-milestone fork:

- **Continue the interpreter's S7–S9 tail** — a self-hosted standard library, whose
  tall pole is the S8 monomorphizer.
- **Pivot to the next self-hosting tier: self-lowering to MIR.** A Candor program
  that lowers Candor to the mid-level IR, with the **Rust MIR interpreter** (Stage A,
  design 0010 §5) as its oracle — the same differential method, one tier up. Where
  self-interpreting proves the language can express *execution*, self-lowering would
  prove it can express *compilation*, and is the stepping stone toward a
  self-compiler and a true native bootstrap.

Independently, **self-checking `interp.cnr` itself** remains open. Today it is gated
on execution and uses the full language deliberately (§2.3); bringing it under the
self-host checker would require the checker to cover the constructs `interp.cnr`
uses that the current self-host corpus does not (broader value-type diagnostics, the
full pointer/unsafe surface) — the same coverage-extension staging the earlier
modules followed. It is on the horizon, not the critical path.

## Rejected / deferred alternatives

Per philosophy §8.6, the record includes what was rejected and why.

- **A dense `[N]u8` memory instead of paging — rejected (S6a).** Memory-feasible
  (~17 MiB) but initialization-time-infeasible under the interpreter-over-
  interpreter: zeroing it is an ~18M-iteration guarded per-byte move under the
  tree-walker. Paging touches only used pages. (§3.)
- **A tagged-value model carried all the way — rejected at S2.** It cannot express
  pointers, `box`, or MMIO, which need real addresses. The convergence onto flat
  memory was the enabling substrate, not an optimization. (§3.)
- **Reading the analyses pass's move facts to drive drops — rejected (S3).** The
  interpreter recomputes ownership during the walk, because the drop schedule is a
  path-independent static fact (0001 §1.6); a runtime drop flag would be redundant
  state the semantics forbid needing. (§4.4.)
- **A parallel `BoxResult` implementation — rejected (S5b).** `BoxResult` is
  synthesized as an ordinary `{boxed(Box T), oom}` enum appended to the arena, so
  `match`/size/drop reuse the S4 enum machinery unchanged. Minting separate
  machinery would have duplicated the enum path. (§5, S5.)
- **Identifying `Alloc`/`AllocVtable` by name — rejected (S5a).** They are found
  *structurally* (the `alloc`+`free` fn-ptr fields; the `vt`-`rawptr` field), so the
  ABI is robust to however a module tree qualifies the names, mirroring the oracle.
- **A general path-vector move-mask — deferred (S3).** One-level partial move covers
  the corpus; deeper `a.f.g` partial moves wait for a fixture that needs them.
- **Self-checking `interp.cnr` under the self-host gates — deferred (§2.3, §8).**
  It uses the full language as an oracle-run program; self-checking it is a separate
  later concern.
- **The std/generic tail (S7–S9), the monomorphizer especially — deferred (§7).**
  The systems corpus is entirely monomorphic; the tail is a distinct milestone.
