//! Arena-based IR with a constant-folding / algebraic-simplification pass.
//!
//! Implements the frozen `spec-arena.md`:
//! - IR nodes are allocated from an [`Arena`] (a region allocator released as a
//!   whole via [`Arena::reset`], never node-by-node).
//! - Operands are [`NodeId`]s (typed indices) into the *same* arena, forming an
//!   expression tree/DAG.
//! - [`fold`] performs a single post-order transforming pass, producing new IR
//!   in a *fresh* `dst` arena while leaving `src` unmodified.

/// A stable handle to a node within a single [`Arena`].
///
/// IDs are valid until [`Arena::reset`] is called on their arena.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct NodeId(usize);

impl NodeId {
    /// The underlying arena index of this handle.
    #[must_use]
    pub fn index(self) -> usize {
        self.0
    }
}

/// An IR node. Operands are [`NodeId`]s into the same arena (spec §2.6).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Node {
    Const(i64),
    Var(u32),
    Add(NodeId, NodeId),
    Sub(NodeId, NodeId),
    Mul(NodeId, NodeId),
    Div(NodeId, NodeId),
    Neg(NodeId),
}

/// A region allocator for [`Node`]s (spec §2.1-§2.4).
///
/// Reclamation is whole-arena only: [`reset`](Arena::reset) invalidates every
/// previously returned [`NodeId`] at once and makes the arena reusable. There is
/// no per-node freeing.
#[derive(Default)]
pub struct Arena {
    nodes: Vec<Node>,
}

impl Arena {
    /// Create an empty arena (spec §2.1).
    #[must_use]
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Append `node` and return a fresh [`NodeId`] (spec §2.2).
    ///
    /// Operands, if any, must be live ids of this same arena.
    pub fn alloc(&mut self, node: Node) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }

    /// The node for `id` (spec §2.3). `id` must be a live id of this arena.
    #[must_use]
    pub fn get(&self, id: NodeId) -> Node {
        self.nodes[id.0]
    }

    /// Release all nodes at once (spec §2.4).
    ///
    /// Every previously returned [`NodeId`] of this arena becomes invalid; the
    /// arena is reusable for new allocations afterwards.
    pub fn reset(&mut self) {
        self.nodes.clear();
    }

    /// Number of live nodes currently in the arena.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the arena holds no live nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

/// The transforming pass (spec §2.5).
///
/// Allocates all result nodes into a freshly created `dst` arena and returns the
/// root of the transformed IR within it. `src` is left unmodified and `dst` is
/// fully independent of `src` (spec §3.4).
#[must_use]
pub fn fold(src: &Arena, root: NodeId) -> (Arena, NodeId) {
    let mut dst = Arena::new();
    let new_root = fold_node(src, root, &mut dst);
    (dst, new_root)
}

/// Post-order copy-and-simplify of `id` from `src` into `dst` (spec §3.2).
fn fold_node(src: &Arena, id: NodeId, dst: &mut Arena) -> NodeId {
    match src.get(id) {
        Node::Const(_) | Node::Var(_) => dst.alloc(src.get(id)),
        Node::Add(a, b) => {
            let a = fold_node(src, a, dst);
            let b = fold_node(src, b, dst);
            simplify_add(dst, a, b)
        }
        Node::Sub(a, b) => {
            let a = fold_node(src, a, dst);
            let b = fold_node(src, b, dst);
            simplify_sub(dst, a, b)
        }
        Node::Mul(a, b) => {
            let a = fold_node(src, a, dst);
            let b = fold_node(src, b, dst);
            simplify_mul(dst, a, b)
        }
        Node::Div(a, b) => {
            let a = fold_node(src, a, dst);
            let b = fold_node(src, b, dst);
            simplify_div(dst, a, b)
        }
        Node::Neg(a) => {
            let a = fold_node(src, a, dst);
            simplify_neg(dst, a)
        }
    }
}

// Simplification rules (spec §3.2). Operands `a`/`b` are already-simplified ids
// in `dst`. Within each node kind the rules run in the spec's order and the
// first applicable rule wins.

fn simplify_add(dst: &mut Arena, a: NodeId, b: NodeId) -> NodeId {
    // (a) constant folding
    if let (Node::Const(x), Node::Const(y)) = (dst.get(a), dst.get(b)) {
        return dst.alloc(Node::Const(x.wrapping_add(y)));
    }
    // (b) identities: Add(x, 0) -> x ; Add(0, x) -> x
    if dst.get(b) == Node::Const(0) {
        return a;
    }
    if dst.get(a) == Node::Const(0) {
        return b;
    }
    // (c) keep
    dst.alloc(Node::Add(a, b))
}

fn simplify_sub(dst: &mut Arena, a: NodeId, b: NodeId) -> NodeId {
    // (a) constant folding
    if let (Node::Const(x), Node::Const(y)) = (dst.get(a), dst.get(b)) {
        return dst.alloc(Node::Const(x.wrapping_sub(y)));
    }
    // (b) identities: Sub(x, 0) -> x ; Sub(0, x) -> Neg(x)
    if dst.get(b) == Node::Const(0) {
        return a;
    }
    if dst.get(a) == Node::Const(0) {
        return dst.alloc(Node::Neg(b));
    }
    // (c) keep
    dst.alloc(Node::Sub(a, b))
}

fn simplify_mul(dst: &mut Arena, a: NodeId, b: NodeId) -> NodeId {
    // (a) constant folding
    if let (Node::Const(x), Node::Const(y)) = (dst.get(a), dst.get(b)) {
        return dst.alloc(Node::Const(x.wrapping_mul(y)));
    }
    // (b) identities: Mul(x, 1) -> x ; Mul(1, x) -> x ; Mul(x, 0) -> 0 ; Mul(0, x) -> 0
    if dst.get(b) == Node::Const(1) {
        return a;
    }
    if dst.get(a) == Node::Const(1) {
        return b;
    }
    if dst.get(b) == Node::Const(0) || dst.get(a) == Node::Const(0) {
        return dst.alloc(Node::Const(0));
    }
    // (c) keep
    dst.alloc(Node::Mul(a, b))
}

fn simplify_div(dst: &mut Arena, a: NodeId, b: NodeId) -> NodeId {
    // (a) constant folding: only when the divisor is a non-zero constant.
    // Div by a constant zero is intentionally left unfolded and falls through.
    if let (Node::Const(x), Node::Const(y)) = (dst.get(a), dst.get(b)) {
        if y != 0 {
            // i64 division truncates toward zero; wrapping_div handles the sole
            // overflow case i64::MIN / -1 -> i64::MIN (spec §3.1).
            return dst.alloc(Node::Const(x.wrapping_div(y)));
        }
    }
    // (b) identity: Div(x, 1) -> x. Div(x, 0) is left unchanged.
    if dst.get(b) == Node::Const(1) {
        return a;
    }
    // (c) keep
    dst.alloc(Node::Div(a, b))
}

fn simplify_neg(dst: &mut Arena, a: NodeId) -> NodeId {
    match dst.get(a) {
        // (a) constant folding: Neg(c) -> Const(0 wrapping_sub c)
        Node::Const(c) => dst.alloc(Node::Const(c.wrapping_neg())),
        // (b) identity: Neg(Neg(x)) -> x
        Node::Neg(inner) => inner,
        // (c) keep
        _ => dst.alloc(Node::Neg(a)),
    }
}

/// Canonical serialization `S(node)` fully unfolded from `id` (spec §3.3).
#[must_use]
pub fn serialize(arena: &Arena, id: NodeId) -> String {
    match arena.get(id) {
        Node::Const(c) => format!("(c {c})"),
        Node::Var(v) => format!("(v {v})"),
        Node::Add(a, b) => format!("(+ {} {})", serialize(arena, a), serialize(arena, b)),
        Node::Sub(a, b) => format!("(- {} {})", serialize(arena, a), serialize(arena, b)),
        Node::Mul(a, b) => format!("(* {} {})", serialize(arena, a), serialize(arena, b)),
        Node::Div(a, b) => format!("(/ {} {})", serialize(arena, a), serialize(arena, b)),
        Node::Neg(a) => format!("(neg {})", serialize(arena, a)),
    }
}

/// Evaluate the IR rooted at `id` under the variable assignment `vars`
/// (spec §3.5). Uses the wrapping arithmetic of §3.1. Callers must stay within
/// the equivalence domain (no division by zero).
#[must_use]
pub fn eval(arena: &Arena, id: NodeId, vars: &dyn Fn(u32) -> i64) -> i64 {
    match arena.get(id) {
        Node::Const(c) => c,
        Node::Var(v) => vars(v),
        Node::Add(a, b) => eval(arena, a, vars).wrapping_add(eval(arena, b, vars)),
        Node::Sub(a, b) => eval(arena, a, vars).wrapping_sub(eval(arena, b, vars)),
        Node::Mul(a, b) => eval(arena, a, vars).wrapping_mul(eval(arena, b, vars)),
        Node::Div(a, b) => eval(arena, a, vars).wrapping_div(eval(arena, b, vars)),
        Node::Neg(a) => eval(arena, a, vars).wrapping_neg(),
    }
}
