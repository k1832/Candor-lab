# corpus-gen — the P19 synthetic-corpus pipeline seed

The third and final P19 artifact for Candor (after the spec pack and the
evaluation harness): **spec-grounded synthetic corpus generation, filtered by the
toolchain, regenerated per edition** (LANG_PHYLOSOPHY.md P19; Bet 6). It produces
labelled `(program, expected)` `.cnr` samples for training the project's own model
authors in the language.

It depends only on `serde`/`serde_json`. Generation is grammar-directed; every
keep/reject decision is delegated to the `candor-proto` toolchain — the generator
never grades its own output.

## Bet 6 — the circularity guard (READ THIS FIRST)

**The corpus is TRAINING material. It is never an evaluation anchor.**

Toolchain filtering validates **internal consistency** — a positive sample
*compiles and `run`s to the sentinel the generator computed*; a negative sample
*makes `check` emit exactly the diagnostic the shape was authored to trip*. That
is all it proves. It does **not** prove the corpus is *correct*, *idiomatic*, or
*representative*: self-generated programs filtered by the same toolchain that will
grade the model would measure nothing (Bet 6; P19 §"the circularity is guarded").

The separation is therefore strict and one-directional:

- **This pipeline feeds training only.** Its output must never be promoted into an
  eval set.
- **The evaluation anchors stay external and are NEVER sourced from here:** the
  human-ported Bet 5 workload basket, independently authored tasks, and real
  programs ported by people (`tools/eval-harness/`, whose anchors are authored
  from the frozen basket specs — not model- or pipeline-generated).

A generated program passing the filter means "internally consistent with the
toolchain," not "matches an externally fixed intent." Only the external anchors
carry the latter meaning.

## What it is

- **`src/shapes.rs`** — the shape library: 25 hand-authored generators (10
  positive, 15 negative), one family per idiom in `docs/specpack/idioms.md`
  (enum+match+`?`, struct+drop+trace, generic fn + `copy` bound + instantiations,
  borrow/reborrow chains, arena/index over `[N]T`, wrapping arithmetic, `for` over
  the Indexed protocol, ...). Each is parameterised from the seeded RNG; positive
  shapes compute their own sentinel, negative shapes target one diagnostic code.
- **`src/oracle.rs`** — the filter: a thin wrapper over `candor-proto check`/`run`.
- **`src/lib.rs`** — the generation driver: round-robin over the shape registry,
  filter each candidate, keep to target, tally kept/rejected per shape.
- **`corpus-seed/`** — a committed 200-sample seed run (`positive/`, `negative/`,
  `manifest.json`), generated with the real toolchain as filter. Small enough for
  the repo; the real pipeline emits far larger sets out-of-tree.
- **`tests/`** — determinism, filter integration, manifest validity.

## Output

```
<out>/positive/pos_NNNN_<shape>.cnr      # compiles + runs to a known sentinel
<out>/negative/neg_NNNN_<shape>.cnr      # trips exactly one diagnostic
<out>/manifest.json                       # per-sample record + per-shape tallies
```

Each `manifest.json` sample records `shape`, `seed`, `draw` (its point in the
draw stream), `params`, `expected` (`sentinel` value **or** `diagnostic` code),
`observed` (what the toolchain reported), and `toolchain_version`. The top-level
`stats` array carries per-shape `kept`/`rejected` counts — **the reject rate is a
grounding signal**: a shape whose rejects are high has drifted from the spec (its
generator predicts behaviour the toolchain does not confirm) and needs fixing, not
the toolchain. At the seed run all shapes filter at 100% kept, which is the
well-grounded outcome; the mechanism that would reject a drifted shape is proven by
`tests/filter.rs`.

## Determinism contract

Same `--seed`, same toolchain ⇒ **byte-identical corpus** (files and manifest).
The only source of variation is the seed: randomness is a seeded xorshift64 stream
(`src/rng.rs`) with **no `Date`, no environment read, no OS RNG**. `candor-proto`
`check`/`run` are themselves deterministic, so the keep/reject decision is a pure
function of the program and the toolchain version. The manifest carries no
timestamp. `tests/determinism.rs` asserts the byte-identity; `corpus-seed/` is
reproducible with `generate --seed 1 --positive 150 --negative 50`.

The recorded `toolchain_version` is a fixed prototype stamp
(`candor-proto 0.1.0 (Bet 5 prototype)`; there is no `candor-proto --version`);
the real pipeline stamps the compiler's version/commit and treats a version change
as a regeneration trigger.

## Regeneration per edition

Editions cost corpus, and the migrator does not pay that bill (P15;
LANG_PHYLOSOPHY.md §"Editions cost corpus"). This pipeline is the payer: on every
edition (or any language/toolchain change that moves a diagnostic code or a
sentinel), **re-run generation against the new toolchain**. Because the filter is
the toolchain itself, a shape that no longer matches the language starts *rejecting*
— the reject rate surfaces the drift automatically, and the shape is updated in
`src/shapes.rs`. The spec pack (`docs/specpack/`) is the shapes' source of truth.

## Scope (honest limits)

- **Hand-authored shapes, not spec-exhaustive.** 25 generators over the idiom
  catalogue — a seed, not the whole grammar. Coverage grows by adding shapes to
  the library, not by scaling any single generator.
- **Single-file samples.** The seed targets programs the prototype both `check`s
  and `run`s single-file; multi-file module-pair shapes (which the prototype
  checks but cannot yet run end-to-end) are future work as the runner matures.
- **Sentinel-and-code filtering only.** Positives are pinned by the `i64` return
  sentinel and negatives by the first diagnostic code; the drop-order `trace` a
  `struct_drop_trace` sample documents in `params.expected_trace` is not asserted
  by the filter (the CLI `run` surfaces only the return value).
- **Grounding, not idiom quality.** The filter proves internal consistency; it does
  not certify a sample is the *idiomatic* way to express its intent. That judgement
  is out of scope by construction (Bet 6).

## Running

```
# build the toolchain filter once
cargo build --release --manifest-path ../../prototype/Cargo.toml

# generate (the seed corpus)
cargo run -- generate --seed 1 --count 200 \
    --candor ../../prototype/target/release/candor-proto --out corpus-seed

# --count splits 3:1 positive:negative by default; override with
#   --positive P --negative Q. --candor also reads $CANDOR_PROTO.

cargo test        # determinism, filter integration, manifest validity
```
Exit 0 on success, 2 on a usage/config error, 1 if a shape is so misgrounded the
target could not be reached within the attempt budget.
