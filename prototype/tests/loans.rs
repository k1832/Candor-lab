//! Stage 3 borrow-checker tests (design 0001 §2.2, §2.3, §3.1, §3.3, §7.4):
//! the mandated negative illustrations (each asserting its error code) and the
//! NLL / region positives that must be accepted.

use candor_proto::check_source;

const PREAMBLE: &str = "
struct S { n: i64 }
struct P { f: i64, g: i64 }
fn use_i(v: i64) -> unit { }
fn mk() -> S { return S { n: 0 }; }
fn mkp() -> P { return P { f: 0, g: 0 }; }
";

fn codes(src: &str) -> Vec<String> {
    let full = format!("{PREAMBLE}{src}");
    let diags = check_source(&full).expect("parse ok");
    diags.into_iter().map(|d| d.code).collect()
}

fn assert_has(src: &str, code: &str) {
    let cs = codes(src);
    assert!(cs.iter().any(|c| c == code), "expected `{code}` for:\n{src}\ngot {cs:?}");
}

fn assert_clean(src: &str) {
    let cs = codes(src);
    assert!(cs.is_empty(), "expected clean for:\n{src}\ngot {cs:?}");
}

// ---- §2.2: move / write / read vs live loans -----------------------------

#[test]
fn move_while_shared_borrowed() {
    // The §2.2 rejected-program illustration.
    assert_has(
        "fn f() -> unit { let x: S = mk(); let b = read x; let y = x; use_i((deref b).n); }",
        "E0802",
    );
}

#[test]
fn write_while_shared_borrowed() {
    assert_has(
        "fn f() -> unit { let mut x: S = mk(); let b = read x; x = mk(); use_i((deref b).n); }",
        "E0803",
    );
}

#[test]
fn two_exclusive_borrows() {
    assert_has(
        "fn f() -> unit { let mut x: S = mk(); let b1 = write x; let b2 = write x; \
         (deref b1).n = 1; (deref b2).n = 2; }",
        "E0801",
    );
}

#[test]
fn overlapping_place_borrows() {
    assert_has(
        "fn f() -> unit { let mut p: P = mkp(); let b = write p; let c = write p.f; \
         (deref b).f = 1; deref c = 2; }",
        "E0801",
    );
}

#[test]
fn disjoint_fields_accepted() {
    assert_clean(
        "fn f() -> unit { let mut p: P = mkp(); let b = write p.f; let c = write p.g; \
         deref b = 1; deref c = 2; }",
    );
}

#[test]
fn index_covers_array_conflict() {
    assert_has(
        "fn f() -> unit { let mut a: [4]i64 = [0, 0, 0, 0]; let b = write a[0]; \
         let c = read a[1]; deref b = 1; use_i(deref c); }",
        "E0801",
    );
}

// ---- §2.1: reborrow rule --------------------------------------------------

#[test]
fn parent_used_while_exclusive_reborrow_live() {
    assert_has(
        "fn f() -> unit { let mut x: S = mk(); let b = write x; let r = write (deref b); \
         use_i((deref b).n); (deref r).n = 1; }",
        "E0804",
    );
}

#[test]
fn parent_written_while_frozen_to_shared() {
    assert_has(
        "fn f() -> unit { let mut x: S = mk(); let b = write x; let r = read (deref b); \
         (deref b).n = 1; use_i((deref r).n); }",
        "E0803",
    );
}

#[test]
fn shared_reborrow_then_parent_reads_ok() {
    // §2.1 revised: a shared reborrow freezes the parent to shared; reads
    // through the parent are still allowed while the reborrow is live.
    assert_clean(
        "fn f() -> unit { let mut x: S = mk(); let b = write x; let r = read (deref b); \
         use_i((deref b).n); use_i((deref r).n); }",
    );
}

// ---- §3.1: same-call overlap (no two-phase) -------------------------------

#[test]
fn no_two_phase_write_read() {
    assert_has(
        "fn g(a: write S, b: read S) -> unit { } \
         fn f() -> unit { let mut x: S = mk(); g(write x, read x); }",
        "E0805",
    );
}

#[test]
fn out_and_read_overlap() {
    assert_has(
        "fn h(a: out S, b: read S) -> unit { a = mk(); } \
         fn f() -> unit { let mut x: S = mk(); h(x, read x); }",
        "E0805",
    );
}

// ---- box moved while pointee borrowed -------------------------------------

#[test]
fn box_moved_while_pointee_borrowed() {
    assert_has(
        "fn sink(b: Box S) -> unit { } \
         fn f(boxv: Box S) -> unit { let b = read (deref boxv); sink(boxv); use_i((deref b).n); }",
        "E0802",
    );
}

// ---- slices are borrows of the array --------------------------------------

#[test]
fn slice_mut_conflicts_with_array_access() {
    assert_has(
        "fn keep_s(s: slice_mut i64) -> unit { } \
         fn f() -> unit { let mut a: [4]i64 = [0, 0, 0, 0]; let s = slice_of_mut(a); \
         use_i(a[0]); keep_s(s); }",
        "E0804",
    );
}

// ---- §3.3: signature regions & provenance ---------------------------------

#[test]
fn returned_borrow_of_local() {
    assert_has("fn f() -> read S { let x: S = mk(); return read x; }", "E0806");
}

#[test]
fn two_borrow_params_return_without_region() {
    assert_has(
        "fn pick(a: read S, b: read S) -> read S { return read (deref a); }",
        "E0807",
    );
}

#[test]
fn provenance_mismatch() {
    assert_has(
        "fn pick2[r](a: read[r] S, b: read S) -> read[r] S { return read (deref b); }",
        "E0808",
    );
}

#[test]
fn compact_default_region_flows_through_call() {
    // Positive: the return derives from the sole borrow param; the caller's loan
    // is extended and the program is accepted.
    assert_clean(
        "fn first(s: read S) -> read S { return read (deref s); } \
         fn f() -> unit { let x: S = mk(); let r = first(read x); use_i((deref r).n); }",
    );
}

#[test]
fn extension_conflict_detected() {
    // The extended argument loan makes a later write to the source a conflict.
    assert_has(
        "fn first(s: read S) -> read S { return read (deref s); } \
         fn f() -> unit { let mut x: S = mk(); let r = first(read x); x = mk(); use_i((deref r).n); }",
        "E0803",
    );
}

// ---- NLL positive, loop-carried negative ----------------------------------

#[test]
fn nll_borrow_dead_then_place_used() {
    assert_clean(
        "fn f() -> unit { let mut x: S = mk(); let b = read x; use_i((deref b).n); let y = x; }",
    );
}

#[test]
fn loop_carried_borrow_conflict() {
    assert_has(
        "fn f() -> unit { let mut x: S = mk(); let b = read x; \
         loop { x = mk(); use_i((deref b).n); } }",
        "E0803",
    );
}

// ---- all-paths-return (design §7.4 / NN#5) --------------------------------

#[test]
fn falls_off_end_of_non_unit_fn() {
    assert_has("fn g(c: bool) -> i64 { if c { return 1; } }", "E0810");
}

#[test]
fn empty_non_unit_fn() {
    assert_has("fn g() -> i64 { }", "E0810");
}

#[test]
fn unit_fn_may_fall_off_end() {
    assert_clean("fn g() -> unit { use_i(1); }");
}
