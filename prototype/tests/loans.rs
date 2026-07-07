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
         fn f() -> unit { let mut x: S = mk(); h(out x, read x); }",
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

#[test]
fn nll_loan_dead_before_loop_not_live_in_loop() {
    // NLL positive (dual of the init.rs fix): `b` borrows `x` and is last used
    // BEFORE the loop, so it is not live inside the loop and the loop's write to
    // `x` is accepted. (Backward liveness is a union analysis whose natural
    // bottom is the empty set, so unvisited successors never over-approximate.)
    assert_clean(
        "fn f() -> unit { let mut x: S = mk(); let b = read x; use_i((deref b).n); \
         while true { x = mk(); } }",
    );
}

// ---- §2.3 / §8.2.1: return-extended loan on an inline call scrutinee -------
// A compact-default fn returning a borrow into its argument, called inline as a
// match scrutinee with a NON-copy payload bound as a borrow binding: the
// argument loan must persist over the binding's live range (verification S1).

const S1: &str = "
struct Big { a: i64, b: i64 }
enum Wrap { one(Big), none }
fn get(w: read Wrap) -> read Wrap { return read (deref w); }
";

#[test]
fn inline_scrutinee_reassign_arg_in_arm() {
    assert_has(
        &format!("{S1}fn f() -> i64 {{ let mut w: Wrap = Wrap::one(Big {{ a: 111, b: 222 }}); \
            match get(read w) {{ case Wrap::one(inner) => {{ w = Wrap::one(Big {{ a: 999, b: 888 }}); \
            return (deref inner).a; }} case Wrap::none => {{ return 0; }} }} }}"),
        "E0803",
    );
}

#[test]
fn inline_scrutinee_write_mode_call_in_arm() {
    assert_has(
        &format!("{S1}fn clobber(w: write Wrap) -> unit {{ (deref w) = Wrap::none; }} \
            fn f() -> i64 {{ let mut w: Wrap = Wrap::one(Big {{ a: 111, b: 222 }}); \
            match get(read w) {{ case Wrap::one(inner) => {{ clobber(write w); \
            return (deref inner).a; }} case Wrap::none => {{ return 0; }} }} }}"),
        "E0801",
    );
}

#[test]
fn inline_scrutinee_arena_mutate_element_in_arm() {
    assert_has(
        "struct Payload { x: i64 } enum Node { leaf(Payload), other } \
         struct Arena { mem: [4]Node, count: u32 } \
         fn arena_get(ar: read Arena, i: u32) -> read Node { return read (deref ar).mem[conv usize (i)]; } \
         fn f() -> i64 { let mut ar: Arena = Arena { mem: [Node::leaf(Payload { x: 111 }), Node::other, Node::other, Node::other], count: 1u32 }; \
         match arena_get(read ar, 0u32) { case Node::leaf(p) => { ar.mem[0] = Node::leaf(Payload { x: 999 }); \
         return (deref p).x; } case Node::other => { return 0; } } }",
        "E0803",
    );
}

#[test]
fn inline_scrutinee_reborrow_of_reborrow() {
    assert_has(
        &format!("{S1}fn outer(w: read Wrap) -> read Wrap {{ return mid(read (deref w)); }} \
            fn mid(w: read Wrap) -> read Wrap {{ return read (deref w); }} \
            fn f() -> i64 {{ let mut w: Wrap = Wrap::one(Big {{ a: 111, b: 222 }}); \
            match outer(read w) {{ case Wrap::one(inner) => {{ w = Wrap::one(Big {{ a: 999, b: 888 }}); \
            return (deref inner).a; }} case Wrap::none => {{ return 0; }} }} }}"),
        "E0803",
    );
}

#[test]
fn inline_scrutinee_copy_payload_reassign_accepted() {
    // Copy payloads are read out at the match head, ending the loan there — the
    // §11.5 fixture depends on this; the reassignment in the arm is legal.
    assert_clean(
        "enum W { one(i64), none } \
         fn get(w: read W) -> read W { return read (deref w); } \
         fn f() -> i64 { let mut w: W = W::one(111); \
         match get(read w) { case W::one(inner) => { w = W::one(999); return inner; } \
         case W::none => { return 0; } } }",
    );
}

#[test]
fn named_binding_control_stays_rejected() {
    // The named-local equivalent already rejected (E0803); it must stay rejected.
    assert_has(
        "struct Big { a: i64, b: i64 } \
         fn getb(w: read Big) -> read Big { return read (deref w); } \
         fn f() -> i64 { let mut b: Big = Big { a: 111, b: 222 }; \
         let r: borrow Big = getb(read b); b = Big { a: 999, b: 888 }; return (deref r).a; }",
        "E0803",
    );
}
