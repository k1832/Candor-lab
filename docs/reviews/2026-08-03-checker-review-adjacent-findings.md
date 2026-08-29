# Adjacent pre-existing bugs from the checker-fix adversarial review (2026-08-03)

Found while attacking the array-repeat/array-literal checker fixes; all five
reproduce identically on the pre-change and post-change binaries, so none are
regressions of that work. Logged here so they survive the review transcript;
each is queued as its own workstream. Severity per the reviewing session.

## P1 — Generic monomorphization renders array type arguments with length 0 (HIGH)

`compiler/src/generics.rs:1441-1444` maps `Type::Array(e, _)` to
`TyKind::Array { size: IntLit 0, .. }` when rendering a type argument.
`fn idf[T: copy](x: T) -> T` called with a `[3]i64` returns **3 on the
tree-walk oracle and 0 on MIR/native (both tiers)** — wrong results in safe
code, engine-divergent. Non-generic array pass-through and generic
copy-struct pass-through are unaffected.

## P2 — Never-initialized needs-drop local runs its hook on garbage (HIGH)

`let mut p: D;` (drop-hooked, never initialized) traces nothing on the
oracle but **`[0]` on MIR and native**: the hook runs over zeroed,
never-constructed storage. Same defect family as the overwrite-drop
dossier's D1 correction (the builder's move mask models moves, not
initialization); any assignment-drop fix must track init state separately
or it will fire hooks on uninitialized memory for the legal
`let mut p: D; p = D{..};` shape.

## P3 — Runtime-valued repeat length diverges (MEDIUM)

`let n: usize = 3; let a = [7i64; n];` checks clean, runs on the oracle,
**faults `Bounds` on MIR/native**. Design 0001 (line 280) requires
compile-time-constant lengths; nothing enforced it. Being fixed in the
checker workstream (reject non-constant repeat sizes), which closes this.

## P4 — Arrays of borrows escape the loan machinery (MEDIUM, soundness)

`let a: [3]borrow i64 = [read x; 3]; x = 9; return deref a[0];` returns
**9** — a write through a live shared borrow, unchecked. The dangling
variant reads a dead frame slot and returns stale data. Reachable through
the array-literal form as well, so this predates E0716 and is not created
by admitting `read`-borrow repeat elements. The loan checker needs to track
borrows stored into array elements (it tracks struct fields; arrays fell
through).

## P5 — Untyped-literal array narrows silently through inference (CLOSED 2026-08-26, was RE-OPENED, MEDIUM)

`let t = [1, 2]; S { a: t, b: 42 }` with `a: [2]u8` stores `a[1] == 0` on
all engines. Root cause: an array of unsuffixed integer literals is never
grounded to a concrete element type. The array-literal unification fix
narrowed this but did NOT close it: the P1 workstream found a live shape —
a generic struct literal whose array field is all-unsuffixed
(`let w: Wrap[[3]i64] = Wrap { v: [4, 5, 6] };`) instantiates the generic
at an ungrounded element type while the annotation instantiates it at the
concrete one, producing two divergent instances of one type (oracle faults;
MIR/native refuse). Four-line repro in the P1 workstream report. Needs the
checker to ground literal element types against the expected type before
the monomorphization shape is recorded.

CLOSED by the P5 grounding workstream (adversarially reviewed, SHIP): an
unsuffixed literal array now grounds its element type from the expected type
where one exists (with per-element E0709 range checks), and to the `i64`
default at the escape points (unannotated `let`, generic type-argument
binding) — no `{integer}` reaches a layout or a monomorphization shape
(debug-asserted at `record_inst`). Both repros flip: the Wrap shape checks
clean and runs `[4,5,6]` on every engine; the binding shape is E0703.
Residual, still open as its own item: P15 below (array-literal-as-index-base
still silently narrows on the oracle). New same-family scalar finding: P16.

## Disposition

P3 and (likely) P5 are closed by the in-flight checker workstream. P1, P2,
and P4 are open, each warranting its own fix + adversarial review; P2 folds
into the overwrite-drop / assignment-drop workstream (same init-tracking
substrate). None block the native-rollback launch precondition, but P1 and
P4 should land before any 1.0 conversation — P4 is a borrow-model soundness
hole and P1 silently computes wrong results in generic code.

## P6 — Orphan-task crash on parent fault inside an open scope (from the rollback review)

A parent that faults between `spawn` and the scope's join `_longjmp`s without
joining; orphaned task threads then dereference the cleared `CURRENT` runtime
pointer and abort the process (reproduced ~60-80% of runs, identical before
and after the rollback change). A theoretical follow-on: an orphan surviving
into the next in-process JIT run would see the new runtime's `live_tasks == 0`
and could lower a live bump via `rt_stack_restore` — unreachable today because
the null-`CURRENT` abort always wins first, but it becomes reachable if the
orphan crash is ever "fixed" by making `rt()` null-tolerant. Fix the orphan
lifecycle (join-or-detach on the fault path), not the symptom.

# Ledger additions from the P4-fix adversarial review (2026-08-18)

Pre-existing holes found while attacking the P4 (arrays-of-borrows) fix. All
four reproduce on the pre-P4-fix binary, so none are regressions of that
work; each is logged here to be fixed in its own workstream, not in the P4
one. Source: the 2026-08-18 adversarial review of the P4 fix.

## P7 — Out-mode borrow escape (HIGH, pre-existing)

A borrow written into an `out`-mode parameter slot leaves the callee without
any provenance or region check: the return-provenance walk only covers
`return` expressions, so a callee can store a borrow of its own local into
the caller's slot and the caller then reads a dead frame. Applies to plain
borrows and to arrays of borrows alike (the review has runnable repros for
both). The fix belongs with the out-slot machinery, next to the E0806 walk.

## P8 — Match-arm borrow shedding (HIGH, pre-existing)

A borrow value produced as a `match` arm's result sheds its loan on the way
out: the carried-loan protocol anchors at `let`/assignment landing sites and
extends through calls, but the arm-result path does not re-carry, so the
landing binding holds an unguarded borrow and the borrowed place reopens.
Plain borrows and arrays of borrows both take this path. (The return-side
walk already recurses into `match` arms; the value-side carry does not.)

## P9 — E1006 annotation-position gap (LATENT, pre-existing)

E1006 ("a borrow type is not a legal type argument", now also arrays of
borrows) fires at explicit type-argument positions. A type argument that is
only ever *inferred* through an annotation position is not routed through
`check_arg_conformance`, so a borrow-storing argument could be laundered in
by inference. Today the gap is unreachable in practice only because the
generic-monomorphization array bug (P1 above: array type arguments render
with length 0) breaks the carrier shapes first — the in-flight generics
workstream that fixes P1 may unblock this path, so re-test E1006 coverage
when P1 lands.

## P10 — Block-scope dangle for plain borrows (MEDIUM-HIGH, pre-existing, now confirmed)

Confirmed by repro during the P4 work: `let mut b = read x; { let y = 3;
b = read y; } return deref b;` checks clean and returns the dead slot's
value. A scope exit is neither a use nor a def in the loan liveness scan,
and no rule ties a loan's life to its referent's scope, so a borrow of a
block-local outliving the block escapes. Arrays of borrows now inherit
exactly this (and no worse) behavior by parity; fixing it means tying loan
places to the scope depth of their roots, a separate workstream.

## P11 — Interface methods via associated types shed their return borrow (HIGH)

`method_returns_borrow` reads the method's declared return type before
substitution, so a method whose signature returns `Self::Item` never
registers as borrow-returning even when `Item` is bound to a borrow (scalar
`read i64` and array `[2]read i64` alike). The landing binding discards the
receiver's loan and a later write through the "live" borrow checks clean and
is observed at runtime. Distinct from the (closed) B1 predicate split: this
is a substitution-ordering hole and it predates the borrow-array work
(reproduced on both binaries). Fix direction: consult the substituted return
type where the call is checked.

## P12 — Instance-name structural prefixes forgeable by user type names (MEDIUM)

The mangling scheme's verbatim fast path leaves plain identifiers unencoded,
so a user type literally named `arr3` (or `arr`, `slice`, `ptr`, `Box`, ...)
can collide with the structural array form in single-file programs
(module-qualified names encode and cannot collide). Pre-existing class,
net-improved by the injective-mangling work (the plain `arr[T]` collision is
now fixed); eliminating the class means encoding every name and accepting
instance-name churn for plain names — a recorded trade-off, not an
oversight.

## P13 — Selfhost mangling does not mirror the qualified-name encoding (LOW)

The selfhost lowering emits type and length names as raw source spans where
the reference now emits the length-prefixed encoding; a selfhost fixture
using a qualified or underscored name in a generic instance would fail
loudly (name mismatch against the reference-built tables), not miscompile.
No fixture hits it today.

## P14 — Pairwise array-literal loan scan double-counts one exclusive loan (COSMETIC)

`[b, b]` where `b` is an exclusive borrow emits a spurious extra E0801
beside the pre-existing E0301; the two propagated copies of one loan are
counted as a conflicting pair. Can only co-occur with E0301, so no legal
program is rejected.

## P15 — Float-to-small-integer conversion panics both Cranelift engines (HIGH, compile-time)

`conv u8` / `conv u16` / `conv i8` of an f64 crashes the Cranelift JIT (both
tiers) and AOT at COMPILE time (cranelift-codegen 0.132.3's x64 emitter hits
unreachable code lowering a sub-32-bit saturating float convert), while the
tree-walk oracle and MIR interpreter execute the same program correctly. An
engine-parity hole outside the float gates' coverage (they only test conv
i32/i64). Minimal repro in the raytracer test header and MEMORY.md. Fix
direction: widen the convert to i32 in the lowering and narrow after. Found
by the ray tracer workstream, 2026-08-26.

## P16 — Generic functions do no return-borrow loan extension (HIGH)

`fn idr[T](p: read T) -> read T` lets the caller shed the argument loan
entirely: the returned borrow carries nothing, so writing the borrowed-from
owner while the result lives checks clean. Same laundering family as P9/P1;
found on the baseline binary during the P7/P8/P11 workstream (its fixes
cover interface methods and out-slots, not plain generic returns).

## P17 — Native trace order around joins diverges from the oracle (MEDIUM)

Native engines merge a task's trace at the JOIN; the oracle emits it at the
SPAWN point. Any parent trace between spawn and join reorders theta
deterministically (oracle [1,99] vs native [99,1]). No existing gated test
has a tracing task plus a tracing parent, so the differential suite never
sees it — a live hole in the per-task projection equivalence claim (design
0012 §6). Pre-existing; surfaced by the orphan-lifecycle review, 2026-08-26.

## P18 — Call-free infinite loops: oracle faults, native spins (MEDIUM)

A call-free spin loop (while f.* == 0 {}) exhausts the oracle's model stack
(it allocates a temp per iteration) and faults bad_pointer, while native
binaries — whose per-statement state stays flat — spin forever. Both
directions of the asymmetry are pre-existing; it also bounds what the
orphan-fix's unconditional fault-path joins can promise (a non-terminating
task hangs the join, matching the already-shipped normal-path join). Fix
directions to weigh: an iteration budget in the oracle mirroring MAX_ADDR
semantics deliberately, or documenting the asymmetry as a model limit.
Surfaced by the orphan-lifecycle review, 2026-08-26.

# Ledger additions from the P5-fix adversarial review (2026-08-26)

## P19 — Array literal as an index BASE escapes grounding (MEDIUM, pre-existing)

`[[1, 2], [3, 4]][0]` into a `[2]u8` slot still silently narrows on the
oracle (the base literal is materialized at the i64 stride while the slot
copies u8); MIR/native refuse the shape. An index base is a non-propagating
position (deliberately — the slot's element type does not describe the
base), so the P5 grounding never sees it, and the base's own landing is a
temporary, not a binding. Pre-existing on both binaries; stays open as its
own item, not folded into P5's closure.

## P21 — Scalar literal range checks ignore the expected type, both directions (HIGH)

Queued fix workstream, not fixed by the P5 work (which covers arrays only).
Direction 1: `let x: u8 = 300;` checks clean and stores 44 on every engine —
silent truncation, violating spec 01 §3.3 ("an over-range literal is
rejected at compile time"); same through a binding (`let x = 300; let y: u8
= x;` is 44). Direction 2: an in-range `u64` maximum literal is REJECTED,
because `check_int_lit_range` runs against the `i64` default at literal
sight, before the expectation is known — the same root defect pointing the
other way. Fix belongs where the expectation is applied (`check_against`),
mirroring what the array elements now get.

# Ledger additions from the P7/P8/P11-fix adversarial review (2026-08-26)

Pre-existing holes found while attacking the P7/P8/P11 fixes. All reproduce
on the pre-fix binary, so none are regressions of that work; each is logged
to be fixed in its own workstream.

## P22 — Generic bodies are Proj-opaque and generic calls shed all loans (HIGH)

Two halves of one family. (a) Generic function bodies are checked ONCE at
the definition site with opaque type parameters, so `Type::Proj`
(`I::Item`) defeats all three of the P7/P8/P11 fixes inside a generic body:
an assoc-typed method return sheds the receiver loan (`fn leak[I: Get](it:
write I) -> I::Item` — reviewer repro k3), an `out I::Item` slot escapes
provenance (repro l2), and a match joining to a Proj result sheds its arm
loans (repro s1). (b) Independently, `check_generic_call` ends with an
unconditional `clear_carried`, so EVERY generic free function returning a
borrow or view sheds its argument loan at the call site — `fn idr[T](p:
read T) -> read T` lets the caller write the borrowed owner and observe it
(repro q1 runs to 9 through a "live" shared borrow). All pre-existing; the
CONCRETE halves of these shapes are closed by the P7/P8/P11 workstream. The
generic halves need per-instantiation borrow information or def-site Proj
bounds — the next major checker workstream.

## P23 — Checker panic on a borrow-returning call with too few arguments (MEDIUM)

`fn g(p: read i64) -> borrow i64` called as `g()` panics the checker
(index out of bounds in `check_user_call`'s return-extension, currently
`check/expr.rs:1697`: `per_arg` is zip-truncated to the argument count but
`region_source_indices` indexes by parameter position) AFTER the correct
E0706 arity diagnostic is queued. Reviewer repro i2. One-line `.get` fix in
its own change; a panic, not an unsoundness.

## P24 — extern/fn-pointer signatures skip the out-slot E0807 rule (LOW)

The P7 signature rule (a borrow-storing `out` parameter with two-plus
borrow inputs is E0807) runs where function bodies are checked, so a bare
fn-pointer TYPE or an extern declaration with that shape is not itself
rejected. Not exploitable from safe code: externs cannot declare borrow
types (the foreign mappability check rejects them: "a Candor borrow/mode is
not a C type") and any Candor callable matching
the shape is rejected at its own declaration, so no callee can exist; the
caller-side extension also refuses to guess with two-plus inputs. A latent
asymmetry to close if fn-pointer signatures ever get their own
well-formedness pass.
