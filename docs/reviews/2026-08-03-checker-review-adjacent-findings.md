# Adjacent pre-existing bugs from the checker-fix adversarial review (2026-08-03)

Found while attacking the array-repeat/array-literal checker fixes; all five
reproduce identically on the pre-change and post-change binaries, so none are
regressions of that work. Logged here so they survive the review transcript;
each is queued as its own workstream. Severity per the reviewing session.

## P1 — Generic monomorphization renders array type arguments with length 0 (HIGH)

`compiler/src/generics.rs:1441-1444` maps `Type::Array(e, _)` to
`TyKind::Array { size: IntLit 0, .. }` when rendering a type argument.
`fn idf[T: copy](x: T) -> T` called with a `[3]i64` returns **3 on the
tree-walk oracle and 0 on MIR/native (both tiers)** — wrong results in safe
code, engine-divergent. Non-generic array pass-through and generic
copy-struct pass-through are unaffected.

## P2 — Never-initialized needs-drop local runs its hook on garbage (HIGH)

`let mut p: D;` (drop-hooked, never initialized) traces nothing on the
oracle but **`[0]` on MIR and native**: the hook runs over zeroed,
never-constructed storage. Same defect family as the overwrite-drop
dossier's D1 correction (the builder's move mask models moves, not
initialization); any assignment-drop fix must track init state separately
or it will fire hooks on uninitialized memory for the legal
`let mut p: D; p = D{..};` shape.

## P3 — Runtime-valued repeat length diverges (MEDIUM)

`let n: usize = 3; let a = [7i64; n];` checks clean, runs on the oracle,
**faults `Bounds` on MIR/native**. Design 0001 (line 280) requires
compile-time-constant lengths; nothing enforced it. Being fixed in the
checker workstream (reject non-constant repeat sizes), which closes this.

## P4 — Arrays of borrows escape the loan machinery (MEDIUM, soundness)

`let a: [3]borrow i64 = [read x; 3]; x = 9; return deref a[0];` returns
**9** — a write through a live shared borrow, unchecked. The dangling
variant reads a dead frame slot and returns stale data. Reachable through
the array-literal form as well, so this predates E0716 and is not created
by admitting `read`-borrow repeat elements. The loan checker needs to track
borrows stored into array elements (it tracks struct fields; arrays fell
through).

## P5 — Untyped-literal array narrows silently through inference (CLOSED 2026-08-26, was RE-OPENED, MEDIUM)

`let t = [1, 2]; S { a: t, b: 42 }` with `a: [2]u8` stores `a[1] == 0` on
all engines. Root cause: an array of unsuffixed integer literals is never
grounded to a concrete element type. The array-literal unification fix
narrowed this but did NOT close it: the P1 workstream found a live shape —
a generic struct literal whose array field is all-unsuffixed
(`let w: Wrap[[3]i64] = Wrap { v: [4, 5, 6] };`) instantiates the generic
at an ungrounded element type while the annotation instantiates it at the
concrete one, producing two divergent instances of one type (oracle faults;
MIR/native refuse). Four-line repro in the P1 workstream report. Needs the
checker to ground literal element types against the expected type before
the monomorphization shape is recorded.

CLOSED by the P5 grounding workstream (adversarially reviewed, SHIP): an
unsuffixed literal array now grounds its element type from the expected type
where one exists (with per-element E0709 range checks), and to the `i64`
default at the escape points (unannotated `let`, generic type-argument
binding) — no `{integer}` reaches a layout or a monomorphization shape
(debug-asserted at `record_inst`). Both repros flip: the Wrap shape checks
clean and runs `[4,5,6]` on every engine; the binding shape is E0703.
Residual, still open as its own item: P15 below (array-literal-as-index-base
still silently narrows on the oracle). New same-family scalar finding: P16.

## Disposition

P3 and (likely) P5 are closed by the in-flight checker workstream. P1, P2,
and P4 are open, each warranting its own fix + adversarial review; P2 folds
into the overwrite-drop / assignment-drop workstream (same init-tracking
substrate). None block the native-rollback launch precondition, but P1 and
P4 should land before any 1.0 conversation — P4 is a borrow-model soundness
hole and P1 silently computes wrong results in generic code.

## P6 — Orphan-task crash on parent fault inside an open scope (from the rollback review)

A parent that faults between `spawn` and the scope's join `_longjmp`s without
joining; orphaned task threads then dereference the cleared `CURRENT` runtime
pointer and abort the process (reproduced ~60-80% of runs, identical before
and after the rollback change). A theoretical follow-on: an orphan surviving
into the next in-process JIT run would see the new runtime's `live_tasks == 0`
and could lower a live bump via `rt_stack_restore` — unreachable today because
the null-`CURRENT` abort always wins first, but it becomes reachable if the
orphan crash is ever "fixed" by making `rt()` null-tolerant. Fix the orphan
lifecycle (join-or-detach on the fault path), not the symptom.

# Ledger additions from the P4-fix adversarial review (2026-08-18)

Pre-existing holes found while attacking the P4 (arrays-of-borrows) fix. All
four reproduce on the pre-P4-fix binary, so none are regressions of that
work; each is logged here to be fixed in its own workstream, not in the P4
one. Source: the 2026-08-18 adversarial review of the P4 fix.

## P7 — Out-mode borrow escape (HIGH, pre-existing)

A borrow written into an `out`-mode parameter slot leaves the callee without
any provenance or region check: the return-provenance walk only covers
`return` expressions, so a callee can store a borrow of its own local into
the caller's slot and the caller then reads a dead frame. Applies to plain
borrows and to arrays of borrows alike (the review has runnable repros for
both). The fix belongs with the out-slot machinery, next to the E0806 walk.

## P8 — Match-arm borrow shedding (HIGH, pre-existing)

A borrow value produced as a `match` arm's result sheds its loan on the way
out: the carried-loan protocol anchors at `let`/assignment landing sites and
extends through calls, but the arm-result path does not re-carry, so the
landing binding holds an unguarded borrow and the borrowed place reopens.
Plain borrows and arrays of borrows both take this path. (The return-side
walk already recurses into `match` arms; the value-side carry does not.)

## P9 — E1006 annotation-position gap (REOPENED 2026-08-30, HIGH, reachable)

E1006 ("a borrow type is not a legal type argument", now also arrays of
borrows) fires at explicit type-argument positions. A type argument that is
only ever *inferred* through an annotation position is not routed through
`check_arg_conformance`, so a borrow-storing argument could be laundered in
by inference. Today the gap is unreachable in practice only because the
generic-monomorphization array bug (P1 above: array type arguments render
with length 0) breaks the carrier shapes first — the in-flight generics
workstream that fixes P1 may unblock this path, so re-test E1006 coverage
when P1 lands.

REOPENED as HIGH/reachable (P22-implementation adversarial review,
2026-08-30): P1 is fixed, and the gap is wider than "inferred annotations" —
`check_arg_conformance` runs ONLY on generic fn calls and impl conformance
(its `check_bounds` callers at generics.rs:75 and :890, and the generic-impl
conformance check at :1110), NEVER on struct-literal / enum-constructor type
arguments or on type annotations. `let o: Opt[read i64] =
Opt::None;` checks clean today (verified). Combined with P11's legalization
of borrow-bound `type Item`, this makes App-of-Proj a live borrow-laundering
route through the ratified P22(a) rule (which is deliberately bare-Proj-
only). Both escape repros check clean and RUN on the current binary
(verified 2026-08-30; also locked in as accepted-today tests
`p22_open_hole_app_of_proj_*` in compiler/tests/generics.rs):

    // shared escape: runs to 9 through a "live" wrapped borrow
    enum Opt[T] { Some(T), None, }
    interface Get { type Item; fn get(read self) -> Self::Item; }
    struct Q { a: i64 }
    impl Get for Q { type Item = read i64;
                     fn get(read self) -> read i64 { return read self.a; } }
    fn wrap[I: Get](it: read I) alloc -> Opt[I::Item] {
        return Opt::Some(it.get());
    }
    // caller: let o: Opt[read i64] = wrap(read q); q.a = 9; match ... b.* == 9

    // exclusive escape: two live aliases; runs to 42
    struct W[T] { v: T }
    // impl binds type Item = write i64; get(write self) -> write i64
    fn wrap[I: Get](it: write I) -> W[I::Item] { return W { v: it.get() }; }
    // caller: let w: W[write i64] = wrap(write q); q.a = 7; w.v.* = 42;

Closing P9 (running the E1006 borrow-argument rule at constructor and
annotation positions) would also close the App-of-Proj residual of P22 the
(iii-b)-flavored way; the measured alternative (extending the conservative
predicate through App) rejects the shipped corelib iterator tree — numbers
in docs/reviews/2026-08-30-generic-borrow-opacity-design.md, "App
extension, measured". Cross-reference: P22 (partial closure), P11.

## P10 — Block-scope dangle for plain borrows (MEDIUM-HIGH, pre-existing, now confirmed)

Confirmed by repro during the P4 work: `let mut b = read x; { let y = 3;
b = read y; } return deref b;` checks clean and returns the dead slot's
value. A scope exit is neither a use nor a def in the loan liveness scan,
and no rule ties a loan's life to its referent's scope, so a borrow of a
block-local outliving the block escapes. Arrays of borrows now inherit
exactly this (and no worse) behavior by parity; fixing it means tying loan
places to the scope depth of their roots, a separate workstream.

## P11 — Interface methods via associated types shed their return borrow (HIGH)

`method_returns_borrow` reads the method's declared return type before
substitution, so a method whose signature returns `Self::Item` never
registers as borrow-returning even when `Item` is bound to a borrow (scalar
`read i64` and array `[2]read i64` alike). The landing binding discards the
receiver's loan and a later write through the "live" borrow checks clean and
is observed at runtime. Distinct from the (closed) B1 predicate split: this
is a substitution-ordering hole and it predates the borrow-array work
(reproduced on both binaries). Fix direction: consult the substituted return
type where the call is checked.

## P12 — Instance-name structural prefixes forgeable by user type names (MEDIUM)

The mangling scheme's verbatim fast path leaves plain identifiers unencoded,
so a user type literally named `arr3` (or `arr`, `slice`, `ptr`, `Box`, ...)
can collide with the structural array form in single-file programs
(module-qualified names encode and cannot collide). Pre-existing class,
net-improved by the injective-mangling work (the plain `arr[T]` collision is
now fixed); eliminating the class means encoding every name and accepting
instance-name churn for plain names — a recorded trade-off, not an
oversight.

## P13 — Selfhost mangling does not mirror the qualified-name encoding (LOW)

The selfhost lowering emits type and length names as raw source spans where
the reference now emits the length-prefixed encoding; a selfhost fixture
using a qualified or underscored name in a generic instance would fail
loudly (name mismatch against the reference-built tables), not miscompile.
No fixture hits it today.

## P14 — Pairwise array-literal loan scan double-counts one exclusive loan (COSMETIC)

`[b, b]` where `b` is an exclusive borrow emits a spurious extra E0801
beside the pre-existing E0301; the two propagated copies of one loan are
counted as a conflicting pair. Can only co-occur with E0301, so no legal
program is rejected.

## P15 — Float-to-small-integer conversion panics both Cranelift engines (CLOSED 2026-08-30, HIGH, compile-time)

`conv u8` / `conv u16` / `conv i8` of an f64 crashes the Cranelift JIT (both
tiers) and AOT at COMPILE time (cranelift-codegen 0.132.3's x64 emitter hits
unreachable code lowering a sub-32-bit saturating float convert), while the
tree-walk oracle and MIR interpreter execute the same program correctly. An
engine-parity hole outside the float gates' coverage (they only test conv
i32/i64). Minimal repro in the raytracer test header and MEMORY.md. Fix
direction: widen the convert to i32 in the lowering and narrow after. Found
by the ray tracer workstream, 2026-08-26.

CLOSED by the P15 workstream (adversarially reviewed): `eval_float_conv`
(src/backend/lower.rs) now saturates sub-32-bit targets via
`fcvt_to_{s,u}int_sat` at I32, extends to the i64 register, then clamps to
the TARGET type's bounds with icmp+select — matching the interpreters'
Rust-`as` rule (out-of-range clamps, negatives into unsigned -> 0, NaN -> 0).
The entry's original "widen to i32 and narrow after" hint was wrong on the
second half: a plain narrow would WRAP past-bounds values (300.7 -> u8 would
give 44, not 255); the clamp is load-bearing. LLVM was already correct
(`llvm.fpto*i.sat.iN` at exact width). Gated five-engine in tests/floats.rs,
tests/floats_f32.rs, and the AOT slice.

## P16 — Generic functions do no return-borrow loan extension (CLOSED 2026-08-30, HIGH)

`fn idr[T](p: read T) -> read T` lets the caller shed the argument loan
entirely: the returned borrow carries nothing, so writing the borrowed-from
owner while the result lives checks clean. Same laundering family as P9/P1;
found on the baseline binary during the P7/P8/P11 workstream (its fixes
cover interface methods and out-slots, not plain generic returns).

CLOSED by the P22(b) call-site workstream (memo
docs/reviews/2026-08-30-generic-borrow-opacity-design.md §1(b)/§3; this is
sub-problem (b)'s q1 shape): `check_generic_call` now applies the concrete
return-borrow extension on the SUBSTITUTED return and parameter types, so
the repro rejects E0803 and the within-window twin stays clean (tests
`p16_generic_borrow_return_*` in compiler/tests/generics.rs). Corpus sweep:
zero diagnostic diffs.

## P17 — Native trace order around joins diverges from the oracle (MEDIUM)

Native engines merge a task's trace at the JOIN; the oracle emits it at the
SPAWN point. Any parent trace between spawn and join reorders theta
deterministically (oracle [1,99] vs native [99,1]). No existing gated test
has a tracing task plus a tracing parent, so the differential suite never
sees it — a live hole in the per-task projection equivalence claim (design
0012 §6). Pre-existing; surfaced by the orphan-lifecycle review, 2026-08-26.

## P18 — Call-free infinite loops: oracle faults, native spins (MEDIUM)

A call-free spin loop (while f.* == 0 {}) exhausts the oracle's model stack
(it allocates a temp per iteration) and faults bad_pointer, while native
binaries — whose per-statement state stays flat — spin forever. Both
directions of the asymmetry are pre-existing; it also bounds what the
orphan-fix's unconditional fault-path joins can promise (a non-terminating
task hangs the join, matching the already-shipped normal-path join). Fix
directions to weigh: an iteration budget in the oracle mirroring MAX_ADDR
semantics deliberately, or documenting the asymmetry as a model limit.
Surfaced by the orphan-lifecycle review, 2026-08-26.

# Ledger additions from the P5-fix adversarial review (2026-08-26)

## P19 — Array literal as an index BASE escapes grounding (CLOSED 2026-08-30, MEDIUM, pre-existing)

`[[1, 2], [3, 4]][0]` into a `[2]u8` slot still silently narrows on the
oracle (the base literal is materialized at the i64 stride while the slot
copies u8); MIR/native refuse the shape. An index base is a non-propagating
position (deliberately — the slot's element type does not describe the
base), so the P5 grounding never sees it, and the base's own landing is a
temporary, not a binding. Pre-existing on both binaries; stays open as its
own item, not folded into P5's closure.

CLOSED by the P21/P19/P23 workstream: an rvalue used as a place base lands
in a temporary — a non-propagating landing — so `check_place`'s rvalue
fallthrough now grounds composite `{integer}` to the i64 default
(`ground_nested_int_lit`), exactly the unannotated-`let` rule. This
tightens the WHOLE rvalue-index-base family, not just the nested-literal
shape: a scalar element out of an unsuffixed literal base is i64 now too
(`let x: u8 = [1, 2][0];` is E0703 where it previously waved through). The
`[2]u8` slot is E0703 on every engine (both front-ends); the grounded
`[2]i64` form runs to the right values on the oracle, while MIR/native
keep their pre-existing loud "unsupported place" subset refusal for rvalue
index bases — a refusal, never a wrong value. Note:
`run.rs::array_literal_index_base_reads_at_grounded_i64_stride` is a VALUE
guard on the oracle's grounded read, not evidence the engines discriminate
the shape — the discrimination evidence is the E0703 rejection tests.

## P21 — Scalar literal range checks ignore the expected type, both directions (CLOSED 2026-08-30, HIGH)

Queued fix workstream, not fixed by the P5 work (which covers arrays only).
Direction 1: `let x: u8 = 300;` checks clean and stores 44 on every engine —
silent truncation, violating spec 01 §3.3 ("an over-range literal is
rejected at compile time"); same through a binding (`let x = 300; let y: u8
= x;` is 44). Direction 2: an in-range `u64` maximum literal is REJECTED,
because `check_int_lit_range` runs against the `i64` default at literal
sight, before the expectation is known — the same root defect pointing the
other way. Fix belongs where the expectation is applied (`check_against`),
mirroring what the array elements now get.

CLOSED for the literal positions (spec-compliance fix, not a new rule):
`check_int_lit_range` now resolves an unsuffixed literal's required type
from the propagating expectation when it is an integer scalar, i64 default
otherwise — sharing `scalar_range` with the array path. Two disciplines
keep the expectation honest at the literal: F1 clears it at
operand/index/builtin positions, and block-statement boundaries clear it on
entry (`check_block_stmts`/`check_block_value`/`check_scope`) — the latter
added after the adversarial review's B1 falsified the original "F1 already
clears every non-propagating position" claim (the expectation used to leak
into statements inside arm/branch block bodies, admitting a bare over-i64
literal into an inner unannotated `let` and wrongly range-checking inner
lets — including array literals, a pre-existing leak — against the outer
slot). Both directions flip: over-range-for-slot is E0709 in lets, args,
struct fields, enum payloads, returns, statics, and element assignments;
`u64`/`usize` slots accept literals up to `u64::MAX` WRITTEN DIRECTLY IN
THE SLOT and store them bit-exactly — the `u64_max_literal.cnr` fixture IS
the five-engine evidence (oracle, MIR, native-noopt, native-opt, AOT/LLVM
via the fixture-scanning gates), alongside the stage-D corpus entries —
while comparisons, operands, and `conv` sources still take the i64
default. The lexer already carries
full u64 (`u64::from_str_radix`) — no second bug there. The
through-a-binding residual is carried forward as its own open item, P25.
Constant FOLDS (`let x: u64 = -(1);`, `let y: u8 = 200 + 100;`) are
compounds, not literals, and keep today's behavior everywhere but array
elements (whose pre-existing fold re-check is retained); the verifier
confirmed the arithmetic-fold shape faults loudly at runtime (overflow)
rather than truncating. RESIDUAL: argument positions not routed through
`check_against` inherit the old gap — `push(s, c)`'s char argument is
checked `expect_integer`-only, missing the `u32` required-type range
check, and `spawn` arguments bypass the argument type check entirely (a
larger pre-existing hole, logged as its own item: P27).

# Ledger additions from the P7/P8/P11-fix adversarial review (2026-08-26)

Pre-existing holes found while attacking the P7/P8/P11 fixes. All reproduce
on the pre-fix binary, so none are regressions of that work; each is logged
to be fixed in its own workstream.

## P22 — Generic bodies are Proj-opaque and generic calls shed all loans (PARTIALLY CLOSED 2026-08-30, HIGH; App-of-Proj residual OPEN as P9)

Two halves of one family. (a) Generic function bodies are checked ONCE at
the definition site with opaque type parameters, so `Type::Proj`
(`I::Item`) defeats all three of the P7/P8/P11 fixes inside a generic body:
an assoc-typed method return sheds the receiver loan (`fn leak[I: Get](it:
write I) -> I::Item` — reviewer repro k3), an `out I::Item` slot escapes
provenance (repro l2), and a match joining to a Proj result sheds its arm
loans (repro s1). (b) Independently, `check_generic_call` ends with an
unconditional `clear_carried`, so EVERY generic free function returning a
borrow or view sheds its argument loan at the call site — `fn idr[T](p:
read T) -> read T` lets the caller write the borrowed owner and observe it
(repro q1 runs to 9 through a "live" shared borrow). All pre-existing; the
CONCRETE halves of these shapes are closed by the P7/P8/P11 workstream. The
generic halves need per-instantiation borrow information or def-site Proj
bounds — the next major checker workstream.

PARTIALLY CLOSED. (b) was implemented in the P22(b) workstream (memo
docs/reviews/2026-08-30-generic-borrow-opacity-design.md §4): the
substituted-type return-borrow extension closes the caller-visible halves
of k3/l2/s1 and q1 (== P16). The BARE-PROJ def-site shapes of (a) are
closed by the Proj-only conservative rule the deciding authority ratified
2026-08-30 (memo §2 option (i); spec 04 §7.6): inside generic code an
opaque projection (`I::Item`, and arrays of it) is treated as potentially
borrow-storing by the loan machinery and the signature rules, via the
named predicate `may_store_borrow` (E1006 and E0201 keep the unwidened
`field_stores_borrow`). Verified closures: l2's def-site dead-frame store
rejects E0806, k3-internal rejects E0801, the s1 match-join twin rejects
E0801, and leak2's missing def-site backstop rejects E0807 exactly like
its concrete twin (tests `p22_*` in compiler/tests/generics.rs; the former
open-hole lock-in is flipped to its rejection twin). The assoc-method
provenance extension keeps the single-borrow-in accessor idiom legal, and
the corelib iterator stack (owned Items) checks clean. Measured
over-rejection on the 370-subject corpus (fixtures + selfhost + ports +
corelib, selfhost, and p20-reference trees): zero diagnostic diffs.

OPEN residual (implementation adversarial review, 2026-08-30): the rule is
bare-Proj-only by scope, and a WRAPPED projection (`Opt[I::Item]`,
`W[I::Item]`) still launders a borrow-bound Item — `fn wrap[I: Get](it:
read I) alloc -> Opt[I::Item] { return Opt::Some(it.get()); }` checks
clean and runs (caller observes 9 through the wrapped "live" borrow; a
struct variant smuggles a WRITE borrow, both aliases live, runs to 42).
The memo's original completeness argument ("E1006 bars borrow-kind
arguments to generic enums") is wrong at constructor/annotation positions
— that is ledger P9, reopened HIGH with both repros inlined there. Locked
in as accepted-today tests `p22_open_hole_app_of_proj_*`; the candidate
predicate extension through App was prototyped and measured (memo, "App
extension, measured"): it rejects the shipped corelib iterator tree, so
the residual needs either constructor-aware provenance or the P9/E1006
closure, under its own ruling.

## P23 — Checker panic on a borrow-returning call with too few arguments (CLOSED 2026-08-30, MEDIUM)

`fn g(p: read i64) -> borrow i64` called as `g()` panics the checker
(index out of bounds in `check_user_call`'s return-extension, currently
`check/expr.rs:1697`: `per_arg` is zip-truncated to the argument count but
`region_source_indices` indexes by parameter position) AFTER the correct
E0706 arity diagnostic is queued. Reviewer repro i2. One-line `.get` fix in
its own change; a panic, not an unsoundness.

CLOSED: the return-extension indexes `per_arg` with `.get`, so the queued
E0706 is delivered. The other `per_arg` consumers were audited: the
fn-pointer and out-slot paths already `.get`, the method path indexes a
vector built by the same zip (in bounds by construction), and the generic
free-fn path returns early on arity mismatch.

## P24 — extern/fn-pointer signatures skip the out-slot E0807 rule (LOW)

The P7 signature rule (a borrow-storing `out` parameter with two-plus
borrow inputs is E0807) runs where function bodies are checked, so a bare
fn-pointer TYPE or an extern declaration with that shape is not itself
rejected. Not exploitable from safe code: externs cannot declare borrow
types (the foreign mappability check rejects them: "a Candor borrow/mode is
not a C type") and any Candor callable matching
the shape is rejected at its own declaration, so no callee can exist; the
caller-side extension also refuses to guess with two-plus inputs. A latent
asymmetry to close if fn-pointer signatures ever get their own
well-formedness pass.

# Ledger additions from the 2026-08-30 fix round (P21/P19/P23 verification and P22(b) review)

## P25 — Scalar `{integer}` flexibility through a binding still truncates (MEDIUM)

Split out of P21's closure (its "through a binding" example): `let x = 300;
let y: u8 = x;` checks clean and stores 44 on every engine. A bare scalar
`{integer}` deliberately keeps its slot flexibility through a `let` (the
documented ASYMMETRY with composites from the P5 work, which grounds only
composite-interior literals at an unannotated binding), but the binding's
VALUE is not tracked, so a narrower later slot truncates silently — the
same user-visible symptom P21 opened with, one binding removed. Closing it
means either grounding bare scalar `{integer}` to i64 at the binding
(making the `u8` use E0703, and breaking the currently-legal `let x = 1;
let y: u8 = x;`) or tracking constant values through bindings — a language
ruling, not a mechanical fix. Surfaced by the P21 closure's verifier,
2026-08-30.

## P26 — Generic calls never push a call group, so same-call overlap is unchecked (CLOSED for free generic calls 2026-08-30, MEDIUM; method path split out as P28)

Found by the P22(b) adversarial review (2026-08-30): `check_generic_call`
captures per-argument loans (and the P22(b) fix now extends the return
loan from them) but, unlike `check_user_call` and the fn-pointer path, it
never calls `push_call_group`, so the §3.1 no-two-phase rule does not run
for generic calls. `fn g[T](a: write T, b: read T)` called as
`g(write x, read x)` is E0805 on the concrete twin and checks CLEAN on
the generic one (repro verified on the post-P22(b) binary; pre-existing —
the P22(b) fix neither created nor touched this path). A mirror gap of
the same family as P22(b), likely a one-call fix at the same site, but
logged as its own item so it gets its own tests and sweep rather than
riding the P22 change. Cross-reference: P22.

CLOSED FOR FREE GENERIC CALLS ONLY (same 2026-08-30 workstream as the
P22(a) rule, separate tests and sweep verification): `check_generic_call`
pushes the per-argument loan group right after the argument loans are
captured — the exact `check_user_call` placement, before the out-slot
extension. The ledger repro rejects E0805, the distinct-owners twin stays
clean, and the out+read overlap twin rejects like its concrete
`out_and_read_overlap` (tests `p26_*` in compiler/tests/generics.rs; the
`synth_arg_type` probe already truncates probe-pushed groups, so no
duplicate diagnostics). Corpus sweep: zero diagnostic diffs. The
adversarial review found the SAME gap on the interface-method call path
(`check_iface_method_call`), which this round did not fix — logged as
ledger P28 below.

## P27 — `spawn` arguments bypass the argument type check entirely (HIGH, pending triage)

`fn t(v: u8) -> unit { trace(conv i64 (v)); }` spawned as `spawn t("x")`
inside a `scope` CHECKS CLEAN — on the pre-P21 compiler and the current one
alike (pre-existing, found while closing P21's literal-range gaps).
Mechanism: `check_spawn` resolves the callee's parameter modes itself and
routes each argument through `gate_spawn_arg`
(check/concurrency.rs), which calls `check_expr`/`check_place` directly
and NEVER `check_against` — the take-mode ownership-transfer branch gates
only PORTABILITY (`non_portable_witness`) and the borrow branch gates the
referent, so the argument's type is never compared to the declared
parameter type. This is a full argument-type-check bypass (a `str` where
`u8` is declared), not just a literal-range gap, and is potentially
memory-unsafe for wider/narrower or pointer-bearing mismatches. Observed
at runtime (2026-08-30): the oracle RUNS the repro and the task reads the
`str` argument as `u8` value 0 — silent wrong value, no fault; MIR and
native currently refuse this particular repro only for the unrelated
"expr string-literal outside subset" limit, so the oracle is the engine
that executes the confusion today. Numbered past P26 (claimed by the
in-flight generics worktree). Needs its own reviewed fix (route spawn
arguments through `check_against` before the portability/loan gates); NOT
fixed in the P21 round by review instruction.

# Ledger additions from the P22(a)-implementation adversarial review (2026-08-30)

## P28 — Interface-method calls never push a call group, so same-call overlap is unchecked (MEDIUM)

The method-path sibling of P26 (whose fix covers free generic calls only):
`check_iface_method_call` (check/generics.rs:1158) captures per-argument
loans into `per_arg` but never calls `push_call_group`, so the §3.1
no-two-phase rule (E0805) does not run for interface-method calls —
`s.fill2(out x, out x)` checks CLEAN where the concrete free-fn twin
`g(out x, out x)` is E0805 (verified 2026-08-30; the generic free-fn twin
also rejects since the P26 fix). Additionally `recv_carried` (the
receiver's loan, captured separately at :1179) is excluded from `per_arg`,
so a receiver-vs-argument overlap would be unchecked even with a group
pushed — a complete fix must fold the receiver loans into the group.
Repro:

    interface Two { fn fill2(read self, a: out i64, b: out i64) -> unit; }
    struct S { v: i64 }
    impl Two for S { fn fill2(read self, a: out i64, b: out i64) -> unit
                     { a = 1; b = 2; } }
    fn main() -> i64 { let s: S = S { v: 1 }; let mut x: i64 = 0;
                       s.fill2(out x, out x); return x; }

NOT fixed in the P22(a) round by review instruction; needs the same
push-after-capture mirror plus the receiver fold, with its own tests.
Cross-reference: P26.

## P29 — Interface methods with `read`/`write`-mode value parameters double-lower the parameter type (HIGH, pre-existing)

Found by the P22(a) adversarial review while probing method-call shapes:
an interface method declaring a borrow-MODE parameter is unusable —

    interface One { fn m(read self, a: read i64) -> i64; }
    // impl for S accordingly...
    fn main() -> i64 { let s: S = S { v: 1 }; let x: i64 = 2;
                       return s.m(read x); }

gives E0703 `type mismatch: expected `borrow borrow i64`, found `borrow
i64`` at every call site (verified 2026-08-30, on the pre-P22(a) binary
and the current one alike). Mechanism: interface-method resolution stores
each parameter as `(mode, lower_param(mode, ty))` (crate::generics
iface lowering, generics.rs:226) — the TYPE is already mode-lowered
(`read i64` -> `borrow i64`) while the mode is kept — and
`check_iface_method_call` then re-applies the mode through
`check_arg_mode(*mode, &pty, ..)`, whose `Read` arm expects a borrow OF
the declared type, i.e. `borrow borrow i64`. Free-fn `ParamInfo` keeps
`decl_ty` unlowered with a separate `lowered` field; the iface path
conflates the two. Every shipped interface method takes value params by
`take` (or only `self` by mode), so the corpus never trips it. Fix is to
store the unlowered type in the pair (or lower at binding time only), with
call-site tests over read/write/out modes; NOT fixed this round.
