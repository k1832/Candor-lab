//! std growable `Vec[T]` (PROPOSAL-selfhost-ergonomics candidate A): compiler-known,
//! allocator-explicit, alloc-copy-free growth, per-element drop + buffer free on drop.

use candor::diag::Severity;
use candor::interp::FaultKind;
use candor::{check_source_real, run_source_real, RunResult};

fn errors(src: &str) -> Vec<String> {
    match check_source_real(src) {
        Ok(diags) => diags.into_iter().filter(|d| d.severity == Severity::Error).map(|d| d.code).collect(),
        Err(parse) => vec![parse.code],
    }
}
fn run_ret(src: &str) -> i64 {
    match run_source_real(src) {
        RunResult::Ok(r) => r.ret,
        RunResult::Fault(f) => panic!("unexpected fault: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!("check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        RunResult::ParseError(d) => panic!("parse error: {}", d.to_json()),
    }
}
fn run_fault(src: &str) -> FaultKind {
    match run_source_real(src) {
        RunResult::Fault(f) => f.kind,
        RunResult::Ok(r) => panic!("expected fault, got ret {}", r.ret),
        RunResult::CheckErrors(d) => panic!("check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        RunResult::ParseError(d) => panic!("parse error: {}", d.to_json()),
    }
}
fn run_trace(src: &str) -> Vec<i64> {
    match run_source_real(src) {
        RunResult::Ok(r) => r.trace,
        RunResult::Fault(f) => panic!("unexpected fault: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!("check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>()),
        RunResult::ParseError(d) => panic!("parse error: {}", d.to_json()),
    }
}

// A COUNTING bump allocator: `live` = allocations minus frees. After a balanced
// program it must return to 0 (no leak, no double free).
const ALLOC: &str = r#"
struct AllocVtable { alloc: fn(ctx: rawptr u8, size: usize, align: usize) alloc -> rawptr u8, free: fn(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) alloc -> unit }
copy struct Alloc { ctx: rawptr u8, vt: rawptr AllocVtable }
struct Bump { next: usize, end: usize, live: i64 }
fn with_window(base: usize, size: usize) -> Bump { return Bump { next: base, end: base + size, live: 0 }; }
fn bump_alloc(ctx: rawptr u8, size: usize, align: usize) -> rawptr u8 { unsafe "reserved window" { let b: Bump = ptr_read(cast_ptr[Bump](ctx)); let a: usize = (b.next + align - 1) / align * align; if a + size > b.end { return ptr_null[u8](); } ptr_write(cast_ptr[Bump](ctx), Bump { next: a + size, end: b.end, live: b.live + 1 }); return addr_to_ptr[u8](a); } }
fn bump_free(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) -> unit { unsafe "reserved window" { let b: Bump = ptr_read(cast_ptr[Bump](ctx)); ptr_write(cast_ptr[Bump](ctx), Bump { next: b.next, end: b.end, live: b.live - 1 }); } }
static BUMP_VT: AllocVtable = AllocVtable { alloc: bump_alloc, free: bump_free };
fn mk_alloc(state: write Bump) -> Alloc { unsafe "outlives every alloc" { return Alloc { ctx: cast_ptr[u8](addr_of_mut(state.*)), vt: addr_of(BUMP_VT) }; } }
"#;

fn with_alloc(body: &str) -> String {
    format!("{ALLOC}\nfn main() alloc -> i64 {{\n  let mut bs: Bump = with_window(16777216, 1048576);\n  let al: Alloc = mk_alloc(write bs);\n{body}\n}}")
}

#[test]
fn vec_new_empty_len_zero() {
    let src = with_alloc("  let mut v: Vec[i64] = vec_new(read al);\n  return conv i64 len(read v);");
    assert_eq!(run_ret(&src), 0);
}

#[test]
fn vec_push_len() {
    let src = with_alloc("  let mut v: Vec[i64] = vec_new(read al);\n  push(write v, 10);\n  push(write v, 20);\n  push(write v, 30);\n  return conv i64 len(read v);");
    assert_eq!(run_ret(&src), 3);
}

#[test]
fn vec_get_reads_element() {
    let src = with_alloc("  let mut v: Vec[i64] = vec_new(read al);\n  push(write v, 10);\n  push(write v, 20);\n  push(write v, 30);\n  return get(read v, 1).*;");
    assert_eq!(run_ret(&src), 20);
}

#[test]
fn vec_growth_across_realloc_preserves() {
    // Initial cap 4; push 12 (cap 4->8->16), then read the first and last back.
    let src = with_alloc("  let mut v: Vec[i64] = vec_new(read al);\n  push(write v, 10); push(write v, 20); push(write v, 30); push(write v, 40);\n  push(write v, 50); push(write v, 60); push(write v, 70); push(write v, 80);\n  push(write v, 90); push(write v, 100); push(write v, 110); push(write v, 120);\n  if get(read v, 0).* == 10 { return get(read v, 11).*; }\n  return -1;");
    assert_eq!(run_ret(&src), 120);
}

#[test]
fn vec_get_out_of_bounds_faults() {
    let src = with_alloc("  let mut v: Vec[i64] = vec_new(read al);\n  push(write v, 10);\n  return get(read v, 5).*;");
    assert_eq!(run_fault(&src), FaultKind::Bounds);
}

// Opt for the `pop` and `for` tests: concrete so `build_named_enum("Opt")` resolves.
const OPT: &str = "enum Opt { Some(i64), None }\n";

#[test]
fn vec_pop_some() {
    let src = format!("{OPT}{}", with_alloc("  let mut v: Vec[i64] = vec_new(read al);\n  push(write v, 10); push(write v, 20); push(write v, 30);\n  let o: Opt = pop(write v);\n  match o { Opt::Some(x) => { return x; } Opt::None => { return -1; } }"));
    assert_eq!(run_ret(&src), 30);
}

#[test]
fn vec_pop_empty_none() {
    let src = format!("{OPT}{}", with_alloc("  let mut v: Vec[i64] = vec_new(read al);\n  let o: Opt = pop(write v);\n  match o { Opt::Some(x) => { return x; } Opt::None => { return 0; } }"));
    assert_eq!(run_ret(&src), 0);
}

#[test]
fn vec_pop_decrements_len() {
    let src = format!("{OPT}{}", with_alloc("  let mut v: Vec[i64] = vec_new(read al);\n  push(write v, 10); push(write v, 20); push(write v, 30);\n  let o: Opt = pop(write v);\n  match o { Opt::Some(x) => { return conv i64 len(read v); } Opt::None => { return -1; } }"));
    assert_eq!(run_ret(&src), 2);
}

#[test]
fn vec_set_overwrites() {
    let src = with_alloc("  let mut v: Vec[i64] = vec_new(read al);\n  push(write v, 10); push(write v, 20); push(write v, 30);\n  set(write v, 1, 99);\n  return get(read v, 1).*;");
    assert_eq!(run_ret(&src), 99);
}

#[test]
fn vec_indexed_for_loop_sums() {
    let src = format!("{OPT}{}", with_alloc("  let mut total: i64 = 0;\n  let mut v: Vec[i64] = vec_new(read al);\n  push(write v, 10); push(write v, 20); push(write v, 30);\n  for x in read v { total = total + x; }\n  return total;"));
    assert_eq!(run_ret(&src), 60);
}

#[test]
fn vec_i64_leak_balance() {
    // A helper owns and drops the Vec; after it returns the allocator is balanced.
    let src = format!("{ALLOC}\nfn fill(al: Alloc) alloc -> unit {{\n  let mut v: Vec[i64] = vec_new(read al);\n  push(write v, 1); push(write v, 2); push(write v, 3); push(write v, 4); push(write v, 5); push(write v, 6);\n}}\nfn main() alloc -> i64 {{\n  let mut bs: Bump = with_window(16777216, 1048576);\n  let al: Alloc = mk_alloc(write bs);\n  fill(al);\n  return bs.live;\n}}");
    assert_eq!(run_ret(&src), 0);
}

// A drop-hooked element type: each drop appends its id to the trace log.
const ELEM: &str = "struct E { id: i64 } drop(write self) { trace(self.id); }\n";

#[test]
fn vec_drops_each_element_exactly_once() {
    // Push 5 (forces a 4->8 realloc that raw-moves the first 4); Vec drop must
    // then drop each of the 5 elements exactly once.
    let src = format!("{ALLOC}{ELEM}\nfn fill(al: Alloc) alloc -> unit {{\n  let mut v: Vec[E] = vec_new(read al);\n  push(write v, E {{ id: 1 }}); push(write v, E {{ id: 2 }}); push(write v, E {{ id: 3 }});\n  push(write v, E {{ id: 4 }}); push(write v, E {{ id: 5 }});\n}}\nfn main() alloc -> i64 {{\n  let mut bs: Bump = with_window(16777216, 1048576);\n  let al: Alloc = mk_alloc(write bs);\n  fill(al);\n  return bs.live;\n}}");
    let mut t = run_trace(&src);
    t.sort();
    assert_eq!(t, vec![1, 2, 3, 4, 5], "each element drops exactly once");
}

#[test]
fn vec_set_drops_overwritten_element_once() {
    let src = format!("{ALLOC}{ELEM}\nfn fill(al: Alloc) alloc -> unit {{\n  let mut v: Vec[E] = vec_new(read al);\n  push(write v, E {{ id: 1 }}); push(write v, E {{ id: 2 }});\n  set(write v, 0, E {{ id: 9 }});\n}}\nfn main() alloc -> i64 {{\n  let mut bs: Bump = with_window(16777216, 1048576);\n  let al: Alloc = mk_alloc(write bs);\n  fill(al);\n  return bs.live;\n}}");
    let mut t = run_trace(&src);
    t.sort();
    assert_eq!(t, vec![1, 2, 9], "overwritten element drops once, survivors drop once");
}

#[test]
fn vec_box_bearing_element_drop_balances() {
    // Vec[BB] where BB owns a Box; dropping the Vec drops each BB, freeing its Box.
    let src = format!("{ALLOC}\nstruct BB {{ p: Box i64 }}\nfn fill(al: Alloc) alloc -> unit {{\n  let mut v: Vec[BB] = vec_new(read al);\n  match box(read al, 7) {{ BoxResult::boxed(b) => {{ push(write v, BB {{ p: b }}); }} BoxResult::oom => {{ return; }} }}\n  match box(read al, 8) {{ BoxResult::boxed(b) => {{ push(write v, BB {{ p: b }}); }} BoxResult::oom => {{ return; }} }}\n}}\nfn main() alloc -> i64 {{\n  let mut bs: Bump = with_window(16777216, 1048576);\n  let al: Alloc = mk_alloc(write bs);\n  fill(al);\n  return bs.live;\n}}");
    assert_eq!(run_ret(&src), 0, "every element Box and the Vec buffer are freed");
}

#[test]
fn vec_drop_in_non_alloc_fn_is_e0401() {
    // Dropping a `Vec` frees its buffer (allocator work), so a function that drops
    // one must carry the `alloc` effect; a non-`alloc` one is E0401.
    let src = "fn sink(v: Vec[i64]) -> unit { return; }\nfn main() -> i64 { return 0; }";
    assert!(errors(src).contains(&"E0401".to_string()), "got {:?}", errors(src));
}

#[test]
fn vec_demonstrator_token_buffer_no_cap() {
    // The friction-2 pattern (a fixed `[1024]Tok` + count) rewritten with `Vec`:
    // push past any fixed cap; the growable owner has no hard limit.
    let src = format!("{ALLOC}\nstruct Tok {{ kind: i64, pos: i64 }}\nfn main() alloc -> i64 {{\n  let mut bs: Bump = with_window(16777216, 1048576);\n  let al: Alloc = mk_alloc(write bs);\n  let mut toks: Vec[Tok] = vec_new(read al);\n  push(write toks, Tok {{ kind: 1, pos: 0 }}); push(write toks, Tok {{ kind: 2, pos: 1 }});\n  push(write toks, Tok {{ kind: 3, pos: 2 }}); push(write toks, Tok {{ kind: 4, pos: 3 }});\n  push(write toks, Tok {{ kind: 5, pos: 4 }}); push(write toks, Tok {{ kind: 6, pos: 5 }});\n  push(write toks, Tok {{ kind: 7, pos: 6 }}); push(write toks, Tok {{ kind: 8, pos: 7 }});\n  push(write toks, Tok {{ kind: 9, pos: 8 }}); push(write toks, Tok {{ kind: 10, pos: 9 }});\n  return conv i64 len(read toks);\n}}");
    assert_eq!(run_ret(&src), 10);
}

// ===========================================================================
// OBL-ITER-BORROW — region-free borrowed-yield iteration (`for read x in read v`)
// ===========================================================================

#[test]
fn vec_ref_for_borrows_each_noncopy_element_without_copy() {
    // `for read x in read v` over a Vec of a NON-`copy`, drop-hooked struct binds
    // `x` to a `read E` reborrow of each element. Reading `x.id` sums 10+20+30=60
    // through the borrow (no copy). The only drops are the THREE owned elements
    // when the Vec is dropped: if iteration had moved/copied any element, the trace
    // would show extra drops or a wrong count. `count`/`get_ref` (RefIndexed) are
    // wired for Vec; the loop-local `read` borrow yields region-free reborrows.
    let src = format!(
        "{ALLOC}{ELEM}\nfn fill(al: Alloc) alloc -> i64 {{\n  \
         let mut v: Vec[E] = vec_new(read al);\n  \
         push(write v, E {{ id: 10 }}); push(write v, E {{ id: 20 }}); push(write v, E {{ id: 30 }});\n  \
         let mut sum: i64 = 0;\n  \
         for read x in read v {{ sum = sum + x.id; }}\n  \
         trace(sum);\n  return sum;\n}}\n\
         fn main() alloc -> i64 {{\n  \
         let mut bs: Bump = with_window(16777216, 1048576);\n  \
         let al: Alloc = mk_alloc(write bs);\n  \
         let s: i64 = fill(al);\n  \
         if bs.live != 0 {{ return -1; }}\n  return s;\n}}"
    );
    assert_eq!(run_ret(&src), 60, "fields read through the per-element borrow, buffer freed (balance 0)");
    let t = run_trace(&src);
    assert_eq!(t.first().copied(), Some(60), "sum traced before any element drops");
    let mut drops = t[1..].to_vec();
    drops.sort();
    assert_eq!(drops, vec![10, 20, 30], "each element dropped exactly once — none moved or copied by the borrow-walk");
}

#[test]
fn vec_ref_for_mutation_during_iteration_rejected() {
    // The loop-local `read` borrow of `v` spans the loop (used by `count`/`get_ref`
    // each turn), so a `push(write v, ...)` inside the body conflicts by XOR
    // (chapter 04) — iterator invalidation caught by the EXISTING loan machinery,
    // no new rule (OBL-ITER-BORROW region-free branch).
    let src = format!(
        "{ALLOC}{ELEM}\nfn fill(al: Alloc) alloc -> unit {{\n  \
         let mut v: Vec[E] = vec_new(read al);\n  \
         push(write v, E {{ id: 1 }}); push(write v, E {{ id: 2 }});\n  \
         for read x in read v {{ push(write v, E {{ id: x.id }}); }}\n}}\n\
         fn main() alloc -> i64 {{ return 0; }}"
    );
    let e = errors(&src);
    assert!(
        e.iter().any(|c| c == "E0801"),
        "expected E0801 (an exclusive borrow excludes all others, §2.2) rejecting mutation-during-iteration, got {e:?}"
    );
}

#[test]
fn vec_copy_indexed_for_loop_still_works_unchanged() {
    // The copy-item `Indexed` path (`for x in read v`, `x` a COPIED item) is
    // untouched by the borrowed variant: a Vec[i64] walk still binds `x` by copy.
    let src = format!(
        "{OPT}{}",
        with_alloc(
            "  let mut total: i64 = 0;\n  let mut v: Vec[i64] = vec_new(read al);\n  \
             push(write v, 3); push(write v, 4); push(write v, 5);\n  \
             for x in read v {{ total = total + x; }}\n  return total;"
        )
    );
    assert_eq!(run_ret(&src), 12);
}
