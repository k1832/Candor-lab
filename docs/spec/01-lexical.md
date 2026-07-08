# 01 — Lexical Structure

**Status: NORMATIVE-DRAFT.** Complete transcription of the settled surface
lexis of design `0006-surface-syntax` (the real toolchain syntax, adversarial
review #1 revised) over the prototype fixture of design `0002`. Rationale — the
token-economics (P13) and no-symbol-table (NN#13) arguments — lives in design
0006 §1–§3; this chapter states rules only. It supersedes design 0002's
throwaway inventory; it does not codify it.

---

## 1. Obligations (restated, now discharged)

1.1 The token grammar SHALL be **tokenizable without a symbol table** and
    without semantic or parse context (NN#13): every token class SHALL be
    decidable from the character stream alone, and the token boundaries of a
    source text SHALL be a function of that text alone, independent of any
    declaration elsewhere. Section 6 is the normative discharge of this clause.

1.2 A canonical source form SHALL exist and be the only conforming interchange
    form (NN#11); the lexical layer admits that canonicalization (chapter 02 §9).

---

## 2. Keywords

2.1 A **hard keyword** is reserved in every position and SHALL NOT be used as an
    identifier. The hard keywords are exactly:

    struct  enum  fn  static  copy  drop  self  let  mut
    take  read  write  out
    if  else  match  loop  while  break  continue  return
    requires  ensures  assert  panic  result
    unsafe  wrapping  saturating  conv  clone
    true  false

2.2 The **type-constructor keywords** `rawptr`, `Box`, and `BoxResult` and the
    **scalar-type keywords**

    i8  i16  i32  i64  isize   u8  u16  u32  u64  usize   bool  unit

    are hard keywords, lexed distinctly from identifiers. `bool` and `unit` are
    types but SHALL NOT appear as integer-literal suffixes (§3.2).

2.3 The **intrinsic keywords** are keyword-led operator forms reserved in every
    position: `sizeof`, `alignof`, `min_of`, `max_of`, `offsetof`, `field_ptr`,
    `cast_ptr`, `addr_to_ptr`, `ptr_null`. `min_of` and `max_of` are **reserved but
    not implemented** this edition: design 0006 §2.4 added them to the compile-time
    `sizeof`/`alignof` family (a type argument yielding the minimum/maximum value
    of that type), but the prototype front-end never implemented them and the
    negative-literal fold (§3.4; chapter 02 §6.6) covers the corpus's
    programmatic-bound use. Their token stays reserved so the spelling remains
    available; implementing or dropping them is deferred to the real-toolchain
    gate (chapter 99, OBL-MINMAX-INTRINSICS).

2.4 **`out` is a hard keyword everywhere** (design 0006 §2.3). It SHALL NOT be an
    identifier in any position. The contextual relaxation is rejected: the
    out-argument marker sits adjacent to arbitrary place expressions, so a
    contextual reading could not be separated from an `out` identifier inside
    that same expression grammar.

2.5 A **contextual keyword** lexes as an ordinary identifier (IDENT) and is
    recognized only by grammatical position; outside that position it denotes an
    identifier. The contextual keywords are:

    - **`alloc`** — recognized only in the effect-marker slot of a signature or
      function-pointer type (chapter 02 §4); elsewhere an identifier (design 0002
      §0.4, retained by 0006 §2.3).
    - **`ok`** — recognized only in variant-leading position within an `enum`
      body, where it marks the single success/unwrap variant of a result-shaped
      enum (chapter 02 §2, §7); elsewhere an identifier. Its slot is a fixed,
      non-expression position (unlike `out`), so contextual recognition raises no
      identifier ambiguity.
    - **`for`** and **`in`** — recognized only in the for-statement header
      (chapter 02 §8.3; design 0009 §4.4): `for` in statement-leading position,
      `in` separating the pattern from the operand; elsewhere identifiers.
    - **`type`** — recognized only in interface-body member position, declaring
      the single associated type (chapter 12 §1, `type Item;`); elsewhere an
      identifier.
    - **`region`** — recognized only in a bracketed declaration list after an
      item name, declaring a region variable (chapter 02 §4, chapter 10 §1.3;
      design 0007 §6.1.1); elsewhere an identifier. The region **use** tag on a
      borrow type is the bare `[r]` (chapter 02 §3), not the keyword.

2.6 **Retired spellings.** The prototype keywords `slice`, `slice_mut`, `borrow`,
    `borrow_mut`, `deref`, and `case` (and the throwaway region forms `slice[r]`
    / `slice_mut[r]`) are **not** part of the canonical token inventory; they are
    recognized only by the migration formatter, which rewrites them to their
    real forms, and SHALL NOT appear in canonical source (design 0006 §4–§5).

---

## 3. Literals

3.1 **Integer literals.**

    INT        = ( DEC | HEX ) [ INT_SUFFIX ]
    DEC        = digit { digit }
    HEX        = "0" ("x" | "X") hexdigit { hexdigit }
    INT_SUFFIX = "i8" | "i16" | "i32" | "i64" | "isize"
               | "u8" | "u16" | "u32" | "u64" | "usize"

    An integer literal is a single token. Its default type is **`i64`**; a
    suffix, written with no separator, overrides it (design 0002 §0.1, retained).
    The suffix SHALL be one of the integer scalar names above; `bool` and `unit`
    are not suffixes.

3.2 The suffix is part of the literal token only when it immediately follows the
    digits with no intervening character; an alphabetic run that is not a valid
    `INT_SUFFIX` is a separate token (`42u8` is one token; `42 u8` and `42bool`
    are two).

3.3 **Over-range is a compile error, never a fault.** An integer literal whose
    magnitude does not fit the type it is required to take — a suffixed literal
    outside its suffix type's range, or an unsuffixed literal outside `i64` — is
    **ill-formed** (rejected at compile time). In particular a **bare unsigned
    over-range literal** such as `9223372036854775808` written on its own fits no
    target type and SHALL be **rejected at compile time**; it SHALL NOT produce a
    runtime fault (design 0006 §2.4, §3).

3.4 **The negative-literal fold is grammatical, not lexical.** A leading `-` is
    never part of an integer-literal token; it lexes as the `-` operator (§4).
    The combined production `NegLiteral := '-' IntLiteral` (chapter 02 §6.6)
    folds a `-` immediately preceding an integer-literal token into one
    compile-time constant, range-checked against its target type; intervening
    whitespace is permitted, and a `(` after `-` does not match the production.
    Thus `-9223372036854775808i64` is a valid `i64`, while
    `-(9223372036854775808)` over-ranges as an ordinary parenthesized primary
    (§3.3).

3.5 **String literals** (byte-string literals):

    STRING = '"' { char | escape } '"'      escape = \" | \\ | \n | \t

    A string literal denotes a sequence of `u8` (the canonical text view is the
    byte slice `[u8]`; chapter 03, OBL-TEXT). The recognized escapes are exactly
    `\"`, `\\`, `\n`, `\t` (design 0002 §1, retained).

3.6 **Boolean literals** are the hard keywords `true` and `false`.

---

## 4. Identifiers, comments, whitespace

4.1 **Identifiers.**

    IDENT = ( alpha | "_" ) { alpha | digit | "_" }

    The single underscore `_` alone is the **wildcard**, not an identifier
    (chapter 02 §7). An identifier SHALL NOT be spelled the same as any hard or
    type keyword (§2.1–§2.3); a contextual keyword (§2.5) may be an identifier
    outside its recognizing position.

4.2 **Comments** are `// … <end-of-line>` (line) and `/* … */` (block,
    **non-nesting**). Comments are lexically equivalent to whitespace.

4.3 **Whitespace is insignificant beyond token separation.** It separates
    adjacent tokens and carries no other meaning: it is not layout-significant,
    and no production depends on line breaks or indentation. Indentation and line
    breaks are fixed by the canonical formatter (chapter 02 §9) but are not part
    of the token grammar.

---

## 5. Operators, punctuation, and maximal munch

5.1 The operator and punctuation tokens are exactly:

    ( ) { } [ ]   ,  .  ;  :  ::  ->  =>  .*
    =  ==  !=  <  <=  >  >=
    +  -  *  /  %
    &  |  ^  ~  <<  >>
    &&  ||  !
    ?

    The **bitwise set** `&` `|` `^` `~` `<<` `>>` is added by design 0006 §2.4;
    `~` is prefix bitwise-not, distinct from prefix logical-`!`. `.*` is the
    postfix dereference token (§5.4). `?` is the postfix propagation operator.
    `::` is reserved **exclusively** for the enum-variant path (design 0001;
    chapter 02 §6, §7), so its two identifier operands parse by position without
    resolving either (NN#13).

5.2 **Maximal munch.** The lexer SHALL always form the **longest** token that the
    contiguous character stream admits. This is what distinguishes, from single
    characters, each multi-character token: `&&` from `&`, `||` from `|`, `<<`
    from `<`, `>>` from `>`, `==` from `=`, `!=` from `!`, `<=`/`>=` from `<`/`>`,
    `::` from `:`, `->` and `=>` from `-`/`=`/`>`, and `.*` from `.` (§5.4).

5.3 **The two documented munch boundaries.** Maximal munch is uniform; its two
    reader-visible consequences are recorded here:
    - The negative-literal fold is **not** a munch: `-42` is two tokens (`-`,
      `42`); folding is grammatical (§3.4, chapter 02 §6.6).
    - The integer suffix is munched into the literal token only when contiguous
      and valid (§3.2).

5.4 **The `.*` tokenization clause.** On reading `.`, the lexer SHALL inspect the
    **immediately following character**: if it is `*`, the two characters form
    the single token `.*` (postfix deref); otherwise `.` is the field-access
    token and the following token is lexed independently. Intervening whitespace
    breaks the pair: `.` then space then `*` is the field token `.` followed by
    the multiply token `*`. This decision uses one character of lookahead and no
    parse context (NN#13; §6).

---

## 6. NN#13 — tokenization requires no parse context (normative)

6.1 Every token class SHALL be decidable from the character stream alone, with
    no reference to the grammar's current production, to any declaration, or to
    any identifier's meaning. Tokenization is a total function of the source
    text. The following are the walked discharge cases; they are **normative**
    for the token boundaries they fix.

6.2 **`.*` versus `.` `*`.** Resolved by §5.4 on one character of lookahead:
    a `.` followed with no space by `*` is the deref token `.*`; a `.` followed
    by an identifier is the field token; a bare `*` (not preceded by an
    unspaced `.`) is the multiply token. `a * b` (multiply) and `a.*` (deref)
    never share a lexical form.

6.3 **`>>` versus `>` `>`.** `>>` is always the single shift token by maximal
    munch (§5.2). Candor has no `<…>` generic bracketing (deferred; OBL-GENERIC-
    BRACKET forbids `<>` precisely so this stays true), so no close-angle context
    competes with the shift token; `a >> n` never needs re-splitting.

6.4 **`&` positions.** Borrows are keywords (`read` / `write`), so `&` is
    **never** a type constructor and never appears in prefix position. `&` is
    always the binary bitwise-and operator and `&&` always the binary logical-and
    operator, the two separated by maximal munch (§5.2). No positional overload of
    `&` exists to carry (design 0006 §2.2, §3): this is the payoff of spelling
    borrow types with keywords rather than `&`.

---

**Gate (OBL-LEX):** discharged by this chapter. Design 0006's token inventory is
transcribed under §1's obligations, satisfying NN#13 (§6), P13, and the NN#11
canonicalization admission; the design 0002 fixture is superseded, not codified.
