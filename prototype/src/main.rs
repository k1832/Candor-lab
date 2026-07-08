//! `candor-proto` CLI.
//!
//! Usage:
//!   candor-proto parse <file>   -- print the parsed AST (Debug), exit 0/1
//!   candor-proto check <file>   -- parse + resolve + Stage 2 check;
//!                                  print JSON diagnostics (one per line),
//!                                  exit 0 if clean, 1 if any error.
//!   candor-proto run <file>     -- parse + check + execute `main`.
//!   candor-proto count <file>   -- parse + check, then emit the frozen Bet 5
//!                                  unit-table counts as JSON (exit 0), or a
//!                                  parse-error JSON (exit 1).
//!   candor-proto migrate <file.cn> [-o <out.cnr>]
//!                               -- P15 migrator (design 0006 §5): parse the
//!                                  throwaway `.cn` file and emit real (`.cnr`)
//!                                  syntax to stdout (or `-o` file).
//!
//! The surface syntax is chosen by file extension (design 0006; spec 01/02):
//!   * `.cnr` -> the real toolchain syntax (borrows/slices as keywords, the
//!     bitwise set, `.*` deref, `?` propagation, `ok`-marked variants, ...).
//!   * `.cn`  -> the throwaway prototype syntax (unchanged).
//!
//! Any other extension is treated as throwaway `.cn`.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match (args.get(1).map(String::as_str), args.get(2)) {
        (Some("parse"), Some(path)) => run_parse(path),
        (Some("check"), Some(path)) => run_check(path),
        (Some("run"), Some(_)) => run_run(&args[2..]),
        (Some("count"), Some(path)) => run_count(path),
        (Some("audit"), Some(path)) => run_audit(path),
        (Some("build"), Some(path)) => run_build(path),
        (Some("compile"), Some(_)) => run_compile(&args[2..]),
        (Some("migrate"), Some(path)) => run_migrate(path, &args[3..]),
        _ => {
            eprintln!("usage: candor-proto (parse|check|run|count) <file>  |  run [--engine=mir] <file>  |  compile <file_or_dir> -o <prog>  |  migrate <file.cn> [-o <out.cnr>]  (.cnr = real syntax, .cn = throwaway)");
            ExitCode::from(2)
        }
    }
}

/// True when the path names a real-syntax (`.cnr`) source file.
fn is_real(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .map(|e| e == "cnr")
        .unwrap_or(false)
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
    let parsed = if is_real(path) {
        candor_proto::parse_source_real(&src)
    } else {
        candor_proto::parse_source(&src)
    };
    match parsed {
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
    if std::path::Path::new(path).is_dir() {
        return report_diags(candor_proto::check_dir(std::path::Path::new(path)));
    }
    let src = match read(path) {
        Ok(s) => s,
        Err(c) => return c,
    };
    let checked = if is_real(path) {
        candor_proto::check_source_real(&src)
    } else {
        candor_proto::check_source(&src)
    };
    match checked {
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

/// Print a diagnostics result (used by both single-file and module-tree check).
fn report_diags(checked: Result<Vec<candor_proto::diag::Diag>, candor_proto::diag::Diag>) -> ExitCode {
    match checked {
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

fn run_run(rest: &[String]) -> ExitCode {
    // `run [--engine=mir|native|tree] <file>` (default engine is the tree-walker).
    let mut engine = "tree";
    let mut path: Option<&str> = None;
    for a in rest {
        match a.as_str() {
            "--engine=mir" => engine = "mir",
            "--engine=native" => engine = "native",
            "--engine=tree" => engine = "tree",
            other if other.starts_with("--engine") => {
                eprintln!("error: unknown --engine (use mir|native|tree)");
                return ExitCode::from(2);
            }
            other => path = Some(other),
        }
    }
    let path = match path {
        Some(p) => p,
        None => {
            eprintln!("usage: candor-proto run [--engine=mir|native] <file>");
            return ExitCode::from(2);
        }
    };
    if engine == "mir" {
        return run_run_mir(path);
    }
    if engine == "native" {
        return run_run_native(path);
    }
    if std::path::Path::new(path).is_dir() {
        return report_run(candor_proto::run_dir(std::path::Path::new(path)));
    }
    let src = match read(path) {
        Ok(s) => s,
        Err(c) => return c,
    };
    let outcome = if is_real(path) {
        candor_proto::run_source_real(&src)
    } else {
        candor_proto::run_source(&src)
    };
    report_run(outcome)
}

/// `run --engine=mir <file>` — the Stage-A precise MIR interpreter.
fn run_run_mir(path: &str) -> ExitCode {
    use candor_proto::MirRunResult;
    let outcome = if std::path::Path::new(path).is_dir() {
        candor_proto::run_dir_mir(std::path::Path::new(path))
    } else {
        let src = match read(path) {
            Ok(s) => s,
            Err(c) => return c,
        };
        if is_real(path) {
            candor_proto::run_source_real_mir(&src)
        } else {
            candor_proto::run_source_mir(&src)
        }
    };
    match outcome {
        MirRunResult::Ok(run) => {
            println!("{}", run.ret);
            ExitCode::SUCCESS
        }
        MirRunResult::Fault(f) => {
            eprintln!("{}", f.to_json());
            ExitCode::from(candor_proto::interp::FAULT_EXIT)
        }
        MirRunResult::CheckErrors(diags) => {
            for d in &diags {
                println!("{}", d.to_json());
            }
            ExitCode::FAILURE
        }
        MirRunResult::ParseError(d) => {
            println!("{}", d.to_json());
            ExitCode::FAILURE
        }
        MirRunResult::Unsupported(what) => {
            eprintln!("error: outside the Stage-A MIR subset: {what}");
            ExitCode::from(3)
        }
    }
}

/// `run --engine=native <file>` — the Stage-B Cranelift JIT engine.
fn run_run_native(path: &str) -> ExitCode {
    use candor_proto::MirRunResult;
    let outcome = if std::path::Path::new(path).is_dir() {
        candor_proto::run_dir_native(std::path::Path::new(path))
    } else {
        let src = match read(path) {
            Ok(s) => s,
            Err(c) => return c,
        };
        if is_real(path) {
            candor_proto::run_source_real_native(&src)
        } else {
            candor_proto::run_source_native(&src)
        }
    };
    match outcome {
        MirRunResult::Ok(run) => {
            println!("{}", run.ret);
            ExitCode::SUCCESS
        }
        MirRunResult::Fault(f) => {
            eprintln!("{}", f.to_json());
            ExitCode::from(candor_proto::interp::FAULT_EXIT)
        }
        MirRunResult::CheckErrors(diags) => {
            for d in &diags {
                println!("{}", d.to_json());
            }
            ExitCode::FAILURE
        }
        MirRunResult::ParseError(d) => {
            println!("{}", d.to_json());
            ExitCode::FAILURE
        }
        MirRunResult::Unsupported(what) => {
            eprintln!("error: outside the native backend subset: {what}");
            ExitCode::from(3)
        }
    }
}

/// Print a run outcome (shared by single-file and module-tree run).
fn report_run(outcome: candor_proto::RunResult) -> ExitCode {
    match outcome {
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

/// `build <dir>` — the Stage-C incremental build (design 0010 §3 / 0008 §2):
/// discover the module tree, compute the DAG, and per module reuse the cached
/// interface artifact or re-analyze it, reporting per-module actions as JSON.
fn run_build(path: &str) -> ExitCode {
    let dir = std::path::Path::new(path);
    if !dir.is_dir() {
        eprintln!("error: `build` takes a module-tree directory");
        return ExitCode::from(2);
    }
    match candor_proto::build::build_dir(dir) {
        Ok(report) => {
            println!("{}", report.to_json());
            if report.ok() {
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

/// `audit <dir_or_file>` — the boundary-module audit surface (design 0011 §6).
fn run_audit(path: &str) -> ExitCode {
    match candor_proto::audit::audit_path(std::path::Path::new(path)) {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(diag) => {
            println!("{}", diag.to_json());
            ExitCode::FAILURE
        }
    }
}

/// `compile <file_or_dir> -o <prog>` — AOT-compile to a linked native executable
/// (design 0010 §5). Lowers the same checked MIR the native engine runs, emits a
/// relocatable object (cranelift-object), and links it with the static runtime
/// via `cc` into a standalone ELF that needs neither the JIT nor `candor-proto`.
fn run_compile(rest: &[String]) -> ExitCode {
    let mut out: Option<&str> = None;
    let mut input: Option<&str> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "-o" => match rest.get(i + 1) {
                Some(o) => {
                    out = Some(o);
                    i += 2;
                }
                None => {
                    eprintln!("error: `-o` requires an output path");
                    return ExitCode::from(2);
                }
            },
            other => {
                input = Some(other);
                i += 1;
            }
        }
    }
    let (input, out) = match (input, out) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            eprintln!("usage: candor-proto compile <file_or_dir> -o <prog>");
            return ExitCode::from(2);
        }
    };
    match candor_proto::compile_path(std::path::Path::new(input), std::path::Path::new(out)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run_count(path: &str) -> ExitCode {
    let src = match read(path) {
        Ok(s) => s,
        Err(c) => return c,
    };
    let counted = if is_real(path) {
        candor_proto::count_source_real(&src)
    } else {
        candor_proto::count_source(&src)
    };
    match counted {
        Ok(counts) => {
            println!("{}", counts.to_json_pretty());
            ExitCode::SUCCESS
        }
        Err(diag) => {
            println!("{}", diag.to_json());
            ExitCode::FAILURE
        }
    }
}

/// `migrate <file.cn> [-o <out.cnr>]` — parse the throwaway front-end and emit
/// real syntax to stdout (default) or to the `-o` file.
fn run_migrate(path: &str, rest: &[String]) -> ExitCode {
    let mut out: Option<&str> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "-o" => match rest.get(i + 1) {
                Some(o) => {
                    out = Some(o);
                    i += 2;
                }
                None => {
                    eprintln!("error: `-o` requires an output path");
                    return ExitCode::from(2);
                }
            },
            other => {
                eprintln!("error: unexpected argument `{other}`");
                return ExitCode::from(2);
            }
        }
    }
    let src = match read(path) {
        Ok(s) => s,
        Err(c) => return c,
    };
    match candor_proto::migrate_source(&src) {
        Ok(real) => match out {
            Some(o) => match std::fs::write(o, real) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("error: cannot write `{o}`: {e}");
                    ExitCode::from(2)
                }
            },
            None => {
                print!("{real}");
                ExitCode::SUCCESS
            }
        },
        Err(diag) => {
            eprintln!("{}", diag.to_json());
            ExitCode::FAILURE
        }
    }
}
