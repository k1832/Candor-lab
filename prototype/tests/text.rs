//! Design 0013 — text and strings. `str` as a core, immutable, UTF-8-validated
//! view (type-distinct from `[u8]`, no coercion), `b"..."` byte-string literals,
//! the core operation surface (byte `len`/index/compare, `substr` with a
//! char-boundary fault, `as_bytes`, `str_from`), byte iteration via the existing
//! `Indexed` desugar, and the std `String` builder (`push`/`append`/`as_str`).
//! Single-file, real (`.cnr`) front-end.

use candor_proto::diag::Severity;
use candor_proto::interp::FaultKind;
use candor_proto::{check_source_real, run_source_real, RunResult};

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
    assert_eq!(run_ret("fn main() -> i64 { let s: str = substr(\"héllo\", 0, 3); return conv i64 len(s); }"), 3);
}

#[test]
fn substr_mid_char_faults() {
    // Offset 2 falls inside the 2-byte `é` — a non-boundary slice is a bug (P5).
    assert_eq!(
        run_fault("fn main() -> i64 { let s: str = substr(\"héllo\", 0, 2); return 0; }"),
        FaultKind::Bounds
    );
}

#[test]
fn substr_out_of_bounds_faults() {
    assert_eq!(
        run_fault("fn main() -> i64 { let s: str = substr(\"abc\", 0, 9); return 0; }"),
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
    assert_eq!(run_ret("fn main() -> i64 { let b: [u8] = as_bytes(\"hello\"); return conv i64 len(b); }"), 5);
}

#[test]
fn str_from_valid_bytes() {
    let src = "fn main() -> i64 {\n\
                 match str_from(b\"hi\") {\n\
                   Utf8Res::Valid(s) => { return conv i64 len(s); }\n\
                   Utf8Res::Invalid(off) => { return -1; }\n\
                 }\n\
               }";
    assert_eq!(run_ret(src), 2);
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
    assert_eq!(run_ret(src), 1);
}

#[test]
fn str_from_then_use() {
    // Roundtrip: as_bytes a str, revalidate, index the recovered view.
    let src = "fn main() -> i64 {\n\
                 let bytes: [u8] = as_bytes(\"abc\");\n\
                 match str_from(bytes) {\n\
                   Utf8Res::Valid(s) => { return conv i64 s[2]; }\n\
                   Utf8Res::Invalid(off) => { return -1; }\n\
                 }\n\
               }";
    assert_eq!(run_ret(src), 99);
}

// ---- String (std, compiler-known): push / append / as_str ------------------

const ALLOC: &str = r#"
struct AllocVtable { alloc: fn(ctx: rawptr u8, size: usize, align: usize) alloc -> rawptr u8, free: fn(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) alloc -> unit }
copy struct Alloc { ctx: rawptr u8, vt: rawptr AllocVtable }
struct Bump { next: usize, end: usize }
fn with_window(base: usize, size: usize) -> Bump { return Bump { next: base, end: base + size }; }
fn bump_alloc(ctx: rawptr u8, size: usize, align: usize) -> rawptr u8 { unsafe "reserved window" { let b: Bump = ptr_read(cast_ptr[Bump](ctx)); let a: usize = (b.next + align - 1) / align * align; if a + size > b.end { return ptr_null[u8](); } ptr_write(cast_ptr[Bump](ctx), Bump { next: a + size, end: b.end }); return addr_to_ptr[u8](a); } }
fn bump_free(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) -> unit { }
static BUMP_VT: AllocVtable = AllocVtable { alloc: bump_alloc, free: bump_free };
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


