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

// ---------------------------------------------------------------------------
// 5. Tagged-union ENUMS + STATIC/CONST data (S2): the flat-model enum (tag +
//    payload union), `match` (tag read -> icmp/branch chain -> per-variant
//    payload projection, incl. the wildcard/default arm), enum-in-struct and
//    struct-in-enum nesting, and the static region (scalar/aggregate/array
//    `static` initializers run before `main`, plus string-literal bytes). Every
//    fixture: clang -O2 build, run, byte-exact (exit / trace / fault-JSON)
//    against the oracle — the same fixtures the AOT corpus carries.
// ---------------------------------------------------------------------------

const ENUM_STATIC_OK: &[(&str, bool)] = &[
    // Enum construct + `match` with a payload binding.
    ("enum R { ok Val(i64), Err } fn dv(n: i64, d: i64) -> R { let q: i64 = n / d; return R::Val(q); } \
      fn main() -> i64 { return match dv(84, 2) { R::Val(v) => v, R::Err => 0 - 1 }; }", true),
    // Multi-variant enum + a wildcard/default arm (borrowed scrutinee).
    ("enum Shape { Circle(i64), Square(i64), Unknown } \
      fn area(s: read Shape) -> i64 { return match s { Shape::Circle(r) => r * r * 3, Shape::Square(w) => w * w, _ => 0 }; } \
      fn main() -> i64 { let a: Shape = Shape::Circle(2); let b: Shape = Shape::Square(5); let c: Shape = Shape::Unknown; \
      return area(read a) + area(read b) + area(read c); }", true),
    // Struct-in-enum: `?` propagation on an `ok`-marked result enum whose payload
    // reads a struct field (the real-syntax `propagate` golden fixture).
    ("struct Acc { total: i64, count: i64 } enum Step { ok Cont(i64), Stop } \
      fn advance(a: write Acc, delta: i64) -> Step { a.*.total = a.total + delta; a.*.count = a.count + 1; \
      if a.count >= 3 { return Step::Stop; } return Step::Cont(a.total); } \
      fn drive(a: write Acc) -> Step { let x: i64 = advance(a, 10)?; let y: i64 = advance(a, 20)?; let z: i64 = advance(a, 30)?; return Step::Cont(z); } \
      fn main() -> i64 { let mut acc: Acc = Acc { total: 0, count: 0 }; let r: Step = drive(write acc); \
      return match r { Step::Cont(v) => v, Step::Stop => acc.total }; }", true),
    // Enum-in-struct: a struct field is an enum; match projects its payload.
    ("enum E { A(i64), B } struct Wrap { tag: i64, e: E } \
      fn main() -> i64 { let w: Wrap = Wrap { tag: 7, e: E::A(35) }; \
      return match w.e { E::A(v) => v + w.tag, E::B => w.tag }; }", true),
    // Static scalar read (the initializer runs before `main`).
    ("static BASE: i64 = 100; static STEP: i64 = 7; fn main() -> i64 { return BASE + STEP + STEP; }", true),
    // Static aggregate (struct) read: field projection through the static's address.
    ("struct P { x: i64, y: i64 } static ORIGIN: P = P { x: 3, y: 4 }; \
      fn main() -> i64 { return ORIGIN.x + ORIGIN.y; }", true),
    // Static array read (dynamic index).
    ("static TABLE: [3]i64 = [10, 20, 30]; fn main() -> i64 { let i: usize = 2; return TABLE[0] + TABLE[i]; }", true),
    // String-literal (byte-array) data: the bytes are laid into the static region.
    ("fn main() -> i64 { let s: [u8] = b\"AZ\"; let i: usize = 1; return conv i64 s[0] + conv i64 s[i]; }", true),
];

#[test]
fn gate_llvm_enums_and_statics() {
    assert!(clang_available(), "clang unavailable: cannot build the LLVM-S2 enum/static gate");
    for (i, (src, real)) in ENUM_STATIC_OK.iter().enumerate() {
        assert_llvm_eq_oracle(src, *real, &format!("s2ok{i}"));
    }
}

// A fault raised through an enum-returning `?` chain (the div-by-zero happens
// inside a `?`-called function that builds a result enum): kind + span must match
// the oracle byte-exact — the same fixture the AOT gate carries.
const ENUM_FAULTS: &[(&str, bool)] = &[
    ("enum R { ok Val(i64), Err } fn dv(n: i64, d: i64) -> R { let q: i64 = n / d; return R::Val(q); } \
      fn run() -> R { let a: i64 = dv(10, 0)?; return R::Val(a); } \
      fn main() -> i64 { return match run() { R::Val(v) => v, R::Err => 0 - 1 }; }", true),
];

#[test]
fn gate_llvm_enum_fault_axis() {
    assert!(clang_available(), "clang unavailable");
    for (i, (src, real)) in ENUM_FAULTS.iter().enumerate() {
        let o = oracle_src(src, *real).expect("faulting program should run");
        assert!(matches!(o, Outcome::Fault { .. }), "expected a fault:\n{src}");
        assert_llvm_eq_oracle(src, *real, &format!("s2fault{i}"));
    }
}

// ---------------------------------------------------------------------------
// 6. The MOVE/DROP SCHEDULE with trace-on-drop (S3): every needs-drop value runs
//    its drop glue at the scheduled point — a struct fires its `drop` hook (the
//    observable `trace`) then drops its fields inner-to-outer (reverse declaration
//    order), an array drops its elements in reverse, an enum switches on its tag
//    and drops the active variant's payload. A moved value (consumed by a call,
//    or a moved-out field) is pruned from the schedule by the static move mask —
//    never double-dropped — while the still-owned remainder drops exactly once.
//    The TRACE sequence is the correctness axis: byte-exact vs the oracle.
// ---------------------------------------------------------------------------

const DROP_OK: &[(&str, bool)] = &[
    // Single struct with a trace-on-drop hook: fires at scope exit (after trace(1)).
    ("struct Res { id: i64 } drop(write self) { trace(self.id); } \
      fn main() -> i64 { let r: Res = Res { id: 42 }; trace(1); return 0; }", true),
    // Two owned values drop in REVERSE declaration order (2 then 1).
    ("struct Res { id: i64 } drop(write self) { trace(self.id); } \
      fn main() -> i64 { let a: Res = Res { id: 1 }; let b: Res = Res { id: 2 }; trace(99); return 0; }", true),
    // Conditional move: `a` is consumed by `eat` (dropped once, inside the callee's
    // param scope) and pruned from main's schedule — no double-drop (trace 5 twice,
    // not thrice: the callee's param drop + trace(n)).
    ("struct Res { id: i64 } drop(write self) { trace(self.id); } \
      fn eat(r: Res) -> i64 { return r.id; } \
      fn main() -> i64 { let a: Res = Res { id: 5 }; let n: i64 = eat(a); trace(n); return 0; }", true),
    // Nested struct: fields drop inner-to-outer (b=2 then a=1).
    ("struct Inner { id: i64 } drop(write self) { trace(self.id); } \
      struct Outer { a: Inner, b: Inner } \
      fn main() -> i64 { let o: Outer = Outer { a: Inner { id: 1 }, b: Inner { id: 2 } }; trace(9); return 0; }", true),
    // Array of droppable elements: dropped in reverse index order (30,20,10).
    ("struct Res { id: i64 } drop(write self) { trace(self.id); } \
      fn main() -> i64 { let a: [3]Res = [Res { id: 10 }, Res { id: 20 }, Res { id: 30 }]; trace(0); return 0; }", true),
    // Enum drop: switch on the tag, drop the ACTIVE variant's payload (trace 77).
    ("struct Res { id: i64 } drop(write self) { trace(self.id); } \
      enum E { Has(Res), None } \
      fn main() -> i64 { let e: E = E::Has(Res { id: 77 }); trace(1); return 0; }", true),
    // Enum drop: a non-droppable variant runs no payload drop (only trace(1)).
    ("struct Res { id: i64 } drop(write self) { trace(self.id); } \
      enum E { Has(Res), None } \
      fn main() -> i64 { let e: E = E::None; trace(1); return 0; }", true),
    // Early return: the drop fires on BOTH control paths (trace 8 each).
    ("struct Res { id: i64 } drop(write self) { trace(self.id); } \
      fn f(early: bool) -> i64 { let r: Res = Res { id: 8 }; if early { trace(100); return 1; } trace(200); return 2; } \
      fn main() -> i64 { let x: i64 = f(true); let y: i64 = f(false); return 0; }", true),
    // Partial move: `p.a` is moved into `eat`; the struct-level schedule skips the
    // moved field and drops only the still-owned `p.b` (trace 1 from the callee,
    // trace 50, then trace 2 for p.b) — no double-drop of the moved `a`.
    ("struct Res { id: i64 } drop(write self) { trace(self.id); } \
      struct Pair { a: Res, b: Res } \
      fn eat(r: Res) -> i64 { return r.id; } \
      fn main() -> i64 { let p: Pair = Pair { a: Res { id: 1 }, b: Res { id: 2 } }; let n: i64 = eat(p.a); trace(50); return 0; }", true),
];

#[test]
fn gate_llvm_drop_trace() {
    assert!(clang_available(), "clang unavailable: cannot build the LLVM-S3 drop/trace gate");
    for (i, (src, real)) in DROP_OK.iter().enumerate() {
        assert_llvm_eq_oracle(src, *real, &format!("s3drop{i}"));
    }
}

// The generics corpus's value-drop fixtures: a generic struct `drop` hook that
// monomorphizes to a concrete value type — the hook fires BEFORE the field drop
// (`gdrop`: Wrap's tag hook, then the Noisy field), and a move into a callee drops
// the value in the callee's param scope (`gdrop_groundfloor`). Both are within the
// S3 value subset (no Box); their trace order must match the oracle byte-exact.
#[test]
fn gate_llvm_drop_trace_corpus() {
    assert!(clang_available(), "clang unavailable");
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/generics");
    for name in ["gdrop.cnr", "gdrop_groundfloor.cnr"] {
        let src = std::fs::read_to_string(dir.join(name)).expect("read generics drop fixture");
        let tag: String = name.chars().filter(|c| c.is_alphanumeric()).collect();
        assert_llvm_eq_oracle(&src, true, &tag);
    }
}

// ---------------------------------------------------------------------------
// 7. HEAP ALLOCATION — Box[T], the allocator ABI, rawptr load/store, and
//    drop-through-Box (S4). The systems corpus (design 0001 §11): a bump/pool
//    allocator dispatched through the `Alloc` handle's vtable (fn-pointer table),
//    `box`/`unbox` moving payloads through the returned block address, observable
//    rawptr/MMIO load/store (rt_mmio_load/store barriers), and the recursive
//    parser/arena whose Box pointees are freed (and their pointee drop schedule
//    run) on drop. Every fixture: clang -O2 build, run, byte-exact (exit / trace /
//    fault-JSON) against the oracle — the same programs the AOT corpus carries.
// ---------------------------------------------------------------------------

#[test]
fn gate_llvm_systems_corpus() {
    assert!(clang_available(), "clang unavailable: cannot build the LLVM-S4 systems gate");
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/selfhost_interp");
    for name in [
        "11_1_allocator",
        "11_2_scheduler",
        "11_3_mmio",
        "11_4_parser",
        "11_5_arena",
    ] {
        let src = std::fs::read_to_string(dir.join(format!("{name}.cnr")))
            .expect("read systems-corpus fixture");
        assert_llvm_eq_oracle(&src, true, name);
    }
}

// Drop-through-Box made OBSERVABLE: a `Res { drop -> trace(id) }` boxed through the
// bump allocator and then DROPPED (not unboxed) at scope exit. The Box's free-on-drop
// recurses into the pointee's drop hook, so the trace ends with the pointee id (77) —
// the arena drop story completed and proven byte-exact against the oracle.
#[test]
fn gate_llvm_drop_through_box() {
    assert!(clang_available(), "clang unavailable");
    let src = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/systems_llvm/box_drop_free.cnr"),
    )
    .expect("read box-drop fixture");
    assert_llvm_eq_oracle(&src, true, "boxdropfree");
}
