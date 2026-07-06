//! Structured diagnostics (P4: machine-readable, provenance-carrying).
//!
//! A `Diag` is the single error currency of the prototype's front end. It is
//! serializable to JSON so the toolchain treats diagnostics as a first-class
//! output, not just human prose.

use serde::Serialize;

use crate::span::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

/// A secondary note attached to a diagnostic, optionally pointing at a span.
#[derive(Clone, Debug, Serialize)]
pub struct Note {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
}

/// A single diagnostic: what failed, where, and why.
#[derive(Clone, Debug, Serialize)]
pub struct Diag {
    pub severity: Severity,
    /// Stable machine code, e.g. `"L0001"` (lexer) or `"P0007"` (parser).
    pub code: String,
    pub message: String,
    pub span: Span,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<Note>,
}

impl Diag {
    pub fn error(code: &str, message: impl Into<String>, span: Span) -> Diag {
        Diag {
            severity: Severity::Error,
            code: code.to_string(),
            message: message.into(),
            span,
            notes: Vec::new(),
        }
    }

    pub fn with_note(mut self, message: impl Into<String>, span: Option<Span>) -> Diag {
        self.notes.push(Note {
            message: message.into(),
            span,
        });
        self
    }

    /// Render as a single-line JSON object.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Diag is always serializable")
    }
}
