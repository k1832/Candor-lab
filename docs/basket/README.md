# Bet 5 Validation Basket — Frozen Functional Specifications

This directory holds the five **frozen, language-agnostic functional
specifications** for the Bet 5 validation basket defined in
`../BET5_CRITERION.md` §2. Each spec defines one basket program purely by its
externally observable behavior and its frozen test suite; per §2.3, "the
program" is *the artifact that passes that suite*, independent of language.

The five specs:

- `spec-allocator.md` — general-purpose allocator over a caller-provided region.
- `spec-scheduler.md` — intrusive-list priority scheduler.
- `spec-mmio.md` — driver-like state machine over a simulated MMIO device.
- `spec-parser.md` — recursive-descent parser producing a typed AST.
- `spec-arena.md` — arena-based IR transformation pass.

## Blindness provenance

These specs were authored in a session that read **only**
`../../LANG_PHYLOSOPHY.md` and `../BET5_CRITERION.md`. The session never saw any
Candor design document, memory-model document, or prototype code. This is the
`BET5_CRITERION.md` §6.2 finding-8 requirement: the specs must be authored blind
to the Candor design so they cannot be shaped to advantage Candor, and so that
"we ported an easier version" is mechanically detectable (§2.3) — the easier
version fails the frozen suite.

## Place in the freeze order

Per `BET5_CRITERION.md` §6.2 these specs are **step (ii)**: authored in sessions
blind to the Candor design docs, committed, hashed, and **published before any
Rust baseline is chosen** (step iii) and before any Candor port begins (the
freeze instant). Step (i) — the criterion, classification/unit table, counting
scripts, and checker-soundness argument — precedes them.

## Author-independence rule

Per §6.2 and §6.7, **spec authors may not author Candor ports**. Session
blindness is the solo-project approximation of author independence. Any change to
a frozen spec after its hash is recorded is a public ledger event (§0, §6.1).
These files are written to be final.
