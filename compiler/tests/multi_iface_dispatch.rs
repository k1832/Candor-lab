//! Soundness regression: when one type impls TWO interfaces that share a method
//! name, dispatch must run exactly the impl the checker resolved — via the
//! bound in a generic context (`[T: A]` vs `[T: B]`), and via the coherent
//! first-match on a direct call. Before the fix, dispatch keyed only on
//! `(target, method)`, so the runtime ran whichever impl registered last,
//! diverging from the checker's resolution (a type-confusion hole when the two
//! methods have different return types).
//!
//! Every case is byte-exact across all five engines (tree-walk oracle, MIR
//! interp, Cranelift no-opt, Cranelift opt, LLVM -O2), the LLVM leg via the
//! trace channel — the same harness shape as `tests/ord.rs`.

use candor::{
    compile_path_llvm, run_source_real, run_source_real_mir, run_source_real_native,
    run_source_real_native_opt, MirRunResult, RunResult,
};
use std::path::Path;
use std::process::Command;

fn oracle_trace(src: &str) -> (i64, Vec<i64>) {
    match run_source_real(src) {
        RunResult::Ok(r) => (r.ret, r.trace),
        RunResult::Fault(f) => panic!("oracle faulted: {}\n{src}", f.to_json()),
        RunResult::CheckErrors(d) => {
            panic!("oracle check errors: {:?}\n{src}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("oracle parse error: {}\n{src}", d.to_json()),
    }
}

fn mir_ret_trace(r: MirRunResult, label: &str, src: &str) -> (i64, Vec<i64>) {
    match r {
        MirRunResult::Ok(run) => (run.ret, run.trace),
        MirRunResult::Fault(f) => panic!("{label} faulted: {}\n{src}", f.to_json()),
        MirRunResult::Unsupported(e) => panic!("{label} unsupported: {e}\n{src}"),
        MirRunResult::CheckErrors(d) => panic!("{label} check errors: {:?}\n{src}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        MirRunResult::ParseError(d) => panic!("{label} parse error: {}\n{src}", d.to_json()),
    }
}

fn clang_available() -> bool {
    Command::new("clang").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn llvm_trace(src: &str, tag: &str) -> Option<Vec<i64>> {
    if !clang_available() {
        return None;
    }
    let dir = std::env::temp_dir();
    let srcp = dir.join(format!("candor-mid-{}-{}.cnr", std::process::id(), tag));
    let outp = dir.join(format!("candor-mid-{}-{}", std::process::id(), tag));
    std::fs::write(&srcp, src).unwrap();
    compile_path_llvm(Path::new(&srcp), &outp).expect("LLVM compile should succeed");
    let output = Command::new(&outp).output().expect("run compiled program");
    let _ = std::fs::remove_file(&srcp);
    let _ = std::fs::remove_file(&outp);
    let trace = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.trim().parse::<i64>().expect("trace line is an integer"))
        .collect();
    Some(trace)
}

/// Run `src` through all five engines; assert byte-exact `ret` (four in-process
/// engines) and traced sequence (all five, LLVM via trace).
fn all_engines(src: &str, tag: &str) -> (i64, Vec<i64>) {
    let (o_ret, o_trace) = oracle_trace(src);
    let (m_ret, m_trace) = mir_ret_trace(run_source_real_mir(src), "mir", src);
    let (n_ret, n_trace) = mir_ret_trace(run_source_real_native(src), "native-noopt", src);
    let (p_ret, p_trace) = mir_ret_trace(run_source_real_native_opt(src), "native-opt", src);
    for (label, ret, trace) in [
        ("mir", m_ret, &m_trace),
        ("native-noopt", n_ret, &n_trace),
        ("native-opt", p_ret, &p_trace),
    ] {
        assert_eq!(ret, o_ret, "{label} ret diverged from oracle for:\n{src}");
        assert_eq!(trace, &o_trace, "{label} trace diverged from oracle for:\n{src}");
    }
    if let Some(l_trace) = llvm_trace(src, tag) {
        assert_eq!(l_trace, o_trace, "llvm trace diverged from oracle for:\n{src}");
    }
    (o_ret, o_trace)
}

const IFACES: &str = concat!(
    "interface A { fn tag(read self) -> i64; }\n",
    "interface B { fn tag(read self) -> i64; }\n",
    "struct W { x: i64 }\n",
    "impl A for W { fn tag(read self) -> i64 { return 1; } }\n",
    "impl B for W { fn tag(read self) -> i64 { return 2; } }\n",
);

#[test]
fn bound_a_dispatches_a_not_last_registered() {
    // The bound is `T: A`, so `x.tag()` MUST be `A::tag` -> 1, even though `B`'s
    // impl registered last (the pre-fix bug returned 2 on every engine).
    let src = format!(
        "{IFACES}\
         fn call_a[T: A](x: read T) -> i64 {{ return x.tag(); }}\n\
         fn main() -> i64 {{ let w: W = W {{ x: 0 }}; let r: i64 = call_a(read w); trace(r); return r; }}\n"
    );
    let (ret, trace) = all_engines(&src, "bound_a");
    assert_eq!(ret, 1);
    assert_eq!(trace, vec![1]);
}

#[test]
fn bound_b_dispatches_b() {
    // The mirror bound `T: B` selects `B::tag` -> 2.
    let src = format!(
        "{IFACES}\
         fn call_b[T: B](x: read T) -> i64 {{ return x.tag(); }}\n\
         fn main() -> i64 {{ let w: W = W {{ x: 0 }}; let r: i64 = call_b(read w); trace(r); return r; }}\n"
    );
    let (ret, trace) = all_engines(&src, "bound_b");
    assert_eq!(ret, 2);
    assert_eq!(trace, vec![2]);
}

#[test]
fn both_bounds_in_one_program_dispatch_independently() {
    // Both instantiations coexist: `A` -> 1, `B` -> 2, summed 10*a + b.
    let src = format!(
        "{IFACES}\
         fn call_a[T: A](x: read T) -> i64 {{ return x.tag(); }}\n\
         fn call_b[T: B](x: read T) -> i64 {{ return x.tag(); }}\n\
         fn main() -> i64 {{\n\
           let w: W = W {{ x: 0 }};\n\
           let a: i64 = call_a(read w); trace(a);\n\
           let b: i64 = call_b(read w); trace(b);\n\
           return a * 10 + b;\n\
         }}\n"
    );
    let (ret, trace) = all_engines(&src, "both_bounds");
    assert_eq!(ret, 12);
    assert_eq!(trace, vec![1, 2]);
}

#[test]
fn type_confusion_bound_runs_checker_approved_impl() {
    // `A::tag -> i64` vs `B::tag -> S` (a struct), both on `W`. Under `[T: A]`
    // the i64-returning `A::tag` MUST run; dispatching `B::tag` here would read a
    // struct's bytes as an i64 (or vice versa) — the type-confusion the checker
    // approved but the runtime violated before the fix.
    let src = concat!(
        "interface A { fn tag(read self) -> i64; }\n",
        "interface B { fn tag(read self) -> S; }\n",
        "struct S { a: i64, b: i64 }\n",
        "struct W { x: i64 }\n",
        "impl A for W { fn tag(read self) -> i64 { return 7; } }\n",
        "impl B for W { fn tag(read self) -> S { return S { a: 111, b: 222 }; } }\n",
        "fn call_a[T: A](x: read T) -> i64 { return x.tag(); }\n",
        "fn main() -> i64 { let w: W = W { x: 0 }; let r: i64 = call_a(read w); trace(r); return r; }\n",
    );
    let (ret, trace) = all_engines(src, "type_confusion");
    assert_eq!(ret, 7);
    assert_eq!(trace, vec![7]);
}

#[test]
fn direct_call_first_match_is_coherent_across_engines() {
    // A direct, non-generic ambiguous call `w.tag()` resolves by the checker's
    // deterministic first-match (impl `A` precedes `B`) -> 1, and every engine's
    // dispatch agrees (no last-registered divergence).
    let src = format!(
        "{IFACES}\
         fn main() -> i64 {{ let w: W = W {{ x: 0 }}; let r: i64 = w.tag(); trace(r); return r; }}\n"
    );
    let (ret, trace) = all_engines(&src, "direct_first");
    assert_eq!(ret, 1);
    assert_eq!(trace, vec![1]);
}
