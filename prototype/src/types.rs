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
    Array(Box<Type>, ArrayLen),
    Slice(Box<Type>),
    SliceMut(Box<Type>),
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
            Type::Slice(_) | Type::SliceMut(_) | Type::Borrow(_) | Type::BorrowMut(_)
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
            Type::Array(e, len) => match len {
                ArrayLen::Lit(n) => format!("[{n}]{}", e.display()),
                ArrayLen::Named(n) => format!("[{n}]{}", e.display()),
                ArrayLen::Unknown => format!("[_]{}", e.display()),
            },
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
                    "fn({}){} -> {}",
                    ps.join(", "),
                    if f.alloc { " alloc" } else { "" },
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
    /// Struct fields (empty for an enum).
    pub fields: Vec<(String, Type)>,
    /// Enum variants (empty for a struct); each is (name, payload types, ok).
    pub variants: Vec<(String, Vec<Type>, bool)>,
}

/// Substitute type parameters (`Param(name)`) by `map` throughout `ty`.
pub fn subst(ty: &Type, map: &std::collections::HashMap<String, Type>) -> Type {
    match ty {
        Type::Param(n) => map.get(n).cloned().unwrap_or_else(|| ty.clone()),
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
        Type::FnPtr(_) => true,
        Type::Never | Type::Error => true,
        Type::BorrowMut(_) | Type::SliceMut(_) | Type::Box(_) | Type::BoxResult(_) => false,
        Type::Array(elem, _) => is_copy_rec(elem, env, stack),
        // An opaque type parameter is copy iff it carries the `copy` bound (§3.1).
        Type::Param(n) => env.param_copy(n).unwrap_or(false),
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
        Type::App(n, args) => {
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
        Type::App(_, _) => {
            if bears_box(ty, env) {
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
        Type::App(n, args) => {
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
        Type::Slice(_) | Type::SliceMut(_) | Type::Borrow(_) | Type::BorrowMut(_) => true,
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
        (Type::App(a, aa), Type::App(b, bb)) => {
            a == b && aa.len() == bb.len() && aa.iter().zip(bb).all(|(x, y)| assignable(x, y))
        }
        (Type::Array(a, la), Type::Array(b, lb)) => {
            assignable(a, b) && (la == lb || matches!(la, ArrayLen::Unknown) || matches!(lb, ArrayLen::Unknown))
        }
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
        }
        _ => false,
    }
}
