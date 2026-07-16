//! The std I/O boundary module (design 0013 I/O layer over the 0011 foreign
//! boundary): the `sys_open`/`sys_close`/`sys_read`/`sys_write` externs with their
//! trust clauses, the safe `open_read`/`read_into`/`write_all`/`close` wrappers
//! that discharge the `foreign` effect, and the host-backed shims that run real
//! deterministic I/O on both engines. The demonstrator opens a fixture file,
//! reads it, uppercases the bytes, and writes the result to a captured stdout.
//!
//! These tests share process-global state (the shim registry and the captured
//! I/O buffers), so they serialize on `GUARD`.

use candor::foreign_io;
use candor::{run_source_real, run_source_real_mir, MirRunResult, RunResult};
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
    let diags = candor::check_source_real(&fixture()).expect("parses");
    let errs: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == candor::diag::Severity::Error)
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
            assert_eq!(f.kind, candor::interp::FaultKind::NoForeignRuntime);
        }
        other => panic!("expected no_foreign_runtime, got ok={}", matches!(other, RunResult::Ok(_))),
    }
}

// ---- 13. candor audit enumerates the externs, trust, and discharge --------
#[test]
fn audit_enumerates_io_externs_trust_and_discharge() {
    let _g = lock();
    let json = candor::audit::audit_path(std::path::Path::new(&dir())).expect("audit");
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


// ---------------------------------------------------------------------------
// File-I/O layer over the std_io boundary (native-String payoff): read a whole
// file into a growable `String`, write a `String` out. Defined as a probe
// prelude (like WRITE_APP_PROBE above) rather than baked into the audited
// `main.cnr` module: `read_to_string` needs a `[u8] -> str` reinterpret to append
// the file bytes, and the only such primitive (`str_from_unchecked`) lowers on
// the tree-walker ONLY — not the MIR/native builtin set. Baking it into the
// shared fixture would make every MIR std_io test `Unsupported` (MIR lowering is
// whole-program/eager). So the read path is proven on the tree-walker via the
// shims; `write_str` (all-native: `as_bytes(as_str(..))` + `write_all`) is proven
// on BOTH engines below.
//
// `StrIoResult` is a String-carrying sibling of the module's non-generic
// `IoResult` (the audited module stays non-generic — see the fork note above); `?`
// propagates `IoError` across it via the identity `From` impl.
const FILE_API: &str = r#"
struct AllocVtable {
    alloc: fn(ctx: rawptr u8, size: usize, align: usize) alloc -> rawptr u8,
    free: fn(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) alloc -> unit,
}
copy struct Alloc { ctx: rawptr u8, vt: rawptr AllocVtable }

struct FreeList { next: usize, end: usize, head: rawptr u8 }
struct FreeBlock { next: rawptr u8, size: usize }

fn with_window(base: usize, size: usize) -> FreeList {
    unsafe "the empty free list starts with a null head; no block is threaded yet" {
        return FreeList { next: base, end: base + size, head: ptr_null[u8]() };
    }
}

fn block_span(size: usize, align: usize) -> usize {
    let mut need: usize = size;
    if need < 16usize {
        need = 16usize;
    }
    return (need + align - 1usize) / align * align;
}

fn freelist_alloc(ctx: rawptr u8, size: usize, align: usize) -> rawptr u8 {
    unsafe "ctx points at the live FreeList whose [next, end) window and address-ordered free chain are reserved to this arena alone; every carved block stays inside [next, end) and is >= header-sized, and a split remainder (kept only when >= MIN_SPLIT) is written into the block's own tail" {
        let st: FreeList = ptr_read(cast_ptr[FreeList](ctx));
        let need: usize = block_span(size, align);
        let mut prev: rawptr u8 = ptr_null[u8]();
        let mut cur: rawptr u8 = st.head;
        while !is_null(cur) {
            let blk: FreeBlock = ptr_read(cast_ptr[FreeBlock](cur));
            if blk.size >= need {
                if blk.size - need >= 32usize {
                    let rem: rawptr u8 = addr_to_ptr[u8](ptr_to_addr(cur) + need);
                    ptr_write(cast_ptr[FreeBlock](rem), FreeBlock { next: blk.next, size: blk.size - need });
                    if is_null(prev) {
                        ptr_write(cast_ptr[FreeList](ctx), FreeList { next: st.next, end: st.end, head: rem });
                    } else {
                        let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
                        ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: rem, size: pblk.size });
                    }
                } else {
                    if is_null(prev) {
                        ptr_write(cast_ptr[FreeList](ctx), FreeList { next: st.next, end: st.end, head: blk.next });
                    } else {
                        let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
                        ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: blk.next, size: pblk.size });
                    }
                }
                return cur;
            }
            prev = cur;
            cur = blk.next;
        }
        let aligned: usize = (st.next + align - 1usize) / align * align;
        if aligned + need > st.end {
            return ptr_null[u8]();
        }
        ptr_write(cast_ptr[FreeList](ctx), FreeList { next: aligned + need, end: st.end, head: st.head });
        return addr_to_ptr[u8](aligned);
    }
}

fn freelist_free(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) -> unit {
    unsafe "the free list is kept ADDRESS-ORDERED; the freed block's own storage (>= header-sized, guaranteed by block_span in alloc) holds its FreeBlock header {next, size}, and a merge joins two blocks ONLY when their byte spans are exactly adjacent (addr + size == neighbour addr), so a merge can never overlap live memory nor bridge a gap" {
        let st: FreeList = ptr_read(cast_ptr[FreeList](ctx));
        let mut cap: usize = block_span(size, align);
        let a: usize = ptr_to_addr(ptr);
        let mut prev: rawptr u8 = ptr_null[u8]();
        let mut cur: rawptr u8 = st.head;
        while !is_null(cur) && ptr_to_addr(cur) < a {
            let cblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](cur));
            prev = cur;
            cur = cblk.next;
        }
        let mut link: rawptr u8 = cur;
        if !is_null(cur) {
            let nblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](cur));
            if a + cap == ptr_to_addr(cur) {
                cap = cap + nblk.size;
                link = nblk.next;
            }
        }
        if !is_null(prev) {
            let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
            if ptr_to_addr(prev) + pblk.size == a {
                ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: link, size: pblk.size + cap });
                return;
            }
        }
        ptr_write(cast_ptr[FreeBlock](ptr), FreeBlock { next: link, size: cap });
        if is_null(prev) {
            ptr_write(cast_ptr[FreeList](ctx), FreeList { next: st.next, end: st.end, head: ptr });
        } else {
            let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
            ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: ptr, size: pblk.size });
        }
    }
}

static FREELIST_VT: AllocVtable = AllocVtable { alloc: freelist_alloc, free: freelist_free };

fn mk_alloc(state: write FreeList) -> Alloc {
    unsafe "ctx (the caller's FreeList local) and vt (the static FREELIST_VT) outlive every String this handle serves; the caller keeps the window and FREELIST_VT alive for the whole run and drops every String before the window dies" {
        return Alloc { ctx: cast_ptr[u8](addr_of_mut(state.*)), vt: addr_of(FREELIST_VT) };
    }
}

interface From[E] { fn from(e: E) -> Self; }
impl From[IoError] for IoError { fn from(e: IoError) -> Self { return e; } }
enum StrIoResult { ok Ok(String), Err(IoError) }

// Loop `read_into` a fixed [64]u8 stack buffer into an owned growable String:
// `Ok(0)` is EOF (return the String); `Ok(n)` appends the first n bytes; `Err` is
// propagated by `?`. The n bytes are appended as a bounded str-view of buf[0..n];
// `str_from_unchecked` is required (not `str_from`) because a multibyte char may
// straddle a read boundary — per-chunk UTF-8 validation would wrongly reject a
// whole that is valid. The bytes are only ever copied (append never re-validates);
// the assembled String is valid UTF-8 iff the source is.
fn read_to_string(a: read Alloc, fd: i32) alloc -> StrIoResult {
    let mut s: String = string_new(a);
    let mut buf: [64]u8 = [0u8; 64];
    loop {
        let n: usize = read_into(fd, slice_of_mut(buf))?;
        if n == 0usize { break; }
        let bytes: [u8] = subslice(slice_of(buf), 0usize, n);
        unsafe "file bytes appended raw: a multibyte char may straddle a read boundary, so per-chunk validation would wrongly reject a valid whole; the accumulated String is valid UTF-8 iff the source is" {
            append(write s, str_from_unchecked(bytes));
        }
    }
    return StrIoResult::Ok(s);
}

// open -> read_to_string -> close. On the read-error path the `?` early-returns
// BEFORE `close`, leaking the fd (accepted for this slice: closing cleanly on the
// error arm fights the owned-String-through-`?` flow).
fn read_file(a: read Alloc, path: read [u8]) alloc -> StrIoResult {
    let fd: usize = open_read(path)?;
    let s: String = read_to_string(a, conv i32 fd)?;
    let cr: IoResult = close(conv i32 fd);
    return StrIoResult::Ok(s);
}

// Write the String's bytes: `as_str` borrows the built text, `as_bytes` is the
// free str -> [u8] retype, `write_all` does the syscall. All-native (no str-view
// constructor), so this lowers on the MIR/native backends too.
fn write_str(fd: i32, s: read String) -> IoResult {
    return write_all(fd, as_bytes(as_str(read s.*)));
}
"#;

fn file_probe(main_body: &str) -> String {
    format!("{}{}{}", module_prefix(), FILE_API, main_body)
}

// The all-native `write_str` in isolation (no `read_to_string`, so no tree-only
// str-view constructor): builds a String from literals + `push`, writes it. This
// program lowers on the MIR backend, so `write_str` is proven on BOTH engines.
const WRITE_STR_NATIVE: &str = r#"
struct AllocVtable {
    alloc: fn(ctx: rawptr u8, size: usize, align: usize) alloc -> rawptr u8,
    free: fn(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) alloc -> unit,
}
copy struct Alloc { ctx: rawptr u8, vt: rawptr AllocVtable }

struct FreeList { next: usize, end: usize, head: rawptr u8 }
struct FreeBlock { next: rawptr u8, size: usize }

fn with_window(base: usize, size: usize) -> FreeList {
    unsafe "the empty free list starts with a null head; no block is threaded yet" {
        return FreeList { next: base, end: base + size, head: ptr_null[u8]() };
    }
}

fn block_span(size: usize, align: usize) -> usize {
    let mut need: usize = size;
    if need < 16usize {
        need = 16usize;
    }
    return (need + align - 1usize) / align * align;
}

fn freelist_alloc(ctx: rawptr u8, size: usize, align: usize) -> rawptr u8 {
    unsafe "ctx points at the live FreeList whose [next, end) window and address-ordered free chain are reserved to this arena alone; every carved block stays inside [next, end) and is >= header-sized, and a split remainder (kept only when >= MIN_SPLIT) is written into the block's own tail" {
        let st: FreeList = ptr_read(cast_ptr[FreeList](ctx));
        let need: usize = block_span(size, align);
        let mut prev: rawptr u8 = ptr_null[u8]();
        let mut cur: rawptr u8 = st.head;
        while !is_null(cur) {
            let blk: FreeBlock = ptr_read(cast_ptr[FreeBlock](cur));
            if blk.size >= need {
                if blk.size - need >= 32usize {
                    let rem: rawptr u8 = addr_to_ptr[u8](ptr_to_addr(cur) + need);
                    ptr_write(cast_ptr[FreeBlock](rem), FreeBlock { next: blk.next, size: blk.size - need });
                    if is_null(prev) {
                        ptr_write(cast_ptr[FreeList](ctx), FreeList { next: st.next, end: st.end, head: rem });
                    } else {
                        let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
                        ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: rem, size: pblk.size });
                    }
                } else {
                    if is_null(prev) {
                        ptr_write(cast_ptr[FreeList](ctx), FreeList { next: st.next, end: st.end, head: blk.next });
                    } else {
                        let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
                        ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: blk.next, size: pblk.size });
                    }
                }
                return cur;
            }
            prev = cur;
            cur = blk.next;
        }
        let aligned: usize = (st.next + align - 1usize) / align * align;
        if aligned + need > st.end {
            return ptr_null[u8]();
        }
        ptr_write(cast_ptr[FreeList](ctx), FreeList { next: aligned + need, end: st.end, head: st.head });
        return addr_to_ptr[u8](aligned);
    }
}

fn freelist_free(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) -> unit {
    unsafe "the free list is kept ADDRESS-ORDERED; the freed block's own storage (>= header-sized, guaranteed by block_span in alloc) holds its FreeBlock header {next, size}, and a merge joins two blocks ONLY when their byte spans are exactly adjacent (addr + size == neighbour addr), so a merge can never overlap live memory nor bridge a gap" {
        let st: FreeList = ptr_read(cast_ptr[FreeList](ctx));
        let mut cap: usize = block_span(size, align);
        let a: usize = ptr_to_addr(ptr);
        let mut prev: rawptr u8 = ptr_null[u8]();
        let mut cur: rawptr u8 = st.head;
        while !is_null(cur) && ptr_to_addr(cur) < a {
            let cblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](cur));
            prev = cur;
            cur = cblk.next;
        }
        let mut link: rawptr u8 = cur;
        if !is_null(cur) {
            let nblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](cur));
            if a + cap == ptr_to_addr(cur) {
                cap = cap + nblk.size;
                link = nblk.next;
            }
        }
        if !is_null(prev) {
            let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
            if ptr_to_addr(prev) + pblk.size == a {
                ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: link, size: pblk.size + cap });
                return;
            }
        }
        ptr_write(cast_ptr[FreeBlock](ptr), FreeBlock { next: link, size: cap });
        if is_null(prev) {
            ptr_write(cast_ptr[FreeList](ctx), FreeList { next: st.next, end: st.end, head: ptr });
        } else {
            let pblk: FreeBlock = ptr_read(cast_ptr[FreeBlock](prev));
            ptr_write(cast_ptr[FreeBlock](prev), FreeBlock { next: ptr, size: pblk.size });
        }
    }
}

static FREELIST_VT: AllocVtable = AllocVtable { alloc: freelist_alloc, free: freelist_free };

fn mk_alloc(state: write FreeList) -> Alloc {
    unsafe "ctx (the caller's FreeList local) and vt (the static FREELIST_VT) outlive every String this handle serves; the caller keeps the window and FREELIST_VT alive for the whole run and drops every String before the window dies" {
        return Alloc { ctx: cast_ptr[u8](addr_of_mut(state.*)), vt: addr_of(FREELIST_VT) };
    }
}

fn write_str(fd: i32, s: read String) -> IoResult {
    return write_all(fd, as_bytes(as_str(read s.*)));
}
fn main() alloc -> i64 {
    let mut st: FreeList = with_window(16777216usize, 1048576usize);
    let a: Alloc = mk_alloc(write st);
    let mut s: String = string_new(a);
    append(write s, "Hi, ");
    append(write s, "Candor");
    push(write s, 33);
    match write_str(stdout(), read s) {
        IoResult::Ok(c) => { return conv i64 c; },
        IoResult::Err(e) => { return 0i64 - 2i64; },
    }
}
"#;


// ===========================================================================
// File-I/O layer: read a file into a native String, write a String out.
// ===========================================================================

const ROUND_TRIP_MAIN: &str = r#"
fn main() alloc -> i64 {
    let mut st: FreeList = with_window(16777216usize, 1048576usize);
    let a: Alloc = mk_alloc(write st);
    let s: String = match read_to_string(read a, stdin()) {
        StrIoResult::Ok(v) => v,
        StrIoResult::Err(e) => { return 0i64 - 1i64; },
    };
    match write_str(stdout(), read s) {
        IoResult::Ok(c) => { return conv i64 c; },
        IoResult::Err(e) => { return 0i64 - 2i64; },
    }
}
"#;
const READ_FILE_MAIN: &str = r#"
fn main() alloc -> i64 {
    let mut st: FreeList = with_window(16777216usize, 1048576usize);
    let a: Alloc = mk_alloc(write st);
    let name: [7]u8 = [114u8, 116u8, 46u8, 116u8, 120u8, 116u8, 0u8];
    let s: String = match read_file(read a, slice_of(name)) {
        StrIoResult::Ok(v) => v,
        StrIoResult::Err(e) => { return 0i64 - 1i64; },
    };
    match write_str(stdout(), read s) {
        IoResult::Ok(c) => { return conv i64 c; },
        IoResult::Err(e) => { return 0i64 - 2i64; },
    }
}
"#;
const ERROR_MAIN: &str = r#"
fn main() alloc -> i64 {
    let mut st: FreeList = with_window(16777216usize, 1048576usize);
    let a: Alloc = mk_alloc(write st);
    let name: [9]u8 = [110u8, 111u8, 112u8, 101u8, 46u8, 116u8, 120u8, 116u8, 0u8];
    match read_file(read a, slice_of(name)) {
        StrIoResult::Ok(v) => { return 1i64; },
        StrIoResult::Err(e) => { match e { IoError::Errno(n) => { return conv i64 n; }, } },
    }
}
"#;

// The read path uses `str_from_unchecked` to append buf[0..n], a tree-walker-only
// intrinsic, so these run on the tree-walker (via the real shims). See FILE_API.

// ---- round-trip: stdin -> read_to_string -> String -> write_str -> stdout ----
#[test]
fn read_to_string_round_trip_byte_equal_tree() {
    let _g = lock();
    // multi-line, multi-byte (UTF-8) content, smaller than the 64-byte buffer.
    let content = "hi\ncand\u{00f6}r\n\u{03c0}\n".to_string();
    let src = file_probe(ROUND_TRIP_MAIN);
    foreign_io::reset();
    foreign_io::register_std_io();
    foreign_io::set_stdin(content.as_bytes());
    let ret = run_ok(&src);
    let out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();
    assert_eq!(out, content.as_bytes(), "String round-tripped byte-for-byte");
    assert_eq!(ret, content.len() as i64, "write_str returns the byte count");
}

// ---- multi-fill: content > the 64-byte buffer forces the loop + String growth --
#[test]
fn read_to_string_multi_fill_grows_across_reads_tree() {
    let _g = lock();
    // ~300 bytes of multi-byte content: many buffer-fills (n == 64) then a short
    // final read (n < 64), so read_to_string loops and the String grows repeatedly.
    let unit = "line \u{00e9}\u{20ac}\u{1f600} 0123456789\n"; // multi-byte, straddles boundaries
    let content = unit.repeat(12);
    assert!(content.len() > 64, "must exceed the internal buffer");
    let src = file_probe(ROUND_TRIP_MAIN);
    foreign_io::reset();
    foreign_io::register_std_io();
    foreign_io::set_stdin(content.as_bytes());
    let ret = run_ok(&src);
    let out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();
    assert_eq!(out, content.as_bytes(), "multi-fill reassembled exactly");
    assert_eq!(ret, content.len() as i64);
}

// ---- read_file: real open + read + close, then write it back out --------------
#[test]
fn read_file_opens_reads_closes_tree() {
    let _g = lock();
    let content = "file body\n\u{00e9}\u{20ac}\u{1f600}\nlast\n".to_string();
    let tdir = std::env::temp_dir().join(format!("candor-io-file-{}", std::process::id()));
    std::fs::create_dir_all(&tdir).unwrap();
    std::fs::write(tdir.join("rt.txt"), content.as_bytes()).unwrap();
    let src = file_probe(READ_FILE_MAIN);
    foreign_io::reset();
    foreign_io::set_root(&tdir);
    foreign_io::register_std_io();
    let ret = run_ok(&src);
    let out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();
    let _ = std::fs::remove_dir_all(&tdir);
    assert_eq!(out, content.as_bytes(), "read_file returned the file bytes exactly");
    assert_eq!(ret, content.len() as i64);
}

// ---- error path: read_file of a missing path is the Err arm (via `?`) ---------
#[test]
fn read_file_missing_path_is_err_tree() {
    let _g = lock();
    let empty = std::env::temp_dir().join(format!("candor-io-file-empty-{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    let src = file_probe(ERROR_MAIN);
    foreign_io::reset();
    foreign_io::set_root(&empty);
    foreign_io::register_std_io();
    let ret = run_ok(&src);
    foreign_io::unregister_std_io();
    assert_eq!(ret, -1, "open of a missing file -> `?` propagates IoError::Errno(-1)");
}

// ---- write_str is all-native: proven on the tree-walker AND the MIR engine -----
#[test]
fn write_str_writes_string_bytes_both_engines() {
    let _g = lock();
    // tree-walker
    foreign_io::reset();
    foreign_io::register_std_io();
    let src = format!("{}{}", module_prefix(), WRITE_STR_NATIVE);
    let tw = run_ok(&src);
    let tw_out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();
    // MIR engine (write_str lowers: as_bytes(as_str(..)) + write_all, no str-view ctor)
    foreign_io::reset();
    foreign_io::register_std_io();
    let mir = run_ok_mir(&src);
    let mir_out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();
    assert_eq!(tw, 11, "wrote 11 bytes");
    assert_eq!(mir, 11);
    assert_eq!(tw_out, b"Hi, Candor!");
    assert_eq!(mir_out, b"Hi, Candor!");
}

// ---- the read path now lowers to MIR: `str_from_unchecked` is a native retype ---
// Appending buf[0..n] as a bounded str-view needs `str_from_unchecked` (the
// `[u8] -> str` reinterpret). That intrinsic now lowers on the MIR/native builtin
// set (`mir::build::is_builtin`) as a pure 16-byte fat-pointer retype (the mirror
// of `as_bytes`), so the whole read path reaches the MIR engine. This proves the
// round-trip byte-exact on BOTH the tree-walker AND the MIR engine through the
// same real shims; the native (Cranelift + LLVM) gates live in tests/aot.rs and
// tests/llvm.rs.
#[test]
fn read_to_string_round_trip_byte_equal_tree_and_mir() {
    let _g = lock();
    let content = "hi\ncand\u{00f6}r\n\u{03c0}\n".to_string();
    let src = file_probe(ROUND_TRIP_MAIN);

    // tree-walker
    foreign_io::reset();
    foreign_io::register_std_io();
    foreign_io::set_stdin(content.as_bytes());
    let tw_ret = run_ok(&src);
    let tw_out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();

    // MIR engine (str_from_unchecked lowers as a CopyVal retype; String is native)
    foreign_io::reset();
    foreign_io::register_std_io();
    foreign_io::set_stdin(content.as_bytes());
    let mir_ret = run_ok_mir(&src);
    let mir_out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();

    assert_eq!(tw_out, content.as_bytes(), "tree round-tripped byte-for-byte");
    assert_eq!(mir_out, content.as_bytes(), "MIR round-tripped byte-for-byte");
    assert_eq!(tw_ret, content.len() as i64);
    assert_eq!(mir_ret, tw_ret, "MIR byte count matches the tree oracle");
}

// ---------------------------------------------------------------------------
// Line-oriented read path: `read_lines` = `read_file`? -> `split_lines`, byte-
// scanning the owned `String` into a `Vec[String]`. The read path is native
// (`str_from_unchecked` lowers), so this runs on the tree-walker AND the MIR
// engine through the same real shims; the Cranelift + LLVM native gates live in
// tests/aot.rs (`gate_aot_native_read_lines`) and tests/llvm.rs
// (`gate_llvm_native_read_lines`). The self-contained fixture reads "rt.txt" into
// lines and writes each line back newline-terminated, so the reconstruction
// equals the newline-terminated file body byte-for-byte.
// ---------------------------------------------------------------------------
fn read_lines_fixture() -> String {
    let path = format!("{}/tests/fixtures/std_io_readpath/read_lines_native.cnr", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).expect("read read-lines fixture")
}

#[test]
fn read_lines_splits_file_into_line_vec_tree_and_mir() {
    let _g = lock();
    let src = read_lines_fixture();
    // Multi-line, multibyte, > 64 bytes so read_file loops and grows the String;
    // four newline-terminated lines -> the per-line reconstruction reproduces the
    // body exactly.
    let content = "hi\ncand\u{00f6}r\n\u{03c0}\nlong line to exceed sixty four bytes across many reads xyz\n";
    let tdir = std::env::temp_dir().join(format!("candor-io-lines-{}", std::process::id()));
    std::fs::create_dir_all(&tdir).unwrap();
    std::fs::write(tdir.join("rt.txt"), content.as_bytes()).unwrap();

    // tree-walker
    foreign_io::reset();
    foreign_io::set_root(&tdir);
    foreign_io::register_std_io();
    let tw = run_ok(&src);
    let tw_out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();

    // MIR engine (split_lines + read_file both lower: native String/Vec + str_from_unchecked)
    foreign_io::reset();
    foreign_io::set_root(&tdir);
    foreign_io::register_std_io();
    let mir = run_ok_mir(&src);
    let mir_out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();
    let _ = std::fs::remove_dir_all(&tdir);

    assert_eq!(tw, 4, "read_lines split the file into four lines");
    assert_eq!(mir, 4, "MIR line count matches the tree oracle");
    assert_eq!(tw_out, content.as_bytes(), "tree: lines reassembled newline-terminated");
    assert_eq!(mir_out, content.as_bytes(), "MIR: lines reassembled newline-terminated");
}

// ===========================================================================
// Buffered I/O layer: `BufReader.read_line` (streaming, owned per-line Strings,
// cross-refill line assembly) and `BufWriter` (accumulate + threshold flush).
// The `buf_io_native.cnr` fixture reads "rt.txt" line by line through
// `read_line` and writes each line back newline-terminated through a buffered
// `BufWriter` (auto-flushing past 4096 bytes), so stdout equals the file body
// re-split by `split_lines`' convention. The read path is native
// (`str_from_unchecked` lowers), so this runs on the tree-walker AND the MIR
// engine through the same real shims; the Cranelift + LLVM native gates live in
// tests/aot.rs and tests/llvm.rs.
// ===========================================================================
fn buf_io_fixture() -> String {
    let path = format!("{}/tests/fixtures/std_io_readpath/buf_io_native.cnr", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).expect("read buf-io fixture")
}

/// Replicate `split_lines`' convention, then rebuild the newline-terminated body
/// the fixture writes back: a trailing '\n' yields no final empty line; interior
/// empty lines are kept; the empty input yields zero lines.
fn expected_reconstruction(body: &[u8]) -> (Vec<u8>, i64) {
    let mut lines: Vec<&[u8]> = Vec::new();
    let mut start = 0usize;
    for (i, &b) in body.iter().enumerate() {
        if b == 10 {
            lines.push(&body[start..i]);
            start = i + 1;
        }
    }
    if start < body.len() {
        lines.push(&body[start..]);
    }
    let mut out = Vec::new();
    for l in &lines {
        out.extend_from_slice(l);
        out.push(10);
    }
    (out, lines.len() as i64)
}

fn run_buf_io_both(tag: &str, content: &[u8]) {
    let src = buf_io_fixture();
    let tdir = std::env::temp_dir().join(format!("candor-bufio-{}-{}", tag, std::process::id()));
    std::fs::create_dir_all(&tdir).unwrap();
    std::fs::write(tdir.join("rt.txt"), content).unwrap();
    let (expected, count) = expected_reconstruction(content);

    // tree-walker
    foreign_io::reset();
    foreign_io::set_root(&tdir);
    foreign_io::register_std_io();
    let tw = run_ok(&src);
    let tw_out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();

    // MIR engine (BufReader/BufWriter lower: native String + str_from_unchecked)
    foreign_io::reset();
    foreign_io::set_root(&tdir);
    foreign_io::register_std_io();
    let mir = run_ok_mir(&src);
    let mir_out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();
    let _ = std::fs::remove_dir_all(&tdir);

    assert_eq!(tw, count, "{tag}: tree line count");
    assert_eq!(mir, count, "{tag}: MIR line count matches the tree oracle");
    assert_eq!(tw_out, expected, "{tag}: tree reassembled newline-terminated");
    assert_eq!(mir_out, expected, "{tag}: MIR reassembled newline-terminated");
}

// ---- streaming read_line: file > buffer, lines spanning refills, auto-flush ----
#[test]
fn buf_reader_read_line_streams_large_file_tree_and_mir() {
    let _g = lock();
    // > 4096 bytes so the BufWriter auto-flush fires mid-stream; a 200-byte line
    // spans many 64-byte refills; multibyte chars straddle refill boundaries; an
    // interior empty line is preserved; trailing '\n' yields no extra empty line.
    let mut content = String::new();
    content.push_str("hi\ncand\u{00f6}r\n\u{03c0}\n\n");
    content.push_str(&"x".repeat(200));
    content.push('\n');
    for i in 0..120 {
        content.push_str(&format!("line {i} \u{00e9}\u{20ac}\u{1f600} 0123456789 padding-xyz\n"));
    }
    assert!(content.len() > 4096, "must cross the BufWriter flush threshold");
    run_buf_io_both("large", content.as_bytes());
}

// ---- edge cases: empty file (-> None immediately), no trailing newline, interior empty ----
#[test]
fn buf_reader_read_line_edge_cases_tree_and_mir() {
    let _g = lock();
    run_buf_io_both("empty", b""); // read_line -> None immediately, zero lines
    run_buf_io_both("no_trailing_nl", b"alpha\nbeta\ngamma"); // final line without '\n' still returned
    run_buf_io_both("interior_empty", b"a\n\nb\n"); // interior empty line kept, trailing '\n' drops final empty
    run_buf_io_both("single_no_nl", b"solo"); // single line, no newline
}

// ---- BufWriter round-trip: buffered writes crossing the flush threshold --------
// Buffers 300 * 20 bytes + a 4-byte tail (6004 bytes) through `bw_write_str`,
// auto-flushing each time the accumulator passes 4096, then a final `bw_flush`.
// The captured stdout must equal the concatenation of every string written,
// byte-for-byte, on BOTH engines (BufWriter is all-native: append + write_all).
const BUFWRITER_DEFS: &str = r#"
struct BufWriter { fd: i32, al: Alloc, buf: String }
fn buf_writer(a: Alloc, fd: i32) alloc -> BufWriter {
    return BufWriter { fd: fd, al: a, buf: string_new(a) };
}
fn bw_flush(w: write BufWriter) alloc -> IoResult {
    let n: usize = write_all(w.fd, as_bytes(as_str(read w.buf)))?;
    w.*.buf = string_new(w.al);
    return IoResult::Ok(n);
}
fn bw_write_str(w: write BufWriter, s: str) alloc -> IoResult {
    append(write w.*.buf, s);
    if len(as_bytes(as_str(read w.buf))) >= 4096usize {
        let n: usize = bw_flush(w)?;
    }
    return IoResult::Ok(0usize);
}
fn main() alloc -> i64 {
    let mut st: FreeList = with_window(16777216usize, 1048576usize);
    let a: Alloc = mk_alloc(write st);
    let mut bw: BufWriter = buf_writer(a, stdout());
    let mut i: i64 = 0i64;
    loop {
        if i >= 300i64 { break; }
        match bw_write_str(write bw, "candor-buffered-0123") {
            IoResult::Ok(c) => { },
            IoResult::Err(e) => { return 0i64 - 1i64; },
        }
        i = i + 1i64;
    }
    match bw_write_str(write bw, "TAIL") {
        IoResult::Ok(c) => { },
        IoResult::Err(e) => { return 0i64 - 2i64; },
    }
    match bw_flush(write bw) {
        IoResult::Ok(c) => { },
        IoResult::Err(e) => { return 0i64 - 3i64; },
    }
    return i + 1i64;
}
"#;

#[test]
fn buf_writer_round_trip_crosses_flush_threshold_both_engines() {
    let _g = lock();
    let src = format!("{}{}{}", module_prefix(), FILE_API, BUFWRITER_DEFS);
    let mut expected: Vec<u8> = Vec::new();
    for _ in 0..300 {
        expected.extend_from_slice(b"candor-buffered-0123");
    }
    expected.extend_from_slice(b"TAIL");

    foreign_io::reset();
    foreign_io::register_std_io();
    let tw = run_ok(&src);
    let tw_out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();

    foreign_io::reset();
    foreign_io::register_std_io();
    let mir = run_ok_mir(&src);
    let mir_out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();

    assert_eq!(tw, 301, "301 buffered writes");
    assert_eq!(mir, 301);
    assert_eq!(tw_out, expected, "tree: buffered writes flushed in order, byte-exact");
    assert_eq!(mir_out, expected, "MIR: buffered writes flushed in order, byte-exact");
    assert_eq!(expected.len(), 6004, "300*20 + 4 bytes");
}
