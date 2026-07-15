# Candor roadmap

Rulings by the deciding authority (2026-07-09) on publication and self-hosting; the maturation
queue below is ordered, not dated.

## Publication staging

1. **This repository is the lab, permanently.** The full experiment record — philosophy,
   adversarial reviews, the frozen Bet 5 experiment, designs, gates — is the project's
   distinctive asset and is never diluted into a product repo.
2. **The 0.x preview ships from a separate distribution repository** created at the packaging
   milestone: the toolchain (renamed `candor`), spec, stdlib, editor support, getting-started.
   Explicitly unstable; this repo linked as the design record.
3. **1.0 is the stability gate, not a date:** P15's edition/migrator promises live, P20's
   pre-registered compile-time targets ratified in CI, the spec's obligations ledger clear of
   pre-stability items (NN#20 mechanization decision included). NN#14's Bet 5 condition is
   already satisfied.

## Self-hosting

The compiler is Rust and remains the bootstrap/reference implementation permanently. A
self-hosted compiler is the project's ultimate dogfood (a compiler is the basket's own home
ground), its largest P19 corpus, and its credibility proof — gated on std, not the language:

1. The P3 text-type budget design (the named deferred obligation).
2. An I/O layer as a boundary module over libc (what the P17 boundary exists for).
3. Then port the CHECKER first — highest value, its own domain — with the Rust implementation
   as the differential oracle, per the house methodology.

## Maturation queue (ordered)

- Graduation-tier eval campaign (the first slope-capable measurement).
- Toolchain packaging: candor-proto → candor, install story, the distribution repo (publication
  step 2 above).
  - **Repo-layout rename (do it here, one commit, while the tree is quiescent).** `prototype/`
    is a historical misnomer — it began as the Bet-5 throwaway measurement prototype but is now
    the production reference compiler (README already calls it "the production toolchain"). Rename
    `prototype/` → `compiler/` (crate/binary already renamed `candor-proto → candor` above), and
    hoist the self-hosted compiler to a top-level `selfhost/` (currently `prototype/selfhost/`) so
    the Rust reference compiler and the Candor self-host are visibly peers, not one nested in the
    other. Pure churn touching every `prototype/...` path reference (docs, tests, nextest
    invocations, `dist/`, PROGRAMS_IN_CANDOR.md) — so serialize it SOLO, after the packaging
    implementation work settles and outside any compiler-crate agent's window. No build-output
    change (NN#16 reproducibility unaffected: a path rename doesn't alter emitted artifacts).
- Text-type budget design (P3's named obligation; gates self-hosting and real std growth).
- I/O boundary module (gates self-hosting).
- Bare-metal target (blocked locally on qemu; the freestanding proof stands meanwhile).
- LLVM second backend behind the MIR seam and differential gate.
- Corpus scale-up; per-edition regeneration.
- Self-hosted checker (after text + I/O).
- Stability-gate proceedings (1.0) when the checklist clears.

## Self-hosting arc — STARTED 2026-07-09

Ruling: port onto the interpreter first (AOT extern lowering deferred; a self-hosted checker runs
on the tree-walker, which is a validated engine). Order and gates:
1. **Lexer + parser** (this slice): a Candor .cnr program that lexes and parses Candor source to
   a canonical AST S-expression, gated by S-expr equality against the Rust front-end over the
   whole corpus (the differential methodology; the Rust parser is the oracle).
2. Type checker for the scalar+aggregate core, oracle = the Rust checker's diagnostics.
3. The analyses (move/init, loans, effects), each oracle-gated.
4. Generics, concurrency, text — the frontier.
The Rust implementation remains the permanent bootstrap and oracle. Each slice is a driver: the
Candor port reads source via the std/io module, emits its result, the harness diffs it.

## Self-hosting target RULED: SELF-CHECKING, 2026-07-10

Deciding authority chose the self-checking tier: the Candor-written checker checks its own
source, oracle-matched, culminating in the fixpoint gate — run checker.cnr / analyses.cnr on the
self-host .cnr compiler source and assert the diagnostic set equals the Rust oracle's.
Self-interpreting and true native bootstrap are named as the horizon, NOT targeted now (they need
the str/collection runtime and a backend ported to Candor — prototype-stretching).

**Honesty boundary:** the self-hosted compiler MAY depend on a small set of compiler-known
primitives (Vec, str, Alloc, CharStep) — the runtime/language split every real language has
(cf. Rust core intrinsics), not cheating. Self-checking does not require them in-language.

**Path (targeted enablers pulled by demand, not a speculative std suite):**
1. A std hash map / symbol table — name resolution (item tables, scopes, the keyword ladder) is
   the checker's hot path, currently a linear scan; every remaining slice consumes it.
2. Richer AST spans in the self-host parser — the REAL blocker: the span-lean arena defers
   composite-span diagnostics (E0302/E0309/E0803/E0809); fuller spans unblock matching the oracle
   on all codes.
3. Extend self-host checker coverage to the feature set the self-host SOURCE uses (only that).
4. The fixpoint gate: the self-host checker checks the self-host source, oracle-matched.

**Fixpoint scoping RESULT (2026-07-10):** the gate is ~a handful of small slices, not N. The
self-host source is structurally clean where it matters (zero persistent loans; no
box/unbox/clone/match/enums), so the deferred error codes (E0302/E0309/E0809/E0401/E0601) are
UNREACHABLE over it — out of scope for self-checking (they matter only for negative error
fixtures). The gate reduces to "the checker traverses the whole source without false-rejecting,
oracle-matched (∅)." First sub-goal DONE: the self-host checker checks lexer.cnr clean (leaf
module, arena growth only).

**Fixpoint path RULED: HONEST import-resolution (deciding authority, 2026-07-10)** — over the
pragmatic concatenate-into-one-blob alternative. Each module is checked IN ISOLATION with its
`use` imports resolved, faithful to the module tree we built (not a concatenated blob) and with a
smaller max input (parser.cnr ~20.7k tokens vs a ~42k blob → better tree-walker perf). Cost: a
real feature (use/pub in the self-host lexer/parser/checker) plus possibly adopting the std Map
into the checker's currently-linear name scan for runtime. Slice order: import-resolution (the
gate) → per-module arena growth → per-module clean gates (parser/checker/analyses) →
analyses-clean (E0301/E0304, empirically inspected for move/init false positives) → Map-in-checker
perf if needed.

**Scheduled — modularize the self-host source (dogfood the module system).** The four
self-host `.cnr` files use zero `use` imports; the oracle harness composes them by
`include_str!` concatenation (convenience, not a language limit — modules stage 1 works,
proven by `tests/fixtures/modules/ok_tree/`). The self-host compiler is the one substantial
Candor program that does NOT exercise modules. Split it into a proper `use`/`pub` tree
(lexer / parser / checker / analyses as real modules sharing `Tok`/`Node`/state via named
imports), teaching the oracle harness to load a module tree instead of concatenating. Expected
to surface real stage-1 module-ergonomics obligations (cross-module type refs need named
imports not alias paths; no `pub use` re-exports) — that friction is valuable signal, logged to
99-obligations.md. Serialized: touches `selfhost/` + every oracle harness, so it runs SOLO,
after the Map enabler and outside any compiler-crate agent's window. Slot: after step 1 (Map)
lands, before the step-3 coverage slices — a split, better-navigable source eases those.

## Self-interpreting — sliced plan of record (2026-07-10)

The next self-hosting tier: a tree-walking interpreter written in Candor
(`selfhost/interp/interp.cnr`) that EXECUTES Candor programs, oracle-matched
(byte-exact `Run{ret,trace}` + fault identity) against the Rust `src/interp/`.
Built in slices; each is solo/serial (writes the crate). Scoped by two read-only
design passes (S3 drop semantics; blockers A/B/C).

- **S1 DONE** — scalar interp (ints/bool, control flow, arithmetic+faults, calls,
  recursion). Overflow without i128 via wrapping-then-decide. 24 fixtures.
- **S2 DONE** — flat byte-memory model + structs/arrays. Value model converged onto
  addressed memory (S1 fixtures still byte-exact). 36 fixtures.
- **S3** — move/drop schedule + trace-on-drop. DOABLE BEFORE HEAP: the drop trace
  comes from a user `drop(write self)` hook calling `trace`, no Box needed. Reverse/
  LIFO order, hook-then-fields, move-suppresses-drop, fault aborts without dropping.
  Recomputes ownership during the walk (not from analyses). Extends the type table
  with is_copy/needs_drop + struct->drophook lookup. 8 fixtures to author.
- **S4** — match/enums (RESEQUENCED before Box: every Box systems fixture matches
  BoxResult). Tag@0/payload@8.
- **S5** — Box/BoxResult + allocator ABI ({ptr,ctx,vt}=24, structural Alloc discovery).
- **S6** — rawptr/fnptr/MMIO + pointer intrinsics + the dense [N]u8 Mem (blocker C,
  ~80 lines, 4-8 MiB, no paging — corpus max address is 3 MiB).
  **MILESTONE: after S6 all five systems-corpus programs run** (11_1..11_5
  allocator/scheduler/mmio/parser/arena) — the non-generic, non-container corpus.
- **S7** — slices/str + std Vec/Map/String (fat pointers=16, Vec/Map=40).
- **S8** — the monomorphizer (blocker B, ~700-900 lines). Gates ONLY the generic
  library tail; the systems corpus is monomorphic, so B is descopable indefinitely.
- **S9** — conv/contracts + whole-corpus close-out.

**Blocker verdicts (design memo):** A (type/layout table) is the TALLEST POLE —
the self-host checker does no type inference, so A must BUILD annotation-directed
type synthesis (`ty_of`), ~1000 lines grown incrementally across S2-S7, with no
oracle to differentially test against. B (monomorphizer) is late and optional.
C (address space) is nearly free (dense arena, no paging). ~6 slices / ~3500-4500
lines of Candor remain to the whole corpus; the systems-corpus milestone (S6) is
much nearer.

### Self-interpreting MILESTONE REACHED (2026-07-11)

S1-S6 done: the self-hosted interpreter runs the entire systems-heavy corpus
(11_1 allocator / 11_2 scheduler / 11_3 mmio / 11_4 parser / 11_5 arena) byte-exact
against the Rust oracle, plus 66 targeted fixtures — 71 total, covering scalars,
flat+paged memory, structs/arrays, move/drop with trace-on-drop, enums/match,
Box/allocator ABI (fn-ptr indirect calls, structural Alloc), rawptr/MMIO/conv.
Confirmed nothing in the systems corpus needs S7+ (all monomorphic). Each real
program surfaced and fixed a genuine interp bug (borrow-param-by-value, the
ret-register/alloc-on-drop clobber, the literal static-region leak, etc.).

Remaining self-interpret tail (the generic library, NOT gated by the milestone):
- **S7** — slices/str + std Vec/Map/String (fat pointers=16, Vec/Map=40).
- **S8** — the monomorphizer (blocker B); gates ONLY generic library programs.
- **S9** — conv-family/contracts close-out (conv itself landed early in S6b).

**Post-milestone fork (open):** continue the interpreter's S7-S9 std/generic tail,
OR pivot to the next self-hosting TIER — self-lowering to MIR (oracle = the Rust
MIR interpreter), the stepping stone toward a self-compiler / true native
bootstrap. The interpreter tier has proven the language can express its own
execution semantics; MIR-lowering would prove it can express compilation.

### Self-check fixpoint COMPLETED across all five modules (2026-07-11)

interp.cnr now self-checks too: the self-host checker (E0102/E0103) and analyses
(move/init, loans, effects, exhaustiveness) check the interpreter's own source
clean, oracle-matched, with teeth. The fixpoint spans lexer/parser/checker/analyses
/interp — Candor checks the program that runs Candor. A near-free bonus (no arena
bump, no new builtins) revealed by a probe correcting a stale "uses unsafe/raw-ptr"
blocker note.

## Next tier RULED: self-lowering to MIR (2026-07-11)

Direction after the self-interpreting milestone: pivot to SELF-LOWERING TO MIR over
the interpreter's std/generic tail (S7-S9). Rationale (feasibility scoping): it is
SMALLER and lower-risk than the interpreter was — the interp's tallest pole (the
annotation-directed type/layout table, built with no oracle) is already climbed and
reused wholesale; there is a complete ~2400-line Rust reference (src/mir/build.rs)
to port; and the gate reuses an independent proven oracle (the Rust MIR interpreter,
one of the four equivalent engines). Strategic value: it proves Candor can express a
COMPILATION TRANSFORM (AST -> CFG-based MIR), the first evidence of a
Candor-expressible compiler middle-end and the on-ramp to dropping the Rust
bootstrap. S7-S9 only deepens the already-proven execution tier; deferred, not
abandoned (the systems corpus is monomorphic and needs none of it).

**Gate: executional.** Candor lowers AST -> MIR, serializes it; the Rust harness
deserializes into a MirProgram, rebuilds Items/consts from the same source, runs the
Rust MIR interpreter (src/mir/interp.rs), and compares RET/TRACE/FAULT byte-exact to
the tree-walker oracle — the exact schema the self-interp gate already uses. Rejected
the structural (byte-compare serialized MIR) gate: it couples two independent
lowerings at incidental temp/block numbering; two correct lowerings differ there.

**Arc (each slice re-shapes logic the interp already has; type table reused):**
- **L0 (enabler, SOLO)** — the MIR serialization boundary: a canonical MirProgram wire
  format + the Rust deserializer + harness, built and proven against HAND-WRITTEN MIR
  before any Candor lowering exists (closes the loop first). Carries spans (load-bearing
  for FAULT identity), fault edges, projection offsets, move masks, fn_ptrs, drop_hooks,
  statics.
- **L1** — scalar + CFG: the IR-builder + control-flow flattening (if/while/loop/break/
  continue -> basic blocks + Goto/Branch/Return/Fault), scalar rvalues, fault edges,
  Trace, Return. The MVP.
- **L2** — flat aggregates (struct/array Place projections, bounds fault edges).
- **L3** — move/drop schedule emitted as explicit MIR Drop ops with static move masks.
- **L4** — enums/match -> tag-switch branch chains.
- **L5** — Box/alloc ABI + rawptr/fnptr + CallIndirect.
- **MILESTONE (after L5):** the self-host lowering lowers all five systems programs
  (11_1..11_5) to MIR, the Rust MIR interp runs them, byte-exact vs the oracle — the
  analog of the interp's systems-corpus milestone.

Tallest poles (both bounded, testable up front): the serialization boundary (L0, the
only new infra with no interp analog) and CFG construction (L1-L4, standard mechanics
with the build.rs reference). No open-ended oracle-less analysis pole this time.

### Self-lowering to MIR MILESTONE REACHED (2026-07-11)

L0-L6 done: the Candor-written lowering (selfhost/lower/lower.cnr) lowers all five
systems-corpus programs (11_1..11_5) to MIR, executed by the Rust MIR interpreter
byte-exact vs the tree-walker oracle — plus 66 targeted fixtures (71 total).
Covers scalars+CFG, aggregates, the move/drop schedule as explicit MIR Drop ops,
enums/match as tag-switch chains, and the full Box/alloc ABI (fn-ptr indirect
calls, statics, rawptr intrinsics, alloc-on-drop) + byte-strings/slices/borrow
params. Emitted wire is byte-identical to serialize(mir::build) modulo inert
parser-span diffs on non-observable statements. Every program monomorphic; nothing
needed beyond L1-L6. The gate: Candor lowers -> serialize -> Rust deserialize ->
mir::interp::run -> byte-exact, reusing L0's proven serialization boundary.

**THREE self-hosting tiers now closed — but on DIFFERENT program sets (stated precisely,
not blurred; corrected 2026-07-11 after the same conflation was found in the README):**
- self-CHECK — oracle-matched diagnostics over the compiler's OWN SOURCE (all 5 self-host
  modules incl. interp.cnr), NOT the 11_* systems corpus.
- self-INTERPRET (interp.cnr runs the systems corpus) — oracle-matched Run{ret,trace}+faults.
- self-LOWER to MIR (lower.cnr compiles the systems corpus to MIR) — oracle-matched execution.
- (Interpret and lower share the identical 11_* corpus files; the checker does not run over it.)

**Remaining tails (none gated by the milestone):**
- Interpreter S7-S9 and Lowering L7+: the std/generic library (slices/str done for the
  corpus subset; Vec/Map/String + the monomorphizer remain) — the generic tail, still
  cleanly separable (the systems corpus is monomorphic).
- Self-lowering interp.cnr/lower.cnr themselves (a deeper fixpoint), as interp.cnr
  already self-checks.
- The final tier: self-COMPILE to native (AST->MIR->native codegen in Candor / a
  ported backend) — the true native bootstrap that removes the Rust dependency. This
  is where a real code generator (Cranelift is Rust-only) must be ported or written;
  the largest remaining undertaking, still a horizon.

### Generic/std self-hosting tail: in-subset COMPLETE (2026-07-11)

(Headline scope note: "complete" means the MONOMORPHIC/in-subset surface -- plain
fn[T]/struct[T]/enum[T] + Vec/Map/String -- runs AND compiles byte-exact. TRAIT-based
generics (iface/gimpl/gbound/fromq/gfromq, 5 of 13 generic fixtures) remain deferred
on a self-host front-end gap, per the body below. Not "every generic program".)

Both proven tiers now cover user generics + std collections, byte-exact vs the Rust
reference:
- **Monomorphizer** (selfhost/mono/mono.cnr, shared): resolves fn[T]/struct[T]/enum[T]
  by discovery+inference+arena-clone-with-substitution; wired into BOTH interp (G1)
  and lower (L-gen). 8 generic fixtures run AND compile byte-exact. The 5 trait-based
  fixtures (iface/gimpl/gbound/fromq/gfromq) are deferred as a FRONT-END gap (the
  self-host parser has no interface/impl, the interp no method-dispatch/?/From) -- NOT
  a monomorphizer gap; mono correctly no-ops.
- **Collections** Vec/Map/String: interp (I-std, ports the tree-walker's bi_* onto its
  paged memory) + lower (L-std, emits the CollectionOp intrinsics) + the Rust MIR
  reference (P0, the prerequisite that gave MIR collection support). 6 collection
  fixtures run AND compile byte-exact.

Parallelism note: G1/P0 and L-gen/I-std ran as two concurrent worktree pairs (disjoint
files, isolated builds), cherry-picked clean -- the safe two-writer pattern.

**Cross-model review (2026-07-11):** after a model mix-up (the self-hosting work was
Opus 4.8, mis-signed Fable 5; corrected once Fable 5 was confirmed active), Fable 5 ran
a fresh-context adversarial audit (four independent reviewers + a first-hand test run)
of Opus 4.8's work. Verdict: GENUINE -- independent oracles, real teeth, no faking, no
hidden ignores, results not hardcoded. Two committed honesty defects fixed (a README
tier/corpus conflation; stale token-count comments). Foundation sound; clear to continue.

**Known scaling constraint:** interp.cnr self-checks with only ~56 tokens under the
parser's fixed [32768] arena. The next interp.cnr-growing feature (multibyte push,
vec_set, or trait generics) needs that arena raised (or interp.cnr split) first.

**Remaining (unchanged, not gated by this tail):** trait-based generics need self-host
front-end work (interface/impl/method-dispatch/?/From); the final tier is self-COMPILE
to native (a Candor/ported code generator -- the true bootstrap, still a horizon).

### Trait-based generics COMPLETE (2026-07-11)

The 5 previously-deferred trait fixtures now run AND compile on both self-host tiers,
closing the generic story: ALL 13 generic fixtures (plain fn[T]/struct[T]/enum[T] +
trait interface/impl dispatch + generic impls + trait bounds + ?/From error widening)
run on interp and compile to MIR on lower, byte-exact vs the Rust reference.

5 slices (~1 day): T1 parser interface/impl (contextual identifiers, lexer parity kept);
T2 mono impl-emission + generic-impl instantiation + interp method dispatch (the tall
pole -- reused clone_subst/dedup, node-id dispatch table since synthesized names are
unaddressable); T3 lower method dispatch (impl methods as free MIR fns); T4 mono From
emission/instantiation + interp ?/From (eval_try, ok/err via the ok-marked variant);
T5 lower ?/From (T_TRY CFG + From-call + early-return). T3‖T4 ran as a worktree pair.

The arc validated the "solid foundation -> fast features" thesis: it fit inside the
arena headroom the F-ARENA-CAP raise provided (interp.cnr ~14k tokens still free),
reused the monomorphizer's existing infra, and every slice had a ready dual oracle
(the fixtures already ran through the Rust monomorphize->interp/MIR pipelines).

**Self-hosted coverage now:** the self-host toolchain checks its own source, and runs
AND compiles-to-MIR: the systems corpus, std collections (Vec/Map/String), and the full
generic/trait surface. What remains for "the whole language" is minor (associated-type
members, multibyte push, vec_set -- all small, logged) and the systems-only corners.

**Remaining horizons (unchanged):** the deferred quality-review debt (OBL-QUALITY-REVIEW:
layout.cnr extraction before native codegen, the Vec::set order, the FAULT-trace harness
blind spot, etc.); and the final tier -- self-COMPILE to native (a Candor/ported code
generator, the true bootstrap).

### Self-compile to NATIVE — systems-corpus MILESTONE REACHED (2026-07-12)

The final self-hosting tier is closed over the systems corpus. selfhost/codegen/codegen.cnr
(a Candor-written code generator) emits x86-64 assembly text; the system assembler links it
against the unchanged aot_runtime.c and runs it as a real process, byte-exact (exit/trace/
fault) vs the oracle -- NO Rust in the compile path. All five systems programs
(11_1..11_5) native-compile, plus 71 targeted fixtures (76 total across N1-N6).

Slices N1 scalar+CFG -> layout.cnr extraction (F-LAYOUT-EXTRACT paid) -> N2 aggregates ->
N3 move/drop -> N4 enums/match -> N5 Box/alloc/rawptr/statics/indirect-calls -> N6 the
corpus. Target = asm text (the self-host does instruction selection + stack allocation +
SysV ABI; the assembler owns encoding/relocation/ELF -- can't reuse Cranelift, which is
Rust-only). All-on-stack, no register allocator -- the SysV/stack pole held across the
parser/arena recursion without one. The C runtime (MEM_BASE mmap, rt_trace/rt_fault/
rt_stack_alloc) is language-agnostic and reused unchanged.

**ALL FOUR self-hosting tiers now close over the same systems corpus:**
- self-CHECK   -- the checker/analyses check the compiler's own source (5 modules)
- self-INTERPRET -- interp.cnr runs the corpus (+ generics + collections)
- self-LOWER   -- lower.cnr compiles the corpus to MIR (+ generics + collections)
- self-COMPILE -- codegen.cnr compiles the corpus to a native executable

Deliberately-simple codegen (no optimization) -- a bootstrap-credibility proof, not a
replacement for the Rust/Cranelift production backend.

**Remaining (honest):** the native codegen now also covers the full user-generic/trait surface (all 13 generic fixtures, via the shared monomorphizer + method dispatch + ?/From). It does NOT
yet generics/collections/the ?/From surface (those run+compile-to-MIR self-hosted but
have no native codegen fixtures); a register allocator (all-on-stack is correct but slow);
and the deferred OBL-QUALITY-REVIEW debt (Vec::set order, the FAULT-trace harness blind
spot, deserialize arity, test-dedup). The Rust compiler remains the production toolchain.

### 0.x distribution — staging COMPLETE + standalone-verified (2026-07-12)

The publication-step-2 packaging is assembled and proven, ready to seed the standalone repo.
- **Toolchain renamed** candor-proto -> candor (Cargo [[bin]]; the candor_proto LIBRARY name
  stays; ~25 user-facing strings swept; the dist shim deleted). candor --version / --help
  added. 602 tests green, clippy clean.
- **dist/seed.sh** assembles a standalone 0.x from the lab's MANIFEST SHIPS rows: prototype/
  -> toolchain/ (tests/ + selfhost/ excluded, still builds), docs/spec -> spec/, the corelib
  seed -> stdlib/ (first-class), tools/{vscode-candor,candor-lsp} -> editor/ (LSP path-dep
  rewritten to the seeded toolchain), dist/ docs+examples -> repo root. No permanent spec/
  stdlib duplication in the lab (drift) -- copied at seed time.
- **Standalone-verified** from a scratch seed with NOTHING pointing back at the lab:
  cargo build --release -> candor; candor run hello -> 42; check clean; candor run stdlib
  (module tree) -> 380; candor build stdlib -> 8 modules, no diagnostics. All 7 examples work.
- Docs (README/INSTALL/LANGUAGE-TOUR) re-pointed to the seeded layout; formerly-dangling
  spec/stdlib/editor references now resolve in the seeded repo.

**Remaining (the operator/publishing action, deliberately out of the lab):** run seed.sh into
a target dir, git init, and publish the standalone 0.x repo -- the deciding authority's call
(it creates/owns the published artifact). Minor non-blocking: benches/p20.rs hardcodes a
fixture path (benches don't affect build/run); --version reports the crate version 0.1.0.

### LLVM backend + the road to a proper language (2026-07-12)

**Performance path RULED: an LLVM backend behind the MIR seam** (the ROADMAP's long-named
"LLVM second backend"). Cranelift is fast-COMPILING but non-optimizing; for C/C++/Rust/Zig-
competitive runtime speed the answer is to emit textual LLVM-IR and let clang -O2 optimize
(the Rust/Swift/Clang model). Cranelift STAYS for fast/debug builds; LLVM for optimized
release -- both behind the same MIR seam, both gated by the four-engine differential harness
(LLVM becomes the 5th verified engine). LLVM 18 (clang/llc/opt) confirmed installed; NO
libLLVM linkage (emit .ll text, shell to clang -- the same emit-text pattern the self-host
codegen uses); aot_runtime.c reused UNCHANGED.

**Honest performance verdict (from scoping):** a MECHANICAL mirror (everything in the flat
MEM_BASE arena via inttoptr) BEATS THE INTERPRETER BUT NOT C/RUST -- inttoptr defeats LLVM's
alias analysis + mem2reg. The payoff is a TWO-TIER value model: address-not-taken SCALAR
locals -> real alloca -> mem2reg -> SSA registers -> LLVM's full loop suite -> genuinely
C/Rust-ballpark for scalar/compute/loop code. AGGREGATE/pointer-heavy code stays capped by the
single shared arena's weak aliasing -- a PERMANENT ceiling of the flat-memory ABI, uncappable
only by a large ABI change (per-object allocas + TBAA), not worth it now. The scalar MVP (S0)
naturally captures the win (scalars have no aggregates -> straight to alloca->registers).
Correctness trap: NEVER emit nsw/nuw (LLVM deletes overflow checks as UB) -- use
llvm.*.with.overflow intrinsics + guarded div. -O2 preserves trace/fault observability
(external rt_* calls are optimization barriers).

Slices: S0 scalar+CFG (the MVP, ~400-600 lines, delivers the scalar perf win) -> S1 aggregates
(forces Tier-F/R) -> S2 enums+statics -> S3 drop (tallest pole) -> S4 Box/alloc/rawptr -> S5
FFI+concurrency -> S6 full corpus + LLVM as the 5th Stage-D engine. Collections are unimplemented
in the Cranelift backend TOO -- same corpus boundary, not a new LLVM gap. Full emitter ~1500-2200
lines.

**DELIVERED (2026-07-12), all six slices S0-S6 committed.** `candor compile --backend=llvm`
emits textual LLVM-IR through clang -O2 on the two-tier value model. S0 scalars->SSA registers
(mem2reg promotion proven); S1 aggregates (Tier-F flat arena); S2 enums+statics; S3 the
move/drop schedule with trace-on-drop (the tallest pole -- mostly free, mir::build bakes the
schedule + static move masks, no runtime drop flags); S4 Box/alloc/rawptr (all 5 systems-corpus
programs byte-exact); S5 FFI (real libc I/O) + structured concurrency (rt_spawn/rt_scope,
spawn-order-deterministic trace). S6: gate_llvm_full_corpus_fifth_engine asserts all 31 corpus
fixtures compile via clang -O2 to the byte-identical observable (exit byte / trace / fault
(kind,span)) as the tree-walking oracle -- the SAME 31 stage_d and aot close on (CollectionOp
excluded from all engines alike, so LLVM coverage == Cranelift coverage). Honest framing: the
five-engine agreement is transitive-through-a-shared-oracle (each engine == the oracle), NOT a
unified all-pairs diff, and is an empirical corpus-bounded determinism guarantee, not a formal
whole-program refinement. Recurring lesson across all six slices: Candor's MIR carries so much
structure (drop schedules, match lowering, box ops, offsets) that each slice reduced to walking
MIR and mirroring the Cranelift backend op-for-op. Emitter landed ~2100 lines.

## ROADMAP to a "proper language" (prototype -> production), ordered

1. **Performance -- the LLVM backend** (above). Optimized native code; the foundational perf move.
   *DONE (S0-S6, 2026-07-12): the optimizing LLVM backend ships behind the MIR seam, covers the
   whole Cranelift-equivalent surface, and is the 5th differentially-verified engine over the
   corpus. Remaining perf work is the aggregate-ABI ceiling (per-object allocas + TBAA), deferred
   as not worth it now.*
2. **Real std + I/O.** Iterator adapters (map/filter/fold over the 0009 protocol), a formatting
   story, and a std I/O layer over the P17 boundary (files/network). This is what lets people
   write REAL programs (today's std is Opt/Res/Arena/List/Vec/Map/String).
   *In progress (2026-07-13). LANDED: the formatting foundation (fmt_i64 + the Show/Display
   convention) and println through the std_io libc-write boundary, both-engine byte-exact;
   iterator adapters (fold/map/filter, fully generic over any Iter); the I/O error story
   (Res-typed IoError + Res/Opt combinators + ? confirmed in-tree/cross-module). Then the
   FOUNDATION for real programs: a reclaiming free-list allocator (first-fit + splitting +
   forward/backward coalescing, native on both backends), and on top of it String/Vec/Map now
   COMPILE TO NATIVE CODE on Cranelift and LLVM (allocate + grow + reclaim through that allocator,
   drop-free on scope end), byte-exact vs the interpreter across all five engines -- lifting the
   old interpreter-only ceiling. Along the way, feature work flushed out and fixed real compiler
   bugs: a coherence-checker overlap bug, method dispatch on call-shaped receivers, a
   cross-module monomorphization span-collision (a memory-safety bug), call-site associated-type
   projection normalization, audit-teeth on generic boundary modules, native str-view lowering
   (as_bytes/str_from_unchecked/str_from/substr), and two collection-op + two len/str[i]
   ergonomics papercuts. Then native FILE I/O on top of native String: whole-file
   read_to_string/read_file/write_str and read_lines/split_lines, plus a streaming BufReader
   (read_line assembling lines across buffer refills) and a BufWriter -- all byte-exact
   interp==native on both backends over the P17 std_io boundary. REMAINING: a streaming
   Lines: Iter (needs the 0009 RefIndexed borrowed-yield DESIGN); realloc / in-place
   string_clear (the one allocator ABI DECISION); best-fit; richer std (dir/network I/O, more
   combinators). The two remaining items are deciding-authority forks, not additive work.*
3. **Stability + packaging.** The 1.0 gate (editions/migrator exist via P15), a package manager,
   dependency handling. What lets OTHERS build on it.
4. **Reach.** More platforms (ARM/macOS/Windows/bare-metal; today x86-64 Linux), richer
   diagnostics/LSP, docs. The eval campaign (P19) measures the human-LLM authorship thesis.

Honest framing: the self-hosting arc (4 tiers) is a CREDIBILITY proof, done. The above is the
gap from "impressive prototype" to "language people use." Performance (1) is the most
foundational and the one with infra already in place (MIR seam + differential gate + emit-text).
