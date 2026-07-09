//! The manifest: the machine-readable record of a generated corpus. One entry
//! per KEPT sample (shape, seed, params, expected sentinel-or-diagnostic,
//! toolchain version) plus per-shape kept/rejected tallies — the reject rate is
//! the generator's grounding signal (high rejects = misgrounded, see README).

use serde::{Deserialize, Serialize};

/// What the generator predicted and the toolchain confirmed.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Expected {
    /// A positive sample: `run` must print exactly this `i64`.
    Sentinel { value: i64 },
    /// A negative sample: `check` must emit exactly this diagnostic code.
    Diagnostic { code: String },
}

/// One kept sample.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SampleRecord {
    pub id: String,
    pub category: String,
    pub shape: String,
    /// The corpus-wide seed this sample descends from.
    pub seed: u64,
    /// The candidate ordinal (within its category) that produced it — the exact
    /// point in the deterministic draw stream, for reproduction.
    pub draw: usize,
    /// Path relative to the corpus root.
    pub file: String,
    pub params: serde_json::Value,
    pub expected: Expected,
    /// The observed filter result (sentinel value or diagnostic code).
    pub observed: serde_json::Value,
    pub toolchain_version: String,
}

/// Per-shape kept/rejected tally within a category.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ShapeStat {
    pub category: String,
    pub shape: String,
    pub kept: usize,
    pub rejected: usize,
}

impl ShapeStat {
    /// Kept fraction of candidates drawn for this shape (1.0 == perfectly grounded).
    pub fn kept_rate(&self) -> f64 {
        let total = self.kept + self.rejected;
        if total == 0 {
            0.0
        } else {
            self.kept as f64 / total as f64
        }
    }
}

/// The whole corpus record.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Manifest {
    pub toolchain_version: String,
    pub seed: u64,
    pub positive_count: usize,
    pub negative_count: usize,
    pub stats: Vec<ShapeStat>,
    pub samples: Vec<SampleRecord>,
}
