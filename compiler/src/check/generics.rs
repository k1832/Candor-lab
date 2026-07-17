//! Generic-aware checking hooks (design 0007), kept out of the concrete-path
//! files. These `Checker` methods handle: naming a generic function as a value
//! (`name::[T]`), calling a generic function with value-argument-driven type
//! inference and bound-conformance checking, and calling an interface-bound
//! method on a type parameter or a concrete type with an impl. They are only
//! exercised when a program contains generics; a program with none never reaches
//! them (its calls resolve as ordinary `fns`).

use std::collections::{HashMap, HashSet};

use crate::ast::*;
use crate::diag::Diag;
use crate::resolve::GenericFnSig;
use crate::span::Span;
use crate::types::*;

use super::{Checker, Use};

/// Recursion bound for transitive associated-type projection resolution
/// (design 0009 §2.2): a malformed cyclic `type Item = ..` binding stops here
/// with the projection unresolved (cleanly rejected) instead of looping.
const MAX_PROJ_DEPTH: usize = 64;

impl<'a> Checker<'a> {
    /// Record a reached instantiation (unless we are checking a generic body at
    /// its definition site, where argument types are still opaque).
    pub(super) fn record_inst(&mut self, name: &str, args: Vec<Type>) {
        if self.def_site {
            return;
        }
        if args.iter().any(|t| matches!(t, Type::Param(_) | Type::Error)) {
            return;
        }
        if !self.insts.iter().any(|(n, a)| n == name && a == &args) {
            self.insts.push((name.to_string(), args));
        }
    }

    /// Check a `name::[T, ...]` generic value (design 0007 §6.2.1): its type is a
    /// concrete fn-pointer to the named instantiation.
    pub(super) fn check_generic_val(&mut self, name: &str, ty_args: &[Ty], span: Span) -> Type {
        let sig = match self.items.generic_fns.get(name) {
            Some(s) => s.clone(),
            None => {
                self.diags.push(Diag::error(
                    "E1001",
                    format!("`{name}` is not a generic function"),
                    span,
                ));
                return Type::Error;
            }
        };
        let args: Vec<Type> = ty_args.iter().map(|t| self.resolve_ty(t)).collect();
        if args.len() != sig.type_params.len() {
            self.diags.push(Diag::error(
                "E1005",
                format!(
                    "generic function `{name}` expects {} type argument(s), found {}",
                    sig.type_params.len(),
                    args.len()
                ),
                span,
            ));
            return Type::Error;
        }
        let map: HashMap<String, Type> =
            sig.type_params.iter().map(|(n, _)| n.clone()).zip(args.iter().cloned()).collect();
        self.check_bounds(&sig, &map, span);
        self.record_inst(name, args.clone());
        if !args.iter().any(|t| matches!(t, Type::Error)) {
            self.shapes.insert((self.cur_item, span.start), crate::generics::Shape::Fn(name.to_string(), args.clone()));
        }
        // The value is a fn-pointer over the substituted signature.
        let params: Vec<(ParamMode, Type)> = sig
            .params
            .iter()
            .map(|p| (p.mode, subst(&p.lowered, &map)))
            .collect();
        Type::FnPtr(crate::types::FnPtrTy {
            params,
            alloc: sig.alloc,
            foreign: false,
            ret: Box::new(subst(&sig.ret, &map)),
        })
    }

    /// Check that each concrete argument satisfies its parameter's declared
    /// bounds (design 0007 §2.1 bound conformance). Emits use-site errors.
    pub(super) fn check_bounds(
        &mut self,
        sig: &GenericFnSig,
        map: &HashMap<String, Type>,
        span: Span,
    ) {
        for (pname, bounds) in &sig.type_params {
            let arg = match map.get(pname) {
                Some(a) => a.clone(),
                None => continue,
            };
            if matches!(arg, Type::Error | Type::Param(_)) {
                continue;
            }
            self.check_arg_conformance(&arg, bounds, span);
        }
    }

    /// Verify one concrete type argument against one parameter's bound set.
    pub(super) fn check_arg_conformance(&mut self, arg: &Type, bounds: &[String], span: Span) {
        // A type parameter never ranges over a borrow type (design 0007 §3.5).
        if arg.is_borrow_kind() {
            self.diags.push(
                Diag::error(
                    "E1006",
                    format!("a borrow type `{}` is not a legal type argument", arg.display()),
                    span,
                )
                .with_note("borrows are for passing and computing, not abstracting over (§3.5)", None),
            );
            return;
        }
        for b in bounds {
            if b == "copy" {
                if !is_copy(arg, self.items) {
                    self.diags.push(
                        Diag::error(
                            "E1007",
                            format!("type argument `{}` does not satisfy the `copy` bound", arg.display()),
                            span,
                        )
                        .with_note("only a `copy` type may instantiate a `copy`-bounded parameter (§3.1)", None),
                    );
                }
            } else if b == "portable" {
                if !is_portable(arg, self.items) {
                    self.diags.push(
                        Diag::error(
                            "E1207",
                            format!("type argument `{}` does not satisfy the `portable` bound", arg.display()),
                            span,
                        )
                        .with_note("only a `portable` type — no `rawptr`, no borrow, transitively (design 0012 §2.2) — may instantiate a `portable`-bounded parameter", None),
                    );
                }
            } else if !self.type_implements(arg, b) {
                self.diags.push(
                    Diag::error(
                        "E1008",
                        format!("type argument `{}` does not implement interface `{}`", arg.display(), b),
                        span,
                    )
                    .with_note("no `impl` of this interface for the type is in scope (§2.1 bound conformance)", None),
                );
            }
        }
    }

    /// Does `ty` implement interface `iface`? A concrete nominal matches a
    /// bare-target impl; a generic application (`List[i64]`) matches a generic
    /// impl whose target head unifies with it (design 0007 stage 2).
    pub(super) fn type_implements(&self, ty: &Type, iface: &str) -> bool {
        self.resolve_impl_for(ty, iface).is_some()
    }

    /// Find the impl of `iface` covering `ty`, with the impl-parameter substitution
    /// (empty for a bare-nominal impl). At most one exists (coherence, §2.3).
    pub(super) fn resolve_impl_for(
        &self,
        ty: &Type,
        iface: &str,
    ) -> Option<(usize, std::collections::HashMap<String, Type>)> {
        for (i, im) in self.items.impls.iter().enumerate() {
            if im.iface != iface {
                continue;
            }
            match ty {
                Type::Named(n) => {
                    if im.target == *n && im.target_args.is_empty() {
                        return Some((i, HashMap::new()));
                    }
                }
                Type::Scalar(s) => {
                    if im.target == scalar_name(*s) && im.target_args.is_empty() {
                        return Some((i, HashMap::new()));
                    }
                }
                Type::App(n, args) => {
                    if im.target == *n && im.target_args.len() == args.len() {
                        let mut map = HashMap::new();
                        if im
                            .target_args
                            .iter()
                            .zip(args)
                            .all(|(d, a)| crate::generics::unify_inst(d, a, &mut map))
                        {
                            return Some((i, map));
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// The interface-method signature substitution for a resolved impl: the
    /// interface's type parameters mapped to the impl's (concrete-ized) interface
    /// arguments, plus `Self` mapped to the receiver type.
    pub(super) fn iface_method_subst(
        &self,
        impl_idx: usize,
        self_ty: &Type,
        impl_map: &std::collections::HashMap<String, Type>,
    ) -> std::collections::HashMap<String, Type> {
        let mut smap = std::collections::HashMap::new();
        smap.insert("Self".to_string(), self_ty.clone());
        let im = &self.items.impls[impl_idx];
        if let Some(info) = self.items.interfaces.get(&im.iface) {
            for (pname, arg) in info.type_params.iter().zip(&im.iface_args) {
                smap.insert(pname.clone(), subst(arg, impl_map));
            }
        }
        // Resolve the interface's associated type for this impl (design 0009
        // §2.2): `Self::Item` in a method signature becomes the impl's binding.
        if let Some((aname, aty)) = &im.assoc {
            let mut am = impl_map.clone();
            am.insert("Self".to_string(), self_ty.clone());
            smap.insert(format!("Self::{aname}"), self.reduce_projs(&subst(aty, &am), &am, 0));
        }
        smap
    }

    /// Resolve `ty::assoc` through the impl of `iface` for `ty` (design 0009
    /// §2.2): the impl's associated-type binding substituted with its parameters,
    /// then reduced transitively to the concrete leaf. `depth` bounds the
    /// adapter-over-adapter recursion (see [`Self::reduce_projs`]).
    fn resolve_assoc_depth(&self, ty: &Type, iface: &str, depth: usize) -> Option<Type> {
        let (idx, impl_map) = self.resolve_impl_for(ty, iface)?;
        let (_, aty) = self.items.impls[idx].assoc.as_ref()?;
        let mut m = impl_map.clone();
        m.insert("Self".to_string(), ty.clone());
        Some(self.reduce_projs(&subst(aty, &m), &m, depth))
    }

    /// Resolve a projection `Base::assoc` to a concrete type when `Base` is bound
    /// to a concrete type (design 0009 §2.2). Mirrors the monomorphizer's
    /// projection resolution (`generics::resolve_proj`): the associated-type
    /// member name selects the interface, so a projection over a concrete base
    /// resolves without the projection itself naming its interface.
    pub(super) fn resolve_proj(&self, base: &Type, assoc: &str) -> Option<Type> {
        self.resolve_proj_depth(base, assoc, 0)
    }

    fn resolve_proj_depth(&self, base: &Type, assoc: &str, depth: usize) -> Option<Type> {
        if depth > MAX_PROJ_DEPTH {
            return None;
        }
        for iface in self.items.interfaces.values() {
            if iface.assoc_type.as_deref() == Some(assoc) {
                if let Some(t) = self.resolve_assoc_depth(base, &iface.name, depth) {
                    return Some(t);
                }
            }
        }
        None
    }

    /// Reduce residual associated-type projections in `t` to their concrete leaf
    /// (design 0009 §2.2, transitive). After substituting an impl's associated
    /// binding, a residual `Base::Assoc` can remain when `Base` mapped to a
    /// concrete `App` (an adapter whose `type Item = I::Item`): concretize the
    /// base through `m` and resolve again, so an adapter-over-adapter chain
    /// reaches the underlying leaf item. A base still opaque (a `Param`, at a
    /// def-site) is left symbolic. `depth` bounds the recursion so a malformed,
    /// cyclic binding terminates with the projection unresolved (cleanly
    /// rejected) rather than looping.
    fn reduce_projs(&self, t: &Type, m: &HashMap<String, Type>, depth: usize) -> Type {
        match t {
            Type::Proj(b, a) => match m.get(b) {
                Some(c) if !matches!(c, Type::Param(_) | Type::Error) => {
                    let c = c.clone();
                    self.resolve_proj_depth(&c, a, depth + 1)
                        .unwrap_or_else(|| t.clone())
                }
                _ => t.clone(),
            },
            Type::App(n, args) => Type::App(
                n.clone(),
                args.iter().map(|x| self.reduce_projs(x, m, depth)).collect(),
            ),
            Type::Array(e, l) => Type::Array(Box::new(self.reduce_projs(e, m, depth)), l.clone()),
            Type::Slice(e) => Type::Slice(Box::new(self.reduce_projs(e, m, depth))),
            Type::SliceMut(e) => Type::SliceMut(Box::new(self.reduce_projs(e, m, depth))),
            Type::RawPtr(e) => Type::RawPtr(Box::new(self.reduce_projs(e, m, depth))),
            Type::Box(e) => Type::Box(Box::new(self.reduce_projs(e, m, depth))),
            Type::BoxResult(e) => Type::BoxResult(Box::new(self.reduce_projs(e, m, depth))),
            Type::Borrow(e) => Type::Borrow(Box::new(self.reduce_projs(e, m, depth))),
            Type::BorrowMut(e) => Type::BorrowMut(Box::new(self.reduce_projs(e, m, depth))),
            Type::FnPtr(f) => Type::FnPtr(crate::types::FnPtrTy {
                foreign: f.foreign,
                params: f
                    .params
                    .iter()
                    .map(|(mode, t)| (*mode, self.reduce_projs(t, m, depth)))
                    .collect(),
                alloc: f.alloc,
                ret: Box::new(self.reduce_projs(&f.ret, m, depth)),
            }),
            _ => t.clone(),
        }
    }

    /// Inject a resolution for every associated-type projection `Base::Assoc`
    /// appearing in `tys` whose base is already pinned to a concrete type in
    /// `smap`, so a subsequent `subst` normalizes the projection to the impl's
    /// binding (design 0009 §2.2). A base still opaque (a `Param`, e.g. at a
    /// def-site) is left untouched: the projection stays symbolic.
    pub(super) fn normalize_projections<'t>(
        &self,
        tys: impl Iterator<Item = &'t Type>,
        smap: &mut HashMap<String, Type>,
    ) {
        let mut projs = Vec::new();
        for t in tys {
            crate::generics::collect_projs(t, &mut projs);
        }
        for (base, assoc) in projs {
            let key = format!("{base}::{assoc}");
            if smap.contains_key(&key) {
                continue;
            }
            if let Some(c) = smap.get(&base) {
                if !matches!(c, Type::Error | Type::Param(_)) {
                    let c = c.clone();
                    if let Some(resolved) = self.resolve_proj(&c, &assoc) {
                        smap.insert(key, resolved);
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Whole-program generic check orchestration (design 0007 §2.1, §5.2)
// ---------------------------------------------------------------------------

use crate::resolve::{resolve_program, FnSig, GenericFnSig as GFS};
use super::FnState;

/// Diagnostics, reached instantiations, and per-node monomorphization shapes.
pub type GenericCheck = (Vec<Diag>, Vec<(String, Vec<Type>)>, std::collections::HashMap<crate::generics::ShapeKey, crate::generics::Shape>);

/// As [`GenericCheck`] plus the per-function foreign-effect report (design 0011
/// §2). The trust boundary is a source-level property — externs and their
/// discharge don't depend on monomorphization — so a generic boundary module
/// yields the same effect reach as a concrete one; this carries it for `audit`.
pub type GenericForeignCheck = (Vec<Diag>, Vec<(String, Vec<Type>)>, std::collections::HashMap<crate::generics::ShapeKey, crate::generics::Shape>, Vec<super::foreign::ForeignFnInfo>);

/// Check a program that contains generics (design 0007): resolve the generic
/// tables, definition-site-check each generic body once against its bounds with
/// opaque type parameters, then check the concrete code (whose generic call sites
/// are typed by value-argument inference and bound conformance). Returns the
/// diagnostics and the reached concrete instantiations (for monomorphization).
pub fn check_generic_program(prog: &Program, real: bool) -> GenericCheck {
    let (diags, insts, shapes, _foreign) = check_generic_program_own(prog, real, prog.items.len());
    (diags, insts, shapes)
}

/// As [`check_generic_program`] but also returns the per-function foreign-effect
/// report (design 0011 §2) for the `audit` command's effect-reach section — the
/// generic counterpart of [`super::check_program_collect`]. Same fidelity: every
/// function (concrete, generic def-site, impl method, drop hook) contributes its
/// resolved discharge/propagate status, exactly as the concrete path does.
pub fn check_generic_program_foreign(prog: &Program, real: bool) -> (Vec<Diag>, Vec<super::foreign::ForeignFnInfo>) {
    let (diags, _insts, _shapes, foreign) = check_generic_program_own(prog, real, prog.items.len());
    (diags, foreign)
}

/// As [`check_generic_program`], but only the first `own_len` items are checked
/// (def-site, concrete, and hook diagnostic passes); the remainder are imported
/// signature-only stubs (design 0008 §2) whose tables/impls resolve and whose
/// generic + `drop`-hook bodies feed instantiation, but whose bodies are never
/// re-analyzed. The generic half of the signature-only re-check tier.
pub fn check_generic_program_own(prog: &Program, real: bool, own_len: usize) -> GenericForeignCheck {
    let mut diags = Vec::new();
    let mut items = resolve_program(prog, &mut diags);
    crate::generics::resolve_tables(prog, &mut items, &mut diags);

    // Drop-hook alloc-on-drop fixpoint over both concrete and generic structs. A
    // generic struct's hook is checked once with opaque type parameters and fixes
    // its alloc-on-drop for *every* instance (design 0007 §3.4, F5); a concrete
    // hook keeps the stage-1 behavior. One monotonic fixpoint covers the mutual
    // dependencies (a hook may drop another alloc-on-drop aggregate).
    let hooks: Vec<(usize, HookInfo)> = prog
        .items
        .iter()
        .enumerate()
        .filter_map(|(i, it)| match it {
            Item::Struct(s) => s.drop_hook.as_ref().map(|b| (i, HookInfo {
                name: s.name.clone(),
                type_params: s.type_params.iter().map(|p| (p.name.clone(), p.bounds.clone())).collect(),
                block: b.clone(),
                span: s.span,
            })),
            _ => None,
        })
        .collect();
    if !hooks.is_empty() {
        let mut guard = 0;
        loop {
            let snapshot = items.clone();
            let mut changed = false;
            for (_, h) in &hooks {
                let alloc = hook_is_alloc(&snapshot, h, real);
                if alloc {
                    if h.type_params.is_empty() {
                        if !items.structs.get(&h.name).map(|s| s.alloc_on_drop).unwrap_or(true) {
                            if let Some(s) = items.structs.get_mut(&h.name) {
                                s.alloc_on_drop = true;
                                changed = true;
                            }
                        }
                    } else if !items.generic_defs.get(&h.name).map(|g| g.alloc_on_drop).unwrap_or(true) {
                        if let Some(g) = items.generic_defs.get_mut(&h.name) {
                            g.alloc_on_drop = true;
                            changed = true;
                        }
                    }
                }
            }
            guard += 1;
            if !changed || guard > hooks.len() + 1 {
                break;
            }
        }
    }

    let mut c = Checker { items: &items, diags, f: FnState::empty(), real, insts: Vec::new(), def_site: false, shapes: std::collections::HashMap::new(), cur_item: 0, type_params: Vec::new(), param_bounds: Vec::new(), expected_ty: None, cur_generic: None, foreign_report: Vec::new() };

    // --- Definition-site checks (opaque T, once per generic) ---
    for (idx, it) in prog.items[..own_len].iter().enumerate() {
        if let Item::Fn(f) = it {
            if !f.type_params.is_empty() {
                if let Some(sig) = c.items.generic_fns.get(&f.name).cloned() {
                    c.cur_item = idx;
                    c.check_generic_fn_def_site(f, &sig);
                }
            }
        }
    }
    for (idx, it) in prog.items[..own_len].iter().enumerate() {
        if let Item::Impl(im) = it {
            c.cur_item = idx;
            c.check_impl_methods_def_site(im);
        }
    }
    // Generic-struct hooks are checked once with opaque `T` (their diagnostics are
    // emitted by the final hook pass below; the alloc fixpoint ran on a snapshot).

    // --- Concrete code (typing generic uses, collecting instantiations) ---
    for (idx, it) in prog.items[..own_len].iter().enumerate() {
        c.cur_item = idx;
        match it {
            Item::Fn(f) if f.type_params.is_empty() => c.check_fn(f),
            Item::Static(s) => c.check_static(s),
            _ => {}
        }
    }
    for (idx, h) in &hooks {
        if *idx >= own_len {
            continue;
        }
        check_one_hook(&mut c, &items, h, real, *idx);
    }

    (c.diags, c.insts, c.shapes, c.foreign_report)
}

/// A struct `drop` hook to check (concrete or generic).
struct HookInfo {
    name: String,
    /// Type parameters and their bounds (empty for a concrete struct).
    type_params: Vec<(String, Vec<String>)>,
    block: Block,
    span: Span,
}

/// Build the synthetic `fn drop(self: write StructT) -> unit` for a hook: a
/// concrete struct's `self` is `Named`; a generic struct's is the parametric
/// application `Wrap[T]` (design 0007 §3.4). Returns the decl, its signature, the
/// param-copy view, and the type-parameter names.
fn synth_hook(h: &HookInfo) -> (FnDecl, FnSig, std::collections::HashMap<String, bool>, Vec<String>) {
    let pnames: Vec<String> = h.type_params.iter().map(|(n, _)| n.clone()).collect();
    let (self_sem, self_ast_kind) = if pnames.is_empty() {
        (Type::Named(h.name.clone()), TyKind::Named(h.name.clone()))
    } else {
        (
            Type::App(h.name.clone(), pnames.iter().map(|n| Type::Param(n.clone())).collect()),
            TyKind::App {
                name: h.name.clone(),
                args: pnames.iter().map(|n| Ty { kind: TyKind::Named(n.clone()), span: h.span }).collect(),
            },
        )
    };
    let sig = FnSig {
        name: format!("drop({})", h.name),
        regions: Vec::new(),
        params: vec![crate::resolve::ParamInfo {
            name: "self".to_string(),
            mode: ParamMode::Write,
            region: None,
            decl_ty: self_sem.clone(),
            lowered: crate::resolve::lower_param(ParamMode::Write, self_sem.clone()),
            span: h.span,
        }],
        alloc: true,
        foreign: false,
        ret: Type::unit(),
        ret_region: None,
        ret_span: h.span,
        span: h.span,
    };
    let fdecl = FnDecl {
        name: sig.name.clone(),
        type_params: Vec::new(),
        regions: Vec::new(),
        foreign: false,
        boundary: false,
        params: vec![Param {
            name: "self".to_string(),
            mode: ParamMode::Write,
            region: None,
            ty: Ty { kind: self_ast_kind, span: h.span },
            span: h.span,
        }],
        alloc: true,
        requires: Vec::new(),
        ensures: Vec::new(),
        ret: None,
        body: h.block.clone(),
        span: h.span,
    };
    let copy_view = h
        .type_params
        .iter()
        .map(|(n, b)| (n.clone(), b.iter().any(|x| x == "copy")))
        .collect();
    (fdecl, sig, copy_view, pnames)
}

/// Is a hook body alloc-effecting (making its type alloc-on-drop)? Checked on a
/// read-only snapshot, opaque type parameters for a generic struct.
fn hook_is_alloc(items: &crate::resolve::Items, h: &HookInfo, real: bool) -> bool {
    let (fdecl, sig, copy_view, pnames) = synth_hook(h);
    let mut view = items.clone();
    view.type_param_copy = copy_view;
    view.type_param_portable = h
        .type_params
        .iter()
        .map(|(n, b)| (n.clone(), b.iter().any(|x| x == "portable")))
        .collect();
    let mut c = Checker {
        items: &view,
        diags: Vec::new(),
        f: FnState::empty(),
        real,
        insts: Vec::new(),
        def_site: !pnames.is_empty(),
        shapes: std::collections::HashMap::new(),
        cur_item: 0,
        type_params: pnames,
        param_bounds: h.type_params.clone(),
        expected_ty: None,
        cur_generic: None, foreign_report: Vec::new(),
    };
    c.check_fn_with_sig(&fdecl, &sig);
    c.f.alloc.site.is_some()
}

/// Definition-site check of one hook body (emitting diagnostics into `outer`).
fn check_one_hook(outer: &mut Checker, items: &crate::resolve::Items, h: &HookInfo, real: bool, item: usize) {
    let (fdecl, sig, copy_view, pnames) = synth_hook(h);
    let mut view = items.clone();
    view.type_param_copy = copy_view;
    view.type_param_portable = h
        .type_params
        .iter()
        .map(|(n, b)| (n.clone(), b.iter().any(|x| x == "portable")))
        .collect();
    let mut c = Checker {
        items: &view,
        diags: std::mem::take(&mut outer.diags),
        f: FnState::empty(),
        real,
        insts: Vec::new(),
        def_site: !pnames.is_empty(),
        shapes: std::collections::HashMap::new(),
        cur_item: item,
        type_params: pnames,
        param_bounds: h.type_params.clone(),
        expected_ty: None,
        cur_generic: None, foreign_report: Vec::new(),
    };
    c.check_fn_with_sig(&fdecl, &sig);
    outer.diags = c.diags;
    outer.shapes.extend(c.shapes);
    outer.insts.extend(c.insts);
    outer.foreign_report.extend(c.foreign_report);
}

impl<'a> Checker<'a> {
    /// Definition-site check of a generic function body (design 0007 §2.1, §3):
    /// its type parameters are opaque, non-`copy` unless bounded `copy`,
    /// needs-drop, fieldless, and only the bound-interface methods are callable.
    pub(super) fn check_generic_fn_def_site(&mut self, f: &FnDecl, sig: &GFS) {
        // Build the def-site view of the items (params + their copy bounds).
        let mut view = self.items.clone();
        view.type_param_copy = sig
            .type_params
            .iter()
            .map(|(n, bounds)| (n.clone(), bounds.iter().any(|b| b == "copy")))
            .collect();
        view.type_param_portable = sig
            .type_params
            .iter()
            .map(|(n, bounds)| (n.clone(), bounds.iter().any(|b| b == "portable")))
            .collect();
        let concrete_sig = FnSig {
            name: sig.name.clone(),
            regions: sig.regions.clone(),
            params: sig.params.clone(),
            alloc: sig.alloc,
            foreign: sig.foreign,
            ret: sig.ret.clone(),
            ret_region: sig.ret_region.clone(),
            ret_span: sig.ret_span,
            span: sig.span,
        };
        let mut c = Checker {
            items: &view,
            diags: std::mem::take(&mut self.diags),
            f: FnState::empty(),
            real: self.real,
            insts: Vec::new(),
            def_site: true,
            shapes: std::collections::HashMap::new(),
            cur_item: self.cur_item,
            type_params: Vec::new(),
            param_bounds: Vec::new(),
            expected_ty: None,
            cur_generic: None, foreign_report: Vec::new(),
        };
        c.type_params = sig.type_params.iter().map(|(n, _)| n.clone()).collect();
        c.param_bounds = sig.type_params.clone();
        c.cur_generic = Some(sig.name.clone());
        c.check_fn_with_sig(f, &concrete_sig);
        self.diags = c.diags;
        self.shapes.extend(c.shapes);
        self.insts.extend(c.insts);
        self.foreign_report.extend(c.foreign_report);
    }

    /// Definition-site check of an impl's method bodies (design 0007 §2.1, §3).
    /// A concrete impl's `Self` is its target nominal; a generic impl
    /// (`impl[T] I for List[T]`) is checked once with opaque type parameters,
    /// `Self` bound to the parametric target, and the parameters' bounds in scope.
    pub(super) fn check_impl_methods_def_site(&mut self, im: &ImplDecl) {
        let generic = !im.type_params.is_empty();
        let app_target = matches!(&im.target.kind, TyKind::App { .. });
        if !generic && !app_target {
            // Concrete nominal-target impl (stage 1): `Self` = the target nominal.
            let target = match &im.target.kind {
                TyKind::Named(n) => n.clone(),
                _ => return,
            };
            let iface_args: Vec<Type> = im.iface_args.iter().map(|a| self.resolve_ty(a)).collect();
            for m in &im.methods {
                let mut fdecl = m.clone();
                crate::generics::subst_self_fndecl(&mut fdecl, &target);
                let sig = self.impl_method_sig(&fdecl, &target, &iface_args);
                let mut c = Checker {
                    items: self.items,
                    diags: std::mem::take(&mut self.diags),
                    f: FnState::empty(),
                    real: self.real,
                    insts: Vec::new(),
                    def_site: false,
                    shapes: std::collections::HashMap::new(),
                    cur_item: self.cur_item,
                    type_params: Vec::new(),
                    param_bounds: Vec::new(),
                    expected_ty: None,
                    cur_generic: None, foreign_report: Vec::new(),
                };
                c.check_fn_with_sig(&fdecl, &sig);
                self.diags = c.diags;
                self.shapes.extend(c.shapes);
                self.insts.extend(c.insts);
                self.foreign_report.extend(c.foreign_report);
            }
            return;
        }
        // Generic (or concrete-application-target) impl: check each method body
        // once with the impl's type parameters opaque and `Self` = the parametric
        // target application (design 0007 §3 conservatism, same as a generic fn).
        let mut view = self.items.clone();
        view.type_param_copy = im
            .type_params
            .iter()
            .map(|p| (p.name.clone(), p.bounds.iter().any(|b| b == "copy")))
            .collect();
        view.type_param_portable = im
            .type_params
            .iter()
            .map(|p| (p.name.clone(), p.bounds.iter().any(|b| b == "portable")))
            .collect();
        let tp_names: Vec<String> = im.type_params.iter().map(|p| p.name.clone()).collect();
        let param_bounds: Vec<(String, Vec<String>)> =
            im.type_params.iter().map(|p| (p.name.clone(), p.bounds.clone())).collect();
        for m in &im.methods {
            let mut fdecl = m.clone();
            crate::generics::subst_self_ty(&mut fdecl, &im.target);
            let mut c = Checker {
                items: &view,
                diags: std::mem::take(&mut self.diags),
                f: FnState::empty(),
                real: self.real,
                insts: Vec::new(),
                def_site: generic,
                shapes: std::collections::HashMap::new(),
                cur_item: self.cur_item,
                type_params: tp_names.clone(),
                param_bounds: param_bounds.clone(),
                expected_ty: None,
                cur_generic: None, foreign_report: Vec::new(),
            };
            let sig = c.impl_method_sig(&fdecl, "", &[]);
            c.check_fn_with_sig(&fdecl, &sig);
            self.diags = c.diags;
            self.shapes.extend(c.shapes);
            self.insts.extend(c.insts);
            self.foreign_report.extend(c.foreign_report);
        }
    }

    fn impl_method_sig(&mut self, m: &FnDecl, _target: &str, _iface_args: &[Type]) -> FnSig {
        let params = m
            .params
            .iter()
            .map(|p| {
                let dty = self.resolve_ty(&p.ty);
                ParamInfoLocal::to_param_info(p, dty)
            })
            .collect();
        let (ret, ret_region, ret_span) = match &m.ret {
            Some(rt) => {
                let base = self.resolve_ty(&rt.ty);
                let t = match rt.borrow {
                    Some(BorrowKind::Shared) => Type::Borrow(Box::new(base)),
                    Some(BorrowKind::Exclusive) => Type::BorrowMut(Box::new(base)),
                    None => base,
                };
                (t, rt.region.clone(), rt.span)
            }
            None => (Type::unit(), None, m.span),
        };
        FnSig {
            name: m.name.clone(),
            regions: m.regions.clone(),
            params,
            alloc: m.alloc,
            foreign: m.foreign,
            ret,
            ret_region,
            ret_span,
            span: m.span,
        }
    }
}

use crate::resolve::ParamInfo;
struct ParamInfoLocal;
impl ParamInfoLocal {
    fn to_param_info(p: &Param, dty: Type) -> ParamInfo {
        ParamInfo {
            name: p.name.clone(),
            mode: p.mode,
            region: p.region.clone(),
            lowered: crate::resolve::lower_param(p.mode, dty.clone()),
            decl_ty: dty,
            span: p.span,
        }
    }
}

// ---------------------------------------------------------------------------
// Generic call sites and interface method calls (design 0007 §2.2, §2.3)
// ---------------------------------------------------------------------------

impl<'a> Checker<'a> {
    /// Type a call to a generic function: infer its type arguments from the value
    /// arguments (§2.2), check bound conformance (§2.1), record the instantiation,
    /// and return the substituted return type. Effects/loans are handled by the
    /// substituted parameter modes exactly as an ordinary call.
    pub(super) fn check_generic_call(&mut self, name: &str, args: &[Expr], span: Span) -> Type {
        let sig = self.items.generic_fns.get(name).cloned().unwrap();
        if args.len() != sig.params.len() {
            self.diags.push(Diag::error(
                "E0706",
                format!("generic function `{}` expects {} argument(s), found {}", name, sig.params.len(), args.len()),
                span,
            ));
            return Type::Error;
        }
        // Infer type arguments by unifying each declared parameter type with the
        // argument's synthesized type.
        let mut subst_map: HashMap<String, Type> = HashMap::new();
        let param_names: HashSet<String> = sig.type_params.iter().map(|(n, _)| n.clone()).collect();
        let mut arg_tys: Vec<Type> = Vec::new();
        for (p, a) in sig.params.iter().zip(args) {
            let inner: &Expr = match &a.kind {
                ExprKind::OutArg(i) => i,
                _ => a,
            };
            let at = self.synth_arg_type(inner);
            arg_tys.push(at.clone());
            unify(&p.decl_ty, &at, &param_names, &mut subst_map);
        }
        // Expected-type-driven inference (design 0007 §6.2): when the value
        // arguments leave a parameter unpinned — e.g. a niladic generic call like
        // `nil()` returning `List[T]` — unify the declared return type against the
        // expected type (a `let`/assignment-target annotation) to recover it. Value
        // arguments still take precedence (`unify` never overwrites) (F4).
        if let Some(expected) = self.expected_ty.clone() {
            unify(&sig.ret, &expected, &param_names, &mut subst_map);
        }
        // Any parameter not inferred defaults to Error (reported as needing annotation).
        let mut targs: Vec<Type> = Vec::new();
        for (n, _) in &sig.type_params {
            match subst_map.get(n) {
                Some(t) => targs.push(t.clone()),
                None => {
                    self.diags.push(Diag::error(
                        "E1002",
                        format!("cannot infer type parameter `{n}` of `{name}` from the arguments"),
                        span,
                    ));
                    targs.push(Type::Error);
                }
            }
        }
        // Normalize associated-type projections in the parameter and return types
        // against the now-known type arguments BEFORE mode-checking the arguments
        // (design 0009 §2.2): a parameter typed `fn(A, I::Item) -> A` with `I`
        // pinned to a concrete iterator gets `I::Item` -> the impl's `Item`, so a
        // concrete fn-pointer argument checks against the resolved signature. A
        // still-opaque base (a self-call at a def-site) leaves the projection symbolic.
        self.normalize_projections(
            sig.params.iter().map(|p| &p.decl_ty).chain(std::iter::once(&sig.ret)),
            &mut subst_map,
        );
        // Emit argument accesses (moves/borrows/effects) against substituted modes.
        for (p, a) in sig.params.iter().zip(args) {
            self.clear_carried();
            let decl = subst(&p.decl_ty, &subst_map);
            self.check_arg_mode(p.mode, &decl, a);
        }
        self.check_bounds(&sig, &subst_map, span);
        // Polymorphic recursion: a self-call whose inferred type argument nests a
        // type parameter under a constructor has no fixed point (design 0007
        // §5.1.1) — a definition-site error, decidable here.
        if self.def_site && self.cur_generic.as_deref() == Some(name)
            && targs.iter().any(param_grows)
        {
            self.diags.push(
                Diag::error(
                    "E1020",
                    format!("polymorphic recursion: `{name}` instantiates itself with a growing type argument"),
                    span,
                )
                .with_note("a self-instantiation nesting a type parameter under a constructor does not terminate (§5.1.1)", None),
            );
        }
        if sig.alloc {
            self.note_alloc(span, format!("call to `alloc` generic function `{name}` (§4.1)"));
        }
        self.record_inst(name, targs.clone());
        if !targs.iter().any(|t| matches!(t, Type::Error)) {
            self.shapes.insert((self.cur_item, span.start), crate::generics::Shape::Fn(name.to_string(), targs.clone()));
        }
        self.clear_carried();
        subst(&sig.ret, &subst_map)
    }

    /// Best-effort synthesis of an argument's type for inference (not emitting).
    /// Reuses the full checker in a diagnostics-suppressed probe.
    pub(super) fn synth_arg_type(&mut self, e: &Expr) -> Type {
        let mark = self.diags.len();
        let saved_cur = self.f.cur;
        self.f.cur = None; // suppress CFG action emission during the probe
        let t = self.check_expr(e, Use::Value);
        self.f.cur = saved_cur;
        self.diags.truncate(mark);
        // Strip a borrow to get the underlying value type for inference.
        match t {
            Type::Borrow(inner) | Type::BorrowMut(inner) => *inner,
            other => other,
        }
    }

    /// Resolve `receiver.method(args)` against interface impls. Returns `None` if
    /// this is not a recognized interface method (so the caller can fall through
    /// to the fn-pointer/error path).
    pub(super) fn try_method_call(
        &mut self,
        base: &Expr,
        method: &str,
        args: &[Expr],
        span: Span,
    ) -> Option<Type> {
        let recv_ty = self.synth_arg_type(base);
        // Byte iteration over `str`/`[u8]` (design 0013 §3): a str/byte-slice
        // receiver answers the ground-floor `Indexed` method `at(read self, i) ->
        // Opt[u8]` (0009), so `for b in read s` yields `u8`. Wired directly (str is
        // not a nominal that can host an impl); the program supplies the ground-floor
        // `Opt` enum the `for`-desugar names.
        if method == "at" {
            let is_byteview = matches!(&recv_ty, crate::types::Type::Str)
                || matches!(&recv_ty, crate::types::Type::Slice(e) if matches!(**e, crate::types::Type::Scalar(crate::token::ScalarTy::U8)));
            if is_byteview {
                self.check_expr(base, Use::BorrowShared);
                if args.len() == 1 {
                    let it = self.check_expr(&args[0], Use::Value);
                    self.expect_integer(&it, args[0].span);
                } else {
                    self.diags.push(Diag::error("E0706", "method `at` expects 1 argument(s)".to_string(), span));
                }
                return Some(crate::types::Type::Named("Opt".to_string()));
            }
            // A `Vec[T]` receiver answers the same ground-floor `Indexed` method
            // `at(read self, i: usize) -> Opt[T]`, wiring `for x in read v` (0009).
            if let crate::types::Type::App(n, _) = &recv_ty {
                if n == "Vec" {
                    self.check_expr(base, Use::BorrowShared);
                    if args.len() == 1 {
                        let it = self.check_expr(&args[0], Use::Value);
                        self.expect_integer(&it, args[0].span);
                    } else {
                        self.diags.push(Diag::error("E0706", "method `at` expects 1 argument(s)".to_string(), span));
                    }
                    return Some(crate::types::Type::Named("Opt".to_string()));
                }
            }
        }
        // The region-free BORROWED-yield protocol `RefIndexed` (OBL-ITER-BORROW):
        // `count(read self) -> usize` (the loop bound) and `get_ref(read self, i:
        // usize) -> read Item` (a reborrow of element `i`, no copy). A `Vec[T]`
        // receiver answers both, wiring `for read x in read v`. `get_ref` is the
        // compact-default single-borrow-in/single-borrow-out shape (0001 §3.3): the
        // returned `read T` derives from `read self` with no region variable.
        if method == "count" {
            if let crate::types::Type::App(n, _) = &recv_ty {
                if n == "Vec" {
                    self.check_expr(base, Use::BorrowShared);
                    if !args.is_empty() {
                        self.diags.push(Diag::error("E0706", "method `count` expects 0 argument(s)".to_string(), span));
                    }
                    return Some(crate::types::Type::usize());
                }
            }
        }
        if method == "get_ref" {
            if let crate::types::Type::App(n, targs) = &recv_ty {
                if n == "Vec" {
                    let elem = targs.first().cloned().unwrap_or(crate::types::Type::Error);
                    self.check_expr(base, Use::BorrowShared);
                    if args.len() == 1 {
                        let it = self.check_expr(&args[0], Use::Value);
                        self.expect_integer(&it, args[0].span);
                    } else {
                        self.diags.push(Diag::error("E0706", "method `get_ref` expects 1 argument(s)".to_string(), span));
                    }
                    return Some(crate::types::Type::Borrow(Box::new(elem)));
                }
            }
        }
        // The receiver's nominal / application / type-parameter identity.
        match &recv_ty {
            Type::Param(pname) => {
                // Def-site: only the bound interfaces provide methods (§2.1).
                let ifaces = self.param_bound_ifaces(pname);
                for iface in &ifaces {
                    if let Some(m) = self.iface_method(iface, method) {
                        // Record which interface resolved this call so monomorphization
                        // can stamp it onto the callee and dispatch honors the bound
                        // (design 0007 §2.3), not whichever impl registered last.
                        self.shapes.insert(
                            (self.cur_item, span.start),
                            crate::generics::Shape::Method(iface.clone()),
                        );
                        let mut smap = std::collections::HashMap::new();
                        smap.insert("Self".to_string(), recv_ty.clone());
                        return Some(self.check_iface_method_call(base, &m, &smap, args, span));
                    }
                }
                self.diags.push(
                    Diag::error(
                        "E1002",
                        format!("no method `{method}` on type parameter `{pname}`"),
                        span,
                    )
                    .with_note("only methods declared by the parameter's bound interfaces are callable (§2.1)", None),
                );
                Some(Type::Error)
            }
            Type::Named(_) | Type::App(_, _) | Type::Scalar(_) => {
                // Concrete: find the impl providing the method for this type (a bare
                // nominal, a builtin scalar, or a generic application matching a
                // generic impl head).
                let found = (0..self.items.impls.len()).find(|&i| {
                    let im = &self.items.impls[i];
                    im.methods.contains_key(method) && self.impl_covers(i, &recv_ty)
                });
                let idx = found?;
                let iface = self.items.impls[idx].iface.clone();
                // Record the resolved interface so dispatch runs this exact impl even
                // when the receiver type impls two interfaces sharing `method`.
                self.shapes.insert(
                    (self.cur_item, span.start),
                    crate::generics::Shape::Method(iface.clone()),
                );
                let (_, impl_map) = self.resolve_impl_for(&recv_ty, &iface)?;
                // Bound conformance for the generic impl's own parameters (§2.1):
                // a `List[NonShow]` receiver calling a method of an
                // `impl[T: Show] … for List[T]` must supply a `Show` type.
                let bounds: Vec<(String, Vec<String>)> = self.items.impls[idx].type_params.clone();
                for (pn, bs) in &bounds {
                    if let Some(arg) = impl_map.get(pn) {
                        self.check_arg_conformance(arg, bs, span);
                    }
                }
                let m = self.iface_method(&iface, method)?;
                let smap = self.iface_method_subst(idx, &recv_ty, &impl_map);
                Some(self.check_iface_method_call(base, &m, &smap, args, span))
            }
            _ => None,
        }
    }

    /// Does the impl at `idx` cover receiver type `ty` (target head + unify)?
    pub(super) fn impl_covers(&self, idx: usize, ty: &Type) -> bool {
        let im = &self.items.impls[idx];
        match ty {
            Type::Named(n) => im.target == *n && im.target_args.is_empty(),
            Type::Scalar(s) => im.target == scalar_name(*s) && im.target_args.is_empty(),
            Type::App(n, args) => {
                im.target == *n && im.target_args.len() == args.len() && {
                    let mut map = std::collections::HashMap::new();
                    im.target_args.iter().zip(args).all(|(d, a)| crate::generics::unify_inst(d, a, &mut map))
                }
            }
            _ => false,
        }
    }

    pub(super) fn param_bound_ifaces(&self, pname: &str) -> Vec<String> {
        // The current def-site's fn signature bounds are encoded in items via the
        // interfaces the parameter is bound to; we recover them from the type
        // parameter's registered bounds carried on the generic sig being checked.
        self
            .param_bounds
            .iter()
            .find(|(n, _)| n == pname)
            .map(|(_, b)| b.clone())
            .unwrap_or_default()
    }

    pub(super) fn iface_method(&self, iface: &str, method: &str) -> Option<crate::resolve::IfaceMethod> {
        self.items
            .interfaces
            .get(iface)
            .and_then(|i| i.methods.iter().find(|m| m.name == method).cloned())
    }

    /// Type-check an interface-method call given its resolved signature. The
    /// receiver is accessed per the `self` mode; the effect is inherited (§4.1).
    fn check_iface_method_call(
        &mut self,
        base: &Expr,
        m: &crate::resolve::IfaceMethod,
        smap: &std::collections::HashMap<String, Type>,
        args: &[Expr],
        span: Span,
    ) -> Type {
        // Receiver access per self mode.
        let recv_use = match m.self_mode {
            ParamMode::Read => Use::BorrowShared,
            ParamMode::Write => Use::BorrowExcl,
            ParamMode::Take | ParamMode::Out => Use::Value,
        };
        self.clear_carried();
        self.check_expr(base, recv_use);
        // The loan the receiver contributes: a borrow returned from a `read`/`write
        // self` method reborrows the receiver, so it carries the receiver's loan out of
        // the call (design 0015 §4.3/§5; 0001 §2.3 step 3: a reborrow extends the
        // parent's obligation). The landing binding then anchors it, so an escaped
        // yield keeps the collection frozen — the `get_ref` hinge §5 rests on.
        let recv_carried = self.take_carried();
        let recv_prov = self.f.carried_prov.clone();
        let self_is_borrow_in = matches!(m.self_mode, ParamMode::Read | ParamMode::Write);
        if args.len() != m.params.len() {
            self.diags.push(Diag::error(
                "E0706",
                format!("method `{}` expects {} argument(s), found {}", m.name, m.params.len(), args.len()),
                span,
            ));
        }
        let mut per_arg: Vec<Vec<usize>> = Vec::new();
        let mut per_prov: Vec<Option<String>> = Vec::new();
        let mut param_is_borrow_in: Vec<bool> = Vec::new();
        for ((mode, pty), a) in m.params.iter().zip(args) {
            self.clear_carried();
            let pty = subst(pty, smap);
            self.check_arg_mode(*mode, &pty, a);
            per_prov.push(self.f.carried_prov.clone());
            per_arg.push(self.take_carried());
            param_is_borrow_in
                .push(matches!(mode, ParamMode::Read | ParamMode::Write) || pty.is_borrow_kind());
        }
        if m.alloc {
            self.note_alloc(span, format!("call to `alloc` interface method `{}` (§4.1)", m.name));
        }
        let ret = subst(&m.ret, smap);
        // Reborrow-through-call-return: a returned borrow derives, by the compact
        // default (0001 §3.3), from the method's sole borrow-in. When that is the
        // receiver (`get_ref`'s `read self`), the return carries the receiver's
        // loan(s); the landing binding then anchors them (`carries_borrow` admits the
        // method call). With more than one borrow-in and no region tag the source is
        // ambiguous — carry nothing, matching the free-fn `region_source_indices` rule.
        if ret.is_borrow_kind() {
            let borrow_in_count =
                usize::from(self_is_borrow_in) + param_is_borrow_in.iter().filter(|b| **b).count();
            if borrow_in_count == 1 {
                if self_is_borrow_in {
                    self.set_carried(recv_carried, recv_prov);
                } else {
                    let idx = param_is_borrow_in.iter().position(|b| *b).unwrap();
                    self.set_carried(per_arg[idx].clone(), per_prov[idx].clone());
                }
            } else {
                self.clear_carried();
            }
        } else {
            self.clear_carried();
        }
        ret
    }
}

/// Unify a declared (parametric) type with a concrete argument type, binding type
/// parameters in `out`. A best-effort structural match (design 0007 §2.2).
fn unify(decl: &Type, arg: &Type, params: &HashSet<String>, out: &mut HashMap<String, Type>) {
    match (decl, arg) {
        (Type::Param(n), a) if params.contains(n) => {
            if !matches!(a, Type::Error | Type::IntLit) {
                out.entry(n.clone()).or_insert_with(|| a.clone());
            } else if matches!(a, Type::IntLit) {
                out.entry(n.clone()).or_insert(Type::Scalar(ScalarTy::I64));
            }
        }
        (Type::App(_, da), Type::App(_, aa)) => {
            for (d, a) in da.iter().zip(aa) {
                unify(d, a, params, out);
            }
        }
        // Infer through a fn-pointer parameter's component types (design 0009
        // §1.2 / E1002 completion): `f: fn(T) -> U` pins `U` from `f`'s return
        // and `T` from its parameters — ordinary body-local unification.
        (Type::FnPtr(d), Type::FnPtr(a)) if d.params.len() == a.params.len() => {
            for ((_, dt), (_, at)) in d.params.iter().zip(&a.params) {
                unify(dt, at, params, out);
            }
            unify(&d.ret, &a.ret, params, out);
        }
        (Type::Box(d), Type::Box(a))
        | (Type::BoxResult(d), Type::BoxResult(a))
        | (Type::RawPtr(d), Type::RawPtr(a))
        | (Type::Slice(d), Type::Slice(a))
        | (Type::SliceMut(d), Type::SliceMut(a))
        | (Type::Borrow(d), Type::Borrow(a))
        | (Type::BorrowMut(d), Type::BorrowMut(a))
        | (Type::Array(d, _), Type::Array(a, _)) => unify(d, a, params, out),
        _ => {}
    }
}

use crate::token::ScalarTy;

// ---------------------------------------------------------------------------
// Generic struct literals and enum constructors (design 0007 §2.2, §5)
// ---------------------------------------------------------------------------

impl<'a> Checker<'a> {
    /// Type a generic struct literal `Name { .. }`, inferring the type arguments
    /// from the field values, checking each field against the substituted type.
    pub(super) fn check_generic_struct_lit(
        &mut self,
        name: &str,
        g: &GenericDecl,
        fields: &[FieldInit],
        span: Span,
    ) -> Type {
        let pset: HashSet<String> = g.params.iter().cloned().collect();
        let mut smap: HashMap<String, Type> = HashMap::new();
        for fi in fields {
            if let Some((_, fty)) = g.fields.iter().find(|(fn_, _)| fn_ == &fi.name) {
                let vt = self.synth_arg_type(&fi.value);
                unify(fty, &vt, &pset, &mut smap);
            }
        }
        let targs = self.finish_type_args(name, g, &smap, span);
        // Fold the resolved arguments (which may have come from the expected-type
        // hint, not the field values) back into `smap`, so a field whose value
        // gives no evidence for a parameter — e.g. a niladic generic-enum ctor
        // like `Node::Nil` in `head: Node[T]` — still gets a *concrete* expected
        // type below, rather than a leftover `Param` that mono cannot lower (F3).
        for (p, t) in g.params.iter().zip(&targs) {
            if !matches!(t, Type::Error) {
                smap.insert(p.clone(), t.clone());
            }
        }
        // Normalize associated-type projections in the field types against the
        // now-known concrete parameter bindings (design 0009 §2.2): a field typed
        // `fn(I::Item) -> U` with `I` pinned to a concrete iterator resolves
        // `I::Item` to the impl's `Item`, so the field value checks against the
        // concrete signature rather than the opaque projection.
        self.normalize_projections(g.fields.iter().map(|(_, t)| t), &mut smap);
        for fi in fields {
            match g.fields.iter().find(|(fn_, _)| fn_ == &fi.name) {
                Some((_, fty)) => {
                    let expect = subst(fty, &smap);
                    self.check_against(&fi.value, &expect);
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
        if !targs.iter().any(|t| matches!(t, Type::Error)) {
            self.record_inst(name, targs.clone());
            self.shapes.insert((self.cur_item, span.start), crate::generics::Shape::Type(name.to_string(), targs.clone()));
        }
        Type::App(name.to_string(), targs)
    }

    /// Type a generic enum constructor `Enum::Variant(args)`, inferring the type
    /// arguments from the payload arguments.
    pub(super) fn check_generic_enum_ctor(
        &mut self,
        enum_name: &str,
        g: &GenericDecl,
        variant: &str,
        args: &[Expr],
        span: Span,
    ) -> Type {
        let v = match g.variants.iter().find(|(n, _, _)| n == variant) {
            Some(v) => v.clone(),
            None => {
                self.diags.push(Diag::error("E0108", format!("`{enum_name}` has no variant `{variant}`"), span));
                for a in args {
                    self.check_expr(a, Use::Value);
                }
                return Type::Error;
            }
        };
        let pset: HashSet<String> = g.params.iter().cloned().collect();
        let mut smap: HashMap<String, Type> = HashMap::new();
        if args.len() == v.1.len() {
            for (a, pty) in args.iter().zip(&v.1) {
                let at = self.synth_arg_type(a);
                unify(pty, &at, &pset, &mut smap);
            }
        } else {
            self.diags.push(Diag::error(
                "E0605",
                format!("variant `{enum_name}::{variant}` expects {} payload(s), found {}", v.1.len(), args.len()),
                span,
            ));
        }
        let targs = self.finish_type_args(enum_name, g, &smap, span);
        // Fold resolved arguments back into `smap` (see `check_generic_struct_lit`)
        // so a niladic-payload nesting still substitutes to a concrete type (F3).
        for (p, t) in g.params.iter().zip(&targs) {
            if !matches!(t, Type::Error) {
                smap.insert(p.clone(), t.clone());
            }
        }
        for (a, pty) in args.iter().zip(&v.1) {
            let expect = subst(pty, &smap);
            self.check_against(a, &expect);
        }
        if !targs.iter().any(|t| matches!(t, Type::Error)) {
            self.record_inst(enum_name, targs.clone());
            self.shapes.insert((self.cur_item, span.start), crate::generics::Shape::Type(enum_name.to_string(), targs.clone()));
        }
        Type::App(enum_name.to_string(), targs)
    }

    /// Collect the ordered concrete type arguments from an inference map, erroring
    /// on any parameter left uninferred at a concrete site.
    fn finish_type_args(&mut self, name: &str, g: &GenericDecl, smap: &HashMap<String, Type>, span: Span) -> Vec<Type> {
        // A hint from the expected type (e.g. a `let` annotation) resolves any
        // parameter the value arguments cannot pin down.
        let hint: Vec<Type> = match &self.expected_ty {
            Some(Type::App(en, eargs)) if en == name && eargs.len() == g.params.len() => eargs.clone(),
            _ => Vec::new(),
        };
        let mut targs = Vec::new();
        for (i, p) in g.params.iter().enumerate() {
            match smap.get(p) {
                Some(t) => targs.push(t.clone()),
                None if !hint.is_empty() && !matches!(hint[i], Type::Error) => targs.push(hint[i].clone()),
                None => {
                    if !self.def_site {
                        self.diags.push(Diag::error(
                            "E1002",
                            format!("cannot infer type parameter `{p}` of `{name}`"),
                            span,
                        ));
                    }
                    targs.push(if self.def_site { Type::Param(p.clone()) } else { Type::Error });
                }
            }
        }
        targs
    }
}

/// Does `t` nest a type parameter under a type constructor (a *growing* argument
/// for polymorphic-recursion detection, design 0007 §5.1.1)? A bare `Param` is
/// not growing; a `Param` inside an `App`/`Box`/... is.
fn param_grows(t: &Type) -> bool {
    fn has_param(t: &Type) -> bool {
        match t {
            Type::Param(_) => true,
            Type::App(_, a) => a.iter().any(has_param),
            Type::Box(e) | Type::BoxResult(e) | Type::Array(e, _) | Type::Slice(e)
            | Type::SliceMut(e) | Type::RawPtr(e) | Type::Borrow(e) | Type::BorrowMut(e) => has_param(e),
            _ => false,
        }
    }
    match t {
        Type::Param(_) => false,
        Type::App(_, a) => a.iter().any(has_param),
        Type::Box(e) | Type::BoxResult(e) | Type::Array(e, _) | Type::Slice(e)
        | Type::SliceMut(e) | Type::RawPtr(e) | Type::Borrow(e) | Type::BorrowMut(e) => has_param(e),
        _ => false,
    }
}

impl<'a> Checker<'a> {
    /// Cross-type `?` (design 0007 §7.1): the operand is a result-shaped enum with
    /// non-`ok` payload `E1`, the enclosing function returns a result-shaped enum
    /// with non-`ok` payload `E2`, and `impl From[E1] for E2` exists. Records the
    /// conversion's effect (inherited from `From::from`, §7.1). Returns `Some(())`
    /// when the conversion is available.
    pub(super) fn try_from_conversion(
        &mut self,
        _operand_ty: &Type,
        ret_ty: &Type,
        operand_enum: &crate::types::EnumTy,
        _ename: &str,
        span: Span,
    ) -> Option<()> {
        let e1 = non_ok_payload(operand_enum)?;
        let ret_enum = crate::check::patterns_resolve_enum(ret_ty, self.items)?;
        // E2 is the enclosing return enum's non-`ok` *payload* type (the error
        // type the conversion targets), not the return enum itself (§7.1).
        let e2 = non_ok_payload(&ret_enum)?;
        // `E2` is a bare nominal or a generic application (`AppErr[i64]`); find the
        // `From[E1]` impl whose target head unifies with it (design 0007 stage 2).
        let (e2_head, e2_args): (String, &[Type]) = match &e2 {
            Type::Named(n) => (n.clone(), &[]),
            Type::App(n, args) => (n.clone(), args.as_slice()),
            _ => return None,
        };
        let found = self.items.impls.iter().find(|im| {
            if crate::generics::base_name(&im.iface) != "From"
                || im.target != e2_head
                || im.iface_args.first() != Some(&e1)
            {
                return false;
            }
            if im.target_args.len() != e2_args.len() {
                return false;
            }
            let mut map = std::collections::HashMap::new();
            im.target_args.iter().zip(e2_args).all(|(d, a)| crate::generics::unify_inst(d, a, &mut map))
        })?;
        // Effect inheritance: the conversion inherits `From::from`'s declared
        // effect (design 0007 §7.1).
        if let Some(iface) = self
            .items
            .interfaces
            .values()
            .find(|i| crate::generics::base_name(&i.name) == "From")
        {
            if let Some(m) = iface.methods.iter().find(|m| m.name == "from") {
                if m.alloc {
                    self.note_alloc(span, "cross-type `?` calls `alloc` `From::from` (§7.1)");
                }
            }
        }
        let _ = found;
        Some(())
    }
}

/// The single non-`ok` variant's first payload type of a result-shaped enum.
fn non_ok_payload(e: &crate::types::EnumTy) -> Option<Type> {
    let ok = e.ok_variant.as_deref();
    e.variants
        .iter()
        .find(|v| Some(v.name.as_str()) != ok)
        .and_then(|v| v.payload.first().cloned())
}
