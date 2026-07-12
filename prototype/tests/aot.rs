//! AOT gate (design 0010 §5, Stage B's cranelift-object note): `candor
//! compile` emits a LINKED NATIVE EXECUTABLE, and that standalone process — run
//! with no JIT and no `candor` present — must reproduce the tree-walking
//! oracle's observable result: `θ` as the trace hook prints it to stdout, the
//! process exit code (main's `i64` mapped to the Unix low-byte protocol, or 2 on
//! fault), and the `(kind, span)` fault JSON on stderr. This is the fourth
//! differential engine beside interpreted / MIR / native-JIT.
//!
//! The executables here are built by `cc` linking the emitted object with the
//! static C runtime. If `cc` is unavailable the gate FAILS LOUDLY (never a silent
//! skip): the compiler cannot produce a runnable artifact without a linker.

use std::path::{Path, PathBuf};
use std::process::Command;

use candor_proto::interp::{Fault, FaultKind};
use candor_proto::foreign_io;
use candor_proto::{run_source, run_source_real, RunResult};

/// The comparable observable outcome of a run (oracle or compiled process).
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// Ran to completion: `θ` (the printed trace) + the process exit byte.
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

/// The tree-walking oracle's outcome for a source string.
fn oracle_src(src: &str, real: bool) -> Option<Outcome> {
    let r = if real { run_source_real(src) } else { run_source(src) };
    match r {
        RunResult::Ok(run) => Some(Outcome::Ok { exit: run.ret as u8, trace: run.trace }),
        RunResult::Fault(f) => Some(oracle_fault(&f)),
        _ => None,
    }
}

/// The tree-walking oracle's outcome for a file or module-tree directory.
fn oracle_path(path: &Path) -> Option<Outcome> {
    if path.is_dir() {
        return match candor_proto::run_dir(path) {
            RunResult::Ok(run) => Some(Outcome::Ok { exit: run.ret as u8, trace: run.trace }),
            RunResult::Fault(f) => Some(oracle_fault(&f)),
            _ => None,
        };
    }
    let src = std::fs::read_to_string(path).ok()?;
    let real = path.extension().map(|e| e == "cnr").unwrap_or(false);
    oracle_src(&src, real)
}

/// Extract a JSON field of the shape `"<key>":<digits>` or `"<key>":"<str>"`.
fn field<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let pat = format!("\"{key}\":");
    let i = json.find(&pat)? + pat.len();
    let rest = &json[i..];
    if let Some(stripped) = rest.strip_prefix('"') {
        let end = stripped.find('"')?;
        Some(&stripped[..end])
    } else {
        let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
        Some(&rest[..end])
    }
}

/// The compiled process's outcome: compile `path` to a temp executable, run it,
/// and translate (exit, stdout, stderr) into an `Outcome`.
fn aot_outcome(path: &Path, tag: &str) -> Result<Outcome, String> {
    let out = std::env::temp_dir().join(format!(
        "candor-aot-gate-{}-{}",
        std::process::id(),
        tag
    ));
    candor_proto::compile_path(path, &out)?;
    let output = Command::new(&out)
        .output()
        .map_err(|e| format!("could not run compiled `{}`: {e}", out.display()))?;
    let _ = std::fs::remove_file(&out);

    let code = output.status.code();
    if code == Some(2) {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let kind = field(&stderr, "kind").ok_or("no kind in fault JSON")?.to_string();
        let start: usize = field(&stderr, "start").ok_or("no start")?.parse().map_err(|_| "bad start")?;
        let end: usize = field(&stderr, "end").ok_or("no end")?.parse().map_err(|_| "bad end")?;
        return Ok(Outcome::Fault { kind, start, end });
    }
    let exit = code.ok_or("compiled process killed by signal")? as u8;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trace: Vec<i64> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.trim().parse::<i64>().expect("trace line is an integer"))
        .collect();
    Ok(Outcome::Ok { exit, trace })
}

/// Assert the compiled executable's observable outcome equals the oracle's.
fn assert_aot_eq_oracle_src(src: &str, real: bool, tag: &str) {
    let o = oracle_src(src, real).expect("oracle should run this program");
    let ext = if real { "cnr" } else { "cn" };
    let srcpath = std::env::temp_dir().join(format!(
        "candor-aot-src-{}-{}.{ext}",
        std::process::id(),
        tag
    ));
    std::fs::write(&srcpath, src).unwrap();
    let a = aot_outcome(&srcpath, tag).expect("compile+run should succeed");
    let _ = std::fs::remove_file(&srcpath);
    assert_eq!(a, o, "AOT vs oracle divergence for:\n{src}");
}

fn cc_available() -> bool {
    Command::new("cc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

// ---------------------------------------------------------------------------
// 1. The fault axes (design 0010 §4/§5): every mode must deliver the same `f★`
//    (kind + span). Six+ axes: overflow, div-by-zero, conv-loss, assert,
//    requires, ensures, panic, shift-overflow, bounds, and `?`-adjacent.
// ---------------------------------------------------------------------------

const FAULTS: &[(&str, bool)] = &[
    ("fn main() -> i64 { let a: i32 = 2147483647i32; let b: i32 = a + 1i32; return conv i64 (b); }", false),
    ("fn main() -> i64 { let z: i64 = 0; let q: i64 = 10 / z; return q; }", false),
    ("fn main() -> i64 { let a: i64 = 300; let b: i8 = conv i8 (a); return 0; }", false),
    ("fn main() -> i64 { let x: i64 = 3; assert(x > 10); return 0; }", false),
    ("fn need(x: i64) requires(x > 0) -> i64 { return x; } fn main() -> i64 { return need(0 - 1); }", false),
    ("fn bad() ensures(result > 0) -> i64 { return 0 - 5; } fn main() -> i64 { return bad(); }", false),
    ("fn main() -> i64 { panic(\"boom\"); return 0; }", false),
    ("fn main() -> i64 { let a: u8 = 1u8; let b: u8 = a << 9u8; return 0; }", true),
    // bounds (array index out of range)
    ("fn main() -> i64 { let a: [3]i64 = [1, 2, 3]; let i: usize = 5; return a[i]; }", false),
    // `?`-adjacent: a division fault inside a `?`-called function
    ("enum R { ok Val(i64), Err } fn dv(n: i64, d: i64) -> R { let q: i64 = n / d; return R::Val(q); } fn run() -> R { let a: i64 = dv(10, 0)?; return R::Val(a); } fn main() -> i64 { return match run() { R::Val(v) => v, R::Err => 0 - 1 }; }", true),
];

#[test]
fn gate_aot_fault_axes() {
    assert!(cc_available(), "cc/linker unavailable: cannot build runnable executables");
    for (i, (src, real)) in FAULTS.iter().enumerate() {
        let o = oracle_src(src, *real).expect("faulting program should run");
        assert!(matches!(o, Outcome::Fault { .. }), "expected a fault:\n{src}");
        assert_aot_eq_oracle_src(src, *real, &format!("fault{i}"));
    }
}

// ---------------------------------------------------------------------------
// 2. A representative in-subset slice: traces + exit codes must match.
// ---------------------------------------------------------------------------

const OK: &[(&str, bool)] = &[
    ("fn main() -> i64 { let a: i64 = 20; let b: i64 = 22; trace(a); trace(b); return a + b; }", false),
    ("fn fib(n: i64) -> i64 { if n < 2 { return n; } return fib(n - 1) + fib(n - 2); } fn main() -> i64 { let r: i64 = fib(10); trace(r); return r; }", false),
    ("fn main() -> i64 { let mut s: i64 = 0; let mut i: i64 = 0; while i < 5 { s = s + i; trace(i); i = i + 1; } trace(s); return s; }", false),
    ("fn main() -> i64 { let a: u8 = 12u8; let b: u8 = 10u8; trace(conv i64 (a & b)); trace(conv i64 (a | b)); trace(conv i64 (a ^ b)); return 0; }", true),
];

#[test]
fn gate_aot_ok_slice() {
    assert!(cc_available(), "cc/linker unavailable");
    for (i, (src, real)) in OK.iter().enumerate() {
        assert_aot_eq_oracle_src(src, *real, &format!("ok{i}"));
    }
}

// ---------------------------------------------------------------------------
// 3. The full runnable corpus (design 0010 §5): every single-file fixture
//    (run/parity/real/generics, incl. the five §11 `.cn` and their `.cnr`
//    twins), the flat corelib, and the two module-tree directories — compiled to
//    real ELF processes and asserted equal to the oracle. Coverage is reported.
// ---------------------------------------------------------------------------

fn single_file_fixtures() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for sub in ["run", "parity", "real", "generics"] {
        let d = fixtures_dir().join(sub);
        if let Ok(rd) = std::fs::read_dir(&d) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().map(|x| x == "cn" || x == "cnr").unwrap_or(false) {
                    out.push(p);
                }
            }
        }
    }
    out.push(fixtures_dir().join("corelib_flat.cnr"));
    out.sort();
    out
}

#[test]
fn gate_aot_full_corpus() {
    assert!(cc_available(), "cc/linker unavailable: cannot build runnable executables");
    let mut equal = 0usize;
    let mut diffs: Vec<String> = Vec::new();
    let mut not_run = 0usize;

    let mut paths = single_file_fixtures();
    for name in ["corelib", "corelib_question"] {
        let d = fixtures_dir().join(name);
        if d.is_dir() {
            paths.push(d);
        }
    }

    for path in &paths {
        let o = match oracle_path(path) {
            Some(o) => o,
            None => {
                not_run += 1;
                continue;
            }
        };
        let tag: String = path
            .to_string_lossy()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect();
        match aot_outcome(path, &tag) {
            Ok(a) => {
                if a == o {
                    equal += 1;
                } else {
                    diffs.push(format!("{}: aot={a:?} oracle={o:?}", path.display()));
                }
            }
            Err(e) => diffs.push(format!("{}: compile/link error: {e}", path.display())),
        }
    }

    eprintln!("AOT GATE: {equal} runnable fixtures compiled+run == oracle; not-runnable={not_run}");
    assert!(diffs.is_empty(), "AOT divergences / failures:\n{}", diffs.join("\n"));
    assert!(equal >= 30, "expected the full runnable corpus (>=30 fixtures), got {equal}");
}


// ---------------------------------------------------------------------------
// 4. Structured concurrency (design 0012 Stage 2): the AOT C runtime's raw-
//    pthread `rt_scope_begin`/`rt_spawn`/`rt_scope_end` must reproduce the
//    oracle's observable outcome — θ merged in spawn order (deterministic),
//    the exit byte, and spawn-order-first fault identity — as a REAL process.
// ---------------------------------------------------------------------------

const CONC: &[(&str, bool)] = &[
    // The parallel-fill flagship: split_mut halves fed to two disjoint spawns
    // writing through their `slice_mut`s; the merged buffer -> exit byte (2211).
    (
        "fn fill(s: write [u8], v: u8, n: usize) -> unit { \
            let mut i: usize = 0; loop { if i >= n { break; } s[i] = v; i = i + 1; } } \
         fn main() -> i64 { let mut buf: [4]u8 = [0u8, 0u8, 0u8, 0u8]; \
            let lo: write [u8]; let hi: write [u8]; \
            split_mut(buf, 2, out lo, out hi); \
            scope { spawn fill(write lo, 1u8, 2); spawn fill(write hi, 2u8, 2); } \
            return conv i64 buf[0] + conv i64 buf[1] * 10 \
                 + conv i64 buf[2] * 100 + conv i64 buf[3] * 1000; }",
        true,
    ),
    // Per-task trace projection merged in spawn order: θ == [100, 3, 4, 200]
    // regardless of the OS-thread interleaving.
    (
        "fn work(o: write i64, v: i64) -> unit { trace(v); o.* = v * v; } \
         fn main() -> i64 { let mut a: i64 = 0; let mut b: i64 = 0; trace(100); \
            scope { spawn work(write a, 3); spawn work(write b, 4); } \
            trace(200); return a + b; }",
        true,
    ),
    // A spawned task faults (div-by-zero): the join delivers the fault on the
    // parent thread via the fault-exit path -> exit 2 + (kind, span) on stderr.
    (
        "fn work(o: write i64, v: i64, d: i64) -> unit { o.* = v / d; } \
         fn main() -> i64 { let mut a: i64 = 0; \
            scope { spawn work(write a, 10, 0); } return a; }",
        true,
    ),
];

#[test]
fn gate_aot_concurrency() {
    assert!(cc_available(), "cc/linker unavailable: cannot build runnable executables");
    for (i, (src, real)) in CONC.iter().enumerate() {
        assert_aot_eq_oracle_src(src, *real, &format!("conc{i}"));
    }
}


// ---------------------------------------------------------------------------
// 5. Native FFI (design 0011 §5, the AOT path): the std_io demonstrator
//    compiled to a REAL native binary that calls REAL libc (open/read/write/
//    close) with NO shim registry — the flat-memory `rawptr` args translated to
//    real host pointers at the boundary. Its observable result (exit byte +
//    stdout bytes) must equal the shim-backed interpreter run. THE milestone: a
//    standalone Candor binary doing genuine libc I/O with no toolchain present.
// ---------------------------------------------------------------------------

fn io_guard() -> std::sync::MutexGuard<'static, ()> {
    static G: std::sync::Mutex<()> = std::sync::Mutex::new(());
    G.lock().unwrap_or_else(|e| e.into_inner())
}

#[test]
fn gate_aot_native_io_real_libc() {
    assert!(cc_available(), "cc/linker unavailable: cannot build runnable executables");
    let _g = io_guard();
    let dir = fixtures_dir().join("std_io");
    let main_cnr = dir.join("main.cnr");
    let src = std::fs::read_to_string(&main_cnr).expect("read io fixture");

    // Expected observable: the shim-backed interpreter (real std::fs + captured
    // stdout via foreign_io), rooted at the fixture dir.
    foreign_io::reset();
    foreign_io::set_root(&dir);
    foreign_io::register_std_io();
    let (exp_ret, exp_out) = match run_source_real(&src) {
        RunResult::Ok(r) => (r.ret, foreign_io::take_stdout()),
        _ => {
            foreign_io::unregister_std_io();
            panic!("shim-backed interpreter should run the io demonstrator");
        }
    };
    foreign_io::unregister_std_io();

    // The milestone: a linked native binary that calls real libc directly, run as
    // a process with the fixture dir as cwd (so `open("input.txt")` resolves).
    let out = std::env::temp_dir().join(format!("candor-aot-io-ok-{}", std::process::id()));
    candor_proto::compile_path(&main_cnr, &out).expect("compile io demonstrator");
    let output = std::process::Command::new(&out)
        .current_dir(&dir)
        .output()
        .expect("run compiled io binary");
    let _ = std::fs::remove_file(&out);

    assert_eq!(output.status.code(), Some(exp_ret as u8 as i32), "exit byte vs shim run");
    assert_eq!(output.stdout, exp_out, "stdout bytes vs shim run");
    // The known observable: the uppercased fixture, 17 bytes read/written.
    assert_eq!(exp_ret, 17);
    assert_eq!(output.stdout, b"HELLO, CANDOR IO\n");
}

#[test]
fn gate_aot_native_io_open_error() {
    assert!(cc_available(), "cc/linker unavailable: cannot build runnable executables");
    let _g = io_guard();
    // The io module minus its demonstrator `main`, plus a `main` that opens a
    // file that does not exist -> the `Fail` arm -> a negative exit byte.
    let fixture = std::fs::read_to_string(fixtures_dir().join("std_io/main.cnr")).unwrap();
    let prefix = &fixture[..fixture.find("fn main").expect("fixture has a main")];
    let main = "fn main() -> i64 {\n\
        let name: [9]u8 = [110u8, 111u8, 112u8, 101u8, 46u8, 116u8, 120u8, 116u8, 0u8];\n\
        match open_read(slice_of(name)) {\n\
            IoResult::Ok(c) => { return 1i64; },\n\
            IoResult::Err(e) => { match e { IoError::Errno(n) => { return conv i64 n; }, } },\n\
        }\n\
    }\n";
    let src = format!("{prefix}{main}");

    let empty = std::env::temp_dir().join(format!("candor-aot-io-empty-{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();

    // Expected: the shim-backed interpreter, rooted at the (empty) dir -> -1.
    foreign_io::reset();
    foreign_io::set_root(&empty);
    foreign_io::register_std_io();
    let exp_ret = match run_source_real(&src) {
        RunResult::Ok(r) => r.ret,
        _ => {
            foreign_io::unregister_std_io();
            panic!("shim-backed interpreter should run the open-error program");
        }
    };
    foreign_io::unregister_std_io();
    assert!(exp_ret < 0, "open of a missing file should be the Fail arm (got {exp_ret})");

    // The native binary: real libc open of a missing file -> the same error value.
    let srcpath = empty.join("prog.cnr");
    std::fs::write(&srcpath, &src).unwrap();
    let out = std::env::temp_dir().join(format!("candor-aot-io-err-{}", std::process::id()));
    candor_proto::compile_path(&srcpath, &out).expect("compile open-error program");
    let output = std::process::Command::new(&out)
        .current_dir(&empty)
        .output()
        .expect("run compiled open-error binary");
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_dir_all(&empty);

    assert_eq!(output.status.code(), Some(exp_ret as u8 as i32), "error exit byte vs shim run");
}
