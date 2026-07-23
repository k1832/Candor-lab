# Prepared answers — the questions the thread will ask

*For the Show HN / Reddit / lobste.rs threads. Answer in your own words; these
are the facts and the framing. Rule of thumb throughout: concede the true part
of every hostile question first, then give the specific, checkable answer.
Never claim past what the repo shows.*

---

### 1. "An LLM-written compiler? So it's slop / how can anyone trust it?"

Concede the premise behind the worry: you should not trust it because the
author was careful. The trust model is redundancy and record, not diligence:
every corpus program must run byte-identical on five independent engines (plus
an external WASM runtime), the compiler self-checks against a Rust reference,
and the bugs that apparatus caught in our own compiler are documented — an
associated-type miscompile (garbage / panic / segfault, different per engine),
a dispatch bug where every engine unanimously ran the *wrong* interface's
method, a formatter that rewrote a reborrow into a move. Each is now a spec
clause plus a standing CI gate. Invite the attack: the differential suite is
public; break it.

### 2. "Why not just use Rust?"

Not a Rust replacement pitch; a different default. Rust is borrow-first: the
sophisticated machinery is ambient, and you buy out with `Rc`/`RefCell`/`unsafe`.
Candor is value-first: moves and explicit copies are the whole story for most
code (no lifetime annotations exist at all), borrowing is the second gear, and
the escape hatches are loud. The measured claim: 53% less annotation per
statement than idiomatic Rust across five hand-ported systems programs, at
identical memory safety for safe code. The measured concession: allocator-class
code (in-band metadata over raw memory) is mostly escape hatch in Candor, and
we document that as a limit. If your life is writing allocators, Rust or C is
the better tool; if it's parsers, state machines, codecs, and services above a
thin unsafe core, that's the bet.

### 3. "Zig?"

Closest philosophical neighbor (explicitness, allocators as values, small
language). The decisive difference: Candor is memory-safe by default — no UB in
safe code is non-negotiable #1 and checked by construction — where Zig chooses
manual memory management with excellent tooling. We also ship contracts,
checked arithmetic by default with an unusual fault model, and the audit story
(one command lists every foreign call, unsafe region, and trust assertion in
your dependency graph).

### 4. "53% less annotation — measured how? Sounds like benchmark marketing."

The full protocol is public and was frozen *before* measurement: five programs
chosen adversarially (including the ones value semantics fits worst), ported by
hand to both languages, counted by committed counting tools against a frozen
unit table, with pre-registered kill thresholds. The first verdict was KILL —
we published it, fixed the metric's genuine defect in an openly data-aware
successor (two hostile reviews before ratification), and re-scored the same
frozen artifacts. If we were doing benchmark marketing, the KILL would not be
in the repo.

### 5. "What does 'the LLM wrote it' actually mean? Who decides?"

Roles, precisely: the model wrote design documents, implementation, tests, and
adversarial reviews (fresh-context sessions with a mandate to break the
design). The human is the deciding authority: rules on every design fork,
ratifies or rejects every proposal, owns every publish. Governance is a
committed document; every disposition is recorded with its rationale. The
philosophy calls this the actual thesis: the language is *for* human–LLM pair
authorship, and it was built by one.

### 6. "Checked arithmetic everywhere — isn't that slow?"

Default-checked with an imprecise-but-inescapable fault model: the compiler
keeps scheduling freedom (reorder, hoist, fuse) because a fault must be
*delivered* but not *precisely located* beyond a bounded window — no value
derived from the faulting op ever becomes observable. Hot numeric kernels use
scoped, source-visible `wrapping`/`saturating` regimes; unchecked-UB arithmetic
exists only inside `unsafe`. Honest accounting: SIMD lanes still pay detection
cost; the fault window collapses where MMIO/observable effects are dense. The
fault model is formalized in the spec (single-threaded core), not folklore.

### 7. "No async? In 2026?"

Deliberate refusal, documented with its cost. Async coloring is a bidirectional
ecosystem partition carried in function types; we watched that movie. Candor
ships structured concurrency (`scope`/`spawn`, compile-time race freedom) and
concedes that ergonomic massive-concurrency I/O is currently "threads plus
explicit state machines" — the philosophy lists this under "what this language
is knowingly worse at." If a partition-free model is ever proven compatible
with the no-runtime rule, it arrives by amendment.

### 8. "x86-64 Linux only? Small std? Why should I care yet?"

You mostly shouldn't, for production — it's an explicitly unstable 0.x preview
and says so in bold. What's interesting now is the design record and the
verification approach; what's usable now is real but small (files, TCP,
strings, iterators, sorting; an HTTP server and a WASM interpreter ship as
examples). 1.0 is a defined gate (editions exercised, compile-time targets in
CI, obligations ledger clear), and its checklist is public.

### 9. "Self-hosted? I heard the compiler is Rust."

Both, stated precisely. The production compiler is Rust and stays the
reference. Separately, ~19,300 lines of Candor implement a checker,
interpreter, MIR lowering, and native codegen that process the systems corpus
byte-exact against that reference — self-hosting as a credibility proof
(including catching real bugs), not a bootstrapped toolchain. We are careful
with this claim; the tour and README state exactly which tier is which.

### 10. "Compile times: what's the actual claim?"

Pre-registered and CI-gated, measured on a committed 50kL / 202-module
reference project: clean type-check ≤3s (currently ~1.4s), incremental rebuild
≤1s (currently ~36ms), a body edit re-analyzes zero downstream modules
(architectural: signatures are the interface), and optimized native compile
≤2× `cc -O2` on comparable code (currently ~1.35×). If a commit breaches any
threshold, CI fails. The mechanisms are the boring ones done thoroughly:
definition-site-checked generics (no re-checking per instantiation) and strict
module DAGs with no cross-signature inference.

### 11. "What breaks if the AI angle turns out overhyped (models plateau)?"

The philosophy answers this one directly (Bet 1): explicitness and local
verifiability serve human readers too, so the *language* survives that failure;
what's predicated on model authors arriving is the tooling investment (spec
pack, corpus pipeline, eval harness), which is staged against evidence, not
faith. Also honestly recorded: our attempt to measure "model competence can be
manufactured" is currently *blocked* — the lab-runnable proxy would measure
prompting rather than training, so we shelved the measurement rather than
laundering it. That refusal is in the repo too.

### 12. "License? Contributions?"

MIT OR Apache-2.0 (the Rust convention; Apache for the patent grant). The
preview repo is generated from the lab by a seeding script; substantive
contributions flow through the lab's design pipeline — an adversarial review
and a recorded disposition — because that pipeline is where the project's
trustworthiness comes from. Bug reports and breakage of the verification
apparatus are the most valuable contribution right now.
