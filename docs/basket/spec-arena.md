# Spec: Arena-Based Compiler Pass (`spec-arena.md`)

**Status:** FROZEN on hash. Authored blind to Candor design docs (see README).
**Source obligation:** `BET5_CRITERION.md` §2.4(e). Restates and sharpens; never
weakens.

---

## 1. Purpose & required features

1.1 The program allocates IR nodes from an **arena** (a region allocator that is
released **as a whole**, never node-by-node) and performs a **transforming pass**
— constant folding plus the algebraic simplifications of §3 — producing a **new
IR** in a **fresh arena**.

1.2 IR nodes use **cross-node references**: operands are `NodeId`s (arena indices
or arena pointers) into the same arena, forming an expression DAG/tree.

1.3 The arena MUST support **whole-arena release** (reset/free) that invalidates
all its nodes at once and makes the arena reusable. Per-node freeing is neither
required nor used.

1.4 The pass MUST produce IR that is **equal to the frozen expected IR** on the
fixed corpus (§4), and **semantically equivalent** to the input over all
well-defined variable assignments (§3.5). No crash/panic on any input; failure is
not an expected outcome for well-formed IR.

---

## 2. Abstract interface

Names indicative; semantics binding. `NodeId` identifies a node within one arena.

2.1 `arena_new() -> Arena` — an empty arena.

2.2 `arena_alloc(arena, node) -> NodeId` — append `node` (whose operands are
existing `NodeId`s of the *same* arena) and return a fresh `NodeId`. IDs are
stable until `arena_reset`.

2.3 `arena_get(arena, id) -> Node` — the node for `id` (must be a live id of this
arena).

2.4 `arena_reset(arena)` — release **all** nodes at once; every previously
returned `NodeId` of this arena becomes invalid; the arena is reusable for new
allocations afterward.

2.5 `fold(src, root) -> (dst, new_root)` — the transforming pass. It allocates all
result nodes into a **freshly created `dst` arena** and returns `new_root`, a
`NodeId` in `dst`. `src` and its nodes MUST be left unmodified; `dst` MUST be
**independent** of `src` (§3.4).

2.6 **Node kinds:** `Const(i64)`, `Var(id: u32)`, `Add(a,b)`, `Sub(a,b)`,
`Mul(a,b)`, `Div(a,b)`, `Neg(a)`, where `a,b` are `NodeId`s.

---

## 3. Transformation semantics (binding)

3.1 **Arithmetic model.** `Add`, `Sub`, `Mul`, `Neg` fold with **two's-complement
64-bit signed wrapping** (fully defined, comparable across implementations).
`Div` truncates **toward zero**; the single overflow case `i64::MIN / -1` wraps to
`i64::MIN`.

3.2 **Evaluation order.** The pass is a single **post-order** traversal: a node is
simplified only after its operands are already simplified. Applied to a node with
already-simplified operands, the following rules run in order; the **first
applicable rule wins**:

**(a) Constant folding** — if all operands are `Const`:
- `Add(c1,c2) → Const(c1 ⊕ c2)`, `Sub → Const(c1 ⊖ c2)`, `Mul → Const(c1 ⊗ c2)`,
  `Neg(c) → Const(⊖0 c)` (⊕⊖⊗ are the wrapping ops of §3.1).
- `Div(c1,c2)`: if `c2 ≠ 0` → `Const(c1 / c2)` (trunc toward zero, §3.1). If
  `c2 == 0` → **not folded** (the node is kept as `Div(Const c1, Const 0)`).

**(b) Algebraic identities** — when (a) did not apply:
- `Add(x, Const 0) → x`; `Add(Const 0, x) → x`.
- `Sub(x, Const 0) → x`; `Sub(Const 0, x) → Neg(x)`.
- `Mul(x, Const 1) → x`; `Mul(Const 1, x) → x`;
  `Mul(x, Const 0) → Const 0`; `Mul(Const 0, x) → Const 0`.
- `Div(x, Const 1) → x` (`Div(x, Const 0)` is left unchanged).
- `Neg(Neg(x)) → x`.

**(c)** Otherwise the node is kept with its simplified operands.

3.3 **Canonical serialization** `S(node)` (for IR-equality across implementations):
- `Const c → (c <decimal>)` (decimal may be negative).
- `Var v → (v <decimal>)`.
- `Add → (+ S(a) S(b))`, `Sub → (- S(a) S(b))`, `Mul → (* S(a) S(b))`,
  `Div → (/ S(a) S(b))`, `Neg → (neg S(a))`.
Tokens are space-separated; exactly one canonical string per IR rooted at a node.
**IR-equality** = string equality of `S(new_root)` fully unfolded from `new_root`.

3.4 **Arena wholeness / independence.** After `fold(src, root)`, calling
`arena_reset(src)` MUST NOT change `S(new_root)` — `dst` holds its own copy of
every reachable node. Every `NodeId` reachable from `new_root` MUST resolve within
`dst` (no dangling reference, no reference back into `src`).

3.5 **Semantic equivalence.** For every variable assignment under which the source
IR performs **no division by zero**, evaluating `src`-from-`root` and
`dst`-from-`new_root` (using the arithmetic of §3.1) MUST yield the identical
`i64`. (Assignments that divide by zero in the source are outside the equivalence
domain.)

3.6 **Determinism.** Given the same `src`/`root`, `S(new_root)` is identical on
every run and every conforming implementation.

---

## 4. Frozen test suite (language-agnostic vectors)

Inputs and expected outputs are given as canonical S-expressions (§3.3); the
harness builds the input in an arena, runs `fold`, and compares `S(new_root)`.

### Constant folding
- **AR1** `(+ (c 1) (c 2))` → `(c 3)`.
- **AR2** `(* (c 3) (c 4))` → `(c 12)`.
- **AR3** `(- (c 10) (c 4))` → `(c 6)`.
- **AR4** `(neg (c 5))` → `(c -5)`.
- **AR9** `(/ (c 8) (c 2))` → `(c 4)`.
- **AR10** `(/ (c 7) (c 2))` → `(c 3)` (trunc toward zero).
- **AR11** `(/ (c -7) (c 2))` → `(c -3)` (trunc toward zero).

### Algebraic identities
- **AR5** `(+ (v 0) (c 0))` → `(v 0)`.
- **AR6** `(* (v 0) (c 1))` → `(v 0)`.
- **AR7** `(* (v 0) (c 0))` → `(c 0)`.
- **AR8** `(/ (v 0) (c 1))` → `(v 0)`.
- **AR13** `(neg (neg (v 0)))` → `(v 0)`.
- **AR14** `(- (c 0) (v 0))` → `(neg (v 0))`.
- **AR23** `(* (c 1) (* (c 1) (v 0)))` → `(v 0)` (identity cascade).

### Div-by-zero & wrapping boundaries
- **AR12** `(/ (c 5) (c 0))` → `(/ (c 5) (c 0))` (NOT folded).
- **AR24** `(/ (v 0) (v 1))` → `(/ (v 0) (v 1))` (variable divisor, unchanged).
- **AR19** `(* (c 9223372036854775807) (c 2))` → `(c -2)` (i64::MAX·2 wraps).
- **AR20** `(- (c -9223372036854775808) (c 1))` → `(c 9223372036854775807)`
  (i64::MIN−1 wraps to i64::MAX).
- **AR21** `(/ (c -9223372036854775808) (c -1))` → `(c -9223372036854775808)`
  (MIN/−1 wraps to MIN, §3.1).

### Nested & cascading
- **AR15** `(+ (* (c 2) (c 3)) (v 0))` → `(+ (c 6) (v 0))`.
- **AR16** `(+ (* (v 0) (c 0)) (c 5))` → `(c 5)` (`x*0→0`, then `0+5→5`).
- **AR17** `(* (+ (c 1) (c 1)) (+ (v 0) (c 0)))` → `(* (c 2) (v 0))`.
- **AR22** `(+ (+ (c 1) (c 2)) (+ (c 3) (c 4)))` → `(c 10)`.
- **AR18** `(+ (v 0) (v 1))` → `(+ (v 0) (v 1))` (no-op; already minimal).

### Structural / arena invariants
- **AR25 (independence, §3.4).** Build AR15's source; `fold`; then
  `arena_reset(src)`; `S(new_root)` MUST still equal `(+ (c 6) (v 0))`.
- **AR26 (no dangling, §3.4).** For AR17's result, every `NodeId` reachable from
  `new_root` resolves within `dst`; none references `src`.
- **AR27 (semantic equivalence, §3.5).** For AR15 with `v0 = 7`, evaluating both
  source and result yields `13`.
- **AR28 (whole-release reuse, §3.3/2.4).** Build AR22 in an arena, `fold`,
  `arena_reset` the source arena, then reuse it to build AR2 and `fold` again →
  `(c 12)`. Confirms the arena is released and reusable as a whole.
- **AR29 (determinism, §3.6).** Any vector re-run yields a byte-identical
  `S(new_root)`.

---

## 5. Non-goals

5.1 **Common-subexpression elimination, GVN, or DAG-sharing** of the output is NOT
required (and MUST NOT be assumed by the equality check, which compares the
unfolded canonical form of §3.3).
5.2 **Rewrites beyond §3.2** (distribution, strength reduction, reassociation,
`x - x → 0`) are NOT required and MUST NOT be added — they would diverge from the
frozen expected outputs.
5.3 **Preserving division-by-zero as a fault value**, or any runtime evaluation of
the IR beyond the equivalence check of §3.5, is out of scope.
5.4 **Per-node deallocation, garbage collection, or reference counting** inside the
arena is NOT required; whole-arena release (§1.3) is the only reclamation model.
5.5 **Performance** (arena throughput, pass speed, memory footprint) is NOT graded,
only IR-equality, the arena invariants, and semantic equivalence (criterion §8.2).
5.6 **Parsing / printing** beyond the canonical S-expression form used by the
harness is out of scope; the source IR is constructed directly via `arena_alloc`.
