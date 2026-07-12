//! The std I/O boundary module (design 0013 I/O layer over the 0011 foreign
//! boundary): the `sys_open`/`sys_close`/`sys_read`/`sys_write` externs with their
//! trust clauses, the safe `open_read`/`read_into`/`write_all`/`close` wrappers
//! that discharge the `foreign` effect, and the host-backed shims that run real
//! deterministic I/O on both engines. The demonstrator opens a fixture file,
//! reads it, uppercases the bytes, and writes the result to a captured stdout.
//!
//! These tests share process-global state (the shim registry and the captured
//! I/O buffers), so they serialize on `GUARD`.

use candor_proto::foreign_io;
use candor_proto::{run_source_real, run_source_real_mir, MirRunResult, RunResult};
use std::sync::{Mutex, MutexGuard};

static GUARD: Mutex<()> = Mutex::new(());

fn lock() -> MutexGuard<'static, ()> {
    GUARD.lock().unwrap_or_else(|e| e.into_inner())
}

fn dir() -> String {
    format!("{}/tests/fixtures/std_io", env!("CARGO_MANIFEST_DIR"))
}
fn fixture() -> String {
    std::fs::read_to_string(format!("{}/main.cnr", dir())).expect("read io fixture")
}
/// The io module minus its demonstrator `main` — externs, `Res`/`IoError`, and the
/// pub wrappers — so a focused test can append its own `main` and drive the
/// *actual* module wrappers (no re-implementation).
fn module_prefix() -> String {
    let f = fixture();
    let idx = f.find("fn main").expect("fixture has a main");
    f[..idx].to_string()
}
fn probe(main_body: &str) -> String {
    format!("{}{}", module_prefix(), main_body)
}

fn run_ok_mir(src: &str) -> i64 {
    match run_source_real_mir(src) {
        MirRunResult::Ok(r) => r.ret,
        MirRunResult::Fault(f) => panic!("mir fault: {}", f.to_json()),
        MirRunResult::Unsupported(s) => panic!("mir unsupported: {s}"),
        other => panic!("mir not ok: {}", matches!(other, MirRunResult::Ok(_))),
    }
}

fn run_ok(src: &str) -> i64 {
    match run_source_real(src) {
        RunResult::Ok(r) => r.ret,
        RunResult::Fault(f) => panic!("fault: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!("check: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        RunResult::ParseError(d) => panic!("parse: {}", d.to_json()),
    }
}

// ---- 1. the module checks clean (discharge + mappability + trust) ----------
#[test]
fn io_module_checks_clean() {
    let _g = lock();
    let diags = candor_proto::check_source_real(&fixture()).expect("parses");
    let errs: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == candor_proto::diag::Severity::Error)
        .map(|d| d.code.clone())
        .collect();
    assert!(errs.is_empty(), "io module should check clean, got {errs:?}");
}

// ---- 2. the demonstrator: real file I/O on the tree-walker -----------------
#[test]
fn demonstrator_reads_transforms_writes_tree_walker() {
    let _g = lock();
    foreign_io::reset();
    foreign_io::set_root(dir());
    foreign_io::register_std_io();
    let ret = run_ok(&fixture());
    let out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();
    assert_eq!(ret, 17, "returns the byte count read/written");
    assert_eq!(out, b"HELLO, CANDOR IO\n", "uppercased fixture written to stdout");
}

// ---- 3. same program, MIR engine, identical result -------------------------
#[test]
fn demonstrator_mir_engine_equal() {
    let _g = lock();
    foreign_io::reset();
    foreign_io::set_root(dir());
    foreign_io::register_std_io();
    let r = run_source_real_mir(&fixture());
    let out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();
    match r {
        MirRunResult::Ok(run) => {
            assert_eq!(run.ret, 17);
            assert_eq!(out, b"HELLO, CANDOR IO\n");
        }
        MirRunResult::Fault(f) => panic!("mir fault: {}", f.to_json()),
        MirRunResult::Unsupported(s) => panic!("mir unsupported: {s}"),
        other => panic!("mir not ok: {}", matches!(other, MirRunResult::Ok(_))),
    }
}

// ---- 4. write_all writes the buffer and returns the count ------------------
const WRITE_PROBE: &str = r#"fn main() -> i64 {
    let data: [3]u8 = [88u8, 89u8, 90u8];
    match write_all(stdout(), slice_of(data)) {
        IoResult::Ok(c) => { return conv i64 c; },
        IoResult::Err(e) => { match e { IoError::Errno(n) => { return 0i64 - 100i64 + conv i64 n; }, } },
    }
}
"#;
#[test]
fn write_all_writes_and_returns_count() {
    let _g = lock();
    foreign_io::reset();
    foreign_io::register_std_io();
    let ret = run_ok(&probe(WRITE_PROBE));
    let out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();
    assert_eq!(ret, 3);
    assert_eq!(out, b"XYZ");
}

// ---- 5. a short write (OS reports fewer bytes) is the Fail arm -------------
#[test]
fn write_all_short_write_is_fail() {
    let _g = lock();
    foreign_io::reset();
    foreign_io::register_std_io();
    // a shim that claims it wrote only 1 of the 3 requested bytes.
    foreign_io::register_shim_override_write_short(1);
    let ret = run_ok(&probe(WRITE_PROBE));
    foreign_io::unregister_std_io();
    assert!(ret < 0, "short write must hit the Fail arm, got {ret}");
}

// ---- 6. a write error (-1) is the Fail arm --------------------------------
#[test]
fn write_all_error_is_fail() {
    let _g = lock();
    foreign_io::reset();
    foreign_io::register_std_io();
    foreign_io::register_shim_override_write_short(-1);
    let ret = run_ok(&probe(WRITE_PROBE));
    foreign_io::unregister_std_io();
    assert!(ret < 0, "write error must hit the Fail arm, got {ret}");
}

// ---- 7. read_into fills the buffer from stdin -----------------------------
const READ_PROBE: &str = r#"fn main() -> i64 {
    let mut buf: [8]u8 = [0u8; 8];
    let n: usize = match read_into(stdin(), slice_of_mut(buf)) {
        IoResult::Ok(c) => c,
        IoResult::Err(e) => { return 0i64 - 9i64; },
    };
    let mut i: usize = 0usize;
    let mut acc: i64 = 0i64;
    loop {
        if i >= n { break; }
        acc = acc + conv i64 buf[i];
        i = i + 1usize;
    }
    return acc + conv i64 n * 1000i64;
}
"#;
#[test]
fn read_into_reads_from_stdin() {
    let _g = lock();
    foreign_io::reset();
    foreign_io::register_std_io();
    foreign_io::set_stdin(b"AB"); // 65, 66
    let ret = run_ok(&probe(READ_PROBE));
    foreign_io::unregister_std_io();
    assert_eq!(ret, 2131, "n=2 (2000) + acc=131 proves the bytes landed");
}

// ---- 8. a read error (-1) is the Fail arm ---------------------------------
#[test]
fn read_into_error_is_fail() {
    let _g = lock();
    foreign_io::reset();
    foreign_io::register_std_io();
    foreign_io::register_shim_override_read_error();
    let ret = run_ok(&probe(READ_PROBE));
    foreign_io::unregister_std_io();
    assert_eq!(ret, -9, "read error hits the Fail arm");
}

// ---- 9. open of a missing file is the Fail arm ----------------------------
#[test]
fn open_failure_is_fail() {
    let _g = lock();
    foreign_io::reset();
    let empty = std::env::temp_dir().join("candor_io_empty_root");
    std::fs::create_dir_all(&empty).unwrap();
    foreign_io::set_root(&empty);
    foreign_io::register_std_io();
    let ret = run_ok(&fixture());
    foreign_io::unregister_std_io();
    assert_eq!(ret, -1, "missing input.txt -> open_read Fail -> main returns -1");
}

// ---- 10. the stdin/stdout/stderr fd constants -----------------------------
const CONST_PROBE: &str = r#"fn main() -> i64 {
    return conv i64 stdin() * 100i64 + conv i64 stdout() * 10i64 + conv i64 stderr();
}
"#;
#[test]
fn std_stream_constants() {
    let _g = lock();
    let ret = run_ok(&probe(CONST_PROBE));
    assert_eq!(ret, 12, "stdin=0, stdout=1, stderr=2");
}

// ---- 11. writes to stderr are captured separately -------------------------
const STDERR_PROBE: &str = r#"fn main() -> i64 {
    let d: [2]u8 = [69u8, 82u8];
    match write_all(stderr(), slice_of(d)) {
        IoResult::Ok(c) => { return conv i64 c; },
        IoResult::Err(e) => { return 0i64 - 1i64; },
    }
}
"#;
#[test]
fn write_to_stderr_captured() {
    let _g = lock();
    foreign_io::reset();
    foreign_io::register_std_io();
    let ret = run_ok(&probe(STDERR_PROBE));
    let err = foreign_io::take_stderr();
    let out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();
    assert_eq!(ret, 2);
    assert_eq!(err, b"ER");
    assert!(out.is_empty(), "nothing went to stdout");
}

// ---- 12. an unregistered extern is the honest no_foreign_runtime fault -----
#[test]
fn no_foreign_runtime_when_unregistered() {
    let _g = lock();
    foreign_io::reset();
    foreign_io::set_root(dir());
    foreign_io::unregister_std_io(); // ensure no shims
    match run_source_real(&fixture()) {
        RunResult::Fault(f) => {
            assert_eq!(f.kind, candor_proto::interp::FaultKind::NoForeignRuntime);
        }
        other => panic!("expected no_foreign_runtime, got ok={}", matches!(other, RunResult::Ok(_))),
    }
}

// ---- 13. candor audit enumerates the externs, trust, and discharge --------
#[test]
fn audit_enumerates_io_externs_trust_and_discharge() {
    let _g = lock();
    let json = candor_proto::audit::audit_path(std::path::Path::new(&dir())).expect("audit");
    for name in ["sys_open", "sys_close", "sys_read", "sys_write"] {
        assert!(json.contains(&format!("\"name\": \"{name}\"")), "extern {name} missing from audit");
    }
    assert!(json.contains("valid_nul_terminated"), "open's trust predicate enumerated");
    assert!(json.contains("thread_confined"), "read/write trust predicate enumerated");
    assert!(json.contains("\"externs\": 4"), "four externs in summary");
    assert!(json.contains("\"exports\": 0"), "no exports");
    assert!(json.contains("\"undischarged_foreign_wrappers\": 0"), "every wrapper discharges");
    assert!(json.contains("discharges foreign"), "wrappers shown as discharging");
    assert!(!json.contains("propagates foreign"), "no wrapper leaks foreign — pub API is safe");
}

// ---- 14/15. `?` widens IoError -> AppErr across write_all (both engines) ----
// The audited `std_io` module stays non-generic (so `candor audit` keeps its
// foreign-effect discharge report — see the run report's fork); the `From`
// widening and its `?` chain are defined HERE, in the probe, and driven over the
// real `write_all`/`open_read` wrappers. `?` unwraps `IoResult::Ok` and, on
// `IoResult::Err`, widens `IoError` into `AppErr` via the `From` impl, returning
// the enclosing `AppResult::Err` (cross-type `?`, design 0007 §7.1).
const WRITE_APP_PROBE: &str = r#"
enum AppErr { Io(IoError), Denied }
enum AppResult { ok Ok(usize), Err(AppErr) }
interface From[E] { fn from(e: E) -> Self; }
impl From[IoError] for AppErr { fn from(e: IoError) -> Self { return AppErr::Io(e); } }
fn write_app(fd: i32, s: read [u8]) -> AppResult {
    let n: usize = write_all(fd, s)?;
    return AppResult::Ok(n);
}
fn main() -> i64 {
    let data: [3]u8 = [88u8, 89u8, 90u8];
    match write_app(stdout(), slice_of(data)) {
        AppResult::Ok(c) => { return conv i64 c; },
        AppResult::Err(e) => { match e { AppErr::Io(io) => { return 0i64 - 1i64; }, AppErr::Denied => { return 0i64 - 2i64; }, } },
    }
}
"#;
#[test]
fn write_app_question_widens_ioerror_both_engines() {
    let _g = lock();
    let src = probe(WRITE_APP_PROBE);
    // tree-walker: happy path — `?` passes Ok through; stdout captures "XYZ".
    foreign_io::reset();
    foreign_io::register_std_io();
    let tw = run_ok(&src);
    let tw_out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();
    // MIR: identical.
    foreign_io::reset();
    foreign_io::register_std_io();
    let mir = run_ok_mir(&src);
    let mir_out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();
    assert_eq!(tw, 3);
    assert_eq!(mir, 3);
    assert_eq!(tw_out, b"XYZ");
    assert_eq!(mir_out, b"XYZ");
}

#[test]
fn write_app_question_propagates_widened_error_both_engines() {
    let _g = lock();
    let src = probe(WRITE_APP_PROBE);
    // A shim that reports a write error (-1) -> write_all Err -> `?` widens
    // IoError -> AppErr and early-returns -> main hits the AppResult::Err arm.
    foreign_io::reset();
    foreign_io::register_std_io();
    foreign_io::register_shim_override_write_short(-1);
    let tw = run_ok(&src);
    foreign_io::unregister_std_io();
    foreign_io::reset();
    foreign_io::register_std_io();
    foreign_io::register_shim_override_write_short(-1);
    let mir = run_ok_mir(&src);
    foreign_io::unregister_std_io();
    assert_eq!(tw, -1, "widened IoError::Errno(-1) -> AppErr::Io, early-returned");
    assert_eq!(mir, -1);
}

// ---- 16. `?` widens an open error across open_read (both engines) -----------
const OPEN_APP_PROBE: &str = r#"
enum AppErr { Io(IoError), Denied }
enum AppResult { ok Ok(i32), Err(AppErr) }
interface From[E] { fn from(e: E) -> Self; }
impl From[IoError] for AppErr { fn from(e: IoError) -> Self { return AppErr::Io(e); } }
fn open_app(path: read [u8]) -> AppResult {
    let fd: usize = open_read(path)?;
    return AppResult::Ok(conv i32 fd);
}
fn main() -> i64 {
    let name: [10]u8 = [105u8, 110u8, 112u8, 117u8, 116u8, 46u8, 116u8, 120u8, 116u8, 0u8];
    match open_app(slice_of(name)) {
        AppResult::Ok(fd) => { return conv i64 fd; },
        AppResult::Err(e) => { match e { AppErr::Io(io) => { return 0i64 - 5i64; }, AppErr::Denied => { return 0i64 - 6i64; }, } },
    }
}
"#;
#[test]
fn open_app_question_widens_open_error_both_engines() {
    let _g = lock();
    let src = probe(OPEN_APP_PROBE);
    let empty = std::env::temp_dir().join("candor_io_app_empty_root");
    std::fs::create_dir_all(&empty).unwrap();
    foreign_io::reset();
    foreign_io::set_root(&empty);
    foreign_io::register_std_io();
    let tw = run_ok(&src);
    foreign_io::unregister_std_io();
    foreign_io::reset();
    foreign_io::set_root(&empty);
    foreign_io::register_std_io();
    let mir = run_ok_mir(&src);
    foreign_io::unregister_std_io();
    // open of a missing file -> open_read Err -> `?` widens to AppErr::Io -> -5.
    assert_eq!(tw, -5, "missing file -> widened IoError -> AppErr::Io arm");
    assert_eq!(mir, -5);
}
