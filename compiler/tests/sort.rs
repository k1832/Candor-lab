//! In-place generic `sort[T: copy]` over `Vec[T]` by a first-class comparator.
//! Iterative introsort (median-of-three quicksort partitioning, insertion sort
//! below a 24-element cutoff, bottom-up-heapsort fallback on depth-budget
//! exhaustion, explicit range stack — no recursion): O(n log n) worst case with
//! no scratch buffer, which matters because the signature carries no allocator
//! handle and a `Vec` does not expose its own. The comparator is an ordinary
//! `fn(read i64, read i64) -> bool` value, so the
//! SAME Vec sorts ascending or descending purely by which comparator is passed —
//! the ascending/descending pair below proves the order is comparator-driven and
//! not hard-coded. Every case is checked byte-exact across all five engines
//! (tree-walk oracle, MIR interp, Cranelift no-opt, Cranelift opt, LLVM -O2) via
//! the same trace-channel harness as `tests/iteration.rs`.
//!
//! `sort` is generic over the element type `T` (bounded `copy`, since the sort
//! shuffles elements by value). It is exercised at two instantiations: the
//! integer cases below (`Vec[i64]`, `T` inferred at the call) prove byte-exact
//! agreement with the earlier monomorphic form, and the `Item`-struct cases
//! (`Vec[Item]`, sorted by a field) prove the generic form lowers correctly for
//! a non-scalar `T` — both byte-exact across all five engines.
//!
//! The adversarial-pattern cases (already-sorted, reverse-sorted, organ-pipe,
//! all-equal, sawtooth) smoke the pivot selection and the equal-stopping scans
//! at ~500 elements; the pinned `inv`/`sum`/`wsum` traces are recomputed in
//! Rust from the same generators.

use candor::{
    check_source_real, compile_path_llvm, run_source_real, run_source_real_mir,
    run_source_real_native, run_source_real_native_opt, MirRunResult, RunResult,
};
use candor::diag::Severity;
use std::path::Path;
use std::process::Command;

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
    assert!(e.is_empty(), "expected clean, got {e:?}\n{src}");
}

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
    let srcp = dir.join(format!("candor-sort-{}-{}.cnr", std::process::id(), tag));
    let outp = dir.join(format!("candor-sort-{}-{}", std::process::id(), tag));
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

/// Run `src` through all five engines and assert byte-exact agreement on `ret`
/// (four in-process engines) and the traced sequence (all five, LLVM via trace).
fn all_engines(src: &str, tag: &str) -> (i64, Vec<i64>) {
    assert_clean(src);
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

// A counting bump allocator (mirrors tests/vec.rs) plus the generic introsort
// `sort` and two i64 comparators. `less_int` orders ascending; `greater_int` is
// its exact reverse, so passing one or the other flips the result — proving the
// comparator genuinely drives the order. The sort body below is the hand-kept
// prelude copy of `tests/fixtures/corelib/core/cmp.cnr`'s comparator family
// (see that file for the bounds justifications).
const PRELUDE: &str = r#"
struct AllocVtable { alloc: fn(ctx: rawptr u8, size: usize, align: usize) alloc -> rawptr u8, free: fn(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) alloc -> unit, realloc: fn(ctx: rawptr u8, ptr: rawptr u8, old_size: usize, new_size: usize, align: usize) alloc -> rawptr u8 }
copy struct Alloc { ctx: rawptr u8, vt: rawptr AllocVtable }
struct Bump { next: usize, end: usize, live: i64 }
fn with_window(base: usize, size: usize) -> Bump { return Bump { next: base, end: base + size, live: 0 }; }
fn bump_alloc(ctx: rawptr u8, size: usize, align: usize) -> rawptr u8 { unsafe "reserved window" { let b: Bump = ptr_read(cast_ptr[Bump](ctx)); let a: usize = (b.next + align - 1) / align * align; if a + size > b.end { return ptr_null[u8](); } ptr_write(cast_ptr[Bump](ctx), Bump { next: a + size, end: b.end, live: b.live + 1 }); return addr_to_ptr[u8](a); } }
fn bump_free(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) -> unit { unsafe "reserved window" { let b: Bump = ptr_read(cast_ptr[Bump](ctx)); ptr_write(cast_ptr[Bump](ctx), Bump { next: b.next, end: b.end, live: b.live - 1 }); } }
fn bump_realloc(ctx: rawptr u8, ptr: rawptr u8, old_size: usize, new_size: usize, align: usize) -> rawptr u8 {
    unsafe "bump cannot reclaim, so it cannot grow in place: carve a fresh block, copy old_size bytes into it, and release the old block through bump_free (a no-op for a real bump, so the old space is leaked as bump semantics require)" {
        let newp: rawptr u8 = bump_alloc(ctx, new_size, align);
        if is_null(newp) { return newp; }
        let a: usize = ptr_to_addr(ptr);
        let base: usize = ptr_to_addr(newp);
        let mut i: usize = 0usize;
        while i < old_size {
            let s: rawptr u8 = addr_to_ptr[u8](a + i);
            let d: rawptr u8 = addr_to_ptr[u8](base + i);
            let v: u8 = ptr_read(s);
            ptr_write(d, v);
            i = i + 1usize;
        }
        bump_free(ctx, ptr, old_size, align);
        return newp;
    }
}
static BUMP_VT: AllocVtable = AllocVtable { alloc: bump_alloc, free: bump_free, realloc: bump_realloc };
fn mk_alloc(state: write Bump) -> Alloc { unsafe "outlives every alloc" { return Alloc { ctx: cast_ptr[u8](addr_of_mut(state.*)), vt: addr_of(BUMP_VT) }; } }

fn insertion_by[T: copy](v: write Vec[T], less: fn(read T, read T) -> bool, lo: usize, hi: usize) alloc -> unit {
    if hi - lo < 2usize {
        return;
    }
    let mut k: usize = lo + 1usize;
    while k < hi {
        let x: T = get(read v.*, k).*;
        let mut j: usize = k;
        while j > lo {
            let w: T = get(read v.*, j - 1usize).*;
            if less(read x, read w) {
                set(write v.*, j, w);
                j = j - 1usize;
            } else {
                break;
            }
        }
        set(write v.*, j, x);
        k = k + 1usize;
    }
}
fn sift_down_by[T: copy](v: write Vec[T], less: fn(read T, read T) -> bool, base: usize, root: usize, end: usize) alloc -> unit {
    let mut i: usize = root;
    while i < end / 2usize {
        let left: usize = 2usize * i + 1usize;
        let mut child: usize = left;
        if left + 1usize < end {
            let l: T = get(read v.*, base + left).*;
            let r: T = get(read v.*, base + left + 1usize).*;
            if less(read l, read r) {
                child = left + 1usize;
            }
        }
        let cur: T = get(read v.*, base + i).*;
        let big: T = get(read v.*, base + child).*;
        if less(read cur, read big) {
            set(write v.*, base + i, big);
            set(write v.*, base + child, cur);
            i = child;
        } else {
            return;
        }
    }
}
fn heapsort_range_by[T: copy](v: write Vec[T], less: fn(read T, read T) -> bool, lo: usize, hi: usize) alloc -> unit {
    let m: usize = hi - lo;
    if m < 2usize {
        return;
    }
    let mut start: usize = m / 2usize;
    while start > 0usize {
        start = start - 1usize;
        sift_down_by(write v.*, less, lo, start, m);
    }
    let mut end: usize = m;
    while end > 1usize {
        end = end - 1usize;
        let top: T = get(read v.*, lo).*;
        let last: T = get(read v.*, lo + end).*;
        set(write v.*, lo, last);
        set(write v.*, lo + end, top);
        sift_down_by(write v.*, less, lo, 0usize, end);
    }
}
fn partition_by[T: copy](v: write Vec[T], less: fn(read T, read T) -> bool, lo: usize, hi: usize) alloc -> usize {
    let mid: usize = lo + (hi - lo) / 2usize;
    let a0: T = get(read v.*, lo).*;
    let a1: T = get(read v.*, mid).*;
    if less(read a1, read a0) {
        set(write v.*, lo, a1);
        set(write v.*, mid, a0);
    }
    let b1: T = get(read v.*, mid).*;
    let b2: T = get(read v.*, hi - 1usize).*;
    if less(read b2, read b1) {
        set(write v.*, mid, b2);
        set(write v.*, hi - 1usize, b1);
        let c0: T = get(read v.*, lo).*;
        let c1: T = get(read v.*, mid).*;
        if less(read c1, read c0) {
            set(write v.*, lo, c1);
            set(write v.*, mid, c0);
        }
    }
    let p: T = get(read v.*, mid).*;
    let park: T = get(read v.*, hi - 2usize).*;
    set(write v.*, mid, park);
    set(write v.*, hi - 2usize, p);
    let mut i: usize = lo;
    let mut j: usize = hi - 2usize;
    loop {
        i = i + 1usize;
        loop {
            if i >= hi - 2usize {
                break;
            }
            let xi: T = get(read v.*, i).*;
            if less(read xi, read p) {
                i = i + 1usize;
            } else {
                break;
            }
        }
        j = j - 1usize;
        loop {
            if j <= lo {
                break;
            }
            let xj: T = get(read v.*, j).*;
            if less(read p, read xj) {
                j = j - 1usize;
            } else {
                break;
            }
        }
        if i >= j {
            break;
        }
        let xi: T = get(read v.*, i).*;
        let xj: T = get(read v.*, j).*;
        set(write v.*, i, xj);
        set(write v.*, j, xi);
    }
    let piv: T = get(read v.*, hi - 2usize).*;
    let xi: T = get(read v.*, i).*;
    set(write v.*, hi - 2usize, xi);
    set(write v.*, i, piv);
    return i;
}
fn sort[T: copy](v: write Vec[T], less: fn(read T, read T) -> bool) alloc -> unit {
    let n: usize = len(read v.*);
    if n < 2usize {
        return;
    }
    let mut budget: usize = 0usize;
    let mut m: usize = n;
    while m > 1usize {
        m = m / 2usize;
        budget = budget + 1usize;
    }
    budget = 2usize * budget;
    let mut st_lo: [64]usize = [0usize; 64];
    let mut st_hi: [64]usize = [0usize; 64];
    let mut st_bud: [64]usize = [0usize; 64];
    st_lo[0usize] = 0usize;
    st_hi[0usize] = n;
    st_bud[0usize] = budget;
    let mut sp: usize = 1usize;
    while sp > 0usize {
        sp = sp - 1usize;
        let mut lo: usize = st_lo[sp];
        let mut hi: usize = st_hi[sp];
        let mut bud: usize = st_bud[sp];
        loop {
            if hi - lo <= 24usize {
                insertion_by(write v.*, less, lo, hi);
                break;
            }
            if bud == 0usize {
                heapsort_range_by(write v.*, less, lo, hi);
                break;
            }
            bud = bud - 1usize;
            let piv: usize = partition_by(write v.*, less, lo, hi);
            if piv - lo < hi - (piv + 1usize) {
                st_lo[sp] = piv + 1usize;
                st_hi[sp] = hi;
                st_bud[sp] = bud;
                sp = sp + 1usize;
                hi = piv;
            } else {
                st_lo[sp] = lo;
                st_hi[sp] = piv;
                st_bud[sp] = bud;
                sp = sp + 1usize;
                lo = piv + 1usize;
            }
        }
    }
}
fn less_int(a: read i64, b: read i64) -> bool { if a.* < b.* { return true; } return false; }
fn greater_int(a: read i64, b: read i64) -> bool { if a.* > b.* { return true; } return false; }
copy struct Item { k: i64, tag: i64 }
fn less_item(a: read Item, b: read Item) -> bool { if a.*.k < b.*.k { return true; } return false; }
fn greater_item(a: read Item, b: read Item) -> bool { if a.*.k > b.*.k { return true; } return false; }
"#;

/// Build, sort with `cmp`, then trace each element in order and return the length.
fn sort_program(elems: &[i64], cmp: &str) -> String {
    let pushes: String =
        elems.iter().map(|e| format!("push(write v, {e});")).collect::<Vec<_>>().join(" ");
    format!(
        "{PRELUDE}\n\
         fn run(al: Alloc) alloc -> i64 {{\n\
           let mut v: Vec[i64] = vec_new(read al);\n\
           {pushes}\n\
           sort(write v, {cmp});\n\
           let mut k: usize = 0usize;\n\
           while k < len(read v) {{ trace(get(read v, k).*); k = k + 1usize; }}\n\
           return conv i64 len(read v);\n\
         }}\n\
         fn main() alloc -> i64 {{\n\
           let mut bs: Bump = with_window(16777216, 1048576);\n\
           let al: Alloc = mk_alloc(write bs);\n\
           return run(al);\n\
         }}"
    )
}

/// Build a `Vec[Item]`, sort by field `k` with `cmp`, then trace each element's
/// `tag` in sorted order and return the length. Proves the generic `sort[T]`
/// lowers for a non-scalar `T`; the `tag`s (distinct from the keys) make the
/// sorted order observable.
fn sort_item_program(items: &[(i64, i64)], cmp: &str) -> String {
    let pushes: String = items
        .iter()
        .map(|(k, tag)| format!("push(write v, Item {{ k: {k}, tag: {tag} }});"))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "{PRELUDE}\n\
         fn run(al: Alloc) alloc -> i64 {{\n\
           let mut v: Vec[Item] = vec_new(read al);\n\
           {pushes}\n\
           sort(write v, {cmp});\n\
           let mut k: usize = 0usize;\n\
           while k < len(read v) {{ trace(get(read v, k).*.tag); k = k + 1usize; }}\n\
           return conv i64 len(read v);\n\
         }}\n\
         fn main() alloc -> i64 {{\n\
           let mut bs: Bump = with_window(16777216, 1048576);\n\
           let al: Alloc = mk_alloc(write bs);\n\
           return run(al);\n\
         }}"
    )
}

#[test]
fn sort_unsorted_ascending_all_engines() {
    let src = sort_program(&[5, 3, 8, 1, 9, 2, 7], "less_int");
    let (ret, trace) = all_engines(&src, "asc");
    assert_eq!(ret, 7);
    assert_eq!(trace, vec![1, 2, 3, 5, 7, 8, 9]);
}

#[test]
fn sort_same_vec_descending_all_engines() {
    // SAME input as the ascending case, only the comparator reversed: the result
    // reverses too, proving the order is comparator-driven, not hard-coded.
    let src = sort_program(&[5, 3, 8, 1, 9, 2, 7], "greater_int");
    let (ret, trace) = all_engines(&src, "desc");
    assert_eq!(ret, 7);
    assert_eq!(trace, vec![9, 8, 7, 5, 3, 2, 1]);
}

#[test]
fn sort_empty_all_engines() {
    let src = sort_program(&[], "less_int");
    let (ret, trace) = all_engines(&src, "empty");
    assert_eq!(ret, 0);
    assert_eq!(trace, Vec::<i64>::new());
}

#[test]
fn sort_single_all_engines() {
    let src = sort_program(&[42], "less_int");
    let (ret, trace) = all_engines(&src, "single");
    assert_eq!(ret, 1);
    assert_eq!(trace, vec![42]);
}

#[test]
fn sort_already_sorted_all_engines() {
    let src = sort_program(&[1, 2, 3, 4, 5], "less_int");
    let (ret, trace) = all_engines(&src, "sorted");
    assert_eq!(ret, 5);
    assert_eq!(trace, vec![1, 2, 3, 4, 5]);
}

#[test]
fn sort_reverse_sorted_all_engines() {
    let src = sort_program(&[5, 4, 3, 2, 1], "less_int");
    let (ret, trace) = all_engines(&src, "reverse");
    assert_eq!(ret, 5);
    assert_eq!(trace, vec![1, 2, 3, 4, 5]);
}

#[test]
fn sort_with_duplicates_all_engines() {
    let src = sort_program(&[3, 1, 2, 3, 1, 2, 3], "less_int");
    let (ret, trace) = all_engines(&src, "dups");
    assert_eq!(ret, 7);
    assert_eq!(trace, vec![1, 1, 2, 2, 3, 3, 3]);
}

/// Build an `n`-element `Vec[i64]` from a deterministic LCG evaluated INSIDE the
/// Candor program (no randomness at test time), sort with `cmp`, then trace the
/// count of adjacent pairs violating the requested order (`viol` is `<` for
/// ascending, `>` for descending; the count must be 0), the element sum, and a
/// position-weighted sum that pins the exact sequence. Expectations are
/// recomputed in Rust from the same LCG (`lcg_vals`).
fn sort_lcg_program(n: usize, seed: i64, cmp: &str, viol: &str) -> String {
    format!(
        "{PRELUDE}\n\
         fn run(al: Alloc) alloc -> i64 {{\n\
           let mut v: Vec[i64] = vec_new(read al);\n\
           let mut s: i64 = {seed};\n\
           let mut k: usize = 0usize;\n\
           while k < {n}usize {{\n\
             s = (s * 1103515245 + 12345) % 2147483648;\n\
             push(write v, s % 1000);\n\
             k = k + 1usize;\n\
           }}\n\
           sort(write v, {cmp});\n\
           let mut inv: i64 = 0;\n\
           let mut sum: i64 = 0;\n\
           let mut wsum: i64 = 0;\n\
           let mut j: usize = 0usize;\n\
           while j < len(read v) {{\n\
             let x: i64 = get(read v, j).*;\n\
             let w: i64 = conv i64 (j + 1usize);\n\
             sum = sum + x;\n\
             wsum = wsum + w * x;\n\
             if j > 0usize {{\n\
               let prev: i64 = get(read v, j - 1usize).*;\n\
               if x {viol} prev {{ inv = inv + 1; }}\n\
             }}\n\
             j = j + 1usize;\n\
           }}\n\
           trace(inv);\n\
           trace(sum);\n\
           trace(wsum);\n\
           return conv i64 len(read v);\n\
         }}\n\
         fn main() alloc -> i64 {{\n\
           let mut bs: Bump = with_window(16777216, 1048576);\n\
           let al: Alloc = mk_alloc(write bs);\n\
           return run(al);\n\
         }}"
    )
}

/// The Rust mirror of the in-program LCG stream (unsorted).
fn lcg_vals(n: usize, seed: i64) -> Vec<i64> {
    let mut s = seed;
    let mut vals: Vec<i64> = Vec::with_capacity(n);
    for _ in 0..n {
        s = (s * 1103515245 + 12345) % 2147483648;
        vals.push(s % 1000);
    }
    vals
}

fn sum_and_weighted(vals: &[i64]) -> (i64, i64) {
    let sum = vals.iter().sum();
    let wsum = vals.iter().enumerate().map(|(i, x)| (i as i64 + 1) * x).sum();
    (sum, wsum)
}

#[test]
fn sort_lcg_large_ascending_all_engines() {
    let (n, seed) = (500, 42);
    let src = sort_lcg_program(n, seed, "less_int", "<");
    let (ret, trace) = all_engines(&src, "lcg_asc");
    let mut vals = lcg_vals(n, seed);
    vals.sort();
    let (sum, wsum) = sum_and_weighted(&vals);
    assert_eq!(ret, n as i64);
    assert_eq!(trace, vec![0, sum, wsum]);
}

#[test]
fn sort_lcg_large_descending_all_engines() {
    // SAME LCG input, comparator reversed: the sequence (pinned by the weighted
    // sum) reverses too.
    let (n, seed) = (500, 42);
    let src = sort_lcg_program(n, seed, "greater_int", ">");
    let (ret, trace) = all_engines(&src, "lcg_desc");
    let mut vals = lcg_vals(n, seed);
    vals.sort();
    vals.reverse();
    let (sum, wsum) = sum_and_weighted(&vals);
    assert_eq!(ret, n as i64);
    assert_eq!(trace, vec![0, sum, wsum]);
}

// ---- adversarial patterns: organ-pipe, all-equal, sawtooth ------------------

/// Like `sort_lcg_program`, but the `n` elements come from an adversarial
/// generator: `fill` is a Candor statement pushing element `k` (of `n`). These
/// patterns smoke the median-of-three pivot selection and the equal-stopping
/// partition scans; the traces pin violations (0), the sum, and the
/// position-weighted sum against the Rust mirror.
fn sort_pattern_program(n: usize, fill: &str, cmp: &str, viol: &str) -> String {
    format!(
        "{PRELUDE}\n\
         fn run(al: Alloc) alloc -> i64 {{\n\
           let mut v: Vec[i64] = vec_new(read al);\n\
           let mut k: usize = 0usize;\n\
           while k < {n}usize {{\n\
             {fill}\n\
             k = k + 1usize;\n\
           }}\n\
           sort(write v, {cmp});\n\
           let mut inv: i64 = 0;\n\
           let mut sum: i64 = 0;\n\
           let mut wsum: i64 = 0;\n\
           let mut j: usize = 0usize;\n\
           while j < len(read v) {{\n\
             let x: i64 = get(read v, j).*;\n\
             let w: i64 = conv i64 (j + 1usize);\n\
             sum = sum + x;\n\
             wsum = wsum + w * x;\n\
             if j > 0usize {{\n\
               let prev: i64 = get(read v, j - 1usize).*;\n\
               if x {viol} prev {{ inv = inv + 1; }}\n\
             }}\n\
             j = j + 1usize;\n\
           }}\n\
           trace(inv);\n\
           trace(sum);\n\
           trace(wsum);\n\
           return conv i64 len(read v);\n\
         }}\n\
         fn main() alloc -> i64 {{\n\
           let mut bs: Bump = with_window(16777216, 1048576);\n\
           let al: Alloc = mk_alloc(write bs);\n\
           return run(al);\n\
         }}"
    )
}

fn organ_pipe_vals(n: usize) -> Vec<i64> {
    (0..n).map(|k| if k < n / 2 { k as i64 } else { (n - k) as i64 }).collect()
}

fn sawtooth_vals(n: usize) -> Vec<i64> {
    (0..n).map(|k| (k % 32) as i64).collect()
}

#[test]
fn sort_organ_pipe_ascending_all_engines() {
    let n = 500;
    let fill = format!(
        "if k < {n}usize / 2usize {{ push(write v, conv i64 k); }} else {{ push(write v, conv i64 ({n}usize - k)); }}"
    );
    let src = sort_pattern_program(n, &fill, "less_int", "<");
    let (ret, trace) = all_engines(&src, "organ_asc");
    let mut vals = organ_pipe_vals(n);
    vals.sort();
    let (sum, wsum) = sum_and_weighted(&vals);
    assert_eq!(ret, n as i64);
    assert_eq!(trace, vec![0, sum, wsum]);
}

#[test]
fn sort_organ_pipe_descending_all_engines() {
    // SAME organ-pipe input, comparator reversed: the pinned weighted sum
    // reverses too — comparator-driven order holds on adversarial input.
    let n = 500;
    let fill = format!(
        "if k < {n}usize / 2usize {{ push(write v, conv i64 k); }} else {{ push(write v, conv i64 ({n}usize - k)); }}"
    );
    let src = sort_pattern_program(n, &fill, "greater_int", ">");
    let (ret, trace) = all_engines(&src, "organ_desc");
    let mut vals = organ_pipe_vals(n);
    vals.sort();
    vals.reverse();
    let (sum, wsum) = sum_and_weighted(&vals);
    assert_eq!(ret, n as i64);
    assert_eq!(trace, vec![0, sum, wsum]);
}

#[test]
fn sort_all_equal_all_engines() {
    let n = 500;
    let src = sort_pattern_program(n, "push(write v, 7);", "less_int", "<");
    let (ret, trace) = all_engines(&src, "equal");
    let vals = vec![7i64; n];
    let (sum, wsum) = sum_and_weighted(&vals);
    assert_eq!(ret, n as i64);
    assert_eq!(trace, vec![0, sum, wsum]);
}

#[test]
fn sort_sawtooth_all_engines() {
    let n = 500;
    let src =
        sort_pattern_program(n, "push(write v, conv i64 (k % 32usize));", "less_int", "<");
    let (ret, trace) = all_engines(&src, "saw");
    let mut vals = sawtooth_vals(n);
    vals.sort();
    let (sum, wsum) = sum_and_weighted(&vals);
    assert_eq!(ret, n as i64);
    assert_eq!(trace, vec![0, sum, wsum]);
}

#[test]
fn sort_items_by_field_ascending_all_engines() {
    // Keys 3,1,2 with distinct tags 100,200,300: sorting ascending by `k`
    // reorders the tags to 200,300,100 — proving the generic `sort` moves whole
    // `Item` values, not just keys, for a non-scalar `T`.
    let src = sort_item_program(&[(3, 100), (1, 200), (2, 300)], "less_item");
    let (ret, trace) = all_engines(&src, "item_asc");
    assert_eq!(ret, 3);
    assert_eq!(trace, vec![200, 300, 100]);
}

#[test]
fn sort_items_by_field_descending_all_engines() {
    // SAME input, comparator reversed: the tag order reverses too, proving the
    // order is comparator-driven for the struct instantiation as well.
    let src = sort_item_program(&[(3, 100), (1, 200), (2, 300)], "greater_item");
    let (ret, trace) = all_engines(&src, "item_desc");
    assert_eq!(ret, 3);
    assert_eq!(trace, vec![100, 300, 200]);
}
