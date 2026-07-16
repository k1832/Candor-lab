//! Native `String` gate (design 0013, Path A): the `New` / `StringAppend` /
//! `StringAsStr` collection intrinsics lowered inline in BOTH native backends,
//! mirroring `mir::interp::collection_op` byte-for-byte. `string_new` over a
//! reclaiming free-list allocator + `append`s that force `string_reserve` growth
//! across the initial capacity must reproduce the oracle's observable trace `θ`
//! and byte length identically under the MIR interpreter and the Cranelift
//! native engine (no-opt + opt). The LLVM `clang -O2` engine covers this fixture
//! transitively through `tests/llvm.rs`'s full-corpus fifth-engine gate.

use candor::interp::{Fault, FaultKind, Run};
use candor::{
    run_source_real, run_source_real_mir, run_source_real_native, run_source_real_native_opt,
    MirRunResult, RunResult,
};

fn fixture() -> String {
    let path = format!("{}/tests/fixtures/run/string_native.cnr", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn oracle(src: &str) -> Run {
    match run_source_real(src) {
        RunResult::Ok(r) => r,
        RunResult::Fault(f) => panic!("oracle faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => {
            panic!("oracle check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("oracle parse error: {}", d.to_json()),
    }
}

fn mir_run(r: MirRunResult, label: &str) -> Run {
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

#[test]
fn string_native_new_append_as_str_all_engines() {
    let src = fixture();
    let o = oracle(&src);
    // Empty (0), single append "hi" (2, 'h'=104, 'i'=105), then the grown
    // 20-byte "abcde...pqrst" (20, 'a'=97, 'k'=107, 't'=116). Then the `push`
    // section: 10 bytes total — 'A' (65), 'é' (195 169), '€' (226 130 172),
    // '😀' (240 159 152 128) — the 1/2/3/4-byte UTF-8 encodings.
    assert_eq!(
        o.trace,
        vec![
            0, 2, 104, 105, 20, 97, 107, 116, 10, 65, 195, 169, 226, 130, 172, 240, 159, 152, 128
        ],
        "oracle trace"
    );
    assert_eq!(o.ret, 20, "oracle ret");

    for (label, r) in [
        ("mir", run_source_real_mir(&src)),
        ("native-noopt", run_source_real_native(&src)),
        ("native-opt", run_source_real_native_opt(&src)),
    ] {
        let run = mir_run(r, label);
        assert_eq!(run.trace, o.trace, "{label} trace diverged from oracle");
        assert_eq!(run.ret, o.ret, "{label} ret diverged from oracle");
    }
}

/// A COUNTING bump allocator preamble for the standalone `push`-fault programs.
const ALLOC: &str = r#"
struct AllocVtable { alloc: fn(ctx: rawptr u8, size: usize, align: usize) alloc -> rawptr u8, free: fn(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) alloc -> unit, realloc: fn(ctx: rawptr u8, ptr: rawptr u8, old_size: usize, new_size: usize, align: usize) alloc -> rawptr u8 }
copy struct Alloc { ctx: rawptr u8, vt: rawptr AllocVtable }
struct Bump { next: usize, end: usize, live: i64 }
fn with_window(base: usize, size: usize) -> Bump { return Bump { next: base, end: base + size, live: 0 }; }
fn bump_alloc(ctx: rawptr u8, size: usize, align: usize) -> rawptr u8 { unsafe "reserved window" { let b: Bump = ptr_read(cast_ptr[Bump](ctx)); let a: usize = (b.next + align - 1) / align * align; if a + size > b.end { return ptr_null[u8](); } ptr_write(cast_ptr[Bump](ctx), Bump { next: a + size, end: b.end, live: b.live + 1 }); return addr_to_ptr[u8](a); } }
fn bump_free(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) -> unit { unsafe "reserved window" { let b: Bump = ptr_read(cast_ptr[Bump](ctx)); ptr_write(cast_ptr[Bump](ctx), Bump { next: b.next, end: b.end, live: b.live - 1 }); } }
fn bump_realloc(ctx: rawptr u8, ptr: rawptr u8, old_size: usize, new_size: usize, align: usize) -> rawptr u8 {
    unsafe "bump cannot reclaim, so it cannot grow in place: carve a fresh block, copy old_size bytes into it, and release the old block through bump_free (a no-op for a real bump, so the old space is leaked as bump semantics require)" {
        let newp: rawptr u8 = bump_alloc(ctx, new_size, align);
        if is_null(newp) {
            return newp;
        }
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
"#;

fn push_fault_src(bad: u32) -> String {
    format!(
        "{ALLOC}\nfn main() alloc -> i64 {{\n  let mut bs: Bump = with_window(16777216, 1048576);\n  let al: Alloc = mk_alloc(write bs);\n  let mut s: String = string_new(read al);\n  push(write s, {bad});\n  return 0;\n}}"
    )
}

fn oracle_fault(src: &str) -> Fault {
    match run_source_real(src) {
        RunResult::Fault(f) => f,
        RunResult::Ok(r) => panic!("oracle: expected fault, got ret {}", r.ret),
        RunResult::CheckErrors(d) => {
            panic!("oracle check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("oracle parse error: {}", d.to_json()),
    }
}

fn engine_fault(r: MirRunResult, label: &str) -> Fault {
    match r {
        MirRunResult::Fault(f) => f,
        MirRunResult::Ok(run) => panic!("{label}: expected fault, got ret {}", run.ret),
        MirRunResult::Unsupported(m) => panic!("{label} unsupported: {m}"),
        MirRunResult::CheckErrors(d) => {
            panic!("{label} check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        MirRunResult::ParseError(d) => panic!("{label} parse error: {}", d.to_json()),
    }
}

/// Pushing a surrogate (0xD800) or an out-of-range scalar (0x110000) must fault
/// `Requires` at the call span — byte-identical kind AND span across the oracle,
/// the MIR interpreter, and the Cranelift native backend (no-opt + opt), matching
/// `utf8_encode_scalar`'s scalar-value backstop.
#[test]
fn string_push_non_scalar_faults_requires_all_engines() {
    for bad in [0xD800u32, 0x110000u32] {
        let src = push_fault_src(bad);
        let o = oracle_fault(&src);
        assert_eq!(o.kind, FaultKind::Requires, "oracle fault kind for {bad:#x}");

        for (label, r) in [
            ("mir", run_source_real_mir(&src)),
            ("native-noopt", run_source_real_native(&src)),
            ("native-opt", run_source_real_native_opt(&src)),
        ] {
            let f = engine_fault(r, label);
            assert_eq!(f.kind, o.kind, "{label} fault kind diverged for {bad:#x}");
            assert_eq!(f.span, o.span, "{label} fault span diverged for {bad:#x}");
        }
    }
}
