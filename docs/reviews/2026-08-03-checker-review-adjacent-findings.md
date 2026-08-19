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

## P5 — Untyped-literal array narrows silently through inference (RE-OPENED, MEDIUM)

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
