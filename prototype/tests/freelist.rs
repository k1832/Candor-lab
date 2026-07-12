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
    unsafe "ctx points at the live FreeList whose [next, end) window and address-ordered free chain are reserved to this arena alone; every carved block stays inside [next, end) and is >= header-sized, and a split remainder (kept only when >= MIN_SPLIT) is written into the block's own tail" {
        let st: FreeList = ptr_read(cast_ptr[FreeList](ctx));
        let need: usize = block_span(size, align);
        let mut prev: rawptr u8 = ptr_null[u8]();
        let mut cur: rawptr u8 = st.head;
        while !is_null(cur) {
            let blk: FreeBlock = ptr_read(cast_ptr[FreeBlock](cur));
            if blk.size >= need {
                // Split only when the remainder can hold its own FreeBlock header
                // AND a minimum payload (MIN_SPLIT = 32 = a 16-byte header + a
                // 16-byte minimum block); below that, hand out the whole block and
                // accept the small internal fragmentation. The remainder takes the
                // block's slot in the address-ordered list (its address is > cur and
                // < blk.next, so the ordering and non-adjacency invariants hold).
                if blk.size - need >= 32usize {
                    let rem: rawptr u8 = addr_to_ptr[u8](ptr_to_addr(cur) + need);
                    ptr_write(cast_ptr[FreeBlock](rem), FreeBlock { next: blk.next, size: blk.size - need });
                    if is_null(prev) {
                        ptr_write(cast_ptr[FreeList](ctx), FreeList { next: st.next, end: st.end, head: rem });
                    } else {
                        let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
                        ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: rem, size: pblk.size });
                    }
                } else {
                    if is_null(prev) {
                        ptr_write(cast_ptr[FreeList](ctx), FreeList { next: st.next, end: st.end, head: blk.next });
                    } else {
                        let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
                        ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: blk.next, size: pblk.size });
                    }
                }
                return cur;
            }
            prev = cur;
            cur = blk.next;
        }
        let aligned: usize = (st.next + align - 1usize) / align * align;
        if aligned + need > st.end {
            return ptr_null[u8]();
        }
        ptr_write(cast_ptr[FreeList](ctx), FreeList { next: aligned + need, end: st.end, head: st.head });
        return addr_to_ptr[u8](aligned);
    }
}

fn freelist_free(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) -> unit {
    unsafe "the free list is kept ADDRESS-ORDERED; the freed block's own storage (>= header-sized, guaranteed by block_span in alloc) holds its FreeBlock header {next, size}, and a merge joins two blocks ONLY when their byte spans are exactly adjacent (addr + size == neighbour addr), so a merge can never overlap live memory nor bridge a gap" {
        let st: FreeList = ptr_read(cast_ptr[FreeList](ctx));
        let mut cap: usize = block_span(size, align);
        let a: usize = ptr_to_addr(ptr);
        // Insertion point: prev = last free block below `a`, cur = first above it.
        let mut prev: rawptr u8 = ptr_null[u8]();
        let mut cur: rawptr u8 = st.head;
        while !is_null(cur) && ptr_to_addr(cur) < a {
            let cblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](cur));
            prev = cur;
            cur = cblk.next;
        }
        // Forward coalesce: absorb `cur` when it begins exactly at a + cap.
        let mut link: rawptr u8 = cur;
        if !is_null(cur) {
            let nblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](cur));
            if a + cap == ptr_to_addr(cur) {
                cap = cap + nblk.size;
                link = nblk.next;
            }
        }
        // Backward coalesce: extend `prev` when it ends exactly at `a`.
        if !is_null(prev) {
            let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
            if ptr_to_addr(prev) + pblk.size == a {
                ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: link, size: pblk.size + cap });
                return;
            }
        }
        // No backward merge: insert the (possibly forward-merged) block at `a`.
        ptr_write(cast_ptr[FreeBlock](ptr), FreeBlock { next: link, size: cap });
        if is_null(prev) {
            ptr_write(cast_ptr[FreeList](ctx), FreeList { next: st.next, end: st.end, head: ptr });
        } else {
            let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
            ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: ptr, size: pblk.size });
        }
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

// ---------------------------------------------------------------------------
// Splitting: one large freed block services MANY small allocations. A no-split
// allocator hands out the whole block ONCE and then OOMs (frontier exhausted),
// so a count far above 1 can only come from splitting the block into pieces.
const SPLIT_DRIVE: &str = r#"
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
    unsafe "ctx points at the live FreeList whose [next, end) window and address-ordered free chain are reserved to this arena alone; every carved block stays inside [next, end) and is >= header-sized, and a split remainder (kept only when >= MIN_SPLIT) is written into the block's own tail" {
        let st: FreeList = ptr_read(cast_ptr[FreeList](ctx));
        let need: usize = block_span(size, align);
        let mut prev: rawptr u8 = ptr_null[u8]();
        let mut cur: rawptr u8 = st.head;
        while !is_null(cur) {
            let blk: FreeBlock = ptr_read(cast_ptr[FreeBlock](cur));
            if blk.size >= need {
                // Split only when the remainder can hold its own FreeBlock header
                // AND a minimum payload (MIN_SPLIT = 32 = a 16-byte header + a
                // 16-byte minimum block); below that, hand out the whole block and
                // accept the small internal fragmentation. The remainder takes the
                // block's slot in the address-ordered list (its address is > cur and
                // < blk.next, so the ordering and non-adjacency invariants hold).
                if blk.size - need >= 32usize {
                    let rem: rawptr u8 = addr_to_ptr[u8](ptr_to_addr(cur) + need);
                    ptr_write(cast_ptr[FreeBlock](rem), FreeBlock { next: blk.next, size: blk.size - need });
                    if is_null(prev) {
                        ptr_write(cast_ptr[FreeList](ctx), FreeList { next: st.next, end: st.end, head: rem });
                    } else {
                        let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
                        ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: rem, size: pblk.size });
                    }
                } else {
                    if is_null(prev) {
                        ptr_write(cast_ptr[FreeList](ctx), FreeList { next: st.next, end: st.end, head: blk.next });
                    } else {
                        let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
                        ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: blk.next, size: pblk.size });
                    }
                }
                return cur;
            }
            prev = cur;
            cur = blk.next;
        }
        let aligned: usize = (st.next + align - 1usize) / align * align;
        if aligned + need > st.end {
            return ptr_null[u8]();
        }
        ptr_write(cast_ptr[FreeList](ctx), FreeList { next: aligned + need, end: st.end, head: st.head });
        return addr_to_ptr[u8](aligned);
    }
}

fn freelist_free(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) -> unit {
    unsafe "the free list is kept ADDRESS-ORDERED; the freed block's own storage (>= header-sized, guaranteed by block_span in alloc) holds its FreeBlock header {next, size}, and a merge joins two blocks ONLY when their byte spans are exactly adjacent (addr + size == neighbour addr), so a merge can never overlap live memory nor bridge a gap" {
        let st: FreeList = ptr_read(cast_ptr[FreeList](ctx));
        let mut cap: usize = block_span(size, align);
        let a: usize = ptr_to_addr(ptr);
        // Insertion point: prev = last free block below `a`, cur = first above it.
        let mut prev: rawptr u8 = ptr_null[u8]();
        let mut cur: rawptr u8 = st.head;
        while !is_null(cur) && ptr_to_addr(cur) < a {
            let cblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](cur));
            prev = cur;
            cur = cblk.next;
        }
        // Forward coalesce: absorb `cur` when it begins exactly at a + cap.
        let mut link: rawptr u8 = cur;
        if !is_null(cur) {
            let nblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](cur));
            if a + cap == ptr_to_addr(cur) {
                cap = cap + nblk.size;
                link = nblk.next;
            }
        }
        // Backward coalesce: extend `prev` when it ends exactly at `a`.
        if !is_null(prev) {
            let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
            if ptr_to_addr(prev) + pblk.size == a {
                ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: link, size: pblk.size + cap });
                return;
            }
        }
        // No backward merge: insert the (possibly forward-merged) block at `a`.
        ptr_write(cast_ptr[FreeBlock](ptr), FreeBlock { next: link, size: cap });
        if is_null(prev) {
            ptr_write(cast_ptr[FreeList](ctx), FreeList { next: st.next, end: st.end, head: ptr });
        } else {
            let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
            ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: ptr, size: pblk.size });
        }
    }
}
fn main() -> i64 {
    unsafe "drive alloc/free directly: one freed block split into many small allocs" {
        // A 160-byte window: carve one 160-byte block, free it, and the frontier
        // is now exhausted (next == end). Every later allocation must come from
        // that single free block.
        let mut st: FreeList = with_window(16777216usize, 160usize);
        let ctx: rawptr u8 = cast_ptr[u8](addr_of_mut(st));
        let big: rawptr u8 = freelist_alloc(ctx, 160usize, 8usize);
        freelist_free(ctx, big, 160usize, 8usize);
        // Without splitting: alloc #1 returns the whole 160-byte block, then the
        // list is empty and the frontier is exhausted, so alloc #2 OOMs -> count 1.
        // With splitting: the block is carved into 16-byte pieces -> count 9.
        let mut count: i64 = 0i64;
        let mut i: i64 = 0i64;
        while i < 12i64 {
            let p: rawptr u8 = freelist_alloc(ctx, 16usize, 8usize);
            if !is_null(p) { count = count + 1i64; }
            i = i + 1i64;
        }
        return count;
    }
}
"#;

#[test]
fn freelist_split_one_block_many_allocs_all_engines() {
    // 9 sixteen-byte allocations carved from one 160-byte block (a no-split
    // allocator would yield 1 before OOM). Splitting stops when the remainder
    // drops below MIN_SPLIT (32): 160,144,...,48 split; the final 32 is handed
    // out whole -> 9 successes, then the block is spent and further allocs OOM.
    assert_all_engines(SPLIT_DRIVE, 9);
}

// ---------------------------------------------------------------------------
// Coalescing: fragment the window into three physically-adjacent blocks, free
// them so the free list holds adjacent pieces, then a LARGE allocation that only
// fits if the neighbours MERGED. Without coalescing the list holds three 64-byte
// blocks (none fits 192) and the frontier is exhausted, so the large alloc OOMs;
// with forward+backward coalescing the three merge into one 192-byte block.
const COALESCE_DRIVE: &str = r#"
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
    unsafe "ctx points at the live FreeList whose [next, end) window and address-ordered free chain are reserved to this arena alone; every carved block stays inside [next, end) and is >= header-sized, and a split remainder (kept only when >= MIN_SPLIT) is written into the block's own tail" {
        let st: FreeList = ptr_read(cast_ptr[FreeList](ctx));
        let need: usize = block_span(size, align);
        let mut prev: rawptr u8 = ptr_null[u8]();
        let mut cur: rawptr u8 = st.head;
        while !is_null(cur) {
            let blk: FreeBlock = ptr_read(cast_ptr[FreeBlock](cur));
            if blk.size >= need {
                // Split only when the remainder can hold its own FreeBlock header
                // AND a minimum payload (MIN_SPLIT = 32 = a 16-byte header + a
                // 16-byte minimum block); below that, hand out the whole block and
                // accept the small internal fragmentation. The remainder takes the
                // block's slot in the address-ordered list (its address is > cur and
                // < blk.next, so the ordering and non-adjacency invariants hold).
                if blk.size - need >= 32usize {
                    let rem: rawptr u8 = addr_to_ptr[u8](ptr_to_addr(cur) + need);
                    ptr_write(cast_ptr[FreeBlock](rem), FreeBlock { next: blk.next, size: blk.size - need });
                    if is_null(prev) {
                        ptr_write(cast_ptr[FreeList](ctx), FreeList { next: st.next, end: st.end, head: rem });
                    } else {
                        let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
                        ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: rem, size: pblk.size });
                    }
                } else {
                    if is_null(prev) {
                        ptr_write(cast_ptr[FreeList](ctx), FreeList { next: st.next, end: st.end, head: blk.next });
                    } else {
                        let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
                        ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: blk.next, size: pblk.size });
                    }
                }
                return cur;
            }
            prev = cur;
            cur = blk.next;
        }
        let aligned: usize = (st.next + align - 1usize) / align * align;
        if aligned + need > st.end {
            return ptr_null[u8]();
        }
        ptr_write(cast_ptr[FreeList](ctx), FreeList { next: aligned + need, end: st.end, head: st.head });
        return addr_to_ptr[u8](aligned);
    }
}

fn freelist_free(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) -> unit {
    unsafe "the free list is kept ADDRESS-ORDERED; the freed block's own storage (>= header-sized, guaranteed by block_span in alloc) holds its FreeBlock header {next, size}, and a merge joins two blocks ONLY when their byte spans are exactly adjacent (addr + size == neighbour addr), so a merge can never overlap live memory nor bridge a gap" {
        let st: FreeList = ptr_read(cast_ptr[FreeList](ctx));
        let mut cap: usize = block_span(size, align);
        let a: usize = ptr_to_addr(ptr);
        // Insertion point: prev = last free block below `a`, cur = first above it.
        let mut prev: rawptr u8 = ptr_null[u8]();
        let mut cur: rawptr u8 = st.head;
        while !is_null(cur) && ptr_to_addr(cur) < a {
            let cblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](cur));
            prev = cur;
            cur = cblk.next;
        }
        // Forward coalesce: absorb `cur` when it begins exactly at a + cap.
        let mut link: rawptr u8 = cur;
        if !is_null(cur) {
            let nblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](cur));
            if a + cap == ptr_to_addr(cur) {
                cap = cap + nblk.size;
                link = nblk.next;
            }
        }
        // Backward coalesce: extend `prev` when it ends exactly at `a`.
        if !is_null(prev) {
            let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
            if ptr_to_addr(prev) + pblk.size == a {
                ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: link, size: pblk.size + cap });
                return;
            }
        }
        // No backward merge: insert the (possibly forward-merged) block at `a`.
        ptr_write(cast_ptr[FreeBlock](ptr), FreeBlock { next: link, size: cap });
        if is_null(prev) {
            ptr_write(cast_ptr[FreeList](ctx), FreeList { next: st.next, end: st.end, head: ptr });
        } else {
            let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
            ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: ptr, size: pblk.size });
        }
    }
}
fn main() -> i64 {
    unsafe "drive alloc/free directly to force forward+backward coalescing" {
        // A 192-byte window carved into three adjacent 64-byte blocks; the
        // frontier is then exhausted (next == end), so a 192-byte allocation can
        // ONLY be served from merged free blocks.
        let mut st: FreeList = with_window(16777216usize, 192usize);
        let ctx: rawptr u8 = cast_ptr[u8](addr_of_mut(st));
        let a: rawptr u8 = freelist_alloc(ctx, 64usize, 8usize);
        let b: rawptr u8 = freelist_alloc(ctx, 64usize, 8usize);
        let c: rawptr u8 = freelist_alloc(ctx, 64usize, 8usize);
        // Free the ends first, then the middle: freeing `b` is physically adjacent
        // to the already-free `a` (backward) and `c` (forward), so all three merge.
        freelist_free(ctx, a, 64usize, 8usize);
        freelist_free(ctx, c, 64usize, 8usize);
        freelist_free(ctx, b, 64usize, 8usize);
        let big: rawptr u8 = freelist_alloc(ctx, 192usize, 8usize);
        let ok: bool = !is_null(big);
        let at_base: bool = ptr_to_addr(big) == 16777216usize;
        // Prove one contiguous 192-byte span: write sentinels at both ends.
        let mut far: i64 = 0i64;
        if ok {
            ptr_write(cast_ptr[i64](big), 12i64);
            let tail: rawptr u8 = addr_to_ptr[u8](ptr_to_addr(big) + 184usize);
            ptr_write(cast_ptr[i64](tail), 34i64);
            far = ptr_read(cast_ptr[i64](big)) + ptr_read(cast_ptr[i64](tail));
        }
        let mut r: i64 = 0i64;
        if ok { r = r + 1i64; }
        if at_base { r = r + 10i64; }
        if far == 46i64 { r = r + 100i64; }
        return r;
    }
}
"#;

#[test]
fn freelist_coalesce_fragment_then_large_alloc_all_engines() {
    // ok(+1) && merged-block-starts-at-base(+10) && 192-byte span writable
    // end-to-end(+100) = 111. Without coalescing the 192-byte alloc OOMs (r == 0).
    assert_all_engines(COALESCE_DRIVE, 111);
}
