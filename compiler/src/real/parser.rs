//! Recursive-descent parser for the real surface syntax (spec chapter 02),
//! producing the SAME `ast::Program` the throwaway front-end targets. Lossless
//! surface constructs are desugared here: `.*` -> `PrefixOp::Deref`, a borrow
//! keyword before a `[T]` slice -> `Slice`/`SliceMut` (matching the throwaway
//! pipeline), the negative-literal fold -> `NegIntLit`, and `read`/`write` before
//! a plain type in parameter position -> the corresponding mode. Genuinely new
//! semantics (`?`, bitwise ops, `ok`-marked variants) reach the shared AST via
//! its new nodes. No symbol table (NN#13).
//!
//! Documented surface choices (design 0006 §3 tensions resolved pragmatically):
//! * A `read`/`write` keyword immediately before a `[T]` slice denotes the
//!   slice's shared/exclusive-ness (Slice/SliceMut), NOT a mode over a slice —
//!   this keeps the desugared type identical to the throwaway `slice`/`slice_mut`
//!   forms so every downstream pass is shared. `read`/`write` before a plain type
//!   or a literal-sized array is the parameter mode (spec 02 §4.2).
//! * After a borrow keyword, `[<ident>] <type>` is read as region + type and
//!   `[<int>]T` as a literal-sized array borrow. A named-size array borrow
//!   (`read [CAP]Node`) is therefore not distinguishable from a region and parses
//!   as region + type — a recorded ambiguity in the spec's array/region overlap.

use crate::ast::*;
use crate::diag::Diag;
use crate::span::Span;
use super::token::{RKw, RTok, RToken};

pub struct RParser {
    tokens: Vec<RToken>,
    pos: usize,
    last_end: usize,
    no_struct: bool,
    /// Fresh-name counter for the `for`-loop desugar (design 0009 §4.2).
    for_ctr: usize,
    /// Module-layer side channel (design 0008): `use` decls collected during
    /// `parse_program`, discarded by the single-file entry point.
    mod_uses: Vec<UseDecl>,
    /// Visibility (`pub`) of each item, parallel to the returned `Program.items`.
    mod_vis: Vec<bool>,
    /// Whether this file opened with the `boundary` preamble (design 0008 §4).
    /// Recorded on every `fn`/`extern`/`export` so the placement and discharge
    /// rules of design 0011 survive the module merge.
    boundary: bool,
    /// Formatter mode: when set, `for` loops are kept as `ExprKind::For` surface
    /// nodes instead of being desugared (design 0009 §4.2), so the blessed
    /// formatter can reproduce the canonical `for` spelling (NN#11).
    preserve_for: bool,
}

type PResult<T> = Result<T, Diag>;

impl RParser {
    pub fn new(tokens: Vec<RToken>) -> RParser {
        RParser { tokens, pos: 0, last_end: 0, no_struct: false, for_ctr: 0, mod_uses: Vec::new(), mod_vis: Vec::new(), boundary: false, preserve_for: false }
    }

    // ----- cursor ----------------------------------------------------------
    fn peek(&self) -> &RTok {
        &self.tokens[self.pos].kind
    }
    fn peek_at(&self, n: usize) -> &RTok {
        let i = (self.pos + n).min(self.tokens.len() - 1);
        &self.tokens[i].kind
    }
    fn cur_span(&self) -> Span {
        self.tokens[self.pos].span
    }
    fn cur_start(&self) -> usize {
        self.tokens[self.pos].span.start
    }
    fn bump(&mut self) -> RToken {
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
    fn at(&self, k: &RTok) -> bool {
        self.peek() == k
    }
    fn at_kw(&self, kw: RKw) -> bool {
        matches!(self.peek(), RTok::Kw(k) if *k == kw)
    }
    fn at_ident(&self, name: &str) -> bool {
        matches!(self.peek(), RTok::Ident(s) if s == name)
    }
    fn eat(&mut self, k: &RTok) -> bool {
        if self.at(k) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn eat_kw(&mut self, kw: RKw) -> bool {
        if self.at_kw(kw) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn expect(&mut self, k: &RTok, what: &str) -> PResult<()> {
        if self.at(k) {
            self.bump();
            Ok(())
        } else {
            Err(self.unexpected(what))
        }
    }
    fn expect_ident(&mut self, what: &str) -> PResult<String> {
        match self.peek().clone() {
            RTok::Ident(s) => {
                self.bump();
                Ok(s)
            }
            _ => Err(self.unexpected(what)),
        }
    }
    fn unexpected(&self, what: &str) -> Diag {
        Diag::error("P0001", format!("expected {what}, found {}", describe(self.peek())), self.cur_span())
    }

    // ----- program & items -------------------------------------------------
    pub fn parse_program(&mut self) -> PResult<Program> {
        let mut items = Vec::new();
        // File-level `boundary` preamble (design 0008 §4): a contextual keyword
        // valid only as the very first item-preamble token of a file. It marks the
        // whole file (module) as a boundary module (the only place `extern`/
        // `export` may sit, design 0011 §1).
        if self.at_ident("boundary") {
            self.bump();
            self.boundary = true;
        }
        while !self.at(&RTok::Eof) {
            // Item-preamble `pub` (design 0008 §2) and `use` (§3) are contextual
            // keywords recognized only at item-leading position, so they stay
            // usable as ordinary identifiers elsewhere.
            let is_pub = self.at_ident("pub");
            if is_pub {
                self.bump();
            }
            if self.at_ident("use") {
                let u = self.parse_use()?;
                // `pub use` (module re-export, design 0008 §3) is beyond stage 1;
                // the visibility flag on an import is ignored here.
                self.mod_uses.push(u);
                continue;
            }
            items.push(self.parse_item()?);
            self.mod_vis.push(is_pub);
        }
        Ok(Program { items })
    }

    /// Parse a `use` import declaration (design 0008 §3). The path separator is
    /// `::`; one-token lookahead after `::` decides the form — `{` opens a group
    /// import, an identifier continues the module path.
    fn parse_use(&mut self) -> PResult<UseDecl> {
        let lo = self.cur_start();
        self.bump(); // `use`
        let mut segments = vec![self.expect_ident("a module path segment")?];
        let mut names: Option<Vec<String>> = None;
        while self.eat(&RTok::ColonColon) {
            if self.at(&RTok::LBrace) {
                self.bump();
                let mut ns = Vec::new();
                while !self.at(&RTok::RBrace) {
                    ns.push(self.expect_ident("an imported item name")?);
                    if !self.eat(&RTok::Comma) {
                        break;
                    }
                }
                self.expect(&RTok::RBrace, "`}`")?;
                names = Some(ns);
                break;
            }
            segments.push(self.expect_ident("a module path segment")?);
        }
        self.expect(&RTok::Semi, "`;`")?;
        Ok(UseDecl { segments, names, span: self.span_from(lo) })
    }

    fn parse_item(&mut self) -> PResult<Item> {
        let copy = self.eat_kw(RKw::Copy);
        if !copy && self.at_ident("interface") {
            return Ok(Item::Interface(self.parse_interface()?));
        }
        if !copy && self.at_ident("impl") {
            return Ok(Item::Impl(self.parse_impl()?));
        }
        if !copy && self.at_ident("extern") {
            return Ok(Item::Extern(self.parse_extern()?));
        }
        if !copy && self.at_ident("export") {
            return Ok(Item::Export(self.parse_export()?));
        }
        match self.peek() {
            RTok::Kw(RKw::Struct) => Ok(Item::Struct(self.parse_struct(copy)?)),
            RTok::Kw(RKw::Enum) => Ok(Item::Enum(self.parse_enum(copy)?)),
            _ if copy => Err(Diag::error("P0002", "`copy` may only precede `struct` or `enum`", self.cur_span())),
            RTok::Kw(RKw::Fn) => Ok(Item::Fn(self.parse_fn()?)),
            RTok::Kw(RKw::Static) => Ok(Item::Static(self.parse_static()?)),
            _ => Err(self.unexpected("an item (`struct`, `enum`, `fn`, `static`, `interface`, `impl`)")),
        }
    }

    /// Parse a declaration bracket after an item name (design 0007 §6.1.1): a
    /// mixed list of `region r` region variables and bare/bounded type parameters.
    /// Returns `(regions, type_params)`. Absent bracket -> both empty.
    fn parse_decl_brackets(&mut self) -> PResult<(Vec<String>, Vec<TypeParam>)> {
        let mut regions = Vec::new();
        let mut tparams = Vec::new();
        if self.eat(&RTok::LBracket) {
            while !self.at(&RTok::RBracket) {
                if self.at_ident("region") {
                    self.bump();
                    regions.push(self.expect_ident("a region variable")?);
                } else {
                    let plo = self.cur_start();
                    let name = self.expect_ident("a type parameter")?;
                    let mut bounds = Vec::new();
                    if self.eat(&RTok::Colon) {
                        loop {
                            // A bound is an interface name or the built-in `copy`.
                            if self.at_kw(RKw::Copy) {
                                self.bump();
                                bounds.push("copy".to_string());
                            } else {
                                bounds.push(self.expect_ident("a bound (interface name or `copy`)")?);
                            }
                            if !self.eat(&RTok::Plus) {
                                break;
                            }
                        }
                    }
                    tparams.push(TypeParam { name, bounds, span: self.span_from(plo) });
                }
                if !self.eat(&RTok::Comma) {
                    break;
                }
            }
            self.expect(&RTok::RBracket, "`]`")?;
        }
        Ok((regions, tparams))
    }

    fn parse_interface(&mut self) -> PResult<InterfaceDecl> {
        let lo = self.cur_start();
        self.bump(); // `interface`
        let name = self.expect_ident("an interface name")?;
        let (_regions, type_params) = self.parse_decl_brackets()?;
        self.expect(&RTok::LBrace, "`{`")?;
        let mut methods = Vec::new();
        let mut assoc_type: Option<String> = None;
        while !self.at(&RTok::RBrace) && !self.at(&RTok::Eof) {
            // Contextual `type Ident;` associated-type member (design 0009 §4.4):
            // `type` is a keyword only in an interface/impl body.
            if self.at_ident("type") {
                self.bump();
                let aname = self.expect_ident("an associated type name")?;
                self.expect(&RTok::Semi, "`;`")?;
                if assoc_type.is_some() {
                    return Err(Diag::error(
                        "P0009",
                        "an interface declares at most one associated type (design 0009 §2.3)",
                        self.cur_span(),
                    ));
                }
                assoc_type = Some(aname);
                continue;
            }
            methods.push(self.parse_method_sig()?);
        }
        self.expect(&RTok::RBrace, "`}`")?;
        Ok(InterfaceDecl { name, type_params, assoc_type, methods, span: self.span_from(lo) })
    }

    /// A method *signature* line inside an interface: `fn m(SELF, params) TAIL ->
    /// ret ;` where SELF is `read self` / `write self` / `take self` / `self`.
    fn parse_method_sig(&mut self) -> PResult<MethodSig> {
        let lo = self.cur_start();
        self.expect(&RTok::Kw(RKw::Fn), "`fn`")?;
        let name = self.expect_ident("a method name")?;
        self.expect(&RTok::LParen, "`(`")?;
        let (has_self, self_mode, mut params) = self.parse_method_params()?;
        self.expect(&RTok::RParen, "`)`")?;
        let mut alloc = false;
        if self.at_ident("alloc") {
            self.bump();
            alloc = true;
        }
        let ret = if self.eat(&RTok::Arrow) {
            Some(self.parse_ret_ty()?)
        } else {
            None
        };
        self.expect(&RTok::Semi, "`;`")?;
        let _ = &mut params;
        Ok(MethodSig { name, has_self, self_mode, params, alloc, ret, span: self.span_from(lo) })
    }

    /// Parse a method's parameter list, detecting an optional `self` receiver at
    /// its head (design 0007 §3.5; a `self`-less method is an associated function,
    /// e.g. `From::from`, §7.1). Returns `(has_self, self_mode, non_self_params)`.
    fn parse_method_params(&mut self) -> PResult<(bool, ParamMode, Vec<Param>)> {
        // `self` receiver forms: `self`, `read self`, `write self`, `take self`.
        let is_self_head = matches!(self.peek(), RTok::Kw(RKw::SelfKw))
            || (matches!(self.peek(), RTok::Kw(RKw::Read) | RTok::Kw(RKw::Write) | RTok::Kw(RKw::Take))
                && matches!(self.peek_at(1), RTok::Kw(RKw::SelfKw)));
        let mut params = Vec::new();
        let (has_self, self_mode) = if is_self_head {
            let mode = match self.peek() {
                RTok::Kw(RKw::Read) => { self.bump(); ParamMode::Read }
                RTok::Kw(RKw::Write) => { self.bump(); ParamMode::Write }
                RTok::Kw(RKw::Take) => { self.bump(); ParamMode::Take }
                _ => ParamMode::Take,
            };
            self.expect(&RTok::Kw(RKw::SelfKw), "`self`")?;
            while self.eat(&RTok::Comma) {
                if self.at(&RTok::RParen) { break; }
                params.push(self.parse_param()?);
            }
            (true, mode)
        } else {
            while !self.at(&RTok::RParen) {
                params.push(self.parse_param()?);
                if !self.eat(&RTok::Comma) { break; }
            }
            (false, ParamMode::Take)
        };
        Ok((has_self, self_mode, params))
    }

    fn parse_impl(&mut self) -> PResult<ImplDecl> {
        let lo = self.cur_start();
        self.bump(); // `impl`
        let (_regions, type_params) = self.parse_decl_brackets()?;
        let iface = self.expect_ident("an interface name")?;
        let mut iface_args = Vec::new();
        if self.eat(&RTok::LBracket) {
            while !self.at(&RTok::RBracket) {
                iface_args.push(self.parse_type()?);
                if !self.eat(&RTok::Comma) {
                    break;
                }
            }
            self.expect(&RTok::RBracket, "`]`")?;
        }
        if !self.at_ident("for") {
            return Err(self.unexpected("`for` in an impl header"));
        }
        self.bump(); // `for`
        let target = self.parse_type()?;
        self.expect(&RTok::LBrace, "`{`")?;
        let mut methods = Vec::new();
        let mut assoc_binding: Option<(String, Ty)> = None;
        while !self.at(&RTok::RBrace) && !self.at(&RTok::Eof) {
            // Contextual `type Ident = Type;` associated-type binding (§4.4).
            if self.at_ident("type") {
                self.bump();
                let aname = self.expect_ident("an associated type name")?;
                self.expect(&RTok::Eq, "`=`")?;
                let bty = self.parse_type()?;
                self.expect(&RTok::Semi, "`;`")?;
                assoc_binding = Some((aname, bty));
                continue;
            }
            methods.push(self.parse_impl_method()?);
        }
        self.expect(&RTok::RBrace, "`}`")?;
        Ok(ImplDecl { type_params, iface, iface_args, target, assoc_binding, methods, home: None, span: self.span_from(lo) })
    }

    /// An impl method: like a free `fn` but its first parameter is a `self`
    /// receiver. Lowered to a `FnDecl` whose first param is named `self`.
    fn parse_impl_method(&mut self) -> PResult<FnDecl> {
        let lo = self.cur_start();
        self.expect(&RTok::Kw(RKw::Fn), "`fn`")?;
        let name = self.expect_ident("a method name")?;
        self.expect(&RTok::LParen, "`(`")?;
        let slo = self.cur_start();
        let (has_self, self_mode, rest) = self.parse_method_params()?;
        let mut params = Vec::new();
        if has_self {
            params.push(Param {
                name: "self".to_string(),
                mode: self_mode,
                region: None,
                // `Self` placeholder; substituted by the generics lowering pass.
                ty: Ty { kind: TyKind::Named("Self".to_string()), span: self.span_from(slo) },
                span: self.span_from(slo),
            });
        }
        params.extend(rest);
        self.expect(&RTok::RParen, "`)`")?;
        let mut alloc = false;
        if self.at_ident("alloc") {
            self.bump();
            alloc = true;
        }
        let ret = if self.eat(&RTok::Arrow) {
            Some(self.parse_ret_ty()?)
        } else {
            None
        };
        let body = self.parse_block()?;
        Ok(FnDecl {
            name,
            type_params: Vec::new(),
            regions: Vec::new(),
            params,
            alloc,
            foreign: false,
            boundary: self.boundary,
            requires: Vec::new(),
            ensures: Vec::new(),
            ret,
            body,
            span: self.span_from(lo),
        })
    }

    /// Parse an `extern "<abi>" { <extern fns> }` block (design 0011 §1). The
    /// `boundary`-file status is captured so the placement rule (E1101) survives
    /// the module merge. Parsing succeeds anywhere; ill placement is a checker
    /// diagnostic, not a parse error, so the message is a P4 semantic one.
    fn parse_extern(&mut self) -> PResult<ExternBlock> {
        let lo = self.cur_start();
        self.bump(); // `extern`
        let abi = self.expect_abi_string()?;
        self.expect(&RTok::LBrace, "`{`")?;
        let mut fns = Vec::new();
        while !self.at(&RTok::RBrace) && !self.at(&RTok::Eof) {
            fns.push(self.parse_extern_fn()?);
        }
        self.expect(&RTok::RBrace, "`}`")?;
        Ok(ExternBlock { abi, boundary_file: self.boundary, fns, span: self.span_from(lo) })
    }

    /// One `fn name(params) foreign -> ret trust "..." { .. };` inside an extern
    /// block. The `foreign` effect marker is optional in the surface (an extern is
    /// implicitly foreign); the trailing `trust` clause is optional.
    fn parse_extern_fn(&mut self) -> PResult<ExternFn> {
        let lo = self.cur_start();
        self.expect(&RTok::Kw(RKw::Fn), "`fn`")?;
        let name = self.expect_ident("a foreign function name")?;
        self.expect(&RTok::LParen, "`(`")?;
        let mut params = Vec::new();
        while !self.at(&RTok::RParen) {
            params.push(self.parse_param()?);
            if !self.eat(&RTok::Comma) {
                break;
            }
        }
        self.expect(&RTok::RParen, "`)`")?;
        // The implicit `foreign` marker (accepted, not required — an extern is a
        // ground source of the effect regardless).
        let _ = self.at_ident("foreign") && { self.bump(); true };
        let ret = if self.eat(&RTok::Arrow) { Some(self.parse_ret_ty()?) } else { None };
        let trust = if self.at_ident("trust") { Some(self.parse_trust()?) } else { None };
        self.expect(&RTok::Semi, "`;`")?;
        Ok(ExternFn { name, params, ret, trust, span: self.span_from(lo) })
    }

    /// Parse a `trust "justification" { pred(args), .. }` clause (design 0011 §3).
    fn parse_trust(&mut self) -> PResult<TrustDecl> {
        let lo = self.cur_start();
        self.bump(); // `trust`
        let justification = match self.peek().clone() {
            RTok::Str(s) => { self.bump(); s }
            _ => return Err(self.unexpected("a trust justification string")),
        };
        self.expect(&RTok::LBrace, "`{`")?;
        let mut predicates = Vec::new();
        while !self.at(&RTok::RBrace) && !self.at(&RTok::Eof) {
            let plo = self.cur_start();
            let name = self.expect_ident("a trust predicate name")?;
            let mut args = Vec::new();
            if self.eat(&RTok::LParen) {
                while !self.at(&RTok::RParen) {
                    args.push(self.expect_ident("a predicate argument")?);
                    if !self.eat(&RTok::Comma) {
                        break;
                    }
                }
                self.expect(&RTok::RParen, "`)`")?;
            }
            predicates.push(TrustPred { name, args, span: self.span_from(plo) });
            if !self.eat(&RTok::Comma) {
                break;
            }
        }
        self.expect(&RTok::RBrace, "`}`")?;
        Ok(TrustDecl { justification, predicates, span: self.span_from(lo) })
    }

    /// Parse an `export "<abi>" fn <symbol>(params) -> ret = <candor_fn>;` item
    /// (design 0011 §1.5).
    fn parse_export(&mut self) -> PResult<ExportDecl> {
        let lo = self.cur_start();
        self.bump(); // `export`
        let abi = self.expect_abi_string()?;
        self.expect(&RTok::Kw(RKw::Fn), "`fn`")?;
        let symbol = self.expect_ident("an export C symbol name")?;
        self.expect(&RTok::LParen, "`(`")?;
        let mut params = Vec::new();
        while !self.at(&RTok::RParen) {
            params.push(self.parse_param()?);
            if !self.eat(&RTok::Comma) {
                break;
            }
        }
        self.expect(&RTok::RParen, "`)`")?;
        let ret = if self.eat(&RTok::Arrow) { Some(self.parse_ret_ty()?) } else { None };
        self.expect(&RTok::Eq, "`=`")?;
        let candor_fn = self.expect_ident("the exported Candor function name")?;
        self.expect(&RTok::Semi, "`;`")?;
        Ok(ExportDecl { abi, boundary_file: self.boundary, symbol, params, ret, candor_fn, span: self.span_from(lo) })
    }

    /// An ABI string after `extern`/`export` (`"C"`). Only a string literal.
    fn expect_abi_string(&mut self) -> PResult<String> {
        match self.peek().clone() {
            RTok::Str(s) => { self.bump(); Ok(s) }
            _ => Err(self.unexpected("an ABI string, e.g. `\"C\"`")),
        }
    }

    fn parse_struct(&mut self, copy: bool) -> PResult<StructDecl> {
        let lo = self.cur_start();
        self.expect(&RTok::Kw(RKw::Struct), "`struct`")?;
        let name = self.expect_ident("a struct name")?;
        let (_regions, type_params) = self.parse_decl_brackets()?;
        self.expect(&RTok::LBrace, "`{`")?;
        let mut fields = Vec::new();
        while !self.at(&RTok::RBrace) {
            let flo = self.cur_start();
            let fname = self.expect_ident("a field name")?;
            self.expect(&RTok::Colon, "`:`")?;
            let ty = self.parse_type()?;
            fields.push(Field { name: fname, ty, span: self.span_from(flo) });
            if !self.eat(&RTok::Comma) {
                break;
            }
        }
        self.expect(&RTok::RBrace, "`}`")?;
        let drop_hook = if self.at_kw(RKw::Drop) {
            Some(self.parse_drop_hook()?)
        } else {
            None
        };
        Ok(StructDecl { copy, name, type_params, fields, drop_hook, span: self.span_from(lo) })
    }

    fn parse_drop_hook(&mut self) -> PResult<Block> {
        self.expect(&RTok::Kw(RKw::Drop), "`drop`")?;
        self.expect(&RTok::LParen, "`(`")?;
        self.expect(&RTok::Kw(RKw::Write), "`write`")?;
        self.expect(&RTok::Kw(RKw::SelfKw), "`self`")?;
        self.expect(&RTok::RParen, "`)`")?;
        self.parse_block()
    }

    fn parse_enum(&mut self, copy: bool) -> PResult<EnumDecl> {
        let lo = self.cur_start();
        self.expect(&RTok::Kw(RKw::Enum), "`enum`")?;
        let name = self.expect_ident("an enum name")?;
        let (_regions, type_params) = self.parse_decl_brackets()?;
        self.expect(&RTok::LBrace, "`{`")?;
        let mut variants = Vec::new();
        while !self.at(&RTok::RBrace) {
            let vlo = self.cur_start();
            // Contextual `ok` marker in variant-leading position (spec 02 §2.2):
            // `ok` here is a keyword; elsewhere it is an identifier. Recognized
            // only when followed by a variant name.
            let ok = self.at_ident("ok") && matches!(self.peek_at(1), RTok::Ident(_));
            if ok {
                self.bump();
            }
            let vname = self.expect_ident("a variant name")?;
            let mut payload = Vec::new();
            if self.eat(&RTok::LParen) {
                while !self.at(&RTok::RParen) {
                    payload.push(self.parse_type()?);
                    if !self.eat(&RTok::Comma) {
                        break;
                    }
                }
                self.expect(&RTok::RParen, "`)`")?;
            }
            variants.push(Variant { name: vname, payload, ok, span: self.span_from(vlo) });
            if !self.eat(&RTok::Comma) {
                break;
            }
        }
        self.expect(&RTok::RBrace, "`}`")?;
        Ok(EnumDecl { copy, name, type_params, variants, span: self.span_from(lo) })
    }

    fn parse_static(&mut self) -> PResult<StaticDecl> {
        let lo = self.cur_start();
        self.expect(&RTok::Kw(RKw::Static), "`static`")?;
        let name = self.expect_ident("a static name")?;
        self.expect(&RTok::Colon, "`:`")?;
        let ty = self.parse_type()?;
        self.expect(&RTok::Eq, "`=`")?;
        let value = self.parse_expr()?;
        self.expect(&RTok::Semi, "`;`")?;
        Ok(StaticDecl { name, ty, value, span: self.span_from(lo) })
    }

    fn parse_fn(&mut self) -> PResult<FnDecl> {
        let lo = self.cur_start();
        self.expect(&RTok::Kw(RKw::Fn), "`fn`")?;
        let name = self.expect_ident("a function name")?;
        let (regions, type_params) = self.parse_decl_brackets()?;
        self.expect(&RTok::LParen, "`(`")?;
        let mut params = Vec::new();
        while !self.at(&RTok::RParen) {
            params.push(self.parse_param()?);
            if !self.eat(&RTok::Comma) {
                break;
            }
        }
        self.expect(&RTok::RParen, "`)`")?;

        // SigTail: `alloc`/`foreign` (contextual) and contract clauses, any order.
        let mut alloc = false;
        let mut foreign = false;
        let mut requires = Vec::new();
        let mut ensures = Vec::new();
        loop {
            if self.at_ident("alloc") {
                self.bump();
                alloc = true;
            } else if self.at_ident("foreign") {
                self.bump();
                foreign = true;
            } else if self.eat_kw(RKw::Requires) {
                self.expect(&RTok::LParen, "`(`")?;
                requires.push(self.parse_delimited_expr()?);
                self.expect(&RTok::RParen, "`)`")?;
            } else if self.eat_kw(RKw::Ensures) {
                self.expect(&RTok::LParen, "`(`")?;
                ensures.push(self.parse_delimited_expr()?);
                self.expect(&RTok::RParen, "`)`")?;
            } else {
                break;
            }
        }

        let ret = if self.eat(&RTok::Arrow) {
            Some(self.parse_ret_ty()?)
        } else {
            None
        };
        let body = self.parse_block()?;
        Ok(FnDecl { name, type_params, regions, params, alloc, foreign, boundary: self.boundary, requires, ensures, ret, body, span: self.span_from(lo) })
    }

    fn parse_param(&mut self) -> PResult<Param> {
        let lo = self.cur_start();
        let name = self.expect_ident("a parameter name")?;
        self.expect(&RTok::Colon, "`:`")?;
        let (mode, region, ty) = self.parse_mode_and_type()?;
        Ok(Param { name, mode, region, ty, span: self.span_from(lo) })
    }

    /// Parse a parameter's `Mode? Type` (spec 02 §4). See the module note: a
    /// borrow keyword before a `[T]` slice becomes a Slice/SliceMut type in
    /// `take` mode; before a plain type or literal-array it is the mode.
    fn parse_mode_and_type(&mut self) -> PResult<(ParamMode, Option<String>, Ty)> {
        match self.peek() {
            RTok::Kw(RKw::Take) => {
                self.bump();
                Ok((ParamMode::Take, None, self.parse_type()?))
            }
            RTok::Kw(RKw::Out) => {
                self.bump();
                Ok((ParamMode::Out, None, self.parse_type()?))
            }
            RTok::Kw(RKw::Read) | RTok::Kw(RKw::Write) => {
                let excl = self.at_kw(RKw::Write);
                self.bump();
                let (region, ty) = self.parse_borrow_body(excl)?;
                match &ty.kind {
                    // A slice keyword-form is passed by value in `take` mode.
                    TyKind::Slice(_) | TyKind::SliceMut(_) => Ok((ParamMode::Take, region, ty)),
                    // A single-place / array borrow keeps the keyword as the mode
                    // over the pointee type (unwrap the borrow layer).
                    TyKind::Borrow(inner) => {
                        Ok((ParamMode::Read, region, (**inner).clone()))
                    }
                    TyKind::BorrowMut(inner) => {
                        Ok((ParamMode::Write, region, (**inner).clone()))
                    }
                    _ => Ok((if excl { ParamMode::Write } else { ParamMode::Read }, region, ty)),
                }
            }
            _ => Ok((ParamMode::Take, None, self.parse_type()?)),
        }
    }

    fn parse_ret_ty(&mut self) -> PResult<RetTy> {
        let lo = self.cur_start();
        if matches!(self.peek(), RTok::Kw(RKw::Read) | RTok::Kw(RKw::Write)) {
            let excl = self.at_kw(RKw::Write);
            self.bump();
            let (region, ty) = self.parse_borrow_body(excl)?;
            let (borrow, region, ty) = match ty.kind {
                // A returned slice is already a borrow value: no extra borrow
                // wrapper, the region rides on the return.
                TyKind::Slice(_) | TyKind::SliceMut(_) => (None, region, ty),
                TyKind::Borrow(inner) => (Some(BorrowKind::Shared), region, *inner),
                TyKind::BorrowMut(inner) => (Some(BorrowKind::Exclusive), region, *inner),
                _ => (
                    Some(if excl { BorrowKind::Exclusive } else { BorrowKind::Shared }),
                    region,
                    ty,
                ),
            };
            Ok(RetTy { borrow, region, ty, span: self.span_from(lo) })
        } else {
            let ty = self.parse_type()?;
            Ok(RetTy { borrow: None, region: None, ty, span: self.span_from(lo) })
        }
    }

    // ----- types -----------------------------------------------------------
    fn parse_type(&mut self) -> PResult<Ty> {
        let lo = self.cur_start();
        let kind = match self.peek().clone() {
            RTok::Scalar(sc) => {
                self.bump();
                TyKind::Scalar(sc)
            }
            RTok::Kw(RKw::RawPtr) => {
                self.bump();
                TyKind::RawPtr(Box::new(self.parse_type()?))
            }
            RTok::Kw(RKw::Box) => {
                self.bump();
                TyKind::Box(Box::new(self.parse_type()?))
            }
            RTok::Kw(RKw::BoxResult) => {
                self.bump();
                TyKind::BoxResult(Box::new(self.parse_type()?))
            }
            RTok::Kw(RKw::Read) | RTok::Kw(RKw::Write) => {
                // Borrow type in type position (return/local annotation). Region,
                // if any, is not represented on a bare `Ty` and is dropped here.
                let excl = self.at_kw(RKw::Write);
                self.bump();
                let (_region, ty) = self.parse_borrow_body(excl)?;
                ty.kind
            }
            RTok::Kw(RKw::Fn) => TyKind::FnPtr(self.parse_fnptr_type()?),
            RTok::LBracket => self.parse_bracket_type()?,
            RTok::Ident(name) => {
                self.bump();
                // `Base::Assoc` in type position is an associated-type projection
                // (design 0009 §2.2): a path whose head is a type-name token and
                // whose final segment names the member. NN#13-clean (§4.4).
                if self.at(&RTok::ColonColon) {
                    self.bump();
                    let assoc = self.expect_ident("an associated type name")?;
                    return Ok(Ty { kind: TyKind::Proj { base: name, assoc }, span: self.span_from(lo) });
                }
                // A bracket *following* a type-name is a type-argument list
                // (design 0007 §6.1.1 use-rule): `List[i64]`, `Pair[T]`.
                if self.at(&RTok::LBracket) {
                    self.bump();
                    let mut args = Vec::new();
                    while !self.at(&RTok::RBracket) {
                        args.push(self.parse_type()?);
                        if !self.eat(&RTok::Comma) {
                            break;
                        }
                    }
                    self.expect(&RTok::RBracket, "`]`")?;
                    TyKind::App { name, args }
                } else {
                    TyKind::Named(name)
                }
            }
            RTok::Kw(RKw::SelfKw) => {
                self.bump();
                if self.at(&RTok::ColonColon) {
                    self.bump();
                    let assoc = self.expect_ident("an associated type name")?;
                    TyKind::Proj { base: "Self".to_string(), assoc }
                } else {
                    TyKind::Named("Self".to_string())
                }
            }
            _ => return Err(self.unexpected("a type")),
        };
        Ok(Ty { kind, span: self.span_from(lo) })
    }

    /// `[N]T` array vs `[T]` slice after a leading `[` (spec 02 §3.3): parse one
    /// component; if a type follows the `]`, it was a size (array), else a slice.
    fn parse_bracket_type(&mut self) -> PResult<TyKind> {
        self.expect(&RTok::LBracket, "`[`")?;
        // Peek: an INT is unambiguously a size; otherwise parse a type and decide
        // on what follows the `]`.
        if let RTok::Int { value, suffix } = self.peek().clone() {
            let slo = self.cur_start();
            self.bump();
            let size = Expr { kind: ExprKind::IntLit { value, suffix }, span: self.span_from(slo) };
            self.expect(&RTok::RBracket, "`]`")?;
            let elem = self.parse_type()?;
            return Ok(TyKind::Array { size: Box::new(size), elem: Box::new(elem) });
        }
        // An identifier immediately closed by `]` and followed by a type-start is
        // a named array size `[N]T`; otherwise the bracket held a slice element.
        if let RTok::Ident(n) = self.peek().clone() {
            if matches!(self.peek_at(1), RTok::RBracket) && starts_type(self.peek_at(2)) {
                let slo = self.cur_start();
                self.bump();
                let size = Expr { kind: ExprKind::Ident(n), span: self.span_from(slo) };
                self.expect(&RTok::RBracket, "`]`")?;
                let elem = self.parse_type()?;
                return Ok(TyKind::Array { size: Box::new(size), elem: Box::new(elem) });
            }
        }
        let elem = self.parse_type()?;
        self.expect(&RTok::RBracket, "`]`")?;
        Ok(TyKind::Slice(Box::new(elem)))
    }

    /// After a consumed `read`/`write` keyword, parse the region tag (if any) and
    /// the borrowed type, returning the full borrow `Ty` (Borrow/BorrowMut for a
    /// single place or array, Slice/SliceMut for a slice keyword-form).
    fn parse_borrow_body(&mut self, excl: bool) -> PResult<(Option<String>, Ty)> {
        let lo = self.cur_start();
        if self.at(&RTok::LBracket) {
            // Could be: region `[r]` (then a type or slice), a literal array size
            // `[N]T`, or a slice `[T]`.
            self.bump(); // `[`
            if let RTok::Int { value, suffix } = self.peek().clone() {
                // `[N]T` literal-sized array borrow.
                let slo = self.cur_start();
                self.bump();
                let size = Expr { kind: ExprKind::IntLit { value, suffix }, span: self.span_from(slo) };
                self.expect(&RTok::RBracket, "`]`")?;
                let elem = self.parse_type()?;
                let inner = Ty {
                    kind: TyKind::Array { size: Box::new(size), elem: Box::new(elem) },
                    span: self.span_from(lo),
                };
                return Ok((None, self.wrap_borrow(excl, inner, lo)));
            }
            if let RTok::Ident(id) = self.peek().clone() {
                if matches!(self.peek_at(1), RTok::RBracket) {
                    if matches!(self.peek_at(2), RTok::LBracket) {
                        // region + slice: `read[r] [T]`
                        self.bump(); // ident (region)
                        self.expect(&RTok::RBracket, "`]`")?;
                        self.expect(&RTok::LBracket, "`[`")?;
                        let elem = self.parse_type()?;
                        self.expect(&RTok::RBracket, "`]`")?;
                        let sl = Ty {
                            kind: if excl {
                                TyKind::SliceMut(Box::new(elem))
                            } else {
                                TyKind::Slice(Box::new(elem))
                            },
                            span: self.span_from(lo),
                        };
                        return Ok((Some(id), sl));
                    }
                    if starts_type(self.peek_at(2)) {
                        // region + plain type: `read[r] T`
                        self.bump(); // ident (region)
                        self.expect(&RTok::RBracket, "`]`")?;
                        let t = self.parse_type()?;
                        return Ok((Some(id), self.wrap_borrow(excl, t, lo)));
                    }
                }
            }
            // Otherwise the bracket held a slice element type: `read [T]`.
            let elem = self.parse_type()?;
            self.expect(&RTok::RBracket, "`]`")?;
            let sl = Ty {
                kind: if excl {
                    TyKind::SliceMut(Box::new(elem))
                } else {
                    TyKind::Slice(Box::new(elem))
                },
                span: self.span_from(lo),
            };
            Ok((None, sl))
        } else {
            // Plain single-place borrow: `read T` / `write T`.
            let t = self.parse_type()?;
            Ok((None, self.wrap_borrow(excl, t, lo)))
        }
    }

    fn wrap_borrow(&self, excl: bool, inner: Ty, lo: usize) -> Ty {
        Ty {
            kind: if excl {
                TyKind::BorrowMut(Box::new(inner))
            } else {
                TyKind::Borrow(Box::new(inner))
            },
            span: self.span_from(lo),
        }
    }

    fn parse_fnptr_type(&mut self) -> PResult<FnPtrTy> {
        self.expect(&RTok::Kw(RKw::Fn), "`fn`")?;
        self.expect(&RTok::LParen, "`(`")?;
        let mut params = Vec::new();
        while !self.at(&RTok::RParen) {
            params.push(self.parse_fnptr_param()?);
            if !self.eat(&RTok::Comma) {
                break;
            }
        }
        self.expect(&RTok::RParen, "`)`")?;
        // Effect markers in canonical order `alloc foreign`, either optional.
        let alloc = if self.at_ident("alloc") { self.bump(); true } else { false };
        let foreign = if self.at_ident("foreign") { self.bump(); true } else { false };
        self.expect(&RTok::Arrow, "`->`")?;
        let ret = self.parse_type()?;
        Ok(FnPtrTy { params, alloc, foreign, ret: Box::new(ret) })
    }

    fn parse_fnptr_param(&mut self) -> PResult<FnPtrParam> {
        let name = if matches!(self.peek(), RTok::Ident(_)) && matches!(self.peek_at(1), RTok::Colon) {
            let n = self.expect_ident("a parameter name")?;
            self.bump(); // `:`
            Some(n)
        } else {
            None
        };
        let (mode, region, ty) = self.parse_mode_and_type()?;
        Ok(FnPtrParam { name, mode, region, ty })
    }

    // ----- blocks & statements --------------------------------------------
    fn parse_block(&mut self) -> PResult<Block> {
        let lo = self.cur_start();
        self.expect(&RTok::LBrace, "`{`")?;
        let mut stmts = Vec::new();
        while !self.at(&RTok::RBrace) && !self.at(&RTok::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&RTok::RBrace, "`}`")?;
        Ok(Block { stmts, span: self.span_from(lo) })
    }

    fn parse_stmt(&mut self) -> PResult<Stmt> {
        let lo = self.cur_start();
        if self.at_kw(RKw::Let) {
            return self.parse_let(lo);
        }
        // `for` is a contextual keyword only in statement-leading position
        // (design 0009 §4.4): a statement beginning with `for` is the loop.
        if self.at_ident("for") {
            return self.parse_for(lo);
        }
        // `scope { ... }` — a concurrency region (design 0012 §1.1, §5). Contextual:
        // `scope` is a keyword only in statement-leading position immediately
        // followed by `{`; elsewhere an ordinary identifier.
        if self.at_ident("scope") && matches!(self.peek_at(1), RTok::LBrace) {
            self.bump(); // `scope`
            let body = self.parse_block()?;
            let e = Expr { kind: ExprKind::Scope(body), span: self.span_from(lo) };
            return Ok(Stmt { kind: StmtKind::Expr(e), span: self.span_from(lo) });
        }
        // `spawn CALLEE(ARGS);` — start one task (design 0012 §1.1, §5). Contextual:
        // `spawn` is a keyword only statement-leading and immediately followed by a
        // callee identifier; a `spawn` outside any `scope` is a CHECKER error (§5),
        // so the grammar stays context-free — the parser accepts it anywhere.
        if self.at_ident("spawn") && matches!(self.peek_at(1), RTok::Ident(_)) {
            self.bump(); // `spawn`
            let call = self.parse_expr()?;
            if !matches!(call.kind, ExprKind::Call { .. }) {
                return Err(self.unexpected("a call `CALLEE(ARGS)` after `spawn`"));
            }
            self.expect(&RTok::Semi, "`;`")?;
            let e = Expr { kind: ExprKind::Spawn(Box::new(call)), span: self.span_from(lo) };
            return Ok(Stmt { kind: StmtKind::Expr(e), span: self.span_from(lo) });
        }
        // A block-like expression in statement position terminates the statement
        // (spec 02 §5.2): no trailing `;`, and a following `(` starts a new
        // statement (never a call). Parse it directly, bypassing postfix.
        if self.at_block_like_start() {
            let e = self.parse_block_like_expr()?;
            return Ok(Stmt { kind: StmtKind::Expr(e), span: self.span_from(lo) });
        }
        let expr = self.parse_expr()?;
        if self.eat(&RTok::Eq) {
            let value = self.parse_expr()?;
            self.expect(&RTok::Semi, "`;`")?;
            return Ok(Stmt { kind: StmtKind::Assign { target: expr, value }, span: self.span_from(lo) });
        }
        self.expect(&RTok::Semi, "`;`")?;
        Ok(Stmt { kind: StmtKind::Expr(expr), span: self.span_from(lo) })
    }

    fn parse_let(&mut self, lo: usize) -> PResult<Stmt> {
        self.expect(&RTok::Kw(RKw::Let), "`let`")?;
        let mutable = self.eat_kw(RKw::Mut);
        let name = self.expect_ident("a binding name")?;
        let ty = if self.eat(&RTok::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        let init = if self.eat(&RTok::Eq) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(&RTok::Semi, "`;`")?;
        Ok(Stmt { kind: StmtKind::Let { mutable, name, ty, init }, span: self.span_from(lo) })
    }

    /// Parse and DESUGAR `for PATTERN in OPERAND { BODY }` (design 0009 §4).
    /// The operand's borrow mode selects the protocol (§4.1): a `read`-prefixed
    /// operand is `Indexed` (borrow + copy), any other operand is `Iter`
    /// (consuming). The desugar lowers to `loop` + `match` over existing AST
    /// nodes (§4.2). `for`/`in` are contextual (§4.4); operand is `ExprNoStruct`.
    fn parse_for(&mut self, lo: usize) -> PResult<Stmt> {
        self.bump(); // `for`
        // A `read`-prefixed binding selects the region-free BORROWED-yield protocol
        // (`RefIndexed`, OBL-ITER-BORROW's region-free branch): `for read x in read
        // coll` binds `x` to a `read Item` reborrow of each element rather than a
        // copy. The borrow is visible in the binding, syntax-directed (NN#13), and
        // requires a `read`-borrowed operand. `write x` (mutating yield) stays
        // deferred (needs swap/replace).
        let by_ref = self.eat(&RTok::Kw(RKw::Read));
        let pat = self.parse_pattern()?;
        if !self.at_ident("in") {
            return Err(self.unexpected("`in` in a `for` header"));
        }
        self.bump(); // `in`
        let operand = self.parse_expr_no_struct()?;
        let mut body = self.parse_block()?;
        let sp = self.span_from(lo);
        if self.preserve_for {
            // Formatter path: keep the surface `for` node (NN#11); never desugar.
            let e = Expr {
                kind: ExprKind::For { pattern: pat, operand: Box::new(operand), body, by_ref },
                span: sp,
            };
            return Ok(Stmt { kind: StmtKind::Expr(e), span: sp });
        }
        let n = self.for_ctr;
        self.for_ctr += 1;
        let indexed = matches!(operand.kind, ExprKind::Prefix { op: PrefixOp::Read, .. });
        let block = if by_ref {
            if !indexed {
                return Err(Diag::error(
                    "P0011",
                    "a `read`-binding `for read x in ...` requires a `read`-borrowed operand (design 0009 §4; OBL-ITER-BORROW region-free branch)",
                    sp,
                ));
            }
            self.desugar_ref_indexed(pat, operand, body, n, sp)?
        } else if indexed {
            self.desugar_indexed(pat, operand, body, n, sp)
        } else {
            let it = format!("__it{n}");
            rewrite_breaks(&mut body, &it, n, sp);
            self.desugar_iter(pat, operand, body, n, sp)
        };
        Ok(Stmt { kind: StmtKind::Expr(block), span: sp })
    }

    /// Borrowed-yield desugar (OBL-ITER-BORROW, region-free branch): the operand's
    /// loop-local `read` borrow `__c` is held across the loop; `count(read self)`
    /// gives the bound and `get_ref(read self, i) -> read Item` yields a *reborrow*
    /// of element `i` (no copy — usable over non-`copy` elements). Region-free by
    /// 0001 §3.3's compact default: `get_ref` has a single borrow-in (`read self`)
    /// and a single borrow-out (`read Item`), so the return derives from `self`
    /// with no region variable and nothing storable (no borrow field, 0001 §3.4).
    /// The loop-local `read` loan on `coll` spans the loop, so a `write coll` in the
    /// body conflicts by XOR (chapter 04) — mutation-during-iteration rejected,
    /// reusing the existing loan machinery (no new rule).
    fn desugar_ref_indexed(&self, pat: Pattern, operand: Expr, body: Block, n: usize, sp: Span) -> PResult<Expr> {
        let name = match &pat.kind {
            PatKind::Binding(nm) => nm.clone(),
            _ => {
                return Err(Diag::error(
                    "P0012",
                    "a `read`-binding `for read x in ...` binds a single name to a per-element borrow (a destructuring pattern would move, which a borrow cannot)",
                    pat.span,
                ));
            }
        };
        let c = format!("__c{n}");
        let cnt = format!("__n{n}");
        let i = format!("__i{n}");
        let usize_ty = Ty { kind: TyKind::Scalar(crate::token::ScalarTy::Usize), span: sp };
        // Loop body: `if __i >= __n { break; } let x = __c.get_ref(__i); __i = __i + 1u; BODY`
        let guard_cond = Expr {
            kind: ExprKind::Binary { op: BinOp::Ge, lhs: Box::new(ident_expr(&i, sp)), rhs: Box::new(ident_expr(&cnt, sp)) },
            span: sp,
        };
        let guard = Stmt {
            kind: StmtKind::Expr(Expr {
                kind: ExprKind::If { cond: Box::new(guard_cond), then_blk: Block { stmts: vec![break_stmt(sp)], span: sp }, else_blk: None },
                span: sp,
            }),
            span: sp,
        };
        let get = method_call(ident_expr(&c, sp), "get_ref", vec![ident_expr(&i, sp)], sp);
        let bind_x = Stmt { kind: StmtKind::Let { mutable: false, name, ty: None, init: Some(get) }, span: sp };
        let inc = assign_stmt(
            ident_expr(&i, sp),
            Expr { kind: ExprKind::Binary { op: BinOp::Add, lhs: Box::new(ident_expr(&i, sp)), rhs: Box::new(Expr { kind: ExprKind::IntLit { value: 1, suffix: None }, span: sp }) }, span: sp },
            sp,
        );
        let mut loop_stmts = vec![guard, bind_x, inc];
        loop_stmts.extend(body.stmts);
        let loope = Expr { kind: ExprKind::Loop(Block { stmts: loop_stmts, span: sp }), span: sp };
        let count_call = method_call(ident_expr(&c, sp), "count", Vec::new(), sp);
        let stmts = vec![
            Stmt { kind: StmtKind::Let { mutable: false, name: c, ty: None, init: Some(operand) }, span: sp },
            Stmt { kind: StmtKind::Let { mutable: false, name: cnt, ty: None, init: Some(count_call) }, span: sp },
            Stmt { kind: StmtKind::Let { mutable: true, name: i, ty: Some(usize_ty), init: Some(Expr { kind: ExprKind::IntLit { value: 0, suffix: None }, span: sp }) }, span: sp },
            Stmt { kind: StmtKind::Expr(loope), span: sp },
        ];
        Ok(block_expr(stmts, sp))
    }

    /// Consuming desugar (design 0009 §4.2): the operand is MOVED into `__it`,
    /// `next(take self)` consumes it each turn, `More` restores the successor,
    /// `Done` leaves it uninitialized; break edges were sink-moved by the caller.
    fn desugar_iter(&self, pat: Pattern, operand: Expr, body: Block, n: usize, sp: Span) -> Expr {
        let it = format!("__it{n}");
        let rest = format!("__rest{n}");
        let mut more_stmts = vec![assign_stmt(ident_expr(&it, sp), ident_expr(&rest, sp), sp)];
        more_stmts.extend(body.stmts);
        let more_arm = MatchArm {
            pattern: variant_pat("IterStep", "More", vec![pat, binding_pat(&rest, sp)], sp),
            guard: None,
            body: block_expr(more_stmts, sp),
            span: sp,
        };
        let done_arm = MatchArm {
            pattern: variant_pat("IterStep", "Done", Vec::new(), sp),
            guard: None,
            body: block_expr(vec![break_stmt(sp)], sp),
            span: sp,
        };
        let scrut = method_call(ident_expr(&it, sp), "next", Vec::new(), sp);
        let matche = Expr { kind: ExprKind::Match { scrutinee: Box::new(scrut), arms: vec![more_arm, done_arm] }, span: sp };
        let loop_body = Block { stmts: vec![Stmt { kind: StmtKind::Expr(matche), span: sp }], span: sp };
        let loope = Expr { kind: ExprKind::Loop(loop_body), span: sp };
        let stmts = vec![
            Stmt { kind: StmtKind::Let { mutable: true, name: it, ty: None, init: Some(operand) }, span: sp },
            Stmt { kind: StmtKind::Expr(loope), span: sp },
        ];
        block_expr(stmts, sp)
    }

    /// Borrow-copy desugar (design 0009 §4.2): a loop-local `read` borrow `__c`
    /// held across the loop, a `usize` cursor `__i`, and `at(read self, i)`
    /// copying each item out. No exit-edge rewrite is needed (§4.2).
    fn desugar_indexed(&self, pat: Pattern, operand: Expr, body: Block, n: usize, sp: Span) -> Expr {
        let c = format!("__c{n}");
        let i = format!("__i{n}");
        let inc = assign_stmt(
            ident_expr(&i, sp),
            Expr { kind: ExprKind::Binary { op: BinOp::Add, lhs: Box::new(ident_expr(&i, sp)), rhs: Box::new(Expr { kind: ExprKind::IntLit { value: 1, suffix: None }, span: sp }) }, span: sp },
            sp,
        );
        let mut some_stmts = vec![inc];
        some_stmts.extend(body.stmts);
        let some_arm = MatchArm {
            pattern: variant_pat("Opt", "Some", vec![pat], sp),
            guard: None,
            body: block_expr(some_stmts, sp),
            span: sp,
        };
        let none_arm = MatchArm {
            pattern: variant_pat("Opt", "None", Vec::new(), sp),
            guard: None,
            body: block_expr(vec![break_stmt(sp)], sp),
            span: sp,
        };
        let scrut = method_call(ident_expr(&c, sp), "at", vec![ident_expr(&i, sp)], sp);
        let matche = Expr { kind: ExprKind::Match { scrutinee: Box::new(scrut), arms: vec![some_arm, none_arm] }, span: sp };
        let loop_body = Block { stmts: vec![Stmt { kind: StmtKind::Expr(matche), span: sp }], span: sp };
        let loope = Expr { kind: ExprKind::Loop(loop_body), span: sp };
        let usize_ty = Ty { kind: TyKind::Scalar(crate::token::ScalarTy::Usize), span: sp };
        let stmts = vec![
            Stmt { kind: StmtKind::Let { mutable: false, name: c, ty: None, init: Some(operand) }, span: sp },
            Stmt { kind: StmtKind::Let { mutable: true, name: i, ty: Some(usize_ty), init: Some(Expr { kind: ExprKind::IntLit { value: 0, suffix: None }, span: sp }) }, span: sp },
            Stmt { kind: StmtKind::Expr(loope), span: sp },
        ];
        block_expr(stmts, sp)
    }

    fn at_block_like_start(&self) -> bool {
        matches!(
            self.peek(),
            RTok::LBrace
                | RTok::Kw(RKw::If)
                | RTok::Kw(RKw::Match)
                | RTok::Kw(RKw::Loop)
                | RTok::Kw(RKw::While)
                | RTok::Kw(RKw::Unsafe)
                | RTok::Kw(RKw::Wrapping)
                | RTok::Kw(RKw::Saturating)
        )
    }

    fn parse_block_like_expr(&mut self) -> PResult<Expr> {
        let lo = self.cur_start();
        let kind = match self.peek() {
            RTok::LBrace => ExprKind::Block(self.parse_block()?),
            RTok::Kw(RKw::If) => self.parse_if()?,
            RTok::Kw(RKw::Match) => self.parse_match()?,
            RTok::Kw(RKw::Loop) => {
                self.bump();
                ExprKind::Loop(self.parse_block()?)
            }
            RTok::Kw(RKw::While) => {
                self.bump();
                let cond = self.parse_expr_no_struct()?;
                let body = self.parse_block()?;
                ExprKind::While { cond: Box::new(cond), body }
            }
            RTok::Kw(RKw::Unsafe) => self.parse_unsafe()?,
            RTok::Kw(RKw::Wrapping) => {
                self.bump();
                ExprKind::Wrapping(self.parse_block()?)
            }
            RTok::Kw(RKw::Saturating) => {
                self.bump();
                ExprKind::Saturating(self.parse_block()?)
            }
            _ => return Err(self.unexpected("a block-like expression")),
        };
        Ok(Expr { kind, span: self.span_from(lo) })
    }

    // ----- expressions (precedence per spec 02 §6) -------------------------
    fn parse_expr(&mut self) -> PResult<Expr> {
        self.parse_or()
    }
    fn parse_expr_no_struct(&mut self) -> PResult<Expr> {
        let saved = self.no_struct;
        self.no_struct = true;
        let r = self.parse_or();
        self.no_struct = saved;
        r
    }
    fn parse_delimited_expr(&mut self) -> PResult<Expr> {
        let saved = self.no_struct;
        self.no_struct = false;
        let r = self.parse_or();
        self.no_struct = saved;
        r
    }

    fn binary_left(
        &mut self,
        next: fn(&mut Self) -> PResult<Expr>,
        ops: &[(RTok, BinOp)],
    ) -> PResult<Expr> {
        let lo = self.cur_start();
        let mut lhs = next(self)?;
        'outer: loop {
            for (tok, op) in ops {
                if self.at(tok) {
                    self.bump();
                    let rhs = next(self)?;
                    lhs = Expr {
                        kind: ExprKind::Binary { op: *op, lhs: Box::new(lhs), rhs: Box::new(rhs) },
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
        self.binary_left(Self::parse_and, &[(RTok::PipePipe, BinOp::Or)])
    }
    fn parse_and(&mut self) -> PResult<Expr> {
        self.binary_left(Self::parse_cmp, &[(RTok::AmpAmp, BinOp::And)])
    }
    /// Comparison is non-associative (spec 02 §6.1): at most one comparison op.
    fn parse_cmp(&mut self) -> PResult<Expr> {
        let lo = self.cur_start();
        let lhs = self.parse_bitor()?;
        let op = match self.peek() {
            RTok::EqEq => Some(BinOp::Eq),
            RTok::Ne => Some(BinOp::Ne),
            RTok::Le => Some(BinOp::Le),
            RTok::Ge => Some(BinOp::Ge),
            RTok::Lt => Some(BinOp::Lt),
            RTok::Gt => Some(BinOp::Gt),
            _ => None,
        };
        match op {
            Some(op) => {
                self.bump();
                let rhs = self.parse_bitor()?;
                // A second comparison is a parse error (non-associative).
                if matches!(
                    self.peek(),
                    RTok::EqEq | RTok::Ne | RTok::Le | RTok::Ge | RTok::Lt | RTok::Gt
                ) {
                    return Err(Diag::error(
                        "P0006",
                        "comparison operators are non-associative; parenthesize",
                        self.cur_span(),
                    ));
                }
                Ok(Expr {
                    kind: ExprKind::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) },
                    span: self.span_from(lo),
                })
            }
            None => Ok(lhs),
        }
    }
    fn parse_bitor(&mut self) -> PResult<Expr> {
        self.binary_left(Self::parse_bitxor, &[(RTok::Pipe, BinOp::BitOr)])
    }
    fn parse_bitxor(&mut self) -> PResult<Expr> {
        self.binary_left(Self::parse_bitand, &[(RTok::Caret, BinOp::BitXor)])
    }
    fn parse_bitand(&mut self) -> PResult<Expr> {
        self.binary_left(Self::parse_shift, &[(RTok::Amp, BinOp::BitAnd)])
    }
    fn parse_shift(&mut self) -> PResult<Expr> {
        self.binary_left(Self::parse_add, &[(RTok::Shl, BinOp::Shl), (RTok::Shr, BinOp::Shr)])
    }
    fn parse_add(&mut self) -> PResult<Expr> {
        self.binary_left(Self::parse_mul, &[(RTok::Plus, BinOp::Add), (RTok::Minus, BinOp::Sub)])
    }
    fn parse_mul(&mut self) -> PResult<Expr> {
        self.binary_left(
            Self::parse_prefix,
            &[(RTok::Star, BinOp::Mul), (RTok::Slash, BinOp::Div), (RTok::Percent, BinOp::Rem)],
        )
    }

    fn parse_prefix(&mut self) -> PResult<Expr> {
        let lo = self.cur_start();
        let kind = match self.peek() {
            RTok::Minus => {
                self.bump();
                // Negative-literal fold: `-` directly before an integer-literal
                // token folds to one constant (spec 02 §6.6). `(` after `-` and a
                // non-literal are ordinary negation.
                if let RTok::Int { value, suffix } = self.peek().clone() {
                    self.bump();
                    ExprKind::NegIntLit { value, suffix }
                } else {
                    ExprKind::Unary { op: UnOp::Neg, expr: Box::new(self.parse_prefix()?) }
                }
            }
            RTok::Bang => {
                self.bump();
                ExprKind::Unary { op: UnOp::Not, expr: Box::new(self.parse_prefix()?) }
            }
            RTok::Tilde => {
                self.bump();
                ExprKind::Unary { op: UnOp::BitNot, expr: Box::new(self.parse_prefix()?) }
            }
            RTok::Kw(RKw::Read) => {
                self.bump();
                self.skip_prefix_region();
                ExprKind::Prefix { op: PrefixOp::Read, expr: Box::new(self.parse_prefix()?) }
            }
            RTok::Kw(RKw::Write) => {
                self.bump();
                self.skip_prefix_region();
                ExprKind::Prefix { op: PrefixOp::Write, expr: Box::new(self.parse_prefix()?) }
            }
            RTok::Kw(RKw::Clone) => {
                self.bump();
                ExprKind::Prefix { op: PrefixOp::Clone, expr: Box::new(self.parse_prefix()?) }
            }
            RTok::Kw(RKw::Conv) => {
                self.bump();
                // `conv ScalarKw Prefix` — target is a single scalar keyword and
                // the operand needs no parentheses (spec 02 §6.4).
                let tlo = self.cur_start();
                let sc = match self.peek().clone() {
                    RTok::Scalar(sc) => {
                        self.bump();
                        sc
                    }
                    _ => {
                        return Err(Diag::error(
                            "P0007",
                            "`conv` target must be a scalar type keyword",
                            self.cur_span(),
                        ))
                    }
                };
                let ty = Ty { kind: TyKind::Scalar(sc), span: self.span_from(tlo) };
                let operand = self.parse_prefix()?;
                ExprKind::Conv { ty, expr: Box::new(operand) }
            }
            RTok::Kw(RKw::Bitcast) => {
                self.bump();
                // `bitcast ScalarKw Prefix` — same-width float<->int bit reinterpret
                // (design 0016 §10), mirroring `conv`'s paren-free surface.
                let tlo = self.cur_start();
                let sc = match self.peek().clone() {
                    RTok::Scalar(sc) => {
                        self.bump();
                        sc
                    }
                    _ => {
                        return Err(Diag::error(
                            "P0007",
                            "`bitcast` target must be a scalar type keyword",
                            self.cur_span(),
                        ))
                    }
                };
                let ty = Ty { kind: TyKind::Scalar(sc), span: self.span_from(tlo) };
                let operand = self.parse_prefix()?;
                ExprKind::Bitcast { ty, expr: Box::new(operand) }
            }
            _ => return self.parse_prop(),
        };
        Ok(Expr { kind, span: self.span_from(lo) })
    }

    /// A region tag on a borrow *operator* in expression position carries no
    /// value in the shared AST; accept and drop it (`read[r] x`).
    fn skip_prefix_region(&mut self) {
        if self.at(&RTok::LBracket)
            && matches!(self.peek_at(1), RTok::Ident(_))
            && matches!(self.peek_at(2), RTok::RBracket)
        {
            self.bump();
            self.bump();
            self.bump();
        }
    }

    fn parse_prop(&mut self) -> PResult<Expr> {
        let lo = self.cur_start();
        let mut e = self.parse_postfix()?;
        while self.at(&RTok::Question) {
            self.bump();
            e = Expr { kind: ExprKind::Try(Box::new(e)), span: self.span_from(lo) };
        }
        Ok(e)
    }

    fn parse_postfix(&mut self) -> PResult<Expr> {
        let lo = self.cur_start();
        let mut expr = self.parse_primary()?;
        loop {
            if self.at(&RTok::DotStar) {
                self.bump();
                expr = Expr {
                    kind: ExprKind::Prefix { op: PrefixOp::Deref, expr: Box::new(expr) },
                    span: self.span_from(lo),
                };
            } else if self.at(&RTok::LParen) {
                self.bump();
                let args = self.parse_arg_list()?;
                self.expect(&RTok::RParen, "`)`")?;
                expr = Expr { kind: ExprKind::Call { callee: Box::new(expr), args }, span: self.span_from(lo) };
            } else if self.at(&RTok::LBracket) {
                self.bump();
                let index = self.parse_delimited_expr()?;
                self.expect(&RTok::RBracket, "`]`")?;
                expr = Expr {
                    kind: ExprKind::Index { base: Box::new(expr), index: Box::new(index) },
                    span: self.span_from(lo),
                };
            } else if self.at(&RTok::Dot) {
                self.bump();
                let field = self.expect_ident("a field name")?;
                expr = Expr { kind: ExprKind::Field { base: Box::new(expr), field, iface: None }, span: self.span_from(lo) };
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
        while !self.at(&RTok::RParen) {
            if self.at(&RTok::Kw(RKw::Out)) {
                let lo = self.cur_start();
                self.bump();
                let place = self.parse_or()?;
                args.push(Expr { kind: ExprKind::OutArg(Box::new(place)), span: self.span_from(lo) });
            } else {
                args.push(self.parse_or()?);
            }
            if !self.eat(&RTok::Comma) {
                break;
            }
        }
        self.no_struct = saved;
        Ok(args)
    }

    fn parse_primary(&mut self) -> PResult<Expr> {
        let lo = self.cur_start();
        let kind = match self.peek().clone() {
            RTok::Int { value, suffix } => {
                self.bump();
                ExprKind::IntLit { value, suffix }
            }
            RTok::Float { bits, ty } => {
                self.bump();
                ExprKind::FloatLit { bits, ty }
            }
            RTok::Str(s) => {
                self.bump();
                ExprKind::StrLit(s)
            }
            RTok::Bytes(s) => {
                self.bump();
                ExprKind::BytesLit(s)
            }
            RTok::Kw(RKw::True) => {
                self.bump();
                ExprKind::BoolLit(true)
            }
            RTok::Kw(RKw::False) => {
                self.bump();
                ExprKind::BoolLit(false)
            }
            RTok::Kw(RKw::Result) => {
                self.bump();
                ExprKind::Result
            }
            RTok::Kw(RKw::SelfKw) => {
                self.bump();
                ExprKind::Ident("self".to_string())
            }
            RTok::Kw(RKw::Break) => {
                self.bump();
                ExprKind::Break
            }
            RTok::Kw(RKw::Continue) => {
                self.bump();
                ExprKind::Continue
            }
            RTok::Kw(RKw::Return) => {
                self.bump();
                if self.starts_expr() {
                    ExprKind::Return(Some(Box::new(self.parse_expr()?)))
                } else {
                    ExprKind::Return(None)
                }
            }
            RTok::Kw(RKw::Assert) => {
                self.bump();
                self.expect(&RTok::LParen, "`(`")?;
                let e = self.parse_delimited_expr()?;
                self.expect(&RTok::RParen, "`)`")?;
                ExprKind::Assert(Box::new(e))
            }
            RTok::Kw(RKw::Panic) => {
                self.bump();
                self.expect(&RTok::LParen, "`(`")?;
                let e = self.parse_delimited_expr()?;
                self.expect(&RTok::RParen, "`)`")?;
                ExprKind::Panic(Box::new(e))
            }
            RTok::LParen => {
                self.bump();
                let inner = self.parse_delimited_expr()?;
                self.expect(&RTok::RParen, "`)`")?;
                ExprKind::Paren(Box::new(inner))
            }
            RTok::LBracket => self.parse_array_literal()?,
            RTok::LBrace => ExprKind::Block(self.parse_block()?),
            RTok::Kw(RKw::If) => self.parse_if()?,
            RTok::Kw(RKw::Match) => self.parse_match()?,
            RTok::Kw(RKw::Loop) => {
                self.bump();
                ExprKind::Loop(self.parse_block()?)
            }
            RTok::Kw(RKw::While) => {
                self.bump();
                let cond = self.parse_expr_no_struct()?;
                let body = self.parse_block()?;
                ExprKind::While { cond: Box::new(cond), body }
            }
            RTok::Kw(RKw::Unsafe) => self.parse_unsafe()?,
            RTok::Kw(RKw::Wrapping) => {
                self.bump();
                ExprKind::Wrapping(self.parse_block()?)
            }
            RTok::Kw(RKw::Saturating) => {
                self.bump();
                ExprKind::Saturating(self.parse_block()?)
            }
            RTok::Kw(RKw::CastPtr) => self.parse_type_arg_intrinsic(IntrinsicKind::CastPtr)?,
            RTok::Kw(RKw::AddrToPtr) => self.parse_type_arg_intrinsic(IntrinsicKind::AddrToPtr)?,
            RTok::Kw(RKw::PtrNull) => self.parse_type_arg_intrinsic(IntrinsicKind::PtrNull)?,
            RTok::Kw(RKw::Offsetof) => {
                self.bump();
                self.expect(&RTok::LParen, "`(`")?;
                let ty = self.parse_type()?;
                self.expect(&RTok::Comma, "`,`")?;
                let field = self.expect_ident("a field name")?;
                self.expect(&RTok::RParen, "`)`")?;
                ExprKind::Offsetof { ty, field }
            }
            RTok::Kw(RKw::FieldPtr) => {
                self.bump();
                self.expect(&RTok::LParen, "`(`")?;
                let ptr = self.parse_expr()?;
                self.expect(&RTok::Comma, "`,`")?;
                let field = self.expect_ident("a field name")?;
                self.expect(&RTok::RParen, "`)`")?;
                ExprKind::FieldPtr { ptr: Box::new(ptr), field }
            }
            RTok::Kw(RKw::Sizeof) => {
                self.bump();
                self.expect(&RTok::LParen, "`(`")?;
                let ty = self.parse_type()?;
                self.expect(&RTok::RParen, "`)`")?;
                ExprKind::Sizeof(ty)
            }
            RTok::Kw(RKw::Alignof) => {
                self.bump();
                self.expect(&RTok::LParen, "`(`")?;
                let ty = self.parse_type()?;
                self.expect(&RTok::RParen, "`)`")?;
                ExprKind::Alignof(ty)
            }
            RTok::Kw(RKw::BoxResult) => {
                self.bump();
                self.expect(&RTok::ColonColon, "`::`")?;
                let variant = self.expect_ident("a variant name")?;
                let args = self.parse_opt_ctor_args()?;
                ExprKind::EnumCtor { enum_name: "BoxResult".to_string(), variant, args }
            }
            RTok::Ident(name) => {
                if matches!(self.peek_at(1), RTok::ColonColon) && matches!(self.peek_at(2), RTok::LBracket) {
                    // `name::[T, ...]` — a generic function named as a value
                    // (design 0007 §6.2.1).
                    self.bump(); // name
                    self.bump(); // ::
                    self.bump(); // [
                    let mut ty_args = Vec::new();
                    while !self.at(&RTok::RBracket) {
                        ty_args.push(self.parse_type()?);
                        if !self.eat(&RTok::Comma) {
                            break;
                        }
                    }
                    self.expect(&RTok::RBracket, "`]`")?;
                    ExprKind::GenericVal { name, ty_args }
                } else if matches!(self.peek_at(1), RTok::ColonColon) {
                    self.bump();
                    self.bump();
                    let variant = self.expect_ident("a variant name")?;
                    let args = self.parse_opt_ctor_args()?;
                    ExprKind::EnumCtor { enum_name: name, variant, args }
                } else if !self.no_struct && matches!(self.peek_at(1), RTok::LBrace) {
                    self.bump();
                    self.parse_struct_literal_body(name)?
                } else {
                    self.bump();
                    ExprKind::Ident(name)
                }
            }
            _ => return Err(self.unexpected("an expression")),
        };
        Ok(Expr { kind, span: self.span_from(lo) })
    }

    fn parse_unsafe(&mut self) -> PResult<ExprKind> {
        self.expect(&RTok::Kw(RKw::Unsafe), "`unsafe`")?;
        let justification = match self.peek().clone() {
            RTok::Str(s) => {
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
        Ok(ExprKind::Unsafe { justification, body })
    }

    fn parse_opt_ctor_args(&mut self) -> PResult<Vec<Expr>> {
        if self.at(&RTok::LParen) {
            self.bump();
            let args = self.parse_arg_list()?;
            self.expect(&RTok::RParen, "`)`")?;
            Ok(args)
        } else {
            Ok(Vec::new())
        }
    }

    fn parse_struct_literal_body(&mut self, name: String) -> PResult<ExprKind> {
        self.expect(&RTok::LBrace, "`{`")?;
        let saved = self.no_struct;
        self.no_struct = false;
        let mut fields = Vec::new();
        while !self.at(&RTok::RBrace) {
            let flo = self.cur_start();
            let fname = self.expect_ident("a field name")?;
            self.expect(&RTok::Colon, "`:`")?;
            let value = self.parse_or()?;
            fields.push(FieldInit { name: fname, value, span: self.span_from(flo) });
            if !self.eat(&RTok::Comma) {
                break;
            }
        }
        self.no_struct = saved;
        self.expect(&RTok::RBrace, "`}`")?;
        Ok(ExprKind::StructLit { name, fields })
    }

    fn parse_type_arg_intrinsic(&mut self, kind: IntrinsicKind) -> PResult<ExprKind> {
        self.bump();
        self.expect(&RTok::LBracket, "`[`")?;
        let ty = self.parse_type()?;
        self.expect(&RTok::RBracket, "`]`")?;
        self.expect(&RTok::LParen, "`(`")?;
        let args = self.parse_arg_list()?;
        self.expect(&RTok::RParen, "`)`")?;
        match kind {
            IntrinsicKind::PtrNull => {
                if !args.is_empty() {
                    return Err(Diag::error("P0004", "`ptr_null[T]()` takes no arguments", self.cur_span()));
                }
                Ok(ExprKind::PtrNull { ty })
            }
            IntrinsicKind::CastPtr | IntrinsicKind::AddrToPtr => {
                let mut it = args.into_iter();
                let arg = it
                    .next()
                    .ok_or_else(|| Diag::error("P0005", "this intrinsic requires exactly one argument", self.cur_span()))?;
                Ok(match kind {
                    IntrinsicKind::CastPtr => ExprKind::CastPtr { ty, arg: Box::new(arg) },
                    _ => ExprKind::AddrToPtr { ty, arg: Box::new(arg) },
                })
            }
        }
    }

    fn parse_array_literal(&mut self) -> PResult<ExprKind> {
        self.expect(&RTok::LBracket, "`[`")?;
        let saved = self.no_struct;
        self.no_struct = false;
        if self.at(&RTok::RBracket) {
            self.no_struct = saved;
            self.bump();
            return Ok(ExprKind::ArrayLit(Vec::new()));
        }
        let first = self.parse_or()?;
        let out = if self.eat(&RTok::Semi) {
            let size = self.parse_or()?;
            self.expect(&RTok::RBracket, "`]`")?;
            ExprKind::ArrayRepeat { value: Box::new(first), size: Box::new(size) }
        } else {
            let mut elems = vec![first];
            while self.eat(&RTok::Comma) {
                if self.at(&RTok::RBracket) {
                    break;
                }
                elems.push(self.parse_or()?);
            }
            self.expect(&RTok::RBracket, "`]`")?;
            ExprKind::ArrayLit(elems)
        };
        self.no_struct = saved;
        Ok(out)
    }

    fn parse_if(&mut self) -> PResult<ExprKind> {
        self.expect(&RTok::Kw(RKw::If), "`if`")?;
        let cond = self.parse_expr_no_struct()?;
        let then_blk = self.parse_block()?;
        let else_blk = if self.eat_kw(RKw::Else) {
            let lo = self.cur_start();
            if self.at_kw(RKw::If) {
                let k = self.parse_if()?;
                Some(Box::new(Expr { kind: k, span: self.span_from(lo) }))
            } else {
                let b = self.parse_block()?;
                Some(Box::new(Expr { kind: ExprKind::Block(b), span: self.span_from(lo) }))
            }
        } else {
            None
        };
        Ok(ExprKind::If { cond: Box::new(cond), then_blk, else_blk })
    }

    /// `match` without the per-arm `case` (spec 02 §8): arms are `Pattern => Expr`,
    /// comma-separated with an optional comma after a block-bodied arm.
    fn parse_match(&mut self) -> PResult<ExprKind> {
        self.expect(&RTok::Kw(RKw::Match), "`match`")?;
        let scrutinee = self.parse_expr_no_struct()?;
        self.expect(&RTok::LBrace, "`{`")?;
        let mut arms = Vec::new();
        while !self.at(&RTok::RBrace) && !self.at(&RTok::Eof) {
            let alo = self.cur_start();
            let pattern = self.parse_pattern()?;
            // An optional `if EXPR` guard (design 0001 §8.2, extended). Parsed in
            // no-struct context (like an `if`/`while` condition): a struct literal
            // in a guard needs parentheses, keeping the `{` unambiguous.
            let guard = if self.eat_kw(RKw::If) {
                Some(self.parse_expr_no_struct()?)
            } else {
                None
            };
            self.expect(&RTok::FatArrow, "`=>`")?;
            let body = if self.at_block_like_start() {
                self.parse_block_like_expr()?
            } else {
                self.parse_expr()?
            };
            arms.push(MatchArm { pattern, guard, body, span: self.span_from(alo) });
            // Optional arm separator (permissive: also allowed after a non-block
            // arm; the arm-boundary rule makes the comma optional after a block).
            self.eat(&RTok::Comma);
        }
        self.expect(&RTok::RBrace, "`}`")?;
        Ok(ExprKind::Match { scrutinee: Box::new(scrutinee), arms })
    }

    fn parse_pattern(&mut self) -> PResult<Pattern> {
        let lo = self.cur_start();
        let kind = match self.peek().clone() {
            RTok::Kw(RKw::BoxResult) => {
                self.bump();
                self.expect(&RTok::ColonColon, "`::`")?;
                let variant = self.expect_ident("a variant name")?;
                let sub = self.parse_opt_pattern_args()?;
                PatKind::Variant { enum_name: "BoxResult".to_string(), variant, sub }
            }
            RTok::Int { .. } | RTok::Minus => {
                // An integer-literal pattern, or an integer-range pattern when a
                // `..`/`..=` follows the first endpoint. A leading `-` folds onto
                // the literal, mirroring expressions (spec 02 §6.6).
                let (lo_value, lo_negative, lo_suffix) = self.parse_int_endpoint()?;
                match self.peek() {
                    RTok::DotDot | RTok::DotDotEq => {
                        let inclusive = matches!(self.peek(), RTok::DotDotEq);
                        self.bump();
                        let (hi_value, hi_negative, hi_suffix) = self.parse_int_endpoint()?;
                        PatKind::IntRange {
                            lo_value,
                            lo_negative,
                            lo_suffix,
                            hi_value,
                            hi_negative,
                            hi_suffix,
                            inclusive,
                        }
                    }
                    _ => PatKind::IntLit { value: lo_value, negative: lo_negative, suffix: lo_suffix },
                }
            }
            RTok::Ident(name) if name == "_" => {
                self.bump();
                PatKind::Wildcard
            }
            RTok::Ident(name) => {
                if matches!(self.peek_at(1), RTok::ColonColon) {
                    self.bump();
                    self.bump();
                    let variant = self.expect_ident("a variant name")?;
                    let sub = self.parse_opt_pattern_args()?;
                    PatKind::Variant { enum_name: name, variant, sub }
                } else {
                    self.bump();
                    PatKind::Binding(name)
                }
            }
            _ => return Err(self.unexpected("a pattern")),
        };
        Ok(Pattern { kind, span: self.span_from(lo) })
    }

    /// Parse one integer endpoint of a literal/range pattern: an optional `-`
    /// sign folded onto an integer literal (spec 02 §6.6).
    fn parse_int_endpoint(&mut self) -> PResult<(u64, bool, Option<crate::token::ScalarTy>)> {
        let negative = self.eat(&RTok::Minus);
        if let RTok::Int { value, suffix } = self.peek().clone() {
            self.bump();
            Ok((value, negative, suffix))
        } else {
            Err(self.unexpected("an integer literal"))
        }
    }

    fn parse_opt_pattern_args(&mut self) -> PResult<Vec<Pattern>> {
        let mut sub = Vec::new();
        if self.eat(&RTok::LParen) {
            while !self.at(&RTok::RParen) {
                sub.push(self.parse_pattern()?);
                if !self.eat(&RTok::Comma) {
                    break;
                }
            }
            self.expect(&RTok::RParen, "`)`")?;
        }
        Ok(sub)
    }

    fn starts_expr(&self) -> bool {
        !matches!(
            self.peek(),
            RTok::Semi | RTok::Comma | RTok::RParen | RTok::RBrace | RTok::RBracket | RTok::Eof
        )
    }
}

enum IntrinsicKind {
    CastPtr,
    AddrToPtr,
    PtrNull,
}

/// Does this token begin a `Type` (spec 02 §3)? Used to disambiguate a named
/// array size from a slice element after `[` (§3.3).
fn starts_type(k: &RTok) -> bool {
    matches!(
        k,
        RTok::Scalar(_)
            | RTok::Ident(_)
            | RTok::Kw(RKw::RawPtr)
            | RTok::Kw(RKw::Box)
            | RTok::Kw(RKw::BoxResult)
            | RTok::Kw(RKw::Read)
            | RTok::Kw(RKw::Write)
            | RTok::Kw(RKw::Fn)
            | RTok::LBracket
    )
}


// ---------------------------------------------------------------------------
// `for`-loop desugar helpers (design 0009 §4.2): build the loop/match AST.
// ---------------------------------------------------------------------------

fn ident_expr(name: &str, sp: Span) -> Expr {
    Expr { kind: ExprKind::Ident(name.to_string()), span: sp }
}
fn block_expr(stmts: Vec<Stmt>, sp: Span) -> Expr {
    Expr { kind: ExprKind::Block(Block { stmts, span: sp }), span: sp }
}
fn assign_stmt(target: Expr, value: Expr, sp: Span) -> Stmt {
    Stmt { kind: StmtKind::Assign { target, value }, span: sp }
}
fn break_stmt(sp: Span) -> Stmt {
    Stmt { kind: StmtKind::Expr(Expr { kind: ExprKind::Break, span: sp }), span: sp }
}
fn binding_pat(name: &str, sp: Span) -> Pattern {
    Pattern { kind: PatKind::Binding(name.to_string()), span: sp }
}
fn variant_pat(enum_name: &str, variant: &str, sub: Vec<Pattern>, sp: Span) -> Pattern {
    Pattern { kind: PatKind::Variant { enum_name: enum_name.to_string(), variant: variant.to_string(), sub }, span: sp }
}
fn method_call(recv: Expr, method: &str, args: Vec<Expr>, sp: Span) -> Expr {
    let callee = Expr { kind: ExprKind::Field { base: Box::new(recv), field: method.to_string(), iface: None }, span: sp };
    Expr { kind: ExprKind::Call { callee: Box::new(callee), args }, span: sp }
}

/// Rewrite every `break` that targets THIS `for` loop into the synthesized
/// sink-move `{ let __sinkN = __itN; break; }` (design 0009 §4.2), so the moved
/// iterator is consumed on the break edge and post-loop init state is
/// path-independent (E0309). Breaks inside a NESTED `loop`/`while` target their
/// own loop and are not rewritten; the walk stops at those bodies.
fn rewrite_breaks(block: &mut Block, it: &str, n: usize, sp: Span) {
    for st in &mut block.stmts {
        match &mut st.kind {
            StmtKind::Expr(e) => rewrite_breaks_expr(e, it, n, sp),
            StmtKind::Let { init: Some(e), .. } => rewrite_breaks_expr(e, it, n, sp),
            StmtKind::Assign { value, .. } => rewrite_breaks_expr(value, it, n, sp),
            _ => {}
        }
    }
}

fn rewrite_breaks_expr(e: &mut Expr, it: &str, n: usize, sp: Span) {
    match &mut e.kind {
        ExprKind::Break => {
            let sink = format!("__sink{n}");
            let let_sink = Stmt {
                kind: StmtKind::Let { mutable: false, name: sink, ty: None, init: Some(ident_expr(it, sp)) },
                span: sp,
            };
            let brk = Stmt { kind: StmtKind::Expr(Expr { kind: ExprKind::Break, span: sp }), span: sp };
            e.kind = ExprKind::Block(Block { stmts: vec![let_sink, brk], span: sp });
        }
        // Do NOT descend into a nested loop: its breaks target itself.
        ExprKind::Loop(_) | ExprKind::While { .. } => {}
        ExprKind::Block(b) => rewrite_breaks(b, it, n, sp),
        ExprKind::If { cond, then_blk, else_blk } => {
            rewrite_breaks_expr(cond, it, n, sp);
            rewrite_breaks(then_blk, it, n, sp);
            if let Some(x) = else_blk {
                rewrite_breaks_expr(x, it, n, sp);
            }
        }
        ExprKind::Match { scrutinee, arms } => {
            rewrite_breaks_expr(scrutinee, it, n, sp);
            for arm in arms {
                rewrite_breaks_expr(&mut arm.body, it, n, sp);
            }
        }
        ExprKind::Unsafe { body, .. } | ExprKind::Wrapping(body) | ExprKind::Saturating(body) => {
            rewrite_breaks(body, it, n, sp);
        }
        ExprKind::Paren(inner) => rewrite_breaks_expr(inner, it, n, sp),
        _ => {}
    }
}

fn describe(k: &RTok) -> String {
    match k {
        RTok::Eof => "end of input".to_string(),
        RTok::Ident(s) => format!("identifier `{s}`"),
        RTok::Int { .. } => "an integer literal".to_string(),
        RTok::Float { .. } => "a float literal".to_string(),
        RTok::Str(_) => "a string literal".to_string(),
        other => format!("`{other:?}`"),
    }
}

/// Convenience wrapper: parse a real-syntax token stream into the shared AST.
pub fn parse(tokens: Vec<RToken>) -> PResult<Program> {
    RParser::new(tokens).parse_program()
}

/// Parse a real-syntax token stream as a *module*: the shared AST plus the
/// module-layer side channels (design 0008) — the `use` imports and the
/// per-item visibility flags parallel to `Program.items`.
pub fn parse_module(tokens: Vec<RToken>) -> PResult<(Program, Vec<UseDecl>, Vec<bool>, bool)> {
    let mut p = RParser::new(tokens);
    let prog = p.parse_program()?;
    let boundary = p.boundary;
    Ok((prog, p.mod_uses, p.mod_vis, boundary))
}

/// Parse a real-syntax token stream for the **formatter** (NN#11): like
/// [`parse_module`] but keeps `for` loops as surface `ExprKind::For` nodes
/// (design 0009 §4.2) rather than desugaring them, so the canonical `for`
/// spelling round-trips.
pub fn parse_format(tokens: Vec<RToken>) -> PResult<(Program, Vec<UseDecl>, Vec<bool>, bool)> {
    let mut p = RParser::new(tokens);
    p.preserve_for = true;
    let prog = p.parse_program()?;
    let boundary = p.boundary;
    Ok((prog, p.mod_uses, p.mod_vis, boundary))
}

/// Parse and also report the file's `boundary`-preamble status (design 0011).
pub fn parse_with_boundary(tokens: Vec<RToken>) -> PResult<(Program, bool)> {
    let mut p = RParser::new(tokens);
    let prog = p.parse_program()?;
    Ok((prog, p.boundary))
}
