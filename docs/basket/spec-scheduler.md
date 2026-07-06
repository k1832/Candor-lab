# Spec: Intrusive-List Priority Scheduler (`spec-scheduler.md`)

**Status:** FROZEN on hash. Authored blind to Candor design docs (see README).
**Source obligation:** `BET5_CRITERION.md` §2.4(b). Restates and sharpens; never
weakens.

---

## 1. Purpose & required features

1.1 The program is a **task scheduler** built on **intrusive doubly-linked
lists**: the list linkage (forward and backward links) is **embedded in the
scheduled entity** (the task), and the scheduler **does not own** the tasks.
Tasks live in caller-owned storage; the scheduler only threads them onto queues
through their embedded link fields.

1.2 The following **do not qualify** (criterion §2.4b) and MUST NOT be used to
implement the queues: an owning container of tasks (a growable vector of tasks, a
node-owning list that allocates per element), or an **index-into-an-array**
rewrite that replaces pointer linkage with array indices. The linkage MUST be
intrusive pointers/handles into the tasks themselves.

1.3 Operations MUST be **O(1)**: insertion at a queue tail, and **removal of an
element from the middle of a queue given a handle to that element** (no scan).

1.4 The scheduling policy is **fixed by this spec** to remove ambiguity:
`NPRIO = 4` strict priority levels numbered `0` (highest) .. `3` (lowest); within
a level, ordering is **FIFO / round-robin** (first admitted runs first; a
yielding task goes to the tail of its level). `pick_next` always returns a task
from the **highest-numbered-priority non-empty level** (0 before 1 before ...).

1.5 The scheduler MUST support at least these lifecycle transitions: **spawn/admit,
block, wake, yield, set-priority (reschedule), exit** — forcing mid-list removal
(block/exit of a task that is not at a queue head) and re-insertion (wake).

1.6 The scheduler MUST NOT crash/panic on invalid requests; all errors are values
(§2).

**Ambiguity resolved (criterion §2.4b, "priority run-queues *or* round-robin").**
This spec mandates **both together**: strict priority across levels, round-robin
FIFO within a level. This is stricter than either alone and makes an
easier single-queue FIFO port detectably fail (T5, T7).

---

## 2. Abstract interface

Names indicative; semantics binding. A **handle** uniquely identifies a task and
permits O(1) access to its embedded links. Errors are returned values.

2.1 `init() -> Scheduler` — empty scheduler; all queues empty; no running task.

2.2 `admit(s, task, prio) -> Result<(), SchedError>` — insert `task` at the
**tail** of run-queue `prio`; task state becomes READY. Errors:
`E_BAD_PRIO` if `prio` not in `0..NPRIO`; `E_ALREADY_QUEUED` if `task` is already
in any queue or is RUNNING.

2.3 `pick_next(s) -> Option<handle>` — select the head task of the lowest-numbered
non-empty level, **remove it from its run-queue**, mark it RUNNING, and return its
handle. Returns `NONE` (a value, not an error) if all run-queues are empty.

2.4 `yield_current(s) -> Result<(), SchedError>` — the RUNNING task returns to
READY at the **tail of its own priority level**; no task is RUNNING afterward.
Error `E_NO_RUNNING` if there is no RUNNING task.

2.5 `block(s, task) -> Result<(), SchedError>` — remove `task` (READY in a
run-queue, or RUNNING) from wherever it is (O(1) via handle) and mark it BLOCKED
(tracked in a blocked set, not in any run-queue). Errors: `E_NOT_SCHEDULABLE` if
`task` is BLOCKED or EXITED or unknown.

2.6 `wake(s, task) -> Result<(), SchedError>` — move a BLOCKED `task` to the
**tail** of its priority level, state READY. Errors: `E_NOT_BLOCKED` if `task` is
not currently BLOCKED.

2.7 `set_priority(s, task, prio) -> Result<(), SchedError>` — reschedule: if
`task` is READY, remove it from its current level and insert at the **tail** of
level `prio`; if BLOCKED, record the new level to be used on the next `wake`; the
task's priority field is updated. Errors: `E_BAD_PRIO`; `E_NOT_SCHEDULABLE` if
EXITED/unknown. A RUNNING task's priority may be set; it takes effect at its next
enqueue (yield/block+wake).

2.8 `exit(s, task) -> Result<(), SchedError>` — remove `task` from all structures
(O(1)); state EXITED; it is no longer schedulable until re-`admit`ted. Error
`E_NOT_SCHEDULABLE` if already EXITED/unknown.

2.9 `queue_dump(s, prio) -> [handle]` — informative: the ordered list of handles
in run-queue `prio`, head first. Used only for test assertions (§4).

---

## 3. Observable-behavior requirements (invariants)

3.1 **Strict priority.** `pick_next` never returns a task from level `k` while any
task exists in a level `< k`.

3.2 **FIFO within level.** Among tasks admitted/woken/yielded into the same level,
`pick_next` returns them in the order they entered that level's tail.

3.3 **Single-membership.** At any instant a task is in exactly one of: a single
run-queue, the RUNNING slot, the blocked set, or EXITED/not-present. Never two at
once.

3.4 **List integrity.** For each run-queue: it is a well-formed doubly-linked list
— for every internal node, `node.next.prev == node` and `node.prev.next == node`;
the head has no predecessor, the tail no successor; the number of reachable nodes
equals the tracked length; there are no cycles other than a sentinel structure if
used.

3.5 **Middle-removal correctness (suite-verified); O(1) and intrusiveness
adjudicator-inspected.** `block`/`exit`/`set_priority` of a task that is neither
head nor tail of its queue MUST leave the remaining queue a well-formed list with
that task absent and the relative order of the others unchanged. This **removal
correctness** is what the suite verifies, via `queue_dump` and the list-integrity
checks of 3.4. The **O(1) cost** of the removal and the **intrusive doubly-linked**
representation (criterion §2.4b) are **not** observable from the suite's outputs —
a scanning or index-into-an-array implementation could produce identical dumps — so
they are **confirmed by adjudicator code inspection under criterion §6.5**, not by
the tests. The requirement itself (1.1, 1.3) stands unchanged; only the claim that
the suite observes it is withdrawn.

3.6 **Determinism.** Given an operation sequence, the sequence of `pick_next`
results and every `queue_dump` are fully determined by this spec (see reference
semantics §4.5). No implementation freedom in ordering.

3.7 **No loss/duplication.** Every admitted task is, until it exits, reachable in
exactly one location (3.3); no task is silently dropped or duplicated.

---

## 4. Frozen test suite (language-agnostic vectors)

Tasks are identified by integer ids `0..63`. "dump(p)" = `queue_dump(p)` head-first.

### Nominal
- **T1.** `admit(0,prio=1)`, `admit(1,prio=1)`, `admit(2,prio=1)`;
  `dump(1) == [0,1,2]`.
- **T2.** After T1, `pick_next` returns 0, then (after `yield_current`) 1, then 2
  — FIFO within a level.
- **T3.** `admit(5,prio=0)` while level 1 non-empty: `pick_next` returns 5 first
  (strict priority, 3.1).
- **T4.** `yield_current` sends the running task to the tail of its level:
  admit [10,11] at prio 2; pick_next -> 10 (running); yield_current;
  `dump(2) == [11,10]`; pick_next -> 11.

### Priority / round-robin
- **T5.** Admit 20@p0, 21@p2, 22@p1, 23@p0. `pick_next` order MUST be
  `20,23,22,21` (p0 FIFO, then p1, then p2). A single-FIFO-queue port fails here.
- **T6.** `set_priority(21, 0)` (from p2 to p0) puts 21 at tail of p0:
  with p0 = [20,23] then reschedule 21 -> `dump(0) == [20,23,21]`.

### Mid-list removal / re-insertion
- **T7.** Admit [30,31,32,33] at p1; `block(32)` (middle); `dump(1) == [30,31,33]`
  and list integrity holds (3.4/3.5); `wake(32)` -> `dump(1) == [30,31,33,32]`.
- **T8.** Admit [40,41,42] at p1; `exit(41)` (middle); `dump(1) == [40,42]`,
  integrity holds; `exit(41)` again MUST error `E_NOT_SCHEDULABLE`.
- **T9.** Spawn/block/wake/yield/exit mixed: admit 50@p1(run via pick_next),
  block(50), admit 51@p1, pick_next->51, yield_current, wake(50);
  `dump(1) == [51,50]`.

### Boundary
- **T10.** `pick_next` on an empty scheduler returns `NONE` (value, not error).
- **T11.** All four levels populated; drain via repeated pick_next+exit; order
  across levels is strictly 0-first then 1,2,3; ends at `NONE`.
- **T12.** Admit the same id twice without exit -> second `admit` errors
  `E_ALREADY_QUEUED`.

### Error
- **T13.** `admit(prio=4)` -> `E_BAD_PRIO`. **T14.** `set_priority(task, 9)` ->
  `E_BAD_PRIO`.
- **T15.** `wake` a task that is READY (not blocked) -> `E_NOT_BLOCKED`.
- **T16.** `block` an already-BLOCKED task -> `E_NOT_SCHEDULABLE`.
- **T17.** `yield_current` with no RUNNING task -> `E_NO_RUNNING`.
- **T18.** `exit` an unknown id -> `E_NOT_SCHEDULABLE`.

### Deterministic pseudo-random stress (T19)

4.1 **PRNG.** xorshift64, identical to `spec-allocator.md` §4.1, but **seed
`0x9E3779B97F4A7C15`**:
```
state := 0x9E3779B97F4A7C15
next(): x:=state; x^=x<<13; x^=x>>7; x^=x<<17; state:=x; return x  (mod 2^64)
```

4.2 **Shadow model.** Maintain a language-neutral reference: four ordered lists
`Q[0..3]`, a `running` slot, a blocked set with each task's recorded priority, and
per-task state. Update the shadow by the exact rules of §2 for each drawn op.

4.3 **Step rule.** For `i` in `0 .. 20000`: draw `w := next()`;
- `id  := (w >> 3) mod 64`
- `op  := w and 0x7`  (values 0..7)
- Dispatch: `0,1` = admit `id` at prio `(w >> 9) mod 4` (skip if id already
  present); `2` = pick_next (ignore result value but apply state change); `3` =
  yield_current (skip if no running); `4` = block `id` (skip if not schedulable);
  `5` = wake `id` (skip if not blocked); `6` = set_priority `id` to
  `(w >> 9) mod 4` (skip if not schedulable); `7` = exit `id` (skip if not
  present). "skip" = the operation is expected to error/no-op; both shadow and
  implementation take no state change.

4.4 **Per-step assertions.** After every step: (3.3) single-membership across all
tasks; (3.4) list integrity for all four queues; and **`queue_dump(p)` for each
`p` equals `Q[p]` in the shadow model**, and the RUNNING/blocked/EXITED
classification matches. Any mismatch fails the suite.

4.5 **Reference determinism.** Because §2's ordering rules are total and the draw
rule is fixed, the shadow model is single-valued; every conforming implementation
MUST reproduce identical `queue_dump`s at every step.

### Reschedule on a BLOCKED task (additional nominal)

- **T20 (set_priority on a BLOCKED task takes effect at wake).** `admit(61,p0)`,
  `admit(62,p0)` so `dump(0) == [61,62]`; `admit(60,p2)`; `block(60)` (60 leaves
  p2 and becomes BLOCKED with recorded priority 2); `set_priority(60, 0)` — 60 is
  BLOCKED, so per §2.7 the new level 0 is recorded for the next `wake` and 60's
  priority field becomes 0, with **no** run-queue change yet (`dump(0)` and
  `dump(2)` unchanged at this point); then `wake(60)` moves 60 to the **tail of
  level 0** (its recorded new level), giving `dump(0) == [61,62,60]` and
  `dump(2) == []`. Confirms §2.7's BLOCKED branch: a priority change on a blocked
  task is applied on re-insertion, not immediately. (Numbered T20 — the next free
  index — to avoid renumbering the frozen error and stress vectors.)

---

## 5. Non-goals

5.1 **Preemption timing, time-slicing quanta, and real time** are NOT modeled; the
scheduler is driven purely by explicit operations.
5.2 **Thread safety / SMP / concurrency** is NOT required.
5.3 **Fairness beyond the specified strict-priority + FIFO policy**, aging, or
starvation avoidance is NOT required (and MUST NOT be added — it would change
observable order and fail §4).
5.4 **Memory allocation for tasks** is the caller's concern; the scheduler
allocates nothing per task (intrusive requirement, 1.1).
5.5 **Performance** is not graded; the suite grades the observable outcomes of
§3–§4, while the **O(1) cost shape and the intrusive doubly-linked representation
are adjudicator-inspected** (§3.5, criterion §6.5, §2.4b), **not** suite-graded
(criterion §8.2).

---

**Revision history.** 2026-07-06: revised per blind adversarial review #1 (`docs/reviews/2026-07-06-basket-specs-review-1.md`); findings 5, 12 applied.
