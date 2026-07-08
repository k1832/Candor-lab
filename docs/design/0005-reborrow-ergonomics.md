# 0005 — Call-site reborrow ergonomics

**Status:** draft
**Date:** 2026-07-08
**Philosophy hooks:** P13 (clarity density — information per token a reviewer
must READ), P2 (local verifiability — everything a caller must know is in the
signature), P3 (one canonical way), NN#11 (formatter is the only form), NN#17
(nothing crosses a signature by inference), Bet 5 / P12 (value-first
ordering), the v4.2 binding commitment ("call-site reborrow ergonomics in the
next design round").

Subordinate to `LANG_PHYLOSOPHY.md` and to `docs/design/0001-memory-model.md`,
which this document amends at §2.1. Where they conflict, they win and this
document changes.

**Revision note (2026-07-08).** Amended per adversarial review #1 (`docs/reviews/2026-07-08-design-0004-0005-review-1.md`), findings 2, 3, 4, 7, 8, 9: the mutation-audit P2 concession is named and option (d) weighed against 0004's greppability rationale; the 0003 update and its scheduled re-review are named; the type-aware formatter requirement and its non-rewrite cases are stated with a prototype-has-no-formatter note; rule 1's `p.f` example is replaced with a §3.4-legal one; the temporary-reborrow residual is stated and "exactly one form" scoped to argument position; and the newly-legal spelling is re-characterized as "ill-formed (type-mismatch or use-after-move by shareability)."

## Problem

Design 0001 §2.1 made call-site reborrowing explicit: passing a held borrow
`b` *down* to a `read`/`write`-mode parameter is written `read (deref b)` /
`write (deref b)`, never bare `b` (bare `b` is the value gear — a *move* of
the borrow value, which is almost never meant). The clarification recorded its
own revisit trigger, in the open: "if the basket shows it is frequent, an
implicit-reborrow rule is the recorded revisit — a P13-vs-P2 trade decided in
the open." The v4.2 amendment made taking up that trade a binding commitment.
The basket evidence is now in, and the trigger has fired.

**The measured evidence.** Counting `read (deref …)` / `write (deref …)`
reborrow expressions across the four borrow-using ports
(`ports/candor/{scheduler,parser,arena,mmio}/*.cn`), split at the `// Test
harness` marker:

| Port | reborrow sites | implementation | harness | recorded friction (README) |
|------|---------------:|---------------:|--------:|----------------------------|
| scheduler | 41 | 7 | 34 | "a steady tax" — every helper call under the same two borrows |
| parser | 66 | 57 | 9 | ~25 recursive-descent cursor sites: "the single largest source of reading noise" |
| arena | 135 | 49 | 86 | "heaviest in `fold_into`, `ir_eq`, `eval` — all recursive over two borrows" |
| mmio | 54 | 21 | 33 | thin driver; reborrows are helper hand-downs, not the valve |
| **total** | **296** | **134** | **162** | — |

296 reborrow expressions; not one of them is a valve, an aliasing decision, or
a place the author chose anything. Every one is the mechanical consequence of
"this parameter is a borrow and I am holding a borrow."

**The P13 case FOR change.** `write (deref pos)` is four tokens (`write`, `(`,
`deref`, `pos`, `)`) carrying exactly one bit: *re-lend, don't move*. P13's
measure is information per token a reviewer must **read**, and the reviewer
skips this ceremony every time — it is the "boilerplate all readers skip" that
P13 prices as expensive at any brevity. The parser proves it at scale: its
recursive descent is a wall of `write (deref pos)`, and the port author names
it the single largest reading-noise source in a program that has *zero valves*
— i.e. the noise is not danger, it is friction pretending to be diligence.

**The P2 case AGAINST change — and why it does not hold.** The objection: a
bare identifier that sometimes moves and sometimes reborrows depending on the
callee's parameter mode is exactly the call-site invisibility P2 forbids — the
reader cannot judge `f(b)` locally without knowing what `f` does to that
argument. But P2 answers its own objection. P2 does not require that meaning
be visible *in the call-site token*; it requires that "everything a caller
must know is in the **signature**." The move-vs-reborrow outcome is determined
entirely by the callee's parameter mode — `write usize` forces an exclusive
reborrow, `read usize` a shared one, a by-value borrow-typed `take` parameter
a move — and that mode is written in the signature, which P2 already obliges
the reader to consult (and which NN#17 guarantees is complete). The explicit
`write (deref pos)` does not *add* a fact the signature lacks; it *re-states
at the call site* a fact the signature already carries. That is redundancy,
not locality.

## The options space

**(a) Status quo.** Explicit `read (deref b)` / `write (deref b)` at every
reborrow. Honest for it: the reborrow point is visible without consulting the
signature. Against: 296 sites of redundancy; the recorded revisit trigger has
fired on three of four ports.

**(b) Fully implicit reborrow (Rust-shaped).** Bare `b` (a held borrow) passed
to a matching `read`/`write`-mode parameter *is* a reborrow, never a move.
Moves of a borrow value happen only in non-call contexts (binding, return) and
when passing to a by-value borrow-typed parameter. Borrowing a **fresh local**
still wears the keyword (`f(write x)`). For it: removes exactly the redundant
bit, keyed on a fact P2 already localizes to the signature. Against: the
reader must consult the callee signature to see the reborrow — answered above
(they must anyway).

**(c) A compact explicit marker.** One visible token — `f(rewrite b)` /
`f(reread b)`, or a symbol-table-free sigil `f(^b)`. For it: keeps the
reborrow point self-evident at the call site; parses without semantic context
(NN#13). Against: it still re-encodes the signature's mode, so by P13's
*information-per-token* test its content is zero regardless of how few tokens
it costs — a shorter way to write nothing new. It also introduces a **third**
spelling of reborrow into the corpus (P3 cost).

**(d) Implicit for shared reborrows only.** `read` is aliasable and harmless,
so make bare `b` a shared reborrow; keep exclusive re-lending loud (`write
(deref b)`). For it: the shared/exclusive asymmetry is a real intuition — the
exclusive case is the one that suspends the parent (§2.1). Against: the
disambiguating fact (the parameter mode) is in the signature for `write`
exactly as much as for `read`; and the measured sites are overwhelmingly
**exclusive** (the parser cursor, the arena walkers, the scheduler helpers are
all `write`), so this removes the ceremony from the rarest case and keeps it
on the common one. It also splits reborrow into two spellings (P3).

## Decision

**Adopt option (b): implicit reborrow of a held borrow; explicit keyword
retained for borrowing a fresh local.** Precisely:

1. **Reborrow (new, implicit).** In argument position for a parameter of mode
   `read`/`write`, if the argument is a **place that already denotes a
   borrow** — a borrow-typed local `b`, or a chained reborrow `(deref b)`
   (borrow-typed struct/enum fields and slice fields cannot denote a borrow,
   §3.4, so there is no `p.f` case) — whose pointee type and shareability
   admit the parameter's mode, the argument is a **reborrow** governed by
   §2.1's reborrow rule — a fresh borrow constrained not to outlive the
   source, suspending (exclusive) or freezing-to-shared (shared) the parent
   for its live range. Bare `b` to such a parameter is **never** a move.
   `write`-param + exclusive source ⇒ exclusive reborrow; `read`-param +
   exclusive-or-shared source ⇒ shared reborrow; `write`-param + shared source
   ⇒ the existing "cannot reborrow exclusive from shared" error, unchanged.
2. **Fresh borrow (unchanged, explicit).** Lending **owned storage** to a
   `read`/`write`-mode parameter still wears the keyword: `f(write x)`,
   `f(read x)`, where `x` is an owned local/place. Passing an owned place bare
   to a mode parameter (`f(x)`) is ill-formed — the fix diagnostic points at
   `write x` / `read x`.
3. **Value gear (unchanged).** Passing a borrow-typed value to a by-value
   (`take`) parameter is the value gear: shared borrows copy, exclusive
   borrows move — bare, no keyword, exactly as today.

**The deciding argument, keyed to §2.** The move-vs-reborrow bit is an item-3
(local verifiability) fact, and P2 already discharges it: it lives in the
callee's parameter mode, in the signature, where NN#17 keeps it complete and
the reviewer is already obliged to read it. The explicit `write (deref b)`
therefore carries no item-3 information the reader lacks; its only remaining
cost is item-7 reading friction — 296 sites of it, three ports naming it their
heaviest. Removing a token sequence that re-states a signature fact loses
nothing verifiable (item 3 intact) and buys back the reading axis P13 exists
to protect. A lower item (7) is not being bought by sacrificing a higher one
(3); the higher item was never in play, because the signature already held it.

**Why fresh-borrow keeps its keyword — the `f(write x)` vs `f(b)` line,
answered.** The distinction pays. `write x` marks a real semantic event: a
loan on **owned storage** is *born* here, freezing `x` for the call — a new
fact in the reader's borrow-check mental model (§2.2's XOR ledger gains an
entry). A reborrow of a borrow already held is not a birth: the storage is
already lent; `b` merely sub-leases a lease the reader is already tracking,
and §2.1's parent-suspension is recoverable from the mere fact that `b` is
handed to a mode parameter. So `write x` earns its tokens under P13's own test
(a keyword marking an aliasing distinction), and `write (deref b)` does not (a
keyword marking a mode the signature already states). The asymmetry is
principled, not a compromise.

**Interaction with the checker (Stage 3).** Small, and stated so it is not
overclaimed. §2.3's loan machinery already creates and scopes reborrow loans;
the only change is where the reborrow node comes from. When the checker meets
a bare borrow place in argument position for a `read`/`write`-mode parameter,
it **inserts the reborrow node** (`read`/`write` of `deref place`) as a
syntactic desugaring at the front of the pipeline, then runs liveness + the
conflict scan unchanged. No new lattice state, no coercion pass, no change to
the XOR check — one desugaring rule keyed on (argument is a borrow-typed
place) × (parameter mode is `read`/`write`) × (pointee admits the mode). The
§2.1 use-after-move diagnostic that previously fired on bare `b` simply has
nothing to fire on: bare `b` no longer moves.

**Interaction with hash-frozen 0003 (the tripwire).** This decision contradicts 0003's §3 conservatism #4 as hashed ("explicit call-site reborrows required; no implicit call-site reborrow"), so 0003's freeze-step-(i) hash tripwire **fires**. It is handled in the same change series, not deferred: 0003 §3 #4 is **rewritten** to the implicit rule, the **desugaring rule is documented in 0003 §2.2**, and a **fresh-session re-review of 0003 is scheduled as part of the implementation verification pass, before the scheduler re-port** — counts stay inadmissible until that re-review passes (0003's standing rule). See finding 3 of `docs/reviews/2026-07-08-design-0004-0005-review-1.md`.

**Interaction with §8.2.1 pattern bindings.** None adverse. A reborrow that
feeds a scrutinee — `match arena_get(read (deref ar), root)` (arena §11.5) —
becomes `match arena_get(ar, root)`; the pattern-binding mode is still derived
from the *result* type (`read Node`), which is unchanged. Pattern semantics
are untouched.

**Interaction with the frozen unit table (M1) and the successor
registration.** Measurement-neutral, and this is load-bearing. §2.1 already
records that these are **use-site tokens**, which `BET5_CRITERION2.md`
(ratified and **frozen** under v4.2) deliberately excludes from M1's
signature-site annotation counts. So this change moves no M1 number and cannot
perturb any frozen metric. The recorded *cost* of the status quo was reading
friction, not an M1 figure — precisely the axis M1 does not measure and
precisely the axis this change improves. The scheduler re-port (a v4.2 binding
commitment, to be re-measured under the frozen successor rules before any
syntax freeze) will be **authored under the new rule**; because the rule
touches only excluded use-site tokens, its M1 re-measurement is unaffected, so
re-measurement neutrality holds by construction.

## Rejected alternatives

- **(a) Status quo — rejected.** The recorded revisit trigger has fired: 296
  sites, "the single largest source of reading noise" (parser), "heaviest in
  the recursive walkers" (arena), "a steady tax" (scheduler). Keeping it is
  choosing to re-encode a signature-borne fact 296 times against P13's
  explicit ruling that reader-skipped boilerplate is expensive at any brevity.
- **(c) Compact marker (`rewrite`/`^b`) — rejected.** P13's test is
  information per token, and the marker's information is zero: it re-states
  the parameter mode. Shrinking a redundant thing does not make it
  non-redundant. Worse, it adds a third reborrow spelling to the corpus
  (status-quo `write (deref b)`, plus the marker, plus the eventual bare
  form), a P3 tax whose only content is a fact the signature already fixes.
  The one merit — reborrow visible without signature consultation — is worth
  less than P2's standing guarantee that signature consultation is required
  and cheap.
- **(d) Implicit shared only — rejected.** The disambiguator (parameter mode)
  is in the signature for `write` exactly as for `read`, so the P2 argument
  for implicitness does not distinguish them. Empirically it removes the
  ceremony from the rarest case: the measured sites are dominated by exclusive
  reborrows (`write (deref pos)`, `write (deref ar)`, `write (deref s)`), so
  (d) would leave the tax exactly where it is heaviest and split reborrow into
  two spellings (P3). The shared/exclusive intuition is real but pays nothing
  here. **Weighed against 0004's greppability rationale, on the record:** (d) is
  in fact the option that would *preserve* the mutation-point audit's call-site
  locality — keeping exclusive re-lends loud is keeping the mutation points
  greppable, the same value 0004 protects when it shrinks `unsafe` (every
  memory-access site stays greppable). That is (d)'s strongest point, and it is
  weighed and declined: the mutation bit is in the signature for `write` exactly
  as for `read` (P2/NN#17), and because the measured sites are overwhelmingly
  exclusive, (d) keeps the tax on the common case to buy a greppability the
  signature already provides — a worse trade than (b)'s uniform rule plus the
  named P2 concession (Consequences and costs).
- **Fully implicit including fresh-local borrow (`f(x)` for owned `x`) —
  rejected.** Dropping the keyword on fresh borrows too would erase the
  visible birth of a loan on owned storage — an aliasing event the reader's
  local borrow-check reasoning genuinely uses, and one not otherwise legible
  without combining the callee signature with `x`'s ownership status at the
  call site. That keyword clears P13's bar; the reborrow keyword does not.
  Cutting both would be cutting past the information line.

## Consequences and costs

- **The reborrow point is no longer self-evident from the call-site token
  alone.** A reader who wants to know whether `f(b)`'s argument is reborrowed
  must read `f`'s signature. This is a real cost, named: it is the same act
  the reader already performs for every argument to judge move-vs-copy, and P2
  both requires and cheapens it (complete, local signatures; NN#17). It is not
  a *new* non-locality — it is the standing one, now covering one more
  decision.
- **P2 concession — the mutation-point audit loses call-site locality (a real
  cost, named).** Under the status quo, `write (deref b)` marks at the call site
  every point where a held borrow is re-lent for mutation, so a reviewer auditing
  "where can this state be mutated through a re-lend" could grep the explicit
  exclusive-reborrow form. Under implicit reborrow that audit now requires reading
  the callee signature for each bare argument — the mutation point is no longer
  self-evident from the call-site token. This is a genuine **P2 (local
  verifiability) concession**, recorded in the documented-limitation style, and it
  honestly **argues the less-guarded reclassification direction** (it is a reason
  one could have kept the ceremony). It is weighed against **0004's greppability
  rationale**, where the parallel move — shrinking `unsafe` — was defended
  precisely because *every memory-access site stays greppable*; the reborrow case
  is weaker on exactly that axis, since the mutation-audit query does lose
  call-site greppability here where 0004's audit query lost none. The decision
  still stands — the disambiguating bit lives in the signature (P2 localizes it;
  NN#17 keeps it complete), the signature carries the bit, and option (b)'s 296
  sites are the measured cost of the alternatives — **but it is an argued trade,
  not a settled matter**, and this concession is the strongest thing said against
  it.
- **P3 — the old explicit form must not survive as an optional second way.**
  Two spellings of one meaning is a P3 violation, so this is decided, not left
  open. **Disposition: canonical-formatter-normalized, not ill-formed.** The
  shipped formatter (NN#11) rewrites `f(write (deref b))` / `f(read (deref
  b))` → `f(b)` wherever the argument fills a matching `read`/`write`-mode
  parameter, so the corpus shows exactly one form **in reborrow-argument position** — not "exactly one form" globally: the explicit borrow operators stay first-class everywhere else, so a temporary reborrow bound to a local (`let r = write (deref b);`), a reborrow used as a field base (`(deref d).base`), and any reborrow whose operand is a non-place all keep the explicit spelling. That temporary-reborrow residual is why the claim is scoped to argument position, not the whole corpus. The explicit spelling is
  *accepted on input* (hand-written and model-generated code, and the
  migrator, all keep working) but never appears in formatted source — the same
  discipline as any formatter normalization. Ill-formedness was rejected as
  the disposition: it would demand a bespoke checker error distinguishing
  "reborrow argument identical-if-bare" from the still-legal general
  expression `let r = write (deref b);` and field-access `(deref d).base`,
  buying nothing over normalization; `deref` and the explicit borrow operators
  remain first-class everywhere outside this argument position.

  **The formatter must be type-aware, and there are cases it must not rewrite.** Deciding whether `f(write (deref b))` → `f(b)` is legal at a given site requires the parameter's mode and type and the argument's type — a purely syntactic rewrite is unsound. Two classes must be **left alone**: (i) a **`take`-mode borrow-typed parameter**, where bare `b` is a *move* of the borrow value while `write (deref b)` is a *reborrow* — the two genuinely differ, so normalizing one to the other would change semantics; and (ii) a **non-place argument** (e.g. `write (deref f())`, or any reborrow whose operand is not a place denoting a borrow), which the implicit rule does not cover and the formatter must not collapse. Only a reborrow argument whose operand is a borrow-typed place filling a matching `read`/`write`-mode parameter is normalized. **Prototype note:** the prototype has **no formatter** — both the explicit and bare forms parse there, and this document's scheduler re-port is written directly in the **canonical bare form**; the formatter normalization obligation therefore binds the **real toolchain (P16)**, recorded as a P16 obligation, not a prototype feature.
- **No silent behavior change — a soundness note.** The newly-legal spelling
  (bare borrow to a mode parameter) was, under the status quo, **ill-formed** — a
  **compile error** in every case, though the *kind* of error depended on the
  source's shareability: an exclusive borrow bare was a **use-after-move** on the
  moved borrow value, while a shared borrow bare was a **type mismatch** (a `copy`
  borrow value handed to a mode parameter that wants a fresh borrow), not a move.
  Either way it was never a different *legal* behavior. So no existing correct
  program changes meaning; the change only turns a class of **ill-formed**
  (type-mismatch or use-after-move by shareability) sites into the intended
  reborrow — pure legalization. This is why the migration is safe under P15
  without behavioral review.

## Migration note

The four ports' 296 reborrow sites are mechanically migratable — the migrator
*is* the formatter normalization rule of the P3 disposition, run once: `write
(deref b)` / `read (deref b)` → `b` at every matching mode-parameter argument.
It is a P15 mechanical migration in the strict sense (a syntactic rewrite with
no behavioral change, per the soundness note above). The scheduler re-port
required by the v4.2 confirmation will be written under the new rule from the
start; because the rule touches only M1-excluded use-site tokens, its
re-measured annotation density is unchanged and the re-measurement remains a
clean test of the frozen criterion.

## Reclassification record

This decision turns on the §2 rule of reclassification, recorded here as the
rule requires.

The status quo's explicit reborrow ceremony was justified in 0001 §2.1 on
**item-3** (P2) grounds: "the reviewer sees exactly where a second-gear handle
is re-lent." This decision reclassifies that ceremony's cost as **item-7**
(ergonomics), and does so legitimately under §2's test — the cost demonstrably
falls on the **reader**, not the author (296 reader-skipped sites; three ports
name it their heaviest reading friction), which is the P13 promotion §2
sanctions. The move: the *information* the ceremony appeared to contribute
(reborrow-not-move) is itself an item-3 fact, but it is already discharged by
the parameter mode in the signature (P2/NN#17). Once that fact is seen to live
in the signature, the call-site ceremony carries no residual item-3 content —
it is item-7 boilerplate whose apparent item-3 value was double-counting a
fact the signature already owns. Removing it therefore sacrifices no higher
item to serve a lower one, which is what §2 forbids and what this
reclassification confirms is not happening.

The retained keyword — `f(write x)` on a fresh borrow of owned storage — is
*not* reclassified: it carries genuine item-3 content (a loan is born on owned
storage here, an aliasing fact not otherwise legible at the call site), so it
stays priced at item 3 and stays explicit, per P13's own test that a keyword
marking an aliasing distinction earns its tokens.

