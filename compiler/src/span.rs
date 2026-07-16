//! Byte-offset source spans.
//!
//! Every token and every AST node carries a `Span` so that later stages
//! (checker, interpreter) and the diagnostics layer (P4) can report precise,
//! machine-readable provenance.

use serde::Serialize;

/// A half-open byte range `[start, end)` into the source text.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Span {
        Span { start, end }
    }

    /// A zero-width span at `at` (used for synthetic/EOF positions).
    pub fn point(at: usize) -> Span {
        Span { start: at, end: at }
    }

    /// Smallest span covering both `self` and `other`.
    pub fn to(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}
