//! SHA-256 dogfood gate: `tests/fixtures/run/sha256.cnr` implements FIPS 180-4
//! in pure Candor (wrapping-block mod-2^32 sums, rotations composed from
//! shift/or, big-endian marshalling through masked `conv`s) and hashes the
//! official vectors — empty, "abc", the 448-bit and 896-bit NIST two-block
//! messages — plus a 10,000-'a' repeated-input vector, asserting each digest
//! against its embedded lowercase-hex string in-program. This test pins the
//! RIGHT answer externally too: the traced digest words must equal the NIST /
//! `hashlib.sha256` reference digests below, byte-exact on every engine — the
//! tree-walking oracle, the MIR interpreter, both Cranelift backends, and LLVM
//! (clang -O2, via the trace channel) when clang is present. The fixture also
//! auto-enlists in the corpus gates (tests/stage_d.rs, tests/aot.rs,
//! tests/llvm.rs) by living in `tests/fixtures/run/`.
//!
//! The official million-'a' vector lives in `sha256_million_a` below (pinned to
//! the NIST reference digest), not in the fixture. Historical note: MIR once
//! unrolled `[97u8; N]` into one CopyVal per element, making a million-element
//! repeat unaffordable at native COMPILE time; the lowering is now a counted
//! loop, so promotion into the fixture is possible compile-wise — but the
//! interpreter-side corpus gates would still pay minutes of RUNTIME per
//! engine, so the fixture keeps the 10,000 repeat.
//! Both interpreters used to leak stack per call until the
//! caller's frame returned (MIR parked the whole callee frame plus the return
//! slot above the caller's region; the tree-walker never popped block locals or
//! statement temporaries), so call-heavy loops crossed the 256 MiB model cap
//! (`interp::mem::MAX_ADDR`) and faulted `bad_pointer` on safe programs — MIR
//! at 15,625 compress calls (million-'a'), the tree-walker at repro scale.
//! `interp_stack_reclamation` below pins the fix: MIR pops to the frame's
//! locals watermark after every statement and copies call returns down to the
//! caller's pre-call stack top; the tree-walker pops each block's bytes at
//! block exit. Since task #144 the native engines roll their (atomic,
//! task-shared) bump back per frame too — `rt_stack_save` at an allocating
//! fn's entry, `rt_stack_restore` at its return, gated on no live tasks — so
//! the reclamation repro now also pins the Cranelift JIT (run_source_real_native) alongside both interpreters; AOT/LLVM/freestanding flatness is pinned by the dedicated parity gates.

use candor::{
    compile_path_llvm, run_source_real, run_source_real_mir, run_source_real_native,
    run_source_real_native_opt, MirRunResult, RunResult,
};
use std::path::Path;
use std::process::Command;

fn fixture() -> String {
    let path = format!("{}/tests/fixtures/run/sha256.cnr", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn oracle(src: &str) -> (i64, Vec<i64>) {
    match run_source_real(src) {
        RunResult::Ok(r) => (r.ret, r.trace),
        RunResult::Fault(f) => panic!("oracle faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => {
            panic!("oracle check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("oracle parse error: {}", d.to_json()),
    }
}

fn mir_out(r: MirRunResult, label: &str) -> (i64, Vec<i64>) {
    match r {
        MirRunResult::Ok(r) => (r.ret, r.trace),
        MirRunResult::Fault(f) => panic!("{label} faulted: {}", f.to_json()),
        MirRunResult::Unsupported(e) => panic!("{label} unsupported: {e}"),
        MirRunResult::CheckErrors(d) => {
            panic!("{label} check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        MirRunResult::ParseError(d) => panic!("{label} parse error: {}", d.to_json()),
    }
}

fn clang_available() -> bool {
    Command::new("clang").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn llvm_trace(src: &str, tag: &str) -> Option<Vec<i64>> {
    if !clang_available() {
        return None;
    }
    let dir = std::env::temp_dir();
    let srcp = dir.join(format!("candor-sha256-{}-{}.cnr", std::process::id(), tag));
    let outp = dir.join(format!("candor-sha256-{}-{}", std::process::id(), tag));
    std::fs::write(&srcp, src).unwrap();
    compile_path_llvm(Path::new(&srcp), &outp).expect("LLVM compile should succeed");
    let output = Command::new(&outp).output().expect("run compiled sha256 program");
    let _ = std::fs::remove_file(&srcp);
    let _ = std::fs::remove_file(&outp);
    let trace = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.trim().parse::<i64>().expect("trace line is an integer"))
        .collect();
    Some(trace)
}

/// Run `src` through every engine, assert byte-exact agreement (`ret` for the
/// four in-process engines; `trace` for all five including LLVM), return the
/// oracle's `(ret, trace)`.
fn all_engines(src: &str, tag: &str) -> (i64, Vec<i64>) {
    let (o_ret, o_trace) = oracle(src);
    let (m_ret, m_trace) = mir_out(run_source_real_mir(src), "mir");
    let (n_ret, n_trace) = mir_out(run_source_real_native(src), "native-noopt");
    let (p_ret, p_trace) = mir_out(run_source_real_native_opt(src), "native-opt");
    for (label, ret, trace) in [
        ("mir", m_ret, &m_trace),
        ("native-noopt", n_ret, &n_trace),
        ("native-opt", p_ret, &p_trace),
    ] {
        assert_eq!(ret, o_ret, "{label} ret diverged from oracle");
        assert_eq!(trace, &o_trace, "{label} trace diverged from oracle");
    }
    if let Some(l_trace) = llvm_trace(src, tag) {
        assert_eq!(l_trace, o_trace, "llvm trace diverged from oracle");
    }
    (o_ret, o_trace)
}

/// The reference digests (NIST CAVP for the four official vectors;
/// `hashlib.sha256(b"a"*10000)` for the repeated-input vector), in the order
/// the fixture hashes them.
const EXPECTED_HEX: [&str; 5] = [
    // SHA-256("")
    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    // SHA-256("abc")
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    // SHA-256("abcdbcde...nopq") — the 448-bit NIST vector
    "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
    // SHA-256("abcdefgh...rstu") — the 896-bit NIST vector
    "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1",
    // SHA-256("a" * 10000)
    "27dd1f61b867b6a0f6e9d8a41c43231de52107e53ae424de8f847b821db4b711",
];

/// The trace the fixture must emit: for each digest its 8 big-endian u32 words
/// (zero-extended to i64), then the summed byte checksum of all five digests.
fn expected_trace() -> (Vec<i64>, i64) {
    let mut trace = Vec::new();
    let mut total: i64 = 0;
    for hex in EXPECTED_HEX {
        let bytes: Vec<u8> = (0..32)
            .map(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap())
            .collect();
        for w in bytes.chunks(4) {
            trace.push(u32::from_be_bytes(w.try_into().unwrap()) as i64);
        }
        total += bytes.iter().map(|b| *b as i64).sum::<i64>();
    }
    trace.push(total);
    (trace, total)
}

#[test]
fn sha256_vectors_all_engines() {
    let src = fixture();
    let (ret, trace) = all_engines(&src, "vectors");
    let (want_trace, total) = expected_trace();
    assert_eq!(trace, want_trace, "digest words diverged from the reference digests");
    // The fixture folds the checksum to an exit byte so the process-level
    // engines (AOT / LLVM, exit-code channel) compare byte-exact.
    assert_eq!(ret, total % 256, "ret is the byte-folded digest checksum");
    assert_eq!(ret, 247);
}

/// SHA-256("a" * 1,000,000) — the official NIST CAVP long vector.
const MILLION_A_HEX: &str = "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0";

/// The official million-'a' vector on the MIR interpreter, pinned to the NIST
/// reference digest. This input used to fault `bad_pointer` on MIR (per-call
/// stack leak, see the module docs); it must now hash byte-exact. The MIR
/// engine runs it alone deliberately: the native engines could now compile it
/// cheaply (the array-repeat lowering is a counted loop), but running it on
/// every engine adds minutes of interpreter/JIT runtime for no extra coverage
/// — and the tree-walking oracle alone would add ~2 debug-build minutes — the reference digest referees this test,
/// `sha256_vectors_all_engines` does the all-engine differential comparison,
/// and `interp_stack_reclamation` stresses the oracle's reclamation harder
/// than this vector does.
#[test]
fn sha256_million_a() {
    let src = fixture()
        .replace("[97u8; 10000]", "[97u8; 1000000]")
        .replace("[10000]u8", "[1000000]u8")
        .replace(
            "27dd1f61b867b6a0f6e9d8a41c43231de52107e53ae424de8f847b821db4b711",
            MILLION_A_HEX,
        );
    // Expected trace: the four official vectors' digest words, the million-'a'
    // digest words, then the summed byte checksum of all five digests.
    let mut want_trace = Vec::new();
    let mut total: i64 = 0;
    for hex in EXPECTED_HEX[..4].iter().chain([&MILLION_A_HEX]) {
        let bytes: Vec<u8> =
            (0..32).map(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap()).collect();
        for w in bytes.chunks(4) {
            want_trace.push(u32::from_be_bytes(w.try_into().unwrap()) as i64);
        }
        total += bytes.iter().map(|b| *b as i64).sum::<i64>();
    }
    want_trace.push(total);
    let (ret, trace) = mir_out(run_source_real_mir(&src), "mir");
    assert_eq!(trace, want_trace, "mir trace diverged from the reference digests");
    assert_eq!(ret, total % 256, "mir ret is the byte-folded digest checksum");
}

/// Regression gate for stack reclamation (formerly `mir_leak_repro`, which
/// pinned the pre-fix fault): an aggregate-returning callee in a hot caller
/// loop moves 40,000 x 8 KiB of call returns — over 320 MiB of frame +
/// return-slot traffic against the 256 MiB model cap. Any regression to
/// caller-frame-exit reclamation (MIR's per-statement watermark pop, the
/// tree-walker's per-block pop, and — since task #144 — the native engines'
/// per-frame `rt_stack_save`/`rt_stack_restore` rollback) faults
/// `bad_pointer` here on a safe program.
#[test]
fn interp_stack_reclamation() {
    let src = "\
        fn blob() -> [8192]u8 { let b: [8192]u8 = [1u8; 8192]; return b; }\n\
        fn main() -> i64 {\n\
            let mut i: i64 = 0;\n\
            let mut acc: i64 = 0;\n\
            while i < 40000 {\n\
                let b: [8192]u8 = blob();\n\
                acc = acc + conv i64 b[0];\n\
                i = i + 1;\n\
            }\n\
            return acc % 256;\n\
        }\n";
    match run_source_real_mir(src) {
        MirRunResult::Ok(r) => assert_eq!(r.ret, 40000 % 256, "MIR ret"),
        MirRunResult::Fault(f) => {
            panic!("MIR stack reclamation regressed (leak repro faulted): {}", f.to_json())
        }
        _ => panic!("unexpected MIR outcome (check/parse error or unsupported)"),
    }
    match run_source_real(src) {
        RunResult::Ok(r) => assert_eq!(r.ret, 40000 % 256, "oracle ret"),
        RunResult::Fault(f) => {
            panic!("tree-walker stack reclamation regressed (leak repro faulted): {}", f.to_json())
        }
        _ => panic!("unexpected oracle outcome (check/parse error)"),
    }
    match run_source_real_native(src) {
        MirRunResult::Ok(r) => assert_eq!(r.ret, 40000 % 256, "native ret"),
        MirRunResult::Fault(f) => {
            panic!("native stack rollback regressed (leak repro faulted): {}", f.to_json())
        }
        _ => panic!("unexpected native outcome (check/parse error or unsupported)"),
    }
}
