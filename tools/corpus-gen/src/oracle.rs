//! The filter: a thin wrapper over the `candor-proto` CLI. The generator NEVER
//! judges its own output — every keep/reject decision is a `candor-proto`
//! `check`/`run` exit code plus its JSON. This is Bet 6's anti-circularity in
//! code: the toolchain validates internal consistency (compiles, runs to the
//! predicted sentinel, emits the predicted diagnostic); it does not — and cannot
//! — validate that the corpus is *correct* or *idiomatic*. The corpus is TRAINING
//! material; the eval anchors stay external (see `README.md#bet-6`).
//!
//! Contract observed from `compiler/src/main.rs`:
//! * `check <file>` — exit 0 clean; exit 1 with one JSON diagnostic per stdout
//!   line otherwise (`P0xxx` = parse, `E....` = check).
//! * `run <file>`   — exit 0 with the `i64` sentinel on stdout; a runtime fault
//!   writes a JSON fault to stderr and exits 2.

use std::path::Path;
use std::process::Command;

/// The candor-proto invocation target (a path or a bare name on `PATH`).
#[derive(Clone)]
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

/// The `"code"` of the first JSON-object line in `text` (a diagnostic's code),
/// matching how the sibling eval-harness reads diagnostics.
pub fn first_code(text: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            return v.get("code").and_then(|c| c.as_str()).map(str::to_string);
        }
    }
    None
}
