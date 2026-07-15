//! Stage C — the incremental, two-hash build (design 0008 §2; design 0010 §3).
//!
//! `candor build <dir>` realizes 0008's load-bearing P20 mechanism: a
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
pub mod codegen;
pub mod sha256;
pub mod stub;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ast::Item;
use crate::diag::{Diag, Severity};
use crate::modules::{self, ModuleParts, TreeParts};

/// The MIR-schema version. Bumping it invalidates every cached codegen artifact
/// by construction (design 0008 §2 F3 salt; 0010 §3). The gate exercises this
/// via [`build_dir_with_salt`].
pub const SCHEMA_VERSION: &str = "candor-mir-schema-c2";

/// The blessed-toolchain identity that co-salts the codegen cache (0010 §3): the
/// compiler version and the pinned backend. Same source + **same toolchain** is
/// the NN#16 reproducibility contract; a toolchain bump must not silently reuse
/// machine code the old compiler emitted.
pub fn toolchain_version() -> String {
    format!("candor {} / cranelift 0.132.3", env!("CARGO_PKG_VERSION"))
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

/// One exported name of a module (design 0008 §2/§3), serialized in the
/// interface artifact so a dirty importer resolves against it without re-parsing
/// this module's source (the Stage-C2 signature-only tier).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportEntry {
    pub local: String,
    pub global: String,
    pub is_pub: bool,
    pub is_fn: bool,
}

/// The per-module interface artifact (design 0008 §2).
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// The import module paths (edges of the DAG) — reconstructs this node when a
    /// dirty importer must be qualified while this module's source is untouched
    /// (or deleted): the signature-only-stub tier never re-parses upstream source.
    pub edges: Vec<String>,
    /// This module's exported type-namespace names (Stage-C2 qualification).
    pub exports_types: Vec<ExportEntry>,
    /// This module's exported value-namespace names (Stage-C2 qualification).
    pub exports_values: Vec<ExportEntry>,
    /// The already-qualified, body-stripped interface **stub items** (design 0008
    /// §2.4): the signature-only context an importer re-checks against — never the
    /// full source items. A body edit upstream does not even parse this module.
    pub stub_items: Vec<crate::ast::Item>,
    /// Per-`pub`-generic codegen hash (generic global name -> hash) — the
    /// per-instantiation cache keys on this (design 0008 §2.4; see `codegen.rs`).
    pub item_codegen_hashes: BTreeMap<String, String>,
    /// The generic instantiations reached in this module's own check (generic
    /// global name + mangled type-arg tuple) — persisted so the codegen tier sees
    /// every reached instance even when this module is reused unparsed.
    pub insts: Vec<(String, Vec<String>)>,
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
    /// The per-instantiation codegen-cache outcomes (design 0008 §2.4).
    pub codegen: Vec<codegen::InstReport>,
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

    /// Instantiations whose lowered form was reused from the codegen cache.
    pub fn codegen_reused(&self) -> Vec<String> {
        self.codegen.iter().filter(|c| c.action == Action::Reused).map(inst_label).collect()
    }

    /// Instantiations (re-)emitted this build (a codegen-cache miss).
    pub fn codegen_emitted(&self) -> Vec<String> {
        self.codegen.iter().filter(|c| c.action == Action::Checked).map(inst_label).collect()
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
        let cg: Vec<String> = self
            .codegen
            .iter()
            .map(|c| {
                format!(
                    "{{\"instance\":{},\"action\":\"{}\",\"key\":{}}}",
                    json_str(&inst_label(c)),
                    c.action.as_str(),
                    json_str(&c.key)
                )
            })
            .collect();
        let diags: Vec<String> = self.diags.iter().map(|d| d.to_json()).collect();
        format!(
            "{{\"modules\":[{}],\"codegen\":[{}],\"diagnostics\":[{}]}}",
            mods.join(","),
            cg.join(","),
            diags.join(",")
        )
    }
}

/// A stable `generic<arg,arg>` label for an instantiation report.
fn inst_label(c: &codegen::InstReport) -> String {
    format!("{}<{}>", c.generic, c.args.join(","))
}

fn json_str(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())
}

/// The resolved build target of a `candor build`: a single package (a bare
/// directory or a dependency-free manifested package) or a multi-package set
/// (design 0017 Open-Q4). The universe/entry/collision differences between the two
/// live here so the two-hash machinery downstream operates over one uniform module
/// universe.
enum Plan {
    Single(modules::DirRoot),
    Multi(crate::resolve_pkg::Resolution),
}

impl Plan {
    /// The discovered module universe — `(full path, source file, resolver index)`
    /// — the per-module import resolvers, and the entry module whose `fn main`
    /// keeps the bare global name.
    #[allow(clippy::type_complexity)]
    fn universe(
        &self,
    ) -> Result<(Vec<(Vec<String>, PathBuf, usize)>, Vec<modules::ImportResolver>, Option<Vec<String>>), Diag> {
        match self {
            Plan::Single(root) => {
                let discovered = modules::discover_module_files(root.discover_root())?
                    .into_iter()
                    .map(|(path, file)| (path, file, 0usize))
                    .collect();
                Ok((discovered, vec![modules::ImportResolver::single()], Some(vec![String::from("main")])))
            }
            Plan::Multi(res) => Ok((
                modules::discover_multi(res)?,
                modules::ImportResolver::per_package(res),
                res.entry_module(),
            )),
        }
    }

    /// Entry-presence gate (path granularity), single- or multi-package.
    fn check_entry(&self, has_path: impl Fn(&str) -> bool, diags: &mut Vec<Diag>) {
        match self {
            Plan::Single(root) => modules::check_entry_present(root, has_path, diags),
            Plan::Multi(res) => modules::check_multi_entry_present(res, has_path, diags),
        }
    }

    /// The multi-package dep-name / local-module collision gate (E0930); a no-op
    /// for a single package.
    fn check_collisions<'a>(&self, paths: impl Iterator<Item = &'a [String]>, diags: &mut Vec<Diag>) {
        if let Plan::Multi(res) = self {
            modules::check_dep_name_collisions(res, paths, diags);
        }
    }
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
    use std::collections::HashMap;
    let cache_dir = dir.join(".candor-cache");
    let cached = load_cache(&cache_dir);

    // 1. Resolve the module universe. A package with `[dependencies]` (design 0017
    //    Open-Q4) resolves its pinned package set and discovers EVERY package's
    //    modules under its pkgid — the same multi-package universe `candor
    //    check`/`run`/`compile` build via `modules::build_tree`, so the incremental
    //    build cannot diverge from the merge build. A dep-free package or bare
    //    directory keeps the historical single-package universe (bare paths,
    //    identity resolver), byte-for-byte unchanged.
    let plan = if modules::has_dependencies(dir)? {
        Plan::Multi(crate::resolve_pkg::resolve(dir)?)
    } else {
        Plan::Single(modules::resolve_dir_root(dir)?)
    };
    let (discovered, resolvers, entry) = plan.universe()?;

    // The present-source universe: read + hash every `.cnr` file (NO parse).
    struct Present {
        path: Vec<String>,
        source: String,
        source_hash: String,
        /// Index into `resolvers`: this module's package import resolver.
        resolver: usize,
    }
    let mut present: BTreeMap<String, Present> = BTreeMap::new();
    for (mp, file, ri) in discovered {
        let source = std::fs::read_to_string(&file).map_err(|e| {
            Diag::error("E0900", format!("cannot read module `{}`: {e}", file.display()), crate::span::Span::point(0))
        })?;
        let source_hash = sha256::hex(source.as_bytes());
        present.insert(path_str(&mp), Present { path: mp, source, source_hash, resolver: ri });
    }

    // 2. Module-path universe = present sources ∪ cached artifacts (a reused
    //    module's source may be absent — the brutal proof deletes it).
    let mut universe: BTreeSet<String> = present.keys().cloned().collect();
    for k in cached.keys() {
        universe.insert(k.clone());
    }

    // 3. Source-cleanliness + edges. A SOURCE-DIRTY module (changed or new source)
    //    is parsed now — it is the module whose own source changed. A SOURCE-CLEAN
    //    module (present hash == cached hash under the same salt, OR source absent
    //    but cached) is NOT parsed: its edges come from its artifact. This is the
    //    signature-only tier's teeth — clean upstream source is never parsed.
    let mut parsed: HashMap<String, modules::ParsedOne> = HashMap::new();
    let mut source_clean: HashMap<String, bool> = HashMap::new();
    let mut edges: HashMap<String, Vec<String>> = HashMap::new();
    for path in &universe {
        let pr = present.get(path);
        let art = cached.get(path);
        let clean = match (pr, art) {
            (Some(pr), Some(a)) => a.schema_salt == salt && a.source_hash == pr.source_hash,
            (None, Some(a)) => a.schema_salt == salt,
            _ => false,
        };
        source_clean.insert(path.clone(), clean);
        if clean {
            edges.insert(path.clone(), art.unwrap().edges.clone());
        } else if let Some(pr) = pr {
            let po = modules::parse_one(&pr.path, &pr.source, entry.as_deref())?;
            // A dirty module's edges are its `use`s RESOLVED through its package's
            // import resolver — cross-package aware in a multi-package build, so a
            // dependency's module is an edge exactly as an in-package one is.
            let mut es: Vec<String> = Vec::new();
            for u in &po.uses {
                match resolvers[pr.resolver].resolve_use(&u.segments) {
                    modules::UseResolution::Intra { target } => es.push(path_str(&target)),
                    modules::UseResolution::Cross { target, is_public_root, .. } => {
                        if is_public_root {
                            es.push(path_str(&target));
                        }
                    }
                }
            }
            es.sort();
            es.dedup();
            edges.insert(path.clone(), es);
            parsed.insert(path.clone(), po);
        } else {
            // Absent source, no salt-matching artifact: the module is gone.
            edges.insert(path.clone(), Vec::new());
        }
    }

    // 4. Module-layer gate: acyclicity (design 0008 §3) + the `main` entry.
    let mut diags: Vec<Diag> = Vec::new();
    if let Some(cyc) = find_path_cycle(&universe, &edges) {
        diags.push(
            Diag::error("E0904", format!("import cycle: {}", cyc.join(" -> ")), crate::span::Span::point(0))
                .with_note("the import graph must be an acyclic DAG (design 0008 §3)", None),
        );
    }
    plan.check_entry(|p| present.contains_key(p) || cached.contains_key(p), &mut diags);
    plan.check_collisions(present.values().map(|pr| pr.path.as_slice()), &mut diags);
    if diags.iter().any(|d| d.severity == Severity::Error) {
        return Ok(BuildReport { cache_dir, modules: Vec::new(), codegen: Vec::new(), diags });
    }

    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| Diag::error("E0910", format!("cannot create cache dir: {e}"), crate::span::Span::point(0)))?;

    let order = topo_paths(&universe, &edges);

    // Per-module build state, keyed by module path (topo order fills them).
    let mut new_sig: HashMap<String, String> = HashMap::new();
    let mut resolved_exports: HashMap<String, modules::ModuleExports> = HashMap::new();
    let mut stub_by_path: HashMap<String, Vec<Item>> = HashMap::new();
    let mut reports: Vec<ModuleReport> = Vec::new();
    // Codegen-tier accumulators (design 0008 §2.4).
    let mut insts_all: Vec<(String, Vec<String>)> = Vec::new();
    let mut item_hashes_all: BTreeMap<String, String> = BTreeMap::new();

    for path in &order {
        let imports = &edges[path];
        let imports_sig: BTreeMap<String, String> = imports
            .iter()
            .map(|i| (i.clone(), new_sig.get(i).cloned().unwrap_or_default()))
            .collect();
        let prior = cached.get(path);
        let reuse = *source_clean.get(path).unwrap_or(&false)
            && prior.is_some_and(|a| a.schema_salt == salt && a.imports == imports_sig);

        if reuse {
            let a = prior.unwrap();
            new_sig.insert(path.clone(), a.signature_hash.clone());
            resolved_exports.insert(path.clone(), me_from_artifact(a));
            stub_by_path.insert(path.clone(), a.stub_items.clone());
            insts_all.extend(a.insts.iter().cloned());
            for (k, v) in &a.item_codegen_hashes {
                item_hashes_all.insert(k.clone(), v.clone());
            }
            reports.push(ModuleReport {
                path: path.clone(),
                action: Action::Reused,
                signature_hash: a.signature_hash.clone(),
                codegen_hash: a.codegen_hash.clone(),
            });
            continue;
        }

        // RECHECK: qualify this module against its imports' (already-resolved)
        // exports, then re-check it against its transitive imports' signature-only
        // stubs — never re-parsing an upstream source.
        let po = match parsed.get(path) {
            Some(po) => po,
            None => {
                // Source-clean but an import's signature moved: parse it now (its
                // own body is needed to re-analyze). Its source must be present.
                let pr = present.get(path).ok_or_else(|| {
                    Diag::error("E0906", format!("module `{path}` must re-check but its source is absent"), crate::span::Span::point(0))
                })?;
                let po = modules::parse_one(&pr.path, &pr.source, entry.as_deref())?;
                parsed.entry(path.clone()).or_insert(po)
            }
        };
        let import_exports: HashMap<String, modules::ModuleExports> = imports
            .iter()
            .filter_map(|i| resolved_exports.get(i).map(|me| (i.clone(), clone_me(me))))
            .collect();
        let ri = present.get(path).map(|p| p.resolver).unwrap_or(0);
        let (qitems, qdiags) = modules::qualify_one(po, &import_exports, &resolvers[ri]);
        let source_hash = present.get(path).map(|p| p.source_hash.clone()).unwrap_or_default();
        let is_pub = po.is_pub.clone();
        let boundary = po.boundary;
        let exports = clone_me(&po.exports);

        let signatures = signatures_of_items(&qitems, &is_pub);
        let signature_hash = hash_signatures(path, boundary, &signatures);

        // Transitive stub context (imports' imports, …), from what topo built.
        let mut stubs: Vec<Item> = Vec::new();
        for imp in transitive_imports(path, &edges) {
            if let Some(items) = stub_by_path.get(&imp) {
                stubs.extend(items.iter().cloned());
            }
        }
        let (mut errs, insts) = crate::check::check_module_stub(&qitems, &stubs);
        let had_error = qdiags.iter().chain(errs.iter()).any(|d| d.severity == Severity::Error);
        diags.extend(qdiags);
        diags.append(&mut errs);

        let codegen_bodies = bodies_of_items(&qitems, &is_pub);
        let codegen_hash = hash_codegen(&signature_hash, &codegen_bodies);
        let item_codegen_hashes = item_codegen_hashes_of(&qitems, &is_pub);
        let stub_items = stub_items_of(&qitems, &is_pub);
        let mangled_insts: Vec<(String, Vec<String>)> = insts
            .iter()
            .map(|(n, args)| (n.clone(), args.iter().map(crate::generics::mangle_ty).collect()))
            .collect();

        new_sig.insert(path.clone(), signature_hash.clone());
        resolved_exports.insert(path.clone(), clone_me(&exports));
        stub_by_path.insert(path.clone(), stub_items.clone());
        insts_all.extend(mangled_insts.iter().cloned());
        for (k, v) in &item_codegen_hashes {
            item_hashes_all.insert(k.clone(), v.clone());
        }

        if !had_error {
            let (exports_types, exports_values) = export_entries(&exports);
            let artifact = Artifact {
                module_path: path.clone(),
                boundary,
                schema_salt: salt.to_string(),
                source_hash,
                signatures,
                codegen_bodies,
                imports: imports_sig,
                signature_hash: signature_hash.clone(),
                codegen_hash: codegen_hash.clone(),
                edges: imports.clone(),
                exports_types,
                exports_values,
                stub_items,
                item_codegen_hashes,
                insts: mangled_insts,
                provenance: Provenance { toolchain: toolchain_version() },
            };
            write_artifact(&cache_dir, path, &artifact)?;
        }
        reports.push(ModuleReport {
            path: path.clone(),
            action: Action::Checked,
            signature_hash,
            codegen_hash,
        });
    }

    // 5. The per-instantiation codegen cache (design 0008 §2.4).
    insts_all.sort();
    insts_all.dedup();
    let codegen = codegen::process(&cache_dir, salt, &insts_all, &item_hashes_all)?;

    Ok(BuildReport { cache_dir, modules: reports, codegen, diags })
}

fn path_str(path: &[String]) -> String {
    path.join("::")
}

/// The `pub`-item (and impl) signatures of a module, canonical and sorted so the
/// signature hash is independent of item ordering (moving a `pub` fn is not a
/// signature change).
fn signatures_of_items(items: &[Item], is_pub: &[bool]) -> Vec<String> {
    let mask = stub::crossing_mask(items, is_pub);
    let mut sigs: Vec<String> = items
        .iter()
        .zip(mask.iter())
        .filter(|(_, &c)| c)
        .map(|(item, _)| canon::item_signature(item))
        .collect();
    sigs.sort();
    sigs
}

/// The canonical codegen bodies of the interface items (the MIR-body proxy).
fn bodies_of_items(items: &[Item], is_pub: &[bool]) -> Vec<String> {
    let mask = stub::crossing_mask(items, is_pub);
    let mut bodies: Vec<String> = items
        .iter()
        .zip(mask.iter())
        .filter(|(_, &c)| c)
        .map(|(item, _)| canon::item_body(item))
        .collect();
    bodies.sort();
    bodies
}

/// The already-qualified, body-stripped interface stub items of a module.
fn stub_items_of(items: &[Item], is_pub: &[bool]) -> Vec<Item> {
    let mask = stub::crossing_mask(items, is_pub);
    items
        .iter()
        .zip(mask.iter())
        .filter(|(_, &c)| c)
        .map(|(item, _)| stub::stub_item(item))
        .collect()
}

/// Per-`pub`-generic codegen hash (generic global name -> hash of its canonical
/// signature+body). The per-instantiation codegen cache keys on this so a body
/// edit invalidates exactly its own instantiations (design 0008 §2.4).
fn item_codegen_hashes_of(items: &[Item], is_pub: &[bool]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for (item, &p) in items.iter().zip(is_pub.iter()) {
        if !p {
            continue;
        }
        let name = match item {
            Item::Fn(f) if !f.type_params.is_empty() => f.name.clone(),
            Item::Struct(s) if !s.type_params.is_empty() => s.name.clone(),
            Item::Enum(e) if !e.type_params.is_empty() => e.name.clone(),
            _ => continue,
        };
        let mut buf = canon::item_signature(item);
        buf.push('\n');
        buf.push_str(&canon::item_body(item));
        out.insert(name, sha256::hex(buf.as_bytes()));
    }
    out
}

/// Clone a [`modules::ModuleExports`] (it holds no `Clone` derive of its own).
fn clone_me(me: &modules::ModuleExports) -> modules::ModuleExports {
    modules::ModuleExports {
        path: me.path.clone(),
        type_exports: me.type_exports.clone(),
        value_exports: me.value_exports.clone(),
    }
}

/// Reconstruct a module's export tables from its cached artifact (Stage-C2): a
/// dirty importer qualifies against these without re-parsing this module.
fn me_from_artifact(a: &Artifact) -> modules::ModuleExports {
    let path: Vec<String> = a.module_path.split("::").map(|s| s.to_string()).collect();
    let mut type_exports = std::collections::HashMap::new();
    let mut value_exports = std::collections::HashMap::new();
    for e in &a.exports_types {
        type_exports.insert(e.local.clone(), modules::Export { global: e.global.clone(), is_pub: e.is_pub, is_fn: e.is_fn });
    }
    for e in &a.exports_values {
        value_exports.insert(e.local.clone(), modules::Export { global: e.global.clone(), is_pub: e.is_pub, is_fn: e.is_fn });
    }
    modules::ModuleExports { path, type_exports, value_exports }
}

/// Serialize a module's export tables into artifact [`ExportEntry`] lists.
fn export_entries(me: &modules::ModuleExports) -> (Vec<ExportEntry>, Vec<ExportEntry>) {
    let mut ty: Vec<ExportEntry> = me
        .type_exports
        .iter()
        .map(|(local, e)| ExportEntry { local: local.clone(), global: e.global.clone(), is_pub: e.is_pub, is_fn: e.is_fn })
        .collect();
    let mut va: Vec<ExportEntry> = me
        .value_exports
        .iter()
        .map(|(local, e)| ExportEntry { local: local.clone(), global: e.global.clone(), is_pub: e.is_pub, is_fn: e.is_fn })
        .collect();
    ty.sort_by(|a, b| a.local.cmp(&b.local));
    va.sort_by(|a, b| a.local.cmp(&b.local));
    (ty, va)
}

/// The transitive import closure of `path` over the edge map (paths, sorted).
fn transitive_imports(path: &str, edges: &std::collections::HashMap<String, Vec<String>>) -> Vec<String> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut stack: Vec<String> = edges.get(path).cloned().unwrap_or_default();
    while let Some(p) = stack.pop() {
        if seen.insert(p.clone()) {
            if let Some(es) = edges.get(&p) {
                stack.extend(es.iter().cloned());
            }
        }
    }
    seen.into_iter().collect()
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

/// The transitive-import stub items for module `m` (every import, its imports,
/// …), each projected to its interface stub ([`stub::stub_item`]) — the
/// signature-only context the checker re-checks `m` against (design 0008 §2).
fn transitive_stubs(mods: &[ModuleParts], m: usize) -> Vec<Item> {
    let mut needed: BTreeSet<usize> = BTreeSet::new();
    for &i in &mods[m].imports {
        closure(mods, i, &mut needed);
    }
    needed.remove(&m);
    let mut stubs = Vec::new();
    for j in needed {
        let mask = stub::crossing_mask(&mods[j].items, &mods[j].is_pub);
        for (item, &c) in mods[j].items.iter().zip(mask.iter()) {
            // Only the interface surface crosses (design 0008 §2).
            if c {
                stubs.push(stub::stub_item(item));
            }
        }
    }
    stubs
}

/// Check every module of a tree against its imports' **signature-only stubs**
/// (design 0008 §2, §2.4), returning the union of diagnostics. This is the
/// signature-only re-check path made whole-tree; the Stage-C2 differential gate
/// asserts it is diagnostic-equivalent to the whole-program merge check, on
/// positive *and* negative fixtures.
pub fn check_tree_stubwise(parts: &TreeParts) -> Vec<Diag> {
    let mods = &parts.modules;
    let mut out: Vec<Diag> = parts.diags.clone();
    for m in 0..mods.len() {
        let stubs = transitive_stubs(mods, m);
        let (diags, _insts) = crate::check::check_module_stub(&mods[m].items, &stubs);
        out.extend(diags);
    }
    out
}

fn closure(mods: &[ModuleParts], m: usize, out: &mut BTreeSet<usize>) {
    if !out.insert(m) {
        return;
    }
    for &i in &mods[m].imports {
        closure(mods, i, out);
    }
}

/// A deterministic topological order over module paths (imports before
/// importers), ties broken by sorted path — reproducible across builds (NN#16).
fn topo_paths(universe: &BTreeSet<String>, edges: &std::collections::HashMap<String, Vec<String>>) -> Vec<String> {
    let nodes: Vec<String> = universe.iter().cloned().collect();
    let mut done: BTreeSet<String> = BTreeSet::new();
    let mut order: Vec<String> = Vec::with_capacity(nodes.len());
    for _ in 0..nodes.len() {
        // The smallest-path node all of whose imports are already placed.
        let next = nodes.iter().find(|n| {
            !done.contains(*n)
                && edges.get(*n).map(|es| es.iter().all(|e| done.contains(e) || !universe.contains(e))).unwrap_or(true)
        });
        match next {
            Some(n) => {
                done.insert(n.clone());
                order.push(n.clone());
            }
            None => break, // a cycle (already diagnosed); bail deterministically.
        }
    }
    order
}

/// Detect an import cycle over the path-keyed edge map, returning the cycle path
/// for the P4 diagnostic (design 0008 §3).
fn find_path_cycle(universe: &BTreeSet<String>, edges: &std::collections::HashMap<String, Vec<String>>) -> Option<Vec<String>> {
    let mut state: std::collections::HashMap<String, u8> = std::collections::HashMap::new();
    let mut stack: Vec<String> = Vec::new();
    fn dfs(
        u: &str,
        edges: &std::collections::HashMap<String, Vec<String>>,
        universe: &BTreeSet<String>,
        state: &mut std::collections::HashMap<String, u8>,
        stack: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        state.insert(u.to_string(), 1);
        stack.push(u.to_string());
        for v in edges.get(u).map(|v| v.as_slice()).unwrap_or(&[]) {
            if !universe.contains(v) {
                continue;
            }
            match state.get(v).copied().unwrap_or(0) {
                1 => {
                    let pos = stack.iter().position(|x| x == v).unwrap();
                    let mut cyc = stack[pos..].to_vec();
                    cyc.push(v.clone());
                    return Some(cyc);
                }
                0 => {
                    if let Some(c) = dfs(v, edges, universe, state, stack) {
                        return Some(c);
                    }
                }
                _ => {}
            }
        }
        stack.pop();
        state.insert(u.to_string(), 2);
        None
    }
    for n in universe {
        if state.get(n).copied().unwrap_or(0) == 0 {
            if let Some(c) = dfs(n, edges, universe, &mut state, &mut stack) {
                return Some(c);
            }
        }
    }
    None
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
