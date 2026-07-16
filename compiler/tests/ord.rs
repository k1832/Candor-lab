//! `Ord` — a comparison interface impl'd for builtin scalar types, and the
//! generic comparison utilities (`min`/`max`/`sort_ord`) built on it. This is the
//! payoff of allowing interface impls for scalars (design 0007 §2.3 + the scalar
//! orphan rule): `Ord` is impl'd for `i64`/`u8`/`usize`/`u32` in the interface's
//! own module, and the generic consumers dispatch `cmp` on the concrete scalar.
//!
//! Every runtime case is byte-exact across all five engines (tree-walk oracle, MIR
//! interp, Cranelift no-opt, Cranelift opt, LLVM -O2) via the same trace-channel
//! harness as `tests/sort.rs`, at two scalar instantiations (`i64` and `u32`) so
//! the `Ord` bound is proven to thread through monomorphization. The coherence
//! gate — overlap (E1009) and scalar orphan (E1013) rejection, blessed acceptance —
//! is asserted alongside.

use candor::diag::Severity;
use candor::{
    check_dir, check_source_real, compile_path_llvm, run_dir, run_source_real,
    run_source_real_mir, run_source_real_native, run_source_real_native_opt, MirRunResult,
    RunResult,
};
use std::path::{Path, PathBuf};
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
    let srcp = dir.join(format!("candor-ord-{}-{}.cnr", std::process::id(), tag));
    let outp = dir.join(format!("candor-ord-{}-{}", std::process::id(), tag));
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

enum Ordering { Less, Equal, Greater }
enum Opt[T] { Some(T), None }
interface Ord {
    fn cmp(read self, other: Self) -> Ordering;
}
impl Ord for i64 {
    fn cmp(read self, other: Self) -> Ordering {
        if self.* < other { return Ordering::Less; }
        if self.* > other { return Ordering::Greater; }
        return Ordering::Equal;
    }
}
impl Ord for u8 {
    fn cmp(read self, other: Self) -> Ordering {
        if self.* < other { return Ordering::Less; }
        if self.* > other { return Ordering::Greater; }
        return Ordering::Equal;
    }
}
impl Ord for usize {
    fn cmp(read self, other: Self) -> Ordering {
        if self.* < other { return Ordering::Less; }
        if self.* > other { return Ordering::Greater; }
        return Ordering::Equal;
    }
}
impl Ord for u32 {
    fn cmp(read self, other: Self) -> Ordering {
        if self.* < other { return Ordering::Less; }
        if self.* > other { return Ordering::Greater; }
        return Ordering::Equal;
    }
}
fn min[T: Ord + copy](v: read Vec[T]) -> Opt[T] {
    let n: usize = len(read v.*);
    if n == 0usize { return Opt::None; }
    let mut best: T = get(read v.*, 0usize).*;
    let mut i: usize = 1usize;
    while i < n {
        let cur: T = get(read v.*, i).*;
        match cur.cmp(best) {
            Ordering::Less => { best = cur; },
            Ordering::Equal => {},
            Ordering::Greater => {},
        }
        i = i + 1usize;
    }
    return Opt::Some(best);
}
fn max[T: Ord + copy](v: read Vec[T]) -> Opt[T] {
    let n: usize = len(read v.*);
    if n == 0usize { return Opt::None; }
    let mut best: T = get(read v.*, 0usize).*;
    let mut i: usize = 1usize;
    while i < n {
        let cur: T = get(read v.*, i).*;
        match cur.cmp(best) {
            Ordering::Greater => { best = cur; },
            Ordering::Less => {},
            Ordering::Equal => {},
        }
        i = i + 1usize;
    }
    return Opt::Some(best);
}
fn sort_ord[T: Ord + copy](v: write Vec[T]) alloc -> unit {
    let n: usize = len(read v.*);
    let mut i: usize = 1usize;
    while i < n {
        let key: T = get(read v.*, i).*;
        let mut j: usize = i;
        while j > 0usize {
            let prev: T = get(read v.*, j - 1usize).*;
            match key.cmp(prev) {
                Ordering::Less => {
                    set(write v.*, j, prev);
                    j = j - 1usize;
                },
                Ordering::Equal => { break; },
                Ordering::Greater => { break; },
            }
        }
        set(write v.*, j, key);
        i = i + 1usize;
    }
}
"#;

/// Build a `Vec[i64]`, sort it with `sort_ord`, trace each element in order, then
/// trace `min` and `max` (via `-1000` sentinels for the empty `Opt::None`). Returns
/// the length. Exercises the `Ord` impl for `i64` end to end.
fn i64_program(elems: &[i64]) -> String {
    let pushes: String =
        elems.iter().map(|e| format!("push(write v, {e});")).collect::<Vec<_>>().join(" ");
    format!(
        "{PRELUDE}\n\
         fn run(al: Alloc) alloc -> i64 {{\n\
           let mut v: Vec[i64] = vec_new(read al);\n\
           {pushes}\n\
           sort_ord(write v);\n\
           let mut k: usize = 0usize;\n\
           while k < len(read v) {{ trace(get(read v, k).*); k = k + 1usize; }}\n\
           match min(read v) {{ Opt::Some(m) => {{ trace(m); }}, Opt::None => {{ trace(-1000); }} }}\n\
           match max(read v) {{ Opt::Some(m) => {{ trace(m); }}, Opt::None => {{ trace(-1000); }} }}\n\
           return conv i64 len(read v);\n\
         }}\n\
         fn main() alloc -> i64 {{\n\
           let mut bs: Bump = with_window(16777216, 1048576);\n\
           let al: Alloc = mk_alloc(write bs);\n\
           return run(al);\n\
         }}"
    )
}

/// The same, at the `u32` instantiation: elements are pushed as `u32`, traced by
/// converting back to `i64`. Proves the `Ord` bound threads through a second scalar
/// type — the `min`/`max`/`sort_ord` monomorphize to the `u32` `Ord` impl.
fn u32_program(elems: &[u32]) -> String {
    let pushes: String =
        elems.iter().map(|e| format!("push(write v, {e}u32);")).collect::<Vec<_>>().join(" ");
    format!(
        "{PRELUDE}\n\
         fn run(al: Alloc) alloc -> i64 {{\n\
           let mut v: Vec[u32] = vec_new(read al);\n\
           {pushes}\n\
           sort_ord(write v);\n\
           let mut k: usize = 0usize;\n\
           while k < len(read v) {{ trace(conv i64 get(read v, k).*); k = k + 1usize; }}\n\
           match min(read v) {{ Opt::Some(m) => {{ trace(conv i64 m); }}, Opt::None => {{ trace(-1000); }} }}\n\
           match max(read v) {{ Opt::Some(m) => {{ trace(conv i64 m); }}, Opt::None => {{ trace(-1000); }} }}\n\
           return conv i64 len(read v);\n\
         }}\n\
         fn main() alloc -> i64 {{\n\
           let mut bs: Bump = with_window(16777216, 1048576);\n\
           let al: Alloc = mk_alloc(write bs);\n\
           return run(al);\n\
         }}"
    )
}

// ---- runtime: byte-exact across all five engines, at i64 --------------------

#[test]
fn i64_unsorted_with_negatives_all_engines() {
    let src = i64_program(&[5, -3, 8, -1, 9, 0, -7]);
    let (ret, trace) = all_engines(&src, "i64_neg");
    assert_eq!(ret, 7);
    // sorted, then min, then max.
    assert_eq!(trace, vec![-7, -3, -1, 0, 5, 8, 9, -7, 9]);
}

#[test]
fn i64_with_duplicates_all_engines() {
    let src = i64_program(&[3, 1, 2, 3, 1, 2, 3]);
    let (ret, trace) = all_engines(&src, "i64_dups");
    assert_eq!(ret, 7);
    assert_eq!(trace, vec![1, 1, 2, 2, 3, 3, 3, 1, 3]);
}

#[test]
fn i64_reverse_sorted_all_engines() {
    let src = i64_program(&[5, 4, 3, 2, 1]);
    let (ret, trace) = all_engines(&src, "i64_rev");
    assert_eq!(ret, 5);
    assert_eq!(trace, vec![1, 2, 3, 4, 5, 1, 5]);
}

#[test]
fn i64_single_all_engines() {
    let src = i64_program(&[42]);
    let (ret, trace) = all_engines(&src, "i64_single");
    assert_eq!(ret, 1);
    assert_eq!(trace, vec![42, 42, 42]);
}

#[test]
fn i64_empty_all_engines() {
    // Empty: nothing sorted/traced; min and max are `Opt::None` -> the -1000 sentinel.
    let src = i64_program(&[]);
    let (ret, trace) = all_engines(&src, "i64_empty");
    assert_eq!(ret, 0);
    assert_eq!(trace, vec![-1000, -1000]);
}

// ---- runtime: the second scalar instantiation (u32) -------------------------

#[test]
fn u32_unsorted_all_engines() {
    let src = u32_program(&[5, 3, 8, 1, 9, 2, 7]);
    let (ret, trace) = all_engines(&src, "u32");
    assert_eq!(ret, 7);
    assert_eq!(trace, vec![1, 2, 3, 5, 7, 8, 9, 1, 9]);
}

#[test]
fn u32_duplicates_all_engines() {
    let src = u32_program(&[4, 4, 2, 2, 4]);
    let (ret, trace) = all_engines(&src, "u32_dups");
    assert_eq!(ret, 5);
    assert_eq!(trace, vec![2, 2, 4, 4, 4, 2, 4]);
}

// ---- coherence gate: overlap (E1009) ----------------------------------------

#[test]
fn overlap_two_scalar_impls_rejected() {
    let src = format!(
        "{PRELUDE}\n\
         impl Ord for i64 {{ fn cmp(read self, other: Self) -> Ordering {{ return Ordering::Less; }} }}\n\
         fn main() -> i64 {{ return 0; }}"
    );
    assert!(errors(&src).contains(&"E1009".to_string()), "expected E1009, got {:?}", errors(&src));
}

// ---- coherence gate: blessed (single-file root owns everything) -------------

#[test]
fn blessed_single_file_checks_clean() {
    let src = format!("{PRELUDE}\nfn main() -> i64 {{ return 0; }}");
    assert_clean(&src);
}

// ---- coherence gate: module-tree orphan (E1013) + blessed run ---------------

fn moddir(name: &str) -> PathBuf {
    PathBuf::from(format!("{}/tests/fixtures/modules/{name}", env!("CARGO_MANIFEST_DIR")))
}

fn mod_codes(name: &str) -> Vec<String> {
    match check_dir(&moddir(name)) {
        Ok(d) => d.into_iter().filter(|x| x.severity == Severity::Error).map(|x| x.code).collect(),
        Err(d) => vec![d.code],
    }
}

#[test]
fn scalar_orphan_across_modules_rejected() {
    // `impl Ord for i64` in a module owning neither `Ord` nor `i64` -> orphan.
    assert!(
        mod_codes("bad_orphan_scalar").contains(&"E1013".to_string()),
        "expected E1013, got {:?}",
        mod_codes("bad_orphan_scalar")
    );
}

#[test]
fn scalar_blessed_in_interface_module_runs() {
    // `impl Ord for i64` in `Ord`'s own module is admitted; the generic consumer
    // dispatches on the concrete `i64`.
    assert!(mod_codes("ok_scalar_impl").is_empty(), "should check clean, got {:?}", mod_codes("ok_scalar_impl"));
    match run_dir(&moddir("ok_scalar_impl")) {
        RunResult::Ok(r) => assert_eq!(r.ret, 12),
        _ => panic!("ok_scalar_impl did not run"),
    }
}
