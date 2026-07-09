# 0012 — Structured Concurrency

**Status:** draft
**Date:** 2026-07-09
**Philosophy hooks:** P10 (concurrency without coloring — the whole mandate),
NN#9 (no bidirectional type-transforming partition), P9 (no mandatory runtime;
spawned threads are explicit), P5/NN#20 (the fault window's synchronization bound
becomes non-vacuous here), P7 (one fault vocabulary and a root policy), P2/NN#19
(the effect set is closed at {alloc, foreign}; thread-safety without growing it),
P6 (small core; when you add, remove), P18 (consistency model adopted from proven
art; ch09 is triggered by this design).

## Problem

P10 requires the first stable version to ship **data-race freedom guaranteed by
the ownership model at compile time** (non-negotiable) and **structured
concurrency — scoped, joined, cancellable** — while shipping *no* `async`/`await`
and nothing with async coloring's shape (NN#9), and *no* green-thread runtime
(P9). P10 is explicit about the ceiling: the answer is threads plus explicit
completion-driven state machines, capable-but-manual, and the partition-free
high-concurrency-I/O question stays **open research this document does not solve.**

This design fixes the construct shape, what may cross a task boundary and by what
mechanism, the data-race-freedom argument, fault behaviour across tasks, and the
fragment of ch09 (consistency model, ADOPTED-PENDING) that this triggers. It must
not contradict the fault-window formalization §12 (the concurrency-composition
CONJECTURE); it should *sharpen* it — the choices below determine which of O1–O5
become provable in the shipped fragment and which stay open.

## Decision

### 1. The model — `scope` / `spawn`, fork-join, no shared mutable state

**Ruling: a block-structured `scope { … }` keyword; `spawn` a statement operator
inside it whose arguments carry the ordinary passing modes (0001 §3.1); every task
joins at the closing brace; NO detached threads, NO shared mutable state, and NO
synchronization primitive beyond the join edge — no mutex, no channel, no
user-facing atomics — in v1.**

#### 1.1 The construct

```
scope {
    spawn check_module(take mod_a, read symtab);
    spawn check_module(take mod_b, read symtab);
}   // join point: both tasks are terminal (returned or faulted, §3) before control passes
```

`scope` opens a **concurrency region** with one guaranteed property: **no task
spawned inside it outlives the closing brace.** The brace *is* the join barrier.
Making it a *block* rather than a value delivers the invariant structurally: there
is no join-handle value to store, return, or leak, so "a task escapes its scope"
is not expressible, not merely rejected.

`spawn CALLEE(ARGS);` starts one task. The callee is a **fn or fn-pointer**; its
body is the capture-free `fn + explicit context` idiom 0009 §5 made load-bearing —
**there are no task closures**, so none of the region-typed-closure machinery 0009
§5.1 refused is dragged in. State a task needs travels as its arguments, threaded
explicitly and visibly.

**Why a keyword, not a library fn (the Iter precedent, decided the other way).**
0009 shipped iteration as library interfaces plus *one* keyword (`for`) because the
open-coded walk was the measured friction. Here the library form —
`scope(ctx, fn(s: read Scope, ctx) { … })` — carries a `Scope` handle that must not
escape the body, needing either a new non-escaping rule or a second data-threading
pass. The block form makes escape structurally impossible and reuses the *lexical
region* the loan checker already understands (0001 §2.3). Two keywords earn their
slots by the test `for` passed: concurrency is "dangerous meaning," so it wears
words (P13).

#### 1.2 What crosses a spawn boundary — the four modes, reused

`spawn` adds **no new passing mode**: arguments use the same four modes as any call
(0001 §3.1), and each mode *is* the data-race story:

| At the spawn site | Mechanism | Effect on the parent |
|-------------------|-----------|----------------------|
| `take arg` | **ownership transferred** into the task (moved, or copied if `copy`) | parent has moved it out — no alias remains |
| `read arg` | a **shared-read loan** on the referent, held for the **whole scope** | parent frozen to shared on that place until the brace |
| `write arg` (incl. `slice_mut`) | an **exclusive loan** on the referent, held for the **whole scope** | parent frozen out of that place until the brace |
| `out arg` | **rejected across a spawn** (no defined join-time single-writer story in v1) | — |

`take` gives the task **exclusive owned data** — the cleanest DRF story, no sharing.
The borrow cases are the interesting ones; §2 walks how the *existing* XOR machinery
makes cross-thread mutation-rejection fall out with no new aliasing rule. The only
novelty is the loan's live range: **a spawn-crossing borrow's loan lives for the
enclosing scope's whole extent**, not to its last syntactic use (the task may touch
it anywhere up to the join). That is one extension of 0001 §2.3's live-range concept,
not a new analysis.

#### 1.3 No synchronization primitive ships — argued from the basket and P6

**v1 is pure fork-join data-parallelism: no `Guarded[T]` (mutex-analog), no
channels, no user-facing atomics.** The argument is basket-grade:

- **A parallel checker** splits modules across tasks; each *owns* its partition
  (`take`) or *shares-reads* the immutable symbol graph (`read`) and *returns an
  owned result*. Aggregation is post-join, single-threaded. **No shared mutable
  state** — the join edge is the only synchronization, and it is structural.
- **A work-split allocator / parallel fill** partitions a buffer into **disjoint**
  chunks; each task mutates its own chunk through a disjoint `slice_mut`. This needs
  *partitioned exclusive* access, not *shared* access — served by `split_mut` (§1.4),
  which XOR already blesses (disjoint places do not conflict, 0001 §2.2). No mutex.

Shipping a lock would be expensive in the coin P6 guards: a `Guarded[T]` or channel
is a runtime primitive with an atomics surface, forcing ch09 §3.2 (the atomic
orderings) to be transcribed now, which triggers the fault-window O1–O5 composition
obligations the formalization records as **CONJECTURE, undischargeable before the
atomics surface exists** (fault-window §13.1). Fork-join's sole sync edge — the join
— is a full barrier the *native runtime* provides (thread create/join), not a
surface the user programs, so the DRF story in v1 is **purely structural** and needs
no user atomics. This settles ch09 §4.1–§4.2 for this edition (§4.1 below).

#### 1.4 Disjoint exclusive partitioning — `split_mut`

The one library primitive v1 adds for parallel *mutation*:

```
fn split_mut[r, T](s: write[r] [T], mid: usize) -> (write[r] [T], write[r] [T])
```

It reborrows one exclusive slice as **two provably-disjoint** exclusive slices
`[0, mid)` and `[mid, len)` (bounds-faulting if `mid > len`). Each half `take`s into
a different task. Being disjoint places, the two exclusive loans do **not** conflict
under XOR (0001 §2.2), and each is the unique exclusive accessor of its bytes — so
siblings mutate disjoint memory with no race, adjudicated by the same loan scan that
adjudicates single-threaded borrows. (The prototype's whole-array index conservatism,
0001 §2.2, is why partitioning needs a primitive that produces *statically* disjoint
slices rather than relying on `a[i]`/`a[j]` disjointness the checker does not track.)

### 2. Data-race freedom — the guarantee falls out of XOR

The compile-time guarantee P10 demands is not a new check; it is the existing loan
discipline with spawn-crossing borrows given scope-length loans (§1.2).

#### 2.1 Cross-thread mutation is rejected by the same XOR rule

At the closing-brace horizon every loan created by every `spawn` is simultaneously
live (each lives the whole scope, §1.2). The XOR rule (0001 §2.2) — *any number of
shared xor exactly one exclusive, per place* — then decides every cross-thread
aliasing question with no addition:

- Two tasks `read` the same data → two shared loans → **legal** (shared xor).
- Two tasks `write` overlapping data → two exclusive loans overlapping → **rejected.**
  Disjoint exclusive loans (`split_mut`, §1.4) do not overlap → legal.
- One task `write`s, the parent also touches the place in-scope → the exclusive loan
  is live across the region → parent frozen out → **rejected.**
- One task `read`s while a sibling `write`s the same data → shared + exclusive loan
  on one place → **rejected.**

A data race is *two accesses to one location, at least one a write, unordered.* Every
such pair maps to a shared+exclusive or exclusive+exclusive loan overlap across the
scope, which the XOR scan already rejects. **Races are unrepresentable in safe code
because the loan scan that already runs makes the racing program not type-check** —
no new rule, the strongest possible form of the P10 guarantee (the guarantee *is* the
memory model, ch09 §1.2).

#### 2.2 The thread-crossing property — `portable`, structural, not an effect

Ownership-transfer (`take`) needs a predicate: which types may be moved into a task?
This is the Send question, and NN#19 forbids answering it by growing the effect set.
It is answered by a **built-in structural property**, computed exactly like `copy`
(0001 §1.3) and spelled `portable`:

> A type is **`portable`** iff it transitively contains **no `rawptr`** and **no
> borrow** — owned value data: scalars, `bool`, `unit`, `portable` structs/enums,
> `[N]T` and `Box T` where `T` is `portable`.

`portable` is a checker-computed predicate, not a tracked effect (NN#19 intact) and
not a user-implemented interface (avoiding 0007's coherence machinery for a purely
structural property). It surfaces in source only as a bound on generic spawn helpers
(`fn parmap[T: portable, …]`) and in diagnostics. A `take arg` is well-formed iff its
type is `portable`. `Box T` is `portable` when `T` is, because its pointer is
*unique-owning* (moved, never aliased) — categorically unlike a `rawptr`.

**Candor needs no `Sync` analog.** A shared borrow across a spawn (`read arg`) is
always safe to hand to any number of tasks, because Candor ships **no interior
mutability** (0001 §4.3): a shared borrow *cannot* mutate, so "shared = immutable =
safe to share" holds with no exceptions. The Sync predicate Rust needs to carve
interior-mutable types out of "freely shareable" has nothing to carve here. Only the
move property (`portable`) needs a marker; the share property is universal — a direct
dividend of the 0001 §4.3 decision.

#### 2.3 `rawptr`, the allocator, and the honest hole assigned to the unsafe author

`rawptr` cannot be classified structurally: whether a raw address is safe to use from
another thread is *semantic*, unknowable to the checker (a mutex protects its payload;
a bump pointer does not). So `portable` **excludes any type containing `rawptr`**, and
the consequences are named:

- The **`Alloc` handle is not `portable`** — a `copy` struct with `rawptr` fields
  (0001 §6.1). This is *correct*: `Alloc` is `copy`, so moving it into a task leaves a
  live copy in the parent, and both would race on the allocator's free list. A
  general-purpose allocator is not thread-safe, and the rawptr exclusion is exactly
  what stops safe code from sharing one by accident.
- **The clean parallel-allocation idiom is a per-task arena over portable owned
  memory.** A `Box [N]u8` (or owned byte array) *is* `portable`; a task that `take`s
  its own backing buffer and bump-allocates within it touches only task-local owned
  bytes — no shared allocator, no race. This is how parallel compilers actually
  allocate (per-thread arenas), and it is the idiom v1 blesses.
- **Sharing a genuinely thread-safe allocator is an `unsafe`-declared decision.** To
  move a rawptr-bearing handle into a task, the author constructs the spawn argument
  under `unsafe "this allocator is thread-safe"` — the trust becomes visible,
  greppable, enumerable (P1/P17), and the checker's silence is honest. It is the
  fault-window §13.4 (iv) "racy/unsafe cross-thread channel is the unsafe author's
  declared responsibility" clause, met where it arises.

This discharges the P2 argument for concurrency: thread-safety is a **structural type
property** (`portable`) plus a **declared trust boundary** (`unsafe`) for the case no
structure can decide — never a new tracked effect.

### 3. Faults across tasks

#### 3.1 Delivery at the join edge — §12.2 instantiated, and provable here

The root fault policy (P7) is program-wide: a fault is a bug manifesting and routes to
the root-declared handler (abort / halt-and-log / user handler). This design adds one
guarantee:

> **A faulting task's fault is delivered at or before its join edge.** The scope's
> closing brace is that task's bounding synchronization operation; the fault is
> delivered before the join's release/acquire retires, so no sibling and not the
> parent observes any post-window state of the faulting task through the join.

This is precisely the fault-window §12.2 CONJECTURE (*delivered before any subsequent
synchronization operation of the faulting thread retires*) — and in the fork-join
fragment it is a **provable instance**, because the join is the *only* synchronizes-
with edge a safe task has (§1.3). The window bound `e⁺` (fault-window §4.1) is the
join edge; truncation says nothing at or after `e⁺` retires, so the faulting task's
owned result is *never delivered to the parent* — the join receives the **fault** in
its place. There is no partial result to leak, because the result crosses only *at*
the join, which does not retire.

The structural invariant does the rest: because *no task outlives the scope* (§1.1),
the parent cannot pass the brace until **every** sibling is terminal. So on a task
fault the scope: (1) records the fault; (2) requests cancellation of all siblings
(§3.3); (3) **joins all siblings** — each completes or reaches a cancellation point
and unwinds cooperatively, never a forced kill; (4) delivers the selected fault (§3.2)
to the parent *at the brace*, where the root fault policy fires in the parent's
context. Under a root policy of `abort` an implementation may abort at step 1; the
*semantics* is nonetheless "delivered no later than the join edge."

#### 3.2 Deterministic fault identity — spawn-order-first

Program-order-first (fault-window §6.2 Option A) pins fault identity in one thread;
across threads there is no total program order, so this design supplies the tie-break:
**the delivered fault is that of the spawn-order-least task that faulted.** Spawn order
is a deterministic total order (the textual order of `spawn` statements), and the
**join-all invariant makes selection decidable**: by the brace every task's outcome is
known, so the scope selects the spawn-order-least faulting task's `(k, s)` regardless
of wall-clock fault order. Fault identity is therefore **deterministic across
schedules and build modes** though scheduling is not — the concurrency lift of §6.2
Option A, preserving P5's "tested artifact behaves like the shipped one" and P4's
diagnostic determinism. (The advisory value-context `c` stays best-effort, §6.4.)

#### 3.3 Cancellation — cooperative, at explicit points, no async

P10 requires *cancellable*. Cancellation is **cooperative, never preemptive**: the
scope *requests* a sibling to stop but never *forces* it, because forcibly killing a
thread mid-mutation corrupts state — the unsound machinery a systems language must
refuse. No async, no unwinding runtime requirement (P7): cancellation is observed by
**ordinary control flow at explicit cancellation points.**

A cancellable task takes a `read Cancel` token as explicit context (§1.1); `spawn`
supplies it. The task polls it:

```
fn worker(take chunk, tok: read Cancel) -> Partial {
    let mut acc = seed();
    for item in chunk {
        if cancelled(tok) { return acc; }   // the cancellation point, visible in source
        acc = step(acc, item);
    }
    return acc;
}
```

`cancelled(read Cancel) -> bool` is a plain query; the task returns by ordinary
control flow. The discipline is honest and visible: between two `cancelled` checks a
task runs to completion, and a task taking no `Cancel` token is **non-cancellable** —
it always runs to completion and the scope waits for it. A nested `scope`'s join is
*also* an implicit cancellation point. The cost is named, not hidden: **a compute-
bound sibling with no cancellation points delays the scope's exit until it finishes.**
The idiom — a `cancelled` check in the outer loop of a long task — is the whole
answer; there is no free lunch and this design does not pretend there is one.

### 4. What this unblocks, and what stays open

#### 4.1 ch09's adoption fragment — triggered, and precisely which fragment

This design is the trigger ch09 named (ch09 §4.1). The fragment adopted for v1 is
narrow:

- **SC-for-DRF baseline (ch09 §3.1)** — adopted. Safe code is DRF (§2), so every safe
  program enjoys sequential consistency; there are no data races for a weaker model to
  describe.
- **The join edge as the sole cross-thread ordering** — the native runtime's thread
  create/join are adopted as a **release (at spawn) / acquire (at join)** pair from the
  C/C++ axis (ch09 §3.1). This is the *only* synchronizes-with edge in a safe program.
- **Atomics (ch09 §3.2) — NOT adopted this edition.** v1 ships no user-facing atomics
  surface (§1.3), so the orderings and fences stay deferred, exactly as ch09 §3.2
  permits. This is the argued answer to ch09's open question: **first-version Candor
  needs no user-facing atomics; scope-join edges suffice**, and the DRF story is purely
  structural.

The `Alloc`/rawptr trust boundary (§2.3) is the ch09 §3.3 "unsafe-code interactions"
surface, handled by assigning ordering responsibility to the unsafe author (fault-
window §13.4 iv), not by a v1 atomics surface.

#### 4.2 Which of O1–O5 this makes provable

The design determines the fate of the fault-window §12.3 obligations:

- **O1** (`W(f)` against synchronizes-with; `e⁺` well-defined): **provable in the
  fork-join fragment** — one synchronizes-with edge per task (its join), so `e⁺` is the
  join edge, trivially well-defined.
- **O2** (cross-thread containment via happens-before): **provable** over the join edge
  (the faulting result crosses only at the non-retiring join, §3.1); the relaxed-atomic
  half is **vacuous** (no relaxed atomics, §1.3).
- **O3** (truncation under interleaving): **provable** — no observable of a faulting
  task is ordered happens-after its bounding join, and siblings are independent
  (disjoint or read-only data, §2).
- **O4** (SC-for-DRF preservation on fault-free runs): **provable** — fault-free
  fork-join runs are DRF and enjoy the adopted SC; the window perturbs only faulting
  runs.
- **O5** (composition with the aliasing model across the safe/unsafe boundary): **stays
  open** exactly where a rawptr crosses a spawn under §2.3 — containment is claimed only
  over happens-before, not over racy/unsafe channels (fault-window §13.4 iv). The honest
  residual.

So this moves O1–O4 from CONJECTURE to provable-in-fragment and leaves O5 open at the
declared unsafe boundary. The general conjecture (§12.2 over an arbitrary atomics
surface) is *not* solved here; it reopens only when atomics are added by amendment.

#### 4.3 What stays open (P10, said plainly)

Ergonomic high-concurrency I/O stays **open research this document does not resolve**
(P10's honest accounting): the v1 answer for an io_uring-saturating storage engine is
threads plus explicit completion-driven state machines, capable but manual. A
partition-free model for that workload — if ever proven compatible with P9 and free of
async coloring — is an amendment, never this edition. User-facing atomics, a
mutex/`Guarded[T]`, and channels are deferred with it; each would require the ch09 §3.2
atomics adoption and the O1–O5 general discharge first.

### 5. NN#13 syntax walk

- **`scope`** — a **contextual keyword** in statement-leading position: `scope {`
  begins a concurrency region, tokenizing without a symbol table; elsewhere it is an
  ordinary identifier. The `{` after `scope` is unambiguously a block (never a struct
  literal — `scope` is not an expression head).
- **`spawn`** — a **contextual keyword** in statement-leading position inside a `scope`
  body: `spawn CALLEE(ARGS);` parses as the ordinary call grammar (0006) prefixed by
  `spawn`, reusing argument-mode and callee-path productions; elsewhere an ordinary
  identifier. A `spawn` outside any `scope` is a checker error, not a parse error — the
  grammar stays context-free (NN#13).
- **`portable`** — not a keyword: a **built-in bound name** resolved like `copy` (0001
  §1.3), appearing only in bracket bound-lists (`[T: portable]`, 0007 §6.1) and
  diagnostics; it reserves nothing globally.
- **`split_mut` / `cancelled` / `Cancel`** — library items (two fns, a compiler-known
  type), no grammar surface, migrated like any name (0007 §6.3).

No hard reserved word is added; every new keyword is contextual, consistent with the
`for`/`type` discipline of 0009 §4.4. The residual is the same narrow class: an
identifier named `scope`/`spawn` in the one statement-leading position the keyword now
claims. The corelib seed uses neither; the migrator flags any such use (P15).

### 6. Prototype stage plan and the nondeterminism gate

The tree-walking interpreter (0001 §7) and the MIR interpreter (0010 §5) are single-
threaded. The staging delivers the *compile-time* guarantee — where P10's value lives —
first, and never forces nondeterminism into the differential oracle.

- **Stage 1 — checker rules only (the bulk of the deliverable).** `scope`/`spawn`
  parsing; the scope-length loan extension (§1.2); the XOR fall-out (§2.1, reusing 0001
  §2.2 with no new aliasing rule); `portable` as a structural predicate beside `copy`;
  the `take`-must-be-`portable` and `write`-across-spawn rules; `split_mut` well-
  formedness; the `unsafe` rawptr-crossing trust point (§2.3). *Gate:* a negative-
  fixture suite in which **every** DRF-violating program (two writers, writer+reader,
  parent-touch-under-live-exclusive-loan, non-`portable` `take`) is rejected with the
  correct diagnostic and full provenance (P4), plus a positive suite (fork-join-of-
  owned, shared-read, `split_mut` partition) accepted. No execution nondeterminism
  exists here; it is a pure loan/property scan.

- **Stage 2 — deterministic sequential execution.** Execute a `scope` by running each
  spawned task **to completion in spawn order** on the single-threaded engine — a
  *sequential schedule*. This is a **valid** schedule: safe fork-join Candor is DRF (§2)
  and ships no user-observable nondeterministic sync (§1.3), so by SC-for-DRF every
  schedule yields the same observable trace up to the join edges; the sequential
  schedule's trace is *the* trace. The oracle stays deterministic and the 0010 §4 gate
  is unchanged: `(k, s, θ)`-trace equality across engines, fault by spawn-order-first
  (§3.2, decidable because sequential execution has run every task before the brace).

- **The nondeterminism gate (the honest extension of the differential methodology).**
  Trace equality needs deterministic schedules or trace-set semantics; the testable
  discipline for the real (threaded) native backend:

  1. **Interaction points are discrete and few.** Safe tasks interact *only* at
     `scope`/`spawn`/join/`cancelled` points — no shared mutable state between them
     (§2) — so the interleaving space is over these points, not over instructions.
  2. **Per-task projection.** `θ` is recorded with a per-event **task tag**. Two runs
     are **trace-equivalent** iff (a) each task's projection `θ|task_i` is equal, and
     (b) events ordered by a join edge respect that order; the relative interleaving of
     events on *disjoint* resources is not asserted. For a program whose tasks emit no
     observable until join (pure compute returning owned results — the parallel-checker
     shape), projections plus join order **fully determine** `θ`, collapsing to exact
     `(k, s, θ)`-trace equality with no weakening.
  3. **Shared observable sinks are a declared-nondeterministic seam.** A writable
     MMIO/device handle is *exclusive*, so XOR (§2.1) already forbids two safe tasks
     from holding one; the only genuinely shared safe sink is process `stdout`. Two
     tasks writing `stdout` is **declared nondeterminism** (P5); the gate tests the
     **trace-set** on that sink (multiset equal, global order not) and the exact
     projection everywhere else.
  4. **Bounded-schedule enumeration.** Because interaction points are discrete and few,
     a **preemption-bounded / partial-order (DPOR-style) enumerator** over schedules —
     branching only at `scope`/`spawn`/join/`cancelled` — is tractable. The gate asserts,
     for **every** enumerated schedule: the projection invariant (2), the declared-
     nondeterminism sink rule (3), and — the sharpest axis — the **spawn-order-first
     fault identity** (§3.2) identical across all schedules. This is 0010 §4's
     `(k, s, θ)` methodology with `θ` compared under task-projection equivalence instead
     of raw-sequence equivalence; the equivalence relation is *itself part of the spec*
     (the declared-nondeterminism boundary, P5), as 0010 §4.3 made the value-context
     exclusion part of the spec being tested.

  *Gate to pass Stage 2 → real backend:* native-threaded execution satisfies the
  projection + trace-set + fault-identity invariants on the basket's parallel ports
  under bounded-schedule enumeration; a schedule violating any invariant has, by the
  spec's definition (P18), changed behaviour, and the job fails.

## Rejected alternatives

- **`async`/`await`, or any effect-system spelling of it — rejected (NN#9, P10).** The
  bidirectional, type-transforming, API-duplicating partition is the schism P10 refuses
  in the first stable version; an effect-system re-encoding reintroduces it under a new
  name. Not reopened here.
- **Detached threads / a returnable `JoinHandle` — rejected.** A handle that outlives
  its scope breaks the structural invariant (§1.1) and re-admits use-after-free of
  borrowed data; the join point must be syntactic and unavoidable, and a join-handle
  value is exactly what §1.1 refuses to make expressible.
- **Unstructured `spawn` returning a future/handle — rejected** for the same reason,
  plus the handle would carry the task's borrows: a borrow-bearing field (0001 §3.4
  forbids it) or a region-typed handle (0007 §3.5's refused machinery).
- **A `Send`/thread-safety tracked effect — rejected (NN#19).** The effect set is closed
  at {alloc, foreign}; thread-crossing is a **structural property** (`portable`, §2.2)
  plus a declared `unsafe` trust boundary (§2.3), never a new effect — which would also
  make it a transitive call-graph partition (P2) for a structurally decidable property.
- **A `Sync`-analog — rejected as unnecessary (§2.2).** No interior mutability exists
  (0001 §4.3), so every shared borrow is immutable and universally shareable; nothing
  for a Sync predicate to carve out.
- **A mutex/`Guarded[T]` or channels in v1 — rejected (§1.3, P6).** The basket needs
  neither; both require the ch09 §3.2 atomics adoption and thereby the open O1–O5
  general discharge (fault-window §13.1) as a precondition. Deferred with the atomics
  surface.
- **Capturing closures as task bodies — rejected (already, 0009 §5).** Task bodies are
  `fn + explicit context`; closures drag region-typed-closure machinery the value-first
  bet exists to avoid. Reopen only via 0009's OBL-GENERICS-CLOSURE gate.
- **Preemptive cancellation — rejected (§3.3).** Forcibly stopping a thread mid-mutation
  corrupts state; cooperative cancellation at explicit points is the only sound option
  without an unwinding runtime (P7).
- **Library-fn `scope(fn body)` instead of a keyword — rejected (§1.1).** The scope-
  handle-escape problem needs new machinery or double data-threading; the block form
  makes escape structurally impossible and reuses the lexical loan region.

## Consequences and costs

*Debts, not absolutions (the philosophy's header warning).*

- **Ergonomic high-concurrency I/O is ceded in v1 (P10's Refusal, in force).** The
  storage-engine answer is threads plus explicit state machines; this design delivers
  the fork-join floor beneath it, not a narrowing of that Refusal.
- **No shared-mutable concurrency without `unsafe`.** A workload that genuinely needs a
  lock-protected shared structure (a concurrent work-stealing deque, a shared cache) has
  no safe path in v1 and must build it in `unsafe` over `rawptr` — visibly, as the
  unsafe author's declared responsibility (§2.3). If a future basket program makes this
  ambient rather than exceptional, that is a finding (the P10-vs-basket revisit), the
  recorded trigger to reconsider a `Guarded[T]`.
- **Shared allocation across tasks is an `unsafe`-declared decision (§2.3).** The clean
  path (per-task arenas over `portable` owned memory) covers the parallel-checker and
  parallel-fill shapes; a program that must share one allocator across tasks pays an
  `unsafe` trust boundary. Restrictive, and named.
- **A compute-bound task with no cancellation points delays scope exit (§3.3).**
  Cooperative cancellation has no free lunch; the idiom is a `cancelled` check in the
  long loop, and its absence is a latency the author owns.
- **The nondeterminism gate is weaker than single-threaded trace equality on shared-
  sink programs (§6).** For tasks sharing `stdout` the gate asserts a trace-*set*, not a
  trace, on that sink — an honest reduction in test strength, confined to a declared-
  nondeterministic seam and to `stdout` alone (every other observable sink is exclusive
  by XOR).
- **The P6 budget line.** Added: two contextual keywords (`scope`, `spawn`), one
  structural predicate (`portable`), one library fn (`split_mut`), a minimal
  cancellation surface (`Cancel`/`cancelled`). *Not added, by the same discipline:* the
  entire synchronization zoo — mutex, channels, atomics, a Sync predicate, a Send
  effect, join-handle values. The marginal core growth is two keywords plus a scope-
  length extension of an existing loan concept; the DRF check, the passing modes, and
  the fault mechanism are all **reused, not grown.** The budget is spent on exactly the
  surface P10 mandates and nothing on the surface it does not.
- **O5 stays open (§4.2).** Cross-thread containment across the safe/unsafe rawptr
  boundary is not proved; it is *assigned* to the unsafe author (fault-window §13.4 iv)
  — a named debt on the formalization's ledger, not a closed one.

## Reclassification record

None. This design does not turn on the §2 rule of reclassification: the concurrency
mandate is P10 (Priority 1 soundness — DRF is non-negotiable), the construct-shape and
no-lock calls are argued from P6/P9 and the basket, and the syntax calls are ordinary
P13, not an item-7→item-3 promotion.
