//! The generics layer (design 0007): resolves generic declarations into the
//! item tables, enforces coherence (one impl per instantiated-interface/type key)
//! and the module-level orphan rule, and monomorphizes reached instantiations
//! into a fully concrete program for the interpreter. Definition-site checking of
//! generic bodies (the opaque-`T` pass) is driven from `check::check_generic_program`.
//!
//! Stage-1 deferrals (recorded): generic *impls* (`impl[T] I for List[T]`),
//! generic struct `drop` hooks, and associated types (design 0007 §1.1, §7.2).

use std::collections::{HashMap, HashSet};

use crate::ast::*;
use crate::diag::Diag;
use crate::resolve::{lower_param, GenericFnSig, IfaceInfo, IfaceMethod, ImplInfo, Items, ParamInfo};
use crate::span::Span;
use crate::types::*;

/// Recursion bound for transitive associated-type projection resolution
/// (design 0009 §2.2): a malformed cyclic `type Item = ..` binding stops here
/// (returns the unresolved projection) instead of looping forever.
const MAX_PROJ_DEPTH: usize = 64;

/// Documented monomorphization depth limit (design 0007 §5.1.1, spec 10.4): the
/// maximum length of an instantiation *chain* — an instance requested by another
/// instance is one deeper, roots are depth 0. It backstops a genuinely divergent
/// chain (`f[T]` -> `g[Wrap[T]]` -> `f[Wrap[Wrap[T]]]` -> ...), NOT the total
/// number of instances: unbounded breadth at shallow depth is legal.
pub const MONO_DEPTH_LIMIT: usize = 64;

/// A monomorphization shape recorded at an expression node during checking: the
/// generic name and the (possibly still-parametric) type arguments to substitute.
#[derive(Clone, Debug)]
pub enum Shape {
    /// A generic function call or `name::[T]` value -> a function instance.
    Fn(String, Vec<Type>),
    /// A generic struct literal or enum constructor -> a type instance.
    Type(String, Vec<Type>),
    /// An interface method call `recv.m(args)` -> the interface the checker
    /// resolved (design 0007 §2.3). Stamped onto the call's `Field` callee so
    /// dispatch runs exactly that interface's impl even when the receiver type
    /// impls two interfaces sharing the method name `m`.
    Method(String),
}

/// The key identifying where a monomorphization [`Shape`] was recorded:
/// `(top-level item index in the merged program, node `span.start`)`. The item
/// index is what makes the key globally unique across modules: design 0008 merges
/// each module's AST *without rebasing spans*, so a bare per-file `span.start`
/// (numbered from ~0 in every file) collides between two nodes in different
/// modules. The item index disambiguates them, while `span.start` stays unique
/// *within* one item's file (the invariant the single-file path already relies on).
pub type ShapeKey = (usize, usize);

/// Does the program contain any generic construct (design 0007)?
pub fn is_generic_program(prog: &Program) -> bool {
    prog.items.iter().any(|it| match it {
        Item::Interface(_) | Item::Impl(_) => true,
        Item::Struct(s) => !s.type_params.is_empty(),
        Item::Enum(e) => !e.type_params.is_empty(),
        Item::Fn(f) => !f.type_params.is_empty(),
        Item::Static(_) => false,
        Item::Extern(_) | Item::Export(_) => false,
    })
}

/// The module of a (possibly module-qualified) global name: everything before the
/// last `::`, or the empty string for a single-file program's un-qualified name.
fn module_of(name: &str) -> String {
    match name.rfind("::") {
        Some(i) => name[..i].to_string(),
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Generic-aware AST-type resolution (param scope)
// ---------------------------------------------------------------------------

/// Arity of a compiler-known builtin generic type constructor, or `None` if
/// `name` is not one. The recognized set mirrors monomorphization's `rewrite_ty`
/// (`Vec`/`Map`); `Map[V]` is single-arg because keys are byte-strings
/// (design 0013 §1.3), not a `Map[K, V]`.
fn builtin_generic_arity(name: &str) -> Option<usize> {
    match name {
        "Vec" | "Map" => Some(1),
        _ => None,
    }
}

/// Resolve an AST type to a semantic `Type` with a set of in-scope type-parameter
/// names mapped to `Type::Param`, and generic applications to `Type::App`.
pub fn resolve_gty(
    ty: &Ty,
    params: &HashSet<String>,
    known_types: &HashSet<String>,
    generic_types: &HashSet<String>,
    diags: &mut Vec<Diag>,
) -> Type {
    match &ty.kind {
        TyKind::Scalar(s) => Type::Scalar(*s),
        TyKind::Named(n) => {
            if n == "Self" {
                Type::Param("Self".to_string())
            } else if params.contains(n) {
                Type::Param(n.clone())
            } else if known_types.contains(n) {
                Type::Named(n.clone())
            } else {
                diags.push(Diag::error("E0102", format!("unknown type `{n}`"), ty.span));
                Type::Error
            }
        }
        TyKind::Proj { base, assoc } => {
            if base != "Self" && !params.contains(base) {
                diags.push(
                    Diag::error(
                        "E1017",
                        format!("associated-type projection `{base}::{assoc}` has no bounded base"),
                        ty.span,
                    )
                    .with_note("`Base::Assoc` needs `Base` a type parameter bounded by an interface with that member (design 0009 §2.2)", None),
                );
                return Type::Error;
            }
            Type::Proj(base.clone(), assoc.clone())
        }
        TyKind::App { name, args } => {
            let ra: Vec<Type> = args
                .iter()
                .map(|a| resolve_gty(a, params, known_types, generic_types, diags))
                .collect();
            // Compiler-known builtin generic constructors (`Vec[T]`, `Map[V]`)
            // resolve in a generic context exactly as monomorphization's
            // `rewrite_ty` treats them: a `Type::App` that survives to lowering.
            // Their arity is fixed, so a wrong-arity application still errors.
            if let Some(arity) = builtin_generic_arity(name) {
                if ra.len() != arity {
                    diags.push(Diag::error(
                        "E1004",
                        format!("`{name}` expects {arity} type argument(s), found {}", ra.len()),
                        ty.span,
                    ));
                    return Type::Error;
                }
                return Type::App(name.clone(), ra);
            }
            if !generic_types.contains(name) && !params.contains(name) {
                diags.push(Diag::error(
                    "E1004",
                    format!("`{name}` is not a generic type"),
                    ty.span,
                ));
                return Type::Error;
            }
            Type::App(name.clone(), ra)
        }
        TyKind::Array { size, elem } => {
            let len = match &size.kind {
                ExprKind::IntLit { value, .. } => ArrayLen::Lit(*value),
                ExprKind::Ident(n) => ArrayLen::Named(n.clone()),
                _ => ArrayLen::Unknown,
            };
            Type::Array(Box::new(resolve_gty(elem, params, known_types, generic_types, diags)), len)
        }
        TyKind::Slice(e) => Type::Slice(Box::new(resolve_gty(e, params, known_types, generic_types, diags))),
        TyKind::SliceMut(e) => Type::SliceMut(Box::new(resolve_gty(e, params, known_types, generic_types, diags))),
        TyKind::RawPtr(e) => Type::RawPtr(Box::new(resolve_gty(e, params, known_types, generic_types, diags))),
        TyKind::Box(e) => Type::Box(Box::new(resolve_gty(e, params, known_types, generic_types, diags))),
        TyKind::BoxResult(e) => Type::BoxResult(Box::new(resolve_gty(e, params, known_types, generic_types, diags))),
        TyKind::Borrow(e) => Type::Borrow(Box::new(resolve_gty(e, params, known_types, generic_types, diags))),
        TyKind::BorrowMut(e) => Type::BorrowMut(Box::new(resolve_gty(e, params, known_types, generic_types, diags))),
        TyKind::FnPtr(fp) => Type::FnPtr(crate::types::FnPtrTy {
            params: fp
                .params
                .iter()
                .map(|p| (p.mode, resolve_gty(&p.ty, params, known_types, generic_types, diags)))
                .collect(),
            alloc: fp.alloc,
            foreign: fp.foreign,
            ret: Box::new(resolve_gty(&fp.ret, params, known_types, generic_types, diags)),
        }),
    }
}

// ---------------------------------------------------------------------------
// Table resolution + coherence + orphan
// ---------------------------------------------------------------------------

/// Resolve every generic declaration into `items` and enforce coherence and the
/// orphan rule. `items` already holds the concrete (non-generic) tables.
pub fn resolve_tables(prog: &Program, items: &mut Items, diags: &mut Vec<Diag>) {
    // Name universes.
    let mut known_types: HashSet<String> = items.structs.keys().cloned().collect();
    known_types.extend(items.enums.keys().cloned());
    let mut generic_types: HashSet<String> = HashSet::new();
    for it in &prog.items {
        match it {
            Item::Struct(s) if !s.type_params.is_empty() => {
                generic_types.insert(s.name.clone());
            }
            Item::Enum(e) if !e.type_params.is_empty() => {
                generic_types.insert(e.name.clone());
            }
            _ => {}
        }
    }
    known_types.extend(generic_types.iter().cloned());
    let no_params: HashSet<String> = HashSet::new();

    // Interfaces first (impls reference them).
    for it in &prog.items {
        if let Item::Interface(idecl) = it {
            let mut pset: HashSet<String> = idecl.type_params.iter().map(|p| p.name.clone()).collect();
            // A bare mention of the interface's associated-type member inside its
            // own method signatures denotes `Self::Item` (design 0009 §2.1/§2.2):
            // add it to the parameter scope so it resolves, then re-project it.
            let assoc = &idecl.assoc_type;
            if let Some(a) = assoc {
                pset.insert(a.clone());
            }
            let methods = idecl
                .methods
                .iter()
                .map(|m| {
                    let params = m
                        .params
                        .iter()
                        .map(|p| {
                            let t = resolve_gty(&p.ty, &pset, &known_types, &generic_types, diags);
                            (p.mode, lower_param(p.mode, selfify_assoc(&t, assoc)))
                        })
                        .collect();
                    let ret = match &m.ret {
                        Some(rt) => selfify_assoc(&ret_type(rt, &pset, &known_types, &generic_types, diags), assoc),
                        None => Type::unit(),
                    };
                    IfaceMethod { name: m.name.clone(), has_self: m.has_self, self_mode: m.self_mode, params, alloc: m.alloc, ret, span: m.span }
                })
                .collect();
            items.interfaces.insert(
                idecl.name.clone(),
                IfaceInfo {
                    name: idecl.name.clone(),
                    type_params: idecl.type_params.iter().map(|p| p.name.clone()).collect(),
                    assoc_type: idecl.assoc_type.clone(),
                    methods,
                    span: idecl.span,
                },
            );
        }
    }

    // Generic structs/enums -> generic_defs.
    for it in &prog.items {
        match it {
            Item::Struct(s) if !s.type_params.is_empty() => {
                let pset: HashSet<String> = s.type_params.iter().map(|p| p.name.clone()).collect();
                let fields = s
                    .fields
                    .iter()
                    .map(|f| (f.name.clone(), resolve_gty(&f.ty, &pset, &known_types, &generic_types, diags)))
                    .collect();
                items.generic_defs.insert(
                    s.name.clone(),
                    GenericDecl {
                        params: s.type_params.iter().map(|p| p.name.clone()).collect(),
                        is_enum: false,
                        copy: s.copy,
                        has_drop: s.drop_hook.is_some(),
                        alloc_on_drop: false,
                        fields,
                        variants: Vec::new(),
                    },
                );
            }
            Item::Enum(e) if !e.type_params.is_empty() => {
                let pset: HashSet<String> = e.type_params.iter().map(|p| p.name.clone()).collect();
                let variants = e
                    .variants
                    .iter()
                    .map(|v| {
                        let payload = v
                            .payload
                            .iter()
                            .map(|t| resolve_gty(t, &pset, &known_types, &generic_types, diags))
                            .collect();
                        (v.name.clone(), payload, v.ok)
                    })
                    .collect();
                items.generic_defs.insert(
                    e.name.clone(),
                    GenericDecl {
                        params: e.type_params.iter().map(|p| p.name.clone()).collect(),
                        is_enum: true,
                        copy: e.copy,
                        has_drop: false,
                        alloc_on_drop: false,
                        fields: Vec::new(),
                        variants,
                    },
                );
            }
            _ => {}
        }
    }

    // Generic functions -> generic_fns.
    for it in &prog.items {
        if let Item::Fn(f) = it {
            if f.type_params.is_empty() {
                continue;
            }
            let pset: HashSet<String> = f.type_params.iter().map(|p| p.name.clone()).collect();
            let params: Vec<ParamInfo> = f
                .params
                .iter()
                .map(|p| {
                    let dty = resolve_gty(&p.ty, &pset, &known_types, &generic_types, diags);
                    ParamInfo {
                        name: p.name.clone(),
                        mode: p.mode,
                        region: p.region.clone(),
                        lowered: lower_param(p.mode, dty.clone()),
                        decl_ty: dty,
                        span: p.span,
                    }
                })
                .collect();
            let (ret, ret_region, ret_span) = match &f.ret {
                Some(rt) => (
                    ret_type(rt, &pset, &known_types, &generic_types, diags),
                    rt.region.clone(),
                    rt.span,
                ),
                None => (Type::unit(), None, f.span),
            };
            // Validate every associated-type projection in the signature (design
            // 0009 §2.2): `Base::Assoc` needs `Base` bounded by an interface that
            // declares `Assoc`. Catches projection on an unbounded parameter.
            {
                let bounds: Vec<(String, Vec<String>)> =
                    f.type_params.iter().map(|p| (p.name.clone(), p.bounds.clone())).collect();
                let mut projs: Vec<(String, String)> = Vec::new();
                for pi in &params {
                    collect_projs(&pi.decl_ty, &mut projs);
                }
                collect_projs(&ret, &mut projs);
                for (base, assoc) in &projs {
                    let ok = base == "Self"
                        || bounds.iter().any(|(n, bs)| {
                            n == base
                                && bs.iter().any(|b| {
                                    items.interfaces.get(b).map(|i| i.assoc_type.as_deref() == Some(assoc.as_str())).unwrap_or(false)
                                })
                        });
                    if !ok {
                        diags.push(
                            Diag::error(
                                "E1017",
                                format!("associated-type projection `{base}::{assoc}` requires `{base}` bounded by an interface declaring `type {assoc}`"),
                                f.span,
                            )
                            .with_note("bound the parameter with the interface that owns the associated type (design 0009 §2.2)", None),
                        );
                    }
                }
            }
            items.generic_fns.insert(
                f.name.clone(),
                GenericFnSig {
                    name: f.name.clone(),
                    type_params: f.type_params.iter().map(|p| (p.name.clone(), p.bounds.clone())).collect(),
                    regions: f.regions.clone(),
                    params,
                    alloc: f.alloc,
                    foreign: f.foreign,
                    ret,
                    ret_region,
                    ret_span,
                    span: f.span,
                },
            );
        }
    }

    // Impls -> coherence + orphan + impl table.
    for it in &prog.items {
        if let Item::Impl(im) = it {
            resolve_impl(im, items, &known_types, &generic_types, &no_params, diags);
        }
    }
}

/// Replace a bare `Named(assoc)` with `Proj("Self", assoc)` throughout a type:
/// inside an interface, the member name denotes the projection on `Self` (§2.1).
fn selfify_assoc(t: &Type, assoc: &Option<String>) -> Type {
    let a = match assoc {
        Some(a) => a,
        None => return t.clone(),
    };
    match t {
        Type::Param(n) if n == a => Type::Proj("Self".to_string(), a.clone()),
        Type::Named(n) if n == a => Type::Proj("Self".to_string(), a.clone()),
        Type::App(n, args) => Type::App(n.clone(), args.iter().map(|x| selfify_assoc(x, assoc)).collect()),
        Type::Array(e, l) => Type::Array(Box::new(selfify_assoc(e, assoc)), l.clone()),
        Type::Slice(e) => Type::Slice(Box::new(selfify_assoc(e, assoc))),
        Type::SliceMut(e) => Type::SliceMut(Box::new(selfify_assoc(e, assoc))),
        Type::RawPtr(e) => Type::RawPtr(Box::new(selfify_assoc(e, assoc))),
        Type::Box(e) => Type::Box(Box::new(selfify_assoc(e, assoc))),
        Type::BoxResult(e) => Type::BoxResult(Box::new(selfify_assoc(e, assoc))),
        Type::Borrow(e) => Type::Borrow(Box::new(selfify_assoc(e, assoc))),
        Type::BorrowMut(e) => Type::BorrowMut(Box::new(selfify_assoc(e, assoc))),
        other => other.clone(),
    }
}

fn ret_type(
    rt: &RetTy,
    params: &HashSet<String>,
    known: &HashSet<String>,
    gen: &HashSet<String>,
    diags: &mut Vec<Diag>,
) -> Type {
    let base = resolve_gty(&rt.ty, params, known, gen, diags);
    match rt.borrow {
        Some(BorrowKind::Shared) => Type::Borrow(Box::new(base)),
        Some(BorrowKind::Exclusive) => Type::BorrowMut(Box::new(base)),
        None => base,
    }
}

fn resolve_impl(
    im: &ImplDecl,
    items: &mut Items,
    known: &HashSet<String>,
    gen: &HashSet<String>,
    _no_params: &HashSet<String>,
    diags: &mut Vec<Diag>,
) {
    // Type parameters of a generic impl (`impl[T] I for List[T]`), with bounds.
    let type_params: Vec<(String, Vec<String>)> =
        im.type_params.iter().map(|p| (p.name.clone(), p.bounds.clone())).collect();
    let pset: HashSet<String> = type_params.iter().map(|(n, _)| n.clone()).collect();

    // Resolve the target head + (parametric) arguments. A generic impl's target is
    // an application `List[T]`; a concrete impl's is a bare nominal or a builtin
    // scalar (`i64`). A scalar target is keyed by its spelling (design 0007 §2.3);
    // the orphan check below confines it to interfaces you own.
    let mut scalar_target = false;
    let (target, target_args): (String, Vec<Type>) = match &im.target.kind {
        TyKind::Named(n) => (n.clone(), Vec::new()),
        TyKind::App { name, args } => {
            let ta: Vec<Type> = args
                .iter()
                .map(|a| resolve_gty(a, &pset, known, gen, diags))
                .collect();
            (name.clone(), ta)
        }
        TyKind::Scalar(s) => {
            scalar_target = true;
            (scalar_name(*s).to_string(), Vec::new())
        }
        _ => {
            diags.push(Diag::error("E1012", "an impl target must be a nominal or scalar type".to_string(), im.target.span));
            return;
        }
    };
    // A builtin scalar has no declared nominal; it is always a valid target head.
    if !scalar_target && !known.contains(&target) {
        diags.push(Diag::error("E0102", format!("unknown type `{target}`"), im.target.span));
        return;
    }
    // A generic impl's parameters must all appear in the target so that each
    // instantiation is driven by a reached target-type instance (design 0007 §5.1).
    if !pset.is_empty() {
        let mut mentioned: HashSet<String> = HashSet::new();
        for a in &target_args {
            collect_params(a, &mut mentioned);
        }
        for (n, _) in &type_params {
            if !mentioned.contains(n) {
                diags.push(
                    Diag::error(
                        "E1016",
                        format!("impl type parameter `{n}` does not appear in the target type"),
                        im.span,
                    )
                    .with_note("every generic-impl parameter must appear in the target so instances follow reached type instantiations (design 0007 §5.1)", None),
                );
                return;
            }
        }
    }
    let iface_args: Vec<Type> = im
        .iface_args
        .iter()
        .map(|a| resolve_gty(a, &pset, known, gen, diags))
        .collect();
    let iface_info = match items.interfaces.get(&im.iface) {
        Some(i) => i.clone(),
        None => {
            diags.push(Diag::error("E1003", format!("unknown interface `{}`", im.iface), im.span));
            return;
        }
    };
    // Coherence: no two impl heads for the same interface may unify (design 0007
    // §2.3). This subsumes the concrete exact-duplicate case and rejects any pair
    // of generic heads with a common instance — with no blanket/overlap impls there
    // is at most one impl per instantiated `(I[args], T)` key.
    if let Some(prev) = items.impls.iter().find(|e| {
        e.iface == im.iface
            && heads_overlap(
                (e.target.as_str(), &e.iface_args, &e.target_args, &e.type_params),
                (target.as_str(), &iface_args, &target_args, &type_params),
            )
    }) {
        let _ = prev;
        diags.push(
            Diag::error(
                "E1009",
                format!("overlapping impl of `{}` for `{}`", im.iface, target),
                im.span,
            )
            .with_note("at most one impl of a given instantiated interface for a given type; two heads that unify overlap (design 0007 §2.3)", None),
        );
        return;
    }
    // Orphan rule at module granularity (design 0007 §2.3): the impl must live in
    // the target head's module or the interface's declaration module. A builtin
    // scalar has no home module (`module_of` yields ""), so a scalar impl is legal
    // only where `home` is "" (the single-file root program, untagged by the module
    // driver) or equals the interface's module — i.e. you may impl an interface for a
    // scalar only if you own the interface (or you are the single root program).
    // Every other module (any dependency, and even a module-tree's non-owner root)
    // is rejected. Since an interface is owned by exactly one module, at most one
    // legal `impl Ord for i64` can exist per linked program; two packages cannot each
    // bless a divergent one, and any duplicate that still reaches the same program is
    // caught as an overlap (E1009 above). This reuses the nominal machinery unchanged.
    let target_mod = module_of(&target);
    let iface_mod = module_of(&im.iface);
    let home = im.home.clone().unwrap_or_default();
    if home != target_mod && home != iface_mod {
        diags.push(
            Diag::error(
                "E1013",
                format!(
                    "orphan impl: `impl {} for {}` must live in the module of `{}` or of `{}`",
                    im.iface, target, target, im.iface
                ),
                im.span,
            )
            .with_note("the orphan rule is enforced at module granularity (design 0007 §2.3, 0008)", None),
        );
        return;
    }
    // Verify the method set matches the interface's.
    let mut methods = HashMap::new();
    for m in &im.methods {
        if !iface_info.methods.iter().any(|im2| im2.name == m.name) {
            diags.push(Diag::error(
                "E1014",
                format!("method `{}` is not declared by interface `{}`", m.name, im.iface),
                m.span,
            ));
        }
        methods.insert(m.name.clone(), impl_method_fn_name(&im.iface, &iface_args, &target, &m.name));
    }
    for want in &iface_info.methods {
        if !methods.contains_key(&want.name) {
            diags.push(Diag::error(
                "E1015",
                format!("impl of `{}` for `{}` is missing method `{}`", im.iface, target, want.name),
                im.span,
            ));
        }
    }
    // Signature conformance (design 0007 §3.5/§4.1): each interface method the
    // impl provides must carry the SAME signature — self receiver, parameter
    // count/modes/types, return type, and effect marker — after substituting
    // `Self` -> the target, the interface's type parameters -> the impl's
    // interface arguments, and the associated type -> the impl's binding.
    check_impl_conformance(im, &iface_info, &iface_args, &target, &target_args, &pset, known, gen, diags);
    // Associated-type binding (design 0009 §2.1): the impl must bind the member
    // the interface declares (`type Item = T`), and only that member.
    let assoc: Option<(String, Type)> = match (&iface_info.assoc_type, &im.assoc_binding) {
        (Some(want), Some((got, bty))) => {
            if want != got {
                diags.push(Diag::error(
                    "E1018",
                    format!("impl of `{}` binds `type {got}`, but the interface declares `type {want}`", im.iface),
                    im.span,
                ));
            }
            Some((got.clone(), resolve_gty(bty, &pset, known, gen, diags)))
        }
        (Some(want), None) => {
            diags.push(
                Diag::error(
                    "E1018",
                    format!("impl of `{}` for `{}` is missing associated type `{want}`", im.iface, target),
                    im.span,
                )
                .with_note("bind it with `type {want} = <type>;` in the impl body (design 0009 §2.1)", None),
            );
            None
        }
        (None, Some((got, _))) => {
            diags.push(Diag::error(
                "E1018",
                format!("interface `{}` declares no associated type, but the impl binds `type {got}`", im.iface),
                im.span,
            ));
            None
        }
        (None, None) => None,
    };
    items.impls.push(ImplInfo {
        iface: im.iface.clone(),
        iface_args,
        target,
        type_params,
        target_args,
        methods,
        assoc,
        span: im.span,
    });
}

/// Collect every associated-type projection `(base, assoc)` mentioned in `t`.
pub(crate) fn collect_projs(t: &Type, out: &mut Vec<(String, String)>) {
    match t {
        Type::Proj(b, a) => out.push((b.clone(), a.clone())),
        Type::App(_, args) => for x in args { collect_projs(x, out); },
        Type::Array(e, _) | Type::Slice(e) | Type::SliceMut(e) | Type::RawPtr(e)
        | Type::Box(e) | Type::BoxResult(e) | Type::Borrow(e) | Type::BorrowMut(e) => collect_projs(e, out),
        Type::FnPtr(f) => { for (_, pt) in &f.params { collect_projs(pt, out); } collect_projs(&f.ret, out); }
        _ => {}
    }
}

/// Collect every `Type::Param` name mentioned in `t`.
fn collect_params(t: &Type, out: &mut HashSet<String>) {
    match t {
        Type::Param(n) => { out.insert(n.clone()); }
        Type::App(_, a) => for x in a { collect_params(x, out); },
        Type::Array(e, _) | Type::Slice(e) | Type::SliceMut(e) | Type::RawPtr(e)
        | Type::Box(e) | Type::BoxResult(e) | Type::Borrow(e) | Type::BorrowMut(e) => collect_params(e, out),
        _ => {}
    }
}

type Head<'a> = (&'a str, &'a Vec<Type>, &'a Vec<Type>, &'a Vec<(String, Vec<String>)>);

/// Do two impl heads unify: SAME target constructor, and (iface_args + target_args)
/// unify under their respective type-parameter sets? Distinct target nominals never
/// overlap, so `impl I for A` and `impl I for B` (distinct nominals) coexist. Each
/// side's parameters are unification variables local to that side; the two are kept
/// in separate binding maps so a shared spelling `T` on both sides does not falsely
/// couple them (design 0007 §2.3 overlap check).
fn heads_overlap(a: Head, b: Head) -> bool {
    let (atgt, aif, atg, ap) = a;
    let (btgt, bif, btg, bp) = b;
    if atgt != btgt || aif.len() != bif.len() || atg.len() != btg.len() {
        return false;
    }
    let aset: HashSet<&str> = ap.iter().map(|(n, _)| n.as_str()).collect();
    let bset: HashSet<&str> = bp.iter().map(|(n, _)| n.as_str()).collect();
    let mut amap: HashMap<String, Type> = HashMap::new();
    let mut bmap: HashMap<String, Type> = HashMap::new();
    aif.iter().zip(bif).all(|(x, y)| unify2(x, y, &aset, &bset, &mut amap, &mut bmap))
        && atg.iter().zip(btg).all(|(x, y)| unify2(x, y, &aset, &bset, &mut amap, &mut bmap))
}

/// Two-sided unification for the overlap check.
fn unify2(
    x: &Type,
    y: &Type,
    aset: &HashSet<&str>,
    bset: &HashSet<&str>,
    amap: &mut HashMap<String, Type>,
    bmap: &mut HashMap<String, Type>,
) -> bool {
    // A parameter of side A binds to `y` (consistently); likewise side B.
    if let Type::Param(n) = x {
        if aset.contains(n.as_str()) {
            return match amap.get(n) {
                Some(prev) => prev == y,
                None => { amap.insert(n.clone(), y.clone()); true }
            };
        }
    }
    if let Type::Param(n) = y {
        if bset.contains(n.as_str()) {
            return match bmap.get(n) {
                Some(prev) => prev == x,
                None => { bmap.insert(n.clone(), x.clone()); true }
            };
        }
    }
    match (x, y) {
        (Type::Scalar(a), Type::Scalar(b)) => a == b,
        (Type::Named(a), Type::Named(b)) => a == b,
        (Type::Param(a), Type::Param(b)) => a == b,
        (Type::App(a, aa), Type::App(b, bb)) => {
            a == b && aa.len() == bb.len()
                && aa.iter().zip(bb).all(|(p, q)| unify2(p, q, aset, bset, amap, bmap))
        }
        (Type::Box(a), Type::Box(b))
        | (Type::BoxResult(a), Type::BoxResult(b))
        | (Type::RawPtr(a), Type::RawPtr(b))
        | (Type::Slice(a), Type::Slice(b))
        | (Type::SliceMut(a), Type::SliceMut(b))
        | (Type::Borrow(a), Type::Borrow(b))
        | (Type::BorrowMut(a), Type::BorrowMut(b))
        | (Type::Array(a, _), Type::Array(b, _)) => unify2(a, b, aset, bset, amap, bmap),
        _ => false,
    }
}

/// The final `::`-separated segment of a possibly module-qualified name. The
/// cross-type `?` conversion interface `From` is a compiler-known lang item
/// (design 0007 §7.1); the module tree qualifies it (`core::res::From`), so it
/// is matched by base name rather than by a hardcoded qualified string (F2).
pub fn base_name(name: &str) -> &str {
    name.rsplit("::").next().unwrap_or(name)
}

/// The mangled free-function name an impl method lowers to.
pub fn impl_method_fn_name(iface: &str, iface_args: &[Type], target: &str, method: &str) -> String {
    let mut key = iface.to_string();
    for a in iface_args {
        key.push('$');
        key.push_str(&mangle_ty(a));
    }
    format!("<impl {key} for {target}>::{method}")
}

/// A mangled fragment for a concrete type, used in instance names.
pub fn mangle_ty(t: &Type) -> String {
    match t {
        Type::Scalar(s) => scalar_name(*s).to_string(),
        Type::IntLit => "int".to_string(),
        Type::Named(n) => n.replace("::", "_"),
        Type::App(n, args) => {
            let mut s = n.replace("::", "_");
            for a in args {
                s.push('_');
                s.push_str(&mangle_ty(a));
            }
            s
        }
        Type::Array(e, _) => format!("arr_{}", mangle_ty(e)),
        Type::Slice(e) => format!("slice_{}", mangle_ty(e)),
        Type::Str => "str".to_string(),
        Type::SliceMut(e) => format!("slicemut_{}", mangle_ty(e)),
        Type::RawPtr(e) => format!("ptr_{}", mangle_ty(e)),
        Type::Box(e) => format!("Box_{}", mangle_ty(e)),
        Type::BoxResult(e) => format!("BoxResult_{}", mangle_ty(e)),
        Type::Borrow(e) => format!("ref_{}", mangle_ty(e)),
        Type::BorrowMut(e) => format!("refmut_{}", mangle_ty(e)),
        Type::FnPtr(_) => "fnptr".to_string(),
        Type::Param(n) => n.clone(),
        Type::Proj(b, a) => format!("{b}_{a}"),
        Type::Never => "never".to_string(),
        Type::Error => "err".to_string(),
    }
}

/// Instance name of a generic function at a concrete type-argument tuple.
pub fn inst_fn_name(name: &str, args: &[Type]) -> String {
    let mut s = name.to_string();
    for a in args {
        s.push('$');
        s.push_str(&mangle_ty(a));
    }
    s
}

/// Instance name of a generic nominal (struct/enum) at a concrete tuple.
pub fn inst_type_name(name: &str, args: &[Type]) -> String {
    inst_fn_name(name, args)
}

// ---------------------------------------------------------------------------
// Monomorphization (design 0007 §5.1–§5.2)
// ---------------------------------------------------------------------------

/// The result of monomorphizing: a fully concrete program plus any depth-limit
/// resource diagnostic (design 0007 §5.1.1).
pub struct Mono {
    pub program: Program,
    pub diags: Vec<Diag>,
}

struct Monomorphizer<'a> {
    shapes: &'a HashMap<ShapeKey, Shape>,
    /// Item index (into `prog.items`) of the def whose body is being rewritten —
    /// the first half of every [`ShapeKey`] lookup, so a node resolves to the
    /// shape the checker recorded for *this* item even when a per-file `span.start`
    /// collides with a node in another module.
    cur_item: usize,
    /// Each generic def name -> its item index, so an emitted instance rewrites its
    /// body under the generic def's own [`ShapeKey`] namespace.
    generic_item: HashMap<String, usize>,
    fn_done: HashSet<String>,
    type_done: HashSet<String>,
    fn_work: Vec<(String, Vec<Type>, usize)>,
    type_work: Vec<(String, Vec<Type>, usize)>,
    out: Vec<Item>,
    diags: Vec<Diag>,
    /// Chain depth to stamp on instances enqueued while the current item is being
    /// emitted: the emitting item's depth + 1 (roots are 0). See [`MONO_DEPTH_LIMIT`].
    next_depth: usize,
    /// The original generic decls, keyed by name.
    generic_fns_ast: HashMap<String, FnDecl>,
    generic_structs_ast: HashMap<String, StructDecl>,
    generic_enums_ast: HashMap<String, EnumDecl>,
    /// App-target impls (generic or concrete-instance), with their resolved head
    /// info, emitted per reached target-type instance (design 0007 stage 2).
    app_impls: Vec<AppImpl>,
    /// Emitted impl instances, keyed by (impl span, concrete target instance name).
    impl_done: HashSet<String>,
    /// Pending impl instances to emit: (app_impls index, concrete param types).
    impl_work: Vec<(usize, Vec<Type>, usize)>,
    /// Associated-type resolution table (design 0009 §2.2): each entry is
    /// `(target_head, target_args, impl_param_names, assoc_member, assoc_type)`.
    /// A projection `C::Assoc` resolves by unifying `C` with the target pattern
    /// and substituting the impl params into `assoc_type`.
    assoc_impls: Vec<AssocImpl>,
}

/// One associated-type resolution record (design 0009 §2.2):
/// `(target_head, target_args, impl_param_names, assoc_member, assoc_type)`.
type AssocImpl = (String, Vec<Type>, Vec<String>, String, Type);

/// A monomorphizable impl whose target is an application (`impl[T] I for List[T]`
/// or a concrete `impl I for AppErr[i64]`): its AST plus the resolved parametric
/// target/interface arguments and parameter names.
struct AppImpl {
    decl: ImplDecl,
    param_names: Vec<String>,
    target_head: String,
    target_args: Vec<Type>,
    iface_args: Vec<Type>,
    /// Item index of this impl in `prog.items` — the [`ShapeKey`] namespace its
    /// method bodies were checked under (design 0008 cross-module span disambig).
    def_item: usize,
}

/// Monomorphize `prog` into a concrete program using the checker-recorded shapes
/// and reached instantiations.
pub fn monomorphize(
    prog: &Program,
    insts: &[(String, Vec<Type>)],
    shapes: &HashMap<ShapeKey, Shape>,
) -> Mono {
    let mut generic_fns_ast = HashMap::new();
    let mut generic_structs_ast = HashMap::new();
    let mut generic_enums_ast = HashMap::new();
    let mut generic_item: HashMap<String, usize> = HashMap::new();
    for (i, it) in prog.items.iter().enumerate() {
        match it {
            Item::Fn(f) if !f.type_params.is_empty() => {
                generic_fns_ast.insert(f.name.clone(), f.clone());
                generic_item.insert(f.name.clone(), i);
            }
            Item::Struct(s) if !s.type_params.is_empty() => {
                generic_structs_ast.insert(s.name.clone(), s.clone());
                generic_item.insert(s.name.clone(), i);
            }
            Item::Enum(e) if !e.type_params.is_empty() => {
                generic_enums_ast.insert(e.name.clone(), e.clone());
                generic_item.insert(e.name.clone(), i);
            }
            _ => {}
        }
    }
    // Resolve the impl tables (quietly) to recover each impl's parametric
    // target/interface arguments, then keep the app-target impls for per-instance
    // emission (design 0007 stage 2).
    let mut qd = Vec::new();
    let mut qitems = crate::resolve::resolve_program(prog, &mut qd);
    resolve_tables(prog, &mut qitems, &mut qd);
    let mut app_impls: Vec<AppImpl> = Vec::new();
    for (i, it) in prog.items.iter().enumerate() {
        if let Item::Impl(im) = it {
            if !matches!(&im.target.kind, TyKind::App { .. }) {
                continue; // bare-nominal target: emitted by `emit_impl`
            }
            if let Some(info) = qitems.impls.iter().find(|e| e.span == im.span) {
                app_impls.push(AppImpl {
                    decl: im.clone(),
                    param_names: info.type_params.iter().map(|(n, _)| n.clone()).collect(),
                    target_head: info.target.clone(),
                    target_args: info.target_args.clone(),
                    iface_args: info.iface_args.clone(),
                    def_item: i,
                });
            }
        }
    }
    let mut assoc_impls: Vec<AssocImpl> = Vec::new();
    for info in &qitems.impls {
        if let Some((aname, aty)) = &info.assoc {
            assoc_impls.push((
                info.target.clone(),
                info.target_args.clone(),
                info.type_params.iter().map(|(n, _)| n.clone()).collect(),
                aname.clone(),
                aty.clone(),
            ));
        }
    }
    let mut m = Monomorphizer {
        shapes,
        cur_item: 0,
        generic_item,
        fn_done: HashSet::new(),
        type_done: HashSet::new(),
        fn_work: Vec::new(),
        type_work: Vec::new(),
        out: Vec::new(),
        diags: Vec::new(),
        next_depth: 0,
        generic_fns_ast,
        generic_structs_ast,
        generic_enums_ast,
        app_impls,
        impl_done: HashSet::new(),
        impl_work: Vec::new(),
        assoc_impls,
    };

    // Seed from checker-reached instantiations.
    for (name, args) in insts {
        m.enqueue(name, args.clone());
    }

    // Rewrite and emit the concrete (non-generic) items.
    let empty: HashMap<String, Type> = HashMap::new();
    for (i, it) in prog.items.iter().enumerate() {
        m.cur_item = i;
        match it {
            Item::Fn(f) if f.type_params.is_empty() => {
                let mut f2 = f.clone();
                m.rewrite_fn(&mut f2, &empty);
                m.out.push(Item::Fn(f2));
            }
            Item::Struct(s) if s.type_params.is_empty() => {
                let mut s2 = s.clone();
                for fld in &mut s2.fields {
                    fld.ty = m.rewrite_ty(&fld.ty, &empty);
                }
                if let Some(b) = &mut s2.drop_hook {
                    m.rewrite_block(b, &empty);
                }
                m.out.push(Item::Struct(s2));
            }
            Item::Enum(e) if e.type_params.is_empty() => {
                let mut e2 = e.clone();
                for v in &mut e2.variants {
                    for t in &mut v.payload {
                        *t = m.rewrite_ty(t, &empty);
                    }
                }
                m.out.push(Item::Enum(e2));
            }
            Item::Static(s) => {
                let mut s2 = s.clone();
                s2.ty = m.rewrite_ty(&s2.ty, &empty);
                m.rewrite_expr(&mut s2.value, &empty);
                m.out.push(Item::Static(s2));
            }
            Item::Impl(im) if !matches!(&im.target.kind, TyKind::App { .. }) => m.emit_impl(im),
            // `extern`/`export` FFI decls are never generic and need no substitution;
            // carry them through so a generic image that also does I/O keeps its
            // foreign symbols live (they would otherwise vanish, faulting at the call).
            Item::Extern(_) | Item::Export(_) => m.out.push(it.clone()),
            _ => {}
        }
    }

    // Drive the worklists to a fixed point (with the documented depth backstop).
    m.drive();

    Mono { program: Program { items: m.out }, diags: m.diags }
}

impl<'a> Monomorphizer<'a> {
    fn enqueue(&mut self, name: &str, args: Vec<Type>) {
        if args.iter().any(|t| matches!(t, Type::Param(_) | Type::Error)) {
            return;
        }
        if self.generic_fns_ast.contains_key(name) {
            let key = inst_fn_name(name, &args);
            if self.fn_done.insert(key) {
                self.fn_work.push((name.to_string(), args.clone(), self.next_depth));
            }
        } else if self.generic_structs_ast.contains_key(name) || self.generic_enums_ast.contains_key(name) {
            let key = inst_type_name(name, &args);
            if self.type_done.insert(key) {
                self.type_work.push((name.to_string(), args.clone(), self.next_depth));
            }
        }
        // Any app-target impl whose target head matches this instance is reached:
        // unify its parametric target arguments with the concrete ones and queue
        // the resulting impl instance (design 0007 stage 2 dispatch).
        for idx in 0..self.app_impls.len() {
            if self.app_impls[idx].target_head != name {
                continue;
            }
            let mut map: HashMap<String, Type> = HashMap::new();
            let ta = self.app_impls[idx].target_args.clone();
            if ta.len() != args.len() {
                continue;
            }
            let ok = ta.iter().zip(&args).all(|(d, a)| unify_inst(d, a, &mut map));
            if !ok {
                continue;
            }
            let pnames = self.app_impls[idx].param_names.clone();
            let cargs: Vec<Type> = pnames.iter().map(|n| map.get(n).cloned().unwrap_or(Type::Error)).collect();
            if cargs.iter().any(|t| matches!(t, Type::Param(_) | Type::Error)) {
                continue;
            }
            let inst_name = inst_type_name(name, &args);
            let key = format!("{}#{}", self.app_impls[idx].decl.span.start, inst_name);
            if self.impl_done.insert(key) {
                self.impl_work.push((idx, cargs, self.next_depth));
            }
        }
    }

    fn drive(&mut self) {
        // Pop priority (impl, then type, then fn) must match the peek below so the
        // depth read is the depth of the item actually popped.
        loop {
            let depth = if let Some((_, _, d)) = self.impl_work.last() {
                *d
            } else if let Some((_, _, d)) = self.type_work.last() {
                *d
            } else if let Some((_, _, d)) = self.fn_work.last() {
                *d
            } else {
                return;
            };
            // Bound the instantiation *chain*, not the total instance count: only a
            // divergent chain climbs past the limit (design 0007 §5.1.1, spec 10.4).
            if depth > MONO_DEPTH_LIMIT {
                self.diags.push(
                    Diag::error(
                        "E1099",
                        format!("monomorphization instantiation chain exceeded the depth limit ({MONO_DEPTH_LIMIT})"),
                        Span::point(0),
                    )
                    .with_note("a generic body instantiating another with a growing type argument does not terminate; this is a compile resource limit, not a type error (design 0007 §5.1.1)", None),
                );
                return;
            }
            self.next_depth = depth + 1;
            if let Some((idx, cargs, _)) = self.impl_work.pop() {
                self.emit_impl_instance(idx, &cargs);
            } else if let Some((name, args, _)) = self.type_work.pop() {
                self.emit_type_instance(&name, &args);
            } else if let Some((name, args, _)) = self.fn_work.pop() {
                self.emit_fn_instance(&name, &args);
            }
        }
    }

    fn param_map(names: &[String], args: &[Type]) -> HashMap<String, Type> {
        names.iter().cloned().zip(args.iter().cloned()).collect()
    }

    fn emit_fn_instance(&mut self, name: &str, args: &[Type]) {
        self.cur_item = self.generic_item[name];
        let decl = self.generic_fns_ast.get(name).unwrap().clone();
        let pnames: Vec<String> = decl.type_params.iter().map(|p| p.name.clone()).collect();
        let mut map = Self::param_map(&pnames, args);
        self.inject_assoc_bindings(&pnames, &mut map);
        let mut f2 = decl;
        f2.name = inst_fn_name(name, args);
        f2.type_params = Vec::new();
        self.rewrite_fn(&mut f2, &map);
        self.out.push(Item::Fn(f2));
    }

    fn emit_type_instance(&mut self, name: &str, args: &[Type]) {
        self.cur_item = self.generic_item[name];
        if let Some(s) = self.generic_structs_ast.get(name).cloned() {
            let pnames: Vec<String> = s.type_params.iter().map(|p| p.name.clone()).collect();
            let map = Self::param_map(&pnames, args);
            let mut s2 = s;
            s2.name = inst_type_name(name, args);
            s2.type_params = Vec::new();
            for fld in &mut s2.fields {
                fld.ty = self.rewrite_ty(&fld.ty, &map);
            }
            // A generic struct's `drop` hook instantiates with the type arguments
            // (design 0007 §3.4): the interpreter runs it exactly as a concrete one.
            if let Some(b) = &mut s2.drop_hook {
                self.rewrite_block(b, &map);
            }
            self.out.push(Item::Struct(s2));
        } else if let Some(e) = self.generic_enums_ast.get(name).cloned() {
            let pnames: Vec<String> = e.type_params.iter().map(|p| p.name.clone()).collect();
            let map = Self::param_map(&pnames, args);
            let mut e2 = e;
            e2.name = inst_type_name(name, args);
            e2.type_params = Vec::new();
            for v in &mut e2.variants {
                for t in &mut v.payload {
                    *t = self.rewrite_ty(t, &map);
                }
            }
            self.out.push(Item::Enum(e2));
        }
    }

    /// Emit each impl method as a concrete free function; the impl block itself is
    /// kept (with concrete iface/target) for the interpreter's dispatch table.
    fn emit_impl(&mut self, im: &ImplDecl) {
        if !im.type_params.is_empty() {
            return;
        }
        // A scalar target (`i64`) keeps its `Self` bound to the scalar type, not a
        // nominal; a nominal target substitutes `Self` -> the nominal name.
        let (target, self_ty): (String, Option<Ty>) = match &im.target.kind {
            TyKind::Named(n) => (n.clone(), None),
            TyKind::Scalar(s) => (scalar_name(*s).to_string(), Some(im.target.clone())),
            _ => return,
        };
        let iface_args: Vec<Type> = im
            .iface_args
            .iter()
            .map(|a| self.ast_to_type(a, &HashMap::new()))
            .collect();
        let empty = HashMap::new();
        let mut kept = im.clone();
        kept.methods.clear();
        for m in &im.methods {
            let mut fm = m.clone();
            fm.name = impl_method_fn_name(&im.iface, &iface_args, &target, &m.name);
            match &self_ty {
                Some(ty) => subst_self_ty(&mut fm, ty),
                None => subst_self_fndecl(&mut fm, &target),
            }
            self.rewrite_fn(&mut fm, &empty);
            self.out.push(Item::Fn(fm));
            // Keep a signature-only stub in the impl block so the interpreter can
            // recover the method's name, self mode, and mangled target.
            let mut stub = m.clone();
            match &self_ty {
                Some(ty) => subst_self_ty(&mut stub, ty),
                None => subst_self_fndecl(&mut stub, &target),
            }
            stub.body = Block { stmts: Vec::new(), span: m.span };
            kept.methods.push(stub);
        }
        kept.target = im.target.clone();
        self.out.push(Item::Impl(kept));
    }

    /// Emit one concrete instance of an app-target impl: substitute the impl's
    /// type parameters, mangle the target to its concrete nominal instance, and
    /// lower each method to a concrete free function plus a signature stub in the
    /// kept impl block (for the interpreter's dispatch table).
    fn emit_impl_instance(&mut self, idx: usize, cargs: &[Type]) {
        self.cur_item = self.app_impls[idx].def_item;
        let ai = &self.app_impls[idx];
        let im = ai.decl.clone();
        let pnames = ai.param_names.clone();
        let mut map = Self::param_map(&pnames, cargs);
        self.inject_assoc_bindings(&pnames, &mut map);
        let ctarget_args: Vec<Type> = ai.target_args.iter().map(|t| subst(t, &map)).collect();
        let target = inst_type_name(&ai.target_head, &ctarget_args);
        let ciface_args: Vec<Type> = ai.iface_args.iter().map(|t| subst(t, &map)).collect();
        let mut kept = im.clone();
        kept.methods.clear();
        kept.type_params = Vec::new();
        for mm in &im.methods {
            let mut fm = mm.clone();
            fm.type_params = Vec::new();
            fm.name = impl_method_fn_name(&im.iface, &ciface_args, &target, &mm.name);
            subst_self_fndecl(&mut fm, &target);
            self.rewrite_fn(&mut fm, &map);
            self.out.push(Item::Fn(fm));
            let mut stub = mm.clone();
            stub.type_params = Vec::new();
            subst_self_fndecl(&mut stub, &target);
            for p in &mut stub.params {
                p.ty = self.rewrite_ty(&p.ty, &map);
            }
            if let Some(rt) = &mut stub.ret {
                rt.ty = self.rewrite_ty(&rt.ty, &map);
            }
            stub.body = Block { stmts: Vec::new(), span: mm.span };
            kept.methods.push(stub);
        }
        kept.target = Ty { kind: TyKind::Named(target.clone()), span: im.target.span };
        kept.iface_args = ciface_args.iter().map(sem_to_ast_preserve).collect();
        self.out.push(Item::Impl(kept));
    }

    fn rewrite_fn(&mut self, f: &mut FnDecl, map: &HashMap<String, Type>) {
        for p in &mut f.params {
            p.ty = self.rewrite_ty(&p.ty, map);
        }
        if let Some(rt) = &mut f.ret {
            rt.ty = self.rewrite_ty(&rt.ty, map);
        }
        for r in &mut f.requires {
            self.rewrite_expr(r, map);
        }
        for r in &mut f.ensures {
            self.rewrite_expr(r, map);
        }
        self.rewrite_block(&mut f.body, map);
    }

    fn rewrite_block(&mut self, b: &mut Block, map: &HashMap<String, Type>) {
        for st in &mut b.stmts {
            match &mut st.kind {
                StmtKind::Let { ty, init, .. } => {
                    if let Some(t) = ty {
                        *t = self.rewrite_ty(t, map);
                    }
                    if let Some(e) = init {
                        self.rewrite_expr(e, map);
                    }
                }
                StmtKind::Assign { target, value } => {
                    self.rewrite_expr(target, map);
                    self.rewrite_expr(value, map);
                }
                StmtKind::Expr(e) => self.rewrite_expr(e, map),
            }
        }
    }

    /// Resolve an AST type to a concrete semantic `Type` under `map`, recording
    /// every reached generic type instantiation.
    fn ast_to_type(&mut self, ty: &Ty, map: &HashMap<String, Type>) -> Type {
        match &ty.kind {
            TyKind::Scalar(s) => Type::Scalar(*s),
            TyKind::Named(n) if n == "str" => Type::Str,
            TyKind::Named(n) => map.get(n).cloned().unwrap_or_else(|| Type::Named(n.clone())),
            TyKind::App { name, args } => {
                let sargs: Vec<Type> = args.iter().map(|a| self.ast_to_type(a, map)).collect();
                self.enqueue(name, sargs.clone());
                Type::App(name.clone(), sargs)
            }
            TyKind::Array { elem, .. } => Type::Array(Box::new(self.ast_to_type(elem, map)), ArrayLen::Unknown),
            TyKind::Slice(e) => Type::Slice(Box::new(self.ast_to_type(e, map))),
            TyKind::SliceMut(e) => Type::SliceMut(Box::new(self.ast_to_type(e, map))),
            TyKind::RawPtr(e) => Type::RawPtr(Box::new(self.ast_to_type(e, map))),
            TyKind::Box(e) => Type::Box(Box::new(self.ast_to_type(e, map))),
            TyKind::BoxResult(e) => Type::BoxResult(Box::new(self.ast_to_type(e, map))),
            TyKind::Borrow(e) => Type::Borrow(Box::new(self.ast_to_type(e, map))),
            TyKind::BorrowMut(e) => Type::BorrowMut(Box::new(self.ast_to_type(e, map))),
            TyKind::FnPtr(_) => Type::Error,
            TyKind::Proj { base, assoc } => {
                let cbase = map.get(base).cloned().unwrap_or_else(|| Type::Named(base.clone()));
                self.resolve_proj(&cbase, assoc)
            }
        }
    }

    /// Rewrite an AST type: substitute parameters, and lower every generic
    /// application to its concrete monomorphic nominal, recording the instance.
    fn rewrite_ty(&mut self, ty: &Ty, map: &HashMap<String, Type>) -> Ty {
        let kind = match &ty.kind {
            TyKind::Named(n) => match map.get(n) {
                Some(t) => self.type_to_ast_kind(t),
                None => TyKind::Named(n.clone()),
            },
            TyKind::App { name, args } if name == "Vec" || name == "Map" => {
                // Compiler-known std `Vec[T]` stays an application through
                // monomorphization; only its arguments are rewritten.
                TyKind::App { name: name.clone(), args: args.iter().map(|a| self.rewrite_ty(a, map)).collect() }
            }
            TyKind::App { name, args } => {
                let sargs: Vec<Type> = args.iter().map(|a| self.ast_to_type(a, map)).collect();
                self.enqueue(name, sargs.clone());
                TyKind::Named(inst_type_name(name, &sargs))
            }
            TyKind::Array { size, elem } => TyKind::Array {
                size: size.clone(),
                elem: Box::new(self.rewrite_ty(elem, map)),
            },
            TyKind::Slice(e) => TyKind::Slice(Box::new(self.rewrite_ty(e, map))),
            TyKind::SliceMut(e) => TyKind::SliceMut(Box::new(self.rewrite_ty(e, map))),
            TyKind::RawPtr(e) => TyKind::RawPtr(Box::new(self.rewrite_ty(e, map))),
            TyKind::Box(e) => TyKind::Box(Box::new(self.rewrite_ty(e, map))),
            TyKind::BoxResult(e) => TyKind::BoxResult(Box::new(self.rewrite_ty(e, map))),
            TyKind::Borrow(e) => TyKind::Borrow(Box::new(self.rewrite_ty(e, map))),
            TyKind::BorrowMut(e) => TyKind::BorrowMut(Box::new(self.rewrite_ty(e, map))),
            TyKind::Scalar(s) => TyKind::Scalar(*s),
            TyKind::FnPtr(fp) => {
                let mut fp2 = fp.clone();
                for p in &mut fp2.params {
                    p.ty = self.rewrite_ty(&p.ty, map);
                }
                fp2.ret = Box::new(self.rewrite_ty(&fp2.ret, map));
                TyKind::FnPtr(fp2)
            }
            TyKind::Proj { base, assoc } => {
                let cbase = map.get(base).cloned().unwrap_or_else(|| Type::Named(base.clone()));
                let resolved = self.resolve_proj(&cbase, assoc);
                self.type_to_ast_kind(&resolved)
            }
        };
        Ty { kind, span: ty.span }
    }

    /// Resolve an associated-type projection `C::assoc` to its concrete type at
    /// monomorphization (design 0009 §2.2): find the impl whose target unifies
    /// with `C` and substitute its parameters into the `assoc` binding.
    fn resolve_proj(&self, c: &Type, assoc: &str) -> Type {
        self.resolve_proj_depth(c, assoc, 0)
    }

    fn resolve_proj_depth(&self, c: &Type, assoc: &str, depth: usize) -> Type {
        if depth > MAX_PROJ_DEPTH {
            return Type::Error;
        }
        for (head, targs, _pnames, aname, aty) in &self.assoc_impls {
            if aname != assoc {
                continue;
            }
            let mut map: HashMap<String, Type> = HashMap::new();
            let matched = match c {
                Type::Named(n) => n == head && targs.is_empty(),
                Type::App(n, args) => {
                    n == head
                        && targs.len() == args.len()
                        && targs.iter().zip(args).all(|(d, a)| unify_inst(d, a, &mut map))
                }
                _ => false,
            };
            if matched {
                return self.reduce_projs(&subst(aty, &map), &map, depth);
            }
        }
        Type::Error
    }

    /// Reduce residual associated-type projections in `t` to their concrete leaf
    /// (design 0009 §2.2, transitive). An impl whose `type Item = I::Item` leaves
    /// a residual `I::Item` after `subst` (its base maps to a concrete `App`, not
    /// a `Param`); concretize the base through the impl's `map` and resolve again,
    /// so an adapter-over-adapter chain reaches the underlying leaf item. A base
    /// that stays opaque is left symbolic; a residual that fails to resolve keeps
    /// its projection (unchanged one-hop behavior). `depth` bounds the recursion
    /// so a malformed cyclic binding terminates instead of looping.
    fn reduce_projs(&self, t: &Type, map: &HashMap<String, Type>, depth: usize) -> Type {
        match t {
            Type::Proj(b, a) => match map.get(b) {
                Some(c) if !matches!(c, Type::Param(_) | Type::Error) => {
                    let c = c.clone();
                    let r = self.resolve_proj_depth(&c, a, depth + 1);
                    if matches!(r, Type::Error) {
                        t.clone()
                    } else {
                        r
                    }
                }
                _ => t.clone(),
            },
            Type::App(n, args) => Type::App(
                n.clone(),
                args.iter().map(|x| self.reduce_projs(x, map, depth)).collect(),
            ),
            Type::Array(e, l) => Type::Array(Box::new(self.reduce_projs(e, map, depth)), l.clone()),
            Type::Slice(e) => Type::Slice(Box::new(self.reduce_projs(e, map, depth))),
            Type::SliceMut(e) => Type::SliceMut(Box::new(self.reduce_projs(e, map, depth))),
            Type::RawPtr(e) => Type::RawPtr(Box::new(self.reduce_projs(e, map, depth))),
            Type::Box(e) => Type::Box(Box::new(self.reduce_projs(e, map, depth))),
            Type::BoxResult(e) => Type::BoxResult(Box::new(self.reduce_projs(e, map, depth))),
            Type::Borrow(e) => Type::Borrow(Box::new(self.reduce_projs(e, map, depth))),
            Type::BorrowMut(e) => Type::BorrowMut(Box::new(self.reduce_projs(e, map, depth))),
            Type::FnPtr(f) => Type::FnPtr(crate::types::FnPtrTy {
                foreign: f.foreign,
                params: f
                    .params
                    .iter()
                    .map(|(mode, t)| (*mode, self.reduce_projs(t, map, depth)))
                    .collect(),
                alloc: f.alloc,
                ret: Box::new(self.reduce_projs(&f.ret, map, depth)),
            }),
            _ => t.clone(),
        }
    }

    /// Inject the resolution of every associated-type projection `Param::Assoc`
    /// whose `Param` is now bound to a concrete type in `map` (design 0009 §2.2).
    /// A shape argument recorded at a generic body node can carry a projection
    /// over a bounded type parameter (e.g. `IterStep[I::Item, ..]`); `subst`
    /// leaves such a projection opaque because its base substitutes to a concrete
    /// `App`, not another `Param`. Seeding the fully-keyed `"Param::Assoc"` entry
    /// lets `subst` normalize it to the impl's binding, so the instance name and
    /// every engine agree — the same normalization the checker performs for
    /// `Self::Item` (`check::generics::iface_method_subst`), generalized to every
    /// bounded parameter at instantiation.
    fn inject_assoc_bindings(&self, pnames: &[String], map: &mut HashMap<String, Type>) {
        let mut assoc_names: Vec<&str> = self.assoc_impls.iter().map(|a| a.3.as_str()).collect();
        assoc_names.sort_unstable();
        assoc_names.dedup();
        let concretes: Vec<(String, Type)> = pnames
            .iter()
            .filter_map(|p| map.get(p).map(|c| (p.clone(), c.clone())))
            .filter(|(_, c)| !matches!(c, Type::Param(_) | Type::Error))
            .collect();
        for (pname, c) in concretes {
            for assoc in &assoc_names {
                let key = format!("{pname}::{assoc}");
                if map.contains_key(&key) {
                    continue;
                }
                let resolved = self.resolve_proj(&c, assoc);
                if !matches!(resolved, Type::Error) {
                    map.insert(key, resolved);
                }
            }
        }
    }

    /// Semantic concrete `Type` -> AST `TyKind`, lowering generic apps to nominals.
    fn type_to_ast_kind(&mut self, t: &Type) -> TyKind {
        match t {
            Type::Scalar(s) => TyKind::Scalar(*s),
            Type::Named(n) => TyKind::Named(n.clone()),
            Type::App(n, args) if n == "Vec" || n == "Map" => {
                TyKind::App { name: n.clone(), args: args.iter().map(|a| self.type_to_ast(a)).collect() }
            }
            Type::App(n, args) => {
                self.enqueue(n, args.clone());
                TyKind::Named(inst_type_name(n, args))
            }
            Type::Array(e, _) => TyKind::Array {
                size: Box::new(Expr { kind: ExprKind::IntLit { value: 0, suffix: None }, span: Span::point(0) }),
                elem: Box::new(self.type_to_ast(e)),
            },
            Type::Slice(e) => TyKind::Slice(Box::new(self.type_to_ast(e))),
            Type::SliceMut(e) => TyKind::SliceMut(Box::new(self.type_to_ast(e))),
            Type::RawPtr(e) => TyKind::RawPtr(Box::new(self.type_to_ast(e))),
            Type::Box(e) => TyKind::Box(Box::new(self.type_to_ast(e))),
            Type::BoxResult(e) => TyKind::BoxResult(Box::new(self.type_to_ast(e))),
            Type::Borrow(e) => TyKind::Borrow(Box::new(self.type_to_ast(e))),
            Type::BorrowMut(e) => TyKind::BorrowMut(Box::new(self.type_to_ast(e))),
            _ => TyKind::Named("unit".to_string()),
        }
    }

    fn type_to_ast(&mut self, t: &Type) -> Ty {
        Ty { kind: self.type_to_ast_kind(t), span: Span::point(0) }
    }

    fn rewrite_expr(&mut self, e: &mut Expr, map: &HashMap<String, Type>) {
        // Apply a recorded monomorphization shape at this node, if any.
        if let Some(shape) = self.shapes.get(&(self.cur_item, e.span.start)).cloned() {
            match shape {
                Shape::Fn(name, sargs) => {
                    let cargs: Vec<Type> = sargs.iter().map(|t| subst(t, map)).collect();
                    self.enqueue(&name, cargs.clone());
                    let inst = inst_fn_name(&name, &cargs);
                    match &mut e.kind {
                        ExprKind::Call { callee, .. } => {
                            if let ExprKind::Ident(id) = &mut callee.kind {
                                *id = inst;
                            }
                        }
                        ExprKind::GenericVal { .. } => {
                            e.kind = ExprKind::Ident(inst);
                        }
                        _ => {}
                    }
                }
                Shape::Type(name, sargs) => {
                    let cargs: Vec<Type> = sargs.iter().map(|t| subst(t, map)).collect();
                    self.enqueue(&name, cargs.clone());
                    let inst = inst_type_name(&name, &cargs);
                    match &mut e.kind {
                        ExprKind::EnumCtor { enum_name, .. } => *enum_name = inst,
                        ExprKind::StructLit { name: n, .. } => *n = inst,
                        _ => {}
                    }
                }
                // The interface is a name, invariant under substitution: stamp it
                // onto the method call's `Field` callee so every instance dispatches
                // exactly the interface the checker resolved.
                Shape::Method(iface) => {
                    if let ExprKind::Call { callee, .. } = &mut e.kind {
                        if let ExprKind::Field { iface: slot, .. } = &mut callee.kind {
                            *slot = Some(iface);
                        }
                    }
                }
            }
        }
        // Recurse into children.
        match &mut e.kind {
            ExprKind::Unary { expr, .. }
            | ExprKind::Prefix { expr, .. }
            | ExprKind::OutArg(expr)
            | ExprKind::Paren(expr)
            | ExprKind::Try(expr)
            | ExprKind::Assert(expr)
            | ExprKind::Panic(expr) => self.rewrite_expr(expr, map),
            ExprKind::Binary { lhs, rhs, .. } => {
                self.rewrite_expr(lhs, map);
                self.rewrite_expr(rhs, map);
            }
            ExprKind::Call { callee, args } => {
                self.rewrite_expr(callee, map);
                for a in args {
                    self.rewrite_expr(a, map);
                }
            }
            ExprKind::Field { base, .. } => self.rewrite_expr(base, map),
            ExprKind::Index { base, index } => {
                self.rewrite_expr(base, map);
                self.rewrite_expr(index, map);
            }
            ExprKind::Conv { ty, expr } => {
                *ty = self.rewrite_ty(ty, map);
                self.rewrite_expr(expr, map);
            }
            ExprKind::Bitcast { ty, expr } => {
                *ty = self.rewrite_ty(ty, map);
                self.rewrite_expr(expr, map);
            }
            ExprKind::CastPtr { ty, arg } | ExprKind::AddrToPtr { ty, arg } => {
                *ty = self.rewrite_ty(ty, map);
                self.rewrite_expr(arg, map);
            }
            ExprKind::PtrNull { ty } | ExprKind::Sizeof(ty) | ExprKind::Alignof(ty) | ExprKind::Offsetof { ty, .. } => {
                *ty = self.rewrite_ty(ty, map);
            }
            ExprKind::FieldPtr { ptr, .. } => self.rewrite_expr(ptr, map),
            ExprKind::ArrayLit(v) => {
                for x in v {
                    self.rewrite_expr(x, map);
                }
            }
            ExprKind::ArrayRepeat { value, size } => {
                self.rewrite_expr(value, map);
                self.rewrite_expr(size, map);
            }
            ExprKind::StructLit { fields, .. } => {
                for f in fields {
                    self.rewrite_expr(&mut f.value, map);
                }
            }
            ExprKind::EnumCtor { args, .. } => {
                for a in args {
                    self.rewrite_expr(a, map);
                }
            }
            ExprKind::GenericVal { ty_args, .. } => {
                for t in ty_args {
                    *t = self.rewrite_ty(t, map);
                }
            }
            ExprKind::Block(b) | ExprKind::Loop(b) | ExprKind::Wrapping(b) | ExprKind::Saturating(b) => {
                self.rewrite_block(b, map)
            }
            ExprKind::Unsafe { body, .. } => self.rewrite_block(body, map),
            ExprKind::If { cond, then_blk, else_blk } => {
                self.rewrite_expr(cond, map);
                self.rewrite_block(then_blk, map);
                if let Some(x) = else_blk {
                    self.rewrite_expr(x, map);
                }
            }
            ExprKind::While { cond, body } => {
                self.rewrite_expr(cond, map);
                self.rewrite_block(body, map);
            }
            ExprKind::Match { scrutinee, arms } => {
                self.rewrite_expr(scrutinee, map);
                for arm in arms {
                    self.rewrite_pattern(&mut arm.pattern, map);
                    self.rewrite_expr(&mut arm.body, map);
                }
            }
            ExprKind::Return(Some(x)) => self.rewrite_expr(x, map),
            _ => {}
        }
    }

    /// Lower a match pattern's generic enum name to the concrete instance, using
    /// the shape recorded at the pattern's span (design 0007 §5).
    fn rewrite_pattern(&mut self, pat: &mut Pattern, map: &HashMap<String, Type>) {
        if let Some(Shape::Type(name, sargs)) = self.shapes.get(&(self.cur_item, pat.span.start)).cloned() {
            let cargs: Vec<Type> = sargs.iter().map(|t| subst(t, map)).collect();
            self.enqueue(&name, cargs.clone());
            let inst = inst_type_name(&name, &cargs);
            if let PatKind::Variant { enum_name, .. } = &mut pat.kind {
                *enum_name = inst;
            }
        }
        if let PatKind::Variant { sub, .. } = &mut pat.kind {
            for s in sub {
                self.rewrite_pattern(s, map);
            }
        }
    }
}

/// Replace every `Self` type in an impl method with the target type expression
/// (a generic impl's `Self` is `List[T]`, an application, not a bare nominal).
pub fn subst_self_ty(f: &mut FnDecl, target: &Ty) {
    fn fix(ty: &mut Ty, target: &Ty) {
        match &mut ty.kind {
            TyKind::Named(n) if n == "Self" => *ty = target.clone(),
            TyKind::App { args, .. } => for a in args { fix(a, target); },
            TyKind::Array { elem, .. } => fix(elem, target),
            TyKind::Slice(e) | TyKind::SliceMut(e) | TyKind::RawPtr(e) | TyKind::Box(e)
            | TyKind::BoxResult(e) | TyKind::Borrow(e) | TyKind::BorrowMut(e) => fix(e, target),
            TyKind::FnPtr(fp) => { for p in &mut fp.params { fix(&mut p.ty, target); } fix(&mut fp.ret, target); }
            _ => {}
        }
    }
    for p in &mut f.params { fix(&mut p.ty, target); }
    if let Some(rt) = &mut f.ret { fix(&mut rt.ty, target); }
}

/// Replace every `Self` type in an impl method with its concrete target nominal.
pub fn subst_self_fndecl(f: &mut FnDecl, target: &str) {
    fn fix(ty: &mut Ty, target: &str) {
        match &mut ty.kind {
            TyKind::Named(n) if n == "Self" => *n = target.to_string(),
            TyKind::App { args, .. } => for a in args { fix(a, target); },
            TyKind::Array { elem, .. } => fix(elem, target),
            TyKind::Slice(e) | TyKind::SliceMut(e) | TyKind::RawPtr(e) | TyKind::Box(e)
            | TyKind::BoxResult(e) | TyKind::Borrow(e) | TyKind::BorrowMut(e) => fix(e, target),
            TyKind::FnPtr(fp) => { for p in &mut fp.params { fix(&mut p.ty, target); } fix(&mut fp.ret, target); }
            _ => {}
        }
    }
    for p in &mut f.params { fix(&mut p.ty, target); }
    if let Some(rt) = &mut f.ret { fix(&mut rt.ty, target); }
}

/// Unify a parametric type `decl` (mentioning impl type parameters) against a
/// concrete type `arg`, binding parameters in `out`. Used to drive app-target impl
/// instantiation from a reached target-type instance (design 0007 stage 2).
/// Convert a concrete semantic type to an AST type *preserving* generic
/// applications (`App` stays `App`), so the interpreter's `resolve_impl_ty` maps
/// it back to the same semantic type and mangles impl-method names identically.
fn sem_to_ast_preserve(t: &Type) -> Ty {
    let kind = match t {
        Type::Scalar(s) => TyKind::Scalar(*s),
        Type::Named(n) => TyKind::Named(n.clone()),
        Type::App(n, args) => TyKind::App { name: n.clone(), args: args.iter().map(sem_to_ast_preserve).collect() },
        Type::Box(e) => TyKind::Box(Box::new(sem_to_ast_preserve(e))),
        Type::BoxResult(e) => TyKind::BoxResult(Box::new(sem_to_ast_preserve(e))),
        Type::RawPtr(e) => TyKind::RawPtr(Box::new(sem_to_ast_preserve(e))),
        Type::Slice(e) => TyKind::Slice(Box::new(sem_to_ast_preserve(e))),
        Type::SliceMut(e) => TyKind::SliceMut(Box::new(sem_to_ast_preserve(e))),
        _ => TyKind::Named("unit".to_string()),
    };
    Ty { kind, span: Span::point(0) }
}

pub(crate) fn unify_inst(decl: &Type, arg: &Type, out: &mut HashMap<String, Type>) -> bool {
    match (decl, arg) {
        (Type::Param(n), a) => {
            match out.get(n) {
                Some(prev) => prev == a,
                None => { out.insert(n.clone(), a.clone()); true }
            }
        }
        (Type::Scalar(a), Type::Scalar(b)) => a == b,
        (Type::Named(a), Type::Named(b)) => a == b,
        (Type::App(a, aa), Type::App(b, bb)) => {
            a == b && aa.len() == bb.len() && aa.iter().zip(bb).all(|(x, y)| unify_inst(x, y, out))
        }
        (Type::Box(a), Type::Box(b))
        | (Type::BoxResult(a), Type::BoxResult(b))
        | (Type::RawPtr(a), Type::RawPtr(b))
        | (Type::Slice(a), Type::Slice(b))
        | (Type::SliceMut(a), Type::SliceMut(b))
        | (Type::Borrow(a), Type::Borrow(b))
        | (Type::BorrowMut(a), Type::BorrowMut(b))
        | (Type::Array(a, _), Type::Array(b, _)) => unify_inst(a, b, out),
        _ => false,
    }
}


// ---------------------------------------------------------------------------
// Impl/interface method-signature conformance (design 0007 §3.5, §4.1)
// ---------------------------------------------------------------------------

/// Check every interface method present in `im` for signature conformance with
/// its interface declaration (design 0007 §3.5, §4.1). Divergence on any axis —
/// self receiver, parameter count/mode/type, return type, or effect marker —
/// is a definition-site error naming both signatures. Missing/extra methods are
/// E1015/E1014 (reported by the caller); this only checks present-on-both ones.
#[allow(clippy::too_many_arguments)]
fn check_impl_conformance(
    im: &ImplDecl,
    iface_info: &IfaceInfo,
    iface_args: &[Type],
    target: &str,
    target_args: &[Type],
    pset: &HashSet<String>,
    known: &HashSet<String>,
    gen: &HashSet<String>,
    diags: &mut Vec<Diag>,
) {
    let target_ty = if target_args.is_empty() {
        Type::Named(target.to_string())
    } else {
        Type::App(target.to_string(), target_args.to_vec())
    };
    // Interface-side substitution: Self -> target, iface params -> iface args,
    // and (if any) `Self::Assoc` -> the impl's associated-type binding.
    let mut smap: HashMap<String, Type> = HashMap::new();
    smap.insert("Self".to_string(), target_ty.clone());
    for (pname, arg) in iface_info.type_params.iter().zip(iface_args) {
        smap.insert(pname.clone(), arg.clone());
    }
    if let (Some(aname), Some((_, bty))) = (&iface_info.assoc_type, &im.assoc_binding) {
        let mut scratch = Vec::new();
        let resolved = resolve_gty(bty, pset, known, gen, &mut scratch);
        let mut m2 = HashMap::new();
        m2.insert("Self".to_string(), target_ty.clone());
        smap.insert(format!("Self::{aname}"), subst(&resolved, &m2));
    }
    // Impl-side substitution: `Self` in a written method type -> the target.
    let mut impl_smap: HashMap<String, Type> = HashMap::new();
    impl_smap.insert("Self".to_string(), target_ty.clone());

    for want in &iface_info.methods {
        let m = match im.methods.iter().find(|m| m.name == want.name) {
            Some(m) => m,
            None => continue, // missing method: E1015 already reported
        };
        let impl_has_self = m.params.first().is_some_and(|p| p.name == "self");
        let impl_self_mode = if impl_has_self { m.params[0].mode } else { ParamMode::Take };
        let impl_nonself: Vec<&Param> = if impl_has_self {
            m.params.iter().skip(1).collect()
        } else {
            m.params.iter().collect()
        };

        // Both signatures rendered in a common (substituted, un-lowered) form so
        // the diagnostics can name each side (P4).
        let exp_params: Vec<(ParamMode, Type)> = want
            .params
            .iter()
            .map(|(md, t)| (*md, unlower(*md, subst(t, &smap))))
            .collect();
        let exp_ret = subst(&want.ret, &smap);
        let mut scratch = Vec::new();
        let impl_params_ty: Vec<(ParamMode, Type)> = impl_nonself
            .iter()
            .map(|p| (p.mode, subst(&resolve_gty(&p.ty, pset, known, gen, &mut scratch), &impl_smap)))
            .collect();
        let impl_ret = match &m.ret {
            Some(rt) => {
                let base = resolve_gty(&rt.ty, pset, known, gen, &mut scratch);
                let wrapped = match rt.borrow {
                    Some(BorrowKind::Shared) => Type::Borrow(Box::new(base)),
                    Some(BorrowKind::Exclusive) => Type::BorrowMut(Box::new(base)),
                    None => base,
                };
                subst(&wrapped, &impl_smap)
            }
            None => Type::unit(),
        };
        let exp_sig = fmt_iface_sig(&want.name, want.has_self, want.self_mode, &exp_params, want.alloc, &exp_ret);
        let got_sig = fmt_iface_sig(&want.name, impl_has_self, impl_self_mode, &impl_params_ty, m.alloc, &impl_ret);
        let both = |d: Diag| {
            d.with_note(format!("interface declares: {exp_sig}"), None)
                .with_note(format!("this impl has:      {got_sig}"), None)
        };

        // Axis 1 — the `self` receiver (presence, then mode).
        if impl_has_self != want.has_self {
            diags.push(both(Diag::error(
                "E1021",
                format!(
                    "impl method `{}` of `{}` for `{}` {} a `self` receiver, but the interface {}",
                    want.name, im.iface, target,
                    if impl_has_self { "takes" } else { "omits" },
                    if want.has_self { "declares one" } else { "declares none" },
                ),
                m.span,
            )));
            continue;
        }
        if impl_has_self && impl_self_mode != want.self_mode {
            diags.push(both(Diag::error(
                "E1021",
                format!(
                    "impl method `{}` of `{}` for `{}` takes `{} self`, but the interface declares `{} self`",
                    want.name, im.iface, target, mode_kw(impl_self_mode), mode_kw(want.self_mode),
                ),
                m.span,
            )));
        }
        // Axis 2 — parameter count (skip per-parameter checks if it diverges).
        if impl_nonself.len() != want.params.len() {
            diags.push(both(Diag::error(
                "E1022",
                format!(
                    "impl method `{}` of `{}` for `{}` takes {} parameter(s), but the interface declares {}",
                    want.name, im.iface, target, impl_nonself.len(), want.params.len(),
                ),
                m.span,
            )));
            continue;
        }
        // Axes 3 & 4 — per-parameter mode and type.
        for (i, ((emode, ety), (amode, aty))) in exp_params.iter().zip(&impl_params_ty).enumerate() {
            if emode != amode {
                diags.push(both(Diag::error(
                    "E1023",
                    format!(
                        "parameter {} of impl method `{}` of `{}` for `{}` has mode `{}`, but the interface declares `{}`",
                        i + 1, want.name, im.iface, target, mode_kw(*amode), mode_kw(*emode),
                    ),
                    m.span,
                )));
            }
            if type_comparable(ety) && type_comparable(aty) && ety != aty {
                diags.push(both(Diag::error(
                    "E1024",
                    format!(
                        "parameter {} of impl method `{}` of `{}` for `{}` has type `{}`, but the interface declares `{}`",
                        i + 1, want.name, im.iface, target, aty.display(), ety.display(),
                    ),
                    m.span,
                )));
            }
        }
        // Axis 5 — return type.
        if type_comparable(&exp_ret) && type_comparable(&impl_ret) && exp_ret != impl_ret {
            diags.push(both(Diag::error(
                "E1025",
                format!(
                    "impl method `{}` of `{}` for `{}` returns `{}`, but the interface declares `{}`",
                    want.name, im.iface, target, impl_ret.display(), exp_ret.display(),
                ),
                m.span,
            )));
        }
        // Axis 6 — effect marker (uniform across impls, exact — §4.1).
        if m.alloc != want.alloc {
            diags.push(both(Diag::error(
                "E1026",
                format!(
                    "impl method `{}` of `{}` for `{}` is {}, but the interface method is {}",
                    want.name, im.iface, target,
                    if m.alloc { "`alloc`" } else { "not `alloc`" },
                    if want.alloc { "`alloc`" } else { "not `alloc`" },
                ),
                m.span,
            ).with_note("an interface method's effect marker is uniform across impls — exact match required (design 0007 §4.1)", None)));
        }
    }
}

/// Strip the borrow wrapper a `read`/`write` parameter's lowered type carries, to
/// recover its written (underlying) type for conformance comparison.
fn unlower(mode: ParamMode, t: Type) -> Type {
    match mode {
        ParamMode::Read => match t { Type::Borrow(i) => *i, other => other },
        ParamMode::Write => match t { Type::BorrowMut(i) => *i, other => other },
        _ => t,
    }
}

fn mode_kw(m: ParamMode) -> &'static str {
    match m {
        ParamMode::Read => "read",
        ParamMode::Write => "write",
        ParamMode::Take => "take",
        ParamMode::Out => "out",
    }
}

/// A type is comparable for conformance only when it has no unresolved projection
/// or error node — an unresolved `Self::Assoc` means the impl's associated-type
/// binding itself errored (E1018), so a cascading signature mismatch is suppressed.
fn type_comparable(t: &Type) -> bool {
    match t {
        Type::Proj(_, _) | Type::Error => false,
        Type::App(_, a) => a.iter().all(type_comparable),
        Type::Array(e, _) | Type::Slice(e) | Type::SliceMut(e) | Type::RawPtr(e)
        | Type::Box(e) | Type::BoxResult(e) | Type::Borrow(e) | Type::BorrowMut(e) => type_comparable(e),
        Type::FnPtr(f) => f.params.iter().all(|(_, t)| type_comparable(t)) && type_comparable(&f.ret),
        _ => true,
    }
}

/// Render a method signature from its resolved pieces for a P4 diagnostic.
fn fmt_iface_sig(
    name: &str,
    has_self: bool,
    self_mode: ParamMode,
    params: &[(ParamMode, Type)],
    alloc: bool,
    ret: &Type,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    if has_self {
        parts.push(format!("{} self", mode_kw(self_mode)));
    }
    for (m, t) in params {
        let pfx = match m {
            ParamMode::Take => String::new(),
            other => format!("{} ", mode_kw(*other)),
        };
        parts.push(format!("{pfx}{}", t.display()));
    }
    format!("fn {name}({}){} -> {}", parts.join(", "), if alloc { " alloc" } else { "" }, ret.display())
}
