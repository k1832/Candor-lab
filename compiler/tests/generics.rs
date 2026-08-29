//! Generics stage-1 tests (design 0007): positive check+run over the worked
//! examples, and negatives for each definition-site / conformance / coherence /
//! orphan / termination rule. Single-file programs use the `.cnr` front-end;
//! the orphan and cross-module cases use the module-tree driver (design 0008).

use candor::diag::Severity;
use candor::{
    check_dir, check_source_real, run_dir, run_dir_mir, run_dir_native, run_dir_native_opt,
    run_source_real, run_source_real_mir, run_source_real_native, run_source_real_native_opt,
    MirRunResult, RunResult,
};
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
fn method_dispatches_on_call_shaped_receiver() {
    // A method call whose receiver is itself a CALL expression. The tree-walker
    // must resolve the receiver's static type from the callee's return type to
    // pick the impl (the checker already did, via its full type walk). Covers a
    // bare call `make_*().m()` and a deref-of-call `pick(read a).*.m()`; each
    // dispatches to its target's own impl.
    let src = "interface I { fn m(read self) -> i64; }
struct A { v: i64 }
struct B { v: i64 }
         impl I for A { fn m(read self) -> i64 { return self.v * 10; } }
         impl I for B { fn m(read self) -> i64 { return self.v + 1; } }
         fn make_a() -> A { return A { v: 4 }; }
         fn make_b() -> B { return B { v: 8 }; }
         fn pick(a: read A) -> read A { return a; }
         fn main() -> i64 {
             let a: A = A { v: 5 };
             return make_a().m() + make_b().m() + pick(read a).*.m();
         }
";
    assert!(codes(src).is_empty(), "call-shaped receiver dispatch should check clean, got {:?}", codes(src));
    match run_source_real(src) {
        // make_a().m() = 4*10 = 40; make_b().m() = 8+1 = 9; pick(read a).*.m() = 5*10 = 50.
        RunResult::Ok(r) => assert_eq!(r.ret, 99, "each call-shaped receiver dispatches to its own impl"),
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
fn iface_method_array_of_borrows_return_keeps_receiver_loan() {
    // P4 review B1: an interface method returning `[N]read T` reborrows its
    // receiver exactly as a `read T`-returning method does — the landing
    // binding must keep the receiver's loan, so a write to the receiver while
    // the array is live is E0803 (previously the loan was shed at the let).
    assert_code(
        "interface Viewer { fn view(read self) -> [2]read i64; }\n\
         struct Pr { a: i64, b: i64 }\n\
         impl Viewer for Pr { fn view(read self) -> [2]read i64 \
         { return [read self.a, read self.b]; } }\n\
         fn main() -> i64 { let mut p: Pr = Pr { a: 1, b: 2 }; \
         let v: [2]read i64 = p.view(); p.a = 9; return v[0].*; }\n",
        "E0803",
    );
}

#[test]
fn iface_method_assoc_type_array_of_borrows_return_keeps_receiver_loan() {
    // Same shape through an associated-type return (`[2]read Self::Item`).
    assert_code(
        "interface RefPair { type Item; fn pair(read self) -> [2]read Self::Item; }\n\
         struct Qr { a: i64, b: i64 }\n\
         impl RefPair for Qr { type Item = i64; fn pair(read self) -> [2]read i64 \
         { return [read self.a, read self.b]; } }\n\
         fn main() -> i64 { let mut q: Qr = Qr { a: 3, b: 4 }; \
         let v: [2]read i64 = q.pair(); q.a = 9; return v[0].*; }\n",
        "E0803",
    );
}

#[test]
fn iface_method_array_of_borrows_return_within_window_accepted() {
    let src = "interface Viewer { fn view(read self) -> [2]read i64; }\n\
         struct Pr { a: i64, b: i64 }\n\
         impl Viewer for Pr { fn view(read self) -> [2]read i64 \
         { return [read self.a, read self.b]; } }\n\
         fn main() -> i64 { let mut p: Pr = Pr { a: 1, b: 2 }; \
         let v: [2]read i64 = p.view(); let s: i64 = v[0].* + v[1].*; \
         p.a = 9; return s + p.a; }\n";
    assert!(codes(src).is_empty(), "expected clean, got {:?}", codes(src));
}

#[test]
fn method_receiver_probe_does_not_duplicate_same_call_overlap() {
    // P4 review M1: `synth_arg_type` probes the receiver expression; the call
    // groups the probe pushed re-raised the receiver call's same-call overlap,
    // tripling E0805. Exactly one diagnostic must remain.
    let src = "interface M { fn m(read self) -> i64; }\n\
         struct Sm { v: i64 }\n\
         impl M for Sm { fn m(read self) -> i64 { return self.v; } }\n\
         fn mk(a: write i64, b: write i64) -> Sm { return Sm { v: a.* + b.* }; }\n\
         fn main() -> i64 { let mut x: i64 = 1; \
         let r: i64 = mk(write x, write x).m(); return r; }\n";
    let cs = codes(src);
    let n = cs.iter().filter(|c| *c == "E0805").count();
    assert_eq!(n, 1, "expected exactly one E0805, got {cs:?}");
}

#[test]
fn array_of_borrows_type_argument_is_rejected() {
    // An array of borrows stores them just the same (P4): no laundering a
    // borrow into abstraction through an array element type.
    assert_code(
        "fn id[T](x: T) -> T { return x; }\n\
         fn main() -> i64 { let f: fn([2]read i64) -> [2]read i64 = id::[[2]read i64]; return 0; }\n",
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

// ---------------------------------------------------------------------------
// Cross-module monomorphization key (design 0007 §5 / 0008): the shape map must
// be keyed by a GLOBALLY-unique identity, not a bare per-file `span.start`.
// ---------------------------------------------------------------------------

/// A generic enum instantiated from two sibling modules whose `Opt::Some` /
/// `unwrap_or` nodes sit at IDENTICAL per-file byte offsets. Under the old
/// `span.start` shape key those nodes collided across modules, so `ga`'s
/// `Opt[i64]` was miscompiled as `b`'s `Opt[i32]` (an 8-byte payload read as 4
/// bytes -> a truncated/garbage value that differed between engines). The
/// `(item_index, span.start)` key makes the two nodes distinct, so every engine
/// monomorphizes each module's instantiation to its own concrete type.
#[test]
fn cross_module_generic_instantiation_no_span_collision() {
    let dir = moddir("generic_span_collision");
    assert!(
        check_dir(&dir).unwrap().iter().all(|d| d.severity != Severity::Error),
        "generic_span_collision should check clean, got {:?}",
        check_dir(&dir).unwrap()
    );
    // ga(1000000000000) keeps its i64 width (no i32 truncation) + gb(20) = ...020.
    let expected = 1000000000020i64;
    match run_dir(&dir) {
        RunResult::Ok(r) => assert_eq!(r.ret, expected, "tree-walker"),
        other => panic!("tree-walker did not run: {:?}", matches!(other, RunResult::Ok(_))),
    }
    for (label, r) in [
        ("mir", run_dir_mir(&dir)),
        ("native", run_dir_native(&dir)),
        ("native-opt", run_dir_native_opt(&dir)),
    ] {
        match r {
            MirRunResult::Ok(run) => assert_eq!(run.ret, expected, "{label}"),
            other => panic!("{label} did not run: ok={}", matches!(other, MirRunResult::Ok(_))),
        }
    }
}


// ---------------------------------------------------------------------------
// Monomorphization backstop: DEPTH of an instantiation chain, not total breadth
// (design 0007 §5.1.1, spec 10.4). Surfaced by the P20 50 kL reference build,
// which authored its per-module `pick[T]` generics as definition-only because the
// old `drive` counter incremented per work-item and never reset -- bounding the
// TOTAL instances program-wide at 64, aborting hundreds of legal, distinct,
// shallow instantiations with E1099.
// ---------------------------------------------------------------------------

/// The shared P20 prelude (i64-only): `Cmp`, the `Ord2` interface + its `i64`
/// impl, and the `maxof` combiner every generated `pick[T]` delegates to.
fn ord2_prelude() -> String {
    "enum Cmp { Lt, Eq, Gt, }\n\
     interface Ord2 { fn compare(read self, other: Self) -> Cmp; }\n\
     impl Ord2 for i64 {\n\
     \x20   fn compare(read self, other: Self) -> Cmp {\n\
     \x20       if self.* < other { return Cmp::Lt; }\n\
     \x20       if self.* > other { return Cmp::Gt; }\n\
     \x20       return Cmp::Eq;\n\
     \x20   }\n\
     }\n\
     fn maxof[T: Ord2 + copy](a: T, b: T) -> T {\n\
     \x20   match a.compare(b) {\n\
     \x20       Cmp::Lt => { return b; },\n\
     \x20       Cmp::Eq => { return a; },\n\
     \x20       Cmp::Gt => { return a; },\n\
     \x20   }\n\
     }\n".to_string()
}

/// Breadth is unbounded: several hundred DISTINCT generic instantiations at
/// shallow depth compile and run. Mirrors the P20 tree's per-module `pick[T]`
/// pattern the generator had to keep definition-only. `pickK(k, k+1)` returns
/// `k+1`, so `main` sums `k+1` over `k in 0..N` -> `N*(N+1)/2`. With N = 300 the
/// program reaches ~300 distinct `pickK[i64]` instances (plus the shared
/// `maxof[i64]` and the `Ord2 for i64` method) -- far past the old count-of-64
/// cliff. Byte-exact across the tree-walker and the MIR engine.
#[test]
fn wide_generic_breadth_monomorphizes() {
    const N: i64 = 300;
    let mut src = ord2_prelude();
    for k in 0..N {
        src.push_str(&format!(
            "fn pick{k}[T: Ord2 + copy](a: T, b: T) -> T {{ return maxof(a, b); }}\n"
        ));
    }
    src.push_str("fn main() -> i64 {\n    let mut acc: i64 = 0i64;\n");
    for k in 0..N {
        src.push_str(&format!("    acc = acc + pick{k}({k}i64, {}i64);\n", k + 1));
    }
    src.push_str("    return acc;\n}\n");

    let expected = N * (N + 1) / 2;
    assert!(
        check_source_real(&src).unwrap().iter().all(|d| d.severity != Severity::Error),
        "wide breadth should check clean, got {:?}",
        check_source_real(&src).unwrap()
    );
    match run_source_real(&src) {
        RunResult::Ok(r) => assert_eq!(r.ret, expected, "tree-walker"),
        other => panic!("tree-walker did not run: ok={}", matches!(other, RunResult::Ok(_))),
    }
    match run_source_real_mir(&src) {
        MirRunResult::Ok(run) => assert_eq!(run.ret, expected, "mir"),
        other => panic!("mir did not run: ok={}", matches!(other, MirRunResult::Ok(_))),
    }
}

/// The recursion guard still fires: a genuinely divergent chain -- mutual
/// recursion with a syntactically GROWING type argument (`ping[T]` -> `pong[Wrap[T]]`
/// -> `ping[Wrap[Wrap[T]]]` -> ...) -- has no fixed point. It is INDIRECT, so the
/// direct-only def-site check (E1020) does not catch it; monomorphization aborts
/// with E1099 at depth rather than looping forever.
#[test]
fn divergent_instantiation_chain_hits_depth_limit() {
    let src = "struct Wrap[T] { v: T }\n\
               fn ping[T](x: T) -> i64 { let w: Wrap[T] = Wrap { v: x }; return pong(w); }\n\
               fn pong[T](x: T) -> i64 { let w: Wrap[T] = Wrap { v: x }; return ping(w); }\n\
               fn main() -> i64 { return ping(0i64); }\n";
    // The bodies check clean (mutual recursion escapes the direct-only E1020).
    assert!(
        check_source_real(src).unwrap().iter().all(|d| d.severity != Severity::Error),
        "divergent chain should check clean at the definition site, got {:?}",
        check_source_real(src).unwrap()
    );
    match run_source_real(src) {
        RunResult::CheckErrors(d) => assert!(
            d.iter().any(|x| x.code == "E1099"),
            "expected E1099 depth backstop, got {:?}",
            d.iter().map(|x| &x.code).collect::<Vec<_>>()
        ),
        other => panic!("expected E1099, got ok={}", matches!(other, RunResult::Ok(_))),
    }
}

// ===========================================================================
// Array type arguments carry their length through monomorphization (review
// finding P1, 2026-08-03): `type_to_ast_kind` used to render every substituted
// `Type::Array` with a hardcoded length of 0, so the MIR/native engines laid
// out zero-length arrays (wrong results in safe code) while the tree-walk
// oracle, whose values carry their own length, computed correctly. The mangler
// also omitted the length, collapsing `[2]i64` and `[3]i64` onto one instance.
// Each test runs on every engine and must agree with the oracle.
// ===========================================================================

/// Run `src` on the oracle, MIR, native, and native `-O2`; assert every return
/// agrees with the oracle's and return it.
fn all_engines_ret(src: &str) -> i64 {
    let o_ret = match run_source_real(src) {
        RunResult::Ok(r) => r.ret,
        RunResult::Fault(f) => panic!("oracle faulted: {}\n{src}", f.to_json()),
        RunResult::CheckErrors(d) => {
            panic!("oracle check errors: {:?}\n{src}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("oracle parse error: {}\n{src}", d.to_json()),
    };
    for (label, res) in [
        ("mir", run_source_real_mir(src)),
        ("native-noopt", run_source_real_native(src)),
        ("native-opt", run_source_real_native_opt(src)),
    ] {
        match res {
            MirRunResult::Ok(r) => {
                assert_eq!(r.ret, o_ret, "{label} ret diverged from oracle for:\n{src}")
            }
            MirRunResult::Fault(f) => panic!("{label} faulted: {}\n{src}", f.to_json()),
            MirRunResult::Unsupported(e) => panic!("{label} unsupported: {e}\n{src}"),
            MirRunResult::CheckErrors(d) => {
                panic!("{label} check errors: {:?}\n{src}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
            }
            MirRunResult::ParseError(d) => panic!("{label} parse error: {}\n{src}", d.to_json()),
        }
    }
    o_ret
}

/// The P1 repro shape: a generic identity handed a `[3]i64` (also the RETURN
/// position — the instance's return type is the same rendered array).
#[test]
fn array_type_argument_round_trips_all_engines() {
    let src = "fn idf[T: copy](x: T) -> T { return x; }\n\
               fn main() -> i64 {\n\
                   let a: [3]i64 = [7, 8, 9];\n\
                   let b: [3]i64 = idf(a);\n\
                   return b[0] + b[1] + b[2];\n\
               }\n";
    assert_eq!(all_engines_ret(src), 24);
}

/// The array survives a two-deep generic call chain (`outer` instantiates
/// `inner` with the same substituted array argument).
#[test]
fn array_through_two_deep_generic_chain_all_engines() {
    let src = "fn inner[T: copy](x: T) -> T { return x; }\n\
               fn outer[T: copy](x: T) -> T { return inner(x); }\n\
               fn main() -> i64 {\n\
                   let a: [3]i64 = [1, 2, 3];\n\
                   let b: [3]i64 = outer(a);\n\
                   return b[0] * 100 + b[1] * 10 + b[2];\n\
               }\n";
    assert_eq!(all_engines_ret(src), 123);
}

/// An array-typed field inside a generic struct: the instance's field type must
/// be laid out at the real length. All-unsuffixed elements: the P5 fix grounds
/// the literal's element type before the instantiation is recorded, so no
/// suffix is needed (this line was `[4i64, 5, 6]` while P5 was open).
#[test]
fn array_field_inside_generic_struct_all_engines() {
    let src = "struct Wrap[T] { v: T }\n\
               fn first[T: copy](w: Wrap[T]) -> T { return w.v; }\n\
               fn main() -> i64 {\n\
                   let w: Wrap[[3]i64] = Wrap { v: [4, 5, 6] };\n\
                   let c: [3]i64 = first(w);\n\
                   return c[0] * 100 + c[1] * 10 + c[2];\n\
               }\n";
    assert_eq!(all_engines_ret(src), 456);
}

/// A generic returning its array argument through a recursive call: the RETURN
/// type is rendered per instance and must keep the length at every depth.
#[test]
fn generic_returning_its_array_argument_all_engines() {
    let src = "fn make[T: copy](x: T, again: bool) -> T {\n\
                   if again { return make(x, false); }\n\
                   return x;\n\
               }\n\
               fn main() -> i64 {\n\
                   let a: [3]i64 = [9, 8, 7];\n\
                   let b: [3]i64 = make(a, true);\n\
                   return b[0] * 100 + b[1] * 10 + b[2];\n\
               }\n";
    assert_eq!(all_engines_ret(src), 987);
}

/// A nested array type argument (`[2][3]i64`): both dimensions must survive.
#[test]
fn nested_array_type_argument_all_engines() {
    let src = "fn idf[T: copy](x: T) -> T { return x; }\n\
               fn main() -> i64 {\n\
                   let m: [2][3]i64 = [[7, 8, 9], [10, 11, 12]];\n\
                   let n: [2][3]i64 = idf(m);\n\
                   return n[0][0] + n[0][1] + n[0][2] + n[1][0] + n[1][1] + n[1][2];\n\
               }\n";
    assert_eq!(all_engines_ret(src), 57);
}

/// Two lengths of the same element type are DISTINCT instantiations: the length
/// is part of the instance key (`arr2_i64` vs `arr3_i64`), so `[2]i64` and
/// `[3]i64` must not collapse onto one monomorphized instance. Before the
/// mangler carried the length, the second call reused the first instance.
#[test]
fn distinct_array_lengths_monomorphize_separately_all_engines() {
    let src = "fn idf[T: copy](x: T) -> T { return x; }\n\
               fn main() -> i64 {\n\
                   let a: [3]i64 = [1, 2, 3];\n\
                   let b: [3]i64 = idf(a);\n\
                   let s: [2]i64 = [100, 200];\n\
                   let t: [2]i64 = idf(s);\n\
                   return b[2] + t[1];\n\
               }\n";
    assert_eq!(all_engines_ret(src), 203);
}

/// A named-constant length (`[N]i64`) renders by its spelling and resolves
/// through each engine's const table, exactly as a source annotation would.
#[test]
fn named_constant_array_length_type_argument_all_engines() {
    let src = "static N: usize = 2;\n\
               fn idf[T: copy](x: T) -> T { return x; }\n\
               fn main() -> i64 {\n\
                   let u: [N]i64 = [1000i64; N];\n\
                   let v: [N]i64 = idf(u);\n\
                   return v[0] + v[1];\n\
               }\n";
    assert_eq!(all_engines_ret(src), 2000);
}

/// Run a module-tree fixture on the oracle, MIR, native, and native `-O2`;
/// assert every return agrees with the oracle's and return it.
fn all_engines_ret_dir(name: &str) -> i64 {
    let dir = moddir(name);
    let o_ret = match run_dir(&dir) {
        RunResult::Ok(r) => r.ret,
        RunResult::Fault(f) => panic!("{name} oracle faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => {
            panic!("{name} oracle check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("{name} oracle parse error: {}", d.to_json()),
    };
    for (label, res) in [
        ("mir", run_dir_mir(&dir)),
        ("native-noopt", run_dir_native(&dir)),
        ("native-opt", run_dir_native_opt(&dir)),
    ] {
        match res {
            MirRunResult::Ok(r) => assert_eq!(r.ret, o_ret, "{name}: {label} ret diverged from oracle"),
            MirRunResult::Fault(f) => panic!("{name}: {label} faulted: {}", f.to_json()),
            MirRunResult::Unsupported(e) => panic!("{name}: {label} unsupported: {e}"),
            MirRunResult::CheckErrors(d) => {
                panic!("{name}: {label} check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
            }
            MirRunResult::ParseError(d) => panic!("{name}: {label} parse error: {}", d.to_json()),
        }
    }
    o_ret
}

/// Instance-name mangling is injective across `_`-bearing identifiers and `::`
/// qualification (B1): named array lengths `a::b::N` (= 2) and `a::b_N` (= 3)
/// both flattened to `arr_a_b_N_i64` under the old `::` -> `_` scheme and
/// collapsed onto one instance (oracle fault / wrong MIR results); they must
/// monomorphize separately and agree everywhere.
#[test]
fn qualified_and_underscore_array_length_names_do_not_collide() {
    assert_eq!(all_engines_ret_dir("mangle_len_collision"), 80);
}

/// The nominal-name half of the same class (B1): struct type arguments
/// `a::b::C` and `a::b_C` both flattened to `Wrap$a_b_C`, so the second
/// instantiation reused the first's layout (wrong field reads). They must key
/// distinct `Wrap` instances and agree everywhere.
#[test]
fn qualified_and_underscore_type_names_do_not_collide() {
    assert_eq!(all_engines_ret_dir("mangle_ty_collision"), 49);
}

/// An array inside a generic ENUM payload keeps its length as well.
#[test]
fn array_in_generic_enum_payload_all_engines() {
    let src = "enum Opt[T] { Some(T), None, }\n\
               fn main() -> i64 {\n\
                   let o: Opt[[3]i64] = Opt::Some([1i64, 2, 3]);\n\
                   match o {\n\
                       Opt::Some(a) => { return a[0] * 100 + a[1] * 10 + a[2]; },\n\
                       Opt::None => { return -1; },\n\
                   }\n\
               }\n";
    assert_eq!(all_engines_ret(src), 123);
}

// ===========================================================================
// P5 (2026-08-03 ledger, re-opened): an all-unsuffixed array literal grounds
// its element type BEFORE the monomorphization shape is recorded — from the
// expected type when one exists, else to the `i64` default (design 0002 §0.1,
// the same default `unify` gives a bare scalar literal argument). While open,
// the annotation and the struct-literal shape instantiated one generic at TWO
// element types ({integer} vs i64): the oracle faulted, MIR/native refused.
// ===========================================================================

/// The exact ledger repro: a generic struct literal whose array field is
/// all-unsuffixed, instantiated under a `Wrap[[3]i64]` annotation. Must check
/// clean and run with `v == [4, 5, 6]` on every engine.
#[test]
fn unsuffixed_array_field_generic_struct_lit_all_engines() {
    let src = "struct Wrap[T] { v: T }\n\
               fn main() -> i64 {\n\
                   let w: Wrap[[3]i64] = Wrap { v: [4, 5, 6] };\n\
                   return w.v[0] * 100 + w.v[1] * 10 + w.v[2];\n\
               }\n";
    assert_eq!(all_engines_ret(src), 456);
}

/// No annotation at all: the field value alone pins the instance, so the
/// grounded `i64` default must be what the instance is recorded at.
#[test]
fn unsuffixed_array_field_unannotated_generic_struct_lit_all_engines() {
    let src = "struct Wrap[T] { v: T }\n\
               fn main() -> i64 {\n\
                   let w = Wrap { v: [4, 5, 6] };\n\
                   return w.v[0] * 100 + w.v[1] * 10 + w.v[2];\n\
               }\n";
    assert_eq!(all_engines_ret(src), 456);
}

/// A generic CALL with an all-unsuffixed array argument: the type argument
/// binds at the grounded `[2]i64`, matching the annotated landing slot.
#[test]
fn unsuffixed_array_argument_generic_call_all_engines() {
    let src = "fn idf[T: copy](x: T) -> T { return x; }\n\
               fn main() -> i64 {\n\
                   let b: [2]i64 = idf([1, 2]);\n\
                   let c = idf([30, 4]);\n\
                   return b[1] * 100 + c[0];\n\
               }\n";
    assert_eq!(all_engines_ret(src), 230);
}

/// A generic instantiated at a NON-default element type still requires the
/// evidence to agree: the value argument grounds to `[2]i64` (the default),
/// so a `Wrap[[2]u8]` annotation is a type mismatch — exactly as the scalar
/// rule treats `let w: Wrap[u8] = Wrap { v: 1 };` — never two instances.
#[test]
fn unsuffixed_array_field_against_u8_annotation_rejected() {
    assert_code(
        "struct Wrap[T] { v: T }\n\
         fn main() -> i64 {\n\
             let w: Wrap[[2]u8] = Wrap { v: [1, 2] };\n\
             return 0;\n\
         }\n",
        "E0703",
    );
}

/// A generic CALL argument is its own typing context (F1/F3): the landing
/// slot's annotation does not leak into the argument, so `[1, 2]` grounds to
/// the `[2]i64` default and the `[2]u8` slot is E0703 — deliberately mirroring
/// the scalar rule (`let a: u8 = idf(1);` is E0703), not collateral of the
/// leak fix.
#[test]
fn unsuffixed_array_argument_does_not_adopt_landing_slot_type() {
    assert_code(
        "fn idf[T: copy](x: T) -> T { return x; }\n\
         fn main() -> i64 {\n\
             let a: [2]u8 = idf([1, 2]);\n\
             return 0;\n\
         }\n",
        "E0703",
    );
}

/// Mixed suffixed/unsuffixed siblings in a generic struct slot (F11): the
/// `2u8` grounds the whole literal to `u8` (d88354c's running unification),
/// the instance binds at `[3]u8`, and the annotation agrees — u8 layout on
/// every engine.
#[test]
fn mixed_suffix_array_field_generic_struct_lit_all_engines() {
    let src = "struct Wrap[T] { v: T }\n\
               fn main() -> i64 {\n\
                   let w: Wrap[[3]u8] = Wrap { v: [1, 2u8, 3] };\n\
                   return conv i64 (w.v[0]) * 100 + conv i64 (w.v[1]) * 10 + conv i64 (w.v[2]);\n\
               }\n";
    assert_eq!(all_engines_ret(src), 123);
}

/// Non-generic engine agreement for the grounded-literal class: a `[2]u8`
/// field literal, a nested `[2][2]u8` literal, and a `[3]u8` repeat must
/// materialize at the annotated element type on every engine (the oracle's
/// repeat path used to lay out i64 slots for `[9; 3]` under a `[3]u8` slot).
#[test]
fn unsuffixed_literals_ground_to_u8_layout_all_engines() {
    let src = "struct S { a: [2]u8, b: i64 }\n\
               fn main() -> i64 {\n\
                   let s = S { a: [1, 2], b: 42 };\n\
                   let m: [2][2]u8 = [[3, 4], [5, 6]];\n\
                   let r: [3]u8 = [9; 3];\n\
                   return conv i64 (s.a[1]) * 100 + conv i64 (m[1][0]) * 10 + conv i64 (r[2]);\n\
               }\n";
    assert_eq!(all_engines_ret(src), 259);
}
