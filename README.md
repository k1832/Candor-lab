# Candor

A systems programming language designed for the human–LLM pair as the unit of
authorship: memory-safe, explicit where meaning lives, locally verifiable, with
source-declared semantics and a compiler built as a conversation partner rather
than a gatekeeper.

**Status: the first-version scope is complete and RUNNING, and Candor's toolchain is substantially self-hosted — it checks itself, and runs and compiles its own kind of program.** Three self-hosting tiers are closed, each by Candor-written tooling verified byte-exact against the Rust reference — though the tiers operate on different program sets, which is worth stating precisely rather than blurring.

**Self-checking** operates on the compiler's *own source*: the Candor-written checker and analyses check all five self-host modules (lexer/parser/checker/analyses/interp) clean.

**Self-interpreting** and **self-lowering** operate on the systems-heavy corpus (a bump allocator, an intrusive-list scheduler, MMIO registers, a recursive-descent parser, a `Box [4096]Node` arena): the Candor-written interpreter *executes* it, and the Candor-written lowering *compiles it to MIR* that the Rust MIR interpreter then runs — both reproducing the reference's exact results. (The checker does not run over that corpus; the interpret and lower tiers share the identical corpus files.) Beyond the systems corpus, both the interpret and lower tiers also cover the **full generic and trait surface** and the **std collections** — user `fn[T]`/`struct[T]`/`enum[T]`, `interface`/`impl` method dispatch, generic impls, trait bounds, `?`/`From` error widening, and `Vec`/`Map`/`String` — via a self-hosted monomorphizer (`mono.cnr`); all thirteen generic fixtures run and compile byte-exact. So the self-hosted toolchain now handles essentially the whole language, not a subset (the honest remaining gaps are minor and logged: associated-type members, multibyte `push`, `vec_set`).

**Self-lowering:** a Candor-written lowering (`lower.cnr`) translates the parsed AST to the compiler's MIR — flattening structured control flow to a basic-block CFG, allocating temps, emitting the move/drop schedule and fault edges as explicit IR — and the Rust MIR interpreter (one of the four proven-equivalent engines) runs that MIR to the same `Run{ret, trace}` and fault identity as the tree-walker, over the entire corpus. It is the first evidence Candor can express a compiler middle-end, not just execution. Two self-checking / self-interpreting fixpoints sit beneath it.

**Self-checking:** the self-hosted front-end — written in Candor — name-resolves *and* runs its full analysis core (move/init, the borrow checker's XOR loans, the alloc-effect partition, match exhaustiveness) over its own source (`lexer.cnr`, `parser.cnr`, `checker.cnr`, `analyses.cnr`), each module checked in isolation with its `use` imports resolved and byte-equal to the Rust oracle.

**Self-interpreting:** a tree-walking interpreter *written in Candor* (`interp.cnr`) executes Candor programs — through scalars, structs/arrays, the move/drop schedule with trace-on-drop, enums/match, `Box`/allocator ABI, and a paged pointer/MMIO memory model — and runs the entire systems-heavy corpus (a bump allocator, an intrusive-list scheduler, MMIO registers, a recursive-descent parser, a `Box [4096]Node` arena) byte-exact against the Rust reference: `Run{ret, trace}` and fault identity alike, riding the prototype's proven four-engine equivalence.

**Self-compiling to native is closed over the systems corpus** — the final tier, the true bootstrap: a Candor-written code generator (`codegen.cnr`) emits x86-64 assembly that the system assembler links against a language-agnostic C runtime and runs as a real process, and it native-compiles **all five systems-corpus programs** (allocator, scheduler, MMIO, parser, arena) to actual executables, byte-exact (exit code / trace / fault) against the oracle — with **no Rust anywhere in the compile path** (just Candor and `as`/`ld`). It does its own instruction selection, stack allocation, and SysV calling convention; only the mechanical encoding/linking is handed to the assembler.

**So all four self-hosting tiers now close over the same corpus — Candor checks, interprets, lowers-to-MIR, and compiles-to-native its own hardest programs.** The native tier also covers the **full user-generic and trait surface** (all thirteen generic fixtures — `fn[T]`/`struct[T]`/ `enum[T]`, `interface`/`impl` dispatch, generic impls, trait bounds, `?`/`From` — via the shared monomorphizer). The one surface the *self-hosted* codegen (`codegen.cnr`) does not yet cover is the std collections (`Vec`/`Map`/`String`); the Rust reference backends now *do* compile them to native code (see the native-collections note below). (The self-hosted codegen is deliberately simple — it does local register allocation over the callee-saved registers but no global optimization; it is a bootstrap-credibility proof, not a competitor to the Rust/Cranelift backend that remains the production toolchain.)

The self-check fixpoint closes over the interpreter too — the checker and analyses check `interp.cnr` clean, so Candor checks the very program that runs Candor. Dogfooding on real self-host source and real corpus programs repeatedly earned its keep — it caught defects the fixture suites had missed: a checker/interpreter identity desync, a false use-after-move on reborrowed parameters, an array-copy misclassification, an interpreter static-region leak that corrupted live memory, an unbounded place-recursion, and a borrow-parameter stored by-value that made a self-hosted allocator read through its own context pointer — each fixed and regression-gated. The self-hosting arc is differentially verified token-for-token / AST-for-AST / diagnostic-for-diagnostic / trace-for-trace against the Rust reference, and the self-hosted compiler is itself a `use`/`pub` module tree, dogfooding the language's own module system.

The language now has text (`str`/`String`, design 0013), a std `Vec[T]` with borrowed-element and UTF-8 char iteration (both region-free paths that vindicated the region-fields ruling's bet), and standalone binaries that do real libc I/O through an auditable trust boundary. Thirteen designs — memory model through structured concurrency to the text budget — each adversarially reviewed; one rejected outright and reworked to acceptance (0012, whose reviewer built a real safe-code race against the draft's own flagship). One design-direction question (region-bearing struct fields) was decided by a high-effort deliberation with a falsifiable re-open trigger — and the self-hosted checker, the exact slice named as that trigger, held the ruling on its hardest evidence.

Structured concurrency — the final language feature — executes on real OS threads with compile-time race freedom, across all engines including standalone AOT binaries. The compilation architecture is fully realized (design 0010, stages A-D closed). Candor compiles through a checked MIR to optimized native x86-64 via Cranelift, with the fault-window reordering license enforced by an executable validator and the P5 invariant made empirical: **five independently-built Stage-D backends** - tree-walking interpreter, MIR interpreter, Cranelift (no-opt), Cranelift optimized, and an LLVM `clang -O2` linked ELF - produce the byte-identical observable (return/exit byte, trace, and fault identity as `(kind, span)`) across the entire corpus. Each engine is verified byte-exact against one shared tree-walking oracle, so their mutual agreement is transitive through that oracle rather than a direct all-pairs diff; it is an empirical, corpus-bounded cross-backend determinism guarantee — a fifth, differently-built optimizing native codegen preserving the exact observable semantics — not a formal whole-program refinement.

Incremental builds prove zero-downstream re-analysis on body edits (P20's mechanism, measured); the P17 boundary and audit command run; the P20 measurement instrument reports baselines; and `candor compile` emits standalone native ELF executables (via cranelift-object + a small C runtime) that match the reference oracle process-for-process across the entire corpus.

A `--freestanding` profile links that same object with **no libc** (`-nostdlib -static -no-pie`, the flat region a static section, a root HALT fault policy, raw syscalls for trace/exit) and runs the allocation-free core payload to its sentinel — `ldd` reports "not a dynamic executable", the NN#6 no-mandatory-runtime proof.

A second, optimizing backend is coming online alongside Cranelift: `candor compile --backend=llvm` emits textual LLVM-IR through `clang -O2` on a two-tier value model — address-never-taken scalars live in `alloca` slots that `mem2reg` promotes to real SSA registers (so the compute hot path gets genuine LLVM optimization), while aggregates and address-taken values use the flat memory arena. It already compiles the **entire systems corpus** (allocator, scheduler, MMIO, parser, `Box [4096]Node` arena) byte-exact — exit code, trace, and fault identity — against the reference oracle, with heap allocation, the move/drop schedule with trace-on-drop, enums, statics, FFI (real libc I/O), and structured concurrency all in-subset — the whole Cranelift-equivalent surface. It is now the **fifth** differentially-verified Stage-D engine (below): all 31 corpus fixtures compile through `clang -O2` to the byte-identical observable.

The **std collections and their allocator now compile to native code**, lifting the previous interpreter-only ceiling. A real reclaiming **free-list allocator** — written in Candor over the sanctioned `rawptr` valve, with first-fit allocation, block splitting, and forward/backward coalescing — runs on both native backends (and reclaims: a freed block is provably reused). On top of it, `String`, `Vec[T]`, and `Map[V]` lower on both Cranelift and LLVM: the 5-word `{buf,len,cap,ctx,vt}` header, `alloc`-new + copy + `free`-old growth (no `realloc`), UTF-8 `push`, bounds and `Requires` faults, hashing (FNV-1a + linear probing + rehash for `Map`), drop-on-overwrite, and buffer-and-element freeing on scope-end drop — every operation byte-exact against the MIR-interpreter oracle across all five engines. Native collections allocate *and* reclaim through the free-list allocator, so a compiled Candor program owns its heap end to end.

The language's central bet (value-first memory model, Bet 5) was tested against a frozen, pre-registered kill criterion: killed as first registered, re-examined under a corrected successor registration (both on the public record), and **provisionally confirmed** with its concessions named — philosophy v4.2.

Since then: the real surface syntax is designed and reviewed (design 0006), a normative specification skeleton exists with every gap tracked (docs/spec/), generics and modules are designed and reviewed (0007/0008), the novel fault-window semantics is formalized to its single-threaded core (reviewed, repaired), and the prototype implements the real syntax with a working P15 migrator, multi-file modules, full stage-2 generics (definition-site-checked, monomorphized without re-analysis, generic impls and drop hooks), a growing core/std library written in Candor (Opt/Res/Arena/List, iteration protocols, Opt::map, From-based error widening) whose P9 layering the checker itself proves, iteration and associated types (design 0009), and VS Code support (TextMate grammar + a diagnostics LSP over the P4 machine-readable diagnostics).

The checker has survived eight adversarial review rounds (twelve soundness holes found and closed, all documented).

Live records: [docs/RESULTS.md](docs/RESULTS.md), [docs/spec/99-obligations.md](docs/spec/99-obligations.md), [docs/reviews/](docs/reviews/). Nothing is stable; NN#14's gate obligations are met but no stability has been declared.

## Documents

| Document | Role |
|---|---|
| [LANG_PHYLOSOPHY.md](LANG_PHYLOSOPHY.md) | The founding document (v4). Normative; outranks everything else in this repository. |
| [GOVERNANCE.md](GOVERNANCE.md) | The deciding authority and amendment mechanics required by philosophy §9. |
| [docs/BET5_CRITERION.md](docs/BET5_CRITERION.md) | Pre-registered kill criterion for Bet 5 (value-first memory model). |
| [docs/design/](docs/design/) | Numbered design documents. Each records what was rejected and why, per philosophy §8.6. |

Document hierarchy (philosophy §9): philosophy > design documents >
implementation > compiler behavior. Conflicts resolve upward — the lower
artifact changes, or the philosophy is amended in the open. Never a quiet
divergence.

## Critical path

The project followed the sequencing in philosophy §8, and has walked the whole of it:

1. **Bet 5 validation** *(complete)* — criterion pre-registered and frozen, memory
   model designed, the throwaway-syntax prototype built, the five-program basket
   ported and measured against idiomatic Rust; verdict published (killed as first
   registered, provisionally confirmed under a corrected successor registration).
2. **Semantic core and specification skeleton** *(complete)* — thirteen reviewed
   designs, a normative spec with a live obligations ledger, the fault model
   formalized.
3. **Toolchain, then breadth** *(in progress)* — the full compiler (four verified
   engines, native + freestanding emission, incremental build, formatter, migrator,
   LSP), a self-hosted core, and the P19 competence apparatus all exist at prototype
   scale. Remaining: self-hosting completion, the 0.x distribution, and the 1.0
   stability gate (see [ROADMAP.md](ROADMAP.md)).

## Repository layout

```
LANG_PHYLOSOPHY.md   founding document
GOVERNANCE.md        deciding authority, amendment process
docs/
  BET5_CRITERION.md  pre-registered kill criterion (FROZEN since the first port commit)
  BET5_UNIT_TABLE.md frozen measurement classification table
  RESULTS.md         live Bet 5 scoring record
  ADJUDICATIONS.md   public rulings on spec/measurement ambiguities
  FREEZE_MANIFEST.md artifact hashes per criterion freeze step
  basket/            frozen language-agnostic specs for the five programs
  design/            numbered design documents (memory model, grammar, soundness)
  reviews/           adversarial review records with dispositions
  measurements/      raw counter output for baselines and ports
prototype/           the compiler: lexer, parser, checker, MIR, Cranelift backend,
                     AOT/freestanding emission, incremental build, formatter, migrator
prototype/selfhost/  Candor compiling itself — lexer/parser/checker/analyses in .cnr,
                     each oracle-gated against the Rust reference. All six slices
                     load as a `use`/`pub` module tree (dogfooding stage-1 modules)
dist/                the extractable 0.x distribution surface (README, tour, examples)
baselines/rust/      the five idiomatic Rust baselines (frozen)
ports/candor/        the five Candor ports (development history public as it happens)
tools/               rust-count, the diagnostics LSP, VS Code extension, and the
                     P19 apparatus (spec pack, eval harness, corpus pipeline)
```

## Provenance

This project is an experiment in human–LLM pair authorship, per its own
thesis: the deciding authority is human (see GOVERNANCE.md); design,
implementation, adversarial review, and porting are performed by LLM
sessions under that authority, with review records, dispositions, and
adjudications on the public record throughout.
