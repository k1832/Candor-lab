//! Token inventory for the **real** Candor surface syntax (design 0006; spec
//! chapter 01). Distinct from the throwaway `token.rs`: borrows/slices are
//! keywords, `&`/`|`/`^`/`~`/`<<`/`>>` are the bitwise set, `.*` is a single
//! postfix-deref token, `?` is postfix propagation, `out` is a hard keyword, and
//! `alloc`/`ok` are contextual (they lex as identifiers). The retired spellings
//! `slice`/`slice_mut`/`borrow`/`borrow_mut`/`deref`/`case` are NOT keywords here
//! (spec 01 §2.6) — they lex as ordinary identifiers.

use crate::span::Span;
use crate::token::ScalarTy;

/// Hard keywords of the real grammar (spec 01 §2.1–§2.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RKw {
    // type-constructor keywords
    RawPtr,
    Box,
    BoxResult,
    // items
    Struct,
    Enum,
    Fn,
    Static,
    Copy,
    Drop,
    SelfKw,
    // modes
    Take,
    Read,
    Write,
    Out,
    // statements / control
    Let,
    Mut,
    If,
    Else,
    Match,
    Loop,
    While,
    Break,
    Continue,
    Return,
    // contracts
    Requires,
    Ensures,
    Assert,
    Panic,
    Result,
    // valve / regimes / ops
    Unsafe,
    Wrapping,
    Saturating,
    Conv,
    Bitcast,
    Clone,
    // bracketed-type-arg / call intrinsics
    CastPtr,
    AddrToPtr,
    PtrNull,
    Offsetof,
    FieldPtr,
    Sizeof,
    Alignof,
    // literals
    True,
    False,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RTok {
    Int { value: u64, suffix: Option<ScalarTy> },
    /// A float literal, carrying its IEEE-754 bit pattern and float type
    /// (`f64` by default, or `f32` for an `f32`-suffixed float form; design 0016).
    Float { bits: u64, ty: crate::token::ScalarTy },
    Str(String),
    Bytes(String),
    Ident(String),
    Scalar(ScalarTy),
    Kw(RKw),
    // grouping & punctuation
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Dot,
    DotStar, // `.*` postfix deref (spec 01 §5.4)
    DotDot,   // `..` half-open range pattern
    DotDotEq, // `..=` inclusive range pattern
    Semi,
    Colon,
    ColonColon,
    Arrow,    // `->`
    FatArrow, // `=>`
    Question, // `?` postfix propagation
    // comparison / assignment
    Eq,
    EqEq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // arithmetic
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    // bitwise (design 0006 §2.4)
    Amp,
    Pipe,
    Caret,
    Tilde,
    Shl,
    Shr,
    // logical
    AmpAmp,
    PipePipe,
    Bang,
    Eof,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RToken {
    pub kind: RTok,
    pub span: Span,
}

/// Map an identifier spelling to a hard keyword of the real grammar, if it is
/// one. `alloc` and `ok` are contextual (spec 01 §2.5) and the retired spellings
/// (`slice`, `borrow`, `deref`, `case`, …) are absent (§2.6): all lex as IDENT.
pub fn real_keyword_from_str(s: &str) -> Option<RKw> {
    Some(match s {
        "rawptr" => RKw::RawPtr,
        "Box" => RKw::Box,
        "BoxResult" => RKw::BoxResult,
        "struct" => RKw::Struct,
        "enum" => RKw::Enum,
        "fn" => RKw::Fn,
        "static" => RKw::Static,
        "copy" => RKw::Copy,
        "drop" => RKw::Drop,
        "self" => RKw::SelfKw,
        "take" => RKw::Take,
        "read" => RKw::Read,
        "write" => RKw::Write,
        "out" => RKw::Out,
        "let" => RKw::Let,
        "mut" => RKw::Mut,
        "if" => RKw::If,
        "else" => RKw::Else,
        "match" => RKw::Match,
        "loop" => RKw::Loop,
        "while" => RKw::While,
        "break" => RKw::Break,
        "continue" => RKw::Continue,
        "return" => RKw::Return,
        "requires" => RKw::Requires,
        "ensures" => RKw::Ensures,
        "assert" => RKw::Assert,
        "panic" => RKw::Panic,
        "result" => RKw::Result,
        "unsafe" => RKw::Unsafe,
        "wrapping" => RKw::Wrapping,
        "saturating" => RKw::Saturating,
        "conv" => RKw::Conv,
        "bitcast" => RKw::Bitcast,
        "clone" => RKw::Clone,
        "cast_ptr" => RKw::CastPtr,
        "addr_to_ptr" => RKw::AddrToPtr,
        "ptr_null" => RKw::PtrNull,
        "offsetof" => RKw::Offsetof,
        "field_ptr" => RKw::FieldPtr,
        "sizeof" => RKw::Sizeof,
        "alignof" => RKw::Alignof,
        "true" => RKw::True,
        "false" => RKw::False,
        _ => return None,
    })
}
