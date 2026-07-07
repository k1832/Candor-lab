# Intrusive-List Priority Scheduler — idiomatic-Rust baseline

Baseline (Bet 5, criterion §2.5) for `docs/basket/spec-scheduler.md`: a strict
4-level priority scheduler (`NPRIO = 4`, level 0 highest .. 3 lowest, FIFO within
a level) built on **intrusive doubly-linked lists** — the list linkage is
embedded in the scheduled `Task`, and the scheduler never owns the tasks.

## Provenance (vendored — adjudication ruling of 2026-07-07)

Per the adjudication ruling of **2026-07-07** (measured-artifact
self-containment): the spec-mandated core mechanics must live **in-source**, so
the intrusive-list machinery is **vendored** into `src/vendored_intrusive/`
rather than pulled from a cargo dependency. The comparison target (a port in a
language with no crate ecosystem) necessarily carries this machinery in-source;
leaving it in a dependency would exclude exactly the unsafe-dense code the
measurement counts. The cargo dependency has been removed (`Cargo.toml` and
`Cargo.lock` no longer reference `intrusive-collections` or its `memoffset`
dependency).

- **Source crate:** [`intrusive-collections`] by Amanieu d'Antras and Amari
  Robinson, version `0.10.2` (crates.io checksum
  `4b719c59241cfaac1042a6d26787e28ed7ee4a4e21a5a907786f54222d1b0062`).
- **License:** MIT OR Apache-2.0 (dual). The upstream per-file copyright headers
  (`Copyright 2016 Amanieu d'Antras`, `Copyright 2020 Amari Robinson`) are
  preserved on each vendored file, with full provenance in
  `src/vendored_intrusive/mod.rs`. This baseline is itself `MIT OR Apache-2.0`.
- **Toolchain:** built and tested with `rustc 1.93.1` (stable).

### Vendored vs. written

- **Vendored (`src/vendored_intrusive/`, ~990 lines across five submodules
  mirroring upstream):** `LinkedList` and its `Cursor`/`CursorMut` (with the
  O(1) `cursor_mut_from_ptr(...).remove()`), the `LinkedListLink` embedded link,
  the `Adapter`/`LinkOps`/`PointerOps` machinery plus the `intrusive_adapter!`
  and `container_of!` macros, `UnsafeRef`, and the `DefaultPointerOps`
  specialization for `UnsafeRef`. The retained code is upstream's, unchanged
  except for **dead-code removal** and the minimal edits compilation requires
  (module-path rewrites `crate::` → `super::`/`$crate::vendored_intrusive::`,
  and `offset_of!` sourced from `core::mem` instead of upstream's `memoffset`
  re-export — stable since Rust 1.77). Its idiomaticity is upstream's.
- **Dead code trimmed (per the ruling):** the red-black tree, the singly- and
  xor-linked lists, the atomic link variant (`AtomicLink`/`AtomicLinkOps`),
  `CursorOwning`, `IntoIter`, `KeyAdapter`, the `&T`/`Pin`/`Box`/`Rc`/`Arc`
  pointer-op specializations, `clone_pointer_from_raw`, and every unused
  cursor/list method (`push_front`, `pop_back`, split/splice/replace/take,
  `back*`, etc.).
- **Written (this repo's own code, `src/lib.rs`, against the spec):** the four
  run-queues, strict-priority + FIFO ordering, the single RUNNING slot, and the
  full operation set (`admit`/`pick_next`/`yield`/`block`/`wake`/`set_priority`/
  `exit`, including the on-BLOCKED reschedule rule of §2.7).
- **Added lint suppressions (vendoring-required, not memory-model relaxations;
  the ruling forbids restyling the vendored code):** module-level allows in
  `src/vendored_intrusive/mod.rs` — `clippy::declare_interior_mutable_const`
  (upstream's own, for the generated `const NEW`), `clippy::missing_safety_doc`,
  `clippy::manual_dangling_ptr`, `clippy::wrong_self_convention`, and the
  edition-bridge `unsafe_op_in_unsafe_fn` (upstream is edition 2018, where an
  `unsafe fn` body is implicitly an unsafe context; this baseline is edition
  2024). The `intrusive_adapter!`-generated impl carries the same
  `unsafe_op_in_unsafe_fn` allow since it expands outside the vendored module.
  With these, `cargo clippy --all-targets -- -D warnings` is clean.

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

- `src/vendored_intrusive/` — vendored `intrusive-collections` subset
  (`adapter`, `linked_list`, `link_ops`, `pointer_ops`, `unsafe_ref`); see
  `mod.rs` for provenance/license and the vendoring-required suppressions.
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
