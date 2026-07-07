//! Candor Bet 5 validation prototype — Stage 1 (lexer, AST, parser).
//!
//! Structured as a library crate so later stages (checker, interpreter) add
//! modules alongside these without reshaping the front end. The `candor-proto`
//! binary is a thin CLI over `parse_source`.

pub mod ast;
pub mod check;
pub mod count;
pub mod diag;
pub mod interp;
pub mod lexer;
pub mod parser;
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
