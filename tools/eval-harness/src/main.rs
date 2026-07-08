//! `eval-harness` CLI.
//!
//! Usage:
//!   eval-harness score <submission_dir> [--tasks <dir>] [--root <dir>]
//!                                       [--candor <path>] [--round <N>]
//!                                       [--report <file>]
//!
//! Loads the task set, scores each submission through candor-proto, prints the
//! JSON report to stdout (and optionally to --report). Exit 0 iff every task
//! passed (a CI gate); 1 if any task failed; 2 on a configuration error.
//!
//! Defaults: --root `.`, --tasks `<root>/tasks`, --candor `$CANDOR_PROTO` or
//! `candor-proto` on PATH.

use std::path::PathBuf;
use std::process::ExitCode;

use eval_harness::{run_scoring, Config};

fn fail_config(msg: &str) -> ExitCode {
    eprintln!("error: {msg}");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) != Some("score") {
        eprintln!("usage: eval-harness score <submission_dir> [--tasks <dir>] [--root <dir>] [--candor <path>] [--round <N>] [--report <file>]");
        return ExitCode::from(2);
    }
    let submission_dir = match args.get(2) {
        Some(d) if !d.starts_with("--") => PathBuf::from(d),
        _ => return fail_config("`score` needs a <submission_dir>"),
    };

    let mut root = PathBuf::from(".");
    let mut tasks_dir: Option<PathBuf> = None;
    let mut candor = std::env::var("CANDOR_PROTO").unwrap_or_else(|_| "candor-proto".to_string());
    let mut round: u32 = 1;
    let mut report_path: Option<PathBuf> = None;

    let mut i = 3;
    while i < args.len() {
        let flag = args[i].as_str();
        // Every flag here takes one value.
        let value = match args.get(i + 1) {
            Some(v) => v.clone(),
            None => return fail_config(&format!("`{flag}` requires a value")),
        };
        match flag {
            "--root" => root = PathBuf::from(value),
            "--tasks" => tasks_dir = Some(PathBuf::from(value)),
            "--candor" => candor = value,
            "--round" => {
                round = match value.parse() {
                    Ok(n) => n,
                    Err(_) => return fail_config("--round expects a number"),
                }
            }
            "--report" => report_path = Some(PathBuf::from(value)),
            other => return fail_config(&format!("unexpected argument `{other}`")),
        }
        i += 2;
    }

    let cfg = Config {
        submission_dir,
        tasks_dir: tasks_dir.unwrap_or_else(|| root.join("tasks")),
        root,
        candor_bin: candor,
        round,
    };

    let report = match run_scoring(&cfg) {
        Ok(r) => r,
        Err(e) => return fail_config(&e),
    };

    let json = report.to_json_pretty();
    println!("{json}");
    if let Some(p) = report_path {
        if let Err(e) = std::fs::write(&p, format!("{json}\n")) {
            return fail_config(&format!("cannot write report `{}`: {e}", p.display()));
        }
    }

    if report.all_passed() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
