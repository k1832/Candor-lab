//! Generics stage-1 tests (design 0007): positive check+run over the worked
//! examples, and negatives for each definition-site / conformance / coherence /
//! orphan / termination rule. Single-file programs use the `.cnr` front-end;
//! the orphan and cross-module cases use the module-tree driver (design 0008).

use candor_proto::diag::Severity;
use candor_proto::{check_dir, check_source_real, run_dir, run_source_real, RunResult};
use std::path::PathBuf;

fn fixture(rel: &str) -> String {
    let path = format!("{}/tests/fixtures/generics/{rel}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn run_ret(rel: &str) -> i64 {
    let src = fixture(rel);
    assert!(
        check_source_real(&src).unwrap().is_empty(),
        "{rel} should check clean, got {:?}",
        check_source_real(&src).unwrap()
    );
    match run_source_real(&src) {
        RunResult::Ok(r) => r.ret,
        RunResult::Fault(f) => panic!("{rel} faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!("{rel} check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        RunResult::ParseError(d) => panic!("{rel} parse error: {}", d.to_json()),
    }
}

fn codes(src: &str) -> Vec<String> {
    match check_source_real(src) {
        Ok(diags) => diags.into_iter().filter(|d| d.severity == Severity::Error).map(|d| d.code).collect(),
        Err(parse) => vec![parse.code],
    }
}

fn assert_code(src: &str, code: &str) {
    let cs = codes(src);
    assert!(cs.iter().any(|c| c == code), "expected `{code}`, got {cs:?}\n{src}");
}

// ---- positive: check clean + run to sentinel --------------------------------

#[test]
fn mono_three_types() {
    assert_eq!(run_ret("mono3.cnr"), 12);
}

#[test]
fn generic_pair_and_swap() {
    assert_eq!(run_ret("pair.cnr"), 7);
}

#[test]
fn copy_bounded_arena() {
    assert_eq!(run_ret("arena.cnr"), 14);
}

#[test]
fn interface_bound_static_dispatch() {
    assert_eq!(run_ret("iface.cnr"), 42);
}

#[test]
fn mixed_region_and_type_params() {
    assert_eq!(run_ret("mixed.cnr"), 9);
}

#[test]
fn generic_function_as_value() {
    assert_eq!(run_ret("nameval.cnr"), 8);
}

#[test]
fn generic_enum_with_match() {
    assert_eq!(run_ret("genenum.cnr"), 106);
}

#[test]
fn cross_type_question_via_from() {
    assert_eq!(run_ret("fromq.cnr"), 7);
}

// ---- positive: a `T`-dropping generic marked `alloc` is accepted (§3.4) ------

#[test]
fn t_dropping_generic_with_alloc_is_accepted() {
    // Owning-and-dropping an opaque `T` is `alloc` by §3.4; declaring it clears
    // the effect error (upper-bound conservatism).
    let src = "fn sink[T](x: T) alloc -> i64 { return 0; }\nfn main() -> i64 { return 0; }\n";
    assert!(codes(src).is_empty(), "got {:?}", codes(src));
}

// ---- negatives --------------------------------------------------------------

#[test]
fn unbounded_method_call_is_def_site_error() {
    assert_code(
        "fn f[T](x: read T) -> i64 { return x.foo(); }\nfn main() -> i64 { return 0; }\n",
        "E1002",
    );
}

#[test]
fn missing_impl_is_conformance_error() {
    assert_code(
        "interface W { fn w(read self) -> i64; }\nstruct N { v: i64 }\nstruct O { w: i64 }\n\
         impl W for N { fn w(read self) -> i64 { return self.v; } }\n\
         fn use_it[T: W](x: read T) -> i64 { return x.w(); }\n\
         fn main() -> i64 { let o: O = O { w: 1 }; return use_it(read o); }\n",
        "E1008",
    );
}

#[test]
fn duplicate_impl_is_rejected() {
    assert_code(
        "interface I { fn m(read self) -> i64; }\nstruct N { v: i64 }\n\
         impl I for N { fn m(read self) -> i64 { return 1; } }\n\
         impl I for N { fn m(read self) -> i64 { return 2; } }\n\
         fn main() -> i64 { return 0; }\n",
        "E1009",
    );
}

#[test]
fn distinct_target_impls_of_one_interface_coexist_and_dispatch() {
    // Two impls of ONE interface for DIFFERENT nominal targets must coexist (this
    // is `impl Show for ShowInt` + `impl Show for String`): distinct target
    // constructors never overlap, so no E1009. Both dispatch to their own method.
    let src = "interface I { fn m(read self) -> i64; }\nstruct A { v: i64 }\nstruct B { v: i64 }\n\
         impl I for A { fn m(read self) -> i64 { return 1; } }\n\
         impl I for B { fn m(read self) -> i64 { return 2; } }\n\
         fn main() -> i64 { let a: A = A { v: 0 }; let b: B = B { v: 0 }; return a.m() * 10 + b.m(); }\n";
    assert!(codes(src).is_empty(), "distinct targets should check clean, got {:?}", codes(src));
    match run_source_real(src) {
        RunResult::Ok(r) => assert_eq!(r.ret, 12, "each target dispatches to its own impl"),
        other => panic!("did not run: ok={}", matches!(other, RunResult::Ok(_))),
    }
}

#[test]
fn borrow_type_argument_is_rejected() {
    assert_code(
        "fn id[T](x: T) -> T { return x; }\n\
         fn main() -> i64 { let f: fn(read i64) -> read i64 = id::[read i64]; return 0; }\n",
        "E1006",
    );
}

#[test]
fn polymorphic_recursion_is_def_site_error() {
    assert_code(
        "struct Wrap[T] { v: T }\n\
         fn grow[T](x: T) -> i64 { let w: Wrap[T] = Wrap { v: x }; return grow(w); }\n\
         fn main() -> i64 { return 0; }\n",
        "E1020",
    );
}

#[test]
fn copy_bound_changes_body_checking() {
    // Without `T: copy`, reading a non-copy element out of a `read` borrow is a
    // move-out-of-borrow error (§3.1); the `copy` bound (arena.cnr) makes it a copy.
    assert_code(
        "struct Arena[T] { mem: [4]T, count: u32 }
         fn get[T](ar: read Arena[T], i: u32) -> T { return ar.mem[conv usize i]; }
         fn main() -> i64 { return 0; }
",
        "E0310",
    );
}

#[test]
fn t_dropping_generic_without_alloc_is_rejected() {
    assert_code(
        "fn sink[T](x: T) -> i64 { return 0; }\nfn main() -> i64 { return 0; }\n",
        "E0401",
    );
}

// ---- module-tree negatives / positives (design 0008) ------------------------

fn moddir(name: &str) -> PathBuf {
    PathBuf::from(format!("{}/tests/fixtures/modules/{name}", env!("CARGO_MANIFEST_DIR")))
}

fn mod_codes(name: &str) -> Vec<String> {
    match check_dir(&moddir(name)) {
        Ok(diags) => diags.into_iter().filter(|d| d.severity == Severity::Error).map(|d| d.code).collect(),
        Err(d) => vec![d.code],
    }
}

#[test]
fn orphan_impl_across_modules_is_rejected() {
    assert!(mod_codes("bad_orphan").contains(&"E1013".to_string()), "got {:?}", mod_codes("bad_orphan"));
}

#[test]
fn legal_cross_module_impl_runs() {
    assert!(mod_codes("ok_impl").is_empty(), "ok_impl should check clean, got {:?}", mod_codes("ok_impl"));
    match run_dir(&moddir("ok_impl")) {
        RunResult::Ok(r) => assert_eq!(r.ret, 42),
        other => panic!("ok_impl did not run: {:?}", matches!(other, RunResult::Ok(_))),
    }
}

// ===========================================================================
// Stage 2: generic impls, generic-struct drop hooks, and their ripple checks
// (design 0007 §2.3, §3.4). +16 tests.
// ===========================================================================

fn trace_of(rel: &str) -> Vec<i64> {
    let src = fixture(rel);
    assert!(
        check_source_real(&src).unwrap().is_empty(),
        "{rel} should check clean, got {:?}",
        check_source_real(&src).unwrap()
    );
    match run_source_real(&src) {
        RunResult::Ok(r) => r.trace,
        other => panic!("{rel} did not run: ok={}", matches!(other, RunResult::Ok(_))),
    }
}

// ---- positive: generic impls + drop hooks run ------------------------------

#[test]
fn generic_impl_method_dispatch() {
    assert_eq!(run_ret("gimpl.cnr"), 40);
}

#[test]
fn bounded_generic_impl_calls_bound_method() {
    assert_eq!(run_ret("gbound.cnr"), 105);
}

#[test]
fn generic_from_impl_cross_type_question() {
    // good=false takes the error path through `AppErr[i64]::from(IoErr)`.
    assert_eq!(run_ret("gfromq.cnr"), 7);
}

#[test]
fn generic_drop_hook_runs_nested_in_order() {
    // The `Wrap[Noisy]` hook fires first (tag 2), then its field `Noisy` (id 1).
    assert_eq!(trace_of("gdrop.cnr"), vec![2, 1]);
}

#[test]
fn generic_drop_hook_ground_floor_runs() {
    // A non-allocating hook over a `copy` `T` stays non-`alloc` yet still runs.
    assert_eq!(trace_of("gdrop_groundfloor.cnr"), vec![4]);
}

// ---- negatives: coherence / conformance ------------------------------------

#[test]
fn overlapping_generic_impls_are_rejected() {
    // Two impl heads that unify (`List[T]` and `List[U]`) overlap (§2.3).
    assert_code(
        "interface I { fn m(read self) -> i64; }\nstruct List[T] { x: T }\n\
         impl[T] I for List[T] { fn m(read self) -> i64 { return 1; } }\n\
         impl[U] I for List[U] { fn m(read self) -> i64 { return 2; } }\n\
         fn main() -> i64 { return 0; }\n",
        "E1009",
    );
}

#[test]
fn generic_and_concrete_impls_on_same_head_overlap_are_rejected() {
    // A generic head and a concrete head for the SAME target constructor still
    // overlap on their common instance (`W[T]` unifies with `W[i64]`), so adding
    // the target-name comparison must NOT weaken this: E1009 (§2.3).
    assert_code(
        "interface I { fn m(read self) -> i64; }\nstruct W[T] { x: T }\n\
         impl[T] I for W[T] { fn m(read self) -> i64 { return 1; } }\n\
         impl I for W[i64] { fn m(read self) -> i64 { return 2; } }\n\
         fn main() -> i64 { return 0; }\n",
        "E1009",
    );
}

#[test]
fn generic_impl_param_not_in_target_is_rejected() {
    // Every generic-impl parameter must appear in the target (§5.1 driving rule).
    assert_code(
        "interface I { fn m(read self) -> i64; }\nstruct N { v: i64 }\n\
         impl[T] I for N { fn m(read self) -> i64 { return 1; } }\n\
         fn main() -> i64 { return 0; }\n",
        "E1016",
    );
}

#[test]
fn bounded_generic_impl_conformance_failure() {
    // Calling a bounded impl's method on `Wrap[Plain]` where `Plain` lacks the
    // bound interface is a use-site conformance error (§2.1).
    assert_code(
        "interface Show { fn show(read self) -> i64; }\n\
         interface Weighable { fn weight(read self) -> i64; }\n\
         struct Plain { n: i64 }\nstruct Wrap[T] { inner: T }\n\
         impl[T: Show] Weighable for Wrap[T] { fn weight(read self) -> i64 { return self.inner.show(); } }\n\
         fn main() -> i64 { let w: Wrap[Plain] = Wrap { inner: Plain { n: 5 } }; return w.weight(); }\n",
        "E1008",
    );
}

// ---- negatives: alloc-on-drop of generic aggregates (§3.4) ------------------

#[test]
fn generic_aggregate_box_dying_unmarked_is_e0401() {
    assert_code(
        "struct Wrap[T] { inner: T }\n\
         fn sink(w: Wrap[Box[i64]]) -> i64 { return 0; }\n\
         fn main() -> i64 { return 0; }\n",
        "E0401",
    );
}

#[test]
fn generic_aggregate_box_dying_marked_is_clean() {
    let src = "struct Wrap[T] { inner: T }\n\
               fn sink(w: Wrap[Box[i64]]) alloc -> i64 { return 0; }\n\
               fn main() -> i64 { return 0; }\n";
    assert!(codes(src).is_empty(), "got {:?}", codes(src));
}

#[test]
fn allocating_hook_makes_generic_aggregate_alloc_on_drop() {
    // The hook allocates (calls an `alloc` fn), so every instance — even the
    // drop-inert `Wrap[i64]` — is alloc-on-drop (§3.4 F5): the unmarked owner errors.
    assert_code(
        "fn boom() alloc -> i64 { return 0; }\n\
         struct Wrap[T] { inner: T, tag: i64 } drop(write self) { let x: i64 = boom(); }\n\
         fn sink(w: Wrap[i64]) -> i64 { return 0; }\n\
         fn main() -> i64 { return 0; }\n",
        "E0401",
    );
}

// ---- negatives: partial-move / move-through-borrow over generic aggregates --

#[test]
fn move_opaque_field_through_borrow_is_e0310() {
    assert_code(
        "struct Pair[T] { a: T, b: T }\n\
         fn take_a[T](p: read Pair[T]) -> T { return p.a; }\n\
         fn main() -> i64 { return 0; }\n",
        "E0310",
    );
}

#[test]
fn partial_move_of_drop_hooked_generic_is_e0303() {
    assert_code(
        "struct Pair[T] { a: T, b: T } drop(write self) { trace(0); }\n\
         fn split[T](p: Pair[T]) -> T { return p.a; }\n\
         fn main() -> i64 { return 0; }\n",
        "E0303",
    );
}

// ---- module-tree: generic-impl orphan rule (design 0008) --------------------

#[test]
fn generic_orphan_impl_across_modules_is_rejected() {
    assert!(
        mod_codes("bad_orphan_generic").contains(&"E1013".to_string()),
        "got {:?}",
        mod_codes("bad_orphan_generic")
    );
}

#[test]
fn legal_generic_impl_across_modules_runs() {
    assert!(
        mod_codes("ok_impl_generic").is_empty(),
        "ok_impl_generic should check clean, got {:?}",
        mod_codes("ok_impl_generic")
    );
    match run_dir(&moddir("ok_impl_generic")) {
        RunResult::Ok(r) => assert_eq!(r.ret, 42),
        other => panic!("ok_impl_generic did not run: {:?}", matches!(other, RunResult::Ok(_))),
    }
}

// ---------------------------------------------------------------------------
// Impl/interface method-signature conformance (design 0007 §3.5, §4.1)
// One negative per divergence axis, plus a multi-axis conforming positive.
// ---------------------------------------------------------------------------

#[test]
fn conformance_self_mode_divergence() {
    // interface: `read self`; impl: `write self` -> E1021.
    assert_code(
        "interface W { fn w(read self) -> i64; }\nstruct N { v: i64 }\n\
         impl W for N { fn w(write self) -> i64 { return self.v; } }\n\
         fn main() -> i64 { return 0; }\n",
        "E1021",
    );
}

#[test]
fn conformance_self_presence_divergence() {
    // interface: no `self` (associated fn); impl: adds `read self` -> E1021.
    assert_code(
        "interface Mk { fn mk(x: i64) -> Self; }\nstruct N { v: i64 }\n\
         impl Mk for N { fn mk(read self, x: i64) -> Self { return N { v: x }; } }\n\
         fn main() -> i64 { return 0; }\n",
        "E1021",
    );
}

#[test]
fn conformance_param_count_divergence() {
    // interface: one non-self param; impl: none -> E1022.
    assert_code(
        "interface W { fn w(read self, a: i64) -> i64; }\nstruct N { v: i64 }\n\
         impl W for N { fn w(read self) -> i64 { return self.v; } }\n\
         fn main() -> i64 { return 0; }\n",
        "E1022",
    );
}

#[test]
fn conformance_param_mode_divergence() {
    // interface: `a: read N`; impl: `a: write N` -> E1023.
    assert_code(
        "interface W { fn w(read self, a: read N) -> i64; }\nstruct N { v: i64 }\n\
         impl W for N { fn w(read self, a: write N) -> i64 { return self.v; } }\n\
         fn main() -> i64 { return 0; }\n",
        "E1023",
    );
}

#[test]
fn conformance_param_type_divergence() {
    // interface: `a: i64`; impl: `a: u8` -> E1024.
    assert_code(
        "interface W { fn w(read self, a: i64) -> i64; }\nstruct N { v: i64 }\n\
         impl W for N { fn w(read self, a: u8) -> i64 { return self.v; } }\n\
         fn main() -> i64 { return 0; }\n",
        "E1024",
    );
}

#[test]
fn conformance_return_type_divergence() {
    // interface: `-> i64`; impl: `-> bool` -> E1025.
    assert_code(
        "interface W { fn w(read self) -> i64; }\nstruct N { v: i64 }\n\
         impl W for N { fn w(read self) -> bool { return true; } }\n\
         fn main() -> i64 { return 0; }\n",
        "E1025",
    );
}

#[test]
fn conformance_effect_marker_divergence() {
    // interface: non-`alloc`; impl: `alloc` (may not exceed) -> E1026.
    assert_code(
        "interface W { fn w(read self) -> i64; }\nstruct N { v: i64 }\n\
         impl W for N { fn w(read self) alloc -> i64 { return self.v; } }\n\
         fn main() -> i64 { return 0; }\n",
        "E1026",
    );
}

#[test]
fn conformance_generic_impl_return_divergence() {
    // A generic impl whose method's return diverges after `Self` substitution.
    // interface expects `-> i64`; impl returns `Wrap[T]` -> E1025.
    assert_code(
        "interface W { fn w(read self) -> i64; }\nstruct Wrap[T] { inner: T }\n\
         impl[T] W for Wrap[T] { fn w(read self) -> Wrap[T] { return Wrap { inner: self.inner }; } }\n\
         fn main() -> i64 { return 0; }\n",
        "E1025",
    );
}

#[test]
fn conformance_conforming_impl_checks_clean_and_runs() {
    // A multi-axis-non-trivial signature (`write self`, a value param, a return)
    // that conforms exactly: checks clean and runs.
    let src = "interface Sink { fn push(write self, v: i64) -> i64; }\nstruct Buf { total: i64 }\n\
         impl Sink for Buf { fn push(write self, v: i64) -> i64 { self.*.total = self.*.total + v; return self.*.total; } }\n\
         fn main() -> i64 { let mut b: Buf = Buf { total: 0 }; return b.push(5); }\n";
    assert!(
        check_source_real(src).unwrap().is_empty(),
        "conforming impl should check clean, got {:?}",
        check_source_real(src).unwrap()
    );
    match run_source_real(src) {
        RunResult::Ok(r) => assert_eq!(r.ret, 5),
        RunResult::Fault(f) => panic!("faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!("check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        RunResult::ParseError(d) => panic!("parse error: {}", d.to_json()),
    }
}
