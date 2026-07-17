//! The full abstract syntax tree for design 0001 (Bet 5 prototype).
//!
//! Every node carries a `Span`. The AST is deliberately a faithful shape of the
//! throwaway grammar: it is built without any symbol table (NN#13), so e.g.
//! `Name::Variant` is always an `EnumCtor`/variant pattern purely by position,
//! and an intrinsic call like `box(a, v)` is an ordinary `Call` node whose
//! callee happens to be a builtin name (resolved later by the checker).

use crate::span::Span;
use crate::token::ScalarTy;

// ---------------------------------------------------------------------------
// Program & items
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Item {
    Struct(StructDecl),
    Enum(EnumDecl),
    Fn(FnDecl),
    Static(StaticDecl),
    /// A generic interface declaration (design 0007 §1.2, §6). A named set of
    /// method signatures; may itself be generic (`interface From[E]`).
    Interface(InterfaceDecl),
    /// An `impl I[args] for Type { .. }` block attaching an interface's methods
    /// to a type (design 0007 §2.3).
    Impl(ImplDecl),
    /// An `extern "C" { .. }` block of foreign declarations (design 0011 §1).
    /// Well-formed only inside a `boundary` file; `boundary_file` records whether
    /// its file carried the marker (the E1101 placement check reads it).
    Extern(ExternBlock),
    /// An `export "C" fn sym(..) -> R = candor_fn;` reverse-direction declaration
    /// (design 0011 §1.5). Also boundary-file-only.
    Export(ExportDecl),
}

// ---------------------------------------------------------------------------
// Foreign boundary (design 0011)
// ---------------------------------------------------------------------------

/// An `extern "<abi>" { .. }` block (design 0011 §1). `abi` is the ABI string
/// (only `"C"` is defined this edition). `boundary_file` is set by the parser to
/// the file's `boundary`-preamble status, so the placement rule (E1101) survives
/// the module merge, which flattens files into one program.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ExternBlock {
    pub abi: String,
    pub boundary_file: bool,
    pub fns: Vec<ExternFn>,
    pub span: Span,
}

/// One foreign function signature inside an `extern` block. It is implicitly
/// `foreign` (the ground source of the effect, §2) and may carry a `trust`
/// clause discharging its preconditions (§3).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ExternFn {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: Option<RetTy>,
    pub trust: Option<TrustDecl>,
    pub span: Span,
}

/// A `trust "justification" { pred(args), .. }` clause (design 0011 §3): a
/// mandatory non-empty justification string plus zero or more predicates drawn
/// from the closed vocabulary. Every predicate is `assumed-proven` — recorded
/// and enumerated, never evaluated.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TrustDecl {
    pub justification: String,
    pub predicates: Vec<TrustPred>,
    pub span: Span,
}

/// One trust predicate: a name from the closed set and its identifier arguments.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TrustPred {
    pub name: String,
    pub args: Vec<String>,
    pub span: Span,
}

/// An `export "<abi>" fn <symbol>(params) -> ret = <candor_fn>;` declaration
/// (design 0011 §1.5): the reverse direction, binding an existing `pub` Candor
/// function to a stable C symbol under a C-mapped signature.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ExportDecl {
    pub abi: String,
    pub boundary_file: bool,
    pub symbol: String,
    pub params: Vec<Param>,
    pub ret: Option<RetTy>,
    pub candor_fn: String,
    pub span: Span,
}

/// A declared type parameter with its bounds (design 0007 §1.2, §6.4). Bounds are
/// interface names and the one built-in structural bound `copy`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TypeParam {
    pub name: String,
    /// Bound names (interface names or the literal `copy`).
    pub bounds: Vec<String>,
    pub span: Span,
}

/// One method *signature* inside an `interface` (design 0007 §1.2, §4.1). No body.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MethodSig {
    pub name: String,
    /// Whether the method takes a `self` receiver (design 0007 §3.5). A method
    /// without `self` is an associated function, e.g. `From::from` (§7.1).
    pub has_self: bool,
    /// The `self` receiver mode when `has_self` (`read`/`write`/`take`).
    pub self_mode: ParamMode,
    /// Non-self parameters.
    pub params: Vec<Param>,
    pub alloc: bool,
    pub ret: Option<RetTy>,
    pub span: Span,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct InterfaceDecl {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    /// The one associated type member (`type Item;`), if declared (design 0009
    /// §2.1). At most one per interface (the §2.3 refusals kept intact).
    pub assoc_type: Option<String>,
    pub methods: Vec<MethodSig>,
    pub span: Span,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ImplDecl {
    /// Type parameters of a *generic* impl (`impl[T] I for List[T]`). Empty for a
    /// concrete impl. (Generic impls are deferred in stage 1; see generics.rs.)
    pub type_params: Vec<TypeParam>,
    pub iface: String,
    /// The interface's type arguments (`From[E1]` -> `[E1]`).
    pub iface_args: Vec<Ty>,
    /// The implementing type (`impl I for Type`).
    pub target: Ty,
    /// The associated-type binding (`type Item = T;`), if the interface declares
    /// an associated type (design 0009 §2.1). `(member name, bound type)`.
    pub assoc_binding: Option<(String, Ty)>,
    pub methods: Vec<FnDecl>,
    /// The module this impl block was declared in (set by the module merge, design
    /// 0008), for the module-granularity orphan check. `None` = single-file.
    pub home: Option<String>,
    pub span: Span,
}

/// A `use` import declaration (design 0008 §3). Parsed by the real front-end
/// only; the module layer (`crate::modules`) resolves it. `segments` is the
/// module path (`use a::b` -> `["a","b"]`); `names` is `Some` for a group import
/// (`use a::b::{x, y}` -> `Some(["x","y"])`) and `None` for a namespace import
/// (`use a::b;`, binding the module `b` as an alias).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UseDecl {
    pub segments: Vec<String>,
    pub names: Option<Vec<String>>,
    pub span: Span,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StructDecl {
    pub copy: bool,
    pub name: String,
    /// Generic type parameters declared in brackets after the name (design 0007).
    pub type_params: Vec<TypeParam>,
    pub fields: Vec<Field>,
    /// Optional `drop(write self) { ... }` hook (design 0001 §1.5).
    pub drop_hook: Option<Block>,
    pub span: Span,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Field {
    pub name: String,
    pub ty: Ty,
    pub span: Span,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EnumDecl {
    pub copy: bool,
    pub name: String,
    /// Generic type parameters (design 0007).
    pub type_params: Vec<TypeParam>,
    pub variants: Vec<Variant>,
    pub span: Span,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Variant {
    pub name: String,
    /// Zero or more payload types (design 0001 §8.2).
    pub payload: Vec<Ty>,
    /// `ok`-marked success variant of a result-shaped enum (design 0006 §2.4;
    /// spec 02 §2.2). Only the real (`.cnr`) front-end ever sets this; the
    /// throwaway front-end always leaves it `false`.
    pub ok: bool,
    pub span: Span,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FnDecl {
    pub name: String,
    /// Generic type parameters declared in the bracket after the name, mixed with
    /// region variables (design 0007 §6.1.1). Empty for a non-generic function.
    pub type_params: Vec<TypeParam>,
    /// Region variables declared in brackets after the name (design 0001 §3.3).
    pub regions: Vec<String>,
    pub params: Vec<Param>,
    /// The `alloc` effect marker (design 0001 §3.2).
    pub alloc: bool,
    /// The `foreign` effect marker (design 0011 §2): the function reaches
    /// undischarged foreign trust. An upper bound like `alloc`.
    pub foreign: bool,
    /// Set by the real front-end when this function's file carried the `boundary`
    /// preamble (design 0008 §4). Only a boundary-module wrapper may *discharge*
    /// the `foreign` effect (design 0011 §2 rule 4).
    pub boundary: bool,
    pub requires: Vec<Expr>,
    pub ensures: Vec<Expr>,
    /// Return spec; `None` means the unit return (`-> unit` omitted).
    pub ret: Option<RetTy>,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StaticDecl {
    pub name: String,
    pub ty: Ty,
    pub value: Expr,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Signature pieces
// ---------------------------------------------------------------------------

/// Parameter-passing mode (design 0001 §3.1). Omitted spelling parses to `Take`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ParamMode {
    Take,
    Read,
    Write,
    Out,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Param {
    pub name: String,
    pub mode: ParamMode,
    /// Region tag on a borrow parameter, e.g. the `r` in `read[r] Slice`.
    pub region: Option<String>,
    pub ty: Ty,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BorrowKind {
    Shared,    // `read`
    Exclusive, // `write`
}

/// A function return type, possibly a borrow return with a region (design §3.3).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RetTy {
    pub borrow: Option<BorrowKind>,
    pub region: Option<String>,
    pub ty: Ty,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Ty {
    pub kind: TyKind,
    pub span: Span,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum TyKind {
    Scalar(ScalarTy),
    /// A user struct/enum name (unresolved — no symbol table). Also a bare type
    /// parameter reference (`T`) inside a generic body; the checker distinguishes.
    Named(String),
    /// A generic type application `Name[arg, ...]` in type position (design 0007
    /// §6.1.1 use-rule): `List[i64]`, `Pair[T]`, `From[E1]`.
    App { name: String, args: Vec<Ty> },
    /// An associated-type projection `Base::Assoc` in type position (design 0009
    /// §2.2): `T::Item`, `Self::Item`, `C::Item`. `base` is a type-parameter name
    /// (or `Self`); `assoc` the member name. Single-valued by coherence (§2.2).
    Proj { base: String, assoc: String },
    /// `[N]T` fixed array; `size` is a const expression (int literal or name).
    Array { size: Box<Expr>, elem: Box<Ty> },
    Slice(Box<Ty>),
    SliceMut(Box<Ty>),
    RawPtr(Box<Ty>),
    Box(Box<Ty>),
    BoxResult(Box<Ty>),
    Borrow(Box<Ty>),
    BorrowMut(Box<Ty>),
    FnPtr(FnPtrTy),
}

/// Non-capturing function-pointer type (design 0001 §6.1). Its type includes
/// parameter modes *and* the `alloc` effect marker.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FnPtrTy {
    pub params: Vec<FnPtrParam>,
    pub alloc: bool,
    /// The `foreign` effect on a function-pointer type (design 0011 §2): an
    /// indirect call through it is a foreign call.
    pub foreign: bool,
    pub ret: Box<Ty>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FnPtrParam {
    /// Optional decorative name (`ctx: rawptr u8`).
    pub name: Option<String>,
    pub mode: ParamMode,
    pub region: Option<String>,
    pub ty: Ty,
}

// ---------------------------------------------------------------------------
// Statements & blocks
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum StmtKind {
    Let {
        mutable: bool,
        name: String,
        ty: Option<Ty>,
        init: Option<Expr>,
    },
    Assign {
        target: Expr,
        value: Expr,
    },
    /// An expression used as a statement (calls, block-like forms, jumps).
    Expr(Expr),
}

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum UnOp {
    Neg,
    Not,
    /// `~a` — prefix bitwise-not (design 0006 §2.4). Integer operand only.
    BitNot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PrefixOp {
    Deref,
    Read,
    Write,
    Clone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    // Bitwise / shift operators (design 0006 §2.4). Integer operands only.
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FieldInit {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    /// An optional `if EXPR` guard between the pattern and `=>` (design 0001
    /// §8.2, extended). The arm fires only when the pattern matches AND the guard
    /// evaluates true; a guarded arm does not contribute to exhaustiveness and
    /// does not shadow later arms (its guard may fail at runtime).
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: Span,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ExprKind {
    IntLit { value: u64, suffix: Option<ScalarTy> },
    /// A negative-literal fold `-<int>` (design 0006 §2.4; spec 01 §3.4). The
    /// stored `value` is the magnitude; the node denotes `-(value)`. Produced
    /// only by the real front-end, which range-checks it against its type.
    NegIntLit { value: u64, suffix: Option<ScalarTy> },
    /// An `f64` floating-point literal (design 0016). Carries the parsed value's
    /// IEEE-754 bit pattern for exact, lossless round-tripping; the literal is
    /// always the concrete type `f64` (a `.` or exponent makes it a float).
    FloatLit { bits: u64, ty: crate::token::ScalarTy },
    StrLit(String),
    /// `b"..."` — a byte-string literal (design 0013): denotes a `[u8]` view over
    /// the raw UTF-8 bytes of the source content. The genuinely-binary escape
    /// hatch, now that `"..."` (`StrLit`) is `str`.
    BytesLit(String),
    BoolLit(bool),
    Ident(String),

    Unary { op: UnOp, expr: Box<Expr> },
    Prefix { op: PrefixOp, expr: Box<Expr> },
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr> },

    Call { callee: Box<Expr>, args: Vec<Expr> },
    /// `out place` — the mandatory call-site marker for an out-mode argument
    /// (design 0001 §3.1; grammar 0002). Only valid as a call argument.
    OutArg(Box<Expr>),
    /// `base.field` — a struct field access, or the *callee* of a method call
    /// `recv.m(args)` (which parses as `Call { callee: Field }`). For a method
    /// call, `iface` records the interface whose method the checker resolved
    /// (design 0007 §2.3): monomorphization stamps it so dispatch runs exactly
    /// that interface's impl when one type impls two interfaces that share a
    /// method name. `None` for a plain field access, and for calls the checker
    /// did not stamp (a non-generic program), where dispatch falls back to the
    /// table's coherent first-match — the same impl the checker chose.
    Field { base: Box<Expr>, field: String, #[serde(default)] iface: Option<String> },
    Index { base: Box<Expr>, index: Box<Expr> },

    /// `conv T (e)` — the only integer conversion form (design 0001 §8.1).
    Conv { ty: Ty, expr: Box<Expr> },

    /// `bitcast T (e)` — same-width bit reinterpretation between a float and an
    /// integer scalar (design 0016 §10). Unlike `conv` (which converts the numeric
    /// VALUE), bitcast reinterprets the identical bits as `T`; it never changes the
    /// bits, never faults, and is regime-independent.
    Bitcast { ty: Ty, expr: Box<Expr> },

    ArrayLit(Vec<Expr>),
    ArrayRepeat { value: Box<Expr>, size: Box<Expr> },
    StructLit { name: String, fields: Vec<FieldInit> },
    /// `Name::Variant` or `Name::Variant(args)` (design 0001 §8.2).
    EnumCtor {
        enum_name: String,
        variant: String,
        args: Vec<Expr>,
    },

    // bracketed-type-arg intrinsics (design 0001 §4.2)
    CastPtr { ty: Ty, arg: Box<Expr> },
    AddrToPtr { ty: Ty, arg: Box<Expr> },
    PtrNull { ty: Ty },
    Offsetof { ty: Ty, field: String },
    /// `field_ptr(p, f)` — safe typed field projection (design 0004): the
    /// address of field `f` of the struct `p` points at. `p` is an expression,
    /// `f` a field selector in field position (no symbol table to parse).
    FieldPtr { ptr: Box<Expr>, field: String },
    Sizeof(Ty),
    Alignof(Ty),

    // block-like / control expressions
    Block(Block),
    If {
        cond: Box<Expr>,
        then_blk: Block,
        else_blk: Option<Box<Expr>>,
    },
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    Loop(Block),
    While { cond: Box<Expr>, body: Block },
    Unsafe { justification: String, body: Block },
    Wrapping(Block),
    Saturating(Block),

    // jump expressions
    Return(Option<Box<Expr>>),
    Break,
    Continue,

    // contracts (design 0001 §7.3)
    Assert(Box<Expr>),
    Panic(Box<Expr>),
    /// The `result` keyword; valid only inside `ensures` (checker-restricted).
    Result,

    Paren(Box<Expr>),

    /// `expr?` — the postfix propagation operator (design 0006 §2.4; spec 02
    /// §6.5). On a result-shaped enum, unwraps the `ok`-marked variant's
    /// payload or early-returns the whole value. Real front-end only.
    Try(Box<Expr>),

    /// `name::[T, ...]` — a generic function named as a *value* with explicit
    /// type arguments (design 0007 §6.2.1). Only in value positions.
    GenericVal { name: String, ty_args: Vec<Ty> },

    /// `for PATTERN in OPERAND { BODY }` (design 0009 §4; spec 02 §8.3).
    /// **Surface-only:** produced *solely* by the formatter's surface parse
    /// (`preserve_for`); the normal real-syntax pipeline desugars `for` into a
    /// `loop`+`match` at parse time (design 0009 §4.2), so the checker /
    /// interpreter / MIR never see this node. It exists only so the blessed
    /// formatter can reproduce the canonical `for` spelling (NN#11).
    For { pattern: Pattern, operand: Box<Expr>, body: Block, by_ref: bool },

    /// `scope { ... }` — a structured-concurrency region (design 0012 §1.1).
    /// Block-like in statement-leading position; every task spawned inside joins
    /// at the closing brace. Real (`.cnr`) front-end only.
    Scope(Block),
    /// `spawn CALLEE(ARGS);` — start one task inside a `scope` (design 0012 §1.1,
    /// §5). The boxed expression is always an `ExprKind::Call`. Under the Stage-2
    /// sequential oracle (design 0012 §6) the task runs to completion at the spawn
    /// point, so execution is a plain call; only checking treats it specially.
    Spawn(Box<Expr>),
}

// ---------------------------------------------------------------------------
// Patterns (design 0001 §8.2 / §8.2.1)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Pattern {
    pub kind: PatKind,
    pub span: Span,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum PatKind {
    /// `_` — binds nothing (grammar-level; exhaustiveness enforced by checker).
    Wildcard,
    /// A plain binding name.
    Binding(String),
    /// `Name::Variant` or `Name::Variant(sub, ...)`.
    Variant {
        enum_name: String,
        variant: String,
        sub: Vec<Pattern>,
    },
    /// An integer-literal pattern (`0x41`, `42u8`, `-5`): matches when an integer
    /// scrutinee equals this value. `negative` denotes the `-<lit>` fold. Valid
    /// only over an integer scrutinee; a literal-pattern match needs a `_`/binding
    /// catch-all to be exhaustive (checker-enforced).
    IntLit {
        value: u64,
        negative: bool,
        suffix: Option<ScalarTy>,
    },
    /// An integer-range pattern (`0x28..=0x3e` inclusive, `0..10` half-open):
    /// matches when an integer scrutinee lies in the range. Each endpoint reuses
    /// the literal-value machinery (`value`/`negative`/`suffix`). Valid only over
    /// an integer scrutinee; like literal patterns, a range-pattern match still
    /// needs a `_`/binding catch-all to be exhaustive (checker-enforced).
    IntRange {
        lo_value: u64,
        lo_negative: bool,
        lo_suffix: Option<ScalarTy>,
        hi_value: u64,
        hi_negative: bool,
        hi_suffix: Option<ScalarTy>,
        inclusive: bool,
    },
}

/// The signed value denoted by an integer-literal pattern's magnitude and sign.
pub fn int_pat_value(value: u64, negative: bool) -> i128 {
    if negative {
        -(value as i128)
    } else {
        value as i128
    }
}
