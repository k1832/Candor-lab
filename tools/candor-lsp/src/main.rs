//! Candor diagnostics LSP (layer 2).
//!
//! A deliberately small, hand-rolled stdio JSON-RPC server. It speaks just
//! enough of the Language Server Protocol to publish diagnostics:
//!
//!   initialize / initialized
//!   textDocument/didOpen | didChange | didSave | didClose
//!   textDocument/publishDiagnostics   (server -> client push)
//!   shutdown / exit
//!
//! Scope is honest and narrow (P16): **diagnostics only**. There is no hover,
//! no completion, and no go-to-definition. Those are semantic-server concerns
//! deferred to the real Candor toolchain; this layer exists so an editor can
//! surface the prototype checker's errors inline while that toolchain is built.
//!
//! The check pipeline runs IN-PROCESS by depending on the `candor` library
//! crate (path dependency) — never by shelling out. `.cnr` files use the real
//! surface syntax entry point (`check_source_real`); any other extension is
//! treated as throwaway `.cn` (`check_source`). Byte-offset spans from
//! `diag.rs` are mapped to LSP line/UTF-16-character positions here.
//!
//! Dependency footprint: `serde_json` only (plus the path dep). No tokio, no
//! tower — the request volume of a single-file diagnostics loop does not warrant
//! an async runtime, and a hand-rolled loop keeps the crate auditable.

use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use serde_json::{json, Value};

use candor::diag::{Diag, Severity};
use candor::span::Span;

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    serve(&mut stdin.lock(), &mut stdout.lock());
}

// ---------------------------------------------------------------------------
// Offset -> position mapping
// ---------------------------------------------------------------------------

/// Precomputed byte offsets of the start of each line, for mapping a byte
/// offset (as carried by `diag.rs` spans) to an LSP `(line, character)` where
/// `character` is a **UTF-16** code-unit count (the LSP default encoding).
pub struct LineIndex {
    /// `line_starts[i]` is the byte offset at which line `i` begins.
    line_starts: Vec<usize>,
    /// Total byte length of the source (for clamping).
    len: usize,
}

impl LineIndex {
    pub fn new(src: &str) -> LineIndex {
        let mut line_starts = vec![0usize];
        for (i, b) in src.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        LineIndex {
            line_starts,
            len: src.len(),
        }
    }

    /// Map a byte offset to an LSP `(line, utf16_character)` pair. Offsets past
    /// the end clamp to the end; an offset landing inside a UTF-8 code point is
    /// floored to the enclosing character boundary (spans are boundary-aligned
    /// in practice, but the server must never panic on malformed input).
    pub fn position(&self, src: &str, offset: usize) -> (u32, u32) {
        let mut offset = offset.min(self.len);
        while offset > 0 && !src.is_char_boundary(offset) {
            offset -= 1;
        }
        // Largest line start <= offset.
        let line = match self.line_starts.binary_search(&offset) {
            Ok(l) => l,
            Err(l) => l - 1,
        };
        let line_start = self.line_starts[line];
        let character = src[line_start..offset].encode_utf16().count();
        (line as u32, character as u32)
    }
}

fn range_of(src: &str, idx: &LineIndex, span: Span) -> Value {
    let (sl, sc) = idx.position(src, span.start);
    let (el, ec) = idx.position(src, span.end);
    json!({
        "start": { "line": sl, "character": sc },
        "end":   { "line": el, "character": ec },
    })
}

// ---------------------------------------------------------------------------
// Diagnostics: run the in-process check pipeline and map to LSP shape
// ---------------------------------------------------------------------------

fn severity_code(sev: Severity) -> u8 {
    match sev {
        Severity::Error => 1,   // LSP DiagnosticSeverity.Error
        Severity::Warning => 2, // LSP DiagnosticSeverity.Warning
    }
}

/// Map one `candor` `Diag` to an LSP `Diagnostic` JSON object. Notes that
/// carry a span become `relatedInformation` (pointing back into the same
/// document); span-less notes are appended to the message.
fn diag_to_lsp(uri: &str, src: &str, idx: &LineIndex, d: &Diag) -> Value {
    let mut related = Vec::new();
    let mut trailing = String::new();
    for note in &d.notes {
        match note.span {
            Some(sp) => related.push(json!({
                "location": { "uri": uri, "range": range_of(src, idx, sp) },
                "message": note.message,
            })),
            None => {
                trailing.push('\n');
                trailing.push_str(&note.message);
            }
        }
    }
    let message = format!("{}{}", d.message, trailing);
    json!({
        "range": range_of(src, idx, d.span),
        "severity": severity_code(d.severity),
        "code": d.code,
        "source": "candor",
        "message": message,
        "relatedInformation": related,
    })
}

/// Run the check pipeline for a document and return the LSP diagnostics array.
///
/// `.cnr` uses the real surface syntax; anything else is throwaway `.cn`. The
/// checker is run under `catch_unwind` so a panic in an unfinished prototype
/// path degrades to a single internal-error diagnostic rather than killing the
/// server loop (a server must survive a bad document).
pub fn diagnostics(uri: &str, src: &str) -> Vec<Value> {
    let idx = LineIndex::new(src);
    let is_real = uri.ends_with(".cnr");
    let owned = src.to_string();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if is_real {
            candor::check_source_real(&owned)
        } else {
            candor::check_source(&owned)
        }
    }));
    let diags: Vec<Diag> = match outcome {
        Ok(Ok(ds)) => ds,       // parsed; zero or more check diagnostics
        Ok(Err(d)) => vec![d],  // a hard parse error is a single diagnostic
        Err(_) => {
            return vec![json!({
                "range": range_of(src, &idx, Span::point(0)),
                "severity": 1,
                "code": "LSP0001",
                "source": "candor-lsp",
                "message": "internal error: the checker panicked on this document",
            })];
        }
    };
    diags.iter().map(|d| diag_to_lsp(uri, src, &idx, d)).collect()
}

// ---------------------------------------------------------------------------
// JSON-RPC framing
// ---------------------------------------------------------------------------

/// Read one `Content-Length`-framed JSON-RPC message. Returns `None` at EOF or
/// on a malformed frame (which ends the loop).
pub fn read_message<R: BufRead>(r: &mut R) -> Option<Value> {
    let mut content_len: Option<usize> = None;
    loop {
        let mut line = String::new();
        let n = r.read_line(&mut line).ok()?;
        if n == 0 {
            return None; // EOF
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break; // end of headers
        }
        if let Some(v) = trimmed.strip_prefix("Content-Length:") {
            content_len = v.trim().parse().ok();
        }
    }
    let len = content_len?;
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf).ok()?;
    serde_json::from_slice(&buf).ok()
}

/// Write one `Content-Length`-framed JSON-RPC message.
pub fn write_message<W: Write>(w: &mut W, msg: &Value) {
    let body = serde_json::to_string(msg).expect("serializable");
    let _ = write!(w, "Content-Length: {}\r\n\r\n{}", body.len(), body);
    let _ = w.flush();
}

// ---------------------------------------------------------------------------
// Server state + dispatch
// ---------------------------------------------------------------------------

/// The document store. Full-text sync (LSP `TextDocumentSyncKind.Full`): each
/// `didChange` carries the whole document, which we recheck directly. No
/// debounce timer — a single-file check is cheap and the loop is synchronous.
#[derive(Default)]
pub struct Backend {
    docs: HashMap<String, String>,
}

impl Backend {
    pub fn new() -> Backend {
        Backend::default()
    }

    /// Handle one incoming message. Returns `(outgoing messages, should_exit)`.
    pub fn dispatch(&mut self, msg: &Value) -> (Vec<Value>, bool) {
        let method = msg.get("method").and_then(Value::as_str);
        let id = msg.get("id").cloned();
        match method {
            Some("initialize") => (vec![response(id, initialize_result())], false),
            Some("initialized") => (vec![], false),

            Some("textDocument/didOpen") => {
                let td = params(msg).get("textDocument").cloned().unwrap_or(Value::Null);
                let uri = str_field(&td, "uri");
                let text = str_field(&td, "text");
                self.docs.insert(uri.clone(), text.clone());
                (vec![publish(&uri, &text)], false)
            }

            Some("textDocument/didChange") => {
                let p = params(msg);
                let uri = str_field(&p.get("textDocument").cloned().unwrap_or(Value::Null), "uri");
                // Full sync: the last content change holds the whole document.
                let text = p
                    .get("contentChanges")
                    .and_then(Value::as_array)
                    .and_then(|c| c.last())
                    .map(|c| str_field(c, "text"))
                    .unwrap_or_default();
                self.docs.insert(uri.clone(), text.clone());
                (vec![publish(&uri, &text)], false)
            }

            Some("textDocument/didSave") => {
                let uri = str_field(
                    &params(msg).get("textDocument").cloned().unwrap_or(Value::Null),
                    "uri",
                );
                let text = self.docs.get(&uri).cloned().unwrap_or_default();
                (vec![publish(&uri, &text)], false)
            }

            Some("textDocument/didClose") => {
                let uri = str_field(
                    &params(msg).get("textDocument").cloned().unwrap_or(Value::Null),
                    "uri",
                );
                self.docs.remove(&uri);
                // Clear diagnostics for the closed document.
                (vec![publish_diagnostics(&uri, Vec::new())], false)
            }

            Some("shutdown") => (vec![response(id, Value::Null)], false),
            Some("exit") => (vec![], true),

            // Unknown request (has a non-null id): reply so the client isn't left
            // hanging. Unknown notification: ignore.
            _ => match id {
                Some(ref v) if !v.is_null() => (vec![response(id, Value::Null)], false),
                _ => (vec![], false),
            },
        }
    }
}

fn params(msg: &Value) -> Value {
    msg.get("params").cloned().unwrap_or(Value::Null)
}

fn str_field(v: &Value, key: &str) -> String {
    v.get(key).and_then(Value::as_str).unwrap_or_default().to_string()
}

fn response(id: Option<Value>, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id.unwrap_or(Value::Null), "result": result })
}

fn publish(uri: &str, text: &str) -> Value {
    publish_diagnostics(uri, diagnostics(uri, text))
}

fn publish_diagnostics(uri: &str, diags: Vec<Value>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": { "uri": uri, "diagnostics": diags },
    })
}

fn initialize_result() -> Value {
    json!({
        "capabilities": {
            // Full-document sync; we push diagnostics ourselves.
            "textDocumentSync": 1,
        },
        "serverInfo": { "name": "candor-lsp", "version": "0.1.0" },
    })
}

/// The blocking read/dispatch/write loop, generic over the streams so tests can
/// drive it with in-memory pipes.
pub fn serve<R: BufRead, W: Write>(reader: &mut R, writer: &mut W) {
    let mut backend = Backend::new();
    while let Some(msg) = read_message(reader) {
        let (outgoing, stop) = backend.dispatch(&msg);
        for out in &outgoing {
            write_message(writer, out);
        }
        if stop {
            break;
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_ascii_single_line() {
        let src = "fn main() {}";
        let idx = LineIndex::new(src);
        assert_eq!(idx.position(src, 0), (0, 0));
        assert_eq!(idx.position(src, 3), (0, 3)); // start of `main`
        assert_eq!(idx.position(src, src.len()), (0, src.len() as u32));
    }

    #[test]
    fn position_multi_line() {
        let src = "let a = 1;\nlet b = 2;\n";
        let idx = LineIndex::new(src);
        // offset 11 is the `l` of the second line's `let`.
        assert_eq!(idx.position(src, 11), (1, 0));
        // offset 15 -> second line, char 4 (`b`).
        assert_eq!(idx.position(src, 15), (1, 4));
    }

    #[test]
    fn position_utf16_counts_code_units_not_bytes() {
        // `é` is 2 UTF-8 bytes but 1 UTF-16 unit; the snowman-like `𝄞` (U+1D11E)
        // is 4 UTF-8 bytes and 2 UTF-16 units (a surrogate pair).
        let src = "\"é𝄞\"x";
        let idx = LineIndex::new(src);
        // Byte offset of the final `x`: `"`(1) + `é`(2) + `𝄞`(4) + `"`(1) = 8.
        let x_off = src.find('x').unwrap();
        assert_eq!(x_off, 8);
        // UTF-16 columns: `"`=1, `é`=1, `𝄞`=2, `"`=1 -> x at column 5.
        assert_eq!(idx.position(src, x_off), (0, 5));
    }

    #[test]
    fn position_floors_into_char_boundary_without_panic() {
        let src = "é"; // 2 bytes, boundaries at 0 and 2
        let idx = LineIndex::new(src);
        // Offset 1 lands inside the code point; must floor to 0, not panic.
        assert_eq!(idx.position(src, 1), (0, 0));
        // Past-the-end clamps.
        assert_eq!(idx.position(src, 99), (0, 1));
    }

    #[test]
    fn parse_error_maps_to_one_diagnostic() {
        // Missing expression after `=` is a parse error (single Diag via Err).
        let src = "fn main() -> i64 {\n    let x: i64 = ;\n}\n";
        let ds = diagnostics("file:///t.cnr", src);
        assert_eq!(ds.len(), 1);
        let d = &ds[0];
        assert_eq!(d["severity"], json!(1));
        // The `;` sits on line 1 (0-based); the range must land there.
        assert_eq!(d["range"]["start"]["line"], json!(1));
        assert!(d["message"].as_str().unwrap().to_lowercase().contains("expected"));
    }

    #[test]
    fn clean_program_has_no_diagnostics() {
        let src = "fn main() -> i64 {\n    return 0;\n}\n";
        assert!(diagnostics("file:///ok.cnr", src).is_empty());
    }

    #[test]
    fn end_to_end_server_loop_publishes_a_diagnostic() {
        // Frame a full session against the real read/dispatch/write loop and
        // assert a publishDiagnostics with a non-empty diagnostics array.
        fn frame(v: Value) -> Vec<u8> {
            let body = serde_json::to_string(&v).unwrap();
            format!("Content-Length: {}\r\n\r\n{}", body.len(), body).into_bytes()
        }
        let bad = "fn main() -> i64 {\n    return undefined_name;\n}\n";
        let mut input = Vec::new();
        input.extend(frame(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}})));
        input.extend(frame(json!({"jsonrpc":"2.0","method":"initialized","params":{}})));
        input.extend(frame(json!({
            "jsonrpc":"2.0","method":"textDocument/didOpen",
            "params": {"textDocument": {"uri":"file:///e2e.cnr","languageId":"candor","version":1,"text": bad}}
        })));
        input.extend(frame(json!({"jsonrpc":"2.0","id":2,"method":"shutdown"})));
        input.extend(frame(json!({"jsonrpc":"2.0","method":"exit"})));

        let mut reader = std::io::Cursor::new(input);
        let mut out: Vec<u8> = Vec::new();
        serve(&mut reader, &mut out);

        // Parse the framed output stream back into messages.
        let text = String::from_utf8(out).unwrap();
        let mut msgs = Vec::new();
        let mut rest = text.as_str();
        while let Some(hdr_end) = rest.find("\r\n\r\n") {
            let header = &rest[..hdr_end];
            let len: usize = header
                .lines()
                .find_map(|l| l.strip_prefix("Content-Length:"))
                .and_then(|v| v.trim().parse().ok())
                .unwrap();
            let body_start = hdr_end + 4;
            let body = &rest[body_start..body_start + len];
            msgs.push(serde_json::from_str::<Value>(body).unwrap());
            rest = &rest[body_start + len..];
        }

        let publish = msgs
            .iter()
            .find(|m| m["method"] == json!("textDocument/publishDiagnostics"))
            .expect("a publishDiagnostics notification");
        assert_eq!(publish["params"]["uri"], json!("file:///e2e.cnr"));
        let diags = publish["params"]["diagnostics"].as_array().unwrap();
        assert!(!diags.is_empty(), "expected at least one diagnostic");
        assert_eq!(diags[0]["source"], json!("candor"));
    }
}
