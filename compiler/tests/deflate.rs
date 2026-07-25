//! Raw DEFLATE (RFC 1951) decoder dogfood program (`deflate.cnr`): stored,
//! fixed-Huffman, and dynamic-Huffman blocks over the full LZ77
//! length/distance alphabet, multi-block streams, and errors as values
//! (`IRes::Err { code, pos }`, never a fault) with a specific code per
//! malformed-input class.
//!
//! The embedded compressed vectors were produced by CPython's zlib (raw
//! deflate, wbits=-15) at levels 0/1/6/9 — a stored block, a fixed block, an
//! overlapping distance-1 run, a dynamic block, a Z_FULL_FLUSH multi-block
//! stream, and a 4000-byte pattern — and the corrupt/truncated variants
//! (reserved block type, bad Huffman symbol, distance too far back,
//! truncation, code-length metadata errors, bad stored NLEN) are each
//! verified to be rejected by zlib as well (provenance in the fixture
//! header).
//!
//! The fixture traces 1 per passing check plus a final checksum folded over
//! every decoded byte, and returns the passing-check count. This test pins
//! the oracle's values and requires the MIR interpreter and both Cranelift
//! JIT tiers to agree byte-for-byte; `tests/aot.rs` and `tests/llvm.rs`
//! cover the linked-ELF engines through their auto-scanned `run/` corpus
//! gates, as does `tests/stage_d.rs`.

use candor::interp::Run;
use candor::{
    run_source_real, run_source_real_mir, run_source_real_native, run_source_real_native_opt,
    MirRunResult, RunResult,
};

/// The number of pass/fail checks traced by the fixture's `main` (each a `1`
/// on pass): 6 valid streams decoded and compared byte-for-byte, 12 malformed
/// streams with exact error codes.
const CHECKS: i64 = 18;

fn fixture() -> String {
    let path = format!("{}/tests/fixtures/run/deflate.cnr", env!("CARGO_MANIFEST_DIR"));
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
fn deflate_all_cases_all_engines() {
    let src = fixture();
    let o = oracle(&src);

    // Every check must have passed: `CHECKS` ones, then the decoded-byte
    // checksum, and a return equal to the check count. A single diverging
    // decoded byte, a wrong length, or a wrong error code drops a 0 (or
    // shifts the checksum) here.
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
