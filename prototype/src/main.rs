//! `candor-proto` CLI.
//!
//! Usage:
//!   candor-proto parse <file>   -- print the parsed AST (Debug), exit 0/1
//!   candor-proto check <file>   -- parse + resolve + Stage 2 check;
//!                                  print JSON diagnostics (one per line),
//!                                  exit 0 if clean, 1 if any error.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match (args.get(1).map(String::as_str), args.get(2)) {
        (Some("parse"), Some(path)) => run_parse(path),
        (Some("check"), Some(path)) => run_check(path),
        (Some("run"), Some(path)) => run_run(path),
        _ => {
            eprintln!("usage: candor-proto (parse|check|run) <file>");
            ExitCode::from(2)
        }
    }
}

fn read(path: &str) -> Result<String, ExitCode> {
    std::fs::read_to_string(path).map_err(|e| {
        eprintln!("error: cannot read `{path}`: {e}");
        ExitCode::from(2)
    })
}

fn run_parse(path: &str) -> ExitCode {
    let src = match read(path) {
        Ok(s) => s,
        Err(c) => return c,
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

fn run_check(path: &str) -> ExitCode {
    let src = match read(path) {
        Ok(s) => s,
        Err(c) => return c,
    };
    match candor_proto::check_source(&src) {
        Ok(diags) => {
            for d in &diags {
                println!("{}", d.to_json());
            }
            if diags.is_empty() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(diag) => {
            println!("{}", diag.to_json());
            ExitCode::FAILURE
        }
    }
}

fn run_run(path: &str) -> ExitCode {
    let src = match read(path) {
        Ok(s) => s,
        Err(c) => return c,
    };
    match candor_proto::run_source(&src) {
        candor_proto::RunResult::Ok(run) => {
            println!("{}", run.ret);
            ExitCode::SUCCESS
        }
        candor_proto::RunResult::Fault(f) => {
            eprintln!("{}", f.to_json());
            ExitCode::from(candor_proto::interp::FAULT_EXIT)
        }
        candor_proto::RunResult::CheckErrors(diags) => {
            for d in &diags {
                println!("{}", d.to_json());
            }
            ExitCode::FAILURE
        }
        candor_proto::RunResult::ParseError(d) => {
            println!("{}", d.to_json());
            ExitCode::FAILURE
        }
    }
}
