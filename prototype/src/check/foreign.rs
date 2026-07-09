//! The foreign boundary's static checks (design 0011): placement (E1101),
//! C-mappability (E1102, recursive), the `trust` clause discipline (E1105/E1106),
//! export well-formedness (E1102/E1107), and the `foreign` effect partition with
//! its discharge rule (E1103). The per-function effect accumulator lives here and
//! is driven from `check/expr.rs` (extern/foreign call sites) and finished in
//! `check/mod.rs` (the discharge decision).

use std::collections::HashMap;

use crate::ast::*;
use crate::diag::Diag;
use crate::resolve::Items;
use crate::span::Span;
use crate::types::{ItemEnv, Type};

/// The closed trust-predicate vocabulary (design 0011 §3). Grows only by
/// amendment, so the audit vocabulary stays finite and enumerable.
pub const TRUST_VOCAB: &[&str] = &[
    "no_retain",
    "no_free",
    "valid_for",
    "valid_nul_terminated",
    "thread_confined",
    "no_overlap",
];

/// One recorded foreign contribution inside a function body.
#[derive(Clone, Debug)]
pub struct ExternCall {
    pub name: String,
    pub has_trust: bool,
    pub span: Span,
}

/// Accumulates a function body's foreign-effect sites (design 0011 §2), so the
/// discharge decision can be made once at the end (it depends on the whole set).
#[derive(Default)]
pub struct ForeignEffect {
    /// Direct `extern` (ground-source) calls made in the body.
    pub externs: Vec<ExternCall>,
    /// First call to an already-`foreign` Candor function or foreign fn-pointer
    /// (a propagated contribution that a wrapper cannot discharge here).
    pub foreign_candor: Option<(Span, String)>,
}

/// The resolved foreign status of one function, for the audit's effect reach.
#[derive(Clone, Debug)]
pub struct ForeignFnInfo {
    pub name: String,
    pub boundary: bool,
    /// The body reaches foreign trust and is *extinguished* here (design 0011 §2
    /// rule 4): a boundary wrapper whose externs all carry trust and which makes
    /// no undischargeable foreign call.
    pub discharges: bool,
    /// The body reaches foreign trust and carries it onward (rule 3/5).
    pub propagates: bool,
}

impl ForeignEffect {
    /// Decide the discharge rule for a function (design 0011 §2). `boundary` is the
    /// function's boundary-module status; `marked` is its `foreign` signature flag.
    /// Returns `(discharges, propagates, needs_mark)`.
    pub fn resolve(&self, boundary: bool, marked: bool) -> (bool, bool, bool) {
        let calls_extern = !self.externs.is_empty();
        let calls_foreign_candor = self.foreign_candor.is_some();
        let any = calls_extern || calls_foreign_candor;
        let all_trust = self.externs.iter().all(|e| e.has_trust);
        // A wrapper discharges iff it is in a boundary module, it directly calls at
        // least one extern, *every* extern it calls carries a trust clause, and it
        // makes no undischargeable foreign call (through a `foreign` Candor fn or a
        // foreign fn-pointer). Anything short of that propagates.
        let discharges = boundary && calls_extern && all_trust && !calls_foreign_candor;
        let propagates = any && !discharges;
        let needs_mark = propagates && !marked;
        (discharges, propagates, needs_mark)
    }
}

/// Recursive C-mappability predicate (design 0011 §1). `Ok(())` if `ty` is
/// C-mappable; `Err(path)` names the offending nested type/field/element. `is_ret`
/// permits `unit` (a `void` return); a `unit` *value/parameter* is not mappable.
pub fn c_mappable(ty: &Type, items: &Items, is_ret: bool) -> Result<(), String> {
    match ty {
        Type::Scalar(crate::token::ScalarTy::Unit) => {
            if is_ret {
                Ok(())
            } else {
                Err("`unit` is only mappable as a `void` return".to_string())
            }
        }
        // integers and `bool` are scalar C types.
        Type::Scalar(_) => Ok(()),
        // `rawptr T` / `void*` is an address word regardless of pointee.
        Type::RawPtr(_) => Ok(()),
        // A foreign function pointer maps to a C function pointer (table row).
        Type::FnPtr(_) => Ok(()),
        Type::Named(n) => {
            if let Some(st) = items.lookup_struct(n) {
                for (fname, fty) in &st.fields {
                    c_mappable(fty, items, false)
                        .map_err(|inner| format!("field `{fname}`: {inner}"))?;
                }
                Ok(())
            } else if items.lookup_enum(n).is_some() {
                Err(format!("Candor `enum` `{n}` has an 8-byte payload tag with no portable C shape"))
            } else {
                Err(format!("unknown or non-mappable type `{n}`"))
            }
        }
        Type::Array(elem, _) => {
            c_mappable(elem, items, false).map_err(|inner| format!("array element: {inner}"))
        }
        Type::Slice(_) | Type::SliceMut(_) => {
            Err("a slice is a 16-byte fat pointer with no C counterpart (pass `(rawptr T, usize)`)".to_string())
        }
        Type::Str => {
            Err("`str` is a 16-byte fat pointer with no C counterpart (pass `(rawptr u8, usize)`)".to_string())
        }
        Type::Box(_) | Type::BoxResult(_) => {
            Err("a `Box` is a 24-byte tri-word with no C counterpart (pass its `rawptr T`)".to_string())
        }
        Type::Borrow(_) | Type::BorrowMut(_) => {
            Err("a Candor borrow/mode is not a C type; use a `rawptr` parameter".to_string())
        }
        Type::App(n, _) => Err(format!("generic type `{n}` is not C-mappable")),
        Type::Param(_) | Type::Proj(..) => {
            Err("a generic type parameter is not C-mappable".to_string())
        }
        Type::IntLit => Ok(()),
        Type::Never | Type::Error => Ok(()),
    }
}

/// Check every `extern`/`export` item: placement (E1101), signature mappability
/// (E1102), the trust discipline (E1105/E1106), and export references (E1107).
pub fn check_foreign_items(items: &Items, prog: &Program, diags: &mut Vec<Diag>) {
    // Map candor fn name -> whether it exists (for export references).
    let fn_names: HashMap<&str, ()> = items.fns.keys().map(|k| (k.as_str(), ())).collect();

    for item in &prog.items {
        match item {
            Item::Extern(eb) => {
                if !eb.boundary_file {
                    diags.push(placement_err("extern", eb.span));
                }
                if eb.abi != "C" {
                    diags.push(
                        Diag::error("E1108", format!("unknown ABI `\"{}\"`; only `\"C\"` is defined this edition", eb.abi), eb.span),
                    );
                }
                for ef in &eb.fns {
                    if let Some(es) = items.externs.get(&ef.name) {
                        check_extern_sig(es, items, diags);
                    }
                }
            }
            Item::Export(ex) => {
                if !ex.boundary_file {
                    diags.push(placement_err("export", ex.span));
                }
                if ex.abi != "C" {
                    diags.push(
                        Diag::error("E1108", format!("unknown ABI `\"{}\"`; only `\"C\"` is defined this edition", ex.abi), ex.span),
                    );
                }
                if let Some(ei) = items.exports.iter().find(|e| e.symbol == ex.symbol && e.span == ex.span) {
                    for p in &ei.params {
                        if let Err(path) = c_mappable(&p.lowered, items, false) {
                            diags.push(mappability_err(&format!("export `{}` parameter `{}`", ei.symbol, p.name), &path, p.span));
                        }
                    }
                    if let Err(path) = c_mappable(&ei.ret, items, true) {
                        diags.push(mappability_err(&format!("export `{}` return", ei.symbol), &path, ei.ret_span));
                    }
                    if !fn_names.contains_key(ei.candor_fn.as_str()) {
                        diags.push(
                            Diag::error(
                                "E1107",
                                format!("export `{}` references unknown Candor function `{}`", ei.symbol, ei.candor_fn),
                                ei.span,
                            )
                            .with_note("an `export` re-exposes an existing `pub` Candor function (design 0011 §1.5)", None),
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

fn check_extern_sig(es: &crate::resolve::ExternSig, items: &Items, diags: &mut Vec<Diag>) {
    for p in &es.params {
        if let Err(path) = c_mappable(&p.lowered, items, false) {
            diags.push(mappability_err(&format!("extern `{}` parameter `{}`", es.name, p.name), &path, p.span));
        }
    }
    if let Err(path) = c_mappable(&es.ret, items, true) {
        diags.push(mappability_err(&format!("extern `{}` return", es.name), &path, es.ret_span));
    }
    if let Some(t) = &es.trust {
        // A trust justification is mandatory and non-empty (design 0011 §3): the
        // implementation enforces its *presence*, never its truth.
        if t.justification.trim().is_empty() {
            diags.push(
                Diag::error("E1106", format!("`trust` on `{}` has an empty justification string", es.name), t.span)
                    .with_note("state, for the auditor, what external proof establishes the preconditions (design 0011 §3)", None),
            );
        }
        for (pname, _args, pspan) in &t.predicates {
            if !TRUST_VOCAB.contains(&pname.as_str()) {
                diags.push(
                    Diag::error("E1105", format!("unknown trust predicate `{pname}`"), *pspan)
                        .with_note(
                            "the trust vocabulary is closed: no_retain, no_free, valid_for, valid_nul_terminated, thread_confined, no_overlap (design 0011 §3)",
                            None,
                        ),
                );
            }
        }
    }
}

fn placement_err(what: &str, span: Span) -> Diag {
    Diag::error(
        "E1101",
        format!("`{what}` declaration is only allowed inside a `boundary` file"),
        span,
    )
    .with_note("open the file with the `boundary` preamble (design 0008 §4; 0011 §1)", None)
}

fn mappability_err(what: &str, path: &str, span: Span) -> Diag {
    Diag::error(
        "E1102",
        format!("{what} is not C-mappable: {path}"),
        span,
    )
    .with_note("only scalars, `rawptr`, and structs/arrays of C-mappable fields cross the boundary (design 0011 §1)", None)
}
