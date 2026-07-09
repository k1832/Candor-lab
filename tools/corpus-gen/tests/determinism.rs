//! Determinism contract: same seed + same toolchain => byte-identical corpus.

mod common;

use corpus_gen::oracle::Oracle;
use corpus_gen::{generate, Config, DEFAULT_TOOLCHAIN_VERSION};
use std::collections::BTreeMap;
use std::path::Path;

fn cfg(seed: u64, out: std::path::PathBuf) -> Config {
    Config {
        seed,
        positive: 8,
        negative: 4,
        out_dir: out,
        oracle: Oracle::new(common::oracle_bin()),
        toolchain_version: DEFAULT_TOOLCHAIN_VERSION.to_string(),
        max_attempts: 4000,
    }
}

/// Read every file under `root` into a path -> bytes map (paths relative to root).
fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut map = BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let rel = p.strip_prefix(root).unwrap().to_string_lossy().into_owned();
                map.insert(rel, std::fs::read(&p).unwrap());
            }
        }
    }
    map
}

#[test]
fn same_seed_is_byte_identical() {
    let base = common::scratch("determinism");
    let a = base.join("a");
    let b = base.join("b");
    generate(&cfg(7, a.clone())).unwrap();
    generate(&cfg(7, b.clone())).unwrap();

    let sa = snapshot(&a);
    let sb = snapshot(&b);
    assert_eq!(sa.keys().collect::<Vec<_>>(), sb.keys().collect::<Vec<_>>(), "file set differs");
    for (k, va) in &sa {
        assert_eq!(va, &sb[k], "byte mismatch in {k}");
    }
    // Sanity: the run actually produced the expected sample set.
    assert!(sa.contains_key("manifest.json"));
    assert_eq!(sa.keys().filter(|k| k.ends_with(".cnr")).count(), 12);
}

#[test]
fn different_seed_differs() {
    let base = common::scratch("determinism-diff");
    let a = base.join("s7");
    let b = base.join("s99");
    generate(&cfg(7, a.clone())).unwrap();
    generate(&cfg(99, b.clone())).unwrap();
    // The manifests must differ (params drawn from the seed).
    let ma = std::fs::read(a.join("manifest.json")).unwrap();
    let mb = std::fs::read(b.join("manifest.json")).unwrap();
    assert_ne!(ma, mb, "distinct seeds produced identical manifests");
}
