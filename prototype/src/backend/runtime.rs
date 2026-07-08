//! Stage B runtime shim (design 0010 §5): the tiny host runtime the JIT'd native
//! code links against. It holds the **flat memory model** (the same base+offset
//! `Mem` substrate the interpreter uses — `interp::mem`), the observable `θ`
//! trace, the stack-bump allocator, and the **fault-exit hook**.
//!
//! ## The flat memory model (design 0001 §4.2, faithful to `interp::mem`)
//! One host buffer of `MAX_ADDR` bytes; a *Candor address* `A` maps to host
//! `base + A`. Native loads/stores are ordinary machine loads/stores at
//! `base + A`; the compiler bakes `base` as a constant. Stack storage bumps from
//! `STACK_BASE`, static storage from `STATIC_BASE` — exactly the interpreter's
//! layout, so a `Ref`/`Deref` round-trips and a program that mints an absolute
//! address (`addr_to_ptr[u32](0x300000)`, the MMIO/allocator fixtures) reads and
//! writes it identically to the oracle.
//!
//! ## The fault-exit hook (design 0010 §3, the chosen *hook* mechanism)
//! Every MIR fault edge lowers to a `call rt_fault(kind, span_start, span_end)`.
//! The hook records `(k, s)` and unwinds the native call stack with `_longjmp`
//! back to the driver's `_setjmp` landing pad — the portable non-local exit the
//! design names. Cranelift frames carry no destructors, so skipping them is
//! sound; the memory buffer and trace live in this Rust-owned `Runtime`, untouched
//! by the unwind. The harness then reports `(k, s)` as a clean process-level
//! result. `TrapCode`/side-table trapping (the doc's alternative) is subsumed:
//! `(k, s)` travel as immediate call arguments, so no PC-keyed side-table exists.

use std::os::raw::{c_int, c_void};
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::interp::mem::MAX_ADDR;

extern "C" {
    // glibc/musl: `_setjmp`/`_longjmp` save/restore the machine context without
    // touching the signal mask (faster, and sufficient — no signals cross here).
    fn _setjmp(env: *mut c_void) -> c_int;
    fn _longjmp(env: *mut c_void, val: c_int) -> !;
}

/// The host runtime: flat memory, the bump pointers, the `θ` trace, the delivered
/// fault, and the `_setjmp` landing buffer.
pub struct Runtime {
    buf: Vec<u8>,
    pub base: *mut u8,
    pub stack_bump: u64,
    pub static_bump: u64,
    pub trace: Vec<i64>,
    /// The delivered fault `(kind, span.start, span.end)` (`None` == ran to return).
    pub fault: Option<(u32, usize, usize)>,
    /// The `_setjmp`/`_longjmp` buffer (glibc `jmp_buf` is ~200 bytes; 512 is safe).
    jmp: [u64; 64],
}

impl Runtime {
    pub fn new() -> Box<Runtime> {
        let mut buf = vec![0u8; MAX_ADDR as usize];
        let base = buf.as_mut_ptr();
        Box::new(Runtime {
            buf,
            base,
            stack_bump: crate::interp::mem::STACK_BASE,
            static_bump: crate::interp::mem::STATIC_BASE,
            trace: Vec::new(),
            fault: None,
            jmp: [0u64; 64],
        })
    }

    /// Reserve `size` bytes of static storage at `align` (compile-time layout uses
    /// the identical arithmetic so baked `StaticAddr`/`StrAddr` constants agree).
    pub fn static_alloc(&mut self, size: u64, align: u64) -> u64 {
        let a = round_up(self.static_bump, align.max(1));
        self.static_bump = a + size;
        a
    }

    /// Write raw bytes at a Candor address (driver-side: string bytes, statics).
    pub fn write_bytes(&mut self, addr: u64, data: &[u8]) {
        let s = addr as usize;
        self.buf[s..s + data.len()].copy_from_slice(data);
    }
}

// The current runtime, read by the shim symbols the JIT calls. The whole-program
// JIT run is single-threaded, so a process-global pointer is sufficient and cheap.
static CURRENT: AtomicPtr<Runtime> = AtomicPtr::new(std::ptr::null_mut());

#[inline]
fn rt() -> &'static mut Runtime {
    unsafe { &mut *CURRENT.load(Ordering::SeqCst) }
}

pub fn set_current(rt: *mut Runtime) {
    CURRENT.store(rt, Ordering::SeqCst);
}

pub fn clear_current() {
    CURRENT.store(std::ptr::null_mut(), Ordering::SeqCst);
}

// ---------------------------------------------------------------------------
// Shim symbols the compiled code calls (registered with the JIT by name).
// ---------------------------------------------------------------------------

/// Reserve + zero a stack slot; returns its Candor address (mirrors
/// `Mem::stack_alloc` + the zero-on-alloc the interpreter performs).
pub extern "C" fn rt_stack_alloc(size: u64, align: u64) -> u64 {
    let r = rt();
    let a = round_up(r.stack_bump, align.max(1));
    r.stack_bump = a + size;
    if size != 0 {
        unsafe { std::ptr::write_bytes(r.base.add(a as usize), 0, size as usize) };
    }
    a
}

/// Byte-copy `len` bytes `src -> dst` within the flat model (`CopyVal`, returns).
pub extern "C" fn rt_copy(dst: u64, src: u64, len: u64) {
    if len == 0 {
        return;
    }
    let r = rt();
    unsafe {
        std::ptr::copy(r.base.add(src as usize), r.base.add(dst as usize), len as usize);
    }
}

/// The observable `trace(x)` hook (INV-OBS-ORDER): append `x` to `θ`. Because it
/// is a call with a side effect, no Cranelift pass reorders across it.
pub extern "C" fn rt_trace(v: i64) {
    rt().trace.push(v);
}

/// The observable rawptr/MMIO **load** hook (INV-OBS-ORDER, design 0010 §1/§2 F1
/// discipline). A rawptr read (`ptr_read` through an `addr_to_ptr` pointer) is an
/// observable per the formalization's substrate note (§2.1); lowering it as a call
/// with a side effect makes it an ordering barrier **Cranelift's egraph respects**
/// at `opt_level=speed` — it is never reordered, coalesced, or eliminated. Reads
/// `size` bytes at the flat address, zero-extended; the caller sign/zero-canonicalizes.
pub extern "C" fn rt_mmio_load(addr: u64, size: u64) -> i64 {
    let r = rt();
    let mut buf = [0u8; 8];
    let n = (size as usize).min(8);
    unsafe {
        std::ptr::copy_nonoverlapping(r.base.add(addr as usize), buf.as_mut_ptr(), n);
    }
    i64::from_le_bytes(buf)
}

/// The observable rawptr/MMIO **store** hook (INV-OBS-ORDER): a rawptr write
/// (`ptr_write` through an `addr_to_ptr` pointer), lowered as a barrier call.
/// Writes the low `size` bytes of `val` at the flat address.
pub extern "C" fn rt_mmio_store(addr: u64, val: i64, size: u64) {
    let r = rt();
    let bytes = val.to_le_bytes();
    let n = (size as usize).min(8);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), r.base.add(addr as usize), n);
    }
}

/// The fault-exit hook: record `(k, s)` and `_longjmp` to the driver. Never
/// returns; the lowering follows the call with an unreachable `trap`.
pub extern "C" fn rt_fault(kind: u32, span_start: u32, span_end: u32) {
    let r = rt();
    r.fault = Some((kind, span_start as usize, span_end as usize));
    unsafe {
        _longjmp(r.jmp.as_mut_ptr() as *mut c_void, 1);
    }
}

/// Establish the `_setjmp` landing pad and run `body` (the static inits + `main`).
/// Returns `true` if `body` ran to completion, `false` if a fault longjmp'd back.
///
/// `body` runs only on the first (`_setjmp == 0`) return; on the fault path the
/// second return skips it. All post-run state is read from the `Runtime` behind a
/// pointer (never a stack local held across `_setjmp`), so the twice-returning
/// call cannot leave a stale cached value.
#[inline(never)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn run_guarded(rt_ptr: *mut Runtime, body: impl FnOnce()) -> bool {
    let env = unsafe { (*rt_ptr).jmp.as_mut_ptr() as *mut c_void };
    let landed = unsafe { _setjmp(env) };
    let landed = std::hint::black_box(landed);
    if landed == 0 {
        body();
        true
    } else {
        false
    }
}

pub fn round_up(x: u64, align: u64) -> u64 {
    let a = align.max(1);
    x.div_ceil(a) * a
}
