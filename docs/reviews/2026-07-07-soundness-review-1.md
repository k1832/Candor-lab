# Independent review #1 of design 0003 (checker-soundness argument)

**Date:** 2026-07-07
**Reviewer:** independent LLM session (heavy reasoning tier); did not author the argument or the
checker. Attacked both the argument text and the running binary; repros under
`prototype/target/scratch-review/`.
**Verdict:** NOT acceptable for freeze step (i) as written. Two decisive breaches — one
wrongly-accepted memory-unsafe program, and the §0 RESOLVED claim shown false — plus three
further findings. The core loan machinery, move-join rule, unsafe boundary, and structural
walks survived attack; the defects cluster at interaction seams (drop-point coverage,
partial-move nesting, provenance through control flow, the free side of the alloc effect).

Dispositions by the deciding authority recorded inline. All five findings **accepted**.

## 1. SOUNDNESS — borrow of a local returned through `match` escapes E0806 (dangling borrow)

`borrow_provenance` returns None for `Match`/`Block` shapes and `check_return_provenance` bails
instead of rejecting; a function returning `match c { ... => read x, ... }` where `x` is a local
is accepted and `run` reads the torn-down stack slot. The direct-return control is correctly
rejected. §4 had disclosed the shape as a "completeness caveat" — mis-rated: accept-invalid
memory unsafety is never a caveat.

**[Disposition: accepted.** Provenance becomes total: an unrecognized borrow-producing shape
REJECTS (None ⇒ E0806); provenance recurses into every `match` arm (each must be region-legal)
and block tails. The reviewer's repro and control become tests.]

## 2. ARGUMENT-DEFECT — reassignment is an unguarded drop point; §0's RESOLVED claim is false

E0309 covers scope exits only, but §1.5 defines reassignment as a drop; a needs-drop place
MaybeInit at a whole-binding reassignment is accepted and the interpreter consults the runtime
mask — the exact drop-flag the resolution claimed to have eliminated. Not memory-unsafe; falsifies
the gating claim.

**[Disposition: accepted, strict direction again.** E0309 extends to whole-binding reassignment
of a needs-drop place in MaybeInit state. Design 0001 §1.6 rule 3 lists both drop points.]

## 3. SOUNDNESS — nested partial move out of a drop-hooked struct silently skips its hook

`is_drop_hooked_partial` inspects only the root local's type; moving `outer.a.leaf` where
`typeof(outer.a)` is drop-hooked but `Outer` is not is accepted and `A`'s hook never runs.

**[Disposition: accepted.** The check walks every proper prefix of the projection: if any
intermediate place's type is drop-hooked, the partial move is rejected (E0303).]

## 4. SOUNDNESS (effect partition) — `free` runs from non-alloc functions via unbox / Box drop

`unbox` and Box-drop call `free` through the vtable at runtime, but neither forces the `alloc`
marker, violating §6.3 given §6.1's alloc-typed `free` field.

**[Disposition: accepted, resolved in the effect's direction, not the exemption's.** Freeing is
allocator work — in an interrupt context calling `free` is as forbidden as calling `alloc`, and
§6.1 already types `free` as alloc-marked. `unbox` and any scope that drops a `Box` (or a
Box-bearing type) are alloc-effecting; design 0001 §6.2/§6.3 gain a sentence stating it and
naming the consequence (a function that receives a Box by value and lets it die is alloc-marked —
honest, and mirrors no_std reality). Rejected alternative, recorded: exempting free from the
effect — rejected because it would let "allocation-free" code call into the allocator.]

## 5. ARGUMENT-DEFECT — claim (e) (fault delivery) asserted, never discharged

**[Disposition: accepted.** Design 0003 gains a §2.x discharging (e): enumerate fault sites; show
the checker never suppresses one except via lexically-scoped regime blocks (defined redefinition,
not fault-skipping) and unsafe (excluded); note E0601 exhaustiveness + E0810 all-paths-return
close the fall-off-with-no-value paths.]

**Process note:** after the fixes land, design 0003 is re-reviewed in a fresh independent session
(a repair that fails its retest teaches more than one that passes — philosophy header). Counts
remain inadmissible until that re-review passes.
