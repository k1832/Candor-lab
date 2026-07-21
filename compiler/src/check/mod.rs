//! The static checker's value-gear half (design 0001 Stage 2): name resolution
//! (via `resolve`), type checking, definite assignment + move checking, the
//! alloc-effect partition, pattern binding modes, and unsafe/rawptr gating.
//!
//! Borrow/loan checking (regions, out-loans, NLL-lite) is Stage 3; this pass is
//! designed to feed it — the CFG (`dataflow`) carries the classified per-point
//! accesses and move points Stage 3 consumes.

pub mod dataflow;
pub mod effects;
pub mod foreign;
pub mod init;
pub mod loans;
pub mod patterns;

mod concurrency;
mod expr;
mod generics;
pub use generics::check_generic_program;

/// Re-export for the generics submodule: resolve a (borrowed) type to its enum.
pub fn patterns_resolve_enum(ty: &crate::types::Type, items: &dyn crate::types::ItemEnv) -> Option<crate::types::EnumTy> {
    patterns::resolve_enum(ty, items).map(|(_, e, _)| e)
}
mod stmt;

use std::collections::HashMap;

use crate::ast::*;
use crate::diag::Diag;
use crate::resolve::{lower_param, resolve_program, FnSig, Items, ParamInfo};
use crate::span::Span;
use crate::types::*;

use dataflow::{Access, Cfg, CfgBlock, FlowState, LoanKind, Place, St, Term};
use effects::AllocEffect;
use foreign::{ForeignEffect, ForeignFnInfo};
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
    /// Inside a `requires`/`ensures`/`assert` condition: contract clauses are
    /// read-only (review #3, 2026-07-07). Gates the E0708 read-only rule.
    in_contract: bool,
    /// Set while re-emitting `ensures` accesses at return points: marks the
    /// emitted actions as contract-origin (for the E0301 note).
    emit_contract: bool,
    blocks: Vec<CfgBlock>,
    cur: Option<usize>,
    /// (continue target, break target, env scope depth) per enclosing loop.
    /// The depth is the number of open scopes *outside* the loop body, so
    /// `break`/`continue` can emit scope-exit drop checks (§1.6 dual) for the
    /// body scopes they unwind.
    loops: Vec<(usize, usize, usize)>,
    alloc: AllocEffect,
    /// The `foreign` effect accumulator (design 0011 §2).
    foreign: ForeignEffect,
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
    /// Inside a `wrapping`/`saturating` regime block: a constant `conv` with
    /// value loss is folded (allowed) rather than a compile error (design 0006
    /// §2.4). Depth counter so nested regimes restore correctly.
    regime_depth: usize,
    /// Structured-concurrency (design 0012). Lexical `scope` nesting depth (0 =
    /// outside any scope): a `spawn` is a checker error unless > 0 (§5).
    scope_depth: usize,
    /// Per-open-scope stack of synthetic scope-length-loan binding names (§1.2):
    /// each spawn borrow re-anchors its loan to a fresh synthetic name registered
    /// here; at the scope's closing brace a synthetic use of each keeps the loan
    /// live to the brace (the whole-scope loan range).
    scope_synth: Vec<Vec<String>>,
    /// Monotonic counter minting the synthetic scope-length-loan binding names.
    synth_ctr: usize,
}

impl FnState {
    fn empty() -> FnState {
        FnState {
            env: vec![HashMap::new()],
            ret_ty: Type::unit(),
            out_params: Vec::new(),
            in_unsafe: false,
            in_ensures: false,
            in_contract: false,
            emit_contract: false,
            blocks: Vec::new(),
            cur: None,
            loops: Vec::new(),
            alloc: AllocEffect::default(),
            foreign: ForeignEffect::default(),
            loans: Vec::new(),
            call_groups: Vec::new(),
            carried: Vec::new(),
            carried_prov: None,
            sig_params: Vec::new(),
            ret_region: None,
            ret_is_borrow: false,
            regime_depth: 0,
            scope_depth: 0,
            scope_synth: Vec::new(),
            synth_ctr: 0,
        }
    }
}

pub struct Checker<'a> {
    items: &'a Items,
    pub diags: Vec<Diag>,
    f: FnState,
    /// True when checking an AST produced by the real (`.cnr`) front-end. Gates
    /// the surface rules that are real-syntax-only: literal over-range rejection
    /// (spec 01 §3.3), constant-`conv` loss rejection (design 0006 §2.4), and the
    /// write-through-borrow-needs-explicit-`.*` rule (spec 02 §6.3). The shared
    /// checker is otherwise identical for both front-ends.
    real: bool,
    /// Concrete generic instantiations reached during checking (design 0007 §5.1):
    /// each `(generic name, concrete type arguments)`. Drives monomorphization.
    pub insts: Vec<(String, Vec<Type>)>,
    /// True while checking a generic body at its definition site with opaque type
    /// parameters (suppresses instantiation collection and gates §3 rules).
    def_site: bool,
    /// Monomorphization shapes recorded per expression-node span start: the
    /// generic construct and its (possibly parametric) type arguments (design
    /// 0007 §5). Consumed by `generics::monomorphize`.
    pub shapes: std::collections::HashMap<crate::generics::ShapeKey, crate::generics::Shape>,
    /// Index of the top-level item currently being checked, in the merged program's
    /// item list — the first half of a [`crate::generics::ShapeKey`], making a
    /// recorded shape's key globally unique across modules despite per-file spans.
    cur_item: usize,
    /// Type-parameter names in scope while checking a generic body at its
    /// definition site (design 0007); a bare type name resolves to `Type::Param`.
    type_params: Vec<String>,
    /// Each in-scope type parameter's bound interface names (§2.1 method lookup).
    param_bounds: Vec<(String, Vec<String>)>,
    /// The expected type at the current value position, if any — a hint used to
    /// resolve otherwise-uninferable generic type arguments (e.g. `Opt::None`).
    expected_ty: Option<Type>,
    /// The generic function currently being definition-site checked (for the
    /// polymorphic-recursion self-instantiation check, design 0007 §5.1.1).
    cur_generic: Option<String>,
    /// Per-function foreign-effect resolution (design 0011 §2), for the audit.
    pub foreign_report: Vec<ForeignFnInfo>,
}

/// Entry point: parse -> resolve -> check. Returns all diagnostics. Used by the
/// throwaway (`.cn`) front-end.
pub fn check_program(prog: &Program) -> Vec<Diag> {
    check_program_opts(prog, false)
}

/// As [`check_program`], but enabling the real-syntax (`.cnr`) surface rules
/// (design 0006 §2.4; spec 01 §3.3, spec 02 §6.3). The downstream analysis is
/// identical; only the extra surface diagnostics differ.
pub fn check_program_real(prog: &Program) -> Vec<Diag> {
    if crate::generics::is_generic_program(prog) {
        return generics::check_generic_program(prog, true).0;
    }
    check_program_opts(prog, true)
}

fn check_program_opts(prog: &Program, real: bool) -> Vec<Diag> {
    check_program_collect(prog, real).0
}

/// Re-check module `own` against its imports' signature-only interface **stubs**
/// (design 0008 §2, §2.4) — the resolution of Stage C's residual (i). `own` are
/// the module's own (already-qualified) items; `stubs` are the qualified stub
/// items of its transitive imports ([`crate::build::stub::stub_item`]). Only
/// `own`'s bodies are analyzed; the stubs contribute signatures, tables, impls,
/// and the checked generic/drop bodies instantiation needs — never a re-analysis
/// of an import's opaque `fn` body, and never a re-parse of its source. Returns
/// the diagnostics **plus the reached generic instantiations** (name, type args)
/// the codegen tier keys its per-instantiation cache on.
pub fn check_module_stub(own: &[Item], stubs: &[Item]) -> (Vec<Diag>, Vec<(String, Vec<Type>)>) {
    let own_len = own.len();
    let mut items: Vec<Item> = Vec::with_capacity(own.len() + stubs.len());
    items.extend(own.iter().cloned());
    items.extend(stubs.iter().cloned());
    let prog = Program { items };
    if crate::generics::is_generic_program(&prog) {
        let (diags, insts, _shapes, _foreign) = generics::check_generic_program_own(&prog, true, own_len);
        (diags, insts)
    } else {
        (check_program_collect_own(&prog, true, own_len).0, Vec::new())
    }
}

/// As [`check_program_real`] but also returns the per-function foreign-effect
/// report (design 0011 §2), for the `audit` command's effect-reach section.
pub fn check_program_real_foreign(prog: &Program) -> (Vec<Diag>, Vec<ForeignFnInfo>) {
    if crate::generics::is_generic_program(prog) {
        return generics::check_generic_program_foreign(prog, true);
    }
    check_program_collect(prog, true)
}

fn check_program_collect(prog: &Program, real: bool) -> (Vec<Diag>, Vec<ForeignFnInfo>) {
    check_program_collect_own(prog, real, prog.items.len())
}

/// As [`check_program_collect`], but only the first `own_len` items are *checked*
/// (their bodies analyzed and diagnostics emitted); the remainder are
/// signature-only interface **stubs** of imported modules (design 0008 §2): their
/// tables are resolved and their `drop`-hook alloc-on-drop is folded in, but
/// their bodies are never re-analyzed — the owning module already did, once. This
/// is the concrete (non-generic) half of the signature-only re-check tier.
fn check_program_collect_own(prog: &Program, real: bool, own_len: usize) -> (Vec<Diag>, Vec<ForeignFnInfo>) {
    let mut diags = Vec::new();
    let mut items = resolve_program(prog, &mut diags);

    // Every struct's `drop` hook is checked as a synthetic
    // `fn drop(self: write StructT) -> unit` (retest 2026-07-08, finding 4). The
    // alloc-on-drop FIXPOINT runs over ALL hooks (own + stub) — a stub type's
    // drop effect is a signature fact the owner resolves; only the diagnostic
    // pass below is restricted to own hooks.
    let hooks: Vec<(usize, String, Block, Span)> = prog
        .items
        .iter()
        .enumerate()
        .filter_map(|(i, it)| match it {
            Item::Struct(s) => s
                .drop_hook
                .as_ref()
                .map(|b| (i, s.name.clone(), b.clone(), s.span)),
            _ => None,
        })
        .collect();

    // Alloc-on-drop fixpoint: a hook body that is alloc-effecting makes its TYPE
    // alloc-on-drop (§1.5/§6.3). Because one hook may drop a value of another
    // alloc-on-drop type, run to a monotonic fixpoint over a read-only snapshot
    // (throwaway diagnostics) before the real, diagnostic-emitting pass.
    if !hooks.is_empty() {
        let mut guard = 0;
        loop {
            let snapshot = items.clone();
            let mut changed = false;
            for (_, sname, block, span) in &hooks {
                let mut c = Checker {
                    items: &snapshot,
                    diags: Vec::new(),
                    f: FnState::empty(),
                    real,
                    insts: Vec::new(),
                    def_site: false,
                    shapes: std::collections::HashMap::new(),
                    cur_item: 0,
                    type_params: Vec::new(),
                    param_bounds: Vec::new(),
                    expected_ty: None,
                    cur_generic: None, foreign_report: Vec::new(),
                };
                let (fdecl, sig) = synth_drop_hook(sname, block, *span);
                c.check_fn_with_sig(&fdecl, &sig);
                if c.f.alloc.site.is_some()
                    && !items.structs.get(sname).map(|s| s.alloc_on_drop).unwrap_or(false)
                {
                    if let Some(s) = items.structs.get_mut(sname) {
                        s.alloc_on_drop = true;
                        changed = true;
                    }
                }
            }
            guard += 1;
            if !changed || guard > hooks.len() + 1 {
                break;
            }
        }
    }

    // Foreign-boundary item checks (design 0011): placement, mappability, trust.
    foreign::check_foreign_items(&items, prog, &mut diags);

    let mut c = Checker {
        items: &items,
        diags,
        f: FnState::empty(),
        real,
        insts: Vec::new(),
        def_site: false,
        shapes: std::collections::HashMap::new(),
        cur_item: 0,
        type_params: Vec::new(),
        param_bounds: Vec::new(),
        expected_ty: None,
        cur_generic: None, foreign_report: Vec::new(),
    };
    for item in &prog.items[..own_len] {
        match item {
            Item::Fn(f) => c.check_fn(f),
            Item::Static(s) => c.check_static(s),
            _ => {}
        }
    }
    // Real (diagnostic-emitting) pass over each OWN hook body, with stable flags.
    for (idx, sname, block, span) in &hooks {
        if *idx >= own_len {
            continue;
        }
        let (fdecl, sig) = synth_drop_hook(sname, block, *span);
        c.check_fn_with_sig(&fdecl, &sig);
    }
    (c.diags, c.foreign_report)
}

/// Build the synthetic `fn drop(self: write StructT) -> unit` whose body is the
/// hook — ordinary checked code (retest 2026-07-08, finding 4). Marked `alloc`
/// so an alloc-effecting hook is *permitted* (it makes the type alloc-on-drop
/// instead of erroring); the caller reads `f.alloc.site` to learn that.
fn synth_drop_hook(struct_name: &str, block: &Block, span: Span) -> (FnDecl, FnSig) {
    let self_ty = Type::Named(struct_name.to_string());
    let sig = FnSig {
        name: format!("drop({struct_name})"),
        regions: Vec::new(),
        params: vec![ParamInfo {
            name: "self".to_string(),
            mode: ParamMode::Write,
            region: None,
            decl_ty: self_ty.clone(),
            lowered: lower_param(ParamMode::Write, self_ty.clone()),
            span,
        }],
        alloc: true,
        foreign: false,
        ret: Type::unit(),
        ret_region: None,
        ret_span: span,
        span,
    };
    let fdecl = FnDecl {
        name: sig.name.clone(),
        type_params: Vec::new(),
        regions: Vec::new(),
        params: vec![Param {
            name: "self".to_string(),
            mode: ParamMode::Write,
            region: None,
            ty: Ty {
                kind: TyKind::Named(struct_name.to_string()),
                span,
            },
            span,
        }],
        alloc: true,
        foreign: false,
        boundary: false,
        requires: Vec::new(),
        ensures: Vec::new(),
        ret: None,
        body: block.clone(),
        span,
    };
    (fdecl, sig)
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
                contract: self.f.emit_contract,
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
                    // A contract clause is read-only: moving a non-copy value out
                    // of it is rejected (review #3, 2026-07-07). Copy reads above
                    // are fine; only genuine moves reach here.
                    self.reject_in_contract(span, "moving a non-`copy` value");
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
                self.reject_in_contract(span, "a `write`-borrow");
                self.record_borrow(place, LoanKind::Excl, span);
                Access::Borrow(LoanKind::Excl)
            }
            Use::Assign => Access::Assign {
                needs_drop: needs_drop(ty, self.items),
                box_paths: box_subpaths(ty, self.items),
            },
            Use::Out => {
                self.reject_in_contract(span, "an `out` argument");
                Access::OutArg {
                    needs_drop: needs_drop(ty, self.items),
                    box_paths: box_subpaths(ty, self.items),
                }
            }
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
        let mut targets: Vec<(String, Vec<Vec<String>>)> = Vec::new();
        for scope in self.f.env.iter().skip(from_depth) {
            for (name, info) in scope.iter() {
                if needs_drop(&info.ty, self.items) {
                    targets.push((name.clone(), box_subpaths(&info.ty, self.items)));
                }
            }
        }
        for (name, box_paths) in targets {
            self.emit(&Some(Place::local(name)), Access::ScopeExit { box_paths }, span);
        }
    }

    /// Is `place` a partial move whose path crosses a `drop`-hooked struct
    /// (design §1.6 rule 2)? The check walks every *proper* prefix of the
    /// projection — the root place and each intermediate place up to, but not
    /// including, the moved sub-place. If any prefix's type declares a `drop`
    /// hook, moving a nested field out would leave that value partially moved and
    /// silently skip its hook (finding 3 of 2026-07-07, nested partial move).
    fn is_drop_hooked_partial(&self, place: &Place) -> bool {
        if !place.is_direct() || place.proj.is_empty() {
            return false;
        }
        let mut ty = match self.lookup_local(&place.root).map(|l| l.ty.clone()) {
            Some(t) => t,
            None => return false,
        };
        for proj in &place.proj {
            match &ty {
                Type::Named(n) => {
                    if self.items.lookup_struct(n).map(|s| s.has_drop).unwrap_or(false) {
                        return true;
                    }
                }
                // A generic aggregate's `drop` hook is recorded on its generic decl
                // (design 0007 §3.4): a partial move across it is equally forbidden.
                Type::App(n, _)
                    if self.items.lookup_generic(n).map(|g| g.has_drop).unwrap_or(false) =>
                {
                    return true;
                }
                _ => {}
            }
            let next = match (&ty, proj) {
                (Type::Named(n), dataflow::Proj::Field(f)) => self
                    .items
                    .lookup_struct(n)
                    .and_then(|st| st.fields.iter().find(|(fn_, _)| fn_ == f).map(|(_, t)| t.clone())),
                (Type::App(n, args), dataflow::Proj::Field(f)) => {
                    crate::types::app_fields(self.items, n, args)
                        .and_then(|fs| fs.into_iter().find(|(fn_, _)| fn_ == f).map(|(_, t)| t))
                }
                _ => None,
            };
            ty = match next {
                Some(t) => t,
                None => return false,
            };
        }
        false
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

    /// A borrow value read out of an existing place *aliases* that place's
    /// borrow: a copy or move of a `read`/`write` borrow — or of a `slice`/
    /// `slice_mut` or a `str` view, each a fat pointer into its backing the same
    /// way — points into the same borrowed memory, so the loan(s) the source
    /// binding holds must extend to wherever the value lands (design §2.3). This is
    /// what makes `let c = b;` (or `let s2 = s;`) keep the source loan live under
    /// the copy; without it the copy sheds the loan and a later move/free/realloc
    /// of the borrowed backing is a use-after-free the checker misses. Fresh
    /// transient loans (same place, kind, span) are recorded and marked carried, so
    /// the landing binding anchors them to its own live range exactly as a fresh
    /// borrow would.
    pub(super) fn propagate_place_loans(&mut self, place: &Option<Place>, ty: &Type) {
        let root = match place {
            Some(p)
                if matches!(
                    ty,
                    Type::Borrow(_)
                        | Type::BorrowMut(_)
                        | Type::Slice(_)
                        | Type::SliceMut(_)
                        | Type::Str
                ) =>
            {
                p.root.clone()
            }
            _ => return,
        };
        let sources: Vec<LoanInfo> = self
            .f
            .loans
            .iter()
            .filter(|l| matches!(&l.anchor, Anchor::Binding(n) if *n == root))
            .cloned()
            .collect();
        if sources.is_empty() {
            return;
        }
        let mut ids = Vec::new();
        for li in sources {
            let id = self.f.loans.len();
            self.f.loans.push(LoanInfo {
                place: li.place,
                kind: li.kind,
                span: li.span,
                anchor: Anchor::Temp,
            });
            ids.push(id);
        }
        self.f.carried = ids;
        self.f.carried_prov = Some(root);
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

    /// The read-only contract rule (review #3, 2026-07-07; design 0001 §7.3):
    /// inside a `requires`/`ensures`/`assert` condition, moves of non-copy
    /// values, `write`-borrows, `out` arguments, and calls that consume or mutate
    /// an argument are rejected — a check that consumes or mutates is incoherent
    /// under P8. A no-op outside a contract clause.
    fn reject_in_contract(&mut self, span: Span, what: &str) {
        if self.f.in_contract {
            self.diags.push(
                Diag::error(
                    "E0708",
                    format!("{what} is not allowed inside a contract clause"),
                    span,
                )
                .with_note(
                    "`requires`/`ensures`/`assert` conditions are read-only: no moves, `write`-borrows, `out` arguments, or calls that take an argument by `take` (non-copy), `write`, or `out` (P8; review #3, 2026-07-07)",
                    None,
                ),
            );
        }
    }

    /// The static name a place expression roots at, if that root is a `static`
    /// item not shadowed by a local. Statics are immutable (review #3,
    /// 2026-07-07): used to reject assignment / `write`-borrow / `out` of a
    /// static. `None` for locals, unknown names, and non-place expressions.
    pub(super) fn static_place_root(&self, e: &Expr) -> Option<String> {
        let root = expr::expr_place_root(e)?;
        if self.lookup_local(&root).is_some() {
            return None;
        }
        if self.items.statics.contains_key(&root) {
            Some(root)
        } else {
            None
        }
    }

    /// Reject a mutating use of a `static` (assignment / `write`-borrow / `out`).
    /// Statics are immutable; reading and `read`-borrowing stay legal.
    fn reject_static_mutation(&mut self, e: &Expr, verb: &str, span: Span) {
        if let Some(name) = self.static_place_root(e) {
            self.diags.push(
                Diag::error(
                    "E0311",
                    format!("cannot {verb} the immutable `static` `{name}`"),
                    span,
                )
                .with_note(
                    "statics hold vtables and constants and are immutable; reading and `read`-borrowing are allowed (a mutable global is a recorded future design question, not an accident) (review #3, 2026-07-07)",
                    None,
                ),
            );
        }
    }

    fn note_alloc(&mut self, span: Span, reason: impl Into<String>) {
        self.f.alloc.note(span, reason);
    }

    // ----- driver ---------------------------------------------------------

    fn check_fn(&mut self, f: &FnDecl) {
        let sig = self.items.fns[&f.name].clone();
        self.check_fn_with_sig(f, &sig);
    }

    /// Check a function body against an explicit signature. Used both for
    /// top-level `fn`s and for synthetic drop-hook functions (finding 4).
    fn check_fn_with_sig(&mut self, f: &FnDecl, sig: &FnSig) {
        self.f = FnState::empty();
        self.f.ret_ty = sig.ret.clone();
        self.f.ret_is_borrow = sig.ret.is_borrow_kind();
        self.f.ret_region = sig.ret_region.clone();
        self.f.sig_params = sig
            .params
            .iter()
            .map(|p| (p.name.clone(), is_borrow_param(p), p.region.clone()))
            .collect();
        self.check_signature_regions(sig);

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

        // requires clauses are ordinary boolean expressions checked in the entry
        // block (design §7.3). They are read-only (review #3, 2026-07-07).
        self.f.in_contract = true;
        for r in &f.requires {
            let t = self.check_expr(r, Use::Value);
            self.expect_bool(&t, r.span, "a `requires` clause");
        }
        self.f.in_contract = false;

        self.check_block_stmts(&f.body);

        // Fall-through: a body that runs off its end is an implicit unit return
        // (an all-paths-return error for a non-unit function; Stage 3 flags it).
        if let Some(cur) = self.f.cur {
            self.set_term(cur, Term::FallThrough);
        }

        // ensures clauses may reference `result` (design §7.3) and are read-only
        // (review #3, 2026-07-07). Checked in two passes:
        //
        //   Pass 1 (type + read-only, reported once, off the CFG): type-check
        //   each clause, flag `result` misuse, and apply the read-only rule.
        //
        //   Pass 2 (dataflow, per return point): re-emit each clause's accesses
        //   into every normal-return block, against that block's post-body state.
        //   A read of a place moved/consumed by the body is then the ordinary
        //   E0301, exactly as if the read were written at the return. We analyze
        //   *once per return block* (not against a meet of the return states):
        //   it is the simpler option — it reuses the existing per-block emit and
        //   the init dataflow with no extra meet machinery — and it is sound and
        //   maximally precise, since each clause is evaluated at runtime at each
        //   return and here meets exactly that return's own state (a meet would
        //   additionally demand initialized-on-all-return-paths, rejecting
        //   programs whose clause is well-defined at every actual return). The
        //   direct diagnostics pass 2 re-raises duplicate pass 1's, so they are
        //   truncated away; only the emitted actions (driving E0301) persist.
        self.f.in_ensures = true;
        self.f.in_contract = true;
        let saved = self.f.cur;
        self.f.cur = None; // pass 1: contract exprs are not part of the CFG
        for e in &f.ensures {
            let t = self.check_expr(e, Use::Value);
            self.expect_bool(&t, e.span, "an `ensures` clause");
        }
        if !f.ensures.is_empty() {
            let diag_mark = self.diags.len();
            let ret_blocks: Vec<usize> = (0..self.f.blocks.len())
                .filter(|&b| matches!(self.f.blocks[b].term, Term::Return | Term::FallThrough))
                .collect();
            self.f.emit_contract = true;
            for b in ret_blocks {
                self.f.cur = Some(b);
                for e in &f.ensures {
                    self.check_expr(e, Use::Value);
                }
            }
            self.f.emit_contract = false;
            self.diags.truncate(diag_mark);
        }
        self.f.cur = saved;
        self.f.in_ensures = false;
        self.f.in_contract = false;

        // Run the value-gear dataflow over the finished CFG.
        let mut cfg = Cfg {
            blocks: std::mem::take(&mut self.f.blocks),
            entry,
            preds: Vec::new(),
        };
        cfg.compute_preds();
        let out_params = self.f.out_params.clone();
        // Owned (non-out) parameters that own a `Box`: dropping one that is still
        // live at function exit frees, so the body is allocator-effecting
        // (finding 4 of 2026-07-07; §6.3). The interpreter drops the parameter
        // scope on return (`interp/eval.rs` `call`), so this drop obligation is
        // real and static-schedulable here.
        // `box_subpaths` reaches both `Box` sub-places and alloc-on-drop-hooked
        // types (retest 2026-07-08), so a non-empty result is the general
        // "freeing drop" predicate here — not just `bears_box`.
        let param_box: Vec<(String, Vec<Vec<String>>)> = sig
            .params
            .iter()
            .filter(|p| p.mode != ParamMode::Out)
            .map(|p| (p.name.clone(), box_subpaths(&p.lowered, self.items)))
            .filter(|(_, paths)| !paths.is_empty())
            .collect();
        let box_drop_sites = init::analyze(&cfg, &entry_state, &out_params, &param_box, &mut self.diags);
        for (span, reason) in box_drop_sites {
            self.note_alloc(span, reason);
        }

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

        // The `foreign` effect discharge decision (design 0011 §2). Depends on the
        // whole set of foreign sites gathered during the body walk.
        let (discharges, propagates, needs_mark) = self.f.foreign.resolve(f.boundary, f.foreign);
        if needs_mark {
            let (site, reason) = self
                .f
                .foreign
                .externs
                .first()
                .map(|e| (e.span, format!("reaches undischarged foreign trust via `{}`", e.name)))
                .or_else(|| self.f.foreign.foreign_candor.clone().map(|(s, n)| (s, format!("calls `foreign` function `{n}`"))))
                .unwrap_or((f.span, "reaches foreign trust".to_string()));
            self.diags.push(
                Diag::error(
                    "E1103",
                    format!("function `{}` reaches undischarged foreign trust but is not marked `foreign`", f.name),
                    site,
                )
                .with_note(reason, Some(site))
                .with_note(
                    "add `foreign` to the signature, or discharge it in a `boundary` wrapper whose externs all carry `trust` clauses (design 0011 §2)",
                    Some(f.span),
                ),
            );
        }
        self.foreign_report.push(ForeignFnInfo {
            name: f.name.clone(),
            boundary: f.boundary,
            discharges,
            propagates,
        });
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
        let saved = self.expected_ty.take();
        self.expected_ty = Some(expected.clone());
        let t = self.check_expr(expr, Use::Value);
        self.expected_ty = saved;
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
            TyKind::Named(n) if n == "str" => Type::Str,
            TyKind::Named(n) => {
                if self.type_params.iter().any(|p| p == n) {
                    Type::Param(n.clone())
                } else if self.items.structs.contains_key(n) || self.items.enums.contains_key(n) {
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
                    foreign: fp.foreign,
                    ret: Box::new(self.resolve_ty(&fp.ret)),
                })
            }
            TyKind::App { name, args } => Type::App(
                name.clone(),
                args.iter().map(|a| self.resolve_ty(a)).collect(),
            ),
            TyKind::Proj { base, assoc } => {
                // `Base::Assoc` (design 0009 §2.2): `Base` must be `Self` or a
                // type parameter in scope carrying a bound interface that declares
                // an associated type; otherwise the projection is unrooted.
                let ok = base == "Self" || self.type_params.iter().any(|p| p == base);
                if !ok {
                    self.diags.push(
                        Diag::error(
                            "E1017",
                            format!("associated-type projection `{base}::{assoc}` has no bounded base"),
                            ty.span,
                        )
                        .with_note("`Base::Assoc` needs `Base` a type parameter bounded by an interface with that member (design 0009 §2.2)", None),
                    );
                    Type::Error
                } else {
                    Type::Proj(base.clone(), assoc.clone())
                }
            }
        }
    }

    /// Region well-formedness for a borrow-kind-returning signature (design §3.3):
    /// two-plus borrow params returning a borrow OR view require a region variable;
    /// an explicit return region must be declared and tag some borrow param.
    fn check_signature_regions(&mut self, sig: &crate::resolve::FnSig) {
        if !sig.ret.is_borrow_kind() {
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

    pub(super) fn in_regime(&self) -> bool {
        self.f.regime_depth > 0
    }
    pub(super) fn regime_enter(&mut self) {
        self.f.regime_depth += 1;
    }
    pub(super) fn regime_exit(&mut self) {
        self.f.regime_depth -= 1;
    }
    pub(super) fn is_real(&self) -> bool {
        self.real
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

    /// A foreign (`extern` or foreign-fn-pointer) call is unsafe in principle
    /// (design 0011 §2 rule 2): it reuses the one `unsafe` valve, not a new one.
    pub(super) fn require_unsafe_foreign(&mut self, span: Span, what: &str) {
        if !self.f.in_unsafe {
            self.diags.push(
                Diag::error(
                    "E0501",
                    format!("foreign call to {what} requires an `unsafe` block"),
                    span,
                )
                .with_note("a foreign call reuses the one `unsafe` valve; it points it at a foreign symbol (design 0011 §2)", None),
            );
        }
    }

    /// Record a direct `extern` (ground-source) call in the foreign accumulator.
    pub(super) fn record_extern_call(&mut self, name: &str, has_trust: bool, span: Span) {
        self.f.foreign.externs.push(crate::check::foreign::ExternCall {
            name: name.to_string(),
            has_trust,
            span,
        });
    }

    /// Record a propagated foreign contribution (a call to a `foreign` Candor fn or
    /// through a foreign fn-pointer) — undischargeable at this wrapper.
    pub(super) fn record_foreign_candor(&mut self, name: &str, span: Span) {
        if self.f.foreign.foreign_candor.is_none() {
            self.f.foreign.foreign_candor = Some((span, name.to_string()));
        }
    }
}

/// A parameter that is itself a borrow (design §3.3): a `read`/`write` mode
/// parameter, or a by-value borrow-kind (slice/borrow) parameter.
fn is_borrow_param(p: &crate::resolve::ParamInfo) -> bool {
    matches!(p.mode, ParamMode::Read | ParamMode::Write) || p.decl_ty.is_borrow_kind()
}
