//! Native `String` gate (design 0013, Path A): the `New` / `StringAppend` /
//! `StringAsStr` collection intrinsics lowered inline in BOTH native backends,
//! mirroring `mir::interp::collection_op` byte-for-byte. `string_new` over a
//! reclaiming free-list allocator + `append`s that force `string_reserve` growth
//! across the initial capacity must reproduce the oracle's observable trace `θ`
//! and byte length identically under the MIR interpreter and the Cranelift
//! native engine (no-opt + opt). The LLVM `clang -O2` engine covers this fixture
//! transitively through `tests/llvm.rs`'s full-corpus fifth-engine gate.

use candor_proto::interp::Run;
use candor_proto::{
    run_source_real, run_source_real_mir, run_source_real_native, run_source_real_native_opt,
    MirRunResult, RunResult,
};

fn fixture() -> String {
    let path = format!("{}/tests/fixtures/run/string_native.cnr", env!("CARGO_MANIFEST_DIR"));
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
fn string_native_new_append_as_str_all_engines() {
    let src = fixture();
    let o = oracle(&src);
    // Empty (0), single append "hi" (2, 'h'=104, 'i'=105), then the grown
    // 20-byte "abcde...pqrst" (20, 'a'=97, 'k'=107, 't'=116).
    assert_eq!(o.trace, vec![0, 2, 104, 105, 20, 97, 107, 116], "oracle trace");
    assert_eq!(o.ret, 20, "oracle ret");

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
