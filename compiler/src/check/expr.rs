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
            ExprKind::For { .. } => unreachable!("`for` is surface-only (formatter); the pipeline desugars it at parse (design 0009 §4.2)"),
            ExprKind::Scope(b) => self.check_scope(b, e.span),
            ExprKind::Spawn(c) => self.check_spawn(c, e.span),
            ExprKind::Ident(_)
            | ExprKind::Field { .. }
            | ExprKind::Index { .. }
            | ExprKind::Prefix {
                op: PrefixOp::Deref,
                ..
            } => {
                let (t, place) = self.check_place(e);
                self.emit_place_action(&place, u, &t, e.span);
                // A borrow value read here as a plain value (copy or move) aliases
                // its source place's borrow: the source loan(s) must follow it to
                // its landing binding (design §2.3). A `read`/`write`/reborrow of a
                // place instead flows through `Use::Borrow*`, which records its own
                // loan; only the bare-value read propagates the source loan.
                if matches!(u, Use::Value) {
                    self.propagate_place_loans(&place, &t);
                }
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
            ExprKind::IntLit { value, suffix } => {
                if self.is_real() {
                    self.check_int_lit_range(*value, *suffix, false, e.span);
                }
                match suffix {
                    Some(s) => Type::Scalar(*s),
                    None => Type::IntLit,
                }
            }
            ExprKind::NegIntLit { value, suffix } => {
                if self.is_real() {
                    self.check_int_lit_range(*value, *suffix, true, e.span);
                }
                match suffix {
                    Some(s) => Type::Scalar(*s),
                    None => Type::IntLit,
                }
            }
            ExprKind::FloatLit { ty, .. } => Type::Scalar(*ty),
            ExprKind::Try(inner) => self.check_try(inner, e.span),
            ExprKind::GenericVal { name, ty_args } => self.check_generic_val(name, ty_args, e.span),
            ExprKind::StrLit(_) => Type::Str,
            ExprKind::BytesLit(_) => Type::Slice(Box::new(Type::Scalar(ScalarTy::U8))),
            ExprKind::BoolLit(_) => Type::bool(),
            ExprKind::Unary { op, expr } => self.check_unary(*op, expr),
            ExprKind::Binary { op, lhs, rhs } => self.check_binary(*op, lhs, rhs, e.span),
            ExprKind::Conv { ty, expr } => self.check_conv(ty, expr, e.span),
            ExprKind::Bitcast { ty, expr } => self.check_bitcast(ty, expr, e.span),
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
                self.regime_enter();
                self.check_block_stmts(b);
                self.regime_exit();
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
            ExprKind::Field { base, field, .. } => {
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
                    // Field access on a generic application substitutes the head's
                    // field types with the concrete/parametric arguments (§5).
                    Type::App(n, args) => {
                        match crate::types::app_fields(self.items, n, args)
                            .and_then(|fs| fs.into_iter().find(|(fn_, _)| fn_ == field))
                        {
                            Some((_, fty)) => {
                                if let Some(p) = place.as_mut() {
                                    p.proj.push(Proj::Field(field.clone()));
                                }
                                (fty, place)
                            }
                            None => {
                                self.diags.push(Diag::error(
                                    "E0107",
                                    format!("type `{}` has no field `{field}`", st.display()),
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
                    // `str[i]` yields the byte `u8` at `i` (design 0013 §3): byte
                    // indexing, bounds-faulting like any slice, never on encoding.
                    Type::Str => Type::Scalar(ScalarTy::U8),
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
            ExprKind::Field { base, field, .. } => {
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
            foreign: sig.foreign,
            ret: Box::new(sig.ret.clone()),
        })
    }

    // ----- operators ------------------------------------------------------

    fn check_unary(&mut self, op: UnOp, expr: &Expr) -> Type {
        let t = self.check_expr(expr, Use::Value);
        match op {
            UnOp::Neg => {
                // Unary minus applies to any numeric operand: integer or a float
                // (design 0016; IEEE negate never faults).
                let is_float = matches!(t, Type::Scalar(s) if s.is_float());
                if !matches!(t, Type::Error) && !is_float {
                    self.expect_integer(&t, expr.span);
                }
                t
            }
            UnOp::Not => {
                self.expect_bool(&t, expr.span, "operand of `!`");
                Type::bool()
            }
            UnOp::BitNot => {
                self.expect_integer(&t, expr.span);
                if matches!(t, Type::Error) {
                    Type::Error
                } else {
                    t
                }
            }
        }
    }

    fn check_binary(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr, span: Span) -> Type {
        let l = self.check_expr(lhs, Use::Value);
        let r = self.check_expr(rhs, Use::Value);
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                self.unify_arith(&l, &r, span)
            }
            // `%` is integer-only; floats have no `Rem` (design 0016 §2).
            BinOp::Rem => self.unify_int(&l, &r, span),
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                // `str` ordering is byte-lexicographic (design 0013 §3); otherwise
                // ordering requires numeric (integer or `f64`) operands.
                if matches!(l, Type::Str) && matches!(r, Type::Str) {
                    // ok: byte-lexicographic compare
                } else {
                    self.unify_arith(&l, &r, span);
                }
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
            // Bitwise and/or/xor: integer operands, unified like arithmetic
            // (design 0006 §2.4; width-exact, never overflow).
            BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor => self.unify_int(&l, &r, span),
            // Shifts: both operands integer; the result takes the left type.
            // The shift amount need not match the shifted type.
            BinOp::Shl | BinOp::Shr => {
                self.expect_integer(&l, lhs.span);
                self.expect_integer(&r, rhs.span);
                if matches!(l, Type::Error) {
                    Type::Error
                } else {
                    l
                }
            }
        }
    }

    /// Unify the operand types of `+ - * /` and ordered comparison: integer OR a
    /// float (`f32`/`f64`; design 0016). No implicit int<->float or `f32`<->`f64`
    /// promotion — a mixed pair is an error. Floats are exempt from the regime
    /// system (IEEE always).
    fn unify_arith(&mut self, l: &Type, r: &Type, span: Span) -> Type {
        if matches!(l, Type::Error) || matches!(r, Type::Error) {
            return Type::Error;
        }
        let l_f = matches!(l, Type::Scalar(s) if s.is_float());
        let r_f = matches!(r, Type::Scalar(s) if s.is_float());
        if l_f || r_f {
            // Both operands must be the *same* float type — no int<->float and no
            // `f32`<->`f64` implicit conversion.
            if let (Type::Scalar(ls), Type::Scalar(rs)) = (l, r) {
                if ls.is_float() && ls == rs {
                    return Type::Scalar(*ls);
                }
            }
            self.diags.push(Diag::error(
                "E0703",
                format!(
                    "arithmetic requires operands of the same type, found `{}` and `{}` (no implicit int<->float / f32<->f64 conversion — use `conv`)",
                    l.display(),
                    r.display()
                ),
                span,
            ));
            return Type::Error;
        }
        self.unify_int(l, r, span)
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
        let is_num = |t: &Type| t.is_integer() || matches!(t, Type::Scalar(s) if s.is_float());
        if !matches!(src, Type::Error) && !is_num(&src) {
            self.diags.push(
                Diag::error(
                    "E0701",
                    format!("`conv` operand must be a number, found `{}`", src.display()),
                    span,
                )
                .with_note("`conv` converts between numeric (integer / `f32` / `f64`) types (§8.1; 0016 §5)", None),
            );
        }
        if !matches!(target, Type::Error) && !is_num(&target) {
            self.diags.push(
                Diag::error(
                    "E0701",
                    format!("`conv` target must be a numeric type, found `{}`", target.display()),
                    span,
                )
                .with_note("pointer retyping uses `cast_ptr`/`addr_to_ptr` (§4.2)", None),
            );
        }
        // Constant-known loss is a compile error in the default regime (design
        // 0006 §2.4): if the operand folds to a constant that does not fit the
        // scalar target, reject at check time. Inside `wrapping`/`saturating`
        // the regime folds it instead, so we skip the check there.
        if self.is_real() && !self.in_regime() {
            if let (Some(v), Type::Scalar(ts)) = (const_int(expr), &target) {
                if ts.is_integer() && !scalar_fits(v, *ts) {
                    self.diags.push(
                        Diag::error(
                            "E0710",
                            format!(
                                "constant `{}` does not fit target type `{}` (a `conv` that is known to lose value is rejected)",
                                v,
                                target.display()
                            ),
                            span,
                        )
                        .with_note(
                            "wrap the conversion in a `wrapping` or `saturating` block to fold it, or use a value that fits (design 0006 §2.4)",
                            None,
                        ),
                    );
                }
            }
        }
        target
    }

    /// Type a `bitcast T (e)` -- same-width bit reinterpretation (design 0016 section
    /// 10). LEGAL: exactly one side a float, the other an integer of the SAME byte
    /// width (f64<->{i64,u64,isize,usize}, f32<->{i32,u32}). A bare `{integer}` on the
    /// int side is width-flexible (concretized to the float's same-width unsigned int
    /// at lowering). Bitcast is total -- it reinterprets bits, never faulting, and is
    /// regime-independent. The result type is `T`.
    fn check_bitcast(&mut self, ty: &Ty, expr: &Expr, span: Span) -> Type {
        use crate::interp::layout::Layout;
        let src = self.check_expr(expr, Use::Value);
        let target = self.resolve_ty(ty);
        if matches!(src, Type::Error) || matches!(target, Type::Error) {
            return target;
        }
        let legal = match &target {
            Type::Scalar(ts) if ts.is_float() => match &src {
                Type::IntLit => {
                    self.check_bitcast_lit_fits(expr, *ts, span);
                    true
                }
                Type::Scalar(s) if s.is_integer() => {
                    Layout::scalar_size(*s) == Layout::scalar_size(*ts)
                }
                _ => false,
            },
            Type::Scalar(ts) if ts.is_integer() => {
                matches!(&src, Type::Scalar(s)
                    if s.is_float() && Layout::scalar_size(*s) == Layout::scalar_size(*ts))
            }
            _ => false,
        };
        if !legal {
            self.diags.push(
                Diag::error(
                    "E0714",
                    format!(
                        "illegal `bitcast`: `{}` and `{}` are not a same-width float<->integer pair",
                        src.display(),
                        target.display()
                    ),
                    span,
                )
                .with_note(
                    "`bitcast` reinterprets the identical bits, so the two types must be the same byte width and exactly one a float — legal: f64<->{i64,u64,isize,usize}, f32<->{i32,u32} (design 0016 §10)",
                    None,
                ),
            );
        }
        target
    }

    /// A bare `{integer}` literal on the int side of a `bitcast` to `float_ty` must
    /// fit that float's same-width UNSIGNED integer, so its full bit pattern is
    /// representable (a high-bit pattern needs an explicit `u64`/`u32` suffix).
    fn check_bitcast_lit_fits(&mut self, expr: &Expr, float_ty: ScalarTy, span: Span) {
        use crate::interp::layout::Layout;
        let uint = if float_ty == ScalarTy::F64 { ScalarTy::U64 } else { ScalarTy::U32 };
        if let Some(v) = const_int(expr) {
            if !scalar_fits(v, uint) {
                self.diags.push(Diag::error(
                    "E0714",
                    format!(
                        "integer literal `{}` does not fit the {}-byte bit pattern of `{}` (add a `{}` suffix for a high-bit pattern)",
                        v,
                        Layout::scalar_size(float_ty),
                        scalar_name(float_ty),
                        scalar_name(uint),
                    ),
                    span,
                ));
            }
        }
    }

    // ----- struct / enum construction -------------------------------------

    fn check_struct_lit(&mut self, name: &str, fields: &[FieldInit], span: Span) -> Type {
        if let Some(g) = self.items.generic_defs.get(name).cloned() {
            if !g.is_enum {
                return self.check_generic_struct_lit(name, &g, fields, span);
            }
        }
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
        if let Some(g) = self.items.generic_defs.get(enum_name).cloned() {
            if g.is_enum {
                return self.check_generic_enum_ctor(enum_name, &g, variant, args, span);
            }
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
                        "subslice" | "substr" | "as_bytes" | "as_str" => args.first().and_then(|a| self.borrow_provenance(a)),
                        _ => {
                            if let Some(sig) = self.items.fns.get(name) {
                                if sig.ret.is_borrow_kind() {
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

    /// Range-check an integer literal (real front-end only; spec 01 §3.3). A
    /// positive literal must fit its type's maximum; a negative-literal fold must
    /// fit its type's minimum. Unsuffixed literals default to `i64`.
    pub(super) fn check_int_lit_range(
        &mut self,
        value: u64,
        suffix: Option<ScalarTy>,
        neg: bool,
        span: Span,
    ) {
        let sty = suffix.unwrap_or(ScalarTy::I64);
        let (min, max) = scalar_range(sty);
        let v: i128 = if neg { -(value as i128) } else { value as i128 };
        if v < min || v > max {
            self.diags.push(
                Diag::error(
                    "E0709",
                    format!(
                        "integer literal `{}{}` is out of range for `{}`",
                        if neg { "-" } else { "" },
                        value,
                        scalar_name(sty)
                    ),
                    span,
                )
                .with_note(
                    "an over-range literal is rejected at compile time, never a runtime fault (spec 01 §3.3)",
                    None,
                ),
            );
        }
    }

    /// Type-check `expr?` (spec 02 §6.5). Requires a result-shaped enum whose
    /// enclosing function returns the same enum type (same-type-only per 0006).
    fn check_try(&mut self, inner: &Expr, span: Span) -> Type {
        let t = self.check_expr(inner, Use::Value);
        if matches!(t, Type::Error | Type::Never) {
            return Type::Error;
        }
        let resolved = patterns::resolve_enum(&t, self.items);
        let (_, einfo, ename) = match resolved {
            Some(x) => x,
            None => {
                self.diags.push(
                    Diag::error(
                        "E0711",
                        format!("`?` applied to `{}`, which is not an enum", t.display()),
                        span,
                    )
                    .with_note("`?` unwraps a result-shaped enum (spec 02 §6.5)", None),
                );
                return Type::Error;
            }
        };
        let ok_name = match &einfo.ok_variant {
            Some(n) => n.clone(),
            None => {
                self.diags.push(
                    Diag::error(
                        "E0711",
                        format!("`?` applied to enum `{ename}`, which is not result-shaped"),
                        span,
                    )
                    .with_note("mark exactly one variant `ok` to make an enum result-shaped (spec 02 §2.2)", None),
                );
                return Type::Error;
            }
        };
        // Same-type propagation, or cross-type propagation through a `From` impl
        // (design 0007 §7.1).
        let ret = self.ret_ty_clone();
        let mut same = match (&t, &ret) {
            (Type::Named(a), Type::Named(b)) => a == b,
            // Same-type `?` on a *generic* result enum: both sides are the same
            // instantiation `App(head, args)` (this never reaches `Type::Named`,
            // so without this arm even single-file generic `?` fell to E0712) (F2).
            (Type::App(a, aa), Type::App(b, bb)) => a == b && aa == bb,
            (Type::BoxResult(_), Type::BoxResult(_)) => true,
            _ => false,
        };
        if !same {
            match self.try_from_conversion(&t, &ret, &einfo, &ename, span) {
                Some(()) => {
                    // Cross-type `?`: treated like same-type control flow below, but
                    // the desugared `E2::from(e1)` call inherits the `From` effect.
                    same = true;
                }
                None => {
                    self.diags.push(
                        Diag::error(
                            "E0712",
                            format!(
                                "`?` on `{}` requires the function to return `{}` or an `impl From[..] for` its error type",
                                t.display(),
                                t.display()
                            ),
                            span,
                        )
                        .with_note("cross-type `?` needs `impl From[E1] for E2` in scope (design 0007 §7.1)", None),
                    );
                }
            }
        }
        let payload = einfo
            .variants
            .iter()
            .find(|v| v.name == ok_name)
            .and_then(|v| v.payload.first().cloned())
            .unwrap_or_else(Type::unit);
        // A `?` is control flow (an early return), not a read, so it cannot be a
        // read-only contract clause. The simplest sound rule is to forbid it
        // inside a `requires`/`ensures`/`assert` condition outright (this fix):
        // modeling its exit as a real return would early-return the function in
        // the middle of contract evaluation, which is incoherent under P8 — and
        // the ensures re-emission runs with `cur` pointing at a return block, so
        // forking there would corrupt the CFG. Reject and do NOT fork.
        if self.f_in_contract() {
            self.diags.push(
                Diag::error(
                    "E0708",
                    "the `?` operator is not allowed inside a contract clause",
                    span,
                )
                .with_note(
                    "a `requires`/`ensures`/`assert` condition is read-only and must not early-return; `?` is control flow, not a read (this fix)",
                    None,
                ),
            );
        } else if same {
            // A well-typed `?` is a conditional early return: model it in the CFG.
            self.emit_try_exit(span);
        }
        payload
    }

    /// Model `?` in the CFG as a conditional early return (design 0003 §2.2/§2.6).
    /// The operand has just been evaluated (and consumed) into the current block;
    /// here that block forks into the `ok` fall-through — where the `?` expression
    /// yields the payload — and a genuine `Return` edge carrying the propagated
    /// value. A `?` return IS a normal return, so the exit block unwinds the open
    /// block scopes exactly as `check_return` does (§1.6 dual, driving the E0309
    /// drop-point checks) and the ensures re-emission (mod.rs) treats it as a
    /// return point (contracts must hold, the propagated value binds `result`).
    fn emit_try_exit(&mut self, span: Span) {
        let cur = match self.cur_get() {
            Some(c) => c,
            None => return,
        };
        let exit_bb = self.new_block();
        let ok_bb = self.new_block();
        self.set_join_span(exit_bb, span);
        self.set_term(cur, Term::Branch(ok_bb, exit_bb));
        // Propagate path: a real return that unwinds the open block scopes.
        self.cur_set(Some(exit_bb));
        self.emit_scope_exits(1, span);
        self.set_term(exit_bb, Term::Return);
        // Ok path: control continues; the payload is the expression's value.
        self.cur_set(Some(ok_bb));
    }

    /// Reject a write target that reaches a field/element *through* a single-place
    /// borrow without an explicit `.*` (design 0006 §2.4 read-only auto-deref;
    /// spec 02 §6.3). Real front-end only: a write keeps every deref explicit, so
    /// a mutation audit (grep `.\* =`) stays complete. Reads auto-deref freely.
    pub(super) fn reject_autoderef_write(&mut self, target: &Expr) {
        if !self.is_real() {
            return;
        }
        if let Some(sp) = self.autoderef_write_violation(target) {
            self.diags.push(
                Diag::error(
                    "E0713",
                    "a write through a borrow needs an explicit `.*` dereference",
                    sp,
                )
                .with_note(
                    "read access auto-derefs a borrow, but every deref on a write target must be written `.*` (design 0006 §2.4)",
                    None,
                ),
            );
        }
    }

    fn autoderef_write_violation(&self, e: &Expr) -> Option<Span> {
        match &e.kind {
            ExprKind::Paren(i) => self.autoderef_write_violation(i),
            ExprKind::Field { base, .. } | ExprKind::Index { base, .. } => {
                let bt = self.ty_of_place(base);
                let base_is_deref = matches!(
                    strip_paren(base).kind,
                    ExprKind::Prefix { op: PrefixOp::Deref, .. }
                );
                if matches!(bt, Type::Borrow(_) | Type::BorrowMut(_)) && !base_is_deref {
                    Some(base.span)
                } else {
                    self.autoderef_write_violation(base)
                }
            }
            _ => None,
        }
    }

    /// Non-emitting type of a place expression, for the write-deref probe.
    fn ty_of_place(&self, e: &Expr) -> Type {
        match &e.kind {
            ExprKind::Paren(i) => self.ty_of_place(i),
            ExprKind::Ident(name) => {
                if let Some(li) = self.lookup_local(name) {
                    li.ty.clone()
                } else if let Some((t, _)) = self.items.statics.get(name) {
                    t.clone()
                } else {
                    Type::Error
                }
            }
            ExprKind::Prefix { op: PrefixOp::Deref, expr } => {
                match self.ty_of_place(expr) {
                    Type::Borrow(x) | Type::BorrowMut(x) | Type::Box(x) => *x,
                    _ => Type::Error,
                }
            }
            ExprKind::Field { base, field, .. } => {
                let bt = peel_borrow(self.ty_of_place(base));
                match bt {
                    Type::Named(n) => self
                        .items
                        .lookup_struct(&n)
                        .and_then(|st| st.fields.iter().find(|(f, _)| f == field).map(|(_, t)| t.clone()))
                        .unwrap_or(Type::Error),
                    _ => Type::Error,
                }
            }
            ExprKind::Index { base, .. } => {
                let bt = peel_borrow(self.ty_of_place(base));
                match bt {
                    Type::Array(e, _) | Type::Slice(e) | Type::SliceMut(e) => *e,
                    _ => Type::Error,
                }
            }
            _ => Type::Error,
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
            if self.items.generic_fns.contains_key(name) {
                return self.check_generic_call(name, args, span);
            }
            if let Some(sig) = self.items.fns.get(name).cloned() {
                return self.check_user_call(&sig, args, span);
            }
            // A foreign (`extern`) call (design 0011 §2): the ground source of the
            // `foreign` effect. It is unsafe in principle and either discharges (in
            // a boundary wrapper covered by trust) or propagates — decided at the
            // function's end from the accumulated set.
            if let Some(es) = self.items.externs.get(name).cloned() {
                self.require_unsafe_foreign(span, &format!("`{}`", es.name));
                self.record_extern_call(&es.name, es.has_trust(), span);
                let sig = es.to_fn_sig();
                return self.check_user_call(&sig, args, span);
            }
        }
        // A method call: `receiver.method(args)` parses as a call whose callee is a
        // field access. Resolve it against interface impls (design 0007 §2.3).
        if let ExprKind::Field { base, field, .. } = &callee.kind {
            if let Some(t) = self.try_method_call(base, field, args, span) {
                return t;
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
                if fp.foreign {
                    // An indirect call through a `foreign` fn-pointer is a foreign
                    // call (design 0011 §2): unsafe, and undischargeable here.
                    self.require_unsafe_foreign(span, "a `foreign` function pointer");
                    self.record_foreign_candor("<fn-pointer>", span);
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
        // A call to a `foreign`-marked Candor function propagates the effect
        // (design 0011 §2 rule 3). Extern ground sources are recorded separately by
        // the caller (they carry trust and may discharge); this branch fires only
        // for an already-`foreign` Candor wrapper (`sig.name` is the extern's own
        // synthesized sig only when reached via the extern path, where propagation
        // is intended too — the discharge rule still extinguishes it there).
        if sig.foreign && !self.items.externs.contains_key(&sig.name) {
            self.record_foreign_candor(&sig.name, span);
        }
        // Return-borrow extension: a returned borrow OR view (`[T]`/`str`) keeps
        // the loan(s) on the argument(s) tagged with the return's region alive
        // (§3.1/§3.3) — a view aliases its source's backing exactly as a borrow
        // does, so a view laundered out of a call must carry the source loan.
        if sig.ret.is_borrow_kind() {
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

    pub(super) fn check_arg_mode(&mut self, mode: ParamMode, decl_ty: &Type, arg: &Expr) {
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
        self.reject_autoderef_write(arg);
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

    /// The first argument's underlying collection type, peeling *every* leading
    /// `read`/`write` borrow. A collection is never itself a borrow, so peeling
    /// all layers recognizes the receiver through a re-borrow of an already-
    /// borrowed param (`get(read v, i)` where `v: read Vec[T]`) identically to
    /// the bare `v` — dispatch recognition only; the arm bodies still borrow-check
    /// the argument. (`synth_arg_type` peels the outermost borrow; a re-borrow
    /// leaves a second one, which this loop strips.)
    fn arg0_collection_ty(&mut self, args: &[Expr]) -> Option<Type> {
        let mut t = self.synth_arg_type(args.first()?);
        while let Type::Borrow(inner) | Type::BorrowMut(inner) = t {
            t = *inner;
        }
        Some(t)
    }

    /// Does the first argument have type `String`? Routes the overloaded builtins
    /// (`push`/`append`/`as_str`) to the String forms, leaving same-named user
    /// functions (e.g. an Arena `push`) alone.
    fn arg0_is_string(&mut self, args: &[Expr]) -> bool {
        matches!(self.arg0_collection_ty(args), Some(Type::Named(n)) if n == "String")
    }

    /// Does the first argument have type `Vec[T]`? Routes the overloaded collection
    /// builtins (`push`/`pop`/`get`/`set`/`len`) to the `Vec` forms, leaving
    /// same-named user functions alone.
    fn arg0_is_vec(&mut self, args: &[Expr]) -> bool {
        matches!(self.arg0_collection_ty(args), Some(Type::App(n, _)) if n == "Vec")
    }

    /// The element type `T` of the `Vec[T]` named by the first argument. `Error`
    /// if the argument is not a `Vec`.
    fn vec_arg_elem(&mut self, args: &[Expr]) -> Type {
        match self.arg0_collection_ty(args) {
            Some(Type::App(n, targs)) if n == "Vec" => targs.first().cloned().unwrap_or(Type::Error),
            _ => Type::Error,
        }
    }

    /// Does the first argument have type `Map[V]`? Routes the overloaded collection
    /// builtins (`insert`/`contains`/`get`) to the `Map` forms.
    fn arg0_is_map(&mut self, args: &[Expr]) -> bool {
        matches!(self.arg0_collection_ty(args), Some(Type::App(n, _)) if n == "Map")
    }

    /// The value type `V` of the `Map[V]` named by the first argument.
    fn map_arg_valty(&mut self, args: &[Expr]) -> Type {
        match self.arg0_collection_ty(args) {
            Some(Type::App(n, targs)) if n == "Map" => targs.first().cloned().unwrap_or(Type::Error),
            _ => Type::Error,
        }
    }

    /// A `Map` key is a byte-string view: `str` or `[u8]` (design 0013 §1.3). No
    /// user-defined-key hashing (the "refuse the language form" ruling).
    fn expect_bytestring(&mut self, t: &Type, span: Span, ctx: &str) {
        let ok = matches!(t, Type::Str | Type::Error)
            || matches!(t, Type::Slice(e) if matches!(**e, Type::Scalar(ScalarTy::U8)));
        if !ok {
            self.mismatch(span, ctx, "str or [u8]", t);
        }
    }

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
            "sqrt" => {
                // `sqrt(x)` — correctly-rounded IEEE square root (design 0016 §11),
                // overloaded by the argument type (`f32 -> f32`, `f64 -> f64`). A
                // total, non-faulting op: `sqrt` of a negative is NaN, not a fault.
                let t = self.arg0(args, span, "sqrt");
                match t {
                    Type::Scalar(s) if s.is_float() => Type::Scalar(s),
                    Type::Error => Type::Error,
                    other => {
                        self.mismatch(span, "sqrt", "a float (f32 or f64)", &other);
                        Type::Error
                    }
                }
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
                    let (t, place) = self.check_place(&args[0]);
                    // An exclusive reborrow of an argument that is ITSELF a
                    // shared slice is illegal independent of any path peeling
                    // — a bare shared binding, a subslice-of-shared result, or
                    // any other expression of shared-slice type (E0809;
                    // retest 2026-07-08 #2, finding 3 residual: the path gate
                    // below only peels derefs, it never examines the
                    // argument's own type).
                    if matches!(t, Type::Slice(_)) {
                        self.e0809(
                            "exclusive reborrow of a shared slice",
                            args[0].span,
                        );
                        Type::Error
                    } else {
                        // `slice_of_mut` is an exclusive borrow of the run, so
                        // it may not reborrow exclusively from behind a
                        // shared deref (§5.2; retest 2026-07-08, finding 3) —
                        // same gate as a write.
                        self.reject_write_through_shared(
                            &args[0],
                            "take an exclusive (`slice_mut`) slice",
                            args[0].span,
                        );
                        self.emit_place_action(&place, Use::BorrowExcl, &t, args[0].span);
                        match t {
                            Type::Array(e, _) | Type::SliceMut(e) => Type::SliceMut(e),
                            Type::Error => Type::Error,
                            other => {
                                self.mismatch(span, "slice_of_mut", "an array", &other);
                                Type::Error
                            }
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
            // ----- text: `String` builder (std, design 0013 §3) --------------
            "string_new" => {
                // `string_new(a: read Alloc) -> String` — allocator-explicit (P9).
                let t = self.arg0(args, span, "string_new");
                let peeled = match &t { Type::Borrow(i) | Type::BorrowMut(i) => (**i).clone(), _ => t.clone() };
                if !matches!(peeled, Type::Error) && !self.is_alloc_handle(&peeled) {
                    self.mismatch(span, "string_new", "read Alloc", &t);
                }
                self.note_alloc(span, "`string_new` builds an owning String (allocates)");
                Type::Named("String".to_string())
            }
            "push" if self.arg0_is_string(args) => {
                // `push(write self: String, c: u32)` — appends one Unicode scalar,
                // UTF-8-encoded. The `enforced requires(is_scalar_value(c))` is a
                // runtime P5 backstop (design 0013 §3), enforced by the interpreter.
                if args.len() == 2 {
                    self.check_expr(&args[0], Use::Value);
                    let c = self.check_expr(&args[1], Use::Value);
                    self.expect_integer(&c, args[1].span);
                } else {
                    self.arity(span, "push", 2);
                }
                self.note_alloc(span, "`push` may grow the String (allocates)");
                Type::unit()
            }
            "append" if self.arg0_is_string(args) => {
                // `append(write self: String, s: read str)` — appends a view.
                if args.len() == 2 {
                    self.check_expr(&args[0], Use::Value);
                    let sv = self.check_expr(&args[1], Use::Value);
                    if !matches!(sv, Type::Str | Type::Error) {
                        self.mismatch(span, "append", "str", &sv);
                    }
                } else {
                    self.arity(span, "append", 2);
                }
                self.note_alloc(span, "`append` may grow the String (allocates)");
                Type::unit()
            }
            "as_str" if self.arg0_is_string(args) => {
                // `as_str(read self: String) -> str` — borrow the built text back.
                self.arg0(args, span, "as_str");
                Type::Str
            }
            // ----- std hash `Map[V]` — byte-string keys (`str`/`[u8]`), P9 -----
            "map_new" => {
                // `map_new(a: read Alloc) -> Map[V]` — allocator-explicit (P9). The
                // value type `V` is fixed by the expected (annotation) type.
                let t = self.arg0(args, span, "map_new");
                let peeled = match &t { Type::Borrow(i) | Type::BorrowMut(i) => (**i).clone(), _ => t.clone() };
                if !matches!(peeled, Type::Error) && !self.is_alloc_handle(&peeled) {
                    self.mismatch(span, "map_new", "read Alloc", &t);
                }
                self.note_alloc(span, "`map_new` builds an owning Map (allocates/frees on drop)");
                match &self.expected_ty {
                    Some(Type::App(n, targs)) if n == "Map" => Type::App("Map".to_string(), targs.clone()),
                    _ => Type::App("Map".to_string(), vec![Type::Error]),
                }
            }
            "insert" if self.arg0_is_map(args) => {
                // `insert(write self: Map[V], key: read str/[u8], v: V)` — moves `v`
                // in; drops the displaced value on an existing key.
                let valty = self.map_arg_valty(args);
                if args.len() == 3 {
                    self.check_expr(&args[0], Use::Value);
                    let k = self.check_expr(&args[1], Use::Value);
                    self.expect_bytestring(&k, args[1].span, "insert");
                    self.check_against(&args[2], &valty);
                } else {
                    self.arity(span, "insert", 3);
                }
                self.note_alloc(span, "`insert` may allocate a key copy / grow the Map");
                Type::unit()
            }
            "contains" if self.arg0_is_map(args) => {
                // `contains(read self: Map[V], key: read str/[u8]) -> bool`.
                if args.len() == 2 {
                    self.check_expr(&args[0], Use::BorrowShared);
                    let k = self.check_expr(&args[1], Use::Value);
                    self.expect_bytestring(&k, args[1].span, "contains");
                } else {
                    self.arity(span, "contains", 2);
                }
                Type::bool()
            }
            "get" if self.arg0_is_map(args) => {
                // `get(read self: Map[V], key: read str/[u8]) -> read V` — a borrow
                // of the stored value; FAULTS if absent (pair with `contains`).
                let valty = self.map_arg_valty(args);
                if args.len() == 2 {
                    self.check_expr(&args[0], Use::BorrowShared);
                    let k = self.check_expr(&args[1], Use::Value);
                    self.expect_bytestring(&k, args[1].span, "get");
                } else {
                    self.arity(span, "get", 2);
                }
                Type::Borrow(Box::new(valty))
            }
            // ----- std collection: `Vec[T]` (growable heap array, P9) ----------
            "vec_new" => {
                // `vec_new(a: read Alloc) -> Vec[T]` — allocator-explicit (P9). The
                // element type `T` is fixed by the expected (annotation) type.
                let t = self.arg0(args, span, "vec_new");
                let peeled = match &t { Type::Borrow(i) | Type::BorrowMut(i) => (**i).clone(), _ => t.clone() };
                if !matches!(peeled, Type::Error) && !self.is_alloc_handle(&peeled) {
                    self.mismatch(span, "vec_new", "read Alloc", &t);
                }
                self.note_alloc(span, "`vec_new` builds an owning Vec (allocates/frees on drop)");
                match &self.expected_ty {
                    Some(Type::App(n, targs)) if n == "Vec" => Type::App("Vec".to_string(), targs.clone()),
                    _ => Type::App("Vec".to_string(), vec![Type::Error]),
                }
            }
            "push" if self.arg0_is_vec(args) => {
                // `push(write self: Vec[T], v: T)` — moves `v` in, growing if full.
                let elem = self.vec_arg_elem(args);
                if args.len() == 2 {
                    self.check_expr(&args[0], Use::Value);
                    self.check_against(&args[1], &elem);
                } else {
                    self.arity(span, "push", 2);
                }
                self.note_alloc(span, "`push` may grow the Vec (allocates)");
                Type::unit()
            }
            "pop" if self.arg0_is_vec(args) => {
                // `pop(write self: Vec[T]) -> Opt[T]` — moves the last element out
                // (ownership transfers into the `Some` payload; nothing is dropped).
                self.arg0(args, span, "pop");
                Type::Named("Opt".to_string())
            }
            "get" if self.arg0_is_vec(args) => {
                // `get(read self: Vec[T], i: usize) -> read T` — a bounds-faulting
                // shared-borrow accessor of the element at `i`.
                let elem = self.vec_arg_elem(args);
                if args.len() == 2 {
                    self.check_expr(&args[0], Use::BorrowShared);
                    let i = self.check_expr(&args[1], Use::Value);
                    self.expect_integer(&i, args[1].span);
                } else {
                    self.arity(span, "get", 2);
                }
                Type::Borrow(Box::new(elem))
            }
            "set" if self.arg0_is_vec(args) => {
                // `set(write self: Vec[T], i: usize, v: T)` — drops the old element
                // at `i` (allocator work if it owns heap) and moves `v` in.
                let elem = self.vec_arg_elem(args);
                if args.len() == 3 {
                    self.check_expr(&args[0], Use::Value);
                    let i = self.check_expr(&args[1], Use::Value);
                    self.expect_integer(&i, args[1].span);
                    self.check_against(&args[2], &elem);
                } else {
                    self.arity(span, "set", 3);
                }
                self.note_alloc(span, "`set` drops the overwritten element (may free)");
                Type::unit()
            }
            // ----- text: `str` core operations (design 0013) -----------------
            "as_bytes" => {
                // `as_bytes(s: str) -> [u8]` is a FREE retype: the UTF-8 view IS a
                // byte run (design 0013 §1.3). No implicit reverse (P2).
                let t = self.arg0(args, span, "as_bytes");
                match t {
                    Type::Str => Type::Slice(Box::new(Type::Scalar(ScalarTy::U8))),
                    Type::Error => Type::Error,
                    other => {
                        self.mismatch(span, "as_bytes", "str", &other);
                        Type::Error
                    }
                }
            }
            "str_from" => {
                // `str_from(b: [u8]) -> Utf8Res` validates UTF-8; on invalid input
                // it yields the byte offset of the first bad sequence (P7 error, not
                // a fault — design 0013 §4). `Utf8Res` is the core result-shaped enum
                // `enum Utf8Res { ok Valid(str), Invalid(usize) }`.
                let t = self.arg0(args, span, "str_from");
                match t {
                    Type::Slice(e) if matches!(*e, Type::Scalar(ScalarTy::U8)) => {}
                    Type::Error => return Some(Type::Named("Utf8Res".to_string())),
                    other => { self.mismatch(span, "str_from", "[u8]", &other); }
                }
                Type::Named("Utf8Res".to_string())
            }
            "str_from_unchecked" => {
                // The unchecked construction (design 0013 §4): skips validation, so
                // it is `unsafe` and carries a mandatory justification (P1).
                self.require_unsafe(span, "str_from_unchecked");
                let t = self.arg0(args, span, "str_from_unchecked");
                match t {
                    Type::Slice(e) if matches!(*e, Type::Scalar(ScalarTy::U8)) => Type::Str,
                    Type::Error => Type::Error,
                    other => { self.mismatch(span, "str_from_unchecked", "[u8]", &other); Type::Error }
                }
            }
            "substr" => {
                // `substr(s: str, a, b) -> str` — the `str` sub-view `[a, b)`, which
                // FAULTS (P5) if `a`/`b` is not on a UTF-8 char boundary or is out of
                // bounds (design 0013 §3). Byte-agnostic sub-runs go via `as_bytes`.
                if args.len() == 3 {
                    let sv = self.check_expr(&args[0], Use::Value);
                    let lo = self.check_expr(&args[1], Use::Value);
                    let hi = self.check_expr(&args[2], Use::Value);
                    self.expect_integer(&lo, args[1].span);
                    self.expect_integer(&hi, args[2].span);
                    match sv {
                        Type::Str => Type::Str,
                        Type::Error => Type::Error,
                        other => { self.mismatch(span, "substr", "str", &other); Type::Error }
                    }
                } else {
                    self.arity(span, "substr", 3);
                    Type::Error
                }
            }
            "char_at" => {
                // `char_at(s: str, pos: usize) -> CharStep` (OBL-TEXT-CHARS value-
                // gear decoder): decode the UTF-8 scalar at `pos`, returning its
                // code point and the next byte position — all owned, no borrow, no
                // alloc. FAULTS (P5) at runtime on a non-boundary/ill-formed `pos`.
                if args.len() == 2 {
                    let sv = self.check_expr(&args[0], Use::Value);
                    let p = self.check_expr(&args[1], Use::Value);
                    self.expect_integer(&p, args[1].span);
                    match sv {
                        Type::Str => Type::Named("CharStep".to_string()),
                        Type::Error => Type::Error,
                        other => {
                            self.mismatch(span, "char_at", "str", &other);
                            Type::Error
                        }
                    }
                } else {
                    self.arity(span, "char_at", 2);
                    Type::Error
                }
            }
            "char_count" => {
                // `char_count(s: str) -> usize` (design 0013 §3): the O(n) Unicode-
                // scalar count (decode-and-advance to the end) — the UTF-8 tax made
                // visible in the name (P4). Ships WITH the char protocol.
                let t = self.arg0(args, span, "char_count");
                match t {
                    Type::Str | Type::Error => {}
                    other => {
                        self.mismatch(span, "char_count", "str", &other);
                    }
                }
                Type::usize()
            }
            "len" => {
                self.arg0(args, span, "len");
                Type::usize()
            }
            // Structured-concurrency blessed primitives (design 0012 §1.4, §3.3).
            "split_mut" => self.check_split_mut(args, span),
            "cancelled" => self.check_cancelled(args, span),
            // Prototype observability intrinsic (Stage 4): appends an i64 to the
            // interpreter's trace log so drop-order tests can observe destruction.
            "trace" => {
                // `trace` observes a word: an integer value, or a float's IEEE bit
                // pattern (the cross-engine observable channel; design 0016).
                let t = self.arg0(args, span, "trace");
                let is_float = matches!(t, Type::Scalar(s) if s.is_float());
                if !matches!(t, Type::Error) && !is_float {
                    self.expect_integer(&t, span);
                }
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

    /// True if `ty` is the compiler-known allocator handle struct, identified
    /// STRUCTURALLY (never by a hardcoded name), mirroring the interpreter's
    /// finding-F1 detection in `interp/eval.rs`: the handle is a struct whose
    /// `vt` field is a `rawptr` to the vtable struct, and the vtable is the
    /// struct with fn-ptr fields `alloc` and `free`. This lets the
    /// allocator-explicit builtins accept the allocator regardless of how the
    /// module tree qualifies its name (bare `Alloc` vs `analyses::Alloc`).
    fn is_alloc_handle(&self, ty: &Type) -> bool {
        let Type::Named(name) = ty else { return false };
        let Some(handle) = self.items.lookup_struct(name) else { return false };
        handle.fields.iter().any(|(fname, fty)| {
            fname == "vt"
                && matches!(fty, Type::RawPtr(inner)
                    if matches!(&**inner, Type::Named(vt) if self.is_alloc_vtable(vt)))
        })
    }

    fn is_alloc_vtable(&self, name: &str) -> bool {
        let Some(vt) = self.items.lookup_struct(name) else { return false };
        vt.fields.iter().any(|(n, t)| n == "alloc" && matches!(t, Type::FnPtr(_)))
            && vt.fields.iter().any(|(n, t)| n == "free" && matches!(t, Type::FnPtr(_)))
    }

    fn arity(&mut self, span: Span, name: &str, n: usize) {
        self.diags.push(Diag::error(
            "E0706",
            format!("`{name}` expects {n} argument(s)"),
            span,
        ));
    }

    pub(super) fn mismatch(&mut self, span: Span, name: &str, want: &str, got: &Type) {
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
                // An integer scrutinee takes the literal-pattern path (design
                // 0001 §8.2 extended): literal arms compare-and-branch, and a
                // `_`/binding catch-all is required for exhaustiveness.
                if let Type::Scalar(s) = sc_ty {
                    if s.is_integer() {
                        return self.check_int_match(scrut, arms, span, s, &sc_place);
                    }
                }
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
        if let Some(d) = patterns::check_exhaustive(arms, &einfo, &ename, span) {
            self.diags.push(d);
        }
        // Record the concrete enum instance behind each arm's pattern so that
        // monomorphization can lower the pattern's enum name (design 0007 §5).
        if let Some((gname, gargs)) = generic_enum_of(&sc_ty) {
            if !gargs.iter().any(|t| matches!(t, Type::Error)) {
                self.record_inst(&gname, gargs.clone());
                for arm in arms {
                    self.shapes.insert((self.cur_item, arm.pattern.span.start), crate::generics::Shape::Type(gname.clone(), gargs.clone()));
                }
            }
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
                    self.check_arm_guard(arm);
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
            // An owned-scrutinee match consumes the scrutinee (§1.6). A move
            // binding takes ownership of the whole value; and an arm whose
            // matched variant carries a drop-inert payload (e.g. `Opt::None`)
            // also consumes it — the value is destructured with nothing left to
            // drop or free, so the scrutinee does not survive to a function-exit
            // drop. This makes a forwarding `map`'s `None` arm non-`alloc`
            // (design 0009 §1.2): without it, the live `o` on the `None` path was
            // conservatively flagged as a `Box`-freeing drop (E0401).
            // An owned-scrutinee match consumes the scrutinee (§1.6). A move
            // binding takes the whole value; and a DIVERGING arm (one that
            // returns/breaks, never falling through to a later use) whose matched
            // variant carries a drop-inert payload also consumes it — the value is
            // gone with nothing to free, so it does not survive to a function-exit
            // drop. This makes a forwarding `map`'s `None => return None` arm
            // non-`alloc` (design 0009 §1.2) without disturbing a non-diverging
            // match that inspects a payload-less enum and reuses it afterward
            // (a bare-tag inspection is not a consume).
            let variant_drop_inert = !is_copy(&sc_ty, self.items)
                && arm_diverges(&arm.body)
                && match &arm.pattern.kind {
                    PatKind::Variant { variant, .. } => einfo
                        .variants
                        .iter()
                        .find(|v| &v.name == variant)
                        .map(|v| v.payload.iter().all(|t| !needs_drop(t, self.items)))
                        .unwrap_or(false),
                    _ => false,
                };
            let moves = hold == HoldMode::Owned
                && (binds.iter().any(|bd| bd.mode == BindMode::Move) || variant_drop_inert);
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
                // A pattern binding introduces a FRESH local; it is an
                // initialization, not a reassignment of a prior value. Declaring
                // it (Uninit) before the binding assignment prevents a loop that
                // re-enters this arm from seeing a stale `MaybeInit` (the binding
                // of a needs-drop item each turn otherwise tripped E0309 on the
                // reassignment-drop rule — the `for`-desugar case, 2026-07-08).
                self.emit(&Some(Place::local(bd.name.clone())), Access::Decl, bd.span);
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
            self.check_arm_guard(arm);
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

    /// Type-check a match arm's optional `if EXPR` guard: the guard must type as
    /// `bool` and is evaluated with the arm's pattern bindings in scope (design
    /// 0001 §8.2, extended). Called with the arm's scope already pushed.
    fn check_arm_guard(&mut self, arm: &MatchArm) {
        if let Some(guard) = &arm.guard {
            let gt = self.check_expr(guard, Use::Value);
            self.expect_bool(&gt, guard.span, "a match guard");
        }
    }

    /// Type-check an integer-scrutinee `match` (design 0001 §8.2, extended for
    /// integer-literal patterns). Each literal is typed against the scrutinee's
    /// integer type; the match is exhaustive only with a `_`/binding catch-all
    /// (integer literals can never enumerate the type); a repeated literal is a
    /// dead arm. Lowers to a compare-and-branch chain (no new MIR construct).
    fn check_int_match(
        &mut self,
        scrut: &Expr,
        arms: &[MatchArm],
        span: Span,
        sty: ScalarTy,
        sc_place: &Option<Place>,
    ) -> Type {
        let sc_ty = Type::Scalar(sty);
        let mut seen: Vec<(i128, i128)> = Vec::new();
        let mut has_catchall = false;
        for arm in arms {
            match &arm.pattern.kind {
                PatKind::IntLit { value, negative, suffix } => {
                    if let Some(v) =
                        self.check_int_endpoint(*value, *negative, *suffix, sty, arm.pattern.span)
                    {
                        self.record_int_interval(&mut seen, v, v, arm.pattern.span, arm.guard.is_some());
                    }
                }
                PatKind::IntRange {
                    lo_value,
                    lo_negative,
                    lo_suffix,
                    hi_value,
                    hi_negative,
                    hi_suffix,
                    inclusive,
                } => {
                    let lo = self
                        .check_int_endpoint(*lo_value, *lo_negative, *lo_suffix, sty, arm.pattern.span);
                    let hi = self
                        .check_int_endpoint(*hi_value, *hi_negative, *hi_suffix, sty, arm.pattern.span);
                    if let (Some(lo), Some(hi)) = (lo, hi) {
                        let valid = if *inclusive { lo <= hi } else { lo < hi };
                        if !valid {
                            self.diags.push(
                                Diag::error(
                                    "E0715",
                                    format!(
                                        "range pattern lower bound `{lo}` exceeds upper bound `{hi}`"
                                    ),
                                    arm.pattern.span,
                                )
                                .with_note(
                                    "a range pattern requires `lo <= hi` for `..=` (or `lo < hi` for `..`)",
                                    None,
                                ),
                            );
                        } else {
                            let last = if *inclusive { hi } else { hi - 1 };
                            self.record_int_interval(&mut seen, lo, last, arm.pattern.span, arm.guard.is_some());
                        }
                    }
                }
                // A guarded catch-all may not fire, so it does NOT make the match
                // exhaustive (design 0001 §8.2, extended).
                PatKind::Wildcard | PatKind::Binding(_) => {
                    if arm.guard.is_none() {
                        has_catchall = true;
                    }
                }
                PatKind::Variant { .. } => {
                    self.diags.push(Diag::error(
                        "E0606",
                        format!(
                            "variant pattern cannot match integer scrutinee `{}`",
                            scalar_name(sty)
                        ),
                        arm.pattern.span,
                    ));
                }
            }
        }
        if !has_catchall {
            self.diags.push(
                Diag::error(
                    "E0601",
                    "non-exhaustive match: an integer match must have a `_` wildcard arm".to_string(),
                    span,
                )
                .with_note(
                    "integer literals can never enumerate the whole type; add a `_` catch-all",
                    None,
                ),
            );
        }

        // The scrutinee is read once at the match head.
        self.emit_place_action(sc_place, Use::ReadOnly, &sc_ty, scrut.span);

        let c0 = match self.cur_get() {
            Some(b) => b,
            None => {
                let mut ty = Type::Never;
                for arm in arms {
                    self.push_scope();
                    if let PatKind::Binding(name) = &arm.pattern.kind {
                        self.add_local(name, sc_ty.clone(), true);
                    }
                    self.check_arm_guard(arm);
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
            let b = self.new_block();
            arm_bbs.push(b);
            self.cur_set(Some(b));
            self.push_scope();
            // A binding arm binds the whole (Copy) integer value: a fresh local,
            // initialized from the scrutinee read.
            if let PatKind::Binding(name) = &arm.pattern.kind {
                self.add_local(name, sc_ty.clone(), true);
                self.emit(&Some(Place::local(name.clone())), Access::Decl, arm.pattern.span);
                self.emit(
                    &Some(Place::local(name.clone())),
                    Access::Assign { needs_drop: false, box_paths: Vec::new() },
                    arm.pattern.span,
                );
            }
            self.check_arm_guard(arm);
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

    /// Type-check one integer endpoint of a literal/range pattern against the
    /// scrutinee type: a present suffix must match it (E0606) and the value must
    /// be in range (E0709). Returns the endpoint's signed value, or `None` when
    /// out of range (so the caller skips overlap accounting for a dead value).
    fn check_int_endpoint(
        &mut self,
        value: u64,
        negative: bool,
        suffix: Option<ScalarTy>,
        sty: ScalarTy,
        span: Span,
    ) -> Option<i128> {
        if let Some(suf) = suffix {
            if suf != sty {
                self.diags.push(Diag::error(
                    "E0606",
                    format!(
                        "literal pattern is `{}` but scrutinee is `{}`",
                        scalar_name(suf),
                        scalar_name(sty)
                    ),
                    span,
                ));
            }
        }
        let v = int_pat_value(value, negative);
        let (min, max) = scalar_range(sty);
        if v < min || v > max {
            self.diags.push(
                Diag::error(
                    "E0709",
                    format!(
                        "integer literal `{}{}` is out of range for `{}`",
                        if negative { "-" } else { "" },
                        value,
                        scalar_name(sty)
                    ),
                    span,
                )
                .with_note(
                    "an over-range literal is rejected at compile time, never a runtime fault (spec 01 §3.3)",
                    None,
                ),
            );
            None
        } else {
            Some(v)
        }
    }

    /// Record the inclusive integer interval `[lo, hi]` an arm covers, emitting
    /// E0602 when it overlaps a value an earlier arm already covers (a dead,
    /// unreachable arm — this is the duplicate-literal check generalized to
    /// ranges). Non-overlapping intervals are accumulated for later arms.
    fn record_int_interval(&mut self, seen: &mut Vec<(i128, i128)>, lo: i128, hi: i128, span: Span, guarded: bool) {
        // `seen` accumulates only the intervals of UNGUARDED arms: a guarded arm's
        // guard may fail at runtime, so it neither shadows a later arm nor is
        // shadowed retroactively (design 0001 §8.2, extended). An arm is still
        // flagged unreachable when an earlier UNGUARDED arm already covers it.
        if seen.iter().any(|&(a, b)| lo <= b && a <= hi) {
            self.diags.push(Diag::error(
                "E0602",
                "overlapping pattern — this arm is unreachable".to_string(),
                span,
            ));
        } else if !guarded {
            seen.push((lo, hi));
        }
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

/// Does this arm body definitely diverge (never fall through to code after the
/// match)? A conservative syntactic check: a `return`/`break`/`continue`/`panic`,
/// or a block ending in one, or an `if`/`match` whose every branch diverges.
/// Used to decide whether an owned drop-inert match arm consumes the scrutinee
/// (design 0009 §1.2 forwarding `map`).
fn arm_diverges(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Return(_) | ExprKind::Break | ExprKind::Continue | ExprKind::Panic(_) => true,
        ExprKind::Paren(i) => arm_diverges(i),
        ExprKind::Block(b) => b.stmts.last().map(|s| match &s.kind {
            StmtKind::Expr(e) => arm_diverges(e),
            _ => false,
        }).unwrap_or(false),
        ExprKind::If { then_blk, else_blk, .. } => {
            let then_div = then_blk.stmts.last().map(|s| matches!(&s.kind, StmtKind::Expr(e) if arm_diverges(e))).unwrap_or(false);
            match else_blk {
                Some(x) => then_div && arm_diverges(x),
                None => false,
            }
        }
        ExprKind::Match { arms, .. } => !arms.is_empty() && arms.iter().all(|a| arm_diverges(&a.body)),
        _ => false,
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


/// Strip enclosing parentheses for structural inspection.
fn strip_paren(e: &Expr) -> &Expr {
    match &e.kind {
        ExprKind::Paren(i) => strip_paren(i),
        _ => e,
    }
}

/// Peel single-place borrow/box layers off a type (the read auto-deref).
fn peel_borrow(mut ty: Type) -> Type {
    loop {
        match ty {
            Type::Borrow(x) | Type::BorrowMut(x) | Type::Box(x) => ty = *x,
            other => return other,
        }
    }
}

/// The inclusive value range of an integer scalar type.
fn scalar_range(s: ScalarTy) -> (i128, i128) {
    match s {
        ScalarTy::I8 => (i8::MIN as i128, i8::MAX as i128),
        ScalarTy::I16 => (i16::MIN as i128, i16::MAX as i128),
        ScalarTy::I32 => (i32::MIN as i128, i32::MAX as i128),
        ScalarTy::I64 | ScalarTy::Isize => (i64::MIN as i128, i64::MAX as i128),
        ScalarTy::U8 => (0, u8::MAX as i128),
        ScalarTy::U16 => (0, u16::MAX as i128),
        ScalarTy::U32 => (0, u32::MAX as i128),
        ScalarTy::U64 | ScalarTy::Usize => (0, u64::MAX as i128),
        // `f64` never reaches an integer-literal fit check; a null range.
        ScalarTy::Bool | ScalarTy::Unit | ScalarTy::F64 | ScalarTy::F32 => (0, 0),
    }
}

fn scalar_fits(v: i128, s: ScalarTy) -> bool {
    let (min, max) = scalar_range(s);
    v >= min && v <= max
}

/// Fold an expression to a compile-time integer constant, if it is one.
fn const_int(e: &Expr) -> Option<i128> {
    match &e.kind {
        ExprKind::IntLit { value, .. } => Some(*value as i128),
        ExprKind::NegIntLit { value, .. } => Some(-(*value as i128)),
        ExprKind::Paren(i) => const_int(i),
        ExprKind::Unary { op: UnOp::Neg, expr } => const_int(expr).map(|x| -x),
        _ => None,
    }
}

/// The generic-enum head and concrete type arguments a (possibly borrowed) type
/// denotes, for monomorphizing match patterns (design 0007 §5).
fn generic_enum_of(ty: &Type) -> Option<(String, Vec<Type>)> {
    match ty {
        Type::App(n, args) => Some((n.clone(), args.clone())),
        Type::Borrow(i) | Type::BorrowMut(i) | Type::Box(i) => generic_enum_of(i),
        _ => None,
    }
}
