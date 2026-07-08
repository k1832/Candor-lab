# 01 — Lexical Structure

**Status: SKELETON.** Structure and obligations only. The normative lexical
detail **binds to design `0006`** (the real-language lexical/grammar design,
being drafted in parallel). This chapter holds placeholders and obligations, not
the throwaway prototype token set of design `0002` (which is a fixture, not a
proposal — philosophy quotes it as "deliberately throwaway syntax").

---

## 1. Obligations (NN#13)

1.1 The token grammar SHALL be **tokenizable without a symbol table** and
    without semantic context (NN#13). Every token class SHALL be decidable from
    the character stream alone.

1.2 Tokenization SHALL be **predictable** (P13): the token boundaries of a
    source text SHALL be a function of that text, independent of any
    declaration elsewhere.

1.3 The token inventory SHALL distinguish, as load-bearing lexical categories:
    integer literals with an explicit optional type suffix (no context-typed
    literals — P2); the semantic keywords that mark ownership transfer, unsafety,
    arithmetic regime, and contract clauses (P13); and the punctuation reserved
    for disambiguation.

1.4 Any token reserved **exclusively** for one grammatical role (in the
    prototype fixture, `::` for enum-variant reference) SHALL be recorded as
    such, so that its two-sided identifier context parses by position without
    resolving either identifier (NN#13).

1.5 A canonical source form SHALL exist and SHALL be the only form (NN#11); the
    lexical layer SHALL admit that canonicalization.

---

## 2. Placeholders (to be filled from design 0006)

2.1 **PLACEHOLDER — real-language token inventory.** The prototype fixture's
    ugly-but-unambiguous keywords (`read`, `write`, `take`, `out`, `unsafe`,
    `wrapping`, `saturating`, `conv`, `deref`, `clone`, ...) are **not**
    normative here. Design 0006 sets the real spelling under P13's clarity-density
    test; this section transcribes it when 0006 lands.

2.2 **PLACEHOLDER — comment, whitespace, string, and numeric-literal lexis.**

2.3 **PLACEHOLDER — contextual keywords.** The set (in the fixture, `alloc`)
    that lex as identifiers and are recognized positionally is design-0006 work;
    each SHALL satisfy §1.1.

**Gate:** this chapter blocks nothing in the semantic core (chapters 03–08 are
specified over abstract constructs, not concrete tokens), but it gates any
stability commitment (chapter 99, obligation OBL-LEX).
