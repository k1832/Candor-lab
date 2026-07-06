# Adversarial review #1 of design 0001 — memory model (draft)

**Date:** 2026-07-06
**Reviewer:** independent LLM session (heavy reasoning tier), hostile brief targeting soundness of the safe subset, internal consistency, P5/NN compliance, Bet 5 measurement validity, and implementability.
**Verdict:** architecture sound (three gears, NLL-lite, effect partition); not yet implementable-without-questions. Four findings are missing semantics the document's own worked examples depend on; two are soundness holes reachable from safe code. Fixes are additive and local; no Non-Negotiable is reopened.

Dispositions recorded inline as **[Disposition: …]** by the deciding authority's session.

---

## 1. BLOCKER — `match` pattern-binding mode undefined; examples depend on two incompatible modes (§8.2 vs §11.4/§11.5)

§11.4 needs payloads bound **by move** (owned scrutinee, `lhs` moved out); §11.5 needs **by copy/borrow** (borrowed scrutinee). Neither rule exists; always-move rejects one example, always-borrow the other.

**[Disposition: accepted as proposed.** A pattern on an owned scrutinee binds payloads by the value gear (move, or copy if the payload type is `copy`), and a move binding partial-moves the scrutinee under §1.6's rules (hence: no move bindings out of a `drop`-hooked type, and joins must agree). A pattern on a `read`/`write`-borrowed scrutinee binds each payload as a `read`/`write` borrow of the corresponding sub-place, subject to the ordinary loan rules. Nested patterns compose the same rules per level.]

## 2. BLOCKER — moving out of a place with a live loan is not rejected (§1.6 vs §2.2)

`let b = read x; let y = x; use(deref b);` — a move is neither a "write" nor a "borrow" in §2.2's consequence list, so nothing rejects it; `deref b` then reads moved-from storage. NN#5 violation in safe code.

**[Disposition: accepted.** A move out of a place (whole or partial) is classified as an **exclusive access** to that place for §2.2's conflict check: it conflicts with any live loan on that place or any overlapping place. Same rule for direct writes/reassignment, stated explicitly.]

## 3. MAJOR — `out` lives outside the aliasing model; pre-initialized slot and "exactly once" unspecified (§3.1)

`f(out x, write x)` / `f(out x, read x)` create two paths to one slot with no loan seeing the overlap. Drop-of-existing-value on `out` unspecified; "assigned exactly once" bans loop-assignment patterns or silently means "at least once."

**[Disposition: accepted with one modification.** (a) `out place` produces an **exclusive loan** on the place for the duration of the call, conflicting with any other argument that touches an overlapping place. (b) Instead of requiring the slot to be statically uninitialized: passing an **initialized** place as `out` first drops the current value at the call site, before the call — the same visible rule as reassignment (§1.5). On a fault mid-call nothing further executes (abort policy), so no drop question arises. (c) "Exactly once" is replaced by: *definitely assigned on every normal-return path, and never read before its first assignment* (loop re-assignment is ordinary reassignment and is permitted).]

## 4. MAJOR — Alloc-outlives-every-Box obligation never stated; safe code frees through a dangling ctx (§6.1–6.2)

Constructing an `Alloc` from a local, returning a `Box` served by it, and dropping the Box later is all-safe code that frees through dead storage.

**[Disposition: accepted.** Constructing an `Alloc` value is the unsafe act that carries the obligation, stated in the doc: the justification must assert that `ctx` and `vt` remain valid for the lifetime of every copy of the handle and every `Box` it serves. The checker cannot verify it; that is what the valve means. The worked example's justification strings are updated to carry it.]

## 5. MAJOR — `as` casts used throughout §11 but never defined; no integer conversion exists though P2 bans implicit conversion (§4.2, §8, §11)

**[Disposition: accepted.** An explicit integer-conversion form is added (throwaway syntax `conv T (e)`): widening is always value-preserving; narrowing or sign-changing conversion **faults on value loss** under the default regime, truncates two's-complement inside a `wrapping` block, saturates inside `saturating`. Pointer-side `as` uses in the examples are deleted — `cast_ptr`/`addr_to_ptr` already produce the target type and gain an explicit target-type annotation form. All §11 examples corrected.]

## 6. MAJOR — Box clone self-contradictory (contains rawptr via Alloc handle) and clone's allocator source unstated (§1.4, §6.2)

**[Disposition: accepted.** The structural rawptr-not-cloneable rule applies to user types only; compiler-known `Box` is cloneable by copying its stored handle, allocating through **that stored handle**, and cloning the pointee. Stated in both §1.4 and §6.2.]

## 7. MAJOR — fn-pointer types carry no effect; the alloc partition is bypassable through an indirect call (§6.1, §6.3)

**[Disposition: accepted.** Effect markers (and parameter modes) are part of the fn-pointer **type**; assigning an `alloc` function to a non-`alloc` fn-pointer type is a checker error; an indirect call's effect is read from the pointer's type. The vtable special-case rule is deleted; `AllocVtable`'s fields are declared with `alloc`-marked fn-pointer types (`free` included, conservatively — an allocator's own bookkeeping may allocate).]

## 8. MAJOR — qualified variant syntax `Type.Variant` needs a symbol table to parse (violates NN#13 and §8.2's own grammar) (§8.2 vs §11)

**[Disposition: accepted, resolved by syntax.** Variant construction and patterns use `Enum::Variant(...)`; the `::` token is reserved for enum-variant reference only, so `Ident :: Ident` is position-disambiguable with no symbol table. Variants are enum-scoped. §8.2 grammar and all §11 examples updated to agree.]

## 9. MAJOR — shared reborrow of an exclusive borrow (used in §11.5) undefined; parent-freezing rule unstated (§2.1)

**[Disposition: accepted.** `read (deref b)` on an exclusive `b` is defined: it creates a shared loan on the pointee and **freezes `b` to shared** for the reborrow's live range (no writes through `b`, no exclusive reborrows). General rule stated: every reborrow places a loan on the pointee that restricts the parent for the reborrow's live range — exclusive reborrow suspends the parent entirely; shared reborrow freezes it to shared.]

## 10. MAJOR (measurement validity, time-critical) — prototype valve set (unsafe-only) is a strict subset of P12's (checked-runtime OR unsafe); M2's absolute ceilings could false-KILL on an artifact of the dropped cell (mem §4.3; criterion §3.3/§4.2/§5.1)

**[Disposition: accepted, resolved on the criterion side + one honesty note here.** The memory-model doc keeps its single valve (§4.3's basket argument stands) but §4.3 now states plainly that prototype valve counts are an **upper bound** on real-Candor valve occurrence. The criterion gains (as a pre-freeze ledger amendment): port authors tag any unsafe region for which a P12 checked-runtime alternative would plausibly have sufficed ("cell-substitutable"); each tag is confirmed or rejected by the adjudicator with recorded public reasoning; M2 KILLs are evaluated on the **full** count, but if excluding confirmed cell-substitutable regions would change a KILL verdict, the outcome is **mandatory §9 review** (which may still escalate to KILL) rather than automatic KILL. The KILL keeps its teeth; a false KILL caused by the prototype's own scope cut gets a recorded human ruling instead of silence.]

## 11. MINOR/MAJOR — `wrapping`/`saturating` scope: textual vs. dynamic at call boundaries unspecified (§8.1)

**[Disposition: accepted.** Regime blocks apply **textually only** — to arithmetic operations lexically inside the block, never to callees. Stated explicitly.]

## 12. MINOR/MAJOR — `read slice T` (borrow of a copy borrow) used inconsistently; perturbs the annotation metric (§8.4, §11.4)

**[Disposition: accepted.** Slices and borrow-typed values are passed **by value** (shared slices/borrows are `copy`; exclusive ones move or are reborrowed at the call site). `read`/`write` parameter modes on a parameter whose type is already a borrow kind (borrow, slice, slice_mut) are ill-formed — one canonical way (P3), and the annotation metric stays deterministic. Examples fixed.]

## 13. MINOR — worked examples not mechanically checked; dead `struct P`; typo (§11.4, §2.3)

**[Disposition: accepted.** Dead struct deleted, typo fixed, and a standing obligation added to the doc: every §11 example becomes a test case of the prototype checker, and the document is re-verified against the implementation before the criterion's freeze step (i).]
