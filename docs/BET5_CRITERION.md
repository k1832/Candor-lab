# Bet 5 — Pre-Registered Kill Criterion

**Artifact type:** Scientific pre-registration. This is not a design document and not a pitch.
It fixes, in advance and in the open, the conditions under which Bet 5 is declared **killed**.
It exists to satisfy the non-optional obligation in `LANG_PHYLOSOPHY.md` §3 (Bet 5) and P12:
*"the existence of a pre-registered, published criterion is non-optional"* and it must be
written so it **cannot be gamed or quietly reinterpreted after the fact**.

**Status:** PRE-REGISTERED.
**Version:** 2.
**Date:** 2026-07-06.
**Governing document:** `LANG_PHYLOSOPHY.md` v4 (normative; this document sits at the design-document
tier under §9 and binds the validation, not the philosophy).
**Adjudicating authority:** the single deciding authority defined in `GOVERNANCE.md` (philosophy §9),
currently k1832. Validation may not begin before that authority is named and ratified, because §9
requires a named amender to adjudicate ambiguity and to enact the verdict as an amendment.

---

## 0. Amendment lock and ledger

0.1 **Change lock.** Every threshold, metric definition, basket specification, and decision rule in
this document may be changed **only before the validation prototype's basket ports begin** — where
"begin" is defined precisely in §6.2 as *the first commit of any Candor basket port after the Rust
baselines have been frozen*. After that instant this document is **frozen**; no threshold may be
added, removed, loosened, tightened, or re-normalized.

0.2 **How a pre-freeze change is made.** Any change before the freeze must be recorded as a numbered
row in the ledger at §0.4, stating the old text, the new text, the date, and the rationale, and must
be enacted by the deciding authority (§9 of the philosophy). A change not recorded in the ledger has
no force.

0.3 **After the freeze.** Nothing in this document changes. If the criterion is later found to be
defective, that finding is itself published (§7) and the *philosophy* is amended under §9; the
pre-registration is never silently retrofitted to match a result already seen.

0.4 **Amendment ledger.**

| # | Date | Section | Old | New | Rationale | Enacted by |
|---|------|---------|-----|-----|-----------|------------|
| 1 | 2026-07-06 | §3.1, §3.2, §6.2(i), §8.4 | Candor annotation defined counterfactually ("absent from a hypothetical pure-value version"); classification table co-designed with the ports it measures | Concrete positive enumeration of token/unit classes, authored and hashed at freeze step (i) in a session blind to any port code before any port exists; counterfactual definition dropped; prototype memory-model syntax frozen once the table is hashed | Review finding 1 | deciding authority (k1832), adversarial review #1 |
| 2 | 2026-07-06 | §2.5, §6.5, §6.7 | Every load-bearing subjective judgment routed to a single interested adjudicator with no open-comment protection | Independence-honesty clause (§6.7): solo-project limit stated plainly; blind classification order; §6.5 rulings published for open comment for a minimum 7-day period before taking effect; independently sourced Rust baselines preferred over commissioned; all rulings on the public record | Review finding 2 | deciding authority (k1832), adversarial review #1 |
| 3 | 2026-07-06 | §4.2, §5.1(b), §6.6 | Home-ground M2 KILL was a fixed 0.40 line / 0.50 function ceiling, auto-KILLing a Candor port at parity with idiomatic Rust (incoherent with M3's WARN) | Home-ground M2 KILL made relative: KILL if valve_line_candor > max(0.40, 1.25 × valve_line_rust) per program; Rust baseline valve fractions measured and recorded in §6.6 before the freeze instant; justification reworked for coherence with M3 | Review finding 3 | deciding authority (k1832), adversarial review #1 |
| 4 | 2026-07-06 | §3.1, §3.5, §4.1 | Primary metric was annotation-token fraction over two non-commensurable lexers | Primary metric is annotation units per logical statement (AST-derived); one declared borrow/mode/region relationship = one unit on both languages regardless of token spelling; unit table part of the frozen classification table; raw token fractions demoted to reported-not-gating | Review finding 4 | deciding authority (k1832), adversarial review #1 |
| 5 | 2026-07-06 | §3.2, §3.3, §4.1 | Ambiguous whether borrow markers count at signatures only or also at use sites | Annotation counting is signature/declaration-site only, symmetric on both languages; use-site borrow expressions excluded on both sides; use-site valve tokens still count toward M2's valve regions | Review finding 5 | deciding authority (k1832), adversarial review #1 |
| 6 | 2026-07-06 | §2.1, §3.7, §6.2(i) | Prototype checker soundness never stated as a precondition of measurement validity | New precondition at freeze step (i): the prototype checker must be sound w.r.t. the memory-model core, with a written soundness argument reviewed in an independent session; counts admissible only from a sound checker | Review finding 6 | deciding authority (k1832), adversarial review #1 |
| 7 | 2026-07-06 | §3.4, §3.5, §4.1b, §5 | No unified load metric; the fraction denominator plus P13 created perverse incentives (annotation down / copies up passed both M1 and M4) | New combined metric M1b: combined memory-model load = annotation units + value-copy units per logical statement; KILL if combined_candor > combined_rust; WARN if > 0.85 × combined_rust; explicit P13-reconciliation sentence added | Review finding 7 | deciding authority (k1832), adversarial review #1 |
| 8 | 2026-07-06 | §2.3, §2.5, §6.2 | Freeze order let the team tune specs/tests to advantage; §2.5/§6.2 ordering contradiction | Freeze order reordered: (i) this document + classification table + counting scripts + checker-soundness argument; (ii) language-agnostic specs and suites authored in sessions blind to the Candor design docs, published before baselines are chosen; (iii) Rust baselines confirmed against the frozen specs, frozen by commit hash with valve fractions recorded; then ports. Spec authors may not author Candor ports | Review finding 8 | deciding authority (k1832), adversarial review #1 |
| 9 | 2026-07-06 | §3.6, §4.1, §6.4 | M1 KILL at >1.0× Rust let parity pass; "first passing version is scored" polish asymmetry and unenforceable no-re-rolling rule | M1 KILL moved to > 0.90 × Rust (≥10% reduction required; parity no longer passes), WARN stays > 0.75×; scored artifact on both sides is the adjudicator-confirmed idiomatic port; full public development history of every Candor port replaces the first-passing-version and no-re-rolling rules; hardest-first order and all-five-attempted retained | Review finding 9 | deciding authority (k1832), adversarial review #1 |
| 10 | 2026-07-06 | §4.7, §5.3 | §5.3 counted WARNs from M7, which defines no WARN condition | M7 removed from the WARN tally; supplementary evidence only, feeding no count | Review finding 10 | deciding authority (k1832), adversarial review #1 |
| 11 | 2026-07-06 | §4.6, §5.3 | M6 depended on a spec pack that may not exist at Bet 5 time and was near-certain to fire | M6 demoted to supplementary evidence alongside M7 (no WARN, feeds no counts); "in context" defined as frozen grammar + memory-model design doc + basket specs; §5.3 counts WARNs only from M1, M1b, M2, M3, M4 | Review finding 11 | deciding authority (k1832), adversarial review #1 |
| 12 | 2026-07-06 | §4.2, §5.1(b) | Prototype valve set (unsafe-only) is a strict subset of P12's (checked-runtime OR unsafe); M2's home-ground ceiling could false-KILL on an artifact of the dropped checked-cell valve | Cell-substitutable tagging added: adjudicator-confirmed tags reported; M2 evaluated on the full count, but a KILL that would be reversed by excluding confirmed cell-substitutable regions becomes a mandatory §9 review instead of an automatic KILL | Memory-model adversarial review #1, finding 10 | deciding authority (k1832) |
| 13 | 2026-07-07 | §6.2 step (i), §3.1, §3.7 | Step (i) artifacts existed unhashed | Freeze step (i) declared COMPLETE: checker-soundness argument passed independent retest #4 after three failed rounds (12 findings fixed, docs/reviews/); all step-(i) artifact hashes recorded in docs/FREEZE_MANIFEST.md at commit 0e8c6fe; prototype memory-model syntax frozen per §3.1. Step (ii) spec hashes recorded there too, pending the publication ruling. No metric, threshold, or definition changed | Administrative completion record | deciding authority (k1832) |
| 14 | 2026-07-07 | §6.6 | Baseline table empty | Step (iii) baselines recorded: five Rust baselines (two adapted from independently sourced crates with engines vendored per adjudication R1, three commissioned from Opus-family sessions blind to the Candor design docs), all suites green, frozen at commit b689860 with measured valve fractions; thirteen §6.5 adjudication rulings recorded in docs/ADJUDICATIONS.md with the open-comment window running from repository publication. No metric, threshold, or definition changed | Administrative completion record; adjudication batch R1-R13 | deciding authority (k1832) |

---

## 1. The claim under test (stated narrowly)

1.1 Bet 5 is: **value-first defaults lower cognitive load for real systems code.**

1.2 The **honest delta over Rust** — and the *only* thing this criterion tests — is the **default
gear ordering**: value semantics (ownership transfer and copy) as the default, borrowing as the
explicit second gear, with the pressure valves (checked runtime alternatives, unsafe regions, raw
pointers) as an explicit third. Per philosophy P12 and §3: Rust already has body-local inference and
compact signature defaults; those are **not** the delta and are **not** on trial here.

1.3 Restated as a falsifiable proposition: *On a fixed, adversarially chosen basket of real systems
programs, value-first ordering causes the memory-model pressure valves to remain **rare in
occurrence even where they are critical in function** — such that annotation weight and valve
ambiency are no worse than idiomatic Rust, and are meaningfully lower on the workloads value
semantics is claimed to fit.*

1.4 The bet is **killed** if the valves become **ambient** rather than exceptional (§4.2 defines
"ambient" numerically), or if value-first ordering produces *more* memory-model load than idiomatic
Rust rather than meaningfully less. Killing the bet is enacted as a §9 amendment to P12 and Bet 5,
not a work-around (philosophy P12: *"if they become ambient, the bet has failed and the philosophy
must be amended, not worked around"*).

1.5 **What a positive result does and does not license.** A pass permits the syntax freeze to
proceed (philosophy §3 gates the freeze on this verdict; §8.5 forbids freezing anything the verdict
could invalidate). A pass is **provisional confirmation of the ordering claim on this basket only**;
it is not a claim about performance, ecosystem, or absolute model competence, none of which this
criterion measures.

---

## 2. The validation artifact

2.1 **The prototype.** The memory-model core with **deliberately throwaway syntax**: a checker plus
an interpreter, **no optimizer, no standard library beyond slices** (philosophy §3, "Validation
prototype"). The syntax is disposable; only the memory-model semantics and its default-gear ordering
are on trial. Because there is no optimizer and no runtime, **no performance or benchmark metric is
admissible** (see §8.2). The checker must be **sound with respect to the memory-model core**: a
written soundness argument, reviewed in a session independent of its authoring, is a **precondition
of the freeze** (§3.7, §6.2 step (i)), because a permissive checker mechanically lowers annotation
and valve counts while the code is not actually memory-safe — and "lower load" would then mean only
"the checker did not demand it."

2.2 **The basket.** Five programs, chosen *adversarially* to include the workloads value semantics
fits worst (allocators, intrusive lists, schedulers) alongside the ones it should fit well (parsers,
state machines, arena passes). The basket is fixed; no program may be substituted, dropped, or
"simplified" after the freeze (§6).

2.3 **Same-program discipline.** Each basket program is defined by a **frozen, language-agnostic
functional specification and a frozen test suite** (§0.1), authored in a session blind to the Candor
design docs and published before any baseline is chosen (§6.2 step (ii)). "The program" means *the
artifact that passes that test suite.* A port that does not pass the suite is **not a completed
port** and is scored as an incompletion (§4.5, a KILL). This is the mechanism that makes *"we ported
an easier version"* mechanically detectable: the easier version fails the frozen suite.

2.4 **Basket program specifications.** Each specification below states the **required features**. A
port lacking a required feature is not a valid port of that program.

**(a) General-purpose allocator.**
- Services **arbitrary allocation sizes** with caller-specified **alignment**. A fixed-size pool,
  slab-only, or bump/arena-only allocator does **not** qualify.
- Supports **free** of individually allocated blocks.
- Performs **coalescing of adjacent free blocks** on free — *or* an explicitly named and justified
  equivalent (e.g. segregated free lists with buddy-style splitting/merging) recorded in the port's
  README and confirmed by the adjudicator (§6.5).
- Maintains free-list metadata via raw pointer / address manipulation (in-band or out-of-band).
- **Functional suite:** randomized alloc/free/realign workload with no-overlap, alignment-honored,
  and no-metadata-corruption assertions; must pass.

**(b) Intrusive-list scheduler.**
- Uses **intrusive doubly-linked lists**: the list linkage is **embedded in the scheduled entities**;
  the scheduler does **not** own the entities. An owning container (`Vec<T>`, a node-owning list) or
  an **index-into-an-array** rewrite does **not** qualify — those are exactly the value-friendly
  dodges the basket exists to forbid.
- Supports **O(1) insertion**, **O(1) removal from the middle given a handle to the element**, and a
  scheduling policy (e.g. priority run-queues or round-robin) with enqueue / dequeue / reschedule.
- **Functional suite:** a spawn / block / wake / yield / exit sequence that forces mid-list removal
  and re-insertion; run-order and list-integrity invariants checked.

**(c) Driver-like state machine over MMIO.**
- Models **memory-mapped registers** with volatile-equivalent access: reads and writes that the
  interpreter observes as ordered, non-elidable external effects.
- Implements a multi-state device protocol with **at least five states** (e.g. a UART/SD-like
  init+transfer FSM, or a ring-buffer NIC-like model) driven by completion/interrupt transitions.
- Handles at least one **fault / timeout recovery** path.
- **Functional suite:** a simulated device model drives the FSM through init, transfer, and
  error-recovery; observed MMIO trace checked against the expected trace.

**(d) Parser.**
- A recursive-descent or table-driven parser for a **non-trivial grammar** (e.g. a JSON superset, or
  an expression language with operator precedence) producing a **typed AST**.
- Reports **errors with source positions**.
- Operates over **slices/views of the input** (zero-copy where idiomatic). This is a deliberately
  value-favorable workload.
- **Functional suite:** valid/invalid corpus; AST-equality and error-position checks.

**(e) Arena-based compiler pass.**
- Allocates AST/IR nodes from an **arena** and performs a **transforming pass** (e.g. constant
  folding or an SSA-lite lowering) producing a new IR, with **cross-node references** (arena indices
  or arena pointers).
- Releases the arena **as a whole**.
- **Functional suite:** input IR to transformed IR equality on a fixed corpus.

2.5 **The Rust baseline.** Each of the five must **also exist in idiomatic Rust**, implementing the
**same frozen functional specification** (§2.3), and is **confirmed against that already-frozen
specification** (§6.2 step (iii)) — the specification is frozen before the baseline is chosen, never
fitted to it. The Rust version is the comparison baseline and must be **human-ported or
independently sourced**; an **independently sourced** baseline (e.g. lifted from an existing
open-source Rust allocator/scheduler) is **preferred over one commissioned for this comparison**,
because a commissioned baseline is more exposed to unconscious shaping (§6.7). It **may not be
generated by the same model that wrote the Candor version** (philosophy §3; anti-gaming §6.3).
"Idiomatic" means: passes `clippy` at default lints with no memory-model lint suppressions beyond
ones documented and confirmed by the adjudicator; not artificially annotation-maximized (to flatter
Candor) nor artificially unsafe-maximized (to flatter Rust). Idiomaticity is confirmed and recorded
by the adjudicator under the open-comment discipline of §6.5. The **scored** artifact on *both*
sides is the **adjudicator-confirmed idiomatic port** (§3.6) — the same polish standard applies to
Candor and Rust. This human-ported basket is also Bet 6's external ground truth (philosophy §3,
Bet 6; P19) — a further reason it must be honest.

---

## 3. Definitions used by all metrics

3.1 **The frozen classification / unit table.** Because syntax is throwaway (§2.1), the memory-model
constructs of both languages are compared by **semantic role**, not by spelling. The table is a
**concrete positive enumeration** — of Candor token/unit classes and their Rust counterparts —
**authored and hashed at freeze step (i)** (§6.2), in a session **blind to any Candor port code,
before any port exists**, and published with its hash. There is **no counterfactual definition**: an
earlier draft defined Candor annotation as whatever would be "absent from a hypothetical pure-value
version," which is a modeling judgment made after seeing the code; it is removed (finding 1). Once
the table is hashed, the **prototype's memory-model syntax is frozen** and may not change afterward,
because changing the ruler after building the object is the exact defect this rule closes. For each
class the table fixes (a) the Candor construct(s), (b) the Rust construct(s), and (c) the **counting
unit** (§3.5) they map to. All counts are mechanical and scriptable; the counting scripts are frozen
with this document (§6.2 step (i)).

3.2 **Annotation unit (memory-model annotation).** An annotation unit is a **declared memory-model
relationship at a signature or declaration site**, counted symmetrically on both languages via the
frozen unit table (§3.1, §3.5), so that **one declared relationship is one unit regardless of token
spelling** — `&mut x` in a signature and a one-keyword Candor mode declaration are each **one unit**,
not two-tokens-versus-one (finding 4). The enumerated classes:
- (a) **borrow markers** — a binding/parameter declared a borrow rather than a value (Rust: `&`,
  `&mut` in a signature);
- (b) **lifetime/region-relationship declarations** — one binding's lifetime/region related to
  another's at a signature (Rust: `'a` and lifetime-bearing `where`/bound clauses; **elided
  lifetimes cost zero**, which is correct and fair to Rust);
- (c) **borrow-mutability declarations** distinct from (a) where the grammar separates them;
- (d) **valve-entry declarations** — a declaration-site mention of a pressure valve: an unsafe-region
  entry, a checked-runtime / interior-mutability wrapper *type* named in a declaration (Rust: `Cell`,
  `RefCell`, `UnsafeCell`, `R*<...>` interior-mutable cells), or a raw-pointer *type* in a
  declaration (Rust: `*const`, `*mut`).
- **Counting is signature/declaration-site only, symmetric on both languages** (finding 5). Use-site
  borrow expressions (taking a borrow at a call site) are **excluded on both sides** — P12's delta
  claim is signature-scoped. Use-site *valve* tokens (raw-pointer operations, `unsafe {}` block
  bodies) are **not** M1 annotation but **do** count toward M2's valve regions (§3.3).
- Explicitly **excluded** from annotation: type names, value bindings, ordinary control flow,
  arithmetic-regime keywords (P5 regimes are not the memory model). Ordinary value copies are the
  intended default gear and are **not** annotation — they are counted separately at §3.4.

3.3 **Valve region.** A source region inside a memory-model pressure valve: (i) an unsafe region,
(ii) a use of a checked-runtime / interior-mutability alternative, or (iii) a raw-pointer
manipulation. In Rust the comparable region is `unsafe {}` blocks plus `Cell`/`RefCell`/`UnsafeCell`
uses plus raw-pointer code. Valve regions are measured from **all** valve tokens, **including
use-site ones** (raw-pointer operations, `unsafe {}` block bodies), not only the declaration-site
mentions counted as annotation in §3.2 (finding 5).

3.4 **Value-copy unit.** One explicit ownership copy/clone of the default gear = **one value-copy
unit** on both languages (Rust: `.clone()`, `.to_owned()`; the Candor equivalents), counted via the
frozen unit table. Counted and reported **separately** from annotation, because the bet's thesis is
that value-first *shifts* work from borrow-annotation to value-copies; both columns must be visible
(§4.4), and the two are combined in M1b (§4.1b).

3.5 **Denominators (normalization).**
- **Primary (gating):** *annotation units per logical statement* = annotation units (§3.2) /
  logical statements, where a **logical statement** is an AST-derived node (a declaration or a
  statement), not a physical line — this defeats formatting games and, per finding 4, compares at a
  **shared normalized unit** rather than as a ratio of ratios over two non-commensurable lexers.
  Used for M1 and, with value-copy units added, for M1b.
- **Value-copy load:** *value-copy units per logical statement*, same denominator; used in M1b
  (§4.1b) and reported in M4 (§4.4).
- **Reported, not gating:** *annotation-token fraction* = annotation tokens / total tokens, taken
  from each language's own lexer. Formerly the primary metric; **demoted** (finding 4) because the
  two lexers are non-commensurable (`&mut` is two tokens, a Candor mode keyword is one). Reported for
  continuity, never used for a KILL/WARN.
- **Valve fractions:** *valve-line fraction* = logical lines wholly or partly inside a valve region /
  total logical lines; *valve-function fraction* = functions containing ≥1 valve construct / total
  functions. Used for M2 and M3.

3.6 **The scored port.** For each program the **scored Candor artifact is the adjudicator-confirmed
idiomatic Candor port** (§2.5 applies the same polish standard to both languages), not merely the
first version that passes the frozen suite. To keep this honest without an unenforceable "first
version only" rule, **the full development history of every Candor port is pushed to the public
repository as it happens** (§6.4): the timestamped public record — not a promise not to re-roll — is
what makes selective best-of-many reporting detectable. All five must still be attempted and
completed (§4.5).

3.7 **Admissibility: sound checker.** Annotation, valve, and copy counts are **admissible only from a
checker that is sound with respect to the memory-model core** (§2.1, §6.2 step (i)). A permissive
checker mechanically lowers annotation and valve counts while the code is not actually memory-safe,
so "lower load" would silently mean "the checker did not demand it" (finding 6). A **written
soundness argument, reviewed in a session independent of its authoring**, is a precondition of the
freeze; counts produced by a checker without that reviewed argument are inadmissible.

---

## 4. Pre-registered metrics, thresholds, and procedures

All thresholds are frozen at §0.1. Each is stated as **KILL** (bet fails; enacted as a §9 amendment)
or **WARN** (mandatory review under §9; the authority must produce a recorded ruling).

### 4.1 Metric M1 — annotation load vs. idiomatic Rust (comparative)

**What is measured.** Per program, `A_candor` = annotation units per logical statement (§3.5
primary) of the scored Candor port; `A_rust` = the same measure on the frozen Rust baseline.
Aggregated two ways: `AGG_weighted` (statement-weighted across all five) and `AGG_mean` (unweighted
mean of the five per-program values). The **worse** of the two aggregates is used for the aggregate
rule — this defeats padding the basket's weight with one large easy program.

**Procedure.** Frozen counting script over both ASTs; deterministic; output archived.

**Thresholds.**
- **KILL** if `AGG_candor > 0.90 × AGG_rust` — Candor fails to deliver at least a **10% reduction**
  in annotation load; parity or a marginal edge no longer passes (finding 9).
- **WARN** if `AGG_candor > 0.75 × AGG_rust` (less than a 25% aggregate reduction — the "lower load"
  claim is present but weak, requiring review).

**Justification.** The bet is that value-first ordering is *lower* load than Rust's borrow-first
ordering. "Lower" must mean *meaningfully* lower on a basket that deliberately includes
value-favorable programs, so a **≥10% reduction is the pass floor**: the KILL line sits at
`0.90 × Rust`, and parity — which the v1 draft let survive — does not, because "no worse than Rust"
was never the bet. The 25% WARN band flags a real-but-weak reduction for a §9 look. Thresholds are
**relative to Rust**, sidestepping absolute calibration of a throwaway syntax and testing exactly
the comparative claim. Counting at a **shared normalized unit per logical statement** (§3.5), not a
ratio of raw token fractions over two lexers, removes the tokenization-granularity artifact
(finding 4).

### 4.1b Metric M1b — combined memory-model load vs. idiomatic Rust (comparative)

**What is measured.** Per program, `L_candor` = (annotation units + value-copy units) per logical
statement (§3.5) of the scored Candor port; `L_rust` = the same on the Rust baseline. Aggregated
worse-of-weighted/mean exactly as in M1.

**Thresholds.**
- **KILL** if `AGG_combined_candor > AGG_combined_rust` (Candor's total memory-model load — declared
  aliasing plus value copies — exceeds idiomatic Rust's; the ordering shifts work without lowering
  it).
- **WARN** if `AGG_combined_candor > 0.85 × AGG_combined_rust` (combined load within 15% of Rust —
  the shift-not-reduction failure is close, requiring review).

**Justification.** M1 alone can be "won" by pushing annotation down while value copies climb, and M4
alone can be "won" the other way; the fraction denominator can also be diluted by token padding. M1b
closes both by pricing the **total** memory-model load a reviewer must hold in mind on one
statement-normalized denominator. **P13 reconciliation:** the claim under test is **fewer aliasing
relationships that must be declared, not fewer tokens per se** — P13 prices information per token a
reviewer must read, and an omission is a genuine reduction only if the aliasing information was not
needed; **any annotation the code omits must be one a sound checker (§3.7) did not require**, never
information withheld from the verifier. M1b's KILL at parity encodes that trading annotation for
copies is not a win; the 15% WARN band flags a near-miss.

### 4.2 Metric M2 — pressure-valve ambiency (defines "ambient")

**What is measured.** Per program: valve-line fraction and valve-function fraction (§3.5), measured
from all valve tokens including use-site ones (§3.3). This is the direct operationalization of
philosophy P12/§3: valves must stay *"rare in occurrence even where critical in function."*

**Thresholds — value-favorable programs (parser, MMIO state machine, arena pass), absolute:**
- **KILL** if valve-line fraction > **0.15** OR valve-function fraction > **0.20**.
- **WARN** if valve-line fraction > **0.08** OR valve-function fraction > **0.10**.

**Thresholds — pointer-rich home-ground programs (allocator, intrusive scheduler), relative to the
frozen Rust baseline:**
- **KILL** if `valve_line_candor > max(0.40, 1.25 × valve_line_rust)` on that program, where
  `valve_line_rust` is the baseline's valve-line fraction measured and recorded in §6.6 before the
  freeze instant (finding 3).
- **WARN** if valve-line fraction > **0.25** OR valve-function fraction > **0.35**.

**Justification.** On the workloads value semantics *claims to fit*, valves must be genuinely
exceptional: past ~15% of lines or ~20% of functions the "value-first fits these" premise is
falsified in the plain sense, so those absolute lines are the KILL, with the 8%/10% WARN band
flagging erosion early. On the **home-ground** pointer-rich programs the philosophy *expects* valves
to concentrate ("critical in function"), and §2.4a deliberately requires raw-pointer free-list
manipulation — so idiomatic Rust matching the same spec may itself sit near or above 0.40. A
**fixed** 0.40 KILL would then auto-fail a Candor port that is merely **at parity with idiomatic
Rust**, while M3 treats being *worse* than Rust as only a WARN — the internal incoherence the v1
draft carried, with numbers asserted without calibration. The home-ground KILL is therefore
**relative**: Candor is killed only when it is both **above the absolute 0.40 floor** *and* **more
than 25% above the measured Rust baseline** on that program. This keeps philosophy §3's demand that
valves stay rare *even here* — past ~40% of lines the valve *is* the program and Candor has collapsed
to "unsafe with extra steps" on the ground it names as its own (§1 identity) — while no longer
punishing Candor for the intrinsic pointer density of a workload idiomatic Rust also pays. The
value-favorable thresholds are unchanged.

**Cell-substitutable regions (upper-bound honesty).** The validation prototype ships only the
`unsafe` valve, while P12's valve set also includes checked-runtime alternatives; prototype valve
counts are therefore an **upper bound** on real-Candor valve occurrence (design 0001 §4.3;
memory-model adversarial review #1, finding 10). Guard: port authors may tag any unsafe region for
which a P12 checked-runtime alternative would plausibly have sufficed as **cell-substitutable**;
each tag is confirmed or rejected by the adjudicator with recorded public reasoning under §6.5's
open-comment discipline. M2 is always evaluated on the **full** count. If — and only if — excluding
the adjudicator-confirmed cell-substitutable regions would reverse an M2 KILL verdict, the outcome
is a **mandatory §9 review** (which may still escalate to KILL) rather than an automatic KILL: the
KILL keeps its teeth, and a false KILL caused by the prototype's own scope cut receives a recorded
public ruling instead of silence.

### 4.3 Metric M3 — home-ground valve parity vs. Rust (comparative, pointer-rich only)

**What is measured.** For the allocator and scheduler only: Candor valve-line fraction vs. the Rust
baseline's `unsafe`+interior-mutability+raw-pointer line fraction on the same program (recorded in
§6.6).

**Thresholds.**
- **WARN** if `valve_line_candor > valve_line_rust` on the allocator OR the scheduler (Candor is
  *more* valve-dependent than idiomatic Rust on Candor's home ground).

**Justification.** Home ground is where the bet is hardest and most load-bearing (philosophy §3:
*"the bet's hardest instance is our home ground"*). If Candor needs valves *more* than Rust here, the
value-first ordering is actively counterproductive where it matters most. It is a WARN rather than a
KILL because M2's home-ground **relative** ceiling — `max(0.40, 1.25 × valve_line_rust)` (§4.2) —
already provides the hard KILL floor, and measurement noise on a throwaway prototype could otherwise
turn a small parity breach into a false KILL. The two are now coherent: being *slightly* worse than
Rust is a WARN (M3); being **more than 25% worse and above 0.40** is a KILL (M2). Per §5, either the
allocator or the scheduler triggering *any* WARN is already a mandatory review.

### 4.4 Metric M4 — value-copy blow-up (absolute; guards the other failure direction)

**What is measured.** Per program: Candor value-copy units per logical statement vs. Rust value-copy
units per logical statement (§3.4, §3.5), aggregated.

**Thresholds.**
- **WARN** if `copy_candor > 2.0 × copy_rust` on the aggregate.

**Justification.** Value-first could "win" M1/M2 by trading annotation for pervasive deep copies a
human would never write — ergonomic on paper, absurd in practice, and a hidden-cost violation of
predictable-cost intent. M1b (§4.1b) now catches the *combined* shift-without-reduction failure as a
KILL/WARN; M4 remains the **copy-specific** WARN that flags a degenerate copy blow-up even when
combined load still passes. Non-KILL because copies are the intended default gear and some increase
is expected and legitimate; a >2× aggregate blow-up is the point at which it stops being a gear and
starts being a smell.

### 4.5 Metric M5 — completability (absolute gate)

**What is measured.** Whether each of the five reaches a passing functional suite (§2.3) in Candor.

**Threshold.**
- **KILL** if **any** basket program cannot be completed in Candor at all.

**Justification.** Philosophy requirement, restated in §6.4: an incompletable program is *itself* the
strongest possible falsification of "value-first fits real systems code," not an excuse to shrink the
basket. There is no WARN tier; incompletion is unconditional KILL.

### 4.6 Metric M6 — model repair friction (SUPPLEMENTARY, non-gating)

**What is measured.** For each program, N models (N and the model set fixed at §6.2) port it with the
spec pack **in context** — defined as the **frozen grammar + the memory-model design doc + the basket
specs** (finding 11) — under **equal adaptation budget** (equal in-context tokens, equal repair
iterations, equal tool feedback). Measured: mean **valve/borrow-related** repair iterations to first
passing port — Candor (valve-related) vs. Rust (borrow/lifetime-related).

**Status.** **Supplementary evidence only** — reported and admissible in a §9 review, but it
**defines no WARN condition and feeds no count in §5.3** (finding 11). Two reasons: (i) philosophy
P19 is explicit that raw model comparisons against a Rust-trained model **confound design with corpus
scale** and "will read as failure for years regardless of merit," so a Candor-vs-Rust model
comparison could never, alone, gate the bet without violating P19 and measuring corpus rather than
ordering; (ii) the required spec pack may not exist at Bet 5 validation time (that is Bet 6), so a
gated metric here is near-certain to fire spuriously. It is retained as an internal friction signal,
not a threshold — flagged for the authority as one of the least-certain areas of this document.

### 4.7 Metric M7 — human task-completion (OPTIONAL, supplementary, non-gating)

If run, a small-N controlled study on a fixed task measuring completion time and correctness;
reported and admissible as review evidence, never a KILL and **defining no WARN — it feeds no count
in §5.3** (finding 10). **Optional** because §8.5 sequencing prioritizes a fast Bet 5 verdict and a
properly powered human study is slow and expensive; it is a tiebreaker, not a gate.

---

## 5. Decision rule

5.1 **KILL (bet failed; §9 amendment mandatory).** The bet is killed if **any** of the following
holds:
- (a) any basket program is not completed in Candor (M5);
- (b) any **per-program** valve KILL threshold in M2 is breached — for value-favorable programs the
  absolute 0.15 line / 0.20 function lines; for the allocator or scheduler the relative ceiling
  `valve_line_candor > max(0.40, 1.25 × valve_line_rust)` (§4.2). These are **absolute per-program
  floors** that strong results elsewhere **cannot** offset — subject only to the cell-substitutable
  review path of §4.2 (if excluding adjudicator-confirmed cell-substitutable regions would reverse
  an M2 KILL, the verdict is mandatory §9 review, not automatic KILL);
- (c) the aggregate annotation KILL threshold in M1 is breached — `AGG_candor > 0.90 × AGG_rust`,
  using the worse of weighted/mean (§4.1);
- (d) the aggregate combined-load KILL threshold in M1b is breached —
  `AGG_combined_candor > AGG_combined_rust`, worse of weighted/mean (§4.1b).

5.2 **Anti-masking rule.** Because the basket was chosen adversarially and pointer-rich code is the
bet's hardest instance, the allocator and scheduler are governed by **independent per-program KILL
floors** (M2, M5). The aggregate metrics (M1, M1b) are computed both statement-weighted and
unweighted and the **worse is used**, so a large easy program cannot dilute a hard failure. No
averaging, no basket subsetting, no "4-of-5" pass.

5.3 **Mandatory review (§9), no KILL.** If no KILL fires, count WARN triggers across **M1, M1b, M2,
M3, M4 only** — M6 and M7 are supplementary and feed no count (§4.6, §4.7; findings 10–11):
- **Any** WARN on the **allocator** or the **scheduler** (any of M1, M1b, M2, M3, M4) → **mandatory
  §9 review** (home-ground sensitivity; philosophy §3).
- **Two or more** WARN triggers in total (any programs, any of the five gating metrics) → **mandatory
  §9 review**.
- A mandatory review **cannot silently pass**: the deciding authority must produce a recorded ruling
  in the §0.4 ledger — *proceed*, *re-scope the design*, or *escalate to KILL* — with its reasoning
  and any dissent (philosophy §9's open-comment discipline for consequential decisions; §6.5).

5.4 **Provisional confirmation.** If no KILL fires, fewer than two WARNs fire in total across the five
gating metrics, and neither the allocator nor the scheduler triggers any WARN, Bet 5 is
**PROVISIONALLY CONFIRMED on this basket**. The result is recorded, published (§7), and the syntax
freeze may proceed (philosophy §3, §8.5). "Provisional" and "on this basket" are load-bearing: the
pass is not a general claim (§1.5).

---

## 6. Anti-gaming provisions

6.1 **No post-hoc metric changes.** After the freeze (§0.1) no metric, threshold, definition,
denominator, or basket program may be added, removed, loosened, tightened, or re-normalized. A
defect found after the freeze is published, not patched into the criterion (§0.3).

6.2 **Freeze order and the freeze instant.** In order:
- **(i)** this document is ratified at v2, and the **frozen classification / unit table (§3.1), the
  counting/measurement scripts, the M6 model set, and the written checker-soundness argument (§3.7)**
  are committed and hashed — the table authored in a session **blind to any port code, before any
  port exists** (findings 1, 6);
- **(ii)** the **language-agnostic functional specifications and test suites** are authored in
  **sessions blind to the Candor design docs**, committed, hashed, and **published before any Rust
  baseline is chosen** (finding 8);
- **(iii)** the five **Rust baselines** are finalized, **confirmed against the already-frozen specs**
  (§2.5), **frozen by commit hash**, and their **measured valve fractions recorded in §6.6**
  (findings 3, 8).

Only then may Candor ports begin. The **freeze instant** is the first commit of any Candor basket
port after (iii). All hashes and baseline valve fractions are recorded in this document before that
instant. **Spec authors may not author Candor ports** — session-blindness is the solo-project
approximation of author independence (finding 8; §6.7). This ordering also fixes the v1
contradiction in which §2.5 baselines implemented a spec §6.2 froze *later*.

6.3 **Baseline independence.** Each Rust baseline is human-ported or independently sourced and **may
not be produced by the same model that authors the corresponding Candor port** (philosophy §3).
Baselines are frozen at §6.2 step (iii) before any Candor port begins, so no baseline can be
retrofitted to flatter a Candor result.

6.4 **Porting order and the public development record.** Candor ports are attempted in
**hardest-first** order: **allocator, scheduler, MMIO state machine, parser, arena pass.**
Hardest-first means the bet's worst case is confronted first and an incompletion is discovered early
rather than after easy wins manufacture momentum. **All five must be attempted**; abandoning any is a
KILL (M5), never a reason to shrink the basket. The **scored** port on each side is the
**adjudicator-confirmed idiomatic port** (§2.5, §3.6). The v1 "only the first passing version is
scored / no re-rolling" rule is **withdrawn as unenforceable** and replaced by a **public-record
rule: the full development history of every Candor port is pushed to the public repository as it
happens** (finding 9) — the timestamped public trail, not an unverifiable promise, is what makes
selective best-of-many reporting detectable. All attempts remain archived.

6.5 **Adjudication of ambiguity, under an open-comment discipline.** Where a classification is
genuinely ambiguous — is a construct an annotation unit (§3.2)? is a coalescing equivalent acceptable
(§2.4a)? is the Rust baseline idiomatic (§2.5)? — the **deciding authority defined in `GOVERNANCE.md`
(philosophy §9)** rules, but under the discipline of §6.7: **classification is performed blind**
(both languages are classified before the which-side-benefits mapping is computed), and every
**ruling is published for open comment for a minimum of 7 days before it takes effect**, with the
ruling and any objections recorded in the §0.4 ledger so the adjudication trail is itself auditable.
**Dependency:** validation may not begin until that authority is named and ratified; `GOVERNANCE.md`
names the authority. Without a named amender there is no one to adjudicate or to enact the verdict as
an amendment.

6.6 **Frozen Rust baselines (recorded before the freeze instant).** The allocator and scheduler
valve-line fractions recorded here are the `valve_line_rust` values used in M2's home-ground relative
KILL (§4.2) and in M3 (§4.3).

| Program | Source (repo / author) | Commit hash | Valve-line fraction (measured) | Valve-function fraction (measured) | Idiomaticity confirmed by | Date |
|---------|------------------------|-------------|--------------------------------|------------------------------------|---------------------------|------|
| Allocator | adapted: linked_list_allocator 0.10.6 (rust-osdev; engine vendored per ruling R1) + commissioned adaptation, Opus-family session | `b689860` (this repo, baselines/rust/allocator) | 0.1929 | 0.6842 | deciding authority (k1832); §6.5 comment window runs from publication | 2026-07-07 |
| Intrusive scheduler | adapted: intrusive-collections 0.10.2 (Amanieu; used subset vendored per R1) + commissioned policy layer, Opus-family session | `b689860` (baselines/rust/scheduler) | 0.1489 | 0.5000 | same | 2026-07-07 |
| MMIO state machine | commissioned, Opus-family session blind to Candor design docs | `b689860` (baselines/rust/mmio) | 0.0231 | 0.1154 | same | 2026-07-07 |
| Parser | commissioned, Opus-family session blind to Candor design docs | `b689860` (baselines/rust/parser) | 0.0000 | 0.0000 | same | 2026-07-07 |
| Arena compiler pass | commissioned, Opus-family session blind to Candor design docs | `b689860` (baselines/rust/arena) | 0.0000 | 0.0000 | same | 2026-07-07 |

Measurement records: `docs/measurements/baselines/*.json` (per-file rust-count output, additive
aggregation over each baseline's `src/`; tests excluded). Resulting home-ground M2 KILL ceilings
(§4.2): allocator `max(0.40, 1.25 × 0.1929) = 0.40`; scheduler `max(0.40, 1.25 × 0.1489) = 0.40`.
Per the baseline-production ruling (FREEZE_MANIFEST): baselines were authored by Opus-family
sessions, so the Candor ports may not be (criterion §6.3); the port-authoring model family is
recorded here when porting begins. Spec-ambiguity rulings from baseline construction:
`docs/ADJUDICATIONS.md` R1-R13.

6.7 **Independence: the solo-project honesty clause.** Full third-party independence — separate teams
for spec-authoring, baseline sourcing, classification, and adjudication — is **not available to a
solo project, and this document does not claim it.** Stated plainly instead: one authority both holds
the bet and adjudicates it. The **available mitigations**, all adopted above, are: (a)
**session-blindness** as the approximation of author independence — the classification/unit table is
authored blind to port code (§3.1, §6.2 i), the specs are authored blind to the Candor design docs
(§6.2 ii), and spec authors may not author ports (§6.2); (b) **blind classification order** — both
languages are classified before the which-side-benefits mapping is computed (§6.5); (c) **§6.5
rulings published for open comment** for a minimum 7-day period before taking effect, with objections
recorded; (d) **independently sourced Rust baselines preferred** over commissioned ones (§2.5); (e)
**all rulings and the full port development history on the public record** (§6.4). These reduce, but
do not eliminate, the conflict; naming the residual honestly is itself required by the philosophy's
warning that admitted costs are not managed costs (`LANG_PHYLOSOPHY.md` header).

---

## 7. Publication commitment

7.1 The result is **published either way** — pass or kill — with all archived ports, baseline hashes,
baseline valve fractions, raw counting-script output, and the §0.4 ledger (philosophy §3, §7: *"its
results are published either way"*).

7.2 The verdict is **enacted as a §9 amendment**:
- **KILL:** an amendment to the philosophy naming Bet 5 and P12, stating the evidence, and appended
  to Appendix A's ledger; missed pre-registered targets are enacted as amendments, not absorbed as
  silence (philosophy §9).
- **PROVISIONAL CONFIRMATION:** recorded as the verdict that unblocks the syntax freeze (philosophy
  §3, §8.5), with the "provisional / this-basket-only" scope preserved.
- **MANDATORY REVIEW:** the authority's recorded ruling (§5.3) is the enacted outcome.

---

## 8. Explicitly rejected alternatives (rationale is the product, §8.6)

8.1 **Absolute Candor-vs-Rust model-correctness comparison as a KILL** — *rejected.* P19 states raw
model comparisons confound design with corpus scale and "will read as failure for years regardless of
merit"; absolute parity is "not a launch criterion." Killing Bet 5 on a metric that principally
measures the absent corpus would falsify the wrong thing. Retained only as supplementary, non-gating
friction evidence (M6).

8.2 **Runtime/benchmark performance** — *rejected.* The bet is about **cognitive load**, and the
validation prototype has **no optimizer and no runtime** (§2.1); any speed number would measure the
interpreter, not the language. Performance is a different priority (philosophy Priority 4, Bet 3) with
its own honesty (Refusals: loses benchmarks against UB-exploiting C++).

8.3 **Total lines of code, or total token count, as the proxy** — *rejected.* Conflates verbosity
with annotation burden and contradicts P13, which prices **information per token a reviewer must
read**, not fewer tokens. A terser program with denser annotation can be *harder* to verify. We count
**annotation units per logical statement** (§3.5), not size.

8.4 **Raw keyword-frequency counting** — *rejected.* Throwaway syntax (§2.1) makes spelling
meaningless; two prototypes could score differently on identical semantics. We count by **semantic
unit class** via a frozen **positive-enumeration** classification / unit table (§3.1–3.2), authored
blind before any port exists — not by a counterfactual applied after seeing the code.

8.5 **Subjective readability surveys as the primary metric** — *rejected as primary.* Not mechanical,
slow, and gameable through framing. Kept only as optional supplementary evidence (M7).

8.6 **Compile-time or borrow-check-error counts** — *rejected.* Those measure the toolchain and the
iteration process, not the final artifact's verification cost, and belong to P20's separate
pre-registered targets. The scored artifact is the **adjudicator-confirmed idiomatic port** (§3.6),
not the path to it.

8.7 **Self-generated tests grading self-generated code** — *rejected.* Circular; measures internal
consistency, not correctness or idiom (philosophy P19, Bet 6). The frozen functional suites (§2.3) and
independently sourced Rust baselines (§2.5, §6.3) are the external anchors that make the numbers mean
something.

8.8 **A "4-of-5 programs pass" or basket-average rule** — *rejected.* The basket was chosen
adversarially; an averaging rule lets the three easy programs mask failure on the allocator and
scheduler, which are the bet's hardest instance and Candor's home ground. Replaced by per-program KILL
floors and the worse-of-weighted/mean aggregate (§5.1–5.2).
