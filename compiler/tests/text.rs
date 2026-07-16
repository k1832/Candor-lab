//! Design 0013 — text and strings. `str` as a core, immutable, UTF-8-validated
//! view (type-distinct from `[u8]`, no coercion), `b"..."` byte-string literals,
//! the core operation surface (byte `len`/index/compare, `substr` with a
//! char-boundary fault, `as_bytes`, `str_from`), byte iteration via the existing
//! `Indexed` desugar, and the std `String` builder (`push`/`append`/`as_str`).
//! Single-file, real (`.cnr`) front-end.

use candor::diag::Severity;
use candor::interp::FaultKind;
use candor::{
    check_source_real, run_source_real, run_source_real_mir, run_source_real_native,
    run_source_real_native_opt, MirRunResult, RunResult,
};

fn errors(src: &str) -> Vec<String> {
    match check_source_real(src) {
        Ok(diags) => diags
            .into_iter()
            .filter(|d| d.severity == Severity::Error)
            .map(|d| d.code)
            .collect(),
        Err(parse) => vec![parse.code],
    }
}

fn assert_clean(src: &str) {
    let e = errors(src);
    assert!(e.is_empty(), "expected clean, got {e:?}");
}

fn run_ret(src: &str) -> i64 {
    match run_source_real(src) {
        RunResult::Ok(r) => r.ret,
        RunResult::Fault(f) => panic!("unexpected fault: {}", f.to_json()),
        RunResult::CheckErrors(d) => {
            panic!("check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("parse error: {}", d.to_json()),
    }
}

fn run_fault(src: &str) -> FaultKind {
    match run_source_real(src) {
        RunResult::Fault(f) => f.kind,
        RunResult::Ok(r) => panic!("expected fault, got ret {}", r.ret),
        RunResult::CheckErrors(d) => {
            panic!("expected fault, got check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("expected fault, got parse error: {}", d.to_json()),
    }
}

// ---- all-engine drivers (design 0013 str-view native): a program's result must
// match byte-for-byte across the tree-walking oracle, the MIR interpreter, and the
// Cranelift native backend (no-opt + opt). The LLVM `clang -O2` fifth engine is
// covered by the `tests/fixtures/run/str_view.cnr` corpus fixture (auto-scanned by
// `tests/llvm.rs`'s full-corpus gate), the same pattern `string_native` uses.

fn run_ret_all(src: &str) -> i64 {
    let o = match run_source_real(src) {
        RunResult::Ok(r) => r,
        RunResult::Fault(f) => panic!("oracle faulted: {}", f.to_json()),
        RunResult::CheckErrors(d) => {
            panic!("check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("parse error: {}", d.to_json()),
    };
    for (label, r) in [
        ("mir", run_source_real_mir(src)),
        ("native-noopt", run_source_real_native(src)),
        ("native-opt", run_source_real_native_opt(src)),
    ] {
        match r {
            MirRunResult::Ok(run) => {
                assert_eq!(run.ret, o.ret, "{label} ret diverged from oracle");
                assert_eq!(run.trace, o.trace, "{label} trace diverged from oracle");
            }
            MirRunResult::Fault(f) => panic!("{label} faulted: {}", f.to_json()),
            MirRunResult::Unsupported(m) => panic!("{label} unsupported: {m}"),
            MirRunResult::CheckErrors(d) => {
                panic!("{label} check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
            }
            MirRunResult::ParseError(d) => panic!("{label} parse error: {}", d.to_json()),
        }
    }
    o.ret
}

fn run_fault_all(src: &str) -> FaultKind {
    let of = match run_source_real(src) {
        RunResult::Fault(f) => f,
        RunResult::Ok(r) => panic!("expected fault, got ret {}", r.ret),
        RunResult::CheckErrors(d) => {
            panic!("expected fault, got check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
        }
        RunResult::ParseError(d) => panic!("expected fault, got parse error: {}", d.to_json()),
    };
    for (label, r) in [
        ("mir", run_source_real_mir(src)),
        ("native-noopt", run_source_real_native(src)),
        ("native-opt", run_source_real_native_opt(src)),
    ] {
        match r {
            // The fault identity (kind AND span) must match the oracle byte-for-byte.
            MirRunResult::Fault(f) => {
                assert_eq!(f.kind, of.kind, "{label} fault kind diverged from oracle");
                assert_eq!(f.span, of.span, "{label} fault span diverged from oracle");
            }
            MirRunResult::Ok(run) => panic!("{label}: expected fault, got ret {}", run.ret),
            MirRunResult::Unsupported(m) => panic!("{label} unsupported: {m}"),
            MirRunResult::CheckErrors(d) => {
                panic!("{label} check errors: {:?}", d.iter().map(|x| &x.code).collect::<Vec<_>>())
            }
            MirRunResult::ParseError(d) => panic!("{label} parse error: {}", d.to_json()),
        }
    }
    of.kind
}

// ---- str literal typing + no implicit coercion (P2) ------------------------

#[test]
fn str_literal_types_as_str() {
    assert_clean("fn main() -> i64 { let s: str = \"hello\"; return conv i64 len(s); }");
}

#[test]
fn bytes_literal_types_as_slice_u8() {
    assert_clean("fn main() -> i64 { let b: [u8] = b\"hello\"; return conv i64 len(b); }");
}

#[test]
fn str_where_bytes_expected_is_rejected() {
    // `"..."` is `str`; a `[u8]` slot rejects it (no coercion, P2).
    assert!(!errors("fn main() -> i64 { let b: [u8] = \"hello\"; return 0; }").is_empty());
}

#[test]
fn bytes_where_str_expected_is_rejected() {
    assert!(!errors("fn main() -> i64 { let s: str = b\"hello\"; return 0; }").is_empty());
}

#[test]
fn str_arg_where_bytes_param_is_rejected() {
    let src = "fn take(b: [u8]) -> i64 { return conv i64 len(b); }\n\
               fn main() -> i64 { return take(\"hi\"); }";
    assert!(!errors(src).is_empty());
}

// ---- byte length, index, comparison ----------------------------------------

#[test]
fn len_is_bytes_not_chars() {
    // "héllo" — the `é` is 2 UTF-8 bytes, so the byte length is 6, not 5.
    assert_eq!(run_ret("fn main() -> i64 { return conv i64 len(\"héllo\"); }"), 6);
}

#[test]
fn byte_index_yields_u8() {
    // 'b' == 98.
    assert_eq!(run_ret("fn main() -> i64 { let s: str = \"abc\"; return conv i64 s[1]; }"), 98);
}

#[test]
fn byte_index_out_of_bounds_faults() {
    assert_eq!(
        run_fault("fn main() -> i64 { let s: str = \"abc\"; return conv i64 s[9]; }"),
        FaultKind::Bounds
    );
}

#[test]
fn str_equality_is_bytewise() {
    assert_eq!(run_ret("fn main() -> i64 { if \"abc\" == \"abc\" { return 1; } return 0; }"), 1);
    assert_eq!(run_ret("fn main() -> i64 { if \"abc\" == \"abd\" { return 1; } return 0; }"), 0);
}

#[test]
fn str_ordering_is_bytelexicographic() {
    assert_eq!(run_ret("fn main() -> i64 { if \"abc\" < \"abd\" { return 1; } return 0; }"), 1);
    assert_eq!(run_ret("fn main() -> i64 { if \"abd\" < \"abc\" { return 1; } return 0; }"), 0);
}

// ---- substr: char-boundary fault (P5) --------------------------------------

#[test]
fn substr_on_boundary_works() {
    // "héllo" bytes: h | é(2 bytes) | l l o. [0,3) = "hé" (3 bytes).
    assert_eq!(run_ret_all("fn main() -> i64 { let s: str = substr(\"héllo\", 0, 3); return conv i64 len(s); }"), 3);
}

#[test]
fn substr_mid_char_faults() {
    // Offset 2 falls inside the 2-byte `é` — a non-boundary slice is a bug (P5).
    assert_eq!(
        run_fault_all("fn main() -> i64 { let s: str = substr(\"héllo\", 0, 2); return 0; }"),
        FaultKind::Bounds
    );
}

#[test]
fn substr_out_of_bounds_faults() {
    assert_eq!(
        run_fault_all("fn main() -> i64 { let s: str = substr(\"abc\", 0, 9); return 0; }"),
        FaultKind::Bounds
    );
}

// ---- Indexed byte iteration (design 0009 / 0013 §3) ------------------------

#[test]
fn for_over_str_yields_bytes() {
    // Sum the bytes of "abc" = 97 + 98 + 99 = 294.
    let src = "enum Opt { ok Some(u8), None }\n\
               fn main() -> i64 {\n\
                 let s: str = \"abc\";\n\
                 let mut total: i64 = 0;\n\
                 for b in read s { total = total + conv i64 b; }\n\
                 return total;\n\
               }";
    assert_eq!(run_ret(src), 294);
}

// ---- as_bytes / str_from roundtrip -----------------------------------------

#[test]
fn as_bytes_is_free_retype() {
    assert_eq!(run_ret_all("fn main() -> i64 { return conv i64 len(as_bytes(\"hello\")); }"), 5);
}

#[test]
fn len_of_inline_slice_call() {
    // `len` of a NON-place fat pointer (an inline `as_bytes`/`substr` call, not a
    // bound local): the arg has no address, so MIR materializes it into a temp
    // before reading the length word — mirroring the tree-walker's `eval_value`.
    // `substr("hello", 1, 4)` == "ell" -> 3 bytes.
    assert_eq!(
        run_ret_all("fn main() -> i64 { return conv i64 len(as_bytes(substr(\"hello\", 1, 4))); }"),
        3
    );
}

#[test]
fn str_from_valid_bytes() {
    let src = "fn main() -> i64 {\n\
                 match str_from(b\"hi\") {\n\
                   Utf8Res::Valid(s) => { return conv i64 len(s); }\n\
                   Utf8Res::Invalid(off) => { return -1; }\n\
                 }\n\
               }";
    assert_eq!(run_ret_all(src), 2);
}

#[test]
fn str_from_invalid_reports_offset() {
    // [0x68, 0xFF]: byte 0 ('h') is valid, byte 1 (0xFF) is an invalid start —
    // valid_up_to == 1, the offset of the first ill-formed sequence.
    let src = "fn main() -> i64 {\n\
                 let a: [2]u8 = [104, 255];\n\
                 let sl: [u8] = slice_of(a);\n\
                 match str_from(sl) {\n\
                   Utf8Res::Valid(s) => { return -1; }\n\
                   Utf8Res::Invalid(off) => { return conv i64 off; }\n\
                 }\n\
               }";
    assert_eq!(run_ret_all(src), 1);
}

#[test]
fn str_from_then_use() {
    // Roundtrip: as_bytes a str, revalidate, then byte-index the recovered `str`
    // view directly ('c' == 99) — `str[i]` is a native op (design 0013 §3).
    let src = "fn main() -> i64 {\n\
                 let bytes: [u8] = as_bytes(\"abc\");\n\
                 match str_from(bytes) {\n\
                   Utf8Res::Valid(s) => { return conv i64 s[2]; }\n\
                   Utf8Res::Invalid(off) => { return -1; }\n\
                 }\n\
               }";
    assert_eq!(run_ret_all(src), 99);
}

#[test]
fn str_index_reads_bytes() {
    // `s[i]` — the byte at `i` (design 0013 §3), byte-exact on every engine.
    // "abc": s[0] == 'a' (97), s[2] == 'c' (99); 97 + 99 == 196.
    let src = "fn main() -> i64 {\n\
                 let s: str = \"abc\";\n\
                 let a: i64 = conv i64 s[0];\n\
                 let c: i64 = conv i64 s[2];\n\
                 return a + c;\n\
               }";
    assert_eq!(run_ret_all(src), 196);
}

#[test]
fn str_index_out_of_bounds_faults() {
    // `s[i]` past the end faults `Bounds` at the index span — the SAME fault the
    // slice-index path emits, byte-exact (kind + span) on every engine.
    let src = "fn main() -> i64 { let s: str = \"abc\"; return conv i64 s[3]; }";
    assert_eq!(run_fault_all(src), FaultKind::Bounds);
}

// ---- String (std, compiler-known): push / append / as_str ------------------

const ALLOC: &str = r#"
struct AllocVtable { alloc: fn(ctx: rawptr u8, size: usize, align: usize) alloc -> rawptr u8, free: fn(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) alloc -> unit, realloc: fn(ctx: rawptr u8, ptr: rawptr u8, old_size: usize, new_size: usize, align: usize) alloc -> rawptr u8 }
copy struct Alloc { ctx: rawptr u8, vt: rawptr AllocVtable }
struct Bump { next: usize, end: usize }
fn with_window(base: usize, size: usize) -> Bump { return Bump { next: base, end: base + size }; }
fn bump_alloc(ctx: rawptr u8, size: usize, align: usize) -> rawptr u8 { unsafe "reserved window" { let b: Bump = ptr_read(cast_ptr[Bump](ctx)); let a: usize = (b.next + align - 1) / align * align; if a + size > b.end { return ptr_null[u8](); } ptr_write(cast_ptr[Bump](ctx), Bump { next: a + size, end: b.end }); return addr_to_ptr[u8](a); } }
fn bump_free(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) -> unit { }
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
fn mk_alloc(state: write Bump) -> Alloc { unsafe "outlives every box" { return Alloc { ctx: cast_ptr[u8](addr_of_mut(state.*)), vt: addr_of(BUMP_VT) }; } }
"#;

fn with_alloc(body: &str) -> String {
    format!(
        "{ALLOC}\nfn main() alloc -> i64 {{\n  let mut bs: Bump = with_window(16777216, 1048576);\n  let al: Alloc = mk_alloc(write bs);\n{body}\n}}"
    )
}

#[test]
fn string_build_and_read_back() {
    // push 'h','i', then append "!!" -> "hi!!" (4 bytes).
    let src = with_alloc(
        "  let mut s: String = string_new(read al);\n\
           push(write s, 104);\n\
           push(write s, 105);\n\
           append(write s, \"!!\");\n\
           let v: str = as_str(read s);\n\
           return conv i64 len(v);",
    );
    assert_eq!(run_ret(&src), 4);
}

#[test]
fn string_as_str_content_matches() {
    let src = with_alloc(
        "  let mut s: String = string_new(read al);\n\
           push(write s, 104);\n\
           push(write s, 105);\n\
           if as_str(read s) == \"hi\" { return 1; }\n\
           return 0;",
    );
    assert_eq!(run_ret(&src), 1);
}

#[test]
fn string_push_multibyte_scalar() {
    // U+00E9 ('é') encodes to 2 bytes; the String byte length reflects that.
    let src = with_alloc(
        "  let mut s: String = string_new(read al);\n\
           push(write s, 233);\n\
           return conv i64 len(as_str(read s));",
    );
    assert_eq!(run_ret(&src), 2);
}

#[test]
fn string_push_surrogate_faults() {
    // 0xD800 is a surrogate — not a Unicode scalar value; the `enforced
    // requires(is_scalar_value(c))` backstop faults (design 0013 §3).
    let src = with_alloc(
        "  let mut s: String = string_new(read al);\n\
           push(write s, 55296);\n\
           return 0;",
    );
    assert_eq!(run_fault(&src), FaultKind::Requires);
}

#[test]
fn string_append_grows_across_reallocs() {
    // Append enough to force several growth reallocations; verify the final view.
    let src = with_alloc(
        "  let mut s: String = string_new(read al);\n\
           append(write s, \"abcde\");\n\
           append(write s, \"fghij\");\n\
           append(write s, \"klmno\");\n\
           let v: str = as_str(read s);\n\
           if v[10] == 107 { return conv i64 len(v); }\n\
           return -1;",
    );
    assert_eq!(run_ret(&src), 15);
}

#[test]
fn string_is_not_portable_across_spawn() {
    // `String` carries a `rawptr` (its buffer + allocator), so it is not
    // `portable` and cannot cross a `scope` spawn by `take` (design 0013 §5).
    let src = format!(
        "{ALLOC}\n\
         fn sink(s: String) alloc -> unit {{ return; }}\n\
         fn main() alloc -> i64 {{\n\
           let mut bs: Bump = with_window(16777216, 1048576);\n\
           let al: Alloc = mk_alloc(write bs);\n\
           let s: String = string_new(read al);\n\
           scope {{ spawn sink(s); }}\n\
           return 0;\n\
         }}"
    );
    assert!(!errors(&src).is_empty(), "String should be rejected as non-portable across spawn");
}

// ---- char_at: value-gear UTF-8 decoder (OBL-TEXT-CHARS) ---------------------

#[test]
fn char_at_decodes_ascii() {
    // 'a' == U+0061 == 97, one byte, next == 1.
    let src = "fn main() -> i64 {\n\
                 let s: str = \"abc\";\n\
                 let step: CharStep = char_at(s, 0);\n\
                 if step.next == 1 { return conv i64 step.cp; }\n\
                 return -1;\n\
               }";
    assert_eq!(run_ret(src), 97);
}

#[test]
fn char_at_decodes_two_byte() {
    // "héllo": the `é` (U+00E9 == 233) starts at byte 1 and is 2 bytes, next == 3.
    let src = "fn main() -> i64 {\n\
                 let s: str = \"héllo\";\n\
                 let step: CharStep = char_at(s, 1);\n\
                 if step.next == 3 { return conv i64 step.cp; }\n\
                 return -1;\n\
               }";
    assert_eq!(run_ret(src), 233);
}

#[test]
fn char_at_decodes_three_byte() {
    // '€' == U+20AC == 8364, three bytes, next == 3.
    let src = "fn main() -> i64 {\n\
                 let s: str = \"€\";\n\
                 let step: CharStep = char_at(s, 0);\n\
                 if step.next == 3 { return conv i64 step.cp; }\n\
                 return -1;\n\
               }";
    assert_eq!(run_ret(src), 8364);
}

#[test]
fn char_at_decodes_four_byte_emoji() {
    // '😀' == U+1F600 == 128512, four bytes, next == 4.
    let src = "fn main() -> i64 {\n\
                 let s: str = \"😀\";\n\
                 let step: CharStep = char_at(s, 0);\n\
                 if step.next == 4 { return conv i64 step.cp; }\n\
                 return -1;\n\
               }";
    assert_eq!(run_ret(src), 128512);
}

#[test]
fn char_at_mid_char_faults() {
    // Byte 2 falls inside the 2-byte `é` of "héllo" — a continuation byte, not a
    // char boundary. The value-gear decoder FAULTS (P5), like `substr`.
    let src = "fn main() -> i64 {\n\
                 let s: str = \"héllo\";\n\
                 let step: CharStep = char_at(s, 2);\n\
                 return conv i64 step.cp;\n\
               }";
    assert_eq!(run_fault(src), FaultKind::Bounds);
}

#[test]
fn char_at_at_end_faults() {
    // `pos == len` has no scalar to decode — an out-of-bounds fault.
    let src = "fn main() -> i64 {\n\
                 let s: str = \"a\";\n\
                 let step: CharStep = char_at(s, 1);\n\
                 return conv i64 step.cp;\n\
               }";
    assert_eq!(run_fault(src), FaultKind::Bounds);
}

// ---- char_count: the O(n) scalar count -------------------------------------

#[test]
fn char_count_mixed_string() {
    // "héllo€": h é l l o € = 6 scalars, but 1+2+1+1+1+3 = 9 bytes.
    let src = "fn main() -> i64 {\n\
                 let s: str = \"héllo€\";\n\
                 return conv i64 char_count(s);\n\
               }";
    assert_eq!(run_ret(src), 6);
}

#[test]
fn char_count_byte_len_differ() {
    // char_count is scalars (6); len is bytes (9). The UTF-8 tax, made visible.
    let src = "fn main() -> i64 {\n\
                 let s: str = \"héllo€\";\n\
                 return conv i64 (len(s) - char_count(s));\n\
               }";
    assert_eq!(run_ret(src), 3);
}

#[test]
fn char_count_empty_is_zero() {
    assert_eq!(run_ret("fn main() -> i64 { return conv i64 char_count(\"\"); }"), 0);
}

#[test]
fn char_count_single_char() {
    assert_eq!(run_ret("fn main() -> i64 { return conv i64 char_count(\"€\"); }"), 1);
}

// ---- the decoder-threading idiom: iterate all chars, exact sequence ---------

#[test]
fn iterate_all_chars_exact_sequence() {
    // Thread the byte position through char_at exactly as the lexer threads its
    // scan cursor (the value-gear idiom; no iterator struct, no borrow). Assert
    // the exact code-point sequence of "héllo€".
    let src = "fn main() -> i64 {\n\
                 let s: str = \"héllo€\";\n\
                 let expected: [6]i64 = [104, 233, 108, 108, 111, 8364];\n\
                 let mut pos: usize = 0;\n\
                 let mut i: usize = 0;\n\
                 while pos < len(s) {\n\
                   let step: CharStep = char_at(s, pos);\n\
                   if conv i64 step.cp != expected[i] { return -1; }\n\
                   pos = step.next;\n\
                   i = i + 1;\n\
                 }\n\
                 if i == 6 { return 1; }\n\
                 return 0;\n\
               }";
    assert_eq!(run_ret(src), 1);
}

#[test]
fn char_at_where_bytes_expected_rejected() {
    // `char_at` takes a `str`, not `[u8]` — no coercion (P2).
    assert!(!errors("fn main() -> i64 { let b: [u8] = b\"hi\"; let s: CharStep = char_at(b, 0); return 0; }").is_empty());
}


// ---- `str ==` / `str !=` lowering (byte-wise, design 0013 §3) ---------------
// The equality operator on `str` is lowered to a length check plus a byte-compare
// loop in the MIR builder, so the MIR interpreter and both native backends inherit
// the tree-walker oracle's semantics: equal iff same length AND identical bytes.
// Each case is byte-exact across the oracle, MIR interp, and Cranelift no-opt/opt
// (`run_ret_all`); the LLVM fifth engine covers `str ==` through `tests/llvm.rs`'s
// full-corpus gate.

#[test]
fn str_eq_equal_is_true() {
    let src = "fn main() -> i64 { if \"abc\" == \"abc\" { return 1; } return 0; }";
    assert_eq!(run_ret_all(src), 1);
}

#[test]
fn str_eq_unequal_same_length_is_false() {
    let src = "fn main() -> i64 { if \"abc\" == \"abd\" { return 1; } return 0; }";
    assert_eq!(run_ret_all(src), 0);
}

#[test]
fn str_eq_unequal_different_length_is_false() {
    // A prefix is not equal even though every shared byte matches — the length
    // guard rejects it before the byte loop can run off the shorter operand.
    let src = "fn main() -> i64 { if \"ab\" == \"abc\" { return 1; } return 0; }";
    assert_eq!(run_ret_all(src), 0);
}

#[test]
fn str_eq_empty_vs_empty_is_true() {
    let src = "fn main() -> i64 { if \"\" == \"\" { return 1; } return 0; }";
    assert_eq!(run_ret_all(src), 1);
}

#[test]
fn str_eq_empty_vs_nonempty_is_false() {
    let src = "fn main() -> i64 { if \"\" == \"x\" { return 1; } return 0; }";
    assert_eq!(run_ret_all(src), 0);
}

#[test]
fn str_ne_is_the_inverse_of_eq() {
    let eq = "fn main() -> i64 { if \"abc\" != \"abc\" { return 1; } return 0; }";
    let ne = "fn main() -> i64 { if \"abc\" != \"abd\" { return 1; } return 0; }";
    assert_eq!(run_ret_all(eq), 0);
    assert_eq!(run_ret_all(ne), 1);
}

#[test]
fn str_eq_multibyte_utf8_is_byte_wise() {
    // `é` is two bytes; equality is over the raw byte run, so equal multibyte
    // strings match and a same-char-count differing pair does not.
    let same = "fn main() -> i64 { if \"héllo\" == \"héllo\" { return 1; } return 0; }";
    let diff = "fn main() -> i64 { if \"héllo\" == \"hello\" { return 1; } return 0; }";
    assert_eq!(run_ret_all(same), 1);
    assert_eq!(run_ret_all(diff), 0);
}

#[test]
fn str_eq_over_bindings_and_substr() {
    // Non-literal `str` operands: a binding compared against a `substr` view.
    let src = "fn main() -> i64 {\n\
                 let s: str = \"hello\";\n\
                 let head: str = substr(s, 0, 3);\n\
                 let c: i64 = 0;\n\
                 if head == \"hel\" { return 1; }\n\
                 return c;\n\
               }";
    assert_eq!(run_ret_all(src), 1);
}
