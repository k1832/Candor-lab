# Candor port: general-purpose allocator

Port of `docs/basket/spec-allocator.md` (frozen) to Candor, per
`BET5_CRITERION.md` §2.4(a). One file, `allocator.cn`, containing the allocator
and the full frozen vector suite (A1–A22) in `main`. `main` returns the
sentinel **777** on full success; any vector failure faults (exit 2).

Toolchain: `prototype/target/release/candor-proto check|run ports/candor/allocator/allocator.cn`.

## Mechanism (spec 1.5, for the adjudicator)

A single **address-ordered first-fit free list with adjacent-block coalescing
on free**, threaded through the free blocks themselves (in-band metadata,
spec 1.6). On `free`, the block is inserted in address order and merged with
its predecessor and/or successor whenever the spans touch. `alloc` carves
first-fit: an alignment front gap or tail remainder ≥ 16 bytes stays on the
free list; smaller fragments are absorbed into the allocated chunk (recorded
in its header, so `free` returns the whole span). `realloc` resizes in place
when the existing span already holds the new size, otherwise
allocate-new-with-the-original-alignment / copy / free-old; on OOM the
original block is untouched (spec 2.4; error precedence per ruling R3).

## Declared per-block overhead

**HDR = 40 bytes** (spec 3.6 / A22; ≤ 64, so A10's literal 64 also holds, per
ruling R4). Header layout, immediately below the payload:

| offset | field |
|---|---|
| payload−40 | chunk_start (address of the block's whole span) |
| payload−32 | chunk_size (span length, header + absorbed slack included) |
| payload−24 | payload_size (caller's requested size) |
| payload−16 | align (caller's requested alignment, preserved by realloc) |
| payload−8 | magic (MAGIC_LIVE / MAGIC_FREE — best-effort pointer validity, spec 2.3/5.3) |

A free block's list node (size, next) occupies its first 16 bytes; the magic
word sits at payload−8, above those 16 bytes, so freeing a zero-gap block
never clobbers its own MAGIC_FREE stamp (that is what makes A18's double-free
detection reliable).

The region is caller-provided: `main` owns a `[1048576]u8` array and takes its
address once, inside `unsafe`. A22 runs on a 10 KiB sub-region per ruling R2.

## Valve shape (Bet 5 data)

The valve concentrates exactly where design 0001 §11.1 predicts — the
free-list machinery:

- `heap_init` — plants the initial free node.
- `heap_alloc` — free-list walk / carve / splice, header stamp (the largest block).
- `heap_free` — header validation, magic stomp, address-ordered insert + coalesce.
- `heap_realloc` — header read/update and the move-copy loop.
- `free_bytes` — read-only list walk.
- Harness: `write_fill` / `check_fill` (touching allocator-returned blocks) and
  the one `addr_of_mut` in `main`.

Everything else — argument validation, error values, size/alignment
arithmetic, the xorshift64 PRNG, all vector orchestration — is safe value/borrow
gear. Free-list addresses travel through safe code as plain `usize` values;
every operation that gives one meaning (`ptr_read`/`ptr_write`) is inside an
`unsafe` block with a true justification.

## Deterministic stress (A21)

Exact spec 4.1–4.5: seed `0x2545F4914F6CDD1D`, 20000 steps, 256 slots,
per-step in-bounds / alignment / size / pairwise-non-overlap / fill-integrity
assertions over all non-empty slots. The language has no bitwise operators,
so XOR is a 64-iteration bit loop and the shifts are wrapping multiplies /
unsigned divides by powers of two (inside `wrapping { }` blocks); the PRNG was
verified against the reference sequence (first three outputs
9181757771948286951, 16460966181113277408, 3825608052996350135). Fill writes
and checks go word-at-a-time (8 bytes per pointer op) — without that, the
20000-step suite is not runnable in reasonable time on the tree-walking
interpreter. Runtime: see the commit log / final report (the interpreter
needs on the order of 20 minutes for the full suite; the vector was NOT
shrunk).

## Language friction notes (Bet 5 data)

1. **Borrow-mode parameter of fixed-array type does not parse.**
   `bl: write [1024]usize` fails: after `read`/`write` in a parameter, `[` is
   consumed as a region-variable bracket (grammar 0002 §2.1 `Mode`), so a
   borrow of an array can only be passed by wrapping the array in a struct
   (`struct Blocks` / `struct Slots` here). P0001 "expected a region variable,
   found an integer literal".

2. **Checker false positive: `match` lexically inside `while`.** Locals that
   are live across a `match` sitting inside a loop are reported E0304
   "possibly-uninitialized" even when definitely initialized (minimal repro:
   a `let`-initialized local read as a match scrutinee inside a `while`).
   Workaround: hoist the match into a helper function (`alloc_or_zero`,
   `realloc_or_zero`) so no `match` appears inside any loop body.

3. **Checker false positive: array-element store under `if/else` in a loop.**
   `sp[slot] = v` under a nested `if` inside an `if/else` chain inside a
   `while` downgrades the whole (fully initialized) array to
   "possibly-uninitialized" (E0304) — the single-`if` (no `else`) and no-loop
   variants are accepted. Workaround: the A21 op bodies moved into
   `a21_alloc`/`a21_free`/`a21_realloc`/`a21_check`, which also reads better.

4. **Non-copy enums are unusable in `assert` conditions.** `assert(code(e) == 3)`
   is E0708 (a consuming call in a contract clause) unless the error enum is
   `copy`. Marking the small result enums `copy` is natural here, but a
   result type that owned a payload could not be inspected by a helper inside
   a contract at all.

5. **No bitwise operators** (`&`, `|`, `^`, shifts): xorshift64 costs a
   64-iteration loop per XOR, and power-of-two checks/masks are spelled with
   `%` and `/`. Purely a prototype-surface gap, but it dominates what would
   otherwise be one-cycle operations.

6. **Word-granularity fills as a performance valve widener.** The natural
   byte-loop fill/check had to become 8-bytes-per-`ptr_read/ptr_write` for
   interpreter speed; this puts *more* code inside `unsafe` than the byte
   version would (a perf-driven, not semantics-driven, widening — worth
   noting for M2-style extent counts).

## Spec ambiguities hit (flagged for adjudication)

- **A9's "one more byte fails":** read as *a request of largest+1 bytes* on
  the empty region errors OOM; the port asserts both that reading and the
  stronger post-state (after `alloc(largest)` succeeds, `alloc(1)` also OOMs).
- **A11 "interleaved order":** taken as free evens ascending, then odds
  ascending (spec names no exact order); any fixed interleaving exercises the
  same both-neighbor coalescing.
- **A7's alignment** for the 400-byte block is unspecified; the port uses 8.
  (A6 already covers the non-trivial alignment-preservation case.)
