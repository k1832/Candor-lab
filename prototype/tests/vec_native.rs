//! Native `Vec[T]` gate (native collections slice #3): the `New` / `VecPush` /
//! `VecGet` / `VecSet` / `VecPop` collection intrinsics lowered inline in BOTH
//! native backends, mirroring `mir::interp::collection_op` byte-for-byte. A
//! `vec_new` over a reclaiming free-list allocator, `push`es that force
//! `vec_reserve` growth across the initial capacity, plus `get`/`set`/`len`/`pop`
//! over a scalar element AND a stride-16 struct, must reproduce the oracle's
//! observable trace `theta` and returned length identically under the MIR
//! interpreter and the Cranelift native engine (no-opt + opt). The LLVM `clang -O2`
//! engine covers this fixture transitively through `tests/llvm.rs`'s auto-scanned
//! fifth-engine corpus gate. The out-of-bounds `get`/`set` faults must deliver the
//! identical `Bounds` kind AND span across every engine.

use candor_proto::interp::{Fault, FaultKind, Run};
use candor_proto::{
    run_source_real, run_source_real_mir, run_source_real_native, run_source_real_native_opt,
    MirRunResult, RunResult,
};

fn fixture() -> String {
    let path = format!("{}/tests/fixtures/run/vec_native.cnr", env!("CARGO_MANIFEST_DIR"));
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
fn vec_native_push_get_set_len_pop_all_engines() {
    let src = fixture();
    let o = oracle(&src);
    // Vec[i64]: len 6, get[0]=10, get[5]=60, set[2]=99, pop -> 60, len 5.
    // Vec[Pair]: len 5, get[0].a=1, get[4].b=10, set[1]={30,40}, get[1].a=30, .b=40.
    assert_eq!(o.trace, vec![6, 10, 60, 99, 60, 5, 5, 1, 10, 30, 40], "oracle trace");
    assert_eq!(o.ret, 5, "oracle ret");

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

/// A bump allocator preamble for the standalone out-of-bounds fault programs.
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

fn oob_get_src() -> String {
    format!("{ALLOC}\nfn main() alloc -> i64 {{\n  let mut bs: Bump = with_window(16777216, 1048576);\n  let al: Alloc = mk_alloc(write bs);\n  let mut v: Vec[i64] = vec_new(read al);\n  push(write v, 10);\n  return get(read v, 5).*;\n}}")
}

fn oob_set_src() -> String {
    format!("{ALLOC}\nfn main() alloc -> i64 {{\n  let mut bs: Bump = with_window(16777216, 1048576);\n  let al: Alloc = mk_alloc(write bs);\n  let mut v: Vec[i64] = vec_new(read al);\n  push(write v, 10);\n  set(write v, 5, 99);\n  return 0;\n}}")
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

/// An out-of-bounds `get`/`set` (index 5 on a 1-element Vec) must fault `Bounds`
/// at the index arg's span — byte-identical kind AND span across the oracle, the
/// MIR interpreter, and the Cranelift native backend (no-opt + opt).
#[test]
fn vec_get_set_out_of_bounds_faults_bounds_all_engines() {
    for src in [oob_get_src(), oob_set_src()] {
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
}


/// Drop-on-overwrite: `set` over an existing element must run that element's drop
/// glue BEFORE moving the new value in, exactly as `mir::interp`'s
/// `drop_value(slot, elem)`. The `Vec` is fully drained (both survivors popped and
/// consumed) so the scope-end collection drop — deferred to the collection-drop
/// slice — is a no-op in every engine, isolating the overwrite drop. The traced
/// hook order must be byte-identical across the oracle, the MIR interpreter, and
/// the Cranelift native backend (no-opt + opt).
fn drop_fixture() -> String {
    let path =
        format!("{}/tests/fixtures/run/vec_drop_overwrite.cnr", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

#[test]
fn vec_set_drops_overwritten_element_all_engines() {
    let src = drop_fixture();
    let o = oracle(&src);
    // set drops old E{1} (trace 1); pop E{2} -> match traces 100, then E{2} drops
    // (trace 2); pop E{9} -> match traces 200, then E{9} drops (trace 9).
    assert_eq!(o.trace, vec![1, 100, 2, 200, 9], "oracle drop-on-overwrite order");

    for (label, r) in [
        ("mir", run_source_real_mir(&src)),
        ("native-noopt", run_source_real_native(&src)),
        ("native-opt", run_source_real_native_opt(&src)),
    ] {
        let run = mir_run(r, label);
        assert_eq!(run.trace, o.trace, "{label} drop-on-overwrite trace diverged");
    }
}
