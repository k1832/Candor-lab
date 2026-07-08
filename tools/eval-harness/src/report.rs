//! The JSON report: per-task results, the aggregate first-attempt rate, and the
//! round label (the slope substrate — re-run round 2 with the same tasks after
//! feeding `feedback_diagnostic` back to the model, and compare rates).

use std::collections::BTreeMap;

use serde::Serialize;

use crate::score::{CatAgg, TaskResult};

#[derive(Debug, Clone, Serialize)]
pub struct Aggregate {
    pub total: usize,
    pub passed: usize,
    /// passed / total, in `[0, 1]`. The P19 "first-attempt correctness" metric
    /// for this round (round 1 = first attempt; later rounds = post-repair).
    pub first_attempt_rate: f64,
    pub by_category: BTreeMap<String, CatAgg>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    /// Round index. Round 1 is the first-attempt score; the operator drives the
    /// model with `feedback_diagnostic` and re-scores as round 2+ for the slope.
    pub round: u32,
    pub candor_proto: String,
    pub submission_dir: String,
    pub aggregate: Aggregate,
    pub tasks: Vec<TaskResult>,
}

impl Report {
    pub fn build(
        round: u32,
        candor_proto: String,
        submission_dir: String,
        tasks: Vec<TaskResult>,
    ) -> Report {
        let total = tasks.len();
        let passed = tasks.iter().filter(|t| t.pass).count();
        let mut by_category: BTreeMap<String, CatAgg> = BTreeMap::new();
        for t in &tasks {
            let e = by_category.entry(t.category.as_str().to_string()).or_default();
            e.total += 1;
            if t.pass {
                e.passed += 1;
            }
        }
        let first_attempt_rate = if total == 0 {
            0.0
        } else {
            passed as f64 / total as f64
        };
        Report {
            round,
            candor_proto,
            submission_dir,
            aggregate: Aggregate {
                total,
                passed,
                first_attempt_rate,
                by_category,
            },
            tasks,
        }
    }

    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).expect("Report is serializable")
    }

    /// True when every task passed (a useful CI gate).
    pub fn all_passed(&self) -> bool {
        self.aggregate.total > 0 && self.aggregate.passed == self.aggregate.total
    }
}
