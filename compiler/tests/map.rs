//! std hash `Map[V]` (PROPOSAL-selfhost-ergonomics candidate B): compiler-known,
//! byte-string keys (`str`/`[u8]`), 64-bit FNV-1a, open addressing + linear
//! probing, alloc-copy-rehash-free growth. The map OWNS a heap byte-copy of each
//! key; drop frees every key copy, drops every live value, and frees the buckets.

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

// The same COUNTING bump allocator the Vec suite uses: `live` = allocs - frees.
const ALLOC: &str = r#"
struct AllocVtable { alloc: fn(ctx: rawptr u8, size: usize, align: usize) alloc -> rawptr u8, free: fn(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) alloc -> unit, realloc: fn(ctx: rawptr u8, ptr: rawptr u8, old_size: usize, new_size: usize, align: usize) alloc -> rawptr u8 }
copy struct Alloc { ctx: rawptr u8, vt: rawptr AllocVtable }
struct Bump { next: usize, end: usize, live: i64 }
fn with_window(base: usize, size: usize) -> Bump { return Bump { next: base, end: base + size, live: 0 }; }
fn bump_alloc(ctx: rawptr u8, size: usize, align: usize) -> rawptr u8 { unsafe "reserved window" { let b: Bump = ptr_read(cast_ptr[Bump](ctx)); let a: usize = (b.next + align - 1) / align * align; if a + size > b.end { return ptr_null[u8](); } ptr_write(cast_ptr[Bump](ctx), Bump { next: a + size, end: b.end, live: b.live + 1 }); return addr_to_ptr[u8](a); } }
fn bump_free(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) -> unit { unsafe "reserved window" { let b: Bump = ptr_read(cast_ptr[Bump](ctx)); ptr_write(cast_ptr[Bump](ctx), Bump { next: b.next, end: b.end, live: b.live - 1 }); } }
fn bump_realloc(ctx: rawptr u8, ptr: rawptr u8, old_size: usize, new_size: usize, align: usize) -> rawptr u8 {
    unsafe "bump cannot reclaim, so it cannot grow in place: carve a fresh block, copy old_size bytes into it, and release the old block through bump_free (a no-op for a real bump, so the old space is leaked as bump semantics require)" {
        let newp: rawptr u8 = bump_alloc(ctx, new_size, align);
        if is_null(newp) {
            return newp;
        }
        let a: usize = ptr_to_addr(ptr);
        let base: usize = ptr_to_addr(newp);
        let mut i: usize = 0usize;
        while i < old_size {
            let s: rawptr u8 = addr_to_ptr[u8](a + i);
            let d: rawptr u8 = addr_to_ptr[u8](base + i);
            let v: u8 = ptr_read(s);
            ptr_write(d, v);
            i = i + 1usize;
        }
        bump_free(ctx, ptr, old_size, align);
        return newp;
    }
}

static BUMP_VT: AllocVtable = AllocVtable { alloc: bump_alloc, free: bump_free, realloc: bump_realloc };
fn mk_alloc(state: write Bump) -> Alloc { unsafe "outlives every alloc" { return Alloc { ctx: cast_ptr[u8](addr_of_mut(state.*)), vt: addr_of(BUMP_VT) }; } }
"#;

fn with_alloc(body: &str) -> String {
    format!("{ALLOC}\nfn main() alloc -> i64 {{\n  let mut bs: Bump = with_window(16777216, 1048576);\n  let al: Alloc = mk_alloc(write bs);\n{body}\n}}")
}

// ---------------------------------------------------------------------------
// Basic operations: new / insert / get / contains / len
// ---------------------------------------------------------------------------

#[test]
fn map_new_empty_len_zero() {
    let src = with_alloc("  let mut m: Map[i64] = map_new(read al);\n  return conv i64 len(read m);");
    assert_eq!(run_ret(&src), 0);
}

#[test]
fn map_insert_get_reads_value() {
    let src = with_alloc("  let mut m: Map[i64] = map_new(read al);\n  insert(write m, \"fn\", 42);\n  return get(read m, \"fn\").*;");
    assert_eq!(run_ret(&src), 42);
}

#[test]
fn map_insert_len() {
    let src = with_alloc("  let mut m: Map[i64] = map_new(read al);\n  insert(write m, \"a\", 1); insert(write m, \"bb\", 2); insert(write m, \"ccc\", 3);\n  return conv i64 len(read m);");
    assert_eq!(run_ret(&src), 3);
}

#[test]
fn map_contains_true_and_false() {
    let src = with_alloc("  let mut m: Map[i64] = map_new(read al);\n  insert(write m, \"let\", 7);\n  if contains(read m, \"let\") { if contains(read m, \"nope\") { return -1; } return 1; }\n  return 0;");
    assert_eq!(run_ret(&src), 1);
}

#[test]
fn map_get_missing_faults() {
    let src = with_alloc("  let mut m: Map[i64] = map_new(read al);\n  insert(write m, \"x\", 1);\n  return get(read m, \"y\").*;");
    assert_eq!(run_fault(&src), FaultKind::Bounds);
}

#[test]
fn map_key_as_byte_slice() {
    // A `[u8]` key (via `as_bytes`) hashes/compares identically to the `str`.
    let src = with_alloc("  let mut m: Map[i64] = map_new(read al);\n  insert(write m, as_bytes(\"key\"), 99);\n  return get(read m, as_bytes(\"key\")).*;");
    assert_eq!(run_ret(&src), 99);
}

// ---------------------------------------------------------------------------
// Overwrite: re-inserting a key updates the value (and drops the old one once)
// ---------------------------------------------------------------------------

#[test]
fn map_overwrite_updates_value_len_stable() {
    let src = with_alloc("  let mut m: Map[i64] = map_new(read al);\n  insert(write m, \"k\", 1);\n  insert(write m, \"k\", 2);\n  insert(write m, \"k\", 3);\n  if len(read m) != conv usize 1 { return -1; }\n  return get(read m, \"k\").*;");
    assert_eq!(run_ret(&src), 3);
}

// ---------------------------------------------------------------------------
// Collision / linear-probing: 6 keys in a cap-8 table (load factor 3/4) forces
// overlapping probe chains; every key must still resolve.
// ---------------------------------------------------------------------------

#[test]
fn map_collision_probing_all_findable() {
    let src = with_alloc(
        "  let mut m: Map[i64] = map_new(read al);\n  \
         insert(write m, \"a\", 1); insert(write m, \"b\", 2); insert(write m, \"c\", 3);\n  \
         insert(write m, \"d\", 4); insert(write m, \"e\", 5); insert(write m, \"f\", 6);\n  \
         let mut sum: i64 = 0;\n  \
         sum = sum + get(read m, \"a\").*; sum = sum + get(read m, \"b\").*; sum = sum + get(read m, \"c\").*;\n  \
         sum = sum + get(read m, \"d\").*; sum = sum + get(read m, \"e\").*; sum = sum + get(read m, \"f\").*;\n  \
         return sum;");
    assert_eq!(run_ret(&src), 21);
}

// ---------------------------------------------------------------------------
// Growth across a rehash boundary: insert past cap 8 (rehash to 16, then 32),
// then confirm the earliest and latest keys still resolve to the right values.
// ---------------------------------------------------------------------------

#[test]
fn map_growth_across_rehash_preserves() {
    let mut body = String::from("  let mut m: Map[i64] = map_new(read al);\n");
    for i in 0..20 {
        body.push_str(&format!("  insert(write m, \"k{i}\", {});\n", i * 10));
    }
    // first and last keys survive two rehashes (8->16->32); len is 20.
    body.push_str("  if len(read m) != conv usize 20 { return -1; }\n");
    body.push_str("  if get(read m, \"k0\").* != 0 { return -2; }\n");
    body.push_str("  return get(read m, \"k19\").*;");
    let src = with_alloc(&body);
    assert_eq!(run_ret(&src), 190);
}

// ---------------------------------------------------------------------------
// Drop correctness against the counting allocator + drop-hooked values
// ---------------------------------------------------------------------------

#[test]
fn map_i64_leak_balance() {
    // A helper owns and drops the Map; every key copy and the bucket buffer are
    // freed, so the allocator returns to balance.
    let src = format!(
        "{ALLOC}\nfn fill(al: Alloc) alloc -> unit {{\n  \
         let mut m: Map[i64] = map_new(read al);\n  \
         insert(write m, \"one\", 1); insert(write m, \"two\", 2); insert(write m, \"three\", 3);\n  \
         insert(write m, \"four\", 4); insert(write m, \"five\", 5); insert(write m, \"six\", 6);\n  \
         insert(write m, \"seven\", 7); insert(write m, \"eight\", 8);\n}}\n\
         fn main() alloc -> i64 {{\n  \
         let mut bs: Bump = with_window(16777216, 1048576);\n  \
         let al: Alloc = mk_alloc(write bs);\n  fill(al);\n  return bs.live;\n}}");
    assert_eq!(run_ret(&src), 0, "every key copy and the bucket buffer are freed");
}

// A drop-hooked value type: each drop appends its id to the trace log.
const ELEM: &str = "struct E { id: i64 } drop(write self) { trace(self.id); }\n";

#[test]
fn map_drops_each_value_exactly_once() {
    // Insert 10 (forces 8->16 rehash that raw-moves the first entries); the Map
    // drop must then drop each of the 10 values exactly once.
    let mut body = String::from("  let mut m: Map[E] = map_new(read al);\n");
    for i in 1..=10 {
        body.push_str(&format!("  insert(write m, \"k{i}\", E {{ id: {i} }});\n"));
    }
    body.push_str("  return 0;");
    let src = format!(
        "{ALLOC}{ELEM}\nfn fill(al: Alloc) alloc -> unit {{\n{}\n}}\n\
         fn main() alloc -> i64 {{\n  \
         let mut bs: Bump = with_window(16777216, 1048576);\n  \
         let al: Alloc = mk_alloc(write bs);\n  fill(al);\n  return bs.live;\n}}",
        body.replace("return 0;", ""));
    let mut t = run_trace(&src);
    t.sort();
    assert_eq!(t, (1..=10).collect::<Vec<i64>>(), "each value drops exactly once");
}

#[test]
fn map_overwrite_drops_old_value_once() {
    // Re-inserting key "k" drops the displaced value (id 1) exactly once at the
    // overwrite; the survivors (2 and the replacement 9) drop once at Map drop.
    let src = format!(
        "{ALLOC}{ELEM}\nfn fill(al: Alloc) alloc -> unit {{\n  \
         let mut m: Map[E] = map_new(read al);\n  \
         insert(write m, \"k\", E {{ id: 1 }}); insert(write m, \"other\", E {{ id: 2 }});\n  \
         insert(write m, \"k\", E {{ id: 9 }});\n}}\n\
         fn main() alloc -> i64 {{\n  \
         let mut bs: Bump = with_window(16777216, 1048576);\n  \
         let al: Alloc = mk_alloc(write bs);\n  fill(al);\n  return bs.live;\n}}");
    let mut t = run_trace(&src);
    t.sort();
    assert_eq!(t, vec![1, 2, 9], "overwritten value drops once; survivors drop once");
    assert_eq!(run_ret(&src), 0, "allocator balanced (keys + buckets freed)");
}

#[test]
fn map_box_bearing_value_drop_balances() {
    // Map[BB] where BB owns a Box; dropping the Map drops each BB, freeing its Box,
    // plus every key copy and the buckets.
    let src = format!(
        "{ALLOC}\nstruct BB {{ p: Box i64 }}\nfn fill(al: Alloc) alloc -> unit {{\n  \
         let mut m: Map[BB] = map_new(read al);\n  \
         match box(read al, 7) {{ BoxResult::boxed(b) => {{ insert(write m, \"seven\", BB {{ p: b }}); }} BoxResult::oom => {{ return; }} }}\n  \
         match box(read al, 8) {{ BoxResult::boxed(b) => {{ insert(write m, \"eight\", BB {{ p: b }}); }} BoxResult::oom => {{ return; }} }}\n}}\n\
         fn main() alloc -> i64 {{\n  \
         let mut bs: Bump = with_window(16777216, 1048576);\n  \
         let al: Alloc = mk_alloc(write bs);\n  fill(al);\n  return bs.live;\n}}");
    assert_eq!(run_ret(&src), 0, "every value Box, key copy, and the buckets are freed");
}

#[test]
fn map_drop_in_non_alloc_fn_is_e0401() {
    // Dropping a `Map` frees its buffer + key copies (allocator work), so a
    // function that drops one must carry the `alloc` effect; a non-`alloc` one
    // is E0401 (alloc-on-drop, exactly like `Vec`).
    let src = "fn sink(m: Map[i64]) -> unit { return; }\nfn main() -> i64 { return 0; }";
    assert!(errors(src).contains(&"E0401".to_string()), "got {:?}", errors(src));
}

// ---------------------------------------------------------------------------
// Demonstrator: the 56-branch keyword ladder (friction #4) replaced by a Map.
// ---------------------------------------------------------------------------

#[test]
fn map_demonstrator_keyword_classify() {
    // The linear `if span_eq(..) { code }` ladder becomes one Map[i64] lookup:
    // a keyword resolves to its code; a non-keyword returns 0 (contains == false).
    // `classify` mirrors `classify_ident` over the identifier's bytes.
    let src = format!(
        "{ALLOC}\n\
         fn classify(kw: read Map[i64], word: str) -> i64 {{\n  \
         if contains(kw, word) {{ return get(kw, word).*; }}\n  \
         return 0;\n}}\n\
         fn main() alloc -> i64 {{\n  \
         let mut bs: Bump = with_window(16777216, 1048576);\n  \
         let al: Alloc = mk_alloc(write bs);\n  \
         let mut kw: Map[i64] = map_new(read al);\n  \
         insert(write kw, \"fn\", 1); insert(write kw, \"let\", 2); insert(write kw, \"if\", 3);\n  \
         insert(write kw, \"else\", 4); insert(write kw, \"return\", 5); insert(write kw, \"struct\", 6);\n  \
         insert(write kw, \"enum\", 7); insert(write kw, \"match\", 8);\n  \
         let a: i64 = classify(read kw, \"return\");\n  \
         let b: i64 = classify(read kw, \"struct\");\n  \
         let c: i64 = classify(read kw, \"identifier\");\n  \
         return a * 100 + b * 10 + c;\n}}");
    // "return"=5, "struct"=6, "identifier"=0  ->  5*100 + 6*10 + 0 = 560
    assert_eq!(run_ret(&src), 560);
}

#[test]
fn map_demonstrator_symbol_table_balance() {
    // A symbol table (name -> slot) built and torn down inside a helper: the
    // allocator returns to balance, proving no key/value/bucket leak.
    let src = format!(
        "{ALLOC}\nfn resolve_all(al: Alloc) alloc -> i64 {{\n  \
         let mut syms: Map[i64] = map_new(read al);\n  \
         insert(write syms, \"x\", 0); insert(write syms, \"y\", 1); insert(write syms, \"z\", 2);\n  \
         insert(write syms, \"count\", 3); insert(write syms, \"total\", 4);\n  \
         let mut hits: i64 = 0;\n  \
         if contains(read syms, \"y\") {{ hits = hits + get(read syms, \"y\").*; }}\n  \
         if contains(read syms, \"total\") {{ hits = hits + get(read syms, \"total\").*; }}\n  \
         return hits;\n}}\n\
         fn main() alloc -> i64 {{\n  \
         let mut bs: Bump = with_window(16777216, 1048576);\n  \
         let al: Alloc = mk_alloc(write bs);\n  \
         let h: i64 = resolve_all(al);\n  \
         if bs.live != 0 {{ return -1; }}\n  return h;\n}}");
    // y=1, total=4  ->  5, and balance 0
    assert_eq!(run_ret(&src), 5);
}
