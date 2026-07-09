# 0011 — The Foreign Boundary

**Status:** draft
**Date:** 2026-07-08
**Philosophy hooks:** P14 (C is first-class; the toolchain generates checked
bindings; the C ABI *is* the stable ABI; C++ is a partial neighbor), P17/NN#18
(the boundary is the audit surface — trust declared, not proven; foreign calls
only through declared boundary modules), P2/NN#19 (the closed effect set —
activating its **second and final** reserved member, *foreign trust*; upper-bound
rule; conservative-default migration), P8 (contract levels — `assumed-proven` as
the honest level for uncheckable boundary properties; `enforced` for the
checkable value subset), P1 (the `unsafe` valve — reused, not duplicated), P6
(small core; when you add, remove).

**Prior art it fills:** 0008 §4 fixed the boundary-module *marker* (file-level
`boundary`, part of the interface artifact, `candor audit --boundaries`) and
deferred the FFI *content* to this round. Spec 08-effects §6 and 05-pointers,
07-contracts hold the matching SKELETONs; spec 99 obligation **OBL-FFI** is the
acceptance gate this design discharges at the design tier.

**Revision history.** 2026-07-08 — initial draft. 2026-07-08 — revised per joint
adversarial review #1 of designs 0010/0011
(`docs/reviews/2026-07-08-design-0010-0011-review-1.md`), dispositions F7–F11: F7
the reverse direction designed as boundary `export` items (§1.5), the audit gains
an exports section (§6), C-ABI emission recorded as a 0010 forward-dependency; F8
the mappability predicate stated recursively, enum-in-struct/enum-in-array
rejected (§1); F9 extern globals deferred with the errno-accessor idiom (§7); F10
the shim/real per-symbol differential obligation recorded in both docs' seam (§5);
F11 by-value argument classification named as the backend's duty, distinct from
layout compatibility (§1, §1.4).

**Prototype status — parse/check/audit surface shipped.** The `boundary` marker,
`extern`/`trust`/`export` parsing (real front-end, boundary-file-gated, E1101),
the recursive C-mappability check (E1102), the `foreign` effect partition with the
§2 discharge rule (E1103; foreign calls reuse the `unsafe` valve), the closed
trust vocabulary (E1105/E1106), and the §6 `audit` JSON are live in the prototype;
the shim registry runs extern calls identically on the tree-walker and MIR engines,
with the `no_foreign_runtime` fault otherwise. The shim now also receives a handle
to the calling engine's flat memory, so a shim can dereference a `rawptr` argument
(the buffer half of a POSIX pointer+length signature); on this a std/io boundary
module (`sys_open`/`sys_close`/`sys_read`/`sys_write` + safe `open_read`/`read_into`/
`write_all`/`close` wrappers) runs real, deterministic file/stdout I/O on both
engines. Real C-ABI calls (including binding these externs to the actual libc
`read`/`write` symbols) and export trampolines remain the 0010 native-backend
forward dependency.

## Problem

P14 and P17 have sat as principle since v2 with no mechanism. 0008 gave them a
*home* (the `boundary` file, enumerable by one command) but left it empty:
no foreign declarations, no type mapping, no effect, no contract attachment,
no ingestion story. Spec 08 §6 says only "when FFI lands"; this is when.

The question this document resolves: **what a boundary module contains, how the
foreign-trust effect partitions the call graph, how trust is declared and
enumerated, and what the tree-walking prototype can honestly do before a native
backend exists.** It must be resolved now because it is the last of P2's two
reserved effect slots and the largest unsafe surface any real Candor program will
have (P1's rationale, P17's §7 auditor). Everything here is designed against the
prototype's *documented* layout (`interp/layout.rs`, 0001 §4.2) as a fixed input,
and against 0010 (the native backend) as a stated *forward* dependency for
actually calling C.

## Decision

### 1. What a boundary module contains

**`extern` blocks, only in `boundary` files.** A foreign declaration is written
inside an `extern "C" { … }` block, and such a block is **well-formed only inside
a file whose preamble is `boundary`** (0008 §4). Outside a boundary file it is a
hard error (proposed **E1101**). This is NN#18 made structural: the *only* place a
foreign signature can exist is the audited unit, so "safe code may reach foreign
only through a boundary module" is enforced by where the declaration is allowed to
sit, not by a whole-program scan.

```
boundary                       // file preamble (0008 §4) — the whole file is a boundary module
use core::mem::{is_null};

extern "C" {
    fn strlen(s: rawptr u8) foreign -> usize
        trust "POSIX 7 strlen: reads bytes up to a NUL, retains no pointer, no thread state" {
            valid_nul_terminated(s),   // assumed-proven: not dynamically checkable
            no_retain(s),
        };

    fn memcpy(dst: rawptr u8, src: rawptr u8, n: usize) foreign -> rawptr u8
        trust "C11 7.24.2.1: dst/src valid for n bytes, non-overlapping, unretained" {
            valid_for(dst, n), valid_for(src, n), no_overlap(dst, src, n),
            no_retain(dst), no_retain(src),
        };
}
```

The `"C"` string names the **ABI**, not a language: it selects the target's C
calling convention (§1.4). Only `"C"` is defined in this edition.

**The type-mapping table.** A foreign signature may use only C-mappable types.
The mapping is fixed by the *target's* C ABI, and per-target facts (widths,
`char` signedness, `long`) are the queryable per-target constants of P5, resolved
by the ingestion tool (§4), never guessed:

| C type | Candor type | note |
|---|---|---|
| `void` (return) | `unit` | |
| `_Bool` | `bool` | 1 byte |
| `char` | `i8` or `u8` | **target-defined signedness** (P5), resolved per target |
| `signed/unsigned char` | `i8` / `u8` | |
| `short` / `unsigned short` | `i16` / `u16` | |
| `int` / `unsigned int` | `i32` / `u32` | on all mainstream targets |
| `long` / `unsigned long` | `i64`/`u64` (LP64) or `i32`/`u32` (ILP32, LLP64) | **target-dependent**, resolved per target |
| `long long` | `i64` / `u64` | |
| `size_t` | `usize` | |
| `ptrdiff_t` / `intptr_t` | `isize` | |
| `int8_t`…`int64_t`, `uint*_t` | `i8`…`i64`, `u8`…`u64` | fixed-width: clean, target-invariant |
| `T *`, `void *`, `const T *` | `rawptr U`, `rawptr u8` | pointee mapped; `const` not modelled (§7) |
| `char *` (string) | `rawptr u8` | NUL-termination is a **contract**, not a type; wrapper converts to/from `[u8]` |
| `T[N]` (as member) | `[N]T` | identical layout (§1, layout decision) |
| `T[N]` / `T[]` (as parameter) | `rawptr T` | C array-to-pointer decay |
| `R (*)(A, B)` | `fn(A, B) foreign -> R` | function pointer carries the effect (§2, spec 08 §5) |
| `struct S { … }` | `struct S { … }` | **no `repr` attribute needed** — see below |

**The layout decision — Candor needs no `repr(C)` attribute.** The prototype's
documented struct layout (`interp/layout.rs`: *field order = declared order,
natural alignment, trailing pad to alignment, nothing reordered* — 0001 §4.2) is
**already the mainstream C struct-layout algorithm** (System V / AAPCS and kin).
Therefore a Candor `struct` of C-mappable fields is C-ABI-compatible **by
construction**, and this design adds *no layout attribute*. This is a real edge
over Rust, whose default layout reorders and so must bolt on `repr(C)` to opt back
into C order; Candor never took the reordering freedom, so it never has to buy it
back. The cost is stated plainly under Consequences: Candor forgoes
field-reordering packing forever at the struct level — but it never had it, so
this ratifies an existing property, it does not spend new budget.

**Layout compatibility is *not* argument classification — a named backend duty.**
The paragraph above establishes that a Candor struct's *in-memory layout* matches
the C struct's. That is necessary but **not sufficient** for passing a struct
**by value** across the C ABI: the platform ABI additionally specifies an
*argument-classification* algorithm (SysV AMD64 sorts each aggregate's eightbytes
into INTEGER/SSE/MEMORY and decides register-vs-stack passing; AArch64 AAPCS has
its own HFA/composite rules) that is **orthogonal to layout** — two types with
identical layout can be classified and passed differently, and getting it wrong
corrupts arguments silently. This design fixes only the *layout* half; the
*classification* half is the **native backend's duty (0010, §1.4 forward-dep)**:
the backend MUST implement the target's by-value aggregate classification when it
lowers a foreign call or a boundary export (§1.5) that passes or returns a struct
by value. Named here so it is not mistaken for something the layout decision
already bought.

**The mappability predicate, stated recursively.** A type is *C-mappable* iff:
- it is a scalar in the table above (integer / `bool` / `unit` return / `rawptr`
  / function pointer), **or**
- it is a `struct` **all of whose fields are C-mappable** (the layout decision
  below makes such a struct C-ABI-compatible by construction), **or**
- it is a fixed array `[N]T` (as a struct member) **whose element `T` is
  C-mappable**.
Everything else is **unmappable**, and because the predicate recurses through
`struct` and `[N]T`, unmappability is *contagious*: a single unmappable field or
element makes the whole aggregate unmappable. In particular a **Candor `enum`
nested in a struct** (`struct S { tag: MyEnum, … }`) and an **array of enums**
(`[N]MyEnum`) are **rejected** (E1102, the diagnostic naming the offending
field/element path), exactly as a bare `enum` is — the 8-byte payload tag
(`layout.rs` `ENUM_TAG`) has no portable C shape at *any* nesting depth, so it
cannot be laundered into C by wrapping it in a struct or an array. The same
recursion rejects a struct or array containing a `Box`, a slice, or a float
field.

**What is UNMAPPABLE and rejected** (well-formed error at the extern boundary,
proposed **E1102**, with a P4 diagnostic naming the offending type):
- **Candor `enum`s** — the interpreter tags every payload with an 8-byte prefix
  (`layout.rs` `ENUM_TAG`); no portable C type has that shape. A tagged union
  must be decomposed by hand into a C `struct { tag; union }` the author declares.
- **`Box T`** (24-byte `{ptr, ctx, vt}`) and **slices** (16-byte `{ptr, len}` fat
  pointers) — Candor abstractions with no C counterpart. A boundary passes their
  *parts*: a `Box`'s pointer as `rawptr T`, a slice as `(rawptr T, usize)`.
- **`float` / `double` / `long double` / `_Complex`** — the scalar set of this
  edition has **no floating type** (0001 §5). Unmappable until floats land;
  recorded as a forward obligation, not finessed.
- **Variadic functions** (`printf(…, ...)`), **bitfields**, **unions**,
  **`#pragma pack` / packed structs**, **flexible array members**, **`_Atomic`**,
  and **C++ constructs** (templates, overloads, references, RAII — P14's honestly
  partial neighbor). All rejected in this edition; each is a named §7 deferral,
  never a silent coercion.

**1.4 Calling convention = the C ABI, per target.** There is no Candor native
calling convention at the boundary and no stable native ABI (P14/NN#15): a foreign
call uses the target platform's own C ABI (SysV AMD64, AArch64 AAPCS, …). The
`"C"` ABI string selects it; the native backend (0010) lowers to it — **including
the target's by-value aggregate argument-classification algorithm** (the duty
named under the layout decision above), which layout compatibility does **not**
supply. Non-C conventions (`stdcall`, `vectorcall`) are §7 deferrals.

### 1.5 Boundary exports — the reverse direction (Candor called from C)

FFI is bidirectional: C code (a callback, a registered handler, a `main`
replacement) may need to **call a Candor function**. The forward direction (§1)
declares foreign *imports* with `extern`; the reverse direction declares Candor
*exports* with a matching **`export` item**, well-formed **only inside a boundary
file** (the same NN#18 confinement as `extern`; an `export` outside a boundary
file is the E1101 placement error). The construct mirrors an `extern` declaration
in reverse: it names an existing `pub` Candor function and re-states its signature
under the `"C"` ABI, binding it to a stable C symbol.

```
boundary

// an ordinary safe Candor function:
pub fn checksum(buf: read [u8]) -> u32 { … }

// export it under the C ABI at symbol `candor_checksum`, C-mapped signature:
export "C" fn candor_checksum(buf: rawptr u8, n: usize) -> u32 = checksum;
```

- **The body is ordinary Candor.** An `export` introduces no new kind of
  function; it exposes an existing `pub` fn at a C-ABI symbol. The exported
  signature uses the §1 type-mapping table (a `read [u8]` argument is presented to
  C as the decomposed `(rawptr u8, usize)` parts, §1's slice rule); the boundary
  wrapper reconstitutes the Candor view. The mappability predicate (§1) is the
  same in this direction — a type C cannot receive cannot cross either way.
- **C-ABI emission is a 0010 forward-dependency.** Emitting the export's C-ABI
  entry trampoline (argument classification, the by-value duty of §1/§1.4) is
  native-backend work; until 0010 lands an `export` **parses, checks, and is
  audit-enumerated** but is not callable from a real C frame — the same honest gap
  §5 states for the forward direction. Recorded as a 0010 forward-dependency in
  both documents (0010 §6).
- **An inbound fault aborts per the root policy — no unwinding across C frames,
  ever.** When a fault fires inside an exported Candor function that was entered
  from a C frame, there is **no Candor stack discipline across the C boundary to
  unwind through** and no defined C-side handler. The fault is therefore delivered
  to the **root fault policy** (chapter 06 §6.1) and, absent a handler, **aborts
  the process** — it never propagates as an exception, a `longjmp`, or any
  non-local exit across the C activation records. This is the only sound choice:
  unwinding across frames the C compiler laid out under a convention Candor does
  not control would be undefined behavior on the C side (NN#1's guarantee stops at
  the boundary). An export whose Candor body may fault therefore carries the
  standing consequence that a fault is a process abort for its C caller — stated,
  not hidden.
- **The `foreign` effect.** An `export` is a *sink* of Candor code into C, not a
  *source* of foreign trust: the exported function is ordinary Candor and does
  **not** acquire `foreign` (it is not calling out). If its body *itself* calls a
  boundary import, that is the ordinary §2 partition, discharged or propagated as
  usual *before* it is exported.

### 2. The foreign-trust effect, activated

Foreign trust is the **second and final** member of P2's closed effect set. It is
a tracked effect on signatures, spelled **`foreign`** in the same slot as `alloc`
(`fn f(x: T) foreign -> R`; both, in canonical order, as `alloc foreign`). Its
partition mirrors the `alloc` partition of spec 08 §3 **with one added rule — a
discharge site** — and that rule is what makes P17's *localization* true rather
than aspirational.

**The partition semantics.**
1. **Ground source.** An `extern` declaration is implicitly `foreign`; it is the
   only origin of the effect. Because externs live only in boundary modules
   (§1), the effect is *born* only there.
2. **A foreign call is unsafe in principle (P17).** Calling an `extern` requires
   an `unsafe` region — the existing valve (spec 05), **not a new one** (§budget).
   Foreign calls add no second valve; they point the one valve at a foreign
   symbol.
3. **One-way propagation, universal ground floor.** A non-`foreign` function
   SHALL NOT call a `foreign` function; calling one makes the caller `foreign` —
   *unless the call is a discharge site*. Non-`foreign` code is callable from
   everywhere (the same universal substrate `alloc` has): the vast majority of a
   program never carries the effect.
4. **Discharge — only in a boundary module.** A boundary-module wrapper that
   calls a `foreign` function inside an `unsafe` region carrying the trust
   declarations (§3) that justify its preconditions **exports a non-`foreign`
   signature**. The effect is *extinguished* at the wrapper. This is the
   localization P17 requires: in the healthy case `foreign` is born at the extern
   and dies at the wrapper, never appearing in the wider call graph — exactly like
   `unsafe`, which does not propagate to a safe wrapper's callers.
5. **Undischarged escape.** A boundary module MAY instead export a **`foreign`**
   signature (a thin wrapper, or a re-exported extern) when the trust genuinely
   cannot be discharged at this layer — e.g. it returns a `rawptr` the caller must
   not outlive. Such a signature keeps the obligation *visible in the effect*, and
   by rule 3 it can be called only from another boundary module or from
   already-`foreign` code. This is the honest escape hatch; it does not defeat the
   audit, because the effect is exactly what the audit enumerates (§6).
6. **Upper bound (NN#19).** A signature MAY overstate `foreign`, SHALL NEVER
   understate. Removing the marker is a non-breaking strengthening; adding one is a
   breaking change (spec 08 §2, chapter 00 §3.3).

Contrast with `alloc`, stated so it cannot drift: `alloc`'s ground floor is
non-alloc code and the effect normally *propagates up* unbounded; `foreign`'s
ground floor is *also* non-foreign code, but the effect is normally *extinguished*
at the boundary wrapper. Same one-way shape, opposite typical lifetime — because a
program is *mostly* not-C, sits on a *localized* C substrate, and P17 exists to
keep that substrate a bounded, named region.

**Fn-pointer effect typing (spec 08 §5, extended).** A foreign function pointer
has type `fn(…) foreign -> R`; an indirect call through it is a foreign call and
takes the effect from the pointer type — one general rule, no special case.
Assigning a `foreign` fn-pointer to a non-`foreign` slot is ill-formed
(understatement); the reverse is permitted conservatism.

**Migration (NN#19) — activation is clean.** NN#19's mechanical path for a new
effect is conservative default: mark every existing function `foreign` unless
declared otherwise, a sound over-approximation, refined afterward. Here that set
is **empty**: no code in the prototype or spec calls foreign (spec 08 §6 SKELETON,
0001 §9 — the basket is pure Candor). Activation therefore adds the effect with
*nothing to migrate*: every existing signature is non-`foreign` and stays valid,
and the first `foreign` marker appears only when the first `extern` is written.
The NN#19 machinery is defined and unused — the honest, cheap case.

### 3. Trust declarations

**Most boundary properties are not checkable, so most trust is `assumed-proven`
(P17).** "Does not retain this pointer," "does not touch it from another thread,"
"the pointee is valid for `n` bytes" cannot be evaluated as Candor booleans at any
runtime. They are attached to the extern as a **`trust` clause** and are
`assumed-proven` contracts (spec 07 §5.2): the check is *skipped*, and the clause
is a greppable, enumerable assertion that verification happened *externally* —
identical in discipline to an `unsafe` justification (spec 05 §1.2).

- **A `trust` clause carries a mandatory, non-empty justification string** — the
  same discipline as `unsafe` (spec 05 §1.2): the implementation enforces its
  *presence*, never its *truth*, and records it with its span for enumeration.
  This is the load-bearing honesty of P17: the string is the human/model claim a
  reviewer audits; the checker never validates it.
- **The predicate vocabulary** is a small named set for the recurring C-boundary
  properties: `no_retain(p)`, `no_free(p)`, `valid_for(p, n)`,
  `valid_nul_terminated(p)`, `thread_confined(p)`, `no_overlap(p, q, n)`. Each is
  `assumed-proven` — never evaluated, present to be *read and enumerated*. The set
  is closed and grows only by amendment (like the effect set), so the audit
  vocabulary stays finite and mechanically enumerable. A free-form assertion is
  carried by the justification string, not by an open predicate grammar.
- **The checkable value-level subset lives on the WRAPPER, at `enforced`.** Null
  checks and range preconditions *are* dynamically checkable in safe Candor, so
  they are not trust — they are `enforced` contracts (spec 07 §2) the safe wrapper
  carries and the runtime checks. The split is the whole point: the extern
  declares what only-external-proof can establish (`assumed-proven` trust); the
  wrapper enforces what the machine can establish (`enforced` value contracts).
  Nothing is trusted that could have been checked.

```
// wrapper (same boundary file); discharges strlen's trust, exports a safe signature
pub fn c_strlen(s: read [u8]) -> usize            // safe: no `foreign`, no rawptr in signature
    requires(len(s) > 0 && s[len(s) - 1] == 0)    // enforced: NUL-termination checked here
{
    unsafe "strlen's trust clause is discharged: s is a live, NUL-terminated view;
            strlen retains nothing (POSIX 7); no thread state touched" {
        strlen(addr_of(s[0]))                      // the foreign call; effect dies here
    }
}
```

### 4. The wrapper pattern and the P14 ingestion staging

**A boundary module's public surface is safe Candor; its audit surface is the
whole module.** Authors export `pub` wrappers (safe signatures, §3) and keep the
raw externs private (or export them `foreign`, §2 rule 5). The auditor reads the
*module*; the consumer calls the *pub API*. The two surfaces are deliberately
different sizes: the trusted surface (externs + trust clauses) is what
`candor audit` enumerates; the safe surface (pub wrappers) is what the ecosystem
links against.

**P14 ingestion is staged honestly — hand-written now, header-generated later.**

- **Stage 1 (this design): hand-written extern declarations, checked against
  nothing.** There is no header oracle yet, so the type mapping in §1 is the
  author's responsibility; the checker verifies only that the declared types are
  C-mappable (§1) and well-placed (§1, E1101/E1102). A wrong `int`→`i64` is not
  caught here — it is exactly the kind of error Stage 2 exists to remove.
- **Stage 2 (deferred, obligation recorded): header-driven generation.** Per P14,
  the *toolchain* consumes C headers and generates the extern block and the §1
  mapping, resolving `long`/`char`/width per target from the real ABI — so the
  generated externs *are* checked, against the header. **The generator is a
  blessed, always-installed tool, not code in the compiler** (P14's explicit Zig
  lesson: `@cImport`'s in-compiler maintenance weight pushed Zig to move it out).
  What the tool **cannot** generate is the *trust* clauses: no tool infers "does
  not retain" from a C signature. Trust stays author-written at every stage — the
  honesty is not automatable, and a generator that guessed it would manufacture
  the false verification claim P17 refuses. Recorded as spec obligation
  **OBL-FFI-GEN** (chapter 99), gated on 0010's backend and a C parser.

### 5. Interpreter / prototype reality

The prototype is a tree-walker (0001 §5); **it cannot execute the C ABI**. The
design is staged so the parts of P17 that matter *before* a backend — the audit
surface, the effect partition, the trust enumeration — are fully exercisable now,
and only the actual foreign *call* waits on 0010.

- **Parse / check / audit-enumerate: fully live now.** `extern` blocks parse; the
  boundary-placement rule (E1101), the C-mappability rule (E1102), the `foreign`
  effect partition (§2), the trust-clause justification discipline (§3), and the
  `candor audit --boundaries` enumeration (§6) all run in the prototype without a
  backend. This is deliberate: the P17 *auditability* claim is testable end-to-end
  before a single C symbol is called.
- **Calls: registered-intrinsic shim for testing; a defined fault otherwise
  (chosen).** The tree-walker maintains a **test-only shim registry** mapping a
  foreign symbol name to a Rust closure. A foreign call whose symbol is registered
  dispatches to the shim (letting harness tests drive the *whole* boundary
  machinery — discharge, effect flow, trust enumeration — with a stand-in
  implementation). A foreign call whose symbol is **not** registered raises a
  defined **`no_foreign_runtime` fault** (chapter 06 fault, routed through the root
  policy) — never undefined behavior, never a silent no-op. This choice (shim +
  defined fault, rather than fault-only) is made because the boundary *machinery*
  is the deliverable of this round; a fault-only prototype could audit but never
  *test* a discharge, leaving P17's most load-bearing path unexercised until the
  backend. The shim registry is harness-scope only and ships no C.
- **Shim/real differential obligation (seam with 0010 §4).** A shim is a
  *stand-in*, and a stand-in that diverges from the real C symbol would let the
  boundary machinery pass tests it should fail. Therefore, **when FFI enters the
  differential corpus (0010 §4), each registered shim carries a per-symbol
  contract that is itself differentially tested against the real C
  implementation** on a hosted target: the shim and the linked C symbol must agree
  observably on the fixture inputs, exactly as compiled-vs-interpreted agreement
  is tested for pure Candor. Until then the shim is trusted for harness use only
  (it ships no C, above). The obligation is recorded in **both** this design and
  0010 §4 so neither side can quietly let the shim drift from the C it imitates.
- **Real foreign calls in the AOT backend (0010), now live — no shim.** The
  cranelift-object backend lowers a foreign `extern "C"` call to a Cranelift call
  on an IMPORTED symbol (`Linkage::Import`), which the system linker binds to the
  real libc symbol in the compiled binary (the hosted profile already links
  libc). A standalone Candor executable therefore does genuine libc I/O with no
  JIT, no `candor-proto`, and no shim registry. Two boundary specifics:
  - **Extern-name -> C-symbol convention.** The edition has no `symbol` attribute,
    and the std/io boundary names its externs `sys_*` because `read`/`write` are
    Candor keywords (the borrow modes). The AOT backend maps an extern's declared
    name to its C symbol by **stripping a leading `sys_`** (identity otherwise):
    `sys_read`->`read`, `sys_write`->`write`, `sys_open`->`open`,
    `sys_close`->`close`. This is the deliberate, documented rename in lieu of a
    per-extern symbol attribute; a `symbol` clause is the natural later refinement.
  - **`rawptr` -> real-pointer translation at the call boundary (critical).** A
    compiled program's `rawptr` value is an OFFSET into the flat `MEM_BASE` region,
    but libc needs a REAL address. So every pointer-typed foreign argument is
    translated `host = MEM_BASE + offset` at the call site (the region is mapped at
    the fixed `MEM_BASE`, so this is the same `base + candor_addr` arithmetic every
    load/store already uses); scalar args are narrowed to their C ABI width (an
    `i32` fd) and a sub-word return (`open`'s `i32`) is sign-extended back to the
    i64 word. The gate compiles the std_io demonstrator to a real ELF process that
    opens a fixture file, uppercases it, and writes to stdout — asserting the same
    observable result (exit byte + stdout bytes) as the shim-backed interpreter run.
- **Freestanding + FFI is a contradiction (stated).** The `--freestanding` profile
  links no libc (`-nostdlib`), so an imported C symbol has nothing to bind to; a
  foreign `extern` call under freestanding is therefore a **compile error**, not a
  link-time surprise. For an unregistered extern under the tree-walker/MIR engines,
  `no_foreign_runtime` remains the honest runtime behavior, and the shim covers
  engine-equality testing.

### 6. `candor audit --boundaries`

0008 §4 promised the command as a structural query over interface artifacts; this
fixes its **output**. It is P4-structured: human prose **and** machine-readable
JSON, from one walk of the boundary modules. For each boundary module it reports
(a) the module path, (b) every `extern` with its full signature and `foreign`
effect, (c) every trust declaration — predicate, justification string, source span
— (d) the **effect reach**: which `pub` wrappers *discharge* `foreign` (safe
surface) and which *propagate* it (undischarged escape, §2 rule 5); and (e) every
`export` (§1.5) — its C symbol, mapped C-ABI signature, and the Candor `pub` fn it
exposes — so the auditor sees the *inbound* trust surface (code C may call) beside
the *outbound* one (code that calls C).

```
$ candor audit --boundaries
boundary module  ffi::libc  (src/ffi/libc.cnr)
  extern "C"  (ABI: sysv-amd64)
    fn strlen(rawptr u8) foreign -> usize
      trust @12:9  "POSIX 7 strlen: reads to NUL, retains nothing, no thread state"
        valid_nul_terminated(s), no_retain(s)
    fn memcpy(rawptr u8, rawptr u8, usize) foreign -> rawptr u8
      trust @18:9  "C11 7.24.2.1: valid for n, non-overlapping, unretained"
        valid_for(dst,n), valid_for(src,n), no_overlap(dst,src,n), no_retain(dst), no_retain(src)
  effect reach
    pub c_strlen(read [u8]) -> usize           discharges foreign   (enforced: NUL-terminated)
    pub c_memcpy_into(write [u8], read [u8])    discharges foreign
  exports  (C may call in)
    export "C" fn candor_checksum(rawptr u8, usize) -> u32 = checksum
      inbound fault -> root policy (abort; no unwinding across C frames)
  summary: 2 externs, 7 trust predicates, 0 undischarged foreign wrappers, 1 C-ABI export
```

The JSON form carries the same fields keyed by module, extern, and trust
predicate, each with its span — the exact shape the §7 auditor's tooling consumes
to answer "show me everything this program trusts" in one command (P17).

### 7. Deferrals

The named forward obligations of this edition — each a §7 deferral referenced
above, collected here so the deferral surface is enumerable in one place, never a
silent coercion:

- **Extern globals / mutable statics** (`extern "C" { static errno: i32; }`).
  Deferred. A foreign *global* is a different trust and aliasing question from a
  foreign *call* — an ambiently-mutable place the C side may change under Candor's
  aliasing model — and it is not needed for the call-oriented surface this design
  fixes. **The errno-accessor idiom in the meantime:** POSIX already exposes
  `errno` as a **thread-local accessor function** (`__errno_location()` on glibc,
  `__error()` on BSD/macOS), not a linkable object, so the portable, in-edition
  way to read it is to declare that accessor as an ordinary `extern` foreign
  *function* returning `rawptr i32` and read through it inside the boundary
  wrapper — no extern-global mechanism required. Direct `extern static` binding is
  the deferred obligation; the accessor-function idiom covers the common case now.
- **Floating types** (`float` / `double` / `long double` / `_Complex`) — no scalar
  float in this edition (0001 §5); unmappable until floats land.
- **Variadic functions, bitfields, unions, `#pragma pack` / packed structs,
  flexible array members, `_Atomic`** — each rejected in this edition (§1).
- **C++ constructs** (templates, overloads, references, RAII) — P14's honestly
  partial neighbor.
- **Non-C calling conventions** (`stdcall`, `vectorcall`) and **`const`
  qualification modelling** (§1) — deferred.
- **Header-driven generation** (OBL-FFI-GEN, §4) and **real foreign calls / C-ABI
  emission** (0010, §5, §1.5) — the staged obligations recorded elsewhere, noted
  here for completeness of the deferral surface.

## Rejected alternatives

- **Inline `extern` anywhere (no boundary module).** Rejected under NN#18/P17. It
  is the C/Rust status quo and it makes "what does this program trust?" a
  whole-program grep with no structural guarantee of completeness. Confining
  `extern` to `boundary` files (E1101) makes the audit surface *the file set*, so
  the enumeration is exact and cheap (0008 §4). The largest unsafe surface in
  practice deserves a structural home, not a lexical convention.
- **`bindgen` inside the compiler (`@cImport`-style).** Rejected under P14's named
  Zig lesson. A C header parser and ABI mapper is a large, target-sensitive,
  perpetually-maintained artifact; welding it into the compiler is the maintenance
  weight Zig explicitly retreated from. The generator is a blessed always-installed
  *tool* (P16's one-toolchain, not one-binary), which also keeps compile speed
  (P20) off the C-parser's critical path.
- **Automatic safety inference (a tool that decides which externs are safe).**
  Rejected under P17's load-bearing honesty. Retention, threading, and validity
  are not inferable from a C signature; a tool that emitted "safe" would
  manufacture the *verification* claim P17 withdrew in v3. Trust is *declared*,
  never *proven* — the wrapper and its `assumed-proven` trust clause are the honest
  form, and they stay author-written at every ingestion stage.
- **A `repr(C)` layout attribute.** Rejected as unnecessary: the prototype's
  declared-order / natural-alignment / no-reorder layout (0001 §4.2) is *already*
  the C algorithm, so a boundary struct is C-compatible with no annotation. Adding
  the attribute would imply Candor has a *non-C* default to opt out of — it does
  not, and inventing one to justify the keyword would be backwards.
- **Auto-mapping Candor `enum`/`Box`/slice to C.** Rejected: each has a
  Candor-specific layout (tagged union, tri-word box, fat pointer) with no
  portable C equivalent. Silently coercing them would hide a lossy, unsound
  reinterpretation exactly where the audit must be sharpest; they are rejected
  (E1102) and decomposed by hand.
- **A second valve for foreign calls.** Rejected under P6 and spec 05 §5's
  "exactly one valve." A foreign call is a raw-pointer-shaped act; it reuses the
  `unsafe` region, so the valve count and the Bet 5 measurement are undisturbed.

## Consequences and costs

- **The effect set is now full.** Activating `foreign` spends P2's **second and
  final** reserved slot. Any third tracked effect is now an amendment under NN#19,
  with its own conservative-default migration — the door is not closed, but the
  budget line is drawn here, deliberately.
- **New bucket-1 surface: `extern`, `foreign`, `trust`, `export`, and the trust-predicate
  vocabulary.** Priced as semantic words (P13), like `unsafe`/`boundary` — see the
  reclassification record. The offsetting "remove" (P6) is that **no new valve and
  no new contract level** are introduced: foreign calls reuse `unsafe` (spec 05
  §5), and trust reuses `assumed-proven` (spec 07 §5.2). The additions are a
  reserved effect slot filled and keywords for a surface that had none, not new
  mechanism classes.
- **Candor forgoes struct field-reordering forever.** The C-compatible-by-default
  layout means the compiler may never reorder struct fields to pack them (as Rust
  does by default). It never did — this ratifies an existing property — but it is
  a real, permanent ceiling on struct packing, named here so no later round
  "discovers" a reordering optimization and breaks the ABI silently.
- **Trust is mostly unverified, and this design says so.** By P17 and the Refusals
  section, most `trust` predicates are `assumed-proven` — the checker enforces the
  justification's *presence*, never its truth. A wrong trust declaration yields
  wrong *values* or memory unsafety in the foreign code, never UB attributed to
  safe Candor (spec 07 §4: the optimizer never assumes a contract). We enumerate
  what is trusted; we do not pretend to verify C. The auditor's guarantee is
  *"here is exactly what is trusted,"* not *"this is correct."*
- **No real C calls until 0010.** The prototype audits and effect-checks the
  boundary and shims calls for testing, but a genuine C-ABI call awaits the native
  backend. `no_foreign_runtime` is the honest gap, not a hidden one.
- **Float, variadics, C++, bitfields, unions, packed structs remain unmappable.**
  Each is a named §7/OBL deferral. A program needing `printf` or a `double`-taking
  C API cannot bind it in this edition — stated as a debt, not a soon-fix.

## Reclassification record

`extern`, `foreign`, and `trust` are priced as bucket-1 semantic words (P13), not
item-7 ergonomics, on the §2 test: each encodes a fact the *reader/verifier* must
know to judge a line — that a call leaves the safe language, that a signature
reaches undischarged foreign trust, and precisely what external property is being
assumed. The token cost falls on the reader's side of the ledger (P17's auditor
reads exactly these), where P13's clarity-density measure legitimately promotes
it — the same basis on which 0008 priced `pub` and `boundary`. No other decision
here turns on the reclassification rule.
