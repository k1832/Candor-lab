//! Borrowed-element iteration — the `RefIndexed` protocol behind `for read x in
//! read coll` (design 0015). A `for read` walk over a collection of a NON-`copy`
//! element (the case `Indexed`/`Iter` cannot express) binds `x` to a `read Item`
//! reborrow of each element: no copy, no move, each element dropped exactly once.
//! The functional walk runs byte-exact on the tree-walker, MIR, and both native
//! backends (LLVM covered transitively by `tests/llvm.rs`'s corpus gate over the
//! shared `tests/fixtures/run/refindexed_native.cnr`). The soundness payoff — the
//! loop's `read` loan freezes the collection — rejects mutation-during-iteration
//! and any escape of the borrowed yield, by the inherited loan machinery.

use candor::diag::Severity;
use candor::{
    check_source_real, run_source_real, run_source_real_mir, run_source_real_native,
    run_source_real_native_opt, MirRunResult, RunResult,
};

fn fixture() -> String {
    let path = format!("{}/tests/fixtures/run/refindexed_native.cnr", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn oracle(src: &str) -> candor::interp::Run {
    match run_source_real(src) {
        RunResult::Ok(r) => r,
        RunResult::Fault(f) => panic!("oracle faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => {
            panic!("oracle check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("oracle parse error: {}", d.to_json()),
    }
}

fn mir_run(r: MirRunResult, label: &str) -> candor::interp::Run {
    match r {
        MirRunResult::Ok(run) => run,
        MirRunResult::Fault(f) => panic!("{label} faulted: {}", f.to_json()),
        MirRunResult::Unsupported(m) => panic!("{label} unsupported: {m}"),
        MirRunResult::CheckErrors(d) => {
            panic!("{label} check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        MirRunResult::ParseError(d) => panic!("{label} parse error: {}", d.to_json()),
    }
}

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

/// The `interface RefIndexed` + a non-`copy`, drop-hooked element `W` + the
/// index-backed `Bag` with a `get_ref` that reborrows a slot from `read self`
/// (the `arena_get` shape). Shared by the soundness cases below.
const PRELUDE: &str = r#"
interface RefIndexed {
    type Item;
    fn count(read self) -> usize;
    fn get_ref(read self, i: usize) -> read Item;
}
struct W { id: i64 }
drop(write self) { trace(self.id); }
struct Bag { a: W, b: W, c: W, n: usize }
impl RefIndexed for Bag {
    type Item = W;
    fn count(read self) -> usize { return self.n; }
    fn get_ref(read self, i: usize) -> read W {
        if i == 0usize { return read self.a; }
        if i == 1usize { return read self.b; }
        return read self.c;
    }
}
fn setn(b: write Bag, k: usize) -> unit { b.*.n = k; }
"#;

#[test]
fn refindexed_for_read_borrows_noncopy_elements_all_engines() {
    let src = fixture();
    let o = oracle(&src);
    // Three field reads through the per-element borrow (10, 20, 30), the sum (60),
    // then the three elements' drop hooks (reverse field order 30, 20, 10) — each
    // element dropped exactly once: the borrow-walk moved and copied nothing.
    assert_eq!(o.ret, 60, "sum of `x.id` read through each `read W` reborrow");
    assert_eq!(o.trace, vec![10, 20, 30, 60, 30, 20, 10], "oracle read + drop trace");

    for (label, r) in [
        ("mir", run_source_real_mir(&src)),
        ("native-noopt", run_source_real_native(&src)),
        ("native-opt", run_source_real_native_opt(&src)),
    ] {
        let run = mir_run(r, label);
        assert_eq!(run.ret, o.ret, "{label} return diverged from oracle");
        assert_eq!(run.trace, o.trace, "{label} borrowed-walk trace diverged from oracle");
    }
}

#[test]
fn refindexed_mutation_during_iteration_rejected() {
    // The loop-local `read` loan on `bag` spans the loop (used by `count`/`get_ref`
    // each turn), so a `write bag` in the body conflicts by XOR (0001 §2.2) — the
    // iterator-invalidation guarantee, caught by the EXISTING loan machinery, no new
    // aliasing rule (design 0015 §4.3).
    let src = format!(
        "{PRELUDE}\nfn main() -> i64 {{\n  \
         let mut bag: Bag = Bag {{ a: W {{ id: 10 }}, b: W {{ id: 20 }}, c: W {{ id: 30 }}, n: 3 }};\n  \
         let mut sum: i64 = 0;\n  \
         for read x in read bag {{ sum = sum + x.id; setn(write bag, 0usize); }}\n  \
         return sum;\n}}"
    );
    let e = errors(&src);
    assert!(
        e.iter().any(|c| c == "E0801" || c == "E0803"),
        "expected a conflicting-borrow error rejecting mutation-during-iteration, got {e:?}"
    );
}

#[test]
fn refindexed_escaping_yield_rejected() {
    // The borrowed yield `x` may not escape the loop and then coexist with a mutation
    // of `bag`: assigning `x` to an outer local keeps a loan derived from `bag` live,
    // so the post-loop `write bag` conflicts (design 0015 §5 case 2 — the escape the
    // now-fixed loan-provenance discipline forbids; `for read` needs no special rule).
    let src = format!(
        "{PRELUDE}\nfn main() -> i64 {{\n  \
         let mut bag: Bag = Bag {{ a: W {{ id: 10 }}, b: W {{ id: 20 }}, c: W {{ id: 30 }}, n: 3 }};\n  \
         let other: W = W {{ id: 99 }};\n  \
         let mut esc: read W = read other;\n  \
         for read x in read bag {{ esc = x; }}\n  \
         setn(write bag, 0usize);\n  \
         return esc.id;\n}}"
    );
    let e = errors(&src);
    assert!(
        e.iter().any(|c| c == "E0801" || c == "E0802" || c == "E0803"),
        "expected a conflicting-borrow error rejecting the escaped borrowed yield, got {e:?}"
    );
}
