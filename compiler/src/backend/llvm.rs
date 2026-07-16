//! S1 — the OPTIMIZED native backend (LLVM): MIR -> textual LLVM-IR, built by
//! `clang -O2`, linked against the same static C runtime (`aot_runtime.c`) the
//! Cranelift AOT object uses. This backend mirrors the *semantics* of
//! `backend::lower` (the Cranelift reference) in `.ll` text — it does not reuse
//! any Cranelift code.
//!
//! ## Scope
//! S0: integer/bool scalars; let/assign; if/else/while/loop/break/continue/return;
//! arithmetic (+ - * / %) with Checked/Wrapping/Saturating regimes and
//! Overflow/DivByZero/ConvLoss faults; comparisons; &&/||; bitwise/shift; unary
//! neg/not; `trace(x)`; assert/panic; requires/ensures; direct fn calls + recursion.
//! S1 adds STRUCTS and ARRAYS (the flat aggregate model): struct/array literals,
//! field/index read+assign, the Index bounds fault, nested aggregates, by-value
//! struct params + struct returns, and `Ref` (address-of). S2 adds TAGGED-UNION
//! ENUMS (tag + payload-union in the flat model; construction = tag+payload stores,
//! `match` = tag read -> icmp/branch chain -> per-variant payload projection) and
//! STATIC/CONST data (the static region: string-literal bytes + `static` initializers
//! run before `main`, addressed via `StaticAddr`/`StrAddr`). S3 adds the MOVE/DROP
//! SCHEDULE (drop glue at each `Drop`, trace-on-drop, static move-mask pruning). S4
//! adds HEAP ALLOCATION: `Box[T]`/`unbox` through a Candor allocator handle's vtable
//! (the fn-pointer dispatch table), raw-pointer load/store (observable rt_mmio
//! barriers), and drop-through-Box (free-on-drop via per-pointee drop glue).
//! S5 adds EXTERN/FFI (boundary `extern` declares + the C/SysV ABI call marshalling
//! — pointer args translated to real host addresses at the trust boundary, the libc
//! I/O path standalone binaries use) and STRUCTURED CONCURRENCY (design 0012 Stage 2:
//! `spawn` a real OS thread via `rt_spawn`, the `scope` join barrier via
//! `rt_scope_begin`/`rt_scope_end`, cross-thread trace merged in spawn order). Only
//! the `Vec`/`Map`/`String` collection intrinsics (MIR-interp only, as in `lower`)
//! remain out of subset, rejected with a precise "out of LLVM-S5 subset" error.
//!
//! ## The two-tier value model
//! Each local is classified by a per-fn MIR scan (LLVM's own "is the address
//! taken?"):
//! * **Tier-R (register).** A scalar (`is_wordy`) local whose address is never
//!   taken -> `alloca i64` in the entry block; a use is a `load`, a definition a
//!   `store`. No such slot's address escapes and there are no aggregates in it, so
//!   `clang -O2`'s mem2reg promotes every Tier-R slot to an SSA register — that is
//!   where the optimization comes from (the S0 perf win, preserved).
//! * **Tier-F (flat).** Aggregates (`!is_wordy`), any local whose address IS taken
//!   (`Ref` / by-address param / byte-copied via `rt_copy`) -> live in the flat
//!   MEM_BASE buffer via `rt_stack_alloc(size, align)` at fn entry, addressed as
//!   `inttoptr(MEM_BASE + candor_off)`. Their stable MEM_BASE-relative offset is
//!   the Candor "address" that `Ref`/`Index`/borrows pass around. Flat loads/stores
//!   through `inttoptr` are correct but optimizer-opaque (the documented flat-arena
//!   ceiling); making aggregates fast is a later ABI project.
//!
//! ## Correctness invariants preserved (mirroring `lower`)
//! * **INV-CHECK.** Checked add/sub/mul use `llvm.{s,u}{add,sub,mul}.with.overflow`
//!   — NEVER `add nsw`/`mul nsw` (LLVM would delete the overflow test as
//!   UB-unreachable). The overflow bit is an explicit `br i1` to a fault block.
//!   Conv/neg range checks and the array-index bounds check are explicit `icmp`.
//! * **INV-FAULT-ID.** Every fault edge is `call void @rt_fault(kind, s_start,
//!   s_end)` then `unreachable`, with the stable `kind_code` map.
//! * **INV-OBS-ORDER.** `trace`/`rt_fault`/`rt_copy`/`rt_stack_alloc` are bare
//!   external declares (no `readnone`/`memory(none)`): side-effecting external
//!   calls are optimization barriers, so `-O2` preserves trace order and fault
//!   points.

use std::path::Path;
use std::process::Command;

use crate::ast::{BinOp, UnOp};
use crate::interp::layout::Layout;
use crate::interp::mem::{round_up, STATIC_BASE};
use crate::interp::FaultKind;
use crate::mir::{
    CollOp, FaultEdge, MirFn, MirProgram, Operand, Place, Proj, Regime, ReplayPolicy, Rvalue,
    Statement, StatementKind, Terminator,
};
use crate::resolve::Items;
use crate::span::Span;
use crate::token::ScalarTy;
use crate::types::{ItemEnv, Type};

use std::collections::HashMap;

use super::lower::kind_code;

/// The static C runtime, reused UNCHANGED (the AOT twin of `runtime.rs`). Linked
/// by `clang` exactly as the Cranelift object links it.
const RUNTIME_C: &str = include_str!("aot_runtime.c");

/// Flat-buffer base: a host load/store of Candor address `a` is at `MEM_BASE + a`.
/// Must match `aot_runtime.c`'s `MEM_BASE` and `interp::mem`/`backend::object`.
const MEM_BASE: i64 = 0x0000_2000_0000_0000;

/// (min, max, bits, signed) for a scalar type — the ranges the interpreter and the
/// oracle use (a private mirror of `lower::ty_range`).
fn ty_range(sty: ScalarTy) -> (i128, i128, u32, bool) {
    let (bits, signed): (u32, bool) = match sty {
        ScalarTy::I8 => (8, true),
        ScalarTy::I16 => (16, true),
        ScalarTy::I32 => (32, true),
        ScalarTy::I64 | ScalarTy::Isize => (64, true),
        ScalarTy::Bool | ScalarTy::U8 => (8, false),
        ScalarTy::U16 => (16, false),
        ScalarTy::U32 => (32, false),
        ScalarTy::U64 | ScalarTy::Usize => (64, false),
        // A float bit pattern is unsigned so it is never sign-extended (design 0016).
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

/// A local's storage tier hinges on whether it is word-sized (a scalar/pointer that
/// fits in an i64 register). Mirror of `lower::is_wordy`.
fn is_wordy(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Scalar(_) | Type::Borrow(_) | Type::BorrowMut(_) | Type::RawPtr(_) | Type::FnPtr(_)
    )
}

fn scalar_of(ty: &Type) -> ScalarTy {
    match ty {
        Type::Scalar(s) => *s,
        _ => ScalarTy::U64,
    }
}

/// The LLVM C-ABI integer type for a boundary parameter/return: a pointer word or a
/// full 64-bit scalar is `i64`; a narrower (<=32-bit) scalar is `i32` (its natural
/// SysV register width). A private mirror of `lower::c_abi_ty`, in LLVM type spelling.
fn c_abi_llty(ty: &Type) -> &'static str {
    match ty {
        Type::RawPtr(_) | Type::FnPtr(_) | Type::Borrow(_) | Type::BorrowMut(_) => "i64",
        Type::Scalar(s) => {
            let (_, _, bits, _) = ty_range(*s);
            if bits <= 32 {
                "i32"
            } else {
                "i64"
            }
        }
        _ => "i64",
    }
}

/// The LLVM global symbol for a Candor function's compiled body. Always quoted:
/// MIR names for compiler-synthesized bodies (e.g. a `static`'s `"<init G>"`
/// initializer) contain characters that are not valid in a bare LLVM identifier.
fn cnf_sym(name: &str) -> String {
    format!("@\"cnf_{name}\"")
}

fn overflow_intrinsic(op: BinOp, signed: bool, bits: u32) -> String {
    let base = match op {
        BinOp::Add => if signed { "sadd" } else { "uadd" },
        BinOp::Sub => if signed { "ssub" } else { "usub" },
        BinOp::Mul => if signed { "smul" } else { "umul" },
        _ => unreachable!("overflow intrinsic for non add/sub/mul"),
    };
    format!("llvm.{base}.with.overflow.i{bits}")
}

/// Lay out `static` + string-literal Candor addresses with the same bump
/// arithmetic the Cranelift driver uses (`backend::object::layout_statics_strings`
/// / `backend::run`), so the `StaticAddr`/`StrAddr` constants baked into function
/// bodies agree with the addresses `candor_entry`'s prologue writes to. Statics
/// first (in program order, each aligned), then interned string bytes.
fn layout_statics_strings(
    prog: &MirProgram,
    lay: &Layout,
) -> (HashMap<String, u64>, HashMap<String, u64>) {
    let mut bump = STATIC_BASE;
    let mut statics = HashMap::new();
    for st in &prog.statics {
        let size = lay.size_of(&st.ty).max(1);
        let align = lay.align_of(&st.ty).max(1);
        let a = round_up(bump, align);
        bump = a + size;
        statics.insert(st.name.clone(), a);
    }
    let mut strings = HashMap::new();
    for sv in super::collect_strings(prog) {
        let len = (sv.len().max(1)) as u64;
        let a = round_up(bump, 1);
        bump = a + len;
        strings.insert(sv, a);
    }
    (statics, strings)
}

/// Emit the whole program as one textual LLVM-IR module: the `rt_*` + intrinsic
/// declares, one `cnf_<name>` function per `MirFn`, then the `candor_entry` glue.
/// `items`/`consts` supply the type layout (aggregate sizes/alignments for the flat
/// `rt_stack_alloc`/`rt_copy` ABI; field offsets/strides are already baked in MIR).
pub fn emit_ll(
    prog: &MirProgram,
    items: &Items,
    consts: &HashMap<String, u64>,
) -> Result<String, String> {
    if prog.get("main").is_none() {
        return Err("no `main` function to compile".to_string());
    }
    let lay = Layout { items, consts };
    let (statics, strings) = layout_statics_strings(prog, &lay);
    let mut out = String::new();
    out.push_str("; Candor LLVM-S1 module\n");
    out.push_str("declare void @rt_trace(i64)\n");
    out.push_str("declare void @rt_fault(i32, i32, i32) #0\n");
    out.push_str("declare i64 @rt_stack_alloc(i64, i64)\n");
    out.push_str("declare void @rt_copy(i64, i64, i64)\n");
    // Correctly-rounded IEEE square root (design 0016 §11): the native LLVM
    // intrinsic, emitted for the Candor `sqrt` builtin / WASM `f*.sqrt`.
    out.push_str("declare double @llvm.sqrt.f64(double)\n");
    out.push_str("declare float @llvm.sqrt.f32(float)\n");
    out.push_str("declare i64 @rt_mmio_load(i64, i64)\n");
    out.push_str("declare void @rt_mmio_store(i64, i64, i64)\n");
    // Structured-concurrency Stage 2 (design 0012): the raw-pthread scope/spawn
    // markers `-O2` must never reorder or elide (bare side-effecting declares).
    // `rt_spawn(faddr, argc, a0..a5)` — reused UNCHANGED from aot_runtime.c.
    out.push_str("declare void @rt_scope_begin()\n");
    out.push_str("declare void @rt_scope_end()\n");
    out.push_str(&format!(
        "declare void @rt_spawn({})\n",
        ["i64"; 2 + super::lower::MAX_SPAWN_ARGS].join(", ")
    ));
    // Boundary `extern`s (design 0011 §5): each imported C symbol declared at its
    // SysV C-ABI signature (pointer/word -> i64, sub-word scalar -> i32, unit ->
    // void), keyed by `c_symbol_name`. A foreign call resolves to the REAL libc
    // symbol in the linked binary — the standalone-binary trust boundary.
    let mut externs: Vec<_> = items.externs.iter().collect();
    externs.sort_by(|a, b| a.0.cmp(b.0));
    for (name, es) in externs {
        let params = es
            .params
            .iter()
            .map(|p| c_abi_llty(&p.lowered))
            .collect::<Vec<_>>()
            .join(", ");
        let ret = if matches!(es.ret, Type::Scalar(ScalarTy::Unit)) {
            "void"
        } else {
            c_abi_llty(&es.ret)
        };
        out.push_str(&format!(
            "declare {ret} @\"{}\"({params})\n",
            super::lower::c_symbol_name(name)
        ));
    }
    out.push('\n');
    for bits in [8u32, 16, 32, 64] {
        for op in [BinOp::Add, BinOp::Sub, BinOp::Mul] {
            for signed in [true, false] {
                let name = overflow_intrinsic(op, signed, bits);
                out.push_str(&format!(
                    "declare {{i{bits}, i1}} @{name}(i{bits}, i{bits})\n"
                ));
            }
        }
    }
    out.push('\n');

    // Per-Box-pointee drop-glue (INV-DROP, design 0010 §5): a recursive Box type's
    // drop is runtime recursion through a synthesized glue fn (terminating on the
    // null pointer), never infinite compile-time unrolling. The fn-pointer dispatch
    // table maps each fn-ptr id (a `u64` baked into vtable/handle statics) to the
    // callee's address, so box/unbox alloc/free dispatch is an indirect call.
    let glue_types = super::lower::collect_glue_types(prog, items, consts);
    let mut glue_index: HashMap<String, usize> = HashMap::new();
    for (i, ty) in glue_types.iter().enumerate() {
        glue_index.insert(type_key(ty), i);
    }
    emit_fnptr_table(&mut out, prog);

    for f in &prog.fns {
        emit_fn(&mut out, f, &lay, &statics, &strings, &prog.drop_hooks, &glue_index)?;
    }
    for (i, ty) in glue_types.iter().enumerate() {
        emit_glue_fn(&mut out, i, ty, &lay, &statics, &strings, &prog.drop_hooks, &glue_index)?;
    }
    emit_entry(&mut out, prog, &lay, &statics, &strings);

    out.push_str("\nattributes #0 = { noreturn }\n");
    Ok(out)
}

/// The `candor_entry() -> i64` glue the runtime calls (mirrors `lower::lower_entry`).
/// The prologue runs the startup work the JIT driver does host-side: copy each
/// string literal's bytes into the flat buffer at its baked Candor address, then
/// run each `static` initializer and write its result to the static's address
/// (wordy: the low `size` bytes; aggregate: `rt_copy` from the returned candor
/// offset). Then call `main` and return its `i64` (or `0` for a non-`i64` main).
fn emit_entry(
    out: &mut String,
    prog: &MirProgram,
    lay: &Layout,
    statics: &HashMap<String, u64>,
    strings: &HashMap<String, u64>,
) {
    // `main` reports its 64-bit return WORD for `i64` or `f64` (design 0016).
    let main_is_i64 = matches!(
        prog.get("main").map(|f| &f.locals[0].ty),
        Some(Type::Scalar(ScalarTy::I64)) | Some(Type::Scalar(ScalarTy::F64))
    );
    out.push_str("define i64 @candor_entry() {\nentry:\n");
    let mut n = 0usize;

    // String-literal bytes -> flat buffer at `MEM_BASE + addr + i`.
    for sv in super::collect_strings(prog) {
        let addr = strings[&sv];
        for (i, byte) in sv.as_bytes().iter().enumerate() {
            let host = MEM_BASE + (addr + i as u64) as i64;
            out.push_str(&format!("  %e{n} = inttoptr i64 {host} to ptr\n"));
            out.push_str(&format!("  store i8 {byte}, ptr %e{n}\n"));
            n += 1;
        }
    }

    // Static initializers: run `cnf_<init_fn>()`, deliver into the static's slot
    // (wordy: store the low `size` bytes; aggregate: `rt_copy` from the returned
    // candor offset).
    for st in &prog.statics {
        let addr = statics[&st.name];
        let size = lay.size_of(&st.ty);
        out.push_str(&format!("  %e{n} = call i64 {}()\n", cnf_sym(&st.init_fn)));
        let r = format!("%e{n}");
        n += 1;
        if is_wordy(&st.ty) {
            if size > 0 {
                let host = MEM_BASE + addr as i64;
                out.push_str(&format!("  %e{n} = inttoptr i64 {host} to ptr\n"));
                let p = format!("%e{n}");
                n += 1;
                let bits = size * 8;
                if bits >= 64 {
                    out.push_str(&format!("  store i64 {r}, ptr {p}\n"));
                } else {
                    out.push_str(&format!("  %e{n} = trunc i64 {r} to i{bits}\n"));
                    out.push_str(&format!("  store i{bits} %e{n}, ptr {p}\n"));
                    n += 1;
                }
            }
        } else if size > 0 {
            out.push_str(&format!("  call void @rt_copy(i64 {addr}, i64 {r}, i64 {size})\n"));
        }
    }

    out.push_str(&format!("  %r = call i64 {}()\n", cnf_sym("main")));
    if main_is_i64 {
        out.push_str("  ret i64 %r\n");
    } else {
        out.push_str("  ret i64 0\n");
    }
    out.push_str("}\n");
}

/// DFS successors from `entry`, tagging each unvisited MIR block with `target`
/// (its return destination / region membership) — mirrors `lower::assign_region`.
fn assign_region(mf: &MirFn, entry: usize, target: &str, ret_target: &mut [Option<String>]) {
    let mut stack = vec![entry];
    while let Some(bid) = stack.pop() {
        if ret_target[bid].is_some() {
            continue;
        }
        ret_target[bid] = Some(target.to_string());
        match &mf.blocks[bid].term {
            Terminator::Goto(n) => stack.push(*n),
            Terminator::Branch { then_bb, else_bb, .. } => {
                stack.push(*then_bb);
                stack.push(*else_bb);
            }
            Terminator::Return | Terminator::Fault(_) => {}
        }
    }
}

/// Classify every local into Tier-R (register: scalar, address never taken) or
/// Tier-F (flat: aggregate or address-taken). A local is Tier-F iff it is a
/// non-word type OR its own storage address is needed — i.e. it is the root of a
/// `Ref` or a `CopyVal` whose projection does not begin with a `Deref` (a leading
/// `Deref` reads the local's pointer *value*, not its address).
fn classify_tiers(mf: &MirFn) -> Vec<bool> {
    let mut tf = vec![false; mf.locals.len()];
    for (i, l) in mf.locals.iter().enumerate() {
        if !is_wordy(&l.ty) {
            tf[i] = true;
        }
    }
    let mark = |place: &Place, tf: &mut [bool]| {
        if !matches!(place.proj.first(), Some(Proj::Deref { .. })) {
            tf[place.root] = true;
        }
    };
    for b in &mf.blocks {
        for st in &b.stmts {
            match &st.kind {
                StatementKind::Assign(_, Rvalue::Ref(p))
                | StatementKind::Store(_, Rvalue::Ref(p)) => mark(p, &mut tf),
                StatementKind::CopyVal { dst, src, .. } => {
                    mark(dst, &mut tf);
                    mark(src, &mut tf);
                }
                // Box/unbox/subslice copy through place ADDRESSES, so their
                // operand slots must live flat (a boxed scalar payload too).
                StatementKind::BoxOp { dst, value, .. } => {
                    mark(dst, &mut tf);
                    mark(value, &mut tf);
                }
                StatementKind::UnboxOp { dst, boxed, .. } => {
                    mark(dst, &mut tf);
                    mark(boxed, &mut tf);
                }
                StatementKind::Subslice { dst, src, .. } => {
                    mark(dst, &mut tf);
                    mark(src, &mut tf);
                }
                // str_from / substr copy through place ADDRESSES (`place_addr`),
                // so their result + source fat-pointer slots must live flat.
                StatementKind::StrFrom { dst, src } => {
                    mark(dst, &mut tf);
                    mark(src, &mut tf);
                }
                StatementKind::Substr { dst, src, .. } => {
                    mark(dst, &mut tf);
                    mark(src, &mut tf);
                }
                // Collection ops write their result and read element/value operands
                // through place ADDRESSES (`place_addr`), so those slots must be flat.
                StatementKind::CollectionOp { dst, op } => {
                    mark(dst, &mut tf);
                    match op {
                        CollOp::VecPush { value, .. } | CollOp::VecSet { value, .. } => {
                            mark(value, &mut tf)
                        }
                        CollOp::MapInsert { key, value, .. } => {
                            mark(key, &mut tf);
                            mark(value, &mut tf);
                        }
                        CollOp::MapContains { key, .. } | CollOp::MapGet { key, .. } => {
                            mark(key, &mut tf)
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
    tf
}

/// Per-function textual-IR emitter. Values are named `%v<n>`, synthetic blocks
/// `L<n>`, MIR blocks `mbb<bid>`; naming everything sidesteps LLVM's implicit
/// numbering of unnamed values/blocks. Tier-R locals are `%loc<id>` allocas;
/// Tier-F locals are `%off<id>` flat candor offsets.
struct FnEmit<'a> {
    out: String,
    tmp: usize,
    lbl: usize,
    mf: &'a MirFn,
    lay: &'a Layout<'a>,
    statics: &'a HashMap<String, u64>,
    strings: &'a HashMap<String, u64>,
    drop_hooks: &'a HashMap<String, String>,
    /// type_key(pointee) -> its drop-glue index (`@__drop_glue_<i>`).
    glue_index: &'a HashMap<String, usize>,
    tier_f: Vec<bool>,
}

impl<'a> FnEmit<'a> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        mf: &'a MirFn,
        lay: &'a Layout<'a>,
        statics: &'a HashMap<String, u64>,
        strings: &'a HashMap<String, u64>,
        drop_hooks: &'a HashMap<String, String>,
        glue_index: &'a HashMap<String, usize>,
    ) -> Self {
        let tier_f = classify_tiers(mf);
        FnEmit {
            out: String::new(),
            tmp: 0,
            lbl: 0,
            mf,
            lay,
            statics,
            strings,
            drop_hooks,
            glue_index,
            tier_f,
        }
    }
    fn t(&mut self) -> String {
        let n = self.tmp;
        self.tmp += 1;
        format!("%v{n}")
    }
    fn l(&mut self) -> String {
        let n = self.lbl;
        self.lbl += 1;
        format!("L{n}")
    }
    fn raw(&mut self, s: &str) {
        self.out.push_str(s);
    }
    fn line(&mut self, s: &str) {
        self.out.push_str("  ");
        self.out.push_str(s);
        self.out.push('\n');
    }
    fn label(&mut self, name: &str) {
        self.out.push_str(name);
        self.out.push_str(":\n");
    }

    // ---- Tier-F flat memory access (mirrors lower::host_addr/load_scalar/store_scalar) ----

    /// `inttoptr(MEM_BASE + candor)` — the host `ptr` for a flat candor address.
    fn host_ptr(&mut self, candor: &str) -> String {
        let a = self.t();
        self.line(&format!("{a} = add i64 {MEM_BASE}, {candor}"));
        let p = self.t();
        self.line(&format!("{p} = inttoptr i64 {a} to ptr"));
        p
    }
    /// Load a scalar leaf from a flat candor address as the canonical i64.
    fn load_scalar(&mut self, candor: &str, sty: ScalarTy) -> String {
        if Layout::scalar_size(sty) == 0 {
            return "0".to_string();
        }
        let (_, _, bits, signed) = ty_range(sty);
        let p = self.host_ptr(candor);
        let raw = self.t();
        self.line(&format!("{raw} = load i{bits}, ptr {p}"));
        if bits >= 64 {
            raw
        } else {
            let ex = self.t();
            let opn = if signed { "sext" } else { "zext" };
            self.line(&format!("{ex} = {opn} i{bits} {raw} to i64"));
            ex
        }
    }
    /// Store the canonical i64 `val` as a scalar leaf at a flat candor address.
    fn store_scalar(&mut self, candor: &str, val: &str, sty: ScalarTy) {
        if Layout::scalar_size(sty) == 0 {
            return;
        }
        let (_, _, bits, _) = ty_range(sty);
        let p = self.host_ptr(candor);
        if bits >= 64 {
            self.line(&format!("store i64 {val}, ptr {p}"));
        } else {
            let tr = self.t();
            self.line(&format!("{tr} = trunc i64 {val} to i{bits}"));
            self.line(&format!("store i{bits} {tr}, ptr {p}"));
        }
    }
    /// Reinterpret an i64 register (an f64 bit pattern) as a native `double` (0016).
    fn as_f64(&mut self, bits: &str) -> String {
        let d = self.t();
        self.line(&format!("{d} = bitcast i64 {bits} to double"));
        d
    }
    /// Reinterpret a native `double` back to its i64 bit pattern (design 0016).
    fn f64_bits(&mut self, d: &str) -> String {
        let b = self.t();
        self.line(&format!("{b} = bitcast double {d} to i64"));
        b
    }
    /// The LLVM scalar type of a float scalar (`float` for `f32`, `double` for f64).
    fn float_llty(sty: ScalarTy) -> &'static str {
        if sty == ScalarTy::F32 { "float" } else { "double" }
    }
    /// Reinterpret an i64 register as a native float of width `sty` (design 0016).
    /// For `f32` the 32-bit pattern lives in the register's low half.
    fn as_float(&mut self, sty: ScalarTy, bits: &str) -> String {
        if sty == ScalarTy::F32 {
            let w = self.t();
            self.line(&format!("{w} = trunc i64 {bits} to i32"));
            let f = self.t();
            self.line(&format!("{f} = bitcast i32 {w} to float"));
            f
        } else {
            self.as_f64(bits)
        }
    }
    /// Reinterpret a native float of width `sty` back to its (zero-extended) i64
    /// bit-pattern register value (design 0016).
    fn float_bits(&mut self, sty: ScalarTy, f: &str) -> String {
        if sty == ScalarTy::F32 {
            let w = self.t();
            self.line(&format!("{w} = bitcast float {f} to i32"));
            let z = self.t();
            self.line(&format!("{z} = zext i32 {w} to i64"));
            z
        } else {
            self.f64_bits(f)
        }
    }

    /// Byte-copy `len` bytes src -> dst within the flat model (mirrors lower::call_copy).
    fn rt_copy(&mut self, dst: &str, src: &str, len: u64) {
        if len == 0 {
            return;
        }
        self.line(&format!("call void @rt_copy(i64 {dst}, i64 {src}, i64 {len})"));
    }

    // ---- local access (tier-aware) ----

    /// Read local `id`'s current i64 value: a `load` from its Tier-R alloca or a
    /// `load_scalar` from its Tier-F flat slot.
    fn load_local(&mut self, id: usize) -> String {
        if self.tier_f[id] {
            let sty = scalar_of(&self.mf.locals[id].ty);
            let off = format!("%off{id}");
            self.load_scalar(&off, sty)
        } else {
            let r = self.t();
            self.line(&format!("{r} = load i64, ptr %loc{id}"));
            r
        }
    }
    /// Write i64 `val` into local `id`'s Tier-R alloca or Tier-F flat slot.
    fn store_local(&mut self, id: usize, val: &str) {
        if self.tier_f[id] {
            let sty = scalar_of(&self.mf.locals[id].ty);
            let off = format!("%off{id}");
            self.store_scalar(&off, val, sty);
        } else {
            self.line(&format!("store i64 {val}, ptr %loc{id}"));
        }
    }

    /// A scalar operand as an LLVM i64 value (an immediate literal or a load of the
    /// local's canonical i64).
    fn operand(&mut self, op: &Operand) -> String {
        match op {
            Operand::Const(v, _) => format!("{}", *v as i64),
            Operand::Local(id) => self.load_local(*id),
        }
    }
    fn operand_sty(&self, op: &Operand) -> ScalarTy {
        match op {
            Operand::Const(_, s) => *s,
            Operand::Local(id) => scalar_of(&self.mf.locals[*id].ty),
        }
    }

    /// Resolve a place to its flat candor address (i64), faulting on an OOB index
    /// (INV-CHECK). Mirrors `lower::place_addr`. A Tier-R root is only valid with a
    /// leading `Deref` (its pointer value is the starting address).
    fn place_addr(&mut self, place: &Place) -> Result<(String, Type), String> {
        let root = place.root;
        let (mut addr, mut ty, rest): (String, Type, &[Proj]) = if self.tier_f[root] {
            (format!("%off{root}"), self.mf.locals[root].ty.clone(), &place.proj[..])
        } else {
            match place.proj.first() {
                Some(Proj::Deref { inner }) => {
                    let v = self.load_local(root);
                    (v, inner.clone(), &place.proj[1..])
                }
                _ => return Err("out of LLVM-S3 subset: address of a register local".to_string()),
            }
        };
        for p in rest {
            match p {
                Proj::Field { offset, ty: fty } => {
                    let a = self.t();
                    self.line(&format!("{a} = add i64 {addr}, {offset}"));
                    addr = a;
                    ty = fty.clone();
                }
                Proj::Deref { inner } => {
                    addr = self.load_scalar(&addr, ScalarTy::U64);
                    ty = inner.clone();
                }
                Proj::Index { index, stride, len, span, slice } => {
                    let i = self.operand(index);
                    if *slice {
                        let base = self.load_scalar(&addr, ScalarTy::U64);
                        let lenaddr = self.t();
                        self.line(&format!("{lenaddr} = add i64 {addr}, 8"));
                        let n = self.load_scalar(&lenaddr, ScalarTy::U64);
                        let oob = self.t();
                        self.line(&format!("{oob} = icmp uge i64 {i}, {n}"));
                        self.fault_if(&oob, FaultKind::Bounds, *span);
                        let off = self.t();
                        self.line(&format!("{off} = mul i64 {i}, {stride}"));
                        let a = self.t();
                        self.line(&format!("{a} = add i64 {base}, {off}"));
                        addr = a;
                    } else {
                        let oob = self.t();
                        self.line(&format!("{oob} = icmp uge i64 {i}, {len}"));
                        self.fault_if(&oob, FaultKind::Bounds, *span);
                        let off = self.t();
                        self.line(&format!("{off} = mul i64 {i}, {stride}"));
                        let a = self.t();
                        self.line(&format!("{a} = add i64 {addr}, {off}"));
                        addr = a;
                    }
                    ty = match &ty {
                        Type::Array(e, _) => (**e).clone(),
                        Type::Slice(e) | Type::SliceMut(e) => (**e).clone(),
                        // `str[i]` -> the byte `u8` (design 0013 §3).
                        Type::Str => Type::Scalar(ScalarTy::U8),
                        _ => Type::Error,
                    };
                }
            }
        }
        Ok((addr, ty))
    }

    /// Reduce an i64 to the canonical i64 of `sty` (trunc then sign/zero extend) —
    /// mirrors `lower::canon`.
    fn canon(&mut self, v: &str, sty: ScalarTy) -> String {
        let (_, _, bits, signed) = ty_range(sty);
        if bits >= 64 {
            return v.to_string();
        }
        let tr = self.t();
        self.line(&format!("{tr} = trunc i64 {v} to i{bits}"));
        let ex = self.t();
        let opn = if signed { "sext" } else { "zext" };
        self.line(&format!("{ex} = {opn} i{bits} {tr} to i64"));
        ex
    }
    fn ext128(&mut self, v: &str, sty: ScalarTy) -> String {
        let (_, _, _, signed) = ty_range(sty);
        let r = self.t();
        let opn = if signed { "sext" } else { "zext" };
        self.line(&format!("{r} = {opn} i64 {v} to i128"));
        r
    }
    fn fit128(&mut self, v128: &str, sty: ScalarTy) -> String {
        let t = self.t();
        self.line(&format!("{t} = trunc i128 {v128} to i64"));
        self.canon(&t, sty)
    }

    /// Emit `call rt_fault(k, s, e)` + `unreachable` (INV-FAULT-ID).
    fn emit_fault(&mut self, kind: FaultKind, span: Span) {
        self.line(&format!(
            "call void @rt_fault(i32 {}, i32 {}, i32 {})",
            kind_code(kind),
            span.start,
            span.end
        ));
        self.line("unreachable");
    }
    /// Branch to a fresh fault block when `cond` (i1) is true; continue otherwise.
    fn fault_if(&mut self, cond: &str, kind: FaultKind, span: Span) {
        let fl = self.l();
        let cont = self.l();
        self.line(&format!("br i1 {cond}, label %{fl}, label %{cont}"));
        self.label(&fl);
        self.emit_fault(kind, span);
        self.label(&cont);
    }

    /// Range-check the i128 value against `sty`; deliver/fit per regime (add/sub/
    /// mul/neg/conv share this — INV-CHECK; mirrors `lower::range_or_fit`).
    fn range_or_fit(
        &mut self,
        v128: &str,
        sty: ScalarTy,
        regime: Regime,
        fault: Option<&FaultEdge>,
    ) -> Result<String, String> {
        let (min, max, _, _) = ty_range(sty);
        match regime {
            Regime::Checked => {
                let lt = self.t();
                self.line(&format!("{lt} = icmp slt i128 {v128}, {min}"));
                let gt = self.t();
                self.line(&format!("{gt} = icmp sgt i128 {v128}, {max}"));
                let bad = self.t();
                self.line(&format!("{bad} = or i1 {lt}, {gt}"));
                let edge = fault.ok_or("INV-CHECK: checked op lacks its fault edge")?;
                self.fault_if(&bad, edge.kind, edge.span);
                Ok(self.fit128(v128, sty))
            }
            Regime::Wrapping => Ok(self.fit128(v128, sty)),
            Regime::Saturating => {
                let lt = self.t();
                self.line(&format!("{lt} = icmp slt i128 {v128}, {min}"));
                let gt = self.t();
                self.line(&format!("{gt} = icmp sgt i128 {v128}, {max}"));
                let fit = self.fit128(v128, sty);
                let lo = self.t();
                self.line(&format!("{lo} = select i1 {lt}, i64 {min}, i64 {fit}"));
                let r = self.t();
                self.line(&format!("{r} = select i1 {gt}, i64 {max}, i64 {lo}"));
                Ok(r)
            }
        }
    }

    /// Lower a foreign `extern "C"` call (design 0011 §5, the standalone-binary
    /// trust boundary; mirrors `lower::lower_extern_call`). Marshal each arg per the
    /// SysV C ABI — a `rawptr`/borrow is a flat-buffer OFFSET translated to the REAL
    /// host pointer (`MEM_BASE + offset`) libc needs; a <=32-bit scalar is narrowed to
    /// its `i32` register width — call the imported C symbol, and canonicalize a
    /// sub-word return back to the i64 word.
    fn extern_call(&mut self, name: &str, args: &[Operand]) -> Result<String, String> {
        let es = self.lay.items.externs[name].clone();
        let mut argv: Vec<String> = Vec::with_capacity(args.len());
        for (i, a) in args.iter().enumerate() {
            let v = self.operand(a);
            let marshalled = match es.params.get(i).map(|p| &p.lowered) {
                Some(Type::RawPtr(_)) | Some(Type::FnPtr(_)) | Some(Type::Borrow(_))
                | Some(Type::BorrowMut(_)) => {
                    let h = self.t();
                    self.line(&format!("{h} = add i64 {MEM_BASE}, {v}"));
                    format!("i64 {h}")
                }
                Some(t) if c_abi_llty(t) == "i32" => {
                    let r = self.t();
                    self.line(&format!("{r} = trunc i64 {v} to i32"));
                    format!("i32 {r}")
                }
                _ => format!("i64 {v}"),
            };
            argv.push(marshalled);
        }
        let csym = super::lower::c_symbol_name(name);
        let argstr = argv.join(", ");
        if matches!(es.ret, Type::Scalar(ScalarTy::Unit)) {
            self.line(&format!("call void @\"{csym}\"({argstr})"));
            return Ok("0".to_string());
        }
        let rllty = c_abi_llty(&es.ret);
        let raw = self.t();
        self.line(&format!("{raw} = call {rllty} @\"{csym}\"({argstr})"));
        if rllty == "i32" {
            if let Type::Scalar(sc) = &es.ret {
                let (_, _, _, signed) = ty_range(*sc);
                let ex = self.t();
                let opn = if signed { "sext" } else { "zext" };
                self.line(&format!("{ex} = {opn} i32 {raw} to i64"));
                return Ok(ex);
            }
        }
        Ok(raw)
    }

    fn eval_rvalue(&mut self, rv: &Rvalue) -> Result<String, String> {
        match rv {
            Rvalue::Use(op) => Ok(self.operand(op)),
            Rvalue::Ref(place) => Ok(self.place_addr(place)?.0),
            Rvalue::Load { place, ty } => {
                if place.proj.is_empty() {
                    Ok(self.load_local(place.root))
                } else {
                    let (a, _) = self.place_addr(place)?;
                    Ok(self.load_scalar(&a, scalar_of(ty)))
                }
            }
            Rvalue::Cmp { op, l, r } => {
                let lsty = self.operand_sty(l);
                let rsty = self.operand_sty(r);
                let lv = self.operand(l);
                let rv = self.operand(r);
                if lsty.is_float() || rsty.is_float() {
                    // IEEE compare: ordered predicates (`oeq`/`olt`/…) are false on a
                    // NaN operand; `une` (unordered-or-not-equal) makes NaN != NaN true.
                    // Both operands share the same float type (checker-guaranteed).
                    let sty = if lsty.is_float() { lsty } else { rsty };
                    let ll = Self::float_llty(sty);
                    let fa = self.as_float(sty, &lv);
                    let fb = self.as_float(sty, &rv);
                    let cc = match op {
                        BinOp::Eq => "oeq",
                        BinOp::Ne => "une",
                        BinOp::Lt => "olt",
                        BinOp::Le => "ole",
                        BinOp::Gt => "ogt",
                        BinOp::Ge => "oge",
                        _ => return Err(format!("non-comparison {op:?} in Cmp")),
                    };
                    let c = self.t();
                    self.line(&format!("{c} = fcmp {cc} {ll} {fa}, {fb}"));
                    let r = self.t();
                    self.line(&format!("{r} = zext i1 {c} to i64"));
                    return Ok(r);
                }
                let l128 = self.ext128(&lv, lsty);
                let r128 = self.ext128(&rv, rsty);
                let cc = match op {
                    BinOp::Eq => "eq",
                    BinOp::Ne => "ne",
                    BinOp::Lt => "slt",
                    BinOp::Le => "sle",
                    BinOp::Gt => "sgt",
                    BinOp::Ge => "sge",
                    _ => return Err(format!("non-comparison {op:?} in Cmp")),
                };
                let c = self.t();
                self.line(&format!("{c} = icmp {cc} i128 {l128}, {r128}"));
                let r = self.t();
                self.line(&format!("{r} = zext i1 {c} to i64"));
                Ok(r)
            }
            Rvalue::Bin { op, regime, ty, l, r, span, fault } => {
                self.eval_bin(*op, *regime, *ty, l, r, *span, fault.as_ref())
            }
            Rvalue::Un { op, regime, ty, v, fault } => {
                let x = self.operand(v);
                match op {
                    UnOp::Not => {
                        let c = self.t();
                        self.line(&format!("{c} = icmp eq i64 {x}, 0"));
                        let r = self.t();
                        self.line(&format!("{r} = zext i1 {c} to i64"));
                        Ok(r)
                    }
                    UnOp::BitNot => {
                        let n = self.t();
                        self.line(&format!("{n} = xor i64 {x}, -1"));
                        Ok(self.canon(&n, *ty))
                    }
                    UnOp::Neg if ty.is_float() => {
                        let ll = Self::float_llty(*ty);
                        let f = self.as_float(*ty, &x);
                        let n = self.t();
                        self.line(&format!("{n} = fneg {ll} {f}"));
                        Ok(self.float_bits(*ty, &n))
                    }
                    UnOp::Neg => {
                        let x128 = self.ext128(&x, *ty);
                        let neg = self.t();
                        self.line(&format!("{neg} = sub i128 0, {x128}"));
                        self.range_or_fit(&neg, *ty, *regime, fault.as_ref())
                    }
                }
            }
            Rvalue::Conv { to, regime, v, fault } => {
                let sty = self.operand_sty(v);
                let x = self.operand(v);
                if to.is_float() || sty.is_float() {
                    return self.eval_float_conv(sty, *to, &x);
                }
                let x128 = self.ext128(&x, sty);
                self.range_or_fit(&x128, *to, *regime, fault.as_ref())
            }
            Rvalue::Bitcast { to, v } => {
                // Pure bit reinterpretation in the i64-register model (design 0016
                // section 10): re-canonicalize the operand's held bit pattern to the
                // target width/signedness (`trunc`+`sext`/`zext` at 32 bits, no-op at
                // 64) -- NOT an `fpto*`/`*tofp`. Never faults.
                let x = self.operand(v);
                Ok(self.canon(&x, *to))
            }
            Rvalue::Sqrt { ty, v } => {
                // Native IEEE square root via the `llvm.sqrt` intrinsic (design 0016
                // §11): bitcast the operand's pattern to a float, call the intrinsic,
                // bitcast back. Total -- never faults.
                let ll = Self::float_llty(*ty);
                let suf = if *ty == ScalarTy::F32 { "f32" } else { "f64" };
                let x = self.operand(v);
                let f = self.as_float(*ty, &x);
                let s = self.t();
                self.line(&format!("{s} = call {ll} @llvm.sqrt.{suf}({ll} {f})"));
                Ok(self.float_bits(*ty, &s))
            }
            Rvalue::Call { func, args } => {
                // A boundary `extern` call (design 0011 §5) targets an imported C
                // symbol with C-ABI marshalling, not a `cnf_` body.
                if self.lay.items.externs.contains_key(func) {
                    return self.extern_call(func, args);
                }
                let vals: Vec<String> = args.iter().map(|a| self.operand(a)).collect();
                let argstr = vals
                    .iter()
                    .map(|v| format!("i64 {v}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                let r = self.t();
                self.line(&format!("{r} = call i64 {}({argstr})", cnf_sym(func)));
                Ok(r)
            }
            // The static's / interned string's baked Candor address (an
            // immediate `u64` into the static region, whose bytes/initializers
            // `candor_entry`'s prologue lays down before `main`).
            Rvalue::StaticAddr(name) => {
                let a = *self
                    .statics
                    .get(name)
                    .ok_or_else(|| format!("unknown static `{name}`"))?;
                Ok(format!("{}", a as i64))
            }
            Rvalue::StrAddr(sv) => {
                let a = *self
                    .strings
                    .get(sv)
                    .ok_or_else(|| format!("unknown string literal `{sv}`"))?;
                Ok(format!("{}", a as i64))
            }
            Rvalue::IsNull(op) => {
                let v = self.operand(op);
                let c = self.t();
                self.line(&format!("{c} = icmp eq i64 {v}, 0"));
                let r = self.t();
                self.line(&format!("{r} = zext i1 {c} to i64"));
                Ok(r)
            }
            Rvalue::PtrArith { base, index, stride } => {
                let b = self.operand(base);
                let i = self.operand(index);
                let m = self.t();
                self.line(&format!("{m} = mul i64 {i}, {stride}"));
                let r = self.t();
                self.line(&format!("{r} = add i64 {b}, {m}"));
                Ok(r)
            }
            // An indirect call through a fn-pointer id (design 0007 §6.2): resolve
            // the id to a callee address via the dispatch table, then call.
            Rvalue::CallIndirect { func, args } => {
                let id = self.operand(func);
                let vals: Vec<String> = args.iter().map(|a| self.operand(a)).collect();
                Ok(self.call_fnptr_id(&id, &vals))
            }
        }
    }

    /// A numeric `conv` where the source and/or target is a float (design 0016 §5).
    /// int->float: `sitofp`/`uitofp` (rounds). `f32`->`f64`: `fpext` (exact);
    /// `f64`->`f32`: `fptrunc` (rounds). float->int: `llvm.fpto{s,u}i.sat` to the
    /// target width (truncate toward zero, saturating; NaN->0), then extend to i64.
    fn eval_float_conv(&mut self, from: ScalarTy, to: ScalarTy, x: &str) -> Result<String, String> {
        // float -> float (widen/narrow), or a same-width identity.
        if from.is_float() && to.is_float() {
            if from == to {
                return Ok(x.to_string());
            }
            let f = self.as_float(from, x);
            let g = self.t();
            if to == ScalarTy::F64 {
                self.line(&format!("{g} = fpext float {f} to double"));
            } else {
                self.line(&format!("{g} = fptrunc double {f} to float"));
            }
            return Ok(self.float_bits(to, &g));
        }
        // int -> float.
        if to.is_float() {
            let ll = Self::float_llty(to);
            let (_, _, _, signed) = ty_range(from);
            let opn = if signed { "sitofp" } else { "uitofp" };
            let d = self.t();
            self.line(&format!("{d} = {opn} i64 {x} to {ll}"));
            return Ok(self.float_bits(to, &d));
        }
        // float -> int.
        let ll = Self::float_llty(from);
        let suf = if from == ScalarTy::F32 { "f32" } else { "f64" };
        let (_, _, bits, signed) = ty_range(to);
        let f = self.as_float(from, x);
        let intr = if signed { "fptosi" } else { "fptoui" };
        let narrow = self.t();
        self.line(&format!(
            "{narrow} = call i{bits} @llvm.{intr}.sat.i{bits}.{suf}({ll} {f})"
        ));
        if bits >= 64 {
            Ok(narrow)
        } else {
            let ex = self.t();
            let e = if signed { "sext" } else { "zext" };
            self.line(&format!("{ex} = {e} i{bits} {narrow} to i64"));
            Ok(ex)
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn eval_bin(
        &mut self,
        op: BinOp,
        regime: Regime,
        ty: ScalarTy,
        l: &Operand,
        r: &Operand,
        span: Span,
        fault: Option<&FaultEdge>,
    ) -> Result<String, String> {
        use BinOp::*;
        let lv = self.operand(l);
        let rv = self.operand(r);
        if ty.is_float() {
            // IEEE-754 arithmetic: bit-cast, native op, bit-cast back. Never faults.
            let ll = Self::float_llty(ty);
            let fa = self.as_float(ty, &lv);
            let fb = self.as_float(ty, &rv);
            let opn = match op {
                Add => "fadd",
                Sub => "fsub",
                Mul => "fmul",
                Div => "fdiv",
                _ => return Err(format!("only + - * / reach a float Bin, got {op:?}")),
            };
            let r = self.t();
            self.line(&format!("{r} = {opn} {ll} {fa}, {fb}"));
            return Ok(self.float_bits(ty, &r));
        }
        let (min, max, bits, signed) = ty_range(ty);
        match op {
            Add | Sub | Mul => match regime {
                Regime::Checked => {
                    let (a, b) = if bits >= 64 {
                        (lv.clone(), rv.clone())
                    } else {
                        let a = self.t();
                        self.line(&format!("{a} = trunc i64 {lv} to i{bits}"));
                        let b = self.t();
                        self.line(&format!("{b} = trunc i64 {rv} to i{bits}"));
                        (a, b)
                    };
                    let name = overflow_intrinsic(op, signed, bits);
                    let p = self.t();
                    self.line(&format!(
                        "{p} = call {{i{bits}, i1}} @{name}(i{bits} {a}, i{bits} {b})"
                    ));
                    let res = self.t();
                    self.line(&format!("{res} = extractvalue {{i{bits}, i1}} {p}, 0"));
                    let ovf = self.t();
                    self.line(&format!("{ovf} = extractvalue {{i{bits}, i1}} {p}, 1"));
                    let edge = fault.ok_or("INV-CHECK: checked arith lacks its fault edge")?;
                    self.fault_if(&ovf, edge.kind, edge.span);
                    if bits >= 64 {
                        Ok(res)
                    } else {
                        let ex = self.t();
                        let e = if signed { "sext" } else { "zext" };
                        self.line(&format!("{ex} = {e} i{bits} {res} to i64"));
                        Ok(ex)
                    }
                }
                Regime::Wrapping => {
                    let opn = match op { Add => "add", Sub => "sub", _ => "mul" };
                    if bits >= 64 {
                        let r = self.t();
                        self.line(&format!("{r} = {opn} i64 {lv}, {rv}"));
                        Ok(r)
                    } else {
                        let a = self.t();
                        self.line(&format!("{a} = trunc i64 {lv} to i{bits}"));
                        let b = self.t();
                        self.line(&format!("{b} = trunc i64 {rv} to i{bits}"));
                        let w = self.t();
                        self.line(&format!("{w} = {opn} i{bits} {a}, {b}"));
                        let ex = self.t();
                        let e = if signed { "sext" } else { "zext" };
                        self.line(&format!("{ex} = {e} i{bits} {w} to i64"));
                        Ok(ex)
                    }
                }
                Regime::Saturating => {
                    let a128 = self.ext128(&lv, ty);
                    let b128 = self.ext128(&rv, ty);
                    let opn = match op { Add => "add", Sub => "sub", _ => "mul" };
                    let res = self.t();
                    self.line(&format!("{res} = {opn} i128 {a128}, {b128}"));
                    self.range_or_fit(&res, ty, Regime::Saturating, None)
                }
            },
            Div | Rem => {
                let z = self.t();
                self.line(&format!("{z} = icmp eq i64 {rv}, 0"));
                self.fault_if(&z, FaultKind::DivByZero, span);
                if !signed {
                    let opn = if op == Div { "udiv" } else { "urem" };
                    let q = self.t();
                    self.line(&format!("{q} = {opn} i64 {lv}, {rv}"));
                    Ok(self.canon(&q, ty))
                } else {
                    let ismin = self.t();
                    self.line(&format!("{ismin} = icmp eq i64 {lv}, {min}"));
                    let ism1 = self.t();
                    self.line(&format!("{ism1} = icmp eq i64 {rv}, -1"));
                    let ov = self.t();
                    self.line(&format!("{ov} = and i1 {ismin}, {ism1}"));
                    if op == Rem {
                        let safe = self.t();
                        self.line(&format!("{safe} = select i1 {ov}, i64 1, i64 {rv}"));
                        let rr = self.t();
                        self.line(&format!("{rr} = srem i64 {lv}, {safe}"));
                        let sel = self.t();
                        self.line(&format!("{sel} = select i1 {ov}, i64 0, i64 {rr}"));
                        Ok(self.canon(&sel, ty))
                    } else {
                        match regime {
                            Regime::Checked => {
                                let edge =
                                    fault.ok_or("INV-CHECK: checked div lacks its fault edge")?;
                                self.fault_if(&ov, edge.kind, edge.span);
                                let q = self.t();
                                self.line(&format!("{q} = sdiv i64 {lv}, {rv}"));
                                Ok(self.canon(&q, ty))
                            }
                            Regime::Wrapping => {
                                let safe = self.t();
                                self.line(&format!("{safe} = select i1 {ov}, i64 1, i64 {rv}"));
                                let q = self.t();
                                self.line(&format!("{q} = sdiv i64 {lv}, {safe}"));
                                let sel = self.t();
                                self.line(&format!("{sel} = select i1 {ov}, i64 {min}, i64 {q}"));
                                Ok(self.canon(&sel, ty))
                            }
                            Regime::Saturating => {
                                let safe = self.t();
                                self.line(&format!("{safe} = select i1 {ov}, i64 1, i64 {rv}"));
                                let q = self.t();
                                self.line(&format!("{q} = sdiv i64 {lv}, {safe}"));
                                let sel = self.t();
                                self.line(&format!("{sel} = select i1 {ov}, i64 {max}, i64 {q}"));
                                Ok(self.canon(&sel, ty))
                            }
                        }
                    }
                }
            }
            BitAnd | BitOr | BitXor => {
                let opn = match op { BitAnd => "and", BitOr => "or", _ => "xor" };
                let r = self.t();
                self.line(&format!("{r} = {opn} i64 {lv}, {rv}"));
                Ok(self.canon(&r, ty))
            }
            Shl | Shr => {
                let neg = self.t();
                self.line(&format!("{neg} = icmp slt i64 {rv}, 0"));
                self.fault_if(&neg, FaultKind::Overflow, span);
                let ge = self.t();
                self.line(&format!("{ge} = icmp uge i64 {rv}, {bits}"));
                let amt = match regime {
                    Regime::Checked => {
                        self.fault_if(&ge, FaultKind::Overflow, span);
                        rv.clone()
                    }
                    Regime::Wrapping => {
                        let m = self.t();
                        self.line(&format!("{m} = urem i64 {rv}, {bits}"));
                        let a = self.t();
                        self.line(&format!("{a} = select i1 {ge}, i64 {m}, i64 {rv}"));
                        a
                    }
                    Regime::Saturating => {
                        let a = self.t();
                        self.line(&format!("{a} = select i1 {ge}, i64 {}, i64 {rv}", bits - 1));
                        a
                    }
                };
                let opn = if op == Shl {
                    "shl"
                } else if signed {
                    "ashr"
                } else {
                    "lshr"
                };
                let raw = self.t();
                self.line(&format!("{raw} = {opn} i64 {lv}, {amt}"));
                Ok(self.canon(&raw, ty))
            }
            _ => Err(format!("comparison/logical {op:?} in Bin")),
        }
    }

    // ---- static drop schedule (INV-DROP; mirrors `lower::emit_drop`) ----

    /// Whether a type carries a drop obligation — mirror of `lower::needs_drop`
    /// (array of droppable / Box / BoxResult / a struct with a drop hook or a
    /// droppable field / an enum with a droppable variant payload).
    fn needs_drop(&self, ty: &Type) -> bool {
        match ty {
            Type::Array(e, _) => self.needs_drop(e),
            Type::Box(_) | Type::BoxResult(_) => true,
            // Compiler-known collections own a heap buffer freed on drop.
            Type::App(n, _) if n == "Vec" || n == "Map" => true,
            Type::Named(n) if n == "String" => true,
            Type::Named(n) if self.lay.items.lookup_struct(n).is_some() => {
                if self.drop_hooks.contains_key(n) {
                    return true;
                }
                let (fields, _, _) = self.lay.struct_layout(n);
                fields.iter().any(|(_, t, _)| self.needs_drop(t))
            }
            Type::Named(n) if self.lay.items.lookup_enum(n).is_some() => match self.lay.enum_info(ty) {
                Some(vs) => vs.iter().any(|(_, ps)| ps.iter().any(|p| self.needs_drop(p))),
                None => false,
            },
            _ => false,
        }
    }

    /// `add i64 candor, off` — a byte offset into a flat candor address.
    fn add_off_candor(&mut self, addr: &str, off: u64) -> String {
        if off == 0 {
            return addr.to_string();
        }
        let a = self.t();
        self.line(&format!("{a} = add i64 {addr}, {off}"));
        a
    }

    /// Recursively drop the value of `ty` at flat candor address `addr`, in
    /// `lower::emit_drop`'s exact order: array/struct fields inner-to-outer (reverse
    /// declaration order), the active enum variant's payload, and the struct drop
    /// hook fired at trace time. `moved` is the static move mask (field-name paths
    /// already moved out); `path` is the current sub-path. Box/BoxResult drop needs
    /// the allocator (S4) and is rejected precisely.
    fn emit_drop(
        &mut self,
        addr: &str,
        ty: &Type,
        moved: &[Vec<String>],
        path: &mut Vec<String>,
    ) -> Result<(), String> {
        if is_moved(moved, path) || !self.needs_drop(ty) {
            return Ok(());
        }
        match ty {
            Type::Array(elem, len) => {
                let n = self.lay.array_len(len);
                let stride = round_up(self.lay.size_of(elem), self.lay.align_of(elem));
                for i in (0..n).rev() {
                    let ea = self.add_off_candor(addr, i * stride);
                    self.emit_drop(&ea, elem, moved, path)?;
                }
            }
            Type::Box(inner) => self.drop_box(addr, inner),
            Type::BoxResult(_) => self.drop_enum(addr, ty, moved, path)?,
            // Compiler-known collections: drop live elements/values, free the
            // buffer through the carried allocator (mirror of `mir::interp`).
            // Precedes the struct arm — `String` is a synthesized nominal struct.
            Type::App(n, args) if n == "Vec" => {
                let elem = args.first().cloned().unwrap_or(Type::Error);
                self.drop_vec(addr, &elem)?;
            }
            Type::App(n, args) if n == "Map" => {
                let valty = args.first().cloned().unwrap_or(Type::Error);
                self.drop_map(addr, &valty)?;
            }
            Type::Named(n) if n == "String" => self.drop_string(addr),
            Type::Named(n) if self.lay.items.lookup_struct(n).is_some() => {
                // A partially-moved struct skips its whole-value hook (the moved
                // field carried the ownership the hook would observe).
                if !partially(moved, path) {
                    if let Some(hook) = self.drop_hooks.get(n).cloned() {
                        let r = self.t();
                        self.line(&format!("{r} = call i64 {}(i64 {addr})", cnf_sym(&hook)));
                    }
                }
                let (fields, _, _) = self.lay.struct_layout(n);
                for (fname, fty, off) in fields.into_iter().rev() {
                    path.push(fname);
                    let fa = self.add_off_candor(addr, off);
                    self.emit_drop(&fa, &fty, moved, path)?;
                    path.pop();
                }
            }
            Type::Named(_) => self.drop_enum(addr, ty, moved, path)?,
            _ => {}
        }
        Ok(())
    }

    /// Drop an enum by switching on its runtime tag (offset 0) to the active
    /// variant, then dropping that variant's payload fields in reverse — mirror of
    /// `lower::drop_enum` (variants with no droppable payload emit no test).
    fn drop_enum(
        &mut self,
        addr: &str,
        ty: &Type,
        moved: &[Vec<String>],
        path: &mut Vec<String>,
    ) -> Result<(), String> {
        let einfo = match self.lay.enum_info(ty) {
            Some(e) => e,
            None => return Ok(()),
        };
        let tag = self.load_scalar(addr, ScalarTy::U64);
        let merge = self.l();
        for (idx, (_, payloads)) in einfo.iter().enumerate() {
            if !payloads.iter().any(|p| self.needs_drop(p)) {
                continue;
            }
            let isv = self.t();
            self.line(&format!("{isv} = icmp eq i64 {tag}, {idx}"));
            let vblock = self.l();
            let next = self.l();
            self.line(&format!("br i1 {isv}, label %{vblock}, label %{next}"));
            self.label(&vblock);
            for i in (0..payloads.len()).rev() {
                let (pty, off) = self.lay.payload_offset(payloads, i);
                path.push(format!("_{i}"));
                let pa = self.add_off_candor(addr, off);
                self.emit_drop(&pa, &pty, moved, path)?;
                path.pop();
            }
            self.line(&format!("br label %{merge}"));
            self.label(&next);
        }
        self.line(&format!("br label %{merge}"));
        self.label(&merge);
        Ok(())
    }

    // ---- allocator ABI + Box/unbox (design 0010 §5; mirrors lower::box_op) ----

    fn load_u64(&mut self, candor: &str) -> String {
        self.load_scalar(candor, ScalarTy::U64)
    }
    fn store_u64(&mut self, candor: &str, val: &str) {
        self.store_scalar(candor, val, ScalarTy::U64);
    }
    fn field_off(&self, sname: &str, field: &str) -> u64 {
        self.lay.field_offset(sname, field).map(|(_, o)| o).unwrap_or(0)
    }

    /// Structurally identify the allocator handle / vtable structs (design 0010
    /// §5; identical to `lower`/`mir::interp`), so box/unbox find `ctx`/`vt`/
    /// `alloc`/`free` regardless of module qualification.
    fn alloc_vtable_name(&self) -> String {
        self.lay
            .items
            .structs
            .iter()
            .find(|(_, s)| {
                s.fields.iter().any(|(n, t)| n == "alloc" && matches!(t, Type::FnPtr(_)))
                    && s.fields.iter().any(|(n, t)| n == "free" && matches!(t, Type::FnPtr(_)))
            })
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| "AllocVtable".to_string())
    }
    fn alloc_struct_name(&self) -> String {
        let vt = self.alloc_vtable_name();
        self.lay
            .items
            .structs
            .iter()
            .find(|(_, s)| {
                s.fields.iter().any(|(n, t)| {
                    n == "vt"
                        && matches!(t, Type::RawPtr(inner) if matches!(&**inner, Type::Named(x) if *x == vt))
                })
            })
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| "Alloc".to_string())
    }

    /// Indirect call by fn-pointer id: index the dispatch table for the callee's
    /// address, then `call` it (mirrors `lower::call_fnptr_id`).
    fn call_fnptr_id(&mut self, id: &str, args: &[String]) -> String {
        let slot = self.t();
        self.line(&format!("{slot} = getelementptr ptr, ptr @\"cn_fnptr_table\", i64 {id}"));
        let fp = self.t();
        self.line(&format!("{fp} = load ptr, ptr {slot}"));
        let argstr =
            args.iter().map(|a| format!("i64 {a}")).collect::<Vec<_>>().join(", ");
        let r = self.t();
        self.line(&format!("{r} = call i64 {fp}({argstr})"));
        r
    }

    /// The `String` collection intrinsics (design 0013), lowered inline to mirror
    /// `mir::interp::collection_op` byte-for-byte (the twin of `lower::collection_op`).
    fn collection_op(&mut self, dst: &Place, op: &CollOp) -> Result<(), String> {
        match op {
            CollOp::New { alloc } => {
                let alloc_addr = self.operand(alloc);
                let astruct = self.alloc_struct_name();
                let ctx_off = self.field_off(&astruct, "ctx");
                let vt_off = self.field_off(&astruct, "vt");
                let ca = self.add_off_candor(&alloc_addr, ctx_off);
                let ctx = self.load_u64(&ca);
                let va = self.add_off_candor(&alloc_addr, vt_off);
                let vt = self.load_u64(&va);
                let (daddr, _) = self.place_addr(dst)?;
                self.store_u64(&daddr, "0");
                let d8 = self.add_off_candor(&daddr, 8);
                self.store_u64(&d8, "0");
                let d16 = self.add_off_candor(&daddr, 16);
                self.store_u64(&d16, "0");
                let d24 = self.add_off_candor(&daddr, 24);
                self.store_u64(&d24, &ctx);
                let d32 = self.add_off_candor(&daddr, 32);
                self.store_u64(&d32, &vt);
            }
            CollOp::StringAsStr { base } => {
                let base = self.operand(base);
                let buf = self.load_u64(&base);
                let b8 = self.add_off_candor(&base, 8);
                let len = self.load_u64(&b8);
                let (daddr, _) = self.place_addr(dst)?;
                self.store_u64(&daddr, &buf);
                let d8 = self.add_off_candor(&daddr, 8);
                self.store_u64(&d8, &len);
            }
            CollOp::StringAppend { base, view, span } => {
                let base = self.operand(base);
                let (vaddr, _) = self.place_addr(view)?;
                let ptr = self.load_u64(&vaddr);
                let v8 = self.add_off_candor(&vaddr, 8);
                let slen = self.load_u64(&v8);
                self.string_reserve(&base, &slen, *span);
                let buf = self.load_u64(&base);
                let b8 = self.add_off_candor(&base, 8);
                let len = self.load_u64(&b8);
                let dstp = self.t();
                self.line(&format!("{dstp} = add i64 {buf}, {len}"));
                self.rt_copy_val(&dstp, &ptr, &slen);
                let newlen = self.t();
                self.line(&format!("{newlen} = add i64 {len}, {slen}"));
                self.store_u64(&b8, &newlen);
            }
            CollOp::StringPush { base, ch, span } => {
                let base = self.operand(base);
                let c = self.operand(ch);
                // Reject non-scalar-values (surrogates, > 0x10FFFF) exactly as
                // `utf8_encode_scalar`; the interp faults `Requires` at the call span.
                let oor = self.t();
                self.line(&format!("{oor} = icmp ugt i64 {c}, 1114111"));
                let ge_lo = self.t();
                self.line(&format!("{ge_lo} = icmp uge i64 {c}, 55296"));
                let le_hi = self.t();
                self.line(&format!("{le_hi} = icmp ule i64 {c}, 57343"));
                let surr = self.t();
                self.line(&format!("{surr} = and i1 {ge_lo}, {le_hi}"));
                let bad = self.t();
                self.line(&format!("{bad} = or i1 {oor}, {surr}"));
                self.fault_if(&bad, FaultKind::Requires, *span);
                // UTF-8 length: <0x80 -> 1, <0x800 -> 2, <0x10000 -> 3, else 4.
                let lt80 = self.t();
                self.line(&format!("{lt80} = icmp ult i64 {c}, 128"));
                let lt800 = self.t();
                self.line(&format!("{lt800} = icmp ult i64 {c}, 2048"));
                let lt10000 = self.t();
                self.line(&format!("{lt10000} = icmp ult i64 {c}, 65536"));
                let l34 = self.t();
                self.line(&format!("{l34} = select i1 {lt10000}, i64 3, i64 4"));
                let l234 = self.t();
                self.line(&format!("{l234} = select i1 {lt800}, i64 2, i64 {l34}"));
                let enc_len = self.t();
                self.line(&format!("{enc_len} = select i1 {lt80}, i64 1, i64 {l234}"));
                self.string_reserve(&base, &enc_len, *span);
                let buf = self.load_u64(&base);
                let b8 = self.add_off_candor(&base, 8);
                let len = self.load_u64(&b8);
                let dstp = self.t();
                self.line(&format!("{dstp} = add i64 {buf}, {len}"));
                // Write the 1-4 encoded bytes (branch per width, mirroring the
                // 1/2/3/4-byte arms of `utf8_encode_scalar` bit for bit).
                let w1 = self.l();
                let t2 = self.l();
                let w2 = self.l();
                let t3 = self.l();
                let w3 = self.l();
                let w4 = self.l();
                let cont = self.l();
                self.line(&format!("br i1 {lt80}, label %{w1}, label %{t2}"));
                self.label(&w1);
                self.store_byte(&dstp, &c);
                self.line(&format!("br label %{cont}"));
                self.label(&t2);
                self.line(&format!("br i1 {lt800}, label %{w2}, label %{t3}"));
                self.label(&w2);
                let b0 = self.utf8_lead(&c, 6, 0xC0);
                self.store_byte(&dstp, &b0);
                let d1 = self.add_off_candor(&dstp, 1);
                let b1 = self.utf8_cont(&c, 0);
                self.store_byte(&d1, &b1);
                self.line(&format!("br label %{cont}"));
                self.label(&t3);
                self.line(&format!("br i1 {lt10000}, label %{w3}, label %{w4}"));
                self.label(&w3);
                let e0 = self.utf8_lead(&c, 12, 0xE0);
                self.store_byte(&dstp, &e0);
                let e1a = self.add_off_candor(&dstp, 1);
                let e1 = self.utf8_cont(&c, 6);
                self.store_byte(&e1a, &e1);
                let e2a = self.add_off_candor(&dstp, 2);
                let e2 = self.utf8_cont(&c, 0);
                self.store_byte(&e2a, &e2);
                self.line(&format!("br label %{cont}"));
                self.label(&w4);
                let f0 = self.utf8_lead(&c, 18, 0xF0);
                self.store_byte(&dstp, &f0);
                let f1a = self.add_off_candor(&dstp, 1);
                let f1 = self.utf8_cont(&c, 12);
                self.store_byte(&f1a, &f1);
                let f2a = self.add_off_candor(&dstp, 2);
                let f2 = self.utf8_cont(&c, 6);
                self.store_byte(&f2a, &f2);
                let f3a = self.add_off_candor(&dstp, 3);
                let f3 = self.utf8_cont(&c, 0);
                self.store_byte(&f3a, &f3);
                self.line(&format!("br label %{cont}"));
                self.label(&cont);
                let newlen = self.t();
                self.line(&format!("{newlen} = add i64 {len}, {enc_len}"));
                self.store_u64(&b8, &newlen);
            }
            CollOp::VecPush { base, elem, value, span } => {
                let base = self.operand(base);
                let stride = round_up(self.lay.size_of(elem), self.lay.align_of(elem));
                let align = self.lay.align_of(elem);
                self.vec_reserve(&base, stride, align, 1, *span);
                let buf = self.load_u64(&base);
                let b8 = self.add_off_candor(&base, 8);
                let len = self.load_u64(&b8);
                let (vaddr, _) = self.place_addr(value)?;
                let off = self.t();
                self.line(&format!("{off} = mul i64 {len}, {stride}"));
                let slot = self.t();
                self.line(&format!("{slot} = add i64 {buf}, {off}"));
                self.rt_copy(&slot, &vaddr, self.lay.size_of(elem));
                let newlen = self.t();
                self.line(&format!("{newlen} = add i64 {len}, 1"));
                self.store_u64(&b8, &newlen);
            }
            CollOp::VecGet { base, elem, index, span } => {
                let base = self.operand(base);
                let b8 = self.add_off_candor(&base, 8);
                let len = self.load_u64(&b8);
                let i = self.operand(index);
                let oob = self.t();
                self.line(&format!("{oob} = icmp uge i64 {i}, {len}"));
                self.fault_if(&oob, FaultKind::Bounds, *span);
                let stride = round_up(self.lay.size_of(elem), self.lay.align_of(elem));
                let buf = self.load_u64(&base);
                let off = self.t();
                self.line(&format!("{off} = mul i64 {i}, {stride}"));
                let slot = self.t();
                self.line(&format!("{slot} = add i64 {buf}, {off}"));
                let (daddr, _) = self.place_addr(dst)?;
                self.store_u64(&daddr, &slot);
            }
            CollOp::VecSet { base, elem, index, value, span } => {
                let base = self.operand(base);
                let b8 = self.add_off_candor(&base, 8);
                let len = self.load_u64(&b8);
                let i = self.operand(index);
                let oob = self.t();
                self.line(&format!("{oob} = icmp uge i64 {i}, {len}"));
                self.fault_if(&oob, FaultKind::Bounds, *span);
                let stride = round_up(self.lay.size_of(elem), self.lay.align_of(elem));
                let buf = self.load_u64(&base);
                let off = self.t();
                self.line(&format!("{off} = mul i64 {i}, {stride}"));
                let slot = self.t();
                self.line(&format!("{slot} = add i64 {buf}, {off}"));
                let (vaddr, _) = self.place_addr(value)?;
                // Drop-on-overwrite: run the old element's drop glue before the move,
                // mirroring the interp's `drop_value(slot, elem)` in `VecSet`.
                self.emit_drop(&slot, elem, &[], &mut Vec::new())?;
                self.rt_copy(&slot, &vaddr, self.lay.size_of(elem));
            }
            CollOp::VecPop { base, elem } => {
                let base = self.operand(base);
                let (daddr, _) = self.place_addr(dst)?;
                let b8 = self.add_off_candor(&base, 8);
                let len = self.load_u64(&b8);
                // Opt discriminants + the `Some` payload offset are compile-time layout.
                let opt = Type::Named("Opt".to_string());
                let einfo = self.lay.enum_info(&opt).ok_or("unknown enum `Opt`")?;
                let some_idx = einfo.iter().position(|(n, _)| n == "Some").unwrap_or(0);
                let none_idx = einfo.iter().position(|(n, _)| n == "None").unwrap_or(1);
                let some_payloads = einfo[some_idx].1.clone();
                let (_, poff) = self.lay.payload_offset(&some_payloads, 0);
                let is_empty = self.t();
                self.line(&format!("{is_empty} = icmp eq i64 {len}, 0"));
                let none_b = self.l();
                let some_b = self.l();
                let cont = self.l();
                self.line(&format!("br i1 {is_empty}, label %{none_b}, label %{some_b}"));
                self.label(&none_b);
                self.store_u64(&daddr, &format!("{none_idx}"));
                self.line(&format!("br label %{cont}"));
                self.label(&some_b);
                let newlen = self.t();
                self.line(&format!("{newlen} = sub i64 {len}, 1"));
                let stride = round_up(self.lay.size_of(elem), self.lay.align_of(elem));
                let buf = self.load_u64(&base);
                let off = self.t();
                self.line(&format!("{off} = mul i64 {newlen}, {stride}"));
                let src = self.t();
                self.line(&format!("{src} = add i64 {buf}, {off}"));
                self.store_u64(&b8, &newlen);
                self.store_u64(&daddr, &format!("{some_idx}"));
                let pdst = self.add_off_candor(&daddr, poff);
                self.rt_copy(&pdst, &src, self.lay.size_of(elem));
                self.line(&format!("br label %{cont}"));
                self.label(&cont);
            }
            CollOp::MapInsert { base, valty, key, value, span } => {
                let base = self.operand(base);
                let (kaddr, _) = self.place_addr(key)?;
                let kptr = self.load_u64(&kaddr);
                let ka8 = self.add_off_candor(&kaddr, 8);
                let klen = self.load_u64(&ka8);
                let vsz = self.lay.size_of(valty);
                let stride = round_up(24 + vsz, 8);
                let (vaddr, _) = self.place_addr(value)?;
                let buf0 = self.load_u64(&base);
                let b16 = self.add_off_candor(&base, 16);
                let cap0 = self.load_u64(&b16);
                let (found, slot) = self.map_find(&buf0, &cap0, stride, &kptr, &klen);
                let is_found = self.t();
                self.line(&format!("{is_found} = icmp ne i64 {found}, 0"));
                let over = self.l();
                let ins = self.l();
                let done = self.l();
                self.line(&format!("br i1 {is_found}, label %{over}, label %{ins}"));
                // Present key: drop the displaced value (drop-on-overwrite, like
                // `VecSet`), then move the new value into the slot.
                self.label(&over);
                let so = self.t();
                self.line(&format!("{so} = mul i64 {slot}, {stride}"));
                let sb = self.t();
                self.line(&format!("{sb} = add i64 {buf0}, {so}"));
                let voff = self.add_off_candor(&sb, 24);
                self.emit_drop(&voff, valty, &[], &mut Vec::new())?;
                self.rt_copy(&voff, &vaddr, vsz);
                self.line(&format!("br label %{done}"));
                // Absent key: grow/rehash if needed, own a key byte-copy, store.
                self.label(&ins);
                self.map_reserve(&base, valty, *span);
                let buf = self.load_u64(&base);
                let cap = self.load_u64(&b16);
                let eslot = self.map_find_empty(&buf, &cap, stride, &kptr, &klen);
                let b24 = self.add_off_candor(&base, 24);
                let ctx = self.load_u64(&b24);
                let b32 = self.add_off_candor(&base, 32);
                let vt = self.load_u64(&b32);
                let kbuf = self.call_alloc(&ctx, &vt, &klen, 1);
                let oom = self.t();
                self.line(&format!("{oom} = icmp eq i64 {kbuf}, 0"));
                self.fault_if(&oom, FaultKind::Panic, *span);
                self.rt_copy_val(&kbuf, &kptr, &klen);
                let eo = self.t();
                self.line(&format!("{eo} = mul i64 {eslot}, {stride}"));
                let eb = self.t();
                self.line(&format!("{eb} = add i64 {buf}, {eo}"));
                self.store_u64(&eb, "1");
                let eb8 = self.add_off_candor(&eb, 8);
                self.store_u64(&eb8, &kbuf);
                let eb16 = self.add_off_candor(&eb, 16);
                self.store_u64(&eb16, &klen);
                let eb24 = self.add_off_candor(&eb, 24);
                self.rt_copy(&eb24, &vaddr, vsz);
                let b8 = self.add_off_candor(&base, 8);
                let len = self.load_u64(&b8);
                let nlen = self.t();
                self.line(&format!("{nlen} = add i64 {len}, 1"));
                self.store_u64(&b8, &nlen);
                self.line(&format!("br label %{done}"));
                self.label(&done);
            }
            CollOp::MapContains { base, valty, key } => {
                let base = self.operand(base);
                let (kaddr, _) = self.place_addr(key)?;
                let kptr = self.load_u64(&kaddr);
                let ka8 = self.add_off_candor(&kaddr, 8);
                let klen = self.load_u64(&ka8);
                let stride = round_up(24 + self.lay.size_of(valty), 8);
                let buf = self.load_u64(&base);
                let b16 = self.add_off_candor(&base, 16);
                let cap = self.load_u64(&b16);
                let (found, _slot) = self.map_find(&buf, &cap, stride, &kptr, &klen);
                let (daddr, _) = self.place_addr(dst)?;
                self.store_scalar(&daddr, &found, ScalarTy::Bool);
            }
            CollOp::MapGet { base, valty, key, span } => {
                let base = self.operand(base);
                let (kaddr, _) = self.place_addr(key)?;
                let kptr = self.load_u64(&kaddr);
                let ka8 = self.add_off_candor(&kaddr, 8);
                let klen = self.load_u64(&ka8);
                let stride = round_up(24 + self.lay.size_of(valty), 8);
                let buf = self.load_u64(&base);
                let b16 = self.add_off_candor(&base, 16);
                let cap = self.load_u64(&b16);
                let (found, slot) = self.map_find(&buf, &cap, stride, &kptr, &klen);
                let missing = self.t();
                self.line(&format!("{missing} = icmp eq i64 {found}, 0"));
                self.fault_if(&missing, FaultKind::Bounds, *span);
                let so = self.t();
                self.line(&format!("{so} = mul i64 {slot}, {stride}"));
                let sb = self.t();
                self.line(&format!("{sb} = add i64 {buf}, {so}"));
                let voff = self.add_off_candor(&sb, 24);
                let (daddr, _) = self.place_addr(dst)?;
                self.store_u64(&daddr, &voff);
            }
        }
        Ok(())
    }

    /// 64-bit FNV-1a over the `klen` key bytes at candor `kptr` (offset basis
    /// 0xcbf29ce484222325, prime 0x100000001b3) — `mir::interp::map_hash` bit for bit.
    fn map_hash(&mut self, kptr: &str, klen: &str) -> String {
        let pre = self.l();
        let head = self.l();
        let body = self.l();
        let done = self.l();
        let h = self.t();
        let i = self.t();
        let hnext = self.t();
        let inext = self.t();
        self.line(&format!("br label %{pre}"));
        self.label(&pre);
        self.line(&format!("br label %{head}"));
        self.label(&head);
        self.line(&format!("{h} = phi i64 [ -3750763034362895579, %{pre} ], [ {hnext}, %{body} ]"));
        self.line(&format!("{i} = phi i64 [ 0, %{pre} ], [ {inext}, %{body} ]"));
        let more = self.t();
        self.line(&format!("{more} = icmp ult i64 {i}, {klen}"));
        self.line(&format!("br i1 {more}, label %{body}, label %{done}"));
        self.label(&body);
        let bp = self.t();
        self.line(&format!("{bp} = add i64 {kptr}, {i}"));
        let byte = self.load_scalar(&bp, ScalarTy::U8);
        let x = self.t();
        self.line(&format!("{x} = xor i64 {h}, {byte}"));
        self.line(&format!("{hnext} = mul i64 {x}, 1099511628211"));
        self.line(&format!("{inext} = add i64 {i}, 1"));
        self.line(&format!("br label %{head}"));
        self.label(&done);
        h
    }

    /// Whether the `len` bytes at candor `p1` and `p2` are equal (i64 0/1) — the
    /// byte compare `map_find` runs once the key lengths match.
    fn mem_eq(&mut self, p1: &str, p2: &str, len: &str) -> String {
        let pre = self.l();
        let head = self.l();
        let cmp = self.l();
        let done = self.l();
        let i = self.t();
        let inext = self.t();
        self.line(&format!("br label %{pre}"));
        self.label(&pre);
        self.line(&format!("br label %{head}"));
        self.label(&head);
        self.line(&format!("{i} = phi i64 [ 0, %{pre} ], [ {inext}, %{cmp} ]"));
        let more = self.t();
        self.line(&format!("{more} = icmp ult i64 {i}, {len}"));
        self.line(&format!("br i1 {more}, label %{cmp}, label %{done}"));
        self.label(&cmp);
        let a1 = self.t();
        self.line(&format!("{a1} = add i64 {p1}, {i}"));
        let av = self.load_scalar(&a1, ScalarTy::U8);
        let b1 = self.t();
        self.line(&format!("{b1} = add i64 {p2}, {i}"));
        let bv = self.load_scalar(&b1, ScalarTy::U8);
        let ne = self.t();
        self.line(&format!("{ne} = icmp ne i64 {av}, {bv}"));
        self.line(&format!("{inext} = add i64 {i}, 1"));
        self.line(&format!("br i1 {ne}, label %{done}, label %{head}"));
        self.label(&done);
        let res = self.t();
        self.line(&format!("{res} = phi i64 [ 1, %{head} ], [ 0, %{cmp} ]"));
        res
    }

    /// Open-addressed linear probe for `key`: returns `(found, slot)` (found =
    /// i64 0/1), stopping at the first empty bucket — the `mir::interp::map_find`
    /// probe order (start `hash & (cap-1)`, step +1 & mask).
    fn map_find(&mut self, buf: &str, cap: &str, stride: u64, kptr: &str, klen: &str) -> (String, String) {
        let cap0 = self.t();
        self.line(&format!("{cap0} = icmp eq i64 {cap}, 0"));
        let bufz = self.t();
        self.line(&format!("{bufz} = icmp eq i64 {buf}, 0"));
        let empty = self.t();
        self.line(&format!("{empty} = or i1 {cap0}, {bufz}"));
        let mask = self.t();
        self.line(&format!("{mask} = sub i64 {cap}, 1"));
        let hash = self.map_hash(kptr, klen);
        let idx0 = self.t();
        self.line(&format!("{idx0} = and i64 {hash}, {mask}"));
        let entry = self.l();
        let probe = self.l();
        let checkk = self.l();
        let cmpb = self.l();
        let postcmp = self.l();
        let nextb = self.l();
        let done = self.l();
        let idxphi = self.t();
        let nidx = self.t();
        let found = self.t();
        let slot = self.t();
        self.line(&format!("br label %{entry}"));
        self.label(&entry);
        self.line(&format!("br i1 {empty}, label %{done}, label %{probe}"));
        self.label(&probe);
        self.line(&format!("{idxphi} = phi i64 [ {idx0}, %{entry} ], [ {nidx}, %{nextb} ]"));
        let off = self.t();
        self.line(&format!("{off} = mul i64 {idxphi}, {stride}"));
        let b = self.t();
        self.line(&format!("{b} = add i64 {buf}, {off}"));
        let state = self.load_u64(&b);
        let is0 = self.t();
        self.line(&format!("{is0} = icmp eq i64 {state}, 0"));
        self.line(&format!("br i1 {is0}, label %{done}, label %{checkk}"));
        self.label(&checkk);
        let b16 = self.add_off_candor(&b, 16);
        let slen = self.load_u64(&b16);
        let leneq = self.t();
        self.line(&format!("{leneq} = icmp eq i64 {slen}, {klen}"));
        self.line(&format!("br i1 {leneq}, label %{cmpb}, label %{nextb}"));
        self.label(&cmpb);
        let b8 = self.add_off_candor(&b, 8);
        let sptr = self.load_u64(&b8);
        let eq = self.mem_eq(&sptr, kptr, klen);
        self.line(&format!("br label %{postcmp}"));
        self.label(&postcmp);
        let iseq = self.t();
        self.line(&format!("{iseq} = icmp ne i64 {eq}, 0"));
        self.line(&format!("br i1 {iseq}, label %{done}, label %{nextb}"));
        self.label(&nextb);
        let inc = self.t();
        self.line(&format!("{inc} = add i64 {idxphi}, 1"));
        self.line(&format!("{nidx} = and i64 {inc}, {mask}"));
        self.line(&format!("br label %{probe}"));
        self.label(&done);
        self.line(&format!(
            "{found} = phi i64 [ 0, %{entry} ], [ 0, %{probe} ], [ 1, %{postcmp} ]"
        ));
        self.line(&format!(
            "{slot} = phi i64 [ 0, %{entry} ], [ 0, %{probe} ], [ {idxphi}, %{postcmp} ]"
        ));
        (found, slot)
    }

    /// The first empty bucket along `key`'s probe chain (caller ensures `key`
    /// absent) — `mir::interp::map_find_empty`.
    fn map_find_empty(&mut self, buf: &str, cap: &str, stride: u64, kptr: &str, klen: &str) -> String {
        let mask = self.t();
        self.line(&format!("{mask} = sub i64 {cap}, 1"));
        let hash = self.map_hash(kptr, klen);
        let idx0 = self.t();
        self.line(&format!("{idx0} = and i64 {hash}, {mask}"));
        let entry = self.l();
        let probe = self.l();
        let nextb = self.l();
        let done = self.l();
        let idxphi = self.t();
        let nidx = self.t();
        self.line(&format!("br label %{entry}"));
        self.label(&entry);
        self.line(&format!("br label %{probe}"));
        self.label(&probe);
        self.line(&format!("{idxphi} = phi i64 [ {idx0}, %{entry} ], [ {nidx}, %{nextb} ]"));
        let off = self.t();
        self.line(&format!("{off} = mul i64 {idxphi}, {stride}"));
        let b = self.t();
        self.line(&format!("{b} = add i64 {buf}, {off}"));
        let state = self.load_u64(&b);
        let is0 = self.t();
        self.line(&format!("{is0} = icmp eq i64 {state}, 0"));
        self.line(&format!("br i1 {is0}, label %{done}, label %{nextb}"));
        self.label(&nextb);
        let inc = self.t();
        self.line(&format!("{inc} = add i64 {idxphi}, 1"));
        self.line(&format!("{nidx} = and i64 {inc}, {mask}"));
        self.line(&format!("br label %{probe}"));
        self.label(&done);
        idxphi
    }

    /// Zero `byte_len` bytes at candor `ptr` (a multiple of 8: the fresh bucket
    /// buffer, whose zero `state` words mark every bucket empty).
    fn zero_words(&mut self, ptr: &str, byte_len: &str) {
        let pre = self.l();
        let head = self.l();
        let body = self.l();
        let done = self.l();
        let i = self.t();
        let inext = self.t();
        self.line(&format!("br label %{pre}"));
        self.label(&pre);
        self.line(&format!("br label %{head}"));
        self.label(&head);
        self.line(&format!("{i} = phi i64 [ 0, %{pre} ], [ {inext}, %{body} ]"));
        let more = self.t();
        self.line(&format!("{more} = icmp ult i64 {i}, {byte_len}"));
        self.line(&format!("br i1 {more}, label %{body}, label %{done}"));
        self.label(&body);
        let p = self.t();
        self.line(&format!("{p} = add i64 {ptr}, {i}"));
        self.store_u64(&p, "0");
        self.line(&format!("{inext} = add i64 {i}, 8"));
        self.line(&format!("br label %{head}"));
        self.label(&done);
    }

    /// Grow + rehash the bucket buffer (initial 8, then x2) once the load factor
    /// crosses 3/4, re-probing every live entry — `mir::interp::map_reserve`
    /// (alloc-new + zero + re-insert + free-old). OOM faults `Panic`.
    fn map_reserve(&mut self, base: &str, valty: &Type, span: Span) {
        let vsz = self.lay.size_of(valty);
        let stride = round_up(24 + vsz, 8);
        let b8 = self.add_off_candor(base, 8);
        let len = self.load_u64(&b8);
        let b16 = self.add_off_candor(base, 16);
        let cap = self.load_u64(&b16);
        let capnz = self.t();
        self.line(&format!("{capnz} = icmp ne i64 {cap}, 0"));
        let lp1 = self.t();
        self.line(&format!("{lp1} = add i64 {len}, 1"));
        let lhs = self.t();
        self.line(&format!("{lhs} = mul i64 {lp1}, 4"));
        let rhs = self.t();
        self.line(&format!("{rhs} = mul i64 {cap}, 3"));
        let within = self.t();
        self.line(&format!("{within} = icmp ule i64 {lhs}, {rhs}"));
        let nogrow = self.t();
        self.line(&format!("{nogrow} = and i1 {capnz}, {within}"));
        let grow = self.l();
        let cont = self.l();
        self.line(&format!("br i1 {nogrow}, label %{cont}, label %{grow}"));
        self.label(&grow);
        let capis0 = self.t();
        self.line(&format!("{capis0} = icmp eq i64 {cap}, 0"));
        let cap2 = self.t();
        self.line(&format!("{cap2} = mul i64 {cap}, 2"));
        let newcap = self.t();
        self.line(&format!("{newcap} = select i1 {capis0}, i64 8, i64 {cap2}"));
        let allocsz = self.t();
        self.line(&format!("{allocsz} = mul i64 {newcap}, {stride}"));
        let b24 = self.add_off_candor(base, 24);
        let ctx = self.load_u64(&b24);
        let b32 = self.add_off_candor(base, 32);
        let vt = self.load_u64(&b32);
        let newbuf = self.call_alloc(&ctx, &vt, &allocsz, 8);
        let oom = self.t();
        self.line(&format!("{oom} = icmp eq i64 {newbuf}, 0"));
        self.fault_if(&oom, FaultKind::Panic, span);
        self.zero_words(&newbuf, &allocsz);
        let oldbuf = self.load_u64(base);
        let hasold = self.t();
        self.line(&format!("{hasold} = icmp ne i64 {oldbuf}, 0"));
        let rehash = self.l();
        let setnew = self.l();
        self.line(&format!("br i1 {hasold}, label %{rehash}, label %{setnew}"));
        self.label(&rehash);
        let oh = self.l();
        let scanbody = self.l();
        let reins = self.l();
        let ohnext = self.l();
        let donescan = self.l();
        let ohi = self.t();
        let ohinext = self.t();
        self.line(&format!("br label %{oh}"));
        self.label(&oh);
        self.line(&format!("{ohi} = phi i64 [ 0, %{rehash} ], [ {ohinext}, %{ohnext} ]"));
        let more = self.t();
        self.line(&format!("{more} = icmp ult i64 {ohi}, {cap}"));
        self.line(&format!("br i1 {more}, label %{scanbody}, label %{donescan}"));
        self.label(&scanbody);
        let obo = self.t();
        self.line(&format!("{obo} = mul i64 {ohi}, {stride}"));
        let ob = self.t();
        self.line(&format!("{ob} = add i64 {oldbuf}, {obo}"));
        let ostate = self.load_u64(&ob);
        let occ = self.t();
        self.line(&format!("{occ} = icmp eq i64 {ostate}, 1"));
        self.line(&format!("br i1 {occ}, label %{reins}, label %{ohnext}"));
        self.label(&reins);
        let ob8 = self.add_off_candor(&ob, 8);
        let okptr = self.load_u64(&ob8);
        let ob16 = self.add_off_candor(&ob, 16);
        let oklen = self.load_u64(&ob16);
        let slot = self.map_find_empty(&newbuf, &newcap, stride, &okptr, &oklen);
        let nbo = self.t();
        self.line(&format!("{nbo} = mul i64 {slot}, {stride}"));
        let nb = self.t();
        self.line(&format!("{nb} = add i64 {newbuf}, {nbo}"));
        self.store_u64(&nb, "1");
        let nb8 = self.add_off_candor(&nb, 8);
        self.store_u64(&nb8, &okptr);
        let nb16 = self.add_off_candor(&nb, 16);
        self.store_u64(&nb16, &oklen);
        if vsz > 0 {
            let nb24 = self.add_off_candor(&nb, 24);
            let ob24 = self.add_off_candor(&ob, 24);
            self.rt_copy(&nb24, &ob24, vsz);
        }
        self.line(&format!("br label %{ohnext}"));
        self.label(&ohnext);
        self.line(&format!("{ohinext} = add i64 {ohi}, 1"));
        self.line(&format!("br label %{oh}"));
        self.label(&donescan);
        let capsz = self.t();
        self.line(&format!("{capsz} = mul i64 {cap}, {stride}"));
        self.call_free_val(&ctx, &vt, &oldbuf, &capsz, 8);
        self.line(&format!("br label %{setnew}"));
        self.label(&setnew);
        self.store_u64(base, &newbuf);
        self.store_u64(&b16, &newcap);
        self.line(&format!("br label %{cont}"));
        self.label(&cont);
    }

    /// Grow a `Vec`'s buffer to fit `need` more elements (alloc-new + copy + free-
    /// old), mirroring `mir::interp::vec_reserve` — element `stride`/`align` scale
    /// the byte sizes, `newcap = (len+need).max(cap*2).max(4)`, OOM faults `Panic`.
    fn vec_reserve(&mut self, base: &str, stride: u64, align: u64, need: u64, span: Span) {
        let b8 = self.add_off_candor(base, 8);
        let len = self.load_u64(&b8);
        let b16 = self.add_off_candor(base, 16);
        let cap = self.load_u64(&b16);
        let lenneed = self.t();
        self.line(&format!("{lenneed} = add i64 {len}, {need}"));
        let need_grow = self.t();
        self.line(&format!("{need_grow} = icmp ugt i64 {lenneed}, {cap}"));
        let grow_b = self.l();
        let cont = self.l();
        self.line(&format!("br i1 {need_grow}, label %{grow_b}, label %{cont}"));

        self.label(&grow_b);
        let cap2 = self.t();
        self.line(&format!("{cap2} = mul i64 {cap}, 2"));
        let m1 = self.umax(&lenneed, &cap2);
        let newcap = self.umax(&m1, "4");
        let allocsz = self.t();
        self.line(&format!("{allocsz} = mul i64 {newcap}, {stride}"));
        let b24 = self.add_off_candor(base, 24);
        let ctx = self.load_u64(&b24);
        let b32 = self.add_off_candor(base, 32);
        let vt = self.load_u64(&b32);
        let oldbuf = self.load_u64(base);
        let has_old = self.t();
        self.line(&format!("{has_old} = icmp ne i64 {oldbuf}, 0"));
        let alloc_b = self.l();
        let realloc_b = self.l();
        let joined = self.l();
        self.line(&format!("br i1 {has_old}, label %{realloc_b}, label %{alloc_b}"));
        self.label(&alloc_b);
        let nb_a = self.call_alloc(&ctx, &vt, &allocsz, align);
        self.line(&format!("br label %{joined}"));
        self.label(&realloc_b);
        let oldsz = self.t();
        self.line(&format!("{oldsz} = mul i64 {len}, {stride}"));
        let nb_r = self.call_realloc(&ctx, &vt, &oldbuf, &oldsz, &allocsz, align);
        self.line(&format!("br label %{joined}"));
        self.label(&joined);
        let newbuf = self.t();
        self.line(&format!("{newbuf} = phi i64 [ {nb_a}, %{alloc_b} ], [ {nb_r}, %{realloc_b} ]"));
        let is_oom = self.t();
        self.line(&format!("{is_oom} = icmp eq i64 {newbuf}, 0"));
        self.fault_if(&is_oom, FaultKind::Panic, span);
        self.store_u64(base, &newbuf);
        self.store_u64(&b16, &newcap);
        self.line(&format!("br label %{cont}"));
        self.label(&cont);
    }

    /// Grow a `String`'s buffer to fit `need` more bytes (alloc-new + copy + free-
    /// old), mirroring `mir::interp::string_reserve` — `newcap =
    /// (len+need).max(cap*2).max(8)`, OOM faults `Panic`.
    fn string_reserve(&mut self, base: &str, need: &str, span: Span) {
        let b8 = self.add_off_candor(base, 8);
        let len = self.load_u64(&b8);
        let b16 = self.add_off_candor(base, 16);
        let cap = self.load_u64(&b16);
        let lenneed = self.t();
        self.line(&format!("{lenneed} = add i64 {len}, {need}"));
        let need_grow = self.t();
        self.line(&format!("{need_grow} = icmp ugt i64 {lenneed}, {cap}"));
        let grow_b = self.l();
        let cont = self.l();
        self.line(&format!("br i1 {need_grow}, label %{grow_b}, label %{cont}"));

        self.label(&grow_b);
        let cap2 = self.t();
        self.line(&format!("{cap2} = mul i64 {cap}, 2"));
        let m1 = self.umax(&lenneed, &cap2);
        let newcap = self.umax(&m1, "8");
        let b24 = self.add_off_candor(base, 24);
        let ctx = self.load_u64(&b24);
        let b32 = self.add_off_candor(base, 32);
        let vt = self.load_u64(&b32);
        let oldbuf = self.load_u64(base);
        let has_old = self.t();
        self.line(&format!("{has_old} = icmp ne i64 {oldbuf}, 0"));
        let alloc_b = self.l();
        let realloc_b = self.l();
        let joined = self.l();
        self.line(&format!("br i1 {has_old}, label %{realloc_b}, label %{alloc_b}"));
        self.label(&alloc_b);
        let nb_a = self.call_alloc(&ctx, &vt, &newcap, 1);
        self.line(&format!("br label %{joined}"));
        self.label(&realloc_b);
        let nb_r = self.call_realloc(&ctx, &vt, &oldbuf, &len, &newcap, 1);
        self.line(&format!("br label %{joined}"));
        self.label(&joined);
        let newbuf = self.t();
        self.line(&format!("{newbuf} = phi i64 [ {nb_a}, %{alloc_b} ], [ {nb_r}, %{realloc_b} ]"));
        let is_oom = self.t();
        self.line(&format!("{is_oom} = icmp eq i64 {newbuf}, 0"));
        self.fault_if(&is_oom, FaultKind::Panic, span);
        self.store_u64(base, &newbuf);
        self.store_u64(&b16, &newcap);
        self.line(&format!("br label %{cont}"));
        self.label(&cont);
    }

    /// Unsigned max via select (wrapping arithmetic; matches the interp `.max`).
    fn umax(&mut self, a: &str, b: &str) -> String {
        let c = self.t();
        self.line(&format!("{c} = icmp ugt i64 {a}, {b}"));
        let r = self.t();
        self.line(&format!("{r} = select i1 {c}, i64 {a}, i64 {b}"));
        r
    }

    /// Call the carried allocator's `alloc` vtable slot with a runtime `size`
    /// (mirrors `box_op`'s alloc dispatch; `align` is a small constant).
    fn call_alloc(&mut self, ctx: &str, vt: &str, size: &str, align: u64) -> String {
        let alloc_off = self.field_off(&self.alloc_vtable_name(), "alloc");
        let aa = self.add_off_candor(vt, alloc_off);
        let afn = self.load_u64(&aa);
        self.call_fnptr_id(&afn, &[ctx.to_string(), size.to_string(), format!("{align}")])
    }

    /// Grow `ptr` (holding `old_size` bytes) to `new_size` through the vtable's
    /// `realloc` slot; returns the (possibly moved) pointer (0 on OOM).
    fn call_realloc(&mut self, ctx: &str, vt: &str, ptr: &str, old_size: &str, new_size: &str, align: u64) -> String {
        let realloc_off = self.field_off(&self.alloc_vtable_name(), "realloc");
        let ra = self.add_off_candor(vt, realloc_off);
        let rfn = self.load_u64(&ra);
        self.call_fnptr_id(&rfn, &[ctx.to_string(), ptr.to_string(), old_size.to_string(), new_size.to_string(), format!("{align}")])
    }

    /// Free through the vtable with a runtime `size` (the String buffer capacity).
    fn call_free_val(&mut self, ctx: &str, vt: &str, ptr: &str, size: &str, align: u64) {
        let free_off = self.field_off(&self.alloc_vtable_name(), "free");
        let fa = self.add_off_candor(vt, free_off);
        let ffn = self.load_u64(&fa);
        self.call_fnptr_id(&ffn, &[ctx.to_string(), ptr.to_string(), size.to_string(), format!("{align}")]);
    }

    /// Byte-copy with a runtime length (the `rt_copy` counterpart for dynamic
    /// collection-buffer sizes).
    fn rt_copy_val(&mut self, dst: &str, src: &str, len: &str) {
        self.line(&format!("call void @rt_copy(i64 {dst}, i64 {src}, i64 {len})"));
    }

    /// Store the low byte of i64 `val` at candor address `addr` (UTF-8 byte writer).
    fn store_byte(&mut self, addr: &str, val: &str) {
        self.store_scalar(addr, val, ScalarTy::U8);
    }

    /// UTF-8 lead byte `prefix | (c >> shift)` (no mask: the scalar's high bits are
    /// already zero for the chosen width).
    fn utf8_lead(&mut self, c: &str, shift: u32, prefix: u32) -> String {
        let sh = self.t();
        self.line(&format!("{sh} = lshr i64 {c}, {shift}"));
        let r = self.t();
        self.line(&format!("{r} = or i64 {sh}, {prefix}"));
        r
    }

    /// UTF-8 continuation byte `0x80 | ((c >> shift) & 0x3F)`.
    fn utf8_cont(&mut self, c: &str, shift: u32) -> String {
        let sh = self.t();
        self.line(&format!("{sh} = lshr i64 {c}, {shift}"));
        let m = self.t();
        self.line(&format!("{m} = and i64 {sh}, 63"));
        let r = self.t();
        self.line(&format!("{r} = or i64 {m}, 128"));
        r
    }

    fn call_free(&mut self, ctx: &str, vt: &str, ptr: &str, size: u64, align: u64) {
        let free_off = self.field_off(&self.alloc_vtable_name(), "free");
        let fa = self.add_off_candor(vt, free_off);
        let ffn = self.load_u64(&fa);
        self.call_fnptr_id(&ffn, &[ctx.to_string(), ptr.to_string(), format!("{size}"), format!("{align}")]);
    }

    fn call_mmio_load(&mut self, addr: &str, size: u64) -> String {
        let r = self.t();
        self.line(&format!("{r} = call i64 @rt_mmio_load(i64 {addr}, i64 {size})"));
        r
    }
    fn call_mmio_store(&mut self, addr: &str, val: &str, size: u64) {
        self.line(&format!("call void @rt_mmio_store(i64 {addr}, i64 {val}, i64 {size})"));
    }

    /// `box(alloc, value)` (design 0001 §6.2): allocate through the handle's vtable,
    /// move `value` into the block, and build the `BoxResult` at `dst` — the boxed
    /// arm carries `{ptr, ctx, vt}`; a null return is out-of-memory (payload dropped,
    /// `oom` arm). Mirrors `lower::box_op` op-for-op.
    fn box_op(
        &mut self,
        dst: &Place,
        inner_ty: &Type,
        result_ty: &Type,
        alloc: &Operand,
        value: &Place,
    ) -> Result<(), String> {
        let alloc_addr = self.operand(alloc);
        let astruct = self.alloc_struct_name();
        let vtstruct = self.alloc_vtable_name();
        let ctx_off = self.field_off(&astruct, "ctx");
        let vt_off = self.field_off(&astruct, "vt");
        let ca = self.add_off_candor(&alloc_addr, ctx_off);
        let ctx = self.load_u64(&ca);
        let va = self.add_off_candor(&alloc_addr, vt_off);
        let vt = self.load_u64(&va);
        let size = self.lay.size_of(inner_ty);
        let align = self.lay.align_of(inner_ty);
        let alloc_off = self.field_off(&vtstruct, "alloc");
        let aa = self.add_off_candor(&vt, alloc_off);
        let afn = self.load_u64(&aa);
        let ret = self.call_fnptr_id(&afn, &[ctx.clone(), format!("{size}"), format!("{align}")]);
        let (value_addr, _) = self.place_addr(value)?;
        let (daddr, _) = self.place_addr(dst)?;

        let einfo = self.lay.enum_info(result_ty);
        let (boxed_idx, oom_idx) = match &einfo {
            Some(v) => (
                v.iter().position(|(_, p)| !p.is_empty()).unwrap_or(0),
                v.iter().position(|(_, p)| p.is_empty()).unwrap_or(1),
            ),
            None => (0, 1),
        };

        let is_oom = self.t();
        self.line(&format!("{is_oom} = icmp eq i64 {ret}, 0"));
        let oom_b = self.l();
        let boxed_b = self.l();
        let cont = self.l();
        self.line(&format!("br i1 {is_oom}, label %{oom_b}, label %{boxed_b}"));

        self.label(&oom_b);
        self.emit_drop(&value_addr, inner_ty, &[], &mut Vec::new())?;
        self.store_u64(&daddr, &format!("{oom_idx}"));
        self.line(&format!("br label %{cont}"));

        self.label(&boxed_b);
        self.rt_copy(&ret, &value_addr, size);
        self.store_u64(&daddr, &format!("{boxed_idx}"));
        let d8 = self.add_off_candor(&daddr, 8);
        self.store_u64(&d8, &ret);
        let d16 = self.add_off_candor(&daddr, 16);
        self.store_u64(&d16, &ctx);
        let d24 = self.add_off_candor(&daddr, 24);
        self.store_u64(&d24, &vt);
        self.line(&format!("br label %{cont}"));

        self.label(&cont);
        Ok(())
    }

    /// `unbox(b)` (design 0001 §6.2): copy the pointee into `dst`, then free the
    /// block through its stored vtable handle (mirrors `lower::unbox_op`).
    fn unbox_op(&mut self, dst: &Place, inner_ty: &Type, boxed: &Place) -> Result<(), String> {
        let (baddr, _) = self.place_addr(boxed)?;
        let ptr = self.load_u64(&baddr);
        let b8 = self.add_off_candor(&baddr, 8);
        let ctx = self.load_u64(&b8);
        let b16 = self.add_off_candor(&baddr, 16);
        let vt = self.load_u64(&b16);
        let size = self.lay.size_of(inner_ty);
        let align = self.lay.align_of(inner_ty);
        let (daddr, _) = self.place_addr(dst)?;
        self.rt_copy(&daddr, &ptr, size);
        self.call_free(&ctx, &vt, &ptr, size, align);
        Ok(())
    }

    /// `subslice(s, lo, hi)` (design 0004): a bounds-checked slice re-header into
    /// `dst`; `lo > hi || hi > len` faults `Bounds` (mirrors `lower::subslice_op`).
    #[allow(clippy::too_many_arguments)]
    fn subslice_op(
        &mut self,
        dst: &Place,
        src: &Place,
        lo: &Operand,
        hi: &Operand,
        stride: u64,
        span: Span,
    ) -> Result<(), String> {
        let (saddr, _) = self.place_addr(src)?;
        let ptr = self.load_u64(&saddr);
        let s8 = self.add_off_candor(&saddr, 8);
        let len = self.load_u64(&s8);
        let lo = self.operand(lo);
        let hi = self.operand(hi);
        let lo_gt_hi = self.t();
        self.line(&format!("{lo_gt_hi} = icmp ugt i64 {lo}, {hi}"));
        let hi_gt_len = self.t();
        self.line(&format!("{hi_gt_len} = icmp ugt i64 {hi}, {len}"));
        let bad = self.t();
        self.line(&format!("{bad} = or i1 {lo_gt_hi}, {hi_gt_len}"));
        self.fault_if(&bad, FaultKind::Bounds, span);
        let (daddr, _) = self.place_addr(dst)?;
        let losc = self.t();
        self.line(&format!("{losc} = mul i64 {lo}, {stride}"));
        let newptr = self.t();
        self.line(&format!("{newptr} = add i64 {ptr}, {losc}"));
        self.store_u64(&daddr, &newptr);
        let newlen = self.t();
        self.line(&format!("{newlen} = sub i64 {hi}, {lo}"));
        let d8 = self.add_off_candor(&daddr, 8);
        self.store_u64(&d8, &newlen);
        Ok(())
    }

    /// `b & 0xC0 == 0x80` (i1): is `b` a UTF-8 continuation byte?
    fn byte_is_cont(&mut self, b: &str) -> String {
        let m = self.t();
        self.line(&format!("{m} = and i64 {b}, 192"));
        let r = self.t();
        self.line(&format!("{r} = icmp eq i64 {m}, 128"));
        r
    }
    /// `b in [lo, hi]` (unsigned, i1).
    fn byte_in_range(&mut self, b: &str, lo: u32, hi: u32) -> String {
        let ge = self.t();
        self.line(&format!("{ge} = icmp uge i64 {b}, {lo}"));
        let le = self.t();
        self.line(&format!("{le} = icmp ule i64 {b}, {hi}"));
        let r = self.t();
        self.line(&format!("{r} = and i1 {ge}, {le}"));
        r
    }

    /// `str_from(b) -> Utf8Res` (design 0013 §4): scan `src`'s `[u8]` bytes for
    /// UTF-8 well-formedness (lead-class / continuation / overlong / surrogate /
    /// range — the `str::from_utf8` algorithm), building `Utf8Res::Valid(str)` (the
    /// same `{ptr, len}` view, retyped) or `Utf8Res::Invalid(offset)`, where
    /// `offset` is the START of the first ill-formed sequence — exactly the
    /// `valid_up_to()` the interpreter oracle reports (byte-exact). No fault.
    fn str_from_op(&mut self, dst: &Place, src: &Place) -> Result<(), String> {
        let (saddr, _) = self.place_addr(src)?;
        let ptr = self.load_u64(&saddr);
        let s8 = self.add_off_candor(&saddr, 8);
        let len = self.load_u64(&s8);
        let (daddr, _) = self.place_addr(dst)?;
        let ures = Type::Named("Utf8Res".to_string());
        let einfo = self.lay.enum_info(&ures).ok_or("unknown enum `Utf8Res`")?;
        let valid_idx = einfo.iter().position(|(n, _)| n == "Valid").unwrap_or(0);
        let invalid_idx = einfo.iter().position(|(n, _)| n == "Invalid").unwrap_or(1);
        let (_, valid_off) = self.lay.payload_offset(&einfo[valid_idx].1, 0);
        let (_, invalid_off) = self.lay.payload_offset(&einfo[invalid_idx].1, 0);

        let pre = self.l();
        let head = self.l();
        let body = self.l();
        let adv1 = self.l();
        let notascii = self.l();
        let m2 = self.l();
        let m2b = self.l();
        let adv2 = self.l();
        let not2 = self.l();
        let m3 = self.l();
        let m3b = self.l();
        let m3c = self.l();
        let adv3 = self.l();
        let not3 = self.l();
        let m4 = self.l();
        let m4b = self.l();
        let m4c = self.l();
        let m4d = self.l();
        let adv4 = self.l();
        let invalid = self.l();
        let valid_exit = self.l();
        let done = self.l();

        let idx = self.t();
        let n1 = self.t();
        let n2 = self.t();
        let n3 = self.t();
        let n4 = self.t();

        self.line(&format!("br label %{pre}"));
        self.label(&pre);
        self.line(&format!("br label %{head}"));
        self.label(&head);
        self.line(&format!(
            "{idx} = phi i64 [ 0, %{pre} ], [ {n1}, %{adv1} ], [ {n2}, %{adv2} ], [ {n3}, %{adv3} ], [ {n4}, %{adv4} ]"
        ));
        let more = self.t();
        self.line(&format!("{more} = icmp ult i64 {idx}, {len}"));
        self.line(&format!("br i1 {more}, label %{body}, label %{valid_exit}"));
        // body: decode the sequence starting at idx.
        self.label(&body);
        let a0 = self.t();
        self.line(&format!("{a0} = add i64 {ptr}, {idx}"));
        let b0 = self.load_scalar(&a0, ScalarTy::U8);
        let ascii = self.t();
        self.line(&format!("{ascii} = icmp ult i64 {b0}, 128"));
        self.line(&format!("br i1 {ascii}, label %{adv1}, label %{notascii}"));
        self.label(&adv1);
        self.line(&format!("{n1} = add i64 {idx}, 1"));
        self.line(&format!("br label %{head}"));
        // 2-byte lead (0xC2..=0xDF).
        self.label(&notascii);
        let is_w2 = self.byte_in_range(&b0, 0xC2, 0xDF);
        self.line(&format!("br i1 {is_w2}, label %{m2}, label %{not2}"));
        self.label(&m2);
        let p1 = self.t();
        self.line(&format!("{p1} = add i64 {idx}, 1"));
        let pres2 = self.t();
        self.line(&format!("{pres2} = icmp ult i64 {p1}, {len}"));
        self.line(&format!("br i1 {pres2}, label %{m2b}, label %{invalid}"));
        self.label(&m2b);
        let a1 = self.t();
        self.line(&format!("{a1} = add i64 {ptr}, {p1}"));
        let bb1 = self.load_scalar(&a1, ScalarTy::U8);
        let cont2 = self.byte_is_cont(&bb1);
        self.line(&format!("br i1 {cont2}, label %{adv2}, label %{invalid}"));
        self.label(&adv2);
        self.line(&format!("{n2} = add i64 {idx}, 2"));
        self.line(&format!("br label %{head}"));
        // 3-byte lead (0xE0..=0xEF).
        self.label(&not2);
        let is_w3 = self.byte_in_range(&b0, 0xE0, 0xEF);
        self.line(&format!("br i1 {is_w3}, label %{m3}, label %{not3}"));
        self.label(&m3);
        let p1b = self.t();
        self.line(&format!("{p1b} = add i64 {idx}, 1"));
        let p2 = self.t();
        self.line(&format!("{p2} = add i64 {idx}, 2"));
        let pres3 = self.t();
        self.line(&format!("{pres3} = icmp ult i64 {p2}, {len}"));
        self.line(&format!("br i1 {pres3}, label %{m3b}, label %{invalid}"));
        self.label(&m3b);
        let a1_3 = self.t();
        self.line(&format!("{a1_3} = add i64 {ptr}, {p1b}"));
        let b1_3 = self.load_scalar(&a1_3, ScalarTy::U8);
        // Second-byte range: E0 -> A0..BF, ED -> 80..9F, else 80..BF.
        let is_e0 = self.t();
        self.line(&format!("{is_e0} = icmp eq i64 {b0}, 224"));
        let is_ed = self.t();
        self.line(&format!("{is_ed} = icmp eq i64 {b0}, 237"));
        let lo3 = self.t();
        self.line(&format!("{lo3} = select i1 {is_e0}, i64 160, i64 128"));
        let hi3 = self.t();
        self.line(&format!("{hi3} = select i1 {is_ed}, i64 159, i64 191"));
        let ge3 = self.t();
        self.line(&format!("{ge3} = icmp uge i64 {b1_3}, {lo3}"));
        let le3 = self.t();
        self.line(&format!("{le3} = icmp ule i64 {b1_3}, {hi3}"));
        let ok2_3 = self.t();
        self.line(&format!("{ok2_3} = and i1 {ge3}, {le3}"));
        self.line(&format!("br i1 {ok2_3}, label %{m3c}, label %{invalid}"));
        self.label(&m3c);
        let a2_3 = self.t();
        self.line(&format!("{a2_3} = add i64 {ptr}, {p2}"));
        let b2_3 = self.load_scalar(&a2_3, ScalarTy::U8);
        let cont3 = self.byte_is_cont(&b2_3);
        self.line(&format!("br i1 {cont3}, label %{adv3}, label %{invalid}"));
        self.label(&adv3);
        self.line(&format!("{n3} = add i64 {idx}, 3"));
        self.line(&format!("br label %{head}"));
        // 4-byte lead (0xF0..=0xF4); anything else is an invalid lead.
        self.label(&not3);
        let is_w4 = self.byte_in_range(&b0, 0xF0, 0xF4);
        self.line(&format!("br i1 {is_w4}, label %{m4}, label %{invalid}"));
        self.label(&m4);
        let p1c = self.t();
        self.line(&format!("{p1c} = add i64 {idx}, 1"));
        let p2c = self.t();
        self.line(&format!("{p2c} = add i64 {idx}, 2"));
        let p3 = self.t();
        self.line(&format!("{p3} = add i64 {idx}, 3"));
        let pres4 = self.t();
        self.line(&format!("{pres4} = icmp ult i64 {p3}, {len}"));
        self.line(&format!("br i1 {pres4}, label %{m4b}, label %{invalid}"));
        self.label(&m4b);
        let a1_4 = self.t();
        self.line(&format!("{a1_4} = add i64 {ptr}, {p1c}"));
        let b1_4 = self.load_scalar(&a1_4, ScalarTy::U8);
        // Second-byte range: F0 -> 90..BF, F4 -> 80..8F, else 80..BF.
        let is_f0 = self.t();
        self.line(&format!("{is_f0} = icmp eq i64 {b0}, 240"));
        let is_f4 = self.t();
        self.line(&format!("{is_f4} = icmp eq i64 {b0}, 244"));
        let lo4 = self.t();
        self.line(&format!("{lo4} = select i1 {is_f0}, i64 144, i64 128"));
        let hi4 = self.t();
        self.line(&format!("{hi4} = select i1 {is_f4}, i64 143, i64 191"));
        let ge4 = self.t();
        self.line(&format!("{ge4} = icmp uge i64 {b1_4}, {lo4}"));
        let le4 = self.t();
        self.line(&format!("{le4} = icmp ule i64 {b1_4}, {hi4}"));
        let ok2_4 = self.t();
        self.line(&format!("{ok2_4} = and i1 {ge4}, {le4}"));
        self.line(&format!("br i1 {ok2_4}, label %{m4c}, label %{invalid}"));
        self.label(&m4c);
        let a2_4 = self.t();
        self.line(&format!("{a2_4} = add i64 {ptr}, {p2c}"));
        let b2_4 = self.load_scalar(&a2_4, ScalarTy::U8);
        let cont4b = self.byte_is_cont(&b2_4);
        self.line(&format!("br i1 {cont4b}, label %{m4d}, label %{invalid}"));
        self.label(&m4d);
        let a3_4 = self.t();
        self.line(&format!("{a3_4} = add i64 {ptr}, {p3}"));
        let b3_4 = self.load_scalar(&a3_4, ScalarTy::U8);
        let cont4c = self.byte_is_cont(&b3_4);
        self.line(&format!("br i1 {cont4c}, label %{adv4}, label %{invalid}"));
        self.label(&adv4);
        self.line(&format!("{n4} = add i64 {idx}, 4"));
        self.line(&format!("br label %{head}"));
        // invalid: build `Utf8Res::Invalid(idx)`.
        self.label(&invalid);
        self.store_u64(&daddr, &format!("{invalid_idx}"));
        let inv_slot = self.add_off_candor(&daddr, invalid_off);
        self.store_u64(&inv_slot, &idx);
        self.line(&format!("br label %{done}"));
        // valid_exit: build `Utf8Res::Valid(str)` — the same `{ptr, len}` view.
        self.label(&valid_exit);
        self.store_u64(&daddr, &format!("{valid_idx}"));
        let val_slot = self.add_off_candor(&daddr, valid_off);
        self.store_u64(&val_slot, &ptr);
        let val_slot8 = self.add_off_candor(&daddr, valid_off + 8);
        self.store_u64(&val_slot8, &len);
        self.line(&format!("br label %{done}"));
        self.label(&done);
        Ok(())
    }

    /// Is byte offset `i` NOT a char boundary of the run at `ptr` (len `len`)?
    /// `i == 0 || i == len` is always a boundary; otherwise the byte at `i` must not
    /// be a continuation byte. Returns an i1 (true = fault); guards the load so
    /// `i == len` never reads past the run.
    fn boundary_bad(&mut self, ptr: &str, i: &str, len: &str) -> String {
        let is0 = self.t();
        self.line(&format!("{is0} = icmp eq i64 {i}, 0"));
        let islen = self.t();
        self.line(&format!("{islen} = icmp eq i64 {i}, {len}"));
        let skip = self.t();
        self.line(&format!("{skip} = or i1 {is0}, {islen}"));
        let bload = self.l();
        let bok = self.l();
        let bmerge = self.l();
        self.line(&format!("br i1 {skip}, label %{bok}, label %{bload}"));
        self.label(&bload);
        let a = self.t();
        self.line(&format!("{a} = add i64 {ptr}, {i}"));
        let byte = self.load_scalar(&a, ScalarTy::U8);
        let cont = self.byte_is_cont(&byte);
        self.line(&format!("br label %{bmerge}"));
        self.label(&bok);
        self.line(&format!("br label %{bmerge}"));
        self.label(&bmerge);
        let bad = self.t();
        self.line(&format!("{bad} = phi i1 [ false, %{bok} ], [ {cont}, %{bload} ]"));
        bad
    }

    /// `substr(s, lo, hi) -> str` (design 0013 §3): the `[lo, hi)` byte sub-view,
    /// faulting `Bounds` at `span` on `lo > hi || hi > len` OR when `lo`/`hi` is not
    /// a UTF-8 character boundary. Mirrors the interpreter `bi_substr` byte-for-byte.
    fn substr_op(&mut self, dst: &Place, src: &Place, lo: &Operand, hi: &Operand, span: Span) -> Result<(), String> {
        let (saddr, _) = self.place_addr(src)?;
        let ptr = self.load_u64(&saddr);
        let s8 = self.add_off_candor(&saddr, 8);
        let len = self.load_u64(&s8);
        let lo = self.operand(lo);
        let hi = self.operand(hi);
        let lo_gt_hi = self.t();
        self.line(&format!("{lo_gt_hi} = icmp ugt i64 {lo}, {hi}"));
        let hi_gt_len = self.t();
        self.line(&format!("{hi_gt_len} = icmp ugt i64 {hi}, {len}"));
        let oob = self.t();
        self.line(&format!("{oob} = or i1 {lo_gt_hi}, {hi_gt_len}"));
        self.fault_if(&oob, FaultKind::Bounds, span);
        let lo_bad = self.boundary_bad(&ptr, &lo, &len);
        let hi_bad = self.boundary_bad(&ptr, &hi, &len);
        let bad = self.t();
        self.line(&format!("{bad} = or i1 {lo_bad}, {hi_bad}"));
        self.fault_if(&bad, FaultKind::Bounds, span);
        let (daddr, _) = self.place_addr(dst)?;
        let newptr = self.t();
        self.line(&format!("{newptr} = add i64 {ptr}, {lo}"));
        self.store_u64(&daddr, &newptr);
        let newlen = self.t();
        self.line(&format!("{newlen} = sub i64 {hi}, {lo}"));
        let d8 = self.add_off_candor(&daddr, 8);
        self.store_u64(&d8, &newlen);
        Ok(())
    }

    /// Drop a Box pointee at flat address `addr` (mirrors `lower::drop_box`):
    /// recursively drop the pointee through its glue fn, then free the block —
    /// guarded by `ptr != 0` (a null / OOM Box owns nothing).
    fn drop_box(&mut self, addr: &str, inner: &Type) {
        let ptr = self.load_u64(addr);
        let a8 = self.add_off_candor(addr, 8);
        let ctx = self.load_u64(&a8);
        let a16 = self.add_off_candor(addr, 16);
        let vt = self.load_u64(&a16);
        let nz = self.t();
        self.line(&format!("{nz} = icmp ne i64 {ptr}, 0"));
        let dob = self.l();
        let cont = self.l();
        self.line(&format!("br i1 {nz}, label %{dob}, label %{cont}"));
        self.label(&dob);
        self.call_drop_glue(&ptr, inner);
        let size = self.lay.size_of(inner);
        let align = self.lay.align_of(inner);
        self.call_free(&ctx, &vt, &ptr, size, align);
        self.line(&format!("br label %{cont}"));
        self.label(&cont);
    }

    /// Drop a compiler-known `Vec[T]` at `addr` (mirror of `mir::interp` drop_value):
    /// with a non-null buffer, drop each live element in reverse index order, then
    /// free the buffer through the carried allocator (`cap * stride` bytes, elem align).
    fn drop_vec(&mut self, addr: &str, elem: &Type) -> Result<(), String> {
        let buf = self.load_u64(addr);
        let nz = self.t();
        self.line(&format!("{nz} = icmp ne i64 {buf}, 0"));
        let dob = self.l();
        let cont = self.l();
        self.line(&format!("br i1 {nz}, label %{dob}, label %{cont}"));
        self.label(&dob);
        let stride = round_up(self.lay.size_of(elem), self.lay.align_of(elem));
        let align = self.lay.align_of(elem);
        if self.needs_drop(elem) {
            let a8 = self.add_off_candor(addr, 8);
            let len = self.load_u64(&a8);
            let pre = self.l();
            let head = self.l();
            let body = self.l();
            let next = self.l();
            let done = self.l();
            let i = self.t();
            let idx = self.t();
            let inext = self.t();
            self.line(&format!("br label %{pre}"));
            self.label(&pre);
            self.line(&format!("br label %{head}"));
            self.label(&head);
            self.line(&format!("{i} = phi i64 [ {len}, %{pre} ], [ {inext}, %{next} ]"));
            let more = self.t();
            self.line(&format!("{more} = icmp ugt i64 {i}, 0"));
            self.line(&format!("br i1 {more}, label %{body}, label %{done}"));
            self.label(&body);
            self.line(&format!("{idx} = sub i64 {i}, 1"));
            let off = self.t();
            self.line(&format!("{off} = mul i64 {idx}, {stride}"));
            let ea = self.t();
            self.line(&format!("{ea} = add i64 {buf}, {off}"));
            self.emit_drop(&ea, elem, &[], &mut Vec::new())?;
            self.line(&format!("br label %{next}"));
            self.label(&next);
            self.line(&format!("{inext} = sub i64 {i}, 1"));
            self.line(&format!("br label %{head}"));
            self.label(&done);
        }
        let a16 = self.add_off_candor(addr, 16);
        let cap = self.load_u64(&a16);
        let a24 = self.add_off_candor(addr, 24);
        let ctx = self.load_u64(&a24);
        let a32 = self.add_off_candor(addr, 32);
        let vt = self.load_u64(&a32);
        let size = self.t();
        self.line(&format!("{size} = mul i64 {cap}, {stride}"));
        self.call_free_val(&ctx, &vt, &buf, &size, align);
        self.line(&format!("br label %{cont}"));
        self.label(&cont);
        Ok(())
    }

    /// Drop a compiler-known `Map[V]` at `addr` (mirror of `mir::interp` drop_value):
    /// with a non-null buffer, for each occupied slot free its owned key bytes then
    /// drop its value, then free the bucket buffer (`cap * stride` bytes, align 8).
    fn drop_map(&mut self, addr: &str, valty: &Type) -> Result<(), String> {
        let buf = self.load_u64(addr);
        let nz = self.t();
        self.line(&format!("{nz} = icmp ne i64 {buf}, 0"));
        let dob = self.l();
        let cont = self.l();
        self.line(&format!("br i1 {nz}, label %{dob}, label %{cont}"));
        self.label(&dob);
        let stride = round_up(24 + self.lay.size_of(valty), 8);
        let a16 = self.add_off_candor(addr, 16);
        let cap = self.load_u64(&a16);
        let a24 = self.add_off_candor(addr, 24);
        let ctx = self.load_u64(&a24);
        let a32 = self.add_off_candor(addr, 32);
        let vt = self.load_u64(&a32);
        let pre = self.l();
        let head = self.l();
        let body = self.l();
        let slotb = self.l();
        let next = self.l();
        let done = self.l();
        let i = self.t();
        let inext = self.t();
        self.line(&format!("br label %{pre}"));
        self.label(&pre);
        self.line(&format!("br label %{head}"));
        self.label(&head);
        self.line(&format!("{i} = phi i64 [ 0, %{pre} ], [ {inext}, %{next} ]"));
        let more = self.t();
        self.line(&format!("{more} = icmp ult i64 {i}, {cap}"));
        self.line(&format!("br i1 {more}, label %{body}, label %{done}"));
        self.label(&body);
        let off = self.t();
        self.line(&format!("{off} = mul i64 {i}, {stride}"));
        let b = self.t();
        self.line(&format!("{b} = add i64 {buf}, {off}"));
        let occ = self.load_u64(&b);
        let isocc = self.t();
        self.line(&format!("{isocc} = icmp eq i64 {occ}, 1"));
        self.line(&format!("br i1 {isocc}, label %{slotb}, label %{next}"));
        self.label(&slotb);
        let b8 = self.add_off_candor(&b, 8);
        let kptr = self.load_u64(&b8);
        let b16 = self.add_off_candor(&b, 16);
        let klen = self.load_u64(&b16);
        self.call_free_val(&ctx, &vt, &kptr, &klen, 1);
        let b24 = self.add_off_candor(&b, 24);
        self.emit_drop(&b24, valty, &[], &mut Vec::new())?;
        self.line(&format!("br label %{next}"));
        self.label(&next);
        self.line(&format!("{inext} = add i64 {i}, 1"));
        self.line(&format!("br label %{head}"));
        self.label(&done);
        let size = self.t();
        self.line(&format!("{size} = mul i64 {cap}, {stride}"));
        self.call_free_val(&ctx, &vt, &buf, &size, 8);
        self.line(&format!("br label %{cont}"));
        self.label(&cont);
        Ok(())
    }

    /// Drop a compiler-known `String` at `addr` (mirror of `mir::interp` drop_value):
    /// free its UTF-8 buffer through the carried allocator (`cap` bytes, align 1) when
    /// the buffer is non-null. Bytes are POD, so there are no element drops.
    fn drop_string(&mut self, addr: &str) {
        let buf = self.load_u64(addr);
        let nz = self.t();
        self.line(&format!("{nz} = icmp ne i64 {buf}, 0"));
        let dob = self.l();
        let cont = self.l();
        self.line(&format!("br i1 {nz}, label %{dob}, label %{cont}"));
        self.label(&dob);
        let a16 = self.add_off_candor(addr, 16);
        let cap = self.load_u64(&a16);
        let a24 = self.add_off_candor(addr, 24);
        let ctx = self.load_u64(&a24);
        let a32 = self.add_off_candor(addr, 32);
        let vt = self.load_u64(&a32);
        self.call_free_val(&ctx, &vt, &buf, &cap, 1);
        self.line(&format!("br label %{cont}"));
        self.label(&cont);
    }

    /// Drop a Box pointee at `addr` by calling its synthesized glue fn (runtime
    /// recursion; the glue fn terminates on a null inner pointer).
    fn call_drop_glue(&mut self, addr: &str, inner: &Type) {
        if !self.needs_drop(inner) {
            return;
        }
        if let Some(i) = self.glue_index.get(&type_key(inner)) {
            let r = self.t();
            self.line(&format!("{r} = call i64 @\"__drop_glue_{i}\"(i64 {addr})"));
        }
    }

    fn lower_stmt(&mut self, st: &Statement) -> Result<(), String> {
        // INV-OBS-ORDER: an observable rawptr/MMIO scalar access lowers to a barrier
        // CALL (rt_mmio_load/store), not an inline load/store, so `-O2` keeps its
        // program order and never coalesces/elides it. An observable aggregate copy
        // (CopyVal) already lowers to the `rt_copy` barrier, so it falls through.
        if st.observable {
            match &st.kind {
                StatementKind::Assign(local, Rvalue::Load { place, ty }) => {
                    let (a, _) = self.place_addr(place)?;
                    let sty = scalar_of(ty);
                    let size = Layout::scalar_size(sty);
                    let raw = self.call_mmio_load(&a, size);
                    let v = self.canon(&raw, sty);
                    self.store_local(*local, &v);
                    return Ok(());
                }
                StatementKind::Store(place, rv) => {
                    let val = self.eval_rvalue(rv)?;
                    let (a, ty) = self.place_addr(place)?;
                    let sty = scalar_of(&ty);
                    let size = Layout::scalar_size(sty);
                    self.call_mmio_store(&a, &val, size);
                    return Ok(());
                }
                _ => {}
            }
        }
        match &st.kind {
            StatementKind::Assign(local, rv) => {
                let v = self.eval_rvalue(rv)?;
                self.store_local(*local, &v);
            }
            StatementKind::Trace(op) => {
                let v = self.operand(op);
                self.line(&format!("call void @rt_trace(i64 {v})"));
            }
            StatementKind::Store(place, rv) => {
                let v = self.eval_rvalue(rv)?;
                if place.proj.is_empty() {
                    self.store_local(place.root, &v);
                } else {
                    let (a, ty) = self.place_addr(place)?;
                    self.store_scalar(&a, &v, scalar_of(&ty));
                }
            }
            StatementKind::CopyVal { dst, src, ty } => {
                let (s, _) = self.place_addr(src)?;
                let (d, _) = self.place_addr(dst)?;
                self.rt_copy(&d, &s, self.lay.size_of(ty));
            }
            // Emit the value's drop glue at its scheduled point (INV-DROP). A
            // drop-inert local carries no obligation, so its Drop stays a no-op; a
            // needs-drop value recurses through `emit_drop` (struct fields / array
            // elements / active enum variant, reverse order, trace-on-drop via the
            // struct's drop hook), pruned by the static move mask.
            StatementKind::Drop { local, moved } => {
                if self.mf.locals[*local].drop_obligation {
                    let ty = self.mf.locals[*local].ty.clone();
                    let addr = format!("%off{local}");
                    self.emit_drop(&addr, &ty, moved, &mut Vec::new())?;
                }
            }
            StatementKind::BoxOp { dst, inner_ty, result_ty, alloc, value } => {
                self.box_op(dst, inner_ty, result_ty, alloc, value)?;
            }
            StatementKind::UnboxOp { dst, inner_ty, boxed } => {
                self.unbox_op(dst, inner_ty, boxed)?;
            }
            StatementKind::Subslice { dst, src, lo, hi, stride, span } => {
                self.subslice_op(dst, src, lo, hi, *stride, *span)?;
            }
            // Structured concurrency Stage 2 (design 0012 §6; mirrors `lower`): `spawn`
            // creates a real OS thread running the task fn at its (address-taken) body
            // with the args MARSHALLED on the PARENT thread (each a single i64 — a
            // scalar value or an aggregate's flat address; shared-nothing is enforced
            // at compile time), padded to MAX_SPAWN_ARGS. The `scope` braces push/join
            // the barrier frame; the join merges per-task traces in spawn order and
            // re-delivers the spawn-order-first fault (all inside `aot_runtime.c`).
            StatementKind::Spawn { func, args } => {
                let argc = args.len();
                let mut vals: Vec<String> = args.iter().map(|a| self.operand(a)).collect();
                while vals.len() < super::lower::MAX_SPAWN_ARGS {
                    vals.push("0".to_string());
                }
                let argstr =
                    vals.iter().map(|v| format!("i64 {v}")).collect::<Vec<_>>().join(", ");
                self.line(&format!(
                    "call void @rt_spawn(i64 ptrtoint (ptr {} to i64), i64 {argc}, {argstr})",
                    cnf_sym(func)
                ));
            }
            StatementKind::ScopeBegin => self.line("call void @rt_scope_begin()"),
            StatementKind::ScopeEnd => self.line("call void @rt_scope_end()"),
            // The compiler-known `String` intrinsics (design 0013), lowered inline to
            // mirror `mir::interp::collection_op` byte-for-byte (Cranelift `lower`'s
            // twin). `Vec`/`Map` + String `push` (UTF-8) are the remaining slices.
            StatementKind::StrFrom { dst, src } => {
                self.str_from_op(dst, src)?;
            }
            StatementKind::Substr { dst, src, lo, hi, span } => {
                self.substr_op(dst, src, lo, hi, *span)?;
            }
            StatementKind::CollectionOp { dst, op } => {
                self.collection_op(dst, op)?;
            }
        }
        Ok(())
    }
}

/// The static move-mask predicates (mirror `lower::is_moved`/`partially`/`prefix`):
/// a path is dropped only if not covered by, and not a strict prefix cut by, the mask.
fn is_moved(mask: &[Vec<String>], path: &[String]) -> bool {
    mask.iter().any(|m| prefix(m, path))
}
fn partially(mask: &[Vec<String>], path: &[String]) -> bool {
    mask.iter().any(|m| m.len() > path.len() && m[..path.len()] == path[..])
}
fn prefix(a: &[String], b: &[String]) -> bool {
    a.len() <= b.len() && a[..] == b[..a.len()]
}

/// Emit one `cnf_<name>` function, replicating `lower::lower_fn`'s block/region
/// structure (requires/ensures predicate regions, the shared final-return block)
/// and its two-tier storage + aggregate ABI.
#[allow(clippy::too_many_arguments)]
fn emit_fn(
    out: &mut String,
    mf: &MirFn,
    lay: &Layout,
    statics: &HashMap<String, u64>,
    strings: &HashMap<String, u64>,
    drop_hooks: &HashMap<String, String>,
    glue_index: &HashMap<String, usize>,
) -> Result<(), String> {
    let mut e = FnEmit::new(mf, lay, statics, strings, drop_hooks, glue_index);

    let params = (0..mf.num_params)
        .map(|i| format!("i64 %a{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    e.raw(&format!("define internal i64 {}({params}) {{\n", cnf_sym(&mf.name)));
    e.label("entry");
    // Tier-R locals -> alloca (mem2reg-promotable); Tier-F locals -> flat slot.
    for i in 0..mf.locals.len() {
        if e.tier_f[i] {
            let ty = &mf.locals[i].ty;
            let size = lay.size_of(ty).max(1);
            let align = lay.align_of(ty).max(1);
            e.line(&format!("%off{i} = call i64 @rt_stack_alloc(i64 {size}, i64 {align})"));
        } else {
            e.line(&format!("%loc{i} = alloca i64"));
        }
    }
    // Bind params _1..=n: a word param carries its value; an aggregate param arrives
    // as the caller's candor offset and is byte-copied into this frame's slot.
    for i in 0..mf.num_params {
        let pid = 1 + i;
        let pty = mf.locals[pid].ty.clone();
        if is_wordy(&pty) {
            let a = format!("%a{i}");
            e.store_local(pid, &a);
        } else {
            let off = format!("%off{pid}");
            let a = format!("%a{i}");
            e.rt_copy(&off, &a, lay.size_of(&pty));
        }
    }

    let final_return = "ret_final".to_string();
    let has_ens = !mf.ensures.is_empty();
    let ensures_start = "ens_start".to_string();
    let body_ret = if has_ens { ensures_start.clone() } else { final_return.clone() };
    let req_labels: Vec<String> = (0..mf.requires.len()).map(|i| format!("req{i}")).collect();
    let ens_labels: Vec<String> = (0..mf.ensures.len()).map(|i| format!("ens{i}")).collect();

    let mut ret_target: Vec<Option<String>> = vec![None; mf.blocks.len()];
    for (i, pred) in mf.requires.iter().enumerate() {
        assign_region(mf, pred.entry, &req_labels[i], &mut ret_target);
    }
    assign_region(mf, mf.entry, &body_ret, &mut ret_target);
    for (i, pred) in mf.ensures.iter().enumerate() {
        assign_region(mf, pred.entry, &ens_labels[i], &mut ret_target);
    }

    let first = match mf.requires.first() {
        Some(p) => format!("mbb{}", p.entry),
        None => format!("mbb{}", mf.entry),
    };
    e.line(&format!("br label %{first}"));

    for (i, pred) in mf.requires.iter().enumerate() {
        e.label(&req_labels[i]);
        let v = e.load_local(pred.value);
        let c = e.t();
        e.line(&format!("{c} = trunc i64 {v} to i1"));
        let ok = if i + 1 < mf.requires.len() {
            format!("mbb{}", mf.requires[i + 1].entry)
        } else {
            format!("mbb{}", mf.entry)
        };
        let fl = e.l();
        e.line(&format!("br i1 {c}, label %{ok}, label %{fl}"));
        e.label(&fl);
        e.emit_fault(pred.kind, pred.span);
    }

    if has_ens {
        e.label(&ensures_start);
        if let Some(rl) = mf.result_local {
            let v = e.load_local(0);
            e.store_local(rl, &v);
        }
        e.line(&format!("br label %mbb{}", mf.ensures[0].entry));
    }
    for (i, pred) in mf.ensures.iter().enumerate() {
        e.label(&ens_labels[i]);
        let v = e.load_local(pred.value);
        let c = e.t();
        e.line(&format!("{c} = trunc i64 {v} to i1"));
        let ok = if i + 1 < mf.ensures.len() {
            format!("mbb{}", mf.ensures[i + 1].entry)
        } else {
            final_return.clone()
        };
        let fl = e.l();
        e.line(&format!("br i1 {c}, label %{ok}, label %{fl}"));
        e.label(&fl);
        e.emit_fault(pred.kind, pred.span);
    }

    e.label(&final_return);
    // Convention (B): a word return delivers _0's value; an aggregate return
    // delivers the candor offset of _0's caller-visible slot.
    if is_wordy(&mf.locals[0].ty) {
        let v = e.load_local(0);
        e.line(&format!("ret i64 {v}"));
    } else {
        e.line("ret i64 %off0");
    }

    for (bid, block) in mf.blocks.iter().enumerate() {
        e.label(&format!("mbb{bid}"));
        if ret_target[bid].is_none() {
            e.line("unreachable");
            continue;
        }
        for st in &block.stmts {
            e.lower_stmt(st)?;
        }
        match &block.term {
            Terminator::Goto(n) => e.line(&format!("br label %mbb{n}")),
            Terminator::Branch { cond, then_bb, else_bb } => {
                let c = e.operand(cond);
                let ci = e.t();
                e.line(&format!("{ci} = trunc i64 {c} to i1"));
                e.line(&format!(
                    "br i1 {ci}, label %mbb{then_bb}, label %mbb{else_bb}"
                ));
            }
            Terminator::Return => {
                let tgt = ret_target[bid].clone().unwrap();
                e.line(&format!("br label %{tgt}"));
            }
            Terminator::Fault(edge) => e.emit_fault(edge.kind, edge.span),
        }
    }

    e.raw("}\n\n");
    out.push_str(&e.out);
    Ok(())
}

/// A canonical key for a monomorphized type (mirrors `lower::type_key`), used to
/// map a Box pointee to its drop-glue index.
fn type_key(ty: &Type) -> String {
    format!("{ty:?}")
}

/// Emit the fn-pointer dispatch table: a module global `[N x ptr]` whose slot `i`
/// is the address of `fn_ptrs[i]`'s compiled body. A fn value baked into a
/// vtable/handle static is the `u64` id `i`; an indirect / vtable call indexes
/// this table to recover the callee address (the AOT twin of
/// `object::compile_object`'s `cn_fnptr_table` data object).
fn emit_fnptr_table(out: &mut String, prog: &MirProgram) {
    if prog.fn_ptrs.is_empty() {
        return;
    }
    let n = prog.fn_ptrs.len();
    let elems: Vec<String> = prog
        .fn_ptrs
        .iter()
        .map(|name| {
            if prog.get(name).is_some() {
                format!("ptr {}", cnf_sym(name))
            } else {
                "ptr null".to_string()
            }
        })
        .collect();
    out.push_str(&format!(
        "@\"cn_fnptr_table\" = internal global [{n} x ptr] [{}]\n\n",
        elems.join(", ")
    ));
}

/// Emit a per-Box-pointee drop-glue function `@__drop_glue_<i>(addr)`: run the
/// value-of-`ty` drop schedule at the address param, then return 0 (INV-DROP,
/// design 0010 §5). Recursive Box types bottom out at the runtime null-pointer
/// guard inside `drop_box`, so this is finite recursion, not infinite unrolling.
#[allow(clippy::too_many_arguments)]
fn emit_glue_fn(
    out: &mut String,
    i: usize,
    ty: &Type,
    lay: &Layout,
    statics: &HashMap<String, u64>,
    strings: &HashMap<String, u64>,
    drop_hooks: &HashMap<String, String>,
    glue_index: &HashMap<String, usize>,
) -> Result<(), String> {
    let glue_mf = MirFn {
        name: format!("__drop_glue_{i}"),
        num_params: 1,
        result_local: None,
        locals: Vec::new(),
        blocks: Vec::new(),
        entry: 0,
        requires: Vec::new(),
        ensures: Vec::new(),
        replay: ReplayPolicy::Precise,
    };
    let mut e = FnEmit::new(&glue_mf, lay, statics, strings, drop_hooks, glue_index);
    e.raw(&format!("define internal i64 @\"__drop_glue_{i}\"(i64 %a0) {{\n"));
    e.label("entry");
    e.emit_drop("%a0", ty, &[], &mut Vec::new())?;
    e.line("ret i64 0");
    e.raw("}\n\n");
    out.push_str(&e.out);
    Ok(())
}

/// Build the emitted module with `clang -O2` and link it against the reused static
/// C runtime into a standalone native executable at `out`.
pub fn link_ll(ll: &str, out: &Path) -> Result<(), String> {
    let tmp = std::env::temp_dir().join(format!(
        "candor-llvm-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let ll_path = tmp.join("candor.ll");
    let rt_path = tmp.join("aot_runtime.c");
    std::fs::write(&ll_path, ll).map_err(|e| e.to_string())?;
    std::fs::write(&rt_path, RUNTIME_C).map_err(|e| e.to_string())?;

    let status = Command::new("clang")
        .arg("-O2")
        .arg("-Wno-override-module")
        .arg(&ll_path)
        .arg(&rt_path)
        .arg("-o")
        .arg(out)
        .arg("-no-pie")
        .arg("-pthread")
        .args(super::object::linker_select_args())
        .status()
        .map_err(|e| format!("could not invoke clang: {e}"))?;

    let _ = std::fs::remove_dir_all(&tmp);
    if !status.success() {
        return Err(format!("clang -O2 failed with status {status}"));
    }
    Ok(())
}
