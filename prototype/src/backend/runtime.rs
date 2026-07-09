//! Stage B runtime shim (design 0010 §5) + Stage 2 native concurrency (design
//! 0012 §6). The tiny host runtime the JIT'd native code links against: the
//! **flat memory model** (`interp::mem`), the observable `θ` trace, the stack-bump
//! allocator, the **fault-exit hook**, and — new in Stage 2 — **real OS-thread
//! `spawn`/`join`** over `std::thread` (pthreads on Linux).
//!
//! ## The flat memory model (design 0001 §4.2, faithful to `interp::mem`)
//! One host buffer of `MAX_ADDR` bytes; a *Candor address* `A` maps to host
//! `base + A`. Native loads/stores are ordinary machine loads/stores at `base + A`;
//! the compiler bakes `base` as a constant. Under Stage-2 parallelism the buffer is
//! the **shared substrate**: every task thread reads/writes it through the same
//! `base`. The Stage-1 checker guarantees DRF over language-visible state, so
//! concurrent tasks only ever touch **disjoint** or **read-only** regions — the
//! unsynchronized shared buffer is sound *because the language forbids the races*.
//!
//! ## Runtime-internal synchronization (BELOW the language, design 0012 §1.3 note)
//! The runtime's own structures are not language-visible state, so they carry their
//! own synchronization: the stack-bump pointer is an `AtomicU64` (a CAS-bumped
//! allocator, giving each concurrent frame a disjoint region), and the trace sink
//! and fault slot are **per-task, thread-local** — each task accumulates into its
//! own buffers, merged deterministically at the join (§6 per-task projection). This
//! is synchronization *beneath* Candor, not a surface the language programs.
//!
//! ## The fault-exit hook, per-thread (design 0010 §3, extended for Stage 2)
//! Every MIR fault edge lowers to `call rt_fault(kind, span_start, span_end)`. The
//! hook records `(k, s)` into the **current thread's** fault slot and `_longjmp`s
//! to the **current thread's** landing pad — the main thread's pad is the driver's
//! (`run_guarded`), a task thread's is `run_task`'s. A cross-thread `_longjmp`
//! would be undefined, so each task catches its own fault locally and reports it as
//! an outcome; the join then re-delivers the spawn-order-first fault (§3.2) on the
//! parent thread.

use std::cell::RefCell;
use std::os::raw::{c_int, c_void};
use std::sync::atomic::{AtomicPtr, AtomicU64, Ordering};
use std::thread::JoinHandle;

use crate::interp::mem::MAX_ADDR;

extern "C" {
    // glibc/musl: `_setjmp`/`_longjmp` save/restore the machine context without
    // touching the signal mask (faster, and sufficient — no signals cross here).
    fn _setjmp(env: *mut c_void) -> c_int;
    fn _longjmp(env: *mut c_void, val: c_int) -> !;
}

/// The host runtime: the flat memory buffer + its base, the (atomic) bump pointers,
/// the final `θ` trace, the delivered fault, and the main thread's landing buffer.
/// During parallel execution the only *shared, mutable* fields are `stack_bump`
/// (atomic) and `buf` (DRF-disjoint accesses); `trace`/`fault` are written once, at
/// the end of the run, from the flushed thread-local of whichever thread finished.
pub struct Runtime {
    #[allow(dead_code)]
    buf: Vec<u8>,
    pub base: *mut u8,
    pub stack_bump: AtomicU64,
    pub static_bump: u64,
    pub trace: Vec<i64>,
    /// The delivered fault `(kind, span.start, span.end)` (`None` == ran to return).
    pub fault: Option<(u32, usize, usize)>,
    /// The main thread's `_setjmp`/`_longjmp` buffer (glibc `jmp_buf` ~200 bytes).
    jmp: [u64; 64],
}

impl Runtime {
    pub fn new() -> Box<Runtime> {
        let mut buf = vec![0u8; MAX_ADDR as usize];
        let base = buf.as_mut_ptr();
        Box::new(Runtime {
            buf,
            base,
            stack_bump: AtomicU64::new(crate::interp::mem::STACK_BASE),
            static_bump: crate::interp::mem::STATIC_BASE,
            trace: Vec::new(),
            fault: None,
            jmp: [0u64; 64],
        })
    }

    /// Reserve `size` bytes of static storage at `align` (driver-side, pre-`main`,
    /// single-threaded; the identical arithmetic bakes the `StaticAddr`/`StrAddr`
    /// constants).
    pub fn static_alloc(&mut self, size: u64, align: u64) -> u64 {
        let a = round_up(self.static_bump, align.max(1));
        self.static_bump = a + size;
        a
    }

    /// Write raw bytes at a Candor address (driver-side: string bytes, statics).
    pub fn write_bytes(&mut self, addr: u64, data: &[u8]) {
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), self.base.add(addr as usize), data.len());
        }
    }
}

// The current runtime, read by the shim symbols the JIT calls. The whole-program
// run is serialized by `RUN_LOCK` (backend::run), but WITHIN a run many task
// threads read this pointer concurrently — it is set (SeqCst) before any task is
// spawned, so every task observes it (thread-creation happens-before).
static CURRENT: AtomicPtr<Runtime> = AtomicPtr::new(std::ptr::null_mut());

#[inline]
fn rt() -> &'static Runtime {
    unsafe { &*CURRENT.load(Ordering::SeqCst) }
}

pub fn set_current(rt: *mut Runtime) {
    CURRENT.store(rt, Ordering::SeqCst);
}

pub fn clear_current() {
    CURRENT.store(std::ptr::null_mut(), Ordering::SeqCst);
}

// ---------------------------------------------------------------------------
// Per-task (thread-local) runtime state (design 0012 §6): the fault landing pad,
// the caught fault, this task's trace buffer, and the stack of open scope frames
// (each a list of child task handles, joined in spawn order at the closing brace).
// ---------------------------------------------------------------------------

/// What a joined task hands back to its parent: its caught fault (if any) and the
/// task's own trace buffer (merged into the parent's in spawn order).
struct TaskOutcome {
    fault: Option<(u32, usize, usize)>,
    trace: Vec<i64>,
}

#[derive(Default)]
struct Tls {
    /// This thread's active `_setjmp` landing (main: the runtime's `jmp`; a task:
    /// `run_task`'s local buffer). `rt_fault`/`rt_scope_end` `_longjmp` here.
    land: *mut c_void,
    /// The fault caught on this thread (by `rt_fault`), read after landing.
    fault: Option<(u32, usize, usize)>,
    /// This thread's `θ` fragment (per-task projection); merged at each join.
    trace: Vec<i64>,
    /// The stack of open scope frames on this thread (nested `scope`s).
    scopes: Vec<Vec<JoinHandle<TaskOutcome>>>,
}

thread_local! {
    static TLS: RefCell<Tls> = RefCell::new(Tls::default());
}

// ---------------------------------------------------------------------------
// Shim symbols the compiled code calls (registered with the JIT by name).
// ---------------------------------------------------------------------------

/// Reserve + zero a stack slot; returns its Candor address. A **CAS-bumped atomic**
/// so concurrent task threads each get a disjoint region (runtime-internal
/// synchronization, design 0012 §1.3 note). The bump never rolls back (as in the
/// single-threaded engine), so disjointness of live frames holds by construction.
pub extern "C" fn rt_stack_alloc(size: u64, align: u64) -> u64 {
    let r = rt();
    let align = align.max(1);
    loop {
        let cur = r.stack_bump.load(Ordering::Relaxed);
        let a = round_up(cur, align);
        let next = a + size;
        if r
            .stack_bump
            .compare_exchange_weak(cur, next, Ordering::SeqCst, Ordering::Relaxed)
            .is_ok()
        {
            if size != 0 {
                unsafe { std::ptr::write_bytes(r.base.add(a as usize), 0, size as usize) };
            }
            return a;
        }
    }
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

/// The observable `trace(x)` hook (INV-OBS-ORDER): append `x` to the **current
/// task's** trace buffer (per-task projection, design 0012 §6). The join merges
/// buffers in spawn order, so the resulting `θ` is schedule-independent.
pub extern "C" fn rt_trace(v: i64) {
    TLS.with(|t| t.borrow_mut().trace.push(v));
}

/// The observable rawptr/MMIO **load** hook (INV-OBS-ORDER, design 0010 §1/§2 F1).
pub extern "C" fn rt_mmio_load(addr: u64, size: u64) -> i64 {
    let r = rt();
    let mut buf = [0u8; 8];
    let n = (size as usize).min(8);
    unsafe {
        std::ptr::copy_nonoverlapping(r.base.add(addr as usize), buf.as_mut_ptr(), n);
    }
    i64::from_le_bytes(buf)
}

/// The observable rawptr/MMIO **store** hook (INV-OBS-ORDER): a barrier call.
pub extern "C" fn rt_mmio_store(addr: u64, val: i64, size: u64) {
    let r = rt();
    let bytes = val.to_le_bytes();
    let n = (size as usize).min(8);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), r.base.add(addr as usize), n);
    }
}

/// The fault-exit hook: record `(k, s)` in THIS thread's fault slot and `_longjmp`
/// to THIS thread's landing pad (never a cross-thread jump). Never returns.
pub extern "C" fn rt_fault(kind: u32, span_start: u32, span_end: u32) {
    let land = TLS.with(|t| {
        let mut t = t.borrow_mut();
        t.fault = Some((kind, span_start as usize, span_end as usize));
        t.land
    });
    unsafe {
        _longjmp(land, 1);
    }
}

// ---------------------------------------------------------------------------
// Structured-concurrency hooks (design 0012 §1.1/§3.4, Stage 2).
// ---------------------------------------------------------------------------

/// The opening `{` of a `scope`: push a fresh frame onto this thread's scope stack.
pub extern "C" fn rt_scope_begin() {
    TLS.with(|t| t.borrow_mut().scopes.push(Vec::new()));
}

/// The task-thread body: establish this thread's own fault landing pad, run the
/// task fn with its marshalled args, catch any fault locally, and hand back the
/// `(fault, trace)` outcome (never `_longjmp`ing across the thread boundary).
#[inline(never)]
fn run_task(faddr: usize, argc: usize, args: [i64; MAX_SPAWN_ARGS]) -> TaskOutcome {
    let mut jmp = [0u64; 64];
    let land = jmp.as_mut_ptr() as *mut c_void;
    TLS.with(|t| t.borrow_mut().land = land);
    let landed = unsafe { _setjmp(land) };
    let landed = std::hint::black_box(landed);
    if landed == 0 {
        call_task(faddr, argc, &args);
    }
    // Read the outcome from the thread-local (stable across the `_longjmp`), taking
    // this task's trace and caught fault. Nested-scope children of this task have
    // already merged their traces into `trace` at their `rt_scope_end`.
    TLS.with(|t| {
        let mut t = t.borrow_mut();
        TaskOutcome { fault: t.fault.take(), trace: std::mem::take(&mut t.trace) }
    })
}

/// Dispatch a compiled task fn (`extern "C" fn(i64, ...) -> i64`) by arity. Every
/// Candor arg — scalar or a pointer to caller-owned aggregate storage — is a single
/// i64 in the backend ABI, so arity alone selects the signature.
fn call_task(faddr: usize, argc: usize, a: &[i64; MAX_SPAWN_ARGS]) {
    let p = faddr as *const u8;
    unsafe {
        match argc {
            0 => (std::mem::transmute::<*const u8, extern "C" fn() -> i64>(p))(),
            1 => (std::mem::transmute::<*const u8, extern "C" fn(i64) -> i64>(p))(a[0]),
            2 => (std::mem::transmute::<*const u8, extern "C" fn(i64, i64) -> i64>(p))(a[0], a[1]),
            3 => (std::mem::transmute::<*const u8, extern "C" fn(i64, i64, i64) -> i64>(p))(a[0], a[1], a[2]),
            4 => (std::mem::transmute::<*const u8, extern "C" fn(i64, i64, i64, i64) -> i64>(p))(a[0], a[1], a[2], a[3]),
            5 => (std::mem::transmute::<*const u8, extern "C" fn(i64, i64, i64, i64, i64) -> i64>(p))(a[0], a[1], a[2], a[3], a[4]),
            6 => (std::mem::transmute::<*const u8, extern "C" fn(i64, i64, i64, i64, i64, i64) -> i64>(p))(a[0], a[1], a[2], a[3], a[4], a[5]),
            _ => panic!("rt_spawn: task arity {argc} exceeds MAX_SPAWN_ARGS"),
        };
    }
}

/// The number of fixed i64 arg slots `rt_spawn` receives (mirrors `lower`'s
/// `MAX_SPAWN_ARGS`).
pub const MAX_SPAWN_ARGS: usize = 6;

/// `spawn CALLEE(args)`: create a **real OS thread** running the task fn at `faddr`
/// with `argc` marshalled i64 args, and record its handle in the innermost open
/// scope frame (joined at the closing brace, in spawn order).
#[allow(clippy::too_many_arguments)]
pub extern "C" fn rt_spawn(
    faddr: i64,
    argc: i64,
    a0: i64,
    a1: i64,
    a2: i64,
    a3: i64,
    a4: i64,
    a5: i64,
) {
    let args = [a0, a1, a2, a3, a4, a5];
    let argc = argc as usize;
    let faddr = faddr as usize;
    // A generous per-task host stack so native (Cranelift) recursion inside a task
    // matches the interpreter's reach; the Candor "stack" itself lives in the flat
    // buffer via `rt_stack_alloc`, so tasks touch little host stack in practice.
    let handle = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(move || run_task(faddr, argc, args))
        .expect("rt_spawn: could not create task thread");
    TLS.with(|t| {
        t.borrow_mut()
            .scopes
            .last_mut()
            .expect("rt_spawn outside a scope (checker E1201 should forbid)")
            .push(handle);
    });
}

/// The closing `}` / join barrier: join every task of the innermost scope frame in
/// **spawn order**, merge their per-task traces into this thread's trace (§6), and
/// deliver the **spawn-order-first** fault (§3.2) — recorded in this thread's fault
/// slot and `_longjmp`d to this thread's landing pad — if any task faulted.
pub extern "C" fn rt_scope_end() {
    let handles = TLS.with(|t| {
        t.borrow_mut().scopes.pop().expect("rt_scope_end without a matching rt_scope_begin")
    });
    let mut outcomes: Vec<TaskOutcome> = Vec::with_capacity(handles.len());
    for h in handles {
        // Join in spawn order; a task thread never panics on a well-formed program.
        outcomes.push(h.join().expect("task thread panicked"));
    }
    // Merge every task's trace in spawn order (deterministic θ, regardless of fault
    // extent), then select the spawn-order-first fault.
    let first_fault = TLS.with(|t| {
        let mut t = t.borrow_mut();
        let mut first = None;
        for o in &outcomes {
            t.trace.extend_from_slice(&o.trace);
            if first.is_none() {
                first = o.fault;
            }
        }
        first
    });
    // Drop the joined outcomes before any non-local exit so the `_longjmp` path
    // leaks nothing.
    drop(outcomes);
    if let Some(f) = first_fault {
        let land = TLS.with(|t| {
            let mut t = t.borrow_mut();
            t.fault = Some(f);
            t.land
        });
        unsafe {
            _longjmp(land, 1);
        }
    }
}

/// Establish the main thread's `_setjmp` landing pad and run `body` (the static
/// inits + `main`). Returns `true` if `body` ran to completion, `false` on a fault
/// `_longjmp`. On return (either path) the thread-local trace/fault are flushed into
/// the runtime for the driver to read — the main thread's trace already carries
/// every joined task's merged trace fragments (design 0012 §6).
#[inline(never)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn run_guarded(rt_ptr: *mut Runtime, body: impl FnOnce()) -> bool {
    let env = unsafe { (*rt_ptr).jmp.as_mut_ptr() as *mut c_void };
    TLS.with(|t| t.borrow_mut().land = env);
    let landed = unsafe { _setjmp(env) };
    let landed = std::hint::black_box(landed);
    let completed = landed == 0;
    if completed {
        body();
    }
    // Flush this (main) thread's per-task trace/fault into the runtime result.
    TLS.with(|t| {
        let mut t = t.borrow_mut();
        unsafe {
            (*rt_ptr).trace = std::mem::take(&mut t.trace);
            if let Some(f) = t.fault.take() {
                (*rt_ptr).fault = Some(f);
            }
        }
    });
    completed
}

pub fn round_up(x: u64, align: u64) -> u64 {
    let a = align.max(1);
    x.div_ceil(a) * a
}
