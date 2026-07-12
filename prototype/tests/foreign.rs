//! The foreign boundary (design 0011): boundary/extern/trust/export parsing and
//! placement, recursive C-mappability, the `foreign` effect partition and its
//! discharge rule, foreign-call-in-unsafe gating, the shim registry running an
//! extern call identically on both engines, the `no_foreign_runtime` fault
//! identity, and the `candor audit` golden output.

use candor_proto::diag::Severity;
use candor_proto::interp::FaultKind;
use candor_proto::{
    check_source_real, run_source_real, run_source_real_mir, MirRunResult, RunResult,
};

fn err_codes(src: &str) -> Vec<String> {
    match check_source_real(src) {
        Ok(diags) => diags
            .into_iter()
            .filter(|d| d.severity == Severity::Error)
            .map(|d| d.code)
            .collect(),
        Err(parse) => vec![parse.code],
    }
}

const DISCHARGE: &str = r#"boundary
extern "C" {
    fn c_id(x: usize) foreign -> usize
        trust "test shim: pure identity, retains nothing" {
            no_retain(x),
        };
}
pub fn wrap(x: usize) -> usize {
    unsafe "c_id trust discharged" {
        return c_id(x);
    }
}
fn main() -> i64 { return 0; }
"#;

// ---- 1. boundary parse + discharge checks clean --------------------------
#[test]
fn boundary_extern_trust_discharge_checks_clean() {
    assert!(err_codes(DISCHARGE).is_empty(), "{:?}", err_codes(DISCHARGE));
}

// ---- 2. E1101 placement: extern outside a boundary file ------------------
#[test]
fn extern_outside_boundary_is_e1101() {
    let src = r#"extern "C" {
    fn c_id(x: usize) foreign -> usize trust "j" { no_retain(x), };
}
fn main() -> i64 { return 0; }
"#;
    assert!(err_codes(src).contains(&"E1101".to_string()));
}

#[test]
fn export_outside_boundary_is_e1101() {
    let src = r#"pub fn f(x: usize) -> usize { return x; }
export "C" fn c_f(x: usize) -> usize = f;
fn main() -> i64 { return 0; }
"#;
    assert!(err_codes(src).contains(&"E1101".to_string()));
}

// ---- 3. E1102 mappability: enum-in-struct, Box, slice --------------------
#[test]
fn enum_in_struct_param_is_e1102() {
    let src = r#"boundary
enum Color { Red, Green }
struct Wrap { c: Color, n: usize }
extern "C" {
    fn c_take(w: Wrap) foreign -> usize trust "j" { no_retain(w), };
}
fn main() -> i64 { return 0; }
"#;
    assert!(err_codes(src).contains(&"E1102".to_string()));
}

#[test]
fn box_param_is_e1102() {
    let src = r#"boundary
extern "C" {
    fn c_take(b: Box u8) foreign -> usize trust "j" { no_retain(b), };
}
fn main() -> i64 { return 0; }
"#;
    assert!(err_codes(src).contains(&"E1102".to_string()));
}

#[test]
fn bare_enum_return_is_e1102() {
    let src = r#"boundary
enum Color { Red, Green }
extern "C" {
    fn c_take(x: usize) foreign -> Color trust "j" { no_retain(x), };
}
fn main() -> i64 { return 0; }
"#;
    assert!(err_codes(src).contains(&"E1102".to_string()));
}

// ---- 4. foreign propagation + discharge negatives ------------------------
#[test]
fn undischarged_wrapper_not_marked_is_e1103() {
    let src = r#"boundary
extern "C" {
    fn c_raw(x: usize) foreign -> usize;
}
pub fn wrap(x: usize) -> usize {
    unsafe "no trust" { return c_raw(x); }
}
fn main() -> i64 { return 0; }
"#;
    assert!(err_codes(src).contains(&"E1103".to_string()));
}

#[test]
fn undischarged_wrapper_marked_foreign_ok() {
    let src = r#"boundary
extern "C" {
    fn c_raw(x: usize) foreign -> usize;
}
pub fn wrap(x: usize) foreign -> usize {
    unsafe "escapes: raw pointer, cannot discharge" { return c_raw(x); }
}
fn main() -> i64 { return 0; }
"#;
    assert!(err_codes(src).is_empty(), "{:?}", err_codes(src));
}

#[test]
fn nonboundary_caller_of_extern_cannot_discharge() {
    // A wrapper in a boundary file whose extern lacks trust must propagate: even
    // marking one extern with trust does not discharge when another lacks it.
    let src = r#"boundary
extern "C" {
    fn c_a(x: usize) foreign -> usize trust "j" { no_retain(x), };
    fn c_b(x: usize) foreign -> usize;
}
pub fn wrap(x: usize) -> usize {
    unsafe "mixed" {
        let a: usize = c_a(x);
        return c_b(a);
    }
}
fn main() -> i64 { return 0; }
"#;
    assert!(err_codes(src).contains(&"E1103".to_string()));
}

// ---- 5. foreign call outside unsafe --------------------------------------
#[test]
fn foreign_call_outside_unsafe_is_e0501() {
    let src = r#"boundary
extern "C" {
    fn c_id(x: usize) foreign -> usize trust "j" { no_retain(x), };
}
pub fn wrap(x: usize) foreign -> usize {
    return c_id(x);
}
fn main() -> i64 { return 0; }
"#;
    assert!(err_codes(src).contains(&"E0501".to_string()));
}

// ---- 6. export mappability + reference ------------------------------------
#[test]
fn export_unknown_fn_is_e1107() {
    let src = r#"boundary
export "C" fn c_f(x: usize) -> u32 = missing;
fn main() -> i64 { return 0; }
"#;
    assert!(err_codes(src).contains(&"E1107".to_string()));
}

#[test]
fn export_unmappable_signature_is_e1102() {
    let src = r#"boundary
enum Color { Red, Green }
pub fn f(c: Color) -> usize { return 0; }
export "C" fn c_f(c: Color) -> usize = f;
fn main() -> i64 { return 0; }
"#;
    assert!(err_codes(src).contains(&"E1102".to_string()));
}

#[test]
fn export_wellformed_checks_clean() {
    let src = r#"boundary
pub fn checksum(x: usize) -> u32 { return conv u32 (x); }
export "C" fn candor_checksum(x: usize) -> u32 = checksum;
fn main() -> i64 { return 0; }
"#;
    assert!(err_codes(src).is_empty(), "{:?}", err_codes(src));
}

// ---- 7. trust vocabulary -------------------------------------------------
#[test]
fn unknown_trust_predicate_is_e1105() {
    let src = r#"boundary
extern "C" {
    fn c_id(x: usize) foreign -> usize trust "j" { bogus_pred(x), };
}
fn main() -> i64 { return 0; }
"#;
    assert!(err_codes(src).contains(&"E1105".to_string()));
}

#[test]
fn empty_trust_justification_is_e1106() {
    let src = r#"boundary
extern "C" {
    fn c_id(x: usize) foreign -> usize trust "" { no_retain(x), };
}
fn main() -> i64 { return 0; }
"#;
    assert!(err_codes(src).contains(&"E1106".to_string()));
}

// ---- 8. shim-backed extern call, engine equality -------------------------
const SHIM_PROG: &str = r#"boundary
extern "C" {
    fn shim_double(x: usize) foreign -> usize
        trust "test: pure doubling, retains nothing" {
            no_retain(x),
        };
}
pub fn call_double(x: usize) -> usize {
    unsafe "discharged: pure function" {
        return shim_double(x);
    }
}
fn main() -> i64 {
    let a: usize = call_double(21);
    trace(conv i64 (a));
    return conv i64 (a);
}
"#;

#[test]
fn shim_extern_call_runs_equal_on_both_engines() {
    candor_proto::foreign::register("shim_double", |args, _mem| args[0] * 2);

    let tree = match run_source_real(SHIM_PROG) {
        RunResult::Ok(r) => r,
        other => panic!("tree engine did not run: {:?}", debug_run(&other)),
    };
    let mir = match run_source_real_mir(SHIM_PROG) {
        MirRunResult::Ok(r) => r,
        other => panic!("mir engine did not run: {:?}", debug_mir(&other)),
    };

    candor_proto::foreign::unregister("shim_double");

    assert_eq!(tree.ret, 42);
    assert_eq!(mir.ret, 42);
    assert_eq!(tree.ret, mir.ret, "engine return divergence");
    assert_eq!(tree.trace, mir.trace, "engine trace divergence");
    assert_eq!(tree.trace, vec![42]);
}

// ---- 9. no_foreign_runtime fault identity across engines -----------------
const NO_SHIM_PROG: &str = r#"boundary
extern "C" {
    fn nf_never_registered(x: usize) foreign -> usize
        trust "no shim exists for this symbol" {
            no_retain(x),
        };
}
pub fn wrap(x: usize) -> usize {
    unsafe "discharged" { return nf_never_registered(x); }
}
fn main() -> i64 { return conv i64 (wrap(1)); }
"#;

#[test]
fn no_foreign_runtime_fault_identical_across_engines() {
    let tree = match run_source_real(NO_SHIM_PROG) {
        RunResult::Fault(f) => f,
        other => panic!("tree: expected fault, got {:?}", debug_run(&other)),
    };
    let mir = match run_source_real_mir(NO_SHIM_PROG) {
        MirRunResult::Fault(f) => f,
        other => panic!("mir: expected fault, got {:?}", debug_mir(&other)),
    };
    assert_eq!(tree.kind, FaultKind::NoForeignRuntime);
    assert_eq!(mir.kind, FaultKind::NoForeignRuntime);
    assert_eq!(tree.kind, mir.kind);
    assert_eq!(tree.span, mir.span, "fault span divergence across engines");
}

// ---- 10. audit golden output over a boundary fixture tree ----------------
#[test]
fn audit_golden_over_boundary_tree() {
    let dir = format!("{}/tests/fixtures/ffi_audit", env!("CARGO_MANIFEST_DIR"));
    let got = candor_proto::audit::audit_path(std::path::Path::new(&dir))
        .expect("audit succeeds");
    let golden_path = format!("{}/tests/fixtures/ffi_audit.golden.json", env!("CARGO_MANIFEST_DIR"));
    let golden = std::fs::read_to_string(&golden_path).expect("read golden");
    assert_eq!(got.trim(), golden.trim(), "audit output drifted from golden");
}

// ---- 11. audit keeps full teeth on a GENERIC boundary module --------------
// Regression (P17): a boundary module that is ALSO generic used to route
// through the generic checker, which DROPPED the already-computed foreign
// effect-reach — so `candor audit` reported an EMPTY discharge/reach exactly
// where a generic Res-typed I/O wrapper would live. The audit must now name the
// externs, their trust predicates, and classify each generic wrapper's reach
// with the same fidelity as the non-generic path (design 0011 §2, §6).
#[test]
fn audit_generic_boundary_keeps_effect_reach() {
    let dir = format!("{}/tests/fixtures/ffi_audit_generic", env!("CARGO_MANIFEST_DIR"));
    let got = candor_proto::audit::audit_path(std::path::Path::new(&dir))
        .expect("audit succeeds");

    // Externs and every trust predicate survive the generic path.
    assert!(got.contains("\"name\": \"c_len\""), "missing extern c_len:\n{got}");
    assert!(got.contains("\"name\": \"c_raw\""), "missing extern c_raw:\n{got}");
    assert!(got.contains("valid_nul_terminated"), "missing trust predicate:\n{got}");
    assert!(got.contains("no_retain"), "missing trust predicate:\n{got}");

    // Effect reach classifies both GENERIC wrappers, not an empty report.
    assert!(
        got.contains("\"function\": \"main::safe_len\"")
            && got.contains("\"status\": \"discharges foreign\""),
        "generic wrapper `safe_len` lost its `discharges` teeth:\n{got}"
    );
    assert!(
        got.contains("\"function\": \"main::thin_raw\"")
            && got.contains("\"status\": \"propagates foreign (undischarged)\""),
        "generic wrapper `thin_raw` lost its `propagates` teeth:\n{got}"
    );
    assert!(
        got.contains("\"undischarged_foreign_wrappers\": 1"),
        "summary undercounts the propagating generic wrapper:\n{got}"
    );
}

fn debug_run(r: &RunResult) -> String {
    match r {
        RunResult::Ok(run) => format!("Ok(ret={})", run.ret),
        RunResult::Fault(f) => format!("Fault({})", f.to_json()),
        RunResult::CheckErrors(d) => format!("CheckErrors({:?})", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        RunResult::ParseError(d) => format!("ParseError({})", d.to_json()),
    }
}

fn debug_mir(r: &MirRunResult) -> String {
    match r {
        MirRunResult::Ok(run) => format!("Ok(ret={})", run.ret),
        MirRunResult::Fault(f) => format!("Fault({})", f.to_json()),
        MirRunResult::CheckErrors(d) => format!("CheckErrors({:?})", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        MirRunResult::ParseError(d) => format!("ParseError({})", d.to_json()),
        MirRunResult::Unsupported(s) => format!("Unsupported({s})"),
    }
}
