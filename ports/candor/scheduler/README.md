# Candor port: intrusive-list priority scheduler

Port of `docs/basket/spec-scheduler.md` (frozen) to Candor, per
`BET5_CRITERION.md` §2.4(b). One file, `scheduler.cn`, containing the
scheduler and the full frozen vector suite (T1–T20, including the T19
20,000-step stress with per-step shadow-model comparison). `main` returns the
sentinel **777** on full success; any vector failure faults (exit 2).

Toolchain: `prototype/target/release/candor-proto check|run ports/candor/scheduler/scheduler.cn`.
The implementation/harness split per ruling R14 (`// Test harness` marker) is
verified: the implementation section parses and checks standalone.

## Mechanism (spec 1.1–1.4, for the adjudicator)

Four strict-priority levels (0 highest .. 3 lowest), each a **circular
doubly-linked list with a sentinel** `Link` embedded in the `Sched` struct.
The linkage is **intrusive**: every `Task` embeds a
`Link { next: rawptr Link, prev: rawptr Link }`, and the queues thread
through those embedded fields. The scheduler owns no task and allocates
nothing (spec 5.4). Tasks live in caller-owned storage — a fixed
`[64]Task` arena local to each test vector — and a handle is a
`rawptr Task` into it. There is no owning container and no
index-into-an-array rewrite anywhere in the measured section: tail insert
and mid-queue removal are O(1) three-node pointer splices, and removal
recovers the `Task` from its interior `Link` by **container_of**
(`offsetof(Task, link)` + `ptr_offset` arithmetic, exactly design 0001
§11.2). The `link` field is deliberately the *last* field of `Task`, so the
container_of offset is non-trivial.

The blocked set is tracked purely by the per-task state field (a blocked
task is on no list); the RUNNING slot is a single nullable handle. Errors
are values (`SchedErr`); the scheduler never faults on invalid requests.

Adjudications honored: **R5** (admit valid only from New/Exited; admitting a
BLOCKED task is `E_ALREADY_QUEUED` — exercised in T12's extension and
throughout T19), **R6** (`pick_next` while a task is RUNNING is a no-op
`none` — exercised throughout T19), **R14** (marker split).

## Valve shape (Bet 5 data)

The valve concentrates exactly where design 0001 §11.2 predicts — the
intrusive linkage:

- `list_insert_tail` / `list_remove` — the doubly-linked splices.
- `task_of` / `t_link` — container_of and its inverse (`offsetof` + pointer
  arithmetic).
- `qsent` / `qsent_r` — addresses of the sentinels embedded in the `Sched`.
- `t_id`/`t_prio`/`t_state`/`t_set_prio`/`t_set_state` — field access through
  a `rawptr Task` handle (a rawptr has no safe field projection, so *every*
  task-field touch is a valve op; writes are field-precise via `offsetof` so
  a stale whole-struct copy can never clobber live link words).
- `sched_init`'s sentinel self-linking, `queue_dump`'s ring walk, and the
  inert `ptr_null` constructors.
- Harness: `arena_base`/`th` (handle derivation) and `check_ring`'s
  integrity walk.

Everything else — the operation-set control flow (`admit`, `pick_next`,
`yield_current`, `block`, `wake`, `set_priority`, `exit_task`), all error
values, priority policy, the T19 **shadow model** (safe arrays + indices —
it is the spec-4.2 oracle, not the measured scheduler), and all vector
orchestration — is safe value/borrow gear. Handles travel through safe code
as inert `rawptr` values; every operation that gives one meaning is inside
an `unsafe` block with a true justification.

## Deterministic stress (T19)

Exact spec 4.1–4.5: seed `0x9E3779B97F4A7C15`, 20,000 steps, ids 0..63, the
8-way op dispatch of §4.3. A "skip" draw still calls the scheduler and
asserts the exact error value (spec 4.3 says the op is "expected to
error/no-op"). After **every** step: `queue_dump(p)` equals shadow `Q[p]`
for all four levels, full list-integrity walk (back-links, ring closure,
cycle guard, reachable-count == tracked length), single-membership
occupancy over all 64 ids, and RUNNING/BLOCKED/EXITED classification +
recorded-priority comparison. The xorshift64 is the allocator port's
reference-verified construction (bit-loop XOR, wrapping-multiply /
unsigned-divide shifts).

**Runtime: the full T1–T20 suite runs in ~25 seconds** on the tree-walking
interpreter (vs ~20 minutes for the allocator's A21 — no byte-fill loops
here). Nothing was shrunk.

## Adaptations and ambiguities (flagged for adjudication)

1. **Spec 2.1 `init() -> Scheduler` is split into `sched_new()` +
   `sched_init(write s)`.** The sentinels are self-referential interior
   pointers, and a Candor move (like a C/Rust move) relocates the value —
   a by-value constructor would return a `Sched` whose sentinels point at
   the dead temporary. `sched_new` returns an inert (null-linked) value;
   `sched_init` self-links the rings in place at the Sched's final address.
   Post-condition of 2.1 holds after the pair. (The alternative — heads as
   nullable head/tail pointers, making `Sched` position-independent — was
   rejected: sentinel rings are the §11.2 idiom and keep `list_remove`
   branch-free.)
2. **`admit` error precedence** when `prio` is bad *and* the task is
   present is not fixed by spec 2.2; the port checks `E_BAD_PRIO` first
   (the spec's listing order). Unreachable from the frozen suite (T19's
   drawn prio is always in range).
3. **`set_priority` on a READY task to its *current* level** is read
   literally per §2.7: remove + insert at tail, i.e. a same-level
   reschedule moves the task to the tail of its own queue. The shadow model
   does the same, and T19 exercises this case (op 6 draws the current
   priority ~25% of the time), so a no-op-if-same implementation would
   diverge from this port's dumps. Worth a ruling if another port reads it
   the other way.
4. **`queue_dump` capacity** is fixed at 64 ids — the suite's id domain;
   single-membership (3.3) bounds any queue by the task count.

## Language friction notes (Bet 5 data)

1. **A block-like statement followed by a `(`-starting statement parses as
   a call.** `unsafe "…" { … }` / `while … { … }` / `if … { … }` followed by
   a statement beginning `(deref s).f = …` produces E0704 "`unit` is not
   callable": the block expression is parsed as a callee and the next
   statement's parenthesized place as its argument list (grammar 0002 §2.3's
   optional `;` after block-like statements interacts badly with `Postfix =
   Primary { "(" … }`). Workaround: a defensive `;` after the closing brace.
   Four sites in this port needed it. This is new friction not seen in the
   allocator port (whose statements after blocks never began with `(`).
2. **No safe field projection through a handle.** A `rawptr Task` cannot
   reach `t.prio` without the valve, so even trivially safe-looking reads
   (`t_id`, `t_state`) are unsafe ops — ~10 one-line accessor valves that a
   language with checked handle projection would not count. This widens
   valve *occurrence* (M2-style counts) beyond the genuinely dangerous
   splice code, for reads whose danger is only pointer validity.
3. **Explicit call-site reborrows remain a steady tax** (`write (deref s)`,
   `read (deref sh)` on every helper call under a borrow — dozens of sites
   in the harness). Known design cost (0001 §2.1); it is use-site tokens,
   excluded from M1, but the reading friction is real in a
   scheduler-shaped program where everything takes the same two borrows.
4. **Struct-wrapping for borrow-mode arrays** (allocator friction note 1)
   recurred: `Arena`, `Occ` exist only because `write [64]Task` /
   `write [64]usize` parameters do not parse.
5. Positive note: the checker's earlier false-positive minefield
   (allocator notes 2–3: `match`-inside-`while`, array stores under
   `if/else` in loops) was avoided by construction here (matches live in
   helper functions; per-op stress steps are separate functions), so no new
   E0304 was hit — but the port's *shape* was chosen defensively around
   those known bugs, which is itself a friction cost.
