//! Stage A — the checked MIR (design 0010 §2, §5).
//!
//! A typed, explicit control-flow-graph mid-level IR lowered from the *checked,
//! resolved, monomorphized* AST. It carries the analysis tier's facts as data
//! (design 0010 §2(a)): fault checks (INV-CHECK), the static drop schedule
//! (INV-DROP), observable markers (INV-OBS-ORDER), and effect information; a
//! separate execution engine (`interp`) runs it *precisely* — every fault edge
//! taken immediately, no reordering, no window-interior retirement, no late
//! detection (the R1–R3-free `density → 1` limit of the formalization §10). This
//! makes the MIR a faithful *precise* carrier of 0001's semantics, validated by
//! `(k, s, θ)` semantic-trace equality against the tree-walking oracle
//! (`tests/stage_a.rs`).
//!
//! ## The MIR invariants (design 0010 §2), as checkable properties
//! * **INV-CHECK** — every default (`Regime::Checked`) arithmetic / conversion op
//!   carries its fault edge *explicitly* as data (`Bin.fault` / `Un.fault` /
//!   `Conv.fault` is `Some`); a `wrapping`/`saturating` op is a *distinct* op
//!   (`regime != Checked`) carrying no overflow edge. Asserted by
//!   [`check_invariants`].
//! * **INV-OBS-ORDER** — observable statements (here `trace`; rawptr/MMIO in the
//!   extension) are marked (`Statement::observable`) and emitted in program order;
//!   the lowering is order-preserving by construction (it only ever *appends*).
//!   Asserted structurally: observables' source spans are monotonic along each
//!   block's linear statement stream.
//! * **INV-FAULT-ID** — the delivered fault is program-order-first `f★`
//!   (kind + span). Stage A is **R1–R3-free**, so the operational f★-replay
//!   obligation (formalization §6.5) is *vacuous* — but the invariant's fields
//!   exist: every `MirFn` records [`ReplayPolicy::Precise`] (the replay origin is
//!   never consulted because no fault is ever detected late), and every fault edge
//!   carries the exact `(k, s)` it delivers.
//! * **INV-DROP** — drops appear as explicit [`StatementKind::Drop`] statements at
//!   exactly the static schedule (reverse declaration order at each scope exit,
//!   skipping moved/never-`needs_drop` locals), never behind a runtime flag.

use std::collections::HashMap;

use crate::ast::{BinOp, UnOp};
use crate::interp::FaultKind;
use crate::span::Span;
use crate::token::ScalarTy;
use crate::types::Type;

pub mod build;
pub mod interp;
pub mod opt;
pub mod serial;

pub use build::{lower_checked, LowerError};

/// Arithmetic regime (design 0001 §7.2; 0006 §2.4). The regime is *baked* into
/// every arithmetic op at lowering (from the enclosing `wrapping`/`saturating`
/// block), so it is greppable IR data — the INV-CHECK "distinct op" requirement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Regime {
    Checked,
    Wrapping,
    Saturating,
}

/// A fault edge carried as explicit MIR data (INV-CHECK / INV-FAULT-ID): the exact
/// fault identity `(k, s)` the op delivers when its check trips.
#[derive(Clone, Copy, Debug)]
pub struct FaultEdge {
    pub kind: FaultKind,
    pub span: Span,
}

/// The precision policy of a `MirFn`. Stage A is *precise* only (R1–R3-free): the
/// f★-replay origin (formalization §6.5) is never needed because no fault is
/// detected late. The field exists so a later batched/vectorized build can record
/// a different policy and discharge the replay obligation (design 0010 §2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplayPolicy {
    /// Every fault edge is taken immediately; replay is vacuous.
    Precise,
}

/// A projection step onto a place (design 0010 §2, INV-CHECK for the fault edge).
#[derive(Clone, Debug)]
pub enum Proj {
    /// `.field` — a compile-time-known byte offset plus the field's type.
    Field { offset: u64, ty: Type },
    /// `deref` — read the pointer stored at the current address and continue at
    /// the pointee (`inner` is the pointee type).
    Deref { inner: Type },
    /// `[i]` — a bounds-faulting index (INV-CHECK: the fault edge on every index).
    /// `stride` is the element stride; `len` the compile-known length (arrays) or
    /// is ignored for slices (the runtime length is read from the slice header —
    /// but slices carry `len` == u64::MAX here so the header check is used).
    Index { index: Operand, stride: u64, len: u64, span: Span, slice: bool },
}

/// A place: a root local plus a chain of projections (INV-DROP move masks apply
/// to field paths; the interpreter resolves it to a concrete address).
#[derive(Clone, Debug)]
pub struct Place {
    pub root: LocalId,
    pub proj: Vec<Proj>,
}

impl Place {
    pub fn local(root: LocalId) -> Place {
        Place { root, proj: Vec::new() }
    }
}

pub type LocalId = usize;
pub type BlockId = usize;

/// An operand: a compile-time integer constant (with its scalar type) or a read of
/// a local. Scalars carry their logical (sign-correct) value directly.
#[derive(Clone, Copy, Debug)]
pub enum Operand {
    Const(i128, ScalarTy),
    Local(LocalId),
}

/// A right-hand-side value. Fault-capable ops carry their fault edge as data.
#[derive(Clone, Debug)]
pub enum Rvalue {
    Use(Operand),
    /// Checked/wrapping/saturating arithmetic, bitwise, and shift (design 0001
    /// §7.2). `ty` is the computation/result scalar type. `span` is the op span
    /// (used for the always-on div-by-zero check on `Div`/`Rem`). `fault` is the
    /// overflow/shift edge — `Some` iff `regime == Checked` for a fallible op
    /// (INV-CHECK).
    Bin {
        op: BinOp,
        regime: Regime,
        ty: ScalarTy,
        l: Operand,
        r: Operand,
        span: Span,
        fault: Option<FaultEdge>,
    },
    /// Unary negate / logical-not / bitwise-not.
    Un {
        op: UnOp,
        regime: Regime,
        ty: ScalarTy,
        v: Operand,
        fault: Option<FaultEdge>,
    },
    /// Comparison (`== != < <= > >=`) producing a `bool`. Never faults.
    Cmp { op: BinOp, l: Operand, r: Operand },
    /// `conv T (e)` scalar conversion (design 0001 §8.1; 0016 for float). `to` is
    /// the target; the source type is recovered from `v`'s operand type. `fault` is
    /// the conv-loss edge — `Some` iff `regime == Checked` and `to` is an integer
    /// (INV-CHECK). An int→`f64` conversion (`to == f64`) never faults and carries
    /// no edge; a `f64`→int conversion saturates (design 0016 §5) so its edge — the
    /// same shape as an int→int conv's, kept uniform for the invariant — is inert
    /// (never taken).
    Conv {
        to: ScalarTy,
        regime: Regime,
        v: Operand,
        fault: Option<FaultEdge>,
    },
    /// `bitcast T (e)` same-width bit reinterpretation (design 0016 section 10). `to`
    /// is the target scalar; the operand's IDENTICAL bits are reinterpreted (never
    /// converted). Total -- never faults, so it carries NO fault edge (falls through
    /// INV-CHECK like a float op) and is regime-independent.
    Bitcast { to: ScalarTy, v: Operand },
    /// `sqrt(x)` — the correctly-rounded IEEE square root of a float (`f32`/`f64`),
    /// emitted natively by both backends (Cranelift `sqrt` / `llvm.sqrt`). A total,
    /// non-faulting unary float->float op: `sqrt` of a negative is NaN (not a
    /// fault) and `sqrt(-0.0) == -0.0`. Carries NO fault edge and is
    /// regime-independent, like a float arithmetic op. `ty` is the float width.
    Sqrt { ty: ScalarTy, v: Operand },
    /// A borrow / `addr_of`: the *address* of a place (a `u64` pointer value).
    Ref(Place),
    /// Read a scalar (or pointer-width) value out of a projected place.
    Load { place: Place, ty: Type },
    /// A direct call to a user function by name (non-generic subset).
    Call { func: String, args: Vec<Operand> },
    /// An indirect call through a fn-pointer id value (design 0007 §6.2): the
    /// `func` operand is a fn-pointer id, resolved to a `MirFn` at run time.
    CallIndirect { func: Operand, args: Vec<Operand> },
    /// Pointer arithmetic `base + index*stride` (wrapping, non-faulting): the
    /// `ptr_offset`/`field_ptr` intrinsics (design 0004). Produces a `u64` address.
    PtrArith { base: Operand, index: Operand, stride: u64 },
    /// `is_null(p)`: the address equals the reserved null (0).
    IsNull(Operand),
    /// The address of a top-level `static` value (design 0008): a `u64` into the
    /// static storage region, initialized before `main` by the static's init fn.
    StaticAddr(String),
    /// The address of an interned string literal's bytes in static storage
    /// (design 0001 §4.2): a `u8` slice's data pointer.
    StrAddr(String),
}

/// A compiler-known collection operation (PROPOSAL-selfhost-ergonomics / design
/// 0013): the `Vec[T]` / `Map[V]` / `String` builtins carried as MIR intrinsics.
/// The algorithm (FNV-1a hash, open-addressed linear probing, alloc-copy-free
/// growth, per-element / per-key/value drop) is NOT lowered into basic blocks; it
/// lives in ONE place — the MIR interpreter's handlers — mirroring the
/// tree-walking oracle (`interp::eval::bi_*`) byte-for-byte over the same flat
/// memory substrate. Every collection header is five `u64` words
/// `{ buf@0, len@8, cap@16, ctx@24, vt@32 }`; keys are byte-strings (`str`/`[u8]`).
#[derive(Clone, Debug)]
pub enum CollOp {
    /// `vec_new` / `map_new` / `string_new`: initialize an empty header
    /// `{ buf:0, len:0, cap:0, ctx, vt }` at `dst`, taking `ctx`/`vt` from the
    /// `Alloc` handle at `alloc` (the init is identical for all three headers).
    New { alloc: Operand },
    /// Vec `push(write v, x)`: move `value` onto the end, growing (alloc-copy-free)
    /// when full. Unit result.
    VecPush { base: Operand, elem: Type, value: Place, span: Span },
    /// Vec `pop(write v) -> Opt[elem]`: move the last element into `dst` as
    /// `Opt::Some`, or write `Opt::None` when empty.
    VecPop { base: Operand, elem: Type },
    /// Vec `get(read v, i) -> read elem`: a bounds-faulting borrow written to `dst`.
    VecGet { base: Operand, elem: Type, index: Operand, span: Span },
    /// Vec `set(write v, i, x)`: drop the overwritten element, then move `value`
    /// into slot `i` (bounds-faulting). Unit result.
    VecSet { base: Operand, elem: Type, index: Operand, value: Place, span: Span },
    /// Map `insert(write m, key, v)`: move `value` in under byte-string `key`
    /// (dropping a displaced value on an existing key; owning a key byte-copy on a
    /// new key; growing/rehashing at load factor 3/4). Unit result.
    MapInsert { base: Operand, valty: Type, key: Place, value: Place, span: Span },
    /// Map `contains(read m, key) -> bool`: the membership bool written to `dst`.
    MapContains { base: Operand, valty: Type, key: Place },
    /// Map `get(read m, key) -> read valty`: a borrow of the stored value written
    /// to `dst`, faulting `Bounds` if the key is absent.
    MapGet { base: Operand, valty: Type, key: Place, span: Span },
    /// String `push(write s, c: u32)`: append one UTF-8-encoded Unicode scalar
    /// value (faulting `Requires` on a surrogate / out-of-range code point). Unit.
    StringPush { base: Operand, ch: Operand, span: Span },
    /// String `append(write s, v: read str)`: append a validated byte view. Unit.
    StringAppend { base: Operand, view: Place, span: Span },
    /// String `as_str(read s) -> str`: a `{ptr@0, len@8}` view written to `dst`.
    StringAsStr { base: Operand },
}

#[derive(Clone, Debug)]
pub enum StatementKind {
    /// Assign an rvalue to a local place.
    Assign(LocalId, Rvalue),
    /// The `trace(x)` observable (design 0010 §5 / INV-OBS-ORDER): appends `x`
    /// (as `i64`) to the observable trace `θ`.
    Trace(Operand),
    /// Store a scalar (or pointer) rvalue into a projected place (field / index /
    /// deref) — the aggregate-construction and field/element-write path (A2).
    Store(Place, Rvalue),
    /// Byte-copy an aggregate value of `ty` from one place to another (a copy or a
    /// move of a whole struct/array/enum — the checker's copy/move facts decide
    /// which, and a moved source is pruned from the drop schedule statically).
    CopyVal { dst: Place, src: Place, ty: Type },
    /// A statically-scheduled drop of a local (INV-DROP), emitted at the exact
    /// scheduled point. `moved` is the checker's static move mask: the set of
    /// field paths already moved out of this local (baked at lowering time, NO
    /// runtime flag) — the drop skips those sub-paths, running hooks/frees only on
    /// the still-owned remainder (field-granular pruning, design 0010 §2 INV-DROP).
    Drop { local: LocalId, moved: Vec<Vec<String>> },
    /// `box(alloc, value)` (design 0001 §6.2): allocate through the handle's
    /// vtable, move `value` into the block, and build the `BoxResult` at `dst`.
    BoxOp { dst: Place, inner_ty: Type, result_ty: Type, alloc: Operand, value: Place },
    /// `unbox(b)` (design 0001 §6.2): copy the pointee into `dst`, then free the
    /// block through its stored vtable handle.
    UnboxOp { dst: Place, inner_ty: Type, boxed: Place },
    /// `subslice(s, lo, hi)` (design 0004): a bounds-checked slice re-header into
    /// `dst`; `lo > hi || hi > len` faults `Bounds` at `span`.
    Subslice { dst: Place, src: Place, lo: Operand, hi: Operand, stride: u64, span: Span },
    /// `str_from(b) -> Utf8Res` (design 0013 §4): UTF-8-validate `src`'s `[u8]`
    /// byte view, building `Utf8Res::Valid(str)` — the SAME `{ptr, len}` fat
    /// pointer, retyped — on a well-formed run, or `Utf8Res::Invalid(offset)`
    /// carrying the byte offset of the first ill-formed sequence (the offset
    /// `str::from_utf8().valid_up_to()` reports). An ENUM result written to `dst`,
    /// never a fault.
    StrFrom { dst: Place, src: Place },
    /// `substr(s, lo, hi) -> str` (design 0013 §3): the `[lo, hi)` byte sub-view
    /// written to `dst`. Faults `Bounds` at `span` when `lo > hi || hi > len`, OR
    /// when `lo`/`hi` does not fall on a UTF-8 character boundary (a continuation
    /// byte `0x80..=0xBF`) — one `Bounds` family for "this offset is not valid for
    /// this str", mirroring the tree-walker `bi_substr`.
    Substr { dst: Place, src: Place, lo: Operand, hi: Operand, span: Span },
    /// `spawn CALLEE(args)` inside a `scope` (design 0012 §1.1). The SEQUENTIAL
    /// ORACLE (tree-walker, MIR interp) runs the task inline at this point in spawn
    /// order — valid by SC-for-DRF (§6). The NATIVE backend (design 0012 Stage 2)
    /// instead creates a real OS thread running `func` with the marshalled `args`
    /// and joins it at the enclosing `ScopeEnd`. `func` is a direct MIR fn name.
    Spawn { func: String, args: Vec<Operand> },
    /// A compiler-known `Vec`/`Map`/`String` collection intrinsic (see [`CollOp`]):
    /// the interpreter runs the ported `interp::eval::bi_*` body onto the shared
    /// memory substrate, matching the tree-walker byte-for-byte.
    CollectionOp { dst: Place, op: CollOp },
    /// The opening `{` of a `scope` concurrency region (design 0012 §1.1). Oracle:
    /// a no-op. Native: push a scope frame onto the running thread's frame stack.
    ScopeBegin,
    /// The closing `}` / join barrier of a `scope` (design 0012 §1.1, §3.4). Oracle:
    /// a no-op (tasks already ran inline). Native: join every task of the top scope
    /// frame in spawn order, merge their per-task traces (deterministic θ), and
    /// deliver the spawn-order-first fault (§3.2) at the brace.
    ScopeEnd,
}

#[derive(Clone, Debug)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
    /// INV-OBS-ORDER: is this an observable operation? Observables are never
    /// reordered, coalesced, or eliminated by the lowering.
    pub observable: bool,
}

#[derive(Clone, Debug)]
pub enum Terminator {
    Goto(BlockId),
    Branch {
        cond: Operand,
        then_bb: BlockId,
        else_bb: BlockId,
    },
    /// Return the current value of local `0` (the return place).
    Return,
    /// A fault edge terminator (assert/panic/contract failure, and the sink of a
    /// branch to a fault block). Delivers `(k, s)` immediately.
    Fault(FaultEdge),
}

#[derive(Clone, Debug)]
pub struct BasicBlock {
    pub stmts: Vec<Statement>,
    pub term: Terminator,
}

/// A typed local. `_0` is the return place; `_1..=num_params` are the parameters.
#[derive(Clone, Debug)]
pub struct LocalDecl {
    pub ty: Type,
    pub name: Option<String>,
    /// INV-DROP: does this local carry a drop obligation (a `needs_drop` type)?
    pub drop_obligation: bool,
}

/// A contract predicate (`requires`/`ensures`) lowered into the function's block
/// graph: execute from `entry` until a `Return`, then read `value` (a `bool`); if
/// false, deliver `(kind, span)`.
#[derive(Clone, Debug)]
pub struct Predicate {
    pub entry: BlockId,
    pub value: LocalId,
    pub span: Span,
    pub kind: FaultKind,
}

#[derive(Clone, Debug)]
pub struct MirFn {
    pub name: String,
    pub num_params: usize,
    /// The local `ensures` reads as `result` (design 0001 §7.3), if any.
    pub result_local: Option<LocalId>,
    pub locals: Vec<LocalDecl>,
    pub blocks: Vec<BasicBlock>,
    pub entry: BlockId,
    pub requires: Vec<Predicate>,
    pub ensures: Vec<Predicate>,
    pub replay: ReplayPolicy,
}

/// A top-level `static` value (design 0008): its type and the MIR init fn that
/// computes it (run once, before `main`, into freshly reserved static storage).
#[derive(Clone, Debug)]
pub struct StaticInit {
    pub name: String,
    pub ty: Type,
    pub init_fn: String,
}

#[derive(Clone, Debug)]
pub struct MirProgram {
    pub fns: Vec<MirFn>,
    pub fn_index: HashMap<String, usize>,
    /// INV-DROP: nominal struct name -> the MIR function its `drop` hook lowered
    /// to (design 0010 §5). A monomorphized generic struct's hook is an ordinary
    /// concrete hook by lowering time; the interpreter calls it at the drop point.
    pub drop_hooks: HashMap<String, String>,
    /// Fn-pointer id table (design 0007 §6.2): a fn value is a `u64` index into
    /// this list; an indirect / vtable call resolves the id back to the `MirFn`.
    pub fn_ptrs: Vec<String>,
    /// Top-level statics, in program order (allocated + initialized before `main`).
    pub statics: Vec<StaticInit>,
}

impl MirProgram {
    pub fn get(&self, name: &str) -> Option<&MirFn> {
        self.fn_index.get(name).map(|i| &self.fns[*i])
    }
}

// ---------------------------------------------------------------------------
// Invariant checks (design 0010 §2). Run as debug assertions by the lowering and
// as an explicit test axis (`tests/stage_a.rs`).
// ---------------------------------------------------------------------------

/// Verify the MIR invariants of a well-formed function. Panics with a precise
/// message on violation (used via `debug_assert` in the lowering).
pub fn check_invariants(f: &MirFn) {
    // INV-FAULT-ID: Stage A is precise; the replay origin is never consulted.
    assert_eq!(f.replay, ReplayPolicy::Precise, "Stage A MIR must be precise");
    for (bi, b) in f.blocks.iter().enumerate() {
        let mut last_obs: Option<usize> = None;
        for s in &b.stmts {
            // INV-CHECK: every default-regime fallible arithmetic/conv op carries
            // its fault edge explicitly.
            if let StatementKind::Assign(_, rv) = &s.kind {
                match rv {
                    // Float ops are IEEE and regime-exempt — they never fault, so
                    // they carry no edge even in the checked regime (design 0016).
                    Rvalue::Bin { op, regime, ty, fault, .. }
                        if *regime == Regime::Checked && bin_is_fallible(*op) && ty.is_integer() =>
                    {
                        assert!(
                            fault.is_some(),
                            "INV-CHECK: checked {op:?} in {}#bb{bi} lacks its fault edge",
                            f.name
                        );
                    }
                    Rvalue::Un { op: UnOp::Neg, regime, ty, fault, .. }
                        if *regime == Regime::Checked && ty.is_integer() =>
                    {
                        assert!(fault.is_some(), "INV-CHECK: checked neg in {} lacks its fault edge", f.name);
                    }
                    // A checked conv to an INTEGER target carries its (conv-loss)
                    // edge — inert for an f64 source (saturating; design 0016). An
                    // int->f64 conv (`to == f64`) is regime-exempt and carries none.
                    Rvalue::Conv { to, regime, fault, .. }
                        if *regime == Regime::Checked && to.is_integer() =>
                    {
                        assert!(fault.is_some(), "INV-CHECK: checked conv in {} lacks its fault edge", f.name);
                    }
                    _ => {}
                }
            }
            // INV-OBS-ORDER: observables appear in nondecreasing source order along
            // the block (the lowering is order-preserving — it only appends).
            if s.observable {
                if let Some(prev) = last_obs {
                    assert!(
                        s.span.start >= prev,
                        "INV-OBS-ORDER: observable reordered in {}#bb{bi}",
                        f.name
                    );
                }
                last_obs = Some(s.span.start);
            }
        }
    }
}

fn bin_is_fallible(op: BinOp) -> bool {
    matches!(
        op,
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem | BinOp::Shl | BinOp::Shr
    )
}
