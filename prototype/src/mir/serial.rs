//! The MIR serialization boundary (self-lowering L0): a canonical, deterministic,
//! human-readable text wire format for a whole [`MirProgram`], with a faithful
//! serializer/deserializer pair.
//!
//! ## Why it exists
//! The self-lowering tier lets a Candor program lower an AST to MIR; the Rust MIR
//! interpreter then runs that MIR, gated byte-exact against the tree-walking
//! oracle. Before any Candor lowering exists, this boundary is built and PROVEN
//! over the *existing* Rust lowering ([`super::build`]): serialize a real
//! `MirProgram`, deserialize it, run it through [`super::interp`], and compare to
//! the oracle. `serialize` is deterministic and `deserialize` is its exact inverse
//! (`serialize(deserialize(serialize(p))) == serialize(p)`), so the wire losslessly
//! carries every field the interpreter reads from `MirProgram`.
//!
//! ## What the wire carries
//! Everything in `MirProgram` *except* the derived `fn_index` (rebuilt from `fns`
//! order on load) and the runtime-only `items`/`consts` (those come from the
//! fixture SOURCE, not the wire — the harness rebuilds them). Every fn, block,
//! statement, terminator, rvalue, operand, place, projection; each `LocalDecl`
//! (type + name + drop obligation); each fault edge (kind + span.start/span.end,
//! LOAD-BEARING — the FAULT comparison checks span identity); projection offsets
//! and index strides/lengths/spans; the `Drop` move masks; the `fn_ptrs` table
//! (order = id, load-bearing for indirect calls); `drop_hooks` (sorted by key for
//! determinism); and the `statics` table.
//!
//! ## The format
//! A uniform S-expression: whitespace-insensitive on read, canonically indented on
//! write. Atoms are bare keywords/decimal integers; strings (names, string
//! literals) are `"…"`-quoted with `\\ \" \n \t` escapes. This is deliberately
//! simple to emit — a future Candor lowering (L1) emits exactly this text.

use std::collections::HashMap;
use std::str::FromStr;

use crate::ast::{BinOp, ParamMode, UnOp};
use crate::interp::FaultKind;
use crate::span::Span;
use crate::token::ScalarTy;
use crate::types::{ArrayLen, FnPtrTy, Type};

use super::{
    BasicBlock, CollOp, FaultEdge, LocalDecl, MirFn, MirProgram, Operand, Place, Predicate, Proj,
    Regime, ReplayPolicy, Rvalue, StaticInit, Statement, StatementKind, Terminator,
};

// ---------------------------------------------------------------------------
// S-expression substrate (the neutral text carrier).
// ---------------------------------------------------------------------------

enum Sexp {
    Atom(String),
    Str(String),
    List(Vec<Sexp>),
}

fn a(s: impl Into<String>) -> Sexp {
    Sexp::Atom(s.into())
}
fn n(v: impl ToString) -> Sexp {
    Sexp::Atom(v.to_string())
}
fn s(v: &str) -> Sexp {
    Sexp::Str(v.to_string())
}
fn l(v: Vec<Sexp>) -> Sexp {
    Sexp::List(v)
}
fn bool_atom(b: bool) -> Sexp {
    Sexp::Atom(if b { "true" } else { "false" }.into())
}

impl Sexp {
    fn as_list(&self) -> Result<&[Sexp], String> {
        match self {
            Sexp::List(v) => Ok(v),
            _ => Err("expected a list".into()),
        }
    }
    fn as_atom(&self) -> Result<&str, String> {
        match self {
            Sexp::Atom(s) => Ok(s),
            _ => Err("expected an atom".into()),
        }
    }
    fn as_str(&self) -> Result<&str, String> {
        match self {
            Sexp::Str(s) => Ok(s),
            _ => Err("expected a quoted string".into()),
        }
    }
    fn num<T: FromStr>(&self) -> Result<T, String> {
        self.as_atom()?.parse().map_err(|_| "malformed number".to_string())
    }
    fn as_bool(&self) -> Result<bool, String> {
        match self.as_atom()? {
            "true" => Ok(true),
            "false" => Ok(false),
            other => Err(format!("expected a bool, got {other}")),
        }
    }
}

/// A list whose head atom is `tag`, returning its trailing children.
fn tagged<'x>(sx: &'x Sexp, tag: &str) -> Result<&'x [Sexp], String> {
    let items = sx.as_list()?;
    match items.first() {
        Some(Sexp::Atom(head)) if head == tag => Ok(&items[1..]),
        Some(Sexp::Atom(head)) => Err(format!("expected `{tag}`, got `{head}`")),
        _ => Err(format!("expected a `{tag}` list")),
    }
}

/// The head atom of a list (the discriminator tag) plus its trailing children.
fn head(sx: &Sexp) -> Result<(&str, &[Sexp]), String> {
    let items = sx.as_list()?;
    match items.first() {
        Some(Sexp::Atom(h)) => Ok((h, &items[1..])),
        _ => Err("expected a tagged list".into()),
    }
}

/// Bounds-checked positional access into a tagged list's children. A truncated
/// wire (fewer args than a variant needs) yields a descriptive `Err` instead of
/// an out-of-bounds panic — the deserializer's `Result` contract made honest now
/// that a Candor-emitted wire feeds it.
fn arg(args: &[Sexp], i: usize) -> Result<&Sexp, String> {
    args.get(i)
        .ok_or_else(|| format!("wire truncated: need arg #{i}, have {}", args.len()))
}

// ---------------------------------------------------------------------------
// Printer: canonical, deterministic indentation.
// ---------------------------------------------------------------------------

fn print_sexp(sx: &Sexp, indent: usize, out: &mut String) {
    match sx {
        Sexp::Atom(text) => out.push_str(text),
        Sexp::Str(text) => {
            out.push('"');
            for c in text.chars() {
                match c {
                    '\\' => out.push_str("\\\\"),
                    '"' => out.push_str("\\\""),
                    '\n' => out.push_str("\\n"),
                    '\t' => out.push_str("\\t"),
                    _ => out.push(c),
                }
            }
            out.push('"');
        }
        Sexp::List(items) => {
            let multiline = items.iter().any(|c| matches!(c, Sexp::List(sub) if !sub.is_empty()));
            out.push('(');
            if let Some((first, rest)) = items.split_first() {
                print_sexp(first, indent, out);
                for child in rest {
                    if multiline {
                        out.push('\n');
                        for _ in 0..indent + 2 {
                            out.push(' ');
                        }
                        print_sexp(child, indent + 2, out);
                    } else {
                        out.push(' ');
                        print_sexp(child, indent, out);
                    }
                }
            }
            out.push(')');
        }
    }
}

// ---------------------------------------------------------------------------
// Tokenizer + reader: whitespace-insensitive parse back to Sexp.
// ---------------------------------------------------------------------------

fn parse_sexp(text: &str) -> Result<Sexp, String> {
    let mut chars = text.chars().peekable();
    let sx = read_sexp(&mut chars)?;
    skip_ws(&mut chars);
    if chars.peek().is_some() {
        return Err("trailing content after top-level s-expression".into());
    }
    Ok(sx)
}

fn skip_ws(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

fn read_sexp(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<Sexp, String> {
    skip_ws(chars);
    match chars.peek().copied() {
        None => Err("unexpected end of input".into()),
        Some('(') => {
            chars.next();
            let mut items = Vec::new();
            loop {
                skip_ws(chars);
                match chars.peek().copied() {
                    None => return Err("unterminated list".into()),
                    Some(')') => {
                        chars.next();
                        return Ok(Sexp::List(items));
                    }
                    _ => items.push(read_sexp(chars)?),
                }
            }
        }
        Some(')') => Err("unexpected `)`".into()),
        Some('"') => {
            chars.next();
            let mut out = String::new();
            loop {
                match chars.next() {
                    None => return Err("unterminated string".into()),
                    Some('"') => return Ok(Sexp::Str(out)),
                    Some('\\') => match chars.next() {
                        Some('\\') => out.push('\\'),
                        Some('"') => out.push('"'),
                        Some('n') => out.push('\n'),
                        Some('t') => out.push('\t'),
                        other => return Err(format!("bad escape: \\{:?}", other)),
                    },
                    Some(c) => out.push(c),
                }
            }
        }
        Some(_) => {
            let mut out = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() || c == '(' || c == ')' || c == '"' {
                    break;
                }
                out.push(c);
                chars.next();
            }
            Ok(Sexp::Atom(out))
        }
    }
}

// ---------------------------------------------------------------------------
// Enum keyword tables (small, closed sets).
// ---------------------------------------------------------------------------

fn scalar_kw(t: ScalarTy) -> &'static str {
    match t {
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
fn scalar_from(kw: &str) -> Result<ScalarTy, String> {
    Ok(match kw {
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
        "bool" => ScalarTy::Bool,
        "unit" => ScalarTy::Unit,
        "f64" => ScalarTy::F64,
        "f32" => ScalarTy::F32,
        other => return Err(format!("unknown scalar type `{other}`")),
    })
}

fn binop_kw(op: BinOp) -> &'static str {
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
fn binop_from(kw: &str) -> Result<BinOp, String> {
    Ok(match kw {
        "add" => BinOp::Add,
        "sub" => BinOp::Sub,
        "mul" => BinOp::Mul,
        "div" => BinOp::Div,
        "rem" => BinOp::Rem,
        "eq" => BinOp::Eq,
        "ne" => BinOp::Ne,
        "lt" => BinOp::Lt,
        "le" => BinOp::Le,
        "gt" => BinOp::Gt,
        "ge" => BinOp::Ge,
        "and" => BinOp::And,
        "or" => BinOp::Or,
        "bitand" => BinOp::BitAnd,
        "bitor" => BinOp::BitOr,
        "bitxor" => BinOp::BitXor,
        "shl" => BinOp::Shl,
        "shr" => BinOp::Shr,
        other => return Err(format!("unknown binop `{other}`")),
    })
}

fn unop_kw(op: UnOp) -> &'static str {
    match op {
        UnOp::Neg => "neg",
        UnOp::Not => "not",
        UnOp::BitNot => "bitnot",
    }
}
fn unop_from(kw: &str) -> Result<UnOp, String> {
    Ok(match kw {
        "neg" => UnOp::Neg,
        "not" => UnOp::Not,
        "bitnot" => UnOp::BitNot,
        other => return Err(format!("unknown unop `{other}`")),
    })
}

fn regime_kw(r: Regime) -> &'static str {
    match r {
        Regime::Checked => "checked",
        Regime::Wrapping => "wrapping",
        Regime::Saturating => "saturating",
    }
}
fn regime_from(kw: &str) -> Result<Regime, String> {
    Ok(match kw {
        "checked" => Regime::Checked,
        "wrapping" => Regime::Wrapping,
        "saturating" => Regime::Saturating,
        other => return Err(format!("unknown regime `{other}`")),
    })
}

fn fault_kw(k: FaultKind) -> &'static str {
    match k {
        FaultKind::Overflow => "overflow",
        FaultKind::DivByZero => "divzero",
        FaultKind::Bounds => "bounds",
        FaultKind::ConvLoss => "convloss",
        FaultKind::Assert => "assert",
        FaultKind::Requires => "requires",
        FaultKind::Ensures => "ensures",
        FaultKind::Panic => "panic",
        FaultKind::BadPointer => "badpointer",
        FaultKind::NoForeignRuntime => "noforeignruntime",
    }
}
fn fault_from(kw: &str) -> Result<FaultKind, String> {
    Ok(match kw {
        "overflow" => FaultKind::Overflow,
        "divzero" => FaultKind::DivByZero,
        "bounds" => FaultKind::Bounds,
        "convloss" => FaultKind::ConvLoss,
        "assert" => FaultKind::Assert,
        "requires" => FaultKind::Requires,
        "ensures" => FaultKind::Ensures,
        "panic" => FaultKind::Panic,
        "badpointer" => FaultKind::BadPointer,
        "noforeignruntime" => FaultKind::NoForeignRuntime,
        other => return Err(format!("unknown fault kind `{other}`")),
    })
}

fn parammode_kw(m: ParamMode) -> &'static str {
    match m {
        ParamMode::Take => "take",
        ParamMode::Read => "read",
        ParamMode::Write => "write",
        ParamMode::Out => "out",
    }
}
fn parammode_from(kw: &str) -> Result<ParamMode, String> {
    Ok(match kw {
        "take" => ParamMode::Take,
        "read" => ParamMode::Read,
        "write" => ParamMode::Write,
        "out" => ParamMode::Out,
        other => return Err(format!("unknown param mode `{other}`")),
    })
}

// ---------------------------------------------------------------------------
// Type <-> Sexp (fully recursive; the wire must carry it losslessly).
// ---------------------------------------------------------------------------

fn ty_to(t: &Type) -> Sexp {
    match t {
        Type::Scalar(sc) => l(vec![a("scalar"), a(scalar_kw(*sc))]),
        Type::IntLit => l(vec![a("intlit")]),
        Type::Named(name) => l(vec![a("named"), s(name)]),
        Type::Param(name) => l(vec![a("param"), s(name)]),
        Type::App(name, args) => {
            let mut v = vec![a("app"), s(name)];
            v.extend(args.iter().map(ty_to));
            l(v)
        }
        Type::Proj(base, assoc) => l(vec![a("typroj"), s(base), s(assoc)]),
        Type::Array(elem, len) => l(vec![a("array"), ty_to(elem), arraylen_to(len)]),
        Type::Slice(inner) => l(vec![a("slice"), ty_to(inner)]),
        Type::SliceMut(inner) => l(vec![a("slicemut"), ty_to(inner)]),
        Type::Str => l(vec![a("str")]),
        Type::RawPtr(inner) => l(vec![a("rawptr"), ty_to(inner)]),
        Type::Box(inner) => l(vec![a("box"), ty_to(inner)]),
        Type::BoxResult(inner) => l(vec![a("boxresult"), ty_to(inner)]),
        Type::Borrow(inner) => l(vec![a("borrow"), ty_to(inner)]),
        Type::BorrowMut(inner) => l(vec![a("borrowmut"), ty_to(inner)]),
        Type::FnPtr(fp) => {
            let params =
                fp.params.iter().map(|(m, pt)| l(vec![a(parammode_kw(*m)), ty_to(pt)])).collect();
            l(vec![
                a("fnptr"),
                bool_atom(fp.alloc),
                bool_atom(fp.foreign),
                ty_to(&fp.ret),
                l(std::iter::once(a("params")).chain(std::iter::once(l(params))).collect()),
            ])
        }
        Type::Never => l(vec![a("never")]),
        Type::Error => l(vec![a("error")]),
    }
}

fn ty_from(sx: &Sexp) -> Result<Type, String> {
    let (tag, args) = head(sx)?;
    Ok(match tag {
        "scalar" => Type::Scalar(scalar_from(arg(args, 0)?.as_atom()?)?),
        "intlit" => Type::IntLit,
        "named" => Type::Named(arg(args, 0)?.as_str()?.to_string()),
        "param" => Type::Param(arg(args, 0)?.as_str()?.to_string()),
        "app" => {
            let name = arg(args, 0)?.as_str()?.to_string();
            let items = args[1..].iter().map(ty_from).collect::<Result<Vec<_>, _>>()?;
            Type::App(name, items)
        }
        "typroj" => Type::Proj(arg(args, 0)?.as_str()?.to_string(), arg(args, 1)?.as_str()?.to_string()),
        "array" => Type::Array(Box::new(ty_from(arg(args, 0)?)?), arraylen_from(arg(args, 1)?)?),
        "slice" => Type::Slice(Box::new(ty_from(arg(args, 0)?)?)),
        "slicemut" => Type::SliceMut(Box::new(ty_from(arg(args, 0)?)?)),
        "str" => Type::Str,
        "rawptr" => Type::RawPtr(Box::new(ty_from(arg(args, 0)?)?)),
        "box" => Type::Box(Box::new(ty_from(arg(args, 0)?)?)),
        "boxresult" => Type::BoxResult(Box::new(ty_from(arg(args, 0)?)?)),
        "borrow" => Type::Borrow(Box::new(ty_from(arg(args, 0)?)?)),
        "borrowmut" => Type::BorrowMut(Box::new(ty_from(arg(args, 0)?)?)),
        "fnptr" => {
            let alloc = arg(args, 0)?.as_bool()?;
            let foreign = arg(args, 1)?.as_bool()?;
            let ret = Box::new(ty_from(arg(args, 2)?)?);
            let param_items = tagged(arg(args, 3)?, "params")?;
            let params = arg(param_items, 0)?
                .as_list()?
                .iter()
                .map(|p| {
                    let (m, rest) = head(p)?;
                    Ok((parammode_from(m)?, ty_from(arg(rest, 0)?)?))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Type::FnPtr(FnPtrTy { params, alloc, foreign, ret })
        }
        "never" => Type::Never,
        "error" => Type::Error,
        other => return Err(format!("unknown type tag `{other}`")),
    })
}

fn arraylen_to(len: &ArrayLen) -> Sexp {
    match len {
        ArrayLen::Lit(v) => l(vec![a("litlen"), n(*v)]),
        ArrayLen::Named(name) => l(vec![a("namedlen"), s(name)]),
        ArrayLen::Unknown => l(vec![a("unknownlen")]),
    }
}
fn arraylen_from(sx: &Sexp) -> Result<ArrayLen, String> {
    let (tag, args) = head(sx)?;
    Ok(match tag {
        "litlen" => ArrayLen::Lit(arg(args, 0)?.num()?),
        "namedlen" => ArrayLen::Named(arg(args, 0)?.as_str()?.to_string()),
        "unknownlen" => ArrayLen::Unknown,
        other => return Err(format!("unknown array-len tag `{other}`")),
    })
}

// ---------------------------------------------------------------------------
// Operand / Place / Proj / Rvalue.
// ---------------------------------------------------------------------------

fn operand_to(op: &Operand) -> Sexp {
    match op {
        Operand::Const(v, sc) => l(vec![a("const"), n(*v), a(scalar_kw(*sc))]),
        Operand::Local(id) => l(vec![a("oplocal"), n(*id)]),
    }
}
fn operand_from(sx: &Sexp) -> Result<Operand, String> {
    let (tag, args) = head(sx)?;
    Ok(match tag {
        "const" => Operand::Const(arg(args, 0)?.num()?, scalar_from(arg(args, 1)?.as_atom()?)?),
        "oplocal" => Operand::Local(arg(args, 0)?.num()?),
        other => return Err(format!("unknown operand tag `{other}`")),
    })
}

fn proj_to(p: &Proj) -> Sexp {
    match p {
        Proj::Field { offset, ty } => l(vec![a("field"), n(*offset), ty_to(ty)]),
        Proj::Deref { inner } => l(vec![a("deref"), ty_to(inner)]),
        Proj::Index { index, stride, len, span, slice } => l(vec![
            a("index"),
            operand_to(index),
            n(*stride),
            n(*len),
            n(span.start),
            n(span.end),
            bool_atom(*slice),
        ]),
    }
}
fn proj_from(sx: &Sexp) -> Result<Proj, String> {
    let (tag, args) = head(sx)?;
    Ok(match tag {
        "field" => Proj::Field { offset: arg(args, 0)?.num()?, ty: ty_from(arg(args, 1)?)? },
        "deref" => Proj::Deref { inner: ty_from(arg(args, 0)?)? },
        "index" => Proj::Index {
            index: operand_from(arg(args, 0)?)?,
            stride: arg(args, 1)?.num()?,
            len: arg(args, 2)?.num()?,
            span: Span { start: arg(args, 3)?.num()?, end: arg(args, 4)?.num()? },
            slice: arg(args, 5)?.as_bool()?,
        },
        other => return Err(format!("unknown proj tag `{other}`")),
    })
}

fn place_to(p: &Place) -> Sexp {
    let proj = std::iter::once(a("proj")).chain(p.proj.iter().map(proj_to)).collect();
    l(vec![a("place"), n(p.root), l(proj)])
}
fn place_from(sx: &Sexp) -> Result<Place, String> {
    let args = tagged(sx, "place")?;
    let root = arg(args, 0)?.num()?;
    let proj = tagged(arg(args, 1)?, "proj")?.iter().map(proj_from).collect::<Result<Vec<_>, _>>()?;
    Ok(Place { root, proj })
}

fn faultedge_to(fe: &FaultEdge) -> Sexp {
    l(vec![a("fedge"), a(fault_kw(fe.kind)), n(fe.span.start), n(fe.span.end)])
}
fn faultedge_from(sx: &Sexp) -> Result<FaultEdge, String> {
    let args = tagged(sx, "fedge")?;
    Ok(FaultEdge {
        kind: fault_from(arg(args, 0)?.as_atom()?)?,
        span: Span { start: arg(args, 1)?.num()?, end: arg(args, 2)?.num()? },
    })
}

fn fault_opt_to(fe: &Option<FaultEdge>) -> Sexp {
    match fe {
        Some(e) => l(vec![a("some"), faultedge_to(e)]),
        None => l(vec![a("none")]),
    }
}
fn fault_opt_from(sx: &Sexp) -> Result<Option<FaultEdge>, String> {
    let (tag, args) = head(sx)?;
    Ok(match tag {
        "some" => Some(faultedge_from(arg(args, 0)?)?),
        "none" => None,
        other => return Err(format!("expected some/none, got `{other}`")),
    })
}

fn args_to(items: &[Operand]) -> Sexp {
    l(std::iter::once(a("args")).chain(items.iter().map(operand_to)).collect())
}
fn args_from(sx: &Sexp) -> Result<Vec<Operand>, String> {
    tagged(sx, "args")?.iter().map(operand_from).collect()
}

fn rvalue_to(rv: &Rvalue) -> Sexp {
    match rv {
        Rvalue::Use(op) => l(vec![a("use"), operand_to(op)]),
        Rvalue::Bin { op, regime, ty, l: lhs, r, span, fault } => l(vec![
            a("bin"),
            a(binop_kw(*op)),
            a(regime_kw(*regime)),
            a(scalar_kw(*ty)),
            operand_to(lhs),
            operand_to(r),
            n(span.start),
            n(span.end),
            fault_opt_to(fault),
        ]),
        Rvalue::Un { op, regime, ty, v, fault } => l(vec![
            a("un"),
            a(unop_kw(*op)),
            a(regime_kw(*regime)),
            a(scalar_kw(*ty)),
            operand_to(v),
            fault_opt_to(fault),
        ]),
        Rvalue::Cmp { op, l: lhs, r } => {
            l(vec![a("cmp"), a(binop_kw(*op)), operand_to(lhs), operand_to(r)])
        }
        Rvalue::Conv { to, regime, v, fault } => l(vec![
            a("conv"),
            a(scalar_kw(*to)),
            a(regime_kw(*regime)),
            operand_to(v),
            fault_opt_to(fault),
        ]),
        Rvalue::Bitcast { to, v } => l(vec![a("bitcast"), a(scalar_kw(*to)), operand_to(v)]),
        Rvalue::Sqrt { ty, v } => l(vec![a("sqrt"), a(scalar_kw(*ty)), operand_to(v)]),
        Rvalue::Ref(place) => l(vec![a("ref"), place_to(place)]),
        Rvalue::Load { place, ty } => l(vec![a("load"), place_to(place), ty_to(ty)]),
        Rvalue::Call { func, args } => l(vec![a("call"), s(func), args_to(args)]),
        Rvalue::CallIndirect { func, args } => {
            l(vec![a("callindirect"), operand_to(func), args_to(args)])
        }
        Rvalue::PtrArith { base, index, stride } => {
            l(vec![a("ptrarith"), operand_to(base), operand_to(index), n(*stride)])
        }
        Rvalue::IsNull(op) => l(vec![a("isnull"), operand_to(op)]),
        Rvalue::StaticAddr(name) => l(vec![a("staticaddr"), s(name)]),
        Rvalue::StrAddr(text) => l(vec![a("straddr"), s(text)]),
    }
}
fn rvalue_from(sx: &Sexp) -> Result<Rvalue, String> {
    let (tag, args) = head(sx)?;
    Ok(match tag {
        "use" => Rvalue::Use(operand_from(arg(args, 0)?)?),
        "bin" => Rvalue::Bin {
            op: binop_from(arg(args, 0)?.as_atom()?)?,
            regime: regime_from(arg(args, 1)?.as_atom()?)?,
            ty: scalar_from(arg(args, 2)?.as_atom()?)?,
            l: operand_from(arg(args, 3)?)?,
            r: operand_from(arg(args, 4)?)?,
            span: Span { start: arg(args, 5)?.num()?, end: arg(args, 6)?.num()? },
            fault: fault_opt_from(arg(args, 7)?)?,
        },
        "un" => Rvalue::Un {
            op: unop_from(arg(args, 0)?.as_atom()?)?,
            regime: regime_from(arg(args, 1)?.as_atom()?)?,
            ty: scalar_from(arg(args, 2)?.as_atom()?)?,
            v: operand_from(arg(args, 3)?)?,
            fault: fault_opt_from(arg(args, 4)?)?,
        },
        "cmp" => Rvalue::Cmp {
            op: binop_from(arg(args, 0)?.as_atom()?)?,
            l: operand_from(arg(args, 1)?)?,
            r: operand_from(arg(args, 2)?)?,
        },
        "conv" => Rvalue::Conv {
            to: scalar_from(arg(args, 0)?.as_atom()?)?,
            regime: regime_from(arg(args, 1)?.as_atom()?)?,
            v: operand_from(arg(args, 2)?)?,
            fault: fault_opt_from(arg(args, 3)?)?,
        },
        "bitcast" => Rvalue::Bitcast {
            to: scalar_from(arg(args, 0)?.as_atom()?)?,
            v: operand_from(arg(args, 1)?)?,
        },
        "sqrt" => Rvalue::Sqrt {
            ty: scalar_from(arg(args, 0)?.as_atom()?)?,
            v: operand_from(arg(args, 1)?)?,
        },
        "ref" => Rvalue::Ref(place_from(arg(args, 0)?)?),
        "load" => Rvalue::Load { place: place_from(arg(args, 0)?)?, ty: ty_from(arg(args, 1)?)? },
        "call" => Rvalue::Call { func: arg(args, 0)?.as_str()?.to_string(), args: args_from(arg(args, 1)?)? },
        "callindirect" => {
            Rvalue::CallIndirect { func: operand_from(arg(args, 0)?)?, args: args_from(arg(args, 1)?)? }
        }
        "ptrarith" => Rvalue::PtrArith {
            base: operand_from(arg(args, 0)?)?,
            index: operand_from(arg(args, 1)?)?,
            stride: arg(args, 2)?.num()?,
        },
        "isnull" => Rvalue::IsNull(operand_from(arg(args, 0)?)?),
        "staticaddr" => Rvalue::StaticAddr(arg(args, 0)?.as_str()?.to_string()),
        "straddr" => Rvalue::StrAddr(arg(args, 0)?.as_str()?.to_string()),
        other => return Err(format!("unknown rvalue tag `{other}`")),
    })
}

// ---------------------------------------------------------------------------
// Statement / Terminator / Block.
// ---------------------------------------------------------------------------

fn moved_to(moved: &[Vec<String>]) -> Sexp {
    let paths = moved
        .iter()
        .map(|p| l(std::iter::once(a("path")).chain(p.iter().map(|seg| s(seg))).collect()));
    l(std::iter::once(a("moved")).chain(paths).collect())
}
fn moved_from(sx: &Sexp) -> Result<Vec<Vec<String>>, String> {
    tagged(sx, "moved")?
        .iter()
        .map(|p| tagged(p, "path")?.iter().map(|seg| seg.as_str().map(String::from)).collect())
        .collect()
}

fn stmtkind_to(k: &StatementKind) -> Sexp {
    match k {
        StatementKind::Assign(local, rv) => l(vec![a("assign"), n(*local), rvalue_to(rv)]),
        StatementKind::Trace(op) => l(vec![a("trace"), operand_to(op)]),
        StatementKind::Store(place, rv) => l(vec![a("store"), place_to(place), rvalue_to(rv)]),
        StatementKind::CopyVal { dst, src, ty } => {
            l(vec![a("copyval"), place_to(dst), place_to(src), ty_to(ty)])
        }
        StatementKind::Drop { local, moved } => l(vec![a("drop"), n(*local), moved_to(moved)]),
        StatementKind::BoxOp { dst, inner_ty, result_ty, alloc, value } => l(vec![
            a("boxop"),
            place_to(dst),
            ty_to(inner_ty),
            ty_to(result_ty),
            operand_to(alloc),
            place_to(value),
        ]),
        StatementKind::UnboxOp { dst, inner_ty, boxed } => {
            l(vec![a("unboxop"), place_to(dst), ty_to(inner_ty), place_to(boxed)])
        }
        StatementKind::Subslice { dst, src, lo, hi, stride, span } => l(vec![
            a("subslice"),
            place_to(dst),
            place_to(src),
            operand_to(lo),
            operand_to(hi),
            n(*stride),
            n(span.start),
            n(span.end),
        ]),
        StatementKind::StrFrom { dst, src } => {
            l(vec![a("strfrom"), place_to(dst), place_to(src)])
        }
        StatementKind::Substr { dst, src, lo, hi, span } => l(vec![
            a("substr"),
            place_to(dst),
            place_to(src),
            operand_to(lo),
            operand_to(hi),
            n(span.start),
            n(span.end),
        ]),
        StatementKind::CollectionOp { dst, op } => l(vec![a("collop"), place_to(dst), collop_to(op)]),
        StatementKind::Spawn { func, args } => l(vec![a("spawn"), s(func), args_to(args)]),
        StatementKind::ScopeBegin => l(vec![a("scopebegin")]),
        StatementKind::ScopeEnd => l(vec![a("scopeend")]),
    }
}

fn collop_to(op: &CollOp) -> Sexp {
    match op {
        CollOp::New { alloc } => l(vec![a("new"), operand_to(alloc)]),
        CollOp::VecPush { base, elem, value, span } => l(vec![
            a("vecpush"), operand_to(base), ty_to(elem), place_to(value), n(span.start), n(span.end),
        ]),
        CollOp::VecPop { base, elem } => l(vec![a("vecpop"), operand_to(base), ty_to(elem)]),
        CollOp::VecGet { base, elem, index, span } => l(vec![
            a("vecget"), operand_to(base), ty_to(elem), operand_to(index), n(span.start), n(span.end),
        ]),
        CollOp::VecSet { base, elem, index, value, span } => l(vec![
            a("vecset"), operand_to(base), ty_to(elem), operand_to(index), place_to(value), n(span.start), n(span.end),
        ]),
        CollOp::MapInsert { base, valty, key, value, span } => l(vec![
            a("mapinsert"), operand_to(base), ty_to(valty), place_to(key), place_to(value), n(span.start), n(span.end),
        ]),
        CollOp::MapContains { base, valty, key } => l(vec![
            a("mapcontains"), operand_to(base), ty_to(valty), place_to(key),
        ]),
        CollOp::MapGet { base, valty, key, span } => l(vec![
            a("mapget"), operand_to(base), ty_to(valty), place_to(key), n(span.start), n(span.end),
        ]),
        CollOp::StringPush { base, ch, span } => l(vec![
            a("stringpush"), operand_to(base), operand_to(ch), n(span.start), n(span.end),
        ]),
        CollOp::StringAppend { base, view, span } => l(vec![
            a("stringappend"), operand_to(base), place_to(view), n(span.start), n(span.end),
        ]),
        CollOp::StringAsStr { base } => l(vec![a("stringasstr"), operand_to(base)]),
    }
}
fn collop_from(sx: &Sexp) -> Result<CollOp, String> {
    let (tag, args) = head(sx)?;
    Ok(match tag {
        "new" => CollOp::New { alloc: operand_from(arg(args, 0)?)? },
        "vecpush" => CollOp::VecPush {
            base: operand_from(arg(args, 0)?)?,
            elem: ty_from(arg(args, 1)?)?,
            value: place_from(arg(args, 2)?)?,
            span: Span { start: arg(args, 3)?.num()?, end: arg(args, 4)?.num()? },
        },
        "vecpop" => CollOp::VecPop { base: operand_from(arg(args, 0)?)?, elem: ty_from(arg(args, 1)?)? },
        "vecget" => CollOp::VecGet {
            base: operand_from(arg(args, 0)?)?,
            elem: ty_from(arg(args, 1)?)?,
            index: operand_from(arg(args, 2)?)?,
            span: Span { start: arg(args, 3)?.num()?, end: arg(args, 4)?.num()? },
        },
        "vecset" => CollOp::VecSet {
            base: operand_from(arg(args, 0)?)?,
            elem: ty_from(arg(args, 1)?)?,
            index: operand_from(arg(args, 2)?)?,
            value: place_from(arg(args, 3)?)?,
            span: Span { start: arg(args, 4)?.num()?, end: arg(args, 5)?.num()? },
        },
        "mapinsert" => CollOp::MapInsert {
            base: operand_from(arg(args, 0)?)?,
            valty: ty_from(arg(args, 1)?)?,
            key: place_from(arg(args, 2)?)?,
            value: place_from(arg(args, 3)?)?,
            span: Span { start: arg(args, 4)?.num()?, end: arg(args, 5)?.num()? },
        },
        "mapcontains" => CollOp::MapContains {
            base: operand_from(arg(args, 0)?)?,
            valty: ty_from(arg(args, 1)?)?,
            key: place_from(arg(args, 2)?)?,
        },
        "mapget" => CollOp::MapGet {
            base: operand_from(arg(args, 0)?)?,
            valty: ty_from(arg(args, 1)?)?,
            key: place_from(arg(args, 2)?)?,
            span: Span { start: arg(args, 3)?.num()?, end: arg(args, 4)?.num()? },
        },
        "stringpush" => CollOp::StringPush {
            base: operand_from(arg(args, 0)?)?,
            ch: operand_from(arg(args, 1)?)?,
            span: Span { start: arg(args, 2)?.num()?, end: arg(args, 3)?.num()? },
        },
        "stringappend" => CollOp::StringAppend {
            base: operand_from(arg(args, 0)?)?,
            view: place_from(arg(args, 1)?)?,
            span: Span { start: arg(args, 2)?.num()?, end: arg(args, 3)?.num()? },
        },
        "stringasstr" => CollOp::StringAsStr { base: operand_from(arg(args, 0)?)? },
        other => return Err(format!("unknown collection op `{other}`")),
    })
}
fn stmtkind_from(sx: &Sexp) -> Result<StatementKind, String> {
    let (tag, args) = head(sx)?;
    Ok(match tag {
        "assign" => StatementKind::Assign(arg(args, 0)?.num()?, rvalue_from(arg(args, 1)?)?),
        "trace" => StatementKind::Trace(operand_from(arg(args, 0)?)?),
        "store" => StatementKind::Store(place_from(arg(args, 0)?)?, rvalue_from(arg(args, 1)?)?),
        "copyval" => StatementKind::CopyVal {
            dst: place_from(arg(args, 0)?)?,
            src: place_from(arg(args, 1)?)?,
            ty: ty_from(arg(args, 2)?)?,
        },
        "drop" => StatementKind::Drop { local: arg(args, 0)?.num()?, moved: moved_from(arg(args, 1)?)? },
        "boxop" => StatementKind::BoxOp {
            dst: place_from(arg(args, 0)?)?,
            inner_ty: ty_from(arg(args, 1)?)?,
            result_ty: ty_from(arg(args, 2)?)?,
            alloc: operand_from(arg(args, 3)?)?,
            value: place_from(arg(args, 4)?)?,
        },
        "unboxop" => StatementKind::UnboxOp {
            dst: place_from(arg(args, 0)?)?,
            inner_ty: ty_from(arg(args, 1)?)?,
            boxed: place_from(arg(args, 2)?)?,
        },
        "subslice" => StatementKind::Subslice {
            dst: place_from(arg(args, 0)?)?,
            src: place_from(arg(args, 1)?)?,
            lo: operand_from(arg(args, 2)?)?,
            hi: operand_from(arg(args, 3)?)?,
            stride: arg(args, 4)?.num()?,
            span: Span { start: arg(args, 5)?.num()?, end: arg(args, 6)?.num()? },
        },
        "strfrom" => StatementKind::StrFrom {
            dst: place_from(arg(args, 0)?)?,
            src: place_from(arg(args, 1)?)?,
        },
        "substr" => StatementKind::Substr {
            dst: place_from(arg(args, 0)?)?,
            src: place_from(arg(args, 1)?)?,
            lo: operand_from(arg(args, 2)?)?,
            hi: operand_from(arg(args, 3)?)?,
            span: Span { start: arg(args, 4)?.num()?, end: arg(args, 5)?.num()? },
        },
        "collop" => StatementKind::CollectionOp {
            dst: place_from(arg(args, 0)?)?,
            op: collop_from(arg(args, 1)?)?,
        },
        "spawn" => {
            StatementKind::Spawn { func: arg(args, 0)?.as_str()?.to_string(), args: args_from(arg(args, 1)?)? }
        }
        "scopebegin" => StatementKind::ScopeBegin,
        "scopeend" => StatementKind::ScopeEnd,
        other => return Err(format!("unknown statement kind `{other}`")),
    })
}

fn stmt_to(st: &Statement) -> Sexp {
    l(vec![a("stmt"), stmtkind_to(&st.kind), n(st.span.start), n(st.span.end), bool_atom(st.observable)])
}
fn stmt_from(sx: &Sexp) -> Result<Statement, String> {
    let args = tagged(sx, "stmt")?;
    Ok(Statement {
        kind: stmtkind_from(arg(args, 0)?)?,
        span: Span { start: arg(args, 1)?.num()?, end: arg(args, 2)?.num()? },
        observable: arg(args, 3)?.as_bool()?,
    })
}

fn term_to(t: &Terminator) -> Sexp {
    match t {
        Terminator::Goto(bid) => l(vec![a("goto"), n(*bid)]),
        Terminator::Branch { cond, then_bb, else_bb } => {
            l(vec![a("branch"), operand_to(cond), n(*then_bb), n(*else_bb)])
        }
        Terminator::Return => l(vec![a("return")]),
        Terminator::Fault(fe) => l(vec![a("faultterm"), faultedge_to(fe)]),
    }
}
fn term_from(sx: &Sexp) -> Result<Terminator, String> {
    let (tag, args) = head(sx)?;
    Ok(match tag {
        "goto" => Terminator::Goto(arg(args, 0)?.num()?),
        "branch" => Terminator::Branch {
            cond: operand_from(arg(args, 0)?)?,
            then_bb: arg(args, 1)?.num()?,
            else_bb: arg(args, 2)?.num()?,
        },
        "return" => Terminator::Return,
        "faultterm" => Terminator::Fault(faultedge_from(arg(args, 0)?)?),
        other => return Err(format!("unknown terminator `{other}`")),
    })
}

fn block_to(b: &BasicBlock) -> Sexp {
    let stmts = std::iter::once(a("stmts")).chain(b.stmts.iter().map(stmt_to)).collect();
    l(vec![a("block"), l(stmts), term_to(&b.term)])
}
fn block_from(sx: &Sexp) -> Result<BasicBlock, String> {
    let args = tagged(sx, "block")?;
    let stmts = tagged(arg(args, 0)?, "stmts")?.iter().map(stmt_from).collect::<Result<Vec<_>, _>>()?;
    Ok(BasicBlock { stmts, term: term_from(arg(args, 1)?)? })
}

// ---------------------------------------------------------------------------
// LocalDecl / Predicate / MirFn / StaticInit / MirProgram.
// ---------------------------------------------------------------------------

fn name_opt_to(name: &Option<String>) -> Sexp {
    match name {
        Some(x) => l(vec![a("some"), s(x)]),
        None => l(vec![a("none")]),
    }
}
fn name_opt_from(sx: &Sexp) -> Result<Option<String>, String> {
    let (tag, args) = head(sx)?;
    Ok(match tag {
        "some" => Some(arg(args, 0)?.as_str()?.to_string()),
        "none" => None,
        other => return Err(format!("expected some/none, got `{other}`")),
    })
}

fn local_to(d: &LocalDecl) -> Sexp {
    l(vec![a("local"), ty_to(&d.ty), name_opt_to(&d.name), bool_atom(d.drop_obligation)])
}
fn local_from(sx: &Sexp) -> Result<LocalDecl, String> {
    let args = tagged(sx, "local")?;
    Ok(LocalDecl {
        ty: ty_from(arg(args, 0)?)?,
        name: name_opt_from(arg(args, 1)?)?,
        drop_obligation: arg(args, 2)?.as_bool()?,
    })
}

fn usize_opt_to(v: &Option<usize>) -> Sexp {
    match v {
        Some(x) => l(vec![a("some"), n(*x)]),
        None => l(vec![a("none")]),
    }
}
fn usize_opt_from(sx: &Sexp) -> Result<Option<usize>, String> {
    let (tag, args) = head(sx)?;
    Ok(match tag {
        "some" => Some(arg(args, 0)?.num()?),
        "none" => None,
        other => return Err(format!("expected some/none, got `{other}`")),
    })
}

fn pred_to(p: &Predicate) -> Sexp {
    l(vec![a("pred"), n(p.entry), n(p.value), n(p.span.start), n(p.span.end), a(fault_kw(p.kind))])
}
fn pred_from(sx: &Sexp) -> Result<Predicate, String> {
    let args = tagged(sx, "pred")?;
    Ok(Predicate {
        entry: arg(args, 0)?.num()?,
        value: arg(args, 1)?.num()?,
        span: Span { start: arg(args, 2)?.num()?, end: arg(args, 3)?.num()? },
        kind: fault_from(arg(args, 4)?.as_atom()?)?,
    })
}

fn replay_kw(r: ReplayPolicy) -> &'static str {
    match r {
        ReplayPolicy::Precise => "precise",
    }
}
fn replay_from(kw: &str) -> Result<ReplayPolicy, String> {
    match kw {
        "precise" => Ok(ReplayPolicy::Precise),
        other => Err(format!("unknown replay policy `{other}`")),
    }
}

fn fn_to(f: &MirFn) -> Sexp {
    let locals = std::iter::once(a("locals")).chain(f.locals.iter().map(local_to)).collect();
    let blocks = std::iter::once(a("blocks")).chain(f.blocks.iter().map(block_to)).collect();
    let requires = std::iter::once(a("requires")).chain(f.requires.iter().map(pred_to)).collect();
    let ensures = std::iter::once(a("ensures")).chain(f.ensures.iter().map(pred_to)).collect();
    l(vec![
        a("fn"),
        s(&f.name),
        n(f.num_params),
        usize_opt_to(&f.result_local),
        l(locals),
        l(blocks),
        n(f.entry),
        l(requires),
        l(ensures),
        a(replay_kw(f.replay)),
    ])
}
fn fn_from(sx: &Sexp) -> Result<MirFn, String> {
    let args = tagged(sx, "fn")?;
    let locals = tagged(arg(args, 3)?, "locals")?.iter().map(local_from).collect::<Result<Vec<_>, _>>()?;
    let blocks = tagged(arg(args, 4)?, "blocks")?.iter().map(block_from).collect::<Result<Vec<_>, _>>()?;
    let requires =
        tagged(arg(args, 6)?, "requires")?.iter().map(pred_from).collect::<Result<Vec<_>, _>>()?;
    let ensures = tagged(arg(args, 7)?, "ensures")?.iter().map(pred_from).collect::<Result<Vec<_>, _>>()?;
    Ok(MirFn {
        name: arg(args, 0)?.as_str()?.to_string(),
        num_params: arg(args, 1)?.num()?,
        result_local: usize_opt_from(arg(args, 2)?)?,
        locals,
        blocks,
        entry: arg(args, 5)?.num()?,
        requires,
        ensures,
        replay: replay_from(arg(args, 8)?.as_atom()?)?,
    })
}

fn static_to(st: &StaticInit) -> Sexp {
    l(vec![a("static"), s(&st.name), ty_to(&st.ty), s(&st.init_fn)])
}
fn static_from(sx: &Sexp) -> Result<StaticInit, String> {
    let args = tagged(sx, "static")?;
    Ok(StaticInit {
        name: arg(args, 0)?.as_str()?.to_string(),
        ty: ty_from(arg(args, 1)?)?,
        init_fn: arg(args, 2)?.as_str()?.to_string(),
    })
}

fn program_to(prog: &MirProgram) -> Sexp {
    let fns = std::iter::once(a("fns")).chain(prog.fns.iter().map(fn_to)).collect();

    // `drop_hooks` is a HashMap; sort by key so the wire is deterministic.
    let mut hooks: Vec<(&String, &String)> = prog.drop_hooks.iter().collect();
    hooks.sort_by(|x, y| x.0.cmp(y.0));
    let hooks = std::iter::once(a("drop_hooks"))
        .chain(hooks.into_iter().map(|(k, v)| l(vec![a("hook"), s(k), s(v)])))
        .collect();

    let fn_ptrs =
        std::iter::once(a("fn_ptrs")).chain(prog.fn_ptrs.iter().map(|p| s(p))).collect();
    let statics =
        std::iter::once(a("statics")).chain(prog.statics.iter().map(static_to)).collect();

    l(vec![a("mir"), l(fns), l(hooks), l(fn_ptrs), l(statics)])
}
fn program_from(sx: &Sexp) -> Result<MirProgram, String> {
    let args = tagged(sx, "mir")?;
    let fns = tagged(arg(args, 0)?, "fns")?.iter().map(fn_from).collect::<Result<Vec<_>, _>>()?;

    let mut drop_hooks = HashMap::new();
    for h in tagged(arg(args, 1)?, "drop_hooks")? {
        let hp = tagged(h, "hook")?;
        drop_hooks.insert(arg(hp, 0)?.as_str()?.to_string(), arg(hp, 1)?.as_str()?.to_string());
    }

    let fn_ptrs = tagged(arg(args, 2)?, "fn_ptrs")?
        .iter()
        .map(|p| p.as_str().map(String::from))
        .collect::<Result<Vec<_>, _>>()?;
    let statics =
        tagged(arg(args, 3)?, "statics")?.iter().map(static_from).collect::<Result<Vec<_>, _>>()?;

    // `fn_index` is derived (name -> position in `fns`), rebuilt on load exactly
    // as the lowering builds it.
    let fn_index = fns.iter().enumerate().map(|(i, f)| (f.name.clone(), i)).collect();

    Ok(MirProgram { fns, fn_index, drop_hooks, fn_ptrs, statics })
}

// ---------------------------------------------------------------------------
// Public boundary.
// ---------------------------------------------------------------------------

/// Serialize a whole `MirProgram` to the canonical wire text. Deterministic.
pub fn serialize(prog: &MirProgram) -> String {
    let mut out = String::new();
    print_sexp(&program_to(prog), 0, &mut out);
    out.push('\n');
    out
}

/// Deserialize wire text back into a `MirProgram`. The exact inverse of
/// [`serialize`]: `serialize(deserialize(serialize(p))) == serialize(p)`.
pub fn deserialize(text: &str) -> Result<MirProgram, String> {
    program_from(&parse_sexp(text)?)
}

#[cfg(test)]
mod tests {
    use super::deserialize;

    /// A parseable-but-truncated wire (a `fn` list missing its locals/blocks/…
    /// tail) must deserialize to `Err`, not panic on an out-of-bounds arg index.
    /// Guards the arity checks that front every positional access.
    #[test]
    fn truncated_wire_errs_not_panics() {
        let wire = r#"(mir (fns (fn "main" 0)) (drop_hooks) (fn_ptrs) (statics))"#;
        let err = deserialize(wire).expect_err("truncated fn must be an error");
        assert!(err.contains("wire truncated"), "expected an arity error, got: {err}");
    }
}
