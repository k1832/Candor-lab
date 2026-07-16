//! The MIR interpreter (design 0010 §5): a *precise* execution engine over the
//! checked MIR — a different engine from the tree-walking oracle, sharing the
//! same semantics *and the same memory substrate*. Every fault edge is taken
//! immediately (zero-width window, R1–R3-free): the `density → 1` precise limit
//! of the formalization §10.
//!
//! ## Memory-substrate integration (design 0010 §2/§5, Stage A2/A3)
//! Every runtime value lives in a flat byte-addressable [`crate::interp::mem::Mem`]
//! at the oracle's exact layout ([`crate::interp::layout::Layout`]): scalars at
//! their natural width; aggregates (structs / fixed arrays / enums / slices /
//! boxes) in real slots; a borrow / `rawptr` / fn-pointer value is a plain word.
//! Absolute addresses are internal to each engine and are *not* part of the `θ`
//! trace the Stage-A gate compares — only observable output, the return value,
//! and the delivered fault identity are. The **init-byte guard** is oracle-only
//! (non-semantic, excluded from the gate), so this engine reads with it off.
//!
//! A3 closes Gate A over the full runnable corpus by executing the remainder:
//! `Box`/`Alloc` (box/unbox/clone/drop-free through the structurally-identified
//! vtable), the `rawptr` intrinsics, slices, interface method dispatch (as
//! ordinary mono-resolved calls), fn-pointer / indirect calls, cross-type `?`
//! (`From`), statics, and move-pruned drops (the static move masks the lowering
//! baked into each `Drop`, applied here with NO runtime flag).

#![allow(clippy::too_many_arguments)]

use std::collections::HashMap;

use crate::interp::layout::Layout;
use crate::interp::mem::{round_up, Mem};
use crate::interp::{Fault, FaultKind, Run};
use crate::resolve::Items;
use crate::span::Span;
use crate::token::ScalarTy;
use crate::types::{ItemEnv, Type};

use super::{
    BinOp, CollOp, FaultEdge, MirFn, MirProgram, Operand, Place, Proj, Regime, Rvalue,
    StatementKind, Terminator, UnOp,
};

/// Run a lowered program's `main` precisely over the shared memory substrate,
/// returning its `(ret, θ)` outcome or the delivered fault `f★`.
pub fn run(prog: &MirProgram, items: &Items, consts: &HashMap<String, u64>) -> Result<Run, Fault> {
    // Identify the compiler-known allocator structs structurally (never by a
    // hardcoded name), so box/unbox resolve `ctx`/`vt`/`alloc`/`free` offsets
    // regardless of how the module tree qualifies `Alloc`/`AllocVtable` (the same
    // identification the oracle uses in `interp::eval`).
    let alloc_vtable_struct = items
        .structs
        .iter()
        .find(|(_, s)| {
            s.fields.iter().any(|(n, t)| n == "alloc" && matches!(t, Type::FnPtr(_)))
                && s.fields.iter().any(|(n, t)| n == "free" && matches!(t, Type::FnPtr(_)))
        })
        .map(|(name, _)| name.clone());
    let alloc_struct = alloc_vtable_struct.as_ref().and_then(|vt| {
        items
            .structs
            .iter()
            .find(|(_, s)| {
                s.fields.iter().any(|(n, t)| {
                    n == "vt"
                        && matches!(t, Type::RawPtr(inner) if matches!(&**inner, Type::Named(x) if x == vt))
                })
            })
            .map(|(name, _)| name.clone())
    });

    let mut engine = Engine {
        prog,
        items,
        consts,
        mem: Mem::new(),
        trace: Vec::new(),
        depth: 0,
        statics: HashMap::new(),
        strings: HashMap::new(),
        alloc_struct,
        alloc_vtable_struct,
    };

    // Statics (design 0008): reserve every address first (forward references),
    // then run each init fn and copy its result into the reserved storage.
    for st in &prog.statics {
        let size = engine.size_of(&st.ty).max(1);
        let align = engine.align_of(&st.ty).max(1);
        let addr = engine.mem.static_alloc(size, align);
        let _ = engine.mem.write(addr, &vec![0u8; size as usize]);
        engine.statics.insert(st.name.clone(), addr);
    }
    let outcome: Result<i64, Fault> = (|| {
        for st in &prog.statics {
            let out = engine.call(&st.init_fn, &[])?;
            let addr = engine.statics[&st.name];
            let size = engine.size_of(&st.ty);
            engine.copy_bytes(addr, out, size)?;
        }
        let ret_addr = engine.call("main", &[])?;
        // `main` reports its 64-bit return word for `i64` or `f64` (design 0016).
        let ret_i64 = match prog.get("main").map(|f| &f.locals[0].ty) {
            Some(Type::Scalar(ScalarTy::I64)) | Some(Type::Scalar(ScalarTy::F64)) => {
                engine.read_int(ret_addr, ScalarTy::I64)? as i64
            }
            _ => 0,
        };
        Ok(ret_i64)
    })();
    match outcome {
        Ok(ret_i64) => Ok(Run { ret: ret_i64, trace: engine.trace }),
        // Thread the pre-fault trace into the escaping fault so the differential
        // harness compares the trace emitted BEFORE the fault, not just kind+span
        // (F-FAULT-TRACE).
        Err(mut f) => {
            f.trace = std::mem::take(&mut engine.trace);
            Err(f)
        }
    }
}

struct Engine<'a> {
    prog: &'a MirProgram,
    items: &'a Items,
    consts: &'a HashMap<String, u64>,
    mem: Mem,
    trace: Vec<i64>,
    depth: usize,
    /// Static name -> its reserved address in static storage.
    statics: HashMap<String, u64>,
    /// Interned string-literal bytes -> their static address (design 0001 §4.2).
    strings: HashMap<String, u64>,
    alloc_struct: Option<String>,
    alloc_vtable_struct: Option<String>,
}

const MAX_DEPTH: usize = 4096;

/// A concrete call frame: the base address of every local's slot.
struct Frame {
    addrs: Vec<u64>,
}

impl<'a> Engine<'a> {
    fn lay(&self) -> Layout<'_> {
        Layout { items: self.items, consts: self.consts }
    }
    fn size_of(&self, ty: &Type) -> u64 {
        self.lay().size_of(ty)
    }
    fn align_of(&self, ty: &Type) -> u64 {
        self.lay().align_of(ty)
    }
    fn field_off(&self, sname: &str, field: &str) -> u64 {
        self.lay().field_offset(sname, field).map(|(_, o)| o).unwrap_or(0)
    }
    fn alloc_struct_name(&self) -> &str {
        self.alloc_struct.as_deref().unwrap_or("Alloc")
    }
    fn alloc_vtable_name(&self) -> &str {
        self.alloc_vtable_struct.as_deref().unwrap_or("AllocVtable")
    }

    fn fault(&self, kind: FaultKind, span: Span, msg: &str) -> Fault {
        Fault::new(kind, span, msg)
    }

    // ---- memory ----
    fn read_int(&mut self, addr: u64, sty: ScalarTy) -> Result<i128, Fault> {
        let size = Layout::scalar_size(sty).max(1);
        let raw = self
            .mem
            .read_uint(addr, size, false)
            .map_err(|_| self.fault(FaultKind::BadPointer, Span::point(0), "read"))?;
        let (_, _, bits, signed) = ty_range(sty);
        let mut v = raw as i128;
        if signed && bits < 128 && (v & (1i128 << (bits - 1))) != 0 {
            v -= 1i128 << bits;
        }
        Ok(v)
    }
    fn write_int(&mut self, addr: u64, value: i128, sty: ScalarTy) -> Result<(), Fault> {
        let size = Layout::scalar_size(sty).max(1);
        let bytes = (value as u128).to_le_bytes();
        self.mem
            .write(addr, &bytes[..size as usize])
            .map_err(|_| self.fault(FaultKind::BadPointer, Span::point(0), "write"))
    }
    fn read_u64(&mut self, addr: u64) -> Result<u64, Fault> {
        self.mem
            .read_u64(addr, false)
            .map_err(|_| self.fault(FaultKind::BadPointer, Span::point(0), "read ptr"))
    }
    fn write_u64(&mut self, addr: u64, value: u64) -> Result<(), Fault> {
        self.mem
            .write(addr, &value.to_le_bytes())
            .map_err(|_| self.fault(FaultKind::BadPointer, Span::point(0), "write ptr"))
    }
    fn copy_bytes(&mut self, dst: u64, src: u64, len: u64) -> Result<(), Fault> {
        if len == 0 {
            return Ok(());
        }
        self.mem
            .copy(dst, src, len, false)
            .map_err(|_| self.fault(FaultKind::BadPointer, Span::point(0), "copy"))
    }

    /// Reserve and zero a stack slot for a local of `ty` (padding counts as init).
    fn alloc_slot(&mut self, ty: &Type) -> u64 {
        let size = self.size_of(ty).max(1);
        let align = self.align_of(ty).max(1);
        let addr = self.mem.stack_alloc(size, align);
        let _ = self.mem.write(addr, &vec![0u8; size as usize]);
        addr
    }

    /// Intern a string literal's bytes into static storage, once, returning its
    /// data address (the `[u8]` slice pointer).
    fn intern_str(&mut self, s: &str) -> Result<u64, Fault> {
        if let Some(a) = self.strings.get(s) {
            return Ok(*a);
        }
        let bytes = s.as_bytes().to_vec();
        let addr = self.mem.static_alloc(bytes.len().max(1) as u64, 1);
        self.mem
            .write(addr, &bytes)
            .map_err(|_| self.fault(FaultKind::BadPointer, Span::point(0), "str"))?;
        self.strings.insert(s.to_string(), addr);
        Ok(addr)
    }

    fn call(&mut self, name: &str, args: &[i128]) -> Result<u64, Fault> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(Fault::new(FaultKind::Panic, Span::point(0), "MIR recursion limit"));
        }
        let mf = self
            .prog
            .get(name)
            .ok_or_else(|| Fault::new(FaultKind::Panic, Span::point(0), format!("no MIR fn `{name}`")))?;
        let base_sp = self.mem.stack_bump;
        let mut frame = Frame { addrs: Vec::with_capacity(mf.locals.len()) };
        for l in &mf.locals {
            let a = self.alloc_slot(&l.ty);
            frame.addrs.push(a);
        }
        // Bind params _1..=n. A word-sized param (scalar / borrow / rawptr / fn-ptr)
        // receives its value directly; an aggregate-by-value param receives the
        // *address* of the caller's argument and byte-copies it into its slot.
        for (i, a) in args.iter().enumerate() {
            let pty = &mf.locals[1 + i].ty;
            let addr = frame.addrs[1 + i];
            if is_wordy(pty) {
                self.write_int(addr, *a, scalar_of(pty))?;
            } else {
                let size = self.size_of(pty);
                self.copy_bytes(addr, *a as u64, size)?;
            }
        }
        // requires (design 0001 §7.3).
        for pred in &mf.requires {
            self.run_cfg(mf, pred.entry, &frame)?;
            let v = self.read_int(frame.addrs[pred.value], ScalarTy::Bool)?;
            if v == 0 {
                self.depth -= 1;
                self.mem.stack_bump = base_sp;
                return Err(Fault::new(pred.kind, pred.span, "`requires` clause violated"));
            }
        }
        // body.
        self.run_cfg(mf, mf.entry, &frame)?;
        let ret_addr = frame.addrs[0];
        // ensures, reading `result`.
        if !mf.ensures.is_empty() {
            if let Some(rl) = mf.result_local {
                let sz = self.size_of(&mf.locals[0].ty);
                self.copy_bytes(frame.addrs[rl], ret_addr, sz)?;
            }
            for pred in &mf.ensures {
                self.run_cfg(mf, pred.entry, &frame)?;
                let v = self.read_int(frame.addrs[pred.value], ScalarTy::Bool)?;
                if v == 0 {
                    self.depth -= 1;
                    self.mem.stack_bump = base_sp;
                    return Err(Fault::new(pred.kind, pred.span, "`ensures` clause violated"));
                }
            }
        }
        self.depth -= 1;
        // Copy the return value out before popping the stack frame.
        let rty = mf.locals[0].ty.clone();
        let rsize = self.size_of(&rty).max(1);
        let out = self.mem.stack_alloc(rsize, self.align_of(&rty).max(1));
        self.copy_bytes(out, ret_addr, self.size_of(&rty))?;
        self.mem.stack_bump = base_sp.max(out + rsize);
        Ok(out)
    }

    /// Call the fn named by a fn-pointer id and read its word-sized return value.
    fn call_id_word(&mut self, id: u64, args: &[i128]) -> Result<u64, Fault> {
        let name = self
            .prog
            .fn_ptrs
            .get(id as usize)
            .cloned()
            .ok_or_else(|| Fault::new(FaultKind::Panic, Span::point(0), "bad fn-pointer id"))?;
        let out = self.call(&name, args)?;
        self.read_u64(out)
    }

    fn call_free(&mut self, ctx: u64, vt: u64, ptr: u64, size: u64, align: u64) -> Result<(), Fault> {
        let free_off = self.field_off(self.alloc_vtable_name(), "free");
        let ffn = self.read_u64(vt + free_off)?;
        self.call_id_word(ffn, &[ctx as i128, ptr as i128, size as i128, align as i128])?;
        Ok(())
    }

    /// Execute the CFG from `entry` until a `Return` (Ok) or a fault edge (Err).
    fn run_cfg(&mut self, mf: &MirFn, entry: usize, frame: &Frame) -> Result<(), Fault> {
        let mut bb = entry;
        loop {
            let block = &mf.blocks[bb];
            for st in &block.stmts {
                match &st.kind {
                    StatementKind::Assign(local, rv) => {
                        let addr = frame.addrs[*local];
                        let sty = scalar_of(&mf.locals[*local].ty);
                        let val = self.eval_rvalue(rv, mf, frame)?;
                        self.write_int(addr, val, sty)?;
                    }
                    StatementKind::Store(place, rv) => {
                        let val = self.eval_rvalue(rv, mf, frame)?;
                        let (addr, ty) = self.place_addr(place, mf, frame)?;
                        let sty = scalar_of(&ty);
                        self.write_int(addr, val, sty)?;
                    }
                    StatementKind::CopyVal { dst, src, ty } => {
                        let (saddr, _) = self.place_addr(src, mf, frame)?;
                        let (daddr, _) = self.place_addr(dst, mf, frame)?;
                        let sz = self.size_of(ty);
                        self.copy_bytes(daddr, saddr, sz)?;
                    }
                    StatementKind::Trace(op) => {
                        let v = self.eval_operand(op, mf, frame)?;
                        self.trace.push(v as i64);
                    }
                    StatementKind::Drop { local, moved } => {
                        let addr = frame.addrs[*local];
                        let ty = mf.locals[*local].ty.clone();
                        self.drop_value(addr, &ty, moved, &mut Vec::new())?;
                    }
                    StatementKind::BoxOp { dst, inner_ty, result_ty, alloc, value } => {
                        self.box_op(dst, inner_ty, result_ty, alloc, value, mf, frame)?;
                    }
                    StatementKind::UnboxOp { dst, inner_ty, boxed } => {
                        self.unbox_op(dst, inner_ty, boxed, mf, frame)?;
                    }
                    StatementKind::Subslice { dst, src, lo, hi, stride, span } => {
                        self.subslice_op(dst, src, *lo, *hi, *stride, *span, mf, frame)?;
                    }
                    StatementKind::StrFrom { dst, src } => {
                        self.str_from_op(dst, src, mf, frame)?;
                    }
                    StatementKind::Substr { dst, src, lo, hi, span } => {
                        self.substr_op(dst, src, *lo, *hi, *span, mf, frame)?;
                    }
                    StatementKind::CollectionOp { dst, op } => {
                        self.collection_op(dst, op, mf, frame)?;
                    }
                    // Stage-2 sequential oracle (design 0012 §6): a `spawn` runs the
                    // task inline at the spawn point (spawn order); a fault propagates
                    // immediately, which is naturally spawn-order-first (§3.2). The
                    // `scope` markers are join-barrier no-ops for the single-threaded
                    // oracle (every task has already run by the closing brace).
                    StatementKind::Spawn { func, args } => {
                        let mut vals = Vec::with_capacity(args.len());
                        for a in args {
                            vals.push(self.eval_operand(a, mf, frame)?);
                        }
                        self.call(func, &vals)?;
                    }
                    StatementKind::ScopeBegin | StatementKind::ScopeEnd => {}
                }
            }
            match &block.term {
                Terminator::Goto(next) => bb = *next,
                Terminator::Branch { cond, then_bb, else_bb } => {
                    let c = self.eval_operand(cond, mf, frame)?;
                    bb = if c != 0 { *then_bb } else { *else_bb };
                }
                Terminator::Return => return Ok(()),
                Terminator::Fault(edge) => return Err(fault_of(edge)),
            }
        }
    }

    /// Resolve a place to its `(address, type)` in memory, faulting on OOB index.
    fn place_addr(&mut self, place: &Place, mf: &MirFn, frame: &Frame) -> Result<(u64, Type), Fault> {
        let mut addr = frame.addrs[place.root];
        let mut ty = mf.locals[place.root].ty.clone();
        for p in &place.proj {
            match p {
                Proj::Field { offset, ty: fty } => {
                    addr += *offset;
                    ty = fty.clone();
                }
                Proj::Deref { inner } => {
                    addr = self.read_u64(addr)?;
                    ty = inner.clone();
                }
                Proj::Index { index, stride, len, span, slice } => {
                    let i = self.eval_operand(index, mf, frame)? as u64;
                    if *slice {
                        // Slice header: {ptr @0, len @8}; the runtime length gates.
                        let base = self.read_u64(addr)?;
                        let n = self.read_u64(addr + 8)?;
                        if i >= n {
                            return Err(self.fault(FaultKind::Bounds, *span, "index out of bounds"));
                        }
                        addr = base + i * *stride;
                    } else {
                        if i >= *len {
                            return Err(self.fault(FaultKind::Bounds, *span, "index out of bounds"));
                        }
                        addr += i * *stride;
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
        Ok((addr, ty))
    }

    fn eval_operand(&mut self, op: &Operand, mf: &MirFn, frame: &Frame) -> Result<i128, Fault> {
        match op {
            Operand::Const(v, _ty) => Ok(*v),
            Operand::Local(id) => {
                let sty = scalar_of(&mf.locals[*id].ty);
                self.read_int(frame.addrs[*id], sty)
            }
        }
    }

    fn eval_rvalue(&mut self, rv: &Rvalue, mf: &MirFn, frame: &Frame) -> Result<i128, Fault> {
        match rv {
            Rvalue::Use(op) => self.eval_operand(op, mf, frame),
            Rvalue::Ref(place) => {
                let (addr, _) = self.place_addr(place, mf, frame)?;
                Ok(addr as i128)
            }
            Rvalue::Load { place, ty } => {
                let (addr, _) = self.place_addr(place, mf, frame)?;
                let sty = scalar_of(ty);
                self.read_int(addr, sty)
            }
            Rvalue::StaticAddr(name) => {
                let a = *self
                    .statics
                    .get(name)
                    .ok_or_else(|| Fault::new(FaultKind::Panic, Span::point(0), "unknown static"))?;
                Ok(a as i128)
            }
            Rvalue::StrAddr(s) => {
                let a = self.intern_str(s)?;
                Ok(a as i128)
            }
            Rvalue::IsNull(op) => {
                let v = self.eval_operand(op, mf, frame)?;
                Ok((v == 0) as i128)
            }
            Rvalue::PtrArith { base, index, stride } => {
                let b = self.eval_operand(base, mf, frame)?;
                let i = self.eval_operand(index, mf, frame)?;
                let r = (b + i * (*stride as i128)) as u64;
                Ok(r as i128)
            }
            Rvalue::Cmp { op, l, r } => {
                let a = self.eval_operand(l, mf, frame)?;
                let b = self.eval_operand(r, mf, frame)?;
                let (lsty, rsty) = (operand_sty(l, mf), operand_sty(r, mf));
                let res = if lsty.is_float() || rsty.is_float() {
                    // IEEE comparison: any NaN operand yields false (except `!=`).
                    // Both operands share the same float type (checker-guaranteed).
                    let sty = if lsty.is_float() { lsty } else { rsty };
                    float_cmp(*op, sty, a as u64, b as u64)
                } else {
                    match op {
                        BinOp::Eq => a == b,
                        BinOp::Ne => a != b,
                        BinOp::Lt => a < b,
                        BinOp::Le => a <= b,
                        BinOp::Gt => a > b,
                        BinOp::Ge => a >= b,
                        _ => unreachable!("non-comparison in Cmp"),
                    }
                };
                Ok(res as i128)
            }
            Rvalue::Bin { op, regime, ty, l, r, span, fault } => {
                use BinOp::*;
                let lv = self.eval_operand(l, mf, frame)?;
                let rv2 = self.eval_operand(r, mf, frame)?;
                if ty.is_float() {
                    // IEEE-754: never faults; regime-exempt (design 0016 §2).
                    return Ok(float_arith(*op, *ty, lv as u64, rv2 as u64) as i128);
                }
                let (regime, ty, span, fault) = (*regime, *ty, *span, fault.as_ref());
                let out = match op {
                    Add => fit(lv + rv2, ty, regime, fault)?,
                    Sub => fit(lv - rv2, ty, regime, fault)?,
                    Mul => fit(lv * rv2, ty, regime, fault)?,
                    Div | Rem => {
                        if rv2 == 0 {
                            return Err(Fault::new(FaultKind::DivByZero, span, "division by zero"));
                        }
                        let raw = if *op == Div { lv / rv2 } else { lv % rv2 };
                        fit(raw, ty, regime, fault)?
                    }
                    BitAnd => fit_bits(lv & rv2, ty),
                    BitOr => fit_bits(lv | rv2, ty),
                    BitXor => fit_bits(lv ^ rv2, ty),
                    Shl | Shr => {
                        let bits = ty_range(ty).2 as i128;
                        let amt = if rv2 < 0 {
                            return Err(Fault::new(FaultKind::Overflow, span, "negative shift amount"));
                        } else if rv2 >= bits {
                            match regime {
                                Regime::Checked => {
                                    return Err(Fault::new(FaultKind::Overflow, span, "shift amount exceeds bit width"));
                                }
                                Regime::Wrapping => rv2 % bits,
                                Regime::Saturating => bits - 1,
                            }
                        } else {
                            rv2
                        };
                        let raw = if *op == Shl { lv << amt } else { lv >> amt };
                        fit_bits(raw, ty)
                    }
                    _ => unreachable!("comparison/logical in Bin"),
                };
                Ok(out)
            }
            Rvalue::Un { op, regime, ty, v, fault } => {
                let x = self.eval_operand(v, mf, frame)?;
                match op {
                    UnOp::Not => Ok((x == 0) as i128),
                    UnOp::Neg if ty.is_float() => Ok(float_neg(*ty, x as u64) as i128),
                    UnOp::Neg => fit(-x, *ty, *regime, fault.as_ref()),
                    UnOp::BitNot => Ok(fit_bits(!x, *ty)),
                }
            }
            Rvalue::Conv { to, regime, v, fault } => {
                let from = operand_sty(v, mf);
                let x = self.eval_operand(v, mf, frame)?;
                if to.is_float() || from.is_float() {
                    // int<->float / f32<->f64 (design 0016 §5): IEEE, regime-exempt,
                    // never faults.
                    return Ok(float_conv(x, from, *to));
                }
                convert(x, *to, *regime, fault.as_ref())
            }
            Rvalue::Bitcast { to, v } => {
                // Pure bit reinterpretation (design 0016 section 10): re-fit the
                // operand's identical bits into the target width/signedness. Same
                // width (checker), so no value changes; never faults.
                let x = self.eval_operand(v, mf, frame)?;
                Ok(fit_bits(x, *to))
            }
            Rvalue::Sqrt { ty, v } => {
                // Correctly-rounded IEEE square root (design 0016 §11); never faults.
                let x = self.eval_operand(v, mf, frame)?;
                Ok(float_sqrt(*ty, x as u64) as i128)
            }
            Rvalue::Call { func, args } => {
                let mut vals = Vec::with_capacity(args.len());
                for a in args {
                    vals.push(self.eval_operand(a, mf, frame)?);
                }
                // A foreign (`extern`) call: no MIR fn exists; dispatch through the
                // shim registry, faulting `no_foreign_runtime` if unregistered
                // (design 0011 §5). Both engines share the one registry, so the
                // trace matches the tree-walker exactly.
                if let Some(es) = self.items.externs.get(func.as_str()) {
                    let sp = es.span;
                    return match crate::foreign::dispatch(func, &vals, &mut self.mem) {
                        Some(v) => Ok(v),
                        None => Err(Fault::new(
                            FaultKind::NoForeignRuntime,
                            sp,
                            format!("no foreign runtime for `{func}` (no shim registered; native backend is a 0010 forward dependency)"),
                        )),
                    };
                }
                let ret_addr = self.call(func, &vals)?;
                let rty = self.prog.get(func).map(|f| f.locals[0].ty.clone()).unwrap_or(Type::unit());
                if is_wordy(&rty) {
                    self.read_int(ret_addr, scalar_of(&rty))
                } else {
                    Ok(ret_addr as i128)
                }
            }
            Rvalue::CallIndirect { func, args } => {
                let id = self.eval_operand(func, mf, frame)? as u64;
                let name = self
                    .prog
                    .fn_ptrs
                    .get(id as usize)
                    .cloned()
                    .ok_or_else(|| Fault::new(FaultKind::Panic, Span::point(0), "bad fn-pointer id"))?;
                let mut vals = Vec::with_capacity(args.len());
                for a in args {
                    vals.push(self.eval_operand(a, mf, frame)?);
                }
                let ret_addr = self.call(&name, &vals)?;
                let rty = self.prog.get(&name).map(|f| f.locals[0].ty.clone()).unwrap_or(Type::unit());
                if is_wordy(&rty) {
                    self.read_int(ret_addr, scalar_of(&rty))
                } else {
                    Ok(ret_addr as i128)
                }
            }
        }
    }

    // ---- box / unbox / subslice ----
    fn box_op(
        &mut self,
        dst: &Place,
        inner_ty: &Type,
        result_ty: &Type,
        alloc: &Operand,
        value: &Place,
        mf: &MirFn,
        frame: &Frame,
    ) -> Result<(), Fault> {
        let alloc_addr = self.eval_operand(alloc, mf, frame)? as u64;
        let ctx_off = self.field_off(self.alloc_struct_name(), "ctx");
        let vt_off = self.field_off(self.alloc_struct_name(), "vt");
        let ctx = self.read_u64(alloc_addr + ctx_off)?;
        let vt = self.read_u64(alloc_addr + vt_off)?;
        let size = self.size_of(inner_ty);
        let align = self.align_of(inner_ty);
        let alloc_off = self.field_off(self.alloc_vtable_name(), "alloc");
        let afn = self.read_u64(vt + alloc_off)?;
        let ret = self.call_id_word(afn, &[ctx as i128, size as i128, align as i128])?;
        let (value_addr, _) = self.place_addr(value, mf, frame)?;
        let (daddr, _) = self.place_addr(dst, mf, frame)?;
        // BoxResult (design 0001 §6.2): [tag @0][Box{ptr @8, ctx @16, vt @24}].
        let einfo = self.lay().enum_info(result_ty);
        let (boxed_idx, oom_idx) = match &einfo {
            Some(v) => {
                let bx = v.iter().position(|(_, p)| !p.is_empty()).unwrap_or(0) as u64;
                let om = v.iter().position(|(_, p)| p.is_empty()).unwrap_or(1) as u64;
                (bx, om)
            }
            None => (0, 1),
        };
        if ret == 0 {
            self.drop_value(value_addr, inner_ty, &[], &mut Vec::new())?;
            self.write_u64(daddr, oom_idx)?;
        } else {
            self.copy_bytes(ret, value_addr, size)?;
            self.write_u64(daddr, boxed_idx)?;
            self.write_u64(daddr + 8, ret)?;
            self.write_u64(daddr + 16, ctx)?;
            self.write_u64(daddr + 24, vt)?;
        }
        Ok(())
    }

    fn unbox_op(&mut self, dst: &Place, inner_ty: &Type, boxed: &Place, mf: &MirFn, frame: &Frame) -> Result<(), Fault> {
        let (baddr, _) = self.place_addr(boxed, mf, frame)?;
        let ptr = self.read_u64(baddr)?;
        let ctx = self.read_u64(baddr + 8)?;
        let vt = self.read_u64(baddr + 16)?;
        let size = self.size_of(inner_ty);
        let align = self.align_of(inner_ty);
        let (daddr, _) = self.place_addr(dst, mf, frame)?;
        self.copy_bytes(daddr, ptr, size)?;
        self.call_free(ctx, vt, ptr, size, align)?;
        Ok(())
    }

    fn subslice_op(
        &mut self,
        dst: &Place,
        src: &Place,
        lo: Operand,
        hi: Operand,
        stride: u64,
        span: Span,
        mf: &MirFn,
        frame: &Frame,
    ) -> Result<(), Fault> {
        let (saddr, _) = self.place_addr(src, mf, frame)?;
        let ptr = self.read_u64(saddr)?;
        let len = self.read_u64(saddr + 8)?;
        let lo = self.eval_operand(&lo, mf, frame)? as u64;
        let hi = self.eval_operand(&hi, mf, frame)? as u64;
        if lo > hi || hi > len {
            return Err(self.fault(FaultKind::Bounds, span, "subslice out of bounds"));
        }
        let (daddr, _) = self.place_addr(dst, mf, frame)?;
        self.write_u64(daddr, ptr + lo * stride)?;
        self.write_u64(daddr + 8, hi - lo)?;
        Ok(())
    }

    /// `str_from(b) -> Utf8Res` (design 0013 §4): UTF-8-validate the `[u8]` view at
    /// `src`, building `Utf8Res::Valid(str)` (the same fat pointer) or
    /// `Utf8Res::Invalid(offset)`. Mirrors the tree-walker `bi_str_from` — the
    /// offset is `str::from_utf8().valid_up_to()`, the start of the first ill-formed
    /// sequence.
    fn str_from_op(&mut self, dst: &Place, src: &Place, mf: &MirFn, frame: &Frame) -> Result<(), Fault> {
        let (saddr, _) = self.place_addr(src, mf, frame)?;
        let ptr = self.read_u64(saddr)?;
        let len = self.read_u64(saddr + 8)?;
        let bytes = self.read_bytes(ptr, len)?;
        let ures = Type::Named("Utf8Res".to_string());
        let einfo = self
            .lay()
            .enum_info(&ures)
            .ok_or_else(|| self.fault(FaultKind::Panic, Span::point(0), "unknown enum `Utf8Res`"))?;
        let valid_idx = einfo.iter().position(|(n, _)| n == "Valid").unwrap_or(0);
        let invalid_idx = einfo.iter().position(|(n, _)| n == "Invalid").unwrap_or(1);
        let (daddr, _) = self.place_addr(dst, mf, frame)?;
        match std::str::from_utf8(&bytes) {
            Ok(_) => {
                self.write_u64(daddr, valid_idx as u64)?;
                let (_, off) = self.lay().payload_offset(&einfo[valid_idx].1, 0);
                // `Valid`'s `str` payload IS the validated `{ptr, len}` fat pointer.
                self.write_u64(daddr + off, ptr)?;
                self.write_u64(daddr + off + 8, len)?;
            }
            Err(e) => {
                self.write_u64(daddr, invalid_idx as u64)?;
                let (_, off) = self.lay().payload_offset(&einfo[invalid_idx].1, 0);
                self.write_u64(daddr + off, e.valid_up_to() as u64)?;
            }
        }
        Ok(())
    }

    /// `substr(s, lo, hi) -> str` (design 0013 §3): the `[lo, hi)` byte sub-view,
    /// faulting `Bounds` at `span` on an out-of-range OR non-char-boundary offset.
    /// Mirrors the tree-walker `bi_substr` (`str_is_boundary`).
    fn substr_op(
        &mut self,
        dst: &Place,
        src: &Place,
        lo: Operand,
        hi: Operand,
        span: Span,
        mf: &MirFn,
        frame: &Frame,
    ) -> Result<(), Fault> {
        let (saddr, _) = self.place_addr(src, mf, frame)?;
        let ptr = self.read_u64(saddr)?;
        let len = self.read_u64(saddr + 8)?;
        let lo = self.eval_operand(&lo, mf, frame)? as u64;
        let hi = self.eval_operand(&hi, mf, frame)? as u64;
        if lo > hi || hi > len {
            return Err(self.fault(FaultKind::Bounds, span, "substr out of bounds"));
        }
        let bytes = self.read_bytes(ptr, len)?;
        if !str_is_boundary(&bytes, lo as usize) || !str_is_boundary(&bytes, hi as usize) {
            return Err(self.fault(FaultKind::Bounds, span, "substr not on a char boundary"));
        }
        let (daddr, _) = self.place_addr(dst, mf, frame)?;
        self.write_u64(daddr, ptr + lo)?;
        self.write_u64(daddr + 8, hi - lo)?;
        Ok(())
    }

    // ---- compiler-known collections (Vec / Map / String) ----
    //
    // These handlers port `interp::eval`'s `bi_vec_*`/`bi_map_*`/`bi_string_*`
    // bodies onto the SAME flat memory substrate (design 0010 §5): identical
    // header layout `{ buf@0, len@8, cap@16, ctx@24, vt@32 }`, identical alloc-copy
    // -free growth, identical FNV-1a hash + open-addressed linear probing, so the
    // MIR execution matches the tree-walker byte-for-byte (the round-trip gate).

    fn read_bytes(&mut self, addr: u64, len: u64) -> Result<Vec<u8>, Fault> {
        if len == 0 {
            return Ok(Vec::new());
        }
        self.mem
            .read(addr, len, false)
            .map_err(|_| self.fault(FaultKind::BadPointer, Span::point(0), "read bytes"))
    }
    fn write_bytes(&mut self, addr: u64, data: &[u8]) -> Result<(), Fault> {
        self.mem
            .write(addr, data)
            .map_err(|_| self.fault(FaultKind::BadPointer, Span::point(0), "write bytes"))
    }

    /// Allocate `size` bytes through the carried allocator's `alloc` vtable slot.
    fn call_alloc(&mut self, ctx: u64, vt: u64, size: u64, align: u64) -> Result<u64, Fault> {
        let alloc_off = self.field_off(self.alloc_vtable_name(), "alloc");
        let afn = self.read_u64(vt + alloc_off)?;
        self.call_id_word(afn, &[ctx as i128, size as i128, align as i128])
    }

    fn call_realloc(&mut self, ctx: u64, vt: u64, ptr: u64, old_size: u64, new_size: u64, align: u64) -> Result<u64, Fault> {
        let realloc_off = self.field_off(self.alloc_vtable_name(), "realloc");
        let rfn = self.read_u64(vt + realloc_off)?;
        self.call_id_word(rfn, &[ctx as i128, ptr as i128, old_size as i128, new_size as i128, align as i128])
    }

    fn collection_op(&mut self, dst: &Place, op: &CollOp, mf: &MirFn, frame: &Frame) -> Result<(), Fault> {
        match op {
            CollOp::New { alloc } => {
                let alloc_addr = self.eval_operand(alloc, mf, frame)? as u64;
                let ctx_off = self.field_off(self.alloc_struct_name(), "ctx");
                let vt_off = self.field_off(self.alloc_struct_name(), "vt");
                let ctx = self.read_u64(alloc_addr + ctx_off)?;
                let vt = self.read_u64(alloc_addr + vt_off)?;
                let (daddr, _) = self.place_addr(dst, mf, frame)?;
                self.write_u64(daddr, 0)?; // buf
                self.write_u64(daddr + 8, 0)?; // len
                self.write_u64(daddr + 16, 0)?; // cap
                self.write_u64(daddr + 24, ctx)?; // ctx
                self.write_u64(daddr + 32, vt)?; // vt
                Ok(())
            }
            CollOp::VecPush { base, elem, value, span } => {
                let base = self.eval_operand(base, mf, frame)? as u64;
                self.vec_reserve(base, elem, 1, *span)?;
                let stride = round_up(self.size_of(elem), self.align_of(elem));
                let buf = self.read_u64(base)?;
                let len = self.read_u64(base + 8)?;
                let (vaddr, _) = self.place_addr(value, mf, frame)?;
                let sz = self.size_of(elem);
                self.copy_bytes(buf + len * stride, vaddr, sz)?;
                self.write_u64(base + 8, len + 1)?;
                Ok(())
            }
            CollOp::VecPop { base, elem } => {
                let base = self.eval_operand(base, mf, frame)? as u64;
                let (daddr, _) = self.place_addr(dst, mf, frame)?;
                let len = self.read_u64(base + 8)?;
                let opt = Type::Named("Opt".to_string());
                let einfo = self
                    .lay()
                    .enum_info(&opt)
                    .ok_or_else(|| self.fault(FaultKind::Panic, Span::point(0), "unknown enum `Opt`"))?;
                let some_idx = einfo.iter().position(|(n, _)| n == "Some").unwrap_or(0);
                let none_idx = einfo.iter().position(|(n, _)| n == "None").unwrap_or(1);
                if len == 0 {
                    self.write_u64(daddr, none_idx as u64)?;
                    return Ok(());
                }
                let newlen = len - 1;
                let stride = round_up(self.size_of(elem), self.align_of(elem));
                let buf = self.read_u64(base)?;
                let src = buf + newlen * stride;
                self.write_u64(base + 8, newlen)?;
                self.write_u64(daddr, some_idx as u64)?;
                let some_payloads = einfo[some_idx].1.clone();
                let (_, off) = self.lay().payload_offset(&some_payloads, 0);
                let sz = self.size_of(elem);
                self.copy_bytes(daddr + off, src, sz)?;
                Ok(())
            }
            CollOp::VecGet { base, elem, index, span } => {
                let base = self.eval_operand(base, mf, frame)? as u64;
                let len = self.read_u64(base + 8)?;
                let i = self.eval_operand(index, mf, frame)? as u64;
                if i >= len {
                    return Err(self.fault(FaultKind::Bounds, *span, "Vec index out of bounds"));
                }
                let stride = round_up(self.size_of(elem), self.align_of(elem));
                let buf = self.read_u64(base)?;
                let (daddr, _) = self.place_addr(dst, mf, frame)?;
                self.write_u64(daddr, buf + i * stride)?;
                Ok(())
            }
            CollOp::VecSet { base, elem, index, value, span } => {
                let base = self.eval_operand(base, mf, frame)? as u64;
                let len = self.read_u64(base + 8)?;
                let i = self.eval_operand(index, mf, frame)? as u64;
                if i >= len {
                    return Err(self.fault(FaultKind::Bounds, *span, "Vec index out of bounds"));
                }
                let stride = round_up(self.size_of(elem), self.align_of(elem));
                let buf = self.read_u64(base)?;
                let slot = buf + i * stride;
                let (vaddr, _) = self.place_addr(value, mf, frame)?;
                self.drop_value(slot, elem, &[], &mut Vec::new())?;
                let sz = self.size_of(elem);
                self.copy_bytes(slot, vaddr, sz)?;
                Ok(())
            }
            CollOp::MapInsert { base, valty, key, value, span } => {
                let base = self.eval_operand(base, mf, frame)? as u64;
                let (kaddr, _) = self.place_addr(key, mf, frame)?;
                let kptr = self.read_u64(kaddr)?;
                let klen = self.read_u64(kaddr + 8)?;
                let key = self.read_bytes(kptr, klen)?;
                let stride = round_up(24 + self.size_of(valty), 8);
                let (vaddr, _) = self.place_addr(value, mf, frame)?;
                let vsz = self.size_of(valty);
                let buf0 = self.read_u64(base)?;
                let cap0 = self.read_u64(base + 16)?;
                if let Some(slot) = self.map_find(buf0, cap0, stride, &key)? {
                    let voff = buf0 + slot * stride + 24;
                    self.drop_value(voff, valty, &[], &mut Vec::new())?;
                    self.copy_bytes(voff, vaddr, vsz)?;
                    return Ok(());
                }
                self.map_reserve(base, valty, *span)?;
                let buf = self.read_u64(base)?;
                let cap = self.read_u64(base + 16)?;
                let slot = self.map_find_empty(buf, cap, stride, &key)?;
                let ctx = self.read_u64(base + 24)?;
                let vt = self.read_u64(base + 32)?;
                let kbuf = self.call_alloc(ctx, vt, klen, 1)?;
                if kbuf == 0 {
                    return Err(self.fault(FaultKind::Panic, *span, "Map key allocation failed (OOM)"));
                }
                self.write_bytes(kbuf, &key)?;
                let b = buf + slot * stride;
                self.write_u64(b, 1)?; // state = occupied
                self.write_u64(b + 8, kbuf)?; // keyptr
                self.write_u64(b + 16, klen)?; // keylen
                self.copy_bytes(b + 24, vaddr, vsz)?; // value
                let len = self.read_u64(base + 8)?;
                self.write_u64(base + 8, len + 1)?;
                Ok(())
            }
            CollOp::MapContains { base, valty, key } => {
                let base = self.eval_operand(base, mf, frame)? as u64;
                let (kaddr, _) = self.place_addr(key, mf, frame)?;
                let kptr = self.read_u64(kaddr)?;
                let klen = self.read_u64(kaddr + 8)?;
                let key = self.read_bytes(kptr, klen)?;
                let stride = round_up(24 + self.size_of(valty), 8);
                let buf = self.read_u64(base)?;
                let cap = self.read_u64(base + 16)?;
                let found = self.map_find(buf, cap, stride, &key)?.is_some();
                let (daddr, _) = self.place_addr(dst, mf, frame)?;
                self.write_int(daddr, found as i128, ScalarTy::Bool)?;
                Ok(())
            }
            CollOp::MapGet { base, valty, key, span } => {
                let base = self.eval_operand(base, mf, frame)? as u64;
                let (kaddr, _) = self.place_addr(key, mf, frame)?;
                let kptr = self.read_u64(kaddr)?;
                let klen = self.read_u64(kaddr + 8)?;
                let key = self.read_bytes(kptr, klen)?;
                let stride = round_up(24 + self.size_of(valty), 8);
                let buf = self.read_u64(base)?;
                let cap = self.read_u64(base + 16)?;
                match self.map_find(buf, cap, stride, &key)? {
                    Some(slot) => {
                        let voff = buf + slot * stride + 24;
                        let (daddr, _) = self.place_addr(dst, mf, frame)?;
                        self.write_u64(daddr, voff)?;
                        Ok(())
                    }
                    None => Err(self.fault(FaultKind::Bounds, *span, "Map key not found")),
                }
            }
            CollOp::StringPush { base, ch, span } => {
                let base = self.eval_operand(base, mf, frame)? as u64;
                let c = self.eval_operand(ch, mf, frame)? as u32;
                let enc = match utf8_encode_scalar(c) {
                    Some(e) => e,
                    None => return Err(self.fault(FaultKind::Requires, *span, "push: not a Unicode scalar value")),
                };
                self.string_reserve(base, enc.len() as u64, *span)?;
                let buf = self.read_u64(base)?;
                let len = self.read_u64(base + 8)?;
                self.write_bytes(buf + len, &enc)?;
                self.write_u64(base + 8, len + enc.len() as u64)?;
                Ok(())
            }
            CollOp::StringAppend { base, view, span } => {
                let base = self.eval_operand(base, mf, frame)? as u64;
                let (vaddr, _) = self.place_addr(view, mf, frame)?;
                let ptr = self.read_u64(vaddr)?;
                let slen = self.read_u64(vaddr + 8)?;
                let bytes = self.read_bytes(ptr, slen)?;
                self.string_reserve(base, slen, *span)?;
                let buf = self.read_u64(base)?;
                let len = self.read_u64(base + 8)?;
                self.write_bytes(buf + len, &bytes)?;
                self.write_u64(base + 8, len + slen)?;
                Ok(())
            }
            CollOp::StringAsStr { base } => {
                let base = self.eval_operand(base, mf, frame)? as u64;
                let buf = self.read_u64(base)?;
                let len = self.read_u64(base + 8)?;
                let (daddr, _) = self.place_addr(dst, mf, frame)?;
                self.write_u64(daddr, buf)?;
                self.write_u64(daddr + 8, len)?;
                Ok(())
            }
        }
    }

    /// Ensure the Vec has room for `need` more elements (alloc-copy-free growth).
    fn vec_reserve(&mut self, base: u64, elem: &Type, need: u64, span: Span) -> Result<(), Fault> {
        let len = self.read_u64(base + 8)?;
        let cap = self.read_u64(base + 16)?;
        if len + need <= cap {
            return Ok(());
        }
        let stride = round_up(self.size_of(elem), self.align_of(elem));
        let align = self.align_of(elem);
        let newcap = (len + need).max(cap * 2).max(4);
        let ctx = self.read_u64(base + 24)?;
        let vt = self.read_u64(base + 32)?;
        let oldbuf = self.read_u64(base)?;
        let newbuf = if oldbuf == 0 {
            self.call_alloc(ctx, vt, newcap * stride, align)?
        } else {
            self.call_realloc(ctx, vt, oldbuf, len * stride, newcap * stride, align)?
        };
        if newbuf == 0 {
            return Err(self.fault(FaultKind::Panic, span, "Vec allocation failed (OOM)"));
        }
        self.write_u64(base, newbuf)?;
        self.write_u64(base + 16, newcap)?;
        Ok(())
    }

    /// Ensure the String has room for `need` more bytes (grow via `realloc`).
    fn string_reserve(&mut self, base: u64, need: u64, span: Span) -> Result<(), Fault> {
        let len = self.read_u64(base + 8)?;
        let cap = self.read_u64(base + 16)?;
        if len + need <= cap {
            return Ok(());
        }
        let newcap = (len + need).max(cap * 2).max(8);
        let ctx = self.read_u64(base + 24)?;
        let vt = self.read_u64(base + 32)?;
        let oldbuf = self.read_u64(base)?;
        let newbuf = if oldbuf == 0 {
            self.call_alloc(ctx, vt, newcap, 1)?
        } else {
            self.call_realloc(ctx, vt, oldbuf, len, newcap, 1)?
        };
        if newbuf == 0 {
            return Err(self.fault(FaultKind::Panic, span, "String allocation failed (OOM)"));
        }
        self.write_u64(base, newbuf)?;
        self.write_u64(base + 16, newcap)?;
        Ok(())
    }

    /// The occupied bucket slot matching `key`, else `None` (probing stops at the
    /// first empty bucket — no tombstones, as no `remove` op ships).
    fn map_find(&mut self, buf: u64, cap: u64, stride: u64, key: &[u8]) -> Result<Option<u64>, Fault> {
        if cap == 0 || buf == 0 {
            return Ok(None);
        }
        let mask = cap - 1;
        let mut idx = map_hash(key) & mask;
        loop {
            let b = buf + idx * stride;
            if self.read_u64(b)? == 0 {
                return Ok(None);
            }
            let klen = self.read_u64(b + 16)?;
            if klen == key.len() as u64 {
                let kptr = self.read_u64(b + 8)?;
                if self.read_bytes(kptr, klen)? == key {
                    return Ok(Some(idx));
                }
            }
            idx = (idx + 1) & mask;
        }
    }

    /// The first empty slot along `key`'s probe chain (caller ensures `key` absent).
    fn map_find_empty(&mut self, buf: u64, cap: u64, stride: u64, key: &[u8]) -> Result<u64, Fault> {
        let mask = cap - 1;
        let mut idx = map_hash(key) & mask;
        loop {
            let b = buf + idx * stride;
            if self.read_u64(b)? == 0 {
                return Ok(idx);
            }
            idx = (idx + 1) & mask;
        }
    }

    /// Grow + rehash (initial 8, then x2) at load factor 3/4 (alloc-rehash-free).
    fn map_reserve(&mut self, base: u64, valty: &Type, span: Span) -> Result<(), Fault> {
        let len = self.read_u64(base + 8)?;
        let cap = self.read_u64(base + 16)?;
        if cap != 0 && (len + 1) * 4 <= cap * 3 {
            return Ok(());
        }
        let stride = round_up(24 + self.size_of(valty), 8);
        let newcap = if cap == 0 { 8 } else { cap * 2 };
        let ctx = self.read_u64(base + 24)?;
        let vt = self.read_u64(base + 32)?;
        let newbuf = self.call_alloc(ctx, vt, newcap * stride, 8)?;
        if newbuf == 0 {
            return Err(self.fault(FaultKind::Panic, span, "Map allocation failed (OOM)"));
        }
        self.write_bytes(newbuf, &vec![0u8; (newcap * stride) as usize])?;
        let oldbuf = self.read_u64(base)?;
        if oldbuf != 0 {
            let vsz = self.size_of(valty);
            for i in 0..cap {
                let ob = oldbuf + i * stride;
                if self.read_u64(ob)? == 1 {
                    let kptr = self.read_u64(ob + 8)?;
                    let klen = self.read_u64(ob + 16)?;
                    let kbytes = self.read_bytes(kptr, klen)?;
                    let slot = self.map_find_empty(newbuf, newcap, stride, &kbytes)?;
                    let nb = newbuf + slot * stride;
                    self.write_u64(nb, 1)?;
                    self.write_u64(nb + 8, kptr)?;
                    self.write_u64(nb + 16, klen)?;
                    if vsz > 0 {
                        self.copy_bytes(nb + 24, ob + 24, vsz)?;
                    }
                }
            }
            self.call_free(ctx, vt, oldbuf, cap * stride, 8)?;
        }
        self.write_u64(base, newbuf)?;
        self.write_u64(base + 16, newcap)?;
        Ok(())
    }
}

impl Engine<'_> {
    /// Execute the static drop schedule for a value of `ty` at `addr` (INV-DROP),
    /// pruned by the lowering's static move mask (`moved` field-name paths; the
    /// current `path` is the position within the value). Mirrors the oracle's
    /// `drop_value`: a hooked struct runs its `<drop>` MIR fn (unless the value is
    /// partially moved) then drops fields in reverse; a `Box` frees through its
    /// stored vtable handle; an enum drops its active variant's payload; an array
    /// drops elements in reverse. Scalars / pointers / borrows are inert.
    fn drop_value(&mut self, addr: u64, ty: &Type, moved: &[Vec<String>], path: &mut Vec<String>) -> Result<(), Fault> {
        if is_moved(moved, path) {
            return Ok(());
        }
        match ty {
            Type::Array(elem, len) => {
                let n = self.lay().array_len(len);
                let stride = round_up(self.size_of(elem), self.align_of(elem));
                for i in (0..n).rev() {
                    self.drop_value(addr + i * stride, elem, moved, path)?;
                }
                Ok(())
            }
            Type::Box(inner) => self.drop_box(addr, inner),
            // Compiler-known `Vec[T]`: drop each live element (`0..len`, reverse),
            // then free the backing buffer through the carried allocator (mirrors
            // the oracle's `drop_value`). Popped elements decremented `len`.
            Type::App(n, args) if n == "Vec" => {
                let buf = self.read_u64(addr)?;
                if buf != 0 {
                    let elem = args.first().cloned().unwrap_or(Type::Error);
                    let stride = round_up(self.size_of(&elem), self.align_of(&elem));
                    let len = self.read_u64(addr + 8)?;
                    for i in (0..len).rev() {
                        self.drop_value(buf + i * stride, &elem, &[], &mut Vec::new())?;
                    }
                    let cap = self.read_u64(addr + 16)?;
                    let ctx = self.read_u64(addr + 24)?;
                    let vt = self.read_u64(addr + 32)?;
                    self.call_free(ctx, vt, buf, cap * stride, self.align_of(&elem))?;
                }
                Ok(())
            }
            // Compiler-known hash `Map[V]`: free each live key byte-copy, drop each
            // live value, then free the bucket buffer (alloc-on-drop).
            Type::App(n, args) if n == "Map" => {
                let buf = self.read_u64(addr)?;
                if buf != 0 {
                    let valty = args.first().cloned().unwrap_or(Type::Error);
                    let stride = round_up(24 + self.size_of(&valty), 8);
                    let cap = self.read_u64(addr + 16)?;
                    let ctx = self.read_u64(addr + 24)?;
                    let vt = self.read_u64(addr + 32)?;
                    for i in 0..cap {
                        let b = buf + i * stride;
                        if self.read_u64(b)? == 1 {
                            let kptr = self.read_u64(b + 8)?;
                            let klen = self.read_u64(b + 16)?;
                            self.call_free(ctx, vt, kptr, klen, 1)?;
                            self.drop_value(b + 24, &valty, &[], &mut Vec::new())?;
                        }
                    }
                    self.call_free(ctx, vt, buf, cap * stride, 8)?;
                }
                Ok(())
            }
            // Compiler-known `String` (design 0013): free its heap buffer through
            // the carried allocator (alloc-on-drop). Must precede the generic struct
            // arm — `String` is a synthesized nominal struct.
            Type::Named(n) if n == "String" => {
                let buf = self.read_u64(addr)?;
                if buf != 0 {
                    let cap = self.read_u64(addr + 16)?;
                    let ctx = self.read_u64(addr + 24)?;
                    let vt = self.read_u64(addr + 32)?;
                    self.call_free(ctx, vt, buf, cap, 1)?;
                }
                Ok(())
            }
            Type::Named(n) if self.items.lookup_struct(n).is_some() => {
                let partial = partially(moved, path);
                if !partial {
                    if let Some(hook) = self.prog.drop_hooks.get(n).cloned() {
                        self.call(&hook, &[addr as i128])?;
                    }
                }
                let (fields, _, _) = self.lay().struct_layout(n);
                for (fname, fty, off) in fields.into_iter().rev() {
                    path.push(fname);
                    self.drop_value(addr + off, &fty, moved, path)?;
                    path.pop();
                }
                Ok(())
            }
            Type::Named(n) if self.items.lookup_enum(n).is_some() => self.drop_enum(addr, ty, moved, path),
            Type::BoxResult(_) => self.drop_enum(addr, ty, moved, path),
            _ => Ok(()),
        }
    }

    fn drop_enum(&mut self, addr: u64, ty: &Type, moved: &[Vec<String>], path: &mut Vec<String>) -> Result<(), Fault> {
        let tag = self.read_u64(addr)? as usize;
        let einfo = match self.lay().enum_info(ty) {
            Some(e) => e,
            None => return Ok(()),
        };
        if tag >= einfo.len() {
            return Ok(());
        }
        let payloads = einfo[tag].1.clone();
        for i in (0..payloads.len()).rev() {
            let (pty, off) = self.lay().payload_offset(&payloads, i);
            path.push(format!("_{i}"));
            self.drop_value(addr + off, &pty, moved, path)?;
            path.pop();
        }
        Ok(())
    }

    fn drop_box(&mut self, addr: u64, inner: &Type) -> Result<(), Fault> {
        let ptr = self.read_u64(addr)?;
        let ctx = self.read_u64(addr + 8)?;
        let vt = self.read_u64(addr + 16)?;
        if ptr != 0 {
            self.drop_value(ptr, inner, &[], &mut Vec::new())?;
            let size = self.size_of(inner);
            let align = self.align_of(inner);
            self.call_free(ctx, vt, ptr, size, align)?;
        }
        Ok(())
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

fn is_wordy(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Scalar(_) | Type::Borrow(_) | Type::BorrowMut(_) | Type::RawPtr(_) | Type::FnPtr(_)
    )
}

fn scalar_of(ty: &Type) -> ScalarTy {
    match ty {
        Type::Scalar(s) => *s,
        // pointers / borrows / fn-pointers / slices-as-addr are u64 words.
        _ => ScalarTy::U64,
    }
}

/// The scalar type of an operand (a const carries it; a local reads its decl).
fn operand_sty(op: &Operand, mf: &MirFn) -> ScalarTy {
    match op {
        Operand::Const(_, s) => *s,
        Operand::Local(id) => scalar_of(&mf.locals[*id].ty),
    }
}

fn fault_of(edge: &FaultEdge) -> Fault {
    let msg = match edge.kind {
        FaultKind::Assert => "assertion failed",
        FaultKind::Panic => "panic",
        FaultKind::Overflow => "arithmetic overflow",
        FaultKind::ConvLoss => "conversion loses value",
        FaultKind::Bounds => "index out of bounds",
        _ => "fault",
    };
    Fault::new(edge.kind, edge.span, msg)
}

// ---------------------------------------------------------------------------
// Scalar arithmetic — replicated from the oracle (src/interp/eval.rs) so both
// engines fit/wrap/saturate/mask identically.
// ---------------------------------------------------------------------------

/// Convert an `f64` to a target integer scalar (design 0016 §5): truncate toward
/// zero, saturating on out-of-range and mapping NaN to 0 (Rust `as` semantics).
/// Returns the sign-correct logical value for `write_int`.
fn f64_to_int(f: f64, tsty: ScalarTy) -> i128 {
    match tsty {
        ScalarTy::I8 => f as i8 as i128,
        ScalarTy::I16 => f as i16 as i128,
        ScalarTy::I32 => f as i32 as i128,
        ScalarTy::I64 | ScalarTy::Isize => f as i64 as i128,
        ScalarTy::U8 => f as u8 as i128,
        ScalarTy::U16 => f as u16 as i128,
        ScalarTy::U32 => f as u32 as i128,
        ScalarTy::U64 | ScalarTy::Usize => f as u64 as i128,
        // Non-integer targets never reach here (the checker rejects them).
        ScalarTy::Bool | ScalarTy::Unit | ScalarTy::F64 | ScalarTy::F32 => 0,
    }
}

/// Convert an `f32` to a target integer scalar (design 0016 §5): truncate toward
/// zero, saturating on out-of-range and mapping NaN to 0 (Rust `as` semantics).
fn f32_to_int(f: f32, tsty: ScalarTy) -> i128 {
    match tsty {
        ScalarTy::I8 => f as i8 as i128,
        ScalarTy::I16 => f as i16 as i128,
        ScalarTy::I32 => f as i32 as i128,
        ScalarTy::I64 | ScalarTy::Isize => f as i64 as i128,
        ScalarTy::U8 => f as u8 as i128,
        ScalarTy::U16 => f as u16 as i128,
        ScalarTy::U32 => f as u32 as i128,
        ScalarTy::U64 | ScalarTy::Usize => f as u64 as i128,
        // Non-integer targets never reach here (the checker rejects them).
        ScalarTy::Bool | ScalarTy::Unit | ScalarTy::F64 | ScalarTy::F32 => 0,
    }
}

/// IEEE ordered/`==` comparison, shared by `f32`/`f64` (design 0016 §4).
fn float_ord_cmp<T: PartialOrd>(op: BinOp, a: T, b: T) -> bool {
    match op {
        BinOp::Eq => a == b,
        BinOp::Ne => a != b,
        BinOp::Lt => a < b,
        BinOp::Le => a <= b,
        BinOp::Gt => a > b,
        BinOp::Ge => a >= b,
        _ => unreachable!("non-comparison in a float compare"),
    }
}

/// IEEE `+ - * /` over a float scalar's raw bit pattern (design 0016 §2). `l`/`r`
/// are the operand bit patterns; the result is the pattern (`f32` zero-extended).
/// Never faults; regime-exempt.
fn float_arith(op: BinOp, sty: ScalarTy, l: u64, r: u64) -> u64 {
    use BinOp::*;
    if sty == ScalarTy::F32 {
        let (a, b) = (f32::from_bits(l as u32), f32::from_bits(r as u32));
        let res = match op {
            Add => a + b,
            Sub => a - b,
            Mul => a * b,
            Div => a / b,
            _ => unreachable!("only + - * / reach a float Bin"),
        };
        res.to_bits() as u64
    } else {
        let (a, b) = (f64::from_bits(l), f64::from_bits(r));
        let res = match op {
            Add => a + b,
            Sub => a - b,
            Mul => a * b,
            Div => a / b,
            _ => unreachable!("only + - * / reach a float Bin"),
        };
        res.to_bits()
    }
}

/// IEEE comparison over a float scalar's raw bits (design 0016 §4): any NaN operand
/// yields `false`, except `!=` (which is `true`).
fn float_cmp(op: BinOp, sty: ScalarTy, l: u64, r: u64) -> bool {
    if sty == ScalarTy::F32 {
        float_ord_cmp(op, f32::from_bits(l as u32), f32::from_bits(r as u32))
    } else {
        float_ord_cmp(op, f64::from_bits(l), f64::from_bits(r))
    }
}

/// IEEE negate (sign flip) over a float scalar's raw bits (design 0016).
fn float_sqrt(sty: ScalarTy, bits: u64) -> u64 {
    if sty == ScalarTy::F32 {
        f32::from_bits(bits as u32).sqrt().to_bits() as u64
    } else {
        f64::from_bits(bits).sqrt().to_bits()
    }
}
fn float_neg(sty: ScalarTy, bits: u64) -> u64 {
    if sty == ScalarTy::F32 {
        (-f32::from_bits(bits as u32)).to_bits() as u64
    } else {
        (-f64::from_bits(bits)).to_bits()
    }
}

/// A numeric `conv` where the source and/or target is a float (design 0016 §5):
/// int->float rounds; `f64`->`f32` rounds (narrowing, may -> `±inf`); `f32`->`f64`
/// is exact (widening); float->int truncates toward zero, saturating (NaN -> 0).
/// `x` is the source's sign-correct register value; the result is the target's.
fn float_conv(x: i128, from: ScalarTy, to: ScalarTy) -> i128 {
    use ScalarTy::*;
    match (from, to) {
        (F32, F32) | (F64, F64) => x,
        (F32, F64) => (f32::from_bits(x as u32) as f64).to_bits() as i128,
        (F64, F32) => (f64::from_bits(x as u64) as f32).to_bits() as i128,
        (F32, t) => f32_to_int(f32::from_bits(x as u32), t),
        (F64, t) => f64_to_int(f64::from_bits(x as u64), t),
        (_, F32) => (x as f32).to_bits() as i128,
        (_, F64) => (x as f64).to_bits() as i128,
        _ => unreachable!("float_conv called with no float operand"),
    }
}

fn ty_range(sty: ScalarTy) -> (i128, i128, u32, bool) {
    let (bits, signed): (u32, bool) = match sty {
        ScalarTy::I8 => (8, true),
        ScalarTy::I16 => (16, true),
        ScalarTy::I32 => (32, true),
        ScalarTy::I64 | ScalarTy::Isize => (64, true),
        ScalarTy::U8 => (8, false),
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

fn fit(value: i128, sty: ScalarTy, regime: Regime, fault: Option<&FaultEdge>) -> Result<i128, Fault> {
    let (min, max, bits, signed) = ty_range(sty);
    if value >= min && value <= max {
        return Ok(value);
    }
    match regime {
        Regime::Checked => {
            let edge = fault.expect("INV-CHECK: checked op lacks its fault edge");
            Err(Fault::new(edge.kind, edge.span, "arithmetic overflow"))
        }
        Regime::Wrapping => {
            let m = 1i128 << bits;
            let mut v = value.rem_euclid(m);
            if signed && v > max {
                v -= m;
            }
            Ok(v)
        }
        Regime::Saturating => Ok(value.clamp(min, max)),
    }
}

fn convert(v: i128, tsty: ScalarTy, regime: Regime, fault: Option<&FaultEdge>) -> Result<i128, Fault> {
    let (tmin, tmax, tbits, tsigned) = ty_range(tsty);
    if v >= tmin && v <= tmax {
        return Ok(v);
    }
    match regime {
        Regime::Checked => {
            let edge = fault.expect("INV-CHECK: checked conv lacks its fault edge");
            Err(Fault::new(edge.kind, edge.span, "conversion loses value"))
        }
        Regime::Wrapping => {
            let m = 1i128 << tbits;
            let mut x = v.rem_euclid(m);
            if tsigned && x > tmax {
                x -= m;
            }
            Ok(x)
        }
        Regime::Saturating => Ok(v.clamp(tmin, tmax)),
    }
}

fn fit_bits(value: i128, sty: ScalarTy) -> i128 {
    let (_, _, bits, signed) = ty_range(sty);
    let m = 1i128 << bits;
    let mut x = value.rem_euclid(m);
    if signed && x >= (m >> 1) {
        x -= m;
    }
    x
}

// ---------------------------------------------------------------------------
// Collection primitives replicated from the oracle (src/interp/eval.rs) so both
// engines hash / encode identically (the round-trip must be byte-exact).
// ---------------------------------------------------------------------------

/// 64-bit FNV-1a over the key bytes (offset basis 0xcbf29ce484222325, prime
/// 0x100000001b3) — the Map key hash, matching `interp::eval::map_hash`.
fn map_hash(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Is byte offset `i` a UTF-8 character boundary of the (well-formed) run `bytes`?
/// A boundary is the start, the end, or any non-continuation byte (`& 0xC0 != 0x80`)
/// — the `substr` boundary predicate, matching `interp::eval::str_is_boundary`.
fn str_is_boundary(bytes: &[u8], i: usize) -> bool {
    i == 0 || i == bytes.len() || (i < bytes.len() && (bytes[i] & 0xC0) != 0x80)
}

/// UTF-8-encode one Unicode scalar value, rejecting surrogates / out-of-range
/// code points (the `String::push` `is_scalar_value` backstop, matching the oracle).
fn utf8_encode_scalar(c: u32) -> Option<Vec<u8>> {
    if c > 0x10FFFF || (0xD800..=0xDFFF).contains(&c) {
        return None;
    }
    let ch = char::from_u32(c)?;
    let mut buf = [0u8; 4];
    Some(ch.encode_utf8(&mut buf).as_bytes().to_vec())
}
