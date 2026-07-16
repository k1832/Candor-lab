//! Shared test helpers: locate the built `candor-proto` oracle and make scratch
//! output directories. The tests need the real toolchain (the filter) — build it
//! first, or point `$CANDOR_PROTO` at it.

use std::path::PathBuf;

/// Locate the `candor-proto` binary: `$CANDOR_PROTO`, else the prototype's
/// release/debug target next to this workspace.
pub fn oracle_bin() -> String {
    if let Ok(p) = std::env::var("CANDOR_PROTO") {
        return p;
    }
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../compiler/target");
    for profile in ["release", "debug"] {
        let cand = base.join(profile).join("candor-proto");
        if cand.exists() {
            return cand.to_string_lossy().into_owned();
        }
    }
    panic!(
        "candor-proto not found. Build it:\n  cargo build --release --manifest-path ../../compiler/Cargo.toml\nor set $CANDOR_PROTO."
    );
}

/// A fresh, empty scratch directory unique to this test process + tag.
pub fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "corpus-gen-test-{}-{}",
        tag,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
