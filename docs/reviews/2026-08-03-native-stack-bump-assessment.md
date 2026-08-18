# Native stack-bump no-rollback assessment (task #144)

Assessment of `rt_stack_alloc`'s never-roll-back design (the task-shared atomic
bump in `compiler/src/backend/runtime.rs`, mirrored in `aot_runtime.c` and
`freestanding_runtime.c`) and what it means for long-running native programs.
All numbers below are measured in this session on the current tree (Cranelift
AOT and LLVM `-O2` AOT builds; the JIT shares the Cranelift lowering).

## Verdict, in one paragraph

Every native call leaks model stack forever, and for the shipped showcases the
budget is small: the **compiled HTTP server dies after ~2,400 requests (LLVM
-O2) or ~630 requests (Cranelift)** — under one second of any load test — and
the **wasm interpreter corrupts its own heap window after ~343k interpreted
instructions (LLVM) or ~29k (Cranelift)**. The failure is silent data
corruption first (the bump grows *through* the user's allocator window,
zeroing live heap), then a host SIGSEGV from wild `rawptr` chains or from the
256 MiB model edge. The shipped examples as documented are safe only by
accident: the server caps itself at 8 requests and the README runs it on the
(reclaiming) tree-walker. A minimal guard is now in place: crossing the model
edge is a clean `bad_pointer` fault instead of host-level UB. The guard cannot
prevent the window corruption — only rollback can — and a rollback design is
recommended below (caller save/restore gated on live-task count) but not
implemented.

## 1. What actually consumes `rt_stack_alloc`

The two backends differ materially.

**Cranelift (`lower.rs::lower_fn`): every local of every call.** Each MIR
local — scalar temporaries included — gets a flat-model slot at function
entry (`lower.rs` ~line 1400). There is no tiering. Regalloc spills go to the
host machine stack, not the model; the model cost is exactly the frame's
locals.

**LLVM (`llvm.rs::classify_tiers`): only Tier-F locals.** Wordy locals
(scalars, borrows, rawptrs, fn-ptrs) become `alloca`s that mem2reg promotes —
zero model cost. Tier-F locals call `rt_stack_alloc`: every non-wordy type
(structs, enums — including field-less enums like `Ordering` — arrays,
`Vec`/`String`/`Map` headers, fat slice values) plus any scalar whose address
is taken (`Ref`, `CopyVal`, box/subslice/collection operands).

Neither backend allocates at the call *site*: aggregate returns are written
into the callee's own `_0` slot and copied down into a caller slot that was
allocated at the caller's entry. Interface-bound calls (`T: Ord`) resolve to
ordinary monomorphized calls; the dispatch itself costs nothing, but the
callee frame (and its `Ordering` slots) does.

Measured per call, 10k-iteration probe (`scratchpad/probe.cnr`, address of a
fresh frame local read via `addr_of`):

| call shape | Cranelift AOT | LLVM -O2 AOT |
|---|---|---|
| `fn(i64, i64) -> i64`, scalar body | 32 B | 0 B |
| scalar fn with one 24 B struct local | 48 B | 24 B |
| fn returning a 24 B struct | 32 B | 24 B |
| generic `T: Ord` compare (2-frame chain, `Ordering` results) | 136 B | 24 B |

Both interpreters reclaim eagerly since 2026-07-25; the leak is native-only.

## 2. Budget for the shipped showcases

Layout constants: bump starts at `STACK_BASE` = 1 MiB; every example places
its free-list window at 16 MiB (base 16777216; the wasm example's outer window
at 32 MiB); model edge `MAX_ADDR` = 256 MiB. So the distance from bump start
to the first heap window is ~15 MiB, and to the edge ~255 MiB.

**HTTP server (`dist/examples/12_http_server`).** Instrumented build printing
the bump every 100 requests:

* LLVM -O2: **6,640 (independent re-measurement in adversarial review: 6,296; treat as ~6.3-6.6 KB) B per request** — roughly 100–150 aggregate-slot calls per
  request (String/Vec-heavy parse and response path). Requests to window:
  (16 MiB − 1 MiB − startup) / 6,640 ≈ **2,370**. Predicted 2,370; the
  *unmodified* handler (only `serve_n` raised) died at **request 2,371**,
  SIGSEGV (exit 139), first bad response = the empty reply of the dying
  process.
* Cranelift: **~25,000 B per request**. Died at **request 629** (first bad
  response: a spurious 404 from a corrupted file-name String, then dead).

Would a load test hit it? **Yes, immediately, and there is no 10x margin — the
margin is negative by orders of magnitude.** The sequential Python driver used
here sustained ~17k req/s locally, killing the server in 0.14 s. Even a polite
`curl` loop at 200 req/s kills it in 12 s. A 10-minute `ab`/`wrk` run at any
plausible HN-thread rate (say 100–1,000 req/s) oversubscribes the entire
lifetime budget by **~25x (LLVM at 100 req/s) to ~950x (Cranelift at
1,000 req/s)** — and an unthrottled local `wrk` (this session's driver
sustained ~17k req/s) by 3.6 orders of magnitude for LLVM (4.2 for Cranelift). The only reason the *shipped* artifact
cannot be killed this way is that `serve_n = 8` and the README says
`candor run` (tree-walker, which reclaims). Anyone who does the natural thing
— compile it natively and raise the cap to demo performance — ships a server
with a ~2,400-request lifetime.

**wasm interpreter (`dist/examples/11_wasm_interp.cnr`).** Instrumented with a
counted-loop module (5 instructions/iteration):

* LLVM -O2: **45.8 B per interpreted wasm instruction** (500k instructions
  leaked 22.9 MB). The run **crossed its own 16 MiB window at ~343k
  instructions and still returned the correct result** — silent corruption,
  observable only because that window happened to hold nothing live.
* Cranelift: **545 B per instruction**; instructions-to-window ≈ **29k**; the
  500k-instruction run hit the 256 MiB edge and (pre-guard) died SIGSEGV with
  all trace output lost.

Any non-toy module (millions of instructions) exhausts the model in well under
a second of interpretation.

## 3. Why corruption comes before any fault

Map of the flat model (one 256 MiB buffer, Candor address = offset):

```
0x0000_0000  NULL
0x0000_1000  STATIC_BASE   statics + string literals (baked, small)
0x0010_0000  STACK_BASE    the bump; grows up, never rolls back
0x0100_0000  16 MiB        the examples' free-list window (user convention,
                           4-8 MiB; wasm's outer window at 32 MiB)
0x1000_0000  MAX_ADDR      model edge
```

Three compounding facts:

1. **The runtime has no idea the window exists.** "Heap" is a user-level
   convention (`with_window(16777216, ...)` + an `unsafe` justification that
   the window "is reserved to this arena alone"). `rt_stack_alloc` compares
   against nothing; the invariant the unsafe block asserts is violated by the
   runtime itself once the bump arrives.
2. **The window sits 15 MiB up; the edge 255 MiB up.** The bump reaches live
   heap ~17x sooner than the edge — for every placement below the edge there
   is a corruption phase before any crash. Placing the window high (the
   introsort bench used 192 MiB) only stretches the phase's onset.
3. **The first touch is a `memset`.** `rt_stack_alloc` zeroes each slot, so
   crossing the window actively wipes free-list block headers and live
   payloads. Failures then cascade *below* any check: native `ptr_read`/
   `ptr_write` are raw machine ops, so a corrupted free-list `next` pointer
   becomes a wild host access — the observed SIGSEGV at request 2,371/629 —
   and in the JIT the flat buffer is a `Vec` on the host heap, so an overrun
   scribbles the test harness process itself.

This is exactly the introsort-bench incident generalized: `bad=14209`
checksums first, segfault later, nothing attributable in between.

## 4. The guard implemented now (Part 2)

`rt_stack_alloc` in all three runtimes now faults cleanly instead of writing
past the model edge:

* `backend/runtime.rs` (JIT), `backend/aot_runtime.c` (hosted AOT),
  `backend/freestanding_runtime.c`: if `size != 0` and the new frontier would
  exceed `MAX_ADDR`, call `rt_fault(bad_pointer, 0, 0)` — the same kind and
  span the MIR interpreter delivers for the first touch past the model
  (`Mem::ensure` → `BadPointer` at `Span::point(0)`). Size-0 reservations
  stay fault-free, matching the interpreters' lazy check.

Semantics: **byte-identical for every program that fits the budget** (the
check is one predictable branch on the already-computed frontier; the full
differential suite is green). Programs that previously segfaulted (or
scribbled the host heap) at the edge now exit 2 with attributable fault JSON —
verified on all four engines, e.g. the wasm probe that died `exit=139` now
prints `{"kind":"bad_pointer","span":{"start":0,"end":0},...}` and even
flushes its trace. Four regression tests pin this (`stage_b.rs`, `aot.rs`,
`llvm.rs`, `freestanding.rs`: `*stack_exhaustion*`); they would die SIGSEGV
on any regression, not just fail.

**What the guard does not do — plainly:** it does not prevent the window
corruption of §3, because the runtime cannot know where user windows are. The
guarded endurance server still dies SIGSEGV at request 2,371. The guard turns
*pure* exhaustion (no window in the bump's path) into a clean fault; programs
with a heap window still corrupt first. Two placement notes follow from the
layout: a window placed entirely **below** `STACK_BASE` (the ~960 KiB between
the statics and 1 MiB) is never crossed and gets fault-before-corruption
guaranteed under the guard; any window above `STACK_BASE` only delays it.

## 5. Rollback options compatible with the task-shared invariant (Part 1.4)

The invariant to preserve: `scope`/`spawn` tasks share one atomic bump, and
live frames of concurrent tasks must stay disjoint. The interpreters' 2026-07-25
reclamation also established the key lowering invariant already in the tree:
an aggregate call's `Assign` and its consuming `CopyVal` are adjacent with no
allocating statement between.

**A. Per-task bump regions.** Each task thread bump-allocates inside its own
chunk (chunks carved from a shared atomic frontier; new chunk when full),
freed wholesale at join; within a thread, caller watermark save/restore around
calls. *Soundness:* chunks are disjoint by construction; within a thread the
MIR interpreter's watermark argument applies verbatim (only the return value
outlives a call, and it is copied down before the pop; borrows cannot be
returned; a spawned task's operands live in ancestor frames that survive the
scope by the loan rules). *Complexity:* highest — runtime chunk management +
both lowerings + spawn/join paths. *Tests:* chunk exhaustion/reuse,
cross-task aggregate args, fault-unwind watermarks, plus the existing
concurrency differential. Also removes the per-call CAS and makes leak-while-
tasks-live bounded per task.

**B. Epoch reset at scope joins.** Record the bump at `rt_scope_begin`,
restore it at `rt_scope_end` after all joins. *Soundness:* every frame
allocated during the scope (by parent or any task) is dead at the join —
returns were copied down, tasks have exited; sound by the same watermark
argument at scope granularity. *Complexity:* trivial (two runtime hooks).
*But:* reclaims nothing for scope-free programs — which is every shipped
showcase. Not sufficient alone.

**C. Caller save/restore gated on "no live tasks".** The caller reads the
bump before each call and restores it after the call's return copy, iff a
global live-task counter is zero (`rt_spawn` increments; the join decrements).
*Soundness:* when the counter is zero exactly one thread exists, so the
check-then-store cannot race (only a running thread can spawn); everything
above the saved watermark is a returned frame, dead by the same argument as A;
when tasks are live all restores are skipped and today's behavior — whose
disjointness proof is untouched — applies verbatim. After a scope joins, the
counter returns to zero and the *enclosing* call's restore also reclaims the
scope's dead task frames. Programs leak only while tasks are actually live.
*Complexity:* low-medium — a counter in the three runtimes plus a few
instructions per call site in both lowerings (load counter, branch, store);
no new memory structures. *Tests:* the full differential corpus referees the
single-threaded equivalence (native comes to match the reclaiming
interpreters, so any program a restore could break is already broken on the
interpreters); add a native bump-parity probe (per-call growth == 0 for the
§1 shapes), a spawn-inside-call scenario (restore skipped while the counter
is nonzero), and re-run the concurrency gates.

**D. Hybrid: C now, A when concurrency-heavy long-running programs matter.**

**Recommendation: C.** It is the smallest reviewed change that fixes the
actual exposure (every showcase is single-threaded), keeps the task-shared
fast path byte-identical when tasks are live, and is refereed by gates that
already exist. B can ride along for free if wanted; A is the eventual
destination and subsumes both. None of this is implemented here — it gets its
own reviewed change.

## 6. Evidence trail

* Probes and load tests: session scratchpad (`probe.cnr`, `httpbench/`,
  `wasmbench/`) — instrumented *copies*; no shipped example was modified. The
  scratchpad is ephemeral (session-local `/tmp`); every headline number above
  was independently reproduced by a fresh-context verifier before this memo
  was finalized, and §1/§2 describe the probe constructions well enough to
  rebuild them (an `addr_of` bump probe plus a raised-`serve_n` server copy).
* Guard: `compiler/src/backend/runtime.rs`, `aot_runtime.c`,
  `freestanding_runtime.c`; tests in `compiler/tests/{stage_b,aot,llvm,freestanding}.rs`.
* Full `cargo nextest run`: 1249/1249 passed. `cargo clippy --all-targets`:
  clean.
