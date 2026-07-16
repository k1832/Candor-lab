//! Native hash `Map[V]` gate (native collections slice #4): the `MapInsert` /
//! `MapContains` / `MapGet` collection intrinsics lowered inline in BOTH native
//! backends, mirroring `mir::interp::collection_op` byte-for-byte (FNV-1a hash +
//! open-addressed linear probing + 3/4-load-factor rehash). A `map_new` over a
//! reclaiming free-list allocator, `insert`s that COLLIDE in the probe chain and
//! cross the load factor to force a growth REHASH, an OVERWRITE of an existing
//! key, plus `get`/`contains`/`len`, must reproduce the oracle's observable trace
//! `theta` and returned length identically under the MIR interpreter and the
//! Cranelift native engine (no-opt + opt). The LLVM `clang -O2` engine covers the
//! same fixture transitively through `tests/llvm.rs`'s auto-scanned fifth-engine
//! corpus gate. The missing-key `get` fault must deliver the identical `Bounds`
//! kind AND span across every engine.

use candor_proto::interp::{Fault, FaultKind, Run};
use candor_proto::{
    run_source_real, run_source_real_mir, run_source_real_native, run_source_real_native_opt,
    MirRunResult, RunResult,
};

fn fixture() -> String {
    let path = format!("{}/tests/fixtures/run/map_native.cnr", env!("CARGO_MANIFEST_DIR"));
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
fn map_native_insert_get_contains_len_collisions_rehash_all_engines() {
    let src = fixture();
    let o = oracle(&src);
    // len 8; overwrite keeps len 8; get a=1, i=99 (overwritten), y=3, h=4, x=5,
    // c=6, let=7, e=8; contains y -> 1, contains zzz -> 0.
    assert_eq!(o.trace, vec![8, 8, 1, 99, 3, 4, 5, 6, 7, 8, 1, 0], "oracle trace");
    assert_eq!(o.ret, 8, "oracle ret");

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

/// A bump allocator preamble for the standalone missing-key fault program.
const ALLOC: &str = r#"
struct AllocVtable { alloc: fn(ctx: rawptr u8, size: usize, align: usize) alloc -> rawptr u8, free: fn(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) alloc -> unit }
copy struct Alloc { ctx: rawptr u8, vt: rawptr AllocVtable }
struct Bump { next: usize, end: usize, live: i64 }
fn with_window(base: usize, size: usize) -> Bump { return Bump { next: base, end: base + size, live: 0 }; }
fn bump_alloc(ctx: rawptr u8, size: usize, align: usize) -> rawptr u8 { unsafe "reserved window" { let b: Bump = ptr_read(cast_ptr[Bump](ctx)); let a: usize = (b.next + align - 1) / align * align; if a + size > b.end { return ptr_null[u8](); } ptr_write(cast_ptr[Bump](ctx), Bump { next: a + size, end: b.end, live: b.live + 1 }); return addr_to_ptr[u8](a); } }
fn bump_free(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) -> unit { unsafe "reserved window" { let b: Bump = ptr_read(cast_ptr[Bump](ctx)); ptr_write(cast_ptr[Bump](ctx), Bump { next: b.next, end: b.end, live: b.live - 1 }); } }
static BUMP_VT: AllocVtable = AllocVtable { alloc: bump_alloc, free: bump_free };
fn mk_alloc(state: write Bump) -> Alloc { unsafe "outlives every alloc" { return Alloc { ctx: cast_ptr[u8](addr_of_mut(state.*)), vt: addr_of(BUMP_VT) }; } }
"#;

fn missing_get_src() -> String {
    format!("{ALLOC}\nfn main() alloc -> i64 {{\n  let mut bs: Bump = with_window(16777216, 1048576);\n  let al: Alloc = mk_alloc(write bs);\n  let mut m: Map[i64] = map_new(read al);\n  insert(write m, \"a\", 1);\n  return get(read m, \"b\").*;\n}}")
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

/// A `get` on an absent key must fault `Bounds` at the key arg's span —
/// byte-identical kind AND span across the oracle, the MIR interpreter, and the
/// Cranelift native backend (no-opt + opt).
#[test]
fn map_get_missing_faults_bounds_all_engines() {
    let src = missing_get_src();
    let o = oracle_fault(&src);
    assert_eq!(o.kind, FaultKind::Bounds, "oracle fault kind");

    for (label, r) in [
        ("mir", run_source_real_mir(&src)),
        ("native-noopt", run_source_real_native(&src)),
        ("native-opt", run_source_real_native_opt(&src)),
    ] {
        let f = engine_fault(r, label);
        assert_eq!(f.kind, o.kind, "{label} fault kind diverged");
        assert_eq!(f.span, o.span, "{label} fault span diverged");
    }
}
