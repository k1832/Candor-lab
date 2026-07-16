//! Bet 5 measurement instrument — Candor side.
//!
//! Mechanical, deterministic counter implementing the frozen classification /
//! unit table `docs/BET5_UNIT_TABLE.md` (table_version 1). It walks the parsed
//! AST (`crate::ast`) and emits the JSON shape the Rust counter also emits, so
//! the two are compared at a shared normalized unit (criterion §3.1, §3.5).
//!
//! Annotation (classes a/b/c/d) is counted at **item-level declaration sites
//! only** — fn signatures (params, regions, return, `drop(write self)` receiver)
//! and data declarations (struct fields, enum payloads, `static` types,
//! fn-pointer type components). Function *bodies* contribute only value-copy
//! units (`clone`) and valve regions (`unsafe` blocks). Local `let` type
//! annotations are use-site-adjacent and never annotation (table §5.7).

use serde::Serialize;

use crate::ast::*;
use crate::span::Span;

pub const TABLE_VERSION: &str = "1";

/// Additive unit-extension version. `table_version` stays "1" (all its fields
/// are unchanged and computed identically); this marks the presence of the
/// exact valve-statement unit (`valve_statements`) added for the successor
/// registration (BET5_CRITERION2 §4.1).
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

/// Count a parsed program against the frozen table. `src` is the original source,
/// used only to map spans to logical lines for the valve fractions (§7).
pub fn count_program(prog: &Program, src: &str) -> Counts {
    let mut c = Counter::default();
    for item in &prog.items {
        c.item(item);
    }
    c.finish(src)
}

#[derive(Default)]
struct Counter {
    sites: Vec<Site>,
    value_copy: usize,
    logical_statements: usize,
    /// Spans that open a valve region: `unsafe` blocks and declaration-site
    /// `rawptr` type nodes (§6).
    valve_spans: Vec<Span>,
    /// Span of every function-like declaration (FnDecl or a `drop` hook), for
    /// valve-function attribution and `total_functions`.
    fn_spans: Vec<Span>,
    /// Span of every logical statement counted, for the valve-statement unit.
    stmt_spans: Vec<Span>,
}

impl Counter {
    fn site(&mut self, class: &'static str, kind: &'static str, span: Span) {
        self.sites.push(Site {
            class,
            kind,
            start: span.start,
            end: span.end,
        });
    }

    // ---- items -----------------------------------------------------------

    fn item(&mut self, item: &Item) {
        match item {
            Item::Struct(s) => {
                self.logical_statements += 1;
                self.stmt_spans.push(s.span);
                for f in &s.fields {
                    self.decl_rawptr(&f.ty);
                }
                if let Some(hook) = &s.drop_hook {
                    // `drop(write self)` ≙ Rust `fn drop(&mut self)`: a function-like
                    // declaration (one logical statement) with an exclusive-self
                    // receiver (table §5.12).
                    self.logical_statements += 1;
                    self.stmt_spans.push(hook.span);
                    self.fn_spans.push(hook.span);
                    self.site("a", "drop_self", Span::point(hook.span.start));
                    self.block(&hook.stmts);
                }
            }
            Item::Enum(e) => {
                self.logical_statements += 1;
                self.stmt_spans.push(e.span);
                for v in &e.variants {
                    for ty in &v.payload {
                        self.decl_rawptr(ty);
                    }
                }
            }
            Item::Fn(f) => {
                self.logical_statements += 1;
                self.stmt_spans.push(f.span);
                self.fn_spans.push(f.span);
                self.fn_sig(f);
                self.block(&f.body.stmts);
            }
            Item::Static(s) => {
                self.logical_statements += 1;
                self.stmt_spans.push(s.span);
                self.decl_rawptr(&s.ty);
                self.expr(&s.value);
            }
            // Generic-only items (design 0007): not part of the frozen Bet 5 unit
            // table; each is one logical declaration and impl method bodies count
            // like ordinary functions.
            Item::Interface(i) => {
                self.logical_statements += 1;
                self.stmt_spans.push(i.span);
            }
            Item::Impl(im) => {
                self.logical_statements += 1;
                self.stmt_spans.push(im.span);
                for m in &im.methods {
                    self.logical_statements += 1;
                    self.stmt_spans.push(m.span);
                    self.fn_spans.push(m.span);
                    self.fn_sig(m);
                    self.block(&m.body.stmts);
                }
            }
            // Foreign-boundary items (design 0011): each declaration counts as one
            // logical statement; declared foreign signatures have no body.
            Item::Extern(eb) => {
                self.logical_statements += 1;
                self.stmt_spans.push(eb.span);
                for ef in &eb.fns {
                    self.logical_statements += 1;
                    self.stmt_spans.push(ef.span);
                    for p in &ef.params {
                        self.decl_rawptr(&p.ty);
                    }
                }
            }
            Item::Export(ex) => {
                self.logical_statements += 1;
                self.stmt_spans.push(ex.span);
            }
        }
    }

    /// Class (a) borrow markers and class (b) region declarations at a signature
    /// (criterion §3.2 (a),(b); table §1,§2).
    fn fn_sig(&mut self, f: &FnDecl) {
        // (b) region variable declarations.
        for _r in &f.regions {
            self.site("b", "region_decl", Span::point(f.span.start));
        }
        for p in &f.params {
            self.param(p);
        }
        if let Some(ret) = &f.ret {
            // (a) borrow return.
            if ret.borrow.is_some() || is_borrow_kind_ty(&ret.ty.kind) {
                self.site("a", "borrow_return", ret.span);
            }
            // (b) region attachment on the return.
            if ret.region.is_some() {
                self.site("b", "region_return", ret.span);
            }
            self.decl_rawptr(&ret.ty);
        }
    }

    fn param(&mut self, p: &Param) {
        // Exactly one class-(a) unit per borrowing/non-value parameter.
        match p.mode {
            ParamMode::Out => self.site("a", "out_param", p.span),
            ParamMode::Read => self.site("a", "read_param", p.span),
            ParamMode::Write => self.site("a", "write_param", p.span),
            ParamMode::Take => {
                if is_borrow_kind_ty(&p.ty.kind) {
                    self.site("a", "slice_param", p.span);
                }
            }
        }
        // (b) region attachment on a borrow parameter (`read[r]`).
        if p.region.is_some() {
            self.site("b", "region_param", p.span);
        }
        // (d) raw-pointer type in the parameter declaration.
        self.decl_rawptr(&p.ty);
    }

    /// Count each `rawptr` type node inside a **declaration** type as one class-(d)
    /// valve-entry unit and open a valve region at it (§4, §6). Recurses through
    /// the type wrappers, including fn-pointer components.
    fn decl_rawptr(&mut self, ty: &Ty) {
        match &ty.kind {
            TyKind::RawPtr(inner) => {
                self.site("d", "rawptr_decl", ty.span);
                self.valve_spans.push(ty.span);
                self.decl_rawptr(inner);
            }
            TyKind::Array { elem, .. } => self.decl_rawptr(elem),
            TyKind::App { args, .. } => {
                for a in args {
                    self.decl_rawptr(a);
                }
            }
            TyKind::Slice(e)
            | TyKind::SliceMut(e)
            | TyKind::Box(e)
            | TyKind::BoxResult(e)
            | TyKind::Borrow(e)
            | TyKind::BorrowMut(e) => self.decl_rawptr(e),
            TyKind::FnPtr(fp) => {
                for pp in &fp.params {
                    self.decl_rawptr(&pp.ty);
                }
                self.decl_rawptr(&fp.ret);
            }
            TyKind::Scalar(_) | TyKind::Named(_) | TyKind::Proj { .. } => {}
        }
    }

    // ---- statements & expressions ---------------------------------------

    fn block(&mut self, stmts: &[Stmt]) {
        for s in stmts {
            self.logical_statements += 1;
            self.stmt_spans.push(s.span);
            self.stmt(s);
        }
    }

    fn stmt(&mut self, s: &Stmt) {
        match &s.kind {
            StmtKind::Let { init, .. } => {
                if let Some(e) = init {
                    self.expr(e);
                }
            }
            StmtKind::Assign { target, value } => {
                self.expr(target);
                self.expr(value);
            }
            StmtKind::Expr(e) => self.expr(e),
        }
    }

    fn expr(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::For { .. } => unreachable!("`for` is surface-only (formatter); the pipeline desugars it at parse (design 0009 §4.2)"),
            ExprKind::Scope(b) => self.block(&b.stmts),
            ExprKind::Spawn(c) => self.expr(c),
            ExprKind::IntLit { .. }
            | ExprKind::NegIntLit { .. }
            | ExprKind::FloatLit { .. }
            | ExprKind::StrLit(_)
            | ExprKind::BytesLit(_)
            | ExprKind::BoolLit(_)
            | ExprKind::Ident(_)
            | ExprKind::GenericVal { .. }
            | ExprKind::Result
            | ExprKind::Break
            | ExprKind::Continue
            | ExprKind::PtrNull { .. }
            | ExprKind::Offsetof { .. }
            | ExprKind::Sizeof(_)
            | ExprKind::Alignof(_) => {}

            ExprKind::Prefix { op, expr } => {
                if *op == PrefixOp::Clone {
                    self.value_copy += 1;
                }
                self.expr(expr);
            }
            ExprKind::Unary { expr, .. } => self.expr(expr),
            ExprKind::Binary { lhs, rhs, .. } => {
                self.expr(lhs);
                self.expr(rhs);
            }
            ExprKind::Call { callee, args } => {
                self.expr(callee);
                for a in args {
                    self.expr(a);
                }
            }
            ExprKind::OutArg(inner) => self.expr(inner),
            ExprKind::Field { base, .. } => self.expr(base),
            ExprKind::Index { base, index } => {
                self.expr(base);
                self.expr(index);
            }
            ExprKind::Conv { expr, .. } => self.expr(expr),
            ExprKind::Bitcast { expr, .. } => self.expr(expr),
            ExprKind::ArrayLit(v) => {
                for x in v {
                    self.expr(x);
                }
            }
            ExprKind::ArrayRepeat { value, size } => {
                self.expr(value);
                self.expr(size);
            }
            ExprKind::StructLit { fields, .. } => {
                for fi in fields {
                    self.expr(&fi.value);
                }
            }
            ExprKind::EnumCtor { args, .. } => {
                for a in args {
                    self.expr(a);
                }
            }
            ExprKind::CastPtr { arg, .. } | ExprKind::AddrToPtr { arg, .. } => self.expr(arg),

            // `field_ptr(p, f)` is a SAFE op (design 0004): it opens NO valve
            // region (the region rule is untouched — a field_ptr outside `unsafe`
            // is not a valve statement; inside one it counts like any statement).
            // Recurse only into `p`; `f` is a field selector, not an expression.
            ExprKind::FieldPtr { ptr, .. } => self.expr(ptr),

            ExprKind::Block(b) => self.block(&b.stmts),
            ExprKind::If {
                cond,
                then_blk,
                else_blk,
            } => {
                self.expr(cond);
                self.block(&then_blk.stmts);
                if let Some(e) = else_blk {
                    self.expr(e);
                }
            }
            ExprKind::Match { scrutinee, arms } => {
                self.expr(scrutinee);
                for arm in arms {
                    self.expr(&arm.body);
                }
            }
            ExprKind::Loop(b) => self.block(&b.stmts),
            ExprKind::While { cond, body } => {
                self.expr(cond);
                self.block(&body.stmts);
            }
            ExprKind::Unsafe { body, .. } => {
                // (d) valve-entry declaration + a valve region over the whole block.
                self.site("d", "unsafe_block", e.span);
                self.valve_spans.push(e.span);
                self.block(&body.stmts);
            }
            ExprKind::Wrapping(b) | ExprKind::Saturating(b) => self.block(&b.stmts),
            ExprKind::Return(opt) => {
                if let Some(x) = opt {
                    self.expr(x);
                }
            }
            ExprKind::Assert(x) | ExprKind::Panic(x) | ExprKind::Paren(x) => self.expr(x),
            // `expr?` is an ordinary postfix operator (design 0006 §2.4): no
            // unit-table impact, recurse into the operand.
            ExprKind::Try(x) => self.expr(x),
        }
    }

    // ---- finish ----------------------------------------------------------

    fn finish(mut self, src: &str) -> Counts {
        // Deterministic order for the audit trail.
        self.sites
            .sort_by(|x, y| (x.start, x.end, x.class, x.kind).cmp(&(y.start, y.end, y.class, y.kind)));
        let count = |cl: &str| self.sites.iter().filter(|s| s.class == cl).count();
        let (a, b, c, d) = (count("a"), count("b"), count("c"), count("d"));

        // Logical lines: source lines bearing >=1 token (table §7). Re-lex; on the
        // (already-parsed) source this cannot fail, but degrade gracefully.
        let line_starts = line_starts(src);
        let code_lines = code_lines(src, &line_starts);
        let total_lines = code_lines.len();

        // Valve lines: code lines within any valve-region span's line range.
        let mut valve_lines: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
        for vs in &self.valve_spans {
            let (lo, hi) = span_lines(*vs, &line_starts);
            for ln in lo..=hi {
                if code_lines.contains(&ln) {
                    valve_lines.insert(ln);
                }
            }
        }

        // Valve statements: logical statements whose span intersects any valve
        // region span without strictly enclosing it (§4.1). Statements inside —
        // or partly inside — a valve region count; an enclosing fn/block/if that
        // merely wraps a valve does not (it strictly contains the valve span).
        let valve_statements = self
            .stmt_spans
            .iter()
            .filter(|st| {
                self.valve_spans
                    .iter()
                    .any(|v| spans_intersect(**st, *v) && !strictly_contains(**st, *v))
            })
            .count();

        let total_functions = self.fn_spans.len();
        let valve_functions = self
            .fn_spans
            .iter()
            .filter(|fs| {
                self.valve_spans
                    .iter()
                    .any(|vs| vs.start >= fs.start && vs.end <= fs.end)
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

/// AST-level borrow-kind test (the resolved-type `is_borrow_kind` lives on
/// `types::Type`; the counter walks the raw AST, so it needs this).
fn is_borrow_kind_ty(k: &TyKind) -> bool {
    matches!(
        k,
        TyKind::Slice(_) | TyKind::SliceMut(_) | TyKind::Borrow(_) | TyKind::BorrowMut(_)
    )
}

/// Half-open spans intersect if they share at least one byte.
fn spans_intersect(a: Span, b: Span) -> bool {
    a.start < b.end && b.start < a.end
}

/// `outer` strictly contains `inner` (covers it and is strictly larger), so a
/// statement that merely wraps a valve region is excluded from the count.
fn strictly_contains(outer: Span, inner: Span) -> bool {
    outer.start <= inner.start
        && inner.end <= outer.end
        && (outer.start < inner.start || inner.end < outer.end)
}

/// Byte offsets at which each source line begins (line 1 starts at 0).
fn line_starts(src: &str) -> Vec<usize> {
    let mut v = vec![0usize];
    for (i, b) in src.bytes().enumerate() {
        if b == b'\n' {
            v.push(i + 1);
        }
    }
    v
}

/// 1-based line number containing byte offset `at`.
fn line_of(at: usize, starts: &[usize]) -> usize {
    match starts.binary_search(&at) {
        Ok(i) => i + 1,
        Err(i) => i, // i is the count of starts <= at
    }
}

fn span_lines(sp: Span, starts: &[usize]) -> (usize, usize) {
    let lo = line_of(sp.start, starts);
    let hi = line_of(sp.end.saturating_sub(1).max(sp.start), starts);
    (lo, hi.max(lo))
}

/// The set of source lines that bear at least one lexer token (§7). Comment-only
/// and blank lines carry no token and are excluded.
fn code_lines(src: &str, starts: &[usize]) -> std::collections::BTreeSet<usize> {
    let mut set = std::collections::BTreeSet::new();
    if let Ok(tokens) = crate::lexer::lex(src) {
        for t in &tokens {
            if matches!(t.kind, crate::token::TokKind::Eof) {
                continue;
            }
            let (lo, hi) = span_lines(t.span, starts);
            for ln in lo..=hi {
                set.insert(ln);
            }
        }
    }
    set
}
