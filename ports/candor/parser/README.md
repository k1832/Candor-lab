# Candor port: recursive-descent expression parser

Port of `docs/basket/spec-parser.md` (frozen) to Candor, per
`BET5_CRITERION.md` §2.4(d) — the deliberately value-favorable workload
(design 0001 §11.4). One file, `parser.cn`: lexer, parser, owned AST,
canonical S-expression serializer, and the full frozen vector suite (P1–P32)
below the `// Test harness` marker (ruling R14). `main` returns the sentinel
**777** on full success; any vector failure faults.

Toolchain: `prototype/target/release/candor-proto check|run ports/candor/parser/parser.cn`.
Suite runtime: ~0.03 s. The implementation section above the marker was
verified to parse and check standalone.

## Architecture (for the adjudicator)

- **Lexer**: a pure `lex(s, from) -> Tok` over the borrowed input `slice u8`;
  `Tok` is a `copy` struct `{kind, start, end}`. Peeking is calling `lex`
  without advancing; consuming is `deref pos = t.end`. Two-character
  operators matched maximally; a byte no token starts with is `TK_ERR` at its
  offset (surfaced as `E_UNEXPECTED_CHAR` wherever that token is demanded —
  P27's rule applied uniformly).
- **Parser**: recursive descent. The six binary tiers (spec 2.2) are one
  level-indexed pair `parse_bin`/`bin_tail` (0 `||` … 5 `* / %`); **left
  associativity is accumulator-passing tail recursion** in `bin_tail`
  (likewise `call_tail` for chained calls, `args_rest` for argument lists).
  Unary is right-associative by direct recursion. Errors are values:
  `PR::err(kind, offset)`, first error only, rulings R10/R11/R12 honored.
- **AST**: `enum Ast` with `Box` children (owned, per 0001 §11.4); call
  argument lists are an owned cons list `enum Args { nil, cons(Box Ast,
  Box Args) }` (no `Vec`/generics in the prototype). Every node carries its
  byte span; **leaves carry only `(start, end)`** — token text is never
  copied (spec §1.4 zero-copy; adjudicator inspection point: `Ast::lit`/
  `Ast::name` payloads are two `usize` offsets, nothing else).
- **Serializer**: consumes the owned AST (the §11.4 evaluator's owned-match +
  `unbox` idiom), emitting leaf text via `subslice(s, st, en)` — the output
  bytes *are* the span-selected input bytes, so the harness's byte-equality
  assertions are simultaneously the §3.5 span/substring check for every leaf
  (P31). Root spans (P1, P8) and P18's exact leaf spans are asserted
  directly in the harness.
- **Allocator**: `Box` construction requires an `Alloc` handle (0001 §6.1),
  so the implementation section carries a bump allocator (same shape as the
  in-tree §11.4 fixture): `Bump {next, end}` state over a dedicated address
  range, no-op `free`, `static BUMP_VT`, handle built once in `mk_alloc`.

## Valve shape (Bet 5 data)

**Exactly as design 0001 §11.4 predicts: the parser proper has zero valves.**
The port's only `unsafe` blocks are the two in the bump-allocator
infrastructure:

- `bump_alloc` — arena state read/update and block carve (`ptr_read`/
  `ptr_write`/`addr_to_ptr`).
- `mk_alloc` — `addr_of_mut`/`addr_of` building the `Alloc` handle, carrying
  the §6.1 outlives-every-copy-and-every-Box justification.

Lexer, parser, AST construction, error reporting, serializer, and the whole
harness are pure value + borrow gear.

## Language friction notes (Bet 5 data)

1. **A consuming visitor must `return` from every match arm (E0302).** A
   `match` over an owned Box-bearing enum whose arms move payloads out cannot
   fall through to the match join — arms that move and arms that don't leave
   inconsistent partial-move states (§1.6 rule 1), *even when nothing is used
   afterwards and the function returns `unit`*. First draft of `ser`/
   `leaf_spans` wrote plain visitor arms and got four E0302s; the fix is a
   `return;` at the end of every arm (turning expression arms into block
   arms). Mechanical, but it means the natural visitor shape is illegal.

2. **Per-call-site reborrow ceremony.** Every recursive-descent call passes
   the cursor as `write (deref pos)` — ~25 occurrences. The known 0001 §2.1
   cost; in this program it is the single largest source of reading noise.
   (Signature-site annotations, which M1 counts, stay modest: one mode
   keyword per parameter.)

3. **Left-assoc folds are tail recursion, not loops, by checker constraint.**
   The natural spelling — `loop` + `match` + a rebound `Box` accumulator —
   was designed around preemptively because of the allocator port's finding
   that a `match` lexically inside a loop trips the E0304 false positive.
   The accumulator-passing helpers (`bin_tail`, `call_tail`, `args_rest`)
   read fine, but the shape was dictated, not chosen.

4. **No generics means a hand-rolled list per element type.** Call arguments
   need `enum Args` with its own cons cells and its own `must_box_args`
   helper — one extra allocation per argument plus a boxed `nil` terminator.
   A second recursive walk (`arg_leaf_spans`) is likewise duplicated per
   list type.

5. **`box` ceremony without `?`.** `box` returns `BoxResult`, so every node
   construction pays a two-arm match; `must_box`/`must_box_args` centralize
   it (arena exhaustion is a `panic` — spec §4 has no OOM kind and the
   harness arena is sized so it cannot occur). The prototype's recorded
   omission of `?`-propagation is what this costs here.

6. **Positive findings** (the value-favorable case behaving as bet): owned
   `Box` AST + borrowed `slice u8` input needed no region annotations
   anywhere (nothing borrowed is returned); moving both children into
   `Ast::bin` at the box call site is exactly the §11.4 pattern and checked
   first try; `copy struct Tok` makes the lexer interface friction-free.

## Spec ambiguities hit (flagged for adjudication)

- **`f(` at end of input** (no vector; R10 arguably adjacent): the port
  reports `E_UNEXPECTED_EOF` at `len`, because after `(` the grammar demands
  *an argument expression or `)`* — the same answer bare `(` at EOF gives —
  reserving `E_EXPECTED_RPAREN` for R10/R11's "expression already parsed, `)`
  (or `,`) demanded" positions (`f(1` → `E_EXPECTED_RPAREN` at `len`).
- **Lex error where `)` is demanded** (e.g. `(1 @`, `f(1 @`): reported as
  `E_UNEXPECTED_CHAR` at the bad byte, generalizing P27's
  lex-error-surfaces-when-demanded rule over R10/R11's wrong-token rule
  (a `TK_ERR` byte is not a "valid-but-wrong token" in R11's sense).
