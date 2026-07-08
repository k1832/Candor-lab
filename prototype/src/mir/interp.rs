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
    BinOp, FaultEdge, MirFn, MirProgram, Operand, Place, Proj, Regime, Rvalue, StatementKind,
    Terminator, UnOp,
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
    for st in &prog.statics {
        let out = engine.call(&st.init_fn, &[])?;
        let addr = engine.statics[&st.name];
        let size = engine.size_of(&st.ty);
        engine.copy_bytes(addr, out, size)?;
    }

    let ret_addr = engine.call("main", &[])?;
    let ret_i64 = match prog.get("main").map(|f| &f.locals[0].ty) {
        Some(Type::Scalar(ScalarTy::I64)) => engine.read_int(ret_addr, ScalarTy::I64)? as i64,
        _ => 0,
    };
    Ok(Run { ret: ret_i64, trace: engine.trace })
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
                let res = match op {
                    BinOp::Eq => a == b,
                    BinOp::Ne => a != b,
                    BinOp::Lt => a < b,
                    BinOp::Le => a <= b,
                    BinOp::Gt => a > b,
                    BinOp::Ge => a >= b,
                    _ => unreachable!("non-comparison in Cmp"),
                };
                Ok(res as i128)
            }
            Rvalue::Bin { op, regime, ty, l, r, span, fault } => {
                use BinOp::*;
                let lv = self.eval_operand(l, mf, frame)?;
                let rv2 = self.eval_operand(r, mf, frame)?;
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
                    UnOp::Neg => fit(-x, *ty, *regime, fault.as_ref()),
                    UnOp::BitNot => Ok(fit_bits(!x, *ty)),
                }
            }
            Rvalue::Conv { to, regime, v, fault } => {
                let x = self.eval_operand(v, mf, frame)?;
                convert(x, *to, *regime, fault.as_ref())
            }
            Rvalue::Call { func, args } => {
                let mut vals = Vec::with_capacity(args.len());
                for a in args {
                    vals.push(self.eval_operand(a, mf, frame)?);
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
