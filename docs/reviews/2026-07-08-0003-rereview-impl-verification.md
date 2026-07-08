# 0003 re-review + implementation verification (designs 0004/0005) — FAILED

**Date:** 2026-07-08
**Reviewer:** fresh session, discharging the 0003 hash-tripwire obligation against the real
binary. **Verdict: (c) FAILS — accept-invalid found**, with the crucial attribution stated
fairly: the root hole is **pre-existing** (confirmed by rebuilding commit b838b62: same repro,
zero new-feature syntax), latent through all four prior soundness reviews. The 0004/0005 feature
code is otherwise verified sound on every attacked seam (E0510 boundary, nested projection,
effect/drop neutrality, interpreter arithmetic incl. nonzero offsets and null non-propagation,
counter region rule, out-mode seam, S1 indistinguishability, loop reborrows).

## The hole: write through a shared borrow is accepted (XOR / claim (b) violated)

```
let b1 = read x; let b2 = read x;
(deref b1).n = 99;            // ACCEPTED; runs; (deref b2).n reads 99
```

`check_place`'s deref arm peels `Borrow` and `BorrowMut` identically with no mutability check;
`canonical()` collapses the deref to the borrow binding, so the write never conflicts with the
shared loan on `x`. Design 0001 §2.1 always said deref-write requires an exclusive borrow — the
checker never enforced it. Consequences: exclusive reborrow from a shared borrow also accepted;
0005's desugaring has no shareability gate, so bare `f(b)` with a shared `b` to a write-mode
parameter slips through keywordless; 0003 §2.2 and 0005 rule 1 falsely claim the gate exists.

**How four reviews missed it:** every prior attack targeted interaction seams (the recorded
pattern) and conflict-pair shapes (two exclusives, shared+exclusive via separate borrow
expressions). Nobody wrote the two-line direct mutation through a shared borrow. Recorded as a
lesson: seam-hunting is not a substitute for enumerating each analysis's own obligations against
the model doc line by line.

## Dispositions (deciding authority's session) — all accepted

1. **Fix the checker, not the text:** (i) assignment/write access through a place whose
projection path derefs a SHARED borrow is rejected (new E08xx); (ii) `write (deref b)` with `b`
shared is rejected (exclusive reborrow from shared); (iii) `reborrow_desugar` gains the
shareability gate 0005 specified (shared borrow to write-mode param: error, not desugar).
Reviewer's repros become tests; legal shapes (write through exclusive; shared reads) covered as
positives.
2. **0003:** §0 gains this failure's history entry; §2.2's claim becomes true by the fix; the
"how it was missed" lesson recorded in §4.
3. **Measurement impact check (mandatory):** after the fix, all five checkable + runnable basket
fixtures and both scored port artifacts are re-run under the fixed checker. If all pass, the
frozen measurements stand (the artifacts never exercised the hole) — recorded in RESULTS.md as a
soundness-precondition confirmation. If any fails, that is a §3.7 admissibility event for the
authority, stated plainly.
4. **Retest:** a fresh session re-verifies the fix before the scheduler re-port proceeds (the
philosophy's failed-retest lesson, fifth iteration).
