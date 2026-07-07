# Bet 5 running results

Live scoring record against the frozen BET5_CRITERION.md (v2, frozen at the freeze instant
`e22ca51`). Measurements per the frozen unit table; artifacts split per ruling R14; adjudication
in docs/ADJUDICATIONS.md. This file records outcomes as they land, per the publish-either-way
commitment — including unfavorable ones.

| Program | M5 complete | Valve-line | Valve-fn | Ceiling (M2) | Ann/stmt (Candor vs Rust) | M2 verdict |
|---|---|---|---|---|---|---|
| Allocator | YES — 22/22 vectors, sentinel 777, 21m20s | **0.6292** | 0.6250 | max(0.40, 1.25×0.1929)=0.40 | 0.0617 vs 0.2943 | **KILL threshold breached** |
| Intrusive scheduler | YES — 20/20 vectors, sentinel 777, ~25s | **0.4120** | 0.8750 | max(0.40, 1.25×0.1489)=0.40 | 0.2948 vs 0.5132 | **KILL threshold breached** |
| MMIO state machine | pending | | | 0.15 abs | vs 0.1667 | |
| Parser | pending | | | 0.15 abs | vs 0.4150 | |
| Arena pass | pending | | | 0.15 abs | vs 0.2051 | |

## Allocator — recorded 2026-07-07

The hardest program completed in Candor (M5 pass) with **dramatically lower annotation load than
the Rust baseline** (0.0617 vs 0.2943 units/statement — a 79% reduction, far past M1's pass bar)
— and with a **valve-line fraction of 0.6292, breaching the frozen home-ground KILL ceiling of
0.40**. No cell-substitutable regions exist to invoke §4.2's relief path (ruling R15), so under
the frozen decision rule §5.1(b) this is a standing KILL condition, to be enacted with the full
basket's evidence after all five ports are attempted (§6.4 requires all five regardless).

## Scheduler — recorded 2026-07-07

Completed (M5 pass, T1-T20 incl. the 20k-step stress; suite ~25s). Annotation load 0.2948 vs
Rust 0.5132 units/statement — a 43% reduction. Valve-line fraction **0.4120 vs the 0.40
ceiling — the second home-ground KILL breach, by 0.012**. Zero cell-substitutable regions
(ruling R19); idiomaticity confirmed, intrusiveness verified in the measured artifact. Port
friction notes (README): rawptr's lack of safe field projection puts ~10 one-line accessors in
the valve; explicit call-site reborrows are a steady tax; one prototype parser bug worked around
with defensive semicolons (recorded as a known prototype issue, not fixed mid-experiment).

## Measurement observation (both home-ground programs)

Observation recorded for the eventual §9 proceeding, per §0.3 (defects are published, not
patched): the valve-line *fraction* is sensitive to total program size. The Candor port
implements the same spec in 178 logical statements against the Rust baseline's ~930 lines; a
denser program with the same absolute quantity of pointer code shows a higher fraction. The
absolute valve content of the two allocators is comparable (Candor 112 valve lines; Rust ~150);
the fraction differs mostly because Candor needed far less safe scaffolding. Whether that means
"the valve is the program" (the criterion's intent for the 0.40 ceiling) or "the denominator
shrank" is exactly the kind of question §0.3 reserves for the public amendment, not for silent
reinterpretation. The number is recorded as the frozen rules compute it.
