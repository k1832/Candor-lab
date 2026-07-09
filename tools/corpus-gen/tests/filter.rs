//! Filter integration: the toolchain — not the generator — decides keep/reject.
//! A designed-to-fail program is rejected; a well-formed one is kept; and every
//! kept sample really satisfies its recorded expectation.

mod common;

use corpus_gen::manifest::Expected;
use corpus_gen::oracle::{first_code, Oracle};
use corpus_gen::{generate, Config, DEFAULT_TOOLCHAIN_VERSION};
use std::path::Path;

/// A positive candidate whose sentinel is DELIBERATELY wrong must fail the
/// filter's run-to-sentinel gate; the correct program passes it.
#[test]
fn designed_to_fail_positive_is_rejected() {
    let oracle = Oracle::new(common::oracle_bin());
    let dir = common::scratch("filter-pos");
    let file = dir.join("cand.cnr");

    // A well-formed program returning 30.
    std::fs::write(&file, "fn main() -> i64 {\n    let a: i64 = 10;\n    let b: i64 = 20;\n    return a + b;\n}\n").unwrap();
    let check = oracle.check(&file).unwrap();
    assert_eq!(check.code, 0, "should compile clean");
    let run = oracle.run(&file).unwrap();
    assert_eq!(run.stdout.trim(), "30", "true sentinel is 30");
    // The filter predicate: kept iff run-output == predicted sentinel.
    assert_ne!(run.stdout.trim(), "31", "a wrong predicted sentinel would be rejected");
}

/// A program that does not compile is rejected at the check gate.
#[test]
fn uncompilable_is_rejected() {
    let oracle = Oracle::new(common::oracle_bin());
    let dir = common::scratch("filter-broken");
    let file = dir.join("broken.cnr");
    std::fs::write(&file, "fn main() -> i64 { let x: i64 = 5; }\n").unwrap(); // E0810
    let check = oracle.check(&file).unwrap();
    assert_ne!(check.code, 0, "a designed-to-fail program must not pass check");
    assert_eq!(first_code(&check.stdout).as_deref(), Some("E0810"));
}

/// Every sample the pipeline KEPT genuinely satisfies its recorded expectation
/// when re-verified against the toolchain — the filter did its job.
#[test]
fn kept_samples_satisfy_their_expectation() {
    let oracle = Oracle::new(common::oracle_bin());
    let dir = common::scratch("filter-corpus");
    let cfg = Config {
        seed: 13,
        positive: 10,
        negative: 15,
        out_dir: dir.clone(),
        oracle: oracle.clone(),
        toolchain_version: DEFAULT_TOOLCHAIN_VERSION.to_string(),
        max_attempts: 5000,
    };
    let manifest = generate(&cfg).unwrap();
    assert_eq!(manifest.samples.len(), 25);

    for s in &manifest.samples {
        let path = dir.join(&s.file);
        assert!(Path::new(&path).exists(), "missing {}", s.file);
        match &s.expected {
            Expected::Sentinel { value } => {
                let check = oracle.check(&path).unwrap();
                assert_eq!(check.code, 0, "{} should compile", s.id);
                let run = oracle.run(&path).unwrap();
                assert_eq!(run.stdout.trim(), value.to_string(), "{} sentinel", s.id);
            }
            Expected::Diagnostic { code } => {
                let check = oracle.check(&path).unwrap();
                assert_ne!(check.code, 0, "{} should be rejected by check", s.id);
                assert_eq!(first_code(&check.stdout).as_deref(), Some(code.as_str()), "{} code", s.id);
            }
        }
    }
}
