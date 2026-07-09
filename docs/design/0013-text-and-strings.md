# 0013 — Text and strings: the P3 text-type budget

**Status:** draft
**Date:** 2026-07-09
**Philosophy hooks:** P3 (one canonical way — text is P3's oldest *named*
stress point, §4/Appendix A v2→v3), P9 (small never-allocating core; owning
collections in std, allocator-explicit), P6 (small core; the budget is fixed —
adding requires removing), P5/P7 (fault vs error at construction), P2/NN#17
(nothing crosses a signature by inference), P13/NN#13 (clarity-dense syntax; the
grammar parses without a symbol table), P17 (the foreign boundary; C-string
interop). Builds on 0006 §3.5/§8.3 (string literals produce `[u8]`; "there is no
string type in this edition"), 0008 §5 (core/std package layering), 0009
(`Iter`/`Indexed` iteration), 0011 §1 (C `char *` maps to `rawptr u8` +
`valid_nul_terminated` contract), 0012 §2.2 (`portable`).
**Prototype:** Stage 1 (`str` in core: `"..."`->`str` / `b"..."`->`[u8]` literals,
byte `len`/index/compare, boundary-faulting `substr`, `as_bytes`, `str_from`->`Utf8Res`,
`str_from_unchecked`, and `Indexed` byte iteration) and Stage 2 (`String` builder:
`string_new`/`push`/`append`/`as_str`) are implemented in the checker + tree-walk
oracle; `String` is compiler-known (§7 note), and char iteration / `char_count`
remain deferred as OBL-TEXT-CHARS. §7 is the stage plan. This document
discharges 0006 §7's deferral ("the text-type budget beyond `[u8]`") and spec
§8.3 / chapter 99's standing obligation **OBL-TEXT**.

## Problem

P3 flagged text at the language's founding as the design-time stress point that
produced Rust's `str` / `String` / `OsStr` / `CStr` sprawl, and set the mandate
verbatim: **"one universal view type in the core; the minimum owning/interop
forms above it, each existing only by recorded justification, not by
accumulation."** That mandate is this document's normative budget. It must be
resolved *now* because text is the gate to standard-library growth (collections,
formatting, OS services) and eventually self-hosting: nothing above the slice
layer can be built until the language decides what a string *is*.

Today's reality (spec §8.3): there is **no string type**. Text is `[u8]`
(borrowed) or `[N]u8` / `Box`-of-bytes (owned); string literals produce `[u8]`
over read-only static storage. That was a deliberate deferral, not a decision.
This document decides.

The collision P3 named is real: "one string type" meets P9's allocator-free
freestanding core. A view can live in the core (it allocates nothing); an owning,
growable string cannot (growth is allocation). So text is *inherently* at least
two things — a core view and a std owner — and the discipline is to admit exactly
those and refuse the rest under the recorded-justification test.

## Decision

### 1. The budget, stated first

Text is **two named types and one existing escape hatch**, plus interop carried
by a *contract*, not a type:

| Form | Layer | Owns? | Allocates? | Invariant | Justification |
|------|-------|-------|-----------|-----------|---------------|
| `str` | **core** | no (borrow) | never | **well-formed UTF-8** | §1.1 |
| `String` | **std** | yes | yes (allocator-explicit) | well-formed UTF-8 | §1.2 |
| `[u8]` | core (0006) | no (borrow) | never | none — raw bytes | §1.3 (pre-existing) |
| C strings | **boundary** | — | — | NUL-terminated (contract) | §1.4 — *no type* |

This is **tighter than the three-type budget P3's example sketches**: the interop
"type" is refused outright. The P3 example named "CStr-like NUL-terminated views
at the boundary" as a candidate third form; §1.4 argues it down to a contract on
existing types, because a distinct type there fails the recorded-justification
test. Two named text types (`str`, `String`) is the whole budget. Every text form
Rust ships and Candor does *not* is refused by name in the Rejected alternatives,
each against the same test: **what invariant does this type carry that `[u8]`
plus a contract cannot — and is that invariant worth a new entry in the fixed
budget (P6)?**

#### 1.1 `str` — the one universal core view (irreducible)

`str` is an **immutable, borrowed, non-owning, allocation-free view of a run of
bytes that are guaranteed to be well-formed UTF-8.** It is shaped exactly like
`[u8]` — a shared/`read`-borrow of a contiguous run (0006 §2.2: `[T]` is the
shared-slice sigil) — and is a *typed refinement* of it: `str` is to `[u8]` what
a validated view is to a raw one. It carries the same cost model as `[u8]`
(a pointer-and-length borrow, no allocation, aliasable, ultra-high-frequency —
0006 §8.4's "cheap, aliasable" case) and adds exactly one thing: the static
guarantee that the bytes decode as UTF-8. It lives in `core` (0008 §5), because a
view allocates nothing and validation is a pure function (§4). It is irreducible
because it is the *only* thing every text-consuming signature in the ecosystem
can name without demanding an allocator — the universal ground floor of text,
the direct analogue of P9's allocation-free core.

#### 1.2 `String` — the one owning, growable form (irreducible)

`String` is the **one owning, growable, allocator-parameterized** text type, in
`std`. It owns a heap run of UTF-8 bytes and carries its `Alloc` explicitly per
P9 (0008 §5; construction takes an `Alloc`, like every std collection). It exists
because a view (`str`) cannot *own* or *grow* text — building text (formatting,
concatenation, reading a line) requires an owner, and per P9 owning heap text is
std and allocator-explicit, never core. It is irreducible for the same reason
`Vec`-equivalents are: the alternative is forcing every text-builder to
hand-thread a `[u8]` buffer and its length, which is the boilerplate a growable
owner exists to remove, and which cannot be a view. One owning form, not two:
there is no `SmallString`, no `Cow`, no interned variant in the budget — each is
a std *library* optimization over `String`/`str`, added (if ever) by its own
recorded justification, not minted here.

The two names `str` and `String` are **deliberately Rust's**. Reusing them is a
P19/Bet-6 decision: models carry strong priors for exactly these two, and the
budget's entire win over Rust is refusing the *other* two names (`OsStr`,
`CStr`), not renaming the two that were correct.

#### 1.3 `[u8]` — the raw escape hatch (pre-existing, retained)

`[u8]` remains exactly what 0006/spec §8.3 already make it: a borrowed view of
raw bytes with **no encoding invariant**. It is the genuinely-binary escape
hatch and the honest home of *non-UTF-8 and non-text data*: a codec's input, a
device register window, a network frame, an OS path on a platform whose paths are
not UTF-8 (§Rejected — `OsStr`). `str` and `[u8]` convert explicitly (§3, no
implicit coercion — P2): `as_bytes(s: str) -> [u8]` is free (a retyping — the
UTF-8 view *is* a byte run); `str_from(b: [u8]) -> Result[...]` validates (§4).

#### 1.4 C-string interop — delegated to the boundary, no new type

0013 defines **no `CStr` type**. NUL-terminated C-string interop is already
resolved by 0011 §1: `char *` maps to `rawptr u8`, and "NUL-termination is a
**contract**, not a type" (0011's table), discharged by the boundary module's
`valid_nul_terminated(p)` trust predicate (0011 §7; the safe wrapper
`c_strlen(s: read [u8]) requires(len(s) > 0 && s[len(s)-1] == 0)` is 0011's own
example). Applying the recorded-justification test: a `CStr` type would carry the
invariant "these bytes are NUL-terminated and contain no interior NUL." That
invariant (a) is not a *text* invariant at all — it is an FFI memory-layout
invariant — and (b) is already expressible as a contract on `[u8]`/`rawptr u8` at
the P17 boundary, which is where trust is meant to be enumerable. Minting a
core/std type for it would duplicate 0011's mechanism and pull an FFI concern into
the text vocabulary. **The type is refused; delegated to 0011.**

**Honest scoping of what the delegation costs (review #1 finding 2).** The CStr
invariant has two parts, and they belong at *different* check levels — the earlier
draft's "buys nothing" overclaimed by routing both to trust. **NUL-termination**
of a foreign `char *` is genuinely uncheckable (the bytes past the pointer are not
Candor's) — correctly `assumed-proven` at the boundary. But **no-interior-NUL** of
*Candor-owned* bytes being passed *to* C is dynamically checkable (a scan), which
is exactly what Rust's `CStr::new` enforces at construction. That checkable half
must ride the Candor→C wrapper as `enforced requires(no_interior_nul(s))` (0011 §3
already places the checkable value-subset at `enforced`), **not** be lumped into
`assumed-proven`. So the refusal of the *type* stands — no `CStr` earns minting —
but the delegation is honest only if the checkable invariant stays `enforced`; a
`CStr` type would have enforced no-interior-NUL by construction, and the boundary
contract must match that, not weaken it. This is the single sharpest budget
decision, and it kills one whole arm of the Rust sprawl — at the stated,
not-zero, cost of keeping the checkable half `enforced` by discipline.

### 2. The core / std / boundary split

- **core:** `str` (the view), `[u8]` (raw bytes), and the pure validation and
  view-conversion functions over them (`str_from`, `as_bytes`, byte `len`, byte
  index, byte compare). Never allocates; present on freestanding targets. This is
  0008 §5's "core … formatting … 0001's `[u8]` universal view lives here,"
  extended with `str`'s validated view.
- **std:** `String` (owning, growable) and everything that builds or grows text
  (`push`, `append`, the owning side of formatting). Allocator-explicit; absent
  from freestanding targets by default (0008 §5). A freestanding program has
  `str` and `[u8]` and never `String` — and that is correct, because a
  freestanding program that needs to *build* text supplies its own buffer as
  `write [u8]` and validates it, exactly as it supplies its own allocator.
- **boundary:** C-string interop (§1.4), owned by 0011 / P17.

`str`'s relationship to `[u8]` is the load-bearing structural fact: **`str` is a
validated, typed view over the same run `[u8]` views.** The raw byte hatch
remains open for data that is genuinely not text; text that *is* text earns the
typed view. Two vocabularies, converted explicitly, is the honest shape — not
sprawl, because the second vocabulary (`[u8]`) already existed and carries a
different (empty) invariant on purpose.

### 3. Operations — the minimum viable surface

The budget applies to operations too. The minimal surface, all in core unless
marked:

- **Length is bytes.** `len(s: str) -> usize` is the **byte** length, O(1),
  identical to `len` on `[u8]`. There is no `len`-that-counts-characters. Counting
  Unicode scalar values is a separate, explicitly-named, O(n) operation
  `char_count(s: str) -> usize` — the UTF-8 tax made visible (P4); **ships WITH the
deferred `Chars` protocol (OBL-TEXT-CHARS), not in the minimal first surface**
(review #1 finding 7). The cheap
  length is bytes, and the operation that costs a scan *says so in its name and
  its cost*.
- **Indexing is byte-indexed.** `s[i]` yields the **byte** `u8` at `i`,
  bounds-faulting like any slice index (0001 §5 / spec §8.1). It does not yield a
  "character." A byte is always a valid `u8`, so byte indexing never faults on an
  encoding boundary — only on bounds.
- **Sub-slicing faults on a non-boundary.** `substr(s: str, a: usize, b: usize)
  -> str` (the `str` form of spec §8.2's bounds-checked `subslice`) returns the
  sub-view `[a, b)` and **faults (P5)** if `a` or `b` does not fall on a UTF-8
  character boundary. This is a fault, not an error value, because slicing text at
  a non-character boundary is a *bug* (P7: faults are bugs) — the program computed
  an offset that its own logic should have kept on a boundary. Code that wants
  byte-boundary-agnostic sub-runs takes `as_bytes(s)` and sub-slices the `[u8]`,
  which is only bounds-checked. The UTF-8 invariant is thus preserved by
  construction across every `str`→`str` operation, dynamically enforced exactly
  where staticity cannot reach.
- **Comparison is byte-wise.** `==` and ordering on `str` are byte-lexicographic
  (which, for well-formed UTF-8, coincides with code-point order — a free
  correctness property of the encoding choice, §4). No locale, no Unicode
  collation, no case folding — those are library concerns far outside the budget
  and are refused here (§Rejected).
- **Iteration yields bytes (ground floor); chars are deferred.** A `str` iterates
  as **bytes** via 0009's `Indexed` (`type Item = u8`, `at(read self, i) ->
  Opt[u8]`): non-`alloc`, interrupt-callable, the ground-floor protocol (0009
  §3.2). `for b in read s` yields `u8`. **Character iteration is not in the
  minimal surface** and is deferred as **OBL-TEXT-CHARS** — see §4 for why it fits
  neither of 0009's two protocols cleanly, which is precisely the UTF-8 tax
  landing on the iteration machinery. When it lands it is a *dedicated* `Chars`
  view yielding `u32` code points, not an overload of byte iteration.
- **Literal syntax.** `"..."` denotes a **`str`** (the migration, §Literals
  below); `b"..."` denotes a `[u8]` (new syntax, §Literals, cleared under NN#13).
- **Building (std).** `String` grows by
  `push(write self, c: u32) enforced requires(is_scalar_value(c))` — appends one
  Unicode scalar value, UTF-8-encoded. **The `requires` is load-bearing, not
  decorative** (review #1 finding 1): without it, `push(s, 0xD800)` (a surrogate)
  or `push(s, 0x110000)` (out of range) would UTF-8-encode to ill-formed bytes and
  forge a false `str` through `as_str` — breaking, at the primary builder, the one
  invariant that is `str`'s reason to exist. This is where the char-as-`u32`
  decision (§Rejected) pays its cost: an `enforced` P5 backstop on the single
  `u32`→text door. A caller decoding from a P7 boundary validates there and the
  check is then provably redundant (P8 static discharge); the backstop guarantees
  no unvalidated integer becomes text. `append(write self, s: read str)` appends a
  view (its bytes are already well-formed by `str`'s invariant — no check).
  `as_str(read String) -> str` borrows the built text back as a view. This
  is the owning side of formatting; the full format machinery is a later std
  round built on `append`.

#### Literals and the migration (NN#13)

- **`"..."` becomes `str`.** A double-quoted literal (0006 §3.5 / spec §3.5:
  `STRING = '"' { char | escape } '"'`, escapes `\" \\ \n \t`) now has type
  `str`, not `[u8]`. Source files are UTF-8, so a literal's bytes are well-formed
  UTF-8 **by construction**; the compiler validates at compile time, so a literal
  → `str` is **infallible and carries zero runtime cost** — the validation of §4
  is discharged statically. This is the whole migration for text that is text.
- **`b"..."` becomes `[u8]`** — a new byte-string literal for genuinely-binary
  data, so raw bytes have a spelling now that `"..."` is `str`. **NN#13
  clearance:** `b"..."` is decidable from the character stream alone — a `b`
  *immediately* followed by `"` opens a byte-string literal; a `b` followed by
  anything else is an identifier. This is one character of lookahead, maximal-munch
  clean (spec §01 6.x), and needs no symbol table. Models carry the `b"..."` prior
  from Rust. It is the only new grammar this document introduces; the budget
  applies to syntax too, and `str` / `String` are plain identifiers that add none.
- **Migration totality (P15).** The corpus has exactly one string-literal site
  today: `let s: [u8] = "5+3+9"` in the parser fixture (0001 §11.4). The migrator
  rewrites literal sites by their annotated/inferred *use*: a site used as bytes
  becomes `b"5+3+9"` (or the binding's `[u8]` type forces `b"..."`); a site used
  as text becomes `str`. The parser consumes its input as bytes (`peek`/`advance`
  over `u8`), so that site migrates to `b"5+3+9"` (or keeps `[u8]` via
  `as_bytes`). Mechanical and total — the rewrite is a local, per-site retyping.

### 4. The encoding decision and its consequences

**`str` guarantees well-formed UTF-8, validated at construction.** This is the
central call. The alternative — `str` as a byte view with UTF-8 *operations*
layered on top, carrying no invariant — is **refused**, because a `str` that
guarantees nothing over `[u8]` *is* `[u8]`: it fails the recorded-justification
test on contact (§1 / §Rejected). The type's entire reason to exist is the
invariant; remove the invariant and you have deleted the type, not simplified it.

Consequences, each named:

- **Construction from arbitrary bytes is a P7 error, not a P5 fault.**
  `str_from(b: [u8]) -> Result[str, Utf8Error]` returns an error value on invalid
  UTF-8. This is deliberate and is the P5-vs-P7 decision: invalid UTF-8 arriving
  from a network socket, a file, or a foreign buffer is an **expected** outcome of
  parsing untrusted input, not a bug in the parser — so it is an error value in
  the signature (P7), propagated with `?`, never a fault. `Utf8Error` carries the
  byte offset of the first invalid sequence (P4: diagnosable). Contrast §3's
  `substr` fault: slicing *already-valid* text off a boundary is a bug; validating
  *unknown* bytes is a routine fallible operation. The two are different questions
  and get P5 and P7 respectively.
- **The freestanding story needs no allocator.** `str` is a view and validation
  is a **pure, allocation-free function** (a forward scan of the byte run
  against the UTF-8 grammar), so all of `str` lives in core and runs on a 64KB
  microcontroller. UTF-8 costs freestanding code *nothing it did not ask for*,
  because the escape hatch is the point: **non-UTF-8 and raw data stay `[u8]` and
  are never `str`.** A driver reading a non-UTF-8 device string keeps `[u8]` and
  never validates; the systems-reality objection ("text may be non-UTF-8 or raw")
  is answered not by weakening `str` but by *not calling that data text* — it is
  bytes, correctly typed `[u8]`. This is exactly how the encoding guarantee avoids
  the OS-path sprawl (§Rejected — `OsStr`).
- **An unchecked construction exists, and is a visible trust hole.**
  `str_from_unchecked(b: [u8]) -> str` skips validation and is an **`unsafe`**
  operation carrying a mandatory justification string (P1), for the case where
  the caller has already proven UTF-8 (e.g. it just built the bytes from other
  `str`s). Greppable exactly like every other trust declaration (P17 register).
  Its wrong use yields a `str` whose invariant is false — a bug, not UB, since no
  optimizer assumes the invariant (P8's rule holds: the UTF-8 guarantee is a type
  invariant checked at construction, never a fact the optimizer builds on; a
  false `str` produces wrong *values* — e.g. a mis-sliced view — not undefined
  behavior). NN#1 stands without an asterisk.
- **Char iteration cost is visible (P4), and its protocol mismatch is the tax.**
  UTF-8 code points are variable-width (1–4 bytes). This breaks *both* of 0009's
  protocols: `Indexed`'s `for` desugar steps the index by a fixed 1, which would
  land mid-character; and `Iter`'s `next` is `alloc`-marked at the interface (0009
  §3.1, forced by `List`), which would drag an allocation marker onto a
  non-allocating `str` scan and make character iteration *not* interrupt-callable.
  Neither is acceptable, so char iteration is **deferred** (OBL-TEXT-CHARS) rather
  than forced into a mis-fitting protocol. It needs either 0009's deferred
  a small dedicated non-`alloc` `Chars`
  interface; that is a 0009 decision, taken when char iteration has a caller. The
  honest present answer: **bytes iterate freely; characters wait for the right
  protocol.**

### 5. Interaction with the memory model and generics

- **`str` is a borrow-kind, so §3.4 binds it.** Like `[T]`/`[u8]`, `str` is a
  borrow, and 0001 §3.4 (no borrow-typed struct fields) applies: **a `str` may
  not be a struct or enum field.** A struct that owns text stores a `String`
  (owning) or `[N]u8` / `Box`-of-bytes; a struct that *borrows* text stores a
  handle/index or is refactored to hold the owner. This is not an exception carved
  for text — it is `str` inheriting the slice rule, and it is a second, independent
  reason `String` is irreducible: the field case has no view answer.
- **`str` is `portable` (0012); `String` is not, by the same rule as `Box`.**
  0012 §2.2 makes a type `portable` iff it transitively contains no `rawptr` and
  no borrow; a *borrow value* crosses to a scoped task iff its **referent** is
  portable (0012's borrow branch). `str`'s referent is a run of `u8` — portable —
  so a `str` shares into a `scope`-spawned task exactly like any `[u8]`, no new
  rule. `String` carries an `Alloc` handle, which carries a `rawptr` (the `Alloc`
  flagship of 0012 §intro), so `String` is **not** `portable` — identical to every
  other allocator-bearing owned value, and honestly so. No text-specific
  concurrency machinery is added; text rides the existing `portable` walk.
- **`str` is concrete, not `Text[Encoding]` — encoding-genericity refused.**
  `str` is a concrete type, not a generic over an encoding parameter. The refusal
  is a budget decision with four supports: (a) **P3/P6** — the mandate is *one*
  view, and a `Text[Enc]` family is the accumulation P3 forbids; (b) **P13/P2** —
  an encoding parameter would put `[Utf8]` on the single highest-frequency type in
  the language, taxing every text signature with annotation weight the value-first
  bet spends elsewhere, and would cross signatures (a caller must name the
  encoding — NN#17 friction) for a distinction almost no code varies; (c)
  **0007/P11** — encoding-genericity buys coherence and monomorphization cost
  (a `str` method per encoding, checked and instantiated per encoding) for **zero
  corpus demand**: no basket program (allocator, scheduler, MMIO driver, parser,
  arena — the adversarial Bet 5 basket) wants multi-encoding text; (d) **the real
  need is served without it** — when a program genuinely handles UTF-16 or
  Latin-1, that data is `[u8]` (or `[u16]`) and an explicit codec function decodes
  it *into* `str`; the encoding is a *decision at a conversion point* (visible,
  greppable, P5's "meaning in the source"), not a type parameter threaded through
  every signature. One encoding in the type; every other encoding is bytes plus an
  explicit codec.

## Rejected alternatives

- **Rust's `str` / `String` / `OsStr` / `CStr` (+`Path`) sprawl — kept two,
  refused the rest.** `str` and `String` are kept (correct, and model-prior-rich).
  Each refusal meets the recorded-justification test:
  - **`OsStr`/`OsString` — refused.** Its invariant is "platform-native OS
    encoding, possibly not UTF-8." In Candor that data is **`[u8]`** (raw bytes,
    §1.3) — the escape hatch already carries the empty invariant that "possibly not
    UTF-8" needs, and a path API takes `[u8]` and validates to `str` only when it
    must display text. No core/std type earns minting for "bytes we won't promise
    are UTF-8"; that is what `[u8]` *is*.
  - **`CStr`/`CString` — refused, delegated to 0011 (§1.4).** Its invariant
    (NUL-terminated, no interior NUL) is an FFI memory-layout fact, already a P8
    contract on `[u8]`/`rawptr u8` at the P17 boundary, where trust is enumerable.
    A type would duplicate 0011's mechanism and pull FFI into the text vocabulary.
  - **`Path`/`PathBuf` — out of budget here.** A path is a std *library* newtype
    over `str`/`String`/`[u8]` with path-manipulation methods; if it is ever
    minted it is a std API decision with its own justification, not a *text* type,
    and it does not enter this budget.
  The sprawl came not from `str`+`String` (those were right) but from every *other*
  invariant getting its own type; Candor routes those invariants to `[u8]` +
  contract and mints no type for them. That is the budget's win.
- **`str` as an unvalidated byte-view-with-operations (no UTF-8 guarantee) —
  refused.** It carries no invariant over `[u8]`, so it *is* `[u8]` under a second
  name — an accumulation with no justification, and a silent invitation to the
  bug UTF-8-validity exists to prevent (§4). The Rust lesson here is the *positive*
  one: `str`'s UTF-8 guarantee was correct; do not walk it back.
- **Validation as a fault (P5) at construction — refused for the byte-input
  case.** Invalid UTF-8 from external input is expected, not a bug; a fault would
  halt a program for correctly detecting bad network data. Error value (P7) at
  construction; fault (P5) only for `substr` off a boundary, which *is* a bug (§4).
- **A distinct `char` scalar type — refused; a code point is a `u32`.** A `char`
  type would carry the invariant "a Unicode scalar value" (`0..=0x10FFFF` minus
  surrogates). That invariant (a) is not needed by most code, which treats code
  points as integers, and (b) is expressible as a P8 contract exactly where it
  *is* needed — a decoder's `ensures(is_scalar_value(result))`, an encoder's
  `enforced requires(is_scalar_value(c))`. Minting a scalar type spends the budget
  (P6 — a new primitive) to move an invariant from a greppable contract into the
  type system, for information a `u32` already carries and that models already
  model as an integer. **Cost named and accepted:** without a `char` type, a `u32`
  holding a code point *could* hold a non-scalar value unless contracted; we accept
  this, placing the invariant on the operations that produce/consume code points.
  This is the most contestable refusal (§Consequences).
- **Mutable `str` views (`write str`, in-place UTF-8 mutation) — refused.** A
  length-changing edit (a 1-byte character replaced by a 3-byte one) resizes the
  run, which a view cannot do; and same-width in-place mutation is a rare,
  sharp-edged operation that can silently break the UTF-8 invariant mid-write.
  Mutation is `String`'s job (owning, growable) or, for the genuine in-place case,
  `write [u8]` inside a validate-on-exit boundary. No `str_mut` in the budget.
- **Encoding-generic `Text[Encoding]` — refused (§5): P3/P6 accumulation, P13/P2
  signature tax, 0007 coherence cost, zero corpus demand; codecs serve the real
  need.**
- **Interning / `Cow` / `SmallString` as budget types — refused.** Each is a
  performance specialization over `str`/`String`, added (if ever) as a std library
  type with its own recorded justification — not a member of the *text budget*,
  which fixes only the universal view and the universal owner.

## Consequences and costs

- **The UTF-8 tax is real and deliberately visible.** Byte length is O(1);
  character count is O(n) and named `char_count` so the cost shows (P4). Byte
  slicing is free; `str` sub-slicing can **fault** on a non-boundary offset — a
  new fault site systems programmers must keep offsets honest around, accepted as
  the price of the invariant. This is a genuine cost, not a managed one: code that
  computes text offsets by byte arithmetic must land them on boundaries or take
  the `[u8]` path deliberately.
- **Character iteration is absent until OBL-TEXT-CHARS.** A program that must walk
  code points today walks bytes and decodes by hand, or waits. This is an honest
  gap driven by the 0009 protocol mismatch (§4), not an oversight; it is on the
  ledger.
- **`str` cannot be a struct field (§5).** Structs that hold text hold `String`
  or byte arrays. Mild friction, consistent with the value-first model and the
  slice rule; the reviewer sees ownership of text explicitly, which is the §3.4
  intent.
- **Two vocabularies, converted explicitly.** `str` for text, `[u8]` for bytes,
  no implicit coercion (P2). Callers write `as_bytes` / `str_from` at the seam.
  This is the honest boundary between "text" and "data," not sprawl — but it *is*
  friction, paid at every conversion, and it is the cost of not pretending bytes
  are always text.
- **No `CStr` type means the NUL invariant lives in a contract, auditably but
  fallibly.** A caller can pass a non-NUL-terminated `[u8]` to a C function by
  writing the wrong boundary contract; the `enforced`/`assumed-proven` level and
  the P17 boundary-module list are where that is caught and enumerated. This debt
  is 0011's / P17's, named here so the delegation is not mistaken for a guarantee.
- **The `char`-as-`u32` refusal is the most contestable call.** If real text code
  turns out to reach for code points constantly and the missing scalar invariant
  becomes a recurring bug source, this is the decision to revisit first — by
  minting `char` under P6's add-requires-remove, or by a std code-point wrapper.
  Recorded as the leading revisit trigger.

## Prototype stage plan (§7)

Honest about what lands, staged so each stage is independently testable:

- **Stage 1 — `str` in core (checker + interp).** Add `str` as a validated `[u8]`
  view: a borrow-kind, §3.4-banned as a field, `portable` via its `u8` referent
  (0012). Retype string literals `"..."` from `[u8]` to `str`, validated at
  compile time (infallible). Add `b"..."` byte-string literals → `[u8]` (NN#13
  lexer change). Core ops: `str_from(b) -> Result[str, Utf8Error]`,
  `str_from_unchecked` (unsafe), `as_bytes`, byte `len`, byte index `s[i]:u8`,
  `substr` (char-boundary-faulting), byte `==`/ordering, `Indexed` byte iteration.
  Migrate the one corpus literal site.
- **Stage 2 — `String` in std.** `String` as a std type generic over `Alloc`
  (0008 §5), `push(u32)` / `append(str)` / `as_str`. Depends on 0007/0009 being in
  place (it is) for the allocator-as-parameter idiom.
- **Deferred (on the ledger):** OBL-TEXT-CHARS (`Chars` view, `char_count`),
  the full formatting machinery over `append`, and any `Path`/`Cow`/interning std
  types — none in the text budget.

What will *not* land in the prototype: character iteration, locale/collation
anything, and any interop type (interop is 0011's contract, already shipped).

## Reclassification record

None. This document turns on no §2 reclassification: the byte-length /
char-count cost split, the char-boundary fault, and the two-vocabulary seam are
all priced under Priority 4 (predictable cost) and P4 (visible cost) directly,
not by promoting an ergonomics concern into a verifiability one.
