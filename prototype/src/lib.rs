//! Candor Bet 5 validation prototype — Stage 1 (lexer, AST, parser).
//!
//! Structured as a library crate so later stages (checker, interpreter) add
//! modules alongside these without reshaping the front end. The `candor-proto`
//! binary is a thin CLI over `parse_source`.

pub mod ast;
pub mod check;
pub mod count;
pub mod diag;
pub mod generics;
pub mod interp;
pub mod lexer;
pub mod modules;
pub mod parser;
pub mod real;
pub mod resolve;
pub mod span;
pub mod token;
pub mod types;

use ast::Program;
use diag::Diag;

/// Lex then parse a whole source string. Returns the AST or the first `Diag`.
pub fn parse_source(src: &str) -> Result<Program, Diag> {
    let tokens = lexer::lex(src)?;
    parser::parse(tokens)
}

/// Parse (and run the checker for its side effect / admissibility note), then
/// count the program against the frozen unit table (`docs/BET5_UNIT_TABLE.md`).
/// Counting is purely syntactic over the AST; a parse error is returned as-is.
pub fn count_source(src: &str) -> Result<count::Counts, Diag> {
    let program = parse_source(src)?;
    let _diags = check::check_program(&program);
    Ok(count::count_program(&program, src))
}

/// Parse then run the Stage 2 static checker. Returns all diagnostics (empty
/// on success). A parse error is returned as a single-element vector.
pub fn check_source(src: &str) -> Result<Vec<Diag>, Diag> {
    let program = parse_source(src)?;
    Ok(check::check_program(&program))
}

/// Parse, fully check, then execute a program's `main`. Returns the run outcome,
/// a fault, or a list of check diagnostics (as the error path).
pub enum RunResult {
    Ok(interp::Run),
    Fault(interp::Fault),
    CheckErrors(Vec<Diag>),
    ParseError(Diag),
}

pub fn run_source(src: &str) -> RunResult {
    let program = match parse_source(src) {
        Ok(p) => p,
        Err(d) => return RunResult::ParseError(d),
    };
    let diags = check::check_program(&program);
    if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return RunResult::CheckErrors(diags);
    }
    match interp::run_program(&program) {
        Ok(run) => RunResult::Ok(run),
        Err(f) => RunResult::Fault(f),
    }
}


// ---------------------------------------------------------------------------
// Real (`.cnr`) surface-syntax entry points (design 0006; spec 01/02).
// Same downstream pipeline as the throwaway front-end; only the parser and the
// real-syntax-only surface checks differ.
// ---------------------------------------------------------------------------

/// Parse a real-syntax (`.cnr`) source string into the shared AST.
pub fn parse_source_real(src: &str) -> Result<Program, Diag> {
    real::parse_source(src)
}

/// Parse + count a real-syntax program against the frozen unit table.
pub fn count_source_real(src: &str) -> Result<count::Counts, Diag> {
    let program = real::parse_source(src)?;
    let _diags = check::check_program_real(&program);
    Ok(count::count_program(&program, src))
}

/// Parse + check a real-syntax program (with the real surface rules enabled).
pub fn check_source_real(src: &str) -> Result<Vec<Diag>, Diag> {
    let program = real::parse_source(src)?;
    Ok(check::check_program_real(&program))
}

/// Parse, check, then run a real-syntax program's `main`.
pub fn run_source_real(src: &str) -> RunResult {
    let program = match real::parse_source(src) {
        Ok(p) => p,
        Err(d) => return RunResult::ParseError(d),
    };
    if generics::is_generic_program(&program) {
        return run_generic(&program);
    }
    let diags = check::check_program_real(&program);
    if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return RunResult::CheckErrors(diags);
    }
    match interp::run_program(&program) {
        Ok(run) => RunResult::Ok(run),
        Err(f) => RunResult::Fault(f),
    }
}

/// Check a generic program, then monomorphize it and run the concrete result
/// (design 0007 §5.2: the interpreter executes the instantiated AST, trusting the
/// definition-site check — it re-runs no analysis tier).
fn run_generic(program: &ast::Program) -> RunResult {
    let (diags, insts, shapes) = check::check_generic_program(program, true);
    if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return RunResult::CheckErrors(diags);
    }
    let mono = generics::monomorphize(program, &insts, &shapes);
    if mono.diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return RunResult::CheckErrors(mono.diags);
    }
    match interp::run_program(&mono.program) {
        Ok(run) => RunResult::Ok(run),
        Err(f) => RunResult::Fault(f),
    }
}

// ---------------------------------------------------------------------------
// Module-tree (`.cnr` directory) entry points (design 0008 stage 1). A directory
// argument is a module tree; it is discovered, `use`-resolved, visibility- and
// cycle-checked, then merged into one program fed to the shared real-syntax
// pipeline. Single files keep their existing behavior.
// ---------------------------------------------------------------------------

/// Check a module tree rooted at `dir`. Returns module-layer diagnostics
/// (imports, visibility, cycles) followed by the shared checker's diagnostics,
/// or a hard I/O/parse error as a single `Diag`.
pub fn check_dir(dir: &std::path::Path) -> Result<Vec<Diag>, Diag> {
    let build = modules::build_tree(dir)?;
    let mut diags = build.diags;
    diags.extend(check::check_program_real(&build.program));
    Ok(diags)
}

/// Build, check, then run a module tree's `main` (in the root `main.cnr`).
pub fn run_dir(dir: &std::path::Path) -> RunResult {
    let build = match modules::build_tree(dir) {
        Ok(b) => b,
        Err(d) => return RunResult::ParseError(d),
    };
    let mut diags = build.diags;
    if generics::is_generic_program(&build.program) {
        if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
            return RunResult::CheckErrors(diags);
        }
        return run_generic(&build.program);
    }
    diags.extend(check::check_program_real(&build.program));
    if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return RunResult::CheckErrors(diags);
    }
    match interp::run_program(&build.program) {
        Ok(run) => RunResult::Ok(run),
        Err(f) => RunResult::Fault(f),
    }
}

// ---------------------------------------------------------------------------
// P15 migrator (design 0006 §5): parse throwaway (`.cn`) syntax with the
// existing front-end, then re-emit the shared AST in real (`.cnr`) syntax.
// Semantic fidelity is by construction — the emitter only re-spells the AST the
// throwaway parser produced. See `real::emit` for the mechanical/author-assisted
// row handling.
// ---------------------------------------------------------------------------

/// Migrate a throwaway (`.cn`) source string to real (`.cnr`) surface syntax.
/// Returns the emitted program text, or the throwaway front-end's parse `Diag`.
pub fn migrate_source(src: &str) -> Result<String, Diag> {
    let program = parse_source(src)?;
    Ok(real::emit::emit_program(&program))
}
