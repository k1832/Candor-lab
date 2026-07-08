//! Task format (P19 external-anchor tasks).
//!
//! A task is a JSON file under `tasks/`. It carries what the OPERATOR gives the
//! model (`prompt_material`: the spec-pack manifest, and for repair tasks the
//! buggy program + the actual candor-proto diagnostic) and the HIDDEN acceptance
//! criterion (`anchor`) the harness scores against. The anchor is never shown to
//! the model — that is what keeps the measurement external (Bet 6).

use serde::{Deserialize, Serialize};

/// The three task shapes. `explain` is defined for completeness but the seed set
/// contains only `generate` and `repair` (the mechanically scorable kinds); see
/// README "What this seed does NOT measure".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Generate,
    Repair,
    Explain,
}

impl Category {
    pub fn as_str(self) -> &'static str {
        match self {
            Category::Generate => "generate",
            Category::Repair => "repair",
            Category::Explain => "explain",
        }
    }
}

/// What the operator hands the model. The harness does not read `spec_pack` for
/// scoring — it is the manifest of context the operator loads. `given_program`
/// and `given_diagnostic` are the repair-task prompt (the buggy source and the
/// real candor-proto JSON diagnostic).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PromptMaterial {
    pub spec_pack: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub given_program: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub given_diagnostic: Option<serde_json::Value>,
}

/// How a submission is accepted.
/// * `run_sentinel` — the hidden `battery_file` main is appended to the
///   submission; it must compile clean AND `run` must emit `expected_sentinel`.
/// * `check_pass` — the assembled program must only pass `check`.
/// * `diagnostic_resolved` — a repair: the submission (a full program) must
///   compile clean and `run` must emit `expected_sentinel` (behaviour unchanged).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnchorKind {
    RunSentinel,
    CheckPass,
    DiagnosticResolved,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Anchor {
    pub kind: AnchorKind,
    /// Generate tasks only: the hidden test-harness main appended to the
    /// submission, resolved relative to the harness root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub battery_file: Option<String>,
    /// The expected `run` sentinel (stdout, trimmed) for `run_sentinel` /
    /// `diagnostic_resolved`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_sentinel: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Task {
    pub id: String,
    pub category: Category,
    pub title: String,
    pub statement: String,
    pub submission_filename: String,
    pub prompt_material: PromptMaterial,
    pub anchor: Anchor,
}

impl Task {
    /// Validate internal coherence of an anchor (fail loudly on a malformed task
    /// file rather than silently mis-scoring).
    pub fn validate(&self) -> Result<(), String> {
        match self.anchor.kind {
            AnchorKind::RunSentinel => {
                if self.anchor.battery_file.is_none() {
                    return Err(format!("task `{}`: run_sentinel needs a battery_file", self.id));
                }
                if self.anchor.expected_sentinel.is_none() {
                    return Err(format!("task `{}`: run_sentinel needs an expected_sentinel", self.id));
                }
            }
            AnchorKind::DiagnosticResolved => {
                if self.anchor.expected_sentinel.is_none() {
                    return Err(format!(
                        "task `{}`: diagnostic_resolved needs an expected_sentinel",
                        self.id
                    ));
                }
            }
            AnchorKind::CheckPass => {}
        }
        Ok(())
    }
}

/// Load and validate every `*.json` task under `dir`, sorted by id, with unique
/// ids enforced.
pub fn load_tasks(dir: &std::path::Path) -> Result<Vec<Task>, String> {
    let mut tasks: Vec<Task> = Vec::new();
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("cannot read tasks dir `{}`: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("reading tasks dir: {e}"))?;
        let path = entry.path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            let text = std::fs::read_to_string(&path)
                .map_err(|e| format!("cannot read `{}`: {e}", path.display()))?;
            let task: Task = serde_json::from_str(&text)
                .map_err(|e| format!("malformed task `{}`: {e}", path.display()))?;
            task.validate()?;
            tasks.push(task);
        }
    }
    tasks.sort_by(|a, b| a.id.cmp(&b.id));
    for pair in tasks.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(format!("duplicate task id `{}`", pair[0].id));
        }
    }
    if tasks.is_empty() {
        return Err(format!("no tasks found under `{}`", dir.display()));
    }
    Ok(tasks)
}
