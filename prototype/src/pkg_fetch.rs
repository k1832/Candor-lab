//! Git-dependency fetch into a content-addressed cache (design 0017 §4/§6).
//!
//! A `{ git = URL, rev = "<sha>" }` dependency is fetched — via the system
//! `git` — into a toolchain-managed cache, **never** committed into the
//! depending repository (reproducibility comes from the lock's pinned sha, not
//! from checked-in copies). The cache is keyed by URL + resolved commit sha, so
//! a given (url, sha) is fetched **once**; a later resolve of the same pin reuses
//! the cached checkout with no re-clone.
//!
//! Cache layout under the root (default per-user cache dir, overridable by
//! `CANDOR_CACHE_DIR` so tests use an isolated temp cache):
//!
//! ```text
//! <root>/git-db/<url-hash>/            bare mirror of the url (for resolving refs)
//! <root>/git-src/<url-hash>-<sha>/     pristine checkout at the exact commit
//! ```
//!
//! The checkout is plain source with **no `.git`** — read-only dependency source
//! that the resolver then treats exactly like a path dependency. A `tag`/`branch`
//! written in the manifest is resolved to its commit sha here; the caller records
//! that resolved sha in `candor.lock` so the build pins to the commit, not the
//! moving ref.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::build::sha256;
use crate::diag::Diag;
use crate::span::Span;

/// A fetched git dependency: its pristine checkout directory and the resolved
/// full commit sha (what the lockfile pins to).
pub struct GitCheckout {
    pub dir: PathBuf,
    pub resolved_rev: String,
}

fn diag(code: &str, message: impl Into<String>) -> Diag {
    Diag::error(code, message, Span::point(0))
}

/// The cache root: `CANDOR_CACHE_DIR` if set (tests point this at a temp dir),
/// else a per-user cache directory. Never the depending repository.
fn cache_root() -> PathBuf {
    if let Some(dir) = std::env::var_os("CANDOR_CACHE_DIR") {
        return PathBuf::from(dir);
    }
    if let Some(xdg) = std::env::var_os("XDG_CACHE_HOME") {
        return PathBuf::from(xdg).join("candor");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".cache").join("candor");
    }
    std::env::temp_dir().join("candor-cache")
}

/// A git commit sha in its canonical form: 40 (sha1) or 64 (sha256) lowercase hex.
fn is_full_sha(s: &str) -> bool {
    (s.len() == 40 || s.len() == 64) && s.chars().all(|c| c.is_ascii_hexdigit())
}

fn url_hash(url: &str) -> String {
    sha256::hex(url.as_bytes())[..16].to_string()
}

/// A checkout is usable iff it exists and carries a package manifest — the atomic
/// rename below means a half-written checkout is never observed under this name.
fn is_valid_checkout(dir: &Path) -> bool {
    dir.join("candor.toml").is_file()
}

/// Fetch `{ git = url, rev, tag?, branch? }` into the content-addressed cache and
/// return its checkout (design 0017 §4). A `tag`/`branch` is resolved to a commit
/// sha; a bare `rev` naming a full sha whose checkout is already cached returns
/// with no git invocation at all (the reuse fast path).
pub fn fetch_git(
    url: &str,
    rev: &str,
    tag: Option<&str>,
    branch: Option<&str>,
) -> Result<GitCheckout, Diag> {
    let root = cache_root();
    let uh = url_hash(url);

    // The ref that determines the commit: a moving ref wins over the bare rev
    // (the manifest wrote it for convenience), else the rev names the commit.
    let moving_ref = branch.or(tag);

    // Reuse fast path: a bare, full-sha rev whose checkout is cached needs no git.
    if moving_ref.is_none() && is_full_sha(rev) {
        let sha = rev.to_ascii_lowercase();
        let checkout = root.join("git-src").join(format!("{uh}-{sha}"));
        if is_valid_checkout(&checkout) {
            return Ok(GitCheckout { dir: checkout, resolved_rev: sha });
        }
    }

    let db = root.join("git-db").join(&uh);
    ensure_mirror(&db, url)?;

    let want = moving_ref.unwrap_or(rev);
    let sha = resolve_commit(&db, url, want)?;

    let checkout = root.join("git-src").join(format!("{uh}-{sha}"));
    if !is_valid_checkout(&checkout) {
        create_checkout(&db, &sha, &checkout)?;
    }
    Ok(GitCheckout { dir: checkout, resolved_rev: sha })
}

/// Ensure a bare mirror of `url` exists under `db`. Cloned once (keyed by url);
/// a rev-pinned build does not re-fetch a present mirror.
fn ensure_mirror(db: &Path, url: &str) -> Result<(), Diag> {
    if db.join("HEAD").is_file() {
        return Ok(());
    }
    if let Some(parent) = db.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| diag("E0934", format!("cannot create git cache dir `{}`: {e}", parent.display())))?;
    }
    // Clone into a unique temp dir, then atomically publish, so a concurrent or
    // interrupted clone never leaves a partial mirror under `db`.
    let tmp = sibling_tmp(db);
    let out = run_git(&["clone".as_ref(), "--bare".as_ref(), "--quiet".as_ref(), url.as_ref(), tmp.as_os_str()])?;
    if !out.status.success() {
        let _ = std::fs::remove_dir_all(&tmp);
        return Err(diag(
            "E0932",
            format!("git clone of `{url}` failed: {}", stderr(&out)),
        ));
    }
    match std::fs::rename(&tmp, db) {
        Ok(()) => Ok(()),
        // A concurrent fetch won the race and published first: reuse theirs.
        Err(_) if db.join("HEAD").is_file() => {
            let _ = std::fs::remove_dir_all(&tmp);
            Ok(())
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&tmp);
            Err(diag("E0934", format!("cannot publish git mirror `{}`: {e}", db.display())))
        }
    }
}

/// Resolve a rev/tag/branch to its full commit sha in the mirror (design 0017 §4:
/// the lockfile records the resolved commit, so the build pins even a moving ref).
fn resolve_commit(db: &Path, url: &str, want: &str) -> Result<String, Diag> {
    let spec = format!("{want}^{{commit}}");
    let out = run_git(&[
        "--git-dir".as_ref(),
        db.as_os_str(),
        "rev-parse".as_ref(),
        "--verify".as_ref(),
        "--quiet".as_ref(),
        spec.as_ref(),
    ])?;
    if !out.status.success() {
        return Err(diag(
            "E0933",
            format!("git dependency `{url}`: cannot resolve `{want}` to a commit (unknown rev/tag/branch)"),
        ));
    }
    let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if !is_full_sha(&sha) {
        return Err(diag(
            "E0933",
            format!("git dependency `{url}`: `{want}` resolved to an unexpected commit id `{sha}`"),
        ));
    }
    Ok(sha)
}

/// Materialize a pristine, read-only checkout of `sha` from the mirror. Built in a
/// temp dir (clone + detached checkout, then `.git` removed so it is plain source
/// and no longer references the mirror) and atomically renamed into place.
fn create_checkout(db: &Path, sha: &str, checkout: &Path) -> Result<(), Diag> {
    if let Some(parent) = checkout.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| diag("E0934", format!("cannot create git cache dir `{}`: {e}", parent.display())))?;
    }
    let tmp = sibling_tmp(checkout);
    let clone = run_git(&["clone".as_ref(), "--quiet".as_ref(), "--no-checkout".as_ref(), db.as_os_str(), tmp.as_os_str()])?;
    if !clone.status.success() {
        let _ = std::fs::remove_dir_all(&tmp);
        return Err(diag("E0932", format!("git clone from cache failed: {}", stderr(&clone))));
    }
    let co = run_git(&["-C".as_ref(), tmp.as_os_str(), "checkout".as_ref(), "--quiet".as_ref(), "--detach".as_ref(), sha.as_ref()])?;
    if !co.status.success() {
        let _ = std::fs::remove_dir_all(&tmp);
        return Err(diag("E0933", format!("git checkout of commit `{sha}` failed: {}", stderr(&co))));
    }
    // Sever the checkout from the object db so it is plain source that survives
    // even if the mirror is later evicted; content-hash + build see only `src/`.
    let _ = std::fs::remove_dir_all(tmp.join(".git"));
    match std::fs::rename(&tmp, checkout) {
        Ok(()) => Ok(()),
        // A concurrent fetch of the same (url, sha) published first: reuse theirs.
        Err(_) if is_valid_checkout(checkout) => {
            let _ = std::fs::remove_dir_all(&tmp);
            Ok(())
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&tmp);
            Err(diag("E0934", format!("cannot publish git checkout `{}`: {e}", checkout.display())))
        }
    }
}

static TMP_NONCE: AtomicU64 = AtomicU64::new(0);

/// A unique sibling temp path next to `target` (same filesystem, so the publish
/// rename is atomic).
fn sibling_tmp(target: &Path) -> PathBuf {
    let n = TMP_NONCE.fetch_add(1, Ordering::SeqCst);
    let name = target.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let parent = target.parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."));
    parent.join(format!(".{name}.tmp-{}-{n}", std::process::id()))
}

fn run_git(args: &[&std::ffi::OsStr]) -> Result<std::process::Output, Diag> {
    Command::new("git").args(args).output().map_err(|e| {
        diag(
            "E0931",
            format!("cannot run `git` to fetch a git dependency (is git installed and on PATH?): {e}"),
        )
    })
}

fn stderr(out: &std::process::Output) -> String {
    let s = String::from_utf8_lossy(&out.stderr);
    let s = s.trim();
    if s.is_empty() {
        "git reported no error output".to_string()
    } else {
        s.to_string()
    }
}
