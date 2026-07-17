//! The tree-walking evaluator (design 0001 §1.5, §4.2, §5, §6, §7, §8).
#![allow(clippy::too_many_arguments)]

use std::collections::HashMap;

use crate::ast::*;
use crate::check::dataflow::{Place, Proj};
use crate::resolve::Items;
use crate::span::Span;
use crate::token::ScalarTy;
use crate::types::{is_copy, needs_drop, scalar_name, ArrayLen, ItemEnv, Type};

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

/// (iface, iface_args, target, method -> free-fn name): one impl record.
type ImplRec = (String, Vec<Type>, String, HashMap<String, String>);

/// Resolve an impl's interface-argument AST type to a semantic type (concrete in
/// a monomorphized program).
fn resolve_impl_ty(ty: &Ty) -> Type {
    match &ty.kind {
        TyKind::Scalar(s) => Type::Scalar(*s),
        TyKind::Named(n) if n == "str" => Type::Str,
        TyKind::Named(n) => Type::Named(n.clone()),
        TyKind::Box(e) => Type::Box(Box::new(resolve_impl_ty(e))),
        TyKind::BoxResult(e) => Type::BoxResult(Box::new(resolve_impl_ty(e))),
        TyKind::App { name, args } => Type::App(name.clone(), args.iter().map(resolve_impl_ty).collect()),
        TyKind::RawPtr(e) => Type::RawPtr(Box::new(resolve_impl_ty(e))),
        TyKind::Slice(e) => Type::Slice(Box::new(resolve_impl_ty(e))),
        TyKind::SliceMut(e) => Type::SliceMut(Box::new(resolve_impl_ty(e))),
        _ => Type::Error,
    }
}

/// Strip borrow/box layers to the underlying nominal type name.
/// The interface-dispatch key for a receiver type: a nominal name or a builtin
/// scalar's spelling (`i64`), stripping borrows/box. Like `strip_to_nominal` but
/// also keys scalars, so `impl I for i64` dispatches (design 0007 §2.3).
fn dispatch_nominal(ty: &Type) -> Option<String> {
    match ty {
        Type::Scalar(s) => Some(scalar_name(*s).to_string()),
        Type::Named(n) => Some(n.clone()),
        Type::Borrow(e) | Type::BorrowMut(e) | Type::Box(e) => dispatch_nominal(e),
        _ => None,
    }
}

fn strip_to_nominal(ty: &Type) -> Option<String> {
    match ty {
        Type::Named(n) => Some(n.clone()),
        Type::Borrow(e) | Type::BorrowMut(e) | Type::Box(e) => strip_to_nominal(e),
        _ => None,
    }
}

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
    /// (target nominal, interface, method name) -> the impl method's free-function
    /// name (design 0007 static dispatch, resolved by the receiver's runtime type
    /// *and* the interface the checker resolved — two interfaces may share a method
    /// name on one target, so the interface is part of the key).
    impl_dispatch: HashMap<(String, String, String), String>,
    /// (target nominal, method name) -> the interface of the *first* impl (in item
    /// order) providing that method — the coherent default when a call site carries
    /// no resolved interface, matching the checker's first-match on a direct call.
    impl_first_iface: HashMap<(String, String), String>,
    /// Full impl records (iface, iface_args, target) -> method free-fn names, for
    /// disambiguating `From[E1]`/`From[E2]` during cross-type `?` (§7.1).
    impls_full: Vec<ImplRec>,
    /// Post-qualification name of the compiler-known `Alloc` handle struct
    /// (fields `ctx`/`vt`), identified structurally so box/unbox resolve its
    /// field offsets under the module tree's qualified names (finding F1).
    alloc_struct: Option<String>,
    /// Post-qualification name of the `AllocVtable` struct (fn-ptr fields
    /// `alloc`/`free`), the pointee of `Alloc.vt`.
    alloc_vtable_struct: Option<String>,
    frames: Vec<Frame>,
    cur_span: Span,
    trace: Vec<i64>,
    /// Content-addressed cache of string/byte literal storage: identical literal
    /// bytes share one static allocation. Literals are immutable read-only views,
    /// so deduping them is sound and — critically — bounds static growth, which
    /// otherwise leaks one allocation per evaluation and collides with the stack.
    literal_cache: HashMap<Vec<u8>, u64>,
}

impl<'a> Interp<'a> {
    pub fn new(program: &'a Program, items: &'a Items) -> Interp<'a> {
        let mut fns = HashMap::new();
        let mut drop_hooks = HashMap::new();
        let mut fn_names = Vec::new();
        let mut fn_id_of = HashMap::new();
        let mut consts = HashMap::new();
        let mut impl_dispatch = HashMap::new();
        let mut impl_first_iface: HashMap<(String, String), String> = HashMap::new();
        let mut impls_full = Vec::new();
        for item in &program.items {
            if let Item::Impl(im) = item {
                // A scalar impl target (`i64`) dispatches under its spelling, keyed
                // exactly as a nominal target is (design 0007 §2.3).
                let target = match &im.target.kind {
                    TyKind::Named(n) => n.clone(),
                    TyKind::Scalar(s) => scalar_name(*s).to_string(),
                    _ => continue,
                };
                let iface_args: Vec<Type> = im.iface_args.iter().map(resolve_impl_ty).collect();
                let mut methods = HashMap::new();
                for m in &im.methods {
                    let fnname = crate::generics::impl_method_fn_name(&im.iface, &iface_args, &target, &m.name);
                    impl_dispatch
                        .entry((target.clone(), im.iface.clone(), m.name.clone()))
                        .or_insert_with(|| fnname.clone());
                    impl_first_iface
                        .entry((target.clone(), m.name.clone()))
                        .or_insert_with(|| im.iface.clone());
                    methods.insert(m.name.clone(), fnname);
                }
                impls_full.push((im.iface.clone(), iface_args, target.clone(), methods));
            }
        }
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
        // Identify the compiler-known allocator structs structurally (never by a
        // hardcoded name), so box/unbox resolve field offsets regardless of how
        // the module tree qualifies `Alloc`/`AllocVtable` (finding F1). The
        // vtable is the struct with fn-ptr fields `alloc` and `free`; the handle
        // is the struct whose `vt` field is a `rawptr` to that vtable.
        let alloc_vtable_struct = items.structs.iter().find(|(_, s)| {
            s.fields.iter().any(|(n, t)| n == "alloc" && matches!(t, Type::FnPtr(_)))
                && s.fields.iter().any(|(n, t)| n == "free" && matches!(t, Type::FnPtr(_)))
        }).map(|(name, _)| name.clone());
        let alloc_struct = alloc_vtable_struct.as_ref().and_then(|vt| {
            items.structs.iter().find(|(_, s)| {
                s.fields.iter().any(|(n, t)| {
                    n == "vt"
                        && matches!(t, Type::RawPtr(inner)
                            if matches!(&**inner, Type::Named(x) if x == vt))
                })
            }).map(|(name, _)| name.clone())
        });
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
            impl_dispatch,
            impl_first_iface,
            impls_full,
            alloc_struct,
            alloc_vtable_struct,
            frames: Vec::new(),
            cur_span: Span::point(0),
            trace: Vec::new(),
            literal_cache: HashMap::new(),
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
                        return Err(self.attach_trace(f));
                    }
                }
                Err(Ctl::Fault(f)) => return Err(self.attach_trace(f)),
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
                // `main` reports its 64-bit return word for `i64` or `f64` (the f64
                // word is its IEEE bit pattern; design 0016).
                let is_word = matches!(
                    ret_ty,
                    Type::Scalar(ScalarTy::I64) | Type::Scalar(ScalarTy::F64)
                );
                let val = if is_word && ret.bytes.len() >= 8 {
                    i64::from_le_bytes(ret.bytes[..8].try_into().unwrap())
                } else {
                    0
                };
                Ok(Run {
                    ret: val,
                    trace: std::mem::take(&mut self.trace),
                })
            }
            Err(Ctl::Fault(f)) => Err(self.attach_trace(f)),
            Err(_) => {
                Err(self.attach_trace(Fault::new(FaultKind::Panic, Span::point(0), "escaped `main`")))
            }
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
    /// A clone of the current move-mask of the local named `name` (its live
    /// moved-out/uninit sub-paths). Used by the reassignment drop-of-old to
    /// honor the static rule that a moved-out place is not dropped (§1.5).
    fn local_mask(&self, name: &str) -> MoveMask {
        let fr = self.frames.last().unwrap();
        for sc in fr.scopes.iter().rev() {
            for l in sc.locals.iter().rev() {
                if l.name == name {
                    return l.mask.clone();
                }
            }
        }
        MoveMask::default()
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

    /// Resolved name of the `Alloc` handle struct (finding F1); falls back to
    /// the bare name for the single-file image, where nothing is qualified.
    fn alloc_struct_name(&self) -> &str {
        self.alloc_struct.as_deref().unwrap_or("Alloc")
    }
    /// Resolved name of the `AllocVtable` struct (finding F1).
    fn alloc_vtable_name(&self) -> &str {
        self.alloc_vtable_struct.as_deref().unwrap_or("AllocVtable")
    }

    // ---- memory helpers ----
    fn fault(&self, kind: FaultKind, msg: impl Into<String>) -> Ctl {
        Ctl::Fault(Fault::new(kind, self.cur_span, msg))
    }

    /// Thread the trace accumulated so far into a fault escaping to the run
    /// boundary, so the differential harness compares the pre-fault trace, not
    /// just kind+span (F-FAULT-TRACE).
    fn attach_trace(&self, mut f: Fault) -> Fault {
        f.trace = self.trace.clone();
        f
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
            // Whole-root mark for a move through `deref`/index. As of the ruling
            // of soundness review #2 (2026-07-07) the checker rejects any
            // non-copy move out of an opaque place (checker error E0310), so this
            // branch is unreachable for any checker-accepted program; the assert
            // documents that invariant (a divergence here was the double-drop
            // hole review #2 found).
            debug_assert!(
                false,
                "opaque (deref/index) move reached the interpreter: the checker must reject it as E0310"
            );
            Vec::new()
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
            TyKind::Named(n) if n == "str" => Type::Str,
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
                foreign: fp.foreign,
                ret: Box::new(self.resolve_ty(&fp.ret)),
            }),
            // Compiler-known std `Vec[T]` is kept as an application (its element
            // type drives per-element drop and stride); never lowered to a nominal.
            TyKind::App { name, args } if name == "Vec" || name == "Map" => {
                Type::App(name.clone(), args.iter().map(|a| self.resolve_ty(a)).collect())
            }
            TyKind::App { .. } | TyKind::Proj { .. } => {
                unreachable!("generic types are monomorphized before interpretation")
            }
        }
    }

    fn fnptr_of(&self, name: &str) -> Type {
        let sig = &self.items.fns[name];
        Type::FnPtr(crate::types::FnPtrTy {
            params: sig.params.iter().map(|p| (p.mode, p.decl_ty.clone())).collect(),
            alloc: sig.alloc,
            foreign: sig.foreign,
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
            ExprKind::Field { base, field, .. } => {
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
                    // `str[i]` — the byte `u8` at `i`, bounds-faulting (design 0013 §3).
                    Type::Str => {
                        let ptr = self.read_u64(a)?;
                        let n = self.read_u64(a + 8)?;
                        if i >= n {
                            return Err(self.fault(FaultKind::Bounds, format!("index {i} out of bounds for str of len {n}")));
                        }
                        pl.proj.push(Proj::Index);
                        Ok((ptr + i, Type::Scalar(ScalarTy::U8), pl))
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
            ExprKind::For { .. } => unreachable!("`for` is surface-only (formatter); the pipeline desugars it at parse (design 0009 §4.2)"),
            // Structured-concurrency Stage-2 SEQUENTIAL ORACLE (design 0012 §6):
            // a `scope` runs as a plain block, and a `spawn` runs the task to
            // completion AT the spawn point (spawn-order schedule). By SC-for-DRF
            // (§4.1) every schedule yields the same observable trace, so this
            // single-threaded execution IS the v1 oracle semantics; a task fault
            // propagates immediately, which is naturally spawn-order-first (§3.2).
            ExprKind::Scope(b) => {
                self.exec_block(b)?;
                Ok(self.unit_val())
            }
            ExprKind::Spawn(c) => {
                self.eval_value(c, None)?;
                Ok(self.unit_val())
            }
            ExprKind::Paren(i) => self.eval_value(i, expected),
            ExprKind::OutArg(i) => self.eval_value(i, expected),
            ExprKind::GenericVal { .. } => {
                unreachable!("generic values are monomorphized before interpretation")
            }
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
            ExprKind::NegIntLit { value, suffix } => {
                // The negative-literal fold: `-(value)` written into its type
                // (design 0006 §2.4). The checker has already range-checked it.
                let sty = self.int_type(suffix, expected);
                let size = Layout::scalar_size(sty).max(1);
                let a = self.mem.stack_alloc(size, size);
                self.write_int(a, -(*value as i128), sty)?;
                Ok(RVal { ty: Type::Scalar(sty), addr: a, origin: Origin::None })
            }
            ExprKind::FloatLit { bits, ty } => {
                let a = self.alloc_float(*ty, *bits)?;
                Ok(RVal { ty: Type::Scalar(*ty), addr: a, origin: Origin::None })
            }
            ExprKind::Try(inner) => self.eval_try(inner, e.span),
            ExprKind::BoolLit(b) => {
                let a = self.mem.stack_alloc(1, 1);
                self.write_bytes(a, &[*b as u8])?;
                Ok(RVal { ty: Type::bool(), addr: a, origin: Origin::None })
            }
            ExprKind::StrLit(s) => {
                // `"..."` is a `str` (design 0013): a validated UTF-8 view. A Rust
                // source string is already well-formed UTF-8, so the view is built
                // infallibly with zero runtime cost. Same fat-pointer shape as a slice.
                let a = self.str_literal(s.as_bytes())?;
                Ok(RVal { ty: Type::Str, addr: a, origin: Origin::None })
            }
            ExprKind::BytesLit(s) => {
                // `b"..."` is a raw `[u8]` view over the literal's bytes (design 0013).
                let a = self.str_literal(s.as_bytes())?;
                Ok(RVal { ty: Type::Slice(Box::new(Type::Scalar(ScalarTy::U8))), addr: a, origin: Origin::None })
            }
            ExprKind::Unary { op, expr } => self.eval_unary(*op, expr, expected),
            ExprKind::Binary { op, lhs, rhs } => self.eval_binary(*op, lhs, rhs, expected, e.span),
            ExprKind::Conv { ty, expr } => self.eval_conv(ty, expr),
            ExprKind::Bitcast { ty, expr } => self.eval_bitcast(ty, expr),
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
            ExprKind::FieldPtr { ptr, field } => {
                // `field_ptr(p, f)` = address(p) + offsetof(StructT, f) — plain
                // arithmetic, no null check (design 0004).
                let pv = self.eval_value(ptr, None)?;
                let struct_name = match &pv.ty {
                    Type::RawPtr(inner) => match &**inner {
                        Type::Named(n) => n.clone(),
                        _ => String::new(),
                    },
                    _ => String::new(),
                };
                let base = self.read_u64(pv.addr)?;
                let (fty, off) = self.field_offset(&struct_name, field);
                let na = base.wrapping_add(off);
                let a = self.mem.stack_alloc(8, 8);
                self.write_bytes(a, &na.to_le_bytes())?;
                Ok(RVal { ty: Type::RawPtr(Box::new(fty)), addr: a, origin: Origin::None })
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

    /// Materialize a byte run into static storage and return the address of a
    /// fresh `{ptr, len}` fat pointer on the stack. Shared by `str`/`[u8]` literals.
    fn str_literal(&mut self, bytes: &[u8]) -> R<u64> {
        let base = match self.literal_cache.get(bytes) {
            Some(&addr) => addr,
            None => {
                let addr = self.mem.static_alloc(bytes.len().max(1) as u64, 1);
                self.write_bytes(addr, bytes)?;
                self.literal_cache.insert(bytes.to_vec(), addr);
                addr
            }
        };
        let a = self.mem.stack_alloc(16, 8);
        self.write_bytes(a, &base.to_le_bytes())?;
        self.write_bytes(a + 8, &(bytes.len() as u64).to_le_bytes())?;
        Ok(a)
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

/// The scalar width a C-mappable (wordy) foreign type reads/writes as: pointers
/// and `usize`-shaped words are `u64`; scalars keep their own width (design 0011).
fn foreign_scalar_of(ty: &Type) -> ScalarTy {
    match ty {
        Type::Scalar(s) => *s,
        _ => ScalarTy::U64,
    }
}

/// Convert an `f64` to a target integer scalar (design 0016 §5): truncate toward
/// zero, saturating on out-of-range and mapping NaN to 0 (Rust `as` semantics).
/// Returns the sign-correct logical value for `write_int`.
fn f64_to_int(f: f64, tsty: ScalarTy) -> i128 {
    match tsty {
        ScalarTy::I8 => f as i8 as i128,
        ScalarTy::I16 => f as i16 as i128,
        ScalarTy::I32 => f as i32 as i128,
        ScalarTy::I64 | ScalarTy::Isize => f as i64 as i128,
        ScalarTy::U8 => f as u8 as i128,
        ScalarTy::U16 => f as u16 as i128,
        ScalarTy::U32 => f as u32 as i128,
        ScalarTy::U64 | ScalarTy::Usize => f as u64 as i128,
        // Non-integer targets never reach here (the checker rejects them).
        ScalarTy::Bool | ScalarTy::Unit | ScalarTy::F64 | ScalarTy::F32 => 0,
    }
}

/// Convert an `f32` to a target integer scalar (design 0016 §5): truncate toward
/// zero, saturating on out-of-range and mapping NaN to 0 (Rust `as` semantics).
fn f32_to_int(f: f32, tsty: ScalarTy) -> i128 {
    match tsty {
        ScalarTy::I8 => f as i8 as i128,
        ScalarTy::I16 => f as i16 as i128,
        ScalarTy::I32 => f as i32 as i128,
        ScalarTy::I64 | ScalarTy::Isize => f as i64 as i128,
        ScalarTy::U8 => f as u8 as i128,
        ScalarTy::U16 => f as u16 as i128,
        ScalarTy::U32 => f as u32 as i128,
        ScalarTy::U64 | ScalarTy::Usize => f as u64 as i128,
        // Non-integer targets never reach here (the checker rejects them).
        ScalarTy::Bool | ScalarTy::Unit | ScalarTy::F64 | ScalarTy::F32 => 0,
    }
}

/// IEEE ordered/`==` comparison, shared by `f32`/`f64` (design 0016 §4).
fn float_ord_cmp<T: PartialOrd>(op: BinOp, a: T, b: T) -> bool {
    match op {
        BinOp::Eq => a == b,
        BinOp::Ne => a != b,
        BinOp::Lt => a < b,
        BinOp::Le => a <= b,
        BinOp::Gt => a > b,
        BinOp::Ge => a >= b,
        _ => unreachable!("non-comparison in a float compare"),
    }
}

/// IEEE `+ - * /` over a float scalar's raw bit pattern (design 0016 §2). `l`/`r`
/// are the operand bit patterns; the result is the pattern (`f32` zero-extended).
/// Never faults; regime-exempt.
fn float_arith(op: BinOp, sty: ScalarTy, l: u64, r: u64) -> u64 {
    use BinOp::*;
    if sty == ScalarTy::F32 {
        let (a, b) = (f32::from_bits(l as u32), f32::from_bits(r as u32));
        let res = match op {
            Add => a + b,
            Sub => a - b,
            Mul => a * b,
            Div => a / b,
            _ => unreachable!("only + - * / reach a float Bin"),
        };
        res.to_bits() as u64
    } else {
        let (a, b) = (f64::from_bits(l), f64::from_bits(r));
        let res = match op {
            Add => a + b,
            Sub => a - b,
            Mul => a * b,
            Div => a / b,
            _ => unreachable!("only + - * / reach a float Bin"),
        };
        res.to_bits()
    }
}

/// IEEE comparison over a float scalar's raw bits (design 0016 §4): any NaN operand
/// yields `false`, except `!=` (which is `true`).
fn float_cmp(op: BinOp, sty: ScalarTy, l: u64, r: u64) -> bool {
    if sty == ScalarTy::F32 {
        float_ord_cmp(op, f32::from_bits(l as u32), f32::from_bits(r as u32))
    } else {
        float_ord_cmp(op, f64::from_bits(l), f64::from_bits(r))
    }
}

/// IEEE negate (sign flip) over a float scalar's raw bits (design 0016).
fn float_neg(sty: ScalarTy, bits: u64) -> u64 {
    if sty == ScalarTy::F32 {
        (-f32::from_bits(bits as u32)).to_bits() as u64
    } else {
        (-f64::from_bits(bits)).to_bits()
    }
}

/// The correctly-rounded IEEE square root of a float given by its bit pattern
/// (design 0016 §11). Total: `sqrt(negative)` is NaN, `sqrt(-0.0) == -0.0`. Rust's
/// `f32::sqrt`/`f64::sqrt` are correctly rounded, matching the native backends and
/// WASM's `f*.sqrt`.
fn float_sqrt(sty: ScalarTy, bits: u64) -> u64 {
    if sty == ScalarTy::F32 {
        f32::from_bits(bits as u32).sqrt().to_bits() as u64
    } else {
        f64::from_bits(bits).sqrt().to_bits()
    }
}

/// A numeric `conv` where the source and/or target is a float (design 0016 §5):
/// int->float rounds; `f64`->`f32` rounds (narrowing, may -> `±inf`); `f32`->`f64`
/// is exact (widening); float->int truncates toward zero, saturating (NaN -> 0).
/// `x` is the source's sign-correct register value; the result is the target's.
fn float_conv(x: i128, from: ScalarTy, to: ScalarTy) -> i128 {
    use ScalarTy::*;
    match (from, to) {
        (F32, F32) | (F64, F64) => x,
        (F32, F64) => (f32::from_bits(x as u32) as f64).to_bits() as i128,
        (F64, F32) => (f64::from_bits(x as u64) as f32).to_bits() as i128,
        (F32, t) => f32_to_int(f32::from_bits(x as u32), t),
        (F64, t) => f64_to_int(f64::from_bits(x as u64), t),
        (_, F32) => (x as f32).to_bits() as i128,
        (_, F64) => (x as f64).to_bits() as i128,
        _ => unreachable!("float_conv called with no float operand"),
    }
}

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
        // A float is carried as its bit pattern; treat as unsigned so no path
        // sign-extends it (design 0016 §6).
        ScalarTy::F64 => (64, false),
        ScalarTy::F32 => (32, false),
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
            Type::Scalar(s) => *s,
            _ => ScalarTy::I64,
        }
    }

    /// Read a float scalar's raw bit pattern from a place (`f32`: 4 bytes; `f64`:
    /// 8 bytes), zero-extended into a `u64` — the value the `float_*` helpers take.
    fn read_float_bits(&mut self, addr: u64, sty: ScalarTy) -> R<u64> {
        Ok(self.read_int(addr, sty)? as u64)
    }
    /// Allocate a float scalar slot and write `bits` (its IEEE pattern) into it.
    fn alloc_float(&mut self, sty: ScalarTy, bits: u64) -> R<u64> {
        let size = Layout::scalar_size(sty).max(1);
        let a = self.mem.stack_alloc(size, size);
        self.write_int(a, bits as i128, sty)?;
        Ok(a)
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

    /// Reduce `value` to the bit pattern of `sty` (width-exact bitwise ops and
    /// shifts, design 0006 §2.4): mask mod 2^bits, then reinterpret sign.
    fn fit_bits(&self, value: i128, sty: ScalarTy) -> i128 {
        let (_, _, bits, signed) = ty_range(sty);
        let m = 1i128 << bits;
        let mut x = value.rem_euclid(m);
        if signed && x >= (m >> 1) {
            x -= m;
        }
        x
    }

    fn eval_unary(&mut self, op: UnOp, expr: &Expr, expected: Option<&Type>) -> R<RVal> {
        match op {
            UnOp::Neg => {
                let v = self.eval_value(expr, expected)?;
                let sty = self.concretize(&v.ty);
                if sty.is_float() {
                    // IEEE negate (flips the sign bit); never faults (design 0016).
                    let bits = self.read_float_bits(v.addr, sty)?;
                    let a = self.alloc_float(sty, float_neg(sty, bits))?;
                    return Ok(RVal { ty: Type::Scalar(sty), addr: a, origin: Origin::None });
                }
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
            UnOp::BitNot => {
                let v = self.eval_value(expr, expected)?;
                let sty = self.concretize(&v.ty);
                let x = self.read_int(v.addr, sty)?;
                // Width-exact complement, re-fitted into the type's bit pattern.
                let r = self.fit_bits(!x, sty);
                let size = Layout::scalar_size(sty).max(1);
                let a = self.mem.stack_alloc(size, size);
                self.write_int(a, r, sty)?;
                Ok(RVal { ty: Type::Scalar(sty), addr: a, origin: Origin::None })
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
            Eq | Ne if matches!(self.eval_ty_probe(lhs), Some(Type::Str)) => {
                // `str` equality is byte-wise over the REFERENT run (design 0013 §3),
                // not the fat-pointer identity.
                let ord = self.str_byte_cmp(lhs, rhs)?;
                let equal = ord == std::cmp::Ordering::Equal;
                let res = if op == Eq { equal } else { !equal };
                let a = self.mem.stack_alloc(1, 1);
                self.write_bytes(a, &[res as u8])?;
                Ok(RVal { ty: Type::bool(), addr: a, origin: Origin::None })
            }
            Lt | Le | Gt | Ge if matches!(self.eval_ty_probe(lhs), Some(Type::Str)) => {
                // Byte-lexicographic ordering on `str` (design 0013 §3).
                let ord = self.str_byte_cmp(lhs, rhs)?;
                use std::cmp::Ordering::*;
                let res = match op { Lt => ord == Less, Le => ord != Greater, Gt => ord == Greater, _ => ord != Less };
                let a = self.mem.stack_alloc(1, 1);
                self.write_bytes(a, &[res as u8])?;
                Ok(RVal { ty: Type::bool(), addr: a, origin: Origin::None })
            }
            Eq | Ne => {
                let l = self.eval_value(lhs, None)?;
                let ot = self.concretize(&l.ty);
                let r = self.eval_value(rhs, Some(&Type::Scalar(ot)))?;
                let equal = if ot.is_float() {
                    // IEEE `==`: any NaN operand compares unequal (NaN == NaN is false).
                    // `equal` is the equality outcome; the outer `res` applies `!` for `!=`.
                    let (lb, rb) = (self.read_float_bits(l.addr, ot)?, self.read_float_bits(r.addr, ot)?);
                    float_cmp(BinOp::Eq, ot, lb, rb)
                } else if ot == ScalarTy::Bool || l.ty.is_integer() {
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
                let res = if ot.is_float() {
                    // IEEE ordered comparison: any NaN operand yields false.
                    let (lb, rb) = (self.read_float_bits(l.addr, ot)?, self.read_float_bits(r.addr, ot)?);
                    float_cmp(op, ot, lb, rb)
                } else {
                    let lv = self.read_int(l.addr, ot)?;
                    let rv = self.read_int(r.addr, ot)?;
                    match op {
                        Lt => lv < rv,
                        Le => lv <= rv,
                        Gt => lv > rv,
                        _ => lv >= rv,
                    }
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
                if sty.is_float() {
                    // IEEE-754 arithmetic: never faults; exempt from the regime
                    // system (design 0016 §2). `%` never reaches here (checker).
                    let r = self.eval_value(rhs, Some(&Type::Scalar(sty)))?;
                    let lb = self.read_float_bits(l.addr, sty)?;
                    let rb = self.read_float_bits(r.addr, sty)?;
                    let a = self.alloc_float(sty, float_arith(op, sty, lb, rb))?;
                    return Ok(RVal { ty: Type::Scalar(sty), addr: a, origin: Origin::None });
                }
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
            BitAnd | BitOr | BitXor => {
                let opty = expected.filter(|t| t.is_integer()).cloned();
                let l = self.eval_value(lhs, opty.as_ref())?;
                let sty = match &opty {
                    Some(Type::Scalar(s)) => *s,
                    _ => self.concretize(&l.ty),
                };
                let r = self.eval_value(rhs, Some(&Type::Scalar(sty)))?;
                let lv = self.read_int(l.addr, sty)?;
                let rv = self.read_int(r.addr, sty)?;
                let raw = match op {
                    BitAnd => lv & rv,
                    BitOr => lv | rv,
                    _ => lv ^ rv,
                };
                let res = self.fit_bits(raw, sty);
                let a = self.mem.stack_alloc(Layout::scalar_size(sty).max(1), Layout::scalar_size(sty).max(1));
                self.write_int(a, res, sty)?;
                Ok(RVal { ty: Type::Scalar(sty), addr: a, origin: Origin::None })
            }
            Shl | Shr => {
                self.cur_span = span;
                let opty = expected.filter(|t| t.is_integer()).cloned();
                let l = self.eval_value(lhs, opty.as_ref())?;
                let sty = match &opty {
                    Some(Type::Scalar(s)) => *s,
                    _ => self.concretize(&l.ty),
                };
                let r = self.eval_value(rhs, None)?;
                let rsty = self.concretize(&r.ty);
                let lv = self.read_int(l.addr, sty)?;
                let amt = self.read_int(r.addr, rsty)?;
                let bits = ty_range(sty).2 as i128;
                self.cur_span = span;
                // Shift amount >= bitwidth: fault in the default regime, mask mod
                // bitwidth under `wrapping`, clamp under `saturating` (design 0006).
                let amt = if amt < 0 {
                    return Err(self.fault(FaultKind::Overflow, "negative shift amount"));
                } else if amt >= bits {
                    match self.regime() {
                        Regime::Checked => {
                            return Err(self.fault(FaultKind::Overflow, "shift amount exceeds bit width"));
                        }
                        Regime::Wrapping => amt % bits,
                        Regime::Saturating => bits - 1,
                    }
                } else {
                    amt
                };
                let raw = match op {
                    Shl => lv << amt,
                    _ => lv >> amt, // arithmetic for signed, logical for unsigned (i128 read is sign-correct)
                };
                let res = self.fit_bits(raw, sty);
                let a = self.mem.stack_alloc(Layout::scalar_size(sty).max(1), Layout::scalar_size(sty).max(1));
                self.write_int(a, res, sty)?;
                Ok(RVal { ty: Type::Scalar(sty), addr: a, origin: Origin::None })
            }
        }
    }

    fn eval_conv(&mut self, ty: &Ty, expr: &Expr) -> R<RVal> {
        let src = self.eval_value(expr, None)?;
        let ssty = self.concretize(&src.ty);
        let tsty = match self.resolve_ty(ty) {
            Type::Scalar(s) => s,
            _ => ScalarTy::I64,
        };
        // Numeric conversions involving a float are IEEE and regime-exempt
        // (design 0016 §5): int->float rounds; f64->f32 rounds (narrowing);
        // f32->f64 is exact; float->int truncates toward zero, saturating (NaN->0).
        if tsty.is_float() || ssty.is_float() {
            let x = self.read_int(src.addr, ssty)?;
            let out = float_conv(x, ssty, tsty);
            let a = self.mem.stack_alloc(Layout::scalar_size(tsty).max(1), Layout::scalar_size(tsty).max(1));
            self.write_int(a, out, tsty)?;
            return Ok(RVal { ty: Type::Scalar(tsty), addr: a, origin: Origin::None });
        }
        let v = self.read_int(src.addr, ssty)?;
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

    /// Evaluate `bitcast T (e)` -- same-width bit reinterpretation (design 0016
    /// section 10). Reads the operand's raw bit pattern and re-fits it into `T`'s
    /// width/signedness; the bits are identical (same width, checker-guaranteed), so
    /// no value changes. Total -- never faults, regime-independent.
    fn eval_bitcast(&mut self, ty: &Ty, expr: &Expr) -> R<RVal> {
        let tsty = match self.resolve_ty(ty) {
            Type::Scalar(s) => s,
            _ => ScalarTy::I64,
        };
        // A bare `{integer}` operand takes the float's same-width unsigned int so its
        // full pattern survives (mirrors the checker / MIR lowering).
        let expected = if tsty.is_float() {
            Some(Type::Scalar(if tsty == ScalarTy::F64 { ScalarTy::U64 } else { ScalarTy::U32 }))
        } else {
            None
        };
        let src = self.eval_value(expr, expected.as_ref())?;
        let ssty = self.concretize(&src.ty);
        let raw = self.read_int(src.addr, ssty)?;
        let out = self.fit_bits(raw, tsty);
        let size = Layout::scalar_size(tsty).max(1);
        let a = self.mem.stack_alloc(size, size);
        self.write_int(a, out, tsty)?;
        Ok(RVal { ty: Type::Scalar(tsty), addr: a, origin: Origin::None })
    }

    /// Evaluate `expr?` (spec 02 §6.5): unwrap the `ok` variant's payload, or
    /// early-return the whole value from the enclosing function.
    fn eval_try(&mut self, inner: &Expr, span: Span) -> R<RVal> {
        self.cur_span = span;
        let v = self.eval_value(inner, None)?;
        let enum_addr = v.addr;
        let ok_name = match &v.ty {
            Type::Named(n) => self.items.enums.get(n).and_then(|e| e.ok_variant.clone()),
            Type::BoxResult(_) => Some("boxed".to_string()),
            _ => None,
        };
        let ok_name = match ok_name {
            Some(n) => n,
            None => return Err(self.fault(FaultKind::Panic, "`?` on a non-result-shaped enum")),
        };
        let einfo = self
            .lay()
            .enum_info(&v.ty)
            .ok_or_else(|| self.fault(FaultKind::Panic, "`?` on a non-enum"))?;
        let ok_idx = einfo
            .iter()
            .position(|(n, _)| n == &ok_name)
            .ok_or_else(|| self.fault(FaultKind::Panic, "`?`: missing ok variant"))?;
        let tag = self.read_u64(enum_addr)? as usize;
        if tag == ok_idx {
            let payloads = einfo[ok_idx].1.clone();
            if payloads.is_empty() {
                if !is_copy(&v.ty, self.items) {
                    self.consume(&v.origin);
                }
                let a = self.mem.stack_alloc(0, 1);
                return Ok(RVal { ty: Type::unit(), addr: a, origin: Origin::None });
            }
            let (pty, off) = self.lay().payload_offset(&payloads, 0);
            let (taddr, tid) = self.alloc_temp(pty.clone());
            let size = self.size_of(&pty);
            self.move_bytes(taddr, enum_addr + off, size)?;
            // Mark the source enum consumed so its now-moved payload is not
            // double-dropped; the payload lives on in the fresh temp.
            self.consume(&v.origin);
            Ok(RVal { ty: pty, addr: taddr, origin: Origin::Temp(tid) })
        } else {
            let rty = self.cur_ret_ty();
            let cross = match (&v.ty, &rty) {
                (Type::Named(a), Type::Named(b)) => a != b,
                _ => false,
            };
            if cross {
                return self.eval_try_from(enum_addr, &v, &einfo, &ok_name, tag, &rty);
            }
            let size = self.size_of(&rty);
            let align = self.align_of(&rty);
            let addr = self.mem.stack_alloc(size.max(1), align.max(1));
            self.move_to(addr, v)?;
            self.f().ret = Some((rty, addr));
            Err(Ctl::Return)
        }
    }

    /// Cross-type `?` (design 0007 §7.1): extract the operand's non-`ok` payload
    /// `e1`, convert it through the matching `From` impl to `e2`, wrap `e2` in the
    /// return enum's non-`ok` variant, and early-return it.
    fn eval_try_from(
        &mut self,
        enum_addr: u64,
        v: &RVal,
        op_info: &[(String, Vec<Type>)],
        _op_ok: &str,
        tag: usize,
        rty: &Type,
    ) -> R<RVal> {
        // The operand's actual (non-`ok`) variant payload = e1.
        let (e1ty, e1off) = self.lay().payload_offset(&op_info[tag].1, 0);
        let e1bytes = self.read_bytes(enum_addr + e1off, self.size_of(&e1ty), true)?;
        // The return enum's non-`ok` variant and its e2 payload type.
        let ret_info = self
            .lay()
            .enum_info(rty)
            .ok_or_else(|| self.fault(FaultKind::Panic, "`?`: return is not an enum"))?;
        let ret_ok = match rty {
            Type::Named(n) => self.items.enums.get(n).and_then(|e| e.ok_variant.clone()),
            _ => None,
        };
        let (ret_nonok_idx, e2ty) = ret_info
            .iter()
            .enumerate()
            .find(|(_, (n, _))| Some(n.as_str()) != ret_ok.as_deref())
            .map(|(i, (_, p))| (i, self.lay().payload_offset(p, 0).0))
            .ok_or_else(|| self.fault(FaultKind::Panic, "`?`: no error variant"))?;
        // Resolve the `From[e1] for e2` impl method.
        let e2nom = match &e2ty {
            Type::Named(n) => n.clone(),
            _ => return Err(self.fault(FaultKind::Panic, "`?`: error payload is not nominal")),
        };
        let fnname = self
            .impls_full
            .iter()
            .find(|(iface, args, target, _)| crate::generics::base_name(iface) == "From" && target == &e2nom && args.first() == Some(&e1ty))
            .and_then(|(_, _, _, methods)| methods.get("from").cloned())
            .ok_or_else(|| self.fault(FaultKind::Panic, "`?`: no matching `From` impl"))?;
        // Consume the operand (its payload moves into the conversion).
        self.consume(&v.origin);
        // Call `from(e1)` -> e2.
        let fnd = self.fns[fnname.as_str()];
        let e2ret = self.call(fnd, vec![CapArg::Val(e1ty, e1bytes)])?;
        // Build the return enum value: tag = ret_nonok_idx, payload = e2 at offset.
        let size = self.size_of(rty);
        let align = self.align_of(rty);
        let addr = self.mem.stack_alloc(size.max(1), align.max(1));
        self.write_bytes(addr, &(ret_nonok_idx as u64).to_le_bytes())?;
        let (_, e2off) = self.lay().payload_offset(&ret_info[ret_nonok_idx].1, 0);
        self.write_bytes(addr + e2off, &e2ret.bytes)?;
        self.f().ret = Some((rty.clone(), addr));
        Err(Ctl::Return)
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
        let (_, alloc_off) = self.field_offset(self.alloc_vtable_name(), "alloc");
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
    /// The receiver's static nominal type name (stripping borrows/box), for
    /// interface method dispatch. Handles the place shapes a receiver can take.
    fn expr_static_nominal(&self, e: &Expr) -> Option<String> {
        dispatch_nominal(&self.expr_static_ty(e)?)
    }

    /// Resolve an interface method call to its impl free-function name (design
    /// 0007 static dispatch). `iface` is the interface the checker resolved,
    /// carried on the call node by monomorphization; when absent (a non-generic
    /// program, which mono does not rewrite) the first impl providing the method
    /// for `nominal` is used — the same coherent first-match the checker made.
    fn dispatch_method(&self, nominal: &str, field: &str, iface: Option<&String>) -> Option<&String> {
        let key_iface = match iface {
            Some(i) => i.clone(),
            None => self
                .impl_first_iface
                .get(&(nominal.to_string(), field.to_string()))?
                .clone(),
        };
        self.impl_dispatch
            .get(&(nominal.to_string(), key_iface, field.to_string()))
    }

    /// The static type of a place expression (local, field, dereference), for
    /// interface method dispatch on a nested receiver like `self.inner.m()`.
    fn expr_static_ty(&self, e: &Expr) -> Option<Type> {
        match &e.kind {
            ExprKind::Paren(i) | ExprKind::OutArg(i) => self.expr_static_ty(i),
            ExprKind::Ident(name) => self.local_addr_ty(name).map(|(_, t)| t),
            ExprKind::Field { base, field, .. } => {
                let bt = self.expr_static_ty(base)?;
                let n = strip_to_nominal(&bt)?;
                self.lay().field_offset(&n, field).map(|(t, _)| t)
            }
            ExprKind::Prefix { op: PrefixOp::Deref, expr } => match self.expr_static_ty(expr)? {
                Type::Box(e) | Type::Borrow(e) | Type::BorrowMut(e) | Type::RawPtr(e) => Some(*e),
                _ => None,
            },
            // The static type of a call is the callee's declared return type, so a
            // method call on a call-shaped receiver (`f(x).m()`, `f(x).*.m()`)
            // resolves. Post-monomorphization the callee's signature is already
            // concrete, so no generic substitution is needed here.
            ExprKind::Call { callee, args } => self.call_static_ret(callee, args),
            _ => None,
        }
    }

    /// The return type of a statically-resolvable call: a builtin collection/
    /// text op, a free function, a nominal-dispatched interface method, or an
    /// `extern` shim. An indirect call through a fn-pointer value has no
    /// statically-known callee here, so it yields `None` (unchanged from before).
    fn call_static_ret(&self, callee: &Expr, args: &[Expr]) -> Option<Type> {
        match &callee.kind {
            ExprKind::Ident(name) => {
                // Builtins shadow same-named user fns exactly as `eval_builtin`
                // dispatches them (builtins tried first, gated by the arg0
                // collection guards), so a chain on `get(v,i).*` / `as_str(..)`
                // resolves to the builtin's result type.
                if let Some(t) = self.builtin_static_ret(name, args) {
                    return Some(t);
                }
                if let Some(sig) = self.items.fns.get(name.as_str()) {
                    return Some(sig.ret.clone());
                }
                self.items.externs.get(name.as_str()).map(|es| es.to_fn_sig().ret)
            }
            ExprKind::Field { base, field, iface } => {
                let nominal = self.expr_static_nominal(base)?;
                let fnname = self.dispatch_method(&nominal, field, iface.as_ref())?;
                self.items.fns.get(fnname.as_str()).map(|sig| sig.ret.clone())
            }
            _ => None,
        }
    }

    /// The result type of a builtin collection/text op call, mirroring the
    /// checker's `check_builtin` result types (src/check/expr.rs) so a method/
    /// field/deref chain on a builtin-call result resolves and dispatches the
    /// same way it would on a user call. Returns `None` for a name/arg-shape
    /// that is not a builtin here (so a same-named user fn still resolves) or
    /// whose result type is fixed by the annotation rather than the arguments
    /// (`vec_new`/`map_new`) and is never chained. The collection ops
    /// (`get`/`push`/`pop`/`set`/`insert`/`contains`/`as_str`) carry the same
    /// arg0 guards `eval_builtin` uses, so an unrelated user `push`/`get` on a
    /// non-collection is left alone.
    fn builtin_static_ret(&self, name: &str, args: &[Expr]) -> Option<Type> {
        let ty = match name {
            "unbox" => match self.expr_static_ty(args.first()?)? {
                Type::Box(inner) => *inner,
                _ => return None,
            },
            "ptr_read" => match self.expr_static_ty(args.first()?)? {
                Type::RawPtr(inner) => *inner,
                _ => return None,
            },
            "ptr_write" => Type::unit(),
            "ptr_offset" => self.expr_static_ty(args.first()?)?,
            "is_null" => Type::bool(),
            "ptr_to_addr" => Type::usize(),
            // `sqrt(x)` returns the argument's float type (design 0016 §11), so a
            // `trace(sqrt(x))` observes it at the right width.
            "sqrt" => self.expr_static_ty(args.first()?)?,
            "addr_of" | "addr_of_mut" => {
                Type::RawPtr(Box::new(self.expr_static_ty(args.first()?)?))
            }
            "subslice" => match self.expr_static_ty(args.first()?)? {
                t @ (Type::Slice(_) | Type::SliceMut(_)) => t,
                _ => return None,
            },
            "as_bytes" => Type::Slice(Box::new(Type::Scalar(ScalarTy::U8))),
            "str_from" => Type::Named("Utf8Res".to_string()),
            "str_from_unchecked" | "substr" => Type::Str,
            "char_at" => Type::Named("CharStep".to_string()),
            "char_count" | "len" => Type::usize(),
            "string_new" => Type::Named("String".to_string()),
            "push" | "append" if self.arg0_is_string(args) => Type::unit(),
            "as_str" if self.arg0_is_string(args) => Type::Str,
            "push" | "set" if self.arg0_is_vec(args) => Type::unit(),
            "pop" if self.arg0_is_vec(args) => Type::Named("Opt".to_string()),
            "get" if self.arg0_is_vec(args) => {
                Type::Borrow(Box::new(self.vec_elem_of(args.first()?)))
            }
            "insert" if self.arg0_is_map(args) => Type::unit(),
            "contains" if self.arg0_is_map(args) => Type::bool(),
            "get" if self.arg0_is_map(args) => {
                Type::Borrow(Box::new(self.map_valty_of(args.first()?)))
            }
            "cancelled" => Type::bool(),
            "trace" => Type::unit(),
            _ => return None,
        };
        Some(ty)
    }

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
            // Foreign (`extern`) call (design 0011 §5): dispatch through the shim
            // registry, or raise `no_foreign_runtime` if unregistered.
            if let Some(es) = self.items.externs.get(name.as_str()).cloned() {
                return self.eval_extern_call(&es, args, span);
            }
        }
        // Byte iteration `s.at(i)` over `str`/`[u8]` (design 0013 §3): the wired
        // ground-floor `Indexed` method yielding `Opt[u8]`. Handled before impl
        // dispatch (str/[u8] host no user impl).
        if let ExprKind::Field { base, field, .. } = &callee.kind {
            if field == "at" {
                let bt = self.expr_static_ty(base).map(|t| match t {
                    Type::Borrow(e) | Type::BorrowMut(e) => *e,
                    other => other,
                });
                let is_byteview = matches!(&bt, Some(Type::Str))
                    || matches!(&bt, Some(Type::Slice(e)) if matches!(**e, Type::Scalar(ScalarTy::U8)));
                if is_byteview {
                    let (ptr, len) = self.read_byteview(base)?;
                    let iv = self.eval_value(&args[0], Some(&Type::usize()))?;
                    let i = self.read_u64(iv.addr)?;
                    if i < len {
                        let byte = self.read_bytes(ptr + i, 1, true)?[0];
                        let tmp = self.mem.stack_alloc(1, 1);
                        self.write_bytes(tmp, &[byte])?;
                        return self.build_named_enum("Opt", "Some", &[(Type::Scalar(ScalarTy::U8), tmp)]);
                    } else {
                        return self.build_named_enum("Opt", "None", &[]);
                    }
                }
                // `Vec[T]` indexed access: `at(read self, i) -> Opt[T]` — copies the
                // element into `Some`, or yields `None` past the end (0009 Indexed).
                let vt = self.expr_static_ty(base).map(|t| match t {
                    Type::Borrow(e) | Type::BorrowMut(e) => *e,
                    other => other,
                });
                if let Some(Type::App(n, targs)) = &vt {
                    if n == "Vec" {
                        let elem = targs.first().cloned().unwrap_or(Type::Error);
                        let vbase = self.vec_base(base)?;
                        let iv = self.eval_value(&args[0], Some(&Type::usize()))?;
                        let i = self.read_u64(iv.addr)?;
                        let len = self.read_u64(vbase + 8)?;
                        if i < len {
                            let buf = self.read_u64(vbase)?;
                            let stride = round_up(self.size_of(&elem), self.align_of(&elem));
                            return self.build_named_enum("Opt", "Some", &[(elem, buf + i * stride)]);
                        } else {
                            return self.build_named_enum("Opt", "None", &[]);
                        }
                    }
                }
            }
            // The region-free BORROWED-yield protocol `RefIndexed` (OBL-ITER-BORROW):
            // `count(read self) -> usize` and `get_ref(read self, i) -> read Item`.
            // A `Vec[T]` receiver answers both, wiring `for read x in read v`.
            // `get_ref` returns a *borrow value* (an 8-byte slot holding the address
            // of element `i` in the buffer) — no copy, so it works over non-`copy`
            // elements.
            if field == "count" || field == "get_ref" {
                let vt = self.expr_static_ty(base).map(|t| match t {
                    Type::Borrow(e) | Type::BorrowMut(e) => *e,
                    other => other,
                });
                if let Some(Type::App(n, targs)) = &vt {
                    if n == "Vec" {
                        let vbase = self.vec_base(base)?;
                        let len = self.read_u64(vbase + 8)?;
                        if field == "count" {
                            return self.usize_val(len);
                        }
                        // get_ref: reborrow of element `i`.
                        let elem = targs.first().cloned().unwrap_or(Type::Error);
                        let iv = self.eval_value(&args[0], Some(&Type::usize()))?;
                        let i = self.read_u64(iv.addr)?;
                        if i >= len {
                            return Err(self.fault(FaultKind::Bounds, format!("Vec index {i} out of bounds (len {len})")));
                        }
                        let stride = round_up(self.size_of(&elem), self.align_of(&elem));
                        let buf = self.read_u64(vbase)?;
                        let a = self.mem.stack_alloc(8, 8);
                        self.write_bytes(a, &(buf + i * stride).to_le_bytes())?;
                        return Ok(RVal { ty: Type::Borrow(Box::new(elem)), addr: a, origin: Origin::None });
                    }
                }
            }
        }
        // Interface method call `recv.m(args)` (design 0007 static dispatch): the
        // impl is chosen by the receiver's runtime nominal type.
        if let ExprKind::Field { base, field, iface } = &callee.kind {
            if let Some(nominal) = self.expr_static_nominal(base) {
                if let Some(fnname) = self.dispatch_method(&nominal, field, iface.as_ref()).cloned() {
                    let fnd = self.fns[fnname.as_str()];
                    let sig = self.items.fns[fnname.as_str()].clone();
                    // Pass the receiver per the method's `self` mode: `read`/`write`
                    // self is borrowed (the checker treated it so, §4.1); `take`
                    // self moves the value in.
                    // Already-borrowed receivers (a `read T`/`write T` parameter)
                    // are passed through (a reborrow); an owned receiver place is
                    // borrowed per the method's `self` mode.
                    let base_is_borrow = matches!(
                        self.expr_static_ty(base),
                        Some(Type::Borrow(_)) | Some(Type::BorrowMut(_))
                    );
                    let recv = match sig.params.first().map(|p| p.mode) {
                        Some(ParamMode::Read) if !base_is_borrow => Expr {
                            kind: ExprKind::Prefix { op: PrefixOp::Read, expr: base.clone() },
                            span,
                        },
                        Some(ParamMode::Write) if !base_is_borrow => Expr {
                            kind: ExprKind::Prefix { op: PrefixOp::Write, expr: base.clone() },
                            span,
                        },
                        _ => (**base).clone(),
                    };
                    let mut all: Vec<Expr> = Vec::with_capacity(args.len() + 1);
                    all.push(recv);
                    all.extend(args.iter().cloned());
                    return self.eval_user_call(fnd, &sig, &all);
                }
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
                    // A `slice`/`slice_mut` value crossing a spawn arrives as a
                    // `read`/`write` reborrow of a slice place (the §2.1 borrow
                    // branch); for a slice a reborrow is the same fat pointer, so
                    // peel the prefix and pass the value directly, WITHOUT consuming
                    // the source (a borrow, not a move).
                    let (a, reborrow) = match &a.kind {
                        ExprKind::Prefix { op: PrefixOp::Read | PrefixOp::Write, expr }
                            if matches!(p.lowered, Type::Slice(_) | Type::SliceMut(_)) =>
                        {
                            (expr.as_ref(), true)
                        }
                        _ => (a, false),
                    };
                    let rv = self.eval_value(a, Some(&p.lowered))?;
                    let sz = self.size_of(&rv.ty);
                    let bytes = self.read_bytes(rv.addr, sz, true)?;
                    if !reborrow && !is_copy(&rv.ty, self.items) {
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

    /// A foreign (`extern`) call (design 0011 §5): read each argument as a scalar
    /// word, dispatch through the shim registry, and materialize the shim's word
    /// result as the declared return type. An unregistered symbol raises the
    /// defined `no_foreign_runtime` fault.
    fn eval_extern_call(&mut self, es: &crate::resolve::ExternSig, args: &[Expr], span: Span) -> R<RVal> {
        self.cur_span = span;
        let mut vals: Vec<i128> = Vec::with_capacity(args.len());
        for (p, a) in es.params.iter().zip(args) {
            let a: &Expr = match &a.kind {
                ExprKind::OutArg(inner) => inner,
                _ => a,
            };
            let rv = self.eval_value(a, Some(&p.lowered))?;
            let sty = foreign_scalar_of(&p.lowered);
            vals.push(self.read_int(rv.addr, sty)?);
        }
        let result = match crate::foreign::dispatch(&es.name, &vals, &mut self.mem) {
            Some(v) => v,
            None => {
                // Deliver at the extern declaration span so the fault is identical
                // across the tree-walker and MIR engines (design 0011 §5).
                return Err(Ctl::Fault(Fault::new(
                    FaultKind::NoForeignRuntime,
                    es.span,
                    format!("no foreign runtime for `{}` (no shim registered; native backend is a 0010 forward dependency)", es.name),
                )));
            }
        };
        if self.size_of(&es.ret) == 0 {
            return Ok(self.unit_val());
        }
        let (addr, id) = self.alloc_temp(es.ret.clone());
        self.write_int(addr, result, foreign_scalar_of(&es.ret))?;
        Ok(RVal { ty: es.ret.clone(), addr, origin: Origin::Temp(id) })
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
            "as_bytes" => {
                // Free retype: the str's fat pointer IS the byte view (design 0013).
                let sv = self.eval_value(&args[0], None)?;
                let a = self.mem.stack_alloc(16, 8);
                self.move_bytes(a, sv.addr, 16)?;
                RVal { ty: Type::Slice(Box::new(Type::Scalar(ScalarTy::U8))), addr: a, origin: Origin::None }
            }
            "str_from_unchecked" => {
                // Skip validation, retype [u8] -> str (unsafe; design 0013 §4).
                let sv = self.eval_value(&args[0], None)?;
                let a = self.mem.stack_alloc(16, 8);
                self.move_bytes(a, sv.addr, 16)?;
                RVal { ty: Type::Str, addr: a, origin: Origin::None }
            }
            "str_from" => self.bi_str_from(args)?,
            "substr" => self.bi_substr(args, span)?,
            "char_at" => self.bi_char_at(args, span)?,
            "char_count" => self.bi_char_count(args)?,
            "string_new" => self.bi_string_new(args)?,
            "push" if self.arg0_is_string(args) => self.bi_string_push(args, span)?,
            "append" if self.arg0_is_string(args) => self.bi_string_append(args)?,
            "as_str" if self.arg0_is_string(args) => self.bi_string_as_str(args)?,
            // std growable `Vec[T]` (allocator-explicit, alloc-copy-free growth).
            "vec_new" => self.bi_vec_new(args)?,
            "push" if self.arg0_is_vec(args) => self.bi_vec_push(args, span)?,
            "pop" if self.arg0_is_vec(args) => self.bi_vec_pop(args, span)?,
            "get" if self.arg0_is_vec(args) => self.bi_vec_get(args, span)?,
            "set" if self.arg0_is_vec(args) => self.bi_vec_set(args, span)?,
            // std hash `Map[V]` — byte-string keys (`str`/`[u8]`), FNV-1a hash,
            // open addressing + linear probing, alloc-copy-rehash-free growth.
            "map_new" => self.bi_map_new(args)?,
            "insert" if self.arg0_is_map(args) => self.bi_map_insert(args, span)?,
            "contains" if self.arg0_is_map(args) => self.bi_map_contains(args, span)?,
            "get" if self.arg0_is_map(args) => self.bi_map_get(args, span)?,
            // Structured-concurrency blessed primitives (design 0012 §1.4, §3.3).
            "split_mut" => self.bi_split_mut(args, span)?,
            "cancelled" => {
                // Sequential oracle: a task always runs to completion, so a token
                // never signals — `cancelled` is always `false` (design 0012 §3.3).
                let _ = self.eval_value(&args[0], None)?;
                let a = self.mem.stack_alloc(1, 1);
                self.write_bytes(a, &[0u8])?;
                RVal { ty: Type::bool(), addr: a, origin: Origin::None }
            }
            "sqrt" => {
                // Correctly-rounded IEEE square root (design 0016 §11); never faults.
                let v = self.eval_value(&args[0], None)?;
                let sty = self.concretize(&v.ty);
                let bits = self.read_float_bits(v.addr, sty)?;
                let a = self.alloc_float(sty, float_sqrt(sty, bits))?;
                RVal { ty: Type::Scalar(sty), addr: a, origin: Origin::None }
            }
            "trace" => {
                // Observe the arg at its own width: an `f64` traces its bit pattern,
                // and a float arithmetic arg is computed with IEEE ops (design 0016).
                let sty = match self.eval_ty_probe(&args[0]) {
                    Some(Type::Scalar(s)) if s.is_float() => s,
                    _ => ScalarTy::I64,
                };
                let v = self.eval_value(&args[0], Some(&Type::Scalar(sty)))?;
                let n = self.read_int(v.addr, sty)?;
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
        let (_, ctx_off) = self.field_offset(self.alloc_struct_name(), "ctx");
        let (_, vt_off) = self.field_offset(self.alloc_struct_name(), "vt");
        let ctx = self.read_u64(alloc_addr + ctx_off)?;
        let vt = self.read_u64(alloc_addr + vt_off)?;
        let vv = self.eval_value(&args[1], None)?;
        let t = vv.ty.clone();
        let size = self.size_of(&t);
        let align = self.align_of(&t);
        let (_, alloc_off) = self.field_offset(self.alloc_vtable_name(), "alloc");
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
        let (_, free_off) = self.field_offset(self.alloc_vtable_name(), "free");
        let ffn = self.read_u64(vt + free_off)?;
        self.call_scalar(ffn, vec![
            (Type::RawPtr(Box::new(Type::Scalar(ScalarTy::U8))), ctx),
            (Type::RawPtr(Box::new(Type::Scalar(ScalarTy::U8))), ptr),
            (Type::usize(), size),
            (Type::usize(), align),
        ])?;
        Ok(())
    }

    /// Grow `ptr` (holding `old_size` bytes) to `new_size` through the allocator's
    /// `realloc` vtable slot; returns the (possibly moved) pointer, 0 on OOM.
    fn call_realloc(&mut self, ctx: u64, vt: u64, ptr: u64, old_size: u64, new_size: u64, align: u64) -> R<u64> {
        let (_, realloc_off) = self.field_offset(self.alloc_vtable_name(), "realloc");
        let rfn = self.read_u64(vt + realloc_off)?;
        self.call_scalar(rfn, vec![
            (Type::RawPtr(Box::new(Type::Scalar(ScalarTy::U8))), ctx),
            (Type::RawPtr(Box::new(Type::Scalar(ScalarTy::U8))), ptr),
            (Type::usize(), old_size),
            (Type::usize(), new_size),
            (Type::usize(), align),
        ])
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

    /// `split_mut(parent, mid, out lo, out hi)` — the blessed disjoint partition
    /// (design 0012 §1.4). Writes two `slice_mut` fat pointers into the caller's
    /// `out` slots: `lo = [base, mid)` and `hi = [base + mid, total)`. The compile
    /// -time disjointness stipulation is enforced by the checker; at run time this
    /// is ordinary sub-slice arithmetic.
    fn bi_split_mut(&mut self, args: &[Expr], span: Span) -> R<RVal> {
        let (addr, ty, _pl) = self.eval_place(&args[0])?;
        let (elem, base, total) = match &ty {
            Type::Array(e, len) => ((**e).clone(), addr, self.lay().array_len(len)),
            Type::SliceMut(e) | Type::Slice(e) => {
                let ptr = self.read_u64(addr)?;
                let ln = self.read_u64(addr + 8)?;
                ((**e).clone(), ptr, ln)
            }
            _ => (Type::Error, addr, 0),
        };
        let midv = self.eval_value(&args[1], Some(&Type::usize()))?;
        let mid = self.read_u64(midv.addr)?;
        // `split_mut` bounds-faults when `mid > len` (design 0012 §1.4); the fault
        // is stamped at the whole-call span so its identity matches every engine.
        if mid > total {
            self.cur_span = span;
            return Err(self.fault(
                FaultKind::Bounds,
                format!("split_mut index {mid} out of bounds for len {total}"),
            ));
        }
        let stride = round_up(self.size_of(&elem), self.align_of(&elem));
        self.write_slice_slot(&args[2], base, mid)?;
        self.write_slice_slot(&args[3], base + mid * stride, total - mid)?;
        Ok(self.unit_val())
    }

    /// Write a `slice_mut` fat pointer `{ptr, len}` into an `out place` slot.
    fn write_slice_slot(&mut self, outarg: &Expr, ptr: u64, len: u64) -> R<()> {
        let inner = match &outarg.kind {
            ExprKind::OutArg(i) => i.as_ref(),
            _ => outarg,
        };
        let (slot, _ty, _pl) = self.eval_place(inner)?;
        self.write_bytes(slot, &ptr.to_le_bytes())?;
        self.write_bytes(slot + 8, &len.to_le_bytes())?;
        Ok(())
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
        // Peel every borrow layer so a re-borrow (`len(read v)` where `v: read
        // Vec[T]` — a pointer-to-pointer) reaches the same base as `len(v)`. The
        // length field is at offset 8 of the Vec/Map the borrow chain points at.
        let (base, ty) = self.deref_borrows(&sv)?;
        if matches!(&ty, Type::App(n, _) if n == "Vec" || n == "Map") {
            let n = self.read_u64(base + 8)?;
            return self.usize_val(n);
        }
        let n = match &ty {
            Type::Slice(_) | Type::SliceMut(_) | Type::Str => self.read_u64(base + 8)?,
            Type::Array(_, len) => self.lay().array_len(len),
            _ => 0,
        };
        self.usize_val(n)
    }

    /// `str_from(b: [u8]) -> Utf8Res` (design 0013 §4): validate UTF-8, returning
    /// `Utf8Res::Valid(str)` (the str borrows `b`) or `Utf8Res::Invalid(offset)`
    /// carrying the byte offset of the first ill-formed sequence (P7 error value).
    fn bi_str_from(&mut self, args: &[Expr]) -> R<RVal> {
        let sv = self.eval_value(&args[0], None)?;
        let ptr = self.read_u64(sv.addr)?;
        let len = self.read_u64(sv.addr + 8)?;
        let bytes = self.read_bytes(ptr, len, true)?;
        match utf8_valid_up_to(&bytes) {
            None => {
                // Valid: the str reuses the SAME fat pointer (a validated view of b).
                let src = sv.addr;
                self.build_named_enum("Utf8Res", "Valid", &[(Type::Str, src)])
            }
            Some(off) => {
                let tmp = self.mem.stack_alloc(8, 8);
                self.write_bytes(tmp, &(off as u64).to_le_bytes())?;
                self.build_named_enum("Utf8Res", "Invalid", &[(Type::usize(), tmp)])
            }
        }
    }

    /// `substr(s, lo, hi) -> str` (design 0013 §3): the sub-view `[lo, hi)`, which
    /// FAULTS (P5) on an out-of-bounds OR non-UTF-8-char-boundary offset. The
    /// boundary and bounds faults reuse `FaultKind::Bounds` (one fault family for
    /// "this offset is not valid for this str"), distinguished by message.
    fn bi_substr(&mut self, args: &[Expr], span: Span) -> R<RVal> {
        let sv = self.eval_value(&args[0], None)?;
        let ptr = self.read_u64(sv.addr)?;
        let len = self.read_u64(sv.addr + 8)?;
        let lo = { let v = self.eval_value(&args[1], Some(&Type::usize()))?; self.read_u64(v.addr)? };
        let hi = { let v = self.eval_value(&args[2], Some(&Type::usize()))?; self.read_u64(v.addr)? };
        self.cur_span = span;
        if lo > hi || hi > len {
            return Err(self.fault(FaultKind::Bounds, format!("substr [{lo}..{hi}) out of bounds for str of len {len}")));
        }
        let bytes = self.read_bytes(ptr, len, true)?;
        if !str_is_boundary(&bytes, lo as usize) || !str_is_boundary(&bytes, hi as usize) {
            return Err(self.fault(FaultKind::Bounds, format!("substr [{lo}..{hi}) does not fall on a UTF-8 character boundary")));
        }
        let a = self.mem.stack_alloc(16, 8);
        self.write_bytes(a, &(ptr + lo).to_le_bytes())?;
        self.write_bytes(a + 8, &(hi - lo).to_le_bytes())?;
        Ok(RVal { ty: Type::Str, addr: a, origin: Origin::None })
    }

    /// `char_at(s: str, pos: usize) -> CharStep` (OBL-TEXT-CHARS value-gear
    /// decoder): decode the UTF-8 scalar at byte offset `pos`, returning
    /// `{ cp, next }` — the code point and the position just past it, both OWNED
    /// (no borrow, no alloc, no iterator struct). Since `str` GUARANTEES
    /// well-formed UTF-8 (design 0013 §4), a valid `str` with `pos` on a boundary
    /// and `pos < len` always decodes. DEFENSIVE behavior on a non-boundary /
    /// ill-formed / truncated `pos` is a FAULT (P5) — the same `Bounds` family as
    /// `substr`'s boundary fault, because decoding valid text off a boundary is a
    /// bug, not a routine P7 error. A false `str` forged via `str_from_unchecked`
    /// yields wrong values here, never UB (0013 §4). `pos == len` also faults:
    /// there is no scalar to decode at the end.
    fn bi_char_at(&mut self, args: &[Expr], span: Span) -> R<RVal> {
        let sv = self.eval_value(&args[0], None)?;
        let ptr = self.read_u64(sv.addr)?;
        let len = self.read_u64(sv.addr + 8)?;
        let pos = { let v = self.eval_value(&args[1], Some(&Type::usize()))?; self.read_u64(v.addr)? };
        self.cur_span = span;
        if pos >= len {
            return Err(self.fault(FaultKind::Bounds, format!("char_at pos {pos} out of bounds for str of len {len}")));
        }
        let bytes = self.read_bytes(ptr, len, true)?;
        match utf8_decode_at(&bytes, pos as usize) {
            Some((cp, next)) => {
                let ty = Type::Named("CharStep".to_string());
                let (addr, id) = self.alloc_temp(ty.clone());
                let (_, cp_off) = self.field_offset("CharStep", "cp");
                let (_, next_off) = self.field_offset("CharStep", "next");
                self.write_bytes(addr + cp_off, &cp.to_le_bytes())?;
                self.write_bytes(addr + next_off, &(next as u64).to_le_bytes())?;
                Ok(RVal { ty, addr, origin: Origin::Temp(id) })
            }
            None => Err(self.fault(
                FaultKind::Bounds,
                format!("char_at pos {pos} does not fall on a UTF-8 character boundary"),
            )),
        }
    }

    /// `char_count(s: str) -> usize` (design 0013 §3): the O(n) Unicode-scalar
    /// count — decode-and-advance to the end, counting. The deferred op landing
    /// with the char protocol. Cannot fault on a valid (well-formed) `str`.
    fn bi_char_count(&mut self, args: &[Expr]) -> R<RVal> {
        let sv = self.eval_value(&args[0], None)?;
        let ptr = self.read_u64(sv.addr)?;
        let len = self.read_u64(sv.addr + 8)?;
        let bytes = self.read_bytes(ptr, len, true)?;
        let mut pos = 0usize;
        let mut n = 0u64;
        while pos < bytes.len() {
            match utf8_decode_at(&bytes, pos) {
                Some((_, next)) => {
                    pos = next;
                    n += 1;
                }
                None => return Err(self.fault(FaultKind::Bounds, format!("char_count: ill-formed UTF-8 at byte {pos}"))),
            }
        }
        self.usize_val(n)
    }

    /// Build a value of the program-defined `Named` enum `nominal`, variant
    /// `variant`, copying each payload field from a source address. Used by the
    /// compiler-known text ops that yield an in-language enum (`Utf8Res`, `Opt`).
    fn build_named_enum(&mut self, nominal: &str, variant: &str, payloads: &[(Type, u64)]) -> R<RVal> {
        let einfo = self
            .lay()
            .enum_info(&Type::Named(nominal.to_string()))
            .ok_or_else(|| self.fault(FaultKind::Panic, format!("unknown enum `{nominal}` (define it to use this text op)")))?;
        let idx = einfo
            .iter()
            .position(|(n, _)| n == variant)
            .ok_or_else(|| self.fault(FaultKind::Panic, format!("enum `{nominal}` has no variant `{variant}`")))?;
        let decl_payloads = einfo[idx].1.clone();
        let ty = Type::Named(nominal.to_string());
        let (addr, id) = self.alloc_temp(ty.clone());
        self.write_bytes(addr, &(idx as u64).to_le_bytes())?;
        for (i, (pty, src)) in payloads.iter().enumerate() {
            let (_declty, off) = self.lay().payload_offset(&decl_payloads, i);
            let sz = self.size_of(pty);
            self.move_bytes(addr + off, *src, sz)?;
        }
        Ok(RVal { ty, addr, origin: Origin::Temp(id) })
    }

    /// Read the `(ptr, len)` of a `str`/`[u8]` expression, peeling a `read`/`write`
    /// borrow (a slice/str reborrow is the same fat pointer).
    fn read_byteview(&mut self, e: &Expr) -> R<(u64, u64)> {
        let v = self.eval_value(e, None)?;
        let fat = match &v.ty {
            Type::Str | Type::Slice(_) | Type::SliceMut(_) => v.addr,
            Type::Borrow(inner) | Type::BorrowMut(inner)
                if matches!(**inner, Type::Str | Type::Slice(_) | Type::SliceMut(_)) =>
            {
                self.read_u64(v.addr)?
            }
            _ => v.addr,
        };
        let ptr = self.read_u64(fat)?;
        let len = self.read_u64(fat + 8)?;
        Ok((ptr, len))
    }

    /// A side-effect-free static-type probe used to route `==`/ordering to the
    /// byte-wise `str` path (design 0013 §3).
    fn eval_ty_probe(&self, e: &Expr) -> Option<Type> {
        match &e.kind {
            ExprKind::StrLit(_) => Some(Type::Str),
            ExprKind::Paren(i) => self.eval_ty_probe(i),
            ExprKind::Call { callee, .. }
                if matches!(&callee.kind, ExprKind::Ident(n) if n == "substr" || n == "as_str" || n == "str_from_unchecked") =>
            {
                Some(Type::Str)
            }
            _ => self.expr_static_ty(e),
        }
    }

    /// Byte-lexicographic comparison of two `str` operands over their referent runs.
    fn str_byte_cmp(&mut self, lhs: &Expr, rhs: &Expr) -> R<std::cmp::Ordering> {
        let (lp, ll) = self.read_byteview(lhs)?;
        let (rp, rl) = self.read_byteview(rhs)?;
        let lb = self.read_bytes(lp, ll, true)?;
        let rb = self.read_bytes(rp, rl, true)?;
        Ok(lb.cmp(&rb))
    }

    /// Peel a `read`/`write` borrow and report whether arg0 is a `String` place.
    fn arg0_is_string(&self, args: &[Expr]) -> bool {
        let Some(a) = args.first() else { return false };
        let inner = match &a.kind {
            ExprKind::Prefix { op: PrefixOp::Read | PrefixOp::Write, expr } => expr.as_ref(),
            _ => a,
        };
        matches!(self.expr_static_ty(inner), Some(Type::Named(n)) if n == "String")
    }

    /// The base address of the `String` a `read`/`write` borrow argument names.
    fn string_base(&mut self, e: &Expr) -> R<u64> {
        let v = self.eval_value(e, None)?;
        Ok(self.deref_borrows(&v)?.0)
    }

    fn string_field_off(&self, field: &str) -> u64 {
        self.lay().field_offset("String", field).map(|(_, o)| o).unwrap_or(0)
    }

    /// `string_new(a: read Alloc) -> String` — an empty owning buffer carrying `a`.
    fn bi_string_new(&mut self, args: &[Expr]) -> R<RVal> {
        let av = self.eval_value(&args[0], None)?;
        let alloc_addr = match &av.ty {
            Type::Borrow(_) | Type::BorrowMut(_) => self.read_u64(av.addr)?,
            _ => av.addr,
        };
        let (_, ctx_off) = self.field_offset(self.alloc_struct_name(), "ctx");
        let (_, vt_off) = self.field_offset(self.alloc_struct_name(), "vt");
        let ctx = self.read_u64(alloc_addr + ctx_off)?;
        let vt = self.read_u64(alloc_addr + vt_off)?;
        let ty = Type::Named("String".to_string());
        let (addr, id) = self.alloc_temp(ty.clone());
        self.write_bytes(addr + self.string_field_off("buf"), &0u64.to_le_bytes())?;
        self.write_bytes(addr + self.string_field_off("len"), &0u64.to_le_bytes())?;
        self.write_bytes(addr + self.string_field_off("cap"), &0u64.to_le_bytes())?;
        self.write_bytes(addr + self.string_field_off("ctx"), &ctx.to_le_bytes())?;
        self.write_bytes(addr + self.string_field_off("vt"), &vt.to_le_bytes())?;
        Ok(RVal { ty, addr, origin: Origin::Temp(id) })
    }

    /// Call the carried allocator's `alloc` vtable slot; returns the new pointer.
    fn string_vt_alloc(&mut self, ctx: u64, vt: u64, size: u64) -> R<u64> {
        let (_, alloc_off) = self.field_offset(self.alloc_vtable_name(), "alloc");
        let afn = self.read_u64(vt + alloc_off)?;
        self.call_scalar(afn, vec![
            (Type::RawPtr(Box::new(Type::Scalar(ScalarTy::U8))), ctx),
            (Type::usize(), size),
            (Type::usize(), 1),
        ])
    }

    /// Ensure the String at `base` has room for `need` more bytes, growing through
    /// its carried allocator's `realloc` (grow-in-place when possible, else move-
    /// copy). Faults on OOM (prototype: `push`/`append` return no result).
    fn string_reserve(&mut self, base: u64, need: u64) -> R<()> {
        let buf_off = self.string_field_off("buf");
        let len = self.read_u64(base + self.string_field_off("len"))?;
        let cap = self.read_u64(base + self.string_field_off("cap"))?;
        if len + need <= cap {
            return Ok(());
        }
        let newcap = (len + need).max(cap * 2).max(8);
        let ctx = self.read_u64(base + self.string_field_off("ctx"))?;
        let vt = self.read_u64(base + self.string_field_off("vt"))?;
        let oldbuf = self.read_u64(base + buf_off)?;
        let newbuf = if oldbuf == 0 {
            self.string_vt_alloc(ctx, vt, newcap)?
        } else {
            self.call_realloc(ctx, vt, oldbuf, len, newcap, 1)?
        };
        if newbuf == 0 {
            return Err(self.fault(FaultKind::Panic, "String allocation failed (OOM)"));
        }
        self.write_bytes(base + buf_off, &newbuf.to_le_bytes())?;
        self.write_bytes(base + self.string_field_off("cap"), &newcap.to_le_bytes())?;
        Ok(())
    }

    /// `push(write self, c: u32)` — append one UTF-8-encoded Unicode scalar. The
    /// `enforced requires(is_scalar_value(c))` is a P5 backstop: a surrogate or
    /// out-of-range code point FAULTS (design 0013 §3), never forging a false str.
    fn bi_string_push(&mut self, args: &[Expr], span: Span) -> R<RVal> {
        let base = self.string_base(&args[0])?;
        let cv = self.eval_value(&args[1], Some(&Type::Scalar(ScalarTy::U32)))?;
        let c = self.read_int(cv.addr, ScalarTy::U32)? as u32;
        self.cur_span = span;
        let enc = match utf8_encode_scalar(c) {
            Some(e) => e,
            None => return Err(self.fault(FaultKind::Requires, format!("push: {c:#x} is not a Unicode scalar value (is_scalar_value backstop)"))),
        };
        self.string_reserve(base, enc.len() as u64)?;
        let buf = self.read_u64(base + self.string_field_off("buf"))?;
        let len = self.read_u64(base + self.string_field_off("len"))?;
        self.write_bytes(buf + len, &enc)?;
        self.write_bytes(base + self.string_field_off("len"), &(len + enc.len() as u64).to_le_bytes())?;
        Ok(self.unit_val())
    }

    /// `append(write self, s: read str)` — append a view's bytes (already valid).
    fn bi_string_append(&mut self, args: &[Expr]) -> R<RVal> {
        let base = self.string_base(&args[0])?;
        let (ptr, slen) = self.read_byteview(&args[1])?;
        let bytes = self.read_bytes(ptr, slen, true)?;
        self.string_reserve(base, slen)?;
        let buf = self.read_u64(base + self.string_field_off("buf"))?;
        let len = self.read_u64(base + self.string_field_off("len"))?;
        self.write_bytes(buf + len, &bytes)?;
        self.write_bytes(base + self.string_field_off("len"), &(len + slen).to_le_bytes())?;
        Ok(self.unit_val())
    }

    /// `as_str(read self) -> str` — a validated view over the built bytes.
    fn bi_string_as_str(&mut self, args: &[Expr]) -> R<RVal> {
        let base = self.string_base(&args[0])?;
        let buf = self.read_u64(base + self.string_field_off("buf"))?;
        let len = self.read_u64(base + self.string_field_off("len"))?;
        let a = self.mem.stack_alloc(16, 8);
        self.write_bytes(a, &buf.to_le_bytes())?;
        self.write_bytes(a + 8, &len.to_le_bytes())?;
        Ok(RVal { ty: Type::Str, addr: a, origin: Origin::None })
    }

    // ===================================================================
    // std growable `Vec[T]` (PROPOSAL-selfhost-ergonomics candidate A). Compiler-
    // known, allocator-explicit. Layout mirrors `String`:
    // `{ buf: rawptr @0, len @8, cap @16, ctx @24, vt @32 }` (5 u64 words).
    // Growth goes through the allocator's `realloc` (grow-in-place or move-copy).
    // ===================================================================

    fn arg0_is_vec(&self, args: &[Expr]) -> bool {
        let Some(a) = args.first() else { return false };
        let inner = match &a.kind {
            ExprKind::Prefix { op: PrefixOp::Read | PrefixOp::Write, expr } => expr.as_ref(),
            _ => a,
        };
        let t = match self.expr_static_ty(inner) {
            Some(Type::Borrow(b)) | Some(Type::BorrowMut(b)) => Some(*b),
            other => other,
        };
        matches!(t, Some(Type::App(n, _)) if n == "Vec")
    }

    /// The element type of the `Vec[T]` named by `e` (peeling `read`/`write`).
    fn vec_elem_of(&self, e: &Expr) -> Type {
        let inner = match &e.kind {
            ExprKind::Prefix { op: PrefixOp::Read | PrefixOp::Write, expr } => expr.as_ref(),
            _ => e,
        };
        let t = match self.expr_static_ty(inner) {
            Some(Type::Borrow(b)) | Some(Type::BorrowMut(b)) => Some(*b),
            other => other,
        };
        match t {
            Some(Type::App(n, targs)) if n == "Vec" => targs.first().cloned().unwrap_or(Type::Error),
            _ => Type::Error,
        }
    }

    /// Peel EVERY borrow layer of an evaluated receiver, following each pointer,
    /// to the base address of the underlying value and its type. A bare `read v`
    /// on an already-borrowed param (`v: read Vec[T]`) is a pointer-to-pointer, so
    /// a single deref stops one level short; this follows the whole chain so a
    /// re-borrow dispatches to the same base as the direct `v`.
    fn deref_borrows(&mut self, rv: &RVal) -> R<(u64, Type)> {
        let mut addr = rv.addr;
        let mut ty = rv.ty.clone();
        while let Type::Borrow(inner) | Type::BorrowMut(inner) = ty {
            addr = self.read_u64(addr)?;
            ty = *inner;
        }
        Ok((addr, ty))
    }

    /// The base address of the `Vec` a `read`/`write`/owned argument names.
    fn vec_base(&mut self, e: &Expr) -> R<u64> {
        let v = self.eval_value(e, None)?;
        Ok(self.deref_borrows(&v)?.0)
    }

    fn vec_stride(&self, elem: &Type) -> u64 {
        round_up(self.size_of(elem), self.align_of(elem))
    }

    /// `vec_new(a: read Alloc) -> Vec[T]` — an empty buffer carrying `a`.
    fn bi_vec_new(&mut self, args: &[Expr]) -> R<RVal> {
        let av = self.eval_value(&args[0], None)?;
        let alloc_addr = match &av.ty {
            Type::Borrow(_) | Type::BorrowMut(_) => self.read_u64(av.addr)?,
            _ => av.addr,
        };
        let (_, ctx_off) = self.field_offset(self.alloc_struct_name(), "ctx");
        let (_, vt_off) = self.field_offset(self.alloc_struct_name(), "vt");
        let ctx = self.read_u64(alloc_addr + ctx_off)?;
        let vt = self.read_u64(alloc_addr + vt_off)?;
        let ty = Type::App("Vec".to_string(), vec![Type::Error]);
        let (addr, id) = self.alloc_temp(ty.clone());
        self.write_bytes(addr, &0u64.to_le_bytes())?;       // buf @0
        self.write_bytes(addr + 8, &0u64.to_le_bytes())?;   // len @8
        self.write_bytes(addr + 16, &0u64.to_le_bytes())?;  // cap @16
        self.write_bytes(addr + 24, &ctx.to_le_bytes())?;   // ctx @24
        self.write_bytes(addr + 32, &vt.to_le_bytes())?;    // vt @32
        Ok(RVal { ty, addr, origin: Origin::Temp(id) })
    }

    /// Ensure room for `need` more elements, growing via the allocator's `realloc`.
    fn vec_reserve(&mut self, base: u64, elem: &Type, need: u64) -> R<()> {
        let len = self.read_u64(base + 8)?;
        let cap = self.read_u64(base + 16)?;
        if len + need <= cap {
            return Ok(());
        }
        let stride = self.vec_stride(elem);
        let align = self.align_of(elem);
        let newcap = (len + need).max(cap * 2).max(4);
        let ctx = self.read_u64(base + 24)?;
        let vt = self.read_u64(base + 32)?;
        let oldbuf = self.read_u64(base)?;
        let newbuf = if oldbuf == 0 {
            let (_, alloc_off) = self.field_offset(self.alloc_vtable_name(), "alloc");
            let afn = self.read_u64(vt + alloc_off)?;
            self.call_scalar(afn, vec![
                (Type::RawPtr(Box::new(Type::Scalar(ScalarTy::U8))), ctx),
                (Type::usize(), newcap * stride),
                (Type::usize(), align),
            ])?
        } else {
            self.call_realloc(ctx, vt, oldbuf, len * stride, newcap * stride, align)?
        };
        if newbuf == 0 {
            return Err(self.fault(FaultKind::Panic, "Vec allocation failed (OOM)"));
        }
        self.write_bytes(base, &newbuf.to_le_bytes())?;
        self.write_bytes(base + 16, &newcap.to_le_bytes())?;
        Ok(())
    }

    /// `push(write self: Vec[T], v: T)` — moves `v` onto the end, growing if full.
    fn bi_vec_push(&mut self, args: &[Expr], span: Span) -> R<RVal> {
        let elem = self.vec_elem_of(&args[0]);
        let base = self.vec_base(&args[0])?;
        self.cur_span = span;
        let rv = self.eval_value(&args[1], Some(&elem))?;
        self.vec_reserve(base, &elem, 1)?;
        let stride = self.vec_stride(&elem);
        let buf = self.read_u64(base)?;
        let len = self.read_u64(base + 8)?;
        self.move_to(buf + len * stride, rv)?;
        self.write_bytes(base + 8, &(len + 1).to_le_bytes())?;
        Ok(self.unit_val())
    }

    /// `pop(write self: Vec[T]) -> Opt[T]` — moves the last element out.
    fn bi_vec_pop(&mut self, args: &[Expr], span: Span) -> R<RVal> {
        let elem = self.vec_elem_of(&args[0]);
        let base = self.vec_base(&args[0])?;
        self.cur_span = span;
        let len = self.read_u64(base + 8)?;
        if len == 0 {
            return self.build_named_enum("Opt", "None", &[]);
        }
        let newlen = len - 1;
        let stride = self.vec_stride(&elem);
        let buf = self.read_u64(base)?;
        let src = buf + newlen * stride;
        self.write_bytes(base + 8, &newlen.to_le_bytes())?;
        self.build_named_enum("Opt", "Some", &[(elem, src)])
    }

    /// `get(read self: Vec[T], i: usize) -> read T` — a bounds-faulting borrow.
    fn bi_vec_get(&mut self, args: &[Expr], span: Span) -> R<RVal> {
        let elem = self.vec_elem_of(&args[0]);
        let base = self.vec_base(&args[0])?;
        self.cur_span = span;
        let len = self.read_u64(base + 8)?;
        let iv = self.eval_value(&args[1], Some(&Type::usize()))?;
        let i = self.read_u64(iv.addr)?;
        if i >= len {
            return Err(self.fault(FaultKind::Bounds, format!("Vec index {i} out of bounds (len {len})")));
        }
        let stride = self.vec_stride(&elem);
        let buf = self.read_u64(base)?;
        let a = self.mem.stack_alloc(8, 8);
        self.write_bytes(a, &(buf + i * stride).to_le_bytes())?;
        Ok(RVal { ty: Type::Borrow(Box::new(elem)), addr: a, origin: Origin::None })
    }

    /// `set(write self: Vec[T], i: usize, v: T)` — drops the overwritten element,
    /// then moves `v` into slot `i` (bounds-faulting).
    fn bi_vec_set(&mut self, args: &[Expr], span: Span) -> R<RVal> {
        let elem = self.vec_elem_of(&args[0]);
        let base = self.vec_base(&args[0])?;
        self.cur_span = span;
        let len = self.read_u64(base + 8)?;
        let iv = self.eval_value(&args[1], Some(&Type::usize()))?;
        let i = self.read_u64(iv.addr)?;
        if i >= len {
            return Err(self.fault(FaultKind::Bounds, format!("Vec index {i} out of bounds (len {len})")));
        }
        let stride = self.vec_stride(&elem);
        let buf = self.read_u64(base)?;
        let dst = buf + i * stride;
        let rv = self.eval_value(&args[2], Some(&elem))?;
        self.drop_value(dst, &elem, &MoveMask::default(), &mut Vec::new())?;
        self.move_to(dst, rv)?;
        Ok(self.unit_val())
    }

    // ===================================================================
    // Compiler-known std hash `Map[V]` (PROPOSAL-selfhost-ergonomics cand. B).
    //
    // KEYS are byte-strings (`str`/`[u8]`) — the self-host hot case (the keyword
    // ladder, the checker item table). No user-defined-key hashing (the "refuse
    // the language form" ruling); the VALUE type `V` is generic. Layout mirrors
    // `Vec`/`String`: `{ buf: rawptr @0, len @8, cap @16, ctx @24, vt @32 }` (5 u64
    // words). `buf` points at `cap` open-addressed BUCKETS, each
    // `{ state: u64 @0 (0=empty,1=occupied), keyptr @8, keylen @16, value: V @24 }`
    // with stride `round_up(24 + size_of(V), 8)`. The hash is 64-bit FNV-1a over
    // the key bytes; the slot is `hash & (cap-1)` with linear probing (`cap` is a
    // power of two). The map OWNS a heap byte-copy of every key (freed on drop).
    // Growth is alloc-new + rehash-move + free-old at load factor 3/4. `len` is
    // read at offset 8 by the shared `len` builtin (same as `Vec`).
    // ===================================================================

    fn arg0_is_map(&self, args: &[Expr]) -> bool {
        let Some(a) = args.first() else { return false };
        let inner = match &a.kind {
            ExprKind::Prefix { op: PrefixOp::Read | PrefixOp::Write, expr } => expr.as_ref(),
            _ => a,
        };
        let t = match self.expr_static_ty(inner) {
            Some(Type::Borrow(b)) | Some(Type::BorrowMut(b)) => Some(*b),
            other => other,
        };
        matches!(t, Some(Type::App(n, _)) if n == "Map")
    }

    /// The value type `V` of the `Map[V]` named by `e` (peeling `read`/`write`).
    fn map_valty_of(&self, e: &Expr) -> Type {
        let inner = match &e.kind {
            ExprKind::Prefix { op: PrefixOp::Read | PrefixOp::Write, expr } => expr.as_ref(),
            _ => e,
        };
        let t = match self.expr_static_ty(inner) {
            Some(Type::Borrow(b)) | Some(Type::BorrowMut(b)) => Some(*b),
            other => other,
        };
        match t {
            Some(Type::App(n, targs)) if n == "Map" => targs.first().cloned().unwrap_or(Type::Error),
            _ => Type::Error,
        }
    }

    /// The base address of the `Map` a `read`/`write`/owned argument names.
    fn map_base(&mut self, e: &Expr) -> R<u64> {
        let v = self.eval_value(e, None)?;
        Ok(self.deref_borrows(&v)?.0)
    }

    fn map_stride(&self, valty: &Type) -> u64 {
        round_up(24 + self.size_of(valty), 8)
    }

    /// 64-bit FNV-1a over the key bytes (deterministic; offset basis
    /// 0xcbf29ce484222325, prime 0x100000001b3).
    fn map_hash(bytes: &[u8]) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h
    }

    fn map_vt_alloc(&mut self, ctx: u64, vt: u64, size: u64, align: u64) -> R<u64> {
        let (_, alloc_off) = self.field_offset(self.alloc_vtable_name(), "alloc");
        let afn = self.read_u64(vt + alloc_off)?;
        self.call_scalar(afn, vec![
            (Type::RawPtr(Box::new(Type::Scalar(ScalarTy::U8))), ctx),
            (Type::usize(), size),
            (Type::usize(), align),
        ])
    }

    /// The slot of `key` if present (an occupied bucket with matching bytes), else
    /// `None`. Linear probing terminates at the first empty bucket — no tombstones
    /// exist (no `remove` op ships).
    fn map_find(&mut self, buf: u64, cap: u64, stride: u64, key: &[u8]) -> R<Option<u64>> {
        if cap == 0 || buf == 0 {
            return Ok(None);
        }
        let mask = cap - 1;
        let mut idx = Self::map_hash(key) & mask;
        loop {
            let b = buf + idx * stride;
            if self.read_u64(b)? == 0 {
                return Ok(None);
            }
            let klen = self.read_u64(b + 16)?;
            if klen == key.len() as u64 {
                let kptr = self.read_u64(b + 8)?;
                if self.read_bytes(kptr, klen, true)? == key {
                    return Ok(Some(idx));
                }
            }
            idx = (idx + 1) & mask;
        }
    }

    /// The first empty slot along `key`'s probe chain — for inserting a NEW key
    /// (the caller has established `key` is absent and `cap > 0`, load factor < 1).
    fn map_find_empty(&mut self, buf: u64, cap: u64, stride: u64, key: &[u8]) -> R<u64> {
        let mask = cap - 1;
        let mut idx = Self::map_hash(key) & mask;
        loop {
            let b = buf + idx * stride;
            if self.read_u64(b)? == 0 {
                return Ok(idx);
            }
            idx = (idx + 1) & mask;
        }
    }

    /// Ensure room for one more entry; grow (initial 8, then x2) and rehash when the
    /// load factor would exceed 3/4. Alloc-new + rehash-move (key ptr/len + value
    /// bytes move; no key re-copy) + free-old.
    fn map_reserve(&mut self, base: u64, valty: &Type) -> R<()> {
        let len = self.read_u64(base + 8)?;
        let cap = self.read_u64(base + 16)?;
        if cap != 0 && (len + 1) * 4 <= cap * 3 {
            return Ok(());
        }
        let stride = self.map_stride(valty);
        let newcap = if cap == 0 { 8 } else { cap * 2 };
        let ctx = self.read_u64(base + 24)?;
        let vt = self.read_u64(base + 32)?;
        let newbuf = self.map_vt_alloc(ctx, vt, newcap * stride, 8)?;
        if newbuf == 0 {
            return Err(self.fault(FaultKind::Panic, "Map allocation failed (OOM)"));
        }
        self.write_bytes(newbuf, &vec![0u8; (newcap * stride) as usize])?;
        let oldbuf = self.read_u64(base)?;
        if oldbuf != 0 {
            let vsz = self.size_of(valty);
            for i in 0..cap {
                let ob = oldbuf + i * stride;
                if self.read_u64(ob)? == 1 {
                    let kptr = self.read_u64(ob + 8)?;
                    let klen = self.read_u64(ob + 16)?;
                    let kbytes = self.read_bytes(kptr, klen, true)?;
                    let slot = self.map_find_empty(newbuf, newcap, stride, &kbytes)?;
                    let nb = newbuf + slot * stride;
                    self.write_bytes(nb, &1u64.to_le_bytes())?;
                    self.write_bytes(nb + 8, &kptr.to_le_bytes())?;
                    self.write_bytes(nb + 16, &klen.to_le_bytes())?;
                    if vsz > 0 {
                        self.move_bytes(nb + 24, ob + 24, vsz)?;
                    }
                }
            }
            self.call_free(ctx, vt, oldbuf, cap * stride, 8)?;
        }
        self.write_bytes(base, &newbuf.to_le_bytes())?;
        self.write_bytes(base + 16, &newcap.to_le_bytes())?;
        Ok(())
    }

    /// `map_new(a: read Alloc) -> Map[V]` — an empty map carrying `a`.
    fn bi_map_new(&mut self, args: &[Expr]) -> R<RVal> {
        let av = self.eval_value(&args[0], None)?;
        let alloc_addr = match &av.ty {
            Type::Borrow(_) | Type::BorrowMut(_) => self.read_u64(av.addr)?,
            _ => av.addr,
        };
        let (_, ctx_off) = self.field_offset(self.alloc_struct_name(), "ctx");
        let (_, vt_off) = self.field_offset(self.alloc_struct_name(), "vt");
        let ctx = self.read_u64(alloc_addr + ctx_off)?;
        let vt = self.read_u64(alloc_addr + vt_off)?;
        let ty = Type::App("Map".to_string(), vec![Type::Error]);
        let (addr, id) = self.alloc_temp(ty.clone());
        self.write_bytes(addr, &0u64.to_le_bytes())?;      // buf @0
        self.write_bytes(addr + 8, &0u64.to_le_bytes())?;  // len @8
        self.write_bytes(addr + 16, &0u64.to_le_bytes())?; // cap @16
        self.write_bytes(addr + 24, &ctx.to_le_bytes())?;  // ctx @24
        self.write_bytes(addr + 32, &vt.to_le_bytes())?;   // vt @32
        Ok(RVal { ty, addr, origin: Origin::Temp(id) })
    }

    /// `insert(write self: Map[V], key: read str/[u8], v: V)` — moves `v` in. On an
    /// existing key, drops the displaced value and reuses the stored key copy; on a
    /// new key, allocates an owned byte-copy of the key (growing/rehashing if full).
    fn bi_map_insert(&mut self, args: &[Expr], span: Span) -> R<RVal> {
        let valty = self.map_valty_of(&args[0]);
        let base = self.map_base(&args[0])?;
        self.cur_span = span;
        let (kptr, klen) = self.read_byteview(&args[1])?;
        let key = self.read_bytes(kptr, klen, true)?;
        let rv = self.eval_value(&args[2], Some(&valty))?;
        let stride = self.map_stride(&valty);
        let buf0 = self.read_u64(base)?;
        let cap0 = self.read_u64(base + 16)?;
        if let Some(slot) = self.map_find(buf0, cap0, stride, &key)? {
            let voff = buf0 + slot * stride + 24;
            self.drop_value(voff, &valty, &MoveMask::default(), &mut Vec::new())?;
            self.move_to(voff, rv)?;
            return Ok(self.unit_val());
        }
        self.map_reserve(base, &valty)?;
        let buf = self.read_u64(base)?;
        let cap = self.read_u64(base + 16)?;
        let slot = self.map_find_empty(buf, cap, stride, &key)?;
        let ctx = self.read_u64(base + 24)?;
        let vt = self.read_u64(base + 32)?;
        let kbuf = self.map_vt_alloc(ctx, vt, klen, 1)?;
        if kbuf == 0 {
            return Err(self.fault(FaultKind::Panic, "Map key allocation failed (OOM)"));
        }
        self.write_bytes(kbuf, &key)?;
        let b = buf + slot * stride;
        self.write_bytes(b, &1u64.to_le_bytes())?;        // state = occupied
        self.write_bytes(b + 8, &kbuf.to_le_bytes())?;    // keyptr
        self.write_bytes(b + 16, &klen.to_le_bytes())?;   // keylen
        self.move_to(b + 24, rv)?;                        // value
        let len = self.read_u64(base + 8)?;
        self.write_bytes(base + 8, &(len + 1).to_le_bytes())?;
        Ok(self.unit_val())
    }

    /// `contains(read self: Map[V], key: read str/[u8]) -> bool`.
    fn bi_map_contains(&mut self, args: &[Expr], span: Span) -> R<RVal> {
        let valty = self.map_valty_of(&args[0]);
        let base = self.map_base(&args[0])?;
        self.cur_span = span;
        let (kptr, klen) = self.read_byteview(&args[1])?;
        let key = self.read_bytes(kptr, klen, true)?;
        let stride = self.map_stride(&valty);
        let buf = self.read_u64(base)?;
        let cap = self.read_u64(base + 16)?;
        let found = self.map_find(buf, cap, stride, &key)?.is_some();
        let a = self.mem.stack_alloc(1, 1);
        self.write_bytes(a, &[found as u8])?;
        Ok(RVal { ty: Type::bool(), addr: a, origin: Origin::None })
    }

    /// `get(read self: Map[V], key: read str/[u8]) -> read V` — a borrow of the
    /// stored value; FAULTS if the key is absent (region-free single-borrow-out,
    /// paired with `contains`, mirroring `Vec::get`).
    fn bi_map_get(&mut self, args: &[Expr], span: Span) -> R<RVal> {
        let valty = self.map_valty_of(&args[0]);
        let base = self.map_base(&args[0])?;
        self.cur_span = span;
        let (kptr, klen) = self.read_byteview(&args[1])?;
        let key = self.read_bytes(kptr, klen, true)?;
        let stride = self.map_stride(&valty);
        let buf = self.read_u64(base)?;
        let cap = self.read_u64(base + 16)?;
        match self.map_find(buf, cap, stride, &key)? {
            Some(slot) => {
                let voff = buf + slot * stride + 24;
                let a = self.mem.stack_alloc(8, 8);
                self.write_bytes(a, &voff.to_le_bytes())?;
                Ok(RVal { ty: Type::Borrow(Box::new(valty)), addr: a, origin: Origin::None })
            }
            None => Err(self.fault(FaultKind::Bounds, "Map key not found".to_string())),
        }
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
        if let Type::Scalar(s) = sv.ty {
            if s.is_integer() {
                return self.eval_int_match(arms, expected, sv, s);
            }
        }
        let (hold, enum_addr, enum_ty) = self.peel_scrutinee(sv.ty.clone(), sv.addr)?;
        let einfo = self
            .lay()
            .enum_info(&enum_ty)
            .ok_or_else(|| self.fault(FaultKind::Panic, "match on non-enum"))?;
        let tag = self.read_u64(enum_addr)? as usize;
        let (vname, payloads) = einfo[tag].clone();
        // Try each arm in order: its pattern must match the tag, and — if the arm
        // is guarded — the guard must evaluate true. A matching pattern whose
        // guard is FALSE falls through to test the following arms (design 0001
        // §8.2, extended): the guard failing must not skip a later matching arm.
        for arm in arms {
            if !pat_matches(&arm.pattern, &vname) {
                continue;
            }
            self.push_scope();
            self.bind_pattern(&arm.pattern, hold, enum_addr, &payloads, &sv)?;
            if !self.eval_arm_guard(arm)? {
                self.drop_scope()?;
                continue;
            }
            return match self.eval_value(&arm.body, expected) {
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
            };
        }
        Err(self.fault(FaultKind::Panic, "no matching arm"))
    }

    /// Evaluate a match arm's optional guard with its bindings in scope. Returns
    /// `true` when the arm has no guard or the guard is true (design 0001 §8.2).
    fn eval_arm_guard(&mut self, arm: &MatchArm) -> R<bool> {
        match &arm.guard {
            None => Ok(true),
            Some(guard) => {
                let gv = self.eval_value(guard, Some(&Type::bool()))?;
                Ok(self.read_bytes(gv.addr, 1, true)?[0] != 0)
            }
        }
    }

    /// Evaluate an integer-scrutinee `match`: read the scalar once, then pick the
    /// first arm whose literal equals it (or the `_`/binding catch-all). A binding
    /// arm binds the whole (Copy) integer value.
    fn eval_int_match(
        &mut self,
        arms: &[MatchArm],
        expected: Option<&Type>,
        sv: RVal,
        sty: ScalarTy,
    ) -> R<RVal> {
        let val = self.read_int(sv.addr, sty)?;
        // Try each arm in order: its pattern must match the value, and — if the
        // arm is guarded — the guard must evaluate true. A matching pattern with a
        // FALSE guard falls through to the following arms (design 0001 §8.2).
        for arm in arms {
            let matches = match &arm.pattern.kind {
                PatKind::IntLit { value, negative, .. } => {
                    crate::ast::int_pat_value(*value, *negative) == val
                }
                PatKind::IntRange { lo_value, lo_negative, hi_value, hi_negative, inclusive, .. } => {
                    let lo = crate::ast::int_pat_value(*lo_value, *lo_negative);
                    let hi = crate::ast::int_pat_value(*hi_value, *hi_negative);
                    lo <= val && if *inclusive { val <= hi } else { val < hi }
                }
                PatKind::Wildcard | PatKind::Binding(_) => true,
                PatKind::Variant { .. } => false,
            };
            if !matches {
                continue;
            }
            self.push_scope();
            if let PatKind::Binding(name) = &arm.pattern.kind {
                let size = self.size_of(&sv.ty).max(1);
                let align = self.align_of(&sv.ty).max(1);
                let a = self.mem.stack_alloc(size, align);
                self.move_bytes(a, sv.addr, size)?;
                self.add_local(name, a, sv.ty.clone(), MoveMask::default(), true);
            }
            if !self.eval_arm_guard(arm)? {
                self.drop_scope()?;
                continue;
            }
            return match self.eval_value(&arm.body, expected) {
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
            };
        }
        Err(self.fault(FaultKind::Panic, "no matching arm"))
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
            PatKind::IntLit { .. } | PatKind::IntRange { .. } => Ok(()),
        }
    }

    fn bind_sub(&mut self, pat: &Pattern, pty: &Type, hold: Hold, sub_addr: u64, sv: &RVal, idx: usize) -> R<()> {
        let name = match &pat.kind {
            PatKind::Wildcard | PatKind::IntLit { .. } | PatKind::IntRange { .. } => return Ok(()),
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
                // Evaluate the RHS *before* dropping the old value (design 0001
                // §1.5 evaluation order). The RHS may move the current value out
                // of the target place — `lst = cons(take lst, ...)` — and a
                // moved-out place is not dropped (§1.5). The drop-of-old below
                // therefore consults the move state the RHS just updated (via
                // `place_owned` and the local's live mask) rather than dropping
                // unconditionally, which double-freed the moved-out value.
                let rv = self.eval_value(value, Some(&tty))?;
                if self.place_is_local_direct(&pl) && self.place_owned(&pl) {
                    let mask = self.local_mask(&pl.root);
                    self.drop_value(addr, &tty, &mask, &mut Vec::new())?;
                }
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
            // Compiler-known std `Vec[T]`: drop each LIVE element (`0..len`), then
            // free the backing buffer through the carried allocator (alloc-on-drop).
            // Popped elements decremented `len`, so they are not re-dropped here.
            Type::App(n, args) if n == "Vec" => {
                let buf = self.read_u64(addr)?; // buf @0
                if buf != 0 {
                    let elem = args.first().cloned().unwrap_or(Type::Error);
                    let stride = round_up(self.size_of(&elem), self.align_of(&elem));
                    let len = self.read_u64(addr + 8)?; // len @8
                    for i in (0..len).rev() {
                        self.drop_value(buf + i * stride, &elem, &MoveMask::default(), &mut Vec::new())?;
                    }
                    let cap = self.read_u64(addr + 16)?; // cap @16
                    let ctx = self.read_u64(addr + 24)?; // ctx @24
                    let vt = self.read_u64(addr + 32)?; // vt @32
                    self.call_free(ctx, vt, buf, cap * stride, self.align_of(&elem))?;
                }
                Ok(())
            }
            // Compiler-known std hash `Map[V]`: free each LIVE key byte-copy, drop
            // each LIVE value, then free the bucket buffer (alloc-on-drop).
            Type::App(n, args) if n == "Map" => {
                let buf = self.read_u64(addr)?; // buf @0
                if buf != 0 {
                    let valty = args.first().cloned().unwrap_or(Type::Error);
                    let stride = round_up(24 + self.size_of(&valty), 8);
                    let cap = self.read_u64(addr + 16)?; // cap @16
                    let ctx = self.read_u64(addr + 24)?; // ctx @24
                    let vt = self.read_u64(addr + 32)?;  // vt @32
                    for i in 0..cap {
                        let b = buf + i * stride;
                        if self.read_u64(b)? == 1 {
                            let kptr = self.read_u64(b + 8)?;
                            let klen = self.read_u64(b + 16)?;
                            self.call_free(ctx, vt, kptr, klen, 1)?;
                            self.drop_value(b + 24, &valty, &MoveMask::default(), &mut Vec::new())?;
                        }
                    }
                    self.call_free(ctx, vt, buf, cap * stride, 8)?;
                }
                Ok(())
            }
            // Compiler-known `String` (design 0013): free its heap buffer through
            // the carried allocator's vtable (alloc-on-drop).
            Type::Named(n) if n == "String" => {
                let buf = self.read_u64(addr + self.string_field_off("buf"))?;
                if buf != 0 {
                    let cap = self.read_u64(addr + self.string_field_off("cap"))?;
                    let ctx = self.read_u64(addr + self.string_field_off("ctx"))?;
                    let vt = self.read_u64(addr + self.string_field_off("vt"))?;
                    self.call_free(ctx, vt, buf, cap, 1)?;
                }
                Ok(())
            }
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
        // Integer literal/range patterns never match an enum tag (checker rejects
        // this combination); listed for match completeness.
        PatKind::IntLit { .. } | PatKind::IntRange { .. } => false,
    }
}

/// UTF-8 validation (design 0013 §4). `None` if `bytes` is well-formed UTF-8;
/// otherwise `Some(offset)` — the byte offset of the first ill-formed sequence.
fn utf8_valid_up_to(bytes: &[u8]) -> Option<usize> {
    match std::str::from_utf8(bytes) {
        Ok(_) => None,
        Err(e) => Some(e.valid_up_to()),
    }
}

/// Is byte offset `i` a UTF-8 character boundary of the (already well-formed)
/// run `bytes`? A boundary is the start of the run, its end, or any byte that is
/// NOT a continuation byte `0x80..=0xBF` (design 0013 §3).
fn str_is_boundary(bytes: &[u8], i: usize) -> bool {
    i == 0 || i == bytes.len() || (i < bytes.len() && (bytes[i] & 0xC0) != 0x80)
}
/// Decode the UTF-8 scalar at byte offset `pos` in the (well-formed-by-contract)
/// run `bytes`, returning `(code_point, next_pos)`. Returns `None` when `pos` is
/// a continuation byte (mid-character), the lead byte is invalid, or the sequence
/// is truncated — none of which occur in a valid `str` (design 0013 §4); the
/// caller (`char_at`/`char_count`) turns `None` into a P5 fault.
fn utf8_decode_at(bytes: &[u8], pos: usize) -> Option<(u32, usize)> {
    let b0 = *bytes.get(pos)?;
    let (extra, mut cp) = if b0 < 0x80 {
        return Some((b0 as u32, pos + 1));
    } else if b0 >> 5 == 0b110 {
        (1usize, (b0 & 0x1F) as u32)
    } else if b0 >> 4 == 0b1110 {
        (2usize, (b0 & 0x0F) as u32)
    } else if b0 >> 3 == 0b11110 {
        (3usize, (b0 & 0x07) as u32)
    } else {
        return None;
    };
    for k in 1..=extra {
        let b = *bytes.get(pos + k)?;
        if b & 0xC0 != 0x80 {
            return None;
        }
        cp = (cp << 6) | (b & 0x3F) as u32;
    }
    Some((cp, pos + 1 + extra))
}


/// UTF-8-encode a Unicode scalar value (design 0013 §3). Returns `None` for a
/// non-scalar `u32` (a surrogate `0xD800..=0xDFFF` or `> 0x10FFFF`), which is the
/// `is_scalar_value` predicate the `push` backstop enforces.
fn utf8_encode_scalar(c: u32) -> Option<Vec<u8>> {
    if c > 0x10FFFF || (0xD800..=0xDFFF).contains(&c) {
        return None;
    }
    let ch = char::from_u32(c)?;
    let mut buf = [0u8; 4];
    Some(ch.encode_utf8(&mut buf).as_bytes().to_vec())
}
