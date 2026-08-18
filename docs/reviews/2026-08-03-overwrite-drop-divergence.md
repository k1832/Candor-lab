# Dossier: overwrite-drop divergence (assignment over a drop-hooked place)

*Lab note, 2026-08-03. Found in the same adversarial review that produced the
array-length and repeat-copy checker fixes. This note is the dossier only — no
fix is applied here. Deciding authority picks the fix slice.*

## The divergence in one sentence

Assigning a new value over a place that already holds a drop-hooked value runs
the old value's `drop` hook on the tree-walking oracle but on **no other
engine** — the MIR interpreter, Cranelift (no-opt and `-O2`), and LLVM all skip
it — so every drop-hooked overwrite is a spec violation and a semantic leak on
the compiled engines.

## Minimal reproduction

```cnr
struct D { id: i64 }
drop(write self) { trace(self.id); }
fn mk(n: i64) -> D { return D { id: n }; }
fn main() -> i64 {
    let mut p: D = mk(1);
    p = mk(9);      // spec 03 §6.8: must destroy the old value (trace 1) first
    p.id = 2;       // scalar field store: no drop involved
    return 0;
}
```

Observed (2026-08-03, all engines from one build):

| engine        | trace    |
|---------------|----------|
| tree-walker   | `[1, 2]` |
| MIR interp    | `[2]`    |
| native no-opt | `[2]`    |
| native `-O2`  | `[2]`    |

The same shape through a **projection** diverges identically — `h.d = D{id:9}`
with `h: H { d: D, n: i64 }` traces `[1, 9]` on the oracle, `[9]` on MIR/native
— so the fix must handle projected places, not just whole locals.

Related shapes checked while narrowing:

- `out`-parameter pre-call drop (04 §6.7(b)): **no divergence yet** — the MIR
  builder rejects `out` mode entirely ("unsupported: param mode Out"). The same
  drop-before-store obligation lands there when `out` does.
- `Vec` element overwrite (`set`): **no divergence** — `CollOp::VecSet` already
  drops the old element before the byte-copy
  (`compiler/src/mir/interp.rs:933-948`, mirrored natively; gated by
  `tests/fixtures/run/vec_drop_overwrite.cnr`). The omission is specific to
  plain place assignment.

## What the spec requires

- **03 §6.8**: "Reassigning a place SHALL first destroy the value it currently
  holds (if any), then store the new one." (`docs/spec/03-types-and-values.md`)
- **03 §7.5**: whole-binding **reassignment is a drop point**, and at every
  drop point of a needs-drop place the initialization state is statically
  path-independent (E0309). So an engine may emit the overwrite drop
  unconditionally (pruned only by the *static* move mask) — no runtime flag.
- **03 §6.7**: a moved-out place is NOT destroyed — the RHS may move the old
  value out of the target (`lst = cons(take lst, ...)`), and that overwrite
  must not drop.
- **04 §6.7(b)**: the same destroy-before-store rule for pre-initialized `out`
  slots, cross-referencing 03 §6.8/§7.5.
- **06 §6.3** is not implicated: no fault is involved; this is the normal path.

The tree-walker is the compliant engine; MIR and everything downstream of it
are wrong.

## Exact code sites

- **Tree-walker (correct)**: `compiler/src/interp/eval.rs:3760-3778`,
  `StmtKind::Assign` — evaluates the RHS first (§1.5 evaluation order), then
  `drop_value(addr, &tty, &mask, ..)` on the old value gated by
  `place_owned`/the local's live mask (the comment there records the
  double-free this gating prevents), then `move_to`.
- **MIR builder (the omission, root cause)**: `compiler/src/mir/build.rs:805-813`,
  `StmtKind::Assign` — `lower_place` + `lower_into(value, &place, &tty)` and a
  `set_owned` bookkeeping call. **No drop of the old value is emitted.** Every
  downstream engine executes this MIR, so one site explains all three divergent
  engines:
  - MIR interpreter: executes only the `StatementKind::Drop` statements the
    builder emitted (`compiler/src/mir/interp.rs:387-391`); none precedes the
    assignment's store.
  - Cranelift (no-opt and `-O2`): `compiler/src/backend/lower.rs:1337`
    translates `StatementKind::Drop`; same absence.
  - LLVM: `compiler/src/backend/llvm.rs:2926`; same absence.
- **Checker (already knows)**: `compiler/src/check/stmt.rs:79-95` records every
  assignment as `Access::Assign { needs_drop: needs_drop(&tty), .. }` for the
  Stage-4 dataflow, and E0309 enforces the §7.5 path-independence that makes an
  unconditional emitted drop sound. The static facts the builder needs already
  exist.

## Recommended fix direction

Emit the overwrite drop in the **MIR builder** (one site fixes all four
downstream engines), mirroring the oracle's order exactly:

1. In `StmtKind::Assign`, when `needs_drop(&tty)` and the target's old value is
   statically owned (prune by the builder's move mask, `self.moves`, exactly as
   scope-exit `emit_drop` does): lower the RHS into a **fresh temp** first,
   then drop the old value of the target place, then `CopyVal` temp → place.
2. The temp is not optional: `lower_into` writes straight into the target, so
   "drop then lower_into" would free the old value before an RHS that reads or
   moves out of it has run — wrong order vs. the oracle and a use-after-drop
   for self-referential RHS. Gate the temp on `needs_drop` so drop-inert
   assignments (the overwhelmingly common case — scalars, copy aggregates)
   lower exactly as today, at zero cost.
3. A place-granular drop is required: `StatementKind::Drop` today takes a whole
   `local` + moved mask. Either extend it with a projection path or add a
   `DropPlace`-shaped statement; all four consumers must implement it
   (`mir/interp.rs` already has `drop_value(addr, ty, ..)` to call; the
   Cranelift and LLVM backends have drop glue to extend at the two sites
   above).
4. Cover `out`-slot pre-call drops (04 §6.7(b)) in the same slice if `out`
   support lands in MIR first.
5. Gate with a `fixtures/run` program asserting the oracle trace (`[1, 2]`
   above) byte-exact on tree/MIR/native-noopt/native-opt, plus the LLVM
   full-corpus sweep in `tests/llvm.rs`.

**Risk notes:**

- *Double-drop through RHS move-out*: the exact bug the tree-walker's gating
  comment records (`eval.rs:3761-3768`). The builder must prune by the move
  state **after** lowering the RHS, not before.
- *Projections*: `h.d = ...` must drop only the field's old value; the
  whole-local `Drop` statement shape cannot express that (hence item 3).
- *Trace-golden churn*: any shipped program that overwrites a drop-hooked place
  would gain hook firings on MIR/native — but the full suite is green today
  with engine-parity trace gates, so no shipped fixture exercises the divergent
  shape; expected churn is zero. New behavior only appears where programs were
  already diverging from the oracle.
- *Perf*: the added temp+copy applies only to needs-drop assignments; the
  P20-gated hot paths (sort, DEFLATE) assign drop-inert values and lower
  unchanged.
- *Hooks can allocate*: a hook running at the assignment point interacts with
  the alloc-effect accounting; the checker already treats reassignment as a
  drop point (`Access::Assign`), so no checker change is expected, but the
  effects gate should be re-run.
