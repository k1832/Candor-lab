//! Semantic type representation, copyability, and type well-formedness
//! (design 0001 §1.3, §3.4, §6.1, §8.1). Stage 2 "value gear".
//!
//! The AST carries *syntactic* types (`ast::Ty`); this module carries the
//! *semantic* `Type` the checker reasons over. Resolution (ast::Ty -> Type)
//! lives in `resolve.rs`, which also builds the [`ItemEnv`] this module queries
//! for nominal (struct/enum) copyability and box-bearing facts.

use crate::ast::ParamMode;
use crate::span::Span;
use crate::token::ScalarTy;

/// Fixed length of an `[N]T` array. Named lengths (a const identifier) are kept
/// opaque and compared by spelling.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArrayLen {
    Lit(u64),
    Named(String),
    /// A length the checker could not evaluate (still a valid array type).
    Unknown,
}

/// A non-capturing function-pointer type: parameter modes + `alloc` effect are
/// part of the type (design 0001 §6.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FnPtrTy {
    pub params: Vec<(ParamMode, Type)>,
    pub alloc: bool,
    /// The `foreign` effect on the fn-pointer type (design 0011 §2).
    pub foreign: bool,
    pub ret: Box<Type>,
}

/// The semantic type lattice the checker manipulates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Scalar(ScalarTy),
    /// An unsuffixed integer literal: flexibly unifies with any integer scalar
    /// (design 0002 §0.1 — literal defaults to i64 if unconstrained).
    IntLit,
    /// A user struct or enum, resolved to exist.
    Named(String),
    /// An opaque generic type parameter (design 0007 §3): appears only while
    /// checking a generic body at its definition site. Never reaches a
    /// monomorphized program or the interpreter.
    Param(String),
    /// A generic nominal applied to concrete/parametric type arguments
    /// (`Pair[T]`, `List[i64]`). Def-site checking only; monomorphization lowers
    /// each reached application to a distinct concrete `Named` instance.
    App(String, Vec<Type>),
    /// An associated-type projection `Base::Assoc` (design 0009 §2.2). Appears
    /// only while checking a generic body at its definition site; monomorphization
    /// resolves it to the impl's concrete choice. Opaque like an unbounded `Param`
    /// (movable/droppable/borrowable, and nothing else) — no bounds (§2.3).
    Proj(String, String),
    Array(Box<Type>, ArrayLen),
    Slice(Box<Type>),
    SliceMut(Box<Type>),
    /// `str` (design 0013): an immutable, borrowed, allocation-free view of a run
    /// of bytes guaranteed to be well-formed UTF-8. Shaped exactly like `Slice(u8)`
    /// (a shared pointer-and-length borrow) but a DISTINCT typed refinement — no
    /// implicit coercion to/from `[u8]` (P2). A borrow-kind (§3.4-banned as a
    /// field); `portable` via its `u8` referent (0012).
    Str,
    RawPtr(Box<Type>),
    Box(Box<Type>),
    BoxResult(Box<Type>),
    Borrow(Box<Type>),
    BorrowMut(Box<Type>),
    FnPtr(FnPtrTy),
    /// The type of `return`/`break`/`continue`/`panic` — bottom, unifies down.
    Never,
    /// A poisoned type: suppresses cascading diagnostics.
    Error,
}

impl Type {
    pub fn unit() -> Type {
        Type::Scalar(ScalarTy::Unit)
    }
    pub fn bool() -> Type {
        Type::Scalar(ScalarTy::Bool)
    }
    pub fn usize() -> Type {
        Type::Scalar(ScalarTy::Usize)
    }

    /// A borrow-kind type (design 0001 §3.4/§3.1): a value that *is* a borrow.
    pub fn is_borrow_kind(&self) -> bool {
        matches!(
            self,
            Type::Slice(_)
                | Type::SliceMut(_)
                | Type::Str
                | Type::Borrow(_)
                | Type::BorrowMut(_)
        )
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Type::IntLit) || matches!(self, Type::Scalar(s) if s.is_integer())
    }

    /// Render for diagnostics (design P4).
    pub fn display(&self) -> String {
        match self {
            Type::Scalar(s) => scalar_name(*s).to_string(),
            Type::IntLit => "{integer}".to_string(),
            Type::Named(n) => n.clone(),
            Type::Param(n) => n.clone(),
            Type::App(n, args) => {
                let a: Vec<String> = args.iter().map(|t| t.display()).collect();
                format!("{n}[{}]", a.join(", "))
            }
            Type::Proj(b, a) => format!("{b}::{a}"),
            Type::Array(e, len) => match len {
                ArrayLen::Lit(n) => format!("[{n}]{}", e.display()),
                ArrayLen::Named(n) => format!("[{n}]{}", e.display()),
                ArrayLen::Unknown => format!("[_]{}", e.display()),
            },
            Type::Str => "str".to_string(),
            Type::Slice(e) => format!("slice {}", e.display()),
            Type::SliceMut(e) => format!("slice_mut {}", e.display()),
            Type::RawPtr(e) => format!("rawptr {}", e.display()),
            Type::Box(e) => format!("Box {}", e.display()),
            Type::BoxResult(e) => format!("BoxResult {}", e.display()),
            Type::Borrow(e) => format!("borrow {}", e.display()),
            Type::BorrowMut(e) => format!("borrow_mut {}", e.display()),
            Type::FnPtr(f) => {
                let ps: Vec<String> = f.params.iter().map(|(_, t)| t.display()).collect();
                format!(
                    "fn({}){}{} -> {}",
                    ps.join(", "),
                    if f.alloc { " alloc" } else { "" },
                    if f.foreign { " foreign" } else { "" },
                    f.ret.display()
                )
            }
            Type::Never => "never".to_string(),
            Type::Error => "<error>".to_string(),
        }
    }
}

pub fn scalar_name(s: ScalarTy) -> &'static str {
    match s {
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

// ---------------------------------------------------------------------------
// Resolved item metadata (the ItemEnv the copyability rules query)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct StructTy {
    pub copy: bool,
    pub has_drop: bool,
    /// The `drop` hook body is alloc-effecting (allocates, boxes, or drops a
    /// box-bearing local), so every scheduled drop of this type is allocator
    /// work — the type is *alloc-on-drop* and propagates the `alloc` effect to
    /// the enclosing function exactly like a `Box` field (design 0001 §1.5/§6.3;
    /// retest 2026-07-08). Computed by checking the hook as a synthetic
    /// `fn drop(self: write StructT) -> unit`.
    pub alloc_on_drop: bool,
    pub fields: Vec<(String, Type)>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct VariantTy {
    pub name: String,
    pub payload: Vec<Type>,
}

#[derive(Clone, Debug)]
pub struct EnumTy {
    pub copy: bool,
    pub variants: Vec<VariantTy>,
    /// The name of the single `ok`-marked variant, if this enum is
    /// result-shaped (design 0006 §2.4; spec 02 §2.2). Drives the `?` operator.
    pub ok_variant: Option<String>,
    pub span: Span,
}

/// Nominal lookups the structural rules need. Implemented by `resolve::Items`.
pub trait ItemEnv {
    fn lookup_struct(&self, name: &str) -> Option<&StructTy>;
    fn lookup_enum(&self, name: &str) -> Option<&EnumTy>;
    /// Is `name` a type parameter in scope, and does it carry the `copy` bound?
    /// `None` = not a type parameter; `Some(b)` = a parameter, `b` = has `copy`.
    /// (Design 0007 §3.1.) Default `None` keeps non-generic call sites unchanged.
    fn param_copy(&self, _name: &str) -> Option<bool> {
        None
    }
    /// Is `name` a type parameter in scope carrying the `portable` bound (design
    /// 0012 §2.2)? `None` = not a type parameter; `Some(b)` = a parameter, `b` =
    /// has `portable`. Default `None` keeps non-generic call sites unchanged.
    fn param_portable(&self, _name: &str) -> Option<bool> {
        None
    }
    /// The generic struct/enum decl for an `App` head, if generic (for field and
    /// payload substitution during def-site checking). Default `None`.
    fn lookup_generic(&self, _name: &str) -> Option<&GenericDecl> {
        None
    }
}

/// A generic struct or enum definition, with its type-parameter names, used to
/// substitute concrete/parametric arguments into field/payload types during
/// def-site checking and monomorphization (design 0007 §5).
#[derive(Clone, Debug)]
pub struct GenericDecl {
    pub params: Vec<String>,
    /// `true` for an enum, `false` for a struct.
    pub is_enum: bool,
    pub copy: bool,
    pub has_drop: bool,
    /// The `drop` hook body allocates, boxes, or drops a non-`copy`/box-bearing
    /// value, so every scheduled drop of any instance is allocator work — the
    /// generic aggregate is *alloc-on-drop* for all instantiations, fixed once at
    /// the definition (design 0007 §3.4, F5). Computed by checking the hook with an
    /// opaque `T`. (A box-bearing field already propagates through `bears_box`; this
    /// flag additionally catches a hook that allocates over a drop-inert `T`.)
    pub alloc_on_drop: bool,
    /// Struct fields (empty for an enum).
    pub fields: Vec<(String, Type)>,
    /// Enum variants (empty for a struct); each is (name, payload types, ok).
    pub variants: Vec<(String, Vec<Type>, bool)>,
}

/// Substitute type parameters (`Param(name)`) by `map` throughout `ty`.
pub fn subst(ty: &Type, map: &std::collections::HashMap<String, Type>) -> Type {
    match ty {
        Type::Param(n) => map.get(n).cloned().unwrap_or_else(|| ty.clone()),
        // A projection resolves when the map carries its fully-keyed entry
        // `"Base::Assoc"` (injected by method/generic-call resolution). Otherwise
        // it stays opaque, but its BASE is substituted when it maps to another
        // parameter — so `Self::Item` with `Self -> C` becomes `C::Item`, the
        // form a generic body checks against (design 0009 §2.2).
        Type::Proj(b, a) => {
            if let Some(t) = map.get(&format!("{b}::{a}")) {
                t.clone()
            } else if let Some(Type::Param(nb)) = map.get(b) {
                Type::Proj(nb.clone(), a.clone())
            } else {
                Type::Proj(b.clone(), a.clone())
            }
        }
        Type::App(n, args) => Type::App(n.clone(), args.iter().map(|a| subst(a, map)).collect()),
        Type::Array(e, l) => Type::Array(Box::new(subst(e, map)), l.clone()),
        Type::Slice(e) => Type::Slice(Box::new(subst(e, map))),
        Type::SliceMut(e) => Type::SliceMut(Box::new(subst(e, map))),
        Type::RawPtr(e) => Type::RawPtr(Box::new(subst(e, map))),
        Type::Box(e) => Type::Box(Box::new(subst(e, map))),
        Type::BoxResult(e) => Type::BoxResult(Box::new(subst(e, map))),
        Type::Borrow(e) => Type::Borrow(Box::new(subst(e, map))),
        Type::BorrowMut(e) => Type::BorrowMut(Box::new(subst(e, map))),
        Type::FnPtr(f) => Type::FnPtr(FnPtrTy {
            foreign: f.foreign,
            params: f.params.iter().map(|(m, t)| (*m, subst(t, map))).collect(),
            alloc: f.alloc,
            ret: Box::new(subst(&f.ret, map)),
        }),
        _ => ty.clone(),
    }
}

/// The substituted field/payload types reachable through a generic application's
/// head, for def-site field access and copy/drop analysis. Returns `None` if the
/// head is not a known generic.
pub fn app_fields(env: &dyn ItemEnv, name: &str, args: &[Type]) -> Option<Vec<(String, Type)>> {
    let g = env.lookup_generic(name)?;
    let map: std::collections::HashMap<String, Type> =
        g.params.iter().cloned().zip(args.iter().cloned()).collect();
    Some(g.fields.iter().map(|(n, t)| (n.clone(), subst(t, &map))).collect())
}

// ---------------------------------------------------------------------------
// Copyability (design 0001 §1.3)
// ---------------------------------------------------------------------------

/// Is `ty` a `copy` type? A structural, checker-computed property gated by the
/// author's `copy` marker on nominal types (design 0001 §1.3).
pub fn is_copy(ty: &Type, env: &dyn ItemEnv) -> bool {
    is_copy_rec(ty, env, &mut Vec::new())
}

fn is_copy_rec(ty: &Type, env: &dyn ItemEnv, stack: &mut Vec<String>) -> bool {
    match ty {
        Type::Scalar(_) | Type::IntLit => true,
        Type::RawPtr(_) => true,
        Type::Borrow(_) => true,
        Type::Slice(_) => true,
        Type::Str => true,
        Type::FnPtr(_) => true,
        Type::Never | Type::Error => true,
        Type::BorrowMut(_) | Type::SliceMut(_) | Type::Box(_) | Type::BoxResult(_) => false,
        Type::Array(elem, _) => is_copy_rec(elem, env, stack),
        // An opaque type parameter is copy iff it carries the `copy` bound (§3.1).
        Type::Param(n) => env.param_copy(n).unwrap_or(false),
        // A projection carries no bound (design 0009 §2.3): conservatively non-copy.
        Type::Proj(_, _) => false,
        // A generic application is copy iff the generic is a `copy` type and every
        // substituted field/payload is copy.
        Type::App(n, args) => {
            if let Some(g) = env.lookup_generic(n) {
                if !g.copy || g.has_drop {
                    return false;
                }
                let map: std::collections::HashMap<String, Type> =
                    g.params.iter().cloned().zip(args.iter().cloned()).collect();
                let field_tys: Vec<Type> = if g.is_enum {
                    g.variants.iter().flat_map(|(_, p, _)| p.iter().cloned()).collect()
                } else {
                    g.fields.iter().map(|(_, t)| t.clone()).collect()
                };
                field_tys.iter().all(|t| is_copy_rec(&subst(t, &map), env, stack))
            } else {
                false
            }
        }
        Type::Named(n) => {
            if stack.iter().any(|s| s == n) {
                return true; // cycle guard (only reachable through non-copy Box)
            }
            stack.push(n.clone());
            let r = if let Some(s) = env.lookup_struct(n) {
                s.copy && !s.has_drop && s.fields.iter().all(|(_, t)| is_copy_rec(t, env, stack))
            } else if let Some(e) = env.lookup_enum(n) {
                e.copy
                    && e.variants
                        .iter()
                        .all(|v| v.payload.iter().all(|t| is_copy_rec(t, env, stack)))
            } else {
                false
            };
            stack.pop();
            r
        }
    }
}

/// Does dropping a value of `ty` do observable work (design 0001 §1.5/§1.6)?
/// True iff the type has a `drop` hook, or transitively contains a drop-hooked
/// type or a `Box` (whose drop frees through its stored handle). Scalars, copy
/// aggregates, `rawptr`, borrows/slices, and aggregates of only such fields are
/// drop-inert. This is the dual of copyability the scope-exit path-independence
/// rule keys on: at a needs-drop place's drop point the interpreter must not
/// have to consult a runtime flag (finding 2026-07-07; §1.6/§7.4).
pub fn needs_drop(ty: &Type, env: &dyn ItemEnv) -> bool {
    needs_drop_rec(ty, env, &mut Vec::new())
}

fn needs_drop_rec(ty: &Type, env: &dyn ItemEnv, stack: &mut Vec<String>) -> bool {
    match ty {
        Type::Box(_) | Type::BoxResult(_) => true,
        Type::Array(elem, _) => needs_drop_rec(elem, env, stack),
        // Conservatively needs-drop unless proven `copy` (§3.3).
        Type::Param(n) => !env.param_copy(n).unwrap_or(false),
        // A projection is unbounded (design 0009 §2.3): conservatively needs-drop.
        Type::Proj(_, _) => true,
        Type::App(n, args) => {
            // Compiler-known std `Vec[T]`/`Map[V]` always free their heap buffer
            // on drop (alloc-on-drop), independent of the element/value type.
            if n == "Vec" || n == "Map" {
                return true;
            }
            if let Some(g) = env.lookup_generic(n) {
                if g.has_drop {
                    return true;
                }
                let map: std::collections::HashMap<String, Type> =
                    g.params.iter().cloned().zip(args.iter().cloned()).collect();
                let tys: Vec<Type> = if g.is_enum {
                    g.variants.iter().flat_map(|(_, p, _)| p.iter().cloned()).collect()
                } else {
                    g.fields.iter().map(|(_, t)| t.clone()).collect()
                };
                tys.iter().any(|t| needs_drop_rec(&subst(t, &map), env, stack))
            } else {
                false
            }
        }
        Type::Named(n) => {
            if stack.iter().any(|s| s == n) {
                // A cycle is only reachable through a `Box`, already accounted
                // for (it returned `true` above); this arm terminates the walk.
                return false;
            }
            stack.push(n.clone());
            let r = if let Some(s) = env.lookup_struct(n) {
                s.has_drop || s.fields.iter().any(|(_, t)| needs_drop_rec(t, env, stack))
            } else if let Some(e) = env.lookup_enum(n) {
                e.variants
                    .iter()
                    .any(|v| v.payload.iter().any(|t| needs_drop_rec(t, env, stack)))
            } else {
                false
            };
            stack.pop();
            r
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Portability (design 0012 §2.2) — the spawn-crossing structural predicate
// ---------------------------------------------------------------------------

/// Is `ty` **`portable`** (design 0012 §2.2)? A type is portable iff it
/// transitively contains **no `rawptr`** and **no borrow** — owned value data
/// only. The walk descends through EVERY field, including the fields of `copy`
/// aggregates (`copy` is not a stopping condition), because a `copy` referent can
/// be copied out from behind a shared borrow, laundering whatever it hides
/// (§2.1/§2.3). `Box T`/`BoxResult T` are portable when `T` is (a unique-owning
/// pointer, categorically unlike `rawptr`). A **function pointer is a portable
/// leaf**: the walk does NOT descend into its signature — a fn-pointer carries no
/// data pointer and cannot capture, and any `rawptr` in its signature is only
/// produced by calling it (gated by `unsafe`), so descending would wrongly reject
/// safe vtable sharing (§2.2, re-review F1).
pub fn is_portable(ty: &Type, env: &dyn ItemEnv) -> bool {
    is_portable_rec(ty, env, &mut Vec::new())
}

fn is_portable_rec(ty: &Type, env: &dyn ItemEnv, stack: &mut Vec<String>) -> bool {
    match ty {
        Type::Scalar(_) | Type::IntLit => true,
        // The two non-portable leaves: a raw pointer, and any borrow/slice.
        Type::RawPtr(_) => false,
        Type::Borrow(_) | Type::BorrowMut(_) | Type::Slice(_) | Type::SliceMut(_) => false,
        // `str`'s referent is a run of `u8` (portable), so a `str` shares into a
        // scoped task exactly like any `[u8]` (design 0013 §5 / 0012 borrow branch).
        Type::Str => true,
        // A fn-pointer is a portable LEAF: do not descend into the signature.
        Type::FnPtr(_) => true,
        Type::Never | Type::Error => true,
        Type::Array(elem, _) => is_portable_rec(elem, env, stack),
        Type::Box(e) | Type::BoxResult(e) => is_portable_rec(e, env, stack),
        // An opaque type parameter is portable iff it carries the `portable` bound.
        Type::Param(n) => env.param_portable(n).unwrap_or(false),
        // A projection carries no bound (design 0009 §2.3): conservatively not portable.
        Type::Proj(_, _) => false,
        Type::App(n, args) => {
            if let Some(g) = env.lookup_generic(n) {
                let map: std::collections::HashMap<String, Type> =
                    g.params.iter().cloned().zip(args.iter().cloned()).collect();
                let field_tys: Vec<Type> = if g.is_enum {
                    g.variants.iter().flat_map(|(_, p, _)| p.iter().cloned()).collect()
                } else {
                    g.fields.iter().map(|(_, t)| t.clone()).collect()
                };
                field_tys.iter().all(|t| is_portable_rec(&subst(t, &map), env, stack))
            } else {
                false
            }
        }
        Type::Named(n) => {
            if stack.iter().any(|s| s == n) {
                return true; // cycle guard (only reachable through a portable Box)
            }
            stack.push(n.clone());
            // Descend through EVERY field/payload — `copy` is NOT a stopping
            // condition (§2.1: the laundering channel is copy-out of a copy
            // aggregate hiding a rawptr).
            let r = if let Some(st) = env.lookup_struct(n) {
                st.fields.iter().all(|(_, t)| is_portable_rec(t, env, stack))
            } else if let Some(e) = env.lookup_enum(n) {
                e.variants.iter().all(|v| v.payload.iter().all(|t| is_portable_rec(t, env, stack)))
            } else {
                false
            };
            stack.pop();
            r
        }
    }
}

/// The first non-portable leaf type reached inside `ty` (design 0012 §2.1: the
/// diagnostic names the offending `rawptr`/borrow, P4). `None` iff `ty` is
/// portable. Mirrors [`is_portable`]'s walk exactly.
pub fn non_portable_witness(ty: &Type, env: &dyn ItemEnv) -> Option<Type> {
    non_portable_witness_rec(ty, env, &mut Vec::new())
}

fn non_portable_witness_rec(ty: &Type, env: &dyn ItemEnv, stack: &mut Vec<String>) -> Option<Type> {
    match ty {
        Type::Scalar(_) | Type::IntLit | Type::Str | Type::FnPtr(_) | Type::Never | Type::Error => None,
        Type::RawPtr(_)
        | Type::Borrow(_)
        | Type::BorrowMut(_)
        | Type::Slice(_)
        | Type::SliceMut(_) => Some(ty.clone()),
        Type::Array(elem, _) => non_portable_witness_rec(elem, env, stack),
        Type::Box(e) | Type::BoxResult(e) => non_portable_witness_rec(e, env, stack),
        Type::Param(n) => {
            if env.param_portable(n).unwrap_or(false) { None } else { Some(ty.clone()) }
        }
        Type::Proj(_, _) => Some(ty.clone()),
        Type::App(n, args) => {
            if let Some(g) = env.lookup_generic(n) {
                let map: std::collections::HashMap<String, Type> =
                    g.params.iter().cloned().zip(args.iter().cloned()).collect();
                let field_tys: Vec<Type> = if g.is_enum {
                    g.variants.iter().flat_map(|(_, p, _)| p.iter().cloned()).collect()
                } else {
                    g.fields.iter().map(|(_, t)| t.clone()).collect()
                };
                field_tys.iter().find_map(|t| non_portable_witness_rec(&subst(t, &map), env, stack))
            } else {
                Some(ty.clone())
            }
        }
        Type::Named(n) => {
            if stack.iter().any(|s| s == n) {
                return None;
            }
            stack.push(n.clone());
            let r = if let Some(st) = env.lookup_struct(n) {
                st.fields.iter().find_map(|(_, t)| non_portable_witness_rec(t, env, stack))
            } else if let Some(e) = env.lookup_enum(n) {
                e.variants.iter().find_map(|v| v.payload.iter().find_map(|t| non_portable_witness_rec(t, env, stack)))
            } else {
                Some(ty.clone())
            };
            stack.pop();
            r
        }
    }
}

/// The field-paths within `ty` that reach a `Box` — the sub-places whose drop
/// calls `free`, so a scope-exit/reassignment drop of any of them is allocator
/// work (design 0001 §6.2/§6.3; finding 4 of 2026-07-07). Returns `[[]]` for a
/// bare `Box T`/`BoxResult T`. Aggregates only field-granular tracking follows
/// (structs); an array element or enum payload is not a named place, so a
/// box reached through one yields the path to that array/enum aggregate — the
/// whole-aggregate drop is what frees, checked at that granularity.
pub fn box_subpaths(ty: &Type, env: &dyn ItemEnv) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    box_subpaths_rec(ty, env, &mut Vec::new(), &mut Vec::new(), &mut out);
    out
}

fn box_subpaths_rec(
    ty: &Type,
    env: &dyn ItemEnv,
    prefix: &mut Vec<String>,
    stack: &mut Vec<String>,
    out: &mut Vec<Vec<String>>,
) {
    match ty {
        Type::Box(_) | Type::BoxResult(_) => out.push(prefix.clone()),
        Type::Array(elem, _) => {
            // No index-granular place tracking: if the element bears a box, the
            // whole-array drop at `prefix` is the freeing point.
            if bears_box(elem, env) {
                out.push(prefix.clone());
            }
        }
        Type::Named(n) => {
            if stack.iter().any(|s| s == n) {
                return;
            }
            stack.push(n.clone());
            if let Some(s) = env.lookup_struct(n) {
                // An alloc-on-drop hook makes the WHOLE-struct drop allocator
                // work (retest 2026-07-08): the drop point is this aggregate
                // place, so record it and stop — its fields' drops run under it.
                if s.alloc_on_drop {
                    out.push(prefix.clone());
                } else {
                    for (fname, fty) in &s.fields {
                        prefix.push(fname.clone());
                        box_subpaths_rec(fty, env, prefix, stack, out);
                        prefix.pop();
                    }
                }
            } else if let Some(e) = env.lookup_enum(n) {
                // Enum payloads are not named field places; a box in any variant
                // frees at the whole-enum drop point `prefix`.
                if e
                    .variants
                    .iter()
                    .any(|v| v.payload.iter().any(|t| bears_box(t, env)))
                {
                    out.push(prefix.clone());
                }
            }
            stack.pop();
        }
        // §3.4: an opaque owner we cannot prove drop-inert frees at its own place.
        Type::Param(n) => {
            if !env.param_copy(n).unwrap_or(false) {
                out.push(prefix.clone());
            }
        }
        // A projection is opaque and unbounded (design 0009 §2.3): frees at its
        // own place, exactly like a non-`copy` parameter.
        Type::Proj(_, _) => out.push(prefix.clone()),
        Type::App(n, _) => {
            // A `Vec[T]` (compiler-known, alloc-on-drop) frees at its own place;
            // and an alloc-on-drop generic aggregate frees at its own place
            // regardless of a bare `Box` field (design 0007 §3.4, F5).
            if n == "Vec"
                || n == "Map"
                || env.lookup_generic(n).map(|g| g.alloc_on_drop).unwrap_or(false)
                || bears_box(ty, env)
            {
                out.push(prefix.clone());
            }
        }
        _ => {}
    }
}

/// Does a value of `ty` own one or more `Box`es (so `clone` allocates, §1.4/§6.3)?
pub fn bears_box(ty: &Type, env: &dyn ItemEnv) -> bool {
    bears_box_rec(ty, env, &mut Vec::new())
}

fn bears_box_rec(ty: &Type, env: &dyn ItemEnv, stack: &mut Vec<String>) -> bool {
    match ty {
        Type::Box(_) | Type::BoxResult(_) => true,
        Type::Array(elem, _) => bears_box_rec(elem, env, stack),
        // Conservatively a `T` may be box-bearing unless proven `copy` (§3.4).
        Type::Param(n) => !env.param_copy(n).unwrap_or(false),
        // A projection is unbounded (design 0009 §2.3): conservatively box-bearing.
        Type::Proj(_, _) => true,
        Type::App(n, args) => {
            // A `Vec[T]`/`Map[V]` owns a heap buffer whose drop frees; treat it
            // as box-bearing so a dropped temporary is accounted allocator work.
            if n == "Vec" || n == "Map" {
                return true;
            }
            if let Some(g) = env.lookup_generic(n) {
                let map: std::collections::HashMap<String, Type> =
                    g.params.iter().cloned().zip(args.iter().cloned()).collect();
                let tys: Vec<Type> = if g.is_enum {
                    g.variants.iter().flat_map(|(_, p, _)| p.iter().cloned()).collect()
                } else {
                    g.fields.iter().map(|(_, t)| t.clone()).collect()
                };
                tys.iter().any(|t| bears_box_rec(&subst(t, &map), env, stack))
            } else {
                false
            }
        }
        Type::Named(n) => {
            if stack.iter().any(|s| s == n) {
                return false;
            }
            stack.push(n.clone());
            let r = if let Some(s) = env.lookup_struct(n) {
                s.fields.iter().any(|(_, t)| bears_box_rec(t, env, stack))
            } else if let Some(e) = env.lookup_enum(n) {
                e.variants
                    .iter()
                    .any(|v| v.payload.iter().any(|t| bears_box_rec(t, env, stack)))
            } else {
                false
            };
            stack.pop();
            r
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Well-formedness helpers
// ---------------------------------------------------------------------------

/// Does this *field* type store a borrow (design 0001 §3.4 — banned)? Slices are
/// borrows too. `Box`/`rawptr` indirection is fine (they are not borrows).
pub fn field_stores_borrow(ty: &Type) -> bool {
    match ty {
        Type::Slice(_) | Type::SliceMut(_) | Type::Str | Type::Borrow(_) | Type::BorrowMut(_) => true,
        Type::Array(elem, _) => field_stores_borrow(elem),
        _ => false,
    }
}

/// Assignability of `from` (a value's type) into a slot expecting `to`.
/// Handles integer-literal flexibility and fn-pointer effect subtyping
/// (design 0001 §6.1: a non-`alloc` fn is assignable to an `alloc` slot, never
/// the reverse).
pub fn assignable(from: &Type, to: &Type) -> bool {
    match (from, to) {
        (Type::Error, _) | (_, Type::Error) => true,
        (Type::Never, _) => true,
        (Type::IntLit, t) | (t, Type::IntLit) => t.is_integer(),
        (Type::Scalar(a), Type::Scalar(b)) => a == b,
        (Type::Named(a), Type::Named(b)) => a == b,
        (Type::Param(a), Type::Param(b)) => a == b,
        (Type::Proj(a, b), Type::Proj(c, d)) => a == c && b == d,
        (Type::App(a, aa), Type::App(b, bb)) => {
            a == b && aa.len() == bb.len() && aa.iter().zip(bb).all(|(x, y)| assignable(x, y))
        }
        (Type::Array(a, la), Type::Array(b, lb)) => {
            assignable(a, b) && (la == lb || matches!(la, ArrayLen::Unknown) || matches!(lb, ArrayLen::Unknown))
        }
        (Type::Str, Type::Str) => true,
        (Type::Slice(a), Type::Slice(b)) => assignable(a, b),
        (Type::SliceMut(a), Type::SliceMut(b)) => assignable(a, b),
        (Type::RawPtr(a), Type::RawPtr(b)) => assignable(a, b),
        (Type::Box(a), Type::Box(b)) => assignable(a, b),
        (Type::BoxResult(a), Type::BoxResult(b)) => assignable(a, b),
        (Type::Borrow(a), Type::Borrow(b)) => assignable(a, b),
        (Type::BorrowMut(a), Type::BorrowMut(b)) => assignable(a, b),
        (Type::FnPtr(a), Type::FnPtr(b)) => {
            a.params.len() == b.params.len()
                && a.params
                    .iter()
                    .zip(&b.params)
                    .all(|((am, at), (bm, bt))| am == bm && at == bt)
                && a.ret == b.ret
                // effect: source alloc requires target alloc (upper bound).
                && (!a.alloc || b.alloc)
                // effect: source foreign requires target foreign (design 0011 §2).
                && (!a.foreign || b.foreign)
        }
        _ => false,
    }
}
