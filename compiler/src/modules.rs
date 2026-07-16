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
pub struct Export {
    pub global: String,
    pub is_pub: bool,
    /// A function (called with args) vs a static (a plain value). Used to
    /// disambiguate a zero-argument `alias::name` — the parser cannot tell
    /// `alias::f()` from `alias::VALUE`, so a function export always lowers to a
    /// call and a static to a value reference.
    pub is_fn: bool,
}

/// The exported name tables of one module (design 0008 §2), reconstructable from
/// a cached interface artifact so a **dirty importer** can be qualified without
/// re-parsing this module's source. Shared by [`parse_one`]/[`qualify_one`], the
/// Stage-C2 signature-only incremental build.
pub struct ModuleExports {
    pub path: Vec<String>,
    pub type_exports: HashMap<String, Export>,
    pub value_exports: HashMap<String, Export>,
}

struct Module {
    path: Vec<String>,
    program: Program,
    uses: Vec<UseDecl>,
    type_exports: HashMap<String, Export>,
    value_exports: HashMap<String, Export>,
    /// Raw file text (for the Stage-C source hash, design 0008 §2).
    source: String,
    /// The file's `boundary` preamble status (design 0011 / 0008 §4).
    boundary: bool,
    /// Per-item `pub` flag, parallel to `program.items`.
    is_pub: Vec<bool>,
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

/// The single-package entry module path (`["main"]`): a bare directory and a
/// dependency-free manifested package keep `fn main` in module `main` unmangled,
/// exactly as before the packaging slice (design 0008 §2.4 / 0017 §5).
fn bare_main_entry() -> [String; 1] {
    [String::from("main")]
}

/// Global (merged-table) name of an item. The program **entry** `fn main` keeps
/// the bare name `main` (so the backend/interpreter finds it); everything else is
/// `module::name`. In a multi-package build every path already carries its
/// package's injective pkgid as its first segment (design 0017 §5), so `mangle`
/// yields `<pkgid>::module::name` there without any extra work — only the root
/// entry is exempted, via `entry`. A single-package build passes `entry =
/// Some(["main"])`, exactly reproducing the historical bare-`main` special case.
fn mangle(path: &[String], name: &str, kind: Kind, entry: Option<&[String]>) -> String {
    if kind == Kind::Value && name == "main" && entry == Some(path) {
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

// ---------------------------------------------------------------------------
// Directory-build root resolution (design 0008 §2.4 + its 2026-07-15 erratum,
// design 0017 §1): a manifest-less bare directory roots its module tree at the
// directory and requires `fn main` in the root `main.cnr`; a manifested package
// (a `candor.toml` beside it) relocates the root to `src/` and takes its entry
// targets from the manifest. This slice ONLY moves where the tree roots — the
// merge into one name-qualified `Program` (design 0008) is unchanged.
// ---------------------------------------------------------------------------

/// A directory build's module-tree root plus its entry-point requirement.
pub struct DirRoot {
    /// The directory the module tree is discovered from: `dir` for a bare
    /// directory, `dir/src` for a manifested package.
    root: PathBuf,
    entry: EntryRule,
}

impl DirRoot {
    /// The directory to discover `.cnr` module files from.
    pub fn discover_root(&self) -> &Path {
        &self.root
    }
}

/// What entry a directory build requires (design 0008 §2.4 + 0017 §1).
enum EntryRule {
    /// Manifest-less bare directory: the root module `main.cnr` must define `fn main`.
    BareMain,
    /// Manifested package: each declared target's entry file must be present (and a
    /// binary entry must define `fn main`). A library-only package requires no entry
    /// file — its public root `src/<name>.cnr` may be absent (a namespace-only `src/`).
    Package(Vec<RequiredEntry>),
}

/// One required entry file of a manifested package.
struct RequiredEntry {
    /// The entry module's path relative to `src/`: `["main"]` or `["bin", name]`.
    path: Vec<String>,
    /// A binary entry must define `fn main`; a library root need only exist.
    needs_main: bool,
    /// The expected file, for the missing-entry diagnostic (`src/main.cnr`, ...).
    file: String,
}

/// Resolve a directory build's module-tree root by manifest presence (reusing
/// [`crate::manifest::load_manifest`]): `Ok(None)` => a manifest-less bare
/// directory (design 0008 §2.4, unchanged); `Ok(Some(_))` => a manifested package
/// rooted at `src/` (0017 §1). A manifest parse error surfaces as a build `Diag`.
pub fn resolve_dir_root(dir: &Path) -> Result<DirRoot, Diag> {
    match crate::manifest::load_manifest(dir) {
        Ok(None) => Ok(DirRoot { root: dir.to_path_buf(), entry: EntryRule::BareMain }),
        Ok(Some(m)) => Ok(DirRoot { root: dir.join("src"), entry: package_entry(&m) }),
        Err(e) => Err(Diag::error(
            e.code,
            format!("cannot read package manifest: {}", e.message),
            Span::point(0),
        )
        .with_note("a manifested package is a directory with a `candor.toml` (design 0017 §1)", None)),
    }
}

/// The entry-file requirements of a manifested package (design 0017 §1): each
/// `[[bin]]` builds from `src/bin/<name>.cnr`; with no `[[bin]]`, a `[lib]`-only
/// package needs no entry file (namespace-only `src/` is allowed), and otherwise a
/// single implicit binary builds from `src/main.cnr`.
fn package_entry(m: &crate::manifest::Manifest) -> EntryRule {
    let mut required = Vec::new();
    if !m.bins.is_empty() {
        for b in &m.bins {
            required.push(RequiredEntry {
                path: vec!["bin".to_string(), b.name.clone()],
                needs_main: true,
                file: format!("src/bin/{}.cnr", b.name),
            });
        }
    } else if m.lib.is_none() {
        // No library and no named binaries: the implicit single binary.
        required.push(RequiredEntry {
            path: vec!["main".to_string()],
            needs_main: true,
            file: "src/main.cnr".to_string(),
        });
    }
    EntryRule::Package(required)
}

/// The bare-directory entry diagnostic (design 0008 §2.4), shared by the merge
/// assembler and the incremental builder so their `E0905` text stays identical.
fn bare_main_diag() -> Diag {
    Diag::error("E0905", "no root module `main.cnr` defining `fn main`", Span::point(0))
        .with_note("a directory program's entry is `fn main` in the root file `main.cnr`", None)
}

/// A manifested package's declared target entry file is missing (design 0017 §1).
fn entry_missing_diag(file: &str) -> Diag {
    Diag::error("E0906", format!("missing package entry file `{file}`"), Span::point(0))
        .with_note("a manifested package's module tree roots at `src/` (design 0017 §1)", None)
}

/// A manifested package's binary entry file exists but defines no `fn main`.
fn entry_no_main_diag(file: &str) -> Diag {
    Diag::error("E0905", format!("package entry `{file}` does not define `fn main`"), Span::point(0))
        .with_note("a binary target's entry is `fn main` in its entry file (design 0017 §1)", None)
}

/// The entry-point gate over parsed modules (the merge path): a bare directory
/// needs `fn main` in module `main`; a manifested package needs each declared
/// target's entry module present, with `fn main` for a binary.
fn check_entry(
    entry: &EntryRule,
    index: &HashMap<String, usize>,
    modules: &[Module],
    diags: &mut Vec<Diag>,
) {
    match entry {
        EntryRule::BareMain => match index.get("main") {
            Some(&mi) if modules[mi].value_exports.contains_key("main") => {}
            _ => diags.push(bare_main_diag()),
        },
        EntryRule::Package(required) => {
            for r in required {
                match index.get(&path_str(&r.path)) {
                    Some(&mi) if !r.needs_main || modules[mi].value_exports.contains_key("main") => {}
                    Some(_) => diags.push(entry_no_main_diag(&r.file)),
                    None => diags.push(entry_missing_diag(&r.file)),
                }
            }
        }
    }
}

/// The entry-point gate for the incremental builder (`build_dir`), which knows
/// module *paths* (present union cached) but not their parsed items — so it checks
/// entry-file presence at path granularity (the coarse gate the bare path always
/// used), never the `fn main` body.
pub fn check_entry_present(root: &DirRoot, has_path: impl Fn(&str) -> bool, diags: &mut Vec<Diag>) {
    match &root.entry {
        EntryRule::BareMain => {
            if !has_path("main") {
                diags.push(bare_main_diag());
            }
        }
        EntryRule::Package(required) => {
            for r in required {
                if !has_path(&path_str(&r.path)) {
                    diags.push(entry_missing_diag(&r.file));
                }
            }
        }
    }
}

/// Internal: discover, parse, resolve imports, and run the module-layer checks
/// (visibility, cycle, entry), returning the per-module data plus the DAG. Both
/// [`build_tree`] (merge) and [`build_tree_parts`] (per-module, Stage C) share it.
fn assemble(root: &DirRoot) -> Result<Assembled, Diag> {
    let mut files = Vec::new();
    discover(&root.root, &[], &mut files)?;
    if files.is_empty() {
        return Err(io_err(&root.root, "no `.cnr` module files found"));
    }

    // Parse every file and collect its exports.
    let bare = bare_main_entry();
    let mut modules: Vec<Module> = Vec::new();
    for (path, file) in files {
        let src = std::fs::read_to_string(&file).map_err(|e| io_err(&file, &e.to_string()))?;
        let (program, uses, vis, boundary) = crate::real::parse_module(&src)?;
        let (type_exports, value_exports) = collect_exports(&path, &program, &vis, Some(bare.as_slice()));
        modules.push(Module {
            path,
            program,
            uses,
            type_exports,
            value_exports,
            source: src,
            boundary,
            is_pub: vis,
        });
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

    // Entry-point convention (design 0008 §2.4 + the 2026-07-15 `src/` erratum,
    // design 0017 §1): a bare directory requires `fn main` in the root `main.cnr`;
    // a manifested package requires each declared target's entry file under `src/`.
    check_entry(&root.entry, &index, &modules, &mut diags);

    Ok(Assembled { modules, scopes, adj, diags })
}

/// The output of [`assemble`]: per-module data plus the import DAG.
struct Assembled {
    modules: Vec<Module>,
    scopes: Vec<Scope>,
    adj: Vec<Vec<(usize, Span)>>,
    diags: Vec<Diag>,
}

/// Qualify every module's names to global form, in place. Returns the qualified
/// items grouped by module (index-parallel to `modules`). Shared by the merge
/// and per-module builders so name-rewriting lives in exactly one place.
fn qualify(modules: &mut [Module], scopes: &[Scope], diags: &mut Vec<Diag>) -> Vec<Vec<Item>> {
    let mut per_module: Vec<Vec<Item>> = Vec::with_capacity(modules.len());
    for i in 0..modules.len() {
        let items = std::mem::take(&mut modules[i].program.items);
        let mut out = Vec::with_capacity(items.len());
        for mut item in items {
            rename_item(&modules[i], &mut item);
            let mut rw = Rewriter { scope: &scopes[i], modules, diags };
            rw.rewrite_item(&mut item);
            out.push(item);
        }
        per_module.push(out);
    }
    per_module
}

/// Build a module tree rooted at `dir` into one merged program plus the
/// module-layer diagnostics. A hard I/O or parse error is returned as `Err`.
///
/// A manifested package with `[dependencies]` resolves its dependency graph
/// (design 0017 §6/§7) and merges every package's module tree into one program
/// with **pkgid-qualified** names (design 0017 §5). A manifest-less or
/// dependency-free package takes the unchanged single-package path below, so its
/// merge — and every observable of a manifest-less build — is byte-for-byte
/// identical to before the packaging slice.
pub fn build_tree(dir: &Path) -> Result<ModuleBuild, Diag> {
    if has_dependencies(dir)? {
        let resolution = crate::resolve_pkg::resolve(dir)?;
        return build_tree_multi(&resolution);
    }
    let root = resolve_dir_root(dir)?;
    let Assembled { mut modules, scopes, mut diags, .. } = assemble(&root)?;
    let per_module = qualify(&mut modules, &scopes, &mut diags);
    let mut merged = Program { items: Vec::new() };
    for items in per_module {
        merged.items.extend(items);
    }
    Ok(ModuleBuild { program: merged, diags })
}

/// Whether `dir` is a manifested package declaring at least one dependency (the
/// gate that routes a build to the multi-package resolver).
pub fn has_dependencies(dir: &Path) -> Result<bool, Diag> {
    match crate::manifest::load_manifest(dir) {
        Ok(Some(m)) => Ok(!m.dependencies.is_empty()),
        Ok(None) => Ok(false),
        Err(e) => Err(Diag::error(e.code, format!("cannot read package manifest: {}", e.message), Span::point(0))),
    }
}

/// One cross-file `use` import's resolution within a build's module universe.
pub enum UseResolution {
    /// A same-package (or single-package) import addressing `target`; a universe
    /// miss is the unresolved-import error E0901.
    Intra { target: Vec<String> },
    /// A cross-package import naming dependency `dep_local`. It binds only when
    /// `is_public_root` and `target` exists; otherwise the package boundary walls
    /// it off (E0903) — only a dependency's public root crosses (design 0008 §3).
    Cross { dep_local: String, target: Vec<String>, is_public_root: bool },
}

/// Resolves one module's cross-file `use` imports to full (pkgid-prefixed in a
/// multi-package build) module paths, applying the package-boundary rule (design
/// 0008 §3 / 0017 §5). A single-package build uses [`ImportResolver::single`]
/// (identity resolution — `use a::b` is the module `a::b`); a multi-package build
/// derives one resolver per package via [`ImportResolver::per_package`]. This is
/// the one place `use`-to-module resolution lives, so the merge builder
/// ([`build_tree_multi`]) and the incremental builder (`candor build`) cannot
/// diverge (design 0017 Open-Q4).
#[derive(Clone, Default)]
pub struct ImportResolver {
    /// The owning package's pkgid prefix; empty in a single-package build.
    prefix: Vec<String>,
    /// Local dependency name -> its resolved pkgid.
    dep_pkgid: HashMap<String, String>,
    /// Dependency pkgid -> that dependency's public-root full (pkgid-prefixed) path.
    dep_public_root: HashMap<String, Vec<String>>,
}

impl ImportResolver {
    /// The single-package resolver: identity resolution, exactly as before the
    /// packaging slice (design 0008 §2.4).
    pub fn single() -> Self {
        Self::default()
    }

    /// One resolver per resolved package, index-parallel to `res.packages`.
    pub fn per_package(res: &crate::resolve_pkg::Resolution) -> Vec<Self> {
        let public_root: HashMap<String, Vec<String>> = res
            .packages
            .iter()
            .map(|p| {
                let mut pr = vec![p.pkgid.clone()];
                pr.extend(p.public_root.iter().cloned());
                (p.pkgid.clone(), pr)
            })
            .collect();
        res.packages
            .iter()
            .map(|p| ImportResolver {
                prefix: vec![p.pkgid.clone()],
                dep_pkgid: p.deps.clone().into_iter().collect(),
                dep_public_root: public_root.clone(),
            })
            .collect()
    }

    /// Resolve one `use`'s written module path (`segments`) to its target.
    pub fn resolve_use(&self, segments: &[String]) -> UseResolution {
        if self.prefix.is_empty() {
            return UseResolution::Intra { target: segments.to_vec() };
        }
        let seg0 = &segments[0];
        if let Some(dep_pkgid) = self.dep_pkgid.get(seg0) {
            let pr = &self.dep_public_root[dep_pkgid];
            let rest = &segments[1..];
            let target = if rest.is_empty() {
                pr.clone()
            } else {
                let mut t = vec![dep_pkgid.clone()];
                t.extend(rest.iter().cloned());
                t
            };
            let is_public_root = &target == pr;
            UseResolution::Cross { dep_local: seg0.clone(), target, is_public_root }
        } else {
            let mut target = self.prefix.clone();
            target.extend(segments.iter().cloned());
            UseResolution::Intra { target }
        }
    }
}

/// Discover every resolved package's `.cnr` modules, each path prefixed by its
/// injective pkgid (design 0017 §5) — the one place a resolved package graph is
/// mapped to module paths, shared by [`build_tree_multi`] (the merge/check build)
/// and the incremental builder (`candor build`), so their module universes are
/// identical (design 0017 Open-Q4). Returns `(module_path, file, package_index)`.
pub fn discover_multi(
    res: &crate::resolve_pkg::Resolution,
) -> Result<Vec<(Vec<String>, PathBuf, usize)>, Diag> {
    let mut out = Vec::new();
    for (pi, p) in res.packages.iter().enumerate() {
        let mut files = Vec::new();
        discover(&p.src_root, std::slice::from_ref(&p.pkgid), &mut files)?;
        if files.is_empty() {
            return Err(io_err(&p.src_root, "no `.cnr` module files found"));
        }
        for (path, file) in files {
            out.push((path, file, pi));
        }
    }
    Ok(out)
}

/// Build a resolved multi-package set (design 0017 §7) into one merged program.
/// Each package's modules are discovered under its `src/` root with the package's
/// injective pkgid as their leading path segment, so the existing merge produces
/// `<pkgid>::module::name` for every item (the root package included, review F2).
/// Cross-package `use` names a dependency by its local name as the first segment;
/// the remainder resolves within that dependency, and only the dependency's
/// externally-reachable public root crosses the boundary (design 0008 §3 / §5).
fn build_tree_multi(res: &crate::resolve_pkg::Resolution) -> Result<ModuleBuild, Diag> {
    let entry = res.entry_module();
    let resolvers = ImportResolver::per_package(res);

    // Discover + parse every package's modules, each path prefixed by its pkgid
    // (the one discovery the incremental builder shares — they cannot diverge).
    let mut modules: Vec<Module> = Vec::new();
    let mut mod_pkg: Vec<usize> = Vec::new(); // module index -> res.packages index
    for (path, file, pi) in discover_multi(res)? {
        let src = std::fs::read_to_string(&file).map_err(|e| io_err(&file, &e.to_string()))?;
        let (program, uses, vis, boundary) = crate::real::parse_module(&src)?;
        let (type_exports, value_exports) =
            collect_exports(&path, &program, &vis, entry.as_deref());
        modules.push(Module {
            path,
            program,
            uses,
            type_exports,
            value_exports,
            source: src,
            boundary,
            is_pub: vis,
        });
        mod_pkg.push(pi);
    }

    let index: HashMap<String, usize> =
        modules.iter().enumerate().map(|(i, m)| (path_str(&m.path), i)).collect();

    let mut diags = Vec::new();

    // Name-collision rule (design 0017 §5): the root package's dependency local
    // names and its own top-level `src/` module names must be disjoint.
    let root_top: std::collections::HashSet<String> = modules
        .iter()
        .zip(mod_pkg.iter())
        .filter(|(_, &pi)| pi == res.root)
        .filter_map(|(m, _)| m.path.get(1).cloned())
        .collect();
    for dep_local in res.root_pkg().deps.keys() {
        if root_top.contains(dep_local) {
            diags.push(dep_name_collision_diag(dep_local));
        }
    }

    // Resolve imports into per-module scopes + the DAG via the shared per-package
    // resolver (design 0017 Open-Q4: the one `use`-to-module resolution scheme).
    let mut scopes: Vec<Scope> = Vec::with_capacity(modules.len());
    let mut adj: Vec<Vec<(usize, Span)>> = vec![Vec::new(); modules.len()];
    for i in 0..modules.len() {
        let mut scope = Scope::default();
        for (name, e) in &modules[i].type_exports {
            scope.type_scope.insert(name.clone(), e.global.clone());
        }
        for (name, e) in &modules[i].value_exports {
            scope.value_scope.insert(name.clone(), e.global.clone());
        }
        let uses = modules[i].uses.clone();
        for u in &uses {
            let display = u.segments.join("::");
            match resolvers[mod_pkg[i]].resolve_use(&u.segments) {
                UseResolution::Intra { target } => match index.get(&path_str(&target)) {
                    Some(&tidx) => {
                        adj[i].push((tidx, u.span));
                        bind_use(u, &display, tidx, &modules, &mut scope, &mut diags);
                    }
                    None => diags.push(
                        Diag::error("E0901", format!("unresolved import: no module `{display}`"), u.span)
                            .with_note("a module is a `.cnr` file; `a::b` is the file `a/b.cnr`", None),
                    ),
                },
                UseResolution::Cross { dep_local, target, is_public_root } => {
                    match index.get(&path_str(&target)) {
                        Some(&tidx) if is_public_root => {
                            adj[i].push((tidx, u.span));
                            bind_use(u, &display, tidx, &modules, &mut scope, &mut diags);
                        }
                        _ => {
                            // Not re-exported from the dependency's public root: the
                            // boundary wall (design 0008 §3 / 0017 §5).
                            for name in u.names.as_deref().unwrap_or(&[]) {
                                diags.push(boundary_private_err(&dep_local, &display, name, u.span));
                            }
                            if u.names.is_none() {
                                diags.push(boundary_private_err(&dep_local, &display, "*", u.span));
                            }
                        }
                    }
                }
            }
        }
        scopes.push(scope);
    }

    if let Some(cycle) = find_cycle(&adj) {
        diags.push(cycle_diag(&modules, &adj, &cycle));
    }

    check_multi_entry(res, &index, &modules, &mut diags);

    let per_module = qualify(&mut modules, &scopes, &mut diags);
    let mut merged = Program { items: Vec::new() };
    for items in per_module {
        merged.items.extend(items);
    }
    Ok(ModuleBuild { program: merged, diags })
}

/// Bind one resolved `use` into a module scope: a group import binds each named
/// item (rejecting a non-`pub` one, E0903), a namespace import binds the module
/// alias. `display` is the module path as written, for diagnostics.
fn bind_use(
    u: &UseDecl,
    display: &str,
    tidx: usize,
    modules: &[Module],
    scope: &mut Scope,
    diags: &mut Vec<Diag>,
) {
    match &u.names {
        None => {
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
                        format!("unresolved import: `{display}` has no item `{name}`"),
                        u.span,
                    ));
                    continue;
                }
                if let Some(e) = ty {
                    if e.is_pub {
                        scope.type_scope.insert(name.clone(), e.global.clone());
                    } else {
                        diags.push(private_err(display, name, u.span));
                    }
                }
                if let Some(e) = val {
                    if e.is_pub {
                        scope.value_scope.insert(name.clone(), e.global.clone());
                    } else {
                        diags.push(private_err(display, name, u.span));
                    }
                }
            }
        }
    }
}

/// The cross-package visibility-wall error (design 0008 §3 / 0017 §5): an item
/// whose module is not the dependency's externally-reachable public root.
fn boundary_private_err(dep: &str, display: &str, name: &str, span: Span) -> Diag {
    Diag::error(
        "E0903",
        format!("`{name}` is not part of dependency `{dep}`'s public API (`{display}` is not re-exported from its public root)"),
        span,
    )
    .with_note("only a dependency's public-root `pub` surface crosses the package boundary (design 0008 §3 / 0017 §5)", None)
}

/// The root package's binary entry must be present with `fn main` (design 0017
/// §1/§7); a library-only root requires no entry.
fn check_multi_entry(
    res: &crate::resolve_pkg::Resolution,
    index: &HashMap<String, usize>,
    modules: &[Module],
    diags: &mut Vec<Diag>,
) {
    let Some(rel) = &res.root_entry else { return };
    let mut p = vec![res.root_pkg().pkgid.clone()];
    p.extend(rel.iter().cloned());
    let file = format!("src/{}.cnr", rel.join("/"));
    match index.get(&path_str(&p)) {
        Some(&mi) if modules[mi].value_exports.contains_key("main") => {}
        Some(_) => diags.push(entry_no_main_diag(&file)),
        None => diags.push(entry_missing_diag(&file)),
    }
}

/// The path-granularity entry-presence gate for the multi-package incremental
/// build (`candor build`): the root package's binary entry module (pkgid-prefixed)
/// must be present. Mirrors [`check_entry_present`] — presence only, never the
/// `fn main` body (the incremental builder knows paths, not parsed items).
pub fn check_multi_entry_present(
    res: &crate::resolve_pkg::Resolution,
    has_path: impl Fn(&str) -> bool,
    diags: &mut Vec<Diag>,
) {
    let Some(rel) = &res.root_entry else { return };
    let mut p = vec![res.root_pkg().pkgid.clone()];
    p.extend(rel.iter().cloned());
    let file = format!("src/{}.cnr", rel.join("/"));
    if !has_path(&path_str(&p)) {
        diags.push(entry_missing_diag(&file));
    }
}

/// The multi-package name-collision gate for the incremental builder (design 0017
/// §5): a root dependency's local name must not equal a root top-level
/// `src/<name>.cnr` module. Mirrors [`build_tree_multi`]'s E0930 so the
/// incremental build agrees with `check`. `paths` are the discovered module paths.
pub fn check_dep_name_collisions<'a>(
    res: &crate::resolve_pkg::Resolution,
    paths: impl Iterator<Item = &'a [String]>,
    diags: &mut Vec<Diag>,
) {
    let root_pkgid = &res.root_pkg().pkgid;
    let root_top: std::collections::HashSet<String> = paths
        .filter(|p| p.len() == 2 && p.first() == Some(root_pkgid))
        .filter_map(|p| p.get(1).cloned())
        .collect();
    for dep_local in res.root_pkg().deps.keys() {
        if root_top.contains(dep_local) {
            diags.push(dep_name_collision_diag(dep_local));
        }
    }
}

/// The dependency-name / local-module collision error (design 0017 §5): a root
/// dependency's local name equals a root top-level `src/<name>.cnr` module.
/// Shared by the merge builder and the incremental builder.
fn dep_name_collision_diag(dep_local: &str) -> Diag {
    Diag::error(
        "E0930",
        format!("dependency name `{dep_local}` collides with the local top-level module `src/{dep_local}.cnr`"),
        Span::point(0),
    )
    .with_note("rename the module, or alias the dependency with a distinct `[dependencies]` key + `package = \"…\"` (design 0017 §5)", None)
}

/// Per-module data for the Stage-C incremental build (design 0008 §2): one node
/// of the import DAG, with its qualified items, `pub` flags, source text, and
/// resolved import edges (indices into [`TreeParts::modules`]).
pub struct ModuleParts {
    pub path: Vec<String>,
    pub source: String,
    pub boundary: bool,
    pub items: Vec<Item>,
    pub is_pub: Vec<bool>,
    pub imports: Vec<usize>,
}

/// The module tree as per-module parts (Stage C), plus the module-layer diags.
pub struct TreeParts {
    pub modules: Vec<ModuleParts>,
    pub diags: Vec<Diag>,
}

/// Build a module tree rooted at `dir` into **per-module** parts (not merged):
/// the Stage-C interface-artifact builder consumes this to hash and check each
/// module independently over the DAG.
pub fn build_tree_parts(dir: &Path) -> Result<TreeParts, Diag> {
    let root = resolve_dir_root(dir)?;
    let Assembled { mut modules, scopes, adj, mut diags } = assemble(&root)?;
    let meta: Vec<(Vec<String>, String, bool, Vec<bool>)> = modules
        .iter()
        .map(|m| (m.path.clone(), m.source.clone(), m.boundary, m.is_pub.clone()))
        .collect();
    let per_module = qualify(&mut modules, &scopes, &mut diags);
    let mut out = Vec::with_capacity(per_module.len());
    for (i, items) in per_module.into_iter().enumerate() {
        let mut imports: Vec<usize> = adj[i].iter().map(|(t, _)| *t).collect();
        imports.sort_unstable();
        imports.dedup();
        let (path, source, boundary, is_pub) = meta[i].clone();
        out.push(ModuleParts { path, source, boundary, items, is_pub, imports });
    }
    Ok(TreeParts { modules: out, diags })
}

/// The exported type/value name tables of a parsed module (design 0008 §2).
/// Shared by the full assembler and the Stage-C2 incremental [`parse_one`].
fn collect_exports(
    path: &[String],
    program: &Program,
    vis: &[bool],
    entry: Option<&[String]>,
) -> (HashMap<String, Export>, HashMap<String, Export>) {
    let mut type_exports = HashMap::new();
    let mut value_exports = HashMap::new();
    for (item, &is_pub) in program.items.iter().zip(vis.iter()) {
        let (name, kind, is_fn) = match item {
            Item::Struct(s) => (&s.name, Kind::Type, false),
            Item::Enum(e) => (&e.name, Kind::Type, false),
            Item::Fn(f) => (&f.name, Kind::Value, true),
            Item::Static(s) => (&s.name, Kind::Value, false),
            Item::Interface(i) => (&i.name, Kind::Type, false),
            Item::Impl(_) => continue,
            Item::Extern(_) | Item::Export(_) => continue,
        };
        let export = Export { global: mangle(path, name, kind, entry), is_pub, is_fn };
        match kind {
            Kind::Type => {
                type_exports.insert(name.clone(), export);
            }
            Kind::Value => {
                value_exports.insert(name.clone(), export);
            }
        }
    }
    (type_exports, value_exports)
}

/// Discover every `.cnr` module file under `dir` (path + file), sorted — the
/// present-source universe of the Stage-C2 incremental build (design 0008 §2).
pub fn discover_module_files(dir: &Path) -> Result<Vec<(Vec<String>, PathBuf)>, Diag> {
    let mut out = Vec::new();
    discover(dir, &[], &mut out)?;
    Ok(out)
}

/// The parsed, still-**unqualified** parts of one module file (Stage-C2): its
/// items, `use` decls, boundary marker, and its own export tables. A dirty
/// module is parsed with this; [`qualify_one`] then qualifies it against its
/// imports' [`ModuleExports`] (cached or fresh) with no other module parsed.
pub struct ParsedOne {
    pub items: Vec<Item>,
    pub uses: Vec<UseDecl>,
    pub boundary: bool,
    /// `pub` flag parallel to `items` (the interface-surface filter, design 0008 §2).
    pub is_pub: Vec<bool>,
    pub exports: ModuleExports,
}

/// Parse one module's source into its unqualified parts (Stage-C2). No other
/// module is touched — the signature-only incremental build parses exactly the
/// modules whose own source changed. `entry` is the module path whose `fn main`
/// keeps the bare global name (the single-package `["main"]`, or the root
/// package's pkgid-prefixed entry in a multi-package build).
pub fn parse_one(path: &[String], source: &str, entry: Option<&[String]>) -> Result<ParsedOne, Diag> {
    let (program, uses, vis, boundary) = crate::real::parse_module(source)?;
    let (type_exports, value_exports) = collect_exports(path, &program, &vis, entry);
    Ok(ParsedOne {
        items: program.items,
        uses,
        boundary,
        is_pub: vis,
        exports: ModuleExports { path: path.to_vec(), type_exports, value_exports },
    })
}

/// Qualify a single parsed module's items to global names (Stage-C2), resolving
/// its `use` imports against `imports` — the export tables of its imported
/// modules, each either freshly parsed (a dirty import) or reconstructed from a
/// cached interface artifact (a reused import). Mirrors the full assembler's
/// per-module scope/rename/rewrite, but for one module with no whole-tree parse.
/// Returns the qualified items and any module-layer (import/visibility) diags.
pub fn qualify_one(
    parsed: &ParsedOne,
    imports: &HashMap<String, ModuleExports>,
    resolver: &ImportResolver,
) -> (Vec<Item>, Vec<Diag>) {
    let mut diags = Vec::new();
    // Synthetic module table: index 0 is self; the rest are the imports (keyed by
    // their full, resolved module path). Only the export tables + path are
    // consulted (by the alias-ctor rewrite and rename).
    let mut synth: Vec<Module> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    synth.push(synth_module(&parsed.exports));
    index.insert(path_str(&parsed.exports.path), 0);
    for (mpath, me) in imports {
        index.insert(mpath.clone(), synth.len());
        synth.push(synth_module(me));
    }

    // Build this module's scope: own exports, then each `use` import — resolved
    // through the shared resolver so the incremental build binds cross-package
    // imports exactly as the merge builder does (design 0017 Open-Q4).
    let mut scope = Scope::default();
    for (name, e) in &parsed.exports.type_exports {
        scope.type_scope.insert(name.clone(), e.global.clone());
    }
    for (name, e) in &parsed.exports.value_exports {
        scope.value_scope.insert(name.clone(), e.global.clone());
    }
    for u in &parsed.uses {
        let display = u.segments.join("::");
        match resolver.resolve_use(&u.segments) {
            UseResolution::Intra { target } => match index.get(&path_str(&target)) {
                Some(&tidx) => bind_use(u, &display, tidx, &synth, &mut scope, &mut diags),
                None => diags.push(
                    Diag::error("E0901", format!("unresolved import: no module `{display}`"), u.span)
                        .with_note("a module is a `.cnr` file; `a::b` is the file `a/b.cnr`", None),
                ),
            },
            UseResolution::Cross { dep_local, target, is_public_root } => {
                match index.get(&path_str(&target)) {
                    Some(&tidx) if is_public_root => {
                        bind_use(u, &display, tidx, &synth, &mut scope, &mut diags)
                    }
                    _ => {
                        for name in u.names.as_deref().unwrap_or(&[]) {
                            diags.push(boundary_private_err(&dep_local, &display, name, u.span));
                        }
                        if u.names.is_none() {
                            diags.push(boundary_private_err(&dep_local, &display, "*", u.span));
                        }
                    }
                }
            }
        }
    }

    // Rename own declarations to global names, then rewrite bodies/types.
    let selfmod = &synth[0];
    let mut out = Vec::with_capacity(parsed.items.len());
    for mut item in parsed.items.iter().cloned() {
        rename_item(selfmod, &mut item);
        let mut rw = Rewriter { scope: &scope, modules: &synth, diags: &mut diags };
        rw.rewrite_item(&mut item);
        out.push(item);
    }
    (out, diags)
}

/// A synthetic [`Module`] carrying only export tables + path (Stage-C2 qualify).
fn synth_module(me: &ModuleExports) -> Module {
    Module {
        path: me.path.clone(),
        program: Program { items: Vec::new() },
        uses: Vec::new(),
        type_exports: me.type_exports.clone(),
        value_exports: me.value_exports.clone(),
        source: String::new(),
        boundary: false,
        is_pub: Vec::new(),
    }
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
        // Foreign symbols keep their literal C names (design 0011); they are not
        // module-qualified.
        Item::Extern(_) | Item::Export(_) => {}
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
            Item::Extern(eb) => {
                for ef in &mut eb.fns {
                    for p in &mut ef.params {
                        self.rewrite_ty(&mut p.ty);
                    }
                    if let Some(rt) = &mut ef.ret {
                        self.rewrite_ty(&mut rt.ty);
                    }
                }
            }
            Item::Export(ex) => {
                for p in &mut ex.params {
                    self.rewrite_ty(&mut p.ty);
                }
                if let Some(rt) = &mut ex.ret {
                    self.rewrite_ty(&mut rt.ty);
                }
                if let Some(g) = self.scope.value_scope.get(&ex.candor_fn) {
                    ex.candor_fn = g.clone();
                }
            }
            Item::Impl(im) => {
                for tp in &mut im.type_params {
                    self.qualify_bounds(&mut tp.bounds);
                }
                if let Some(g) = self.scope.type_scope.get(&im.iface) {
                    im.iface = g.clone();
                }
                for a in &mut im.iface_args {
                    self.rewrite_ty(a);
                }
                self.rewrite_ty(&mut im.target);
                if let Some((_, ty)) = &mut im.assoc_binding {
                    self.rewrite_ty(ty);
                }
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
            ExprKind::For { .. } => unreachable!("`for` is surface-only (formatter); the pipeline desugars it at parse (design 0009 §4.2)"),
            ExprKind::Scope(b) => self.rewrite_block(b, locals),
            ExprKind::Spawn(c) => self.rewrite_expr(c, locals),
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
            | ExprKind::Bitcast { ty, expr: e }
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
            | ExprKind::FloatLit { .. }
            | ExprKind::StrLit(_)
            | ExprKind::BytesLit(_)
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
            PatKind::IntLit { .. } | PatKind::IntRange { .. } => {}
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
