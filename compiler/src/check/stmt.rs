//! Statement type checking and the small accessors the CFG lowering uses
//! (design 0001 §7.4 definite assignment; §1.5 reassignment drops the old value
//! — recorded as an `Assign` action for Stage 4).

use crate::ast::*;
use crate::types::{bears_box, box_subpaths, needs_drop, Type};

use super::dataflow::{Access, Place};
use crate::ast::{ExprKind, PrefixOp};
use super::{Checker, Use};

impl<'a> Checker<'a> {
    pub(super) fn check_block_stmts(&mut self, b: &Block) {
        self.push_scope();
        for s in &b.stmts {
            self.check_stmt(s);
        }
        self.emit_scope_exits(self.f.env.len() - 1, b.span);
        self.pop_scope();
    }

    pub(super) fn check_block_value(&mut self, b: &Block) -> Type {
        self.push_scope();
        for s in &b.stmts {
            self.check_stmt(s);
        }
        self.emit_scope_exits(self.f.env.len() - 1, b.span);
        let diverged = self.cur_get().is_none();
        self.pop_scope();
        if diverged {
            Type::Never
        } else {
            Type::unit()
        }
    }

    pub(super) fn check_stmt(&mut self, s: &Stmt) {
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
                        let nd = needs_drop(&t, self.items);
                        let bp = box_subpaths(&t, self.items);
                        self.add_local(name, t, true);
                        self.emit(
                            &Some(Place::local(name.clone())),
                            Access::Assign { needs_drop: nd, box_paths: bp },
                            s.span,
                        );
                        // A borrow value landing in a binding anchors the loan(s)
                        // it carries to this binding's live range (design §2.3):
                        // a fresh borrow, a return-extended call result, or a copy
                        // of an existing borrow (a bare identifier used to shed its
                        // loan here — the loan-copy UAF).
                        if self.carries_borrow(e) {
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
                self.reject_static_mutation(target, "assign to", s.span);
                self.reject_write_through_shared(target, "assign to", s.span);
                self.reject_autoderef_write(target);
                let (tt, place) = self.check_place(target);
                self.clear_carried();
                self.check_against(value, &tt);
                if place.is_some() {
                    self.emit(
                        &place,
                        Access::Assign {
                            needs_drop: needs_drop(&tt, self.items),
                            box_paths: box_subpaths(&tt, self.items),
                        },
                        s.span,
                    );
                }
                match &place {
                    // A borrow value assigned to a whole binding anchors the loan(s)
                    // it carries to that binding (§2.3); a store through a projection
                    // targets no borrow binding (§3.4 bans borrow fields), so any
                    // carried loan is dropped.
                    Some(p) if self.carries_borrow(value) && p.proj.is_empty() => {
                        let name = p.root.clone();
                        self.anchor_carried(&name);
                    }
                    _ => self.clear_carried(),
                }
            }
            StmtKind::Expr(e) => {
                let t = self.check_expr(e, Use::Value);
                // A `Box`-bearing temporary is dropped (freed) at the end of the
                // statement that created it (§1.5) — the free side of the alloc
                // effect (finding 4; §6.2/§6.3).
                if bears_box(&t, self.items) {
                    self.note_alloc(
                        e.span,
                        "a `Box`-bearing temporary is dropped (freed) at the end of this statement (§6.2/§6.3)",
                    );
                }
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
        let depth = self.f.env.len();
        self.f.loops.push((cont, brk, depth));
    }
    pub(super) fn loops_pop(&mut self) {
        self.f.loops.pop();
    }
    pub(super) fn loops_break(&self) -> Option<usize> {
        self.f.loops.last().map(|(_, b, _)| *b)
    }
    pub(super) fn loops_continue(&self) -> Option<usize> {
        self.f.loops.last().map(|(c, _, _)| *c)
    }
    /// The env scope depth outside the innermost loop body (§1.6 dual): the
    /// scopes `break`/`continue` unwinds are `env[depth..]`.
    pub(super) fn loops_scope_depth(&self) -> Option<usize> {
        self.f.loops.last().map(|(_, _, d)| *d)
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
    pub(super) fn f_in_contract(&self) -> bool {
        self.f.in_contract
    }
    pub(super) fn set_in_contract(&mut self, v: bool) {
        self.f.in_contract = v;
    }
    pub(super) fn ret_ty_clone(&self) -> Type {
        self.f.ret_ty.clone()
    }

    /// Whether an expression's *value* is a borrow that carries a loan needing to
    /// be anchored at its landing binding (design §2.1/§3.1). Recognized: an
    /// explicit `read`/`write` borrow, a slice op, a `str`/`[u8]` view retype of a
    /// String (`as_str`/`as_bytes`/`substr`/`str_from_unchecked` — the view aliases
    /// the source String's heap buffer), a call whose signature returns a borrow OR
    /// a view (`[T]`/`str`) (its return-extended loan is carried), and a bare place already holding a
    /// `read`/`write` borrow, a `slice`/`slice_mut`, or a `str` view — a copy that
    /// aliases the source, so the source loan must extend to the new binding
    /// (`let c = b;` / `let s2 = s;`). Without the last case a copied borrow or
    /// view shed its loan, admitting a use-after-free.
    pub(super) fn carries_borrow(&mut self, e: &crate::ast::Expr) -> bool {
        match &e.kind {
            ExprKind::Paren(i) => self.carries_borrow(i),
            ExprKind::Prefix {
                op: PrefixOp::Read | PrefixOp::Write,
                ..
            } => true,
            ExprKind::Ident(name) => matches!(
                self.lookup_local(name).map(|li| &li.ty),
                Some(
                    Type::Borrow(_) | Type::BorrowMut(_) | Type::Slice(_) | Type::SliceMut(_) | Type::Str
                )
            ),
            ExprKind::Call { callee, .. } => {
                if let ExprKind::Ident(name) = &callee.kind {
                    if matches!(
                        name.as_str(),
                        "slice_of"
                            | "slice_of_mut"
                            | "subslice"
                            | "as_str"
                            | "as_bytes"
                            | "substr"
                            | "str_from_unchecked"
                    ) {
                        return true;
                    }
                    if let Some(sig) = self.items.fns.get(name) {
                        // A user fn returning a borrow OR a view (`[T]`/`str`) carries
                        // its return-extended argument loan, exactly as a borrow return
                        // does; without this a view laundered out of a call sheds the
                        // source loan (the function-return view UAF).
                        return sig.ret.is_borrow_kind();
                    }
                    return false;
                }
                // A method call `recv.m(args)` whose resolved method returns a borrow
                // reborrows the receiver, so it carries the receiver's loan out of the
                // call — the same return-extension a free-fn borrow return gets (design
                // 0015 §4.3/§5; the `get_ref` yield). Without this the `for read` yield,
                // and any borrow-returning interface method, sheds its source loan and
                // the escape UAF §5 case (2) forbids slips through.
                if let ExprKind::Field { base, field, .. } = &callee.kind {
                    return self.method_returns_borrow(base, field);
                }
                false
            }
            _ => false,
        }
    }

    /// Does `base.field(..)`, resolved against the receiver's interface impl (or the
    /// `Vec`-wired `get_ref`), return a borrow? Used by `carries_borrow` to decide
    /// whether a method call's returned reborrow keeps its source loan. Pure w.r.t.
    /// the carried-loan state (it probes the receiver type via `synth_arg_type`,
    /// which it saves/restores around).
    fn method_returns_borrow(&mut self, base: &crate::ast::Expr, field: &str) -> bool {
        let saved = std::mem::take(&mut self.f.carried);
        let saved_prov = self.f.carried_prov.take();
        let recv_ty = self.synth_arg_type(base);
        self.f.carried = saved;
        self.f.carried_prov = saved_prov;
        if field == "get_ref" {
            if let Type::App(n, _) = &recv_ty {
                if n == "Vec" {
                    return true;
                }
            }
        }
        let ret = match &recv_ty {
            Type::Param(p) => self
                .param_bound_ifaces(p)
                .iter()
                .find_map(|i| self.iface_method(i, field))
                .map(|m| m.ret),
            Type::Named(_) | Type::App(_, _) | Type::Scalar(_) => (0..self.items.impls.len())
                .find(|&i| {
                    self.items.impls[i].methods.contains_key(field) && self.impl_covers(i, &recv_ty)
                })
                .and_then(|idx| {
                    let iface = self.items.impls[idx].iface.clone();
                    self.iface_method(&iface, field)
                })
                .map(|m| m.ret),
            _ => None,
        };
        matches!(ret, Some(t) if t.is_borrow_kind())
    }
}

