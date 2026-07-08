# Task format (P19 external-anchor tasks)

Each task is one `*.json` file in this directory. The harness loads every file,
sorts by `id`, and enforces unique ids. Fields:

| field                        | type                          | meaning |
|------------------------------|-------------------------------|---------|
| `id`                         | string                        | unique task id; also the temp-file and report key |
| `category`                   | `"generate" \| "repair" \| "explain"` | task shape (`explain` reserved — see README) |
| `title`                      | string                        | one-line human label |
| `statement`                  | string                        | the task text handed to the model (with the exact required signatures) |
| `submission_filename`        | string                        | the file the model produces; matched inside the submission dir |
| `prompt_material.spec_pack`  | [string]                      | the spec-pack files the operator loads as context (manifest only; not read for scoring) |
| `prompt_material.given_program` | string?                    | repair only: path (relative to root) to the buggy source shown to the model |
| `prompt_material.given_diagnostic` | object?                 | repair only: the ACTUAL `candor-proto check` JSON diagnostic for that bug |
| `anchor.kind`                | `"run_sentinel" \| "check_pass" \| "diagnostic_resolved"` | how the submission is accepted |
| `anchor.battery_file`        | string?                       | generate only: the hidden test-harness `main` appended to the submission (relative to root) |
| `anchor.expected_sentinel`   | string?                       | the trimmed `run` stdout a correct submission must emit |

## The anchor is hidden

`prompt_material` is everything the operator gives the model. `anchor` is the
acceptance criterion and is NEVER shown. For a generate task the model writes the
item(s) named in `statement`; the harness concatenates the submission with
`battery_file` (a `main` authored FRESH against the basket spec) and scores
`check` then `run`. For a repair task the submission is a full program; it is
scored directly for a clean `check` plus the `expected_sentinel`.

## Scoring rules (by `anchor.kind`)

- **`run_sentinel`** — assemble `submission + battery_file`; pass iff `check`
  exits 0 AND `run` emits `expected_sentinel`.
- **`check_pass`** — assemble as above; pass iff `check` exits 0. (Defined for
  completeness; the seed set does not use it.)
- **`diagnostic_resolved`** — the repair rule; the submission alone must `check`
  clean AND `run` emit `expected_sentinel` (behaviour unchanged by the fix).

## Failure stages (in the report)

`parse` (P0xxx/lexer) · `check` (E-codes) · `run` (compiled but faulted) ·
`wrong-sentinel` (ran, wrong output) · `missing` (no submission file).
