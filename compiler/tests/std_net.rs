//! The std::net TCP-client boundary module (design 0013 std net over the 0011
//! foreign boundary): the ONE new `sys_tcp_connect` extern (a connected fd) with
//! its trust clause and the safe `tcp_connect` wrapper, plus the std::io
//! `write_all`/`read_into`/`close` wrappers REUSED on the socket fd. Sockets ARE
//! file descriptors, so the interpreter keeps socket fds and file fds in ONE shared
//! table and the existing read/write/close shims drive send/recv/close.
//!
//! The gate spins a loopback TCP server (`TcpListener` on `127.0.0.1:0`, an
//! ephemeral port) on a thread, threads the chosen port into the fixture source,
//! and runs the Candor client: connect -> write request -> read response to EOF ->
//! close, returning `bytes * 1000 + checksum` so the harness asserts byte-exact.
//!
//! Shares the process-global shim registry, so tests serialize on `GUARD`.

use candor::foreign_io;
use candor::{run_source_real, run_source_real_mir, MirRunResult, RunResult};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Mutex, MutexGuard};
use std::thread::JoinHandle;

static GUARD: Mutex<()> = Mutex::new(());

fn lock() -> MutexGuard<'static, ()> {
    GUARD.lock().unwrap_or_else(|e| e.into_inner())
}

fn dir() -> String {
    format!("{}/tests/fixtures/std_io_net", env!("CARGO_MANIFEST_DIR"))
}
fn fixture() -> String {
    std::fs::read_to_string(format!("{}/main.cnr", dir())).expect("read net fixture")
}
/// The fixture with its placeholder port literal replaced by `port` — the way the
/// harness threads the loopback server's ephemeral port into the client program.
fn fixture_with_port(port: u16) -> String {
    let src = fixture();
    assert_eq!(src.matches("12345i32").count(), 1, "exactly one port placeholder");
    src.replace("12345i32", &format!("{port}i32"))
}

fn run_ok(src: &str) -> i64 {
    match run_source_real(src) {
        RunResult::Ok(r) => r.ret,
        RunResult::Fault(f) => panic!("fault: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!("check: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        RunResult::ParseError(d) => panic!("parse: {}", d.to_json()),
    }
}
fn run_ok_mir(src: &str) -> i64 {
    match run_source_real_mir(src) {
        MirRunResult::Ok(r) => r.ret,
        MirRunResult::Fault(f) => panic!("mir fault: {}", f.to_json()),
        MirRunResult::Unsupported(s) => panic!("mir unsupported: {s}"),
        other => panic!("mir not ok: {}", matches!(other, MirRunResult::Ok(_))),
    }
}

/// The value the demonstrator returns for a given server response: it drains the
/// whole response to EOF, returning `total_bytes * 1000 + sum(bytes)`.
fn expected(resp: &[u8]) -> i64 {
    let sum: i64 = resp.iter().map(|&b| b as i64).sum();
    resp.len() as i64 * 1000 + sum
}

/// A loopback server that reads the client's request (one read, drained) then
/// writes a FIXED response and closes — deterministic regardless of timing. Reading
/// first keeps the write side open across the client's `write_all`, so the native
/// client's raw-libc write never sees a broken pipe. Accepts exactly one connection.
fn serve_fixed(response: &'static [u8]) -> (u16, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let port = listener.local_addr().unwrap().port();
    let h = std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut scratch = [0u8; 64];
            let _ = stream.read(&mut scratch);
            let _ = stream.write_all(response);
        }
    });
    (port, h)
}

/// A loopback ECHO server: reads the request and writes the same bytes back, then
/// closes. Accepts exactly one connection.
fn serve_echo() -> (u16, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let port = listener.local_addr().unwrap().port();
    let h = std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut scratch = [0u8; 64];
            if let Ok(n) = stream.read(&mut scratch) {
                let _ = stream.write_all(&scratch[..n]);
            }
        }
    });
    (port, h)
}

fn register() {
    foreign_io::reset();
    foreign_io::register_std_io();
    foreign_io::register_std_net();
}
fn unregister() {
    foreign_io::unregister_std_net();
    foreign_io::unregister_std_io();
}

// ---- 1. the module checks clean (discharge + mappability + trust) ----------
#[test]
fn net_module_checks_clean() {
    let _g = lock();
    let diags = candor::check_source_real(&fixture()).expect("parses");
    let errs: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == candor::diag::Severity::Error)
        .map(|d| d.code.clone())
        .collect();
    assert!(errs.is_empty(), "net module should check clean, got {errs:?}");
}

// ---- 2. fixed-response round trip, byte-exact on the tree-walker AND MIR -----
#[test]
fn tcp_client_fixed_response_tree_and_mir() {
    let _g = lock();
    let want = expected(b"PONG\n");

    let (port, srv) = serve_fixed(b"PONG\n");
    let src = fixture_with_port(port);
    register();
    let tw = run_ok(&src);
    unregister();
    srv.join().unwrap();

    let (port, srv) = serve_fixed(b"PONG\n");
    let src = fixture_with_port(port);
    register();
    let mir = run_ok_mir(&src);
    unregister();
    srv.join().unwrap();

    assert_eq!(tw, want, "tree: connect -> write -> read fixed response -> checksum");
    assert_eq!(mir, want, "MIR: matches the tree oracle byte-for-byte");
    assert_eq!(want, 5318, "\"PONG\\n\": 5 bytes, checksum 318");
}

// ---- 3. echo round trip: the response is the client's own request ----------
#[test]
fn tcp_client_echo_tree_and_mir() {
    let _g = lock();
    // The client always sends "PING"; an echo server returns it verbatim.
    let want = expected(b"PING");

    let (port, srv) = serve_echo();
    let src = fixture_with_port(port);
    register();
    let tw = run_ok(&src);
    unregister();
    srv.join().unwrap();

    let (port, srv) = serve_echo();
    let src = fixture_with_port(port);
    register();
    let mir = run_ok_mir(&src);
    unregister();
    srv.join().unwrap();

    assert_eq!(tw, want, "tree: echoed request round-trips");
    assert_eq!(mir, want, "MIR: matches the tree oracle");
    assert_eq!(want, 4302, "\"PING\": 4 bytes, checksum 302");
}

// ---- 4. connection refused (nothing listening) is the Err arm, not a crash ---
#[test]
fn tcp_connect_refused_is_err_tree_and_mir() {
    let _g = lock();
    // Bind then drop a listener to obtain a port that is free again, so connect(2)
    // is refused — the Err arm -> main returns -1.
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let src = fixture_with_port(port);

    register();
    let tw = run_ok(&src);
    unregister();

    register();
    let mir = run_ok_mir(&src);
    unregister();

    assert_eq!(tw, -1, "tree: refused connection -> TcpResult::Err -> -1");
    assert_eq!(mir, -1, "MIR: refused connection -> Err arm");
}

// ---- 5. an unregistered extern is the honest no_foreign_runtime fault --------
#[test]
fn no_foreign_runtime_when_unregistered() {
    let _g = lock();
    foreign_io::reset();
    foreign_io::unregister_std_net();
    foreign_io::unregister_std_io();
    let src = fixture_with_port(1u16);
    match run_source_real(&src) {
        RunResult::Fault(f) => assert_eq!(f.kind, candor::interp::FaultKind::NoForeignRuntime),
        other => panic!("expected no_foreign_runtime, got ok={}", matches!(other, RunResult::Ok(_))),
    }
}

// ---- 6. candor audit enumerates the new sys_tcp_connect extern + discharge ----
#[test]
fn audit_enumerates_tcp_connect_extern_and_discharge() {
    let _g = lock();
    let json = candor::audit::audit_path(std::path::Path::new(&dir())).expect("audit");
    assert!(json.contains("\"name\": \"sys_tcp_connect\""), "sys_tcp_connect enumerated:\n{json}");
    assert!(json.contains("TCP client connect"), "connect trust justification enumerated");
    assert!(json.contains("thread_confined"), "the trust predicate enumerated");
    assert!(json.contains("\"undischarged_foreign_wrappers\": 0"), "every wrapper discharges foreign");
    assert!(json.contains("discharges foreign"), "wrappers shown as discharging foreign");
    assert!(!json.contains("propagates foreign"), "no wrapper leaks foreign — pub API is safe");
}

// ---- 7. freestanding gate (0011 §5 / F5): the net boundary's foreign surface is
// enumerable, so the freestanding-composition check rejects a freestanding graph
// containing it. Confirms the new extern does not slip past the freestanding wall.
#[test]
fn tcp_connect_foreign_surface_is_visible_to_freestanding_gate() {
    let surface = candor::audit::first_boundary_surface(std::path::Path::new(&dir()))
        .expect("boundary surface enumeration")
        .expect("the net module is a boundary with a foreign extern");
    assert_eq!(
        surface.foreign_extern.as_deref(),
        Some("sys_tcp_connect"),
        "the first foreign extern the freestanding check names (source order)"
    );
}
