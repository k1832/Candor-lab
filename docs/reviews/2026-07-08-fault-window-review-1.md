# Adversarial review #1 of the fault-window formalization — UNSOUND AS DRAFTED, reparable

**Date:** 2026-07-08. Hostile-semanticist review; verified the degenerate case against the
interpreter. **Verdict: (c) → (b) after amendments.** Three theorem-breaks share one root: R1
licenses reordering as if observables and fault-capable ops were internal τ-steps. The honest
parts held: the collapse proof, the Option-A analysis, the e⁺ non-retirement reading, NN#5
preservation, and the scope-honesty ledger.

Dispositions (deciding authority's session) — all nine accepted:
1. R1 gains the window constraint: no fault-capable op reorders before its e⁻ (fixes the
   suppressed-observable break; resolves the R1/R3 contradiction).
2+4+5. Observables become effect-order-total: R1 restricted to τ-steps only; observable events
   keep full program order, never coalesced or eliminated (kills the MMIO reorder/DCE break AND
   the per-address aliasing unsoundness; Theorem 2 then holds trivially — the honest proof).
3. Control-dependence defined and folded into → and R1's side condition; speculation of
   observables across control edges forbidden (closes the laundering counterexample).
6. Dependence stated as static def-use on f's output place; truncation is the dynamic fact.
7. O2 rescoped: containment over happens-before AND relaxed visibility, with racy/unsafe
   channels explicitly assigned to the unsafe author's responsibility per §13.4.
8. Store-granularity observability stated; torn multi-store device transactions named as
   P5-legal and flagged for the eventual volatile-access design.
9. The f★-recovery obligation stated as load-bearing, with the replay origin defined (the last
   retired observable).
