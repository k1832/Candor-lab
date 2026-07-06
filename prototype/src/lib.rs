//! Candor Bet 5 validation prototype — Stage 1 (lexer, AST, parser).
//!
//! Structured as a library crate so later stages (checker, interpreter) add
//! modules alongside these without reshaping the front end. The `candor-proto`
//! binary is a thin CLI over `parse_source`.

pub mod ast;
pub mod diag;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod token;

use ast::Program;
use diag::Diag;

/// Lex then parse a whole source string. Returns the AST or the first `Diag`.
pub fn parse_source(src: &str) -> Result<Program, Diag> {
    let tokens = lexer::lex(src)?;
    parser::parse(tokens)
}
