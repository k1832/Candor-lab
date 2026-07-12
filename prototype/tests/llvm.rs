//! LLVM-S0 gate (the first OPTIMIZED native backend): `compile_path_llvm` emits
//! textual LLVM-IR from the scalar+control-flow MIR, `clang -O2` builds it, and
//! the standalone process — linked against the same static C runtime as the
//! Cranelift AOT object — must reproduce the tree-walking oracle's observable
//! result byte-exact: `θ` (the printed trace), the process exit byte, and the
//! `(kind, span)` fault JSON on stderr.
//!
//! This mirrors `tests/aot.rs`'s `aot_outcome`, restricted to the S0 subset
//! (scalars + control flow): overflow, div-by-zero, conv-loss, assert, requires,
//! ensures, panic, shift-overflow faults, plus arithmetic / fib recursion /
//! while-loop / bitwise OK slices. A perf spot-check proves `-O2` promoted the
//! alloca-per-local slots to SSA registers (mem2reg).

use std::path::Path;
use std::process::Command;

use candor_proto::interp::{Fault, FaultKind};
use candor_proto::{run_source, run_source_real, RunResult};

#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    Ok { exit: u8, trace: Vec<i64> },
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

/// The compiled (clang -O2) process's outcome for a source file.
fn llvm_outcome(path: &Path, tag: &str) -> Result<Outcome, String> {
    let out = std::env::temp_dir().join(format!("candor-llvm-gate-{}-{}", std::process::id(), tag));
    candor_proto::compile_path_llvm(path, &out)?;
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

fn assert_llvm_eq_oracle(src: &str, real: bool, tag: &str) {
    let o = oracle_src(src, real).expect("oracle should run this program");
    let ext = if real { "cnr" } else { "cn" };
    let srcpath =
        std::env::temp_dir().join(format!("candor-llvm-src-{}-{}.{ext}", std::process::id(), tag));
    std::fs::write(&srcpath, src).unwrap();
    let a = llvm_outcome(&srcpath, tag).expect("compile+run should succeed");
    let _ = std::fs::remove_file(&srcpath);
    assert_eq!(a, o, "LLVM-S0 vs oracle divergence for:\n{src}");
}

fn clang_available() -> bool {
    Command::new("clang")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// 1. Fault axes — the S0 scalar subset: every mode delivers the same (kind, span).
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
];

#[test]
fn gate_llvm_fault_axes() {
    assert!(clang_available(), "clang unavailable: cannot build the LLVM-S0 gate");
    for (i, (src, real)) in FAULTS.iter().enumerate() {
        let o = oracle_src(src, *real).expect("faulting program should run");
        assert!(matches!(o, Outcome::Fault { .. }), "expected a fault:\n{src}");
        assert_llvm_eq_oracle(src, *real, &format!("fault{i}"));
    }
}

// ---------------------------------------------------------------------------
// 2. OK slice — traces + exit codes must match byte-exact.
// ---------------------------------------------------------------------------

const OK: &[(&str, bool)] = &[
    ("fn main() -> i64 { let a: i64 = 20; let b: i64 = 22; trace(a); trace(b); return a + b; }", false),
    ("fn fib(n: i64) -> i64 { if n < 2 { return n; } return fib(n - 1) + fib(n - 2); } fn main() -> i64 { let r: i64 = fib(10); trace(r); return r; }", false),
    ("fn main() -> i64 { let mut s: i64 = 0; let mut i: i64 = 0; while i < 5 { s = s + i; trace(i); i = i + 1; } trace(s); return s; }", false),
    ("fn main() -> i64 { let a: u8 = 12u8; let b: u8 = 10u8; trace(conv i64 (a & b)); trace(conv i64 (a | b)); trace(conv i64 (a ^ b)); return 0; }", true),
];

#[test]
fn gate_llvm_ok_slice() {
    assert!(clang_available(), "clang unavailable");
    for (i, (src, real)) in OK.iter().enumerate() {
        assert_llvm_eq_oracle(src, *real, &format!("ok{i}"));
    }
}

// ---------------------------------------------------------------------------
// 3. Perf spot-check — prove the optimization is REAL: the emitted .ll uses the
//    alloca-per-local model, and `clang -O2` (mem2reg) promotes every slot to a
//    register, leaving NO `alloca` in the optimized IR. This is what turns the
//    naive load/store lowering into optimized native code.
// ---------------------------------------------------------------------------

#[test]
fn perf_mem2reg_promotes_locals() {
    assert!(clang_available(), "clang unavailable");
    let src = "fn main() -> i64 { let mut s: i64 = 0; let mut i: i64 = 0; \
               while i < 100 { s = s + i; i = i + 1; } return s; }";
    let srcpath =
        std::env::temp_dir().join(format!("candor-llvm-perf-{}.cn", std::process::id()));
    std::fs::write(&srcpath, src).unwrap();
    let ll = candor_proto::emit_llvm_ir(&srcpath).expect("emit .ll");

    // The unoptimized emission is the alloca-per-local model.
    assert!(ll.contains("alloca i64"), "expected alloca-per-local IR, got:\n{ll}");

    // clang -O2 mem2reg should eliminate every alloca (slots -> SSA registers).
    let llpath = std::env::temp_dir().join(format!("candor-llvm-perf-{}.ll", std::process::id()));
    std::fs::write(&llpath, &ll).unwrap();
    let out = Command::new("clang")
        .args(["-O2", "-S", "-emit-llvm", "-o", "-"])
        .arg(&llpath)
        .output()
        .expect("clang -O2 -S -emit-llvm");
    let _ = std::fs::remove_file(&srcpath);
    let _ = std::fs::remove_file(&llpath);
    assert!(out.status.success(), "clang -O2 -emit-llvm failed: {}", String::from_utf8_lossy(&out.stderr));
    let opt = String::from_utf8_lossy(&out.stdout);
    assert!(
        !opt.contains("alloca"),
        "mem2reg should have removed every alloca under -O2; optimized IR still spills:\n{opt}"
    );
}

// ---------------------------------------------------------------------------
// 4. Aggregates (S1): the flat two-tier model — struct/array literals, field/
//    index read+assign, nested aggregates, by-value struct params + struct
//    returns, and the array-index bounds fault. Every fixture: clang -O2 build,
//    run, and byte-exact (exit / trace / fault-JSON) against the oracle.
// ---------------------------------------------------------------------------

const AGG_OK: &[(&str, bool)] = &[
    // Struct literal + field read, passed by borrow.
    ("struct Point { x: i64, y: i64 } fn add(p: read Point) -> i64 { return p.x + p.y; } \
      fn main() -> i64 { let pt: Point = Point { x: 40, y: 2 }; return add(read pt); }", true),
    // Field assign + field read (trace order + exit).
    ("struct P { x: i64, y: i64 } fn main() -> i64 { let mut p: P = P { x: 1, y: 2 }; \
      p.x = 10; trace(p.x); trace(p.y); return p.x + p.y; }", true),
    // Nested struct field read.
    ("struct Inner { a: i64, b: i64 } struct Outer { p: Inner, c: i64 } \
      fn main() -> i64 { let o: Outer = Outer { p: Inner { a: 3, b: 4 }, c: 5 }; \
      return o.p.a + o.p.b + o.c; }", true),
    // By-value struct param + struct return (the byte-copy ABI both ways).
    ("struct P { x: i64, y: i64 } fn mk(a: i64, b: i64) -> P { return P { x: a, y: b }; } \
      fn sum(p: P) -> i64 { return p.x + p.y; } \
      fn main() -> i64 { let p: P = mk(30, 12); return sum(p); }", true),
    // Array listed literal + index read (dynamic index).
    ("fn main() -> i64 { let a: [3]i64 = [10, 20, 30]; let i: usize = 2; return a[i] + a[0]; }", true),
    // Array-repeat literal `[e; N]` (scalar element via the byte-copy path).
    ("fn main() -> i64 { let a: [4]i64 = [7; 4]; let i: usize = 3; return a[i] + a[1]; }", true),
    // Array index-assign (mutating an element) + trace of each slot.
    ("fn main() -> i64 { let mut a: [3]i64 = [1, 2, 3]; a[1] = 20; \
      trace(a[0]); trace(a[1]); trace(a[2]); return a[0] + a[1] + a[2]; }", true),
    // Array of structs: nested index + field projection.
    ("struct P { x: i64, y: i64 } \
      fn main() -> i64 { let a: [2]P = [P { x: 1, y: 2 }, P { x: 3, y: 4 }]; \
      let i: usize = 1; return a[i].x + a[i].y; }", true),
    // Sub-word (u8) array elements: byte-granular flat load/store.
    ("fn main() -> i64 { let a: [3]u8 = [1u8, 2u8, 3u8]; let i: usize = 2; \
      return conv i64 (a[i]) + conv i64 (a[0]); }", true),
];

#[test]
fn gate_llvm_aggregates() {
    assert!(clang_available(), "clang unavailable: cannot build the LLVM-S1 aggregate gate");
    for (i, (src, real)) in AGG_OK.iter().enumerate() {
        assert_llvm_eq_oracle(src, *real, &format!("agg{i}"));
    }
}

// The array-index bounds fault (now in the S1 subset): kind + span must match the
// oracle byte-exact — the same fixture the AOT gate carries.
const AGG_FAULTS: &[(&str, bool)] = &[
    ("fn main() -> i64 { let a: [3]i64 = [1, 2, 3]; let i: usize = 5; return a[i]; }", false),
    ("struct P { x: i64, y: i64 } \
      fn main() -> i64 { let a: [2]P = [P { x: 1, y: 2 }, P { x: 3, y: 4 }]; \
      let i: usize = 9; return a[i].x; }", false),
];

#[test]
fn gate_llvm_aggregate_bounds_fault() {
    assert!(clang_available(), "clang unavailable");
    for (i, (src, real)) in AGG_FAULTS.iter().enumerate() {
        let o = oracle_src(src, *real).expect("faulting program should run");
        assert!(matches!(o, Outcome::Fault { .. }), "expected a bounds fault:\n{src}");
        assert_llvm_eq_oracle(src, *real, &format!("aggfault{i}"));
    }
}
