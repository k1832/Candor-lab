# Finding: "no runtime drop flags" refuted for conditional initialization

**Date:** 2026-07-07
**Source:** surfaced by the author of design 0003 (checker-soundness argument) while attempting to
discharge design 0001 §1.5/§10.6's claim that drop scheduling is fully static. Recorded in 0003 §0.

## The defect

The checker accepts a local that is initialized on one path and not the other and never read:

```
let x: R;            // R has a drop hook
if c { x = mk(7); }  // accepted, zero diagnostics
                     // scope exit: dropped iff c — decided by a runtime mask
```

`join_st` yields `MaybeInit` with no error, and the interpreter decides the scope-exit drop by
consulting the runtime `MoveMask` — a drop flag in exactly the sense §10.6 claims to avoid.
Verified end-to-end (trace `[7]` on the true path, nothing on the false path). Memory safety is
not violated — the interpreter executes correctly — but the §1.6 join-agreement rule discharges
the static-drop claim only for the *move* dimension, not the *initialization* dimension.

## Disposition (deciding authority)

**Accepted; resolved in the strict direction.** The checker gains the dual of §1.6's move-join
rule: at a place's drop point (scope exit), its initialization state must be **path-independent**
for any type whose drop is observable (has a drop hook, or transitively contains one or a `Box`).
Path-dependent initialization of such a place at scope exit is a new checker error; the author's
fix is to initialize on all paths, consume on all paths, or narrow the scope. Types with no drop
work (scalars, copy aggregates, rawptr) are exempt — their "drop" is a no-op and no runtime
decision exists.

Consequences:
- Design 0001 §1.6/§7.4 are clarified to state the rule (the interpreter's mask consultation
  becomes an internal assertion, not a semantic decision point).
- Design 0003 §0 is updated from "hole, resolution pending" to "resolved" once the checker rule
  lands, with the rule cited.
- Rejected alternative, recorded: amending 0001 to *permit* runtime-flagged drops for
  conditionally-initialized locals. Rejected because §10.6's refusal of drop flags was argued
  from P4/P9 (no hidden control flow or bookkeeping in drop), and the rejected pattern —
  maybe-initialized, never consumed, live to scope exit — is marginal in real code; paying a
  semantic hole to keep it is a bad trade. If basket porting shows the pattern matters, this
  returns as a recorded design change, not a silent one.
