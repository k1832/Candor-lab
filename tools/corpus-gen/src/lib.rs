//! corpus-gen — the P19 synthetic-corpus pipeline seed.
//!
//! Grammar-directed generation of `.cnr` programs, FILTERED by the `candor-proto`
//! toolchain: positives are kept iff they compile and `run` to a
//! generator-predicted sentinel; negatives are kept iff `check` emits exactly the
//! diagnostic the shape was authored to trip. The output is a
//! `(program, expected)` corpus + a `manifest.json`.
//!
//! Bet 6 (anti-circularity): this corpus is TRAINING material. Toolchain
//! filtering validates INTERNAL CONSISTENCY (compiles / runs to sentinel / emits
//! code), never correctness or idiom. The evaluation anchors stay EXTERNAL and
//! are NEVER sourced from this pipeline (see `README.md`, and the sibling
//! `tools/eval-harness`).

pub mod manifest;
pub mod oracle;
pub mod rng;
pub mod shapes;

use manifest::{Expected, Manifest, SampleRecord, ShapeStat};
use oracle::{first_code, Oracle};
use rng::Rng;
use serde_json::json;
use shapes::{Candidate, Gen};
use std::io;
use std::path::{Path, PathBuf};

/// The default toolchain-version stamp. There is no `candor-proto --version`, so
/// the seed corpus records a fixed prototype stamp (kept out of the RNG for
/// determinism); the real pipeline stamps the compiler's version/commit hash.
pub const DEFAULT_TOOLCHAIN_VERSION: &str = "candor-proto 0.1.0 (Bet 5 prototype)";

/// A generation request.
pub struct Config {
    pub seed: u64,
    pub positive: usize,
    pub negative: usize,
    pub out_dir: PathBuf,
    pub oracle: Oracle,
    pub toolchain_version: String,
    /// Safety bound: max candidates drawn per category before giving up (a shape
    /// that never passes the filter would otherwise loop forever).
    pub max_attempts: usize,
}

/// One category's identity.
#[derive(Clone, Copy)]
enum Category {
    Positive,
    Negative,
}

impl Category {
    fn label(self) -> &'static str {
        match self {
            Category::Positive => "positive",
            Category::Negative => "negative",
        }
    }
    fn prefix(self) -> &'static str {
        match self {
            Category::Positive => "pos",
            Category::Negative => "neg",
        }
    }
    fn shapes(self) -> &'static [(&'static str, Gen)] {
        match self {
            Category::Positive => shapes::POSITIVE,
            Category::Negative => shapes::NEGATIVE,
        }
    }
}

/// The verdict of running one candidate through the toolchain filter.
enum Verdict {
    /// Kept; `observed` records what the toolchain reported (sentinel / code).
    Kept(serde_json::Value),
    /// Rejected; the string is a human reason (for reporting, not the corpus).
    Rejected(String),
}

/// Generate a corpus into `cfg.out_dir`, returning the manifest. Overwrites the
/// `positive/` and `negative/` subtrees and `manifest.json`.
pub fn generate(cfg: &Config) -> io::Result<Manifest> {
    let pos_dir = cfg.out_dir.join("positive");
    let neg_dir = cfg.out_dir.join("negative");
    for d in [&pos_dir, &neg_dir] {
        if d.exists() {
            std::fs::remove_dir_all(d)?;
        }
        std::fs::create_dir_all(d)?;
    }
    let staging = cfg.out_dir.join(".staging.cnr");

    let mut rng = Rng::new(cfg.seed);
    let mut samples: Vec<SampleRecord> = Vec::new();
    let mut stats: Vec<ShapeStat> = Vec::new();

    generate_category(cfg, Category::Positive, cfg.positive, &mut rng, &staging, &mut samples, &mut stats)?;
    generate_category(cfg, Category::Negative, cfg.negative, &mut rng, &staging, &mut samples, &mut stats)?;

    if staging.exists() {
        std::fs::remove_file(&staging)?;
    }

    let manifest = Manifest {
        toolchain_version: cfg.toolchain_version.clone(),
        seed: cfg.seed,
        positive_count: cfg.positive,
        negative_count: cfg.negative,
        stats,
        samples,
    };
    let json = serde_json::to_string_pretty(&manifest).expect("manifest serializes");
    std::fs::write(cfg.out_dir.join("manifest.json"), format!("{json}\n"))?;
    Ok(manifest)
}

#[allow(clippy::too_many_arguments)]
fn generate_category(
    cfg: &Config,
    cat: Category,
    target: usize,
    rng: &mut Rng,
    staging: &Path,
    samples: &mut Vec<SampleRecord>,
    stats: &mut Vec<ShapeStat>,
) -> io::Result<()> {
    let shapes = cat.shapes();
    let base = stats.len();
    for (name, _) in shapes {
        stats.push(ShapeStat {
            category: cat.label().to_string(),
            shape: (*name).to_string(),
            kept: 0,
            rejected: 0,
        });
    }
    let stat_of = |name: &str| -> usize {
        base + shapes.iter().position(|(n, _)| *n == name).unwrap()
    };

    let dir = cfg.out_dir.join(cat.label());
    let mut kept = 0usize;
    let mut draw = 0usize;
    while kept < target {
        if draw >= cfg.max_attempts {
            return Err(io::Error::other(
                format!(
                    "gave up after {} {} candidates with only {}/{} kept — a shape is misgrounded",
                    draw,
                    cat.label(),
                    kept,
                    target
                ),
            ));
        }
        // Round-robin over the shape registry: deterministic, even coverage.
        let (name, gen) = shapes[draw % shapes.len()];
        let cand = gen(rng);
        debug_assert_eq!(cand.shape, name);
        let si = stat_of(cand.shape);

        match filter(&cfg.oracle, staging, &cand)? {
            Verdict::Kept(observed) => {
                let id = format!("{}_{:04}_{}", cat.prefix(), kept, cand.shape);
                let file_rel = format!("{}/{}.cnr", cat.label(), id);
                std::fs::write(dir.join(format!("{id}.cnr")), &cand.program)?;
                samples.push(SampleRecord {
                    id,
                    category: cat.label().to_string(),
                    shape: cand.shape.to_string(),
                    seed: cfg.seed,
                    draw,
                    file: file_rel,
                    params: cand.params,
                    expected: cand.expected,
                    observed,
                    toolchain_version: cfg.toolchain_version.clone(),
                });
                stats[si].kept += 1;
                kept += 1;
            }
            Verdict::Rejected(reason) => {
                eprintln!("  reject [{}] {}: {}", cat.label(), cand.shape, reason);
                stats[si].rejected += 1;
            }
        }
        draw += 1;
    }
    Ok(())
}

/// Run one candidate through the toolchain filter (Bet 6's guard in code).
fn filter(oracle: &Oracle, staging: &Path, cand: &Candidate) -> io::Result<Verdict> {
    std::fs::write(staging, &cand.program)?;
    match &cand.expected {
        Expected::Sentinel { value } => {
            let check = oracle.check(staging)?;
            if check.code != 0 {
                let code = first_code(&check.stdout).unwrap_or_else(|| "?".to_string());
                return Ok(Verdict::Rejected(format!("check failed ({code})")));
            }
            let run = oracle.run(staging)?;
            if run.code != 0 {
                return Ok(Verdict::Rejected(format!("run faulted (exit {})", run.code)));
            }
            let got = run.stdout.trim();
            if got != value.to_string() {
                return Ok(Verdict::Rejected(format!("sentinel {value} != {got}")));
            }
            Ok(Verdict::Kept(json!({ "check_exit": 0, "run_sentinel": value })))
        }
        Expected::Diagnostic { code } => {
            let check = oracle.check(staging)?;
            if check.code == 0 {
                return Ok(Verdict::Rejected("check unexpectedly clean".to_string()));
            }
            match first_code(&check.stdout) {
                Some(got) if &got == code => {
                    Ok(Verdict::Kept(json!({ "check_exit": check.code, "code": got })))
                }
                Some(got) => Ok(Verdict::Rejected(format!("code {got} != {code}"))),
                None => Ok(Verdict::Rejected("no diagnostic code".to_string())),
            }
        }
    }
}
