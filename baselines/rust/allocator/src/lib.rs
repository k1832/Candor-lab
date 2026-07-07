#![no_std]
//! General-purpose dynamic memory allocator over a single caller-provided byte
//! region, satisfying `docs/basket/spec-allocator.md`.
//!
//! The free-list engine (first-fit placement with adjacent-hole coalescing on
//! free) is the `linked_list_allocator` crate's `Heap`/hole-list engine,
//! **vendored** into `src/vendored_llalloc.rs` (adjudication ruling of
//! 2026-07-07, measured-artifact self-containment). This module adapts it to
//! the spec's abstract interface: a caller-provided region, arbitrary sizes,
//! caller-specified alignment, pointer-only `free`/`realloc`, and errors
//! returned as values. See `README.md` for the full list of adaptations.

mod vendored_llalloc;

use core::alloc::Layout;
use core::marker::PhantomData;
use core::mem::size_of;
use core::ptr::{self, NonNull};

use crate::vendored_llalloc::Heap;

/// Largest alignment the allocator honors (spec `MAX_ALIGN`).
pub const MAX_ALIGN: usize = 4096;

/// Per-block metadata overhead in bytes (the spec's declared `HDR`, `<= 64`).
///
/// For an `align == 1` request the allocator consumes exactly `HDR + size`
/// bytes (rounded to the free-list granularity), which is what the A22 partial
/// coalescing vector relies on.
pub const HDR: usize = size_of::<Header>();

/// Liveness marker written into every live block's header. A freed block no
/// longer carries it, which is how double-free and stale pointers are rejected.
const MAGIC_LIVE: usize = 0x616C_6C6F_635F_6864; // b"alloc_hd"
const MAGIC_FREE: usize = 0;

/// In-band per-allocation header, stored immediately before the user pointer.
#[repr(C)]
#[derive(Clone, Copy)]
struct Header {
    /// Liveness marker (`MAGIC_LIVE` while allocated).
    magic: usize,
    /// Block start returned by the underlying free-list allocator.
    base: *mut u8,
    /// Size passed to the free-list allocator (needed to free the exact block).
    raw_size: usize,
    /// Caller-requested alignment, preserved across `realloc` (spec §2.4).
    align: usize,
    /// Caller-requested usable size, used for `realloc` content preservation.
    size: usize,
}

/// Failure values returned in-band; the allocator never faults (spec §2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllocError {
    /// `E_INVALID_SIZE`: a zero size was requested.
    InvalidSize,
    /// `E_INVALID_ALIGN`: alignment is zero, not a power of two, or `> MAX_ALIGN`.
    InvalidAlign,
    /// `E_INVALID_PTR`: the pointer is not a currently-live allocation.
    InvalidPtr,
    /// `E_OUT_OF_MEMORY`: no free block can satisfy the request.
    OutOfMemory,
}

/// An allocator managing exactly the region handed to [`Allocator::init`].
pub struct Allocator<'a> {
    heap: Heap,
    region_start: *mut u8,
    region_end: *mut u8,
    _region: PhantomData<&'a mut [u8]>,
}

impl<'a> Allocator<'a> {
    /// Creates an allocator that manages exactly `region`.
    ///
    /// The spec requires `region.len() >= MIN_REGION` (1 MiB); a region too
    /// small to hold even the free-list metadata yields an allocator whose
    /// every allocation fails gracefully with `OutOfMemory` rather than panics.
    pub fn init(region: &'a mut [u8]) -> Self {
        let start = region.as_mut_ptr();
        let len = region.len();
        let heap = if len < 3 * size_of::<usize>() {
            Heap::empty()
        } else {
            // SAFETY: `region` is a valid, exclusively-borrowed byte span that
            // lives for `'a` (which outlives `self`); the heap takes exclusive
            // ownership of it for bookkeeping and allocations.
            unsafe { Heap::new(start, len) }
        };
        Allocator {
            heap,
            region_start: start,
            region_end: start.wrapping_add(len),
            _region: PhantomData,
        }
    }

    /// Allocates a block of at least `size` bytes aligned to `align`.
    pub fn alloc(&mut self, size: usize, align: usize) -> Result<NonNull<u8>, AllocError> {
        if size == 0 {
            return Err(AllocError::InvalidSize);
        }
        if align == 0 || !align.is_power_of_two() || align > MAX_ALIGN {
            return Err(AllocError::InvalidAlign);
        }

        // Reserve `pad` (a multiple of `align`, at least `HDR`) in front of the
        // user pointer so the header sits just below an `align`-aligned address.
        let pad = align_up(HDR, align);
        let raw_size = pad.checked_add(size).ok_or(AllocError::OutOfMemory)?;
        let layout =
            Layout::from_size_align(raw_size, align).map_err(|_| AllocError::OutOfMemory)?;

        let base = self
            .heap
            .allocate_first_fit(layout)
            .map_err(|()| AllocError::OutOfMemory)?
            .as_ptr();

        // SAFETY: the free-list returned a `raw_size`-byte block at `base`
        // aligned to `align`; `pad <= raw_size` and is a multiple of `align`, so
        // `user` is `align`-aligned and the header fits in `[base, user)`.
        let user = unsafe { base.add(pad) };
        let header = Header {
            magic: MAGIC_LIVE,
            base,
            raw_size,
            align,
            size,
        };
        // SAFETY: `[user - HDR, user)` lies within the just-allocated block.
        unsafe { write_header(user, &header) };
        Ok(NonNull::new(user).expect("allocated user pointer is non-null"))
    }

    // `free`/`realloc` take an arbitrary caller pointer and validate it against
    // the region bounds inside `header_at` *before* any dereference, so they are
    // memory-safe for every input, including wild, interior, and null pointers.
    // clippy's `not_unsafe_ptr_arg_deref` cannot see that guard; marking the
    // functions `unsafe` would contradict the spec's error-as-value contract.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    /// Frees a block previously returned by [`alloc`](Self::alloc) or
    /// [`realloc`](Self::realloc). Rejects null, out-of-range, and already-freed
    /// pointers with `InvalidPtr`.
    pub fn free(&mut self, ptr: *mut u8) -> Result<(), AllocError> {
        let header = self.header_at(ptr).ok_or(AllocError::InvalidPtr)?;
        // SAFETY: `header_at` validated that `ptr` is a live allocation, so its
        // header is within the region; clearing the marker first makes a
        // subsequent free of the same pointer fail the liveness check.
        unsafe { clear_magic(ptr) };
        self.dealloc(&header);
        Ok(())
    }

    // `free`/`realloc` take an arbitrary caller pointer and validate it against
    // the region bounds inside `header_at` *before* any dereference, so they are
    // memory-safe for every input, including wild, interior, and null pointers.
    // clippy's `not_unsafe_ptr_arg_deref` cannot see that guard; marking the
    // functions `unsafe` would contradict the spec's error-as-value contract.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    /// Resizes the block at `ptr` to at least `new_size`, preserving its
    /// original alignment and the leading `min(old, new)` bytes. The block may
    /// move; on `OutOfMemory` the original stays live and unchanged (spec §2.4).
    pub fn realloc(&mut self, ptr: *mut u8, new_size: usize) -> Result<NonNull<u8>, AllocError> {
        if new_size == 0 {
            return Err(AllocError::InvalidSize);
        }
        let header = self.header_at(ptr).ok_or(AllocError::InvalidPtr)?;

        // Place the replacement before releasing the original, so a failure
        // leaves the original untouched.
        let new_ptr = self.alloc(new_size, header.align)?;
        let keep = header.size.min(new_size);
        // SAFETY: both blocks are live and disjoint; `keep` bytes are valid and
        // initialized in the source and writable in the destination.
        unsafe { ptr::copy_nonoverlapping(ptr, new_ptr.as_ptr(), keep) };
        // SAFETY: `ptr` is still the live original validated above.
        unsafe { clear_magic(ptr) };
        self.dealloc(&header);
        Ok(new_ptr)
    }

    /// Half-open address range `[start, end)` of the managed region, for
    /// informative in-bounds assertions (spec §3.1).
    pub fn region(&self) -> (*const u8, *const u8) {
        (self.region_start, self.region_end)
    }

    /// Bytes currently available for allocation (informative only, spec §2.5).
    pub fn free_bytes(&self) -> usize {
        self.heap.free()
    }

    fn dealloc(&mut self, header: &Header) {
        let layout = Layout::from_size_align(header.raw_size, header.align)
            .expect("layout was valid when the block was allocated");
        // SAFETY: `base`/`raw_size` describe a live block produced by
        // `allocate_first_fit` with `layout`; `base` is non-null.
        unsafe {
            let base = NonNull::new_unchecked(header.base);
            self.heap.deallocate(base, layout);
        }
    }

    /// Validates `ptr` as a currently-live allocation and returns a copy of its
    /// header, or `None` if the pointer is null, out of range, or not live.
    fn header_at(&self, ptr: *mut u8) -> Option<Header> {
        if ptr.is_null() {
            return None;
        }
        let addr = ptr as usize;
        // The header occupies `[ptr - HDR, ptr)`; require it to lie in-region so
        // the read below can never touch memory outside the region.
        if addr < self.region_start as usize + HDR || addr > self.region_end as usize {
            return None;
        }
        // SAFETY: `[ptr - HDR, ptr)` is inside the region; every byte is
        // initialized and an unaligned read of the plain-old-data header is
        // well-defined for any bit pattern.
        let header = unsafe { read_header(ptr) };
        if header.magic != MAGIC_LIVE {
            return None;
        }
        // A live block's recorded base must itself be in-region.
        if (header.base as usize) < self.region_start as usize
            || header.base as usize >= self.region_end as usize
        {
            return None;
        }
        Some(header)
    }
}

/// Rounds `value` up to the next multiple of the power-of-two `align`.
fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

/// Writes `header` into `[user - HDR, user)`.
///
/// # Safety
/// `[user - HDR, user)` must be a writable byte range owned by the caller.
unsafe fn write_header(user: *mut u8, header: &Header) {
    ptr::write_unaligned(user.sub(HDR).cast::<Header>(), *header);
}

/// Reads the header stored in `[user - HDR, user)`.
///
/// # Safety
/// `[user - HDR, user)` must be a readable, initialized byte range.
unsafe fn read_header(user: *mut u8) -> Header {
    ptr::read_unaligned(user.sub(HDR).cast::<Header>())
}

/// Clears the liveness marker of the header stored below `user`.
///
/// # Safety
/// `[user - HDR, user)` must be a writable byte range holding a `Header`.
unsafe fn clear_magic(user: *mut u8) {
    // `magic` is the first field of the `repr(C)` header.
    ptr::write_unaligned(user.sub(HDR).cast::<usize>(), MAGIC_FREE);
}
