# 0016 — Floating point (`f64`)

Status: accepted (prototype slice). Adds one IEEE-754 binary64 scalar type to
Candor. This is a design *note*, not a full adversarial-review round: floats are
standard; only the decisions below need pinning.

## 1. Scope

- **`f64` first.** One new scalar type, `f64` (IEEE-754 binary64). `f32` is
  deferred to a later slice.
- Size 8, align 8. `f64` is `copy` and drop-inert (a scalar leaf, exactly like
  `i64`), and `portable` (owned value data, no borrow/rawptr).

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

`f32`; float **formatting** (`fmt_f64` — a separate slice); transcendental / math
functions (`sqrt`, `sin`, …); WASM float opcodes (now unblocked); the full
NaN-payload / signaling-NaN edge cases; a flexible float-literal type.
