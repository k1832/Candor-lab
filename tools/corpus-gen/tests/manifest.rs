//! Manifest validity: it round-trips through serde, its counts agree with the
//! files on disk, and every sample carries a complete record.

mod common;

use corpus_gen::manifest::Manifest;
use corpus_gen::oracle::Oracle;
use corpus_gen::{generate, Config, DEFAULT_TOOLCHAIN_VERSION};

#[test]
fn manifest_is_valid_and_agrees_with_disk() {
    let dir = common::scratch("manifest");
    let cfg = Config {
        seed: 5,
        positive: 8,
        negative: 6,
        out_dir: dir.clone(),
        oracle: Oracle::new(common::oracle_bin()),
        toolchain_version: DEFAULT_TOOLCHAIN_VERSION.to_string(),
        max_attempts: 4000,
    };
    let produced = generate(&cfg).unwrap();

    // Re-load from disk and confirm it deserializes.
    let raw = std::fs::read_to_string(dir.join("manifest.json")).unwrap();
    let m: Manifest = serde_json::from_str(&raw).unwrap();

    assert_eq!(m.seed, 5);
    assert_eq!(m.positive_count, 8);
    assert_eq!(m.negative_count, 6);
    assert!(!m.toolchain_version.is_empty());
    assert_eq!(m.samples.len(), 14);
    assert_eq!(m.samples.len(), produced.samples.len());

    // Every listed file exists; every listed .cnr on disk is listed exactly once.
    let mut listed = std::collections::BTreeSet::new();
    for s in &m.samples {
        assert!(dir.join(&s.file).exists(), "missing {}", s.file);
        assert!(!s.toolchain_version.is_empty());
        assert!(listed.insert(s.file.clone()), "duplicate file {}", s.file);
    }
    let on_disk = std::fs::read_dir(dir.join("positive")).unwrap().count()
        + std::fs::read_dir(dir.join("negative")).unwrap().count();
    assert_eq!(on_disk, m.samples.len(), "orphan files on disk");

    // Stats: kept totals equal the requested counts.
    let pos_kept: usize = m.stats.iter().filter(|s| s.category == "positive").map(|s| s.kept).sum();
    let neg_kept: usize = m.stats.iter().filter(|s| s.category == "negative").map(|s| s.kept).sum();
    assert_eq!(pos_kept, 8);
    assert_eq!(neg_kept, 6);
}
