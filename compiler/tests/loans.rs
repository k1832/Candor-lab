//! Stage 3 borrow-checker tests (design 0001 §2.2, §2.3, §3.1, §3.3, §7.4):
//! the mandated negative illustrations (each asserting its error code) and the
//! NLL / region positives that must be accepted.

use candor::check_source;

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

// ---- loan-copy use-after-free (design 0015 review F1) ---------------------
// A borrow copied into a new binding must keep its source loan live under the
// copy: without it `let c = b;` shed the loan and the checker admitted a
// use-after-free (a move/write/exclusive access to the borrowed place while the
// copy still points in). Each rejection below was accepted before the fix.

#[test]
fn loan_copy_then_move_out_box() {
    // The verified repro shape: `c` copies a borrow into the box, then the box
    // is moved out while `c` is still live.
    assert_has(
        "fn sink(b: Box S) -> unit { } \
         fn f(boxv: Box S) -> unit { \
             let b = read (deref boxv); let c = b; sink(boxv); use_i((deref c).n); }",
        "E0802",
    );
}

#[test]
fn loan_copy_then_move_out_local() {
    assert_has(
        "fn f() -> unit { let x: S = mk(); let b = read x; let c = b; \
             let y = x; use_i((deref c).n); }",
        "E0802",
    );
}

#[test]
fn loan_copy_chained_then_move_out() {
    // Propagation is transitive: `d` copies `c` copies `b`; the loan on `x`
    // reaches `d` and freezes the move.
    assert_has(
        "fn f() -> unit { let x: S = mk(); let b = read x; let c = b; let d = c; \
             let y = x; use_i((deref d).n); }",
        "E0802",
    );
}

#[test]
fn loan_copy_then_write_source() {
    assert_has(
        "fn f() -> unit { let mut x: S = mk(); let b = read x; let c = b; \
             x = mk(); use_i((deref c).n); }",
        "E0803",
    );
}

#[test]
fn loan_copy_exclusive_then_read_source() {
    // An exclusive-borrow copy (a move of the `write` borrow) still freezes the
    // source: reading it while the copy is live is E0804.
    assert_has(
        "fn f() -> unit { let mut x: S = mk(); let b = write x; let c = b; \
             use_i(x.n); use_i((deref c).n); }",
        "E0804",
    );
}

#[test]
fn loan_copy_then_return_borrow_of_local() {
    // The copy still traces to the local, so returning it escapes a borrow of a
    // dying local (E0806) — the copy did not launder the provenance.
    assert_has(
        "fn f() -> read S { let x: S = mk(); let b = read x; let c = b; return c; }",
        "E0806",
    );
}

#[test]
fn loan_copy_of_return_extended_borrow_then_write() {
    // A copy of a call's return-extended borrow keeps the underlying argument
    // loan: a later write to the source is a conflict.
    assert_has(
        "fn first(s: read S) -> read S { return read (deref s); } \
         fn f() -> unit { let mut x: S = mk(); let r = first(read x); let c = r; \
             x = mk(); use_i((deref c).n); }",
        "E0803",
    );
}

#[test]
fn loan_copy_dead_before_conflict_is_clean() {
    // NLL positive: the copy dies before the source is rewritten, so the copy
    // pattern itself introduces no false positive.
    assert_clean(
        "fn f() -> unit { let mut x: S = mk(); let b = read x; let c = b; \
             use_i((deref c).n); x = mk(); }",
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

// ---- slice-copy use-after-free (design 0015 review F1 residual) -----------
// A slice (`[T]`/`[T] mut`) is a fat pointer into its backing, `copy` the same
// way a borrow is. A slice copied into a new binding must keep its source loan
// live under the copy: without it `let s2 = s;` shed the loan and the checker
// admitted a use-after-free (a write/realloc/move of the backing while the copy
// still points in). Each rejection below checked CLEAN before the fix.

#[test]
fn slice_copy_then_read_source_exclusive() {
    // An exclusive-slice copy freezes the backing: reading the array directly
    // while the copy is live is E0804 (the copy did not shed the loan).
    assert_has(
        "fn f() -> unit { let mut a: [4]i64 = [0, 0, 0, 0]; let s = slice_of_mut(a); \
         let s2 = s; use_i(a[0]); use_i(s2[0]); }",
        "E0804",
    );
}

#[test]
fn slice_copy_then_write_source() {
    // A shared-slice copy keeps the backing frozen: writing the array while the
    // copy is live is E0803.
    assert_has(
        "fn f() -> unit { let mut a: [4]i64 = [0, 0, 0, 0]; let s = slice_of(a); \
         let s2 = s; a[0] = 1; use_i(s2[0]); }",
        "E0803",
    );
}

#[test]
fn slice_copy_chained_then_read_source() {
    // Propagation is transitive across slice copies: `s3` copies `s2` copies `s`;
    // the exclusive loan on `a` reaches `s3` and freezes the direct read.
    assert_has(
        "fn f() -> unit { let mut a: [4]i64 = [0, 0, 0, 0]; let s = slice_of_mut(a); \
         let s2 = s; let s3 = s2; use_i(a[0]); use_i(s3[0]); }",
        "E0804",
    );
}

#[test]
fn slice_copy_dead_before_conflict_is_clean() {
    // NLL positive: the slice copy dies before the backing is rewritten, so the
    // copy pattern itself introduces no false positive.
    assert_clean(
        "fn f() -> unit { let mut a: [4]i64 = [0, 0, 0, 0]; let s = slice_of(a); \
         let s2 = s; use_i(s2[0]); a[0] = 1; }",
    );
}

// ---- str/[u8]-view use-after-free (design 0015 review F1 residual) ---------
// `as_str`/`as_bytes`/`substr`/`str_from_unchecked` produce a `str`/`[u8]` VIEW
// into a native String's heap buffer. The view must keep a loan on the source
// String live for its own range: without it the checker admitted a use-after-
// free — a `push`/`append` that forces a `string_reserve` GROWTH (alloc-new +
// copy + FREE-old), or a move/drop of the source, while the view still points
// into the freed old buffer. Each rejection below checked CLEAN before the fix
// (the confirmed repro read stale/garbage bytes on every engine). `use_i` and
// the `Alloc` handle struct come from `PREAMBLE`/`STR_PREAMBLE`.

// Real (`.cnr`) syntax: the `str`/`String` view builtins and the `conv` cast used
// below are real-surface constructs, so these go through `check_source_real`.
const STR_PREAMBLE: &str = "
struct AllocVtable { alloc: fn(ctx: rawptr u8, size: usize, align: usize) alloc -> rawptr u8, free: fn(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) alloc -> unit, realloc: fn(ctx: rawptr u8, ptr: rawptr u8, old_size: usize, new_size: usize, align: usize) alloc -> rawptr u8 }
copy struct Alloc { ctx: rawptr u8, vt: rawptr AllocVtable }
fn use_i(v: i64) -> unit { }
";

fn str_codes(src: &str) -> Vec<String> {
    let full = format!("{STR_PREAMBLE}{src}");
    let diags = candor::check_source_real(&full).expect("parse ok");
    diags.into_iter().map(|d| d.code).collect()
}

fn assert_str_has(src: &str, code: &str) {
    let cs = str_codes(src);
    assert!(cs.iter().any(|c| c == code), "expected `{code}` for:\n{src}\ngot {cs:?}");
}

fn assert_str_clean(src: &str) {
    let cs = str_codes(src);
    assert!(cs.is_empty(), "expected clean for:\n{src}\ngot {cs:?}");
}

#[test]
fn str_view_realloc_while_view_live() {
    // The confirmed repro: `vb = as_bytes(as_str(read s))` views s's buffer, then
    // `append` forces a grow (free-old) while vb is live — E0801 (the grow's
    // `write s` conflicts with the view's shared loan on s).
    assert_str_has(
        "fn f(a: read Alloc) alloc -> unit { let mut s: String = string_new(a);          append(write s, \"abcde\"); let v: str = as_str(read s); let vb: [u8] = as_bytes(v);          append(write s, \"fghij\"); use_i(conv i64 vb[0]); }",
        "E0801",
    );
}

#[test]
fn str_view_move_source_while_view_live() {
    // Moving the source String out while a view of it is live is E0802: the view
    // would dangle into the moved-from buffer.
    assert_str_has(
        "fn f(a: read Alloc) alloc -> unit { let mut s: String = string_new(a);          append(write s, \"abcde\"); let v: str = as_str(read s); let owned: String = s;          use_i(conv i64 len(v)); }",
        "E0802",
    );
}

#[test]
fn substr_view_realloc_while_view_live() {
    // The loan propagates transitively through `substr` (str -> str sub-view):
    // `sub` copies `v`'s loan on s, so a later grow of s is still E0801.
    assert_str_has(
        "fn f(a: read Alloc) alloc -> unit { let mut s: String = string_new(a);          append(write s, \"abcdefgh\"); let v: str = as_str(read s);          let sub: str = substr(v, 0usize, 3usize); append(write s, \"ijklmnop\");          use_i(conv i64 len(sub)); }",
        "E0801",
    );
}

#[test]
fn str_view_dead_before_mutation_is_clean() {
    // NLL positive (the pervasive make-view-use-done pattern): the view dies at
    // its last use, so a subsequent grow of s is accepted — no false positive.
    assert_str_clean(
        "fn f(a: read Alloc) alloc -> unit { let mut s: String = string_new(a);          append(write s, \"abcde\"); let v: str = as_str(read s); let vb: [u8] = as_bytes(v);          use_i(conv i64 vb[0]); append(write s, \"fghij\"); }",
    );
}

// ---- function-return view provenance (design 0015 review F1 residual) ------
// A USER function returning a borrow keeps the loan on its argument alive at the
// call site (return-extension, §3.3). A view return (`[T]`/`str`/`[u8]`) is a
// borrow-kind return too — the returned view aliases the source's backing exactly
// as a borrow does — yet the call-site machinery only extended `borrow`/`borrow_mut`
// returns, so a view laundered out of a user call SHED its source loan: the caller
// could then grow/free/move the backing while the returned view was still live (a
// confirmed check-clean stale/freed read on the native engine). Treating a
// borrow-kind return uniformly closes it; each rejection below checked CLEAN before.

#[test]
fn view_return_realloc_while_view_live() {
    // A user fn returns a `[u8]` view of its `read String` param; the caller grows
    // the String (alloc-new + FREE-old) while the returned view is live. The
    // returned view carries the argument loan on `s`, so the grow's `write s`
    // conflicts with the view's shared loan — E0801.
    assert_str_has(
        "fn view_of(s: read String) -> [u8] { return as_bytes(as_str(read s)); }          fn f(a: read Alloc) alloc -> unit { let mut s: String = string_new(a);          append(write s, \"abcde\"); let vb: [u8] = view_of(read s);          append(write s, \"fghij\"); use_i(conv i64 vb[0]); }",
        "E0801",
    );
}

#[test]
fn view_return_dead_before_mutation_is_clean() {
    // NLL positive: the returned view dies at its last use before the grow, so the
    // return-extension introduces no false positive.
    assert_str_clean(
        "fn view_of(s: read String) -> [u8] { return as_bytes(as_str(read s)); }          fn f(a: read Alloc) alloc -> unit { let mut s: String = string_new(a);          append(write s, \"abcde\"); let vb: [u8] = view_of(read s);          use_i(conv i64 vb[0]); append(write s, \"fghij\"); }",
    );
}

#[test]
fn view_return_of_local_rejected() {
    // A view return must derive from an input, not a local: returning a view of the
    // callee's OWN local String (dropped/freed at fn exit) is E0806 — the same
    // provenance rule a borrow return obeys, now applied to view returns.
    assert_str_has(
        "fn leak_view(a: read Alloc) alloc -> [u8] { let mut s: String = string_new(a);          append(write s, \"abcde\"); return as_bytes(as_str(read s)); }",
        "E0806",
    );
}

#[test]
fn two_param_view_return_needs_region() {
    // A view return from two borrow params is ambiguous with no region to
    // disambiguate the source (exactly as a two-borrow-param borrow return is):
    // E0807. Without it the loan was extended to neither param and shed silently.
    assert_str_has(
        "fn pick(x: read String, y: read String) -> [u8] { return as_bytes(as_str(read x)); }",
        "E0807",
    );
}

#[test]
fn slice_return_extends_arg_loan() {
    // The same flow for `slice T`: a user fn returns a sub-slice of its slice
    // param; the caller writes the backing array while the returned slice is live.
    // The returned slice carries the argument's shared loan on the array — E0803.
    assert_clean("fn head(s: slice i64) -> slice i64 { return subslice(s, 0, 2); }");
    assert_has(
        "fn head(s: slice i64) -> slice i64 { return subslice(s, 0, 2); } \
         fn f() -> unit { let mut arr: [4]i64 = [0, 0, 0, 0]; let sub = head(slice_of(arr)); \
         arr[0] = 1; use_i(sub[0]); }",
        "E0803",
    );
    // NLL positive: the returned slice dies before the write, so no false positive.
    assert_clean(
        "fn head(s: slice i64) -> slice i64 { return subslice(s, 0, 2); } \
         fn f() -> unit { let mut arr: [4]i64 = [0, 0, 0, 0]; let sub = head(slice_of(arr)); \
         use_i(sub[0]); arr[0] = 1; }",
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

// ---- design 0005: implicit call-site reborrow -----------------------------

#[test]
fn implicit_exclusive_reborrow_accepted() {
    // A bare exclusive borrow `b` passed to a `write`-mode parameter reborrows
    // (it does NOT move); the desugared node is `write (deref b)`.
    assert_clean(
        "fn setn(s: write S, v: i64) -> unit { (deref s).n = v; } \
         fn bump(b: write S) -> unit { setn(b, 42); }",
    );
}

#[test]
fn implicit_shared_reborrow_accepted() {
    // A bare shared borrow `b` passed to a `read`-mode parameter reborrows.
    assert_clean(
        "fn getn(s: read S) -> i64 { return (deref s).n; } \
         fn peek(b: read S) -> i64 { return getn(b); }",
    );
}

#[test]
fn implicit_shared_downgrade_from_exclusive() {
    // A bare EXCLUSIVE borrow to a `read`-mode parameter is a SHARED reborrow
    // (legal downgrade), exactly as the explicit `read (deref b)` is.
    assert_clean(
        "fn getn(s: read S) -> i64 { return (deref s).n; } \
         fn viaexcl(b: write S) -> i64 { return getn(b); }",
    );
}

#[test]
fn take_mode_borrow_param_bare_moves_use_after_move() {
    // A `take`-mode borrow-typed parameter is UNTOUCHED by 0005: bare `b` is a
    // MOVE of the borrow value, so a second use is a use-after-move (E0301).
    assert_has(
        "fn consume(x: borrow_mut S) -> unit { } \
         fn f(b: borrow_mut S) -> unit { consume(b); consume(b); }",
        "E0301",
    );
}

#[test]
fn nonplace_borrow_arg_requires_explicit_reborrow() {
    // A non-place borrow argument (a call result) is NOT implicitly reborrowed:
    // an exclusive-borrow result passed bare to a `read`-mode parameter is a type
    // mismatch (E0703) — the explicit `read (deref ...)` downgrade is still
    // required for non-places.
    assert_has(
        "fn idm(s: write S) -> write S { return s; } \
         fn getn(s: read S) -> i64 { return (deref s).n; } \
         fn f(b: write S) -> i64 { return getn(idm(b)); }",
        "E0703",
    );
    // The explicit reborrow of the call result is accepted (the required form).
    assert_clean(
        "fn idm(s: write S) -> write S { return s; } \
         fn getn(s: read S) -> i64 { return (deref s).n; } \
         fn f(b: write S) -> i64 { return getn(read (deref idm(b))); }",
    );
}

#[test]
fn implicit_reborrow_composes_with_return_extension_s1() {
    // Design 0005 × the S1 return-extension shape. A bare exclusive borrow `w`
    // passed to the compact-default borrow-returning `get` (a `read`-mode param)
    // is an implicit SHARED reborrow; the returned borrow return-extends the loan
    // on the PARENT `w`, carried over the match binding `inner`. Writing through
    // `w` in the arm then conflicts exactly as the explicit spelling does (E0803).
    let base = "struct Big { a: i64, b: i64 } enum Wrap { one(Big), none } \
        fn get(w: read Wrap) -> read Wrap { return read (deref w); } ";
    let implicit = format!(
        "{base}fn outer(w: write Wrap) -> i64 {{ match get(w) {{ \
         case Wrap::one(inner) => {{ (deref w) = Wrap::none; return (deref inner).a; }} \
         case Wrap::none => {{ return 0; }} }} }}"
    );
    assert_has(&implicit, "E0803");
    // The explicit spelling `get(read (deref w))` yields the identical diagnostic.
    let explicit = format!(
        "{base}fn outer(w: write Wrap) -> i64 {{ match get(read (deref w)) {{ \
         case Wrap::one(inner) => {{ (deref w) = Wrap::none; return (deref inner).a; }} \
         case Wrap::none => {{ return 0; }} }} }}"
    );
    assert_has(&explicit, "E0803");
}
