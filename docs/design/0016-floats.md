# 0016 — Floating point (`f64`, `f32`)

Status: accepted (prototype slice). Adds the IEEE-754 binary64 (`f64`) and
binary32 (`f32`) scalar types to Candor. This is a design *note*, not a full
adversarial-review round: floats are standard; only the decisions below need
pinning. `f64` landed first (§1–§8); `f32` mirrors it (§9).

## 1. Scope

- **`f64` first, then `f32`.** Two new scalar types, `f64` (IEEE-754 binary64)
  and `f32` (IEEE-754 binary32). `f64` landed first; `f32` (§9) mirrors it.
- `f64`: size 8, align 8. `f32`: size 4, align 4. Both are `copy` and drop-inert
  (a scalar leaf, exactly like `i64`), and `portable` (owned value data, no
  borrow/rawptr).

## 2. IEEE semantics — floats are EXEMPT from the arithmetic regime system

The regime system (design 0006 §2.4: `checked` / `wrapping` / `saturating`, with
overflow/div-by-zero faults) is **integer-only**. `f64` arithmetic is always pure
IEEE-754:

- `+ - * /` and unary `-` never fault. Overflow produces `±inf`; `x / 0.0`
  produces `±inf` (or `NaN` for `0.0/0.0`); no `Overflow`/`DivByZero` fault is
  ever raised for a float op.
- A `wrapping { }` / `saturating { }` block does **not** change float behaviour —
  a float op inside one is still plain IEEE. (Confirmed reading of 0006 §2.4,
  whose fault machinery is defined over integer widths / two's-complement range;
  it has no meaning for a float and is not applied to one.)
- `%` (`Rem`) is **not** defined for `f64` in this slice (integer-only). Bitwise
  ops / shifts remain integer-only.

## 3. Literal grammar

- A numeric literal is `f64` iff it contains a `.` **or** an exponent (`e`/`E`);
  otherwise it is an integer literal (unchanged).
- A `.` literal must have **a digit on both sides**: `0.5`, `3.14`, `2.0` are
  floats; `.5` and `5.` are rejected (clarity). `1e10`, `1.0e-5`, `6.022e23` are
  floats via the exponent. An integer-suffixed literal (`5i32`) stays integer;
  float literals take no suffix in this slice (a `.`/exponent literal is always
  `f64`, a concrete type — there is no flexible float-literal type like the
  integer `{integer}`).
- A literal whose magnitude exceeds `f64` range parses to `±inf` (Rust
  `str::parse::<f64>` semantics), documented, not an error.

## 4. NaN / inf

`NaN` and `±inf` exist and flow through arithmetic. Comparisons follow IEEE: any
ordered/`==` comparison involving `NaN` is `false`; `!=` with a `NaN` operand is
`true` (so `NaN != NaN` is `true`, `NaN < 1.0` is `false`, `NaN == NaN` is
`false`). Produced e.g. by `0.0/0.0` (NaN) and `1.0/0.0` (`+inf`). NaN payload /
signaling-NaN edge cases are out of scope.

## 5. Conversions (`conv`)

No implicit int↔float promotion — Candor is explicit; mixed int/float arithmetic
is a type error. Conversions are explicit `conv`:

- **`conv f64 <int>`** (int→f64): always succeeds; rounds to nearest, ties to
  even (exact for `|x| < 2^53`). Not regime-sensitive.
- **`conv i64 <f64>` / `conv i32 <f64>` / …** (f64→int): truncates toward zero,
  **saturating** on out-of-range (`NaN → 0`, `+inf →` target max, `-inf →` target
  min). Not regime-sensitive — consistent with floats being IEEE/regime-exempt,
  and identical to Rust's `as`, Cranelift `fcvt_to_*int_sat`, and LLVM
  `llvm.fpto*i.sat` (so all engines agree bit-for-bit). A `conv` out of a float
  therefore never faults, in any regime.

Rationale for saturating (vs. a checked fault): floats are regime-exempt, and the
three lowerings above are the only cross-engine-deterministic truncation with a
defined out-of-range result, so it keeps the five-engine gate bit-identical.

## 6. Value representation (implementation)

The interpreters and both native backends carry every scalar as its sign/bit-
correct value in an `i128`/`i64` "register" and 8 bytes in flat memory. An `f64`
is carried as its **`f64::to_bits()` 64-bit pattern** (zero-extended) in that same
slot — flat memory is byte-identical to the IEEE encoding. Each float op bit-casts
the pattern to a native `double`, computes with real IEEE arithmetic (Rust `f64`
in the tree-walker / MIR interp; `fadd`/`fsub`/`fmul`/`fdiv`/`fcmp`/`fneg` +
`fcvt`/`sitofp`/`fptosi.sat` in Cranelift / LLVM), and bit-casts back. Nothing in
the load/store/call-ABI machinery changes (still an 8-byte word). `ty_range(f64)`
is defined as unsigned-64 so no path sign-extends the bit pattern.

Because IEEE-754 `f64` is deterministic on the shared x86-64 target, all five
engines (tree-walker, MIR interp, Cranelift no-opt/opt, LLVM -O2) produce
**bit-identical** results, asserted via `f64::to_bits` in `tests/floats.rs`.

## 7. Deferred (noted, not built)

**shortest** round-trip formatting (Ryū/Grisu) and scientific-notation
output (the initial `fmt_f64` slice landed — see §8); transcendental / math
functions (`sqrt`, `sin`, …); WASM float opcodes (now unblocked); the full
NaN-payload / signaling-NaN edge cases; a flexible float-literal type.


## 8. Formatting (`fmt_f64`) — landed

Float-to-`String` rendering lives in the `String` formatting image
`prototype/tests/fixtures/std_fmt.cnr` (alongside `fmt_i64`), written in Candor
over this slice's `f64` ops. It is a *defined, documented* format — not the
shortest round trip (Ryū), which is deferred (§7):

- **Format.** Fixed-point decimal, up to 15 significant digits, trailing zeros
  stripped. `NaN`→"NaN", `±inf`→"inf"/"-inf", `0.0` and `-0.0`→"0" (sign of
  zero dropped), finite negatives signed. Examples: `1.5`→"1.5", `10.0`→"10",
  `6.022e23`→"602200000000000000000000".
- **Round-trip guarantee.** Any `f64` nearest to a ≤ 15-significant-digit decimal
  with magnitude in `[1e-15, 1e39)` re-parses to identical bits (checked in
  `tests/fmt.rs` against Rust `f64::from_str`; 30M-sample validated). Values that
  need 16–17 significant digits (unrounded `sqrt(2)`, `0.1 + 0.2`, …) do **not**
  round-trip, and magnitudes outside the covered range are best-effort.
- **Precision limit (why 15).** Digits are extracted with `f64`-only arithmetic:
  normalize `|x|` to a decimal exponent by comparison against a power-of-ten table,
  then form the significand with ONE scaled multiply/divide (`< 10^15 < 2^53`, an
  exact integer). A 16th–17th correct digit would need an integer past `f64`'s
  exact `2^53` range, unreachable without a bitcast or bignum (neither in this
  slice) — a genuine `f64`-only limit, not a bug. The single scaled op also avoids
  the rounding drift a repeated-`*10` digit loop accumulates.
- **Cross-engine gate.** The String path is MIR-interp-only (CollectionOp), so a
  String-free twin, `tests/fixtures/run/fmt_f64_trace.cnr`, reproduces the same
  algorithm and TRACES the ASCII bytes; the corpus gates prove those bytes are
  byte-identical across all five engines.


## 9. `f32` (IEEE-754 binary32) — landed

`f32` mirrors `f64` exactly, at single precision. Everything in §2 (regime-EXEMPT
IEEE), §4 (NaN/inf), and §6 (value representation) applies unchanged, with 4-byte
single precision in place of 8-byte double. Only the two decisions where `f32`
differs from `f64` are pinned here.

### 9.1 Literal grammar — an `f32` *suffix* on a float form

`f64` literals are suffix-free: a suffix-less `.`/exponent literal is always `f64`
(§3). An `f32` literal is spelled by adding the **`f32` suffix** to a *float-form*
literal:

- `1.5f32`, `10.0f32`, `1.0e-5f32`, `6.022e23f32` are `f32`.
- The suffix attaches ONLY to a float form (a `.` with a digit on both sides, or
  an exponent). An **integer form + `f32` is rejected**: `10f32` is a lex error
  (the integer-literal suffix space stays integer-only). Write `10.0f32`.
- `f32` is the only float-literal suffix. Any other suffix on a float form —
  including `f64` — is rejected (`1.5f64` is a lex error, `L0005`), keeping `f64`
  the suffix-free default float type.
- The text before the suffix is parsed at the literal's precision
  (`str::parse::<f32>`), so an over-range magnitude yields `±inf` (as `f64` does).

Rationale: a suffix is far more usable than forcing every `f32` through
`conv f32 (..)`, and confining the float suffix to float forms keeps a clean rule
— *integer* suffixes on integer forms, the *`f32`* suffix on float forms — with no
ambiguity. There is still no flexible float-literal type: `f32`/`f64` are concrete.

### 9.2 Conversions (`conv`) — scope

No implicit promotion between `f32`, `f64`, and integers (mixed operands are a type
error; `f32 + f64` is rejected exactly like `f64 + i64`). All crossings are explicit
`conv`, all IEEE and regime-exempt (never fault):

- **`conv f32 <int>`** (int→f32): rounds to nearest, ties to even.
- **`conv f64 <f32>`** (widening): **exact** (`fpromote` / `fpext`).
- **`conv f32 <f64>`** (narrowing): rounds to nearest; may lose precision and may
  overflow to `±inf` (`fdemote` / `fptrunc`). A value that survives f64→f32→f64
  therefore differs in bits from the original f64 — asserted in the gate.
- **`conv i{N} <f32>`** (f32→int): truncates toward zero, **saturating**
  (NaN→0, out-of-range clamps), identical to Rust `as` / Cranelift
  `fcvt_to_*int_sat` / LLVM `llvm.fpto*i.sat.iN.f32`.
- **`conv f32 <f32>`** / **`conv f64 <f64>`** are identity.

### 9.3 Value representation & how `f32` is distinguished from `f64`

An `f32` is carried as its `f32::to_bits()` 32-bit pattern **zero-extended** into
the same i128/i64 register slot, and **4 bytes** in flat memory. `ty_range(f32)` is
unsigned-32, so the pattern is never sign-extended (as `f64` is unsigned-64).

No new width tag was needed anywhere. The existing `ScalarTy` tag already
distinguishes `F32` from `F64`, and it is carried at every op site that needs it —
`Operand::Const(_, ScalarTy)`, and the `ty: ScalarTy` field on MIR `Bin`/`Un`/`Conv`
— while `Cmp` recovers the width from its operands' scalar type (both operands are
the same float type, checker-guaranteed). `ast::ExprKind::FloatLit` / the real
`Float` token gained a `ty: ScalarTy` field (`F32` or `F64`) so the literal's width
survives to lowering. The Conv wire format is still UNCHANGED (source and target
recovered from the operand type / `to` field), preserving self-host MIR parity.

Each float op bit-casts the pattern to a native single (`f32` in the interpreters;
Cranelift `bitcast` after an `ireduce`/`uextend`; LLVM `trunc`/`bitcast` +
`bitcast`/`zext`), computes real IEEE `f32` arithmetic, and casts back. Because
IEEE-754 binary32 is deterministic on the shared x86-64 target, all five engines
produce **bit-identical** results, asserted via `f32::to_bits` in
`tests/floats_f32.rs` and the `tests/fixtures/run/floats_f32.cnr` native corpus
fixture. NaN keeps the §4 caveat: a computed NaN's sign is IEEE-unspecified across
engines, so NaN is gated by behaviour, not exact bits.
