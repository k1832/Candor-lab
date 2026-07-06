# 0001 — Memory Model (Bet 5 Validation Prototype)

**Status:** Design. Subordinate to `LANG_PHYLOSOPHY.md`, which outranks this document (§9 hierarchy). Where this document and the philosophy conflict, the philosophy wins and this document is the artifact that changes.

**Scope.** This document specifies the memory/ownership model for the *Bet 5 validation prototype* only: a **checker plus tree-walking interpreter**, deliberately throwaway syntax, **no optimizer**, **no stdlib beyond slices and one allocation surface**. It is written to be implemented from without further questions, and to be the target the adversarial basket is authored against: an allocator, an intrusive-list scheduler, an MMIO driver state machine, a parser, and an arena-based compiler pass (Bet 5, §3).

**What the prototype must not do.** It must not accidentally make Candor look better or worse than the real language would (Bet 5 measurement validity). Two consequences recur below: (a) where a real-language convenience is cheap to implement soundly, the prototype includes it, so the measurement is not biased by artificial friction (e.g. non-lexical borrows); (b) where a real-language convenience would *hide* a pressure valve, the prototype refuses it, so valve frequency is measured honestly (e.g. no borrow-typed struct fields).

**Throwaway syntax contract.** Keywords are ugly-but-unambiguous. The grammar parses without a symbol table (NN#13): every construct is disambiguated by keyword and grammatical position, never by resolving an identifier's type. Do not read the syntax as a proposal for the real language; read it as a fixture.

---

## 0. Reading order and the three gears

The model has exactly **three gears**, and every construct below lands in one:

1. **Value gear** (default) — ownership and moves/copies. No annotation beyond types.
2. **Borrow gear** (second) — shared and exclusive borrows. Marked with keywords, inferred body-locally, never inferred across a signature.
3. **Valve gear** (escape) — `unsafe` regions with raw pointers, author-chosen, justification-carrying, greppable.

Bet 5 is the claim that gears 1–2 carry the *common* systems workload and gear 3 stays *rare in occurrence even where critical in function* (P12). The prototype exists to measure that. Therefore the boundary between gear 2 and gear 3 is drawn to be **visible and honest**, not to be flattering.

There is deliberately **no fourth gear** (no checked-runtime interior-mutability type). §4.3 defends that.

---

## 1. The value model

### 1.1 What a value is

A **value** is a fully-initialized instance of a type occupying storage. Every value has, at every program point, **exactly one owner**: a local binding, a struct field, an array element slot, or the pointee slot of a `Box`. Ownership is unique and total — there is no shared ownership in safe code.

A **place** is an expression denoting storage that holds (or will hold) a value: a local, a field access `p.f`, an index `a[i]`, or a dereference `deref b`. Places are what you borrow, move out of, and assign to.

### 1.2 Move vs. copy

Passing or assigning a value from a place does one of two things, determined **solely by the value's type** (so cost is visible from the type, per P4/P9 — no invisible expensive copies):

- **Copy** — the source place remains valid and holds an equal value. Permitted only for types that are `copy` (§1.3). A copy is always a flat, dependency-free bit copy of known, bounded cost.
- **Move** — ownership transfers; the source place becomes **invalid** (uninitialized) and may not be read until reassigned. The default for all non-`copy` types. A move is a shallow ownership transfer; it never runs user code and never deep-copies.

There is **no third option**. In particular there is no implicit deep copy: producing an independent duplicate of a non-`copy` value requires the explicit `clone` operator (§1.4).

### 1.3 Which types are `copy`

`copy` is a structural, checker-computed property, **and** it may be requested by the author only where it is cheap:

- The scalar primitives are `copy`: all sized integers, `bool`, `unit`, and any `rawptr T` (a raw pointer is just an address; copying one is inert in safe code — see §4.2).
- A **shared borrow** value (`read`-borrow, §2) is `copy`. An **exclusive borrow** value (`write`-borrow) is **not** `copy` (it moves — this is the aliasing rule made a type property, §2.2).
- A fixed array `[N]T` is `copy` **iff** `T` is `copy`. (Bit copy of `N` elements: O(N) but predictable and visible from the type.)
- A `struct` or `enum` is `copy` **iff** the author writes the `copy` marker on its declaration *and* every field/variant payload is `copy` *and* the type has no `drop` hook (§1.5). The marker is opt-in so that copyability — and thus the presence of implicit duplication at use sites — is an author decision recorded at the type, never a silent consequence of field shapes.
- `slice T` (shared slice) is `copy`; `slice_mut T`, `Box T`, and every owning aggregate without the `copy` marker **move**.

Rationale for opt-in struct copy: a two-`i32` struct is cheap to copy, but making copyability automatic means adding a field can silently flip a type from move to copy (or back), changing move-checking outcomes far away. The marker keeps that decision local and greppable (P2), at the cost of one keyword on the small number of types that want it.

### 1.4 Explicit `clone` and `copy` operators

- `clone place` produces an owned, independent duplicate of the value at `place`. It is defined **structurally**: recursively `clone` each field/element; for `Box T` it allocates a new box (so `clone` on a box-bearing type is an **`alloc`-effecting** operation, §6.3, and the call site inherits the effect). `clone` is available for any type all of whose fields are cloneable; a type is not cloneable if it contains a `rawptr` (the prototype will not guess how to duplicate raw graph memory) — such a type must be duplicated by hand inside a valve.
- No separate `copy` operator is needed: for a `copy`-typed place, ordinary use already copies. `clone` on a `copy` type is permitted and equals a copy.

`clone` is the *only* implicit-cost-free way to say "I am paying for a duplicate here," and it is always visible in the source (P13: the expensive thing wears a word).

### 1.5 Destruction and deterministic drop order

A value is **destroyed** (dropped) when its owner ceases to exist. Destruction is fully deterministic and part of the source-declared semantics (P5):

- A type may declare **at most one** `drop(write self) { ... }` hook. It runs **before** the value's fields are destroyed. It may not move `self` or any field out (it borrows `self` exclusively). It runs exactly once per live value.
- After the hook (or immediately, if there is none), fields are destroyed in **reverse field-declaration order**; array elements are destroyed from **highest index to lowest**; enum payloads by the same field rule.
- A local binding is destroyed at the **end of its enclosing block**, and locals in the same block are destroyed in **reverse order of first initialization** (LIFO). A temporary is destroyed at the end of the full statement that created it.
- A place that has been **moved out of is not destroyed** at scope end — its ownership left. Move state is a static property at each program point (§1.6), so the interpreter needs **no runtime drop flags**.
- Reassigning a place first destroys the value it currently holds (if any), then stores the new one.

Drop order is specified precisely because a reviewer must be able to predict resource release (an allocator freeing its backing, an MMIO driver quiescing a device) from the source alone.

### 1.6 Partial moves — allowed, but only statically

Moving a **single field** out of a struct (or one element out of an array by a constant index) is permitted **when the type has no `drop` hook**. The remaining fields are still owned and are destroyed individually at scope end; the moved-out field is not.

Two hard restrictions keep this implementable without runtime bookkeeping:

1. **No conditional move divergence.** At every control-flow join, each place must have the **same move state** on all incoming paths. If a field is moved on one branch of an `if`, it must be moved (or the whole value consumed) on the other. This is what lets move state be a purely static, per-point fact — no drop flags in the interpreter (NN#5's spirit: the interpreter never has to decide at runtime whether to drop).
2. **No partial move out of a `drop`-hooked type.** Its `drop` needs the whole value; taking a field would leave a partially-owned value the hook cannot run on. Move the whole value or borrow into it.

Rejected stricter rule (no partial moves at all) and looser rule (drop flags) are discussed in §10.

---

## 2. Borrowing — the second gear

### 2.1 Two borrow kinds and their expressions

Borrows are produced by keyword operators on places:

- `read place` → a **shared borrow** of the place. Type: `& T` written `borrow T`. Read-only, aliasable, `copy`.
- `write place` → an **exclusive borrow** of the place. Type: `borrow_mut T`. Read-write, unique, **moves** (not `copy`). Requires the place to be mutable (a `let mut` local, or reachable through an existing exclusive borrow).

Dereference: `deref b` is a **place** denoting the borrowed storage. On the right of `=` it reads (copying if `copy`, or serving as a place to re-borrow); on the left it writes: `deref b = v` (only if `b` is exclusive).

A borrow of a place through another borrow is a **reborrow**: `write (deref b)` where `b` is exclusive yields a fresh exclusive borrow constrained to not outlive `b`. Reborrows are how borrows are passed down the call stack.

### 2.2 The aliasing rule (XOR)

At every program point, for every place, the set of **live** borrows (§2.3) reaching it must satisfy:

> **Either** any number of shared borrows, **xor** exactly one exclusive borrow — never both, never two exclusives.

Consequences enforced by the checker:
- While an exclusive borrow of a place is live, the place may be accessed **only** through that borrow (no direct read, no direct write, no second borrow).
- While any shared borrow of a place is live, the place may be read directly and re-shared, but **not written** and **not exclusively borrowed**.
- Borrowing overlapping places (e.g. `p` and `p.f`) conflicts by the same rule; disjoint fields (`p.f` and `p.g`) do **not** conflict — the checker tracks borrows at place granularity, including distinct fields, but treats any index `a[i]` as covering the whole array `a` (no index-sensitive disjointness in the prototype; §10).

### 2.3 Borrow duration — non-lexical, body-local, aggressive (NLL-lite)

Borrow scope is **not** tied to lexical blocks. A borrow's **live range** is inferred body-locally by liveness, and the aliasing rule is checked over live ranges. Concrete, implementable discipline:

1. **Build the CFG** of the function body (basic blocks, edges for `if`/`match`/`loop`/`break`/`return`).
2. **Reference liveness (backward dataflow).** A borrow value `b` (and any borrow reborrowed from it) is **live** at a point `P` if there is a path from `P` to a *use* of `b` — a `deref`, a pass-by-borrow, a store of `b`, a reborrow of `b` — that does not first pass through a redefinition of the binding holding `b`. Standard backward liveness; converges in one worklfrom pass over a finite lattice.
3. **Loan set.** Each borrow expression creates a **loan** on the place it borrows, tagged shared/exclusive. The loan is *in scope* exactly over the live range of the borrow value(s) carrying it (including reborrows, whose loans extend the parent's obligation).
4. **Conflict check.** At every point, for every place, if the in-scope loans violate the XOR rule (§2.2), reject with a diagnostic naming both loans, their creation sites, and the point of conflict (P4: full provenance).

This is Rust-style NLL restricted to intra-procedural, **lifetime-variable-free** region reasoning: because nothing crosses a signature by inference (NN#17), there is no cross-function region unification, no lifetime solver — just liveness plus a conflict scan. It is small to implement and it is *aggressive* in exactly the way P12 demands ("infer as aggressively as soundness allows within a body").

**Why NLL and not lexical scoping (justification for a prototype).** Lexical borrows produce *false* conflicts the real language would not (a borrow held to end-of-block that is actually dead after its last use). Those false conflicts push authors toward extra blocks, clones, or valves — inflating exactly the metrics Bet 5 measures (annotation density, valve frequency). A prototype that used lexical borrows would make the value-first model look *more annoying* than the real Candor, biasing the measurement against the bet. NLL-lite costs one liveness pass and removes that bias. This is the clearest case of design choice (a) from the scope note.

**Accepted prototype limitation:** no *two-phase* borrows. A pattern like `push(write v, read v[0])` — reserving an exclusive borrow while a shared borrow is briefly used to compute an argument — is rejected. This can produce false positives on nested method-call-shaped code. It is accepted for the prototype (documented so the basket authors write around it) rather than implemented, because two-phase borrows add a reservation state to the loan lattice that is not worth the prototype's budget. If the basket shows this friction is frequent, that is a finding, not a silent tax.

### 2.4 No cross-signature inference

Region relationships that a caller must know are **written in the signature** (§3.3). The body may infer everything about borrows *inside* itself; the signature reveals everything about borrows *crossing* it. This is P2/NN#17 applied to lifetimes: the reviewer is not the compiler.

---

## 3. Signatures

### 3.1 Parameter-passing modes (the caller-visible set)

A parameter is written `name: MODE Type`. There are **four** modes. **Omitting the mode means `take`** — the value gear is the syntactic default, the borrow gears wear keywords. This ordering *is* the value-first bet made visible in every signature (P12).

| Mode | Keyword | Caller sees | Callee gets | Ownership |
|------|---------|-------------|-------------|-----------|
| by value (default) | `take` (or omitted) | its argument **moved in** (or copied, if the type is `copy`) | an owned value it may mutate, move, or drop | transferred **in** |
| shared borrow | `read` | argument **borrowed shared**; unchanged, still owned by caller | a `read`-borrow: may read, may re-share, may not mutate or move | borrowed |
| exclusive borrow | `write` | argument **borrowed exclusively**; still owned by caller, untouchable during the call | a `write`-borrow: may read and mutate through it, may not move the pointee out | borrowed |
| out / init | `out` | a place it **owns but has not initialized**; after the call it is initialized | a slot it **must fully initialize before normal return** and **may not read before initializing** | caller keeps ownership; callee fills it |

Return values use `-> T` and **move out** (RVO is a permissible interpreter optimization but is semantically a move, never a hidden copy).

`out` rules (checker obligations, reusing definite-assignment analysis, §7.4): on every normal-return path the out-place is definitely assigned exactly once and never read before assignment; if the function **faults** before assigning, the slot stays uninitialized and is **not** dropped by the caller (the caller's definite-assignment state for that place is "still uninitialized," so scope exit does not drop it). `out` exists because in-place initialization of a caller-owned slot — a device-state struct, a freshly allocated node — is a real systems pattern that the "no uninitialized reads" rule (NN#5) otherwise makes clumsy; it reuses machinery the checker needs anyway.

Why these four and not more/fewer: §10.1.

### 3.2 Effects on signatures

The only tracked effect in the prototype is **allocation** (§6.3). A function that may allocate is written `fn f(...) alloc -> T`. The marker is an upper bound (P2: signatures may overstate, never understate). A non-`alloc` function may not call an `alloc` function; `alloc` functions may call anything. This is the one-way partition with a universal ground floor (P2), in minimal form. No other effect exists in the prototype (§9).

### 3.3 Returning a borrow — regions with compact defaults

When a function returns a borrow, the caller must know **which input** that borrow came from, without inference (NN#17). The scheme:

- **Region variables** are declared in brackets after the function name and attached to borrow parameters and borrow returns: `fn pick[r](a: read[r] Slice, b: read Slice) -> read[r] Elem`. The return borrows *from `a`'s region*; `b` is unrelated. The checker verifies, body-locally, that the returned borrow's **provenance** (the place it ultimately derives from) is reachable through the `r`-tagged parameter.
- **Compact default (the common case stays clean):** if a function has **exactly one** borrow parameter and returns a borrow, and no region variables are written, the returned borrow is **defined** to derive from that sole borrow parameter. This is a *syntactic* rule the caller applies by inspection — not inference across the signature. Example: `fn first(s: read Slice) -> read Elem` needs no brackets; the return borrows `s`.
- **The rare case carries the weight:** if a function has **two or more** borrow parameters and returns a borrow, region variables are **mandatory** — there is no default, and the checker rejects an unannotated borrow return. Ambiguity is never silently resolved.

This is P12's "compact defaults so the rare signature carries annotation weight," made concrete: one-borrow-in/one-borrow-out (the overwhelmingly common shape, e.g. every slice accessor) is annotation-free; multi-source returns pay for their complexity.

A returned borrow whose provenance is a **local** (not any input region) is rejected: a borrow may not outlive the body it was born in. This is checked body-locally because provenance is tracked within the body and the signature declares the only legal escape region.

### 3.4 Struct fields holding borrows — **disallowed**

A struct or enum field **may not** have a borrow type (`read`/`write` borrow, `slice`, `slice_mut`). Owned values and `rawptr T` fields are allowed; borrows are not. (Slices are borrows, so they are covered by this ban.)

This is the single most consequential decision in the document. Its reasons and its effect on the basket:

- **Checker simplicity.** Borrow-typed fields would require *lifetime-parameterized types*: every struct that stores a borrow gets a region parameter, and every use of that struct in a signature must name the region. That is precisely the annotation-heavy machinery whose *absence* is the value-first bet. Allowing borrow fields would drag region annotations across the whole surface, contradicting P12's premise and multiplying checker complexity (region parameters on types, variance, well-formedness) far beyond a prototype's budget.
- **Measurement honesty (the decisive reason).** Pointer-rich structures — intrusive lists, free lists, graphs with back-edges — are exactly where Bet 5 is stressed. If the prototype let them be built from *safe borrow fields hidden behind type lifetimes*, the valve would be **invisible** and the measurement would understate how often real systems code reaches for pointers. By banning borrow fields, every stored inter-object reference must be either (a) an **owning** relation (`Box`, nested value), (b) a **handle/index** (a plain `u32`/`usize` into some slice or arena — safe, `copy`, the value-gear idiom for graphs), or (c) a **`rawptr`** (the valve, `unsafe` to create/deref). The author must *choose visibly*, and the reviewer (and the Bet 5 metric) sees exactly which. This is design choice (b) from the scope note: refuse the convenience that would hide a valve.

Borrows remain fully first-class as **parameters, locals, and return values** — they are a gear for *passing and computing*, not for *storing*. "Second gear, not a storage class" is the discipline.

Consequences per basket program are worked in §11; in short: the intrusive scheduler and the allocator's free list land on `rawptr` (valve, correctly visible); the arena's back-references land on indices (safe value gear); the parser's AST lands on `Box`/owned (safe value gear).

---

## 4. Pressure valves

### 4.1 `unsafe` regions

```
unsafe "reason this is sound" {
    // raw-pointer operations permitted here
}
```

The **justification string is mandatory and must be a non-empty string literal** (P1: unsafe carries a stated justification; the checker enforces presence, not truth). The checker records every unsafe region with its justification and source span so the toolchain can answer "show me everything this program trusts" (P17's audit posture, in prototype form). `unsafe` is a block, not a modifier, so it is greppable and its extent is syntactic.

`unsafe` grants exactly one new power: **raw-pointer operations** (§4.2). It does *not* disable move checking, borrow checking, overflow checking, or bounds checking on safe values — those still apply to everything that is not a raw-pointer operation. The safe/unsafe boundary is narrow by construction.

### 4.2 Raw pointers

Type: `rawptr T`. A raw pointer is an address; it may be null and is not tracked by the borrow checker or the ownership system. **Holding, moving, copying, and comparing a `rawptr` value is safe** (it is a `copy` scalar, and a `rawptr` field in a struct is inert in safe code). Everything that **creates, offsets, casts, or dereferences** one requires an `unsafe` region. This draws the audit line cleanly: *every* line that gives a raw pointer meaning is inside an `unsafe` block; safe code may shuffle addresses around but can never act on them.

Operations (all require `unsafe`):

| Operation | Signature | Meaning |
|-----------|-----------|---------|
| `addr_of(place)` | `place: T` → `rawptr T` | address of a place |
| `addr_of_mut(place)` | mutable `place: T` → `rawptr T` | address of a mutable place |
| `ptr_read(p)` | `rawptr T` → `T` | bitwise read of `*p` (author guarantees `p` valid + initialized) |
| `ptr_write(p, v)` | `rawptr T, T` → `unit` | bitwise store to `*p`; does **not** drop the old value |
| `ptr_offset(p, n)` | `rawptr T, isize` → `rawptr T` | pointer arithmetic by `n` elements |
| `ptr_null()` | → `rawptr T` | null pointer of element type `T` (annotate `T` at call) |
| `is_null(p)` | `rawptr T` → `bool` | (safe to call; but only meaningful in unsafe workflows) |
| `ptr_to_addr(p)` | `rawptr T` → `usize` | address as integer |
| `addr_to_ptr(a)` | `usize` → `rawptr T` | integer to pointer (MMIO, fixed addresses) |
| `cast_ptr(p)` | `rawptr T` → `rawptr U` | reinterpret element type |
| `offsetof(Type, field)` | (compile-time) → `usize` | byte offset of a field (for `container_of`) |

`ptr_read` moving semantics: `ptr_read` yields an owned value by bit copy; the author is responsible for not creating two owners of a move-only value (the checker cannot help here — that is what the valve *is*). The dereference forms deliberately do **not** drop or move-track, because the whole point of the valve is that the author has taken over that responsibility, visibly.

This set is exactly what the allocator (free-list splice), the scheduler (`container_of`, doubly-linked splice), and the MMIO driver (fixed-address volatile access) need — no more. `is_null` is callable from safe code so that a `rawptr` returned across an unsafe boundary can be null-checked without re-entering `unsafe`.

### 4.3 No checked-runtime interior-mutability type (RefCell-analog) — and why

The prototype ships **exactly one valve: `unsafe` + raw pointers.** It deliberately omits a checked runtime shared-mutability cell (a `RefCell`-analog that faults on a dynamic borrow-rule violation).

Defense (this is a measurement decision, not just a scope cut):

- **The basket does not need it.** Interior mutability buys safe *aliased mutation*. Of the five programs, the two that alias mutable state — the free-list allocator and the intrusive scheduler — do so through genuinely pointer-shaped structures (a free list is a linked list threaded through the free blocks themselves; an intrusive node is reachable from two links at once). A `RefCell` does **not** help those: they are `rawptr` structures in any language, and would be `unsafe` in the real Candor too. The arena's back-references use **indices**, which are safe *without* interior mutability. So omitting the cell does **not** push any basket program from safe into `unsafe` — the safe alternative for the one case it might serve (graph back-edges) is the index idiom, which we have.
- **One valve keeps the Bet 5 signal clean.** Bet 5 measures *how often* authors reach for a valve. Two valves (unsafe-pointer and checked-cell) would split that signal — a reviewer counting "escapes from the safe gears" would have to sum two categories with different costs. A single valve gives one unambiguous frequency. This is a positive measurement argument for omission, not merely an argument that omission is harmless.
- **P6 budget.** With no generics, a cell would be a special-cased built-in type carrying a runtime borrow flag and a fault path — real implementation surface for zero basket benefit.

Named residual risk: if a *future* basket program legitimately needs safe aliased mutation over non-pointer data (e.g. an observer graph), its absence would force that program into `unsafe` and **overstate** valve frequency. The prototype's five programs do not include such a case; if the basket is extended and one appears, adding the cell is the correct response and this decision should be revisited (it is recorded in §10.3 as the alternative most likely to be wrong).

---

## 5. Slices and arrays

### 5.1 Arrays

`[N]T` is a fixed-size, compile-time-constant-length, contiguous owned block of `N` values of `T`. It is an owned value: it moves, or copies iff `T` is `copy` (§1.3). It is dropped element-wise, high index to low (§1.5). `a[i]` is a place; construction is `[e0, e1, ...]` or a repeat form `[e; N]` (which requires `e` to be `copy` or evaluates it once and... — for the prototype, `[e; N]` requires `T: copy` and copies `e` into each slot).

### 5.2 Slices

A slice is a **borrow** of a contiguous run of `T` — conceptually `(pointer, length)`, but the pointer is a safe borrow, not a `rawptr`. Two kinds, mirroring the two borrow gears:

- `slice T` — a **shared** slice. `copy`, aliasable, read-only. A shared borrow of a run.
- `slice_mut T` — an **exclusive** slice. Moves, unique, read-write. An exclusive borrow of a run.

Slices obey the borrow rules of §2 against the array/allocation they view (taking a `slice_mut` of an array is an exclusive loan on the array). Because slices are borrows, **they may not be struct fields** (§3.4) and they follow NLL live ranges (§2.3).

Operations: `slice_of(place)` borrows a whole array; `subslice(s, lo, hi)` reborrows `[lo, hi)` (bounds-faulting if `lo > hi` or `hi > len`); `len(s) -> usize`; `s[i]` is a place (read for `slice`, read/write for `slice_mut`). String and byte literals produce `slice u8` viewing read-only static storage (§8.4).

### 5.3 Bounds checks fault (P5)

Every `a[i]`, `s[i]`, and `subslice` range is checked at runtime: `0 <= i < len` (and `lo <= hi <= len`). A violation **faults** (§7). There is no unchecked indexing outside a valve, and even inside a valve indexing is via raw pointers, not slice indexing — so slice/array indexing is *always* bounds-checked (NN#1: no side door).

---

## 6. Allocator surface

Heap allocation happens **only** through an explicitly passed allocator (P9/NN#7), and any function that may allocate says so with the `alloc` effect (§3.2). Because the prototype has no generics or traits, the allocator interface is a concrete, compiler-known shape.

### 6.1 The allocator interface

An allocator is a small **copy handle**:

```
struct AllocVtable {
    alloc: fn(ctx: rawptr u8, size: usize, align: usize) -> rawptr u8,   // null == out of memory
    free:  fn(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) -> unit,
}

copy struct Alloc {          // copy: it is a handle, freely passed and stored
    ctx: rawptr u8,          // opaque allocator state (inert in safe code)
    vt:  rawptr AllocVtable, // inert in safe code
}
```

`Alloc` is `copy` and its `rawptr` fields are inert in safe code (§4.2), so a handle can be passed by value, stored in a `Box`, and copied without a valve. Only the *implementation* of an allocator dereferences `ctx`/`vt` (inside `unsafe`). This is "the capability travels as an ordinary value" (P2) in concrete form.

Function pointers (`fn(...) -> ...`, with parameter modes) are the one mechanism the vtable needs; they may not capture (no closures), so a vtable entry is always a top-level function. Calling through a function pointer is an ordinary call for effect/borrow purposes.

- **To BE an allocator** (the allocator basket program): define `ctx` state, write `alloc`/`free` functions matching the vtable signatures, and build an `Alloc { ctx: addr_of_mut(state), vt: addr_of(MY_VTABLE) }` (the `addr_of`s are in `unsafe`). §11.1.
- **To USE an allocator:** call the safe surface below.

### 6.2 The safe allocation surface: `Box T`

Raw `alloc`/`free` return `rawptr u8` — untyped, uninitialized, valve territory. So that *using* an allocator is not itself a valve (which would overstate valve frequency, §0), the prototype provides one compiler-known owning heap type, `Box T` (the *only* heap-owning abstraction; no `Vec`, no `Rc`):

- `box(a: read Alloc, v: T) -> BoxResult T` — allocates `sizeof(T)`/`alignof(T)` through `a`, moves `v` into it, returns `boxed(Box T)` on success or `oom` on allocation failure. `BoxResult T` is a compiler-known sum `enum BoxResult { boxed(Box T), oom }` (allocation failure is a **value**, P7/NN#8 — never a fault). `box` is `alloc`-effecting.
- `Box T` **owns** its pointee and its originating `Alloc` handle (a `copy` value, so no borrow field — consistent with §3.4). Dropping a `Box T` drops the pointee (running its `drop`, §1.5) and then calls `free` through the stored handle. Deterministic, RAII-style.
- `deref` on a `Box T` yields the pointee place (read via `read`-borrow, write via `write`-borrow). Moving the pointee out is `unbox(b: Box T) -> T` (frees the box storage, returns the owned pointee).

`Box`/`slice`/`[N]T` are the prototype's only parametric types, all compiler-known; user code cannot define generics (§8.3). `Box` is what makes the parser's owned AST safe (§11.4) without exposing generics.

### 6.3 The tracked allocation effect (minimal)

- The effect is a single boolean per function signature: `alloc` present or absent.
- Any call to `box`, any `clone` of a box-bearing type, and any call to an `alloc`-marked function makes the enclosing function `alloc`-marked. Calling a vtable's `alloc`/`free` function pointer is `alloc`-marked as well.
- A non-`alloc` function calling an `alloc` function is a **checker error** with a diagnostic tracing the allocation to its source (P4). The ground floor — allocation-free code — is callable from everywhere (P2).
- Effects are upper bounds: a function may be marked `alloc` and never allocate (permitted conservatism); it may not allocate while unmarked (NN#19).

This is the tracked-effect set at its minimal, honest core: one effect, one-way partition, capability-as-value. No foreign-trust effect exists because the prototype has no FFI (§9).

---

## 7. Faults in the prototype

### 7.1 The interpreter faults precisely — and that is sound

The interpreter delivers every fault **immediately, at the faulting operation** (a zero-width fault window). This is a **sound refinement** of P5's imprecise-but-inescapable semantics: P5 says a fault is delivered *no later than* the next synchronization or observable effect; delivering it *immediately* is always within that bound, and produces a strict subset of P5-legal observable behaviors. The prototype has **no optimizer**, so it never exercises the reordering freedom the fault *window* exists to permit — it deliberately does not test that part of the model. A future optimizing implementation may widen the window up to P5's bound; the prototype's precise trapping will remain one of the legal behaviors, so programs validated here stay valid there.

### 7.2 What faults

- **Integer overflow** (default arithmetic regime): `+ - * /` and negation fault on overflow. Division/remainder by zero faults.
- **Bounds violations** (§5.3): array/slice index and subslice range out of bounds.
- **Contract/assert failures** (§7.3), if the construct is used.
- **Explicit panic**: `panic("msg")` faults unconditionally.

Raw-pointer dereferences inside `unsafe` are **not** checked (that is the valve's meaning); a bad `ptr_read` is the author's declared responsibility, not a defined fault.

### 7.3 Contracts, minimal

The prototype includes only the **`enforced`** level (P8), because audit and assumed-proven do not affect the memory-model measurement:

- `assert(cond);` — faults if `cond` is false.
- Function clauses: `fn f(...) requires(cond0) ensures(cond1) -> T { ... }`. `requires` is checked at entry (faults on violation), `ensures` at each normal return (`cond1` may reference the return value via the keyword `result`). These are thin sugar over `assert` at entry/exit; they are dynamically checked and **never** license any optimizer assumption — a matter of principle even though the prototype has no optimizer (P8, kept true so the basket's contracts mean the same thing in a later implementation).

Included because the allocator's invariants (a freed block's size matches its allocation) and the MMIO state machine's legal transitions are naturally expressed as contracts, giving the fault vocabulary (P7) something real to unify, and giving the basket a correctness oracle (P8's rationale). Audit and assumed-proven are omitted (§9).

### 7.4 Fault policy and definite assignment

- **Root fault policy:** the prototype's single policy is **abort** — on any fault the interpreter halts and emits a structured fault report (kind, source span, value context) as machine-readable data (P4/P7). No unwinding, no drops run after a fault (truncated execution, P5). This is the root-declared policy in its simplest form; the prototype does not exercise halt-and-log or custom handlers.
- **Definite-assignment analysis** (serving NN#5): a forward dataflow pass proves every place is initialized before it is read. Locals may be declared then assigned later (`let x; ... x = e;`) provided every path to a use assigns first; joins require agreement (a place is "definitely initialized" only if initialized on all incoming edges). This same pass discharges `out`-parameter obligations (§3.1) and interacts with move state (§1.6): a moved-out place is "uninitialized" until reassigned. **No value is ever read while uninitialized, ever** (NN#5) — this is a compile-time guarantee, not a runtime check.

---

## 8. Type-system minimum

### 8.1 Scalars and the ground types

- Integers: `i8 i16 i32 i64 isize` (signed) and `u8 u16 u32 u64 usize` (unsigned). Sized and target-defined widths for `isize`/`usize` (queryable per target, P5). Overflow-checked by default (§7.2).
- `bool`, `unit`.
- Arithmetic regimes: default checked; `wrapping { ... }` and `saturating { ... }` scoped blocks change the overflow behavior of arithmetic *inside the block only*, greppably (P5). **Unchecked arithmetic does not exist outside a valve** (NN#4) — and the prototype provides no unchecked-arith operation at all; overflow is either checked, wrapped, or saturated. Both block forms are trivial for the interpreter (they select the overflow rule) and are included because P5 mandates their existence and greppability.

### 8.2 Aggregates and control

- `struct Name { f0: T0, f1: T1, ... }` — nominal, named fields. Optional `copy` marker (§1.3) and optional `drop` hook (§1.5).
- `enum Name { V0(T), V1, V2(A, B), ... }` — nominal tagged sum, variants with zero or more payloads. **Pattern matching** via `match e { case V0(x) => ..., case V1 => ..., case V2(a, b) => ... }`; matches must be exhaustive (checker-enforced). Sum types are the mechanism for errors-as-values (P7): a fallible function returns `enum ...Result { ok(T), err(E) }` and the caller `match`es. (A lightweight `?`-style propagation operator is a syntax convenience the throwaway prototype omits; `match` suffices for measuring the model.)
- Functions `fn` (§3) and non-capturing function pointers (§6.1).
- Control: `if/else`, `match`, `loop { ... }` with `break`/`continue`, `while cond { ... }`, `return`. One loop family; the canonical-form question (P3) is out of scope for a throwaway grammar.

### 8.3 No user-defined generics

User code cannot declare generic types or functions. The only parametric types are the compiler-known `[N]T`, `slice T`, `slice_mut T`, `rawptr T`, `Box T`, and `BoxResult T`. Justification: the five basket programs need containers only in the shapes these builtins already provide — the allocator uses raw memory; the scheduler uses concrete `Node` structs; the MMIO driver uses concrete register/state structs; the parser builds concrete AST `enum`s held in `Box`; the arena stores concrete nodes and hands out borrows by index. **No basket program requires a user-defined generic container.** Adding definition-site generics (P11) is real, load-bearing complexity (interface bounds, coherence) that the prototype spends its budget avoiding (P6, §8 "keep the prototype small"). If a basket program *did* need one, that would be evidence to add them — the basket does not, and that absence is itself a small datum for Bet 5.

### 8.4 Text: byte slices only

There is **no string type**. Text is `slice u8` (borrowed) or `[N]u8`/`Box`-of-bytes (owned). String literals produce `slice u8` viewing read-only static storage. This follows P3's explicit warning that "one string type" collides with the freestanding/allocator-free reality, and refuses to prejudge the text-budget question inside a throwaway prototype. The parser basket consumes `read slice u8`; that is the whole text story.

---

## 9. What the prototype deliberately omits

Each omission with one line on why it does not invalidate the Bet 5 measurement (which is about *value-vs-borrow-vs-valve ergonomics on single-threaded systems code*):

- **Concurrency and data-race machinery (P10).** The interpreter is single-threaded; the value/borrow/valve ergonomics being measured are orthogonal to threading, and adding concurrency would only add noise to the metric.
- **All tracked effects except `alloc` (P2).** No foreign-trust effect because there is **no FFI/boundary modules (P17)** in the prototype — the basket is pure Candor, so the foreign audit surface is empty and untested here.
- **Contract levels `audit` and `assumed-proven` (P8).** Only `enforced` is kept; the other levels change *what a violation does*, not *what the memory model expresses*, so they are irrelevant to Bet 5.
- **Generics, traits/interfaces (P11).** §8.3 — the basket doesn't need them; keeping them out preserves the small core and does not constrain the value/borrow patterns under test.
- **The `?`-propagation sugar and rich pattern syntax.** `match` covers the semantics; sugar affects token count, not the model, and would only muddy an already-throwaway syntax.
- **Modules beyond a single compilation unit.** The prototype is single-file; nothing in the memory model depends on module boundaries (signatures are already fully explicit, so multi-file would be trivial and adds nothing to measure).
- **Comptime beyond constant array sizes and `offsetof` (P6).** No general compile-time execution; the model under test needs none.
- **The imprecise fault window and any optimizer (P5).** §7.1 — precise trapping is a sound subset; the window exists to enable optimization the prototype does not perform, so not exercising it is correct, not a gap.
- **Interior-mutability cell (RefCell-analog).** §4.3 — omitted with a measurement argument, not just a budget cut.
- **A second string/owning-text type (P3).** §8.4 — byte slices suffice for the basket.

None of these omissions touch the question Bet 5 asks. The two that a careful reader might worry *do* touch it — generics and the interior-mutability cell — are argued at their point (§8.3, §4.3) to be absent from the basket's needs, which is why their omission does not bias the frequency-of-valves measurement.

---

## 10. Rejected alternatives (the rationale is the product — §8.6)

### 10.1 Passing-mode set

- **Rejected: a separate `by-copy` mode.** Redundant: whether `take` copies or moves is already determined by the type's copyability (§1.3), and copyable types are exactly the cheap ones, so a distinct keyword would add ceremony without new information (P13).
- **Rejected: only `take`/`read`/`write` (no `out`).** In-place initialization of a caller-owned uninitialized slot is a real systems pattern that NN#5 otherwise makes clumsy, and `out` reuses the definite-assignment machinery the checker needs anyway (§7.4). The marginal cost is low; the expressiveness (device-state init, node init) is real. Kept.
- **Rejected: making mode mandatory on every parameter (no default).** That would put a keyword on every by-value scalar, taxing the *common* case — the opposite of P12's "compact defaults so the rare thing carries weight." Defaulting the omitted mode to `take` makes value-passing clean and forces the *borrow* gears to wear the keyword, which is the value-first ordering rendered in syntax.
- **Rejected: a `move`/`copy` distinction at the *call site* (Rust-style explicit `move`).** The type already determines it; a call-site keyword would be redundant with the type and violate "one canonical way" (P3).

### 10.2 Borrow discipline

- **Rejected: strictly lexical (scope-based) borrows.** Simpler to implement, but produces false conflicts the real language would not, pushing authors to extra blocks/clones/valves and **biasing the Bet 5 metrics against the value-first model** (§2.3). Measurement validity outranks the small implementation saving.
- **Rejected: full Polonius-style / cross-function region inference.** Overkill for a prototype and, more importantly, cross-signature inference is forbidden by NN#17 regardless. Body-local NLL with no lifetime variables is the sweet spot: aggressive inside the body (P12), nothing inferred across the signature.
- **Rejected: implementing two-phase borrows.** Adds a reservation state to the loan lattice for a class of false positives that the basket authors can write around; deferred as a documented limitation (§2.3) rather than spent on.

### 10.3 Valve set

- **Rejected: shipping a checked interior-mutability cell alongside `unsafe`.** §4.3 — two valves split the Bet 5 signal, the basket does not need it, and P6. **This is the decision most likely to be wrong** (recorded as such): if an extended basket needs safe aliased mutation over non-pointer data, the cell should be added and this revisited.
- **Rejected: `unsafe` as a per-expression modifier instead of a block.** A block is greppable with a bounded extent and forces the justification string to attach to a reviewable region (P1/P17); a per-expression modifier scatters the audit surface.
- **Rejected: making `rawptr` deref safe (Zig-style) with the unsafety only at construction.** Would move the audit line to construction and let dangerous derefs hide in safe code; drawing the line at *any operation that gives a pointer meaning* keeps "every meaningful pointer act is inside `unsafe`" true and the audit trivial.
- **Rejected: no valve at all (pure safe subset).** The allocator and intrusive scheduler are *provably* inexpressible without raw pointers; a valve is mandatory for the basket. The design choice is *which* valve and *how visible*, not *whether*.

### 10.4 Struct fields holding borrows

- **Rejected: allowing borrow-typed fields (with type lifetime parameters).** §3.4 — it would (a) drag region annotations across the whole surface, contradicting the value-first premise and exploding checker complexity, and (b) **hide** the pointer-graph valve behind lifetimes, corrupting the very measurement the prototype exists for. Banning borrow fields both simplifies the checker and *sharpens* the Bet 5 signal by forcing an author to choose visibly among owning / index / `rawptr`.
- **Rejected: allowing borrow fields only in non-escaping local structs.** The "does this struct escape" analysis is itself non-local and fragile; the clean, greppable rule "borrows are for passing and computing, never for storing" is worth more than the convenience.

### 10.5 Copyability

- **Rejected: automatic structural copy for all-`copy`-field structs (Rust-derive-by-default feel).** Adding a field could silently flip a type's move/copy behavior and change move-checking far away (P2 locality). Opt-in `copy` marker keeps that a local, greppable decision.
- **Rejected: no implicit copy at all, even for scalars.** Would force `clone` on every `i32` use — absurd ceremony for the truly free case (P13). Scalars, `copy`-marked structs, and shared borrows copy implicitly; everything else moves.

### 10.6 Partial moves

- **Rejected: forbidding partial moves entirely.** Real code moves one field out of a struct; forbidding it forces clones or restructuring, biasing the model toward looking clumsier than the real language.
- **Rejected: allowing conditional partial moves via runtime drop flags.** Adds hidden runtime bookkeeping to the interpreter and hidden control flow to drop (P4/P9 forbid invisible control flow/cost). The static rule (§1.6 — move states must agree at joins; no partial move out of `drop`-hooked types) keeps drop fully static and predictable.

### 10.7 Allocator surface

- **Rejected: a built-in global allocator / implicit `new`.** Violates NN#7 (no hidden allocation) and P9 (explicit allocators). Allocation must be an explicitly threaded value.
- **Rejected: a trait-based `Allocator` interface.** No traits in the prototype (§8.3); the concrete `Alloc` vtable-handle achieves polymorphism-by-value without the definition-site-generics machinery.
- **Rejected: only raw `alloc`/`free`, no `Box`.** Would make *every* heap use a valve (untyped `rawptr` + `unsafe` init), **overstating** valve frequency and biasing Bet 5 (§6.2). `Box` gives safe typed heap ownership so that "uses an allocator" is a value-gear act while "implements an allocator" is a valve act — which is the honest split.

---

## 11. Worked examples — the basket's hardest patterns

Each sketch shows the hardest pattern that program needs, in throwaway syntax, and names the gear. Where a valve is forced, it is said plainly — Bet 5's measurement depends on valves being **visible, not hidden**.

### 11.1 Allocator — free-list splice (**valve**)

Hardest pattern: threading a singly-linked free list *through the free blocks themselves*, i.e. writing/reading a `next` pointer into memory the allocator does not own as typed values.

```
copy struct Alloc { ctx: rawptr u8, vt: rawptr AllocVtable }

struct Pool { head: rawptr u8, block_size: usize }   // head: first free block, or null

// vtable functions (top-level, match the fn-pointer signatures)
fn pool_alloc(ctx: rawptr u8, size: usize, align: usize) -> rawptr u8 {
    unsafe "pool owns [ctx..); head chains free blocks each >= block_size" {
        let p: Pool = ptr_read(cast_ptr(ctx));       // load pool state
        if is_null(p.head) { return ptr_null(); }    // OOM is a value (null), not a fault
        let block: rawptr u8 = p.head;
        let next: rawptr u8 = ptr_read(cast_ptr(block));   // *block == next free
        ptr_write(cast_ptr(ctx), Pool { head: next, block_size: p.block_size });
        return block;
    }
}

fn pool_free(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) -> unit {
    unsafe "push freed block onto head; block is >= pointer-sized" {
        let p: Pool = ptr_read(cast_ptr(ctx));
        ptr_write(cast_ptr(ptr), p.head);            // *block = old head
        ptr_write(cast_ptr(ctx), Pool { head: ptr, block_size: p.block_size });
    }
}
```
**Gear: valve.** The free list is a `rawptr`-threaded structure; §3.4 correctly refuses to let it be safe borrow fields, so it lands in `unsafe`, visibly. OOM is a `rawptr` null return checked safely by the caller via `Box`/`is_null` — a *value*, per P7 (§6.2). This is a valve that is *critical in function but confined in occurrence* — exactly Bet 5's shape.

### 11.2 Intrusive-list scheduler — doubly-linked splice + `container_of` (**valve, the design's hardest case**)

Hardest pattern: a task is enqueued by embedding a link node inside it; removal must recover the task from the interior link pointer (`container_of`) and splice a doubly-linked list.

```
struct Link { next: rawptr Link, prev: rawptr Link }   // rawptr fields: inert in safe code
struct Task { link: Link, prio: u8, /* ... */ }

fn list_remove(n: rawptr Link) -> unit {
    unsafe "n is linked; neighbors are valid nodes of this list" {
        let nn: Link = ptr_read(n);
        let prev: rawptr Link = nn.prev;
        let next: rawptr Link = nn.next;
        // prev.next = next ; next.prev = prev
        let pv: Link = ptr_read(prev);
        ptr_write(prev, Link { next: next, prev: pv.prev });
        let nx: Link = ptr_read(next);
        ptr_write(next, Link { next: nx.next, prev: prev });
    }
}

fn task_of(link: rawptr Link) -> rawptr Task {
    unsafe "link is the `link` field of a live Task (container_of)" {
        return cast_ptr(ptr_offset(cast_ptr(link) as rawptr u8,
                                   0 - (offsetof(Task, link) as isize)));
    }
}
```
**Gear: valve.** A node reachable from two links at once cannot be an exclusive borrow (aliasing rule, §2.2) and cannot be a safe borrow field (§3.4). `container_of` needs `offsetof` + pointer arithmetic. **This program stressed the design most** — see the summary. It validates three decisions at once: `rawptr` fields must be *allowed but inert in safe code* (§4.2), `offsetof`/`cast_ptr`/`ptr_offset` must exist as valve intrinsics, and the safe gears must *not* pretend to express this (which is why banning borrow fields is right, not restrictive).

### 11.3 MMIO driver state machine — safe logic, valve I/O (**hybrid: value gear + valve**)

Hardest pattern: a state machine whose *logic* is a safe sum type but whose *effects* are volatile reads/writes to a fixed device address.

```
enum State { idle, arming, running, faulted }
struct Uart { base: usize, state: State }   // base: fixed MMIO address

fn reg_write(base: usize, off: usize, val: u32) -> unit {
    unsafe "device MMIO window at [base..base+0x40); volatile store" {
        ptr_write(addr_to_ptr(base + off) as rawptr u32, val);   // one valve op
    }
}
fn reg_read(base: usize, off: usize) -> u32 {
    unsafe "device MMIO window; volatile load" {
        return ptr_read(addr_to_ptr(base + off) as rawptr u32);
    }
}

fn step(d: write Uart, ev: Event) -> unit {
    // safe value gear: the whole transition table is a plain match on a sum type
    let s: State = match ev {
        case Event.start => State.arming,
        case Event.ready => State.running,
        case Event.err   => State.faulted,
        case Event.stop  => State.idle,
    };
    match s {
        case State.arming  => reg_write((deref d).base, 0x00, 1),   // valve escapes only here
        case State.running => reg_write((deref d).base, 0x04, 1),
        case State.faulted => reg_write((deref d).base, 0x00, 0),
        case State.idle    => reg_write((deref d).base, 0x00, 0),
    };
    (deref d).state = s;   // exclusive borrow, safe
}
```
**Gears: value + borrow for the machine, valve for the register poke.** The state (`enum`), the transition table (`match`), and the driver mutation (`write Uart`) are all safe; the valve is a thin, greppable seam around the two `reg_*` functions. This is the encouraging Bet 5 case: pointer-danger is *localized to I/O*, the logic is value-first.

### 11.4 Parser — owned AST over a borrowed input (**pure value + borrow gear, no valve**)

Hardest pattern: return a freshly *owned* subtree while *borrowing* the input, with heap-owned recursive nodes — the case value semantics is supposed to fit best.

```
enum Expr { num(i64), add(Box Expr, Box Expr), mul(Box Expr, Box Expr) }

struct P { src: rawptr u8, pos: usize, len: usize }   // (throwaway cursor; src via slice below)

// returns an owned Box Expr; input slice is only read (borrow gear), never stored in the AST
fn parse_term(a: read Alloc, s: read slice u8, pos: write usize) alloc
        -> BoxResult Expr {
    let n: i64 = read_number(s, pos);          // reads through the shared slice borrow
    return box(a, Expr.num(n));                // owned node on the explicit allocator
}

fn parse_expr(a: read Alloc, s: read slice u8, pos: write usize) alloc
        -> BoxResult Expr {
    match parse_term(a, s, write (deref pos)) {   // reborrow the cursor down
        case BoxResult.oom => return BoxResult.oom,
        case BoxResult.boxed(lhs) => {
            if peek(s, pos) == PLUS {
                advance(pos);
                match parse_expr(a, s, write (deref pos)) {
                    case BoxResult.oom => return BoxResult.oom,   // lhs drops here, deterministically
                    case BoxResult.boxed(rhs) =>
                        return box(a, Expr.add(lhs, rhs)),        // move both owned children in
                }
            }
            return BoxResult.boxed(lhs);
        }
    }
}
```
**Gears: value (owned `Box` AST, moved-in children) + borrow (`read slice u8` input, `write usize` cursor).** No valve. The AST never *stores* a borrow into the input (§3.4 satisfied by owning parsed data), so no region annotation is needed — the returned `Box Expr` derives from *fresh* allocation, not from `s`, and the `read`/`write` params carry no region variables (single-borrow-return default never even triggers, because nothing borrowed is returned). Allocation is explicit and effect-marked (`alloc`, threaded `Alloc`), per P9. This is the workload Bet 5 predicts value-first fits cleanly, and it does.

### 11.5 Arena compiler pass — back-references by index, valve confined to the arena (**value + index gear; valve only inside the arena**)

Hardest pattern: an IR graph with back-edges (a node references its dominator/parent), built in an arena, walked by a pass. Back-references are the classic pointer-graph temptation.

```
enum Node { leaf(i64), binop(u8, u32, u32) }   // u32 fields are *indices*, not pointers

struct Arena { mem: Box [4096]Node, count: u32 }   // owns its backing (Box), safe

fn arena_push(ar: write Arena, n: Node) -> u32 {
    let i: u32 = (deref ar).count;
    (deref ar).mem[i as usize] = n;           // safe: bounds-checked array store
    (deref ar).count = i + 1;                 // checked add; faults on overflow
    return i;                                  // stable handle
}

fn arena_get(ar: read Arena, i: u32) -> read Node {   // single borrow param -> return borrows `ar`
    return read (deref ar).mem[i as usize];    // bounds-checked; region default: derives from `ar`
}

// a pass that follows back-references purely by index (value gear)
fn fold_consts(ar: write Arena, root: u32) -> i64 {
    match arena_get(read (deref ar), root) {   // reborrow ar as shared for the read
        case Node.leaf(v) => return v,
        case Node.binop(op, l, r) => {
            let lv: i64 = fold_consts(write (deref ar), l);   // follow child index
            let rv: i64 = fold_consts(write (deref ar), r);   // follow child index
            return apply(op, lv, rv),
        }
    }
}
```
**Gears: value + index (safe) for the whole graph; the only valve is inside the arena's *backing allocation*, not shown here — `Box [4096]Node` hides it (§6.2).** Back-references are `u32` indices: `copy`, storable in `enum` payloads (not borrow fields, so §3.4 is satisfied), and dereferenced by a safe, bounds-checked `arena_get`. `arena_get` is the textbook compact-default case: one borrow parameter, returns a borrow, no region variables needed (§3.3). This demonstrates the **safe alternative to the pointer valve** — the index/handle idiom — which is precisely the value-first move Bet 5 bets systems programmers will make when the language nudges them. The valve stays sealed inside `Box`.

---

## 12. Summary of gear placement across the basket

| Basket program | Hardest pattern | Gear it lands in | Valve visible? |
|----------------|-----------------|------------------|----------------|
| Allocator | free-list splice through freed blocks | **valve** | yes — `unsafe` free-list ops |
| Intrusive scheduler | doubly-linked splice + `container_of` | **valve** | yes — `rawptr` fields + `offsetof` |
| MMIO driver | volatile fixed-address I/O | **value/borrow + valve** | yes — thin I/O seam only |
| Parser | owned recursive AST over borrowed input | **value + borrow** | no valve |
| Arena pass | graph back-references | **value + index** (valve sealed in `Box`) | only inside the arena backing |

Two programs are pure-valve at their core (allocator, scheduler) — and they are exactly the pointer-rich structures Bet 5 predicts *concentrate* the valve while staying rare across a codebase. Two are valve-free or index-safe (parser, arena pass). One is a clean hybrid (driver). The prototype's job is to let the basket be measured; this table is the shape of the answer it will produce, made honest by refusing to hide any valve.
