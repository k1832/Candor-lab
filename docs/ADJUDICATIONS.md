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
