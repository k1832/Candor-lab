//! S0 — the first OPTIMIZED native backend (LLVM): MIR -> textual LLVM-IR, built
//! by `clang -O2`, linked against the same static C runtime (`aot_runtime.c`) the
//! Cranelift AOT object uses. This backend mirrors the *semantics* of
//! `backend::lower` (the Cranelift reference) in `.ll` text — it does not reuse
//! any Cranelift code.
//!
//! ## Scope (S0): scalar + control flow only
//! Integer/bool scalars; let/assign; if/else/while/loop/break/continue/return;
//! arithmetic (+ - * / %) with Checked/Wrapping/Saturating regimes and
//! Overflow/DivByZero/ConvLoss faults; comparisons; &&/||; bitwise/shift; unary
//! neg/not; `trace(x)`; assert/panic; requires/ensures; direct fn calls +
//! recursion. Aggregates/pointers/box/collections/concurrency are out of subset
//! and rejected with a precise error (never faked).
//!
//! ## The perf recipe (Tier-R): alloca-per-local
//! Every scalar local is an `alloca i64` in the entry block; a use is a `load`, a
//! definition a `store`. No local's address is taken and there are no aggregates,
//! so `clang -O2`'s mem2reg promotes every slot to an SSA register — that is where
//! the optimization comes from. Scalars never route through the flat MEM_BASE
//! buffer (that would defeat mem2reg), unlike the aggregate path in `lower`.
//!
//! ## Correctness invariants preserved (mirroring `lower`)
//! * **INV-CHECK.** Checked add/sub/mul use `llvm.{s,u}{add,sub,mul}.with.overflow`
//!   — NEVER `add nsw`/`mul nsw` (LLVM would delete the overflow test as
//!   UB-unreachable). The overflow bit is an explicit `br i1` to a fault block.
//!   Conv/neg range checks are explicit `icmp` in i128 against the target range.
//! * **INV-FAULT-ID.** Every fault edge is `call void @rt_fault(kind, s_start,
//!   s_end)` then `unreachable`, with the stable `kind_code` map.
//! * **INV-OBS-ORDER.** `trace`/`rt_fault` are bare external declares (no
//!   `readnone`/`memory(none)`): side-effecting external calls are optimization
//!   barriers, so `-O2` preserves trace order and fault points.

use std::path::Path;
use std::process::Command;

use crate::ast::{BinOp, UnOp};
use crate::interp::FaultKind;
use crate::mir::{
    FaultEdge, MirFn, MirProgram, Operand, Regime, Rvalue, Statement, StatementKind, Terminator,
};
use crate::span::Span;
use crate::token::ScalarTy;
use crate::types::Type;

use super::lower::kind_code;

/// The static C runtime, reused UNCHANGED (the AOT twin of `runtime.rs`). Linked
/// by `clang` exactly as the Cranelift object links it.
const RUNTIME_C: &str = include_str!("aot_runtime.c");

/// (min, max, bits, signed) for a scalar type — the ranges the interpreter and the
/// oracle use (a private mirror of `lower::ty_range`).
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

fn scalar_of(ty: &Type) -> ScalarTy {
    match ty {
        Type::Scalar(s) => *s,
        _ => ScalarTy::U64,
    }
}

fn overflow_intrinsic(op: BinOp, signed: bool, bits: u32) -> String {
    let base = match op {
        BinOp::Add => if signed { "sadd" } else { "uadd" },
        BinOp::Sub => if signed { "ssub" } else { "usub" },
        BinOp::Mul => if signed { "smul" } else { "umul" },
        _ => unreachable!("overflow intrinsic for non add/sub/mul"),
    };
    format!("llvm.{base}.with.overflow.i{bits}")
}

/// Emit the whole program as one textual LLVM-IR module: the `rt_*` + intrinsic
/// declares, one `cnf_<name>` function per `MirFn`, then the `candor_entry` glue.
pub fn emit_ll(prog: &MirProgram) -> Result<String, String> {
    if prog.get("main").is_none() {
        return Err("no `main` function to compile".to_string());
    }
    let mut out = String::new();
    out.push_str("; Candor LLVM-S0 module\n");
    out.push_str("declare void @rt_trace(i64)\n");
    out.push_str("declare void @rt_fault(i32, i32, i32) #0\n\n");
    for bits in [8u32, 16, 32, 64] {
        for op in [BinOp::Add, BinOp::Sub, BinOp::Mul] {
            for signed in [true, false] {
                let name = overflow_intrinsic(op, signed, bits);
                out.push_str(&format!(
                    "declare {{i{bits}, i1}} @{name}(i{bits}, i{bits})\n"
                ));
            }
        }
    }
    out.push('\n');

    for f in &prog.fns {
        emit_fn(&mut out, f)?;
    }
    emit_entry(&mut out, prog);

    out.push_str("\nattributes #0 = { noreturn }\n");
    Ok(out)
}

/// The `candor_entry() -> i64` glue the runtime calls (mirrors `lower::lower_entry`
/// for the scalar subset: no statics/strings, just call `main`).
fn emit_entry(out: &mut String, prog: &MirProgram) {
    let main_is_i64 = matches!(
        prog.get("main").map(|f| &f.locals[0].ty),
        Some(Type::Scalar(ScalarTy::I64))
    );
    out.push_str("define i64 @candor_entry() {\nentry:\n");
    out.push_str("  %r = call i64 @cnf_main()\n");
    if main_is_i64 {
        out.push_str("  ret i64 %r\n");
    } else {
        out.push_str("  ret i64 0\n");
    }
    out.push_str("}\n");
}

/// DFS successors from `entry`, tagging each unvisited MIR block with `target`
/// (its return destination / region membership) — mirrors `lower::assign_region`.
fn assign_region(mf: &MirFn, entry: usize, target: &str, ret_target: &mut [Option<String>]) {
    let mut stack = vec![entry];
    while let Some(bid) = stack.pop() {
        if ret_target[bid].is_some() {
            continue;
        }
        ret_target[bid] = Some(target.to_string());
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

/// Per-function textual-IR emitter. Values are named `%v<n>`, synthetic blocks
/// `L<n>`, MIR blocks `mbb<bid>`; naming everything sidesteps LLVM's implicit
/// numbering of unnamed values/blocks.
struct FnEmit<'a> {
    out: String,
    tmp: usize,
    lbl: usize,
    mf: &'a MirFn,
}

impl<'a> FnEmit<'a> {
    fn new(mf: &'a MirFn) -> Self {
        FnEmit { out: String::new(), tmp: 0, lbl: 0, mf }
    }
    fn t(&mut self) -> String {
        let n = self.tmp;
        self.tmp += 1;
        format!("%v{n}")
    }
    fn l(&mut self) -> String {
        let n = self.lbl;
        self.lbl += 1;
        format!("L{n}")
    }
    fn raw(&mut self, s: &str) {
        self.out.push_str(s);
    }
    fn line(&mut self, s: &str) {
        self.out.push_str("  ");
        self.out.push_str(s);
        self.out.push('\n');
    }
    fn label(&mut self, name: &str) {
        self.out.push_str(name);
        self.out.push_str(":\n");
    }

    /// A scalar operand as an LLVM i64 value (an immediate literal or a `%v<n>`
    /// loaded from the local's slot). Locals always hold the canonical i64.
    fn operand(&mut self, op: &Operand) -> String {
        match op {
            Operand::Const(v, _) => format!("{}", *v as i64),
            Operand::Local(id) => {
                let r = self.t();
                self.line(&format!("{r} = load i64, ptr %loc{id}"));
                r
            }
        }
    }
    fn operand_sty(&self, op: &Operand) -> ScalarTy {
        match op {
            Operand::Const(_, s) => *s,
            Operand::Local(id) => scalar_of(&self.mf.locals[*id].ty),
        }
    }

    /// Reduce an i64 to the canonical i64 of `sty` (trunc then sign/zero extend) —
    /// mirrors `lower::canon`.
    fn canon(&mut self, v: &str, sty: ScalarTy) -> String {
        let (_, _, bits, signed) = ty_range(sty);
        if bits >= 64 {
            return v.to_string();
        }
        let tr = self.t();
        self.line(&format!("{tr} = trunc i64 {v} to i{bits}"));
        let ex = self.t();
        let opn = if signed { "sext" } else { "zext" };
        self.line(&format!("{ex} = {opn} i{bits} {tr} to i64"));
        ex
    }
    fn ext128(&mut self, v: &str, sty: ScalarTy) -> String {
        let (_, _, _, signed) = ty_range(sty);
        let r = self.t();
        let opn = if signed { "sext" } else { "zext" };
        self.line(&format!("{r} = {opn} i64 {v} to i128"));
        r
    }
    fn fit128(&mut self, v128: &str, sty: ScalarTy) -> String {
        let t = self.t();
        self.line(&format!("{t} = trunc i128 {v128} to i64"));
        self.canon(&t, sty)
    }

    /// Emit `call rt_fault(k, s, e)` + `unreachable` (INV-FAULT-ID).
    fn emit_fault(&mut self, kind: FaultKind, span: Span) {
        self.line(&format!(
            "call void @rt_fault(i32 {}, i32 {}, i32 {})",
            kind_code(kind),
            span.start,
            span.end
        ));
        self.line("unreachable");
    }
    /// Branch to a fresh fault block when `cond` (i1) is true; continue otherwise.
    fn fault_if(&mut self, cond: &str, kind: FaultKind, span: Span) {
        let fl = self.l();
        let cont = self.l();
        self.line(&format!("br i1 {cond}, label %{fl}, label %{cont}"));
        self.label(&fl);
        self.emit_fault(kind, span);
        self.label(&cont);
    }

    /// Range-check the i128 value against `sty`; deliver/fit per regime (add/sub/
    /// mul/neg/conv share this — INV-CHECK; mirrors `lower::range_or_fit`).
    fn range_or_fit(
        &mut self,
        v128: &str,
        sty: ScalarTy,
        regime: Regime,
        fault: Option<&FaultEdge>,
    ) -> Result<String, String> {
        let (min, max, _, _) = ty_range(sty);
        match regime {
            Regime::Checked => {
                let lt = self.t();
                self.line(&format!("{lt} = icmp slt i128 {v128}, {min}"));
                let gt = self.t();
                self.line(&format!("{gt} = icmp sgt i128 {v128}, {max}"));
                let bad = self.t();
                self.line(&format!("{bad} = or i1 {lt}, {gt}"));
                let edge = fault.ok_or("INV-CHECK: checked op lacks its fault edge")?;
                self.fault_if(&bad, edge.kind, edge.span);
                Ok(self.fit128(v128, sty))
            }
            Regime::Wrapping => Ok(self.fit128(v128, sty)),
            Regime::Saturating => {
                let lt = self.t();
                self.line(&format!("{lt} = icmp slt i128 {v128}, {min}"));
                let gt = self.t();
                self.line(&format!("{gt} = icmp sgt i128 {v128}, {max}"));
                let fit = self.fit128(v128, sty);
                let lo = self.t();
                self.line(&format!("{lo} = select i1 {lt}, i64 {min}, i64 {fit}"));
                let r = self.t();
                self.line(&format!("{r} = select i1 {gt}, i64 {max}, i64 {lo}"));
                Ok(r)
            }
        }
    }

    fn eval_rvalue(&mut self, rv: &Rvalue) -> Result<String, String> {
        match rv {
            Rvalue::Use(op) => Ok(self.operand(op)),
            Rvalue::Cmp { op, l, r } => {
                let lsty = self.operand_sty(l);
                let rsty = self.operand_sty(r);
                let lv = self.operand(l);
                let rv = self.operand(r);
                let l128 = self.ext128(&lv, lsty);
                let r128 = self.ext128(&rv, rsty);
                let cc = match op {
                    BinOp::Eq => "eq",
                    BinOp::Ne => "ne",
                    BinOp::Lt => "slt",
                    BinOp::Le => "sle",
                    BinOp::Gt => "sgt",
                    BinOp::Ge => "sge",
                    _ => return Err(format!("non-comparison {op:?} in Cmp")),
                };
                let c = self.t();
                self.line(&format!("{c} = icmp {cc} i128 {l128}, {r128}"));
                let r = self.t();
                self.line(&format!("{r} = zext i1 {c} to i64"));
                Ok(r)
            }
            Rvalue::Bin { op, regime, ty, l, r, span, fault } => {
                self.eval_bin(*op, *regime, *ty, l, r, *span, fault.as_ref())
            }
            Rvalue::Un { op, regime, ty, v, fault } => {
                let x = self.operand(v);
                match op {
                    UnOp::Not => {
                        let c = self.t();
                        self.line(&format!("{c} = icmp eq i64 {x}, 0"));
                        let r = self.t();
                        self.line(&format!("{r} = zext i1 {c} to i64"));
                        Ok(r)
                    }
                    UnOp::BitNot => {
                        let n = self.t();
                        self.line(&format!("{n} = xor i64 {x}, -1"));
                        Ok(self.canon(&n, *ty))
                    }
                    UnOp::Neg => {
                        let x128 = self.ext128(&x, *ty);
                        let neg = self.t();
                        self.line(&format!("{neg} = sub i128 0, {x128}"));
                        self.range_or_fit(&neg, *ty, *regime, fault.as_ref())
                    }
                }
            }
            Rvalue::Conv { to, regime, v, fault } => {
                let sty = self.operand_sty(v);
                let x = self.operand(v);
                let x128 = self.ext128(&x, sty);
                self.range_or_fit(&x128, *to, *regime, fault.as_ref())
            }
            Rvalue::Call { func, args } => {
                let vals: Vec<String> = args.iter().map(|a| self.operand(a)).collect();
                let argstr = vals
                    .iter()
                    .map(|v| format!("i64 {v}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                let r = self.t();
                self.line(&format!("{r} = call i64 @cnf_{func}({argstr})"));
                Ok(r)
            }
            other => Err(format!("out of LLVM-S0 scalar subset: {other:?}")),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn eval_bin(
        &mut self,
        op: BinOp,
        regime: Regime,
        ty: ScalarTy,
        l: &Operand,
        r: &Operand,
        span: Span,
        fault: Option<&FaultEdge>,
    ) -> Result<String, String> {
        use BinOp::*;
        let lv = self.operand(l);
        let rv = self.operand(r);
        let (min, max, bits, signed) = ty_range(ty);
        match op {
            Add | Sub | Mul => match regime {
                Regime::Checked => {
                    let (a, b) = if bits >= 64 {
                        (lv.clone(), rv.clone())
                    } else {
                        let a = self.t();
                        self.line(&format!("{a} = trunc i64 {lv} to i{bits}"));
                        let b = self.t();
                        self.line(&format!("{b} = trunc i64 {rv} to i{bits}"));
                        (a, b)
                    };
                    let name = overflow_intrinsic(op, signed, bits);
                    let p = self.t();
                    self.line(&format!(
                        "{p} = call {{i{bits}, i1}} @{name}(i{bits} {a}, i{bits} {b})"
                    ));
                    let res = self.t();
                    self.line(&format!("{res} = extractvalue {{i{bits}, i1}} {p}, 0"));
                    let ovf = self.t();
                    self.line(&format!("{ovf} = extractvalue {{i{bits}, i1}} {p}, 1"));
                    let edge = fault.ok_or("INV-CHECK: checked arith lacks its fault edge")?;
                    self.fault_if(&ovf, edge.kind, edge.span);
                    if bits >= 64 {
                        Ok(res)
                    } else {
                        let ex = self.t();
                        let e = if signed { "sext" } else { "zext" };
                        self.line(&format!("{ex} = {e} i{bits} {res} to i64"));
                        Ok(ex)
                    }
                }
                Regime::Wrapping => {
                    let opn = match op { Add => "add", Sub => "sub", _ => "mul" };
                    if bits >= 64 {
                        let r = self.t();
                        self.line(&format!("{r} = {opn} i64 {lv}, {rv}"));
                        Ok(r)
                    } else {
                        let a = self.t();
                        self.line(&format!("{a} = trunc i64 {lv} to i{bits}"));
                        let b = self.t();
                        self.line(&format!("{b} = trunc i64 {rv} to i{bits}"));
                        let w = self.t();
                        self.line(&format!("{w} = {opn} i{bits} {a}, {b}"));
                        let ex = self.t();
                        let e = if signed { "sext" } else { "zext" };
                        self.line(&format!("{ex} = {e} i{bits} {w} to i64"));
                        Ok(ex)
                    }
                }
                Regime::Saturating => {
                    let a128 = self.ext128(&lv, ty);
                    let b128 = self.ext128(&rv, ty);
                    let opn = match op { Add => "add", Sub => "sub", _ => "mul" };
                    let res = self.t();
                    self.line(&format!("{res} = {opn} i128 {a128}, {b128}"));
                    self.range_or_fit(&res, ty, Regime::Saturating, None)
                }
            },
            Div | Rem => {
                let z = self.t();
                self.line(&format!("{z} = icmp eq i64 {rv}, 0"));
                self.fault_if(&z, FaultKind::DivByZero, span);
                if !signed {
                    let opn = if op == Div { "udiv" } else { "urem" };
                    let q = self.t();
                    self.line(&format!("{q} = {opn} i64 {lv}, {rv}"));
                    Ok(self.canon(&q, ty))
                } else {
                    let ismin = self.t();
                    self.line(&format!("{ismin} = icmp eq i64 {lv}, {min}"));
                    let ism1 = self.t();
                    self.line(&format!("{ism1} = icmp eq i64 {rv}, -1"));
                    let ov = self.t();
                    self.line(&format!("{ov} = and i1 {ismin}, {ism1}"));
                    if op == Rem {
                        let safe = self.t();
                        self.line(&format!("{safe} = select i1 {ov}, i64 1, i64 {rv}"));
                        let rr = self.t();
                        self.line(&format!("{rr} = srem i64 {lv}, {safe}"));
                        let sel = self.t();
                        self.line(&format!("{sel} = select i1 {ov}, i64 0, i64 {rr}"));
                        Ok(self.canon(&sel, ty))
                    } else {
                        match regime {
                            Regime::Checked => {
                                let edge =
                                    fault.ok_or("INV-CHECK: checked div lacks its fault edge")?;
                                self.fault_if(&ov, edge.kind, edge.span);
                                let q = self.t();
                                self.line(&format!("{q} = sdiv i64 {lv}, {rv}"));
                                Ok(self.canon(&q, ty))
                            }
                            Regime::Wrapping => {
                                let safe = self.t();
                                self.line(&format!("{safe} = select i1 {ov}, i64 1, i64 {rv}"));
                                let q = self.t();
                                self.line(&format!("{q} = sdiv i64 {lv}, {safe}"));
                                let sel = self.t();
                                self.line(&format!("{sel} = select i1 {ov}, i64 {min}, i64 {q}"));
                                Ok(self.canon(&sel, ty))
                            }
                            Regime::Saturating => {
                                let safe = self.t();
                                self.line(&format!("{safe} = select i1 {ov}, i64 1, i64 {rv}"));
                                let q = self.t();
                                self.line(&format!("{q} = sdiv i64 {lv}, {safe}"));
                                let sel = self.t();
                                self.line(&format!("{sel} = select i1 {ov}, i64 {max}, i64 {q}"));
                                Ok(self.canon(&sel, ty))
                            }
                        }
                    }
                }
            }
            BitAnd | BitOr | BitXor => {
                let opn = match op { BitAnd => "and", BitOr => "or", _ => "xor" };
                let r = self.t();
                self.line(&format!("{r} = {opn} i64 {lv}, {rv}"));
                Ok(self.canon(&r, ty))
            }
            Shl | Shr => {
                let neg = self.t();
                self.line(&format!("{neg} = icmp slt i64 {rv}, 0"));
                self.fault_if(&neg, FaultKind::Overflow, span);
                let ge = self.t();
                self.line(&format!("{ge} = icmp uge i64 {rv}, {bits}"));
                let amt = match regime {
                    Regime::Checked => {
                        self.fault_if(&ge, FaultKind::Overflow, span);
                        rv.clone()
                    }
                    Regime::Wrapping => {
                        let m = self.t();
                        self.line(&format!("{m} = urem i64 {rv}, {bits}"));
                        let a = self.t();
                        self.line(&format!("{a} = select i1 {ge}, i64 {m}, i64 {rv}"));
                        a
                    }
                    Regime::Saturating => {
                        let a = self.t();
                        self.line(&format!("{a} = select i1 {ge}, i64 {}, i64 {rv}", bits - 1));
                        a
                    }
                };
                let opn = if op == Shl {
                    "shl"
                } else if signed {
                    "ashr"
                } else {
                    "lshr"
                };
                let raw = self.t();
                self.line(&format!("{raw} = {opn} i64 {lv}, {amt}"));
                Ok(self.canon(&raw, ty))
            }
            _ => Err(format!("comparison/logical {op:?} in Bin")),
        }
    }

    fn lower_stmt(&mut self, st: &Statement) -> Result<(), String> {
        if st.observable && !matches!(st.kind, StatementKind::Trace(_)) {
            return Err(format!("out of LLVM-S0 subset (observable): {:?}", st.kind));
        }
        match &st.kind {
            StatementKind::Assign(local, rv) => {
                let v = self.eval_rvalue(rv)?;
                self.line(&format!("store i64 {v}, ptr %loc{local}"));
            }
            StatementKind::Trace(op) => {
                let v = self.operand(op);
                self.line(&format!("call void @rt_trace(i64 {v})"));
            }
            StatementKind::Store(place, rv) => {
                if !place.proj.is_empty() {
                    return Err("out of LLVM-S0 subset: projected store".to_string());
                }
                let v = self.eval_rvalue(rv)?;
                self.line(&format!("store i64 {v}, ptr %loc{}", place.root));
            }
            // A scalar local carries no drop obligation, so its Drop is a no-op;
            // an aggregate drop is out of subset.
            StatementKind::Drop { local, .. } => {
                if !matches!(self.mf.locals[*local].ty, Type::Scalar(_)) {
                    return Err("out of LLVM-S0 subset: drop of a non-scalar".to_string());
                }
            }
            other => return Err(format!("out of LLVM-S0 subset: {other:?}")),
        }
        Ok(())
    }
}

/// Emit one `cnf_<name>` function, replicating `lower::lower_fn`'s block/region
/// structure (requires/ensures predicate regions, the shared final-return block).
fn emit_fn(out: &mut String, mf: &MirFn) -> Result<(), String> {
    let mut e = FnEmit::new(mf);

    let params = (0..mf.num_params)
        .map(|i| format!("i64 %a{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    e.raw(&format!("define internal i64 @cnf_{}({params}) {{\n", mf.name));
    e.label("entry");
    for i in 0..mf.locals.len() {
        e.line(&format!("%loc{i} = alloca i64"));
    }
    for i in 0..mf.num_params {
        e.line(&format!("store i64 %a{i}, ptr %loc{}", 1 + i));
    }

    let final_return = "ret_final".to_string();
    let has_ens = !mf.ensures.is_empty();
    let ensures_start = "ens_start".to_string();
    let body_ret = if has_ens { ensures_start.clone() } else { final_return.clone() };
    let req_labels: Vec<String> = (0..mf.requires.len()).map(|i| format!("req{i}")).collect();
    let ens_labels: Vec<String> = (0..mf.ensures.len()).map(|i| format!("ens{i}")).collect();

    let mut ret_target: Vec<Option<String>> = vec![None; mf.blocks.len()];
    for (i, pred) in mf.requires.iter().enumerate() {
        assign_region(mf, pred.entry, &req_labels[i], &mut ret_target);
    }
    assign_region(mf, mf.entry, &body_ret, &mut ret_target);
    for (i, pred) in mf.ensures.iter().enumerate() {
        assign_region(mf, pred.entry, &ens_labels[i], &mut ret_target);
    }

    let first = match mf.requires.first() {
        Some(p) => format!("mbb{}", p.entry),
        None => format!("mbb{}", mf.entry),
    };
    e.line(&format!("br label %{first}"));

    for (i, pred) in mf.requires.iter().enumerate() {
        e.label(&req_labels[i]);
        let v = e.t();
        e.line(&format!("{v} = load i64, ptr %loc{}", pred.value));
        let c = e.t();
        e.line(&format!("{c} = trunc i64 {v} to i1"));
        let ok = if i + 1 < mf.requires.len() {
            format!("mbb{}", mf.requires[i + 1].entry)
        } else {
            format!("mbb{}", mf.entry)
        };
        let fl = e.l();
        e.line(&format!("br i1 {c}, label %{ok}, label %{fl}"));
        e.label(&fl);
        e.emit_fault(pred.kind, pred.span);
    }

    if has_ens {
        e.label(&ensures_start);
        if let Some(rl) = mf.result_local {
            let v = e.t();
            e.line(&format!("{v} = load i64, ptr %loc0"));
            e.line(&format!("store i64 {v}, ptr %loc{rl}"));
        }
        e.line(&format!("br label %mbb{}", mf.ensures[0].entry));
    }
    for (i, pred) in mf.ensures.iter().enumerate() {
        e.label(&ens_labels[i]);
        let v = e.t();
        e.line(&format!("{v} = load i64, ptr %loc{}", pred.value));
        let c = e.t();
        e.line(&format!("{c} = trunc i64 {v} to i1"));
        let ok = if i + 1 < mf.ensures.len() {
            format!("mbb{}", mf.ensures[i + 1].entry)
        } else {
            final_return.clone()
        };
        let fl = e.l();
        e.line(&format!("br i1 {c}, label %{ok}, label %{fl}"));
        e.label(&fl);
        e.emit_fault(pred.kind, pred.span);
    }

    e.label(&final_return);
    let v = e.t();
    e.line(&format!("{v} = load i64, ptr %loc0"));
    e.line(&format!("ret i64 {v}"));

    for (bid, block) in mf.blocks.iter().enumerate() {
        e.label(&format!("mbb{bid}"));
        if ret_target[bid].is_none() {
            e.line("unreachable");
            continue;
        }
        for st in &block.stmts {
            e.lower_stmt(st)?;
        }
        match &block.term {
            Terminator::Goto(n) => e.line(&format!("br label %mbb{n}")),
            Terminator::Branch { cond, then_bb, else_bb } => {
                let c = e.operand(cond);
                let ci = e.t();
                e.line(&format!("{ci} = trunc i64 {c} to i1"));
                e.line(&format!(
                    "br i1 {ci}, label %mbb{then_bb}, label %mbb{else_bb}"
                ));
            }
            Terminator::Return => {
                let tgt = ret_target[bid].clone().unwrap();
                e.line(&format!("br label %{tgt}"));
            }
            Terminator::Fault(edge) => e.emit_fault(edge.kind, edge.span),
        }
    }

    e.raw("}\n\n");
    out.push_str(&e.out);
    Ok(())
}

/// Build the emitted module with `clang -O2` and link it against the reused static
/// C runtime into a standalone native executable at `out`.
pub fn link_ll(ll: &str, out: &Path) -> Result<(), String> {
    let tmp = std::env::temp_dir().join(format!(
        "candor-llvm-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let ll_path = tmp.join("candor.ll");
    let rt_path = tmp.join("aot_runtime.c");
    std::fs::write(&ll_path, ll).map_err(|e| e.to_string())?;
    std::fs::write(&rt_path, RUNTIME_C).map_err(|e| e.to_string())?;

    let status = Command::new("clang")
        .arg("-O2")
        .arg("-Wno-override-module")
        .arg(&ll_path)
        .arg(&rt_path)
        .arg("-o")
        .arg(out)
        .arg("-no-pie")
        .arg("-pthread")
        .status()
        .map_err(|e| format!("could not invoke clang: {e}"))?;

    let _ = std::fs::remove_dir_all(&tmp);
    if !status.success() {
        return Err(format!("clang -O2 failed with status {status}"));
    }
    Ok(())
}
