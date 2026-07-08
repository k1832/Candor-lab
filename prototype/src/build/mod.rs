//! Stage C — the incremental, two-hash build (design 0008 §2; design 0010 §3).
//!
//! `candor-proto build <dir>` realizes 0008's load-bearing P20 mechanism: a
//! per-module **interface artifact** carrying two content hashes, and a build
//! over the import DAG that reuses a module's analysis whenever its source and
//! every import's **signature hash** are unchanged. A body edit that leaves
//! every `pub` signature hash unchanged re-analyzes **nothing downstream**
//! (0008 §2's analysis-invalidation tier); a `pub`-signature edit invalidates
//! exactly the direct importers and, transitively, only those whose own
//! signature hash changes in turn (the §2 cascade).
//!
//! ## The artifact (per module, serde JSON — determinism matters, speed does not
//! yet): module path + boundary marker; the canonical, span-free **signatures**
//! of its `pub` items (and all impls, the module-level coherence unit); the
//! canonical **codegen bodies** of those items; the two hashes — a
//! **signature hash** over path+boundary+signatures and a **codegen hash** that
//! additionally covers the bodies; the imports' signature hashes it was built
//! against; and a `schema_salt` = `SCHEMA_VERSION` + toolchain identity (the F3
//! salt: a toolchain/schema bump invalidates every cached artifact by
//! construction, 0008 §2 / 0010 §3).
//!
//! ## Honest scope (stage C1). The prototype has no generic-MIR emission (Stage
//! A/B lower *monomorphized* programs), so item (3) — 0008's "checked MIR body"
//! — is stood in for by the source-derived canonical body ([`canon::item_body`]),
//! the faithful codegen-input proxy. And the prototype's checker is
//! whole-program: to re-check a dirty module against its imports' signatures we
//! reconstruct a sub-program of the module **plus the full items of its
//! transitive imports** (upstream context), because the checker consumes full
//! items rather than signature-only stubs. Consequence, stated so it cannot
//! drift: re-checking a module re-touches its **upstream** bodies as context,
//! but **never a downstream (dependent) module** — the zero-downstream P20 claim
//! the gate asserts is exactly true; the residual is the upstream re-touch.

pub mod canon;
pub mod sha256;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ast::{Item, Program};
use crate::diag::{Diag, Severity};
use crate::modules::{self, ModuleParts};

/// The MIR-schema version. Bumping it invalidates every cached codegen artifact
/// by construction (design 0008 §2 F3 salt; 0010 §3). The gate exercises this
/// via [`build_dir_with_salt`].
pub const SCHEMA_VERSION: &str = "candor-mir-schema-c1";

/// The blessed-toolchain identity that co-salts the codegen cache (0010 §3): the
/// compiler version and the pinned backend. Same source + **same toolchain** is
/// the NN#16 reproducibility contract; a toolchain bump must not silently reuse
/// machine code the old compiler emitted.
pub fn toolchain_version() -> String {
    format!("candor-proto {} / cranelift 0.132.3", env!("CARGO_PKG_VERSION"))
}

/// The full schema/toolchain salt recorded in every artifact.
pub fn schema_salt() -> String {
    format!("{SCHEMA_VERSION}|{}", toolchain_version())
}

/// Provenance recorded in the artifact for P16 auditability (design 0008 §2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Provenance {
    pub toolchain: String,
}

/// The per-module interface artifact (design 0008 §2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Artifact {
    /// (1) the module path.
    pub module_path: String,
    /// (1) the boundary marker (design 0008 §4 / 0011).
    pub boundary: bool,
    /// The schema/toolchain salt this artifact was emitted under (F3).
    pub schema_salt: String,
    /// SHA-256 of the module's source text (the source-change gate).
    pub source_hash: String,
    /// (2) canonical, span-free signatures of the `pub` items (+ impls), sorted.
    pub signatures: Vec<String>,
    /// (3) canonical codegen bodies of those items, sorted (the MIR-body proxy).
    pub codegen_bodies: Vec<String>,
    /// The imports' signature hashes at build time (module path -> sig hash).
    pub imports: BTreeMap<String, String>,
    /// (4a) the signature hash — over path + boundary + signatures.
    pub signature_hash: String,
    /// (4b) the codegen hash — additionally over the bodies.
    pub codegen_hash: String,
    pub provenance: Provenance,
}

/// What the build did to a module — the gate's machine-readable evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Re-analyzed (source or an import signature changed, or no cache).
    Checked,
    /// Cached analysis reused (source + every import signature unchanged).
    Reused,
}

impl Action {
    fn as_str(self) -> &'static str {
        match self {
            Action::Checked => "checked",
            Action::Reused => "reused",
        }
    }
}

/// Per-module build outcome.
#[derive(Debug, Clone)]
pub struct ModuleReport {
    pub path: String,
    pub action: Action,
    pub signature_hash: String,
    pub codegen_hash: String,
}

/// The whole-build report (modules in topological order).
#[derive(Debug, Clone)]
pub struct BuildReport {
    pub cache_dir: PathBuf,
    pub modules: Vec<ModuleReport>,
    pub diags: Vec<Diag>,
}

impl BuildReport {
    /// Paths of the modules that were re-analyzed, in topo order.
    pub fn checked(&self) -> Vec<String> {
        self.modules.iter().filter(|m| m.action == Action::Checked).map(|m| m.path.clone()).collect()
    }

    /// Paths of the modules whose cached analysis was reused, in topo order.
    pub fn reused(&self) -> Vec<String> {
        self.modules.iter().filter(|m| m.action == Action::Reused).map(|m| m.path.clone()).collect()
    }

    /// The signature hash reported for `path`, if any.
    pub fn sig_hash(&self, path: &str) -> Option<&str> {
        self.modules.iter().find(|m| m.path == path).map(|m| m.signature_hash.as_str())
    }

    /// True when the build produced no error-severity diagnostics.
    pub fn ok(&self) -> bool {
        !self.diags.iter().any(|d| d.severity == Severity::Error)
    }

    /// Machine-readable per-module action report (P20's gate evidence, P4).
    pub fn to_json(&self) -> String {
        let mods: Vec<String> = self
            .modules
            .iter()
            .map(|m| {
                format!(
                    "{{\"path\":{},\"action\":\"{}\",\"signature_hash\":{},\"codegen_hash\":{}}}",
                    json_str(&m.path),
                    m.action.as_str(),
                    json_str(&m.signature_hash),
                    json_str(&m.codegen_hash)
                )
            })
            .collect();
        let diags: Vec<String> = self.diags.iter().map(|d| d.to_json()).collect();
        format!("{{\"modules\":[{}],\"diagnostics\":[{}]}}", mods.join(","), diags.join(","))
    }
}

fn json_str(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())
}

/// `candor build <dir>` (design 0010 §3): discover the module tree, compute the
/// DAG, and per module reuse or re-analyze per the two-hash rule. Uses the real
/// schema/toolchain salt.
pub fn build_dir(dir: &Path) -> Result<BuildReport, Diag> {
    build_dir_with_salt(dir, &schema_salt())
}

/// As [`build_dir`], but with an explicit salt — the gate uses this to simulate
/// a `SCHEMA_VERSION`/toolchain bump and assert the cache invalidates (F3).
pub fn build_dir_with_salt(dir: &Path, salt: &str) -> Result<BuildReport, Diag> {
    let parts = modules::build_tree_parts(dir)?;
    let cache_dir = dir.join(".candor-cache");

    // Module-layer errors (unresolved import, cycle, private, no entry) mean the
    // DAG is not buildable; report them and emit no artifacts.
    if parts.diags.iter().any(|d| d.severity == Severity::Error) {
        return Ok(BuildReport { cache_dir, modules: Vec::new(), diags: parts.diags });
    }

    let mods = &parts.modules;
    let order = topo_order(mods);
    let cached = load_cache(&cache_dir);

    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| Diag::error("E0910", format!("cannot create cache dir: {e}"), crate::span::Span::point(0)))?;

    let mut diags = parts.diags;
    let mut reports: Vec<ModuleReport> = Vec::with_capacity(mods.len());
    // module index -> its signature hash computed THIS build (imports read it).
    let mut new_sig: Vec<Option<String>> = vec![None; mods.len()];

    for &m in &order {
        let part = &mods[m];
        let path = path_str(&part.path);
        let source_hash = sha256::hex(part.source.as_bytes());
        let signatures = signatures_of(part);
        let signature_hash = hash_signatures(&path, part.boundary, &signatures);

        // The imports' current signature hashes (available: imports precede us).
        let imports: BTreeMap<String, String> = part
            .imports
            .iter()
            .map(|&i| (path_str(&mods[i].path), new_sig[i].clone().expect("import processed before importer")))
            .collect();

        let prior = cached.get(&path);
        let reuse = prior.is_some_and(|a| {
            a.schema_salt == salt
                && a.source_hash == source_hash
                && a.signature_hash == signature_hash
                && a.imports == imports
        });

        let (action, codegen_hash) = if reuse {
            new_sig[m] = Some(signature_hash.clone());
            (Action::Reused, prior.unwrap().codegen_hash.clone())
        } else {
            // Re-analyze this module against its imports (see the module-doc
            // residual: imports enter as full-item upstream context).
            let errs = check_module(mods, m);
            let had_error = errs.iter().any(|d| d.severity == Severity::Error);
            diags.extend(errs);

            let codegen_bodies = bodies_of(part);
            let codegen_hash = hash_codegen(&signature_hash, &codegen_bodies);
            new_sig[m] = Some(signature_hash.clone());

            if !had_error {
                let artifact = Artifact {
                    module_path: path.clone(),
                    boundary: part.boundary,
                    schema_salt: salt.to_string(),
                    source_hash,
                    signatures,
                    codegen_bodies,
                    imports,
                    signature_hash: signature_hash.clone(),
                    codegen_hash: codegen_hash.clone(),
                    provenance: Provenance { toolchain: toolchain_version() },
                };
                write_artifact(&cache_dir, &path, &artifact)?;
            }
            (Action::Checked, codegen_hash)
        };

        reports.push(ModuleReport { path, action, signature_hash, codegen_hash });
    }

    // Report in topological order (deterministic, imports first).
    Ok(BuildReport { cache_dir, modules: reports, diags })
}

fn path_str(path: &[String]) -> String {
    path.join("::")
}

/// The `pub`-item (and impl) signatures of a module, canonical and sorted so the
/// signature hash is independent of item ordering (moving a `pub` fn is not a
/// signature change).
fn signatures_of(part: &ModuleParts) -> Vec<String> {
    let mut sigs: Vec<String> = part
        .items
        .iter()
        .zip(part.is_pub.iter())
        .filter(|(item, &is_pub)| is_pub || matches!(item, Item::Impl(_)))
        .map(|(item, _)| canon::item_signature(item))
        .collect();
    sigs.sort();
    sigs
}

/// The canonical codegen bodies of the same items (the MIR-body proxy).
fn bodies_of(part: &ModuleParts) -> Vec<String> {
    let mut bodies: Vec<String> = part
        .items
        .iter()
        .zip(part.is_pub.iter())
        .filter(|(item, &is_pub)| is_pub || matches!(item, Item::Impl(_)))
        .map(|(item, _)| canon::item_body(item))
        .collect();
    bodies.sort();
    bodies
}

fn hash_signatures(path: &str, boundary: bool, signatures: &[String]) -> String {
    let mut buf = String::new();
    buf.push_str(path);
    buf.push('\n');
    buf.push_str(if boundary { "boundary\n" } else { "\n" });
    for s in signatures {
        buf.push_str(s);
        buf.push('\n');
    }
    sha256::hex(buf.as_bytes())
}

fn hash_codegen(signature_hash: &str, bodies: &[String]) -> String {
    let mut buf = String::new();
    buf.push_str(signature_hash);
    buf.push('\n');
    for b in bodies {
        buf.push_str(b);
        buf.push('\n');
    }
    sha256::hex(buf.as_bytes())
}

/// Re-check one module against its imports. The checkable sub-program is the
/// module plus the full items of its transitive imports (the named upstream
/// residual — the checker takes full items, not signature-only stubs). Since the
/// imports already checked clean, the sub-program is clean iff the module is.
fn check_module(mods: &[ModuleParts], m: usize) -> Vec<Diag> {
    let mut needed: BTreeSet<usize> = BTreeSet::new();
    closure(mods, m, &mut needed);
    let mut items: Vec<Item> = Vec::new();
    for j in needed {
        items.extend(mods[j].items.iter().cloned());
    }
    crate::check::check_program_real(&Program { items })
}

fn closure(mods: &[ModuleParts], m: usize, out: &mut BTreeSet<usize>) {
    if !out.insert(m) {
        return;
    }
    for &i in &mods[m].imports {
        closure(mods, i, out);
    }
}

/// A deterministic topological order (imports before importers). The DAG is
/// acyclic by construction (design 0008 §3, checked upstream), and ties break by
/// module index (discovery/path-sorted), so the order is reproducible.
fn topo_order(mods: &[ModuleParts]) -> Vec<usize> {
    let n = mods.len();
    let mut indeg = vec![0usize; n];
    let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (m, part) in mods.iter().enumerate() {
        indeg[m] = part.imports.len();
        for &i in &part.imports {
            dependents[i].push(m);
        }
    }
    let mut order = Vec::with_capacity(n);
    let mut done = vec![false; n];
    for _ in 0..n {
        // pick the smallest-index ready node (deterministic).
        let next = (0..n).find(|&x| !done[x] && indeg[x] == 0);
        let x = match next {
            Some(x) => x,
            None => break, // cycle (should not happen post-check); bail.
        };
        done[x] = true;
        order.push(x);
        for &d in &dependents[x] {
            indeg[d] -= 1;
        }
    }
    order
}

fn cache_file(cache_dir: &Path, path: &str) -> PathBuf {
    cache_dir.join(format!("{}.json", path.replace("::", "__")))
}

fn write_artifact(cache_dir: &Path, path: &str, artifact: &Artifact) -> Result<(), Diag> {
    let json = serde_json::to_string_pretty(artifact)
        .map_err(|e| Diag::error("E0911", format!("cannot serialize artifact: {e}"), crate::span::Span::point(0)))?;
    let file = cache_file(cache_dir, path);
    std::fs::write(&file, format!("{json}\n"))
        .map_err(|e| Diag::error("E0912", format!("cannot write artifact `{}`: {e}", file.display()), crate::span::Span::point(0)))
}

fn load_cache(cache_dir: &Path) -> BTreeMap<String, Artifact> {
    let mut out = BTreeMap::new();
    let entries = match std::fs::read_dir(cache_dir) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(&p) {
            if let Ok(a) = serde_json::from_str::<Artifact>(&text) {
                out.insert(a.module_path.clone(), a);
            }
        }
    }
    out
}
