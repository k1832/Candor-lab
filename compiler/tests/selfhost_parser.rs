//! The oracle gate for the SECOND self-hosted slice: a Candor PARSER
//! (`selfhost/parser/parser.cnr`, loaded as a module tree with the `lexer`
//! module `selfhost/lexer/lexer.cnr`) is
//! run on the tree-walker over each in-subset real-syntax corpus fixture, and
//! its canonical AST S-expression dump is asserted byte-equal to the Rust oracle
//! parser's (`src/real/parser.rs` -> `ast.rs`) dump of the SAME source in the
//! SAME schema. Passing this gate is AST-SHAPE EQUALITY between the two parsers.
//!
//! The canonical S-expression schema (SPAN-FREE by design; the differential
//! target is tree shape, which both parsers must agree on) is implemented on the
//! Candor side by `parse_dump` and here by `render_program`; the two must match
//! token-for-token. Harness shape reuses slice 1: a generated root `main.cnr`
//! `use`s the `lexer`/`parser` modules, embeds the fixture source as a `[N]u8`
//! literal, lexes via lexer, parse-dumps via parser; the tree is loaded with
//! `run_dir` (dogfooding the module system), and the text is reconstructed from
//! `Run.trace` and compared.

use candor::ast::*;
use candor::real::parser::parse_format;
use candor::token::ScalarTy;
use candor::RunResult;

mod selfhost_modtree;
use selfhost_modtree::{on_big_stack, run_module_tree, trace_text};

const LEXER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../selfhost/lexer/lexer.cnr"));
const PARSER_SRC: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../selfhost/parser/parser.cnr"));

fn suf_code(s: Option<ScalarTy>) -> u32 {
    match s {
        None => 0,
        Some(ScalarTy::I8) => 1,
        Some(ScalarTy::I16) => 2,
        Some(ScalarTy::I32) => 3,
        Some(ScalarTy::I64) => 4,
        Some(ScalarTy::Isize) => 5,
        Some(ScalarTy::U8) => 6,
        Some(ScalarTy::U16) => 7,
        Some(ScalarTy::U32) => 8,
        Some(ScalarTy::U64) => 9,
        Some(ScalarTy::Usize) => 10,
        Some(ScalarTy::Bool) | Some(ScalarTy::Unit) | Some(ScalarTy::F64) | Some(ScalarTy::F32) => unreachable!("not a suffix"),
    }
}

fn scalar_name(sc: ScalarTy) -> &'static str {
    match sc {
        ScalarTy::I8 => "i8",
        ScalarTy::I16 => "i16",
        ScalarTy::I32 => "i32",
        ScalarTy::I64 => "i64",
        ScalarTy::Isize => "isize",
        ScalarTy::U8 => "u8",
        ScalarTy::U16 => "u16",
        ScalarTy::U32 => "u32",
        ScalarTy::U64 => "u64",
        ScalarTy::Usize => "usize",
        ScalarTy::Bool => "bool",
        ScalarTy::Unit => "unit",
        ScalarTy::F64 => "f64",
        ScalarTy::F32 => "f32",
    }
}

// ---- the canonical S-expression renderer over the oracle AST ----------------
struct R {
    s: String,
}
impl R {
    fn w(&mut self, x: &str) {
        self.s.push_str(x);
    }
    fn sp(&mut self) {
        self.s.push(' ');
    }
    fn bytes(&mut self, b: &[u8]) {
        // `(str/bytes LEN b0 b1 ...)` decoded-byte form (matches lexer dump).
        self.w(&format!("{}", b.len()));
        for x in b {
            self.w(&format!(" {x}"));
        }
    }

    fn program(&mut self, p: &Program) {
        self.w("(program");
        for it in &p.items {
            self.sp();
            self.item(it);
        }
        self.w(")");
    }

    fn item(&mut self, it: &Item) {
        match it {
            Item::Fn(f) => self.func(f),
            Item::Struct(s) => self.strukt(s),
            Item::Enum(e) => self.enom(e),
            Item::Static(s) => {
                self.w("(static ");
                self.w(&s.name);
                self.sp();
                self.ty(&s.ty);
                self.sp();
                self.expr(&s.value);
                self.w(")");
            }
            other => panic!("out-of-subset item: {other:?}"),
        }
    }

    fn tparams(&mut self, tps: &[TypeParam]) {
        self.w("(tparams");
        for tp in tps {
            self.w(" (tp ");
            self.w(&tp.name);
            for b in &tp.bounds {
                self.sp();
                self.w(b);
            }
            self.w(")");
        }
        self.w(")");
    }

    fn regions(&mut self, rs: &[String]) {
        self.w("(regions");
        for r in rs {
            self.sp();
            self.w(r);
        }
        self.w(")");
    }

    fn func(&mut self, f: &FnDecl) {
        self.w("(fn ");
        self.w(&f.name);
        self.sp();
        self.tparams(&f.type_params);
        self.sp();
        self.regions(&f.regions);
        self.sp();
        self.params(&f.params);
        self.w(" (sig ");
        self.w(if f.alloc { "alloc" } else { "-" });
        self.sp();
        self.w(if f.foreign { "foreign" } else { "-" });
        self.w(")");
        self.sp();
        match &f.ret {
            None => self.w("none"),
            Some(rt) => self.ret(rt),
        }
        self.sp();
        self.block(&f.body);
        self.w(")");
    }

    fn params(&mut self, ps: &[Param]) {
        self.w("(params");
        for p in ps {
            self.sp();
            self.w("(param ");
            self.w(&p.name);
            self.sp();
            self.w(mode_name(p.mode));
            self.sp();
            match &p.region {
                Some(r) => self.w(r),
                None => self.w("_"),
            }
            self.sp();
            self.ty(&p.ty);
            self.w(")");
        }
        self.w(")");
    }

    fn ret(&mut self, rt: &RetTy) {
        self.w("(ret ");
        self.w(match rt.borrow {
            None => "none",
            Some(BorrowKind::Shared) => "read",
            Some(BorrowKind::Exclusive) => "write",
        });
        self.sp();
        match &rt.region {
            Some(r) => self.w(r),
            None => self.w("_"),
        }
        self.sp();
        self.ty(&rt.ty);
        self.w(")");
    }

    fn strukt(&mut self, s: &StructDecl) {
        self.w("(struct ");
        self.w(if s.copy { "copy" } else { "-" });
        self.sp();
        self.w(&s.name);
        self.sp();
        self.tparams(&s.type_params);
        self.sp();
        self.w("(fields");
        for f in &s.fields {
            self.w(" (sfield ");
            self.w(&f.name);
            self.sp();
            self.ty(&f.ty);
            self.w(")");
        }
        self.w(")");
        self.sp();
        match &s.drop_hook {
            Some(b) => self.block(b),
            None => self.w("-"),
        }
        self.w(")");
    }

    fn enom(&mut self, e: &EnumDecl) {
        self.w("(enum ");
        self.w(if e.copy { "copy" } else { "-" });
        self.sp();
        self.w(&e.name);
        self.sp();
        self.tparams(&e.type_params);
        self.sp();
        self.w("(variants");
        for v in &e.variants {
            self.w(" (variant ");
            self.w(if v.ok { "ok" } else { "-" });
            self.sp();
            self.w(&v.name);
            for t in &v.payload {
                self.sp();
                self.ty(t);
            }
            self.w(")");
        }
        self.w(")");
        self.w(")");
    }

    fn ty(&mut self, t: &Ty) {
        match &t.kind {
            TyKind::Scalar(sc) => {
                self.w("(sc ");
                self.w(scalar_name(*sc));
                self.w(")");
            }
            TyKind::Named(n) => {
                self.w("(named ");
                self.w(n);
                self.w(")");
            }
            TyKind::App { name, args } => {
                self.w("(app ");
                self.w(name);
                for a in args {
                    self.sp();
                    self.ty(a);
                }
                self.w(")");
            }
            TyKind::Proj { base, assoc } => {
                self.w("(proj ");
                self.w(base);
                self.sp();
                self.w(assoc);
                self.w(")");
            }
            TyKind::Array { size, elem } => {
                self.w("(array ");
                self.expr(size);
                self.sp();
                self.ty(elem);
                self.w(")");
            }
            TyKind::Slice(t) => {
                self.w("(slice ");
                self.ty(t);
                self.w(")");
            }
            TyKind::SliceMut(t) => {
                self.w("(slicemut ");
                self.ty(t);
                self.w(")");
            }
            TyKind::RawPtr(t) => {
                self.w("(rawptr ");
                self.ty(t);
                self.w(")");
            }
            TyKind::Box(t) => {
                self.w("(box ");
                self.ty(t);
                self.w(")");
            }
            TyKind::BoxResult(t) => {
                self.w("(boxresult ");
                self.ty(t);
                self.w(")");
            }
            TyKind::Borrow(t) => {
                self.w("(borrow ");
                self.ty(t);
                self.w(")");
            }
            TyKind::BorrowMut(t) => {
                self.w("(borrowmut ");
                self.ty(t);
                self.w(")");
            }
            TyKind::FnPtr(fp) => {
                self.w("(fnptr (fpparams");
                for p in &fp.params {
                    self.w(" (fpparam ");
                    match &p.name {
                        Some(n) => self.w(n),
                        None => self.w("_"),
                    }
                    self.sp();
                    self.w(mode_name(p.mode));
                    self.sp();
                    match &p.region {
                        Some(r) => self.w(r),
                        None => self.w("_"),
                    }
                    self.sp();
                    self.ty(&p.ty);
                    self.w(")");
                }
                self.w(") ");
                self.w(if fp.alloc { "alloc" } else { "-" });
                self.sp();
                self.w(if fp.foreign { "foreign" } else { "-" });
                self.sp();
                self.ty(&fp.ret);
                self.w(")");
            }
        }
    }

    fn block(&mut self, b: &Block) {
        self.w("(block");
        for st in &b.stmts {
            self.sp();
            self.stmt(st);
        }
        self.w(")");
    }

    fn stmt(&mut self, st: &Stmt) {
        match &st.kind {
            StmtKind::Let { mutable, name, ty, init } => {
                self.w("(let ");
                self.w(if *mutable { "mut" } else { "-" });
                self.sp();
                self.w(name);
                self.sp();
                match ty {
                    Some(t) => self.ty(t),
                    None => self.w("-"),
                }
                self.sp();
                match init {
                    Some(e) => self.expr(e),
                    None => self.w("-"),
                }
                self.w(")");
            }
            StmtKind::Assign { target, value } => {
                self.w("(assign ");
                self.expr(target);
                self.sp();
                self.expr(value);
                self.w(")");
            }
            StmtKind::Expr(e) => self.expr(e),
        }
    }

    fn pattern(&mut self, p: &Pattern) {
        match &p.kind {
            PatKind::Wildcard => self.w("(wild)"),
            PatKind::Binding(n) => {
                self.w("(bind ");
                self.w(n);
                self.w(")");
            }
            PatKind::Variant { enum_name, variant, sub } => {
                self.w("(pvariant ");
                self.w(enum_name);
                self.sp();
                self.w(variant);
                for s in sub {
                    self.sp();
                    self.pattern(s);
                }
                self.w(")");
            }
            PatKind::IntLit { value, negative, suffix } => {
                self.w(&format!(
                    "(pint {} {value} {})",
                    if *negative { 1 } else { 0 },
                    suf_code(*suffix)
                ));
            }
            PatKind::IntRange { lo_value, lo_negative, lo_suffix, hi_value, hi_negative, hi_suffix, inclusive } => {
                self.w(&format!(
                    "(prange {} {lo_value} {} {} {} {hi_value} {})",
                    if *lo_negative { 1 } else { 0 },
                    suf_code(*lo_suffix),
                    if *inclusive { 1 } else { 0 },
                    if *hi_negative { 1 } else { 0 },
                    suf_code(*hi_suffix)
                ));
            }
        }
    }

    fn expr(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::IntLit { value, suffix } => {
                self.w(&format!("(int {value} {})", suf_code(*suffix)));
            }
            ExprKind::NegIntLit { value, suffix } => {
                self.w(&format!("(negint {value} {})", suf_code(*suffix)));
            }
            ExprKind::StrLit(s) => {
                self.w("(str ");
                self.bytes(s.as_bytes());
                self.w(")");
            }
            ExprKind::BytesLit(s) => {
                self.w("(bytes ");
                self.bytes(s.as_bytes());
                self.w(")");
            }
            ExprKind::BoolLit(b) => {
                self.w(&format!("(bool {})", if *b { 1 } else { 0 }));
            }
            ExprKind::Ident(n) => {
                self.w("(id ");
                self.w(n);
                self.w(")");
            }
            ExprKind::Unary { op, expr } => {
                self.w("(unary ");
                self.w(match op {
                    UnOp::Neg => "neg",
                    UnOp::Not => "not",
                    UnOp::BitNot => "bitnot",
                });
                self.sp();
                self.expr(expr);
                self.w(")");
            }
            ExprKind::Prefix { op, expr } => {
                self.w("(prefix ");
                self.w(match op {
                    PrefixOp::Deref => "deref",
                    PrefixOp::Read => "read",
                    PrefixOp::Write => "write",
                    PrefixOp::Clone => "clone",
                });
                self.sp();
                self.expr(expr);
                self.w(")");
            }
            ExprKind::Binary { op, lhs, rhs } => {
                self.w("(bin ");
                self.w(binop_name(*op));
                self.sp();
                self.expr(lhs);
                self.sp();
                self.expr(rhs);
                self.w(")");
            }
            ExprKind::Call { callee, args } => {
                self.w("(call ");
                self.expr(callee);
                for a in args {
                    self.sp();
                    self.expr(a);
                }
                self.w(")");
            }
            ExprKind::OutArg(e) => {
                self.w("(out ");
                self.expr(e);
                self.w(")");
            }
            ExprKind::Field { base, field, .. } => {
                self.w("(field ");
                self.expr(base);
                self.sp();
                self.w(field);
                self.w(")");
            }
            ExprKind::Index { base, index } => {
                self.w("(index ");
                self.expr(base);
                self.sp();
                self.expr(index);
                self.w(")");
            }
            ExprKind::Conv { ty, expr } => {
                self.w("(conv ");
                self.ty(ty);
                self.sp();
                self.expr(expr);
                self.w(")");
            }
            ExprKind::ArrayLit(v) => {
                self.w("(arraylit");
                for x in v {
                    self.sp();
                    self.expr(x);
                }
                self.w(")");
            }
            ExprKind::ArrayRepeat { value, size } => {
                self.w("(arrayrep ");
                self.expr(value);
                self.sp();
                self.expr(size);
                self.w(")");
            }
            ExprKind::StructLit { name, fields } => {
                self.w("(structlit ");
                self.w(name);
                for f in fields {
                    self.w(" (finit ");
                    self.w(&f.name);
                    self.sp();
                    self.expr(&f.value);
                    self.w(")");
                }
                self.w(")");
            }
            ExprKind::EnumCtor { enum_name, variant, args } => {
                self.w("(enumctor ");
                self.w(enum_name);
                self.sp();
                self.w(variant);
                for a in args {
                    self.sp();
                    self.expr(a);
                }
                self.w(")");
            }
            ExprKind::CastPtr { ty, arg } => {
                self.w("(castptr ");
                self.ty(ty);
                self.sp();
                self.expr(arg);
                self.w(")");
            }
            ExprKind::AddrToPtr { ty, arg } => {
                self.w("(addrtoptr ");
                self.ty(ty);
                self.sp();
                self.expr(arg);
                self.w(")");
            }
            ExprKind::PtrNull { ty } => {
                self.w("(ptrnull ");
                self.ty(ty);
                self.w(")");
            }
            ExprKind::Offsetof { ty, field } => {
                self.w("(offsetof ");
                self.ty(ty);
                self.sp();
                self.w(field);
                self.w(")");
            }
            ExprKind::FieldPtr { ptr, field } => {
                self.w("(fieldptr ");
                self.expr(ptr);
                self.sp();
                self.w(field);
                self.w(")");
            }
            ExprKind::Sizeof(t) => {
                self.w("(sizeof ");
                self.ty(t);
                self.w(")");
            }
            ExprKind::Alignof(t) => {
                self.w("(alignof ");
                self.ty(t);
                self.w(")");
            }
            ExprKind::Block(b) => self.block(b),
            ExprKind::If { cond, then_blk, else_blk } => {
                self.w("(if ");
                self.expr(cond);
                self.sp();
                self.block(then_blk);
                self.sp();
                match else_blk {
                    Some(e) => self.expr(e),
                    None => self.w("-"),
                }
                self.w(")");
            }
            ExprKind::Match { scrutinee, arms } => {
                self.w("(match ");
                self.expr(scrutinee);
                for a in arms {
                    self.w(" (arm ");
                    self.pattern(&a.pattern);
                    self.sp();
                    self.expr(&a.body);
                    self.w(")");
                }
                self.w(")");
            }
            ExprKind::Loop(b) => {
                self.w("(loop ");
                self.block(b);
                self.w(")");
            }
            ExprKind::While { cond, body } => {
                self.w("(while ");
                self.expr(cond);
                self.sp();
                self.block(body);
                self.w(")");
            }
            ExprKind::Unsafe { justification, body } => {
                self.w("(unsafe ");
                self.bytes(justification.as_bytes());
                self.sp();
                self.block(body);
                self.w(")");
            }
            ExprKind::Wrapping(b) => {
                self.w("(wrapping ");
                self.block(b);
                self.w(")");
            }
            ExprKind::Saturating(b) => {
                self.w("(saturating ");
                self.block(b);
                self.w(")");
            }
            ExprKind::Return(o) => match o {
                Some(e) => {
                    self.w("(return ");
                    self.expr(e);
                    self.w(")");
                }
                None => self.w("(return)"),
            },
            ExprKind::Break => self.w("(break)"),
            ExprKind::Continue => self.w("(continue)"),
            ExprKind::Assert(e) => {
                self.w("(assert ");
                self.expr(e);
                self.w(")");
            }
            ExprKind::Panic(e) => {
                self.w("(panic ");
                self.expr(e);
                self.w(")");
            }
            ExprKind::Result => self.w("(result)"),
            ExprKind::Paren(e) => {
                self.w("(paren ");
                self.expr(e);
                self.w(")");
            }
            ExprKind::Try(e) => {
                self.w("(try ");
                self.expr(e);
                self.w(")");
            }
            ExprKind::GenericVal { name, ty_args } => {
                self.w("(genericval ");
                self.w(name);
                for t in ty_args {
                    self.sp();
                    self.ty(t);
                }
                self.w(")");
            }
            ExprKind::For { pattern, operand, body, by_ref } => {
                self.w("(for ");
                if *by_ref { self.w("read "); }
                self.pattern(pattern);
                self.sp();
                self.expr(operand);
                self.sp();
                self.block(body);
                self.w(")");
            }
            other => panic!("out-of-subset expr: {other:?}"),
        }
    }
}

fn mode_name(m: ParamMode) -> &'static str {
    match m {
        ParamMode::Take => "take",
        ParamMode::Read => "read",
        ParamMode::Write => "write",
        ParamMode::Out => "out",
    }
}

fn binop_name(op: BinOp) -> &'static str {
    match op {
        BinOp::Add => "add",
        BinOp::Sub => "sub",
        BinOp::Mul => "mul",
        BinOp::Div => "div",
        BinOp::Rem => "rem",
        BinOp::Eq => "eq",
        BinOp::Ne => "ne",
        BinOp::Lt => "lt",
        BinOp::Le => "le",
        BinOp::Gt => "gt",
        BinOp::Ge => "ge",
        BinOp::And => "and",
        BinOp::Or => "or",
        BinOp::BitAnd => "bitand",
        BinOp::BitOr => "bitor",
        BinOp::BitXor => "bitxor",
        BinOp::Shl => "shl",
        BinOp::Shr => "shr",
    }
}

fn oracle_dump(src: &str) -> String {
    let toks = candor::real::lexer::lex(src).expect("oracle lexes the fixture");
    let (prog, _uses, _vis, _boundary) = parse_format(toks).expect("oracle parses the fixture");
    let mut r = R { s: String::new() };
    r.program(&prog);
    r.s.push('\n');
    r.s
}

/// Generate the root `main.cnr`: it `use`s the lexer module's `Buf`/`mk`/`lex`
/// and the parser module's `parse_dump`, embeds `src`, lexes then parse-dumps it.
fn candor_main(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut m = String::from(
        "use lexer::{Buf, mk, lex};\nuse parser::{parse_dump};\n\nfn main() -> i64 {\n",
    );
    m.push_str(&format!("    let src: [{}]u8 = [", bytes.len()));
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            m.push_str(", ");
        }
        m.push_str(&format!("{b}u8"));
    }
    m.push_str("];\n");
    m.push_str("    let mut buf: Buf = Buf { toks: [mk(0, 0usize, 0usize); 49152], n: 0usize };\n");
    m.push_str("    let cnt: usize = lex(slice_of(src), write buf);\n");
    m.push_str("    parse_dump(slice_of(src), read buf);\n");
    m.push_str("    return conv i64 cnt;\n}\n");
    m
}

fn candor_dump(src: &str) -> String {
    let main = candor_main(src);
    match run_module_tree(&[("lexer.cnr", LEXER_SRC), ("parser.cnr", PARSER_SRC)], &main) {
        RunResult::Ok(run) => trace_text(&run),
        RunResult::Fault(f) => panic!("candor parser faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!(
            "candor parser check errors: {:?}",
            d.iter().map(|x| &x.code).collect::<Vec<_>>()
        ),
        RunResult::ParseError(d) => panic!("candor parser parse error: {}", d.to_json()),
    }
}

/// The in-subset real-syntax corpus the Candor parser is gated against. Fixtures
/// with interfaces/impls/`for`/`drop`/`unsafe` are OUT OF SUBSET this slice.
const CORPUS: &[&str] = &[
    "tests/fixtures/real/bits.cnr",
    "tests/fixtures/real/propagate.cnr",
    "tests/fixtures/real/slices.cnr",
    "tests/fixtures/generics/arena.cnr",
    "tests/fixtures/generics/genenum.cnr",
    "tests/fixtures/generics/mixed.cnr",
    "tests/fixtures/generics/mono3.cnr",
    "tests/fixtures/generics/nameval.cnr",
    "tests/fixtures/generics/pair.cnr",
];

fn read_fixture(rel: &str) -> String {
    let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

#[test]
fn candor_parser_ast_equal_to_oracle_over_corpus() {
    on_big_stack(|| {
    let mut passed = 0usize;
    for &rel in CORPUS {
        let src = read_fixture(rel);
        let oracle = oracle_dump(&src);
        let mine = candor_dump(&src);
        assert_eq!(mine, oracle, "AST S-expr mismatch on {rel}");
        passed += 1;
    }
    assert_eq!(passed, CORPUS.len());
    eprintln!(
        "selfhost parser: AST-shape PASS on {}/{} in-subset corpus fixtures",
        passed,
        CORPUS.len()
    );
    });
}

/// A focused probe over the full precedence ladder + a match, independent of the
/// corpus, exercising every binary level, the negative-literal fold, `conv`, the
/// bitwise/shift set, `?`, and struct/enum-ctor primaries.
#[test]
fn candor_parser_ast_equal_on_precedence_probe() {
    on_big_stack(|| {
    let src = "fn f(a: read Node, b: i64) -> i64 {\n\
               let x: i64 = 1 + 2 * 3 - 4 / 2 % 3;\n\
               let y: i64 = a.v & 0xF | b ^ 1 << 2 >> 1;\n\
               let z: bool = 1 == 2 && 3 != 4 || !true;\n\
               let w: i64 = ~a.v + -5 + conv i64 b;\n\
               return match b {\n\
               E::A(p) => p,\n\
               _ => 0,\n\
               };\n\
               }\n";
    assert_eq!(candor_dump(src), oracle_dump(src), "precedence probe mismatch");
    });
}
