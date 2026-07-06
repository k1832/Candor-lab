# Blind adversarial review #1 of the basket specs

**Date:** 2026-07-06
**Reviewer:** independent LLM session (heavy reasoning tier), under the same blindness constraint
as the spec author (read only the philosophy, the criterion, and docs/basket/ — no design docs,
no prototype). Briefed to re-derive vector arithmetic by hand and to attack each suite with
"cheaper implementation that still passes" strategies (criterion §2.3 detectability).
**Verdict:** arena spec clean; parser needs one number; mmio needs rework of its algorithm text
and one scenario; allocator and scheduler each need one substantive fix. 3 BLOCKERs, 3 MAJORs,
6 MINORs. All fixes are local edits or single added vectors; no basket expansion needed.

**Dispositions (deciding authority):** all 12 findings **accepted as proposed**. The fix pass is
executed by a session under the same blindness constraint, reading only the permitted files plus
this review record (itself produced blind, so it carries no design-doc taint).

Summary of findings and the accepted fixes:

| # | Sev | Spec | Finding | Accepted fix |
|---|-----|------|---------|--------------|
| 1 | BLOCKER | mmio §3.4 | Write loop never increments `transferred`; binding algorithm cannot emit its own binding traces (M1/M8) | WRITE branch: after `write DATA=buf[transferred]`, `transferred += 1; stall := 0`; write only while `transferred < n` |
| 2 | BLOCKER | mmio §2.4/§2.5, M10 | Recover-RESET always clears `stuck`; M10's recover-timeout→FATAL is unrealizable | Add `stuck_persists: bool` scenario flag; RESET rule keeps `stuck` when set; M10 references it |
| 3 | BLOCKER | parser P18 | `Int 2` span stated `[5,6)`; correct is `[6,7)` (byte 5 is the tab) | Change to `[6,7)` |
| 4 | MAJOR | allocator §1.5/§3.6 | Bump-and-reset-when-empty cheat passes all coalescing vectors | Add partial-coalescing vector: 10 contiguous 1 KiB blocks, free the adjacent middle three (others live), then an allocation of three-blocks-worth (net of the implementation's per-block overhead) MUST succeed |
| 5 | MAJOR | scheduler §3.5 | O(1) middle-removal / intrusive-doubly-linked not suite-observable; §3.5 claims otherwise | Relabel: suite verifies removal correctness; O(1)/intrusiveness confirmed by adjudicator code inspection under criterion §6.5 |
| 6 | MAJOR | allocator §3.8/§4.5 | Cross-implementation OOM-set identity contradicts §5.2 strategy freedom | Rescope reproducibility to same-implementation; draw sequence (not OOM set) is identical across implementations |
| 7 | MINOR | mmio §2.4 | IRQ_ACK→READY transition ambiguous; M6 depends on it | State: CTRL write with IRQ_ACK while COMPLETE sets DS := READY |
| 8 | MINOR | mmio M9 | Not a fully frozen trace (cmd/n unfixed, ellipses) | Pin cmd/n/buf and write the full expected trace |
| 9 | MINOR | parser §1.4/§3.5 | Zero-copy claimed observable; it is not | Drop the claim; adjudicator-confirmed property, not suite-enforced |
| 10 | MINOR | allocator A6 | Realloc alignment-preservation tested vacuously (align=1) | Allocate the A6 block with align=64 |
| 11 | MINOR | mmio §3.4 | Post-fix write loop would false-timeout for n ≥ 8 (latent; suite max n=3) | Reset `stall := 0` on every written word (progress) |
| 12 | MINOR | scheduler §2.7 | set_priority-on-BLOCKED has no nominal vector | Add: admit@p2, block, set_priority→p0, wake, assert tail of p0 |

Arithmetic verified correct by the reviewer (fix pass must not touch): arena AR4, AR7, AR10,
AR11, AR14, AR16, AR17, AR19–AR23, AR25, AR28; both xorshift64 stress derivations; parser
P1–P17, P19–P30 (only P18 wrong); MMIO trace values M1–M8 (once §3.4 is fixed; M2 READ path
correct as written); scheduler T1–T12 and the shadow-model oracle.
