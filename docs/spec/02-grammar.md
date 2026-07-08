# 02 — Grammar

**Status: SKELETON.** Structure and obligations only. The normative grammar
**binds to design `0006`**. This chapter holds placeholders, not the throwaway
prototype grammar of design `0002` (a fixture fit to the design-0001 worked
examples, explicitly not a language proposal).

---

## 1. Obligations (NN#13, P2)

1.1 The grammar SHALL **parse without a symbol table** and without semantic
    context (NN#13): no production may require resolving an identifier's type or
    any declaration elsewhere in the source.

1.2 Type position and expression position SHALL be **grammatically** determined,
    never resolved (NN#13).

1.3 **Nothing crosses a public signature by inference** (NN#17, P2). A
    signature SHALL state, syntactically and completely: parameter types and
    passing modes, ownership and aliasing expectations (regions), the return
    type and its mode, declared contracts, and tracked effects. Body-local
    inference is permitted and encouraged; signature-level inference is
    forbidden.

1.4 The grammar SHALL admit a **permissive-parse / strict-check** separation:
    the parser fixes shape, the checker enforces the semantic rules of chapters
    03–08 with precise provenance (P4).

1.5 Semantic distinctions — ownership transfer, unsafety, arithmetic regime,
    contract clauses — SHALL be marked by keywords (P13); compactness is
    REQUIRED where verbosity carries no reader information.

---

## 2. Placeholders (to be filled from design 0006)

2.1 **PLACEHOLDER — item, type, statement, expression, and pattern grammar.**
    The prototype EBNF (design 0002 §2) is a fixture; the real productions are
    design-0006 work and are transcribed here when 0006 lands.

2.2 **PLACEHOLDER — signature grammar.** Must realize §1.3 completely: modes
    (value/shared/exclusive/out), region variables and their compact defaults
    (chapter 04 §7), effect markers (chapter 08), and contract clauses
    (chapter 07).

2.3 **PLACEHOLDER — the closed effect-marker set at the grammar level**
    (allocation, foreign trust — NN#19); no other effect keyword ships in the
    first stable edition.

**Gate:** blocks any stability commitment (chapter 99, obligation OBL-GRAM); does
not block the semantic core, which is stated over abstract constructs.
