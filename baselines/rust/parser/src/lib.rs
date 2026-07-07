//! Recursive-descent parser for CalcLang, per `docs/basket/spec-parser.md`.
//!
//! The parser borrows the input byte string: `Int` and `Ident` nodes hold a
//! `&str` sub-slice of the input rather than copied token text (spec §1.4), and
//! every node records a `[start, end)` byte span (spec §3.2). Errors are
//! returned as `{kind, offset}` values (spec §4); the parser never panics on
//! malformed input.

use std::str;

/// The first error encountered, reported as a value (spec §1.3, §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseError {
    pub kind: ErrorKind,
    /// 0-based byte offset; end-of-input errors use `offset == input.len()`.
    pub offset: usize,
}

/// The error kinds enumerated by spec §4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// `E_UNEXPECTED_CHAR`
    UnexpectedChar,
    /// `E_UNEXPECTED_EOF`
    UnexpectedEof,
    /// `E_EXPECTED_EXPR`
    ExpectedExpr,
    /// `E_EXPECTED_RPAREN`
    ExpectedRparen,
    /// `E_TRAILING_INPUT`
    TrailingInput,
}

/// Typed AST node (spec §3.1). Every node carries a `[start, end)` byte span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node<'a> {
    Int {
        text: &'a str,
        span: (usize, usize),
    },
    Ident {
        text: &'a str,
        span: (usize, usize),
    },
    Unary {
        op: &'static str,
        operand: Box<Node<'a>>,
        span: (usize, usize),
    },
    Binary {
        op: &'static str,
        left: Box<Node<'a>>,
        right: Box<Node<'a>>,
        span: (usize, usize),
    },
    Call {
        callee: Box<Node<'a>>,
        args: Vec<Node<'a>>,
        span: (usize, usize),
    },
}

impl<'a> Node<'a> {
    /// The node's `[start, end)` byte span into the input (spec §3.2).
    pub fn span(&self) -> (usize, usize) {
        match *self {
            Node::Int { span, .. }
            | Node::Ident { span, .. }
            | Node::Unary { span, .. }
            | Node::Binary { span, .. }
            | Node::Call { span, .. } => span,
        }
    }

    /// Canonical S-expression serialization `S(node)` (spec §3.3).
    pub fn serialize(&self) -> String {
        let mut out = String::new();
        self.write_sexpr(&mut out);
        out
    }

    fn write_sexpr(&self, out: &mut String) {
        match self {
            Node::Int { text, .. } | Node::Ident { text, .. } => out.push_str(text),
            Node::Unary { op, operand, .. } => {
                out.push('(');
                out.push('u');
                out.push_str(op);
                out.push(' ');
                operand.write_sexpr(out);
                out.push(')');
            }
            Node::Binary {
                op, left, right, ..
            } => {
                out.push('(');
                out.push_str(op);
                out.push(' ');
                left.write_sexpr(out);
                out.push(' ');
                right.write_sexpr(out);
                out.push(')');
            }
            Node::Call { callee, args, .. } => {
                out.push_str("(call ");
                callee.write_sexpr(out);
                for arg in args {
                    out.push(' ');
                    arg.write_sexpr(out);
                }
                out.push(')');
            }
        }
    }

    /// Collect `(start, end, text)` for every `Int`/`Ident` leaf, in
    /// left-to-right order, for the span/substring check of spec §3.5.
    pub fn collect_leaves(&self, out: &mut Vec<(usize, usize, &'a str)>) {
        match self {
            Node::Int { text, span } | Node::Ident { text, span } => {
                out.push((span.0, span.1, text));
            }
            Node::Unary { operand, .. } => operand.collect_leaves(out),
            Node::Binary { left, right, .. } => {
                left.collect_leaves(out);
                right.collect_leaves(out);
            }
            Node::Call { callee, args, .. } => {
                callee.collect_leaves(out);
                for arg in args {
                    arg.collect_leaves(out);
                }
            }
        }
    }
}

/// Parse `input` and return the canonical serialization `S(root)` (spec §3.3),
/// or the first error as a value (spec §4).
pub fn parse(input: &[u8]) -> Result<String, ParseError> {
    parse_ast(input).map(|node| node.serialize())
}

/// Parse `input` into the typed AST (spec §3), or return the first error.
pub fn parse_ast(input: &[u8]) -> Result<Node<'_>, ParseError> {
    let mut parser = Parser { input, pos: 0 };
    let node = parser.expr()?;
    let token = parser.peek()?;
    if token.kind != TokKind::Eof {
        // A complete expression followed by a further token (spec §2.3).
        return Err(ParseError {
            kind: ErrorKind::TrailingInput,
            offset: token.start,
        });
    }
    Ok(node)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokKind {
    Int,
    Ident,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    LParen,
    RParen,
    Comma,
    Lt,
    Le,
    Gt,
    Ge,
    EqEq,
    Ne,
    AndAnd,
    OrOr,
    Eof,
}

#[derive(Debug, Clone, Copy)]
struct Token {
    kind: TokKind,
    start: usize,
    end: usize,
}

struct Parser<'a> {
    input: &'a [u8],
    /// Next byte to lex from. Leading whitespace is skipped lazily on `peek`.
    pos: usize,
}

fn is_whitespace(b: u8) -> bool {
    matches!(b, 0x20 | 0x09 | 0x0A | 0x0D)
}

impl<'a> Parser<'a> {
    /// Lex the next token without consuming it. Whitespace is skipped (spec
    /// §2.1). At end of input returns an `Eof` token at `input.len()`. A byte
    /// that starts no token is `E_UNEXPECTED_CHAR` at its offset (spec §2.1).
    fn peek(&self) -> Result<Token, ParseError> {
        let len = self.input.len();
        let mut i = self.pos;
        while i < len && is_whitespace(self.input[i]) {
            i += 1;
        }
        if i >= len {
            return Ok(Token {
                kind: TokKind::Eof,
                start: len,
                end: len,
            });
        }
        let two_char_is = |second: u8| i + 1 < len && self.input[i + 1] == second;
        let (kind, end) = match self.input[i] {
            b'0'..=b'9' => {
                let mut j = i + 1;
                while j < len && self.input[j].is_ascii_digit() {
                    j += 1;
                }
                (TokKind::Int, j)
            }
            b'A'..=b'Z' | b'a'..=b'z' | b'_' => {
                let mut j = i + 1;
                while j < len && (self.input[j].is_ascii_alphanumeric() || self.input[j] == b'_') {
                    j += 1;
                }
                (TokKind::Ident, j)
            }
            b'+' => (TokKind::Plus, i + 1),
            b'-' => (TokKind::Minus, i + 1),
            b'*' => (TokKind::Star, i + 1),
            b'/' => (TokKind::Slash, i + 1),
            b'%' => (TokKind::Percent, i + 1),
            b'(' => (TokKind::LParen, i + 1),
            b')' => (TokKind::RParen, i + 1),
            b',' => (TokKind::Comma, i + 1),
            // Two-character tokens are matched maximally (spec §2.1).
            b'!' if two_char_is(b'=') => (TokKind::Ne, i + 2),
            b'!' => (TokKind::Bang, i + 1),
            b'<' if two_char_is(b'=') => (TokKind::Le, i + 2),
            b'<' => (TokKind::Lt, i + 1),
            b'>' if two_char_is(b'=') => (TokKind::Ge, i + 2),
            b'>' => (TokKind::Gt, i + 1),
            b'=' if two_char_is(b'=') => (TokKind::EqEq, i + 2),
            b'&' if two_char_is(b'&') => (TokKind::AndAnd, i + 2),
            b'|' if two_char_is(b'|') => (TokKind::OrOr, i + 2),
            // Lone `=`, `&`, `|` and any other byte are unexpected.
            _ => {
                return Err(ParseError {
                    kind: ErrorKind::UnexpectedChar,
                    offset: i,
                })
            }
        };
        Ok(Token {
            kind,
            start: i,
            end,
        })
    }

    /// Consume and return the next token.
    fn advance(&mut self) -> Result<Token, ParseError> {
        let token = self.peek()?;
        self.pos = token.end;
        Ok(token)
    }

    fn slice(&self, start: usize, end: usize) -> &'a str {
        // INT/IDENT lexemes are ASCII by construction, so this never fails.
        str::from_utf8(&self.input[start..end]).expect("INT/IDENT lexemes are ASCII")
    }

    fn expr(&mut self) -> Result<Node<'a>, ParseError> {
        self.or()
    }

    /// Parse one left-associative binary-operator level: `next (op next)*`.
    fn binary_level(
        &mut self,
        ops: &[(TokKind, &'static str)],
        next: fn(&mut Parser<'a>) -> Result<Node<'a>, ParseError>,
    ) -> Result<Node<'a>, ParseError> {
        let mut left = next(self)?;
        loop {
            let token = self.peek()?;
            if let Some(&(_, op)) = ops.iter().find(|(kind, _)| *kind == token.kind) {
                self.advance()?;
                let right = next(self)?;
                let span = (left.span().0, right.span().1);
                left = Node::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                    span,
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn or(&mut self) -> Result<Node<'a>, ParseError> {
        self.binary_level(&[(TokKind::OrOr, "||")], Parser::and)
    }

    fn and(&mut self) -> Result<Node<'a>, ParseError> {
        self.binary_level(&[(TokKind::AndAnd, "&&")], Parser::equality)
    }

    fn equality(&mut self) -> Result<Node<'a>, ParseError> {
        self.binary_level(
            &[(TokKind::EqEq, "=="), (TokKind::Ne, "!=")],
            Parser::comparison,
        )
    }

    fn comparison(&mut self) -> Result<Node<'a>, ParseError> {
        self.binary_level(
            &[
                (TokKind::Lt, "<"),
                (TokKind::Le, "<="),
                (TokKind::Gt, ">"),
                (TokKind::Ge, ">="),
            ],
            Parser::additive,
        )
    }

    fn additive(&mut self) -> Result<Node<'a>, ParseError> {
        self.binary_level(&[(TokKind::Plus, "+"), (TokKind::Minus, "-")], Parser::mul)
    }

    fn mul(&mut self) -> Result<Node<'a>, ParseError> {
        self.binary_level(
            &[
                (TokKind::Star, "*"),
                (TokKind::Slash, "/"),
                (TokKind::Percent, "%"),
            ],
            Parser::unary,
        )
    }

    /// `unary := ("-" | "!") unary | postfix` (right-associative prefix).
    fn unary(&mut self) -> Result<Node<'a>, ParseError> {
        let token = self.peek()?;
        let op = match token.kind {
            TokKind::Minus => "-",
            TokKind::Bang => "!",
            _ => return self.postfix(),
        };
        self.advance()?;
        let operand = self.unary()?;
        let span = (token.start, operand.span().1);
        Ok(Node::Unary {
            op,
            operand: Box::new(operand),
            span,
        })
    }

    /// `postfix := primary ("(" args ")")*` (left-associative calls).
    fn postfix(&mut self) -> Result<Node<'a>, ParseError> {
        let mut callee = self.primary()?;
        while self.peek()?.kind == TokKind::LParen {
            self.advance()?;
            let args = self.args()?;
            let close = self.peek()?;
            if close.kind != TokKind::RParen {
                return Err(ParseError {
                    kind: ErrorKind::ExpectedRparen,
                    offset: close.start,
                });
            }
            self.advance()?;
            let span = (callee.span().0, close.end);
            callee = Node::Call {
                callee: Box::new(callee),
                args,
                span,
            };
        }
        Ok(callee)
    }

    /// `args := ε | expr ("," expr)*` (no trailing comma).
    fn args(&mut self) -> Result<Vec<Node<'a>>, ParseError> {
        let mut args = Vec::new();
        if self.peek()?.kind == TokKind::RParen {
            return Ok(args);
        }
        loop {
            args.push(self.expr()?);
            if self.peek()?.kind == TokKind::Comma {
                self.advance()?;
            } else {
                break;
            }
        }
        Ok(args)
    }

    /// `primary := INT | IDENT | "(" expr ")"`.
    fn primary(&mut self) -> Result<Node<'a>, ParseError> {
        let token = self.peek()?;
        match token.kind {
            TokKind::Int => {
                self.advance()?;
                let text = self.slice(token.start, token.end);
                Ok(Node::Int {
                    text,
                    span: (token.start, token.end),
                })
            }
            TokKind::Ident => {
                self.advance()?;
                let text = self.slice(token.start, token.end);
                Ok(Node::Ident {
                    text,
                    span: (token.start, token.end),
                })
            }
            TokKind::LParen => {
                self.advance()?;
                let inner = self.expr()?;
                let close = self.peek()?;
                if close.kind != TokKind::RParen {
                    return Err(ParseError {
                        kind: ErrorKind::ExpectedRparen,
                        offset: close.start,
                    });
                }
                self.advance()?;
                // Grouping is transparent: the inner node keeps its own span.
                Ok(inner)
            }
            TokKind::Eof => Err(ParseError {
                kind: ErrorKind::UnexpectedEof,
                offset: token.start,
            }),
            _ => Err(ParseError {
                kind: ErrorKind::ExpectedExpr,
                offset: token.start,
            }),
        }
    }
}
