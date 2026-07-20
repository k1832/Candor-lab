//! Host-backed std I/O shims for the boundary registry (design 0013 std I/O over
//! the 0011 foreign boundary). These are the Rust stand-ins the two engines call
//! when a Candor program invokes the `sys_read`/`sys_write`/`sys_open`/`sys_close`
//! externs of the std/io boundary module. They perform **real host I/O** —
//! `std::fs` for files, a captured in-process buffer for stdout/stderr — so the
//! tree-walker and MIR engines run genuine, deterministic I/O against a fixture
//! file and an assertable output buffer, before the native backend (0010) can
//! emit real libc calls.
//!
//! Harness scope only, exactly like the rest of the shim registry: this ships no
//! C and is not compiled into any Candor binary. The AOT path resolves the same
//! externs to real libc (a 0010 forward dependency; not yet wired).
//!
//! ## The fd model
//! - `0`/`1`/`2` are stdin/stdout/stderr. stdout/stderr writes append to captured
//!   buffers (`take_stdout`/`take_stderr`); stdin reads drain a settable buffer.
//! - `open` returns an fd `>= 3` backed by a real `std::fs::File`, resolved
//!   against a test-set root directory (`set_root`) so a Candor program can open a
//!   fixed logical name (`"input.txt"`) deterministically.
//! - Errors return `-1` (the POSIX convention), which the safe wrapper turns into
//!   the `Err` arm of its result-shaped return.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::interp::mem::Mem;

/// Linux `open(2)` flag bits the shim honors (design 0011 §1: `flags` is `i32`).
const O_WRONLY: i128 = 0x1;
const O_RDWR: i128 = 0x2;
const O_CREAT: i128 = 0x40;
const O_TRUNC: i128 = 0x200;
const O_APPEND: i128 = 0x400;

/// A backing handle for an open fd (>= 3): a real file or a connected TCP socket.
/// Both `std::fs::File` and `std::net::TcpStream` implement `Read`/`Write`, so the
/// shared `sys_read`/`sys_write`/`sys_close` shims dispatch through this enum
/// uniformly — a socket fd lives in the same fd table as a file fd (design 0013
/// std net over the 0011 boundary: sockets ARE file descriptors).
enum Handle {
    File(File),
    Tcp(TcpStream),
    /// A listening socket (design 0013 std net, server side): it lives in the same
    /// fd table as file/socket fds so `sys_close` drops it uniformly, but `sys_read`/
    /// `sys_write` on it are a usage error (a listener carries no byte stream) — only
    /// `sys_tcp_accept`/`sys_tcp_port` consume it.
    Listener(TcpListener),
}

impl Handle {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Handle::File(f) => f.read(buf),
            Handle::Tcp(s) => s.read(buf),
            Handle::Listener(_) => Err(std::io::ErrorKind::InvalidInput.into()),
        }
    }
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        match self {
            Handle::File(f) => f.write(data),
            Handle::Tcp(s) => s.write(data),
            Handle::Listener(_) => Err(std::io::ErrorKind::InvalidInput.into()),
        }
    }
}

struct IoState {
    root: PathBuf,
    files: HashMap<i32, Handle>,
    next_fd: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    stdin: Vec<u8>,
    stdin_pos: usize,
    last_listen_port: Option<u16>,
}

impl IoState {
    fn new() -> Self {
        IoState {
            root: PathBuf::from("."),
            files: HashMap::new(),
            next_fd: 3,
            stdout: Vec::new(),
            stderr: Vec::new(),
            stdin: Vec::new(),
            stdin_pos: 0,
            last_listen_port: None,
        }
    }
}

fn state() -> &'static Mutex<IoState> {
    static S: OnceLock<Mutex<IoState>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(IoState::new()))
}

// ---- test/harness controls -------------------------------------------------

/// Reset all captured I/O state (call at the start of a demonstrator run).
pub fn reset() {
    let mut s = state().lock().unwrap();
    *s = IoState::new();
}

/// Point `open` at a directory: a relative Candor path resolves against `root`.
pub fn set_root(root: impl Into<PathBuf>) {
    state().lock().unwrap().root = root.into();
}

/// Preload the bytes a `sys_read` on fd 0 (stdin) will drain.
pub fn set_stdin(bytes: &[u8]) {
    let mut s = state().lock().unwrap();
    s.stdin = bytes.to_vec();
    s.stdin_pos = 0;
}

/// Take everything written to fd 1 (stdout) so far.
pub fn take_stdout() -> Vec<u8> {
    std::mem::take(&mut state().lock().unwrap().stdout)
}

/// Take everything written to fd 2 (stderr) so far.
pub fn take_stderr() -> Vec<u8> {
    std::mem::take(&mut state().lock().unwrap().stderr)
}

/// The port the most recent `sys_tcp_listen` bound (via `getsockname`), or `None`
/// if none has bound yet. The harness reads this to learn a server's EPHEMERAL port
/// (`listen(0)`) race-free — before it connects — without depending on the Candor
/// program's own stdout timing.
pub fn last_listen_port() -> Option<u16> {
    state().lock().unwrap().last_listen_port
}

// ---- registration ----------------------------------------------------------

/// Register the four std/io shims into the foreign registry. Idempotent.
pub fn register_std_io() {
    crate::foreign::register("sys_open", shim_open);
    crate::foreign::register("sys_close", shim_close);
    crate::foreign::register("sys_read", shim_read);
    crate::foreign::register("sys_write", shim_write);
    crate::foreign::register("sys_listdir", shim_listdir);
}

/// Remove the std/io shims, restoring `no_foreign_runtime` for their symbols.
pub fn unregister_std_io() {
    for sym in ["sys_open", "sys_close", "sys_read", "sys_write", "sys_listdir"] {
        crate::foreign::unregister(sym);
    }
}

/// Register the std/net shim (`sys_tcp_connect`) into the foreign registry. The
/// socket fd it returns lives in the SAME fd table as file fds, so the std/io
/// `sys_read`/`sys_write`/`sys_close` shims (registered by `register_std_io`)
/// operate on the socket — register both for a network client. Idempotent.
pub fn register_std_net() {
    crate::foreign::register("sys_tcp_connect", shim_tcp_connect);
    crate::foreign::register("sys_tcp_listen", shim_tcp_listen);
    crate::foreign::register("sys_tcp_accept", shim_tcp_accept);
    crate::foreign::register("sys_tcp_port", shim_tcp_port);
}

/// Remove the std/net shim, restoring `no_foreign_runtime` for its symbol.
pub fn unregister_std_net() {
    for sym in ["sys_tcp_connect", "sys_tcp_listen", "sys_tcp_accept", "sys_tcp_port"] {
        crate::foreign::unregister(sym);
    }
}

/// Register the PRODUCTION std I/O + net shims for the `candor run` CLI. Unlike
/// [`register_std_io`]/[`register_std_net`] (the harness path, which buffers
/// stdout/stderr and resolves `open`/`listdir` against a test-set `root`), these
/// do REAL host I/O: fd 1/2 write to the process's stdout/stderr (flushed), fd 0
/// reads its stdin, and `open` resolves the path exactly as given (cwd-relative or
/// absolute). fd >= 3 file/socket read/write/close, `sys_listdir` (over the default
/// `.`-rooted `read_dir`, i.e. as-given), and `sys_tcp_connect` reuse the SAME real
/// implementations as the harness path. Idempotent; intended for the CLI process
/// only (the test harness registers its own buffer/root shims). Do NOT mix with the
/// harness registration in one process.
pub fn register_std_io_production() {
    crate::foreign::register("sys_open", shim_open_production);
    crate::foreign::register("sys_close", shim_close);
    crate::foreign::register("sys_read", shim_read_production);
    crate::foreign::register("sys_write", shim_write_production);
    crate::foreign::register("sys_listdir", shim_listdir);
    crate::foreign::register("sys_tcp_connect", shim_tcp_connect);
    crate::foreign::register("sys_tcp_listen", shim_tcp_listen);
    crate::foreign::register("sys_tcp_accept", shim_tcp_accept);
    crate::foreign::register("sys_tcp_port", shim_tcp_port);
}

// ---- test-only shim overrides (fault-injection) ----------------------------

/// Override `sys_write` with a shim that reports it wrote exactly `n` bytes
/// (a short write when `0 <= n < count`, or an error when `n < 0`), so the
/// wrapper's short-write / error handling can be exercised deterministically.
pub fn register_shim_override_write_short(n: i128) {
    crate::foreign::register("sys_write", move |_args, _mem| n);
}

/// Override `sys_read` with a shim that reports an error (-1).
pub fn register_shim_override_read_error() {
    crate::foreign::register("sys_read", |_args, _mem| -1);
}

// ---- the shims -------------------------------------------------------------

/// Read a NUL-terminated byte string out of Candor flat memory at `addr`.
fn read_cstr(mem: &mut Mem, addr: u64) -> Vec<u8> {
    let mut out = Vec::new();
    let mut a = addr;
    loop {
        match mem.read(a, 1, false) {
            Ok(b) if !b.is_empty() && b[0] != 0 => {
                out.push(b[0]);
                a += 1;
            }
            _ => break,
        }
    }
    out
}

/// Build an `OpenOptions` from the POSIX `open(2)` flag bits (shared by the test
/// and production `open` shims — only the path resolution differs between them).
fn open_options_from_flags(flags: i128) -> OpenOptions {
    let mut oo = OpenOptions::new();
    let writing = flags & O_WRONLY != 0 || flags & O_RDWR != 0;
    if flags & O_RDWR != 0 {
        oo.read(true).write(true);
    } else if writing {
        oo.write(true);
    } else {
        oo.read(true);
    }
    if flags & O_CREAT != 0 {
        oo.create(true);
    }
    if flags & O_TRUNC != 0 {
        oo.truncate(true);
    }
    if flags & O_APPEND != 0 {
        oo.append(true);
    }
    oo
}

/// Open `path` with `oo`, insert the real file into the shared fd table, and
/// return its fd (>= 3); `-1` on failure. Shared by the test and production shims.
fn insert_open(s: &mut IoState, oo: OpenOptions, path: &Path) -> i128 {
    match oo.open(path) {
        Ok(f) => {
            let fd = s.next_fd;
            s.next_fd += 1;
            s.files.insert(fd, Handle::File(f));
            fd as i128
        }
        Err(_) => -1,
    }
}

/// `sys_open(path: rawptr u8, flags: i32, mode: i32) -> i32` — real `std::fs`,
/// path resolved against the test `root` (harness path).
fn shim_open(args: &[i128], mem: &mut Mem) -> i128 {
    let path_addr = args[0] as u64;
    let flags = args[1];
    let name = read_cstr(mem, path_addr);
    let rel = match String::from_utf8(name) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let mut s = state().lock().unwrap();
    let full = s.root.join(rel);
    let oo = open_options_from_flags(flags);
    insert_open(&mut s, oo, &full)
}

/// Production `sys_open`: resolve the path bytes exactly as given (cwd-relative or
/// absolute), NOT joined to the test root. Otherwise identical to [`shim_open`].
fn shim_open_production(args: &[i128], mem: &mut Mem) -> i128 {
    let path_addr = args[0] as u64;
    let flags = args[1];
    let name = read_cstr(mem, path_addr);
    let path = match String::from_utf8(name) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let oo = open_options_from_flags(flags);
    let mut s = state().lock().unwrap();
    insert_open(&mut s, oo, Path::new(&path))
}

/// Production `sys_read`: fd 0 reads the process's real stdin; fd >= 3 reads the
/// real file/socket in the shared fd table (reused from the harness path).
fn shim_read_production(args: &[i128], mem: &mut Mem) -> i128 {
    let fd = args[0] as i32;
    let buf = args[1] as u64;
    let count = args[2] as usize;
    let mut tmp = vec![0u8; count];
    let n = if fd == 0 {
        // stdin read must not run under the IoState lock (it may block).
        match std::io::stdin().read(&mut tmp) {
            Ok(n) => n,
            Err(_) => return -1,
        }
    } else {
        let mut s = state().lock().unwrap();
        match s.files.get_mut(&fd) {
            Some(f) => match f.read(&mut tmp) {
                Ok(n) => n,
                Err(_) => return -1,
            },
            None => return -1,
        }
    };
    if mem.write(buf, &tmp[..n]).is_err() {
        return -1;
    }
    n as i128
}

/// Production `sys_write`: fd 1/2 write to the process's real stdout/stderr
/// (flushed, so a write-then-exit program never loses output); fd >= 3 writes the
/// real file/socket in the shared fd table (reused from the harness path).
fn shim_write_production(args: &[i128], mem: &mut Mem) -> i128 {
    let fd = args[0] as i32;
    let buf = args[1] as u64;
    let count = args[2] as usize;
    let data = match mem.read(buf, count as u64, false) {
        Ok(d) => d,
        Err(_) => return -1,
    };
    match fd {
        1 => write_all_flush(&mut std::io::stdout(), &data),
        2 => write_all_flush(&mut std::io::stderr(), &data),
        _ => {
            let mut s = state().lock().unwrap();
            match s.files.get_mut(&fd) {
                Some(f) => match f.write(&data) {
                    Ok(n) => n as i128,
                    Err(_) => -1,
                },
                None => -1,
            }
        }
    }
}

/// Write `data` in full and flush, returning the byte count (`write_all` loops over
/// short writes) or `-1` on error — the POSIX-shaped result the wrapper expects.
fn write_all_flush(w: &mut impl Write, data: &[u8]) -> i128 {
    match w.write_all(data).and_then(|()| w.flush()) {
        Ok(()) => data.len() as i128,
        Err(_) => -1,
    }
}

/// `sys_close(fd: i32) -> i32` — drop the file (0 on success, -1 if unknown).
fn shim_close(args: &[i128], _mem: &mut Mem) -> i128 {
    let fd = args[0] as i32;
    let mut s = state().lock().unwrap();
    if fd <= 2 {
        return 0; // std streams are not real files here
    }
    if s.files.remove(&fd).is_some() {
        0
    } else {
        -1
    }
}

/// `sys_read(fd: i32, buf: rawptr u8, count: usize) -> isize` — fill Candor memory
/// at `buf` with up to `count` bytes read from the fd; return the count (or -1).
fn shim_read(args: &[i128], mem: &mut Mem) -> i128 {
    let fd = args[0] as i32;
    let buf = args[1] as u64;
    let count = args[2] as usize;
    let mut tmp = vec![0u8; count];
    let n = {
        let mut s = state().lock().unwrap();
        if fd == 0 {
            let avail = s.stdin.len().saturating_sub(s.stdin_pos);
            let take = avail.min(count);
            let start = s.stdin_pos;
            tmp[..take].copy_from_slice(&s.stdin[start..start + take]);
            s.stdin_pos += take;
            take
        } else if let Some(f) = s.files.get_mut(&fd) {
            match f.read(&mut tmp) {
                Ok(n) => n,
                Err(_) => return -1,
            }
        } else {
            return -1;
        }
    };
    if mem.write(buf, &tmp[..n]).is_err() {
        return -1;
    }
    n as i128
}

/// `sys_write(fd: i32, buf: rawptr u8, count: usize) -> isize` — write `count`
/// bytes read from Candor memory at `buf` to the fd; return the count (or -1).
fn shim_write(args: &[i128], mem: &mut Mem) -> i128 {
    let fd = args[0] as i32;
    let buf = args[1] as u64;
    let count = args[2] as usize;
    let data = match mem.read(buf, count as u64, false) {
        Ok(d) => d,
        Err(_) => return -1,
    };
    let mut s = state().lock().unwrap();
    match fd {
        1 => {
            s.stdout.extend_from_slice(&data);
            data.len() as i128
        }
        2 => {
            s.stderr.extend_from_slice(&data);
            data.len() as i128
        }
        _ => {
            if let Some(f) = s.files.get_mut(&fd) {
                match f.write(&data) {
                    Ok(n) => n as i128,
                    Err(_) => -1,
                }
            } else {
                -1
            }
        }
    }
}

/// `sys_listdir(path: rawptr u8, dst: rawptr u8, dcap: usize) -> isize` — the
/// opendir/readdir enumerator over `std::fs::read_dir` (which already excludes
/// `.` and `..`, matching the native shim's explicit skip). Two-call sizing: it
/// returns the total bytes the entry names need (each name + a NUL separator), and
/// — only when `dcap` is large enough — writes them NUL-separated into `dst`. The
/// entry SET matches the native shim; the raw order may differ (the test sorts).
/// Returns `-1` on error (unreadable path), the POSIX convention the wrapper turns
/// into its `Err` arm.
fn shim_listdir(args: &[i128], mem: &mut Mem) -> i128 {
    use std::os::unix::ffi::OsStrExt;
    let path_addr = args[0] as u64;
    let dst_addr = args[1] as u64;
    let dcap = args[2] as u64;
    let name = read_cstr(mem, path_addr);
    let dir = {
        let s = state().lock().unwrap();
        s.root.join(std::ffi::OsStr::from_bytes(&name))
    };
    let rd = match std::fs::read_dir(&dir) {
        Ok(r) => r,
        Err(_) => return -1,
    };
    let mut entries: Vec<Vec<u8>> = Vec::new();
    for ent in rd {
        match ent {
            Ok(e) => entries.push(e.file_name().as_bytes().to_vec()),
            Err(_) => return -1,
        }
    }
    let needed: u64 = entries.iter().map(|n| n.len() as u64 + 1).sum();
    if dcap >= needed && needed > 0 {
        let mut buf = Vec::with_capacity(needed as usize);
        for n in &entries {
            buf.extend_from_slice(n);
            buf.push(0);
        }
        if mem.write(dst_addr, &buf).is_err() {
            return -1;
        }
    }
    needed as i128
}

/// `sys_tcp_connect(host: rawptr u8, host_len: usize, port: i32) -> i32` — resolve
/// `host` (a `host_len`-byte view, e.g. `127.0.0.1`) + `port` via
/// `std::net::TcpStream::connect`, insert the connected stream into the SHARED fd
/// table, and return its fd (>= 3). The returned fd is an ordinary entry alongside
/// file fds, so `sys_read`/`sys_write`/`sys_close` operate on the socket. Returns
/// -1 on any failure (host parse or a refused/unreachable connection), the
/// POSIX-shaped convention the safe wrapper turns into its `Err` arm.
fn shim_tcp_connect(args: &[i128], mem: &mut Mem) -> i128 {
    let host_addr = args[0] as u64;
    let host_len = args[1] as u64;
    let port = args[2] as u16;
    let host = match mem.read(host_addr, host_len, false) {
        Ok(b) => match String::from_utf8(b) {
            Ok(s) => s,
            Err(_) => return -1,
        },
        Err(_) => return -1,
    };
    let stream = match TcpStream::connect((host.as_str(), port)) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let mut s = state().lock().unwrap();
    let fd = s.next_fd;
    s.next_fd += 1;
    s.files.insert(fd, Handle::Tcp(stream));
    fd as i128
}


/// `sys_tcp_listen(port: i32) -> i32` — bind a fresh TCP socket to `127.0.0.1:port`
/// (`port == 0` requests an ephemeral port), start listening, insert the listener
/// into the SHARED fd table, and return its fd (>= 3). The bound port is recorded so
/// the harness can learn an ephemeral server's port race-free ([`last_listen_port`]).
/// Returns -1 on any failure (port in use, permission), the POSIX-shaped convention
/// the safe wrapper turns into its `Err` arm.
fn shim_tcp_listen(args: &[i128], _mem: &mut Mem) -> i128 {
    let port = args[0] as u16;
    let listener = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(l) => l,
        Err(_) => return -1,
    };
    let mut s = state().lock().unwrap();
    s.last_listen_port = listener.local_addr().ok().map(|a| a.port());
    let fd = s.next_fd;
    s.next_fd += 1;
    s.files.insert(fd, Handle::Listener(listener));
    fd as i128
}

/// `sys_tcp_accept(fd: i32) -> i32` — block until a client connects to the listener
/// at `fd`, then insert the connected stream into the SHARED fd table and return its
/// fd (>= 3), so `sys_read`/`sys_write`/`sys_close` drive the connection. The listener
/// is `try_clone`d so the blocking `accept` runs WITHOUT the `IoState` lock held (a
/// held lock across a blocking call would stall every other fd op). Returns -1 if
/// `fd` is not a listener or the accept fails.
fn shim_tcp_accept(args: &[i128], _mem: &mut Mem) -> i128 {
    let fd = args[0] as i32;
    let listener = {
        let s = state().lock().unwrap();
        match s.files.get(&fd) {
            Some(Handle::Listener(l)) => match l.try_clone() {
                Ok(c) => c,
                Err(_) => return -1,
            },
            _ => return -1,
        }
    };
    let stream = match listener.accept() {
        Ok((s, _)) => s,
        Err(_) => return -1,
    };
    let mut s = state().lock().unwrap();
    let cfd = s.next_fd;
    s.next_fd += 1;
    s.files.insert(cfd, Handle::Tcp(stream));
    cfd as i128
}

/// `sys_tcp_port(fd: i32) -> i32` — the actual port the listener at `fd` is bound to
/// (`getsockname`), letting a server that asked for an ephemeral port discover it.
/// Returns -1 if `fd` is not a listener or `getsockname` fails.
fn shim_tcp_port(args: &[i128], _mem: &mut Mem) -> i128 {
    let fd = args[0] as i32;
    let s = state().lock().unwrap();
    match s.files.get(&fd) {
        Some(Handle::Listener(l)) => match l.local_addr() {
            Ok(a) => a.port() as i128,
            Err(_) => -1,
        },
        _ => -1,
    }
}
