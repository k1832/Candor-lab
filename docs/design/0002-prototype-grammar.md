# 0002 — Prototype Grammar (Bet 5 Validation Prototype)

**Status:** draft
**Date:** 2026-07-06
**Philosophy hooks:** NN#13 (the grammar parses without semantic context / a
symbol table), P4 (diagnostics as structured data with provenance), P13
(clarity-dense syntax; keywords where meaning lives). Subordinate to
`LANG_PHYLOSOPHY.md` and to design `0001-memory-model.md`, which this document
serves and does not amend.

## Problem

Design 0001 specifies the memory model of the throwaway-syntax prototype in
prose and worked examples (§11). Stage 1 of the prototype (lexer + parser)
needs a single, complete, mechanically-checkable grammar so that every construct
0001 names is representable and every §11 example parses. This document is that
grammar, plus the token inventory, the argument that it is symbol-table-free
(NN#13), and a record of every place 0001's example code was grammatically
inconsistent (with the ruling made — 0001 itself is left unchanged).

The grammar is a **fixture**, not a language proposal (0001 §0). Keywords are
ugly-but-unambiguous; the §11 examples are the ground truth the grammar is fit
to.

## Decision

### 0. Grammar-level decisions (the questions 0001 left to Stage 1)

1. **Integer-literal typing.** An integer literal is `i64` **by default** and
   carries an *optional* type suffix naming any integer scalar type, written
   with no separator: `42`, `42u8`, `0x40`, `4096u32`. The suffix must be one
   of `i8 i16 i32 i64 isize u8 u16 u32 u64 usize` (integer types only — `bool`
   and `unit` are not literal suffixes). Hexadecimal (`0x…`) and decimal forms
   are accepted. Rationale: local inference forbids "typed purely by context";
   a concrete default plus an explicit suffix keeps every literal's type
   readable at the literal (P2/P13) and is the smallest rule that parses the
   §11 examples (which mix bare `1`, `0x00`, and `0`).

2. **Wildcard patterns.** `_` **is** a pattern, usable as a whole match arm
   pattern and as a payload sub-pattern. 0001 requires matches to be
   *exhaustive* (checker-enforced, §8.2), which is orthogonal to whether `_`
   exists; `_` is the greppable way to name "a payload I do not bind" (§8.2.1
   speaks of "payloads a pattern does not name"). Including it costs the parser
   nothing and does not weaken exhaustiveness (the checker still decides). No
   §11 example uses it; a parser unit test covers it.

3. **Statement termination.** Simple statements (`let`, assignment, and
   expression statements including `return`/`break`/`continue`/`assert`/
   `panic`) end with `;`. **Block-like** expressions used in statement position
   (`if`, `match`, `loop`, `while`, `unsafe`, `wrapping`, `saturating`, and a
   bare `{ … }` block) may stand as a statement **with or without** a trailing
   `;`. This single rule accepts both §11.3 (`match s { … };` with a semicolon)
   and §11.4/§11.5 (`match … { … }` as a function body, no semicolon).

4. **`alloc` is a contextual keyword, not a reserved word.** Design 0001 §6.1
   names a struct field `alloc` (`AllocVtable { alloc: fn(...) alloc -> ... }`)
   *and* uses `alloc` as the effect marker. To keep both, `alloc` lexes as an
   ordinary identifier; the parser recognizes it as the effect marker only in
   the one position it can appear — immediately after a function/`fn`-pointer
   parameter list, before `requires`/`ensures`/`->`/the body. Everywhere else
   `alloc` is an identifier (a field name, a variable). This is positional
   disambiguation, exactly the NN#13 discipline (see §"Symbol-table-free").

5. **Intrinsic split.** Only the intrinsics whose syntax is *not* an ordinary
   call are keywords: the bracketed-type-arg forms `cast_ptr[U](…)`,
   `addr_to_ptr[T](…)`, `ptr_null[T]()`, and the compile-time forms
   `offsetof(Type, field)`, `sizeof(Type)`, `alignof(Type)`. Every other
   builtin (`ptr_read`, `ptr_write`, `ptr_offset`, `is_null`, `ptr_to_addr`,
   `addr_of`, `addr_of_mut`, `box`, `unbox`, `slice_of`, `subslice`, `len`)
   is spelled as a normal `name(args)` call and parses as a `Call` whose callee
   is an ordinary identifier — the checker recognizes the builtin name later.
   This keeps the parser lean and NN#13-clean: `box(a, v)` parses identically
   whether or not `box` is special.

6. **`return`/`break`/`continue` are expressions.** They produce a value of the
   never/unit kind (the checker decides). Making them expressions is what lets
   a match arm body be `case BoxResult::oom => return BoxResult::oom` (§11.4)
   without a special "arm-jump" production.

7. **Struct-literal ambiguity.** In the head of `if`, `while`, and `match`, a
   bare `Ident {` is **not** a struct literal (it is the start of the block);
   struct literals there must be parenthesized. This is the standard Rust
   no-struct-literal restriction and is what lets `match ev { … }` parse (`ev`
   is the scrutinee, `{` opens the arms). Inside `(…)`, `[…]`, call arguments,
   and index brackets the restriction is lifted.

8. **Grammar-introduced spellings (0001 names the construct, not the syntax):**
   - **Drop hook placement:** `struct N { fields } drop(write self) { … }` — the
     hook trails the struct body. 0001 §1.5 names the hook `drop(write self)`
     but gives no placement; trailing it keeps it greppable and outside the
     field list.
   - **Top-level values:** `static NAME: Type = Expr;`. 0001 §6.1/§11.1 refer to
     `POOL_VTABLE`/`MY_VTABLE` as "top-level values" without a declaration
     syntax; `static` is the minimal representable form. (`::` stays reserved
     for variants, so `static` does not collide with anything.)
     **Statics are immutable** (soundness review #3, 2026-07-07): the checker
     rejects assignment to, `write`-borrow of, or `out`-passing of a static
     (E0311); reading and `read`-borrowing stay legal. A mutable global is a
     recorded future design question (concurrency, P9), not an accident of the
     prototype.
   - **`borrow T` / `borrow_mut T` type keywords** (0001 §2.1 names them) are
     accepted in type position for completeness even though §11 never spells a
     borrow *type* (borrows are introduced by `read`/`write` modes and
     operators). They never appear in the fixtures.

### 1. Token inventory

Whitespace separates tokens and is otherwise insignificant. Comments are
`// … <eol>` (line) and `/* … */` (block, non-nesting).

**Literals**
```
INT     = ( DEC | HEX ) [ INT_SUFFIX ]
DEC     = digit { digit }
HEX     = "0" ("x"|"X") hexdigit { hexdigit }
INT_SUFFIX = "i8"|"i16"|"i32"|"i64"|"isize"|"u8"|"u16"|"u32"|"u64"|"usize"
STRING  = '"' { char | escape } '"'          escape = \" \\ \n \t
IDENT   = (alpha | "_") { alpha | digit | "_" }     -- "_" alone is the wildcard
```
Integer literals default to `i64`; a suffix overrides. `bool`/`unit` are not
suffixes.

**Scalar-type keywords** (also lexed distinctly from identifiers)
```
i8 i16 i32 i64 isize   u8 u16 u32 u64 usize   bool unit
```

**Hard keywords**
```
slice  slice_mut  rawptr  Box  BoxResult  borrow  borrow_mut
struct enum fn static copy drop self
take read write out
let mut if else match case loop while break continue return
requires ensures assert panic result
unsafe wrapping saturating
deref clone conv
cast_ptr addr_to_ptr ptr_null offsetof sizeof alignof
true false
```

**Contextual keyword** (lexes as IDENT, recognized positionally): `alloc`.

**Punctuation / operators**
```
( ) { } [ ]   ,  .  ;  :  ::  ->  =>
=  ==  !=  <  <=  >  >=
+  -  *  /  %   &&  ||  !
```
`::` is reserved **exclusively** for enum-variant reference (0001 §0/§8.2).

### 2. Grammar (EBNF)

Notation: `{ X }` zero-or-more, `[ X ]` optional, `|` alternation, `( )`
grouping, `"lit"` terminal, `UPPER` token class.

#### 2.1 Program & items
```
Program   = { Item } EOF
Item      = [ "copy" ] ( Struct | Enum ) | Fn | Static
          -- "copy" may precede only Struct or Enum

Struct    = "struct" IDENT "{" [ FieldList ] "}" [ DropHook ]
FieldList = Field { "," Field } [ "," ]
Field     = IDENT ":" Type
DropHook  = "drop" "(" "write" "self" ")" Block

Enum      = "enum" IDENT "{" [ VariantList ] "}"
VariantList = Variant { "," Variant } [ "," ]
Variant   = IDENT [ "(" TypeList ")" ]
TypeList  = Type { "," Type } [ "," ]

Fn        = "fn" IDENT [ Regions ] "(" [ ParamList ] ")" SigTail
            [ "->" RetTy ] Block
Regions   = "[" IDENT { "," IDENT } [ "," ] "]"
ParamList = Param { "," Param } [ "," ]
Param     = IDENT ":" Mode Type
Mode      = [ "take" | "out" | ( "read" | "write" ) [ "[" IDENT "]" ] ]
            -- omitted => take; the [IDENT] region tag is a borrow region
SigTail   = { "alloc" | "requires" "(" Expr ")" | "ensures" "(" Expr ")" }
RetTy     = [ ( "read" | "write" ) [ "[" IDENT "]" ] ] Type

Static    = "static" IDENT ":" Type "=" Expr ";"
```

#### 2.2 Types
```
Type   = ScalarKw
       | "slice" Type | "slice_mut" Type | "rawptr" Type
       | "Box" Type   | "BoxResult" Type
       | "borrow" Type | "borrow_mut" Type
       | "[" ArraySize "]" Type            -- fixed array [N]T
       | FnPtrType
       | IDENT                             -- named struct/enum (unresolved)
ArraySize = INT | IDENT                    -- a const (literal or const name)
FnPtrType = "fn" "(" [ FnPtrParamList ] ")" [ "alloc" ] "->" Type
FnPtrParamList = FnPtrParam { "," FnPtrParam } [ "," ]
FnPtrParam     = [ IDENT ":" ] Mode Type   -- name optional (decorative)
```

#### 2.3 Statements
```
Block  = "{" { Stmt } "}"
Stmt   = Let
       | Expr "=" Expr ";"                 -- assignment (LHS is a place)
       | BlockLikeExpr [ ";" ]             -- block-like expr statement
       | Expr ";"                          -- simple expr statement
Let    = "let" [ "mut" ] IDENT [ ":" Type ] [ "=" Expr ] ";"
BlockLikeExpr = Block | If | Match | Loop | While | Unsafe
              | "wrapping" Block | "saturating" Block
```
(In practice the parser parses one `Expr`, then dispatches on a following `=`
for assignment, an optional `;` for block-like expressions, or a required `;`
otherwise.)

#### 2.4 Expressions (lowest to highest precedence)
```
Expr    = Or
Or      = And   { "||" And }
And     = Cmp   { "&&" Cmp }
Cmp     = Add   { ("=="|"!="|"<"|"<="|">"|">=") Add }
Add     = Mul   { ("+"|"-") Mul }
Mul     = Prefix { ("*"|"/"|"%") Prefix }
Prefix  = ("-"|"!") Prefix
        | ("deref"|"read"|"write"|"clone") Prefix
        | "conv" Type "(" Expr ")"
        | Postfix
Postfix = Primary { "(" [ ArgList ] ")"      -- call
                  | "[" Expr "]"             -- index
                  | "." IDENT }              -- field
ArgList = Arg { "," Arg } [ "," ]
Arg     = [ "out" ] Expr                     -- `out place` marks an out-mode argument (§3.1)
```
Prefix keyword-operators bind **looser** than postfix, so `read (deref ar).mem[i]`
is `read( ((deref ar).mem)[i] )`; the §11 examples parenthesize `(deref x)`
accordingly.

```
Primary = INT | STRING | "true" | "false" | "self"
        | "result" | "break" | "continue"
        | "return" [ Expr ]
        | "assert" "(" Expr ")" | "panic" "(" Expr ")"
        | "(" Expr ")"
        | ArrayLit
        | Block | If | Match | Loop | While | Unsafe
        | "wrapping" Block | "saturating" Block
        | TypeArgIntrinsic | Offsetof | Sizeof | Alignof
        | "BoxResult" "::" IDENT [ "(" [ ArgList ] ")" ]   -- compiler-known enum
        | IDENT "::" IDENT [ "(" [ ArgList ] ")" ]          -- EnumCtor
        | IDENT "{" [ FieldInitList ] "}"                   -- StructLit (if allowed)
        | IDENT                                             -- variable / callee

ArrayLit = "[" "]" | "[" Expr ";" Expr "]" | "[" Expr { "," Expr } [ "," ] "]"
FieldInitList = FieldInit { "," FieldInit } [ "," ]
FieldInit     = IDENT ":" Expr

TypeArgIntrinsic = ("cast_ptr"|"addr_to_ptr"|"ptr_null") "[" Type "]"
                   "(" [ ArgList ] ")"
Offsetof = "offsetof" "(" Type "," IDENT ")"
Sizeof   = "sizeof" "(" Type ")"
Alignof  = "alignof" "(" Type ")"

If     = "if" ExprNoStruct Block [ "else" ( If | Block ) ]
Match  = "match" ExprNoStruct "{" { Arm } "}"
Arm    = "case" Pattern "=>" Expr [ "," ]     -- separator optional (esp. after a block arm)
Loop   = "loop" Block
While  = "while" ExprNoStruct Block
Unsafe = "unsafe" STRING Block                -- justification string mandatory
```
`ExprNoStruct` is `Expr` parsed with the "bare `Ident {` is not a struct
literal" restriction (§0.7). The restriction is lifted inside any `(…)`,
`[…]`, call argument list, or index.

#### 2.5 Patterns
```
Pattern = "_"                                          -- wildcard
        | IDENT                                        -- binding
        | ( IDENT | "BoxResult" ) "::" IDENT [ "(" [ PatList ] ")" ]  -- variant
PatList = Pattern { "," Pattern } [ "," ]
```
Payload sub-patterns compose (0001 §8.2.1): a variant payload may itself be a
binding, `_`, or a nested variant pattern. Binding *modes* (move vs borrow) are
determined by the checker from how the scrutinee is held (§8.2.1); the parser
carries only the pattern shape, exactly as the task requires.

### 3. Symbol-table-free parsing (NN#13)

The grammar is LL-parseable by recursive descent with **at most two tokens of
lookahead** and **no identifier resolution**:

- **`::` is exclusively variant reference.** `IDENT :: IDENT` is always an
  `EnumCtor`/variant pattern by position; the parser never asks whether the
  left identifier is an enum. This is decided on `peek()==IDENT &&
  peek(1)=="::"`, no table.
- **Type vs expression position is grammatical, never resolved.** A type only
  appears after `:`, after a parametric type keyword, inside `conv`/`offsetof`/
  `sizeof`/`alignof`/`cast_ptr[…]` brackets, in a return, or in a param — all
  keyword/position-determined. `[` starts an array *type* only in type position
  and an array *literal* only in expression position; the two never compete.
- **Struct literal vs block** is resolved by the `ExprNoStruct` context flag
  (§0.7), a purely syntactic bit, not a lookup.
- **Intrinsics with special syntax are keywords**; every other builtin is an
  ordinary `Call`, so no name needs classifying to parse it (§0.5).
- **`alloc` is positional** (§0.4): after a `)` that closes a parameter list it
  is the effect marker; elsewhere it is an identifier. The decision uses only
  the parser's current production, not what any identifier denotes.
- **Modes are keyword-led** (`read`/`write`/`take`/`out`); an omitted mode is
  `take` by position. No parameter's type is consulted to find its mode.

No production requires knowing a declaration elsewhere in the file. The parser
builds the full AST for all five §11 programs with every referenced helper name
(`read_number`, `peek`, `advance`, `apply`, `POOL_VTABLE`, …) left undefined.

### 4. Divergences from design 0001's example code

0001 is the fixture and is **not** modified. Where its §11 code is
grammatically inconsistent or under-specified, the ruling is here and (only
where a fixture is affected) the fixture carries an inline `ADAPTED` note.

- **D1 — In-block `return` terminated with `,` (§11.5).** The last statement of
  the `binop` arm's block is written `return apply(op, lv, rv),` — a comma where
  a statement terminator belongs. Every other statement inside a block in the
  same document ends with `;`; the comma is an *arm separator* that leaked one
  level inside the block. **Ruling:** block statements end with `;`; a match arm
  that is a `{ … }` block needs no separating comma. **Fixture adaptation:**
  `tests/fixtures/11_5_arena.cn` writes `return apply(op, lv, rv);` with an
  inline `ADAPTED` comment. This is the only semantic change to any §11 code.

- **D2 — `alloc` is both a keyword and a field name (§6.1).** `AllocVtable`'s
  first field is literally named `alloc`, while `alloc` is also the effect
  marker. A single reserved word cannot be both. **Ruling:** `alloc` is a
  contextual keyword (§0.4); the field name parses as an identifier and the
  effect marker is recognized positionally. No fixture change; the allocator
  fixture keeps the field named `alloc`.

- **D3 — `match` statement terminator is inconsistent.** §11.3 writes
  `match s { … };` (trailing `;`) while §11.4/§11.5 write `match … { … }` as a
  bare statement (no `;`). **Ruling:** a block-like expression statement takes
  an *optional* `;` (§0.3). Both spellings parse; no fixture change.

- **D4 — Constructs named but not given syntax.** The `drop` hook's placement,
  top-level vtable *values* (`POOL_VTABLE`), and the exact bracket spelling of
  region variables are described but not shown as a complete production.
  **Ruling:** the grammar fixes trailing `drop(write self) { … }`, a `static`
  item, and `fn name[r](… read[r] …) -> read[r] …` respectively (§0.8, §2.1).
  Not inconsistencies — recorded so the choices are auditable.

- **D-A — `out` call-site spelling now implemented as design 0001 §3.1 writes it
  (resolved 2026-07-07).** Design 0001 §3.1's examples spell an out-mode argument
  `f(out x)`. This was previously passed *bare* (`f(x)`), which parsed but left
  the out-mutation invisible at the call site (against §3.1's intent and P13).
  The `Arg = [ "out" ] Expr` production above and the checker now make `out` the
  **mandatory** call-site marker for out-mode arguments: the marker is required
  for out-mode parameters and rejected for non-out parameters. This **resolves**
  a divergence (0001 §3.1's `f(out x)` spelling is implemented as written) rather
  than adding one; recorded here for auditability. D1-D4 above are unchanged.

**Fixture additions (not divergences, no semantics changed):**
- `11_1_allocator.cn` includes `struct AllocVtable { … }` from §6.1, which
  §11.1 references as `rawptr AllocVtable` (exercises fn-pointer field types).
- `11_3_mmio.cn` adds `enum Event { start, ready, err, stop }`, which §11.3 uses
  (`Event::start`, …) without a shown declaration. Not required to parse (no
  symbol table); added for a self-describing fixture.
- `11_2_scheduler.cn` keeps the design's `/* ... */` placeholder verbatim
  (block comments are supported).

All five §11 programs parse under this grammar; the golden test suite asserts it
(`tests/golden.rs`).

## Rejected alternatives

- **Untyped integer literals typed purely by context.** Rejected: it conflicts
  with local inference (P2) — a literal's type would depend on distant context.
  Default-`i64`-plus-suffix keeps the type readable at the literal (§0.1).
- **`as`-style or implicit numeric conversion instead of `conv T (e)`.**
  Rejected to match 0001 §8.1 exactly: `conv` is the only integer-conversion
  form and a lossy conversion "wears a word" (P13). Pointer retyping stays in
  the `unsafe` `cast_ptr[U]`/`addr_to_ptr[T]` intrinsics.
- **Making every builtin (`box`, `ptr_read`, …) a keyword.** Rejected: only the
  forms with non-call syntax (bracketed type arg / type+field) need to be
  keywords; treating the rest as ordinary calls keeps the parser and the token
  set smaller and the "no symbol table" story cleaner (§0.5).
- **`alloc` as a hard reserved word.** Rejected: it would make the design's own
  `AllocVtable.alloc` field unspellable (D2). Contextual recognition is the
  minimal fix and is still positional, honoring NN#13.
- **No wildcard pattern.** Rejected: `_` is the greppable spelling for an
  unbound payload (§8.2.1) and is independent of exhaustiveness, which the
  checker enforces regardless (§0.2).
- **Lexical/strict statement terminators (mandatory `;` after every `match`).**
  Rejected: it would reject §11.4/§11.5, whose function bodies are a bare
  `match`. Optional `;` after block-like statements accepts all of §11 (§0.3).
- **Allowing bare `Ident {` as a struct literal in `match`/`if`/`while` heads.**
  Rejected: it makes `match ev { … }` ambiguous. The `ExprNoStruct` restriction
  is the standard, symbol-table-free resolution (§0.7).

## Consequences and costs

- **Two-token lookahead is load-bearing** for `IDENT "::"`, `IDENT "{"`, and the
  optional `name ":"` in fn-pointer params. This is cheap but must be preserved
  by any later grammar change.
- **`alloc`'s contextual status is a standing subtlety.** A future construct
  that puts an identifier immediately after a `)`-closing-a-param-list would
  collide with the effect marker. None exists today; a reviewer changing
  signatures must keep this position reserved.
- **The grammar over-accepts by design.** It parses `read`/`write` modes on
  already-borrow-typed parameters, `result` outside `ensures`, borrow returns
  with local provenance, non-exhaustive matches, and assignment to non-place
  expressions. All of these are 0001 semantic rules the **checker** enforces
  (Stage 2); the parser's job is shape, not law. This is intentional: a
  permissive grammar with a strict checker keeps error provenance precise (P4)
  and the grammar symbol-table-free (NN#13).
- **`borrow`/`borrow_mut` type keywords are dead in the fixtures.** They exist so
  every construct 0001 names is representable; if Stage 2 confirms they are
  never needed in source, they are candidates for removal (P6 "when you add,
  remove").
