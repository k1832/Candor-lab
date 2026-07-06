# Spec: General-Purpose Allocator (`spec-allocator.md`)

**Status:** FROZEN on hash. Authored blind to Candor design docs (see README).
**Source obligation:** `BET5_CRITERION.md` §2.4(a). This spec restates and
sharpens those requirements; it never weakens them.

---

## 1. Purpose & required features

1.1 The program is a **general-purpose dynamic memory allocator** that manages a
single contiguous **caller-provided byte region** ("the region"). All backing
storage comes from that region; the allocator MUST NOT obtain memory from any
other source (no OS calls, no global heap). This keeps the spec freestanding.

1.2 The allocator MUST service **arbitrary allocation sizes** in bytes, from 1 up
to the largest that fits the region's current free space.

1.3 The allocator MUST honor a **caller-specified alignment** that is any power of
two from 1 up to `MAX_ALIGN = 4096` inclusive.

1.4 The allocator MUST support **freeing an individual block** previously
returned, in any order, without freeing others.

1.5 On free, the allocator MUST **coalesce adjacent free blocks** into a single
larger free block — *or* implement a named, justified equivalent (e.g. segregated
free lists with buddy-style split/merge) that yields the same observable
reclamation behavior of §3.6. The chosen mechanism MUST be recorded in the port
README and confirmed by the adjudicator (criterion §6.5).

1.6 Free-list bookkeeping MUST be maintained via **raw address / pointer
manipulation** over the region (in-band headers or out-of-band metadata), not via
an owning container of the language's standard library.

1.7 **Minimum managed heap.** The implementation MUST correctly manage a region
of at least `MIN_REGION = 1 MiB` (1,048,576 bytes). Metadata overhead is
permitted but MUST be internal to the region; the allocator MUST NOT require more
than `MIN_REGION` bytes to satisfy the suite of §4.

1.8 The allocator MUST NOT crash, panic, abort, or invoke undefined behavior on
any input, including invalid input and exhaustion. All failure is reported **as a
value** (§2).

---

## 2. Abstract interface

Operation names are indicative; **semantics are binding**. Errors are values
returned in-band, never faults.

2.1 `init(region) -> Allocator`
- `region`: a mutable contiguous byte span of length `N >= MIN_REGION`.
- Returns an allocator managing exactly `region`. All later operations act only
  within `region`.

2.2 `alloc(a, size, align) -> Result<Ptr, AllocError>`
- Returns a pointer/offset into `region` to a block of at least `size` bytes,
  aligned so that `(address mod align) == 0`.
- Errors (returned as values):
  - `E_INVALID_SIZE` if `size == 0`.
  - `E_INVALID_ALIGN` if `align == 0`, `align` is not a power of two, or
    `align > MAX_ALIGN`.
  - `E_OUT_OF_MEMORY` if no free block can satisfy `size`+`align` given current
    fragmentation.

2.3 `free(a, ptr) -> Result<(), AllocError>`
- Releases a block previously returned by `alloc`/`realloc` and not yet freed.
- Errors: `E_INVALID_PTR` if `ptr` was not a currently-live allocation of `a`
  (out of range, interior, or already freed). Implementations that cannot detect
  every invalid pointer MUST at minimum reject out-of-range and null pointers;
  double-free of a still-tracked block MUST be rejected.

2.4 `realloc(a, ptr, new_size) -> Result<Ptr, AllocError>`
- Resizes the live block at `ptr` to at least `new_size`, **preserving the
  alignment the block was originally allocated with** and preserving the first
  `min(old_size, new_size)` bytes of content. The returned pointer MAY differ
  from `ptr` (the block MAY move); if it moves, the old block is freed.
- Errors: `E_INVALID_SIZE` if `new_size == 0`; `E_INVALID_PTR` as in 2.3;
  `E_OUT_OF_MEMORY` if the resize cannot be satisfied — in which case the
  original block MUST remain live and unchanged.

2.5 (Optional, non-scored) `free_bytes(a) -> integer` — total currently
allocatable bytes, used only by informative test assertions, never required for
correctness.

**Resolution of "realign" (criterion §2.4a suite wording).** The randomized
"realign" workload of §2.4(a) is realized by `realloc` (2.4): resizing forces
blocks to move and thereby be re-placed at fresh addresses that still honor the
original alignment. This is the binding interpretation of "realign".

---

## 3. Observable-behavior requirements (invariants)

For every sequence of operations, the following MUST hold and are checked by §4.

3.1 **In-bounds.** Every returned pointer and its whole block lie within
`region`.

3.2 **Alignment honored.** Every returned pointer satisfies
`(address mod requested_align) == 0`.

3.3 **Size honored.** The usable block is at least the requested size.

3.4 **No overlap of live blocks.** At any instant, the byte ranges of any two
distinct live allocations are disjoint.

3.5 **Metadata integrity.** Content written by the caller into a live block is
never altered by the allocator or by operations on *other* blocks. A distinct
fill byte written to each block MUST read back unchanged until that block is
freed or realloc'd.

3.6 **Reclamation and coalescing (observable).** If blocks occupying a combined
contiguous span are all freed, a subsequent single `alloc` of a size up to that
combined span (minus at most one block-header's worth of overhead, `HDR <= 64`
bytes) MUST succeed. Concretely: fill the region with K equal blocks, free all K,
then an allocation of `K * blocksize - HDR` bytes MUST succeed — proving adjacent
free blocks were coalesced rather than left fragmented.

3.7 **Exhaustion is graceful.** When space is insufficient, `alloc`/`realloc`
return the OOM error value; no live block is disturbed.

3.8 **Determinism of success/failure (same-implementation).** For a **fixed
implementation**, given the same region size and operation sequence, the set of
which operations succeed vs. return which error MUST be reproducible run to run.
**Across** implementations this set need not match: §5.2 grants placement-strategy
freedom (first-fit, best-fit, segregated lists), and different strategies fragment
differently, so which steps hit `E_OUT_OF_MEMORY` is implementation-dependent. What
every implementation MUST hold on every run are the §3.1–3.5
overlap/alignment/size/integrity invariants and the graceful-OOM guarantee (§3.7).
Actual addresses likewise MAY differ between implementations (declared
nondeterminism).

---

## 4. Frozen test suite (language-agnostic vectors)

All vectors use `MIN_REGION = 1 MiB` unless stated. "MUST error X" means the
named error value is returned and no state changes.

### Nominal
- **A1.** `alloc(16, 1)` succeeds; write 16 bytes; read back equal.
- **A2.** `alloc(1, 1)` succeeds (minimum size).
- **A3.** `alloc(4096, 4096)` succeeds; returned pointer is 4096-aligned.
- **A4.** Ten `alloc(1000, 8)` calls succeed; all ten pointers 8-aligned and
  pairwise non-overlapping (3.4).
- **A5.** `alloc(64,64)`, write fill `0xA5`, then `alloc(64,64)` for a second
  block, write `0x5A`; first block still reads `0xA5` (3.5).
- **A6.** `alloc(100, 64)` (a **64-aligned** block), fill `0xC3`, then `realloc`
  it to 400 bytes; the first 100 bytes still read `0xC3`; the new pointer is 400
  bytes usable **and is still 64-aligned**. `realloc` MUST preserve the block's
  original alignment (2.4) — a vacuous `align=1` request could not catch a
  non-preserving implementation, so the alignment here is non-trivial.
- **A7.** `realloc` a 400-byte block down to 50 bytes; first 50 bytes preserved;
  the operation succeeds.

### Boundary
- **A8.** Alignments `1,2,4,8,16,32,...,4096` each requested with size 32; each
  returned pointer honors its alignment (3.2).
- **A9.** Allocate the largest single block the empty region admits (implementation
  reports it via `free_bytes` or by binary search); it succeeds; one more byte
  fails with `E_OUT_OF_MEMORY`.
- **A10.** Coalescing (3.6): with `blocksize = 1024`, allocate until full (K
  blocks), free all K, then `alloc(K*1024 - 64, 1)` MUST succeed.
- **A11.** Free in reverse order of allocation, then in interleaved order; both
  followed by A10-style full-span reallocation succeeding.

### Error
- **A12.** `alloc(0, 8)` MUST error `E_INVALID_SIZE`.
- **A13.** `alloc(16, 0)` MUST error `E_INVALID_ALIGN`.
- **A14.** `alloc(16, 3)` (not a power of two) MUST error `E_INVALID_ALIGN`.
- **A15.** `alloc(16, 8192)` (> MAX_ALIGN) MUST error `E_INVALID_ALIGN`.
- **A16.** `alloc(2 MiB, 1)` on a 1 MiB region MUST error `E_OUT_OF_MEMORY`.
- **A17.** `free` of a null / out-of-range pointer MUST error `E_INVALID_PTR`.
- **A18.** Double free: `p = alloc(...)`; `free(p)` ok; second `free(p)` MUST
  error `E_INVALID_PTR`.
- **A19.** `realloc(new_size=0)` MUST error `E_INVALID_SIZE`.
- **A20.** `realloc` that cannot fit MUST error `E_OUT_OF_MEMORY` and leave the
  original block live and unchanged (verified by reading its fill bytes).

### Deterministic pseudo-random stress (A21)

4.1 **PRNG.** xorshift64 (Marsaglia), 64-bit state, all arithmetic mod 2^64:
```
state := 0x2545F4914F6CDD1D            # seed (nonzero)
next():
    x := state
    x := x XOR (x << 13)
    x := x XOR (x >> 7)
    x := x XOR (x << 17)
    state := x
    return x
```

4.2 **Slots.** Maintain 256 slots, each empty or holding {ptr, size, align,
fill}. `SLOTS = 256`.

4.3 **Step rule.** For `i` in `0 .. 20000`: draw `w := next()`; then:
- `slot := (w >> 2) mod 256`
- `op   := w and 0x3`
- If `op in {0,1}` (**ALLOC**): if `slot` empty, let
  `size := ((w >> 10) mod 4096) + 1`, `align := 1 << ((w >> 24) mod 5)`
  (i.e. one of 1,2,4,8,16), `fill := (w >> 32) and 0xFF`. Call `alloc(size,
  align)`. On success, write `fill` to all `size` bytes and record the slot. On
  `E_OUT_OF_MEMORY`, leave slot empty. If `slot` non-empty, no-op.
- If `op == 2` (**FREE**): if `slot` non-empty, first verify all its bytes equal
  its `fill` (assertion 3.5), then `free`; mark empty. Else no-op.
- If `op == 3` (**REALLOC**): if `slot` non-empty, let
  `nsize := ((w >> 10) mod 4096) + 1`; verify current fill bytes; call
  `realloc(nsize)`; on success rewrite `fill` to all `nsize` bytes, update
  recorded size/ptr; on OOM keep the old block (verify unchanged). Else no-op.

4.4 **Per-step assertions.** After every step, over all non-empty slots: (3.1)
in-bounds, (3.2) alignment, (3.3) size, (3.4) pairwise non-overlap, (3.5) fill
integrity. Any violation fails the suite.

4.5 **Reproducibility.** With the fixed seed and rule above, the **draw sequence**
— the `op`/`slot`/`size`/`align`/`fill` values produced by the PRNG — is identical
for every implementation. The **set of steps that hit `E_OUT_OF_MEMORY` is NOT
required to match across implementations**: placement-strategy freedom (§5.2) makes
OOM timing implementation-dependent, so each implementation replays its own OOM set
deterministically (§3.8, same-implementation) while two implementations may diverge
on which draws OOM. The per-step invariants of §4.4 MUST hold for every
implementation regardless.

### Partial coalescing (targeted anti-cheat)

- **A22 (partial coalescing of an adjacent middle run).** Let `HDR` be the
  implementation's declared per-block overhead (the same `HDR <= 64` of §3.6).
  Carve the region into **10 contiguous 1 KiB blocks** by ten successive
  `alloc(1024 - HDR, 1)` calls `b0 .. b9` (each request sized so one block plus its
  header occupies exactly 1024 bytes, laying the ten down back to back). Then
  `free(b3)`, `free(b4)`, `free(b5)` — freeing **only** the adjacent middle three
  while `b0,b1,b2,b6,b7,b8,b9` stay **live**. Now `alloc(3*1024 - HDR, 1)`
  (three-blocks-worth net of one header) **MUST succeed**: the only place it can fit
  is the reclaimed `b3+b4+b5` span, which requires that the three freed blocks were
  **coalesced into one 3 KiB free region**. This catches two cheaper
  implementations that A10/A11 (free-**all**, then one big alloc) let pass: (i) a
  **bump allocator that only resets when the region is completely empty** — here the
  region is never empty (seven blocks stay live), so a bump-reset cheat has no
  contiguous run to hand back and the alloc fails; (ii) a **non-coalescing free
  list** that keeps `b3,b4,b5` as three separate 1 KiB free entries — no single
  entry is `3*1024 - HDR` bytes, so the alloc fails. Only genuine adjacent-free
  coalescing satisfies A22. (Numbered A22 — the next free index — to avoid
  renumbering the frozen error and stress vectors.)

---

## 5. Non-goals

5.1 **Thread safety / concurrency** is NOT required; the suite is single-threaded.
5.2 **Performance, throughput, fragmentation-optimality, and allocation strategy**
(first-fit, best-fit, etc.) are NOT graded; only the §3 invariants and §4 outcomes
are. The criterion measures cognitive load, not speed (criterion §8.2).
5.3 **Detecting every invalid pointer** is NOT required beyond §2.3's minimum.
5.4 **Shrink-in-place vs. move on realloc** is an implementation choice; only the
observable guarantees of 2.4 are graded.
5.5 **Reclaiming memory to the OS / growing the region** is out of scope; the
region is fixed at `init`.

---

**Revision history.** 2026-07-06: revised per blind adversarial review #1 (`docs/reviews/2026-07-06-basket-specs-review-1.md`); findings 4, 6, 10 applied.
