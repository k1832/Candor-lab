# Candor

A systems programming language designed for the human–LLM pair as the unit of
authorship: memory-safe, explicit where meaning lives, locally verifiable, with
source-declared semantics and a compiler built as a conversation partner rather
than a gatekeeper.

**Status: design surface complete; implementation catching up.** The language's
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
impls and drop hooks), a first core/std library seed written in Candor whose
P9 layering the checker itself proves, and VS Code support (TextMate grammar
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

The project follows the sequencing in philosophy §8:

1. **Bet 5 validation** *(current phase)* — pre-register the kill criterion,
   design the memory-model core, build a throwaway-syntax prototype
   (checker + interpreter), port the adversarial workload basket
   (allocator, intrusive-list scheduler, MMIO driver state machine, parser,
   arena compiler pass), publish the verdict either way.
2. **Semantic core and specification skeleton** — gated on Bet 5 surviving.
3. **Minimal toolchain**, then breadth.

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
prototype/           Bet 5 validation prototype (throwaway syntax; checker + interpreter)
baselines/rust/      the five idiomatic Rust baselines (frozen)
ports/candor/        the five Candor ports (development history public as it happens)
tools/rust-count/    the Rust-side measurement counter
```

## Provenance

This project is an experiment in human–LLM pair authorship, per its own
thesis: the deciding authority is human (see GOVERNANCE.md); design,
implementation, adversarial review, and porting are performed by LLM
sessions under that authority, with review records, dispositions, and
adjudications on the public record throughout.
