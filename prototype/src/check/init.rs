//! Definite assignment + move checking as one integrated place-state analysis
//! (design 0001 §1.6, §7.4). A forward dataflow over the shared CFG.
//!
//! The asymmetry the spec demands (§1.6 vs §7.4) is realized in the lattice:
//! definite assignment only requires all-paths-initialized *before a read*
//! (`MaybeInit` is legal until read), whereas move state must *agree* at every
//! join — a place live on one path and moved on another is an error even if
//! never read again (`join_st` flags it).

use std::collections::BTreeSet;
use std::collections::VecDeque;

use crate::check::dataflow::*;
use crate::diag::Diag;
use crate::span::Span;

/// Run the analysis, pushing diagnostics.
pub fn analyze(cfg: &Cfg, entry: &FlowState, out_params: &[String], diags: &mut Vec<Diag>) {
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
    let mut changed = true;
    let mut guard = 0;
    while changed && guard < n * 4 + 16 {
        changed = false;
        guard += 1;
        for &b in &order {
            let in_b = incoming(cfg, b, entry, &out_state, &reach, &visited, None);
            let mut st = in_b;
            for a in &cfg.blocks[b].actions {
                apply(&mut st, a, out_params, None);
            }
            visited[b] = true;
            if st != out_state[b] {
                out_state[b] = st;
                changed = true;
            }
        }
    }

    // --- Reporting pass ----------------------------------------------------
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
            apply(&mut st, a, out_params, Some(diags));
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
        }
    }
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

fn apply(st: &mut FlowState, a: &Action, out_params: &[String], report: Option<&mut Vec<Diag>>) {
    let opaque = !a.place.is_direct();
    let root = Place::local(a.place.root.clone());
    let key = if opaque { &root } else { &a.place };
    match &a.access {
        Access::Read | Access::Borrow(_) => {
            if let Some(d) = report {
                require_init(st, key, out_params, a.span, d);
            }
        }
        Access::Move {
            movable: _,
            drop_hooked_partial,
        } => {
            if opaque {
                if let Some(d) = report {
                    require_init(st, &root, out_params, a.span, d);
                }
            } else {
                if let Some(d) = report {
                    require_init(st, key, out_params, a.span, d);
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
        Access::Assign => {
            if opaque {
                if let Some(d) = report {
                    require_init(st, &root, out_params, a.span, d);
                }
            } else {
                st.set(&a.place, St::Init);
            }
        }
        Access::OutArg => {
            if opaque {
                if let Some(d) = report {
                    require_init(st, &root, out_params, a.span, d);
                }
            } else {
                st.set(&a.place, St::Init);
            }
        }
        Access::Decl => {
            st.set(&a.place, St::Uninit);
        }
    }
}

fn require_init(st: &FlowState, place: &Place, out_params: &[String], span: Span, diags: &mut Vec<Diag>) {
    let s = st.get(place);
    if s == St::Init {
        return;
    }
    let is_out = out_params.contains(&place.root) && place.is_direct() && place.proj.is_empty();
    match s {
        St::Moved => diags.push(
            Diag::error(
                "E0301",
                format!("use of moved value `{}`", place.display()),
                span,
            )
            .with_note("value was moved out on an earlier path (§1.2)", None),
        ),
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
