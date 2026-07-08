# 02 — Grammar

**Status: NORMATIVE-DRAFT.** Complete transcription of the settled surface
grammar of design `0006-surface-syntax` (real toolchain syntax, adversarial
review #1 revised) over the prototype fixture of design `0002`. Rationale — the
sigil-vs-keyword calls, the NN#13 ambiguity walks, and the canonical-form
argument — lives in design 0006 §2–§6; this chapter states productions and rules
only. Notation: `{ X }` zero-or-more, `[ X ]` optional, `|` alternation, `( )`
grouping, `"lit"` terminal, UPPER a token class from chapter 01.

---

## 1. Obligations and conformance

1.1 The grammar SHALL **parse without a symbol table** and without semantic
    context (NN#13): no production may require resolving an identifier's type or
    any declaration elsewhere. Section 10 is the normative discharge.

1.2 **Two-token lookahead is the normative ceiling.** A conforming parser SHALL
    decide every production with at most **two tokens** of lookahead and **no
    identifier resolution** (design 0002 §"Consequences", retained by 0006 §3).
    No conforming grammar extension may raise this ceiling.

1.3 Type position and expression position SHALL be **grammatically** determined,
    never resolved (NN#13; §3, §10).

1.4 The grammar admits a **permissive-parse / strict-check** separation: the
    parser fixes shape; the checker enforces chapters 03–08 with precise
    provenance (P4). A rule stated here as a checker fact is not a parse
    ambiguity.

1.5 **Nothing crosses a signature by inference** (NN#17, P2). A signature (§4)
    SHALL state syntactically and completely: each parameter's type and passing
    mode, borrow regions, the return type and its mode, declared contracts, and
    the tracked effect marker. Body-local inference is permitted; signature-level
    inference is forbidden.

---

## 2. Items

    Program   = { Item } EOF
    Item      = [ "copy" ] ( Struct | Enum ) | Fn | Static
    Struct    = "struct" IDENT "{" [ FieldList ] "}" [ DropHook ]
    FieldList = Field { "," Field } [ "," ]
    Field     = IDENT ":" Type
    DropHook  = "drop" "(" "write" "self" ")" Block
    Enum      = "enum" IDENT "{" [ VariantList ] "}"
    VariantList = Variant { "," Variant } [ "," ]
    Variant   = [ "ok" ] IDENT [ "(" TypeList ")" ]
    TypeList  = Type { "," Type } [ "," ]
    Static    = "static" IDENT ":" Type "=" Expr ";"

2.1 `copy` may precede only a `Struct` or `Enum` (chapter 03 §4). A `DropHook`'s
    receiver SHALL be exactly `write self` (chapter 04).

2.2 **The `ok` marker (result-shaped enums).** The contextual keyword `ok`
    (chapter 01 §2.5) may prefix a variant in any position in the list. **At most
    one** variant of an enum SHALL be `ok`-marked; an enum is **result-shaped**
    iff exactly one of its variants is `ok`-marked (design 0006 §2.4). The
    `ok`-marked variant is the success/unwrap variant of the `?` operator (§6.5,
    §7). The parser sees only the `ok` token in variant position; which payload
    `?` unwraps is a checker fact (NN#13).

2.3 `static` bindings are immutable (chapter 03); a mutable global is not
    expressible this edition.

---

## 3. Types

    Type    = ScalarKw | "bool" | "unit"
            | "rawptr" Type | "Box" Type | "BoxResult" Type
            | Borrow
            | Array | Slice
            | FnPtrType
            | IDENT                              -- named struct/enum (unresolved)
    Borrow  = ( "read" | "write" ) [ Region ] Type
    Region  = "[" IDENT "]"
    Array   = "[" ArraySize "]" Type             -- owned fixed array [N]T
    Slice   = "[" Type "]"                        -- shared slice (region-free)
    ArraySize = INT | IDENT                       -- a const (literal or const name)
    ScalarKw  = "i8"|"i16"|"i32"|"i64"|"isize"|"u8"|"u16"|"u32"|"u64"|"usize"
    FnPtrType = "fn" "(" [ FnPtrParamList ] ")" [ "alloc" ] "->" Type
    FnPtrParamList = FnPtrParam { "," FnPtrParam } [ "," ]
    FnPtrParam     = [ IDENT ":" ] Mode Type

3.1 **Borrow types wear keywords** (design 0006 §2.2): the shared borrow of a
    place is `read T`, the exclusive borrow is `write T`. `&` is never a type
    constructor (chapter 01 §6.4). This frees `&` wholly for the bitwise-and
    operator (§6).

3.2 **Slices.** A **shared slice** is the bare `[T]` (`Slice`); an **exclusive
    slice** is `write [T]` (`Borrow` over a `Slice`). A shared slice carries a
    region only by wearing the keyword: `read[r] [T]` (`Borrow` = `read` `Region`
    `Slice`); the exclusive region slice is `write[r] [T]`. The bare `[T]` is the
    compact default and `read` appears on a shared slice only to carry a region,
    never redundantly (design 0006 §2.2; OBL-SLICE-REGION resolved).

3.3 **Array-versus-slice disambiguation.** After `[`, the parser parses one
    component; if a `Type` follows the closing `]`, the bracket held an
    `ArraySize` and the type is an `Array` (`[N]T`); otherwise the bracket held a
    `Type` and the type is a `Slice` (`[T]`). `ArraySize` (a const `INT`/`IDENT`)
    and `Type` are lexically distinguishable and no symbol table is consulted
    (design 0006 §3; §10).

3.4 A borrow of a sized array is `read [N]T` / `write [N]T` (`Borrow` over an
    `Array`), and an exclusive borrow of a slice is `write [T]`; both now parse as
    one borrow whose shared/exclusive-ness and run/array shape are in the type
    (design 0006 §2.2).

---

## 4. Signatures

    Fn        = "fn" IDENT [ Regions ] "(" [ ParamList ] ")" SigTail
                [ "->" RetTy ] Block
    Regions   = "[" "region" IDENT { "," "region" IDENT } [ "," ] "]"
    ParamList = Param { "," Param } [ "," ]
    Param     = IDENT ":" Mode Type
    Mode      = [ "take" | "out" | ( "read" | "write" ) [ Region ] ]
    SigTail   = { "alloc"
                | "requires" "(" Expr ")"
                | "ensures"  "(" Expr ")" }
    RetTy     = [ ( "read" | "write" ) [ Region ] ] Type

4.1 **Modes, take-by-omission (P12).** The four modes are `take`, `read`,
    `write`, `out`. An **omitted mode is `take`** (value/ownership transfer);
    this default is normative and not optional. `read`/`write` may carry a region
    tag (`Region`). `out` is a hard keyword (chapter 01 §2.4).

4.2 **The mode-versus-borrow-type canonical parse.** In parameter position a
    leading `read`/`write` SHALL be parsed as the **mode**, not as a borrow
    *type* over the following `Type` (design 0006 §3). Both derivations denote the
    same thing (a shared/exclusive borrow of a `T` passed in); this rule fixes the
    tree. The formatter never emits the redundant `take read T` composition
    (§9); the sole spelling of a `take`-mode borrow parameter is `read T`.

4.3 **The effect marker.** `alloc` in `SigTail` (and in `FnPtrType`) is the sole
    tracked effect keyword of this edition (chapter 08; NN#19). The effect set is
    closed and grows only by amendment with conservative-default migration
    (OBL-FFI). Effects are upper bounds (chapter 00 §3.3).

4.4 **Contracts.** `requires`/`ensures` clauses (§4, chapter 07) take a boolean
    `Expr`; `ensures` may name the return value `result`. These are bucket-1
    words and part of the complete signature (§1.5).

4.5 A return borrow wears its mode/region in `RetTy` (`read[r] [u8]`,
    `write T`); an omitted return mode is a by-value return.

4.6 **Region declarations wear the `region` keyword** (design 0007 §6.1.1;
    chapter 10 §1.3). Each entry of the post-name `Regions` list is
    `"region" IDENT`; a bare bracketed identifier is not a region declaration.
    This supersedes the bare-`[r]` declaration list and is the same syntax used
    when the bracket also carries type parameters (`fn choose[region r, T](…)`,
    chapter 10). The region **use** on a borrow type or mode keeps the bare tag
    (`read[r] T`; `Region`, §3); only the declaration wears the keyword.
    `region` is a contextual keyword (chapter 01 §2.5).

---

## 5. Statements

    Block  = "{" { Stmt } "}"
    Stmt   = Let
           | Expr "=" Expr ";"              -- assignment; LHS is a place
           | BlockLikeExpr                  -- no trailing ";" (see 5.2)
           | Expr ";"                       -- simple expression statement
    Let    = "let" [ "mut" ] IDENT [ ":" Type ] [ "=" Expr ] ";"
    BlockLikeExpr = Block | If | Match | Loop | While | For | Unsafe
                  | "wrapping" Block | "saturating" Block

5.1 **Assignment is statement position only** (precedence level 13; §6). `=` is
    not an expression operator; its LHS is a place (chapters 03/04).

5.2 **A block-like expression in statement position terminates the statement**
    (design 0006 §2.6 grammar fix). It takes **no** trailing `;`, and a following
    `(` begins a **new statement**, never an argument list. This removes the
    defensive-`;` workaround the prototype required after `unsafe {…}` and regime
    blocks.

---

## 6. Expressions

6.1 **Normative precedence table.** Candor adopts Rust's precedence, scoped to
    the operators it defines (design 0006 §2.4). Tightest first; this table is
    **normative** and the formatter's redundant-paren removal (§9) keys on it.

    | #  | Operators                                                    | Assoc  |
    |----|--------------------------------------------------------------|--------|
    | 1  | postfix `.*`, field `.`, index `a[i]`, call `f(…)`           | left   |
    | 2  | postfix `?`                                                   | postfix|
    | 3  | prefix `-` `!` `~`; borrow `read`/`write`; `conv T`          | prefix |
    | 4  | `*` `/` `%`                                                   | left   |
    | 5  | `+` `-`                                                       | left   |
    | 6  | `<<` `>>`                                                     | left   |
    | 7  | `&`                                                           | left   |
    | 8  | `^`                                                           | left   |
    | 9  | `\|`                                                          | left   |
    | 10 | `==` `!=` `<` `>` `<=` `>=`                                   | none   |
    | 11 | `&&`                                                          | left   |
    | 12 | `\|\|`                                                        | left   |
    | 13 | `=` (assignment, statement position only)                    | —      |

    Within level 1, `.*` binds tightest, so `s.*.f` is `(s.*).f`. Level 10 is
    **non-associative**: `a < b < c` is a parse error.

6.2 The layered EBNF realizing the table:

    Expr     = OrExpr
    OrExpr   = AndExpr  { "||" AndExpr }
    AndExpr  = CmpExpr  { "&&" CmpExpr }
    CmpExpr  = BitOr    [ ( "==" | "!=" | "<" | ">" | "<=" | ">=" ) BitOr ]
    BitOr    = BitXor   { "|" BitXor }
    BitXor   = BitAnd   { "^" BitAnd }
    BitAnd   = Shift    { "&" Shift }
    Shift    = AddExpr  { ( "<<" | ">>" ) AddExpr }
    AddExpr  = MulExpr  { ( "+" | "-" ) MulExpr }
    MulExpr  = Prefix   { ( "*" | "/" | "%" ) Prefix }
    Prefix   = ( "-" | "!" | "~" ) Prefix
             | ( "read" | "write" ) [ Region ] Prefix     -- borrow operator
             | "conv" ScalarKw Prefix                      -- conversion (6.4)
             | PropExpr
    PropExpr = PostfixExpr { "?" }
    PostfixExpr = Primary { ".*" | "." IDENT | "[" Expr "]" | "(" [ ArgList ] ")" }
    ArgList  = Arg { "," Arg } [ "," ]
    Arg      = [ "out" ] Expr                              -- out-mode argument (§4.1)

    `CmpExpr`'s single optional comparison encodes non-associativity (6.1).

6.3 **Dereference `.*` (postfix), read-only auto-deref.** `.*` is the postfix
    deref (retiring prefix `deref`), binding tightest (level 1). A **read** of a
    field/element through a tracked borrow needs no `.*` (`x.f`, `x.mem[i]`
    auto-deref); a **write** keeps every deref explicit (`x.*.f = v`,
    `x.*.mem[i] = v`); a bare pointee **value** takes `.*` in both positions
    (`b.*`). Whether a name is a borrow is a checker fact; the parser accepts both
    forms and the formatter canonicalizes (§9; design 0006 §2.4).

6.4 **Conversion `conv T e`.** The target `T` SHALL be a single **scalar-type
    keyword** (`ScalarKw` — an integer/`usize`/`isize` scalar name), never a
    bracketed or composite type; the operand `e` is the following prefix
    expression (parens only when it needs them). Restricting the target to one
    keyword token is what makes the parens droppable unambiguously (§10). `conv`
    is a semantic event (fault-on-loss by default; chapter 06); `as`/`T(e)` are
    rejected (design 0006 §2.4, §6).

6.5 **Propagation `?` (postfix).** On a value of a **result-shaped enum** (§2.2),
    `expr?` evaluates to the unwrapped payload if the value is the `ok`-marked
    variant, else `return`s that value unchanged from the enclosing function.
    `expr?` is well-formed **only** where the enclosing function returns the same
    enum `E` (same-type; cross-type conversion needs traits, deferred — chapter
    99, OBL-GENERICS). No ternary `?:` exists, so `?` is unambiguously postfix.

6.6 **Negative-literal fold.** The production `NegLiteral := "-" IntLiteral`
    folds a `-` **immediately preceding an integer-literal token** into one
    compile-time constant, range-checked against its target type. Intervening
    whitespace is permitted (`-5` and `- 5` both fold); a `(` after `-` does not
    match (it is unary negation of a parenthesized primary), and `-` before a
    non-literal is ordinary arithmetic negation. So `-9223372036854775808i64` is
    a valid `i64`, while a bare over-range or out-of-range folded constant is a
    **compile error** (chapter 01 §3.3), never a runtime fault (design 0006 §2.4).

6.7 Primaries:

    Primary = INT | STRING | "true" | "false" | "self" | "result"
            | "break" | "continue" | "return" [ Expr ]
            | "assert" "(" Expr ")" | "panic" "(" Expr ")"
            | "(" Expr ")"
            | ArrayLit
            | BlockLikeExpr
            | Intrinsic
            | "BoxResult" "::" IDENT [ "(" [ ArgList ] ")" ]   -- compiler-known
            | IDENT "::" IDENT [ "(" [ ArgList ] ")" ]         -- enum constructor
            | IDENT "{" [ FieldInitList ] "}"                  -- struct literal
            | IDENT                                            -- variable / callee
    ArrayLit = "[" "]" | "[" Expr ";" Expr "]" | "[" Expr { "," Expr } [ "," ] "]"
    FieldInitList = FieldInit { "," FieldInit } [ "," ]
    FieldInit     = IDENT ":" Expr
    Intrinsic = ( "cast_ptr" | "addr_to_ptr" | "ptr_null" ) "[" Type "]"
                    "(" [ ArgList ] ")"
              | "offsetof" "(" Type "," IDENT ")"
              | "field_ptr" "(" Expr "," IDENT ")"   -- safe field projection (0004)
              | "sizeof"  "(" Type ")"
              | "alignof" "(" Type ")"
              | ( "min_of" | "max_of" ) "(" Type ")" -- added by 0006 §2.4

6.8 **`field_ptr(p, f)`** is the safe, un-gated field-address form (design 0004);
    its second argument `f` is a field selector (field position), not an
    expression. `min_of`/`max_of` are **reserved but not
    implemented** this edition (chapter 01 §2.3; chapter 99 OBL-MINMAX-INTRINSICS):
    the negative-literal fold (§6.6) covers the corpus's programmatic-bound use, so
    the `i64::MIN` associated-constant spelling stays rejected — `::` is reserved
    for enum variants — without relying on them (design 0006 §2.4).

6.9 **Regime and unsafe expressions.** `wrapping Block` and `saturating Block`
    (chapter 06) and `unsafe STRING Block` are block-like expressions that yield a
    value, usable in expression position (design 0006 §2.6). `unsafe` REQUIRES a
    mandatory justification `STRING`; a per-expression `unsafe` modifier is
    rejected (design 0006 §2.6).

---

## 7. Patterns

    Pattern = "_"                                              -- wildcard
            | IDENT                                            -- binding
            | ( IDENT | "BoxResult" ) "::" IDENT [ "(" [ PatList ] ")" ]  -- variant
    PatList = Pattern { "," Pattern } [ "," ]

7.1 Payload sub-patterns compose: a variant payload may be a binding, `_`, or a
    nested variant pattern (chapter 03). Binding **modes** (move vs borrow) are
    determined by the checker from how the scrutinee is held, not by the parser.

7.2 The `ok` marker (§2.2) is a **variant-declaration** marker, not a pattern
    form: patterns name variants by their `IDENT` (`E::V`); the `ok`-ness of the
    matched variant is a checker fact keyed by `?` (§6.5), never spelled in a
    pattern.

---

## 8. `match`, and the arm-boundary rule

    Match = "match" ExprNoStruct "{" { Arm } "}"
    Arm   = Pattern "=>" Expr [ "," ]
    If    = "if" ExprNoStruct Block [ "else" ( If | Block ) ]
    Loop  = "loop" Block
    While = "while" ExprNoStruct Block
    For   = "for" Pattern "in" ExprNoStruct Block

8.1 **The per-arm `case` is dropped** (design 0006 §2.5): an arm is
    `Pattern "=>" Expr`, comma-separated. The arm separator comma is **optional
    after a block-bodied arm** — a following `Pattern` begins the next arm without
    a comma; elsewhere arms are comma-separated with an optional trailing comma.
    This is the **arm-boundary rule** and is decidable by the parser.

8.2 **`ExprNoStruct`.** In the head of `if`, `while`, and `match`, a bare
    `Ident {` is **not** a struct literal (it starts the block); a struct literal
    there SHALL be parenthesized. The restriction is lifted inside any `(…)`,
    `[…]`, call-argument list, or index. This is what keeps `match` symbol-table-
    free (design 0002 §0.7, retained by 0006 §2.5).

8.3 **The `for` statement (design 0009 §4.4; chapter 12).** `for` and `in` are
    **contextual keywords** — keywords only in the for-statement header (`for` in
    statement-leading position, `in` separating `Pattern` from `OPERAND`), and
    ordinary identifiers everywhere else. The operand parses as **`ExprNoStruct`**
    (§8.2), extending the `if`/`while`/`match` restriction so the `{` opening the
    loop body is never misread as a struct literal; a struct-literal operand SHALL
    be parenthesized. The operand's borrow mode (`coll` versus `read coll`, both
    `ExprNoStruct` via the borrow-operator `Prefix`, §6.2) selects the iteration
    protocol — a checker fact, not a parse fact (chapter 12 §3).

---

## 9. Canonical form (P3 / NN#11 — the formatter is the only form)

9.1 **The formatted form is the only conforming interchange form.** A conforming
    program presented for interchange SHALL be in the canonical form the blessed
    formatter emits (P16/NN#11); there is no configurable option. The parser MAY
    accept non-canonical input, but the formatter's output is the sole conforming
    form.

9.2 The canonical form is fixed as (design 0006 §4): 4-space indentation, never
    tabs; K&R braces (opening brace on the construct's line, closing brace on its
    own line); **mandatory blocks** on every `if`/`else`/`loop`/`while`/`match`/
    `fn`/`unsafe` (no brace-less bodies); one statement per line; simple
    statements end with `;` and block-like statements take none (§5.2); one space
    around every binary operator and `=>` and after `,` and after each mode/borrow
    keyword, and **no** space for postfix `.*`, `?`, field `.`, index `[…]`, or
    call `(…)`; trailing commas in every multi-line list and none in single-line
    lists; `snake_case` functions/locals/fields, `PascalCase` types/variants,
    `SCREAMING_SNAKE` statics.

9.3 The formatter performs these **normalizations** (input accepted, output
    canonical; design 0006 §4):
    - **Reborrow collapse (type-aware).** `f(write b.*)` / `f(read b.*)` filling a
      matching `read`/`write`-mode parameter → `f(b)` (design 0005); it SHALL NOT
      rewrite a `take`-mode borrow argument or a non-place operand (OBL-FMT-
      REBORROW).
    - **Read-only auto-deref collapse.** A `.*` immediately preceding a field `.`
      or index `[…]` on a **read** path → dropped (`s.*.base` → `s.base`); a `.*`
      on an **assignment target** is always kept (`p.*.f = e`), as is a bare-value
      `.*` (`b.*`) (§6.3).
    - **Redundant-paren removal (uniform).** A parenthesized subexpression whose
      operator binds tighter than its context — or equal with matching
      associativity — loses its parens, by the §6.1 table (`a & mask == 0`, not
      `(a & mask) == 0`).
    - **Throwaway-spelling rewrite.** `deref`/`case`/`slice`/`borrow` (and
      `slice[r]`) → their real forms (chapter 01 §2.6).

---

## 10. NN#13 — parseable without a symbol table

10.1 No production consults a declaration elsewhere in the file; two-token
     lookahead is the ceiling (§1.2). The following resolutions are stated as
     **non-normative notes** walking design 0006 §3; each resolves by grammatical
     position or maximal munch, never by resolving an identifier's type.

10.2 *(note)* **`&`** is never a type constructor (borrows are keywords, §3.1);
     it is always the binary bitwise-and operator, `&&` the logical-and, split by
     maximal munch (chapter 01 §6.4).

10.3 *(note)* **`.*` vs `.` `*`** resolves on one character of lookahead after
     `.` (chapter 01 §5.4, §6.2); `a * b` (multiply) and `a.*` (deref) never
     compete.

10.4 *(note)* **`[T]` slice vs `[N]T` array vs `[…]` literal vs `a[i]` index**
     resolves by position (type vs expression) plus the array/slice component
     test (§3.3); no symbol table.

10.5 *(note)* **`conv T e` vs a bracketed operand** cannot arise: the `conv`
     target is a single `ScalarKw` token (§6.4), so `conv [u8] …` is not a
     production and the operand begins immediately after the one keyword.

10.6 *(note)* **`read T`/`write T` mode vs borrow-type** overlap in parameter
     position is fixed by the canonical mode parse (§4.2): the leading keyword is
     the mode.

10.7 *(note)* **Slice regions `read[r] [T]`/`write[r] [T]`** are keyword-led: the
     region bracket immediately follows the borrow keyword and the slice type
     follows, with one-token lookahead after each bracket (§3.2).

10.8 *(note)* **`>>`** stays unambiguously a shift because no `<…>` generic
     bracketing exists (OBL-GENERIC-BRACKET forbids `<>`); **`-` + literal** is
     the §6.6 fold decided on the next token's lexical class; **`?`** is pure
     postfix; **`::`** is solely the enum-variant path; **`->`** is solely the
     return arrow (never in expression position).

---

**Gate (OBL-GRAM):** discharged by this chapter. Design 0006's productions are
transcribed, realizing complete signatures (modes, regions, effect, contracts;
§1.5, §4), the permissive-parse / strict-check separation (§1.4), the NN#13
symbol-table-free property (§10), and the NN#11 canonical form (§9). The design
0002 fixture is superseded, not codified.
