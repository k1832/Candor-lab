//! Expression and place type checking with mode-aware calls, intrinsics,
//! `unsafe`/rawptr gating, the alloc effect, and control-flow lowering into the
//! shared CFG (design 0001 §3.1, §4.2, §5, §6, §8.1, §8.2.1).

use crate::ast::*;
use crate::diag::Diag;
use crate::span::Span;
use crate::token::ScalarTy;
use crate::types::*;

use super::dataflow::{Access, LoanKind, Place, Proj, Term};
use super::patterns::{self, BindMode, HoldMode};
use super::{Checker, Use};

impl<'a> Checker<'a> {
    pub(super) fn check_expr(&mut self, e: &Expr, u: Use) -> Type {
        match &e.kind {
            ExprKind::Ident(_)
            | ExprKind::Field { .. }
            | ExprKind::Index { .. }
            | ExprKind::Prefix {
                op: PrefixOp::Deref,
                ..
            } => {
                let (t, place) = self.check_place(e);
                self.emit_place_action(&place, u, &t, e.span);
                t
            }
            ExprKind::Paren(inner) => self.check_expr(inner, u),
            ExprKind::OutArg(inner) => {
                self.diags.push(
                    Diag::error(
                        "E0308",
                        "`out` marker is only valid on an out-mode call argument",
                        e.span,
                    )
                    .with_note("write `out place` only where the parameter is an out-mode parameter (§3.1)", None),
                );
                self.check_expr(inner, u)
            }
            ExprKind::Prefix {
                op: PrefixOp::Read,
                expr,
            } => {
                let t = self.check_expr(expr, Use::BorrowShared);
                Type::Borrow(Box::new(t))
            }
            ExprKind::Prefix {
                op: PrefixOp::Write,
                expr,
            } => {
                self.reject_static_mutation(expr, "`write`-borrow", e.span);
                self.reject_write_through_shared(expr, "take a `write`-borrow", e.span);
                let t = self.check_expr(expr, Use::BorrowExcl);
                Type::BorrowMut(Box::new(t))
            }
            ExprKind::Prefix {
                op: PrefixOp::Clone,
                expr,
            } => {
                let t = self.check_expr(expr, Use::ReadOnly);
                if bears_box(&t, self.items) {
                    self.note_alloc(e.span, "`clone` of a value that owns a `Box` allocates (§6.3)");
                }
                t
            }
            ExprKind::IntLit { suffix, .. } => match suffix {
                Some(s) => Type::Scalar(*s),
                None => Type::IntLit,
            },
            ExprKind::StrLit(_) => Type::Slice(Box::new(Type::Scalar(ScalarTy::U8))),
            ExprKind::BoolLit(_) => Type::bool(),
            ExprKind::Unary { op, expr } => self.check_unary(*op, expr),
            ExprKind::Binary { op, lhs, rhs } => self.check_binary(*op, lhs, rhs, e.span),
            ExprKind::Conv { ty, expr } => self.check_conv(ty, expr, e.span),
            ExprKind::Call { callee, args } => self.check_call(callee, args, e.span),
            ExprKind::StructLit { name, fields } => self.check_struct_lit(name, fields, e.span),
            ExprKind::EnumCtor {
                enum_name,
                variant,
                args,
            } => self.check_enum_ctor(enum_name, variant, args, e.span),
            ExprKind::ArrayLit(elems) => self.check_array_lit(elems),
            ExprKind::ArrayRepeat { value, size } => {
                let t = self.check_expr(value, Use::Value);
                let _ = self.check_expr(size, Use::Value);
                Type::Array(Box::new(t), ArrayLen::Unknown)
            }
            ExprKind::CastPtr { ty, arg } => {
                self.require_unsafe(e.span, "cast_ptr");
                self.check_expr(arg, Use::Value);
                Type::RawPtr(Box::new(self.resolve_ty(ty)))
            }
            ExprKind::AddrToPtr { ty, arg } => {
                self.require_unsafe(e.span, "addr_to_ptr");
                let t = self.check_expr(arg, Use::Value);
                self.expect_integer(&t, arg.span);
                Type::RawPtr(Box::new(self.resolve_ty(ty)))
            }
            ExprKind::PtrNull { ty } => {
                self.require_unsafe(e.span, "ptr_null");
                Type::RawPtr(Box::new(self.resolve_ty(ty)))
            }
            ExprKind::Offsetof { ty, .. } => {
                let _ = self.resolve_ty(ty);
                Type::usize()
            }
            ExprKind::FieldPtr { ptr, field } => self.check_field_ptr(ptr, field, e.span),
            ExprKind::Sizeof(ty) | ExprKind::Alignof(ty) => {
                let _ = self.resolve_ty(ty);
                Type::usize()
            }
            ExprKind::Block(b) => self.check_block_value(b),
            ExprKind::If {
                cond,
                then_blk,
                else_blk,
            } => self.check_if(cond, then_blk, else_blk.as_deref(), e.span),
            ExprKind::Match { scrutinee, arms } => self.check_match(scrutinee, arms, e.span),
            ExprKind::Loop(b) => self.check_loop(b, e.span),
            ExprKind::While { cond, body } => self.check_while(cond, body, e.span),
            ExprKind::Unsafe { justification, body } => {
                if justification.is_empty() {
                    self.diags.push(
                        Diag::error(
                            "E0502",
                            "an `unsafe` justification must be a non-empty string literal",
                            e.span,
                        )
                        .with_note("`unsafe` carries a stated justification (§4.1)", None),
                    );
                }
                let saved = self.f_in_unsafe();
                self.set_in_unsafe(true);
                self.check_block_stmts(body);
                self.set_in_unsafe(saved);
                Type::unit()
            }
            ExprKind::Wrapping(b) | ExprKind::Saturating(b) => {
                self.check_block_stmts(b);
                Type::unit()
            }
            ExprKind::Return(opt) => self.check_return(opt.as_deref(), e.span),
            ExprKind::Break => {
                self.do_break(e.span);
                Type::Never
            }
            ExprKind::Continue => {
                self.do_continue(e.span);
                Type::Never
            }
            ExprKind::Assert(cond) => {
                // An `assert` condition is a read-only contract clause (review #3).
                let saved = self.f_in_contract();
                self.set_in_contract(true);
                let t = self.check_expr(cond, Use::Value);
                self.set_in_contract(saved);
                self.expect_bool(&t, cond.span, "an `assert`");
                Type::unit()
            }
            ExprKind::Panic(msg) => {
                self.check_expr(msg, Use::Value);
                self.diverge();
                Type::Never
            }
            ExprKind::Result => {
                if self.f_in_ensures() {
                    self.ret_ty_clone()
                } else {
                    self.diags.push(
                        Diag::error(
                            "E0702",
                            "`result` may appear only inside an `ensures` clause",
                            e.span,
                        )
                        .with_note("`result` names the return value of the function (§7.3)", None),
                    );
                    Type::Error
                }
            }
        }
    }

    // ----- places ---------------------------------------------------------

    pub(super) fn check_place(&mut self, e: &Expr) -> (Type, Option<Place>) {
        match &e.kind {
            ExprKind::Paren(inner) => self.check_place(inner),
            ExprKind::Ident(name) => {
                if let Some(li) = self.lookup_local(name) {
                    (li.ty.clone(), Some(Place::local(name)))
                } else if let Some((t, _)) = self.items.statics.get(name) {
                    (t.clone(), None)
                } else if let Some(sig) = self.items.fns.get(name) {
                    (self.fnptr_of_sig(sig), None)
                } else {
                    self.diags.push(Diag::error(
                        "E0103",
                        format!("unknown name `{name}`"),
                        e.span,
                    ));
                    (Type::Error, None)
                }
            }
            ExprKind::Prefix {
                op: PrefixOp::Deref,
                expr,
            } => {
                let (t, p) = self.check_place(expr);
                let inner = match &t {
                    Type::Borrow(x) | Type::BorrowMut(x) | Type::Box(x) => (**x).clone(),
                    Type::Error => Type::Error,
                    other => {
                        self.diags.push(Diag::error(
                            "E0703",
                            format!("cannot `deref` a value of type `{}`", other.display()),
                            e.span,
                        ));
                        Type::Error
                    }
                };
                let place = p.map(|mut pl| {
                    pl.proj.push(Proj::Deref);
                    pl
                });
                (inner, place)
            }
            ExprKind::Field { base, field } => {
                let (bt, bp) = self.check_place(base);
                let (st, mut place) = self.autoderef(bt, bp);
                match &st {
                    Type::Named(n) => {
                        let fld = self
                            .items
                            .lookup_struct(n)
                            .and_then(|s| s.fields.iter().find(|(fn_, _)| fn_ == field).cloned());
                        match fld {
                            Some((_, fty)) => {
                                if let Some(p) = place.as_mut() {
                                    p.proj.push(Proj::Field(field.clone()));
                                }
                                (fty, place)
                            }
                            None => {
                                self.diags.push(Diag::error(
                                    "E0107",
                                    format!("type `{n}` has no field `{field}`"),
                                    e.span,
                                ));
                                (Type::Error, place)
                            }
                        }
                    }
                    Type::Error => (Type::Error, place),
                    other => {
                        self.diags.push(Diag::error(
                            "E0703",
                            format!("field access on non-struct type `{}`", other.display()),
                            e.span,
                        ));
                        (Type::Error, place)
                    }
                }
            }
            ExprKind::Index { base, index } => {
                let it = self.check_expr(index, Use::Value);
                self.expect_integer(&it, index.span);
                let (bt, bp) = self.check_place(base);
                let (st, mut place) = self.autoderef(bt, bp);
                let elem = match &st {
                    Type::Array(e, _) | Type::Slice(e) | Type::SliceMut(e) => (**e).clone(),
                    Type::Error => Type::Error,
                    other => {
                        self.diags.push(Diag::error(
                            "E0703",
                            format!("cannot index a value of type `{}`", other.display()),
                            e.span,
                        ));
                        Type::Error
                    }
                };
                if let Some(p) = place.as_mut() {
                    p.proj.push(Proj::Index);
                }
                (elem, place)
            }
            _ => {
                let t = self.check_expr(e, Use::Value);
                (t, None)
            }
        }
    }

    /// Peel `borrow`/`borrow_mut`/`Box` layers, extending the place with `deref`
    /// projections (making it opaque to field-granular move tracking).
    fn autoderef(&self, mut ty: Type, mut place: Option<Place>) -> (Type, Option<Place>) {
        loop {
            match ty {
                Type::Borrow(x) | Type::BorrowMut(x) | Type::Box(x) => {
                    ty = *x;
                    place = place.map(|mut p| {
                        p.proj.push(Proj::Deref);
                        p
                    });
                }
                other => return (other, place),
            }
        }
    }

    /// Reject a write or exclusive reborrow that passes through a SHARED borrow.
    /// Design 0001 §2.1/§2.2: `deref b = v` and `write (deref b)` require `b`
    /// exclusive; every `deref` on the path to a written place must peel a
    /// `write`-borrow (or a `Box`), never a `read`-borrow. The reviewer's XOR
    /// hole (0003 §0 2026-07-08) was this check missing — `check_place` peeled
    /// `Borrow`/`BorrowMut` identically.
    pub(super) fn reject_write_through_shared(&mut self, e: &Expr, action: &str, span: Span) {
        let (_, shared) = self.write_path_probe(e);
        if shared {
            self.e0809(format!("cannot {action} through a shared (`read`) borrow"), span);
        }
    }

    /// Emit the shared-borrow write/reborrow error (§2.1/§2.2).
    pub(super) fn e0809(&mut self, msg: impl Into<String>, span: Span) {
        self.diags.push(
            Diag::error("E0809", msg, span).with_note(
                "a deref-write or exclusive reborrow requires an exclusive (`write`) borrow; a shared (`read`) borrow may only be read or re-shared (§2.1/§2.2)",
                None,
            ),
        );
    }

    /// Non-emitting probe for `reject_write_through_shared`: returns the type the
    /// place `e` holds and whether any `deref` on the path to it peels a SHARED
    /// borrow. Autoderef steps into a field/index of a borrow count as derefs on
    /// the path; peeling an exclusive borrow or a `Box` does not set the flag.
    fn write_path_probe(&self, e: &Expr) -> (Type, bool) {
        match &e.kind {
            ExprKind::Paren(i) => self.write_path_probe(i),
            ExprKind::Ident(name) => {
                let t = if let Some(li) = self.lookup_local(name) {
                    li.ty.clone()
                } else if let Some((t, _)) = self.items.statics.get(name) {
                    t.clone()
                } else {
                    Type::Error
                };
                (t, false)
            }
            ExprKind::Prefix {
                op: PrefixOp::Deref,
                expr,
            } => {
                let (t, mut shared) = self.write_path_probe(expr);
                let inner = match t {
                    Type::Borrow(x) => {
                        shared = true;
                        *x
                    }
                    Type::BorrowMut(x) | Type::Box(x) => *x,
                    _ => Type::Error,
                };
                (inner, shared)
            }
            ExprKind::Field { base, field } => {
                let (bt, mut shared) = self.write_path_probe(base);
                let st = self.autoderef_probe(bt, &mut shared);
                let fty = match &st {
                    Type::Named(n) => self
                        .items
                        .lookup_struct(n)
                        .and_then(|s| s.fields.iter().find(|(f, _)| f == field).map(|(_, t)| t.clone()))
                        .unwrap_or(Type::Error),
                    _ => Type::Error,
                };
                (fty, shared)
            }
            ExprKind::Index { base, .. } => {
                let (bt, mut shared) = self.write_path_probe(base);
                let st = self.autoderef_probe(bt, &mut shared);
                let elem = match st {
                    // Owned array: indexing does not cross a borrow (unaffected).
                    Type::Array(e, _) => *e,
                    // Shared slice: indexing it is a write through a shared borrow
                    // (§5.2; retest 2026-07-08, finding 2) — `s[i]=v` rejects.
                    Type::Slice(e) => {
                        shared = true;
                        *e
                    }
                    // Exclusive slice: an ordinary exclusive write path (legal).
                    Type::SliceMut(e) => *e,
                    _ => Type::Error,
                };
                (elem, shared)
            }
            _ => (Type::Error, false),
        }
    }

    /// Peel borrow/box layers off a base type for the write-path probe, setting
    /// `shared` if any peeled layer is a SHARED borrow.
    fn autoderef_probe(&self, mut ty: Type, shared: &mut bool) -> Type {
        loop {
            match ty {
                Type::Borrow(x) => {
                    *shared = true;
                    ty = *x;
                }
                Type::BorrowMut(x) | Type::Box(x) => ty = *x,
                // A slice IS a borrow (§5.2): a shared `slice` on a write path is
                // a shared borrow of the run; `slice_mut` is exclusive. Stop at
                // the slice so the `Index` caller peels the element (retest
                // 2026-07-08, finding 2). Arrays are owned and unaffected.
                Type::Slice(x) => {
                    *shared = true;
                    return Type::Slice(x);
                }
                other => return other,
            }
        }
    }

    pub(super) fn fnptr_of_sig(&self, sig: &crate::resolve::FnSig) -> Type {
        Type::FnPtr(crate::types::FnPtrTy {
            params: sig
                .params
                .iter()
                .map(|p| (p.mode, p.decl_ty.clone()))
                .collect(),
            alloc: sig.alloc,
            ret: Box::new(sig.ret.clone()),
        })
    }

    // ----- operators ------------------------------------------------------

    fn check_unary(&mut self, op: UnOp, expr: &Expr) -> Type {
        let t = self.check_expr(expr, Use::Value);
        match op {
            UnOp::Neg => {
                self.expect_integer(&t, expr.span);
                t
            }
            UnOp::Not => {
                self.expect_bool(&t, expr.span, "operand of `!`");
                Type::bool()
            }
        }
    }

    fn check_binary(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr, span: Span) -> Type {
        let l = self.check_expr(lhs, Use::Value);
        let r = self.check_expr(rhs, Use::Value);
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => {
                self.unify_int(&l, &r, span)
            }
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                self.unify_int(&l, &r, span);
                Type::bool()
            }
            BinOp::Eq | BinOp::Ne => {
                if !matches!(l, Type::Error)
                    && !matches!(r, Type::Error)
                    && !assignable(&l, &r)
                    && !assignable(&r, &l)
                {
                    self.diags.push(Diag::error(
                        "E0703",
                        format!("cannot compare `{}` with `{}`", l.display(), r.display()),
                        span,
                    ));
                }
                Type::bool()
            }
            BinOp::And | BinOp::Or => {
                self.expect_bool(&l, lhs.span, "operand of a boolean operator");
                self.expect_bool(&r, rhs.span, "operand of a boolean operator");
                Type::bool()
            }
        }
    }

    fn unify_int(&mut self, l: &Type, r: &Type, span: Span) -> Type {
        if matches!(l, Type::Error) || matches!(r, Type::Error) {
            return Type::Error;
        }
        if !l.is_integer() || !r.is_integer() {
            self.diags.push(Diag::error(
                "E0703",
                format!(
                    "arithmetic requires integer operands, found `{}` and `{}`",
                    l.display(),
                    r.display()
                ),
                span,
            ));
            return Type::Error;
        }
        match (l, r) {
            (Type::IntLit, Type::IntLit) => Type::IntLit,
            (Type::IntLit, t) | (t, Type::IntLit) => t.clone(),
            (a, b) if a == b => a.clone(),
            _ => {
                self.diags.push(Diag::error(
                    "E0703",
                    format!("mismatched integer types `{}` and `{}`", l.display(), r.display()),
                    span,
                ));
                l.clone()
            }
        }
    }

    fn check_conv(&mut self, ty: &Ty, expr: &Expr, span: Span) -> Type {
        let src = self.check_expr(expr, Use::Value);
        let target = self.resolve_ty(ty);
        if !matches!(src, Type::Error) && !src.is_integer() {
            self.diags.push(
                Diag::error(
                    "E0701",
                    format!("`conv` operand must be an integer, found `{}`", src.display()),
                    span,
                )
                .with_note("`conv` converts between integer types only (§8.1)", None),
            );
        }
        if !matches!(target, Type::Error) && !target.is_integer() {
            self.diags.push(
                Diag::error(
                    "E0701",
                    format!("`conv` target must be an integer type, found `{}`", target.display()),
                    span,
                )
                .with_note("pointer retyping uses `cast_ptr`/`addr_to_ptr` (§4.2)", None),
            );
        }
        target
    }

    // ----- struct / enum construction -------------------------------------

    fn check_struct_lit(&mut self, name: &str, fields: &[FieldInit], span: Span) -> Type {
        let sinfo = self.items.lookup_struct(name).cloned();
        match sinfo {
            Some(s) => {
                for fi in fields {
                    match s.fields.iter().find(|(fn_, _)| fn_ == &fi.name) {
                        Some((_, fty)) => {
                            self.check_against(&fi.value, fty);
                        }
                        None => {
                            self.diags.push(Diag::error(
                                "E0107",
                                format!("type `{name}` has no field `{}`", fi.name),
                                fi.span,
                            ));
                            self.check_expr(&fi.value, Use::Value);
                        }
                    }
                }
                Type::Named(name.to_string())
            }
            None => {
                self.diags.push(Diag::error(
                    "E0102",
                    format!("unknown struct type `{name}`"),
                    span,
                ));
                for fi in fields {
                    self.check_expr(&fi.value, Use::Value);
                }
                Type::Error
            }
        }
    }

    fn check_enum_ctor(&mut self, enum_name: &str, variant: &str, args: &[Expr], span: Span) -> Type {
        if enum_name == "BoxResult" {
            return match variant {
                "boxed" => {
                    let t = if args.len() == 1 {
                        self.check_expr(&args[0], Use::Value)
                    } else {
                        Type::Error
                    };
                    match t {
                        Type::Box(inner) => Type::BoxResult(inner),
                        Type::Error => Type::BoxResult(Box::new(Type::Error)),
                        other => {
                            self.diags.push(Diag::error(
                                "E0703",
                                format!("`BoxResult::boxed` expects a `Box`, found `{}`", other.display()),
                                span,
                            ));
                            Type::BoxResult(Box::new(Type::Error))
                        }
                    }
                }
                "oom" => Type::BoxResult(Box::new(Type::Error)),
                _ => {
                    self.diags.push(Diag::error(
                        "E0108",
                        format!("`BoxResult` has no variant `{variant}`"),
                        span,
                    ));
                    Type::Error
                }
            };
        }
        let einfo = self.items.lookup_enum(enum_name).cloned();
        match einfo {
            Some(e) => match e.variants.iter().find(|v| v.name == variant) {
                Some(v) => {
                    if args.len() != v.payload.len() {
                        self.diags.push(Diag::error(
                            "E0605",
                            format!(
                                "variant `{}::{}` expects {} payload(s), found {}",
                                enum_name,
                                variant,
                                v.payload.len(),
                                args.len()
                            ),
                            span,
                        ));
                    }
                    for (a, pty) in args.iter().zip(&v.payload) {
                        self.check_against(a, pty);
                    }
                    Type::Named(enum_name.to_string())
                }
                None => {
                    self.diags.push(Diag::error(
                        "E0108",
                        format!("enum `{enum_name}` has no variant `{variant}`"),
                        span,
                    ));
                    for a in args {
                        self.check_expr(a, Use::Value);
                    }
                    Type::Error
                }
            },
            None => {
                self.diags.push(Diag::error(
                    "E0102",
                    format!("unknown enum type `{enum_name}`"),
                    span,
                ));
                for a in args {
                    self.check_expr(a, Use::Value);
                }
                Type::Error
            }
        }
    }

    fn check_array_lit(&mut self, elems: &[Expr]) -> Type {
        let mut ty = Type::Error;
        for (i, el) in elems.iter().enumerate() {
            let t = self.check_expr(el, Use::Value);
            if i == 0 {
                ty = t;
            }
        }
        Type::Array(Box::new(ty), ArrayLen::Lit(elems.len() as u64))
    }

    // ----- helpers used by control flow (in this file) --------------------

    /// Check that a returned borrow's provenance is a region-legal input, not a
    /// local (design §3.3): a borrow may not outlive the body it was born in.
    ///
    /// Provenance is **total** (finding 1 of 2026-07-07): every borrow-producing
    /// shape is either recognized and checked, recursed into (a `match` arm, an
    /// `if` branch), or — if unrecognized — REJECTED (`None` ⇒ E0806), never
    /// skipped. Skipping let a borrow of a local laundered through a `match`
    /// escape the region check and dangle.
    fn check_return_provenance(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::Paren(i) => self.check_return_provenance(i),
            ExprKind::Match { arms, .. } => {
                // Every arm's tail value must independently derive from a legal
                // input region.
                for arm in arms {
                    self.check_return_provenance(&arm.body);
                }
            }
            ExprKind::If {
                then_blk,
                else_blk,
                ..
            } => {
                // A block evaluates to unit/never (below), so the `then` block
                // never carries a borrow tail; the borrow value of an `if` is the
                // `else` expression. Recurse into whatever can carry it.
                self.check_return_block_tail(then_blk);
                if let Some(els) = else_blk {
                    self.check_return_provenance(els);
                }
            }
            ExprKind::Block(b) => self.check_return_block_tail(b),
            // Diverging shapes yield no borrow value to escape the body.
            ExprKind::Return(_) | ExprKind::Break | ExprKind::Continue | ExprKind::Panic(_) => {}
            _ => self.check_return_root(e),
        }
    }

    /// A block evaluates to unit or never in this prototype (`check_block_value`
    /// returns `unit`/`never`; a `Block` has no tail expression), so it can never
    /// produce a borrow value — there is no borrow provenance to check. Present so
    /// the walk stays *total*: block tails are handled (as carrying no borrow),
    /// not silently skipped (finding 1 of 2026-07-07; the reviewer's note that
    /// blocks evaluate to unit/never, verified against `check_block_value`).
    fn check_return_block_tail(&mut self, _b: &Block) {}

    /// The base case of the total provenance walk: an atomic borrow-producing
    /// expression. Its provenance root must be a region-legal input parameter;
    /// an unrecognized shape (`None`) is rejected E0806, never skipped.
    fn check_return_root(&mut self, e: &Expr) {
        let root = match self.borrow_provenance(e) {
            None => {
                self.diags.push(
                    Diag::error(
                        "E0806",
                        "returned borrow does not provably derive from an input; it may borrow a local that does not outlive the body",
                        e.span,
                    )
                    .with_note("return an owned value, or a borrow whose provenance is a `read`/`write`/slice parameter (§3.3)", None),
                );
                return;
            }
            Some(r) => r,
        };
        let found = self
            .f
            .sig_params
            .iter()
            .find(|(n, _, _)| *n == root)
            .cloned();
        match found {
            None => self.diags.push(
                Diag::error(
                    "E0806",
                    format!("returned borrow of local `{root}` does not live long enough"),
                    e.span,
                )
                .with_note("a borrow may not outlive the body it was born in; return an owned value or borrow an input (§3.3)", None),
            ),
            Some((_, is_bp, region)) => {
                if !is_bp {
                    self.diags.push(
                        Diag::error(
                            "E0806",
                            format!("returned borrow derives from owned parameter `{root}`, which does not outlive the body"),
                            e.span,
                        )
                        .with_note("borrow-return provenance must be a `read`/`write`/slice parameter (§3.3)", None),
                    );
                } else if let Some(r) = self.f.ret_region.clone() {
                    if region.as_deref() != Some(r.as_str()) {
                        self.diags.push(
                            Diag::error(
                                "E0808",
                                format!("returned borrow is tagged region `{r}` but derives from `{root}`, which is not in that region"),
                                e.span,
                            )
                            .with_note("the returned borrow must derive from the parameter carrying the return's region (§3.3)", None),
                        );
                    }
                }
            }
        }
    }

    /// The input root a borrow value ultimately derives from (design §3.3),
    /// computed purely from the expression shape.
    fn borrow_provenance(&self, e: &Expr) -> Option<String> {
        match &e.kind {
            ExprKind::Paren(i) => self.borrow_provenance(i),
            ExprKind::Prefix {
                op: PrefixOp::Read | PrefixOp::Write,
                expr,
            } => expr_place_root(expr),
            ExprKind::Ident(n) => {
                if self.lookup_local(n).map(|l| l.ty.is_borrow_kind()).unwrap_or(false) {
                    Some(n.clone())
                } else {
                    None
                }
            }
            ExprKind::Call { callee, args } => {
                if let ExprKind::Ident(name) = &callee.kind {
                    match name.as_str() {
                        "slice_of" | "slice_of_mut" => args.first().and_then(expr_place_root),
                        "subslice" => args.first().and_then(|a| self.borrow_provenance(a)),
                        _ => {
                            if let Some(sig) = self.items.fns.get(name) {
                                if matches!(sig.ret, Type::Borrow(_) | Type::BorrowMut(_)) {
                                    let src = region_source_indices(sig);
                                    return src
                                        .first()
                                        .and_then(|&i| args.get(i))
                                        .and_then(|a| self.borrow_provenance(a));
                                }
                            }
                            None
                        }
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn f_in_unsafe(&self) -> bool {
        self.in_unsafe_get()
    }
    fn f_in_ensures(&self) -> bool {
        self.in_ensures_get()
    }
}

impl<'a> Checker<'a> {
    pub(super) fn expect_integer(&mut self, t: &Type, span: Span) {
        if !matches!(t, Type::Error | Type::Never) && !t.is_integer() {
            self.diags.push(Diag::error(
                "E0703",
                format!("expected an integer, found `{}`", t.display()),
                span,
            ));
        }
    }

    /// `field_ptr(p, f)` (design 0004): SAFE typed field projection. `p` must be
    /// `rawptr StructT` for a compiler-known struct and `f` a statically known
    /// field of `StructT`; otherwise E0510. Result is `rawptr FieldT`. Not gated
    /// by E0501 — it joins `offsetof`/`is_null` on the safe side (§4.2, 0003 §2.5).
    fn check_field_ptr(&mut self, ptr: &Expr, field: &str, span: Span) -> Type {
        let pt = self.check_expr(ptr, Use::Value);
        match &pt {
            Type::Error => Type::Error,
            Type::RawPtr(inner) => match &**inner {
                Type::Error => Type::Error,
                Type::Named(n) => {
                    let fld = self
                        .items
                        .lookup_struct(n)
                        .and_then(|s| s.fields.iter().find(|(fn_, _)| fn_ == field).cloned());
                    match fld {
                        Some((_, fty)) => Type::RawPtr(Box::new(fty)),
                        None => {
                            self.diags.push(
                                Diag::error(
                                    "E0510",
                                    format!("`{n}` has no field `{field}` for `field_ptr`"),
                                    span,
                                )
                                .with_note(
                                    "`field_ptr(p, f)` needs `f` a statically known field of the struct `p` points at (0004)",
                                    None,
                                ),
                            );
                            Type::Error
                        }
                    }
                }
                other => {
                    self.diags.push(
                        Diag::error(
                            "E0510",
                            format!(
                                "`field_ptr` needs `rawptr` of a struct, found `rawptr {}`",
                                other.display()
                            ),
                            span,
                        )
                        .with_note("`field_ptr(p, f)` projects a field of a struct pointee (0004)", None),
                    );
                    Type::Error
                }
            },
            other => {
                self.diags.push(
                    Diag::error(
                        "E0510",
                        format!("`field_ptr` needs a `rawptr StructT` operand, found `{}`", other.display()),
                        span,
                    )
                    .with_note("`field_ptr(p, f)` projects a field of a struct pointee (0004)", None),
                );
                Type::Error
            }
        }
    }

    // ----- calls ----------------------------------------------------------

    fn check_call(&mut self, callee: &Expr, args: &[Expr], span: Span) -> Type {
        if let ExprKind::Ident(name) = &callee.kind {
            if let Some(t) = self.check_builtin(name, args, span) {
                return t;
            }
            if let Some(sig) = self.items.fns.get(name).cloned() {
                return self.check_user_call(&sig, args, span);
            }
        }
        // Indirect call: the callee is a fn-pointer value (§6.1).
        let ct = self.check_expr(callee, Use::Value);
        match ct {
            Type::FnPtr(fp) => {
                let n = fp.params.len().min(args.len());
                let mut group: Vec<usize> = Vec::new();
                for ((mode, pty), a) in fp.params.iter().zip(args) {
                    self.clear_carried();
                    self.check_arg_mode(*mode, pty, a);
                    group.extend(self.take_carried());
                }
                for a in args.iter().skip(n) {
                    self.check_expr(a, Use::Value);
                }
                self.push_call_group(group);
                self.reject_consuming_call(fp.params.iter().map(|(m, t)| (*m, t)), span);
                if fp.alloc {
                    self.note_alloc(span, "indirect call through an `alloc` fn-pointer (§6.1)");
                }
                self.clear_carried();
                *fp.ret
            }
            Type::Error => Type::Error,
            other => {
                self.diags.push(Diag::error(
                    "E0704",
                    format!("`{}` is not callable", other.display()),
                    span,
                ));
                Type::Error
            }
        }
    }

    /// Apply the read-only contract rule to a call site (review #3): if any
    /// parameter is passed by `write`, `out`, or `take` of a non-`copy` type, the
    /// call consumes or mutates its argument and is rejected inside a contract
    /// clause. `read`-mode and copy-`take` arguments are fine.
    fn reject_consuming_call<'p>(
        &mut self,
        params: impl Iterator<Item = (ParamMode, &'p Type)>,
        span: Span,
    ) {
        if !self.f.in_contract {
            return;
        }
        for (mode, ty) in params {
            let consumes = match mode {
                ParamMode::Write | ParamMode::Out => true,
                ParamMode::Take => !is_copy(ty, self.items),
                ParamMode::Read => false,
            };
            if consumes {
                self.reject_in_contract(span, "a call that consumes or mutates an argument");
                return;
            }
        }
    }

    fn check_user_call(&mut self, sig: &crate::resolve::FnSig, args: &[Expr], span: Span) -> Type {
        if args.len() != sig.params.len() {
            self.diags.push(Diag::error(
                "E0706",
                format!(
                    "function `{}` expects {} argument(s), found {}",
                    sig.name,
                    sig.params.len(),
                    args.len()
                ),
                span,
            ));
        }
        // Evaluate each argument, capturing the loan(s) it contributes (§3.1).
        let mut per_arg: Vec<Vec<usize>> = Vec::new();
        let mut per_prov: Vec<Option<String>> = Vec::new();
        for (p, a) in sig.params.iter().zip(args) {
            self.clear_carried();
            self.check_arg_mode(p.mode, &p.decl_ty, a);
            per_prov.push(self.f.carried_prov.clone());
            per_arg.push(self.take_carried());
        }
        let group: Vec<usize> = per_arg.iter().flatten().copied().collect();
        self.push_call_group(group);
        // A contract clause is read-only: a call that takes any argument by
        // `take` (non-copy), `write`, or `out` consumes or mutates it and is
        // rejected (review #3, 2026-07-07). `read`-mode and copy-`take` are fine.
        self.reject_consuming_call(sig.params.iter().map(|p| (p.mode, &p.decl_ty)), span);
        if sig.alloc {
            self.note_alloc(span, format!("call to `alloc` function `{}` (§6.3)", sig.name));
        }
        // Return-borrow extension: the returned borrow keeps the loan(s) on the
        // argument(s) tagged with the return's region alive (§3.1/§3.3).
        if matches!(sig.ret, Type::Borrow(_) | Type::BorrowMut(_)) {
            let src = region_source_indices(sig);
            let mut ids = Vec::new();
            let mut prov = None;
            for i in src {
                ids.extend(per_arg[i].iter().copied());
                if prov.is_none() {
                    prov = per_prov[i].clone();
                }
            }
            self.set_carried(ids, prov);
        } else {
            self.clear_carried();
        }
        sig.ret.clone()
    }

    /// Design 0005 implicit call-site reborrow. If `inner` is a place that
    /// already denotes a borrow (a borrow-typed local, or a chained `deref b`)
    /// passed to a `read`/`write`-mode parameter, desugar it to the explicit
    /// reborrow node `read (deref inner)` / `write (deref inner)` BEFORE loan
    /// analysis, so the inserted node is indistinguishable from the hand-written
    /// form to Stage 3. Bare `b` to such a parameter is a reborrow, never a move.
    /// Non-place borrow arguments (call results) and owned places return `None`
    /// and keep their existing treatment; `take`-mode never reaches here.
    /// `write`-param + shared source desugars to `write (deref b)`, which the
    /// existing machinery rejects as "cannot reborrow exclusive from shared".
    fn reborrow_desugar(&self, inner: &Expr, op: PrefixOp) -> Option<Expr> {
        if !matches!(
            self.place_borrow_ty(inner),
            Some(Type::Borrow(_) | Type::BorrowMut(_))
        ) {
            return None;
        }
        let deref = Expr {
            kind: ExprKind::Prefix {
                op: PrefixOp::Deref,
                expr: Box::new(inner.clone()),
            },
            span: inner.span,
        };
        Some(Expr {
            kind: ExprKind::Prefix {
                op,
                expr: Box::new(deref),
            },
            span: inner.span,
        })
    }

    /// Non-emitting probe of the type a place expression holds, for the
    /// borrow-denoting-place test above. Only the place shapes that can denote a
    /// borrow are recognized (identifier, `deref` chain, `paren`); §3.4 bans
    /// borrow-typed struct/enum fields, so there is no `p.f` case.
    fn place_borrow_ty(&self, e: &Expr) -> Option<Type> {
        match &e.kind {
            ExprKind::Paren(i) => self.place_borrow_ty(i),
            ExprKind::Ident(name) => self.lookup_local(name).map(|li| li.ty.clone()),
            ExprKind::Prefix {
                op: PrefixOp::Deref,
                expr,
            } => match self.place_borrow_ty(expr)? {
                Type::Borrow(x) | Type::BorrowMut(x) | Type::Box(x) => Some(*x),
                _ => None,
            },
            _ => None,
        }
    }

    fn check_arg_mode(&mut self, mode: ParamMode, decl_ty: &Type, arg: &Expr) {
        let out_marked = matches!(&arg.kind, ExprKind::OutArg(_));
        let inner: &Expr = match &arg.kind {
            ExprKind::OutArg(i) => i,
            _ => arg,
        };
        if mode != ParamMode::Out && out_marked {
            self.diags.push(
                Diag::error(
                    "E0308",
                    "`out` marker on an argument to a non-out parameter",
                    arg.span,
                )
                .with_note("the `out` marker is written only for out-mode parameters (§3.1)", None),
            );
        }
        match mode {
            ParamMode::Take => {
                self.check_against(inner, decl_ty);
            }
            ParamMode::Read => {
                let expected = Type::Borrow(Box::new(decl_ty.clone()));
                match self.reborrow_desugar(inner, PrefixOp::Read) {
                    Some(node) => self.check_against(&node, &expected),
                    None => self.check_against(inner, &expected),
                };
            }
            ParamMode::Write => {
                // Design 0005 shareability gate (0001 §2.1/§2.2): a held SHARED
                // borrow passed to a write-mode parameter would reborrow
                // exclusively from shared — rejected E0809, never desugared.
                if matches!(self.place_borrow_ty(inner), Some(Type::Borrow(_))) {
                    self.e0809(
                        "cannot reborrow an exclusive (`write`) borrow from a shared (`read`) borrow passed to a `write`-mode parameter",
                        inner.span,
                    );
                    return;
                }
                let expected = Type::BorrowMut(Box::new(decl_ty.clone()));
                match self.reborrow_desugar(inner, PrefixOp::Write) {
                    Some(node) => self.check_against(&node, &expected),
                    None => self.check_against(inner, &expected),
                };
            }
            ParamMode::Out => {
                if !out_marked {
                    self.diags.push(
                        Diag::error(
                            "E0307",
                            "an out-mode argument must carry the `out` marker",
                            arg.span,
                        )
                        .with_note("spell it `out place`: a caller-owned slot the callee fills must be visible at the call site (§3.1)", None),
                    );
                }
                self.check_out_arg(inner, decl_ty);
            }
        }
    }

    fn check_out_arg(&mut self, arg: &Expr, expected: &Type) {
        self.reject_static_mutation(arg, "pass as `out`", arg.span);
        // An `out` argument is a write to the slot (§3.1), so it may not route
        // through a shared (`read`) deref — same gate as an assignment or an
        // exclusive reborrow (retest 2026-07-08, finding 1).
        self.reject_write_through_shared(arg, "pass as `out`", arg.span);
        let (t, place) = self.check_place(arg);
        match &place {
            Some(p) => {
                self.emit(
                    &place,
                    Access::OutArg {
                        needs_drop: needs_drop(&t, self.items),
                        box_paths: box_subpaths(&t, self.items),
                    },
                    arg.span,
                );
                // `out place` is an exclusive loan on the slot for the call (§3.1).
                let id = self.new_temp_loan(p.canonical(), LoanKind::Excl, arg.span);
                self.set_carried(vec![id], Some(p.canonical().root));
            }
            None => self.diags.push(
                Diag::error("E0705", "an `out` argument must be a place", arg.span)
                    .with_note("pass a local or field the caller owns (§3.1)", None),
            ),
        }
        if !matches!(t, Type::Error) && !assignable(&t, expected) {
            self.diags.push(Diag::error(
                "E0703",
                format!(
                    "`out` argument type `{}` does not match parameter `{}`",
                    t.display(),
                    expected.display()
                ),
                arg.span,
            ));
        }
    }

    // ----- builtins (spelled as ordinary calls, design 0002 §0.5) ---------

    fn check_builtin(&mut self, name: &str, args: &[Expr], span: Span) -> Option<Type> {
        let t = match name {
            "box" => {
                if args.len() == 2 {
                    self.check_expr(&args[0], Use::Value);
                    let v = self.check_expr(&args[1], Use::Value);
                    self.note_alloc(span, "call to `box` allocates (§6.3)");
                    Type::BoxResult(Box::new(v))
                } else {
                    self.arity(span, "box", 2);
                    Type::Error
                }
            }
            "unbox" => {
                let b = self.arg0(args, span, "unbox");
                // `unbox` frees the box storage through the stored handle — the
                // free side of the alloc effect (finding 4; §6.2/§6.3).
                self.note_alloc(span, "`unbox` frees the box storage (§6.2/§6.3)");
                match b {
                    Type::Box(inner) => *inner,
                    Type::Error => Type::Error,
                    other => {
                        self.mismatch(span, "unbox", "Box T", &other);
                        Type::Error
                    }
                }
            }
            "ptr_read" => {
                self.require_unsafe(span, "ptr_read");
                match self.arg0(args, span, "ptr_read") {
                    Type::RawPtr(inner) => *inner,
                    Type::Error => Type::Error,
                    other => {
                        self.mismatch(span, "ptr_read", "rawptr T", &other);
                        Type::Error
                    }
                }
            }
            "ptr_write" => {
                self.require_unsafe(span, "ptr_write");
                if args.len() == 2 {
                    self.check_expr(&args[0], Use::Value);
                    self.check_expr(&args[1], Use::Value);
                } else {
                    self.arity(span, "ptr_write", 2);
                }
                Type::unit()
            }
            "ptr_offset" => {
                self.require_unsafe(span, "ptr_offset");
                if args.len() == 2 {
                    let p = self.check_expr(&args[0], Use::Value);
                    let n = self.check_expr(&args[1], Use::Value);
                    self.expect_integer(&n, args[1].span);
                    p
                } else {
                    self.arity(span, "ptr_offset", 2);
                    Type::Error
                }
            }
            "is_null" => {
                self.arg0(args, span, "is_null");
                Type::bool()
            }
            "ptr_to_addr" => {
                self.require_unsafe(span, "ptr_to_addr");
                self.arg0(args, span, "ptr_to_addr");
                Type::usize()
            }
            "addr_of" | "addr_of_mut" => {
                self.require_unsafe(span, name);
                if args.len() == 1 {
                    let (t, place) = self.check_place(&args[0]);
                    self.emit_place_action(&place, Use::ReadOnly, &t, args[0].span);
                    Type::RawPtr(Box::new(t))
                } else {
                    self.arity(span, name, 1);
                    Type::Error
                }
            }
            "slice_of" => {
                if args.len() == 1 {
                    let (t, place) = self.check_place(&args[0]);
                    self.emit_place_action(&place, Use::BorrowShared, &t, args[0].span);
                    match t {
                        Type::Array(e, _) | Type::Slice(e) | Type::SliceMut(e) => {
                            Type::Slice(e)
                        }
                        Type::Error => Type::Error,
                        other => {
                            self.mismatch(span, "slice_of", "an array", &other);
                            Type::Error
                        }
                    }
                } else {
                    self.arity(span, "slice_of", 1);
                    Type::Error
                }
            }
            "slice_of_mut" => {
                if args.len() == 1 {
                    // `slice_of_mut` is an exclusive borrow of the run, so it may
                    // not reborrow exclusively from behind a shared deref (§5.2;
                    // retest 2026-07-08, finding 3) — same gate as a write.
                    self.reject_write_through_shared(
                        &args[0],
                        "take an exclusive (`slice_mut`) slice",
                        args[0].span,
                    );
                    let (t, place) = self.check_place(&args[0]);
                    self.emit_place_action(&place, Use::BorrowExcl, &t, args[0].span);
                    match t {
                        Type::Array(e, _) | Type::Slice(e) | Type::SliceMut(e) => Type::SliceMut(e),
                        Type::Error => Type::Error,
                        other => {
                            self.mismatch(span, "slice_of_mut", "an array", &other);
                            Type::Error
                        }
                    }
                } else {
                    self.arity(span, "slice_of_mut", 1);
                    Type::Error
                }
            }
            "subslice" => {
                if args.len() == 3 {
                    let s = self.check_expr(&args[0], Use::Value);
                    let lo = self.check_expr(&args[1], Use::Value);
                    let hi = self.check_expr(&args[2], Use::Value);
                    self.expect_integer(&lo, args[1].span);
                    self.expect_integer(&hi, args[2].span);
                    match s {
                        Type::Slice(_) | Type::SliceMut(_) => s,
                        Type::Error => Type::Error,
                        other => {
                            self.mismatch(span, "subslice", "a slice", &other);
                            Type::Error
                        }
                    }
                } else {
                    self.arity(span, "subslice", 3);
                    Type::Error
                }
            }
            "len" => {
                self.arg0(args, span, "len");
                Type::usize()
            }
            // Prototype observability intrinsic (Stage 4): appends an i64 to the
            // interpreter's trace log so drop-order tests can observe destruction.
            "trace" => {
                let t = self.arg0(args, span, "trace");
                self.expect_integer(&t, span);
                Type::unit()
            }
            _ => return None,
        };
        Some(t)
    }

    fn arg0(&mut self, args: &[Expr], span: Span, name: &str) -> Type {
        if args.len() == 1 {
            self.check_expr(&args[0], Use::Value)
        } else {
            self.arity(span, name, 1);
            for a in args {
                self.check_expr(a, Use::Value);
            }
            Type::Error
        }
    }

    fn arity(&mut self, span: Span, name: &str, n: usize) {
        self.diags.push(Diag::error(
            "E0706",
            format!("`{name}` expects {n} argument(s)"),
            span,
        ));
    }

    fn mismatch(&mut self, span: Span, name: &str, want: &str, got: &Type) {
        self.diags.push(Diag::error(
            "E0703",
            format!("`{name}` expects `{want}`, found `{}`", got.display()),
            span,
        ));
    }

    // ----- control-flow lowering into the CFG -----------------------------

    fn check_if(
        &mut self,
        cond: &Expr,
        then_blk: &Block,
        else_blk: Option<&Expr>,
        span: Span,
    ) -> Type {
        let ct = self.check_expr(cond, Use::Value);
        self.expect_bool(&ct, cond.span, "an `if` condition");
        let c0 = match self.cur_get() {
            Some(b) => b,
            None => {
                self.check_block_value(then_blk);
                if let Some(e) = else_blk {
                    self.check_expr(e, Use::Value);
                }
                return Type::unit();
            }
        };
        let then_bb = self.new_block();
        let else_bb = self.new_block();
        let join_bb = self.new_block();
        self.set_join_span(join_bb, span);
        self.set_term(c0, Term::Branch(then_bb, else_bb));

        self.cur_set(Some(then_bb));
        let tt = self.check_block_value(then_blk);
        if let Some(cur) = self.cur_get() {
            self.set_term(cur, Term::Goto(join_bb));
        }

        self.cur_set(Some(else_bb));
        let et = if let Some(e) = else_blk {
            self.check_expr(e, Use::Value)
        } else {
            Type::unit()
        };
        if let Some(cur) = self.cur_get() {
            self.set_term(cur, Term::Goto(join_bb));
        }

        self.cur_set(Some(join_bb));
        join_types(tt, et)
    }

    fn check_match(&mut self, scrut: &Expr, arms: &[MatchArm], span: Span) -> Type {
        let (sc_ty, sc_place) = self.check_place(scrut);
        // A borrow returned by an inline call scrutinee carries return-extended
        // argument loans (§3.3). They are NOT dropped at the match head: they
        // must persist over the live range of every non-copy borrow binding
        // derived from the scrutinee (§2.3/§8.2.1) — the same treatment the
        // named-local path gets via `anchor_carried`. Captured here, re-anchored
        // onto each derived borrow binding below.
        let scrut_carried = self.take_carried();
        let resolved = patterns::resolve_enum(&sc_ty, self.items);
        let (hold, einfo, ename) = match resolved {
            Some(x) => x,
            None => {
                if !matches!(sc_ty, Type::Error) {
                    self.diags.push(Diag::error(
                        "E0603",
                        format!("match scrutinee is not an enum: `{}`", sc_ty.display()),
                        scrut.span,
                    ));
                }
                for arm in arms {
                    self.check_expr(&arm.body, Use::Value);
                }
                return Type::Error;
            }
        };
        let pats: Vec<&Pattern> = arms.iter().map(|a| &a.pattern).collect();
        if let Some(d) = patterns::check_exhaustive(&pats, &einfo, &ename, span) {
            self.diags.push(d);
        }
        // The scrutinee is read at the match head.
        self.emit_place_action(&sc_place, Use::ReadOnly, &sc_ty, scrut.span);

        let c0 = match self.cur_get() {
            Some(b) => b,
            None => {
                let mut ty = Type::Never;
                for arm in arms {
                    let mut binds = Vec::new();
                    patterns::analyze_pattern(
                        &arm.pattern,
                        &einfo,
                        &ename,
                        hold,
                        self.items,
                        &mut self.diags,
                        &mut binds,
                    );
                    self.push_scope();
                    for b in &binds {
                        self.add_local(&b.name, b.ty.clone(), b.movable);
                    }
                    ty = join_types(ty, self.check_expr(&arm.body, Use::Value));
                    self.pop_scope();
                }
                return ty;
            }
        };

        let join_bb = self.new_block();
        self.set_join_span(join_bb, span);
        let mut arm_bbs = Vec::new();
        let mut result = Type::Never;
        for arm in arms {
            let mut binds = Vec::new();
            patterns::analyze_pattern(
                &arm.pattern,
                &einfo,
                &ename,
                hold,
                self.items,
                &mut self.diags,
                &mut binds,
            );
            let b = self.new_block();
            arm_bbs.push(b);
            self.cur_set(Some(b));
            self.push_scope();
            let moves = hold == HoldMode::Owned && binds.iter().any(|bd| bd.mode == BindMode::Move);
            if moves {
                if let Some(p) = sc_place.clone() {
                    let dh = self.is_drop_hooked_partial(&p);
                    let mv = self.local_movable(&p.root);
                    self.emit(
                        &Some(p),
                        Access::Move {
                            movable: mv,
                            drop_hooked_partial: dh,
                        },
                        scrut.span,
                    );
                }
            }
            for bd in &binds {
                self.add_local(&bd.name, bd.ty.clone(), bd.movable);
                self.emit(
                    &Some(Place::local(bd.name.clone())),
                    Access::Assign {
                        needs_drop: needs_drop(&bd.ty, self.items),
                        box_paths: box_subpaths(&bd.ty, self.items),
                    },
                    bd.span,
                );
                // A borrowed-scrutinee binding is a reborrow of a scrutinee
                // sub-place: it carries a loan restricting the scrutinee (§8.2.1).
                if let Some(scp) = &sc_place {
                    let kind = match bd.mode {
                        BindMode::BorrowShared => Some(LoanKind::Shared),
                        BindMode::BorrowExcl => Some(LoanKind::Excl),
                        _ => None,
                    };
                    if let Some(k) = kind {
                        self.record_binding_loan(scp, k, bd.span, &bd.name);
                    }
                }
                // Inline call scrutinee (a temporary, no `sc_place`): the
                // return-extended argument loans persist over this borrow
                // binding's live range (§2.3/§3.3). Re-anchor a copy of each.
                if matches!(bd.mode, BindMode::BorrowShared | BindMode::BorrowExcl) {
                    for &cl in &scrut_carried {
                        let li = self.f.loans[cl].clone();
                        self.record_binding_loan(&li.place, li.kind, bd.span, &bd.name);
                    }
                }
            }
            let bt = self.check_expr(&arm.body, Use::Value);
            self.pop_scope();
            if let Some(cur) = self.cur_get() {
                self.set_term(cur, Term::Goto(join_bb));
            }
            result = join_types(result, bt);
        }
        self.set_term(c0, Term::Switch(arm_bbs));
        self.cur_set(Some(join_bb));
        result
    }

    fn check_loop(&mut self, body: &Block, span: Span) -> Type {
        let c0 = match self.cur_get() {
            Some(b) => b,
            None => {
                self.check_block_stmts(body);
                return Type::Never;
            }
        };
        let header = self.new_block();
        self.set_term(c0, Term::Goto(header));
        let break_bb = self.new_block();
        self.set_join_span(break_bb, span);
        self.loops_push(header, break_bb);
        self.cur_set(Some(header));
        self.check_block_stmts(body);
        if let Some(cur) = self.cur_get() {
            self.set_term(cur, Term::Goto(header));
        }
        self.loops_pop();
        self.cur_set(Some(break_bb));
        Type::unit()
    }

    fn check_while(&mut self, cond: &Expr, body: &Block, span: Span) -> Type {
        let c0 = match self.cur_get() {
            Some(b) => b,
            None => {
                self.check_expr(cond, Use::Value);
                self.check_block_stmts(body);
                return Type::unit();
            }
        };
        let header = self.new_block();
        self.set_term(c0, Term::Goto(header));
        self.cur_set(Some(header));
        let ct = self.check_expr(cond, Use::Value);
        self.expect_bool(&ct, cond.span, "a `while` condition");
        let body_bb = self.new_block();
        let exit_bb = self.new_block();
        self.set_join_span(exit_bb, span);
        self.set_term(header, Term::Branch(body_bb, exit_bb));
        self.loops_push(header, exit_bb);
        self.cur_set(Some(body_bb));
        self.check_block_stmts(body);
        if let Some(cur) = self.cur_get() {
            self.set_term(cur, Term::Goto(header));
        }
        self.loops_pop();
        self.cur_set(Some(exit_bb));
        Type::unit()
    }

    fn check_return(&mut self, operand: Option<&Expr>, span: Span) -> Type {
        if let Some(e) = operand {
            let ret = self.ret_ty_clone();
            self.check_against(e, &ret);
            if self.f.ret_is_borrow {
                self.check_return_provenance(e);
            }
        }
        // A `return` unwinds every open block scope (all but the parameter
        // scope, `env[0]`), dropping their locals here (§1.6 dual).
        self.emit_scope_exits(1, span);
        if let Some(cur) = self.cur_get() {
            self.set_term(cur, Term::Return);
            self.cur_set(None);
        }
        Type::Never
    }

    fn do_break(&mut self, span: Span) {
        match self.loops_break() {
            Some(brk) => {
                if let Some(cur) = self.cur_get() {
                    // `break` unwinds the loop body scopes it exits (§1.6 dual).
                    if let Some(depth) = self.loops_scope_depth() {
                        self.emit_scope_exits(depth, span);
                    }
                    self.set_term(cur, Term::Goto(brk));
                    self.cur_set(None);
                }
            }
            None => self.diags.push(Diag::error(
                "E0707",
                "`break` outside of a loop",
                span,
            )),
        }
    }

    fn do_continue(&mut self, span: Span) {
        match self.loops_continue() {
            Some(cont) => {
                if let Some(cur) = self.cur_get() {
                    // `continue` unwinds the loop body scopes it exits (§1.6 dual).
                    if let Some(depth) = self.loops_scope_depth() {
                        self.emit_scope_exits(depth, span);
                    }
                    self.set_term(cur, Term::Goto(cont));
                    self.cur_set(None);
                }
            }
            None => self.diags.push(Diag::error(
                "E0707",
                "`continue` outside of a loop",
                span,
            )),
        }
    }

    fn diverge(&mut self) {
        if let Some(cur) = self.cur_get() {
            self.set_term(cur, Term::Diverge);
            self.cur_set(None);
        }
    }
}

fn join_types(a: Type, b: Type) -> Type {
    match (&a, &b) {
        (Type::Never, _) => b,
        (_, Type::Never) => a,
        _ => {
            if assignable(&b, &a) {
                a
            } else {
                b
            }
        }
    }
}

/// The canonical root binding of the place an expression denotes.
pub(super) fn expr_place_root(e: &Expr) -> Option<String> {
    match &e.kind {
        ExprKind::Paren(i) => expr_place_root(i),
        ExprKind::Ident(n) => Some(n.clone()),
        ExprKind::Field { base, .. } | ExprKind::Index { base, .. } => expr_place_root(base),
        ExprKind::Prefix {
            op: PrefixOp::Deref,
            expr,
        } => expr_place_root(expr),
        _ => None,
    }
}

/// Which argument indices are the return-region source for a borrow-returning
/// call (design §3.3): the params tagged with the return region, or — under the
/// compact default — the sole borrow parameter.
fn region_source_indices(sig: &crate::resolve::FnSig) -> Vec<usize> {
    let bidx: Vec<usize> = sig
        .params
        .iter()
        .enumerate()
        .filter(|(_, p)| matches!(p.mode, ParamMode::Read | ParamMode::Write) || p.decl_ty.is_borrow_kind())
        .map(|(i, _)| i)
        .collect();
    match &sig.ret_region {
        Some(r) => bidx
            .into_iter()
            .filter(|&i| sig.params[i].region.as_deref() == Some(r.as_str()))
            .collect(),
        None => {
            if bidx.len() == 1 {
                vec![bidx[0]]
            } else {
                Vec::new()
            }
        }
    }
}
