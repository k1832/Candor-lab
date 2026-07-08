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
use candor_proto::{check_dir, check_source_real, run_dir, run_source_real, RunResult};
use std::path::PathBuf;

const SENTINEL: i64 = 337;

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

// ---- positive: the seed runs to its sentinel (single-file image, see F1) ----

#[test]
fn seed_runs_to_sentinel() {
    // The module-tree driver cannot RUN allocation (finding F1), so the seed's
    // end-to-end runtime is proven on its flattened single-file image, which
    // uses the exact same module bodies (and the real cross-type `?`).
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

// ---- F1 regression guard: the module-tree driver cannot RUN allocation ------

#[test]
fn module_driver_cannot_run_allocation_yet() {
    // Documents finding F1 as an executable tripwire: `run_dir` on the seed
    // currently panics inside `bi_box` (bare-name offset lookup misses the
    // module-qualified `Alloc`). When F1 is fixed this stops panicking and the
    // assertion below trips, prompting a switch to a real `run_dir` sentinel.
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let outcome = std::panic::catch_unwind(|| run_dir(&dir("corelib")));
    std::panic::set_hook(prev);
    assert!(
        outcome.is_err(),
        "F1 appears fixed: `run_dir` no longer panics on the allocating seed — \
         replace `seed_runs_to_sentinel`'s single-file image with a real `run_dir` sentinel"
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
