//! `candor.toml` — the package manifest (design 0017 §2/§3/§4).
//!
//! A package is a directory with a `candor.toml` at its root (0017 §1); its
//! *absence* marks today's degenerate, manifest-less bare directory, which stays
//! valid unchanged. This module owns the manifest's data model and its parse +
//! validation — it does **not** resolve dependencies or drive a build (later
//! slices).
//!
//! The schema is **closed** (0017 §2): an unknown key is an *error*, not silently
//! ignored — required for reproducibility and for a future migrator to reason
//! about every field. Errors are structured (a code + message, P4 spirit); bad
//! input never panics.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// The one language edition legal today (0017 §3). Written `"2026"` as a
/// placeholder for the naming authority; the *field* exists now to reserve the
/// edition mechanism (adding it later would be the un-migratable break it guards).
pub const CURRENT_EDITION: &str = "2026";

/// A parsed, validated `candor.toml`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Manifest {
    pub package: Package,
    /// Present ⇒ the package exposes a library API (root module `src/<name>.cnr`).
    pub lib: Option<Lib>,
    /// Buildable binary targets (`[[bin]]`), in manifest order.
    pub bins: Vec<Bin>,
    /// Dependencies, keyed by their **local** name, sorted for determinism.
    pub dependencies: Vec<Dependency>,
}

/// The `[package]` stanza (0017 §2/§3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub edition: String,
    pub freestanding: bool,
}

/// A semver triple `major.minor.patch` (0017 §3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

/// The `[lib]` stanza — a presence marker (0017 §2). It carries no fields today.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Lib {}

/// A `[[bin]]` target (0017 §2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Bin {
    pub name: String,
}

/// One `[dependencies]` entry (0017 §4/§5).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Dependency {
    /// The key in `[dependencies]` — the name this package refers to it by.
    pub local_name: String,
    /// The underlying package name when it differs from `local_name` (the alias
    /// form, 0017 §5). `None` ⇒ the underlying name equals `local_name`.
    pub package: Option<String>,
    pub source: Source,
}

/// A dependency *source* (0017 §4). Registry is deferred; the schema is
/// forward-compatible by construction — a future `{ registry = … }` source slots
/// in as a new variant without breaking existing manifests.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Source {
    /// `{ path = "../json" }` — a local directory, resolved relative to this
    /// manifest. Subsumes vendoring (0017 §4).
    Path { path: String },
    /// `{ git = "URL", rev = "<sha>" }` — `rev` pins an exact commit
    /// (reproducible by default). `tag`/`branch` may be written for convenience.
    Git {
        url: String,
        rev: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
    },
}

/// A structured manifest error: a stable machine code plus a precise message
/// (P4). Never a panic on bad input.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ManifestError {
    pub code: &'static str,
    pub message: String,
}

impl ManifestError {
    fn new(code: &'static str, message: impl Into<String>) -> ManifestError {
        ManifestError { code, message: message.into() }
    }
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for ManifestError {}

// ---------------------------------------------------------------------------
// Raw deserialization targets. These enforce the *closed* top-level/[package]
// schema via `deny_unknown_fields`; dependency *values* are read as free-form
// tables (below) so a future source kind stays forward-compatible.

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawManifest {
    package: RawPackage,
    lib: Option<RawLib>,
    #[serde(default)]
    bin: Vec<RawBin>,
    #[serde(default)]
    dependencies: BTreeMap<String, toml::Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPackage {
    name: String,
    version: String,
    edition: String,
    #[serde(default)]
    freestanding: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLib {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBin {
    name: String,
}

// ---------------------------------------------------------------------------

/// Parse and validate a `candor.toml` from its text.
pub fn parse_manifest(text: &str) -> Result<Manifest, ManifestError> {
    // Closed schema: toml + `deny_unknown_fields` names the offending key on an
    // unknown/missing field precisely; pass its message straight through.
    let raw: RawManifest = toml::from_str(text)
        .map_err(|e| ManifestError::new("M0002", e.message().to_string()))?;

    let name = validate_ident(&raw.package.name, "package name", "M0100")?;
    let version = parse_version(&raw.package.version)?;
    validate_edition(&raw.package.edition)?;

    let bins = raw
        .bin
        .iter()
        .map(|b| {
            validate_ident(&b.name, "binary name", "M0110").map(|name| Bin { name })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut dependencies = Vec::with_capacity(raw.dependencies.len());
    for (local_name, value) in &raw.dependencies {
        dependencies.push(parse_dependency(local_name, value)?);
    }

    Ok(Manifest {
        package: Package {
            name,
            version,
            edition: raw.package.edition,
            freestanding: raw.package.freestanding,
        },
        lib: raw.lib.map(|_| Lib {}),
        bins,
        dependencies,
    })
}

/// Load `candor.toml` from a directory. `Ok(None)` ⇒ the directory has no
/// manifest — today's degenerate, manifest-less package (0017 §1/§6), left
/// unchanged. `Ok(Some(_))` ⇒ a manifested package.
pub fn load_manifest(dir: &Path) -> Result<Option<Manifest>, ManifestError> {
    let path = dir.join("candor.toml");
    match std::fs::read_to_string(&path) {
        Ok(text) => parse_manifest(&text).map(Some),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(ManifestError::new(
            "M0001",
            format!("cannot read `{}`: {e}", path.display()),
        )),
    }
}

/// A globally-unique identity charset (0017 §2/§3): a lowercase ASCII letter,
/// then lowercase-alnum plus `_`/`-`.
fn validate_ident(s: &str, what: &str, code: &'static str) -> Result<String, ManifestError> {
    if s.is_empty() {
        return Err(ManifestError::new(code, format!("{what} must not be empty")));
    }
    let first = s.chars().next().unwrap();
    if !first.is_ascii_lowercase() {
        return Err(ManifestError::new(
            code,
            format!("{what} `{s}` must start with a lowercase letter"),
        ));
    }
    if let Some(bad) = s
        .chars()
        .find(|c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_' || *c == '-'))
    {
        return Err(ManifestError::new(
            code,
            format!("{what} `{s}` contains invalid character `{bad}` (allowed: lowercase a-z, 0-9, `_`, `-`)"),
        ));
    }
    Ok(s.to_string())
}

/// A semver triple `major.minor.patch`, each a non-negative integer (0017 §3).
fn parse_version(s: &str) -> Result<Version, ManifestError> {
    let bad = |detail: &str| {
        ManifestError::new(
            "M0101",
            format!("version `{s}` is not a valid `major.minor.patch` semver triple ({detail})"),
        )
    };
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return Err(bad("expected exactly three dot-separated components"));
    }
    let mut nums = [0u64; 3];
    for (slot, part) in nums.iter_mut().zip(parts.iter()) {
        *slot = part
            .parse::<u64>()
            .map_err(|_| bad(&format!("`{part}` is not a non-negative integer")))?;
    }
    Ok(Version { major: nums[0], minor: nums[1], patch: nums[2] })
}

/// Exactly one edition is legal today (0017 §3); editions are never inferred.
fn validate_edition(s: &str) -> Result<(), ManifestError> {
    if s == CURRENT_EDITION {
        Ok(())
    } else {
        Err(ManifestError::new(
            "M0102",
            format!("unknown edition `{s}`; the only supported edition is `{CURRENT_EDITION}`"),
        ))
    }
}

/// Read one `[dependencies]` value as a source-selecting inline table (0017 §4).
///
/// Forward-compatible by construction: the *kind* is chosen by which recognized
/// key is present, so a future `registry` source adds a branch here without
/// disturbing existing manifests; an unrecognized kind is a precise error rather
/// than silent acceptance.
fn parse_dependency(local_name: &str, value: &toml::Value) -> Result<Dependency, ManifestError> {
    let table = value.as_table().ok_or_else(|| {
        ManifestError::new(
            "M0200",
            format!("dependency `{local_name}` must be an inline table selecting a source (e.g. `{{ path = \"…\" }}` or `{{ git = \"…\", rev = \"…\" }}`)"),
        )
    })?;

    let package = match table.get("package") {
        Some(v) => Some(as_str(local_name, "package", v)?),
        None => None,
    };
    if let Some(alias) = &package {
        validate_ident(alias, "dependency `package` alias", "M0203")?;
    }

    let source = if table.contains_key("path") {
        reject_extra_keys(local_name, table, &["path", "package"])?;
        let path = as_str(local_name, "path", &table["path"])?;
        if path.is_empty() {
            return Err(ManifestError::new(
                "M0202",
                format!("dependency `{local_name}`: `path` must not be empty"),
            ));
        }
        Source::Path { path }
    } else if table.contains_key("git") {
        reject_extra_keys(local_name, table, &["git", "rev", "tag", "branch", "package"])?;
        let url = as_str(local_name, "git", &table["git"])?;
        if url.is_empty() {
            return Err(ManifestError::new(
                "M0202",
                format!("dependency `{local_name}`: `git` URL must not be empty"),
            ));
        }
        let rev = match table.get("rev") {
            Some(v) => as_str(local_name, "rev", v)?,
            None => {
                return Err(ManifestError::new(
                    "M0202",
                    format!("git dependency `{local_name}` requires `rev` (an exact commit sha pins the build; 0017 §4)"),
                ))
            }
        };
        if rev.is_empty() {
            return Err(ManifestError::new(
                "M0202",
                format!("dependency `{local_name}`: `rev` must not be empty"),
            ));
        }
        let tag = table.get("tag").map(|v| as_str(local_name, "tag", v)).transpose()?;
        let branch = table.get("branch").map(|v| as_str(local_name, "branch", v)).transpose()?;
        Source::Git { url, rev, tag, branch }
    } else {
        let kinds: Vec<&str> = table
            .keys()
            .map(String::as_str)
            .filter(|k| *k != "package")
            .collect();
        return Err(ManifestError::new(
            "M0201",
            format!(
                "dependency `{local_name}`: unknown source kind; recognized source keys are `path`, `git` (found: {})",
                if kinds.is_empty() { "none".to_string() } else { format!("`{}`", kinds.join("`, `")) }
            ),
        ));
    };

    Ok(Dependency { local_name: local_name.to_string(), package, source })
}

fn as_str(local_name: &str, key: &str, value: &toml::Value) -> Result<String, ManifestError> {
    value.as_str().map(str::to_string).ok_or_else(|| {
        ManifestError::new(
            "M0202",
            format!("dependency `{local_name}`: `{key}` must be a string"),
        )
    })
}

fn reject_extra_keys(
    local_name: &str,
    table: &toml::Table,
    allowed: &[&str],
) -> Result<(), ManifestError> {
    if let Some(key) = table.keys().find(|k| !allowed.contains(&k.as_str())) {
        return Err(ManifestError::new(
            "M0202",
            format!(
                "dependency `{local_name}`: unexpected key `{key}` (allowed here: `{}`)",
                allowed.join("`, `")
            ),
        ));
    }
    Ok(())
}
