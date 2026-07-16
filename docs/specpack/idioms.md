# Candor idioms — canonical patterns from the corpus

One exemplar per pattern, drawn from `compiler/tests/fixtures/**`. These are how
correct Candor actually looks; prefer them over invented shapes.

## 1. Error enums + `From` widening + `?`
Result-shaped enum: exactly one `ok`-marked variant. `?` unwraps it or returns.
Widen a narrow error to an app error via a `From` impl (placed in the target's
module, per the orphan rule). (`corelib/core/res.cnr`)
```
pub enum Res[T, E] { ok Ok(T), Err(E) }
pub enum IoErr { Eof, Bad }
pub enum AppErr { FromIo(IoErr), Denied }

interface From[E] { fn from(e: E) -> Self; }
impl From[IoErr] for AppErr { fn from(e: IoErr) -> Self { return AppErr::FromIo(e); } }

fn drive(a: write Acc) -> Step {          // `?` works single- and multi-file
    let x: i64 = advance(a, 10)?;         // reborrow `a` bare (0005); unwrap Cont, else return Stop
    return Step::Cont(x);
}
```
`?` requires a `From` impl for cross-type widening; the explicit `Enum::Variant(from_value)` construction remains available where no impl exists.

## 2. Opt handling (result-shaped, on the ground floor)
`Some` is `ok`-marked so `?` propagates `None`. `unwrap_or` needs `T: copy` (it
returns payload/default by value, and `copy` keeps `Opt[T]` drop-inert → non-`alloc`).
(`corelib/core/opt.cnr`)
```
pub enum Opt[T] { ok Some(T), None }
pub fn unwrap_or[T: copy](o: Opt[T], d: T) -> T {
    match o { Opt::Some(v) => { return v; } Opt::None => { return d; } }
}
```

## 3. Arena + indices (value+index gear, no borrows stored)
Fixed inline `[N]T` backing, `T: copy` load-bearing: `get` copies an element out
by value, so no borrow crosses the signature and no region variable is needed.
Never allocates → lives in `core`. Write-path derefs are explicit (`ar.*.count`);
read-path auto-derefs (`ar.count`). Out-of-range index faults on the bounds check.
(`corelib/core/arena.cnr`)
```
pub struct Arena[T: copy] { mem: [8]T, count: u32 }
pub fn push[T: copy](ar: write Arena[T], x: T) -> bool {
    if ar.*.count >= 8u32 { return false; }        // capacity spent = a value, not a fault
    let i: u32 = ar.*.count;
    ar.*.mem[conv usize i] = x;
    ar.*.count = i + 1u32;
    return true;
}
pub fn get[T: copy](ar: read Arena[T], i: u32) -> T { return ar.mem[conv usize i]; }
```

## 4. Box-chain (cons) list over an explicit Alloc handle
The chain is owned through `Box`; dropping the list frees it structurally →
alloc-on-drop by construction (a non-`alloc` consumer that lets it die is E0401).
`push_front` boxes the WHOLE old list (a full move, never a partial move);
`pop_front` consumes and returns the popped item + shrunk rest (no swap/replace
primitive exists). (`corelib/std/list.cnr`)
```
pub enum List[T] { Nil, Cons(T, Box List[T]) }
pub enum Grow[T] { ok Grown(List[T]), GrowOom }

pub fn push_front[T](a: read Alloc, l: List[T], x: T) alloc -> Grow[T] {
    match box(a, l) {
        BoxResult::oom      => { return Grow::GrowOom; }
        BoxResult::boxed(b) => { return Grow::Grown(List::Cons(x, b)); }
    }
}
pub fn pop_front[T](l: List[T]) alloc -> Popped[T] {       // alloc: unbox frees the head cell
    match l {
        List::Nil          => { return Popped { item: Opt::None, rest: List::Nil }; }
        List::Cons(x, tail) => { return Popped { item: Opt::Some(x), rest: unbox(tail) }; }
    }
}
pub fn length[T](l: read List[T]) -> u32 {
    match l {
        List::Nil          => { return 0u32; }
        List::Cons(x, tail) => { return 1u32 + length(read (tail.*.*)); }   // peel match-borrow, then Box
    }
}
```

## 5. Bump allocator + `Alloc` handle (the valve boundary)
The `Alloc` handle is a `copy` vtable value (two `alloc`-typed fn-pointers +
`ctx`). DEFINING it carries no effect; USING `box`/`free` through it does. The two
`unsafe` valves are the only ones in the seed — each justification states a real
liveness/aliasing promise the checker cannot verify. (`std/alloc.cnr`, `std/bump.cnr`)
```
pub struct AllocVtable {
    alloc: fn(ctx: rawptr u8, size: usize, align: usize) alloc -> rawptr u8,
    free:  fn(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) alloc -> unit,
}
pub copy struct Alloc { ctx: rawptr u8, vt: rawptr AllocVtable }

fn bump_alloc(ctx: rawptr u8, size: usize, align: usize) -> rawptr u8 {
    unsafe "ctx points at the live Bump whose [next,end) window is reserved to this arena alone" {
        let b: Bump = ptr_read(cast_ptr[Bump](ctx));
        let aligned: usize = (b.next + align - 1) / align * align;
        if aligned + size > b.end { return ptr_null[u8](); }
        ptr_write(cast_ptr[Bump](ctx), Bump { next: aligned + size, end: b.end });
        return addr_to_ptr[u8](aligned);
    }
}
static BUMP_VT: AllocVtable = AllocVtable { alloc: bump_alloc, free: bump_free };
```

## 6. Intrusive rawptr structures + `container_of`
Intrusive links are `rawptr` fields (inert in safe code); all pointer ops are
inside `unsafe`. `container_of` recovers the struct from an embedded field via
`field_ptr`/`offsetof`. Enum-in-struct through `ptr_read`/`ptr_write`. Compare
addresses with `ptr_to_addr`. (`run/11_2_scheduler.cnr`)
```
struct Link { next: rawptr Link, prev: rawptr Link }
struct Task { link: Link, prio: u8 }

fn task_of(link: rawptr Link) -> rawptr Task {
    unsafe "link is the link field of a live Task (container_of)" {
        return cast_ptr[Task](ptr_offset(cast_ptr[u8](link), 0 - conv isize offsetof(Task, link)));
    }
}
```

## 7. The two `for` protocols
Operand borrow mode selects the protocol (syntax-directed, greppable). (`corelib/main.cnr`)
```
for x in read ar3 { arsum = arsum + x; }   // Indexed: read-borrows, copies items out, NON-alloc (ground floor)
for v in l2 { listsum = listsum + v; }     // Iter: MOVES l2, consumes each cell (unbox) -> inherits alloc
```
Impls: `impl[T: copy] Indexed for Arena[T] { type Item = T; fn at(read self, i: usize) -> Opt[T] {...} }`
and `impl[T] Iter for List[T] { type Item = T; fn next(take self) alloc -> IterStep[T, List[T]] {...} }`.
Mutating the collection inside `for x in read coll` is E0303 (the live read loan
forbids `write coll`). Higher-order code is capture-free: pass a `fn` plus an
explicit context value, e.g. `map(Opt::Some(20), inc)`.

## 8. Module layout: core / std (design 0008)
`core/*` = always-available, never allocates (proven by the effect partition):
`opt`, `res`, `arena`, `iter`. `std/*` = allocator-explicit: `alloc`, `bump`,
`list`. Entry is `fn main` in the root `main.cnr`. Cross-module use:
```
use core::opt::{Opt, unwrap_or, map};
use std::alloc::{Alloc};
use std::list::{List, push_front, pop_front, length};
```
Exported items are `pub`; `impl From[..] for T` goes in T's or `From`'s module.

## 9. Test-harness sentinel pattern
`main` returns an `i64` sentinel the harness asserts on; `trace(v)` (built-in)
records values in order, so a `drop` hook that calls `trace(self.id)` lets tests
assert drop order/count. A counting allocator (trace +N/-N) unmasks double-frees.
(`run/*.cnr`, `check.rs` `RDROP`)
```
struct R { id: i64 } drop(write self) { trace(self.id); }
fn main() -> i64 { let r: R = R { id: 3 }; return 0; }   // harness sees trace == [3] at scope exit
```

**Reminder (from grammar.md divergences):** write `Box T` (space form), not
`Box[T]`; use `BoxResult::boxed` / `BoxResult::oom`; avoid `min_of`/`max_of`.
