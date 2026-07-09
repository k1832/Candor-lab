//! Hand-written lexer. Produces a `Vec<Token>` (terminated by `Eof`) or a
//! structured `Diag` on the first bad input. It never panics on bad input.

use crate::diag::Diag;
use crate::span::Span;
use crate::token::{keyword_from_str, scalar_from_str, TokKind, Token};

pub struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Lexer<'a> {
        Lexer {
            src: src.as_bytes(),
            pos: 0,
        }
    }

    /// Tokenize the whole input. On success the final token is always `Eof`.
    pub fn tokenize(mut self) -> Result<Vec<Token>, Diag> {
        let mut out = Vec::new();
        loop {
            self.skip_trivia()?;
            if self.pos >= self.src.len() {
                out.push(Token {
                    kind: TokKind::Eof,
                    span: Span::point(self.pos),
                });
                return Ok(out);
            }
            out.push(self.next_token()?);
        }
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn peek2(&self) -> Option<u8> {
        self.src.get(self.pos + 1).copied()
    }

    /// Skip whitespace, line comments (`// ...`) and block comments (`/* ... */`).
    fn skip_trivia(&mut self) -> Result<(), Diag> {
        loop {
            match self.peek() {
                Some(b) if b.is_ascii_whitespace() => self.pos += 1,
                Some(b'/') if self.peek2() == Some(b'/') => {
                    while let Some(b) = self.peek() {
                        if b == b'\n' {
                            break;
                        }
                        self.pos += 1;
                    }
                }
                Some(b'/') if self.peek2() == Some(b'*') => {
                    let start = self.pos;
                    self.pos += 2;
                    loop {
                        match self.peek() {
                            None => {
                                return Err(Diag::error(
                                    "L0004",
                                    "unterminated block comment",
                                    Span::new(start, self.pos),
                                ))
                            }
                            Some(b'*') if self.peek2() == Some(b'/') => {
                                self.pos += 2;
                                break;
                            }
                            Some(_) => self.pos += 1,
                        }
                    }
                }
                _ => return Ok(()),
            }
        }
    }

    fn next_token(&mut self) -> Result<Token, Diag> {
        let start = self.pos;
        let b = self.peek().expect("caller guarantees a byte");

        // identifiers / keywords / scalar types
        // `b"..."` byte-string literal (design 0013): a `b` IMMEDIATELY followed
        // by `"` opens a byte string; a `b` followed by anything else is an
        // identifier (one char of lookahead, NN#13).
        if b == b'b' && self.peek2() == Some(b'"') {
            self.pos += 1; // consume `b`
            return self.lex_string(start, true);
        }
        if is_ident_start(b) {
            return Ok(self.lex_ident(start));
        }
        // numbers
        if b.is_ascii_digit() {
            return self.lex_number(start);
        }
        // strings
        if b == b'"' {
            return self.lex_string(start, false);
        }

        // punctuation & operators
        let two = (b, self.peek2());
        let (kind, len) = match two {
            (b':', Some(b':')) => (TokKind::ColonColon, 2),
            (b'-', Some(b'>')) => (TokKind::Arrow, 2),
            (b'=', Some(b'>')) => (TokKind::FatArrow, 2),
            (b'=', Some(b'=')) => (TokKind::EqEq, 2),
            (b'!', Some(b'=')) => (TokKind::Ne, 2),
            (b'<', Some(b'=')) => (TokKind::Le, 2),
            (b'>', Some(b'=')) => (TokKind::Ge, 2),
            (b'&', Some(b'&')) => (TokKind::AmpAmp, 2),
            (b'|', Some(b'|')) => (TokKind::PipePipe, 2),
            (b'(', _) => (TokKind::LParen, 1),
            (b')', _) => (TokKind::RParen, 1),
            (b'{', _) => (TokKind::LBrace, 1),
            (b'}', _) => (TokKind::RBrace, 1),
            (b'[', _) => (TokKind::LBracket, 1),
            (b']', _) => (TokKind::RBracket, 1),
            (b',', _) => (TokKind::Comma, 1),
            (b'.', _) => (TokKind::Dot, 1),
            (b';', _) => (TokKind::Semi, 1),
            (b':', _) => (TokKind::Colon, 1),
            (b'=', _) => (TokKind::Eq, 1),
            (b'<', _) => (TokKind::Lt, 1),
            (b'>', _) => (TokKind::Gt, 1),
            (b'+', _) => (TokKind::Plus, 1),
            (b'-', _) => (TokKind::Minus, 1),
            (b'*', _) => (TokKind::Star, 1),
            (b'/', _) => (TokKind::Slash, 1),
            (b'%', _) => (TokKind::Percent, 1),
            (b'!', _) => (TokKind::Bang, 1),
            _ => {
                return Err(Diag::error(
                    "L0001",
                    format!("unexpected character `{}`", b as char),
                    Span::new(start, start + 1),
                ))
            }
        };
        self.pos += len;
        Ok(Token {
            kind,
            span: Span::new(start, self.pos),
        })
    }

    fn lex_ident(&mut self, start: usize) -> Token {
        while let Some(b) = self.peek() {
            if is_ident_continue(b) {
                self.pos += 1;
            } else {
                break;
            }
        }
        let text = std::str::from_utf8(&self.src[start..self.pos]).expect("ascii ident");
        let kind = if let Some(kw) = keyword_from_str(text) {
            TokKind::Kw(kw)
        } else if let Some(sc) = scalar_from_str(text) {
            TokKind::Scalar(sc)
        } else {
            TokKind::Ident(text.to_string())
        };
        Token {
            kind,
            span: Span::new(start, self.pos),
        }
    }

    fn lex_number(&mut self, start: usize) -> Result<Token, Diag> {
        let radix;
        if self.peek() == Some(b'0') && matches!(self.peek2(), Some(b'x') | Some(b'X')) {
            self.pos += 2;
            radix = 16;
            let digit_start = self.pos;
            while let Some(b) = self.peek() {
                if b.is_ascii_hexdigit() {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            if self.pos == digit_start {
                return Err(Diag::error(
                    "L0002",
                    "hex literal `0x` has no digits",
                    Span::new(start, self.pos),
                ));
            }
        } else {
            radix = 10;
            while let Some(b) = self.peek() {
                if b.is_ascii_digit() {
                    self.pos += 1;
                } else {
                    break;
                }
            }
        }
        let digits_end = self.pos;
        let digit_slice = if radix == 16 {
            &self.src[start + 2..digits_end]
        } else {
            &self.src[start..digits_end]
        };
        let digit_str = std::str::from_utf8(digit_slice).expect("ascii digits");
        let value = u64::from_str_radix(digit_str, radix).map_err(|_| {
            Diag::error(
                "L0003",
                "integer literal out of range for u64",
                Span::new(start, digits_end),
            )
        })?;

        // optional type suffix directly adjacent to the digits, e.g. `42u8`
        let mut suffix = None;
        if let Some(b) = self.peek() {
            if is_ident_start(b) {
                let suf_start = self.pos;
                while let Some(c) = self.peek() {
                    if is_ident_continue(c) {
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                let suf = std::str::from_utf8(&self.src[suf_start..self.pos]).expect("ascii");
                match scalar_from_str(suf) {
                    Some(sc) if sc.is_integer() => suffix = Some(sc),
                    _ => {
                        return Err(Diag::error(
                            "L0005",
                            format!("`{suf}` is not a valid integer-literal suffix"),
                            Span::new(suf_start, self.pos),
                        ))
                    }
                }
            }
        }
        Ok(Token {
            kind: TokKind::Int { value, suffix },
            span: Span::new(start, self.pos),
        })
    }

    fn lex_string(&mut self, start: usize, is_bytes: bool) -> Result<Token, Diag> {
        self.pos += 1; // opening quote
        let mut buf = String::new();
        loop {
            match self.peek() {
                None => {
                    return Err(Diag::error(
                        "L0006",
                        "unterminated string literal",
                        Span::new(start, self.pos),
                    ))
                }
                Some(b'"') => {
                    self.pos += 1;
                    let kind = if is_bytes { TokKind::Bytes(buf) } else { TokKind::Str(buf) };
                    return Ok(Token {
                        kind,
                        span: Span::new(start, self.pos),
                    });
                }
                Some(b'\\') => {
                    self.pos += 1;
                    match self.peek() {
                        Some(b'"') => buf.push('"'),
                        Some(b'\\') => buf.push('\\'),
                        Some(b'n') => buf.push('\n'),
                        Some(b't') => buf.push('\t'),
                        other => {
                            return Err(Diag::error(
                                "L0007",
                                format!(
                                    "unknown string escape `\\{}`",
                                    other.map(|c| c as char).unwrap_or(' ')
                                ),
                                Span::new(self.pos - 1, self.pos + 1),
                            ))
                        }
                    }
                    self.pos += 1;
                }
                Some(b) => {
                    // Preserve the raw source byte so multibyte UTF-8 in a string
                    // literal survives intact (design 0013: a `"..."` literal's bytes
                    // ARE its UTF-8 encoding). `b as char` would re-encode each byte
                    // as a codepoint, corrupting non-ASCII text. Escapes above only
                    // add ASCII, so `buf` stays well-formed UTF-8.
                    unsafe { buf.as_mut_vec().push(b); }
                    self.pos += 1;
                }
            }
        }
    }
}

fn is_ident_start(b: u8) -> bool {
    b == b'_' || b.is_ascii_alphabetic()
}

fn is_ident_continue(b: u8) -> bool {
    b == b'_' || b.is_ascii_alphanumeric()
}

/// Convenience wrapper.
pub fn lex(src: &str) -> Result<Vec<Token>, Diag> {
    Lexer::new(src).tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{Kw, ScalarTy};

    fn kinds(src: &str) -> Vec<TokKind> {
        lex(src).unwrap().into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn integers_decimal_hex_and_suffix() {
        assert_eq!(
            kinds("0 42 0x40 42u8"),
            vec![
                TokKind::Int { value: 0, suffix: None },
                TokKind::Int { value: 42, suffix: None },
                TokKind::Int { value: 0x40, suffix: None },
                TokKind::Int { value: 42, suffix: Some(ScalarTy::U8) },
                TokKind::Eof,
            ]
        );
    }

    #[test]
    fn keywords_scalars_and_idents() {
        assert_eq!(
            kinds("fn read block_size usize Box BoxResult"),
            vec![
                TokKind::Kw(Kw::Fn),
                TokKind::Kw(Kw::Read),
                TokKind::Ident("block_size".to_string()),
                TokKind::Scalar(ScalarTy::Usize),
                TokKind::Kw(Kw::Box),
                TokKind::Kw(Kw::BoxResult),
                TokKind::Eof,
            ]
        );
    }

    #[test]
    fn alloc_is_not_a_hard_keyword() {
        // `alloc` must lex as an ordinary identifier (contextual keyword).
        assert_eq!(
            kinds("alloc"),
            vec![TokKind::Ident("alloc".to_string()), TokKind::Eof]
        );
    }

    #[test]
    fn operators_and_punctuation() {
        assert_eq!(
            kinds(":: -> => == != <= >= && || ="),
            vec![
                TokKind::ColonColon,
                TokKind::Arrow,
                TokKind::FatArrow,
                TokKind::EqEq,
                TokKind::Ne,
                TokKind::Le,
                TokKind::Ge,
                TokKind::AmpAmp,
                TokKind::PipePipe,
                TokKind::Eq,
                TokKind::Eof,
            ]
        );
    }

    #[test]
    fn strings_and_comments() {
        let toks = kinds(
            "// line\n\"just a reason\" /* block */ \"esc \\\" done\"",
        );
        assert_eq!(
            toks,
            vec![
                TokKind::Str("just a reason".to_string()),
                TokKind::Str("esc \" done".to_string()),
                TokKind::Eof,
            ]
        );
    }

    #[test]
    fn spans_are_byte_offsets() {
        let toks = lex("fn  x").unwrap();
        assert_eq!(toks[0].span, Span::new(0, 2));
        assert_eq!(toks[1].span, Span::new(4, 5));
    }

    #[test]
    fn bad_char_is_structured_error_not_panic() {
        let e = lex("let x = @;").unwrap_err();
        assert_eq!(e.code, "L0001");
        assert_eq!(e.span, Span::new(8, 9));
    }

    #[test]
    fn unterminated_string_errors() {
        let e = lex("\"oops").unwrap_err();
        assert_eq!(e.code, "L0006");
    }
}
