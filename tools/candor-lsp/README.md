# candor-lsp

A minimal, **diagnostics-only** Language Server for Candor (editor-support
layer 2). It runs the prototype checker in-process and pushes diagnostics to the
editor. That is the whole feature set.

## Scope (P16 deferral)

**No hover. No completion. No go-to-definition.** Those are semantic-analysis
features that belong to the real Candor toolchain and are deferred per **P16**.
This server exists only so an editor can surface the `candor` checker's
errors inline while that toolchain is built. It implements exactly:

- `initialize` / `initialized`
- `textDocument/didOpen`, `didChange`, `didSave`, `didClose`
- `textDocument/publishDiagnostics` (server → client push)
- `shutdown` / `exit`

## Architecture

- **Hand-rolled stdio JSON-RPC.** Dependency footprint is `serde_json` plus the
  path dependency on the prototype. No `tokio`, no `tower` — a single-file
  diagnostics loop does not justify an async runtime, and a synchronous
  read/dispatch/write loop stays auditable. (`tower-lsp` was the alternative; it
  is the heavier choice here.)
- **In-process check pipeline.** The server depends on the `candor`
  **library** crate by path (`../../compiler`) and calls it directly — it never
  shells out to the `candor-proto` binary. `compiler/src` is consumed, never
  modified.
  - `.cnr` → `candor::check_source_real` (real surface syntax; generics
    are handled inside `check_program_real`).
  - anything else → `candor::check_source` (throwaway `.cn`).
- **Full-document sync.** `didChange` carries the whole document (LSP
  `TextDocumentSyncKind.Full`) and is rechecked immediately. The check is cheap
  for single files, so there is no debounce timer.
- **Panic guard.** The checker call is wrapped in `catch_unwind`; a panic in an
  unfinished prototype path degrades to a single internal-error diagnostic
  rather than killing the loop.

### Span mapping

`diag.rs` carries **byte-offset** half-open spans `[start, end)`. LSP wants
`(line, character)` where `character` is a **UTF-16** code-unit count. `LineIndex`
precomputes line-start byte offsets; a position is `(binary-search for the line,
UTF-16 length of the line prefix up to the offset)`. Offsets past end clamp, and
an offset landing inside a multi-byte code point is floored to the enclosing
char boundary so the server never panics.

### Diagnostic mapping

| `candor` `Diag` | LSP `Diagnostic` |
|---|---|
| `Severity::Error` / `Warning` | `severity` `1` / `2` |
| `code` | `code` |
| `message` | `message` |
| `span` | `range` (via the mapping above) |
| `notes[]` with a span | `relatedInformation[]` (same document URI) |
| `notes[]` without a span | appended to `message` |

`check_source*` returns `Result<Vec<Diag>, Diag>`: an `Ok(vec)` is the (possibly
empty) list of check diagnostics; an `Err(d)` is a single hard parse error. Both
are mapped and published; a clean file publishes an empty array (clearing stale
diagnostics).

## Build & test

```sh
cargo build --release      # -> target/release/candor-lsp
cargo test                 # offset↔position mapping + an end-to-end loop test
cargo clippy --all-targets # clean
```

The tests cover the byte-offset→UTF-16-position mapping (ASCII, multi-line,
surrogate-pair, and mid-code-point flooring) and one end-to-end run that frames
`initialize`→`didOpen`(bad `.cnr`)→`shutdown`→`exit` through the real
read/dispatch/write loop and asserts a published diagnostic.

## Wiring into an editor

See `../vscode-candor` — its `extension.js` launches this binary over stdio via
`vscode-languageclient` and points at `target/release/candor-lsp` by default
(overridable with the `candor.lsp.serverPath` setting).
