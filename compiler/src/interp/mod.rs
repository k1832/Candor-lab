//! Stage 4 — the tree-walking interpreter (design 0001 §1.5, §4.2, §5, §6, §7, §8).
//!
//! # Memory model
//!
//! All runtime values live in a single flat, byte-addressable memory
//! ([`mem::Mem`]) at defined addresses with defined layouts. A `rawptr` value is
//! a plain `u64` address into that memory; `addr_of`, `ptr_read`, `ptr_offset`,
//! `container_of`, `cast_ptr`, and fixed-address (`addr_to_ptr`) MMIO all operate
//! on these addresses directly.
//!
//! ## Layout rules (documented, see `layout.rs`)
//! * Field order is declared order — never reordered.
//! * Every scalar has natural size/alignment: `i8`/`u8`/`bool` = 1, `i16`/`u16` =
//!   2, `i32`/`u32` = 4, `i64`/`u64`/`isize`/`usize`/`rawptr`/fn-ptr/borrow = 8.
//!   `unit` = 0.
//! * A struct lays fields out in order at their natural alignment; struct size is
//!   rounded up to the struct's alignment (max field alignment).
//! * An array `[N]T` is `N` contiguous `T` (stride = `sizeof(T)`, already aligned).
//! * A `slice`/`slice_mut` is a `(addr: u64, len: u64)` pair (16 bytes).
//! * An enum is `{ tag: u64 @0, payload @8 }`; each variant's payload is laid out
//!   struct-style starting at offset 8. Enum size = round_up(8 + max payload, 8).
//! * `Box T` is `{ ptr: u64 @0, ctx: u64 @8, vt: u64 @16 }` (24 bytes) — the
//!   pointee address plus its owning `Alloc` handle (§6.2).
//! * `BoxResult T` is the compiler-known enum `{ boxed(Box T), oom }`.
//!
//! ## The initialized-byte guard
//! `Mem` keeps an initialized-byte bitmap. Every *typed* read (including
//! `ptr_read`) faults if it touches a never-written byte. Per §4.2 the valve
//! *permits* reading uninitialized memory at the author's responsibility, so this
//! guard is a **prototype diagnostic aid** (it catches interpreter bugs and
//! honest use-before-init that escapes the checker), **not** language semantics.
//! Reading an address beyond the model's cap is a fault ("address beyond memory
//! model"); reading in-range written bytes returns whatever is there (the valve's
//! meaning).
//!
//! ## Drops from static move facts (no runtime drop flags)
//! §1.6 forbids conditional move divergence and proves move state agrees at every
//! join. Consequently the set of live drop obligations at every scope/statement
//! exit is *path-independent* — a static fact. The interpreter therefore does not
//! carry any per-value runtime boolean that "decides whether to drop": it simply
//! collects the drop schedule from the moves it performs (each a static fact the
//! checker validated) and, at each block/statement end, drops exactly the places
//! that are statically owned there, in the §1.5 order. A partial move records the
//! moved sub-path so the remainder still drops. This is equivalent to a static
//! drop-obligation pass, only *collected* during the walk.

pub mod eval;
pub mod layout;
pub mod mem;

use serde::Serialize;

use crate::ast::Program;
use crate::resolve::resolve_program;
use crate::span::Span;

/// The kinds of precise runtime fault (design 0001 §7.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FaultKind {
    Overflow,
    DivByZero,
    Bounds,
    ConvLoss,
    Assert,
    Requires,
    Ensures,
    Panic,
    /// Reading an address beyond the memory model, or the init-byte guard.
    BadPointer,
    /// A foreign (`extern`) call to a symbol with no registered shim and no
    /// native backend (design 0011 §5). The honest runtime gap before 0010.
    NoForeignRuntime,
}

/// A structured, machine-readable fault report (design 0001 §7.4, P4/P7).
#[derive(Clone, Debug, Serialize)]
pub struct Fault {
    pub kind: FaultKind,
    pub span: Span,
    pub message: String,
    /// Values emitted by `trace(x)` BEFORE the fault, threaded in at the run
    /// boundary so a differential harness can compare the pre-fault trace, not
    /// just the fault's kind+span (F-FAULT-TRACE). Empty until the run's
    /// top-level attaches the accumulated trace.
    pub trace: Vec<i64>,
}

impl Fault {
    pub fn new(kind: FaultKind, span: Span, message: impl Into<String>) -> Fault {
        Fault {
            kind,
            span,
            message: message.into(),
            trace: Vec::new(),
        }
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Fault is serializable")
    }
}

/// The result of a completed run: `main`'s return value plus the drop/trace log.
#[derive(Clone, Debug)]
pub struct Run {
    /// `main`'s `i64` return value, or `0` for a unit `main`.
    pub ret: i64,
    /// Values appended by the `trace(x)` builtin (drop-order observation).
    pub trace: Vec<i64>,
}

/// Exit code for a runtime fault — distinct from the check-error exit (1).
pub const FAULT_EXIT: u8 = 2;

/// Parse+check are the caller's job; this runs a fully-checked program's `main`.
pub fn run_program(prog: &Program) -> Result<Run, Fault> {
    let mut diags = Vec::new();
    let items = resolve_program(prog, &mut diags);
    let mut interp = eval::Interp::new(prog, &items);
    interp.run_main()
}
