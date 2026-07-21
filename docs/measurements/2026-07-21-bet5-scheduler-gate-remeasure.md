# Bet 5 — scheduler re-port re-measurement, at the 1.0 syntax-freeze gate

**Class:** MEASUREMENT under a FROZEN criterion (`docs/BET5_CRITERION2.md`, ratified
and frozen 2026-07-08). The criterion, unit table (`docs/BET5_UNIT_TABLE.md`), spec
(`docs/basket/spec-scheduler.md`), and counting tools are unchanged; nothing here
retrofits the ruler. **Date:** 2026-07-21. **Author:** Bet-5 gate re-measure session.
Publish-either-way discipline (criterion §8.1): the numbers below are recorded as the
frozen tools compute them, whatever the direction.

## Why this record exists

`docs/1.0-GATE-TRIAGE.md` row 6a carries the binding v4.2 commitment (philosophy
Appendix A, `docs/RESULTS.md` §"§9 review ruling — 2026-07-08"): *"the scheduler is
re-ported and re-measured under the frozen successor rules before any syntax freeze;
if the re-measurement worsens, the confirmation returns to review."* A syntax freeze
precedes 1.0. The re-port and its first re-measurement were performed and recorded on
2026-07-08 (`ports/candor/scheduler-v2/`, commit `af47102`; discharge commit `da6c7d0`).
This record is the **at-gate re-confirmation**: it re-verifies, under the *current*
(further-hardened) compiler and the *current* frozen counters, that the 2026-07-08
re-measurement still holds — the confirmation the "before any syntax freeze" clause
requires at the moment the freeze becomes due. It does not disturb the frozen
2026-07-08 artifacts (`docs/measurements/ports/scheduler-v2.json`,
`docs/measurements/ports/scheduler.v2.json`), which remain the frozen evidence.

## Artifacts measured (both frozen; not re-authored at the gate)

- **Original port** (frozen evidence): `ports/candor/scheduler/scheduler.cn`.
- **Re-port** (the v4.2 commitment artifact, authored 2026-07-08 under designs
  0004/0005): `ports/candor/scheduler-v2/scheduler.cn`. Per criterion §9.1's spirit,
  no new port was authored at the gate (an artifact authored after the verdict is
  known is unauditable); the frozen re-port is re-measured, per §7.1's procedure of
  re-running the frozen counters on the frozen artifacts.
- **Rust baseline** (frozen at `b689860`, §7.1): `baselines/rust/scheduler/src/`.

## Post-confirmation features the re-port uses (the experiment)

Both design consequences recorded at Bet 5's confirmation shipped into the compiler
(`compiler/src/`), verified present this session:

- **Design 0004 — safe typed field projection `field_ptr` (E0510), a SAFE op.**
  Used at **11 sites** in the re-port (0 in the original). Its measurable effect is
  exactly the a-priori philosophy-tier ruling's prediction (`docs/ADJUDICATIONS.md`,
  2026-07-08): `t_link` — whose whole `unsafe` body was pure forward projection —
  becomes `return field_ptr(t, link);`, leaving the valve entirely. `task_of`
  (inverse container_of, a negative offset to a non-field type) correctly stays a
  genuine valve, as 0004 states `field_ptr` cannot express it.
- **Design 0005 — implicit call-site reborrow.** The re-port is written in the
  canonical bare form; explicit `read/write (deref b)` reborrow ceremony is reduced.
  0005 is measurement-neutral by construction (use-site tokens, excluded from M1).

## Functional gate (same frozen vectors as the original)

Toolchain: `compiler/target/release/candor` (rebuilt this session, `cargo build
--release`). Full frozen suite T1–T20 incl. the T19 20,000-step shadow-compared stress.

| Port | `candor check` | `candor run` (T1–T20) |
|---|---|---|
| original `scheduler/scheduler.cn` | exit 0 | **sentinel 777**, exit 0 |
| re-port `scheduler-v2/scheduler.cn` | exit 0 | **sentinel 777**, exit 0 |

Both pass all vectors under the current compiler.

## Instrument-stability check (the frozen counters reproduce the frozen JSONs)

Measured section = implementation above the first column-0 `// Test harness` line
(ruling R14); `candor count` on that section; `rust-count` per baseline `src/` file.

| Quantity | original | re-port | frozen JSON says |
|---|---|---|---|
| `valve_statements` (candor) | 47 | 45 | 47 / 45 — reproduced |
| `logical_statements` | 173 | 172 | 173 / 172 — reproduced |
| `valve.lines` / `total_lines` | 89 / 216 | 83 / 211 | reproduced |
| `valve.functions` / `total_functions` | 21 / 24 | 21 / 24 | reproduced |
| annotation a/b/c/d total | 51 | 50 | reproduced |
| Rust baseline `valve_statements` (Σ src/) | — | — | **89** — reproduced (2+2+76+2+0+3+4) |

The current frozen tools reproduce every recorded figure exactly; the instrument is
stable across the toolchain hardening since 2026-07-08.

## Scores under the frozen successor criterion

M1/M1b/M4/M5 carry over as already computed (criterion §6.1; all pass). The only
re-measured metric is the scheduler's valve/annotation content.

| Metric (frozen def) | original port | **re-port** | Rust baseline | frozen threshold | direction |
|---|---|---|---|---|---|
| **V1 `R_valve` = V_c/V_rust** (operative gate, §4.2) | 47/89 = **0.5281** | 45/89 = **0.5056** | — | KILL > 1.25 / WARN > 1.00 → **PASS** | **improved** |
| annotation density (M1) = ann/stmt | 51/173 = 0.2948 | 50/172 = **0.2907** | 136/265 = 0.5132 | pass (far below bar) | improved |
| valve-line fraction (v1 ruler, retired §4.4-era) | 89/216 = **0.4120** | 83/211 = **0.3934** | — | old 0.40 home-ground ceiling | improved — now **below** the old ceiling |
| valve-fn fraction (retired to reported-only, §4.4) | 21/24 = 0.8750 | 21/24 = 0.8750 | 35/70 = 0.5000 | not gating | unchanged |
| value copies (M4) | 0 | 0 | 1 | no WARN | unchanged |

## Verdict — per the frozen criterion and the binding commitment

**IMPROVED.** On the operative gating metric (V1 `R_valve`) the re-port moves
0.5281 → **0.5056** — a PASS with wide margin (≤ 1.00, no WARN; far below KILL 1.25).
It also improves annotation density (0.2948 → 0.2907) and the retired v1 valve-line
fraction (0.4120 → 0.3934, crossing below the old 0.40 breach), and is unchanged on
the retired valve-fn fraction (0.875) and on copies (0). **On no frozen metric does
the re-measurement worsen.**

**Consequence:** the commitment's return-to-review trigger — *"if the re-measurement
worsens, the confirmation returns to review"* — **does not fire.** Bet 5's provisional
confirmation (philosophy v4.2) **stands**.

**Home-ground routing (unchanged, for completeness).** The scheduler remains a
home-ground, **baseline-sensitive** program under §6.6: registered `R_valve` 0.5056
(pass) vs authored-only 45/2 = 22.5 (KILL) still diverge, routing to the standing
mandatory §6.4 review exactly as at the 2026-07-08 confirmation. This routing is not a
worsening (the registered/measured content improved); it is the same status the v4.2
ruling already dispositioned (authored-only gap owned by Bets 4/6 as an
ecosystem-bootstrap fact, not a memory-model defect).
