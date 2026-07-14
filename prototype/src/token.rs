//! Token kinds for the throwaway Candor prototype grammar.
//!
//! Keywords are "ugly-but-unambiguous" (design 0001 §0). `alloc` is deliberately
//! *not* a hard keyword: design 0001 §6.1 names a struct field `alloc`, so the
//! effect marker is recognized positionally (a contextual keyword), never as a
//! reserved word. Every other disambiguator is a hard keyword below.

use crate::span::Span;

/// Fixed-width scalar types (design 0001 §8.1). Also usable as integer-literal
/// suffixes (integer types only).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ScalarTy {
    I8,
    I16,
    I32,
    I64,
    Isize,
    U8,
    U16,
    U32,
    U64,
    Usize,
    Bool,
    Unit,
    /// IEEE-754 binary64 (design 0016). NOT an integer, so not a valid literal
    /// suffix, and exempt from the arithmetic regime system.
    F64,
    /// IEEE-754 binary32 (design 0016). Spelled only via a float-form `f32`-suffixed
    /// literal (`1.5f32`) or `conv f32 (..)`; like `f64`, not an integer and exempt
    /// from the arithmetic regime system.
    F32,
}

impl ScalarTy {
    /// Is this a signed/unsigned integer type (thus a valid literal suffix)?
    /// `f64` is a scalar but not an integer (design 0016).
    pub fn is_integer(self) -> bool {
        matches!(
            self,
            ScalarTy::I8
                | ScalarTy::I16
                | ScalarTy::I32
                | ScalarTy::I64
                | ScalarTy::Isize
                | ScalarTy::U8
                | ScalarTy::U16
                | ScalarTy::U32
                | ScalarTy::U64
                | ScalarTy::Usize
        )
    }

    /// Is this a floating-point type (`f32`/`f64`; design 0016)?
    pub fn is_float(self) -> bool {
        matches!(self, ScalarTy::F64 | ScalarTy::F32)
    }
}

/// Hard keywords. `alloc` is intentionally absent (contextual, see module docs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kw {
    // parametric / constructor type keywords
    Slice,
    SliceMut,
    RawPtr,
    Box,
    BoxResult,
    Borrow,
    BorrowMut,
    // items
    Struct,
    Enum,
    Fn,
    Static,
    Copy,
    Drop,
    SelfKw,
    // parameter modes
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
    Case,
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
    // valve / regime blocks
    Unsafe,
    Wrapping,
    Saturating,
    // prefix operator keywords
    Deref,
    Clone,
    Conv,
    Bitcast,
    // bracketed-type-arg intrinsics
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
pub enum TokKind {
    // literals & names
    Int { value: u64, suffix: Option<ScalarTy> },
    Str(String),
    Bytes(String),
    Ident(String),
    Scalar(ScalarTy),
    Kw(Kw),
    // punctuation & operators
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Dot,
    Semi,
    Colon,
    ColonColon, // `::` — reserved EXCLUSIVELY for enum-variant reference (NN#13)
    Arrow,      // `->`
    FatArrow,   // `=>`
    Eq,
    EqEq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    AmpAmp,
    PipePipe,
    Bang,
    Eof,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokKind,
    pub span: Span,
}

/// Map an identifier spelling to a scalar type keyword, if it is one.
pub fn scalar_from_str(s: &str) -> Option<ScalarTy> {
    Some(match s {
        "i8" => ScalarTy::I8,
        "i16" => ScalarTy::I16,
        "i32" => ScalarTy::I32,
        "i64" => ScalarTy::I64,
        "isize" => ScalarTy::Isize,
        "u8" => ScalarTy::U8,
        "u16" => ScalarTy::U16,
        "u32" => ScalarTy::U32,
        "u64" => ScalarTy::U64,
        "usize" => ScalarTy::Usize,
        "f64" => ScalarTy::F64,
        "f32" => ScalarTy::F32,
        "bool" => ScalarTy::Bool,
        "unit" => ScalarTy::Unit,
        _ => return None,
    })
}

/// Map an identifier spelling to a hard keyword, if it is one.
pub fn keyword_from_str(s: &str) -> Option<Kw> {
    Some(match s {
        "slice" => Kw::Slice,
        "slice_mut" => Kw::SliceMut,
        "rawptr" => Kw::RawPtr,
        "Box" => Kw::Box,
        "BoxResult" => Kw::BoxResult,
        "borrow" => Kw::Borrow,
        "borrow_mut" => Kw::BorrowMut,
        "struct" => Kw::Struct,
        "enum" => Kw::Enum,
        "fn" => Kw::Fn,
        "static" => Kw::Static,
        "copy" => Kw::Copy,
        "drop" => Kw::Drop,
        "self" => Kw::SelfKw,
        "take" => Kw::Take,
        "read" => Kw::Read,
        "write" => Kw::Write,
        "out" => Kw::Out,
        "let" => Kw::Let,
        "mut" => Kw::Mut,
        "if" => Kw::If,
        "else" => Kw::Else,
        "match" => Kw::Match,
        "case" => Kw::Case,
        "loop" => Kw::Loop,
        "while" => Kw::While,
        "break" => Kw::Break,
        "continue" => Kw::Continue,
        "return" => Kw::Return,
        "requires" => Kw::Requires,
        "ensures" => Kw::Ensures,
        "assert" => Kw::Assert,
        "panic" => Kw::Panic,
        "result" => Kw::Result,
        "unsafe" => Kw::Unsafe,
        "wrapping" => Kw::Wrapping,
        "saturating" => Kw::Saturating,
        "deref" => Kw::Deref,
        "clone" => Kw::Clone,
        "conv" => Kw::Conv,
        "bitcast" => Kw::Bitcast,
        "cast_ptr" => Kw::CastPtr,
        "addr_to_ptr" => Kw::AddrToPtr,
        "ptr_null" => Kw::PtrNull,
        "offsetof" => Kw::Offsetof,
        "field_ptr" => Kw::FieldPtr,
        "sizeof" => Kw::Sizeof,
        "alignof" => Kw::Alignof,
        "true" => Kw::True,
        "false" => Kw::False,
        _ => return None,
    })
}
