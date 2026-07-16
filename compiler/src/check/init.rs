//! Definite assignment + move checking as one integrated place-state analysis
//! (design 0001 §1.6, §7.4). A forward dataflow over the shared CFG.
//!
//! The asymmetry the spec demands (§1.6 vs §7.4) is realized in the lattice:
//! definite assignment only requires all-paths-initialized *before a read*
//! (`MaybeInit` is legal until read), whereas move state must *agree* at every
//! join — a place live on one path and moved on another is an error even if
//! never read again (`join_st` flags it).
//!
//! The dual of that move-join rule (the scope-exit path-independence rule,
//! finding 2026-07-07) closes the initialization dimension for types whose drop
//! is observable: at a *needs-drop* place's drop point (a `ScopeExit` action) a
//! `MaybeInit` state is **E0309** — otherwise the interpreter would decide the
//! drop from a runtime flag. Drop-inert types stay exempt (`MaybeInit` at exit
//! is harmless — their drop is a no-op).

use std::collections::BTreeSet;
use std::collections::VecDeque;

use crate::check::dataflow::*;
use crate::diag::Diag;
use crate::span::Span;

/// Run the analysis, pushing diagnostics. Returns the box-drop sites (span +
/// provenance message) the checker's own drop scheduling produces — a scope-exit
/// drop, a reassignment/out drop, or a function-exit drop of a still-live
/// `Box`-bearing place — each of which frees and so makes the enclosing function
/// allocator-effecting (finding 4 of 2026-07-07; §6.2/§6.3).
pub fn analyze(
    cfg: &Cfg,
    entry: &FlowState,
    out_params: &[String],
    param_box: &[(String, Vec<Vec<String>>)],
    diags: &mut Vec<Diag>,
) -> Vec<(Span, String)> {
    let n = cfg.blocks.len();
    let reach = reachable(cfg);

    // --- Fixpoint (silent) -------------------------------------------------
    // A forward *must*-analysis needs its join to meet only over predecessors
    // whose out-state has actually been computed. `visited[p]` records that:
    // an unvisited (not-yet-computed) predecessor edge — notably a loop
    // back-edge on the first pass — contributes identity/TOP to the meet, not
    // the pessimistic bottom (`Uninit`) state, so a loop body that reads a
    // value initialized before the loop is not falsely degraded (E0304).
    let mut out_state: Vec<FlowState> = vec![FlowState::new(); n];
    let mut visited = vec![false; n];
    let order: Vec<BlockId> = (0..n).filter(|b| reach.contains(b)).collect();
    // The fixpoint iterates in reverse-postorder (RPO), not block-creation order,
    // so a block is (re)computed only after its non-back-edge predecessors are
    // visited. Iterating in creation order let a back-edge continuation block
    // whose only predecessor is a HIGHER-numbered block seed itself from `entry`
    // (bottom/Uninit) on the first pass, poisoning the loop header and driving a
    // period-2 oscillation between `Init` and `MaybeInit` that never converged
    // (surfaced by the `for`-desugar's `loop { match { .. break } }` shape,
    // 2026-07-08). RPO makes the forward must-analysis converge monotonically.
    let rpo_order = rpo(cfg, &reach);
    let mut changed = true;
    let mut guard = 0;
    while changed && guard < n * 4 + 16 {
        changed = false;
        guard += 1;
        for &b in &rpo_order {
            let in_b = incoming(cfg, b, entry, &out_state, &reach, &visited, None);
            let mut st = in_b;
            for a in &cfg.blocks[b].actions {
                apply(&mut st, a, out_params, None, None);
            }
            visited[b] = true;
            if st != out_state[b] {
                out_state[b] = st;
                changed = true;
            }
        }
    }

    // --- Reporting pass ----------------------------------------------------
    let mut alloc_sites: Vec<(Span, String)> = Vec::new();
    for &b in &order {
        let mut disagree: Vec<String> = Vec::new();
        let in_b = incoming(cfg, b, entry, &out_state, &reach, &visited, Some(&mut disagree));
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for key in disagree {
            if seen.insert(key.clone()) {
                diags.push(
                    Diag::error(
                        "E0302",
                        format!("place `{key}` has inconsistent move state at this join"),
                        cfg.blocks[b].join_span,
                    )
                    .with_note(
                        "it is moved on one incoming path and still live on another (§1.6)",
                        None,
                    ),
                );
            }
        }
        let mut st = in_b;
        for a in &cfg.blocks[b].actions {
            apply(&mut st, a, out_params, Some(diags), Some(&mut alloc_sites));
        }
        if matches!(cfg.blocks[b].term, Term::Return | Term::FallThrough) {
            for op in out_params {
                if st.get(&Place::local(op.clone())) != St::Init {
                    diags.push(
                        Diag::error(
                            "E0305",
                            format!("`out` parameter `{op}` is not assigned on this return path"),
                            cfg.blocks[b].join_span,
                        )
                        .with_note("an `out` slot must be initialized on every normal return (§3.1)", None),
                    );
                }
            }
            // Function-exit drop of an owned `Box`-bearing parameter that is still
            // live (not moved out on this path): the box is freed here, so the
            // function is allocator-effecting (finding 4; §6.3).
            for (name, paths) in param_box {
                if let Some(msg) = box_drop_here(&st, name, &[], paths) {
                    alloc_sites.push((cfg.blocks[b].join_span, msg));
                }
            }
        }
    }
    alloc_sites
}

/// If any `Box`-reaching sub-place of `root`+`base_proj` (per `paths`) is still
/// live (`Init`/`MaybeInit`) in `st`, its drop frees: return a provenance message
/// naming the freeing place. `None` when every box sub-place is already gone
/// (moved out / never initialized) — no free happens here.
fn box_drop_here(
    st: &FlowState,
    root: &str,
    base_proj: &[Proj],
    paths: &[Vec<String>],
) -> Option<String> {
    for bp in paths {
        let mut proj = base_proj.to_vec();
        for f in bp {
            proj.push(Proj::Field(f.clone()));
        }
        let pl = Place {
            root: root.to_string(),
            proj,
        };
        if matches!(st.get(&pl), St::Init | St::MaybeInit) {
            return Some(format!(
                "drop of `Box`-bearing place `{}` frees (§6.2/§6.3)",
                pl.display()
            ));
        }
    }
    None
}

fn incoming(
    cfg: &Cfg,
    b: BlockId,
    entry: &FlowState,
    out_state: &[FlowState],
    reach: &BTreeSet<BlockId>,
    visited: &[bool],
    disagree: Option<&mut Vec<String>>,
) -> FlowState {
    if b == cfg.entry {
        return entry.clone();
    }
    // Meet only over predecessors that are both reachable and already computed:
    // a not-yet-visited edge is treated as identity/TOP (omitted from the meet),
    // never as bottom. At the fixpoint every reachable predecessor is visited,
    // so the reporting pass still meets over *all* incoming paths — a genuine
    // uninit path (E0304) and a move-state disagreement (§1.6) are both kept.
    let preds: Vec<BlockId> = cfg.preds[b]
        .iter()
        .copied()
        .filter(|p| reach.contains(p) && visited[*p])
        .collect();
    if preds.is_empty() {
        return entry.clone();
    }
    let mut acc = out_state[preds[0]].clone();
    let mut local_dis: Vec<String> = Vec::new();
    for &p in &preds[1..] {
        acc.join(&out_state[p], &mut local_dis);
    }
    if let Some(d) = disagree {
        d.extend(local_dis);
    }
    acc
}

fn apply(
    st: &mut FlowState,
    a: &Action,
    out_params: &[String],
    mut report: Option<&mut Vec<Diag>>,
    mut alloc: Option<&mut Vec<(Span, String)>>,
) {
    let opaque = !a.place.is_direct();
    let root = Place::local(a.place.root.clone());
    let key = if opaque { &root } else { &a.place };
    match &a.access {
        Access::Read | Access::Borrow(_) => {
            if let Some(d) = report.as_deref_mut() {
                require_init(st, key, out_params, a.span, a.contract, d);
            }
        }
        Access::Move {
            movable: _,
            drop_hooked_partial,
        } => {
            if opaque {
                // Moving a non-copy value out of a place whose projection path
                // crosses a `deref` or index is rejected (ruling of soundness
                // review #2, 2026-07-07). A copy value read through such a path
                // is an `Access::Read`, not a `Move`, so reaching this arm means
                // a genuine non-copy move out of an opaque place: through a
                // borrow it would hollow out the lender's value; through a `Box`
                // the defined extraction is `unbox`; array elements are not
                // move-trackable at index granularity in the prototype's place
                // model. (0001 §1.6/§2.1; 0003 §2.1/§2.4.)
                if let Some(d) = report.as_deref_mut() {
                    require_init(st, &root, out_params, a.span, a.contract, d);
                    d.push(
                        Diag::error(
                            "E0310",
                            format!(
                                "cannot move a non-copy value out of `{}`: its place path goes through a `deref` or index",
                                a.place.display()
                            ),
                            a.span,
                        )
                        .with_note(
                            "moving through a `deref` would hollow out the value behind the borrow/pointer, which is never granted (§2.1); use `unbox` to extract a `Box` pointee, or move the whole binding",
                            None,
                        )
                        .with_note(
                            "reading a `copy` value through `deref`/index is unaffected (it copies); only non-copy moves are rejected — array-element moves are limited to copy element types in the prototype (§1.6; review #2, 2026-07-07)",
                            None,
                        ),
                    );
                }
                // Mark the whole root moved — the interpreter's conservative
                // opaque-move behavior. The program is already E0310-rejected;
                // matching the interpreter's move state here removes the
                // checker/interpreter divergence that finding 2 (the false-E0401
                // free of a box that runtime leaks) grew out of.
                st.set(&root, St::Moved);
            } else {
                if let Some(d) = report.as_deref_mut() {
                    require_init(st, key, out_params, a.span, a.contract, d);
                    if *drop_hooked_partial {
                        d.push(
                            Diag::error(
                                "E0303",
                                format!("cannot partially move out of `{}` (its type has a `drop` hook)", a.place.display()),
                                a.span,
                            )
                            .with_note("move or borrow the whole value; a drop-hooked type cannot be left partial (§1.6)", None),
                        );
                    }
                }
                st.set(&a.place, St::Moved);
            }
        }
        Access::Assign { needs_drop, box_paths } => {
            if opaque {
                if let Some(d) = report.as_deref_mut() {
                    require_init(st, &root, out_params, a.span, a.contract, d);
                }
            } else {
                // The OLD value at this place is dropped by the reassignment
                // (§1.5). Check its drop point BEFORE overwriting the state.
                // E0309 is scoped to a *whole-binding* reassignment (per the
                // disposition); the box-drop (free) side applies to any direct
                // place, including a box-typed field, since freeing it is
                // allocator work regardless of granularity (finding 4).
                if *needs_drop && a.place.proj.is_empty() {
                    if let Some(d) = report.as_deref_mut() {
                        if st.get(&a.place) == St::MaybeInit {
                            d.push(reassign_drop_diag(&a.place, a.span));
                        }
                    }
                }
                if let Some(al) = alloc.as_deref_mut() {
                    if let Some(msg) = box_drop_here(st, &a.place.root, &a.place.proj, box_paths) {
                        al.push((a.span, msg));
                    }
                }
                st.set(&a.place, St::Init);
            }
        }
        Access::OutArg { needs_drop, box_paths } => {
            if opaque {
                if let Some(d) = report.as_deref_mut() {
                    require_init(st, &root, out_params, a.span, a.contract, d);
                }
            } else {
                // Passing a place as `out` drops its old value at the call site —
                // the same drop point as a reassignment (finding 2).
                if *needs_drop && a.place.proj.is_empty() {
                    if let Some(d) = report.as_deref_mut() {
                        if st.get(&a.place) == St::MaybeInit {
                            d.push(out_drop_diag(&a.place, a.span));
                        }
                    }
                }
                if let Some(al) = alloc.as_deref_mut() {
                    if let Some(msg) = box_drop_here(st, &a.place.root, &a.place.proj, box_paths) {
                        al.push((a.span, msg));
                    }
                }
                st.set(&a.place, St::Init);
            }
        }
        Access::Decl => {
            st.set(&a.place, St::Uninit);
        }
        Access::ScopeExit { box_paths } => {
            // The dual of §1.6's move-join rule (finding 2026-07-07): at a
            // needs-drop place's drop point its initialization must be
            // path-independent. `MaybeInit` here means it is initialized on
            // some incoming paths and not others, so the interpreter would
            // decide the drop from a runtime flag — rejected as E0309.
            // (`Init` always-drops, `Uninit`/`Moved` never-drop: both static.)
            if let Some(d) = report {
                if st.get(&a.place) == St::MaybeInit {
                    d.push(
                        Diag::error(
                            "E0309",
                            format!(
                                "place `{}` may be initialized here but not on every path reaching this scope exit",
                                a.place.display()
                            ),
                            a.span,
                        )
                        .with_note(
                            "its type runs drop work, so its drop must be a static fact of this program point, not a runtime decision (§1.6 dual)",
                            None,
                        )
                        .with_note(
                            "initialize it on every path, consume it on every path, or narrow its scope (finding 2026-07-07)",
                            None,
                        ),
                    );
                }
            }
            // A still-live `Box`-bearing local dropped here frees (finding 4).
            if let Some(al) = alloc {
                if let Some(msg) = box_drop_here(st, &a.place.root, &a.place.proj, box_paths) {
                    al.push((a.span, msg));
                }
            }
        }
    }
}

/// The reassignment drop-point diagnostic (finding 2 of 2026-07-07): whole-binding
/// reassignment of a needs-drop place in a `MaybeInit` state — the old value's
/// drop would be a runtime decision. Same code as the scope-exit rule (E0309),
/// message distinguishing the reassignment drop point.
fn reassign_drop_diag(place: &Place, span: Span) -> Diag {
    Diag::error(
        "E0309",
        format!(
            "reassigning `{}` here drops its old value, but it may be initialized on only some paths reaching this reassignment",
            place.display()
        ),
        span,
    )
    .with_note(
        "its type runs drop work, so the drop dropped by this reassignment must be a static fact, not a runtime decision (§1.5/§1.6 rule 3)",
        None,
    )
    .with_note(
        "initialize it on every path, consume it on every path, or narrow its scope (finding 2 of 2026-07-07)",
        None,
    )
}

/// The out-argument drop-point diagnostic (finding 2): passing a needs-drop place
/// in a `MaybeInit` state as `out` drops its old value at the call site.
fn out_drop_diag(place: &Place, span: Span) -> Diag {
    Diag::error(
        "E0309",
        format!(
            "passing `{}` as `out` drops its old value at the call, but it may be initialized on only some paths reaching this call",
            place.display()
        ),
        span,
    )
    .with_note(
        "an `out` argument is the same drop point as a reassignment; its drop must be a static fact, not a runtime decision (§3.1/§1.5)",
        None,
    )
    .with_note(
        "initialize it on every path, consume it on every path, or narrow its scope (finding 2 of 2026-07-07)",
        None,
    )
}

fn require_init(
    st: &FlowState,
    place: &Place,
    out_params: &[String],
    span: Span,
    contract: bool,
    diags: &mut Vec<Diag>,
) {
    let s = st.get(place);
    if s == St::Init {
        return;
    }
    let is_out = out_params.contains(&place.root) && place.is_direct() && place.proj.is_empty();
    // Contract clauses (`ensures`) are analyzed against the post-body state at
    // each return point, so a read of a body-moved place reads the same as if
    // written at the return (review #3, 2026-07-07): note the contract position.
    let contract_note = "this access is in an `ensures` clause, checked against the post-body state at the return point (review #3, 2026-07-07)";
    match s {
        St::Moved => {
            let mut d = Diag::error(
                "E0301",
                format!("use of moved value `{}`", place.display()),
                span,
            )
            .with_note("value was moved out on an earlier path (§1.2)", None);
            if contract {
                d = d.with_note(contract_note, None);
            }
            diags.push(d);
        }
        St::Uninit | St::MaybeInit => {
            if is_out {
                diags.push(
                    Diag::error(
                        "E0306",
                        format!("`out` parameter `{}` read before its first assignment", place.display()),
                        span,
                    )
                    .with_note("an `out` slot may not be read before it is assigned (§3.1)", None),
                );
            } else {
                diags.push(
                    Diag::error(
                        "E0304",
                        format!("use of possibly-uninitialized value `{}`", place.display()),
                        span,
                    )
                    .with_note("not initialized on every path reaching here (§7.4)", None),
                );
            }
        }
        St::Init => {}
    }
}

/// Reverse-postorder of the reachable CFG from the entry (design 0001 §7.4): the
/// canonical iteration order for a forward dataflow analysis, so back-edges are
/// the only edges that point backward and Gauss-Seidel converges without
/// oscillation.
fn rpo(cfg: &Cfg, reach: &BTreeSet<BlockId>) -> Vec<BlockId> {
    let n = cfg.blocks.len();
    let mut seen = vec![false; n];
    let mut post: Vec<BlockId> = Vec::new();
    // Iterative postorder DFS (avoids recursion depth limits on large bodies).
    let mut stack: Vec<(BlockId, usize)> = vec![(cfg.entry, 0)];
    seen[cfg.entry] = true;
    while let Some(&mut (b, ref mut i)) = stack.last_mut() {
        let succs = successors(&cfg.blocks[b].term);
        if *i < succs.len() {
            let s = succs[*i];
            *i += 1;
            if s < n && !seen[s] {
                seen[s] = true;
                stack.push((s, 0));
            }
        } else {
            post.push(b);
            stack.pop();
        }
    }
    post.reverse();
    post.into_iter().filter(|b| reach.contains(b)).collect()
}

fn reachable(cfg: &Cfg) -> BTreeSet<BlockId> {
    let mut seen = BTreeSet::new();
    let mut q = VecDeque::new();
    q.push_back(cfg.entry);
    seen.insert(cfg.entry);
    while let Some(b) = q.pop_front() {
        for s in successors(&cfg.blocks[b].term) {
            if seen.insert(s) {
                q.push_back(s);
            }
        }
    }
    seen
}

/// Exposed for Stage 3: the ordered move points of a function (place + span).
pub fn move_points(cfg: &Cfg) -> Vec<(Span, String)> {
    let mut m = Vec::new();
    for blk in &cfg.blocks {
        for a in &blk.actions {
            if matches!(a.access, Access::Move { .. }) {
                m.push((a.span, a.place.display()));
            }
        }
    }
    m
}
