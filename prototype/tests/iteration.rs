//! Design 0009 — iteration and associated types (generics stage 3). Tests the
//! associated-type member + `Base::Assoc` projection, the two by-value iteration
//! protocols (`Iter`/`Indexed`), the `for`-statement desugar, take-self receiver
//! consumption, and the E1002 completion (`Opt::map`). Single-file (the real
//! `.cnr` front-end), so each program is self-contained.

use candor_proto::diag::Severity;
use candor_proto::{check_source_real, run_source_real, run_source_real_mir, MirRunResult, RunResult};

fn errors(src: &str) -> Vec<String> {
    match check_source_real(src) {
        Ok(diags) => diags
            .into_iter()
            .filter(|d| d.severity == Severity::Error)
            .map(|d| d.code)
            .collect(),
        Err(parse) => vec![parse.code],
    }
}

fn assert_clean(src: &str) {
    let e = errors(src);
    assert!(e.is_empty(), "expected clean, got {e:?}");
}

fn run_ret(src: &str) -> i64 {
    match run_source_real(src) {
        RunResult::Ok(r) => r.ret,
        other => panic!("did not run: {}", describe(other)),
    }
}

fn run_ret_mir(src: &str) -> i64 {
    match run_source_real_mir(src) {
        MirRunResult::Ok(r) => r.ret,
        MirRunResult::Fault(f) => panic!("MIR fault: {}", f.to_json()),
        MirRunResult::CheckErrors(d) => {
            panic!("MIR check-errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        MirRunResult::ParseError(d) => panic!("MIR parse-error: {}", d.to_json()),
        MirRunResult::Unsupported(w) => panic!("MIR unsupported: {w}"),
    }
}

fn run_trace(src: &str) -> (i64, Vec<i64>) {
    match run_source_real(src) {
        RunResult::Ok(r) => (r.ret, r.trace),
        other => panic!("did not run: {}", describe(other)),
    }
}

fn describe(r: RunResult) -> String {
    match r {
        RunResult::Ok(run) => format!("ok({})", run.ret),
        RunResult::Fault(f) => format!("fault: {}", f.to_json()),
        RunResult::CheckErrors(d) => format!("check-errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        RunResult::ParseError(d) => format!("parse-error: {}", d.to_json()),
    }
}

// The `Indexed` protocol prelude (Opt + interface + Arena impl). §3.2/§8.
const INDEXED: &str = r#"
enum Opt[T] { ok Some(T), None }
interface Indexed { type Item; fn at(read self, i: usize) -> Opt[Item]; }
struct Arena[T: copy] { mem: [8]T, count: u32 }
impl[T: copy] Indexed for Arena[T] {
    type Item = T;
    fn at(read self, i: usize) -> Opt[T] {
        if i >= conv usize self.count { return Opt::None; }
        return Opt::Some(self.mem[i]);
    }
}
fn unwrap_or[T: copy](o: Opt[T], d: T) -> T { match o { Opt::Some(v) => { return v; } Opt::None => { return d; } } }
fn push[T: copy](ar: write Arena[T], x: T) -> bool { if ar.*.count >= 8u32 { return false; } let i: u32 = ar.*.count; ar.*.mem[conv usize i] = x; ar.*.count = i + 1u32; return true; }
"#;

// The `Iter` protocol prelude with a real bump allocator (List consumption). §3.1/§8.1.
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
"#;

// ---- associated types: positives ------------------------------------------

#[test]
fn assoc_type_decl_impl_projection_checks() {
    // interface `type Item;`, impl `type Item = T;`, and `Opt[Item]` projection.
    assert_clean(&format!("{INDEXED}\nfn main() -> i64 {{ return 0; }}"));
}

#[test]
fn projection_in_generic_fn_signature_and_body() {
    // `fn first[C: Indexed](c: read C) -> Opt[C::Item]` — projection in the
    // signature and resolved through the method call in the body (design 0009 §2.2).
    let src = format!(
        "{INDEXED}\n\
         fn first[C: Indexed](c: read C) -> Opt[C::Item] {{ return c.at(0); }}\n\
         fn main() -> i64 {{\n\
             let ar: Arena[i64] = Arena {{ mem: [42, 0, 0, 0, 0, 0, 0, 0], count: 1u32 }};\n\
             return unwrap_or(first(read ar), 0);\n\
         }}"
    );
    assert_clean(&src);
    assert_eq!(run_ret(&src), 42);
}

// ---- associated types: negatives ------------------------------------------

#[test]
fn impl_missing_associated_type_rejected() {
    // An impl that omits the interface's `type Item = ..` binding is E1018.
    let src = r#"
enum Opt[T] { ok Some(T), None }
interface Indexed { type Item; fn at(read self, i: usize) -> Opt[Item]; }
struct Arena[T: copy] { mem: [8]T, count: u32 }
impl[T: copy] Indexed for Arena[T] {
    fn at(read self, i: usize) -> Opt[T] { return Opt::None; }
}
fn main() -> i64 { return 0; }
"#;
    assert!(errors(src).contains(&"E1018".to_string()), "want E1018, got {:?}", errors(src));
}

#[test]
fn associated_type_name_mismatch_rejected() {
    // Binding `type Elem` where the interface declares `type Item` is E1018.
    let src = r#"
enum Opt[T] { ok Some(T), None }
interface Indexed { type Item; fn at(read self, i: usize) -> Opt[Item]; }
struct Arena[T: copy] { mem: [8]T, count: u32 }
impl[T: copy] Indexed for Arena[T] {
    type Elem = T;
    fn at(read self, i: usize) -> Opt[T] { return Opt::None; }
}
fn main() -> i64 { return 0; }
"#;
    assert!(errors(src).contains(&"E1018".to_string()), "want E1018, got {:?}", errors(src));
}

#[test]
fn projection_on_unbounded_param_rejected() {
    // `T::Item` on a parameter with no interface bound declaring `Item` is E1017.
    let src = r#"
enum Opt[T] { ok Some(T), None }
interface Indexed { type Item; fn at(read self, i: usize) -> Opt[Item]; }
fn bad[T](x: T) -> T::Item { return x.at(0); }
fn main() -> i64 { return 0; }
"#;
    assert!(errors(src).contains(&"E1017".to_string()), "want E1017, got {:?}", errors(src));
}

#[test]
fn projected_item_type_mismatch_rejected() {
    // A generic body treats the opaque projection `C::Item` as a concrete type:
    // `first` claims to return `Opt[i64]` but yields `Opt[C::Item]`. The opaque
    // projection is not `i64`, so the definition-site check rejects it (E0703) —
    // NN#10 holds for projections exactly as for parameters (design 0009 §2.2).
    let src = r#"
enum Opt[T] { ok Some(T), None }
interface Indexed { type Item; fn at(read self, i: usize) -> Opt[Item]; }
fn first[C: Indexed](c: read C) -> Opt[i64] { return c.at(0); }
fn main() -> i64 { return 0; }
"#;
    assert!(errors(src).contains(&"E0703".to_string()), "want E0703, got {:?}", errors(src));
}

// ---- for over Indexed ------------------------------------------------------

#[test]
fn for_over_indexed_runs() {
    let src = format!(
        "{INDEXED}\n\
         fn main() -> i64 {{\n\
             let ar: Arena[i64] = Arena {{ mem: [10, 20, 30, 0, 0, 0, 0, 0], count: 3u32 }};\n\
             let mut s: i64 = 0;\n\
             for x in read ar {{ s = s + x; }}\n\
             return s;\n\
         }}"
    );
    assert_clean(&src);
    assert_eq!(run_ret(&src), 60);
}

#[test]
fn mutation_during_indexed_iteration_rejected() {
    // Writing the collection under the loop's live `read` loan is caught by the
    // existing XOR loan discipline (design 0009 §4.3): E08xx conflicting borrow.
    let src = format!(
        "{INDEXED}\n\
         fn main() -> i64 {{\n\
             let mut ar: Arena[i64] = Arena {{ mem: [10, 20, 30, 0, 0, 0, 0, 0], count: 3u32 }};\n\
             for x in read ar {{ let ok: bool = push(write ar, x); }}\n\
             return 0;\n\
         }}"
    );
    let e = errors(&src);
    assert!(
        e.iter().any(|c| c == "E0801" || c == "E0303"),
        "want a conflicting-borrow (E0303/E08xx), got {e:?}"
    );
}

#[test]
fn nested_for_loops_run() {
    let src = format!(
        "{INDEXED}\n\
         fn main() -> i64 {{\n\
             let ar: Arena[i64] = Arena {{ mem: [1, 2, 3, 0, 0, 0, 0, 0], count: 3u32 }};\n\
             let mut total: i64 = 0;\n\
             for x in read ar {{ for y in read ar {{ total = total + x * y; }} }}\n\
             return total;\n\
         }}"
    );
    assert_clean(&src);
    assert_eq!(run_ret(&src), 36);
}

// ---- for over Iter (consuming) --------------------------------------------

#[test]
fn for_over_iter_consuming_runs() {
    let src = format!(
        "{ITER}\n\
         fn prepend(a: read Alloc, v: i64, rest: List[i64]) alloc -> List[i64] {{ match box(a, rest) {{ BoxResult::oom => {{ return List::Nil; }} BoxResult::boxed(b) => {{ return List::Cons(v, b); }} }} }}\n\
         fn main() alloc -> i64 {{\n\
             let mut bs: Bump = with_window(16777216, 1048576);\n\
             let al: Alloc = mk_alloc(write bs);\n\
             let l0: List[i64] = List::Nil;\n\
             let l1: List[i64] = prepend(read al, 4, l0);\n\
             let l2: List[i64] = prepend(read al, 6, l1);\n\
             let mut s: i64 = 0;\n\
             for v in l2 {{ s = s + v; }}\n\
             return s;\n\
         }}"
    );
    assert_clean(&src);
    assert_eq!(run_ret(&src), 10);
}

#[test]
fn has_positive_with_break_checks_clean() {
    // The reviewer's `has_positive` non-example (design 0009 §4.2): the break
    // sink-move keeps `__it`'s post-loop init state path-independent (no E0309).
    let src = format!(
        "{ITER}\n\
         fn has_positive(list: List[i64]) alloc -> bool {{\n\
             let mut found: bool = false;\n\
             for x in list {{ if x > 0 {{ found = true; break; }} }}\n\
             return found;\n\
         }}\n\
         fn main() alloc -> i64 {{ return 0; }}"
    );
    assert_clean(&src);
}

#[test]
fn early_break_drops_remaining_items_once() {
    // A `for` over a `List` of drop-hooked items breaking early: every item —
    // those yielded AND the un-visited remainder consumed by the sink-move — is
    // dropped exactly once (design 0009 §4.2/§4.3). The trace observes it.
    let src = format!(
        "{ITER}\n\
         struct Tr {{ v: i64 }} drop(write self) {{ trace(self.v); }}\n\
         fn prepend(a: read Alloc, v: i64, rest: List[Tr]) alloc -> List[Tr] {{ match box(a, rest) {{ BoxResult::oom => {{ return List::Nil; }} BoxResult::boxed(b) => {{ return List::Cons(Tr {{ v: v }}, b); }} }} }}\n\
         fn build(a: read Alloc) alloc -> List[Tr] {{\n\
             let l0: List[Tr] = List::Nil;\n\
             let l1: List[Tr] = prepend(a, 3, l0);\n\
             let l2: List[Tr] = prepend(a, 2, l1);\n\
             let l3: List[Tr] = prepend(a, 1, l2);\n\
             return l3;\n\
         }}\n\
         fn main() alloc -> i64 {{\n\
             let mut bs: Bump = with_window(16777216, 1048576);\n\
             let al: Alloc = mk_alloc(write bs);\n\
             let lst: List[Tr] = build(read al);\n\
             let mut count: i64 = 0;\n\
             for x in lst {{ count = count + x.v; if x.v >= 2 {{ break; }} }}\n\
             return count;\n\
         }}"
    );
    assert_clean(&src);
    let (ret, mut trace) = run_trace(&src);
    assert_eq!(ret, 3, "1 + 2, then break");
    trace.sort();
    assert_eq!(trace, vec![1, 2, 3], "every item dropped exactly once (incl. the remainder)");
}

// ---- take-self receiver consumption ---------------------------------------

#[test]
fn take_self_use_after_consume_rejected() {
    // `next(take self)` consumes the receiver via receiver syntax (0007's amended
    // ruling); using it afterward is E0301.
    let src = format!(
        "{ITER}\n\
         fn length[T](l: read List[T]) -> u32 {{ match l {{ List::Nil => {{ return 0u32; }} List::Cons(x, t) => {{ return 1u32; }} }} }}\n\
         fn use_after(list: List[i64]) alloc -> u32 {{\n\
             match list.next() {{ IterStep::More(x, r) => {{ }} IterStep::Done => {{ }} }}\n\
             return length(read list);\n\
         }}\n\
         fn main() alloc -> i64 {{ return 0; }}"
    );
    assert!(errors(&src).contains(&"E0301".to_string()), "want E0301, got {:?}", errors(&src));
}

// ---- for-operand parse restriction (ExprNoStruct) -------------------------

#[test]
fn for_operand_struct_literal_rejected() {
    // A bare struct literal as the `for` operand is excluded (ExprNoStruct, §4.4):
    // the `{` opens the loop body, so `Range { .. }` misparses — a parse error.
    let src = r#"
struct Range { lo: i64, hi: i64 }
fn main() -> i64 {
    for x in Range { lo: 0, hi: 3 } { }
    return 0;
}
"#;
    let e = errors(src);
    assert!(e.iter().any(|c| c.starts_with('P')), "want a parse error, got {e:?}");
}

// ---- Opt::map end-to-end (E1002) ------------------------------------------

#[test]
fn opt_map_end_to_end() {
    // `map[T, U](Opt[T], fn(T) -> U) -> Opt[U]` — inference through the fn-pointer
    // parameter's return type (E1002 completion), a named-fn call site, forwarding
    // `T` so `map` stays non-`alloc` (design 0009 §1.2/§5.2).
    let src = r#"
enum Opt[T] { ok Some(T), None }
fn map[T, U](o: Opt[T], f: fn(T) -> U) -> Opt[U] { match o { Opt::Some(v) => { return Opt::Some(f(v)); } Opt::None => { return Opt::None; } } }
fn unwrap_or[T: copy](o: Opt[T], d: T) -> T { match o { Opt::Some(v) => { return v; } Opt::None => { return d; } } }
fn dbl(x: i64) -> i64 { return x * 2; }
fn main() -> i64 {
    let a: Opt[i64] = map(Opt::Some(21), dbl);
    return unwrap_or(a, 0);
}
"#;
    assert_clean(src);
    // Both engines: the tree-walker and the monomorphizing MIR pipeline must
    // agree that `U` is inferred through `f`'s return type and `map` runs.
    assert_eq!(run_ret(src), 42);
    assert_eq!(run_ret_mir(src), 42);
}
