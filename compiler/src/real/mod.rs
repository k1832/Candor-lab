//! The real (`.cnr`) surface-syntax front-end (design 0006; spec chapters
//! 01/02). A parallel lexer + parser that produces the SAME [`crate::ast`] the
//! throwaway (`.cn`) front-end targets, feeding the identical checker /
//! interpreter / counter pipeline. Only genuinely-new semantics reach the AST as
//! new nodes (`?`, bitwise ops, negative-literal fold, `ok`-marked variants); the
//! rest is desugared losslessly in the parser (`.*` -> deref, `read`/`write`
//! borrows and `[T]`/`write [T]` slices -> the existing borrow/slice types).
//!
//! The CLI selects this front-end by file extension: `.cnr` = real syntax,
//! `.cn` = throwaway (unchanged). The checker runs its real-syntax-only surface
//! rules (literal over-range, constant-`conv` loss, write-through-borrow needs
//! `.*`) for programs parsed here (see [`crate::check::check_program_real`]).
//!
//! [`emit`] is the reverse direction: the P15 migrator's AST -> real-syntax
//! pretty-printer (design 0006 §5), used by `candor migrate`.

pub mod emit;
pub mod fmt;
pub mod lexer;
pub mod parser;
pub mod token;

use crate::ast::Program;
use crate::diag::Diag;
use crate::manifest::Edition;

/// Lex then parse a whole real-syntax source string (default [`Edition::E2026`]).
pub fn parse_source(src: &str) -> Result<Program, Diag> {
    parse_source_in(src, Edition::E2026)
}

/// Lex then parse a real-syntax source string under a specific surface
/// [`Edition`] (1.0-gate item 1). Only the lexer consults the edition; the parser
/// is edition-agnostic.
pub fn parse_source_in(src: &str, edition: Edition) -> Result<Program, Diag> {
    let tokens = lexer::lex_in(src, edition)?;
    parser::parse(tokens)
}

/// Lex then parse a real-syntax source string as a *module* (design 0008):
/// returns the AST, its `use` imports, and per-item visibility flags. Used by
/// the module-tree builder (`crate::modules`); the single-file path uses
/// [`parse_source`] and ignores both side channels.
pub fn parse_module(src: &str) -> Result<(Program, Vec<crate::ast::UseDecl>, Vec<bool>, bool), Diag> {
    parse_module_in(src, Edition::E2026)
}

/// As [`parse_module`], under a specific surface [`Edition`] (1.0-gate item 1).
/// The module-tree builder passes each package's manifest edition here.
pub fn parse_module_in(
    src: &str,
    edition: Edition,
) -> Result<(Program, Vec<crate::ast::UseDecl>, Vec<bool>, bool), Diag> {
    let tokens = lexer::lex_in(src, edition)?;
    parser::parse_module(tokens)
}

/// Parse a real-syntax source string, returning the AST and the file's
/// `boundary`-preamble status (design 0011 audit). Used by `candor audit`.
pub fn parse_with_boundary(src: &str) -> Result<(Program, bool), Diag> {
    parse_with_boundary_in(src, Edition::E2026)
}

/// As [`parse_with_boundary`], under a specific surface [`Edition`] (1.0-gate
/// item 1). The audit walk passes each package's manifest edition here.
pub fn parse_with_boundary_in(src: &str, edition: Edition) -> Result<(Program, bool), Diag> {
    let tokens = lexer::lex_in(src, edition)?;
    parser::parse_with_boundary(tokens)
}
