# 0006 — The real surface syntax

**Status:** draft
**Date:** 2026-07-08
**Philosophy hooks:** P13 (clarity-dense: words where meaning lives, compact
elsewhere; the measure is *information per token a reviewer must read* — the §2
reclassification canon), P3 (one canonical way; the formatter's output is the
only form — NN#11), NN#13 (the grammar parses without a symbol table), P2
(signatures are complete contracts; no cross-signature inference — NN#17), P12
(value-first rendered in syntax: omitted mode = `take`; borrows wear keywords),
P7 (errors are values with a lightweight propagation operator), P6 (small core;
adding requires removing), P16 (one blessed formatter). Subordinate to
`LANG_PHYLOSOPHY.md` and to designs 0001/0004/0005, whose *semantics* this
document re-spells and does not change. Where they conflict, the higher document
wins and this one changes.

**What this is.** The syntax that ships. It retires the deliberately-throwaway
prototype spellings of 0001/0002 now that Bet 5 is provisionally confirmed
(v4.2). It governs the real toolchain and will be adversarially reviewed. It is
mined from ~5,500 lines of genuine Candor in `ports/candor/*` and the friction
their READMEs record; every call below is argued from token economics and
misreading risk, not from any incumbent's convention.

**What this is not.** Not a change to the memory model (0001), the fault model
(P5), the effect set (P2), or the checker's soundness argument (0003). Those are
fixed; this document changes only how they are *written*. Nothing here designs
generics, traits, modules, or FFI — those are deferred (§7) because they belong
to P11/P17 rounds and would move decisions this document must not prejudge.

**Revision history.** 2026-07-08 — revised per adversarial review #1
(`docs/reviews/2026-07-08-design-0006-review-1.md`): conv target restricted to a
scalar keyword; `out` reverted to a hard keyword; complete normative precedence
table (Rust's, reconciled with the `as` rejection); read-only auto-deref adopted;
explicit `ok` marker for result-shaped enums; constant-conv loss and bare
over-range literals made compile errors; counts restated on distinct programs;
migration totality scoped; slice-region spelling designed (OBL-SLICE-REGION).

2026-07-08 — noted per joint adversarial review #1 of designs 0007/0008
(`docs/reviews/2026-07-08-design-0007-0008-review-1.md`): F1 forward-note added to
the `>>` reserved-token entry (§3) — once generics land, region parameters are
declared with the `region` keyword and bare bracketed identifiers are type
parameters.

---

## 1. Principles applied — the token-economics test, operationally

P13's measure is *information a reviewer must read per token*, priced at
Priority 3 (verification), not Priority 7 (author brevity). Operationally, every
construct sorts into one of three buckets, and the bucket fixes the spelling:

1. **Semantic-distinction keywords stay words.** Anything that changes what a
   reader must believe about ownership, aliasing, arithmetic regime, unsafety,
   or a contract is a *word*, because a keyword that prevents a misreading is
   cheap at any length: `read`/`write`/`take`/`out`, `unsafe`, `wrapping`/
   `saturating`, `conv`, `requires`/`ensures`, `drop`, `alloc`. These are the
   places "meaning lives."
2. **High-frequency plumbing gets a compact form.** A token sequence the
   reviewer *skips every time* — that carries no fact the signature does not
   already carry — is boilerplate, expensive at any brevity. It gets a sigil or
   disappears: dereference (`.*`), propagation (`?`), the per-arm `case`, the
   per-call reborrow ceremony (0005), the parentheses around a converted
   primary. Their content is zero-to-one bit; they must cost zero-to-one token.
3. **Rare-and-dangerous stays loud.** Low-frequency, high-consequence acts wear
   a full block and a mandatory justification even though they are rare — an
   auditor's eye must catch them: `unsafe "reason" { … }`. Brevity here is a
   disservice.

The whole document is these three rules applied to the constructs 0001 defines.
Two corpus facts drive most of the compaction: **594 dereferences (266 of them
the exact idiom `(deref x).field`)** and **304 explicit reborrows** — both pure
plumbing, both bucket 2. All counts in this document are measured on **distinct
programs**: the `scheduler-v2` re-port (an already-evolved copy of `scheduler`)
is excluded, correcting a 23% double-count found in review (see §5).

---

## 2. The syntax, by area

Each area gives the throwaway form, the real form, and the P13 argument. Line
counts and site counts are measured from `ports/candor/*`.

### 2.1 Items — `fn` / `struct` / `enum` / `static`, drop hooks

| Construct | Throwaway | Real | Note |
|---|---|---|---|
| function | `fn f(x: T) alloc -> R { }` | *unchanged* | structural keyword stays (bucket 1) |
| struct | `struct S { f: T }` | *unchanged* | |
| enum | `enum E { V(T), W }` | *unchanged* | variants `E::V` keep `::` (NN#13) |
| static | `static N: T = e;` | *unchanged*, immutable | mutable global deferred (P10) |
| drop hook | `struct S {…} drop(write self) { }` | *unchanged* | rare + dangerous ordering event → loud (bucket 1/3) |

Items are the frame a reviewer navigates by; their keywords are the highest-value
words in the language and none of them is plumbing. **No change.** The one
addition is a per-variant `ok` marker for result-shaped enums (§2.4, for `?`),
which needs no new item keyword — an enum is result-shaped exactly when one of its
variants is `ok`-marked.

### 2.2 Types — borrows and slices

**Throwaway:** `borrow T` / `borrow_mut T` (single-place borrows) and
`slice T` / `slice_mut T` (run borrows). The borrow-type keywords are *dead in
the fixtures* (0002 §"Consequences"): borrows are introduced by `read`/`write`
operators and modes, so a borrow *type* is spelled only in a local's annotation,
a return, or a fn-pointer parameter. Slices, by contrast, are ubiquitous.

**Real:**

| Meaning | Throwaway | Real |
|---|---|---|
| shared borrow of a place | `borrow T` | `read T` |
| exclusive borrow of a place | `borrow_mut T` | `write T` |
| shared slice (run view) | `slice T` | `[T]` |
| exclusive slice | `slice_mut T` | `write [T]` |
| shared borrow of a sized array | `read [N]T` (didn't parse) | `read [N]T` |
| exclusive borrow of a sized array | `write [N]T` (didn't parse) | `write [N]T` |
| owned fixed array | `[N]T` | *unchanged* |

**The borrow-type call: `read T` / `write T`, and NOT `&T` / `&mut T`.** This is
the load-bearing type decision, and it is argued *against* the Rust default on
Candor's own terms:

- **The keyword already exists and means exactly this.** `read`/`write` are
  already the borrow *operators* and the parameter *modes*. Spelling the borrow
  *type* with the same word is one-concept-one-word (P3): a reviewer who has
  learned `write x` (produce an exclusive borrow) reads `write T` (the type of
  one) with zero new vocabulary. `&mut T` would introduce a *second* rendering
  of "exclusive borrow" alongside the `write` operator — two spellings of one
  idea, a P3 cost paid for nothing.
- **It keeps `&` for bitwise-and, which the corpus demands.** The single most-
  cited missing operator is bitwise-and/or/xor (§2.3): the allocator and both
  schedulers pay a **64-iteration bit loop** for one xor because `&`/`|`/`^` do
  not exist. If `&` also meant "borrow type," then `a & b` (and) versus `&T`
  (borrow) would be a positional overload the tokenizer must carry forever.
  Reserving `&` *wholly* for arithmetic removes that overload before it is born
  — a direct NN#13 simplification, not a Rust reflex. This is the decisive
  argument.
- **Borrows-wear-keywords is P12 rendered in syntax.** Value-first means the
  *owned* type is the bare one (`T`, `[N]T`) and the borrow is the marked one.
  A keyword marks it; a sigil would make the second gear as quiet as the first,
  flattening exactly the ordering Bet 5 rests on.

**The slice call: `[T]` shared, `write [T]` exclusive.** Here the compact sigil
*wins*, and the asymmetry with single-place borrows is principled: a bare `[T]`
cannot be misread as an owned value (owned runs always carry a size, `[N]T`), so
no keyword is needed to disambiguate it from ownership — and the shared slice is
the cheap, aliasable, ultra-high-frequency case (every `[u8]` text view). P13
says compact where high-frequency and unambiguous. Exclusivity, which *does*
restrict aliasing, still wears `write`. So the rule is uniform: **the keyword
appears exactly where the bare type would otherwise read as owned, and `write`
always appears because exclusivity is a semantic distinction.** This also
dissolves two prototype rules — the "no mode on a slice-typed parameter" ban
(0001 §3.1) and the "borrow-mode array parameter does not parse" friction
(allocator note 1, scheduler note 4, which forced wrapper structs `Buf`/`Arena`/
`Occ`): `s: read [u8]` and `bl: write [1024]usize` now parse as one thing —
a borrow whose shared/exclusive-ness and run/scalar/array shape are all in the
type.

**Slice regions (OBL-SLICE-REGION).** A slice is a borrow, so it can be the
region-tagged input or the region-tagged return of a function that carries region
variables (0001 §3.3). The region hangs on the borrow keyword exactly as for a
single-place borrow (`read[r] T` / `write[r] T`): the exclusive slice is
`write[r] [T]`, and a region-tagged shared slice reuses the shared-borrow keyword,
`read[r] [T]`. The bare `[T]` shared slice stays the compact default for the
annotation-free case (one borrow in / one borrow out, 0001 §3.3); `read` appears
on a shared slice *only* to carry a region variable, never redundantly. So
`fn head[r](s: read[r] [u8], n: read usize) -> read[r] [u8]` is writable — the
return borrows `s`'s region — closing the hole OBL-SLICE-REGION recorded (a slice
plus a second borrow parameter returning a borrow was previously unspellable). The
0001 §3.3 counting rule (a slice parameter counts as a borrow parameter, matching
the checker) is stated there; the throwaway prototype gains a matching stopgap
spelling `slice[r] T` / `slice_mut[r] T`, so the affected function shape is
writable before the real parser exists.

### 2.3 Parameter modes

**Unchanged (P12).** Modes stay `take` (omitted) / `read` / `write` / `out`;
omission = `take` *is* the value-first bet made visible in every signature and is
not negotiable here. All four are bucket-1 words. **`out` stays a hard keyword
everywhere** — reserved in all positions, never an ordinary identifier. The
contextual-`out` relaxation considered in review is rejected: the marker position
is expression-ful (an `out` argument is a place expression), so a contextual
reading would have to disambiguate `out`-the-marker from `out`-the-identifier
*inside that same expression grammar*, and the `alloc` analogy is false — `alloc`
is an effect keyword in a fixed, non-expression slot (`fn … alloc -> …`), never
adjacent to arbitrary expressions. The recorded friction (`let out = fold(…)`
rejected, arena note 1) therefore stands as a real cost, not one this document
fixes; `out` is simply not an available identifier.

### 2.4 Expressions

**Dereference — postfix `.*`, retiring prefix `deref`.** This is the highest-
value single change in the document. The corpus has **594 derefs, 266 of them
the exact shape `(deref x).field`**; the prefix keyword plus its mandatory
parentheses is bucket-2 plumbing wrapping nearly every field touch through a
borrow. Chain depth is *shallow* — essentially always one deref before a field
or index, never a deep `deref deref deref` tower — which is precisely why a
*sigil* suffices and nothing heavier is warranted.

`.*` is postfix, binds tighter than field access and indexing (see the §2.4
precedence table), and reads as "the pointee." It is chosen over postfix `^`
(which would steal the char needed for bitwise-xor) and over C's `->` (see
Rejected).

**Read-only auto-deref (adopted).** A **read** through a tracked borrow needs no
`.*`: for `x: read T` or `x: write T`, `x.f` and `x.mem[i]` read the pointee's
field/element directly — the declaration already carries the fact that `x` is a
borrow, so the sigil adds no information the reader lacks. A **write** keeps every
deref explicit: `x.*.f = v`, `x.*.mem[i] = v`. **Chain rule (the simple one):**
auto-deref applies *at each field-/element-access step* on a **read** path, so a
read drops every `.*` that immediately precedes a `.`/`[…]`; **any write path (an
assignment target) requires every deref written explicitly.** A bare pointee
*value* (not a field access) still takes `.*` in both positions (`b.*`), because
there is no field step to hang the auto-deref on. This preserves exactly the
mutation-visibility cue conceded in 0005: a `.*` on the left of `=` still marks
every store through a borrow, so a mutation audit (grep `.\* =`) stays complete,
while the 277 read-side field/element touches lose their ceremony. Full
explicitness (keeping `.*` on reads too) is the rejected alternative (§6).

| Access | Throwaway | Real (canonical) |
|---|---|---|
| field read | `(deref s).base` | `s.base` |
| element read | `(deref ar).mem[i]` | `ar.mem[i]` |
| field/element write | `deref pos = t.end` / `(deref p).f = e` | `pos.* = t.end` / `p.*.f = e` |
| bare value read | `deref b` | `b.*` |

**Borrow operators — `read` / `write`, unchanged.** They mark the birth of a
loan and stay words (bucket 1). Call-site *reborrow* ceremony is already gone
per design 0005: a held borrow passed bare to a `read`/`write` parameter
reborrows; on real syntax the **304** `read b.*` / `write b.*` sites collapse to
`b` (the formatter, §4). Fresh borrows of owned storage keep the keyword
(`f(write x)`) — that keyword marks a real aliasing event and earns its tokens
(0005).

**Field projection on `rawptr` — `field_ptr(p, f)`, unchanged (design 0004).**
Safe, un-gated, field-selector position; retires the `cast_ptr∘ptr_offset∘
offsetof` incantation as the canonical way to take a field address.

**Conversion — keep `conv`, drop the required parens; target is a scalar-type
keyword; reject `as`.** `conv T (e)` (34 sites, "verbose" per the READMEs)
becomes `conv T e` where the **target `T` is a scalar type keyword** (an integer
or `usize`/`isize`/pointer-width scalar name — the only things `conv` converts;
34/34 corpus sites are scalar) and `e` is a postfix expression (parens only when
the operand needs them): `conv usize i`, `conv u8 x.f`. Restricting the target to
a single scalar-keyword token is what makes dropping the parens unambiguous — see
the §3 walk (`conv` × `[T]`). The keyword *stays* — this is a deliberate refusal
of `e as T`:

- A conversion is a *semantic event*: under 0001 §8.1 a narrowing/sign-changing
  `conv` **faults on value loss** by default (truncates only inside `wrapping`,
  saturates inside `saturating`). It must wear a word (P13: the dangerous thing
  is loud).
- **Compile-time-known loss is a compile error (fold rule).** If the operand of a
  `conv` is a constant expression whose value does not fit the target type, the
  program is **rejected at compile time** (not a runtime fault) — the loss is
  known statically, so it is a static error in the default regime. Inside a
  `wrapping`/`saturating` block the constant is folded by that regime's rule
  instead. A non-constant operand keeps the runtime fault-on-loss behaviour.
- `as` is worse than neutral here: every reader and every model carries the Rust
  prior that `as` *silently truncates*. Adopting `as` for a fault-on-loss
  operation would actively mislead — a real Bet 6 hazard (models generate to
  their prior). `T(e)` is rejected too: it reads as construction/call. A
  distinct word with no incumbent baggage is the honest spelling. Pointer
  retyping stays in the `unsafe` `cast_ptr[U]` / `addr_to_ptr[T]` forms.

**Literal suffixes — unchanged.** `42`, `42u8`, `0x40`, `4096u32`; default
`i64`; suffixes are the integer scalar names (0002 §0.1). Add hex is already
there.

**ADDED — bitwise operators.** The basket's top structural gap: xorshift PRNGs
cost a 64-iteration bit loop (allocator, both schedulers) and power-of-two
masks are spelled with `%`/`/`. Add, with C-family spellings now that `&` is
free (§2.2):

| Op | Spelling | Notes |
|---|---|---|
| and / or / xor | `a & b` / `a \| b` / `a ^ b` | width-exact, never overflow |
| not | `~a` | prefix; distinct from logical `!` |
| shift | `a << n` / `a >> n` | `>>` is arithmetic for signed, logical for unsigned |

Shift-amount `n >= bitwidth(a)` **faults** in the default regime (NN#4 spirit:
no silent nonsense), is masked mod bitwidth inside `wrapping`, and clamps inside
`saturating`. `&`/`&&` and `|`/`||` are distinguished by maximal munch, as today.

**Normative precedence table.** Candor adopts **Rust's operator precedence
wholesale**, scoped to the operators Candor defines — spelling *and* binding match
the dominant prior, and C's famous `&`-below-`==` bug stays fixed the way Rust
fixes it (`a & mask == 0` parses as `(a & mask) == 0`). Tightest first; this table
is normative and the formatter's redundant-paren removal (§4) keys on it:

| # | Operators | Assoc |
|---|---|---|
| 1 (tightest) | postfix `.*` (deref), field `.`, index `a[i]`, call `f(…)` | left |
| 2 | postfix `?` | postfix |
| 3 | prefix `-` (arith neg) `!` (logical not) `~` (bitwise not); borrow `read`/`write`; `conv T` | prefix |
| 4 | `*` `/` `%` | left |
| 5 | `+` `-` | left |
| 6 | `<<` `>>` | left |
| 7 | `&` | left |
| 8 | `^` | left |
| 9 | `\|` | left |
| 10 | `==` `!=` `<` `>` `<=` `>=` | non-associative |
| 11 | `&&` | left |
| 12 | `\|\|` | left |
| 13 (loosest) | `=` (assignment, statement position only) | — |

Within level 1, `.*` binds tightest, so `s.*.f` is `(s.*).f`. Comparison is
**non-associative** (`a < b < c` is a parse error), following the prior.

**Reconciliation with the `as` rejection (recorded).** Adopting Rust's bitwise
spelling *and* precedence is not the contamination the `as` rejection guards
against, and the distinction is principled: `as` was rejected because Candor's
conversion **semantics differ** from the prior (fault-on-loss vs silent
truncation), so the familiar spelling would import a *false* belief. Bitwise-and/
or/xor/shift semantics **do not differ** from the prior (width-exact, the same bit
operations); therefore reusing the familiar spelling and precedence imports a
*true* belief — it is prior-alignment (Bet 6 working *for* the reader/model), not
contamination. The test is whether the prior's meaning matches Candor's, not
whether the token is borrowed.

**ADDED — an `i64::MIN`-expressible literal.** Throwaway `-9223372036854775808`
*faults* (unary minus is arithmetic over an already-overflowing magnitude;
arena note 2 calls it "a real trap: the program checks clean and faults at the
literal"). Fix: **`-` directly preceding an integer literal is a single combined
grammar production `NegLiteral := '-' IntLiteral`** — one compile-time-folded
literal constant, range-checked against its type at compile time. Intervening
**whitespace is permitted** (`- 5` and `-5` both fold); **parentheses break the
production** — `-(9223372036854775808)` is unary negation applied to a
parenthesized primary (the magnitude over-ranges first, so it errors as an
ordinary expression), and `- x` for a variable stays ordinary faulting negation.
So `-9223372036854775808` (or `-9223372036854775808i64`) is a valid `i64`. **A
bare over-range literal with no sign — e.g. `9223372036854775808` on its own — is
a compile error** (it fits no target type), as is any signed literal outside its
target's range; never a runtime fault. Walked in §3. For the programmatic bound,
add compile-time intrinsics **`min_of(T)` / `max_of(T)`** to the `sizeof`/`alignof`
family (greppable, NN#13-clean). An `i64::MIN` associated-constant spelling is
rejected: `::` is reserved exclusively for enum variants (NN#13), and reopening it
is not worth one constant.

**ADDED — `?` propagation (P7's blessed operator), scoped honestly.** The parser
pays a two-arm `match` per fallible construction and hoists `must_box`/
`must_box_args` helpers purely to avoid it (parser note 5: "box ceremony without
`?`"). P7 explicitly blesses a lightweight propagation operator; ship it, but
scope it to what the *no-generics* core can carry:

- **Spelling.** Postfix `expr?`. On a value of a **result-shaped enum**, if it is
  the `ok`-marked variant, the expression evaluates to the unwrapped payload; if
  any other variant, the enclosing function `return`s that value unchanged.
- **Result-shaped enum — the explicit `ok` marker.** Exactly one variant of an
  enum may be prefixed with the keyword `ok`; that variant is the success/unwrap
  variant, and an enum is *result-shaped* precisely when it has an `ok`-marked
  variant. No separate `result` declaration modifier is needed — the marker is
  itself the greppable flag an auditor or model keys on, and it names the success
  variant **position-independently** (it may appear anywhere in the variant list):

  ```
  enum BoxResult { ok Boxed(Box Expr), OutOfMemory }
  ```

  `box(a, v)?` evaluates to the `Box Expr` if the result is `Boxed`, else returns
  the whole `BoxResult` (`OutOfMemory`) from the enclosing function. The parser
  sees only the `ok` token in variant position; which payload `?` unwraps is a
  checker fact (NN#13-clean — no symbol table at parse). The rejected positional
  rule ("first variant is success") is recorded in §6.
- **The honest limit — same-type only, conversion deferred.** `expr?` is legal
  only where the enclosing function returns the *same* enum `E`. Cross-type
  propagation (a callee's `E1` error flowing into a caller's `E2`) needs an
  error-conversion mechanism (`From`-style), which needs traits — **P11, deferred
  (§7)**. The same-type form already erases the parser's `must_box` matches
  (`box(a, v)?`) and every self-recursive fallible call, which is the measured
  cost.

### 2.5 Control flow

**`match` — drop the per-arm `case`.** Throwaway `match e { case P => x, … }`
becomes `match e { P => x, … }`. `case` is pure bucket-2 boilerplate — **185
occurrences** across the corpus, each carrying no information the `P =>` shape
does not. Arms are `Pattern => Expr`, comma-separated (optional trailing/after-
block comma, as today). The `Ident {`-is-not-a-struct-literal restriction in
scrutinee position (0002 §0.7) is retained — it is what keeps `match` symbol-
table-free.

**Loops — unchanged.** `loop { }`, `while cond { }`, `break`/`continue`. One loop
family (P3). No `for` — iterators need traits/generics (deferred, §7).

**`if` — unchanged**, braces mandatory (§4).

### 2.6 Unsafe

**Block + mandatory justification string — unchanged** (P1). It is the canonical
bucket-3 construct: rare, dangerous, loud, greppable, span-bounded. Design 0004
already moved *pure address arithmetic* (`field_ptr`) out of it, so what remains
inside is genuinely memory-touching.

- **Expression position:** `unsafe "reason" { … }` may stand as an expression
  (its block yields a value), so an author can drive a value *before* a valve
  load without a wrapper statement (mmio note: "any future volatile-load modeling
  needs a seam on the access"). This is already grammatically available; it is
  now blessed as canonical for the single-value case.
- **A per-expression `unsafe` modifier stays rejected** (0001 §10.3): a block
  keeps the justification attached to a reviewable region.
- **Grammar fix (statement-position):** a block-like expression in statement
  position *terminates the statement* — a following `(` begins a new statement,
  never an argument list. This removes the "defensive `;` after `unsafe {…}`
  before a `(deref s)…` statement" workaround (scheduler note 1, mmio note).

### 2.7 Contracts, arithmetic regimes, statics

- **Contracts — unchanged:** `requires(c)` / `ensures(c)` / `assert(c)` /
  `panic(m)` / the `result` keyword in `ensures`. All bucket-1 words. Only the
  `enforced` level is in the prototype; the `audit` / `assumed-proven` level
  *spellings* are a small later addition (semantics fixed by P8) and are not
  designed here to avoid guessing their module-attachment syntax.
- **Arithmetic regimes — unchanged:** `wrapping { }` / `saturating { }` blocks,
  greppable, bucket-1 (P5). The corpus uses 21 such blocks and reports them
  matching spec semantics exactly (arena note 5). Named single-operation forms
  stay deferred; the block is canonical.
- **Statics — unchanged:** `static N: T = e;`, immutable (E0311).

---

## 3. NN#13 — the grammar still parses without a symbol table

Every added or changed sigil is walked for ambiguity; each resolves by
*grammatical position or maximal munch*, never by resolving an identifier's type.

- **`&` — borrow-type vs bitwise-and: resolved by elimination.** Borrows are
  keywords (`read`/`write`), so `&` is *never* a type constructor. `&` (and `&&`)
  is always a binary operator in expression position. No overload exists. This is
  the payoff of §2.2's borrow-keyword call.
- **`*` — deref vs multiplication: resolved by the leading `.`** Bare `*` is only
  ever infix multiplication. Dereference is `.*` — a `.` followed by `*` in
  postfix position. After a `.`, the next token is either an IDENT (field) or `*`
  (deref): one-token lookahead, no table. `a * b` (multiply) and `a.*` (deref)
  never compete.
- **`>>` — shift vs future generic close: reserved now.** Candor has no `<…>`
  generic syntax today; when generics arrive (P11, deferred) their bracketing
  **must not** be `<>`, precisely so `>>` stays unambiguously a shift. Recorded
  as a constraint on the future round. When that round lands, a generic bracket
  list declares **region parameters with the `region` keyword** (`fn choose[region
  r, T]`) and treats **bare bracketed identifiers as type parameters** (0007 §6.1),
  so the existing region-variable bracket (`fn pick[r]`, 0001 §3.3) and the new
  type-parameter bracket never collide.
- **`[T]` slice vs `[N]T` array vs `[…]` literal vs `a[i]` index: position +
  content.** Type position vs expression position is grammatical (0002 §3). Within
  a type, after `[` parse one component; if a Type follows the `]` it is an array
  `[N]T`, else it is a slice `[T]` — the component (a const-expr size vs a Type)
  and the trailing Type are lexically distinguishable (scalar-type keywords and
  the type grammar decide, no symbol table).
- **`conv T e` × `[T]` — resolved by restricting the target to a scalar-type
  keyword.** With the parens dropped, `conv <Type> <postfix-expr>` could in
  principle read a bracketed target (`conv [u8] xs`) and then be unable to tell
  where the target type ends and a `[…]` index/literal operand begins — the
  ambiguity the review flagged. It is removed by grammar, not lookahead: the
  target production is **`ScalarType`** (a single scalar type-keyword token —
  `u8`, `i64`, `usize`, …), never a bracketed or composite type. So the token
  after `conv` is always exactly one keyword and the postfix operand begins
  immediately after it; `conv [u8] …` is simply not a production. This matches the
  semantics (conv is scalar-only, 34/34 corpus sites) and needs no symbol table.
- **`read T`/`write T` — mode vs borrow-type overlap: the mode parse is
  canonical.** In parameter position a parameter is `Mode? Type`, and a borrow
  *type* is itself spelled `read T`/`write T` (§2.2). So `p: read T` has two
  derivations — mode `read` over type `T`, or default (`take`) mode over the
  borrow-*type* `read T`. The rule: **in parameter position the leading
  `read`/`write` is parsed as the mode.** Both derivations denote the identical
  thing (a shared/exclusive borrow of a `T` passed in), so the parse is
  unambiguous *in meaning*; the canonical choice fixes the *tree*. The formatter
  never emits the redundant composition (`take read T`): a `take`-mode borrow-
  typed parameter is written `read T`, and there is exactly one spelling.
- **Slice regions `read[r] [T]` / `write[r] [T]` — position-led, no new
  ambiguity.** The region variable `r` sits in the same `[…]` slot it occupies on
  a single-place borrow (`read[r] T`): it is lexically the bracket that
  *immediately follows the borrow keyword* `read`/`write`, and the slice type
  `[T]` follows. `read` is keyword-led, so `read[r] [T]` parses as borrow-keyword,
  region-bracket, slice-type with one-token lookahead after each bracket — the
  same machinery as `read[r] T`. A bare `[T]` (no leading keyword) is the
  region-free shared slice, as before.
- **`-` + integer literal (negative-literal fold) — combined production.** The
  production `NegLiteral := '-' IntLiteral` fires when `-` immediately precedes an
  integer-literal *token* (intervening whitespace allowed); it yields one folded,
  range-checked constant. A `(` after `-` does not match (the operand is a
  parenthesized expression → ordinary unary negation), and `-` before a
  non-literal is ordinary negation. No symbol table: the decision is purely on the
  next token's lexical class (IntLiteral vs `(` vs other). An out-of-range folded
  constant — a signed over-range literal, or a bare unsigned over-range literal
  like `9223372036854775808` — is a **compile error** at this production, not a
  parse ambiguity.
- **`?` propagation: pure postfix.** No ternary `?:` exists (Candor uses `if`
  expressions), so `?` is unambiguously the postfix propagation operator. Result-
  shape is a *checker* fact; the parser sees only `expr?`.
- **`->` stays solely the return arrow.** It was *not* taken for deref (see §6),
  so it never appears in expression position and never competes with anything.
- **`~a`, `min_of(T)`: keyword/position-led**, exactly like the existing `!` and
  `sizeof` families (`conv` is covered above).

No production consults a declaration elsewhere in the file. Two-token lookahead
remains the ceiling (0002 §"Consequences").

---

## 4. Canonical form (P3 / NN#11 — the formatter is the only form)

Brief and decisive; the formatter enforces all of it and there is no option:

- **Indentation:** 4 spaces, never tabs. **Braces:** K&R — opening brace on the
  same line as the construct, closing brace on its own line. **Blocks are
  mandatory** on every `if`/`else`/`loop`/`while`/`match`/`fn`/`unsafe` — no
  brace-less bodies.
- **One statement per line;** simple statements end with `;`. Block-like
  statements take no trailing `;` (the §2.6 grammar fix makes the optional-`;`
  rule unnecessary).
- **Spacing:** one space around every binary operator and `=>`; **no** space for
  postfix `.*`, `?`, field `.`, index `[…]`, or call `(…)`; one space after `,`
  and after mode/borrow keywords (`write x`, not `writex`).
- **Trailing commas** in every multi-line list (params, args, fields, variants,
  match arms); none in single-line lists.
- **Names:** `snake_case` for functions, locals, fields; `PascalCase` for
  structs, enums, and enum variants; `SCREAMING_SNAKE` for `static`s.
- **Normalizations the formatter performs (input accepted, output canonical):**
  - **Reborrow collapse (real syntax).** Explicit reborrow `f(write b.*)` /
    `f(read b.*)` in a matching mode-argument position → `f(b)` (design 0005,
    type-aware — it must *not* rewrite a `take`-mode borrow-typed parameter or a
    non-place operand).
  - **Read-only auto-deref collapse.** A `.*` that immediately precedes a field
    `.` or index `[…]` on a **read** path → dropped (`s.*.base` → `s.base`; §2.4);
    a `.*` on an **assignment target** is always kept (`p.*.f = e` stays), as is a
    bare-value `.*` (`b.*`). Reads carry no `.*` before a field/element; writes
    keep every one.
  - **Redundant-paren removal — uniform.** Parentheses the now-complete precedence
    table (§2.4) makes non-load-bearing are dropped everywhere by one rule (not
    just around a `conv` operand): a parenthesized subexpression whose operator
    binds tighter than its context — or equal with matching associativity — loses
    its parens. This yields the single canonical spelling (`a & mask == 0`, not
    `(a & mask) == 0`, once `&` binds above `==`).
  - **Throwaway spellings** `deref`/`case`/`slice`/`borrow` (and `slice[r]`) →
    their §2 real forms.

---

## 5. Migration (P15 — mechanical, by tool)

The prototype and the six ports migrate by tool when the real parser exists. Most
rows are a syntactic rewrite with no behavioral change (P15 strict sense); the
rows marked *author-assisted* are **not** total by tool and need a human in the
loop. The totality claim is scoped to the mechanical rows only:

| Throwaway | Real | Mechanical? |
|---|---|---|
| `borrow T` / `borrow_mut T` | `read T` / `write T` | yes |
| `slice T` / `slice_mut T` | `[T]` / `write [T]` | yes |
| `slice[r] T` / `slice_mut[r] T` | `read[r] [T]` / `write[r] [T]` | yes |
| `(deref b).f` read / `deref b` / `deref b = v` write | `b.f` / `b.*` / `b.* = v` | yes (auto-deref on reads, §2.4) |
| `read (deref b)` / `write (deref b)` (arg pos) | `b` | yes (0005) |
| `case P => e` | `P => e` | yes |
| `conv T (e)` | `conv T e` | yes |
| xorshift 64-iteration bit loop | `^` / `<<` / `>>` | **author-assisted** (semantic rewrite) |
| `must_box`/two-arm `BoxResult` match | `box(a, v)?` | **author-assisted** (same-type `?` only) |
| `-MAX - 1` (MIN construction) | `-9223372036854775808` | **author-assisted** (idiom recognition, not a token map) |
| wrapper structs `Buf`/`Occ`/`Arena` for array params | `write [N]T` params | **author-assisted** (unwrap: struct removal + field-access rewrite) |

The scheduler-v2 re-port (`ports/candor/scheduler-v2`) is already authored in the
0004/0005 evolved forms and is the reference for the mechanical rows. Being a
re-port of `scheduler`, it is **excluded from every corpus count** in this
document (§1): counting both double-counted its constructs (a 23% inflation the
review caught), so all counts here are on distinct programs.

---

## 6. Rejected alternatives (§8.6 — the load-bearing sigil-vs-keyword calls)

- **`&T` / `&mut T` for borrow types — rejected.** Would positionally overload
  `&` between borrow-type and the newly-added bitwise-and, and add a second
  rendering of "exclusive borrow" beside the `write` operator (P3). `read T` /
  `write T` reuses the existing keyword and frees `&` entirely for arithmetic
  (§2.2). The decisive call of the document.
- **Full deref explicitness (every `.*` written, reads included) — rejected;
  read-only auto-deref adopted instead (§2.4).** Keeping `.*` explicit on reads
  *and* writes would cost the sigil at all **277** read-side field/element sites
  on top of the **68** write sites, for a fact the declaration already carries (a
  name of borrow type is a borrow — the reader knows without the sigil). The
  adopted rule drops the sigil on the 277 reads and keeps it on the 68 writes,
  which is where it earns its tokens: a `.*` on the left of `=` is the
  mutation-visibility cue conceded in 0005, and keeping *every* write-path deref
  explicit means a mutation audit (grep `.\* =`) stays complete. Design 0004's
  "gives the pointer meaning" objection applies to *rawptr*, not to
  checker-tracked borrows, so read auto-deref is memory-safe. Full auto-deref
  (dropping the write sigil too) is *also* rejected, for exactly that audit
  reason: the write boundary must stay visible.
- **`->` for deref-and-field (`s->base`) — rejected.** Corpus-tempting (saves one
  char over `.*.` at 345 sites, C/LLM-familiar) but splits dereference into two
  spellings (`->` for field, `.*` for bare) against P3, and overloads `->` across
  return-arrow and expression positions. The marginal token saving does not buy
  the second spelling.
- **Postfix `^` for deref — rejected.** Steals the natural char for bitwise-xor,
  which the corpus needs more (the xorshift bit loop).
- **`e as T` / `T(e)` for conversion — rejected.** `as` imports the Rust silent-
  truncation prior onto a fault-on-loss operation (a Bet 6 generation hazard);
  `T(e)` reads as construction. Keep the neutral word `conv` (§2.4).
- **Keeping per-arm `case` — rejected.** 185 sites of bucket-2 boilerplate.
- **`i64::MIN` associated constant — rejected.** Reopens `::` (NN#13). Negative-
  literal folding + `min_of`/`max_of` intrinsics cover it.
- **Per-expression `unsafe` modifier — rejected** (0001 §10.3): scatters the
  audit surface; the block keeps justification attached to a region.
- **Positional "first variant is `ok`" for result-shaped enums — rejected in
  favour of the explicit `ok` marker (§2.4).** Which variant is the success/unwrap
  channel is a semantic distinction, and this document's own bucket-1 rule says a
  semantic distinction wears a word, not a position. The explicit `ok` marker
  (exactly one variant, anywhere in the list) is greppable, order-independent, and
  removes the need for a separate `result enum` declaration modifier; the
  positional rule saved one keyword at the cost of a silent, position-encoded fact
  — the wrong trade under P13.

---

## 7. Explicitly deferred (with reasons)

Nothing here is designed, because each needs a decision this document must not
prejudge:

- **Generics and traits/interfaces (P11).** Deferred wholesale. Consequences that
  therefore also defer: **converting `?`** (cross-type error propagation needs
  `From`-style traits — §2.4); **`for` / iterators**; **user-defined containers**;
  **operator overloading**; and **the generic-bracket token choice** — which is
  *constrained now* to avoid `<>` so `>>` stays a shift (§3).
- **Module system, visibility, imports.** Deferred; the prototype is single-file
  (0001 §9). Signatures are already fully explicit, so module syntax adds no
  memory-model decision — it is a P20/packaging round.
- **FFI / boundary-module syntax (P17).** No foreign surface in the basket;
  spelling boundary modules waits on the P17 round.
- **The text-type budget beyond `[u8]` (P3's named obligation).** This document
  ships the universal view (`[u8]`); the owning/interop text forms are a P3
  design-doc decision, not a syntax one, and are not accreted here.
- **Mutable statics / concurrency primitives (P9/P10).** Deferred; a mutable
  global raises aliasing questions the single-threaded core must not answer by
  accident (0001 §8.2).
- **`audit` / `assumed-proven` contract-level spelling (P8).** Semantics fixed;
  the module/contract attachment syntax is a small later addition.
- **Async / coloring — not deferred, refused** (NN#9). Named so no future round
  reads its absence as an omission.
