# 0004 — Safe typed field projection on `rawptr`

**Status:** draft
**Date:** 2026-07-08
**Philosophy hooks:** P1 (unsafe is explicit/local/auditable), P2 (local
verifiability), P13 (clarity-density; the reclassification canon), §2 priority
order, Bet 5 and its v4.2 disposition, P12's concession clause. Subordinate to
`LANG_PHYLOSOPHY.md` and to design 0001, which it extends (§4.2's op set). Where
they conflict, the higher document wins and this one changes.

**Revision note (2026-07-08).** Amended per adversarial review #1 (`docs/reviews/2026-07-08-design-0004-0005-review-1.md`), findings 1, 5, 6: the `table_version`-bump framing is withdrawn for a §9 philosophy-tier a-priori ruling plus a counter-update / region-rule-preservation requirement; E0510 is named as a new checker rule tied to the 0003 §2.5 edit; and impact is restated in the frozen region metric (≈1 valve statement; the two shrinks metric-invisible).

This document discharges the **first** of the two binding commitments attached
to the v4.2 provisional confirmation of Bet 5 (Appendix A, v4.1→v4.2): *"the
next design round takes up safe typed field projection on `rawptr`."* The
second (call-site reborrow ergonomics) is a separate document. It is written to
be implemented in the prototype checker, because the same disposition binds the
scheduler to be **re-ported and re-measured under the frozen successor rules**
before any syntax freeze, and the re-port is the artifact that uses this op.

## Problem

`rawptr T` has no safe field projection. Given a `rawptr StructT` handle,
computing the *address* of one of its fields — a pure arithmetic act that reads
and writes nothing — requires an `unsafe` region today, because the only way to
express it is the composite

```
cast_ptr[FieldT](ptr_offset(cast_ptr[u8](p), conv isize (offsetof(StructT, f))))
```

every intrinsic of which (`cast_ptr`, `ptr_offset`) is gated by E0501 (0003
§2.5). So `&p->f` — address-of-a-field, no dereference — lands in the valve.

This puts *pure address computation* inside the pressure valve and inflates
valve surface on pointer-rich code, precisely the class P12's concession names
as Candor's home ground. Quantified from the frozen ports:

- **Scheduler** (`ports/candor/scheduler/scheduler.cn`, 19 `unsafe` blocks). Four
  functions carry `offsetof`-based forward field addressing that is pure
  arithmetic: `t_link` (`&t->link`, its *entire* body), and the field-precise
  writers `t_set_prio` / `t_set_state` (the address half of a
  `cast_ptr∘ptr_offset∘offsetof` followed by a `ptr_write`). `t_link` is a valve
  block that contains **no memory access at all** — it computes an address and
  returns it.
- The scheduler README's friction note 2 counts "~10 one-line accessor valves"
  a language with handle projection "would not count." That framing is
  optimistic and this document declines it: the deref-bearing accessors
  (`t_id` / `t_prio` / `t_state` read a whole `Task` via `ptr_read`;
  `t_set_prio` / `t_set_state` end in `ptr_write`) **must keep an unsafe op**,
  because dereference stays gated (below). Only *pure-address* accessors leave
  the valve. The honest avoidable cost is narrower than the note implies, and
  saying so is the point.
- **Allocator** (`ports/candor/allocator/allocator.cn`, 9 `unsafe` blocks). It
  uses **zero** `offsetof` and one `addr_of` (region base). Its in-band metadata
  is addressed by raw `usize` byte offsets (payload−40, …) over untyped memory,
  not by typed struct-field projection. Projection helps it **essentially not at
  all** — the conceded 63%-valve class (P12) is untouched, and this document
  does not pretend otherwise.

## Decision

Add one **safe** operation to the core:

```
field_ptr(p, f)     // p : rawptr StructT,  f : a field of StructT   =>   rawptr FieldT
```

- **Form.** An intrinsic in the existing family (`offsetof`, `cast_ptr[U]`,
  `addr_to_ptr[T]`). `f` sits in **field-selector position** — the same
  grammatical slot as `place.f` — so it parses without a symbol table (NN#13):
  `.`/field-position always denotes a field by grammar, resolved by the checker
  against `p`'s statically known pointee type, never by resolving `f`'s type. No
  `[T]` bracket is needed: `StructT` is carried by `p : rawptr StructT`
  body-locally (P2 — nothing crosses a *signature* by inference; a pointer's own
  element type at the use site is not a signature). A `p ->. f` operator spelling
  is a real-language syntax question deferred to §Rejected — the throwaway
  register takes the loud intrinsic.
- **Semantics — plain arithmetic, no null check.** `field_ptr(p, f)` is defined
  as `address(p) + offsetof(StructT, f)`, **unconditionally**. It desugars
  exactly to the `offsetof`+`ptr_offset` composite it replaces, so it does what
  that composite does: on a null `p` it yields the offset as an address (null
  for a field at offset 0, a small non-null address otherwise). **Null is not
  propagated.** Justification: (a) a null check is a hidden branch, forbidden by
  P4/P9 (no invisible control flow or cost) and by "projection is pure
  arithmetic"; (b) it would pretend the op knows something about validity it does
  not; (c) it must agree with `ptr_offset`, which does not null-check. The result
  is an inert address either way — meaning it requires a *gated* deref, and a
  bad-address deref traps as `BadPointer` (0003 §2.6, already outside claim (e)).
  No new fault, no new UB.
- **No bounds or validity knowledge.** `field_ptr` knows a field's static offset
  and nothing else. It asserts nothing about `p`'s validity, initialization, or
  provenance — exactly like `offsetof`, and unlike a borrow.
- **Checker rule (E0510, new).** `p` must have type `rawptr StructT` for a struct
  (or a compiler-known struct type), and `f` must be a **statically known field**
  of `StructT`; otherwise a **new checker rule, E0510** — a well-formedness
  diagnostic, not an unsafety gate. `field_ptr` is **not** gated by E0501 — it
  joins `offsetof`/`is_null` on the safe side of the boundary. It carries no
  effect (§3.2). This adds a rule to the checker and to 0003's soundness boundary:
  **0003 §2.5's safe-carve-out enumeration is edited to name `field_ptr` and its
  E0510 rule** (and 0001 §4.2's operation table gains the same), so the three
  documents enumerate one safe set (review finding 5).

**Why it is safe — the §4.2 audit line.** 0001 §4.2 draws the line at *"every op
that gives a raw pointer meaning is inside `unsafe`,"* and 0003 §2.5 enumerates
the gated set as the ops that *"give a raw pointer meaning"* —
read/write/create/cast/offset — with `offsetof` and `is_null` and
holding/copying a `rawptr` explicitly carved out as safe. `field_ptr` gives **no
meaning**: it neither reads nor writes memory, creates no borrow, and yields an
address that is inert in safe code (§4.2: a `rawptr` value may be held, copied,
and compared safely; only acting on it is gated). It is `offsetof` (safe) plus
copying a computed address (safe) packaged as one typed op. It therefore belongs
on the safe side of the line the line already drew, and it does **not move the
line**: dereference — the act that gives a pointer meaning — stays gated.

## What this does NOT change

- **Deref/read/write stay `unsafe`.** `ptr_read`, `ptr_write`, `addr_of(_mut)`,
  `cast_ptr`, `addr_to_ptr`, `ptr_null`, `ptr_offset` remain E0501-gated. You may
  now compute a field's address in safe code; you still may not read or write
  through it without an `unsafe` block and its justification.
- **The audit line's meaning, restated:** every op that reads, writes, creates,
  or reinterprets a pointer is inside `unsafe`; address *arithmetic* that
  produces an inert address is safe. `field_ptr` is the second kind, joining
  `offsetof`.
- **Soundness (0003) claims (a)–(e): no impact, argued.** (a) no uninitialized
  reads — `field_ptr` reads nothing. (b) XOR aliasing — it produces a `rawptr`,
  which is outside the borrow system entirely; it creates no loan. (c) no
  use-after-move/drop — it operates on `rawptr` (a `copy`, untracked scalar) and
  yields another; nothing tracked is touched. (d) no allocation in a non-`alloc`
  context — it has no effect. (e) faults delivered — it defines no fault
  condition (address arithmetic does not overflow-fault, as with `ptr_offset`); a
  null-derived address that is later dereferenced traps as `BadPointer`, which
  (e) already excludes. Because claims (a)–(e) hold "for the safe fragment and
  explicitly exclude raw-pointer meaning" (0003 §2.5), and `field_ptr` adds
  address arithmetic without adding memory access, the soundness argument is
  untouched — it gains one more entry in the safe-carve-out list, not one more
  proof obligation.

**Estimated impact, in the frozen metric (honest, from the actual ports).** The
frozen metric counts **valve statements** — statements intersecting an `unsafe`
region (§4.1) — so the impact is stated in that unit, not in block counts.
- *Scheduler:* **≈1 valve statement removed** — `t_link`, whose whole `unsafe`
  body was pure forward projection: its block disappears, so its statement(s)
  leave the valve entirely. The block-count view (19 → 18 blocks) is a coarser
  restatement of the same single removal. The **two "shrinks"** (`t_set_prio`,
  `t_set_state`: the 3-op `cast_ptr∘ptr_offset∘offsetof` address expression
  collapses to one safe `field_ptr`) are **metric-invisible**: each block keeps
  its `ptr_write`, so the statement still intersects an `unsafe` region and still
  counts as exactly one valve statement — the arithmetic got cleaner inside the
  region, but the region, and therefore the count, is unchanged. The three read
  accessors, `task_of` (inverse `container_of` — a *negative* offset to a type
  that is **not** a field of the source; not expressible as projection and
  correctly still a valve), the sentinel/arena `addr_of` sites, the whole-node
  list splices, and `th`'s index arithmetic are **unchanged**. Net in the frozen
  metric: **≈1 valve statement**, at `t_link` — modest and real, not the "~10"
  the README implied.
- *Allocator:* **0.** No `offsetof`; its valve is genuine in-band-metadata deref
  over untyped memory. P12's concession stands exactly as written.

A secondary, non-counting benefit: `field_ptr` makes **field-precise reads and
writes** through a handle clean (`ptr_read(field_ptr(t, id))`,
`ptr_write(field_ptr(t, prio), v)`) — the hygiene the setters currently buy with
in-`unsafe` `offsetof` arithmetic, now with the address safe and only the access
gated.

**Interaction with the counting rules (the un-gameable part).** The frozen metric
(`docs/BET5_CRITERION2.md`, ratified and frozen under v4.2) is **region-based**:
a valve statement is one that intersects an `unsafe` region (§4.1). A **safe** op
therefore auto-excludes from the count with *nothing to classify* — there is no
unit-table slot for it to occupy and, contrary to an earlier draft of this
section, **no `table_version` bump is required or appropriate** (bumping a frozen
criterion to admit a token the metric never counts would be both unnecessary and
criterion-prohibited). That framing is withdrawn. What replaces it, so the
yardstick is fixed a priori and the shrink is not chosen after the fact:

- **A §9 philosophy-tier a-priori ruling.** That `field_ptr` is
  safe-hence-uncounted is recorded as a **philosophy-tier ruling** — above the
  criterion, not a modification of it — in `ADJUDICATIONS.md` under an explicit
  philosophy-tier heading, **BEFORE any re-port code exists**. It rests on the
  same ground that made `offsetof`/`is_null` safe carve-outs (the op gives no
  memory access), applied once, in the open, before the numbers.
- **A counter-update requirement, with the region rule preserved.** The counter
  must be updated to parse the new grammar (`field_ptr`), and the update must
  ship with a test **demonstrating the frozen region rule is preserved** — a
  `field_ptr` outside any `unsafe` region contributes zero valve statements, a
  `field_ptr` textually inside one still counts exactly as its enclosing region
  does (the cross-counter fixture is extended to cover both). The metric
  definition does not move; only the parser learns a new safe token.

Because the ruling is philosophy-tier and pre-recorded, and the metric is
region-based, the shrink cannot be a post-hoc reclassification of a frozen
criterion. If an adjudicator instead holds `field_ptr` should be treated as
valve-bearing, the reduction above is zero by definition and the confirmation is
unaffected — either way the number is fixed before the re-port.

## Rejected alternatives

- **Keep it `unsafe` (status quo).** The measured cost: `t_link` is a valve block
  with no memory access, and every field-precise write drags a 3-op `offsetof`
  incantation through the valve. Pure address computation sits in the audit
  surface an auditor must read as *dangerous*, diluting the signal of the ops
  that actually touch memory. Rejected — this is the P12-concession cost the
  disposition told us to attack.
- **Full safe pointer arithmetic (`ptr_offset` in safe code).** Breaks the audit
  line: arbitrary offsets have no static field guarantee, so a safe out-of-object
  address becomes trivial and the "every meaningful pointer act is gated"
  invariant weakens toward Zig's. `field_ptr`'s restriction to a *statically known
  field of the pointee type* is exactly what keeps it arithmetic-with-a-proof
  rather than free arithmetic. Rejected.
- **Auto-deref sugar (`p.f` reads/writes through a `rawptr`).** This gives the
  pointer *meaning* — it reads or writes memory — which is precisely what the
  audit line gates. It would put unchecked memory access into safe code through a
  syntax door. Rejected outright; it is the thing §4.2 exists to forbid.
- **Projection behind a justification-free `unsafe`.** Ceremony without
  information (P13): an `unsafe` block whose justification would read "computes an
  address, touches nothing" trains reviewers and models to skim `unsafe`, which
  degrades the audit surface for the blocks that *do* touch memory. A valve token
  should mean danger; an address computation is not danger. Rejected.

## Consequences and costs

- **One more operation in the core (P6).** The budget is "adding requires
  removing," and this adds `field_ptr`. What pays for it: the manual
  `cast_ptr∘ptr_offset∘offsetof` forward-projection idiom is **demoted from the
  canonical vocabulary** (P3 — one way): it remains reachable inside `unsafe` for
  the inverse/exotic cases `field_ptr` cannot express (`container_of`), but is no
  longer the blessed way to take a field's address. `offsetof` itself is **not**
  removed — `task_of`/`container_of` still need it. So the honest ledger is +1
  core op, paid partly by canonicalization, not by a clean deletion; the residual
  is a named P6 debt, not a managed one. An adjudicator who holds the payment
  insufficient may treat `field_ptr` as owing the budget.
- **The a-priori-ruling requirement (process cost).** Recorded above and
  un-dodgeable: because the criterion is frozen and region-based, `field_ptr`'s
  safe-hence-uncounted status is fixed by a **philosophy-tier ruling pre-recorded
  in `ADJUDICATIONS.md` before the re-port**, not by any amendment to the frozen
  table (there is no `table_version` bump — the metric never counts a safe op).
  The counter must additionally be updated to parse the new grammar with a test
  showing the frozen region rule is preserved. This is process cost the
  disposition imposed by fixing the yardstick a priori — correctly, since a
  movable yardstick proves nothing.
- **Does shrinking the valve make `unsafe` less greppable-complete?** No, argued.
  The audit query — "show me everything this program can read, write, or trust"
  (P1/P17) — enumerates `unsafe` blocks, boundary modules, and `assumed-proven`
  contracts. Every op that reads, writes, or creates a pointer stays in `unsafe`,
  so the enumeration still covers every memory-access site; `field_ptr` reads and
  writes nothing, so its absence from a block hides no capability. And projections
  are themselves **typed and greppable** (`grep field_ptr`): the address-
  computation surface stays queryable — it simply no longer inflates the *danger*
  count. The valve becomes *more* honest, containing only genuinely dangerous
  ops, not arithmetic noise. Greppable-completeness is a property of the
  memory-access set, and that set is unchanged.
- **Bootstrap/measurement honesty.** The reduction is small — **≈1 valve
  statement in the frozen metric** (the `t_link` block; the two scheduler shrinks
  are metric-invisible, and the allocator is nil). Presented as small, it
  strengthens rather than weakens the Bet 5 record: the disposition's authored-vs-
  registered gap is an ecosystem-maturity fact (Bets 4/6), and this op does not
  paper over it — it removes a genuine but narrow measurement artifact (pure
  arithmetic mis-counted as valve) without touching the allocator's real spine.

## Reclassification record (§2 rule)

This decision turns on a reclassification and records it per §2. The P12
concession filed projection as a *"P13-vs-P2 trade,"* which invites reading it as
an item-7 ergonomics/brevity win (fewer tokens). It is argued here as an **item-3
(local verifiability)** win: moving pure address arithmetic out of `unsafe`
reduces what a *reviewer* must read and hold in mind as dangerous — the valve
token set shrinks to genuine memory-access ops, so the auditor's danger surface
is smaller and truer. The cost demonstrably falls on the reader/verifier (the
audit-surface reader), not merely on the author's keystrokes, which is the §2
condition for a legitimate promotion. The brevity is incidental; the verification
saving is the argument. Recorded so the promotion is auditable and not a relabel.
