//! CLI real-I/O gate: `candor run` (tree-walker and `--engine=mir`) must perform
//! GENUINE host I/O through the production foreign runtime, not the harness's
//! captured-buffer / test-root shims. Unlike `tests/std_io.rs` (which drives the
//! in-process buffer shims), this spawns the BUILT `candor` binary as a separate
//! process — the only faithful test of the CLI's real stdout/stderr and filesystem
//! effects. Each case creates a real temp dir, runs a Candor program that writes a
//! file, reads it back, lists the directory, and prints a line to stdout, then
//! asserts the on-disk bytes, the recovered listing, and the process's stdout.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Remove the temp dir on scope exit so a failing assertion still cleans up.
struct TempDir(PathBuf);
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn byte_array_literal(bytes: &[u8]) -> String {
    let elems: Vec<String> = bytes.iter().map(|b| format!("{b}u8")).collect();
    format!("[{}]", elems.join(", "))
}

/// A self-contained boundary-module program (baked with the given absolute paths):
/// create+write `file`, close, reopen+read back and verify, `listdir` `dir` and
/// find the file, then write "io-ok\n" to real stdout. Returns the sentinel `42`
/// only if every step succeeded (a distinct negative code per failure otherwise).
fn program(file: &Path, dir: &Path) -> String {
    let mut file_bytes = file.to_str().unwrap().as_bytes().to_vec();
    file_bytes.push(0);
    let mut dir_bytes = dir.to_str().unwrap().as_bytes().to_vec();
    dir_bytes.push(0);
    let target = b"data.txt";
    let content = b"hello\n";
    let line = b"io-ok\n";
    format!(
        r#"boundary

extern "C" {{
    fn sys_open(path: rawptr u8, flags: i32, mode: i32) foreign -> i32
        trust "POSIX open(2): reads a NUL-terminated path, retains no pointer, thread-confined to this call" {{
            valid_nul_terminated(path),
            no_retain(path),
        }};
    fn sys_close(fd: i32) foreign -> i32
        trust "POSIX close(2): closes a descriptor; touches no Candor memory" {{
        }};
    fn sys_read(fd: i32, buf: rawptr u8, count: usize) foreign -> isize
        trust "POSIX read(2): writes at most count bytes into buf, retains no pointer, thread-confined" {{
            valid_for(buf, count),
            no_retain(buf),
            thread_confined(buf),
        }};
    fn sys_write(fd: i32, buf: rawptr u8, count: usize) foreign -> isize
        trust "POSIX write(2): reads at most count bytes from buf, retains no pointer, thread-confined" {{
            valid_for(buf, count),
            no_retain(buf),
            thread_confined(buf),
        }};
    fn sys_listdir(path: rawptr u8, dst: rawptr u8, dcap: usize) foreign -> isize
        trust "opendir/readdir(3) enumerator: reads a NUL-terminated path; writes at most dcap bytes of NUL-separated entry names into dst and returns the total bytes the names need (dcap 0 sizes only); retains no pointer, thread-confined to this call" {{
            valid_nul_terminated(path),
            no_retain(path),
            valid_for(dst, dcap),
            no_retain(dst),
            thread_confined(dst),
        }};
}}

fn do_open(path: read [u8], flags: i32) -> i32 {{
    unsafe "sys_open trust discharged: path is a live NUL-terminated [u8] view; open retains nothing (POSIX)" {{
        return sys_open(addr_of(path[0]), flags, 420i32);
    }}
}}

fn do_close(fd: i32) -> i32 {{
    unsafe "sys_close trust discharged: fd is a descriptor; close touches no Candor memory" {{
        return sys_close(fd);
    }}
}}

fn do_read(fd: i32, buf: write [u8]) -> isize {{
    unsafe "sys_read trust discharged: buf is a live [u8] view valid for len(buf) bytes; read retains nothing (POSIX), thread-confined" {{
        return sys_read(fd, addr_of_mut(buf[0]), len(buf));
    }}
}}

fn do_write(fd: i32, s: read [u8]) -> isize {{
    unsafe "sys_write trust discharged: s is a live [u8] view valid for len(s) bytes; write retains nothing (POSIX), thread-confined" {{
        return sys_write(fd, addr_of(s[0]), len(s));
    }}
}}

fn do_listdir(path: read [u8], dst: write [u8]) -> isize {{
    unsafe "sys_listdir trust discharged: path is a live NUL-terminated [u8] view; writes at most len(dst) bytes of NUL-separated names into dst, retaining nothing (POSIX opendir/readdir), thread-confined" {{
        return sys_listdir(addr_of(path[0]), addr_of_mut(dst[0]), len(dst));
    }}
}}

fn contains_entry(buf: read [u8], filled: usize, target: read [u8]) -> bool {{
    let tlen: usize = len(target);
    let mut i: usize = 0usize;
    let mut seg_start: usize = 0usize;
    loop {{
        if i >= filled {{ break; }}
        if buf[i] == 0u8 {{
            let seg_len: usize = i - seg_start;
            if seg_len == tlen {{
                let mut k: usize = 0usize;
                let mut eq: bool = true;
                loop {{
                    if k >= tlen {{ break; }}
                    if buf[seg_start + k] != target[k] {{ eq = false; break; }}
                    k = k + 1usize;
                }}
                if eq {{ return true; }}
            }}
            seg_start = i + 1usize;
        }}
        i = i + 1usize;
    }}
    return false;
}}

fn main() -> i64 {{
    let file: [{file_len}]u8 = {file_lit};
    let dir: [{dir_len}]u8 = {dir_lit};
    let target: [8]u8 = {target_lit};
    let content: [6]u8 = {content_lit};
    let line: [6]u8 = {line_lit};

    let wfd: i32 = do_open(slice_of(file), 577i32);
    if wfd < 0i32 {{ return 0i64 - 10i64; }}
    let wn: isize = do_write(wfd, slice_of(content));
    if wn < 0isize {{ return 0i64 - 11i64; }}
    if conv usize wn < 6usize {{ return 0i64 - 12i64; }}
    let c1: i32 = do_close(wfd);

    let rfd: i32 = do_open(slice_of(file), 0i32);
    if rfd < 0i32 {{ return 0i64 - 20i64; }}
    let mut rbuf: [16]u8 = [0u8; 16];
    let rn: isize = do_read(rfd, slice_of_mut(rbuf));
    if rn < 0isize {{ return 0i64 - 21i64; }}
    if conv usize rn != 6usize {{ return 0i64 - 22i64; }}
    let mut vi: usize = 0usize;
    loop {{
        if vi >= 6usize {{ break; }}
        if rbuf[vi] != content[vi] {{ return 0i64 - 23i64; }}
        vi = vi + 1usize;
    }}
    let c2: i32 = do_close(rfd);

    let mut lbuf: [512]u8 = [0u8; 512];
    let needed: isize = do_listdir(slice_of(dir), slice_of_mut(lbuf));
    if needed < 0isize {{ return 0i64 - 30i64; }}
    let filled: usize = conv usize needed;
    if !contains_entry(slice_of(lbuf), filled, slice_of(target)) {{ return 0i64 - 31i64; }}

    let sn: isize = do_write(1i32, slice_of(line));
    if sn < 0isize {{ return 0i64 - 40i64; }}

    return 42i64;
}}
"#,
        file_len = file_bytes.len(),
        file_lit = byte_array_literal(&file_bytes),
        dir_len = dir_bytes.len(),
        dir_lit = byte_array_literal(&dir_bytes),
        target_lit = byte_array_literal(target),
        content_lit = byte_array_literal(content),
        line_lit = byte_array_literal(line),
    )
}

fn run_engine(engine_flag: Option<&str>, tag: &str) {
    let base = std::env::temp_dir().join(format!("candor-cli-io-{}-{}", std::process::id(), tag));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("create temp dir");
    let guard = TempDir(base.clone());

    let data = base.join("data.txt");
    let src = base.join("prog.cnr");
    std::fs::write(&src, program(&data, &base)).expect("write program");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_candor"));
    cmd.arg("run");
    if let Some(f) = engine_flag {
        cmd.arg(f);
    }
    cmd.arg(&src);
    let out = cmd.output().expect("spawn candor run");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    // Real filesystem effect: the program actually created the file with the bytes.
    let disk = std::fs::read(&data).expect("program should have created data.txt on disk");
    assert_eq!(disk, b"hello\n", "[{tag}] on-disk file bytes; stderr={stderr}");

    // Real stdout effect: the line the program wrote to fd 1 reached the process.
    assert!(
        stdout.contains("io-ok"),
        "[{tag}] stdout should contain the written line, got: {stdout:?} stderr={stderr}"
    );
    // The sentinel return (42) is printed only when file+read-back+listdir all held.
    assert!(
        stdout.contains("42"),
        "[{tag}] program should return sentinel 42, got: {stdout:?} stderr={stderr}"
    );
    assert_eq!(out.status.code(), Some(0), "[{tag}] exit code; stderr={stderr}");

    drop(guard);
}

#[test]
fn cli_run_tree_walker_does_real_file_dir_stdout_io() {
    run_engine(None, "tree");
}

#[test]
fn cli_run_mir_does_real_file_dir_stdout_io() {
    run_engine(Some("--engine=mir"), "mir");
}
