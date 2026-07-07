# Bet 5 — Frozen Classification / Unit Table

**Artifact type:** Frozen measurement instrument. This is the concrete positive
enumeration required by `BET5_CRITERION.md` §3.1 and produced at freeze step
`§6.2(i)`. It fixes, for every memory-model construct of both languages, its
semantic class, its counterpart in the other language, and the counting unit it
maps to. Together with the counting scripts (`prototype/src/count.rs` for Candor,
`tools/rust-count/` for Rust) it is the ruler; once hashed, neither the table nor
the prototype's memory-model syntax may change (`BET5_CRITERION.md` §3.1, §6.1).

**table_version:** `1`
**Governing document:** `BET5_CRITERION.md` v2 (normative). Where this table and
the criterion conflict, the criterion wins.
**Status:** PRE-REGISTERED (freeze step i).

**Blindness precondition (satisfied, recorded).** This table is authored in a
session **blind to any Candor port code, before any port exists**
(`BET5_CRITERION.md` §3.1, §6.2 i). At authoring time the repository contains no
basket port of any of the five programs; the only `.cn` source is the set of
`§11`-derived checker fixtures that ship with the prototype (design
`0001-memory-model.md` §11), which are worked examples of the memory model, not
scored ports. The which-side-benefits mapping (§6.5 blind-classification order)
is therefore not computable from anything in front of the author, so the
enumeration below is not fitted to a result already seen.

---

## 0. What is and is not counted (scope, from criterion §3.2)

Counting is **signature / declaration-site only** and **symmetric** across the two
languages. **One declared memory-model relationship = one unit regardless of token
spelling** (§3.2, finding 4/5). The four annotation classes are:

- **(a) borrow markers** — a binding/parameter/return declared a borrow (or a
  non-default caller-visible ownership mode) rather than a plain value.
- **(b) lifetime / region-relationship declarations** — one binding's region
  related to another's at a signature; **elision costs zero**.
- **(c) borrow-mutability declarations** — counted *only where the grammar
  separates mutability from the borrow marker of (a)*.
- **(d) valve-entry declarations** — a declaration-site mention of a pressure
  valve (unsafe entry, interior-mutability wrapper *type*, raw-pointer *type*).

**Excluded from annotation entirely** (§3.2): type names, value bindings, ordinary
control flow, arithmetic-regime keywords, and — by the ruling in §5.5 below — the
`alloc` effect. Ordinary value copies are the intended default gear and are counted
**separately** as value-copy units (§3.4), never as annotation.

**Use-site rule (§3.2, finding 5).** Use-site *borrow* expressions (`read x` /
`write x` operators in Candor, `&x` / `&mut x` at a call site in Rust) are **not**
counted at all — they are neither M1 annotation nor, by themselves, valve tokens.
Use-site *valve* tokens (raw-pointer operations, `unsafe {}` block bodies) are
**not** M1 annotation but **do** count toward the M2 valve region (§3.3).

---

## 1. Annotation class (a) — borrow markers

| Candor construct (AST) | Rust counterpart (syn) | Unit |
|---|---|---|
| `Param.mode == Read` (`read` parameter) | `&T` in a fn signature (incl. `&self`) | 1 |
| `Param.mode == Write` (`write` parameter) | `&mut T` in a fn signature (incl. `&mut self`) | 1 |
| `Param.mode == Out` (`out` parameter) — see §5.1 | *(no Rust counterpart; an `&mut`-out param is already counted above)* | 1 |
| `Param.ty` is a borrow-kind type (`slice`/`slice_mut`/`borrow`/`borrow_mut`) with mode `take` — see §5.2 | `&[T]` / `&mut [T]` / any `&`/`&mut` reference type in a param | 1 |
| Borrow return: `RetTy.borrow.is_some()` **or** `RetTy.ty` is borrow-kind — see §5.3 | `-> &T` / `-> &mut T` (a reference return type) | 1 |

**Per-parameter rule (no double counting).** A single parameter yields **exactly
one** class-(a) unit: `Out` → 1; else `Read`/`Write` → 1; else borrow-kind type
→ 1; else (plain `take` value) → 0. A `read`/`write` mode on an already-borrow-kind
type is ill-formed (design §3.1) and, if it ever appears, still yields a single
unit.

## 2. Annotation class (b) — region / lifetime relationships

| Candor construct (AST) | Rust counterpart (syn) | Unit |
|---|---|---|
| Each region variable in `FnDecl.regions` (the `[r]` declaration) | Each lifetime parameter declared in `<...>` (`'a`) | 1 each |
| Each `Param.region.is_some()` (a `read[r]` / `write[r]` attachment) | Each **named, non-`'static`** lifetime attached in a param type (`&'a T`, `Foo<'a>`) | 1 each |
| `RetTy.region.is_some()` (region tag on a borrow return) | Each named lifetime attached in the return type | 1 each |
| *(no Candor `where` clause)* | Each lifetime bound in a `where`/generic bound (`'a: 'b`, `T: 'a`) | 1 each |

**Elision costs zero** (§3.2). In Candor the compact default — exactly one borrow
parameter returning a borrow, no region written — is annotation-free (design §3.3);
it contributes **0** to class (b). In Rust an elided lifetime (`&T`, `&self`, `'_`)
and the special `'static` contribute **0** (see §5.8). This makes the worked pair
symmetric: Candor `fn pick[r](a: read[r] S, b: read S) -> read[r] E` = 3 class-(b)
units (decl + param attach + return attach); Rust `fn pick<'a>(a: &'a [E], b: &[E])
-> &'a E` = 3 (decl + param attach + return attach).

## 3. Annotation class (c) — borrow-mutability (structurally empty)

Class (c) is counted **only where the grammar separates mutability from the borrow
marker of class (a)**. Neither language separates them: Candor fuses borrow-kind and
mutability into a single keyword (`read` = shared, `write` = exclusive); Rust fuses
them into a single marker (`&` = shared, `&mut` = exclusive, counted as one class-(a)
unit each). Therefore **class (c) = 0 on both languages by construction** (§5.4). The
counters always emit `c: 0`; the field exists for schema symmetry and auditability.

## 4. Annotation class (d) — valve-entry declarations

| Candor construct (AST) | Rust counterpart (syn) | Unit |
|---|---|---|
| Each `unsafe "…" { … }` block (`ExprKind::Unsafe`) at its entry | Each `unsafe { … }` block; each `unsafe fn` declaration (§5.14) | 1 each |
| Each `rawptr T` type node in a **declaration** position — see §5.7 | Each `*const T` / `*mut T` type node in a declaration position | 1 each |
| *(N/A — the prototype ships no interior-mutability cell, design §4.3; §5.13)* | Each `Cell` / `RefCell` / `UnsafeCell` type mention in a declaration (§5.9) | 1 each |

Declaration positions for the raw-pointer type: struct fields, enum-variant
payloads, fn parameter/return types, `static` types, fn-pointer parameter/return
types, and `let` type annotations. **Not** counted: raw-pointer element types inside
use-site type-argument expressions (`cast_ptr[T]`, `ptr_null[T]`, `addr_to_ptr[T]`,
`conv`, `sizeof`, `alignof`, `offsetof`) — those are use-site, excluded from
annotation (§5.7).

---

## 5. Value-copy units (criterion §3.4)

| Candor construct (AST) | Rust counterpart (syn) | Unit |
|---|---|---|
| Each `clone` prefix operator (`ExprKind::Prefix { op: Clone }`) | Each `.clone()` **or** `.to_owned()` method call (a zero-argument method call so named) | 1 each |

Reported and gated **separately** from annotation (M1b/M4). **Rust matching is
method-name based** (§5.6): any method call whose method identifier is exactly
`clone` or `to_owned` and which takes no explicit arguments counts as one value-copy
unit **regardless of receiver type**. Accepted, recorded limitation: this over-counts
`Rc::clone`/`Arc::clone` (a refcount bump) and `.clone()` on `Copy` types, because
syn carries no type information to scope by receiver. It is an **upper bound** on
value-copies, symmetric with the Candor side, where `clone` on a `copy` type is also
syntactically counted (design §1.4). Fully-qualified `Clone::clone(x)` / `x.clone()`
via UFCS other than the `.method()` form is **not** matched (rare, recorded).

---

## 6. Valve regions (criterion §3.3) — for M2, not M1

A valve region is measured from **all** valve tokens, **including use-site ones**
(§3.3, finding 5).

**Candor valve region spans:** (i) each `unsafe` block's span; (ii) each `rawptr T`
type node's span in a declaration. All meaningful raw-pointer *operations* live
inside `unsafe` blocks by construction (design §4.2), so (i) already captures
use-site pointer manipulation; (ii) adds the declaration-site pointer surface.

**Rust valve region spans:** (i) each `unsafe { … }` block; (ii) each `unsafe fn`
item (its whole body); (iii) each `*const`/`*mut` raw-pointer type node; (iv) each
`Cell`/`RefCell`/`UnsafeCell` type node (declaration *or* use).

**Valve line (numerator).** A *logical line* (§7) that is wholly or partly inside a
valve region — i.e. a code line bearing ≥1 token whose source line falls within a
valve-region span's line range.

**Valve function.** A function (Candor `FnDecl`; Rust free fn, impl method, or trait
method) whose source span contains ≥1 valve region. A Candor `drop(write self)` hook
is treated as a function-like declaration here (§5.12). `total_functions` is the
count of all such functions.

---

## 7. Logical-statement and logical-line denominators (criterion §3.5)

**Logical statement** (primary gating denominator; §3.5 "an AST-derived node — a
declaration or a statement"). Counted node kinds:

*Candor* (from `prototype/src/ast.rs`):
- Each top-level `Item` — `Struct`, `Enum`, `Fn`, `Static` — counts **1** (a
  declaration).
- Each `Stmt` node anywhere in any body (recursively through nested blocks) —
  `Let`, `Assign`, `Expr` — counts **1**.
- **Not** counted as separate statements: struct `Field`s, enum `Variant`s, function
  parameters, match arms, and expression sub-nodes (§5.10) — the enclosing
  declaration or statement is the unit.

*Rust* (from `syn`):
- Each declaration item — free `fn`, `impl` method, `trait` method, `struct`,
  `enum`, `union`, `const`, `static`, `type` alias, `trait`, `mod` — counts **1**.
- Each `Stmt` in a block that is a `Local` (`let`), an `Expr`/`Semi` statement, or a
  `Macro` statement — counts **1**. A `Stmt::Item` is **not** double-counted here (it
  is already counted as a declaration item).
- **Not** counted: fields, variants, parameters, arms, sub-expressions.

**Logical line** (valve fraction denominator; §3.5 "logical lines"). A source line
(1-based) bearing **≥1 token** — for Candor from the prototype lexer, for Rust from
the `proc-macro2` token stream. Blank lines and comment-only lines carry no token and
are excluded (§5.11). `total_lines` is the count of such lines; `valve.lines` is the
count of them lying within a valve-region span's line range.

---

## 8. Deterministic JSON output (both counters)

Both counters emit, with sorted keys and a `per_site` list sorted by
`(start, end, class, kind)`:

```json
{
  "table_version": "1",
  "annotation_units": { "a": N, "b": N, "c": 0, "d": N, "per_site": [ {"class":"a","kind":"read_param","start":S,"end":E}, ... ] },
  "value_copy_units": N,
  "logical_statements": N,
  "valve": { "lines": N, "functions": N, "total_lines": N, "total_functions": N }
}
```

Every counted annotation site appears in `per_site` with its class, a kind tag, and
its byte span (Candor) / byte span mapped from the source (Rust) — the audit trail
required by §3.1 ("all counts are mechanical and scriptable").

---

## 5.x — Ambiguity ledger (adjudication discipline, criterion §6.5)

Every judgment call above, its decision, and a one-line rationale. Rulings are
recorded here for the open-comment discipline of §6.5.

- **§5.1 `out` mode → class (a), 1 unit.** `out` is a caller-visible, non-default
  ownership/init mode declaring a memory-model relationship the caller must know at
  the signature (caller keeps ownership; callee must initialize the slot, design
  §3.1); the nearest §3.2 class is (a) "a parameter declared [non-value] rather than
  a value." *Decision: count it, class (a).*
- **§5.2 `slice`/`slice_mut`/`borrow`/`borrow_mut` param *types* → class (a), 1
  unit.** A slice/borrow type **is** a borrow crossing the signature (design §5.2),
  passed by value with no mode keyword; its Rust counterpart `&[T]`/`&mut [T]` is a
  class-(a) borrow marker. *Decision: the borrow-kind param type is the marker.*
- **§5.3 Borrow returns → class (a), 1 unit.** A `-> read/write T` or `-> slice/
  borrow T` return declares a borrow crossing the signature boundary, symmetric with
  Rust `-> &T`. *Decision: count the borrow return, class (a).*
- **§5.4 Class (c) empty on both.** §3.2(c) applies only "where the grammar separates"
  mutability from the borrow marker; neither Candor (`read`/`write`) nor Rust
  (`&`/`&mut`) separates them. *Decision: `c` is structurally 0 on both sides.*
- **§5.5 `alloc` effect (and fn-pointer `alloc` marker) → NOT counted.** §3.2's
  classes are *memory-model* annotation (borrow / region / valve). The `alloc` effect
  is P2 effect tracking (design §3.2, §6.3), outside the borrow/ownership/valve model,
  directly analogous to the arithmetic-regime keywords §3.2 explicitly excludes. It is
  P2, not the memory model on trial. *Decision: exclude `alloc` from all annotation
  classes.*
- **§5.6 Rust value-copy scoping → method-name based, any receiver.** Any zero-arg
  `.clone()`/`.to_owned()` counts, regardless of receiver type, because syn has no
  type inference to scope by receiver. Known over-count on `Rc::clone`/`Copy`-type
  clones; accepted as an upper bound symmetric with Candor's syntactic `clone`.
  *Decision: count `.clone()`/`.to_owned()` method calls, receiver-agnostic.*
- **§5.7 rawptr / `*const` / `*mut` counted per type node, declaration positions
  only.** §3.2 is "signature/declaration-site only"; a `rawptr` element type inside a
  use-site type-argument expression (`cast_ptr[T]`, `conv`, `sizeof`, …) is a use
  site, excluded. Each raw-pointer type node in a declaration = 1 unit (so
  `rawptr rawptr u8` = 2, symmetric with two `*mut`). *Decision: per-node, decl only.*
- **§5.8 Rust `'static` and `'_` → class (b) 0.** They declare no inter-*binding*
  region relationship (`'static` is a fixed region; `'_` is elision), so like elided
  lifetimes they cost zero. Only named, non-`'static` lifetimes count. *Decision: 0.*
- **§5.9 Cell-family scope → exactly `Cell`/`RefCell`/`UnsafeCell`.** §3.2 names those
  three interior-mutable cells; `Mutex`/`RwLock`/`Atomic*` (concurrency, out of the
  single-threaded prototype's scope) and `Rc`/`Arc` (shared ownership, not interior
  mutability) are excluded. Matched by final path segment ident. *Decision: the
  three named cells only.*
- **§5.10 Logical statement = declarations + body statements; sub-declarations
  excluded.** Fields, variants, params, and arms are not separate logical statements —
  the enclosing declaration/statement is the unit — keeping the denominator symmetric
  (a Rust `struct` is one item; its fields are neither `Stmt` nor `Item`). *Decision:
  count items + `Stmt` nodes; not their sub-parts.*
- **§5.11 Logical line = a source line bearing ≥1 token.** Comment-only and blank
  lines carry no token and are excluded, defeating formatting/comment games while
  staying mechanical and language-neutral. *Decision: token-bearing source lines.*
- **§5.12 Candor `drop(write self)` hook → function-like.** A `drop` hook is the
  counterpart of Rust `impl Drop { fn drop(&mut self) }`: it counts as one
  function (`total_functions`, valve-function-eligible), one logical-statement
  declaration, and one class-(a) unit for its exclusive-`self` receiver, symmetric
  with the `&mut self` Rust receiver. *Decision: treat the hook as a function-like
  declaration.*
- **§5.13 Interior-mutability cell absent on Candor side.** The prototype ships only
  the `unsafe` valve (design §4.3); there is no Cell analog, so the Candor class-(d)
  interior-mutability count is structurally 0 — the honest upper-bound consequence the
  criterion absorbs via cell-substitutable tagging (§4.2). *Decision: Candor
  interior-mut = 0; Rust-only column.*
- **§5.14 Rust `unsafe fn` → class (d) 1 unit + whole body a valve region.** An
  `unsafe fn` is a declaration-site valve entry (class d) and its entire body is a
  valve region (§3.3, "unsafe blocks + unsafe fns"). Candor has no `unsafe fn` —
  `unsafe` is a block only (design §4.1) — so this counterpart is Rust-only; the
  asymmetry is intrinsic to the two grammars, not a measurement choice.
