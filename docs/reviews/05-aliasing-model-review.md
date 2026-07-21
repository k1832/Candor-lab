# Adversarial review — the unsafe-code aliasing model (spec ch05 §6)

**Artifact:** [docs/spec/05-unsafe-and-pointers.md](../spec/05-unsafe-and-pointers.md) §6 (+ Appendix 05-A)
**Date:** 2026-07-21 · **Mandate:** break it — a rule that licenses a miscompile, is
vacuous, contradicts the fault/contract model, or mis-states the implementation.
**Verdict:** **SOUND-WITH-REPAIRS** — safe to commit as NORMATIVE-DRAFT; the one
REPAIR below must land before OBL-ALIAS drops "discharged-pending-review."

## The killer attack — DEFENDED (the spec's strongest point)

The model's novel choice: a rawptr **write** aliasing a live borrow is UB-in-unsafe,
but a rawptr **read** of a live `write` borrow is deliberately *not* UB (narrower
than Rust). The attack: construct a §6.3-licensed optimization (DSE / store
forwarding on a write-borrow path) that a legal rawptr read observes. **It does not
construct:** §6.3.1(a) grants only non-aliasing of *modification* ("modified only
through that borrow"), never observation-uniqueness; DSE needs a *no-reader* fact
§6.3 nowhere grants, so §6.1.1 ("assuming more than §6.3 is a spec violation")
forbids it; forwarding/redundant-load-elim need only modification-non-aliasing,
and a rawptr write that would break them is UB per §6.4.1.

## Findings

- **REPAIR — "observable" is undefined for non-faulting executions.** §6.2.5 /
  §6.3.1(c) anchor ordering on "observable," citing ch06's fault-truncation/window
  clauses — neither defines the term for the common non-faulting run, and ch09 is
  ADOPTED-PENDING. MMIO's own prohibitions are self-contained; "any other
  observable" is not. **Fix:** a first-class "observable effect" definition valid
  for non-faulting runs, or scope §6.3.1(c) to the enumerated observables.
- **NIT — §6.5.3 (future borrow-based `noalias`) is under-stated, not over-promised.**
  It needs no provenance model, but it must say explicitly that the tightening
  withdraws §6.4.1's rawptr-*read* carve-out (LLVM `noalias` forbids foreign reads
  too), not merely "narrows sound programs."
- **Code-hygiene NIT:** `lower.rs:20`'s "rawptr/MMIO not yet MIR-marked" comment is
  stale vs `mir/build.rs` (`mark_last_observable` fires on `ptr_read`/`ptr_write`).

## Spot-checks of Appendix 05-A — all held

Independently verified, not taken on the appendix's word: `llvm.rs` emits **no**
`!tbaa`/`noalias`/`!alias.scope`/`!invariant.load` (flat `inttoptr` arena; MMIO via
`rt_mmio_*`); `lower.rs` uses default `MemFlags` everywhere with observable/fault
accesses as barrier calls; `mir/opt.rs`'s sole pass elides only dead pure
non-fault-capable τ-assigns; `mir/build.rs` marks every `ptr_read`/`ptr_write`
observable (rawptr fully volatile — stronger than §6.2.3 requires). The
implementation sits strictly within the §6.3 upper bound.

## Composition + teeth — hold

§6.3 has teeth (borrow-`noalias`/TBAA emission would be a checkable violation) and
forbids nothing the backends do; §6.1.3 defers fault/consistency ordering to
ch06/ch09 rather than contradicting them; MMIO never-elided/reordered matches the
barrier-call lowering; ledger/front-matter status changes accurate.
