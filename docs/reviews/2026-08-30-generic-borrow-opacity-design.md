# Generic borrow opacity: assessment and design memo (ledger P22)

Workstream: ledger P22 (docs/reviews/2026-08-03-checker-review-adjacent-findings.md),
the two-halved generic-borrow family found by the P7/P8/P11-fix adversarial
review (repro shapes k3, l2, s1, q1). Written 2026-08-30 against d7152ca plus
this workstream's own change. All measurements in this memo were re-run in this
workstream; repro sources are inlined below so they survive the scratchpad.

**Ruling (2026-08-30): the deciding authority ratified option (i), the
Proj-only conservative rule, as recommended below. IMPLEMENTED (same day, its
own reviewed workstream) FOR THE RATIFIED SCOPE — bare `I::Item` and arrays
of it: the named predicate `may_store_borrow` covers the loan machinery and
`check_signature_regions` (E1006/E0201 keep the unwidened
`field_stores_borrow`), with the assoc-method provenance extension; spec 04
§7.6 records the rule. The bare-Proj shapes closed as §2(i) predicted — l2
E0806, k3-internal E0801, leak2 def-site E0807 — and the leak2 lock-in test
flipped to its rejection twin. Re-measured over-rejection on the full corpus:
zero diffs. P26 (the same-call-overlap mirror gap logged in §4) was closed
for free generic calls in the same workstream (the interface-method path is
ledger P28). HOWEVER, the implementation's adversarial review REFUTED this
memo's completeness argument for (i): a WRAPPED projection (`Opt[I::Item]`)
still launders a borrow-bound Item, because E1006 does not run at
constructor/annotation positions (ledger P9, reopened) — see the corrected
§2(i) paragraph and the new "App extension, measured" section. P22 is
therefore PARTIALLY closed: bare-Proj def-site shapes closed, App-of-Proj
residual open under P9.**

Summary of verdicts:

* Sub-problem (b) — generic calls shed all loans — is CALL-SITE-side, contained,
  and is **implemented in this workstream** (~65 lines in two checker files, no
  architecture change). It also closes ledger **P16** outright and, together
  with the pre-existing P7 generic out-slot extension, closes the
  **caller-visible halves of k3, l2, and s1 in their single-borrow-input
  forms** (the projection resolves at a concrete call site). One caller-side
  shape of the family remains OPEN because its backstop is def-site: two
  post-substitution borrow inputs with a `-> I::Item` return (repro leak2,
  §1) — the compact default is ambiguous there by design, and the concrete
  side's E0807 signature backstop is defeated by Proj opacity, so it belongs
  with the (a) ruling.
* Sub-problem (a) — Proj opacity inside generic bodies — is DEF-SITE-side and
  remains open. The measured recommendation is option **(i) refined to
  Proj-only** ("Proj" = an opaque associated-type projection such as
  `I::Item`, the interface's associated type seen from inside a generic body
  before any impl binds it): treat a bare `I::Item` (and arrays of it) as
  potentially borrow-storing in the loan machinery. Measured over-rejection on
  the entire shipped corpus, including the 50 kL P20 reference and the corelib
  iterator stack: **zero subjects**. It closes every def-site repro shape,
  including leak2's missing E0807 backstop (reasoned through in §2). Not
  implemented here — it needs a deciding-authority ruling (it makes some
  per-instantiation-legal programs def-site-illegal, which is exactly the
  check-once philosophy, but that is a language-visible strictness).
* Option (ii) (per-instantiation loan re-checking) is cheap in wall-clock terms
  at ratified scale (measured upper bound: tens of milliseconds) but breaks the
  ratified architecture twice over — design 0007 §5.2's "instantiation is
  codegen, never re-analysis" and P20's exact `T2 == 0` gate — so its real cost
  is governance, not time.

## 1. The two sub-problems, separated

### (b) Call-site: `check_generic_call` ended with an unconditional loan clear

`check_generic_call` (compiler/src/check/generics.rs) computed substituted
parameter and return types for every generic call — the P7 out-slot extension
already used them — and then ended with an unconditional `clear_carried()`. So
EVERY generic free function returning a borrow or a view shed its argument
loan at the call site. Reviewer repro q1 (== ledger P16):

    fn idr[T](p: read T) -> read T { return p; }
    fn main() -> i64 {
        let mut x: i64 = 5;
        let r: read i64 = idr(read x);
        x = 9;                      // write while the borrow lives
        return r.*;                 // observed 9 through a "live" shared borrow
    }

Verified on the pre-fix binary: checks clean, runs to 9 on the oracle.

The caller knows the full substituted signature at the call site, so the
concrete return-borrow rule (`check_user_call`'s §3.1/§3.3 extension:
`field_stores_borrow(ret)` → carry the region-source arguments' loans) applies
directly with the substituted types. This is contained: no def-site or
architecture change, and the landing-site predicate (`carries_borrow`) gets the
substituted answer through the same per-call-span `borrow_valued` record the
P11 method-call fix introduced. Implemented — see §4.

### (a) Def-site: opaque `I::Item` defeats the loan machinery inside generic bodies

Generic bodies are checked once with opaque type parameters (design 0007
§5.2). Inside such a body, `field_stores_borrow(Type::Proj(..)) == false`, so
every loan rule keyed on that predicate silently stands down. The checker
cannot know whether `I::Item` stores a borrow — an impl may legally bind
`type Item = read i64` (the P11 shapes).

What this workstream measured, shape by shape, against the post-(b) binary:

| shape | caller-visible half | def-site-internal half |
|---|---|---|
| k3 (`-> I::Item` assoc return) | **CLOSED by (b)** — E0803 (projection resolves at the concrete call site) | **OPEN** — `let r: I::Item = it.get(); it.bump();` checks clean; write through the receiver while `r` lives |
| l2 (`out I::Item` slot) | **CLOSED pre-existing** — the P7 generic out-slot extension already runs on substituted types (E0803) | **OPEN** — `o = it.get()` with `it: I` (owned, dies at return) stores a dead-frame borrow into the caller's slot; checks clean, RUNS |
| s1 (match joining to Proj) | **CLOSED by (b)** — E0803 (the call's return extension does not care what the body's match did) | **OPEN** — same class as k3-internal through the match landing |
| q1 (`-> read T`) | **CLOSED by (b)** — E0803 | n/a (no Proj; `read T` is borrow-kind syntactically) |
| leak2 (`-> I::Item`, TWO borrow inputs) | **OPEN** — the compact default is ambiguous with two post-substitution borrow inputs, so (b) deliberately carries nothing, matching the concrete rule | **OPEN** — this is where the backstop belongs: the concrete twin cannot even be declared (E0807 without a region), but `field_stores_borrow(Proj) == false` skips the signature rule, and no region tag is parseable on a `-> I::Item` return, so the ambiguous branch is the only reachable one |

The leak2 residual, found by the (b) adversarial review — checked clean and
ran to 9 through a "live" borrow when this memo was written (verified; it was
locked in as a currently-accepted test, `p22_open_hole_two_borrow_inputs_
proj_return_still_accepted`, now flipped by the (i) implementation to its
rejection twin `p22_two_borrow_inputs_proj_return_rejected_at_def_site`):

    fn leak2[I: Get](it: read I, extra: I::Item) alloc -> I::Item {
        return it.get();
    }
    // caller: let b: read i64 = leak2(read q, read x); q.a = 9; b.* == 9

This is not fixable at the call site without diverging from the concrete
rule: with two borrow inputs and no region information the caller genuinely
cannot know which argument the result derives from, and carrying the union
would falsely reject callers that write the other input's owner. The concrete
side makes the ambiguous case unreachable by rejecting the CALLEE's signature
(E0807); restoring that def-site backstop is option-(a) work — see §2 (i).

The open l2 repro, inlined (checks clean and runs today; the read lands on a
dead frame):

    interface Get { type Item; fn get(read self) -> Self::Item; }
    struct Q { a: i64 }
    impl Get for Q { type Item = read i64;
                     fn get(read self) -> read i64 { return read self.a; } }
    fn fill[I: Get](o: out I::Item, it: I) alloc -> unit { o = it.get(); }
    fn main() alloc -> i64 {
        let q: Q = Q { a: 5 };
        let mut s: read i64;
        fill(out s, q);            // q moves in, dies at fill's return
        return s.*;                // dead-frame read; runs, prints 5 today
    }

The open k3-internal repro: `fn inner[I: Get](it: write I) -> i64 { let r:
I::Item = it.get(); it.bump(); let s: I::Item = r; return 0; }` — the write
through `it` while `r` (concretely a borrow of `it`'s innards) is live checks
clean today.

## 2. Design options for (a)

### (i) Conservative def-site rule — RECOMMENDED, refined to Proj-only

Treat an opaque projection as potentially borrow-storing for loan purposes at
the definition site. Two variants were prototyped behind a toggle (a scoped
predicate flip across the loan machinery's 31 `field_stores_borrow` call sites
in check/{expr,mod,stmt,generics}.rs, explicitly excluding the E1006
type-argument rule and the E0201 field rule) and swept over the full corpus:
368 subjects = every `.cnr`/`.cn` under compiler/tests/fixtures + selfhost +
ports, plus the corelib module tree and the 50 kL P20 reference tree, 284 of
them clean at baseline.

* **Bare `T` + `I::Item` (the blunt reading of the ledger's "and bare T?"):
  catastrophic and unnecessary.** 12 previously-clean subjects newly fail,
  including the ENTIRE 50 kL reference and the corelib tree; the sweep gains
  +812 E0806 and +805 E0807 error lines. Every `fn max[T](a: T, b: T) -> T`
  becomes "a borrow return from two or more borrow parameters". And it buys
  nothing: E1006 already bars borrow-kind type arguments ("borrows are for
  passing and computing, not abstracting over", 0007 §3.5), so a bare `T` can
  never be instantiated at a borrow or view. Bare `T` needs no conservative
  treatment. Rejected variant.
* **Proj-only (bare `I::Item` and arrays of it): measured over-rejection is
  ZERO.** Not one of the 368 subjects changes a single diagnostic — including
  the real associated-type generic code we ship (corelib core/iter.cnr,
  iter_adapters.cnr, iter_terminals.cnr, std/list.cnr, arena.cnr,
  split_lines.cnr). And it closes the open holes, verified on the toggled
  binary: l2 def-site rejects E0806 ("borrow assigned to `out` parameter `o`
  does not provably derive from an input"), k3-internal rejects E0801
  (conflicting borrow of `it`).

**Does (i) close the leak2 residual? Yes — it restores the missing E0807
backstop.** Reasoned against the prototype's own mechanics: the toggle's
predicate flip covered `check_signature_regions` (the E0807/E0808 signature
rule in check/mod.rs, one of the 31 flipped sites). Under Proj-only, leak2's
declared signature `(it: read I, extra: I::Item) -> I::Item` reads at the def
site as: borrow-kind return (Proj now counts), TWO borrow inputs (`it` by
`read` mode, `extra` by its now-borrow-storing Proj type), no region variable
— and none is spellable on a Proj return. That is precisely the concrete
twin's E0807 ("a borrow return from two or more borrow parameters requires a
region variable"), issued at the definition, which makes the call-site
ambiguous branch unreachable for generic code exactly as it already is for
concrete code. The zero-over-rejection sweep already priced this rule in: no
shipped generic def has two-plus borrow inputs with a Proj return. Residual
grammar note for the (i) workstream: single-borrow-input Proj returns keep
the compact default and stay legal; if a legitimate two-input shape ever
appears, it needs region tags to become spellable on projection returns (a
small grammar addition, decidable then). This finding materially STRENGTHENS
the (i) recommendation — it closes the one caller-side shape (b) cannot.

Why Proj-only coverage was ARGUED complete — CORRECTED 2026-08-30 by the
implementation's adversarial review: this memo originally claimed a nested
position such as `Opt[I::Item]` would, at Item = borrow, instantiate a
generic enum at a borrow-kind argument and be caught by E1006. That is
WRONG about where E1006 runs: `check_arg_conformance` fires only on generic
FN call type arguments (check/generics.rs check_bounds callers, :75/:890)
and generic-impl conformance (:1110) — never on struct-literal or
enum-constructor type arguments, and never on type annotations (`let o:
Opt[read i64] = Opt::None;` checks clean today — verified). So App-of-Proj
IS a live laundering route: `fn wrap[I: Get](it: read I) alloc ->
Opt[I::Item] { return Opt::Some(it.get()); }` checks clean under the
shipped (i) rule and runs (repros inlined in the reopened ledger P9 entry;
locked in as `p22_open_hole_app_of_proj_*`). The parenthetical about P9
was carrying the whole claim: P9 is not a modulo, it is the hole. Closing
it (running the E1006 borrow-argument rule at constructor and annotation
positions) is the (iii-b)-flavored path; extending the predicate through
App instead was prototyped and measured — see "App extension, measured"
below.

Costs and residual work for a shipped (i):

* Zero compile-time cost (a predicate change), no architecture change,
  check-once preserved, P20 untouched.
* Expressiveness: the `-> I::Item` / `out I::Item` def-site shapes come under
  the E0806 provenance regime. So that legal accessor idioms (`fn first[I:
  Get](it: read I) -> I::Item { return it.get(); }`) stay legal, the shipped
  rule should extend `borrow_provenance` to assoc-method calls
  (receiver-rooted, compact default — the mirror of what
  `check_iface_method_call` already does on the loan side). The dead-frame l2
  still rejects, correctly, as "derives from owned parameter". The prototype
  did NOT include this extension and still measured zero over-rejection, so
  today's corpus cost is zero either way; the extension is for future user
  code.
* The prototype's zero must be read honestly: shipped generic code today
  simply never binds Item to a borrow and never returns bare `I::Item` from a
  generic free fn. The rule's conservatism prices exactly the code that would
  start doing so.
* Needs: a named predicate (not a flip of `field_stores_borrow`, which E1006
  and E0201 also consume with different meanings), the provenance extension,
  diagnostics wording, def-site tests, a 0007 §3.5/0009 §2.2 spec amendment,
  and a deciding-authority ruling — this makes some programs that every
  concrete instantiation would accept ill-formed at the definition site. That
  is precedented def-site conservatism (E1020 polymorphic recursion, §3.4
  alloc-on-drop fixed conservatively for every instance), but it is
  language-visible. Estimated as a small workstream: one to two focused days
  of checker work plus the spec/ratification loop.

### (ii) Per-instantiation borrow re-checking

What design 0007 §5.2 ratified, verbatim: "the body is type-, move-, loan-,
and effect-checked exactly once, at its definition… Instantiation is codegen,
never re-analysis… The check-once architecture is total: types, moves, loans,
and effects are all settled at the definition." That promise is what P20's CI
gate operationalizes (ratified 2026-07-21; "cc-calibrated" 2026-08-03 — the
CI gate scales its timing ceilings by how long `cc -O2` takes to compile a
fixed 49 kL C tree on the runner, a pure-hardware yardstick, so noisy shared
runners relax the limits but reference-class hardware enforces the ratified
numbers exactly): clean check ≤ 3000 ms at the 50 kL reference (actual
1327 ms on reference hardware at bfd127c), T1 ≤ 1000 ms, **T2
downstream-reanalyzed == 0 (exact, never scaled)**, release ≤ 2× cc -O2
(actual 1.37×).

Measured on the committed 50 kL reference (202 modules, merged tree; this
workstream's harness, test profile opt-level=1 — ratios and counts are the
meaningful figures, not absolute times):

* 5797 functions total; **801 generic fn defs**; **1597 reached
  instantiations** (direct, deduped), **1601 monomorphized instance fns**
  (transitive closure) — a ~2.0× instantiation multiplier.
* **Borrow-relevant generic bodies: 0 of 801.** No generic def in the
  reference has a borrow-kind param/return, a non-take mode, or borrow ops in
  its body — the ratified-scale tree is deliberately a scalar workload, so a
  loan-only per-instantiation pass would re-check NOTHING there and cost ~0.
* Upper bound for FULL per-instance re-analysis — measurement basis stated
  precisely, since the number is load-bearing: both timings are medians of
  five in-process `check::check_program_real` calls over a MERGED module-tree
  `Program` (the output of `modules::build_tree` on
  benches/p20-reference/candor), same build, test profile (opt-level=1, not
  the release build the CI bench times — so these support bounds and deltas,
  not comparison against the ratified absolute ceilings). Left side: that
  merged program as shipped, which takes the GENERIC checking path (double
  resolve, def-site passes, bound conformance, instantiation recording) —
  **1993 ms**. Right side: the output of `generics::monomorphize` on the very
  same merged program, which takes the concrete path — **58 ms** for all 7398
  fns including the 1601 instance bodies. The two sides run different checker
  paths by construction (that difference is the point: instantiation-time
  re-analysis IS the concrete path over instance bodies), so the honest
  reading is not a ratio but a bound: complete concrete re-analysis of every
  instance body in the 50 kL tree costs tens of milliseconds — a few percent
  of the clean-check budget, far inside the ~1.7 s headroom under the 3000 ms
  ceiling, and smaller still on the release build.

So wall-clock is NOT the argument against (ii). The real costs:

1. **It breaks T2 as an invariant, not as a number.** A generic BODY edit
   would have to re-run the loan layer in every downstream module that
   instantiates it. T2 ("a body edit re-analyzes only the edited module",
   downstream == 0, exact, never scaled, CI-gated) fails structurally, and
   the signature-bounded invalidation story of 0008/0010 §3 goes with it.
2. **There is no separable loan layer.** The checker is single-pass — loans
   are interleaved with typing and effects — so "re-check only the LOAN layer"
   is new architecture, and re-running full body checks per instantiation
   re-derives effects that §5.2/§3.4 promise are never re-derived (generic
   alloc-ness is fixed once, conservatively, at the def site).
3. It reopens the ratified 0007 §5.2 text and the P20 gate definition —
   a re-ratification, for a hole that (i) closes at zero measured cost.

Viable only if (i)'s conservatism ever proves untenable AND (iii) is refused.

### (iii) Declared bounds — language-visible

Two flavors:

* **(iii-a)** Interfaces declare, per associated type, whether it may be
  borrow-kind (e.g. `type Item` defaults to value-only; `type Item: view`
  opts in). Def-site checking is then precise: undeclared Items are known
  non-borrow (no conservatism at all), declared ones get (i)'s treatment.
  Cost: grammar + resolver + conformance (E1021-family) + spec chapter 10 +
  ratification, and a migration story for existing interfaces. Medium
  workstream.
* **(iii-b)** The stronger cut: forbid borrow-kind associated-type bindings
  outright — extend E1006's "borrows are for passing and computing, not
  abstracting over" to `type Item = read T` and kin. This erases sub-problem
  (a) entirely (no Proj can ever be a borrow) and would let the P11
  substituted-return machinery and (i)'s conservatism both retire. Notably,
  the shipped borrowed-iteration design (0015 RefIndexed) already avoids
  borrow-bound Items — `get_ref` returns `read Item` structurally with Item
  owned — and design 0009's Iter yields owned Items, so today this ban costs
  nothing in shipped code. But P11's fix implicitly legalized the binding, so
  banning it is a breaking language decision that needs the authority.

### (iv) Status quo, documented as a 1.0 obligation

Not defensible: the residual def-site holes are real unsoundness reachable
from safe code (the l2 repro checks clean and RUNS to a dead-frame read
today), and the ledger's own standard already classifies borrow-model
soundness holes as pre-1.0 work (P4's disposition). Documenting without
fixing just moves the P22 entry into 99-obligations with the same content.

### Recommendation

**Option (i), Proj-only, with the assoc-method provenance extension**, spec'd
as a 0007 §3.5 / 0009 §2.2 amendment and put through the review pipeline.
Zero measured over-rejection on everything we ship; closes every def-site
repro shape — l2 and k3-internal measured on the toggled binary, leak2's
E0807 backstop reasoned through the same flipped rule (§2 (i)); preserves
check-once, the single-pass checker, and every P20 number; consistent with
the existing def-site conservatism precedents.

**Runner-up: (iii-b)** (ban borrow-kind assoc bindings). Trigger condition:
the first confirmed report of legitimate generic code that (i) rejects at the
def site and that users actually need (a Proj-borrow accessor idiom the
provenance extension cannot accept), or a P9-closure finding that inference
can still launder a borrow-storing Item past E1006. The two compose: (iii-b)
supersedes (i) later without a compatibility break, since it only turns (i)'s
def-site rejections into earlier, clearer ones at the impl site.

## 3. Interaction with ledger P16 — subsumed

P16 ("generic functions do no return-borrow loan extension",
`fn idr[T](p: read T) -> read T`) IS sub-problem (b)'s q1 shape. The
implemented call-site rule closes it: the repro now rejects E0803 and the
within-window twin stays clean (tests `p16_generic_borrow_return_*` in
compiler/tests/generics.rs). Once this lands, the P16 ledger entry can be
marked closed by this workstream; the ledger itself is left untouched here.

One deliberate asymmetry left standing, for the record: a CONCRETE fn
returning a borrow via a generic call in return position (`fn f(p: read i64)
-> read i64 { return idr(read p); }`) remains conservatively rejected E0806 —
`borrow_provenance` does not recognize generic callees. That is a sound
rejection, not a hole; extending the provenance walk to generic calls (the
substituted mirror of its concrete-call branch) is a small follow-up if it
ever bites real code.

## 4. What was implemented (sub-problem (b) only)

`compiler/src/check/generics.rs` — `check_generic_call` now ends with the
concrete return-borrow extension judged on the SUBSTITUTED return type: if
`field_stores_borrow(subst(ret))`, the loans captured for the region-source
arguments (`generic_region_source_indices`, the substituted-type mirror of
`expr::region_source_indices` — region tags from the declaration; compact
default = the sole borrow input, judged after substitution so a take-mode
`s: [T]` view argument counts; ambiguous = carry nothing) are set as carried,
and the call span is recorded in `borrow_valued` (the P11 pattern).
Otherwise `clear_carried()` exactly as before.

`compiler/src/check/stmt.rs` — `carries_borrow`'s Ident-callee branch
consults `borrow_valued` for generic callees, so `let`/assignment landings
anchor the carried loans (the raw generic signature cannot show borrow-ness;
the recorded substituted answer can).

Two deliberate non-mirrorings, for the record: the concrete path's P23
arity panic is NOT reproduced (`check_generic_call` early-returns on an
arity mismatch before the extension ever indexes `per_arg`); and the
adversarial review found that `check_generic_call` also never pushes a call
group, so the §3.1 same-call overlap rule (E0805) silently skips generic
calls — a second pre-existing mirror gap of this family, logged as new
ledger item **P26** rather than fixed here.

Tests (compiler/tests/generics.rs, mirroring the concrete twins in
loans.rs), seventeen in all:

* `p16_*` core twins: q1 rejected (E0803) plus within-window accepted;
  exclusive borrow return (E0804); view return via `[T]` (E0803) plus
  within-window; assignment-landing twin (E0803); owned-return branch sheds
  the argument loan (clean — no sibling leak past `clear_carried`);
  array-of-borrows return twin of loans.rs's P4 shape (E0803); shared return
  deriving from an exclusive input carries the exclusive loan (E0804).
* `p16_generic_region_*`: the region-tagged signature family exercising the
  `Some(r)` arm of `generic_region_source_indices` — def-site E0807 (two
  borrow params, no region) and E0808 (provenance mismatch) twins, the
  call-site tagged-source conflict (E0803), and the untagged-source dual
  (clean: the tag selects WHICH argument's loan extends).
* `p22_*` measured-closure locks: `-> I::Item` resolving to a borrow at the
  call site (k3 half, E0803), `out I::Item` extending the caller loan (l2
  half, E0803), the match-joining s1 half (E0803), and the leak2 OPEN-HOLE
  lock-in (asserts the current acceptance with a comment naming the ledger
  item; flips to a rejection twin when the (a) ruling lands).

Verification (re-run after rebasing the branch onto main 58d26c3, which
brought the P15 float-conv fix and its gates):

* Corpus sweep, clean-58d26c3 baseline binary vs fixed binary, 368 subjects
  (all fixture `.cnr`/`.cn` files + selfhost + ports + corelib tree + the
  50 kL p20-reference tree): **zero diagnostic diffs** — no shipped program
  changes behavior; the bug shapes do not occur in the corpus. The same
  zero-diff result held pre-rebase against d7152ca.
* The option-(i)/(ii) measurement instrumentation (predicate toggle, counting
  harness) was fully reverted; the toggle env var is inert on the shipped
  binary (re-verified).
* `cargo nextest run` (full default profile, rebased tree): **1441/1441
  passed** (main added float-conv gates; this workstream adds the seventeen
  generics tests above).
* `cargo clippy --all-targets`: green — the only output is the pre-existing
  "unknown lint `clippy::chunks_exact_to_as_chunks`" warning from
  src/build/sha256.rs's allow for a newer clippy than this environment runs.

## 5. App extension, measured (2026-08-30, post-ruling; prototype reverted)

The (i)-implementation adversarial review refuted §2(i)'s completeness
argument (see the corrected paragraph above): App-of-Proj launders a
borrow-bound Item past the shipped rule. The obvious candidate closure —
extend the conservative predicate through App — was prototyped and measured
in the implementation workstream, behind a local env toggle
(`CANDOR_P22_APP_PROTO`, `OnceLock`-cached; fully reverted after
measurement, re-verified inert): `may_store_borrow(App(_, args))` is true
when any (transitively) instantiated argument contains a Proj.

Measured over-rejection, toggle off vs on, same release binary, the full
370-subject corpus (fixtures + selfhost + ports as files, plus the corelib,
selfhost, and p20-reference module trees):

* **3 subjects diff; 1 previously-clean subject newly rejects — the corelib
  TREE itself** (0 -> 24 E0806 error lines). The two other diffs are the
  same errors seen through the standalone module files (core/iter_adapters.cnr
  +17 E0806 lines, std/list.cnr +2; both already non-clean checked standalone,
  as bare modules). Total +43 E0806 lines corpus-wide.
* The rejected shapes are exactly the shipped iterator stack: every generic
  adapter `next` returning `IterStep[Item, Self]` CONSTRUCTORS
  (`return IterStep::Done;`, `return IterStep::More(x, TakeN { inner: rest,
  n: self.n - 1usize });`, Zip's `More(Pair {...}, Zip {...})` — spans mapped
  to iter_adapters.cnr:39/43/46/192 and 13 more) plus list.cnr's fold-style
  `acc` returns. Mechanism: the App-flagged return makes `ret_is_borrow`
  true at the def site, and the provenance walk has no rule for constructor
  expressions — `None` => E0806. Fixtures (non-corelib), selfhost, ports,
  and the 50 kL reference: zero diffs.
* **The P20 bench cannot even run with the toggle on**: its corelib
  continuity tree fails the cold build (`cold build had errors`,
  benches/p20.rs:118) — the headline cost is correctness of shipped code,
  not time. Timing was measured on the basis available: release CLI `check`
  of the 50 kL reference tree (which stays clean under the toggle), five
  runs each — toggle off 1.49–1.61 s (median 1.55), toggle on 1.48–1.51 s
  (median 1.50). The predicate itself costs nothing measurable.

Recommendation: do NOT ship the naive App extension. It rejects the exact
corpus the ratified zero-measurement protected, and making it shippable
means constructor-aware provenance (per-payload provenance through
enum/struct literals, plus landing-site loan plumbing for wrapped borrows) —
a real design workstream, not a predicate tweak. The cheaper and cleaner
closure for the App-of-Proj residual is the P9/E1006 route (the (iii-b)
flavor): run the borrow-type-argument rule at constructor and annotation
positions, so a borrow-bound Item cannot be NAMED at the laundering
positions at all. Expected corpus cost is zero (no shipped code names a
borrow-kind constructor/annotation type argument), to be confirmed by that
workstream's own sweep; it needs its own ruling since P11 implicitly
legalized the binding it constrains. Until then the residual is locked in
as accepted-today tests (`p22_open_hole_app_of_proj_*`) and tracked as the
reopened ledger P9 / P22-partial.
