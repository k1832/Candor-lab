//! Golden run tests: every runnable §11 basket fixture must parse + fully check +
//! execute to completion, returning its asserted sentinel value.

use candor_proto::{run_source, RunResult};

fn run_ret(name: &str) -> i64 {
    let path = format!("{}/tests/fixtures/run/{name}", env!("CARGO_MANIFEST_DIR"));
    let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    match run_source(&src) {
        RunResult::Ok(r) => r.ret,
        RunResult::Fault(f) => panic!("{name} faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!(
            "{name} check errors: {:?}",
            d.iter().map(|x| &x.code).collect::<Vec<_>>()
        ),
        RunResult::ParseError(d) => panic!("{name} parse error: {}", d.to_json()),
    }
}

#[test]
fn run_11_1_allocator() {
    assert_eq!(run_ret("11_1_allocator.cn"), 1234);
}
#[test]
fn run_11_2_scheduler() {
    assert_eq!(run_ret("11_2_scheduler.cn"), 42);
}
#[test]
fn run_11_3_mmio() {
    assert_eq!(run_ret("11_3_mmio.cn"), 42);
}
#[test]
fn run_11_4_parser() {
    assert_eq!(run_ret("11_4_parser.cn"), 17);
}
#[test]
fn run_11_5_arena() {
    assert_eq!(run_ret("11_5_arena.cn"), 5);
}
