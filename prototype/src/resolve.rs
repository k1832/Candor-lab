//! Item resolution (design 0001 §8.2): builds the program's item table
//! (structs, enums + variant lookup, fns, statics), detects duplicates,
//! resolves type names, and runs type well-formedness (borrow-typed fields
//! banned §3.4; copy-marker validity §1.3; modes on borrow-kind params §3.1).
//! Single-file programs, no modules.

use std::collections::HashMap;
use std::collections::HashSet;

use crate::ast::*;
use crate::diag::Diag;
use crate::span::Span;
use crate::types::*;

/// A resolved parameter: declared pointee/value type plus its *lowered* value
/// type as seen inside the callee (design 0001 §3.1).
#[derive(Clone, Debug)]
pub struct ParamInfo {
    pub name: String,
    pub mode: ParamMode,
    /// Region tag on a borrow parameter (design 0001 §3.3), e.g. `r` in `read[r] T`.
    pub region: Option<String>,
    pub decl_ty: Type,
    pub lowered: Type,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FnSig {
    pub name: String,
    pub regions: Vec<String>,
    pub params: Vec<ParamInfo>,
    pub alloc: bool,
    pub ret: Type,
    /// Region tag on a borrow return (design 0001 §3.3), if written.
    pub ret_region: Option<String>,
    pub ret_span: Span,
    pub span: Span,
}

/// The resolved program item table.
#[derive(Default)]
pub struct Items {
    pub structs: HashMap<String, StructTy>,
    pub enums: HashMap<String, EnumTy>,
    pub fns: HashMap<String, FnSig>,
    pub statics: HashMap<String, (Type, Span)>,
}

impl ItemEnv for Items {
    fn lookup_struct(&self, name: &str) -> Option<&StructTy> {
        self.structs.get(name)
    }
    fn lookup_enum(&self, name: &str) -> Option<&EnumTy> {
        self.enums.get(name)
    }
}

/// Lower a parameter's declared type to the value type the callee binds
/// (design 0001 §3.1): read -> shared borrow, write -> exclusive borrow.
pub fn lower_param(mode: ParamMode, ty: Type) -> Type {
    match mode {
        ParamMode::Take | ParamMode::Out => ty,
        ParamMode::Read => Type::Borrow(Box::new(ty)),
        ParamMode::Write => Type::BorrowMut(Box::new(ty)),
    }
}

struct Resolver<'a> {
    type_names: &'a HashSet<String>,
    diags: &'a mut Vec<Diag>,
}

impl<'a> Resolver<'a> {
    fn resolve_ty(&mut self, ty: &Ty) -> Type {
        match &ty.kind {
            TyKind::Scalar(s) => Type::Scalar(*s),
            TyKind::Named(n) => {
                if self.type_names.contains(n) {
                    Type::Named(n.clone())
                } else {
                    self.diags.push(
                        Diag::error("E0102", format!("unknown type `{n}`"), ty.span)
                            .with_note("no struct or enum with this name is declared", None),
                    );
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
                let mut params = Vec::new();
                for p in &fp.params {
                    let pty = self.resolve_ty(&p.ty);
                    self.check_mode_on_borrow(p.mode, &pty, p.ty.span);
                    params.push((p.mode, pty));
                }
                Type::FnPtr(crate::types::FnPtrTy {
                    params,
                    alloc: fp.alloc,
                    ret: Box::new(self.resolve_ty(&fp.ret)),
                })
            }
        }
    }

    /// A `read`/`write` mode on an already-borrow-kind type is ill-formed
    /// (design 0001 §3.1, review finding 12).
    fn check_mode_on_borrow(&mut self, mode: ParamMode, ty: &Type, span: Span) {
        if matches!(mode, ParamMode::Read | ParamMode::Write) && ty.is_borrow_kind() {
            self.diags.push(
                Diag::error(
                    "E0203",
                    format!(
                        "mode `{}` on a borrow-kind type `{}` is ill-formed",
                        if mode == ParamMode::Read { "read" } else { "write" },
                        ty.display()
                    ),
                    span,
                )
                .with_note(
                    "slices and borrows are passed by value; drop the mode keyword",
                    None,
                ),
            );
        }
    }
}

/// Resolve a whole program into an [`Items`] table, pushing diagnostics.
pub fn resolve_program(prog: &Program, diags: &mut Vec<Diag>) -> Items {
    // Phase 0: collect type names (structs + enums share the type namespace)
    // and detect duplicate item names.
    let mut type_names: HashSet<String> = HashSet::new();
    let mut seen_types: HashMap<String, Span> = HashMap::new();
    let mut seen_fns: HashMap<String, Span> = HashMap::new();
    let mut seen_statics: HashMap<String, Span> = HashMap::new();

    for item in &prog.items {
        match item {
            Item::Struct(s) => dup_check(&mut seen_types, &s.name, s.span, "type", diags, || {
                type_names.insert(s.name.clone());
            }),
            Item::Enum(e) => dup_check(&mut seen_types, &e.name, e.span, "type", diags, || {
                type_names.insert(e.name.clone());
            }),
            Item::Fn(f) => dup_check(&mut seen_fns, &f.name, f.span, "function", diags, || {}),
            Item::Static(s) => {
                dup_check(&mut seen_statics, &s.name, s.span, "static", diags, || {})
            }
        }
    }

    // Phase 1: resolve field/variant/signature types.
    let mut items = Items::default();
    {
        let mut r = Resolver {
            type_names: &type_names,
            diags,
        };
        for item in &prog.items {
            match item {
                Item::Struct(s) => {
                    let mut seen_f: HashMap<String, Span> = HashMap::new();
                    let mut fields = Vec::new();
                    for f in &s.fields {
                        if let Some(prev) = seen_f.insert(f.name.clone(), f.span) {
                            r.diags.push(
                                Diag::error(
                                    "E0105",
                                    format!("duplicate field `{}`", f.name),
                                    f.span,
                                )
                                .with_note("first declared here", Some(prev)),
                            );
                            continue;
                        }
                        let fty = r.resolve_ty(&f.ty);
                        // §3.4: no borrow-typed fields (slices included).
                        if field_stores_borrow(&fty) {
                            r.diags.push(
                                Diag::error(
                                    "E0201",
                                    format!(
                                        "struct field `{}` may not have a borrow type `{}`",
                                        f.name,
                                        fty.display()
                                    ),
                                    f.ty.span,
                                )
                                .with_note(
                                    "borrows are a gear for passing and computing, not storing (§3.4); use an owning value, an index, or a rawptr",
                                    None,
                                ),
                            );
                        }
                        fields.push((f.name.clone(), fty));
                    }
                    items.structs.insert(
                        s.name.clone(),
                        StructTy {
                            copy: s.copy,
                            has_drop: s.drop_hook.is_some(),
                            fields,
                            span: s.span,
                        },
                    );
                }
                Item::Enum(e) => {
                    let mut seen_v: HashMap<String, Span> = HashMap::new();
                    let mut variants = Vec::new();
                    for v in &e.variants {
                        if let Some(prev) = seen_v.insert(v.name.clone(), v.span) {
                            r.diags.push(
                                Diag::error(
                                    "E0106",
                                    format!("duplicate variant `{}`", v.name),
                                    v.span,
                                )
                                .with_note("first declared here", Some(prev)),
                            );
                            continue;
                        }
                        let payload: Vec<Type> =
                            v.payload.iter().map(|t| r.resolve_ty(t)).collect();
                        for (t, ast_t) in payload.iter().zip(&v.payload) {
                            if field_stores_borrow(t) {
                                r.diags.push(
                                    Diag::error(
                                        "E0201",
                                        format!(
                                            "enum payload may not have a borrow type `{}`",
                                            t.display()
                                        ),
                                        ast_t.span,
                                    )
                                    .with_note("borrows may not be stored (§3.4)", None),
                                );
                            }
                        }
                        variants.push(VariantTy {
                            name: v.name.clone(),
                            payload,
                        });
                    }
                    items.enums.insert(
                        e.name.clone(),
                        EnumTy {
                            copy: e.copy,
                            variants,
                            span: e.span,
                        },
                    );
                }
                Item::Fn(f) => {
                    let mut seen_p: HashMap<String, Span> = HashMap::new();
                    let mut params = Vec::new();
                    for p in &f.params {
                        if let Some(prev) = seen_p.insert(p.name.clone(), p.span) {
                            r.diags.push(
                                Diag::error(
                                    "E0107",
                                    format!("duplicate parameter `{}`", p.name),
                                    p.span,
                                )
                                .with_note("first declared here", Some(prev)),
                            );
                        }
                        let dty = r.resolve_ty(&p.ty);
                        r.check_mode_on_borrow(p.mode, &dty, p.ty.span);
                        let lowered = lower_param(p.mode, dty.clone());
                        params.push(ParamInfo {
                            name: p.name.clone(),
                            mode: p.mode,
                            region: p.region.clone(),
                            decl_ty: dty,
                            lowered,
                            span: p.span,
                        });
                    }
                    let (ret, ret_region, ret_span) = match &f.ret {
                        Some(rt) => {
                            let base = r.resolve_ty(&rt.ty);
                            let t = match rt.borrow {
                                Some(BorrowKind::Shared) => Type::Borrow(Box::new(base)),
                                Some(BorrowKind::Exclusive) => Type::BorrowMut(Box::new(base)),
                                None => base,
                            };
                            (t, rt.region.clone(), rt.span)
                        }
                        None => (Type::unit(), None, f.span),
                    };
                    items.fns.insert(
                        f.name.clone(),
                        FnSig {
                            name: f.name.clone(),
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
                Item::Static(s) => {
                    let ty = r.resolve_ty(&s.ty);
                    items.statics.insert(s.name.clone(), (ty, s.span));
                }
            }
        }
    }

    // Phase 2: copy-marker validity (needs the full table for nominal copyability).
    for item in &prog.items {
        match item {
            Item::Struct(s) if s.copy => {
                let info = &items.structs[&s.name];
                if info.has_drop {
                    diags.push(
                        Diag::error(
                            "E0202",
                            format!("`copy` struct `{}` may not have a `drop` hook", s.name),
                            s.span,
                        )
                        .with_note("a copy type has no destructor (§1.3)", None),
                    );
                }
                for (fname, fty) in &info.fields {
                    if !is_copy(fty, &items) {
                        diags.push(
                            Diag::error(
                                "E0202",
                                format!(
                                    "`copy` struct `{}` has non-copy field `{}: {}`",
                                    s.name,
                                    fname,
                                    fty.display()
                                ),
                                s.span,
                            )
                            .with_note("every field of a copy type must itself be copy (§1.3)", None),
                        );
                    }
                }
            }
            Item::Enum(e) if e.copy => {
                let info = &items.enums[&e.name];
                for v in &info.variants {
                    for pty in &v.payload {
                        if !is_copy(pty, &items) {
                            diags.push(
                                Diag::error(
                                    "E0202",
                                    format!(
                                        "`copy` enum `{}` has non-copy payload `{}` in variant `{}`",
                                        e.name,
                                        pty.display(),
                                        v.name
                                    ),
                                    e.span,
                                )
                                .with_note("every payload of a copy enum must be copy (§1.3)", None),
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }

    items
}

fn dup_check(
    seen: &mut HashMap<String, Span>,
    name: &str,
    span: Span,
    what: &str,
    diags: &mut Vec<Diag>,
    on_new: impl FnOnce(),
) {
    if let Some(prev) = seen.insert(name.to_string(), span) {
        diags.push(
            Diag::error("E0101", format!("duplicate {what} `{name}`"), span)
                .with_note("first declared here", Some(prev)),
        );
    } else {
        on_new();
    }
}
