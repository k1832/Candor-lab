//! Stage B — the first NATIVE backend (design 0010 §5): MIR -> Cranelift IR ->
//! JIT'd native code, no optimization, whole-program. Gated by the same trace-
//! equality discipline as Stage A: the compiled artifact's `(k, s, θ)` must equal
//! the MIR interpreter's (and transitively the tree-walking oracle's).
//!
//! `run` compiles the whole `MirProgram` in-process (cranelift-jit) and executes
//! it against the flat-memory `runtime` shim, returning the same `Run`/`Fault`
//! the interpreter does. The lowering lives in `lower`; this module owns the JIT
//! driver: it lays out statics/strings, allocates the fn-pointer dispatch table,
//! compiles, then runs the static initializers and `main` inside the fault-exit
//! landing pad (`runtime::run_guarded`).

pub mod llvm;
pub mod lower;
pub mod object;
pub mod runtime;

use std::collections::{HashMap, HashSet};

use crate::interp::layout::Layout;
use crate::interp::{Fault, Run};
use crate::mir::{MirProgram, Rvalue, StatementKind};
use crate::resolve::Items;
use crate::span::Span;

use runtime::Runtime;

// The native engine uses a process-global current-runtime pointer, one JIT, and a
// big-stack execution thread; serialize whole runs so parallel test threads (or
// callers) never race on that shared state.
static RUN_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Compile the whole program with Cranelift and run `main` natively, returning
/// its `(ret, θ)` or the delivered fault `f★` — the same contract as
/// `mir::interp::run`, so the Stage-B gate compares them directly.
pub fn run(prog: &MirProgram, items: &Items, consts: &HashMap<String, u64>, optimize: bool) -> Result<Run, Fault> {
    let _guard = RUN_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut rt = Runtime::new();
    let mem_base = rt.base as i64;
    let lay = Layout { items, consts };

    // Static + string layout: reserve deterministic Candor addresses (the compiler
    // bakes them as constants; the driver places the bytes here).
    let mut statics: HashMap<String, u64> = HashMap::new();
    for st in &prog.statics {
        let size = lay.size_of(&st.ty).max(1);
        let align = lay.align_of(&st.ty).max(1);
        let addr = rt.static_alloc(size, align);
        statics.insert(st.name.clone(), addr);
    }
    let mut strings: HashMap<String, u64> = HashMap::new();
    for s in collect_strings(prog) {
        let bytes = s.as_bytes().to_vec();
        let addr = rt.static_alloc(bytes.len().max(1) as u64, 1);
        rt.write_bytes(addr, &bytes);
        strings.insert(s, addr);
    }

    // The fn-pointer dispatch table (host array; filled by `compile` post-finalize).
    let table: &'static mut [u64] = vec![0u64; prog.fn_ptrs.len().max(1)].leak();
    let table_ptr = table.as_mut_ptr();

    let compiled = match lower::compile(prog, items, consts, mem_base, &statics, &strings, table_ptr, optimize) {
        Ok(c) => c,
        Err(e) => {
            // A genuinely unreachable construct for the JIT is reported, never
            // silently skipped (design 0010 §5).
            return Err(Fault::new(crate::interp::FaultKind::Panic, Span::point(0), format!("backend: {e}")));
        }
    };

    // The compiled code recurses on the *host* stack (the interpreter used heap
    // frames capped at MAX_DEPTH); run it on a generous 512 MiB stack so deep-but-
    // finite native recursion matches the interpreter's reach.
    let rt_ptr: *mut Runtime = &mut *rt;
    let rt_addr = rt_ptr as usize;
    let main_addr = compiled.main_ptr as usize;
    let inits: Vec<(u64, u64, bool, usize)> = compiled
        .static_inits
        .iter()
        .map(|(a, sz, w, p)| (*a, *sz, *w, *p as usize))
        .collect();
    let ret_ptr = Box::into_raw(Box::new(0i64));
    let ret_addr = ret_ptr as usize;
    let handle = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(move || {
            let rt_ptr = rt_addr as *mut Runtime;
            runtime::set_current(rt_ptr);
            let completed = runtime::run_guarded(rt_ptr, || {
                for (addr, size, wordy, p) in &inits {
                    let f: extern "C" fn() -> i64 =
                        unsafe { std::mem::transmute::<*const u8, _>(*p as *const u8) };
                    let out = f();
                    if *wordy {
                        let s = (*size).min(8) as usize;
                        unsafe { (*rt_ptr).write_bytes(*addr, &out.to_le_bytes()[..s]) };
                    } else {
                        runtime::rt_copy(*addr, out as u64, *size);
                    }
                }
                let mainf: extern "C" fn() -> i64 =
                    unsafe { std::mem::transmute::<*const u8, _>(main_addr as *const u8) };
                let r = mainf();
                unsafe { *(ret_addr as *mut i64) = r };
            });
            runtime::clear_current();
            completed
        })
        .expect("spawn JIT thread");
    let completed = handle.join().expect("JIT thread panicked");
    let ret = unsafe { *Box::from_raw(ret_ptr) };

    if !completed {
        if let Some((k, s, e)) = rt.fault {
            let kind = lower::code_kind(k);
            return Err(Fault::new(kind, Span::new(s, e), fault_msg(kind)));
        }
    }
    // `main` reports its 64-bit return word for `i64` or `f64` (the f64 word is
    // its IEEE bit pattern; design 0016).
    let ret_i64 = match prog.get("main").map(|f| &f.locals[0].ty) {
        Some(crate::types::Type::Scalar(
            crate::token::ScalarTy::I64 | crate::token::ScalarTy::F64,
        )) => ret,
        _ => 0,
    };
    Ok(Run { ret: ret_i64, trace: std::mem::take(&mut rt.trace) })
}

fn fault_msg(kind: crate::interp::FaultKind) -> &'static str {
    use crate::interp::FaultKind::*;
    match kind {
        Assert => "assertion failed",
        Panic => "panic",
        Overflow => "arithmetic overflow",
        ConvLoss => "conversion loses value",
        Bounds => "index out of bounds",
        DivByZero => "division by zero",
        Requires => "`requires` clause violated",
        Ensures => "`ensures` clause violated",
        BadPointer => "bad pointer",
        NoForeignRuntime => "no foreign runtime (native backend is a 0010 forward dependency)",
    }
}

/// Collect every string-literal (`StrAddr`) body appearing in the program.
fn collect_strings(prog: &MirProgram) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    let note = |s: &str, seen: &mut HashSet<String>, out: &mut Vec<String>| {
        if seen.insert(s.to_string()) {
            out.push(s.to_string());
        }
    };
    for f in &prog.fns {
        for b in &f.blocks {
            for st in &b.stmts {
                let rv = match &st.kind {
                    StatementKind::Assign(_, rv) | StatementKind::Store(_, rv) => Some(rv),
                    _ => None,
                };
                if let Some(Rvalue::StrAddr(s)) = rv {
                    note(s, &mut seen, &mut out);
                }
            }
        }
    }
    out
}
