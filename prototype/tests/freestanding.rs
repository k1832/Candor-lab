//! Freestanding gate (design 0010 §5; LANG_PHYLOSOPHY P7/P9/NN#6): the NN#6 proof
//! that Candor has "no mandatory runtime; freestanding targets are first-class".
//!
//! `candor-proto compile --freestanding` emits an ELF linked `-nostdlib -static
//! -no-pie` — NO libc, no OS-facing runtime, a root-declared HALT fault policy
//! (P7's second point), the flat region a static NOBITS section (no mmap), and the
//! checker-proven allocation-free CORE layer as its payload. This gate:
//!   1. runs the payload + fault-axis programs as REAL standalone processes and
//!      asserts θ / exit / fault-identity equality vs the tree-walking oracle;
//!   2. asserts the PROOF: `ldd` == "not a dynamic executable", and `nm`/`objdump`
//!      show NO libc symbols and NO undefined symbols;
//!   3. asserts the core payload runs to its sentinel.
//!
//! If `cc` (the linker) is unavailable the gate FAILS LOUDLY, never silently skips.

use std::path::{Path, PathBuf};
use std::process::Command;

use candor_proto::interp::{Fault, FaultKind};
use candor_proto::{run_source, run_source_real, RunResult};

/// The comparable observable outcome of a run (oracle or freestanding process).
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// Ran to completion: θ (the printed trace) + the process exit byte.
    Ok { exit: u8, trace: Vec<i64> },
    /// Faulted: the delivered `(kind, span)` (exit 2 on the process side).
    Fault { kind: String, start: usize, end: usize },
}

fn kind_str(k: FaultKind) -> String {
    use FaultKind::*;
    match k {
        Overflow => "overflow",
        DivByZero => "div_by_zero",
        Bounds => "bounds",
        ConvLoss => "conv_loss",
        Assert => "assert",
        Requires => "requires",
        Ensures => "ensures",
        Panic => "panic",
        BadPointer => "bad_pointer",
        NoForeignRuntime => "no_foreign_runtime",
    }
    .to_string()
}

fn oracle_fault(f: &Fault) -> Outcome {
    Outcome::Fault { kind: kind_str(f.kind), start: f.span.start, end: f.span.end }
}

fn oracle_src(src: &str, real: bool) -> Option<Outcome> {
    let r = if real { run_source_real(src) } else { run_source(src) };
    match r {
        RunResult::Ok(run) => Some(Outcome::Ok { exit: run.ret as u8, trace: run.trace }),
        RunResult::Fault(f) => Some(oracle_fault(&f)),
        _ => None,
    }
}

fn oracle_path(path: &Path) -> Option<Outcome> {
    let src = std::fs::read_to_string(path).ok()?;
    let real = path.extension().map(|e| e == "cnr").unwrap_or(false);
    oracle_src(&src, real)
}

fn cc_available() -> bool {
    Command::new("cc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn scratch(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("candor-fs-gate-{}-{}", std::process::id(), tag))
}

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Compile `path` freestanding to a temp ELF; return its path (caller removes it).
fn compile_fs(path: &Path, tag: &str) -> PathBuf {
    let out = scratch(tag);
    candor_proto::compile_path_freestanding(path, &out)
        .unwrap_or_else(|e| panic!("freestanding compile of {} failed: {e}", path.display()));
    out
}

/// Run a freestanding ELF and translate (exit, stdout, stderr) into an `Outcome`.
/// On the fault path the runtime writes `candor fault: <kind> <start> <end>` to
/// stderr and exits 2 (the HALT policy).
fn run_outcome(exe: &Path) -> Outcome {
    let output = Command::new(exe)
        .output()
        .unwrap_or_else(|e| panic!("could not run freestanding `{}`: {e}", exe.display()));
    let code = output.status.code().expect("process killed by signal");
    if code == 2 {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let line = stderr.lines().find(|l| l.starts_with("candor fault:")).expect("fault line");
        let toks: Vec<&str> = line.trim_start_matches("candor fault:").split_whitespace().collect();
        return Outcome::Fault {
            kind: toks[0].to_string(),
            start: toks[1].parse().expect("start int"),
            end: toks[2].parse().expect("end int"),
        };
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trace: Vec<i64> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.trim().parse::<i64>().expect("trace line is an integer"))
        .collect();
    Outcome::Ok { exit: code as u8, trace }
}

// ---------------------------------------------------------------------------
// 1. The core payload: arena + opt + checked arithmetic + a drop hook, no Box/
//    Alloc. Compiled freestanding, run as a process, asserted == oracle and run
//    to its sentinel.
// ---------------------------------------------------------------------------

#[test]
fn gate_freestanding_core_payload() {
    assert!(cc_available(), "cc/linker unavailable: cannot build freestanding executables");
    let payload = fixtures_dir().join("freestanding/payload.cnr");
    let oracle = oracle_path(&payload).expect("oracle should run the core payload");

    let exe = compile_fs(&payload, "payload");
    let got = run_outcome(&exe);
    let _ = std::fs::remove_file(&exe);

    assert_eq!(got, oracle, "freestanding payload diverged from the oracle");
    // The sentinel: 60 (arena sum) + 3 (count) + 12 (opt) = 75, θ ends at the
    // drop hook's 99 — the core layer ran to completion.
    assert_eq!(got, Outcome::Ok { exit: 75, trace: vec![60, 3, 12, 99] });
}

// ---------------------------------------------------------------------------
// 2. The fault axis (P7 HALT policy): each faulting program halts with exit 2 and
//    a fault line whose kind + span equal the oracle's f★.
// ---------------------------------------------------------------------------

const FAULTS: &[(&str, &str)] = &[
    // overflow (checked i32 add)
    ("ovf", "fn main() -> i64 { let a: i32 = 2147483647i32; let b: i32 = a + 1i32; return conv i64 (b); }"),
    // bounds (array index out of range — the arena's built-in bounds check)
    ("bounds", "fn main() -> i64 { let a: [3]i64 = [1, 2, 3]; let i: usize = 5; return a[i]; }"),
    // division by zero
    ("div", "fn main() -> i64 { let z: i64 = 0; let q: i64 = 10 / z; return q; }"),
];

#[test]
fn gate_freestanding_fault_axis() {
    assert!(cc_available(), "cc/linker unavailable");
    for (tag, src) in FAULTS {
        let oracle = oracle_src(src, false).expect("faulting program should run");
        assert!(matches!(oracle, Outcome::Fault { .. }), "expected a fault:\n{src}");

        let srcpath = scratch(&format!("{tag}src")).with_extension("cn");
        std::fs::write(&srcpath, src).unwrap();
        let exe = compile_fs(&srcpath, tag);
        let got = run_outcome(&exe);
        let _ = std::fs::remove_file(&exe);
        let _ = std::fs::remove_file(&srcpath);

        assert_eq!(got, oracle, "freestanding fault identity diverged for:\n{src}");
    }
}

// ---------------------------------------------------------------------------
// 3. THE PROOF (NN#6): the emitted ELF is statically linked with no libc. `ldd`
//    reports "not a dynamic executable"; `nm` shows no undefined symbols and no
//    libc symbol names; `readelf` shows no PT_INTERP (no dynamic loader).
// ---------------------------------------------------------------------------

fn tool(name: &str, args: &[&str], exe: &Path) -> String {
    let out = Command::new(name)
        .args(args)
        .arg(exe)
        .output()
        .unwrap_or_else(|e| panic!("could not run `{name}`: {e}"));
    // nm exits non-zero with "no symbols" on a stripped dynsym table — that is a
    // valid (empty) result, so read both streams.
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn gate_freestanding_no_libc_proof() {
    assert!(cc_available(), "cc/linker unavailable");
    let payload = fixtures_dir().join("freestanding/payload.cnr");
    let exe = compile_fs(&payload, "proof");

    // (a) ldd: not a dynamic executable.
    let ldd = tool("ldd", &[], &exe);
    assert!(
        ldd.contains("not a dynamic executable"),
        "ldd says the freestanding binary is dynamic:\n{ldd}"
    );

    // (b) no PT_INTERP program header (no dynamic loader is invoked).
    let phdrs = tool("readelf", &["-l"], &exe);
    assert!(!phdrs.contains("INTERP"), "the binary has a PT_INTERP segment:\n{phdrs}");
    // and it is a static EXEC, not a PIE/DYN.
    let hdr = tool("readelf", &["-h"], &exe);
    assert!(hdr.contains("EXEC"), "expected an EXEC ELF (static, -no-pie):\n{hdr}");

    // (c) no undefined symbols — everything the object needs is satisfied by the
    //     freestanding runtime, nothing is left for a libc to provide.
    let undef = tool("nm", &["-u"], &exe);
    let undef_lines: Vec<&str> = undef.lines().filter(|l| l.trim_start().starts_with('U')).collect();
    assert!(undef_lines.is_empty(), "unexpected undefined symbols:\n{undef}");

    // (d) no libc symbol NAMES anywhere in the symbol table.
    let syms = tool("nm", &[], &exe);
    for bad in [
        "printf", "malloc", "free", "mmap", "munmap", "pthread", "setjmp",
        "longjmp", "_setjmp", "_longjmp", "__libc", "GLIBC", "fwrite", "puts",
        "__stack_chk", "abort",
    ] {
        assert!(
            !syms.to_lowercase().contains(&bad.to_lowercase()),
            "found a libc symbol `{bad}` in the freestanding binary:\n{syms}"
        );
    }

    // (e) the flat region is a real symbol pinned at the fixed VA MEM_BASE.
    assert!(
        syms.contains("candor_flat_region") && syms.contains("candor_entry"),
        "expected the flat-region + entry symbols:\n{syms}"
    );

    let _ = std::fs::remove_file(&exe);
}
