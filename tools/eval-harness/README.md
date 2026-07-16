# eval-harness — the P19 evaluation-harness seed

The third P19 artifact for Candor (after the spec pack; the synthetic-corpus
pipeline is deferred per §8 sequencing). It makes **model competence in Candor a
scored, first-class quality metric of every release** (P19), measured on
**external anchors** and reported as **first-attempt correctness** and the
substrate for a **competence slope** (Bet 6).

The harness **defines and scores tasks**. It **does not drive a model** — that is
the operator's side (documented below). It has no model-API calls and depends
only on `serde`/`serde_json`; every correctness judgement is delegated to the
`candor-proto` toolchain (the oracle), never re-implemented here.

## What it is

- **`tasks/*.json`** + `tasks/SCHEMA.md` — 12 externally-anchored tasks.
- **`anchors/`** — the hidden acceptance batteries (generate) and the buggy
  programs + captured diagnostics (repair).
- **`src/`** — `eval-harness score <submission_dir>`: matches each submission to
  its task, runs it through `candor-proto`, and emits a JSON report.
- **`tests/`** — the runner scored against 3 known-good and 3 known-defective
  submissions (`cargo test`).

## Task-set composition (23 tasks (12 seed + 11 graduation))

**8 generation tasks**, derived from SMALL sub-problems of the frozen basket
specs (`docs/basket/`), each with a `main` authored FRESH against the spec text:
`align_up` (allocator), a bounded arena push/get (arena), `Opt::unwrap_or`,
`checked_sub` returning a result-shaped `Opt`, `min_i64`, `Res` + `?`
propagation, a `saturating` u8 add (mmio/arithmetic regime), and cross-type `?`
via a `From`-widening impl. The model submits the item(s); the hidden battery is
appended and the program is scored `check` then `run`-sentinel.

**4 repair tasks**, each a corpus-shaped program with ONE injected bug and the
ACTUAL `candor-proto` JSON diagnostic it emits: a use-after-move (**E0301**),
path-dependent init at a drop point (**E0309**), a missing `alloc` effect marker
(**E0401**), and a conflicting exclusive borrow (**E0801**). The model submits a
repaired program; the anchor is: compiles clean + sentinel unchanged.

## Scoring stages

Every task result carries pass/fail and, on failure, the stage:
`parse` · `check` · `run` (compiled, then faulted) · `wrong-sentinel` (ran, wrong
output) · `missing` (no submission). The report aggregates
`first_attempt_rate = passed / total`, split by category. Each failure also
carries `feedback_diagnostic` — the machine-readable diagnostic/fault — which is
the **repair-loop hook**: the operator hands it back to the model and re-scores
with `--round 2` for the slope.

## How the operator runs a real measurement

The seed scores submissions; producing them is the operator's loop:

1. Load the `prompt_material` (the spec-pack files, and for repair the buggy
   program + its diagnostic) plus the task `statement` into the model under test.
2. Collect the model's outputs into a submission directory, one file per task's
   `submission_filename`.
3. `eval-harness score <dir> --candor <path-to-candor-proto> --round 1` → the
   round-1 report (first-attempt rate).
4. For each failed task, feed `feedback_diagnostic` back to the model, collect
   round-2 outputs, and `score ... --round 2`. The round-over-round pass-rate
   delta is the competence-slope substrate.

A real measurement run therefore requires only: a built `candor-proto`, a model
driven by the operator over the prompt material, and (for the slope) at least two
rounds or two model releases scored against this same frozen task set.

## Bet 6 anti-circularity note

The anchors are authored FROM the frozen basket specs and the spec pack — the
external ground truth — and are **never model-generated**. Self-generated tests
grading self-generated code measure nothing (Bet 6); a battery `main` written
against the spec, and a diagnostic captured from the real toolchain, are
independent of whatever wrote the submission. The oracle (`candor-proto`) filters
for internal consistency; the *anchors* are what make a pass mean "matches an
externally fixed intent."

## What this seed does NOT measure (honesty)

- **No model is driven.** The harness defines and scores; generation/repair is
  the operator's side. There are no model-API calls.
- **No adaptation-budget accounting.** P19's efficiency metric (correctness *per
  unit of adaptation budget* — fine-tuning tokens/examples) is not tracked here;
  the report gives a raw first-attempt rate and a round label only. Budget
  accounting is future work.
- **The task set is small and hand-authored.** 12 tasks over SMALL sub-problems,
  hand-distilled, not pipeline-generated; it can drift as the language changes.
- **Anchors are behavioural.** A `diagnostic_resolved` or `run_sentinel` anchor
  checks compile-clean + sentinel; it does not prove the fix is idiomatic or that
  a sentinel was reached for the right reason (a hardcoded return could pass a
  weak battery). Batteries use multiple asserts to narrow this, but it is a known
  limit — idiom quality is not scored.
- **`explain` is reserved, not seeded.** Grading free-form explanations is the
  circularity trap; the seed contains only the mechanically scorable `generate`
  and `repair` kinds.

## From the seed to graduation

These tasks are the SMALL sub-problems. The basket's FULL programs
(`docs/basket/spec-{allocator,arena,mmio,parser,scheduler}.md`, realized in
`compiler/tests/fixtures/run/11_*.cnr`) become the **graduation tasks** later:
the same runner shape (spec + statement as prompt material; a fresh acceptance
main or the program's own `main` sentinel as the anchor; `check`/`run` scoring),
scaled from a single function to a whole ported program. The corpus pipeline,
when funded against Bet 1 evidence, feeds the same anchors.

## Running

```
# build the oracle once
cargo build --manifest-path ../../compiler/Cargo.toml

# score, pointing at the built oracle
cargo run -- score tests/submissions_good \
    --candor ../../compiler/target/debug/candor-proto --round 1

cargo test        # runner scored vs known-good / known-defective submissions
```
`--candor` also reads `$CANDOR_PROTO`. Exit 0 iff every task passed (a CI gate),
1 if any failed, 2 on a configuration error.
