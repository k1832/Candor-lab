//! Stage C2 gate (design 0008 §2/§2.4; design 0010 §3) — closing Stage C's two
//! named residuals so the two-hash incremental architecture is fully real:
//!
//!   (a) THE BRUTAL PROOF — signature-only import stubs. A body edit to a
//!       downstream module re-checks it against its imports' interface ARTIFACTS
//!       (signature-only stubs), never their source: we prove it by **deleting an
//!       upstream module's source after caching** and asserting the rebuild still
//!       succeeds (an upstream re-parse would fail on the missing file).
//!   (b) PER-INSTANTIATION CODEGEN CACHE. A non-generic body edit reuses every
//!       cached instantiation of untouched generics; a generic BODY edit
//!       invalidates EXACTLY its own instantiations and never cascades analysis.
//!   (c) all Stage-C assertions still hold (see `stage_c.rs`, unchanged).
//!   (d) DIFFERENTIAL EQUIVALENCE — a tree checked via stubs yields the same
//!       diagnostics as the whole-program merge check, on positive AND negative
//!       fixtures.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use candor::build::{self, BuildReport};
use candor::diag::{Diag, Severity};
use candor::{check, modules};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR")))
}

fn temp_copy(name: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dst = std::env::temp_dir().join(format!("candor_stage_c2_{}_{}", std::process::id(), n));
    let _ = std::fs::remove_dir_all(&dst);
    copy_tree(&fixture(name), &dst);
    dst
}

fn copy_tree(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let p = entry.path();
        let target = dst.join(entry.file_name());
        if p.is_dir() {
            if p.file_name().and_then(|s| s.to_str()) == Some(".candor-cache") {
                continue;
            }
            copy_tree(&p, &target);
        } else {
            std::fs::copy(&p, &target).unwrap();
        }
    }
}

fn build(dir: &Path) -> BuildReport {
    let r = build::build_dir(dir).expect("build should not hard-error");
    assert!(r.ok(), "build had errors: {:?}", r.diags.iter().map(|d| &d.code).collect::<Vec<_>>());
    r
}

fn write(dir: &Path, rel: &str, contents: &str) {
    std::fs::write(dir.join(rel), contents).unwrap();
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

// ===========================================================================
// (a) THE BRUTAL PROOF — signature-only import stubs, upstream source deleted
// ===========================================================================

#[test]
fn gate_a2_body_edit_never_parses_upstream_source() {
    // build_c: main -> {a, b} -> base. `base` is upstream of `a`.
    let dir = temp_copy("build_c");
    let first = build(&dir);
    assert_eq!(first.checked().len(), 4, "first build checks all modules");

    // Delete the UPSTREAM module's source AFTER caching, then edit the DOWNSTREAM
    // module `a`'s BODY. If re-checking `a` re-parsed `base`, the build would fail
    // on the missing file. It does not: `a` is re-checked against `base`'s cached
    // interface artifact (its signature-only stub), never its source.
    std::fs::remove_file(dir.join("base.cnr")).unwrap();
    write(&dir, "a.cnr", "use base::{f};\n\npub fn a_val() -> i64 { return f(11); }\n");

    let r = build::build_dir(&dir).expect("build must not hard-error with upstream source gone");
    assert!(r.ok(), "re-check against the artifact stub must succeed: {:?}", r.diags.iter().map(|d| &d.code).collect::<Vec<_>>());
    assert_eq!(r.checked(), vec!["a".to_string()], "only the edited downstream module re-checks");
    assert!(r.reused().contains(&"base".to_string()), "the deleted upstream module is reused from its artifact, not re-parsed");
    assert_eq!(sorted(r.reused()), vec!["b".to_string(), "base".to_string(), "main".to_string()]);
}

#[test]
fn gate_a2_upstream_signature_stub_resolves_calls() {
    // Deleting `base` and editing BOTH importers' bodies still resolves every
    // `base::f` call from the stub alone — signatures cross, bodies do not.
    let dir = temp_copy("build_c");
    build(&dir);
    std::fs::remove_file(dir.join("base.cnr")).unwrap();
    write(&dir, "a.cnr", "use base::{f};\n\npub fn a_val() -> i64 { return f(1) + f(2); }\n");
    write(&dir, "b.cnr", "use base::{f};\n\npub fn b_val() -> i64 { return f(3); }\n");
    let r = build::build_dir(&dir).unwrap();
    assert!(r.ok(), "{:?}", r.diags.iter().map(|d| &d.code).collect::<Vec<_>>());
    assert_eq!(sorted(r.checked()), vec!["a".to_string(), "b".to_string()]);
}

// ===========================================================================
// (b) PER-INSTANTIATION CODEGEN CACHE (Residual 2)
// ===========================================================================

#[test]
fn gate_b2_initial_build_emits_every_instantiation() {
    let dir = temp_copy("codegen_c2");
    let r = build(&dir);
    assert_eq!(
        sorted(r.codegen_emitted()),
        vec!["gen::id<i64>".to_string(), "gen::id<u32>".to_string(), "gen::snd<u32,i64>".to_string()],
        "the first build emits every reached monomorphized instance"
    );
    assert!(r.codegen_reused().is_empty(), "nothing is cached yet");

    // A second identical build reuses every instantiation (content-addressed).
    let r2 = build(&dir);
    assert!(r2.codegen_emitted().is_empty(), "an unchanged rebuild re-emits nothing");
    assert_eq!(r2.codegen_reused().len(), 3, "every instantiation is reused");
}

#[test]
fn gate_b2_nongeneric_edit_reuses_every_instantiation() {
    let dir = temp_copy("codegen_c2");
    build(&dir);
    // Edit only a NON-generic body (a literal in `main`); no generic body moves.
    write(&dir, "main.cnr",
        "use gen::{id, snd};\nfn main() -> i64 {\n    let a: i64 = id(6);\n    let b: u32 = id(7u32);\n    let c: i64 = snd(1u32, 9);\n    return a + conv i64 b + c;\n}\n");
    let r = build(&dir);
    assert_eq!(r.checked(), vec!["main".to_string()], "only `main` re-checks");
    assert!(r.codegen_emitted().is_empty(), "a non-generic edit re-emits no instantiation");
    assert_eq!(
        sorted(r.codegen_reused()),
        vec!["gen::id<i64>".to_string(), "gen::id<u32>".to_string(), "gen::snd<u32,i64>".to_string()],
        "every cached instantiation of every untouched generic is reused"
    );
}

#[test]
fn gate_b2_generic_body_edit_invalidates_exactly_its_own() {
    let dir = temp_copy("codegen_c2");
    build(&dir);
    // Edit `id`'s BODY only (its signature is unchanged); `snd` is untouched.
    write(&dir, "gen.cnr",
        "pub fn id[T: copy](x: T) -> T { let y: T = x; return y; }\n\npub fn snd[T: copy, U: copy](a: T, b: U) -> U { return b; }\n");
    let r = build(&dir);
    // 0008's promise: never cascades analysis — `main` is NOT re-analyzed.
    assert_eq!(r.checked(), vec!["gen".to_string()], "only the edited generic's module re-checks; analysis never cascades to `main`");
    assert_eq!(
        sorted(r.codegen_emitted()),
        vec!["gen::id<i64>".to_string(), "gen::id<u32>".to_string()],
        "exactly the edited generic's own instantiations are re-emitted"
    );
    assert_eq!(r.codegen_reused(), vec!["gen::snd<u32,i64>".to_string()], "the untouched sibling generic's instantiation is reused");
}

// ===========================================================================
// (d) DIFFERENTIAL EQUIVALENCE — stub-checking == whole-program merge check
// ===========================================================================

fn key(d: &Diag) -> String {
    format!("{}|{}|{}..{}", d.code, d.message, d.span.start, d.span.end)
}

fn merge_diags(dir: &Path) -> Vec<String> {
    let mb = modules::build_tree(dir).unwrap();
    let mut v: Vec<String> = if mb.diags.iter().any(|d| d.severity == Severity::Error) {
        mb.diags.iter().map(key).collect()
    } else {
        let mut base: Vec<String> = mb.diags.iter().map(key).collect();
        base.extend(check::check_program_real(&mb.program).iter().map(key));
        base
    };
    v.sort();
    v.dedup();
    v
}

fn stub_diags(dir: &Path) -> Vec<String> {
    let parts = modules::build_tree_parts(dir).unwrap();
    let mut v: Vec<String> = if parts.diags.iter().any(|d| d.severity == Severity::Error) {
        parts.diags.iter().map(key).collect()
    } else {
        build::check_tree_stubwise(&parts).iter().map(key).collect()
    };
    v.sort();
    v.dedup();
    v
}

#[test]
fn gate_d2_stub_checking_equals_whole_program_merge() {
    // Positive AND negative module trees: every diagnostic the whole-program
    // merge check produces, the signature-only per-module stub check produces —
    // and no other. The stub tier changes compile *scheduling*, never *meaning*.
    let positives = [
        "corelib", "build_c", "codegen_c2", "corelib_question",
        "modules/ok_impl", "modules/ok_impl_generic", "modules/ok_tree",
    ];
    let negatives = [
        "corelib_neg_orphan", "corelib_neg_private", "corelib_neg_unresolved",
        "modules/bad_orphan", "modules/bad_orphan_generic",
        "modules/bad_private", "modules/bad_unresolved", "modules/bad_cycle",
    ];
    for name in positives.iter().chain(negatives.iter()) {
        let dir = fixture(name);
        let m = merge_diags(&dir);
        let s = stub_diags(&dir);
        assert_eq!(m, s, "stub vs merge diagnostic divergence in `{name}`");
    }
}
