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
//! **Scope:** path dependencies are resolved fully. A git dependency is a clean
//! deferral — it errors rather than fetching (the content-addressed cache is a
//! follow-up); the resolver is shaped so a git source slots in as a fetched
//! directory without disturbing the path-dependency milestone.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::build::sha256;
use crate::diag::Diag;
use crate::manifest::{self, Manifest, Source, Version};
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

/// A pinned dependency source (design 0017 §4/§6).
pub enum ResolvedSource {
    /// A canonicalized local directory.
    Path(PathBuf),
    /// A git source pinned to an exact commit (fetch is deferred; see module docs).
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
    push_node(&mut nodes, &mut dir_index, &mut name_index, root_dir.clone(), root_manifest, vec![root_name])?;

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
            let dep_dir = resolve_source_dir(&dir, dep)?;
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
                    push_node(&mut nodes, &mut dir_index, &mut name_index, dep_dir.clone(), m, child_via)?
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

    let lock_reused = write_or_verify_lock(&root_dir, &packages)?;

    Ok(Resolution { packages, root, root_entry, lock_reused })
}

/// Create a graph node, enforcing single-version unification (design 0017 §6):
/// a package name reached via a **different** canonical source is a hard conflict.
fn push_node(
    nodes: &mut Vec<Node>,
    dir_index: &mut BTreeMap<PathBuf, usize>,
    name_index: &mut BTreeMap<String, usize>,
    dir: PathBuf,
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
    nodes.push(Node { dir, manifest, via, deps: Vec::new() });
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

/// Resolve a dependency's package directory (design 0017 §4). Path deps are
/// resolved relative to the depending manifest's directory and canonicalized;
/// git deps are a clean deferral (they error rather than fetch).
fn resolve_source_dir(from_dir: &Path, dep: &manifest::Dependency) -> Result<PathBuf, Diag> {
    match &dep.source {
        Source::Path { path } => {
            let joined = from_dir.join(path);
            canonicalize(&joined, &format!("dependency `{}`", dep.local_name))
        }
        Source::Git { url, rev, .. } => Err(diag(
            "E0924",
            format!(
                "git dependency `{}` ({url} @ {rev}) is not yet fetched: git sources are deferred; use a path dependency (design 0017 §4)",
                dep.local_name
            ),
        )),
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
    let source = ResolvedSource::Path(node.dir.clone());
    let pkgid = pkgid_of(&node.manifest.package.name, &node.dir);
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

/// The injective package id (design 0017 §5, review F2): the package name plus a
/// hash of its resolved source. The `#` separator can never occur in a source
/// identifier, so the pkgid segment cannot collide with any module name — which
/// is what keeps two same-named modules from different packages from merging into
/// one node of the flat program (securing cross-package acyclicity, review F6b).
fn pkgid_of(name: &str, canonical_dir: &Path) -> String {
    let hash = sha256::hex(canonical_dir.to_string_lossy().as_bytes());
    format!("{name}#{}", &hash[..16])
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
    source: LockSource,
    content_hash: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
enum LockSource {
    Path { path: String },
    Git { git: String, rev: String },
}

fn lock_from(packages: &[ResolvedPackage]) -> LockFile {
    let mut entries: Vec<LockEntry> = packages
        .iter()
        .map(|p| LockEntry {
            name: p.name.clone(),
            version: format!("{}.{}.{}", p.version.major, p.version.minor, p.version.patch),
            edition: p.edition.clone(),
            source: match &p.source {
                ResolvedSource::Path(dir) => LockSource::Path { path: dir.to_string_lossy().into_owned() },
                ResolvedSource::Git { url, rev } => LockSource::Git { git: url.clone(), rev: rev.clone() },
            },
            content_hash: p.content_hash.clone(),
        })
        .collect();
    entries.sort_by(|a, b| a.name.cmp(&b.name).then(a.content_hash.cmp(&b.content_hash)));
    LockFile { package: entries }
}

/// Write or verify `candor.lock` (design 0017 §6). A pre-existing lock that is
/// consistent with the freshly resolved set is reused verbatim (not rewritten, so
/// its bytes are stable); an absent or manifest-diverged lock is (re)written.
/// Returns whether the on-disk lock was reused.
fn write_or_verify_lock(root_dir: &Path, packages: &[ResolvedPackage]) -> Result<bool, Diag> {
    let lock = lock_from(packages);
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
        }
    }
    std::fs::write(&path, text)
        .map_err(|e| diag("E0929", format!("cannot write `{}`: {e}", path.display())))?;
    Ok(false)
}
