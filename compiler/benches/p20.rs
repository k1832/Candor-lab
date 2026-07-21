//! The P20 measurement harness (design 0010 §3, §5 Stage D).
//!
//! Measures the pre-registered P20 targets against TWO trees and labels both in
//! the JSON:
//!
//!  * **corelib** — the 13-module / ~1.8 kL fixture stand-in (the historical
//!    prototype baseline; kept for continuity).
//!  * **reference** — the FULL-SCALE measurement subject at
//!    `benches/p20-reference/candor` (N ≈ 200 modules, M ≈ 50 kL; design 0010 §3),
//!    the scale the ratified targets are DEFINED against. Generated + committed by
//!    `cargo run --release --bin p20-gen` (the frozen-instrument discipline).
//!
//! Per tree it reports the same shapes:
//!  * **T1 — incremental rebuild** (one body edit, no `pub`-signature change, warm
//!    cache): wall time to rebuild after touching a single module's body.
//!  * **T2 — incremental analysis scope**: a body edit re-analyzes ONLY the edited
//!    module (`downstream_reanalyzed` must be 0).
//!  * **T4 — analysis throughput** (`check`, no codegen): `check_ms` + lines/s.
//!  * **cold build** / **no-op rebuild** and the full-vs-rebuild incrementality ratio.
//!
//! And a **release baseline** for the ratified `--release ≤ 2× C` clause: the
//! candor `compile --release` (LLVM `-O2`) whole-tree native compile of the
//! reference vs `cc -O2` of the parallel C translation-unit set, both to an `-O2`
//! native ELF — an honest same-shape, comparable-scale compile-time ratio.
//!
//! The ratified targets (measurable at reference scale): clean `check` ≤ 3000 ms;
//! T1 incremental ≤ 1000 ms; T2 downstream == 0; `release_vs_cc_ratio` ≤ 2.0. A
//! target that FAILS at scale is REPORTED (a `pass` flag per target in the JSON),
//! not tuned away.
//!
//! Emits JSON to stdout and to `target/p20.json`. Run: `cargo bench --bench p20`.

use std::path::{Path, PathBuf};
use std::process::Command;
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

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

/// One tree's measured P20 shapes.
struct TreeMetrics {
    modules: usize,
    lines: usize,
    cold_ms: f64,
    noop_ms: f64,
    t1_ms: f64,
    t2_downstream: usize,
    modules_reanalyzed_on_edit: usize,
    check_ms: f64,
    lines_per_sec: f64,
    full_vs_rebuild: f64,
}

/// Measure a tree's cold/noop/T1/T2/check shapes. Copies the tree to a scratch
/// dir so the committed source keeps no `.candor-cache`. `edit_rel` is the module
/// file (relative to the tree root) whose BODY is edited for T1/T2; `edit_stem` is
/// its file stem, used to recognize the edited module in the re-analysis set.
/// Panics only on structural faults (build errors, wrong reuse count) — a broken
/// tree is a bug, not a finding; timing thresholds are reported, never asserted.
fn measure_tree(src_dir: &Path, edit_rel: &str, edit_stem: &str, iters: usize) -> TreeMetrics {
    let work = std::env::temp_dir().join(format!("candor-p20-{}-{}", edit_stem, std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    copy_tree(src_dir, &work);

    let n_modules = build_dir(&work).expect("build").modules.len();
    let n_lines = count_lines(&work);

    // Cold build (fresh cache): every module Checked.
    let mut cold = Vec::new();
    for _ in 0..iters {
        let _ = std::fs::remove_dir_all(work.join(".candor-cache"));
        let t = Instant::now();
        let r = build_dir(&work).expect("cold build");
        cold.push(ms(t));
        assert!(r.ok(), "cold build had errors");
        assert_eq!(r.checked().len(), n_modules, "cold build should check every module");
    }
    let cold_ms = median(cold);

    // No-op rebuild (warm cache, no edit): every module Reused.
    let mut noop = Vec::new();
    for _ in 0..iters {
        let t = Instant::now();
        let r = build_dir(&work).expect("noop rebuild");
        noop.push(ms(t));
        assert_eq!(r.reused().len(), n_modules, "no-op rebuild should reuse every module");
    }
    let noop_ms = median(noop);

    // T1: one body edit (comment append — source changes, no pub-signature change),
    // warm cache. T2: exactly one module re-analyzed, zero downstream.
    let edited = work.join(edit_rel);
    let original = std::fs::read_to_string(&edited).unwrap();
    let mut rebuild = Vec::new();
    let mut reanalyzed = Vec::new();
    let mut downstream = 0usize;
    for i in 0..iters {
        std::fs::write(&edited, format!("{original}\n// p20 body edit {i}\n")).unwrap();
        let t = Instant::now();
        let r = build_dir(&work).expect("incremental rebuild");
        rebuild.push(ms(t));
        let checked = r.checked();
        assert!(
            checked.iter().any(|m| m.ends_with(edit_stem)),
            "the edited module was not re-analyzed: {checked:?}"
        );
        // T2 is REPORTED, not asserted: downstream = re-analyzed modules that are
        // not the edited one.
        downstream = checked.iter().filter(|m| !m.ends_with(edit_stem)).count();
        reanalyzed.push(checked.len());
    }
    std::fs::write(&edited, original).unwrap();
    let t1_ms = median(rebuild);
    let modules_reanalyzed_on_edit = *reanalyzed.iter().max().unwrap();

    // T4: analysis throughput (check only, no codegen).
    let mut check_ms = Vec::new();
    for _ in 0..(iters * 2) {
        let t = Instant::now();
        let diags = candor::check_dir(&work).expect("check");
        check_ms.push(ms(t));
        assert!(!diags.iter().any(|d| d.severity == candor::diag::Severity::Error));
    }
    let check_med = median(check_ms);
    let lines_per_sec = (n_lines as f64) / (check_med / 1000.0);
    let full_vs_rebuild = cold_ms / t1_ms.max(1e-6);

    let _ = std::fs::remove_dir_all(&work);
    TreeMetrics {
        modules: n_modules,
        lines: n_lines,
        cold_ms,
        noop_ms,
        t1_ms,
        t2_downstream: downstream,
        modules_reanalyzed_on_edit,
        check_ms: check_med,
        lines_per_sec,
        full_vs_rebuild,
    }
}

/// Read the reference project's committed manifest to learn the T1/T2 edit target.
fn edit_target_from_manifest(root: &Path) -> String {
    let raw = std::fs::read_to_string(root.join("p20-manifest.json")).expect("p20-manifest.json");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("manifest json");
    v["edit_target"].as_str().expect("edit_target").to_string()
}

/// Median wall time (ms) of the candor `--release` (LLVM `-O2`) whole-tree native
/// compile of the reference vs `cc -O2` of the parallel C TU set. Both compile a
/// comparable-scale, same-shape scalar program to an optimized native ELF, so the
/// ratio is an honest compile-time comparison for the `--release ≤ 2× C` clause.
fn release_baseline(candor_src: &Path, c_dir: &Path, iters: usize) -> (f64, f64, usize) {
    // Compile candor from a scratch copy so the committed tree keeps no cache.
    let cwork = std::env::temp_dir().join(format!("candor-p20-rel-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&cwork);
    copy_tree(candor_src, &cwork);
    let cand_out = std::env::temp_dir().join(format!("candor-p20-rel-{}.bin", std::process::id()));

    let mut cand = Vec::new();
    for _ in 0..iters {
        let t = Instant::now();
        candor::compile_path_llvm(&cwork, &cand_out).expect("candor --release compile");
        cand.push(ms(t));
    }

    // Compile the parallel C TU set with `cc -O2`.
    let mut c_files: Vec<PathBuf> = std::fs::read_dir(c_dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "c").unwrap_or(false))
        .collect();
    c_files.sort();
    let c_out = std::env::temp_dir().join(format!("candor-p20-c-{}.bin", std::process::id()));
    let mut c_lines = 0usize;
    for p in &c_files {
        c_lines += std::fs::read_to_string(p).map(|s| s.lines().count()).unwrap_or(0);
    }

    let mut cc = Vec::new();
    for _ in 0..iters {
        let t = Instant::now();
        let status = Command::new("cc")
            .arg("-O2")
            .arg("-I")
            .arg(c_dir)
            .args(&c_files)
            .arg("-o")
            .arg(&c_out)
            .status()
            .expect("cc -O2");
        cc.push(ms(t));
        assert!(status.success(), "cc -O2 baseline failed");
    }

    let _ = std::fs::remove_dir_all(&cwork);
    let _ = std::fs::remove_file(&cand_out);
    let _ = std::fs::remove_file(&c_out);
    (median(cand), median(cc), c_lines)
}

fn pass(cond: bool) -> &'static str {
    if cond {
        "true"
    } else {
        "false"
    }
}

fn main() {
    // --- corelib stand-in (continuity baseline). ---
    let corelib = manifest().join("tests/fixtures/corelib");
    let cm = measure_tree(&corelib, "core/opt.cnr", "opt", 7);

    // --- the full-scale reference measurement subject. ---
    let ref_root = manifest().join("benches/p20-reference");
    let ref_candor = ref_root.join("candor");
    let ref_c = ref_root.join("c");
    let edit_rel = edit_target_from_manifest(&ref_root);
    let edit_stem = Path::new(&edit_rel).file_stem().unwrap().to_str().unwrap().to_string();
    let rm = measure_tree(&ref_candor, &edit_rel, &edit_stem, 3);

    // --- release baseline (compile-time ratio vs cc -O2). ---
    let (cand_rel_ms, cc_ms, c_lines) = release_baseline(&ref_candor, &ref_c, 3);
    let ratio = cand_rel_ms / cc_ms.max(1e-6);

    // Ratified targets, evaluated at reference scale.
    let check_pass = rm.check_ms <= 3000.0;
    let t1_pass = rm.t1_ms <= 1000.0;
    let t2_pass = rm.t2_downstream == 0;
    let ratio_pass = ratio <= 2.0;

    let json = format!(
        "{{\n\
         \x20 \"instrument\": \"p20\",\n\
         \x20 \"trees\": {{\n\
         \x20   \"corelib\": {{\n\
         \x20     \"role\": \"continuity baseline (not the ratified scale)\",\n\
         \x20     \"modules\": {}, \"lines\": {},\n\
         \x20     \"T1_incremental_rebuild_ms\": {:.3},\n\
         \x20     \"T2_downstream_reanalyzed\": {},\n\
         \x20     \"T2_modules_reanalyzed_on_body_edit\": {},\n\
         \x20     \"T4_check_lines_per_sec\": {:.0},\n\
         \x20     \"check_ms\": {:.3},\n\
         \x20     \"cold_build_ms\": {:.3},\n\
         \x20     \"noop_rebuild_ms\": {:.3},\n\
         \x20     \"full_build_vs_rebuild_ratio\": {:.2}\n\
         \x20   }},\n\
         \x20   \"reference\": {{\n\
         \x20     \"role\": \"ratified P20 scale (design 0010 §3, N~200 modules, M~50kL)\",\n\
         \x20     \"modules\": {}, \"lines\": {},\n\
         \x20     \"T1_incremental_rebuild_ms\": {:.3},\n\
         \x20     \"T2_downstream_reanalyzed\": {},\n\
         \x20     \"T2_modules_reanalyzed_on_body_edit\": {},\n\
         \x20     \"T4_check_lines_per_sec\": {:.0},\n\
         \x20     \"check_ms\": {:.3},\n\
         \x20     \"cold_build_ms\": {:.3},\n\
         \x20     \"noop_rebuild_ms\": {:.3},\n\
         \x20     \"full_build_vs_rebuild_ratio\": {:.2},\n\
         \x20     \"edit_target\": {},\n\
         \x20     \"targets\": {{\n\
         \x20       \"check_le_3000ms\": {{ \"actual\": {:.3}, \"pass\": {} }},\n\
         \x20       \"T1_le_1000ms\": {{ \"actual\": {:.3}, \"pass\": {} }},\n\
         \x20       \"T2_downstream_eq_0\": {{ \"actual\": {}, \"pass\": {} }}\n\
         \x20     }}\n\
         \x20   }}\n\
         \x20 }},\n\
         \x20 \"release_baseline\": {{\n\
         \x20   \"note\": \"candor `compile --release` (LLVM -O2) whole-tree native compile of the {}-line reference vs `cc -O2` of the parallel {}-line C TU set; both emit an -O2 native ELF. Same-shape scalar workload; compile-time ratio.\",\n\
         \x20   \"candor_release_compile_ms\": {:.3},\n\
         \x20   \"cc_O2_compile_ms\": {:.3},\n\
         \x20   \"release_vs_cc_ratio\": {:.3},\n\
         \x20   \"target_le_2x\": {{ \"actual\": {:.3}, \"pass\": {} }}\n\
         \x20 }}\n\
         }}",
        cm.modules, cm.lines, cm.t1_ms, cm.t2_downstream, cm.modules_reanalyzed_on_edit,
        cm.lines_per_sec, cm.check_ms, cm.cold_ms, cm.noop_ms, cm.full_vs_rebuild,
        rm.modules, rm.lines, rm.t1_ms, rm.t2_downstream, rm.modules_reanalyzed_on_edit,
        rm.lines_per_sec, rm.check_ms, rm.cold_ms, rm.noop_ms, rm.full_vs_rebuild,
        serde_json::to_string(&edit_rel).unwrap(),
        rm.check_ms, pass(check_pass),
        rm.t1_ms, pass(t1_pass),
        rm.t2_downstream, pass(t2_pass),
        rm.lines, c_lines,
        cand_rel_ms, cc_ms, ratio, ratio, pass(ratio_pass),
    );
    println!("{json}");
    let out = manifest().join("target/p20.json");
    let _ = std::fs::write(&out, format!("{json}\n"));
    eprintln!("(written to {})", out.display());

    // Surface any ratified-target FAILURE plainly on stderr (a reportable finding).
    for (name, ok) in [
        ("check <= 3000ms", check_pass),
        ("T1 <= 1000ms", t1_pass),
        ("T2 downstream == 0", t2_pass),
        ("release <= 2x cc -O2", ratio_pass),
    ] {
        if !ok {
            eprintln!("P20 FINDING: ratified target FAILS at reference scale: {name}");
        }
    }
}
