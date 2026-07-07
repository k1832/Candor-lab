//! The tree-walking evaluator (design 0001 §1.5, §4.2, §5, §6, §7, §8).
#![allow(clippy::too_many_arguments)]

use std::collections::HashMap;

use crate::ast::*;
use crate::check::dataflow::{Place, Proj};
use crate::resolve::Items;
use crate::span::Span;
use crate::token::ScalarTy;
use crate::types::{is_copy, needs_drop, ArrayLen, ItemEnv, Type};

use super::layout::Layout;
use super::mem::Mem;
use super::{Fault, FaultKind, Run};

// ---------------------------------------------------------------------------
// Control-flow signal and runtime values
// ---------------------------------------------------------------------------

enum Ctl {
    Fault(Fault),
    Return,
    Break,
    Continue,
}
type R<T> = Result<T, Ctl>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Regime {
    Checked,
    Wrapping,
    Saturating,
}

#[derive(Clone)]
enum Origin {
    Place(Place),
    Temp(usize),
    None,
}

struct RVal {
    ty: Type,
    addr: u64,
    origin: Origin,
}

/// Which sub-paths of a place are moved-out or never initialized (§1.6). A
/// stored path marks its whole subtree; the empty path marks the whole value.
#[derive(Clone, Default)]
struct MoveMask {
    moved: Vec<Vec<String>>,
}

impl MoveMask {
    fn whole() -> MoveMask {
        MoveMask {
            moved: vec![Vec::new()],
        }
    }
    fn is_moved(&self, path: &[String]) -> bool {
        self.moved.iter().any(|m| prefix(m, path))
    }
    fn partially(&self, path: &[String]) -> bool {
        self.moved
            .iter()
            .any(|m| m.len() > path.len() && m[..path.len()] == path[..])
    }
    fn mark(&mut self, path: Vec<String>) {
        self.moved.push(path);
    }
    fn set_owned(&mut self, path: &[String]) {
        self.moved.retain(|m| !(prefix(m, path) || prefix(path, m)));
    }
}

fn prefix(a: &[String], b: &[String]) -> bool {
    a.len() <= b.len() && a[..] == b[..a.len()]
}

struct Local {
    name: String,
    addr: u64,
    ty: Type,
    mask: MoveMask,
    owns: bool,
}

struct Temp {
    addr: u64,
    ty: Type,
    mask: MoveMask,
    live: bool,
}

struct Scope {
    locals: Vec<Local>,
}

struct Frame {
    #[allow(dead_code)]
    base_sp: u64,
    scopes: Vec<Scope>,
    temps: Vec<Temp>,
    regime: Regime,
    ret: Option<(Type, u64)>,
    result_addr: Option<u64>,
    ret_ty: Type,
}

fn new_frame(base_sp: u64) -> Frame {
    Frame {
        base_sp,
        scopes: vec![Scope { locals: Vec::new() }],
        temps: Vec::new(),
        regime: Regime::Checked,
        ret: None,
        result_addr: None,
        ret_ty: Type::unit(),
    }
}

// ---------------------------------------------------------------------------

pub struct Interp<'a> {
    program: &'a Program,
    items: &'a Items,
    mem: Mem,
    consts: HashMap<String, u64>,
    fns: HashMap<String, &'a FnDecl>,
    drop_hooks: HashMap<String, &'a Block>,
    fn_names: Vec<String>,
    fn_id_of: HashMap<String, u64>,
    statics: HashMap<String, (u64, Type)>,
    frames: Vec<Frame>,
    cur_span: Span,
    trace: Vec<i64>,
}

impl<'a> Interp<'a> {
    pub fn new(program: &'a Program, items: &'a Items) -> Interp<'a> {
        let mut fns = HashMap::new();
        let mut drop_hooks = HashMap::new();
        let mut fn_names = Vec::new();
        let mut fn_id_of = HashMap::new();
        let mut consts = HashMap::new();
        for item in &program.items {
            match item {
                Item::Fn(f) => {
                    fn_id_of.insert(f.name.clone(), fn_names.len() as u64);
                    fn_names.push(f.name.clone());
                    fns.insert(f.name.clone(), f);
                }
                Item::Struct(s) => {
                    if let Some(b) = &s.drop_hook {
                        drop_hooks.insert(s.name.clone(), b);
                    }
                }
                Item::Static(s) => {
                    if let ExprKind::IntLit { value, .. } = &s.value.kind {
                        consts.insert(s.name.clone(), *value);
                    }
                }
                _ => {}
            }
        }
        Interp {
            program,
            items,
            mem: Mem::new(),
            consts,
            fns,
            drop_hooks,
            fn_names,
            fn_id_of,
            statics: HashMap::new(),
            frames: Vec::new(),
            cur_span: Span::point(0),
            trace: Vec::new(),
        }
    }

    pub fn run_main(&mut self) -> Result<Run, Fault> {
        self.frames.push(new_frame(self.mem.stack_bump));
        // Pre-reserve static addresses so forward references resolve.
        let statics: Vec<(String, Type, &Expr)> = self
            .program
            .items
            .iter()
            .filter_map(|it| match it {
                Item::Static(s) => Some((
                    s.name.clone(),
                    self.items.statics[&s.name].0.clone(),
                    &s.value,
                )),
                _ => None,
            })
            .collect();
        for (name, ty, _) in &statics {
            let size = self.size_of(ty);
            let align = self.align_of(ty);
            let addr = self.mem.static_alloc(size, align);
            self.statics.insert(name.clone(), (addr, ty.clone()));
        }
        for (name, ty, expr) in &statics {
            let addr = self.statics[name].0;
            match self.eval_value(expr, Some(ty)) {
                Ok(rv) => {
                    if let Err(Ctl::Fault(f)) = self.move_to(addr, rv) {
                        return Err(f);
                    }
                }
                Err(Ctl::Fault(f)) => return Err(f),
                Err(_) => {}
            }
        }
        self.frames.pop();

        let main = match self.fns.get("main") {
            Some(f) => *f,
            None => return Err(Fault::new(FaultKind::Panic, Span::point(0), "no `main`")),
        };
        let ret_ty = self.items.fns["main"].ret.clone();
        match self.call(main, Vec::new()) {
            Ok(ret) => {
                let val = if matches!(ret_ty, Type::Scalar(ScalarTy::I64)) && ret.bytes.len() >= 8 {
                    i64::from_le_bytes(ret.bytes[..8].try_into().unwrap())
                } else {
                    0
                };
                Ok(Run {
                    ret: val,
                    trace: std::mem::take(&mut self.trace),
                })
            }
            Err(Ctl::Fault(f)) => Err(f),
            Err(_) => Err(Fault::new(FaultKind::Panic, Span::point(0), "escaped `main`")),
        }
    }

    // ---- frame accessors ----
    fn f(&mut self) -> &mut Frame {
        self.frames.last_mut().unwrap()
    }
    fn regime(&self) -> Regime {
        self.frames.last().unwrap().regime
    }
    fn push_scope(&mut self) {
        self.f().scopes.push(Scope { locals: Vec::new() });
    }
    fn add_local(&mut self, name: &str, addr: u64, ty: Type, mask: MoveMask, owns: bool) {
        self.f().scopes.last_mut().unwrap().locals.push(Local {
            name: name.to_string(),
            addr,
            ty,
            mask,
            owns,
        });
    }
    fn local_addr_ty(&self, name: &str) -> Option<(u64, Type)> {
        let fr = self.frames.last().unwrap();
        for sc in fr.scopes.iter().rev() {
            for l in sc.locals.iter().rev() {
                if l.name == name {
                    return Some((l.addr, l.ty.clone()));
                }
            }
        }
        None
    }
    fn with_local_mut<T>(&mut self, name: &str, g: impl FnOnce(&mut Local) -> T) -> Option<T> {
        let fr = self.frames.last_mut().unwrap();
        for sc in fr.scopes.iter_mut().rev() {
            for l in sc.locals.iter_mut().rev() {
                if l.name == name {
                    return Some(g(l));
                }
            }
        }
        None
    }

    // ---- layout wrappers ----
    fn lay(&self) -> Layout<'_> {
        Layout {
            items: self.items,
            consts: &self.consts,
        }
    }
    fn size_of(&self, ty: &Type) -> u64 {
        self.lay().size_of(ty)
    }
    fn align_of(&self, ty: &Type) -> u64 {
        self.lay().align_of(ty)
    }
    fn field_offset(&self, s: &str, f: &str) -> (Type, u64) {
        self.lay().field_offset(s, f).unwrap_or((Type::Error, 0))
    }

    // ---- memory helpers ----
    fn fault(&self, kind: FaultKind, msg: impl Into<String>) -> Ctl {
        Ctl::Fault(Fault::new(kind, self.cur_span, msg))
    }
    fn write_bytes(&mut self, addr: u64, data: &[u8]) -> R<()> {
        self.mem
            .write(addr, data)
            .map_err(|_| self.fault(FaultKind::BadPointer, "write beyond memory model"))
    }
    fn read_bytes(&mut self, addr: u64, len: u64, guard: bool) -> R<Vec<u8>> {
        self.mem.read(addr, len, guard).map_err(|e| match e {
            super::mem::MemErr::Oob => self.fault(FaultKind::BadPointer, "read beyond memory model"),
            super::mem::MemErr::Uninit => {
                self.fault(FaultKind::BadPointer, "read of never-written memory (init-byte guard)")
            }
        })
    }
    fn read_u64(&mut self, addr: u64) -> R<u64> {
        Ok(u64::from_le_bytes(self.read_bytes(addr, 8, true)?.try_into().unwrap()))
    }
    fn alloc_temp(&mut self, ty: Type) -> (u64, usize) {
        let size = self.size_of(&ty);
        let align = self.align_of(&ty);
        let addr = self.mem.stack_alloc(size, align);
        // Zero-fill the whole temp so struct/enum padding counts as initialized
        // (the init-byte guard must not fault on legitimate padding). Field/tag
        // writes then overwrite the meaningful bytes.
        if size > 0 {
            let _ = self.mem.write(addr, &vec![0u8; size as usize]);
        }
        let id = self.f().temps.len();
        self.f().temps.push(Temp {
            addr,
            ty,
            mask: MoveMask::default(),
            live: true,
        });
        (addr, id)
    }

    // -----------------------------------------------------------------------
    // Move / consume
    // -----------------------------------------------------------------------

    /// Move (or copy, for `copy` types) `rv` into the slot at `dst`.
    fn move_to(&mut self, dst: u64, rv: RVal) -> R<()> {
        let size = self.size_of(&rv.ty);
        if size > 0 {
            let bytes = self.read_bytes(rv.addr, size, true)?;
            self.write_bytes(dst, &bytes)?;
        }
        if !is_copy(&rv.ty, self.items) {
            self.consume(&rv.origin);
        }
        Ok(())
    }

    fn consume(&mut self, origin: &Origin) {
        match origin {
            Origin::Temp(id) => {
                if let Some(t) = self.f().temps.get_mut(*id) {
                    t.live = false;
                }
            }
            Origin::Place(p) => self.mark_place_moved(p),
            Origin::None => {}
        }
    }

    fn mark_place_moved(&mut self, p: &Place) {
        let path = if p.proj.iter().all(|x| matches!(x, Proj::Field(_))) {
            p.field_path()
        } else {
            Vec::new() // through deref/index: mark the whole root
        };
        self.with_local_mut(&p.root, |l| l.mask.mark(path));
    }

    fn set_place_owned(&mut self, p: &Place) {
        let path = p.field_path();
        self.with_local_mut(&p.root, |l| l.mask.set_owned(&path));
    }

    fn place_is_local_direct(&self, p: &Place) -> bool {
        p.proj.iter().all(|x| matches!(x, Proj::Field(_)))
            && self.local_addr_ty(&p.root).is_some()
    }
    fn place_owned(&self, p: &Place) -> bool {
        let fr = self.frames.last().unwrap();
        for sc in fr.scopes.iter().rev() {
            for l in sc.locals.iter().rev() {
                if l.name == p.root {
                    return !l.mask.is_moved(&p.field_path());
                }
            }
        }
        false
    }
}

// ===========================================================================
// Places and values
// ===========================================================================

enum CapArg {
    Val(Type, Vec<u8>),
    Out(u64),
}
struct RetVal {
    ty: Type,
    bytes: Vec<u8>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Hold {
    Owned,
    Shared,
    Excl,
}

use super::mem::round_up;

impl<'a> Interp<'a> {
    fn unit_val(&mut self) -> RVal {
        let a = self.mem.stack_alloc(0, 1);
        RVal {
            ty: Type::unit(),
            addr: a,
            origin: Origin::None,
        }
    }

    fn resolve_ty(&self, ty: &Ty) -> Type {
        match &ty.kind {
            TyKind::Scalar(s) => Type::Scalar(*s),
            TyKind::Named(n) => Type::Named(n.clone()),
            TyKind::Array { size, elem } => {
                let len = match &size.kind {
                    ExprKind::IntLit { value, .. } => ArrayLen::Lit(*value),
                    ExprKind::Ident(n) => ArrayLen::Named(n.clone()),
                    _ => ArrayLen::Unknown,
                };
                Type::Array(Box::new(self.resolve_ty(elem)), len)
            }
            TyKind::Slice(e) => Type::Slice(Box::new(self.resolve_ty(e))),
            TyKind::SliceMut(e) => Type::SliceMut(Box::new(self.resolve_ty(e))),
            TyKind::RawPtr(e) => Type::RawPtr(Box::new(self.resolve_ty(e))),
            TyKind::Box(e) => Type::Box(Box::new(self.resolve_ty(e))),
            TyKind::BoxResult(e) => Type::BoxResult(Box::new(self.resolve_ty(e))),
            TyKind::Borrow(e) => Type::Borrow(Box::new(self.resolve_ty(e))),
            TyKind::BorrowMut(e) => Type::BorrowMut(Box::new(self.resolve_ty(e))),
            TyKind::FnPtr(fp) => Type::FnPtr(crate::types::FnPtrTy {
                params: fp.params.iter().map(|p| (p.mode, self.resolve_ty(&p.ty))).collect(),
                alloc: fp.alloc,
                ret: Box::new(self.resolve_ty(&fp.ret)),
            }),
        }
    }

    fn fnptr_of(&self, name: &str) -> Type {
        let sig = &self.items.fns[name];
        Type::FnPtr(crate::types::FnPtrTy {
            params: sig.params.iter().map(|p| (p.mode, p.decl_ty.clone())).collect(),
            alloc: sig.alloc,
            ret: Box::new(sig.ret.clone()),
        })
    }

    fn peel_place(&mut self, mut addr: u64, mut ty: Type, pl: &mut Place) -> R<(u64, Type)> {
        loop {
            match ty {
                Type::Borrow(x) | Type::BorrowMut(x) | Type::Box(x) => {
                    addr = self.read_u64(addr)?;
                    pl.proj.push(Proj::Deref);
                    ty = *x;
                }
                other => return Ok((addr, other)),
            }
        }
    }

    fn eval_place(&mut self, e: &Expr) -> R<(u64, Type, Place)> {
        self.cur_span = e.span;
        match &e.kind {
            ExprKind::Paren(i) => self.eval_place(i),
            ExprKind::Ident(name) => {
                if let Some((addr, ty)) = self.local_addr_ty(name) {
                    Ok((addr, ty, Place::local(name)))
                } else if let Some((addr, ty)) = self.statics.get(name).cloned() {
                    Ok((addr, ty, Place::local(name)))
                } else {
                    Err(self.fault(FaultKind::Panic, format!("unknown place `{name}`")))
                }
            }
            ExprKind::Prefix { op: PrefixOp::Deref, expr } => {
                let (a, t, mut pl) = self.eval_place(expr)?;
                let inner = match &t {
                    Type::Borrow(x) | Type::BorrowMut(x) | Type::Box(x) => (**x).clone(),
                    _ => Type::Error,
                };
                let ptr = self.read_u64(a)?;
                pl.proj.push(Proj::Deref);
                Ok((ptr, inner, pl))
            }
            ExprKind::Field { base, field } => {
                let (a, t, pl) = self.eval_place(base)?;
                let mut pl = pl;
                let (a, t) = self.peel_place(a, t, &mut pl)?;
                match &t {
                    Type::Named(n) => {
                        let (fty, off) = self.field_offset(n, field);
                        pl.proj.push(Proj::Field(field.clone()));
                        Ok((a + off, fty, pl))
                    }
                    _ => Err(self.fault(FaultKind::Panic, "field of non-struct")),
                }
            }
            ExprKind::Index { base, index } => {
                let iv = self.eval_value(index, Some(&Type::usize()))?;
                let i = u64::from_le_bytes(self.read_bytes(iv.addr, 8, true)?[..8].try_into().unwrap());
                let (a, t, pl) = self.eval_place(base)?;
                let mut pl = pl;
                let (a, t) = self.peel_place(a, t, &mut pl)?;
                match &t {
                    Type::Array(elem, len) => {
                        let n = self.lay().array_len(len);
                        if i >= n {
                            return Err(self.fault(FaultKind::Bounds, format!("index {i} out of bounds for array of len {n}")));
                        }
                        let stride = round_up(self.size_of(elem), self.align_of(elem));
                        pl.proj.push(Proj::Index);
                        Ok((a + i * stride, (**elem).clone(), pl))
                    }
                    Type::Slice(elem) | Type::SliceMut(elem) => {
                        let ptr = self.read_u64(a)?;
                        let n = self.read_u64(a + 8)?;
                        if i >= n {
                            return Err(self.fault(FaultKind::Bounds, format!("index {i} out of bounds for slice of len {n}")));
                        }
                        let stride = round_up(self.size_of(elem), self.align_of(elem));
                        pl.proj.push(Proj::Index);
                        Ok((ptr + i * stride, (**elem).clone(), pl))
                    }
                    _ => Err(self.fault(FaultKind::Panic, "index of non-array")),
                }
            }
            _ => {
                let rv = self.eval_value(e, None)?;
                Ok((rv.addr, rv.ty, Place::local("<tmp>")))
            }
        }
    }

    fn eval_value(&mut self, e: &Expr, expected: Option<&Type>) -> R<RVal> {
        self.cur_span = e.span;
        match &e.kind {
            ExprKind::Paren(i) => self.eval_value(i, expected),
            ExprKind::OutArg(i) => self.eval_value(i, expected),
            ExprKind::Ident(name) => {
                if self.local_addr_ty(name).is_some() || self.statics.contains_key(name) {
                    let (addr, ty, pl) = self.eval_place(e)?;
                    Ok(RVal { ty, addr, origin: Origin::Place(pl) })
                } else if let Some(id) = self.fn_id_of.get(name).copied() {
                    let a = self.mem.stack_alloc(8, 8);
                    self.write_bytes(a, &id.to_le_bytes())?;
                    Ok(RVal { ty: self.fnptr_of(name), addr: a, origin: Origin::None })
                } else {
                    Err(self.fault(FaultKind::Panic, format!("unknown name `{name}`")))
                }
            }
            ExprKind::Field { .. } | ExprKind::Index { .. } | ExprKind::Prefix { op: PrefixOp::Deref, .. } => {
                let (addr, ty, pl) = self.eval_place(e)?;
                Ok(RVal { ty, addr, origin: Origin::Place(pl) })
            }
            ExprKind::IntLit { value, suffix } => {
                let sty = self.int_type(suffix, expected);
                let size = Layout::scalar_size(sty).max(1);
                let a = self.mem.stack_alloc(size, size);
                let bytes = value.to_le_bytes();
                self.write_bytes(a, &bytes[..size as usize])?;
                Ok(RVal { ty: Type::Scalar(sty), addr: a, origin: Origin::None })
            }
            ExprKind::BoolLit(b) => {
                let a = self.mem.stack_alloc(1, 1);
                self.write_bytes(a, &[*b as u8])?;
                Ok(RVal { ty: Type::bool(), addr: a, origin: Origin::None })
            }
            ExprKind::StrLit(s) => {
                let bytes = s.clone().into_bytes();
                let base = self.mem.static_alloc(bytes.len() as u64, 1);
                self.write_bytes(base, &bytes)?;
                let a = self.mem.stack_alloc(16, 8);
                self.write_bytes(a, &base.to_le_bytes())?;
                self.write_bytes(a + 8, &(bytes.len() as u64).to_le_bytes())?;
                Ok(RVal { ty: Type::Slice(Box::new(Type::Scalar(ScalarTy::U8))), addr: a, origin: Origin::None })
            }
            ExprKind::Unary { op, expr } => self.eval_unary(*op, expr, expected),
            ExprKind::Binary { op, lhs, rhs } => self.eval_binary(*op, lhs, rhs, expected, e.span),
            ExprKind::Conv { ty, expr } => self.eval_conv(ty, expr),
            ExprKind::Prefix { op: PrefixOp::Read, expr } => self.eval_borrow(expr, false),
            ExprKind::Prefix { op: PrefixOp::Write, expr } => self.eval_borrow(expr, true),
            ExprKind::Prefix { op: PrefixOp::Clone, expr } => self.eval_clone(expr),
            ExprKind::Call { callee, args } => self.eval_call(callee, args, e.span),
            ExprKind::StructLit { name, fields } => self.eval_struct_lit(name, fields),
            ExprKind::EnumCtor { enum_name, variant, args } => self.eval_enum_ctor(enum_name, variant, args, expected),
            ExprKind::ArrayLit(elems) => self.eval_array_lit(elems, expected),
            ExprKind::ArrayRepeat { value, size } => self.eval_array_repeat(value, size),
            ExprKind::CastPtr { ty, arg } => {
                let p = self.eval_value(arg, None)?;
                let addr = self.read_u64(p.addr)?;
                let a = self.mem.stack_alloc(8, 8);
                self.write_bytes(a, &addr.to_le_bytes())?;
                Ok(RVal { ty: Type::RawPtr(Box::new(self.resolve_ty(ty))), addr: a, origin: Origin::None })
            }
            ExprKind::AddrToPtr { ty, arg } => {
                let v = self.eval_value(arg, Some(&Type::usize()))?;
                let addr = self.read_u64(v.addr)?;
                let a = self.mem.stack_alloc(8, 8);
                self.write_bytes(a, &addr.to_le_bytes())?;
                Ok(RVal { ty: Type::RawPtr(Box::new(self.resolve_ty(ty))), addr: a, origin: Origin::None })
            }
            ExprKind::PtrNull { ty } => {
                let a = self.mem.stack_alloc(8, 8);
                self.write_bytes(a, &0u64.to_le_bytes())?;
                Ok(RVal { ty: Type::RawPtr(Box::new(self.resolve_ty(ty))), addr: a, origin: Origin::None })
            }
            ExprKind::Offsetof { ty, field } => {
                let n = match &self.resolve_ty(ty) {
                    Type::Named(name) => self.field_offset(name, field).1,
                    _ => 0,
                };
                self.usize_val(n)
            }
            ExprKind::Sizeof(ty) => {
                let n = self.size_of(&self.resolve_ty(ty));
                self.usize_val(n)
            }
            ExprKind::Alignof(ty) => {
                let n = self.align_of(&self.resolve_ty(ty));
                self.usize_val(n)
            }
            ExprKind::Block(b) => {
                self.exec_block(b)?;
                Ok(self.unit_val())
            }
            ExprKind::If { cond, then_blk, else_blk } => self.eval_if(cond, then_blk, else_blk.as_deref()),
            ExprKind::Match { scrutinee, arms } => self.eval_match(scrutinee, arms, expected),
            ExprKind::Loop(b) => self.eval_loop(b),
            ExprKind::While { cond, body } => self.eval_while(cond, body),
            ExprKind::Unsafe { body, .. } => {
                self.exec_block(body)?;
                Ok(self.unit_val())
            }
            ExprKind::Wrapping(b) => self.regime_block(b, Regime::Wrapping),
            ExprKind::Saturating(b) => self.regime_block(b, Regime::Saturating),
            ExprKind::Return(opt) => self.do_return(opt.as_deref()),
            ExprKind::Break => Err(Ctl::Break),
            ExprKind::Continue => Err(Ctl::Continue),
            ExprKind::Assert(c) => {
                let cv = self.eval_value(c, Some(&Type::bool()))?;
                if self.read_bytes(cv.addr, 1, true)?[0] == 0 {
                    return Err(self.fault(FaultKind::Assert, "assertion failed"));
                }
                Ok(self.unit_val())
            }
            ExprKind::Panic(msg) => {
                let m = match &msg.kind {
                    ExprKind::StrLit(s) => s.clone(),
                    _ => "panic".to_string(),
                };
                Err(self.fault(FaultKind::Panic, m))
            }
            ExprKind::Result => {
                let ra = self.frames.last().unwrap().result_addr.unwrap_or(0);
                Ok(RVal { ty: self.cur_ret_ty(), addr: ra, origin: Origin::None })
            }
        }
    }

    fn cur_ret_ty(&self) -> Type {
        self.frames.last().unwrap().ret_ty.clone()
    }

    fn usize_val(&mut self, n: u64) -> R<RVal> {
        let a = self.mem.stack_alloc(8, 8);
        self.write_bytes(a, &n.to_le_bytes())?;
        Ok(RVal { ty: Type::usize(), addr: a, origin: Origin::None })
    }

    fn int_type(&self, suffix: &Option<ScalarTy>, expected: Option<&Type>) -> ScalarTy {
        if let Some(s) = suffix {
            return *s;
        }
        if let Some(Type::Scalar(s)) = expected {
            if s.is_integer() {
                return *s;
            }
        }
        ScalarTy::I64
    }

    fn eval_borrow(&mut self, expr: &Expr, _excl: bool) -> R<RVal> {
        let (addr, inner, _pl) = self.eval_place(expr)?;
        let a = self.mem.stack_alloc(8, 8);
        self.write_bytes(a, &addr.to_le_bytes())?;
        let ty = if _excl {
            Type::BorrowMut(Box::new(inner))
        } else {
            Type::Borrow(Box::new(inner))
        };
        Ok(RVal { ty, addr: a, origin: Origin::None })
    }
}

// ===========================================================================
// Arithmetic, comparison, conversion
// ===========================================================================

fn ty_range(sty: ScalarTy) -> (i128, i128, u32, bool) {
    let (bits, signed): (u32, bool) = match sty {
        ScalarTy::I8 => (8, true),
        ScalarTy::I16 => (16, true),
        ScalarTy::I32 => (32, true),
        ScalarTy::I64 | ScalarTy::Isize => (64, true),
        ScalarTy::U8 => (8, false),
        ScalarTy::U16 => (16, false),
        ScalarTy::U32 => (32, false),
        ScalarTy::U64 | ScalarTy::Usize => (64, false),
        _ => (64, true),
    };
    let (min, max) = if signed {
        (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1)
    } else {
        (0, (1i128 << bits) - 1)
    };
    (min, max, bits, signed)
}

impl<'a> Interp<'a> {
    fn concretize(&self, ty: &Type) -> ScalarTy {
        match ty {
            Type::Scalar(s) if s.is_integer() => *s,
            Type::Scalar(ScalarTy::Bool) => ScalarTy::Bool,
            _ => ScalarTy::I64,
        }
    }

    fn read_int(&mut self, addr: u64, sty: ScalarTy) -> R<i128> {
        let size = Layout::scalar_size(sty).max(1);
        let raw = self.mem.read_uint(addr, size, true).map_err(|_| self.fault(FaultKind::BadPointer, "read"))?;
        let (_, _, bits, signed) = ty_range(sty);
        let mut v = raw as i128;
        if signed && bits < 128 && (v & (1i128 << (bits - 1))) != 0 {
            v -= 1i128 << bits;
        }
        Ok(v)
    }

    fn write_int(&mut self, addr: u64, value: i128, sty: ScalarTy) -> R<()> {
        let size = Layout::scalar_size(sty).max(1);
        let u = value as u128;
        let bytes = u.to_le_bytes();
        self.write_bytes(addr, &bytes[..size as usize])
    }

    /// Fit `value` into `sty` under the current regime, faulting on overflow.
    fn fit(&self, value: i128, sty: ScalarTy) -> R<i128> {
        let (min, max, bits, signed) = ty_range(sty);
        if value >= min && value <= max {
            return Ok(value);
        }
        match self.regime() {
            Regime::Checked => Err(Ctl::Fault(Fault::new(
                FaultKind::Overflow,
                self.cur_span,
                "arithmetic overflow",
            ))),
            Regime::Wrapping => {
                let m = 1i128 << bits;
                let mut v = value.rem_euclid(m);
                if signed && v > max {
                    v -= m;
                }
                Ok(v)
            }
            Regime::Saturating => Ok(value.clamp(min, max)),
        }
    }

    fn eval_unary(&mut self, op: UnOp, expr: &Expr, expected: Option<&Type>) -> R<RVal> {
        match op {
            UnOp::Neg => {
                let v = self.eval_value(expr, expected)?;
                let sty = self.concretize(&v.ty);
                let x = self.read_int(v.addr, sty)?;
                let r = self.fit(-x, sty)?;
                let a = self.mem.stack_alloc(Layout::scalar_size(sty).max(1), Layout::scalar_size(sty).max(1));
                self.write_int(a, r, sty)?;
                Ok(RVal { ty: Type::Scalar(sty), addr: a, origin: Origin::None })
            }
            UnOp::Not => {
                let v = self.eval_value(expr, Some(&Type::bool()))?;
                let b = self.read_bytes(v.addr, 1, true)?[0];
                let a = self.mem.stack_alloc(1, 1);
                self.write_bytes(a, &[(b == 0) as u8])?;
                Ok(RVal { ty: Type::bool(), addr: a, origin: Origin::None })
            }
        }
    }

    fn eval_binary(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr, expected: Option<&Type>, span: Span) -> R<RVal> {
        use BinOp::*;
        match op {
            And | Or => {
                let l = self.eval_value(lhs, Some(&Type::bool()))?;
                let lb = self.read_bytes(l.addr, 1, true)?[0] != 0;
                let res = if op == And {
                    if lb {
                        let r = self.eval_value(rhs, Some(&Type::bool()))?;
                        self.read_bytes(r.addr, 1, true)?[0] != 0
                    } else {
                        false
                    }
                } else if lb {
                    true
                } else {
                    let r = self.eval_value(rhs, Some(&Type::bool()))?;
                    self.read_bytes(r.addr, 1, true)?[0] != 0
                };
                let a = self.mem.stack_alloc(1, 1);
                self.write_bytes(a, &[res as u8])?;
                Ok(RVal { ty: Type::bool(), addr: a, origin: Origin::None })
            }
            Eq | Ne => {
                let l = self.eval_value(lhs, None)?;
                let ot = self.concretize(&l.ty);
                let r = self.eval_value(rhs, Some(&Type::Scalar(ot)))?;
                let equal = if ot == ScalarTy::Bool || l.ty.is_integer() {
                    self.read_int(l.addr, ot)? == self.read_int(r.addr, ot)?
                } else {
                    let sz = self.size_of(&l.ty);
                    self.read_bytes(l.addr, sz, true)? == self.read_bytes(r.addr, sz, true)?
                };
                let res = if op == Eq { equal } else { !equal };
                let a = self.mem.stack_alloc(1, 1);
                self.write_bytes(a, &[res as u8])?;
                Ok(RVal { ty: Type::bool(), addr: a, origin: Origin::None })
            }
            Lt | Le | Gt | Ge => {
                let l = self.eval_value(lhs, None)?;
                let ot = self.concretize(&l.ty);
                let r = self.eval_value(rhs, Some(&Type::Scalar(ot)))?;
                let lv = self.read_int(l.addr, ot)?;
                let rv = self.read_int(r.addr, ot)?;
                let res = match op {
                    Lt => lv < rv,
                    Le => lv <= rv,
                    Gt => lv > rv,
                    _ => lv >= rv,
                };
                let a = self.mem.stack_alloc(1, 1);
                self.write_bytes(a, &[res as u8])?;
                Ok(RVal { ty: Type::bool(), addr: a, origin: Origin::None })
            }
            Add | Sub | Mul | Div | Rem => {
                self.cur_span = span;
                let opty = expected.filter(|t| t.is_integer()).cloned();
                let l = self.eval_value(lhs, opty.as_ref())?;
                let sty = match &opty {
                    Some(Type::Scalar(s)) => *s,
                    _ => self.concretize(&l.ty),
                };
                let r = self.eval_value(rhs, Some(&Type::Scalar(sty)))?;
                let lv = self.read_int(l.addr, sty)?;
                let rv = self.read_int(r.addr, sty)?;
                self.cur_span = span;
                let raw = match op {
                    Add => lv + rv,
                    Sub => lv - rv,
                    Mul => lv * rv,
                    Div | Rem => {
                        if rv == 0 {
                            return Err(self.fault(FaultKind::DivByZero, "division by zero"));
                        }
                        if op == Div {
                            lv / rv
                        } else {
                            lv % rv
                        }
                    }
                    _ => unreachable!(),
                };
                let res = self.fit(raw, sty)?;
                let a = self.mem.stack_alloc(Layout::scalar_size(sty).max(1), Layout::scalar_size(sty).max(1));
                self.write_int(a, res, sty)?;
                Ok(RVal { ty: Type::Scalar(sty), addr: a, origin: Origin::None })
            }
        }
    }

    fn eval_conv(&mut self, ty: &Ty, expr: &Expr) -> R<RVal> {
        let src = self.eval_value(expr, None)?;
        let ssty = self.concretize(&src.ty);
        let v = self.read_int(src.addr, ssty)?;
        let tsty = match self.resolve_ty(ty) {
            Type::Scalar(s) => s,
            _ => ScalarTy::I64,
        };
        let (tmin, tmax, tbits, tsigned) = ty_range(tsty);
        let out = if v >= tmin && v <= tmax {
            v
        } else {
            match self.regime() {
                Regime::Checked => {
                    return Err(self.fault(FaultKind::ConvLoss, "conversion loses value"));
                }
                Regime::Wrapping => {
                    let m = 1i128 << tbits;
                    let mut x = v.rem_euclid(m);
                    if tsigned && x > tmax {
                        x -= m;
                    }
                    x
                }
                Regime::Saturating => v.clamp(tmin, tmax),
            }
        };
        let a = self.mem.stack_alloc(Layout::scalar_size(tsty).max(1), Layout::scalar_size(tsty).max(1));
        self.write_int(a, out, tsty)?;
        Ok(RVal { ty: Type::Scalar(tsty), addr: a, origin: Origin::None })
    }

    fn eval_clone(&mut self, expr: &Expr) -> R<RVal> {
        let src = self.eval_value(expr, None)?;
        let ty = src.ty.clone();
        let (addr, id) = self.alloc_temp(ty.clone());
        self.clone_into(addr, src.addr, &ty)?;
        Ok(RVal { ty, addr, origin: Origin::Temp(id) })
    }

    fn clone_into(&mut self, dst: u64, src: u64, ty: &Type) -> R<()> {
        if is_copy(ty, self.items) {
            let sz = self.size_of(ty);
            return self.move_bytes(dst, src, sz);
        }
        match ty {
            Type::Box(inner) => self.clone_box(dst, src, inner),
            Type::Array(elem, len) => {
                let n = self.lay().array_len(len);
                let stride = round_up(self.size_of(elem), self.align_of(elem));
                for i in 0..n {
                    self.clone_into(dst + i * stride, src + i * stride, elem)?;
                }
                Ok(())
            }
            Type::Named(n) if self.items.lookup_struct(n).is_some() => {
                let (fields, _, _) = self.lay().struct_layout(n);
                for (_, fty, off) in fields {
                    self.clone_into(dst + off, src + off, &fty)?;
                }
                Ok(())
            }
            Type::Named(_) | Type::BoxResult(_) => {
                let tag = self.read_u64(src)?;
                self.write_bytes(dst, &tag.to_le_bytes())?;
                let einfo = self.lay().enum_info(ty).unwrap();
                let payloads = einfo[tag as usize].1.clone();
                for (i, _) in payloads.iter().enumerate() {
                    let (pty, off) = self.lay().payload_offset(&payloads, i);
                    self.clone_into(dst + off, src + off, &pty)?;
                }
                Ok(())
            }
            _ => {
                let sz = self.size_of(ty);
                self.move_bytes(dst, src, sz)
            }
        }
    }

    fn clone_box(&mut self, dst: u64, src: u64, inner: &Type) -> R<()> {
        let ptr = self.read_u64(src)?;
        let ctx = self.read_u64(src + 8)?;
        let vt = self.read_u64(src + 16)?;
        let size = self.size_of(inner);
        let align = self.align_of(inner);
        let (_, alloc_off) = self.field_offset("AllocVtable", "alloc");
        let afn = self.read_u64(vt + alloc_off)?;
        let newptr = self.call_scalar(afn, vec![(Type::RawPtr(Box::new(Type::Scalar(ScalarTy::U8))), ctx), (Type::usize(), size), (Type::usize(), align)])?;
        if newptr == 0 {
            return Err(self.fault(FaultKind::Panic, "clone: allocation failed"));
        }
        self.clone_into(newptr, ptr, inner)?;
        self.write_bytes(dst, &newptr.to_le_bytes())?;
        self.write_bytes(dst + 8, &ctx.to_le_bytes())?;
        self.write_bytes(dst + 16, &vt.to_le_bytes())?;
        Ok(())
    }

    fn move_bytes(&mut self, dst: u64, src: u64, len: u64) -> R<()> {
        if len == 0 {
            return Ok(());
        }
        let b = self.read_bytes(src, len, true)?;
        self.write_bytes(dst, &b)
    }
}

// ===========================================================================
// Calls, builtins, invocation
// ===========================================================================

impl<'a> Interp<'a> {
    fn eval_call(&mut self, callee: &Expr, args: &[Expr], span: Span) -> R<RVal> {
        if let ExprKind::Ident(name) = &callee.kind {
            if let Some(rv) = self.eval_builtin(name, args, span)? {
                return Ok(rv);
            }
            if self.fns.contains_key(name.as_str()) {
                let fnd = self.fns[name.as_str()];
                let sig = self.items.fns[name.as_str()].clone();
                return self.eval_user_call(fnd, &sig, args);
            }
        }
        // indirect call through a fn-pointer value
        let cv = self.eval_value(callee, None)?;
        let id = self.read_u64(cv.addr)?;
        let name = self.fn_names[id as usize].clone();
        let fnd = self.fns[name.as_str()];
        let sig = self.items.fns[name.as_str()].clone();
        self.eval_user_call(fnd, &sig, args)
    }

    fn eval_user_call(&mut self, fnd: &'a FnDecl, sig: &crate::resolve::FnSig, args: &[Expr]) -> R<RVal> {
        let mut caps: Vec<CapArg> = Vec::new();
        let mut outs: Vec<Place> = Vec::new();
        for (p, a) in sig.params.iter().zip(args) {
            let a: &Expr = match &a.kind {
                ExprKind::OutArg(inner) => inner,
                _ => a,
            };
            match p.mode {
                ParamMode::Out => {
                    let (addr, ty, pl) = self.eval_place(a)?;
                    if self.place_is_local_direct(&pl) && self.place_owned(&pl) {
                        self.drop_value(addr, &ty, &MoveMask::default(), &mut Vec::new())?;
                    }
                    if self.place_is_local_direct(&pl) {
                        self.mark_place_moved(&pl);
                    }
                    caps.push(CapArg::Out(addr));
                    outs.push(pl);
                }
                ParamMode::Take => {
                    let rv = self.eval_value(a, Some(&p.lowered))?;
                    let sz = self.size_of(&rv.ty);
                    let bytes = self.read_bytes(rv.addr, sz, true)?;
                    if !is_copy(&rv.ty, self.items) {
                        self.consume(&rv.origin);
                    }
                    caps.push(CapArg::Val(p.lowered.clone(), bytes));
                }
                ParamMode::Read | ParamMode::Write => {
                    let rv = self.eval_value(a, Some(&p.lowered))?;
                    let bytes = self.read_bytes(rv.addr, 8, true)?;
                    caps.push(CapArg::Val(p.lowered.clone(), bytes));
                }
            }
        }
        let ret = self.call(fnd, caps)?;
        for pl in &outs {
            self.set_place_owned(pl);
        }
        self.ret_to_rval(ret)
    }

    fn ret_to_rval(&mut self, ret: RetVal) -> R<RVal> {
        if self.size_of(&ret.ty) == 0 {
            return Ok(self.unit_val());
        }
        let (addr, id) = self.alloc_temp(ret.ty.clone());
        self.write_bytes(addr, &ret.bytes)?;
        Ok(RVal { ty: ret.ty, addr, origin: Origin::Temp(id) })
    }

    fn call_scalar(&mut self, fn_id: u64, args: Vec<(Type, u64)>) -> R<u64> {
        let name = self.fn_names[fn_id as usize].clone();
        let fnd = self.fns[name.as_str()];
        let caps: Vec<CapArg> = args
            .into_iter()
            .map(|(ty, v)| {
                let sz = self.size_of(&ty) as usize;
                CapArg::Val(ty, v.to_le_bytes()[..sz.min(8)].to_vec())
            })
            .collect();
        let ret = self.call(fnd, caps)?;
        let mut buf = [0u8; 8];
        let n = ret.bytes.len().min(8);
        buf[..n].copy_from_slice(&ret.bytes[..n]);
        Ok(u64::from_le_bytes(buf))
    }

    fn call(&mut self, fnd: &'a FnDecl, caps: Vec<CapArg>) -> R<RetVal> {
        let sig = self.items.fns[fnd.name.as_str()].clone();
        let base = self.mem.stack_bump;
        let mut fr = new_frame(base);
        fr.ret_ty = sig.ret.clone();
        self.frames.push(fr);

        for (p, cap) in sig.params.iter().zip(caps) {
            match cap {
                CapArg::Val(ty, bytes) => {
                    let size = self.size_of(&ty);
                    let align = self.align_of(&ty);
                    let addr = self.mem.stack_alloc(size, align);
                    if !bytes.is_empty() {
                        self.write_bytes(addr, &bytes)?;
                    }
                    let owns = p.mode != ParamMode::Out;
                    self.add_local(&p.name, addr, ty, MoveMask::default(), owns);
                }
                CapArg::Out(addr) => {
                    self.add_local(&p.name, addr, p.lowered.clone(), MoveMask::whole(), false);
                }
            }
        }

        for r in &fnd.requires {
            let rv = self.eval_value(r, Some(&Type::bool()))?;
            if self.read_bytes(rv.addr, 1, true)?[0] == 0 {
                return Err(self.fault(FaultKind::Requires, "`requires` clause violated"));
            }
        }

        let flow = self.exec_block(&fnd.body);
        let retval = match flow {
            Ok(()) => None,
            Err(Ctl::Return) => self.f().ret.take(),
            Err(Ctl::Fault(f)) => {
                self.frames.pop();
                self.mem.stack_bump = base;
                return Err(Ctl::Fault(f));
            }
            Err(_) => None,
        };

        let ret_ty = sig.ret.clone();
        let (rty, rbytes) = match retval {
            Some((ty, addr)) => {
                let sz = self.size_of(&ty);
                (ty.clone(), self.read_bytes(addr, sz, true)?)
            }
            None => {
                let sz = self.size_of(&ret_ty) as usize;
                (ret_ty.clone(), vec![0u8; sz])
            }
        };

        if !fnd.ensures.is_empty() {
            let sz = self.size_of(&rty).max(1);
            let al = self.align_of(&rty).max(1);
            let ra = self.mem.stack_alloc(sz, al);
            if !rbytes.is_empty() {
                self.write_bytes(ra, &rbytes)?;
            }
            self.f().result_addr = Some(ra);
            for e in &fnd.ensures {
                let rv = self.eval_value(e, Some(&Type::bool()))?;
                if self.read_bytes(rv.addr, 1, true)?[0] == 0 {
                    return Err(self.fault(FaultKind::Ensures, "`ensures` clause violated"));
                }
            }
        }

        self.drop_scope()?; // parameter scope
        self.frames.pop();
        self.mem.stack_bump = base;
        Ok(RetVal { ty: rty, bytes: rbytes })
    }

    // ---- builtins ----
    fn eval_builtin(&mut self, name: &str, args: &[Expr], span: Span) -> R<Option<RVal>> {
        let rv = match name {
            "box" => self.bi_box(args)?,
            "unbox" => self.bi_unbox(args)?,
            "ptr_read" => self.bi_ptr_read(args)?,
            "ptr_write" => self.bi_ptr_write(args)?,
            "ptr_offset" => self.bi_ptr_offset(args)?,
            "is_null" => {
                let p = self.eval_value(&args[0], None)?;
                let addr = self.read_u64(p.addr)?;
                let a = self.mem.stack_alloc(1, 1);
                self.write_bytes(a, &[(addr == 0) as u8])?;
                RVal { ty: Type::bool(), addr: a, origin: Origin::None }
            }
            "ptr_to_addr" => {
                let p = self.eval_value(&args[0], None)?;
                let addr = self.read_u64(p.addr)?;
                self.usize_val(addr)?
            }
            "addr_of" | "addr_of_mut" => {
                let (addr, ty, _pl) = self.eval_place(&args[0])?;
                let a = self.mem.stack_alloc(8, 8);
                self.write_bytes(a, &addr.to_le_bytes())?;
                RVal { ty: Type::RawPtr(Box::new(ty)), addr: a, origin: Origin::None }
            }
            "slice_of" | "slice_of_mut" => self.bi_slice_of(args, name == "slice_of_mut")?,
            "subslice" => self.bi_subslice(args)?,
            "len" => self.bi_len(args)?,
            "trace" => {
                let v = self.eval_value(&args[0], Some(&Type::Scalar(ScalarTy::I64)))?;
                let n = self.read_int(v.addr, ScalarTy::I64)?;
                self.trace.push(n as i64);
                self.unit_val()
            }
            _ => return Ok(None),
        };
        let _ = span;
        Ok(Some(rv))
    }

    fn bi_box(&mut self, args: &[Expr]) -> R<RVal> {
        let av = self.eval_value(&args[0], None)?;
        // `box(a: read Alloc, ...)`: the handle may arrive owned or as a borrow.
        let alloc_addr = match &av.ty {
            Type::Borrow(_) | Type::BorrowMut(_) => self.read_u64(av.addr)?,
            _ => av.addr,
        };
        let (_, ctx_off) = self.field_offset("Alloc", "ctx");
        let (_, vt_off) = self.field_offset("Alloc", "vt");
        let ctx = self.read_u64(alloc_addr + ctx_off)?;
        let vt = self.read_u64(alloc_addr + vt_off)?;
        let vv = self.eval_value(&args[1], None)?;
        let t = vv.ty.clone();
        let size = self.size_of(&t);
        let align = self.align_of(&t);
        let (_, alloc_off) = self.field_offset("AllocVtable", "alloc");
        let afn = self.read_u64(vt + alloc_off)?;
        let ret = self.call_scalar(afn, vec![
            (Type::RawPtr(Box::new(Type::Scalar(ScalarTy::U8))), ctx),
            (Type::usize(), size),
            (Type::usize(), align),
        ])?;
        let brt = Type::BoxResult(Box::new(t.clone()));
        let (addr, id) = self.alloc_temp(brt.clone());
        if ret == 0 {
            self.drop_and_consume(vv)?;
            self.write_bytes(addr, &1u64.to_le_bytes())?; // tag: oom
        } else {
            self.move_to(ret, vv)?;
            self.write_bytes(addr, &0u64.to_le_bytes())?; // tag: boxed
            self.write_bytes(addr + 8, &ret.to_le_bytes())?;
            self.write_bytes(addr + 16, &ctx.to_le_bytes())?;
            self.write_bytes(addr + 24, &vt.to_le_bytes())?;
        }
        Ok(RVal { ty: brt, addr, origin: Origin::Temp(id) })
    }

    fn bi_unbox(&mut self, args: &[Expr]) -> R<RVal> {
        let bv = self.eval_value(&args[0], None)?;
        let inner = match &bv.ty {
            Type::Box(x) => (**x).clone(),
            _ => Type::Error,
        };
        let ptr = self.read_u64(bv.addr)?;
        let ctx = self.read_u64(bv.addr + 8)?;
        let vt = self.read_u64(bv.addr + 16)?;
        let size = self.size_of(&inner);
        let align = self.align_of(&inner);
        let (addr, id) = self.alloc_temp(inner.clone());
        self.move_bytes(addr, ptr, size)?;
        self.call_free(ctx, vt, ptr, size, align)?;
        if !is_copy(&bv.ty, self.items) {
            self.consume(&bv.origin);
        }
        Ok(RVal { ty: inner, addr, origin: Origin::Temp(id) })
    }

    fn call_free(&mut self, ctx: u64, vt: u64, ptr: u64, size: u64, align: u64) -> R<()> {
        let (_, free_off) = self.field_offset("AllocVtable", "free");
        let ffn = self.read_u64(vt + free_off)?;
        self.call_scalar(ffn, vec![
            (Type::RawPtr(Box::new(Type::Scalar(ScalarTy::U8))), ctx),
            (Type::RawPtr(Box::new(Type::Scalar(ScalarTy::U8))), ptr),
            (Type::usize(), size),
            (Type::usize(), align),
        ])?;
        Ok(())
    }

    fn bi_ptr_read(&mut self, args: &[Expr]) -> R<RVal> {
        let pv = self.eval_value(&args[0], None)?;
        let inner = match &pv.ty {
            Type::RawPtr(x) => (**x).clone(),
            _ => Type::Error,
        };
        let addr = self.read_u64(pv.addr)?;
        let size = self.size_of(&inner);
        let (a, id) = self.alloc_temp(inner.clone());
        self.move_bytes(a, addr, size)?;
        Ok(RVal { ty: inner, addr: a, origin: Origin::Temp(id) })
    }

    fn bi_ptr_write(&mut self, args: &[Expr]) -> R<RVal> {
        let pv = self.eval_value(&args[0], None)?;
        let addr = self.read_u64(pv.addr)?;
        let vv = self.eval_value(&args[1], None)?;
        let size = self.size_of(&vv.ty);
        self.move_bytes(addr, vv.addr, size)?;
        if !is_copy(&vv.ty, self.items) {
            self.consume(&vv.origin);
        }
        Ok(self.unit_val())
    }

    fn bi_ptr_offset(&mut self, args: &[Expr]) -> R<RVal> {
        let pv = self.eval_value(&args[0], None)?;
        let inner = match &pv.ty {
            Type::RawPtr(x) => (**x).clone(),
            _ => Type::Error,
        };
        let base = self.read_u64(pv.addr)?;
        let nv = self.eval_value(&args[1], Some(&Type::Scalar(ScalarTy::Isize)))?;
        let n = self.read_int(nv.addr, ScalarTy::Isize)?;
        let stride = self.size_of(&inner) as i128;
        let na = (base as i128 + n * stride) as u64;
        let a = self.mem.stack_alloc(8, 8);
        self.write_bytes(a, &na.to_le_bytes())?;
        Ok(RVal { ty: Type::RawPtr(Box::new(inner)), addr: a, origin: Origin::None })
    }

    fn bi_slice_of(&mut self, args: &[Expr], excl: bool) -> R<RVal> {
        let (addr, ty, _pl) = self.eval_place(&args[0])?;
        let (elem, n) = match &ty {
            Type::Array(e, len) => ((**e).clone(), self.lay().array_len(len)),
            Type::Slice(e) | Type::SliceMut(e) => {
                let ptr = self.read_u64(addr)?;
                let ln = self.read_u64(addr + 8)?;
                let a = self.mem.stack_alloc(16, 8);
                self.write_bytes(a, &ptr.to_le_bytes())?;
                self.write_bytes(a + 8, &ln.to_le_bytes())?;
                let t = if excl { Type::SliceMut(e.clone()) } else { Type::Slice(e.clone()) };
                return Ok(RVal { ty: t, addr: a, origin: Origin::None });
            }
            _ => (Type::Error, 0),
        };
        let a = self.mem.stack_alloc(16, 8);
        self.write_bytes(a, &addr.to_le_bytes())?;
        self.write_bytes(a + 8, &n.to_le_bytes())?;
        let t = if excl { Type::SliceMut(Box::new(elem)) } else { Type::Slice(Box::new(elem)) };
        Ok(RVal { ty: t, addr: a, origin: Origin::None })
    }

    fn bi_subslice(&mut self, args: &[Expr]) -> R<RVal> {
        let sv = self.eval_value(&args[0], None)?;
        let elem = match &sv.ty {
            Type::Slice(e) | Type::SliceMut(e) => (**e).clone(),
            _ => Type::Error,
        };
        let ptr = self.read_u64(sv.addr)?;
        let len = self.read_u64(sv.addr + 8)?;
        let lo = self.eval_value(&args[1], Some(&Type::usize()))?;
        let lo = self.read_u64(lo.addr)?;
        let hi = self.eval_value(&args[2], Some(&Type::usize()))?;
        let hi = self.read_u64(hi.addr)?;
        if lo > hi || hi > len {
            return Err(self.fault(FaultKind::Bounds, format!("subslice [{lo}..{hi}) out of bounds for len {len}")));
        }
        let stride = round_up(self.size_of(&elem), self.align_of(&elem));
        let a = self.mem.stack_alloc(16, 8);
        self.write_bytes(a, &(ptr + lo * stride).to_le_bytes())?;
        self.write_bytes(a + 8, &(hi - lo).to_le_bytes())?;
        Ok(RVal { ty: sv.ty.clone(), addr: a, origin: Origin::None })
    }

    fn bi_len(&mut self, args: &[Expr]) -> R<RVal> {
        let sv = self.eval_value(&args[0], None)?;
        let n = match &sv.ty {
            Type::Slice(_) | Type::SliceMut(_) => self.read_u64(sv.addr + 8)?,
            Type::Array(_, len) => self.lay().array_len(len),
            _ => 0,
        };
        self.usize_val(n)
    }

    fn drop_and_consume(&mut self, rv: RVal) -> R<()> {
        let mask = MoveMask::default();
        self.drop_value(rv.addr, &rv.ty, &mask, &mut Vec::new())?;
        if !is_copy(&rv.ty, self.items) {
            self.consume(&rv.origin);
        }
        Ok(())
    }
}

// ===========================================================================
// Literals, match, control flow, statements, drops
// ===========================================================================

impl<'a> Interp<'a> {
    fn eval_struct_lit(&mut self, name: &str, fields: &[FieldInit]) -> R<RVal> {
        let ty = Type::Named(name.to_string());
        let (addr, id) = self.alloc_temp(ty.clone());
        let (flayout, _, _) = self.lay().struct_layout(name);
        for (fname, fty, off) in flayout {
            if let Some(fi) = fields.iter().find(|f| f.name == fname) {
                let rv = self.eval_value(&fi.value, Some(&fty))?;
                self.move_to(addr + off, rv)?;
            }
        }
        Ok(RVal { ty, addr, origin: Origin::Temp(id) })
    }

    fn eval_enum_ctor(&mut self, enum_name: &str, variant: &str, args: &[Expr], expected: Option<&Type>) -> R<RVal> {
        if enum_name == "BoxResult" {
            let inner = match expected {
                Some(Type::BoxResult(t)) => (**t).clone(),
                _ => Type::Error,
            };
            let ty = Type::BoxResult(Box::new(inner.clone()));
            let (addr, id) = self.alloc_temp(ty.clone());
            if variant == "boxed" {
                self.write_bytes(addr, &0u64.to_le_bytes())?;
                let rv = self.eval_value(&args[0], Some(&Type::Box(Box::new(inner))))?;
                self.move_to(addr + 8, rv)?;
            } else {
                self.write_bytes(addr, &1u64.to_le_bytes())?;
            }
            return Ok(RVal { ty, addr, origin: Origin::Temp(id) });
        }
        let einfo = self
            .lay()
            .enum_info(&Type::Named(enum_name.to_string()))
            .ok_or_else(|| self.fault(FaultKind::Panic, "unknown enum"))?;
        let idx = einfo.iter().position(|(n, _)| n == variant).unwrap_or(0);
        let payloads = einfo[idx].1.clone();
        let ty = Type::Named(enum_name.to_string());
        let (addr, id) = self.alloc_temp(ty.clone());
        self.write_bytes(addr, &(idx as u64).to_le_bytes())?;
        for (i, a) in args.iter().enumerate() {
            let (pty, off) = self.lay().payload_offset(&payloads, i);
            let rv = self.eval_value(a, Some(&pty))?;
            self.move_to(addr + off, rv)?;
        }
        Ok(RVal { ty, addr, origin: Origin::Temp(id) })
    }

    fn eval_array_lit(&mut self, elems: &[Expr], expected: Option<&Type>) -> R<RVal> {
        let expected_elem = match expected {
            Some(Type::Array(e, _)) => Some((**e).clone()),
            _ => None,
        };
        let mut rvs = Vec::new();
        for el in elems {
            rvs.push(self.eval_value(el, expected_elem.as_ref())?);
        }
        let elem_ty = expected_elem
            .or_else(|| rvs.first().map(|r| r.ty.clone()))
            .unwrap_or(Type::Error);
        let n = elems.len() as u64;
        let aty = Type::Array(Box::new(elem_ty.clone()), ArrayLen::Lit(n));
        let (addr, id) = self.alloc_temp(aty.clone());
        let stride = round_up(self.size_of(&elem_ty), self.align_of(&elem_ty));
        for (i, rv) in rvs.into_iter().enumerate() {
            self.move_to(addr + i as u64 * stride, rv)?;
        }
        Ok(RVal { ty: aty, addr, origin: Origin::Temp(id) })
    }

    fn eval_array_repeat(&mut self, value: &Expr, size: &Expr) -> R<RVal> {
        let nv = self.eval_value(size, Some(&Type::usize()))?;
        let n = self.read_u64(nv.addr)?;
        let ev = self.eval_value(value, None)?;
        let elem_ty = ev.ty.clone();
        let aty = Type::Array(Box::new(elem_ty.clone()), ArrayLen::Lit(n));
        let (addr, id) = self.alloc_temp(aty.clone());
        let stride = round_up(self.size_of(&elem_ty), self.align_of(&elem_ty));
        let esize = self.size_of(&elem_ty);
        for i in 0..n {
            self.move_bytes(addr + i * stride, ev.addr, esize)?;
        }
        Ok(RVal { ty: aty, addr, origin: Origin::Temp(id) })
    }

    // ---- match ----
    fn eval_match(&mut self, scrut: &Expr, arms: &[MatchArm], expected: Option<&Type>) -> R<RVal> {
        let sv = self.eval_value(scrut, None)?;
        let (hold, enum_addr, enum_ty) = self.peel_scrutinee(sv.ty.clone(), sv.addr)?;
        let einfo = self
            .lay()
            .enum_info(&enum_ty)
            .ok_or_else(|| self.fault(FaultKind::Panic, "match on non-enum"))?;
        let tag = self.read_u64(enum_addr)? as usize;
        let (vname, payloads) = einfo[tag].clone();
        let arm = match arms.iter().find(|a| pat_matches(&a.pattern, &vname)) {
            Some(a) => a,
            None => return Err(self.fault(FaultKind::Panic, "no matching arm")),
        };
        self.push_scope();
        self.bind_pattern(&arm.pattern, hold, enum_addr, &payloads, &sv)?;
        let body_res = self.eval_value(&arm.body, expected);
        match body_res {
            Ok(rv) => {
                let out = self.materialize(rv)?;
                self.drop_scope()?;
                Ok(out)
            }
            Err(Ctl::Fault(f)) => Err(Ctl::Fault(f)),
            Err(ctl) => {
                self.drop_scope()?;
                Err(ctl)
            }
        }
    }

    fn peel_scrutinee(&mut self, ty: Type, addr: u64) -> R<(Hold, u64, Type)> {
        let mut hold = Hold::Owned;
        let mut addr = addr;
        let mut ty = ty;
        loop {
            match ty {
                Type::Borrow(x) => {
                    addr = self.read_u64(addr)?;
                    if hold == Hold::Owned {
                        hold = Hold::Shared;
                    }
                    ty = *x;
                }
                Type::BorrowMut(x) => {
                    addr = self.read_u64(addr)?;
                    if hold == Hold::Owned {
                        hold = Hold::Excl;
                    }
                    ty = *x;
                }
                Type::Box(x) => {
                    addr = self.read_u64(addr)?;
                    ty = *x;
                }
                other => return Ok((hold, addr, other)),
            }
        }
    }

    fn bind_pattern(&mut self, pat: &Pattern, hold: Hold, enum_addr: u64, payloads: &[Type], sv: &RVal) -> R<()> {
        match &pat.kind {
            PatKind::Wildcard => Ok(()),
            PatKind::Binding(_name) => {
                // whole-scrutinee binding: unused by the fixtures; bind by borrow/copy.
                Ok(())
            }
            PatKind::Variant { sub, .. } => {
                for (i, sp) in sub.iter().enumerate() {
                    let (pty, off) = self.lay().payload_offset(payloads, i);
                    let sub_addr = enum_addr + off;
                    self.bind_sub(sp, &pty, hold, sub_addr, sv, i)?;
                }
                Ok(())
            }
        }
    }

    fn bind_sub(&mut self, pat: &Pattern, pty: &Type, hold: Hold, sub_addr: u64, sv: &RVal, idx: usize) -> R<()> {
        let name = match &pat.kind {
            PatKind::Wildcard => return Ok(()),
            PatKind::Binding(n) => n.clone(),
            PatKind::Variant { .. } => {
                return Err(self.fault(FaultKind::Panic, "nested patterns unsupported in prototype interpreter"));
            }
        };
        let copy = is_copy(pty, self.items);
        match hold {
            Hold::Owned => {
                if copy {
                    let size = self.size_of(pty);
                    let a = self.mem.stack_alloc(size.max(1), self.align_of(pty).max(1));
                    self.move_bytes(a, sub_addr, size)?;
                    self.add_local(&name, a, pty.clone(), MoveMask::default(), true);
                } else {
                    // move: alias the payload sub-place, mark scrutinee moved
                    self.add_local(&name, sub_addr, pty.clone(), MoveMask::default(), true);
                    self.mark_scrutinee_moved(sv, vec![format!("_{idx}")]);
                }
            }
            Hold::Shared | Hold::Excl => {
                if copy {
                    let size = self.size_of(pty);
                    let a = self.mem.stack_alloc(size.max(1), self.align_of(pty).max(1));
                    self.move_bytes(a, sub_addr, size)?;
                    self.add_local(&name, a, pty.clone(), MoveMask::default(), true);
                } else {
                    let a = self.mem.stack_alloc(8, 8);
                    self.write_bytes(a, &sub_addr.to_le_bytes())?;
                    let bty = if hold == Hold::Excl {
                        Type::BorrowMut(Box::new(pty.clone()))
                    } else {
                        Type::Borrow(Box::new(pty.clone()))
                    };
                    self.add_local(&name, a, bty, MoveMask::default(), false);
                }
            }
        }
        Ok(())
    }

    fn mark_scrutinee_moved(&mut self, sv: &RVal, sub: Vec<String>) {
        match &sv.origin {
            Origin::Place(p) => {
                if p.proj.iter().all(|x| matches!(x, Proj::Field(_))) {
                    let mut fp = p.field_path();
                    fp.extend(sub);
                    let root = p.root.clone();
                    self.with_local_mut(&root, |l| l.mask.mark(fp));
                } else {
                    let root = p.root.clone();
                    self.with_local_mut(&root, |l| l.mask.mark(Vec::new()));
                }
            }
            Origin::Temp(id) => {
                if let Some(t) = self.f().temps.get_mut(*id) {
                    t.mask.mark(sub);
                }
            }
            Origin::None => {}
        }
    }

    fn materialize(&mut self, rv: RVal) -> R<RVal> {
        if self.size_of(&rv.ty) == 0 {
            return Ok(rv);
        }
        if is_copy(&rv.ty, self.items) {
            let size = self.size_of(&rv.ty);
            let align = self.align_of(&rv.ty);
            let a = self.mem.stack_alloc(size, align);
            self.move_bytes(a, rv.addr, size)?;
            Ok(RVal { ty: rv.ty, addr: a, origin: Origin::None })
        } else {
            let (a, id) = self.alloc_temp(rv.ty.clone());
            let ty = rv.ty.clone();
            self.move_to(a, rv)?;
            Ok(RVal { ty, addr: a, origin: Origin::Temp(id) })
        }
    }

    // ---- control flow ----
    fn eval_if(&mut self, cond: &Expr, then_blk: &Block, else_blk: Option<&Expr>) -> R<RVal> {
        let cv = self.eval_value(cond, Some(&Type::bool()))?;
        let c = self.read_bytes(cv.addr, 1, true)?[0] != 0;
        if c {
            self.exec_block(then_blk)?;
        } else if let Some(e) = else_blk {
            self.eval_value(e, None)?;
        }
        Ok(self.unit_val())
    }

    fn eval_loop(&mut self, body: &Block) -> R<RVal> {
        loop {
            match self.exec_block(body) {
                Ok(()) => {}
                Err(Ctl::Break) => break,
                Err(Ctl::Continue) => {}
                Err(other) => return Err(other),
            }
        }
        Ok(self.unit_val())
    }

    fn eval_while(&mut self, cond: &Expr, body: &Block) -> R<RVal> {
        loop {
            let cv = self.eval_value(cond, Some(&Type::bool()))?;
            if self.read_bytes(cv.addr, 1, true)?[0] == 0 {
                break;
            }
            match self.exec_block(body) {
                Ok(()) => {}
                Err(Ctl::Break) => break,
                Err(Ctl::Continue) => {}
                Err(other) => return Err(other),
            }
        }
        Ok(self.unit_val())
    }

    fn regime_block(&mut self, b: &Block, regime: Regime) -> R<RVal> {
        let save = self.f().regime;
        self.f().regime = regime;
        let r = self.exec_block(b);
        self.f().regime = save;
        r?;
        Ok(self.unit_val())
    }

    fn do_return(&mut self, opt: Option<&Expr>) -> R<RVal> {
        match opt {
            Some(e) => {
                let rty = self.cur_ret_ty();
                let rv = self.eval_value(e, Some(&rty))?;
                let size = self.size_of(&rty);
                let align = self.align_of(&rty);
                let addr = self.mem.stack_alloc(size.max(1), align.max(1));
                self.move_to(addr, rv)?;
                self.f().ret = Some((rty, addr));
            }
            None => {
                let a = self.mem.stack_alloc(0, 1);
                self.f().ret = Some((Type::unit(), a));
            }
        }
        Err(Ctl::Return)
    }

    // ---- statements ----
    fn exec_block(&mut self, b: &Block) -> R<()> {
        self.push_scope();
        let mut pending: R<()> = Ok(());
        for s in &b.stmts {
            match self.exec_stmt(s) {
                Ok(()) => {}
                Err(Ctl::Fault(f)) => return Err(Ctl::Fault(f)),
                Err(ctl) => {
                    pending = Err(ctl);
                    break;
                }
            }
        }
        self.drop_scope()?;
        pending
    }

    fn exec_stmt(&mut self, s: &Stmt) -> R<()> {
        let base = self.f().temps.len();
        let r = self.exec_stmt_inner(s);
        if let Err(Ctl::Fault(_)) = &r {
            return r;
        }
        self.drop_stmt_temps(base)?;
        r
    }

    fn exec_stmt_inner(&mut self, s: &Stmt) -> R<()> {
        match &s.kind {
            StmtKind::Let { name, ty, init, .. } => {
                let decl = ty.as_ref().map(|t| self.resolve_ty(t));
                match init {
                    Some(e) => {
                        let rv = self.eval_value(e, decl.as_ref())?;
                        let lty = decl.unwrap_or_else(|| rv.ty.clone());
                        let size = self.size_of(&lty);
                        let align = self.align_of(&lty);
                        let addr = self.mem.stack_alloc(size.max(1), align.max(1));
                        self.move_to(addr, rv)?;
                        self.add_local(name, addr, lty, MoveMask::default(), true);
                    }
                    None => {
                        let lty = decl.unwrap_or(Type::Error);
                        let size = self.size_of(&lty);
                        let align = self.align_of(&lty);
                        let addr = self.mem.stack_alloc(size.max(1), align.max(1));
                        self.add_local(name, addr, lty, MoveMask::whole(), true);
                    }
                }
                Ok(())
            }
            StmtKind::Assign { target, value } => {
                let (addr, tty, pl) = self.eval_place(target)?;
                if self.place_is_local_direct(&pl) && self.place_owned(&pl) {
                    self.drop_value(addr, &tty, &MoveMask::default(), &mut Vec::new())?;
                }
                let rv = self.eval_value(value, Some(&tty))?;
                self.move_to(addr, rv)?;
                if self.place_is_local_direct(&pl) {
                    self.set_place_owned(&pl);
                }
                Ok(())
            }
            StmtKind::Expr(e) => {
                self.eval_value(e, None)?;
                Ok(())
            }
        }
    }

    // ---- drops ----
    fn drop_scope(&mut self) -> R<()> {
        let sc = self.f().scopes.pop().unwrap();
        for l in sc.locals.into_iter().rev() {
            if l.owns {
                // §1.6 dual (finding 2026-07-07). For a *needs-drop* local the
                // checker (E0309) now guarantees this scope-exit drop decision
                // is a static, path-independent fact: the local is either
                // initialized on every path (always drop) or uninitialized/moved
                // on every path (always skip) — never conditionally initialized.
                // The interpreter therefore no longer *decides* a conditional
                // drop from a runtime flag; the mask read in `drop_value` merely
                // reflects that static fact. We assert the residual structural
                // invariant it relies on — a drop-hooked value is never reached
                // here partially moved, so its whole-value hook decision is
                // unambiguous (E0303/E0309). Drop-inert (exempt) types keep the
                // mask purely as their harmless no-op mechanism, unasserted.
                if needs_drop(&l.ty, self.items) {
                    debug_assert!(
                        !(self.ty_is_drop_hooked(&l.ty) && l.mask.partially(&[])),
                        "needs-drop value `{}` reached scope exit partially moved;                          the checker (E0303/E0309) should have forbidden this",
                        l.name
                    );
                }
                self.drop_value(l.addr, &l.ty, &l.mask, &mut Vec::new())?;
            }
        }
        Ok(())
    }

    /// Does `ty` name a struct that declares a `drop` hook (design §1.5)?
    fn ty_is_drop_hooked(&self, ty: &Type) -> bool {
        matches!(ty, Type::Named(n) if self.drop_hooks.contains_key(n))
    }

    fn drop_stmt_temps(&mut self, base: usize) -> R<()> {
        let n = self.f().temps.len();
        for i in (base..n).rev() {
            let (live, ty, addr, mask) = {
                let t = &self.f().temps[i];
                (t.live, t.ty.clone(), t.addr, t.mask.clone())
            };
            if live && !is_copy(&ty, self.items) {
                self.drop_value(addr, &ty, &mask, &mut Vec::new())?;
            }
        }
        self.f().temps.truncate(base);
        Ok(())
    }

    fn drop_value(&mut self, addr: u64, ty: &Type, mask: &MoveMask, path: &mut Vec<String>) -> R<()> {
        if mask.is_moved(path) {
            return Ok(());
        }
        match ty {
            Type::Array(elem, len) => {
                let n = self.lay().array_len(len);
                let stride = round_up(self.size_of(elem), self.align_of(elem));
                for i in (0..n).rev() {
                    self.drop_value(addr + i * stride, elem, mask, path)?;
                }
                Ok(())
            }
            Type::Box(inner) => self.drop_box(addr, inner),
            Type::Named(n) if self.items.lookup_struct(n).is_some() => {
                let partial = mask.partially(path);
                if !partial {
                    if let Some(hook) = self.drop_hooks.get(n).copied() {
                        let sname = n.clone();
                        self.run_drop_hook(hook, &sname, addr)?;
                    }
                }
                let (fields, _, _) = self.lay().struct_layout(n);
                for (fname, fty, off) in fields.into_iter().rev() {
                    path.push(fname);
                    self.drop_value(addr + off, &fty, mask, path)?;
                    path.pop();
                }
                Ok(())
            }
            Type::Named(n) if self.items.lookup_enum(n).is_some() => self.drop_enum(addr, ty, mask, path),
            Type::BoxResult(_) => self.drop_enum(addr, ty, mask, path),
            _ => Ok(()),
        }
    }

    fn drop_enum(&mut self, addr: u64, ty: &Type, mask: &MoveMask, path: &mut Vec<String>) -> R<()> {
        let tag = self.read_u64(addr)? as usize;
        let einfo = self.lay().enum_info(ty).unwrap();
        let payloads = einfo[tag].1.clone();
        for i in (0..payloads.len()).rev() {
            let (pty, off) = self.lay().payload_offset(&payloads, i);
            path.push(format!("_{i}"));
            self.drop_value(addr + off, &pty, mask, path)?;
            path.pop();
        }
        Ok(())
    }

    fn drop_box(&mut self, addr: u64, inner: &Type) -> R<()> {
        let ptr = self.read_u64(addr)?;
        let ctx = self.read_u64(addr + 8)?;
        let vt = self.read_u64(addr + 16)?;
        if ptr != 0 {
            self.drop_value(ptr, inner, &MoveMask::default(), &mut Vec::new())?;
            let size = self.size_of(inner);
            let align = self.align_of(inner);
            self.call_free(ctx, vt, ptr, size, align)?;
        }
        Ok(())
    }

    fn run_drop_hook(&mut self, hook: &'a Block, struct_name: &str, addr: u64) -> R<()> {
        let base = self.mem.stack_bump;
        self.frames.push(new_frame(base));
        let sa = self.mem.stack_alloc(8, 8);
        self.write_bytes(sa, &addr.to_le_bytes())?;
        let self_ty = Type::BorrowMut(Box::new(Type::Named(struct_name.to_string())));
        self.add_local("self", sa, self_ty, MoveMask::default(), false);
        let flow = self.exec_block(hook);
        if let Err(Ctl::Fault(f)) = flow {
            self.frames.pop();
            self.mem.stack_bump = base;
            return Err(Ctl::Fault(f));
        }
        self.drop_scope()?;
        self.frames.pop();
        self.mem.stack_bump = base;
        Ok(())
    }
}

fn pat_matches(pat: &Pattern, vname: &str) -> bool {
    match &pat.kind {
        PatKind::Wildcard | PatKind::Binding(_) => true,
        PatKind::Variant { variant, .. } => variant == vname,
    }
}
