# 0010 — The compilation architecture

**Status:** draft
**Date:** 2026-07-08
**Philosophy hooks:** **P16/NN#16** (one blessed toolchain — compiler, build
system, and their reproducible, provenance-aware output; the language server and
package manager depend on this doc's artifacts but are not designed here),
**P20** (compile speed *earned by construction* — this doc realizes 0008's
two-hash mechanism as a running build and pre-registers the measurable targets
CI tracks as release criteria), **P5/NN#1/NN#2** (source-declared semantics: a
build mode changes speed, never observable behavior — the load-bearing
constraint on the backend and every optimization), **P11/NN#10** (definition-site
generics: instantiation is cached codegen, never re-analysis), **P4** (structured
diagnostics and traces are a codegen output, not an afterthought), **P9/NN#6**
(freestanding targets are first-class — a backend constraint). Subordinate to
`LANG_PHYLOSOPHY.md`, to the fault-window formalization
(`docs/spec/drafts/fault-window-formalization.md`, R1–R3 below), and to designs
0001 (the semantics to preserve), 0007 (what monomorphizes), and 0008 (the
interface artifact this doc's compiler emits). Where they conflict the higher
document wins and this one changes.

**Revision history.** 2026-07-08 — initial draft.

## Problem

Everything above this document is a *front*: front-ends, resolver, module DAG,
and the whole `check/` analysis tier (types, generics, borrows, init, effects,
contracts) exist in the prototype and run over an AST that a **tree-walking
interpreter** then executes (design 0001 §7). That interpreter is the reference
semantics — but it is not a compiler, it emits no native code, it realizes none
of P20's incremental machinery (0008 §6 stages 3–4 are unbuilt), and it cannot
substantiate the compile-speed claim §1 makes or the reproducible, native
artifact P16 requires. This document turns the front into a **compiler**: it
fixes the backend, the mid-level IR, the incremental build that realizes 0008's
two hashes, the differential-testing regime that keeps compiled and interpreted
behavior identical (P5), the pre-registered P20 targets, and the staging that
gets there without pretending the arc is one step.

It is the largest remaining arc, and the philosophy's own budget applies to it
(§8 sequencing): every "ships at stability" is a gate obligation, not a
founding-day one. So this document *stages* rather than specifies-and-builds.

## Decision

### 1. The backend — Cranelift is the blessed default; LLVM is a deferred, optional second backend behind the same seam

This is the load-bearing call. Four candidates, judged against Candor's actual
constraints, not against generic compiler-engineering taste.

**The constraints that decide it:**

1. **Checked arithmetic on every default op → branch density.** Every default
   `+ - * / conv index` carries a fault check (0001 §7.2). The IR the backend
   consumes is dense with overflow/bounds tests and fault edges. The backend must
   lower that pattern *cheaply* and must not need the check to be laundered into
   something exotic.
2. **The fault window forbids reorderings a normal optimizer performs.**
   Observable events are **effect-order-total** — never reordered, coalesced, or
   eliminated (R1/§8.3); a fault-capable op may not be hoisted before its
   preceding observable `e⁻` (R1 side-condition ii); the delivered fault is
   **program-order-first** `f★` (§6.2). A backend must give us enough ordering
   control to hold these.
3. **The soundness-vs-optimizer crux (the reason this is not a perf question).**
   NN#1 says safe code has *no* undefined behavior, and P5 makes overflow a
   *defined* fault. A backend whose optimizer assumes C-style **signed-overflow
   UB** — that overflow "cannot happen" — may *delete a Candor overflow check*
   ("this add cannot overflow, so the fault branch is dead") or narrow a value's
   range on that false premise. That is not a slow build; it is an **unsound**
   one: safe Candor code would acquire the UB NN#1 forbids, through the backend's
   back door. The invariant that closes the door: **the compiler never hands the
   backend IR that makes a promise Candor's semantics does not hold.** Concretely
   — (a) a default arithmetic op is lowered to an *explicit* overflow-detecting
   operation plus a branch to a fault block, so the check is **data in the IR the
   optimizer sees**, never a fact it may assume away; (b) the lowering never sets
   a UB-implying flag (LLVM's `nsw`/`nuw`); (c) observable ops are marked
   volatile/atomic so the effect-order-total rule survives the optimizer.
4. **Compile speed (P20) favors Cranelift-class codegen over LLVM -O2.**
5. **Reproducibility (NN#16): same source + lockfile + toolchain ⇒ bit-identical
   artifact.** An *in-process, vendored, deterministic* backend makes this
   tractable; shelling out to an external toolchain does not.
6. **Freestanding target breadth (P9/NN#6).**

**How each candidate scores:**

- **Cranelift (recommended default).** Its IR has **no signed-overflow-UB
  concept to trip**: `iadd` is defined 2's-complement wrapping, and checked
  arithmetic lowers to `iadd` + an overflow-flag test + `trapnz`/`brif` to a
  fault block — constraint (a) is the *natural* lowering, not a workaround. Its
  optimizer is deliberately light (no aggressive UB-exploiting range passes), so
  the *entire class* of "optimizer assumed UB Candor forbids" bug is
  **definitionally absent** — there is no live wire to avoid. It is built for
  fast, single-pass-ish codegen (Wasmtime, rustc's debug backend), matching
  P20's stated preference. It is a vendored Rust crate compiled in-process and
  deterministic, so NN#16 reproducibility is near-free. Ordering control is
  explicit. **Its real cost, named:** narrower target coverage — x86-64,
  aarch64, riscv64, s390x, Wasm; **no 16/32-bit embedded** (no AVR/thumb), which
  is exactly some of P9's freestanding home ground. And it will lose peak
  benchmarks to LLVM -O2 (accepted per §2's Priority Order — peak perf is not a
  default, but Priority 4 says it must be *reachable*, which motivates the second
  backend).

- **LLVM (recommended *optional*, deferred second backend).** Best-in-class
  optimization and the target breadth Cranelift lacks (embedded included). But it
  can be made sound for Candor **only by discipline** — never emit `nsw`/`nuw`,
  always lower checked ops via `llvm.sadd.with.overflow` intrinsics, mark every
  observable volatile/atomic. Soundness then rests on us *not tripping a live
  wire* on every lowering, forever. It is slow at -O2 (against P20) and it is a
  large external dependency whose version bleeds into reproducibility. **Verdict:
  worth having, not worth being the default** — kept behind the MIR→backend seam
  (§2) for release-optimization builds and for targets Cranelift cannot reach,
  gated by differential testing (§4) so it can never diverge observably.

- **C emission (rejected as a primary path).** Tempting for portability, but it
  maximizes the very hazard of constraint 3: C is a *field* of UB (signed
  overflow, strict aliasing, uninitialized reads, unspecified evaluation order),
  each an assumption the C optimizer may exploit against Candor's defined
  semantics. Preserving Candor's *defined evaluation order* (0001 §7, formalized
  `≤po`) requires sequencing every subexpression into its own statement; checked
  arithmetic requires `__builtin_*_overflow` rather than `+`; and NN#16
  reproducibility now depends on an external C compiler's exact version. The
  soundness surface is the largest of any option and the reproducibility story
  the weakest. Rejected as the blessed path (retained only as a possible
  last-resort porting target, behind the same seam, never the default).

- **Custom backend (rejected).** Full control, but it is a multi-year register
  allocator / instruction selector the project has no budget for (§8), and it
  buys nothing Cranelift does not already give for the systems targets that
  matter first.

**Recommendation.** **Cranelift is the default and only backend the first
toolchain ships**, chosen because its minimal-assumption optimizer makes the
soundness crux *structurally* safe rather than disciplined-safe, its speed and
in-process determinism serve P20 and NN#16 directly, and checked arithmetic is
its natural lowering. **LLVM is a recorded, deferred, optional second backend**
behind the §2 seam, added when (i) a measured peak-performance need makes
Priority 4's "reachable" bite, or (ii) a required target is outside Cranelift's
set — and *only* under the §4 differential guarantee that it never changes an
observable trace. The full trade is on the record: we cede peak -O2 benchmarks
and broad embedded coverage *now* to buy soundness-by-construction, compile
speed, and reproducibility *now*.

### 2. The IR — one checked MIR, the carrier of 0001's semantics and 0008's artifact body

There is **one mid-level IR**. It is 0008's "checked generic MIR" — the same
artifact that crosses the module boundary in a `pub` generic's interface (0008
§2.4) is the artifact the backend lowers. This single IR plays three roles and
must serve all three without a second lowering:

- **(a) The checker's facts annotate it.** Drop schedules (0001 §7.4 — static,
  no runtime flags), move masks / init facts (NN#5), effect markers
  (def-site-resolved `alloc`, later foreign-trust — 0008 §2.4), region/loan
  provenance, and contract-check points ride *on* the MIR as annotations, not as
  re-derivable side computations. The analysis tier (which survives from the
  prototype) produces them once; the MIR carries them.
- **(b) Monomorphization operates on it** (P11/0007 §5): a `pub` generic's MIR
  body is emitted *already checked* into its interface artifact; instantiation
  substitutes concrete types and lowers — **zero semantic re-analysis** (0008
  §2.4). The MIR is therefore both the analysis output *and* the monomorphization
  input, which is why it is one IR and not two.
- **(c) It lowers to the backend** (Cranelift IR today; LLVM IR behind the same
  seam tomorrow). The lowering is the only backend-specific code; everything
  above it is backend-agnostic.

**Fault-semantics preservation, stated as MIR invariants** (each is a checkable
property of a well-formed MIR function; the lowering must preserve every one, and
a backend pass that breaks one is a soundness bug, not a perf regression):

- **INV-CHECK.** Every default arithmetic, conversion, and indexing op carries
  its fault check *explicitly* — an overflow/bounds-detecting op plus an edge to
  a fault block. No op relies on the backend to insert a check or to assume one
  away (the §1 crux, in IR form). `wrapping`/`saturating` regions lower to
  *distinct, non-faulting* opcodes, so the regime is visible in the IR exactly as
  it is greppable in source (P5).
- **INV-OBS-ORDER.** Observable operations — MMIO/volatile accesses, atomics,
  syscalls, foreign calls, and fault delivery itself — are **effect-order-total**:
  the MIR marks them, and the lowering may **never** reorder, coalesce, or
  eliminate them (R1/§8.3). They emit in full program order.
- **INV-FAULT-ID.** The delivered fault is **program-order-first** `f★` (§6.2):
  the MIR's fault edges preserve `≤po`, and no fault-capable op is hoisted before
  its `e⁻` (R1 side-condition ii, the window constraint). Every build mode
  therefore delivers the *same fault identity* (kind + source span); only the
  non-observable value-context `c` may differ within the window (§6.4).
- **INV-EFFECTS.** Each MIR item carries its def-site-resolved effect set; codegen
  **consumes** it and **never re-derives** an effect (0008 §2.4). Effect
  resolution is the once-only analysis-tier job.
- **INV-DROP / INV-MOVE.** Drops are emitted at exactly the statically scheduled
  points (0001 §7.4, no runtime drop flags); no lowering may emit a read of a
  place not statically proven initialized (NN#5).
- **INV-R1-ONLY.** The **only** reordering the lowering may perform, or delegate
  to the backend, is **R1**: reorder/fuse/eliminate mutually-independent,
  fault-independent *internal* (`τ`) steps. R2 (window-interior retirement) and R3
  (late fault detection — the freedom Bet 3 buys) are backend *detection*
  liberties it may exploit, but never in violation of INV-OBS-ORDER or
  INV-FAULT-ID. This invariant is how P5's "build modes change speed, never
  behavior" becomes an object the compiler can enforce: any candidate optimization
  is legal iff it is an R1/R2/R3 rewrite over the MIR, and illegal otherwise.

### 3. The incremental build — 0008's two hashes, realized

`candor build` is the compiler realizing 0008 §2 as a running build graph:

- **Artifact format.** Per module, a serialized interface artifact (0008 §2):
  module path + boundary marker; every `pub` item's full signature; every `pub`
  generic's and `inline` item's *checked MIR* body; and the **two content
  hashes** — a **signature hash** over signatures+markers, and a **codegen hash**
  additionally covering the MIR bodies. Provenance (package identity, toolchain
  version, source hashes) is recorded in the artifact for P16 auditability.
- **The build graph.** Nodes are modules over the acyclic import DAG (0008 §3).
  A `candor build` (i) resolves the DAG, (ii) for each module whose imports'
  **signature hashes** are unchanged and whose own source is unchanged, reuses its
  cached analysis; otherwise re-analyzes it against its imports' signatures
  *only* — never their bodies (P20, 0008 §2's analysis-invalidation tier); (iii)
  emits/reuses machine code per instantiation from a **content-addressed codegen
  cache** keyed on the codegen hash + type arguments (0008 §2.4), fully parallel
  and shared across the build (and, under NN#16, across builds). A body edit that
  leaves every `pub` signature hash unchanged re-analyzes **nothing downstream**
  and re-emits only that body's own instantiations — P20 delivered literally.
- **Reproducibility (NN#16).** Codegen is content-addressed and the backend is
  deterministic and in-process, so same source + lockfile + toolchain ⇒
  bit-identical artifact. A CI rebuild-and-compare job asserts it.

**The P20 target harness — pre-registered PROPOSED numbers.** P20 makes
falsifiability a *duty*: measurable targets, tracked in CI as release criteria,
withdrawn by amendment if missed. These numbers are **PROPOSED**, to be ratified
(or amended) by the deciding authority **before the toolchain stability gate**;
the reference project is an N-module, M-line representative systems codebase with
its corelib dependency (initial reference: **N = 200 modules, M ≈ 50 000 lines**,
grown as the corpus grows). Targets:

- **T1 — incremental rebuild** (one body edit, no `pub`-signature change, warm
  cache): **< 2.5 s wall, single-digit-seconds hard ceiling.** This is the
  headline P20 claim and the one 0008 §6 stage 3 first makes measurable.
- **T2 — incremental analysis scope** (instrumented, not timing): a body edit
  re-analyzes **only** the edited module and re-emits **only** its own
  instantiations — zero downstream re-analysis. A regression here is a P20
  *architecture* break, caught structurally.
- **T3 — clean debug build** (Cranelift, no-opt) of the reference project:
  **within K = 3× of a C toolchain baseline** (`clang -O0` on an equivalent-LOC C
  project, same host).
- **T4 — analysis throughput** (`check`, no codegen): **≥ 100 000 lines/s/core**,
  scaling ~linearly across cores over the DAG (P20's parallel-by-construction).
- **T5 — release build** (LLVM backend, *when it exists*): within **Kr = 1.3×** of
  `clang -O2` on equivalent LOC — recorded as a target for the *optional* backend,
  **not** a stability gate for the shipped Cranelift toolchain.

**The CI mechanism.** A dedicated benchmark job runs T1–T5 on the reference
project every release-candidate build, on pinned hardware, and **fails the
release** if a ratified target regresses. Per P20/§9 a sustained miss is enacted
as an amendment (withdraw or restate the §1 claim), never absorbed as silence.

### 4. Differential testing — the interpreter is the oracle, P5 is the test axis

P5/NN#2 says every build mode agrees observably. This document makes that
**testable** by keeping the tree-walking interpreter as the **reference oracle**
and asserting trace equality across every execution path.

- **The oracle.** The prototype's tree-walking interpreter (0001 §7) is the
  reference semantics. Stage A adds an **interpreter-over-MIR** whose first duty
  is to reproduce the tree-walker exactly (validating that the MIR lost no
  semantics); thereafter the MIR-interpreter is the oracle the *compiled* artifact
  is checked against.
- **The trace.** The observable trace is the formalization's `θ` — the
  `Obs`-subsequence of the run (MMIO/volatile writes, syscalls, foreign effects,
  program output) **plus the fault event** `(k, s)` if any. Two runs are
  **trace-equal** iff their `θ` match; the non-observable value-context `c` and
  the window-interior retirement (R2) may differ and are excluded from equality
  (§4.3/§6.4) — that exclusion is *itself* part of the spec being tested.
- **Every basket program and every corelib/check fixture** must produce
  **identical traces compiled vs interpreted**, and identical traces **across
  build modes** (Cranelift-noopt vs Cranelift-opt vs LLVM-if-present vs
  MIR-interpreter). Cross-mode trace equality *is* NN#2 mechanized: a mode that
  changes an observable trace has changed behavior, and the job fails.
- **Fault-identity equality is a first-class axis.** For every faulting fixture,
  every mode must deliver the **same `f★`** — same fault kind and same source span
  (program-order-first, §6.2). This is the sharpest test of INV-FAULT-ID and the
  one an aggressive optimizer is most likely to break; it is checked on every
  faulting program, not sampled.

This harness is also the test runner's engine (P4/P16) and the gate every stage
below is measured by.

### 5. Staging — four stages, each with a gate that must pass before the next

Per §8 sequencing, the arc is staged and honest about what each stage validates.

- **Stage A — MIR + interpreter-over-MIR (replaces tree-walking as the pipeline
  front).** Define the checked MIR (§2); lower AST→MIR carrying *all* checker
  facts (drop schedule, move masks, effect markers, fault checks, INV-*). Build
  the MIR-interpreter. **No backend yet.** *Gate:* the MIR-interpreter reproduces
  the tree-walking interpreter's observable trace — including fault identity —
  **bit-for-bit** on every basket program and every corelib/check fixture. This
  proves the IR is a faithful carrier of 0001's semantics *before* any native
  codegen exists. (The tree-walker is retained as the reference oracle, not
  deleted.)

- **Stage B — single backend (Cranelift), no optimization, whole-program.** Lower
  MIR→Cranelift IR→native; no incremental artifacts yet (whole-program each
  build). *Gate:* the compiled artifact's observable trace == the interpreter's
  on the full basket + fixtures, including fault identity (§4), on at least one
  hosted target. This is the first native codegen and the first compiled-vs-
  interpreted differential test.

- **Stage C — incremental artifacts + the two-hash tiers.** Realize 0008 §2:
  interface-artifact format, signature-hash analysis gate, codegen-hash
  content-addressed cache, `candor build` over the DAG. *Gate:* T2 holds
  (instrumented: a body edit re-analyzes nothing downstream), T1 is *measured in
  CI* (even before ratification), and the NN#16 rebuild-and-compare job is green
  (bit-identical artifacts). This is where P20's incrementality first exists to be
  measured (0008 §6 stage 3).

- **Stage D — optimization within the R1 license + P20 measurement.** Enable
  Cranelift optimization (and/or add the LLVM backend), with **every pass
  validated as an R1/R2/R3 rewrite** over the MIR (INV-R1-ONLY). *Gate:* cross-
  mode differential testing (noopt vs opt vs interpreter, and LLVM if present) is
  green on the full corpus *including fault identity*; T3–T5 are measured and
  tracked in CI as release criteria. Any optimization that changes an observable
  trace is a soundness bug that **blocks the stage**, never a tolerated perf win.

Each gate is a hard precondition for the next: no backend before the IR is
validated (A→B), no incrementality before native codegen agrees with the oracle
(B→C), no optimization before incrementality and reproducibility hold (C→D).

### 6. P16 components this document does NOT design

Each with its one-line dependency on what is fixed here:

- **Package manager / dependency resolution + lockfile** — depends on this doc's
  interface-artifact and provenance format (0008 deferred the lockfile to it).
- **Formatter internals (canonical form)** — depends on the surface grammar
  (0006), independent of codegen; only its *shipping alongside the compiler* is a
  P16 obligation this doc's toolchain will host.
- **Documentation generator** — depends on the interface artifact's
  signature + contract data (§3), reading it, not codegen.
- **Language server** — depends on the incremental analysis tier and interface
  artifacts (§3), sharing the signature-hash engine; interactive, not batch (P16).
- **Test runner CLI/reporting** — its *engine* is §4's differential harness; its
  command surface and structured-trace reporting (P4) are a separate, dependent
  round.
- **C-header ingestion / boundary-module FFI codegen** (P14/P17) — depends on the
  boundary-module FFI content 0008 §4 deferred; lowers through this MIR when it
  lands.
- **The normative spec + mechanized fault formalization** (P18/NN#20) — a sibling
  at the design tier, not this doc; its R1–R3 license *constrains* this backend
  (§2) rather than being produced by it.

## Rejected alternatives

- **LLVM as the default backend.** Rejected for the first toolchain: it makes
  soundness *disciplined* (never trip `nsw`/`nuw`, always intrinsics) rather than
  *structural*, it is slow at -O2 against P20, and its external-version dependency
  weakens NN#16. Kept as a deferred optional second backend (§1) precisely so its
  strengths — peak perf, target breadth — are *reachable* (Priority 4) without
  paying its costs by default.
- **C emission as the blessed path.** Rejected (§1): it maximizes the soundness
  surface of the §1 crux (a whole field of UB the C optimizer may exploit against
  Candor's defined semantics), forces defended sequencing to preserve evaluation
  order, and makes reproducibility hostage to an external compiler version.
- **A custom backend.** Rejected (§1): multi-year cost with no budget (§8) and no
  advantage over Cranelift on the systems targets that matter first.
- **Two IRs (a separate "analysis IR" and "codegen IR").** Rejected: 0008 §2.4
  *requires* the checked generic body that crosses the module boundary to be the
  monomorphization/codegen input; a second lowering would either re-derive facts
  (violating INV-EFFECTS / P20's check-once) or risk the two IRs disagreeing about
  semantics. One MIR, three roles (§2).
- **A bytecode VM as the shipping execution model.** Rejected: it satisfies
  neither P9 (freestanding native images) nor P20's competitive-native-speed
  claim; the interpreter's role is *oracle*, not deployment target.
- **Re-checking (any analysis) at codegen / instantiation.** Rejected under
  P11/NN#10 and P20 (0008 §2.4): instantiation is codegen over already-checked
  MIR, never re-analysis; INV-EFFECTS makes this an IR invariant.
- **Optimizing before the IR is validated (Stage D work before Stage A's gate).**
  Rejected as a sequencing error: an optimizer is only *definable* as R1/R2/R3
  rewrites over a MIR whose semantics are proven faithful to 0001; optimizing an
  unvalidated IR would test speed against an oracle we do not yet trust.
- **Deleting the tree-walking interpreter once the compiler works.** Rejected: it
  is the differential oracle (§4) — the mechanization of P5's build-modes-agree
  invariant — and retiring it would remove the only independent check that the
  compiler preserves the reference semantics.

## Consequences and costs

- **Cranelift's target gap is a real debt (P9/NN#6).** The first toolchain cannot
  target 16/32-bit embedded (AVR, thumb) — some of the freestanding home ground
  §1 of the philosophy names. The mitigation (the LLVM backend behind the seam)
  is *deferred*, so until it lands those targets are unreachable. Named, not
  dissolved; the seam (§2) is what keeps the debt payable.
- **Peak performance is not reachable until Stage D + LLVM.** Priority 4 requires
  peak machine speed be *reachable* explicitly; between shipping the Cranelift
  toolchain and adding the LLVM backend, it is not. This is a staged gap, not a
  permanent refusal, but it is a gap.
- **The MIR invariants are a permanent tax on every future optimization.** Every
  pass anyone ever writes must be provably an R1/R2/R3 rewrite; "obviously
  correct" optimizations that reorder an observable or hoist a fault-capable op
  are *forbidden*, even when they would be legal in a UB-exploiting compiler. This
  is the price of P5, paid in optimizer expressiveness, forever.
- **The differential harness is a standing cost.** Every basket program and
  corelib fixture must run under every build mode on every release candidate, and
  the fixture corpus must grow with the language. This is real CI weight; it is
  also the only thing that keeps NN#2 from decaying to folklore.
- **The proposed P20 numbers may not survive contact.** T1–T5 are guesses
  disciplined by P20's duty to pre-register *something* falsifiable. If the
  ratified targets cannot be met, the §1 compile-speed claim is withdrawn by
  amendment (P20/§9) — the cost of honesty is that the headline claim is
  genuinely at risk, by design.
- **One IR serving three roles couples them.** A change to the MIR for codegen
  reasons can ripple into the artifact format (0008) and monomorphization (0007);
  the coupling is deliberate (it is what makes check-once/instantiate-cached
  sound) but it means the MIR schema is a high-blast-radius interface that
  versions with the edition (P15).

## Reclassification record

No decision in this document turns on the §2 rule of reclassification.
