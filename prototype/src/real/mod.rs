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

/// Lex then parse a whole real-syntax source string.
pub fn parse_source(src: &str) -> Result<Program, Diag> {
    let tokens = lexer::lex(src)?;
    parser::parse(tokens)
}

/// Lex then parse a real-syntax source string as a *module* (design 0008):
/// returns the AST, its `use` imports, and per-item visibility flags. Used by
/// the module-tree builder (`crate::modules`); the single-file path uses
/// [`parse_source`] and ignores both side channels.
pub fn parse_module(src: &str) -> Result<(Program, Vec<crate::ast::UseDecl>, Vec<bool>, bool), Diag> {
    let tokens = lexer::lex(src)?;
    parser::parse_module(tokens)
}

/// Parse a real-syntax source string, returning the AST and the file's
/// `boundary`-preamble status (design 0011 audit). Used by `candor audit`.
pub fn parse_with_boundary(src: &str) -> Result<(Program, bool), Diag> {
    let tokens = lexer::lex(src)?;
    parser::parse_with_boundary(tokens)
}
