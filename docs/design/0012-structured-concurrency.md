# 0012 — Structured Concurrency

**Status:** draft (reworked)
**Date:** 2026-07-09
**Philosophy hooks:** P10 (concurrency without coloring — the mandate), NN#9 (no
bidirectional type-transforming partition), P9 (no mandatory runtime), P5/NN#20 (the
fault window's synchronization bound becomes non-vacuous here), P7 (one fault
vocabulary), P2/NN#19 (thread-safety without growing the effect set), P6 (small core;
when you add, remove), P18 (ch09's consistency model is triggered here).

**Revision note (2026-07-09) — reworked in full against adversarial review #1**
(`docs/reviews/2026-07-09-design-0012-review-1.md`), the project's first design-tier
rejection (verdict (c)). The central claim ("races unrepresentable in safe code") was
**falsified by construction**: a `copy`-with-`rawptr` handle (the `Alloc` flagship,
0001 §6.1) crossed a spawn as a `read` borrow and then **copied out from behind that
shared borrow** (0001 §1.3–1.4) hands two tasks one mutable free list with **zero
`unsafe`.** All ten findings accepted; this revision implements the deciding
authority's directions. The safety core moved: the spawn-crossing gate is unified and
now gates borrow *referents* (§2); `split_mut` is a priced blessed primitive, not a
loan-scan fall-out (§1.4); result-return via write-borrowed slots is THE mechanism with
its containment restated honestly (§3.1); the §12 proof is replaced by per-edge
truncation (§4). **Preserved (what held):** the keyword construct (§1.1), cooperative
cancellation (§3.3), atomics deferral (§4.1), spawn-order-first fault identity (§3.2),
per-task arenas (§2.5), O4, and the NN#13 walk (§5). This rework returns to review (the
failed-retest discipline applies at design tier).

## Problem

P10 requires the first stable version to ship **data-race freedom guaranteed by the
ownership model at compile time** (non-negotiable) and **structured concurrency —
scoped, joined, cancellable** — while shipping *no* `async`/`await` and nothing with
async coloring's shape (NN#9), and *no* green-thread runtime (P9). P10 fixes the
ceiling: the answer is threads plus explicit completion-driven state machines, and the
partition-free high-concurrency-I/O question stays **open research this document does
not solve.** This design fixes the construct shape, **the single gate deciding what may
cross a task boundary**, the DRF argument, how results come back, cross-task fault
behaviour, and the fragment of ch09 (ADOPTED-PENDING) this triggers. It must not
contradict fault-window §12 (the concurrency-composition CONJECTURE); it *sharpens* it
— determining which of O1–O5 become provable in the shipped fragment.

## Decision

### 1. The model — `scope` / `spawn`, fork-join, no shared mutable state

**Ruling: a block-structured `scope { … }` keyword; `spawn` a statement operator inside
it whose arguments are decided by the unified gate (§2); every task joins at the closing
brace; NO detached threads, NO shared mutable state, NO synchronization primitive beyond
the join edge — no mutex, channel, or user-facing atomics — in v1.**

#### 1.1 The construct

```
scope {
    spawn check_module(take mod_a, read symtab, write out_a);
    spawn check_module(take mod_b, read symtab, write out_b);
}   // join point: both tasks are terminal (returned or faulted, §3) before control passes
```

`scope` opens a **concurrency region** with one guaranteed property: **no task spawned
inside it outlives the closing brace.** The brace *is* the join barrier. A *block*
rather than a value delivers the invariant structurally: there is no join-handle value
to store, return, or leak, so "a task escapes its scope" is not expressible, not merely
rejected. `spawn CALLEE(ARGS);` starts one task; the callee is a **fn or fn-pointer**,
its body the capture-free `fn + explicit context` idiom (0009 §5) — **no task
closures**, so none of the region-typed-closure machinery 0009 §5.1 refused is dragged
in. State a task needs travels as arguments, each admitted or rejected by §2.

**Why a keyword, not a library fn (held under review).** The library form
`scope(ctx, fn(s: read Scope, ctx){…})` carries a `Scope` handle that must not escape,
needing new machinery or double data-threading. The block form makes escape structurally
impossible and reuses the lexical loan region the checker already understands (0001
§2.3). Concurrency is "dangerous meaning," so it wears words (P13), earning its slot by
the test `for` passed (0009).

#### 1.2 What crosses a spawn boundary — modes reused, gate deferred to §2

`spawn` adds **no new passing mode**. What each mode *does* at the boundary is below;
**whether an argument is well-formed at all is decided by the single gate of §2.** The
old edition's piecemeal gate (a `take`-only portability rule that collided with the
`slice_mut` crossing of §1.4) was review finding 2 and is resolved in §2.1.

| At the spawn site | Mechanism | Effect on the parent |
|-------------------|-----------|----------------------|
| `take arg` | **ownership transferred** in (moved, or copied if `copy`) | parent has moved it out — no alias remains |
| `read arg` | **shared-read loan** on the referent, held the **whole scope** | parent frozen to shared on that place until the brace |
| `write arg` (and a `slice_mut` value passed by value) | **exclusive loan** on the referent, held the **whole scope** | parent frozen out of that place until the brace |
| `out arg` | **rejected** across a spawn (results use `write` slots, §3.1) | — |

The one novelty is the loan's live range: **a spawn-crossing borrow's loan lives for the
whole scope**, not to its last syntactic use — one extension of 0001 §2.3, not a new
analysis. A `slice_mut` value crosses as a **borrow** (an exclusive scope-length loan on
the run it views), never as a `take` of an owned portable — so the old §1.4 phrase "each
half `take`s into a different task" was wrong (finding 2) and is corrected in §1.4/§2.

#### 1.3 No synchronization primitive ships — from the basket and P6

**v1 is pure fork-join data-parallelism: no `Guarded[T]`, no channels, no user-facing
atomics.** The parallel checker splits modules across tasks (each `take`s its partition
or `read`s the immutable symbol graph, and writes its result into a caller-owned slot,
§3.1); aggregation is post-join, single-threaded — no shared mutable state. A work-split
fill partitions a buffer into disjoint chunks each mutated through a disjoint `slice_mut`
(`split_mut`, §1.4). No mutex. Shipping a lock would force ch09 §3.2 (atomic orderings)
now, triggering the O1–O5 obligations fault-window §13.1 records as CONJECTURE,
undischargeable before the atomics surface exists. Fork-join's synchronization edges —
`spawn` (release) and join (acquire), plus the same pair per nested scope — are full
barriers the **native runtime** provides, not a surface the user programs. (Correction:
the join is *not* a task's "sole" sync edge — spawn edges and nested scopes are also sync
ops; §4 accounts for all of them. The no-lock conclusion stands: every such edge is
runtime-provided.) This settles ch09 §4.1–§4.2 (§4.1).

#### 1.4 Disjoint exclusive partitioning — `split_mut`, a priced blessed primitive

```
fn split_mut[region r, T](s: write[r] [T], mid: usize) -> (write[r] [T], write[r] [T])
```

It reborrows one exclusive slice as **two disjoint** exclusive slices `[0, mid)` and
`[mid, len)` (bounds-faulting if `mid > len`). **This disjointness is NOT a loan-scan
fall-out** — the old "XOR already blesses it" was review finding 5. The scan is
**index-insensitive** (0001 §2.2/§10: any `a[i]` covers the *whole* array), so two
`slice_mut` views of one array are, to the scan, **overlapping and in conflict**; it
cannot derive `[0, mid)` ⊥ `[mid, len)`. So `split_mut` is re-characterized honestly:

- **A compiler-blessed primitive** with a **stipulated-disjointness postcondition**: the
  compiler *knows by fiat* the two outputs denote disjoint places and stamps that onto
  their loans, so the XOR scan treats them as non-overlapping — the one place the
  index-insensitive scan is told an answer it cannot compute.
- **An `unsafe` body.** The implementation computes the two sub-slice pointers by raw
  arithmetic (`ptr_offset`) under `unsafe "the halves are disjoint by mid ≤ len"`; the
  stipulated disjointness is discharged there, by the unsafe author.
- **Priced as a new blessed exception** (P6, below): one more compiler-known item whose
  soundness rests on a hand-checked postcondition, on the ledger line with `Box`/`slice`
  — not a free consequence of an existing rule.

Each half then crosses into a task **as an exclusive scope-length loan** (§1.2), never as
a `take`. Being stipulated-disjoint, the two loans do not conflict under XOR, and each is
the unique exclusive accessor of its bytes — siblings mutate disjoint memory, no race
(§2.4).

**Nested `split_mut`** composes: each application re-establishes its bound on its input, and
sub-places of a stipulated-disjoint half remain disjoint from the other half (re-review F4a).

### 2. The unified spawn-crossing gate — and the guarantee it carries

The guarantee P10 demands is the existing loan discipline plus **one gate on every spawn
argument** (finding 1: unified, and gating borrow **referents**, since the falsifying
counterexample crossed through an *ungated* `read`).

#### 2.1 The gate (one rule, two branches, the exemption stated)

> **Every `spawn` argument is gated.** For argument `a`:
> - **Ownership-transfer branch (`take`, explicit or default).** `a` moves/copies an
>   owned value into the task. **Well-formed iff the argument's type is `portable`**
>   (§2.2).
> - **Borrow branch (`read`/`write`, and any `slice`/`slice_mut`/borrow value passed by
>   value).** `a` crosses as a **scope-length loan**. This is the **explicit exemption
>   from the take-gate**: a borrow is *never itself* `portable`, so gating the borrow
>   value would forbid all cross-thread sharing outright. The gate instead falls on the
>   **referent**: **well-formed iff the referent type is `portable`** — no `rawptr` and
>   no borrow reachable from the referent, transitively, **including through `copy`-typed
>   fields.**

The exemption resolves finding 2's three-rule collision by stating it as *the rule*:
sharing crosses as scope-length loans, and their obligation is on the referent; `take`
carries the whole-type obligation because it transfers an owned value whose every
reachable byte becomes the task's.

**Why "through `copy`-typed fields" is load-bearing.** A shared borrow cannot *mutate*
its referent — but if the referent is `copy`, it can be **copied out** from behind the
borrow into an owned value (0001 §1.3–1.4), and that value carries whatever the referent
transitively contains, including a `rawptr`. So "shared = immutable = safe to share" is
**false whenever the referent is a `copy` aggregate hiding a `rawptr`.** The gate closes
this by computing `portable` *transitively through `copy` fields* — the walk does not
stop at a `copy` boundary (§2.3 walks the exact laundering).

#### 2.2 `portable` — structural, transitive, and part of the exported interface

> A type is **`portable`** iff it transitively contains **no `rawptr`** and **no borrow**
> — owned value data only: scalars, `bool`, `unit`, `portable` structs/enums, `[N]T` and
> `Box T` where `T` is `portable`. **The walk descends through every field, including the
> fields of `copy` aggregates** — `copy` is not a stopping condition. `Box T` is
> `portable` when `T` is, because its pointer is *unique-owning* — categorically unlike a
> `rawptr`.

> **Function pointers are a `portable` leaf** (re-review F1): a fn-pointer value carries
> no data pointer and cannot capture (0001 §6.1); any `rawptr` in its *signature* is only
> produced by calling it, and acting on that result is gated by `unsafe` (0001 §4.2). The
> portability walk does **not** descend into signature types — descending would wrongly
> reject safe vtable sharing (`AllocVtable`, `POOL_VTABLE`), which launders no live state.


`portable` is a checker-computed predicate, not a tracked effect (NN#19 intact) and not a
user interface (avoiding 0007 coherence for a structural property). It surfaces as a
generic bound (`fn parmap[T: portable, …]`) and in diagnostics.

**`portable` is part of a type's EXPORTED interface (forced 0007/0008 update).** Like
`copy`, whether a type is `portable` is a structural fact a caller in another module must
see without the definition — a `[T: portable]` boundary bound, and the portability of an
exported/opaque type, must be answerable from the interface. This design **records a
forced update to 0007 (bounds) and 0008 (modules)**: `portable` joins `copy` in the
properties an exported type publishes (the copy capability: 0007 §3.1; the exported interface artifact: 0008 §2 — which the forced 0008 update extends with a structural-properties field for opaque exported types). It reserves no keyword globally (§5).

**No `Sync` analog is needed.** A `read` across a spawn whose referent is `portable` is
safe to hand to any number of tasks: no interior mutability (0001 §4.3) means a shared
borrow cannot mutate, and a `portable` referent means nothing copy-out-able behind it can
launder a `rawptr`. "Shared + `portable` referent = immutable, un-launderable = safe to
share" holds with no exceptions — the share property is universal over portable
referents; only the crossing property (`portable`) needs a marker.

#### 2.3 The race non-example — the `Alloc` laundering, worked and now rejected

```
// alloc : Alloc  — a copy struct { ctx: rawptr u8, vt: rawptr AllocVtable } (0001 §6.1)
scope { spawn worker(read alloc); }   // crossing a shared borrow of a copy-with-rawptr handle
```

**Old rules (admitted it — the bug).** `read alloc` is a shared loan; the old §1.2 gated
only `take`, so a `read` crossed ungated. Inside the task:

```
fn worker(a: read Alloc) alloc -> unit {
    let mine = deref a;             // Alloc is copy -> COPIES the handle OUT of the shared borrow
    let b = box(read mine, node);   // owned Alloc, live rawptr into the SHARED free list -> task allocates
}
```

Parent still owns `alloc`; task owns `mine`; both splice the same free list — **a data
race on the allocator's free list, zero `unsafe`.** The channel is "copy-out from behind
a shared borrow" over a `copy` aggregate hiding a `rawptr`.

**New gate (rejects it at the spawn site).** `read alloc` takes the **borrow branch**;
the referent type is `Alloc`. `portable(Alloc)` descends into `Alloc`'s fields **even
though `Alloc` is `copy`**, finds `ctx: rawptr u8`, and returns **not `portable`** → the
`spawn` argument is **ill-formed and rejected**, diagnostic naming the `rawptr` field
(P4). The task body is never reached; `deref a`'s copy-out never happens. **The
laundering channel does not open.** (This is why O2 is provable only after this fix —
§4.2: with the channel closed there is no safe relaxed/racy cross-thread edge to leak
through.)

#### 2.4 The legal parallel-fill — worked and accepted

```
// buf : [N]u8   (u8 is portable; [N]u8 is portable)
let whole = write buf;                 // exclusive borrow -> slice_mut u8
let (lo, hi) = split_mut(whole, N/2);  // two STIPULATED-DISJOINT slice_mut u8 (§1.4)
scope {
    spawn fill(lo);   // slice_mut value crosses as an exclusive scope-length loan (borrow branch)
    spawn fill(hi);   // ditto, on the disjoint half
}   // join: both halves filled; buf usable again after the brace
```

Each of `lo`, `hi` takes the **borrow branch**; referent is a run of `u8`, and
`portable([u8])` walks `u8` (scalar, no `rawptr`) → **`portable`.** Both well-formed. The
loans are on the **stipulated-disjoint** places `split_mut` blessed, so the scan — *told*
they are disjoint — sees no overlap: two non-overlapping exclusive loans, **legal** under
XOR; each task is the unique accessor of its half → **no race, accepted.** This is
exactly the `slice_mut`-of-`portable`-elements crossing finding 1 required legalized. By
contrast `slice_mut Alloc` is **rejected** (referent `[Alloc]` hides a `rawptr`) — even a
disjoint fill of an allocator-handle buffer does not cross, correctly.

#### 2.5 Cross-thread rejection, and the `rawptr`/allocator boundary

With every crossing argument gated, the closing-brace horizon has every scope-length loan
live, and XOR (0001 §2.2) decides every cross-thread aliasing question with **no new
aliasing rule**: two `read`s → legal; two overlapping `write`s → rejected (disjoint
`split_mut` halves → legal); a `write` while the parent touches the place → rejected; a
`read` racing a sibling `write` → rejected. Every racing pair maps to a shared+exclusive
or exclusive+exclusive overlap the scan rejects — **and the gate forecloses the one
channel that bypassed the scan** (a `rawptr` laundered out of a `copy` referent). Races
are unrepresentable in safe code because the loan scan plus the referent gate make the
racing program not type-check.

The residual the gate cannot structurally decide is a genuine `rawptr` crossing:

- The **`Alloc` handle is not `portable`** (§2.3) — *correct*: a general-purpose
  allocator is not thread-safe, and the `rawptr` exclusion is what stops safe code from
  sharing one by accident.
- **The clean idiom is a per-task arena over `portable` owned memory** (held under
  review). A `Box [N]u8` *is* `portable`; a task that `take`s its own backing buffer and
  bump-allocates within it touches only task-local owned bytes — no shared allocator, no
  race. This is how parallel compilers allocate; v1 blesses it.
- **The `alloc` effect crosses a spawn.** A spawned `alloc`-marked callee makes the
  **enclosing function `alloc`-marked**, by the same one-way partition as any call (0001
  §3.2/§6.3): `spawn box_node(...)` propagates `alloc` to the function containing the
  `scope`. The spawn boundary is not an effect-laundering boundary.
- **Sharing a thread-safe allocator is an `unsafe`-declared decision.** To move a
  `rawptr`-bearing handle into a task, the author writes the argument under
  `unsafe "this allocator is thread-safe"` — the trust becomes greppable/enumerable
  (P1/P17), the checker's silence honest. It is fault-window §13.4(iv) met where it
  arises, and the sole residual behind O5 (§4.2).

Thread-safety is thus a **structural property** (`portable`, gating both branches) plus a
**declared trust boundary** (`unsafe`) — never a new tracked effect (P2/NN#19).

### 3. Faults across tasks, and how results come back

#### 3.1 Results via write-borrowed slots — THE mechanism, honest containment

The old design *relied on* tasks "returning owned results" but supplied **no mechanism**
across the handle-free join (a review finding). This edition states it:

> **A task returns results by writing them, in place, through a `write`-borrowed slot the
> parent owns.** The parent allocates one slot per task and passes `write slot` (an
> exclusive scope-length loan, referent `portable`); the task writes its output through
> the borrow; **after the brace the parent reads the slots.** No returned handle — the
> result travels *into caller-owned storage*, not out through a value.

Slot disjointness is decided as any exclusive loans (0001 §2.2): **distinct
locals/fields are disjoint places the scan tracks** (`write out_a`, `write out_b` on two
named locals do not conflict, §1.1), so a fixed fan-out uses distinct slots directly; a
fan-out into **one buffer** must carve disjoint sub-slices with `split_mut` (§1.4),
because index-insensitivity forbids per-element `results[i]` loans (they all alias). The
result path and the parallel-fill path are the same primitive.

**Containment restated honestly, and loudly (finding 7).** The old §3.1 claimed "there is
no partial result to leak, because the result crosses only *at* the join." With
write-slots that is **false**:

> **Pre-fault in-place writes to a write-borrowed slot ARE visible to the parent
> post-join.** A task's slot-writes are ordinary in-model memory effects happening
> *during* the window, *before* the fault. **How many of a faulting (or cancelled) task's
> slot-writes retired at delivery is a window-interior, build-mode-dependent fact**
> (fault-window §4.3 / R2): the window has no *observable* event, but it does hold these
> memory writes, and truncation bounds observables and cross-thread happens-before
> delivery, **not** the retirement of window-interior memory effects. So a faulting task
> may leave its slot **partly or wholly written**, and after the brace the parent sees
> whatever retired.

This is **not** a containment defect: a slot write is not an observable and crosses to the
parent only *at* the join edge (a happens-before edge Thm 1 bounds, §4.2) — it is exactly
the window-interior nondeterminism P5 licenses, surfaced because results live in
shared-with-the-parent storage. The obligation is to **name** it: **on any non-successful
task, its slot holds an indeterminate partial value**; the parent must not read a
non-successful task's slot as valid. Under `abort` nothing observes it; under `halt-and-
log`/user handler the parent resumes and MUST gate each slot on that task's outcome.

A faulting task's fault is delivered **at or before its join edge** (and, per §4, before
**every** spawn/join sync edge). The join receives the fault in place of a valid result;
the *identity* of the delivered fault is deterministic (§3.2), the *side-effect extent*
in the slots is not (this paragraph).

#### 3.2 Deterministic fault identity — spawn-order-first, scoped to identity only

Program-order-first (fault-window §6.2 Option A) pins identity per thread; across threads
there is no total program order, so this design supplies the tie-break: **the delivered
fault is that of the spawn-order-least task that faulted.** Spawn order is a deterministic
total order (textual `spawn` order), and the join-all invariant makes selection decidable
— by the brace every outcome is known. Fault identity is therefore **deterministic across
schedules and build modes** though scheduling is not (held under review), preserving P5's
tested-artifact guarantee and P4's diagnostic determinism.

**Scope of the determinism (finding 7).** Spawn-order-first pins **which fault is
delivered — its identity `(k, s)` — and *only* that.** It does **not** pin **how much of
any task's side effects retired**: a faulting or cancelled task's slot-write extent
(§3.1), and which window-interior writes of other tasks retired before the selected fault
truncated the scope, are build-mode- and schedule-dependent. Determinism of fault
*identity* is not determinism of side-effect *extent*; the design claims the former and
disclaims the latter. (The advisory value-context `c` stays best-effort, §6.4.)

#### 3.3 Cancellation — cooperative, at explicit points, no async (held under review)

Cancellation is **cooperative, never preemptive**: the scope *requests* a sibling to stop
but never *forces* it — forcibly killing a thread mid-mutation corrupts state, the unsound
machinery a systems language must refuse. No async, no unwinding runtime (P7):
cancellation is observed by **ordinary control flow at explicit cancellation points.**

```
fn worker(take chunk, tok: read Cancel, out slot: write Partial) -> unit {
    let mut acc = seed();
    for item in chunk {
        if cancelled(tok) { deref out slot = acc; return; }  // cancellation point, visible in source
        acc = step(acc, item);
    }
    deref out slot = acc;
}
```

`cancelled(read Cancel) -> bool` is a plain query. Between two checks a task runs to
completion; a task taking no `Cancel` token is **non-cancellable** and the scope waits for
it. A nested `scope`'s join is *also* an implicit cancellation point. The idiom — a
`cancelled` check in the outer loop — is the whole answer; there is no free lunch.

#### 3.4 Scope-body faults — cancel-then-join, and the unbounded-but-legal delay

Because *no task outlives the scope* (§1.1), the parent cannot pass the brace until every
sibling is terminal. On a task fault the scope performs **cancel-then-join**: (1) record
the fault; (2) request cancellation of all siblings; (3) join all siblings — each
completes or reaches a cancellation point and unwinds cooperatively, never a forced kill;
(4) deliver the selected fault (§3.2) *at the brace*, where the root policy fires in the
parent's context. Under `abort` an implementation may abort at step 1; the *semantics* is
"delivered no later than the join edge."

**The unbounded-but-legal delivery delay, named loudly.** Cancellation is cooperative, so
**a compute-bound sibling with no cancellation points delays fault delivery to the brace
by however long it runs.** Task A faults at wall time `t`; the scope cannot deliver until
every sibling is terminal (join-all); a sibling B with no `cancelled` check runs to its
natural end, and A's fault waits for the brace — a delay **unbounded in wall time**
(bounded only by B's termination). **It is P5-legal, and exactly why: the join edge is the
fault thread's bounding synchronization operation, and P5 requires delivery no later than
the bounding sync op retires — the join has not retired while B runs, so delivering A's
fault at the brace, whenever the brace is reached, is within the P5 bound.** The cost is a
latency the author owns (a `cancelled` check in B's loop is the fix), not a soundness
defect — named here, not hidden.

**Arena-backed `Box` across a join** (re-review F4b): a `Box` serviced by a task-local arena is
`portable` and crosses via a write-slot, but freeing it after the arena dies violates the
constructor's 0001 §6.1 liveness obligation — an `unsafe`-declared trust, not a safe-code hole.
The clean idiom returns owned value data, never arena-backed `Box`es.

### 4. The formal claim — per-edge truncation, Thm 1 applied per edge

The old §4 rested on "the join is the only synchronizes-with edge a safe task has" —
review-proven **false**: spawn edges and nested scopes are also sync ops, so a task has
**several** sync edges. The sole-edge argument is **deleted**; per-edge truncation reaches
the same O1–O4 conclusion by a sound route.

#### 4.1 ch09's adoption fragment — triggered, and which fragment (held under review)

- **SC-for-DRF baseline (ch09 §3.1)** — adopted. Safe code is DRF (§2), so every safe
  program enjoys sequential consistency; no races for a weaker model to describe.
- **`spawn` and join as the cross-thread ordering** — the native runtime's thread
  create/join are adopted as a **release (at `spawn`) / acquire (at join)** pair (ch09
  §3.1), *plus the same pair per nested scope*. (Correction: more than one per task.)
- **Atomics (ch09 §3.2) — NOT adopted this edition.** No user-facing atomics (§1.3), so
  orderings/fences stay deferred; the DRF story is purely structural.

The `Alloc`/`rawptr` boundary (§2.5) is the ch09 §3.3 unsafe-interactions surface, handled
by assigning ordering to the unsafe author (fault-window §13.4 iv), not a v1 atomics
surface.

#### 4.2 Per-edge truncation, and which of O1–O5 this makes provable

**The pin.** The fault-window bound `e⁺` (fault-window §4.1, §12.1) is, in the fork-join
fragment, the **po-least element of a task that is either an observable event or a
synchronization operation** — and a safe task's sync operations are exactly its **`spawn`
edges, its join edge, and the `spawn`/join edges of every nested `scope` it opens.** These
are **discrete and finitely many** per task. The compiler **pins fault delivery before
EVERY one of them retires**: for each edge `s`, nothing at or after `s` in that task's
program order executes once a fault preceding `s` is enabled.

**The proof shape: Thm 1 applied per edge.** Each sync edge `s` is an instance of the
single-thread window bound with `e⁺ := s`. Fault-window **Theorem 1 (Containment)** then
applies *at that edge*: for a fault `f` with `f <po s`, (a) no observable data/control-
dependent on `f` appears, and (b) no observable `≥po s` retires — so nothing dependent on
`f` crosses `s` to another thread. Since the sync edges are the *only* synchronizes-with
edges (§4.1) and each is individually a Thm 1 instance, **cross-thread containment is the
finite conjunction of per-edge Thm 1 applications** — no single-edge argument and no new
theorem needed. The general §12.2 conjecture over an arbitrary atomics surface is not
solved; the fork-join fragment is discharged because its sync edges are enumerable and
each is a Thm 1 instance.

Under the per-edge pin **and after §2.1 closes the laundering channel** (fault-window
§12.3):

- **O1** (`W(f)` vs synchronizes-with; `e⁺` well-defined): **provable in-fragment** —
  `e⁺` is the po-least observable-or-sync-edge, and the sync edges are the discrete,
  finite spawn/join/nested-scope boundaries.
- **O2** (cross-thread containment via hb, no leak via relaxed/racy visibility):
  **provable in-fragment — but *only because §2.1 closed the laundering channel*.** After
  the gate every safe cross-thread channel is an hb edge (the release/acquire pair)
  bounded by per-edge Thm 1; there is **no safe relaxed-atomic or racy channel** (no
  atomics §1.3; no `rawptr` laundered out of a `copy` referent §2.3), so the relaxed/racy
  half is **vacuous in safe code.** Had the channel stayed open, O2 would be *false* —
  the precise sense of "O2 provable only after fix 1."
- **O3** (truncation under interleaving): **provable** — per-edge, no observable is
  ordered happens-after a task's bounding sync edge once truncation fires; siblings share
  only disjoint or read-only `portable`-referent data (§2).
- **O4** (SC-for-DRF preservation on fault-free runs): **provable** (held under review) —
  fault-free fork-join runs are DRF and enjoy the adopted SC (Thm 2 lifted); the window
  perturbs only faulting runs.
- **O5** (composition with the aliasing model across the safe/unsafe boundary): **stays
  open** exactly where a `rawptr` crosses under §2.5's `unsafe` — containment is claimed
  only over hb, not over racy/unsafe channels (fault-window §13.4 iv). The honest
  residual, now narrowed to the single explicitly `unsafe` crossing.

So per-edge truncation moves **O1–O4 from CONJECTURE to provable-in-fragment** (O2
conditioned on §2.1) and leaves **O5 open** at the declared unsafe boundary. The general
conjecture reopens only when atomics are added by amendment.

#### 4.3 What stays open (P10, said plainly, held under review)

Ergonomic high-concurrency I/O stays **open research this document does not resolve**: the
v1 answer for an io_uring-saturating storage engine is threads plus explicit completion-
driven state machines, capable but manual. A partition-free model — if ever proven
compatible with P9 and free of async coloring — is an amendment. Atomics, a
mutex/`Guarded[T]`, and channels are deferred with it; each needs the ch09 §3.2 adoption
and the O1–O5 general discharge first.

### 5. NN#13 syntax walk (held under review)

- **`scope`** — a **contextual keyword** in statement-leading position; elsewhere an
  ordinary identifier. `{` after `scope` is unambiguously a block.
- **`spawn`** — a **contextual keyword** statement-leading inside a `scope` body;
  `spawn CALLEE(ARGS);` parses as the ordinary call grammar (0006) prefixed by `spawn`. A
  `spawn` outside any `scope` is a checker error, not a parse error — the grammar stays
  context-free (NN#13).
- **`portable`** — not a keyword: a **built-in bound name** resolved like `copy`,
  appearing in bracket bound-lists (`[T: portable]`) and diagnostics; reserves nothing
  globally. It is now also an exported-interface property (§2.2), a checker/interface fact,
  not a grammar surface.
- **`split_mut` / `cancelled` / `Cancel`** — a blessed primitive, one fn, one
  compiler-known type; no grammar surface, migrated like any name.

No hard reserved word is added; every new keyword is contextual, consistent with the
`for`/`type` discipline (0009 §4.4). The corelib seed uses neither `scope` nor `spawn`;
the migrator flags any such use (P15).

### 6. Prototype stage plan and the nondeterminism gate

The tree-walking interpreter (0001 §7) and the MIR interpreter (0010 §5) are
single-threaded. Staging delivers the *compile-time* guarantee first and never forces
nondeterminism into the differential oracle.

- **Stage 1 — checker rules only (the bulk). [IMPLEMENTED 2026-07-09 in the prototype: parse + full gate + scope-length loans + `split_mut` + sequential-oracle execution on both engines; see `tests/concurrency.rs`.]** `scope`/`spawn` parsing; the scope-length
  loan extension (§1.2); `portable` as a structural predicate beside `copy`, **computed
  transitively through `copy` fields** and published in interfaces (§2.2); **the unified
  gate** (§2.1); the XOR fall-out over gated arguments (§2.5); `split_mut` well-formedness
  **as a blessed primitive with a stipulated-disjoint postcondition** (§1.4); the
  write-slot result path (§3.1); the `unsafe` `rawptr`-crossing trust point (§2.5).
  *Gate:* a negative-fixture suite rejecting **every** DRF-violating program with correct
  diagnostic and provenance (P4) — two writers, writer+reader, parent-touch-under-live-
  exclusive, non-`portable` `take`, **and the `Alloc`-laundering `read` of a
  copy-with-`rawptr` referent (§2.3)** — plus a positive suite (fork-join-of-owned,
  shared-read of a `portable` referent, `split_mut` partition, write-slot collection)
  accepted. Pure loan/property scan; no execution nondeterminism. **This is where
  race-freedom is established** (see the honesty note below).

- **Stage 2 — deterministic sequential oracle + REAL native parallelism. [IMPLEMENTED 2026-07-09 in the prototype: the tree-walker and MIR interp stay the sequential oracle; the native Cranelift JIT engine now runs `scope`/`spawn` on REAL OS threads (`rt_spawn`/`rt_join` over `std::thread`), with per-task thread-local trace buffers merged at the join in spawn order (deterministic θ), an atomic stack-bump + thread-local fault landing (runtime-internal sync below the language, §1.3 note), and spawn-order-first fault delivery collected at the brace. `split_mut` (§1.4) now has a REAL runtime on every engine — the tree-walker writes two `slice_mut` headers over `[0,mid)`/`[mid,len)` (bounds-faulting when `mid > len`), and the MIR/native lowering expresses it as inline slice-header ops (`slice_of_mut` + two bounds-checked `subslice`s), so the disjoint-fill flagship EXECUTES end-to-end (no hidden hook); the bounds-fault identity `(bounds, call-span)` is engine-consistent. The AOT C runtime (`src/backend/aot_runtime.c`) implements `rt_scope_begin`/`rt_spawn`/`rt_scope_end` over raw pthreads (per-task outcome structs, spawn-order join+merge for deterministic θ, po-least fault via the fault-exit path), so AOT-compiled concurrent programs run as real processes at trace/exit/fault parity with the JIT. The nondeterminism gate runs every native-supported fixture 50–200× (`tests/concurrency_native.rs`). See `tests/concurrency.rs` (checker/oracle/split_mut execution), `tests/concurrency_native.rs` (threaded gate), and `tests/aot.rs` (compiled-process parity).]** Run each spawned task to completion in
  spawn order on the single-threaded engine — a *sequential schedule*, which is **valid**
  because safe fork-join Candor is DRF (by the Stage-1 checker) and ships no user-
  observable nondeterministic sync (§1.3): by SC-for-DRF every schedule yields the same
  observable trace up to the sync edges. The 0010 §4 gate is unchanged: `(k, s, θ)`-trace
  equality across engines, fault by spawn-order-first (§3.2, decidable since sequential
  execution has run every task before the brace). Result slots are read post-join as on
  the threaded backend.

- **The nondeterminism gate (for the threaded native backend).** (1) **Interaction points
  are discrete and few** — safe tasks interact only at `scope`/`spawn`/join/`cancelled`,
  so the interleaving space is over these, not instructions. (2) **Per-task projection:**
  `θ` carries a per-event task tag; two runs are trace-equivalent iff each `θ|task_i` is
  equal and sync-edge-ordered events respect that order (disjoint-resource interleaving
  not asserted). For tasks emitting no observable until join and returning via write-slots
  (the parallel-checker shape), projections plus sync-edge order fully determine `θ`,
  collapsing to exact `(k, s, θ)` equality. (3) **`stdout` — a P5 declared-nondeterminism
  extension.** A writable MMIO/device handle is exclusive, so the gate (§2.1) + XOR forbid
  two safe tasks holding one; the only shared safe sink is process `stdout`. **Multi-writer
  `stdout` order is hereby recorded as an explicit extension of P5's declared-
  nondeterminism set — a P5-touching move, flagged as such:** the gate tests the
  **trace-*set*** on `stdout` (multiset equal, global order not) and the exact projection
  everywhere else, enlarging P5's declared set by exactly this one member. (4)
  **Bounded-schedule enumeration:** a preemption-bounded / partial-order (DPOR-style)
  enumerator branching only at `scope`/`spawn`/join/`cancelled` is tractable, asserting for
  every schedule the projection invariant, the `stdout` trace-set rule, and the
  **spawn-order-first fault identity** (§3.2) identical across all schedules.

  **The honesty this gate owes (finding 6).** The differential gate **presupposes DRF; it
  cannot establish it.** Race-freedom is carried **entirely by the Stage-1 checker** (§2);
  Stage 2 tests fault-identity determinism and observable-projection equivalence *given*
  DRF. Crucially, **the DPOR enumerator's state space excludes checker-hole races by
  construction**: it branches only at `scope`/`spawn`/join/`cancelled` and models *no*
  instruction-level interleaving of shared-memory accesses — because the checker guarantees
  there are none. So a race that slipped a checker hole is **not even representable** in
  the enumerated state space, and the gate would never surface it. The gate checks the
  fault/observable story atop a DRF assumption, not a second, independent proof of DRF.

  *Gate to pass Stage 2 → real backend:* native-threaded execution satisfies the projection
  + `stdout`-trace-set + fault-identity invariants on the basket's parallel ports under
  bounded-schedule enumeration; a schedule violating any invariant has, by the spec's
  definition (P18), changed behaviour, and the job fails.

## Rejected alternatives

- **`async`/`await` or any effect-system spelling — rejected (NN#9, P10).** The
  bidirectional, API-duplicating partition is the schism P10 refuses; an effect re-encoding
  reintroduces it under a new name.
- **Detached threads / returnable `JoinHandle` / unstructured `spawn` returning a
  future — rejected.** A handle outliving its scope breaks the §1.1 invariant and re-admits
  use-after-free of borrowed data, and would carry the task's borrows (0001 §3.4 / 0007
  §3.5); the join must be syntactic and unavoidable.
- **Gating only `take` on portability (the old design) — rejected as unsound.** It left
  `read`/`write` referents ungated, laundering a `rawptr` out of a `copy` referent with zero
  `unsafe` (§2.3); the gate must fall on borrow referents too, transitively through `copy`
  fields (§2.1).
- **`split_mut` disjointness as a loan-scan fall-out — rejected as false.** The scan is
  index-insensitive (0001 §2.2/§10); disjointness must be *stipulated* by a blessed primitive
  over a compiler-internal construction (no user-spellable slice-from-raw-parts op exists in 0001 §4.2/§5.2 — the intrinsic is exactly the point), and *priced* (§1.4).
- **A `Send` effect / a `Sync`-analog — rejected (NN#19, §2.2).** Thread-crossing is the
  structural `portable` plus a declared `unsafe` boundary, never an effect; no interior
  mutability (0001 §4.3) plus the referent gate leaves nothing for a Sync predicate to carve.
- **A mutex/`Guarded[T]` or channels in v1 — rejected (§1.3, P6).** Both need the ch09 §3.2
  atomics adoption and thereby the open O1–O5 discharge as a precondition.
- **`out` across a spawn — rejected.** No defined join-time single-writer story; results use
  `write`-borrowed slots (§3.1), ordinary exclusive loans the gate and XOR govern.
- **Capturing closures as task bodies — rejected (0009 §5).** Reopen only via 0009's
  OBL-GENERICS-CLOSURE gate. **Preemptive cancellation — rejected (§3.3):** killing a thread
  mid-mutation corrupts state. **Library-fn `scope(fn body)` — rejected (§1.1):** the
  handle-escape problem needs machinery the block form makes unnecessary.

## Consequences and costs

*Debts, not absolutions.*

- **Ergonomic high-concurrency I/O is ceded in v1 (P10's Refusal, in force):** this design is
  the fork-join floor beneath the storage-engine answer, not a narrowing of the Refusal.
- **No shared-mutable concurrency without `unsafe`** (§2.5). If a future basket program makes
  it ambient, that is the recorded P10-vs-basket revisit toward a `Guarded[T]`.
- **Shared allocation across tasks is an `unsafe`-declared decision (§2.5);** the clean path
  is per-task arenas over `portable` memory. The `alloc` effect crosses the spawn (a spawned
  `alloc`-callee makes the enclosing fn `alloc`).
- **Result slots of non-successful tasks hold indeterminate partial values (§3.1)** — a
  resuming parent (non-`abort`) must gate each slot on its task's outcome. Fault *identity* is
  deterministic; side-effect *extent* is not (§3.2).
- **A compute-bound task with no cancellation points delays scope exit and fault delivery
  (§3.4)** — unbounded in wall time, but P5-legal (the join is the bounding sync op).
- **The nondeterminism gate is weaker on shared sinks and does not itself prove DRF (§6):**
  `stdout` is asserted as a trace-*set* (an explicit P5 declared-nondeterminism member);
  race-freedom rests entirely on the Stage-1 checker, the DPOR state space excluding
  checker-hole races by construction.
- **The P6 budget line — updated.** *Added:* two contextual keywords (`scope`, `spawn`); one
  structural predicate (`portable`), now also an exported-interface property (forced 0007/0008
  update, §2.2); **one compiler-blessed primitive `split_mut` with a stipulated-disjoint
  postcondition over an `unsafe` body, priced as a new blessed exception — NOT a free
  loan-scan fall-out (§1.4, the change from the rejected edition)**; the write-slot result
  convention (§3.1, no new surface); a minimal cancellation surface (`Cancel`/`cancelled`).
  *Not added:* mutex, channels, atomics, a Sync predicate, a Send effect, join-handle values,
  an `out`-across-spawn story. The DRF check, passing modes, and fault mechanism are reused,
  not grown.
- **O5 stays open (§4.2)** — assigned to the unsafe author (fault-window §13.4 iv), now
  narrowed to the single explicitly `unsafe` `rawptr` crossing (the safe laundering that
  widened it is closed by §2.1).

## Reclassification record

None. The mandate is P10 (Priority 1 soundness — DRF non-negotiable); the construct-shape and
no-lock calls are argued from P6/P9 and the basket; the syntax is ordinary P13. The rework
corrects a soundness defect (the laundering channel) and two honesty defects (missing result
mechanism, false sole-sw-edge proof), none a reclassification.
