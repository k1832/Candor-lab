# Candor Language Specification — Front Matter

**Document class:** NORMATIVE specification (philosophy P18, NN#20).
**Status of this skeleton:** pre-stability working draft. This file and its
sibling chapters are the *spec skeleton* of the §8 critical path ("the semantic
core and its spec skeleton — P18's models are cheapest before code exists").

---

## 1. Identity and authority

1.1 This document is the **normative specification** of the Candor language. It
    defines the language's semantics **independently of any implementation**
    (P18, NN#20).

1.2 The specification **binds the compiler absolutely**. Per the philosophy §9
    hierarchy (`LANG_PHILOSOPHY.md` > design documents > implementation >
    compiler behavior), the specification sits at the design-document tier and
    the implementation is its subject. Where the compiler and this document
    disagree, the **compiler is wrong** by definition; "the compiler is the
    spec" is explicitly rejected (P18).

1.3 This document is **subordinate to `LANG_PHILOSOPHY.md`**. Where the
    philosophy and this specification conflict, the philosophy wins and this
    document is the artifact that changes.

1.4 The design documents under `docs/design/` carry the **rationale**; this
    specification carries the **normative clauses**. Each chapter states its
    rules without rationale prose and points back to the design document that
    argues them. The current normative core is transcribed from design
    `0001-memory-model.md` (the battle-tested prototype semantic core, hardened
    across five adversarial reviews and twelve enumerated holes), with
    extensions from designs `0004` (`field_ptr`) and `0005` (implicit reborrow),
    and its soundness claim structure from `0003-checker-soundness.md`.

---

## 2. Conformance language

2.1 The key words **MUST**, **MUST NOT**, **SHALL**, **SHALL NOT**, **REQUIRED**,
    **SHOULD**, and **MAY** are used in the RFC-2119 sense.

2.2 A **conforming implementation** SHALL accept every program this document
    designates well-formed and SHALL reject every program this document
    designates ill-formed, and SHALL give each accepted program exactly the
    observable behavior this document defines, subject only to the declared
    indeterminacy of the fault window (chapter 06) and declared nondeterminism
    (P5).

2.3 A **conforming program** is one this document designates well-formed.

2.4 Where a chapter is not yet NORMATIVE-DRAFT (see §4), a conforming
    implementation is bound only by the obligations that chapter states, not by
    unwritten detail; the gap is tracked in chapter 99.

---

## 3. Versioning discipline

3.1 The language evolves by **edition** (P15). This specification is versioned
    per edition; a normative clause is stable within an edition.

3.2 Any breaking change to a normative clause SHALL ship with an automatic
    migrator (NN#14); a normative change that cannot be mechanically migrated
    SHALL NOT ship. Sound conservative over-approximation qualifies as
    mechanical migration.

3.3 Effects are **upper bounds** (NN#19): a signature may overstate, never
    understate. Removing an effect marker is a non-breaking strengthening; adding
    one is a breaking change. This asymmetry is normative and SHALL NOT be
    resolved by drift.

3.4 **No stability commitment ("1.0")** for this specification may precede Bet
    5's pre-registered verdict (NN#14) and the discharge of the mandatory
    pre-stability obligations of chapter 99 (notably NN#20, the fault-window
    formalization).

---

## 4. Status ledger

Each chapter carries one status:

- **NORMATIVE-DRAFT** — content exists and is battle-tested; the clauses are a
  genuine transcription of validated semantics, complete for their scope.
- **SKELETON** — structure and obligations exist; normative detail is deferred
  and tracked in chapter 99.
- **ADOPTED-PENDING** — the content is external proven art to be adopted, with
  the named source; the structure and the decisions-before-landing are recorded.

| Chapter | Title | Status | Source / obligation |
|---------|-------|--------|---------------------|
| 00 | Front matter | NORMATIVE (process) | philosophy §9, P18, NN#20 |
| 01 | Lexical structure | NORMATIVE-DRAFT | design 0006 (token inventory); NN#13/P13/NN#11 |
| 02 | Grammar | NORMATIVE-DRAFT | design 0006 (real EBNF); NN#13/NN#17/P2/NN#11 |
| 03 | Types and values | NORMATIVE-DRAFT | design 0001 §1/§5/§8; 0006 spellings |
| 04 | Ownership and borrows | NORMATIVE-DRAFT | design 0001 §2/§3/§5 + 0005; 0006 spellings |
| 05 | Unsafe and pointers | NORMATIVE-DRAFT | design 0001 §4 + 0004; P18 aliasing model (§6, OBL-ALIAS) |
| 06 | Faults | NORMATIVE-DRAFT + SKELETON | design 0001 §7 + P5; NN#20 window: single-threaded discharged (J1), concurrency composition deferred |
| 07 | Contracts | NORMATIVE-DRAFT + SKELETON | design 0001 §7.3 + P8 |
| 08 | Effects | NORMATIVE-DRAFT + SKELETON | design 0001 §3.2/§6 + P17 |
| 09 | Memory consistency model | ADOPTED-PENDING | C/C++20 axis (P18); P10 |
| 10 | Generics and interface bounds | NORMATIVE-DRAFT | design 0007 + 0009 §2; P11/NN#10 |
| 11 | Modules | NORMATIVE-DRAFT + SKELETON | design 0008; P20/NN#17 |
| 12 | Iteration and associated types | NORMATIVE-DRAFT | design 0009; P11/P4 |
| 13 | Coherence and dispatch soundness | NORMATIVE-DRAFT | design 0018 §4; 0007 §2.3 + 0009 §2.2; NN#1/NN#20 |
| 99 | Obligations tracker | NORMATIVE (process) | all SKELETON/PENDING items |

4.1 A SKELETON or ADOPTED-PENDING designation is **not** a licence for
    undefined behavior. Every such chapter's stated obligations bind now; only
    the deferred detail is open. No side door to UB in safe code opens through
    an unfinished chapter (NN#1).

---

## 5. Scope of the current core

5.1 The normative core transcribed here is the **safe-language semantics** of
    the value/borrow/valve model, the fault model's prototype-validated part,
    the `enforced` contract level, and the `alloc` effect.

5.2 User-defined generics and interface bounds (chapter 10), the module
    structure (chapter 11 §§1–6), iteration with associated types (chapter 12),
    and the coherence/dispatch soundness invariant (chapter 13) are now
    specified. The core deliberately **does not yet** specify: concurrency
    and its consistency model (P10, chapter 09), FFI / boundary-module *content*
    and the foreign-trust effect (P17, chapters 08/11 §10.1), the imprecise
    fault window's concurrency composition (NN#20; the single-threaded core is
    now specified, chapter 06 §7.4), and the P20 module interface-artifact /
    two-hash incrementality
    machinery (chapter 11 §§7–10, SKELETON). Each open item is an obligation in
    chapter 99, not an omission that licenses UB.
