//! Stage C gate (design 0010 §3; design 0008 §2) — the two-hash incremental
//! build. Asserts, as the P20 evidence:
//!   (a) rebuild reuses every module (zero re-analysis) when nothing changed;
//!   (b) a body edit re-checks ONLY that module, every downstream reused, and the
//!       edited module's signature hash is STABLE (analysis-invalidation tier);
//!   (c) a `pub`-signature edit re-checks exactly the module + its direct
//!       importers, and stops where signatures stop changing (the §2 cascade);
//!   (d) two clean rebuilds produce BIT-IDENTICAL artifacts (NN#16);
//!   (e) a schema/toolchain salt bump invalidates the whole cache (F3).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use candor::build::{self, BuildReport};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR")))
}

/// Copy a fixture tree into a fresh temp dir (no `.candor-cache`), returning the
/// copy root. Each call is unique so tests do not collide.
fn temp_copy(name: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dst = std::env::temp_dir().join(format!("candor_stage_c_{}_{}", std::process::id(), n));
    let _ = std::fs::remove_dir_all(&dst);
    copy_tree(&fixture(name), &dst);
    dst
}

fn copy_tree(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let p = entry.path();
        let target = dst.join(entry.file_name());
        if p.is_dir() {
            // Never copy a stray cache dir into the pristine tree.
            if p.file_name().and_then(|s| s.to_str()) == Some(".candor-cache") {
                continue;
            }
            copy_tree(&p, &target);
        } else {
            std::fs::copy(&p, &target).unwrap();
        }
    }
}

fn build(dir: &Path) -> BuildReport {
    let report = build::build_dir(dir).expect("build should not hard-error");
    assert!(report.ok(), "build had errors: {:?}", report.diags.iter().map(|d| &d.code).collect::<Vec<_>>());
    report
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

fn write(dir: &Path, rel: &str, contents: &str) {
    std::fs::write(dir.join(rel), contents).unwrap();
}

// ---- (a) rebuild reuses every module (zero re-analysis) --------------------

#[test]
fn gate_a_rebuild_reuses_every_module() {
    let dir = temp_copy("corelib");
    let first = build(&dir);
    assert!(first.reused().is_empty(), "first build should reuse nothing");
    assert_eq!(first.checked().len(), 9, "first build should check all 9 modules");

    let second = build(&dir);
    assert!(second.checked().is_empty(), "second build re-analyzed {:?}", second.checked());
    assert_eq!(second.reused().len(), 9, "second build should reuse all 9 modules");
}

// ---- (b) a body edit re-checks ONLY that module ----------------------------

#[test]
fn gate_b_body_edit_reanalyzes_only_edited_module() {
    let dir = temp_copy("build_c");
    let first = build(&dir);
    let base_sig_before = first.sig_hash("base").unwrap().to_string();

    // Edit the BODY of `base::f` (a real value change; still type-valid). No
    // `pub` signature changes.
    write(&dir, "base.cnr", "pub fn f(x: i64) -> i64 { return x + 100; }\n\npub fn g(x: i64) -> i64 { return x + 2; }\n");

    let second = build(&dir);
    assert_eq!(second.checked(), vec!["base".to_string()], "only `base` should re-check");
    assert_eq!(sorted(second.reused()), vec!["a".to_string(), "b".to_string(), "main".to_string()],
        "every downstream module must be reused");

    // The analysis-invalidation tier's core assertion: the signature hash is
    // STABLE across a body edit, which is *why* nothing downstream re-analyzes.
    let base_sig_after = second.sig_hash("base").unwrap();
    assert_eq!(base_sig_after, base_sig_before, "a body edit must not change the signature hash");
}

// ---- (c) a pub-signature edit cascades exactly per 0008 §2 -----------------

#[test]
fn gate_c_signature_edit_invalidates_precise_set() {
    let dir = temp_copy("build_c");
    let first = build(&dir);
    let base_sig_before = first.sig_hash("base").unwrap().to_string();
    let a_sig_before = first.sig_hash("a").unwrap().to_string();
    let b_sig_before = first.sig_hash("b").unwrap().to_string();

    // Edit the `pub` SIGNATURE of `base::g` (add a parameter). `a`/`b` call only
    // `base::f`, so they re-check but stay valid and their OWN signatures do not
    // change — the cascade stops before `main`.
    write(&dir, "base.cnr", "pub fn f(x: i64) -> i64 { return x + 1; }\n\npub fn g(x: i64, y: i64) -> i64 { return x + y; }\n");

    let second = build(&dir);
    assert_eq!(sorted(second.checked()), vec!["a".to_string(), "b".to_string(), "base".to_string()],
        "the edited module and its direct importers must re-check");
    assert_eq!(second.reused(), vec!["main".to_string()],
        "the cascade must stop at `main` (its imports' signatures did not change)");

    // The hash evidence for the cascade: base's signature changed; a/b's did not.
    assert_ne!(second.sig_hash("base").unwrap(), base_sig_before, "base's signature must change");
    assert_eq!(second.sig_hash("a").unwrap(), a_sig_before, "a's signature must be unchanged");
    assert_eq!(second.sig_hash("b").unwrap(), b_sig_before, "b's signature must be unchanged");
}

// ---- (d) bit-identical artifacts across a clean rebuild --------------------

#[test]
fn gate_d_clean_rebuild_is_bit_identical() {
    let dir = temp_copy("corelib");

    let a = build(&dir);
    let bytes_a = read_cache(&a.cache_dir);
    std::fs::remove_dir_all(&a.cache_dir).unwrap();

    let b = build(&dir);
    let bytes_b = read_cache(&b.cache_dir);

    assert_eq!(bytes_a.len(), 9, "expected one artifact per module");
    assert_eq!(bytes_a, bytes_b, "two clean rebuilds must produce byte-identical artifacts");
}

/// Read every artifact file (sorted by name) as raw bytes.
fn read_cache(cache_dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out: Vec<(String, Vec<u8>)> = std::fs::read_dir(cache_dir)
        .unwrap()
        .filter_map(|e| {
            let p = e.unwrap().path();
            if p.extension().and_then(|s| s.to_str()) == Some("json") {
                let name = p.file_name().unwrap().to_str().unwrap().to_string();
                Some((name, std::fs::read(&p).unwrap()))
            } else {
                None
            }
        })
        .collect();
    out.sort();
    out
}

// ---- (e) the schema/toolchain salt invalidates the cache (F3) --------------

#[test]
fn gate_e_schema_salt_bump_invalidates_cache() {
    let dir = temp_copy("build_c");
    let salt1 = build::schema_salt();
    let salt2 = format!("{salt1}|BUMPED");

    // Fresh cache under salt1: everything checks.
    let first = build::build_dir_with_salt(&dir, &salt1).unwrap();
    assert!(first.ok());
    assert_eq!(first.checked().len(), 4, "first build checks all modules");

    // Same salt: everything reuses.
    let same = build::build_dir_with_salt(&dir, &salt1).unwrap();
    assert!(same.checked().is_empty(), "same salt must reuse everything");

    // Bumped salt: the whole cache is invalid — every module re-checks, by
    // construction (a toolchain upgrade cannot silently reuse the old output).
    let bumped = build::build_dir_with_salt(&dir, &salt2).unwrap();
    assert_eq!(bumped.checked().len(), 4, "a salt bump must invalidate every artifact");
    assert!(bumped.reused().is_empty(), "no artifact may be reused across a salt bump");
}
