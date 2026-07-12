# A tour of Candor

This is the pitch, not the spec. If you write systems code and you're curious
what Candor actually feels like, read this top to bottom — every snippet runs on
the 0.x toolchain (`candor run`). For the normative rules, see the spec that
ships alongside; for the *why*, see the lab's philosophy and design docs.

Candor's premise: code is now written by a human and an LLM together, and the
expensive step is no longer writing code but *verifying* it. So every feature is
chosen to make a function checkable by reading it and its signatures — locally,
without holding the whole program in your head.

## Values first

A Candor program is `fn main() -> i64`, returning a sentinel. Values are owned and
move by default; there is no garbage collector and no hidden reference counting.

```candor
fn main() -> i64 {
    trace(1);          // `trace` records values in order (visible in a compiled binary)
    trace(2);
    return 42;         // the program's result
}
```

Memory safety here is not "a borrow checker bolted on." It is the founding bet —
*Bet 5* on the lab's public record — that a value-first model (own it, move it,
borrow it briefly) carries less cognitive load than lifetime-annotated references,
while giving up nothing on safety. The bet was pre-registered with a kill
criterion, tested, and provisionally confirmed.

## The gears: borrow modes wear keywords

When you don't want to move a value, you lend it. The *mode* is written at the
call site and in the signature, so aliasing is visible without a symbol table:

- `read x` — a shared, read-only loan.
- `write x` — an exclusive loan you can mutate through.
- `out x` — an uninitialized slot the callee must fill.
- `take x` — hand over ownership (a move).

```candor
struct Acc {
    total: i64,
    count: i64,
}

fn add(a: write Acc, delta: i64) -> unit {
    a.*.total = a.total + delta;   // write path is explicit: `a.*.field`
    a.*.count = a.count + 1;       // read path auto-derefs: `a.count`
}

fn main() -> i64 {
    let mut acc: Acc = Acc { total: 0, count: 0 };
    add(write acc, 40);            // the `write` is at the call site too
    add(write acc, 2);
    return acc.total;              // 42
}
```

The explicit `.*` on the write path is deliberate: mutation is exactly where you
look when reviewing, so the language makes it greppable.

## Enums, match, and `?`

Errors are values. A *result-shaped* enum marks exactly one variant `ok`; `?`
unwraps that variant or returns the rest. `?` widens a narrow error into a wider
one through a `From` impl — no boilerplate at the call site.

```candor
enum IoErr { Eof, Bad }
enum AppErr { FromIo(IoErr), Denied }
enum Res[T, E] { ok Ok(T), Err(E) }

interface From[E] { fn from(e: E) -> Self; }
impl From[IoErr] for AppErr { fn from(e: IoErr) -> Self { return AppErr::FromIo(e); } }

fn read_byte(ok: bool) -> Res[i64, IoErr] {
    if ok { return Res::Ok(65); }
    return Res::Err(IoErr::Eof);
}

fn decode(ok: bool) -> Res[i64, AppErr] {
    let b: i64 = read_byte(ok)?;   // IoErr widens to AppErr right here
    return Res::Ok(b + 1);
}
```

## Generics and `for`

Generics are monomorphized and bounded (`copy`, `portable`, ...). A container
implements an iteration *protocol* and `for` walks it. `for x in read coll`
read-borrows and copies items out — allocation-free, the ground floor. (A
consuming `for x in coll` moves the collection; used for owning structures.)

```candor
struct Stack[T: copy] { mem: [8]T, count: u32 }

impl[T: copy] Indexed for Stack[T] {
    type Item = T;
    fn at(read self, i: usize) -> Opt[T] { /* ... */ }
}

// later:
let mut sum: i64 = 0;
for x in read s { sum = sum + x; }   // no allocation, interrupt-callable
```

## Effects: what a function may do, in its signature

Capabilities are effects, written on the signature and checked. `alloc` means
"may touch the allocator"; `foreign` means "calls undischarged FFI." A function
with no effect marker is the no-allocation, no-FFI floor — the part you can call
from an interrupt handler. The effect partition is *proven*, not documented: the
`core` library never allocates because the checker says so.

```candor
fn push_front[T](a: read Alloc, l: List[T], x: T) alloc -> Grow[T] { /* ... */ }
```

## Contracts

Preconditions, postconditions, and assertions are executable, analyzable clauses.
A violated `enforced` contract is a fault (below), routed through the root policy
— never silently assumed away by the optimizer.

```candor
fn half(x: i64) requires(x >= 0) ensures(result >= 0) -> i64 {
    return x / 2;
}
```

## The fault model

Arithmetic is *checked by default*: an operation that would overflow is a fault,
not a silent wrap. When you want modular arithmetic, you ask for it with a
`wrapping` block. A fault has a stable *identity* — a kind and a source span —
that is identical across the interpreter, the native JIT, and the AOT binary.
That per-target one-semantics is a core promise: the same program means the same
thing in every build mode.

```candor
fn main() -> i64 {
    let big: i64 = 9223372036854775807i64;
    return big + 1;    // fault: {"kind":"overflow", ...}, exit code 2, on every engine
}
```

Faults are the model's honesty: rather than undefined behavior or a wrap you
didn't ask for, you get a defined, located, reproducible stop.

## Structured concurrency

`spawn` a task only inside a `scope`; the `scope` joins every task before it
returns, so no task outlives the data it borrows. The checker proves the tasks
touch disjoint memory — here, two halves of a buffer via `split_mut` — so the
parallel writes are race-free *at compile time*. (The lab rejected an earlier
draft of this design because a reviewer built a genuine race against its own
flagship example; the fixed design is what ships.)

```candor
fn main() -> i64 {
    let mut buf: [4]u8 = [0u8, 0u8, 0u8, 0u8];
    let lo: write [u8];
    let hi: write [u8];
    split_mut(buf, 2, out lo, out hi);      // two disjoint exclusive loans
    scope {
        spawn fill(write lo, 1u8, 2);       // real OS threads, provably race-free
        spawn fill(write hi, 2u8, 2);
    }
    return conv i64 buf[0] + conv i64 buf[3] * 1000;
}
```

## Where to go next

- `examples/` — every snippet above, whole and runnable.
- `spec/` and `specpack/` — the normative rules and the model-facing
  distillation, both present at the repo root of this standalone distribution.
- `stdlib/` — the `core`/`std` seed (a module tree); `candor run stdlib`
  builds and runs it end-to-end.
- The lab repository — the philosophy, the twelve designs (each recording what was
  rejected and why), and the Bet 5 experiment. That is the authority; this preview
  follows it.
