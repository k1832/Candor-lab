# 0017 — Packages, Manifests, and Dependency Resolution

**Status:** ACCEPTED (with review repairs applied), 2026-07-15 (adversarial review: `docs/reviews/0017-packages-review.md`)
**Date:** 2026-07-15
**Philosophy hooks:** P16/NN#16 (one blessed toolchain *including the package
manager*; reproducible-by-default builds; recorded provenance; no configuration
where convention suffices), P3/NN#11 (one canonical way), P15/NN#14 (evolution
by edition, migration by machine; no stability before Bet 5's verdict), P9
(subtractively layered stdlib — core/std as packages), P2/NN#17/NN#19 (the closed
effect set travels on signatures; upper-bound rule), P17/NN#18 (the foreign
boundary is the audit surface — now across dependencies), P6 (small core; when
you add, remove), P20 (compile speed earned by construction — the package
boundary as an interface-artifact boundary).

**Prior art it fills.** 0008 §1 (lines 70–77, 432–438) fixed the *minimal*
manifest — a globally-unique package **name** plus an optional verified
`freestanding` claim — and **explicitly deferred the dependency system to
"package-manager scope"**: "A dependency lockfile is *not* part of it … deferred
to that round." 0008 §3 defined externally-reachable visibility in terms of "the
package's public root" and "external consumers" — vocabulary that already
anticipates a package boundary this document makes concrete. 0008 §5 layered
`core`/`std` *as packages* and derived freestanding-compatibility from the import
DAG. 0008 §2 fixed the per-module interface artifact and signature-bounded
invalidation (P20). 0011 fixed the `boundary`-module foreign surface and the
`candor audit` enumeration (P17). This document is the deferred round: it fills
the seam 0008 named and does not reopen 0008's module model — it extends it.
ROADMAP item 3 ("Stability + packaging … a package manager, dependency handling.
What lets OTHERS build on it") is this work.

## Problem

A Candor "project" today is one directory of `.cnr` files that
`modules::build_tree` (0008 §1) merges into a single name-qualified `Program`.
There is no package boundary, no version, no way for one Candor codebase to
depend on another, no resolver, and no lockfile. "A language others can build
on" (P16 names the package manager as part of the one toolchain; the ROADMAP
names this as the gate to "others build on it") is definitionally: **package A
declares a dependency on package B, and the toolchain resolves and builds them
together.** That capability does not exist until this round exists.

Three facts constrain the design and make parts of it hard to reverse:

1. **The manifest, version, and edition are load-bearing for the eventual 1.0
   gate** (P15/NN#14, ROADMAP publication step 3). A package's declared *edition*
   is how the language's edition/migrator promise (NN#14) reaches real code; the
   field must be present now even though only one edition value is legal today,
   because a package with no declared edition cannot be mechanically migrated
   (you cannot migrate from an unknown starting dialect). These are *format*
   decisions that ship into every manifest ever written.

2. **The cross-package import mechanism must not contradict 0008.** 0008 imports
   resolve by filesystem position within one build root and merge into one
   `Program`. How a `use` crosses a *package* boundary is the load-bearing
   coherence question, and it is hard to reverse once packages exist in the wild.

3. **A dependency imports someone else's trust surface** (P17). A dependency may
   contain `boundary` modules with `extern "C"` foreign declarations (0011),
   `unsafe` regions (P1), and `assumed-proven` contracts (P8). The package
   boundary must not *hide* these; the auditor's "show me everything this program
   trusts" (P17 §7) must see through it.

### Scope

**In scope (decided here, because they are hard to reverse):** the manifest file
and its minimal schema (name, version, edition, target, dependencies,
`freestanding`); the package version and language-edition schemes; the dependency
*source* model (path and git); the cross-package import mechanism; the resolver's
selection semantics and the lockfile; how the build driver extends from
one-directory `build_tree` to resolve-and-build-many; and the soundness story
(acyclicity, visibility, the merged-`Program` model, and the cross-package trust
surface).

**Deferred, with reasons (§Rejected):** a central **registry** and network fetch
protocol (path + git unblock the real goal without standing up hosting, a naming
authority, signing, and availability infrastructure — P6, §8 sequencing);
SemVer-range **solving** / a SAT resolver (without a registry there are no ranges
to solve); `[patch]`/override machinery for diamonds (0.x errors clearly
instead); vendoring-as-a-mode (path deps already cover the local-copy case); a
stable native **ABI** and separate-compilation/dynamic linking (that is P14's C
ABI, out of scope — 0.x links by merging checked IR); the *content* of any second
edition (there is one edition today; the mechanism is reserved, not populated).

This document is a numbered design document below the §9 amendment threshold
(GOVERNANCE, "Design decisions below the amendment threshold"): it *implements*
commitments already made by NN#16 (a package manager, reproducible builds,
provenance), NN#14/#15 (editions/migrator), and NN#19 (the freestanding claim),
without altering the wording of any Non-Negotiable. If disposition finds any
decision here would change an NN's wording, that part proceeds by the §9 ritual
(GOVERNANCE, 14-day comment) rather than by this doc.

## Decision

### 1. The unit — a package is a directory with a manifest

**A package is a directory containing a `candor.toml` manifest at its root.** The
manifest is the one thing the filesystem cannot supply (0008 §1: name is
"irreducibly non-local"); everything about *module structure within* the package
stays pure convention (0008 §1, P16). The manifest's presence is what
distinguishes a *package* from today's bare directory-of-`.cnr` (which stays
valid as the **degenerate, manifest-less package**, §6).

**Module tree root: `src/`.** By convention (P16, no config) a package's module
tree is rooted at `src/` beside the manifest; a library's public root module is
`src/<name>.cnr` (or a namespace-only `src/` directory), and a binary's entry is
`fn main` in `src/main.cnr` (0008's entry ruling, relocated under `src/`). This
is the one place this document *refines* 0008's "root-level `main.cnr`" ruling,
**ACCEPTED by the deciding authority (2026-07-15)** and recorded as a dated erratum
to **0008 §2.4** (`docs/design/0008-modules.md`; review F6a) so the two neighboring
designs do not diverge (GOVERNANCE §9) — no longer an open question. The
manifest-less directory (§6) keeps 0008's behavior unchanged: `main.cnr` at the
directory root, no `src/`.

### 2. The manifest — `candor.toml`, minimal, formatter-canonical

**The manifest is TOML** (`candor.toml`), parsed by the one toolchain,
**canonicalized by the shipped formatter** (P3/NN#11: one canonical form — the
formatter owns the manifest's shape exactly as it owns source), and validated
against a **closed schema** (an unknown key is an error, not silently ignored —
required for reproducibility and for a future migrator to reason about every
field). TOML is chosen as declarative *data*, not code (§Rejected rejects a
`.cnr` manifest). The schema is kept as small as convention-over-config allows:

```toml
[package]
name = "http"           # required. globally-unique identity (0008 §1).
version = "0.3.1"        # required. semver (§3).
edition = "2026"         # required. the language edition this source is written in (§3, P15).
freestanding = true      # optional (default false). the verified claim of 0008 §5.

[lib]                    # optional. present => this package exposes a library API.
                         # root module: src/http.cnr (or namespace-only src/).

[[bin]]                  # optional, repeatable. a buildable binary target.
name = "httpc"           # entry: fn main in src/bin/httpc.cnr (single-bin: src/main.cnr).

[dependencies]
json = { path = "../json" }                             # a local path dependency (§4)
tls  = { git = "https://…/tls.git", rev = "a1b2c3d4" }  # a git dependency (§4)
```

Only `[package].name` is strictly irreducible; `version` and `edition` are
required because neither is derivable (a version is a human claim; an edition
cannot be inferred without breaking migration — §3). Everything else is optional
and conventional. **A package with a `[lib]` and no `[[bin]]` is a library; with
`[[bin]]` targets it also builds binaries; the two coexist** (a package may be
both, like Cargo). Absent both, a package with `src/main.cnr` is an implicit
single binary (convention). This target model is the minimum needed to answer
"what does building this package produce" (a linkable library API, an executable,
or both) — it is not a build-script or feature-flag system (none of which 0.x
ships; P6).

### 3. Version and edition — two orthogonal axes

**Package version: semver** (`major.minor.patch`). The breaking/compatible
distinction maps onto 0008 §2's interface artifact: a package's public API *is*
the set of its externally-reachable `pub` signatures (0008 §3 reachability), and
its aggregate **signature hash** (0008 §2) is the machine-checkable oracle for
**"did the *typed* public interface change"** — **not** a complete breaking-change
oracle. It is a one-directional witness: a signature-hash delta *is* a
public-interface change, but a breaking change can occur with **no** signature-hash
delta, because the signature hash misses (1) **behavioral** changes that keep the
same types, (2) `inline`/generic **body** changes (0008 §2: those move the
*codegen* hash, not the *signature* hash, yet a consumer compiles those bodies),
and (3) **trust-surface weakening** inside a `boundary` module. This still ties one
honest direction of semver to a fact the toolchain already computes. Crucially,
P2's effect-upper-bound rule makes that direction bite: a leaf function that
*starts allocating* ripples its `alloc` marker up through every `pub` signature
above it (P2, "the churn is the truth"), which changes the package's signature
hash — a **major** version bump. Semver is therefore the honest carrier of P2's
"API churn when the truth changes" refusal.
*(Optional 0.x lint, not required — and strictly **one-way**: the toolchain may
warn when a **signature-hash change carries no version bump** (an under-bump, the
unsound direction). It must **not** warn on a major bump with no hash delta — that
is a *correct* bump for a behavioral, `inline`/generic-body, or trust-surface break
the signature hash cannot see. A P4/P16 provenance aid.)*

**Language edition: a separate axis** (`edition = "YYYY"`), the language's own
breaking-change unit (P15). It is orthogonal to package version: `http 3.0.0` may
still be `edition = "2026"`, and a package may raise its edition without a major
version if the migrator (NN#14) makes the change mechanical. **Editions are
required and never inferred**, because P15/NN#14's automatic-migrator promise
depends on knowing the *starting* dialect: you cannot mechanically migrate code
whose edition is unknown. Old editions keep compiling (NN#15's spirit); the
compiler accepts every shipped edition simultaneously, and **cross-edition
dependencies link normally** — a package at edition 2028 may depend on one at
edition 2026, because editions change surface syntax and check rules, *not* the
edition-agnostic interface artifact (checked signatures + IR, 0008 §2) through
which a dependency is consumed. This is the coherence that makes editions
composable across the dependency graph.

**This composability is not free — it binds every future edition (review F4).**
"Cross-edition deps link normally" holds *only because* the interface artifact is
edition-agnostic, which in turn **requires that no future edition ever change the
*semantics* of the interface artifact** (the meaning of a signature + the checked
IR). If a post-1.0 edition ever did — e.g. a Bet 5 re-measurement failure forcing a
memory-model rework, the exact un-migratable break NN#14 guards — then "link
normally" across that edition boundary would become **unsound**. There is a
corollary cost: because git dependencies are rev-pinned and therefore
**unmigratable in place**, the compiler must **retain every shipped edition's
front-end** to keep consuming them. This is a constraint the eventual 1.0 edition
mechanism *inherits* (recorded in §Settled), not a free consequence of the design.

**Pre-1.0 reality (P15/NN#14, ROADMAP step 3).** There is exactly **one legal
edition value today** (the 0.x preview edition, written `"2026"` here as a
placeholder for the authority to name). No stability commitment may precede Bet
5's verdict (NN#14; the v4.2 disposition provisionally confirmed Bet 5 with
binding re-measurement commitments). The edition *field* exists now to reserve
the mechanism — adding it later would be the un-migratable break the field
exists to prevent. Until 1.0, an edition bump is not yet exercised; the field is
the load-bearing reservation.

### 4. Dependency sources — path and git; registry deferred

A dependency names an exact **source**, not a version range (there is no
registry to resolve ranges against — §Rejected):

- **Path** — `{ path = "../json" }`. A local directory (a sibling package, a
  workspace member, or a vendored copy). Resolved relative to the depending
  manifest. This is the monorepo/workspace and local-development case, and it
  subsumes vendoring (§Rejected: no separate vendoring mode).
- **Git** — `{ git = "URL", rev = "<commit-sha>" }`. `rev` pins an **exact
  commit** for reproducibility (P16/NN#16: reproducible by default). A `tag` or
  `branch` may be written for convenience, but the **lockfile records the
  resolved commit sha** it points to, so the *build* is always pinned even when
  the manifest is not. Git sources are fetched into a toolchain-managed,
  content-addressed cache (keyed by URL + sha); they are **not** committed into
  the depending repository (reproducibility comes from the lock's pinned sha, not
  from checking in copies).

**A central registry and its network-fetch protocol are deferred** (§Rejected,
with justification). The `[dependencies]` value is an inline table whose keys
select the source kind, so a future registry source
(`{ registry = "…", version = "^1.2" }`) slots in **without breaking any existing
manifest** — the schema is forward-compatible by construction. **Provenance is
recorded regardless of source** (P16/NN#16): the lockfile (§6) records, per
resolved package, its exact source (canonicalized path, or git URL + resolved
commit) and a content hash — answering P16's stated audit question, "what exactly
is in this binary and where did it come from."

### 5. Cross-package imports — a package is a namespace root (extends 0008 §3)

This is the load-bearing 0008 interaction, and it needs **no new visibility
mechanism** — 0008 §3 already built it.

**A dependency package appears as a single root namespace named for the
dependency**, and a `use` crosses the package boundary by naming it as the first
path segment:

```
use json::parse::{decode};   // `json` is a [dependencies] entry;
                             // `parse` is a module within json's src/ root;
                             // `decode` is one of its externally-reachable pub items.
```

This is a strict *extension* of 0008 §3's `::` path model, not a contradiction:
0008 already resolves `net::tcp` as filesystem position within the build root;
0017 lets the **first segment be a dependency name** (from `[dependencies]`), with
the remainder resolving as filesystem position within *that dependency's* `src/`
root. The current package's own modules are still addressed by their bare paths
from its own `src/` root (unchanged from 0008). The `::` / `.` lexical
distinction (0008 §3, parse-without-symbol-table) is preserved.

**What crosses the boundary is exactly 0008 §3's externally-reachable surface.**
0008 §3 defines an item as externally reachable "iff it is `pub` **and** there is
a path from the package's public root to its module in which every step is a
public module re-export" (`pub use`). That predicate — written for "external
consumers" — is *precisely* the package API wall. A dependency's
package-internal modules (`pub` but not re-exported from its public root) are
**invisible** to a consumer; only its public root's `pub` surface and `pub use`
chains cross. **The package boundary reuses 0008 §3 verbatim**; "crate-internal
helper" (0008's phrase) means "not re-exported past the package's public root,"
which is now literally "not visible to a dependent package." No `pub(crate)` is
needed or added (0008 §3, P6).

**Name-collision rule (locally checkable).** The set of dependency names (from
`[dependencies]`) and the set of the package's own top-level `src/` module names
must be **disjoint**; a collision is a compile error (P4 diagnostic) whose fix is
to rename the local module or **alias the dependency** in the manifest
(`[dependencies]` key is the local name; a `package = "…"` field names the
underlying package when they differ). This is decidable from the manifest plus a
`src/` listing — no whole-program analysis.

**Linking model (preserves the merged `Program`).** The existing pipeline merges
the whole module tree into one name-qualified `Program` by mangling each item to
`module::name` (`prototype/src/modules.rs`). This extends by prepending an
**injective package-identity** segment to the mangle: an item becomes
`<pkgid>::module::name`, where `<pkgid>` is the package **name + its resolved-source
hash** (§6). **The prefix is applied to *every* item in the merged table, including
the root (buildable) package's own items — not only dependencies' items.** That is
what makes the merge collision-proof: because the root is prefixed too, a local
top-level module can never collide with a *transitive* dependency that happens to
share its name (the §5 disjoint-check catches only *direct* dependency names, so it
cannot be relied on for the transitive case), and two distinct versions/sources of
`util` cannot collide in the merged table. It also secures **cross-package
acyclicity**: two same-named modules from different packages carry distinct
`<pkgid>` prefixes and therefore never merge into one node of the module DAG (§8;
review F6b). The merged `Program` is still fed unchanged to the resolver / checker /
interpreter (as `modules.rs` does today); **cross-package linking introduces no new
mechanism for 0.x** — it is the same merge, with package-qualified names. (Separate compilation, dynamic linking, and a stable
native ABI are P14's C-ABI story, out of scope — §Scope.)

### 6. Resolver and lockfile — simplest correct for 0.x

**Selection: exact-source pinning with single-version-per-package unification.**
With path + git sources (no registry, §4) a dependency names an *exact* source,
so there are **no version ranges to solve** and therefore no solver: resolution
is a transitive walk of the dependency graph collecting every reachable package
and its pinned source. This is deliberately **not** MVS and **not** a SAT/PubGrub
solver (§Rejected) — those solve a problem 0.x does not have.

- **Diamonds.** If two dependents reach the same package name via the **same**
  source, it is unified to one build node (deduped). If they reach it via
  **different** sources/versions, 0.x raises a **hard conflict error** naming both
  request paths (a P4 diagnostic), leaving resolution to the user (bump one, or
  point both at one path). A single version of each package name exists per build
  (P3: one canonical resolution; §Rejected rejects the npm/Cargo multiple-versions
  model for 0.x). `[patch]`/override machinery is deferred (§Open-questions).

- **The lockfile: `candor.lock`** (TOML, formatter-canonical), written at the
  **root (buildable) package**. For each resolved package it records: `name`,
  `version`, `edition`, the **exact source** (canonicalized path, or git URL +
  resolved commit sha), and a **content hash** of the package's sources
  (reproducibility + provenance, P16/NN#16). **Same source + same lock + same
  toolchain ⇒ bit-identical artifacts** (NN#16, verbatim).

- **When written.** The lock is created/updated only by resolution triggered by a
  manifest change or an explicit update action — **never silently re-resolved on
  an ordinary build** (a floating re-resolve is not reproducible). A present lock
  consistent with the manifest is used verbatim; a manifest that adds a dependency
  absent from the lock resolves just that addition and updates the lock. (Cargo
  semantics, chosen because they satisfy NN#16 and match the model corpus, a
  Bet 4/6 tailwind.)

### 7. The build driver — resolve, build-per-package, merge

`candor build` extends today's `build_tree` / `build::build_dir`
(`prototype/src/`) as:

1. **Read** the root `candor.toml`.
2. **Resolve** (§6): transitively read each dependency's manifest, produce the
   pinned package set, and write/verify `candor.lock`. Reject a package-level
   dependency cycle here (§8).
3. **Build each package** in dependency-topological order via the existing
   per-package tree builder (`build_tree_parts` / `build_dir`, 0008 §2),
   producing each package's **interface artifacts** (0008 §2: signatures + the two
   hashes).
4. **Resolve cross-package `use`** against the dependency's *interface artifacts*
   (0008 §2: "a module is consumed only through its compiled interface artifact …
   no textual inclusion"). **A dependency is consumed exactly the way 0008 §2
   already prescribes a module be consumed** — the package boundary *is* a natural
   interface-artifact boundary.
5. **Merge** the checked IR into one `Program` with package-qualified mangling
   (§5) for the checker/interpreter/codegen path (as `modules.rs` merges today).

**Interaction with 0008 §2's incremental model — a win, not a cost.** Because a
dependency crosses only through its interface artifacts, cross-package builds
inherit 0008 §2's signature-bounded invalidation *at the package granularity*:
rebuilding a dependency's bodies does **not** re-analyze a dependent unless the
dependency's public **signature hash** changed (P20, delivered across packages by
the same mechanism 0008 §2 delivers it within one). The content-addressed codegen
cache (0008 §2.4, with its toolchain salt) extends its key to include package
identity + version (§5), so two versions of a package cannot alias a cache entry.
For the prototype's current *merge-everything* stage, the honest 0.x MVP is
step-5's merge with package-qualified names (works today); the per-package
interface-artifact resolution of steps 3–4 is the incremental upgrade (0008 §2
stages 3–4), not a prerequisite for the first working multi-package build.

### 8. Soundness and coherence

- **Acyclicity is preserved at two composed levels.** The **package** dependency
  graph must be acyclic (a package cannot transitively depend on itself) —
  enforced by the resolver (§6) with a P4 cycle diagnostic in the shape of 0008
  §3's module-cycle diagnostic, one level up. The **module** DAG across the merged
  tree remains acyclic — enforced unchanged by the merged-`Program` checker (0008
  §3). Cross-package edges only ever point from a dependent to a dependency's
  public API (a dependency's manifest cannot name a package that depends on it),
  so **whole-program acyclicity holds by construction**, and the merged `Program`
  the checker sees is still one acyclic module DAG. P20's topological order (0008
  §3's stated reason acyclicity is non-negotiable) is preserved.

- **Visibility needs no new mechanism** (§5): 0008 §3's external-reachability
  predicate *is* the package API wall, reused verbatim.

- **The single merged-`Program` model is preserved** (§5): cross-package linking
  is package-qualified mangling into the same merged program; no new linking
  mechanism for 0.x.

- **The memory/effect model is untouched by crossing a package boundary.** A
  dependency's `pub` signatures already carry their full effect sets (0008 §2,
  P2: `alloc` today, `foreign` per 0011), so importing them is identical to
  importing a local module — the closed effect set travels on signatures across
  packages exactly as within one (NN#17/NN#19). The **`freestanding` claim
  composes across the graph**: 0008 §5 derives freestanding-compatibility from the
  transitive import DAG — now including cross-package edges — so a package that
  declares `freestanding = true` but pulls in a dependency that transitively
  imports `std` **fails the build** (0008 §5's tool-checked claim, extended across
  packages). **The `std`-import check is necessary but not sufficient:** 0011 §5
  makes a `foreign` extern under `--freestanding` a **compile error** (there is no
  libc to link), so a dependency that imports only `core` yet exposes a transitive
  `boundary`/`foreign` extern violates the claim even though its import DAG never
  touches `std`. Freestanding composition must therefore reject any **transitive
  `boundary`/`foreign` surface**, not only a transitive `std` import — and the
  whole-graph audit (above) already enumerates every boundary module in the
  resolved graph, so the data to enforce "freestanding claim ⇒ zero transitive
  foreign surface" already exists. The claim thus becomes a *dependency
  constraint*, coherently with P9's layering (`core`/`std` are themselves packages,
  0008 §5).

- **The foreign / audit trust boundary — the adversary's target — is aggregated,
  not hidden.** A dependency may contain `boundary` modules with `extern "C"`
  foreign declarations (0011), `unsafe` regions (P1), and `assumed-proven`
  contracts (P8). When A depends on B, **B's foreign surface becomes part of A's
  trusted computing base**, and P17 §7's "show me everything this program trusts"
  must see it. Therefore `candor audit` (0011 §6) is extended to walk the **whole
  resolved package graph**, not just the local package: every `boundary` module in
  every transitive dependency is enumerated, **attributed to its package +
  version + source** (P16 provenance). The "no hidden I/O" guarantee rests on this
  **structural whole-graph audit walk, not on the effect system riding across the
  boundary**: 0011 §2 rule 4 makes a *discharged* boundary wrapper export a
  **non-`foreign` signature** (the effect is born at the extern and extinguished at
  the wrapper — the healthy norm), so a consumer that imports such a wrapper sees a
  *pure* signature with nothing in its effect set. The `foreign` effect *does* ride
  on signatures for **un-discharged / propagating** surface — and the partition of
  0011 §2 is preserved across packages exactly as within one — but the guarantee
  cannot depend on that, because the discharged wrapper is the norm; it depends on
  the structural walk enumerating every boundary module graph-wide.

  **Implementation prerequisite (review F1b).** The aggregation as promised is a
  strict superset of what the shipping `candor audit` (`prototype/src/audit.rs`)
  computes today: the current tool enumerates **only** the `boundary` foreign
  surface (externs/exports + effect-reach) and walks **no** `unsafe` regions and
  **no** `assumed-proven` contracts — even for the local package. That already
  falls short of philosophy §7's own success criterion ("lists boundary modules,
  unsafe regions, assumed-proven contracts … with one command"). 0017's whole-graph
  audit therefore **depends on first extending `candor audit` to enumerate `unsafe`
  + `assumed-proven`** graph-wide; that extension also repairs the pre-existing
  local-audit gap, and is the **first implementation slice** this document names.

  The lockfile (§6) additionally records a per-dependency **trust summary** (counts
  of boundary modules / `unsafe` regions / `assumed-proven` contracts) so that
  *adding or updating* a dependency surfaces its trust delta in review — the
  supply-chain question P16 names. **The 0.x policy is enumerate-only, not
  gating.** A named residual risk: a git-rev bump could silently introduce a new
  boundary module; the content hash + trust summary make the delta *visible*, but a
  *gating* trust-delta diff on lock updates is a **first-1.0 need** (Open-Q1), not
  delivered here.

## Rejected alternatives

- **A `.cnr` (Candor-syntax) manifest instead of TOML.** Rejected. It mixes code
  with data and creates a bootstrap tangle (you would need to run Candor to
  configure building Candor) and invites arbitrary computation in the manifest —
  the non-reproducible, non-declarative wound `build.rs` is in Cargo. A manifest
  is declarative data the toolchain reads once; TOML canonicalized by the
  formatter (P3) keeps "one canonical form" without inventing a grammar (P6). The
  self-hosting elegance of a Candor manifest does not pay for the reproducibility
  and simplicity it costs.

- **A central registry + SemVer ranges + a SAT/PubGrub (or full-MVS) resolver,
  now.** Rejected under P6 and §8-sequencing. A registry is heavy standing infra
  (hosting, a naming authority, signing, availability, a fetch protocol,
  security), and — decisively — **without a registry there are no version ranges
  to solve**, so a solver solves a problem 0.x does not have. Path + git unblock
  the actual goal ("A depends on B, build them together") for real. The
  `[dependencies]` inline-table schema (§4) leaves a forward-compatible seam so a
  registry source slots in later without breaking existing manifests.

- **No manifest / directory-only forever (pure 0008).** Rejected. A bare
  directory cannot express a dependency, a version, or an edition — and "others
  build on it" *is* A-depends-on-B at a named version. 0008 §1 already conceded
  that package *name* is irreducibly non-local and cannot come from the
  filesystem; a dependency edge, a version, and an edition are the same kind of
  non-local fact, and belong in the same minimal manifest.

- **Vendoring-only (copy dependencies into your tree; no resolver, no lock).**
  Rejected under P16/NN#16. It has no provenance record, no dedup (diamonds
  duplicate on disk), and only manual updates — the opposite of reproducible,
  provenance-aware builds. Vendoring remains *available* as an ordinary path
  dependency (§4), which is the honest place for it.

- **Lockfile-free (resolve fresh on every build).** Rejected under NN#16:
  reproducible-by-default is a Non-Negotiable, and a floating re-resolve is by
  definition not reproducible. The lock is the mechanism NN#16 names.

- **The npm/Cargo model of multiple private versions of one package in a single
  build.** Rejected for 0.x under P3/P6 — this is the named Candor divergence from
  Cargo. Allowing many versions of `X` is where diamond type-incompatibility bugs
  and binary bloat live; single-version-per-package unification with a hard
  conflict error (§6) is simpler and one-canonical (P3). If real-world diamonds
  make this too strict, the escape is a `[patch]`/override mechanism (deferred),
  not silent multi-versioning.

## Consequences and costs

- **A manifest is config, and this document spends more of it than 0008 did.**
  0008 held the manifest to one identity line plus an optional claim and called
  that "minimal config, not no config" honestly (0008 §Consequences). This
  document adds `version`, `edition`, `[dependencies]`, and an optional target
  stanza. Each is defended as irreducibly non-local (a version and an edition are
  human/temporal facts the filesystem cannot supply; a dependency edge is
  external by nature), but the manifest is no longer one line — the P16 budget is
  spent, and named as spent.

- **The `edition` field is a no-op today.** With one legal edition, the required
  field carries no information yet — a real cost (a mandatory field that does
  nothing looks like ceremony). It is paid deliberately: adding the field *after*
  packages exist in the wild is exactly the un-migratable break the field exists
  to prevent (P15/NN#14). The reservation is the point.

- **The `src/` refinement touches an 0008 convention.** Relocating a manifested
  package's module root from `main.cnr`-at-root (0008's ruling) to `src/` is a
  small break for anyone who read 0008 literally. It is migratable (mechanical
  move) and the manifest-less directory is unchanged, but it is a refinement of a
  neighboring design and is **ACCEPTED by the deciding authority (2026-07-15)**,
  recorded as an erratum to 0008 §2.4 (review F6a) — not slipped in, and no longer
  an open question.

- **The whole-graph audit is only as good as the walk.** Extending `candor audit`
  across the dependency graph makes the trust surface *visible*, but visibility is
  not verification (P17's standing honesty: boundary contracts are mostly trust,
  not proof). A large dependency graph has a large trusted surface, and this
  document enumerates it rather than shrinking it. The supply-chain gate (a
  trust-*delta* check on lock updates) is named as a first-1.0 need (Open-Q1), not
  delivered here.

- **Single-version unification will sometimes say no.** A hard conflict error on
  a version diamond (§6) is simple and correct but less permissive than Cargo; in
  a young ecosystem with few packages this is cheap, but it will occasionally
  force a human to reconcile versions the toolchain could (with more machinery)
  have reconciled itself. Accepted for 0.x; the override seam is the escape.

## Settled by disposition (2026-07-15)

*The deciding authority (k1832) **ACCEPTED** 0017 subject to the review repairs
(F1a–F6a, applied above), **ACCEPTED** the `src/` root move, and chose an
**enumerate-only** supply-chain audit for 0.x (gating deferred to first-1.0). The
following are no longer open:*

- **The `src/` module root (was Open-Q3; review F6a) — ACCEPTED.** A manifested
  package's module tree roots at `src/` (§1); the change is recorded as a dated
  **erratum to 0008 §2.4** (`docs/design/0008-modules.md`) so the two neighboring
  designs do not quietly diverge (GOVERNANCE §9). The manifest-less directory keeps
  0008's original `main.cnr`-at-root behavior unchanged. The `modules.rs`
  implementation of `src/` support lands with the packaging implementation, not
  now.

- **Cross-edition linking imposes an invariant the 1.0 edition mechanism inherits
  (was part of Open-Q2; review F4).** Recorded in §3: cross-edition "link normally"
  is sound *only* while every future edition preserves the *semantics* of the
  interface artifact (signature meaning + checked IR); an edition that ever changed
  them (e.g. a post-1.0 Bet 5 re-measurement forcing a memory-model rework) would
  make cross-edition linking unsound, and the compiler must retain every shipped
  edition's front-end (git deps are rev-pinned, unmigratable in place). This is a
  constraint the eventual 1.0 edition mechanism (ROADMAP step 3) inherits, not a
  free consequence. The remaining second-edition *migrator-run* mechanics travel
  with that 1.0 work.

## Open questions and risks (remaining)

*Ordered by how much the author wants the deciding authority to keep watching them.
Open-Q2 (the editions invariant) and Open-Q3 (`src/`) are resolved above; the
identifiers Open-Q1/Q4/Q5 are kept stable.*

1. **Trust-delta *gating* for the supply chain — a first-1.0 need, not 0.x
   (Open-Q1).** §8's 0.x policy is settled as **enumerate-only** with a visible
   per-dependency trust delta in the lock; the open item is the **gating** upgrade
   — a hard trust-delta check that fails a lock update introducing new
   foreign/`unsafe`/`assumed-proven` surface — named as a **first-1.0**
   requirement. Fetching and building a git dependency's `boundary` module remains
   the largest new attack surface the package system opens (P17's domain). The
   `candor audit` extension (F1b) is the first implementation slice this document
   depends on.

4. **Incremental-build interaction (coherent in sketch, heavy in
   implementation).** §7 makes the package boundary the interface-artifact
   boundary (0008 §2 stages 3–4) and salts the codegen cache key with package
   identity + version. The story is coherent, but the implementation weight (0008
   §2's two-hash machinery, now per package, plus a resolved-graph cache) is real
   and lands on the same stages 0008 §6 already flagged as the expensive ones.

5. **Single-version unification vs real diamonds.** Is a hard conflict error
   acceptable for the first release, or will path/git diamonds be common enough
   that a `[patch]`/override mechanism must ship in 0.x rather than being deferred?
