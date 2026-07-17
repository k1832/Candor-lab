//! A span-free, deterministic canonical renderer for AST items (design 0008 §2).
//!
//! The interface artifact's two hashes must be **body-vs-signature separable**
//! and **span-independent**: editing a body must not perturb the signature hash,
//! and a byte-offset shift (a comment above a `pub` item) must not perturb any
//! hash. `real::emit` cannot serve — it `unreachable!`s on generic items — so
//! this module renders the full AST (generics included) to a compact canonical
//! string that carries no `Span`.
//!
//! Two projections per item:
//! * [`item_signature`] — the exported contract only (types, modes, regions,
//!   effects, contracts, struct fields, enum variants, method signatures); **no
//!   bodies**. This feeds the signature hash (analysis-invalidation tier).
//! * [`item_body`] — the item's checked body (fn bodies, drop hooks, static
//!   initializers, impl-method bodies). This feeds the codegen hash together
//!   with the signature (codegen-invalidation tier). It stands in for the
//!   serialized checked-MIR body 0008 §2.4 names: the prototype has no
//!   generic-MIR emission (Stage A/B lower *monomorphized* programs), so the
//!   source-derived body is the honest codegen-input proxy.

use crate::ast::*;

/// Canonical, span-free rendering of a `pub` item's **signature** (no body).
pub fn item_signature(item: &Item) -> String {
    let mut p = Printer::default();
    p.item_sig(item);
    p.out
}

/// Canonical, span-free rendering of an item's **body** (empty when it has none).
pub fn item_body(item: &Item) -> String {
    let mut p = Printer::default();
    p.item_body(item);
    p.out
}

#[derive(Default)]
struct Printer {
    out: String,
}

impl Printer {
    fn push(&mut self, s: &str) {
        self.out.push_str(s);
    }

    // --- signature projection ------------------------------------------------

    fn item_sig(&mut self, item: &Item) {
        match item {
            Item::Struct(s) => {
                if s.copy {
                    self.push("copy ");
                }
                self.push("struct ");
                self.push(&s.name);
                self.type_params(&s.type_params);
                self.push("{");
                for f in &s.fields {
                    self.push(&f.name);
                    self.push(":");
                    self.ty(&f.ty);
                    self.push(";");
                }
                self.push("}");
                // Whether a struct has a drop hook is a semantic fact of its type
                // (its drop behavior), so its *presence* is part of the signature;
                // the hook's *body* is codegen (rendered by `item_body`).
                if s.drop_hook.is_some() {
                    self.push("drop;");
                }
            }
            Item::Enum(e) => {
                if e.copy {
                    self.push("copy ");
                }
                self.push("enum ");
                self.push(&e.name);
                self.type_params(&e.type_params);
                self.push("{");
                for v in &e.variants {
                    if v.ok {
                        self.push("ok ");
                    }
                    self.push(&v.name);
                    self.push("(");
                    for t in &v.payload {
                        self.ty(t);
                        self.push(",");
                    }
                    self.push(");");
                }
                self.push("}");
            }
            Item::Fn(f) => self.fn_sig(f),
            Item::Static(s) => {
                self.push("static ");
                self.push(&s.name);
                self.push(":");
                self.ty(&s.ty);
            }
            Item::Interface(i) => {
                self.push("interface ");
                self.push(&i.name);
                self.type_params(&i.type_params);
                if let Some(a) = &i.assoc_type {
                    self.push(" type ");
                    self.push(a);
                }
                self.push("{");
                for m in &i.methods {
                    self.method_sig(m);
                    self.push(";");
                }
                self.push("}");
            }
            Item::Impl(im) => {
                self.push("impl");
                self.type_params(&im.type_params);
                self.push(" ");
                self.push(&im.iface);
                if !im.iface_args.is_empty() {
                    self.push("[");
                    for a in &im.iface_args {
                        self.ty(a);
                        self.push(",");
                    }
                    self.push("]");
                }
                self.push(" for ");
                self.ty(&im.target);
                if let Some((n, t)) = &im.assoc_binding {
                    self.push(" type ");
                    self.push(n);
                    self.push("=");
                    self.ty(t);
                }
                self.push("{");
                for m in &im.methods {
                    self.fn_sig(m);
                    self.push(";");
                }
                self.push("}");
            }
            Item::Extern(eb) => {
                self.push("extern ");
                self.push(&eb.abi);
                self.push("{");
                for ef in &eb.fns {
                    self.push(&ef.name);
                    self.push("(");
                    self.params(&ef.params);
                    self.push(")");
                    self.ret(&ef.ret);
                    self.push(";");
                }
                self.push("}");
            }
            Item::Export(ex) => {
                self.push("export ");
                self.push(&ex.abi);
                self.push(" ");
                self.push(&ex.symbol);
                self.push("(");
                self.params(&ex.params);
                self.push(")");
                self.ret(&ex.ret);
                self.push("=");
                self.push(&ex.candor_fn);
            }
        }
    }

    fn fn_sig(&mut self, f: &FnDecl) {
        self.push("fn ");
        self.push(&f.name);
        self.type_params(&f.type_params);
        if !f.regions.is_empty() {
            self.push("<");
            for r in &f.regions {
                self.push(r);
                self.push(",");
            }
            self.push(">");
        }
        self.push("(");
        self.params(&f.params);
        self.push(")");
        if f.alloc {
            self.push(" alloc");
        }
        if f.foreign {
            self.push(" foreign");
        }
        self.ret(&f.ret);
        for r in &f.requires {
            self.push(" requires(");
            self.expr(r);
            self.push(")");
        }
        for e in &f.ensures {
            self.push(" ensures(");
            self.expr(e);
            self.push(")");
        }
    }

    fn method_sig(&mut self, m: &MethodSig) {
        self.push("fn ");
        self.push(&m.name);
        self.push("(");
        if m.has_self {
            self.push(mode_str(m.self_mode));
            self.push(" self,");
        }
        self.params(&m.params);
        self.push(")");
        if m.alloc {
            self.push(" alloc");
        }
        self.ret(&m.ret);
    }

    fn type_params(&mut self, tps: &[TypeParam]) {
        if tps.is_empty() {
            return;
        }
        self.push("[");
        for tp in tps {
            self.push(&tp.name);
            if !tp.bounds.is_empty() {
                self.push(":");
                for b in &tp.bounds {
                    self.push(b);
                    self.push("+");
                }
            }
            self.push(",");
        }
        self.push("]");
    }

    fn params(&mut self, params: &[Param]) {
        for p in params {
            self.push(&p.name);
            self.push(":");
            self.push(mode_str(p.mode));
            if let Some(r) = &p.region {
                self.push("[");
                self.push(r);
                self.push("]");
            }
            self.push(" ");
            self.ty(&p.ty);
            self.push(",");
        }
    }

    fn ret(&mut self, ret: &Option<RetTy>) {
        if let Some(r) = ret {
            self.push("->");
            if let Some(b) = r.borrow {
                self.push(match b {
                    BorrowKind::Shared => "read",
                    BorrowKind::Exclusive => "write",
                });
            }
            if let Some(rg) = &r.region {
                self.push("[");
                self.push(rg);
                self.push("]");
            }
            self.ty(&r.ty);
        }
    }

    fn ty(&mut self, ty: &Ty) {
        match &ty.kind {
            TyKind::Scalar(s) => self.push(&format!("{s:?}")),
            TyKind::Named(n) => self.push(n),
            TyKind::App { name, args } => {
                self.push(name);
                self.push("[");
                for a in args {
                    self.ty(a);
                    self.push(",");
                }
                self.push("]");
            }
            TyKind::Proj { base, assoc } => {
                self.push(base);
                self.push("::");
                self.push(assoc);
            }
            TyKind::Array { size, elem } => {
                self.push("[");
                self.expr(size);
                self.push("]");
                self.ty(elem);
            }
            TyKind::Slice(t) => {
                self.push("[]");
                self.ty(t);
            }
            TyKind::SliceMut(t) => {
                self.push("mut[]");
                self.ty(t);
            }
            TyKind::RawPtr(t) => {
                self.push("rawptr ");
                self.ty(t);
            }
            TyKind::Box(t) => {
                self.push("box ");
                self.ty(t);
            }
            TyKind::BoxResult(t) => {
                self.push("boxresult ");
                self.ty(t);
            }
            TyKind::Borrow(t) => {
                self.push("read ");
                self.ty(t);
            }
            TyKind::BorrowMut(t) => {
                self.push("write ");
                self.ty(t);
            }
            TyKind::FnPtr(fp) => {
                self.push("fn(");
                for p in &fp.params {
                    self.push(mode_str(p.mode));
                    self.push(" ");
                    self.ty(&p.ty);
                    self.push(",");
                }
                self.push(")");
                if fp.alloc {
                    self.push("alloc");
                }
                if fp.foreign {
                    self.push("foreign");
                }
                self.push("->");
                self.ty(&fp.ret);
            }
        }
    }

    // --- body projection -----------------------------------------------------

    fn item_body(&mut self, item: &Item) {
        match item {
            Item::Fn(f) => self.block(&f.body),
            Item::Static(s) => self.expr(&s.value),
            Item::Struct(s) => {
                if let Some(b) = &s.drop_hook {
                    self.push("drop");
                    self.block(b);
                }
            }
            Item::Impl(im) => {
                for m in &im.methods {
                    self.push(&m.name);
                    self.block(&m.body);
                }
            }
            // Enums, interfaces, and foreign declarations have no checked body.
            Item::Enum(_) | Item::Interface(_) | Item::Extern(_) | Item::Export(_) => {}
        }
    }

    fn block(&mut self, b: &Block) {
        self.push("{");
        for s in &b.stmts {
            self.stmt(s);
            self.push(";");
        }
        self.push("}");
    }

    fn stmt(&mut self, s: &Stmt) {
        match &s.kind {
            StmtKind::Let { mutable, name, ty, init } => {
                self.push(if *mutable { "let mut " } else { "let " });
                self.push(name);
                if let Some(t) = ty {
                    self.push(":");
                    self.ty(t);
                }
                if let Some(e) = init {
                    self.push("=");
                    self.expr(e);
                }
            }
            StmtKind::Assign { target, value } => {
                self.expr(target);
                self.push("=");
                self.expr(value);
            }
            StmtKind::Expr(e) => self.expr(e),
        }
    }

    fn expr(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::For { .. } => unreachable!("`for` is surface-only (formatter); the pipeline desugars it at parse (design 0009 §4.2)"),
            ExprKind::Scope(b) => {
                self.push("scope");
                self.block(b);
            }
            ExprKind::Spawn(c) => {
                self.push("spawn");
                self.expr(c);
            }
            ExprKind::IntLit { value, suffix } => {
                self.push(&value.to_string());
                self.suffix(suffix);
            }
            ExprKind::NegIntLit { value, suffix } => {
                self.push("-");
                self.push(&value.to_string());
                self.suffix(suffix);
            }
            ExprKind::FloatLit { bits, ty } => {
                self.push("f");
                self.push(&bits.to_string());
                if *ty == crate::token::ScalarTy::F32 {
                    self.push("f32");
                }
            }
            ExprKind::StrLit(s) => {
                self.push("\"");
                self.push(s);
                self.push("\"");
            }
            ExprKind::BytesLit(s) => {
                self.push("b\"");
                self.push(s);
                self.push("\"");
            }
            ExprKind::BoolLit(b) => self.push(if *b { "true" } else { "false" }),
            ExprKind::Ident(n) => self.push(n),
            ExprKind::Unary { op, expr } => {
                self.push(&format!("{op:?}"));
                self.expr(expr);
            }
            ExprKind::Prefix { op, expr } => {
                self.push(&format!("{op:?}"));
                self.expr(expr);
            }
            ExprKind::Binary { op, lhs, rhs } => {
                self.push("(");
                self.expr(lhs);
                self.push(&format!("{op:?}"));
                self.expr(rhs);
                self.push(")");
            }
            ExprKind::Call { callee, args } => {
                self.expr(callee);
                self.push("(");
                for a in args {
                    self.expr(a);
                    self.push(",");
                }
                self.push(")");
            }
            ExprKind::OutArg(e) => {
                self.push("out ");
                self.expr(e);
            }
            ExprKind::Field { base, field, .. } => {
                self.expr(base);
                self.push(".");
                self.push(field);
            }
            ExprKind::Index { base, index } => {
                self.expr(base);
                self.push("[");
                self.expr(index);
                self.push("]");
            }
            ExprKind::Conv { ty, expr } => {
                self.push("conv ");
                self.ty(ty);
                self.push(" ");
                self.expr(expr);
            }
            ExprKind::Bitcast { ty, expr } => {
                self.push("bitcast ");
                self.ty(ty);
                self.push(" ");
                self.expr(expr);
            }
            ExprKind::ArrayLit(v) => {
                self.push("[");
                for e in v {
                    self.expr(e);
                    self.push(",");
                }
                self.push("]");
            }
            ExprKind::ArrayRepeat { value, size } => {
                self.push("[");
                self.expr(value);
                self.push(";");
                self.expr(size);
                self.push("]");
            }
            ExprKind::StructLit { name, fields } => {
                self.push(name);
                self.push("{");
                for f in fields {
                    self.push(&f.name);
                    self.push(":");
                    self.expr(&f.value);
                    self.push(",");
                }
                self.push("}");
            }
            ExprKind::EnumCtor { enum_name, variant, args } => {
                self.push(enum_name);
                self.push("::");
                self.push(variant);
                self.push("(");
                for a in args {
                    self.expr(a);
                    self.push(",");
                }
                self.push(")");
            }
            ExprKind::CastPtr { ty, arg } => {
                self.push("castptr ");
                self.ty(ty);
                self.expr(arg);
            }
            ExprKind::AddrToPtr { ty, arg } => {
                self.push("addrptr ");
                self.ty(ty);
                self.expr(arg);
            }
            ExprKind::PtrNull { ty } => {
                self.push("null ");
                self.ty(ty);
            }
            ExprKind::Offsetof { ty, field } => {
                self.push("offsetof ");
                self.ty(ty);
                self.push(field);
            }
            ExprKind::FieldPtr { ptr, field } => {
                self.push("fieldptr ");
                self.expr(ptr);
                self.push(field);
            }
            ExprKind::Sizeof(ty) => {
                self.push("sizeof ");
                self.ty(ty);
            }
            ExprKind::Alignof(ty) => {
                self.push("alignof ");
                self.ty(ty);
            }
            ExprKind::Block(b) => self.block(b),
            ExprKind::If { cond, then_blk, else_blk } => {
                self.push("if ");
                self.expr(cond);
                self.block(then_blk);
                if let Some(e) = else_blk {
                    self.push("else ");
                    self.expr(e);
                }
            }
            ExprKind::Match { scrutinee, arms } => {
                self.push("match ");
                self.expr(scrutinee);
                self.push("{");
                for arm in arms {
                    self.pattern(&arm.pattern);
                    self.push("=>");
                    self.expr(&arm.body);
                    self.push(",");
                }
                self.push("}");
            }
            ExprKind::Loop(b) => {
                self.push("loop");
                self.block(b);
            }
            ExprKind::While { cond, body } => {
                self.push("while ");
                self.expr(cond);
                self.block(body);
            }
            ExprKind::Unsafe { justification, body } => {
                self.push("unsafe(");
                self.push(justification);
                self.push(")");
                self.block(body);
            }
            ExprKind::Wrapping(b) => {
                self.push("wrapping");
                self.block(b);
            }
            ExprKind::Saturating(b) => {
                self.push("saturating");
                self.block(b);
            }
            ExprKind::Return(e) => {
                self.push("return");
                if let Some(e) = e {
                    self.push(" ");
                    self.expr(e);
                }
            }
            ExprKind::Break => self.push("break"),
            ExprKind::Continue => self.push("continue"),
            ExprKind::Assert(e) => {
                self.push("assert ");
                self.expr(e);
            }
            ExprKind::Panic(e) => {
                self.push("panic ");
                self.expr(e);
            }
            ExprKind::Result => self.push("result"),
            ExprKind::Paren(e) => {
                self.push("(");
                self.expr(e);
                self.push(")");
            }
            ExprKind::Try(e) => {
                self.expr(e);
                self.push("?");
            }
            ExprKind::GenericVal { name, ty_args } => {
                self.push(name);
                self.push("::[");
                for a in ty_args {
                    self.ty(a);
                    self.push(",");
                }
                self.push("]");
            }
        }
    }

    fn suffix(&mut self, suffix: &Option<crate::token::ScalarTy>) {
        if let Some(s) = suffix {
            self.push(&format!("_{s:?}"));
        }
    }

    fn pattern(&mut self, p: &Pattern) {
        match &p.kind {
            PatKind::Wildcard => self.push("_"),
            PatKind::Binding(n) => self.push(n),
            PatKind::Variant { enum_name, variant, sub } => {
                self.push(enum_name);
                self.push("::");
                self.push(variant);
                self.push("(");
                for s in sub {
                    self.pattern(s);
                    self.push(",");
                }
                self.push(")");
            }
            PatKind::IntLit { value, negative, suffix } => {
                if *negative {
                    self.push("-");
                }
                self.push(&value.to_string());
                self.suffix(suffix);
            }
            PatKind::IntRange { lo_value, lo_negative, lo_suffix, hi_value, hi_negative, hi_suffix, inclusive } => {
                if *lo_negative {
                    self.push("-");
                }
                self.push(&lo_value.to_string());
                self.suffix(lo_suffix);
                self.push(if *inclusive { "..=" } else { ".." });
                if *hi_negative {
                    self.push("-");
                }
                self.push(&hi_value.to_string());
                self.suffix(hi_suffix);
            }
        }
    }
}

fn mode_str(m: ParamMode) -> &'static str {
    match m {
        ParamMode::Take => "take",
        ParamMode::Read => "read",
        ParamMode::Write => "write",
        ParamMode::Out => "out",
    }
}
