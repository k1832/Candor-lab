//! The static checker's value-gear half (design 0001 Stage 2): name resolution
//! (via `resolve`), type checking, definite assignment + move checking, the
//! alloc-effect partition, pattern binding modes, and unsafe/rawptr gating.
//!
//! Borrow/loan checking (regions, out-loans, NLL-lite) is Stage 3; this pass is
//! designed to feed it — the CFG (`dataflow`) carries the classified per-point
//! accesses and move points Stage 3 consumes.

pub mod dataflow;
pub mod effects;
pub mod init;
pub mod patterns;

mod expr;
mod stmt;

use std::collections::HashMap;

use crate::ast::*;
use crate::diag::Diag;
use crate::resolve::{resolve_program, Items};
use crate::span::Span;
use crate::types::*;

use dataflow::{Access, Cfg, CfgBlock, FlowState, Place, St, Term};
use effects::AllocEffect;

/// How an expression's result is consumed at its use site.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Use {
    /// Consumed as a value: a place is moved (non-copy) or copied (copy).
    Value,
    BorrowShared,
    BorrowExcl,
    /// Assignment target (LHS place).
    Assign,
    /// `out` argument place.
    Out,
    /// Read but not consumed (e.g. `clone` operand, `addr_of` place).
    ReadOnly,
}

#[derive(Clone, Debug)]
struct LocalInfo {
    ty: Type,
    movable: bool,
}

/// Per-function mutable state (CFG builder + environment).
struct FnState {
    env: Vec<HashMap<String, LocalInfo>>,
    ret_ty: Type,
    out_params: Vec<String>,
    in_unsafe: bool,
    in_ensures: bool,
    blocks: Vec<CfgBlock>,
    cur: Option<usize>,
    /// (continue target, break target) per enclosing loop.
    loops: Vec<(usize, usize)>,
    alloc: AllocEffect,
}

impl FnState {
    fn empty() -> FnState {
        FnState {
            env: vec![HashMap::new()],
            ret_ty: Type::unit(),
            out_params: Vec::new(),
            in_unsafe: false,
            in_ensures: false,
            blocks: Vec::new(),
            cur: None,
            loops: Vec::new(),
            alloc: AllocEffect::default(),
        }
    }
}

pub struct Checker<'a> {
    items: &'a Items,
    pub diags: Vec<Diag>,
    f: FnState,
}

/// Entry point: parse -> resolve -> check. Returns all diagnostics.
pub fn check_program(prog: &Program) -> Vec<Diag> {
    let mut diags = Vec::new();
    let items = resolve_program(prog, &mut diags);
    let mut c = Checker {
        items: &items,
        diags,
        f: FnState::empty(),
    };
    for item in &prog.items {
        match item {
            Item::Fn(f) => c.check_fn(f),
            Item::Static(s) => c.check_static(s),
            _ => {}
        }
    }
    c.diags
}

impl<'a> Checker<'a> {
    // ----- environment ----------------------------------------------------

    fn push_scope(&mut self) {
        self.f.env.push(HashMap::new());
    }
    fn pop_scope(&mut self) {
        self.f.env.pop();
    }
    fn add_local(&mut self, name: &str, ty: Type, movable: bool) {
        self.f
            .env
            .last_mut()
            .unwrap()
            .insert(name.to_string(), LocalInfo { ty, movable });
    }
    fn lookup_local(&self, name: &str) -> Option<&LocalInfo> {
        for scope in self.f.env.iter().rev() {
            if let Some(li) = scope.get(name) {
                return Some(li);
            }
        }
        None
    }
    fn local_movable(&self, name: &str) -> bool {
        self.lookup_local(name).map(|l| l.movable).unwrap_or(true)
    }

    // ----- CFG building ---------------------------------------------------

    fn new_block(&mut self) -> usize {
        let id = self.f.blocks.len();
        self.f.blocks.push(CfgBlock {
            actions: Vec::new(),
            term: Term::Diverge,
            join_span: Span::point(0),
        });
        id
    }
    fn set_term(&mut self, b: usize, term: Term) {
        self.f.blocks[b].term = term;
    }
    fn emit(&mut self, place: &Option<Place>, access: Access, span: Span) {
        if let (Some(cur), Some(p)) = (self.f.cur, place) {
            self.f.blocks[cur].actions.push(dataflow::Action {
                place: p.clone(),
                access,
                span,
                tracked: true,
            });
        }
    }

    /// Emit the access an expression's use site imposes on a place.
    fn emit_place_action(&mut self, place: &Option<Place>, u: Use, ty: &Type, span: Span) {
        let access = match u {
            Use::Value => {
                if is_copy(ty, self.items) {
                    Access::Read
                } else {
                    let (movable, dh) = match place {
                        Some(p) => (self.local_movable(&p.root), self.is_drop_hooked_partial(p)),
                        None => (true, false),
                    };
                    Access::Move {
                        movable,
                        drop_hooked_partial: dh,
                    }
                }
            }
            Use::BorrowShared | Use::BorrowExcl => Access::Borrow,
            Use::Assign => Access::Assign,
            Use::Out => Access::OutArg,
            Use::ReadOnly => Access::Read,
        };
        self.emit(place, access, span);
    }

    /// Is `place` a partial move out of a `drop`-hooked struct (design §1.6)?
    fn is_drop_hooked_partial(&self, place: &Place) -> bool {
        if !place.is_direct() || place.proj.is_empty() {
            return false;
        }
        match self.lookup_local(&place.root).map(|l| l.ty.clone()) {
            Some(Type::Named(n)) => self
                .items
                .lookup_struct(&n)
                .map(|s| s.has_drop)
                .unwrap_or(false),
            _ => false,
        }
    }

    fn note_alloc(&mut self, span: Span, reason: impl Into<String>) {
        self.f.alloc.note(span, reason);
    }

    // ----- driver ---------------------------------------------------------

    fn check_fn(&mut self, f: &FnDecl) {
        let sig = self.items.fns[&f.name].clone();
        self.f = FnState::empty();
        self.f.ret_ty = sig.ret.clone();

        // Entry block + parameter environment + initial flow state.
        let entry = self.new_block();
        self.f.cur = Some(entry);
        let mut entry_state = FlowState::new();
        for p in &sig.params {
            let is_out = p.mode == ParamMode::Out;
            self.add_local(&p.name, p.lowered.clone(), true);
            if is_out {
                self.f.out_params.push(p.name.clone());
                entry_state.set(&Place::local(p.name.clone()), St::Uninit);
            } else {
                entry_state.set(&Place::local(p.name.clone()), St::Init);
            }
        }

        // requires clauses are ordinary boolean expressions (design §7.3).
        for r in &f.requires {
            let t = self.check_expr(r, Use::Value);
            self.expect_bool(&t, r.span, "a `requires` clause");
        }

        self.check_block_stmts(&f.body);

        // Fall-through: a body that does not diverge is a normal return.
        if let Some(cur) = self.f.cur {
            self.set_term(cur, Term::Return);
        }

        // ensures clauses may reference `result` (design §7.3).
        self.f.in_ensures = true;
        let saved = self.f.cur;
        self.f.cur = None; // contract exprs are not part of the CFG
        for e in &f.ensures {
            let t = self.check_expr(e, Use::Value);
            self.expect_bool(&t, e.span, "an `ensures` clause");
        }
        self.f.cur = saved;
        self.f.in_ensures = false;

        // Run the value-gear dataflow over the finished CFG.
        let mut cfg = Cfg {
            blocks: std::mem::take(&mut self.f.blocks),
            entry,
            preds: Vec::new(),
        };
        cfg.compute_preds();
        let out_params = self.f.out_params.clone();
        init::analyze(&cfg, &entry_state, &out_params, &mut self.diags);

        if let Some(d) = self.f.alloc.finish(f.alloc, &f.name, f.span) {
            self.diags.push(d);
        }
    }

    fn check_static(&mut self, s: &StaticDecl) {
        let expected = self.items.statics[&s.name].0.clone();
        self.f = FnState::empty();
        self.f.cur = None; // constant context: no CFG
        self.check_against(&s.value, &expected);
    }

    // ----- shared helpers -------------------------------------------------

    /// Check `expr` as a value against an expected type, with fn-ptr effect and
    /// integer-literal flexibility. Emits E0402 / E0703 on mismatch.
    fn check_against(&mut self, expr: &Expr, expected: &Type) -> Type {
        let t = self.check_expr(expr, Use::Value);
        if matches!(t, Type::Error) || matches!(expected, Type::Error) || matches!(t, Type::Never) {
            return t;
        }
        if !assignable(&t, expected) {
            if let (Type::FnPtr(from), Type::FnPtr(to)) = (&t, expected) {
                if from.alloc && !to.alloc {
                    self.diags.push(
                        Diag::error(
                            "E0402",
                            "an `alloc` function may not be assigned to a non-`alloc` fn-pointer".to_string(),
                            expr.span,
                        )
                        .with_note("effects are upper bounds: the pointer type understates the callee's effect (§6.1)", None),
                    );
                    return t;
                }
            }
            self.diags.push(
                Diag::error(
                    "E0703",
                    format!(
                        "type mismatch: expected `{}`, found `{}`",
                        expected.display(),
                        t.display()
                    ),
                    expr.span,
                ),
            );
        }
        t
    }

    fn expect_bool(&mut self, t: &Type, span: Span, what: &str) {
        if !matches!(t, Type::Error | Type::Never) && *t != Type::bool() {
            self.diags.push(Diag::error(
                "E0703",
                format!("{what} must be `bool`, found `{}`", t.display()),
                span,
            ));
        }
    }

    /// Resolve an AST type appearing in expression position (`conv`, intrinsics).
    fn resolve_ty(&mut self, ty: &Ty) -> Type {
        match &ty.kind {
            TyKind::Scalar(s) => Type::Scalar(*s),
            TyKind::Named(n) => {
                if self.items.structs.contains_key(n) || self.items.enums.contains_key(n) {
                    Type::Named(n.clone())
                } else {
                    self.diags
                        .push(Diag::error("E0102", format!("unknown type `{n}`"), ty.span));
                    Type::Error
                }
            }
            TyKind::Array { size, elem } => {
                let len = match &size.kind {
                    ExprKind::IntLit { value, .. } => ArrayLen::Lit(*value),
                    ExprKind::Ident(n) => ArrayLen::Named(n.clone()),
                    _ => ArrayLen::Unknown,
                };
                Type::Array(Box::new(self.resolve_ty(elem)), len)
            }
            TyKind::Slice(e) => Type::Slice(Box::new(self.resolve_ty(e))),
            TyKind::SliceMut(e) => Type::SliceMut(Box::new(self.resolve_ty(e))),
            TyKind::RawPtr(e) => Type::RawPtr(Box::new(self.resolve_ty(e))),
            TyKind::Box(e) => Type::Box(Box::new(self.resolve_ty(e))),
            TyKind::BoxResult(e) => Type::BoxResult(Box::new(self.resolve_ty(e))),
            TyKind::Borrow(e) => Type::Borrow(Box::new(self.resolve_ty(e))),
            TyKind::BorrowMut(e) => Type::BorrowMut(Box::new(self.resolve_ty(e))),
            TyKind::FnPtr(fp) => {
                let params = fp
                    .params
                    .iter()
                    .map(|p| (p.mode, self.resolve_ty(&p.ty)))
                    .collect();
                Type::FnPtr(crate::types::FnPtrTy {
                    params,
                    alloc: fp.alloc,
                    ret: Box::new(self.resolve_ty(&fp.ret)),
                })
            }
        }
    }

    fn require_unsafe(&mut self, span: Span, op: &str) {
        if !self.f.in_unsafe {
            self.diags.push(
                Diag::error(
                    "E0501",
                    format!("raw-pointer operation `{op}` requires an `unsafe` block"),
                    span,
                )
                .with_note("only holding/moving/copying/comparing a rawptr is safe (§4.2)", None),
            );
        }
    }
}
