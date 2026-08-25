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
    /// Spawned-but-not-yet-joined task count (`rt_spawn` increments BEFORE the
    /// thread exists; the joins — `rt_scope_end`, or `quiesce_open_scopes` on a
    /// fault path — decrement AFTER their join returns). When it
    /// reads 0 exactly one thread is running, which gates every stack-bump
    /// rollback (`rt_stack_restore`): restoring while tasks share the atomic
    /// bump could reclaim a concurrent task's live frame.
    pub live_tasks: AtomicU64,
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
            live_tasks: AtomicU64::new(0),
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
    /// The stack of open scope frames on this thread (nested `scope`s): the
    /// stack-bump mark at the scope's `{` plus the spawned task handles.
    scopes: Vec<ScopeFrame>,
}

/// One open `scope` on a thread: the bump mark recorded at `rt_scope_begin`
/// (restored at the join iff no task remains live — memo option B riding along
/// with option C, so a loop of scopes stays flat without waiting for the
/// enclosing function to return) and the child task handles, joined in spawn
/// order at the closing brace.
struct ScopeFrame {
    mark: u64,
    tasks: Vec<JoinHandle<TaskOutcome>>,
}

thread_local! {
    static TLS: RefCell<Tls> = RefCell::new(Tls::default());
}

// ---------------------------------------------------------------------------
// Shim symbols the compiled code calls (registered with the JIT by name).
// ---------------------------------------------------------------------------

/// Reserve + zero a stack slot; returns its Candor address. A **CAS-bumped atomic**
/// so concurrent task threads each get a disjoint region (runtime-internal
/// synchronization, design 0012 §1.3 note). The bump ROLLS BACK per call frame
/// via `rt_stack_save`/`rt_stack_restore` (task #144 rollback), gated on the
/// live-task counter — see `rt_stack_restore` for the full scheme; while tasks
/// are live no restore runs, so disjointness of concurrent live frames holds
/// exactly as before.
///
/// Exhaustion guard: zeroing `[a, a+size)` past `MAX_ADDR` would write outside
/// the host buffer (UB — heap corruption or a segfault). The interpreters report
/// the first touch past the model as `BadPointer` at span 0 in the MIR engine
/// (`Mem::ensure`; the tree-walk oracle carries a real span -- exhaustion is a
/// native-only pin, so no differential comparison exists today), so
/// deliver that same clean fault here instead. A `size == 0` reservation writes
/// nothing and stays fault-free, exactly like the interpreters' lazy check.
pub extern "C" fn rt_stack_alloc(size: u64, align: u64) -> u64 {
    let r = rt();
    let align = align.max(1);
    loop {
        let cur = r.stack_bump.load(Ordering::Relaxed);
        let a = round_up(cur, align);
        let next = a + size;
        if size != 0 && next > MAX_ADDR {
            let code = super::lower::kind_code(crate::interp::FaultKind::BadPointer);
            rt_fault(code, 0, 0); // never returns (`_longjmp` to this thread's pad)
        }
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

/// Read the current stack bump — the frame mark an allocating function saves at
/// entry, BEFORE its first `rt_stack_alloc`. The LLVM lowering emits the
/// save/restore pair only for functions with model-stack locals (Tier-F-free
/// functions emit neither and cost nothing); the Cranelift lowering emits the
/// pair unconditionally, which is equivalent there because every local gets a
/// flat model slot.
pub extern "C" fn rt_stack_save() -> u64 {
    rt().stack_bump.load(Ordering::SeqCst)
}

/// Roll the stack bump back to `mark` (the callee's entry mark) — the model
/// stack's "pop", run at every function return (task #144, memo option C).
///
/// ## Gate: only when NO task is live
/// `live_tasks == 0` means every spawned task has been joined, so exactly one
/// thread is running — the check-then-store cannot race (only a running thread
/// can spawn, and `rt_spawn` increments the counter before its thread exists).
/// Everything above `mark` is then a returned frame: dead by the interpreters'
/// watermark argument (returns are copied down, borrows cannot be returned).
/// While tasks ARE live every restore is skipped — the old leak-until-idle
/// behavior, whose cross-task disjointness proof is untouched. The enclosing
/// scope's join (`rt_scope_end`) restores to its own mark once the counter
/// returns to 0, reclaiming the tasks' dead frames.
///
/// ## The parked return value
/// CALLEE-side placement makes the aggregate-return hand-off safe by the same
/// invariant the reclaiming MIR interpreter relies on: the callee restores to
/// its entry mark and returns the address of its `_0` slot, which now sits
/// ABOVE the bump — stale but intact, exactly like the interpreter's popped
/// parked slot. The MIR lowering keeps a call's `Assign` and its consuming
/// `CopyVal` adjacent with no allocating statement between (tripwired by #141's
/// debug asserts in `mir::interp`), and `rt_copy` never allocates, so the bytes
/// are consumed before any allocation can clobber them. A fault path never
/// restores: `rt_fault` unwinds by `_longjmp`. A main-thread fault terminates
/// the program; a TASK fault is caught in `run_task` and the parent continues
/// -- the dead task frames are then reclaimed by the scope-join restore before
/// the fault is re-delivered, so a stale bump is unobservable either way.
pub extern "C" fn rt_stack_restore(mark: u64) {
    let r = rt();
    if r.live_tasks.load(Ordering::SeqCst) == 0 {
        r.stack_bump.store(mark, Ordering::SeqCst);
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
///
/// ## Orphan lifecycle on the fault path (ledger P6)
/// The `_longjmp` unwinds past every open `scope` on this thread, skipping their
/// `rt_scope_end` joins — so BEFORE jumping, every already-spawned task of every
/// open scope is joined (`quiesce_open_scopes`). Without this the orphaned task
/// threads outlive the run and dereference the cleared `CURRENT` pointer (an
/// abort), and the leaked `live_tasks` counts would disable stack-bump rollback.
/// Fault identity stays oracle-deterministic: in the sequential oracle a spawned
/// task runs to completion AT its spawn point, which is program-order-earlier
/// than this fault, so a quiesced task's fault (spawn-order-first) supersedes
/// this thread's own — the oracle's program-order-earliest fault: spec 06 §7.4(b) pins the single-threaded oracle's answer, and native is gated to match the oracle.
pub extern "C" fn rt_fault(kind: u32, span_start: u32, span_end: u32) {
    let own = (kind, span_start as usize, span_end as usize);
    let delivered = quiesce_open_scopes().unwrap_or(own);
    let land = TLS.with(|t| {
        let mut t = t.borrow_mut();
        t.fault = Some(delivered);
        t.land
    });
    unsafe {
        _longjmp(land, 1);
    }
}

/// Join every task of every still-open scope on THIS thread — the fault path's
/// half of the task lifecycle (a normal path joins at each `rt_scope_end`).
/// Returns the spawn-order-first fault among the joined tasks, if any faulted.
///
/// Open frames are strictly nested and spawns always land in the innermost open
/// frame, so iterating frames outermost-first visits tasks in spawn (= program)
/// order. Each task's trace is merged into this thread's buffer in that same
/// order (on a faulting run only fault identity is gated; trace extent is
/// declared-nondeterministic, design 0012 §3.2). The joins are unconditional:
/// The joins are unconditional. A non-terminating task hangs here; it also hangs the already-shipped rt_scope_end join, so this adds no new hang class. A blocking foreign call blocks the sequential oracle at its spawn point too. (A call-free spin loop is the known engine asymmetry: the oracle exhausts its model stack and faults where native spins -- pre-existing, ledgered.) No stack-bump
/// restore happens on this path ("a fault path never restores"): a main-thread
/// fault terminates the run, and a task-thread fault leaves the reclaim to its
/// parent's scope join.
fn quiesce_open_scopes() -> Option<(u32, usize, usize)> {
    let frames = TLS.with(|t| std::mem::take(&mut t.borrow_mut().scopes));
    if frames.iter().all(|f| f.tasks.is_empty()) {
        return None;
    }
    let r = rt();
    let mut first_fault = None;
    for frame in frames {
        for h in frame.tasks {
            let outcome = h.join().expect("task thread panicked");
            // The task's thread is gone; only now may its liveness count drop
            // (the rollback gate in `rt_stack_restore` relies on this ordering).
            r.live_tasks.fetch_sub(1, Ordering::SeqCst);
            TLS.with(|t| t.borrow_mut().trace.extend_from_slice(&outcome.trace));
            if first_fault.is_none() {
                first_fault = outcome.fault;
            }
        }
    }
    first_fault
}

// ---------------------------------------------------------------------------
// Structured-concurrency hooks (design 0012 §1.1/§3.4, Stage 2).
// ---------------------------------------------------------------------------

/// The opening `{` of a `scope`: push a fresh frame onto this thread's scope
/// stack, recording the current bump as the scope's rollback mark. Everything
/// allocated during the scope (parent calls — unrestored while tasks are live —
/// and every task's frames) is dead at the join, so the mark is restorable then.
pub extern "C" fn rt_scope_begin() {
    let mark = rt().stack_bump.load(Ordering::SeqCst);
    TLS.with(|t| t.borrow_mut().scopes.push(ScopeFrame { mark, tasks: Vec::new() }));
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
    // Count the task live BEFORE its thread can exist, so no thread ever runs
    // while an `rt_stack_restore` observes the counter at 0 (the rollback gate).
    rt().live_tasks.fetch_add(1, Ordering::SeqCst);
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
            .tasks
            .push(handle);
    });
}

/// The closing `}` / join barrier: join every task of the innermost scope frame in
/// **spawn order**, merge their per-task traces into this thread's trace (§6), and
/// deliver the **spawn-order-first** fault (§3.2) — recorded in this thread's fault
/// slot and `_longjmp`d to this thread's landing pad — if any task faulted.
pub extern "C" fn rt_scope_end() {
    let frame = TLS.with(|t| {
        t.borrow_mut().scopes.pop().expect("rt_scope_end without a matching rt_scope_begin")
    });
    let r = rt();
    let mut outcomes: Vec<TaskOutcome> = Vec::with_capacity(frame.tasks.len());
    for h in frame.tasks {
        // Join in spawn order; a task thread never panics on a well-formed program.
        outcomes.push(h.join().expect("task thread panicked"));
        // The task's thread is gone; only now may its liveness count drop (the
        // rollback gate in `rt_stack_restore` relies on this ordering).
        r.live_tasks.fetch_sub(1, Ordering::SeqCst);
    }
    // Scope-join rollback (memo option B): with every child joined, all frames
    // the scope's tasks (and the parent's unrestored calls while they were live)
    // allocated above the `rt_scope_begin` mark are dead — restore to it, unless
    // an OUTER scope's tasks are still live (then this thread may be a task
    // itself, or siblings share the bump, and the restore stays unsound).
    if r.live_tasks.load(Ordering::SeqCst) == 0 {
        r.stack_bump.store(frame.mark, Ordering::SeqCst);
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
        // The `_longjmp` below unwinds past ENCLOSING open scopes too, so their
        // tasks must be quiesced first (ledger P6). An outer frame's tasks were
        // spawned before this scope opened — program-order earlier — so an outer
        // task's fault supersedes this scope's (§7.4(b), matching the oracle,
        // which faults at the outer spawn point before this scope even begins).
        let f = quiesce_open_scopes().unwrap_or(f);
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
