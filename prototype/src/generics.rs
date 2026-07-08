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

/// Documented monomorphization depth limit (design 0007 §5.1.1): a deterministic
/// resource backstop for the undecidable instantiation tail.
pub const MONO_DEPTH_LIMIT: usize = 64;

/// A monomorphization shape recorded at an expression node during checking: the
/// generic name and the (possibly still-parametric) type arguments to substitute.
#[derive(Clone, Debug)]
pub enum Shape {
    /// A generic function call or `name::[T]` value -> a function instance.
    Fn(String, Vec<Type>),
    /// A generic struct literal or enum constructor -> a type instance.
    Type(String, Vec<Type>),
}

/// Does the program contain any generic construct (design 0007)?
pub fn is_generic_program(prog: &Program) -> bool {
    prog.items.iter().any(|it| match it {
        Item::Interface(_) | Item::Impl(_) => true,
        Item::Struct(s) => !s.type_params.is_empty(),
        Item::Enum(e) => !e.type_params.is_empty(),
        Item::Fn(f) => !f.type_params.is_empty(),
        Item::Static(_) => false,
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
        TyKind::App { name, args } => {
            let ra: Vec<Type> = args
                .iter()
                .map(|a| resolve_gty(a, params, known_types, generic_types, diags))
                .collect();
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
            let pset: HashSet<String> = idecl.type_params.iter().map(|p| p.name.clone()).collect();
            let methods = idecl
                .methods
                .iter()
                .map(|m| {
                    let params = m
                        .params
                        .iter()
                        .map(|p| {
                            let t = resolve_gty(&p.ty, &pset, &known_types, &generic_types, diags);
                            (p.mode, lower_param(p.mode, t))
                        })
                        .collect();
                    let ret = match &m.ret {
                        Some(rt) => ret_type(rt, &pset, &known_types, &generic_types, diags),
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
                if s.drop_hook.is_some() {
                    diags.push(Diag::error(
                        "E1011",
                        format!("generic struct `{}` may not declare a `drop` hook in stage 1", s.name),
                        s.span,
                    ));
                }
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
                        has_drop: false,
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
            let params = f
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
            items.generic_fns.insert(
                f.name.clone(),
                GenericFnSig {
                    name: f.name.clone(),
                    type_params: f.type_params.iter().map(|p| (p.name.clone(), p.bounds.clone())).collect(),
                    regions: f.regions.clone(),
                    params,
                    alloc: f.alloc,
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
    no_params: &HashSet<String>,
    diags: &mut Vec<Diag>,
) {
    // Generic impls are deferred (stage 1).
    if !im.type_params.is_empty() || matches!(&im.target.kind, TyKind::App { .. }) {
        diags.push(
            Diag::error(
                "E1010",
                "generic `impl` blocks are deferred in stage 1".to_string(),
                im.span,
            )
            .with_note("only `impl I[concrete] for ConcreteType` is supported this stage (design 0007)", None),
        );
        return;
    }
    let target = match &im.target.kind {
        TyKind::Named(n) => n.clone(),
        _ => {
            diags.push(Diag::error("E1012", "an impl target must be a nominal type".to_string(), im.target.span));
            return;
        }
    };
    if !known.contains(&target) {
        diags.push(Diag::error("E0102", format!("unknown type `{target}`"), im.target.span));
        return;
    }
    let iface_args: Vec<Type> = im
        .iface_args
        .iter()
        .map(|a| resolve_gty(a, no_params, known, gen, diags))
        .collect();
    let iface_info = match items.interfaces.get(&im.iface) {
        Some(i) => i.clone(),
        None => {
            diags.push(Diag::error("E1003", format!("unknown interface `{}`", im.iface), im.span));
            return;
        }
    };
    // Coherence: at most one impl per (I[args], T) key (design 0007 §2.3).
    if items
        .impls
        .iter()
        .any(|e| e.iface == im.iface && e.iface_args == iface_args && e.target == target)
    {
        diags.push(
            Diag::error(
                "E1009",
                format!("duplicate impl of `{}` for `{}`", im.iface, target),
                im.span,
            )
            .with_note("at most one impl of a given instantiated interface for a given type (§2.3)", None),
        );
        return;
    }
    // Orphan rule at module granularity (design 0007 §2.3, 0008): the impl must
    // live in the target's module or the interface's declaration module. In a
    // single-file program `home` is absent and every module is "", so it is
    // trivially satisfied.
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
    // Verify method set matches the interface's; record mangled free-fn names.
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
    items.impls.push(ImplInfo {
        iface: im.iface.clone(),
        iface_args,
        target,
        methods,
        span: im.span,
    });
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
        Type::SliceMut(e) => format!("slicemut_{}", mangle_ty(e)),
        Type::RawPtr(e) => format!("ptr_{}", mangle_ty(e)),
        Type::Box(e) => format!("Box_{}", mangle_ty(e)),
        Type::BoxResult(e) => format!("BoxResult_{}", mangle_ty(e)),
        Type::Borrow(e) => format!("ref_{}", mangle_ty(e)),
        Type::BorrowMut(e) => format!("refmut_{}", mangle_ty(e)),
        Type::FnPtr(_) => "fnptr".to_string(),
        Type::Param(n) => n.clone(),
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
    shapes: &'a HashMap<usize, Shape>,
    fn_done: HashSet<String>,
    type_done: HashSet<String>,
    fn_work: Vec<(String, Vec<Type>)>,
    type_work: Vec<(String, Vec<Type>)>,
    out: Vec<Item>,
    diags: Vec<Diag>,
    depth: usize,
    /// The original generic decls, keyed by name.
    generic_fns_ast: HashMap<String, FnDecl>,
    generic_structs_ast: HashMap<String, StructDecl>,
    generic_enums_ast: HashMap<String, EnumDecl>,
}

/// Monomorphize `prog` into a concrete program using the checker-recorded shapes
/// and reached instantiations.
pub fn monomorphize(
    prog: &Program,
    insts: &[(String, Vec<Type>)],
    shapes: &HashMap<usize, Shape>,
) -> Mono {
    let mut generic_fns_ast = HashMap::new();
    let mut generic_structs_ast = HashMap::new();
    let mut generic_enums_ast = HashMap::new();
    for it in &prog.items {
        match it {
            Item::Fn(f) if !f.type_params.is_empty() => {
                generic_fns_ast.insert(f.name.clone(), f.clone());
            }
            Item::Struct(s) if !s.type_params.is_empty() => {
                generic_structs_ast.insert(s.name.clone(), s.clone());
            }
            Item::Enum(e) if !e.type_params.is_empty() => {
                generic_enums_ast.insert(e.name.clone(), e.clone());
            }
            _ => {}
        }
    }
    let mut m = Monomorphizer {
        shapes,
        fn_done: HashSet::new(),
        type_done: HashSet::new(),
        fn_work: Vec::new(),
        type_work: Vec::new(),
        out: Vec::new(),
        diags: Vec::new(),
        depth: 0,
        generic_fns_ast,
        generic_structs_ast,
        generic_enums_ast,
    };

    // Seed from checker-reached instantiations.
    for (name, args) in insts {
        m.enqueue(name, args.clone());
    }

    // Rewrite and emit the concrete (non-generic) items.
    let empty: HashMap<String, Type> = HashMap::new();
    for it in &prog.items {
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
            Item::Impl(im) => m.emit_impl(im),
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
                self.fn_work.push((name.to_string(), args));
            }
        } else if self.generic_structs_ast.contains_key(name) || self.generic_enums_ast.contains_key(name) {
            let key = inst_type_name(name, &args);
            if self.type_done.insert(key) {
                self.type_work.push((name.to_string(), args));
            }
        }
    }

    fn drive(&mut self) {
        while !self.fn_work.is_empty() || !self.type_work.is_empty() {
            self.depth += 1;
            if self.depth > MONO_DEPTH_LIMIT {
                self.diags.push(
                    Diag::error(
                        "E1099",
                        format!("monomorphization exceeded the depth limit ({MONO_DEPTH_LIMIT})"),
                        Span::point(0),
                    )
                    .with_note("this is a compile resource limit, not a type error (design 0007 §5.1.1)", None),
                );
                return;
            }
            if let Some((name, args)) = self.type_work.pop() {
                self.emit_type_instance(&name, &args);
            } else if let Some((name, args)) = self.fn_work.pop() {
                self.emit_fn_instance(&name, &args);
            }
        }
    }

    fn param_map(names: &[String], args: &[Type]) -> HashMap<String, Type> {
        names.iter().cloned().zip(args.iter().cloned()).collect()
    }

    fn emit_fn_instance(&mut self, name: &str, args: &[Type]) {
        let decl = self.generic_fns_ast.get(name).unwrap().clone();
        let pnames: Vec<String> = decl.type_params.iter().map(|p| p.name.clone()).collect();
        let map = Self::param_map(&pnames, args);
        let mut f2 = decl;
        f2.name = inst_fn_name(name, args);
        f2.type_params = Vec::new();
        self.rewrite_fn(&mut f2, &map);
        self.out.push(Item::Fn(f2));
    }

    fn emit_type_instance(&mut self, name: &str, args: &[Type]) {
        if let Some(s) = self.generic_structs_ast.get(name).cloned() {
            let pnames: Vec<String> = s.type_params.iter().map(|p| p.name.clone()).collect();
            let map = Self::param_map(&pnames, args);
            let mut s2 = s;
            s2.name = inst_type_name(name, args);
            s2.type_params = Vec::new();
            for fld in &mut s2.fields {
                fld.ty = self.rewrite_ty(&fld.ty, &map);
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
        let target = match &im.target.kind {
            TyKind::Named(n) => n.clone(),
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
            subst_self_fndecl(&mut fm, &target);
            self.rewrite_fn(&mut fm, &empty);
            self.out.push(Item::Fn(fm));
            // Keep a signature-only stub in the impl block so the interpreter can
            // recover the method's name, self mode, and mangled target.
            let mut stub = m.clone();
            subst_self_fndecl(&mut stub, &target);
            stub.body = Block { stmts: Vec::new(), span: m.span };
            kept.methods.push(stub);
        }
        kept.target = Ty { kind: TyKind::Named(target.clone()), span: im.target.span };
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
        };
        Ty { kind, span: ty.span }
    }

    /// Semantic concrete `Type` -> AST `TyKind`, lowering generic apps to nominals.
    fn type_to_ast_kind(&mut self, t: &Type) -> TyKind {
        match t {
            Type::Scalar(s) => TyKind::Scalar(*s),
            Type::Named(n) => TyKind::Named(n.clone()),
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
        if let Some(shape) = self.shapes.get(&e.span.start).cloned() {
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
        if let Some(Shape::Type(name, sargs)) = self.shapes.get(&pat.span.start).cloned() {
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
