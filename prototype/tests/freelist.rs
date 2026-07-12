//! The reclaiming free-list allocator gate (design 0001 §6.1/§11.1): a fixed-
//! window first-fit allocator whose `free` reclaims and whose `alloc` reuses.
//!
//! REUSE is proven by a window sized for exactly one block: a pure bump
//! allocator serves one box then OOMs, so five sequential boxes succeeding
//! (RET 100) can only happen if `free` reclaimed and `alloc` re-handed the
//! block. OOM is proven by a zero-headroom window taking the `BoxResult::oom`
//! arm (RET 42). Both run through the box/drop path on every engine: the tree-
//! walking oracle, the precise MIR interpreter, and the Cranelift native
//! backends (no-opt + opt). The LLVM engine covers them transitively through
//! `tests/llvm.rs`'s full-corpus fifth-engine gate (the fixtures live in
//! `tests/fixtures/run/`, which that gate scans).

use candor_proto::{
    run_source_real, run_source_real_mir, run_source_real_native, run_source_real_native_opt,
    MirRunResult, RunResult,
};

fn oracle_ret(src: &str) -> i64 {
    match run_source_real(src) {
        RunResult::Ok(r) => r.ret,
        other => panic!("oracle did not run: {}", describe(other)),
    }
}

fn mir_ret(r: MirRunResult, label: &str) -> i64 {
    match r {
        MirRunResult::Ok(run) => run.ret,
        MirRunResult::Fault(f) => panic!("{label} faulted: {}", f.to_json()),
        MirRunResult::Unsupported(m) => panic!("{label} unsupported: {m}"),
        MirRunResult::CheckErrors(d) => {
            panic!("{label} check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        MirRunResult::ParseError(d) => panic!("{label} parse error: {}", d.to_json()),
    }
}

fn describe(r: RunResult) -> String {
    match r {
        RunResult::Ok(run) => format!("ret {}", run.ret),
        RunResult::Fault(f) => format!("fault {}", f.to_json()),
        RunResult::CheckErrors(d) => {
            format!("check errors {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => format!("parse error {}", d.to_json()),
    }
}

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/run/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

/// Assert every non-LLVM engine agrees on `expected` for `src`.
fn assert_all_engines(src: &str, expected: i64) {
    assert_eq!(oracle_ret(src), expected, "tree-walking oracle");
    assert_eq!(mir_ret(run_source_real_mir(src), "MIR"), expected, "MIR interpreter");
    assert_eq!(mir_ret(run_source_real_native(src), "native-noopt"), expected, "native (no-opt)");
    assert_eq!(mir_ret(run_source_real_native_opt(src), "native-opt"), expected, "native (opt)");
}

// A freed block is reclaimed and reused: five boxes through a one-block window.
#[test]
fn freelist_box_reuse_all_engines() {
    assert_all_engines(&fixture("freelist_reuse.cnr"), 100);
}

// Exhausted window: `box` takes the `BoxResult::oom` arm on every engine.
#[test]
fn freelist_box_oom_all_engines() {
    assert_all_engines(&fixture("freelist_oom.cnr"), 42);
}

// A focused unit driving `alloc`/`free` directly (no box): alloc two blocks,
// free the first, alloc again — the third alloc must reuse the freed block's
// address, and the still-live second block must be untouched.
const DIRECT_DRIVE: &str = r#"
struct AllocVtable {
    alloc: fn(ctx: rawptr u8, size: usize, align: usize) alloc -> rawptr u8,
    free: fn(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) alloc -> unit,
}
struct FreeList { next: usize, end: usize, head: rawptr u8 }
struct FreeBlock { next: rawptr u8, size: usize }

fn block_span(size: usize, align: usize) -> usize {
    let mut need: usize = size;
    if need < 16usize { need = 16usize; }
    return (need + align - 1usize) / align * align;
}
fn with_window(base: usize, size: usize) -> FreeList {
    unsafe "empty list, null head" {
        return FreeList { next: base, end: base + size, head: ptr_null[u8]() };
    }
}
fn freelist_alloc(ctx: rawptr u8, size: usize, align: usize) -> rawptr u8 {
    unsafe "ctx owns the window and free chain; carved blocks stay in [next,end) and are >= header-sized" {
        let st: FreeList = ptr_read(cast_ptr[FreeList](ctx));
        let need: usize = block_span(size, align);
        let mut prev: rawptr u8 = ptr_null[u8]();
        let mut cur: rawptr u8 = st.head;
        while !is_null(cur) {
            let blk: FreeBlock = ptr_read(cast_ptr[FreeBlock](cur));
            if blk.size >= need {
                if is_null(prev) {
                    ptr_write(cast_ptr[FreeList](ctx), FreeList { next: st.next, end: st.end, head: blk.next });
                } else {
                    let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
                    ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: blk.next, size: pblk.size });
                }
                return cur;
            }
            prev = cur;
            cur = blk.next;
        }
        let aligned: usize = (st.next + align - 1usize) / align * align;
        if aligned + need > st.end { return ptr_null[u8](); }
        ptr_write(cast_ptr[FreeList](ctx), FreeList { next: aligned + need, end: st.end, head: st.head });
        return addr_to_ptr[u8](aligned);
    }
}
fn freelist_free(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) -> unit {
    unsafe "push freed block; its own storage holds the FreeBlock header" {
        let st: FreeList = ptr_read(cast_ptr[FreeList](ctx));
        let cap: usize = block_span(size, align);
        ptr_write(cast_ptr[FreeBlock](ptr), FreeBlock { next: st.head, size: cap });
        ptr_write(cast_ptr[FreeList](ctx), FreeList { next: st.next, end: st.end, head: ptr });
    }
}
fn main() -> i64 {
    unsafe "drive alloc/free directly over a fixed window" {
        let mut st: FreeList = with_window(16777216usize, 4096usize);
        let ctx: rawptr u8 = cast_ptr[u8](addr_of_mut(st));
        let p0: rawptr u8 = freelist_alloc(ctx, 8usize, 8usize);
        let p1: rawptr u8 = freelist_alloc(ctx, 8usize, 8usize);
        ptr_write(cast_ptr[i64](p1), 777i64);
        freelist_free(ctx, p0, 8usize, 8usize);
        let p2: rawptr u8 = freelist_alloc(ctx, 8usize, 8usize);
        let reused: bool = ptr_to_addr(p2) == ptr_to_addr(p0);
        let distinct: bool = ptr_to_addr(p2) != ptr_to_addr(p1);
        let live: i64 = ptr_read(cast_ptr[i64](p1));
        let mut r: i64 = 0i64;
        if reused { r = r + 1i64; }
        if distinct { r = r + 10i64; }
        if live == 777i64 { r = r + 100i64; }
        return r;
    }
}
"#;

#[test]
fn freelist_direct_alloc_free_reuse_all_engines() {
    // reused(+1) && distinct-from-live(+10) && live-block-untouched(+100) = 111.
    assert_all_engines(DIRECT_DRIVE, 111);
}
