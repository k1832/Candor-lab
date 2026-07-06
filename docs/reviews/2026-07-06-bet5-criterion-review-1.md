# Adversarial review #1 of BET5_CRITERION.md (v1 draft)

**Date:** 2026-07-06
**Reviewer:** independent LLM session (heavy reasoning tier), hostile-methodologist brief; saw the philosophy and the criterion draft, no drafting history.
**Verdict:** not fit to freeze as written; targeted defects, not a rotten foundation. Findings 1–8 must be fixed before the freeze instant.

Disposition of each finding is recorded inline as **[Disposition: …]** by the deciding authority's session. The revision landing after this review implements the accepted fixes; the criterion's own §0.4 ledger records the resulting threshold changes.

---

## 1. BLOCKER — The frozen classification table does not exist for Candor and is co-designed with the thing it measures (§3.1, §3.2, §8.4)

§3.2 defines Candor annotation tokens only by a counterfactual ("absent from a hypothetical pure-value version"), which is a modeling judgment, not a mechanical rule; the Candor side of the table cannot exist until the throwaway syntax exists, which happens during prototype construction by the same team, before the freeze instant. The syntax, the classifier, and the ports are co-designed by one interested team before the lock — building the ruler after seeing the object.

**Fix proposed:** author and hash the Candor classification table as a concrete positive enumeration of token classes at freeze step (i), by someone other than the port authors; forbid prototype syntax changes after the hash; drop the counterfactual definition.
**[Disposition: accepted.** Independence is approximated honestly for a one-person project: the table is authored in a session blind to any port code, before any port exists, and is published with the hash; see finding 2's disposition for the general independence posture.]

## 2. BLOCKER — Every load-bearing subjective judgment routes to a single interested adjudicator (§2.5, §6.5)

Idiomaticity of the Rust baseline, annotation-classification ambiguity, and coalescing-equivalence all route to the deciding authority, whose bet is on trial. §9's open-comment protection does not apply to §6.5 rulings.

**Fix proposed:** blind classification before the which-side-benefits mapping is computed; independent confirmation of Rust idiomaticity; publish disagreements; subject §6.5 rulings to open-comment discipline.
**[Disposition: accepted, with an honesty clause.** Full third-party independence is not available to a solo project; the criterion now states this limit plainly instead of claiming an independence it cannot deliver, and adopts the available mitigations: blind classification order, §6.5 rulings published for open comment with a minimum comment period before they take effect, independently sourced (not commissioned) Rust baselines preferred, and all rulings on the public record.]

## 3. BLOCKER — The M2 home-ground KILL line is a false-KILL trap (§2.4a, §4.2, §4.3, §5.1b)

§2.4a requires raw-pointer free-list manipulation, and real idiomatic Rust allocators matching that spec are plausibly above the 0.40 valve-line / 0.50 valve-function ceilings — so a Candor port at parity with idiomatic Rust would be auto-KILLED while M3 treats being worse than Rust as a mere WARN. Internally incoherent, and the numbers were asserted without calibration.

**Fix proposed:** measure the frozen Rust baselines' own valve fractions before the freeze and set the home-ground KILL relative to them, e.g. `valve_candor > max(0.40, 1.25 × valve_rust)`.
**[Disposition: accepted as proposed** (variant (a)); Rust baseline valve fractions are measured and recorded in §6.6 before the freeze instant.]

## 4. MAJOR — Annotation fractions compared across two lexers with different tokenization granularity (§3.1, §3.2, §3.5)

A ratio of ratios over non-commensurable token streams; `&mut` is two tokens, a Candor mode keyword is one.

**Fix proposed:** count at a shared normalized unit (each borrow-mode declaration = 1 on both sides); normalize the denominator to logical statements from the AST, not raw lexer tokens.
**[Disposition: accepted.** The primary metric becomes annotation *units* per logical statement, with the unit table part of the frozen classification table; raw token fractions are demoted to reported-not-gating.]

## 5. MAJOR — Ambiguous whether borrow markers count at signatures only or also at call sites (§3.2a)

Call-site `&` counting would artificially flatter Candor; P12's delta claim is signature-scoped.

**Fix proposed:** count annotation at signature/declaration sites only, symmetrically.
**[Disposition: accepted,** with one amendment: use-site *valve* tokens (raw-pointer ops, unsafe blocks) still count toward M2's valve regions — finding applies to M1's annotation classes only.]

## 6. MAJOR — Prototype checker soundness never stated as a precondition of measurement validity (§2.1, §2.3, §3.6)

A permissive checker mechanically lowers annotation and valve counts while the code is not actually memory-safe; "lower load" would really mean "checker didn't demand it."

**Fix proposed:** checker soundness (with a written, independently reviewed soundness argument) as a freeze-step precondition; counts admissible only from a sound checker.
**[Disposition: accepted.]**

## 7. MAJOR — No unified load metric; fraction denominator plus P13 create perverse incentives (§3.5, §4.1, §4.4)

Annotation down + copies way up can pass both M1 and M4; token-padding dilutes fractions; minimizing annotation can reward withholding aliasing information the verifier needs.

**Fix proposed:** combined annotation+copy load metric with its own thresholds; statement-normalized denominator; explicit P13 reconciliation (the claim under test is fewer aliasing relationships to declare, not fewer tokens per se, and omissions must be checker-licensed).
**[Disposition: accepted.** Combined metric M1b added: KILL if combined Candor load > combined Rust load; WARN above 0.85×.]

## 8. MAJOR — Freeze order lets the team tune specs/tests to advantage; independence imposed only on baselines (§2.5, §6.2, §6.3)

Specs written after baselines, by the port authors, with the Candor design in hand; also an ordering contradiction (§2.5 says baselines implement a spec that §6.2 freezes later).

**Fix proposed:** freeze order becomes specs/tests → baselines → ports; spec authors may not be Candor port authors; baselines confirmed against the already-frozen spec.
**[Disposition: accepted,** with the same solo-project honesty clause as finding 2: spec-authoring sessions are blind to the Candor design docs, and specs are published before baselines are chosen.]

## 9. MAJOR — M1 KILL too lenient ("lower" bet survives at parity); polish asymmetry between first-pass Candor and idiomatic Rust (§4.1, §3.6, §6.4)

**Fix proposed:** make meaningful reduction the KILL line or justify parity survival; hold both sides to the same polish standard; define "attempt"; external witness for no-re-rolling.
**[Disposition: accepted with modification.** M1 KILL moves from >1.0× to >0.90× Rust (the bet must show at least a 10% reduction; parity no longer passes), WARN stays at >0.75×. Polish symmetry: the scored artifact on *both* sides is the adjudicator-confirmed idiomatic port, and the full development history of the Candor ports is pushed to the public repository as it happens — the public timestamped record replaces the unenforceable "first passing version" rule.]

## 10. MINOR — §5.3 counts WARNs from M7, which defines no WARN condition (§4.7, §5.3)

**[Disposition: accepted.** M7 removed from the WARN tally; supplementary evidence only.]

## 11. MINOR — M6 depends on a spec pack that may not exist at Bet 5 time; near-certain to fire; N/model set deferred (§4.6)

**[Disposition: accepted.** M6 demoted to supplementary evidence alongside M7 — reported, admissible in review, feeding no counts; "in context" defined as the frozen grammar + memory-model design doc + basket specs.]
