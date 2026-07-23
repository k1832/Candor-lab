# We pre-registered a kill criterion for our language's core bet. It killed. Here's what happened next.

*Draft blog post for the Candor launch. Every number and event below is on the
public record in the lab repository (github.com/k1832/Candor-lab) — the criteria
are frozen with hashes, the reviews are committed, and the verdicts were enacted
as registered. Post under your own name; edit freely.*

---

I've been building a systems programming language called Candor together with an
LLM. Not "with AI assistance" in the autocomplete sense — the model wrote the
design documents, the compiler, and the reviews, and I acted as the deciding
authority: ruling on design forks, ratifying or rejecting proposals, and owning
every publish. The experiment was to take the human–LLM pair seriously as *the*
authorship model and see whether discipline could make it trustworthy.

This post isn't mainly about the language. It's about the moment our own
measurement apparatus told us our founding bet had failed, and what doing the
honest thing with that verdict looked like.

## The bet

Candor's core design bet ("Bet 5" in our philosophy doc) is that **value-first
semantics — ownership transfers and copies as the default gear, borrowing as the
second — lowers the cognitive load of real systems code**, compared to
borrow-first languages. The uncomfortable part, which we wrote down on day one:
pointer-rich code is concentrated in exactly the programs a systems language
exists for. The bet's hardest instance is our home ground.

A bet you can't lose is marketing. So before building anything real, we froze a
falsification protocol:

- A fixed basket of five systems programs, chosen adversarially to include the
  workloads value semantics fits *worst*: a memory allocator, an intrusive-list
  scheduler, an MMIO driver state machine, a recursive-descent parser, an arena
  compiler pass.
- Each ported by hand to Candor and to idiomatic Rust as the baseline.
- Frozen metrics: annotation density (how much ceremony you write), and the
  "pressure valve" fraction — how much of the program has to drop out of the
  safe value-first model into explicit unsafe/raw-pointer escape hatches.
- Pre-registered thresholds, hashed and committed *before* measurement. Breach
  the ceilings, and the criterion computes KILL. Publish either way.

## It killed

On 2026-07-07 the criterion computed **KILL**, and we published it as
registered.

The evidence cut both ways, which is exactly why publishing it mattered. The
load claim passed decisively: a 53% aggregate reduction in annotation per
statement versus idiomatic Rust across the whole basket (the allocator was
−79%), with zero copy inflation and all five programs completing their full
test vectors. But three of five programs breached the valve-fraction ceilings —
including the allocator at 63% valve lines against a 0.40 ceiling.

The most valuable thing the experiment produced was understanding *why*, and it
wasn't flattering to our metric. Candor implements the same specifications in
2–5× fewer statements than the Rust baselines. The allocator port is 178 logical
statements against roughly 930 lines of Rust. Identical *absolute* amounts of
pointer code therefore show up as much higher *fractions* — we had accidentally
built a ruler that punished the language for needing less scaffolding. Two of
the three breaches were substantially this artifact. (For the record: the two
allocators' absolute valve content is comparable — about 112 lines in Candor
versus about 150 in Rust.)

One breach survived every artifact we could identify. The allocator really is
63% escape-hatch, and no measurement correction changes that. Value-first does
not carry allocator-class code — programs whose essence is in-band metadata
threaded through the raw memory they manage. We recorded that as a named,
permanent limit of the design: on that class, the honest posture is "the valve
is the program, and the valve is visible."

## What "honest" cost us, and bought us

The tempting move at this point is quiet repair: adjust the thresholds, re-run,
declare victory. Our governance rules (also written before any of this) forbid
retrofitting a frozen criterion. So instead:

1. We enacted the KILL as registered and published the numbers.
2. We wrote a **successor criterion** — openly data-aware, changing only the
   defective operationalization (measuring absolute valve content and a
   valve-ratio against the baseline, instead of the density-sensitive
   fraction), with the conflict of interest named in the document itself.
3. The successor went through two hostile reviews before ratification, then
   re-scored the *same frozen artifacts*. Verdict: provisionally confirmed,
   with the allocator concession standing.
4. The confirmation came with **binding commitments**, because the ports had
   told us exactly where the friction was: the largest avoidable valve surface
   was reading a field's address through a raw pointer (so we designed safe
   typed field projection), and the largest reading friction was reborrow
   ceremony (so we redesigned that). And the scheduler — the closest breach,
   0.4120 against a 0.40 ceiling — had to be re-ported and re-measured under
   the frozen successor rules before any stability milestone, with the
   confirmation returning to review if the number got worse.

Last week we ran that re-measurement. The re-port, using the two features that
exist *because* the original measurement hurt, came in at 0.3934 — below the old
ceiling. The features built to fix the breach measurably fixed the breach. That
closed the loop the original KILL opened: bet → kill → diagnose → repair the
metric honestly → repair the *language* where the metric was right → re-measure
under frozen rules.

## The other thing that kept us honest: five compilers that must agree

A language whose code is written by a model needs its trust located somewhere
other than "the author was careful." Ours lives in redundancy: every Candor
program in our test corpus runs on five independent execution engines — a
tree-walking interpreter, a mid-level IR interpreter, Cranelift without and with
optimization, and an LLVM backend — and the results must match byte-for-byte.
For WebAssembly programs, also against an external reference runtime. The
compiler is additionally self-hosted at the credibility level: about 19,300
lines of Candor that check, interpret, lower, and natively compile the systems
corpus, verified byte-exact against the Rust reference implementation.

This apparatus is not decorative. It caught, among others: an associated-type
normalization bug where well-typed safe code produced garbage on one engine, a
panic on another, and a segfault on a third; a dispatch bug where the checker
resolved one interface's method and every engine unanimously executed a
different one; and our canonical formatter rewriting a reborrow into a move —
on the standard library's own source. Every one became a spec clause and a
standing CI gate after it was fixed. The differential suite is the reason "no
undefined behavior in safe code" is an observed property of this compiler
rather than an aspiration.

## Where it stands

Candor 0.x is a public preview (github.com/k1832/Candor): memory-safe and
value-first, checked arithmetic by default, errors as values, contracts,
structured concurrency without async coloring, an audited FFI boundary you can
enumerate with one command, explicit allocators, freestanding no-libc binaries,
and a small real std. It builds in about half a minute and the examples include
a from-scratch WebAssembly interpreter and a working HTTP server, both written
in Candor. Compile times are a pre-registered, CI-gated claim, not a vibe: a
50,000-line reference project type-checks in about 1.4 seconds, incremental
rebuilds are ~36ms, and optimized native output currently lands at ~1.35× the
compile time of `cc -O2` on comparable code.

It is explicitly unstable — 1.0 is a gate we've defined, not a date — and it
targets x86-64 Linux today. The full design record, including every adversarial
review, the frozen criteria, the KILL, and a retrospective on what we'd do
differently, is public in the lab repository. If the story above makes you want
to poke holes in it, the repo is arranged for exactly that.
