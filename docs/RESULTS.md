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

---

# Re-scoring under the ratified successor registration (BET5_CRITERION2.md, frozen 2026-07-08)

Mechanical computation from the frozen v2 measurements (docs/measurements/*/*.v2.json), per the
registration's decision rule. M1/M1b/M4/M5 carry over as already computed (all pass; M1 worse
ratio 0.474, decisive).

| Program | V_candor | V_rust | R_valve | Gate | Outcome |
|---|---|---|---|---|---|
| Allocator | 96 | 86 (17 authored + 69 vendored) | 1.116 | home-ground, carved (P12 concession) | **WARN → mandatory review** |
| Scheduler | 47 | 89 (2 authored + 87 vendored) | 0.528 | home-ground, gates normally | pass under registered rule; **23.5 KILL under authored-only** → **baseline-sensitive → mandatory review** |
| MMIO | 3 | 6 | 0.500 | value-favorable | pass |
| Parser | 9 | 0 → V2 (0.0289) | — | value-favorable floor | pass |
| Arena | 0 | 0 → V2 (0.0000) | — | value-favorable floor | pass |

**Outcome: MANDATORY §9 REVIEW** (no clean confirmation; no KILL). The review must dispose of:
1. The allocator's home-ground WARN on the conceded allocator class (P12 v4.1).
2. The scheduler yardstick question, verbatim from the registration: is "idiomatic Rust's
   inherent valve demand" the self-contained measured artifact (R1: 89 statements) or the
   authored valve figure (2 statements, the crate carrying the rest) — noting symmetrically that
   a mature Candor ecosystem would provide the same machinery to its authors, so the
   authored-only reading cuts both ways across time.
3. The standing evidence: the load claim passed decisively on every metric and every program;
   value-first's cost concentrates precisely and only where P12 v4.1 already concedes it.

## §9 review ruling — 2026-07-08

**Bet 5 is PROVISIONALLY CONFIRMED on this basket** (philosophy amendment v4.2). The registered
self-contained yardstick governs the scheduler (the authored-only 2-vs-47 gap is recorded as an
ecosystem-bootstrap fact, owned by Bets 4/6, not a memory-model defect); the allocator-class
concession stands. Binding commitments: safe rawptr field projection and reborrow ergonomics
enter the next design round, and the scheduler is re-ported and re-measured under the frozen
successor rules before any syntax freeze — returning to review if the numbers worsen.

## Soundness-precondition confirmation — 2026-07-08

A pre-existing checker hole (write through a shared borrow, latent through all four original
soundness reviews) was found by the design-0004/0005 re-review and closed as E0809
(docs/reviews/2026-07-08-0003-rereview-impl-verification.md). Mandatory measurement-impact check:
all five checkable and five runnable frozen fixtures and all five scored port artifacts re-run
under the fixed checker — every one passes with byte-identical checker output. No frozen
measurement exercised the hole; the Bet 5 record stands. No §3.7 admissibility event.

## v4.2 commitment discharged: scheduler re-port re-measurement — 2026-07-08

The scheduler was re-ported under the evolved language (safe field_ptr, implicit call-site
reborrow — designs 0004/0005, adversarially reviewed; the checker meanwhile hardened by twelve
closed holes across eight review rounds, with the frozen record confirmed intact throughout).
The re-port (ports/candor/scheduler-v2/) passes the full frozen suite (T1-T20 incl. the 20k
stress, sentinel 777, ~22s) and measures, under the frozen successor rules and the a-priori
field_ptr ruling:

- **Valve statements 45 vs v1's 47** (R_valve = 45/89 = 0.5056 vs 0.5281) — the a-priori
  prediction (~1 statement, block granularity) slightly beaten at statement granularity, honestly
  reconciled. Annotation 0.2907/stmt (v1: 0.2948). All 41 explicit reborrow sites now the bare
  canonical form.
- **Not worse than the frozen baseline result → the return-to-review clause does not fire.**
  Philosophy v4.2's binding commitments are discharged: both design items shipped and reviewed,
  the re-port re-measured under frozen rules, result equal-or-better. NN#14's conditional gate
  obligations are met; Bet 5's provisional confirmation stands as ruled.
