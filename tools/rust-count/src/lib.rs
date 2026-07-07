//! Bet 5 measurement instrument — Rust side.
//!
//! Mechanical, deterministic counter implementing the frozen classification /
//! unit table `docs/BET5_UNIT_TABLE.md` (table_version 1), emitting the SAME
//! JSON shape as the Candor counter (`prototype/src/count.rs`) so the two are
//! compared at a shared normalized unit (criterion §3.1, §3.5).
//!
//! Annotation (classes a/b/c/d) is counted at **item-level declaration sites
//! only** — fn signatures (params incl. receiver, return, lifetime generics),
//! and data declarations (fields, statics, consts, type aliases, fn-pointer type
//! components). Function *bodies* contribute only value-copy units
//! (`.clone()`/`.to_owned()`) and valve regions (`unsafe` blocks, raw-pointer /
//! Cell-family type mentions). Local `let` type annotations are use-site-adjacent
//! and never annotation (table §5.7).

use proc_macro2::LineColumn;
use serde::Serialize;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    Block, FnArg, GenericArgument, GenericParam, Generics, ImplItem, Item, PathArguments,
    ReturnType, Signature, Stmt, TraitItem, Type, WherePredicate,
};

pub const TABLE_VERSION: &str = "1";

/// Additive unit-extension version (mirrors the Candor counter). `table_version`
/// stays "1"; this marks the presence of the exact valve-statement unit.
pub const UNIT_EXT_VERSION: &str = "2";

#[derive(Serialize)]
pub struct Counts {
    pub table_version: &'static str,
    pub unit_ext_version: &'static str,
    pub annotation_units: Annotation,
    pub value_copy_units: usize,
    pub logical_statements: usize,
    /// Logical statements whose byte span intersects any valve region span
    /// without strictly enclosing it (the exact valve-statement unit, §4.1).
    pub valve_statements: usize,
    pub valve: Valve,
}

#[derive(Serialize)]
pub struct Annotation {
    pub a: usize,
    pub b: usize,
    pub c: usize,
    pub d: usize,
    pub per_site: Vec<Site>,
}

#[derive(Serialize, Clone)]
pub struct Site {
    pub class: &'static str,
    pub kind: &'static str,
    pub start: usize,
    pub end: usize,
}

#[derive(Serialize)]
pub struct Valve {
    pub lines: usize,
    pub functions: usize,
    pub total_lines: usize,
    pub total_functions: usize,
}

impl Counts {
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).expect("Counts is serializable")
    }
}

/// Parse and count a Rust source string against the frozen table.
pub fn count_str(src: &str) -> Result<Counts, syn::Error> {
    let file = syn::parse_file(src)?;
    let mut c = Counter::new(src);
    for item in &file.items {
        c.count_item(item);
    }
    Ok(c.finish(src))
}

// A source span reduced to ((start_line, start_col), (end_line, end_col)).
type Sp = (LineColumn, LineColumn);

fn sp_of<T: Spanned>(t: &T) -> Sp {
    let s = t.span();
    (s.start(), s.end())
}

fn lc_le(a: LineColumn, b: LineColumn) -> bool {
    (a.line, a.column) <= (b.line, b.column)
}

fn lc_lt(a: LineColumn, b: LineColumn) -> bool {
    (a.line, a.column) < (b.line, b.column)
}

/// Half-open spans (in (line, column) space) intersect if they share a position.
fn sp_intersect(a: Sp, b: Sp) -> bool {
    lc_lt(a.0, b.1) && lc_lt(b.0, a.1)
}

/// `outer` strictly contains `inner`: covers it and is strictly larger, so a
/// statement merely wrapping a valve region is excluded from the count.
fn sp_strictly_contains(outer: Sp, inner: Sp) -> bool {
    lc_le(outer.0, inner.0)
        && lc_le(inner.1, outer.1)
        && (lc_lt(outer.0, inner.0) || lc_lt(inner.1, outer.1))
}

struct Counter {
    line_starts: Vec<usize>,
    sites: Vec<Site>,
    value_copy: usize,
    logical_statements: usize,
    valve_spans: Vec<Sp>,
    fn_spans: Vec<Sp>,
    stmt_spans: Vec<Sp>,
}

impl Counter {
    fn new(src: &str) -> Counter {
        Counter {
            line_starts: line_starts(src),
            sites: Vec::new(),
            value_copy: 0,
            logical_statements: 0,
            valve_spans: Vec::new(),
            fn_spans: Vec::new(),
            stmt_spans: Vec::new(),
        }
    }

    fn byte(&self, lc: LineColumn) -> usize {
        // 1-based line, 0-based column; source assumed ASCII for the audit offset.
        self.line_starts
            .get(lc.line.saturating_sub(1))
            .copied()
            .unwrap_or(0)
            + lc.column
    }

    fn site(&mut self, class: &'static str, kind: &'static str, sp: Sp) {
        self.sites.push(Site {
            class,
            kind,
            start: self.byte(sp.0),
            end: self.byte(sp.1),
        });
    }

    // ---- items -----------------------------------------------------------

    fn count_item(&mut self, item: &Item) {
        match item {
            Item::Fn(f) => {
                self.logical_statements += 1;
                self.function(&f.sig, Some(&f.block));
            }
            Item::Struct(s) => {
                self.logical_statements += 1;
                self.stmt_spans.push(sp_of(s));
                self.generics_decl(&s.generics);
                for field in &s.fields {
                    self.decl_type(&field.ty);
                }
            }
            Item::Enum(e) => {
                self.logical_statements += 1;
                self.stmt_spans.push(sp_of(e));
                self.generics_decl(&e.generics);
                for v in &e.variants {
                    for field in &v.fields {
                        self.decl_type(&field.ty);
                    }
                }
            }
            Item::Union(u) => {
                self.logical_statements += 1;
                self.stmt_spans.push(sp_of(u));
                self.generics_decl(&u.generics);
                for field in &u.fields.named {
                    self.decl_type(&field.ty);
                }
            }
            Item::Const(c) => {
                self.logical_statements += 1;
                self.stmt_spans.push(sp_of(c));
                self.decl_type(&c.ty);
            }
            Item::Static(s) => {
                self.logical_statements += 1;
                self.stmt_spans.push(sp_of(s));
                self.decl_type(&s.ty);
            }
            Item::Type(t) => {
                self.logical_statements += 1;
                self.stmt_spans.push(sp_of(t));
                self.generics_decl(&t.generics);
                self.decl_type(&t.ty);
            }
            Item::Trait(t) => {
                self.logical_statements += 1;
                self.stmt_spans.push(sp_of(t));
                self.generics_decl(&t.generics);
                for ti in &t.items {
                    match ti {
                        TraitItem::Fn(m) => {
                            self.logical_statements += 1;
                            self.function(&m.sig, m.default.as_ref());
                        }
                        TraitItem::Const(c) => {
                            self.logical_statements += 1;
                            self.stmt_spans.push(sp_of(c));
                            self.decl_type(&c.ty);
                        }
                        TraitItem::Type(t) => {
                            self.logical_statements += 1;
                            self.stmt_spans.push(sp_of(t));
                        }
                        _ => {}
                    }
                }
            }
            Item::Impl(im) => {
                self.generics_decl(&im.generics);
                for ii in &im.items {
                    match ii {
                        ImplItem::Fn(m) => {
                            self.logical_statements += 1;
                            self.function(&m.sig, Some(&m.block));
                        }
                        ImplItem::Const(c) => {
                            self.logical_statements += 1;
                            self.stmt_spans.push(sp_of(c));
                            self.decl_type(&c.ty);
                        }
                        ImplItem::Type(t) => {
                            self.logical_statements += 1;
                            self.stmt_spans.push(sp_of(t));
                            self.decl_type(&t.ty);
                        }
                        _ => {}
                    }
                }
            }
            Item::Mod(m) => {
                if let Some((_, items)) = &m.content {
                    self.logical_statements += 1;
                    self.stmt_spans.push(sp_of(m));
                    for it in items {
                        self.count_item(it);
                    }
                }
            }
            _ => {}
        }
    }

    /// A function-like declaration: its signature is annotation; its body (if any)
    /// contributes value-copy units, valve regions, and nested statements.
    fn function(&mut self, sig: &Signature, body: Option<&Block>) {
        let fn_sp = sig_body_span(sig, body);
        self.fn_spans.push(fn_sp);
        self.stmt_spans.push(fn_sp);

        // (d) `unsafe fn`: valve-entry declaration + the whole body a valve region.
        if sig.unsafety.is_some() {
            self.site("d", "unsafe_fn", sp_of(&sig.fn_token));
            self.valve_spans.push(fn_sp);
        }

        self.generics_decl(&sig.generics);
        for input in &sig.inputs {
            match input {
                FnArg::Receiver(r) => {
                    if let Some((amp, lt)) = &r.reference {
                        let kind = if r.mutability.is_some() {
                            "ref_mut_self"
                        } else {
                            "ref_self"
                        };
                        self.site("a", kind, sp_of(amp));
                        if let Some(lt) = lt {
                            if named_lifetime(&lt.ident.to_string()) {
                                self.site("b", "region_use", sp_of(lt));
                            }
                        }
                    }
                }
                FnArg::Typed(pt) => self.decl_type(&pt.ty),
            }
        }
        if let ReturnType::Type(_, ty) = &sig.output {
            self.decl_type(ty);
        }

        if let Some(block) = body {
            let mut v = BodyVisitor { c: self };
            v.visit_block(block);
        }
    }

    /// Lifetime *declarations* and lifetime *bounds* of a generics list (class b).
    fn generics_decl(&mut self, g: &Generics) {
        for p in &g.params {
            if let GenericParam::Lifetime(lp) = p {
                self.site("b", "region_decl", sp_of(&lp.lifetime));
                for b in &lp.bounds {
                    if named_lifetime(&b.ident.to_string()) {
                        self.site("b", "region_bound", sp_of(b));
                    }
                }
            }
        }
        if let Some(wc) = &g.where_clause {
            for pred in &wc.predicates {
                match pred {
                    WherePredicate::Lifetime(pl) => {
                        for b in &pl.bounds {
                            if named_lifetime(&b.ident.to_string()) {
                                self.site("b", "region_bound", sp_of(b));
                            }
                        }
                    }
                    WherePredicate::Type(pt) => {
                        for b in &pt.bounds {
                            if let syn::TypeParamBound::Lifetime(lt) = b {
                                if named_lifetime(&lt.ident.to_string()) {
                                    self.site("b", "region_bound", sp_of(lt));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Walk a **declaration** type, recording class-(a) references, class-(b)
    /// lifetime attachments, and class-(d) raw-pointer / Cell-family mentions.
    fn decl_type(&mut self, ty: &Type) {
        match ty {
            Type::Reference(r) => {
                let kind = if r.mutability.is_some() {
                    "ref_mut"
                } else {
                    "ref"
                };
                self.site("a", kind, sp_of(&r.and_token));
                if let Some(lt) = &r.lifetime {
                    if named_lifetime(&lt.ident.to_string()) {
                        self.site("b", "region_use", sp_of(lt));
                    }
                }
                self.decl_type(&r.elem);
            }
            Type::Ptr(p) => {
                let kind = if p.const_token.is_some() {
                    "rawptr_const"
                } else {
                    "rawptr_mut"
                };
                let sp = sp_of(p);
                self.site("d", kind, sp);
                self.valve_spans.push(sp);
                self.decl_type(&p.elem);
            }
            Type::Path(tp) => {
                if let Some(seg) = tp.path.segments.last() {
                    if is_cell(&seg.ident.to_string()) {
                        let sp = sp_of(ty);
                        self.site("d", "cell", sp);
                        self.valve_spans.push(sp);
                    }
                }
                if let Some(q) = &tp.qself {
                    self.decl_type(&q.ty);
                }
                for seg in &tp.path.segments {
                    if let PathArguments::AngleBracketed(ab) = &seg.arguments {
                        for arg in &ab.args {
                            match arg {
                                GenericArgument::Lifetime(lt) => {
                                    if named_lifetime(&lt.ident.to_string()) {
                                        self.site("b", "region_use", sp_of(lt));
                                    }
                                }
                                GenericArgument::Type(t) => self.decl_type(t),
                                _ => {}
                            }
                        }
                    }
                }
            }
            Type::Tuple(t) => {
                for e in &t.elems {
                    self.decl_type(e);
                }
            }
            Type::Array(a) => self.decl_type(&a.elem),
            Type::Slice(s) => self.decl_type(&s.elem),
            Type::Paren(p) => self.decl_type(&p.elem),
            Type::Group(g) => self.decl_type(&g.elem),
            Type::BareFn(bf) => {
                for input in &bf.inputs {
                    self.decl_type(&input.ty);
                }
                if let ReturnType::Type(_, ty) = &bf.output {
                    self.decl_type(ty);
                }
            }
            _ => {}
        }
    }

    // ---- finish ----------------------------------------------------------

    fn finish(mut self, src: &str) -> Counts {
        self.sites.sort_by(|x, y| {
            (x.start, x.end, x.class, x.kind).cmp(&(y.start, y.end, y.class, y.kind))
        });
        let count = |cl: &str| self.sites.iter().filter(|s| s.class == cl).count();
        let (a, b, c, d) = (count("a"), count("b"), 0usize, count("d"));

        let code = code_lines(src);
        let total_lines = code.len();
        let mut valve_lines: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
        for vs in &self.valve_spans {
            for ln in vs.0.line..=vs.1.line {
                if code.contains(&ln) {
                    valve_lines.insert(ln);
                }
            }
        }

        // Valve statements: logical statements whose span intersects a valve
        // region span without strictly enclosing it (§4.1) — statements inside
        // or partly inside a valve region, excluding enclosing fns/blocks.
        let valve_statements = self
            .stmt_spans
            .iter()
            .filter(|st| {
                self.valve_spans
                    .iter()
                    .any(|v| sp_intersect(**st, *v) && !sp_strictly_contains(**st, *v))
            })
            .count();

        let total_functions = self.fn_spans.len();
        let valve_functions = self
            .fn_spans
            .iter()
            .filter(|fs| {
                self.valve_spans
                    .iter()
                    .any(|vs| lc_le(fs.0, vs.0) && lc_le(vs.1, fs.1))
            })
            .count();

        Counts {
            table_version: TABLE_VERSION,
            unit_ext_version: UNIT_EXT_VERSION,
            annotation_units: Annotation {
                a,
                b,
                c,
                d,
                per_site: self.sites,
            },
            value_copy_units: self.value_copy,
            logical_statements: self.logical_statements,
            valve_statements,
            valve: Valve {
                lines: valve_lines.len(),
                functions: valve_functions,
                total_lines,
                total_functions,
            },
        }
    }
}

/// Body traversal: value-copy calls, `unsafe` blocks, in-body valve type mentions,
/// nested items, and statement counting. Annotation is NOT counted here.
struct BodyVisitor<'a> {
    c: &'a mut Counter,
}

impl<'ast, 'a> Visit<'ast> for BodyVisitor<'a> {
    fn visit_stmt(&mut self, s: &'ast Stmt) {
        match s {
            Stmt::Item(it) => {
                // A nested item: hand back to the item counter; do NOT recurse as a
                // body (its signature types are declaration annotation, not valve).
                self.c.count_item(it);
            }
            Stmt::Local(l) => {
                self.c.logical_statements += 1;
                self.c.stmt_spans.push(sp_of(s));
                syn::visit::visit_local(self, l);
            }
            Stmt::Expr(..) => {
                self.c.logical_statements += 1;
                self.c.stmt_spans.push(sp_of(s));
                syn::visit::visit_stmt(self, s);
            }
            Stmt::Macro(_) => {
                self.c.logical_statements += 1;
                self.c.stmt_spans.push(sp_of(s));
                syn::visit::visit_stmt(self, s);
            }
        }
    }

    fn visit_expr_method_call(&mut self, m: &'ast syn::ExprMethodCall) {
        let name = m.method.to_string();
        if (name == "clone" || name == "to_owned") && m.args.is_empty() {
            self.c.value_copy += 1;
        }
        syn::visit::visit_expr_method_call(self, m);
    }

    fn visit_expr_unsafe(&mut self, u: &'ast syn::ExprUnsafe) {
        let sp = sp_of(u);
        self.c.site("d", "unsafe_block", sp);
        self.c.valve_spans.push(sp);
        syn::visit::visit_expr_unsafe(self, u);
    }

    fn visit_type_ptr(&mut self, p: &'ast syn::TypePtr) {
        // A raw-pointer type mention in the body (cast target, let-type, turbofish):
        // a valve token (region/line), not declaration annotation.
        self.c.valve_spans.push(sp_of(p));
        syn::visit::visit_type_ptr(self, p);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        if let Some(seg) = path.segments.last() {
            if is_cell(&seg.ident.to_string()) {
                self.c.valve_spans.push(sp_of(path));
            }
        }
        syn::visit::visit_path(self, path);
    }
}

fn named_lifetime(ident: &str) -> bool {
    ident != "static" && ident != "_"
}

fn is_cell(ident: &str) -> bool {
    matches!(ident, "Cell" | "RefCell" | "UnsafeCell")
}

fn sig_body_span(sig: &Signature, body: Option<&Block>) -> Sp {
    let start = sig.span().start();
    let end = match body {
        Some(b) => b.span().end(),
        None => sig.span().end(),
    };
    (start, end)
}

fn line_starts(src: &str) -> Vec<usize> {
    let mut v = vec![0usize];
    for (i, b) in src.bytes().enumerate() {
        if b == b'\n' {
            v.push(i + 1);
        }
    }
    v
}

/// Source lines bearing >=1 token (table §7), from the proc-macro2 token stream.
/// Regular `//` and `/* */` comments and blank lines carry no token and are
/// excluded (doc comments lex to attribute tokens and do count — table §7 note).
fn code_lines(src: &str) -> std::collections::BTreeSet<usize> {
    use std::str::FromStr;
    let mut set = std::collections::BTreeSet::new();
    if let Ok(ts) = proc_macro2::TokenStream::from_str(src) {
        collect_token_lines(ts, &mut set);
    }
    set
}

fn collect_token_lines(ts: proc_macro2::TokenStream, set: &mut std::collections::BTreeSet<usize>) {
    for tt in ts {
        let span = tt.span();
        for ln in span.start().line..=span.end().line {
            set.insert(ln);
        }
        if let proc_macro2::TokenTree::Group(g) = tt {
            collect_token_lines(g.stream(), set);
        }
    }
}
