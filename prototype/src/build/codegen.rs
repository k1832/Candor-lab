//! The per-instantiation content-addressed codegen cache (design 0008 §2.4;
//! design 0010 §3) — Stage C2's second residual made real.
//!
//! A monomorphized instance is keyed by **(the defining module's per-generic
//! codegen hash, the type-argument tuple, the schema/toolchain salt)** and stored
//! under `.candor-cache/codegen/`. The gate this delivers:
//!
//! * a rebuild after a **non-generic** body edit **reuses every** cached
//!   instantiation of an untouched generic — a non-generic edit perturbs neither
//!   any generic's per-item codegen hash nor a type-argument tuple, so no key
//!   moves; and
//! * a **generic body** edit invalidates **exactly its own** instantiations — the
//!   edited generic's per-item codegen hash moves (and *only* it, even for a
//!   sibling generic in the same module), so only that generic's keys are re-
//!   emitted; instantiation **never cascades analysis** (0008's promise).
//!
//! ## The key granularity — an under-specification resolved (reported).
//! Designs 0008/0010 name the key as "codegen hash of the **defining module**".
//! A module-granular hash would over-invalidate: editing one generic's body would
//! re-emit *every* instantiation of *every* generic in that module, contradicting
//! "invalidates exactly its own instantiations". We therefore record, in the
//! module's interface artifact, a **per-`pub`-generic codegen hash** (a component
//! *of* the module's codegen artifact), and key each instantiation on the hash of
//! **its** generic. This is finer than, and subsumed by, the module-level hash;
//! it is the reading that makes both design clauses simultaneously true.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::diag::Diag;

use super::{sha256, Action};

/// One reached monomorphized instantiation and what the codegen tier did with it.
#[derive(Debug, Clone)]
pub struct InstReport {
    /// The generic's global (module-qualified) name.
    pub generic: String,
    /// The mangled type-argument tuple.
    pub args: Vec<String>,
    /// The content-addressed cache key (hex).
    pub key: String,
    /// Reused (key already cached) or emitted (a miss — lowered and stored).
    pub action: Action,
}

/// The `.candor-cache/codegen/` area.
pub fn codegen_dir(cache_dir: &Path) -> PathBuf {
    cache_dir.join("codegen")
}

/// The content-addressed key for an instantiation (design 0008 §2.4 / 0010 §3):
/// `sha256(salt | generic | per-generic-codegen-hash | type-arg tuple)`.
pub fn inst_key(salt: &str, generic: &str, item_codegen_hash: &str, args: &[String]) -> String {
    let mut buf = String::new();
    buf.push_str(salt);
    buf.push('\n');
    buf.push_str(generic);
    buf.push('\n');
    buf.push_str(item_codegen_hash);
    buf.push('\n');
    buf.push_str(&args.join(","));
    sha256::hex(buf.as_bytes())
}

/// Process every reached instantiation against the on-disk cache: reuse the
/// entry when its key is already present, else lower it (here: a reusable-form
/// proxy string standing in for the monomorphized MIR — the same MIR-body proxy
/// Stage C1 uses) and store it. Returns one [`InstReport`] per instantiation,
/// deterministically ordered.
///
/// `insts` is the deduplicated `(generic, args)` set reached across the build;
/// `item_hashes` maps each generic's global name to its per-generic codegen hash
/// (from the defining module's interface artifact).
pub fn process(
    cache_dir: &Path,
    salt: &str,
    insts: &[(String, Vec<String>)],
    item_hashes: &BTreeMap<String, String>,
) -> Result<Vec<InstReport>, Diag> {
    let dir = codegen_dir(cache_dir);
    std::fs::create_dir_all(&dir).map_err(|e| {
        Diag::error("E0913", format!("cannot create codegen cache dir: {e}"), crate::span::Span::point(0))
    })?;

    let mut reports = Vec::with_capacity(insts.len());
    for (generic, args) in insts {
        // A generic whose defining module is absent from the build (should not
        // happen for a resolved DAG) has no per-item hash; skip it rather than
        // mis-key it.
        let item_hash = match item_hashes.get(generic) {
            Some(h) => h,
            None => continue,
        };
        let key = inst_key(salt, generic, item_hash, args);
        let file = dir.join(format!("{key}.mir"));
        let action = if file.exists() {
            Action::Reused
        } else {
            // The reusable lowered form (proxy): identity + args + the generic's
            // codegen hash. Content-addressed by `key`, so a body edit that moves
            // the hash writes a fresh entry and leaves untouched generics' cached.
            let body = format!("{generic}<{}>@{item_hash}\n", args.join(","));
            std::fs::write(&file, body).map_err(|e| {
                Diag::error("E0914", format!("cannot write codegen entry `{}`: {e}", file.display()), crate::span::Span::point(0))
            })?;
            Action::Checked
        };
        reports.push(InstReport { generic: generic.clone(), args: args.clone(), key, action });
    }
    reports.sort_by(|a, b| (a.generic.as_str(), &a.args).cmp(&(b.generic.as_str(), &b.args)));
    reports
        .dedup_by(|a, b| a.generic == b.generic && a.args == b.args);
    Ok(reports)
}
