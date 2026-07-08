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
Two corpus facts drive most of the compaction: **678 dereferences (345 of them
the exact idiom `(deref x).field`)** and **~300 explicit reborrows** — both pure
plumbing, both bucket 2.

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
addition is a declaration *modifier* for result-shaped enums, `result enum`
(§2.5, for `?`).

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

### 2.3 Parameter modes

**Unchanged in spirit (P12), one contextual fix.** Modes stay `take` (omitted) /
`read` / `write` / `out`; omission = `take` *is* the value-first bet made visible
in every signature and is not negotiable here. All four are bucket-1 words. The
only change: **`out` becomes a contextual keyword** (reserved only in parameter-
mode position and as a leading argument marker, an ordinary identifier
elsewhere), fixing the recorded friction that `let out = fold(…)` was rejected
(arena note 1). This mirrors how `alloc` is already contextual (0002 §0.4) and
costs nothing at NN#13 — the position, not the identifier's meaning, decides.

### 2.4 Expressions

**Dereference — postfix `.*`, retiring prefix `deref`.** This is the highest-
value single change in the document. The corpus has **678 derefs, 345 of them
the exact shape `(deref x).field`**; the prefix keyword plus its mandatory
parentheses is bucket-2 plumbing wrapping nearly every field touch through a
borrow. Chain depth is *shallow* — essentially always one deref before a field
or index, never a deep `deref deref deref` tower — which is precisely why a
*sigil* suffices and nothing heavier is warranted.

| Throwaway | Real |
|---|---|
| `(deref s).base` | `s.*.base` |
| `deref pos = t.end` | `pos.* = t.end` |
| `(deref ar).mem[i]` | `ar.*.mem[i]` |
| `deref b` (value) | `b.*` |

`.*` is postfix, binds tighter than field access and indexing, and reads as "the
pointee." It is chosen over postfix `^` (which would steal the char needed for
bitwise-xor) and over C's `->` (see Rejected). **Auto-deref (`s.base`) is
rejected** and is the most contestable call in the document — argued in §6.

**Borrow operators — `read` / `write`, unchanged.** They mark the birth of a
loan and stay words (bucket 1). Call-site *reborrow* ceremony is already gone
per design 0005: a held borrow passed bare to a `read`/`write` parameter
reborrows; the ~300 `read (deref b)` / `write (deref b)` sites collapse to `b`.
Fresh borrows of owned storage keep the keyword (`f(write x)`) — that keyword
marks a real aliasing event and earns its tokens (0005).

**Field projection on `rawptr` — `field_ptr(p, f)`, unchanged (design 0004).**
Safe, un-gated, field-selector position; retires the `cast_ptr∘ptr_offset∘
offsetof` incantation as the canonical way to take a field address.

**Conversion — keep `conv`, drop the required parens; reject `as`.**
`conv T (e)` (34 sites, "verbose" per the READMEs) becomes `conv T e` where `e`
is a postfix expression (parens only when the operand needs them):
`conv usize i`, `conv u8 x.*.f`. The keyword *stays* — this is a deliberate
refusal of `e as T`:

- A conversion is a *semantic event*: under 0001 §8.1 a narrowing/sign-changing
  `conv` **faults on value loss** by default (truncates only inside `wrapping`,
  saturates inside `saturating`). It must wear a word (P13: the dangerous thing
  is loud).
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
`saturating`. **Precedence is given explicitly to avoid C's famous `&`-below-`==`
trap:** shifts bind above additive; `&`, `^`, `|` (in that order) bind *above*
the comparison operators, not below — so `a & mask == 0` parses as
`(a & mask) == 0`. Logical `&&`/`||` stay lowest. `&`/`&&` and `|`/`||` are
distinguished by maximal munch, as today.

**ADDED — an `i64::MIN`-expressible literal.** Throwaway `-9223372036854775808`
*faults* (unary minus is arithmetic over an already-overflowing magnitude;
arena note 2 calls it "a real trap: the program checks clean and faults at the
literal"). Fix: **unary minus applied directly to an integer literal is a
compile-time-folded literal constant**, range-checked against its type at compile
time — so `-9223372036854775808` (or `-9223372036854775808i64`) is a valid `i64`
and an out-of-range literal is a *compile error*, never a runtime fault.
`- x` for a variable `x` stays an ordinary (faulting) arithmetic negation. For
the programmatic bound, add compile-time intrinsics **`min_of(T)` / `max_of(T)`**
to the `sizeof`/`alignof` family (greppable, NN#13-clean). An `i64::MIN`
associated-constant spelling is rejected: `::` is reserved exclusively for enum
variants (NN#13), and reopening it is not worth one constant.

**ADDED — `?` propagation (P7's blessed operator), scoped honestly.** The parser
pays a two-arm `match` per fallible construction and hoists `must_box`/
`must_box_args` helpers purely to avoid it (parser note 5: "box ceremony without
`?`"). P7 explicitly blesses a lightweight propagation operator; ship it, but
scope it to what the *no-generics* core can carry:

- **Spelling.** Postfix `expr?`. On a value of a **result-shaped enum**, if it is
  the success variant, the expression evaluates to the unwrapped payload; if any
  other variant, the enclosing function `return`s that value unchanged.
- **Result-shaped enum.** An enum declared `result enum E { ok(T), … }`: the
  **first** variant is the success/unwrap variant, every other variant
  propagates. The `result` modifier is the greppable flag an auditor or model
  keys on; the parser only needs to see `result enum`, and the checker resolves
  the rest (NN#13-clean — no symbol table at parse).
- **The honest limit — same-type only, conversion deferred.** `expr?` is legal
  only where the enclosing function returns the *same* enum `E`. Cross-type
  propagation (a callee's `E1` error flowing into a caller's `E2`) needs an
  error-conversion mechanism (`From`-style), which needs traits — **P11, deferred
  (§7)**. The same-type form already erases the parser's `must_box` matches
  (`box(a, v)?`) and every self-recursive fallible call, which is the measured
  cost. Recorded soft spot: "first variant is success" is positional rather than
  marked; an explicit `ok`-variant marker is the alternative if the positional
  rule reads badly at review (§6).

### 2.5 Control flow

**`match` — drop the per-arm `case`.** Throwaway `match e { case P => x, … }`
becomes `match e { P => x, … }`. `case` is pure bucket-2 boilerplate — **198
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
  as a constraint on the future round.
- **`[T]` slice vs `[N]T` array vs `[…]` literal vs `a[i]` index: position +
  content.** Type position vs expression position is grammatical (0002 §3). Within
  a type, after `[` parse one component; if a Type follows the `]` it is an array
  `[N]T`, else it is a slice `[T]` — the component (a const-expr size vs a Type)
  and the trailing Type are lexically distinguishable (scalar-type keywords and
  the type grammar decide, no symbol table).
- **`?` propagation: pure postfix.** No ternary `?:` exists (Candor uses `if`
  expressions), so `?` is unambiguously the postfix propagation operator. Result-
  shape is a *checker* fact; the parser sees only `expr?`.
- **`->` stays solely the return arrow.** It was *not* taken for deref (see §6),
  so it never appears in expression position and never competes with anything.
- **`conv T e`, `~a`, `min_of(T)`: keyword/position-led**, exactly like the
  existing `conv`, `!`, and `sizeof` families.

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
  explicit reborrow `write (deref b)` / `read (deref b)` in a matching mode-
  argument position → bare `b` (design 0005, type-aware — it must *not* rewrite a
  `take`-mode borrow-typed parameter or a non-place operand); the throwaway
  `deref`/`case`/`slice`/`borrow` spellings → their §2 real forms; redundant
  parens around a `conv` primary operand → dropped.

---

## 5. Migration (P15 — mechanical, by tool)

The prototype and the six ports migrate by tool when the real parser exists. The
throwaway→real map is a syntactic rewrite with no behavioral change (P15 strict
sense), except the two rows marked *author-assisted*:

| Throwaway | Real | Mechanical? |
|---|---|---|
| `borrow T` / `borrow_mut T` | `read T` / `write T` | yes |
| `slice T` / `slice_mut T` | `[T]` / `write [T]` | yes |
| `(deref b).f` / `deref b` / `deref b = v` | `b.*.f` / `b.*` / `b.* = v` | yes |
| `read (deref b)` / `write (deref b)` (arg pos) | `b` | yes (0005) |
| `case P => e` | `P => e` | yes |
| `conv T (e)` | `conv T e` | yes |
| `-MAX - 1` (MIN construction) | `-9223372036854775808` | yes |
| wrapper structs `Buf`/`Occ`/`Arena` for array params | `write [N]T` params | yes (unwrap) |
| xorshift 64-iteration bit loop | `^` / `<<` / `>>` | **author-assisted** (semantic rewrite) |
| `must_box`/two-arm `BoxResult` match | `box(a, v)?` | **author-assisted** (same-type `?` only) |

The scheduler-v2 re-port (`ports/candor/scheduler-v2`) is already authored in the
0004/0005 evolved forms and is the reference for the mechanical rows.

---

## 6. Rejected alternatives (§8.6 — the load-bearing sigil-vs-keyword calls)

- **`&T` / `&mut T` for borrow types — rejected.** Would positionally overload
  `&` between borrow-type and the newly-added bitwise-and, and add a second
  rendering of "exclusive borrow" beside the `write` operator (P3). `read T` /
  `write T` reuses the existing keyword and frees `&` entirely for arithmetic
  (§2.2). The decisive call of the document.
- **Auto-deref (`s.base` for `s.*.base`) — rejected; the most contestable call.**
  It would take all 678 derefs to zero tokens and is memory-safe on checker-
  tracked borrows (design 0004's "gives the pointer meaning" objection applies to
  *rawptr*, not safe borrows). Rejected anyway because P13 prices dereference as
  *load-bearing semantics* (the reader must know whether a name is a value or a
  borrow to reason about aliasing and the `.* =` write path), and the corpus
  shows chains are shallow — so a one-token sigil captures the whole cost while
  keeping the borrow boundary visible. If review finds `.*.` still too noisy at
  the 345 field sites, auto-deref-on-read-only (keeping `.* =` explicit for
  writes) is the recorded fallback.
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
- **Keeping per-arm `case` — rejected.** 198 sites of bucket-2 boilerplate.
- **`i64::MIN` associated constant — rejected.** Reopens `::` (NN#13). Negative-
  literal folding + `min_of`/`max_of` intrinsics cover it.
- **Per-expression `unsafe` modifier — rejected** (0001 §10.3): scatters the
  audit surface; the block keeps justification attached to a region.
- **Positional "first variant is `ok`" vs an explicit `ok` marker for `result
  enum` — chosen positional, recorded.** The marker is the fallback if the
  positional rule reads badly; kept minimal (no per-variant keyword) for now.

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
