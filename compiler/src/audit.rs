//! `candor audit <dir_or_file>` — the program's trusted-computing-base surface
//! (design 0011 §6; 0008 §4's `candor audit --boundaries`; philosophy §7). A
//! structural query, as machine-readable JSON from one walk (P4), over the three
//! kinds of trust a reader inherits:
//!
//! - **boundary modules**: their externs (full signature + `foreign` effect +
//!   every trust predicate, justification, and span), their exports, and the
//!   **effect reach** — which functions *discharge* `foreign` and which
//!   *propagate* it.
//! - **unsafe regions**: every `unsafe "justification" { .. }` block in the whole
//!   program (boundary or not), with its enclosing function, file, justification,
//!   and span — the memory-safety trust a plain `unsafe` block (raw pointer
//!   arithmetic, `cast_ptr`, ...) makes the reader inherit even when it never
//!   touches the foreign boundary (philosophy §7; 0017 review F1b).
//!
//! The **assumed-proven** contracts of philosophy §7 are not a distinct language
//! construct this edition (there is no `assumed-proven` annotation in the AST or
//! checker). The assumed-proven surface *is* exactly the union enumerated above:
//! the boundary `trust` clauses (every predicate is recorded-and-assumed, never
//! evaluated — see `ast::TrustDecl`) plus the `unsafe` justifications. No section
//! is invented for a construct that does not exist.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::ast::*;
use crate::diag::Diag;
use crate::manifest::Edition;

#[derive(Serialize, Clone)]
pub struct AuditReport {
    pub boundary_modules: Vec<ModuleReport>,
    pub effect_reach: Vec<ReachEntry>,
    pub unsafe_regions: Vec<UnsafeReport>,
    pub summary: Summary,
}

#[derive(Serialize, Clone)]
pub struct ModuleReport {
    pub module: String,
    pub file: String,
    pub abi: Vec<String>,
    pub externs: Vec<ExternReport>,
    pub exports: Vec<ExportReport>,
}

#[derive(Serialize, Clone)]
pub struct ExternReport {
    pub name: String,
    pub signature: String,
    pub foreign: bool,
    pub trust: Option<TrustReport>,
}

#[derive(Serialize, Clone)]
pub struct TrustReport {
    pub justification: String,
    pub span: SpanRep,
    pub predicates: Vec<PredReport>,
}

#[derive(Serialize, Clone)]
pub struct PredReport {
    pub predicate: String,
    pub args: Vec<String>,
    pub span: SpanRep,
}

#[derive(Serialize, Clone)]
pub struct ExportReport {
    pub symbol: String,
    pub signature: String,
    pub candor_fn: String,
    pub inbound_fault: String,
}

#[derive(Serialize, Clone)]
pub struct ReachEntry {
    pub function: String,
    pub status: String,
}

/// One `unsafe "justification" { .. }` region (design 0001 §11.4; philosophy §7).
/// Its `function` is the module-qualified enclosing definition, `span` covers the
/// whole `unsafe` expression, and `justification` is the mandatory (P1) rationale
/// string the author must supply.
#[derive(Serialize, Clone)]
pub struct UnsafeReport {
    pub function: String,
    pub file: String,
    pub justification: String,
    pub span: SpanRep,
}

#[derive(Serialize, Clone)]
pub struct Summary {
    pub boundary_modules: usize,
    pub externs: usize,
    pub trust_predicates: usize,
    pub undischarged_foreign_wrappers: usize,
    pub exports: usize,
    pub unsafe_regions: usize,
}

/// The per-package trust-surface counts `candor.lock` records so that *adding or
/// updating* a dependency surfaces its trust delta in the lock diff (design 0017
/// §8; enumerate-only, not gating — Open-Q1). These are exactly the structural
/// fields of an audit [`Summary`], produced by the same per-package walk, so the
/// recorded counts track the real trust surface.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Default)]
pub struct TrustSummary {
    pub boundary_modules: usize,
    pub externs: usize,
    pub unsafe_regions: usize,
}

#[derive(Serialize, Clone, Copy)]
pub struct SpanRep {
    pub start: usize,
    pub end: usize,
}

impl From<crate::span::Span> for SpanRep {
    fn from(s: crate::span::Span) -> SpanRep {
        SpanRep { start: s.start, end: s.end }
    }
}

// ----- whole-graph aggregation (design 0017 §8; review F1b) ----------------

/// The aggregated audit of a manifested package and its whole resolved
/// dependency graph. The top-level `boundary_modules` / `effect_reach` /
/// `unsafe_regions` / `summary` are the **root package's** local surface — the
/// exact shape a single-package audit emits, so existing readers are unaffected.
/// The added `packages` array is the per-package attribution layer: every package
/// in the resolved set (root + every transitive path dependency) with its own
/// trust surface tagged by name + version + source. A dependency's `foreign`
/// externs and `unsafe` regions are thus visible and traceable to it — a
/// dependency cannot hide I/O from the consumer's `candor audit`.
#[derive(Serialize)]
struct GraphAuditReport {
    #[serde(flatten)]
    root: AuditReport,
    packages: Vec<PackageAudit>,
}

/// One package's audit, attributed to its provenance (design 0017 §8, P16).
#[derive(Serialize)]
struct PackageAudit {
    package: String,
    version: String,
    source: PackageSource,
    is_root: bool,
    #[serde(flatten)]
    audit: AuditReport,
}

/// A package's pinned source, mirroring the resolver's `ResolvedSource`
/// (design 0017 §4/§6): a canonical directory, or a git url pinned to a rev.
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum PackageSource {
    Path { path: String },
    Git { url: String, rev: String },
}

/// True when `path` is a manifested package directory declaring at least one
/// dependency — the gate that routes the audit to the whole-graph aggregation.
fn is_multi_package(path: &Path) -> Result<bool, Diag> {
    if !path.is_dir() {
        return Ok(false);
    }
    match crate::manifest::load_manifest(path) {
        Ok(Some(m)) => Ok(!m.dependencies.is_empty()),
        Ok(None) => Ok(false),
        Err(e) => Err(Diag::error(
            e.code,
            format!("cannot read package manifest: {}", e.message),
            crate::span::Span::point(0),
        )),
    }
}

/// Resolve the root package's dependency graph (reusing the resolver) and audit
/// every package in the pinned set, attributing each finding to its package
/// (design 0017 §8). The root package's own surface is also surfaced at the top
/// level for the backward-compatible single-package shape.
fn audit_graph(root_dir: &Path) -> Result<String, Diag> {
    let resolution = crate::resolve_pkg::resolve(root_dir)?;
    let mut packages = Vec::with_capacity(resolution.packages.len());
    let mut root_audit: Option<AuditReport> = None;
    for (i, pkg) in resolution.packages.iter().enumerate() {
        // Each package is audited over its own `src/` tree; the structural
        // boundary/unsafe enumeration is a pure parse per file, and the effect
        // reach runs the checker over that package alone.
        let audit = audit_program(&pkg.src_root, pkg.edition_kind())?;
        let is_root = i == resolution.root;
        if is_root {
            root_audit = Some(audit.clone());
        }
        packages.push(PackageAudit {
            package: pkg.name.clone(),
            version: format!("{}.{}.{}", pkg.version.major, pkg.version.minor, pkg.version.patch),
            source: package_source(&pkg.source),
            is_root,
            audit,
        });
    }
    let root = root_audit.expect("the resolved set always contains the root package");
    Ok(serde_json::to_string_pretty(&GraphAuditReport { root, packages })
        .expect("GraphAuditReport serializes"))
}

fn package_source(src: &crate::resolve_pkg::ResolvedSource) -> PackageSource {
    match src {
        crate::resolve_pkg::ResolvedSource::Path(dir) => {
            PackageSource::Path { path: dir.to_string_lossy().into_owned() }
        }
        crate::resolve_pkg::ResolvedSource::Git { url, rev } => {
            PackageSource::Git { url: url.clone(), rev: rev.clone() }
        }
    }
}

/// Audit a `.cnr` file or a directory of them. Returns pretty JSON (design 0011
/// §6), or a hard parse/IO error.
///
/// A manifested package that declares dependencies is audited across its **whole
/// resolved dependency graph** (design 0017 §8): every package's trust surface is
/// enumerated and attributed to it. A bare file, a manifest-less directory, or a
/// dependency-free package takes the unchanged single-package walk.
pub fn audit_path(path: &Path) -> Result<String, Diag> {
    if is_multi_package(path)? {
        return audit_graph(path);
    }
    let report = audit_program(path, dir_edition(path))?;
    Ok(serde_json::to_string_pretty(&report).expect("AuditReport serializes"))
}

/// The surface edition of a bare file or a single (dependency-free) package
/// directory (1.0-gate item 1): a manifested package's declared edition, else the
/// 2026 default (a bare file or manifest-less directory).
fn dir_edition(path: &Path) -> Edition {
    if path.is_dir() {
        if let Ok(Some(m)) = crate::manifest::load_manifest(path) {
            return m.package.edition_kind();
        }
    }
    Edition::E2026
}

/// Audit one program rooted at `path` (a `.cnr` file or a module-tree directory):
/// its boundary modules, effect reach, and `unsafe` regions. This is the per-
/// package walk the whole-graph audit runs over each resolved package.
fn audit_program(path: &Path, edition: Edition) -> Result<AuditReport, Diag> {
    let Surface { boundary_modules, unsafe_regions, externs, trust_predicates, exports } =
        structural_surface(path, edition)?;

    // Effect reach: run the checker over the merged/single program.
    let (foreign_report, undischarged) = effect_reach(path, edition)?;

    let summary = Summary {
        boundary_modules: boundary_modules.len(),
        externs,
        trust_predicates,
        undischarged_foreign_wrappers: undischarged,
        exports,
        unsafe_regions: unsafe_regions.len(),
    };
    Ok(AuditReport { boundary_modules, effect_reach: foreign_report, unsafe_regions, summary })
}

/// Count a package's trust surface — boundary modules, foreign externs, and
/// `unsafe` regions — over its own `src/` tree (design 0017 §8). This reuses the
/// exact structural enumeration `candor audit` performs per package (the boundary
/// walk + [`collect_unsafe_regions`]); it deliberately skips the checker's effect
/// reach, which the counts do not need. It is the post-resolution pass's source
/// of the lockfile's per-package trust summary — kept here, beside the walk it
/// reuses, so the resolver never depends on the audit's enumeration itself.
pub fn trust_counts(src_root: &Path, edition: Edition) -> Result<TrustSummary, Diag> {
    let s = structural_surface(src_root, edition)?;
    Ok(TrustSummary {
        boundary_modules: s.boundary_modules.len(),
        externs: s.externs,
        unsafe_regions: s.unsafe_regions.len(),
    })
}

/// The boundary/foreign surface a freestanding graph may not contain (design
/// 0017 §8, review F5; 0011 §5). It is the specific item to name in the
/// composition diagnostic so the author can act.
pub struct BoundarySurface {
    /// The module-qualified boundary module (e.g. `hal::hal`).
    pub module: String,
    /// The first `foreign` extern the module declares, when any — the specific
    /// item that has no libc to link against under freestanding (0011 §5).
    pub foreign_extern: Option<String>,
}

/// The first boundary/foreign surface a package rooted at `src_root` contributes,
/// or `None` when it has none — the datum the freestanding composition check
/// (`resolve_pkg`) rejects on (review F5). This reuses the exact structural
/// enumeration `candor audit` walks ([`structural_surface`]) — the single source
/// of truth for a package's boundary modules and `foreign` externs — so the check
/// consumes the audit's data instead of re-walking. Boundary modules are
/// enumerated in module-path-sorted order and externs in source order, so the
/// reported surface is deterministic.
pub fn first_boundary_surface(src_root: &Path, edition: Edition) -> Result<Option<BoundarySurface>, Diag> {
    let s = structural_surface(src_root, edition)?;
    Ok(s.boundary_modules.into_iter().next().map(|m| BoundarySurface {
        module: m.module,
        foreign_extern: m.externs.into_iter().next().map(|e| e.name),
    }))
}

/// The structural trust surface of a program rooted at `path`: its boundary
/// modules, `unsafe` regions, and the extern/predicate/export tallies — a pure
/// parse walk (no checker). Shared by [`audit_program`] and [`trust_counts`] so
/// the enumeration lives in exactly one place.
struct Surface {
    boundary_modules: Vec<ModuleReport>,
    unsafe_regions: Vec<UnsafeReport>,
    externs: usize,
    trust_predicates: usize,
    exports: usize,
}

fn structural_surface(path: &Path, edition: Edition) -> Result<Surface, Diag> {
    let files = collect_files(path)?;
    let mut modules = Vec::new();
    let mut unsafe_regions = Vec::new();
    let mut total_externs = 0usize;
    let mut total_preds = 0usize;
    let mut total_exports = 0usize;

    for (module, file, display) in &files {
        let src = std::fs::read_to_string(file)
            .map_err(|e| Diag::error("E0900", format!("cannot read `{}`: {e}", file.display()), crate::span::Span::point(0)))?;
        let (prog, boundary) = crate::real::parse_with_boundary_in(&src, edition)?;
        collect_unsafe_regions(&prog, module, display, &mut unsafe_regions);
        if !boundary {
            continue;
        }
        let mut abi: Vec<String> = Vec::new();
        let mut externs = Vec::new();
        let mut exports = Vec::new();
        for item in &prog.items {
            match item {
                Item::Extern(eb) => {
                    if !abi.contains(&eb.abi) {
                        abi.push(eb.abi.clone());
                    }
                    for ef in &eb.fns {
                        total_externs += 1;
                        let trust = ef.trust.as_ref().map(|t| {
                            total_preds += t.predicates.len();
                            TrustReport {
                                justification: t.justification.clone(),
                                span: t.span.into(),
                                predicates: t
                                    .predicates
                                    .iter()
                                    .map(|p| PredReport {
                                        predicate: p.name.clone(),
                                        args: p.args.clone(),
                                        span: p.span.into(),
                                    })
                                    .collect(),
                            }
                        });
                        externs.push(ExternReport {
                            name: ef.name.clone(),
                            signature: extern_sig_str(ef),
                            foreign: true,
                            trust,
                        });
                    }
                }
                Item::Export(ex) => {
                    total_exports += 1;
                    exports.push(ExportReport {
                        symbol: ex.symbol.clone(),
                        signature: export_sig_str(ex),
                        candor_fn: ex.candor_fn.clone(),
                        inbound_fault: "root policy (abort; no unwinding across C frames)".to_string(),
                    });
                }
                _ => {}
            }
        }
        modules.push(ModuleReport {
            module: module.clone(),
            file: display.clone(),
            abi,
            externs,
            exports,
        });
    }

    Ok(Surface {
        boundary_modules: modules,
        unsafe_regions,
        externs: total_externs,
        trust_predicates: total_preds,
        exports: total_exports,
    })
}

/// Run the checker over the whole program and return the boundary-module effect
/// reach (functions that discharge or propagate `foreign`) plus the undischarged
/// count. Names are module-qualified for a directory build.
fn effect_reach(path: &Path, edition: Edition) -> Result<(Vec<ReachEntry>, usize), Diag> {
    let prog = if path.is_dir() {
        crate::modules::build_tree(path)?.program
    } else {
        let src = std::fs::read_to_string(path)
            .map_err(|e| Diag::error("E0900", format!("cannot read `{}`: {e}", path.display()), crate::span::Span::point(0)))?;
        crate::real::parse_source_in(&src, edition)?
    };
    let (_diags, report) = crate::check::check_program_real_foreign(&prog);
    let mut out = Vec::new();
    let mut undischarged = 0usize;
    for r in &report {
        if !r.boundary {
            continue;
        }
        if r.discharges {
            out.push(ReachEntry { function: r.name.clone(), status: "discharges foreign".to_string() });
        } else if r.propagates {
            undischarged += 1;
            out.push(ReachEntry { function: r.name.clone(), status: "propagates foreign (undischarged)".to_string() });
        }
    }
    Ok((out, undischarged))
}

/// Discover the `.cnr` files to audit with their module paths (design 0008 §1).
fn collect_files(path: &Path) -> Result<Vec<(String, std::path::PathBuf, String)>, Diag> {
    if path.is_dir() {
        let mut out = Vec::new();
        discover(path, &[], &mut out);
        out.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(out)
    } else {
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("module").to_string();
        let display = path.file_name().and_then(|s| s.to_str()).unwrap_or("module.cnr").to_string();
        Ok(vec![(stem, path.to_path_buf(), display)])
    }
}

fn discover(dir: &Path, prefix: &[String], out: &mut Vec<(String, std::path::PathBuf, String)>) {
    let mut entries: Vec<std::path::PathBuf> = match std::fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(|e| e.ok().map(|e| e.path())).collect(),
        Err(_) => return,
    };
    entries.sort();
    for p in entries {
        if p.is_dir() {
            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                let mut sub = prefix.to_vec();
                sub.push(name.to_string());
                discover(&p, &sub, out);
            }
        } else if p.extension().and_then(|s| s.to_str()) == Some("cnr") {
            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                let mut mp = prefix.to_vec();
                mp.push(stem.to_string());
                let display = format!("{}.cnr", mp.join("/"));
                out.push((mp.join("::"), p, display));
            }
        }
    }
}

// ----- unsafe-region enumeration (philosophy §7; 0017 review F1b) ----------

/// Walk every executable body in a parsed module — free functions, impl methods,
/// struct `drop` hooks, and static initializers — recording each `unsafe` region.
/// Enumerated for *every* module, boundary or not: an `unsafe` block need not
/// touch the foreign surface at all (raw pointer arithmetic, `cast_ptr`, ...), so
/// restricting to boundary files would hide exactly the memory-safety trust §7
/// requires `candor audit` to surface.
fn collect_unsafe_regions(prog: &Program, module: &str, file: &str, out: &mut Vec<UnsafeReport>) {
    for item in &prog.items {
        match item {
            Item::Fn(f) => walk_block_unsafe(&f.body, &format!("{module}::{}", f.name), file, out),
            Item::Impl(im) => {
                let ty = ty_c(&im.target);
                for m in &im.methods {
                    walk_block_unsafe(&m.body, &format!("{module}::{ty}::{}", m.name), file, out);
                }
            }
            Item::Struct(s) => {
                if let Some(hook) = &s.drop_hook {
                    walk_block_unsafe(hook, &format!("{module}::{}::drop", s.name), file, out);
                }
            }
            Item::Static(st) => walk_expr_unsafe(&st.value, &format!("{module}::{}", st.name), file, out),
            _ => {}
        }
    }
}

fn walk_block_unsafe(b: &Block, func: &str, file: &str, out: &mut Vec<UnsafeReport>) {
    for st in &b.stmts {
        match &st.kind {
            StmtKind::Let { init: Some(e), .. } => walk_expr_unsafe(e, func, file, out),
            StmtKind::Let { init: None, .. } => {}
            StmtKind::Assign { target, value } => {
                walk_expr_unsafe(target, func, file, out);
                walk_expr_unsafe(value, func, file, out);
            }
            StmtKind::Expr(e) => walk_expr_unsafe(e, func, file, out),
        }
    }
}

/// Total recursive walk over an expression, mirroring the pipeline's own expr walk
/// (`count::Counter::expr`), recording every `unsafe` region it finds. Exhaustive
/// by construction — a new `ExprKind` breaks the build here rather than silently
/// dropping a region from the trust surface.
fn walk_expr_unsafe(e: &Expr, func: &str, file: &str, out: &mut Vec<UnsafeReport>) {
    match &e.kind {
        ExprKind::Unsafe { justification, body } => {
            out.push(UnsafeReport {
                function: func.to_string(),
                file: file.to_string(),
                justification: justification.clone(),
                span: e.span.into(),
            });
            walk_block_unsafe(body, func, file, out);
        }

        ExprKind::For { .. } => unreachable!("`for` is desugared at parse (design 0009 §4.2)"),
        ExprKind::Scope(b) => walk_block_unsafe(b, func, file, out),
        ExprKind::Spawn(c) => walk_expr_unsafe(c, func, file, out),

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

        ExprKind::Prefix { expr, .. } | ExprKind::Unary { expr, .. } => walk_expr_unsafe(expr, func, file, out),
        ExprKind::Binary { lhs, rhs, .. } => {
            walk_expr_unsafe(lhs, func, file, out);
            walk_expr_unsafe(rhs, func, file, out);
        }
        ExprKind::Call { callee, args } => {
            walk_expr_unsafe(callee, func, file, out);
            for a in args {
                walk_expr_unsafe(a, func, file, out);
            }
        }
        ExprKind::OutArg(inner) => walk_expr_unsafe(inner, func, file, out),
        ExprKind::Field { base, .. } => walk_expr_unsafe(base, func, file, out),
        ExprKind::Index { base, index } => {
            walk_expr_unsafe(base, func, file, out);
            walk_expr_unsafe(index, func, file, out);
        }
        ExprKind::Conv { expr, .. } | ExprKind::Bitcast { expr, .. } => walk_expr_unsafe(expr, func, file, out),
        ExprKind::ArrayLit(v) => {
            for x in v {
                walk_expr_unsafe(x, func, file, out);
            }
        }
        ExprKind::ArrayRepeat { value, size } => {
            walk_expr_unsafe(value, func, file, out);
            walk_expr_unsafe(size, func, file, out);
        }
        ExprKind::StructLit { fields, .. } => {
            for fi in fields {
                walk_expr_unsafe(&fi.value, func, file, out);
            }
        }
        ExprKind::EnumCtor { args, .. } => {
            for a in args {
                walk_expr_unsafe(a, func, file, out);
            }
        }
        ExprKind::CastPtr { arg, .. } | ExprKind::AddrToPtr { arg, .. } => walk_expr_unsafe(arg, func, file, out),
        ExprKind::FieldPtr { ptr, .. } => walk_expr_unsafe(ptr, func, file, out),
        ExprKind::Block(b) => walk_block_unsafe(b, func, file, out),
        ExprKind::If { cond, then_blk, else_blk } => {
            walk_expr_unsafe(cond, func, file, out);
            walk_block_unsafe(then_blk, func, file, out);
            if let Some(el) = else_blk {
                walk_expr_unsafe(el, func, file, out);
            }
        }
        ExprKind::Match { scrutinee, arms } => {
            walk_expr_unsafe(scrutinee, func, file, out);
            for arm in arms {
                if let Some(g) = &arm.guard {
                    walk_expr_unsafe(g, func, file, out);
                }
                walk_expr_unsafe(&arm.body, func, file, out);
            }
        }
        ExprKind::Loop(b) => walk_block_unsafe(b, func, file, out),
        ExprKind::While { cond, body } => {
            walk_expr_unsafe(cond, func, file, out);
            walk_block_unsafe(body, func, file, out);
        }
        ExprKind::Wrapping(b) | ExprKind::Saturating(b) => walk_block_unsafe(b, func, file, out),
        ExprKind::Return(opt) => {
            if let Some(x) = opt {
                walk_expr_unsafe(x, func, file, out);
            }
        }
        ExprKind::Assert(x) | ExprKind::Panic(x) | ExprKind::Paren(x) | ExprKind::Try(x) => {
            walk_expr_unsafe(x, func, file, out)
        }
    }
}

// ----- signature rendering (design 0011 §6 output form) --------------------

fn extern_sig_str(ef: &ExternFn) -> String {
    let ps: Vec<String> = ef.params.iter().map(|p| ty_c(&p.ty)).collect();
    let ret = ef.ret.as_ref().map(|r| ty_c(&r.ty)).unwrap_or_else(|| "unit".to_string());
    format!("fn {}({}) foreign -> {}", ef.name, ps.join(", "), ret)
}

fn export_sig_str(ex: &ExportDecl) -> String {
    let ps: Vec<String> = ex.params.iter().map(|p| ty_c(&p.ty)).collect();
    let ret = ex.ret.as_ref().map(|r| ty_c(&r.ty)).unwrap_or_else(|| "unit".to_string());
    format!("fn {}({}) -> {}", ex.symbol, ps.join(", "), ret)
}

/// Render a boundary type in C-facing form for the audit (design 0011 §6).
fn ty_c(ty: &Ty) -> String {
    use crate::ast::TyKind::*;
    match &ty.kind {
        Scalar(s) => format!("{s:?}").to_lowercase(),
        Named(n) => n.clone(),
        RawPtr(e) => format!("rawptr {}", ty_c(e)),
        Array { elem, .. } => format!("[N]{}", ty_c(elem)),
        _ => "?".to_string(),
    }
}
