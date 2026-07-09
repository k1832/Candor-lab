//! The shape library: grammar-directed generators, one family per idiom in
//! `docs/specpack/idioms.md`. Each generator draws its parameters from the seeded
//! `Rng`, emits a well-formed `.cnr` program, and — crucially — states what the
//! toolchain must confirm:
//!
//! * a POSITIVE shape computes its own sentinel (the value `main` returns), so
//!   the filter can require `run` to print exactly it;
//! * a NEGATIVE shape is authored to trip ONE specific diagnostic, so the filter
//!   can require `check` to emit exactly that code.
//!
//! Scope honesty: these shapes are HAND-AUTHORED, not spec-exhaustive. Coverage
//! grows by adding shapes here (see `README.md#scope`).

use crate::manifest::Expected;
use crate::rng::Rng;
use serde_json::json;

/// A generated candidate program plus what the toolchain must confirm about it.
pub struct Candidate {
    pub shape: &'static str,
    pub program: String,
    pub params: serde_json::Value,
    pub expected: Expected,
}

/// A shape generator: a name plus a function from the RNG to a candidate.
pub type Gen = fn(&mut Rng) -> Candidate;

/// The positive shapes (well-formed by construction; sentinel filtered).
pub const POSITIVE: &[(&str, Gen)] = &[
    ("arith_checked", p_arith_checked),
    ("arith_wrapping", p_arith_wrapping),
    ("enum_match", p_enum_match),
    ("question_propagate", p_question_propagate),
    ("generic_id", p_generic_id),
    ("generic_bound_copy", p_generic_bound_copy),
    ("struct_drop_trace", p_struct_drop_trace),
    ("arena_index", p_arena_index),
    ("for_indexed", p_for_indexed),
    ("borrow_reborrow", p_borrow_reborrow),
];

/// The negative shapes (each designed to trip exactly one diagnostic; code filtered).
pub const NEGATIVE: &[(&str, Gen)] = &[
    ("neg_use_after_move", n_use_after_move),
    ("neg_copy_with_drop", n_copy_with_drop),
    ("neg_borrow_field", n_borrow_field),
    ("neg_two_ok_variants", n_two_ok_variants),
    ("neg_unknown_type", n_unknown_type),
    ("neg_write_needs_deref", n_write_needs_deref),
    ("neg_missing_return", n_missing_return),
    ("neg_wrong_arg_count", n_wrong_arg_count),
    ("neg_break_outside_loop", n_break_outside_loop),
    ("neg_unknown_variant", n_unknown_variant),
    ("neg_unknown_field", n_unknown_field),
    ("neg_literal_out_of_range", n_literal_out_of_range),
    ("neg_non_exhaustive_match", n_non_exhaustive_match),
    ("neg_alloc_effect", n_alloc_effect),
    ("neg_copy_bound", n_copy_bound),
];

// ----------------------------------------------------------------------------
// Positive shapes
// ----------------------------------------------------------------------------

/// Idiom: wrapping arithmetic (grammar `+ - * & | ^`), non-wrapping regime.
/// Constants chosen so the checked result is in range (no fault) and computed here.
fn p_arith_checked(r: &mut Rng) -> Candidate {
    let ops = ["+", "-", "*", "|", "&", "^"];
    let op = ops[r.index(ops.len())];
    let a = r.range(0, 1000);
    let b = r.range(0, 1000);
    let value = match op {
        "+" => a + b,
        "-" => a - b,
        "*" => a * b,
        "|" => a | b,
        "&" => a & b,
        "^" => a ^ b,
        _ => unreachable!(),
    };
    let program = format!(
        "// shape: arith_checked — non-wrapping i64 arithmetic; sentinel computed by the generator\n\
         fn main() -> i64 {{\n    let a: i64 = {a};\n    let b: i64 = {b};\n    return a {op} b;\n}}\n"
    );
    Candidate {
        shape: "arith_checked",
        program,
        params: json!({ "a": a, "b": b, "op": op }),
        expected: Expected::Sentinel { value },
    }
}

/// Idiom: wrapping arithmetic — deliberate i64 overflow contained by `wrapping`.
/// The sentinel is the two's-complement wrap the generator computes.
fn p_arith_wrapping(r: &mut Rng) -> Candidate {
    let b = r.range(1, 1000);
    let a = i64::MAX;
    let value = a.wrapping_add(b);
    let program = format!(
        "// shape: arith_wrapping — deliberate i64 overflow under `wrapping`; sentinel = wrapping_add\n\
         fn main() -> i64 {{\n    let mut r: i64 = 0;\n    wrapping {{ r = {a}i64 + {b}; }}\n    return r;\n}}\n"
    );
    Candidate {
        shape: "arith_wrapping",
        program,
        params: json!({ "a": a, "b": b }),
        expected: Expected::Sentinel { value },
    }
}

/// Idiom: result-shaped enum + exhaustive `match`.
fn p_enum_match(r: &mut Rng) -> Candidate {
    let v = r.range(-500, 500);
    let d = r.range(-500, 500);
    let c = r.boolean();
    let value = if c { v } else { d };
    let program = format!(
        "// shape: enum_match — result-shaped enum + exhaustive match\n\
         enum Opt[T] {{ ok Some(T), None }}\n\
         fn pick(c: bool) -> Opt[i64] {{\n    if c {{ return Opt::Some({v}); }}\n    return Opt::None;\n}}\n\
         fn main() -> i64 {{\n    match pick({c}) {{\n        Opt::Some(v) => {{ return v; }}\n        Opt::None => {{ return {d}; }}\n    }}\n}}\n"
    );
    Candidate {
        shape: "enum_match",
        program,
        params: json!({ "v": v, "d": d, "c": c }),
        expected: Expected::Sentinel { value },
    }
}

/// Idiom: `?` propagation on an `ok`-marked result enum (same-type, no `From`).
fn p_question_propagate(r: &mut Rng) -> Candidate {
    let v = r.range(-500, 500);
    let k = r.range(0, 500);
    let value = v + k;
    let program = format!(
        "// shape: question_propagate — `?` unwraps the ok variant of a result-shaped enum\n\
         enum Res[T, E] {{ ok Ok(T), Err(E) }}\n\
         enum Fail {{ Bad }}\n\
         fn step(c: bool) -> Res[i64, Fail] {{\n    if c {{ return Res::Ok({v}); }}\n    return Res::Err(Fail::Bad);\n}}\n\
         fn drive(c: bool) -> Res[i64, Fail] {{\n    let x: i64 = step(c)?;\n    return Res::Ok(x + {k});\n}}\n\
         fn main() -> i64 {{\n    match drive(true) {{\n        Res::Ok(v) => {{ return v; }}\n        Res::Err(e) => {{ return -1; }}\n    }}\n}}\n"
    );
    Candidate {
        shape: "question_propagate",
        program,
        params: json!({ "v": v, "k": k }),
        expected: Expected::Sentinel { value },
    }
}

/// Idiom: generic fn + monomorphization at distinct type arguments.
fn p_generic_id(r: &mut Rng) -> Candidate {
    let a = r.range(-500, 500);
    let b = r.range(-500, 500);
    let value = a + b;
    let program = format!(
        "// shape: generic_id — generic identity, monomorphized at two i64 sites\n\
         fn id[T](x: T) -> T {{ return x; }}\n\
         fn main() -> i64 {{\n    let a: i64 = id({a});\n    let b: i64 = id({b});\n    return a + b;\n}}\n"
    );
    Candidate {
        shape: "generic_id",
        program,
        params: json!({ "a": a, "b": b }),
        expected: Expected::Sentinel { value },
    }
}

/// Idiom: generic fn with a `copy` bound (`unwrap_or`), two instantiations.
fn p_generic_bound_copy(r: &mut Rng) -> Candidate {
    let v = r.range(-500, 500);
    let d1 = r.range(-500, 500);
    let d2 = r.range(-500, 500);
    let value = v + d2;
    let program = format!(
        "// shape: generic_bound_copy — generic fn under a `copy` bound (unwrap_or)\n\
         enum Opt[T] {{ ok Some(T), None }}\n\
         fn unwrap_or[T: copy](o: Opt[T], d: T) -> T {{\n    match o {{ Opt::Some(v) => {{ return v; }} Opt::None => {{ return d; }} }}\n}}\n\
         fn main() -> i64 {{\n    let s: Opt[i64] = Opt::Some({v});\n    let n: Opt[i64] = Opt::None;\n    return unwrap_or(s, {d1}) + unwrap_or(n, {d2});\n}}\n"
    );
    Candidate {
        shape: "generic_bound_copy",
        program,
        params: json!({ "v": v, "d1": d1, "d2": d2 }),
        expected: Expected::Sentinel { value },
    }
}

/// Idiom: struct + `drop` hook + `trace` (drop-order harness pattern). The drop
/// trace `[id]` is recorded in params; the filter observes the return sentinel.
fn p_struct_drop_trace(r: &mut Rng) -> Candidate {
    let id = r.range(0, 1000);
    let k = r.range(0, 1000);
    let value = id + k;
    let program = format!(
        "// shape: struct_drop_trace — struct with a drop hook that traces its id\n\
         struct R {{ id: i64 }} drop(write self) {{ trace(self.id); }}\n\
         fn main() -> i64 {{\n    let r: R = R {{ id: {id} }};\n    let x: i64 = r.id;\n    return x + {k};\n}}\n"
    );
    Candidate {
        shape: "struct_drop_trace",
        program,
        params: json!({ "id": id, "k": k, "expected_trace": [id] }),
        expected: Expected::Sentinel { value },
    }
}

/// Idiom: arena + inline `[N]T` backing + index gear (`copy` bound, write-path `.*`).
fn p_arena_index(r: &mut Rng) -> Candidate {
    let v0 = r.range(0, 300);
    let v1 = r.range(0, 300);
    let v2 = r.range(0, 300);
    let value = v0 + v1 + v2;
    let program = format!(
        "// shape: arena_index — fixed [8]T arena, push/get by value (copy bound)\n\
         struct Arena[T: copy] {{ mem: [8]T, count: u32 }}\n\
         fn push[T: copy](ar: write Arena[T], x: T) -> bool {{\n    if ar.*.count >= 8u32 {{ return false; }}\n    let i: u32 = ar.*.count;\n    ar.*.mem[conv usize i] = x;\n    ar.*.count = i + 1u32;\n    return true;\n}}\n\
         fn get[T: copy](ar: read Arena[T], i: u32) -> T {{ return ar.mem[conv usize i]; }}\n\
         fn main() -> i64 {{\n    let mut ar: Arena[i64] = Arena {{ mem: [0, 0, 0, 0, 0, 0, 0, 0], count: 0u32 }};\n    let p0: bool = push(write ar, {v0});\n    let p1: bool = push(write ar, {v1});\n    let p2: bool = push(write ar, {v2});\n    return get(read ar, 0u32) + get(read ar, 1u32) + get(read ar, 2u32);\n}}\n"
    );
    Candidate {
        shape: "arena_index",
        program,
        params: json!({ "v0": v0, "v1": v1, "v2": v2 }),
        expected: Expected::Sentinel { value },
    }
}

/// Idiom: `for x in read coll` over the Indexed protocol (non-alloc ground floor).
fn p_for_indexed(r: &mut Rng) -> Candidate {
    let v0 = r.range(0, 300);
    let v1 = r.range(0, 300);
    let v2 = r.range(0, 300);
    let value = v0 + v1 + v2;
    let program = format!(
        "// shape: for_indexed — for-loop over the Indexed protocol (borrows, copies items out)\n\
         enum Opt[T] {{ ok Some(T), None }}\n\
         interface Indexed {{ type Item; fn at(read self, i: usize) -> Opt[Item]; }}\n\
         struct Arena[T: copy] {{ mem: [8]T, count: u32 }}\n\
         fn push[T: copy](ar: write Arena[T], x: T) -> bool {{\n    if ar.*.count >= 8u32 {{ return false; }}\n    let i: u32 = ar.*.count;\n    ar.*.mem[conv usize i] = x;\n    ar.*.count = i + 1u32;\n    return true;\n}}\n\
         impl[T: copy] Indexed for Arena[T] {{ type Item = T; fn at(read self, i: usize) -> Opt[T] {{ if i >= conv usize self.count {{ return Opt::None; }} return Opt::Some(self.mem[i]); }} }}\n\
         fn main() -> i64 {{\n    let mut ar: Arena[i64] = Arena {{ mem: [0, 0, 0, 0, 0, 0, 0, 0], count: 0u32 }};\n    let p0: bool = push(write ar, {v0});\n    let p1: bool = push(write ar, {v1});\n    let p2: bool = push(write ar, {v2});\n    let mut s: i64 = 0;\n    for x in read ar {{ s = s + x; }}\n    return s;\n}}\n"
    );
    Candidate {
        shape: "for_indexed",
        program,
        params: json!({ "v0": v0, "v1": v1, "v2": v2 }),
        expected: Expected::Sentinel { value },
    }
}

/// Idiom: borrow/reborrow chains — `write` mutation then `read` observation.
fn p_borrow_reborrow(r: &mut Rng) -> Candidate {
    let t0 = r.range(-300, 300);
    let d1 = r.range(-300, 300);
    let d2 = r.range(-300, 300);
    let value = t0 + d1 + d2;
    let program = format!(
        "// shape: borrow_reborrow — write-borrow mutation, then read-borrow observation\n\
         struct Acc {{ total: i64 }}\n\
         fn bump(a: write Acc, d: i64) -> unit {{ a.*.total = a.total + d; }}\n\
         fn peek(a: read Acc) -> i64 {{ return a.total; }}\n\
         fn main() -> i64 {{\n    let mut acc: Acc = Acc {{ total: {t0} }};\n    bump(write acc, {d1});\n    bump(write acc, {d2});\n    return peek(read acc);\n}}\n"
    );
    Candidate {
        shape: "borrow_reborrow",
        program,
        params: json!({ "t0": t0, "d1": d1, "d2": d2 }),
        expected: Expected::Sentinel { value },
    }
}

// ----------------------------------------------------------------------------
// Negative shapes — each trips exactly one diagnostic. `nonce` only varies the
// file bytes (a comment), never the semantics, so the emitted code is stable.
// ----------------------------------------------------------------------------

fn nonce(r: &mut Rng) -> i64 {
    r.range(1000, 9999)
}

fn neg(shape: &'static str, code: &'static str, program: String, params: serde_json::Value) -> Candidate {
    Candidate {
        shape,
        program,
        params,
        expected: Expected::Diagnostic { code: code.to_string() },
    }
}

/// E0301 — use of a moved value.
fn n_use_after_move(r: &mut Rng) -> Candidate {
    let n = nonce(r);
    let program = format!(
        "// shape: neg_use_after_move — expect E0301 (nonce {n})\n\
         struct S {{ v: i64 }}\n\
         fn consume(s: S) -> i64 {{ return s.v; }}\n\
         fn main() -> i64 {{\n    let a: S = S {{ v: 1 }};\n    let x: i64 = consume(a);\n    let y: i64 = consume(a);\n    return x + y;\n}}\n"
    );
    neg("neg_use_after_move", "E0301", program, json!({ "nonce": n }))
}

/// E0202 — a `copy` struct may not have a drop hook.
fn n_copy_with_drop(r: &mut Rng) -> Candidate {
    let n = nonce(r);
    let program = format!(
        "// shape: neg_copy_with_drop — expect E0202 (nonce {n})\n\
         copy struct Tag {{ id: i64 }} drop(write self) {{ trace(self.id); }}\n\
         fn main() -> i64 {{ return 0; }}\n"
    );
    neg("neg_copy_with_drop", "E0202", program, json!({ "nonce": n }))
}

/// E0201 — a struct field may not have a borrow type.
fn n_borrow_field(r: &mut Rng) -> Candidate {
    let n = nonce(r);
    let program = format!(
        "// shape: neg_borrow_field — expect E0201 (nonce {n})\n\
         struct Holder {{ r: read i64, tag: i64 }}\n\
         fn main() -> i64 {{ return 0; }}\n"
    );
    neg("neg_borrow_field", "E0201", program, json!({ "nonce": n }))
}

/// E0109 — more than one `ok`-marked variant.
fn n_two_ok_variants(r: &mut Rng) -> Candidate {
    let n = nonce(r);
    let program = format!(
        "// shape: neg_two_ok_variants — expect E0109 (nonce {n})\n\
         enum Sig {{ ok A(i64), ok B(i64) }}\n\
         fn main() -> i64 {{ return 0; }}\n"
    );
    neg("neg_two_ok_variants", "E0109", program, json!({ "nonce": n }))
}

/// E0102 — unknown type.
fn n_unknown_type(r: &mut Rng) -> Candidate {
    let n = nonce(r);
    let program = format!(
        "// shape: neg_unknown_type — expect E0102 (nonce {n})\n\
         fn main() -> i64 {{ let x: Widget = other(); return 0; }}\n"
    );
    neg("neg_unknown_type", "E0102", program, json!({ "nonce": n }))
}

/// E0713 — a write through a borrow needs an explicit `.*`.
fn n_write_needs_deref(r: &mut Rng) -> Candidate {
    let n = nonce(r);
    let program = format!(
        "// shape: neg_write_needs_deref — expect E0713 (nonce {n})\n\
         struct S {{ v: i64 }}\n\
         fn setit(s: write S) -> unit {{ s.v = 5; }}\n\
         fn main() -> i64 {{ let mut a: S = S {{ v: 1 }}; setit(write a); return a.v; }}\n"
    );
    neg("neg_write_needs_deref", "E0713", program, json!({ "nonce": n }))
}

/// E0810 — control may reach the end of a non-unit function.
fn n_missing_return(r: &mut Rng) -> Candidate {
    let n = nonce(r);
    let program = format!(
        "// shape: neg_missing_return — expect E0810 (nonce {n})\n\
         fn main() -> i64 {{ let x: i64 = 5; }}\n"
    );
    neg("neg_missing_return", "E0810", program, json!({ "nonce": n }))
}

/// E0706 — wrong argument count.
fn n_wrong_arg_count(r: &mut Rng) -> Candidate {
    let n = nonce(r);
    let program = format!(
        "// shape: neg_wrong_arg_count — expect E0706 (nonce {n})\n\
         fn add(a: i64, b: i64) -> i64 {{ return a + b; }}\n\
         fn main() -> i64 {{ return add(1); }}\n"
    );
    neg("neg_wrong_arg_count", "E0706", program, json!({ "nonce": n }))
}

/// E0707 — `break` outside a loop.
fn n_break_outside_loop(r: &mut Rng) -> Candidate {
    let n = nonce(r);
    let program = format!(
        "// shape: neg_break_outside_loop — expect E0707 (nonce {n})\n\
         fn main() -> i64 {{ break; return 0; }}\n"
    );
    neg("neg_break_outside_loop", "E0707", program, json!({ "nonce": n }))
}

/// E0108 — enum has no such variant.
fn n_unknown_variant(r: &mut Rng) -> Candidate {
    let n = nonce(r);
    let program = format!(
        "// shape: neg_unknown_variant — expect E0108 (nonce {n})\n\
         enum Color {{ Red, Green }}\n\
         fn main() -> i64 {{ let c: Color = Color::Blue; return 0; }}\n"
    );
    neg("neg_unknown_variant", "E0108", program, json!({ "nonce": n }))
}

/// E0107 — type has no such field.
fn n_unknown_field(r: &mut Rng) -> Candidate {
    let n = nonce(r);
    let program = format!(
        "// shape: neg_unknown_field — expect E0107 (nonce {n})\n\
         struct S {{ v: i64 }}\n\
         fn main() -> i64 {{ let s: S = S {{ v: 1 }}; return s.w; }}\n"
    );
    neg("neg_unknown_field", "E0107", program, json!({ "nonce": n }))
}

/// E0709 — integer literal out of range for its type.
fn n_literal_out_of_range(r: &mut Rng) -> Candidate {
    let n = nonce(r);
    let lit = r.range(256, 999);
    let program = format!(
        "// shape: neg_literal_out_of_range — expect E0709 (nonce {n})\n\
         fn main() -> i64 {{ let x: u8 = {lit}u8; return 0; }}\n"
    );
    neg("neg_literal_out_of_range", "E0709", program, json!({ "nonce": n, "lit": lit }))
}

/// E0601 — non-exhaustive match.
fn n_non_exhaustive_match(r: &mut Rng) -> Candidate {
    let n = nonce(r);
    let program = format!(
        "// shape: neg_non_exhaustive_match — expect E0601 (nonce {n})\n\
         enum Opt[T] {{ ok Some(T), None }}\n\
         fn main() -> i64 {{ let o: Opt[i64] = Opt::Some(1); match o {{ Opt::Some(v) => {{ return v; }} }} }}\n"
    );
    neg("neg_non_exhaustive_match", "E0601", program, json!({ "nonce": n }))
}

/// E0401 — a non-`alloc` function performs allocation (dropping a `Box`-bearing value).
fn n_alloc_effect(r: &mut Rng) -> Candidate {
    let n = nonce(r);
    let program = format!(
        "// shape: neg_alloc_effect — expect E0401 (nonce {n})\n\
         enum L {{ Nil, Cons(i64, Box L) }}\n\
         fn drop_it(l: L) -> i64 {{ return 0; }}\n\
         fn main() -> i64 {{ return 0; }}\n"
    );
    neg("neg_alloc_effect", "E0401", program, json!({ "nonce": n }))
}

/// E1007 — type argument does not satisfy a `copy` bound.
fn n_copy_bound(r: &mut Rng) -> Candidate {
    let n = nonce(r);
    let program = format!(
        "// shape: neg_copy_bound — expect E1007 (nonce {n})\n\
         fn need_copy[T: copy](x: T) -> T {{ return x; }}\n\
         struct NoCopy {{ v: i64 }}\n\
         fn main() -> i64 {{ let a: NoCopy = NoCopy {{ v: 1 }}; let b: NoCopy = need_copy(a); return 0; }}\n"
    );
    neg("neg_copy_bound", "E1007", program, json!({ "nonce": n }))
}
