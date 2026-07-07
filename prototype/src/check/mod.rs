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
pub mod loans;
pub mod patterns;

mod expr;
mod stmt;

use std::collections::HashMap;

use crate::ast::*;
use crate::diag::Diag;
use crate::resolve::{resolve_program, Items};
use crate::span::Span;
use crate::types::*;

use dataflow::{Access, Cfg, CfgBlock, FlowState, LoanKind, Place, St, Term};
use effects::AllocEffect;
use loans::{Anchor, LoanInfo, LoanTables};

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
    /// (continue target, break target, env scope depth) per enclosing loop.
    /// The depth is the number of open scopes *outside* the loop body, so
    /// `break`/`continue` can emit scope-exit drop checks (§1.6 dual) for the
    /// body scopes they unwind.
    loops: Vec<(usize, usize, usize)>,
    alloc: AllocEffect,
    // ---- Stage 3 loan machinery ----
    /// Every loan created in the body (design 0001 §2.3).
    loans: Vec<LoanInfo>,
    /// Loan ids that are co-arguments of one call (§3.1, no two-phase).
    call_groups: Vec<Vec<usize>>,
    /// Loans carried by the just-evaluated expression's value (borrow exprs,
    /// call-return extension). Read by the binding site to anchor them.
    carried: Vec<usize>,
    /// Provenance root of the just-evaluated borrow value (for return checks).
    carried_prov: Option<String>,
    /// Signature facts the return-provenance check needs (name, is_borrow_param,
    /// region tag) and the return's region/borrow-ness.
    sig_params: Vec<(String, bool, Option<String>)>,
    ret_region: Option<String>,
    ret_is_borrow: bool,
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
            loans: Vec::new(),
            call_groups: Vec::new(),
            carried: Vec::new(),
            carried_prov: None,
            sig_params: Vec::new(),
            ret_region: None,
            ret_is_borrow: false,
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
            Use::BorrowShared => {
                self.record_borrow(place, LoanKind::Shared, span);
                Access::Borrow(LoanKind::Shared)
            }
            Use::BorrowExcl => {
                self.record_borrow(place, LoanKind::Excl, span);
                Access::Borrow(LoanKind::Excl)
            }
            Use::Assign => Access::Assign,
            Use::Out => Access::OutArg,
            Use::ReadOnly => Access::Read,
        };
        self.emit(place, access, span);
    }

    /// Emit scope-exit drop-point checks (the dual of §1.6's move-join rule,
    /// finding 2026-07-07) for every *needs-drop* local declared in
    /// `self.f.env[from_depth..]`, at `span`. These mirror the interpreter's
    /// `drop_scope` points: a lexical block end and the body scopes a
    /// `return`/`break`/`continue` unwinds. A no-op once control has left the
    /// block (`cur` is `None`) — that path emitted its own exits already.
    fn emit_scope_exits(&mut self, from_depth: usize, span: Span) {
        if self.f.cur.is_none() {
            return;
        }
        let mut targets: Vec<String> = Vec::new();
        for scope in self.f.env.iter().skip(from_depth) {
            for (name, info) in scope.iter() {
                if needs_drop(&info.ty, self.items) {
                    targets.push(name.clone());
                }
            }
        }
        for name in targets {
            self.emit(&Some(Place::local(name)), Access::ScopeExit, span);
        }
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

    /// Create a loan on `place`'s conflict-granularity anchor (design §2.2/§2.3)
    /// and mark it as carried by the expression's value. Anchoring to a binding
    /// (or extension across a call) happens later, at the value's landing site.
    pub(super) fn record_borrow(&mut self, place: &Option<Place>, kind: LoanKind, span: Span) {
        match place {
            Some(p) => {
                let canon = p.canonical();
                let root = canon.root.clone();
                let id = self.f.loans.len();
                self.f.loans.push(LoanInfo {
                    place: canon,
                    kind,
                    span,
                    anchor: Anchor::Temp,
                });
                self.f.carried = vec![id];
                self.f.carried_prov = Some(root);
            }
            None => {
                self.f.carried = Vec::new();
                self.f.carried_prov = None;
            }
        }
    }

    /// Anchor the currently-carried loans to a landing binding `name` (a `let` or
    /// assignment target): they are now in scope over `name`'s live range (§2.3).
    pub(super) fn anchor_carried(&mut self, name: &str) {
        let ids = std::mem::take(&mut self.f.carried);
        for id in ids {
            self.f.loans[id].anchor = Anchor::Binding(name.to_string());
        }
        self.f.carried_prov = None;
    }

    pub(super) fn take_carried(&mut self) -> Vec<usize> {
        std::mem::take(&mut self.f.carried)
    }
    pub(super) fn set_carried(&mut self, ids: Vec<usize>, prov: Option<String>) {
        self.f.carried = ids;
        self.f.carried_prov = prov;
    }
    pub(super) fn clear_carried(&mut self) {
        self.f.carried = Vec::new();
        self.f.carried_prov = None;
    }
    /// Create a loan directly anchored to a binding (a borrowed-scrutinee
    /// pattern binding, §8.2.1): in scope over that binding's live range.
    pub(super) fn record_binding_loan(&mut self, place: &Place, kind: LoanKind, span: Span, name: &str) {
        self.f.loans.push(LoanInfo {
            place: place.canonical(),
            kind,
            span,
            anchor: Anchor::Binding(name.to_string()),
        });
    }

    pub(super) fn new_temp_loan(&mut self, place: Place, kind: LoanKind, span: Span) -> usize {
        let id = self.f.loans.len();
        self.f.loans.push(LoanInfo {
            place,
            kind,
            span,
            anchor: Anchor::Temp,
        });
        id
    }
    pub(super) fn push_call_group(&mut self, group: Vec<usize>) {
        if group.len() >= 2 {
            self.f.call_groups.push(group);
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
        self.f.ret_is_borrow = matches!(sig.ret, Type::Borrow(_) | Type::BorrowMut(_));
        self.f.ret_region = sig.ret_region.clone();
        self.f.sig_params = sig
            .params
            .iter()
            .map(|p| (p.name.clone(), is_borrow_param(p), p.region.clone()))
            .collect();
        self.check_signature_regions(&sig);

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

        // Fall-through: a body that runs off its end is an implicit unit return
        // (an all-paths-return error for a non-unit function; Stage 3 flags it).
        if let Some(cur) = self.f.cur {
            self.set_term(cur, Term::FallThrough);
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

        // Stage 3: NLL-lite loan liveness + XOR/move/write conflict scan,
        // same-call overlap, and all-paths-return (design §2.2/§2.3/§3.1/§7.4).
        let tables = LoanTables {
            loans: std::mem::take(&mut self.f.loans),
            call_groups: std::mem::take(&mut self.f.call_groups),
        };
        let ret_non_unit = !matches!(
            sig.ret,
            Type::Scalar(crate::token::ScalarTy::Unit) | Type::Never | Type::Error
        );
        loans::analyze(&cfg, &tables, ret_non_unit, f.span, &mut self.diags);

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

    /// Region well-formedness for a borrow-returning signature (design §3.3):
    /// two-plus borrow params returning a borrow require a region variable;
    /// an explicit return region must be declared and tag some borrow param.
    fn check_signature_regions(&mut self, sig: &crate::resolve::FnSig) {
        if !matches!(sig.ret, Type::Borrow(_) | Type::BorrowMut(_)) {
            return;
        }
        let borrow_params: Vec<&crate::resolve::ParamInfo> =
            sig.params.iter().filter(|p| is_borrow_param(p)).collect();
        match &sig.ret_region {
            Some(r) => {
                if !sig.regions.contains(r) {
                    self.diags.push(
                        Diag::error(
                            "E0807",
                            format!("return region `{r}` is not declared in the signature"),
                            sig.ret_span,
                        )
                        .with_note("declare it in brackets after the function name, e.g. `fn f[r](...)` (§3.3)", None),
                    );
                } else if !borrow_params.iter().any(|p| p.region.as_deref() == Some(r.as_str())) {
                    self.diags.push(
                        Diag::error(
                            "E0808",
                            format!("return region `{r}` tags no borrow parameter"),
                            sig.ret_span,
                        )
                        .with_note("attach the region to the borrow parameter the return derives from (§3.3)", None),
                    );
                }
            }
            None => {
                if borrow_params.len() >= 2 {
                    self.diags.push(
                        Diag::error(
                            "E0807",
                            "a borrow return from two or more borrow parameters requires a region variable".to_string(),
                            sig.ret_span,
                        )
                        .with_note("region variables are mandatory here; there is no compact default (§3.3)", None),
                    );
                }
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

/// A parameter that is itself a borrow (design §3.3): a `read`/`write` mode
/// parameter, or a by-value borrow-kind (slice/borrow) parameter.
fn is_borrow_param(p: &crate::resolve::ParamInfo) -> bool {
    matches!(p.mode, ParamMode::Read | ParamMode::Write) || p.decl_ty.is_borrow_kind()
}
