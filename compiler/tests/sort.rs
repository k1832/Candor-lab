//! In-place generic `sort[T: copy]` over `Vec[T]` by a first-class comparator. Insertion
//! sort (simple, obviously correct, easy to verify byte-exact; an O(n log n)
//! variant is a follow-up). The comparator is an ordinary
//! `fn(read i64, read i64) -> bool` value, so the
//! SAME Vec sorts ascending or descending purely by which comparator is passed —
//! the ascending/descending pair below proves the order is comparator-driven and
//! not hard-coded. Every case is checked byte-exact across all five engines
//! (tree-walk oracle, MIR interp, Cranelift no-opt, Cranelift opt, LLVM -O2) via
//! the same trace-channel harness as `tests/iteration.rs`.
//!
//! `sort` is generic over the element type `T` (bounded `copy`, since insertion
//! sort shuffles elements by value). It is exercised at two instantiations: the
//! integer cases below (`Vec[i64]`, `T` inferred at the call) prove byte-exact
//! agreement with the earlier monomorphic form, and the `Item`-struct cases
//! (`Vec[Item]`, sorted by a field) prove the generic form lowers correctly for
//! a non-scalar `T` — both byte-exact across all five engines.

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

// A counting bump allocator (mirrors tests/vec.rs) plus the generic insertion
// `sort` and two i64 comparators. `less_int` orders ascending; `greater_int` is
// its exact reverse, so passing one or the other flips the result — proving the
// comparator genuinely drives the order.
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

fn sort[T: copy](v: write Vec[T], less: fn(read T, read T) -> bool) alloc -> unit {
    let n: usize = len(read v.*);
    let mut i: usize = 1usize;
    while i < n {
        let key: T = get(read v.*, i).*;
        let mut j: usize = i;
        while j > 0usize {
            let prev: T = get(read v.*, j - 1usize).*;
            if (less)(read key, read prev) {
                set(write v.*, j, prev);
                j = j - 1usize;
            } else {
                break;
            }
        }
        set(write v.*, j, key);
        i = i + 1usize;
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
