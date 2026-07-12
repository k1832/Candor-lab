//! Design 0009 — iterator adapters (the std+I/O milestone's headline feature).
//! Capture-free, additive user code over the `Iter` protocol (`next(take self)`
//! threading `Self` through `IterStep::More{item, successor}`): an eager `fold`
//! and two lazy adapters (`map`, `filter`) that themselves `impl Iter`, plus a
//! `list -> filter -> map -> fold` chain proving they compose. Every program is
//! single-file (the real `.cnr` front-end) and asserted on all three engines —
//! the tree-walker, the monomorphizing MIR interpreter, and the native JIT.
//!
//! A capture-free transform is a BARE fn pointer (no environment): a top-level
//! `fn` referenced by name and stored in an adapter's struct field (as the
//! allocator vtable stores `bump_alloc`/`bump_free`). Capturing closures remain
//! unavailable (OBL-GENERICS-CLOSURE), so state travels as an explicit value.
//!
//! FULLY GENERIC adapters (projection normalization). An adapter over an
//! arbitrary `I: Iter` inner — `struct MapIter[I, U] { f: fn(I::Item) -> U, .. }`
//! and `gmap`/`gfold[I: Iter, ..](it: I, f: fn(.., I::Item) -> ..)` — now checks
//! and runs: a concrete fn-pointer argument/field (`fn(i64) -> U`) unifies
//! against the projection `I::Item` because call-site inference NORMALIZES the
//! parameter/field types (resolving `I::Item` to the impl's concrete binding)
//! BEFORE argument mode-checking (`src/check/generics.rs`,
//! `normalize_projections`). Adapters may therefore be generic over their inner
//! iterator, not just its element type. See `generic_inner_adapter_over_any_iter`
//! and design 0009 §9 / OBL-GENERICS-ITER.

use candor_proto::{run_source_real, run_source_real_mir, run_source_real_native};
use candor_proto::{check_source_real, MirRunResult, RunResult};
use candor_proto::diag::Severity;

// The `Iter` protocol prelude with a real bump allocator (List consumption),
// verbatim from `iteration.rs` plus a `prepend` builder. Each test appends its
// adapters and `main`.
const ITER: &str = r#"
enum IterStep[T, S] { ok More(T, S), Done }
interface Iter { type Item; fn next(take self) alloc -> IterStep[Item, Self]; }
struct AllocVtable { alloc: fn(ctx: rawptr u8, size: usize, align: usize) alloc -> rawptr u8, free: fn(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) alloc -> unit }
copy struct Alloc { ctx: rawptr u8, vt: rawptr AllocVtable }
struct Bump { next: usize, end: usize }
fn with_window(base: usize, size: usize) -> Bump { return Bump { next: base, end: base + size }; }
fn bump_alloc(ctx: rawptr u8, size: usize, align: usize) -> rawptr u8 { unsafe "reserved window" { let b: Bump = ptr_read(cast_ptr[Bump](ctx)); let a: usize = (b.next + align - 1) / align * align; if a + size > b.end { return ptr_null[u8](); } ptr_write(cast_ptr[Bump](ctx), Bump { next: a + size, end: b.end }); return addr_to_ptr[u8](a); } }
fn bump_free(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) -> unit { }
static BUMP_VT: AllocVtable = AllocVtable { alloc: bump_alloc, free: bump_free };
fn mk_alloc(state: write Bump) -> Alloc { unsafe "outlives every box" { return Alloc { ctx: cast_ptr[u8](addr_of_mut(state.*)), vt: addr_of(BUMP_VT) }; } }
enum List[T] { Nil, Cons(T, Box List[T]) }
impl[T] Iter for List[T] {
    type Item = T;
    fn next(take self) alloc -> IterStep[T, List[T]] {
        match self { List::Nil => { return IterStep::Done; } List::Cons(x, tail) => { return IterStep::More(x, unbox(tail)); } }
    }
}
fn prepend(a: read Alloc, v: i64, rest: List[i64]) alloc -> List[i64] { match box(a, rest) { BoxResult::oom => { return List::Nil; } BoxResult::boxed(b) => { return List::Cons(v, b); } } }
"#;

fn assert_clean(src: &str) {
    let e: Vec<String> = match check_source_real(src) {
        Ok(diags) => diags
            .into_iter()
            .filter(|d| d.severity == Severity::Error)
            .map(|d| d.code)
            .collect(),
        Err(parse) => vec![parse.code],
    };
    assert!(e.is_empty(), "expected clean, got {e:?}");
}

// All three engines must agree on the return value: the tree-walker, the
// monomorphizing MIR interpreter, and the native (Cranelift JIT) backend.
fn assert_ret(src: &str, want: i64) {
    assert_clean(src);
    match run_source_real(src) {
        RunResult::Ok(r) => assert_eq!(r.ret, want, "tree-walker"),
        other => panic!("tree did not run: {}", describe(other)),
    }
    match run_source_real_mir(src) {
        MirRunResult::Ok(r) => assert_eq!(r.ret, want, "MIR"),
        other => panic!("MIR did not run: {}", describe_mir(other)),
    }
    match run_source_real_native(src) {
        MirRunResult::Ok(r) => assert_eq!(r.ret, want, "native"),
        other => panic!("native did not run: {}", describe_mir(other)),
    }
}

// A `[4, 3, 2, 1]` list over the bump allocator, bound as `l4`.
const BUILD_1234: &str = r#"
    let mut bs: Bump = with_window(16777216, 1048576);
    let al: Alloc = mk_alloc(write bs);
    let l0: List[i64] = List::Nil;
    let l1: List[i64] = prepend(read al, 1, l0);
    let l2: List[i64] = prepend(read al, 2, l1);
    let l3: List[i64] = prepend(read al, 3, l2);
    let l4: List[i64] = prepend(read al, 4, l3);
"#;

fn dbl_add() -> &'static str {
    "fn dbl(x: i64) -> i64 { return x * 2; }\nfn add(a: i64, x: i64) -> i64 { return a + x; }\nfn is_even(x: read i64) -> bool { if x.* / 2 * 2 == x.* { return true; } return false; }\n"
}

// ---- fold (eager): drive `next`, thread the accumulator through a bare fn -----

#[test]
fn fold_sums_a_list() {
    // `fold[T, A]` is generic over the element `T` and the accumulator `A`; it
    // drives `List::next` (consuming each step) and threads `acc` through the
    // capture-free `fn(A, T) -> A`. Sum of [4, 3, 2, 1] = 10.
    let src = format!(
        "{ITER}\n{}\n\
         fn fold[T, A](it: List[T], init: A, f: fn(A, T) -> A) alloc -> A {{\n\
             let mut acc: A = init;\n\
             let mut cur: List[T] = it;\n\
             loop {{ match cur.next() {{ IterStep::More(x, rest) => {{ acc = f(acc, x); cur = rest; }} IterStep::Done => {{ break; }} }} }}\n\
             return acc;\n\
         }}\n\
         fn main() alloc -> i64 {{\n{BUILD_1234}\n\
             return fold(l4, 0, add);\n\
         }}",
        dbl_add()
    );
    assert_ret(&src, 10);
}

// ---- map (lazy): a generic struct holding a bare fn, itself `impl Iter` -------

const MAP_ITER: &str = r#"
struct MapIter[T, U] { inner: List[T], f: fn(T) -> U }
impl[T, U] Iter for MapIter[T, U] {
    type Item = U;
    fn next(take self) alloc -> IterStep[U, MapIter[T, U]] {
        match self.inner.next() {
            IterStep::More(x, rest) => { return IterStep::More((self.f)(x), MapIter { inner: rest, f: self.f }); }
            IterStep::Done => { return IterStep::Done; }
        }
    }
}
fn map(it: List[i64], f: fn(i64) -> i64) -> MapIter[i64, i64] { return MapIter { inner: it, f: f }; }
fn fold_map[T, U, A](it: MapIter[T, U], init: A, f: fn(A, U) -> A) alloc -> A {
    let mut acc: A = init;
    let mut cur: MapIter[T, U] = it;
    loop { match cur.next() { IterStep::More(x, rest) => { acc = f(acc, x); cur = rest; } IterStep::Done => { break; } } }
    return acc;
}
"#;

#[test]
fn map_doubles_then_folds() {
    // `MapIter` owns its inner `List` and a bare `fn(T) -> U` in a struct field;
    // its `next` applies the fn to each yielded item and returns the successor
    // `MapIter`. Doubling [4, 3, 2, 1] -> [8, 6, 4, 2], summed = 20.
    let src = format!(
        "{ITER}\n{MAP_ITER}\n{}\n\
         fn main() alloc -> i64 {{\n{BUILD_1234}\n\
             let m: MapIter[i64, i64] = map(l4, dbl);\n\
             return fold_map(m, 0, add);\n\
         }}",
        dbl_add()
    );
    assert_ret(&src, 20);
}

// ---- filter (lazy): predicate borrows the item; skip until one passes --------

const FILTER_ITER: &str = r#"
struct FilterIter[T] { inner: List[T], keep: fn(read T) -> bool }
impl[T] Iter for FilterIter[T] {
    type Item = T;
    fn next(take self) alloc -> IterStep[T, FilterIter[T]] {
        let mut cur: List[T] = self.inner;
        loop {
            match cur.next() {
                IterStep::More(x, rest) => {
                    if (self.keep)(read x) { return IterStep::More(x, FilterIter { inner: rest, keep: self.keep }); }
                    cur = rest;
                }
                IterStep::Done => { return IterStep::Done; }
            }
        }
    }
}
fn filter(it: List[i64], keep: fn(read i64) -> bool) -> FilterIter[i64] { return FilterIter { inner: it, keep: keep }; }
fn fold_filter[T, A](it: FilterIter[T], init: A, f: fn(A, T) -> A) alloc -> A {
    let mut acc: A = init;
    let mut cur: FilterIter[T] = it;
    loop { match cur.next() { IterStep::More(x, rest) => { acc = f(acc, x); cur = rest; } IterStep::Done => { break; } } }
    return acc;
}
"#;

#[test]
fn filter_keeps_evens_then_folds() {
    // The predicate `fn(read T) -> bool` BORROWS the item (so a kept item can
    // still be yielded — a by-value predicate would move it, E0301); `next`
    // loops the inner iterator, skipping items the predicate rejects. Evens of
    // [4, 3, 2, 1] -> [4, 2], summed = 6.
    let src = format!(
        "{ITER}\n{FILTER_ITER}\n{}\n\
         fn main() alloc -> i64 {{\n{BUILD_1234}\n\
             let flt: FilterIter[i64] = filter(l4, is_even);\n\
             return fold_filter(flt, 0, add);\n\
         }}",
        dbl_add()
    );
    assert_ret(&src, 6);
}

// ---- the composing chain: list -> filter -> map -> fold ----------------------

#[test]
fn chain_filter_map_fold_composes() {
    // Lazy adapters nest: `MapFilter` owns a `FilterIter` (a concrete inner whose
    // `Item` resolves — the projection-normalization limit, see the module note).
    // [4, 3, 2, 1] -> keep evens [4, 2] -> double [8, 4] -> sum = 12. The result
    // fuses one pass: no intermediate list is materialized.
    let chain = r#"
struct FilterIter[T] { inner: List[T], keep: fn(read T) -> bool }
impl[T] Iter for FilterIter[T] {
    type Item = T;
    fn next(take self) alloc -> IterStep[T, FilterIter[T]] {
        let mut cur: List[T] = self.inner;
        loop {
            match cur.next() {
                IterStep::More(x, rest) => {
                    if (self.keep)(read x) { return IterStep::More(x, FilterIter { inner: rest, keep: self.keep }); }
                    cur = rest;
                }
                IterStep::Done => { return IterStep::Done; }
            }
        }
    }
}
struct MapFilter[U] { inner: FilterIter[i64], f: fn(i64) -> U }
impl[U] Iter for MapFilter[U] {
    type Item = U;
    fn next(take self) alloc -> IterStep[U, MapFilter[U]] {
        match self.inner.next() {
            IterStep::More(x, rest) => { return IterStep::More((self.f)(x), MapFilter { inner: rest, f: self.f }); }
            IterStep::Done => { return IterStep::Done; }
        }
    }
}
fn fold_mapfilter[U, A](it: MapFilter[U], init: A, f: fn(A, U) -> A) alloc -> A {
    let mut acc: A = init;
    let mut cur: MapFilter[U] = it;
    loop { match cur.next() { IterStep::More(x, rest) => { acc = f(acc, x); cur = rest; } IterStep::Done => { break; } } }
    return acc;
}
"#;
    let src = format!(
        "{ITER}\n{chain}\n{}\n\
         fn main() alloc -> i64 {{\n{BUILD_1234}\n\
             let flt: FilterIter[i64] = FilterIter {{ inner: l4, keep: is_even }};\n\
             let mf: MapFilter[i64] = MapFilter {{ inner: flt, f: dbl }};\n\
             return fold_mapfilter(mf, 0, add);\n\
         }}",
        dbl_add()
    );
    assert_ret(&src, 12);
}

// ---- fully generic adapter over an arbitrary `I: Iter` (projection normalized) --

#[test]
fn generic_inner_adapter_over_any_iter() {
    // A fully generic adapter — inner `I: Iter`, field `f: fn(I::Item) -> U`, and
    // itself `impl Iter` — now checks and runs. `gmap`/`gfold` are generic over
    // ANY `Iter` (not a concrete inner): a concrete `fn(i64) -> i64` unifies
    // against the projection `I::Item` because the call/struct-literal paths
    // normalize `I::Item` to the impl's binding (`i64`) before checking the
    // argument. Double [4, 3, 2, 1] -> [8, 6, 4, 2], summed = 20.
    let src = format!(
        "{ITER}\n\
         struct MapIter[I, U] {{ inner: I, f: fn(I::Item) -> U }}\n\
         impl[I: Iter, U] Iter for MapIter[I, U] {{\n\
             type Item = U;\n\
             fn next(take self) alloc -> IterStep[U, MapIter[I, U]] {{\n\
                 match self.inner.next() {{\n\
                     IterStep::More(x, rest) => {{ return IterStep::More((self.f)(x), MapIter {{ inner: rest, f: self.f }}); }}\n\
                     IterStep::Done => {{ return IterStep::Done; }}\n\
                 }}\n\
             }}\n\
         }}\n\
         fn gmap[I: Iter, U](it: I, f: fn(I::Item) -> U) -> MapIter[I, U] {{ return MapIter {{ inner: it, f: f }}; }}\n\
         fn gfold[I: Iter, A](it: I, init: A, f: fn(A, I::Item) -> A) alloc -> A {{\n\
             let mut acc: A = init;\n\
             let mut cur: I = it;\n\
             loop {{ match cur.next() {{ IterStep::More(x, rest) => {{ acc = f(acc, x); cur = rest; }} IterStep::Done => {{ break; }} }} }}\n\
             return acc;\n\
         }}\n\
         {}\n\
         fn main() alloc -> i64 {{\n{BUILD_1234}\n\
             let m: MapIter[List[i64], i64] = gmap(l4, dbl);\n\
             return gfold(m, 0, add);\n\
         }}",
        dbl_add()
    );
    assert_ret(&src, 20);
}

fn describe(r: RunResult) -> String {
    match r {
        RunResult::Ok(run) => format!("ok({})", run.ret),
        RunResult::Fault(f) => format!("fault: {}", f.to_json()),
        RunResult::CheckErrors(d) => format!("check-errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        RunResult::ParseError(d) => format!("parse-error: {}", d.to_json()),
    }
}

fn describe_mir(r: MirRunResult) -> String {
    match r {
        MirRunResult::Ok(run) => format!("ok({})", run.ret),
        MirRunResult::Fault(f) => format!("fault: {}", f.to_json()),
        MirRunResult::CheckErrors(d) => format!("check-errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        MirRunResult::ParseError(d) => format!("parse-error: {}", d.to_json()),
        MirRunResult::Unsupported(w) => format!("unsupported: {w}"),
    }
}
