//! Assignment-drop gate (spec 03 §6.8/§7.5; the 2026-08-03 overwrite-drop
//! divergence dossier): reassigning a needs-drop place destroys the OLD value
//! first — on EVERY engine, in the oracle's exact hook order — and a
//! never-initialized needs-drop local runs NO hook at scope exit (defect P2 /
//! correction D1: the builder's move mask now also carries initialization).
//! Each fixture is proven byte-identical under the tree-walking oracle, the
//! MIR interpreter, and the Cranelift native backend (no-opt + opt); the LLVM
//! `clang -O2` engine covers the same fixtures transitively through
//! `tests/llvm.rs`'s full-corpus fifth-engine gate (they live in `fixtures/run/`).

use std::collections::HashMap;

use candor::interp::Run;
use candor::mir::{self, serial};
use candor::{
    ast, check, diag, generics, real, resolve, run_source_real, run_source_real_mir,
    run_source_real_native, run_source_real_native_opt, MirRunResult, RunResult,
};

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/run/{name}", env!("CARGO_MANIFEST_DIR"));
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

/// Assert every non-LLVM engine reproduces the oracle's `(ret, trace)` byte-exact,
/// and that the oracle itself matches `expected_ret` / `expected_trace`.
fn gate(name: &str, expected_ret: i64, expected_trace: &[i64]) {
    let src = fixture(name);
    let o = oracle(&src);
    assert_eq!(o.ret, expected_ret, "{name}: oracle ret");
    assert_eq!(o.trace, expected_trace, "{name}: oracle trace");
    for (label, r) in [
        ("mir", run_source_real_mir(&src)),
        ("native-noopt", run_source_real_native(&src)),
        ("native-opt", run_source_real_native_opt(&src)),
    ] {
        let run = mir_run(r, label);
        assert_eq!(run.ret, o.ret, "{name}: {label} ret diverged from oracle");
        assert_eq!(run.trace, o.trace, "{name}: {label} trace diverged from oracle");
    }
}

/// The dossier's minimal reproduction: overwrite drops the old value (1), the
/// scalar field store drops nothing, scope exit drops the final value (2).
#[test]
fn gate_assign_drop_overwrite() {
    gate("assign_drop_overwrite.cnr", 0, &[1, 2]);
}

/// Overwrite through a projection: `h.d = ...` drops only the FIELD's old
/// value (1); scope exit recurses into the replacement (9).
#[test]
fn gate_assign_drop_projection() {
    gate("assign_drop_proj.cnr", 0, &[1, 9]);
}

/// First assignment after `let mut p: D;`: no old value, no hook — only the
/// scope exit drops (5). The D1 shape a naive drop-before-assignment breaks.
#[test]
fn gate_assign_drop_uninit_first_assignment() {
    gate("assign_drop_uninit_first.cnr", 0, &[5]);
}

/// Never-initialized needs-drop local: NO hook at scope exit (defect P2 —
/// MIR/native used to run the hook over zeroed storage, tracing [0, 7]).
#[test]
fn gate_assign_drop_never_initialized() {
    gate("assign_drop_never_init.cnr", 0, &[7]);
}

/// `p = wrap(p)`: the RHS moves the old value out, so the reassignment must
/// not drop it — exactly one drop, at scope exit (3). The double-drop guard.
#[test]
fn gate_assign_drop_rhs_moves_old_value() {
    gate("assign_drop_rhs_move.cnr", 0, &[3]);
}

/// Both-arms conditional initialization: each arm is a FIRST init (no drop —
/// the sibling arm's assignment is not on this path), the post-join
/// reassignment drops the arm's value: [1, 3] then [2, 3].
#[test]
fn gate_assign_drop_conditional_init_if() {
    gate("assign_drop_cond_init.cnr", 0, &[1, 3, 2, 3]);
}

/// The same conditional-init shape through match arms.
#[test]
fn gate_assign_drop_conditional_init_match() {
    gate("assign_drop_match_init.cnr", 0, &[1, 3, 2, 3]);
}

/// Overwrite drops inside a loop: each iteration drops the previous value.
#[test]
fn gate_assign_drop_inside_loop() {
    gate("assign_drop_loop.cnr", 0, &[0, 1, 2, 3]);
}

/// A move inside a returning `if` arm must not leak its move mark into the
/// fall-through path's return drop (the arm-state restore in the builder).
#[test]
fn gate_assign_drop_arm_return_no_mark_leak() {
    gate("assign_drop_arm_return.cnr", 0, &[5, 1, 5, 2]);
}

/// Overwriting a Box-typed local frees the OLD box exactly once: the bump
/// allocator's live count returns to 0 on every engine.
#[test]
fn gate_assign_drop_box_overwrite_frees() {
    gate("assign_drop_box_free.cnr", 0, &[20]);
}

/// Loop break-edge move state (review 2026-08-19 B1): a value moved on the
/// breaking arm stays moved after the loop — no drop at the return.
#[test]
fn gate_assign_drop_loop_break_move() {
    gate("assign_drop_loop_break_move.cnr", 0, &[100, 0, 999]);
}

/// The Box variant of B1: a post-loop drop would double-free (live < 0).
#[test]
fn gate_assign_drop_loop_box_break() {
    gate("assign_drop_loop_box_break.cnr", 0, &[7, 0]);
}

/// B2: post-loop reassignment after a moving break must not drop; the in-loop
/// reassignment's own overwrite drop still fires (trace 1).
#[test]
fn gate_assign_drop_loop_reinit_break() {
    gate("assign_drop_loop_reinit_break.cnr", 0, &[1, 102, 2, 3, 90]);
}

/// The Box variant of B2: every box freed exactly once (live == 0).
#[test]
fn gate_assign_drop_loop_box_reinit() {
    gate("assign_drop_loop_box_reinit.cnr", 0, &[102, 203, 0]);
}

/// B1 with the move BEFORE the break test on the same iteration.
#[test]
fn gate_assign_drop_loop_natural_break() {
    gate("assign_drop_loop_natural_break.cnr", 0, &[101, 1, 102, 2, 3, 90]);
}

/// B3: no overwrite drop through a Box auto-deref (`bx.d = ...`) — the oracle
/// drops only direct (all-Field) places.
#[test]
fn gate_assign_drop_box_field_projection() {
    gate("assign_drop_box_field_proj.cnr", 0, &[77, 9]);
}

/// M1: an inner if whose arms all return contributes no state to the outer
/// merge — the post-if reassignment still drops on the surviving path.
#[test]
fn gate_assign_drop_dead_join() {
    gate("assign_drop_dead_join.cnr", 0, &[1, 2, 3, 90, 101, 1, 91]);
}

/// M1 for loops: a break-less loop never reaches the enclosing if-join.
#[test]
fn gate_assign_drop_dead_loop_join() {
    gate("assign_drop_dead_loop_join.cnr", 0, &[1, 2, 3, 90, 101, 1, 91]);
}

/// M2: reassigning `h.d` after `sink(h.d.x)` must not re-drop the moved-out
/// `.x` — the mask is rebased onto the assigned place (oracle and builder).
#[test]
fn gate_assign_drop_partial_move_projection() {
    gate("assign_drop_partial_move_proj.cnr", 0, &[101, 1, 90, 9]);
}

/// The 2026-08-19 review's 81-program loop template: every combination of
/// A..D over {nothing, `sink(p);`, `p = D { id: _ };`} in
/// `loop { A; if i > 0 { B; break; } C; i = i + 1; } D` is either rejected by
/// the shared checker on every path or agrees with the oracle byte-exact on
/// every in-process engine. The reviewer measured 10 divergences and 3 clean
/// regressions on this template before the loop break-state fix.
#[test]
fn gate_assign_drop_loop_template_enumeration() {
    let slot = |k: usize, id: i64| -> String {
        match k {
            0 => String::new(),
            1 => "sink(p);".to_string(),
            _ => format!("p = D {{ id: {id} }};"),
        }
    };
    let mut ran = 0;
    let mut rejected = 0;
    for a in 0..3usize {
        for b in 0..3usize {
            for c in 0..3usize {
                for d in 0..3usize {
                    let src = format!(
                        r#"
struct D {{ id: i64 }}
drop(write self) {{ trace(self.id); }}
fn sink(d: D) -> i64 {{ trace(100 + d.id); return 0; }}
fn go() -> i64 {{
    let mut p: D = D {{ id: 0 }};
    let mut i: i64 = 0;
    loop {{
        {}
        if i > 0 {{ {} break; }}
        {}
        i = i + 1;
    }}
    {}
    return 0;
}}
fn main() -> i64 {{ go(); trace(999); return 0; }}
"#,
                        slot(a, 1),
                        slot(b, 2),
                        slot(c, 3),
                        slot(d, 4)
                    );
                    let tag = format!("A{a} B{b} C{c} D{d}");
                    let o = match run_source_real(&src) {
                        RunResult::Ok(r) => Some((r.ret, r.trace)),
                        RunResult::CheckErrors(_) => None,
                        RunResult::Fault(f) => panic!("{tag}: oracle faulted: {}", f.to_json()),
                        RunResult::ParseError(p) => panic!("{tag}: parse error: {}", p.to_json()),
                    };
                    for (label, r) in [
                        ("mir", run_source_real_mir(&src)),
                        ("native-noopt", run_source_real_native(&src)),
                        ("native-opt", run_source_real_native_opt(&src)),
                    ] {
                        match (&o, r) {
                            (Some((ret, trace)), MirRunResult::Ok(run)) => {
                                assert_eq!(&run.ret, ret, "{tag}: {label} ret diverged from oracle");
                                assert_eq!(&run.trace, trace, "{tag}: {label} trace diverged from oracle");
                            }
                            (None, MirRunResult::CheckErrors(_)) => {}
                            (Some(_), _) => panic!("{tag}: {label} did not run a checker-clean program"),
                            (None, _) => panic!("{tag}: {label} accepted a checker-rejected program"),
                        }
                    }
                    if o.is_some() {
                        ran += 1;
                    } else {
                        rejected += 1;
                    }
                }
            }
        }
    }
    // Pinned population (the reviewer's measurement): a shift means the
    // checker's accepted set changed — re-derive the expected counts.
    assert_eq!((ran, rejected), (37, 44), "template population shifted");
}

/// The serialization boundary carries the new `DropPlace` statement: the
/// overwrite fixture's MIR round-trips byte-stable (serialize -> deserialize
/// -> serialize) and the deserialized program still reproduces the oracle.
#[test]
fn gate_assign_drop_serial_round_trip() {
    let src = fixture("assign_drop_overwrite.cnr");
    let program = real::parse_source(&src).expect("parse");
    assert!(!generics::is_generic_program(&program));
    let diags = check::check_program_real(&program);
    assert!(!diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)));
    let mut resolve_diags = Vec::new();
    let items = resolve::resolve_program(&program, &mut resolve_diags);
    let mut consts = HashMap::new();
    for it in &program.items {
        if let ast::Item::Static(st) = it {
            if let ast::ExprKind::IntLit { value, .. } = &st.value.kind {
                consts.insert(st.name.clone(), *value);
            }
        }
    }
    let mp = mir::lower_checked(&program, &items).expect("lower to MIR");
    let wire1 = serial::serialize(&mp);
    assert!(wire1.contains("dropplace"), "the overwrite fixture must lower a DropPlace");
    let mp2 = serial::deserialize(&wire1).expect("deserialize");
    assert_eq!(wire1, serial::serialize(&mp2), "wire text must be round-trip stable");
    let o = oracle(&src);
    let run = mir::interp::run(&mp2, &items, &consts).expect("deserialized MIR run");
    assert_eq!(run.ret, o.ret, "deserialized MIR ret diverged from oracle");
    assert_eq!(run.trace, o.trace, "deserialized MIR trace diverged from oracle");
}

/// The ONE-SIDED conditional-init shape (`if c { p = ...; } p = ...;`) is a
/// path-DEPENDENT drop point for a needs-drop place, statically rejected as
/// E0309 (§7.5) — the discipline that lets every engine emit the overwrite
/// drop unconditionally, with no runtime flag. All engines share this checker.
#[test]
fn gate_assign_drop_one_sided_conditional_rejected() {
    let src = r#"
struct D { id: i64 }
drop(write self) { trace(self.id); }
fn main() -> i64 {
    let mut p: D;
    if 1 < 2 {
        p = D { id: 1 };
    }
    p = D { id: 3 };
    return 0;
}
"#;
    let codes = |d: &[candor::diag::Diag]| d.iter().map(|x| x.code.clone()).collect::<Vec<_>>();
    match run_source_real(src) {
        RunResult::CheckErrors(d) => {
            assert!(
                d.iter().any(|x| x.code == "E0309"),
                "oracle: expected E0309, got {:?}",
                codes(&d)
            );
        }
        _ => panic!("oracle: expected an E0309 check error, but the program was not rejected"),
    }
    match run_source_real_mir(src) {
        MirRunResult::CheckErrors(d) => {
            assert!(
                d.iter().any(|x| x.code == "E0309"),
                "mir: expected E0309, got {:?}",
                codes(&d)
            );
        }
        _ => panic!("mir: expected an E0309 check error, but the program was not rejected"),
    }
}
