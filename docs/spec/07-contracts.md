# 07 — Contracts

**Status: NORMATIVE-DRAFT** (§§1–4, the `enforced` level and the
optimizer-never-assumes rule, transcribed from design `0001-memory-model` §7.3
and design `0003` §2.7) **+ SKELETON** (§5, the `audit` and `assumed-proven`
levels — their P8 semantics stated, their full specification deferred). Rationale
is in design 0001 and philosophy P8.

---

## 1. Contract clauses

1.1 A function signature MAY carry preconditions (`requires`), postconditions
    (`ensures`), and a program MAY carry assertions (`assert`) as **executable,
    analyzable** clauses. `ensures` MAY reference the return value.

1.2 The enforcement **level** is declared **in the source** (per module or per
    contract), **never by a build flag** (P8). The levels are `enforced`,
    `audit`, and `assumed-proven`.

---

## 2. The `enforced` level (fully specified)

2.1 An `enforced` contract SHALL be checked dynamically wherever it is not
    statically proven. `requires` is checked at entry; `ensures` at each normal
    return; `assert` at its program point.

2.2 A violated `enforced` contract is a **fault** (chapter 06 §2.1) and routes
    through the root fault policy (chapter 06 §6).

---

## 3. The read-only rule for clauses

3.1 A `requires`, `ensures`, or `assert` condition SHALL be **read-only**: inside
    it, a program SHALL NOT move a non-`copy` value, take an exclusive (`write`)
    borrow, pass an `out` argument, or call a function that takes any argument by
    `take` (non-copy), `write`, or `out`. Reads, shared (`read`) borrows, and
    copy-`take` calls are permitted.

3.2 `ensures` accesses SHALL be **dataflow-checked against the post-body state at
    each normal return**: a clause that reads a parameter the body moved, or
    dereferences a `Box` the body already consumed, is the ordinary
    use-of-moved/freed error (chapter 04 appendix, E0301), analyzed once per
    return block against that block's own state (the precise must-semantics, since
    the clause runs at runtime at each return). `requires` is entry-checked and
    obeys the same read-only rule.

3.3 The `?` operator SHALL NOT appear inside a `requires`, `ensures`, or `assert`
    condition (E0708). A `?` is control flow — a conditional early return — not a
    read; a contract condition is a pure boolean with no escape, and modeling a
    `?` exit as a real return in the middle of contract evaluation is incoherent
    under the optimizer-never-assumes rule. This is the simplest sound rule
    (checker-soundness fix, 2026-07-08).

---

## 4. The optimizer-never-assumes rule (NORMATIVE now — load-bearing and settled)

4.1 **The optimizer SHALL NEVER assume a contract holds.** Contracts are checks,
    documentation, and oracles — **never** facts the compiler builds on. This rule
    is normative at every level, including `assumed-proven`.

4.2 A wrong `assumed-proven` contract therefore yields wrong **values**, never
    undefined behavior; NN#1 stands without an asterisk. `assumed-proven` buys
    **only** the removal of check overhead, **no** optimization headroom.

4.3 Contract-informed optimization is permitted **only** where the compiler
    **itself** proves the contract — in which case it is ordinary static analysis
    assuming nothing.

---

## 5. `audit` and `assumed-proven` — SKELETON (P8)

**Status: SKELETON.** The P8 semantics of these levels are stated here; their full
specification (structured recording format, enumeration surface) is deferred. The
prototype implements only `enforced` (design 0001 §7.3); §4 above binds all three
levels now.

5.1 **`audit`.** A violated `audit` contract SHALL be **recorded** (structured,
    P4-visible) and execution SHALL **continue**. It is semantically safe
    **precisely because** §4.1 holds: no analysis anywhere trusted the contract,
    so continuing past a violated one yields a logically wrong program, never an
    undefined one.

5.2 **`assumed-proven`.** The dynamic check SHALL be **skipped**; the annotation
    is an auditable assertion that verification happened externally, greppable
    exactly like an `unsafe` region. By §4.2 a wrong `assumed-proven` contract
    yields wrong values, never UB.

5.3 **Deferred detail (acceptance criterion).** The specification SHALL define the
    structured recording format for `audit` violations and the enumeration surface
    that lets the toolchain answer "show me every `assumed-proven` contract"
    (P17-style audit; chapter 99, obligation OBL-CONTRACT).

**Scope honesty (P8):** contracts localize **interface-level** intent; they do not
verify lock ordering, liveness, or protocol correctness.
