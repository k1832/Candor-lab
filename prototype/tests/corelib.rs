//! Corelib seed tests (fixtures/corelib/): the first standard-library seed for
//! Candor — real `.cnr` code exercising generics + modules + P9's core/std
//! layering. Asserts what the prototype supports in-tree (the module-tree
//! CHECKER), and demonstrates the runtime + cross-type `?` on the single-file
//! front-end for the two features the module-tree DRIVER cannot yet handle.
//!
//! STAGE-3 FINDINGS surfaced by this seed (see the run report), each captured
//! by a test below so the fix is a visible, tripping change:
//!   F1. `box`/`unbox` do not dispatch under the module-tree interpreter
//!       (`bi_box` resolves `Alloc`/`AllocVtable` field offsets by BARE name,
//!       but `run_dir` registers module-qualified names -> offset 0 -> the
//!       vtable fn-pointer is misread -> panic). So the seed's std allocator
//!       layer cannot RUN via `run_dir`; the sentinel is proven on the
//!       single-file image `corelib_flat.cnr` instead (F1 guard below).
//!   F2. The `?` operator does not resolve under the module-tree checker
//!       (E0712 for BOTH same- and cross-type `?` in any multi-file program).
//!       Cross-type `?` (design 0007 §7.1) is therefore demonstrated
//!       single-file (`cross_type_question_works_single_file`).

use candor_proto::diag::Severity;
use candor_proto::{
    check_dir, check_source_real, run_dir, run_source_real, run_source_real_mir,
    run_source_real_native, run_source_real_native_opt, MirRunResult, RunResult,
};
use std::path::PathBuf;

const SENTINEL: i64 = 380;

fn dir(name: &str) -> PathBuf {
    PathBuf::from(format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR")))
}

fn fixture(rel: &str) -> String {
    let path = format!("{}/tests/fixtures/{rel}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn tree_codes(name: &str) -> Vec<String> {
    match check_dir(&dir(name)) {
        Ok(diags) => diags.into_iter().map(|d| d.code).collect(),
        Err(d) => vec![d.code],
    }
}

fn src_error_codes(src: &str) -> Vec<String> {
    match check_source_real(src) {
        Ok(diags) => diags
            .into_iter()
            .filter(|d| d.severity == Severity::Error)
            .map(|d| d.code)
            .collect(),
        Err(parse) => vec![parse.code],
    }
}

// ---- positive: the seed tree checks clean ----------------------------------

#[test]
fn tree_checks_clean() {
    let codes = tree_codes("corelib");
    assert!(codes.is_empty(), "corelib should check clean, got {codes:?}");
}

// ---- positive: the flattened single-file image runs to the sentinel (twin) --

#[test]
fn seed_runs_to_sentinel() {
    // Redundant twin of `tree_runs_to_sentinel` (finding F1 is fixed): the same
    // module bodies, flattened into one file, run end-to-end to the sentinel
    // (and exercise the real cross-type `?`).
    let src = fixture("corelib_flat.cnr");
    assert!(
        check_source_real(&src).unwrap().is_empty(),
        "corelib_flat should check clean, got {:?}",
        check_source_real(&src).unwrap()
    );
    match run_source_real(&src) {
        RunResult::Ok(r) => assert_eq!(r.ret, SENTINEL),
        other => panic!("corelib_flat did not run to the sentinel: {}", describe(other)),
    }
}

// ---- positive: a non-alloc consumer of `core` compiles (the ground floor) ---

#[test]
fn ground_floor_core_compiles() {
    // A function with NO `alloc` marker that uses `core` (Opt + Arena) type-
    // checks: core is the always-available, never-allocating floor (P9/design
    // 0008 §5). The alloc partition proves no core item carries `alloc`.
    let src = r#"
enum Opt[T] { ok Some(T), None }
fn unwrap_or[T: copy](o: Opt[T], d: T) -> T { match o { Opt::Some(v) => { return v; } Opt::None => { return d; } } }
struct Arena[T: copy] { mem: [8]T, count: u32 }
fn get[T: copy](ar: read Arena[T], i: u32) -> T { return ar.mem[conv usize i]; }
fn ground(a: Opt[i64]) -> i64 {
    let ar: Arena[i64] = Arena { mem: [7, 7, 7, 7, 7, 7, 7, 7], count: 8u32 };
    return unwrap_or(a, 0) + get(read ar, 3);
}
fn main() -> i64 { return ground(Opt::Some(5)); }
"#;
    assert!(
        check_source_real(src).unwrap().is_empty(),
        "ground-floor core use should check clean, got {:?}",
        check_source_real(src).unwrap()
    );
}

// ---- positive/negative: dropping a `List` in a non-alloc fn is E0401 --------

#[test]
fn list_drop_is_alloc_taxed() {
    // `List` is alloc-on-drop by construction (owns a `Box` chain), so a
    // non-`alloc` consumer that lets one die is rejected — the P2 one-way
    // partition working as designed (freeing is allocator work; design 0007
    // §3.4/§8.5, the ground-floor "tax").
    let src = r#"
enum List[T] { Nil, Cons(T, Box List[T]) }
fn drop_list(l: List[i64]) -> i64 { return 0; }
fn main() -> i64 { return 0; }
"#;
    assert!(
        src_error_codes(src).contains(&"E0401".to_string()),
        "want E0401, got {:?}",
        src_error_codes(src)
    );
}

// ---- feature demo: cross-type `?` (works single-file; F2 blocks it in-tree) --

#[test]
fn cross_type_question_works_single_file() {
    let src = r#"
enum Res[T, E] { ok Ok(T), Err(E) }
enum IoErr { Eof }
enum AppErr { FromIo(IoErr) }
interface From[E] { fn from(e: E) -> Self; }
impl From[IoErr] for AppErr { fn from(e: IoErr) -> Self { return AppErr::FromIo(e); } }
fn read_at(ok: bool) -> Res[i64, IoErr] { if ok { return Res::Ok(42); } return Res::Err(IoErr::Eof); }
fn decode(ok: bool) -> Res[i64, AppErr] { let n: i64 = read_at(ok)?; return Res::Ok(n + 1); }
fn main() -> i64 { match decode(true) { Res::Ok(v) => { return v; } Res::Err(e) => { return -1; } } }
"#;
    assert!(
        check_source_real(src).unwrap().is_empty(),
        "cross-type `?` should check clean single-file, got {:?}",
        check_source_real(src).unwrap()
    );
    match run_source_real(src) {
        RunResult::Ok(r) => assert_eq!(r.ret, 43),
        other => panic!("cross-type `?` did not run: {}", describe(other)),
    }
}

// ---- negative: importing a private (non-`pub`) item across the tree ---------

#[test]
fn private_access_across_tree_rejected() {
    assert!(
        tree_codes("corelib_neg_private").contains(&"E0903".to_string()),
        "want E0903, got {:?}",
        tree_codes("corelib_neg_private")
    );
}

// ---- negative: an impl placed outside the orphan rule's two homes -----------

#[test]
fn orphan_impl_across_tree_rejected() {
    assert!(
        tree_codes("corelib_neg_orphan").contains(&"E1013".to_string()),
        "want E1013, got {:?}",
        tree_codes("corelib_neg_orphan")
    );
}

// ---- negative: importing a name a module does not export -------------------

#[test]
fn unresolved_import_across_tree_rejected() {
    assert!(
        tree_codes("corelib_neg_unresolved").contains(&"E0902".to_string()),
        "want E0902, got {:?}",
        tree_codes("corelib_neg_unresolved")
    );
}

// ---- F1 (fixed): the module-tree DRIVER runs the allocating seed to sentinel -

#[test]
fn tree_runs_to_sentinel() {
    // Finding F1 is fixed: `box`/`unbox` now resolve the `Alloc`/`AllocVtable`
    // layouts by their post-qualification names (identified structurally), so
    // `run_dir` executes the seed's std allocator layer end-to-end. This is the
    // primary runtime sentinel; the single-file image below is the redundant
    // twin (same module bodies, flattened).
    match run_dir(&dir("corelib")) {
        RunResult::Ok(r) => assert_eq!(r.ret, SENTINEL),
        other => panic!("corelib tree did not run to the sentinel: {}", describe(other)),
    }
}

// ---- F2 (fixed): the `?` operator resolves under the module-tree checker ------

#[test]
fn question_operator_across_tree() {
    // Finding F2 is fixed: both same-type and cross-type `?` now resolve in a
    // MULTI-FILE program (the From-impl / interface lookup is matched by base
    // name across the tree's qualified names, and same-type `?` on a generic
    // result enum is recognized). The fixture drives `same` (same-type) and
    // `cross` (widens `IoErr` -> `AppErr` via `impl From`, design 0007 §7.1).
    let codes = tree_codes("corelib_question");
    assert!(codes.is_empty(), "corelib_question should check clean, got {codes:?}");
    match run_dir(&dir("corelib_question")) {
        RunResult::Ok(r) => assert_eq!(r.ret, 186), // 43 (same) + 43 (cross ok) + 100 (cross err)
        other => panic!("corelib_question did not run: {}", describe(other)),
    }
}

// ---- F3 (fixed): a generic struct's scalar sibling of a Box-bearing field -----

#[test]
fn generic_struct_scalar_sibling_reads() {
    // Finding F3 is fixed: a generic struct with a scalar field beside a
    // recursive Box-bearing generic-enum field no longer misreads the scalar (it
    // used to read 0 / panic `unknown enum`). Root cause: a niladic generic-enum
    // ctor in a field position never pinned the struct parameter, so the resolved
    // argument (from the expected type) was not folded back before substituting
    // the field's expected type. `l.count` must read 7.
    let src = r#"
enum Node[T] { Nil, Cons(T, Box Node[T]) }
struct Holder[T] { count: u32, head: Node[T] }
fn main() alloc -> i64 {
    let h: Holder[i64] = Holder { count: 7u32, head: Node::Nil };
    return conv i64 h.count;
}
"#;
    assert!(
        check_source_real(src).unwrap().is_empty(),
        "F3 repro should check clean, got {:?}",
        check_source_real(src).unwrap()
    );
    match run_source_real(src) {
        RunResult::Ok(r) => assert_eq!(r.ret, 7),
        other => panic!("F3 repro did not run to 7: {}", describe(other)),
    }
}

// ---- F4 (fixed): expected-type-driven inference for a niladic generic call ----

#[test]
fn niladic_generic_call_infers_from_annotation() {
    // Finding F4 is fixed: `let x: List[i64] = nil();` where `nil` is
    // `fn nil[T]() -> List[T]` now infers `T` from the annotation (design 0007
    // §6.2) instead of E1002 — the value arguments give no evidence, so the
    // declared return type is unified against the expected type.
    let src = r#"
enum List[T] { Nil, Cons(T, Box List[T]) }
fn nil[T]() -> List[T] { return List::Nil; }
fn is_empty[T](l: read List[T]) -> bool { match l { List::Nil => { return true; } List::Cons(x, t) => { return false; } } }
fn main() alloc -> i64 { let x: List[i64] = nil(); if is_empty(read x) { return 5; } return 9; }
"#;
    assert!(
        check_source_real(src).unwrap().is_empty(),
        "F4 annotation repro should check clean, got {:?}",
        check_source_real(src).unwrap()
    );
    match run_source_real(src) {
        RunResult::Ok(r) => assert_eq!(r.ret, 5),
        other => panic!("F4 annotation repro did not run to 5: {}", describe(other)),
    }
}

#[test]
fn niladic_generic_call_infers_from_assignment_target() {
    // F4, assignment-target flavor: the expected type driving inference comes
    // from the assignment target's type, not a `let` annotation.
    let src = r#"
enum List[T] { Nil, Cons(T, Box List[T]) }
fn nil[T]() -> List[T] { return List::Nil; }
fn main() alloc -> i64 { let mut x: List[i64] = List::Nil; x = nil(); return 0; }
"#;
    assert!(
        check_source_real(src).unwrap().is_empty(),
        "F4 assignment repro should check clean, got {:?}",
        check_source_real(src).unwrap()
    );
    match run_source_real(src) {
        RunResult::Ok(r) => assert_eq!(r.ret, 0),
        other => panic!("F4 assignment repro did not run: {}", describe(other)),
    }
}

// ---- Res/Opt combinators: the six seed additions, exact values, all engines --

#[test]
fn res_opt_combinators_run_to_sentinel() {
    // `and_then`/`ok_or` (Opt side, hosted in `core::res`) + `map`/`map_err`/
    // `unwrap_or`/`ok` (Res side), flattened into one image and driven to an exact
    // sentinel on the tree-walker, MIR, native, and native-opt engines. Proves the
    // capture-free fn-pointer combinators (OBL-GENERICS-CLOSURE) monomorphize and
    // lower natively, and that the `alloc`-taxed discarders drop a concrete
    // (non-Box) error cleanly on every backend.
    let src = fixture("corelib_combinators.cnr");
    assert!(
        check_source_real(&src).unwrap().is_empty(),
        "combinators image should check clean, got {:?}",
        check_source_real(&src).unwrap()
    );
    match run_source_real(&src) {
        RunResult::Ok(r) => assert_eq!(r.ret, 452, "tree-walker"),
        other => panic!("combinators did not run: {}", describe(other)),
    }
    for (label, r) in [
        ("mir", run_source_real_mir(&src)),
        ("native", run_source_real_native(&src)),
        ("native-opt", run_source_real_native_opt(&src)),
    ] {
        match r {
            MirRunResult::Ok(run) => assert_eq!(run.ret, 452, "{label}"),
            other => panic!("{label} did not run: ok={}", matches!(other, MirRunResult::Ok(_))),
        }
    }
}

// ---- and_then: tree-walker only (aggregate-returning indirect call) ----------

#[test]
fn and_then_runs_on_tree_walker_mir_native_gap() {
    // `and_then`'s `f: fn(T) -> Opt[U]` is an INDIRECT (fn-pointer) call returning
    // an AGGREGATE (`Opt[U]`). The MIR lowering rejects that shape
    // (`mir/build.rs`, "indirect/unknown aggregate call"), so `and_then` runs only
    // on the tree-walker; scalar-returning indirect combinators (`map`/`map_err`)
    // lower on every engine. This pins the backend gap: if it is ever closed, the
    // Unsupported assertion below flips and this test surfaces the fix.
    let src = r#"
enum Opt[T] { ok Some(T), None }
fn and_then[T, U](o: Opt[T], f: fn(T) -> Opt[U]) -> Opt[U] {
    match o { Opt::Some(v) => { return f(v); }, Opt::None => { return Opt::None; }, }
}
fn step(x: i64) -> Opt[i64] { return Opt::Some(x * 2); }
fn main() -> i64 {
    let mut acc: i64 = 0;
    match and_then(Opt::Some(4), step) { Opt::Some(v) => { acc = acc + v; }, Opt::None => { acc = acc + 0; }, }
    match and_then(Opt::None, step) { Opt::Some(v) => { acc = acc + 1000; }, Opt::None => { acc = acc + 40; }, }
    return acc;
}
"#;
    assert!(
        check_source_real(src).unwrap().is_empty(),
        "and_then image should check clean, got {:?}",
        check_source_real(src).unwrap()
    );
    match run_source_real(src) {
        RunResult::Ok(r) => assert_eq!(r.ret, 48, "Some(4)->8 plus None->40"),
        other => panic!("and_then did not run on the tree-walker: {}", describe(other)),
    }
    assert!(
        matches!(candor_proto::run_source_real_mir(src), MirRunResult::Unsupported(_)),
        "aggregate-returning indirect call is a known MIR/native lowering gap"
    );
}

// ---- negative: E0712 is NARROWED, not gone ----------------------------------

#[test]
fn question_without_return_or_from_is_e0712() {
    // The stale `res.cnr` comment claimed E0712 fired for ALL in-tree `?`. Reality
    // (src/check/expr.rs check_try): `?` resolves same-type (Named==Named/App==App)
    // and cross-type (`try_from_conversion` via `From`); E0712 now fires ONLY when
    // neither holds. Here the enclosing return error type is `Other` and there is
    // no `impl From[IoErr] for Other`, so the `?` is genuinely unconvertible.
    let src = r#"
enum Res[T, E] { ok Ok(T), Err(E) }
enum IoErr { Eof }
enum Other { Nope }
fn read_at(ok: bool) -> Res[i64, IoErr] { if ok { return Res::Ok(42); } return Res::Err(IoErr::Eof); }
fn decode(ok: bool) -> Res[i64, Other] { let n: i64 = read_at(ok)?; return Res::Ok(n); }
fn main() -> i64 { return 0; }
"#;
    assert!(
        src_error_codes(src).contains(&"E0712".to_string()),
        "want E0712 (no return-match, no From), got {:?}",
        src_error_codes(src)
    );
}

fn describe(r: RunResult) -> String {
    match r {
        RunResult::Ok(run) => format!("ok({})", run.ret),
        RunResult::Fault(f) => format!("fault: {}", f.to_json()),
        RunResult::CheckErrors(d) => {
            format!("check-errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => format!("parse-error: {}", d.to_json()),
    }
}
