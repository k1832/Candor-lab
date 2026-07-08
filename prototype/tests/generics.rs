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
