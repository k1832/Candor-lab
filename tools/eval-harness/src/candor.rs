//! The oracle: a thin wrapper over the `candor-proto` CLI (the prototype
//! toolchain). The harness NEVER re-implements the language's judgement — every
//! pass/fail is a `candor-proto check`/`run` exit code + its JSON output.
//!
//! Contract observed from `prototype/src/main.rs`:
//! * `check <file>` — exit 0 clean; exit 1 with one JSON diagnostic per stdout
//!   line otherwise (code `P0xxx`/`L*` = parse/lex, `E*` = check).
//! * `run <file>`   — exit 0 with the `i64` sentinel on stdout; a fault writes a
//!   JSON fault object to stderr and exits with the fault code (2); check/parse
//!   errors reaching `run` go to stdout with exit 1.

use std::path::Path;
use std::process::Command;

/// The candor-proto invocation target (a path or a bare name on `PATH`).
pub struct Oracle {
    bin: String,
}

/// The captured result of one CLI invocation.
pub struct CmdOut {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Oracle {
    pub fn new(bin: impl Into<String>) -> Oracle {
        Oracle { bin: bin.into() }
    }

    pub fn bin(&self) -> &str {
        &self.bin
    }

    fn invoke(&self, subcmd: &str, file: &Path) -> std::io::Result<CmdOut> {
        let out = Command::new(&self.bin).arg(subcmd).arg(file).output()?;
        Ok(CmdOut {
            code: out.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        })
    }

    pub fn check(&self, file: &Path) -> std::io::Result<CmdOut> {
        self.invoke("check", file)
    }

    pub fn run(&self, file: &Path) -> std::io::Result<CmdOut> {
        self.invoke("run", file)
    }
}

/// Parse the first non-empty line of `text` as a JSON object (a diagnostic or a
/// fault), returning it and, when present, its `"code"` string.
pub fn first_json(text: &str) -> Option<(serde_json::Value, Option<String>)> {
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            let code = v.get("code").and_then(|c| c.as_str()).map(str::to_string);
            return Some((v, code));
        }
    }
    None
}
