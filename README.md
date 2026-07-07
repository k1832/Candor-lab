# Candor

A systems programming language designed for the human–LLM pair as the unit of
authorship: memory-safe, explicit where meaning lives, locally verifiable, with
source-declared semantics and a compiler built as a conversation partner rather
than a gatekeeper.

**Status: Bet 5 validation in progress — and the pre-registered criterion is
biting.** The language's central bet (value-first memory model, Bet 5) is
being tested right now against a frozen, pre-registered kill criterion:
five systems programs ported to a working Candor prototype (checker +
interpreter) and measured against idiomatic Rust baselines. Two of five
ports are scored: annotation load is dramatically below Rust's (−79% and
−43%), and the pressure-valve fraction has **breached the frozen KILL
ceiling on both home-ground programs**. Live scoring:
[docs/RESULTS.md](docs/RESULTS.md). Per the philosophy, results are
published either way, and a killed bet amends the philosophy in the open
rather than being quietly reinterpreted. Nothing here is stable, and
nothing may be declared stable until the verdict is enacted.

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
