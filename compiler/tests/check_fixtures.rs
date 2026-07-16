//! Golden: every design 0001 §11 example, in its self-contained CHECKABLE form,
//! must pass the Stage 2 checker with zero diagnostics.

use candor::check_source;

fn assert_checks(name: &str) {
    let path = format!("{}/tests/fixtures/check/{name}", env!("CARGO_MANIFEST_DIR"));
    let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let diags = check_source(&src).expect("parse ok");
    assert!(
        diags.is_empty(),
        "{name} produced diagnostics: {:?}",
        diags.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}

#[test]
fn check_11_1_allocator() {
    assert_checks("11_1_allocator.cn");
}
#[test]
fn check_11_2_scheduler() {
    assert_checks("11_2_scheduler.cn");
}
#[test]
fn check_11_3_mmio() {
    assert_checks("11_3_mmio.cn");
}
#[test]
fn check_11_4_parser() {
    assert_checks("11_4_parser.cn");
}
#[test]
fn check_11_5_arena() {
    assert_checks("11_5_arena.cn");
}

// ---- migrated real-syntax (`.cnr`) siblings (design 0006 §5; dual-suite) ----
// The P15-migrated fixtures must also check clean under the real front-end.

use candor::check_source_real;

fn assert_checks_real(name: &str) {
    let path = format!("{}/tests/fixtures/check/{name}", env!("CARGO_MANIFEST_DIR"));
    let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let diags = check_source_real(&src).expect("parse ok");
    assert!(
        diags.is_empty(),
        "{name} produced diagnostics: {:?}",
        diags.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}

#[test]
fn check_real_11_1_allocator() {
    assert_checks_real("11_1_allocator.cnr");
}
#[test]
fn check_real_11_2_scheduler() {
    assert_checks_real("11_2_scheduler.cnr");
}
#[test]
fn check_real_11_3_mmio() {
    assert_checks_real("11_3_mmio.cnr");
}
#[test]
fn check_real_11_4_parser() {
    assert_checks_real("11_4_parser.cnr");
}
#[test]
fn check_real_11_5_arena() {
    assert_checks_real("11_5_arena.cnr");
}
