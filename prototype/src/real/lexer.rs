//! Hand-written lexer for the real surface syntax (spec chapter 01). Maximal
//! munch with the two documented boundaries: the negative-literal fold is
//! grammatical (a leading `-` is always its own token, §3.4/§5.3), and the
//! integer suffix is munched only when contiguous and valid (§3.2). `.*` is
//! decided on one character of lookahead after `.` (§5.4). Tokenization consults
//! no parse context or symbol table (NN#13, §6).

use crate::diag::Diag;
use crate::span::Span;
use crate::token::{scalar_from_str, ScalarTy};

use super::token::{real_keyword_from_str, RTok, RToken};

pub struct RLexer<'a> {
    src: &'a [u8],
    pos: usize,
    /// Byte spans of `//` line and `/* */` block comments, in source order.
    /// Collected only when [`RLexer::tokenize_collecting`] is used (the
    /// formatter); the ordinary [`RLexer::tokenize`] path leaves it empty.
    comments: Vec<Span>,
}

impl<'a> RLexer<'a> {
    pub fn new(src: &'a str) -> RLexer<'a> {
        RLexer { src: src.as_bytes(), pos: 0, comments: Vec::new() }
    }

    pub fn tokenize(mut self) -> Result<Vec<RToken>, Diag> {
        let mut out = Vec::new();
        loop {
            self.skip_trivia()?;
            if self.pos >= self.src.len() {
                out.push(RToken { kind: RTok::Eof, span: Span::point(self.pos) });
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

    fn skip_trivia(&mut self) -> Result<(), Diag> {
        loop {
            match self.peek() {
                Some(b) if b.is_ascii_whitespace() => self.pos += 1,
                Some(b'/') if self.peek2() == Some(b'/') => {
                    let start = self.pos;
                    while let Some(b) = self.peek() {
                        if b == b'\n' {
                            break;
                        }
                        self.pos += 1;
                    }
                    self.comments.push(Span::new(start, self.pos));
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
                    self.comments.push(Span::new(start, self.pos));
                }
                _ => return Ok(()),
            }
        }
    }

    fn next_token(&mut self) -> Result<RToken, Diag> {
        let start = self.pos;
        let b = self.peek().expect("caller guarantees a byte");

        // `b"..."` byte-string literal (design 0013, NN#13): one char lookahead.
        if b == b'b' && self.peek2() == Some(b'"') {
            self.pos += 1;
            return self.lex_string(start, true);
        }
        if is_ident_start(b) {
            return Ok(self.lex_ident(start));
        }
        if b.is_ascii_digit() {
            return self.lex_number(start);
        }
        if b == b'"' {
            return self.lex_string(start, false);
        }

        // Range operators (`..=` inclusive, `..` half-open). A `.` followed by
        // `.` is never a float continuation (that needs a digit after the dot),
        // so this classification is unambiguous.
        if b == b'.' && self.peek2() == Some(b'.') {
            let (kind, len) = if self.src.get(self.pos + 2).copied() == Some(b'=') {
                (RTok::DotDotEq, 3)
            } else {
                (RTok::DotDot, 2)
            };
            self.pos += len;
            return Ok(RToken { kind, span: Span::new(start, self.pos) });
        }

        let two = (b, self.peek2());
        let (kind, len) = match two {
            // `.*` deref token: one char of lookahead, whitespace breaks the pair.
            (b'.', Some(b'*')) => (RTok::DotStar, 2),
            (b':', Some(b':')) => (RTok::ColonColon, 2),
            (b'-', Some(b'>')) => (RTok::Arrow, 2),
            (b'=', Some(b'>')) => (RTok::FatArrow, 2),
            (b'=', Some(b'=')) => (RTok::EqEq, 2),
            (b'!', Some(b'=')) => (RTok::Ne, 2),
            (b'<', Some(b'<')) => (RTok::Shl, 2),
            (b'>', Some(b'>')) => (RTok::Shr, 2),
            (b'<', Some(b'=')) => (RTok::Le, 2),
            (b'>', Some(b'=')) => (RTok::Ge, 2),
            (b'&', Some(b'&')) => (RTok::AmpAmp, 2),
            (b'|', Some(b'|')) => (RTok::PipePipe, 2),
            (b'(', _) => (RTok::LParen, 1),
            (b')', _) => (RTok::RParen, 1),
            (b'{', _) => (RTok::LBrace, 1),
            (b'}', _) => (RTok::RBrace, 1),
            (b'[', _) => (RTok::LBracket, 1),
            (b']', _) => (RTok::RBracket, 1),
            (b',', _) => (RTok::Comma, 1),
            (b'.', _) => (RTok::Dot, 1),
            (b';', _) => (RTok::Semi, 1),
            (b':', _) => (RTok::Colon, 1),
            (b'=', _) => (RTok::Eq, 1),
            (b'<', _) => (RTok::Lt, 1),
            (b'>', _) => (RTok::Gt, 1),
            (b'+', _) => (RTok::Plus, 1),
            (b'-', _) => (RTok::Minus, 1),
            (b'*', _) => (RTok::Star, 1),
            (b'/', _) => (RTok::Slash, 1),
            (b'%', _) => (RTok::Percent, 1),
            (b'&', _) => (RTok::Amp, 1),
            (b'|', _) => (RTok::Pipe, 1),
            (b'^', _) => (RTok::Caret, 1),
            (b'~', _) => (RTok::Tilde, 1),
            (b'?', _) => (RTok::Question, 1),
            (b'!', _) => (RTok::Bang, 1),
            _ => {
                return Err(Diag::error(
                    "L0001",
                    format!("unexpected character `{}`", b as char),
                    Span::new(start, start + 1),
                ))
            }
        };
        self.pos += len;
        Ok(RToken { kind, span: Span::new(start, self.pos) })
    }

    fn lex_ident(&mut self, start: usize) -> RToken {
        while let Some(b) = self.peek() {
            if is_ident_continue(b) {
                self.pos += 1;
            } else {
                break;
            }
        }
        let text = std::str::from_utf8(&self.src[start..self.pos]).expect("ascii ident");
        let kind = if let Some(kw) = real_keyword_from_str(text) {
            RTok::Kw(kw)
        } else if let Some(sc) = scalar_from_str(text) {
            RTok::Scalar(sc)
        } else {
            RTok::Ident(text.to_string())
        };
        RToken { kind, span: Span::new(start, self.pos) }
    }

    fn lex_number(&mut self, start: usize) -> Result<RToken, Diag> {
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
                return Err(Diag::error("L0002", "hex literal `0x` has no digits", Span::new(start, self.pos)));
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
            // Float continuation (design 0016): a `.` with a digit on BOTH sides,
            // or an exponent (`e`/`E`). A `.` not followed by a digit is NOT part
            // of the number (e.g. `5.field`, `1.` is rejected below via the
            // digit-on-both-sides rule — `.` alone leaves an integer here).
            let is_frac = self.peek() == Some(b'.')
                && self.peek2().map(|c| c.is_ascii_digit()).unwrap_or(false);
            let is_exp = matches!(self.peek(), Some(b'e') | Some(b'E'))
                && match self.peek2() {
                    Some(c) if c.is_ascii_digit() => true,
                    Some(b'+') | Some(b'-') => true,
                    _ => false,
                };
            if is_frac || is_exp {
                return self.finish_float(start);
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
            Diag::error("L0003", "integer literal out of range for u64", Span::new(start, digits_end))
        })?;

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
        Ok(RToken { kind: RTok::Int { value, suffix }, span: Span::new(start, self.pos) })
    }

    /// Finish lexing a float literal (design 0016). `start` is the literal start;
    /// the cursor sits just past the integer part, at a `.`+digit or an exponent.
    /// Consumes the fractional digits and/or exponent, then parses the whole span
    /// as `f64` (Rust's `str::parse`, so an over-range magnitude yields `±inf`).
    fn finish_float(&mut self, start: usize) -> Result<RToken, Diag> {
        if self.peek() == Some(b'.') {
            self.pos += 1; // `.`
            while let Some(b) = self.peek() {
                if b.is_ascii_digit() {
                    self.pos += 1;
                } else {
                    break;
                }
            }
        }
        if matches!(self.peek(), Some(b'e') | Some(b'E')) {
            self.pos += 1; // `e`/`E`
            if matches!(self.peek(), Some(b'+') | Some(b'-')) {
                self.pos += 1;
            }
            let exp_start = self.pos;
            while let Some(b) = self.peek() {
                if b.is_ascii_digit() {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            if self.pos == exp_start {
                return Err(Diag::error("L0008", "float exponent has no digits", Span::new(start, self.pos)));
            }
        }
        // The numeric text ends here; an optional `f32` suffix selects single
        // precision (design 0016). A suffix-less float form stays `f64` (the
        // default float type); any other suffix is rejected.
        let num_end = self.pos;
        let mut float_ty = ScalarTy::F64;
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
                if suf == "f32" {
                    float_ty = ScalarTy::F32;
                } else {
                    return Err(Diag::error(
                        "L0005",
                        format!("`{suf}` is not a valid float-literal suffix (only `f32`; design 0016)"),
                        Span::new(suf_start, self.pos),
                    ));
                }
            }
        }
        let text = std::str::from_utf8(&self.src[start..num_end]).expect("ascii float");
        // Parse at the literal's precision so an over-range magnitude yields the
        // type's `±inf` (Rust `str::parse` semantics), mirroring the `f64` slice.
        let bits = match float_ty {
            ScalarTy::F32 => text
                .parse::<f32>()
                .map_err(|_| Diag::error("L0009", "malformed float literal", Span::new(start, num_end)))?
                .to_bits() as u64,
            _ => text
                .parse::<f64>()
                .map_err(|_| Diag::error("L0009", "malformed float literal", Span::new(start, num_end)))?
                .to_bits(),
        };
        Ok(RToken { kind: RTok::Float { bits, ty: float_ty }, span: Span::new(start, self.pos) })
    }

    fn lex_string(&mut self, start: usize, is_bytes: bool) -> Result<RToken, Diag> {
        self.pos += 1;
        let mut buf = String::new();
        loop {
            match self.peek() {
                None => {
                    return Err(Diag::error("L0006", "unterminated string literal", Span::new(start, self.pos)))
                }
                Some(b'"') => {
                    self.pos += 1;
                    let kind = if is_bytes { RTok::Bytes(buf) } else { RTok::Str(buf) };
                    return Ok(RToken { kind, span: Span::new(start, self.pos) });
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
                                format!("unknown string escape `\\{}`", other.map(|c| c as char).unwrap_or(' ')),
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
pub fn lex(src: &str) -> Result<Vec<RToken>, Diag> {
    RLexer::new(src).tokenize()
}

impl RLexer<'_> {
    /// Like [`RLexer::tokenize`] but also returns the byte spans of every comment
    /// (in source order). The formatter uses this to re-attach comments the AST
    /// does not carry (spec 02 §9; NN#11).
    pub fn tokenize_collecting(mut self) -> Result<(Vec<RToken>, Vec<Span>), Diag> {
        let toks = self.tokenize_inner()?;
        Ok((toks, self.comments))
    }

    fn tokenize_inner(&mut self) -> Result<Vec<RToken>, Diag> {
        let mut out = Vec::new();
        loop {
            self.skip_trivia()?;
            if self.pos >= self.src.len() {
                out.push(RToken { kind: RTok::Eof, span: Span::point(self.pos) });
                return Ok(out);
            }
            out.push(self.next_token()?);
        }
    }
}

/// Lex `src`, returning both the token stream and the comment spans (formatter).
pub fn lex_with_comments(src: &str) -> Result<(Vec<RToken>, Vec<Span>), Diag> {
    RLexer::new(src).tokenize_collecting()
}
