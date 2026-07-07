//! Stage 3 borrow checker (design 0001 §2.2, §2.3, §3.1, §7.4): NLL-lite loan
//! liveness plus the XOR / move / write conflict scan, same-call overlap (no
//! two-phase borrows), and the all-paths-return soundness check.
//!
//! Loans are created by Stage 2's checker at every borrow (`read`/`write`),
//! reborrow, slice op, call-argument, `out` slot, and borrowed-scrutinee pattern
//! binding. A loan anchored to a binding is *in scope over that binding's live
//! range*; a transient (call-argument) loan lives only at its creation and has
//! its same-call overlaps checked directly. The scan reports both loans, their
//! creation sites, and the conflict point (P4: full provenance).

use std::collections::{BTreeSet, HashSet};
use std::collections::VecDeque;

use crate::check::dataflow::*;
use crate::diag::Diag;
use crate::span::Span;

/// Where a loan lives (design §2.3).
#[derive(Clone, Debug)]
pub enum Anchor {
    /// Carried by a named borrow value: in scope over that binding's live range.
    Binding(String),
    /// A transient (call-argument / return-extended-but-unbound) loan: not part
    /// of the liveness scan; its same-call overlaps are checked directly.
    Temp,
}

#[derive(Clone, Debug)]
pub struct LoanInfo {
    /// Conflict-granularity place the loan restricts (already canonical).
    pub place: Place,
    pub kind: LoanKind,
    pub span: Span,
    pub anchor: Anchor,
}

pub struct LoanTables {
    pub loans: Vec<LoanInfo>,
    pub call_groups: Vec<Vec<usize>>,
}

pub fn analyze(
    cfg: &Cfg,
    t: &LoanTables,
    ret_non_unit: bool,
    fn_span: Span,
    diags: &mut Vec<Diag>,
) {
    all_paths_return(cfg, ret_non_unit, fn_span, diags);
    let live = Liveness::compute(cfg);
    conflict_scan(cfg, t, &live, diags);
    same_call_overlaps(t, diags);
}

// ---------------------------------------------------------------------------
// all-paths-return (design §7.4 / NN#5)
// ---------------------------------------------------------------------------

fn all_paths_return(cfg: &Cfg, ret_non_unit: bool, fn_span: Span, diags: &mut Vec<Diag>) {
    if !ret_non_unit {
        return;
    }
    let reach = reachable(cfg);
    for b in &reach {
        if matches!(cfg.blocks[*b].term, Term::FallThrough) {
            diags.push(
                Diag::error(
                    "E0810",
                    "control may reach the end of this non-unit function without returning a value",
                    fn_span,
                )
                .with_note("every path out of a non-unit function must `return` a value (§7.4)", None),
            );
            return;
        }
    }
}

// ---------------------------------------------------------------------------
// Backward liveness of borrow-value bindings (design §2.3, step 2)
// ---------------------------------------------------------------------------

struct Liveness {
    /// Roots live *after* each action executes (= before the next point).
    after: Vec<Vec<HashSet<String>>>,
    reach: BTreeSet<BlockId>,
}

impl Liveness {
    fn compute(cfg: &Cfg) -> Liveness {
        let n = cfg.blocks.len();
        let reach = reachable(cfg);
        let mut live_in: Vec<HashSet<String>> = vec![HashSet::new(); n];
        let mut after: Vec<Vec<HashSet<String>>> = (0..n)
            .map(|b| vec![HashSet::new(); cfg.blocks[b].actions.len()])
            .collect();

        let mut changed = true;
        let mut guard = 0;
        while changed && guard < n * 4 + 16 {
            changed = false;
            guard += 1;
            for b in reach.iter().rev() {
                let b = *b;
                let mut cur: HashSet<String> = HashSet::new();
                for s in successors(&cfg.blocks[b].term) {
                    for r in &live_in[s] {
                        cur.insert(r.clone());
                    }
                }
                let acts = &cfg.blocks[b].actions;
                let mut afterb = vec![HashSet::new(); acts.len()];
                for i in (0..acts.len()).rev() {
                    afterb[i] = cur.clone();
                    transfer(&mut cur, &acts[i]);
                }
                after[b] = afterb;
                if cur != live_in[b] {
                    live_in[b] = cur;
                    changed = true;
                }
            }
        }
        Liveness { after, reach }
    }

    fn live_after(&self, b: BlockId, i: usize) -> &HashSet<String> {
        &self.after[b][i]
    }
}

/// live_before = (live_after \ def) ∪ use.
fn transfer(set: &mut HashSet<String>, a: &Action) {
    let root = a.place.root.clone();
    match &a.access {
        Access::Decl => {
            set.remove(&root);
        }
        Access::Assign { .. } => {
            if a.place.proj.is_empty() {
                set.remove(&root); // whole-binding rebind: a definition
            } else {
                set.insert(root); // write *through* a borrow: a use of it
            }
        }
        Access::Read | Access::Borrow(_) | Access::OutArg { .. } | Access::Move { .. } => {
            set.insert(root);
        }
        Access::ScopeExit { .. } => {} // a drop point is neither a use nor a def of a loan
    }
}

// ---------------------------------------------------------------------------
// The conflict scan (design §2.2 / §2.3 step 4)
// ---------------------------------------------------------------------------

/// The access an action imposes on its place for the XOR check.
enum AccessKind {
    /// A direct read (shared): conflicts only with a live *exclusive* loan.
    Read,
    /// A move out (exclusive): conflicts with any live loan.
    Move,
    /// A write / reassignment / `out` init (exclusive): conflicts with any loan.
    Write,
    /// Creating a borrow of the given kind: shared conflicts with a live
    /// exclusive loan, exclusive with any live loan.
    Borrow(LoanKind),
    None,
}

fn classify(a: &Action) -> AccessKind {
    match &a.access {
        Access::Read => AccessKind::Read,
        Access::Move { .. } => AccessKind::Move,
        Access::Assign { .. } | Access::OutArg { .. } => AccessKind::Write,
        Access::Borrow(k) => AccessKind::Borrow(*k),
        Access::Decl | Access::ScopeExit { .. } => AccessKind::None,
    }
}

fn conflict_scan(cfg: &Cfg, t: &LoanTables, live: &Liveness, diags: &mut Vec<Diag>) {
    let mut seen: HashSet<(String, usize, usize)> = HashSet::new();
    for &b in live.reach.iter() {
        for (i, a) in cfg.blocks[b].actions.iter().enumerate() {
            let kind = classify(a);
            if matches!(kind, AccessKind::None) {
                continue;
            }
            let canon = a.place.canonical();
            let after = live.live_after(b, i);
            for loan in &t.loans {
                let name = match &loan.anchor {
                    Anchor::Binding(n) => n,
                    Anchor::Temp => continue,
                };
                if !after.contains(name) {
                    continue;
                }
                if !overlaps(&canon, &loan.place) {
                    continue;
                }
                if let Some((code, msg)) = judge(&kind, loan.kind, &canon) {
                    let key = (code.to_string(), a.span.start, loan.span.start);
                    if seen.insert(key) {
                        diags.push(
                            Diag::error(code, msg, a.span)
                                .with_note(
                                    format!("...conflicts with the borrow of `{}` created here", loan.place.display()),
                                    Some(loan.span),
                                )
                                .with_note("a place under a live borrow may be accessed only through that borrow (§2.2)", None),
                        );
                    }
                }
            }
        }
    }
}

/// Returns (code, message) if the access conflicts with a live loan of `lk`.
fn judge(access: &AccessKind, lk: LoanKind, place: &Place) -> Option<(&'static str, String)> {
    let p = place.display();
    match access {
        AccessKind::Read => match lk {
            LoanKind::Excl => Some(("E0804", format!("cannot read `{p}` while it is exclusively borrowed"))),
            LoanKind::Shared => None,
        },
        AccessKind::Move => Some(("E0802", format!("cannot move out of `{p}` while it is borrowed"))),
        AccessKind::Write => Some(("E0803", format!("cannot write to `{p}` while it is borrowed"))),
        AccessKind::Borrow(bk) => match (bk, lk) {
            (LoanKind::Shared, LoanKind::Shared) => None,
            _ => Some(("E0801", format!("conflicting borrow of `{p}` (an exclusive borrow excludes all other borrows) (§2.2)"))),
        },
        AccessKind::None => None,
    }
}

// ---------------------------------------------------------------------------
// Same-call overlap: no two-phase borrows (design §2.3 limitation, §3.1)
// ---------------------------------------------------------------------------

fn same_call_overlaps(t: &LoanTables, diags: &mut Vec<Diag>) {
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    for group in &t.call_groups {
        for a in 0..group.len() {
            for b in (a + 1)..group.len() {
                let (li, lj) = (&t.loans[group[a]], &t.loans[group[b]]);
                if !overlaps(&li.place, &lj.place) {
                    continue;
                }
                if matches!((li.kind, lj.kind), (LoanKind::Shared, LoanKind::Shared)) {
                    continue;
                }
                let key = (group[a].min(group[b]), group[a].max(group[b]));
                if seen.insert(key) {
                    diags.push(
                        Diag::error(
                            "E0805",
                            format!(
                                "arguments in this call borrow overlapping places `{}` and `{}` incompatibly",
                                li.place.display(),
                                lj.place.display()
                            ),
                            lj.span,
                        )
                        .with_note(
                            format!("the other borrow of `{}` is here", li.place.display()),
                            Some(li.span),
                        )
                        .with_note("the prototype has no two-phase borrows; split the call (§2.3)", None),
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------

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
