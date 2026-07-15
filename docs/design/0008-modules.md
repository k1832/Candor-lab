# 0008 — The Module System

**Status:** draft
**Date:** 2026-07-08
**Philosophy hooks:** P20 (the load-bearing mechanism — this document must
deliver it literally), P2/NN#17 (nothing crosses a public signature by
inference), P16/NN#16 (one toolchain; no configuration where convention
suffices), P3/NN#11 (one canonical way), P1/P17/NN#18 (unsafe- and
boundary-module enumerability), P6 (small core), P9 (subtractively layered
stdlib), P11/NN#10 (definition-site generics), P13 (clarity-dense syntax).

**Revision history.** 2026-07-08 — revised per joint adversarial review #1 of
designs 0007/0008 (`docs/reviews/2026-07-08-design-0007-0008-review-1.md`): F5 the
codegen tier consumes def-site-resolved effects and never re-derives them (§2.4); F6
manifest honesty — identity plus an optional `freestanding` claim, dependency
lockfile deferred to package-manager scope (§1, §5); F4 seam — the interface's
*declaration* module is the placement referent, the uniqueness key is the
instantiated interface (§Decision, §Consequences). 2026-07-08 — joint adversarial
review #1 of designs 0010/0011
(`docs/reviews/2026-07-08-design-0010-0011-review-1.md`) F3: the §2 codegen
*cache key* carries a schema/toolchain salt so a toolchain upgrade cannot
silently reuse machine code the old compiler emitted (§2 codegen-invalidation
bullet).

## Problem

P20 makes a falsifiable competitive claim in §1 and names its mechanism:

> Strict module DAGs with no textual inclusion and no cross-signature inference
> make compilation incremental and parallel *by construction*: a module's
> interface is its signature set, so downstream work is invalidated only when
> signatures change, not when bodies do.

That mechanism does not exist until the module system exists. 0001 §9 and 0006
§7 both deferred modules deliberately — the prototype is single-file, and
because signatures are already fully explicit (P2), modules add no *memory
model* decision. They add a *P20/packaging* decision, and this document makes
it. The single load-bearing call is the generics-across-modules tension
(§2.4): monomorphization needs a generic *body* to instantiate, but P20 says
bodies do not cross the interface. That is resolved honestly below, not
finessed.

This document fixes the module *unit*, *interface artifact*, *dependency DAG*,
the *boundary-module hook* (P17, FFI content deferred), and the *core/std
layering* (P9). It leaves a named seam for 0007 (interfaces/impls, coherence,
the orphan rule): the module is the coherence unit; an impl is an ordinary
item under these visibility and interface rules; 0007 §2.3 fixes that placement predicate — the interface's *declaration* module
is the referent, and the uniqueness key is the instantiated interface (F4).

## Decision

### 1. The unit — the filesystem *is* the module tree (P16)

**One file is one module. The directory hierarchy is the namespace
hierarchy. There is no module-membership declaration and no manifest stanza
listing sources.** A file `net/tcp.cd` is the module addressed as `net::tcp`.
A directory `net/` is the namespace `net`. Convention supplies the whole tree;
config supplies none of it (P16: no configuration where there can be a
convention). This is strictly less ceremony than Rust, which requires a
`mod tcp;` declaration *in addition to* the file existing — a second source of
truth that can disagree with the first.

- A directory may carry its **own** module body via a sibling file named for it:
  `net.cd` beside `net/` is the body of module `net`; `net/*.cd` are its
  children. A bare directory with no sibling `.cd` is a **namespace-only**
  module (it has children but no items of its own).
- **No reserved body filename** (`mod.cd`, `index.cd`): rejected in §Rejected.
  The `foo.cd`-beside-`foo/` rule keeps a directory's body file
  distinguishable in a plain `ls`.
- **One package declares exactly one thing** the filesystem cannot supply: its
  globally-unique **name** (for provenance, P16) plus an **optional verified
  `freestanding` claim** (§5), in a minimal
  manifest at the package root. Module *structure within* a package is pure
  convention; only package *identity* is declared, because a name unique across
  the ecosystem is irreducibly non-local. That is the entire manifest budget
  this document spends. **A dependency lockfile is *not* part of it** — pinning
  resolved dependency versions is **package-manager scope, deferred** to that round.

**No textual inclusion, ever.** This is NN-level in P20. A module is never
spliced into another as source text; nothing is `#include`d, `import`ed as
text, or otherwise re-parsed per consumer. A module is consumed only through
its compiled **interface artifact** (§2). Textual inclusion is precisely the
thing P20 forbids *by construction*, because it makes the "interface" a body
that is re-parsed and re-checked in every consumer — the opposite of
signature-bounded invalidation.

### 2. The interface — the exported signature set as a compiled artifact

**A module's public surface is its set of `pub` item signatures. Everything
else is private to the module.** `pub` is the export construct (P13: a word,
because visibility is a semantic fact a reviewer must know, not skippable
boilerplate). Private-by-default follows P2 (locality) and P6 (the smallest
default).

A signature is already a **complete contract** (P2/NN#17): it carries full
types, parameter modes (`take`/`read`/`write`/`out`, 0006 §2.3), region tags
(0001 §3.3), the effect set (`alloc` today, foreign-trust later — P2's closed
set), and declared contracts (`requires`/`ensures`, 0006 §2.7). Nothing a
caller needs is inferred across the boundary — that is NN#17, and the module
interface inherits it for free because signatures were built that way from
0001.

**The interface artifact.** The compiler emits, per module, a serialized
artifact containing:
1. the module path and its boundary marker if any (§4);
2. for every `pub` item, its full signature as above;
3. for every `pub` **generic** item and every `pub` item marked **`inline`**,
   its *checked* mid-level IR body (§2.4 explains why this is sound and why it
   does not reopen NN#17);
4. **two content hashes** — a **signature hash** over (1)+(2)+the markers, and
   a **codegen hash** that additionally covers the (3) bodies.

**The P20 invariant, stated as a binding design guarantee.** Invalidation is
split into two tiers, gated by the two hashes:

- **Analysis-invalidation is gated by the signature hash alone.** A downstream
  module's type-checking, borrow-checking, effect-checking, and
  contract-checking depend *only* on the signature hashes of the modules it
  imports. **Editing a body without changing any `pub` signature leaves every
  signature hash unchanged, so no downstream module is re-analyzed — not one
  line.** Editing a `pub` signature changes exactly that module's signature
  hash, invalidating exactly its direct importers, and transitively only those
  whose own signature hash changes in turn. This is P20 delivered literally:
  downstream *work* — the expensive, serial, cascading analysis work — is
  invalidated only when signatures change, never when bodies do.
- **Codegen-invalidation is gated by the codegen hash**, is content-addressed,
  and is fully parallel and cache-shared (§2.4). It never triggers
  re-*analysis*; it triggers at most re-*emission* of specific machine code
  from already-checked IR. The codegen *cache key* additionally carries a
  schema/toolchain salt (the MIR-schema version and compiler/backend identity) so
  a toolchain upgrade cannot silently reuse machine code the old compiler emitted
  (0010 §3, joint design 0010/0011 review #1 F3).

Per-module analyses (effects, contracts, borrow checking) are what P20 promised
"never require whole-program passes": each runs over one module against the
explicit signatures of its imports, so the modules of a DAG level analyze in
parallel.

**What the invariant forbids (and its one opt-in).** Cross-module *inlining* is
not the default, because inlining another module's body into your codegen makes
your output depend on that body — reintroducing body-level downstream coupling.
A function whose body a consumer may inline must be marked **`inline`** at its
definition. The marker is honest and greppable (P13): it moves that one
function's body into the interface artifact (item 3), so edits to *that* body
now participate in codegen-invalidation of consumers — but never in their
analysis-invalidation. Everything unmarked keeps the pure P20 guarantee by
construction. The default is opacity; visibility across the boundary is an
explicit, auditable decision.

### 2.4 Generic bodies across modules — the load-bearing resolution

The tension is real and classic: P11 checks a public generic completely at its
definition site, but a *monomorphizing* instantiation needs the generic's body
to emit code, and P20 says bodies do not cross the interface. Two framings were
weighed:

- **(A)** the generic's checked body is *part of* the interface artifact, and
  instantiation is codegen from it;
- **(B)** per-module codegen with a shared instantiation cache keyed by
  (generic-definition-content, type arguments).

**These converge, and the unified model is the decision.** The insight that
dissolves the tension: P11 already guarantees that a generic that compiles
"cannot produce type errors at instantiation," and that "instantiation is
cacheable codegen, never re-analysis." So what a downstream instantiation
consumes from a generic is **codegen input, not analysis input.** The
expensive, cascading part — type/borrow/effect/contract checking — happens
*once*, at the generic's own module, and is never redone by any instantiator.

Therefore: **the interface artifact of a `pub` generic carries its body as
serialized, already-checked mid-level IR** (item 3 above), and a downstream
instantiation performs **zero semantic analysis** — it reads verified IR and
lowers it to machine code. **The IR carries the generic's def-site-resolved
effects** (its `alloc` marking, including the conservative drop-glue alloc-ness of
0007 §3.4/§5.2); the codegen tier **consumes** these resolved effects and **never
re-derives an effect** at instantiation — effect resolution belongs to the once-only
analysis at the generic's own module, not to codegen. This does **not** reopen NN#17, because NN#17
forbids *inference across a signature*: nothing about the caller is inferred
from the generic body, and nothing about the generic is inferred from the
caller. The body crosses as a *finished proof-carrying artifact*, not as
source to be re-reasoned-about. (A) is simply this artifact viewed from the
producer side; (B) is the same artifact viewed from the cache side — the shared
cache is keyed on the generic's codegen hash, and "the generic body changed" is
exactly "the codegen-hash key changed, so this instantiation's cache entry is
stale." Same mechanism, two vocabularies; we adopt both descriptions of the one
thing.

**The precise invalidation story for generics, stated so it cannot drift:**
- Editing a generic's **body** changes its **codegen** hash but not its
  **signature** hash. Consequence: every downstream module still passes
  analysis untouched (P20 holds); only the *instantiations* of that generic are
  re-emitted, and only those — a cheap, parallel, per-instantiation codegen
  step, exactly what P11 calls cacheable codegen. Body edits to a generic never
  cascade analysis. This is the honest boundary of P20's slogan: "body edits
  never invalidate downstream" is a guarantee about **analysis**; a generic (or
  `inline`) body edit forces re-*codegen* of its own instantiations, which is
  the cheap tier by design.
- Editing a generic's **signature or its declared bounds** changes the
  signature hash and invalidates dependents' analysis normally.

**The instantiation strategy itself (monomorphize vs. shared code) is P11's,
not this document's.** The module system is strategy-agnostic: it supplies the
checked-IR interface artifact and the content-addressed codegen cache that
*either* strategy consumes, and P11's deterministic, per-target-documented,
source-overridable default decides which is emitted. Keeping the strategy in
P11 and the artifact plumbing here is deliberate: the codegen cache is shared
across the whole build (and, with the reproducible-build guarantee of P16,
across builds), so an instantiation `Vec<u32>` needed by two modules is emitted
once, keyed by content, regardless of strategy.

**Program entry (ruling, 2026-07-08, surfaced by stage 1):** a directory build's
root module is the root-level file `main.cd` (prototype: `main.cnr`), and the
program entry is its `fn main` — the filesystem-is-the-tree convention applied
to entry as well; no manifest entry-point field.

**Erratum (2026-07-15, design 0017 review F6a).** For a **manifested package**
(design 0017: a directory carrying a `candor.toml`), the module-tree root
**relocates from the directory-root `main.cnr` to `src/`**: a library's public root
is `src/<name>.cnr`, and a binary's entry is `fn main` in `src/main.cnr` (or
`src/bin/<name>.cnr`). The ruling above is **unchanged for the manifest-less bare
directory** (the degenerate package, design 0017 §6): its root module stays the
directory-root `main.cnr`, no `src/`. This erratum records the refinement so the two
neighboring designs do not quietly diverge (GOVERNANCE §9); it resolves 0017 review
F6a and preserves the doc hierarchy. The `modules.rs` implementation of `src/`
support lands with the packaging implementation, not with this erratum. **Implemented 2026-07-15** (`prototype/src/modules.rs` `resolve_dir_root`/`check_entry`, `prototype/src/build/mod.rs`; gate `prototype/tests/packages.rs`): a directory build branches on `candor.toml` presence, roots a manifested package's tree at `src/`, and derives the entry target from the manifest (`src/main.cnr` | `src/bin/<name>.cnr` | library root `src/<name>.cnr`), leaving the manifest-less bare directory's directory-root `main.cnr` behavior unchanged.

### 3. The dependency DAG

**Import spelling.** `use net::tcp;` brings the module `tcp` into scope (uses
read `net::tcp::connect`). `use net::tcp::{connect, listen};` brings named
items directly. The path separator is **`::`**, lexically distinct from the
`.` of value/field projection (0006). That distinction is a P2/NN#13 win: a
reader — and the parser, without a symbol table — tells a *module path*
(`a::b`) from a *value projection* (`a.b`) by token alone. The token after
`::` decides the form with one-token lookahead: an identifier (path segment),
`{` (group import), or `[` (a generic-value instantiation `name::[T]`,
design 0007 §6.2.1).

**No glob imports and no glob re-exports.** Every imported name is listed
explicitly. A glob would (a) hide where a name came from (P2 locality) and (b)
make the imported name-set grow silently when the source module adds a `pub`
item. Whole-module re-export (`pub use tcp;`) names *one* module as a
namespace, which is not a glob.

**Acyclic, checker-enforced.** The import graph must be a DAG. A cycle is a
compile error with a P4-grade diagnostic: the full cycle path is printed
(`A::x` uses `B` at line L1, `B` uses `C` at line L2, `C` uses `A` at line L3),
machine-readable, with the offending `use` sites as the implicated
definitions. Acyclicity is **not a style preference** — it is what makes the
P20 mechanism exist: signature-bounded incremental invalidation and parallel
per-level compilation both require a topological order, which a cycle does not
have. An SCC would have to be analyzed as a unit (§Rejected), destroying
signature-bounded incrementality. No cycles, by construction, is the price of
P20, so the checker charges it.

**Visibility — two concepts, no lattice (P6).** Items are private by default;
`pub` exports them. There is **no `pub(crate)` / `pub(super)` / `pub(in path)`
machinery.** The real need those serve — "visible inside my package, not part
of my public API" — is met by the *shape of the DAG* instead of a visibility
sublanguage:

- An item is **externally reachable** iff it is `pub` **and** there is a path
  from the package's public root to its module in which every step is a public
  module re-export.
- A child module is, by default, package-internal: nameable by its full path
  from within the package, but not re-exported to external consumers. A parent
  promotes a child into its public surface with **`pub use child;`** (there is
  no `mod` declaration to hang `pub` on, so module-level publicness rides on a
  re-export statement — one statement, only where promotion is intended).

So "crate-internal helper" is expressed by *not* re-exporting its module
publicly, not by a `pub(crate)` keyword. Two concepts (item `pub`, module `pub
use`) instead of a five-point visibility lattice — the P6 budget spent where it
buys the most reader clarity, since "who can call this?" is now answered by
reading the re-export chain, not by evaluating a lattice.

**Re-exports and the one-canonical-name question (P3).** Re-export (`pub use
a::b::foo;`) is **allowed**, because the stdlib facade (§5) and package-level
API curation require it — but ruled so it does not fork canonicity. An item has
exactly **one definition site**, which is its canonical identity and its
provenance; a re-export creates an additional *path* to the same entity, never
a second entity. The toolchain enforces this: importing the same item by two
paths is idempotent, not a name clash (no diamond ambiguity), and hover/docs
always report the canonical definition path. P3 asks for one construct per
concept and one canonical *identity*, which this preserves; it does not demand
that a name be reachable by exactly one string, which would make facades
impossible and defeat P9's layering.

### 4. Boundary modules — the P17 hook (marker now, FFI content deferred)

A **boundary module** is the P17 unit of foreign-trust audit. This document
fixes the *marker and its guarantees* and defers the FFI *content* to the
P14/P17 round (0006 §7 deferred the FFI surface for the same reason).

- **The marker is file-level and greppable:** a boundary module's file opens
  with the module-preamble keyword **`boundary`** (a contextual keyword valid
  only as the first item-preamble token of a file). Because a file is a module,
  the whole module is thereby a boundary module. The marker is part of the
  module's interface artifact (§2, item 1), so a consumer and an auditor both
  see it.
- **The guarantee:** only boundary modules may host foreign declarations; safe
  code may reach foreign functions only through a boundary module (NN#18). The
  foreign-trust *effect* (P2's second tracked effect, deferred) rides on the
  individual foreign *signatures* inside such a module, not on importers of the
  module.
- **Enumerable by one command:** the toolchain walks the module tree and lists
  every module carrying the `boundary` marker — `candor audit --boundaries`
  answers "show me everything this program trusts" (P1/P17, §7 of the
  philosophy). This is a pure structural query over interface artifacts, so it
  is exact and cheap.

Deferred to P14/P17: the syntax of foreign signatures, how contracts (P8)
attach to them, the foreign-trust effect spelling, and the C-header ingestion
(P14). This document guarantees only that when they arrive they have a
declared, enumerable home.

### 5. Stdlib layering (P9) — core and std as packages

The subtractive layering of P9 maps directly onto packages and the import DAG:

- **`core`** is a package: the always-available, never-allocating layer (types,
  slices, contracts, formatting — 0001's `[u8]` universal view lives here). It
  imports nothing outside itself and carries no `alloc` effect anywhere in its
  public surface.
- **`std`** is a *separate* package layered on top of `core`: collections, OS
  services, allocator-explicit throughout (P9's allocator-as-parameter, 0001
  §6), absent from freestanding targets by default. `std` re-exports selected
  `core` items through its facade (§3's `pub use`) so hosted code has one
  import surface — the facade is why re-export must exist.
- **"Every library against core runs everywhere" is derived, not declared.** A
  library that (transitively) imports only `core` is freestanding-compatible;
  the toolchain computes this from the import DAG and reports it. There is **no
  `no_std` attribute** — core-only-ness is a property *read off* the graph, not
  a dialect a file opts into. This is a deliberate P16 convention-over-config
  win and, more importantly, a P2 mitigation: with no attribute to split on,
  there is no ambient flag around which a parallel-variant ecosystem can
  crystallize (P2's named no_std-gravity risk).
- A package may *optionally* record a **verified `freestanding` claim** in its
  manifest; the toolchain fails the build if a `std` import ever sneaks into a
  package that claims it. This is source-declared, tool-checked intent — one
  semantics, not a second compilation mode (the spirit of P5): the claim
  changes what the *build accepts*, never what the code *means*.

### 6. Prototype migration path

The single-file corpus does **nothing** and stays valid. Single-file is the
**degenerate module**: one file is one module is the whole program, with an
empty import set and no `pub` needed. The R14 harness convention (the first
column-0 `// Test harness` line splitting the measured implementation from its
in-file vector harness, per docs/ADJUDICATIONS.md) is exactly a **proto-module
boundary inside one file** — the single-file world already separates "the
module" from "its test driver" by a textual marker. When multi-file lands, that
harness moves to a sibling test module and the R14 marker convention is
subsumed by real module boundaries; the existing ports need no change to their
measured sections.

Multi-file is scoped as an **implementation stage**, additive over the
single-file prototype, in this order:
1. **Filesystem → module-tree mapping** (the §1 convention: files, dirs,
   `foo.cd`-beside-`foo/`, the package manifest's one identity line).
2. **`use` resolution and the acyclicity check** (§3), with the P4 cycle
   diagnostic.
3. **Per-module interface artifacts and the two hashes** (§2) — this is the
   stage that first *realizes* P20's incrementality; before it, "single-file"
   trivially has nothing downstream to invalidate.
4. **The content-addressed codegen cache** for generics/`inline` (§2.4) — this
   stage co-arrives with the P11 generics implementation, since it is that
   feature's plumbing.

**Status — stages 1–2 shipped** in the prototype (`prototype/src/modules.rs`, real (`.cnr`) front-end only; single files stay valid degenerate modules): filesystem→module mapping, `use` resolution, `pub`/private visibility (error family `E09xx`), and the acyclic-DAG check with the P4 cycle diagnostic; the `foo.cnr`-beside-`foo/` body merge, `pub use` re-exports, and the §2 interface-artifact/hash tiers remain deferred.

Stages 1–2 are a small import resolver and can land before generics exist;
stages 3–4 are where the P20 targets in CI (single-digit-second incremental
rebuild) first become measurable.

## Rejected alternatives

- **Header files / textual inclusion (`#include`).** Rejected at the NN level of
  P20. Textual inclusion makes a module's "interface" a body that is re-parsed
  and re-checked in every consumer — the exact opposite of signature-bounded
  invalidation, and the single largest source of C/C++ build-time blowup. It is
  what P20 forbids *by construction*.
- **Manifest-declared module membership (`mod x;`, Bazel `BUILD` source lists,
  Cargo-style module trees).** Rejected under P16. The filesystem already *is*
  a tree; declaring it a second time creates two sources of truth that drift
  (a file present but undeclared, or declared but absent). Convention where a
  convention exists; the only declaration kept is package *identity*, which the
  filesystem genuinely cannot supply.
- **Reserved directory-body filename (`mod.rs`, `index.js`, `__init__.py`).**
  Rejected in favor of `foo.cd`-beside-`foo/`. Magic filenames are config by
  another name and produce the "twelve tabs all named mod.rs" navigation
  problem; a directory's body file should be identifiable in a plain listing.
- **Inline `module Name { }` blocks (many modules per file).** Rejected. File =
  module is the convention that makes "one interface artifact per file" the
  incremental unit; letting one file host several modules muddies the unit and
  reintroduces a nesting syntax P6 need not buy.
- **Cyclic imports with the SCC as the compilation unit** (ML functor-ish /
  some whole-program compilers). Rejected. Compiling a strongly-connected
  component as a unit means any body edit anywhere in the SCC re-analyzes the
  whole SCC — signature-bounded incrementality dies exactly where the cycle is,
  which in real code tends to be the core. P20 wants the finest-grained DAG;
  cycles also hide the layering violations P9 wants visible. Acyclicity is the
  cost of P20, charged by the checker.
- **`pub(crate)` / `pub(super)` / `pub(in path)` visibility lattice.**
  Rejected under P6. Visibility becomes a lattice a reader must *evaluate* to
  answer "who can call this?" Two concepts — item `pub` and module `pub use` —
  cover the one real need (package-internal) at far lower reader cost, using the
  DAG shape that already exists.
- **Glob imports / glob re-exports (`use net::*`).** Rejected under P2/P3. A
  glob hides a name's origin and lets the imported set grow silently when the
  source adds a `pub` item — a body-ish change leaking into consumers. Every
  imported name is listed.
- **Generic bodies as opaque, re-checked per instantiation (the C++-template
  model).** Rejected under P11/P20. Re-checking a template per instantiation is
  the canonical local-verifiability violation P11 exists to prevent and the
  canonical compile-time blowup P20 exists to prevent. Our generics cross the
  boundary as *already-checked* IR precisely so instantiation is codegen, never
  re-analysis.
- **A `no_std` opt-in attribute for freestanding code.** Rejected under
  P16/P2. Freestanding-compatibility is a property *derivable* from the import
  DAG; deriving it avoids a dialect attribute and denies the parallel-variant
  ecosystem an ambient flag to split on (P2's stated no_std-gravity risk).

## Consequences and costs

- **`inline` and generics reintroduce a body-level dependency — named, not
  hidden.** Editing an `inline` function's body or a generic's body re-emits
  its consumers' *codegen* (never their analysis). P20's "body edits never
  invalidate downstream" is therefore precisely a guarantee about **analysis**;
  the codegen tier is the cheap, parallel, content-addressed cost, but it is
  not zero. A project that marks everything `inline` forfeits the guarantee it
  paid for — the default opacity is what protects it, and the marker is the
  greppable place a reviewer sees the trade.
- **The two-hash artifact is real implementation weight.** The compiler must
  compute and persist a signature hash and a codegen hash per module and cache
  keyed on both. This is the machinery P20 *is*; it is a cost the single-file
  prototype does not pay and the multi-file stages 3–4 must.
- **No `pub(crate)` means "who can reach this?" is answered by reading the
  re-export chain,** not one keyword at the definition. For a deeply nested
  package this is more reading than a `pub(crate)` tag would be; we judge it the
  cheaper total cost because it removes a lattice, and the chain is a finite
  walk the language server can render on hover (P16).
- **Package identity, plus an optional freestanding claim, is the whole manifest budget.** We claimed "no
  configuration where convention suffices," and honored it for module
  structure; a globally-unique package name is the place convention genuinely
  cannot reach, so one declaration stands, joined only by the opt-in `freestanding` claim (§5); a
  dependency **lockfile is deferred to package-manager scope**, not spent here.
  Calling that "no config" would be
  dishonest; it is *minimal* config.
- **Re-export facades create multiple paths to one name.** We ruled identity to
  the definition site and made multi-path imports idempotent, but a reader who
  sees `std::Foo` and `core::Foo` must still learn they are one entity. The
  toolchain shows canonical provenance; the residual cost is that the facade
  exists at all, accepted because P9's layering requires it.
- **0007 seam — the placement predicate, now fixed by 0007 §2.3.** Interfaces/impls,
  coherence, and the orphan rule need module-level placement rules (an impl
  belongs in the interface's module or the type's module — the orphan rule's
  classic shape). This document fixes only that the module is the coherence
  unit and an impl is an ordinary item under these `pub`/interface rules; 0007 §2.3
  fixes the placement predicate: **the interface's *declaration* module is the
  placement referent** (where `interface I[…]` is written), and for a generic
  interface **the uniqueness key is the *instantiated* interface** `(I[args…], T)`,
  not the bare `I` (F4).

## Reclassification record

`pub` and `boundary` are priced as bucket-1 semantic words (P13), not as an
item-7 ergonomics concern, on the standard §2 test: both encode a fact the
*reader/verifier* must know to judge a line — who may call an item, and whether
a module is trusted foreign surface — so their token cost falls on the reader's
side of the ledger, where P13's clarity-density measure legitimately promotes
it. No other decision here turns on the reclassification rule.
