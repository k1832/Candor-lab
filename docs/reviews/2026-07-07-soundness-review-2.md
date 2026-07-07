# Independent re-review (#2) of design 0003 — retest FAILED

**Date:** 2026-07-07
**Reviewer:** fresh independent LLM session (heavy reasoning tier); verified all five review-#1
fixes hold (repros reject, controls preserved, 188/0 tests), then found a NEW accept-invalid
adjacent to finding 3. Verdict: NOT acceptable for freeze step (i). The philosophy's header
warning performed as designed: the repair failed its retest.

Dispositions by the deciding authority recorded inline. All three findings **accepted**.

## 1. SOUNDNESS (decisive) — moves through `deref`/index are untracked; double-drop demonstrated

`init.rs::apply` treats a move of an *opaque* place (containing `Proj::Deref` or `Proj::Index`)
as a plain read of the root: never sets `Moved`, never runs the drop-hooked-partial check
(`is_drop_hooked_partial` bails on opaque places). The interpreter, divergently, marks the whole
root moved. Consequences proven against the binary: `let taken = (deref bx).inner; let w =
unbox(bx);` checks clean and a §1.5 run-exactly-once drop hook runs **twice** (double-free for
resource-owning types, NN#1); same through a `write` borrow with no Box involved; same for
drop-hooked array elements by constant index (bypassing §1.6). Falsifies 0003 §2.1, §2.4, and
§3 conservatism #5 (which described `init.rs` behavior that only exists in the interpreter).

**[Disposition: accepted — and the root cause is a design gap, resolved by ruling.** Design 0001
never defines a move of a non-copy value out through a `deref`. The ruling, strict direction:
**moving a non-copy value out of any place containing a `deref` or index projection is rejected**
(new E03xx code). Rationale: through a borrow it would hollow out the lender's value (the §2.1
deref description — "reads, copying if copy" — never granted moves); through a Box, `unbox` is
the defined extraction; for arrays, index-granular move tracking is beyond the prototype's place
model (index covers the whole array), so non-copy element extraction is a recorded prototype
conservatism — 0001 §1.6's element-move allowance is narrowed to copy element types in the
prototype, noted in the doc. The basket needs none of the rejected shapes (arena nodes are copy;
the parser moves nothing out of arrays). Design 0001 §1.6/§2.1 gain the clarification; 0003
§2.1/§2.4/§3 corrected; the interpreter's whole-root-move path for opaque moves becomes
unreachable for accepted programs (assert it).]

## 2. Measurement — false-positive E0401 from the same divergence

A non-alloc function partially moving through a Box deref was rejected as "frees" though the
runtime leaks instead — reject-valid, but a count-biasing symptom of finding 1.

**[Disposition: accepted; resolved by finding 1's fix** (the program is now rejected as an
illegal opaque move, and the checker/interpreter divergence disappears).]

## 3. TEXT — §2.6's "closed" fault enumeration omits `BadPointer`

`FaultKind` has 10 variants; §2.6 names 8, omitting `BadPointer` (raw-ptr OOB + init-byte guard),
which is legitimately outside claim (e) but must be named and placed for the enumeration to be
shown closed rather than asserted.

**[Disposition: accepted.** §2.6 enumerates all `FaultKind` variants and places `BadPointer`
(unsafe-only sites + diagnostic guard) explicitly outside claim (e)'s scope.]

**Process note:** after these fixes land, design 0003 goes to retest #3 in a fresh session.
Counts remain inadmissible until a retest passes.
