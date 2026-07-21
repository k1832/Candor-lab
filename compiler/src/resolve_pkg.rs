//! Cross-package dependency resolution and `candor.lock` (design 0017 §6/§7).
//!
//! From a root (buildable) package's `[dependencies]`, this transitively loads
//! each dependency's `candor.toml` (via [`crate::manifest::load_manifest`]) and
//! produces the **pinned package set**: every reachable package with its
//! resolved source, an **injective package id** (name + resolved-source hash,
//! design 0017 §5 / review F2), and a per-package map from each local dependency
//! name to the resolved pkgid it names.
//!
//! Selection is exact-source pinning with **single-version unification** (§6):
//! the same package name reached via the **same** source is one build node
//! (deduped, keyed by canonicalized directory); the same name via **different**
//! sources is a hard conflict error (P4). The package graph must be acyclic (§8).
//!
//! **Scope:** path and git dependencies are both resolved. A git dependency is
//! fetched into a content-addressed cache ([`crate::pkg_fetch`]) keyed by its url
//! and resolved commit sha, then treated exactly like a path dependency (its
//! checkout feeds the same transitive-load + build path); the lockfile records
//! the git url, the resolved sha, and the content hash (design 0017 §4/§6).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::build::sha256;
use crate::diag::Diag;
use crate::manifest::{self, Edition, Manifest, Source, Version};
use crate::span::Span;

/// A resolved package in the pinned build set (design 0017 §6).
pub struct ResolvedPackage {
    /// The injective package id `name#<source-hash>` (design 0017 §5, review F2):
    /// the first mangled-name segment of every item this package contributes.
    pub pkgid: String,
    pub name: String,
    pub version: Version,
    pub edition: String,
    pub freestanding: bool,
    /// The canonicalized package root directory (holding `candor.toml`).
    pub dir: PathBuf,
    /// The module-tree root: `dir/src` (design 0017 §1).
    pub src_root: PathBuf,
    /// The resolved, pinned source (canonical path, or git url+rev).
    pub source: ResolvedSource,
    /// A content hash over the package's `src/` sources (reproducibility, §6).
    pub content_hash: String,
    /// `true` for the buildable root package.
    pub is_root: bool,
    /// The src-relative module path of the package's public root — its external
    /// API surface (design 0008 §3 external reachability; a lib's `src/<name>.cnr`).
    pub public_root: Vec<String>,
    /// Local dependency name -> resolved pkgid (drives cross-package `use`, §5).
    pub deps: BTreeMap<String, String>,
}

impl ResolvedPackage {
    /// This package's surface [`Edition`] (1.0-gate item 1). Total because the
    /// manifest was validated when it was loaded.
    pub fn edition_kind(&self) -> Edition {
        Edition::from_field(&self.edition).unwrap_or_default()
    }
}

/// A pinned dependency source (design 0017 §4/§6).
#[derive(Clone)]
pub enum ResolvedSource {
    /// A canonicalized local directory.
    Path(PathBuf),
    /// A git source pinned to an exact commit: its url and the resolved commit
    /// sha the build is pinned to (fetched into the content-addressed cache).
    Git { url: String, rev: String },
}

/// The resolved build: the pinned package set in dependency-topological order
/// (dependencies before dependents), plus the root package's index.
pub struct Resolution {
    pub packages: Vec<ResolvedPackage>,
    pub root: usize,
    /// The src-relative module path of the root's `fn main` entry, if it is a
    /// binary (`["main"]` for the implicit single binary) — the one item whose
    /// mangled name stays the bare global `main` so the entry is found.
    pub root_entry: Option<Vec<String>>,
    /// Whether a pre-existing, consistent `candor.lock` was reused verbatim (§6).
    pub lock_reused: bool,
}

impl Resolution {
    pub fn root_pkg(&self) -> &ResolvedPackage {
        &self.packages[self.root]
    }

    /// The root package's `fn main` entry as a full combined module path
    /// (`[pkgid, "main"]`), for the mangle's bare-`main` special case.
    pub fn entry_module(&self) -> Option<Vec<String>> {
        self.root_entry.as_ref().map(|rel| {
            let mut p = vec![self.root_pkg().pkgid.clone()];
            p.extend(rel.iter().cloned());
            p
        })
    }
}

fn diag(code: &str, message: impl Into<String>) -> Diag {
    Diag::error(code, message, Span::point(0))
}

/// A node of the package graph during resolution.
struct Node {
    dir: PathBuf,
    manifest: Manifest,
    /// This node's pinned source (path directory, or fetched git checkout).
    source: ResolvedSource,
    /// The dependency chain that first reached this node (package names), for the
    /// conflict / cycle diagnostics.
    via: Vec<String>,
    /// Local dependency name -> resolved node index.
    deps: Vec<(String, usize)>,
}

/// Resolve the pinned package set from a root package directory (design 0017 §6).
/// The caller guarantees `root_dir` has a manifest with at least one dependency
/// (a manifest-less or dependency-free package uses the single-package path).
pub fn resolve(root_dir: &Path) -> Result<Resolution, Diag> {
    let (root_dir, mut resolution) = resolve_graph(root_dir)?;
    check_freestanding_composition(&resolution)?;
    let summaries = trust_summaries(&resolution.packages)?;
    resolution.lock_reused = write_or_verify_lock(&root_dir, &resolution.packages, &summaries)?;
    Ok(resolution)
}

/// The audit-free resolver core (design 0017 §6): the transitive graph walk that
/// produces the pinned package set. It never touches `audit`; the trust-summary
/// augmentation happens in the post-resolution pass [`trust_summaries`] driven by
/// [`resolve`]. Returns the canonicalized root dir alongside the resolution (the
/// lock is written by the caller, not here).
fn resolve_graph(root_dir: &Path) -> Result<(PathBuf, Resolution), Diag> {
    let root_dir = canonicalize(root_dir, "root package")?;
    let root_manifest = match manifest::load_manifest(&root_dir) {
        Ok(Some(m)) => m,
        Ok(None) => return Err(diag("E0920", "internal: resolve called on a manifest-less directory")),
        Err(e) => return Err(diag(e.code, format!("cannot read package manifest: {}", e.message))),
    };

    let mut nodes: Vec<Node> = Vec::new();
    let mut dir_index: BTreeMap<PathBuf, usize> = BTreeMap::new();
    let mut name_index: BTreeMap<String, usize> = BTreeMap::new();

    let root_name = root_manifest.package.name.clone();
    let root_source = ResolvedSource::Path(root_dir.clone());
    push_node(&mut nodes, &mut dir_index, &mut name_index, root_dir.clone(), root_source, root_manifest, vec![root_name])?;

    // BFS: resolve each node's dependencies, creating or deduping nodes.
    let mut i = 0;
    while i < nodes.len() {
        // Take what we need without holding a borrow across the mutation of `nodes`.
        let dir = nodes[i].dir.clone();
        let via = nodes[i].via.clone();
        let dep_specs = nodes[i].manifest.dependencies.clone();
        let mut deps: Vec<(String, usize)> = Vec::new();
        for dep in &dep_specs {
            let dep_name = dep.package.clone().unwrap_or_else(|| dep.local_name.clone());
            let (dep_dir, dep_source) = resolve_source(&dir, dep)?;
            let mut child_via = via.clone();
            child_via.push(dep_name.clone());
            let idx = match dir_index.get(&dep_dir) {
                Some(&idx) => idx,
                None => {
                    let m = match manifest::load_manifest(&dep_dir) {
                        Ok(Some(m)) => m,
                        Ok(None) => {
                            return Err(diag(
                                "E0921",
                                format!(
                                    "dependency `{}` at `{}` has no `candor.toml` (not a package; design 0017 §1)",
                                    dep.local_name,
                                    dep_dir.display()
                                ),
                            ))
                        }
                        Err(e) => {
                            return Err(diag(
                                e.code,
                                format!("dependency `{}`: cannot read manifest: {}", dep.local_name, e.message),
                            ))
                        }
                    };
                    if let Some(alias) = &dep.package {
                        if &m.package.name != alias {
                            return Err(diag(
                                "E0922",
                                format!(
                                    "dependency `{}` declares `package = \"{}\"` but `{}` names package `{}`",
                                    dep.local_name, alias, dep_dir.display(), m.package.name
                                ),
                            ));
                        }
                    }
                    push_node(&mut nodes, &mut dir_index, &mut name_index, dep_dir.clone(), dep_source, m, child_via)?
                }
            };
            deps.push((dep.local_name.clone(), idx));
        }
        nodes[i].deps = deps;
        i += 1;
    }

    // Package-level acyclicity (design 0017 §8).
    if let Some(cycle) = find_cycle(&nodes) {
        return Err(cycle_diag(&nodes, &cycle));
    }

    // Assemble resolved packages (topological order: dependencies before dependents).
    let order = topo_order(&nodes);
    let root_entry = root_entry_of(&nodes[0].manifest);
    let mut packages: Vec<ResolvedPackage> = Vec::with_capacity(nodes.len());
    let mut node_to_pkg: BTreeMap<usize, usize> = BTreeMap::new();
    for &n in &order {
        node_to_pkg.insert(n, packages.len());
        packages.push(resolved_package(&nodes[n], n == 0)?);
    }
    // Fill each package's dep pkgid map now that every package has a pkgid.
    for &n in &order {
        let pkg_idx = node_to_pkg[&n];
        let deps: BTreeMap<String, String> = nodes[n]
            .deps
            .iter()
            .map(|(local, child)| (local.clone(), packages[node_to_pkg[child]].pkgid.clone()))
            .collect();
        packages[pkg_idx].deps = deps;
    }
    let root = node_to_pkg[&0];

    Ok((root_dir, Resolution { packages, root, root_entry, lock_reused: false }))
}

/// Enforce the root package's `freestanding` claim across the whole resolved
/// graph (design 0017 §8, review F5). When the ROOT declares `freestanding =
/// true`, the final artifact links no libc (0011 §5), so **no** package in the
/// transitive graph may contribute a `foreign` boundary surface, and the graph
/// must not import the `std` package (0008 §5).
///
/// The gate is the **root's** flag — the final artifact's property — not each
/// dependency's own claim: a dependency's boundary surface is rejected regardless
/// of what the dependency itself declares, because the surface (not the flag) is
/// what has no libc to link against. The boundary/foreign surface is read from the
/// audit's structural enumeration ([`crate::audit::first_boundary_surface`]) — the
/// single source of truth for a package's boundary modules and `foreign` externs —
/// so this consumes the same data `candor audit` reports rather than re-walking.
///
/// Packages are visited in the resolution's deterministic topological order, and
/// each package's surface is itself deterministically ordered, so the first
/// violation reported is stable.
fn check_freestanding_composition(res: &Resolution) -> Result<(), Diag> {
    if !res.root_pkg().freestanding {
        return Ok(());
    }
    let root = &res.root_pkg().name;
    for pkg in &res.packages {
        // A transitive `std` import (0008 §5). Under P9's package layering `std`
        // is itself a package, so the `std` package appearing anywhere in the
        // resolved set *is* a std import sneaking into a freestanding graph; a
        // freestanding target must import only `core`/pure packages.
        if pkg.name == "std" {
            return Err(freestanding_std_diag(root));
        }
        // The load-bearing F5 case: a transitive `boundary`/`foreign` surface. A
        // declared `foreign` extern (even one never called) has no libc to bind to
        // under freestanding (0011 §5).
        if let Some(surface) = crate::audit::first_boundary_surface(&pkg.src_root, pkg.edition_kind())? {
            return Err(freestanding_boundary_diag(root, pkg, &surface));
        }
    }
    Ok(())
}

fn freestanding_boundary_diag(
    root: &str,
    pkg: &ResolvedPackage,
    surface: &crate::audit::BoundarySurface,
) -> Diag {
    let item = match &surface.foreign_extern {
        Some(ext) => {
            format!("boundary module `{}` declaring `foreign` extern `{}`", surface.module, ext)
        }
        None => format!("boundary module `{}`", surface.module),
    };
    let subject = if pkg.is_root {
        format!("package `{}` itself contributes a {item}", pkg.name)
    } else {
        format!("dependency `{}` contributes a {item}", pkg.name)
    };
    diag("E0935", format!("freestanding composition rejected: {subject}"))
        .with_note(
            format!(
                "the root package `{root}` declares `freestanding = true`, which links no libc: a transitive `foreign` extern has nothing to bind to (design 0011 §5)"
            ),
            None,
        )
        .with_note(
            "drop the foreign boundary surface from the transitive graph, or drop the `freestanding` claim (design 0017 §8, review F5)",
            None,
        )
}

fn freestanding_std_diag(root: &str) -> Diag {
    diag(
        "E0935",
        format!("freestanding composition rejected: the resolved graph of `{root}` imports the `std` package"),
    )
    .with_note(
        "`std` is not freestanding-compatible: a freestanding target must import only `core`/pure packages (design 0008 §5)",
        None,
    )
}

/// Compute each resolved package's trust summary and write/verify the lock. This
/// is the **post-resolution pass** (design 0017 §8): the resolver core above
/// ([`resolve_graph`]) is audit-free — it never references `audit`. Only here,
/// once the pinned package set is fixed, do we walk each package's audit surface
/// for its trust summary and augment the lock, keeping the resolver/audit
/// coupling out (the reason the trust summary was deferred from the whole-graph
/// audit slice).
fn trust_summaries(packages: &[ResolvedPackage]) -> Result<Vec<crate::audit::TrustSummary>, Diag> {
    // Reuse the impl #4 per-package audit enumeration to COUNT boundary modules,
    // foreign externs, and `unsafe` regions over the same `src/` tree the content
    // hash already covers — so the counts are reproducible with the sources.
    packages.iter().map(|p| crate::audit::trust_counts(&p.src_root, p.edition_kind())).collect()
}

/// Create a graph node, enforcing single-version unification (design 0017 §6):
/// a package name reached via a **different** canonical source is a hard conflict.
fn push_node(
    nodes: &mut Vec<Node>,
    dir_index: &mut BTreeMap<PathBuf, usize>,
    name_index: &mut BTreeMap<String, usize>,
    dir: PathBuf,
    source: ResolvedSource,
    manifest: Manifest,
    via: Vec<String>,
) -> Result<usize, Diag> {
    let name = manifest.package.name.clone();
    if let Some(&existing) = name_index.get(&name) {
        if nodes[existing].dir != dir {
            return Err(conflict_diag(&name, &nodes[existing], &dir, &via));
        }
    }
    let idx = nodes.len();
    dir_index.insert(dir.clone(), idx);
    name_index.insert(name, idx);
    nodes.push(Node { dir, manifest, source, via, deps: Vec::new() });
    Ok(idx)
}

fn conflict_diag(name: &str, existing: &Node, new_dir: &Path, new_via: &[String]) -> Diag {
    diag(
        "E0923",
        format!("dependency conflict: package `{name}` is required from two different sources"),
    )
    .with_note(
        format!("via {} -> `{}`", existing.via.join(" -> "), existing.dir.display()),
        None,
    )
    .with_note(format!("via {} -> `{}`", new_via.join(" -> "), new_dir.display()), None)
    .with_note(
        "single-version unification requires one source per package name (design 0017 §6); reconcile the two",
        None,
    )
}

/// Resolve a dependency to its package directory and pinned source (design 0017
/// §4). A **path** dep is resolved relative to the depending manifest and
/// canonicalized. A **git** dep is fetched into the content-addressed cache
/// ([`crate::pkg_fetch`]); its checkout is then treated exactly like a path
/// directory, and the resolved commit sha (recorded in the lock) pins the build.
fn resolve_source(
    from_dir: &Path,
    dep: &manifest::Dependency,
) -> Result<(PathBuf, ResolvedSource), Diag> {
    match &dep.source {
        Source::Path { path } => {
            let joined = from_dir.join(path);
            let dir = canonicalize(&joined, &format!("dependency `{}`", dep.local_name))?;
            Ok((dir.clone(), ResolvedSource::Path(dir)))
        }
        Source::Git { url, rev, tag, branch } => {
            let checkout = crate::pkg_fetch::fetch_git(url, rev, tag.as_deref(), branch.as_deref())
                .map_err(|d| {
                    Diag::error(
                        &d.code,
                        format!("git dependency `{}`: {}", dep.local_name, d.message),
                        d.span,
                    )
                })?;
            let source = ResolvedSource::Git { url: url.clone(), rev: checkout.resolved_rev };
            Ok((checkout.dir, source))
        }
    }
}

fn canonicalize(path: &Path, what: &str) -> Result<PathBuf, Diag> {
    path.canonicalize().map_err(|e| {
        diag("E0925", format!("cannot resolve {what} path `{}`: {e}", path.display()))
    })
}

/// The root package's `fn main` entry module (src-relative), if it builds a
/// binary. Mirrors the manifest target model (design 0017 §1/§2): an implicit
/// single binary is `src/main.cnr`; a `[[bin]]` names `src/bin/<name>.cnr`.
fn root_entry_of(m: &Manifest) -> Option<Vec<String>> {
    if !m.bins.is_empty() {
        Some(vec!["bin".to_string(), m.bins[0].name.clone()])
    } else if m.lib.is_none() {
        Some(vec!["main".to_string()])
    } else {
        None
    }
}

fn resolved_package(node: &Node, is_root: bool) -> Result<ResolvedPackage, Diag> {
    let src_root = node.dir.join("src");
    let content_hash = hash_sources(&src_root)?;
    let source = node.source.clone();
    let pkgid = pkgid_of(&node.manifest.package.name);
    Ok(ResolvedPackage {
        pkgid,
        name: node.manifest.package.name.clone(),
        version: node.manifest.package.version,
        edition: node.manifest.package.edition.clone(),
        freestanding: node.manifest.package.freestanding,
        dir: node.dir.clone(),
        src_root,
        source,
        content_hash,
        is_root,
        public_root: vec![node.manifest.package.name.clone()],
        deps: BTreeMap::new(),
    })
}

/// The injective package id (design 0017 §5, review F2): the package **name**.
/// Single-version-per-package unification (E0923, `create_node`) makes names
/// unique within any successfully-resolved build, so the name alone is injective;
/// no source hash is needed (and hashing content would rename every symbol on a
/// body edit, breaking §7 incrementality — 2026-07-15 erratum). Every item —
/// including the root package's own — is prefixed by its package's pkgid, so the
/// leading path segment uniquely identifies the owning package: two same-named
/// modules from different packages never merge into one node of the flat program
/// (securing cross-package acyclicity, review F6b).
fn pkgid_of(name: &str) -> String {
    name.to_string()
}

/// A deterministic content hash over a package's `src/` sources (design 0017 §6):
/// the sorted (relative-path, bytes) pairs, so it is stable across hosts (NN#16).
fn hash_sources(src_root: &Path) -> Result<String, Diag> {
    let files = crate::modules::discover_module_files(src_root)?;
    let mut sorted: Vec<(String, PathBuf)> = files.iter().map(|(p, f)| (p.join("::"), f.clone())).collect();
    sorted.sort();
    let mut buf: Vec<u8> = Vec::new();
    for (path, file) in sorted {
        let bytes = std::fs::read(&file)
            .map_err(|e| diag("E0926", format!("cannot read source `{}`: {e}", file.display())))?;
        buf.extend_from_slice(path.as_bytes());
        buf.push(0);
        buf.extend_from_slice(&sha256::hex(&bytes).into_bytes());
        buf.push(b'\n');
    }
    Ok(sha256::hex(&buf))
}

// ---------------------------------------------------------------------------
// Package-graph cycle detection + topological order (design 0017 §8).
// ---------------------------------------------------------------------------

fn find_cycle(nodes: &[Node]) -> Option<Vec<usize>> {
    let mut state = vec![0u8; nodes.len()]; // 0 white, 1 gray, 2 black
    let mut stack = Vec::new();
    for s in 0..nodes.len() {
        if state[s] == 0 {
            if let Some(c) = cycle_dfs(s, nodes, &mut state, &mut stack) {
                return Some(c);
            }
        }
    }
    None
}

fn cycle_dfs(u: usize, nodes: &[Node], state: &mut [u8], stack: &mut Vec<usize>) -> Option<Vec<usize>> {
    state[u] = 1;
    stack.push(u);
    for &(_, v) in &nodes[u].deps {
        if state[v] == 1 {
            let pos = stack.iter().position(|&x| x == v).unwrap();
            let mut cyc = stack[pos..].to_vec();
            cyc.push(v);
            return Some(cyc);
        } else if state[v] == 0 {
            if let Some(c) = cycle_dfs(v, nodes, state, stack) {
                return Some(c);
            }
        }
    }
    stack.pop();
    state[u] = 2;
    None
}

fn cycle_diag(nodes: &[Node], cycle: &[usize]) -> Diag {
    let names: Vec<String> = cycle.iter().map(|&i| nodes[i].manifest.package.name.clone()).collect();
    diag("E0927", format!("package dependency cycle: {}", names.join(" -> ")))
        .with_note("a package cannot transitively depend on itself (design 0017 §8)", None)
}

/// Dependencies before dependents, ties broken by node index — deterministic.
fn topo_order(nodes: &[Node]) -> Vec<usize> {
    let mut done = vec![false; nodes.len()];
    let mut order = Vec::with_capacity(nodes.len());
    for _ in 0..nodes.len() {
        let next = (0..nodes.len())
            .find(|&n| !done[n] && nodes[n].deps.iter().all(|&(_, d)| done[d]));
        match next {
            Some(n) => {
                done[n] = true;
                order.push(n);
            }
            None => break, // a cycle (already diagnosed) — bail deterministically.
        }
    }
    order
}

// ---------------------------------------------------------------------------
// candor.lock (design 0017 §6): TOML, written at the root package.
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct LockFile {
    package: Vec<LockEntry>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct LockEntry {
    name: String,
    version: String,
    edition: String,
    content_hash: String,
    /// The per-package trust summary (design 0017 §8). Additive: an older lock
    /// without it still parses (`default` = zeroes) and, if the real counts are
    /// zero, is reused; otherwise it is regenerated additively like any other
    /// toolchain-generated lock field.
    #[serde(default)]
    trust: crate::audit::TrustSummary,
    source: LockSource,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
enum LockSource {
    Path { path: String },
    Git { git: String, rev: String },
}

/// The path from `root` to `target` recorded in `candor.lock`'s `[package.source]`
/// (design 0017 §6). Both inputs are canonicalized absolute paths; the result is
/// relative, so moving the whole tree preserves every source path and the lock
/// stays portable across hosts (NN#16). A target under `root` strips the shared
/// prefix; a sibling or other location builds the `../` chain from the common
/// ancestor. The root package's own directory maps to `"."`.
fn relative_to(root: &Path, target: &Path) -> PathBuf {
    let root: Vec<_> = root.components().collect();
    let target: Vec<_> = target.components().collect();
    let common = root.iter().zip(&target).take_while(|(a, b)| a == b).count();
    let mut rel = PathBuf::new();
    for _ in common..root.len() {
        rel.push("..");
    }
    for component in &target[common..] {
        rel.push(component.as_os_str());
    }
    if rel.as_os_str().is_empty() {
        rel.push(".");
    }
    rel
}

fn lock_from(root_dir: &Path, packages: &[ResolvedPackage], summaries: &[crate::audit::TrustSummary]) -> LockFile {
    let mut entries: Vec<LockEntry> = packages
        .iter()
        .zip(summaries)
        .map(|(p, &trust)| LockEntry {
            name: p.name.clone(),
            version: format!("{}.{}.{}", p.version.major, p.version.minor, p.version.patch),
            edition: p.edition.clone(),
            content_hash: p.content_hash.clone(),
            trust,
            source: match &p.source {
                ResolvedSource::Path(dir) => LockSource::Path { path: relative_to(root_dir, dir).to_string_lossy().into_owned() },
                ResolvedSource::Git { url, rev } => LockSource::Git { git: url.clone(), rev: rev.clone() },
            },
        })
        .collect();
    entries.sort_by(|a, b| a.name.cmp(&b.name).then(a.content_hash.cmp(&b.content_hash)));
    LockFile { package: entries }
}

/// Write or verify `candor.lock` (design 0017 §6). A pre-existing lock that is
/// consistent with the freshly resolved set is reused verbatim (not rewritten, so
/// its bytes are stable); an absent or manifest-diverged lock is (re)written.
/// Returns whether the on-disk lock was reused.
///
/// Before overwriting a diverged-but-parseable lock, the **trust-delta gate**
/// (design 0017 §8, Open-Q1) runs: a lock update that grows any dependency's
/// foreign/`unsafe` surface is a hard error (`E0936`) unless the user accepts it
/// via `CANDOR_ACCEPT_TRUST_DELTA` — the committed lock's trust surface is not
/// silently expanded.
fn write_or_verify_lock(
    root_dir: &Path,
    packages: &[ResolvedPackage],
    summaries: &[crate::audit::TrustSummary],
) -> Result<bool, Diag> {
    let lock = lock_from(root_dir, packages, summaries);
    let text = format!(
        "# This file is generated by Candor's resolver (design 0017 §6).\n# It pins the exact resolved dependency set for reproducible builds.\n\n{}",
        toml::to_string(&lock).map_err(|e| diag("E0928", format!("cannot serialize candor.lock: {e}")))?
    );
    let path = root_dir.join("candor.lock");
    if let Ok(existing) = std::fs::read_to_string(&path) {
        if let Ok(parsed) = toml::from_str::<LockFile>(&existing) {
            if parsed == lock {
                return Ok(true);
            }
            // Trust-delta gate (design 0017 §8, Open-Q1): the fresh set diverged
            // from the reviewed lock; refuse to overwrite if the divergence *grows*
            // a dependency's foreign/`unsafe` surface. Initial creation (no or
            // unparseable lock) and a no-change rebuild (returned above) never
            // reach here, so establishing a baseline and a stable rebuild are
            // never gated.
            let root_name = packages.iter().find(|p| p.is_root).map(|p| p.name.as_str());
            if let Some(delta) = trust_delta_growth(&parsed, &lock, root_name) {
                if !accept_trust_delta() {
                    return Err(diag(
                        "E0936",
                        format!(
                            "candor.lock update introduces new dependency trust surface (design 0017 §8):\n{delta}\n\
                             a lock update must not silently grow the foreign/`unsafe` surface the committed lock vouched for.\n\
                             review the delta above, then re-run with CANDOR_ACCEPT_TRUST_DELTA=1 to accept it and update the lock.",
                        ),
                    ));
                }
                eprintln!(
                    "note: CANDOR_ACCEPT_TRUST_DELTA set; accepting the grown dependency trust surface and updating candor.lock:\n{delta}",
                );
            }
        }
    }
    std::fs::write(&path, text)
        .map_err(|e| diag("E0929", format!("cannot write `{}`: {e}", path.display())))?;
    Ok(false)
}

/// Detect a **growth** of dependency trust surface between the on-disk lock `old`
/// and the freshly resolved `fresh` (design 0017 §8, Open-Q1). Returns a
/// per-dependency delta description when any dependency's foreign/`unsafe` count
/// (`boundary_modules` / `externs` / `unsafe_regions`) increased, or a **new**
/// dependency appears carrying nonzero surface; `None` when the surface only
/// shrank, stayed equal, or a new dependency is pure. The `root` package's own
/// surface is the author's code, not a supply-chain delta, so it is never gated.
///
/// `assumed-proven` is not gated separately: this edition has no distinct
/// `assumed-proven` construct — its surface is the boundary `trust` clauses plus
/// `unsafe` justifications already counted (design 0017 §8) — so gating the
/// three recorded counts covers it; a dedicated count is a follow-up.
fn trust_delta_growth(
    old: &LockFile,
    fresh: &LockFile,
    root: Option<&str>,
) -> Option<String> {
    let prior: BTreeMap<&str, &crate::audit::TrustSummary> =
        old.package.iter().map(|e| (e.name.as_str(), &e.trust)).collect();
    let mut grown: Vec<String> = Vec::new();
    for entry in &fresh.package {
        if Some(entry.name.as_str()) == root {
            continue;
        }
        let now = &entry.trust;
        match prior.get(entry.name.as_str()) {
            Some(was) => {
                let mut deltas = Vec::new();
                if now.boundary_modules > was.boundary_modules {
                    deltas.push(format!("boundary_modules {} -> {}", was.boundary_modules, now.boundary_modules));
                }
                if now.externs > was.externs {
                    deltas.push(format!("externs {} -> {}", was.externs, now.externs));
                }
                if now.unsafe_regions > was.unsafe_regions {
                    deltas.push(format!("unsafe_regions {} -> {}", was.unsafe_regions, now.unsafe_regions));
                }
                if !deltas.is_empty() {
                    grown.push(format!("  {}: {}", entry.name, deltas.join(", ")));
                }
            }
            None if now.boundary_modules > 0 || now.externs > 0 || now.unsafe_regions > 0 => {
                grown.push(format!(
                    "  {} (new dependency): boundary_modules 0 -> {}, externs 0 -> {}, unsafe_regions 0 -> {}",
                    entry.name, now.boundary_modules, now.externs, now.unsafe_regions,
                ));
            }
            None => {}
        }
    }
    (!grown.is_empty()).then(|| grown.join("\n"))
}

/// Whether the user opted to accept a grown dependency trust surface, letting a
/// gated lock update proceed (design 0017 §8, Open-Q1). An env var rather
/// than a CLI flag because `write_or_verify_lock` sits behind `build_tree`'s many
/// callers plus the `build`/`audit` entries: threading a flag through every
/// re-resolving command is far more invasive than reading it here, mirroring the
/// `CANDOR_LINKER` precedent (an env var read at its point of use).
fn accept_trust_delta() -> bool {
    std::env::var_os("CANDOR_ACCEPT_TRUST_DELTA").is_some_and(|v| !v.is_empty())
}
