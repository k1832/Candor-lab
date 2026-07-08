//! The module tree (design 0008 §1, §3) — prototype stage 1.
//!
//! Maps a directory of real-syntax (`.cnr`) files onto design 0008's unit rule
//! (**file = module, directory = namespace**), resolves `use` imports across
//! files, enforces `pub` visibility and an acyclic import DAG, then **merges the
//! whole tree into one single-program AST** by qualifying every top-level name
//! with its module path. That merged [`Program`] is fed unchanged to the
//! existing resolver / checker / interpreter — the module system is a front
//! layer, not a checker change.
//!
//! Conventions (documented, no configuration — P16):
//! * A file `foo.cnr` directly under the root is the module `foo`; a file
//!   `bar/baz.cnr` is the module `bar::baz`; a subdirectory is a namespace.
//! * The **root module is `main.cnr`**, and the program entry `fn main` must be
//!   defined there. Its `fn main` keeps the un-mangled global name `main`;
//!   every other item is mangled to `module::name`.
//!
//! Stage-1 scope (per design 0008 §6, stage 1): filesystem→module mapping, `use`
//! resolution, `pub`/private visibility, and the acyclic-DAG check with a P4
//! cycle diagnostic. Deferred: the `foo.cnr`-beside-`foo/` directory-body merge
//! (a file `foo.cnr` and a directory `foo/` name independent modules here),
//! `pub use` module re-exports and the external-reachability chain, and the
//! signature-hash interface artifacts / two-tier invalidation of §2.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::ast::*;
use crate::diag::Diag;
use crate::span::Span;

/// The result of building a module tree: one merged, name-qualified program
/// plus every module-layer diagnostic (unresolved imports, visibility, cycles).
pub struct ModuleBuild {
    pub program: Program,
    pub diags: Vec<Diag>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Type,  // struct / enum
    Value, // fn / static
}

#[derive(Clone)]
struct Export {
    global: String,
    is_pub: bool,
    /// A function (called with args) vs a static (a plain value). Used to
    /// disambiguate a zero-argument `alias::name` — the parser cannot tell
    /// `alias::f()` from `alias::VALUE`, so a function export always lowers to a
    /// call and a static to a value reference.
    is_fn: bool,
}

struct Module {
    path: Vec<String>,
    program: Program,
    uses: Vec<UseDecl>,
    type_exports: HashMap<String, Export>,
    value_exports: HashMap<String, Export>,
}

/// Per-module resolved import scope (local names + imports -> global names).
#[derive(Default)]
struct Scope {
    type_scope: HashMap<String, String>,
    value_scope: HashMap<String, String>,
    aliases: HashMap<String, usize>, // namespace-import alias -> module index
}

fn path_str(path: &[String]) -> String {
    path.join("::")
}

/// Global (merged-table) name of an item. The root `fn main` keeps the name
/// `main`; everything else is `module::name`.
fn mangle(path: &[String], name: &str, kind: Kind) -> String {
    if kind == Kind::Value && path == ["main"] && name == "main" {
        "main".to_string()
    } else {
        format!("{}::{}", path_str(path), name)
    }
}

/// Discover every `.cnr` module file under `dir`, deepest determinism first
/// (entries sorted). `prefix` is the namespace path of `dir` itself.
fn discover(dir: &Path, prefix: &[String], out: &mut Vec<(Vec<String>, PathBuf)>) -> Result<(), Diag> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| io_err(dir, &e.to_string()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            let name = match p.file_name().and_then(|s| s.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            let mut sub = prefix.to_vec();
            sub.push(name);
            discover(&p, &sub, out)?;
        } else if p.extension().and_then(|s| s.to_str()) == Some("cnr") {
            let stem = match p.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            let mut mp = prefix.to_vec();
            mp.push(stem);
            out.push((mp, p));
        }
    }
    Ok(())
}

fn io_err(path: &Path, msg: &str) -> Diag {
    Diag::error(
        "E0900",
        format!("cannot read module tree at `{}`: {msg}", path.display()),
        Span::point(0),
    )
}

/// Build a module tree rooted at `dir` into one merged program plus the
/// module-layer diagnostics. A hard I/O or parse error is returned as `Err`.
pub fn build_tree(dir: &Path) -> Result<ModuleBuild, Diag> {
    let mut files = Vec::new();
    discover(dir, &[], &mut files)?;
    if files.is_empty() {
        return Err(io_err(dir, "no `.cnr` module files found"));
    }

    // Parse every file and collect its exports.
    let mut modules: Vec<Module> = Vec::new();
    for (path, file) in files {
        let src = std::fs::read_to_string(&file).map_err(|e| io_err(&file, &e.to_string()))?;
        let (program, uses, vis) = crate::real::parse_module(&src)?;
        let mut type_exports = HashMap::new();
        let mut value_exports = HashMap::new();
        for (item, &is_pub) in program.items.iter().zip(vis.iter()) {
            let (name, kind, is_fn) = match item {
                Item::Struct(s) => (&s.name, Kind::Type, false),
                Item::Enum(e) => (&e.name, Kind::Type, false),
                Item::Fn(f) => (&f.name, Kind::Value, true),
                Item::Static(s) => (&s.name, Kind::Value, false),
                // An interface exports its name in the type namespace (design 0007);
                // an `impl` block exports no name (its methods dispatch via the impl
                // table, resolved after merge).
                Item::Interface(i) => (&i.name, Kind::Type, false),
                Item::Impl(_) => continue,
            };
            let export = Export { global: mangle(&path, name, kind), is_pub, is_fn };
            match kind {
                Kind::Type => {
                    type_exports.insert(name.clone(), export);
                }
                Kind::Value => {
                    value_exports.insert(name.clone(), export);
                }
            }
        }
        modules.push(Module { path, program, uses, type_exports, value_exports });
    }

    let index: HashMap<String, usize> =
        modules.iter().enumerate().map(|(i, m)| (path_str(&m.path), i)).collect();

    let mut diags = Vec::new();

    // Resolve imports into per-module scopes, and record the import DAG edges.
    let mut scopes: Vec<Scope> = Vec::with_capacity(modules.len());
    let mut adj: Vec<Vec<(usize, Span)>> = vec![Vec::new(); modules.len()];
    for (i, m) in modules.iter().enumerate() {
        let mut scope = Scope::default();
        for (name, e) in &m.type_exports {
            scope.type_scope.insert(name.clone(), e.global.clone());
        }
        for (name, e) in &m.value_exports {
            scope.value_scope.insert(name.clone(), e.global.clone());
        }
        for u in &m.uses {
            let target = path_str(&u.segments);
            let &tidx = match index.get(&target) {
                Some(t) => t,
                None => {
                    diags.push(
                        Diag::error("E0901", format!("unresolved import: no module `{target}`"), u.span)
                            .with_note("a module is a `.cnr` file; `a::b` is the file `a/b.cnr`", None),
                    );
                    continue;
                }
            };
            adj[i].push((tidx, u.span));
            match &u.names {
                None => {
                    // Namespace import `use a::b;` -> alias `b`.
                    let alias = u.segments.last().cloned().unwrap_or_default();
                    scope.aliases.insert(alias, tidx);
                }
                Some(names) => {
                    let tm = &modules[tidx];
                    for name in names {
                        let ty = tm.type_exports.get(name);
                        let val = tm.value_exports.get(name);
                        if ty.is_none() && val.is_none() {
                            diags.push(Diag::error(
                                "E0902",
                                format!("unresolved import: `{target}` has no item `{name}`"),
                                u.span,
                            ));
                            continue;
                        }
                        if let Some(e) = ty {
                            if e.is_pub {
                                scope.type_scope.insert(name.clone(), e.global.clone());
                            } else {
                                diags.push(private_err(&target, name, u.span));
                            }
                        }
                        if let Some(e) = val {
                            if e.is_pub {
                                scope.value_scope.insert(name.clone(), e.global.clone());
                            } else {
                                diags.push(private_err(&target, name, u.span));
                            }
                        }
                    }
                }
            }
        }
        scopes.push(scope);
    }

    // Acyclic-DAG enforcement (design 0008 §3) with a P4 cycle diagnostic.
    if let Some(cycle) = find_cycle(&adj) {
        diags.push(cycle_diag(&modules, &adj, &cycle));
    }

    // Entry-point convention: the root module `main.cnr` must define `fn main`.
    match index.get("main") {
        Some(&mi) if modules[mi].value_exports.contains_key("main") => {}
        _ => diags.push(
            Diag::error("E0905", "no root module `main.cnr` defining `fn main`", Span::point(0))
                .with_note("a directory program's entry is `fn main` in the root file `main.cnr`", None),
        ),
    }

    // Rewrite every module's names to global form and merge into one program.
    let mut merged = Program { items: Vec::new() };
    for i in 0..modules.len() {
        let items = std::mem::take(&mut modules[i].program.items);
        for mut item in items {
            rename_item(&modules[i], &mut item);
            let mut rw = Rewriter { scope: &scopes[i], modules: &modules, diags: &mut diags };
            rw.rewrite_item(&mut item);
            merged.items.push(item);
        }
    }

    Ok(ModuleBuild { program: merged, diags })
}

fn private_err(module: &str, name: &str, span: Span) -> Diag {
    Diag::error("E0903", format!("private item: `{name}` in `{module}` is not `pub`"), span)
        .with_note("items are private by default; add `pub` at its definition to export it", None)
}

/// Set an item declaration's own name to its module-qualified global name.
fn rename_item(m: &Module, item: &mut Item) {
    match item {
        Item::Struct(s) => s.name = m.type_exports[&s.name].global.clone(),
        Item::Enum(e) => e.name = m.type_exports[&e.name].global.clone(),
        Item::Fn(f) => f.name = m.value_exports[&f.name].global.clone(),
        Item::Static(s) => s.name = m.value_exports[&s.name].global.clone(),
        Item::Interface(i) => i.name = m.type_exports[&i.name].global.clone(),
        // An impl's own name is not qualified, but it is tagged with its home
        // module for the orphan check (design 0007 §2.3 / 0008).
        Item::Impl(im) => im.home = Some(path_str(&m.path)),
    }
}

// ---------------------------------------------------------------------------
// Cycle detection over the import DAG
// ---------------------------------------------------------------------------

fn find_cycle(adj: &[Vec<(usize, Span)>]) -> Option<Vec<usize>> {
    let n = adj.len();
    let mut state = vec![0u8; n]; // 0 white, 1 gray, 2 black
    let mut stack: Vec<usize> = Vec::new();
    for start in 0..n {
        if state[start] == 0 {
            if let Some(c) = dfs(start, adj, &mut state, &mut stack) {
                return Some(c);
            }
        }
    }
    None
}

fn dfs(u: usize, adj: &[Vec<(usize, Span)>], state: &mut [u8], stack: &mut Vec<usize>) -> Option<Vec<usize>> {
    state[u] = 1;
    stack.push(u);
    for &(v, _) in &adj[u] {
        if state[v] == 1 {
            // Back-edge to a gray node: extract the cycle from the stack.
            let pos = stack.iter().position(|&x| x == v).unwrap();
            let mut cyc = stack[pos..].to_vec();
            cyc.push(v);
            return Some(cyc);
        } else if state[v] == 0 {
            if let Some(c) = dfs(v, adj, state, stack) {
                return Some(c);
            }
        }
    }
    stack.pop();
    state[u] = 2;
    None
}

fn edge_span(adj: &[Vec<(usize, Span)>], from: usize, to: usize) -> Span {
    adj[from].iter().find(|(t, _)| *t == to).map(|(_, s)| *s).unwrap_or(Span::point(0))
}

fn cycle_diag(modules: &[Module], adj: &[Vec<(usize, Span)>], cycle: &[usize]) -> Diag {
    let names: Vec<String> = cycle.iter().map(|&i| path_str(&modules[i].path)).collect();
    let primary = edge_span(adj, cycle[cycle.len() - 2], cycle[cycle.len() - 1]);
    let mut d = Diag::error(
        "E0904",
        format!("import cycle: {}", names.join(" -> ")),
        primary,
    )
    .with_note("the import graph must be an acyclic DAG (design 0008 §3)", None);
    for w in cycle.windows(2) {
        let span = edge_span(adj, w[0], w[1]);
        d = d.with_note(
            format!("`{}` imports `{}` here", path_str(&modules[w[0]].path), path_str(&modules[w[1]].path)),
            Some(span),
        );
    }
    d
}

// ---------------------------------------------------------------------------
// Name-qualification rewrite
// ---------------------------------------------------------------------------

struct Rewriter<'a> {
    scope: &'a Scope,
    modules: &'a [Module],
    diags: &'a mut Vec<Diag>,
}

impl<'a> Rewriter<'a> {
    fn rewrite_item(&mut self, item: &mut Item) {
        match item {
            Item::Struct(s) => {
                for f in &mut s.fields {
                    self.rewrite_ty(&mut f.ty);
                }
                if let Some(b) = &mut s.drop_hook {
                    let mut locals = vec![set(&["self"])];
                    self.rewrite_block(b, &mut locals);
                }
            }
            Item::Enum(e) => {
                for v in &mut e.variants {
                    for t in &mut v.payload {
                        self.rewrite_ty(t);
                    }
                }
            }
            Item::Fn(f) => {
                let mut locals = vec![std::collections::HashSet::new()];
                for tp in &mut f.type_params {
                    self.qualify_bounds(&mut tp.bounds);
                }
                for p in &mut f.params {
                    self.rewrite_ty(&mut p.ty);
                    locals[0].insert(p.name.clone());
                }
                for r in &mut f.requires {
                    self.rewrite_expr(r, &mut locals);
                }
                for r in &mut f.ensures {
                    self.rewrite_expr(r, &mut locals);
                }
                if let Some(rt) = &mut f.ret {
                    self.rewrite_ty(&mut rt.ty);
                }
                self.rewrite_block(&mut f.body, &mut locals);
            }
            Item::Static(s) => {
                let mut locals = vec![std::collections::HashSet::new()];
                self.rewrite_ty(&mut s.ty);
                self.rewrite_expr(&mut s.value, &mut locals);
            }
            Item::Interface(i) => {
                for m in &mut i.methods {
                    for p in &mut m.params {
                        self.rewrite_ty(&mut p.ty);
                    }
                    if let Some(rt) = &mut m.ret {
                        self.rewrite_ty(&mut rt.ty);
                    }
                }
            }
            Item::Impl(im) => {
                if let Some(g) = self.scope.type_scope.get(&im.iface) {
                    im.iface = g.clone();
                }
                for a in &mut im.iface_args {
                    self.rewrite_ty(a);
                }
                self.rewrite_ty(&mut im.target);
                for m in &mut im.methods {
                    let mut locals = vec![std::collections::HashSet::new()];
                    for p in &mut m.params {
                        self.rewrite_ty(&mut p.ty);
                        locals[0].insert(p.name.clone());
                    }
                    if let Some(rt) = &mut m.ret {
                        self.rewrite_ty(&mut rt.ty);
                    }
                    self.rewrite_block(&mut m.body, &mut locals);
                }
            }
        }
    }

    /// Qualify each bound interface name to its global form (`copy` is built-in).
    fn qualify_bounds(&mut self, bounds: &mut [String]) {
        for b in bounds.iter_mut() {
            if b != "copy" {
                if let Some(g) = self.scope.type_scope.get(b) {
                    *b = g.clone();
                }
            }
        }
    }

    fn rewrite_ty(&mut self, ty: &mut Ty) {
        match &mut ty.kind {
            TyKind::Named(n) => {
                if let Some(g) = self.scope.type_scope.get(n) {
                    *n = g.clone();
                }
            }
            TyKind::App { name, args } => {
                if let Some(g) = self.scope.type_scope.get(name) {
                    *name = g.clone();
                }
                for a in args {
                    self.rewrite_ty(a);
                }
            }
            TyKind::Array { size, elem } => {
                let mut locals = vec![std::collections::HashSet::new()];
                self.rewrite_expr(size, &mut locals);
                self.rewrite_ty(elem);
            }
            TyKind::Slice(e)
            | TyKind::SliceMut(e)
            | TyKind::RawPtr(e)
            | TyKind::Box(e)
            | TyKind::BoxResult(e)
            | TyKind::Borrow(e)
            | TyKind::BorrowMut(e) => self.rewrite_ty(e),
            TyKind::FnPtr(fp) => {
                for p in &mut fp.params {
                    self.rewrite_ty(&mut p.ty);
                }
                self.rewrite_ty(&mut fp.ret);
            }
            TyKind::Proj { base, .. } => {
                if let Some(g) = self.scope.type_scope.get(base) {
                    *base = g.clone();
                }
            }
            TyKind::Scalar(_) => {}
        }
    }

    fn rewrite_block(&mut self, block: &mut Block, locals: &mut Vec<Scopeset>) {
        locals.push(std::collections::HashSet::new());
        for stmt in &mut block.stmts {
            self.rewrite_stmt(stmt, locals);
        }
        locals.pop();
    }

    fn rewrite_stmt(&mut self, stmt: &mut Stmt, locals: &mut Vec<Scopeset>) {
        match &mut stmt.kind {
            StmtKind::Let { name, ty, init, .. } => {
                if let Some(t) = ty {
                    self.rewrite_ty(t);
                }
                if let Some(e) = init {
                    self.rewrite_expr(e, locals);
                }
                locals.last_mut().unwrap().insert(name.clone());
            }
            StmtKind::Assign { target, value } => {
                self.rewrite_expr(target, locals);
                self.rewrite_expr(value, locals);
            }
            StmtKind::Expr(e) => self.rewrite_expr(e, locals),
        }
    }

    fn rewrite_expr(&mut self, expr: &mut Expr, locals: &mut Vec<Scopeset>) {
        // Cross-module `alias::item` (a namespace import) rewrites the whole node.
        if let ExprKind::EnumCtor { enum_name, .. } = &expr.kind {
            if self.scope.aliases.contains_key(enum_name) {
                self.rewrite_alias_ctor(expr, locals);
                return;
            }
        }
        match &mut expr.kind {
            ExprKind::Ident(n) => {
                if !is_local(locals, n) {
                    if let Some(g) = self.scope.value_scope.get(n) {
                        *n = g.clone();
                    }
                }
            }
            ExprKind::EnumCtor { enum_name, args, .. } => {
                if let Some(g) = self.scope.type_scope.get(enum_name) {
                    *enum_name = g.clone();
                }
                for a in args {
                    self.rewrite_expr(a, locals);
                }
            }
            ExprKind::StructLit { name, fields } => {
                if let Some(g) = self.scope.type_scope.get(name) {
                    *name = g.clone();
                }
                for f in fields {
                    self.rewrite_expr(&mut f.value, locals);
                }
            }
            ExprKind::Unary { expr: e, .. }
            | ExprKind::Prefix { expr: e, .. }
            | ExprKind::OutArg(e)
            | ExprKind::Paren(e)
            | ExprKind::Try(e)
            | ExprKind::Assert(e)
            | ExprKind::Panic(e) => self.rewrite_expr(e, locals),
            ExprKind::Binary { lhs, rhs, .. } => {
                self.rewrite_expr(lhs, locals);
                self.rewrite_expr(rhs, locals);
            }
            ExprKind::Call { callee, args } => {
                self.rewrite_expr(callee, locals);
                for a in args {
                    self.rewrite_expr(a, locals);
                }
            }
            ExprKind::Field { base, .. } => self.rewrite_expr(base, locals),
            ExprKind::Index { base, index } => {
                self.rewrite_expr(base, locals);
                self.rewrite_expr(index, locals);
            }
            ExprKind::Conv { ty, expr: e }
            | ExprKind::CastPtr { ty, arg: e }
            | ExprKind::AddrToPtr { ty, arg: e } => {
                self.rewrite_ty(ty);
                self.rewrite_expr(e, locals);
            }
            ExprKind::PtrNull { ty }
            | ExprKind::Offsetof { ty, .. }
            | ExprKind::Sizeof(ty)
            | ExprKind::Alignof(ty) => self.rewrite_ty(ty),
            ExprKind::FieldPtr { ptr, .. } => self.rewrite_expr(ptr, locals),
            ExprKind::ArrayLit(v) => {
                for e in v {
                    self.rewrite_expr(e, locals);
                }
            }
            ExprKind::ArrayRepeat { value, size } => {
                self.rewrite_expr(value, locals);
                self.rewrite_expr(size, locals);
            }
            ExprKind::Block(b) => self.rewrite_block(b, locals),
            ExprKind::If { cond, then_blk, else_blk } => {
                self.rewrite_expr(cond, locals);
                self.rewrite_block(then_blk, locals);
                if let Some(e) = else_blk {
                    self.rewrite_expr(e, locals);
                }
            }
            ExprKind::Match { scrutinee, arms } => {
                self.rewrite_expr(scrutinee, locals);
                for arm in arms {
                    locals.push(std::collections::HashSet::new());
                    self.rewrite_pattern(&mut arm.pattern, locals);
                    self.rewrite_expr(&mut arm.body, locals);
                    locals.pop();
                }
            }
            ExprKind::Loop(b) | ExprKind::Wrapping(b) | ExprKind::Saturating(b) => {
                self.rewrite_block(b, locals)
            }
            ExprKind::While { cond, body } => {
                self.rewrite_expr(cond, locals);
                self.rewrite_block(body, locals);
            }
            ExprKind::Unsafe { body, .. } => self.rewrite_block(body, locals),
            ExprKind::Return(e) => {
                if let Some(e) = e {
                    self.rewrite_expr(e, locals);
                }
            }
            ExprKind::GenericVal { name, ty_args } => {
                if !is_local(locals, name) {
                    if let Some(g) = self.scope.value_scope.get(name) {
                        *name = g.clone();
                    }
                }
                for a in ty_args {
                    self.rewrite_ty(a);
                }
            }
            ExprKind::IntLit { .. }
            | ExprKind::NegIntLit { .. }
            | ExprKind::StrLit(_)
            | ExprKind::BoolLit(_)
            | ExprKind::Break
            | ExprKind::Continue
            | ExprKind::Result => {}
        }
    }

    fn rewrite_alias_ctor(&mut self, expr: &mut Expr, locals: &mut Vec<Scopeset>) {
        let span = expr.span;
        let (alias, item, mut args) = match std::mem::replace(&mut expr.kind, ExprKind::Break) {
            ExprKind::EnumCtor { enum_name, variant, args } => (enum_name, variant, args),
            _ => unreachable!(),
        };
        for a in args.iter_mut() {
            self.rewrite_expr(a, locals);
        }
        let midx = self.scope.aliases[&alias];
        let module = path_str(&self.modules[midx].path);
        let val = self.modules[midx].value_exports.get(&item).cloned();
        let is_type = self.modules[midx].type_exports.contains_key(&item);
        match val {
            Some(e) => {
                if !e.is_pub {
                    self.diags.push(private_err(&module, &item, span));
                }
                expr.kind = if e.is_fn {
                    ExprKind::Call {
                        callee: Box::new(Expr { kind: ExprKind::Ident(e.global), span }),
                        args,
                    }
                } else {
                    ExprKind::Ident(e.global)
                };
            }
            None => {
                if is_type {
                    self.diags.push(Diag::error(
                        "E0902",
                        format!("`{module}::{item}` is a type, not a value"),
                        span,
                    ));
                } else {
                    self.diags.push(Diag::error(
                        "E0902",
                        format!("unresolved import: `{module}` has no item `{item}`"),
                        span,
                    ));
                }
                expr.kind = ExprKind::EnumCtor { enum_name: alias, variant: item, args };
            }
        }
    }

    fn rewrite_pattern(&mut self, pat: &mut Pattern, locals: &mut Vec<Scopeset>) {
        match &mut pat.kind {
            PatKind::Wildcard => {}
            PatKind::Binding(name) => {
                locals.last_mut().unwrap().insert(name.clone());
            }
            PatKind::Variant { enum_name, sub, .. } => {
                if let Some(g) = self.scope.type_scope.get(enum_name) {
                    *enum_name = g.clone();
                }
                for s in sub {
                    self.rewrite_pattern(s, locals);
                }
            }
        }
    }
}

type Scopeset = std::collections::HashSet<String>;

fn set(names: &[&str]) -> Scopeset {
    names.iter().map(|s| s.to_string()).collect()
}

fn is_local(locals: &[Scopeset], name: &str) -> bool {
    locals.iter().any(|s| s.contains(name))
}
