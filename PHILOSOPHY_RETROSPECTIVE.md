# Retrospective — What Designing, Building, and Measuring Candor Taught

*Read alongside `LANG_PHILOSOPHY.md`. The philosophy stated the hypotheses; this
records what testing them produced. It is a journey log, not a critique of the
prose — the findings below come from the actual arc: seventeen adversarially
reviewed designs, a self-hosting compiler across five execution engines, a package
system, a real standard library, and two pre-registered measurement campaigns that
were run and scored, not imagined.*

*Scope, stated honestly like everything else here: one long build, largely by one
model author, on one target (x86-64 Linux). These are the findings the first pass
raised. The second pass confirms or kills them.*

---

## The arc

The project did the thing in the order the philosophy demanded. **Design first**:
the memory model, then generics, modules, faults, the foreign boundary, packages —
each a document, each run through a hostile review with a mandate to break it, each
ruling recorded with its rejected alternatives. **Prototype and validate the
load-bearing bet next**: a throwaway-syntax compiler + interpreter, used only to
port a fixed, adversarially-chosen basket of five systems programs and *measure*
whether value-first defaults actually lower annotation load. **Then breadth**: the
real syntax, the five engines, self-hosting, packaging, the std. **Measure the
model-authorship bet in parallel**: an eval harness scoring first-attempt
correctness on external anchors.

Two of those phases produced results that were more instructive than any argument in
the design docs, because they were uncomfortable and we had committed in advance to
publishing them either way.

---

## What the measurements taught

### Bet 5: the verdict was a KILL, and the most valuable finding was that our metric was wrong

The pre-registered criterion for "value-first lowers cognitive load" computed **KILL**
on the first run, and we enacted it as registered rather than retrofitting the
thresholds. But the evidence cut cleanly in two directions, and *that split is the
finding*:

- The **load claim passed decisively** on every program — a 53% aggregate reduction
  in annotation-per-statement versus idiomatic Rust (allocator −79%, arena −52%,
  scheduler −43%, MMIO −29%). Value-first genuinely does write less ceremony.
- The **valve-fraction gates breached** on three of five programs — and the
  post-mortem found *why*, and it was partly the ruler, not the language. Candor
  implements the same specification in 2–5× fewer statements (the allocator: 178
  logical statements against the Rust baseline's ~930 lines). So identical
  *absolute* pointer-code content shows up as a higher *fraction*. The two
  allocators' absolute valve content is comparable (112 vs ~150 lines); the fraction
  diverged mostly because Candor needed far less safe scaffolding around it.

The lesson was not the verdict. **It was that pre-registration plus publish-either-
way did exactly what it was for: it forced us to confront that our first
operationalization measured density as if it were pointer-heaviness.** The honest
move — write a *data-aware* successor criterion that fixes only the defective metric,
name the conflict in the open, re-score the frozen artifacts — produced a provisional
confirmation that means something *because* the first one was allowed to fail
loudly. A project that had tuned the thresholds to pass would have learned nothing.
The falsifiability apparatus earned its keep by biting us.

### The allocator concession: we measured our own thesis failing on its hardest instance

One breach survived every measurement artifact: the allocator, at 63% valve lines.
Value-first does **not** carry allocator-class code — programs whose core is in-band
metadata threaded through the memory it describes. There the pressure valve is not
rare-but-critical; it *is* the program's spine. We measured this precisely and
recorded it as a named limit of the memory model: "the valve is the program, and the
valve is visible."

That is the most honest thing the project produced, and it is more useful than a
clean pass would have been, for two reasons. First, it is *true on our home ground* —
allocators are in the identity statement — so pretending otherwise would have been
the exact self-deception the philosophy warns against. Second, the measurement named
its own remedies: the two largest friction sources in the ports were **the lack of
safe typed field projection on `rawptr`** (≈10 one-line accessors forced into the
valve) and **call-site reborrow ceremony**. Those weren't opinions; they were the
top line-items in a scored port, and they became the next design round's agenda,
driven by data instead of taste. **Finding: the way to improve a value-first
language is to measure where the valve concentrates and attack those specific
ergonomics — not to argue about defaults in the abstract.**

### The eval saturated: we learned we cannot yet measure the second bet

The model-competence campaign ran three rounds and returned **12/12, then 23/23,
slope delta 0** — perfect scores, no measurable slope. A strong model already aces the
seed and graduation task sets, so they cannot detect the thing the bet is about
(improvement *per unit of adaptation*). "A positive floor, not a slope," as the round
README put it.

Scoping the criterion further this session found the deeper wall: the only axis the
lab can actually run — in-context adaptation (spec pack in the prompt of an unchanged
model) — measures **prompting, not manufacturing**, and would launder frontier-model
improvement as a confirmation of the project's corpus work. The bet as stated needs a
fine-tuning / corpus-training pipeline the lab does not have.

**Finding: Bet 6 is not in the same epistemic position as Bet 5, and running the
measurement is how we learned it.** Bet 5 was falsifiable by *labor* — a human ports
a basket. Bet 6, done honestly, is falsifiable only by *capital* — infrastructure
that is itself a bet-scale investment. It may be un-fund-able to test, which is a
weaker and more honest status than "awaiting verdict." We did not reason our way to
this; we tried to measure and hit the ceiling.

---

## What the implementation taught

### Soundness broke — three times — and never where it was validated

The philosophy makes soundness priority #1 and defends it with everything aimed at
the memory model: the validation basket, the spec, the formalized fault window. That
worked: **the value/borrow model is the one thing that did not break.** Validating
the right thing first paid off exactly as intended.

But safe, well-typed code produced undefined behavior three times this build, all in
the machinery the design treated as merely "complex" rather than "risky":

- An associated-type projection that failed to normalize through a concrete base →
  the tree-walker returned garbage, the MIR interpreter panicked, the native backend
  **segfaulted** — three different failures from one well-typed program.
- Method dispatch keyed on `(type, method)` but not the interface → a call the
  checker resolved as `A::tag` ran `B::tag` on every engine; with differing return
  types, type confusion the checker had *approved*.
- The canonical formatter rewrote a reborrow (`read v.*`) to a move (`v`) → borrow-
  broken output, on the shipped standard library's own source.

**Finding: a sound design is a hypothesis about the compiler, not a guarantee of
one — and the hypothesis was false in the generics/coherence/dispatch plumbing,
which got no validation prototype, no spec tier, no bet.** The memory model was
doubted rigorously and held; the trait machinery was assumed and broke.

### The five engines were the enforcer — differential execution turned NN#1 into an observed property

Every one of those bugs was caught the same way: **independent implementations of the
semantics disagreeing.** Tree-walker vs. MIR interpreter vs. two native backends,
gated to byte-identical results, is what made "no undefined behavior in safe code" a
thing we *observed* rather than *asserted*. A single implementation can only be
consistent with its own bugs; four that must agree cannot hide a divergence.

This was the most load-bearing engineering decision in the whole build, and the
philosophy does not name it. **Finding: differential execution across ≥2 independent
engines is the operational counterpart to the spec — the spec says what is true, the
engines catch when a compiler drifts from itself — and it belongs in the philosophy
as a first-class methodology, not as an implementation detail we happened to have.**

### "Fails safely" vs. "miscompiles" was the real severity axis

Repeatedly the question at an unfinished feature was not "is it done" but "when it is
*not* done, does the compiler reject the case at check time, or silently miscompile
it?" `str ==` not being lowered was a clean rejection — a fine limitation.
Associated-type projection not normalizing was a miscompile — a soundness hole. Same
shape of gap, opposite stakes, and the entire difference was reject-vs-miscompile.

**Finding: an incomplete compiler is sound iff every construct it cannot fully handle
is rejected at check time. A gap that type-checks and then miscompiles is an NN#1
violation regardless of whether the feature is "shipped." This is the single most
useful operational rule the build produced, and it turns a compiler's unfinished
edges from a roadmap into a soundness surface.**

### Reproducibility leaked through paths, twice

"Bit-identical artifacts" did not fail as a grand property; it failed as two small
sites where the build environment slipped into an artifact: a package-id derived from
the absolute directory (leaking the path into every mangled symbol) and an absolute
path written into the lockfile. Both produced machine-specific outputs from identical
source. **Finding: reproducibility is defended site-by-site, not asserted once — the
recurring enemy is a path, a clock, or an unordered iteration reaching an artifact,
and it is worth naming that specific threat rather than the goal.**

### Self-hosting was a measurement instrument, not just a milestone

Writing the compiler in the language — checked, interpreted, lowered, and compiled by
itself, each tier byte-exact against the Rust oracle — was the credibility proof it
was meant to be. But it was also the *largest* program in the language, and it
dogfooded the module system and generics at a scale no fixture reached: the two
module-tree qualification gaps this session were found because corelib/self-host-
shaped code exercised bounded generic impls that single-file fixtures never did.
**Finding: dogfooding at scale is an instrument that finds soundness and ergonomic
obligations a curated test suite structurally cannot. It should be scheduled as
measurement, not celebrated as an achievement.**

---

## What the process taught

- **The design pipeline was the highest-leverage thing the project did.** Design
  0012 (concurrency) was rejected outright when a reviewer built a real safe-code data
  race against its flagship, then reworked to acceptance. The borrowed-iteration
  design's use-after-frees, the package system's reproducibility SINK, and the Bet-6
  criterion's construct-validity failure were all caught in hostile review *before*
  they shipped. Draft → break-it review → ruling → recorded rationale is slow, and it
  is the reason the design held under a very long build. Nothing here would change it.

- **Bet 1 was validated in real time.** This document, and the compiler it describes,
  were authored by the human–model pair the whole thesis is premised on. The design
  earned its keep concretely: machine-readable diagnostics drove the repair loop,
  local signatures made it safe to delegate work across agents, and the differential
  engines caught the model author's own mistakes. The thesis was not a forecast this
  session; it was the operating condition.

- **The discipline held under load.** "When you add, remove," publish-either-way,
  record-what-you-rejected, the priority order — these survived a build long and
  intense enough to break sloppier rules. That is itself a finding: the process is not
  ceremony that works when calm; it worked precisely when the temptation to cut it was
  highest.

---

## What I would carry into a restart

These fall out of the findings above; they are not new opinions imposed on the text.

1. **Add a validation bet for the trait/coherence/dispatch system, at Bet 5's tier.**
   It is as unmigratable as the memory model (coherence and dispatch keying are baked
   into every generic signature in the corpus) and it is where soundness actually
   broke, three times. It deserved a prototype and a kill criterion and got neither.
2. **Elevate differential execution and "reject-don't-miscompile" to named
   commitments.** They are what enforced NN#1 in practice; leaving them unwritten
   left the enforcement to luck.
3. **Grade Bet 6 as possibly untestable, not merely unverified** — because measuring
   it is what revealed the difference, and "same standard as Bet 5" is not true.
4. **Make the canonical formatter's semantics-preservation a hard rule** — a tool
   that can rewrite the source's meaning contradicts "meaning lives in the source" at
   the center, and we shipped exactly that bug.
5. **Keep the measurement loop pointed at the valve, not the defaults.** The
   allocator concession and its named friction sources (safe `rawptr` projection,
   reborrow ceremony) are how the language actually improves; the abstract argument
   about value-first ordering was settled by the number, not the prose.

The through-line: the philosophy is a superb theory of how to *design* a sound
language and a thin theory of how to *build and measure* one — and the two campaigns
that mattered most, Bet 5's KILL-then-confirm and the eval's saturation, taught more
by being run and scored than any design document taught by being argued. The project
already knows how to doubt its design. The restart should teach it to doubt its
compiler and its metrics with the same seriousness, because that is where, this time,
the truth diverged from the source.
