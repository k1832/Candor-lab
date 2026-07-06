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

#[derive(Clone, Debug)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Clone, Debug)]
pub enum Item {
    Struct(StructDecl),
    Enum(EnumDecl),
    Fn(FnDecl),
    Static(StaticDecl),
}

#[derive(Clone, Debug)]
pub struct StructDecl {
    pub copy: bool,
    pub name: String,
    pub fields: Vec<Field>,
    /// Optional `drop(write self) { ... }` hook (design 0001 §1.5).
    pub drop_hook: Option<Block>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub ty: Ty,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct EnumDecl {
    pub copy: bool,
    pub name: String,
    pub variants: Vec<Variant>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Variant {
    pub name: String,
    /// Zero or more payload types (design 0001 §8.2).
    pub payload: Vec<Ty>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FnDecl {
    pub name: String,
    /// Region variables declared in brackets after the name (design 0001 §3.3).
    pub regions: Vec<String>,
    pub params: Vec<Param>,
    /// The `alloc` effect marker (design 0001 §3.2).
    pub alloc: bool,
    pub requires: Vec<Expr>,
    pub ensures: Vec<Expr>,
    /// Return spec; `None` means the unit return (`-> unit` omitted).
    pub ret: Option<RetTy>,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Debug)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParamMode {
    Take,
    Read,
    Write,
    Out,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    pub mode: ParamMode,
    /// Region tag on a borrow parameter, e.g. the `r` in `read[r] Slice`.
    pub region: Option<String>,
    pub ty: Ty,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorrowKind {
    Shared,    // `read`
    Exclusive, // `write`
}

/// A function return type, possibly a borrow return with a region (design §3.3).
#[derive(Clone, Debug)]
pub struct RetTy {
    pub borrow: Option<BorrowKind>,
    pub region: Option<String>,
    pub ty: Ty,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Ty {
    pub kind: TyKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum TyKind {
    Scalar(ScalarTy),
    /// A user struct/enum name (unresolved — no symbol table).
    Named(String),
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
#[derive(Clone, Debug)]
pub struct FnPtrTy {
    pub params: Vec<FnPtrParam>,
    pub alloc: bool,
    pub ret: Box<Ty>,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrefixOp {
    Deref,
    Read,
    Write,
    Clone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FieldInit {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    IntLit { value: u64, suffix: Option<ScalarTy> },
    StrLit(String),
    BoolLit(bool),
    Ident(String),

    Unary { op: UnOp, expr: Box<Expr> },
    Prefix { op: PrefixOp, expr: Box<Expr> },
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr> },

    Call { callee: Box<Expr>, args: Vec<Expr> },
    Field { base: Box<Expr>, field: String },
    Index { base: Box<Expr>, index: Box<Expr> },

    /// `conv T (e)` — the only integer conversion form (design 0001 §8.1).
    Conv { ty: Ty, expr: Box<Expr> },

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
}

// ---------------------------------------------------------------------------
// Patterns (design 0001 §8.2 / §8.2.1)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Pattern {
    pub kind: PatKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
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
}
