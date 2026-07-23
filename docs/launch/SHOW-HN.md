# Show HN — title and first comment

*Conventions honored: plain title, no adjectives or hype; the first comment is
the author's context, short, with the honest limits up front. Post the preview
repo URL (github.com/k1832/Candor). Submit a weekday morning US time and stay
in the thread for the first two hours.*

## Title (pick one)

1. `Show HN: Candor – a memory-safe systems language co-authored with an LLM`
2. `Show HN: Candor – a systems language whose design experiment is public, kill verdict included`
3. `Show HN: Candor – value-first memory-safe systems language (0.x preview)`

Recommendation: #1. It states the artifact and the one genuinely unusual fact.
The design-record angle then belongs in the first comment, where it can carry
nuance.

## First comment (post immediately after submitting)

---

Author here. Candor is a memory-safe systems language built on one design bet:
value semantics as the default gear (moves and explicit copies, no lifetime
annotations), borrowing second, and visible escape hatches where neither fits.
Checked arithmetic by default, errors as values, contracts, structured
concurrency without async coloring, explicit allocators, an FFI boundary you
can audit with one command, freestanding no-libc binaries.

The unusual part is how it was built and validated. The design and
implementation were written by an LLM with me as the deciding authority, and
because "the author was careful" is not a trust model, the trust lives in the
apparatus instead:

- Every program in the test corpus must produce byte-identical results on five
  independent execution engines (two interpreters, Cranelift ±opt, LLVM);
  WebAssembly tests are additionally checked against an external runtime.

- The founding bet had a pre-registered, frozen kill criterion measured on five
  adversarially chosen systems programs (allocator, intrusive scheduler, MMIO
  driver, parser, arena) hand-ported to Candor and to Rust. The first verdict
  was KILL, and we published it — the metric was partly at fault (it punished
  the language for needing 2–5× fewer statements), but one finding stands
  permanently: value-first does not carry allocator-class code, and we document
  that as a limit rather than working around it. The repaired criterion, the
  hostile reviews of it, and the eventual provisional confirmation are all in
  the lab repo.

- Compile speed is a CI-gated claim, not a vibe: a 50kL reference project
  type-checks in ~1.4s, incremental rebuilds ~36ms, optimized native at ~1.35×
  `cc -O2` compile time on comparable code. If a push breaches the ratified
  thresholds, the build fails.

Honest limits: this is an unstable 0.x preview (1.0 is a defined gate, not a
date), x86-64 Linux only, one platform tier, and the standard library is small.
The compiler is written in Rust; the self-hosting is at the credibility level
(~19kL of Candor that checks, interprets, lowers, and natively compiles the
systems corpus byte-exact against the Rust reference), not a bootstrapped
toolchain.

Ninety-second version: clone, `cargo build --release` (~30s), then
`candor run examples/11_wasm_interp.cnr` runs a from-scratch WASM interpreter
written in Candor, and `examples/12_http_server` is a web server you can curl.

The full design record — seventeen adversarially reviewed design docs, the
frozen criteria, the KILL, the soundness bugs the differential suite caught in
our own compiler, and a retrospective — is in the lab repo, linked from the
README. If you want to attack something, that's the place to start.

---

## Mechanics

- Submit the PREVIEW repo (`github.com/k1832/Candor`), not the lab; the README
  leads with the showcase and links the lab.
- Best window: Tuesday–Thursday, 7–9am US Pacific.
- Do not submit the story post the same day; if the Show HN goes well, the
  story post is the follow-up a week later (or someone will submit it for you).
- If the thread asks something not in PREPARED-ANSWERS.md, answer plainly and
  add it to the sheet afterward.
