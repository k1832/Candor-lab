# Adversarial review — 0017 Packages, Manifests, Dependency Resolution

**Design:** [docs/design/0017-packages.md](../design/0017-packages.md)
**Date:** 2026-07-15
**Reviewer:** adversarial pass (fresh context), mandate: break it — a soundness /
security / coherence hole, or a way it fails its own goals.
**Verdict:** **SURVIVES WITH REPAIRS.** No SINK — the "no hidden I/O" property is
*achievable* and consistent with P17's enumerate-don't-verify model — but the headline
"aggregated audit" claim is only partially sound as written, and the review found a
**pre-existing gap in `candor audit`** (it enumerates only the boundary foreign surface,
not `unsafe`/`assumed-proven`), plus several coherence repairs against 0008/0011.

Dispositions are the deciding authority's; recorded inline per finding once made.

---

## F1a — The effect-propagation claim is false for discharged wrappers. Severity: REPAIR

§8 says *"a safe package that imports a function transitively reaching foreign code sees
the effect in the signature."* 0011 §2 rule 4 says the opposite is the **healthy norm**:
a boundary wrapper that discharges its trust *exports a non-`foreign` signature* — the
effect "is born at the extern and dies at the wrapper, never appearing in the wider call
graph." So a dependency's `pub fn read_config() -> Config` wrapping an I/O extern exposes
a **pure** signature; the consumer sees nothing. The headline "no hidden I/O" property
therefore rests **entirely on the structural whole-graph audit walk, not the effect
system**. **Repair:** strike the effect-propagation sentence; attribute the guarantee to
the audit walk only.
**Disposition:** ACCEPTED (2026-07-15). Repair applied to 0017 §8: the false
effect-propagation sentence is struck; the "no hidden I/O" guarantee is attributed
to the structural whole-graph audit walk (0011 §2 rule 4 makes a discharged wrapper
export a non-`foreign` signature — the norm), not to the effect system riding across
the boundary.

## F1b — The aggregated audit covers a strict subset of the TCB. Severity: REPAIR (security-critical); also a **pre-existing gap in `candor audit`**

§8 promises to aggregate the dependency graph's `foreign`/**`unsafe`**/**`assumed-proven`**
surface. But the cited mechanism (`prototype/src/audit.rs:107-158,181-204`) enumerates
**only** `Item::Extern`/`Item::Export` in `boundary` files + `effect_reach` — it walks
**no** `unsafe` regions and **no** non-boundary `assumed-proven` contracts, *even for the
local package today*. A git dependency with a plain `unsafe` block doing unchecked pointer
arithmetic (a memory-safety trust the consumer inherits) appears in `candor audit`
**nowhere**. The philosophy §7 success criterion ("lists boundary modules, **unsafe
regions**, **assumed-proven contracts** … with one command") is **not met by the current
tool**. So 0017's promised TCB enumeration is a strict subset of the real TCB — and the
shortfall exists in the shipping `candor audit` independent of 0017.
**Repair:** the audit must walk `unsafe` + all `assumed-proven` graph-wide (and fix the
local audit to match philosophy §7); state that 0017's aggregation depends on it.
**Disposition:** ACCEPTED (2026-07-15). Repair applied to 0017 §8: the aggregation
now *depends on* first extending `candor audit` to enumerate `unsafe` +
`assumed-proven` graph-wide (matching philosophy §7), which also fixes the
pre-existing local-audit gap; 0.x policy stays enumerate-only with gating deferred
to Open-Q1. **The `candor audit` extension is queued as the first implementation
slice.**

## F2 — Package-qualified mangling: root package not prefixed → mis-link. Severity: REPAIR

`prototype/src/modules.rs:94` mangles to `module::name` with **no package prefix today**;
the bare-`main` special case (line 95) shows a flat merged namespace. §5 claims a
`<pkgid>::module::name` prefix but is ambiguous whether the **root** package's items are
prefixed. If not: `app` (local top-level module `util`) → dep `widget` → real package
`util` yields two `util::…` subtrees in one flat table → mis-link. The §5 disjoint-check
covers only **direct** dep names ("no whole-program analysis"), so it can't catch a
collision with a **transitive** dependency's package name.
**Repair:** prepend an **injective** pkgid (name + resolved-source hash) to **every** item
including the root package's. (Also closes the attack-6b acyclicity concern — two
same-named modules must not merge into one DAG node.)
**Disposition:** ACCEPTED (2026-07-15). Repair applied to 0017 §5: an injective
pkgid (name + resolved-source hash) is prepended to **every** item including the
root package's own, so a local module cannot collide with a transitive dependency's
package name; this also secures cross-package acyclicity (closes F6b).

## F3 — The semver signature-hash "oracle" overclaims; the lint is mis-directional. Severity: REPAIR

§3: *"the aggregate signature hash is the machine-checkable oracle for 'did the public API
change'."* The signature hash (0008 §2) covers the **typed** interface only; it misses
(1) behavioral changes with unchanged types; (2) `inline`/generic **body** changes (0008
§2: those change the *codegen* hash, not the *signature* hash — yet consumers compile
those bodies); (3) trust-surface weakening inside a boundary module. The optional lint
warns when a version bump *disagrees* with the hash delta — so a **correct major bump**
for a behavioral/inline/trust break (no hash delta) is flagged, pressuring authors to
*under*-bump. **Repair:** soften the claim to "the typed interface"; make the lint one-way
(flag hash-change-**without**-bump only). (Same-source-same-content-hash is closed.)
**Disposition:** ACCEPTED (2026-07-15). Repair applied to 0017 §3: the oracle is
softened to "did the *typed* public interface change" (it misses behavioral,
`inline`/generic-body, and trust-surface breaks); the optional lint is made one-way
— flag a signature-hash change *without* a version bump only, never a major bump
with no hash delta.

## F4 — Cross-edition linking silently constrains every future edition. Severity: REPAIR (1.0-gating)

§3 asserts cross-edition deps link "because editions change surface syntax and check
rules, not the edition-agnostic interface artifact." That is a **binding constraint 0017
imposes on all future editions**: an edition may **never** change the *semantics* of the
interface artifact (signature meaning + checked IR). If a post-1.0 Bet 5 re-measurement
failure forced a memory-model rework (the exact un-migratable break NN#14 guards),
"link normally" becomes **unsound** across editions — and because git deps are rev-pinned
(unmigrable in place), the compiler must retain every edition's front-end forever (an
unnamed cost). **Repair:** record the "editions must preserve interface-artifact
semantics" invariant explicitly as a constraint the 1.0 edition mechanism inherits, not a
free consequence.
**Disposition:** ACCEPTED (2026-07-15). Repair applied to 0017 §3 (and recorded in
the §Settled disposition section): cross-edition linking imposes a binding invariant
— no future edition may change interface-artifact semantics — with the corollary
that the compiler retains every shipped edition's front-end; a constraint the 1.0
edition mechanism inherits.

## F5 — `freestanding` composition checks the wrong axis. Severity: REPAIR

§8 extends only the **std-import** DAG check. But 0011 §5 also makes a `foreign` extern
under `--freestanding` a **compile error** (no libc to link). Attack: `blink`
(`freestanding=true`) → `hal` (`freestanding=true`, imports only `core`) which has a
`boundary` module with `extern "C" { fn hal_write(...) foreign }`. The import DAG never
touches `std`, so 0017's check passes — yet the transitive libc extern violates 0011 §5.
The data exists (the whole-graph audit enumerates every boundary module). **Repair:**
freestanding composition must reject any **transitive `boundary`/`foreign` surface**, not
just `std` imports.
**Disposition:** ACCEPTED (2026-07-15). Repair applied to 0017 §8: `freestanding`
composition rejects any **transitive `boundary`/`foreign`** surface (0011 §5), not
only a transitive `std` import; the whole-graph audit already enumerates the data.
**Implemented 2026-07-15** — a post-resolution composition check gated on the
**root's** `freestanding` claim walks the pinned package set and rejects (E0935)
any package that contributes a `boundary`/`foreign` surface, and any transitive
import of the `std` package (0008 §5). The boundary/foreign surface is read from
the audit's structural enumeration (`audit::first_boundary_surface`, reusing
`structural_surface` — the single source of truth), so a **declared-but-uncalled**
transitive `foreign` extern (the `blink -> hal` escape) is caught; the diagnostic
names the offending package + the specific boundary module / extern. The gate is
the final artifact's property, not each dependency's own flag. Files:
`prototype/src/resolve_pkg.rs` (`check_freestanding_composition`),
`prototype/src/audit.rs` (`first_boundary_surface`); gate:
`prototype/tests/packages.rs` (F5 escape with an uncalled extern, transitive std
import, legit freestanding composes, non-freestanding unaffected).

## F6a — `src/` root contradicts 0008 + the implementation; a governance obligation. Severity: REPAIR

0008 §2.4 ruled the directory-build root is the **root-level `main.cnr`**, and
`prototype/src/modules.rs:242-249` **enforces** it (E0905). 0017 §1's `src/main.cnr`
contradicts both. GOVERNANCE §9 forbids *quiet* divergence — a **deferred** Open-Q3 is a
lingering disagreement between two neighboring designs plus a shipped implementation.
**Repair:** the authority must actually **issue an 0008 erratum** (accept the `src/` move
and amend 0008 + `modules.rs`) **or reject** the `src/` move — not defer it.
**Disposition:** ACCEPTED (2026-07-15) — the `src/` root move is accepted, no longer
deferred. Applied to 0017 §1/§Consequences (moved to §Settled) and issued as a dated
**erratum to 0008 §2.4** (`docs/design/0008-modules.md`), per GOVERNANCE §9. The
`modules.rs` change lands with the packaging implementation. **Implemented 2026-07-15** — the `src/` module-root relocation shipped (`prototype/src/modules.rs`, `prototype/src/build/mod.rs`; gate `prototype/tests/packages.rs`); see the dated erratum note in 0008 §2.4.

## F6b — Package + module acyclicity: closed by construction (conditional on F2)

With an acyclic package DAG (resolver-enforced) cross-package edges point only
dependent→dependency, so the merged module DAG stays acyclic — **provided** the injective
pkgid (F2) keeps two same-named modules from different packages from merging into one
node. No dev-dep/build-dep stanza exists → no build-graph cycle vector. **Closed**, given
F2.
**Disposition:** Closed by F2 (2026-07-15) — the injective pkgid keeps two
same-named modules from distinct packages from merging into one module-DAG node.

---

## What is genuinely closed (strengthens the design)

- **No execution-during-resolution** hidden-I/O vector: `extern` outside a `boundary`
  file is a parse error (0011 E1101), code outside the module root isn't compiled, and a
  `.cnr`/`build.rs` manifest is rejected — foreign *calls* are structurally confined.
- **Private-item leakage across packages is closed**: `modules.rs:216-229` rejects
  non-`pub` imports (E0903); the package wall reuses 0008 §3 `pub`-reachability verbatim.
- **Same-source-same-code** is guaranteed by the content hash (no semantic-drift-under-one-source).

## Overall

0017's core is sound and elegant (the 0008 visibility reuse, semver-tied-to-signature-hash,
single-merged-Program linking). The "no hidden I/O" property is **real iff** the whole-graph
walk is (i) implemented, (ii) extended to `unsafe` + all `assumed-proven` (F1b), and (iii)
the trust-**delta** gating (Open-Q1) is treated as first-release, not aspirational.

**Required before acceptance:** F1a (strike the false effect claim), **F1b** (audit must
cover `unsafe`/`assumed-proven` graph-wide — the load-bearing one, and it exposes a
pre-existing `candor audit` gap vs philosophy §7), F2 (injective pkgid incl. root), F3
(soften the oracle; fix the lint direction), F4 (record the edition-preserves-interface
invariant), F5 (freestanding rejects transitive foreign), F6a (issue an actual 0008
erratum — a governance obligation, not an option). F1b, F2, F5 are load-bearing; F3/F4
protect the 1.0 gate.
