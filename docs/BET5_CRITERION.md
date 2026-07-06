# Bet 5 — Pre-Registered Kill Criterion

**Artifact type:** Scientific pre-registration. This is not a design document and not a pitch.
It fixes, in advance and in the open, the conditions under which Bet 5 is declared **killed**.
It exists to satisfy the non-optional obligation in `LANG_PHYLOSOPHY.md` §3 (Bet 5) and P12:
*"the existence of a pre-registered, published criterion is non-optional"* and it must be
written so it **cannot be gamed or quietly reinterpreted after the fact**.

**Status:** PRE-REGISTERED.
**Version:** 1.
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
| — | — | — | — | (none yet — document at v1 as first written) | — | — |

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
"ambient" numerically), or if value-first ordering produces *more* annotation weight than idiomatic
Rust rather than less. Killing the bet is enacted as a §9 amendment to P12 and Bet 5, not a
work-around (philosophy P12: *"if they become ambient, the bet has failed and the philosophy must be
amended, not worked around"*).

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
admissible** (see §8.2).

2.2 **The basket.** Five programs, chosen *adversarially* to include the workloads value semantics
fits worst (allocators, intrusive lists, schedulers) alongside the ones it should fit well (parsers,
state machines, arena passes). The basket is fixed; no program may be substituted, dropped, or
"simplified" after the freeze (§6).

2.3 **Same-program discipline.** Each basket program is defined by a **frozen, language-agnostic
functional specification and a frozen test suite** (§0.1). "The program" means *the artifact that
passes that test suite.* A port that does not pass the suite is **not a completed port** and is
scored as an incompletion (§4.5, a KILL). This is the mechanism that makes *"we ported an easier
version"* mechanically detectable: the easier version fails the frozen suite.

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
**same frozen functional specification** (§2.3). The Rust version is the comparison baseline and must
be **human-ported or independently sourced** (e.g. lifted from an existing open-source Rust
allocator/scheduler). It **may not be generated by the same model that wrote the Candor version**
(philosophy §3; anti-gaming §6.3). "Idiomatic" means: passes `clippy` at default lints with no
memory-model lint suppressions beyond ones documented and confirmed by the adjudicator; not
artificially annotation-maximized (to flatter Candor) nor artificially unsafe-maximized (to flatter
Rust). Idiomaticity is confirmed and recorded by the adjudicator (§6.5). This human-ported basket is
also Bet 6's external ground truth (philosophy §3, Bet 6; P19) — a further reason it must be honest.

---

## 3. Definitions used by all metrics

3.1 **Token stream.** All token counts are taken from the **prototype's own lexer** (Candor) and from
a Rust token classifier over the `rustc`/`syn` token stream (Rust). Counting is **mechanical and
scriptable**; the counting scripts are frozen with this document (§6.2). Because syntax is throwaway
(§2.1), tokens are classified by **semantic role**, not by spelling, via a frozen classification
table.

3.2 **Annotation token (memory-model annotation).** A token whose *sole purpose* is to declare
aliasing, ownership, lifetime, or valve semantics — i.e. a token that would be **absent from a
hypothetical pure-value version** of the same code that names the same data and performs the same
operations. Counted classes:
- (a) **borrow markers** — tokens marking a binding/parameter as a borrow rather than a value
  (Rust: `&`, `&mut`);
- (b) **lifetime/region-relationship tokens** — tokens relating one binding's lifetime/region to
  another's (Rust: `'a` and lifetime-bearing `where`/bound clauses; **elided lifetimes cost zero**,
  which is correct and fair to Rust);
- (c) **borrow-mutability markers** distinct from (a) where the grammar separates them;
- (d) **valve-entry tokens** — unsafe-region markers; checked-runtime / interior-mutability wrapper
  *type mentions* (Rust: `Cell`, `RefCell`, `R*<...>` interior-mutable cells, `UnsafeCell`); raw
  pointer type tokens and raw pointer operations (Rust: `*const`, `*mut`, `.as_ptr`, `ptr::*`).
- Explicitly **excluded** from annotation: type names, value bindings, ordinary control flow,
  arithmetic-regime keywords (P5 regimes are not the memory model). Ordinary value copies are the
  intended default gear and are **not** annotation — they are counted separately at §3.4.

3.3 **Valve region.** A source region inside a memory-model pressure valve: (i) an unsafe region,
(ii) a use of a checked-runtime / interior-mutability alternative, or (iii) a raw-pointer
manipulation. In Rust the comparable region is `unsafe {}` blocks plus `Cell`/`RefCell`/`UnsafeCell`
uses plus raw-pointer code.

3.4 **Value-copy token.** A token that performs an explicit ownership copy/clone of the default gear
(Rust: `.clone()`, `.to_owned()`; the Candor equivalents). Counted and reported **separately** from
annotation, because the bet's thesis is that value-first *shifts* work from borrow-annotation to
value-copies; both columns must be visible (§4.4).

3.5 **Denominators (normalization).**
- **Primary:** *annotation-token fraction* = annotation tokens / total tokens. Formatting-independent,
  aligned with P13 ("information per token a reviewer must read"). Used for all annotation KILL/WARN
  rules.
- **Secondary (reported, not gating):** annotation tokens per 1000 logical lines (declarations and
  statements, counted from the parser's AST — not physical lines, to defeat formatting games).
- **Valve fractions:** *valve-line fraction* = logical lines wholly or partly inside a valve region /
  total logical lines; *valve-function fraction* = functions containing ≥1 valve construct / total
  functions.

3.6 **The measured port.** For each program, **the port that is scored is the first Candor version
that passes the frozen functional suite** (§2.3). All prior attempts are recorded but not scored;
you may not port five times and select the best (§6.4).

---

## 4. Pre-registered metrics, thresholds, and procedures

All thresholds are frozen at §0.1. Each is stated as **KILL** (bet fails; enacted as a §9 amendment)
or **WARN** (mandatory review under §9; the authority must produce a recorded ruling).

### 4.1 Metric M1 — annotation density vs. idiomatic Rust (comparative)

**What is measured.** Per program, `A_candor` = annotation-token fraction (§3.5 primary) of the
Candor port; `A_rust` = the same fraction of the frozen Rust baseline. Aggregated two ways:
`AGG_weighted` (token-weighted across all five) and `AGG_mean` (unweighted mean of the five
per-program fractions). The **worse** of the two aggregates is used for the aggregate rule — this
defeats padding the basket's weight with one large easy program.

**Procedure.** Frozen counting script over both token streams; deterministic; output archived.

**Thresholds.**
- **KILL** if `AGG_candor > AGG_rust` (Candor requires *more* memory-model annotation overall than
  idiomatic Rust — the ordering delivers no relief; the bet's core claim is false).
- **WARN** if `AGG_candor > 0.75 × AGG_rust` (less than a 25% aggregate reduction — the "lower load"
  claim is present but weak, requiring review).

**Justification.** The bet is that value-first ordering is *lower* load than Rust's borrow-first
ordering. If Candor needs *more* annotation, the claim is simply wrong, so equality-or-worse is the
KILL line. The 25% WARN band encodes that "lower" should be *meaningfully* lower on a basket that
deliberately includes value-favorable programs; a marginal edge over Rust is not the win the bet
promises and merits a §9 look. Thresholds are **relative to Rust**, which sidesteps needing an
absolute calibration for a throwaway syntax and tests exactly the comparative claim.

### 4.2 Metric M2 — pressure-valve ambiency (absolute; defines "ambient")

**What is measured.** Per program: valve-line fraction and valve-function fraction (§3.5). This is
the direct operationalization of philosophy P12/§3: valves must stay *"rare in occurrence even where
critical in function."* "Occurrence" is measured as these two fractions.

**Thresholds — value-favorable programs (parser, MMIO state machine, arena pass):**
- **KILL** if valve-line fraction > **0.15** OR valve-function fraction > **0.20**.
- **WARN** if valve-line fraction > **0.08** OR valve-function fraction > **0.10**.

**Thresholds — pointer-rich home-ground programs (allocator, intrusive scheduler):**
- **KILL** if valve-line fraction > **0.40** OR valve-function fraction > **0.50**.
- **WARN** if valve-line fraction > **0.25** OR valve-function fraction > **0.35**.

**Justification.** On the workloads value semantics *claims to fit*, valves must be genuinely
exceptional: past ~15% of lines or ~20% of functions the "value-first fits these" premise is
falsified in the plain sense, so that is the KILL line; the 8%/10% WARN band flags erosion before it
becomes failure. On the home-ground pointer-rich programs the philosophy *expects* valves to
concentrate ("critical in function"), so the KILL line is looser — but there is a ceiling:
philosophy §3 says the valves must stay rare *even here*. If **more than 40% of lines** in the
hardest program sit inside valves, the safe value-first substrate is no longer carrying the program —
the valve *is* the program, and Candor has collapsed to "unsafe with extra steps" precisely on the
ground it names as its own (§1 identity). That is a KILL, not a caveat.

### 4.3 Metric M3 — home-ground valve parity vs. Rust (comparative, pointer-rich only)

**What is measured.** For the allocator and scheduler only: Candor valve-line fraction vs. the Rust
baseline's `unsafe`+interior-mutability+raw-pointer line fraction on the same program.

**Thresholds.**
- **WARN** if `valve_line_candor > valve_line_rust` on the allocator OR the scheduler (Candor is
  *more* valve-dependent than idiomatic Rust on Candor's home ground).

**Justification.** Home ground is where the bet is hardest and most load-bearing (philosophy §3:
*"the bet's hardest instance is our home ground"*). If Candor needs valves *more* than Rust here, the
value-first ordering is actively counterproductive where it matters most. It is a WARN rather than a
KILL because M2's absolute 0.40 ceiling already provides the hard floor and measurement noise on a
throwaway prototype could otherwise produce a false KILL; but any breach forces the authority to rule
(and, per §5, either allocator or scheduler triggering *any* WARN is already a mandatory review).

### 4.4 Metric M4 — value-copy blow-up (absolute; guards the other failure direction)

**What is measured.** Per program: Candor value-copy-token fraction vs. Rust value-copy-token
fraction (§3.4).

**Thresholds.**
- **WARN** if `copy_candor > 2.0 × copy_rust` on the aggregate.

**Justification.** Value-first could "win" M1/M2 by trading annotation for pervasive deep copies a
human would never write — ergonomic on paper, absurd in practice, and a hidden-cost violation of
predictable-cost intent. This WARN catches that degenerate win. Non-KILL because copies are the
intended default gear and some increase is expected and legitimate; a >2x aggregate blow-up is the
point at which it stops being a gear and starts being a smell.

### 4.5 Metric M5 — completability (absolute gate)

**What is measured.** Whether each of the five reaches a passing functional suite (§2.3) in Candor.

**Threshold.**
- **KILL** if **any** basket program cannot be completed in Candor at all.

**Justification.** Philosophy requirement, restated in §6.4: an incompletable program is *itself* the
strongest possible falsification of "value-first fits real systems code," not an excuse to shrink the
basket. There is no WARN tier; incompletion is unconditional KILL.

### 4.6 Metric M6 — model repair friction (WARN-only, P19-bounded)

**What is measured.** For each program, N models (N and the model set fixed at §6.2) port it with the
spec pack in context under **equal adaptation budget** (equal in-context tokens, equal repair
iterations, equal tool feedback). Measured: mean **valve/borrow-related** repair iterations to first
passing port — Candor (valve-related) vs. Rust (borrow/lifetime-related).

**Thresholds.**
- **WARN** if aggregate Candor valve-repair iterations > aggregate Rust borrow-repair iterations.

**Justification and the deliberate non-KILL.** Philosophy §3 lists model correctness as a candidate
Bet 5 metric, but P19 is explicit that **raw model comparisons against a Rust-trained model confound
design with corpus scale** and "will read as failure for years regardless of merit," and that
"absolute parity with Rust-trained models is a long-term consequence, not a launch criterion." At
Bet 5 validation time there is no adapted Candor model and no corpus (that is Bet 6). Making a
Candor-vs-Rust model comparison a KILL would therefore violate P19 and measure corpus, not ordering.
It is retained as a **WARN-only friction signal** — an internal ratio of memory-model-related repairs
— that can trigger review but can never, alone, kill the bet. This choice is flagged for the
authority as one of the least-certain calls in this document.

### 4.7 Metric M7 — human task-completion (OPTIONAL, supplementary, non-KILL)

If run, a small-N controlled study on a fixed task measuring completion time and correctness;
reported and admissible as review evidence, never a KILL. **Optional** because §8.5 sequencing
prioritizes a fast Bet 5 verdict and a properly powered human study is slow and expensive; it is a
tiebreaker, not a gate.

---

## 5. Decision rule

5.1 **KILL (bet failed; §9 amendment mandatory).** The bet is killed if **any** of the following
holds:
- (a) any basket program is not completed in Candor (M5);
- (b) any **per-program** valve KILL threshold in M2 is breached (including, and especially, the
  allocator's or scheduler's 0.40 line / 0.50 function ceiling — these are **absolute per-program
  floors** that strong results elsewhere **cannot** offset);
- (c) the aggregate annotation KILL threshold in M1 is breached (using the worse of weighted/mean).

5.2 **Anti-masking rule.** Because the basket was chosen adversarially and pointer-rich code is the
bet's hardest instance, the allocator and scheduler are governed by **independent per-program KILL
floors** (M2, M5). The aggregate metrics (M1) are computed both token-weighted and unweighted and the
**worse is used**, so a large easy program cannot dilute a hard failure. No averaging, no basket
subsetting, no "4-of-5" pass.

5.3 **Mandatory review (§9), no KILL.** If no KILL fires, count WARN triggers across M1, M2, M3, M4,
M6, M7:
- **Any** WARN on the **allocator** or the **scheduler** (any metric) → **mandatory §9 review**
  (home-ground sensitivity; philosophy §3).
- **Two or more** WARN triggers in total (any programs, any metrics) → **mandatory §9 review**.
- A mandatory review **cannot silently pass**: the deciding authority must produce a recorded ruling
  in the §0.4 ledger — *proceed*, *re-scope the design*, or *escalate to KILL* — with its reasoning
  and any dissent (philosophy §9's open-comment discipline for consequential decisions).

5.4 **Provisional confirmation.** If no KILL fires, fewer than two WARNs fire in total, and neither
the allocator nor the scheduler triggers any WARN, Bet 5 is **PROVISIONALLY CONFIRMED on this
basket**. The result is recorded, published (§7), and the syntax freeze may proceed (philosophy §3,
§8.5). "Provisional" and "on this basket" are load-bearing: the pass is not a general claim (§1.5).

---

## 6. Anti-gaming provisions

6.1 **No post-hoc metric changes.** After the freeze (§0.1) no metric, threshold, definition,
denominator, or basket program may be added, removed, loosened, tightened, or re-normalized. A
defect found after the freeze is published, not patched into the criterion (§0.3).

6.2 **Freeze order and the freeze instant.** In order: (i) this document is ratified at v1 and the
counting/measurement scripts and the M6 model set are committed and hashed; (ii) the five Rust
baselines are finalized and **frozen by commit hash**, recorded in §6.6; (iii) the frozen
language-agnostic functional specs and test suites are committed and hashed. Only then may Candor
ports begin. The **freeze instant** is the first commit of any Candor basket port after (ii). All
hashes are recorded in this document before that instant.

6.3 **Baseline independence.** Each Rust baseline is human-ported or independently sourced and **may
not be produced by the same model that authors the corresponding Candor port** (philosophy §3).
Baselines are frozen (§6.2) before any Candor port begins, so no baseline can be retrofitted to
flatter a Candor result.

6.4 **Porting order and no re-rolling.** Candor ports are attempted in **hardest-first** order:
**allocator, scheduler, MMIO state machine, parser, arena pass.** Hardest-first means the bet's
worst case is confronted first and an incompletion is discovered early rather than after easy wins
manufacture momentum. **All five must be attempted**; abandoning any is a KILL (M5), never a reason
to shrink the basket. The **scored** port is the first version to pass the frozen suite (§3.6); all
attempts are archived; selecting a best-of-many is forbidden.

6.5 **Adjudication of ambiguity.** Where a classification is genuinely ambiguous — is a token
annotation (§3.2)? is a coalescing equivalent acceptable (§2.4a)? is the Rust baseline idiomatic
(§2.5)? — the **deciding authority defined in `GOVERNANCE.md` (philosophy §9)** rules. Every such
ambiguity and its ruling is recorded in the §0.4 ledger with reasoning, so the adjudication trail is
itself auditable. **Dependency:** validation may not begin until that authority is named and
ratified; `GOVERNANCE.md` names the authority. Without a named amender there is no one to
adjudicate or to enact the verdict as an amendment.

6.6 **Frozen Rust baselines (recorded before the freeze instant).**

| Program | Source (repo / author) | Commit hash | Idiomaticity confirmed by | Date |
|---------|------------------------|-------------|---------------------------|------|
| Allocator | *(to be recorded before freeze)* | — | — | — |
| Intrusive scheduler | — | — | — | — |
| MMIO state machine | — | — | — | — |
| Parser | — | — | — | — |
| Arena compiler pass | — | — | — | — |

---

## 7. Publication commitment

7.1 The result is **published either way** — pass or kill — with all archived ports, baseline hashes,
raw counting-script output, and the §0.4 ledger (philosophy §3, §7: *"its results are published
either way"*).

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
measures the absent corpus would falsify the wrong thing. Retained only as WARN-only friction (M6).

8.2 **Runtime/benchmark performance** — *rejected.* The bet is about **cognitive load**, and the
validation prototype has **no optimizer and no runtime** (§2.1); any speed number would measure the
interpreter, not the language. Performance is a different priority (philosophy Priority 4, Bet 3) with
its own honesty (Refusals: loses benchmarks against UB-exploiting C++).

8.3 **Total lines of code, or total token count, as the proxy** — *rejected.* Conflates verbosity
with annotation burden and contradicts P13, which prices **information per token a reviewer must
read**, not fewer tokens. A terser program with denser annotation can be *harder* to verify. We count
annotation *fraction*, not size.

8.4 **Raw keyword-frequency counting** — *rejected.* Throwaway syntax (§2.1) makes spelling
meaningless; two prototypes could score differently on identical semantics. We count by **semantic
token class** via a frozen classification table (§3.1–3.2).

8.5 **Subjective readability surveys as the primary metric** — *rejected as primary.* Not mechanical,
slow, and gameable through framing. Kept only as optional supplementary evidence (M7).

8.6 **Compile-time or borrow-check-error counts** — *rejected.* Those measure the toolchain and the
iteration process, not the final artifact's verification cost, and belong to P20's separate
pre-registered targets. The scored artifact is the *completed* port (§3.6), not the path to it.

8.7 **Self-generated tests grading self-generated code** — *rejected.* Circular; measures internal
consistency, not correctness or idiom (philosophy P19, Bet 6). The frozen functional suites (§2.3) and
independently sourced Rust baselines (§2.5, §6.3) are the external anchors that make the numbers mean
something.

8.8 **A "4-of-5 programs pass" or basket-average rule** — *rejected.* The basket was chosen
adversarially; an averaging rule lets the three easy programs mask failure on the allocator and
scheduler, which are the bet's hardest instance and Candor's home ground. Replaced by per-program KILL
floors and the worse-of-weighted/mean aggregate (§5.1–5.2).
