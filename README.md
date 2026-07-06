# Candor

A systems programming language designed for the human–LLM pair as the unit of
authorship: memory-safe, explicit where meaning lives, locally verifiable, with
source-declared semantics and a compiler built as a conversation partner rather
than a gatekeeper.

**Status: pre-prototype.** Nothing here is stable, and per the founding
philosophy nothing *may* be declared stable until Bet 5's pre-registered
verdict is in (see below).

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
  BET5_CRITERION.md  pre-registered kill criterion
  design/            numbered design documents (0001-..., 0002-...)
prototype/           Bet 5 validation prototype (throwaway syntax; checker + interpreter)
```
