//! The PRINTLN slice (design 0011/0013): the `print_str`/`println_i64` layer
//! wiring the `fmt_i64` formatting primitive (`fixtures/std_fmt.cnr`) through the
//! std/io libc-write boundary (`fixtures/std_io/main.cnr`) to produce real,
//! captured stdout.
//!
//! The image is composed as the two lower slices compose: the std/io module
//! MINUS its demonstrator `main` (`io_prefix`, so the *actual* wrappers run — no
//! re-implementation), then the NON-GENERIC formatting core sliced verbatim out
//! of `std_fmt.cnr` (the bump allocator + `fmt_i64`, up to the `Show` interface),
//! then the print layer, then a test-supplied `main`.
//!
//! ## Engine coverage — an honest, load-bearing boundary
//! The slice runs and produces its exact bytes on the TREE-WALKER. It does NOT run
//! on the MIR engine: `write_all` takes `[u8]`, and the only bridge from a built
//! `String`'s text to `[u8]` is `as_bytes` (str -> [u8]), which the MIR lowering
//! does not implement (`is_builtin`/`lower_into` in `mir/build.rs` know `as_str`
//! but not `as_bytes`; using it yields `Unsupported("indirect/unknown aggregate
//! call")`). `println_mir_boundary_is_unsupported_as_bytes` PINS that boundary so
//! it cannot regress silently and so wiring `as_bytes` into MIR (the natural next
//! step) trips a red test prompting the upgrade to a both-engines parity assert.
//! See the STD-IO-PRINT obligation.
//!
//! The `Show` convention (and a `print_show[T: Show]`) is deliberately EXCLUDED:
//! the monomorphizer drops `extern` items and any `interface`/`impl` marks the
//! program generic, so combining `Show` with the io boundary erases the externs
//! (`sys_write` -> `unknown name` at runtime). See the STD-IO-PRINT obligation.
//!
//! These tests share process-global state (the shim registry and captured I/O
//! buffers), so they serialize on `GUARD`, exactly like `tests/std_io.rs`.

use candor::foreign_io;
use candor::{run_source_real, run_source_real_mir, MirRunResult, RunResult};
use std::sync::{Mutex, MutexGuard};

static GUARD: Mutex<()> = Mutex::new(());

fn lock() -> MutexGuard<'static, ()> {
    GUARD.lock().unwrap_or_else(|e| e.into_inner())
}

fn read_fixture(rel: &str) -> String {
    let path = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

/// The std/io module minus its demonstrator `main` — the `boundary` preamble, the
/// externs with trust clauses, `IoResult`, and the pub wrappers — so the composed
/// image drives the REAL module wrappers (no re-implementation).
fn io_prefix() -> String {
    let f = read_fixture("std_io/main.cnr");
    let idx = f.find("fn main").expect("io fixture has a main");
    f[..idx].to_string()
}

/// The non-generic formatting core of `std_fmt.cnr` — the bump allocator and
/// `fmt_i64`, sliced verbatim from the allocator's first `struct` up to (but not
/// including) the `Show` interface. The generic `Opt[T]` (above the allocator) and
/// the `Show` interface/impls (below `fmt_i64`) are excluded so the composed image
/// stays non-generic and keeps its externs.
fn fmt_core() -> String {
    let f = read_fixture("std_fmt.cnr");
    let start = f.find("struct AllocVtable").expect("std_fmt has the allocator");
    let end = f.find("interface Show").expect("std_fmt has the Show interface");
    f[start..end].to_string()
}

/// Compose the full print image: io wrappers + formatting core + print layer +
/// the test's `main`.
fn image(main_body: &str) -> String {
    format!(
        "{}\n{}\n{}\n{}",
        io_prefix(),
        fmt_core(),
        read_fixture("std_print.cnr"),
        main_body,
    )
}

fn run_tree_ok(src: &str) -> i64 {
    match run_source_real(src) {
        RunResult::Ok(r) => r.ret,
        RunResult::Fault(f) => panic!("tree fault: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!("tree check: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        RunResult::ParseError(d) => panic!("tree parse: {}", d.to_json()),
    }
}

/// Run the composed image on the tree-walker with the host-backed shims and
/// return `(ret, captured_stdout)`.
fn run_tree_captured(main_body: &str) -> (i64, Vec<u8>) {
    let src = image(main_body);
    foreign_io::reset();
    foreign_io::register_std_io();
    let ret = run_tree_ok(&src);
    let out = foreign_io::take_stdout();
    foreign_io::unregister_std_io();
    (ret, out)
}

const MAIN_PREAMBLE: &str = "fn main() alloc -> i64 {\n\
    let mut bs: Bump = with_window(16777216, 1048576);\n\
    let al: Alloc = mk_alloc(write bs);\n";

fn print_main(body: &str) -> String {
    format!("{MAIN_PREAMBLE}{body}\n  return 0;\n}}")
}

const SPEC_MAIN: &str = "  let _a: IoResult = println_i64(al, 0);\n\
       let _b: IoResult = println_i64(al, 42);\n\
       let _c: IoResult = println_i64(al, 0 - 42);\n\
       let _d: IoResult = println_i64(al, -9223372036854775808);";

// ---- 1. the composed image checks clean ------------------------------------
#[test]
fn print_image_checks_clean() {
    let _g = lock();
    let src = image(&print_main("  let _r: IoResult = println_i64(al, 0);"));
    let diags = candor::check_source_real(&src).unwrap_or_else(|p| panic!("parse: {}", p.to_json()));
    let errs: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == candor::diag::Severity::Error)
        .map(|d| d.code.clone())
        .collect();
    assert!(errs.is_empty(), "print image should check clean, got {errs:?}");
}

// ---- 2. println_i64 across zero / positive / negative / i64::MIN -----------
#[test]
fn println_i64_renders_and_writes_line_per_value() {
    let _g = lock();
    let (ret, out) = run_tree_captured(&print_main(SPEC_MAIN));
    assert_eq!(ret, 0);
    assert_eq!(
        out, b"0\n42\n-42\n-9223372036854775808\n",
        "each value on its own line, i64::MIN total"
    );
}

// ---- 3. print_str writes a str view verbatim (no newline) ------------------
#[test]
fn print_str_writes_verbatim() {
    let _g = lock();
    let (ret, out) = run_tree_captured(&print_main(
        "  let _a: IoResult = print_str(\"hi\");\n\
           let _b: IoResult = print_str(\"!\");",
    ));
    assert_eq!(ret, 0);
    assert_eq!(out, b"hi!", "verbatim, no trailing newline");
}

// ---- 4. print_str and println_i64 compose on one line ----------------------
#[test]
fn print_str_and_println_compose() {
    let _g = lock();
    let (ret, out) = run_tree_captured(&print_main(
        "  let _a: IoResult = print_str(\"n=\");\n\
           let _b: IoResult = println_i64(al, 7);",
    ));
    assert_eq!(ret, 0);
    assert_eq!(out, b"n=7\n");
}

// ---- 5. the exact spec line, asserted as a UTF-8 string --------------------
#[test]
fn println_i64_exact_spec_bytes() {
    let _g = lock();
    let (_ret, out) = run_tree_captured(&print_main(SPEC_MAIN));
    // Documented, load-bearing assertion of the whole slice's output contract.
    assert_eq!(String::from_utf8(out).unwrap(), "0\n42\n-42\n-9223372036854775808\n");
}

// ---- 6. both engines produce the exact spec bytes (`as_bytes` now lowers) ---
#[test]
fn println_both_engines_byte_parity() {
    let _g = lock();
    // `as_bytes` now has a MIR lowering (a free `str` -> `[u8]` retype, mirroring
    // the tree-walker), so the whole println path reaches the write boundary on
    // BOTH engines. Assert byte-for-byte parity of the captured stdout.
    const EXPECT: &[u8] = b"0\n42\n-42\n-9223372036854775808\n";
    let src = image(&print_main(SPEC_MAIN));

    foreign_io::reset();
    foreign_io::register_std_io();
    let tret = run_tree_ok(&src);
    let tout = foreign_io::take_stdout();
    foreign_io::unregister_std_io();

    foreign_io::reset();
    foreign_io::register_std_io();
    let mres = run_source_real_mir(&src);
    let mout = foreign_io::take_stdout();
    foreign_io::unregister_std_io();

    assert_eq!(tret, 0);
    assert_eq!(tout, EXPECT, "tree-walker spec bytes");
    match mres {
        MirRunResult::Ok(run) => {
            assert_eq!(run.ret, 0);
            assert_eq!(mout, EXPECT, "MIR spec bytes");
        }
        MirRunResult::Fault(f) => panic!("mir fault: {}", f.to_json()),
        MirRunResult::Unsupported(msg) => panic!("mir unsupported: {msg}"),
        other => panic!("mir not ok: {}", matches!(other, MirRunResult::Ok(_))),
    }
    assert_eq!(tout, mout, "both engines byte-for-byte");
}

// ---- 7. `print_show[T: Show]` through the I/O boundary ----------------------
// The generic path the println slice had to skip: the `Show` convention wired
// through `write_all`. It exercises BOTH just-landed fixes at once — the `Show`
// interface makes the image generic (so it goes through the monomorphizer, which
// now KEEPS the `sys_write` extern), and rendering to bytes goes through
// `as_bytes` (which now lowers on MIR).

/// The FULL `std_fmt.cnr` (allocator + `fmt_i64` + the `Show` interface/impls +
/// `show_it`) so the image is generic — unlike `fmt_core`, which slices Show out.
fn fmt_full() -> String {
    read_fixture("std_fmt.cnr")
}

/// The generic `Show` print layer (`print_show`/`println_show`) — the `T: Show`
/// sibling of the non-generic `std_print.cnr`, now that the monomorphizer carries
/// the io `extern`s through (so a generic image keeps `sys_write`).
fn show_layer() -> String {
    read_fixture("std_print_show.cnr")
}

/// Compose the generic print image: io wrappers + full formatting (with Show) +
/// the generic `Show` print layer + the test\'s `main`.
fn show_image(main_body: &str) -> String {
    format!("{}\n{}\n{}\n{}", io_prefix(), fmt_full(), show_layer(), main_body)
}

#[test]
fn print_show_through_boundary_renders_bytes() {
    let _g = lock();
    // A `Show` leaf (`ShowInt`) and a `T: Show` composition (`Opt[ShowInt]`),
    // rendered straight to stdout through the boundary. Tree-walker is the
    // load-bearing observation; MIR must match now that externs survive mono
    // and `as_bytes` lowers.
    let body = print_main(
        "  let x: ShowInt = ShowInt { val: 42 };
           let _a: IoResult = print_show(read x, al);
           let y: Opt[ShowInt] = Opt::Some(ShowInt { val: 9 });
           let _b: IoResult = print_show(read y, al);",
    );
    let src = show_image(&body);
    const EXPECT: &[u8] = b"42Some(9)";

    foreign_io::reset();
    foreign_io::register_std_io();
    let tret = run_tree_ok(&src);
    let tout = foreign_io::take_stdout();
    foreign_io::unregister_std_io();
    assert_eq!(tret, 0);
    assert_eq!(tout, EXPECT, "tree-walker print_show bytes");

    foreign_io::reset();
    foreign_io::register_std_io();
    let mres = run_source_real_mir(&src);
    let mout = foreign_io::take_stdout();
    foreign_io::unregister_std_io();
    match mres {
        MirRunResult::Ok(run) => {
            assert_eq!(run.ret, 0);
            assert_eq!(mout, EXPECT, "MIR print_show bytes");
        }
        MirRunResult::Fault(f) => panic!("mir fault: {}", f.to_json()),
        MirRunResult::Unsupported(msg) => panic!("mir unsupported: {msg}"),
        other => panic!("mir not ok: {}", matches!(other, MirRunResult::Ok(_))),
    }
}

// ---- 8. `println_show[T: Show]` — the unified `Show` print surface ----------
// `println_show` is `print_show` plus a trailing newline (byte 10), the `Show`
// analog of `println_i64`. It renders each value through the SAME generic-through-
// boundary path (Show interface makes the image generic; the extern survives mono;
// `as_bytes` lowers on MIR), so both engines must produce identical bytes. Exercised
// over a `Show` leaf (`ShowInt`) and a `T: Show` composition (`Opt[ShowInt]`), each
// on its own line.
#[test]
fn println_show_renders_line_per_value_both_engines() {
    let _g = lock();
    let body = print_main(
        "  let x: ShowInt = ShowInt { val: 42 };
           let _a: IoResult = println_show(read x, al);
           let y: Opt[ShowInt] = Opt::Some(ShowInt { val: 9 });
           let _b: IoResult = println_show(read y, al);
           let z: Opt[ShowInt] = Opt::None;
           let _c: IoResult = println_show(read z, al);",
    );
    let src = show_image(&body);
    const EXPECT: &[u8] = b"42\nSome(9)\nNone\n";

    foreign_io::reset();
    foreign_io::register_std_io();
    let tret = run_tree_ok(&src);
    let tout = foreign_io::take_stdout();
    foreign_io::unregister_std_io();
    assert_eq!(tret, 0);
    assert_eq!(tout, EXPECT, "tree-walker println_show bytes");

    foreign_io::reset();
    foreign_io::register_std_io();
    let mres = run_source_real_mir(&src);
    let mout = foreign_io::take_stdout();
    foreign_io::unregister_std_io();
    match mres {
        MirRunResult::Ok(run) => {
            assert_eq!(run.ret, 0);
            assert_eq!(mout, EXPECT, "MIR println_show bytes");
        }
        MirRunResult::Fault(f) => panic!("mir fault: {}", f.to_json()),
        MirRunResult::Unsupported(msg) => panic!("mir unsupported: {msg}"),
        other => panic!("mir not ok: {}", matches!(other, MirRunResult::Ok(_))),
    }
    assert_eq!(tout, mout, "both engines byte-for-byte");
}


// ---- 9. `Show for String` renders a String through the SAME boundary ---------
// The payoff of the coherence fix: `impl Show for String` now coexists with
// `impl Show for ShowInt` (distinct target nominals no longer falsely overlap),
// so a `String` renders through the very same generic `Show`-through-boundary path
// as the other `Show` witnesses. Build a String, render it through `println_show`,
// and assert its text + trailing newline byte-for-byte on BOTH engines.
#[test]
fn println_show_string_both_engines() {
    let _g = lock();
    let body = print_main(
        "  let mut msg: String = string_new(read al);
           append(write msg, \"hello\");
           let _a: IoResult = println_show(read msg, al);",
    );
    let src = show_image(&body);
    const EXPECT: &[u8] = b"hello\n";

    foreign_io::reset();
    foreign_io::register_std_io();
    let tret = run_tree_ok(&src);
    let tout = foreign_io::take_stdout();
    foreign_io::unregister_std_io();
    assert_eq!(tret, 0);
    assert_eq!(tout, EXPECT, "tree-walker Show-for-String bytes");

    foreign_io::reset();
    foreign_io::register_std_io();
    let mres = run_source_real_mir(&src);
    let mout = foreign_io::take_stdout();
    foreign_io::unregister_std_io();
    match mres {
        MirRunResult::Ok(run) => {
            assert_eq!(run.ret, 0);
            assert_eq!(mout, EXPECT, "MIR Show-for-String bytes");
        }
        MirRunResult::Fault(f) => panic!("mir fault: {}", f.to_json()),
        MirRunResult::Unsupported(msg) => panic!("mir unsupported: {msg}"),
        other => panic!("mir not ok: {}", matches!(other, MirRunResult::Ok(_))),
    }
    assert_eq!(tout, mout, "both engines byte-for-byte");
}
