//! The P20 measurement harness (design 0010 §3, §5 Stage D).
//!
//! This is the measurement **INSTRUMENT** for P20's pre-registered targets — not
//! a CI gate yet, and its prototype numbers are **baselines, not claims**: T1/T3–T5's
//! real numbers await the real toolchain at the reference scale (N = 200 modules,
//! M ≈ 50 000 lines; design 0010 §3). Here the reference project is the corelib
//! fixture tree (8 modules, ~520 lines). The harness measures the *shapes* of:
//!
//!  * **T1 — incremental rebuild** (one body edit, no `pub`-signature change, warm
//!    cache): wall time to rebuild after touching a single module's body.
//!  * **T2 — incremental analysis scope** (instrumented, not timing): a body edit
//!    re-analyzes ONLY the edited module — zero downstream re-analysis. Already
//!    asserted green in the Stage C gate (`tests/stage_c.rs`); re-measured here as
//!    the `downstream_reanalyzed` count (must be 0).
//!  * **T4 — analysis throughput** (`check`, no codegen): lines/s through the checker.
//!  * **full-build vs rebuild ratio** — cold (all-checked) build over warm
//!    incremental rebuild, the P20 incrementality dividend.
//!
//! Emits JSON to stdout and to `target/p20-baseline.json`. Run: `cargo bench --bench p20`.

use std::path::{Path, PathBuf};
use std::time::Instant;

use candor::build::build_dir;

fn manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// Recursively copy a directory tree (source only — skips any `.candor-cache`).
fn copy_tree(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap().flatten() {
        let p = entry.path();
        let name = entry.file_name();
        if name == ".candor-cache" {
            continue;
        }
        let target = dst.join(&name);
        if p.is_dir() {
            copy_tree(&p, &target);
        } else {
            std::fs::copy(&p, &target).unwrap();
        }
    }
}

fn count_lines(dir: &Path) -> usize {
    let mut n = 0;
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let p = entry.path();
        if p.is_dir() {
            n += count_lines(&p);
        } else if p.extension().map(|e| e == "cnr").unwrap_or(false) {
            n += std::fs::read_to_string(&p).map(|s| s.lines().count()).unwrap_or(0);
        }
    }
    n
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

/// Median of a set of measured milliseconds.
fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn main() {
    let corelib = manifest().join("tests/fixtures/corelib");
    let work = std::env::temp_dir().join(format!("candor-p20-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    copy_tree(&corelib, &work);

    let n_modules = build_dir(&work).expect("build").modules.len();
    let n_lines = count_lines(&work);

    // --- Cold build (fresh cache): every module Checked. ---
    let mut cold = Vec::new();
    for _ in 0..7 {
        let _ = std::fs::remove_dir_all(work.join(".candor-cache"));
        let t = Instant::now();
        let r = build_dir(&work).expect("cold build");
        cold.push(ms(t));
        assert!(r.ok(), "cold build had errors");
        assert_eq!(r.checked().len(), n_modules, "cold build should check every module");
    }
    let cold_ms = median(cold);

    // --- No-op rebuild (warm cache, no edit): every module Reused (cache-hit floor). ---
    let mut noop = Vec::new();
    for _ in 0..7 {
        let t = Instant::now();
        let r = build_dir(&work).expect("noop rebuild");
        noop.push(ms(t));
        assert_eq!(r.reused().len(), n_modules, "no-op rebuild should reuse every module");
    }
    let noop_ms = median(noop);

    // --- T1 shape: one body edit (a comment append — source changes, no pub-signature
    //     change), warm cache. T2: exactly one module re-analyzed, zero downstream. ---
    let edited = work.join("core/opt.cnr");
    let original = std::fs::read_to_string(&edited).unwrap();
    let mut rebuild = Vec::new();
    let mut checked_on_edit = Vec::new();
    let mut downstream_reanalyzed = 0usize;
    for i in 0..7 {
        // Distinct comment each iteration so the source hash actually changes.
        std::fs::write(&edited, format!("{original}\n// p20 body edit {i}\n")).unwrap();
        let t = Instant::now();
        let r = build_dir(&work).expect("incremental rebuild");
        rebuild.push(ms(t));
        let checked = r.checked();
        // T2: the ONLY re-analyzed module is the edited one; everything else reused.
        assert!(
            checked.iter().all(|m| m.ends_with("opt")),
            "T2 violated: unexpected re-analysis {checked:?}"
        );
        checked_on_edit.push(checked.len());
        downstream_reanalyzed = checked.len().saturating_sub(1);
    }
    std::fs::write(&edited, original).unwrap();
    let rebuild_ms = median(rebuild);
    let checked_edit = *checked_on_edit.iter().max().unwrap();

    // --- T4 shape: analysis throughput (check only, no codegen). ---
    let mut check_ms = Vec::new();
    for _ in 0..15 {
        let t = Instant::now();
        let diags = candor::check_dir(&work).expect("check");
        check_ms.push(ms(t));
        assert!(!diags.iter().any(|d| d.severity == candor::diag::Severity::Error));
    }
    let check_med = median(check_ms);
    let lines_per_sec = (n_lines as f64) / (check_med / 1000.0);

    let full_vs_rebuild = cold_ms / rebuild_ms.max(1e-6);

    let json = format!(
        "{{\n  \"instrument\": \"p20-baseline\",\n  \"note\": \"prototype baselines, NOT ratified P20 claims; real numbers await toolchain scale (N=200 modules, M~50k lines)\",\n  \"reference_project\": {{ \"tree\": \"corelib\", \"modules\": {n_modules}, \"lines\": {n_lines} }},\n  \"T1_incremental_rebuild_ms\": {rebuild_ms:.3},\n  \"T2_downstream_reanalyzed\": {downstream_reanalyzed},\n  \"T2_modules_reanalyzed_on_body_edit\": {checked_edit},\n  \"T4_check_lines_per_sec\": {lines_per_sec:.0},\n  \"check_ms\": {check_med:.3},\n  \"cold_build_ms\": {cold_ms:.3},\n  \"noop_rebuild_ms\": {noop_ms:.3},\n  \"full_build_vs_rebuild_ratio\": {full_vs_rebuild:.2}\n}}"
    );
    println!("{json}");
    let out = manifest().join("target/p20-baseline.json");
    let _ = std::fs::write(&out, format!("{json}\n"));
    eprintln!("(written to {})", out.display());

    let _ = std::fs::remove_dir_all(&work);
}
