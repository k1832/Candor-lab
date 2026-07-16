//! Signature-only interface stubs (design 0008 §2, §2.4) — the resolution of
//! Stage C's residual (i): an importer is re-checked against its imports'
//! **interface artifacts**, never their full source items.
//!
//! A stub is an already-qualified [`Item`] carrying the module's exported
//! *contract* — every type/enum/interface/impl definition (these ARE signatures:
//! layout, variants, method heads), every `pub` fn/generic **signature**, and
//! the **checked bodies instantiation needs** (0008 §2.4): generic fn bodies,
//! `impl`-method bodies, and struct `drop` hooks (the def-site-resolved drop
//! effect crosses in the artifact). What a stub **strips** is the one thing P20
//! keeps opaque across the boundary: the body of a **non-generic** free `fn` and
//! a `static`'s initializer — pure downstream codegen input, never analysis
//! input. The checker (`check::check_module_stub`) resolves a stub's tables but
//! never re-checks its bodies; the owning module already did, once.

use crate::ast::*;
use crate::span::Span;

/// Project an already-qualified item to its interface stub (see module docs).
pub fn stub_item(item: &Item) -> Item {
    match item {
        // A non-generic free fn crosses as its signature only — its body is
        // opaque codegen the importer never analyzes (P20's default opacity).
        Item::Fn(f) if f.type_params.is_empty() => Item::Fn(FnDecl {
            body: empty_block(f.body.span),
            ..f.clone()
        }),
        // A static's initializer is codegen; only its type is the signature.
        Item::Static(s) => Item::Static(StaticDecl {
            value: Expr { kind: ExprKind::IntLit { value: 0, suffix: None }, span: s.value.span },
            ..s.clone()
        }),
        // Everything else — generic fns (0008 §2.4 checked body), impls, structs
        // (incl. `drop` hooks), enums, interfaces, foreign decls — crosses whole:
        // its body is either a signature fact or the checked body instantiation
        // consumes.
        other => other.clone(),
    }
}

fn empty_block(span: Span) -> Block {
    Block { stmts: Vec::new(), span }
}

use std::collections::HashSet;

/// The head nominal of a type, if any (`List[T]` -> `List`, `Foo` -> `Foo`).
fn ty_head(ty: &Ty) -> Option<&str> {
    match &ty.kind {
        TyKind::Named(n) => Some(n),
        TyKind::App { name, .. } => Some(name),
        _ => None,
    }
}

/// The module-qualified names of a module's **private** (non-`pub`) items.
pub fn private_locals(items: &[Item], is_pub: &[bool]) -> HashSet<String> {
    let mut out = HashSet::new();
    for (item, &p) in items.iter().zip(is_pub.iter()) {
        if p {
            continue;
        }
        let name = match item {
            Item::Struct(s) => &s.name,
            Item::Enum(e) => &e.name,
            Item::Interface(i) => &i.name,
            Item::Fn(f) => &f.name,
            Item::Static(s) => &s.name,
            _ => continue,
        };
        out.insert(name.clone());
    }
    out
}

/// The private-local names an impl **references** (its interface, target head,
/// interface args, and associated-type binding) — the items a crossing impl must
/// drag across the boundary so the stub sub-program stays well-formed.
fn impl_refs(im: &crate::ast::ImplDecl, priv_locals: &HashSet<String>) -> Vec<String> {
    let mut refs = Vec::new();
    let consider = |n: &str, out: &mut Vec<String>| {
        if priv_locals.contains(n) {
            out.push(n.to_string());
        }
    };
    consider(&im.iface, &mut refs);
    if let Some(h) = ty_head(&im.target) {
        consider(h, &mut refs);
    }
    for a in &im.iface_args {
        if let Some(h) = ty_head(a) {
            consider(h, &mut refs);
        }
    }
    if let Some((_, t)) = &im.assoc_binding {
        if let Some(h) = ty_head(t) {
            consider(h, &mut refs);
        }
    }
    refs
}

/// The interface-surface mask (design 0008 §2): which items of a module cross the
/// boundary as stubs, parallel to `items`. A `pub` item crosses. An impl crosses
/// **unless** it is module-internal (both its interface and target are private
/// locals). A private local that a crossing impl references is **dragged** across
/// so the stub sub-program remains well-formed (a `pub` type may carry a `From`
/// impl whose interface is private; the impl and its interface must travel
/// together). Privates neither `pub` nor dragged stay module-internal.
pub fn crossing_mask(items: &[Item], is_pub: &[bool]) -> Vec<bool> {
    let priv_locals = private_locals(items, is_pub);

    // Which impls cross, and the private locals they drag (to a fixpoint, since a
    // dragged type may itself be the target of another crossing impl).
    let mut dragged: HashSet<String> = HashSet::new();
    loop {
        let mut changed = false;
        for (item, &p) in items.iter().zip(is_pub.iter()) {
            if let Item::Impl(im) = item {
                let iface_internal = priv_locals.contains(&im.iface);
                let target_internal = ty_head(&im.target).map(|h| priv_locals.contains(h)).unwrap_or(false);
                let crosses = !(iface_internal && target_internal);
                if crosses {
                    for r in impl_refs(im, &priv_locals) {
                        if dragged.insert(r) {
                            changed = true;
                        }
                    }
                }
            } else if p {
                // A `pub` item may reference private locals in its signature; those
                // are already handled by the resolver's own visibility rules, so we
                // do not drag them here (the corpus has no such leak).
            }
        }
        if !changed {
            break;
        }
    }

    items
        .iter()
        .zip(is_pub.iter())
        .map(|(item, &p)| match item {
            Item::Impl(im) => {
                let iface_internal = priv_locals.contains(&im.iface);
                let target_internal = ty_head(&im.target).map(|h| priv_locals.contains(h)).unwrap_or(false);
                !(iface_internal && target_internal)
            }
            Item::Interface(i) => p || dragged.contains(&i.name),
            Item::Struct(s) => p || dragged.contains(&s.name),
            Item::Enum(e) => p || dragged.contains(&e.name),
            _ => p,
        })
        .collect()
}
