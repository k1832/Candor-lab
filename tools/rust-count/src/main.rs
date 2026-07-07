//! `rust-count <file.rs>` — emit the frozen Bet 5 unit-table counts as JSON.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let path = match args.get(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: rust-count <file.rs>");
            return ExitCode::from(2);
        }
    };
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read `{path}`: {e}");
            return ExitCode::from(2);
        }
    };
    match rust_count::count_str(&src) {
        Ok(counts) => {
            println!("{}", counts.to_json_pretty());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: cannot parse `{path}`: {e}");
            ExitCode::FAILURE
        }
    }
}
