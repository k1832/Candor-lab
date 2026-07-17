# Philosophy of the Language — v4

*The founding document for a systems programming language designed in the era of human–LLM pair authorship.*

**Status:** Normative, amendable (§9). This document precedes and outranks all design documents.
**What this document is:** principles, priorities, bets, and refusals — the criteria by which all future design decisions are judged.
**What this document is not:** a specification or a design. Where it names a mechanism (a fault window, an effect set), it does so only to fix a *position of principle* that designers must not quietly reopen. Everything else — syntax, algorithms, library shape — belongs to the design documents beneath it.
**Version:** 4.2 (v4 plus the enacted Bet 5 verdict and its §9 disposition — Appendix A). v2 and v3 were driven by independent adversarial reviews; v4 by a *history-blind* review of v3 (the reviewer saw no version history or ledger). The blind test caught one v3 fix that had not actually landed — recorded in the ledger, because a repair that fails its retest teaches more than one that passes. Appendix A records every change and why; the ledger is part of the document.
**A warning the blind review earned:** this document's fluency at pre-admitting costs can create the impression that admitted costs are managed costs. They are not the same. A named cost still has to be borne; the Refusals section is a list of debts, not of absolutions.
**Audience:** The designers and implementers of the language — human, LLM, or both.
**How to use it:** Resolve open questions by the Priority Order (§2) and the spirit of the Principles (§4). A proposal violating a Non-Negotiable (§5) proceeds only by amending this document first (§9). A proposal to add something in Refusals (§6) carries the same heavy burden.

---

## 1. Identity

This is a systems programming language for the systems core: kernels, drivers, firmware, allocators, runtimes, codecs, databases, embedded targets. It competes with C, C++, Rust, Zig, and Carbon on their home ground.

Its thesis, in one sentence:

> **A systems language designed for the human–LLM pair as the unit of authorship: memory-safe, explicit where meaning lives, locally verifiable, with source-declared semantics and a compiler built as a conversation partner rather than a gatekeeper.**

Positioning against the incumbents:

- Against **C and C++**: safety, coherent tooling, and sanity.
- Against **Rust**: a bet on lower cognitive load (Bet 5 — a stated research bet, not a promise), compile speed earned by construction (P20 — a mechanism, not a slogan), and an ecosystem never split by function coloring.
- Against **Zig**: compile-time safety guarantees rather than runtime checks and discipline.
- Against **Carbon**: freedom from the C++ object model.
- Against **all of them**: *design-fit for the automated authorship loop* — generation, verification, diagnosis, repair, and migration performed by models under human supervision. Individual pieces exist elsewhere; the incumbents cannot retrofit the whole: per-target one-semantics across build modes, an uncolored ecosystem, a grammar that parses without a symbol table, one canonical form from day one. We claim coherence, not novelty of parts.

The economics behind the premise: code **generation** has become cheap; **verification** — by a human reviewer, a model, or a prover — is now the bottleneck. Every design decision optimizes the cost of verifying code, even when that raises the cost of writing it. Verification cost is measured in what a reviewer must read and hold in mind — token count included (P13). Scope honesty, up front: this language localizes the *localizable* fraction of verification — aliasing, ownership, contracts, effects. Whole-system properties (lock ordering, liveness, protocol correctness) remain design-level work that no language-level principle here addresses; §6 names this as a standing limit, not a solved problem.

One consequence the thesis imposes: if models are the primary authors, then **model competence in this language is a deliverable of the project, not an emergent hope** (P19). A new language starts with zero corpus, and its most distinctive features are exactly what models will have no priors for. The bootstrap is a first-class engineering problem.

---

## 2. Priority Order

When principles conflict — and they will — resolve in this order. A lower item may never be bought by sacrificing a higher one.

1. **Soundness.** Safe code has no undefined behavior. No exceptions, ever.
2. **Source-declared semantics.** For a given target, a program's observable behavior is determined by its source text alone; build modes and optimization levels change speed, never behavior. (Scope for faulting executions: P5 — the scope is itself part of the declared semantics.) Anything semantic that must vary — check levels, arithmetic regimes, fault policy — is declared in the source, visibly and greppably, never in a build flag.
3. **Local verifiability.** The correctness of a line of code can be judged from information near that line. Signatures are contracts: everything a caller must know is in the signature.
4. **Predictable cost.** No hidden allocation, no hidden control flow, no invisible performance cliffs — including cliffs introduced by the compiler's own strategy choices (P11). Peak machine performance must always be *reachable*, explicitly.
5. **Simplicity of the whole.** The total language must remain learnable and mechanically analyzable. Every feature is judged by what it costs the whole.
6. **Compile speed.** Fast feedback is a design constraint on the language itself (P20).
7. **Ergonomics and brevity.**

**Rule of reclassification.** Arguments will be made that an item-7 concern is "really" an item-3 concern (P13 legitimately makes one such move). Reclassification is legitimate only when the cost demonstrably falls on the *reader or verifier* of code, not its author — and any design decision that turns on a reclassification must record it, with the argument, in its rationale. Promotion by relabeling is how priority orders die; this rule is the audit trail.

Not on this list: peak benchmark performance by default. The language must let an expert reach the machine's full speed through explicit, auditable means; it must not sacrifice items 1–5 to win unannotated benchmarks against UB-exploiting C++.

---

## 3. The Bets (Falsifiable Assumptions)

Ordered by how load-bearing they are. Each states what kills it, and — a standard v4 applies symmetrically after the blind review caught it applied selectively — each existential bet carries a pre-registered kill criterion, not just the technically risky one. Under §9, material evidence against a bet is a mandatory review trigger for the principles that depend on it.

**Bet 5 — Value-first defaults lower cognitive load for real systems code. (The design research bet.)** Stated honestly and narrowly: Rust already has body-local inference and compact signature defaults, so the genuine delta of P12 is *value semantics as the default gear, borrowing as the second* — and the bet is that this ordering fits enough real systems workloads (parsers, codecs, state machines, protocol logic) that the pressure valves for pointer-rich structures (allocators, intrusive lists, schedulers) remain *rare in occurrence even where they are critical in function*. The uncomfortable part, named: pointer-rich code is concentrated in precisely the kernels and allocators of our identity statement — the bet's hardest instance is our home ground.
*Falsification:* before the syntax freeze, designers must pre-register a kill criterion — candidate metrics: annotation density per KLOC versus idiomatic Rust on ported systems workloads; frequency with which pressure valves become ambient rather than exceptional; first-attempt model generation/repair correctness under equal adaptation budgets (P19); human task-completion studies. Exact thresholds belong to the validation plan; *the existence of a pre-registered, published criterion is non-optional*.
*Validation prototype, to break the circularity* (a model needs programs; programs need a language): the memory-model core with deliberately throwaway syntax — a checker plus interpreter, no optimizer, no stdlib beyond slices — exercised by porting a fixed basket of representative systems programs: an allocator, an intrusive-list scheduler, a driver-like state machine over MMIO, a parser, an arena-based compiler pass. The basket is chosen *adversarially* to include the workloads value semantics fits worst. This human-ported basket does double duty as Bet 6's external ground truth.
*Consequence (P15):* no stability commitment may precede this bet's verdict, because a post-1.0 memory-model rework is exactly the break that cannot be mechanically migrated.
*Disposition (amendment, 2026-07-08 — Appendix A v4.1→v4.2):* the successor registration
(`docs/BET5_CRITERION2.md`, ratified and frozen after two hostile reviews) re-scored the frozen
artifacts to a mandatory §9 review, and the review ruled: **Bet 5 is provisionally confirmed on
this basket.** The load claim passed decisively on every metric and program (53% aggregate
annotation reduction); the valve-content gates passed everywhere except the conceded allocator
class (WARN) and the scheduler's baseline-sensitive result, where the authority ruled the
registered self-contained yardstick correct *for a language-model bet* — the authored-only
alternative (2 vs 47 unsafe statements) measures ecosystem maturity, which Bets 4 and 6 own, and
is recorded as a true and important bootstrap fact, not a model defect. **Binding commitments
attached to the confirmation:** the next design round takes up safe typed field projection on
`rawptr` and call-site reborrow ergonomics, and the scheduler is re-ported and re-measured under
the frozen successor rules before any syntax freeze; if the re-measurement worsens, the
confirmation returns to review. NN#14's stability gate is now conditioned on those commitments,
not merely on time passing.
*Verdict of the first registration (amendment, 2026-07-07 — Appendix A v4→v4.1):* the pre-registered criterion computed **KILL** and the verdict is enacted as registered (`docs/RESULTS.md`; the registration is not retrofitted). The evidence cut both ways: the **load claim was decisively confirmed** — 53% aggregate annotation reduction versus idiomatic Rust across the full basket, with zero copy inflation and all five programs completable — while three valve-fraction ceilings were breached, of which the published defect analysis (§0.3 of the criterion) attributes two substantially to a fraction-versus-density artifact (Candor implements the same specifications in 2–5× fewer statements, so identical absolute valve content yields proportionally higher fractions), and one — the allocator, 63% valve lines — stands on any reading (see P12's concession). Bet 5's ordering claim therefore remains open, not vindicated: a **successor criterion must be pre-registered** — openly data-aware, changing only the defective operationalization, with the conflict named — and pass on the frozen artifacts before any stability reliance on P12. NN#14's gate remains closed until then.

**Bet 6 — Model competence can be manufactured. (The bootstrap research bet — held to the same standard as Bet 5.)** Incumbents' corpus advantage is an *active barrier* to a language whose primary authorship mode is model-driven. We bet competence can be deliberately built: a machine-consumable specification pack, synthetic corpus generation grounded in that spec and filtered by the toolchain, and evaluation harnesses (P19).
*Falsification, pre-registered like Bet 5's:* after a fixed, published adaptation budget (fine-tuning tokens / examples), models must reach a threshold of first-attempt correctness on **externally anchored** tasks — the human-ported Bet 5 basket and independently authored problems, never self-generated tests grading self-generated code — with a competence *slope* that is positive across releases. A flat slope, or corpus-quality collapse toward the generator's own distribution, kills the bet. The circularity risk is named because it is real: compile + test + contract-check filters for internal consistency, not correctness or idiom; the external anchors are what make the measurement mean something.
*If this bet fails:* the thesis language has no authors at the scale the thesis assumes, and the project must either fund the corpus the slow human way or concede the positioning.

**Bet 1 — Authorship has inverted.** Most code in this language's lifetime will be generated by models and verified by human–model pairs. Honest accounting of what this bet actually carries: it does little *language-design* work — explicitness and local verifiability serve human authors too, so the design survives its failure — but it does all the *project-economics* work: P19's specification pack, corpus pipeline, and evaluation harness are large standing investments predicated on model authors arriving. *If this bet fails*, the language degrades gracefully; the project's budget does not — P19's scale should be staged against evidence of Bet 1, not committed in full on faith (§8, Sequencing).

**Bet 2 — Locality stays valuable as models improve.** The naive version of this bet ("models fail at non-local reasoning") is a snapshot of a transient weakness, and this document does not rest on it. The durable version: locality is what makes verification *cheap and parallelizable at any capability level* — fewer tokens consulted per judgment, incremental re-verification of only what changed, audit by sampling rather than by whole-program reading. Long contexts make non-local reasoning *possible*, not *free*; a reviewer (human or model) who must consult three files to judge one line pays that cost on every judgment, forever. *If even this fails* — if whole-program verification becomes as cheap as local — the explicitness tax loses its return and P2's stringency should be revisited by amendment.

**Bet 3 — Checked arithmetic is affordable *given imprecise fault semantics and scoped escapes* — with hardware honesty.** Imprecise faults (P5) restore the compiler's *scheduling* freedom: reordering, hoisting, fusion. They do **not** erase *detection* cost: mainstream SIMD units expose no per-lane overflow flags, so checked vectorized loops pay widening/compare work regardless of trap semantics. The honest claim: imprecision makes default-checked code cheap in scalar and branchy code; in genuinely hot numeric kernels the scoped regimes of P5 will be needed — routinely, not exceptionally — and they are first-class for that reason. A second honesty, in our own domain: the imprecision window collapses wherever externally visible effects are dense (MMIO, DMA, shared memory), which is much of kernel code; there the model degenerates toward precise faulting and the freedom shrinks. Those same paths are where P8's proven contracts and P5's scoped regimes carry the load. *If this bet fails* — if default-checked code is unacceptably slow across the board — NN#4 forces the cost onto scoped regions rather than silent wraparound, and the language becomes more ceremonial than intended, not less safe.

**Bet 4 — Corpus coherence is a tailwind, not a moat.** A uniform canonical-form corpus should make models better at this language than its market share predicts; but corpus *scale* dominates purity in current evidence, and transfer learning helps incumbents too. We claim coherence compounds and never hurts; we do not claim it is a barrier.

---

## 4. Principles, with Rationale

### P1. Safe by default; unsafety is explicit, local, and auditable

All code is memory-safe and data-race-free unless enclosed in an explicitly marked unsafe region. Unsafe regions are greppable, must carry a stated justification, and expose a narrow, checkable boundary to safe code. (The foreign boundary, the largest unsafe surface in practice, is P17.)

*Rationale.* The industry verdict is in: the large majority of severe vulnerabilities in unsafe-language codebases are memory-safety failures, and public policy now formally discourages new unsafe-by-default systems code. But systems programming requires touching hardware, so the escape hatch must exist — as a visible, reviewable exception, never ambient permission.

### P2. Local verifiability beats expressiveness

Everything needed to judge a line correct should be visible near that line. Concretely: no implicit conversions; no overload-resolution mysteries; no argument-dependent lookup; no type or lifetime inference *across* function boundaries (inference inside a body is fine and encouraged — locality is defined at the signature line); a grammar that parses without a symbol table. Signatures are complete contracts: types, ownership and aliasing expectations, failure modes, declared contracts, and tracked effects.

**The tracked-effect set is closed and deliberately tiny: allocation, and foreign trust (P17). Nothing else in the first stable version.** Growing this set is an amendment (NN#19) — a genuinely open path, because a new tracked effect *can* ship with a mechanical migration: conservative defaults (every existing function and boundary module is marked as having the effect unless declared otherwise), which is sound over-approximation, fully automatic, refined incrementally afterward. Blocking is excluded from the initial set as a *revisable judgment call* — the churn budget is spent on allocation first because allocation is the effect the stdlib layering (P9) and freestanding story load-bearingly depend on — not as a matter of principle; field evidence from kernel and embedded users is the named revisit trigger. Fallibility needs no effect: it is in the return type (P7). Fault-potential is not tracked because it would be vacuous.

**Effects are upper bounds — "signatures never lie" means signatures never *understate*.** A function marked as allocating that currently doesn't is permitted conservatism; removing the marker is a non-breaking strengthening; adding one is a breaking change. This asymmetry is stated so it cannot be resolved by drift.

**Effect checking is a partition, and this document says so.** An earlier version of this document claimed effect tracking was "not coloring" by definition. A history-blind review correctly rejected that: a no-allocation context *cannot call* an allocating function — that is a checker-enforced call-graph partition, transitive like async coloring. The defense is withdrawn; the honest argument is about *shape and degree*, and here it is. Async coloring is ruinous for three reasons this partition lacks: it is **bidirectional** (sync cannot call async without an executor; async calling blocking sync stalls the runtime — both directions break), it **transforms function types and calling conventions**, so APIs and trait hierarchies genuinely ship twice, and the constraint it enforces is one the runtime *invented*. The allocation partition is **one-way with a universal ground floor** — allocation-free code is callable from everywhere, so the ecosystem's shared substrate is the unrestricted subset; the capability travels as an ordinary *value* (the allocator parameter), not as a function-type transformation, so one API serves both worlds by taking an allocator; and the constraint enforced is one physics already imposes — code in an interrupt context must not allocate whether or not a checker says so. The checker makes a real wall visible; async built a wall where none existed. **The residual risk is named, not dissolved:** Rust's no_std/alloc split shows that even a one-way partition exerts ecosystem gravity toward parallel library variants. Mitigations: the subtractively layered stdlib (P9), allocator-as-parameter as the single polymorphism mechanism (no dual trait hierarchies, by construction), and the deciding authority's mandate (§9) to treat emerging parallel-variant ecosystems as a P3 violation. If the gravity wins anyway, that is a P2-vs-P10 conflict to be resolved by amendment, in the open.

**The churn is the truth — with its full cost.** Tracked effects propagate: a leaf function that starts allocating ripples the marker through every public signature above it, and under P15 that is a breaking API change. The toolchain makes the *diff* one command; it does not make the *release cascade* cheap — every transitive dependent owes a major version, which is human coordination across maintainers that no tool absorbs (§6 names this debt). It is still deliberate: a dependency that silently starts heap-allocating has genuinely broken its contract with a kernel consumer; the churn is the change told loudly instead of discovered in an interrupt handler.

*Rationale.* Both human and model reviewers pay for non-locality on every judgment (Bet 2's durable form). Implicitness optimizes writing; explicitness optimizes verification; verification is the bottleneck. This is the single most important consequence of the thesis, and it is why P12's inference stops at the signature: information a caller needs must be written down even when a clever compiler could infer it — the reviewer is not the compiler.

### P3. One canonical way

For each concept, one construct. One loop form, one error-handling style, one dominant idiom per task. A zero-configuration canonical formatter ships with the compiler, and formatted form is the only form. Where two ways emerge, the deciding authority (§9) deprecates one.

A known stress point, named now so it is designed rather than accreted: **text.** "One string type" collides with the allocator-free core / freestanding reality (P9) — the exact pressure that produced Rust's `str`/`String`/`OsStr`/`CStr` sprawl. The design documents must resolve text under P3's authority with a defended budget (one universal view type in the core; the minimum owning/interop forms above it, each existing only by recorded justification), not by accumulation.

*Rationale.* Every stylistic alternative splits the training distribution, weakens model priors, complicates diffs, and doubles what a reviewer must recognize. Uniformity is what makes the corpus compound (Bet 4) and keeps automated repair loops reliable.

### P4. The compiler is a conversation partner

Diagnostics are a first-class output format: every error and warning is emitted as human prose *and* as structured, machine-readable data carrying (a) what failed, (b) the full chain of reasoning — for ownership and type errors, the provenance of every inference involved, (c) the definitions implicated, and (d) where possible, a mechanically applicable fix. The compiler ships the test runner and can emit structured execution traces.

*Rationale.* The dominant debugging loop is becoming: run → read diagnostic → patch → repeat, executed by a model under human supervision. Diagnostic quality and machine-legibility set the speed of that loop. We design diagnostics *into* the semantics (P5's fault guarantees) rather than around them, and treat the loop as half the product.

### P5. Source-declared semantics; faults are inescapable, not silent

For a given compilation target, observable behavior is a function of source text alone — no "checks in debug, corruption in release" tier, no compiler flag that changes meaning. The precise invariant, stated with its scope (the scope is part of the declared semantics, not a footnote):

- **Fault-free executions are deterministic across build modes.** Same source + same target + same inputs ⇒ identical observable behavior in every build mode, for any execution that completes without a fault.
- **Faulting executions are truncated executions.** Integer overflow (and kin) faults by default. A fault is *imprecise but inescapable*: no value derived from the faulting operation ever becomes observable; the fault is guaranteed to be delivered; observable behavior is identical across build modes up to a **fault window** around the faulting operation, within which effects independent of it may or may not have retired — and that window may differ between build modes.
- **The window's bound is defined, and the semantics is honestly novel.** The bound: a fault is delivered no later than the next synchronization operation or externally visible effect that follows the faulting operation in program order; nothing past that point executes. Prior art exists in pieces — formalized imprecise exceptions, hardware imprecise traps — but their composition with an adopted C/C++-family consistency model under an optimizing compiler is **novel semantics, and this document says so** rather than sheltering it under P18's "adopt proven art." Consequence: formalizing the fault model is mandatory pre-stability work (NN#20), part of the same validation tier as Bet 5's artifact. An earlier draft hand-waved "bounded" while demanding rigor of everything else; the blind review caught it, and the concession is recorded.
- **Where observable effects are dense, the window collapses.** MMIO, DMA, and shared-memory writes bound the window tightly — in effect-dense kernel code the model degenerates toward precise faulting and the optimizer's freedom shrinks. Accepted and named (Bet 3): those paths are exactly where scoped regimes and proven contracts (P8) carry the performance load instead.
- **Alternative arithmetic regimes are scoped and source-declared — and safe ones are defined ones.** Wrapping and saturating arithmetic exist as explicit block-level regions (and named single operations), visible and greppable — for hash loops, DSP kernels, and domains like avionics control laws where saturate-and-continue is a requirement. **Unchecked arithmetic — overflow as undefined — exists only inside unsafe regions**, because under NN#1 it cannot exist in safe code, full stop; an earlier draft listed it among the safe regimes, which was a side door left open in a document that hunts side doors, and it is closed. A scoped region is one reviewable decision, not a carpet of sigils; this is how the corpus avoids splitting into "checked" and "fast" dialects.
- **Uninitialized memory can never be read. Evaluation order is defined** (in faulting executions, the fault window is the sole, bounded license against it). **Nondeterminism exists only where explicitly declared** — allocation addresses, hash iteration order, and their kin are declared nondeterminism.
- **Platform-dependent facts are per-target defined, not undefined.** Pointer width, endianness, and target arithmetic details are queryable constants; code that depends on them says so. "One semantics" never promised bit-identical behavior across *different* targets, and will not pretend to.

*Rationale.* A regenerate-and-test loop is only sound if the tested artifact behaves like the shipped one; semantics that shift with compiler flags poison automated debugging and human trust alike. Stating the invariant more strongly (platform-invariance, precise traps) would be dishonest; stating it without the faulting-execution scope would hide a hole. The whole truth: *meaning lives in the source, failures are loud, and the only indeterminacy is a bounded, defined window around a failure that was never going to be silent.*

### P6. Small core; compile-time execution is transparent and interface-bounded

Deliberately small feature set: no function overloading, no implementation inheritance, no exceptions. Compile-time execution exists for constant evaluation and code generation, and is *transparent*: the compiler can always render the expansion as ordinary source. Compile-time codegen is not the generics mechanism of public APIs — that is P11's definition-site-checked generics, because unrestricted comptime is instantiation-checked, the C++-template failure mode. Comptime-generated interfaces may not appear in public library signatures. The budget is fixed: adding requires removing.

*Rationale.* Bounded generics carry public polymorphism (verifiable in isolation); comptime carries constant evaluation and *private* code generation (powerful, dumpable, reach ends at published signatures). One mechanism per role, checked where verification is cheapest.

### P7. Errors are values; faults are bugs — one vocabulary, one policy

Fallible functions declare failure modes in their signatures as ordinary values (sum types), with a lightweight propagation operator — that is the path for *expected* failure. **Faults** are the other category, unified across this document: an arithmetic fault (P5), a violated `enforced` contract (P8), a failed assertion, an explicit panic — all are *bugs manifesting*, and all route through one **root-declared fault policy**: every program or embedded image declares at its root what a fault does — abort, halt-and-log, or a user-supplied handler. Unwinding is not required machinery; freestanding targets choose their handler explicitly. Where even a fault is unacceptable (a control loop that must not stop), the answer is deciding *before* deployment that the fault cannot occur — P5's scoped regimes and P8's proven contracts — never silencing it after.

*Rationale.* Exceptions are hidden control flow, disqualifying under P2 and unusable in kernels. Errors-as-values is one of the few settled questions in systems design; we follow the consensus. The unification clause exists because "fault" and "panic" left as separate, unmapped words invite quiet divergence — and an interrupt handler halting the machine must be a choice the source visibly made.

### P8. Intent is checkable: contracts in the language — and contracts never license optimization

Function signatures may carry preconditions, postconditions, and type invariants as executable, analyzable clauses. Enforcement is static where provable, dynamic otherwise, at a check level **declared in the source** (per module or per contract), never by build flag. The levels, with their semantics fully defined:

- **`enforced`** — checked dynamically where not proven; violation is a fault (P7).
- **`audit`** — checked dynamically; violation is *recorded* (structured, P4-visible) and execution continues. Semantically safe precisely because of the rule below: no analysis anywhere trusted the contract, so continuing past a violated one yields a logically wrong program, never an undefined one.
- **`assumed-proven`** — the dynamic check is skipped; the annotation is an auditable assertion that verification happened externally, greppable exactly like unsafe regions.

**The rule that makes all three sound: the optimizer may never assume a contract holds.** Contracts are checks and documentation and oracles — never facts the compiler builds on. A wrong `assumed-proven` contract therefore yields wrong *values*, not undefined behavior; NN#1 stands without an asterisk. The cost is stated so nobody "fixes" it later: `assumed-proven` buys only the removal of check overhead, no optimization headroom. Contract-informed optimization is possible only where the compiler *itself* proves the contract — in which case it is ordinary static analysis, assuming nothing.

*Rationale.* The characteristic failure of generated code is *plausible but wrong*: compiles, looks right, violates an intention nobody wrote down. Contracts are the highest-leverage feature against that gap — documentation a model reads when calling, a target it implements against, an oracle the repair loop tests. Every prior contract language walked back "checks always on"; source-declared levels keep the true invariant — semantics recorded in source, reviewable in diffs. The optimizer question is answered in the conservative direction because the alternative puts UB back into safe code through a side door. Scope honesty (§1): contracts localize *interface-level* intent; they do not verify lock ordering, liveness, or protocol correctness, and this document claims no otherwise.

### P9. Predictable cost: no runtime, explicit allocators

No garbage collector, no mandatory runtime threads, freestanding-capable from day one. Heap allocation happens only through explicitly passed allocators; a library that allocates says so in its signatures (P2's tracked effect). No hidden control flow, no invisible copies.

**Standard library scope:** a small always-available core (types, slices, contracts, formatting) that never allocates; a standard collection/OS layer that is allocator-explicit throughout and absent on freestanding targets by default. One standard library, subtractively layered — every library written against the core runs everywhere. The layering is also the first line of defense against the parallel-variant gravity P2 names.

*Rationale.* This is what "systems core" means: interrupt handlers and 64KB microcontrollers, where a runtime is disqualifying and every allocation is a decision. Explicit allocators also yield a healthier ecosystem — everything works everywhere.

### P10. Concurrency without coloring

Data-race freedom is guaranteed by the ownership model — compile-time, non-negotiable. Concurrency primitives are structured (scoped, joined, cancellable). The language does **not** ship `async`/`await` or any mechanism with async coloring's shape — a bidirectional partition carried in transformed function types, forcing duplicated APIs — in its first stable version. (P2 owns the admission that effect checking is itself a one-way partition, and the argument for why its shape is acceptable; the two principles now reference one honest account instead of two definitions.) If a partition-free ergonomic model for high-concurrency I/O is later proven compatible with P9, it may be added by amendment; a split ecosystem may not, ever.

*Honest accounting:* this cedes ergonomic massive-concurrency I/O in the first stable version — including part of our own ground, since a modern storage engine is an io_uring-saturating program. The path there is threads plus explicit completion-driven state machines: the architecture the highest-performance engines use today — capable but manual. We accept "capable but manual" over "ergonomic but schismed." Whether a better answer exists (without a green-thread runtime, which P9 forbids, and without effect systems, which reintroduce the bidirectional partition under a new name) is open research this document flags rather than resolves.

*Rationale.* Rust's async demonstrated the alternative's cost: two dialects, incompatible libraries, the largest single source of ecosystem complexity. Better to lack a feature than to ship a schism.

### P11. Generics are checked where they are defined — and instantiated predictably

Public generic code is type-checked completely at its definition site against declared interface bounds; a generic that compiles cannot produce type errors at instantiation. Instantiation strategy (monomorphization vs. shared code) is **not** an invisible implementation whim: the default strategy is deterministic, documented per target, and stable within an edition, and source-level declarations override it where cost control demands. Priority 4 applies to the compiler's own choices — a strategy the programmer cannot predict from source is a hidden performance cliff, exactly what this language forbids.

*Rationale — with the cost named.* Definition-site checking requires an interface/bound system: declared capabilities, associated items, coherence rules — real, irreducible complexity, a meaningful fraction of what makes Rust's traits heavy. We pay it deliberately, because instantiation-time checking is the canonical violation of local verifiability. It is also a load-bearing input to P20: code checked once at definition is not re-checked per instantiation. The simplicity budget (P6) is spent here in preference to almost anywhere else; designers keep the bound system as small as coherence allows and reject expressive growth by default.

### P12. Memory model: values first, borrows second, signatures always explicit

The default style is value-oriented: ownership transfers and copies with explicit cost, no lifetime annotation. Borrowing is the second gear. Inference division is fixed by P2: **within a body, infer as aggressively as soundness allows; at signatures, nothing is inferred** — aliasing, ownership, and lifetime relationships a caller must know are written, with the common patterns as compact defaults so the *rare* signature carries annotation weight. Where neither gear fits — intrusive structures, self-reference, true shared mutability — the pressure valves are explicit: checked runtime alternatives or unsafe regions, chosen by the author, visible to the reviewer.

*Rationale.* This is Bet 5, the project's primary risk, stated in its honest, narrowed form: the delta over Rust is not explicit signatures or body inference (Rust has both) but the *value-first ordering* — and the hardest instance of the bet is our own domain, where pointer-rich structures concentrate. The bet holds if the valves stay rare in occurrence even where critical in function; if they become ambient, the bet has failed and the philosophy must be amended (§9), not worked around. The kill criterion is pre-registered before the syntax freeze (Bet 5); the validation basket is chosen adversarially. Validate this before everything: until the memory model exists and real systems programs have been written in it, all else is decoration on an unproven core.

*Concession from the first validation (amendment, 2026-07-07):* **value-first does not carry allocator-class code** — programs whose core is in-band metadata over raw memory (free lists threaded through the blocks they describe). There the valve is not rare-but-critical; it is the program's spine (63% of the measured allocator's lines, a figure that survives every identified measurement artifact). This is recorded as a named limit of P12, not worked around: on that class, Candor's honest posture is "the valve is the program, and the valve is visible," and the load reduction still applies to everything around it. Two design consequences are recorded for the next design round rather than decided here: safe typed field *projection* on `rawptr` (reading a field's address without dereferencing) was the ports' largest avoidable valve-surface cost, and the explicit call-site reborrow ceremony was their largest reading-friction cost; both are P13-vs-P2 trades to be weighed in the open.

### P13. Clarity-dense syntax: words where meaning lives, compact everywhere else

Explicit keywords for *semantic distinctions* — ownership transfer, unsafety, arithmetic regime, contract clauses; compactness where verbosity adds tokens without meaning. The measure is **information per token a reviewer must read**: a keyword preventing a misreading is cheap at any length; boilerplate all readers skip is expensive at any brevity. The grammar is unambiguous without semantic context and tokenizes predictably.

*Rationale.* The binding scarcity is reading effort and context-window tokens — our own economics. Sigils encoding load-bearing semantics in one character (`?` propagation) earn their place; keywords making rare dangerous things loud (`unsafe`, `wrapping`) earn theirs; ceremony that is neither is cut. (This principle's pricing under Priority 3 rather than 7 is the canonical legitimate reclassification — recorded per §2's rule.)

### P14. C is a first-class citizen; C++ is a neighbor; the stable ABI is C's

Calling C requires no hand-written bindings: the *toolchain* consumes C headers and generates checked bindings automatically — whether inside the compiler or in a blessed always-installed tool is an implementation decision, informed by the maintenance weight that pushed Zig to move `@cImport` out of the compiler. C++ interop goes through tooling-generated shims at the C-compatible surface, and coverage is honestly partial: template-heavy, overload-heavy, RAII-idiomatic C++ APIs will not map cleanly, and we do not promise they will. The language never adopts C++ semantics natively.

**ABI position:** the language guarantees **no stable native ABI** initially — within-language linkage may change between editions. The stable ABI *is* the C ABI, spoken at boundary modules (P17): that is how kernels load modules, how plugins link, how distributions ship. Committing to a stable native ABI is a permanent tax on layout and optimization freedom, paid only if evidence demands it — by amendment, never by drift.

*Rationale.* Billions of lines of C are not being rewritten; for its first decade this language lives *inside* a C world, and incremental adoption is the only adoption. The C++ refusal is deliberate: Carbon is structurally condemned to inherit C++'s complexity in exchange for native interop. We take the genuinely cheap majority of interop value; for some C++ libraries the unmappable part *is* the API, and the answer there is a shim someone writes once, in C terms, on purpose.

### P15. Evolution by edition, migration by machine — with the residual costs named

The language evolves through infrequent, batched editions. Any breaking change ships with a fully automatic migrator in the compiler; a break that cannot be mechanically migrated does not ship. Conservative over-approximation counts as mechanical migration where it is sound — this is what keeps NN#19's effect-set amendment path genuinely open rather than frozen by rule interaction. Old editions keep compiling. Migrator maintenance is a permanent, funded line item of the toolchain team (§9).

Consequences stated rather than implied:
- **Editions cost corpus, and the migrator does not pay that bill.** Migrators update codebases; they do not update the frozen corpora models were trained on. Mitigation, not denial: editions are rare, canonical form is preserved across editions wherever possible, and P19's pipeline regenerates its synthetic corpus per edition. The residual cost is accepted as the price of not dying the C++ death.
- **The migrator rule makes a post-stability Bet 5 failure fatal**, because a memory-model rework is precisely the break no tool can migrate (over-approximation cannot save a changed ownership discipline). Therefore — as a hard consequence — **no stability commitment (no "1.0") may precede Bet 5's pre-registered verdict.**

*Rationale.* Two facts in tension, both true: every breaking change devalues the trained corpus (Bet 4 — stability compounds), and models can mechanically migrate code at a scale humans never could, so refusing all evolution is choosing slow death over surgery.

### P16. One blessed toolchain — reproducible, provenance-aware, and interactive

Compiler, package manager, build system, formatter, test runner, documentation generator, **and language server** ship as one coherent tool from the first stable release. No configuration where there can be a convention. Builds are **reproducible by default** — same source, same lockfile, same toolchain ⇒ bit-identical artifacts — and the package manager records full dependency provenance, because the most common audit question of this era is not "where is the unsafe code" but "what exactly is in this binary and where did it come from."

The language server is not an afterthought listed for completeness: for a language whose product is the human–LLM authorship loop, **interactive semantic services — hover provenance, signature contracts at the call site, effect and regime visibility, incremental diagnostics — are the loop's inner surface**, as central as batch diagnostics (P4). An earlier draft omitted it; for this thesis, that was not a small omission.

*Rationale.* Cargo is arguably half of Rust's success; the absence of an equivalent is C++'s permanent wound. The automated authorship loop — build, test, trace, diagnose, migrate — only works if every project speaks to the same tools the same way, and it is only *trustworthy* if what was tested is bit-for-bit what ships (the toolchain-level mirror of P5).

### P17. The foreign boundary is the audit surface — trust declared, not proven

Every foreign call is unsafe in principle; a program sitting on a large C substrate cannot be made safe by grepping for `unsafe`. The boundary is therefore the unit of audit: foreign interfaces are wrapped in declared **boundary modules** that (a) localize all FFI unsafety, (b) attach contracts (P8) to foreign signatures, and (c) are enumerable by the toolchain — "show me everything this program trusts" is one command. Safe code may not call foreign functions except through a boundary module.

**Stated honestly:** most properties that matter at a C boundary — "does not retain this pointer," "does not free it," "does not touch it from another thread" — are not dynamically checkable, so most boundary contracts will be `assumed-proven` **trust declarations**, not verified facts. The checkable value-level subset is checked; the rest is trust made *visible, structured, and enumerable* rather than ambient. The auditability claim survives in full; a verification claim was never honestly available, and this document does not make it.

*Rationale.* Seamless import (P14) and auditability must not trade against each other: import stays frictionless, and the trust relationship becomes a reviewable artifact. The security auditor's week (§7) starts from the boundary-module list — knowing exactly *what is trusted* is the realistic and valuable guarantee.

### P18. The semantics is defined by a specification, not by the compiler

The language has a **normative written specification, independent of any implementation**, from the first stable release — covering the safe-language semantics, the fault model (P5), the contract levels (P8), the **memory consistency model** (the ordering semantics of atomics, unsafe code, and boundary-module interactions), and the **unsafe-code aliasing model** (what unsafe code may assume about references and pointers it touches — the question Rust has spent a decade answering after the fact, and which NN#1's soundness quietly rests on; it is named here so it is budgeted, not discovered).

The position on novelty, stated precisely after an earlier draft stated it too broadly: **adopt proven art where proven art suffices** — the consistency model comes from the C/C++/Rust axis, where a decade of committee work exists and novelty is risk without reward. Where this language's own commitments *require* novelty — the imprecise-fault truncation semantics of P5 composed with that adopted model — the novelty is named, owned, and **formalized as mandatory pre-stability work**, in the same validation tier as Bet 5's artifact. Mechanized formalization of the memory-model core is strongly preferred; at minimum, the specification is the arbiter and the compiler is its subject. "The compiler is the spec" is how "no UB in safe code" decays from theorem to folklore, and NN#1 deserves better than folklore.

*Rationale.* The document's two highest priorities — soundness and source-declared semantics — are claims *about* semantics; they are only meaningful if the semantics exists somewhere other than the implementation's changelog. Data-race freedom via ownership does not eliminate the need for a consistency model, and safe-language soundness does not eliminate the need for an aliasing model underneath it.

### P19. Model competence is a shipped artifact — with external anchors

If models are the primary authors (Bet 1), their competence in this language is engineered, not awaited (Bet 6). The project ships, as core infrastructure alongside the compiler: a **machine-consumable specification pack** (grammar, semantics summary, idiom catalog, diagnostic taxonomy — the distilled form of P18 and this document); a **synthetic corpus pipeline** — spec-grounded generation, filtered by the toolchain (compiles, passes tests, satisfies contracts), regenerated per edition (P15); and an **evaluation harness** measuring model competence as a first-class quality metric of every release.

**The circularity is guarded, not assumed away:** toolchain filtering validates internal consistency, not correctness or idiom — self-generated tests grading self-generated code measure nothing. Evaluation therefore rests on **external anchors**: the human-ported Bet 5 workload basket as ground truth, independently authored tasks, and real programs ported by people. Metrics are **slope and efficiency** — repair and generation correctness *per unit of adaptation budget*, improvement per corpus token, first-attempt fix rate on the language's own diagnostics over time — because raw comparisons against incumbents confound design with corpus scale and will read as failure for years regardless of merit. Absolute parity with Rust-trained models is a long-term consequence, not a launch criterion. Bet 6's pre-registered kill criterion (§3) is evaluated on these anchors and nothing else.

*Rationale.* A philosophy that optimizes only the verification side of the loop leaves generation to hope. A new language's distinctive constructs — a novel memory model, fault windows, boundary modules, contract levels — are precisely what models have no priors for; Rust needed years of organic corpus before models handled the borrow checker acceptably. A language for the human–LLM pair with no plan for the LLM half of day one is a philosophy with a hole in it.

### P20. Compile speed is earned by construction, not claimed

Fast compilation is a competitive claim in §1, and a claim without a mechanism is marketing; here are the mechanisms, as commitments. **Definition-site checking (P11) is a compile-speed architecture:** each generic is checked once, at its definition — instantiation is cacheable codegen, never re-analysis. **Strict module DAGs with no textual inclusion and no cross-signature inference (P2/NN#17)** make compilation incremental and parallel *by construction*: a module's interface is its signature set, so downstream work is invalidated only when signatures change, not when bodies do. Effects, contracts, and borrow checking are per-module analyses over explicit signatures — they never require whole-program passes. And the claim is **held to the document's own falsifiability standard**: pre-stability, the project pre-registers measurable compile-time targets (of the form: incremental rebuild of a large representative project in single-digit seconds; clean build within a stated multiple of the C toolchain baseline) tracked in CI as release criteria. If the targets cannot be met, the §1 claim is withdrawn by amendment rather than quietly kept.

*Rationale.* An earlier draft claimed compile speed in its identity and never argued it — a reviewer correctly called it marketing smuggled past the falsifiability apparatus. The honest accounting: this design *also* carries costs Rust carries (bound checking, coherence, borrow analysis) plus new ones (contracts, effect propagation). The bet is that the architecture above — check-once generics, signature-bounded invalidation, no textual inclusion — dominates those costs. That is measurable, so it is measured.

---

## 5. Non-Negotiables

Violating any of these requires amending this document first (§9); the burden of proof is heavy.

1. **No undefined behavior in safe code.** None — no side doors: contracts never license optimizer assumptions (P8), and unchecked arithmetic exists only inside unsafe regions (P5).
2. **Source-declared semantics.** For a given target, build configuration never changes observable behavior of fault-free executions, and faulting executions differ only within the bounded, defined fault window of P5. Every semantic choice — check level, arithmetic regime, fault policy — lives in source text, never in a build flag.
3. **Safe by default; unsafety only inside explicit, marked, justified regions.**
4. **Silent integer wraparound is impossible in default code.** Overflow faults inescapably; wrapping and saturating regimes are scoped and source-visible; unchecked overflow requires an unsafe region.
5. **No reads of uninitialized memory, ever.**
6. **No garbage collector; no mandatory runtime; freestanding targets are first-class.**
7. **No hidden allocation; allocators are explicit.**
8. **Errors are values in signatures; there are no exceptions.**
9. **No bidirectional, type-transforming concurrency partition (async-style function coloring) in the first stable version; no split ecosystems, ever.** (The one-way effect partition is owned and argued in P2.)
10. **Public generics are fully checked at definition site.**
11. **One canonical source form, enforced by the shipped formatter.**
12. **Every diagnostic is machine-readable, with provenance.**
13. **The grammar parses without semantic context.**
14. **No breaking change without an automatic migrator (sound conservative over-approximation qualifies); no stability commitment before Bet 5's pre-registered verdict.**
15. **No native C++ semantics; no stable native ABI without amendment — the stable ABI is C's, at boundary modules.**
16. **One toolchain — including the language server; builds are reproducible; provenance is recorded.**
17. **Nothing crosses a public signature by inference.** What a caller must know is written down.
18. **Foreign calls only through declared boundary modules.**
19. **The tracked-effect set is closed** (allocation, foreign trust); it grows only by amendment, shipped with conservative-default migration. Effects are upper bounds: signatures may overstate, never understate.
20. **A normative specification, independent of the compiler, defines the semantics** — including the memory consistency model, the unsafe-code aliasing model, and the formalized fault model.

---

## 6. Refusals — What This Language Is Knowingly Worse At

Accepted costs, not oversights — and, per this document's own warning, **debts, not absolutions**: each entry is something that will actually hurt. Do not "fix" them; do not mistake their being listed for their being handled.

- **Worse than C++ at template metaprogramming wizardry.** Expressiveness that defeats local verification is declined.
- **Worse than Rust and Go at ergonomic high-concurrency I/O, in the first stable version** — including for storage engines in our own domain, where the answer is threads and explicit state machines (P10).
- **Whole-system verification is not solved here.** Lock ordering, liveness, protocol correctness, cross-thread reclamation strategy — the expensive verification in kernel review — remain design-level human work. This language localizes the localizable fraction (aliasing, ownership, contracts, effects) and claims no more; the thesis is narrower than the word "verification" suggests, and this entry keeps it honest.
- **API churn when the truth changes — including the human cascade.** A dependency that starts allocating breaks its consumers' signatures transitively (P2). The diff is mechanical; the release cascade — coordinated major versions across maintainers — is not, and no tool absorbs it.
- **A one-way partition, with real gravity.** Effect checking partitions the call graph (P2 owns this). The no_std precedent says parallel-variant ecosystems are a live risk despite the mitigations; if the gravity wins, the resolution is a public P2-vs-P10 amendment, not denial.
- **Worse than C at portability to museum hardware.** Semantics are tightly defined per target; platforms that cannot honor them are not targets.
- **Worse than Zig at fitting in one head in one weekend.** Definition-site generics and the safety model cost surface; paid where P11 says, minimized elsewhere.
- **Loses benchmarks against UB-exploiting hand-tuned C++, and pays real detection costs in checked hot loops** (Bet 3's hardware honesty) — closed visibly by scoped regimes, never silently.
- **More ceremonial than its rivals where semantics are dangerous.** By design (P13).
- **Slower to accumulate features.** The budget (P6) means no is the default answer, including to good ideas.
- **Boundary contracts are mostly trust, not proof** (P17). We enumerate what is trusted; we do not pretend to verify C.
- **Its central bets may fail.** Bets 5 and 6 are research, each with a pre-registered kill criterion. If they fail, this document changes rather than being quietly reinterpreted.

---

## 7. What Success Looks Like

- A kernel module, a microcontroller firmware image, and a database storage engine are natural projects in the language, with no runtime shims.
- A competent C or Rust programmer reads unfamiliar code and judges its correctness locally — every signature tells them what they need.
- Bet 5's validation prototype exists *before* the syntax freeze, with a pre-registered kill criterion, and its results are published either way. Bet 6's criterion is evaluated on external anchors, same standard.
- The normative specification exists; the consistency model is adopted from proven art; the fault and aliasing models are formalized; the compiler is tested against the spec, not vice versa.
- The pre-registered compile-time targets (P20) are met in CI, or the claim is withdrawn by amendment.
- Model competence, measured by P19's externally anchored harness as slope per adaptation budget, improves with every release — a tailwind confirmed, not a moat assumed.
- Two codebases by strangers look like the work of one author.
- A security auditor lists boundary modules, unsafe regions, `assumed-proven` contracts, and scoped arithmetic regimes with one command — and can reproduce the binary bit-for-bit from source and lockfile.

---

## 8. To the Designers Who Come Next

You — human, model, or pair — will face questions this document does not answer: the borrow-inference algorithm, the contract-proving boundary, the text-type budget, the partition-free concurrency model P10 leaves open, syntax itself. Resolve them like this:

1. Check the Non-Negotiables. If the proposal violates one, the path is §9, not cleverness.
2. Apply the Priority Order — and its reclassification rule. Soundness, then source-declared semantics, then local verifiability, then predictable cost — before elegance, before speed of writing, before benchmarks.
3. Ask the thesis question: *does this make code cheaper to verify, for a human and a model working together?* Cheaper to verify includes cheaper to read (P13) — fewer tokens carrying more meaning.
4. When you add, remove. The budget is fixed.
5. Validate Bet 5 first — prototype, adversarial workload basket, pre-registered kill criterion — and do not freeze anything the verdict could invalidate. Build P19's competence pipeline in parallel, staged against Bet 1's evidence, because Bet 6 has no later.
6. Write down what you rejected and why, next to what you chose. The rationale is the product; future designers — and future models trained on your reasoning — need it more than the decision itself.

**Sequencing — the project obeys the budget too.** The blind review observed, correctly, that a document preaching "small core, no is the default" for the language commits the *project* to a foundation-scale roster: spec, mechanized formalization, contracts, effects, generics, comptime, editions with funded migrators, boundary tooling, formatter, reproducible builds, corpus pipeline, eval harness, language server. The resolution: **this document binds the end-state; a staged critical path binds day one.** Every "ships with the first stable release" is a stability-gate obligation, not a founding-day one. The critical path is short and ordered by what gates what: Bet 5's prototype (nothing survives its failure), the semantic core and its spec skeleton (P18's models are cheapest before code exists), a minimal toolchain, then breadth — with P19 staged against Bet 1's evidence rather than funded in full on faith. "When you add, remove" applies to project scope with the same force it applies to language surface; a philosophy that exempted its own project from its budget discipline would deserve the review it got.

---

## 9. Governance and Amendment

An amendment process needs an amender. The rules:

- **This document is amendable, in the open.** An amendment must (a) name the principle or non-negotiable it changes, (b) state the evidence or argument that overcame the burden of proof, and (c) append the change and rationale to Appendix A's ledger. Silent reinterpretation is forbidden.
- **Who amends.** A **single deciding authority** — an individual or small council, publicly named — enacts amendments and owns the default *no*: deprecating the second way (P3), funding migrators (P15), owning the formatter's output, keeping the effect set closed (P2), and treating parallel-variant ecosystems as a P3 violation. For amendments to **Non-Negotiables**, the authority's power is deliberately slowed: the proposal, its rationale, and the evidence must be published for open comment for a defined period before enactment, and the enacted amendment records the objections alongside the decision.
- **Succession is defined before it is needed.** A subordinate governance charter — required by this document, written at project founding — specifies how the authority is appointed, replaced, and succeeded.
- **Bets are review triggers.** Material evidence against a Bet (§3) — especially Bets 5 and 6 — makes review of the dependent principles mandatory. Missed pre-registered targets (Bet 5, Bet 6, P20) are enacted as amendments, not absorbed as silence.
- **Hierarchy:** this philosophy > design documents > implementation > compiler behavior. A conflict discovered lower resolves upward: the lower artifact changes, or this document is amended — never a quiet divergence. The specification (P18) sits at the design-document tier and binds the implementation absolutely.

---

## Appendix A — Change Ledger

### v1 → v2 (first adversarial review)

| # | Change | Finding | Resolution |
|---|--------|---------|------------|
| 1 | P12 rewritten; NN#17 added; Bet 5 created | Cross-signature lifetime inference contradicted P2; "lower load than Rust" was an unproven promise | Inference body-local only; signatures explicit with compact defaults; claim demoted to named research bet |
| 2 | P5 rewritten; NN#2/#4 reworded | Precise traps blocked optimization; per-op opt-outs would split the corpus; cross-platform "one semantics" unachievable | Imprecise-but-inescapable faults; scoped source-declared regimes; invariant scoped per-target; declared nondeterminism |
| 3 | P6/P11 rewritten | Comptime (instantiation-checked) contradicted definition-site generics; trait-system cost unacknowledged | Roles split: bounded generics for public APIs, comptime for constant eval and private codegen; cost named and paid |
| 4 | P8 check levels moved to source | "Checks in all build modes" repeated the failure every contract language walked back | Levels are per-module/per-contract source declarations |
| 5 | Bet 4 downgraded; §1 rewritten | Corpus scale dominates purity; parts of the "unretrofittable" claim already retrofitted by Rust | Coherence reframed as tailwind; claim narrowed to whole-loop design-fit |
| 6 | P10 accounting expanded; coloring defined | "Cedes network services" understated — storage engines are our domain; "blocking visible" was coloring unlabeled | Honest cost accounting; coloring defined as call-graph partition |
| 7 | P13 rewritten | "Verbosity is fine" argued from the wrong scarcity | Standard changed to clarity-density: information per token read |
| 8 | P14 softened | In-compiler C import understated maintenance; "cheap 90%" of C++ optimistic | Toolchain-level import; C++ coverage stated as partial |
| 9 | P7 panic policy added | Silence on panics in freestanding targets | Root-declared policy; no unwinding requirement |
| 10 | P17/NN#18 added | FFI is a soundness hole grep-for-unsafe cannot cover | Boundary modules: localized, contract-annotated, enumerable |
| 11 | P9 stdlib scope added | Silence risked an embedded/hosted split | Subtractive layering; one library |
| 12 | §9 governance added | Normative-yet-unamendable was a contradiction | Amendment process with public ledger |

### v2 → v3 (second adversarial review)

| # | Change | Finding | Resolution |
|---|--------|---------|------------|
| 1 | P5 invariant re-scoped; NN#2 reworded | Imprecise faults silently broke "one semantics per target" for faulting executions | Invariant stated with true scope: fault-free deterministic; faulting truncated within a fault window |
| 2 | Bet 3 rewritten | Imprecision restores scheduling, not SIMD detection cost; window collapses in MMIO-dense code | Hardware honesty; scoped regimes expected routinely; degradation named |
| 3 | P2 effects confronted; NN#19 added; blocking demoted | Signature effects are a transitive effect system; ripple makes internal changes breaking | Set closed and tiny; churn owned; blocking cut to documentation |
| 4 | P8 optimizer question answered; P7 unified | Silence on whether the optimizer may assume contracts invited UB; audit/fault semantics undefined | Contracts never license optimization; audit = record-and-continue; one fault vocabulary and policy |
| 5 | Bet 5 operationalized and narrowed | No kill criterion; circular validation; honest delta vs Rust is value-first only | Pre-registered criterion; throwaway-syntax prototype + adversarial basket; 1.0 gated on verdict |
| 6 | Bet 6 and P19 added | Cold-start silence; incumbents' corpus advantage is an active barrier | Competence as shipped artifact; slope metrics |
| 7 | P18/NN#20 added | No normative spec; total silence on memory consistency model | Spec independent of compiler; consistency model from proven art |
| 8 | P11 instantiation predictability | Compiler-chosen strategy was an invisible performance cliff | Deterministic, documented, stable-within-edition default; source override |
| 9 | §9 completed | No defined amender; no succession | Authority named; NN amendments slowed by open comment; succession charter |
| 10 | P17 reframed | Boundary contracts oversold — retention/threading not checkable | Trust declarations, enumerable; verification claim withdrawn |
| 11 | P15 residual costs named | Migrators fix codebases, not trained corpora; post-1.0 Bet 5 failure unmigratable | Corpus cost accepted; stability gate explicit |
| 12 | §2 reclassification rule; P16 reproducibility; P14 ABI; P3 text | Priority order gameable; silences on supply chain, ABI, string types | Rule with recorded rationale; reproducible builds; C-ABI position; text as named design obligation |

### v3 → v4 (history-blind review of v3 — reviewer saw no version history)

| # | Change | Finding | Resolution |
|---|--------|---------|------------|
| 1 | P2 partition argument rewritten; P10 and NN#9 re-scoped | **Re-flag: the v3 fix failed its blind test.** "Effects aren't coloring" was a definitional dodge — a no-alloc caller cannot call an allocating function; that is a checker-enforced partition | Defense withdrawn on the record; replaced by an argument of shape and degree (one-way with universal ground floor; capability as value, not function-type transformation; enforcing a constraint physics imposes) plus a named residual risk (no_std gravity) with mitigations and an amendment path if gravity wins |
| 2 | P15 migration rule + NN#19 | Rule interaction silently froze the effect set forever: adding an effect can't be mechanically migrated, so the amendment could never ship — making blocking's exclusion permanent by accident | Conservative-default over-approximation defined as qualifying mechanical migration; blocking's exclusion restated as a revisable judgment call with a named revisit trigger |
| 3 | P5 window bounded and owned; P18 narrowed; NN#20 extended | "Bounded fault window" was bounded by nothing — research-grade novel semantics hand-waved in a document that punishes vagueness, and in contradiction with P18's no-novelty rule | Bound defined (next synchronization or observable effect in program order); novelty conceded and named; formalization made mandatory pre-stability work; P18 narrowed to "proven art where proven art suffices" |
| 4 | P5 unchecked arithmetic; NN#1/#4 | Unchecked overflow in a safe scoped regime is literally UB in safe code — an open side door | Unchecked moved inside unsafe regions only; safe regimes are wrapping and saturating |
| 5 | Bets 1/2/6 restated | Falsifiability applied rigorously to Bet 5, cosmetically to Bets 1/2/6; Bet 6 (equally existential) had no kill criterion; Bet 6's validation was circular (self-generated tests grading self-generated code) | Bet 6 given a pre-registered criterion on external anchors (human-ported basket, independent tasks); Bet 1's real work named as project-economics with staged funding; Bet 2 restated in its durable form (locality is cheap verification at any capability) |
| 6 | P20 added; §1 claim referenced to it | "Faster compiles" claimed in the identity with no supporting principle — marketing smuggled past the falsifiability apparatus | Mechanisms named (check-once generics, signature-bounded invalidation, no textual inclusion); pre-registered measurable targets tracked in CI; claim withdrawn by amendment if missed |
| 7 | §1 and P8 scope honesty; Refusals entry | "Verification" quietly redefined as its localizable fraction; lock ordering, liveness, protocol correctness untouched | Scope stated in the identity; Refusals names whole-system verification as unsolved by this language |
| 8 | P2 upper-bound rule; Refusals cascade entry | Reverse-direction effects undefined (is a stale marker a lie?); release cascade cost understated as "one command" | Effects defined as upper bounds — never understate, overstating is permitted conservatism; human release-cascade named as an unabsorbed debt |
| 9 | P18 aliasing model; P16 language server | Two silences: the unsafe-code aliasing model (Rust's decade-long open wound, soundness-load-bearing) and the language server (the loop's inner surface, absent from the toolchain list) | Both added as named obligations — aliasing model in the spec's mandatory scope, language server in the blessed toolchain |
| 10 | §8 Sequencing added; header warning added | The project exempts itself from the language's budget philosophy — foundation-scale commitments with no economics; and the document's fluency at admitting costs masquerades as managing them | End-state vs. critical-path distinction; staged funding tied to bets; "when you add, remove" extended to project scope; the meta-warning written into the header where every future reader starts |

### v4 → v4.1 (enactment of the Bet 5 verdict — §9 amendment, 2026-07-07)

| # | Change | Finding | Resolution |
|---|--------|---------|------------|
| 1 | Bet 5 verdict clause added; P12 concession added; header version bumped | The pre-registered criterion (docs/BET5_CRITERION.md, frozen at the first port commit) computed KILL: three valve-fraction ceilings breached across the five-program basket, while the load claim passed decisively (53% aggregate annotation reduction vs idiomatic Rust, zero copy inflation, all five programs completable). Published defect analysis attributes two breaches substantially to a fraction-vs-density artifact; the allocator's (63% valve lines) stands on any reading | Verdict enacted as registered, never retrofitted (criterion §0.3). Bet 5 remains open pending a successor pre-registration — openly data-aware, changing only the defective operationalization — that must pass on the frozen artifacts before any stability reliance on P12; NN#14's gate stays closed. P12 gains a named limit: value-first does not carry allocator-class code (the valve is that program's spine, visibly). Design consequences recorded for the next round, not decided: safe typed field projection on rawptr; the call-site reborrow ceremony. Full record: docs/RESULTS.md, docs/ADJUDICATIONS.md, docs/reviews/, enacted from docs/proposals/2026-07-07-bet5-amendment.md (Option C) |

### v4.1 → v4.2 (disposition of the Bet 5 mandatory review — §9 amendment, 2026-07-08)

| # | Change | Finding | Resolution |
|---|--------|---------|------------|
| 1 | Bet 5 disposition clause added; NN#14 gate conditioned on commitments; header version bumped | The ratified successor registration (docs/BET5_CRITERION2.md; two hostile reviews, all outcome tables under chosen and rejected designs published) re-scored the frozen artifacts to MANDATORY REVIEW: load claim decisively confirmed (53% aggregate reduction), allocator WARN on the conceded class, scheduler baseline-sensitive (0.528 registered / 23.5 authored-only) | Review ruling: Bet 5 provisionally confirmed on this basket. Registered self-contained yardstick held correct for a language-model bet; the authored-only gap recorded as an ecosystem-bootstrap fact owned by Bets 4/6. Binding commitments: safe rawptr field projection and reborrow ergonomics in the next design round; the scheduler re-ported and re-measured under the frozen successor rules before any syntax freeze, returning to review if worse. Full record: docs/RESULTS.md, docs/reviews/2026-07-07-criterion2-review-1.md, docs/reviews/2026-07-08-criterion2-review-2.md |
