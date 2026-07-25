//! JSON parser + pretty/compact printer dogfood program (`json.cnr`): full
//! RFC 8259 grammar over the native `Vec`/`String` surface, errors as values
//! (`PRes::Err { code, pos }`, never a fault), and the round-trip property
//! `parse(print(v)) == v` for both printers on every valid document.
//!
//! Number policy (documented in the fixture header): integer-form tokens that
//! fit i64 are canonical i64 values; fractions, exponents, and beyond-i64
//! integers keep their raw lexeme and print verbatim.
//!
//! The fixture traces 1 per passing check (valid-doc round-trips, pinned
//! printer bytes, exact malformed-input `(code, pos)` values) plus a final
//! checksum folded over every printed byte, and returns the passing-check
//! count. This test pins the oracle's values and requires the MIR interpreter
//! and both Cranelift JIT tiers to agree byte-for-byte; `tests/aot.rs` and
//! `tests/llvm.rs` cover the linked-ELF engines through their auto-scanned
//! `run/` corpus gates, as does `tests/stage_d.rs`.

use candor::interp::Run;
use candor::{
    run_source_real, run_source_real_mir, run_source_real_native, run_source_real_native_opt,
    MirRunResult, RunResult,
};

/// The number of pass/fail checks traced by the fixture's `main` (each a `1`
/// on pass): 12 valid documents x 3 (parse + compact and pretty round-trips),
/// 7 pinned printer outputs, 25 malformed documents.
const CHECKS: i64 = 68;

fn fixture() -> String {
    let path = format!("{}/tests/fixtures/run/json.cnr", env!("CARGO_MANIFEST_DIR"));
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
fn json_all_cases_all_engines() {
    let src = fixture();
    let o = oracle(&src);

    // Every check must have passed: `CHECKS` ones, then the printed-byte
    // checksum, and a return equal to the check count. A wrong (code, pos)
    // error value, a round-trip mismatch, or a single diverging printed byte
    // drops a 0 (or shifts the checksum) here.
    assert_eq!(o.trace.len(), CHECKS as usize + 1, "trace = checks + checksum");
    assert_eq!(&o.trace[..CHECKS as usize], vec![1i64; CHECKS as usize], "oracle checks all pass");
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
