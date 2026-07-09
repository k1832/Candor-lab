//! `corpus-gen generate --seed N --count K` — the P19 synthetic-corpus CLI.
//!
//! Usage:
//!   corpus-gen generate [--seed N] [--count K] [--positive P] [--negative Q]
//!                       [--out DIR] [--candor PATH] [--toolchain-version STR]
//!
//! `--count K` sets the total kept samples, split 3:1 positive:negative by
//! default; `--positive`/`--negative` override the split explicitly. Every
//! candidate is filtered by `candor-proto` (`--candor` or `$CANDOR_PROTO`, else
//! `candor-proto` on `PATH`). Exit 0 on success, 2 on a usage/config error, 1 if
//! generation could not reach the target (a misgrounded shape).

use corpus_gen::{generate, Config, DEFAULT_TOOLCHAIN_VERSION};
use corpus_gen::oracle::Oracle;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) != Some("generate") {
        eprintln!("usage: corpus-gen generate [--seed N] [--count K] [--positive P] [--negative Q] [--out DIR] [--candor PATH] [--toolchain-version STR]");
        return ExitCode::from(2);
    }

    let mut seed: u64 = 1;
    let mut count: Option<usize> = None;
    let mut positive: Option<usize> = None;
    let mut negative: Option<usize> = None;
    let mut out = PathBuf::from("corpus");
    let mut candor = std::env::var("CANDOR_PROTO").unwrap_or_else(|_| "candor-proto".to_string());
    let mut toolchain_version = DEFAULT_TOOLCHAIN_VERSION.to_string();

    let mut i = 2;
    while i < args.len() {
        let need = |i: usize| -> Result<&String, ExitCode> {
            args.get(i + 1).ok_or_else(|| {
                eprintln!("error: `{}` needs a value", args[i]);
                ExitCode::from(2)
            })
        };
        match args[i].as_str() {
            "--seed" => match need(i).map(|v| v.parse::<u64>()) {
                Ok(Ok(v)) => seed = v,
                _ => {
                    eprintln!("error: --seed needs a u64");
                    return ExitCode::from(2);
                }
            },
            "--count" => match need(i).map(|v| v.parse::<usize>()) {
                Ok(Ok(v)) => count = Some(v),
                _ => return ExitCode::from(2),
            },
            "--positive" => match need(i).map(|v| v.parse::<usize>()) {
                Ok(Ok(v)) => positive = Some(v),
                _ => return ExitCode::from(2),
            },
            "--negative" => match need(i).map(|v| v.parse::<usize>()) {
                Ok(Ok(v)) => negative = Some(v),
                _ => return ExitCode::from(2),
            },
            "--out" => match need(i) {
                Ok(v) => out = PathBuf::from(v),
                Err(c) => return c,
            },
            "--candor" => match need(i) {
                Ok(v) => candor = v.clone(),
                Err(c) => return c,
            },
            "--toolchain-version" => match need(i) {
                Ok(v) => toolchain_version = v.clone(),
                Err(c) => return c,
            },
            other => {
                eprintln!("error: unknown argument `{other}`");
                return ExitCode::from(2);
            }
        }
        i += 2;
    }

    // Resolve the positive/negative split: explicit overrides win; else split
    // `--count` 3:1 (150/50 at the seed default of 200); else the 150/50 default.
    let (positive, negative) = match (positive, negative, count) {
        (Some(p), Some(q), _) => (p, q),
        (Some(p), None, Some(k)) => (p, k.saturating_sub(p)),
        (None, Some(q), Some(k)) => (k.saturating_sub(q), q),
        (Some(p), None, None) => (p, 0),
        (None, Some(q), None) => (0, q),
        (None, None, Some(k)) => {
            let p = k * 3 / 4;
            (p, k - p)
        }
        (None, None, None) => (150, 50),
    };

    let cfg = Config {
        seed,
        positive,
        negative,
        out_dir: out.clone(),
        oracle: Oracle::new(candor.clone()),
        toolchain_version,
        max_attempts: (positive + negative).max(1) * 200,
    };

    match generate(&cfg) {
        Ok(m) => {
            eprintln!(
                "generated {} samples ({} positive / {} negative) into {} — filtered by `{}`",
                m.samples.len(),
                m.positive_count,
                m.negative_count,
                out.display(),
                candor
            );
            for s in &m.stats {
                let total = s.kept + s.rejected;
                eprintln!(
                    "  {:<9} {:<26} kept {:>3}/{:<3} ({:.0}%)",
                    s.category,
                    s.shape,
                    s.kept,
                    total,
                    s.kept_rate() * 100.0
                );
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
