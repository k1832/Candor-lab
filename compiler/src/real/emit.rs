//! AST -> real-syntax (`.cnr`) pretty-printer: the P15 migrator's core (design
//! 0006 §5; spec 01/02). The migrator parses a throwaway (`.cn`) program with the
//! existing front-end and re-emits the SHARED [`crate::ast`] in real surface
//! syntax. Because it walks the same AST the throwaway parser produced, semantic
//! fidelity is guaranteed by construction: the emitter only *re-spells*.
//!
//! Mechanical rows applied here (design 0006 §5, the "yes" rows):
//!   * borrow/slice type keywords: `slice T`->`[T]`, `slice_mut T`->`write [T]`,
//!     `borrow T`->`read T`, `borrow_mut T`->`write T`;
//!   * dereference: prefix `deref` -> postfix `.*`, with **read-only auto-deref**
//!     (a `.*` before a field/index on a read path is dropped; a write target and
//!     a bare-value deref keep `.*`) — spec 02 §6.3/§9.3;
//!   * explicit reborrow collapse: `read (deref b)` / `write (deref b)` in an
//!     argument position -> bare `b` (design 0005);
//!   * `case P => e` -> `P => e`; `conv T (e)` -> `conv T e`.
//!
//! Author-assisted rows (design 0006 §5, the "author-assisted" rows) are NOT
//! rewritten; a `// MIGRATE:` marker is emitted where one is detectable — namely
//! a two-arm `BoxResult` match (the `box(a, v)?` candidate). Result-shaped enums
//! need no `ok` marker unless `?` is used on them (spec 02 §2.2), and the migrator
//! never introduces `?`, so plain emission parses.

use crate::ast::*;
use crate::token::ScalarTy;

// Binding powers (bigger binds tighter), mirroring spec 02 §6.1. Used to add
// exactly the parentheses the canonical precedence makes load-bearing.
pub(crate) const BP_MIN: u32 = 0;
pub(crate) const BP_OR: u32 = 30;
pub(crate) const BP_AND: u32 = 35;
pub(crate) const BP_CMP: u32 = 40;
pub(crate) const BP_BITOR: u32 = 45;
pub(crate) const BP_BITXOR: u32 = 50;
pub(crate) const BP_BITAND: u32 = 55;
pub(crate) const BP_SHIFT: u32 = 60;
pub(crate) const BP_ADD: u32 = 65;
pub(crate) const BP_MUL: u32 = 70;
pub(crate) const BP_PREFIX: u32 = 80;
pub(crate) const BP_POSTFIX: u32 = 90;
pub(crate) const BP_ATOM: u32 = 100;

/// Emit a whole program in real (`.cnr`) surface syntax.
pub fn emit_program(p: &Program) -> String {
    let mut e = Emitter { buf: String::new(), indent: 0 };
    for (i, item) in p.items.iter().enumerate() {
        if i > 0 {
            e.buf.push('\n');
        }
        e.emit_item(item);
    }
    if !e.buf.ends_with('\n') {
        e.buf.push('\n');
    }
    e.buf
}

struct Emitter {
    buf: String,
    indent: usize,
}

impl Emitter {
    fn push(&mut self, s: &str) {
        self.buf.push_str(s);
    }

    fn indent_str(&mut self) {
        for _ in 0..self.indent {
            self.buf.push_str("    ");
        }
    }

    // ----- items ----------------------------------------------------------
    fn emit_item(&mut self, item: &Item) {
        match item {
            Item::Struct(s) => self.emit_struct(s),
            Item::Enum(e) => self.emit_enum(e),
            Item::Fn(f) => self.emit_fn(f),
            Item::Static(s) => self.emit_static(s),
            // The throwaway front-end (the migrator's input) never produces
            // generic items (design 0007 is real-syntax only).
            Item::Interface(_) | Item::Impl(_) => {
                unreachable!("throwaway syntax has no generic items")
            }
            Item::Extern(_) | Item::Export(_) => {
                unreachable!("throwaway syntax has no foreign items (design 0011)")
            }
        }
    }

    fn emit_struct(&mut self, s: &StructDecl) {
        if s.copy {
            self.push("copy ");
        }
        self.push("struct ");
        self.push(&s.name);
        if s.fields.is_empty() {
            self.push(" {}");
        } else {
            self.push(" {\n");
            self.indent += 1;
            for f in &s.fields {
                self.indent_str();
                self.push(&f.name);
                self.push(": ");
                self.emit_type(&f.ty);
                self.push(",\n");
            }
            self.indent -= 1;
            self.push("}");
        }
        if let Some(hook) = &s.drop_hook {
            self.push(" drop(write self) ");
            self.emit_block(hook);
        }
        self.push("\n");
    }

    fn emit_enum(&mut self, e: &EnumDecl) {
        if e.copy {
            self.push("copy ");
        }
        self.push("enum ");
        self.push(&e.name);
        if e.variants.is_empty() {
            self.push(" {}");
        } else {
            self.push(" {\n");
            self.indent += 1;
            for v in &e.variants {
                self.indent_str();
                if v.ok {
                    self.push("ok ");
                }
                self.push(&v.name);
                if !v.payload.is_empty() {
                    self.push("(");
                    for (i, t) in v.payload.iter().enumerate() {
                        if i > 0 {
                            self.push(", ");
                        }
                        self.emit_type(t);
                    }
                    self.push(")");
                }
                self.push(",\n");
            }
            self.indent -= 1;
            self.push("}");
        }
        self.push("\n");
    }

    fn emit_static(&mut self, s: &StaticDecl) {
        self.push("static ");
        self.push(&s.name);
        self.push(": ");
        self.emit_type(&s.ty);
        self.push(" = ");
        self.emit_expr(&s.value, BP_MIN, false);
        self.push(";\n");
    }

    fn emit_fn(&mut self, f: &FnDecl) {
        self.push("fn ");
        self.push(&f.name);
        if !f.regions.is_empty() {
            self.push("[");
            let decls: Vec<String> = f.regions.iter().map(|r| format!("region {r}")).collect();
            self.push(&decls.join(", "));
            self.push("]");
        }
        self.push("(");
        for (i, p) in f.params.iter().enumerate() {
            if i > 0 {
                self.push(", ");
            }
            self.push(&p.name);
            self.push(": ");
            self.emit_mode_type(p.mode, p.region.as_deref(), &p.ty);
        }
        self.push(")");
        if f.alloc {
            self.push(" alloc");
        }
        for r in &f.requires {
            self.push(" requires(");
            self.emit_expr(r, BP_MIN, false);
            self.push(")");
        }
        for en in &f.ensures {
            self.push(" ensures(");
            self.emit_expr(en, BP_MIN, false);
            self.push(")");
        }
        if let Some(ret) = &f.ret {
            self.push(" -> ");
            self.emit_ret_ty(ret);
        }
        self.push(" ");
        self.emit_block(&f.body);
        self.push("\n");
    }

    // ----- signature pieces -----------------------------------------------
    /// A parameter's `Mode? Type` (spec 02 §4). Take-mode emits only the type
    /// (an exclusive slice's `write` rides on the *type*, `write [T]`); the borrow
    /// modes wear their keyword and optional region.
    fn emit_mode_type(&mut self, mode: ParamMode, region: Option<&str>, ty: &Ty) {
        match mode {
            ParamMode::Take => self.emit_type(ty),
            ParamMode::Out => {
                self.push("out ");
                self.emit_type(ty);
            }
            ParamMode::Read => {
                self.push("read");
                self.emit_region(region);
                self.push(" ");
                self.emit_type(ty);
            }
            ParamMode::Write => {
                self.push("write");
                self.emit_region(region);
                self.push(" ");
                self.emit_type(ty);
            }
        }
    }

    fn emit_ret_ty(&mut self, ret: &RetTy) {
        match ret.borrow {
            None => self.emit_type(&ret.ty),
            Some(BorrowKind::Shared) => {
                self.push("read");
                self.emit_region(ret.region.as_deref());
                self.push(" ");
                self.emit_type(&ret.ty);
            }
            Some(BorrowKind::Exclusive) => {
                self.push("write");
                self.emit_region(ret.region.as_deref());
                self.push(" ");
                self.emit_type(&ret.ty);
            }
        }
    }

    fn emit_region(&mut self, region: Option<&str>) {
        if let Some(r) = region {
            self.push("[");
            self.push(r);
            self.push("]");
        }
    }

    // ----- types ----------------------------------------------------------
    fn emit_type(&mut self, ty: &Ty) {
        match &ty.kind {
            TyKind::Scalar(sc) => self.push(scalar_kw(*sc)),
            TyKind::Named(n) => self.push(n),
            TyKind::App { .. } => unreachable!("throwaway syntax has no generic types"),
            TyKind::Proj { base, assoc } => {
                self.push(base);
                self.push("::");
                self.push(assoc);
            }
            TyKind::Slice(t) => {
                self.push("[");
                self.emit_type(t);
                self.push("]");
            }
            TyKind::SliceMut(t) => {
                self.push("write [");
                self.emit_type(t);
                self.push("]");
            }
            TyKind::Borrow(t) => {
                self.push("read ");
                self.emit_type(t);
            }
            TyKind::BorrowMut(t) => {
                self.push("write ");
                self.emit_type(t);
            }
            TyKind::RawPtr(t) => {
                self.push("rawptr ");
                self.emit_type(t);
            }
            TyKind::Box(t) => {
                self.push("Box ");
                self.emit_type(t);
            }
            TyKind::BoxResult(t) => {
                self.push("BoxResult ");
                self.emit_type(t);
            }
            TyKind::Array { size, elem } => {
                self.push("[");
                self.emit_expr(size, BP_MIN, false);
                self.push("]");
                self.emit_type(elem);
            }
            TyKind::FnPtr(fp) => {
                self.push("fn(");
                for (i, p) in fp.params.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    if let Some(n) = &p.name {
                        self.push(n);
                        self.push(": ");
                    }
                    self.emit_mode_type(p.mode, p.region.as_deref(), &p.ty);
                }
                self.push(")");
                if fp.alloc {
                    self.push(" alloc");
                }
                self.push(" -> ");
                self.emit_type(&fp.ret);
            }
        }
    }

    // ----- blocks & statements --------------------------------------------
    fn emit_block(&mut self, b: &Block) {
        if b.stmts.is_empty() {
            self.push("{}");
            return;
        }
        self.push("{\n");
        self.indent += 1;
        for st in &b.stmts {
            self.indent_str();
            self.emit_stmt(st);
            self.push("\n");
        }
        self.indent -= 1;
        self.indent_str();
        self.push("}");
    }

    fn emit_stmt(&mut self, s: &Stmt) {
        match &s.kind {
            StmtKind::Let { mutable, name, ty, init } => {
                self.push("let ");
                if *mutable {
                    self.push("mut ");
                }
                self.push(name);
                if let Some(t) = ty {
                    self.push(": ");
                    self.emit_type(t);
                }
                if let Some(e) = init {
                    self.push(" = ");
                    self.emit_expr(e, BP_MIN, false);
                }
                self.push(";");
            }
            StmtKind::Assign { target, value } => {
                self.emit_expr(target, BP_MIN, true); // LHS: write path (keep every `.*`)
                self.push(" = ");
                self.emit_expr(value, BP_MIN, false);
                self.push(";");
            }
            StmtKind::Expr(e) => {
                let inner = strip_paren(e);
                if is_block_like(&inner.kind) {
                    self.emit_migrate_marker(inner);
                    self.emit_block_like(inner); // block-like statement: no `;`
                } else {
                    self.emit_expr(e, BP_MIN, false);
                    self.push(";");
                }
            }
        }
    }

    /// Emit a `// MIGRATE:` marker before an author-assisted site that the tool
    /// declines to rewrite. Detected: a two-arm `BoxResult` match (design 0006 §5,
    /// `must_box`/two-arm BoxResult match -> `box(a, v)?`).
    fn emit_migrate_marker(&mut self, e: &Expr) {
        if let ExprKind::Match { arms, .. } = &e.kind {
            if is_boxresult_ladder(arms) {
                self.push("// MIGRATE: two-arm BoxResult match -> `box(a, v)?` (author-assisted, design 0006 §5)\n");
                self.indent_str();
            }
        }
    }

    // ----- expressions (precedence per spec 02 §6.1) ----------------------
    fn emit_expr(&mut self, e: &Expr, parent_bp: u32, wp: bool) {
        let e = strip_paren(e);
        let bp = self.expr_bp(e);
        let paren = bp < parent_bp;
        if paren {
            self.push("(");
        }
        self.emit_expr_bare(e, wp);
        if paren {
            self.push(")");
        }
    }

    fn expr_bp(&self, e: &Expr) -> u32 {
        match &e.kind {
            ExprKind::Binary { op, .. } => bin_bp(*op),
            ExprKind::Unary { .. } | ExprKind::Conv { .. } | ExprKind::Bitcast { .. } => BP_PREFIX,
            ExprKind::Prefix { op, expr } => match op {
                PrefixOp::Deref => BP_POSTFIX,
                PrefixOp::Clone => BP_PREFIX,
                PrefixOp::Read | PrefixOp::Write => {
                    if as_deref_inner(expr).is_some() {
                        BP_POSTFIX // reborrow collapses to a bare place
                    } else {
                        BP_PREFIX
                    }
                }
            },
            ExprKind::Try(_)
            | ExprKind::Call { .. }
            | ExprKind::Field { .. }
            | ExprKind::Index { .. }
            | ExprKind::EnumCtor { .. }
            | ExprKind::StructLit { .. }
            | ExprKind::CastPtr { .. }
            | ExprKind::AddrToPtr { .. }
            | ExprKind::PtrNull { .. }
            | ExprKind::Offsetof { .. }
            | ExprKind::FieldPtr { .. }
            | ExprKind::Sizeof(_)
            | ExprKind::Alignof(_) => BP_POSTFIX,
            _ => BP_ATOM,
        }
    }

    fn emit_expr_bare(&mut self, e: &Expr, wp: bool) {
        match &e.kind {
            ExprKind::IntLit { value, suffix } => {
                self.push(&value.to_string());
                if let Some(s) = suffix {
                    self.push(scalar_kw(*s));
                }
            }
            ExprKind::NegIntLit { value, suffix } => {
                self.push("-");
                self.push(&value.to_string());
                if let Some(s) = suffix {
                    self.push(scalar_kw(*s));
                }
            }
            ExprKind::StrLit(s) => self.emit_string(s),
            ExprKind::BytesLit(s) => {
                self.push("b");
                self.emit_string(s);
            }
            ExprKind::BoolLit(b) => self.push(if *b { "true" } else { "false" }),
            ExprKind::Ident(n) => self.push(n),
            ExprKind::Result => self.push("result"),
            ExprKind::Spawn(c) => {
                self.push("spawn ");
                self.emit_expr(c, BP_MIN, false);
            }
            ExprKind::Break => self.push("break"),
            ExprKind::Continue => self.push("continue"),
            ExprKind::Return(opt) => {
                self.push("return");
                if let Some(inner) = opt {
                    self.push(" ");
                    self.emit_expr(inner, BP_MIN, false);
                }
            }
            ExprKind::Assert(inner) => {
                self.push("assert(");
                self.emit_expr(inner, BP_MIN, false);
                self.push(")");
            }
            ExprKind::Panic(inner) => {
                self.push("panic(");
                self.emit_expr(inner, BP_MIN, false);
                self.push(")");
            }
            ExprKind::Unary { op, expr } => {
                self.push(match op {
                    UnOp::Neg => "-",
                    UnOp::Not => "!",
                    UnOp::BitNot => "~",
                });
                self.emit_expr(expr, BP_PREFIX, false);
            }
            ExprKind::Conv { ty, expr } => {
                self.push("conv ");
                self.emit_type(ty);
                self.push(" ");
                self.emit_expr(expr, BP_PREFIX, false);
            }
            ExprKind::Bitcast { ty, expr } => {
                self.push("bitcast ");
                self.emit_type(ty);
                self.push(" ");
                self.emit_expr(expr, BP_PREFIX, false);
            }
            ExprKind::Prefix { op, expr } => match op {
                PrefixOp::Deref => {
                    // Bare pointee (no field/index step): `.*` kept on both a read
                    // value and a write target (spec 02 §6.3).
                    self.emit_expr(expr, BP_POSTFIX, wp);
                    self.push(".*");
                }
                PrefixOp::Clone => {
                    self.push("clone ");
                    self.emit_expr(expr, BP_PREFIX, false);
                }
                PrefixOp::Read => self.emit_borrow_op("read", expr),
                PrefixOp::Write => self.emit_borrow_op("write", expr),
            },
            ExprKind::Binary { op, lhs, rhs } => {
                let obp = bin_bp(*op);
                self.emit_expr(lhs, obp, false);
                self.push(" ");
                self.push(bin_sym(*op));
                self.push(" ");
                self.emit_expr(rhs, obp + 1, false);
            }
            ExprKind::Try(inner) => {
                self.emit_expr(inner, BP_POSTFIX, false);
                self.push("?");
            }
            ExprKind::Call { callee, args } => {
                self.emit_expr(callee, BP_POSTFIX, false);
                self.push("(");
                self.emit_args(args);
                self.push(")");
            }
            ExprKind::OutArg(place) => {
                self.push("out ");
                self.emit_expr(place, BP_MIN, false);
            }
            ExprKind::Field { base, field, .. } => {
                self.emit_place_base(base, wp);
                self.push(".");
                self.push(field);
            }
            ExprKind::Index { base, index } => {
                self.emit_place_base(base, wp);
                self.push("[");
                self.emit_expr(index, BP_MIN, false); // index value is a read
                self.push("]");
            }
            ExprKind::EnumCtor { enum_name, variant, args } => {
                self.push(enum_name);
                self.push("::");
                self.push(variant);
                if !args.is_empty() {
                    self.push("(");
                    self.emit_args(args);
                    self.push(")");
                }
            }
            ExprKind::StructLit { name, fields } => {
                self.push(name);
                self.push(" { ");
                for (i, f) in fields.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.push(&f.name);
                    self.push(": ");
                    self.emit_expr(&f.value, BP_MIN, false);
                }
                self.push(" }");
            }
            ExprKind::ArrayLit(elems) => {
                self.push("[");
                for (i, el) in elems.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.emit_expr(el, BP_MIN, false);
                }
                self.push("]");
            }
            ExprKind::ArrayRepeat { value, size } => {
                self.push("[");
                self.emit_expr(value, BP_MIN, false);
                self.push("; ");
                self.emit_expr(size, BP_MIN, false);
                self.push("]");
            }
            ExprKind::CastPtr { ty, arg } => self.emit_type_arg_call("cast_ptr", ty, Some(arg)),
            ExprKind::AddrToPtr { ty, arg } => self.emit_type_arg_call("addr_to_ptr", ty, Some(arg)),
            ExprKind::PtrNull { ty } => self.emit_type_arg_call("ptr_null", ty, None),
            ExprKind::Offsetof { ty, field } => {
                self.push("offsetof(");
                self.emit_type(ty);
                self.push(", ");
                self.push(field);
                self.push(")");
            }
            ExprKind::FieldPtr { ptr, field } => {
                self.push("field_ptr(");
                self.emit_expr(ptr, BP_MIN, false);
                self.push(", ");
                self.push(field);
                self.push(")");
            }
            ExprKind::Sizeof(ty) => {
                self.push("sizeof(");
                self.emit_type(ty);
                self.push(")");
            }
            ExprKind::Alignof(ty) => {
                self.push("alignof(");
                self.emit_type(ty);
                self.push(")");
            }
            ExprKind::Paren(inner) => self.emit_expr_bare(strip_paren(inner), wp),
            _ if is_block_like(&e.kind) => self.emit_block_like(e),
            _ => unreachable!("unhandled expression kind in migrator emit"),
        }
    }

    /// The base of a field/element access. On a **read** path a bare-deref base
    /// auto-derefs (drops `.*`); on a **write** path (assignment target) every
    /// deref is kept explicit (spec 02 §6.3/§9.3).
    fn emit_place_base(&mut self, base: &Expr, wp: bool) {
        if let Some(inner) = as_deref_inner(base) {
            self.emit_expr(inner, BP_POSTFIX, wp);
            if wp {
                self.push(".*");
            }
        } else {
            self.emit_expr(base, BP_POSTFIX, wp);
        }
    }

    /// A `read`/`write` borrow *operator*. An explicit reborrow of a bare deref
    /// (`read (deref b)`) collapses to the bare place `b` (design 0005); a fresh
    /// borrow of a sub-place (`read (deref ar).mem[i]`) keeps the keyword and lets
    /// the place auto-deref.
    fn emit_borrow_op(&mut self, kw: &str, operand: &Expr) {
        if let Some(inner) = as_deref_inner(operand) {
            self.emit_expr(inner, BP_MIN, false);
        } else {
            self.push(kw);
            self.push(" ");
            self.emit_expr(operand, BP_PREFIX, false);
        }
    }

    fn emit_type_arg_call(&mut self, name: &str, ty: &Ty, arg: Option<&Expr>) {
        self.push(name);
        self.push("[");
        self.emit_type(ty);
        self.push("](");
        if let Some(a) = arg {
            self.emit_expr(a, BP_MIN, false);
        }
        self.push(")");
    }

    fn emit_args(&mut self, args: &[Expr]) {
        for (i, a) in args.iter().enumerate() {
            if i > 0 {
                self.push(", ");
            }
            self.emit_expr(a, BP_MIN, false);
        }
    }

    // ----- block-like expressions -----------------------------------------
    fn emit_block_like(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::Block(b) => self.emit_block(b),
            ExprKind::If { cond, then_blk, else_blk } => {
                self.push("if ");
                self.emit_head(cond);
                self.push(" ");
                self.emit_block(then_blk);
                if let Some(else_e) = else_blk {
                    self.push(" else ");
                    let inner = strip_paren(else_e);
                    match &inner.kind {
                        ExprKind::Block(b) => self.emit_block(b),
                        _ => self.emit_block_like(inner),
                    }
                }
            }
            ExprKind::Match { scrutinee, arms } => {
                self.push("match ");
                self.emit_head(scrutinee);
                self.push(" {\n");
                self.indent += 1;
                for arm in arms {
                    self.indent_str();
                    self.emit_pattern(&arm.pattern);
                    self.push(" => ");
                    let body = strip_paren(&arm.body);
                    if is_block_like(&body.kind) {
                        self.emit_block_like(body);
                    } else {
                        self.emit_expr(&arm.body, BP_MIN, false);
                    }
                    self.push(",\n");
                }
                self.indent -= 1;
                self.indent_str();
                self.push("}");
            }
            ExprKind::Loop(b) => {
                self.push("loop ");
                self.emit_block(b);
            }
            ExprKind::Scope(b) => {
                self.push("scope ");
                self.emit_block(b);
            }
            ExprKind::While { cond, body } => {
                self.push("while ");
                self.emit_head(cond);
                self.push(" ");
                self.emit_block(body);
            }
            ExprKind::Unsafe { justification, body } => {
                self.push("unsafe ");
                self.emit_string(justification);
                self.push(" ");
                self.emit_block(body);
            }
            ExprKind::Wrapping(b) => {
                self.push("wrapping ");
                self.emit_block(b);
            }
            ExprKind::Saturating(b) => {
                self.push("saturating ");
                self.emit_block(b);
            }
            _ => unreachable!("emit_block_like on a non-block-like expression"),
        }
    }

    /// A head expression (`if`/`while` cond, `match` scrutinee): a bare `Ident {`
    /// is not a struct literal there (spec 02 §8.2), so a struct-literal head is
    /// parenthesized.
    fn emit_head(&mut self, e: &Expr) {
        let inner = strip_paren(e);
        if matches!(inner.kind, ExprKind::StructLit { .. }) {
            self.push("(");
            self.emit_expr(inner, BP_MIN, false);
            self.push(")");
        } else {
            self.emit_expr(e, BP_MIN, false);
        }
    }

    fn emit_pattern(&mut self, p: &Pattern) {
        match &p.kind {
            PatKind::Wildcard => self.push("_"),
            PatKind::Binding(n) => self.push(n),
            PatKind::Variant { enum_name, variant, sub } => {
                self.push(enum_name);
                self.push("::");
                self.push(variant);
                if !sub.is_empty() {
                    self.push("(");
                    for (i, s) in sub.iter().enumerate() {
                        if i > 0 {
                            self.push(", ");
                        }
                        self.emit_pattern(s);
                    }
                    self.push(")");
                }
            }
            PatKind::IntLit { value, negative, suffix } => {
                if *negative {
                    self.push("-");
                }
                self.push(&value.to_string());
                if let Some(sc) = suffix {
                    self.push(scalar_kw(*sc));
                }
            }
            PatKind::IntRange { lo_value, lo_negative, lo_suffix, hi_value, hi_negative, hi_suffix, inclusive } => {
                if *lo_negative {
                    self.push("-");
                }
                self.push(&lo_value.to_string());
                if let Some(sc) = lo_suffix {
                    self.push(scalar_kw(*sc));
                }
                self.push(if *inclusive { "..=" } else { ".." });
                if *hi_negative {
                    self.push("-");
                }
                self.push(&hi_value.to_string());
                if let Some(sc) = hi_suffix {
                    self.push(scalar_kw(*sc));
                }
            }
        }
    }

    fn emit_string(&mut self, s: &str) {
        self.push("\"");
        for c in s.chars() {
            match c {
                '"' => self.push("\\\""),
                '\\' => self.push("\\\\"),
                '\n' => self.push("\\n"),
                '\t' => self.push("\\t"),
                _ => self.buf.push(c),
            }
        }
        self.push("\"");
    }
}

// ----- helpers ------------------------------------------------------------

/// Peel `Paren` layers off an expression.
pub(crate) fn strip_paren(e: &Expr) -> &Expr {
    let mut cur = e;
    while let ExprKind::Paren(inner) = &cur.kind {
        cur = inner;
    }
    cur
}

/// If `e` (paren-stripped) is a bare dereference `deref X`, return the dereffed
/// operand `X`; otherwise `None`. Used for auto-deref and reborrow collapse.
pub(crate) fn as_deref_inner(e: &Expr) -> Option<&Expr> {
    match &strip_paren(e).kind {
        ExprKind::Prefix { op: PrefixOp::Deref, expr } => Some(strip_paren(expr)),
        _ => None,
    }
}

pub(crate) fn is_block_like(kind: &ExprKind) -> bool {
    matches!(
        kind,
        ExprKind::Block(_)
            | ExprKind::If { .. }
            | ExprKind::Match { .. }
            | ExprKind::Scope(_)
            | ExprKind::Loop(_)
            | ExprKind::While { .. }
            | ExprKind::Unsafe { .. }
            | ExprKind::Wrapping(_)
            | ExprKind::Saturating(_)
    )
}

/// A two-arm `BoxResult` match: an arm pattern names a `BoxResult` variant. This
/// is the `box(a, v)?` migration candidate (design 0006 §5, author-assisted).
fn is_boxresult_ladder(arms: &[MatchArm]) -> bool {
    arms.iter().any(|a| {
        matches!(&a.pattern.kind, PatKind::Variant { enum_name, .. } if enum_name == "BoxResult")
    })
}

pub(crate) fn scalar_kw(sc: ScalarTy) -> &'static str {
    match sc {
        ScalarTy::I8 => "i8",
        ScalarTy::I16 => "i16",
        ScalarTy::I32 => "i32",
        ScalarTy::I64 => "i64",
        ScalarTy::Isize => "isize",
        ScalarTy::U8 => "u8",
        ScalarTy::U16 => "u16",
        ScalarTy::U32 => "u32",
        ScalarTy::U64 => "u64",
        ScalarTy::Usize => "usize",
        ScalarTy::Bool => "bool",
        ScalarTy::Unit => "unit",
        ScalarTy::F64 => "f64",
        ScalarTy::F32 => "f32",
    }
}

pub(crate) fn bin_bp(op: BinOp) -> u32 {
    match op {
        BinOp::Or => BP_OR,
        BinOp::And => BP_AND,
        BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => BP_CMP,
        BinOp::BitOr => BP_BITOR,
        BinOp::BitXor => BP_BITXOR,
        BinOp::BitAnd => BP_BITAND,
        BinOp::Shl | BinOp::Shr => BP_SHIFT,
        BinOp::Add | BinOp::Sub => BP_ADD,
        BinOp::Mul | BinOp::Div | BinOp::Rem => BP_MUL,
    }
}

pub(crate) fn bin_sym(op: BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Rem => "%",
        BinOp::Eq => "==",
        BinOp::Ne => "!=",
        BinOp::Lt => "<",
        BinOp::Le => "<=",
        BinOp::Gt => ">",
        BinOp::Ge => ">=",
        BinOp::And => "&&",
        BinOp::Or => "||",
        BinOp::BitAnd => "&",
        BinOp::BitOr => "|",
        BinOp::BitXor => "^",
        BinOp::Shl => "<<",
        BinOp::Shr => ">>",
    }
}
