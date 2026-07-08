//! AST → MIR lowering (design 0010 §2, §5). Lowers the *checked, resolved,
//! monomorphized* AST into the checked MIR, carrying the analysis tier's facts.
//!
//! Stage-A subset: non-generic functions over scalars/booleans — arithmetic (all
//! three regimes and every fault kind: overflow, div-by-zero, conv-loss),
//! comparisons, short-circuit `&&`/`||`, `if`/`while`/`loop`, `return`/`break`/
//! `continue`, `assert`/`panic`, `requires`/`ensures` contracts, `trace`
//! observables, and direct calls to user functions with value (`take`) scalar
//! parameters. Anything outside this subset lowers to [`LowerError::Unsupported`],
//! which the gate harness records as "out-of-subset" (not a failure) — the honest
//! coverage boundary.
//!
//! Fault spans are matched to the oracle by *simulating the tree-walker's
//! `cur_span` threading at lowering time* and baking the exact `(k, s)` into each
//! fault edge (see `cur_span` mutations below); the gate proves the match.

use std::collections::HashMap;

use crate::ast::{
    BinOp, Block, Expr, ExprKind, FnDecl, Item, ParamMode, Program, Stmt, StmtKind, Ty, TyKind,
    UnOp,
};
use crate::interp::FaultKind;
use crate::resolve::Items;
use crate::span::Span;
use crate::token::ScalarTy;
use crate::types::{needs_drop, Type};

use super::{
    check_invariants, BasicBlock, BlockId, FaultEdge, LocalDecl, LocalId, MirFn, MirProgram,
    Operand, Predicate, Regime, ReplayPolicy, Rvalue, Statement, StatementKind, Terminator,
};

/// A construct outside the Stage-A MIR subset. The gate treats it as out-of-subset
/// coverage, not a gate failure.
#[derive(Clone, Debug)]
pub struct LowerError(pub String);

fn unsupported<T>(what: impl Into<String>) -> Result<T, LowerError> {
    Err(LowerError(what.into()))
}

/// Lower a checked, resolved program to MIR. Generic programs must be
/// monomorphized first; a still-generic item is reported as unsupported here.
pub fn lower_checked(program: &Program, items: &Items) -> Result<MirProgram, LowerError> {
    let mut fns = Vec::new();
    let mut fn_index = HashMap::new();
    for it in &program.items {
        if let Item::Fn(fnd) = it {
            if !fnd.type_params.is_empty() || !fnd.regions.is_empty() {
                return unsupported(format!("generic/region fn `{}`", fnd.name));
            }
            let mut lw = Lowerer::new(items);
            let mf = lw.lower_fn(fnd)?;
            debug_assert_eq!(mf.name, fnd.name);
            check_invariants(&mf);
            fn_index.insert(mf.name.clone(), fns.len());
            fns.push(mf);
        }
    }
    Ok(MirProgram { fns, fn_index })
}

type LR<T> = Result<T, LowerError>;

struct Scope {
    locals: Vec<(String, LocalId, Type)>,
}

struct Loop {
    continue_bb: BlockId,
    break_bb: BlockId,
}

struct Lowerer<'a> {
    items: &'a Items,
    locals: Vec<LocalDecl>,
    blocks: Vec<BasicBlock>,
    cur: BlockId,
    reachable: bool,
    scopes: Vec<Scope>,
    loops: Vec<Loop>,
    regime: Regime,
    ret_ty: Type,
    result_local: Option<LocalId>,
    /// Mirror of the oracle's `cur_span`, threaded through lowering so fault edges
    /// carry the exact span the tree-walker delivers.
    cur_span: Span,
}

impl<'a> Lowerer<'a> {
    fn new(items: &'a Items) -> Lowerer<'a> {
        Lowerer {
            items,
            locals: Vec::new(),
            blocks: Vec::new(),
            cur: 0,
            reachable: true,
            scopes: Vec::new(),
            loops: Vec::new(),
            regime: Regime::Checked,
            ret_ty: Type::unit(),
            result_local: None,
            cur_span: Span::point(0),
        }
    }

    // ---- construction helpers ----
    fn new_local(&mut self, ty: Type, name: Option<String>) -> LocalId {
        let drop_obligation = needs_drop(&ty, self.items);
        let id = self.locals.len();
        self.locals.push(LocalDecl { ty, name, drop_obligation });
        id
    }
    fn new_block(&mut self) -> BlockId {
        let id = self.blocks.len();
        self.blocks.push(BasicBlock {
            stmts: Vec::new(),
            term: Terminator::Return,
        });
        id
    }
    fn emit(&mut self, kind: StatementKind, span: Span, observable: bool) {
        if !self.reachable {
            return;
        }
        self.blocks[self.cur].stmts.push(Statement { kind, span, observable });
    }
    /// Terminate the current block; the region becomes unreachable until a caller
    /// switches to a fresh labeled block.
    fn terminate(&mut self, term: Terminator) {
        if !self.reachable {
            return;
        }
        self.blocks[self.cur].term = term;
        self.reachable = false;
    }
    fn switch_to(&mut self, bb: BlockId) {
        self.cur = bb;
        self.reachable = true;
    }
    /// Assign an rvalue into a fresh temp local of the given type; return the temp.
    fn emit_temp(&mut self, ty: Type, rv: Rvalue, span: Span) -> LocalId {
        let id = self.new_local(ty, None);
        self.emit(StatementKind::Assign(id, rv), span, false);
        id
    }

    // ---- scopes / env ----
    fn push_scope(&mut self) {
        self.scopes.push(Scope { locals: Vec::new() });
    }
    fn pop_scope_with_drops(&mut self) {
        let sc = self.scopes.pop().unwrap();
        // INV-DROP: drop the scope's owned locals in reverse declaration order.
        for (_, id, _ty) in sc.locals.iter().rev() {
            if self.locals[*id].drop_obligation {
                self.emit(StatementKind::Drop(*id), Span::point(0), false);
            }
        }
    }
    fn bind(&mut self, name: &str, id: LocalId, ty: Type) {
        self.scopes
            .last_mut()
            .unwrap()
            .locals
            .push((name.to_string(), id, ty));
    }
    fn lookup(&self, name: &str) -> Option<(LocalId, Type)> {
        for sc in self.scopes.iter().rev() {
            for (n, id, ty) in sc.locals.iter().rev() {
                if n == name {
                    return Some((*id, ty.clone()));
                }
            }
        }
        None
    }

    // ---- type resolution (scalar subset) ----
    fn resolve_ty(&self, ty: &Ty) -> LR<Type> {
        match &ty.kind {
            TyKind::Scalar(s) => Ok(Type::Scalar(*s)),
            other => unsupported(format!("type {other:?}")),
        }
    }

    fn concretize(&self, ty: &Type) -> ScalarTy {
        match ty {
            Type::Scalar(s) if s.is_integer() => *s,
            Type::Scalar(ScalarTy::Bool) => ScalarTy::Bool,
            _ => ScalarTy::I64,
        }
    }
    fn int_type(&self, suffix: &Option<ScalarTy>, expected: Option<&Type>) -> ScalarTy {
        if let Some(s) = suffix {
            return *s;
        }
        if let Some(Type::Scalar(s)) = expected {
            if s.is_integer() {
                return *s;
            }
        }
        ScalarTy::I64
    }

    // ---- function lowering ----
    fn lower_fn(&mut self, fnd: &FnDecl) -> LR<MirFn> {
        let sig = self
            .items
            .fns
            .get(fnd.name.as_str())
            .ok_or_else(|| LowerError(format!("no sig for `{}`", fnd.name)))?;
        self.ret_ty = sig.ret.clone();
        // _0 = return place.
        let _ret = self.new_local(sig.ret.clone(), None);
        // Parameters _1..=n (value/`take` scalars only).
        let mut params = Vec::new();
        for p in &sig.params {
            if p.mode != ParamMode::Take {
                return unsupported(format!("param mode {:?}", p.mode));
            }
            if !is_scalar(&p.lowered) {
                return unsupported("non-scalar parameter");
            }
            let id = self.new_local(p.lowered.clone(), Some(p.name.clone()));
            params.push((p.name.clone(), id, p.lowered.clone()));
        }
        // A `result` local for `ensures`.
        if !fnd.ensures.is_empty() {
            if !is_scalar(&sig.ret) {
                return unsupported("non-scalar `ensures` result");
            }
            self.result_local = Some(self.new_local(sig.ret.clone(), Some("result".to_string())));
        }

        // Contract predicates are lowered into their own blocks over the same
        // frame (params + result are in scope).
        self.push_scope();
        for (n, id, ty) in &params {
            self.bind(n, *id, ty.clone());
        }
        let requires = {
            let mut ps = Vec::new();
            for r in &fnd.requires {
                ps.push(self.lower_predicate(r, FaultKind::Requires)?);
            }
            ps
        };

        // Body entry block.
        let entry = self.new_block();
        self.switch_to(entry);
        self.lower_block(&fnd.body)?;
        // Fall-off: return the (default) return place.
        if self.reachable {
            self.terminate(Terminator::Return);
        }

        // `ensures` predicates read `result`; bound by the interpreter at exit.
        let ensures = {
            let mut ps = Vec::new();
            for e in &fnd.ensures {
                if let Some(rl) = self.result_local {
                    self.bind("result", rl, self.ret_ty.clone());
                }
                ps.push(self.lower_predicate(e, FaultKind::Ensures)?);
            }
            ps
        };
        self.pop_scope_with_drops();

        Ok(MirFn {
            name: fnd.name.clone(),
            num_params: params.len(),
            result_local: self.result_local,
            locals: std::mem::take(&mut self.locals),
            blocks: std::mem::take(&mut self.blocks),
            entry,
            requires,
            ensures,
            replay: ReplayPolicy::Precise,
        })
    }

    fn lower_predicate(&mut self, cond: &Expr, kind: FaultKind) -> LR<Predicate> {
        let bb = self.new_block();
        self.switch_to(bb);
        let (v, _) = self.lower_value(cond, Some(&Type::bool()))?;
        let span = self.cur_span; // oracle's cur_span at the contract fault
        let value = self.emit_temp(Type::bool(), Rvalue::Use(v), span);
        self.terminate(Terminator::Return);
        Ok(Predicate { entry: bb, value, span, kind })
    }

    // ---- blocks / statements ----
    fn lower_block(&mut self, b: &Block) -> LR<()> {
        self.push_scope();
        for s in &b.stmts {
            if !self.reachable {
                break;
            }
            self.lower_stmt(s)?;
        }
        self.pop_scope_with_drops();
        Ok(())
    }

    fn lower_stmt(&mut self, s: &Stmt) -> LR<()> {
        match &s.kind {
            StmtKind::Let { name, ty, init, .. } => {
                let decl = match ty {
                    Some(t) => Some(self.resolve_ty(t)?),
                    None => None,
                };
                match init {
                    Some(e) => {
                        let (op, oty) = self.lower_value(e, decl.as_ref())?;
                        let lty = decl.unwrap_or(oty);
                        if !is_scalar(&lty) {
                            return unsupported("non-scalar local");
                        }
                        let id = self.new_local(lty.clone(), Some(name.clone()));
                        self.emit(StatementKind::Assign(id, Rvalue::Use(op)), s.span, false);
                        self.bind(name, id, lty);
                    }
                    None => {
                        let lty = decl.ok_or_else(|| LowerError("untyped uninit let".into()))?;
                        if !is_scalar(&lty) {
                            return unsupported("non-scalar local");
                        }
                        let id = self.new_local(lty.clone(), Some(name.clone()));
                        self.bind(name, id, lty);
                    }
                }
                Ok(())
            }
            StmtKind::Assign { target, value } => {
                let (id, tty) = match &target.kind {
                    ExprKind::Ident(n) => self
                        .lookup(n)
                        .ok_or_else(|| LowerError(format!("assign to unknown `{n}`")))?,
                    _ => return unsupported("assign to non-local place"),
                };
                let (op, _) = self.lower_value(value, Some(&tty))?;
                self.emit(StatementKind::Assign(id, Rvalue::Use(op)), s.span, false);
                Ok(())
            }
            StmtKind::Expr(e) => {
                self.lower_value(e, None)?;
                Ok(())
            }
        }
    }

    // ---- expressions ----
    /// Lower an expression to an operand and its type, emitting statements. Threads
    /// `cur_span` exactly as the oracle's `eval_value`/`eval_place`.
    fn lower_value(&mut self, e: &Expr, expected: Option<&Type>) -> LR<(Operand, Type)> {
        self.cur_span = e.span;
        match &e.kind {
            ExprKind::Paren(i) => self.lower_value(i, expected),
            ExprKind::IntLit { value, suffix } => {
                let sty = self.int_type(suffix, expected);
                Ok((Operand::Const(*value as i128, sty), Type::Scalar(sty)))
            }
            ExprKind::NegIntLit { value, suffix } => {
                let sty = self.int_type(suffix, expected);
                Ok((Operand::Const(-(*value as i128), sty), Type::Scalar(sty)))
            }
            ExprKind::BoolLit(b) => Ok((Operand::Const(*b as i128, ScalarTy::Bool), Type::bool())),
            ExprKind::Ident(name) => {
                let (id, ty) = self
                    .lookup(name)
                    .ok_or_else(|| LowerError(format!("unknown name `{name}`")))?;
                Ok((Operand::Local(id), ty))
            }
            ExprKind::Unary { op, expr } => self.lower_unary(*op, expr, expected, e.span),
            ExprKind::Binary { op, lhs, rhs } => self.lower_binary(*op, lhs, rhs, expected, e.span),
            ExprKind::Conv { ty, expr } => self.lower_conv(ty, expr),
            ExprKind::Call { callee, args } => self.lower_call(callee, args, e.span),
            ExprKind::Block(b) => {
                self.lower_block(b)?;
                Ok(self.unit())
            }
            ExprKind::If { cond, then_blk, else_blk } => {
                self.lower_if(cond, then_blk, else_blk.as_deref())?;
                Ok(self.unit())
            }
            ExprKind::While { cond, body } => {
                self.lower_while(cond, body)?;
                Ok(self.unit())
            }
            ExprKind::Loop(b) => {
                self.lower_loop(b)?;
                Ok(self.unit())
            }
            ExprKind::Wrapping(b) => self.lower_regime(b, Regime::Wrapping),
            ExprKind::Saturating(b) => self.lower_regime(b, Regime::Saturating),
            ExprKind::Unsafe { body, .. } => {
                self.lower_block(body)?;
                Ok(self.unit())
            }
            ExprKind::Return(opt) => {
                self.lower_return(opt.as_deref())?;
                Ok(self.unit())
            }
            ExprKind::Break => {
                let bb = self
                    .loops
                    .last()
                    .ok_or_else(|| LowerError("break outside loop".into()))?
                    .break_bb;
                self.terminate(Terminator::Goto(bb));
                Ok(self.unit())
            }
            ExprKind::Continue => {
                let bb = self
                    .loops
                    .last()
                    .ok_or_else(|| LowerError("continue outside loop".into()))?
                    .continue_bb;
                self.terminate(Terminator::Goto(bb));
                Ok(self.unit())
            }
            ExprKind::Assert(c) => {
                let (cond, _) = self.lower_value(c, Some(&Type::bool()))?;
                let span = self.cur_span;
                let ok_bb = self.new_block();
                let fail_bb = self.new_block();
                self.terminate(Terminator::Branch { cond, then_bb: ok_bb, else_bb: fail_bb });
                self.switch_to(fail_bb);
                self.terminate(Terminator::Fault(FaultEdge { kind: FaultKind::Assert, span }));
                self.switch_to(ok_bb);
                Ok(self.unit())
            }
            ExprKind::Panic(_) => {
                self.terminate(Terminator::Fault(FaultEdge { kind: FaultKind::Panic, span: e.span }));
                Ok(self.unit())
            }
            ExprKind::Result => {
                let rl = self
                    .result_local
                    .ok_or_else(|| LowerError("`result` outside ensures".into()))?;
                Ok((Operand::Local(rl), self.ret_ty.clone()))
            }
            other => unsupported(format!("expr {}", variant_name(other))),
        }
    }

    fn unit(&self) -> (Operand, Type) {
        (Operand::Const(0, ScalarTy::Unit), Type::unit())
    }

    fn lower_unary(&mut self, op: UnOp, expr: &Expr, expected: Option<&Type>, _span: Span) -> LR<(Operand, Type)> {
        match op {
            UnOp::Not => {
                let (v, _) = self.lower_value(expr, Some(&Type::bool()))?;
                let id = self.emit_temp(Type::bool(), Rvalue::Un { op, regime: self.regime, ty: ScalarTy::Bool, v, fault: None }, self.cur_span);
                Ok((Operand::Local(id), Type::bool()))
            }
            UnOp::Neg | UnOp::BitNot => {
                let (v, vty) = self.lower_value(expr, expected)?;
                let sty = self.concretize(&vty);
                // Neg can overflow (e.g. negating INT_MIN) in the checked regime;
                // BitNot never faults.
                let fault = if op == UnOp::Neg && self.regime == Regime::Checked {
                    Some(FaultEdge { kind: FaultKind::Overflow, span: self.cur_span })
                } else {
                    None
                };
                let id = self.emit_temp(Type::Scalar(sty), Rvalue::Un { op, regime: self.regime, ty: sty, v, fault }, self.cur_span);
                Ok((Operand::Local(id), Type::Scalar(sty)))
            }
        }
    }

    fn lower_binary(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr, expected: Option<&Type>, span: Span) -> LR<(Operand, Type)> {
        use BinOp::*;
        match op {
            And | Or => self.lower_short_circuit(op, lhs, rhs),
            Eq | Ne | Lt | Le | Gt | Ge => {
                let (l, lty) = self.lower_value(lhs, None)?;
                let ot = self.concretize(&lty);
                let (r, _) = self.lower_value(rhs, Some(&Type::Scalar(ot)))?;
                let id = self.emit_temp(Type::bool(), Rvalue::Cmp { op, l, r }, self.cur_span);
                Ok((Operand::Local(id), Type::bool()))
            }
            Add | Sub | Mul | Div | Rem | BitAnd | BitOr | BitXor | Shl | Shr => {
                let opty = expected.filter(|t| t.is_integer()).cloned();
                let (l, lty) = self.lower_value(lhs, opty.as_ref())?;
                let sty = match &opty {
                    Some(Type::Scalar(s)) => *s,
                    _ => self.concretize(&lty),
                };
                let rexp = if matches!(op, Shl | Shr) { None } else { Some(Type::Scalar(sty)) };
                let (r, _) = self.lower_value(rhs, rexp.as_ref())?;
                // Mirror the oracle: arithmetic and shift ops reset cur_span to the
                // whole-op span before the fault check; bitwise ops do not.
                let fallible = matches!(op, Add | Sub | Mul | Div | Rem | Shl | Shr);
                if fallible {
                    self.cur_span = span;
                }
                let fault = if fallible && self.regime == Regime::Checked {
                    Some(FaultEdge { kind: FaultKind::Overflow, span })
                } else {
                    None
                };
                let id = self.emit_temp(
                    Type::Scalar(sty),
                    Rvalue::Bin { op, regime: self.regime, ty: sty, l, r, span, fault },
                    span,
                );
                Ok((Operand::Local(id), Type::Scalar(sty)))
            }
        }
    }

    fn lower_short_circuit(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr) -> LR<(Operand, Type)> {
        let result = self.new_local(Type::bool(), None);
        let (l, _) = self.lower_value(lhs, Some(&Type::bool()))?;
        let rhs_bb = self.new_block();
        let short_bb = self.new_block();
        let join = self.new_block();
        // `&&`: if lhs then eval rhs else result=false. `||`: if lhs then
        // result=true else eval rhs.
        let (then_bb, else_bb) = if op == BinOp::And { (rhs_bb, short_bb) } else { (short_bb, rhs_bb) };
        self.terminate(Terminator::Branch { cond: l, then_bb, else_bb });
        self.switch_to(rhs_bb);
        let (r, _) = self.lower_value(rhs, Some(&Type::bool()))?;
        self.emit(StatementKind::Assign(result, Rvalue::Use(r)), self.cur_span, false);
        self.terminate(Terminator::Goto(join));
        self.switch_to(short_bb);
        let short_val = op == BinOp::Or;
        self.emit(StatementKind::Assign(result, Rvalue::Use(Operand::Const(short_val as i128, ScalarTy::Bool))), self.cur_span, false);
        self.terminate(Terminator::Goto(join));
        self.switch_to(join);
        Ok((Operand::Local(result), Type::bool()))
    }

    fn lower_conv(&mut self, ty: &Ty, expr: &Expr) -> LR<(Operand, Type)> {
        let (v, _) = self.lower_value(expr, None)?;
        let to = match self.resolve_ty(ty)? {
            Type::Scalar(s) => s,
            _ => return unsupported("non-scalar conversion target"),
        };
        // Oracle delivers conv-loss at the operand's trailing span (== cur_span).
        let fault = if self.regime == Regime::Checked {
            Some(FaultEdge { kind: FaultKind::ConvLoss, span: self.cur_span })
        } else {
            None
        };
        let id = self.emit_temp(Type::Scalar(to), Rvalue::Conv { to, regime: self.regime, v, fault }, self.cur_span);
        Ok((Operand::Local(id), Type::Scalar(to)))
    }

    fn lower_call(&mut self, callee: &Expr, args: &[Expr], span: Span) -> LR<(Operand, Type)> {
        let name = match &callee.kind {
            ExprKind::Ident(n) => n.clone(),
            _ => return unsupported("indirect/method call"),
        };
        if name == "trace" {
            let (v, _) = self.lower_value(&args[0], Some(&Type::Scalar(ScalarTy::I64)))?;
            self.emit(StatementKind::Trace(v), span, true);
            return Ok(self.unit());
        }
        let sig = match self.items.fns.get(name.as_str()) {
            Some(s) => s.clone(),
            None => return unsupported(format!("call to `{name}`")),
        };
        if !is_scalar(&sig.ret) && !matches!(sig.ret, Type::Scalar(ScalarTy::Unit)) {
            return unsupported("non-scalar return");
        }
        let mut ops = Vec::new();
        for (p, a) in sig.params.iter().zip(args) {
            if p.mode != ParamMode::Take || !is_scalar(&p.lowered) {
                return unsupported("non-value/non-scalar argument");
            }
            let (op, _) = self.lower_value(a, Some(&p.lowered))?;
            ops.push(op);
        }
        if sig.params.len() != args.len() {
            return unsupported("arity mismatch");
        }
        let ret = sig.ret.clone();
        let id = self.emit_temp(ret.clone(), Rvalue::Call { func: name, args: ops }, span);
        Ok((Operand::Local(id), ret))
    }

    fn lower_if(&mut self, cond: &Expr, then_blk: &Block, else_blk: Option<&Expr>) -> LR<()> {
        let (c, _) = self.lower_value(cond, Some(&Type::bool()))?;
        let then_bb = self.new_block();
        let else_bb = self.new_block();
        let join = self.new_block();
        self.terminate(Terminator::Branch { cond: c, then_bb, else_bb });
        self.switch_to(then_bb);
        self.lower_block(then_blk)?;
        if self.reachable {
            self.terminate(Terminator::Goto(join));
        }
        self.switch_to(else_bb);
        if let Some(e) = else_blk {
            self.lower_value(e, None)?;
        }
        if self.reachable {
            self.terminate(Terminator::Goto(join));
        }
        self.switch_to(join);
        Ok(())
    }

    fn lower_while(&mut self, cond: &Expr, body: &Block) -> LR<()> {
        let head = self.new_block();
        let body_bb = self.new_block();
        let exit = self.new_block();
        self.terminate(Terminator::Goto(head));
        self.switch_to(head);
        let (c, _) = self.lower_value(cond, Some(&Type::bool()))?;
        self.terminate(Terminator::Branch { cond: c, then_bb: body_bb, else_bb: exit });
        self.switch_to(body_bb);
        self.loops.push(Loop { continue_bb: head, break_bb: exit });
        self.lower_block(body)?;
        self.loops.pop();
        if self.reachable {
            self.terminate(Terminator::Goto(head));
        }
        self.switch_to(exit);
        Ok(())
    }

    fn lower_loop(&mut self, body: &Block) -> LR<()> {
        let body_bb = self.new_block();
        let exit = self.new_block();
        self.terminate(Terminator::Goto(body_bb));
        self.switch_to(body_bb);
        self.loops.push(Loop { continue_bb: body_bb, break_bb: exit });
        self.lower_block(body)?;
        self.loops.pop();
        if self.reachable {
            self.terminate(Terminator::Goto(body_bb));
        }
        self.switch_to(exit);
        Ok(())
    }

    fn lower_regime(&mut self, b: &Block, regime: Regime) -> LR<(Operand, Type)> {
        let save = self.regime;
        self.regime = regime;
        let r = self.lower_block(b);
        self.regime = save;
        r?;
        Ok(self.unit())
    }

    fn lower_return(&mut self, opt: Option<&Expr>) -> LR<()> {
        if let Some(e) = opt {
            let ret_ty = self.ret_ty.clone();
            let (op, _) = self.lower_value(e, Some(&ret_ty))?;
            self.emit(StatementKind::Assign(0, Rvalue::Use(op)), self.cur_span, false);
        }
        self.terminate(Terminator::Return);
        Ok(())
    }
}

fn is_scalar(ty: &Type) -> bool {
    matches!(ty, Type::Scalar(s) if s.is_integer() || *s == ScalarTy::Bool)
}

fn variant_name(k: &ExprKind) -> &'static str {
    match k {
        ExprKind::Match { .. } => "match",
        ExprKind::StructLit { .. } => "struct-literal",
        ExprKind::EnumCtor { .. } => "enum-ctor",
        ExprKind::ArrayLit(_) => "array-literal",
        ExprKind::ArrayRepeat { .. } => "array-repeat",
        ExprKind::Index { .. } => "index",
        ExprKind::Field { .. } => "field",
        ExprKind::Prefix { .. } => "prefix",
        ExprKind::Try(_) => "try",
        ExprKind::StrLit(_) => "string-literal",
        _ => "unsupported-expr",
    }
}
