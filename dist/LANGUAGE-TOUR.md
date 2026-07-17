# A tour of Candor

This is the pitch, not the spec. If you write systems code and you're curious
what Candor actually feels like, read this top to bottom. The pure-compute
snippets run as-is on the 0.x toolchain with `candor run`, and `examples/` carries
each one whole and runnable; I/O works under both `candor run` and `candor compile`
(the I/O section says how). For the normative rules, see the spec that ships alongside; for the
*why*, see the lab's philosophy and design docs.

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

## The standard library: collections, ordering, iterators, strings

Beyond the language there is a small, honest standard library — the `stdlib/`
module tree in this distribution (`candor run stdlib` builds and runs it
end-to-end). It is deliberately *pure-compute*: safe Candor over the allocator
handle, with no I/O (that story is below).

**Collections** are allocator-explicit. `Vec[T]`, the hash `Map[V]` (keyed by a
byte-string view — `str` or `[u8]`), and the growable `String` are built through
an `Alloc` handle you pass in; nothing allocates behind your back.

```candor
let mut v: Vec[i64] = vec_new(read a);   // `a` is an allocator handle
push(write v, 5);
push(write v, 3);
let n: usize = len(read v);
```

**Ordering** is an ordinary interface. `Ord` is impl'd for the scalar types in
its own module (a scalar has no home module of its own), so `[T: Ord]` bounds the
generic reducers and the in-place sort — the comparison dispatches through the
impl; nothing is compiler-blessed.

```candor
interface Ord { fn cmp(read self, other: Self) -> Ordering; }
impl Ord for i64 { /* ... returns Less / Equal / Greater ... */ }

let lo: Opt[i64] = min(read v);   // [T: Ord]
let hi: Opt[i64] = max(read v);
sort_ord(write v);                // ascending, dispatched through Ord::cmp
sort(write v, less);              // or comparator-driven: less: fn(read i64, read i64) -> bool
```

**Iterators** are two by-value protocols — a consuming `Iter` and a borrow-copy
`Indexed` — with a surface of lazy *adapters* and eager *terminals*. Each adapter
(`take_n`/`skip_n`/`enumerate`/`zip`/`take_while`/`skip_while`) owns its inner
iterator and composes; each terminal (`find`/`nth`/`count`/`any`/`all`/`fold`/
`collect`) drives the chain to close it.

```candor
// range(0, 10) |> take_n(5), summed by a terminal:
let sum: i64 = fold(take_n(range(0, 10), 5), 0, add);   // 0+1+2+3+4 = 10
// adapters compose: enumerate, then find the pair whose value is 11:
let hit: Opt[Pair[usize, i64]] = find(enumerate(range(10, 13)), is_eleven);
```

**Strings** carry the byte-scan utilities: `starts_with`/`ends_with`/`contains`/
`find` and the view-returning `trim` allocate nothing; `split` (into owned
`String` pieces) and `join` allocate through the handle.

```candor
let parts: Vec[String] = split(read a, "a,b,c", ",");   // ["a", "b", "c"]
let joined: String = join(read a, read parts, "-");      // "a-b-c"
```

See `examples/08_ordering.cnr`, `examples/09_strings.cnr`, and
`examples/10_iterators.cnr` — each reduces the relevant stdlib module to a single
runnable file.


## Effects: what a function may do, in its signature

Capabilities are effects, written on the signature and checked. `alloc` means
"may touch the allocator"; `foreign` means "calls undischarged FFI." A function
with no effect marker is the no-allocation, no-FFI floor — the part you can call
from an interrupt handler. The effect partition is *proven*, not documented: the
`core` library never allocates because the checker says so.

```candor
fn push_front[T](a: read Alloc, l: List[T], x: T) alloc -> Grow[T] { /* ... */ }
```

## I/O over the audited boundary

Files, directories, and TCP reach the host through the `foreign` effect and a
*boundary* module — the one place `extern "C"` may appear, where each foreign
function states a `trust` clause the checker cannot verify but records for
`candor audit` (see `examples/07_boundary`). A safe wrapper discharges the effect,
so ordinary callers use it without `foreign`.

Both ways of running a program do **real** I/O:

- `candor run` INTERPRETS, backing the boundary's `sys_*` calls with the host's
  real streams and filesystem — stdout/stderr/stdin, cwd-relative file opens,
  directory listing, `tcp_connect`.
- `candor compile` produces a NATIVE binary linked against a small C runtime that
  carries the same shims. The compiled binary does the same real I/O with no
  interpreter in the loop.

Either way the trust surface stays auditable: `candor audit` enumerates every
foreign extern a program reaches, and a `freestanding = true` package rejects any
transitive foreign surface. The 0.x `stdlib/` seed itself ships no I/O module yet —
the boundary machinery is in the language, and both runtimes back it.


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

- `examples/` — the snippets above, whole and runnable; `08_ordering`,
  `09_strings`, and `10_iterators` exercise the collections, ordering, string,
  and iterator surface end-to-end.
- `spec/` and `specpack/` — the normative rules and the model-facing
  distillation, both present at the repo root of this standalone distribution.
- `stdlib/` — the pure-compute `core`/`std` seed (a module tree with the ordering,
  iterator, and string modules among others); `candor run stdlib` builds and runs
  it end-to-end and returns its sentinel.
- The lab repository — the philosophy, the twelve designs (each recording what was
  rejected and why), and the Bet 5 experiment. That is the authority; this preview
  follows it.
