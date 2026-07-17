//! AST → MIR lowering (design 0010 §2, §5). Lowers the *checked, resolved,
//! monomorphized* AST into the checked MIR, carrying the analysis tier's facts.
//!
//! Stage-A subset: non-generic functions over scalars/booleans — arithmetic (all
//! three regimes and every fault kind: overflow, div-by-zero, conv-loss),
//! comparisons, short-circuit `&&`/`||`, `if`/`while`/`loop`, `return`/`break`/
//! `continue`, `assert`/`panic`, `requires`/`ensures` contracts, `trace`
//! observables, and direct calls to user functions with value (`take`) scalar
//! parameters. Anything outside this subset lowers to [`LowerError::Unsupported`],
//! which the gate harness records as "out-of-subset" (not a failure) — the honest
//! coverage boundary.
//!
//! Fault spans are matched to the oracle by *simulating the tree-walker's
//! `cur_span` threading at lowering time* and baking the exact `(k, s)` into each
//! fault edge (see `cur_span` mutations below); the gate proves the match.

use std::collections::HashMap;

use crate::ast::{
    BinOp, Block, Expr, ExprKind, FieldInit, FnDecl, Item, MatchArm, ParamMode, PatKind, PrefixOp,
    Program, Stmt, StmtKind, Ty, TyKind, UnOp,
};
use crate::interp::FaultKind;
use crate::resolve::Items;
use crate::span::Span;
use crate::token::ScalarTy;
use crate::interp::layout::Layout;
use crate::interp::mem::round_up;
use crate::types::{is_copy, needs_drop, scalar_name, ArrayLen, Type};

use super::{
    check_invariants, BasicBlock, BlockId, CollOp, FaultEdge, LocalDecl, LocalId, MirFn,
    MirProgram, Operand, Place, Predicate, Proj, Regime, ReplayPolicy, Rvalue, StaticInit,
    Statement, StatementKind, Terminator,
};

/// A construct outside the Stage-A MIR subset. The gate treats it as out-of-subset
/// coverage, not a gate failure.
#[derive(Clone, Debug)]
pub struct LowerError(pub String);

fn unsupported<T>(what: impl Into<String>) -> Result<T, LowerError> {
    Err(LowerError(what.into()))
}

/// Interface-impl dispatch, built from the monomorphized AST (design 0007): the
/// generics layer resolved these; `resolve::Items` does not carry them.
/// One `From`-impl record: (iface base, iface args, target nominal, method map).
type FromImpl = (String, Vec<Type>, String, HashMap<String, String>);

/// The borrow mode a match scrutinee is held under (design 0001 §4.1).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Hold {
    Owned,
    Shared,
    Excl,
}

#[derive(Default)]
struct ImplTables {
    /// (target nominal, interface, method name) -> the impl method's free-function
    /// name. The interface is part of the key: two interfaces may share a method
    /// name on one target, and dispatch must run the one the checker resolved.
    dispatch: HashMap<(String, String, String), String>,
    /// (target nominal, method name) -> the interface of the *first* impl (in item
    /// order) providing that method — the coherent default when a call site carries
    /// no resolved interface, matching the checker's first-match on a direct call.
    first_iface: HashMap<(String, String), String>,
    /// (iface base, iface args, target, methods) — for `From` resolution in `?`.
    from_impls: Vec<FromImpl>,
}

fn build_impl_tables(program: &Program) -> ImplTables {
    let mut t = ImplTables::default();
    for it in &program.items {
        if let Item::Impl(im) = it {
            // A scalar impl target (`i64`) dispatches under its spelling, keyed the
            // same way a nominal target is (design 0007 §2.3).
            let target = match &im.target.kind {
                TyKind::Named(n) => n.clone(),
                TyKind::Scalar(s) => scalar_name(*s).to_string(),
                _ => continue,
            };
            let iface_args: Vec<Type> = im.iface_args.iter().map(resolve_impl_ty).collect();
            let mut methods = HashMap::new();
            for m in &im.methods {
                let fnname = crate::generics::impl_method_fn_name(&im.iface, &iface_args, &target, &m.name);
                t.dispatch
                    .entry((target.clone(), im.iface.clone(), m.name.clone()))
                    .or_insert_with(|| fnname.clone());
                t.first_iface
                    .entry((target.clone(), m.name.clone()))
                    .or_insert_with(|| im.iface.clone());
                methods.insert(m.name.clone(), fnname);
            }
            t.from_impls.push((crate::generics::base_name(&im.iface).to_string(), iface_args, target.clone(), methods));
        }
    }
    t
}

/// Resolve an impl's interface-argument AST type to a semantic type (concrete in
/// a monomorphized program) — mirrors the oracle's `resolve_impl_ty`.
fn resolve_impl_ty(ty: &Ty) -> Type {
    match &ty.kind {
        TyKind::Scalar(s) => Type::Scalar(*s),
        TyKind::Named(n) if n == "str" => Type::Str,
        TyKind::Named(n) => Type::Named(n.clone()),
        TyKind::Box(e) => Type::Box(Box::new(resolve_impl_ty(e))),
        TyKind::BoxResult(e) => Type::BoxResult(Box::new(resolve_impl_ty(e))),
        TyKind::App { name, args } => Type::App(name.clone(), args.iter().map(resolve_impl_ty).collect()),
        TyKind::RawPtr(e) => Type::RawPtr(Box::new(resolve_impl_ty(e))),
        TyKind::Slice(e) => Type::Slice(Box::new(resolve_impl_ty(e))),
        TyKind::SliceMut(e) => Type::SliceMut(Box::new(resolve_impl_ty(e))),
        _ => Type::Error,
    }
}


/// Lower a checked, resolved program to MIR. Generic programs must be
/// monomorphized first; a still-generic item is reported as unsupported here.
pub fn lower_checked(program: &Program, items: &Items) -> Result<MirProgram, LowerError> {
    let mut consts = HashMap::new();
    for it in &program.items {
        if let Item::Static(st) = it {
            if let ExprKind::IntLit { value, .. } = &st.value.kind {
                consts.insert(st.name.clone(), *value);
            }
        }
    }
    // Build the interface-impl dispatch tables from the (monomorphized) AST — the
    // impls the generics layer resolved, not carried in `Items` (design 0007). A
    // method call `recv.m` lowers to the mono-resolved free fn; cross-type `?`
    // finds its `From` impl the same way the oracle does.
    let impls = build_impl_tables(program);

    // Assign fn-pointer ids up front over every callable name (user fns, drop
    // hooks, static inits), so a fn value / vtable slot is a stable `u64` index
    // (design 0007 §6.2). The lowering bakes ids; the interpreter resolves them.
    let mut fn_ptrs: Vec<String> = Vec::new();
    let mut fn_ptr_id: HashMap<String, usize> = HashMap::new();
    let reg = |name: String, fn_ptrs: &mut Vec<String>, fn_ptr_id: &mut HashMap<String, usize>| {
        if !fn_ptr_id.contains_key(&name) {
            fn_ptr_id.insert(name.clone(), fn_ptrs.len());
            fn_ptrs.push(name);
        }
    };
    for it in &program.items {
        match it {
            Item::Struct(sd) if sd.drop_hook.is_some() => {
                reg(format!("<drop {}>", sd.name), &mut fn_ptrs, &mut fn_ptr_id);
            }
            Item::Static(st) => {
                reg(format!("<init {}>", st.name), &mut fn_ptrs, &mut fn_ptr_id);
            }
            Item::Fn(fnd) => {
                reg(fnd.name.clone(), &mut fn_ptrs, &mut fn_ptr_id);
            }
            _ => {}
        }
    }

    let mut fns = Vec::new();
    let mut fn_index = HashMap::new();
    let mut drop_hooks = HashMap::new();
    let mut statics = Vec::new();
    // Lower each struct `drop` hook to an ordinary MIR function taking `write self`
    // (INV-DROP: the schedule is static; the hook is called at the drop point).
    for it in &program.items {
        if let Item::Struct(sd) = it {
            if let Some(block) = &sd.drop_hook {
                let hook_name = format!("<drop {}>", sd.name);
                let mut lw = Lowerer::new(items, &consts, &fn_ptr_id, &impls);
                let mf = lw.lower_hook(&hook_name, &sd.name, block)?;
                check_invariants(&mf);
                drop_hooks.insert(sd.name.clone(), hook_name.clone());
                fn_index.insert(hook_name, fns.len());
                fns.push(mf);
            }
        }
    }
    // Lower each `static`'s initializer to an `<init NAME>` fn returning the value.
    for it in &program.items {
        if let Item::Static(st) = it {
            let sty = items
                .statics
                .get(&st.name)
                .map(|(t, _)| t.clone())
                .ok_or_else(|| LowerError(format!("no type for static `{}`", st.name)))?;
            let init_name = format!("<init {}>", st.name);
            let mut lw = Lowerer::new(items, &consts, &fn_ptr_id, &impls);
            let mf = lw.lower_static_init(&init_name, &sty, &st.value)?;
            check_invariants(&mf);
            fn_index.insert(init_name.clone(), fns.len());
            fns.push(mf);
            statics.push(StaticInit { name: st.name.clone(), ty: sty, init_fn: init_name });
        }
    }
    for it in &program.items {
        if let Item::Fn(fnd) = it {
            if !fnd.type_params.is_empty() {
                return unsupported(format!("generic fn `{}`", fnd.name));
            }
            let mut lw = Lowerer::new(items, &consts, &fn_ptr_id, &impls);
            let mf = lw.lower_fn(fnd)?;
            debug_assert_eq!(mf.name, fnd.name);
            check_invariants(&mf);
            fn_index.insert(mf.name.clone(), fns.len());
            fns.push(mf);
        }
    }
    Ok(MirProgram { fns, fn_index, drop_hooks, fn_ptrs, statics })
}

type LR<T> = Result<T, LowerError>;

struct Scope {
    locals: Vec<(String, LocalId, Type)>,
}

struct Loop {
    continue_bb: BlockId,
    break_bb: BlockId,
    scope_depth: usize,
}

struct Lowerer<'a> {
    items: &'a Items,
    consts: &'a HashMap<String, u64>,
    fn_ptr_id: &'a HashMap<String, usize>,
    impls: &'a ImplTables,
    /// Static move state (the compile-time analog of the oracle's `MoveMask`):
    /// per-local, the field-name paths already moved out. Consulted when emitting
    /// a `Drop` so the drop skips moved sub-paths — the static mask baked into the
    /// MIR (design 0010 §2 INV-DROP), NO runtime flag.
    moves: HashMap<LocalId, Vec<Vec<String>>>,
    locals: Vec<LocalDecl>,
    blocks: Vec<BasicBlock>,
    cur: BlockId,
    reachable: bool,
    scopes: Vec<Scope>,
    loops: Vec<Loop>,
    regime: Regime,
    ret_ty: Type,
    result_local: Option<LocalId>,
    /// Mirror of the oracle's `cur_span`, threaded through lowering so fault edges
    /// carry the exact span the tree-walker delivers.
    cur_span: Span,
}

impl<'a> Lowerer<'a> {
    fn new(
        items: &'a Items,
        consts: &'a HashMap<String, u64>,
        fn_ptr_id: &'a HashMap<String, usize>,
        impls: &'a ImplTables,
    ) -> Lowerer<'a> {
        Lowerer {
            items,
            consts,
            fn_ptr_id,
            impls,
            moves: HashMap::new(),
            locals: Vec::new(),
            blocks: Vec::new(),
            cur: 0,
            reachable: true,
            scopes: Vec::new(),
            loops: Vec::new(),
            regime: Regime::Checked,
            ret_ty: Type::unit(),
            result_local: None,
            cur_span: Span::point(0),
        }
    }

    // ---- construction helpers ----
    fn new_local(&mut self, ty: Type, name: Option<String>) -> LocalId {
        let drop_obligation = needs_drop(&ty, self.items);
        let id = self.locals.len();
        self.locals.push(LocalDecl { ty, name, drop_obligation });
        id
    }
    fn new_block(&mut self) -> BlockId {
        let id = self.blocks.len();
        self.blocks.push(BasicBlock {
            stmts: Vec::new(),
            term: Terminator::Return,
        });
        id
    }
    fn emit(&mut self, kind: StatementKind, span: Span, observable: bool) {
        if !self.reachable {
            return;
        }
        self.blocks[self.cur].stmts.push(Statement { kind, span, observable });
    }
    /// INV-OBS-ORDER: mark the just-emitted statement of the current block as an
    /// observable (a rawptr/MMIO access — the formalization's substrate note, §2.1:
    /// a volatile load/store through a raw pointer). Marking is conservative (it
    /// only ever ADDS an ordering constraint the backend must honor); in-model
    /// borrow derefs (§2.2) are left non-observable.
    fn mark_last_observable(&mut self) {
        if !self.reachable {
            return;
        }
        if let Some(st) = self.blocks[self.cur].stmts.last_mut() {
            st.observable = true;
        }
    }
    /// Terminate the current block; the region becomes unreachable until a caller
    /// switches to a fresh labeled block.
    fn terminate(&mut self, term: Terminator) {
        if !self.reachable {
            return;
        }
        self.blocks[self.cur].term = term;
        self.reachable = false;
    }
    fn switch_to(&mut self, bb: BlockId) {
        self.cur = bb;
        self.reachable = true;
    }
    /// Assign an rvalue into a fresh temp local of the given type; return the temp.
    fn emit_temp(&mut self, ty: Type, rv: Rvalue, span: Span) -> LocalId {
        let id = self.new_local(ty, None);
        self.emit(StatementKind::Assign(id, rv), span, false);
        id
    }

    // ---- scopes / env ----
    fn push_scope(&mut self) {
        self.scopes.push(Scope { locals: Vec::new() });
    }
    fn pop_scope_with_drops(&mut self) {
        let sc = self.scopes.pop().unwrap();
        // INV-DROP: drop the scope's owned locals in reverse declaration order.
        let ids: Vec<LocalId> = sc.locals.iter().rev().map(|(_, id, _)| *id).collect();
        for id in ids {
            self.emit_drop(id);
        }
    }
    /// Emit drops for the innermost scope's owned locals WITHOUT popping it — used
    /// on a guarded arm's guard-false edge, where the arm's bindings die as
    /// control falls through to the next arm but the scope is still owned by the
    /// body path (which pops it normally).
    fn emit_scope_drops_no_pop(&mut self) {
        let ids: Vec<LocalId> = self
            .scopes
            .last()
            .unwrap()
            .locals
            .iter()
            .rev()
            .map(|(_, id, _)| *id)
            .collect();
        for id in ids {
            self.emit_drop(id);
        }
    }

    /// Emit a `Drop` for a needs-drop local, pruned by the static move mask: skip
    /// entirely if the whole value is moved; otherwise carry the moved sub-paths
    /// so the drop runs only the still-owned remainder (INV-DROP, no runtime flag).
    fn emit_drop(&mut self, id: LocalId) {
        if !self.reachable || !self.locals[id].drop_obligation {
            return;
        }
        let mask = self.moves.get(&id).cloned().unwrap_or_default();
        // Whole-value moved (empty path in the mask): nothing to drop.
        if mask.iter().any(|m| m.is_empty()) {
            return;
        }
        self.emit(StatementKind::Drop { local: id, moved: mask }, Span::point(0), false);
    }

    /// Emit the drop schedule for *every* live scope, innermost-first — the drops
    /// that run when control leaves the function via `return` (INV-DROP). The
    /// caller terminates immediately after, so the natural scope-exit drops below
    /// become unreachable (no double-drop).
    fn emit_return_drops(&mut self) {
        let ids: Vec<LocalId> = self
            .scopes
            .iter()
            .rev()
            .flat_map(|sc| sc.locals.iter().rev().map(|(_, id, _)| *id))
            .collect();
        for id in ids {
            self.emit_drop(id);
        }
    }

    /// Emit the drop schedule for the scopes opened *inside* the innermost loop
    /// body (down to `keep` remaining scopes) — the drops that run on `break`/
    /// `continue` (INV-DROP).
    fn emit_loop_exit_drops(&mut self, keep: usize) {
        let ids: Vec<LocalId> = self
            .scopes
            .iter()
            .skip(keep)
            .rev()
            .flat_map(|sc| sc.locals.iter().rev().map(|(_, id, _)| *id))
            .collect();
        for id in ids {
            self.emit_drop(id);
        }
    }

    // ---- static move tracking (the compile-time MoveMask) ----

    /// The (root local, field-name path) a place expression denotes, if it is a
    /// local reached by only `.field` steps (the shape the checker permits a
    /// non-copy move out of; deref/index moves are E0310-rejected). `None` for a
    /// non-trackable operand (temp, call result, opaque place).
    fn ast_place_path(&self, e: &Expr) -> Option<(LocalId, Vec<String>)> {
        match &e.kind {
            ExprKind::Paren(i) | ExprKind::OutArg(i) => self.ast_place_path(i),
            ExprKind::Ident(name) => self.lookup(name).map(|(id, _)| (id, Vec::new())),
            ExprKind::Field { base, field, .. } => {
                let (id, mut path) = self.ast_place_path(base)?;
                path.push(field.clone());
                Some((id, path))
            }
            _ => None,
        }
    }

    /// Mark the value `e` names as moved out (a non-copy by-value consume), so its
    /// scheduled drop is pruned. No-op for non-trackable operands.
    fn mark_moved(&mut self, e: &Expr) {
        if let Some((id, path)) = self.ast_place_path(e) {
            self.moves.entry(id).or_default().push(path);
        }
    }

    /// Mark a moved local path re-initialized (a reassignment restores ownership).
    fn set_owned(&mut self, id: LocalId, path: &[String]) {
        if let Some(v) = self.moves.get_mut(&id) {
            v.retain(|m| !(prefix(m, path) || prefix(path, m)));
        }
    }

    fn bind(&mut self, name: &str, id: LocalId, ty: Type) {
        self.scopes
            .last_mut()
            .unwrap()
            .locals
            .push((name.to_string(), id, ty));
    }
    fn lookup(&self, name: &str) -> Option<(LocalId, Type)> {
        for sc in self.scopes.iter().rev() {
            for (n, id, ty) in sc.locals.iter().rev() {
                if n == name {
                    return Some((*id, ty.clone()));
                }
            }
        }
        None
    }

    // ---- type resolution + layout (shared substrate) ----
    fn lay(&self) -> Layout<'_> {
        Layout { items: self.items, consts: self.consts }
    }
    fn size_of(&self, ty: &Type) -> u64 {
        self.lay().size_of(ty)
    }
    fn align_of(&self, ty: &Type) -> u64 {
        self.lay().align_of(ty)
    }
    fn stride_of(&self, elem: &Type) -> u64 {
        round_up(self.size_of(elem), self.align_of(elem))
    }

    fn resolve_ty(&self, ty: &Ty) -> LR<Type> {
        Ok(match &ty.kind {
            TyKind::Scalar(s) => Type::Scalar(*s),
            TyKind::Named(n) if n == "str" => Type::Str,
            TyKind::Named(n) => Type::Named(n.clone()),
            TyKind::Array { size, elem } => {
                let len = match &size.kind {
                    ExprKind::IntLit { value, .. } => ArrayLen::Lit(*value),
                    ExprKind::Ident(n) => ArrayLen::Named(n.clone()),
                    _ => ArrayLen::Unknown,
                };
                Type::Array(Box::new(self.resolve_ty(elem)?), len)
            }
            TyKind::Borrow(e) => Type::Borrow(Box::new(self.resolve_ty(e)?)),
            TyKind::BorrowMut(e) => Type::BorrowMut(Box::new(self.resolve_ty(e)?)),
            TyKind::RawPtr(e) => Type::RawPtr(Box::new(self.resolve_ty(e)?)),
            TyKind::Slice(e) => Type::Slice(Box::new(self.resolve_ty(e)?)),
            TyKind::SliceMut(e) => Type::SliceMut(Box::new(self.resolve_ty(e)?)),
            TyKind::Box(e) => Type::Box(Box::new(self.resolve_ty(e)?)),
            TyKind::BoxResult(e) => Type::BoxResult(Box::new(self.resolve_ty(e)?)),
            TyKind::FnPtr(fp) => Type::FnPtr(crate::types::FnPtrTy {
                params: fp
                    .params
                    .iter()
                    .map(|pp| Ok((pp.mode, self.resolve_ty(&pp.ty)?)))
                    .collect::<LR<Vec<_>>>()?,
                alloc: fp.alloc,
                foreign: fp.foreign,
                ret: Box::new(self.resolve_ty(&fp.ret)?),
            }),
            // Compiler-known collections `Vec[T]`/`Map[V]` (design 0013): a nominal
            // type application that lays out through the shared `Layout` App path.
            TyKind::App { name, args } => Type::App(
                name.clone(),
                args.iter().map(|a| self.resolve_ty(a)).collect::<LR<Vec<_>>>()?,
            ),
            other => return unsupported(format!("type {other:?}")),
        })
    }

    /// Is `ty` a scalar/pointer-width value (fits in a local read as an int)?
    fn is_wordy(&self, ty: &Type) -> bool {
        matches!(
            ty,
            Type::Scalar(_)
                | Type::Borrow(_)
                | Type::BorrowMut(_)
                | Type::RawPtr(_)
                | Type::FnPtr(_)
        )
    }

    fn concretize(&self, ty: &Type) -> ScalarTy {
        match ty {
            Type::Scalar(s) => *s,
            _ => ScalarTy::I64,
        }
    }
    fn int_type(&self, suffix: &Option<ScalarTy>, expected: Option<&Type>) -> ScalarTy {
        if let Some(s) = suffix {
            return *s;
        }
        if let Some(Type::Scalar(s)) = expected {
            if s.is_integer() {
                return *s;
            }
        }
        ScalarTy::I64
    }

    // ---- function lowering ----
    fn lower_fn(&mut self, fnd: &FnDecl) -> LR<MirFn> {
        let sig = self
            .items
            .fns
            .get(fnd.name.as_str())
            .ok_or_else(|| LowerError(format!("no sig for `{}`", fnd.name)))?;
        self.ret_ty = sig.ret.clone();
        // _0 = return place.
        let _ret = self.new_local(sig.ret.clone(), None);
        // Parameters _1..=n (value/`take` scalars only).
        let mut params = Vec::new();
        for p in &sig.params {
            match p.mode {
                ParamMode::Read | ParamMode::Write => {
                    // A `read`/`write` param is a `Borrow`/`BorrowMut` — an 8-byte
                    // address (word-sized).
                    if !self.is_wordy(&p.lowered) {
                        return unsupported("non-word borrow parameter");
                    }
                }
                ParamMode::Take => {
                    // By-value aggregate params (drop-inert or needs-drop) are owned;
                    // the param scope's drop schedule runs any drop at fn exit.
                }
                _ => return unsupported(format!("param mode {:?}", p.mode)),
            }
            let id = self.new_local(p.lowered.clone(), Some(p.name.clone()));
            params.push((p.name.clone(), id, p.lowered.clone()));
        }
        // A `result` local for `ensures`.
        if !fnd.ensures.is_empty() {
            if !is_scalar(&sig.ret) {
                return unsupported("non-scalar `ensures` result");
            }
            self.result_local = Some(self.new_local(sig.ret.clone(), Some("result".to_string())));
        }

        // Contract predicates are lowered into their own blocks over the same
        // frame (params + result are in scope).
        self.push_scope();
        for (n, id, ty) in &params {
            self.bind(n, *id, ty.clone());
        }
        let requires = {
            let mut ps = Vec::new();
            for r in &fnd.requires {
                ps.push(self.lower_predicate(r, FaultKind::Requires)?);
            }
            ps
        };

        // Body entry block.
        let entry = self.new_block();
        self.switch_to(entry);
        self.lower_block(&fnd.body)?;
        // Fall-off: return the (default) return place.
        if self.reachable {
            self.terminate(Terminator::Return);
        }

        // `ensures` predicates read `result`; bound by the interpreter at exit.
        let ensures = {
            let mut ps = Vec::new();
            for e in &fnd.ensures {
                if let Some(rl) = self.result_local {
                    self.bind("result", rl, self.ret_ty.clone());
                }
                ps.push(self.lower_predicate(e, FaultKind::Ensures)?);
            }
            ps
        };
        self.pop_scope_with_drops();

        Ok(MirFn {
            name: fnd.name.clone(),
            num_params: params.len(),
            result_local: self.result_local,
            locals: std::mem::take(&mut self.locals),
            blocks: std::mem::take(&mut self.blocks),
            entry,
            requires,
            ensures,
            replay: ReplayPolicy::Precise,
        })
    }

    /// Lower a struct `drop` hook block to a MIR function `<drop Name>(write self)`.
    fn lower_hook(&mut self, hook_name: &str, struct_name: &str, block: &Block) -> LR<MirFn> {
        self.ret_ty = Type::unit();
        let _ret = self.new_local(Type::unit(), None); // _0
        let self_ty = Type::BorrowMut(Box::new(Type::Named(struct_name.to_string())));
        let sid = self.new_local(self_ty.clone(), Some("self".to_string()));
        self.push_scope();
        self.bind("self", sid, self_ty);
        let entry = self.new_block();
        self.switch_to(entry);
        self.lower_block(block)?;
        if self.reachable {
            self.terminate(Terminator::Return);
        }
        self.pop_scope_with_drops(); // `self` is a borrow, not owned — nothing drops
        Ok(MirFn {
            name: hook_name.to_string(),
            num_params: 1,
            result_local: None,
            locals: std::mem::take(&mut self.locals),
            blocks: std::mem::take(&mut self.blocks),
            entry,
            requires: Vec::new(),
            ensures: Vec::new(),
            replay: ReplayPolicy::Precise,
        })
    }

    fn lower_predicate(&mut self, cond: &Expr, kind: FaultKind) -> LR<Predicate> {
        let bb = self.new_block();
        self.switch_to(bb);
        let (v, _) = self.lower_value(cond, Some(&Type::bool()))?;
        let span = self.cur_span; // oracle's cur_span at the contract fault
        let value = self.emit_temp(Type::bool(), Rvalue::Use(v), span);
        self.terminate(Terminator::Return);
        Ok(Predicate { entry: bb, value, span, kind })
    }

    // ---- blocks / statements ----
    fn lower_block(&mut self, b: &Block) -> LR<()> {
        self.push_scope();
        for s in &b.stmts {
            if !self.reachable {
                break;
            }
            self.lower_stmt(s)?;
        }
        self.pop_scope_with_drops();
        Ok(())
    }

    fn lower_stmt(&mut self, s: &Stmt) -> LR<()> {
        match &s.kind {
            StmtKind::Let { name, ty, init, .. } => {
                let decl = match ty {
                    Some(t) => Some(self.resolve_ty(t)?),
                    None => None,
                };
                match init {
                    Some(e) => match &decl {
                        Some(lty) => {
                            let lty = lty.clone();
                            let id = self.new_local(lty.clone(), Some(name.clone()));
                            self.lower_into(e, &Place::local(id), &lty)?;
                            self.bind(name, id, lty);
                        }
                        None => {
                            // No annotation (e.g. the `for` desugar's `let __it = list`):
                            // infer the type, then move/copy the value into the local.
                            match self.infer_ty(e) {
                                Some(oty) if !self.is_wordy(&oty) => {
                                    let id = self.new_local(oty.clone(), Some(name.clone()));
                                    self.lower_into(e, &Place::local(id), &oty)?;
                                    self.bind(name, id, oty);
                                }
                                _ => {
                                    let (op, oty) = self.lower_value(e, None)?;
                                    if !self.is_wordy(&oty) {
                                        return unsupported("non-word inferred local");
                                    }
                                    let id = self.new_local(oty.clone(), Some(name.clone()));
                                    self.emit(StatementKind::Assign(id, Rvalue::Use(op)), s.span, false);
                                    self.bind(name, id, oty);
                                }
                            }
                        }
                    },
                    None => {
                        let lty = decl.ok_or_else(|| LowerError("untyped uninit let".into()))?;
                        let id = self.new_local(lty.clone(), Some(name.clone()));
                        self.bind(name, id, lty);
                    }
                }
                Ok(())
            }
            StmtKind::Assign { target, value } => {
                let (place, tty) = self.lower_place(target)?;
                self.lower_into(value, &place, &tty)?;
                // A reassignment reinitializes the target place (design 0001 §1.5):
                // clear its moved paths so a later drop of it is not pruned.
                if let Some((id, path)) = self.ast_place_path(target) {
                    self.set_owned(id, &path);
                }
                Ok(())
            }
            StmtKind::Expr(e) => {
                self.lower_value(e, None)?;
                Ok(())
            }
        }
    }

    // ---- expressions ----
    /// Lower an expression to an operand and its type, emitting statements. Threads
    /// `cur_span` exactly as the oracle's `eval_value`/`eval_place`.
    fn lower_value(&mut self, e: &Expr, expected: Option<&Type>) -> LR<(Operand, Type)> {
        self.cur_span = e.span;
        match &e.kind {
            ExprKind::Paren(i) => self.lower_value(i, expected),
            ExprKind::IntLit { value, suffix } => {
                let sty = self.int_type(suffix, expected);
                Ok((Operand::Const(*value as i128, sty), Type::Scalar(sty)))
            }
            ExprKind::NegIntLit { value, suffix } => {
                let sty = self.int_type(suffix, expected);
                Ok((Operand::Const(-(*value as i128), sty), Type::Scalar(sty)))
            }
            ExprKind::FloatLit { bits, ty } => Ok((
                // The float bit pattern is carried (zero-extended) in the const slot.
                Operand::Const(*bits as i128, *ty),
                Type::Scalar(*ty),
            )),
            ExprKind::BoolLit(b) => Ok((Operand::Const(*b as i128, ScalarTy::Bool), Type::bool())),
            ExprKind::Ident(name) => {
                if let Some((id, ty)) = self.lookup(name) {
                    Ok((Operand::Local(id), ty))
                } else if let Some((sty, _)) = self.items.statics.get(name) {
                    // A `static` read by value (design 0008): load from its place.
                    if self.is_wordy(sty) {
                        let sty = sty.clone();
                        let (place, _) = self.lower_place(e)?;
                        let id = self.emit_temp(sty.clone(), Rvalue::Load { place, ty: sty.clone() }, self.cur_span);
                        Ok((Operand::Local(id), sty))
                    } else {
                        unsupported("aggregate static in value position")
                    }
                } else if let Some(id) = self.fn_ptr_id.get(name) {
                    // A function named as a value (design 0007 §6.2): its fn-ptr id.
                    Ok((Operand::Const(*id as i128, ScalarTy::U64), self.fnptr_ty(name)))
                } else {
                    unsupported(format!("unknown name `{name}`"))
                }
            }
            ExprKind::GenericVal { name, .. } => {
                // A monomorphized generic fn named as a value: its fn-ptr id.
                if let Some(id) = self.fn_ptr_id.get(name) {
                    Ok((Operand::Const(*id as i128, ScalarTy::U64), self.fnptr_ty(name)))
                } else {
                    unsupported(format!("generic value `{name}`"))
                }
            }
            ExprKind::Unary { op, expr } => self.lower_unary(*op, expr, expected, e.span),
            ExprKind::Binary { op, lhs, rhs } => self.lower_binary(*op, lhs, rhs, expected, e.span),
            ExprKind::Conv { ty, expr } => self.lower_conv(ty, expr),
            ExprKind::Bitcast { ty, expr } => self.lower_bitcast(ty, expr),
            ExprKind::Call { callee, args } => self.lower_call(callee, args, e.span),
            ExprKind::Block(b) => {
                self.lower_block(b)?;
                Ok(self.unit())
            }
            ExprKind::If { cond, then_blk, else_blk } => {
                self.lower_if(cond, then_blk, else_blk.as_deref())?;
                Ok(self.unit())
            }
            ExprKind::While { cond, body } => {
                self.lower_while(cond, body)?;
                Ok(self.unit())
            }
            ExprKind::Loop(b) => {
                self.lower_loop(b)?;
                Ok(self.unit())
            }
            // Sequential oracle (design 0012 §6): a `scope` lowers as a plain block,
            // a `spawn` as the task's call run at the spawn point.
            ExprKind::Scope(b) => {
                // Stage 2 (design 0012 §6): emit structured markers the NATIVE
                // backend turns into real thread creation + a join barrier; the
                // MIR interp (sequential oracle) treats them as no-ops.
                self.emit(StatementKind::ScopeBegin, e.span, false);
                self.lower_block(b)?;
                self.emit(StatementKind::ScopeEnd, e.span, false);
                Ok(self.unit())
            }
            ExprKind::Spawn(c) => {
                self.lower_spawn(c, e.span)?;
                Ok(self.unit())
            }
            ExprKind::Wrapping(b) => self.lower_regime(b, Regime::Wrapping),
            ExprKind::Saturating(b) => self.lower_regime(b, Regime::Saturating),
            ExprKind::Unsafe { body, .. } => {
                self.lower_block(body)?;
                Ok(self.unit())
            }
            ExprKind::Return(opt) => {
                self.lower_return(opt.as_deref())?;
                Ok(self.unit())
            }
            ExprKind::Break => {
                let (bb, depth) = self
                    .loops
                    .last()
                    .map(|l| (l.break_bb, l.scope_depth))
                    .ok_or_else(|| LowerError("break outside loop".into()))?;
                self.emit_loop_exit_drops(depth);
                self.terminate(Terminator::Goto(bb));
                Ok(self.unit())
            }
            ExprKind::Continue => {
                let (bb, depth) = self
                    .loops
                    .last()
                    .map(|l| (l.continue_bb, l.scope_depth))
                    .ok_or_else(|| LowerError("continue outside loop".into()))?;
                self.emit_loop_exit_drops(depth);
                self.terminate(Terminator::Goto(bb));
                Ok(self.unit())
            }
            ExprKind::Assert(c) => {
                let (cond, _) = self.lower_value(c, Some(&Type::bool()))?;
                let span = self.cur_span;
                let ok_bb = self.new_block();
                let fail_bb = self.new_block();
                self.terminate(Terminator::Branch { cond, then_bb: ok_bb, else_bb: fail_bb });
                self.switch_to(fail_bb);
                self.terminate(Terminator::Fault(FaultEdge { kind: FaultKind::Assert, span }));
                self.switch_to(ok_bb);
                Ok(self.unit())
            }
            ExprKind::Panic(_) => {
                self.terminate(Terminator::Fault(FaultEdge { kind: FaultKind::Panic, span: e.span }));
                Ok(self.unit())
            }
            ExprKind::Result => {
                let rl = self
                    .result_local
                    .ok_or_else(|| LowerError("`result` outside ensures".into()))?;
                Ok((Operand::Local(rl), self.ret_ty.clone()))
            }
            ExprKind::Try(inner) => self.lower_try(inner, e.span, None),
            ExprKind::Match { scrutinee, arms } => {
                self.lower_match(scrutinee, arms, None)?;
                Ok(self.unit())
            }
            ExprKind::Field { .. }
            | ExprKind::Index { .. }
            | ExprKind::Prefix { op: PrefixOp::Deref, .. } => {
                let (place, ty) = self.lower_place(e)?;
                if self.is_wordy(&ty) {
                    let id = self.emit_temp(ty.clone(), Rvalue::Load { place, ty: ty.clone() }, self.cur_span);
                    Ok((Operand::Local(id), ty))
                } else {
                    unsupported("aggregate in value position")
                }
            }
            ExprKind::Prefix { op: PrefixOp::Read, expr } => self.lower_borrow(expr, false),
            ExprKind::Prefix { op: PrefixOp::Write, expr } => self.lower_borrow(expr, true),
            // rawptr intrinsics that carry a bracketed type argument (design 0004).
            ExprKind::CastPtr { ty, arg } => {
                let (op, _) = self.lower_value(arg, None)?;
                let rty = Type::RawPtr(Box::new(self.resolve_ty(ty)?));
                let id = self.emit_temp(rty.clone(), Rvalue::Use(op), self.cur_span);
                Ok((Operand::Local(id), rty))
            }
            ExprKind::AddrToPtr { ty, arg } => {
                let (op, _) = self.lower_value(arg, Some(&Type::usize()))?;
                let rty = Type::RawPtr(Box::new(self.resolve_ty(ty)?));
                let id = self.emit_temp(rty.clone(), Rvalue::Use(op), self.cur_span);
                Ok((Operand::Local(id), rty))
            }
            ExprKind::PtrNull { ty } => {
                let rty = Type::RawPtr(Box::new(self.resolve_ty(ty)?));
                let id = self.emit_temp(rty.clone(), Rvalue::Use(Operand::Const(0, ScalarTy::U64)), self.cur_span);
                Ok((Operand::Local(id), rty))
            }
            ExprKind::Offsetof { ty, field } => {
                let n = match self.resolve_ty(ty)? {
                    Type::Named(name) => self.lay().field_offset(&name, field).map(|(_, o)| o).unwrap_or(0),
                    _ => 0,
                };
                Ok((Operand::Const(n as i128, ScalarTy::Usize), Type::usize()))
            }
            ExprKind::Sizeof(ty) => {
                let n = self.size_of(&self.resolve_ty(ty)?);
                Ok((Operand::Const(n as i128, ScalarTy::Usize), Type::usize()))
            }
            ExprKind::Alignof(ty) => {
                let n = self.align_of(&self.resolve_ty(ty)?);
                Ok((Operand::Const(n as i128, ScalarTy::Usize), Type::usize()))
            }
            ExprKind::FieldPtr { ptr, field } => {
                let (base, pty) = self.lower_value(ptr, None)?;
                let (fty, off) = match &pty {
                    Type::RawPtr(inner) => match &**inner {
                        Type::Named(n) => self.lay().field_offset(n, field).unwrap_or((Type::Error, 0)),
                        _ => (Type::Error, 0),
                    },
                    _ => (Type::Error, 0),
                };
                let rty = Type::RawPtr(Box::new(fty));
                let id = self.emit_temp(
                    rty.clone(),
                    Rvalue::PtrArith { base, index: Operand::Const(off as i128, ScalarTy::Usize), stride: 1 },
                    self.cur_span,
                );
                Ok((Operand::Local(id), rty))
            }
            other => unsupported(format!("expr {}", variant_name(other))),
        }
    }

    // ---- places & aggregate stores (Stage A2 memory substrate) ----

    /// Lower a place expression to a MIR [`Place`] and its type. Auto-peels
    /// borrow/box layers exactly as the oracle's `peel_place`.
    fn lower_place(&mut self, e: &Expr) -> LR<(Place, Type)> {
        self.cur_span = e.span;
        match &e.kind {
            ExprKind::Paren(i) => self.lower_place(i),
            ExprKind::Ident(name) => {
                if let Some((id, ty)) = self.lookup(name) {
                    Ok((Place::local(id), ty))
                } else if let Some((sty, _)) = self.items.statics.get(name) {
                    // A `static` place: reach it through its address (design 0008),
                    // modeled as `*(&STATIC)` so field/index/addr_of compose.
                    let sty = sty.clone();
                    let root = self.emit_temp(
                        Type::RawPtr(Box::new(sty.clone())),
                        Rvalue::StaticAddr(name.clone()),
                        self.cur_span,
                    );
                    Ok((Place { root, proj: vec![Proj::Deref { inner: sty.clone() }] }, sty))
                } else {
                    unsupported(format!("place of unknown `{name}`"))
                }
            }
            ExprKind::Prefix { op: PrefixOp::Deref, expr } => {
                // Deref of a value-producing expression (e.g. `vec.get(i).*` /
                // `map.get(k).*`, whose intrinsic yields a borrow): lower it to the
                // pointer operand and deref that temp.
                if let ExprKind::Call { .. } = &expr.kind {
                    let (op, t) = self.lower_value(expr, None)?;
                    let inner = match &t {
                        Type::Borrow(x) | Type::BorrowMut(x) | Type::Box(x) | Type::RawPtr(x) => (**x).clone(),
                        _ => return unsupported("deref of non-pointer call result"),
                    };
                    return Ok((self.deref_place(op, inner.clone()), inner));
                }
                let (mut pl, t) = self.lower_place(expr)?;
                let inner = match &t {
                    Type::Borrow(x) | Type::BorrowMut(x) | Type::Box(x) | Type::RawPtr(x) => (**x).clone(),
                    _ => return unsupported("deref of non-pointer"),
                };
                pl.proj.push(Proj::Deref { inner: inner.clone() });
                Ok((pl, inner))
            }
            ExprKind::Field { base, field, .. } => {
                let (mut pl, t) = self.lower_place(base)?;
                let t = self.peel(&mut pl, t);
                match &t {
                    Type::Named(n) => {
                        let (fty, off) = self
                            .lay()
                            .field_offset(n, field)
                            .ok_or_else(|| LowerError(format!("no field `{field}` on `{n}`")))?;
                        pl.proj.push(Proj::Field { offset: off, ty: fty.clone() });
                        Ok((pl, fty))
                    }
                    _ => unsupported("field of non-struct"),
                }
            }
            ExprKind::Index { base, index } => {
                let (idx, _) = self.lower_value(index, Some(&Type::usize()))?;
                let (mut pl, t) = self.lower_place(base)?;
                let t = self.peel(&mut pl, t);
                match &t {
                    Type::Array(elem, len) => {
                        let n = self.lay().array_len(len);
                        let stride = self.stride_of(elem);
                        pl.proj.push(Proj::Index { index: idx, stride, len: n, span: self.cur_span, slice: false });
                        Ok((pl, (**elem).clone()))
                    }
                    Type::Slice(elem) | Type::SliceMut(elem) => {
                        let stride = self.stride_of(elem);
                        // Slice index: the runtime length lives in the header (read
                        // by the interpreter); the bounds fault is INV-CHECK.
                        pl.proj.push(Proj::Index { index: idx, stride, len: 0, span: self.cur_span, slice: true });
                        Ok((pl, (**elem).clone()))
                    }
                    // `str[i]` yields the byte `u8` at `i` (design 0013 §3): a `str`
                    // is a `{ptr@0, len@8}` fat pointer with stride-1 `u8` elements,
                    // so it reuses the slice-index header read and its `Bounds` fault.
                    Type::Str => {
                        let u8t = Type::Scalar(ScalarTy::U8);
                        let stride = self.stride_of(&u8t);
                        pl.proj.push(Proj::Index { index: idx, stride, len: 0, span: self.cur_span, slice: true });
                        Ok((pl, u8t))
                    }
                    _ => unsupported("index of non-array"),
                }
            }
            _ => unsupported("unsupported place"),
        }
    }

    /// Auto-peel borrow/box layers on a place (the field/index base auto-deref).
    fn peel(&self, pl: &mut Place, mut ty: Type) -> Type {
        loop {
            match ty {
                Type::Borrow(x) | Type::BorrowMut(x) | Type::Box(x) => {
                    pl.proj.push(Proj::Deref { inner: (*x).clone() });
                    ty = *x;
                }
                other => return other,
            }
        }
    }

    fn lower_borrow(&mut self, expr: &Expr, excl: bool) -> LR<(Operand, Type)> {
        let (place, inner) = self.lower_place(expr)?;
        let ty = if excl {
            Type::BorrowMut(Box::new(inner))
        } else {
            Type::Borrow(Box::new(inner))
        };
        let id = self.emit_temp(ty.clone(), Rvalue::Ref(place), self.cur_span);
        Ok((Operand::Local(id), ty))
    }

    /// Get a place for an aggregate argument: a real place if `e` names one, else
    /// materialize the value into a fresh temp and return that temp's place.
    /// Whether `e` names a place (so `lower_place` handles it directly). Mirrors
    /// `materialize_place`'s place arms; a non-place expr must be materialized
    /// into a temp before it can be addressed.
    fn is_place_expr(e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Paren(i) => Self::is_place_expr(i),
            ExprKind::Ident(_)
            | ExprKind::Field { .. }
            | ExprKind::Index { .. }
            | ExprKind::Prefix { op: PrefixOp::Deref, .. } => true,
            _ => false,
        }
    }

    fn materialize_place(&mut self, e: &Expr, ty: &Type) -> LR<Place> {
        match &e.kind {
            ExprKind::Ident(_)
            | ExprKind::Field { .. }
            | ExprKind::Index { .. }
            | ExprKind::Paren(_)
            | ExprKind::Prefix { op: PrefixOp::Deref, .. } => Ok(self.lower_place(e)?.0),
            _ => {
                let tmp = self.new_local(ty.clone(), None);
                self.lower_into(e, &Place::local(tmp), ty)?;
                Ok(Place::local(tmp))
            }
        }
    }

    /// A match scrutinee is a place (matched in place) or a value expression such
    /// as a call — the latter is materialized into a temp first.
    fn lower_scrutinee(&mut self, e: &Expr) -> LR<(Place, Type)> {
        match &e.kind {
            ExprKind::Paren(i) => self.lower_scrutinee(i),
            ExprKind::Ident(_)
            | ExprKind::Field { .. }
            | ExprKind::Index { .. }
            | ExprKind::Prefix { op: PrefixOp::Deref, .. } => self.lower_place(e),
            ExprKind::Call { callee, args } => {
                let ret = match &callee.kind {
                    ExprKind::Ident(n) if self.items.fns.contains_key(n.as_str()) => {
                        self.items.fns[n.as_str()].ret.clone()
                    }
                    // A `box(alloc, v)` scrutinee: its `BoxResult[typeof v]`.
                    ExprKind::Ident(n) if n == "box" => {
                        let inner = self
                            .infer_ty(&args[1])
                            .ok_or_else(|| LowerError("cannot infer `box` value type".into()))?;
                        Type::BoxResult(Box::new(inner))
                    }
                    ExprKind::Field { base, field, iface } => {
                        let (fnname, _) = self
                            .resolve_method(base, field, iface.as_ref(), args, e.span)
                            .ok_or_else(|| LowerError("match on unresolved method call".into()))?;
                        self.items.fns[fnname.as_str()].ret.clone()
                    }
                    // A compiler-known builtin returning an enum (`str_from -> Utf8Res`,
                    // `pop -> Opt`): its result type comes from the static-ret table.
                    ExprKind::Ident(n) => match self.builtin_static_ret(n, args) {
                        Some(ty) => ty,
                        None => return unsupported("match on an indirect call"),
                    },
                    _ => return unsupported("match on an indirect call"),
                };
                let place = self.materialize_place(e, &ret)?;
                Ok((place, ret))
            }
            ExprKind::EnumCtor { .. } => {
                // A freshly-constructed enum: materialize into a temp of its type.
                // Its type is inferred from the ctor's expected type — unavailable
                // here — so keep it out of subset (rare in practice).
                unsupported("match on a freshly-constructed enum value")
            }
            _ => unsupported("unsupported match scrutinee"),
            }
    }

    /// Best-effort static type of a value expression, for sizing a materialized
    /// scrutinee / box inner (the checker already typed it; this recovers enough).
    fn infer_ty(&self, e: &Expr) -> Option<Type> {
        match &e.kind {
            ExprKind::Paren(i) | ExprKind::OutArg(i) => self.infer_ty(i),
            ExprKind::IntLit { suffix, .. } | ExprKind::NegIntLit { suffix, .. } => {
                Some(Type::Scalar(suffix.unwrap_or(ScalarTy::I64)))
            }
            ExprKind::BoolLit(_) => Some(Type::bool()),
            ExprKind::Ident(n) => self
                .lookup(n)
                .map(|(_, t)| t)
                .or_else(|| self.items.statics.get(n).map(|(t, _)| t.clone())),
            ExprKind::Field { .. } | ExprKind::Index { .. } | ExprKind::Prefix { op: PrefixOp::Deref, .. } => {
                self.static_ty(e)
            }
            ExprKind::StructLit { name, .. } => Some(Type::Named(name.clone())),
            ExprKind::EnumCtor { enum_name, .. } => Some(Type::Named(enum_name.clone())),
            ExprKind::ArrayRepeat { value, size } => {
                let elem = self.infer_ty(value)?;
                let n = match &size.kind {
                    ExprKind::IntLit { value, .. } => ArrayLen::Lit(*value),
                    ExprKind::Ident(nm) => ArrayLen::Named(nm.clone()),
                    _ => return None,
                };
                Some(Type::Array(Box::new(elem), n))
            }
            ExprKind::ArrayLit(els) => {
                let elem = self.infer_ty(els.first()?)?;
                Some(Type::Array(Box::new(elem), ArrayLen::Lit(els.len() as u64)))
            }
            ExprKind::Conv { ty, .. } => self.resolve_ty(ty).ok(),
            ExprKind::Bitcast { ty, .. } => self.resolve_ty(ty).ok(),
            ExprKind::Call { callee, args } => match &callee.kind {
                ExprKind::Ident(n) if self.items.fns.contains_key(n.as_str()) => {
                    Some(self.items.fns[n.as_str()].ret.clone())
                }
                ExprKind::Field { base, field, iface } => {
                    let (fnname, _) = self.resolve_method(base, field, iface.as_ref(), args, e.span)?;
                    Some(self.items.fns.get(&fnname)?.ret.clone())
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// Lower a `match` to switch-on-tag + payload binds (design 0010 §8.2.1). The
    /// scrutinee is read as a place; the tag selects an arm; copy-bindings copy
    /// their payload out. Scope is limited to drop-inert enums with copy payload
    /// binds (no scrutinee move/drop to reconcile) — anything else is out of
    /// subset (never a silent divergence).
    fn lower_match(
        &mut self,
        scrutinee: &Expr,
        arms: &[MatchArm],
        dst: Option<(&Place, &Type)>,
    ) -> LR<()> {
        let (mut splace, sty) = self.lower_scrutinee(scrutinee)?;
        // An integer scrutinee takes the literal-pattern path: a compare-and-branch
        // chain over existing MIR ops (design 0001 §8.2, extended). No enum tag.
        if let Type::Scalar(s) = &sty {
            if s.is_integer() {
                let s = *s;
                return self.lower_int_match(&splace, s, arms, dst);
            }
        }
        // The borrow mode of the scrutinee decides how non-copy payloads bind
        // (design 0001 §4.1, mirroring the oracle's `Hold`): an owned scrutinee
        // *moves* its payload out (and is pruned from its own drop); a borrowed
        // scrutinee binds a *borrow* of the payload — a copy payload always copies.
        let hold = match &sty {
            Type::Borrow(_) => Hold::Shared,
            Type::BorrowMut(_) => Hold::Excl,
            _ => Hold::Owned,
        };
        let scrut_path = if hold == Hold::Owned { self.ast_place_path(scrutinee) } else { None };
        let ety = self.peel(&mut splace, sty);
        let einfo = self
            .lay()
            .enum_info(&ety)
            .ok_or_else(|| LowerError("match on non-enum".into()))?;
        // Read the tag once.
        let mut tagpl = splace.clone();
        tagpl.proj.push(Proj::Field { offset: 0, ty: Type::Scalar(ScalarTy::U64) });
        let tag = self.emit_temp(
            Type::Scalar(ScalarTy::U64),
            Rvalue::Load { place: tagpl, ty: Type::Scalar(ScalarTy::U64) },
            self.cur_span,
        );
        let join = self.new_block();
        let mut test_open = true;
        for arm in arms {
            if !test_open {
                break;
            }
            match &arm.pattern.kind {
                PatKind::Wildcard | PatKind::Binding(_) => {
                    if arm.guard.is_some() {
                        // A guarded catch-all is NOT a terminal default: on a false
                        // guard control falls through to a later arm, so the match
                        // stays open (an unguarded catch-all follows — required by
                        // exhaustiveness).
                        let next_bb = self.new_block();
                        self.lower_match_arm(arm, &splace, &einfo, None, dst, hold, scrut_path.as_ref(), Some(next_bb))?;
                        if self.reachable {
                            self.terminate(Terminator::Goto(join));
                        }
                        self.switch_to(next_bb);
                    } else {
                        // Default arm (exhaustive tail): unconditional.
                        self.lower_match_arm(arm, &splace, &einfo, None, dst, hold, scrut_path.as_ref(), None)?;
                        if self.reachable {
                            self.terminate(Terminator::Goto(join));
                        }
                        test_open = false;
                    }
                }
                PatKind::IntLit { .. } => return unsupported("integer-literal pattern on enum scrutinee"),
                PatKind::IntRange { .. } => return unsupported("integer-range pattern on enum scrutinee"),
                PatKind::Variant { variant, .. } => {
                    let idx = einfo
                        .iter()
                        .position(|(n, _)| n == variant)
                        .ok_or_else(|| LowerError(format!("no variant `{variant}`")))?;
                    let cmp = self.emit_temp(
                        Type::bool(),
                        Rvalue::Cmp {
                            op: BinOp::Eq,
                            l: Operand::Local(tag),
                            r: Operand::Const(idx as i128, ScalarTy::U64),
                        },
                        self.cur_span,
                    );
                    let arm_bb = self.new_block();
                    let next_bb = self.new_block();
                    self.terminate(Terminator::Branch { cond: Operand::Local(cmp), then_bb: arm_bb, else_bb: next_bb });
                    self.switch_to(arm_bb);
                    self.lower_match_arm(arm, &splace, &einfo, Some(idx), dst, hold, scrut_path.as_ref(), Some(next_bb))?;
                    if self.reachable {
                        self.terminate(Terminator::Goto(join));
                    }
                    self.switch_to(next_bb);
                }
            }
        }
        if test_open {
            // Non-exhaustive fall-through — unreachable for a checked program, but
            // mirror the oracle's "no matching arm" panic if it is ever reached.
            self.terminate(Terminator::Fault(FaultEdge { kind: FaultKind::Panic, span: self.cur_span }));
        }
        self.switch_to(join);
        Ok(())
    }

    /// Lower an integer-scrutinee `match` to a compare-and-branch chain (design
    /// 0001 §8.2, extended). The scrutinee is loaded once; each literal arm tests
    /// `scrutinee == literal`; the `_`/binding arm is the fall-through. This uses
    /// only existing MIR ops (integer `Cmp` + conditional `Branch`), so the native
    /// backends need no change.
    fn lower_int_match(
        &mut self,
        splace: &Place,
        sty: ScalarTy,
        arms: &[MatchArm],
        dst: Option<(&Place, &Type)>,
    ) -> LR<()> {
        let scalar = Type::Scalar(sty);
        let val = self.emit_temp(
            scalar.clone(),
            Rvalue::Load { place: splace.clone(), ty: scalar.clone() },
            self.cur_span,
        );
        let join = self.new_block();
        let mut test_open = true;
        for arm in arms {
            if !test_open {
                break;
            }
            match &arm.pattern.kind {
                PatKind::IntLit { value, negative, .. } => {
                    let konst = crate::ast::int_pat_value(*value, *negative);
                    let cmp = self.emit_temp(
                        Type::bool(),
                        Rvalue::Cmp {
                            op: BinOp::Eq,
                            l: Operand::Local(val),
                            r: Operand::Const(konst, sty),
                        },
                        self.cur_span,
                    );
                    let arm_bb = self.new_block();
                    let next_bb = self.new_block();
                    self.terminate(Terminator::Branch { cond: Operand::Local(cmp), then_bb: arm_bb, else_bb: next_bb });
                    self.switch_to(arm_bb);
                    self.lower_int_arm(arm, None, val, &scalar, dst, Some(next_bb))?;
                    if self.reachable {
                        self.terminate(Terminator::Goto(join));
                    }
                    self.switch_to(next_bb);
                }
                PatKind::IntRange { lo_value, lo_negative, hi_value, hi_negative, inclusive, .. } => {
                    let lo = crate::ast::int_pat_value(*lo_value, *lo_negative);
                    let hi = crate::ast::int_pat_value(*hi_value, *hi_negative);
                    let ge_lo = self.emit_temp(
                        Type::bool(),
                        Rvalue::Cmp {
                            op: BinOp::Ge,
                            l: Operand::Local(val),
                            r: Operand::Const(lo, sty),
                        },
                        self.cur_span,
                    );
                    let hi_bb = self.new_block();
                    let arm_bb = self.new_block();
                    let next_bb = self.new_block();
                    self.terminate(Terminator::Branch { cond: Operand::Local(ge_lo), then_bb: hi_bb, else_bb: next_bb });
                    self.switch_to(hi_bb);
                    let le_hi = self.emit_temp(
                        Type::bool(),
                        Rvalue::Cmp {
                            op: if *inclusive { BinOp::Le } else { BinOp::Lt },
                            l: Operand::Local(val),
                            r: Operand::Const(hi, sty),
                        },
                        self.cur_span,
                    );
                    self.terminate(Terminator::Branch { cond: Operand::Local(le_hi), then_bb: arm_bb, else_bb: next_bb });
                    self.switch_to(arm_bb);
                    self.lower_int_arm(arm, None, val, &scalar, dst, Some(next_bb))?;
                    if self.reachable {
                        self.terminate(Terminator::Goto(join));
                    }
                    self.switch_to(next_bb);
                }
                PatKind::Wildcard => {
                    if arm.guard.is_some() {
                        let next_bb = self.new_block();
                        self.lower_int_arm(arm, None, val, &scalar, dst, Some(next_bb))?;
                        if self.reachable {
                            self.terminate(Terminator::Goto(join));
                        }
                        self.switch_to(next_bb);
                    } else {
                        self.lower_int_arm(arm, None, val, &scalar, dst, None)?;
                        if self.reachable {
                            self.terminate(Terminator::Goto(join));
                        }
                        test_open = false;
                    }
                }
                PatKind::Binding(name) => {
                    if arm.guard.is_some() {
                        let next_bb = self.new_block();
                        self.lower_int_arm(arm, Some(name.clone()), val, &scalar, dst, Some(next_bb))?;
                        if self.reachable {
                            self.terminate(Terminator::Goto(join));
                        }
                        self.switch_to(next_bb);
                    } else {
                        self.lower_int_arm(arm, Some(name.clone()), val, &scalar, dst, None)?;
                        if self.reachable {
                            self.terminate(Terminator::Goto(join));
                        }
                        test_open = false;
                    }
                }
                PatKind::Variant { .. } => return unsupported("variant pattern on integer scrutinee"),
            }
        }
        if test_open {
            // Non-exhaustive fall-through — unreachable for a checked program (the
            // checker requires a catch-all), but mirror the "no matching arm" panic.
            self.terminate(Terminator::Fault(FaultEdge { kind: FaultKind::Panic, span: self.cur_span }));
        }
        self.switch_to(join);
        Ok(())
    }

    /// Lower one arm of an integer `match`. A binding arm binds the whole (Copy)
    /// integer value into a fresh local from the already-loaded scrutinee temp.
    fn lower_int_arm(
        &mut self,
        arm: &MatchArm,
        bind_name: Option<String>,
        val: LocalId,
        scalar: &Type,
        dst: Option<(&Place, &Type)>,
        next_bb: Option<BlockId>,
    ) -> LR<()> {
        self.push_scope();
        if let Some(name) = bind_name {
            let local = self.new_local(scalar.clone(), Some(name.clone()));
            self.emit(
                StatementKind::Store(Place::local(local), Rvalue::Load { place: Place::local(val), ty: scalar.clone() }),
                self.cur_span,
                false,
            );
            self.bind(&name, local, scalar.clone());
        }
        self.finish_arm(arm, dst, next_bb)
    }

    /// Lower a match arm's optional guard, its body, and pop the arm scope. Called
    /// with the arm's bindings already in scope. When the arm is guarded, the
    /// guard lowers to a conditional `Branch` (reusing existing MIR ops, no backend
    /// change): guard-true runs the body; guard-false drops the arm's bindings and
    /// falls through to `next_bb`, the next arm's test (design 0001 §8.2, extended).
    fn finish_arm(
        &mut self,
        arm: &MatchArm,
        dst: Option<(&Place, &Type)>,
        next_bb: Option<BlockId>,
    ) -> LR<()> {
        if let Some(guard) = &arm.guard {
            let next = next_bb.expect("a guarded arm must have a fall-through target");
            let (cond, _) = self.lower_value(guard, Some(&Type::bool()))?;
            let body_bb = self.new_block();
            let gfalse_bb = self.new_block();
            self.terminate(Terminator::Branch { cond, then_bb: body_bb, else_bb: gfalse_bb });
            // Guard-false edge: the arm's bindings die here; fall through to test
            // the next arm against the same scrutinee.
            self.switch_to(gfalse_bb);
            self.emit_scope_drops_no_pop();
            self.terminate(Terminator::Goto(next));
            self.switch_to(body_bb);
        }
        match dst {
            Some((d, ty)) => self.lower_into(&arm.body, d, ty)?,
            None => {
                self.lower_value(&arm.body, None)?;
            }
        }
        self.pop_scope_with_drops();
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_match_arm(
        &mut self,
        arm: &MatchArm,
        splace: &Place,
        einfo: &[(String, Vec<Type>)],
        idx: Option<usize>,
        dst: Option<(&Place, &Type)>,
        hold: Hold,
        scrut_path: Option<&(LocalId, Vec<String>)>,
        next_bb: Option<BlockId>,
    ) -> LR<()> {
        self.push_scope();
        if let (PatKind::Variant { sub, .. }, Some(idx)) = (&arm.pattern.kind, idx) {
            let payloads = einfo[idx].1.clone();
            for (i, sp) in sub.iter().enumerate() {
                let name = match &sp.kind {
                    PatKind::Wildcard => continue,
                    PatKind::Binding(n) => n.clone(),
                    PatKind::Variant { .. } => return unsupported("nested match patterns"),
                    PatKind::IntLit { .. } => return unsupported("integer-literal sub-pattern"),
                    PatKind::IntRange { .. } => return unsupported("integer-range sub-pattern"),
                };
                let (pty, off) = self.lay().payload_offset(&payloads, i);
                let mut src = splace.clone();
                src.proj.push(Proj::Field { offset: off, ty: pty.clone() });
                let copy = is_copy(&pty, self.items);
                if copy {
                    // Copy-bind: copy the payload into a fresh owned local.
                    let local = self.new_local(pty.clone(), Some(name.clone()));
                    if self.is_wordy(&pty) {
                        self.emit(
                            StatementKind::Store(Place::local(local), Rvalue::Load { place: src, ty: pty.clone() }),
                            self.cur_span,
                            false,
                        );
                    } else {
                        self.emit(
                            StatementKind::CopyVal { dst: Place::local(local), src, ty: pty.clone() },
                            self.cur_span,
                            false,
                        );
                    }
                    self.bind(&name, local, pty);
                } else if hold == Hold::Owned {
                    // Move-bind: the payload moves into an owned local (a byte-move);
                    // mark the scrutinee's `_i` path moved so it is not double-dropped.
                    let local = self.new_local(pty.clone(), Some(name.clone()));
                    self.emit(
                        StatementKind::CopyVal { dst: Place::local(local), src, ty: pty.clone() },
                        self.cur_span,
                        false,
                    );
                    if let Some((root, base)) = scrut_path {
                        let mut path = base.clone();
                        path.push(format!("_{i}"));
                        self.moves.entry(*root).or_default().push(path);
                    }
                    self.bind(&name, local, pty);
                } else {
                    // Borrow-bind: the payload binds as a borrow of the scrutinee's
                    // sub-place (a shared/exclusive loan; never moved or dropped).
                    let bty = if hold == Hold::Excl {
                        Type::BorrowMut(Box::new(pty.clone()))
                    } else {
                        Type::Borrow(Box::new(pty.clone()))
                    };
                    let local = self.new_local(bty.clone(), Some(name.clone()));
                    self.emit(StatementKind::Assign(local, Rvalue::Ref(src)), self.cur_span, false);
                    self.bind(&name, local, bty);
                }
            }
        }
        self.finish_arm(arm, dst, next_bb)
    }

    /// Lower `inner?` (spec 02 §6.5): on the `ok` variant, unwrap the payload — a
    /// word payload becomes the `?` value, an aggregate payload (e.g. an owned
    /// `String`) is moved into `dst`; otherwise early-return the whole value with
    /// the enclosing scopes' drop schedule (INV-DROP), same-type by a whole-value
    /// copy or cross-type via the matching `From` conversion (design 0007 §7.1).
    fn lower_try(&mut self, inner: &Expr, span: Span, dst: Option<&Place>) -> LR<(Operand, Type)> {
        self.cur_span = span;
        let ety = match &inner.kind {
            ExprKind::Call { callee, .. } => match &callee.kind {
                ExprKind::Ident(n) => self
                    .items
                    .fns
                    .get(n.as_str())
                    .map(|s| s.ret.clone())
                    .ok_or_else(|| LowerError(format!("`?` on unknown call `{n}`")))?,
                _ => return unsupported("`?` on an indirect call"),
            },
            _ => return unsupported("`?` on a non-call operand"),
        };
        let ename = match &ety {
            Type::Named(n) => n.clone(),
            _ => return unsupported("`?` on a non-nominal enum"),
        };
        let ok = self
            .items
            .enums
            .get(&ename)
            .and_then(|e| e.ok_variant.clone())
            .ok_or_else(|| LowerError("`?` on a non-result enum".into()))?;
        let einfo = self
            .lay()
            .enum_info(&ety)
            .ok_or_else(|| LowerError("`?` on a non-enum".into()))?;
        let ok_idx = einfo
            .iter()
            .position(|(n, _)| n == &ok)
            .ok_or_else(|| LowerError("`?`: missing ok variant".into()))?;
        // Materialize the operand into an enum temp.
        let tmp = self.new_local(ety.clone(), None);
        self.lower_into(inner, &Place::local(tmp), &ety)?;
        let splace = Place::local(tmp);
        let mut tagpl = splace.clone();
        tagpl.proj.push(Proj::Field { offset: 0, ty: Type::Scalar(ScalarTy::U64) });
        let tag = self.emit_temp(
            Type::Scalar(ScalarTy::U64),
            Rvalue::Load { place: tagpl, ty: Type::Scalar(ScalarTy::U64) },
            span,
        );
        let cmp = self.emit_temp(
            Type::bool(),
            Rvalue::Cmp { op: BinOp::Eq, l: Operand::Local(tag), r: Operand::Const(ok_idx as i128, ScalarTy::U64) },
            span,
        );
        let ok_bb = self.new_block();
        let err_bb = self.new_block();
        let join = self.new_block();
        self.terminate(Terminator::Branch { cond: Operand::Local(cmp), then_bb: ok_bb, else_bb: err_bb });
        // Error path: same-type moves the whole value into the return place;
        // cross-type runs the matching `From` conversion first (design 0007 §7.1).
        self.switch_to(err_bb);
        if ety == self.ret_ty {
            self.emit(
                StatementKind::CopyVal { dst: Place::local(0), src: splace.clone(), ty: ety.clone() },
                span,
                false,
            );
        } else {
            self.lower_try_from(&splace, &ety, &einfo, &ok, span)?;
        }
        self.emit_return_drops();
        self.terminate(Terminator::Return);
        // Ok path: unwrap payload 0 (word-sized) as the `?` value.
        self.switch_to(ok_bb);
        let payloads = einfo[ok_idx].1.clone();
        if payloads.is_empty() {
            self.terminate(Terminator::Goto(join));
            self.switch_to(join);
            return Ok(self.unit());
        }
        let (pty, off) = self.lay().payload_offset(&payloads, 0);
        let mut ppl = splace.clone();
        ppl.proj.push(Proj::Field { offset: off, ty: pty.clone() });
        if self.is_wordy(&pty) {
            let val = self.emit_temp(pty.clone(), Rvalue::Load { place: ppl, ty: pty.clone() }, span);
            self.terminate(Terminator::Goto(join));
            self.switch_to(join);
            return Ok((Operand::Local(val), pty));
        }
        // Aggregate ok-payload (e.g. an owned `String`): move the whole payload
        // out of the enum temp into the caller's destination — mirrors the
        // tree-walker, which returns the payload's address unchanged (the enum
        // temp is never dropped, so this is a move, not a copy).
        let dst = dst.ok_or_else(|| LowerError("`?` aggregate payload without a destination".into()))?;
        self.emit(StatementKind::CopyVal { dst: dst.clone(), src: ppl, ty: pty.clone() }, span, false);
        self.terminate(Terminator::Goto(join));
        self.switch_to(join);
        Ok((self.unit().0, pty))
    }

    /// Evaluate `e` and write it into the destination place `dst` of type `ty`
    /// (the move/copy substrate: struct/array construction, scalar/pointer stores,
    /// and whole-aggregate copies via [`StatementKind::CopyVal`]).
    fn lower_into(&mut self, e: &Expr, dst: &Place, ty: &Type) -> LR<()> {
        self.cur_span = e.span;
        match &e.kind {
            ExprKind::Paren(i) => self.lower_into(i, dst, ty),
            ExprKind::StructLit { name, fields } => {
                let (flayout, _, _) = self.lay().struct_layout(name);
                for (fname, fty, off) in flayout {
                    if let Some(fi) = find_field(fields, &fname) {
                        let mut sub = dst.clone();
                        sub.proj.push(Proj::Field { offset: off, ty: fty.clone() });
                        self.lower_into(&fi.value, &sub, &fty)?;
                    }
                }
                Ok(())
            }
            ExprKind::ArrayLit(elems) => {
                let elem = match ty {
                    Type::Array(x, _) => (**x).clone(),
                    _ => return unsupported("array literal of non-array type"),
                };
                let stride = self.stride_of(&elem);
                for (i, el) in elems.iter().enumerate() {
                    let mut sub = dst.clone();
                    sub.proj.push(Proj::Index {
                        index: Operand::Const(i as i128, ScalarTy::Usize),
                        stride,
                        len: elems.len() as u64,
                        span: el.span,
                        slice: false,
                    });
                    self.lower_into(el, &sub, &elem)?;
                }
                Ok(())
            }
            ExprKind::ArrayRepeat { value, size } => {
                let elem = match ty {
                    Type::Array(x, _) => (**x).clone(),
                    _ => return unsupported("array-repeat of non-array type"),
                };
                let n = match &size.kind {
                    ExprKind::IntLit { value, .. } => *value,
                    ExprKind::Ident(nm) => *self.consts.get(nm).unwrap_or(&0),
                    _ => return unsupported("non-constant array-repeat length"),
                };
                let stride = self.stride_of(&elem);
                // Evaluate the element once into a temp, then copy it into each slot
                // (mirrors the oracle: one `eval_value`, N byte-copies).
                let tmp = self.new_local(elem.clone(), None);
                self.lower_into(value, &Place::local(tmp), &elem)?;
                for i in 0..n {
                    let mut sub = dst.clone();
                    sub.proj.push(Proj::Index {
                        index: Operand::Const(i as i128, ScalarTy::Usize),
                        stride,
                        len: n,
                        span: self.cur_span,
                        slice: false,
                    });
                    self.emit(
                        StatementKind::CopyVal { dst: sub, src: Place::local(tmp), ty: elem.clone() },
                        self.cur_span,
                        false,
                    );
                }
                Ok(())
            }
            ExprKind::EnumCtor { variant, args, .. } => {
                let einfo = self
                    .lay()
                    .enum_info(ty)
                    .ok_or_else(|| LowerError("enum ctor of non-enum".into()))?;
                let idx = einfo
                    .iter()
                    .position(|(n, _)| n == variant)
                    .ok_or_else(|| LowerError(format!("no variant `{variant}`")))?;
                let payloads = einfo[idx].1.clone();
                let mut tagpl = dst.clone();
                tagpl.proj.push(Proj::Field { offset: 0, ty: Type::Scalar(ScalarTy::U64) });
                self.emit(
                    StatementKind::Store(tagpl, Rvalue::Use(Operand::Const(idx as i128, ScalarTy::U64))),
                    self.cur_span,
                    false,
                );
                for (i, arg) in args.iter().enumerate() {
                    let (pty, off) = self.lay().payload_offset(&payloads, i);
                    let mut sub = dst.clone();
                    sub.proj.push(Proj::Field { offset: off, ty: pty.clone() });
                    self.lower_into(arg, &sub, &pty)?;
                }
                Ok(())
            }
            ExprKind::Match { scrutinee, arms } => self.lower_match(scrutinee, arms, Some((dst, ty))),
            ExprKind::Try(inner) => {
                // `?` producing an aggregate value into `dst` (cross-type or
                // aggregate-payload `?`): the aggregate ok-payload is copied into
                // `dst` inside `lower_try`; a word payload is stored here.
                let (op, oty) = self.lower_try(inner, e.span, Some(dst))?;
                if self.is_wordy(&oty) {
                    self.emit(StatementKind::Store(dst.clone(), Rvalue::Use(op)), self.cur_span, false);
                }
                Ok(())
            }
            // A block match-arm/expression in aggregate position: lower its
            // statements. A value-producing arm is a bare expression (handled
            // above); a block arm diverges (`return`/`break`), so `dst` is left
            // unwritten on that path — matching the value-position handling.
            ExprKind::Block(b) => self.lower_block(b),
            // A string literal materializes as a `[u8]` slice header (design 0001 §4.2).
            ExprKind::StrLit(bytes) | ExprKind::BytesLit(bytes) => {
                self.lower_str_into(bytes, dst);
                Ok(())
            }
            _ if self.is_wordy(ty) => {
                let (op, _) = self.lower_value(e, Some(ty))?;
                self.emit(StatementKind::Store(dst.clone(), Rvalue::Use(op)), self.cur_span, false);
                Ok(())
            }
            // Aggregate-producing collection intrinsics: `vec_new`/`map_new`/
            // `string_new` (Vec/Map/String), `pop` (Opt), `as_str` (str).
            ExprKind::Call { callee, args }
                if matches!(&callee.kind, ExprKind::Ident(n) if self.is_collection_agg(n, args)) =>
            {
                let name = match &callee.kind { ExprKind::Ident(n) => n.clone(), _ => unreachable!() };
                self.lower_collection_agg(&name, args, dst)
            }
            // Aggregate-producing builtins (box/unbox/ptr_read/slice_of/subslice).
            ExprKind::Call { callee, args } if matches!(&callee.kind, ExprKind::Ident(n) if is_builtin(n)) => {
                let name = match &callee.kind { ExprKind::Ident(n) => n.clone(), _ => unreachable!() };
                self.lower_builtin_into(&name, args, dst, ty)
            }
            // Aggregate-by-value return from a direct / method / indirect call.
            ExprKind::Call { callee, args } => {
                let (fname, all): (String, Vec<Expr>) = match &callee.kind {
                    ExprKind::Ident(n) if self.items.fns.contains_key(n.as_str()) => (n.clone(), args.to_vec()),
                    ExprKind::Field { base, field, iface } => match self.resolve_method(base, field, iface.as_ref(), args, e.span) {
                        Some(m) => m,
                        None => return unsupported("aggregate method call"),
                    },
                    _ => return unsupported("indirect/unknown aggregate call"),
                };
                let sig = self.items.fns[fname.as_str()].clone();
                let ops = self.lower_call_args(&sig, &all)?;
                let addr = self.new_local(Type::RawPtr(Box::new(ty.clone())), None);
                self.emit(
                    StatementKind::Assign(addr, Rvalue::Call { func: fname, args: ops }),
                    self.cur_span,
                    false,
                );
                let src = Place { root: addr, proj: vec![Proj::Deref { inner: ty.clone() }] };
                self.emit(
                    StatementKind::CopyVal { dst: dst.clone(), src, ty: ty.clone() },
                    self.cur_span,
                    false,
                );
                Ok(())
            }
            // Whole-aggregate value coming from a place (`let q = p;`): byte-copy,
            // marking a non-copy source moved (INV-DROP static prune).
            ExprKind::Ident(_)
            | ExprKind::Field { .. }
            | ExprKind::Index { .. }
            | ExprKind::Prefix { op: PrefixOp::Deref, .. } => {
                let (src, _) = self.lower_place(e)?;
                if !is_copy(ty, self.items) {
                    self.mark_moved(e);
                }
                self.emit(
                    StatementKind::CopyVal { dst: dst.clone(), src, ty: ty.clone() },
                    self.cur_span,
                    false,
                );
                Ok(())
            }
            _ => unsupported(format!("aggregate init {}", variant_name(&e.kind))),
        }
    }

    fn unit(&self) -> (Operand, Type) {
        (Operand::Const(0, ScalarTy::Unit), Type::unit())
    }

    fn lower_unary(&mut self, op: UnOp, expr: &Expr, expected: Option<&Type>, _span: Span) -> LR<(Operand, Type)> {
        match op {
            UnOp::Not => {
                let (v, _) = self.lower_value(expr, Some(&Type::bool()))?;
                let id = self.emit_temp(Type::bool(), Rvalue::Un { op, regime: self.regime, ty: ScalarTy::Bool, v, fault: None }, self.cur_span);
                Ok((Operand::Local(id), Type::bool()))
            }
            UnOp::Neg | UnOp::BitNot => {
                let (v, vty) = self.lower_value(expr, expected)?;
                let sty = self.concretize(&vty);
                // Neg can overflow (e.g. negating INT_MIN) in the checked regime;
                // BitNot never faults.
                let fault = if op == UnOp::Neg && self.regime == Regime::Checked && sty.is_integer() {
                    Some(FaultEdge { kind: FaultKind::Overflow, span: self.cur_span })
                } else {
                    // f64 negate is IEEE and never faults (design 0016).
                    None
                };
                let id = self.emit_temp(Type::Scalar(sty), Rvalue::Un { op, regime: self.regime, ty: sty, v, fault }, self.cur_span);
                Ok((Operand::Local(id), Type::Scalar(sty)))
            }
        }
    }

    fn lower_binary(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr, expected: Option<&Type>, span: Span) -> LR<(Operand, Type)> {
        use BinOp::*;
        match op {
            And | Or => self.lower_short_circuit(op, lhs, rhs),
            // `str` equality is byte-wise (same length AND identical bytes), not
            // fat-pointer identity — the tree-walker oracle (`eval_binary` +
            // `str_byte_cmp`, design 0013 §3). Lower it into a length check and a
            // byte-compare loop over existing MIR ops so all backends inherit the
            // same semantics from one place.
            Eq | Ne if self.is_str_ty(lhs) => self.lower_str_eq(op, lhs, rhs, span),
            Eq | Ne | Lt | Le | Gt | Ge => {
                let (l, lty) = self.lower_value(lhs, None)?;
                let ot = self.concretize(&lty);
                let (r, _) = self.lower_value(rhs, Some(&Type::Scalar(ot)))?;
                let id = self.emit_temp(Type::bool(), Rvalue::Cmp { op, l, r }, self.cur_span);
                Ok((Operand::Local(id), Type::bool()))
            }
            Add | Sub | Mul | Div | Rem | BitAnd | BitOr | BitXor | Shl | Shr => {
                let opty = expected.filter(|t| t.is_integer()).cloned();
                let (l, lty) = self.lower_value(lhs, opty.as_ref())?;
                let sty = match &opty {
                    Some(Type::Scalar(s)) => *s,
                    _ => self.concretize(&lty),
                };
                let rexp = if matches!(op, Shl | Shr) { None } else { Some(Type::Scalar(sty)) };
                let (r, _) = self.lower_value(rhs, rexp.as_ref())?;
                // Mirror the oracle: arithmetic and shift ops reset cur_span to the
                // whole-op span before the fault check; bitwise ops do not.
                let fallible = matches!(op, Add | Sub | Mul | Div | Rem | Shl | Shr) && sty.is_integer();
                if fallible {
                    self.cur_span = span;
                }
                let fault = if fallible && self.regime == Regime::Checked {
                    Some(FaultEdge { kind: FaultKind::Overflow, span })
                } else {
                    None
                };
                let id = self.emit_temp(
                    Type::Scalar(sty),
                    Rvalue::Bin { op, regime: self.regime, ty: sty, l, r, span, fault },
                    span,
                );
                Ok((Operand::Local(id), Type::Scalar(sty)))
            }
        }
    }

    /// Whether `e` has type `str`, mirroring the oracle's `eval_ty_probe`: a
    /// string literal and the `str`-returning text builtins are `str` before any
    /// binding exists to look up; everything else falls back to `static_ty`.
    fn is_str_ty(&self, e: &Expr) -> bool {
        match &e.kind {
            ExprKind::StrLit(_) => true,
            ExprKind::Paren(i) => self.is_str_ty(i),
            _ => matches!(self.static_ty(e), Some(Type::Str)),
        }
    }

    /// Lower `str == str` / `str != str` into a byte-wise comparison matching the
    /// tree-walker oracle (`str_byte_cmp`): equal iff same byte length AND
    /// identical bytes. Both operands are materialized as `{ptr@0, len@8}` fat
    /// pointers (the existing `str` representation); the length word and per-byte
    /// `str[i]` reads reuse the slice-header index lowering, so the MIR interpreter
    /// and both native backends inherit the same behavior from this one place.
    fn lower_str_eq(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr, span: Span) -> LR<(Operand, Type)> {
        let u8t = Type::Scalar(ScalarTy::U8);
        let usizet = Type::usize();
        let lp = self.materialize_place(lhs, &Type::Str)?;
        let rp = self.materialize_place(rhs, &Type::Str)?;

        let result = self.new_local(Type::bool(), None);
        let i = self.new_local(usizet.clone(), None);
        self.emit(
            StatementKind::Assign(i, Rvalue::Use(Operand::Const(0, ScalarTy::Usize))),
            span,
            false,
        );

        let len_of = |b: &mut Self, base: &Place| {
            let mut lp = base.clone();
            lp.proj.push(Proj::Field { offset: 8, ty: usizet.clone() });
            b.emit_temp(usizet.clone(), Rvalue::Load { place: lp, ty: usizet.clone() }, span)
        };
        let len_l = len_of(self, &lp);
        let len_r = len_of(self, &rp);
        let len_eq = self.emit_temp(
            Type::bool(),
            Rvalue::Cmp { op: BinOp::Eq, l: Operand::Local(len_l), r: Operand::Local(len_r) },
            span,
        );

        let loop_bb = self.new_block();
        let body_bb = self.new_block();
        let incr_bb = self.new_block();
        let eq_bb = self.new_block();
        let neq_bb = self.new_block();
        let join = self.new_block();

        // Equal lengths -> scan bytes; unequal lengths -> not equal.
        self.terminate(Terminator::Branch { cond: Operand::Local(len_eq), then_bb: loop_bb, else_bb: neq_bb });

        // Loop header: all bytes consumed (i >= len) -> equal; else compare byte i.
        self.switch_to(loop_bb);
        let done = self.emit_temp(
            Type::bool(),
            Rvalue::Cmp { op: BinOp::Ge, l: Operand::Local(i), r: Operand::Local(len_l) },
            span,
        );
        self.terminate(Terminator::Branch { cond: Operand::Local(done), then_bb: eq_bb, else_bb: body_bb });

        // Body: `str[i]` reuses the slice-index header read; `i < len` here holds,
        // so the INV-CHECK bounds edge it carries is dead.
        self.switch_to(body_bb);
        let byte_at = |b: &mut Self, base: &Place| {
            let mut pl = base.clone();
            pl.proj.push(Proj::Index { index: Operand::Local(i), stride: 1, len: 0, span, slice: true });
            b.emit_temp(u8t.clone(), Rvalue::Load { place: pl, ty: u8t.clone() }, span)
        };
        let lbyte = byte_at(self, &lp);
        let rbyte = byte_at(self, &rp);
        let byte_eq = self.emit_temp(
            Type::bool(),
            Rvalue::Cmp { op: BinOp::Eq, l: Operand::Local(lbyte), r: Operand::Local(rbyte) },
            span,
        );
        self.terminate(Terminator::Branch { cond: Operand::Local(byte_eq), then_bb: incr_bb, else_bb: neq_bb });

        // Increment: `i < len <= u64::MAX`, so a wrapping add never wraps.
        self.switch_to(incr_bb);
        let next = self.emit_temp(
            usizet.clone(),
            Rvalue::Bin {
                op: BinOp::Add,
                regime: Regime::Wrapping,
                ty: ScalarTy::Usize,
                l: Operand::Local(i),
                r: Operand::Const(1, ScalarTy::Usize),
                span,
                fault: None,
            },
            span,
        );
        self.emit(StatementKind::Assign(i, Rvalue::Use(Operand::Local(next))), span, false);
        self.terminate(Terminator::Goto(loop_bb));

        // `==` is true on the equal edge; `!=` is true on the not-equal edge.
        self.switch_to(eq_bb);
        self.emit(
            StatementKind::Assign(result, Rvalue::Use(Operand::Const((op == BinOp::Eq) as i128, ScalarTy::Bool))),
            span,
            false,
        );
        self.terminate(Terminator::Goto(join));

        self.switch_to(neq_bb);
        self.emit(
            StatementKind::Assign(result, Rvalue::Use(Operand::Const((op == BinOp::Ne) as i128, ScalarTy::Bool))),
            span,
            false,
        );
        self.terminate(Terminator::Goto(join));

        self.switch_to(join);
        Ok((Operand::Local(result), Type::bool()))
    }

    fn lower_short_circuit(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr) -> LR<(Operand, Type)> {
        let result = self.new_local(Type::bool(), None);
        let (l, _) = self.lower_value(lhs, Some(&Type::bool()))?;
        let rhs_bb = self.new_block();
        let short_bb = self.new_block();
        let join = self.new_block();
        // `&&`: if lhs then eval rhs else result=false. `||`: if lhs then
        // result=true else eval rhs.
        let (then_bb, else_bb) = if op == BinOp::And { (rhs_bb, short_bb) } else { (short_bb, rhs_bb) };
        self.terminate(Terminator::Branch { cond: l, then_bb, else_bb });
        self.switch_to(rhs_bb);
        let (r, _) = self.lower_value(rhs, Some(&Type::bool()))?;
        self.emit(StatementKind::Assign(result, Rvalue::Use(r)), self.cur_span, false);
        self.terminate(Terminator::Goto(join));
        self.switch_to(short_bb);
        let short_val = op == BinOp::Or;
        self.emit(StatementKind::Assign(result, Rvalue::Use(Operand::Const(short_val as i128, ScalarTy::Bool))), self.cur_span, false);
        self.terminate(Terminator::Goto(join));
        self.switch_to(join);
        Ok((Operand::Local(result), Type::bool()))
    }

    fn lower_conv(&mut self, ty: &Ty, expr: &Expr) -> LR<(Operand, Type)> {
        let (v, vty) = self.lower_value(expr, None)?;
        let from = self.concretize(&vty);
        let to = match self.resolve_ty(ty)? {
            Type::Scalar(s) => s,
            _ => return unsupported("non-scalar conversion target"),
        };
        // An int<->f64 conversion is IEEE and regime-exempt — never faults
        // (design 0016 §5). An integer->integer conv delivers conv-loss at the
        // operand's trailing span (== cur_span) in the checked regime.
        // An int->f64 conv (`to == f64`) never faults. Every other checked conv to
        // an integer target carries a ConvLoss edge — for an f64 source (float->int)
        // it is inert (saturating; design 0016 §5), kept only for INV-CHECK uniformity.
        let _ = from;
        let fault = if self.regime == Regime::Checked && to.is_integer() {
            Some(FaultEdge { kind: FaultKind::ConvLoss, span: self.cur_span })
        } else {
            None
        };
        let id = self.emit_temp(Type::Scalar(to), Rvalue::Conv { to, regime: self.regime, v, fault }, self.cur_span);
        Ok((Operand::Local(id), Type::Scalar(to)))
    }

    /// Lower `bitcast T (e)` -- same-width bit reinterpretation (design 0016 section
    /// 10). Emits a `Rvalue::Bitcast` (no regime, NO fault edge: bitcast is total).
    /// A bare `{integer}` operand takes the float's same-width unsigned int so its
    /// full bit pattern survives (mirrors the tree-walker / checker).
    fn lower_bitcast(&mut self, ty: &Ty, expr: &Expr) -> LR<(Operand, Type)> {
        let to = match self.resolve_ty(ty)? {
            Type::Scalar(s) => s,
            _ => return unsupported("non-scalar bitcast target"),
        };
        let expected = if to.is_float() {
            Some(Type::Scalar(if to == ScalarTy::F64 { ScalarTy::U64 } else { ScalarTy::U32 }))
        } else {
            None
        };
        let (v, _vty) = self.lower_value(expr, expected.as_ref())?;
        let id = self.emit_temp(Type::Scalar(to), Rvalue::Bitcast { to, v }, self.cur_span);
        Ok((Operand::Local(id), Type::Scalar(to)))
    }

    /// Lower a call's argument list to operands (word args by value; aggregate
    /// args by address into a byte-copy — see the callee's param binding).
    fn lower_call_args(&mut self, sig: &crate::resolve::FnSig, args: &[Expr]) -> LR<Vec<Operand>> {
        if sig.params.len() != args.len() {
            return unsupported("arity mismatch");
        }
        let mut ops = Vec::new();
        for (p, a) in sig.params.iter().zip(args) {
            if !matches!(p.mode, ParamMode::Take | ParamMode::Read | ParamMode::Write) {
                return unsupported("out/unsupported argument mode");
            }
            let a: &Expr = match &a.kind {
                ExprKind::OutArg(inner) => inner,
                _ => a,
            };
            // A `slice`/`slice_mut` value crossing a spawn arrives as a `read`/`write`
            // reborrow of a slice place (the §2.1 borrow branch); for a slice a
            // reborrow is the same fat pointer, so peel the prefix and pass the value
            // (a borrow, not a move — no `mark_moved`).
            let (a, reborrow): (&Expr, bool) = match &a.kind {
                ExprKind::Prefix { op: PrefixOp::Read | PrefixOp::Write, expr }
                    if matches!(p.lowered, Type::Slice(_) | Type::SliceMut(_)) =>
                {
                    (expr.as_ref(), true)
                }
                _ => (a, false),
            };
            if self.is_wordy(&p.lowered) {
                let (op, _) = self.lower_value(a, Some(&p.lowered))?;
                ops.push(op);
            } else {
                let place = self.materialize_place(a, &p.lowered)?;
                // A by-value (`take`) non-copy aggregate arg moves the source; prune
                // its later drop (INV-DROP static mask).
                if !reborrow && p.mode == ParamMode::Take && !is_copy(&p.lowered, self.items) {
                    self.mark_moved(a);
                }
                let id = self.emit_temp(
                    Type::RawPtr(Box::new(p.lowered.clone())),
                    Rvalue::Ref(place),
                    self.cur_span,
                );
                ops.push(Operand::Local(id));
            }
        }
        Ok(ops)
    }

    /// Lower `split_mut(parent, mid, out lo, out hi)` (design 0012 §1.4) into
    /// ordinary slice-header ops: build the parent's `slice_mut` header, then two
    /// bounds-checked `subslice`s — `lo = [0, mid)` and `hi = [mid, len)` — into the
    /// caller's out-slots. `subslice` faults `Bounds` at `span` when `mid > len`, so
    /// the bounds identity `(bounds, call-span)` matches every engine. The
    /// stipulated disjointness is a compile-time fact; at run time this is plain
    /// sub-slice arithmetic, so the native backend inherits it with NO runtime hook.
    fn lower_split_mut(&mut self, args: &[Expr], span: Span) -> LR<(Operand, Type)> {
        if args.len() != 4 {
            return unsupported("split_mut arity");
        }
        // 1. Build the parent's slice header `whole : slice_mut elem`.
        let (apl, aty) = self.lower_place(&args[0])?;
        let elem = match &aty {
            Type::Array(e, _) | Type::Slice(e) | Type::SliceMut(e) => (**e).clone(),
            _ => return unsupported("split_mut parent is not a slice/array"),
        };
        let whole_ty = Type::SliceMut(Box::new(elem.clone()));
        let whole_id = self.new_local(whole_ty.clone(), None);
        let whole = Place::local(whole_id);
        match &aty {
            Type::Array(el, l) => {
                let n = self.lay().array_len(l);
                let addr = self.emit_temp(
                    Type::RawPtr(Box::new((**el).clone())),
                    Rvalue::Ref(apl),
                    span,
                );
                let mut f0 = whole.clone();
                f0.proj.push(Proj::Field { offset: 0, ty: Type::Scalar(ScalarTy::U64) });
                self.emit(StatementKind::Store(f0, Rvalue::Use(Operand::Local(addr))), span, false);
                let mut f8 = whole.clone();
                f8.proj.push(Proj::Field { offset: 8, ty: Type::Scalar(ScalarTy::U64) });
                self.emit(
                    StatementKind::Store(f8, Rvalue::Use(Operand::Const(n as i128, ScalarTy::U64))),
                    span,
                    false,
                );
            }
            Type::Slice(_) | Type::SliceMut(_) => {
                self.emit(
                    StatementKind::CopyVal { dst: whole.clone(), src: apl, ty: aty.clone() },
                    span,
                    false,
                );
            }
            _ => unreachable!(),
        }
        // 2. The length operand: read `whole.len` (works for both array and slice).
        let mut lenf = whole.clone();
        lenf.proj.push(Proj::Field { offset: 8, ty: Type::Scalar(ScalarTy::U64) });
        let len_id = self.emit_temp(
            Type::Scalar(ScalarTy::U64),
            Rvalue::Load { place: lenf, ty: Type::Scalar(ScalarTy::U64) },
            span,
        );
        let len_op = Operand::Local(len_id);
        // 3. mid, and the element stride shared by both halves.
        let (mid, _) = self.lower_value(&args[1], Some(&Type::usize()))?;
        let stride = self.stride_of(&elem);
        // 4. lo = subslice(whole, 0, mid) ; hi = subslice(whole, mid, len).
        let lo_inner = match &args[2].kind {
            ExprKind::OutArg(i) => i.as_ref(),
            _ => &args[2],
        };
        let (lo_place, _) = self.lower_place(lo_inner)?;
        self.emit(
            StatementKind::Subslice {
                dst: lo_place,
                src: whole.clone(),
                lo: Operand::Const(0, ScalarTy::U64),
                hi: mid,
                stride,
                span,
            },
            span,
            false,
        );
        let hi_inner = match &args[3].kind {
            ExprKind::OutArg(i) => i.as_ref(),
            _ => &args[3],
        };
        let (hi_place, _) = self.lower_place(hi_inner)?;
        self.emit(
            StatementKind::Subslice {
                dst: hi_place,
                src: whole,
                lo: mid,
                hi: len_op,
                stride,
                span,
            },
            span,
            false,
        );
        Ok(self.unit())
    }

    fn lower_call(&mut self, callee: &Expr, args: &[Expr], span: Span) -> LR<(Operand, Type)> {
        if let ExprKind::Ident(name) = &callee.kind {
            // Word/unit-producing builtin intrinsics (aggregate ones flow through
            // `lower_into`). `trace` is the observable (INV-OBS-ORDER).
            if name == "trace" {
                // Observe the arg at its own width so an `f64` traces its bit pattern
                // and a float arithmetic arg is computed with IEEE, not int, ops.
                let exp = match self.static_ty(&args[0]) {
                    Some(Type::Scalar(s)) if s.is_float() => s,
                    _ => ScalarTy::I64,
                };
                let (v, _) = self.lower_value(&args[0], Some(&Type::Scalar(exp)))?;
                self.emit(StatementKind::Trace(v), span, true);
                return Ok(self.unit());
            }
            if name == "split_mut" {
                return self.lower_split_mut(args, span);
            }
            if is_builtin(name) {
                return self.lower_builtin_value(name, args, span);
            }
            // Word/unit-producing collection intrinsics (Vec/Map/String), gated by
            // the receiver's static type exactly as the oracle's `arg0_is_*` guards;
            // a non-collection call of the same name falls through to a user fn.
            if let Some(res) = self.lower_collection_value(name, args, span)? {
                return Ok(res);
            }
            // A monomorphized method resolved to a free fn, or a direct user fn.
            if let Some(sig) = self.items.fns.get(name.as_str()).cloned() {
                return self.lower_direct_call(name.clone(), &sig, args, span);
            }
            // A foreign (`extern`) call (design 0011 §5): lower to a direct call on
            // the C symbol; the interpreter dispatches it through the shim registry.
            if let Some(es) = self.items.externs.get(name.as_str()) {
                let sig = es.to_fn_sig();
                return self.lower_direct_call(name.clone(), &sig, args, span);
            }
            // An indirect call through a fn-pointer local.
            if self.lookup(name).is_some() {
                return self.lower_indirect_call(callee, args, span);
            }
            return unsupported(format!("call to `{name}`"));
        }
        // A method call `recv.m(args)` -> the mono-resolved impl free fn.
        if let ExprKind::Field { base, field, iface } = &callee.kind {
            if let Some((fnname, all)) = self.resolve_method(base, field, iface.as_ref(), args, span) {
                let sig = self.items.fns[fnname.as_str()].clone();
                return self.lower_direct_call(fnname, &sig, &all, span);
            }
        }
        // Indirect call through any other fn-pointer-valued expression.
        self.lower_indirect_call(callee, args, span)
    }

    /// Lower a `spawn CALLEE(args)` (design 0012 §1.1). A direct call to a known
    /// user fn becomes a `Spawn` statement the native backend threads; anything
    /// else (fn-pointer / method spawn) falls back to an inline call, which the
    /// native backend then runs sequentially — an honestly-noted boundary, none of
    /// the Stage-2 gate fixtures hit it.
    fn lower_spawn(&mut self, call: &Expr, span: Span) -> LR<()> {
        if let ExprKind::Call { callee, args } = &call.kind {
            if let ExprKind::Ident(name) = &callee.kind {
                if let Some(sig) = self.items.fns.get(name.as_str()).cloned() {
                    let ops = self.lower_call_args(&sig, args)?;
                    self.emit(StatementKind::Spawn { func: name.clone(), args: ops }, span, false);
                    return Ok(());
                }
            }
        }
        self.lower_value(call, None)?;
        Ok(())
    }

    fn lower_direct_call(&mut self, name: String, sig: &crate::resolve::FnSig, args: &[Expr], span: Span) -> LR<(Operand, Type)> {
        if !self.is_wordy(&sig.ret) && !matches!(sig.ret, Type::Scalar(ScalarTy::Unit)) {
            return unsupported("aggregate return in value position");
        }
        let ops = self.lower_call_args(sig, args)?;
        let ret = sig.ret.clone();
        let id = self.emit_temp(ret.clone(), Rvalue::Call { func: name, args: ops }, span);
        Ok((Operand::Local(id), ret))
    }

    fn lower_indirect_call(&mut self, callee: &Expr, args: &[Expr], span: Span) -> LR<(Operand, Type)> {
        let (fop, fty) = self.lower_value(callee, None)?;
        let ret = match &fty {
            Type::FnPtr(fp) => (*fp.ret).clone(),
            _ => return unsupported("indirect call of non-fn-pointer"),
        };
        if !self.is_wordy(&ret) && !matches!(ret, Type::Scalar(ScalarTy::Unit)) {
            return unsupported("aggregate return through fn pointer");
        }
        let sig = self.fnptr_sig(&fty);
        let ops = self.lower_call_args(&sig, args)?;
        let id = self.emit_temp(ret.clone(), Rvalue::CallIndirect { func: fop, args: ops }, span);
        Ok((Operand::Local(id), ret))
    }

    fn lower_if(&mut self, cond: &Expr, then_blk: &Block, else_blk: Option<&Expr>) -> LR<()> {
        let (c, _) = self.lower_value(cond, Some(&Type::bool()))?;
        let then_bb = self.new_block();
        let else_bb = self.new_block();
        let join = self.new_block();
        self.terminate(Terminator::Branch { cond: c, then_bb, else_bb });
        self.switch_to(then_bb);
        self.lower_block(then_blk)?;
        if self.reachable {
            self.terminate(Terminator::Goto(join));
        }
        self.switch_to(else_bb);
        if let Some(e) = else_blk {
            self.lower_value(e, None)?;
        }
        if self.reachable {
            self.terminate(Terminator::Goto(join));
        }
        self.switch_to(join);
        Ok(())
    }

    fn lower_while(&mut self, cond: &Expr, body: &Block) -> LR<()> {
        let head = self.new_block();
        let body_bb = self.new_block();
        let exit = self.new_block();
        self.terminate(Terminator::Goto(head));
        self.switch_to(head);
        let (c, _) = self.lower_value(cond, Some(&Type::bool()))?;
        self.terminate(Terminator::Branch { cond: c, then_bb: body_bb, else_bb: exit });
        self.switch_to(body_bb);
        let scope_depth = self.scopes.len();
        self.loops.push(Loop { continue_bb: head, break_bb: exit, scope_depth });
        self.lower_block(body)?;
        self.loops.pop();
        if self.reachable {
            self.terminate(Terminator::Goto(head));
        }
        self.switch_to(exit);
        Ok(())
    }

    fn lower_loop(&mut self, body: &Block) -> LR<()> {
        let body_bb = self.new_block();
        let exit = self.new_block();
        self.terminate(Terminator::Goto(body_bb));
        self.switch_to(body_bb);
        let scope_depth = self.scopes.len();
        self.loops.push(Loop { continue_bb: body_bb, break_bb: exit, scope_depth });
        self.lower_block(body)?;
        self.loops.pop();
        if self.reachable {
            self.terminate(Terminator::Goto(body_bb));
        }
        self.switch_to(exit);
        Ok(())
    }

    fn lower_regime(&mut self, b: &Block, regime: Regime) -> LR<(Operand, Type)> {
        let save = self.regime;
        self.regime = regime;
        let r = self.lower_block(b);
        self.regime = save;
        r?;
        Ok(self.unit())
    }

    fn lower_return(&mut self, opt: Option<&Expr>) -> LR<()> {
        if let Some(e) = opt {
            let ret_ty = self.ret_ty.clone();
            self.lower_into(e, &Place::local(0), &ret_ty)?;
        }
        self.emit_return_drops();
        self.terminate(Terminator::Return);
        Ok(())
    }
}


impl<'a> Lowerer<'a> {
    // ---- static initializer lowering ----
    fn lower_static_init(&mut self, init_name: &str, ty: &Type, value: &Expr) -> LR<MirFn> {
        self.ret_ty = ty.clone();
        let _ret = self.new_local(ty.clone(), None); // _0
        self.push_scope();
        let entry = self.new_block();
        self.switch_to(entry);
        self.lower_into(value, &Place::local(0), ty)?;
        if self.reachable {
            self.terminate(Terminator::Return);
        }
        self.pop_scope_with_drops();
        Ok(MirFn {
            name: init_name.to_string(),
            num_params: 0,
            result_local: None,
            locals: std::mem::take(&mut self.locals),
            blocks: std::mem::take(&mut self.blocks),
            entry,
            requires: Vec::new(),
            ensures: Vec::new(),
            replay: ReplayPolicy::Precise,
        })
    }

    // ---- fn-pointer types / synthetic signatures ----
    fn fnptr_ty(&self, name: &str) -> Type {
        match self.items.fns.get(name) {
            Some(sig) => Type::FnPtr(crate::types::FnPtrTy {
                params: sig.params.iter().map(|p| (p.mode, p.decl_ty.clone())).collect(),
                alloc: sig.alloc,
                foreign: sig.foreign,
                ret: Box::new(sig.ret.clone()),
            }),
            None => Type::FnPtr(crate::types::FnPtrTy { params: Vec::new(), alloc: false, foreign: false, ret: Box::new(Type::unit()) }),
        }
    }
    fn fnptr_sig(&self, fty: &Type) -> crate::resolve::FnSig {
        let fp = match fty {
            Type::FnPtr(fp) => fp,
            _ => unreachable!("fnptr_sig on non-fn-pointer"),
        };
        let params = fp
            .params
            .iter()
            .enumerate()
            .map(|(i, (mode, ty))| crate::resolve::ParamInfo {
                name: format!("_a{i}"),
                mode: *mode,
                region: None,
                decl_ty: ty.clone(),
                lowered: crate::resolve::lower_param(*mode, ty.clone()),
                span: Span::point(0),
            })
            .collect();
        crate::resolve::FnSig {
            name: String::new(),
            regions: Vec::new(),
            params,
            alloc: fp.alloc,
            foreign: fp.foreign,
            ret: (*fp.ret).clone(),
            ret_region: None,
            ret_span: Span::point(0),
            span: Span::point(0),
        }
    }

    // ---- interface method dispatch (mono-resolved to a free fn) ----
    fn static_ty(&self, e: &Expr) -> Option<Type> {
        match &e.kind {
            ExprKind::Paren(i) | ExprKind::OutArg(i) => self.static_ty(i),
            ExprKind::Ident(name) => self.lookup(name).map(|(_, t)| t),
            ExprKind::Field { base, field, .. } => {
                let bt = self.static_ty(base)?;
                let n = strip_to_nominal(&bt)?;
                self.lay().field_offset(&n, field).map(|(t, _)| t)
            }
            ExprKind::Prefix { op: PrefixOp::Deref, expr } => match self.static_ty(expr)? {
                Type::Box(e) | Type::Borrow(e) | Type::BorrowMut(e) | Type::RawPtr(e) => Some(*e),
                _ => None,
            },
            // A builtin collection/text-op call's result type, so a chain on
            // `get(v,i).*` / `as_str(..)` resolves here just as it does for a
            // user call. Mirrors the interp static-typer and the checker's
            // `check_builtin` result types.
            ExprKind::Call { callee, args } => match &callee.kind {
                ExprKind::Ident(name) => self.builtin_static_ret(name, args),
                _ => None,
            },
            _ => None,
        }
    }

    /// The result type of a builtin collection/text op call, mirroring the
    /// checker's `check_builtin` types (src/check/expr.rs) and the interp
    /// static-typer (src/interp/eval.rs). Returns `None` for a name/arg-shape
    /// that is not a builtin here (so a same-named user fn still resolves) or
    /// whose result type is fixed by the annotation rather than the arguments
    /// (`vec_new`/`map_new`, never chained). The collection ops carry the same
    /// receiver guard (`collection_ty`) the lowering uses.
    fn builtin_static_ret(&self, name: &str, args: &[Expr]) -> Option<Type> {
        let recv = || args.first().and_then(|a| self.collection_ty(a));
        let ty = match name {
            "unbox" => match self.static_ty(args.first()?)? {
                Type::Box(inner) => *inner,
                _ => return None,
            },
            "ptr_read" => match self.static_ty(args.first()?)? {
                Type::RawPtr(inner) => *inner,
                _ => return None,
            },
            "ptr_write" => Type::unit(),
            "ptr_offset" => self.static_ty(args.first()?)?,
            "is_null" => Type::bool(),
            "ptr_to_addr" => Type::usize(),
            // `sqrt(x)` returns the argument's float type (design 0016 §11).
            "sqrt" => self.static_ty(args.first()?)?,
            "addr_of" | "addr_of_mut" => Type::RawPtr(Box::new(self.static_ty(args.first()?)?)),
            "subslice" => match self.static_ty(args.first()?)? {
                t @ (Type::Slice(_) | Type::SliceMut(_)) => t,
                _ => return None,
            },
            "as_bytes" => Type::Slice(Box::new(Type::Scalar(ScalarTy::U8))),
            "str_from" => Type::Named("Utf8Res".to_string()),
            "str_from_unchecked" | "substr" => Type::Str,
            "char_at" => Type::Named("CharStep".to_string()),
            "char_count" | "len" => Type::usize(),
            "string_new" => Type::Named("String".to_string()),
            "push" | "append" if matches!(recv(), Some(Type::Named(n)) if n == "String") => Type::unit(),
            "as_str" if matches!(recv(), Some(Type::Named(n)) if n == "String") => Type::Str,
            "push" | "set" if matches!(recv(), Some(Type::App(n, _)) if n == "Vec") => Type::unit(),
            "pop" if matches!(recv(), Some(Type::App(n, _)) if n == "Vec") => {
                Type::Named("Opt".to_string())
            }
            "get" if matches!(recv(), Some(Type::App(n, _)) if n == "Vec") => match recv()? {
                Type::App(_, targs) => Type::Borrow(Box::new(targs.into_iter().next().unwrap_or(Type::Error))),
                _ => return None,
            },
            "insert" if matches!(recv(), Some(Type::App(n, _)) if n == "Map") => Type::unit(),
            "contains" if matches!(recv(), Some(Type::App(n, _)) if n == "Map") => Type::bool(),
            "get" if matches!(recv(), Some(Type::App(n, _)) if n == "Map") => match recv()? {
                Type::App(_, targs) => Type::Borrow(Box::new(targs.into_iter().next().unwrap_or(Type::Error))),
                _ => return None,
            },
            "cancelled" => Type::bool(),
            "trace" => Type::unit(),
            _ => return None,
        };
        Some(ty)
    }
    fn static_nominal(&self, e: &Expr) -> Option<String> {
        dispatch_nominal(&self.static_ty(e)?)
    }
    fn resolve_method(&self, base: &Expr, field: &str, iface: Option<&String>, args: &[Expr], span: Span) -> Option<(String, Vec<Expr>)> {
        let nominal = self.static_nominal(base)?;
        let key_iface = match iface {
            Some(i) => i.clone(),
            None => self.impls.first_iface.get(&(nominal.clone(), field.to_string()))?.clone(),
        };
        let fnname = self.impls.dispatch.get(&(nominal, key_iface, field.to_string())).cloned()?;
        let self_mode = self.items.fns.get(&fnname).and_then(|s| s.params.first().map(|p| p.mode));
        let base_is_borrow = matches!(self.static_ty(base), Some(Type::Borrow(_)) | Some(Type::BorrowMut(_)));
        let recv = match self_mode {
            Some(ParamMode::Read) if !base_is_borrow => Expr {
                kind: ExprKind::Prefix { op: PrefixOp::Read, expr: Box::new(base.clone()) },
                span,
            },
            Some(ParamMode::Write) if !base_is_borrow => Expr {
                kind: ExprKind::Prefix { op: PrefixOp::Write, expr: Box::new(base.clone()) },
                span,
            },
            _ => base.clone(),
        };
        let mut all = Vec::with_capacity(args.len() + 1);
        all.push(recv);
        all.extend(args.iter().cloned());
        Some((fnname, all))
    }

    // ---- pointer / place helpers ----
    fn deref_place(&mut self, ptr: Operand, inner: Type) -> Place {
        let root = match ptr {
            Operand::Local(id) => id,
            other => self.emit_temp(Type::RawPtr(Box::new(inner.clone())), Rvalue::Use(other), self.cur_span),
        };
        Place { root, proj: vec![Proj::Deref { inner }] }
    }
    fn pointee(ty: &Type) -> LR<Type> {
        match ty {
            Type::RawPtr(x) | Type::Borrow(x) | Type::BorrowMut(x) | Type::Box(x) => Ok((**x).clone()),
            _ => unsupported("deref of non-pointer intrinsic operand"),
        }
    }
    /// The address of an `Alloc` handle argument (a borrow, or an owned handle).
    fn alloc_addr_operand(&mut self, e: &Expr) -> LR<Operand> {
        if let ExprKind::Prefix { op: PrefixOp::Read | PrefixOp::Write, .. } = &e.kind {
            return Ok(self.lower_value(e, None)?.0);
        }
        let (pl, ty) = self.lower_place(e)?;
        match &ty {
            Type::Borrow(_) | Type::BorrowMut(_) => {
                let id = self.emit_temp(ty.clone(), Rvalue::Load { place: pl, ty: ty.clone() }, self.cur_span);
                Ok(Operand::Local(id))
            }
            _ => {
                let id = self.emit_temp(Type::RawPtr(Box::new(ty.clone())), Rvalue::Ref(pl), self.cur_span);
                Ok(Operand::Local(id))
            }
        }
    }

    // ---- word/unit builtin intrinsics ----
    fn lower_builtin_value(&mut self, name: &str, args: &[Expr], span: Span) -> LR<(Operand, Type)> {
        match name {
            "ptr_read" => {
                let (ptr, pty) = self.lower_value(&args[0], None)?;
                let inner = Self::pointee(&pty)?;
                if !self.is_wordy(&inner) {
                    return unsupported("aggregate ptr_read in value position");
                }
                let place = self.deref_place(ptr, inner.clone());
                let id = self.emit_temp(inner.clone(), Rvalue::Load { place, ty: inner.clone() }, span);
                if matches!(pty, Type::RawPtr(_)) {
                    self.mark_last_observable();
                }
                Ok((Operand::Local(id), inner))
            }
            "ptr_write" => {
                let (ptr, pty) = self.lower_value(&args[0], None)?;
                let inner = Self::pointee(&pty)?;
                let place = self.deref_place(ptr, inner.clone());
                self.lower_into(&args[1], &place, &inner)?;
                if matches!(pty, Type::RawPtr(_)) {
                    self.mark_last_observable();
                }
                Ok(self.unit())
            }
            "ptr_offset" => {
                let (base, pty) = self.lower_value(&args[0], None)?;
                let inner = Self::pointee(&pty)?;
                let stride = self.size_of(&inner);
                let (index, _) = self.lower_value(&args[1], Some(&Type::Scalar(ScalarTy::Isize)))?;
                let rty = Type::RawPtr(Box::new(inner));
                let id = self.emit_temp(rty.clone(), Rvalue::PtrArith { base, index, stride }, span);
                Ok((Operand::Local(id), rty))
            }
            "addr_of" | "addr_of_mut" => {
                let (pl, ty) = self.lower_place(&args[0])?;
                let rty = Type::RawPtr(Box::new(ty));
                let id = self.emit_temp(rty.clone(), Rvalue::Ref(pl), span);
                Ok((Operand::Local(id), rty))
            }
            "is_null" => {
                let (ptr, _) = self.lower_value(&args[0], None)?;
                let id = self.emit_temp(Type::bool(), Rvalue::IsNull(ptr), span);
                Ok((Operand::Local(id), Type::bool()))
            }
            "ptr_to_addr" => {
                let (ptr, _) = self.lower_value(&args[0], None)?;
                let id = self.emit_temp(Type::usize(), Rvalue::Use(ptr), span);
                Ok((Operand::Local(id), Type::usize()))
            }
            "len" => {
                // `len(read Vec[T]/Map[V])`: the length word lives at offset 8 of the
                // collection the borrow names (same field the oracle's `bi_len` reads).
                if matches!(self.collection_ty(&args[0]), Some(Type::App(n, _)) if n == "Vec" || n == "Map") {
                    let (base, _) = self.collection_base(&args[0])?;
                    let root = self.operand_local(base, Type::usize());
                    let place = Place {
                        root,
                        proj: vec![
                            Proj::Deref { inner: Type::usize() },
                            Proj::Field { offset: 8, ty: Type::usize() },
                        ],
                    };
                    let id = self.emit_temp(Type::usize(), Rvalue::Load { place, ty: Type::usize() }, span);
                    return Ok((Operand::Local(id), Type::usize()));
                }
                // A place arg reads its length word in place; a non-place arg (an
                // inline call returning a fat pointer, e.g. `len(as_bytes(x))`) has
                // no address, so materialize it into a temp local first, then read
                // its length — mirroring the tree-walker's `eval_value` in `bi_len`.
                let (pl, ty) = if Self::is_place_expr(&args[0]) {
                    self.lower_place(&args[0])?
                } else {
                    let aty = self
                        .static_ty(&args[0])
                        .ok_or_else(|| LowerError("cannot type `len` argument".into()))?;
                    let pl = self.materialize_place(&args[0], &aty)?;
                    (pl, aty)
                };
                match &ty {
                    // `str`/`[u8]`/`[T]` fat pointers carry the length word at offset 8.
                    Type::Slice(_) | Type::SliceMut(_) | Type::Str => {
                        let mut lp = pl;
                        lp.proj.push(Proj::Field { offset: 8, ty: Type::Scalar(ScalarTy::U64) });
                        let id = self.emit_temp(Type::usize(), Rvalue::Load { place: lp, ty: Type::usize() }, span);
                        Ok((Operand::Local(id), Type::usize()))
                    }
                    Type::Array(_, l) => {
                        let n = self.lay().array_len(l);
                        Ok((Operand::Const(n as i128, ScalarTy::Usize), Type::usize()))
                    }
                    _ => unsupported("len of non-slice/array"),
                }
            }
            "unbox" => {
                let (bpl, bty) = self.lower_place(&args[0])?;
                let inner = Self::pointee(&bty)?;
                if !self.is_wordy(&inner) {
                    return unsupported("aggregate unbox in value position");
                }
                let dst = self.new_local(inner.clone(), None);
                self.emit(
                    StatementKind::UnboxOp { dst: Place::local(dst), inner_ty: inner.clone(), boxed: bpl },
                    span,
                    false,
                );
                self.mark_moved(&args[0]);
                Ok((Operand::Local(dst), inner))
            }
            "sqrt" => {
                // Native IEEE square root (design 0016 §11): a total, non-faulting
                // unary float->float op emitted as `Rvalue::Sqrt`.
                let (v, vty) = self.lower_value(&args[0], None)?;
                let sty = match vty {
                    Type::Scalar(s) if s.is_float() => s,
                    _ => return unsupported("sqrt of a non-float"),
                };
                let rty = Type::Scalar(sty);
                let id = self.emit_temp(rty.clone(), Rvalue::Sqrt { ty: sty, v }, span);
                Ok((Operand::Local(id), rty))
            }
            _ => unsupported(format!("builtin `{name}` in value position")),
        }
    }

    // ---- compiler-known collection intrinsics (Vec / Map / String) ----

    /// The collection type (`Vec[T]`/`Map[V]`/`String`) an `arg0` receiver names,
    /// peeling a `read`/`write` prefix and any borrow layer — the MIR analogue of
    /// the oracle's `arg0_is_vec/map/string` guards.
    fn collection_ty(&self, arg0: &Expr) -> Option<Type> {
        let inner = match &arg0.kind {
            ExprKind::Prefix { op: PrefixOp::Read | PrefixOp::Write, expr } => expr.as_ref(),
            _ => arg0,
        };
        match self.static_ty(inner)? {
            Type::Borrow(b) | Type::BorrowMut(b) => Some(*b),
            other => Some(other),
        }
    }

    /// The element / value type of a `Vec[T]` / `Map[V]` receiver.
    fn collection_elem(&self, arg0: &Expr) -> LR<Type> {
        match self.collection_ty(arg0) {
            Some(Type::App(_, targs)) => Ok(targs.into_iter().next().unwrap_or(Type::Error)),
            _ => unsupported("collection op on a non-collection receiver"),
        }
    }

    /// Lower a collection-op receiver to a SINGLE pointer to the collection. The
    /// collection ops perform exactly one `Deref` on the base, so the base must
    /// carry exactly one borrow layer. Two shapes need adjustment:
    ///   * a re-borrow (`read v` where `v: read Vec[T]`) is a pointer-to-pointer
    ///     (`&&C`); each extra borrow layer is collapsed with a `Load`.
    ///   * a receiver naming the collection BY VALUE (`get(v,i).*`, an owned
    ///     `v`) is addressed by reference (`&C`) rather than loaded as an
    ///     aggregate value, mirroring `alloc_addr_operand`.
    fn collection_base(&mut self, e: &Expr) -> LR<(Operand, Type)> {
        let by_borrow = matches!(
            &e.kind,
            ExprKind::Prefix { op: PrefixOp::Read | PrefixOp::Write, .. }
        ) || matches!(self.static_ty(e), Some(Type::Borrow(_)) | Some(Type::BorrowMut(_)));
        let (mut op, mut ty) = if by_borrow {
            self.lower_value(e, None)?
        } else {
            let (pl, pty) = self.lower_place(e)?;
            let rty = Type::Borrow(Box::new(pty.clone()));
            let id = self.emit_temp(rty.clone(), Rvalue::Ref(pl), self.cur_span);
            (Operand::Local(id), rty)
        };
        while let Type::Borrow(inner) | Type::BorrowMut(inner) = &ty {
            if !matches!(&**inner, Type::Borrow(_) | Type::BorrowMut(_)) {
                break;
            }
            let inner_ty = (**inner).clone();
            let root = self.operand_local(op, ty.clone());
            let place = Place { root, proj: vec![Proj::Deref { inner: inner_ty.clone() }] };
            let id = self.emit_temp(
                inner_ty.clone(),
                Rvalue::Load { place, ty: inner_ty.clone() },
                self.cur_span,
            );
            op = Operand::Local(id);
            ty = inner_ty;
        }
        Ok((op, ty))
    }

    /// Root an operand in a local (materializing a const into a temp), for building
    /// a projected place over it.
    fn operand_local(&mut self, op: Operand, ty: Type) -> LocalId {
        match op {
            Operand::Local(id) => id,
            other => self.emit_temp(ty, Rvalue::Use(other), self.cur_span),
        }
    }

    /// Does `name`/`args` name an aggregate-producing collection intrinsic
    /// (`vec_new`/`map_new`/`string_new`, `pop` on a Vec, `as_str` on a String)?
    fn is_collection_agg(&self, name: &str, args: &[Expr]) -> bool {
        match name {
            "vec_new" | "map_new" | "string_new" => true,
            "pop" => matches!(args.first().and_then(|a| self.collection_ty(a)), Some(Type::App(n, _)) if n == "Vec"),
            "as_str" => matches!(args.first().and_then(|a| self.collection_ty(a)), Some(Type::Named(n)) if n == "String"),
            _ => false,
        }
    }

    /// Lower an aggregate-producing collection intrinsic into `dst`.
    fn lower_collection_agg(&mut self, name: &str, args: &[Expr], dst: &Place) -> LR<()> {
        let span = self.cur_span;
        let op = match name {
            "vec_new" | "map_new" | "string_new" => {
                let alloc = self.alloc_addr_operand(&args[0])?;
                CollOp::New { alloc }
            }
            "pop" => {
                let elem = self.collection_elem(&args[0])?;
                let (base, _) = self.collection_base(&args[0])?;
                CollOp::VecPop { base, elem }
            }
            "as_str" => {
                let (base, _) = self.collection_base(&args[0])?;
                CollOp::StringAsStr { base }
            }
            _ => return unsupported(format!("collection aggregate `{name}`")),
        };
        self.emit(StatementKind::CollectionOp { dst: dst.clone(), op }, span, false);
        Ok(())
    }

    /// Materialize a `str`/`[u8]` byte-view argument as a place over its 16-byte
    /// `{ptr@0, len@8}` fat pointer (a key or an append view).
    fn coll_view_place(&mut self, e: &Expr) -> LR<Place> {
        self.materialize_place(e, &Type::Str)
    }

    /// Lower a word/unit-producing collection intrinsic (`push`/`pop`-are-agg,
    /// `set`/`get`/`insert`/`contains`/`append`). Returns `Ok(None)` when `name`
    /// matches no collection op for `arg0`'s type, so a same-named user fn wins.
    fn lower_collection_value(
        &mut self,
        name: &str,
        args: &[Expr],
        span: Span,
    ) -> LR<Option<(Operand, Type)>> {
        let ct = match args.first().and_then(|a| self.collection_ty(a)) {
            Some(t) => t,
            None => return Ok(None),
        };
        let is_vec = matches!(&ct, Type::App(n, _) if n == "Vec");
        let is_map = matches!(&ct, Type::App(n, _) if n == "Map");
        let is_string = matches!(&ct, Type::Named(n) if n == "String");
        let first_targ = match &ct {
            Type::App(_, t) => t.first().cloned().unwrap_or(Type::Error),
            _ => Type::Error,
        };
        let unit_dst = |s: &mut Self| Place::local(s.new_local(Type::unit(), None));
        let op = match name {
            "push" if is_vec => {
                let elem = first_targ;
                let (base, _) = self.collection_base(&args[0])?;
                let value = self.materialize_place(&args[1], &elem)?;
                if !is_copy(&elem, self.items) {
                    self.mark_moved(&args[1]);
                }
                // OOM faults after the value is evaluated: the oracle's `cur_span` is
                // then the value arg's span (see `bi_vec_push`).
                CollOp::VecPush { base, elem, value, span: args[1].span }
            }
            "push" if is_string => {
                let (base, _) = self.collection_base(&args[0])?;
                let (ch, _) = self.lower_value(&args[1], Some(&Type::Scalar(ScalarTy::U32)))?;
                // `bi_string_push` resets `cur_span` to the call span before its
                // scalar-value backstop / OOM fault.
                CollOp::StringPush { base, ch, span }
            }
            "set" if is_vec => {
                let elem = first_targ;
                let (base, _) = self.collection_base(&args[0])?;
                let (index, _) = self.lower_value(&args[1], Some(&Type::usize()))?;
                // Mirror `bi_vec_set`'s order: bounds-check BEFORE evaluating the
                // value arg, so an out-of-bounds set faults without running the
                // value's side effects. `VecGet` performs exactly that check (read
                // len, eval index, fault `Bounds` at the index span); its slot-borrow
                // result is discarded. Materializing the value only afterwards keeps
                // the fault path byte-exact to the oracle (no pre-fault side effects).
                let probe = Place::local(self.new_local(Type::Borrow(Box::new(elem.clone())), None));
                self.emit(
                    StatementKind::CollectionOp {
                        dst: probe,
                        op: CollOp::VecGet {
                            base,
                            elem: elem.clone(),
                            index,
                            span: args[1].span,
                        },
                    },
                    span,
                    false,
                );
                let value = self.materialize_place(&args[2], &elem)?;
                if !is_copy(&elem, self.items) {
                    self.mark_moved(&args[2]);
                }
                // Bounds fault span: the index arg's span (the oracle checks bounds
                // right after evaluating the index — `bi_vec_set`).
                CollOp::VecSet { base, elem, index, value, span: args[1].span }
            }
            "get" if is_vec => {
                let elem = first_targ;
                let (base, _) = self.collection_base(&args[0])?;
                let (index, _) = self.lower_value(&args[1], Some(&Type::usize()))?;
                let rty = Type::Borrow(Box::new(elem.clone()));
                let dst = self.new_local(rty.clone(), None);
                self.emit(
                    StatementKind::CollectionOp {
                        dst: Place::local(dst),
                        op: CollOp::VecGet { base, elem, index, span: args[1].span },
                    },
                    span,
                    false,
                );
                return Ok(Some((Operand::Local(dst), rty)));
            }
            "get" if is_map => {
                let valty = first_targ;
                let (base, _) = self.collection_base(&args[0])?;
                let key = self.coll_view_place(&args[1])?;
                let rty = Type::Borrow(Box::new(valty.clone()));
                let dst = self.new_local(rty.clone(), None);
                self.emit(
                    StatementKind::CollectionOp {
                        dst: Place::local(dst),
                        op: CollOp::MapGet { base, valty, key, span: args[1].span },
                    },
                    span,
                    false,
                );
                return Ok(Some((Operand::Local(dst), rty)));
            }
            "insert" if is_map => {
                let valty = first_targ;
                let (base, _) = self.collection_base(&args[0])?;
                let key = self.coll_view_place(&args[1])?;
                let value = self.materialize_place(&args[2], &valty)?;
                if !is_copy(&valty, self.items) {
                    self.mark_moved(&args[2]);
                }
                // OOM faults after the value is evaluated (`bi_map_insert`).
                CollOp::MapInsert { base, valty, key, value, span: args[2].span }
            }
            "contains" if is_map => {
                let valty = first_targ;
                let (base, _) = self.collection_base(&args[0])?;
                let key = self.coll_view_place(&args[1])?;
                let dst = self.new_local(Type::bool(), None);
                self.emit(
                    StatementKind::CollectionOp { dst: Place::local(dst), op: CollOp::MapContains { base, valty, key } },
                    span,
                    false,
                );
                return Ok(Some((Operand::Local(dst), Type::bool())));
            }
            "append" if is_string => {
                let (base, _) = self.collection_base(&args[0])?;
                let view = self.coll_view_place(&args[1])?;
                // OOM span: the view arg's span (`bi_string_append` reads it, leaving
                // `cur_span` at the arg before the reserve/OOM check).
                CollOp::StringAppend { base, view, span: args[1].span }
            }
            _ => return Ok(None),
        };
        let dst = unit_dst(self);
        self.emit(StatementKind::CollectionOp { dst, op }, span, false);
        Ok(Some(self.unit()))
    }

    // ---- aggregate-producing builtins ----
    fn lower_builtin_into(&mut self, name: &str, args: &[Expr], dst: &Place, ty: &Type) -> LR<()> {
        let span = self.cur_span;
        match name {
            "ptr_read" => {
                let (ptr, pty) = self.lower_value(&args[0], None)?;
                let inner = Self::pointee(&pty)?;
                let place = self.deref_place(ptr, inner.clone());
                self.emit(StatementKind::CopyVal { dst: dst.clone(), src: place, ty: inner }, span, false);
                if matches!(pty, Type::RawPtr(_)) {
                    self.mark_last_observable();
                }
                Ok(())
            }
            "unbox" => {
                let (bpl, _) = self.lower_place(&args[0])?;
                self.emit(
                    StatementKind::UnboxOp { dst: dst.clone(), inner_ty: ty.clone(), boxed: bpl },
                    span,
                    false,
                );
                self.mark_moved(&args[0]);
                Ok(())
            }
            "box" => {
                let inner = match ty {
                    Type::BoxResult(t) => (**t).clone(),
                    _ => return unsupported("box result is not a BoxResult"),
                };
                let alloc = self.alloc_addr_operand(&args[0])?;
                let value = self.materialize_place(&args[1], &inner)?;
                if !is_copy(&inner, self.items) {
                    self.mark_moved(&args[1]);
                }
                self.emit(
                    StatementKind::BoxOp { dst: dst.clone(), inner_ty: inner, result_ty: ty.clone(), alloc, value },
                    span,
                    false,
                );
                Ok(())
            }
            "slice_of" | "slice_of_mut" => {
                let (apl, aty) = self.lower_place(&args[0])?;
                match &aty {
                    Type::Array(elem, l) => {
                        let n = self.lay().array_len(l);
                        let addr = self.emit_temp(Type::RawPtr(Box::new((**elem).clone())), Rvalue::Ref(apl), span);
                        let mut f0 = dst.clone();
                        f0.proj.push(Proj::Field { offset: 0, ty: Type::Scalar(ScalarTy::U64) });
                        self.emit(StatementKind::Store(f0, Rvalue::Use(Operand::Local(addr))), span, false);
                        let mut f8 = dst.clone();
                        f8.proj.push(Proj::Field { offset: 8, ty: Type::Scalar(ScalarTy::U64) });
                        self.emit(StatementKind::Store(f8, Rvalue::Use(Operand::Const(n as i128, ScalarTy::U64))), span, false);
                        Ok(())
                    }
                    Type::Slice(_) | Type::SliceMut(_) => {
                        self.emit(StatementKind::CopyVal { dst: dst.clone(), src: apl, ty: aty.clone() }, span, false);
                        Ok(())
                    }
                    _ => unsupported("slice_of on non-array/slice"),
                }
            }
            "subslice" => {
                let elem = match ty {
                    Type::Slice(e) | Type::SliceMut(e) => (**e).clone(),
                    _ => return unsupported("subslice result is not a slice"),
                };
                let src = self.materialize_place(&args[0], ty)?;
                let (lo, _) = self.lower_value(&args[1], Some(&Type::usize()))?;
                let (hi, _) = self.lower_value(&args[2], Some(&Type::usize()))?;
                let stride = self.stride_of(&elem);
                self.emit(
                    StatementKind::Subslice { dst: dst.clone(), src, lo, hi, stride, span },
                    span,
                    false,
                );
                Ok(())
            }
            "as_bytes" => {
                // Free retype: a `str`'s `{ptr@0, len@8}` fat pointer IS its `[u8]`
                // byte view (design 0013 §1.3), so copy the 16-byte header verbatim.
                let src = self.materialize_place(&args[0], &Type::Str)?;
                self.emit(StatementKind::CopyVal { dst: dst.clone(), src, ty: ty.clone() }, span, false);
                Ok(())
            }
            "str_from_unchecked" => {
                // Mirror of `as_bytes` (the `str` -> `[u8]` retype): a `[u8]`'s
                // `{ptr@0, len@8}` fat pointer IS a `str` (design 0013 §4). The
                // unsafe variant skips UTF-8 validation, so copy the 16-byte header
                // verbatim into the `str` destination.
                let bytes = Type::Slice(Box::new(Type::Scalar(ScalarTy::U8)));
                let src = self.materialize_place(&args[0], &bytes)?;
                self.emit(StatementKind::CopyVal { dst: dst.clone(), src, ty: ty.clone() }, span, false);
                Ok(())
            }
            "str_from" => {
                // UTF-8-validate the `[u8]` fat pointer (design 0013 §4), building
                // the `Utf8Res` enum at `dst`. The backends/MIR interp mirror the
                // tree-walker `bi_str_from` byte-for-byte (`str::from_utf8`).
                let bytes = Type::Slice(Box::new(Type::Scalar(ScalarTy::U8)));
                let src = self.materialize_place(&args[0], &bytes)?;
                self.emit(StatementKind::StrFrom { dst: dst.clone(), src }, span, false);
                Ok(())
            }
            "substr" => {
                // The `[lo, hi)` char-boundary-checked sub-view (design 0013 §3);
                // `lo`/`hi` are byte offsets. The `Bounds` fault reuses the call
                // span, exactly as the tree-walker `bi_substr` (`self.cur_span = span`).
                let src = self.materialize_place(&args[0], &Type::Str)?;
                let (lo, _) = self.lower_value(&args[1], Some(&Type::usize()))?;
                let (hi, _) = self.lower_value(&args[2], Some(&Type::usize()))?;
                self.emit(StatementKind::Substr { dst: dst.clone(), src, lo, hi, span }, span, false);
                Ok(())
            }
            _ => unsupported(format!("aggregate builtin `{name}`")),
        }
    }

    fn lower_str_into(&mut self, bytes: &str, dst: &Place) {
        let span = self.cur_span;
        let mut f0 = dst.clone();
        f0.proj.push(Proj::Field { offset: 0, ty: Type::Scalar(ScalarTy::U64) });
        self.emit(StatementKind::Store(f0, Rvalue::StrAddr(bytes.to_string())), span, false);
        let mut f8 = dst.clone();
        f8.proj.push(Proj::Field { offset: 8, ty: Type::Scalar(ScalarTy::U64) });
        self.emit(
            StatementKind::Store(f8, Rvalue::Use(Operand::Const(bytes.len() as i128, ScalarTy::U64))),
            span,
            false,
        );
    }

    // ---- cross-type `?` (From conversion, design 0007 §7.1) ----
    fn lower_try_from(
        &mut self,
        splace: &Place,
        ety: &Type,
        einfo: &[(String, Vec<Type>)],
        ok_name: &str,
        span: Span,
    ) -> LR<()> {
        // The operand's single non-`ok` (error) variant -> e1.
        let (nonok_idx, e1_payloads) = einfo
            .iter()
            .enumerate()
            .find(|(_, (n, _))| n != ok_name)
            .map(|(i, (_, p))| (i, p.clone()))
            .ok_or_else(|| LowerError("`?`: no error variant".into()))?;
        let _ = ety;
        let (e1ty, e1off) = self.lay().payload_offset(&e1_payloads, 0);
        // The return enum's non-`ok` variant -> e2.
        let ret_ename = match &self.ret_ty {
            Type::Named(n) => n.clone(),
            _ => return unsupported("cross-type `?`: return is not a nominal enum"),
        };
        let ret_ok = self.items.enums.get(&ret_ename).and_then(|e| e.ok_variant.clone());
        let ret_info = self
            .lay()
            .enum_info(&self.ret_ty)
            .ok_or_else(|| LowerError("`?`: return is not an enum".into()))?;
        let (ret_nonok_idx, e2_payloads) = ret_info
            .iter()
            .enumerate()
            .find(|(_, (n, _))| Some(n) != ret_ok.as_ref())
            .map(|(i, (_, p))| (i, p.clone()))
            .ok_or_else(|| LowerError("`?`: no return error variant".into()))?;
        let (e2ty, e2off) = self.lay().payload_offset(&e2_payloads, 0);
        let e2nom = match &e2ty {
            Type::Named(n) => n.clone(),
            _ => return unsupported("cross-type `?`: error payload is not nominal"),
        };
        // Resolve the `From[e1] for e2` impl method.
        let fnname = self
            .impls
            .from_impls
            .iter()
            .find(|(iface, args, target, _)| iface == "From" && target == &e2nom && args.first() == Some(&e1ty))
            .and_then(|(_, _, _, methods)| methods.get("from").cloned())
            .ok_or_else(|| LowerError("cross-type `?`: no matching `From` impl".into()))?;
        let _ = nonok_idx;
        // e1 lives at splace + e1off; pass it to `from` (by value/address per width).
        let mut e1pl = splace.clone();
        e1pl.proj.push(Proj::Field { offset: e1off, ty: e1ty.clone() });
        let arg = if self.is_wordy(&e1ty) {
            let id = self.emit_temp(e1ty.clone(), Rvalue::Load { place: e1pl, ty: e1ty.clone() }, span);
            Operand::Local(id)
        } else {
            let id = self.emit_temp(Type::RawPtr(Box::new(e1ty.clone())), Rvalue::Ref(e1pl), span);
            Operand::Local(id)
        };
        // e2 = from(e1); build the return enum in `_0`.
        let addr = self.new_local(Type::RawPtr(Box::new(e2ty.clone())), None);
        self.emit(
            StatementKind::Assign(addr, Rvalue::Call { func: fnname, args: vec![arg] }),
            span,
            false,
        );
        let mut tagpl = Place::local(0);
        tagpl.proj.push(Proj::Field { offset: 0, ty: Type::Scalar(ScalarTy::U64) });
        self.emit(
            StatementKind::Store(tagpl, Rvalue::Use(Operand::Const(ret_nonok_idx as i128, ScalarTy::U64))),
            span,
            false,
        );
        let mut e2dst = Place::local(0);
        e2dst.proj.push(Proj::Field { offset: e2off, ty: e2ty.clone() });
        let src = Place { root: addr, proj: vec![Proj::Deref { inner: e2ty.clone() }] };
        self.emit(StatementKind::CopyVal { dst: e2dst, src, ty: e2ty }, span, false);
        Ok(())
    }
}

/// Strip borrow/box layers to the underlying nominal type name.
fn strip_to_nominal(ty: &Type) -> Option<String> {
    match ty {
        Type::Named(n) => Some(n.clone()),
        Type::Borrow(e) | Type::BorrowMut(e) | Type::Box(e) => strip_to_nominal(e),
        _ => None,
    }
}

/// The interface-dispatch key for a receiver type: a nominal name or a builtin
/// scalar's spelling (`i64`), stripping borrows/box. Like `strip_to_nominal` but
/// also keys scalars, so `impl I for i64` dispatches (design 0007 §2.3).
fn dispatch_nominal(ty: &Type) -> Option<String> {
    match ty {
        Type::Scalar(s) => Some(scalar_name(*s).to_string()),
        Type::Named(n) => Some(n.clone()),
        Type::Borrow(e) | Type::BorrowMut(e) | Type::Box(e) => dispatch_nominal(e),
        _ => None,
    }
}

/// Names lowered as builtin intrinsics (not user functions).
fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "box" | "unbox" | "ptr_read" | "ptr_write" | "ptr_offset" | "addr_of" | "addr_of_mut"
            | "is_null" | "ptr_to_addr" | "slice_of" | "slice_of_mut" | "subslice" | "as_bytes"
            | "str_from_unchecked" | "str_from" | "substr" | "len" | "sqrt"
    )
}

fn prefix(a: &[String], b: &[String]) -> bool {
    a.len() <= b.len() && a[..] == b[..a.len()]
}

fn find_field<'f>(fields: &'f [FieldInit], name: &str) -> Option<&'f FieldInit> {
    fields.iter().find(|f| f.name == name)
}

#[allow(dead_code)]
fn is_scalar(ty: &Type) -> bool {
    matches!(ty, Type::Scalar(s) if s.is_integer() || *s == ScalarTy::Bool)
}

fn variant_name(k: &ExprKind) -> &'static str {
    match k {
        ExprKind::Match { .. } => "match",
        ExprKind::StructLit { .. } => "struct-literal",
        ExprKind::EnumCtor { .. } => "enum-ctor",
        ExprKind::ArrayLit(_) => "array-literal",
        ExprKind::ArrayRepeat { .. } => "array-repeat",
        ExprKind::Index { .. } => "index",
        ExprKind::Field { .. } => "field",
        ExprKind::Prefix { .. } => "prefix",
        ExprKind::Try(_) => "try",
        ExprKind::StrLit(_) => "string-literal",
        ExprKind::BytesLit(_) => "byte-string-literal",
        _ => "unsupported-expr",
    }
}
