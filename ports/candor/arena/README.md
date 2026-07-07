# Candor port: arena-based compiler pass (`spec-arena.md`)

Single file, `arena.cn`. Implementation above the `// Test harness` marker
(R14); the AR1–AR29 vector harness below it. `candor-proto run` returns the
sentinel `777` on full success; suite runtime ≈ 25 ms.

## Architecture

- `copy enum Node` — the seven kinds of spec 2.6; operands are `u32` NodeIds
  (arena indices), the index gear of design 0001 §11.5.
- `struct Arena { mem: [256]Node, count: u32 }` — plain fixed array owned by
  value plus a bump count (§11.5's sketch, the non-`Box` variant). `arena_get`
  asserts id liveness (`id < count`), which is what makes `arena_reset`
  (count = 0) observable: the whole generation's ids stop resolving at once.
- `fold` is a single post-order traversal (`fold_into`) into a freshly created
  `dst`, with per-kind rule appliers (`fold_add`…`fold_neg`) whose `return`
  order is literally the spec §3.2 first-applicable-wins order.
- Wrapping semantics (§3.1) via `wrapping`-block helpers `wadd/wsub/wmul/wneg/
  wdiv`. Verified against the toolchain before porting: `/` truncates toward
  zero, and inside `wrapping`, `MIN / -1 → MIN`, `-MIN → MIN`, `MAX*2 → -2`,
  `MIN-1 → MAX` — exactly the spec's table, no workaround needed.
- Equality oracle: structural recursion (`ir_eq`) over the unfolded trees.
  The spec's `S(·)` is injective on unfolded trees, so `S`-string equality
  and `ir_eq` coincide (spec 3.3; harness-side, per R14).

**Valve count: zero.** No `unsafe` block, no `rawptr` anywhere in the file —
including the harness. §11.5 predicted the valve confined to a `Box` backing;
with a plain owned array backing, it does not arise at all.

## Adaptations flagged for adjudication

1. **`fold` return shape.** Candor has no tuple type, so
   `fold(src, root) -> (dst, new_root)` (spec 2.5) returns
   `struct FoldOut { dst: Arena, new_root: u32 }`. Conforming per the spec §2
   preamble (names indicative, semantics binding); the index arena has no
   interior self-references, so returning it by move is sound (contrast R16's
   scheduler case). Semantics of 2.5 hold: fresh `dst` created inside `fold`,
   `src` held by `read` borrow only.
2. **`arena_get` liveness enforcement.** Spec 2.3 says the id "must be a live
   id of this arena"; the port enforces it with `assert(i < count)`, so a dead
   id faults. Spec 1.4's no-crash clause covers well-formed IR; a dead id is
   ill-formed use. This is also what makes reset invalidation (2.4) observable.
3. **Fixed capacity 256.** The spec fixes no capacity; the frozen corpus needs
   fewer than 16 nodes per arena. Allocation past capacity faults on the
   bounds check (outside well-formed use, as above).

## Friction notes (Bet 5 data)

1. **`out` is a hard keyword**, so the natural binding name for a fold result
   (`let out = fold(...)`) is rejected by the parser; renamed `fo`. Minor.
2. **`i64::MIN` has no literal spelling.** Unary minus is an arithmetic op, so
   `-9223372036854775808` faults (overflow) under the default regime at
   runtime. MIN must be constructed inside a `wrapping` block
   (`-MAX - 1`). Harness-only here (AR20/AR21 inputs), but it is a real trap:
   the program checks clean and faults at the literal's evaluation.
3. **No nested builder calls.** `badd(write s, bc(write s, 1), ...)` would put
   two overlapping loans on `s` in one argument list (no two-phase borrows,
   design 0001 §2.3 accepted limitation), so IR construction is one statement
   per node. Verbose but mechanical; this is the friction 0001 predicted.
4. **Explicit call-site reborrow ceremony** (`read (deref x)` /
   `write (deref x)`) on every recursive call and helper hand-down — same
   finding as the allocator and scheduler ports; heaviest here in `fold_into`,
   `ir_eq`, and `eval`, which are all recursive over two borrows.
5. **Positive datum:** the wrapping regime blocks match the spec's §3.1
   semantics exactly (including truncating division and the MIN/-1 wrap), so
   the folding evaluator needed no bit-twiddling workarounds at all — unlike
   the xorshift shims the allocator/scheduler harnesses needed for missing
   bitwise operators.
6. **Positive datum:** the whole pass is pure value + index gear. `Node` being
   `copy` means `arena_get` returns an owned copy — no borrow returns, no
   region annotations anywhere in the file (the compact default of 0001 §3.3
   never even triggers).

## Vector map

AR1–AR4, AR9–AR11 constant folding; AR5–AR8, AR13, AR14, AR23 identities;
AR12, AR24 div-by-zero/variable divisor unfolded; AR19–AR21 wrapping
boundaries; AR15–AR18, AR22 nested/cascade; AR25 independence + explicit
source-unchanged observation (structural twin); AR26 no-dangling per R13
(every reachable id resolves within `dst`); AR27 semantic equivalence
(v0 = 7 → 13 on both sides); AR28 whole-release reuse (same arena rebuilt,
fresh ids from 0); AR29 determinism (double fold, structural identity ⇔
byte-identical `S`). `check` additionally asserts, on every vector, that the
result's reachable ids resolve within `dst` and that `src.count` is unchanged
by `fold`.
