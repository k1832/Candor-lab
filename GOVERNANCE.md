# Governance

This document is the subordinate governance charter required by
[LANG_PHILOSOPHY.md](LANG_PHILOSOPHY.md) §9. It specifies who the deciding
authority is, how amendments are enacted, and how succession works. Where this
document and the philosophy conflict, the philosophy wins.

## Deciding authority

The deciding authority is **k1832** (repository owner), an individual, acting
with LLM assistance. The authority owns the default *no*: deprecating second
ways (P3), keeping the effect set closed (P2/NN#19), owning the formatter's
output, funding migrators (P15), and treating emerging parallel-variant
ecosystems as a P3 violation.

## Amendments

Per philosophy §9, an amendment to the philosophy must:

1. Name the principle or non-negotiable it changes.
2. State the evidence or argument that overcame the burden of proof.
3. Append the change and rationale to the philosophy's Appendix A ledger.

Amendments are enacted as commits touching `LANG_PHILOSOPHY.md`, authored or
approved by the deciding authority. Silent reinterpretation is forbidden; a
conflict discovered in a lower-tier artifact (design doc, implementation)
resolves upward — the lower artifact changes or the philosophy is amended.

**Non-Negotiable amendments are slowed**: the proposal, rationale, and
evidence must be published for open comment (as a GitHub issue or pull
request on this repository) for at least **14 days** before enactment, and
the enacted amendment records the objections alongside the decision. While
the project is pre-publication and has no external commenters, the comment
period still applies — it forces the delay that guards against impulse, and
the record is written as if readers exist, because eventually they will.

## Pre-registered targets

Missed pre-registered targets (Bet 5, Bet 6, P20 compile-time targets) are
review triggers that must be enacted as amendments, not absorbed as silence.
The Bet 5 criterion lives in [docs/BET5_CRITERION.md](docs/BET5_CRITERION.md);
once basket porting begins, its thresholds are frozen and may not be changed.

## Design decisions below the amendment threshold

Decisions that do not touch the philosophy are recorded as numbered design
documents in `docs/design/`, following
[docs/design/TEMPLATE.md](docs/design/TEMPLATE.md). Every design document
records rejected alternatives next to what was chosen (philosophy §8.6) —
the rationale is the product.

## Succession

Defined before it is needed, per §9: if the deciding authority becomes
permanently unavailable, authority passes to whoever the authority has named
in writing in this file (currently: no successor named), else to the
maintainer with commit access of longest standing. A successor inherits the
full burden of this charter, including the amendment ledger discipline.
