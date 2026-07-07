# Adjudication log (criterion §6.5)

Rulings by the deciding authority on ambiguities flagged during validation. Per criterion §6.5/§6.7,
each ruling is published for a minimum 7-day open-comment period before taking effect; while the
repository is private the period runs from the repository's publication date (FREEZE_MANIFEST
publication ruling), and objections received are recorded here.

## 2026-07-07 — Baseline construction batch

**R1 (structural). Measured-artifact self-containment.** The measured Rust source must contain the
spec-mandated core mechanics in-source; cargo dependencies providing them (allocator free-list
engine, scheduler intrusive-list machinery) are vendored — used subset only, upstream provenance
and license preserved, no restyling beyond dead-code removal. *Reason:* the comparison target
necessarily carries this machinery in-source; a dependency boundary would exclude exactly the
unsafe-dense code M2/M3 exist to count. Enacted at commit b689860.

**R2 (allocator).** A22 runs on a region of exactly ten blocks (10 KiB), the sole reading under
which the vector tests what it claims (a 1 MiB region's tail would satisfy the final allocation and
defeat the anti-cheat). — **R3 (allocator).** `realloc` error precedence follows §2.4's listed
order: `E_INVALID_SIZE` (new_size==0) is checked before pointer validity. — **R4 (allocator).**
A10's literal 64 is an upper bound on per-block overhead; A22 uses the implementation's declared
HDR. Both hold for any HDR ≤ 64; implementations must declare HDR and satisfy both as written.

**R5 (scheduler).** `admit` is valid only from New/Exited; a BLOCKED task is "present" and admitting
it is `E_ALREADY_QUEUED` (§4.3's presence semantics govern §2.2's error). — **R6 (scheduler).**
`pick_next` while a task is RUNNING is a no-op returning None (no preemption, §5.1); callers
deschedule first.

**R7 (mmio).** `err_at` is supplied per-transfer-attempt as an ordered schedule consumed one entry
per transfer; M9's "armed for every transfer" means one entry per attempt. — **R8 (mmio).** CTRL
writes evaluate the §2.4 bit-rules independently in listed order; for the driver's fixed values no
ambiguity is reachable. — **R9 (mmio).** Exposing `init`/`transfer` as public entry points mirroring
§3's named functions is the intended reading of M6 (a transfer without re-init); `run()` remains the
composed path.

**R10 (parser).** An unclosed call argument list reports `E_EXPECTED_RPAREN` at the offending
token's offset (EOF ⇒ offset = input length), by analogy to grouping (P24). — **R11 (parser).** A
valid-but-wrong token where `)` is required reports `E_EXPECTED_RPAREN` at that token's start
offset. — **R12 (parser).** Parenthesized grouping is span-transparent: the inner node keeps its own
span; parens contribute no node and no span, consistent with P6/P19's serialization.

**R13 (arena).** Under an index arena, AR26's "no reference into src" is verified as "every
reachable id resolves within dst" — the natural index-arena reading of a property a pointer arena
enforces by construction; spec §1.2 explicitly permits index arenas.

*All thirteen rulings adopt the baseline authors' flagged resolutions after review; none required a
baseline change beyond R1's vendoring. Objection window: open; none received as of this writing.*

## 2026-07-07 — Allocator port scoring batch

**R14 (measurement symmetry).** The Candor language is single-file, so a port carries its vector
harness in-file; the Rust baselines were measured over `src/` excluding `tests/`. Ruling: the
measured Candor artifact is the implementation section, mechanically delimited by the first
line matching `// Test harness` (everything above it); the harness below is excluded. Split
verified to parse and check standalone. Applies to all five ports.

**R15 (allocator idiomaticity + cell-substitutability).** The scored allocator port is confirmed
idiomatic: unsafe blocks wrap genuinely interleaved free-list pointer work (per design 0001's own
§11.1 idiom and its §10.3 rejection of per-expression unsafe); justification strings are true;
no artificial valve inflation or minimization found. Cell-substitutable tags: **none** — every
unsafe region is raw-pointer arithmetic no checked-runtime alternative could replace. The §4.2
relief path therefore does not apply, and the measured M2 breach stands as measured.

## 2026-07-07 — Scheduler port scoring batch

**R14 clarification.** The harness marker is the first line *beginning with* `// Test harness`
(column 0), not any line containing the phrase — prose mentions in header comments do not split.
(Surfaced by the scheduler port's header referencing the marker; the allocator measurement is
unaffected: its marker line was already the first such line.)

**R16 (scheduler).** Splitting spec 2.1's `init() -> Scheduler` into `sched_new()` plus in-place
`sched_init(write s)` is a conforming reading: self-referential sentinel nodes cannot survive a
by-value move in any language with move semantics, and the spec's interface names are indicative,
not binding (spec §2 preamble). — **R17 (scheduler).** Unspecified `admit` error precedence:
E_BAD_PRIO before presence, as ported; recorded as the adjudicated order. — **R18 (scheduler).**
`set_priority` to the same READY level is move-to-tail, the literal reading; the shadow model
agrees and T19 exercises it.

**R19 (scheduler idiomaticity + cell-substitutability).** Scored port confirmed idiomatic:
linkage genuinely embedded (rawptr Link fields, offsetof-based container_of — no owning-container
or index dodge in the measured scheduler; the safe index-based shadow model lives in the excluded
harness, which is its proper place as test oracle); valves concentrated in splice/container_of/
field accessors exactly as design §11.2 predicts; justifications true. Cell-substitutable tags:
**none** — intrusive pointer traversal has no checked-runtime substitute. The measured M2 breach
(0.4120 > 0.40) stands as measured.

**R20 (porting order).** §6.4's hardest-first rule exists so the bet's worst case is confronted
before easy wins manufacture momentum. With both home-ground programs (allocator, scheduler)
completed, scored, and their KILL breaches published, that purpose is fully discharged; the three
value-favorable ports (MMIO in progress, parser, arena) may proceed in parallel. All five are
still attempted and scored; nothing about the order of the remaining three can mask anything.
