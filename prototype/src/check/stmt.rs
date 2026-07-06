//! Statement type checking and the small accessors the CFG lowering uses
//! (design 0001 §7.4 definite assignment; §1.5 reassignment drops the old value
//! — recorded as an `Assign` action for Stage 4).

use crate::ast::*;
use crate::types::Type;

use super::dataflow::{Access, Place};
use crate::ast::{ExprKind, PrefixOp};
use super::{Checker, Use};

impl<'a> Checker<'a> {
    pub(super) fn check_block_stmts(&mut self, b: &Block) {
        self.push_scope();
        for s in &b.stmts {
            self.check_stmt(s);
        }
        self.pop_scope();
    }

    pub(super) fn check_block_value(&mut self, b: &Block) -> Type {
        self.push_scope();
        for s in &b.stmts {
            self.check_stmt(s);
        }
        let diverged = self.cur_get().is_none();
        self.pop_scope();
        if diverged {
            Type::Never
        } else {
            Type::unit()
        }
    }

    fn check_stmt(&mut self, s: &Stmt) {
        match &s.kind {
            StmtKind::Let {
                name, ty, init, ..
            } => {
                let decl_ty = ty.as_ref().map(|t| self.resolve_ty(t));
                match init {
                    Some(e) => {
                        self.clear_carried();
                        let t = match &decl_ty {
                            Some(dt) => {
                                self.check_against(e, dt);
                                dt.clone()
                            }
                            None => self.check_expr(e, Use::Value),
                        };
                        self.add_local(name, t, true);
                        self.emit(&Some(Place::local(name.clone())), Access::Assign, s.span);
                        // A borrow value landing in a binding anchors its loan(s)
                        // to this binding's live range (design §2.3).
                        if carries_borrow(self, e) {
                            self.anchor_carried(name);
                        } else {
                            self.clear_carried();
                        }
                    }
                    None => {
                        let t = decl_ty.unwrap_or(Type::Error);
                        self.add_local(name, t, true);
                        self.emit(&Some(Place::local(name.clone())), Access::Decl, s.span);
                    }
                }
            }
            StmtKind::Assign { target, value } => {
                let (tt, place) = self.check_place(target);
                self.clear_carried();
                self.check_against(value, &tt);
                if place.is_some() {
                    self.emit(&place, Access::Assign, s.span);
                }
                if let (true, Some(p)) = (carries_borrow(self, value), &place) {
                    if p.proj.is_empty() {
                        let name = p.root.clone();
                        self.anchor_carried(&name);
                    } else {
                        self.clear_carried();
                    }
                } else {
                    self.clear_carried();
                }
            }
            StmtKind::Expr(e) => {
                self.check_expr(e, Use::Value);
            }
        }
    }

    // ----- accessors over the per-function CFG builder state --------------

    pub(super) fn cur_get(&self) -> Option<usize> {
        self.f.cur
    }
    pub(super) fn cur_set(&mut self, v: Option<usize>) {
        self.f.cur = v;
    }
    pub(super) fn set_join_span(&mut self, b: usize, span: crate::span::Span) {
        self.f.blocks[b].join_span = span;
    }
    pub(super) fn loops_push(&mut self, cont: usize, brk: usize) {
        self.f.loops.push((cont, brk));
    }
    pub(super) fn loops_pop(&mut self) {
        self.f.loops.pop();
    }
    pub(super) fn loops_break(&self) -> Option<usize> {
        self.f.loops.last().map(|(_, b)| *b)
    }
    pub(super) fn loops_continue(&self) -> Option<usize> {
        self.f.loops.last().map(|(c, _)| *c)
    }
    pub(super) fn in_unsafe_get(&self) -> bool {
        self.f.in_unsafe
    }
    pub(super) fn set_in_unsafe(&mut self, v: bool) {
        self.f.in_unsafe = v;
    }
    pub(super) fn in_ensures_get(&self) -> bool {
        self.f.in_ensures
    }
    pub(super) fn ret_ty_clone(&self) -> Type {
        self.f.ret_ty.clone()
    }
}

/// Whether an expression's *value* is a borrow that carries a loan needing to be
/// anchored at its landing binding (design §2.1/§3.1). A conservative syntactic
/// whitelist: explicit borrows, slice ops, and calls that return a borrow.
fn carries_borrow(c: &Checker, e: &crate::ast::Expr) -> bool {
    match &e.kind {
        ExprKind::Paren(i) => carries_borrow(c, i),
        ExprKind::Prefix {
            op: PrefixOp::Read | PrefixOp::Write,
            ..
        } => true,
        ExprKind::Call { callee, .. } => {
            if let ExprKind::Ident(name) = &callee.kind {
                if matches!(name.as_str(), "slice_of" | "slice_of_mut" | "subslice") {
                    return true;
                }
                if let Some(sig) = c.items.fns.get(name) {
                    return matches!(
                        sig.ret,
                        crate::types::Type::Borrow(_) | crate::types::Type::BorrowMut(_)
                    );
                }
            }
            false
        }
        _ => false,
    }
}
