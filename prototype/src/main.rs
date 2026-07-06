//! `candor-proto` CLI.
//!
//! Usage: `candor-proto parse <file>`
//! On success: prints the parsed AST (Debug) to stdout, exit 0.
//! On error:   prints one JSON diagnostic (P4) to stdout, exit 1.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("parse") => match args.get(2) {
            Some(path) => run_parse(path),
            None => {
                eprintln!("usage: candor-proto parse <file>");
                ExitCode::from(2)
            }
        },
        _ => {
            eprintln!("usage: candor-proto parse <file>");
            ExitCode::from(2)
        }
    }
}

fn run_parse(path: &str) -> ExitCode {
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read `{path}`: {e}");
            return ExitCode::from(2);
        }
    };
    match candor_proto::parse_source(&src) {
        Ok(program) => {
            println!("{program:#?}");
            ExitCode::SUCCESS
        }
        Err(diag) => {
            println!("{}", diag.to_json());
            ExitCode::FAILURE
        }
    }
}
