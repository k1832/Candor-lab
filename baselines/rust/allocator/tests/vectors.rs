//! Frozen test suite from `docs/basket/spec-allocator.md` §4. Each test is named
//! by its vector ID (A1..A22). Every numbered vector is encoded, including the
//! full deterministic xorshift64 stress sequence (A21) with per-step invariant
//! assertions and the partial-coalescing anti-cheat (A22).

use allocator::{AllocError, Allocator, HDR, MAX_ALIGN};
use core::ptr::NonNull;

const MIN_REGION: usize = 1 << 20; // 1 MiB

/// Runs `body` with a fresh 8-aligned region of exactly `bytes` bytes.
///
/// The region is backed by a `Vec<u64>` so it is 8-aligned, which the A22
/// exact-carving arithmetic depends on.
macro_rules! with_region {
    ($bytes:expr, $a:ident, $body:block) => {{
        let bytes: usize = $bytes;
        let mut backing = vec![0u64; bytes.div_ceil(8)];
        // SAFETY: `backing` owns `bytes` initialized, 8-aligned bytes that
        // outlive the borrow; nothing else touches them while `$a` is alive.
        let region: &mut [u8] =
            unsafe { core::slice::from_raw_parts_mut(backing.as_mut_ptr().cast(), bytes) };
        let mut $a = Allocator::init(region);
        $body
    }};
}

fn addr(p: NonNull<u8>) -> usize {
    p.as_ptr() as usize
}

fn write_fill(p: NonNull<u8>, size: usize, fill: u8) {
    // SAFETY: `[p, p + size)` is a live block of at least `size` bytes.
    unsafe { core::ptr::write_bytes(p.as_ptr(), fill, size) };
}

fn read_slice<'b>(p: NonNull<u8>, size: usize) -> &'b [u8] {
    // SAFETY: `[p, p + size)` is a live, initialized block.
    unsafe { core::slice::from_raw_parts(p.as_ptr(), size) }
}

fn all_equal(p: NonNull<u8>, size: usize, fill: u8) -> bool {
    read_slice(p, size).iter().all(|&b| b == fill)
}

// ---------------------------------------------------------------- Nominal ----

#[test]
fn a1_alloc_16_1_roundtrip() {
    with_region!(MIN_REGION, a, {
        let p = a.alloc(16, 1).unwrap();
        write_fill(p, 16, 0x37);
        assert!(all_equal(p, 16, 0x37));
    });
}

#[test]
fn a2_alloc_minimum_size() {
    with_region!(MIN_REGION, a, {
        let p = a.alloc(1, 1).unwrap();
        write_fill(p, 1, 0xEE);
        assert!(all_equal(p, 1, 0xEE));
    });
}

#[test]
fn a3_alloc_4096_aligned() {
    with_region!(MIN_REGION, a, {
        let p = a.alloc(4096, 4096).unwrap();
        assert_eq!(addr(p) % 4096, 0);
    });
}

#[test]
fn a4_ten_allocs_nonoverlapping() {
    with_region!(MIN_REGION, a, {
        let mut ptrs = Vec::new();
        for _ in 0..10 {
            let p = a.alloc(1000, 8).unwrap();
            assert_eq!(addr(p) % 8, 0);
            ptrs.push(p);
        }
        for i in 0..ptrs.len() {
            for j in (i + 1)..ptrs.len() {
                let (ai, aj) = (addr(ptrs[i]), addr(ptrs[j]));
                assert!(ai + 1000 <= aj || aj + 1000 <= ai, "blocks overlap");
            }
        }
    });
}

#[test]
fn a5_neighbor_write_does_not_corrupt() {
    with_region!(MIN_REGION, a, {
        let p1 = a.alloc(64, 64).unwrap();
        write_fill(p1, 64, 0xA5);
        let p2 = a.alloc(64, 64).unwrap();
        write_fill(p2, 64, 0x5A);
        assert!(all_equal(p1, 64, 0xA5));
    });
}

#[test]
fn a6_realloc_grow_preserves_content_and_alignment() {
    with_region!(MIN_REGION, a, {
        let p = a.alloc(100, 64).unwrap();
        assert_eq!(addr(p) % 64, 0);
        write_fill(p, 100, 0xC3);
        let q = a.realloc(p.as_ptr(), 400).unwrap();
        assert_eq!(
            addr(q) % 64,
            0,
            "realloc must preserve the original alignment"
        );
        assert!(all_equal(q, 100, 0xC3));
        // The new block is usable for the full 400 bytes.
        write_fill(q, 400, 0xC3);
        assert!(all_equal(q, 400, 0xC3));
    });
}

#[test]
fn a7_realloc_shrink_preserves_prefix() {
    with_region!(MIN_REGION, a, {
        let p = a.alloc(400, 8).unwrap();
        write_fill(p, 400, 0x11);
        let q = a.realloc(p.as_ptr(), 50).unwrap();
        assert!(all_equal(q, 50, 0x11));
    });
}

// --------------------------------------------------------------- Boundary ----

#[test]
fn a8_all_alignments_honored() {
    with_region!(MIN_REGION, a, {
        let mut align = 1usize;
        while align <= MAX_ALIGN {
            let p = a.alloc(32, align).unwrap();
            assert_eq!(addr(p) % align, 0, "alignment {align} not honored");
            align <<= 1;
        }
    });
}

#[test]
fn a9_largest_block_then_one_more_fails() {
    // Binary-search the largest single block a fresh region admits.
    let mut lo = 1usize;
    let mut hi = MIN_REGION;
    while lo < hi {
        let mid = lo + (hi - lo).div_ceil(2);
        let ok = with_region!(MIN_REGION, a, { a.alloc(mid, 1).is_ok() });
        if ok {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    let largest = lo;
    with_region!(MIN_REGION, a, {
        assert!(a.alloc(largest, 1).is_ok(), "largest block must succeed");
    });
    with_region!(MIN_REGION, a, {
        assert_eq!(
            a.alloc(largest + 1, 1),
            Err(AllocError::OutOfMemory),
            "one byte over the maximum must be OOM"
        );
    });
}

#[test]
fn a10_coalesce_full_span() {
    with_region!(MIN_REGION, a, {
        let mut ptrs = Vec::new();
        loop {
            match a.alloc(1024, 1) {
                Ok(p) => ptrs.push(p),
                Err(AllocError::OutOfMemory) => break,
                Err(e) => panic!("unexpected error {e:?}"),
            }
        }
        let k = ptrs.len();
        for p in &ptrs {
            a.free(p.as_ptr()).unwrap();
        }
        assert!(
            a.alloc(k * 1024 - 64, 1).is_ok(),
            "coalesced free space must satisfy K*1024-64"
        );
    });
}

#[test]
fn a11_free_orders_then_full_span() {
    // Reverse order.
    with_region!(MIN_REGION, a, {
        let mut ptrs = Vec::new();
        loop {
            match a.alloc(1024, 1) {
                Ok(p) => ptrs.push(p),
                Err(AllocError::OutOfMemory) => break,
                Err(e) => panic!("unexpected error {e:?}"),
            }
        }
        let k = ptrs.len();
        for p in ptrs.iter().rev() {
            a.free(p.as_ptr()).unwrap();
        }
        assert!(
            a.alloc(k * 1024 - 64, 1).is_ok(),
            "reverse-free span failed"
        );
    });
    // Interleaved order (evens then odds).
    with_region!(MIN_REGION, a, {
        let mut ptrs = Vec::new();
        loop {
            match a.alloc(1024, 1) {
                Ok(p) => ptrs.push(p),
                Err(AllocError::OutOfMemory) => break,
                Err(e) => panic!("unexpected error {e:?}"),
            }
        }
        let k = ptrs.len();
        for (i, p) in ptrs.iter().enumerate() {
            if i % 2 == 0 {
                a.free(p.as_ptr()).unwrap();
            }
        }
        for (i, p) in ptrs.iter().enumerate() {
            if i % 2 == 1 {
                a.free(p.as_ptr()).unwrap();
            }
        }
        assert!(
            a.alloc(k * 1024 - 64, 1).is_ok(),
            "interleaved-free span failed"
        );
    });
}

// ------------------------------------------------------------------ Error ----

#[test]
fn a12_alloc_zero_size() {
    with_region!(MIN_REGION, a, {
        assert_eq!(a.alloc(0, 8), Err(AllocError::InvalidSize));
    });
}

#[test]
fn a13_alloc_zero_align() {
    with_region!(MIN_REGION, a, {
        assert_eq!(a.alloc(16, 0), Err(AllocError::InvalidAlign));
    });
}

#[test]
fn a14_alloc_non_power_of_two_align() {
    with_region!(MIN_REGION, a, {
        assert_eq!(a.alloc(16, 3), Err(AllocError::InvalidAlign));
    });
}

#[test]
fn a15_alloc_align_over_max() {
    with_region!(MIN_REGION, a, {
        assert_eq!(a.alloc(16, 8192), Err(AllocError::InvalidAlign));
    });
}

#[test]
fn a16_alloc_too_large_oom() {
    with_region!(MIN_REGION, a, {
        assert_eq!(a.alloc(2 * MIN_REGION, 1), Err(AllocError::OutOfMemory));
    });
}

#[test]
fn a17_free_null_and_out_of_range() {
    with_region!(MIN_REGION, a, {
        assert_eq!(a.free(core::ptr::null_mut()), Err(AllocError::InvalidPtr));
        // A clearly out-of-range pointer (well above the region).
        assert_eq!(a.free(usize::MAX as *mut u8), Err(AllocError::InvalidPtr));
    });
}

#[test]
fn a18_double_free() {
    with_region!(MIN_REGION, a, {
        let p = a.alloc(128, 8).unwrap();
        assert_eq!(a.free(p.as_ptr()), Ok(()));
        assert_eq!(a.free(p.as_ptr()), Err(AllocError::InvalidPtr));
    });
}

#[test]
fn a19_realloc_zero_size() {
    with_region!(MIN_REGION, a, {
        let p = a.alloc(64, 8).unwrap();
        assert_eq!(a.realloc(p.as_ptr(), 0), Err(AllocError::InvalidSize));
    });
}

#[test]
fn a20_realloc_oom_leaves_original() {
    with_region!(MIN_REGION, a, {
        let p = a.alloc(128, 8).unwrap();
        write_fill(p, 128, 0x7C);
        // Grow beyond the whole region: cannot fit.
        assert_eq!(
            a.realloc(p.as_ptr(), 2 * MIN_REGION),
            Err(AllocError::OutOfMemory)
        );
        // Original block is still live and unchanged.
        assert!(all_equal(p, 128, 0x7C));
        assert_eq!(a.free(p.as_ptr()), Ok(()));
    });
}

// ----------------------------------------------- A21 deterministic stress ----

struct Xorshift64 {
    state: u64,
}

impl Xorshift64 {
    fn new() -> Self {
        Xorshift64 {
            state: 0x2545_F491_4F6C_DD1D,
        }
    }
    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

#[derive(Clone, Copy)]
struct Slot {
    ptr: NonNull<u8>,
    size: usize,
    align: usize,
    fill: u8,
}

#[test]
fn a21_deterministic_stress() {
    const SLOTS: usize = 256;
    const STEPS: usize = 20_000;

    with_region!(MIN_REGION, a, {
        let (lo_ptr, hi_ptr) = a.region();
        let region_lo = lo_ptr as usize;
        let region_hi = hi_ptr as usize;

        let mut rng = Xorshift64::new();
        let mut slots: [Option<Slot>; SLOTS] = [None; SLOTS];

        for _ in 0..STEPS {
            let w = rng.next();
            let slot = ((w >> 2) % 256) as usize;
            let op = w & 0x3;

            match op {
                0 | 1 => {
                    if slots[slot].is_none() {
                        let size = ((w >> 10) % 4096) as usize + 1;
                        let align = 1usize << ((w >> 24) % 5);
                        let fill = ((w >> 32) & 0xFF) as u8;
                        match a.alloc(size, align) {
                            Ok(ptr) => {
                                write_fill(ptr, size, fill);
                                slots[slot] = Some(Slot {
                                    ptr,
                                    size,
                                    align,
                                    fill,
                                });
                            }
                            Err(AllocError::OutOfMemory) => {}
                            Err(e) => panic!("unexpected alloc error {e:?}"),
                        }
                    }
                }
                2 => {
                    if let Some(s) = slots[slot].take() {
                        assert!(
                            all_equal(s.ptr, s.size, s.fill),
                            "fill integrity before free"
                        );
                        a.free(s.ptr.as_ptr()).unwrap();
                    }
                }
                3 => {
                    if let Some(mut s) = slots[slot] {
                        let nsize = ((w >> 10) % 4096) as usize + 1;
                        assert!(
                            all_equal(s.ptr, s.size, s.fill),
                            "fill integrity before realloc"
                        );
                        match a.realloc(s.ptr.as_ptr(), nsize) {
                            Ok(np) => {
                                write_fill(np, nsize, s.fill);
                                s.ptr = np;
                                s.size = nsize;
                                slots[slot] = Some(s);
                            }
                            Err(AllocError::OutOfMemory) => {
                                assert!(
                                    all_equal(s.ptr, s.size, s.fill),
                                    "OOM realloc must leave block unchanged"
                                );
                            }
                            Err(e) => panic!("unexpected realloc error {e:?}"),
                        }
                    }
                }
                _ => unreachable!(),
            }

            // Per-step invariants (spec §4.4) over all live slots.
            let mut live: Vec<(usize, usize)> = Vec::new();
            for s in slots.iter().flatten() {
                let start = addr(s.ptr);
                let end = start + s.size;
                // (3.1) in-bounds and (3.3) size honored.
                assert!(
                    start >= region_lo && end <= region_hi,
                    "block out of region"
                );
                // (3.2) alignment.
                assert_eq!(start % s.align, 0, "alignment violated");
                // (3.5) fill integrity.
                assert!(all_equal(s.ptr, s.size, s.fill), "fill integrity violated");
                live.push((start, end));
            }
            // (3.4) pairwise non-overlap, checked via sorted intervals.
            live.sort_unstable();
            for pair in live.windows(2) {
                assert!(pair[0].1 <= pair[1].0, "live blocks overlap");
            }
        }
    });
}

// -------------------------------------------- A22 partial coalescing check ----

#[test]
fn a22_partial_middle_coalesce() {
    // Region sized to exactly ten 1 KiB blocks, no tail: the final allocation
    // can only land in the reclaimed b3+b4+b5 span if the three freed middle
    // blocks were coalesced into one 3 KiB free region.
    const BLOCK: usize = 1024;
    let region_bytes = 10 * BLOCK;
    with_region!(region_bytes, a, {
        let mut b = Vec::new();
        for _ in 0..10 {
            b.push(a.alloc(BLOCK - HDR, 1).expect("ten 1 KiB blocks must fit"));
        }
        // The region must now be full: an eleventh block cannot fit.
        assert_eq!(
            a.alloc(1, 1),
            Err(AllocError::OutOfMemory),
            "region should be full"
        );

        a.free(b[3].as_ptr()).unwrap();
        a.free(b[4].as_ptr()).unwrap();
        a.free(b[5].as_ptr()).unwrap();

        // b0,b1,b2,b6,b7,b8,b9 stay live; only the coalesced middle can hold this.
        let big = a
            .alloc(3 * BLOCK - HDR, 1)
            .expect("coalesced 3 KiB middle span must satisfy the request");

        // It must not overlap any still-live block.
        let big_lo = addr(big);
        let big_hi = big_lo + (3 * BLOCK - HDR);
        for (i, p) in b.iter().enumerate() {
            if matches!(i, 3..=5) {
                continue;
            }
            let lo = addr(*p);
            let hi = lo + (BLOCK - HDR);
            assert!(
                big_hi <= lo || hi <= big_lo,
                "reused span overlaps live block b{i}"
            );
        }
    });
}
