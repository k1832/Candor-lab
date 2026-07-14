//! Stage D — the one explicit **MIR-level R1 rewrite**, with a per-rewrite
//! validator that makes INV-R1-ONLY executable (design 0010 §2, §5;
//! fault-window-formalization §5.1 R1).
//!
//! This proves the *license machinery*: an optimization is legal iff it is an
//! R1/R2/R3 rewrite over the MIR, and illegal otherwise (INV-R1-ONLY). The pass
//! here is **dead-local elimination** — R1's "eliminate an independent internal
//! (`τ`) step if dead" clause, in its most conservative form:
//!
//! * **R1 is `τ`-steps only.** We remove only statements that are **not
//!   observable** (never an MMIO/rawptr access, never `trace`) — observables are
//!   effect-order-total (§5.1) and may never be eliminated.
//! * **fault-independent only.** We remove only `Assign(local, rv)` whose `rv` is
//!   provably **side-effect-free AND non-fault-capable** (a genuine `τ`-step, per
//!   [`is_pure_nonfaulting`]). A checked arithmetic/conv/index op is *fault-capable*
//!   — its fault is a potential observable (§2.1) — so eliminating it could
//!   suppress a fault; it is therefore **never** removed, even when its result is
//!   dead. This is the exact discipline the formalization demands and a
//!   UB-exploiting DCE would violate.
//! * **dependence-independent only.** We remove an assignment only when its
//!   destination local is **dead** — read by no operation anywhere in the function
//!   (`→`-independent: no op depends on it, §3.4). Liveness is computed
//!   conservatively: every mention of a local other than as a bare `Assign`
//!   *target* counts as a use.
//! * **never across a fault edge in the forbidden direction.** The rewrite removes
//!   a statement *in place*; it never moves an operation past a fault edge or an
//!   observable, so R1 side-condition (ii) (no fault-capable op before its `e⁻`)
//!   holds vacuously.
//!
//! The four-engine gate (`tests/stage_d.rs`) then proves *empirically* that the
//! rewritten MIR yields the identical `(k, s, θ)` — the license, validated.

use std::collections::HashSet;

use crate::ast::UnOp;
use crate::mir::{
    BasicBlock, CollOp, LocalId, MirFn, MirProgram, Operand, Place, Proj, Rvalue, Statement,
    StatementKind, Terminator,
};

/// A record of one applied R1 rewrite (for reporting / testing the machinery).
#[derive(Debug, Clone)]
pub struct Rewrite {
    pub func: String,
    pub block: usize,
    pub local: LocalId,
    pub kind: &'static str,
}

/// The result of the optimizer: the rewritten program plus the validated log.
pub struct Optimized {
    pub prog: MirProgram,
    pub rewrites: Vec<Rewrite>,
}

/// Run the R1 pass over every function. Each removal is checked against the R1
/// side conditions by [`validate_r1_removal`] *before* it is applied (the
/// INV-R1-ONLY enforcement made executable — a violating candidate is never
/// removed and, in debug, panics).
pub fn optimize(mut prog: MirProgram) -> Optimized {
    let mut rewrites = Vec::new();
    for f in &mut prog.fns {
        dce_fn(f, &mut rewrites);
        // The rewrite must preserve every MIR invariant (INV-CHECK / INV-OBS-ORDER
        // / INV-FAULT-ID unchanged; only dead `τ`-assigns dropped).
        crate::mir::check_invariants(f);
    }
    Optimized { prog, rewrites }
}

/// Dead-local elimination over one function, iterated to a fixpoint (removing a
/// dead store can free the pure temp that fed it, so a second pass collects it).
fn dce_fn(f: &mut MirFn, log: &mut Vec<Rewrite>) {
    let nparams = f.num_params;
    let name = f.name.clone();
    loop {
        let live = live_locals(f);
        let mut removed_any = false;
        for (bi, block) in f.blocks.iter_mut().enumerate() {
            let mut new_log: Vec<Rewrite> = Vec::new();
            block.stmts.retain(|st| {
                // R1 targets a dead pure `τ`-definition of a scalar local: an
                // `Assign(temp, rv)` or a bare-root `Store(local, rv)` (a `let`
                // binding). Projected stores (field/index/deref) are NOT bare defs
                // and are left alone (their root's address/value is used).
                let (local, rv) = match &st.kind {
                    StatementKind::Assign(l, rv) => (*l, rv),
                    StatementKind::Store(place, rv) if place.proj.is_empty() => (place.root, rv),
                    _ => return true,
                };
                let candidate = !st.observable
                    && local != 0
                    && local > nparams
                    && !live.contains(&local)
                    && is_pure_nonfaulting(rv);
                if candidate {
                    // The validator asserts the R1 side conditions for THIS rewrite
                    // (INV-R1-ONLY made executable).
                    debug_assert!(
                        validate_r1_removal(st, local, &live, nparams),
                        "INV-R1-ONLY: illegal DCE candidate in {name}#bb{bi} of _{local}"
                    );
                    new_log.push(Rewrite { func: name.clone(), block: bi, local, kind: rv_kind(rv) });
                    return false;
                }
                true
            });
            if !new_log.is_empty() {
                removed_any = true;
                log.extend(new_log);
            }
        }
        if !removed_any {
            break;
        }
    }
}

/// The executable INV-R1-ONLY check: assert every R1 side condition holds for a
/// candidate removal (a dead, non-observable, pure-non-faulting `τ`-assign).
pub fn validate_r1_removal(
    st: &Statement,
    local: LocalId,
    live: &HashSet<LocalId>,
    nparams: usize,
) -> bool {
    // (i) τ-steps only — never an observable.
    if st.observable {
        return false;
    }
    let rv = match &st.kind {
        StatementKind::Assign(l, rv) if *l == local => rv,
        StatementKind::Store(place, rv) if place.proj.is_empty() && place.root == local => rv,
        _ => return false,
    };
    // (ii) fault-independent: side-effect-free AND non-fault-capable.
    if !is_pure_nonfaulting(rv) {
        return false;
    }
    // (iii) dependence-independent: the destination is dead (read by no op).
    if live.contains(&local) {
        return false;
    }
    // (iv) never the return place (_0) or a parameter (_1..=nparams).
    if local == 0 || local <= nparams {
        return false;
    }
    true
}

fn rv_kind(rv: &Rvalue) -> &'static str {
    match rv {
        Rvalue::Use(_) => "use",
        Rvalue::Cmp { .. } => "cmp",
        Rvalue::IsNull(_) => "is_null",
        Rvalue::Ref(_) => "ref",
        Rvalue::Load { .. } => "load",
        Rvalue::PtrArith { .. } => "ptr_arith",
        Rvalue::StaticAddr(_) => "static_addr",
        Rvalue::StrAddr(_) => "str_addr",
        Rvalue::Un { .. } => "un",
        _ => "other",
    }
}

/// Is `rv` a pure, **non-fault-capable** `τ`-step whose elimination (when its
/// result is dead) can suppress neither an observable nor a fault? Deliberately
/// conservative: anything fault-capable (checked/`Div`/`Rem`/`Shl`/`Shr`, a
/// bounds-faulting `Index` projection, a checked `Conv`/`Neg`) or side-effecting
/// (`Call`/`CallIndirect`) is excluded and therefore never removed.
fn is_pure_nonfaulting(rv: &Rvalue) -> bool {
    match rv {
        Rvalue::Use(_)
        | Rvalue::Cmp { .. }
        | Rvalue::IsNull(_)
        | Rvalue::PtrArith { .. }
        | Rvalue::StaticAddr(_)
        | Rvalue::StrAddr(_)
        // bitcast and sqrt are pure, total ops -- never fault.
        | Rvalue::Bitcast { .. }
        | Rvalue::Sqrt { .. } => true,
        // not/bitnot never fault (fault is always None); neg is fault-capable.
        Rvalue::Un { op: UnOp::Not | UnOp::BitNot, .. } => true,
        // A ref/load is pure and non-faulting only when its place has no
        // bounds-faulting `Index` projection.
        Rvalue::Ref(place) | Rvalue::Load { place, .. } => !place_has_index(place),
        _ => false,
    }
}

fn place_has_index(place: &Place) -> bool {
    place.proj.iter().any(|p| matches!(p, Proj::Index { .. }))
}

/// Conservative liveness: a local is LIVE if it appears anywhere except as the
/// bare destination of an `Assign`. Every read/borrow/drop/projection-root
/// mention counts, so a local we call dead is genuinely `→`-independent.
fn live_locals(f: &MirFn) -> HashSet<LocalId> {
    let mut live = HashSet::new();
    for b in &f.blocks {
        for st in &b.stmts {
            stmt_uses(&st.kind, &mut live);
        }
        term_uses(&b.term, &mut live);
    }
    // Predicate value locals and the ensures `result` local are read implicitly.
    for p in f.requires.iter().chain(f.ensures.iter()) {
        live.insert(p.value);
    }
    if let Some(rl) = f.result_local {
        live.insert(rl);
    }
    live
}

fn op_use(op: &Operand, live: &mut HashSet<LocalId>) {
    if let Operand::Local(id) = op {
        live.insert(*id);
    }
}

fn place_use(place: &Place, live: &mut HashSet<LocalId>) {
    // The root's address/value is consumed by any projected access, and even a
    // bare-root place used as a Store/CopyVal target is treated as a use
    // (conservative — keeps more locals live).
    live.insert(place.root);
    for p in &place.proj {
        if let Proj::Index { index, .. } = p {
            op_use(index, live);
        }
    }
}

fn rvalue_uses(rv: &Rvalue, live: &mut HashSet<LocalId>) {
    match rv {
        Rvalue::Use(o) | Rvalue::IsNull(o) => op_use(o, live),
        Rvalue::Bin { l, r, .. } | Rvalue::Cmp { l, r, .. } => {
            op_use(l, live);
            op_use(r, live);
        }
        Rvalue::Un { v, .. } | Rvalue::Conv { v, .. } | Rvalue::Bitcast { v, .. } | Rvalue::Sqrt { v, .. } => op_use(v, live),
        Rvalue::Ref(p) | Rvalue::Load { place: p, .. } => place_use(p, live),
        Rvalue::Call { args, .. } => args.iter().for_each(|a| op_use(a, live)),
        Rvalue::CallIndirect { func, args } => {
            op_use(func, live);
            args.iter().for_each(|a| op_use(a, live));
        }
        Rvalue::PtrArith { base, index, .. } => {
            op_use(base, live);
            op_use(index, live);
        }
        Rvalue::StaticAddr(_) | Rvalue::StrAddr(_) => {}
    }
}

fn stmt_uses(kind: &StatementKind, live: &mut HashSet<LocalId>) {
    match kind {
        // The `Assign` TARGET local is a write, not a use; its rvalue's operands are.
        StatementKind::Assign(_, rv) => rvalue_uses(rv, live),
        StatementKind::Trace(o) => op_use(o, live),
        StatementKind::Store(place, rv) => {
            // A bare-root store `local = rv` DEFINES root (not a use); a projected
            // store `local.f = rv` / `(deref p) = rv` USES root's address/value.
            if !place.proj.is_empty() {
                place_use(place, live);
            } else {
                for p in &place.proj {
                    if let Proj::Index { index, .. } = p {
                        op_use(index, live);
                    }
                }
            }
            rvalue_uses(rv, live);
        }
        StatementKind::CopyVal { dst, src, .. } => {
            place_use(dst, live);
            place_use(src, live);
        }
        StatementKind::Drop { local, .. } => {
            live.insert(*local);
        }
        StatementKind::BoxOp { dst, alloc, value, .. } => {
            place_use(dst, live);
            op_use(alloc, live);
            place_use(value, live);
        }
        StatementKind::UnboxOp { dst, boxed, .. } => {
            place_use(dst, live);
            place_use(boxed, live);
        }
        StatementKind::Subslice { dst, src, lo, hi, .. } => {
            place_use(dst, live);
            place_use(src, live);
            op_use(lo, live);
            op_use(hi, live);
        }
        StatementKind::StrFrom { dst, src } => {
            place_use(dst, live);
            place_use(src, live);
        }
        StatementKind::Substr { dst, src, lo, hi, .. } => {
            place_use(dst, live);
            place_use(src, live);
            op_use(lo, live);
            op_use(hi, live);
        }
        // A collection intrinsic reads its receiver base, index/key/value operands
        // and places, and writes its result place — all live uses.
        StatementKind::CollectionOp { dst, op } => {
            place_use(dst, live);
            collop_uses(op, live);
        }
        // A `spawn`'s marshalled args are live uses (they cross into the task);
        // the scope markers reference no locals.
        StatementKind::Spawn { args, .. } => {
            for a in args {
                op_use(a, live);
            }
        }
        StatementKind::ScopeBegin | StatementKind::ScopeEnd => {}
    }
}

fn collop_uses(op: &CollOp, live: &mut HashSet<LocalId>) {
    match op {
        CollOp::New { alloc } => op_use(alloc, live),
        CollOp::VecPush { base, value, .. } => {
            op_use(base, live);
            place_use(value, live);
        }
        CollOp::VecPop { base, .. } => op_use(base, live),
        CollOp::VecGet { base, index, .. } => {
            op_use(base, live);
            op_use(index, live);
        }
        CollOp::VecSet { base, index, value, .. } => {
            op_use(base, live);
            op_use(index, live);
            place_use(value, live);
        }
        CollOp::MapInsert { base, key, value, .. } => {
            op_use(base, live);
            place_use(key, live);
            place_use(value, live);
        }
        CollOp::MapContains { base, key, .. } | CollOp::MapGet { base, key, .. } => {
            op_use(base, live);
            place_use(key, live);
        }
        CollOp::StringPush { base, ch, .. } => {
            op_use(base, live);
            op_use(ch, live);
        }
        CollOp::StringAppend { base, view, .. } => {
            op_use(base, live);
            place_use(view, live);
        }
        CollOp::StringAsStr { base } => op_use(base, live),
    }
}

fn term_uses(term: &Terminator, live: &mut HashSet<LocalId>) {
    if let Terminator::Branch { cond, .. } = term {
        op_use(cond, live);
    }
}

/// Total statement count of a program (before/after the pass — reporting only).
pub fn stmt_count(prog: &MirProgram) -> usize {
    prog.fns.iter().flat_map(|f| f.blocks.iter()).map(|b: &BasicBlock| b.stmts.len()).sum()
}
