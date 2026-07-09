//! Structured-concurrency checking (design 0012): the `scope` region, the unified
//! spawn-crossing gate (§2.1), scope-length loans (§1.2), the `split_mut` blessed
//! primitive (§1.4), and the cooperative-cancellation surface (§3.3).
//!
//! Execution is the Stage-2 sequential oracle (interp/eval.rs, mir): a spawn runs
//! its task to completion at the spawn point, valid by SC-for-DRF (§6). This file
//! is the compile-time guarantee: no DRF-violating program type-checks.

use crate::ast::*;
use crate::diag::Diag;
use crate::span::Span;
use crate::types::*;

use super::dataflow::{Access, LoanKind, Place};
use super::loans::Anchor;
use super::Use;
use super::Checker;

impl<'a> Checker<'a> {
    // -----------------------------------------------------------------------
    // `scope { ... }` — the concurrency region (design 0012 §1.1)
    // -----------------------------------------------------------------------
    pub(super) fn check_scope(&mut self, b: &Block, span: Span) -> Type {
        // A `scope` is control flow; it may not appear inside a contract clause
        // (design 0012; E0708 family, the read-only-contract rule).
        if self.f_in_contract() {
            self.reject_in_contract(span, "a `scope` concurrency region");
        }
        self.scope_enter();
        self.push_scope();
        for s in &b.stmts {
            self.check_stmt(s);
        }
        // The whole-scope loan range (§1.2): a synthetic use of each spawn-crossing
        // borrow's scope-length binding AT the closing brace keeps its loan live to
        // the brace, so any parent access to a borrowed place during the scope, and
        // any conflicting sibling borrow, is rejected by the ordinary XOR scan.
        let synth = self.scope_synth_pop();
        for name in synth {
            self.emit(&Some(Place::local(name)), Access::Read, b.span);
        }
        self.emit_scope_exits(self.env_len() - 1, b.span);
        self.pop_scope();
        self.scope_exit();
        Type::unit()
    }

    // -----------------------------------------------------------------------
    // `spawn CALLEE(ARGS)` — start one task (design 0012 §1.1, §2.1 the gate)
    // -----------------------------------------------------------------------
    pub(super) fn check_spawn(&mut self, call: &Expr, span: Span) -> Type {
        if self.f_in_contract() {
            self.reject_in_contract(span, "a `spawn` statement");
        }
        // §5: a `spawn` outside any `scope` is a checker error (not a parse error).
        if self.scope_depth_get() == 0 {
            self.diags.push(
                Diag::error(
                    "E1201",
                    "`spawn` is only valid inside a `scope { ... }` region",
                    span,
                )
                .with_note("a task must join at a scope's closing brace; there are no detached threads (design 0012 §1.1)", None),
            );
            // Still type-check the callee to surface cascades.
            self.check_expr(call, Use::Value);
            return Type::unit();
        }
        let (callee, args) = match &call.kind {
            ExprKind::Call { callee, args } => (callee.as_ref(), args.as_slice()),
            _ => return Type::unit(),
        };
        let name = match &callee.kind {
            ExprKind::Ident(n) => n.clone(),
            _ => {
                self.diags.push(
                    Diag::error(
                        "E1205",
                        "a `spawn` callee must be a named function",
                        span,
                    )
                    .with_note("the task body is a fn or fn-pointer named at the spawn site (design 0012 §1.1)", None),
                );
                self.check_expr(call, Use::Value);
                return Type::unit();
            }
        };
        // Resolve the callee's parameter modes + declared types (concrete or generic).
        let (params, callee_alloc): (Vec<(ParamMode, Type)>, bool) = if let Some(sig) = self.items.fns.get(&name) {
            (sig.params.iter().map(|p| (p.mode, p.decl_ty.clone())).collect(), sig.alloc)
        } else if let Some(g) = self.items.generic_fns.get(&name) {
            (g.params.iter().map(|p| (p.mode, p.decl_ty.clone())).collect(), g.alloc)
        } else {
            self.diags.push(Diag::error(
                "E0102",
                format!("`spawn` of unknown function `{name}`"),
                span,
            ));
            return Type::unit();
        };
        // The `alloc` effect crosses the spawn (design 0012 §2.5): a spawned
        // `alloc`-callee makes the function containing the `scope` `alloc`-marked.
        if callee_alloc {
            self.note_alloc(span, format!("`spawn` of `alloc` function `{name}` propagates the `alloc` effect across the spawn boundary (design 0012 §2.5)"));
        }
        if params.len() != args.len() {
            self.diags.push(Diag::error(
                "E0706",
                format!("function `{}` expects {} argument(s), found {}", name, params.len(), args.len()),
                span,
            ));
        }
        for ((mode, decl_ty), arg) in params.iter().zip(args) {
            self.gate_spawn_arg(*mode, decl_ty, arg);
        }
        Type::unit()
    }

    /// The unified spawn-crossing gate (design 0012 §2.1), one argument.
    ///
    /// Branch selection (§2.1): a `read`/`write` argument, AND any `slice`/
    /// `slice_mut`/borrow value passed by value (even to a `take`-mode parameter),
    /// takes the BORROW branch — it crosses as a scope-length loan and the gate
    /// falls on the REFERENT. Everything else is the ownership-transfer (`take`)
    /// branch and the gate falls on the whole transferred type.
    fn gate_spawn_arg(&mut self, mode: ParamMode, decl_ty: &Type, arg: &Expr) {
        let inner = spawn_inner(arg);
        // Decide the branch + (for the borrow branch) the loan kind and referent.
        let borrow: Option<(LoanKind, Type)> = match mode {
            ParamMode::Read => Some((LoanKind::Shared, decl_ty.clone())),
            ParamMode::Write => Some((LoanKind::Excl, decl_ty.clone())),
            ParamMode::Out => {
                self.diags.push(
                    Diag::error(
                        "E1204",
                        "an `out` argument may not cross a spawn boundary",
                        arg.span,
                    )
                    .with_note("a task returns results by writing a `write`-borrowed slot the parent owns, not through an `out` parameter (design 0012 §3.1)", None),
                );
                self.check_expr(inner, Use::Value);
                self.clear_carried();
                return;
            }
            // A `take`-mode parameter whose type is a slice/borrow VALUE crosses as a
            // borrow (§1.2/§2.1): the exclusive/shared loan is on the run it views.
            ParamMode::Take => match decl_ty {
                Type::Slice(e) | Type::Borrow(e) => Some((LoanKind::Shared, (**e).clone())),
                Type::SliceMut(e) | Type::BorrowMut(e) => Some((LoanKind::Excl, (**e).clone())),
                _ => None,
            },
        };
        match borrow {
            // Ownership-transfer branch: the whole transferred type must be portable.
            None => {
                let at = self.check_expr(inner, Use::Value);
                self.clear_carried();
                let gate_ty = if matches!(at, Type::Error) { decl_ty } else { &at };
                if let Some(w) = non_portable_witness(gate_ty, self.items) {
                    self.diags.push(
                        Diag::error(
                            "E1202",
                            format!("cannot `take` a non-`portable` value of type `{}` across a spawn", gate_ty.display()),
                            arg.span,
                        )
                        .with_note(
                            format!("`{}` is reachable here; a `take` transfers every owned byte to the task, so the whole type must be `portable` — no `rawptr`, no borrow, transitively including through `copy` fields (design 0012 §2.1)", w.display()),
                            None,
                        ),
                    );
                }
            }
            // Borrow branch: the REFERENT must be portable; the borrow becomes a
            // scope-length loan (§1.2) held to the closing brace.
            Some((kind, referent)) => {
                let (rty, place) = self.check_place(inner);
                let u = if kind == LoanKind::Shared { Use::BorrowShared } else { Use::BorrowExcl };
                self.clear_carried();
                self.emit_place_action(&place, u, &rty, inner.span);
                let ids = self.take_carried();
                if let Some(w) = non_portable_witness(&referent, self.items) {
                    self.diags.push(
                        Diag::error(
                            "E1203",
                            format!("cannot share a non-`portable` referent `{}` across a spawn", referent.display()),
                            arg.span,
                        )
                        .with_note(
                            format!("`{}` is reachable behind this borrow; a `copy` referent can be copied out and launder it, so the referent must be `portable` (design 0012 §2.1/§2.3)", w.display()),
                            None,
                        ),
                    );
                }
                if place.is_some() {
                    let sname = self.fresh_scope_loan_name();
                    self.reanchor_scope_loan(&ids, &sname);
                    self.emit(
                        &Some(Place::local(sname.clone())),
                        Access::Assign { needs_drop: false, box_paths: Vec::new() },
                        arg.span,
                    );
                    self.scope_synth_push(sname);
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // `split_mut` — the priced blessed disjoint-partition primitive (§1.4)
    // -----------------------------------------------------------------------
    //
    // `split_mut(PARENT, MID, out LO, out HI)`: reborrows one exclusive slice/array
    // `PARENT` as two exclusive halves bound to the distinct locals `LO`/`HI`. The
    // index-insensitive loan scan (0001 §2.2) cannot derive `[0,mid) ⊥ [mid,len)`,
    // so this disjointness is STIPULATED BY FIAT: `split_mut` binds the two halves
    // to two DISTINCT place roots (`LO`, `HI`) — which the scan treats as disjoint —
    // while holding an exclusive parent-freeze loan on `PARENT` for as long as
    // either half is live. Two halves thus cross into sibling spawns without mutual
    // conflict (the parallel-fill flagship), yet any third access to `PARENT` — and
    // any hand-rolled second exclusive view of it — conflicts. This is the one
    // place the scan is told an answer it cannot compute (§1.4, a new blessed
    // exception on the P6 ledger, not a loan-scan fall-out).
    //
    // NOTE (under-specification, reported): the design writes `split_mut` with a
    // tuple return `-> (write [T], write [T])`. The prototype AST has no tuples, so
    // the results are delivered through two `out` slots — an equivalent surface.
    pub(super) fn check_split_mut(&mut self, args: &[Expr], span: Span) -> Type {
        if args.len() != 4 {
            self.diags.push(Diag::error("E0706", "`split_mut` expects 4 argument(s): (parent, mid, out lo, out hi)", span));
            for a in args { self.check_expr(a, Use::Value); }
            return Type::unit();
        }
        // arg0: the parent exclusive slice/array place, frozen for the halves' life.
        let (pt, pplace) = self.check_place(&args[0]);
        let elem = match &pt {
            Type::Array(e, _) | Type::SliceMut(e) => (**e).clone(),
            Type::Slice(_) => {
                self.e0809("exclusive reborrow of a shared slice in `split_mut`", args[0].span);
                Type::Error
            }
            Type::Error => Type::Error,
            other => {
                self.mismatch(span, "split_mut", "a `slice_mut`/array place", other);
                Type::Error
            }
        };
        // NB: no borrow ACTION is emitted on the parent here. The two per-half
        // parent-freeze loans below (anchored to `lo`/`hi`) ARE the exclusive
        // borrow of the parent; because the `out` slots read as loan uses, the
        // halves are live across this call, so any parent-place action emitted
        // here would spuriously conflict with the freeze loans it is creating.
        // Downstream conflicts (a third access, a hand-rolled second view) are
        // caught by the freeze loans directly.
        // arg1: the split index.
        let mid = self.check_expr(&args[1], Use::Value);
        self.expect_integer(&mid, args[1].span);
        let result_ty = Type::SliceMut(Box::new(elem));
        // arg2 / arg3: the two out slots. Each is initialized to a half, and pins an
        // exclusive parent-freeze loan on PARENT over its own live range.
        for oi in [2usize, 3usize] {
            let inner = match &args[oi].kind {
                ExprKind::OutArg(i) => i.as_ref(),
                _ => {
                    self.diags.push(
                        Diag::error("E0307", "each `split_mut` result must be an `out place`", args[oi].span)
                            .with_note("spell it `out lo` / `out hi`: caller-owned slots the primitive fills (design 0012 §1.4)", None),
                    );
                    continue;
                }
            };
            let (slot_ty, slot_place) = self.check_place(inner);
            if !matches!(slot_ty, Type::Error) && !assignable(&result_ty, &slot_ty) {
                self.diags.push(Diag::error(
                    "E0703",
                    format!("`split_mut` result type `{}` does not match slot `{}`", result_ty.display(), slot_ty.display()),
                    args[oi].span,
                ));
            }
            if let Some(sp) = &slot_place {
                // The slot is initialized here (an out-init, path-independent).
                self.emit(
                    &slot_place,
                    Access::OutArg { needs_drop: false, box_paths: Vec::new() },
                    args[oi].span,
                );
                // Parent-freeze: an exclusive loan on PARENT, anchored to this slot's
                // binding — live exactly while the half is live (§1.4).
                if let Some(pp) = &pplace {
                    let root = sp.root.clone();
                    self.record_binding_loan(&pp.canonical(), LoanKind::Excl, span, &root);
                }
            }
        }
        Type::unit()
    }

    // -----------------------------------------------------------------------
    // Cooperative cancellation (design 0012 §3.3)
    // -----------------------------------------------------------------------
    /// `cancelled(read Cancel) -> bool` — a plain query at an explicit cancellation
    /// point. Under the sequential oracle a task always runs to completion, so the
    /// token never signals; the surface exists so cancellation-point control flow
    /// type-checks and executes (design 0012 §3.3).
    pub(super) fn check_cancelled(&mut self, args: &[Expr], span: Span) -> Type {
        if args.len() != 1 {
            self.diags.push(Diag::error("E0706", "`cancelled` expects 1 argument(s): (read Cancel)", span));
            for a in args { self.check_expr(a, Use::Value); }
            return Type::bool();
        }
        let expected = Type::Borrow(Box::new(Type::Named("Cancel".to_string())));
        self.check_against(&args[0], &expected);
        Type::bool()
    }
    // ----- scope-tracking accessors over FnState -------------------------
    fn scope_enter(&mut self) {
        self.f.scope_depth += 1;
        self.f.scope_synth.push(Vec::new());
    }
    fn scope_exit(&mut self) {
        self.f.scope_depth -= 1;
    }
    fn scope_depth_get(&self) -> usize {
        self.f.scope_depth
    }
    fn scope_synth_pop(&mut self) -> Vec<String> {
        self.f.scope_synth.pop().unwrap_or_default()
    }
    fn scope_synth_push(&mut self, name: String) {
        if let Some(top) = self.f.scope_synth.last_mut() {
            top.push(name);
        }
    }
    fn fresh_scope_loan_name(&mut self) -> String {
        let n = self.f.synth_ctr;
        self.f.synth_ctr += 1;
        format!("__scope_loan{n}")
    }
    fn reanchor_scope_loan(&mut self, ids: &[usize], name: &str) {
        for &id in ids {
            self.f.loans[id].anchor = Anchor::Binding(name.to_string());
        }
    }
    fn env_len(&self) -> usize {
        self.f.env.len()
    }
}

/// Peel a leading `read`/`write`/paren off a spawn argument to reach the place or
/// value expression the gate reasons over.
fn spawn_inner(arg: &Expr) -> &Expr {
    match &arg.kind {
        ExprKind::Prefix { op: PrefixOp::Read | PrefixOp::Write, expr } => expr,
        ExprKind::Paren(i) => spawn_inner(i),
        _ => arg,
    }
}
