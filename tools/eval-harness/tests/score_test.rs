//! Runner tests: the harness scored against known-good and known-defective
//! submissions, plus task-set and stage-classification checks. Requires a built
//! `candor-proto` (set `$CANDOR_PROTO`, or the sibling `prototype/target/...`).

use std::path::{Path, PathBuf};

use eval_harness::score::Stage;
use eval_harness::task::{load_tasks, AnchorKind, Category};
use eval_harness::{config_under_root, run_scoring, Report};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Locate the candor-proto oracle for the tests.
fn candor_bin() -> String {
    if let Ok(p) = std::env::var("CANDOR_PROTO") {
        return p;
    }
    let manifest = root();
    for cand in [
        "../../prototype/target/debug/candor-proto",
        "../../prototype/target/release/candor-proto",
    ] {
        let p = manifest.join(cand);
        if p.exists() {
            return p.display().to_string();
        }
    }
    panic!("candor-proto not found; run `cargo build` in prototype/ or set CANDOR_PROTO");
}

fn score(sub_subdir: &str) -> Report {
    let cfg = config_under_root(
        &root(),
        root().join("tests").join(sub_subdir),
        candor_bin(),
        1,
    );
    run_scoring(&cfg).expect("scoring should complete")
}

fn find<'a>(r: &'a Report, id: &str) -> &'a eval_harness::TaskResult {
    r.tasks.iter().find(|t| t.id == id).expect("task present")
}

#[test]
fn task_set_composition() {
    let tasks = load_tasks(&root().join("tasks")).expect("tasks load");
    assert_eq!(tasks.len(), 23, "12 seed + 11 graduation tasks");
    let generate = tasks.iter().filter(|t| t.category == Category::Generate).count();
    let repair = tasks.iter().filter(|t| t.category == Category::Repair).count();
    assert_eq!(generate, 16, "8 seed + 8 graduation generation tasks");
    assert_eq!(repair, 7, "4 seed + 3 graduation repair tasks");
    let grad = tasks.iter().filter(|t| t.id.starts_with("grad_")).count();
    assert_eq!(grad, 8, "8 grad_* generation tasks");

    // Every generate task carries a run_sentinel anchor with a battery; every
    // repair task carries a diagnostic-resolved anchor and a real diagnostic.
    for t in &tasks {
        match t.category {
            Category::Generate => {
                assert_eq!(t.anchor.kind, AnchorKind::RunSentinel, "{}", t.id);
                assert!(t.anchor.battery_file.is_some(), "{}", t.id);
            }
            Category::Repair => {
                assert_eq!(t.anchor.kind, AnchorKind::DiagnosticResolved, "{}", t.id);
                let pm = &t.prompt_material;
                assert!(pm.given_program.is_some(), "{} has a buggy program", t.id);
                let diag = pm.given_diagnostic.as_ref().expect("repair diagnostic");
                assert!(diag.get("code").is_some(), "{} diagnostic has a code", t.id);
            }
            Category::Explain => {}
        }
        t.validate().expect("anchor coherent");
    }
}

#[test]
fn good_submissions_all_pass() {
    let report = score("submissions_good");
    assert_eq!(report.aggregate.total, 23);
    assert_eq!(report.aggregate.passed, 23, "all known-good submissions pass");
    assert!((report.aggregate.first_attempt_rate - 1.0).abs() < f64::EPSILON);
    assert!(report.all_passed());
    for t in &report.tasks {
        assert!(t.pass, "{} should pass but got {:?}", t.id, t.stage);
        assert!(t.stage.is_none());
        assert!(t.feedback_diagnostic.is_none());
    }
}

#[test]
fn defective_submissions_fail_by_stage() {
    let report = score("submissions_bad");

    // Check-stage: undefined name -> E0103, with feedback for the repair loop.
    let align = find(&report, "gen_align_up");
    assert!(!align.pass);
    assert_eq!(align.stage, Some(Stage::Check));
    assert_eq!(align.failure_code.as_deref(), Some("E0103"));
    assert!(align.feedback_diagnostic.is_some(), "check failures feed back a diagnostic");

    // Wrong-sentinel: compiles + runs but returns 18 instead of 10.
    let min = find(&report, "gen_min_i64");
    assert!(!min.pass);
    assert_eq!(min.stage, Some(Stage::WrongSentinel));
    assert_eq!(min.expected_sentinel.as_deref(), Some("10"));
    assert_eq!(min.actual_sentinel.as_deref(), Some("18"));

    // Parse-stage: a syntax break -> P0001.
    let mv = find(&report, "repair_move_e0301");
    assert!(!mv.pass);
    assert_eq!(mv.stage, Some(Stage::Parse));
    assert_eq!(mv.failure_code.as_deref(), Some("P0001"));
    assert!(mv.feedback_diagnostic.is_some());

    // Graduation control: the OBVIOUS marker fix for repair_alloc_restructure
    // (adding `alloc`) is rejected at the non-`alloc` fn-pointer slot -> E0402.
    let alloc_r = find(&report, "repair_alloc_restructure");
    assert!(!alloc_r.pass);
    assert_eq!(alloc_r.stage, Some(Stage::Check));
    assert_eq!(alloc_r.failure_code.as_deref(), Some("E0402"));

    // Graduation control: the natural bare-`+` wrap counter compiles but FAULTS
    // at runtime (checked overflow) -> Run stage, not WrongSentinel.
    let wrap = find(&report, "grad_wrap_counter");
    assert!(!wrap.pass);
    assert_eq!(wrap.stage, Some(Stage::Run));

    // Graduation control: a single-FIFO scheduler (ignores priority) trips the
    // hidden battery's asserts -> Run fault.
    let sched = find(&report, "grad_sched_pick");
    assert!(!sched.pass);
    assert_eq!(sched.stage, Some(Stage::Run));

    // Tasks with no submission file are Missing, not silently absent.
    let missing = find(&report, "gen_arena_push_get");
    assert!(!missing.pass);
    assert_eq!(missing.stage, Some(Stage::Missing));
}

#[test]
fn run_fault_is_its_own_stage() {
    // A submission that compiles clean but violates the hidden battery's asserts
    // must classify as Run (a fault), not WrongSentinel or Check.
    let dir = std::env::temp_dir().join(format!("eh-fault-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("gen_align_up.cnr"),
        "fn align_up(x: usize, align: usize) -> usize {\n    return x;\n}\n",
    )
    .unwrap();

    let cfg = config_under_root(&root(), dir.clone(), candor_bin(), 1);
    let report = run_scoring(&cfg).expect("scoring completes");
    let r = find(&report, "gen_align_up");
    assert!(!r.pass);
    assert_eq!(r.stage, Some(Stage::Run));
    let fb = r.feedback_diagnostic.as_ref().expect("fault feeds back");
    assert!(fb.get("kind").is_some(), "fault carries a kind");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn report_round_label_is_the_slope_substrate() {
    // The round label rides through the report so an operator can compare
    // round-1 (first attempt) against round-2 (post-repair) pass rates.
    let cfg = config_under_root(&root(), root().join("tests/submissions_good"), candor_bin(), 2);
    let report = run_scoring(&cfg).unwrap();
    assert_eq!(report.round, 2);
    let json = report.to_json_pretty();
    assert!(json.contains("\"first_attempt_rate\""));
    assert!(json.contains("\"round\": 2"));
}

#[test]
fn tasks_dir_is_discoverable() {
    // Guard against a missing tasks tree regressing to a silent empty run.
    assert!(root().join("tasks").is_dir());
    assert!(Path::new(&candor_bin()).exists() || std::env::var("CANDOR_PROTO").is_ok());
}

#[test]
fn graduation_reference_solutions_pass() {
    // The graduation answer key (reference solutions authored against the specs)
    // must pass every graduation task. Seed tasks are absent here (Missing) and
    // are asserted elsewhere; here we only require each graduation task to pass.
    let report = score("reference_solutions");
    let grad_ids = [
        "grad_intrusive_container_of",
        "grad_bump_box_compute",
        "grad_out_divmod",
        "grad_coalesce_run",
        "grad_sched_pick",
        "grad_mmio_recover",
        "grad_xorshift",
        "grad_wrap_counter",
        "repair_cascade_parse_move",
        "repair_alloc_restructure",
        "repair_writepath_return",
    ];
    for id in grad_ids {
        let t = find(&report, id);
        assert!(t.pass, "graduation reference `{}` should pass but got {:?}", id, t.stage);
    }
}
