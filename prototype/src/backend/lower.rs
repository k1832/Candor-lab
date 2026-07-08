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

use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{
    types, AbiParam, Block, FuncRef, InstBuilder, MemFlags, Signature, TrapCode, UserFuncName, Value,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, DataId, FuncId, Linkage, Module};

use crate::interp::layout::Layout;
use crate::interp::FaultKind;
use crate::ast::{BinOp, UnOp};
use crate::mir::{
    FaultEdge, MirFn, MirProgram, Operand, Place, Proj, Regime, Rvalue, Statement, StatementKind,
    Terminator,
};
use crate::resolve::Items;
use crate::span::Span;
use crate::token::ScalarTy;
use crate::types::{ItemEnv, Type};

use super::runtime;

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
        // Foreign calls are not yet lowered by the native backend (0011 §5, a
        // 0010 forward dependency); the code exists only for match totality.
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
                callrefs: HashMap::new(),
                shimrefs: HashMap::new(),
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
                callrefs: HashMap::new(),
                shimrefs: HashMap::new(),
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
    let main_is_i64 = matches!(
        prog.get("main").map(|f| &f.locals[0].ty),
        Some(Type::Scalar(ScalarTy::I64))
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
            callrefs: HashMap::new(),
            shimrefs: HashMap::new(),
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
    let mut module = JITModule::new(builder);

    let shims = Shims::declare(&mut module)?;
    let glue_types = collect_glue_types(prog, items, consts);
    let (func_ids, glue_ids) = declare_functions(&mut module, prog, &glue_types, "")?;
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

        Ok(Shims { stack_alloc, copy, trace, mmio_load, mmio_store, fault })
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
    callrefs: HashMap<String, FuncRef>,
    shimrefs: HashMap<&'static str, FuncRef>,
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
                let x128 = self.ext128(x, sty);
                self.range_or_fit(x128, *to, *regime, fault.as_ref())
            }
            Rvalue::Call { func, args } => {
                let vals: Vec<Value> = args.iter().map(|a| self.eval_operand(a, mf)).collect();
                let r = self.callref(func);
                let c = self.b.ins().call(r, &vals);
                self.b.inst_results(c)[0]
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

    fn eval_bin(&mut self, op: BinOp, regime: Regime, ty: ScalarTy, l: &Operand, r: &Operand, span: Span, fault: Option<&FaultEdge>, mf: &MirFn) -> Value {
        use BinOp::*;
        let lv = self.eval_operand(l, mf);
        let rv = self.eval_operand(r, mf);
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
