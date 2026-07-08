//! Candor P19 evaluation-harness seed.
//!
//! This crate DEFINES externally-anchored model-competence tasks and SCORES
//! submissions against them via the `candor-proto` oracle. It drives no model:
//! generation/repair is the operator's side (see README). Anchors are authored
//! from the frozen basket specs and spec-pack, never model-generated — the Bet 6
//! anti-circularity requirement.

pub mod candor;
pub mod report;
pub mod score;
pub mod task;

use std::path::{Path, PathBuf};

pub use candor::Oracle;
pub use report::Report;
pub use score::{Stage, TaskResult};
pub use task::{Category, Task};

/// A scoring run's configuration.
pub struct Config {
    /// Directory holding the model's submission files.
    pub submission_dir: PathBuf,
    /// Directory of `*.json` task files.
    pub tasks_dir: PathBuf,
    /// Root that resolves anchor `battery_file` paths.
    pub root: PathBuf,
    /// The candor-proto binary (path or name on `PATH`).
    pub candor_bin: String,
    /// Round label for the report (slope substrate).
    pub round: u32,
}

/// Load the task set, score every task, and build the report.
pub fn run_scoring(cfg: &Config) -> Result<Report, String> {
    let tasks = task::load_tasks(&cfg.tasks_dir)?;
    let oracle = Oracle::new(cfg.candor_bin.clone());
    let mut results: Vec<TaskResult> = Vec::with_capacity(tasks.len());
    for t in &tasks {
        let r = score::score_task(&oracle, t, &cfg.submission_dir, &cfg.root).map_err(|e| {
            format!(
                "scoring `{}`: failed to invoke `{}` ({e}). Build/point --candor at candor-proto.",
                t.id, cfg.candor_bin
            )
        })?;
        results.push(r);
    }
    Ok(Report::build(
        cfg.round,
        cfg.candor_bin.clone(),
        cfg.submission_dir.display().to_string(),
        results,
    ))
}

/// Convenience for callers (and tests) that keep tasks/anchors under one root.
pub fn config_under_root(
    root: &Path,
    submission_dir: PathBuf,
    candor_bin: String,
    round: u32,
) -> Config {
    Config {
        submission_dir,
        tasks_dir: root.join("tasks"),
        root: root.to_path_buf(),
        candor_bin,
        round,
    }
}
