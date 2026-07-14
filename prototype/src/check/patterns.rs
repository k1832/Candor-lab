//! Pattern binding-mode derivation and match exhaustiveness
//! (design 0001 §8.2 exhaustiveness, §8.2.1 binding modes).
//!
//! The binding mode of each payload variable is fixed by *how the scrutinee is
//! held* (§8.2.1); the chosen mode is recorded per binding for Stage 3. The
//! compiler-known `BoxResult T` enum is synthesized on demand (§6.2).

use crate::ast::{PatKind, Pattern};
use crate::diag::Diag;
use crate::span::Span;
use crate::types::{is_copy, EnumTy, ItemEnv, Type, VariantTy};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoldMode {
    Owned,
    Shared,
    Excl,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindMode {
    Move,
    Copy,
    BorrowShared,
    BorrowExcl,
}

#[derive(Clone, Debug)]
pub struct Binding {
    pub name: String,
    pub ty: Type,
    pub mode: BindMode,
    /// Retained for the value-gear move analysis; borrow-bindings are borrow
    /// *values* (design 0001 §8.2.1) and so are always movable-as-values.
    pub movable: bool,
    pub span: Span,
}

/// Resolve the enum being matched: peels borrows/box, synthesizes `BoxResult`.
pub fn resolve_enum(subject: &Type, items: &dyn ItemEnv) -> Option<(HoldMode, EnumTy, String)> {
    fn synth_box_result(t: &Type) -> EnumTy {
        EnumTy {
            copy: false,
            variants: vec![
                VariantTy {
                    name: "boxed".to_string(),
                    payload: vec![Type::Box(Box::new(t.clone()))],
                },
                VariantTy {
                    name: "oom".to_string(),
                    payload: vec![],
                },
            ],
            ok_variant: Some("boxed".to_string()),
            span: Span::point(0),
        }
    }
    match subject {
        Type::Named(n) => items.lookup_enum(n).map(|e| (HoldMode::Owned, e.clone(), n.clone())),
        // A generic enum application: substitute the arguments into the variant
        // payloads to obtain a concrete enum shape (design 0007 §5).
        Type::App(n, args) => items.lookup_generic(n).filter(|g| g.is_enum).map(|g| {
            let map: std::collections::HashMap<String, Type> =
                g.params.iter().cloned().zip(args.iter().cloned()).collect();
            let variants = g
                .variants
                .iter()
                .map(|(vn, payload, _)| VariantTy {
                    name: vn.clone(),
                    payload: payload.iter().map(|t| crate::types::subst(t, &map)).collect(),
                })
                .collect();
            let ok_variant = g.variants.iter().find(|(_, _, ok)| *ok).map(|(vn, _, _)| vn.clone());
            (HoldMode::Owned, EnumTy { copy: g.copy, variants, ok_variant, span: Span::point(0) }, n.clone())
        }),
        Type::BoxResult(t) => Some((HoldMode::Owned, synth_box_result(t), "BoxResult".to_string())),
        Type::Borrow(inner) => {
            resolve_enum(inner, items).map(|(_, e, n)| (HoldMode::Shared, e, n))
        }
        Type::BorrowMut(inner) => {
            resolve_enum(inner, items).map(|(_, e, n)| (HoldMode::Excl, e, n))
        }
        Type::Box(inner) => resolve_enum(inner, items).map(|(_, e, n)| (HoldMode::Owned, e, n)),
        _ => None,
    }
}

fn mode_for(hold: HoldMode, ty: &Type, items: &dyn ItemEnv) -> (BindMode, bool) {
    let copy = is_copy(ty, items);
    match hold {
        HoldMode::Owned => {
            if copy {
                (BindMode::Copy, true)
            } else {
                (BindMode::Move, true)
            }
        }
        HoldMode::Shared => {
            if copy {
                (BindMode::Copy, true)
            } else {
                (BindMode::BorrowShared, true)
            }
        }
        HoldMode::Excl => {
            if copy {
                (BindMode::Copy, true)
            } else {
                (BindMode::BorrowExcl, true)
            }
        }
    }
}

/// The value type of a payload binding: a borrowed-scrutinee binding is a
/// *borrow* of the payload sub-place (design 0001 §8.2.1), so its type wears the
/// borrow (`borrow T` / `borrow_mut T`); a moved/copied binding is the payload
/// type itself.
fn binding_ty(mode: BindMode, ty: &Type) -> Type {
    match mode {
        BindMode::Move | BindMode::Copy => ty.clone(),
        BindMode::BorrowShared => Type::Borrow(Box::new(ty.clone())),
        BindMode::BorrowExcl => Type::BorrowMut(Box::new(ty.clone())),
    }
}

fn sub_hold(bind: BindMode) -> HoldMode {
    match bind {
        BindMode::Move | BindMode::Copy => HoldMode::Owned,
        BindMode::BorrowShared => HoldMode::Shared,
        BindMode::BorrowExcl => HoldMode::Excl,
    }
}

/// Analyze one arm pattern against the enum `einfo` held under `hold`,
/// collecting payload bindings and emitting arity/variant diagnostics.
pub fn analyze_pattern(
    pat: &Pattern,
    einfo: &EnumTy,
    ename: &str,
    hold: HoldMode,
    items: &dyn ItemEnv,
    diags: &mut Vec<Diag>,
    out: &mut Vec<Binding>,
) {
    match &pat.kind {
        PatKind::Wildcard => {}
        PatKind::Binding(name) => {
            // A whole-value binding of the scrutinee.
            let ty = Type::Named(ename.to_string());
            let (mode, movable) = mode_for(hold, &ty, items);
            out.push(Binding {
                name: name.clone(),
                ty: binding_ty(mode, &ty),
                mode,
                movable,
                span: pat.span,
            });
        }
        PatKind::Variant {
            enum_name,
            variant,
            sub,
        } => {
            if enum_name != ename && enum_name != "BoxResult" {
                diags.push(Diag::error(
                    "E0604",
                    format!("pattern names enum `{enum_name}` but scrutinee is `{ename}`"),
                    pat.span,
                ));
            }
            let vinfo = match einfo.variants.iter().find(|v| v.name == *variant) {
                Some(v) => v,
                None => {
                    diags.push(Diag::error(
                        "E0108",
                        format!("enum `{ename}` has no variant `{variant}`"),
                        pat.span,
                    ));
                    return;
                }
            };
            if sub.len() != vinfo.payload.len() {
                diags.push(Diag::error(
                    "E0605",
                    format!(
                        "variant `{}::{}` expects {} payload(s), found {}",
                        ename,
                        variant,
                        vinfo.payload.len(),
                        sub.len()
                    ),
                    pat.span,
                ));
                return;
            }
            for (sp, pty) in sub.iter().zip(&vinfo.payload) {
                bind_sub(sp, pty, hold, items, diags, out);
            }
        }
        PatKind::IntLit { .. } => {
            diags.push(Diag::error(
                "E0606",
                format!("integer-literal pattern cannot match enum scrutinee `{ename}`"),
                pat.span,
            ));
        }
        PatKind::IntRange { .. } => {
            diags.push(Diag::error(
                "E0606",
                format!("integer-range pattern cannot match enum scrutinee `{ename}`"),
                pat.span,
            ));
        }
    }
}

fn bind_sub(
    pat: &Pattern,
    ty: &Type,
    hold: HoldMode,
    items: &dyn ItemEnv,
    diags: &mut Vec<Diag>,
    out: &mut Vec<Binding>,
) {
    match &pat.kind {
        PatKind::Wildcard => {}
        PatKind::Binding(name) => {
            let (mode, movable) = mode_for(hold, ty, items);
            out.push(Binding {
                name: name.clone(),
                ty: binding_ty(mode, ty),
                mode,
                movable,
                span: pat.span,
            });
        }
        PatKind::Variant { .. } => {
            let (mode, _) = mode_for(hold, ty, items);
            let h = sub_hold(mode);
            if let Some((_, einfo, ename)) = resolve_enum(ty, items) {
                analyze_pattern(pat, &einfo, &ename, h, items, diags, out);
            }
        }
        PatKind::IntLit { .. } => {
            diags.push(Diag::error(
                "E0606",
                "integer-literal sub-patterns are not supported".to_string(),
                pat.span,
            ));
        }
        PatKind::IntRange { .. } => {
            diags.push(Diag::error(
                "E0606",
                "integer-range sub-patterns are not supported".to_string(),
                pat.span,
            ));
        }
    }
}

/// Exhaustiveness (design 0001 §8.2). Returns a diagnostic if not exhaustive.
pub fn check_exhaustive(
    arms: &[crate::ast::MatchArm],
    einfo: &EnumTy,
    enum_name: &str,
    span: Span,
) -> Option<Diag> {
    let mut covered: Vec<String> = Vec::new();
    for arm in arms {
        // A guarded arm may not fire (its guard can be false at runtime), so it
        // does NOT contribute to exhaustiveness (design 0001 §8.2, extended).
        if arm.guard.is_some() {
            continue;
        }
        match &arm.pattern.kind {
            PatKind::Wildcard | PatKind::Binding(_) => return None,
            PatKind::Variant { variant, .. } => covered.push(variant.clone()),
            PatKind::IntLit { .. } | PatKind::IntRange { .. } => {}
        }
    }
    let missing: Vec<String> = einfo
        .variants
        .iter()
        .filter(|v| !covered.contains(&v.name))
        .map(|v| format!("{enum_name}::{}", v.name))
        .collect();
    if missing.is_empty() {
        None
    } else {
        Some(
            Diag::error(
                "E0601",
                format!("non-exhaustive match: missing {}", missing.join(", ")),
                span,
            )
            .with_note("every variant must be covered, or use a `_` wildcard arm", None),
        )
    }
}
