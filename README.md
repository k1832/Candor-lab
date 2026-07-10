# Candor

A systems programming language designed for the human–LLM pair as the unit of
authorship: memory-safe, explicit where meaning lives, locally verifiable, with
source-declared semantics and a compiler built as a conversation partner rather
than a gatekeeper.

**Status: the first-version scope is complete and RUNNING, and Candor is
beginning to compile itself.** The self-hosting arc is underway — a lexer,
parser, type checker, and the borrow checker's move + XOR-loan core, each
*written in Candor* and
each differentially verified token-for-token / AST-for-AST / diagnostic-for-
diagnostic against the Rust reference. The language now has text (`str`/`String`,
design 0013), a std `Vec[T]`, and standalone binaries that do real libc I/O
through an auditable trust boundary. Thirteen designs — memory model through
structured concurrency to the text budget — each adversarially reviewed; one
rejected outright and reworked to acceptance (0012, whose reviewer built a real
safe-code race against the draft's own flagship). One design-direction question
(region-bearing struct fields) was decided by a high-effort deliberation with a
falsifiable re-open trigger — and the self-hosted checker, the exact slice named
as that trigger, held the ruling on its hardest evidence.

Structured concurrency — the final language feature — executes on real
OS threads with compile-time race freedom, across all engines including
standalone AOT binaries. The compilation architecture is fully realized (design 0010,
stages A-D closed). Candor compiles through a checked MIR to optimized native x86-64
via Cranelift, with the fault-window reordering license enforced by an
executable validator and the P5 invariant made empirical: four engines -
interpreter, MIR, native, native-optimized - produce identical observable
traces and identical fault identity across the entire corpus. Incremental
builds prove zero-downstream re-analysis on body edits (P20's mechanism,
measured); the P17 boundary and audit command run; the P20 measurement
instrument reports baselines; and candor-proto compile emits standalone
native ELF executables (via cranelift-object + a small C runtime) that match
the reference oracle process-for-process across the entire corpus. A
`--freestanding` profile links that same object with **no libc** (`-nostdlib
-static -no-pie`, the flat region a static section, a root HALT fault policy, raw
syscalls for trace/exit) and runs the allocation-free core payload to its sentinel
— `ldd` reports "not a dynamic executable", the NN#6 no-mandatory-runtime proof. The language's
central bet (value-first memory model, Bet 5) was tested against a frozen,
pre-registered kill criterion: killed as first registered, re-examined under a
corrected successor registration (both on the public record), and **provisionally
confirmed** with its concessions named — philosophy v4.2. Since then: the real
surface syntax is designed and reviewed (design 0006), a normative specification
skeleton exists with every gap tracked (docs/spec/), generics and modules are
designed and reviewed (0007/0008), the novel fault-window semantics is formalized
to its single-threaded core (reviewed, repaired), and the prototype implements
the real syntax with a working P15 migrator, multi-file modules, full stage-2
generics (definition-site-checked, monomorphized without re-analysis, generic
impls and drop hooks), a growing core/std library written in Candor (Opt/Res/Arena/List, iteration
protocols, Opt::map, From-based error widening) whose P9 layering the checker
itself proves, iteration and associated types (design 0009), and VS Code support (TextMate grammar
+ a diagnostics LSP over the P4 machine-readable diagnostics). The checker has survived eight
adversarial review rounds (twelve soundness holes found and closed, all
documented). Live records: [docs/RESULTS.md](docs/RESULTS.md),
[docs/spec/99-obligations.md](docs/spec/99-obligations.md),
[docs/reviews/](docs/reviews/). Nothing is stable; NN#14's gate obligations are
met but no stability has been declared.

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
                     each oracle-gated against the Rust reference
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
