//! Scoring: assemble each submission with its anchor, run it through the oracle,
//! and classify the outcome by failure STAGE.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

use crate::candor::{first_json, Oracle};
use crate::task::{AnchorKind, Category, Task};

/// Where a task failed. `None` on the result means it passed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Stage {
    /// The submission file was not present in the submission dir.
    Missing,
    /// candor-proto rejected the shape (P0xxx / lexer codes).
    Parse,
    /// candor-proto rejected the semantics (E-codes: move/borrow/effect/...).
    Check,
    /// Compiled clean but faulted at runtime (a failed `assert`, overflow, ...).
    Run,
    /// Compiled and ran, but emitted the wrong sentinel.
    WrongSentinel,
}

/// The per-task score record.
#[derive(Debug, Clone, Serialize)]
pub struct TaskResult {
    pub id: String,
    pub category: Category,
    pub pass: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<Stage>,
    /// The diagnostic code at a parse/check failure (e.g. `"E0301"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_sentinel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_sentinel: Option<String>,
    /// The repair-loop hook: the machine-readable diagnostic (or fault) the
    /// operator feeds back to the model for a round-2 attempt. The substrate of
    /// the P19 competence-slope metric.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_diagnostic: Option<serde_json::Value>,
}

impl TaskResult {
    fn pass(task: &Task) -> TaskResult {
        TaskResult {
            id: task.id.clone(),
            category: task.category,
            pass: true,
            stage: None,
            failure_code: None,
            expected_sentinel: None,
            actual_sentinel: None,
            feedback_diagnostic: None,
        }
    }

    fn fail(task: &Task, stage: Stage) -> TaskResult {
        TaskResult {
            id: task.id.clone(),
            category: task.category,
            pass: false,
            stage: Some(stage),
            failure_code: None,
            expected_sentinel: None,
            actual_sentinel: None,
            feedback_diagnostic: None,
        }
    }
}

static TEMP_SEQ: AtomicU64 = AtomicU64::new(0);

fn temp_path(id: &str) -> PathBuf {
    let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let name = format!("eval-harness-{}-{}-{}.cnr", std::process::id(), seq, id);
    std::env::temp_dir().join(name)
}

/// Score one task against a submission directory. `root` resolves the anchor's
/// `battery_file`. Oracle-launch failures propagate (an operator/config error,
/// not a task failure).
pub fn score_task(
    oracle: &Oracle,
    task: &Task,
    submission_dir: &Path,
    root: &Path,
) -> std::io::Result<TaskResult> {
    let sub_path = submission_dir.join(&task.submission_filename);
    let submission = match std::fs::read_to_string(&sub_path) {
        Ok(s) => s,
        Err(_) => return Ok(TaskResult::fail(task, Stage::Missing)),
    };

    // Assemble the program the oracle actually sees.
    let program = match &task.anchor.battery_file {
        Some(bf) => {
            let battery = std::fs::read_to_string(root.join(bf))?;
            format!("{submission}\n{battery}")
        }
        None => submission,
    };

    let temp = temp_path(&task.id);
    std::fs::write(&temp, &program)?;
    let result = evaluate(oracle, task, &temp);
    let _ = std::fs::remove_file(&temp);
    result
}

fn evaluate(oracle: &Oracle, task: &Task, temp: &Path) -> std::io::Result<TaskResult> {
    // Stage 1: check.
    let chk = oracle.check(temp)?;
    if chk.code != 0 {
        return Ok(from_diag_output(task, &chk.stdout));
    }
    if task.anchor.kind == AnchorKind::CheckPass {
        return Ok(TaskResult::pass(task));
    }

    // Stage 2: run.
    let expected = task.anchor.expected_sentinel.clone().unwrap_or_default();
    let r = oracle.run(temp)?;
    if r.code == 0 {
        let actual = r.stdout.trim().to_string();
        if actual == expected {
            return Ok(TaskResult::pass(task));
        }
        let mut res = TaskResult::fail(task, Stage::WrongSentinel);
        res.expected_sentinel = Some(expected);
        res.actual_sentinel = Some(actual);
        return Ok(res);
    }

    // Non-zero run: a fault (stderr JSON) or a check error surfaced at run.
    if let Some((fault, _)) = first_json(&r.stderr) {
        let mut res = TaskResult::fail(task, Stage::Run);
        res.expected_sentinel = Some(expected);
        res.feedback_diagnostic = Some(fault);
        return Ok(res);
    }
    let mut res = from_diag_output(task, &r.stdout);
    res.expected_sentinel = Some(expected);
    Ok(res)
}

/// Turn a diagnostic-bearing stdout into a Parse/Check failure result.
fn from_diag_output(task: &Task, stdout: &str) -> TaskResult {
    match first_json(stdout) {
        Some((diag, code)) => {
            let stage = match code.as_deref() {
                Some(c) if c.starts_with('E') => Stage::Check,
                _ => Stage::Parse, // P0xxx parser + L* lexer codes
            };
            let mut res = TaskResult::fail(task, stage);
            res.failure_code = code;
            res.feedback_diagnostic = Some(diag);
            res
        }
        // Non-zero exit with no parseable diagnostic: treat as a parse-stage
        // rejection with no feedback rather than silently passing.
        None => TaskResult::fail(task, Stage::Parse),
    }
}

/// Category rollup for the aggregate.
#[derive(Debug, Clone, Serialize, Default)]
pub struct CatAgg {
    pub total: usize,
    pub passed: usize,
}

pub fn category_key(c: Category) -> &'static str {
    c.as_str()
}
