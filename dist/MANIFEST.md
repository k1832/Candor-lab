# Distribution manifest — what ships, what stays in the lab

This file is the curation the operator executes when seeding the standalone
`candor` distribution repository from this lab (ROADMAP.md, publication step 2).
The lab repository is, permanently, the full experiment record; the distribution
is a separate, user-facing surface that *links back* to the lab as its design
record and authority. Nothing below is a copy of the lab's identity — the
philosophy, the reviews, the Bet 5 experiment, and the designs are never diluted
into the product repo.

Paths are given relative to the lab root.

## SHIPS — included in the distribution repo

| What | Lab source | Notes |
|---|---|---|
| **Toolchain source** | `compiler/` (the Rust crate) | The compiler/interpreter/AOT backend. The crate's `[[bin]]` is `candor`. Ship `src/`, `Cargo.toml`, `Cargo.lock`, `benches/`. |
| **Normative spec** | `docs/spec/` | Chapters 00–12 + `99-obligations.md`. The reference. |
| **Spec-pack** | `docs/specpack/` | The model-facing distillation (grammar, semantics, idioms, diagnostics). |
| **Standard-library seed** | `compiler/tests/fixtures/corelib/` (`core/*`, `std/*`) | Relocate to a first-class `stdlib/` (or `lib/`) path, not a test fixture. `Opt`/`Res`/`Arena`/`iter` + `alloc`/`bump`/`list`. |
| **Editor tools** | `tools/vscode-candor/`, `tools/candor-lsp/` | TextMate grammar + diagnostics LSP. |
| **Getting-started + examples** | `dist/README.md`, `dist/INSTALL.md`, `dist/LANGUAGE-TOUR.md`, `dist/examples/` | This staging tree becomes the distribution root. |

## STAYS LAB-ONLY — never copied; linked as the design record

| What | Lab source | Why it stays |
|---|---|---|
| **Founding philosophy** | `LANG_PHILOSOPHY.md`, `GOVERNANCE.md` | The normative authority and its governance — the lab's distinctive asset. Linked, not copied. |
| **Design documents** | `docs/design/` (0001–0012) | The record of what was decided and, per philosophy §8.6, what was rejected and why. |
| **Adversarial reviews** | `docs/reviews/` | The soundness-hole record. Lab methodology, not product surface. |
| **Bet 5 experiment record** | `docs/BET5_CRITERION.md`, `docs/BET5_CRITERION2.md`, `docs/BET5_UNIT_TABLE.md`, `docs/RESULTS.md`, `docs/ADJUDICATIONS.md`, `docs/FREEZE_MANIFEST.md`, `docs/basket/`, `docs/proposals/`, `baselines/`, `ports/` | The frozen founding experiment and its measurement basis. |
| **Measurements** | `docs/measurements/` | Raw counter output; a lab instrument. |
| **Eval / corpus lab tools** | `tools/eval-harness/`, `tools/corpus-gen/`, `tools/rust-count/` | Measurement and corpus-generation tooling — lab-internal, explicitly NOT shipped. |
| **Lab meta** | `ROADMAP.md`, `MEMORY.md`, `README.md` (the lab README), `GOVERNANCE.md` | Experiment-facing. The distribution has its own `README.md` (this tree). |

## Operator actions (the seeding step)

1. Create the standalone `candor` repo (this is the operator's git action, not
   done here).
2. Copy in only the **SHIPS** rows above; take `dist/`'s contents as the repo root
   (`README.md`, `INSTALL.md`, `LANGUAGE-TOUR.md`, `examples/`).
3. Relocate the stdlib seed out of `compiler/tests/fixtures/corelib/` into a
   first-class library path.
4. Add the back-links to the lab repository (design record, philosophy, Bet 5
   criterion) wherever the docs reference "the lab."
5. Do NOT copy any **STAYS LAB-ONLY** path.
