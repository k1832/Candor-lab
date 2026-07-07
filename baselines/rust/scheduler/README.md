# Intrusive-List Priority Scheduler — idiomatic-Rust baseline

Baseline (Bet 5, criterion §2.5) for `docs/basket/spec-scheduler.md`: a strict
4-level priority scheduler (`NPRIO = 4`, level 0 highest .. 3 lowest, FIFO within
a level) built on **intrusive doubly-linked lists** — the list linkage is
embedded in the scheduled `Task`, and the scheduler never owns the tasks.

## Provenance

- **Intrusive-list machinery:** the [`intrusive-collections`] crate by Amanieu
  d'Antras, **used as a Cargo dependency** (not vendored).
  - **Exact version:** `intrusive-collections v0.10.2`
    (crates.io checksum `4b719c59241cfaac1042a6d26787e28ed7ee4a4e21a5a907786f54222d1b0062`,
    pinned in `Cargo.lock`).
  - **Used, not derived:** the crate's `LinkedList`, `LinkedListLink`,
    `UnsafeRef`, and `intrusive_adapter!` provide the intrusive doubly-linked
    list, its embedded link field, the non-owning handle type, and the
    container-of adapter. The O(1) mid-removal is the crate's
    `cursor_mut_from_ptr(...).remove()`. No intrusive-list code is copied or
    re-derived; it is called through the public API.
- **Scheduler policy (this repo's own code, written against the spec):** the
  four run-queues, strict-priority + FIFO ordering, the single RUNNING slot, and
  the full operation set (`admit`/`pick_next`/`yield`/`block`/`wake`/
  `set_priority`/`exit`, including the on-BLOCKED reschedule rule of §2.7).
- **Toolchain:** built and tested with `rustc 1.93.1` (stable).

[`intrusive-collections`]: https://crates.io/crates/intrusive-collections

## Why this is genuinely intrusive (not a value-friendly dodge)

- The link field lives **inside** `Task` (`link: LinkedListLink`). The queues are
  `LinkedList<TaskAdapter>` holding **non-owning `UnsafeRef<Task>` handles** into
  caller storage — not a `Vec<Task>`, not a node-owning list, not array indices.
- Tasks live in **caller-owned storage** (`Box<Task>`, stable heap address); the
  scheduler threads them onto queues through their embedded links and owns none
  of them.
- Removal of a middle element is **O(1) via the handle**
  (`cursor_mut_from_ptr` + `remove`, no scan); insertion is `push_back`, O(1).

## Use of `unsafe`

`unsafe` appears only where the intrusive technique genuinely requires it, not
minimized or maximized:

- `UnsafeRef::from_raw(task)` — build a non-owning handle from a caller-owned
  `&Task` for insertion into a queue.
- `cursor_mut_from_ptr(task)` — position a cursor at a task's embedded link node
  for O(1) middle removal.

Each site has a `// SAFETY:` note in `src/lib.rs`. The state machine guarantees a
task is linked into exactly the queue named by its `prio` field when READY, which
is the invariant these two operations rely on.

## Spec ambiguities (flagged, resolved consistently, not silently)

Two points are under-specified by §2 but load-bearing for the T19 stress model.
Both are resolved the same way in the scheduler and in the test's shadow model,
and both are noted here rather than silently decided:

1. **`admit` on a BLOCKED task.** §2.2 lists `E_ALREADY_QUEUED` only for a task
   "already in any queue or RUNNING", but §4.3 says the stress `admit` is skipped
   "if id already present", and §4.3's `exit` treats a BLOCKED task as present.
   For a deterministic shadow model, `admit` must be a no-op/error when the task
   is present in **any** active state. Resolution: `admit` is valid only from
   `New`/`Exited`; a BLOCKED (or READY/RUNNING) task returns `E_ALREADY_QUEUED`.
2. **`pick_next` while a task is already RUNNING.** §2.3 returns `NONE` only when
   all run-queues are empty, but §5.1 excludes preemption, so silently picking a
   second task would drop the current one (violating §3.7). Resolution: with a
   task already RUNNING, `pick_next` is a no-op and returns `None`; the caller
   must yield/block/exit first. (The T1–T20 vectors always deschedule between
   picks, so this only affects T19.)

## Layout

- `src/lib.rs` — `Task`, `Scheduler`, `State`, `SchedError`.
- `tests/vectors.rs` — every frozen vector T1..T20, named by ID, plus the T19
  xorshift64 stress (seed `0x9E3779B97F4A7C15`, 20 000 steps) with a
  language-neutral shadow model and per-step `queue_dump` + list-integrity +
  single-membership assertions.

## How to run

```sh
cargo test        # all vectors T1..T20 (T19 = shadow-model stress)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```
