# Independent retest #3 of design 0003 — FAILED

**Date:** 2026-07-07
**Reviewer:** third fresh independent session. All prior fixes hold (full repro matrix re-run,
199/0 tests); the E0310 boundary survived every attack (double indirection, slice-element moves,
copy-aggregate-of-rawptr, swap shapes, opaque-scrutinee matches). The new hole is at the one seam
no prior review covered: **contracts × dataflow**.

Dispositions by the deciding authority inline. All four findings **accepted**.

## 1. SOUNDNESS (decisive) — `ensures` clauses bypass init/move/loan analysis; use-after-free demonstrated

`check_fn` type-checks `ensures` with no CFG block, so its reads/moves/borrows reach no analysis;
the interpreter evaluates the clause after the body, before drops. Repro: a fn that `unbox`es its
`Box` param and whose `ensures` dereferences that param — checks clean, reads freed heap at
runtime. Same for reads of body-moved params (incl. drop-hooked). `requires` is unaffected
(checked in the entry block). Falsifies claims (a)/(c).

**[Disposition: accepted; ruling — contract clauses are READ-ONLY.** No moves, no exclusive
borrows, no out-arguments inside `requires`/`ensures`/`assert` conditions (new checker error):
under P8 contracts are checks and oracles, and a check that consumes or mutates is incoherent.
Additionally, `ensures` accesses are analyzed against the post-body state at every normal-return
point — a read of a moved/consumed place is the ordinary E0301, exactly as if written in the body
at the return. Design 0001 §7.3 gains both rules; 0003 §2 gains the contract-boundary sub-argument
(finding 2's demanded disclosure becomes a discharge).]

## 2. TEXT — the argument never disclosed the contract/dataflow gap

**[Disposition: accepted; discharged by finding 1's fix and the new 0003 sub-argument.]**

## 3. OBSERVATION — assignment to a static is accepted and silently mutates it

Not a breach of claims (a)-(e) in the single-threaded model, but an unadjudicated spec silence.

**[Disposition: accepted; ruling — statics are immutable.** Assignment to (or exclusive borrow /
out-passing of) a static is a checker error. Statics exist for vtables and constants (0002); a
mutable global is a real design question (concurrency, P9) the prototype must not answer by
accident. If a basket port needs one, that returns as a recorded design change. Design 0001 §8.2
(or where statics are described) gains the sentence.]

## 4. TEXT — stale empirical test tally in 0003 §5

**[Disposition: accepted;** the tally is corrected at fix time and rephrased so the hashed
document states the suite-green invariant and points to CI/the repo for the count, rather than
hardcoding a number that rots.]

**Process note:** retest #4 follows in a fresh session. Counts remain inadmissible.
