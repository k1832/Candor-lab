# Bet 5 freeze manifest

Records the artifact hashes required by BET5_CRITERION.md §6.2. Each freeze step appends its
section here when it completes; the criterion's §0.4 ledger references this file. Hashes are
SHA-256 over file contents at the named git commit.

## Step (i) — COMPLETE, 2026-07-07

Base commit: `0e8c6fe06633a8a1a6b300a76c77db1b551a5225`

Precondition discharged: the checker-soundness argument (design 0003) was reviewed and retested
by four independent sessions (docs/reviews/2026-07-07-soundness-review-{1,2,3,4}.md); retest #4
returned ACCEPTABLE as-is. Per criterion §3.1, the prototype's memory-model syntax is FROZEN as
of this manifest; grammar changes past this point require a pre-freeze ledger amendment while
§0.1 still permits one.

| Artifact | SHA-256 |
|---|---|
| docs/BET5_CRITERION.md (v2) | 763a0be79005d20ba5d10e4ea6aa8da94f380ec355ba340ec3065bd3a3c6617e |
| docs/BET5_UNIT_TABLE.md (table_version 1) | 79b22249fb2be5f6bd62a5d5b2a5fee73ed15df303375830107d0377c60e9b3b |
| docs/design/0001-memory-model.md | 76517ae69474e1cfdb3928131515121007968ae09c5eb033710acdcb10271b0b |
| docs/design/0002-prototype-grammar.md | a33e3685b3450e8fc2235084411adf05940af5a40f20ff247e1e1790a06b37c5 |
| docs/design/0003-checker-soundness.md | 4e502375789a1fa6051f61870746ecd9615e1e8e2e3cff439e6b722bd7a0f1e4 |
| prototype/src/count.rs (Candor counter) | 00523ebad1608df580595a8fbe79f2605466a8bf6abcb8d04c625bbc72b7efb5 |
| tools/rust-count/src/lib.rs (Rust counter) | 13bf94d86057c563bd8cba8a041cf9f4c177127db69062f3cbb617ff81ec1386 |

Note on the M6 model set: M6 was demoted to supplementary, non-gating evidence by ledger row 11,
so no model set is required at this step.

## Step (ii) — specs authored, reviewed, and provisionally published

The five language-agnostic basket specs exist, blind-authored and blind-reviewed
(docs/reviews/2026-07-06-basket-specs-review-1.md).

**Publication ruling (deciding authority, 2026-07-07):** provisional publication by annotated git
tag (`bet5-specs-v1`) plus the hashes below, while the repository remains private. The repository
MUST become public before the freeze instant (criterion §0.1); until then, the criterion's
open-comment and publish-either-way provisions are honored as written-for-the-public-record but
have no external witnesses — stated plainly rather than pretended otherwise. Baseline selection
may proceed against the tagged spec hashes.

**Baseline-production ruling (deciding authority, 2026-07-07):** hybrid per program — allocator
and scheduler adapted from independently-sourced open-source Rust (sources and commits recorded
in criterion §6.6 when frozen); MMIO, parser, and arena commissioned from Opus-family model
sessions blind to the Candor design documents. Consequence, binding under criterion §6.3: the
Candor basket ports may NOT be authored by Opus-family models; they will be authored by a
different model family, recorded in §6.6 alongside the baselines.

| Artifact | SHA-256 |
|---|---|
| docs/basket/spec-allocator.md | 95fc2365331069a8ec0e85a023ae88de6f1c0f5a48e5516d7d64490626e3950f |
| docs/basket/spec-arena.md | a32971ce51617bf2d4e1c9caf075125f569694016e29c246c08c9a93bd9046bd |
| docs/basket/spec-mmio.md | 3ba13588d27e6e14877468339c55c4be92f66cedf479eb26c9712378710b1231 |
| docs/basket/spec-parser.md | 86947fcc090e9fe1c930c40ac58c35d74dd2a66c4c697d1a90e7f673ce92c0a0 |
| docs/basket/spec-scheduler.md | d3433acfb0a6fc57a087986f185c652369ab92971e9d21afd7b906c833745ce9 |

## Step (iii) — Rust baselines: NOT STARTED

Baseline sources, commit hashes, adjudicated idiomaticity, and measured valve fractions are
recorded in criterion §6.6 when chosen. The freeze instant (criterion §0.1) is the first commit
of any Candor basket port after step (iii) completes.
