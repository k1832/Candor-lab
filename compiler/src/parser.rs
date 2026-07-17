//! Recursive-descent parser. No symbol table (NN#13): every construct is
//! disambiguated by keyword and grammatical position. Returns the first
//! structured `Diag` on error (single-error reporting, as specified).

use crate::ast::*;
use crate::diag::Diag;
use crate::span::Span;
use crate::token::{Kw, TokKind, Token};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    last_end: usize,
    /// When true, a bare `Ident {` is NOT a struct literal (used in `if`/`while`/
    /// `match` head positions, mirroring Rust's no-struct-literal restriction).
    no_struct: bool,
}

type PResult<T> = Result<T, Diag>;

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            pos: 0,
            last_end: 0,
            no_struct: false,
        }
    }

    // ----- token cursor helpers -------------------------------------------

    fn peek(&self) -> &TokKind {
        &self.tokens[self.pos].kind
    }

    fn peek_at(&self, n: usize) -> &TokKind {
        let i = (self.pos + n).min(self.tokens.len() - 1);
        &self.tokens[i].kind
    }

    fn cur_span(&self) -> Span {
        self.tokens[self.pos].span
    }

    fn cur_start(&self) -> usize {
        self.tokens[self.pos].span.start
    }

    fn bump(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        self.last_end = t.span.end;
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    fn span_from(&self, lo: usize) -> Span {
        Span::new(lo, self.last_end)
    }

    fn at(&self, k: &TokKind) -> bool {
        self.peek() == k
    }

    fn at_kw(&self, kw: Kw) -> bool {
        matches!(self.peek(), TokKind::Kw(k) if *k == kw)
    }

    fn at_ident(&self, name: &str) -> bool {
        matches!(self.peek(), TokKind::Ident(s) if s == name)
    }

    fn eat(&mut self, k: &TokKind) -> bool {
        if self.at(k) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn eat_kw(&mut self, kw: Kw) -> bool {
        if self.at_kw(kw) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, k: &TokKind, what: &str) -> PResult<()> {
        if self.at(k) {
            self.bump();
            Ok(())
        } else {
            Err(self.unexpected(what))
        }
    }

    fn expect_ident(&mut self, what: &str) -> PResult<String> {
        match self.peek().clone() {
            TokKind::Ident(s) => {
                self.bump();
                Ok(s)
            }
            _ => Err(self.unexpected(what)),
        }
    }

    fn unexpected(&self, what: &str) -> Diag {
        Diag::error(
            "P0001",
            format!("expected {what}, found {}", describe(self.peek())),
            self.cur_span(),
        )
    }

    // ----- program & items ------------------------------------------------

    pub fn parse_program(&mut self) -> PResult<Program> {
        let mut items = Vec::new();
        while !self.at(&TokKind::Eof) {
            items.push(self.parse_item()?);
        }
        Ok(Program { items })
    }

    fn parse_item(&mut self) -> PResult<Item> {
        let copy = self.eat_kw(Kw::Copy);
        match self.peek() {
            TokKind::Kw(Kw::Struct) => Ok(Item::Struct(self.parse_struct(copy)?)),
            TokKind::Kw(Kw::Enum) => Ok(Item::Enum(self.parse_enum(copy)?)),
            _ if copy => Err(Diag::error(
                "P0002",
                "`copy` may only precede `struct` or `enum`",
                self.cur_span(),
            )),
            TokKind::Kw(Kw::Fn) => Ok(Item::Fn(self.parse_fn()?)),
            TokKind::Kw(Kw::Static) => Ok(Item::Static(self.parse_static()?)),
            _ => Err(self.unexpected("an item (`struct`, `enum`, `fn`, `static`)")),
        }
    }

    fn parse_struct(&mut self, copy: bool) -> PResult<StructDecl> {
        let lo = self.cur_start();
        self.expect(&TokKind::Kw(Kw::Struct), "`struct`")?;
        let name = self.expect_ident("a struct name")?;
        self.expect(&TokKind::LBrace, "`{`")?;
        let mut fields = Vec::new();
        while !self.at(&TokKind::RBrace) {
            let flo = self.cur_start();
            let fname = self.expect_ident("a field name")?;
            self.expect(&TokKind::Colon, "`:`")?;
            let ty = self.parse_type()?;
            fields.push(Field {
                name: fname,
                ty,
                span: self.span_from(flo),
            });
            if !self.eat(&TokKind::Comma) {
                break;
            }
        }
        self.expect(&TokKind::RBrace, "`}`")?;
        let drop_hook = if self.at_kw(Kw::Drop) {
            Some(self.parse_drop_hook()?)
        } else {
            None
        };
        Ok(StructDecl {
            type_params: Vec::new(),
            copy,
            name,
            fields,
            drop_hook,
            span: self.span_from(lo),
        })
    }

    fn parse_drop_hook(&mut self) -> PResult<Block> {
        self.expect(&TokKind::Kw(Kw::Drop), "`drop`")?;
        self.expect(&TokKind::LParen, "`(`")?;
        self.expect(&TokKind::Kw(Kw::Write), "`write`")?;
        self.expect(&TokKind::Kw(Kw::SelfKw), "`self`")?;
        self.expect(&TokKind::RParen, "`)`")?;
        self.parse_block()
    }

    fn parse_enum(&mut self, copy: bool) -> PResult<EnumDecl> {
        let lo = self.cur_start();
        self.expect(&TokKind::Kw(Kw::Enum), "`enum`")?;
        let name = self.expect_ident("an enum name")?;
        self.expect(&TokKind::LBrace, "`{`")?;
        let mut variants = Vec::new();
        while !self.at(&TokKind::RBrace) {
            let vlo = self.cur_start();
            let vname = self.expect_ident("a variant name")?;
            let mut payload = Vec::new();
            if self.eat(&TokKind::LParen) {
                while !self.at(&TokKind::RParen) {
                    payload.push(self.parse_type()?);
                    if !self.eat(&TokKind::Comma) {
                        break;
                    }
                }
                self.expect(&TokKind::RParen, "`)`")?;
            }
            variants.push(Variant {
                name: vname,
                payload,
                ok: false,
                span: self.span_from(vlo),
            });
            if !self.eat(&TokKind::Comma) {
                break;
            }
        }
        self.expect(&TokKind::RBrace, "`}`")?;
        Ok(EnumDecl {
            type_params: Vec::new(),
            copy,
            name,
            variants,
            span: self.span_from(lo),
        })
    }

    fn parse_static(&mut self) -> PResult<StaticDecl> {
        let lo = self.cur_start();
        self.expect(&TokKind::Kw(Kw::Static), "`static`")?;
        let name = self.expect_ident("a static name")?;
        self.expect(&TokKind::Colon, "`:`")?;
        let ty = self.parse_type()?;
        self.expect(&TokKind::Eq, "`=`")?;
        let value = self.parse_expr()?;
        self.expect(&TokKind::Semi, "`;`")?;
        Ok(StaticDecl {
            name,
            ty,
            value,
            span: self.span_from(lo),
        })
    }

    fn parse_fn(&mut self) -> PResult<FnDecl> {
        let lo = self.cur_start();
        self.expect(&TokKind::Kw(Kw::Fn), "`fn`")?;
        let name = self.expect_ident("a function name")?;
        let regions = self.parse_region_list()?;
        self.expect(&TokKind::LParen, "`(`")?;
        let mut params = Vec::new();
        while !self.at(&TokKind::RParen) {
            params.push(self.parse_param()?);
            if !self.eat(&TokKind::Comma) {
                break;
            }
        }
        self.expect(&TokKind::RParen, "`)`")?;

        // signature tail: `alloc` (contextual keyword) and contract clauses,
        // in any order, until `->` or the body `{`.
        let mut alloc = false;
        let mut requires = Vec::new();
        let mut ensures = Vec::new();
        loop {
            if self.at_ident("alloc") {
                self.bump();
                alloc = true;
            } else if self.eat_kw(Kw::Requires) {
                self.expect(&TokKind::LParen, "`(`")?;
                requires.push(self.parse_expr()?);
                self.expect(&TokKind::RParen, "`)`")?;
            } else if self.eat_kw(Kw::Ensures) {
                self.expect(&TokKind::LParen, "`(`")?;
                ensures.push(self.parse_expr()?);
                self.expect(&TokKind::RParen, "`)`")?;
            } else {
                break;
            }
        }

        let ret = if self.eat(&TokKind::Arrow) {
            Some(self.parse_ret_ty()?)
        } else {
            None
        };
        let body = self.parse_block()?;
        Ok(FnDecl {
            type_params: Vec::new(),
            name,
            regions,
            params,
            alloc,
            foreign: false,
            boundary: false,
            requires,
            ensures,
            ret,
            body,
            span: self.span_from(lo),
        })
    }

    fn parse_region_list(&mut self) -> PResult<Vec<String>> {
        let mut regions = Vec::new();
        if self.eat(&TokKind::LBracket) {
            while !self.at(&TokKind::RBracket) {
                regions.push(self.expect_ident("a region variable")?);
                if !self.eat(&TokKind::Comma) {
                    break;
                }
            }
            self.expect(&TokKind::RBracket, "`]`")?;
        }
        Ok(regions)
    }

    fn parse_param(&mut self) -> PResult<Param> {
        let lo = self.cur_start();
        let name = self.expect_ident("a parameter name")?;
        self.expect(&TokKind::Colon, "`:`")?;
        let (mode, region) = self.parse_mode()?;
        let ty = self.parse_type()?;
        Ok(Param {
            name,
            mode,
            region,
            ty,
            span: self.span_from(lo),
        })
    }

    /// Parse an optional parameter mode plus an optional region tag on a borrow.
    fn parse_mode(&mut self) -> PResult<(ParamMode, Option<String>)> {
        let mode = match self.peek() {
            TokKind::Kw(Kw::Take) => {
                self.bump();
                ParamMode::Take
            }
            TokKind::Kw(Kw::Out) => {
                self.bump();
                ParamMode::Out
            }
            TokKind::Kw(Kw::Read) => {
                self.bump();
                ParamMode::Read
            }
            TokKind::Kw(Kw::Write) => {
                self.bump();
                ParamMode::Write
            }
            _ => ParamMode::Take, // omitted => take
        };
        let region = if matches!(mode, ParamMode::Read | ParamMode::Write)
            && self.at(&TokKind::LBracket)
        {
            self.bump();
            let r = self.expect_ident("a region variable")?;
            self.expect(&TokKind::RBracket, "`]`")?;
            Some(r)
        } else {
            None
        };
        Ok((mode, region))
    }

    fn parse_ret_ty(&mut self) -> PResult<RetTy> {
        let lo = self.cur_start();
        let borrow = match self.peek() {
            TokKind::Kw(Kw::Read) => {
                self.bump();
                Some(BorrowKind::Shared)
            }
            TokKind::Kw(Kw::Write) => {
                self.bump();
                Some(BorrowKind::Exclusive)
            }
            _ => None,
        };
        let region = if borrow.is_some() && self.at(&TokKind::LBracket) {
            self.bump();
            let r = self.expect_ident("a region variable")?;
            self.expect(&TokKind::RBracket, "`]`")?;
            Some(r)
        } else {
            None
        };
        let ty = self.parse_type()?;
        Ok(RetTy {
            borrow,
            region,
            ty,
            span: self.span_from(lo),
        })
    }

    // ----- types ----------------------------------------------------------

    fn parse_type(&mut self) -> PResult<Ty> {
        let lo = self.cur_start();
        let kind = match self.peek().clone() {
            TokKind::Scalar(sc) => {
                self.bump();
                TyKind::Scalar(sc)
            }
            TokKind::Kw(Kw::Slice) => {
                self.bump();
                TyKind::Slice(Box::new(self.parse_type()?))
            }
            TokKind::Kw(Kw::SliceMut) => {
                self.bump();
                TyKind::SliceMut(Box::new(self.parse_type()?))
            }
            TokKind::Kw(Kw::RawPtr) => {
                self.bump();
                TyKind::RawPtr(Box::new(self.parse_type()?))
            }
            TokKind::Kw(Kw::Box) => {
                self.bump();
                TyKind::Box(Box::new(self.parse_type()?))
            }
            TokKind::Kw(Kw::BoxResult) => {
                self.bump();
                TyKind::BoxResult(Box::new(self.parse_type()?))
            }
            TokKind::Kw(Kw::Borrow) => {
                self.bump();
                TyKind::Borrow(Box::new(self.parse_type()?))
            }
            TokKind::Kw(Kw::BorrowMut) => {
                self.bump();
                TyKind::BorrowMut(Box::new(self.parse_type()?))
            }
            TokKind::Kw(Kw::Fn) => TyKind::FnPtr(self.parse_fnptr_type()?),
            TokKind::LBracket => {
                self.bump();
                let size = self.parse_array_size()?;
                self.expect(&TokKind::RBracket, "`]`")?;
                let elem = self.parse_type()?;
                TyKind::Array {
                    size: Box::new(size),
                    elem: Box::new(elem),
                }
            }
            TokKind::Ident(name) => {
                self.bump();
                TyKind::Named(name)
            }
            _ => return Err(self.unexpected("a type")),
        };
        Ok(Ty {
            kind,
            span: self.span_from(lo),
        })
    }

    /// Array size is a const expression: an integer literal or a name.
    fn parse_array_size(&mut self) -> PResult<Expr> {
        let lo = self.cur_start();
        let kind = match self.peek().clone() {
            TokKind::Int { value, suffix } => {
                self.bump();
                ExprKind::IntLit { value, suffix }
            }
            TokKind::Ident(name) => {
                self.bump();
                ExprKind::Ident(name)
            }
            _ => return Err(self.unexpected("an array size (integer literal or name)")),
        };
        Ok(Expr {
            kind,
            span: self.span_from(lo),
        })
    }

    fn parse_fnptr_type(&mut self) -> PResult<FnPtrTy> {
        self.expect(&TokKind::Kw(Kw::Fn), "`fn`")?;
        self.expect(&TokKind::LParen, "`(`")?;
        let mut params = Vec::new();
        while !self.at(&TokKind::RParen) {
            params.push(self.parse_fnptr_param()?);
            if !self.eat(&TokKind::Comma) {
                break;
            }
        }
        self.expect(&TokKind::RParen, "`)`")?;
        let alloc = if self.at_ident("alloc") {
            self.bump();
            true
        } else {
            false
        };
        self.expect(&TokKind::Arrow, "`->`")?;
        let ret = self.parse_type()?;
        Ok(FnPtrTy {
            params,
            alloc,
            foreign: false,
            ret: Box::new(ret),
        })
    }

    fn parse_fnptr_param(&mut self) -> PResult<FnPtrParam> {
        // optional `name :`
        let name = if matches!(self.peek(), TokKind::Ident(_))
            && matches!(self.peek_at(1), TokKind::Colon)
        {
            let n = self.expect_ident("a parameter name")?;
            self.bump(); // `:`
            Some(n)
        } else {
            None
        };
        let (mode, region) = self.parse_mode()?;
        let ty = self.parse_type()?;
        Ok(FnPtrParam {
            name,
            mode,
            region,
            ty,
        })
    }

    // ----- blocks & statements --------------------------------------------

    fn parse_block(&mut self) -> PResult<Block> {
        let lo = self.cur_start();
        self.expect(&TokKind::LBrace, "`{`")?;
        let mut stmts = Vec::new();
        while !self.at(&TokKind::RBrace) && !self.at(&TokKind::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&TokKind::RBrace, "`}`")?;
        Ok(Block {
            stmts,
            span: self.span_from(lo),
        })
    }

    fn parse_stmt(&mut self) -> PResult<Stmt> {
        let lo = self.cur_start();
        if self.at_kw(Kw::Let) {
            return self.parse_let(lo);
        }
        // Otherwise: an expression, possibly the LHS of an assignment.
        let expr = self.parse_expr()?;
        if self.eat(&TokKind::Eq) {
            let value = self.parse_expr()?;
            self.expect(&TokKind::Semi, "`;`")?;
            return Ok(Stmt {
                kind: StmtKind::Assign {
                    target: expr,
                    value,
                },
                span: self.span_from(lo),
            });
        }
        // Block-like expressions may stand as a statement without a `;`.
        if is_block_like(&expr.kind) {
            self.eat(&TokKind::Semi);
        } else {
            self.expect(&TokKind::Semi, "`;`")?;
        }
        Ok(Stmt {
            kind: StmtKind::Expr(expr),
            span: self.span_from(lo),
        })
    }

    fn parse_let(&mut self, lo: usize) -> PResult<Stmt> {
        self.expect(&TokKind::Kw(Kw::Let), "`let`")?;
        let mutable = self.eat_kw(Kw::Mut);
        let name = self.expect_ident("a binding name")?;
        let ty = if self.eat(&TokKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        let init = if self.eat(&TokKind::Eq) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(&TokKind::Semi, "`;`")?;
        Ok(Stmt {
            kind: StmtKind::Let {
                mutable,
                name,
                ty,
                init,
            },
            span: self.span_from(lo),
        })
    }

    // ----- expressions (precedence climbing) ------------------------------

    fn parse_expr(&mut self) -> PResult<Expr> {
        self.parse_or()
    }

    /// Parse an expression where a bare `Ident {` is not a struct literal.
    fn parse_expr_no_struct(&mut self) -> PResult<Expr> {
        let saved = self.no_struct;
        self.no_struct = true;
        let r = self.parse_or();
        self.no_struct = saved;
        r
    }

    fn parse_binary_level(
        &mut self,
        next: fn(&mut Self) -> PResult<Expr>,
        ops: &[(TokKind, BinOp)],
    ) -> PResult<Expr> {
        let lo = self.cur_start();
        let mut lhs = next(self)?;
        'outer: loop {
            for (tok, op) in ops {
                if self.at(tok) {
                    self.bump();
                    let rhs = next(self)?;
                    lhs = Expr {
                        kind: ExprKind::Binary {
                            op: *op,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        span: self.span_from(lo),
                    };
                    continue 'outer;
                }
            }
            break;
        }
        Ok(lhs)
    }

    fn parse_or(&mut self) -> PResult<Expr> {
        self.parse_binary_level(Self::parse_and, &[(TokKind::PipePipe, BinOp::Or)])
    }

    fn parse_and(&mut self) -> PResult<Expr> {
        self.parse_binary_level(Self::parse_cmp, &[(TokKind::AmpAmp, BinOp::And)])
    }

    fn parse_cmp(&mut self) -> PResult<Expr> {
        self.parse_binary_level(
            Self::parse_add,
            &[
                (TokKind::EqEq, BinOp::Eq),
                (TokKind::Ne, BinOp::Ne),
                (TokKind::Le, BinOp::Le),
                (TokKind::Ge, BinOp::Ge),
                (TokKind::Lt, BinOp::Lt),
                (TokKind::Gt, BinOp::Gt),
            ],
        )
    }

    fn parse_add(&mut self) -> PResult<Expr> {
        self.parse_binary_level(
            Self::parse_mul,
            &[(TokKind::Plus, BinOp::Add), (TokKind::Minus, BinOp::Sub)],
        )
    }

    fn parse_mul(&mut self) -> PResult<Expr> {
        self.parse_binary_level(
            Self::parse_prefix,
            &[
                (TokKind::Star, BinOp::Mul),
                (TokKind::Slash, BinOp::Div),
                (TokKind::Percent, BinOp::Rem),
            ],
        )
    }

    fn parse_prefix(&mut self) -> PResult<Expr> {
        let lo = self.cur_start();
        let kind = match self.peek() {
            TokKind::Minus => {
                self.bump();
                ExprKind::Unary {
                    op: UnOp::Neg,
                    expr: Box::new(self.parse_prefix()?),
                }
            }
            TokKind::Bang => {
                self.bump();
                ExprKind::Unary {
                    op: UnOp::Not,
                    expr: Box::new(self.parse_prefix()?),
                }
            }
            TokKind::Kw(Kw::Deref) => {
                self.bump();
                ExprKind::Prefix {
                    op: PrefixOp::Deref,
                    expr: Box::new(self.parse_prefix()?),
                }
            }
            TokKind::Kw(Kw::Read) => {
                self.bump();
                ExprKind::Prefix {
                    op: PrefixOp::Read,
                    expr: Box::new(self.parse_prefix()?),
                }
            }
            TokKind::Kw(Kw::Write) => {
                self.bump();
                ExprKind::Prefix {
                    op: PrefixOp::Write,
                    expr: Box::new(self.parse_prefix()?),
                }
            }
            TokKind::Kw(Kw::Clone) => {
                self.bump();
                ExprKind::Prefix {
                    op: PrefixOp::Clone,
                    expr: Box::new(self.parse_prefix()?),
                }
            }
            TokKind::Kw(Kw::Conv) => {
                self.bump();
                let ty = self.parse_type()?;
                self.expect(&TokKind::LParen, "`(`")?;
                let inner = self.parse_delimited_expr()?;
                self.expect(&TokKind::RParen, "`)`")?;
                ExprKind::Conv {
                    ty,
                    expr: Box::new(inner),
                }
            }
            TokKind::Kw(Kw::Bitcast) => {
                self.bump();
                let ty = self.parse_type()?;
                self.expect(&TokKind::LParen, "`(`")?;
                let inner = self.parse_delimited_expr()?;
                self.expect(&TokKind::RParen, "`)`")?;
                ExprKind::Bitcast {
                    ty,
                    expr: Box::new(inner),
                }
            }
            _ => return self.parse_postfix(),
        };
        Ok(Expr {
            kind,
            span: self.span_from(lo),
        })
    }

    fn parse_postfix(&mut self) -> PResult<Expr> {
        let lo = self.cur_start();
        let mut expr = self.parse_primary()?;
        loop {
            if self.at(&TokKind::LParen) {
                self.bump();
                let args = self.parse_arg_list()?;
                self.expect(&TokKind::RParen, "`)`")?;
                expr = Expr {
                    kind: ExprKind::Call {
                        callee: Box::new(expr),
                        args,
                    },
                    span: self.span_from(lo),
                };
            } else if self.at(&TokKind::LBracket) {
                self.bump();
                let index = self.parse_delimited_expr()?;
                self.expect(&TokKind::RBracket, "`]`")?;
                expr = Expr {
                    kind: ExprKind::Index {
                        base: Box::new(expr),
                        index: Box::new(index),
                    },
                    span: self.span_from(lo),
                };
            } else if self.at(&TokKind::Dot) {
                self.bump();
                let field = self.expect_ident("a field name")?;
                expr = Expr {
                    kind: ExprKind::Field {
                        base: Box::new(expr),
                        field,
                        iface: None,
                    },
                    span: self.span_from(lo),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_arg_list(&mut self) -> PResult<Vec<Expr>> {
        let saved = self.no_struct;
        self.no_struct = false;
        let mut args = Vec::new();
        while !self.at(&TokKind::RParen) {
            if self.at(&TokKind::Kw(Kw::Out)) {
                let lo = self.cur_start();
                self.bump();
                let place = self.parse_or()?;
                args.push(Expr {
                    kind: ExprKind::OutArg(Box::new(place)),
                    span: self.span_from(lo),
                });
            } else {
                args.push(self.parse_or()?);
            }
            if !self.eat(&TokKind::Comma) {
                break;
            }
        }
        self.no_struct = saved;
        Ok(args)
    }

    /// Parse an expression inside `(...)` or `[...]`, allowing struct literals.
    fn parse_delimited_expr(&mut self) -> PResult<Expr> {
        let saved = self.no_struct;
        self.no_struct = false;
        let r = self.parse_or();
        self.no_struct = saved;
        r
    }

    fn parse_primary(&mut self) -> PResult<Expr> {
        let lo = self.cur_start();
        let kind = match self.peek().clone() {
            TokKind::Int { value, suffix } => {
                self.bump();
                ExprKind::IntLit { value, suffix }
            }
            TokKind::Str(s) => {
                self.bump();
                ExprKind::StrLit(s)
            }
            TokKind::Bytes(s) => {
                self.bump();
                ExprKind::BytesLit(s)
            }
            TokKind::Kw(Kw::True) => {
                self.bump();
                ExprKind::BoolLit(true)
            }
            TokKind::Kw(Kw::False) => {
                self.bump();
                ExprKind::BoolLit(false)
            }
            TokKind::Kw(Kw::Result) => {
                self.bump();
                ExprKind::Result
            }
            TokKind::Kw(Kw::SelfKw) => {
                self.bump();
                ExprKind::Ident("self".to_string())
            }
            TokKind::Kw(Kw::Break) => {
                self.bump();
                ExprKind::Break
            }
            TokKind::Kw(Kw::Continue) => {
                self.bump();
                ExprKind::Continue
            }
            TokKind::Kw(Kw::Return) => {
                self.bump();
                if self.starts_expr() {
                    ExprKind::Return(Some(Box::new(self.parse_expr()?)))
                } else {
                    ExprKind::Return(None)
                }
            }
            TokKind::Kw(Kw::Assert) => {
                self.bump();
                self.expect(&TokKind::LParen, "`(`")?;
                let e = self.parse_delimited_expr()?;
                self.expect(&TokKind::RParen, "`)`")?;
                ExprKind::Assert(Box::new(e))
            }
            TokKind::Kw(Kw::Panic) => {
                self.bump();
                self.expect(&TokKind::LParen, "`(`")?;
                let e = self.parse_delimited_expr()?;
                self.expect(&TokKind::RParen, "`)`")?;
                ExprKind::Panic(Box::new(e))
            }
            TokKind::LParen => {
                self.bump();
                let inner = self.parse_delimited_expr()?;
                self.expect(&TokKind::RParen, "`)`")?;
                ExprKind::Paren(Box::new(inner))
            }
            TokKind::LBracket => self.parse_array_literal()?,
            TokKind::LBrace => ExprKind::Block(self.parse_block()?),
            TokKind::Kw(Kw::If) => self.parse_if()?,
            TokKind::Kw(Kw::Match) => self.parse_match()?,
            TokKind::Kw(Kw::Loop) => {
                self.bump();
                ExprKind::Loop(self.parse_block()?)
            }
            TokKind::Kw(Kw::While) => {
                self.bump();
                let cond = self.parse_expr_no_struct()?;
                let body = self.parse_block()?;
                ExprKind::While {
                    cond: Box::new(cond),
                    body,
                }
            }
            TokKind::Kw(Kw::Unsafe) => {
                self.bump();
                let justification = match self.peek().clone() {
                    TokKind::Str(s) => {
                        self.bump();
                        s
                    }
                    _ => {
                        return Err(Diag::error(
                            "P0003",
                            "`unsafe` requires a justification string literal",
                            self.cur_span(),
                        ))
                    }
                };
                let body = self.parse_block()?;
                ExprKind::Unsafe {
                    justification,
                    body,
                }
            }
            TokKind::Kw(Kw::Wrapping) => {
                self.bump();
                ExprKind::Wrapping(self.parse_block()?)
            }
            TokKind::Kw(Kw::Saturating) => {
                self.bump();
                ExprKind::Saturating(self.parse_block()?)
            }
            TokKind::Kw(Kw::CastPtr) => self.parse_type_arg_intrinsic(IntrinsicKind::CastPtr)?,
            TokKind::Kw(Kw::AddrToPtr) => {
                self.parse_type_arg_intrinsic(IntrinsicKind::AddrToPtr)?
            }
            TokKind::Kw(Kw::PtrNull) => self.parse_type_arg_intrinsic(IntrinsicKind::PtrNull)?,
            TokKind::Kw(Kw::Offsetof) => {
                self.bump();
                self.expect(&TokKind::LParen, "`(`")?;
                let ty = self.parse_type()?;
                self.expect(&TokKind::Comma, "`,`")?;
                let field = self.expect_ident("a field name")?;
                self.expect(&TokKind::RParen, "`)`")?;
                ExprKind::Offsetof { ty, field }
            }
            TokKind::Kw(Kw::FieldPtr) => {
                // `field_ptr(p, f)` — first arg an expression, second a field
                // selector (identifier in field position; no symbol table needed
                // to parse, NN#13). Design 0004.
                self.bump();
                self.expect(&TokKind::LParen, "`(`")?;
                let ptr = self.parse_expr()?;
                self.expect(&TokKind::Comma, "`,`")?;
                let field = self.expect_ident("a field name")?;
                self.expect(&TokKind::RParen, "`)`")?;
                ExprKind::FieldPtr { ptr: Box::new(ptr), field }
            }
            TokKind::Kw(Kw::Sizeof) => {
                self.bump();
                self.expect(&TokKind::LParen, "`(`")?;
                let ty = self.parse_type()?;
                self.expect(&TokKind::RParen, "`)`")?;
                ExprKind::Sizeof(ty)
            }
            TokKind::Kw(Kw::Alignof) => {
                self.bump();
                self.expect(&TokKind::LParen, "`(`")?;
                let ty = self.parse_type()?;
                self.expect(&TokKind::RParen, "`)`")?;
                ExprKind::Alignof(ty)
            }
            // `BoxResult::Variant` — compiler-known enum used by name.
            TokKind::Kw(Kw::BoxResult) => {
                self.bump();
                self.expect(&TokKind::ColonColon, "`::`")?;
                let variant = self.expect_ident("a variant name")?;
                let args = self.parse_opt_ctor_args()?;
                ExprKind::EnumCtor {
                    enum_name: "BoxResult".to_string(),
                    variant,
                    args,
                }
            }
            TokKind::Ident(name) => {
                if matches!(self.peek_at(1), TokKind::ColonColon) {
                    self.bump(); // name
                    self.bump(); // ::
                    let variant = self.expect_ident("a variant name")?;
                    let args = self.parse_opt_ctor_args()?;
                    ExprKind::EnumCtor {
                        enum_name: name,
                        variant,
                        args,
                    }
                } else if !self.no_struct && matches!(self.peek_at(1), TokKind::LBrace) {
                    self.bump(); // name
                    self.parse_struct_literal_body(name)?
                } else {
                    self.bump();
                    ExprKind::Ident(name)
                }
            }
            _ => return Err(self.unexpected("an expression")),
        };
        Ok(Expr {
            kind,
            span: self.span_from(lo),
        })
    }

    fn parse_opt_ctor_args(&mut self) -> PResult<Vec<Expr>> {
        if self.at(&TokKind::LParen) {
            self.bump();
            let args = self.parse_arg_list()?;
            self.expect(&TokKind::RParen, "`)`")?;
            Ok(args)
        } else {
            Ok(Vec::new())
        }
    }

    fn parse_struct_literal_body(&mut self, name: String) -> PResult<ExprKind> {
        self.expect(&TokKind::LBrace, "`{`")?;
        let saved = self.no_struct;
        self.no_struct = false;
        let mut fields = Vec::new();
        while !self.at(&TokKind::RBrace) {
            let flo = self.cur_start();
            let fname = self.expect_ident("a field name")?;
            self.expect(&TokKind::Colon, "`:`")?;
            let value = self.parse_or()?;
            fields.push(FieldInit {
                name: fname,
                value,
                span: self.span_from(flo),
            });
            if !self.eat(&TokKind::Comma) {
                break;
            }
        }
        self.no_struct = saved;
        self.expect(&TokKind::RBrace, "`}`")?;
        Ok(ExprKind::StructLit { name, fields })
    }

    fn parse_type_arg_intrinsic(&mut self, kind: IntrinsicKind) -> PResult<ExprKind> {
        self.bump(); // intrinsic keyword
        self.expect(&TokKind::LBracket, "`[`")?;
        let ty = self.parse_type()?;
        self.expect(&TokKind::RBracket, "`]`")?;
        self.expect(&TokKind::LParen, "`(`")?;
        let args = self.parse_arg_list()?;
        self.expect(&TokKind::RParen, "`)`")?;
        match kind {
            IntrinsicKind::PtrNull => {
                if !args.is_empty() {
                    return Err(Diag::error(
                        "P0004",
                        "`ptr_null[T]()` takes no arguments",
                        self.span_from(self.cur_start()),
                    ));
                }
                Ok(ExprKind::PtrNull { ty })
            }
            IntrinsicKind::CastPtr | IntrinsicKind::AddrToPtr => {
                let mut it = args.into_iter();
                let arg = it.next().ok_or_else(|| {
                    Diag::error(
                        "P0005",
                        "this intrinsic requires exactly one argument",
                        self.cur_span(),
                    )
                })?;
                Ok(match kind {
                    IntrinsicKind::CastPtr => ExprKind::CastPtr {
                        ty,
                        arg: Box::new(arg),
                    },
                    _ => ExprKind::AddrToPtr {
                        ty,
                        arg: Box::new(arg),
                    },
                })
            }
        }
    }

    fn parse_array_literal(&mut self) -> PResult<ExprKind> {
        self.expect(&TokKind::LBracket, "`[`")?;
        let saved = self.no_struct;
        self.no_struct = false;
        if self.at(&TokKind::RBracket) {
            self.no_struct = saved;
            self.bump();
            return Ok(ExprKind::ArrayLit(Vec::new()));
        }
        let first = self.parse_or()?;
        let out = if self.eat(&TokKind::Semi) {
            // `[e; N]` repeat form
            let size = self.parse_or()?;
            self.expect(&TokKind::RBracket, "`]`")?;
            ExprKind::ArrayRepeat {
                value: Box::new(first),
                size: Box::new(size),
            }
        } else {
            let mut elems = vec![first];
            while self.eat(&TokKind::Comma) {
                if self.at(&TokKind::RBracket) {
                    break;
                }
                elems.push(self.parse_or()?);
            }
            self.expect(&TokKind::RBracket, "`]`")?;
            ExprKind::ArrayLit(elems)
        };
        self.no_struct = saved;
        Ok(out)
    }

    fn parse_if(&mut self) -> PResult<ExprKind> {
        self.expect(&TokKind::Kw(Kw::If), "`if`")?;
        let cond = self.parse_expr_no_struct()?;
        let then_blk = self.parse_block()?;
        let else_blk = if self.eat_kw(Kw::Else) {
            let lo = self.cur_start();
            if self.at_kw(Kw::If) {
                let k = self.parse_if()?;
                Some(Box::new(Expr {
                    kind: k,
                    span: self.span_from(lo),
                }))
            } else {
                let b = self.parse_block()?;
                Some(Box::new(Expr {
                    kind: ExprKind::Block(b),
                    span: self.span_from(lo),
                }))
            }
        } else {
            None
        };
        Ok(ExprKind::If {
            cond: Box::new(cond),
            then_blk,
            else_blk,
        })
    }

    fn parse_match(&mut self) -> PResult<ExprKind> {
        self.expect(&TokKind::Kw(Kw::Match), "`match`")?;
        let scrutinee = self.parse_expr_no_struct()?;
        self.expect(&TokKind::LBrace, "`{`")?;
        let mut arms = Vec::new();
        while !self.at(&TokKind::RBrace) && !self.at(&TokKind::Eof) {
            let alo = self.cur_start();
            self.expect(&TokKind::Kw(Kw::Case), "`case`")?;
            let pattern = self.parse_pattern()?;
            self.expect(&TokKind::FatArrow, "`=>`")?;
            let body = self.parse_expr()?;
            arms.push(MatchArm {
                pattern,
                guard: None,
                body,
                span: self.span_from(alo),
            });
            self.eat(&TokKind::Comma); // optional arm separator
        }
        self.expect(&TokKind::RBrace, "`}`")?;
        Ok(ExprKind::Match {
            scrutinee: Box::new(scrutinee),
            arms,
        })
    }

    fn parse_pattern(&mut self) -> PResult<Pattern> {
        let lo = self.cur_start();
        let kind = match self.peek().clone() {
            TokKind::Kw(Kw::BoxResult) => {
                self.bump();
                self.expect(&TokKind::ColonColon, "`::`")?;
                let variant = self.expect_ident("a variant name")?;
                let sub = self.parse_opt_pattern_args()?;
                PatKind::Variant {
                    enum_name: "BoxResult".to_string(),
                    variant,
                    sub,
                }
            }
            TokKind::Ident(name) if name == "_" => {
                self.bump();
                PatKind::Wildcard
            }
            TokKind::Ident(name) => {
                if matches!(self.peek_at(1), TokKind::ColonColon) {
                    self.bump(); // name
                    self.bump(); // ::
                    let variant = self.expect_ident("a variant name")?;
                    let sub = self.parse_opt_pattern_args()?;
                    PatKind::Variant {
                        enum_name: name,
                        variant,
                        sub,
                    }
                } else {
                    self.bump();
                    PatKind::Binding(name)
                }
            }
            _ => return Err(self.unexpected("a pattern")),
        };
        Ok(Pattern {
            kind,
            span: self.span_from(lo),
        })
    }

    fn parse_opt_pattern_args(&mut self) -> PResult<Vec<Pattern>> {
        let mut sub = Vec::new();
        if self.eat(&TokKind::LParen) {
            while !self.at(&TokKind::RParen) {
                sub.push(self.parse_pattern()?);
                if !self.eat(&TokKind::Comma) {
                    break;
                }
            }
            self.expect(&TokKind::RParen, "`)`")?;
        }
        Ok(sub)
    }

    /// Does the current token begin an expression (used for `return` operand)?
    fn starts_expr(&self) -> bool {
        !matches!(
            self.peek(),
            TokKind::Semi
                | TokKind::Comma
                | TokKind::RParen
                | TokKind::RBrace
                | TokKind::RBracket
                | TokKind::Eof
        )
    }
}

enum IntrinsicKind {
    CastPtr,
    AddrToPtr,
    PtrNull,
}

fn is_block_like(kind: &ExprKind) -> bool {
    matches!(
        kind,
        ExprKind::Block(_)
            | ExprKind::If { .. }
            | ExprKind::Match { .. }
            | ExprKind::Loop(_)
            | ExprKind::While { .. }
            | ExprKind::Unsafe { .. }
            | ExprKind::Wrapping(_)
            | ExprKind::Saturating(_)
    )
}

fn describe(k: &TokKind) -> String {
    match k {
        TokKind::Eof => "end of input".to_string(),
        TokKind::Ident(s) => format!("identifier `{s}`"),
        TokKind::Int { .. } => "an integer literal".to_string(),
        TokKind::Str(_) => "a string literal".to_string(),
        other => format!("`{other:?}`"),
    }
}

/// Convenience wrapper: lex is done by the caller; parse a token stream.
pub fn parse(tokens: Vec<Token>) -> PResult<Program> {
    Parser::new(tokens).parse_program()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diag::Severity;
    use crate::lexer::lex;

    fn prog(src: &str) -> Program {
        let toks = lex(src).expect("lex ok");
        parse(toks).unwrap_or_else(|d| panic!("parse failed: {}", d.to_json()))
    }

    fn parse_err(src: &str) -> Diag {
        let toks = lex(src).expect("lex ok");
        parse(toks).expect_err("expected a parse error")
    }

    /// Parse a single expression by wrapping it in a trivial function body.
    fn expr(src: &str) -> Expr {
        let p = prog(&format!("fn f() -> unit {{ let x = {src}; }}"));
        match &p.items[0] {
            Item::Fn(f) => match &f.body.stmts[0].kind {
                StmtKind::Let { init: Some(e), .. } => e.clone(),
                _ => panic!("no let init"),
            },
            _ => panic!("not a fn"),
        }
    }

    fn only_fn(p: &Program) -> &FnDecl {
        match &p.items[0] {
            Item::Fn(f) => f,
            _ => panic!("expected fn"),
        }
    }

    #[test]
    fn struct_with_copy_and_fields() {
        let p = prog("copy struct Alloc { ctx: rawptr u8, vt: rawptr AllocVtable }");
        match &p.items[0] {
            Item::Struct(s) => {
                assert!(s.copy);
                assert_eq!(s.name, "Alloc");
                assert_eq!(s.fields.len(), 2);
                assert!(s.drop_hook.is_none());
            }
            _ => panic!(),
        }
    }

    #[test]
    fn struct_with_drop_hook() {
        let p = prog("struct R { fd: i32 } drop(write self) { close(read self); }");
        match &p.items[0] {
            Item::Struct(s) => assert!(s.drop_hook.is_some()),
            _ => panic!(),
        }
    }

    #[test]
    fn struct_field_named_alloc_is_ok() {
        // `alloc` is a contextual keyword; it is a legal field name (design §6.1).
        let p = prog("struct V { alloc: fn(ctx: rawptr u8) alloc -> rawptr u8 }");
        match &p.items[0] {
            Item::Struct(s) => {
                assert_eq!(s.fields[0].name, "alloc");
                match &s.fields[0].ty.kind {
                    TyKind::FnPtr(fp) => {
                        assert!(fp.alloc);
                        assert_eq!(fp.params.len(), 1);
                    }
                    _ => panic!("expected fn-ptr type"),
                }
            }
            _ => panic!(),
        }
    }

    #[test]
    fn enum_variants_with_and_without_payload() {
        let p = prog("enum Expr { num(i64), add(Box Expr, Box Expr), stop }");
        match &p.items[0] {
            Item::Enum(e) => {
                assert_eq!(e.variants.len(), 3);
                assert_eq!(e.variants[0].payload.len(), 1);
                assert_eq!(e.variants[1].payload.len(), 2);
                assert_eq!(e.variants[2].payload.len(), 0);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn fn_modes_effect_and_borrow_return() {
        let p = prog("fn g(a: read Alloc, s: slice u8, pos: write usize) alloc -> read Node { }");
        let f = only_fn(&p);
        assert!(f.alloc);
        assert_eq!(f.params[0].mode, ParamMode::Read);
        assert_eq!(f.params[1].mode, ParamMode::Take);
        assert_eq!(f.params[2].mode, ParamMode::Write);
        let ret = f.ret.as_ref().unwrap();
        assert_eq!(ret.borrow, Some(BorrowKind::Shared));
    }

    #[test]
    fn fn_region_variables_and_tagged_borrows() {
        let p = prog("fn pick[r](a: read[r] Slice, b: read Slice) -> read[r] Elem { }");
        let f = only_fn(&p);
        assert_eq!(f.regions, vec!["r".to_string()]);
        assert_eq!(f.params[0].region, Some("r".to_string()));
        assert_eq!(f.params[1].region, None);
        assert_eq!(f.ret.as_ref().unwrap().region, Some("r".to_string()));
    }

    #[test]
    fn fn_out_mode_and_contracts() {
        let p = prog("fn init(x: out usize) requires(true) ensures(result == 0) -> usize { }");
        let f = only_fn(&p);
        assert_eq!(f.params[0].mode, ParamMode::Out);
        assert_eq!(f.requires.len(), 1);
        assert_eq!(f.ensures.len(), 1);
        // `result` keyword parsed inside ensures
        assert!(matches!(
            &f.ensures[0].kind,
            ExprKind::Binary { .. }
        ));
    }

    #[test]
    fn static_item() {
        let p = prog("static V: AllocVtable = AllocVtable { alloc: pa, free: pf };");
        match &p.items[0] {
            Item::Static(s) => {
                assert_eq!(s.name, "V");
                assert!(matches!(s.value.kind, ExprKind::StructLit { .. }));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn box_array_type() {
        let p = prog("struct Arena { mem: Box [4096]Node, count: u32 }");
        match &p.items[0] {
            Item::Struct(s) => match &s.fields[0].ty.kind {
                TyKind::Box(inner) => assert!(matches!(inner.kind, TyKind::Array { .. })),
                _ => panic!("expected Box [N]T"),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn enum_ctor_with_and_without_args() {
        assert!(matches!(expr("Expr::num(n)").kind, ExprKind::EnumCtor { .. }));
        match expr("BoxResult::oom").kind {
            ExprKind::EnumCtor { enum_name, variant, args } => {
                assert_eq!(enum_name, "BoxResult");
                assert_eq!(variant, "oom");
                assert!(args.is_empty());
            }
            _ => panic!(),
        }
    }

    #[test]
    fn struct_literal_expr() {
        match expr("Pool { head: next, block_size: p.block_size }").kind {
            ExprKind::StructLit { name, fields } => {
                assert_eq!(name, "Pool");
                assert_eq!(fields.len(), 2);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn prefix_and_postfix_precedence() {
        // read applies to the whole postfix place `(deref ar).mem[...]`.
        match expr("read (deref ar).mem[conv usize (i)]").kind {
            ExprKind::Prefix { op: PrefixOp::Read, expr } => {
                assert!(matches!(expr.kind, ExprKind::Index { .. }));
            }
            _ => panic!("expected read of an index place"),
        }
    }

    #[test]
    fn binary_precedence_mul_over_add() {
        match expr("1 + 2 * 3").kind {
            ExprKind::Binary { op: BinOp::Add, rhs, .. } => {
                assert!(matches!(rhs.kind, ExprKind::Binary { op: BinOp::Mul, .. }));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn conv_form() {
        match expr("conv isize (offsetof(Task, link))").kind {
            ExprKind::Conv { ty, expr } => {
                assert!(matches!(ty.kind, TyKind::Scalar(_)));
                assert!(matches!(expr.kind, ExprKind::Offsetof { .. }));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn type_arg_intrinsics() {
        assert!(matches!(expr("cast_ptr[Pool](ctx)").kind, ExprKind::CastPtr { .. }));
        assert!(matches!(expr("addr_to_ptr[u32](base + off)").kind, ExprKind::AddrToPtr { .. }));
        assert!(matches!(expr("ptr_null[u8]()").kind, ExprKind::PtrNull { .. }));
        assert!(matches!(expr("sizeof(Node)").kind, ExprKind::Sizeof(_)));
        assert!(matches!(expr("alignof(Node)").kind, ExprKind::Alignof(_)));
    }

    #[test]
    fn array_literals() {
        assert!(matches!(expr("[1, 2, 3]").kind, ExprKind::ArrayLit(_)));
        assert!(matches!(expr("[0; 8]").kind, ExprKind::ArrayRepeat { .. }));
    }

    #[test]
    fn match_with_case_arms_and_wildcard() {
        let e = expr("match x { case Node::leaf(v) => v, case _ => 0 }");
        match e.kind {
            ExprKind::Match { arms, .. } => {
                assert_eq!(arms.len(), 2);
                assert!(matches!(arms[0].pattern.kind, PatKind::Variant { .. }));
                assert!(matches!(arms[1].pattern.kind, PatKind::Wildcard));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn nested_variant_pattern() {
        let e = expr("match x { case Wrap::a(Inner::b(y)) => y }");
        match e.kind {
            ExprKind::Match { arms, .. } => match &arms[0].pattern.kind {
                PatKind::Variant { sub, .. } => {
                    assert!(matches!(sub[0].kind, PatKind::Variant { .. }));
                }
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn unsafe_requires_justification() {
        assert!(matches!(
            expr("unsafe \"reason\" { }").kind,
            ExprKind::Unsafe { .. }
        ));
        let e = parse_err("fn f() -> unit { unsafe { } }");
        assert_eq!(e.code, "P0003");
    }

    #[test]
    fn wrapping_and_saturating_blocks() {
        assert!(matches!(expr("wrapping { }").kind, ExprKind::Wrapping(_)));
        assert!(matches!(expr("saturating { }").kind, ExprKind::Saturating(_)));
    }

    #[test]
    fn control_flow_statements() {
        let p = prog(
            "fn f() -> unit { \
               if cond { return; } else { } \
               while cond { break; } \
               loop { continue; } \
               assert(cond); \
               panic(\"boom\"); \
             }",
        );
        assert_eq!(only_fn(&p).body.stmts.len(), 5);
    }

    #[test]
    fn assignment_to_place() {
        let p = prog("fn f() -> unit { (deref d).state = s; }");
        assert!(matches!(
            only_fn(&p).body.stmts[0].kind,
            StmtKind::Assign { .. }
        ));
    }

    #[test]
    fn let_without_init() {
        let p = prog("fn f() -> unit { let x: usize; x = 3; }");
        match &only_fn(&p).body.stmts[0].kind {
            StmtKind::Let { init, mutable, .. } => {
                assert!(init.is_none());
                assert!(!mutable);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn match_used_as_statement_and_as_value() {
        // as a value (let init)
        assert!(matches!(expr("match e { case A::x => 1 }").kind, ExprKind::Match { .. }));
        // as a statement, with and without trailing `;`
        prog("fn f() -> unit { match e { case A::x => g() } }");
        prog("fn f() -> unit { match e { case A::x => g() }; }");
    }

    #[test]
    fn error_has_span_and_code() {
        let e = parse_err("fn f( { }");
        assert_eq!(e.severity, Severity::Error);
        assert!(!e.code.is_empty());
        assert!(e.span.end >= e.span.start);
    }
}
