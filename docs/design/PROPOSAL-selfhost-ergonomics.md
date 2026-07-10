# PROPOSAL — Self-hosting ergonomics: the first design pressure from dogfooding

**Status: PROPOSAL — NOT-YET-A-DESIGN.** This is an *input to a decision*, not a
decision. It changes no rule, discharges no obligation, and amends no document.
It collects the ergonomic evidence the self-hosting arc has produced so far,
sorts it into essential (a paid-for cost of soundness) versus accidental
(fixable without touching a Non-Negotiable), and lays out candidate responses
with their philosophy cost so the deciding authority can choose. Where a
candidate reopens a settled design, this document **presents the trade and stops**
— it does not pick.

**Date:** 2026-07-10
**Evidence source:** `docs/spec/99-obligations.md` OBL-SELFHOST-ERGO (the six
frictions from the lexer slice), read against the self-hosted lexer
`prototype/selfhost/lexer/lexer.cnr` and the corelib
(`prototype/tests/fixtures/corelib/`). More frictions will accrue from the parser
slice; this works from what is recorded.
**Philosophy hooks:** **P19** (dogfooding — self-hosting as the forcing function),
**P6** (small fixed core; add-requires-remove), **P9** (allocator-explicit;
owning collections in std, never core), **P3** (one canonical way), **P12/Bet 5**
(value-first, borrows are a gear for passing not storing), **P2/NN#17** (nothing
crosses a signature by inference). Subordinate to `LANG_PHYLOSOPHY.md` and to
designs 0001, 0007, 0009, 0012, 0013. Where they conflict, the higher document
wins; this proposal has no authority to override any of them.

**What this is.** The P19 forcing function producing its first design pressure:
concrete, counted evidence that several deferred questions are now real, framed
as options.

**What this is not.** Not an enactment, not a design, not a review disposition.
No file but this one is touched.

---

## 1. The evidence, organized

OBL-SELFHOST-ERGO records six frictions from writing one real Candor program (a
lexer for the `.cnr` surface, gated by token-stream equality against the Rust
oracle). Each is stated below with its **root** (the design decision that causes
it), **how often it bit** (counted against `lexer.cnr`, 26 functions / 435
lines), and its **classification**: *essential* (a real safety cost the
philosophy accepts — removing it would weaken a Non-Negotiable) or *accidental*
(a convenience gap fixable in std or the toolchain with no Non-Negotiable
touched). The discipline this project runs on is refusing accidental sprawl while
paying essential costs honestly; the whole value of the sort is getting each
friction on the correct side of that line.

### The taxonomy at a glance

| # | Friction | Root (design decision) | Bit how often | Class |
|---|----------|------------------------|---------------|-------|
| 1 | View cannot be a struct field; source is threaded | **0001 §3.4** no-borrow-fields | **15 of 26 fns** thread `src: [u8]` | **Essential** |
| 2 | No growable `Vec`; fixed `[1024]Tok` + count, hard cap | No std growable collection yet (P9 shape undesigned) | 1 structural cap (`Buf.toks`), silent overflow risk | **Accidental** |
| 3 | No owned text without an `Alloc`; manual byte itoa | P9 allocator-explicit **+** no std formatting over `String` | 1 hand-rolled `emit_num` (19 lines), whole dump is a `trace` byte drip | **Mixed** (essential half + accidental half) |
| 4 | No match-on-byte-span / no map; keyword ladder | No std map + no keyword-set construct | **56** `span_eq` branches in `classify_ident`/`suffix_code` | **Accidental** |
| 5 | `out` is a hard keyword the self-lexer must dodge | `out` reserved as a mode keyword (0006) | Pervasive-but-trivial (any `out` identifier is unusable) | **Accidental** (essential *cause*) |
| 6 | `buf.*.f[i]=` write path vs `buf.f` read path asymmetry | Reborrow/deref surface (0005/0006 `.*`) | 2 write derefs (`push_tok`) vs bare reads (`dump`) | **Accidental** |

**Counts: one essential, one mixed, four accidental** — with the caveat that the
mixed friction (3) splits into an essential half (must thread an `Alloc`) and an
accidental half (must hand-roll formatting), and that friction 1's *classification*
as essential is exactly the question candidate C reopens. If the authority takes
candidate C, friction 1 moves; until then it is the paradigm essential cost.

### 1.1 Source-threading — friction (1), the biggest structural tax

**Root: 0001 §3.4, the no-borrow-fields rule.** A `[u8]` (and, per 0013 §5, a
`str`) is a borrow, and a borrow may not be a struct or enum field. So there is no
`struct Lexer { src: [u8], pos: usize }` holding the input alongside the cursor;
the input is a bare `[u8]` parameter threaded through **15 of the lexer's 26
functions** — `span_eq`, `classify_ident`, `suffix_code`, `scan_ident`,
`scan_number`, `scan_string`, `scan_op`, `scan_token`, `skip_trivia`, `lex`,
`emit_bytes`, `decoded_len`, `emit_decoded`, `emit_tok`, `dump`. Every helper that
touches a byte re-declares `src` in its signature and re-passes it at every call.

**This is essential complexity — the price of soundness, working as intended.**
0001 §3.4 calls itself "the single most consequential decision in the document,"
and its *decisive* reason is measurement honesty: a borrow field would need
lifetime-parameterized types (a region parameter on `Lexer` and on every signature
naming it), and hiding an inter-object reference behind a type lifetime would make
the valve **invisible** and cause the Bet-5 metric to understate how often systems
code reaches for pointers. Source-threading is the *visible cost* that keeps the
pointer-safety accounting honest — it is Rust's `struct Lexer<'a> { src: &'a [u8] }`
lifetime parameter, spelled instead as an explicit parameter, which is precisely
the machinery Candor's value-first bet traded away. It is not a defect. It is the
bet being paid. (Candidate C asks whether the bill is too large; §2 argues both
sides.)

### 1.2 No growable collection — friction (2)

**Root: no std growable collection exists yet.** `Buf.toks` is `[1024]Tok` with a
companion `n: usize`; `push_tok` writes `toks[n]` and increments. The cap is a
**real defect** — a source file with more than 1024 tokens silently corrupts (the
array index would fault at 1024, but the design intent, a growable buffer, is
absent). This is **accidental**: 0013 and 0007 already name `Vec`-equivalents as
the exact thing a growable owner exists to remove, P9 already fixes their shape
(std, allocator-explicit), and nothing in the memory model forbids one. The lexer
uses a fixed array only because the std collection has not been built.

### 1.3 No owned text without an `Alloc` — friction (3), the mixed one

**Root: two decisions, one essential and one accidental.** The lexer renders its
token dump as a *byte drip* through the built-in `trace` sink — `emit_num`
(lines 341–359) is a hand-rolled `itoa`, dividing by ten into a `[24]u8` scratch
buffer and tracing digits in reverse. There is no `String` in play because the
harness gave the lexer no `Alloc` handle.

- **Essential half:** per P9 (0013 §1.2), building owned text needs an allocator,
  threaded explicitly. `String` is std and allocator-parameterized *on purpose*;
  a freestanding renderer that will not carry an `Alloc` correctly has no
  `String`. That the lexer must either thread an `Alloc` or drip bytes is the P9
  cost, and it is honest.
- **Accidental half:** even *with* an `Alloc`, the digit loop is hand-rolled
  because there is no std formatting layer over 0013's `String::append`. 0013 §3
  ships `push`/`append`/`as_str` and explicitly parks "the full format machinery"
  as "a later std round built on `append`." `emit_num` is that missing round,
  written by hand.

### 1.4 The keyword ladder — friction (4)

**Root: no std map and no byte-span match construct.** `classify_ident` and
`suffix_code` are a linear ladder of **56 `if span_eq(src, s, e, b"…")` branches**
— one per keyword, scalar type, and integer suffix. You cannot `match` on a
`[u8]` span (patterns match values and variants, not byte runs — 0002), and there
is no dictionary to look a spelling up in. This is **accidental**: the keyword set
is compile-time-known (a perfect-hash or length-bucketed `match` could be
generated), and a general runtime dictionary is a std collection gated only on a
`Hash` interface (0007 already uses `[K: Hash, V]` as its multi-bound example).
Neither needs a Non-Negotiable touched.

### 1.5 The `out` keyword collision — friction (5)

**Root: `out` is a reserved mode keyword (0006).** A program self-lexing Candor
cannot use `out` as an identifier. The *cause* is essential — `out` is a needed
parameter mode and reserving it is correct — but the *friction* is accidental and
trivial: it is the ordinary cost of any reserved word, and it bites exactly one
identifier name. It belongs on the ledger, not in a redesign.

### 1.6 The deref asymmetry — friction (6)

**Root: the reborrow/deref surface (0005/0006 `.*`).** The write path spells
`buf.*.toks[i] = t` and `buf.*.n = i + 1` (`push_tok`, lines 312–313); the read
path spells the bare `buf.n` / `buf.toks[i]` (`dump`, lines 429–431). Writing
through a `write Buf` needs the `.*`; reading through a `read Buf` does not, and
the author must remember which. **Accidental** — a diagnostics/formatter concern
(OBL-FMT-REBORROW is already the toolchain home for normalizing reborrow
spelling), not a language defect.

---

## 2. The candidate responses, each with its philosophy cost

### A — A std `Vec[T]` (growable collection)

**What it needs.** A std generic `struct Vec[T] { buf: rawptr T, len: usize, cap:
usize }` over an explicit `Alloc` (0007 §6.3's "every future container becomes a
library generic"; P9's allocator-explicit rule). Growth is reallocation: the
current `AllocVtable` (`std/alloc.cnr`) carries `alloc` and `free` but **no
`realloc`**, so `Vec` grows either by adding a `realloc` fn-pointer to the vtable
or by alloc-new + copy + free through the existing two. Either is a **std
addition, not a language change** — no new effect (the ops are `alloc`-marked like
every allocator call), no memory-model change, no new syntax.

**Does it earn its slot?** Yes. The fixed `[1024]Tok` cap is a genuine defect
(§1.2), `Vec` is the canonical remedy, and 0013 already names the "hand-thread a
buffer and its length" boilerplate — exactly `Buf.toks`/`Buf.n` — as the thing a
growable owner exists to remove. **P6 is not threatened:** P6 fixes the *core*
budget, and `Vec` is std, where owning collections are supposed to live (P9). The
only open sub-question is the vtable shape (`realloc` slot vs. alloc-copy-free),
which is a std API call, not a philosophy call.

**Philosophy cost:** near zero. One std generic type + one vtable decision.

### B — A std `Map` / dictionary

**The keyword ladder is the presenting symptom, but it is the wrong patient.** The
56-branch ladder classifies a *compile-time-known, closed* keyword set. The
right fix for *that* is not a runtime hash map — it is a generated perfect hash or
a length-bucketed `match`, which is codegen or a std frozen-set, carrying no
runtime hashing at all. A **general runtime `Map[K, V]`** is a separate, real
need (symbol tables in the parser slice will want it), and it is a **std library
type, not language support**: it needs a `Hash` interface (0007's own
`[K: Hash, V]` example) and the same allocator-explicit machinery as `Vec`. No
language feature is required for either the ladder or the map.

**Argument for "language support" — and its rejection.** One could imagine a
`match` that pattern-matches byte-string literals (`match spelling { b"fn" => …
}`). That *is* a language change (a new pattern form) and it is **refused here on
P3/P6 grounds**: it is a second construct for "classify a value" that a std map or
generated hash already covers, and it puts byte-run matching — a narrow need — into
the core pattern grammar. The ladder's real fix is std + codegen.

**Philosophy cost:** zero for the map (std, gated on `Hash`); the byte-match
language form is refused, not deferred.

### C — Region-bearing struct fields (the hard one)

This is the friction-(1) question, and it is the **only candidate that reopens a
settled, Non-Negotiable-adjacent design.** Allowing a view in a struct field needs
the region-parameterized-types machinery that 0007 §3.5 and 0012 §2.2 deliberately
declined — a region parameter on the storing type, variance, and well-formedness,
threaded across every signature naming the type. It is coupled to **OBL-ITER-BORROW**
(0009 §3.4), whose acceptance criterion is *exactly* "chapter 10 §2.3's
region-parameterized-type question reopened, or a region-free borrowed-yield model
found." The same reopening would answer both. **This proposal argues both sides
and decides neither.**

**The case FOR reopening (source-threading is a defect to fix).**
- The tax is measured and large: 15 of 26 functions carry `src` for no reason the
  reader cares about; it is signature noise on three-quarters of the program.
- It compounds. The parser slice will thread `src` *and* a token buffer *and* an
  arena through a still-deeper call tree; the friction grows super-linearly with
  the size of the self-hosted compiler, which is the whole remaining arc.
- OBL-ITER-BORROW is already an independent debt demanding the same machinery. If
  region-parameterized types must be built for borrowed iteration anyway, building
  them once answers both the iterator-borrow question and the view-field question,
  and the amortized cost drops.
- It is the difference between "a borrow is a gear for passing" and "a borrow is a
  gear you may also *hold for a bounded scope*," which is what a lexer *is*.

**The case AGAINST reopening (source-threading is the philosophy working).**
- 0001 §3.4 names measurement honesty as its *decisive* reason: a borrow field
  hides a valve behind a type lifetime and makes the Bet-5 pointer metric lie.
  Source-threading keeps the reference visible. Removing it re-hides exactly what
  Bet 5 exists to measure.
- The machinery is the machinery the value-first bet was *placed against*. Region
  parameters on types, variance, and well-formedness are "a fraction of a second
  borrow system" (0009 §5.1's phrase for the adjacent closure question); adopting
  them is not a patch, it is switching bets. Rust's lifetime-parameter structs
  *are* the complexity Candor traded away — this friction is that trade being felt,
  which is evidence the trade is real, not evidence it was wrong.
- The friction has a value-gear answer already in the idiom: a struct that needs to
  associate a cursor with input holds an *owner* (`String`/`[N]u8`/`Box`-of-bytes)
  or a *handle/index*, and threads the view as a parameter — 0001 §3.4's sanctioned
  (a)/(b)/(c) choices. Source-threading is (c-adjacent): the reference travels as a
  parameter, visibly.
- The pressure is, so far, *one* program's *ergonomics* (a Priority-7 concern). The
  bar the project set for reopening a refusal (0007 §1.1, 0009 §1.3) is a
  **basket-grade measured** case that the friction is the *dominant* reading-cost or
  an expressiveness *failure* — not an annoyance. One lexer threading `src` is
  evidence-gathering, not yet that bar.

**What the authority must weigh:** whether the coupled reopening (view-fields +
OBL-ITER-BORROW) has now crossed the basket-grade bar, or whether source-threading
should be **ratified as an accepted essential cost** and the evidence held open
until the parser slice reports. This is an amendment-scale memory-model decision.
This proposal lays out the trade and stops.

### D — Owned text without a threaded `Alloc` (the itoa friction)

**Split the friction along its §1.3 seam.**
- The **essential half** — you must thread an `Alloc` to build owned text — is the
  P9 cost and is **accepted, not fixed.** A renderer that will hold no allocator
  drips bytes; that is correct P9 behavior, and the honest answer is "thread the
  `Alloc`," identical to every std collection.
- The **accidental half** — even with `String`, `emit_num` is hand-rolled — is a
  **std formatting round over 0013 §3's `append`**, which 0013 *already names as a
  later std round*. Shipping integer/format helpers over `String::append` deletes
  the hand-rolled itoa with no language cost and no new text-budget entry (P3's
  budget is untouched; formatting is library gear over `String`).

**Philosophy cost:** zero language cost; a std formatting design, plus the standing
P9 discipline of threading the allocator.

### E — The `out` collision and the deref asymmetry (minor ergonomics)

- **`out` collision:** **accept and record.** `out` is a needed mode keyword;
  reserving it is correct, and no program gets to name a variable `out` for the
  same reason none gets to name one `fn`. This is a ledger entry, not a change.
- **Deref asymmetry:** **toolchain, not language.** The `buf.*.f` vs `buf.f`
  split is a diagnostics-and-formatter concern, adjacent to OBL-FMT-REBORROW's
  existing mandate to normalize reborrow spelling. A type-aware formatter and a
  sharp "did you mean `buf.*`?" diagnostic address it; the language stays as is.

---

## 3. Recommendation framing

Each candidate is sorted into one of four categories. The categories are the
authority's menu; the reopen-scale item is **presented, not decided.**

| Candidate | Recommended category | One-line rationale |
|-----------|----------------------|--------------------|
| **A — std `Vec[T]`** | **Ship as std** (no philosophy cost) | Fixes a real defect (fixed cap); std + allocator-explicit satisfies P9; P6 untouched (core budget unaffected). One open std sub-question: `realloc` slot vs. alloc-copy-free. |
| **B — std `Map` + keyword classify** | **Ship as std** (map) / **refuse** (byte-match language form) | Runtime `Map` is a std type gated on `Hash`; the ladder's real fix is a generated perfect hash / frozen set. A byte-string `match` pattern is refused on P3/P6. |
| **C — Region-bearing struct fields** | **REOPEN a design — authority's amendment-scale call** | Reopens region-parameterized types (0007 §3.5 / 0012 §2.2), coupled to OBL-ITER-BORROW. Both sides argued in §2C; **not decided here.** |
| **D — Owned text / itoa** | **Ship as std** (formatting round) + **accept essential cost** (thread the `Alloc`) | Formatting over 0013 `append` is an already-named std round; the P9 allocator-threading is the accepted honest cost. |
| **E — `out` collision** | **Accept as essential cost — record and move on** | A needed keyword's ordinary reservation cost. |
| **E — deref asymmetry** | **Defer to toolchain** (formatter/diagnostic) | Adjacent to OBL-FMT-REBORROW; no language change. |

**The one that genuinely needs the deciding authority's amendment-scale
decision is candidate C** — region-bearing struct fields, framed as a binary:
either **reopen** the region-parameterized-types question (discharging the
view-field friction *and* OBL-ITER-BORROW together), or **ratify** source-threading
as an accepted essential cost and hold the evidence open until the parser slice
reports. Everything else is std work (A, B-map, D-formatting), an accepted cost
(D-`Alloc`, E-`out`), a toolchain concern (E-deref), or a refusal (B-byte-match).
No other friction rises to amendment scale.

**A note on sequencing, not a recommendation:** candidates A, B, and D are the
std-collections/formatting round the text and generics designs already anticipate
(0013 §3, 0007 §6.3, 0009 §9), and could proceed independently of the C decision.
C should not be rushed to clear a Priority-7 friction; the basket-grade bar the
project set for reopening refusals is the right gate, and the parser slice is the
next evidence toward it.

---

## 4. The meta-point: this is the forcing function working

Self-hosting is the P19/dogfooding mechanism the philosophy built *precisely to
produce this document.* The critical path (MEMORY.md: Bet 5 prototype → semantic
core → toolchain → breadth) runs on measured pressure, not speculation, and the
whole discipline of 0007/0009 — refuse a feature until a *basket-grade measured*
case exists, then admit the *smallest* form that clears it — depends on a source
of real measured cases. The self-hosted lexer is that source, doing its job.

Read correctly, OBL-SELFHOST-ERGO is **good news**: five of six frictions land as
std or toolchain work with no philosophy cost, which means the language's core did
not buckle under the first real program — it held, and pushed the growth outward to
the library and tooling layers exactly where P9 and P6 intend growth to go. The one
friction that reaches the core (source-threading) reaches a decision the project
*already knew it was deferring* (0001 §3.4's named cost, OBL-ITER-BORROW's open
question) — the arc did not surprise the design, it *dated* it, turning a deferred
abstraction into a concrete, counted bill the authority can now weigh.

That is evidence-gathering working as designed. A friction surfacing is not the
language failing; it is the forcing function forcing. This proposal is the first
such force made legible — six counted costs, five of them cheap, one of them the
memory-model question the value-first bet always knew it would eventually have to
re-answer under real load. The decision on that one is the authority's; the
evidence for it is now on the table.

---

*This document is a PROPOSAL. It enacts nothing. It amends no obligation, design,
or spec. Its only output is the framing above, offered to the deciding authority.*

---

## Deciding-authority disposition (2026-07-10)

Delegated to the orchestrator with instruction to gather more evidence first.

- **Candidate C (region-bearing struct fields) — HELD, not decided.** One self-hosted module is
  thin evidence to reopen region-parameterized types (the priority order's most expensive
  deferred feature; P11 spent its refusals avoiding exactly this surface). The philosophy's own
  rule — a refusal reopens only when pressure cases genuinely exist — says wait. The parser slice
  (larger, more struct-heavy, running now) is the next evidence point: it either sharpens the case
  for B (reopen) or confirms source-threading is a tolerable essential cost (ratify A). Decision
  deferred to after the parser slice's ergonomics land; if it then remains genuinely balanced, a
  dedicated high-effort deliberation is warranted before any amendment.
- **Candidates A, B, D, E (accidental / std / toolchain) — cleared to proceed as evidenced.**
  std Vec, a std hash map, std formatting, and formatter/diagnostic polish are outward growth with
  no Non-Negotiable cost; they ship when scheduled, gated behind the currently-running prototype
  work to avoid source conflicts. std Vec's one open sub-question (a `realloc` vtable slot vs.
  alloc-copy-free growth) is resolved at build time, defaulting to alloc-copy-free unless the
  Alloc-vtable change proves cheap — it needs no authority call.

## Candidate C — RULED (A), high-effort deliberation, 2026-07-10

**Ruling: ratify 0001 §3.4 (no borrow fields). Source-threading is an accepted essential cost,
not re-openable without new evidence of the named trigger shape below.** Decided by a high-effort
deliberation (Fable-5, fresh context) the deciding authority authorized for this amendment-scale
call; full argument in the review record. The decisive findings:

- **The pressure case got WEAKER at 6.5× scale.** The parser slice records source-threading under
  "what worked cleanly" (threaded via 0005's implicit reborrow), not among its seven frictions.
  The reopen bar (a basket-grade case where the friction is the *dominant* reading cost or an
  expressiveness *failure*) is met by neither: the friction is wide-but-shallow (2-3 uniform
  tokens/signature, zero ambiguity, provenance made visible = P2/P13 value), and its single narrow
  root is "one pass-context struct per program cannot hold its input." Every recursive data
  structure (Tok, Node) chose spans and u32 arena indices with no complaint — the index-arena is
  natural Candor style, and borrow fields would not fix the arena idiom anyway (a self-referential
  tree can't be built from borrows).
- **B is not containable.** Region-parameterized fields = the second borrow system minus only
  higher-ranked regions: region params on type decls (spelled at every use per NN#17), variance,
  outlives/subtyping (the `P` struct's own first use needs TWO provenances → conflation), a region
  solver replacing 0001 §2.3's one-pass liveness (killing the P20 compile-speed foundation), a
  dropck well via 0007, and re-blinding the frozen Bet 5 valve accounting — against the binding
  v4.2 re-measurement commitment. P11 ("reject expressive growth by default") + priority order
  (item 5 Simplicity over item 7 Ergonomics) resolves it.
- **B's three-for-one bundle dissolves.** OBL-ITER-BORROW's own criterion is disjunctive (a
  region-FREE borrowed-yield model suffices); OBL-TEXT-CHARS is dischargeable by a value-gear
  decoder (next_char threading pos, the lexer's own idiom); the threading friction was already
  cheaply absorbed by 0005. One-for-one at full price, the other two purchasable near-zero.

**Named re-open trigger (the only thing that reopens this):** a future self-hosting slice (e.g.
the checker: symbol table + source + tokens + AST) whose pass-context tuple grows to 4-5 threaded
views on the *majority* of signatures. Today it is 2, and clean.

**Consequential dispositions:** OBL-ITER-BORROW → route to its region-free branch. OBL-TEXT-CHARS
→ value-gear decoder design. OBL-TEXT-RESULT (transient borrow-in-payload) → left OPEN as a
separate, strictly-smaller question, NOT foreclosed by this ruling. OBL-SELFHOST-ERGO friction #1
→ recorded as accepted essential cost.
