# Rust baseline: general-purpose allocator

Idiomatic-Rust baseline for the Candor Bet 5 basket, implementing the frozen
functional specification `docs/basket/spec-allocator.md`. This is the comparison
baseline (criterion §2.5); it was written blind to the Candor design documents.

## Provenance (independently sourced, adapted)

- **Source crate:** [`linked_list_allocator`](https://crates.io/crates/linked_list_allocator)
  by Philipp Oppermann, Apache-2.0/MIT.
- **Exact version derived from / depended on:** `0.10.6` (crates.io; pinned as
  `=0.10.6` in `Cargo.toml`, recorded in `Cargo.lock`).
- **Source studied at:** git commit `a1d90498ff82872c5eb89edef77a08b25b4f14c4`
  (`https://github.com/rust-osdev/linked-list-allocator`, `Cargo.toml` version
  `0.10.6`), whose `src/hole.rs` implements the free-list engine used here.

The crate provides the load-bearing part of the spec: a linked list of free
"holes" stored **in-band in the free memory itself** (raw pointer/address
manipulation, no owning std container — spec §1.6), with **first-fit placement,
alignment-aware splitting, and adjacent-hole coalescing on free** (spec §1.5,
§3.6). This module (`src/lib.rs`) is a thin adapter over that engine; the
allocation-strategy and coalescing logic are the crate's, unmodified.

## Adaptations beyond the source (disclosed per the task)

The crate exposes a `Heap` with `allocate_first_fit(Layout) -> NonNull<u8>` and
an **`unsafe fn deallocate(ptr, layout)` that requires the caller to pass the
original `Layout` back**, and no `realloc`. The spec requires a different API
surface, all added in `src/lib.rs`:

1. **Caller-provided region, borrowed not `'static`.** `Allocator<'a>` manages a
   caller-supplied `&'a mut [u8]` (spec §2.1) via `PhantomData`, instead of the
   crate's `'static`/raw-pointer framing.
2. **Pointer-only `free`/`realloc`.** The spec's `free(ptr)` and
   `realloc(ptr, new_size)` carry no `Layout`. To recover it, each allocation
   carries a small **in-band header** (`Header`, `repr(C)`) immediately before
   the returned user pointer, recording the block base, the layout passed to the
   engine, the caller's requested alignment, and the requested size. The public
   constant `HDR` is this header's size (40 bytes on 64-bit, `<= 64` per §3.6).
   For an `align == 1` request the allocator consumes exactly `HDR + size` bytes
   (rounded to the 8-byte free-list granularity), which is what the A22 exact
   1 KiB carving relies on.
3. **Alignment handling.** The user pointer is placed `pad = round_up(HDR, align)`
   bytes into an `align`-aligned engine block, so the returned pointer honors any
   power-of-two alignment up to `MAX_ALIGN = 4096` (§1.3) with the header sitting
   just below it.
4. **Errors as values.** All failures are the in-band `AllocError` enum
   (`InvalidSize`, `InvalidAlign`, `InvalidPtr`, `OutOfMemory`) mapping to the
   spec's `E_*` values (§2); nothing panics or aborts on any input (§1.8).
   `init` on a region too small even for engine metadata degrades to an
   allocator whose every request returns `OutOfMemory` rather than panicking.
5. **Invalid-pointer / double-free rejection (§2.3).** `free`/`realloc` validate
   the pointer against the region bounds *before* any dereference, then check a
   liveness marker (`magic`) in the header. `free` clears the marker before
   releasing the block, so a double free — or a null, out-of-range, or stale
   pointer — is rejected with `InvalidPtr`.
6. **`realloc` (§2.4).** Added entirely. It always allocates the replacement
   (preserving the block's original alignment from the header), copies
   `min(old, new)` bytes, then frees the original. If the allocation fails the
   original is left live and unchanged (`OutOfMemory`). Placement/OOM timing is
   implementation-dependent, as §3.8/§4.5 permit.
7. **`free_bytes` / `region`** informative accessors (§2.5, §3.1), used only by
   test assertions.

## Idiomaticity / lint notes (criterion §2.5)

- `cargo clippy` (default lints) is clean, with **one documented suppression**:
  `#[allow(clippy::not_unsafe_ptr_arg_deref)]` on `free`/`realloc`. These take an
  arbitrary caller `*mut u8` and validate it against the region bounds before any
  dereference, so they are genuinely memory-safe for every input (including wild,
  interior, and null pointers). The lint cannot see that guard and would force an
  `unsafe` signature, which contradicts the spec's error-as-value contract. This
  is not a memory-model safety relaxation — it is disclosed for the adjudicator
  per §2.5.
- The core library is `#![no_std]` (backing region is caller-provided); only the
  tests use `std`.

## Layout

- `src/lib.rs` — the allocator (core logic, `no_std`).
- `tests/vectors.rs` — the frozen suite, one test per vector `A1..A22`,
  including the full deterministic xorshift64 stress sequence (A21, seed
  `0x2545F4914F6CDD1D`, 20 000 steps, 256 slots, per-step no-overlap / alignment
  / in-bounds / fill-integrity assertions) and the partial-coalescing anti-cheat
  (A22, run on a region sized to exactly ten 1 KiB blocks so the reclaimed span
  is the only place the 3 KiB request can fit).

## Running

```sh
cargo test        # all A1..A22 vectors
cargo clippy --all-targets
cargo fmt --check
```
