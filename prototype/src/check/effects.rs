//! The one tracked effect: allocation (design 0001 §3.2, §6.3).
//!
//! A single boolean per function. `box`, `clone` of a box-bearing value, a call
//! to an `alloc`-marked function, and an indirect call through an `alloc`-typed
//! fn-pointer each make the enclosing function `alloc`. A non-`alloc` function
//! that performs any of these is a checker error (E0401). Effects are upper
//! bounds: a marked function need not allocate.

use crate::diag::Diag;
use crate::span::Span;

/// Accumulates the first allocation-effecting site seen in a function body,
/// so a non-`alloc` function can be diagnosed with provenance (design P4).
#[derive(Default)]
pub struct AllocEffect {
    pub site: Option<(Span, String)>,
}

impl AllocEffect {
    pub fn note(&mut self, span: Span, reason: impl Into<String>) {
        if self.site.is_none() {
            self.site = Some((span, reason.into()));
        }
    }

    /// After checking a function body, verify the effect partition.
    pub fn finish(&self, fn_marked_alloc: bool, fn_name: &str, fn_span: Span) -> Option<Diag> {
        if fn_marked_alloc {
            return None; // marked: upper bound, always fine
        }
        self.site.as_ref().map(|(span, reason)| {
            Diag::error(
                "E0401",
                format!("non-`alloc` function `{fn_name}` performs allocation"),
                *span,
            )
            .with_note(reason.clone(), Some(*span))
            .with_note(
                "add the `alloc` effect to the signature, or remove the allocation",
                Some(fn_span),
            )
        })
    }
}
