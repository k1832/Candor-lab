# Adversarial review — 0015 Borrowed-element iteration (RefIndexed)

**Design:** [docs/design/0015-borrowed-iteration.md](../design/0015-borrowed-iteration.md)
**Date:** 2026-07-13
**Reviewer:** adversarial pass (fresh context), mandate: break the soundness argument
with a use-after-free or mutation-during-shared-borrow in *safe* Candor.
**Verdict:** SURVIVES WITH REPAIRS — the `RefIndexed` shape is right, but §5's
soundness argument is **unsound as written**, and the review surfaced a **pre-existing
SINK-grade safe-code use-after-free in the current checker** (independent of 0015).

Dispositions are the deciding authority's; recorded inline per finding once made.

---

## F1 — Escape via copy-to-outer-local is a safe-code use-after-free. Severity: REPAIR (0015) / **SINK (pre-existing checker bug)**

§5 case (2) claims an escaping `x` "keeps the loop loan `L` live past the loop." The
implemented loan model does the **opposite**: a borrow copied into a binding **sheds
its loan** — `carries_borrow` returns `false` for a bare identifier
(`prototype/src/check/stmt.rs:171-194`), so `let c = b` anchors nothing
(`stmt.rs:63-67, 93-102`); and the loan `x` does carry is rooted at `__c`, not `coll`
(deref collapses to the root, `dataflow.rs:69-72`; distinct roots don't overlap,
`dataflow.rs:91-93`), so it could never freeze `coll` regardless.

**Verified repro (checks clean, exit 0; faults at runtime reading freed heap):**
```
fn use_it(b: read S) -> i64 { return (deref b).x; }
fn escape2(a: read Alloc) alloc -> i64 {
    match box(read a, S { x: 7 }) {
        case BoxResult::oom => return -1,
        case BoxResult::boxed(bx) => {
            let b = read (deref bx);
            let c = b;                       // copy: loan on bx is SHED here
            let owned: S = unbox(bx);        // frees the box while c still points in
            return use_it(read (deref c));   // use-after-free — accepted by the checker
        }
    }
}
```
`candor check` → exit 0 (no diagnostic); `candor run` → panics reading a freed slot.
This is a pre-existing loan-model defect, but 0015's `for read` is the **first
iteration surface to hand out escapable `read Item` loop locals** (`Indexed` yields
owned copies), turning a latent hole into an easily-reached one.

**Suggested direction:** (1) [higher priority, cross-cutting] repair loan propagation
so a copied/aliased borrow keeps its source loan live (`0001 §2.3` / `loans.rs`),
**or** (2) 0015's own open-Q1 fallback: constrain the `for read` variable `x` to be
non-escaping (a new, narrow rule confined to the desugar).

**Disposition:** _(pending deciding authority)_ — **checker fix landed 2026-07-13** (implements suggested direction (1), the cross-cutting loan-propagation repair; ledger `LOAN-COPY-UAF`). A `read`/`write` borrow copied into a binding now propagates its source loan(s) to the new binding's live range (`propagate_place_loans` + the extended `carries_borrow` gate, `src/check/{mod,expr,stmt}.rs`), so the verified repro is rejected E0802 and the copy-then-{move,write,return} family is caught (8 regression tests in `tests/loans.rs`), full suite + selfhost loans oracle green, no false positives. The UAF is closed independent of 0015; whether 0015 additionally adopts the (2) non-escape constraint on `x`, and the acceptance of 0015 itself, remain the deciding authority's call. Residual: the analogous `slice`-copy shed (a `copy` slice aliasing its array) is a separate pre-existing sub-class, not closed here — see ledger.

## F2 — Mutation-during-borrow, in-loop: CLOSED

The loop loan `L0` (`read coll` anchored to `__c`, live across the loop because `__c`
feeds every `get_ref`) freezes `coll` to shared for the whole body. Verified: holding
`read coll` across `coll = …` is rejected **E0803** (`loans.rs` conflict scan); direct
`write`/move/exclusive-borrow of `coll` in the body is caught. The doc's in-loop
mutation-safety claim holds.

## F3 — Loop-carried loan gap: CLOSED (but not by the doc's stated mechanism)

Protection comes from `L0` spanning the loop (backward liveness of `__c`), **not** from
"each `get_ref` return reborrow carrying `L`" — that per-iteration loan is rooted at
`__c` and dies each turn. `L0` does not collapse; `count` is called once; the loop
never calls `get_ref` out of range. The doc's §5 should be corrected to attribute the
guarantee to `L0`'s span, not the per-iteration reborrow.

## F4 — Native growable collections: CLOSED for safe code, CAVEAT

Grow needs `write self`; `L0` forbids any exclusive borrow of `coll` in-loop, so
`push`/grow is rejected exactly like F2. Residual: an impl that grows through an
interior `rawptr` under `read self` would dangle — but that is an unsafe-valve author
bug (the model has no interior mutability, `0001 §4.3`), not a safe-code hole. Should
be stated as a scope caveat on `RefIndexed` impl authors.

## F5 — Provenance mis-attribution from `i: usize`: CLOSED

`region_source_indices` (`expr.rs:2513-2534`) filters to borrow params only; `i: usize`
(take-mode) doesn't count, so the compact default fires on the sole borrow-in
`read self`. The `usize` does not perturb the tie-to-self. Return route separately
confirmed closed: returning a copied borrow is rejected **E0806** (`expr.rs:792`). Of
§5's three escape routes, fields (`§3.4`) and return (`§3.3`) hold; only local-escape
(F1) is open.

---

## Overall

Option A / `RefIndexed` is the **right shape** — no borrow fields, compact-default
provenance, in-loop XOR protection, and attacks F2–F5 are genuinely closed; no surveyed
alternative does better on the target capability (C is sound but can't back `for`; D/E
collapse to A or to refused machinery). But **do not accept 0015 on its current
soundness text**: F1 shows the "escaped borrow keeps `L` live" step is false against the
implemented loan model, and a real safe-code UAF exists on that path.

**Required before acceptance (recommended):**
1. **Treat F1's underlying loan-copy defect as a standalone SINK-grade memory-safety
   bug** and fix it (or explicitly scope it), independent of 0015 — it is reachable in
   the shipping compiler today.
2. Revise 0015 §5 to (a) attribute in-loop safety to `L0`'s span (per F3), and (b)
   either depend on the F1 fix, or add the non-escape constraint on `x` to the desugar
   spec as the self-contained guarantee.
