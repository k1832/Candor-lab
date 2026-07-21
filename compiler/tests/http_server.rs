//! The flagship showcase gate: an HTTP/1.0 static-file server written in Candor
//! (`tests/fixtures/http_server/main.cnr`) over the std I/O + std net boundary
//! (design 0011 foreign boundary, 0013 bytes/str + std net, server side).
//!
//! The server LISTENS on an ephemeral loopback port, announces it (traced, and one
//! decimal line to stdout), then serves exactly two connections and exits with the
//! sentinel (the count served). The harness plays a plain Rust `TcpStream` client:
//! it learns the port, sends `GET /hello.txt` and `GET /missing.txt` over two HTTP/1.0
//! connections, and asserts the exact 200 (status line + Content-Length + body) and
//! 404 responses byte-for-byte.
//!
//! Two engines are covered: the shim-backed interpreter (tree-walker + MIR, run on a
//! server thread while the harness client drives it in-process) and — when a linker
//! is present — a REAL compiled binary (the production runtime's socket shims), run as
//! a separate process that prints its port to stdout. Plus the audit + freestanding
//! assertions over the boundary module. Tests share the process-global shim registry,
//! so they serialize on `GUARD`.

use candor::foreign_io;
use candor::{run_source_real, run_source_real_mir, MirRunResult, RunResult};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

static GUARD: Mutex<()> = Mutex::new(());

fn lock() -> MutexGuard<'static, ()> {
    GUARD.lock().unwrap_or_else(|e| e.into_inner())
}

fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/http_server")
}
fn www() -> PathBuf {
    dir().join("www")
}
fn fixture() -> String {
    std::fs::read_to_string(dir().join("main.cnr")).expect("read http_server fixture")
}

/// The hello.txt the server serves and the exact HTTP responses the client expects.
const HELLO_BODY: &[u8] = b"Hello, Candor!\n";
fn expect_200() -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(b"HTTP/1.0 200 OK\r\nContent-Length: 15\r\n\r\n");
    v.extend_from_slice(HELLO_BODY);
    v
}
const EXPECT_404: &[u8] = b"HTTP/1.0 404 Not Found\r\nContent-Length: 0\r\n\r\n";

fn cc_available() -> bool {
    Command::new("cc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Open one HTTP/1.0 connection to `port`, send `GET <path>`, and drain the whole
/// response to EOF (the server closes after replying) — the client half of one
/// request/response cycle.
fn get(port: u16, path: &str) -> Vec<u8> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect to server");
    let req = format!("GET {path} HTTP/1.0\r\n\r\n");
    stream.write_all(req.as_bytes()).expect("send request");
    let mut resp = Vec::new();
    stream.read_to_end(&mut resp).expect("read response to EOF");
    resp
}

/// Drive the two-request showcase against a server listening on `port`: assert the
/// 200 (with exact body) for `/hello.txt` and the 404 for `/missing.txt`.
fn drive_two_requests(port: u16) {
    let ok = get(port, "/hello.txt");
    assert!(
        ok.starts_with(b"HTTP/1.0 200 OK\r\n"),
        "200 status line, got: {:?}",
        String::from_utf8_lossy(&ok)
    );
    assert_eq!(ok, expect_200(), "exact 200 response bytes (status + Content-Length + body)");

    let nf = get(port, "/missing.txt");
    assert!(
        nf.starts_with(b"HTTP/1.0 404 Not Found\r\n"),
        "404 status line, got: {:?}",
        String::from_utf8_lossy(&nf)
    );
    assert_eq!(nf, EXPECT_404, "exact 404 response bytes");
}

/// Wait (bounded) for the interpreter's `sys_tcp_listen` shim to record the bound
/// ephemeral port — the race-free way the in-process harness learns where to connect.
fn wait_for_port() -> u16 {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(p) = foreign_io::last_listen_port() {
            return p;
        }
        assert!(Instant::now() < deadline, "server never bound a port");
        std::thread::sleep(Duration::from_millis(2));
    }
}

// ---- 1. the module checks clean (discharge + mappability + trust) ----------
#[test]
fn http_module_checks_clean() {
    let _g = lock();
    let diags = candor::check_source_real(&fixture()).expect("parses");
    let errs: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == candor::diag::Severity::Error)
        .map(|d| d.code.clone())
        .collect();
    assert!(errs.is_empty(), "http server module should check clean, got {errs:?}");
}

// ---- 2. end-to-end on the tree-walker: server thread + in-process client ----
#[test]
fn http_server_end_to_end_tree() {
    let _g = lock();
    foreign_io::reset();
    foreign_io::set_root(www());
    foreign_io::register_std_io();
    foreign_io::register_std_net();

    let src = fixture();
    let srv = std::thread::spawn(move || match run_source_real(&src) {
        RunResult::Ok(r) => (r.ret, r.trace),
        RunResult::Fault(f) => panic!("fault: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!("check: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        RunResult::ParseError(d) => panic!("parse: {}", d.to_json()),
    });

    let port = wait_for_port();
    drive_two_requests(port);
    let (ret, trace) = srv.join().unwrap();

    foreign_io::unregister_std_net();
    foreign_io::unregister_std_io();

    assert_eq!(ret, 2, "server served exactly two connections then returned the sentinel");
    assert_eq!(trace.first().copied(), Some(port as i64), "the traced tcp_port matches the bound port");
}

// ---- 3. the same end-to-end on the MIR interpreter -------------------------
#[test]
fn http_server_end_to_end_mir() {
    let _g = lock();
    foreign_io::reset();
    foreign_io::set_root(www());
    foreign_io::register_std_io();
    foreign_io::register_std_net();

    let src = fixture();
    let srv = std::thread::spawn(move || match run_source_real_mir(&src) {
        MirRunResult::Ok(r) => (r.ret, r.trace),
        MirRunResult::Fault(f) => panic!("mir fault: {}", f.to_json()),
        MirRunResult::Unsupported(s) => panic!("mir unsupported: {s}"),
        other => panic!("mir not ok: {}", matches!(other, MirRunResult::Ok(_))),
    });

    let port = wait_for_port();
    drive_two_requests(port);
    let (ret, trace) = srv.join().unwrap();

    foreign_io::unregister_std_net();
    foreign_io::unregister_std_io();

    assert_eq!(ret, 2, "MIR: served two connections, sentinel returned");
    assert_eq!(trace.first().copied(), Some(port as i64), "MIR: traced port matches");
}

// ---- 4. the compiled binary: production socket shims, separate process -----
#[test]
fn gate_aot_native_http_server() {
    if !cc_available() {
        return; // no linker: cannot build a runnable executable
    }
    let _g = lock();
    let work = std::env::temp_dir().join(format!("candor-aot-http-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).unwrap();
    let out = work.join("httpd");
    candor::compile_path(&dir().join("main.cnr"), &out).expect("compile http server");

    // Run the server as its own process, rooted at the www dir so its `open("hello.txt")`
    // resolves; it prints its ephemeral port as the first stdout line.
    let mut child = Command::new(&out)
        .current_dir(www())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn compiled server");
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    let mut portline = String::new();
    reader.read_line(&mut portline).expect("read port line from server stdout");
    let port: u16 = portline.trim().parse().expect("parse port line");

    drive_two_requests(port);

    let status = child.wait().expect("wait for server exit");
    let _ = std::fs::remove_dir_all(&work);
    assert_eq!(status.code(), Some(2), "native server exit byte = connections served (sentinel)");
}

// ---- 5. candor audit enumerates the three new server externs + discharge ----
#[test]
fn audit_enumerates_server_externs_and_discharge() {
    let _g = lock();
    let json = candor::audit::audit_path(&dir()).expect("audit");
    for name in ["sys_tcp_listen", "sys_tcp_accept", "sys_tcp_port"] {
        assert!(json.contains(&format!("\"name\": \"{name}\"")), "{name} enumerated:\n{json}");
    }
    assert!(json.contains("TCP server listen"), "listen trust justification enumerated");
    assert!(json.contains("TCP server accept"), "accept trust justification enumerated");
    assert!(json.contains("\"undischarged_foreign_wrappers\": 0"), "every wrapper discharges foreign");
    assert!(json.contains("discharges foreign"), "wrappers shown as discharging foreign");
    assert!(!json.contains("propagates foreign"), "no wrapper leaks foreign — pub API is safe");
}

// ---- 6. freestanding gate: the server's foreign surface stays visible -------
#[test]
fn server_foreign_surface_is_visible_to_freestanding_gate() {
    let _g = lock();
    let surface = candor::audit::first_boundary_surface(&dir(), candor::manifest::Edition::E2026)
        .expect("boundary surface enumeration")
        .expect("the http server module is a boundary with a foreign extern");
    assert_eq!(
        surface.foreign_extern.as_deref(),
        Some("sys_tcp_listen"),
        "the first foreign extern the freestanding check names (source order)"
    );
}
