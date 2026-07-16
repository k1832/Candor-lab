//! Std string utilities (design 0013 text layer): the byte-scan predicates
//! `starts_with` / `ends_with` / `contains` / `find`, the view-returning `trim`,
//! and the two allocating builders `split` (OWNED `String` pieces into a
//! `Vec[String]`) and `join`, written over the existing `str` / `String` /
//! `Vec[T]` surface (`as_bytes`, `substr`, `str ==`, `append`, `push`, `get`).
//!
//! The `str_utils.cnr` fixture decides every result against a known-good literal
//! and traces 1 per passing check, returning the passing-check count. The
//! tree-walking oracle, the MIR interpreter, and the Cranelift native engine
//! (no-opt + opt) must agree byte-for-byte on the all-ones trace and the total.
//! The LLVM `clang -O2` engine covers the fixture transitively through
//! `tests/llvm.rs`'s auto-scanned fifth-engine corpus gate; `tests/aot.rs`'s
//! Cranelift ELF corpus scans it too.
//!
//! UTF-8 safety (P5): the ≥2 multi-byte cases ("héllo,wörld") prove `find`/`split`
//! offsets stay char-boundary-aligned — a mid-char `substr` would fault and
//! diverge from the oracle, so an all-ones trace is itself the proof.

use candor::interp::Run;
use candor::{
    run_source_real, run_source_real_mir, run_source_real_native, run_source_real_native_opt,
    MirRunResult, RunResult,
};

/// The number of checks traced by the fixture's `main` (each a `1` on pass).
const CHECKS: i64 = 45;

fn fixture() -> String {
    let path = format!("{}/tests/fixtures/run/str_utils.cnr", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn oracle(src: &str) -> Run {
    match run_source_real(src) {
        RunResult::Ok(r) => r,
        RunResult::Fault(f) => panic!("oracle faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => {
            panic!("oracle check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("oracle parse error: {}", d.to_json()),
    }
}

fn mir_run(r: MirRunResult, label: &str) -> Run {
    match r {
        MirRunResult::Ok(run) => run,
        MirRunResult::Fault(f) => panic!("{label} faulted: {}", f.to_json()),
        MirRunResult::Unsupported(m) => panic!("{label} unsupported: {m}"),
        MirRunResult::CheckErrors(d) => {
            panic!("{label} check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        MirRunResult::ParseError(d) => panic!("{label} parse error: {}", d.to_json()),
    }
}

#[test]
fn str_utils_all_cases_all_engines() {
    let src = fixture();
    let o = oracle(&src);

    // Every check must have passed: an all-ones trace whose length is the check
    // count, and a return equal to that count. A wrong expected value (or a
    // mid-char `substr` fault under a multi-byte case) would drop a 0 here.
    assert_eq!(o.trace, vec![1i64; CHECKS as usize], "oracle trace (one 1 per passing check)");
    assert_eq!(o.ret, CHECKS, "oracle ret == passing-check count");

    for (label, r) in [
        ("mir", run_source_real_mir(&src)),
        ("native-noopt", run_source_real_native(&src)),
        ("native-opt", run_source_real_native_opt(&src)),
    ] {
        let run = mir_run(r, label);
        assert_eq!(run.trace, o.trace, "{label} trace diverged from oracle");
        assert_eq!(run.ret, o.ret, "{label} ret diverged from oracle");
    }
}
