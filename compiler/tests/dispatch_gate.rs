//! Design-0018 §5 validation campaign — **gate (d)**, the dispatch-consistency
//! check (§5.2). For every interface-method call in a corpus program, the impl
//! the CHECKER resolved (`resolve`) must equal the impl every engine actually
//! DISPATCHED, and the program's output must equal an author-declared expected
//! value. This is the gate that catches the §2.2 bug class — "all engines agree
//! on the WRONG impl" (checker resolved `A::tag`, every engine ran `B::tag`) —
//! which the plain five-engine agreement gate (a) is structurally blind to (§6).
//!
//! ## What each half asserts
//! - **`dispatch = resolve` (compiler-internal, §4.1).** `resolve` is the set of
//!   interfaces the checker selected, read from its recorded monomorphization
//!   shapes via `resolved_interfaces_real` — computed independently of the runtime
//!   dispatch tables. `dispatch` is the set of `(target, interface, method)` keys
//!   each engine's dispatch path resolved, captured by the off-by-default
//!   `dispatch_trace` instrumentation. If dispatch drifts from resolution (the
//!   §2.2 regression: key on `(target, method)` instead of the resolved interface),
//!   the two disagree — a red gate — even though every engine still agrees with
//!   every other (gate (a) stays green). See `regression_reasoning` below.
//! - **Author-declared expected output (partial external oracle, §6.4).** Each
//!   program carries a declared `ret` + `trace`, checked byte-exact on all engines,
//!   pinning the RIGHT answer independently of the compiler's own `resolve`.
//!
//! ## Engines gated
//! Output (`ret` + traced sequence) is asserted byte-exact on the tree-walker
//! oracle, the MIR interpreter, and both Cranelift backends (no-opt / opt), plus
//! LLVM when `clang` is present — gate (a). Dispatch keys are captured directly
//! from the tree-walker (execution order) and the MIR lowering (the resolution the
//! native backends inherit unchanged), so the two directly-introspected engines
//! carry the native/LLVM backends transitively via that byte-exact output gate;
//! no native dispatch introspection is claimed. Keys are compared as SETS: the
//! tree-walker records per execution and MIR per lowered call site, so their
//! orders differ, while execution ORDER is already pinned byte-exact by the
//! `trace` oracle above.

use candor::diag::Severity;
use candor::{
    check_dir, check_source_real, compile_path_llvm, dispatch_trace, resolved_interfaces_real,
    run_source_real, run_source_real_mir, run_source_real_native, run_source_real_native_opt,
    MirRunResult, RunResult,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

type Key = (String, String, String);

fn oracle(src: &str) -> (i64, Vec<i64>) {
    match run_source_real(src) {
        RunResult::Ok(r) => (r.ret, r.trace),
        RunResult::Fault(f) => panic!("oracle faulted: {}\n{src}", f.to_json()),
        RunResult::CheckErrors(d) => panic!("oracle check errors: {:?}\n{src}", codes_of(&d)),
        RunResult::ParseError(d) => panic!("oracle parse error: {}\n{src}", d.to_json()),
    }
}

fn mir_ret_trace(r: MirRunResult, label: &str, src: &str) -> (i64, Vec<i64>) {
    match r {
        MirRunResult::Ok(run) => (run.ret, run.trace),
        MirRunResult::Fault(f) => panic!("{label} faulted: {}\n{src}", f.to_json()),
        MirRunResult::Unsupported(e) => panic!("{label} unsupported: {e}\n{src}"),
        MirRunResult::CheckErrors(d) => panic!("{label} check errors: {:?}\n{src}", codes_of(&d)),
        MirRunResult::ParseError(d) => panic!("{label} parse error: {}\n{src}", d.to_json()),
    }
}

fn codes_of(d: &[candor::diag::Diag]) -> Vec<String> {
    d.iter().map(|x| x.code.clone()).collect()
}

fn clang_available() -> bool {
    Command::new("clang").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn llvm_trace(src: &str, tag: &str) -> Option<Vec<i64>> {
    if !clang_available() {
        return None;
    }
    let dir = std::env::temp_dir();
    let srcp = dir.join(format!("candor-gate-{}-{}.cnr", std::process::id(), tag));
    let outp = dir.join(format!("candor-gate-{}-{}", std::process::id(), tag));
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

/// Run `src` under the tree-walker with dispatch recording on; return
/// `(ret, trace, executed dispatch keys)`.
fn tree_dispatch(src: &str) -> (i64, Vec<i64>, Vec<Key>) {
    dispatch_trace::start();
    let r = run_source_real(src);
    let keys = dispatch_trace::take();
    let (ret, trace) = match r {
        RunResult::Ok(run) => (run.ret, run.trace),
        RunResult::Fault(f) => panic!("tree faulted: {}\n{src}", f.to_json()),
        RunResult::CheckErrors(d) => panic!("tree check errors: {:?}\n{src}", codes_of(&d)),
        RunResult::ParseError(d) => panic!("tree parse error: {}\n{src}", d.to_json()),
    };
    (ret, trace, keys)
}

/// Run `src` through the MIR engine with dispatch recording on; return
/// `(ret, trace, resolved dispatch keys)`.
fn mir_dispatch(src: &str) -> (i64, Vec<i64>, Vec<Key>) {
    dispatch_trace::start();
    let r = run_source_real_mir(src);
    let keys = dispatch_trace::take();
    let (ret, trace) = mir_ret_trace(r, "mir", src);
    (ret, trace, keys)
}

/// Gate (d)'s core consistency predicate (§4.1), extracted so it is directly
/// testable: the set of interfaces the checker RESOLVED must equal the set every
/// engine DISPATCHED. The §2.2 regression keys dispatch on `(target, method)`
/// instead of the resolved interface, so the two sets drift and this returns
/// `Err`. The live-regression check that would prove the gate fires cannot run in
/// CI (it edits the compiler); `gate_d_comparator_rejects_dispatch_resolve_drift`
/// feeds this predicate the exact mismatch the bug produces and asserts it
/// rejects, standing in for that check structurally.
fn dispatch_matches_resolve(
    resolved: &BTreeSet<String>,
    dispatched: &BTreeSet<String>,
) -> Result<(), String> {
    if resolved == dispatched {
        Ok(())
    } else {
        Err(format!(
            "resolve != dispatch interfaces: resolved {resolved:?}, dispatched {dispatched:?}"
        ))
    }
}

#[test]
fn gate_d_comparator_rejects_dispatch_resolve_drift() {
    // Structural sensitivity check for gate (d): the §2.2 regression makes an engine
    // dispatch a DIFFERENT interface than the checker resolved (checker picked `A`,
    // engine ran `B`). That live regression can't run in CI, so we assert the gate's
    // own comparator REJECTS the exact drift the bug produces...
    let resolved = BTreeSet::from(["A".to_string()]);
    let dispatched_bug = BTreeSet::from(["B".to_string()]);
    assert!(
        dispatch_matches_resolve(&resolved, &dispatched_bug).is_err(),
        "comparator must reject a resolve/dispatch interface mismatch"
    );
    // ...and ACCEPTS an agreeing pair, so the rejection above is not vacuous.
    assert!(dispatch_matches_resolve(&resolved, &resolved).is_ok());
}

/// The full gate (d) assertion for one corpus program.
///
/// `keys` is the author-declared set of `(target, interface, method)` dispatch
/// keys the program's calls MUST resolve to — declared by reasoning about which
/// impl the bound / coherent-first-match rule selects, independent of the
/// compiler. `ret`/`trace` are the author-declared output oracle.
fn gate_d(
    tag: &str,
    src: &str,
    ret: i64,
    trace: &[i64],
    keys: &[(&str, &str, &str)],
) {
    // --- gate (a) + external output oracle: byte-exact across every engine ---
    let (o_ret, o_trace) = oracle(src);
    let (m_ret, m_trace) = mir_ret_trace(run_source_real_mir(src), "mir", src);
    let (n_ret, n_trace) = mir_ret_trace(run_source_real_native(src), "native-noopt", src);
    let (p_ret, p_trace) = mir_ret_trace(run_source_real_native_opt(src), "native-opt", src);
    for (label, r, t) in [
        ("mir", m_ret, &m_trace),
        ("native-noopt", n_ret, &n_trace),
        ("native-opt", p_ret, &p_trace),
    ] {
        assert_eq!(r, o_ret, "{label} ret diverged from oracle for:\n{src}");
        assert_eq!(t, &o_trace, "{label} trace diverged from oracle for:\n{src}");
    }
    if let Some(l_trace) = llvm_trace(src, tag) {
        assert_eq!(l_trace, o_trace, "llvm trace diverged from oracle for:\n{src}");
    }
    assert_eq!(o_ret, ret, "declared ret mismatch for {tag}");
    assert_eq!(o_trace, trace, "declared trace mismatch for {tag}");

    // --- gate (d): dispatch = resolve, plus author-declared dispatch keys ---
    let declared: BTreeSet<Key> =
        keys.iter().map(|(t, i, m)| (t.to_string(), i.to_string(), m.to_string())).collect();

    let (t_ret, t_trace, t_keys) = tree_dispatch(src);
    let (mi_ret, mi_trace, mi_keys) = mir_dispatch(src);
    assert_eq!((t_ret, &t_trace), (ret, &trace.to_vec()), "tree output for {tag}");
    assert_eq!((mi_ret, &mi_trace), (ret, &trace.to_vec()), "mir output for {tag}");

    let t_set: BTreeSet<Key> = t_keys.into_iter().collect();
    let mi_set: BTreeSet<Key> = mi_keys.into_iter().collect();
    assert_eq!(t_set, declared, "tree-walker dispatched keys != declared for {tag}");
    assert_eq!(mi_set, declared, "MIR dispatched keys != declared for {tag}");

    // dispatch = resolve: the interfaces the checker RESOLVED must equal the
    // interfaces every engine DISPATCHED. Under the §2.2 regression these diverge.
    let resolved = resolved_interfaces_real(src).expect("resolve");
    let dispatched_ifaces: BTreeSet<String> = declared.iter().map(|(_, i, _)| i.clone()).collect();
    dispatch_matches_resolve(&resolved, &dispatched_ifaces).unwrap_or_else(|e| panic!("{e} for {tag}"));
}

// Shared §2.2 shape: one type carrying two interfaces with a same-named method.
const AB_W: &str = concat!(
    "interface A { fn tag(read self) -> i64; }\n",
    "interface B { fn tag(read self) -> i64; }\n",
    "struct W { x: i64 }\n",
    "impl A for W { fn tag(read self) -> i64 { return 1; } }\n",
    "impl B for W { fn tag(read self) -> i64 { return 2; } }\n",
);

fn errors(src: &str) -> Vec<String> {
    match check_source_real(src) {
        Ok(d) => d.into_iter().filter(|x| x.severity == Severity::Error).map(|x| x.code).collect(),
        Err(p) => vec![p.code],
    }
}

// =====================================================================
// Group 1 — the §2.2 regression shape (bound-directed vs direct)
// =====================================================================

#[test]
fn bound_a_dispatches_a() {
    // Under `[T: A]`, `x.tag()` MUST resolve+dispatch `A::tag` -> 1, though `B`
    // registered last. A regression to `(target, method)` keying would run `B`.
    let src = format!(
        "{AB_W}\
         fn call_a[T: A](x: read T) -> i64 {{ return x.tag(); }}\n\
         fn main() -> i64 {{ let w: W = W {{ x: 0 }}; let r: i64 = call_a(read w); trace(r); return r; }}\n"
    );
    gate_d("bound_a", &src, 1, &[1], &[("W", "A", "tag")]);
}

#[test]
fn bound_b_dispatches_b() {
    let src = format!(
        "{AB_W}\
         fn call_b[T: B](x: read T) -> i64 {{ return x.tag(); }}\n\
         fn main() -> i64 {{ let w: W = W {{ x: 0 }}; let r: i64 = call_b(read w); trace(r); return r; }}\n"
    );
    gate_d("bound_b", &src, 2, &[2], &[("W", "B", "tag")]);
}

#[test]
fn both_bounds_and_direct_in_one_program() {
    // A::tag (bound), B::tag (bound), and a direct `w.tag()` -> coherent first-match
    // A. Three call sites, two interfaces; the §2.2 bug would collapse all three to
    // one interface. Distinct expected outputs (1, 2, 1) distinguish which ran.
    let src = format!(
        "{AB_W}\
         fn call_a[T: A](x: read T) -> i64 {{ return x.tag(); }}\n\
         fn call_b[T: B](x: read T) -> i64 {{ return x.tag(); }}\n\
         fn main() -> i64 {{\n\
           let w: W = W {{ x: 0 }};\n\
           let a: i64 = call_a(read w); trace(a);\n\
           let b: i64 = call_b(read w); trace(b);\n\
           let d: i64 = w.tag(); trace(d);\n\
           return a * 100 + b * 10 + d;\n\
         }}\n"
    );
    gate_d("both_and_direct", &src, 121, &[1, 2, 1], &[("W", "A", "tag"), ("W", "B", "tag")]);
}

#[test]
fn type_confusion_bound_runs_checker_approved_impl() {
    // `A::tag -> i64` vs `B::tag -> S` (a struct), both on `W`. Under `[T: A]` the
    // i64-returning `A::tag` MUST run; dispatching `B::tag` would read a struct as
    // an i64 — the type confusion the checker blessed but the runtime violated
    // pre-fix. `dispatch = resolve` names it directly.
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
    gate_d("type_confusion", src, 7, &[7], &[("W", "A", "tag")]);
}

// =====================================================================
// Group 2 — scalar impls, generic-at-two-types, nested adapters
// =====================================================================

#[test]
fn scalar_impls_two_interfaces_on_i64() {
    // `impl A for i64` and `impl B for i64` (the §2.3 scalar-target shape). The
    // bound picks the interface; dispatch keys on the scalar spelling `i64`.
    let src = concat!(
        "interface A { fn tag(read self) -> i64; }\n",
        "interface B { fn tag(read self) -> i64; }\n",
        "impl A for i64 { fn tag(read self) -> i64 { return 10; } }\n",
        "impl B for i64 { fn tag(read self) -> i64 { return 20; } }\n",
        "fn ca[T: A](x: read T) -> i64 { return x.tag(); }\n",
        "fn cb[T: B](x: read T) -> i64 { return x.tag(); }\n",
        "fn main() -> i64 { let n: i64 = 5; let a: i64 = ca(read n); trace(a); let b: i64 = cb(read n); trace(b); return a + b; }\n",
    );
    gate_d("scalar", src, 30, &[10, 20], &[("i64", "A", "tag"), ("i64", "B", "tag")]);
}

#[test]
fn generic_body_instantiated_at_two_types() {
    // One generic body `go[T: Show]` instantiated at `P` and `Q`, each with its own
    // `Show` impl. One resolved interface (`Show`), two dispatch targets.
    let src = concat!(
        "interface Show { fn sh(read self) -> i64; }\n",
        "struct P { x: i64 }\n",
        "struct Q { y: i64 }\n",
        "impl Show for P { fn sh(read self) -> i64 { return 1; } }\n",
        "impl Show for Q { fn sh(read self) -> i64 { return 2; } }\n",
        "fn go[T: Show](x: read T) -> i64 { return x.sh(); }\n",
        "fn main() -> i64 { let p: P = P { x: 0 }; let q: Q = Q { y: 0 }; let a: i64 = go(read p); trace(a); let b: i64 = go(read q); trace(b); return a * 10 + b; }\n",
    );
    gate_d("gen2types", src, 12, &[1, 2], &[("P", "Show", "sh"), ("Q", "Show", "sh")]);
}

#[test]
fn nested_composed_generic_adapter() {
    // Multi-hop dispatch: `get(w)` -> `Wrap[Leaf]::val` -> `self.inner.val()` ->
    // `Leaf::val`. Both hops resolve+dispatch `Src`, on distinct targets.
    let src = concat!(
        "interface Src { fn val(read self) -> i64; }\n",
        "struct Leaf { n: i64 }\n",
        "impl Src for Leaf { fn val(read self) -> i64 { return self.n; } }\n",
        "struct Wrap[T: Src] { inner: T }\n",
        "impl[T: Src] Src for Wrap[T] { fn val(read self) -> i64 { return self.inner.val() + 100; } }\n",
        "fn get[T: Src](x: read T) -> i64 { return x.val(); }\n",
        "fn main() -> i64 { let l: Leaf = Leaf { n: 7 }; let w: Wrap[Leaf] = Wrap { inner: l }; let r: i64 = get(read w); trace(r); return r; }\n",
    );
    gate_d(
        "nested",
        src,
        107,
        &[107],
        &[("Wrap$Leaf", "Src", "val"), ("Leaf", "Src", "val")],
    );
}

// =====================================================================
// Group 3 — coherence rejections stay rejections (gate (b))
// =====================================================================

#[test]
fn overlap_same_interface_twice_rejected() {
    // Two impls of ONE interface for ONE type -> E1009. This is the "two impls both
    // selectable for one (interface, type)" shape the mechanical design-kill limb
    // targets: coherence rejects it, so `resolve` stays single-valued.
    let src = concat!(
        "interface A { fn tag(read self) -> i64; }\n",
        "struct W { x: i64 }\n",
        "impl A for W { fn tag(read self) -> i64 { return 1; } }\n",
        "impl A for W { fn tag(read self) -> i64 { return 2; } }\n",
        "fn main() -> i64 { return 0; }\n",
    );
    assert!(errors(src).contains(&"E1009".to_string()), "expected E1009, got {:?}", errors(src));
}

#[test]
fn generic_and_concrete_overlap_rejected() {
    // `impl[T] A for Pair[T]` and `impl A for Pair[i64]` unify on `Pair[i64]` ->
    // E1009 (§2.3). Two heads that overlap are rejected, not silently picked.
    let src = concat!(
        "interface A { fn tag(read self) -> i64; }\n",
        "struct Pair[T] { a: T, b: T }\n",
        "impl[T] A for Pair[T] { fn tag(read self) -> i64 { return 1; } }\n",
        "impl A for Pair[i64] { fn tag(read self) -> i64 { return 2; } }\n",
        "fn main() -> i64 { return 0; }\n",
    );
    assert!(errors(src).contains(&"E1009".to_string()), "expected E1009, got {:?}", errors(src));
}

fn moddir(name: &str) -> PathBuf {
    PathBuf::from(format!("{}/tests/fixtures/modules/{name}", env!("CARGO_MANIFEST_DIR")))
}

#[test]
fn scalar_orphan_across_modules_rejected() {
    // `impl Ord for i64` in a module owning neither `Ord` nor `i64` -> orphan
    // E1013 (§2.3, the orphan-rule half of gate (b)). Reuses the shared
    // module-tree fixture.
    let codes = match check_dir(&moddir("bad_orphan_scalar")) {
        Ok(d) => d.into_iter().filter(|x| x.severity == Severity::Error).map(|x| x.code).collect::<Vec<_>>(),
        Err(d) => vec![d.code],
    };
    assert!(codes.contains(&"E1013".to_string()), "expected E1013, got {codes:?}");
}

// =====================================================================
// Group 4 — the mechanical design-kill limb (§5.3)
// =====================================================================

#[test]
fn design_kill_limb_two_selectable_impls_is_unconstructible() {
    // The mechanical design-kill fires if two DISTINCT LEGAL impls are both validly
    // selectable for ONE call — an ambiguity `resolve` cannot single-value. We try
    // to construct it and assert the compiler REJECTS every attempt, so `resolve`
    // stays single-valued:
    //   1. Two impls of the same (interface, type): overlap E1009 (above).
    //   2. Two impls of DIFFERENT interfaces sharing a method name: NOT an
    //      ambiguity — the resolution rule single-values it (bound in a generic
    //      context, coherent first-match on a direct call). So a direct `w.tag()`
    //      with both `A` and `B` impl'd is ACCEPTED and deterministic, not a kill.
    // If (1) ever type-checked, or (2) were reported ambiguous with two impls
    // silently both selectable, that would be the AUTO DESIGN-KILL finding.
    let overlap = concat!(
        "interface A { fn tag(read self) -> i64; }\n",
        "struct W { x: i64 }\n",
        "impl A for W { fn tag(read self) -> i64 { return 1; } }\n",
        "impl A for W { fn tag(read self) -> i64 { return 2; } }\n",
        "fn main() -> i64 { let w: W = W { x: 0 }; return w.tag(); }\n",
    );
    assert!(
        errors(overlap).contains(&"E1009".to_string()),
        "AUTO DESIGN-KILL: two impls of one (interface, type) were NOT rejected: {:?}",
        errors(overlap)
    );

    // The two-interface direct call: single-valued by first-match, accepted, runs A.
    let direct = format!(
        "{AB_W}fn main() -> i64 {{ let w: W = W {{ x: 0 }}; let r: i64 = w.tag(); trace(r); return r; }}\n"
    );
    assert!(errors(&direct).is_empty(), "two-interface direct call should be single-valued, got {:?}", errors(&direct));
    gate_d("kill_direct_singlevalued", &direct, 1, &[1], &[("W", "A", "tag")]);
}
