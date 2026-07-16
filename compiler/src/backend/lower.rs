//! Stage B lowering (design 0010 §1, §2, §5): MIR -> Cranelift IR -> native.
//!
//! One Cranelift function per `MirFn`, no optimization (`opt_level=none`), whole
//! program compiled up front. The lowering is the *only* backend-specific code;
//! it preserves every MIR invariant by construction:
//!
//! * **INV-CHECK.** Each checked arithmetic/conv/index op lowers to *compute +
//!   explicit range/overflow test + conditional branch to a fault edge* — the
//!   "natural" Cranelift lowering the design names (§1): the check is **data the
//!   backend sees**, never a fact it may assume away. Cranelift's `iadd` is
//!   defined 2's-complement wrapping and its light mid-end assumes no
//!   signed-overflow UB, so the INV-CHECK crux class is structurally absent.
//! * **INV-FAULT-ID.** Every fault edge lowers to `call rt_fault(k, s_start,
//!   s_end)` (immediate `(k, s)`, no PC-keyed side-table) followed by an
//!   unreachable `trap`. Stage A is precise (`ReplayPolicy::Precise`), so faults
//!   fire program-order-first with no reordering — the replay obligation is
//!   vacuous, held by *not reordering* fault-capable ops (no optimization runs).
//! * **INV-OBS-ORDER.** The one MIR-marked observable, `trace`, lowers to a call
//!   with a side effect, across which no Cranelift pass reorders; at `opt_level=
//!   none` nothing is reordered at all. (rawptr/MMIO are not yet MIR-marked
//!   observable; see the module docs on `mir` — the honest boundary.)
//!
//! ## Values and the flat model
//! Every scalar SSA value is the *canonical i64* of its Candor type
//! (sign-extended if signed, zero-extended if unsigned). Aggregates live in the
//! flat `Runtime` buffer at Candor addresses; the compiler bakes the buffer base
//! as a constant and a load/store is `base + candor_addr`. A compiled function
//! allocates each local's slot with `rt_stack_alloc` on entry and returns the
//! word value (wordy return type) or the return place's address (aggregate).

#![allow(clippy::too_many_arguments)]

use std::collections::HashMap;

use cranelift_codegen::ir::condcodes::{FloatCC, IntCC};
use cranelift_codegen::ir::{
    types, AbiParam, Block, BlockArg, FuncRef, InstBuilder, MemFlags, Signature, TrapCode,
    UserFuncName, Value,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, DataId, FuncId, Linkage, Module};

use crate::interp::layout::Layout;
use crate::interp::FaultKind;
use crate::ast::{BinOp, UnOp};
use crate::mir::{
    CollOp, FaultEdge, MirFn, MirProgram, Operand, Place, Proj, Regime, Rvalue, Statement,
    StatementKind, Terminator,
};
use crate::resolve::Items;
use crate::span::Span;
use crate::token::ScalarTy;
use crate::types::{ItemEnv, Type};

use super::runtime;

/// The fixed number of marshalled i64 argument slots a `spawn` passes to
/// `rt_spawn` (design 0012 Stage 2). Task fns cross scalar/pointer args (each a
/// single i64); six slots covers the parallel-checker/fill shapes with headroom.
pub(super) const MAX_SPAWN_ARGS: usize = 6;

/// FaultKind -> stable numeric code passed to `rt_fault` and decoded by the driver.
pub fn kind_code(k: FaultKind) -> u32 {
    match k {
        FaultKind::Overflow => 0,
        FaultKind::DivByZero => 1,
        FaultKind::Bounds => 2,
        FaultKind::ConvLoss => 3,
        FaultKind::Assert => 4,
        FaultKind::Requires => 5,
        FaultKind::Ensures => 6,
        FaultKind::Panic => 7,
        FaultKind::BadPointer => 8,
        // The AOT backend lowers foreign calls to real libc (0011 §5), so it never
        // raises this; it is the interpreter engines' unregistered-shim fault and
        // the code exists here only for match totality.
        FaultKind::NoForeignRuntime => 9,
    }
}

pub fn code_kind(c: u32) -> FaultKind {
    match c {
        0 => FaultKind::Overflow,
        1 => FaultKind::DivByZero,
        2 => FaultKind::Bounds,
        3 => FaultKind::ConvLoss,
        4 => FaultKind::Assert,
        5 => FaultKind::Requires,
        6 => FaultKind::Ensures,
        9 => FaultKind::NoForeignRuntime,
        7 => FaultKind::Panic,
        _ => FaultKind::BadPointer,
    }
}

/// (min, max, bits, signed) for a scalar type — the same ranges the interpreter
/// (`mir::interp`) and the oracle use.
fn ty_range(sty: ScalarTy) -> (i128, i128, u32, bool) {
    let (bits, signed): (u32, bool) = match sty {
        ScalarTy::I8 => (8, true),
        ScalarTy::I16 => (16, true),
        ScalarTy::I32 => (32, true),
        ScalarTy::I64 | ScalarTy::Isize => (64, true),
        ScalarTy::Bool | ScalarTy::U8 => (8, false),
        ScalarTy::U16 => (16, false),
        ScalarTy::U32 => (32, false),
        ScalarTy::U64 | ScalarTy::Usize => (64, false),
        // A float bit pattern is unsigned so it is never sign-extended (design 0016).
        ScalarTy::F64 => (64, false),
        ScalarTy::F32 => (32, false),
        _ => (64, true),
    };
    let (min, max) = if signed {
        (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1)
    } else {
        (0, (1i128 << bits) - 1)
    };
    (min, max, bits, signed)
}

fn is_wordy(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Scalar(_) | Type::Borrow(_) | Type::BorrowMut(_) | Type::RawPtr(_) | Type::FnPtr(_)
    )
}

fn scalar_of(ty: &Type) -> ScalarTy {
    match ty {
        Type::Scalar(s) => *s,
        _ => ScalarTy::U64,
    }
}

fn int_ty(size: u64) -> types::Type {
    match size {
        1 => types::I8,
        2 => types::I16,
        4 => types::I32,
        _ => types::I64,
    }
}

/// The C symbol a boundary `extern` binds to (design 0011 §5, AOT path). The
/// std/io boundary names its externs `sys_*` because `read`/`write` are Candor
/// keywords (the borrow modes); the C symbol is the declared name with that
/// `sys_` prefix stripped (identity for any other extern). This is the deliberate
/// extern-name -> C-symbol convention the edition uses in lieu of a `symbol`
/// attribute: `sys_read`->`read`, `sys_write`->`write`, `sys_open`->`open`,
/// `sys_close`->`close`.
pub(super) fn c_symbol_name(extern_name: &str) -> &str {
    extern_name.strip_prefix("sys_").unwrap_or(extern_name)
}

/// The C-ABI Cranelift type for a boundary parameter/return: a pointer word is
/// `I64`; a scalar is passed in its natural register width (<=32-bit as `I32`,
/// else `I64`), matching the SysV C ABI for the fixed integer/pointer args a
/// POSIX extern takes.
fn c_abi_ty(ty: &Type) -> types::Type {
    match ty {
        Type::RawPtr(_) | Type::FnPtr(_) | Type::Borrow(_) | Type::BorrowMut(_) => types::I64,
        Type::Scalar(s) => {
            let (_, _, bits, _) = ty_range(*s);
            if bits <= 32 { types::I32 } else { types::I64 }
        }
        _ => types::I64,
    }
}

/// A compiled program: the JIT module (kept alive so its code stays mapped) plus
/// the finalized entry pointers the driver invokes.
pub struct Compiled {
    _module: JITModule,
    pub main_ptr: *const u8,
    /// (static addr, byte size, wordy, init fn ptr) in program order.
    pub static_inits: Vec<(u64, u64, bool, *const u8)>,
}

/// The declared user-function and drop-glue FuncId maps, keyed by MIR name.
type FuncIdMaps = (HashMap<String, FuncId>, HashMap<String, FuncId>);

/// Where indirect calls read their function-pointer dispatch table from.
///   * `Host` — a leaked host array whose address is baked as a constant (JIT).
///   * `Data` — an emitted data object filled with function-address relocations,
///     addressed with `symbol_value` (AOT object).
#[derive(Clone, Copy)]
pub(super) enum FnTable {
    Host(i64),
    Data(DataId),
}

/// Declare every user function and drop-glue up front, returning their FuncId
/// maps (keyed by MIR name). `prefix` is prepended to the emitted **symbol
/// name** only (the AOT path prefixes so the Candor `main` never clashes with the
/// C runtime's `main`); the maps stay keyed by the original name, so call
/// resolution — which goes through the FuncId, not the symbol — is unchanged.
pub(super) fn declare_functions<M: Module>(
    module: &mut M,
    prog: &MirProgram,
    glue_types: &[Type],
    prefix: &str,
) -> Result<FuncIdMaps, String> {
    let mut func_ids: HashMap<String, FuncId> = HashMap::new();
    for f in &prog.fns {
        let mut sig = module.make_signature();
        for _ in 0..f.num_params {
            sig.params.push(AbiParam::new(types::I64));
        }
        sig.returns.push(AbiParam::new(types::I64));
        let id = module
            .declare_function(&format!("{prefix}{}", f.name), Linkage::Local, &sig)
            .map_err(|e| e.to_string())?;
        func_ids.insert(f.name.clone(), id);
    }
    // Drop glue (design 0010 §5, INV-DROP): a Box's pointee is dropped through a
    // synthesized per-type glue function, so a recursive Box type's drop is
    // runtime recursion (terminating on `ptr == 0`), never infinite compile-time
    // unrolling.
    let mut glue_ids: HashMap<String, FuncId> = HashMap::new();
    for (i, ty) in glue_types.iter().enumerate() {
        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(types::I64));
        sig.returns.push(AbiParam::new(types::I64));
        let id = module
            .declare_function(&format!("{prefix}__drop_glue_{i}"), Linkage::Local, &sig)
            .map_err(|e| e.to_string())?;
        glue_ids.insert(type_key(ty), id);
    }
    Ok((func_ids, glue_ids))
}

/// Declare every boundary `extern` as an IMPORTED C symbol (design 0011 §5, the
/// AOT path): a foreign call resolves to a REAL libc symbol in the linked binary
/// (the hosted profile pulls libc; the freestanding profile forbids FFI). The map
/// is keyed by the extern's Candor name; the emitted/imported symbol name is
/// `c_symbol_name` and the signature is the C ABI of the extern's declared type.
pub(super) fn declare_externs<M: Module>(
    module: &mut M,
    items: &Items,
) -> Result<HashMap<String, FuncId>, String> {
    let mut ids: HashMap<String, FuncId> = HashMap::new();
    for (name, es) in &items.externs {
        let mut sig = module.make_signature();
        for p in &es.params {
            sig.params.push(AbiParam::new(c_abi_ty(&p.lowered)));
        }
        if !matches!(es.ret, Type::Scalar(ScalarTy::Unit)) {
            sig.returns.push(AbiParam::new(c_abi_ty(&es.ret)));
        }
        let id = module
            .declare_function(c_symbol_name(name), Linkage::Import, &sig)
            .map_err(|e| e.to_string())?;
        ids.insert(name.clone(), id);
    }
    Ok(ids)
}

/// Define every user function + drop-glue body — the shared MIR->Cranelift-IR
/// lowering, generic over the backend `Module` (JIT or object). Identical IR is
/// built either way; only `mem_base`/`fntable` differ (the module-plumbing delta,
/// design 0010 §1's "the lowering is the only backend-specific code").
#[allow(clippy::too_many_arguments)]
pub(super) fn define_functions<M: Module>(
    module: &mut M,
    prog: &MirProgram,
    items: &Items,
    consts: &HashMap<String, u64>,
    mem_base: i64,
    statics: &HashMap<String, u64>,
    strings: &HashMap<String, u64>,
    fntable: FnTable,
    shims: &Shims,
    func_ids: &HashMap<String, FuncId>,
    glue_ids: &HashMap<String, FuncId>,
    extern_ids: &HashMap<String, FuncId>,
    glue_types: &[Type],
) -> Result<(), String> {
    let mut ctx = module.make_context();
    let mut fctx = FunctionBuilderContext::new();

    for f in &prog.fns {
        let fid = func_ids[&f.name];
        ctx.func.signature = {
            let mut sig = Signature::new(module.isa().default_call_conv());
            for _ in 0..f.num_params {
                sig.params.push(AbiParam::new(types::I64));
            }
            sig.returns.push(AbiParam::new(types::I64));
            sig
        };
        ctx.func.name = UserFuncName::user(0, fid.as_u32());
        {
            let mut b = FunctionBuilder::new(&mut ctx.func, &mut fctx);
            let mut cg = Cg {
                b: &mut b,
                module,
                prog,
                items,
                consts,
                mem_base,
                statics,
                strings,
                fntable,
                shims,
                func_ids,
                glue_ids,
                extern_ids,
                callrefs: HashMap::new(),
                shimrefs: HashMap::new(),
                externrefs: HashMap::new(),
                addr: Vec::new(),
            };
            cg.lower_fn(f);
            b.seal_all_blocks();
            b.finalize();
        }
        module.define_function(fid, &mut ctx).map_err(|e| e.to_string())?;
        module.clear_context(&mut ctx);
    }

    for ty in glue_types {
        let fid = glue_ids[&type_key(ty)];
        ctx.func.signature = {
            let mut sig = Signature::new(module.isa().default_call_conv());
            sig.params.push(AbiParam::new(types::I64));
            sig.returns.push(AbiParam::new(types::I64));
            sig
        };
        ctx.func.name = UserFuncName::user(0, fid.as_u32());
        {
            let mut b = FunctionBuilder::new(&mut ctx.func, &mut fctx);
            let mut cg = Cg {
                b: &mut b,
                module,
                prog,
                items,
                consts,
                mem_base,
                statics,
                strings,
                fntable,
                shims,
                func_ids,
                glue_ids,
                extern_ids,
                callrefs: HashMap::new(),
                shimrefs: HashMap::new(),
                externrefs: HashMap::new(),
                addr: Vec::new(),
            };
            cg.lower_glue(ty);
            b.seal_all_blocks();
            b.finalize();
        }
        module.define_function(fid, &mut ctx).map_err(|e| e.to_string())?;
        module.clear_context(&mut ctx);
    }
    Ok(())
}

/// Define the AOT entry wrapper `candor_entry() -> i64` (object path only). It
/// runs the startup work the JIT driver does host-side: copy string-literal bytes
/// into the flat buffer at their baked Candor addresses, run each static
/// initializer and write its result to the static's address, then call `main` and
/// return its `i64` (or `0` for a non-`i64` `main`, mirroring `backend::run`). The
/// runtime library establishes the `setjmp` landing pad and calls this.
#[allow(clippy::too_many_arguments)]
pub(super) fn define_entry<M: Module>(
    module: &mut M,
    prog: &MirProgram,
    items: &Items,
    consts: &HashMap<String, u64>,
    mem_base: i64,
    statics: &HashMap<String, u64>,
    strings: &HashMap<String, u64>,
    fntable: FnTable,
    shims: &Shims,
    func_ids: &HashMap<String, FuncId>,
    glue_ids: &HashMap<String, FuncId>,
    extern_ids: &HashMap<String, FuncId>,
    entry_id: FuncId,
) -> Result<(), String> {
    let mut ctx = module.make_context();
    let mut fctx = FunctionBuilderContext::new();
    ctx.func.signature = {
        let mut sig = Signature::new(module.isa().default_call_conv());
        sig.returns.push(AbiParam::new(types::I64));
        sig
    };
    ctx.func.name = UserFuncName::user(0, entry_id.as_u32());
    // `main` reports its 64-bit return WORD when it returns `i64` or `f64` (the
    // f64 word is its IEEE bit pattern; design 0016).
    let main_is_i64 = matches!(
        prog.get("main").map(|f| &f.locals[0].ty),
        Some(Type::Scalar(ScalarTy::I64)) | Some(Type::Scalar(ScalarTy::F64))
    );
    {
        let mut b = FunctionBuilder::new(&mut ctx.func, &mut fctx);
        let mut cg = Cg {
            b: &mut b,
            module,
            prog,
            items,
            consts,
            mem_base,
            statics,
            strings,
            fntable,
            shims,
            func_ids,
            glue_ids,
            extern_ids,
            callrefs: HashMap::new(),
            shimrefs: HashMap::new(),
            externrefs: HashMap::new(),
            addr: Vec::new(),
        };
        cg.lower_entry(main_is_i64);
        b.seal_all_blocks();
        b.finalize();
    }
    module.define_function(entry_id, &mut ctx).map_err(|e| e.to_string())?;
    Ok(())
}

/// Compile the whole program with the in-process JIT. `mem_base` is the flat
/// buffer base; `statics`/`strings` are the pre-computed Candor addresses baked
/// into `StaticAddr`/`StrAddr`; `fnptr_table` is a host array (len =
/// `prog.fn_ptrs`) filled after finalization and read by indirect calls.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn compile(
    prog: &MirProgram,
    items: &Items,
    consts: &HashMap<String, u64>,
    mem_base: i64,
    statics: &HashMap<String, u64>,
    strings: &HashMap<String, u64>,
    fnptr_table: *mut u64,
    optimize: bool,
) -> Result<Compiled, String> {
    // Stage D: flip the native engine's optimizer on. `opt_level=speed` enables
    // Cranelift's egraph mid-end (constant folding, GVN, licm, DCE over `τ`-steps).
    // Its INV-CHECK safety is structural (§1: `iadd` is wrapping, no signed-overflow
    // UB, so it never deletes an overflow check as "dead"); INV-OBS-ORDER /
    // INV-FAULT-ID are secured by the lowering (observables + faults are barrier
    // CALLS the egraph will not reorder past — the F1 discipline made real).
    let mut builder = if optimize {
        JITBuilder::with_flags(&[("opt_level", "speed")], default_libcall_names())
            .map_err(|e| e.to_string())?
    } else {
        JITBuilder::new(default_libcall_names()).map_err(|e| e.to_string())?
    };
    builder.symbol("rt_stack_alloc", runtime::rt_stack_alloc as *const u8);
    builder.symbol("rt_copy", runtime::rt_copy as *const u8);
    builder.symbol("rt_trace", runtime::rt_trace as *const u8);
    builder.symbol("rt_mmio_load", runtime::rt_mmio_load as *const u8);
    builder.symbol("rt_mmio_store", runtime::rt_mmio_store as *const u8);
    builder.symbol("rt_fault", runtime::rt_fault as *const u8);
    builder.symbol("rt_scope_begin", runtime::rt_scope_begin as *const u8);
    builder.symbol("rt_spawn", runtime::rt_spawn as *const u8);
    builder.symbol("rt_scope_end", runtime::rt_scope_end as *const u8);
    let mut module = JITModule::new(builder);

    let shims = Shims::declare(&mut module)?;
    let glue_types = collect_glue_types(prog, items, consts);
    let (func_ids, glue_ids) = declare_functions(&mut module, prog, &glue_types, "")?;
    let extern_ids = declare_externs(&mut module, items)?;
    define_functions(
        &mut module,
        prog,
        items,
        consts,
        mem_base,
        statics,
        strings,
        FnTable::Host(fnptr_table as i64),
        &shims,
        &func_ids,
        &glue_ids,
        &extern_ids,
        &glue_types,
    )?;

    module.finalize_definitions().map_err(|e| e.to_string())?;

    // Fill the fn-pointer table with finalized addresses (indirect-call dispatch).
    for (i, name) in prog.fn_ptrs.iter().enumerate() {
        if let Some(id) = func_ids.get(name) {
            let p = module.get_finalized_function(*id);
            unsafe { *fnptr_table.add(i) = p as u64 };
        }
    }

    let main_ptr = module.get_finalized_function(func_ids["main"]);
    let lay = Layout { items, consts };
    let mut static_inits = Vec::new();
    for st in &prog.statics {
        let addr = statics[&st.name];
        let size = lay.size_of(&st.ty);
        let wordy = is_wordy(&st.ty);
        let p = module.get_finalized_function(func_ids[&st.init_fn]);
        static_inits.push((addr, size, wordy, p));
    }

    Ok(Compiled { _module: module, main_ptr, static_inits })
}
/// Imported shim FuncIds.
pub(super) struct Shims {
    stack_alloc: FuncId,
    copy: FuncId,
    trace: FuncId,
    mmio_load: FuncId,
    mmio_store: FuncId,
    fault: FuncId,
    // Structured-concurrency Stage 2 (design 0012): real thread creation + join.
    scope_begin: FuncId,
    spawn: FuncId,
    scope_end: FuncId,
}

impl Shims {
    pub(super) fn declare<M: Module>(module: &mut M) -> Result<Shims, String> {
        let mut sig1 = module.make_signature();
        sig1.params.push(AbiParam::new(types::I64));
        sig1.params.push(AbiParam::new(types::I64));
        sig1.returns.push(AbiParam::new(types::I64));
        let stack_alloc = module
            .declare_function("rt_stack_alloc", Linkage::Import, &sig1)
            .map_err(|e| e.to_string())?;

        let mut sig3 = module.make_signature();
        for _ in 0..3 {
            sig3.params.push(AbiParam::new(types::I64));
        }
        let copy = module
            .declare_function("rt_copy", Linkage::Import, &sig3)
            .map_err(|e| e.to_string())?;

        let mut sigt = module.make_signature();
        sigt.params.push(AbiParam::new(types::I64));
        let trace = module
            .declare_function("rt_trace", Linkage::Import, &sigt)
            .map_err(|e| e.to_string())?;

        // rt_mmio_load(addr: i64, size: i64) -> i64
        let mut sigml = module.make_signature();
        sigml.params.push(AbiParam::new(types::I64));
        sigml.params.push(AbiParam::new(types::I64));
        sigml.returns.push(AbiParam::new(types::I64));
        let mmio_load = module
            .declare_function("rt_mmio_load", Linkage::Import, &sigml)
            .map_err(|e| e.to_string())?;

        // rt_mmio_store(addr: i64, val: i64, size: i64)
        let mut sigms = module.make_signature();
        for _ in 0..3 {
            sigms.params.push(AbiParam::new(types::I64));
        }
        let mmio_store = module
            .declare_function("rt_mmio_store", Linkage::Import, &sigms)
            .map_err(|e| e.to_string())?;

        let mut sigf = module.make_signature();
        for _ in 0..3 {
            sigf.params.push(AbiParam::new(types::I32));
        }
        let fault = module
            .declare_function("rt_fault", Linkage::Import, &sigf)
            .map_err(|e| e.to_string())?;

        // rt_scope_begin() / rt_scope_end(): the join-barrier markers (no args).
        let sig0 = module.make_signature();
        let scope_begin = module
            .declare_function("rt_scope_begin", Linkage::Import, &sig0)
            .map_err(|e| e.to_string())?;
        let scope_end = module
            .declare_function("rt_scope_end", Linkage::Import, &sig0)
            .map_err(|e| e.to_string())?;

        // rt_spawn(faddr, argc, a0..a5): create a real OS thread running the task
        // fn at `faddr` with `argc` marshalled i64 args (padded to MAX_SPAWN_ARGS).
        let mut sigsp = module.make_signature();
        for _ in 0..(2 + MAX_SPAWN_ARGS) {
            sigsp.params.push(AbiParam::new(types::I64));
        }
        let spawn = module
            .declare_function("rt_spawn", Linkage::Import, &sigsp)
            .map_err(|e| e.to_string())?;

        Ok(Shims { stack_alloc, copy, trace, mmio_load, mmio_store, fault, scope_begin, spawn, scope_end })
    }
}

/// Per-function codegen context, generic over the backend `Module` (JIT or
/// object) so one lowering serves both.
struct Cg<'a, 'b, M: Module> {
    b: &'a mut FunctionBuilder<'b>,
    module: &'a mut M,
    prog: &'a MirProgram,
    items: &'a Items,
    consts: &'a HashMap<String, u64>,
    mem_base: i64,
    statics: &'a HashMap<String, u64>,
    strings: &'a HashMap<String, u64>,
    fntable: FnTable,
    shims: &'a Shims,
    func_ids: &'a HashMap<String, FuncId>,
    glue_ids: &'a HashMap<String, FuncId>,
    extern_ids: &'a HashMap<String, FuncId>,
    callrefs: HashMap<String, FuncRef>,
    shimrefs: HashMap<&'static str, FuncRef>,
    externrefs: HashMap<String, FuncRef>,
    /// Candor address (SSA value) of each local's slot.
    addr: Vec<Value>,
}

impl<M: Module> Cg<'_, '_, M> {
    fn lay(&self) -> Layout<'_> {
        Layout { items: self.items, consts: self.consts }
    }
    fn size_of(&self, ty: &Type) -> u64 {
        self.lay().size_of(ty)
    }
    fn align_of(&self, ty: &Type) -> u64 {
        self.lay().align_of(ty)
    }

    fn shimref(&mut self, key: &'static str, id: FuncId) -> FuncRef {
        if let Some(r) = self.shimrefs.get(key) {
            return *r;
        }
        let r = self.module.declare_func_in_func(id, self.b.func);
        self.shimrefs.insert(key, r);
        r
    }
    fn callref(&mut self, name: &str) -> FuncRef {
        if let Some(r) = self.callrefs.get(name) {
            return *r;
        }
        let id = self.func_ids[name];
        let r = self.module.declare_func_in_func(id, self.b.func);
        self.callrefs.insert(name.to_string(), r);
        r
    }
    fn externref(&mut self, name: &str) -> FuncRef {
        if let Some(r) = self.externrefs.get(name) {
            return *r;
        }
        let id = self.extern_ids[name];
        let r = self.module.declare_func_in_func(id, self.b.func);
        self.externrefs.insert(name.to_string(), r);
        r
    }

    /// Lower a foreign `extern "C"` call (design 0011 §5, AOT path). Marshal each
    /// argument per the C ABI: a `rawptr` argument is a flat-memory OFFSET, so it
    /// is translated to the REAL host address (`MEM_BASE + offset`) that libc
    /// needs; a scalar is narrowed to its ABI register width. The call targets the
    /// imported C symbol; the result is canonicalized back to the i64 word.
    fn lower_extern_call(&mut self, name: &str, args: &[Operand], mf: &MirFn) -> Value {
        let es = self.items.externs[name].clone();
        let mut vals: Vec<Value> = Vec::with_capacity(args.len());
        for (i, a) in args.iter().enumerate() {
            let v = self.eval_operand(a, mf);
            let marshalled = match es.params.get(i).map(|p| &p.lowered) {
                // rawptr / borrow: translate the Candor offset to a real pointer.
                Some(Type::RawPtr(_)) | Some(Type::FnPtr(_)) | Some(Type::Borrow(_))
                | Some(Type::BorrowMut(_)) => self.host_addr(v),
                // narrow a <=32-bit scalar to its C ABI width (e.g. `i32` fd).
                Some(t) if c_abi_ty(t) == types::I32 => self.b.ins().ireduce(types::I32, v),
                _ => v,
            };
            vals.push(marshalled);
        }
        let r = self.externref(name);
        let c = self.b.ins().call(r, &vals);
        let results = self.b.inst_results(c).to_vec();
        if results.is_empty() {
            // `void` return (unit extern): the MIR temp holds an ignored 0.
            return self.iconst(0);
        }
        let raw = results[0];
        // Canonicalize a sub-word C return (e.g. `i32` from `open`) to the i64 word.
        match &es.ret {
            Type::Scalar(s) => {
                let (_, _, bits, signed) = ty_range(*s);
                if bits <= 32 {
                    if signed {
                        self.b.ins().sextend(types::I64, raw)
                    } else {
                        self.b.ins().uextend(types::I64, raw)
                    }
                } else {
                    raw
                }
            }
            _ => raw,
        }
    }

    // ---- low-level helpers ----
    fn iconst(&mut self, v: i64) -> Value {
        self.b.ins().iconst(types::I64, v)
    }
    fn iconst128(&mut self, v: i128) -> Value {
        let lo = self.b.ins().iconst(types::I64, (v as u128 as u64) as i64);
        let hi = self.b.ins().iconst(types::I64, ((v as u128 >> 64) as u64) as i64);
        self.b.ins().iconcat(lo, hi)
    }
    fn host_addr(&mut self, candor: Value) -> Value {
        let base = self.iconst(self.mem_base);
        self.b.ins().iadd(base, candor)
    }
    /// The base address (SSA value) of the fn-pointer dispatch table: a baked
    /// host constant under the JIT, the address of the emitted data object (via a
    /// relocated `symbol_value`) under the AOT object backend.
    fn fntable_base(&mut self) -> Value {
        match self.fntable {
            FnTable::Host(p) => self.iconst(p),
            FnTable::Data(id) => {
                let gv = self.module.declare_data_in_func(id, self.b.func);
                self.b.ins().symbol_value(types::I64, gv)
            }
        }
    }
    fn canon(&mut self, v: Value, sty: ScalarTy) -> Value {
        let (_, _, bits, signed) = ty_range(sty);
        if bits >= 64 {
            return v;
        }
        let nt = int_ty((bits / 8) as u64);
        let r = self.b.ins().ireduce(nt, v);
        if signed {
            self.b.ins().sextend(types::I64, r)
        } else {
            self.b.ins().uextend(types::I64, r)
        }
    }
    fn ext128(&mut self, v: Value, sty: ScalarTy) -> Value {
        let (_, _, _, signed) = ty_range(sty);
        if signed {
            self.b.ins().sextend(types::I128, v)
        } else {
            self.b.ins().uextend(types::I128, v)
        }
    }
    fn fit128(&mut self, v: Value, sty: ScalarTy) -> Value {
        let t = self.b.ins().ireduce(types::I64, v);
        self.canon(t, sty)
    }
    /// Reinterpret an i64 register as a native float of width `sty` (design 0016).
    /// For `f32` the 32-bit pattern lives in the register's low half.
    fn as_float(&mut self, sty: ScalarTy, bits: Value) -> Value {
        if sty == ScalarTy::F32 {
            let w = self.b.ins().ireduce(types::I32, bits);
            self.b.ins().bitcast(types::F32, MemFlags::new(), w)
        } else {
            self.b.ins().bitcast(types::F64, MemFlags::new(), bits)
        }
    }
    /// Reinterpret a native float of width `sty` back to its (zero-extended) i64
    /// bit-pattern register value (design 0016).
    fn float_bits(&mut self, sty: ScalarTy, f: Value) -> Value {
        if sty == ScalarTy::F32 {
            let w = self.b.ins().bitcast(types::I32, MemFlags::new(), f);
            self.b.ins().uextend(types::I64, w)
        } else {
            self.b.ins().bitcast(types::I64, MemFlags::new(), f)
        }
    }

    fn load_scalar(&mut self, candor: Value, sty: ScalarTy) -> Value {
        let (_, _, bits, signed) = ty_range(sty);
        let size = crate::interp::layout::Layout::scalar_size(sty);
        if size == 0 {
            return self.iconst(0);
        }
        let ha = self.host_addr(candor);
        let lt = int_ty(size);
        let raw = self.b.ins().load(lt, MemFlags::new(), ha, 0);
        if bits >= 64 {
            raw
        } else if signed {
            self.b.ins().sextend(types::I64, raw)
        } else {
            self.b.ins().uextend(types::I64, raw)
        }
    }
    fn store_scalar(&mut self, candor: Value, val: Value, sty: ScalarTy) {
        let size = crate::interp::layout::Layout::scalar_size(sty);
        if size == 0 {
            return;
        }
        let ha = self.host_addr(candor);
        let st = int_ty(size);
        let v = if size >= 8 { val } else { self.b.ins().ireduce(st, val) };
        self.b.ins().store(MemFlags::new(), v, ha, 0);
    }
    fn call_copy(&mut self, dst: Value, src: Value, len: u64) {
        if len == 0 {
            return;
        }
        let r = self.shimref("copy", self.shims.copy);
        let l = self.iconst(len as i64);
        self.b.ins().call(r, &[dst, src, l]);
    }

    /// Observable rawptr/MMIO load (INV-OBS-ORDER): a barrier CALL, not an inline
    /// `load` — so Cranelift's egraph (`opt_level=speed`) neither reorders it past a
    /// fault/other observable nor eliminates it. Returns the zero-extended word.
    fn call_mmio_load(&mut self, addr: Value, size: u64) -> Value {
        let r = self.shimref("mmio_load", self.shims.mmio_load);
        let sz = self.iconst(size as i64);
        let c = self.b.ins().call(r, &[addr, sz]);
        self.b.inst_results(c)[0]
    }
    /// Observable rawptr/MMIO store (INV-OBS-ORDER): the barrier-CALL counterpart.
    fn call_mmio_store(&mut self, addr: Value, val: Value, size: u64) {
        let r = self.shimref("mmio_store", self.shims.mmio_store);
        let sz = self.iconst(size as i64);
        self.b.ins().call(r, &[addr, val, sz]);
    }
    fn stack_alloc(&mut self, size: u64, align: u64) -> Value {
        let r = self.shimref("stack_alloc", self.shims.stack_alloc);
        let s = self.iconst(size as i64);
        let a = self.iconst(align as i64);
        let c = self.b.ins().call(r, &[s, a]);
        self.b.inst_results(c)[0]
    }

    /// Emit the fault-exit hook + unreachable trap (INV-FAULT-ID).
    fn emit_fault(&mut self, edge_kind: FaultKind, span: Span) {
        let r = self.shimref("fault", self.shims.fault);
        let k = self.b.ins().iconst(types::I32, kind_code(edge_kind) as i64);
        let s = self.b.ins().iconst(types::I32, span.start as i64);
        let e = self.b.ins().iconst(types::I32, span.end as i64);
        self.b.ins().call(r, &[k, s, e]);
        self.b.ins().trap(TrapCode::user(1).unwrap());
    }

    /// Branch to a fresh fault block when `cond_true` (leaves the current block =
    /// the fall-through continuation).
    fn fault_if(&mut self, cond_true: Value, kind: FaultKind, span: Span) {
        let fault = self.b.create_block();
        let cont = self.b.create_block();
        self.b.ins().brif(cond_true, fault, &[], cont, &[]);
        self.b.switch_to_block(fault);
        self.emit_fault(kind, span);
        self.b.switch_to_block(cont);
    }

    // ---- operands / rvalues ----
    fn operand_sty(&self, op: &Operand, mf: &MirFn) -> ScalarTy {
        match op {
            Operand::Const(_, s) => *s,
            Operand::Local(id) => scalar_of(&mf.locals[*id].ty),
        }
    }
    fn eval_operand(&mut self, op: &Operand, mf: &MirFn) -> Value {
        match op {
            Operand::Const(v, _) => self.iconst(*v as i64),
            Operand::Local(id) => {
                let a = self.addr[*id];
                self.load_scalar(a, scalar_of(&mf.locals[*id].ty))
            }
        }
    }

    /// Resolve a place to its Candor address (faulting on OOB index, INV-CHECK).
    fn place_addr(&mut self, place: &Place, mf: &MirFn) -> (Value, Type) {
        let mut addr = self.addr[place.root];
        let mut ty = mf.locals[place.root].ty.clone();
        for p in &place.proj {
            match p {
                Proj::Field { offset, ty: fty } => {
                    let off = self.iconst(*offset as i64);
                    addr = self.b.ins().iadd(addr, off);
                    ty = fty.clone();
                }
                Proj::Deref { inner } => {
                    addr = self.load_scalar(addr, ScalarTy::U64);
                    ty = inner.clone();
                }
                Proj::Index { index, stride, len, span, slice } => {
                    let i = self.eval_operand(index, mf);
                    if *slice {
                        let base = self.load_scalar(addr, ScalarTy::U64);
                        let eight = self.iconst(8);
                        let lenaddr = self.b.ins().iadd(addr, eight);
                        let n = self.load_scalar(lenaddr, ScalarTy::U64);
                        let oob = self.b.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, i, n);
                        self.fault_if(oob, FaultKind::Bounds, *span);
                        let str_ = self.iconst(*stride as i64);
                        let off = self.b.ins().imul(i, str_);
                        addr = self.b.ins().iadd(base, off);
                    } else {
                        let lenc = self.iconst(*len as i64);
                        let oob = self.b.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, i, lenc);
                        self.fault_if(oob, FaultKind::Bounds, *span);
                        let str_ = self.iconst(*stride as i64);
                        let off = self.b.ins().imul(i, str_);
                        addr = self.b.ins().iadd(addr, off);
                    }
                    ty = match &ty {
                        Type::Array(e, _) => (**e).clone(),
                        Type::Slice(e) | Type::SliceMut(e) => (**e).clone(),
                        // `str[i]` -> the byte `u8` (design 0013 §3).
                        Type::Str => Type::Scalar(ScalarTy::U8),
                        _ => Type::Error,
                    };
                }
            }
        }
        (addr, ty)
    }

    fn eval_rvalue(&mut self, rv: &Rvalue, mf: &MirFn) -> Value {
        match rv {
            Rvalue::Use(op) => self.eval_operand(op, mf),
            Rvalue::Ref(place) => self.place_addr(place, mf).0,
            Rvalue::Load { place, ty } => {
                let (a, _) = self.place_addr(place, mf);
                self.load_scalar(a, scalar_of(ty))
            }
            Rvalue::StaticAddr(name) => {
                let a = self.statics[name];
                self.iconst(a as i64)
            }
            Rvalue::StrAddr(s) => {
                let a = self.strings[s];
                self.iconst(a as i64)
            }
            Rvalue::IsNull(op) => {
                let v = self.eval_operand(op, mf);
                let z = self.iconst(0);
                let c = self.b.ins().icmp(IntCC::Equal, v, z);
                self.b.ins().uextend(types::I64, c)
            }
            Rvalue::PtrArith { base, index, stride } => {
                let b = self.eval_operand(base, mf);
                let i = self.eval_operand(index, mf);
                let s = self.iconst(*stride as i64);
                let m = self.b.ins().imul(i, s);
                self.b.ins().iadd(b, m)
            }
            Rvalue::Cmp { op, l, r } => {
                let lsty = self.operand_sty(l, mf);
                let rsty = self.operand_sty(r, mf);
                let lv = self.eval_operand(l, mf);
                let rv = self.eval_operand(r, mf);
                if lsty.is_float() || rsty.is_float() {
                    // IEEE compare: `NotEqual` is unordered-or-not-equal (NaN != NaN
                    // is true); the others are ordered (any NaN yields false). Both
                    // operands share the same float type (checker-guaranteed).
                    let sty = if lsty.is_float() { lsty } else { rsty };
                    let fa = self.as_float(sty, lv);
                    let fb = self.as_float(sty, rv);
                    let cc = match op {
                        BinOp::Eq => FloatCC::Equal,
                        BinOp::Ne => FloatCC::NotEqual,
                        BinOp::Lt => FloatCC::LessThan,
                        BinOp::Le => FloatCC::LessThanOrEqual,
                        BinOp::Gt => FloatCC::GreaterThan,
                        BinOp::Ge => FloatCC::GreaterThanOrEqual,
                        _ => unreachable!("non-comparison in Cmp"),
                    };
                    let c = self.b.ins().fcmp(cc, fa, fb);
                    return self.b.ins().uextend(types::I64, c);
                }
                let l128 = self.ext128(lv, lsty);
                let r128 = self.ext128(rv, rsty);
                let cc = match op {
                    BinOp::Eq => IntCC::Equal,
                    BinOp::Ne => IntCC::NotEqual,
                    BinOp::Lt => IntCC::SignedLessThan,
                    BinOp::Le => IntCC::SignedLessThanOrEqual,
                    BinOp::Gt => IntCC::SignedGreaterThan,
                    BinOp::Ge => IntCC::SignedGreaterThanOrEqual,
                    _ => unreachable!("non-comparison in Cmp"),
                };
                let c = self.b.ins().icmp(cc, l128, r128);
                self.b.ins().uextend(types::I64, c)
            }
            Rvalue::Bin { op, regime, ty, l, r, span, fault } => {
                self.eval_bin(*op, *regime, *ty, l, r, *span, fault.as_ref(), mf)
            }
            Rvalue::Un { op, regime, ty, v, fault } => {
                let x = self.eval_operand(v, mf);
                match op {
                    UnOp::Not => {
                        let z = self.iconst(0);
                        let c = self.b.ins().icmp(IntCC::Equal, x, z);
                        self.b.ins().uextend(types::I64, c)
                    }
                    UnOp::BitNot => {
                        let n = self.b.ins().bnot(x);
                        self.canon(n, *ty)
                    }
                    UnOp::Neg if ty.is_float() => {
                        let f = self.as_float(*ty, x);
                        let n = self.b.ins().fneg(f);
                        self.float_bits(*ty, n)
                    }
                    UnOp::Neg => {
                        let x128 = self.ext128(x, *ty);
                        let zero = self.iconst128(0);
                        let neg = self.b.ins().isub(zero, x128);
                        self.range_or_fit(neg, *ty, *regime, fault.as_ref())
                    }
                }
            }
            Rvalue::Conv { to, regime, v, fault } => {
                let sty = self.operand_sty(v, mf);
                let x = self.eval_operand(v, mf);
                if to.is_float() || sty.is_float() {
                    return self.eval_float_conv(sty, *to, x);
                }
                let x128 = self.ext128(x, sty);
                self.range_or_fit(x128, *to, *regime, fault.as_ref())
            }
            Rvalue::Bitcast { to, v } => {
                // Pure bit reinterpretation in the i64-register model (design 0016
                // section 10): the register already holds the operand's bit pattern,
                // so we only re-canonicalize to the target width/signedness (a no-op
                // at 64 bits, ireduce+extend at 32) -- NOT an `fcvt`. Never faults.
                let x = self.eval_operand(v, mf);
                self.canon(x, *to)
            }
            Rvalue::Sqrt { ty, v } => {
                // Native IEEE square root via Cranelift's `sqrt` instruction (design
                // 0016 §11): reinterpret the operand's bit pattern as a float, take
                // the native sqrt, reinterpret back. Total -- never faults.
                let x = self.eval_operand(v, mf);
                let f = self.as_float(*ty, x);
                let s = self.b.ins().sqrt(f);
                self.float_bits(*ty, s)
            }
            Rvalue::Call { func, args } => {
                // A foreign `extern` call (design 0011 §5) has no MIR FuncId; it is
                // lowered to a call on the imported C symbol with C-ABI marshalling.
                if self.extern_ids.contains_key(func) {
                    self.lower_extern_call(func, args, mf)
                } else {
                    let vals: Vec<Value> = args.iter().map(|a| self.eval_operand(a, mf)).collect();
                    let r = self.callref(func);
                    let c = self.b.ins().call(r, &vals);
                    self.b.inst_results(c)[0]
                }
            }
            Rvalue::CallIndirect { func, args } => {
                let id = self.eval_operand(func, mf);
                let vals: Vec<Value> = args.iter().map(|a| self.eval_operand(a, mf)).collect();
                // faddr = *(fnptr_table + id*8)
                let base = self.fntable_base();
                let eight = self.iconst(8);
                let off = self.b.ins().imul(id, eight);
                let slot = self.b.ins().iadd(base, off);
                let faddr = self.b.ins().load(types::I64, MemFlags::new(), slot, 0);
                let mut sig = Signature::new(self.module.isa().default_call_conv());
                for _ in 0..args.len() {
                    sig.params.push(AbiParam::new(types::I64));
                }
                sig.returns.push(AbiParam::new(types::I64));
                let sr = self.b.import_signature(sig);
                let c = self.b.ins().call_indirect(sr, faddr, &vals);
                self.b.inst_results(c)[0]
            }
        }
    }

    /// Range-check `v128` against `sty`; deliver/fit per regime (add/sub/mul/neg/
    /// conv share this — INV-CHECK).
    fn range_or_fit(&mut self, v128: Value, sty: ScalarTy, regime: Regime, fault: Option<&FaultEdge>) -> Value {
        let (min, max, _, _) = ty_range(sty);
        match regime {
            Regime::Checked => {
                let minc = self.iconst128(min);
                let maxc = self.iconst128(max);
                let lt = self.b.ins().icmp(IntCC::SignedLessThan, v128, minc);
                let gt = self.b.ins().icmp(IntCC::SignedGreaterThan, v128, maxc);
                let bad = self.b.ins().bor(lt, gt);
                let edge = fault.expect("INV-CHECK: checked op lacks its fault edge");
                self.fault_if(bad, edge.kind, edge.span);
                self.fit128(v128, sty)
            }
            Regime::Wrapping => self.fit128(v128, sty),
            Regime::Saturating => {
                // Compare in i128 but SELECT in i64: the egraph rewrites a
                // select-min/max into `smax`/`smin`, and Cranelift's x86 backend has
                // NO ISLE lowering for `smax.i128`/`smin.i128` (a codegen gap only
                // `opt_level=speed` exposes — Stage D fault-axis finding). Selecting
                // the already-fitted i64 value keeps every min/max at i64 width, which
                // IS lowerable. The clamp bounds fit i64 (they are `sty`'s range).
                let minc = self.iconst128(min);
                let maxc = self.iconst128(max);
                let lt = self.b.ins().icmp(IntCC::SignedLessThan, v128, minc);
                let gt = self.b.ins().icmp(IntCC::SignedGreaterThan, v128, maxc);
                let fit = self.fit128(v128, sty);
                let min64 = self.iconst(min as i64);
                let max64 = self.iconst(max as i64);
                let lo = self.b.ins().select(lt, min64, fit);
                self.b.ins().select(gt, max64, lo)
            }
        }
    }

    /// A numeric `conv` where the source and/or target is a float (design 0016 §5).
    /// int->float rounds; `f32`->`f64` is exact (`fpromote`); `f64`->`f32` rounds
    /// (`fdemote`); float->int truncates toward zero, saturating (`fcvt_to_*int_sat`
    /// — NaN->0, out-of-range clamps). `x` is the i64 register value.
    fn eval_float_conv(&mut self, from: ScalarTy, to: ScalarTy, x: Value) -> Value {
        // float -> float (widen/narrow), or a same-width identity.
        if from.is_float() && to.is_float() {
            if from == to {
                return x;
            }
            let f = self.as_float(from, x);
            let g = if to == ScalarTy::F64 {
                self.b.ins().fpromote(types::F64, f)
            } else {
                self.b.ins().fdemote(types::F32, f)
            };
            return self.float_bits(to, g);
        }
        // int -> float: the register already holds the canonical sign/zero-extended
        // value, so pick the matching signedness.
        if to.is_float() {
            let ft = if to == ScalarTy::F32 { types::F32 } else { types::F64 };
            let (_, _, _, signed) = ty_range(from);
            let f = if signed {
                self.b.ins().fcvt_from_sint(ft, x)
            } else {
                self.b.ins().fcvt_from_uint(ft, x)
            };
            return self.float_bits(to, f);
        }
        // float -> int: saturate to the exact target width, then canonicalize to i64.
        let (_, _, bits, signed) = ty_range(to);
        let f = self.as_float(from, x);
        let it = int_ty(crate::interp::layout::Layout::scalar_size(to));
        let narrow = if signed {
            self.b.ins().fcvt_to_sint_sat(it, f)
        } else {
            self.b.ins().fcvt_to_uint_sat(it, f)
        };
        if bits >= 64 {
            narrow
        } else if signed {
            self.b.ins().sextend(types::I64, narrow)
        } else {
            self.b.ins().uextend(types::I64, narrow)
        }
    }

    fn eval_bin(&mut self, op: BinOp, regime: Regime, ty: ScalarTy, l: &Operand, r: &Operand, span: Span, fault: Option<&FaultEdge>, mf: &MirFn) -> Value {
        use BinOp::*;
        let lv = self.eval_operand(l, mf);
        let rv = self.eval_operand(r, mf);
        if ty.is_float() {
            // IEEE-754 arithmetic: bit-cast, native op, bit-cast back. Never faults.
            let fa = self.as_float(ty, lv);
            let fb = self.as_float(ty, rv);
            let res = match op {
                Add => self.b.ins().fadd(fa, fb),
                Sub => self.b.ins().fsub(fa, fb),
                Mul => self.b.ins().fmul(fa, fb),
                Div => self.b.ins().fdiv(fa, fb),
                _ => unreachable!("only + - * / reach a float Bin"),
            };
            return self.float_bits(ty, res);
        }
        let (_, _, bits, signed) = ty_range(ty);
        match op {
            Add | Sub | Mul => {
                let a = self.ext128(lv, ty);
                let b = self.ext128(rv, ty);
                let res = match op {
                    Add => self.b.ins().iadd(a, b),
                    Sub => self.b.ins().isub(a, b),
                    _ => self.b.ins().imul(a, b),
                };
                self.range_or_fit(res, ty, regime, fault)
            }
            Div | Rem => {
                let z = self.iconst(0);
                let iz = self.b.ins().icmp(IntCC::Equal, rv, z);
                self.fault_if(iz, FaultKind::DivByZero, span);
                if !signed {
                    let q = if op == Div {
                        self.b.ins().udiv(lv, rv)
                    } else {
                        self.b.ins().urem(lv, rv)
                    };
                    self.canon(q, ty)
                } else {
                    // Signed: guard MIN/-1 (Cranelift sdiv/srem trap on it).
                    let (min, max, _, _) = ty_range(ty);
                    let minc = self.iconst(min as i64);
                    let m1 = self.iconst(-1);
                    let is_min = self.b.ins().icmp(IntCC::Equal, lv, minc);
                    let is_m1 = self.b.ins().icmp(IntCC::Equal, rv, m1);
                    let ov = self.b.ins().band(is_min, is_m1);
                    if op == Rem {
                        // MIN % -1 == 0; otherwise srem with a safe divisor.
                        let one = self.iconst(1);
                        let safe = self.b.ins().select(ov, one, rv);
                        let rr = self.b.ins().srem(lv, safe);
                        let zero = self.iconst(0);
                        let sel = self.b.ins().select(ov, zero, rr);
                        self.canon(sel, ty)
                    } else {
                        match regime {
                            Regime::Checked => {
                                let edge = fault.expect("INV-CHECK: checked div lacks its fault edge");
                                self.fault_if(ov, edge.kind, edge.span);
                                let q = self.b.ins().sdiv(lv, rv);
                                self.canon(q, ty)
                            }
                            Regime::Wrapping => {
                                let one = self.iconst(1);
                                let safe = self.b.ins().select(ov, one, rv);
                                let q = self.b.ins().sdiv(lv, safe);
                                let minv = self.iconst(min as i64);
                                let sel = self.b.ins().select(ov, minv, q);
                                self.canon(sel, ty)
                            }
                            Regime::Saturating => {
                                let one = self.iconst(1);
                                let safe = self.b.ins().select(ov, one, rv);
                                let q = self.b.ins().sdiv(lv, safe);
                                let maxv = self.iconst(max as i64);
                                let sel = self.b.ins().select(ov, maxv, q);
                                self.canon(sel, ty)
                            }
                        }
                    }
                }
            }
            BitAnd => {
                let x = self.b.ins().band(lv, rv);
                self.canon(x, ty)
            }
            BitOr => {
                let x = self.b.ins().bor(lv, rv);
                self.canon(x, ty)
            }
            BitXor => {
                let x = self.b.ins().bxor(lv, rv);
                self.canon(x, ty)
            }
            Shl | Shr => {
                // Amount handling mirrors mir::interp exactly.
                let bitsc = self.iconst(bits as i64);
                let zero = self.iconst(0);
                let neg = self.b.ins().icmp(IntCC::SignedLessThan, rv, zero);
                self.fault_if(neg, FaultKind::Overflow, span);
                let ge = self.b.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, rv, bitsc);
                let amt = match regime {
                    Regime::Checked => {
                        self.fault_if(ge, FaultKind::Overflow, span);
                        rv
                    }
                    Regime::Wrapping => {
                        let m = self.b.ins().urem(rv, bitsc);
                        self.b.ins().select(ge, m, rv)
                    }
                    Regime::Saturating => {
                        let bm1 = self.iconst(bits as i64 - 1);
                        self.b.ins().select(ge, bm1, rv)
                    }
                };
                let raw = if op == Shl {
                    self.b.ins().ishl(lv, amt)
                } else if signed {
                    self.b.ins().sshr(lv, amt)
                } else {
                    self.b.ins().ushr(lv, amt)
                };
                self.canon(raw, ty)
            }
            _ => unreachable!("comparison/logical in Bin"),
        }
    }

    // ---- statements ----
    fn lower_stmt(&mut self, st: &Statement, mf: &MirFn) {
        // INV-OBS-ORDER: an observable rawptr/MMIO scalar access lowers to a barrier
        // CALL (rt_mmio_load/store) rather than an inline load/store, so the optimizer
        // holds its program order (the F1 discipline, design 0010 §1/§2).
        if st.observable {
            match &st.kind {
                StatementKind::Assign(local, Rvalue::Load { place, ty }) => {
                    let (a, _) = self.place_addr(place, mf);
                    let sty = scalar_of(ty);
                    let size = crate::interp::layout::Layout::scalar_size(sty);
                    let raw = self.call_mmio_load(a, size);
                    let v = self.canon(raw, sty);
                    let da = self.addr[*local];
                    self.store_scalar(da, v, scalar_of(&mf.locals[*local].ty));
                    return;
                }
                StatementKind::Store(place, rv) => {
                    let val = self.eval_rvalue(rv, mf);
                    let (a, ty) = self.place_addr(place, mf);
                    let sty = scalar_of(&ty);
                    let size = crate::interp::layout::Layout::scalar_size(sty);
                    self.call_mmio_store(a, val, size);
                    return;
                }
                // Aggregate observable read (CopyVal) already lowers to rt_copy — a
                // barrier call — so it falls through to the normal path below.
                _ => {}
            }
        }
        match &st.kind {
            StatementKind::Assign(local, rv) => {
                let val = self.eval_rvalue(rv, mf);
                let a = self.addr[*local];
                self.store_scalar(a, val, scalar_of(&mf.locals[*local].ty));
            }
            StatementKind::Store(place, rv) => {
                let val = self.eval_rvalue(rv, mf);
                let (a, ty) = self.place_addr(place, mf);
                self.store_scalar(a, val, scalar_of(&ty));
            }
            StatementKind::CopyVal { dst, src, ty } => {
                let (s, _) = self.place_addr(src, mf);
                let (d, _) = self.place_addr(dst, mf);
                self.call_copy(d, s, self.size_of(ty));
            }
            StatementKind::Trace(op) => {
                let v = self.eval_operand(op, mf);
                let r = self.shimref("trace", self.shims.trace);
                self.b.ins().call(r, &[v]);
            }
            StatementKind::Drop { local, moved } => {
                let a = self.addr[*local];
                let ty = mf.locals[*local].ty.clone();
                self.emit_drop(a, &ty, moved, &mut Vec::new());
            }
            StatementKind::BoxOp { dst, inner_ty, result_ty, alloc, value } => {
                self.box_op(dst, inner_ty, result_ty, alloc, value, mf);
            }
            StatementKind::UnboxOp { dst, inner_ty, boxed } => {
                self.unbox_op(dst, inner_ty, boxed, mf);
            }
            StatementKind::Subslice { dst, src, lo, hi, stride, span } => {
                self.subslice_op(dst, src, lo, hi, *stride, *span, mf);
            }
            StatementKind::StrFrom { dst, src } => {
                self.str_from_op(dst, src, mf);
            }
            StatementKind::Substr { dst, src, lo, hi, span } => {
                self.substr_op(dst, src, lo, hi, *span, mf);
            }
            // The compiler-known `String` intrinsics (design 0013) lowered inline,
            // mirroring the MIR interpreter byte-for-byte (`mir::interp::collection_op`).
            // `Vec`/`Map` + String `push` (UTF-8) are the remaining forward slices.
            StatementKind::CollectionOp { dst, op } => {
                self.collection_op(dst, op, mf);
            }
            // Stage 2 (design 0012 §6): `spawn` becomes real thread creation, the
            // `scope` markers push/join a frame. Args are evaluated on the PARENT
            // thread (the marshalling) and handed to the task's thread by value.
            StatementKind::Spawn { func, args } => {
                let fref = self.callref(func);
                let faddr = self.b.ins().func_addr(types::I64, fref);
                let argc = self.iconst(args.len() as i64);
                let mut vals = vec![faddr, argc];
                for a in args {
                    let v = self.eval_operand(a, mf);
                    vals.push(v);
                }
                while vals.len() < 2 + MAX_SPAWN_ARGS {
                    let z = self.iconst(0);
                    vals.push(z);
                }
                let r = self.shimref("spawn", self.shims.spawn);
                self.b.ins().call(r, &vals);
            }
            StatementKind::ScopeBegin => {
                let r = self.shimref("scope_begin", self.shims.scope_begin);
                self.b.ins().call(r, &[]);
            }
            StatementKind::ScopeEnd => {
                let r = self.shimref("scope_end", self.shims.scope_end);
                self.b.ins().call(r, &[]);
            }
        }
    }

    fn lower_fn(&mut self, mf: &MirFn) {
        let entry = self.b.create_block();
        self.b.switch_to_block(entry);
        self.b.append_block_params_for_function_params(entry);
        let params: Vec<Value> = self.b.block_params(entry).to_vec();

        // Allocate each local's stack slot (mirrors interp `alloc_slot`).
        self.addr = Vec::with_capacity(mf.locals.len());
        for l in &mf.locals {
            let size = self.size_of(&l.ty).max(1);
            let align = self.align_of(&l.ty).max(1);
            let a = self.stack_alloc(size, align);
            self.addr.push(a);
        }
        // Bind params _1..=n.
        for (i, p) in params.iter().enumerate() {
            let pty = mf.locals[1 + i].ty.clone();
            let a = self.addr[1 + i];
            if is_wordy(&pty) {
                self.store_scalar(a, *p, scalar_of(&pty));
            } else {
                self.call_copy(a, *p, self.size_of(&pty));
            }
        }

        // Cranelift blocks for MIR blocks + synthetic region blocks.
        let clblocks: Vec<Block> = (0..mf.blocks.len()).map(|_| self.b.create_block()).collect();
        let final_return = self.b.create_block();
        let ensures_start = if !mf.ensures.is_empty() { Some(self.b.create_block()) } else { None };
        let body_ret = ensures_start.unwrap_or(final_return);
        let req_checks: Vec<Block> = mf.requires.iter().map(|_| self.b.create_block()).collect();
        let ens_checks: Vec<Block> = mf.ensures.iter().map(|_| self.b.create_block()).collect();

        // Assign each MIR block its return-target (region membership).
        let mut ret_target: Vec<Option<Block>> = vec![None; mf.blocks.len()];
        for (i, pred) in mf.requires.iter().enumerate() {
            assign_region(mf, pred.entry, req_checks[i], &mut ret_target);
        }
        assign_region(mf, mf.entry, body_ret, &mut ret_target);
        for (i, pred) in mf.ensures.iter().enumerate() {
            assign_region(mf, pred.entry, ens_checks[i], &mut ret_target);
        }

        // entry -> first region.
        let first = if let Some(p) = mf.requires.first() { clblocks[p.entry] } else { clblocks[mf.entry] };
        self.b.ins().jump(first, &[]);

        // requires check blocks.
        for (i, pred) in mf.requires.iter().enumerate() {
            self.b.switch_to_block(req_checks[i]);
            let v = self.load_scalar(self.addr[pred.value], ScalarTy::Bool);
            let ok = if i + 1 < mf.requires.len() {
                clblocks[mf.requires[i + 1].entry]
            } else {
                clblocks[mf.entry]
            };
            let fault = self.b.create_block();
            self.b.ins().brif(v, ok, &[], fault, &[]);
            self.b.switch_to_block(fault);
            self.emit_fault(pred.kind, pred.span);
        }

        // ensures_start: copy _0 -> result_local, enter ensures[0].
        if let Some(es) = ensures_start {
            self.b.switch_to_block(es);
            if let Some(rl) = mf.result_local {
                let sz = self.size_of(&mf.locals[0].ty);
                let d = self.addr[rl];
                let s = self.addr[0];
                self.call_copy(d, s, sz);
            }
            let t = clblocks[mf.ensures[0].entry];
            self.b.ins().jump(t, &[]);
        }
        // ensures check blocks.
        for (i, pred) in mf.ensures.iter().enumerate() {
            self.b.switch_to_block(ens_checks[i]);
            let v = self.load_scalar(self.addr[pred.value], ScalarTy::Bool);
            let ok = if i + 1 < mf.ensures.len() {
                clblocks[mf.ensures[i + 1].entry]
            } else {
                final_return
            };
            let fault = self.b.create_block();
            self.b.ins().brif(v, ok, &[], fault, &[]);
            self.b.switch_to_block(fault);
            self.emit_fault(pred.kind, pred.span);
        }

        // final_return: value per convention (B).
        self.b.switch_to_block(final_return);
        let rty = mf.locals[0].ty.clone();
        let rv = if is_wordy(&rty) {
            self.load_scalar(self.addr[0], scalar_of(&rty))
        } else {
            self.addr[0]
        };
        self.b.ins().return_(&[rv]);

        // Fill each MIR block.
        for (bid, block) in mf.blocks.iter().enumerate() {
            self.b.switch_to_block(clblocks[bid]);
            let rt = ret_target[bid];
            if rt.is_none() {
                // Dead / unassigned: terminate defensively.
                self.b.ins().trap(TrapCode::user(2).unwrap());
                continue;
            }
            for st in &block.stmts {
                self.lower_stmt(st, mf);
            }
            match &block.term {
                Terminator::Goto(n) => {
                    let t = clblocks[*n];
                    self.b.ins().jump(t, &[]);
                }
                Terminator::Branch { cond, then_bb, else_bb } => {
                    let c = self.eval_operand(cond, mf);
                    self.b.ins().brif(c, clblocks[*then_bb], &[], clblocks[*else_bb], &[]);
                }
                Terminator::Return => {
                    self.b.ins().jump(rt.unwrap(), &[]);
                }
                Terminator::Fault(edge) => {
                    self.emit_fault(edge.kind, edge.span);
                }
            }
        }
    }

    /// Build the AOT entry wrapper body (see `define_entry`).
    fn lower_entry(&mut self, main_is_i64: bool) {
        let entry = self.b.create_block();
        self.b.switch_to_block(entry);

        // Copy each string literal's bytes into the flat buffer at its baked
        // Candor address (the JIT driver does this host-side before running).
        let strs: Vec<(String, u64)> =
            self.strings.iter().map(|(k, v)| (k.clone(), *v)).collect();
        for (sv, addr) in strs {
            for (i, byte) in sv.as_bytes().iter().enumerate() {
                let a = self.iconst((addr + i as u64) as i64);
                let ha = self.host_addr(a);
                let v = self.b.ins().iconst(types::I8, *byte as i64);
                self.b.ins().store(MemFlags::new(), v, ha, 0);
            }
        }

        // Run each static initializer and write its result to the static's addr
        // (wordy: store the low `size` bytes; aggregate: `rt_copy` from the
        // returned address) — mirroring `backend::run`'s init loop.
        let inits: Vec<(u64, u64, bool, String)> = self
            .prog
            .statics
            .iter()
            .map(|st| {
                (
                    self.statics[&st.name],
                    self.size_of(&st.ty),
                    is_wordy(&st.ty),
                    st.init_fn.clone(),
                )
            })
            .collect();
        for (addr, size, wordy, init_fn) in inits {
            let r = self.callref(&init_fn);
            let c = self.b.ins().call(r, &[]);
            let out = self.b.inst_results(c)[0];
            if wordy {
                if size > 0 {
                    let a = self.iconst(addr as i64);
                    let ha = self.host_addr(a);
                    let st = int_ty(size);
                    let v = if size >= 8 { out } else { self.b.ins().ireduce(st, out) };
                    self.b.ins().store(MemFlags::new(), v, ha, 0);
                }
            } else {
                let d = self.iconst(addr as i64);
                self.call_copy(d, out, size);
            }
        }

        // Call `main` and return its `i64` (or 0 for a non-`i64` main).
        let r = self.callref("main");
        let c = self.b.ins().call(r, &[]);
        let ret0 = self.b.inst_results(c)[0];
        let ret = if main_is_i64 { ret0 } else { self.iconst(0) };
        self.b.ins().return_(&[ret]);
    }
}

/// DFS successors from `entry`, tagging each unvisited MIR block with `target`.
fn assign_region(mf: &MirFn, entry: usize, target: Block, ret_target: &mut [Option<Block>]) {
    let mut stack = vec![entry];
    while let Some(bid) = stack.pop() {
        if ret_target[bid].is_some() {
            continue;
        }
        ret_target[bid] = Some(target);
        match &mf.blocks[bid].term {
            Terminator::Goto(n) => stack.push(*n),
            Terminator::Branch { then_bb, else_bb, .. } => {
                stack.push(*then_bb);
                stack.push(*else_bb);
            }
            Terminator::Return | Terminator::Fault(_) => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Aggregate ops (Box/unbox/subslice) and the static drop schedule (INV-DROP),
// ported op-for-op from `mir::interp`. Box/drop dispatch through the allocator
// vtable as ordinary indirect calls (design 0010 §5); the drop recursion is
// unrolled at compile time over the static type + static move mask (no runtime
// drop flag), with a runtime tag switch only for enum payload dispatch.
// ---------------------------------------------------------------------------

impl<M: Module> Cg<'_, '_, M> {
    fn load_u64(&mut self, candor: Value) -> Value {
        self.load_scalar(candor, ScalarTy::U64)
    }
    fn store_u64(&mut self, candor: Value, val: Value) {
        self.store_scalar(candor, val, ScalarTy::U64);
    }
    fn add_off(&mut self, addr: Value, off: u64) -> Value {
        let o = self.iconst(off as i64);
        self.b.ins().iadd(addr, o)
    }
    fn field_off(&self, sname: &str, field: &str) -> u64 {
        self.lay().field_offset(sname, field).map(|(_, o)| o).unwrap_or(0)
    }
    /// Structurally identify the allocator handle / vtable structs (design 0010
    /// §5; identical to `mir::interp`), so box/unbox find `ctx`/`vt`/`alloc`/
    /// `free` regardless of module qualification.
    fn alloc_vtable_name(&self) -> String {
        self.items
            .structs
            .iter()
            .find(|(_, s)| {
                s.fields.iter().any(|(n, t)| n == "alloc" && matches!(t, Type::FnPtr(_)))
                    && s.fields.iter().any(|(n, t)| n == "free" && matches!(t, Type::FnPtr(_)))
            })
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| "AllocVtable".to_string())
    }
    fn alloc_struct_name(&self) -> String {
        let vt = self.alloc_vtable_name();
        self.items
            .structs
            .iter()
            .find(|(_, s)| {
                s.fields.iter().any(|(n, t)| {
                    n == "vt"
                        && matches!(t, Type::RawPtr(inner) if matches!(&**inner, Type::Named(x) if *x == vt))
                })
            })
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| "Alloc".to_string())
    }

    /// Indirect call by fn-pointer id (box/unbox vtable dispatch); returns the
    /// callee's word result.
    fn call_fnptr_id(&mut self, id: Value, args: &[Value]) -> Value {
        let base = self.fntable_base();
        let eight = self.iconst(8);
        let off = self.b.ins().imul(id, eight);
        let slot = self.b.ins().iadd(base, off);
        let faddr = self.b.ins().load(types::I64, MemFlags::new(), slot, 0);
        let mut sig = Signature::new(self.module.isa().default_call_conv());
        for _ in 0..args.len() {
            sig.params.push(AbiParam::new(types::I64));
        }
        sig.returns.push(AbiParam::new(types::I64));
        let sr = self.b.import_signature(sig);
        let c = self.b.ins().call_indirect(sr, faddr, args);
        self.b.inst_results(c)[0]
    }

    fn call_free(&mut self, ctx: Value, vt: Value, ptr: Value, size: u64, align: u64) {
        let free_off = self.field_off(&self.alloc_vtable_name(), "free");
        let fa = self.add_off(vt, free_off);
        let ffn = self.load_u64(fa);
        let sz = self.iconst(size as i64);
        let al = self.iconst(align as i64);
        self.call_fnptr_id(ffn, &[ctx, ptr, sz, al]);
    }

    fn box_op(&mut self, dst: &Place, inner_ty: &Type, result_ty: &Type, alloc: &Operand, value: &Place, mf: &MirFn) {
        let alloc_addr = self.eval_operand(alloc, mf);
        let astruct = self.alloc_struct_name();
        let vtstruct = self.alloc_vtable_name();
        let ctx_off = self.field_off(&astruct, "ctx");
        let vt_off = self.field_off(&astruct, "vt");
        let ca = self.add_off(alloc_addr, ctx_off);
        let ctx = self.load_u64(ca);
        let va = self.add_off(alloc_addr, vt_off);
        let vt = self.load_u64(va);
        let size = self.size_of(inner_ty);
        let align = self.align_of(inner_ty);
        let alloc_off = self.field_off(&vtstruct, "alloc");
        let aa = self.add_off(vt, alloc_off);
        let afn = self.load_u64(aa);
        let sz = self.iconst(size as i64);
        let al = self.iconst(align as i64);
        let ret = self.call_fnptr_id(afn, &[ctx, sz, al]);
        let (value_addr, _) = self.place_addr(value, mf);
        let (daddr, _) = self.place_addr(dst, mf);

        let einfo = self.lay().enum_info(result_ty);
        let (boxed_idx, oom_idx) = match &einfo {
            Some(v) => (
                v.iter().position(|(_, p)| !p.is_empty()).unwrap_or(0) as i64,
                v.iter().position(|(_, p)| p.is_empty()).unwrap_or(1) as i64,
            ),
            None => (0, 1),
        };

        let zero = self.iconst(0);
        let is_oom = self.b.ins().icmp(IntCC::Equal, ret, zero);
        let oom_b = self.b.create_block();
        let boxed_b = self.b.create_block();
        let cont = self.b.create_block();
        self.b.ins().brif(is_oom, oom_b, &[], boxed_b, &[]);

        self.b.switch_to_block(oom_b);
        self.emit_drop(value_addr, inner_ty, &[], &mut Vec::new());
        let oi = self.iconst(oom_idx);
        self.store_u64(daddr, oi);
        self.b.ins().jump(cont, &[]);

        self.b.switch_to_block(boxed_b);
        self.call_copy(ret, value_addr, size);
        let bi = self.iconst(boxed_idx);
        self.store_u64(daddr, bi);
        let d8 = self.add_off(daddr, 8);
        self.store_u64(d8, ret);
        let d16 = self.add_off(daddr, 16);
        self.store_u64(d16, ctx);
        let d24 = self.add_off(daddr, 24);
        self.store_u64(d24, vt);
        self.b.ins().jump(cont, &[]);

        self.b.switch_to_block(cont);
    }

    fn unbox_op(&mut self, dst: &Place, inner_ty: &Type, boxed: &Place, mf: &MirFn) {
        let (baddr, _) = self.place_addr(boxed, mf);
        let ptr = self.load_u64(baddr);
        let b8 = self.add_off(baddr, 8);
        let ctx = self.load_u64(b8);
        let b16 = self.add_off(baddr, 16);
        let vt = self.load_u64(b16);
        let size = self.size_of(inner_ty);
        let align = self.align_of(inner_ty);
        let (daddr, _) = self.place_addr(dst, mf);
        self.call_copy(daddr, ptr, size);
        self.call_free(ctx, vt, ptr, size, align);
    }

    /// The `String` collection intrinsics (design 0013), lowered inline to mirror
    /// `mir::interp::collection_op` byte-for-byte. `New` (the shared empty-header
    /// init) and `string_reserve` (alloc-new + rt_copy + free-old growth) are the
    /// reusable substrate for the coming `Vec`/`Map` slices.
    fn collection_op(&mut self, dst: &Place, op: &CollOp, mf: &MirFn) {
        match op {
            CollOp::New { alloc } => {
                let alloc_addr = self.eval_operand(alloc, mf);
                let astruct = self.alloc_struct_name();
                let ctx_off = self.field_off(&astruct, "ctx");
                let vt_off = self.field_off(&astruct, "vt");
                let ca = self.add_off(alloc_addr, ctx_off);
                let ctx = self.load_u64(ca);
                let va = self.add_off(alloc_addr, vt_off);
                let vt = self.load_u64(va);
                let (daddr, _) = self.place_addr(dst, mf);
                let zero = self.iconst(0);
                self.store_u64(daddr, zero);
                let d8 = self.add_off(daddr, 8);
                self.store_u64(d8, zero);
                let d16 = self.add_off(daddr, 16);
                self.store_u64(d16, zero);
                let d24 = self.add_off(daddr, 24);
                self.store_u64(d24, ctx);
                let d32 = self.add_off(daddr, 32);
                self.store_u64(d32, vt);
            }
            CollOp::StringAsStr { base } => {
                let base = self.eval_operand(base, mf);
                let buf = self.load_u64(base);
                let b8 = self.add_off(base, 8);
                let len = self.load_u64(b8);
                let (daddr, _) = self.place_addr(dst, mf);
                self.store_u64(daddr, buf);
                let d8 = self.add_off(daddr, 8);
                self.store_u64(d8, len);
            }
            CollOp::StringAppend { base, view, span } => {
                let base = self.eval_operand(base, mf);
                let (vaddr, _) = self.place_addr(view, mf);
                let ptr = self.load_u64(vaddr);
                let v8 = self.add_off(vaddr, 8);
                let slen = self.load_u64(v8);
                self.string_reserve(base, slen, *span);
                let buf = self.load_u64(base);
                let b8 = self.add_off(base, 8);
                let len = self.load_u64(b8);
                let dstp = self.b.ins().iadd(buf, len);
                self.call_copy_val(dstp, ptr, slen);
                let newlen = self.b.ins().iadd(len, slen);
                self.store_u64(b8, newlen);
            }
            CollOp::StringPush { base, ch, span } => {
                let base = self.eval_operand(base, mf);
                let c = self.eval_operand(ch, mf);
                // Reject non-scalar-values (surrogates, > 0x10FFFF) exactly as
                // `utf8_encode_scalar`; the interp faults `Requires` at the call span.
                let max = self.iconst(0x10FFFF);
                let oor = self.b.ins().icmp(IntCC::UnsignedGreaterThan, c, max);
                let lo = self.iconst(0xD800);
                let hi = self.iconst(0xDFFF);
                let ge_lo = self.b.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, c, lo);
                let le_hi = self.b.ins().icmp(IntCC::UnsignedLessThanOrEqual, c, hi);
                let surr = self.b.ins().band(ge_lo, le_hi);
                let bad = self.b.ins().bor(oor, surr);
                self.fault_if(bad, FaultKind::Requires, *span);
                // UTF-8 length: <0x80 -> 1, <0x800 -> 2, <0x10000 -> 3, else 4.
                let c80 = self.iconst(0x80);
                let c800 = self.iconst(0x800);
                let c10000 = self.iconst(0x10000);
                let lt80 = self.b.ins().icmp(IntCC::UnsignedLessThan, c, c80);
                let lt800 = self.b.ins().icmp(IntCC::UnsignedLessThan, c, c800);
                let lt10000 = self.b.ins().icmp(IntCC::UnsignedLessThan, c, c10000);
                let one = self.iconst(1);
                let two = self.iconst(2);
                let three = self.iconst(3);
                let four = self.iconst(4);
                let l34 = self.b.ins().select(lt10000, three, four);
                let l234 = self.b.ins().select(lt800, two, l34);
                let enc_len = self.b.ins().select(lt80, one, l234);
                self.string_reserve(base, enc_len, *span);
                let buf = self.load_u64(base);
                let b8 = self.add_off(base, 8);
                let len = self.load_u64(b8);
                let dstp = self.b.ins().iadd(buf, len);
                // Write the 1-4 encoded bytes (branch per width, mirroring the
                // 1/2/3/4-byte arms of `utf8_encode_scalar` bit for bit).
                let w1 = self.b.create_block();
                let t2 = self.b.create_block();
                let w2 = self.b.create_block();
                let t3 = self.b.create_block();
                let w3 = self.b.create_block();
                let w4 = self.b.create_block();
                let cont = self.b.create_block();
                self.b.ins().brif(lt80, w1, &[], t2, &[]);
                self.b.switch_to_block(w1);
                self.store_byte(dstp, c);
                self.b.ins().jump(cont, &[]);
                self.b.switch_to_block(t2);
                self.b.ins().brif(lt800, w2, &[], t3, &[]);
                self.b.switch_to_block(w2);
                let b0 = self.utf8_lead(c, 6, 0xC0);
                self.store_byte(dstp, b0);
                let d1 = self.add_off(dstp, 1);
                let b1 = self.utf8_cont(c, 0);
                self.store_byte(d1, b1);
                self.b.ins().jump(cont, &[]);
                self.b.switch_to_block(t3);
                self.b.ins().brif(lt10000, w3, &[], w4, &[]);
                self.b.switch_to_block(w3);
                let e0 = self.utf8_lead(c, 12, 0xE0);
                self.store_byte(dstp, e0);
                let e1a = self.add_off(dstp, 1);
                let e1 = self.utf8_cont(c, 6);
                self.store_byte(e1a, e1);
                let e2a = self.add_off(dstp, 2);
                let e2 = self.utf8_cont(c, 0);
                self.store_byte(e2a, e2);
                self.b.ins().jump(cont, &[]);
                self.b.switch_to_block(w4);
                let f0 = self.utf8_lead(c, 18, 0xF0);
                self.store_byte(dstp, f0);
                let f1a = self.add_off(dstp, 1);
                let f1 = self.utf8_cont(c, 12);
                self.store_byte(f1a, f1);
                let f2a = self.add_off(dstp, 2);
                let f2 = self.utf8_cont(c, 6);
                self.store_byte(f2a, f2);
                let f3a = self.add_off(dstp, 3);
                let f3 = self.utf8_cont(c, 0);
                self.store_byte(f3a, f3);
                self.b.ins().jump(cont, &[]);
                self.b.switch_to_block(cont);
                let newlen = self.b.ins().iadd(len, enc_len);
                self.store_u64(b8, newlen);
            }
            CollOp::VecPush { base, elem, value, span } => {
                let base = self.eval_operand(base, mf);
                let stride = crate::interp::mem::round_up(self.size_of(elem), self.align_of(elem));
                let align = self.align_of(elem);
                self.vec_reserve(base, stride as i64, align as i64, 1, *span);
                let buf = self.load_u64(base);
                let b8 = self.add_off(base, 8);
                let len = self.load_u64(b8);
                let (vaddr, _) = self.place_addr(value, mf);
                let stridev = self.iconst(stride as i64);
                let off = self.b.ins().imul(len, stridev);
                let slot = self.b.ins().iadd(buf, off);
                self.call_copy(slot, vaddr, self.size_of(elem));
                let one = self.iconst(1);
                let newlen = self.b.ins().iadd(len, one);
                self.store_u64(b8, newlen);
            }
            CollOp::VecGet { base, elem, index, span } => {
                let base = self.eval_operand(base, mf);
                let b8 = self.add_off(base, 8);
                let len = self.load_u64(b8);
                let i = self.eval_operand(index, mf);
                let oob = self.b.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, i, len);
                self.fault_if(oob, FaultKind::Bounds, *span);
                let stride = crate::interp::mem::round_up(self.size_of(elem), self.align_of(elem));
                let buf = self.load_u64(base);
                let stridev = self.iconst(stride as i64);
                let off = self.b.ins().imul(i, stridev);
                let slot = self.b.ins().iadd(buf, off);
                let (daddr, _) = self.place_addr(dst, mf);
                self.store_u64(daddr, slot);
            }
            CollOp::VecSet { base, elem, index, value, span } => {
                let base = self.eval_operand(base, mf);
                let b8 = self.add_off(base, 8);
                let len = self.load_u64(b8);
                let i = self.eval_operand(index, mf);
                let oob = self.b.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, i, len);
                self.fault_if(oob, FaultKind::Bounds, *span);
                let stride = crate::interp::mem::round_up(self.size_of(elem), self.align_of(elem));
                let buf = self.load_u64(base);
                let stridev = self.iconst(stride as i64);
                let off = self.b.ins().imul(i, stridev);
                let slot = self.b.ins().iadd(buf, off);
                let (vaddr, _) = self.place_addr(value, mf);
                // Drop-on-overwrite: run the old element's drop glue before the move,
                // mirroring the interp's `drop_value(slot, elem)` in `VecSet`.
                self.emit_drop(slot, elem, &[], &mut Vec::new());
                self.call_copy(slot, vaddr, self.size_of(elem));
            }
            CollOp::VecPop { base, elem } => {
                let base = self.eval_operand(base, mf);
                let (daddr, _) = self.place_addr(dst, mf);
                let b8 = self.add_off(base, 8);
                let len = self.load_u64(b8);
                // Opt discriminants + the `Some` payload offset are compile-time layout.
                let opt = Type::Named("Opt".to_string());
                let einfo = self.lay().enum_info(&opt).expect("unknown enum `Opt`");
                let some_idx = einfo.iter().position(|(n, _)| n == "Some").unwrap_or(0);
                let none_idx = einfo.iter().position(|(n, _)| n == "None").unwrap_or(1);
                let some_payloads = einfo[some_idx].1.clone();
                let (_, poff) = self.lay().payload_offset(&some_payloads, 0);
                let zero = self.iconst(0);
                let is_empty = self.b.ins().icmp(IntCC::Equal, len, zero);
                let none_b = self.b.create_block();
                let some_b = self.b.create_block();
                let cont = self.b.create_block();
                self.b.ins().brif(is_empty, none_b, &[], some_b, &[]);
                self.b.switch_to_block(none_b);
                let nidx = self.iconst(none_idx as i64);
                self.store_u64(daddr, nidx);
                self.b.ins().jump(cont, &[]);
                self.b.switch_to_block(some_b);
                let one = self.iconst(1);
                let newlen = self.b.ins().isub(len, one);
                let stride = crate::interp::mem::round_up(self.size_of(elem), self.align_of(elem));
                let buf = self.load_u64(base);
                let stridev = self.iconst(stride as i64);
                let off = self.b.ins().imul(newlen, stridev);
                let src = self.b.ins().iadd(buf, off);
                self.store_u64(b8, newlen);
                let sidx = self.iconst(some_idx as i64);
                self.store_u64(daddr, sidx);
                let pdst = self.add_off(daddr, poff);
                self.call_copy(pdst, src, self.size_of(elem));
                self.b.ins().jump(cont, &[]);
                self.b.switch_to_block(cont);
            }
            CollOp::MapInsert { base, valty, key, value, span } => {
                let base = self.eval_operand(base, mf);
                let (kaddr, _) = self.place_addr(key, mf);
                let kptr = self.load_u64(kaddr);
                let ka8 = self.add_off(kaddr, 8);
                let klen = self.load_u64(ka8);
                let vsz = self.size_of(valty);
                let stride = crate::interp::mem::round_up(24 + vsz, 8) as i64;
                let (vaddr, _) = self.place_addr(value, mf);
                let buf0 = self.load_u64(base);
                let b16 = self.add_off(base, 16);
                let cap0 = self.load_u64(b16);
                let (found, slot) = self.map_find(buf0, cap0, stride, kptr, klen);
                let zero = self.iconst(0);
                let one = self.iconst(1);
                let is_found = self.b.ins().icmp(IntCC::NotEqual, found, zero);
                let over_b = self.b.create_block();
                let ins_b = self.b.create_block();
                let done = self.b.create_block();
                self.b.ins().brif(is_found, over_b, &[], ins_b, &[]);
                // Present key: drop the displaced value (drop-on-overwrite, like
                // `VecSet`), then move the new value into the slot.
                self.b.switch_to_block(over_b);
                let stridev = self.iconst(stride);
                let so = self.b.ins().imul(slot, stridev);
                let sb = self.b.ins().iadd(buf0, so);
                let voff = self.add_off(sb, 24);
                self.emit_drop(voff, valty, &[], &mut Vec::new());
                self.call_copy(voff, vaddr, vsz);
                self.b.ins().jump(done, &[]);
                // Absent key: grow/rehash if needed, own a key byte-copy, store.
                self.b.switch_to_block(ins_b);
                self.map_reserve(base, valty, *span);
                let buf = self.load_u64(base);
                let cap = self.load_u64(b16);
                let eslot = self.map_find_empty(buf, cap, stride, kptr, klen);
                let b24 = self.add_off(base, 24);
                let ctx = self.load_u64(b24);
                let b32 = self.add_off(base, 32);
                let vt = self.load_u64(b32);
                let kbuf = self.call_alloc(ctx, vt, klen, 1);
                let oom = self.b.ins().icmp(IntCC::Equal, kbuf, zero);
                self.fault_if(oom, FaultKind::Panic, *span);
                self.call_copy_val(kbuf, kptr, klen);
                let stridev2 = self.iconst(stride);
                let eo = self.b.ins().imul(eslot, stridev2);
                let eb = self.b.ins().iadd(buf, eo);
                self.store_u64(eb, one);
                let eb8 = self.add_off(eb, 8);
                self.store_u64(eb8, kbuf);
                let eb16 = self.add_off(eb, 16);
                self.store_u64(eb16, klen);
                let eb24 = self.add_off(eb, 24);
                self.call_copy(eb24, vaddr, vsz);
                let b8 = self.add_off(base, 8);
                let len = self.load_u64(b8);
                let nlen = self.b.ins().iadd(len, one);
                self.store_u64(b8, nlen);
                self.b.ins().jump(done, &[]);
                self.b.switch_to_block(done);
            }
            CollOp::MapContains { base, valty, key } => {
                let base = self.eval_operand(base, mf);
                let (kaddr, _) = self.place_addr(key, mf);
                let kptr = self.load_u64(kaddr);
                let ka8 = self.add_off(kaddr, 8);
                let klen = self.load_u64(ka8);
                let stride = crate::interp::mem::round_up(24 + self.size_of(valty), 8) as i64;
                let buf = self.load_u64(base);
                let b16 = self.add_off(base, 16);
                let cap = self.load_u64(b16);
                let (found, _slot) = self.map_find(buf, cap, stride, kptr, klen);
                let (daddr, _) = self.place_addr(dst, mf);
                self.store_scalar(daddr, found, ScalarTy::Bool);
            }
            CollOp::MapGet { base, valty, key, span } => {
                let base = self.eval_operand(base, mf);
                let (kaddr, _) = self.place_addr(key, mf);
                let kptr = self.load_u64(kaddr);
                let ka8 = self.add_off(kaddr, 8);
                let klen = self.load_u64(ka8);
                let stride = crate::interp::mem::round_up(24 + self.size_of(valty), 8) as i64;
                let buf = self.load_u64(base);
                let b16 = self.add_off(base, 16);
                let cap = self.load_u64(b16);
                let (found, slot) = self.map_find(buf, cap, stride, kptr, klen);
                let zero = self.iconst(0);
                let missing = self.b.ins().icmp(IntCC::Equal, found, zero);
                self.fault_if(missing, FaultKind::Bounds, *span);
                let stridev = self.iconst(stride);
                let so = self.b.ins().imul(slot, stridev);
                let sb = self.b.ins().iadd(buf, so);
                let voff = self.add_off(sb, 24);
                let (daddr, _) = self.place_addr(dst, mf);
                self.store_u64(daddr, voff);
            }
        }
    }

    /// 64-bit FNV-1a over the `klen` key bytes at candor `kptr` (offset basis
    /// 0xcbf29ce484222325, prime 0x100000001b3) — `mir::interp::map_hash` bit for bit.
    fn map_hash(&mut self, kptr: Value, klen: Value) -> Value {
        let basis = self.iconst(0xcbf2_9ce4_8422_2325u64 as i64);
        let prime = self.iconst(0x0000_0100_0000_01b3);
        let zero = self.iconst(0);
        let head = self.b.create_block();
        let hcur = self.b.append_block_param(head, types::I64);
        let icur = self.b.append_block_param(head, types::I64);
        let body = self.b.create_block();
        let done = self.b.create_block();
        let hout = self.b.append_block_param(done, types::I64);
        self.b.ins().jump(head, &[BlockArg::from(basis), BlockArg::from(zero)]);
        self.b.switch_to_block(head);
        let more = self.b.ins().icmp(IntCC::UnsignedLessThan, icur, klen);
        self.b.ins().brif(more, body, &[], done, &[BlockArg::from(hcur)]);
        self.b.switch_to_block(body);
        let bp = self.b.ins().iadd(kptr, icur);
        let byte = self.load_scalar(bp, ScalarTy::U8);
        let x = self.b.ins().bxor(hcur, byte);
        let m = self.b.ins().imul(x, prime);
        let one = self.iconst(1);
        let ni = self.b.ins().iadd(icur, one);
        self.b.ins().jump(head, &[BlockArg::from(m), BlockArg::from(ni)]);
        self.b.switch_to_block(done);
        hout
    }

    /// Whether the `len` bytes at candor `p1` and `p2` are equal (i64 0/1) — the
    /// byte compare `map_find` runs once the key lengths match.
    fn mem_eq(&mut self, p1: Value, p2: Value, len: Value) -> Value {
        let zero = self.iconst(0);
        let one = self.iconst(1);
        let head = self.b.create_block();
        let icur = self.b.append_block_param(head, types::I64);
        let cmp = self.b.create_block();
        let done = self.b.create_block();
        let res = self.b.append_block_param(done, types::I64);
        self.b.ins().jump(head, &[BlockArg::from(zero)]);
        self.b.switch_to_block(head);
        let more = self.b.ins().icmp(IntCC::UnsignedLessThan, icur, len);
        self.b.ins().brif(more, cmp, &[], done, &[BlockArg::from(one)]);
        self.b.switch_to_block(cmp);
        let a1 = self.b.ins().iadd(p1, icur);
        let b1 = self.b.ins().iadd(p2, icur);
        let av = self.load_scalar(a1, ScalarTy::U8);
        let bv = self.load_scalar(b1, ScalarTy::U8);
        let ne = self.b.ins().icmp(IntCC::NotEqual, av, bv);
        let inc = self.b.ins().iadd(icur, one);
        self.b.ins().brif(ne, done, &[BlockArg::from(zero)], head, &[BlockArg::from(inc)]);
        self.b.switch_to_block(done);
        res
    }

    /// Open-addressed linear probe for `key` in the bucket buffer: returns
    /// `(found, slot)` (found = i64 0/1), stopping at the first empty bucket — the
    /// `mir::interp::map_find` probe order (start `hash & (cap-1)`, step +1 & mask).
    fn map_find(&mut self, buf: Value, cap: Value, stride: i64, kptr: Value, klen: Value) -> (Value, Value) {
        let zero = self.iconst(0);
        let one = self.iconst(1);
        let cap0 = self.b.ins().icmp(IntCC::Equal, cap, zero);
        let bufz = self.b.ins().icmp(IntCC::Equal, buf, zero);
        let empty = self.b.ins().bor(cap0, bufz);
        let mask = self.b.ins().isub(cap, one);
        let hash = self.map_hash(kptr, klen);
        let idx0 = self.b.ins().band(hash, mask);
        let probe = self.b.create_block();
        let idxp = self.b.append_block_param(probe, types::I64);
        let done = self.b.create_block();
        let foundp = self.b.append_block_param(done, types::I64);
        let slotp = self.b.append_block_param(done, types::I64);
        self.b.ins().brif(
            empty,
            done,
            &[BlockArg::from(zero), BlockArg::from(zero)],
            probe,
            &[BlockArg::from(idx0)],
        );
        self.b.switch_to_block(probe);
        let stridev = self.iconst(stride);
        let off = self.b.ins().imul(idxp, stridev);
        let b = self.b.ins().iadd(buf, off);
        let state = self.load_u64(b);
        let is0 = self.b.ins().icmp(IntCC::Equal, state, zero);
        let checkk = self.b.create_block();
        self.b.ins().brif(
            is0,
            done,
            &[BlockArg::from(zero), BlockArg::from(zero)],
            checkk,
            &[],
        );
        self.b.switch_to_block(checkk);
        let b16 = self.add_off(b, 16);
        let slen = self.load_u64(b16);
        let leneq = self.b.ins().icmp(IntCC::Equal, slen, klen);
        let cmpb = self.b.create_block();
        let nextb = self.b.create_block();
        self.b.ins().brif(leneq, cmpb, &[], nextb, &[]);
        self.b.switch_to_block(cmpb);
        let b8 = self.add_off(b, 8);
        let sptr = self.load_u64(b8);
        let eq = self.mem_eq(sptr, kptr, klen);
        let iseq = self.b.ins().icmp(IntCC::NotEqual, eq, zero);
        self.b.ins().brif(
            iseq,
            done,
            &[BlockArg::from(one), BlockArg::from(idxp)],
            nextb,
            &[],
        );
        self.b.switch_to_block(nextb);
        let inc = self.b.ins().iadd(idxp, one);
        let nidx = self.b.ins().band(inc, mask);
        self.b.ins().jump(probe, &[BlockArg::from(nidx)]);
        self.b.switch_to_block(done);
        (foundp, slotp)
    }

    /// The first empty bucket along `key`'s probe chain (caller ensures `key`
    /// absent) — `mir::interp::map_find_empty`.
    fn map_find_empty(&mut self, buf: Value, cap: Value, stride: i64, kptr: Value, klen: Value) -> Value {
        let one = self.iconst(1);
        let zero = self.iconst(0);
        let mask = self.b.ins().isub(cap, one);
        let hash = self.map_hash(kptr, klen);
        let idx0 = self.b.ins().band(hash, mask);
        let probe = self.b.create_block();
        let idxp = self.b.append_block_param(probe, types::I64);
        let done = self.b.create_block();
        let slotp = self.b.append_block_param(done, types::I64);
        self.b.ins().jump(probe, &[BlockArg::from(idx0)]);
        self.b.switch_to_block(probe);
        let stridev = self.iconst(stride);
        let off = self.b.ins().imul(idxp, stridev);
        let b = self.b.ins().iadd(buf, off);
        let state = self.load_u64(b);
        let is0 = self.b.ins().icmp(IntCC::Equal, state, zero);
        let nextb = self.b.create_block();
        self.b.ins().brif(is0, done, &[BlockArg::from(idxp)], nextb, &[]);
        self.b.switch_to_block(nextb);
        let inc = self.b.ins().iadd(idxp, one);
        let nidx = self.b.ins().band(inc, mask);
        self.b.ins().jump(probe, &[BlockArg::from(nidx)]);
        self.b.switch_to_block(done);
        slotp
    }

    /// Zero `byte_len` bytes at candor `ptr` (a multiple of 8: the fresh bucket
    /// buffer, whose zero `state` words mark every bucket empty).
    fn zero_words(&mut self, ptr: Value, byte_len: Value) {
        let zero = self.iconst(0);
        let eight = self.iconst(8);
        let head = self.b.create_block();
        let icur = self.b.append_block_param(head, types::I64);
        let body = self.b.create_block();
        let done = self.b.create_block();
        self.b.ins().jump(head, &[BlockArg::from(zero)]);
        self.b.switch_to_block(head);
        let more = self.b.ins().icmp(IntCC::UnsignedLessThan, icur, byte_len);
        self.b.ins().brif(more, body, &[], done, &[]);
        self.b.switch_to_block(body);
        let p = self.b.ins().iadd(ptr, icur);
        self.store_u64(p, zero);
        let ni = self.b.ins().iadd(icur, eight);
        self.b.ins().jump(head, &[BlockArg::from(ni)]);
        self.b.switch_to_block(done);
    }

    /// Grow + rehash the bucket buffer (initial 8, then x2) once the load factor
    /// crosses 3/4, re-probing every live entry — `mir::interp::map_reserve`
    /// (alloc-new + zero + re-insert + free-old). OOM faults `Panic`.
    fn map_reserve(&mut self, base: Value, valty: &Type, span: Span) {
        let vsz = self.size_of(valty);
        let stride = crate::interp::mem::round_up(24 + vsz, 8) as i64;
        let zero = self.iconst(0);
        let one = self.iconst(1);
        let b8 = self.add_off(base, 8);
        let len = self.load_u64(b8);
        let b16 = self.add_off(base, 16);
        let cap = self.load_u64(b16);
        let capnz = self.b.ins().icmp(IntCC::NotEqual, cap, zero);
        let lp1 = self.b.ins().iadd(len, one);
        let four = self.iconst(4);
        let three = self.iconst(3);
        let lhs = self.b.ins().imul(lp1, four);
        let rhs = self.b.ins().imul(cap, three);
        let within = self.b.ins().icmp(IntCC::UnsignedLessThanOrEqual, lhs, rhs);
        let nogrow = self.b.ins().band(capnz, within);
        let grow = self.b.create_block();
        let cont = self.b.create_block();
        self.b.ins().brif(nogrow, cont, &[], grow, &[]);
        self.b.switch_to_block(grow);
        let capis0 = self.b.ins().icmp(IntCC::Equal, cap, zero);
        let eight = self.iconst(8);
        let two = self.iconst(2);
        let cap2 = self.b.ins().imul(cap, two);
        let newcap = self.b.ins().select(capis0, eight, cap2);
        let stridev = self.iconst(stride);
        let allocsz = self.b.ins().imul(newcap, stridev);
        let b24 = self.add_off(base, 24);
        let ctx = self.load_u64(b24);
        let b32 = self.add_off(base, 32);
        let vt = self.load_u64(b32);
        let newbuf = self.call_alloc(ctx, vt, allocsz, 8);
        let oom = self.b.ins().icmp(IntCC::Equal, newbuf, zero);
        self.fault_if(oom, FaultKind::Panic, span);
        self.zero_words(newbuf, allocsz);
        let oldbuf = self.load_u64(base);
        let hasold = self.b.ins().icmp(IntCC::NotEqual, oldbuf, zero);
        let rehash = self.b.create_block();
        let setnew = self.b.create_block();
        self.b.ins().brif(hasold, rehash, &[], setnew, &[]);
        self.b.switch_to_block(rehash);
        let oh = self.b.create_block();
        let ohi = self.b.append_block_param(oh, types::I64);
        let scanbody = self.b.create_block();
        let donescan = self.b.create_block();
        self.b.ins().jump(oh, &[BlockArg::from(zero)]);
        self.b.switch_to_block(oh);
        let more = self.b.ins().icmp(IntCC::UnsignedLessThan, ohi, cap);
        self.b.ins().brif(more, scanbody, &[], donescan, &[]);
        self.b.switch_to_block(scanbody);
        let obo = self.b.ins().imul(ohi, stridev);
        let ob = self.b.ins().iadd(oldbuf, obo);
        let ostate = self.load_u64(ob);
        let occ = self.b.ins().icmp(IntCC::Equal, ostate, one);
        let reins = self.b.create_block();
        let ohnext = self.b.create_block();
        self.b.ins().brif(occ, reins, &[], ohnext, &[]);
        self.b.switch_to_block(reins);
        let ob8 = self.add_off(ob, 8);
        let okptr = self.load_u64(ob8);
        let ob16 = self.add_off(ob, 16);
        let oklen = self.load_u64(ob16);
        let slot = self.map_find_empty(newbuf, newcap, stride, okptr, oklen);
        let nbo = self.b.ins().imul(slot, stridev);
        let nb = self.b.ins().iadd(newbuf, nbo);
        self.store_u64(nb, one);
        let nb8 = self.add_off(nb, 8);
        self.store_u64(nb8, okptr);
        let nb16 = self.add_off(nb, 16);
        self.store_u64(nb16, oklen);
        if vsz > 0 {
            let nb24 = self.add_off(nb, 24);
            let ob24 = self.add_off(ob, 24);
            self.call_copy(nb24, ob24, vsz);
        }
        self.b.ins().jump(ohnext, &[]);
        self.b.switch_to_block(ohnext);
        let inc = self.b.ins().iadd(ohi, one);
        self.b.ins().jump(oh, &[BlockArg::from(inc)]);
        self.b.switch_to_block(donescan);
        let capsz = self.b.ins().imul(cap, stridev);
        self.call_free_val(ctx, vt, oldbuf, capsz, 8);
        self.b.ins().jump(setnew, &[]);
        self.b.switch_to_block(setnew);
        self.store_u64(base, newbuf);
        self.store_u64(b16, newcap);
        self.b.ins().jump(cont, &[]);
        self.b.switch_to_block(cont);
    }

    /// Grow a `Vec`'s buffer to fit `need` more elements through the allocator's
    /// `realloc` (grow-in-place or move-copy), mirroring `mir::interp::vec_reserve`
    /// — element `stride`/`align` scale the byte sizes, `newcap =
    /// (len+need).max(cap*2).max(4)`, OOM faults `Panic`.
    fn vec_reserve(&mut self, base: Value, stride: i64, align: i64, need: i64, span: Span) {
        let b8 = self.add_off(base, 8);
        let len = self.load_u64(b8);
        let b16 = self.add_off(base, 16);
        let cap = self.load_u64(b16);
        let needv = self.iconst(need);
        let lenneed = self.b.ins().iadd(len, needv);
        let need_grow = self.b.ins().icmp(IntCC::UnsignedGreaterThan, lenneed, cap);
        let grow_b = self.b.create_block();
        let cont = self.b.create_block();
        self.b.ins().brif(need_grow, grow_b, &[], cont, &[]);

        self.b.switch_to_block(grow_b);
        let two = self.iconst(2);
        let cap2 = self.b.ins().imul(cap, two);
        let m1 = self.umax(lenneed, cap2);
        let four = self.iconst(4);
        let newcap = self.umax(m1, four);
        let stridev = self.iconst(stride);
        let allocsz = self.b.ins().imul(newcap, stridev);
        let b24 = self.add_off(base, 24);
        let ctx = self.load_u64(b24);
        let b32 = self.add_off(base, 32);
        let vt = self.load_u64(b32);
        let oldbuf = self.load_u64(base);
        let zero = self.iconst(0);
        let has_old = self.b.ins().icmp(IntCC::NotEqual, oldbuf, zero);
        let alloc_b = self.b.create_block();
        let realloc_b = self.b.create_block();
        let joined = self.b.create_block();
        let newbuf = self.b.append_block_param(joined, types::I64);
        self.b.ins().brif(has_old, realloc_b, &[], alloc_b, &[]);
        self.b.switch_to_block(alloc_b);
        let nb_a = self.call_alloc(ctx, vt, allocsz, align);
        self.b.ins().jump(joined, &[BlockArg::from(nb_a)]);
        self.b.switch_to_block(realloc_b);
        let oldsz = self.b.ins().imul(len, stridev);
        let nb_r = self.call_realloc(ctx, vt, oldbuf, oldsz, allocsz, align);
        self.b.ins().jump(joined, &[BlockArg::from(nb_r)]);
        self.b.switch_to_block(joined);
        let is_oom = self.b.ins().icmp(IntCC::Equal, newbuf, zero);
        self.fault_if(is_oom, FaultKind::Panic, span);
        self.store_u64(base, newbuf);
        self.store_u64(b16, newcap);
        self.b.ins().jump(cont, &[]);
        self.b.switch_to_block(cont);
    }

    /// Grow a `String`'s buffer to fit `need` more bytes through the allocator's
    /// `realloc` (grow-in-place or move-copy), mirroring `mir::interp::string_reserve`
    /// — `newcap = (len+need).max(cap*2).max(8)`, OOM faults `Panic`.
    fn string_reserve(&mut self, base: Value, need: Value, span: Span) {
        let b8 = self.add_off(base, 8);
        let len = self.load_u64(b8);
        let b16 = self.add_off(base, 16);
        let cap = self.load_u64(b16);
        let lenneed = self.b.ins().iadd(len, need);
        let need_grow = self.b.ins().icmp(IntCC::UnsignedGreaterThan, lenneed, cap);
        let grow_b = self.b.create_block();
        let cont = self.b.create_block();
        self.b.ins().brif(need_grow, grow_b, &[], cont, &[]);

        self.b.switch_to_block(grow_b);
        let two = self.iconst(2);
        let cap2 = self.b.ins().imul(cap, two);
        let m1 = self.umax(lenneed, cap2);
        let eight = self.iconst(8);
        let newcap = self.umax(m1, eight);
        let b24 = self.add_off(base, 24);
        let ctx = self.load_u64(b24);
        let b32 = self.add_off(base, 32);
        let vt = self.load_u64(b32);
        let oldbuf = self.load_u64(base);
        let zero = self.iconst(0);
        let has_old = self.b.ins().icmp(IntCC::NotEqual, oldbuf, zero);
        let alloc_b = self.b.create_block();
        let realloc_b = self.b.create_block();
        let joined = self.b.create_block();
        let newbuf = self.b.append_block_param(joined, types::I64);
        self.b.ins().brif(has_old, realloc_b, &[], alloc_b, &[]);
        self.b.switch_to_block(alloc_b);
        let nb_a = self.call_alloc(ctx, vt, newcap, 1);
        self.b.ins().jump(joined, &[BlockArg::from(nb_a)]);
        self.b.switch_to_block(realloc_b);
        let nb_r = self.call_realloc(ctx, vt, oldbuf, len, newcap, 1);
        self.b.ins().jump(joined, &[BlockArg::from(nb_r)]);
        self.b.switch_to_block(joined);
        let is_oom = self.b.ins().icmp(IntCC::Equal, newbuf, zero);
        self.fault_if(is_oom, FaultKind::Panic, span);
        self.store_u64(base, newbuf);
        self.store_u64(b16, newcap);
        self.b.ins().jump(cont, &[]);
        self.b.switch_to_block(cont);
    }

    /// Unsigned max via select (wrapping arithmetic only; matches the interp `.max`).
    fn umax(&mut self, a: Value, b: Value) -> Value {
        let gt = self.b.ins().icmp(IntCC::UnsignedGreaterThan, a, b);
        self.b.ins().select(gt, a, b)
    }

    /// Call the carried allocator's `alloc` vtable slot with a runtime `size`
    /// (mirrors `box_op`'s alloc dispatch; `align` is a small constant).
    fn call_alloc(&mut self, ctx: Value, vt: Value, size: Value, align: i64) -> Value {
        let alloc_off = self.field_off(&self.alloc_vtable_name(), "alloc");
        let aa = self.add_off(vt, alloc_off);
        let afn = self.load_u64(aa);
        let al = self.iconst(align);
        self.call_fnptr_id(afn, &[ctx, size, al])
    }

    /// Grow `ptr` (holding `old_size` bytes) to `new_size` through the vtable's
    /// `realloc` slot; returns the (possibly moved) pointer (0 on OOM).
    fn call_realloc(&mut self, ctx: Value, vt: Value, ptr: Value, old_size: Value, new_size: Value, align: i64) -> Value {
        let realloc_off = self.field_off(&self.alloc_vtable_name(), "realloc");
        let ra = self.add_off(vt, realloc_off);
        let rfn = self.load_u64(ra);
        let al = self.iconst(align);
        self.call_fnptr_id(rfn, &[ctx, ptr, old_size, new_size, al])
    }

    /// Free through the vtable with a runtime `size` (the String buffer capacity).
    fn call_free_val(&mut self, ctx: Value, vt: Value, ptr: Value, size: Value, align: i64) {
        let free_off = self.field_off(&self.alloc_vtable_name(), "free");
        let fa = self.add_off(vt, free_off);
        let ffn = self.load_u64(fa);
        let al = self.iconst(align);
        self.call_fnptr_id(ffn, &[ctx, ptr, size, al]);
    }

    /// Byte-copy with a runtime length (the `call_copy` counterpart for dynamic
    /// collection-buffer sizes).
    fn call_copy_val(&mut self, dst: Value, src: Value, len: Value) {
        let r = self.shimref("copy", self.shims.copy);
        self.b.ins().call(r, &[dst, src, len]);
    }

    /// Store the low byte of `val` at candor address `addr` (the UTF-8 byte writer).
    fn store_byte(&mut self, addr: Value, val: Value) {
        self.store_scalar(addr, val, ScalarTy::U8);
    }

    /// UTF-8 lead byte `prefix | (c >> shift)` (no mask: the scalar's high bits are
    /// already zero for the chosen width).
    fn utf8_lead(&mut self, c: Value, shift: i64, prefix: i64) -> Value {
        let s = self.iconst(shift);
        let sh = self.b.ins().ushr(c, s);
        let p = self.iconst(prefix);
        self.b.ins().bor(p, sh)
    }

    /// UTF-8 continuation byte `0x80 | ((c >> shift) & 0x3F)`.
    fn utf8_cont(&mut self, c: Value, shift: i64) -> Value {
        let s = self.iconst(shift);
        let sh = self.b.ins().ushr(c, s);
        let m = self.iconst(0x3F);
        let masked = self.b.ins().band(sh, m);
        let p = self.iconst(0x80);
        self.b.ins().bor(p, masked)
    }

    fn subslice_op(&mut self, dst: &Place, src: &Place, lo: &Operand, hi: &Operand, stride: u64, span: Span, mf: &MirFn) {
        let (saddr, _) = self.place_addr(src, mf);
        let ptr = self.load_u64(saddr);
        let s8 = self.add_off(saddr, 8);
        let len = self.load_u64(s8);
        let lo = self.eval_operand(lo, mf);
        let hi = self.eval_operand(hi, mf);
        let lo_gt_hi = self.b.ins().icmp(IntCC::UnsignedGreaterThan, lo, hi);
        let hi_gt_len = self.b.ins().icmp(IntCC::UnsignedGreaterThan, hi, len);
        let bad = self.b.ins().bor(lo_gt_hi, hi_gt_len);
        self.fault_if(bad, FaultKind::Bounds, span);
        let (daddr, _) = self.place_addr(dst, mf);
        let strc = self.iconst(stride as i64);
        let losc = self.b.ins().imul(lo, strc);
        let newptr = self.b.ins().iadd(ptr, losc);
        self.store_u64(daddr, newptr);
        let newlen = self.b.ins().isub(hi, lo);
        let d8 = self.add_off(daddr, 8);
        self.store_u64(d8, newlen);
    }

    /// `str_from(b) -> Utf8Res` (design 0013 §4): scan `src`'s `[u8]` bytes for
    /// UTF-8 well-formedness (the standard lead-class / continuation / overlong /
    /// surrogate / range checks of `str::from_utf8`), building `Utf8Res::Valid(str)`
    /// (the same `{ptr, len}` fat pointer, retyped) or `Utf8Res::Invalid(offset)`.
    /// The `offset` is the START of the first ill-formed sequence — exactly the
    /// `valid_up_to()` the interpreter oracle reports (byte-exact). No fault.
    fn str_from_op(&mut self, dst: &Place, src: &Place, mf: &MirFn) {
        let (saddr, _) = self.place_addr(src, mf);
        let ptr = self.load_u64(saddr);
        let s8 = self.add_off(saddr, 8);
        let len = self.load_u64(s8);
        let (daddr, _) = self.place_addr(dst, mf);
        let ures = Type::Named("Utf8Res".to_string());
        let einfo = self.lay().enum_info(&ures).expect("unknown enum `Utf8Res`");
        let valid_idx = einfo.iter().position(|(n, _)| n == "Valid").unwrap_or(0);
        let invalid_idx = einfo.iter().position(|(n, _)| n == "Invalid").unwrap_or(1);
        let (_, valid_off) = self.lay().payload_offset(&einfo[valid_idx].1, 0);
        let (_, invalid_off) = self.lay().payload_offset(&einfo[invalid_idx].1, 0);

        let head = self.b.create_block();
        let idx = self.b.append_block_param(head, types::I64);
        let body = self.b.create_block();
        let adv1 = self.b.create_block();
        let notascii = self.b.create_block();
        let m2 = self.b.create_block();
        let m2b = self.b.create_block();
        let adv2 = self.b.create_block();
        let not2 = self.b.create_block();
        let m3 = self.b.create_block();
        let m3b = self.b.create_block();
        let m3c = self.b.create_block();
        let adv3 = self.b.create_block();
        let not3 = self.b.create_block();
        let m4 = self.b.create_block();
        let m4b = self.b.create_block();
        let m4c = self.b.create_block();
        let m4d = self.b.create_block();
        let adv4 = self.b.create_block();
        let invalid = self.b.create_block();
        let valid_exit = self.b.create_block();
        let done = self.b.create_block();

        let zero = self.iconst(0);
        // Hoist the small step constants so they dominate every branch's offset math.
        let one = self.iconst(1);
        let two = self.iconst(2);
        let three = self.iconst(3);
        let four = self.iconst(4);
        self.b.ins().jump(head, &[BlockArg::from(zero)]);
        // head(idx): loop while idx < len.
        self.b.switch_to_block(head);
        let more = self.b.ins().icmp(IntCC::UnsignedLessThan, idx, len);
        self.b.ins().brif(more, body, &[], valid_exit, &[]);
        // body: decode the sequence starting at idx.
        self.b.switch_to_block(body);
        let addr0 = self.b.ins().iadd(ptr, idx);
        let b0 = self.load_scalar(addr0, ScalarTy::U8);
        let c80 = self.iconst(0x80);
        let ascii = self.b.ins().icmp(IntCC::UnsignedLessThan, b0, c80);
        self.b.ins().brif(ascii, adv1, &[], notascii, &[]);
        // adv1: one ASCII byte consumed.
        self.b.switch_to_block(adv1);
        let n1 = self.b.ins().iadd(idx, one);
        self.b.ins().jump(head, &[BlockArg::from(n1)]);
        // notascii: classify a 2-byte lead (0xC2..=0xDF).
        self.b.switch_to_block(notascii);
        let is_w2 = self.byte_in_range(b0, 0xC2, 0xDF);
        self.b.ins().brif(is_w2, m2, &[], not2, &[]);
        // m2: need one continuation byte present.
        self.b.switch_to_block(m2);
        let p1 = self.b.ins().iadd(idx, one);
        let pres2 = self.b.ins().icmp(IntCC::UnsignedLessThan, p1, len);
        self.b.ins().brif(pres2, m2b, &[], invalid, &[]);
        self.b.switch_to_block(m2b);
        let a1 = self.b.ins().iadd(ptr, p1);
        let b1 = self.load_scalar(a1, ScalarTy::U8);
        let cont2 = self.byte_is_cont(b1);
        self.b.ins().brif(cont2, adv2, &[], invalid, &[]);
        self.b.switch_to_block(adv2);
        let n2 = self.b.ins().iadd(idx, two);
        self.b.ins().jump(head, &[BlockArg::from(n2)]);
        // not2: classify a 3-byte lead (0xE0..=0xEF).
        self.b.switch_to_block(not2);
        let is_w3 = self.byte_in_range(b0, 0xE0, 0xEF);
        self.b.ins().brif(is_w3, m3, &[], not3, &[]);
        self.b.switch_to_block(m3);
        let p2 = self.b.ins().iadd(idx, two);
        let pres3 = self.b.ins().icmp(IntCC::UnsignedLessThan, p2, len);
        self.b.ins().brif(pres3, m3b, &[], invalid, &[]);
        self.b.switch_to_block(m3b);
        let p1_3 = self.b.ins().iadd(idx, one);
        let a1_3 = self.b.ins().iadd(ptr, p1_3);
        let b1_3 = self.load_scalar(a1_3, ScalarTy::U8);
        // Second-byte range: E0 -> A0..BF, ED -> 80..9F, else 80..BF.
        let is_e0 = self.icmp_eq_imm(b0, 0xE0);
        let is_ed = self.icmp_eq_imm(b0, 0xED);
        let c80v = self.iconst(0x80);
        let ca0 = self.iconst(0xA0);
        let cbf = self.iconst(0xBF);
        let c9f = self.iconst(0x9F);
        let lo3 = self.b.ins().select(is_e0, ca0, c80v);
        let hi3 = self.b.ins().select(is_ed, c9f, cbf);
        let ge_lo3 = self.b.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, b1_3, lo3);
        let le_hi3 = self.b.ins().icmp(IntCC::UnsignedLessThanOrEqual, b1_3, hi3);
        let ok2_3 = self.b.ins().band(ge_lo3, le_hi3);
        self.b.ins().brif(ok2_3, m3c, &[], invalid, &[]);
        self.b.switch_to_block(m3c);
        let a2_3 = self.b.ins().iadd(ptr, p2);
        let b2_3 = self.load_scalar(a2_3, ScalarTy::U8);
        let cont3 = self.byte_is_cont(b2_3);
        self.b.ins().brif(cont3, adv3, &[], invalid, &[]);
        self.b.switch_to_block(adv3);
        let n3 = self.b.ins().iadd(idx, three);
        self.b.ins().jump(head, &[BlockArg::from(n3)]);
        // not3: classify a 4-byte lead (0xF0..=0xF4); anything else is an invalid lead.
        self.b.switch_to_block(not3);
        let is_w4 = self.byte_in_range(b0, 0xF0, 0xF4);
        self.b.ins().brif(is_w4, m4, &[], invalid, &[]);
        self.b.switch_to_block(m4);
        let p3 = self.b.ins().iadd(idx, three);
        let pres4 = self.b.ins().icmp(IntCC::UnsignedLessThan, p3, len);
        self.b.ins().brif(pres4, m4b, &[], invalid, &[]);
        self.b.switch_to_block(m4b);
        let p1_4 = self.b.ins().iadd(idx, one);
        let a1_4 = self.b.ins().iadd(ptr, p1_4);
        let b1_4 = self.load_scalar(a1_4, ScalarTy::U8);
        // Second-byte range: F0 -> 90..BF, F4 -> 80..8F, else 80..BF.
        let is_f0 = self.icmp_eq_imm(b0, 0xF0);
        let is_f4 = self.icmp_eq_imm(b0, 0xF4);
        let c80w = self.iconst(0x80);
        let c90 = self.iconst(0x90);
        let cbfw = self.iconst(0xBF);
        let c8f = self.iconst(0x8F);
        let lo4 = self.b.ins().select(is_f0, c90, c80w);
        let hi4 = self.b.ins().select(is_f4, c8f, cbfw);
        let ge_lo4 = self.b.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, b1_4, lo4);
        let le_hi4 = self.b.ins().icmp(IntCC::UnsignedLessThanOrEqual, b1_4, hi4);
        let ok2_4 = self.b.ins().band(ge_lo4, le_hi4);
        self.b.ins().brif(ok2_4, m4c, &[], invalid, &[]);
        self.b.switch_to_block(m4c);
        let p2_4 = self.b.ins().iadd(idx, two);
        let a2_4 = self.b.ins().iadd(ptr, p2_4);
        let b2_4 = self.load_scalar(a2_4, ScalarTy::U8);
        let cont4b = self.byte_is_cont(b2_4);
        self.b.ins().brif(cont4b, m4d, &[], invalid, &[]);
        self.b.switch_to_block(m4d);
        let a3_4 = self.b.ins().iadd(ptr, p3);
        let b3_4 = self.load_scalar(a3_4, ScalarTy::U8);
        let cont4c = self.byte_is_cont(b3_4);
        self.b.ins().brif(cont4c, adv4, &[], invalid, &[]);
        self.b.switch_to_block(adv4);
        let n4 = self.b.ins().iadd(idx, four);
        self.b.ins().jump(head, &[BlockArg::from(n4)]);
        // invalid(idx): build `Utf8Res::Invalid(idx)` (idx = first bad-sequence start).
        self.b.switch_to_block(invalid);
        let inv_tag = self.iconst(invalid_idx as i64);
        self.store_u64(daddr, inv_tag);
        let inv_slot = self.add_off(daddr, invalid_off);
        self.store_u64(inv_slot, idx);
        self.b.ins().jump(done, &[]);
        // valid_exit: build `Utf8Res::Valid(str)` — the same `{ptr, len}` view.
        self.b.switch_to_block(valid_exit);
        let val_tag = self.iconst(valid_idx as i64);
        self.store_u64(daddr, val_tag);
        let val_slot = self.add_off(daddr, valid_off);
        self.store_u64(val_slot, ptr);
        let val_slot8 = self.add_off(daddr, valid_off + 8);
        self.store_u64(val_slot8, len);
        self.b.ins().jump(done, &[]);
        self.b.switch_to_block(done);
    }

    /// `b in [lo, hi]` (unsigned) for an already-zero-extended byte value.
    fn byte_in_range(&mut self, b: Value, lo: i64, hi: i64) -> Value {
        let lov = self.iconst(lo);
        let hiv = self.iconst(hi);
        let ge = self.b.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, b, lov);
        let le = self.b.ins().icmp(IntCC::UnsignedLessThanOrEqual, b, hiv);
        self.b.ins().band(ge, le)
    }
    /// `b == imm`.
    fn icmp_eq_imm(&mut self, b: Value, imm: i64) -> Value {
        let v = self.iconst(imm);
        self.b.ins().icmp(IntCC::Equal, b, v)
    }
    /// Is `b` a UTF-8 continuation byte (`b & 0xC0 == 0x80`)?
    fn byte_is_cont(&mut self, b: Value) -> Value {
        let mask = self.iconst(0xC0);
        let masked = self.b.ins().band(b, mask);
        let c80 = self.iconst(0x80);
        self.b.ins().icmp(IntCC::Equal, masked, c80)
    }

    /// `substr(s, lo, hi) -> str` (design 0013 §3): the `[lo, hi)` byte sub-view,
    /// faulting `Bounds` at `span` on `lo > hi || hi > len` OR when `lo`/`hi` is not
    /// a UTF-8 character boundary. Mirrors the interpreter `bi_substr` byte-for-byte.
    fn substr_op(&mut self, dst: &Place, src: &Place, lo: &Operand, hi: &Operand, span: Span, mf: &MirFn) {
        let (saddr, _) = self.place_addr(src, mf);
        let ptr = self.load_u64(saddr);
        let s8 = self.add_off(saddr, 8);
        let len = self.load_u64(s8);
        let lo = self.eval_operand(lo, mf);
        let hi = self.eval_operand(hi, mf);
        let lo_gt_hi = self.b.ins().icmp(IntCC::UnsignedGreaterThan, lo, hi);
        let hi_gt_len = self.b.ins().icmp(IntCC::UnsignedGreaterThan, hi, len);
        let oob = self.b.ins().bor(lo_gt_hi, hi_gt_len);
        self.fault_if(oob, FaultKind::Bounds, span);
        // Char-boundary check for lo and hi (a non-boundary continuation byte faults).
        let lo_bad = self.boundary_bad(ptr, lo, len);
        let hi_bad = self.boundary_bad(ptr, hi, len);
        let bad = self.b.ins().bor(lo_bad, hi_bad);
        self.fault_if(bad, FaultKind::Bounds, span);
        let (daddr, _) = self.place_addr(dst, mf);
        let newptr = self.b.ins().iadd(ptr, lo);
        self.store_u64(daddr, newptr);
        let newlen = self.b.ins().isub(hi, lo);
        let d8 = self.add_off(daddr, 8);
        self.store_u64(d8, newlen);
    }

    /// Is byte offset `i` NOT a char boundary of the run at `ptr` (len `len`)?
    /// `i == 0 || i == len` is always a boundary; otherwise the byte at `i` must
    /// not be a continuation byte. Returns an i1 (true = fault). Guards the load so
    /// `i == len` never reads past the run.
    fn boundary_bad(&mut self, ptr: Value, i: Value, len: Value) -> Value {
        let zero = self.iconst(0);
        let is0 = self.b.ins().icmp(IntCC::Equal, i, zero);
        let islen = self.b.ins().icmp(IntCC::Equal, i, len);
        let skip = self.b.ins().bor(is0, islen);
        let load_b = self.b.create_block();
        let merge = self.b.create_block();
        let bad = self.b.append_block_param(merge, types::I8);
        // The `skip` (boundary) path carries `false` (an I8 bool, matching `icmp`).
        let f = self.b.ins().iconst(types::I8, 0);
        self.b.ins().brif(skip, merge, &[BlockArg::from(f)], load_b, &[]);
        self.b.switch_to_block(load_b);
        let a = self.b.ins().iadd(ptr, i);
        let byte = self.load_scalar(a, ScalarTy::U8);
        let cont = self.byte_is_cont(byte);
        self.b.ins().jump(merge, &[BlockArg::from(cont)]);
        self.b.switch_to_block(merge);
        bad
    }

    /// Drop a Box pointee at `addr` by calling its synthesized glue function
    /// (runtime recursion; terminates on `ptr == 0`).
    fn call_drop_glue(&mut self, addr: Value, inner: &Type) {
        if !self.needs_drop(inner) {
            return;
        }
        let key = type_key(inner);
        if let Some(id) = self.glue_ids.get(&key).copied() {
            let r = self.module.declare_func_in_func(id, self.b.func);
            self.b.ins().call(r, &[addr]);
        }
    }

    /// Body of a drop-glue function: drop the value of `ty` at the address param.
    fn lower_glue(&mut self, ty: &Type) {
        let entry = self.b.create_block();
        self.b.switch_to_block(entry);
        self.b.append_block_params_for_function_params(entry);
        let addr = self.b.block_params(entry)[0];
        self.emit_drop(addr, ty, &[], &mut Vec::new());
        let z = self.iconst(0);
        self.b.ins().return_(&[z]);
    }

    // ---- static drop schedule (INV-DROP) ----
    fn needs_drop(&self, ty: &Type) -> bool {
        match ty {
            Type::Array(e, _) => self.needs_drop(e),
            Type::Box(_) | Type::BoxResult(_) => true,
            // Compiler-known collections own a heap buffer freed on drop.
            Type::App(n, _) if n == "Vec" || n == "Map" => true,
            Type::Named(n) if n == "String" => true,
            Type::Named(n) if self.items.lookup_struct(n).is_some() => {
                if self.prog.drop_hooks.contains_key(n) {
                    return true;
                }
                let (fields, _, _) = self.lay().struct_layout(n);
                fields.iter().any(|(_, t, _)| self.needs_drop(t))
            }
            Type::Named(n) if self.items.lookup_enum(n).is_some() => {
                match self.lay().enum_info(ty) {
                    Some(vs) => vs.iter().any(|(_, ps)| ps.iter().any(|p| self.needs_drop(p))),
                    None => false,
                }
            }
            _ => false,
        }
    }

    fn emit_drop(&mut self, addr: Value, ty: &Type, moved: &[Vec<String>], path: &mut Vec<String>) {
        if is_moved(moved, path) || !self.needs_drop(ty) {
            return;
        }
        match ty {
            Type::Array(elem, len) => {
                let n = self.lay().array_len(len);
                let stride = crate::interp::mem::round_up(self.size_of(elem), self.align_of(elem));
                for i in (0..n).rev() {
                    let ea = self.add_off(addr, i * stride);
                    self.emit_drop(ea, elem, moved, path);
                }
            }
            Type::Box(inner) => self.drop_box(addr, inner),
            Type::BoxResult(_) => self.drop_enum(addr, ty, moved, path),
            // Compiler-known collections: drop live elements/values, free the
            // buffer through the carried allocator (mirror of `mir::interp`).
            // Precedes the struct arm — `String` is a synthesized nominal struct.
            Type::App(n, args) if n == "Vec" => {
                let elem = args.first().cloned().unwrap_or(Type::Error);
                self.drop_vec(addr, &elem);
            }
            Type::App(n, args) if n == "Map" => {
                let valty = args.first().cloned().unwrap_or(Type::Error);
                self.drop_map(addr, &valty);
            }
            Type::Named(n) if n == "String" => self.drop_string(addr),
            Type::Named(n) if self.items.lookup_struct(n).is_some() => {
                let partial = partially(moved, path);
                if !partial {
                    if let Some(hook) = self.prog.drop_hooks.get(n).cloned() {
                        let r = self.callref(&hook);
                        self.b.ins().call(r, &[addr]);
                    }
                }
                let (fields, _, _) = self.lay().struct_layout(n);
                for (fname, fty, off) in fields.into_iter().rev() {
                    path.push(fname);
                    let fa = self.add_off(addr, off);
                    self.emit_drop(fa, &fty, moved, path);
                    path.pop();
                }
            }
            Type::Named(_) => self.drop_enum(addr, ty, moved, path),
            _ => {}
        }
    }

    fn drop_box(&mut self, addr: Value, inner: &Type) {
        let ptr = self.load_u64(addr);
        let a8 = self.add_off(addr, 8);
        let ctx = self.load_u64(a8);
        let a16 = self.add_off(addr, 16);
        let vt = self.load_u64(a16);
        let zero = self.iconst(0);
        let nz = self.b.ins().icmp(IntCC::NotEqual, ptr, zero);
        let dob = self.b.create_block();
        let cont = self.b.create_block();
        self.b.ins().brif(nz, dob, &[], cont, &[]);
        self.b.switch_to_block(dob);
        self.call_drop_glue(ptr, inner);
        let size = self.size_of(inner);
        let align = self.align_of(inner);
        self.call_free(ctx, vt, ptr, size, align);
        self.b.ins().jump(cont, &[]);
        self.b.switch_to_block(cont);
    }

    /// Drop a compiler-known `Vec[T]` at `addr` (mirror of `mir::interp` drop_value):
    /// with a non-null buffer, drop each live element in reverse index order, then
    /// free the buffer through the carried allocator (`cap * stride` bytes, elem align).
    fn drop_vec(&mut self, addr: Value, elem: &Type) {
        let buf = self.load_u64(addr);
        let zero = self.iconst(0);
        let nz = self.b.ins().icmp(IntCC::NotEqual, buf, zero);
        let dob = self.b.create_block();
        let cont = self.b.create_block();
        self.b.ins().brif(nz, dob, &[], cont, &[]);
        self.b.switch_to_block(dob);
        let stride = crate::interp::mem::round_up(self.size_of(elem), self.align_of(elem));
        let align = self.align_of(elem);
        let stridev = self.iconst(stride as i64);
        if self.needs_drop(elem) {
            let a8 = self.add_off(addr, 8);
            let len = self.load_u64(a8);
            let head = self.b.create_block();
            let icur = self.b.append_block_param(head, types::I64);
            let body = self.b.create_block();
            let after = self.b.create_block();
            self.b.ins().jump(head, &[BlockArg::from(len)]);
            self.b.switch_to_block(head);
            let more = self.b.ins().icmp(IntCC::UnsignedGreaterThan, icur, zero);
            self.b.ins().brif(more, body, &[], after, &[]);
            self.b.switch_to_block(body);
            let one = self.iconst(1);
            let idx = self.b.ins().isub(icur, one);
            let off = self.b.ins().imul(idx, stridev);
            let ea = self.b.ins().iadd(buf, off);
            self.emit_drop(ea, elem, &[], &mut Vec::new());
            self.b.ins().jump(head, &[BlockArg::from(idx)]);
            self.b.switch_to_block(after);
        }
        let a16 = self.add_off(addr, 16);
        let cap = self.load_u64(a16);
        let a24 = self.add_off(addr, 24);
        let ctx = self.load_u64(a24);
        let a32 = self.add_off(addr, 32);
        let vt = self.load_u64(a32);
        let size = self.b.ins().imul(cap, stridev);
        self.call_free_val(ctx, vt, buf, size, align as i64);
        self.b.ins().jump(cont, &[]);
        self.b.switch_to_block(cont);
    }

    /// Drop a compiler-known `Map[V]` at `addr` (mirror of `mir::interp` drop_value):
    /// with a non-null buffer, for each occupied slot free its owned key bytes then
    /// drop its value, then free the bucket buffer (`cap * stride` bytes, align 8).
    fn drop_map(&mut self, addr: Value, valty: &Type) {
        let buf = self.load_u64(addr);
        let zero = self.iconst(0);
        let nz = self.b.ins().icmp(IntCC::NotEqual, buf, zero);
        let dob = self.b.create_block();
        let cont = self.b.create_block();
        self.b.ins().brif(nz, dob, &[], cont, &[]);
        self.b.switch_to_block(dob);
        let stride = crate::interp::mem::round_up(24 + self.size_of(valty), 8);
        let stridev = self.iconst(stride as i64);
        let a16 = self.add_off(addr, 16);
        let cap = self.load_u64(a16);
        let a24 = self.add_off(addr, 24);
        let ctx = self.load_u64(a24);
        let a32 = self.add_off(addr, 32);
        let vt = self.load_u64(a32);
        let one = self.iconst(1);
        let head = self.b.create_block();
        let icur = self.b.append_block_param(head, types::I64);
        let body = self.b.create_block();
        let after = self.b.create_block();
        self.b.ins().jump(head, &[BlockArg::from(zero)]);
        self.b.switch_to_block(head);
        let more = self.b.ins().icmp(IntCC::UnsignedLessThan, icur, cap);
        self.b.ins().brif(more, body, &[], after, &[]);
        self.b.switch_to_block(body);
        let off = self.b.ins().imul(icur, stridev);
        let b = self.b.ins().iadd(buf, off);
        let occ = self.load_u64(b);
        let isocc = self.b.ins().icmp(IntCC::Equal, occ, one);
        let slotb = self.b.create_block();
        let next = self.b.create_block();
        self.b.ins().brif(isocc, slotb, &[], next, &[]);
        self.b.switch_to_block(slotb);
        let b8 = self.add_off(b, 8);
        let kptr = self.load_u64(b8);
        let b16 = self.add_off(b, 16);
        let klen = self.load_u64(b16);
        self.call_free_val(ctx, vt, kptr, klen, 1);
        let b24 = self.add_off(b, 24);
        self.emit_drop(b24, valty, &[], &mut Vec::new());
        self.b.ins().jump(next, &[]);
        self.b.switch_to_block(next);
        let inc = self.b.ins().iadd(icur, one);
        self.b.ins().jump(head, &[BlockArg::from(inc)]);
        self.b.switch_to_block(after);
        let size = self.b.ins().imul(cap, stridev);
        self.call_free_val(ctx, vt, buf, size, 8);
        self.b.ins().jump(cont, &[]);
        self.b.switch_to_block(cont);
    }

    /// Drop a compiler-known `String` at `addr` (mirror of `mir::interp` drop_value):
    /// free its UTF-8 buffer through the carried allocator (`cap` bytes, align 1) when
    /// the buffer is non-null. Bytes are POD, so there are no element drops.
    fn drop_string(&mut self, addr: Value) {
        let buf = self.load_u64(addr);
        let zero = self.iconst(0);
        let nz = self.b.ins().icmp(IntCC::NotEqual, buf, zero);
        let dob = self.b.create_block();
        let cont = self.b.create_block();
        self.b.ins().brif(nz, dob, &[], cont, &[]);
        self.b.switch_to_block(dob);
        let a16 = self.add_off(addr, 16);
        let cap = self.load_u64(a16);
        let a24 = self.add_off(addr, 24);
        let ctx = self.load_u64(a24);
        let a32 = self.add_off(addr, 32);
        let vt = self.load_u64(a32);
        self.call_free_val(ctx, vt, buf, cap, 1);
        self.b.ins().jump(cont, &[]);
        self.b.switch_to_block(cont);
    }

    fn drop_enum(&mut self, addr: Value, ty: &Type, moved: &[Vec<String>], path: &mut Vec<String>) {
        let einfo = match self.lay().enum_info(ty) {
            Some(e) => e,
            None => return,
        };
        let tag = self.load_u64(addr);
        let merge = self.b.create_block();
        for (idx, (_, payloads)) in einfo.iter().enumerate() {
            if !payloads.iter().any(|p| self.needs_drop(p)) {
                continue;
            }
            let idxc = self.iconst(idx as i64);
            let is_v = self.b.ins().icmp(IntCC::Equal, tag, idxc);
            let vblock = self.b.create_block();
            let next = self.b.create_block();
            self.b.ins().brif(is_v, vblock, &[], next, &[]);
            self.b.switch_to_block(vblock);
            for i in (0..payloads.len()).rev() {
                let (pty, off) = self.lay().payload_offset(payloads, i);
                path.push(format!("_{i}"));
                let pa = self.add_off(addr, off);
                self.emit_drop(pa, &pty, moved, path);
                path.pop();
            }
            self.b.ins().jump(merge, &[]);
            self.b.switch_to_block(next);
        }
        self.b.ins().jump(merge, &[]);
        self.b.switch_to_block(merge);
    }
}

fn is_moved(mask: &[Vec<String>], path: &[String]) -> bool {
    mask.iter().any(|m| prefix(m, path))
}
fn partially(mask: &[Vec<String>], path: &[String]) -> bool {
    mask.iter().any(|m| m.len() > path.len() && m[..path.len()] == path[..])
}
fn prefix(a: &[String], b: &[String]) -> bool {
    a.len() <= b.len() && a[..] == b[..a.len()]
}


/// A canonical key for a monomorphized type (used to dedupe drop glue).
fn type_key(ty: &Type) -> String {
    format!("{ty:?}")
}

/// Collect every `Box` pointee type that needs a drop-glue function: walk every
/// droppable type (drop-obligation locals, `BoxOp` inners) and gather each
/// distinct `Box(inner)` where `inner` needs drop, recursing through `inner` —
/// deduped by key so recursive Box types terminate.
pub(super) fn collect_glue_types(prog: &MirProgram, items: &Items, consts: &HashMap<String, u64>) -> Vec<Type> {
    let lay = Layout { items, consts };
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    let mut roots: Vec<Type> = Vec::new();
    for f in &prog.fns {
        for l in &f.locals {
            if l.drop_obligation {
                roots.push(l.ty.clone());
            }
        }
        for b in &f.blocks {
            for st in &b.stmts {
                if let StatementKind::BoxOp { inner_ty, .. } = &st.kind {
                    roots.push(inner_ty.clone());
                }
            }
        }
    }
    for r in roots {
        walk_glue(&r, &lay, items, prog, &mut seen, &mut out);
    }
    out
}

fn walk_glue(
    ty: &Type,
    lay: &Layout,
    items: &Items,
    prog: &MirProgram,
    seen: &mut std::collections::HashSet<String>,
    out: &mut Vec<Type>,
) {
    match ty {
        Type::Box(inner) => {
            if type_needs_drop(inner, lay, items, prog) {
                let key = type_key(inner);
                if seen.insert(key) {
                    out.push((**inner).clone());
                    walk_glue(inner, lay, items, prog, seen, out);
                }
            }
        }
        Type::BoxResult(t) => {
            let bx = Type::Box(Box::new((**t).clone()));
            walk_glue(&bx, lay, items, prog, seen, out);
        }
        Type::Array(e, _) => walk_glue(e, lay, items, prog, seen, out),
        Type::App(n, args) if n == "Vec" || n == "Map" => {
            for a in args {
                walk_glue(a, lay, items, prog, seen, out);
            }
        }
        Type::Named(n) if items.lookup_struct(n).is_some() => {
            let (fields, _, _) = lay.struct_layout(n);
            for (_, fty, _) in fields {
                walk_glue(&fty, lay, items, prog, seen, out);
            }
        }
        Type::Named(_) => {
            if let Some(vs) = lay.enum_info(ty) {
                for (_, ps) in vs {
                    for p in ps {
                        walk_glue(&p, lay, items, prog, seen, out);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Standalone `needs_drop` (mirrors `Cg::needs_drop`) for the collection pass.
fn type_needs_drop(ty: &Type, lay: &Layout, items: &Items, prog: &MirProgram) -> bool {
    match ty {
        Type::Array(e, _) => type_needs_drop(e, lay, items, prog),
        Type::Box(_) | Type::BoxResult(_) => true,
        Type::App(n, _) if n == "Vec" || n == "Map" => true,
        Type::Named(n) if n == "String" => true,
        Type::Named(n) if items.lookup_struct(n).is_some() => {
            if prog.drop_hooks.contains_key(n) {
                return true;
            }
            let (fields, _, _) = lay.struct_layout(n);
            fields.iter().any(|(_, t, _)| type_needs_drop(t, lay, items, prog))
        }
        Type::Named(n) if items.lookup_enum(n).is_some() => match lay.enum_info(ty) {
            Some(vs) => vs.iter().any(|(_, ps)| ps.iter().any(|p| type_needs_drop(p, lay, items, prog))),
            None => false,
        },
        _ => false,
    }
}
