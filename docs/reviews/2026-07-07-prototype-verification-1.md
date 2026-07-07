# Fresh-context verification #1 of the Bet 5 prototype

**Date:** 2026-07-07
**Verifier:** independent LLM session (heavy reasoning tier), fresh context, driving the real
`candor-proto` binary with adversarial programs across every section of design 0001 — the
verification design 0001 mandates before the criterion's freeze step (i).
**Verdict:** not yet fit to freeze. One soundness hole (the Stage 3 temporary-loan simplification,
targeted deliberately after Stage 3 flagged it); one minor conformance gap; two doc defects.
Everything else — value model, borrows, signatures, valves, slices, Alloc/Box, faults/contracts,
conv/regimes, patterns, grammar/NN#13 — verified conformant at both the accept/reject boundary and
runtime. Full coverage statement in the session record.

Dispositions by the deciding authority's session recorded inline.

## S1. SOUNDNESS (blocker) — return-extended loan on an inline call scrutinee dies at the call

A compact-default function returning a borrow into its argument, called inline as a match
scrutinee, with a **non-copy** payload bound as a borrow binding: the argument's loan expires at
the call, so the arm can reassign/write/move the argument while the binding is live. Demonstrated
end-to-end: `run` returns the overwritten value (999) from storage aliased by a "live" borrow.
Four shapes confirmed (reassign; write-mode call; the §11.5 arena shape with a non-copy payload;
reborrow-of-reborrow provenance). The named-binding equivalent correctly rejects (E0803), and
copy payloads are safe (read out at the match head), which is why the §11 fixtures never tripped it.

**[Disposition: accepted — fix before anything else.** The returned borrow of an inline call
scrutinee must carry its argument loans for the live range of every binding derived from it, the
same treatment the named-local path already gets. The verifier's four repro shapes become negative
tests; the copy-payload control (returns 111) becomes a positive test.]

## C1. CONFORMANCE-MINOR — `unsafe ""` accepted

§4.1 requires a non-empty justification; the checker enforces presence only.

**[Disposition: accepted.** Empty justification is a checker error; whitespace-only remains legal
per the doc's letter — the checker enforces presence and non-emptiness, not truth or quality.]

## D-A. DOC-DEFECT — `out` call-site spelling

Design 0001 §3.1 writes `f(out x)`, which does not parse; the implementation passes out-arguments
bare with correct semantics, unrecorded in grammar 0002's divergence ledger. Bare spelling makes
the out-mutation invisible at the call site, against §3.1's intent and P13.

**[Disposition: accepted, resolved in the doc's direction.** `out x` becomes mandatory call-site
syntax for out-mode arguments (grammar production added to 0002, divergence ledger updated;
parser + checker enforce: `out` marker required for out-params, rejected for non-out params).
P13/P2 rule the choice: a caller-visible mutation of a caller-owned slot must be visible at the
call site. Design 0001 §3.1's examples become correct as written.]

## D-B. DOC-DEFECT — §8.2.1 references drop hooks on enums, which the grammar cannot express

The rejection path for move-bindings out of a drop-hooked *enum* is vacuous: 0002 attaches drop
hooks to structs only.

**[Disposition: accepted, resolved by striking.** The prototype does not need enum drop hooks;
design 0001 §8.2.1's enum-drop-hook language is corrected to structs (edit recorded in the doc);
the guarantee that a move-binding never fires on a drop-hooked scrutinee stands, now non-vacuously
scoped to where drop hooks exist.]
