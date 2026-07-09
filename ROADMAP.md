# Candor roadmap

Rulings by the deciding authority (2026-07-09) on publication and self-hosting; the maturation
queue below is ordered, not dated.

## Publication staging

1. **This repository is the lab, permanently.** The full experiment record — philosophy,
   adversarial reviews, the frozen Bet 5 experiment, designs, gates — is the project's
   distinctive asset and is never diluted into a product repo.
2. **The 0.x preview ships from a separate distribution repository** created at the packaging
   milestone: the toolchain (renamed `candor`), spec, stdlib, editor support, getting-started.
   Explicitly unstable; this repo linked as the design record.
3. **1.0 is the stability gate, not a date:** P15's edition/migrator promises live, P20's
   pre-registered compile-time targets ratified in CI, the spec's obligations ledger clear of
   pre-stability items (NN#20 mechanization decision included). NN#14's Bet 5 condition is
   already satisfied.

## Self-hosting

The compiler is Rust and remains the bootstrap/reference implementation permanently. A
self-hosted compiler is the project's ultimate dogfood (a compiler is the basket's own home
ground), its largest P19 corpus, and its credibility proof — gated on std, not the language:

1. The P3 text-type budget design (the named deferred obligation).
2. An I/O layer as a boundary module over libc (what the P17 boundary exists for).
3. Then port the CHECKER first — highest value, its own domain — with the Rust implementation
   as the differential oracle, per the house methodology.

## Maturation queue (ordered)

- Graduation-tier eval campaign (the first slope-capable measurement).
- Toolchain packaging: candor-proto → candor, install story, the distribution repo (publication
  step 2 above).
- Text-type budget design (P3's named obligation; gates self-hosting and real std growth).
- I/O boundary module (gates self-hosting).
- Bare-metal target (blocked locally on qemu; the freestanding proof stands meanwhile).
- LLVM second backend behind the MIR seam and differential gate.
- Corpus scale-up; per-edition regeneration.
- Self-hosted checker (after text + I/O).
- Stability-gate proceedings (1.0) when the checklist clears.
