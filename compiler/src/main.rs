//! `candor` CLI.
//!
//! Usage:
//!   candor parse <file>   -- print the parsed AST (Debug), exit 0/1
//!   candor check <file>   -- parse + resolve + Stage 2 check;
//!                                  print JSON diagnostics (one per line),
//!                                  exit 0 if clean, 1 if any error.
//!   candor run <file>     -- parse + check + execute `main`.
//!   candor count <file>   -- parse + check, then emit the frozen Bet 5
//!                                  unit-table counts as JSON (exit 0), or a
//!                                  parse-error JSON (exit 1).
//!   candor manifest <dir_or_file>
//!                               -- parse + validate a package manifest
//!                                  (candor.toml; design 0017 §2) and print it
//!                                  as JSON. Exit 0 on a valid manifest, 1 on a
//!                                  manifest error or a manifest-less directory.
//!   candor migrate <file.cn> [-o <out.cnr>]
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
        (Some("manifest"), Some(path)) => run_manifest(path),
        (Some("compile"), Some(_)) => run_compile(&args[2..]),
        (Some("migrate"), Some(path)) => run_migrate(path, &args[3..]),
        (Some("fmt"), Some(path)) => run_fmt(path, &args[3..]),
        (Some("--version" | "-V" | "version"), _) => {
            println!("candor {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        (Some("--help" | "-h" | "help"), _) => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

/// The CLI usage text, shared by `--help` (stdout, exit 0) and the unknown-command
/// error path (stderr, exit 2).
const USAGE: &str = "usage: candor (parse|check|run|count|audit|build|manifest) <file>  |  run [--engine=mir] <file>  |  compile <file_or_dir> -o <prog> [--release] [--freestanding] [--linker=<name>]  |  migrate <file.cn> [-o <out.cnr>]  |  fmt <file_or_dir.cnr> [--check|--stdout]  |  --version  |  --help   (.cnr = real syntax, .cn = throwaway)";

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
        candor::parse_source_real(&src)
    } else {
        candor::parse_source(&src)
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
        return report_diags(candor::check_dir(std::path::Path::new(path)));
    }
    let src = match read(path) {
        Ok(s) => s,
        Err(c) => return c,
    };
    let checked = if is_real(path) {
        candor::check_source_real(&src)
    } else {
        candor::check_source(&src)
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
fn report_diags(checked: Result<Vec<candor::diag::Diag>, candor::diag::Diag>) -> ExitCode {
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
            eprintln!("usage: candor run [--engine=mir|native] <file>");
            return ExitCode::from(2);
        }
    };
    // The CLI does REAL host I/O: register the production foreign runtime (fd 1/2 ->
    // real stdout/stderr, fd 0 -> real stdin, open/listdir as-given) before running.
    candor::foreign_io::register_std_io_production();
    if engine == "mir" {
        return run_run_mir(path);
    }
    if engine == "native" {
        return run_run_native(path);
    }
    if std::path::Path::new(path).is_dir() {
        return report_run(candor::run_dir(std::path::Path::new(path)));
    }
    let src = match read(path) {
        Ok(s) => s,
        Err(c) => return c,
    };
    let outcome = if is_real(path) {
        candor::run_source_real(&src)
    } else {
        candor::run_source(&src)
    };
    report_run(outcome)
}

/// `run --engine=mir <file>` — the Stage-A precise MIR interpreter.
fn run_run_mir(path: &str) -> ExitCode {
    use candor::MirRunResult;
    candor::foreign_io::register_std_io_production();
    let outcome = if std::path::Path::new(path).is_dir() {
        candor::run_dir_mir(std::path::Path::new(path))
    } else {
        let src = match read(path) {
            Ok(s) => s,
            Err(c) => return c,
        };
        if is_real(path) {
            candor::run_source_real_mir(&src)
        } else {
            candor::run_source_mir(&src)
        }
    };
    match outcome {
        MirRunResult::Ok(run) => {
            println!("{}", run.ret);
            ExitCode::SUCCESS
        }
        MirRunResult::Fault(f) => {
            eprintln!("{}", f.to_json());
            ExitCode::from(candor::interp::FAULT_EXIT)
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
    use candor::MirRunResult;
    let outcome = if std::path::Path::new(path).is_dir() {
        candor::run_dir_native(std::path::Path::new(path))
    } else {
        let src = match read(path) {
            Ok(s) => s,
            Err(c) => return c,
        };
        if is_real(path) {
            candor::run_source_real_native(&src)
        } else {
            candor::run_source_native(&src)
        }
    };
    match outcome {
        MirRunResult::Ok(run) => {
            println!("{}", run.ret);
            ExitCode::SUCCESS
        }
        MirRunResult::Fault(f) => {
            eprintln!("{}", f.to_json());
            ExitCode::from(candor::interp::FAULT_EXIT)
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
fn report_run(outcome: candor::RunResult) -> ExitCode {
    match outcome {
        candor::RunResult::Ok(run) => {
            // trace() output is the program's observable θ; show it on the tree-walking
            // `run` path so a newcomer sees what their program emitted, then the return
            // value. Suppressed when CANDOR_QUIET is set (test harnesses parse bare ret).
            if std::env::var_os("CANDOR_QUIET").is_none() {
                for t in &run.trace {
                    println!("trace {t}");
                }
            }
            println!("{}", run.ret);
            ExitCode::SUCCESS
        }
        candor::RunResult::Fault(f) => {
            eprintln!("{}", f.to_json());
            ExitCode::from(candor::interp::FAULT_EXIT)
        }
        candor::RunResult::CheckErrors(diags) => {
            for d in &diags {
                println!("{}", d.to_json());
            }
            ExitCode::FAILURE
        }
        candor::RunResult::ParseError(d) => {
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
    match candor::build::build_dir(dir) {
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
    match candor::audit::audit_path(std::path::Path::new(path)) {
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

/// `compile [--release] [--freestanding] <file_or_dir> -o <prog>` — AOT-compile
/// to a linked native executable (design 0010 §5). The default lowers the checked
/// MIR through the Cranelift backend (fast compile, no global optimization — the
/// dev/debug build) and links it via `cc` into a standalone ELF. `--release`
/// selects the LLVM `-O2` backend instead (optimized native — the release build;
/// `--backend=llvm` is a synonym). `--freestanding` links the no-libc runtime
/// (`-nostdlib -static -no-pie`), the NN#6 proof artifact — no JIT, no libc.
/// `--linker=<name>` selects an alternate linker (e.g. `mold`, `lld`, `gold`,
/// `bfd`), passed to `cc`/`clang` as `-fuse-ld=<name>` — opt-in and explicit so a
/// build's linker stays a deterministic toolchain choice (NN#16 reproducibility),
/// never "whatever happens to be installed".
fn run_compile(rest: &[String]) -> ExitCode {
    let mut out: Option<&str> = None;
    let mut input: Option<&str> = None;
    let mut freestanding = false;
    let mut llvm = false;
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
            "--freestanding" => {
                freestanding = true;
                i += 1;
            }
            // `--release` is the blessed name for the optimized LLVM backend;
            // `--backend=llvm` remains as a synonym.
            "--release" | "--backend=llvm" => {
                llvm = true;
                i += 1;
            }
            // `--linker=<name>` selects the linker (e.g. mold, lld, gold, bfd),
            // passed to cc/clang as `-fuse-ld=<name>`. Opt-in and explicit so the
            // build stays reproducible; sets CANDOR_LINKER, which the backends read.
            s if s.starts_with("--linker=") => {
                let name = &s["--linker=".len()..];
                if name.is_empty() {
                    eprintln!("error: `--linker=` requires a linker name (e.g. --linker=mold)");
                    return ExitCode::from(2);
                }
                std::env::set_var("CANDOR_LINKER", name);
                i += 1;
            }
            other => {
                input = Some(other);
                i += 1;
            }
        }
    }
    let (input, out) = match (input, out) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            eprintln!("usage: candor compile [--release] [--freestanding] [--linker=<name>] <file_or_dir> -o <prog>");
            return ExitCode::from(2);
        }
    };
    let (inp, outp) = (std::path::Path::new(input), std::path::Path::new(out));
    let r = if llvm {
        candor::compile_path_llvm(inp, outp)
    } else if freestanding {
        candor::compile_path_freestanding(inp, outp)
    } else {
        candor::compile_path(inp, outp)
    };
    match r {
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
        candor::count_source_real(&src)
    } else {
        candor::count_source(&src)
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
/// `candor manifest <dir_or_file>` (design 0017 §2): parse + validate a package
/// manifest and print it as JSON. A directory is read as `<dir>/candor.toml`
/// (its absence prints a note and exits 1); a file path is parsed directly.
fn run_manifest(path: &str) -> ExitCode {
    let p = std::path::Path::new(path);
    let parsed = if p.is_dir() {
        match candor::manifest::load_manifest(p) {
            Ok(Some(m)) => Ok(m),
            Ok(None) => {
                eprintln!("error: no `candor.toml` in `{path}` (a manifest-less directory is the degenerate package)");
                return ExitCode::FAILURE;
            }
            Err(e) => Err(e),
        }
    } else {
        let src = match read(path) {
            Ok(s) => s,
            Err(c) => return c,
        };
        candor::manifest::parse_manifest(&src)
    };
    match parsed {
        Ok(manifest) => {
            println!("{}", serde_json::to_string_pretty(&manifest).expect("manifest is serializable"));
            ExitCode::SUCCESS
        }
        Err(e) => {
            println!("{}", serde_json::to_string(&e).expect("manifest error is serializable"));
            ExitCode::FAILURE
        }
    }
}

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
    match candor::migrate_source(&src) {
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

/// `fmt <file_or_dir> [--check|--stdout]` — the blessed canonical formatter
/// (P16/NN#11; spec 02 §9). Formats every `.cnr` file (recursively for a
/// directory). Default: rewrite each file in place. `--check`: write nothing and
/// exit nonzero if any file is not already canonical (CI gate). `--stdout`: print
/// the formatted form of a single file, writing nothing.
fn run_fmt(path: &str, rest: &[String]) -> ExitCode {
    let mut check = false;
    let mut stdout = false;
    for a in rest {
        match a.as_str() {
            "--check" => check = true,
            "--stdout" => stdout = true,
            other => {
                eprintln!("error: unexpected argument `{other}` (use --check or --stdout)");
                return ExitCode::from(2);
            }
        }
    }
    let root = std::path::Path::new(path);
    let mut files = Vec::new();
    if root.is_dir() {
        collect_cnr(root, &mut files);
        files.sort();
    } else {
        if !is_real(path) {
            eprintln!("error: `fmt` only formats `.cnr` files");
            return ExitCode::from(2);
        }
        files.push(root.to_path_buf());
    }
    if files.is_empty() {
        eprintln!("error: no `.cnr` files at `{path}`");
        return ExitCode::from(2);
    }
    let mut any_diff = false;
    let mut had_error = false;
    for f in &files {
        let src = match std::fs::read_to_string(f) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: cannot read `{}`: {e}", f.display());
                had_error = true;
                continue;
            }
        };
        match candor::format_source_real(&src) {
            Ok(out) => {
                if stdout {
                    print!("{out}");
                    continue;
                }
                if out != src {
                    any_diff = true;
                    if check {
                        println!("{}", f.display());
                    } else if let Err(e) = std::fs::write(f, &out) {
                        eprintln!("error: cannot write `{}`: {e}", f.display());
                        had_error = true;
                    }
                }
            }
            Err(diag) => {
                eprintln!("{}: {}", f.display(), diag.to_json());
                had_error = true;
            }
        }
    }
    if had_error {
        return ExitCode::FAILURE;
    }
    if check && any_diff {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// Recursively collect `.cnr` files under `dir`.
fn collect_cnr(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_cnr(&p, out);
        } else if p.extension().map(|e| e == "cnr").unwrap_or(false) {
            out.push(p);
        }
    }
}
