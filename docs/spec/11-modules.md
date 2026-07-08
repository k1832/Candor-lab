# 11 — Modules

**Status: NORMATIVE-DRAFT (shipped subset) + SKELETON (deferred).**
Transcription of design `0008-modules`. Sections 1–6 (the multi-file stages 1–2
shipped in the prototype: filesystem→module mapping, `use` resolution, `pub`/
private visibility, the acyclic-DAG check) are **NORMATIVE-DRAFT**. Sections 7–10
(interface artifacts and the two-hash tiers, `pub use` re-exports and external
reachability, directory bodies, and the boundary/`inline`/manifest/layering
machinery) are **SKELETON**: their obligations bind now, their normative detail
is deferred and tracked in chapter 99. Rationale is in design 0008.

---

## 1. The module unit

1.1 **One file is one module; the directory hierarchy is the namespace
    hierarchy.** A file `net/tcp.cd` is the module addressed `net::tcp`; a
    directory `net/` is the namespace `net`. There SHALL be **no
    module-membership declaration** and no manifest stanza listing sources —
    convention supplies the whole tree (P16; design 0008 §1).

1.2 **No textual inclusion, ever** (NN-level under P20). A module SHALL NOT be
    spliced into another as source text; nothing is `#include`d or re-parsed per
    consumer. A module is consumed only through its exported signatures (design
    0008 §1). Inline `module Name { }` blocks are rejected: file = module is the
    unit (design 0008 §Rejected).

1.3 Single-file programs stay valid as the **degenerate module** — one file, one
    module, empty import set, no `pub` needed (design 0008 §6).

---

## 2. Program entry

2.1 A directory build's **root module is the root-level file `main.cd`** and the
    program entry is its **`fn main`** — the filesystem-is-the-tree convention
    applied to entry; there is no manifest entry-point field (design 0008 §2.4
    ruling).

---

## 3. Imports

3.1 **`use net::tcp;`** brings the module `tcp` into scope (uses read
    `net::tcp::connect`); **`use net::tcp::{connect, listen};`** brings named
    items directly. The path separator is **`::`** (design 0008 §3).

3.2 **No glob imports and no glob re-exports.** Every imported name SHALL be
    listed explicitly; a glob would hide a name's origin (P2) and let the imported
    set grow silently when the source adds a `pub` item (design 0008 §3).

---

## 4. The `::` separator and one-token lookahead

4.1 **`::` is lexically distinct from the `.` of value/field projection.** A
    reader and the parser tell a **module path** (`a::b`) from a **value
    projection** (`a.b`) by token alone, with **no symbol table** (P2/NN#13). `::`
    is never an expression operator (design 0008 §3).

4.2 **The token after `::` decides the form with one-token lookahead:** an
    **identifier** (a path segment), **`{`** (a group import), or **`[`** (a
    generic-value instantiation `name::[T]`, chapter 10 §9.4). This is within the
    two-token ceiling of chapter 02 §1.2 (design 0008 §3).

---

## 5. Visibility

5.1 **Items are private by default; `pub` exports them.** A module's public
    surface is its set of `pub` item signatures; everything else is private to the
    module (P2/P6; design 0008 §2). `pub` is a bucket-1 semantic word — a fact the
    reviewer must know (P13; design 0008 §Reclassification).

5.2 **There SHALL be no `pub(crate)` / `pub(super)` / `pub(in path)` lattice.**
    The "visible inside my package, not part of my public API" need is met by the
    **shape of the DAG**: a child module is package-internal by default (nameable
    by full path within the package, not externally re-exported) unless a parent
    promotes it (§8) (design 0008 §3).

---

## 6. The acyclic DAG

6.1 **The import graph SHALL be a DAG.** A cycle is a compile error with a
    P4-grade diagnostic printing the full cycle path with each offending `use`
    site (design 0008 §3).

6.2 Acyclicity is **not a style preference**: signature-bounded incremental
    invalidation and parallel per-level compilation both require a topological
    order (P20), so the checker enforces it. A strongly-connected component
    compiled as a unit is rejected (design 0008 §3, §Rejected).

---

## 7. Interface artifacts and the two-hash tiers — SKELETON

7.1 **Obligation (binds now).** A module's public surface is its set of `pub`
    signatures, each already a complete contract (types, modes, region tags,
    effect set, contracts) — **nothing a caller needs is inferred across the
    boundary** (NN#17). This clause is normative now; the *artifact* machinery
    below is deferred.

7.2 *(deferred detail.)* The compiler emits, per module, a serialized **interface
    artifact** carrying the module path and any boundary marker, every `pub`
    signature, the checked mid-level IR body of every `pub` **generic** and every
    `pub` **`inline`** item, and **two content hashes** — a **signature hash** and
    a **codegen hash** (design 0008 §2).

7.3 *(deferred detail.)* **Analysis-invalidation is gated by the signature hash
    alone**: editing a body without changing any `pub` signature re-analyzes **no**
    downstream module. **Codegen-invalidation is gated by the codegen hash**, is
    content-addressed, parallel, and cache-shared, and triggers at most
    re-*emission* of machine code from already-checked IR, never re-analysis
    (design 0008 §2, §2.4).

7.4 *(deferred detail — the generics seam.)* The interface artifact of a `pub`
    generic carries its body as **serialized, already-checked mid-level IR**; a
    downstream instantiation performs **zero semantic analysis** and lowers
    verified IR to machine code. The IR carries the generic's **def-site-resolved
    effects** (including the conservative drop-glue alloc-ness of chapter 10 §5.5);
    the codegen tier **consumes** these and **never re-derives an effect**. This
    does not reopen NN#17 — the body crosses as a finished proof-carrying
    artifact, not as source to re-reason about (design 0008 §2.4).

7.5 *(deferred detail — the schema salt.)* The codegen **cache key** SHALL
    additionally carry a **schema/toolchain salt** (the MIR-schema version and
    compiler/backend identity) so a toolchain upgrade cannot silently reuse
    machine code the old compiler emitted (design 0008 §2, joint 0010/0011 review
    F3).

7.6 *(deferred detail — the `inline` opt-in.)* Cross-module inlining is **not**
    the default; a function whose body a consumer may inline SHALL be marked
    **`inline`**, moving its body into the interface artifact and into consumers'
    **codegen**-invalidation (never their analysis) (design 0008 §2).

---

## 8. `pub use` and external reachability — SKELETON

8.1 **Obligation (binds now).** Re-export exists (the stdlib facade and
    package-level API curation require it), but an item SHALL have exactly **one
    definition site** as its canonical identity; a re-export creates an additional
    **path** to the same entity, never a second entity (P3; design 0008 §3).

8.2 *(deferred detail.)* An item is **externally reachable** iff it is `pub`
    **and** there is a path from the package's public root to its module in which
    every step is a public module re-export. A parent promotes a child into its
    public surface with **`pub use child;`** (there is no `mod` declaration to hang
    `pub` on). Importing the same item by two paths is idempotent, not a name
    clash; hover/docs report the canonical definition path (design 0008 §3).

---

## 9. Directory bodies — SKELETON

9.1 *(deferred detail.)* A directory MAY carry its **own** module body via a
    sibling file named for it: `net.cd` beside `net/` is the body of module `net`,
    `net/*.cd` its children; a bare directory with no sibling `.cd` is a
    **namespace-only** module. There SHALL be **no reserved directory-body
    filename** (`mod.cd`, `index.cd`) — the `foo.cd`-beside-`foo/` rule keeps the
    body file distinguishable in a plain listing (design 0008 §1, §Rejected).

---

## 10. Boundary, manifest, and layering — SKELETON

10.1 *(deferred detail — the boundary marker.)* A **boundary module** is the P17
     foreign-trust unit, marked by the file-level contextual keyword
     **`boundary`** as the first item-preamble token; the marker is part of the
     interface artifact and enumerable by `candor audit --boundaries`. Only
     boundary modules may host foreign declarations. The FFI **content** — foreign
     signatures, the foreign-trust effect, contract attachment, header ingestion —
     is **not yet normative** and lives with design 0011 / OBL-FFI (chapter 99;
     design 0008 §4).

10.2 *(deferred detail — package identity.)* One package declares exactly one
     thing the filesystem cannot supply: its globally-unique **name**, plus an
     **optional verified `freestanding` claim**, in a minimal root manifest.
     Module structure within a package is pure convention; a dependency
     **lockfile is deferred to package-manager scope** (design 0008 §1, §5). The
     shipped status of the manifest identity line is a recorded gap (chapter 99).

10.3 *(deferred detail — stdlib layering, P9.)* **`core`** is the
     always-available, never-allocating package (no `alloc` anywhere in its public
     surface); **`std`** is a separate package layered on `core` (collections, OS
     services, allocator-explicit), absent from freestanding targets by default.
     Freestanding-compatibility is **read off the import DAG**, not a dialect —
     there is **no `no_std` attribute** (P16/P2; design 0008 §5).

---

**Gate (OBL-MODULES-ARTIFACT):** the §§7–10 SKELETON detail is tracked in chapter
99. Sections 1–6 discharge the shipped multi-file subset; the P20 incrementality
machinery (§7), `pub use`/facades (§8), directory bodies (§9), and the boundary/
manifest/layering surface (§10) remain deferred, their obligations binding now.
