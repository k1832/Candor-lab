//! Native line splitting (design 0013 file-I/O layer): `split_lines` byte-scans a
//! native `String`'s UTF-8 bytes for the ASCII newline (10) and builds one OWNED
//! `String` per line into a `Vec[String]`, reusing the native `String` /
//! `Vec[T]` / `str_from_unchecked` intrinsics already lowered inline in both
//! backends. The tree-walking oracle, the MIR interpreter, and the Cranelift
//! native engine (no-opt + opt) must agree byte-for-byte on the traced line
//! count, per-line byte lengths and bytes, and the `fold`-composed length sum.
//! The LLVM `clang -O2` engine covers this fixture transitively through
//! `tests/llvm.rs`'s auto-scanned fifth-engine corpus gate; the Cranelift ELF
//! corpus in `tests/aot.rs` scans it too.
//!
//! Convention (asserted below): a trailing newline does NOT yield a final empty
//! line ("a\nb\n" -> ["a","b"]); an interior empty line IS preserved
//! ("a\n\nb" -> ["a","","b"]); the empty string yields an EMPTY Vec.

use candor::interp::Run;
use candor::{
    run_source_real, run_source_real_mir, run_source_real_native, run_source_real_native_opt,
    MirRunResult, RunResult,
};

fn fixture() -> String {
    let path = format!("{}/tests/fixtures/run/split_lines.cnr", env!("CARGO_MANIFEST_DIR"));
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
fn split_lines_all_cases_all_engines() {
    let src = fixture();
    let o = oracle(&src);
    // Six cases in order, each `dump_vec`ed as [count, (len, bytes...)*]:
    //   "a\nb\n"   -> ["a","b"]        : 2, (1,97), (1,98)          — trailing newline
    //   "a\nb"     -> ["a","b"]        : 2, (1,97), (1,98)          — no trailing newline
    //   "a\n\nb"   -> ["a","","b"]     : 3, (1,97), (0), (1,98)     — interior empty line
    //   ""         -> []              : 0                          — empty string
    //   "abc"      -> ["abc"]          : 1, (3,97,98,99)            — single line, no newline
    // then the compose check: fold-sum of ["hello","world","!"] lengths = 5+5+1 = 11.
    assert_eq!(
        o.trace,
        vec![
            2, 1, 97, 1, 98, // "a\nb\n"
            2, 1, 97, 1, 98, // "a\nb"
            3, 1, 97, 0, 1, 98, // "a\n\nb"
            0, // ""
            1, 3, 97, 98, 99, // "abc"
            11, // fold-sum of line lengths
        ],
        "oracle trace"
    );
    assert_eq!(o.ret, 11, "oracle ret (the folded line-length sum)");

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
