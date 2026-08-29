//! Statement type checking and the small accessors the CFG lowering uses
//! (design 0001 §7.4 definite assignment; §1.5 reassignment drops the old value
//! — recorded as an `Assign` action for Stage 4).

use crate::ast::*;
use crate::types::{bears_box, box_subpaths, ground_nested_int_lit, may_store_borrow, needs_drop, Type};

use super::dataflow::{Access, Place};
use crate::ast::{ExprKind, PrefixOp};
use super::{Checker, Use};

impl<'a> Checker<'a> {
    pub(super) fn check_block_stmts(&mut self, b: &Block) {
        // A block's STATEMENTS are their own typing contexts: the surrounding
        // slot's expected type describes the block's value (an arm or branch
        // expression), never a statement inside it, so the expectation clears
        // at this boundary (B1) — a `let y = <literal>;` in an arm body must
        // ground and range-check against its own annotation or the i64
        // default, not the outer slot's scalar.
        let saved = self.expected_ty.take();
        self.push_scope();
        for s in &b.stmts {
            self.check_stmt(s);
        }
        self.emit_scope_exits(self.f.env.len() - 1, b.span);
        self.pop_scope();
        self.expected_ty = saved;
    }

    pub(super) fn check_block_value(&mut self, b: &Block) -> Type {
        // Same statement-boundary clearing as `check_block_stmts` (B1): the
        // block yields unit/Never, so the outer expectation never describes
        // anything inside it.
        let saved = self.expected_ty.take();
        self.push_scope();
        for s in &b.stmts {
            self.check_stmt(s);
        }
        self.emit_scope_exits(self.f.env.len() - 1, b.span);
        let diverged = self.cur_get().is_none();
        self.pop_scope();
        self.expected_ty = saved;
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
                            // An unannotated binding grounds a `{integer}`
                            // carried inside a composite (an all-unsuffixed
                            // array literal) to the `i64` default at the
                            // landing site (P5): a composite's layout is fixed
                            // by its element type, so `let t = [1, 2];` is
                            // `[2]i64` — a later `[2]u8` slot is then E0703,
                            // not a silently truncating copy. This is a
                            // deliberate ASYMMETRY with the scalar rule: a
                            // bare scalar `{integer}` keeps its flexibility
                            // through a `let` (`let x = 1; let y: u8 = x;`
                            // stays legal), but composite flexibility through
                            // a binding was producing silently wrong values
                            // (the engines had already fixed an i64 stride),
                            // so composites give it up here.
                            None => ground_nested_int_lit(&self.check_expr(e, Use::Value)),
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
                // An assignment into an `out`-mode slot whose type stores
                // borrows is semantically a RETURN of a borrow (P7, spec 04
                // §6.5/§7): the value escapes into the caller's frame, so it
                // must obey the same provenance rules as a returned borrow
                // (E0806 family). Covers the whole slot and element stores
                // into an out array-of-borrows alike. The return's own region
                // tag does not govern the out slot, so it is masked here.
                if may_store_borrow(&tt)
                    && place
                        .as_ref()
                        .is_some_and(|p| self.f.out_params.contains(&p.root))
                {
                    let saved_region = self.f.ret_region.take();
                    self.f.out_prov = place.as_ref().map(|p| p.root.clone());
                    self.check_return_provenance(value);
                    self.f.out_prov = None;
                    self.f.ret_region = saved_region;
                }
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
                    // it carries to that binding (§2.3). A store through a projection
                    // usually targets no borrow slot (§3.4 bans borrow fields) and
                    // drops the carried loan; the exception is a projected slot whose
                    // TYPE is a borrow — in practice an array element (P4) — where the
                    // loan anchors on the place's root binding, the whole-array
                    // granularity choice. (For a deref store the root is the
                    // written-through binding, a conservative approximation: its live
                    // range may end before the slot's.)
                    Some(p)
                        if self.carries_borrow(value)
                            && (p.proj.is_empty() || may_store_borrow(&tt)) =>
                    {
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
            // `may_store_borrow` is the borrow-value type predicate: the five
            // borrow kinds plus arrays of them (P4, whole-array granularity),
            // plus an opaque `I::Item` inside generic code (P22(a)).
            ExprKind::Ident(name) => self
                .lookup_local(name)
                .map(|li| may_store_borrow(&li.ty))
                .unwrap_or(false),
            // An array element read (`a[0]`) copies a borrow out of the array,
            // aliasing the same borrowed place; a whole-array or nested-array
            // read likewise. Probe the place's type without emitting.
            ExprKind::Index { .. } => {
                let (t, _) = self.write_path_probe(e);
                may_store_borrow(&t)
            }
            // An array of borrows carries every element's loan (P4).
            ExprKind::ArrayLit(elems) => {
                elems.iter().any(|el| self.carries_borrow(el))
            }
            ExprKind::ArrayRepeat { value, .. } => self.carries_borrow(value),
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
                    if self.items.generic_fns.contains_key(name) {
                        // Borrow-ness of a generic call's return depends on the
                        // instantiation (P16/P22b): consult the substituted-return
                        // answer recorded when the call was checked, exactly as
                        // the P11 method-call path below does.
                        return self.f.borrow_valued.contains(&(e.span.start, e.span.end));
                    }
                    if let Some(sig) = self.items.fns.get(name) {
                        // A user fn returning a borrow OR a view (`[T]`/`str`) carries
                        // its return-extended argument loan, exactly as a borrow return
                        // does; without this a view laundered out of a call sheds the
                        // source loan (the function-return view UAF). An array of
                        // borrows aliases its sources the same way (P4).
                        return may_store_borrow(&sig.ret);
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
                    // The substituted-return answer recorded when the call was
                    // checked (P11) overrides the raw-signature probe: a method
                    // declared `-> Self::Item` is borrow-returning whenever the
                    // impl's binding of `Item` is, which the unsubstituted
                    // signature cannot show.
                    if self.f.borrow_valued.contains(&(e.span.start, e.span.end)) {
                        return true;
                    }
                    return self.method_returns_borrow(base, field);
                }
                false
            }
            // A `match` whose checked result type stores a borrow carries the
            // union of its arms' loans (P8); recorded when the match was
            // checked — arm syntax (e.g. a rebind of a pattern-bound borrow)
            // is not re-derivable at the landing site.
            ExprKind::Match { .. } => self.f.borrow_valued.contains(&(e.span.start, e.span.end)),
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
        let recv_ty = self.synth_arg_type(base);
        self.f.carried = saved;
        if field == "get_ref" {
            if let Type::App(n, _) = &recv_ty {
                if n == "Vec" {
                    return true;
                }
            }
        }
        let ret = self.iface_method_for_ty(&recv_ty, field).map(|m| m.ret);
        // Arrays of borrows returned from a method alias their sources exactly
        // as a bare borrow return does (P4); so does an opaque `Self::Item`
        // return at a generic def site (P22(a)).
        matches!(ret, Some(t) if may_store_borrow(&t))
    }

    /// Resolve `base.field(..)` to its interface method declaration, probing
    /// the receiver's type. Pure w.r.t. the carried-loan state (the probe is
    /// saved/restored around). `None` when the receiver answers no interface
    /// method by that name (including the `Vec`-wired `get_ref`/`at`, which
    /// have no declaration to return).
    pub(super) fn receiver_iface_method(
        &mut self,
        base: &crate::ast::Expr,
        field: &str,
    ) -> Option<crate::resolve::IfaceMethod> {
        let saved = std::mem::take(&mut self.f.carried);
        let recv_ty = self.synth_arg_type(base);
        self.f.carried = saved;
        self.iface_method_for_ty(&recv_ty, field)
    }

    /// The interface method a receiver TYPE answers for `field`: a bound
    /// interface's method for an opaque type parameter, or the covering
    /// impl's interface method for a nominal/instantiated/scalar receiver.
    fn iface_method_for_ty(&self, recv_ty: &Type, field: &str) -> Option<crate::resolve::IfaceMethod> {
        match recv_ty {
            Type::Param(p) => self
                .param_bound_ifaces(p)
                .iter()
                .find_map(|i| self.iface_method(i, field)),
            Type::Named(_) | Type::App(_, _) | Type::Scalar(_) => (0..self.items.impls.len())
                .find(|&i| {
                    self.items.impls[i].methods.contains_key(field) && self.impl_covers(i, recv_ty)
                })
                .and_then(|idx| {
                    let iface = self.items.impls[idx].iface.clone();
                    self.iface_method(&iface, field)
                }),
            _ => None,
        }
    }
}

