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
    BinOp, Block, Expr, ExprKind, FieldInit, FnDecl, Item, MatchArm, ParamMode, PatKind, PrefixOp,
    Program, Stmt, StmtKind, Ty, TyKind, UnOp,
};
use crate::interp::FaultKind;
use crate::resolve::Items;
use crate::span::Span;
use crate::token::ScalarTy;
use crate::interp::layout::Layout;
use crate::interp::mem::round_up;
use crate::types::{is_copy, needs_drop, ArrayLen, Type};

use super::{
    check_invariants, BasicBlock, BlockId, FaultEdge, LocalDecl, LocalId, MirFn, MirProgram,
    Operand, Place, Predicate, Proj, Regime, ReplayPolicy, Rvalue, Statement, StatementKind,
    Terminator,
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
    let mut consts = HashMap::new();
    for it in &program.items {
        if let Item::Static(st) = it {
            if let ExprKind::IntLit { value, .. } = &st.value.kind {
                consts.insert(st.name.clone(), *value);
            }
        }
    }
    let mut fns = Vec::new();
    let mut fn_index = HashMap::new();
    let mut drop_hooks = HashMap::new();
    // Lower each struct `drop` hook to an ordinary MIR function taking `write self`
    // (INV-DROP: the schedule is static; the hook is called at the drop point).
    for it in &program.items {
        if let Item::Struct(sd) = it {
            if let Some(block) = &sd.drop_hook {
                let hook_name = format!("<drop {}>", sd.name);
                let mut lw = Lowerer::new(items, &consts);
                let mf = lw.lower_hook(&hook_name, &sd.name, block)?;
                check_invariants(&mf);
                drop_hooks.insert(sd.name.clone(), hook_name.clone());
                fn_index.insert(hook_name, fns.len());
                fns.push(mf);
            }
        }
    }
    for it in &program.items {
        if let Item::Fn(fnd) = it {
            if !fnd.type_params.is_empty() {
                return unsupported(format!("generic fn `{}`", fnd.name));
            }
            let mut lw = Lowerer::new(items, &consts);
            let mf = lw.lower_fn(fnd)?;
            debug_assert_eq!(mf.name, fnd.name);
            check_invariants(&mf);
            fn_index.insert(mf.name.clone(), fns.len());
            fns.push(mf);
        }
    }
    Ok(MirProgram { fns, fn_index, drop_hooks })
}

type LR<T> = Result<T, LowerError>;

struct Scope {
    locals: Vec<(String, LocalId, Type)>,
}

struct Loop {
    continue_bb: BlockId,
    break_bb: BlockId,
    scope_depth: usize,
}

struct Lowerer<'a> {
    items: &'a Items,
    consts: &'a HashMap<String, u64>,
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
    fn new(items: &'a Items, consts: &'a HashMap<String, u64>) -> Lowerer<'a> {
        Lowerer {
            items,
            consts,
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
    /// Emit the drop schedule for *every* live scope, innermost-first — the drops
    /// that run when control leaves the function via `return` (INV-DROP). The
    /// caller terminates immediately after, so the natural scope-exit drops below
    /// become unreachable (no double-drop).
    fn emit_return_drops(&mut self) {
        let ids: Vec<LocalId> = self
            .scopes
            .iter()
            .rev()
            .flat_map(|sc| sc.locals.iter().rev().map(|(_, id, _)| *id))
            .collect();
        for id in ids {
            if self.locals[id].drop_obligation {
                self.emit(StatementKind::Drop(id), Span::point(0), false);
            }
        }
    }

    /// Emit the drop schedule for the scopes opened *inside* the innermost loop
    /// body (down to `keep` remaining scopes) — the drops that run on `break`/
    /// `continue` (INV-DROP).
    fn emit_loop_exit_drops(&mut self, keep: usize) {
        let ids: Vec<LocalId> = self
            .scopes
            .iter()
            .skip(keep)
            .rev()
            .flat_map(|sc| sc.locals.iter().rev().map(|(_, id, _)| *id))
            .collect();
        for id in ids {
            if self.locals[id].drop_obligation {
                self.emit(StatementKind::Drop(id), Span::point(0), false);
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

    // ---- type resolution + layout (shared substrate) ----
    fn lay(&self) -> Layout<'_> {
        Layout { items: self.items, consts: self.consts }
    }
    fn size_of(&self, ty: &Type) -> u64 {
        self.lay().size_of(ty)
    }
    fn align_of(&self, ty: &Type) -> u64 {
        self.lay().align_of(ty)
    }
    fn stride_of(&self, elem: &Type) -> u64 {
        round_up(self.size_of(elem), self.align_of(elem))
    }

    fn resolve_ty(&self, ty: &Ty) -> LR<Type> {
        Ok(match &ty.kind {
            TyKind::Scalar(s) => Type::Scalar(*s),
            TyKind::Named(n) => Type::Named(n.clone()),
            TyKind::Array { size, elem } => {
                let len = match &size.kind {
                    ExprKind::IntLit { value, .. } => ArrayLen::Lit(*value),
                    ExprKind::Ident(n) => ArrayLen::Named(n.clone()),
                    _ => ArrayLen::Unknown,
                };
                Type::Array(Box::new(self.resolve_ty(elem)?), len)
            }
            TyKind::Borrow(e) => Type::Borrow(Box::new(self.resolve_ty(e)?)),
            TyKind::BorrowMut(e) => Type::BorrowMut(Box::new(self.resolve_ty(e)?)),
            TyKind::RawPtr(e) => Type::RawPtr(Box::new(self.resolve_ty(e)?)),
            TyKind::Slice(e) => Type::Slice(Box::new(self.resolve_ty(e)?)),
            TyKind::SliceMut(e) => Type::SliceMut(Box::new(self.resolve_ty(e)?)),
            TyKind::Box(e) => Type::Box(Box::new(self.resolve_ty(e)?)),
            TyKind::BoxResult(e) => Type::BoxResult(Box::new(self.resolve_ty(e)?)),
            other => return unsupported(format!("type {other:?}")),
        })
    }

    /// Is `ty` a scalar/pointer-width value (fits in a local read as an int)?
    fn is_wordy(&self, ty: &Type) -> bool {
        matches!(
            ty,
            Type::Scalar(_)
                | Type::Borrow(_)
                | Type::BorrowMut(_)
                | Type::RawPtr(_)
                | Type::FnPtr(_)
        )
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
            match p.mode {
                ParamMode::Read | ParamMode::Write => {
                    // A `read`/`write` param is a `Borrow`/`BorrowMut` — an 8-byte
                    // address (word-sized).
                    if !self.is_wordy(&p.lowered) {
                        return unsupported("non-word borrow parameter");
                    }
                }
                ParamMode::Take => {
                    // By-value aggregate params are in subset when drop-inert; a
                    // needs-drop by-value aggregate would need an owned-param drop
                    // schedule we don't yet emit.
                    if !self.is_wordy(&p.lowered) && needs_drop(&p.lowered, self.items) {
                        return unsupported("needs-drop by-value aggregate parameter");
                    }
                }
                _ => return unsupported(format!("param mode {:?}", p.mode)),
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

    /// Lower a struct `drop` hook block to a MIR function `<drop Name>(write self)`.
    fn lower_hook(&mut self, hook_name: &str, struct_name: &str, block: &Block) -> LR<MirFn> {
        self.ret_ty = Type::unit();
        let _ret = self.new_local(Type::unit(), None); // _0
        let self_ty = Type::BorrowMut(Box::new(Type::Named(struct_name.to_string())));
        let sid = self.new_local(self_ty.clone(), Some("self".to_string()));
        self.push_scope();
        self.bind("self", sid, self_ty);
        let entry = self.new_block();
        self.switch_to(entry);
        self.lower_block(block)?;
        if self.reachable {
            self.terminate(Terminator::Return);
        }
        self.pop_scope_with_drops(); // `self` is a borrow, not owned — nothing drops
        Ok(MirFn {
            name: hook_name.to_string(),
            num_params: 1,
            result_local: None,
            locals: std::mem::take(&mut self.locals),
            blocks: std::mem::take(&mut self.blocks),
            entry,
            requires: Vec::new(),
            ensures: Vec::new(),
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
                    Some(e) => match &decl {
                        Some(lty) => {
                            let lty = lty.clone();
                            let id = self.new_local(lty.clone(), Some(name.clone()));
                            self.lower_into(e, &Place::local(id), &lty)?;
                            self.bind(name, id, lty);
                        }
                        None => {
                            // No annotation: only the wordy/scalar inference path.
                            let (op, oty) = self.lower_value(e, None)?;
                            if !self.is_wordy(&oty) {
                                return unsupported("non-word inferred local");
                            }
                            let id = self.new_local(oty.clone(), Some(name.clone()));
                            self.emit(StatementKind::Assign(id, Rvalue::Use(op)), s.span, false);
                            self.bind(name, id, oty);
                        }
                    },
                    None => {
                        let lty = decl.ok_or_else(|| LowerError("untyped uninit let".into()))?;
                        let id = self.new_local(lty.clone(), Some(name.clone()));
                        self.bind(name, id, lty);
                    }
                }
                Ok(())
            }
            StmtKind::Assign { target, value } => {
                let (place, tty) = self.lower_place(target)?;
                self.lower_into(value, &place, &tty)?;
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
                let (bb, depth) = self
                    .loops
                    .last()
                    .map(|l| (l.break_bb, l.scope_depth))
                    .ok_or_else(|| LowerError("break outside loop".into()))?;
                self.emit_loop_exit_drops(depth);
                self.terminate(Terminator::Goto(bb));
                Ok(self.unit())
            }
            ExprKind::Continue => {
                let (bb, depth) = self
                    .loops
                    .last()
                    .map(|l| (l.continue_bb, l.scope_depth))
                    .ok_or_else(|| LowerError("continue outside loop".into()))?;
                self.emit_loop_exit_drops(depth);
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
            ExprKind::Try(inner) => self.lower_try(inner, e.span),
            ExprKind::Match { scrutinee, arms } => {
                self.lower_match(scrutinee, arms, None)?;
                Ok(self.unit())
            }
            ExprKind::Field { .. }
            | ExprKind::Index { .. }
            | ExprKind::Prefix { op: PrefixOp::Deref, .. } => {
                let (place, ty) = self.lower_place(e)?;
                if self.is_wordy(&ty) {
                    let id = self.emit_temp(ty.clone(), Rvalue::Load { place, ty: ty.clone() }, self.cur_span);
                    Ok((Operand::Local(id), ty))
                } else {
                    unsupported("aggregate in value position")
                }
            }
            ExprKind::Prefix { op: PrefixOp::Read, expr } => self.lower_borrow(expr, false),
            ExprKind::Prefix { op: PrefixOp::Write, expr } => self.lower_borrow(expr, true),
            other => unsupported(format!("expr {}", variant_name(other))),
        }
    }

    // ---- places & aggregate stores (Stage A2 memory substrate) ----

    /// Lower a place expression to a MIR [`Place`] and its type. Auto-peels
    /// borrow/box layers exactly as the oracle's `peel_place`.
    fn lower_place(&mut self, e: &Expr) -> LR<(Place, Type)> {
        self.cur_span = e.span;
        match &e.kind {
            ExprKind::Paren(i) => self.lower_place(i),
            ExprKind::Ident(name) => {
                let (id, ty) = self
                    .lookup(name)
                    .ok_or_else(|| LowerError(format!("place of unknown `{name}`")))?;
                Ok((Place::local(id), ty))
            }
            ExprKind::Prefix { op: PrefixOp::Deref, expr } => {
                let (mut pl, t) = self.lower_place(expr)?;
                let inner = match &t {
                    Type::Borrow(x) | Type::BorrowMut(x) | Type::Box(x) | Type::RawPtr(x) => (**x).clone(),
                    _ => return unsupported("deref of non-pointer"),
                };
                pl.proj.push(Proj::Deref { inner: inner.clone() });
                Ok((pl, inner))
            }
            ExprKind::Field { base, field } => {
                let (mut pl, t) = self.lower_place(base)?;
                let t = self.peel(&mut pl, t);
                match &t {
                    Type::Named(n) => {
                        let (fty, off) = self
                            .lay()
                            .field_offset(n, field)
                            .ok_or_else(|| LowerError(format!("no field `{field}` on `{n}`")))?;
                        pl.proj.push(Proj::Field { offset: off, ty: fty.clone() });
                        Ok((pl, fty))
                    }
                    _ => unsupported("field of non-struct"),
                }
            }
            ExprKind::Index { base, index } => {
                let (idx, _) = self.lower_value(index, Some(&Type::usize()))?;
                let (mut pl, t) = self.lower_place(base)?;
                let t = self.peel(&mut pl, t);
                match &t {
                    Type::Array(elem, len) => {
                        let n = self.lay().array_len(len);
                        let stride = self.stride_of(elem);
                        pl.proj.push(Proj::Index { index: idx, stride, len: n, span: self.cur_span });
                        Ok((pl, (**elem).clone()))
                    }
                    _ => unsupported("index of non-array"),
                }
            }
            _ => unsupported("unsupported place"),
        }
    }

    /// Auto-peel borrow/box layers on a place (the field/index base auto-deref).
    fn peel(&self, pl: &mut Place, mut ty: Type) -> Type {
        loop {
            match ty {
                Type::Borrow(x) | Type::BorrowMut(x) | Type::Box(x) => {
                    pl.proj.push(Proj::Deref { inner: (*x).clone() });
                    ty = *x;
                }
                other => return other,
            }
        }
    }

    fn lower_borrow(&mut self, expr: &Expr, excl: bool) -> LR<(Operand, Type)> {
        let (place, inner) = self.lower_place(expr)?;
        let ty = if excl {
            Type::BorrowMut(Box::new(inner))
        } else {
            Type::Borrow(Box::new(inner))
        };
        let id = self.emit_temp(ty.clone(), Rvalue::Ref(place), self.cur_span);
        Ok((Operand::Local(id), ty))
    }

    /// Get a place for an aggregate argument: a real place if `e` names one, else
    /// materialize the value into a fresh temp and return that temp's place.
    fn materialize_place(&mut self, e: &Expr, ty: &Type) -> LR<Place> {
        match &e.kind {
            ExprKind::Ident(_)
            | ExprKind::Field { .. }
            | ExprKind::Index { .. }
            | ExprKind::Paren(_)
            | ExprKind::Prefix { op: PrefixOp::Deref, .. } => Ok(self.lower_place(e)?.0),
            _ => {
                let tmp = self.new_local(ty.clone(), None);
                self.lower_into(e, &Place::local(tmp), ty)?;
                Ok(Place::local(tmp))
            }
        }
    }

    /// A match scrutinee is a place (matched in place) or a value expression such
    /// as a call — the latter is materialized into a temp first.
    fn lower_scrutinee(&mut self, e: &Expr) -> LR<(Place, Type)> {
        match &e.kind {
            ExprKind::Paren(i) => self.lower_scrutinee(i),
            ExprKind::Ident(_)
            | ExprKind::Field { .. }
            | ExprKind::Index { .. }
            | ExprKind::Prefix { op: PrefixOp::Deref, .. } => self.lower_place(e),
            ExprKind::Call { callee, .. } => {
                let name = match &callee.kind {
                    ExprKind::Ident(n) => n.clone(),
                    _ => return unsupported("match on an indirect call"),
                };
                let ret = self
                    .items
                    .fns
                    .get(name.as_str())
                    .map(|s| s.ret.clone())
                    .ok_or_else(|| LowerError(format!("match on unknown call `{name}`")))?;
                let place = self.materialize_place(e, &ret)?;
                Ok((place, ret))
            }
            ExprKind::EnumCtor { .. } => {
                // A freshly-constructed enum: materialize into a temp of its type.
                // Its type is inferred from the ctor's expected type — unavailable
                // here — so keep it out of subset (rare in practice).
                unsupported("match on a freshly-constructed enum value")
            }
            _ => unsupported("unsupported match scrutinee"),
        }
    }

    /// Lower a `match` to switch-on-tag + payload binds (design 0010 §8.2.1). The
    /// scrutinee is read as a place; the tag selects an arm; copy-bindings copy
    /// their payload out. Scope is limited to drop-inert enums with copy payload
    /// binds (no scrutinee move/drop to reconcile) — anything else is out of
    /// subset (never a silent divergence).
    fn lower_match(
        &mut self,
        scrutinee: &Expr,
        arms: &[MatchArm],
        dst: Option<(&Place, &Type)>,
    ) -> LR<()> {
        let (mut splace, sty) = self.lower_scrutinee(scrutinee)?;
        let ety = self.peel(&mut splace, sty);
        if crate::types::needs_drop(&ety, self.items) {
            return unsupported("match on a needs-drop enum (scrutinee drop/move unimplemented)");
        }
        let einfo = self
            .lay()
            .enum_info(&ety)
            .ok_or_else(|| LowerError("match on non-enum".into()))?;
        // Read the tag once.
        let mut tagpl = splace.clone();
        tagpl.proj.push(Proj::Field { offset: 0, ty: Type::Scalar(ScalarTy::U64) });
        let tag = self.emit_temp(
            Type::Scalar(ScalarTy::U64),
            Rvalue::Load { place: tagpl, ty: Type::Scalar(ScalarTy::U64) },
            self.cur_span,
        );
        let join = self.new_block();
        let mut test_open = true;
        for arm in arms {
            if !test_open {
                break;
            }
            match &arm.pattern.kind {
                PatKind::Wildcard | PatKind::Binding(_) => {
                    // Default arm (exhaustive tail): unconditional.
                    self.lower_match_arm(arm, &splace, &einfo, None, dst)?;
                    if self.reachable {
                        self.terminate(Terminator::Goto(join));
                    }
                    test_open = false;
                }
                PatKind::Variant { variant, .. } => {
                    let idx = einfo
                        .iter()
                        .position(|(n, _)| n == variant)
                        .ok_or_else(|| LowerError(format!("no variant `{variant}`")))?;
                    let cmp = self.emit_temp(
                        Type::bool(),
                        Rvalue::Cmp {
                            op: BinOp::Eq,
                            l: Operand::Local(tag),
                            r: Operand::Const(idx as i128, ScalarTy::U64),
                        },
                        self.cur_span,
                    );
                    let arm_bb = self.new_block();
                    let next_bb = self.new_block();
                    self.terminate(Terminator::Branch { cond: Operand::Local(cmp), then_bb: arm_bb, else_bb: next_bb });
                    self.switch_to(arm_bb);
                    self.lower_match_arm(arm, &splace, &einfo, Some(idx), dst)?;
                    if self.reachable {
                        self.terminate(Terminator::Goto(join));
                    }
                    self.switch_to(next_bb);
                }
            }
        }
        if test_open {
            // Non-exhaustive fall-through — unreachable for a checked program, but
            // mirror the oracle's "no matching arm" panic if it is ever reached.
            self.terminate(Terminator::Fault(FaultEdge { kind: FaultKind::Panic, span: self.cur_span }));
        }
        self.switch_to(join);
        Ok(())
    }

    fn lower_match_arm(
        &mut self,
        arm: &MatchArm,
        splace: &Place,
        einfo: &[(String, Vec<Type>)],
        idx: Option<usize>,
        dst: Option<(&Place, &Type)>,
    ) -> LR<()> {
        self.push_scope();
        if let (PatKind::Variant { sub, .. }, Some(idx)) = (&arm.pattern.kind, idx) {
            let payloads = einfo[idx].1.clone();
            for (i, sp) in sub.iter().enumerate() {
                let name = match &sp.kind {
                    PatKind::Wildcard => continue,
                    PatKind::Binding(n) => n.clone(),
                    PatKind::Variant { .. } => return unsupported("nested match patterns"),
                };
                let (pty, off) = self.lay().payload_offset(&payloads, i);
                if !is_copy(&pty, self.items) {
                    return unsupported("move payload bind in match (copy binds only)");
                }
                let local = self.new_local(pty.clone(), Some(name.clone()));
                let mut src = splace.clone();
                src.proj.push(Proj::Field { offset: off, ty: pty.clone() });
                if self.is_wordy(&pty) {
                    self.emit(
                        StatementKind::Store(Place::local(local), Rvalue::Load { place: src, ty: pty.clone() }),
                        self.cur_span,
                        false,
                    );
                } else {
                    self.emit(
                        StatementKind::CopyVal { dst: Place::local(local), src, ty: pty.clone() },
                        self.cur_span,
                        false,
                    );
                }
                self.bind(&name, local, pty);
            }
        }
        match dst {
            Some((d, ty)) => self.lower_into(&arm.body, d, ty)?,
            None => {
                self.lower_value(&arm.body, None)?;
            }
        }
        self.pop_scope_with_drops();
        Ok(())
    }

    /// Lower `inner?` (spec 02 §6.5) for a *same-type* result enum: unwrap the
    /// `ok` variant's (word-sized) payload, or early-return the whole value with
    /// the enclosing scopes' drop schedule (INV-DROP). Cross-type `?` (a `From`
    /// conversion) and aggregate payloads remain out of subset.
    fn lower_try(&mut self, inner: &Expr, span: Span) -> LR<(Operand, Type)> {
        self.cur_span = span;
        let ety = match &inner.kind {
            ExprKind::Call { callee, .. } => match &callee.kind {
                ExprKind::Ident(n) => self
                    .items
                    .fns
                    .get(n.as_str())
                    .map(|s| s.ret.clone())
                    .ok_or_else(|| LowerError(format!("`?` on unknown call `{n}`")))?,
                _ => return unsupported("`?` on an indirect call"),
            },
            _ => return unsupported("`?` on a non-call operand"),
        };
        if ety != self.ret_ty {
            return unsupported("cross-type `?` (From conversion out of subset)");
        }
        let ename = match &ety {
            Type::Named(n) => n.clone(),
            _ => return unsupported("`?` on a non-nominal enum"),
        };
        let ok = self
            .items
            .enums
            .get(&ename)
            .and_then(|e| e.ok_variant.clone())
            .ok_or_else(|| LowerError("`?` on a non-result enum".into()))?;
        let einfo = self
            .lay()
            .enum_info(&ety)
            .ok_or_else(|| LowerError("`?` on a non-enum".into()))?;
        let ok_idx = einfo
            .iter()
            .position(|(n, _)| n == &ok)
            .ok_or_else(|| LowerError("`?`: missing ok variant".into()))?;
        // Materialize the operand into an enum temp.
        let tmp = self.new_local(ety.clone(), None);
        self.lower_into(inner, &Place::local(tmp), &ety)?;
        let splace = Place::local(tmp);
        let mut tagpl = splace.clone();
        tagpl.proj.push(Proj::Field { offset: 0, ty: Type::Scalar(ScalarTy::U64) });
        let tag = self.emit_temp(
            Type::Scalar(ScalarTy::U64),
            Rvalue::Load { place: tagpl, ty: Type::Scalar(ScalarTy::U64) },
            span,
        );
        let cmp = self.emit_temp(
            Type::bool(),
            Rvalue::Cmp { op: BinOp::Eq, l: Operand::Local(tag), r: Operand::Const(ok_idx as i128, ScalarTy::U64) },
            span,
        );
        let ok_bb = self.new_block();
        let err_bb = self.new_block();
        let join = self.new_block();
        self.terminate(Terminator::Branch { cond: Operand::Local(cmp), then_bb: ok_bb, else_bb: err_bb });
        // Error path: move the whole enum into the return place and early-return.
        self.switch_to(err_bb);
        self.emit(
            StatementKind::CopyVal { dst: Place::local(0), src: splace.clone(), ty: ety.clone() },
            span,
            false,
        );
        self.emit_return_drops();
        self.terminate(Terminator::Return);
        // Ok path: unwrap payload 0 (word-sized) as the `?` value.
        self.switch_to(ok_bb);
        let payloads = einfo[ok_idx].1.clone();
        if payloads.is_empty() {
            self.terminate(Terminator::Goto(join));
            self.switch_to(join);
            return Ok(self.unit());
        }
        let (pty, off) = self.lay().payload_offset(&payloads, 0);
        if !self.is_wordy(&pty) {
            return unsupported("`?` with an aggregate payload");
        }
        let mut ppl = splace.clone();
        ppl.proj.push(Proj::Field { offset: off, ty: pty.clone() });
        let val = self.emit_temp(pty.clone(), Rvalue::Load { place: ppl, ty: pty.clone() }, span);
        self.terminate(Terminator::Goto(join));
        self.switch_to(join);
        Ok((Operand::Local(val), pty))
    }

    /// Evaluate `e` and write it into the destination place `dst` of type `ty`
    /// (the move/copy substrate: struct/array construction, scalar/pointer stores,
    /// and whole-aggregate copies via [`StatementKind::CopyVal`]).
    fn lower_into(&mut self, e: &Expr, dst: &Place, ty: &Type) -> LR<()> {
        self.cur_span = e.span;
        match &e.kind {
            ExprKind::Paren(i) => self.lower_into(i, dst, ty),
            ExprKind::StructLit { name, fields } => {
                let (flayout, _, _) = self.lay().struct_layout(name);
                for (fname, fty, off) in flayout {
                    if let Some(fi) = find_field(fields, &fname) {
                        let mut sub = dst.clone();
                        sub.proj.push(Proj::Field { offset: off, ty: fty.clone() });
                        self.lower_into(&fi.value, &sub, &fty)?;
                    }
                }
                Ok(())
            }
            ExprKind::ArrayLit(elems) => {
                let elem = match ty {
                    Type::Array(x, _) => (**x).clone(),
                    _ => return unsupported("array literal of non-array type"),
                };
                let stride = self.stride_of(&elem);
                for (i, el) in elems.iter().enumerate() {
                    let mut sub = dst.clone();
                    sub.proj.push(Proj::Index {
                        index: Operand::Const(i as i128, ScalarTy::Usize),
                        stride,
                        len: elems.len() as u64,
                        span: el.span,
                    });
                    self.lower_into(el, &sub, &elem)?;
                }
                Ok(())
            }
            ExprKind::ArrayRepeat { value, size } => {
                let elem = match ty {
                    Type::Array(x, _) => (**x).clone(),
                    _ => return unsupported("array-repeat of non-array type"),
                };
                let n = match &size.kind {
                    ExprKind::IntLit { value, .. } => *value,
                    ExprKind::Ident(nm) => *self.consts.get(nm).unwrap_or(&0),
                    _ => return unsupported("non-constant array-repeat length"),
                };
                let stride = self.stride_of(&elem);
                // Evaluate the element once into a temp, then copy it into each slot
                // (mirrors the oracle: one `eval_value`, N byte-copies).
                let tmp = self.new_local(elem.clone(), None);
                self.lower_into(value, &Place::local(tmp), &elem)?;
                for i in 0..n {
                    let mut sub = dst.clone();
                    sub.proj.push(Proj::Index {
                        index: Operand::Const(i as i128, ScalarTy::Usize),
                        stride,
                        len: n,
                        span: self.cur_span,
                    });
                    self.emit(
                        StatementKind::CopyVal { dst: sub, src: Place::local(tmp), ty: elem.clone() },
                        self.cur_span,
                        false,
                    );
                }
                Ok(())
            }
            ExprKind::EnumCtor { enum_name, variant, args } => {
                if enum_name == "BoxResult" {
                    return unsupported("BoxResult constructor (Box out of subset)");
                }
                let einfo = self
                    .lay()
                    .enum_info(ty)
                    .ok_or_else(|| LowerError(format!("enum ctor of non-enum `{enum_name}`")))?;
                let idx = einfo
                    .iter()
                    .position(|(n, _)| n == variant)
                    .ok_or_else(|| LowerError(format!("no variant `{variant}`")))?;
                let payloads = einfo[idx].1.clone();
                let mut tagpl = dst.clone();
                tagpl.proj.push(Proj::Field { offset: 0, ty: Type::Scalar(ScalarTy::U64) });
                self.emit(
                    StatementKind::Store(tagpl, Rvalue::Use(Operand::Const(idx as i128, ScalarTy::U64))),
                    self.cur_span,
                    false,
                );
                for (i, arg) in args.iter().enumerate() {
                    let (pty, off) = self.lay().payload_offset(&payloads, i);
                    let mut sub = dst.clone();
                    sub.proj.push(Proj::Field { offset: off, ty: pty.clone() });
                    self.lower_into(arg, &sub, &pty)?;
                }
                Ok(())
            }
            ExprKind::Match { scrutinee, arms } => self.lower_match(scrutinee, arms, Some((dst, ty))),
            _ if self.is_wordy(ty) => {
                let (op, _) = self.lower_value(e, Some(ty))?;
                self.emit(StatementKind::Store(dst.clone(), Rvalue::Use(op)), self.cur_span, false);
                Ok(())
            }
            // Aggregate-by-value return: call, then byte-copy the out-slot the
            // callee returned (its address) into the destination.
            ExprKind::Call { callee, args } => {
                let name = match &callee.kind {
                    ExprKind::Ident(n) => n.clone(),
                    _ => return unsupported("indirect/method aggregate call"),
                };
                let sig = match self.items.fns.get(name.as_str()) {
                    Some(s) => s.clone(),
                    None => return unsupported(format!("aggregate call to `{name}`")),
                };
                let ops = self.lower_call_args(&sig, args)?;
                let addr = self.new_local(Type::RawPtr(Box::new(ty.clone())), None);
                self.emit(
                    StatementKind::Assign(addr, Rvalue::Call { func: name, args: ops }),
                    self.cur_span,
                    false,
                );
                let src = Place { root: addr, proj: vec![Proj::Deref { inner: ty.clone() }] };
                self.emit(
                    StatementKind::CopyVal { dst: dst.clone(), src, ty: ty.clone() },
                    self.cur_span,
                    false,
                );
                Ok(())
            }
            // Whole-aggregate value coming from a place (`let q = p;`): byte-copy.
            ExprKind::Ident(_)
            | ExprKind::Field { .. }
            | ExprKind::Index { .. }
            | ExprKind::Prefix { op: PrefixOp::Deref, .. } => {
                if needs_drop(ty, self.items) && !is_copy(ty, self.items) {
                    return unsupported("move of a needs-drop aggregate (static drop pruning unimplemented)");
                }
                let (src, _) = self.lower_place(e)?;
                self.emit(
                    StatementKind::CopyVal { dst: dst.clone(), src, ty: ty.clone() },
                    self.cur_span,
                    false,
                );
                Ok(())
            }
            _ => unsupported(format!("aggregate init {}", variant_name(&e.kind))),
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

    /// Lower a call's argument list to operands (word args by value; aggregate
    /// args by address into a byte-copy — see the callee's param binding).
    fn lower_call_args(&mut self, sig: &crate::resolve::FnSig, args: &[Expr]) -> LR<Vec<Operand>> {
        if sig.params.len() != args.len() {
            return unsupported("arity mismatch");
        }
        let mut ops = Vec::new();
        for (p, a) in sig.params.iter().zip(args) {
            if !matches!(p.mode, ParamMode::Take | ParamMode::Read | ParamMode::Write) {
                return unsupported("out/unsupported argument mode");
            }
            let a: &Expr = match &a.kind {
                ExprKind::OutArg(inner) => inner,
                _ => a,
            };
            if self.is_wordy(&p.lowered) {
                let (op, _) = self.lower_value(a, Some(&p.lowered))?;
                ops.push(op);
            } else {
                if !is_copy(&p.lowered, self.items) && needs_drop(&p.lowered, self.items) {
                    return unsupported("move of a needs-drop aggregate argument");
                }
                let place = self.materialize_place(a, &p.lowered)?;
                let id = self.emit_temp(
                    Type::RawPtr(Box::new(p.lowered.clone())),
                    Rvalue::Ref(place),
                    self.cur_span,
                );
                ops.push(Operand::Local(id));
            }
        }
        Ok(ops)
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
        // Aggregate-by-value returns are handled by `lower_into`; here only
        // word-sized (and unit) returns flow through the scalar `Call` rvalue.
        if !self.is_wordy(&sig.ret) && !matches!(sig.ret, Type::Scalar(ScalarTy::Unit)) {
            return unsupported("aggregate return in value position");
        }
        let ops = self.lower_call_args(&sig, args)?;
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
        let scope_depth = self.scopes.len();
        self.loops.push(Loop { continue_bb: head, break_bb: exit, scope_depth });
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
        let scope_depth = self.scopes.len();
        self.loops.push(Loop { continue_bb: body_bb, break_bb: exit, scope_depth });
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
            self.lower_into(e, &Place::local(0), &ret_ty)?;
        }
        self.emit_return_drops();
        self.terminate(Terminator::Return);
        Ok(())
    }
}

fn find_field<'f>(fields: &'f [FieldInit], name: &str) -> Option<&'f FieldInit> {
    fields.iter().find(|f| f.name == name)
}

#[allow(dead_code)]
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
