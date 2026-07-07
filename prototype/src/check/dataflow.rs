//! The intra-procedural control-flow graph and the place-state lattice shared
//! by the value-gear dataflow analyses (design 0001 §2.3, §7.4).
//!
//! Stage 2 runs *forward* definite-assignment + move checking (`init.rs`) over
//! this CFG. It is built once (during type checking) and carries, per program
//! point, the classified place access — exactly the "move points and access
//! classifications" Stage 3's backward loan-liveness will consume on the very
//! same graph.

use std::collections::BTreeMap;

use crate::span::Span;

pub type BlockId = usize;

/// A place: a local root plus a projection path (design 0001 §1.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Place {
    pub root: String,
    pub proj: Vec<Proj>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Proj {
    Field(String),
    Deref,
    Index,
}

impl Place {
    pub fn local(name: impl Into<String>) -> Place {
        Place {
            root: name.into(),
            proj: Vec::new(),
        }
    }
    /// Field-only path (no `deref`/index): finely move-trackable.
    pub fn is_direct(&self) -> bool {
        self.proj
            .iter()
            .all(|p| matches!(p, Proj::Field(_)))
    }
    pub fn field_path(&self) -> Vec<String> {
        self.proj
            .iter()
            .filter_map(|p| match p {
                Proj::Field(f) => Some(f.clone()),
                _ => None,
            })
            .collect()
    }
    pub fn display(&self) -> String {
        let mut s = self.root.clone();
        for p in &self.proj {
            match p {
                Proj::Field(f) => s.push_str(&format!(".{f}")),
                Proj::Deref => s = format!("(deref {s})"),
                Proj::Index => s.push_str("[..]"),
            }
        }
        s
    }

    /// The conflict-granularity place (design 0001 §2.2): a place reached through
    /// a `deref` collapses to its root binding (a reborrow anchors on the parent;
    /// a borrow through a `Box` anchors on the box binding); any index covers the
    /// whole array, so the projection is truncated at the first `Index`; distinct
    /// fields stay distinct.
    pub fn canonical(&self) -> Place {
        if self.proj.iter().any(|p| matches!(p, Proj::Deref)) {
            return Place::local(self.root.clone());
        }
        let mut proj = Vec::new();
        for p in &self.proj {
            match p {
                Proj::Field(f) => proj.push(Proj::Field(f.clone())),
                Proj::Index => break,
                Proj::Deref => unreachable!(),
            }
        }
        Place {
            root: self.root.clone(),
            proj,
        }
    }
}

/// Do two *canonical* places overlap (design 0001 §2.2)? Same root and one
/// field-path is a prefix of the other (`p` overlaps `p.f`; `p.f` and `p.g` do
/// not). Canonicalization has already collapsed derefs and indices.
pub fn overlaps(a: &Place, b: &Place) -> bool {
    if a.root != b.root {
        return false;
    }
    let n = a.proj.len().min(b.proj.len());
    a.proj[..n] == b.proj[..n]
}

/// Shared vs. exclusive borrow discriminator carried on a `Borrow` action and
/// on every loan (design 0001 §2.1/§2.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoanKind {
    Shared,
    Excl,
}

/// How a program point touches a place (design 0001 §2.2 access classification).
#[derive(Clone, Debug)]
pub enum Access {
    /// Read the value (definite-assignment: must be initialized).
    Read,
    /// Consume/move the value out (non-copy): sets it moved-from.
    Move {
        movable: bool,
        drop_hooked_partial: bool,
    },
    /// Store into the place (definite assignment gen). `needs_drop` is true when
    /// the place's type runs drop work, so a *reassignment* (whole binding in a
    /// `MaybeInit` state) is a path-dependent drop point — rejected E0309 (§1.6
    /// rule 3 extended to the reassignment drop point; finding 2 of 2026-07-07).
    /// `box_paths` are the field-paths (relative to the place) reaching a `Box`,
    /// so the old value's drop frees — allocator work (finding 4).
    Assign {
        needs_drop: bool,
        box_paths: Vec<Vec<String>>,
    },
    /// Take a borrow (shared/exclusive) — Stage 3 loan point.
    Borrow(LoanKind),
    /// Caller-side `out` argument: the place is initialized after the call, and
    /// the caller's old value is dropped at the call site — the same drop point as
    /// a reassignment (finding 2). Same fields as `Assign`.
    OutArg {
        needs_drop: bool,
        box_paths: Vec<Vec<String>>,
    },
    /// Introduce an uninitialized local (`let x;`).
    Decl,
    /// A place leaves scope here (a drop point). Emitted only for *needs-drop*
    /// places (§1.5): the dual of §1.6's move-join rule requires the place's
    /// initialization state to be path-independent at this point, else the
    /// interpreter would decide the drop from a runtime flag (finding
    /// 2026-07-07). A no-op for dataflow state; a reporting-only check.
    /// `box_paths` are the field-paths (relative to the place) that reach a `Box`;
    /// a drop of any still live here frees, making the enclosing function
    /// allocator-effecting (finding 4 of 2026-07-07).
    ScopeExit {
        box_paths: Vec<Vec<String>>,
    },
}

#[derive(Clone, Debug)]
pub struct Action {
    pub place: Place,
    pub access: Access,
    pub span: Span,
    /// True when the root is a tracked local (param/let/binding).
    pub tracked: bool,
    /// True when this access originates from a contract clause (`ensures`)
    /// re-emitted at a return point (review #3, 2026-07-07): a diagnostic on it
    /// notes the contract position so a use-of-moved read reads as coming from
    /// the clause, not the body.
    pub contract: bool,
}

#[derive(Clone, Debug)]
pub enum Term {
    Goto(BlockId),
    Branch(BlockId, BlockId),
    Switch(Vec<BlockId>),
    /// Normal return of an explicit `return e;` (or `return;`).
    Return,
    /// The body ran off its end without an explicit `return` — an implicit
    /// unit return. For a non-unit function this is the all-paths-return error
    /// (§7.4/NN#5); Stage 3 flags it.
    FallThrough,
    /// Panic / no successor.
    Diverge,
}

#[derive(Clone, Debug)]
pub struct CfgBlock {
    pub actions: Vec<Action>,
    pub term: Term,
    /// Span used to report a move-state disagreement at this join (§1.6).
    pub join_span: Span,
}

#[derive(Clone, Debug)]
pub struct Cfg {
    pub blocks: Vec<CfgBlock>,
    pub entry: BlockId,
    pub preds: Vec<Vec<BlockId>>,
}

impl Cfg {
    pub fn compute_preds(&mut self) {
        let mut preds = vec![Vec::new(); self.blocks.len()];
        for (b, blk) in self.blocks.iter().enumerate() {
            for s in successors(&blk.term) {
                preds[s].push(b);
            }
        }
        self.preds = preds;
    }
}

pub fn successors(t: &Term) -> Vec<BlockId> {
    match t {
        Term::Goto(x) => vec![*x],
        Term::Branch(a, b) => vec![*a, *b],
        Term::Switch(v) => v.clone(),
        Term::Return | Term::FallThrough | Term::Diverge => vec![],
    }
}

// ---------------------------------------------------------------------------
// Place-state lattice (definite assignment × move state, design §1.6/§7.4)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum St {
    Init,
    Uninit,
    Moved,
    /// Initialized on some incoming paths, uninitialized on others (never moved).
    MaybeInit,
}

/// The per-local place tree: either a leaf state or a partial-move field split.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tree {
    Leaf(St),
    /// Only *touched* fields are recorded; untouched fields inherit the sibling
    /// baseline (`base`).
    Fields { base: St, kids: BTreeMap<String, Tree> },
}

impl Tree {
    pub fn leaf(st: St) -> Tree {
        Tree::Leaf(st)
    }

    /// Aggregate read-state of the whole subtree: any moved field dominates
    /// (partial move blocks a whole-value read).
    pub fn agg(&self) -> St {
        match self {
            Tree::Leaf(s) => *s,
            Tree::Fields { base, kids } => {
                let mut acc = *base;
                for k in kids.values() {
                    acc = read_combine(acc, k.agg());
                }
                acc
            }
        }
    }

    fn navigate(&self, path: &[String]) -> St {
        if path.is_empty() {
            return self.agg();
        }
        match self {
            Tree::Leaf(s) => *s,
            Tree::Fields { base, kids } => match kids.get(&path[0]) {
                Some(k) => k.navigate(&path[1..]),
                None => *base,
            },
        }
    }

    fn set(&mut self, path: &[String], st: St) {
        if path.is_empty() {
            *self = Tree::Leaf(st);
            return;
        }
        let base = match self {
            Tree::Leaf(s) => *s,
            Tree::Fields { base, .. } => *base,
        };
        if let Tree::Leaf(_) = self {
            *self = Tree::Fields {
                base,
                kids: BTreeMap::new(),
            };
        }
        if let Tree::Fields { kids, base } = self {
            let child = kids.entry(path[0].clone()).or_insert(Tree::Leaf(*base));
            child.set(&path[1..], st);
        }
    }
}

/// Combine states of distinct fields for a *whole-value read* (all must be live).
fn read_combine(a: St, b: St) -> St {
    use St::*;
    match (a, b) {
        (Moved, _) | (_, Moved) => Moved,
        (Uninit, _) | (_, Uninit) => Uninit,
        (MaybeInit, _) | (_, MaybeInit) => MaybeInit,
        (Init, Init) => Init,
    }
}

/// The flow state: one place tree per tracked local.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowState {
    pub locals: BTreeMap<String, Tree>,
}

impl FlowState {
    pub fn new() -> FlowState {
        FlowState {
            locals: BTreeMap::new(),
        }
    }

    pub fn get(&self, place: &Place) -> St {
        match self.locals.get(&place.root) {
            Some(t) => t.navigate(&place.field_path()),
            None => St::Uninit,
        }
    }

    pub fn set(&mut self, place: &Place, st: St) {
        let t = self
            .locals
            .entry(place.root.clone())
            .or_insert(Tree::Leaf(St::Uninit));
        t.set(&place.field_path(), st);
    }

    /// Join `other` into `self`. Records any place whose move-state disagreed
    /// across the merge (design 0001 §1.6 rule 1) into `disagree`.
    pub fn join(&mut self, other: &FlowState, disagree: &mut Vec<String>) {
        let keys: Vec<String> = self
            .locals
            .keys()
            .chain(other.locals.keys())
            .cloned()
            .collect();
        for k in keys {
            let a = self
                .locals
                .get(&k)
                .cloned()
                .unwrap_or(Tree::Leaf(St::Uninit));
            let b = other
                .locals
                .get(&k)
                .cloned()
                .unwrap_or(Tree::Leaf(St::Uninit));
            let mut d = false;
            let joined = join_tree(&a, &b, &mut d);
            if d {
                disagree.push(k.clone());
            }
            self.locals.insert(k, joined);
        }
    }
}

impl Default for FlowState {
    fn default() -> Self {
        Self::new()
    }
}

fn join_tree(a: &Tree, b: &Tree, disagree: &mut bool) -> Tree {
    match (a, b) {
        (Tree::Leaf(x), Tree::Leaf(y)) => Tree::Leaf(join_st(*x, *y, disagree)),
        _ => {
            let (abase, akids) = as_fields(a);
            let (bbase, bkids) = as_fields(b);
            let mut kids = BTreeMap::new();
            let all: Vec<String> = akids.keys().chain(bkids.keys()).cloned().collect();
            for k in all {
                let av = akids.get(&k).cloned().unwrap_or(Tree::Leaf(abase));
                let bv = bkids.get(&k).cloned().unwrap_or(Tree::Leaf(bbase));
                kids.insert(k, join_tree(&av, &bv, disagree));
            }
            Tree::Fields {
                base: join_st(abase, bbase, disagree),
                kids,
            }
        }
    }
}

fn as_fields(t: &Tree) -> (St, BTreeMap<String, Tree>) {
    match t {
        Tree::Leaf(s) => (*s, BTreeMap::new()),
        Tree::Fields { base, kids } => (*base, kids.clone()),
    }
}

/// Join two leaf states. Sets `disagree` when a live value meets a moved-out
/// one — the §1.6 move-agreement violation.
pub fn join_st(a: St, b: St, disagree: &mut bool) -> St {
    use St::*;
    if a == b {
        return a;
    }
    match (a, b) {
        (Init, Moved) | (Moved, Init) | (Moved, MaybeInit) | (MaybeInit, Moved) => {
            *disagree = true;
            Moved
        }
        (Moved, Uninit) | (Uninit, Moved) => Moved,
        (Init, Uninit)
        | (Uninit, Init)
        | (Init, MaybeInit)
        | (MaybeInit, Init)
        | (Uninit, MaybeInit)
        | (MaybeInit, Uninit) => MaybeInit,
        _ => MaybeInit,
    }
}
