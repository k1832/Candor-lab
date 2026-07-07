# Bet 5 running results

Live scoring record against the frozen BET5_CRITERION.md (v2, frozen at the freeze instant
`e22ca51`). Measurements per the frozen unit table; artifacts split per ruling R14; adjudication
in docs/ADJUDICATIONS.md. This file records outcomes as they land, per the publish-either-way
commitment — including unfavorable ones.

| Program | M5 complete | Valve-line | Valve-fn | Ceiling (M2) | Ann/stmt (Candor vs Rust) | M2 verdict |
|---|---|---|---|---|---|---|
| Allocator | YES — 22/22 vectors, sentinel 777, 21m20s | **0.6292** | 0.6250 | max(0.40, 1.25×0.1929)=0.40 | 0.0617 vs 0.2943 | **KILL threshold breached** |
| Intrusive scheduler | YES — 20/20 vectors, sentinel 777, ~25s | **0.4120** | 0.8750 | max(0.40, 1.25×0.1489)=0.40 | 0.2948 vs 0.5132 | **KILL threshold breached** |
| MMIO state machine | YES — M1-M10 byte-exact, sentinel 777, ~0.02s | 0.0469 | **0.2500** | 0.15 line / 0.20 fn abs | 0.1183 vs 0.1667 | **KILL threshold breached (fn fraction)** |
| Parser | YES — P1-P32, sentinel 777, ~0.03s | 0.0408 | 0.1000 | 0.15 line / 0.20 fn abs | 0.1768 vs 0.4150 | **pass** |
| Arena pass | YES — 29/29 vectors, sentinel 777, ~25ms | **0.0000** | 0.0000 | 0.15 abs | 0.0991 vs 0.2051 | **pass** |

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

## Arena — recorded 2026-07-07

Completed (M5 pass, AR1-AR29, ~25ms). Zero valve regions — even the backing array needed no Box,
so §11.5's "valve sealed inside the backing" never arises. Annotation 0.0991 vs Rust 0.2051
units/statement (−52%). M2: 0.0000 vs the 0.15 value-favorable ceiling — clean pass. Friction
notes in the port README (reborrow ceremony in recursive walkers; no i64::MIN literal; `out` is
a reserved word). Rulings R21-R24.

## MMIO — recorded 2026-07-07

Completed (M5 pass, M1-M10 byte-exact incl. recovery scenarios; suite ~0.02s). Annotation 0.1183
vs Rust 0.1667 (−29%). Valve-line 0.0469 — well under the 0.15 ceiling. Valve-FUNCTION 2/8 =
0.2500 vs the 0.20 ceiling: **breached**, by the thinnest possible valve (two one-pointer-op
register accessors, the exact architecture design 0001 §11.3 prescribes) sitting in an
8-function program. Ruling R28: no honest refactor reduces the fraction, so the breach stands
as measured — the purest instance yet of the fraction-vs-density artifact.

## Measurement observation (home-ground programs; now also MMIO)

Observation recorded for the eventual §9 proceeding, per §0.3 (defects are published, not
patched): the valve-line *fraction* is sensitive to total program size. The Candor port
implements the same spec in 178 logical statements against the Rust baseline's ~930 lines; a
denser program with the same absolute quantity of pointer code shows a higher fraction. The
absolute valve content of the two allocators is comparable (Candor 112 valve lines; Rust ~150);
the fraction differs mostly because Candor needed far less safe scaffolding. Whether that means
"the valve is the program" (the criterion's intent for the 0.40 ceiling) or "the denominator
shrank" is exactly the kind of question §0.3 reserves for the public amendment, not for silent
reinterpretation. The number is recorded as the frozen rules compute it.

## Parser — recorded 2026-07-07

Completed (M5 pass, P1-P32, ~0.03s). Annotation 0.1768 vs Rust 0.4150 (−57%). Valve confined to
the bump-allocator infrastructure exactly as §11.4 predicts: 0.0408 line / 0.1000 function —
clean pass. Rulings R29-R31.

---

# FINAL VERDICT under the frozen decision rule (§5)

All five programs attempted and completed — **M5: pass.**

**M1 (annotation load):** Candor aggregate 0.1624 weighted / 0.1502 mean vs Rust 0.3427 / 0.3189.
Worse ratio **0.474** against a KILL line of 0.90 and WARN of 0.75 — a **53% aggregate reduction.
Decisive pass.** The bet's core cognitive-load claim is confirmed on this basket.

**M1b (combined load):** ratio 0.472 vs KILL 1.0 / WARN 0.85 — **pass** (Candor used zero value
copies; the feared annotation-for-copies trade never materialized). **M4:** no WARN (0 vs 1 copies).

**M2 (valve ambiency):** allocator **0.6292 > 0.40 — KILL**; scheduler **0.4120 > 0.40 — KILL**;
MMIO valve-function **0.2500 > 0.20 — KILL** (valve-line 0.0469 passes); parser pass; arena pass.
No cell-substitutable relief applies anywhere (rulings R15, R19, R28: every valve is genuine
pointer/MMIO work with no checked-runtime substitute).

**M3:** WARN on both home-ground programs (Candor more valve-dependent than the Rust baselines,
whose unsafe partly hides in prior scaffolding density).

**Verdict: §5.1(b) fires three times. Bet 5 is KILLED as pre-registered.** Per §7.2 the verdict
is enacted as a §9 amendment naming Bet 5 and P12, with this record as evidence. Alongside it,
per §0.3, the criterion-defect observations are published, not retrofitted: every M2 breach is
partly or wholly a *fraction-vs-density* artifact — Candor implements the same specs in 2-5x
fewer statements, so identical absolute valve content yields a 2-5x higher fraction, and MMIO's
breach consists of the design's own prescribed ideal architecture (a two-function one-op valve
seam) inside an eight-function program. The allocator's 63% valve-line fraction is the one
breach that likely survives any denominator correction. What a future re-registration should
measure instead (absolute valve content against spec-mandated pointer work, or fractions
normalized to the baseline's density) belongs to the amendment proceeding, not to this record.
