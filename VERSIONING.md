# Versioning

Candor has **two independent version axes**. Do not conflate them:

1. **The language** — `0.x` → `1.0` → editions. Governs what the *compiler and
   semantics* promise.
2. **A package** — semver (`major.minor.patch`). Governs what a *library's public
   interface* promises to its dependents (design 0017 §3).

This document is the authoritative policy. It synthesizes philosophy P15/P18/P20,
NN#14/#16, and design 0017; where it and those disagree, they win and this is a bug.

---

## 1. The language

### 1.0 is a *gate*, not a date

Candor commits to stability only when a checklist clears — never on a schedule
(philosophy §1; ROADMAP publication staging). Until then the version is `0.x` and
means exactly one thing: **preview, no promises.**

### `0.x` — the current state

- **No stability promise of any kind.** Syntax, semantics, diagnostics, the standard
  library, and the CLI surface may all change **without notice and without a
  migration path** before 1.0. Do not build anything load-bearing on a `0.x` release.
- The `0.x` number is **informational**, not a compatibility contract. A bump from
  `0.4` to `0.5` signals "the preview moved," not "here is what broke." Pin the exact
  toolchain commit if you need reproducibility during the preview.
- This is required by **NN#14**: no stability commitment may precede Bet 5's
  pre-registered verdict. That verdict is in — Bet 5 is *provisionally confirmed* —
  so the remaining gate to 1.0 is the checklist below, not the founding bet.

### The 1.0 gate (what must clear before any stability promise)

From ROADMAP publication step 3 (philosophy P15/P18/P20):

1. **P15 editions + automatic migrator machinery is live and exercised in CI** — the
   evolution mechanism works before it is relied upon.
2. **P20's pre-registered compile-time targets are ratified in CI** as release
   criteria (or the compile-speed claim is withdrawn by amendment).
3. **The spec's obligations ledger is clear of pre-stability items** — including
   NN#20 (the fault model formalized) and the P18 normative spec.
4. **Bet 5's pre-registered verdict is recorded** (done).

1.0 is the promise that, from that point, **breaking changes reach code only through
editions with automatic migrators** — never silently, never by drift.

### Post-1.0 — evolution by edition (P15)

- The language evolves in **infrequent, batched editions**, not continuous churn.
- Every breaking change ships with a **fully automatic migrator** in the compiler; a
  break that cannot be mechanically migrated **does not ship**. Sound conservative
  over-approximation counts as mechanical migration (this is what keeps the
  effect-set amendment path, NN#19, open).
- **Old editions keep compiling** — the compiler retains every shipped edition's
  front-end.
- A package declares its **edition** in its manifest (0017); the edition is
  **orthogonal to the package's semver** (a `3.0.0` package may be edition-2027).
- **Cross-edition invariant (0017 F4 / NN#14 corollary):** no future edition may
  change the *semantics* of the interface artifact (signature meaning + checked IR).
  Editions change surface syntax and check rules, not the meaning packages link
  against — otherwise rev-pinned git dependencies would become un-migratable in place.

---

## 2. Packages (semver — design 0017 §3)

### The scheme

- **`major.minor.patch`**, required in `candor.toml`.
- **major** — a breaking change to the public interface.
- **minor** — backward-compatible additions.
- **patch** — no interface change (bug fixes, internal-only edits).

### The signature-hash oracle — and its limits (stated honestly)

The compiler computes an aggregate **signature hash** (design 0008 §2) over a
package's *typed public interface*. It is the **machine-checkable oracle for one
direction only**: a signature-hash change means the typed public interface changed →
that is a **major** bump.

It is **not** a complete breaking-change detector. A breaking change can occur with
**no** signature-hash delta, and these are the **author's** responsibility to major-
bump:

1. **behavioral** changes that keep the types (same signature, different result);
2. **`inline`/generic body** changes that consumers compile into their own code;
3. **trust-surface** weakening inside a boundary module (a wrapper that starts
   trusting more).

Semver is the honest carrier of the P2 contract; the hash catches the typed subset
and no more, and this document says so rather than overclaiming.

### The one-way lint

The toolchain warns in exactly one direction — the **unsound** one:

- **Warns** when a signature-hash change carries **no** version bump (an *under*-bump:
  you changed the interface and didn't say so).
- **Does not warn** on a major bump with no hash delta — that is the *correct* major
  bump for a behavioral/inline/trust break the hash cannot see. The lint is a
  provenance aid (P4/P16); it must never pressure an author to *under*-bump.

### Locking, resolution, and reproducibility

- **`candor.lock`** pins the exact resolved set (versions, content hashes, sources).
  Same source + same lock + same toolchain ⇒ **bit-identical artifacts** (NN#16). The
  lock is written on a manifest change or an explicit update, never re-resolved on an
  ordinary build.
- **Single-version unification:** one version per package name per build; a divergent
  diamond (two incompatible versions of one package) is a hard conflict. SemVer-range
  solving, a registry, and `[patch]`/override machinery are **deferred** to a later
  release — `0.x` uses exact path/git pins.
- **Trust-delta gate:** a lock update that grows a dependency's `unsafe`/foreign
  surface is a hard error (E0936) until reviewed and accepted, so a version bump
  cannot silently enlarge the trusted base.

### The `0.x` caveat for packages

While the language is `0.x`, package semver is a **discipline the tooling computes and
lints, not yet a guarantee**: downstream stability arrives with the language's 1.0.
Treat `0.x`-era package versions as provisional, exactly like the language itself.

---

## 3. Quick reference

| Question | Answer |
|---|---|
| What does `0.x` promise? | Nothing. Preview. Pin the toolchain commit if you need stability. |
| When is 1.0? | When the gate clears (editions+migrator in CI, P20 targets, ledger clear) — not a date. |
| How does the language break code after 1.0? | Only via an edition, with an automatic migrator. Old editions keep compiling. |
| When do I bump a package's *major*? | Any breaking change — the compiler *catches* typed-interface changes; behavioral / `inline` / trust breaks are on you. |
| Will the tool stop me under-bumping? | It *warns* on a hash change with no bump. It never nags a major bump with no hash change. |
| Are builds reproducible? | Yes — same source + lock + toolchain ⇒ bit-identical (NN#16). |
