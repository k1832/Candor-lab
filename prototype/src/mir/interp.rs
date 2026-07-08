//! The MIR interpreter (design 0010 §5): a *precise* execution engine over the
//! checked MIR — a different engine from the tree-walking oracle, sharing the
//! same semantics. Every fault edge is taken immediately (zero-width window,
//! R1–R3-free): the `density → 1` precise limit of the formalization §10.
//!
//! ## Memory-substrate decision (design 0010 §2/§5 "DECIDE")
//! Stage A's gate is `(k, s, θ)` *semantic-trace* equality, not byte-state
//! equality. For the scalar/control/contract core a **typed value model** (a
//! sign-correct `i128` plus its `ScalarTy`) reproduces `θ` exactly while staying
//! small and auditable. The flat `interp::mem::Mem` substrate is the documented
//! next step: it becomes semantically load-bearing only once rawptr/MMIO/`Box`
//! observables and aggregate layout enter the subset, where sharing it with the
//! oracle is the right call. The engine is structured so that swap is local.

use crate::interp::{Fault, FaultKind, Run};
use crate::span::Span;
use crate::token::ScalarTy;
use crate::types::Type;

use super::{
    BinOp, FaultEdge, MirFn, MirProgram, Operand, Regime, Rvalue, StatementKind, Terminator, UnOp,
};

/// A runtime value: a sign-correct integer/boolean with its scalar type. `Unit`
/// is `(0, Unit)`.
#[derive(Clone, Copy, Debug)]
struct Value {
    v: i128,
}

impl Value {
    fn scalar(v: i128, _ty: ScalarTy) -> Value {
        Value { v }
    }
    fn unit() -> Value {
        Value { v: 0 }
    }
    fn boolean(b: bool) -> Value {
        Value { v: b as i128 }
    }
    fn truthy(&self) -> bool {
        self.v != 0
    }
}

/// Run a lowered program's `main` precisely, returning its `(ret, θ)` outcome or
/// the delivered fault `f★`.
pub fn run(prog: &MirProgram) -> Result<Run, Fault> {
    let mut engine = Engine { prog, trace: Vec::new(), depth: 0 };
    let ret = engine.call("main", &[])?;
    let ret_i64 = match prog.get("main").map(|f| &f.locals[0].ty) {
        Some(Type::Scalar(ScalarTy::I64)) => ret.v as i64,
        _ => 0,
    };
    Ok(Run { ret: ret_i64, trace: engine.trace })
}

struct Engine<'a> {
    prog: &'a MirProgram,
    trace: Vec<i64>,
    depth: usize,
}

const MAX_DEPTH: usize = 4096;

impl<'a> Engine<'a> {
    fn call(&mut self, name: &str, args: &[Value]) -> Result<Value, Fault> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(Fault::new(FaultKind::Panic, Span::point(0), "MIR recursion limit"));
        }
        let mf = self
            .prog
            .get(name)
            .ok_or_else(|| Fault::new(FaultKind::Panic, Span::point(0), format!("no MIR fn `{name}`")))?;
        let mut frame: Vec<Value> = mf
            .locals
            .iter()
            .map(|l| default_value(&l.ty))
            .collect();
        for (i, a) in args.iter().enumerate() {
            frame[1 + i] = *a;
        }
        // requires (design 0001 §7.3).
        for pred in &mf.requires {
            self.run_cfg(mf, pred.entry, &mut frame)?;
            if !frame[pred.value].truthy() {
                self.depth -= 1;
                return Err(Fault::new(pred.kind, pred.span, "`requires` clause violated"));
            }
        }
        // body.
        self.run_cfg(mf, mf.entry, &mut frame)?;
        let ret = frame[0];
        // ensures, reading `result`.
        if !mf.ensures.is_empty() {
            if let Some(rl) = mf.result_local {
                frame[rl] = ret;
            }
            for pred in &mf.ensures {
                self.run_cfg(mf, pred.entry, &mut frame)?;
                if !frame[pred.value].truthy() {
                    self.depth -= 1;
                    return Err(Fault::new(pred.kind, pred.span, "`ensures` clause violated"));
                }
            }
        }
        self.depth -= 1;
        Ok(ret)
    }

    /// Execute the CFG from `entry` until a `Return` (Ok) or a fault edge (Err).
    fn run_cfg(&mut self, mf: &MirFn, entry: usize, frame: &mut [Value]) -> Result<(), Fault> {
        let mut bb = entry;
        loop {
            let block = &mf.blocks[bb];
            for st in &block.stmts {
                match &st.kind {
                    StatementKind::Assign(local, rv) => {
                        let val = self.eval_rvalue(rv, frame)?;
                        frame[*local] = val;
                    }
                    StatementKind::Trace(op) => {
                        let val = eval_operand(op, frame);
                        self.trace.push(val.v as i64);
                    }
                    StatementKind::Drop(_) => { /* drop-inert in the scalar subset */ }
                }
            }
            match &block.term {
                Terminator::Goto(next) => bb = *next,
                Terminator::Branch { cond, then_bb, else_bb } => {
                    bb = if eval_operand(cond, frame).truthy() { *then_bb } else { *else_bb };
                }
                Terminator::Return => return Ok(()),
                Terminator::Fault(edge) => return Err(fault_of(edge)),
            }
        }
    }

    fn eval_rvalue(&mut self, rv: &Rvalue, frame: &[Value]) -> Result<Value, Fault> {
        match rv {
            Rvalue::Use(op) => Ok(eval_operand(op, frame)),
            Rvalue::Cmp { op, l, r } => {
                let a = eval_operand(l, frame).v;
                let b = eval_operand(r, frame).v;
                let res = match op {
                    BinOp::Eq => a == b,
                    BinOp::Ne => a != b,
                    BinOp::Lt => a < b,
                    BinOp::Le => a <= b,
                    BinOp::Gt => a > b,
                    BinOp::Ge => a >= b,
                    _ => unreachable!("non-comparison in Cmp"),
                };
                Ok(Value::boolean(res))
            }
            Rvalue::Bin { op, regime, ty, l, r, span, fault } => {
                use BinOp::*;
                let lv = eval_operand(l, frame).v;
                let rv = eval_operand(r, frame).v;
                let (regime, ty, span, fault) = (*regime, *ty, *span, fault.as_ref());
                let out = match op {
                    Add => fit(lv + rv, ty, regime, fault)?,
                    Sub => fit(lv - rv, ty, regime, fault)?,
                    Mul => fit(lv * rv, ty, regime, fault)?,
                    Div | Rem => {
                        if rv == 0 {
                            return Err(Fault::new(FaultKind::DivByZero, span, "division by zero"));
                        }
                        let raw = if *op == Div { lv / rv } else { lv % rv };
                        fit(raw, ty, regime, fault)?
                    }
                    BitAnd => fit_bits(lv & rv, ty),
                    BitOr => fit_bits(lv | rv, ty),
                    BitXor => fit_bits(lv ^ rv, ty),
                    Shl | Shr => {
                        let bits = ty_range(ty).2 as i128;
                        let amt = if rv < 0 {
                            return Err(Fault::new(FaultKind::Overflow, span, "negative shift amount"));
                        } else if rv >= bits {
                            match regime {
                                Regime::Checked => {
                                    return Err(Fault::new(FaultKind::Overflow, span, "shift amount exceeds bit width"));
                                }
                                Regime::Wrapping => rv % bits,
                                Regime::Saturating => bits - 1,
                            }
                        } else {
                            rv
                        };
                        let raw = if *op == Shl { lv << amt } else { lv >> amt };
                        fit_bits(raw, ty)
                    }
                    _ => unreachable!("comparison/logical in Bin"),
                };
                Ok(Value::scalar(out, ty))
            }
            Rvalue::Un { op, regime, ty, v, fault } => {
                let x = eval_operand(v, frame).v;
                match op {
                    UnOp::Not => Ok(Value::boolean(x == 0)),
                    UnOp::Neg => Ok(Value::scalar(fit(-x, *ty, *regime, fault.as_ref())?, *ty)),
                    UnOp::BitNot => Ok(Value::scalar(fit_bits(!x, *ty), *ty)),
                }
            }
            Rvalue::Conv { to, regime, v, fault } => {
                let x = eval_operand(v, frame).v;
                Ok(Value::scalar(convert(x, *to, *regime, fault.as_ref())?, *to))
            }
            Rvalue::Call { func, args } => {
                let vals: Vec<Value> = args.iter().map(|a| eval_operand(a, frame)).collect();
                self.call(func, &vals)
            }
        }
    }

}

fn eval_operand(op: &Operand, frame: &[Value]) -> Value {
    match op {
        Operand::Const(v, ty) => Value::scalar(*v, *ty),
        Operand::Local(id) => frame[*id],
    }
}

fn default_value(ty: &Type) -> Value {
    match ty {
        Type::Scalar(s) => Value::scalar(0, *s),
        _ => Value::unit(),
    }
}

fn fault_of(edge: &FaultEdge) -> Fault {
    let msg = match edge.kind {
        FaultKind::Assert => "assertion failed",
        FaultKind::Panic => "panic",
        FaultKind::Overflow => "arithmetic overflow",
        FaultKind::ConvLoss => "conversion loses value",
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

/// Fit under the arithmetic regime; a checked out-of-range value delivers the op's
/// overflow edge.
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

/// Reduce to the bit pattern of `sty` (width-exact bitwise/shift, design 0006 §2.4).
fn fit_bits(value: i128, sty: ScalarTy) -> i128 {
    let (_, _, bits, signed) = ty_range(sty);
    let m = 1i128 << bits;
    let mut x = value.rem_euclid(m);
    if signed && x >= (m >> 1) {
        x -= m;
    }
    x
}
