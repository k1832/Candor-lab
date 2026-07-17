//! The blessed canonical formatter (P16 / NN#11): AST -> the *one* conforming
//! `.cnr` source form (spec 02 §9; design 0006 §4). Unlike [`super::emit`] (the
//! P15 migrator, which only handles the throwaway-producible AST subset), this
//! prints the **full real AST** — generics, interfaces/impls, the foreign
//! boundary, `use` imports, `pub` visibility, and surface `for` loops — and it
//! **preserves comments**, which the AST does not carry, by re-attaching them
//! from the token stream (see [`Comment`] and [`super::lexer::lex_with_comments`]).
//!
//! Canonical rules realized here are exactly spec 02 §9 / design 0006 §4:
//! 4-space indent, K&R braces, mandatory blocks, one statement per line, the
//! §9.2 spacing, trailing commas in multi-line lists (none in single-line), and
//! the §9.3 normalizations (reborrow collapse, read-only auto-deref collapse,
//! uniform redundant-paren removal, throwaway-spelling rewrite). The precedence-
//! keyed paren logic and the reborrow/auto-deref collapses are shared with the
//! migrator ([`super::emit`]) so the two never diverge.
//!
//! ## Silent-rule decisions (spec 02 §9 is silent; minimal consistent choices,
//! recorded here as future §9 amendments — NN#11):
//!  * **S1 Blank lines.** Runs of blank lines collapse to at most one; a single
//!    blank line between two items/statements/fields is preserved iff the source
//!    had one (detected from spans). No blank line is emitted at a block's top or
//!    bottom edge. (Deterministic and idempotent.)
//!  * **S2 Comment attachment.** A comment on the same source line as, and after,
//!    a node is a *trailing* comment (kept on that node's output line); every
//!    other comment is *standalone* (its own line at the enclosing indent, in
//!    source order). Mid-expression comments are lifted to the next statement
//!    boundary as standalone. No comment is ever dropped.
//!  * **S3 Signature-tail order.** Markers then contracts: `alloc`, `foreign`,
//!    `requires(..)`*, `ensures(..)`* — regardless of source order.
//!  * **S4 Decl-bracket order.** Region parameters (`region r`) precede type
//!    parameters in `[..]` decl brackets, regardless of source order.
//!  * **S5 Redundant `foreign` inside `extern`.** An `extern` block's members are
//!    implicitly `foreign`; the redundant marker is dropped (it is not retained
//!    in the AST). The `foreign` effect on a free `fn` is kept.
//!  * **S6 Empty aggregates.** `struct S {}` / `enum E {}` on one line; a
//!    non-empty aggregate is always multi-line (one field/variant per line).
//!  * **S7 Single-line calls/struct-literals/arg-lists.** Argument, struct-
//!    literal, array, and parameter lists render on one line (no trailing comma);
//!    only field, variant, and match-arm lists are one-per-line (trailing comma).

use crate::ast::*;
use crate::diag::Diag;
use crate::span::Span;

use super::emit::{
    as_deref_inner, bin_bp, bin_sym, scalar_kw, strip_paren, BP_MIN, BP_POSTFIX, BP_PREFIX,
};

/// A source comment, re-attached from the token stream (the AST drops comments).
#[derive(Clone)]
struct Comment {
    start: usize,
    end: usize,
    /// Verbatim comment text (`// ...` or `/* ... */`), trailing whitespace trimmed.
    text: String,
    /// True when only whitespace precedes the comment on its source line (it is a
    /// standalone comment); false when code precedes it (a trailing comment).
    own_line: bool,
}

/// Format a whole real-syntax (`.cnr`) source string into canonical form.
/// Lexes with comment capture, parses (keeping surface `for`), and prints.
pub fn format_source(src: &str) -> Result<String, Diag> {
    let (tokens, comment_spans) = super::lexer::lex_with_comments(src)?;
    let (program, uses, vis, boundary) = super::parser::parse_format(tokens)?;
    let comments = build_comments(src, &comment_spans);
    let mut f = Fmt { buf: String::new(), indent: 0, src, comments, ci: 0, last_pos: 0 };
    f.emit_unit(&program, &uses, &vis, boundary);
    Ok(f.buf)
}

fn build_comments(src: &str, spans: &[Span]) -> Vec<Comment> {
    let bytes = src.as_bytes();
    spans
        .iter()
        .map(|s| {
            // own_line: scan back from `start` over spaces/tabs to a newline or BOF.
            let mut i = s.start;
            let mut own = true;
            while i > 0 {
                let b = bytes[i - 1];
                if b == b'\n' {
                    break;
                }
                if b != b' ' && b != b'\t' && b != b'\r' {
                    own = false;
                    break;
                }
                i -= 1;
            }
            let text = src[s.start..s.end].trim_end().to_string();
            Comment { start: s.start, end: s.end, text, own_line: own }
        })
        .collect()
}

/// A merged top-level entry, in source order (`use` imports interleave with items).
enum Top<'a> {
    Use(&'a UseDecl),
    Item(&'a Item, bool), // (item, is_pub)
}

fn top_span(t: &Top) -> Span {
    match t {
        Top::Use(u) => u.span,
        Top::Item(it, _) => item_span(it),
    }
}

fn item_span(it: &Item) -> Span {
    match it {
        Item::Struct(s) => s.span,
        Item::Enum(e) => e.span,
        Item::Fn(f) => f.span,
        Item::Static(s) => s.span,
        Item::Interface(i) => i.span,
        Item::Impl(i) => i.span,
        Item::Extern(e) => e.span,
        Item::Export(e) => e.span,
    }
}

struct Fmt<'a> {
    buf: String,
    indent: usize,
    src: &'a str,
    comments: Vec<Comment>,
    ci: usize,
    last_pos: usize,
}

impl Fmt<'_> {
    fn push(&mut self, s: &str) {
        self.buf.push_str(s);
    }
    fn write_indent(&mut self, indent: usize) {
        for _ in 0..indent {
            self.buf.push_str("    ");
        }
    }
    fn indent_str(&mut self) {
        let n = self.indent;
        self.write_indent(n);
    }

    fn count_nl(&self, a: usize, b: usize) -> usize {
        if a >= b {
            return 0;
        }
        self.src.as_bytes()[a..b.min(self.src.len())].iter().filter(|&&c| c == b'\n').count()
    }

    /// Emit a single blank line if the source gap [`last_pos`, pos) held one
    /// (silent rule S1); never at a block edge or as a double blank.
    fn blank_if_gap(&mut self, pos: usize) {
        if self.buf.is_empty() || self.buf.ends_with("{\n") || self.buf.ends_with("\n\n") {
            return;
        }
        if self.count_nl(self.last_pos, pos) >= 2 {
            self.buf.push('\n');
        }
    }

    /// Flush every pending comment whose start is before `before`, each on its
    /// own line at `indent` (silent rule S2). Guarantees no comment is dropped.
    fn flush_before(&mut self, before: usize, indent: usize) {
        while self.ci < self.comments.len() && self.comments[self.ci].start < before {
            let c = self.comments[self.ci].clone();
            self.blank_if_gap(c.start);
            self.write_indent(indent);
            self.push(&c.text);
            self.buf.push('\n');
            self.last_pos = c.end;
            self.ci += 1;
        }
    }

    /// If the next pending comment sits on the same source line as (and after) a
    /// node ending at `after`, attach it as a trailing comment (S2). Call right
    /// after emitting the node's content, before its newline.
    fn attach_trailing(&mut self, after: usize) {
        while self.ci < self.comments.len() {
            let c = &self.comments[self.ci];
            if !c.own_line && c.start >= after && self.count_nl(after, c.start) == 0 {
                let text = c.text.clone();
                let end = c.end;
                self.push(" ");
                self.push(&text);
                self.last_pos = end;
                self.ci += 1;
            } else {
                break;
            }
        }
    }

    // ----- top level ------------------------------------------------------
    fn emit_unit(&mut self, program: &Program, uses: &[UseDecl], vis: &[bool], boundary: bool) {
        // Merge uses and items into one source-ordered stream.
        let mut tops: Vec<Top> = Vec::new();
        for u in uses {
            tops.push(Top::Use(u));
        }
        for (i, it) in program.items.iter().enumerate() {
            tops.push(Top::Item(it, vis.get(i).copied().unwrap_or(false)));
        }
        tops.sort_by_key(|t| top_span(t).start);

        if boundary {
            self.push("boundary\n");
            // The `boundary` keyword has no AST span; treat the first entry's
            // leading gap as following it.
            self.last_pos = 0;
        }

        for t in &tops {
            let sp = top_span(t);
            self.flush_before(sp.start, 0);
            self.blank_if_gap(sp.start);
            match t {
                Top::Use(u) => self.emit_use(u),
                Top::Item(it, is_pub) => self.emit_item(it, *is_pub),
            }
            self.last_pos = sp.end;
            self.attach_trailing(sp.end);
            self.buf.push('\n');
        }
        // Trailing comments at end of file.
        self.flush_before(usize::MAX, 0);
        if !self.buf.ends_with('\n') {
            self.buf.push('\n');
        }
    }

    fn emit_use(&mut self, u: &UseDecl) {
        self.push("use ");
        self.push(&u.segments.join("::"));
        if let Some(names) = &u.names {
            self.push("::{");
            self.push(&names.join(", "));
            self.push("}");
        }
        self.push(";");
    }

    fn emit_item(&mut self, item: &Item, is_pub: bool) {
        if is_pub {
            self.push("pub ");
        }
        match item {
            Item::Struct(s) => self.emit_struct(s),
            Item::Enum(e) => self.emit_enum(e),
            Item::Fn(f) => self.emit_fn(f),
            Item::Static(s) => self.emit_static(s),
            Item::Interface(i) => self.emit_interface(i),
            Item::Impl(i) => self.emit_impl(i),
            Item::Extern(e) => self.emit_extern(e),
            Item::Export(e) => self.emit_export(e),
        }
    }

    // ----- type parameters / decl brackets --------------------------------
    fn emit_type_params(&mut self, tps: &[TypeParam]) {
        if tps.is_empty() {
            return;
        }
        self.push("[");
        for (i, tp) in tps.iter().enumerate() {
            if i > 0 {
                self.push(", ");
            }
            self.emit_type_param(tp);
        }
        self.push("]");
    }

    fn emit_type_param(&mut self, tp: &TypeParam) {
        self.push(&tp.name);
        if !tp.bounds.is_empty() {
            self.push(": ");
            self.push(&tp.bounds.join(" + "));
        }
    }

    /// Combined `[region r, .., T, ..]` decl bracket for a fn (S4: regions first).
    fn emit_fn_brackets(&mut self, regions: &[String], tps: &[TypeParam]) {
        if regions.is_empty() && tps.is_empty() {
            return;
        }
        self.push("[");
        let mut first = true;
        for r in regions {
            if !first {
                self.push(", ");
            }
            first = false;
            self.push("region ");
            self.push(r);
        }
        for tp in tps {
            if !first {
                self.push(", ");
            }
            first = false;
            self.emit_type_param(tp);
        }
        self.push("]");
    }

    // ----- items ----------------------------------------------------------
    fn emit_struct(&mut self, s: &StructDecl) {
        if s.copy {
            self.push("copy ");
        }
        self.push("struct ");
        self.push(&s.name);
        self.emit_type_params(&s.type_params);
        if s.fields.is_empty() {
            self.push(" {}");
        } else {
            self.push(" {\n");
            self.indent += 1;
            self.last_pos = s.fields[0].span.start;
            for f in &s.fields {
                self.flush_before(f.span.start, self.indent);
                self.blank_if_gap(f.span.start);
                self.indent_str();
                self.push(&f.name);
                self.push(": ");
                self.emit_type(&f.ty);
                self.push(",");
                self.last_pos = f.span.end;
                self.attach_trailing(f.span.end);
                self.push("\n");
            }
            let close = s.drop_hook.as_ref().map(|h| h.span.start).unwrap_or(s.span.end);
            self.flush_before(close, self.indent);
            self.indent -= 1;
            self.indent_str();
            self.push("}");
        }
        if let Some(hook) = &s.drop_hook {
            self.push(" drop(write self) ");
            self.emit_block(hook);
        }
    }

    fn emit_enum(&mut self, e: &EnumDecl) {
        if e.copy {
            self.push("copy ");
        }
        self.push("enum ");
        self.push(&e.name);
        self.emit_type_params(&e.type_params);
        if e.variants.is_empty() {
            self.push(" {}");
            return;
        }
        self.push(" {\n");
        self.indent += 1;
        self.last_pos = e.variants[0].span.start;
        for v in &e.variants {
            self.flush_before(v.span.start, self.indent);
            self.blank_if_gap(v.span.start);
            self.indent_str();
            if v.ok {
                self.push("ok ");
            }
            self.push(&v.name);
            if !v.payload.is_empty() {
                self.push("(");
                for (i, t) in v.payload.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.emit_type(t);
                }
                self.push(")");
            }
            self.push(",");
            self.last_pos = v.span.end;
            self.attach_trailing(v.span.end);
            self.push("\n");
        }
        self.flush_before(e.span.end, self.indent);
        self.indent -= 1;
        self.indent_str();
        self.push("}");
    }

    fn emit_static(&mut self, s: &StaticDecl) {
        self.push("static ");
        self.push(&s.name);
        self.push(": ");
        self.emit_type(&s.ty);
        self.push(" = ");
        self.emit_expr(&s.value, BP_MIN, false);
        self.push(";");
    }

    fn emit_fn(&mut self, f: &FnDecl) {
        self.push("fn ");
        self.push(&f.name);
        self.emit_fn_brackets(&f.regions, &f.type_params);
        self.emit_params(&f.params);
        self.emit_sig_tail(f.alloc, f.foreign, &f.requires, &f.ensures);
        if let Some(ret) = &f.ret {
            self.push(" -> ");
            self.emit_ret_ty(ret);
        }
        self.push(" ");
        self.emit_block(&f.body);
    }

    /// Canonical signature tail (S3): `alloc`, `foreign`, then contracts.
    fn emit_sig_tail(&mut self, alloc: bool, foreign: bool, requires: &[Expr], ensures: &[Expr]) {
        if alloc {
            self.push(" alloc");
        }
        if foreign {
            self.push(" foreign");
        }
        for r in requires {
            self.push(" requires(");
            self.emit_expr(r, BP_MIN, false);
            self.push(")");
        }
        for en in ensures {
            self.push(" ensures(");
            self.emit_expr(en, BP_MIN, false);
            self.push(")");
        }
    }

    fn emit_params(&mut self, params: &[Param]) {
        self.push("(");
        for (i, p) in params.iter().enumerate() {
            if i > 0 {
                self.push(", ");
            }
            self.push(&p.name);
            self.push(": ");
            self.emit_mode_type(p.mode, p.region.as_deref(), &p.ty);
        }
        self.push(")");
    }

    fn emit_interface(&mut self, it: &InterfaceDecl) {
        self.push("interface ");
        self.push(&it.name);
        self.emit_type_params(&it.type_params);
        self.push(" {\n");
        self.indent += 1;
        if let Some(assoc) = &it.assoc_type {
            self.indent_str();
            self.push("type ");
            self.push(assoc);
            self.push(";\n");
        }
        if let Some(m0) = it.methods.first() {
            self.last_pos = m0.span.start;
        }
        for m in &it.methods {
            self.flush_before(m.span.start, self.indent);
            self.blank_if_gap(m.span.start);
            self.indent_str();
            self.emit_method_sig(m);
            self.last_pos = m.span.end;
            self.attach_trailing(m.span.end);
            self.push("\n");
        }
        self.flush_before(it.span.end, self.indent);
        self.indent -= 1;
        self.indent_str();
        self.push("}");
    }

    fn emit_method_sig(&mut self, m: &MethodSig) {
        self.push("fn ");
        self.push(&m.name);
        self.push("(");
        let mut first = true;
        if m.has_self {
            self.push(&self_receiver(m.self_mode));
            first = false;
        }
        for p in &m.params {
            if !first {
                self.push(", ");
            }
            first = false;
            self.push(&p.name);
            self.push(": ");
            self.emit_mode_type(p.mode, p.region.as_deref(), &p.ty);
        }
        self.push(")");
        if m.alloc {
            self.push(" alloc");
        }
        if let Some(ret) = &m.ret {
            self.push(" -> ");
            self.emit_ret_ty(ret);
        }
        self.push(";");
    }

    fn emit_impl(&mut self, im: &ImplDecl) {
        self.push("impl");
        self.emit_type_params(&im.type_params);
        self.push(" ");
        self.push(&im.iface);
        if !im.iface_args.is_empty() {
            self.push("[");
            for (i, a) in im.iface_args.iter().enumerate() {
                if i > 0 {
                    self.push(", ");
                }
                self.emit_type(a);
            }
            self.push("]");
        }
        self.push(" for ");
        self.emit_type(&im.target);
        self.push(" {\n");
        self.indent += 1;
        if let Some((name, ty)) = &im.assoc_binding {
            self.indent_str();
            self.push("type ");
            self.push(name);
            self.push(" = ");
            self.emit_type(ty);
            self.push(";\n");
        }
        if let Some(m0) = im.methods.first() {
            self.last_pos = m0.span.start;
        }
        for (i, m) in im.methods.iter().enumerate() {
            self.flush_before(m.span.start, self.indent);
            self.blank_if_gap(m.span.start);
            self.indent_str();
            self.emit_impl_method(m);
            self.last_pos = m.span.end;
            self.attach_trailing(m.span.end);
            self.push("\n");
            let _ = i;
        }
        self.flush_before(im.span.end, self.indent);
        self.indent -= 1;
        self.indent_str();
        self.push("}");
    }

    /// An impl method prints like a free `fn`, but its leading `self` parameter
    /// is rendered as the receiver (`read self`, ...), not as `self: Self`.
    fn emit_impl_method(&mut self, f: &FnDecl) {
        self.push("fn ");
        self.push(&f.name);
        self.emit_fn_brackets(&f.regions, &f.type_params);
        self.push("(");
        let mut idx = 0;
        let mut first = true;
        if let Some(p0) = f.params.first() {
            if p0.name == "self" {
                self.push(&self_receiver(p0.mode));
                first = false;
                idx = 1;
            }
        }
        for p in &f.params[idx..] {
            if !first {
                self.push(", ");
            }
            first = false;
            self.push(&p.name);
            self.push(": ");
            self.emit_mode_type(p.mode, p.region.as_deref(), &p.ty);
        }
        self.push(")");
        self.emit_sig_tail(f.alloc, f.foreign, &f.requires, &f.ensures);
        if let Some(ret) = &f.ret {
            self.push(" -> ");
            self.emit_ret_ty(ret);
        }
        self.push(" ");
        self.emit_block(&f.body);
    }

    fn emit_extern(&mut self, e: &ExternBlock) {
        self.push("extern ");
        self.emit_string(&e.abi);
        self.push(" {\n");
        self.indent += 1;
        if let Some(f0) = e.fns.first() {
            self.last_pos = f0.span.start;
        }
        for xf in &e.fns {
            self.flush_before(xf.span.start, self.indent);
            self.blank_if_gap(xf.span.start);
            self.indent_str();
            self.emit_extern_fn(xf);
            self.last_pos = xf.span.end;
            self.attach_trailing(xf.span.end);
            self.push("\n");
        }
        self.flush_before(e.span.end, self.indent);
        self.indent -= 1;
        self.indent_str();
        self.push("}");
    }

    fn emit_extern_fn(&mut self, xf: &ExternFn) {
        self.push("fn ");
        self.push(&xf.name);
        self.emit_params(&xf.params);
        if let Some(ret) = &xf.ret {
            self.push(" -> ");
            self.emit_ret_ty(ret);
        }
        if let Some(t) = &xf.trust {
            self.emit_trust(t);
        }
        self.push(";");
    }

    fn emit_trust(&mut self, t: &TrustDecl) {
        self.push("\n");
        self.indent += 1;
        self.indent_str();
        self.push("trust ");
        self.emit_string(&t.justification);
        if t.predicates.is_empty() {
            self.push(" {}");
        } else {
            self.push(" {\n");
            self.indent += 1;
            for p in &t.predicates {
                self.indent_str();
                self.push(&p.name);
                if !p.args.is_empty() {
                    self.push("(");
                    self.push(&p.args.join(", "));
                    self.push(")");
                }
                self.push(",\n");
            }
            self.indent -= 1;
            self.indent_str();
            self.push("}");
        }
        self.indent -= 1;
    }

    fn emit_export(&mut self, e: &ExportDecl) {
        self.push("export ");
        self.emit_string(&e.abi);
        self.push(" fn ");
        self.push(&e.symbol);
        self.emit_params(&e.params);
        if let Some(ret) = &e.ret {
            self.push(" -> ");
            self.emit_ret_ty(ret);
        }
        self.push(" = ");
        self.push(&e.candor_fn);
        self.push(";");
    }

    // ----- signature pieces -----------------------------------------------
    fn emit_mode_type(&mut self, mode: ParamMode, region: Option<&str>, ty: &Ty) {
        match mode {
            ParamMode::Take => self.emit_type(ty),
            ParamMode::Out => {
                self.push("out ");
                self.emit_type(ty);
            }
            ParamMode::Read => {
                self.push("read");
                self.emit_region(region);
                self.push(" ");
                self.emit_type(ty);
            }
            ParamMode::Write => {
                self.push("write");
                self.emit_region(region);
                self.push(" ");
                self.emit_type(ty);
            }
        }
    }

    fn emit_ret_ty(&mut self, ret: &RetTy) {
        match ret.borrow {
            None => self.emit_type(&ret.ty),
            Some(BorrowKind::Shared) => {
                self.push("read");
                self.emit_region(ret.region.as_deref());
                self.push(" ");
                self.emit_type(&ret.ty);
            }
            Some(BorrowKind::Exclusive) => {
                self.push("write");
                self.emit_region(ret.region.as_deref());
                self.push(" ");
                self.emit_type(&ret.ty);
            }
        }
    }

    fn emit_region(&mut self, region: Option<&str>) {
        if let Some(r) = region {
            self.push("[");
            self.push(r);
            self.push("]");
        }
    }

    // ----- types ----------------------------------------------------------
    fn emit_type(&mut self, ty: &Ty) {
        match &ty.kind {
            TyKind::Scalar(sc) => self.push(scalar_kw(*sc)),
            TyKind::Named(n) => self.push(n),
            TyKind::App { name, args } => {
                self.push(name);
                self.push("[");
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.emit_type(a);
                }
                self.push("]");
            }
            TyKind::Proj { base, assoc } => {
                self.push(base);
                self.push("::");
                self.push(assoc);
            }
            TyKind::Slice(t) => {
                self.push("[");
                self.emit_type(t);
                self.push("]");
            }
            TyKind::SliceMut(t) => {
                self.push("write [");
                self.emit_type(t);
                self.push("]");
            }
            TyKind::Borrow(t) => {
                self.push("read ");
                self.emit_type(t);
            }
            TyKind::BorrowMut(t) => {
                self.push("write ");
                self.emit_type(t);
            }
            TyKind::RawPtr(t) => {
                self.push("rawptr ");
                self.emit_type(t);
            }
            TyKind::Box(t) => {
                self.push("Box ");
                self.emit_type(t);
            }
            TyKind::BoxResult(t) => {
                self.push("BoxResult ");
                self.emit_type(t);
            }
            TyKind::Array { size, elem } => {
                self.push("[");
                self.emit_expr(size, BP_MIN, false);
                self.push("]");
                self.emit_type(elem);
            }
            TyKind::FnPtr(fp) => {
                self.push("fn(");
                for (i, p) in fp.params.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    if let Some(n) = &p.name {
                        self.push(n);
                        self.push(": ");
                    }
                    self.emit_mode_type(p.mode, p.region.as_deref(), &p.ty);
                }
                self.push(")");
                if fp.alloc {
                    self.push(" alloc");
                }
                self.push(" -> ");
                self.emit_type(&fp.ret);
            }
        }
    }

    // ----- blocks & statements --------------------------------------------
    fn emit_block(&mut self, b: &Block) {
        // Flush comments strictly inside a block with no statements.
        let has_inner = self.ci < self.comments.len()
            && self.comments[self.ci].start > b.span.start
            && self.comments[self.ci].start < b.span.end;
        if b.stmts.is_empty() && !has_inner {
            self.push("{}");
            self.last_pos = b.span.end;
            return;
        }
        self.push("{\n");
        self.indent += 1;
        if let Some(s0) = b.stmts.first() {
            self.last_pos = s0.span.start;
        }
        for st in &b.stmts {
            self.flush_before(st.span.start, self.indent);
            self.blank_if_gap(st.span.start);
            self.indent_str();
            self.emit_stmt(st);
            self.last_pos = st.span.end;
            self.attach_trailing(st.span.end);
            self.push("\n");
        }
        self.flush_before(b.span.end, self.indent);
        self.indent -= 1;
        self.indent_str();
        self.push("}");
        self.last_pos = b.span.end;
    }

    fn emit_stmt(&mut self, s: &Stmt) {
        match &s.kind {
            StmtKind::Let { mutable, name, ty, init } => {
                self.push("let ");
                if *mutable {
                    self.push("mut ");
                }
                self.push(name);
                if let Some(t) = ty {
                    self.push(": ");
                    self.emit_type(t);
                }
                if let Some(e) = init {
                    self.push(" = ");
                    self.emit_expr(e, BP_MIN, false);
                }
                self.push(";");
            }
            StmtKind::Assign { target, value } => {
                self.emit_expr(target, BP_MIN, true); // LHS write path: keep every `.*`
                self.push(" = ");
                self.emit_expr(value, BP_MIN, false);
                self.push(";");
            }
            StmtKind::Expr(e) => {
                let inner = strip_paren(e);
                if is_block_like(&inner.kind) {
                    self.emit_block_like(inner); // block-like statement: no `;`
                } else {
                    self.emit_expr(e, BP_MIN, false);
                    self.push(";");
                }
            }
        }
    }

    // ----- expressions (precedence per spec 02 §6.1) ----------------------
    fn emit_expr(&mut self, e: &Expr, parent_bp: u32, wp: bool) {
        let e = strip_paren(e);
        let bp = self.expr_bp(e);
        let paren = bp < parent_bp;
        if paren {
            self.push("(");
        }
        self.emit_expr_bare(e, wp);
        if paren {
            self.push(")");
        }
    }

    fn expr_bp(&self, e: &Expr) -> u32 {
        match &e.kind {
            ExprKind::Binary { op, .. } => bin_bp(*op),
            ExprKind::Unary { .. } | ExprKind::Conv { .. } | ExprKind::Bitcast { .. } => BP_PREFIX,
            ExprKind::Prefix { op, expr } => match op {
                PrefixOp::Deref => BP_POSTFIX,
                PrefixOp::Clone => BP_PREFIX,
                PrefixOp::Read | PrefixOp::Write => {
                    if as_deref_inner(expr).is_some() {
                        BP_POSTFIX
                    } else {
                        BP_PREFIX
                    }
                }
            },
            ExprKind::Try(_)
            | ExprKind::Call { .. }
            | ExprKind::Field { .. }
            | ExprKind::Index { .. }
            | ExprKind::EnumCtor { .. }
            | ExprKind::StructLit { .. }
            | ExprKind::GenericVal { .. }
            | ExprKind::CastPtr { .. }
            | ExprKind::AddrToPtr { .. }
            | ExprKind::PtrNull { .. }
            | ExprKind::Offsetof { .. }
            | ExprKind::FieldPtr { .. }
            | ExprKind::Sizeof(_)
            | ExprKind::Alignof(_) => BP_POSTFIX,
            _ => super::emit::BP_ATOM,
        }
    }

    fn emit_expr_bare(&mut self, e: &Expr, wp: bool) {
        match &e.kind {
            ExprKind::IntLit { value, suffix } => {
                self.push(&value.to_string());
                if let Some(s) = suffix {
                    self.push(scalar_kw(*s));
                }
            }
            ExprKind::NegIntLit { value, suffix } => {
                self.push("-");
                self.push(&value.to_string());
                if let Some(s) = suffix {
                    self.push(scalar_kw(*s));
                }
            }
            ExprKind::StrLit(s) => self.emit_string(s),
            ExprKind::BytesLit(s) => {
                self.push("b");
                self.emit_string(s);
            }
            ExprKind::BoolLit(b) => self.push(if *b { "true" } else { "false" }),
            ExprKind::Ident(n) => self.push(n),
            ExprKind::Result => self.push("result"),
            ExprKind::Spawn(c) => {
                self.push("spawn ");
                self.emit_expr(c, BP_MIN, false);
            }
            ExprKind::Break => self.push("break"),
            ExprKind::Continue => self.push("continue"),
            ExprKind::Return(opt) => {
                self.push("return");
                if let Some(inner) = opt {
                    self.push(" ");
                    self.emit_expr(inner, BP_MIN, false);
                }
            }
            ExprKind::Assert(inner) => {
                self.push("assert(");
                self.emit_expr(inner, BP_MIN, false);
                self.push(")");
            }
            ExprKind::Panic(inner) => {
                self.push("panic(");
                self.emit_expr(inner, BP_MIN, false);
                self.push(")");
            }
            ExprKind::Unary { op, expr } => {
                self.push(match op {
                    UnOp::Neg => "-",
                    UnOp::Not => "!",
                    UnOp::BitNot => "~",
                });
                self.emit_expr(expr, BP_PREFIX, false);
            }
            ExprKind::Conv { ty, expr } => {
                self.push("conv ");
                self.emit_type(ty);
                self.push(" ");
                self.emit_expr(expr, BP_PREFIX, false);
            }
            ExprKind::Bitcast { ty, expr } => {
                self.push("bitcast ");
                self.emit_type(ty);
                self.push(" ");
                self.emit_expr(expr, BP_PREFIX, false);
            }
            ExprKind::Prefix { op, expr } => match op {
                PrefixOp::Deref => {
                    self.emit_expr(expr, BP_POSTFIX, wp);
                    self.push(".*");
                }
                PrefixOp::Clone => {
                    self.push("clone ");
                    self.emit_expr(expr, BP_PREFIX, false);
                }
                PrefixOp::Read => self.emit_borrow_op("read", expr),
                PrefixOp::Write => self.emit_borrow_op("write", expr),
            },
            ExprKind::Binary { op, lhs, rhs } => {
                let obp = bin_bp(*op);
                self.emit_expr(lhs, obp, false);
                self.push(" ");
                self.push(bin_sym(*op));
                self.push(" ");
                self.emit_expr(rhs, obp + 1, false);
            }
            ExprKind::Try(inner) => {
                self.emit_expr(inner, BP_POSTFIX, false);
                self.push("?");
            }
            ExprKind::Call { callee, args } => {
                self.emit_expr(callee, BP_POSTFIX, false);
                self.push("(");
                self.emit_args(args);
                self.push(")");
            }
            ExprKind::OutArg(place) => {
                self.push("out ");
                self.emit_expr(place, BP_MIN, false);
            }
            ExprKind::Field { base, field, .. } => {
                self.emit_place_base(base, wp);
                self.push(".");
                self.push(field);
            }
            ExprKind::Index { base, index } => {
                self.emit_place_base(base, wp);
                self.push("[");
                self.emit_expr(index, BP_MIN, false);
                self.push("]");
            }
            ExprKind::EnumCtor { enum_name, variant, args } => {
                self.push(enum_name);
                self.push("::");
                self.push(variant);
                if !args.is_empty() {
                    self.push("(");
                    self.emit_args(args);
                    self.push(")");
                }
            }
            ExprKind::GenericVal { name, ty_args } => {
                self.push(name);
                self.push("::[");
                for (i, t) in ty_args.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.emit_type(t);
                }
                self.push("]");
            }
            ExprKind::StructLit { name, fields } => {
                self.push(name);
                if fields.is_empty() {
                    self.push(" {}");
                } else {
                    self.push(" { ");
                    for (i, f) in fields.iter().enumerate() {
                        if i > 0 {
                            self.push(", ");
                        }
                        self.push(&f.name);
                        self.push(": ");
                        self.emit_expr(&f.value, BP_MIN, false);
                    }
                    self.push(" }");
                }
            }
            ExprKind::ArrayLit(elems) => {
                self.push("[");
                for (i, el) in elems.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.emit_expr(el, BP_MIN, false);
                }
                self.push("]");
            }
            ExprKind::ArrayRepeat { value, size } => {
                self.push("[");
                self.emit_expr(value, BP_MIN, false);
                self.push("; ");
                self.emit_expr(size, BP_MIN, false);
                self.push("]");
            }
            ExprKind::CastPtr { ty, arg } => self.emit_type_arg_call("cast_ptr", ty, Some(arg)),
            ExprKind::AddrToPtr { ty, arg } => self.emit_type_arg_call("addr_to_ptr", ty, Some(arg)),
            ExprKind::PtrNull { ty } => self.emit_type_arg_call("ptr_null", ty, None),
            ExprKind::Offsetof { ty, field } => {
                self.push("offsetof(");
                self.emit_type(ty);
                self.push(", ");
                self.push(field);
                self.push(")");
            }
            ExprKind::FieldPtr { ptr, field } => {
                self.push("field_ptr(");
                self.emit_expr(ptr, BP_MIN, false);
                self.push(", ");
                self.push(field);
                self.push(")");
            }
            ExprKind::Sizeof(ty) => {
                self.push("sizeof(");
                self.emit_type(ty);
                self.push(")");
            }
            ExprKind::Alignof(ty) => {
                self.push("alignof(");
                self.emit_type(ty);
                self.push(")");
            }
            ExprKind::Paren(inner) => self.emit_expr_bare(strip_paren(inner), wp),
            _ if is_block_like(&e.kind) => self.emit_block_like(e),
            _ => unreachable!("unhandled expression kind in formatter"),
        }
    }

    fn emit_place_base(&mut self, base: &Expr, wp: bool) {
        if let Some(inner) = as_deref_inner(base) {
            self.emit_expr(inner, BP_POSTFIX, wp);
            if wp {
                self.push(".*");
            }
        } else {
            self.emit_expr(base, BP_POSTFIX, wp);
        }
    }

    /// A `read`/`write` borrow *operator*. The design-0005 reborrow collapse
    /// (`read (b.*)` -> bare `b`, spec 02 §9.3) is **type-aware**: it is only sound
    /// when `b` is a *read* (shared/copy) borrow, where `read (b.*)` and bare `b`
    /// denote the same non-moving reborrow. When `b` is a *write* (exclusive, non-
    /// copy) borrow, `read (b.*)` is a non-moving read-reborrow while bare `b`
    /// **moves** the write borrow -- so the collapse changes semantics (a later use
    /// of `b` then fails E0301 "use of moved value"). The formatter has no type
    /// table and cannot tell a read borrow (collapse safe) from a write borrow
    /// (collapse unsound), so it must NOT collapse: the keyword and `.*` are always
    /// preserved. Redundant parens still drop, since `.*` binds tighter than the
    /// borrow operator (`read (b.*)` prints as `read b.*`).
    ///
    /// Follow-up: a future *type-aware* fmt (with a type table) could restore the
    /// collapse for the read-borrow-only case, which is genuinely behaviour-
    /// preserving. Do not reintroduce it without that type information.
    fn emit_borrow_op(&mut self, kw: &str, operand: &Expr) {
        self.push(kw);
        self.push(" ");
        self.emit_expr(operand, BP_PREFIX, false);
    }

    fn emit_type_arg_call(&mut self, name: &str, ty: &Ty, arg: Option<&Expr>) {
        self.push(name);
        self.push("[");
        self.emit_type(ty);
        self.push("](");
        if let Some(a) = arg {
            self.emit_expr(a, BP_MIN, false);
        }
        self.push(")");
    }

    fn emit_args(&mut self, args: &[Expr]) {
        for (i, a) in args.iter().enumerate() {
            if i > 0 {
                self.push(", ");
            }
            self.emit_expr(a, BP_MIN, false);
        }
    }

    // ----- block-like expressions -----------------------------------------
    fn emit_block_like(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::Block(b) => self.emit_block(b),
            ExprKind::If { cond, then_blk, else_blk } => {
                self.push("if ");
                self.emit_head(cond);
                self.push(" ");
                self.emit_block(then_blk);
                if let Some(else_e) = else_blk {
                    self.push(" else ");
                    let inner = strip_paren(else_e);
                    match &inner.kind {
                        ExprKind::Block(b) => self.emit_block(b),
                        _ => self.emit_block_like(inner),
                    }
                }
            }
            ExprKind::Match { scrutinee, arms } => {
                self.push("match ");
                self.emit_head(scrutinee);
                self.push(" {\n");
                self.indent += 1;
                if let Some(a0) = arms.first() {
                    self.last_pos = a0.span.start;
                }
                for arm in arms {
                    self.flush_before(arm.span.start, self.indent);
                    self.blank_if_gap(arm.span.start);
                    self.indent_str();
                    self.emit_pattern(&arm.pattern);
                    if let Some(guard) = &arm.guard {
                        self.push(" if ");
                        self.emit_head(guard);
                    }
                    self.push(" => ");
                    let body = strip_paren(&arm.body);
                    if is_block_like(&body.kind) {
                        self.emit_block_like(body);
                    } else {
                        self.emit_expr(&arm.body, BP_MIN, false);
                    }
                    self.push(",");
                    self.last_pos = arm.span.end;
                    self.attach_trailing(arm.span.end);
                    self.push("\n");
                }
                self.flush_before(e.span.end, self.indent);
                self.indent -= 1;
                self.indent_str();
                self.push("}");
            }
            ExprKind::For { pattern, operand, body, by_ref } => {
                self.push("for ");
                if *by_ref { self.push("read "); }
                self.emit_pattern(pattern);
                self.push(" in ");
                self.emit_head(operand);
                self.push(" ");
                self.emit_block(body);
            }
            ExprKind::Loop(b) => {
                self.push("loop ");
                self.emit_block(b);
            }
            ExprKind::Scope(b) => {
                self.push("scope ");
                self.emit_block(b);
            }
            ExprKind::While { cond, body } => {
                self.push("while ");
                self.emit_head(cond);
                self.push(" ");
                self.emit_block(body);
            }
            ExprKind::Unsafe { justification, body } => {
                self.push("unsafe ");
                self.emit_string(justification);
                self.push(" ");
                self.emit_block(body);
            }
            ExprKind::Wrapping(b) => {
                self.push("wrapping ");
                self.emit_block(b);
            }
            ExprKind::Saturating(b) => {
                self.push("saturating ");
                self.emit_block(b);
            }
            _ => unreachable!("emit_block_like on a non-block-like expression"),
        }
    }

    fn emit_head(&mut self, e: &Expr) {
        let inner = strip_paren(e);
        if matches!(inner.kind, ExprKind::StructLit { .. }) {
            self.push("(");
            self.emit_expr(inner, BP_MIN, false);
            self.push(")");
        } else {
            self.emit_expr(e, BP_MIN, false);
        }
    }

    fn emit_pattern(&mut self, p: &Pattern) {
        match &p.kind {
            PatKind::Wildcard => self.push("_"),
            PatKind::Binding(n) => self.push(n),
            PatKind::Variant { enum_name, variant, sub } => {
                self.push(enum_name);
                self.push("::");
                self.push(variant);
                if !sub.is_empty() {
                    self.push("(");
                    for (i, s) in sub.iter().enumerate() {
                        if i > 0 {
                            self.push(", ");
                        }
                        self.emit_pattern(s);
                    }
                    self.push(")");
                }
            }
            PatKind::IntLit { value, negative, suffix } => {
                if *negative {
                    self.push("-");
                }
                self.push(&value.to_string());
                if let Some(sc) = suffix {
                    self.push(scalar_kw(*sc));
                }
            }
            PatKind::IntRange { lo_value, lo_negative, lo_suffix, hi_value, hi_negative, hi_suffix, inclusive } => {
                if *lo_negative {
                    self.push("-");
                }
                self.push(&lo_value.to_string());
                if let Some(sc) = lo_suffix {
                    self.push(scalar_kw(*sc));
                }
                self.push(if *inclusive { "..=" } else { ".." });
                if *hi_negative {
                    self.push("-");
                }
                self.push(&hi_value.to_string());
                if let Some(sc) = hi_suffix {
                    self.push(scalar_kw(*sc));
                }
            }
        }
    }

    fn emit_string(&mut self, s: &str) {
        self.push("\"");
        for c in s.chars() {
            match c {
                '"' => self.push("\\\""),
                '\\' => self.push("\\\\"),
                '\n' => self.push("\\n"),
                '\t' => self.push("\\t"),
                _ => self.buf.push(c),
            }
        }
        self.push("\"");
    }
}

fn self_receiver(mode: ParamMode) -> String {
    match mode {
        ParamMode::Take => "self".to_string(),
        ParamMode::Read => "read self".to_string(),
        ParamMode::Write => "write self".to_string(),
        ParamMode::Out => "out self".to_string(),
    }
}

fn is_block_like(kind: &ExprKind) -> bool {
    matches!(
        kind,
        ExprKind::Block(_)
            | ExprKind::If { .. }
            | ExprKind::Match { .. }
            | ExprKind::For { .. }
            | ExprKind::Scope(_)
            | ExprKind::Loop(_)
            | ExprKind::While { .. }
            | ExprKind::Unsafe { .. }
            | ExprKind::Wrapping(_)
            | ExprKind::Saturating(_)
    )
}
