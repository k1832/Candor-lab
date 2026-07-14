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
functions beyond `sqrt` (`sin`, `cos`, `exp`, …; `sqrt` landed — see §11); the full
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


## 10. `bitcast` — same-width float<->int BIT reinterpretation — landed

`conv` (§5, §9.2) converts the numeric **value** (`conv i64 (1.0)` == `1`). `bitcast`
reinterprets the identical **bits** as a same-width type (`bitcast i64 (1.0)` ==
`0x3FF0000000000000`). It is the prerequisite for the WASM float opcodes (whose
interpreter holds floats as `i64` stack bits and must reinterpret them to do
arithmetic) and is generally useful (float hashing/serialization, WASM's own
`reinterpret` ops).

### 10.1 Spelling — a `bitcast <ty>` keyword, parallel to `conv`

`bitcast T (e)` is a prefix operator keyword, uniform with `conv T (e)`: the prototype
grammar takes `bitcast T ( e )` (parenthesized operand), the real/surface grammar the
paren-free `bitcast ScalarKw <prefix-operand>` (exactly as `conv`). Chosen over
intrinsic functions (`f64_to_bits`/…) because a keyword mirrors `conv` one-for-one —
same parse slot, same precedence (prefix), same emit/format — and needs no per-type
family of builtins.

### 10.2 Semantics — same-BYTE-WIDTH float<->int only; total; regime-independent

- **Legal pairs.** Exactly one side a float, the other an integer of the **same byte
  width**: `f64 <-> {i64, u64, isize, usize}` (8 bytes), `f32 <-> {i32, u32}` (4
  bytes). The rule is byte-width equality with one float side (so `isize`/`usize`, both
  8 bytes, pair with `f64`) — the principled generalization of "f64<->i64, f32<->i32".
- **Rejected at check time** (`E0714`): a different-width bitcast (`f64 <-> i32`), a
  both-integer bitcast (`i64 <-> u64` — that is `wrapping { conv }`, §6 / obligations
  U64<->I64), a both-float bitcast (`f64 <-> f32` — also different width), and a
  float<->bool/unit or non-scalar target.
- **No value change.** `bitcast i64 (1.0)` == `0x3FF0000000000000`; `bitcast f64
  (0x4000000000000000)` == `2.0`; `bitcast i32 (1.0f32)` == `0x3F800000`.
- **Total.** Bitcast NEVER faults (unlike `conv`'s checked-overflow / the inert
  saturating float->int edge). It carries no fault edge.
- **Regime-independent.** Like a float op (§2), a bitcast inside a
  `wrapping`/`saturating` block is unchanged — it is a pure reinterpret with no
  overflow notion, so the regime system has nothing to act on. Documented and gated.
- **A bare `{integer}` on the int side** (e.g. `bitcast f64 (0x4000000000000000)`) is
  width-flexible: it takes the float's same-width UNSIGNED integer (`u64`/`u32`) so its
  full bit pattern is preserved (a high-bit pattern such as `0xFFF8…` needs an explicit
  `u64` suffix, since a suffix-less literal is range-checked against `i64`).

### 10.3 NaN payload — preserved EXACTLY

Bitcast is precisely where a **specific** NaN bit-pattern survives: there is no
arithmetic to canonicalize the payload/sign, so `bitcast i64 (bitcast f64
(0x7FF8000000000001))` reproduces `0x7FF8000000000001` bit-for-bit on every engine.
This is the case bitcast uniquely handles — a *computed* NaN's sign is
IEEE-unspecified across a constant-folding compiler vs. runtime hardware (§4, §6), so
`tests/floats.rs` gates computed NaN by behaviour; `tests/bitcast.rs` gates a
reinterpreted NaN by EXACT bits.

### 10.4 Implementation — a new MIR op, native `bitcast`/re-canonicalize (NOT `fcvt`)

Threaded through every layer as a sibling of `conv`, but as a distinct, simpler op:
`ast::ExprKind::Bitcast { ty, expr }`, checker `check_bitcast` (the §10.2 pair rule),
both interpreters, and a **new** MIR rvalue `Bitcast { to, v }`. A new op (not a
`conv` "reinterpret" mode) is cleaner because `conv` carries `regime` + a `fault`
edge (INV-CHECK) that a total, regime-free bitcast would only have to null out;
`Bitcast` carries neither and falls through INV-CHECK like a float op. Its wire form
is `(bitcast <to-scalar> <operand>)` — additive, so existing `conv` MIR and self-host
lowering parity are untouched.

Because floats are already carried as their `to_bits()` pattern in the shared
`i128`/`i64` register and byte-identically in flat memory (§6, §9.3), a bitcast changes
no bits: it only **re-canonicalizes** the operand's held pattern to the target's
width/signedness. The interpreters use `fit_bits(x, to)`; both backends use the
existing `canon` (a **no-op at 64 bits**, `ireduce`/`trunc` + `sext`/`zext` at 32) —
which IS the raw register-level reinterpret, and pointedly NOT an `fcvt` /
`fpto*i` / `*tofp` (those would change the value). Nothing in load/store/call-ABI
changes.

### 10.5 Cross-engine gate

`tests/bitcast.rs` + `tests/fixtures/run/bitcast.cnr` assert BIT-IDENTICAL results
across all five engines (tree-walker / MIR / Cranelift no-opt / Cranelift -O2 / LLVM
-O2): f64<->i64 and f32<->i32 round trips over a spread; the known encodings above for
f64 and f32; the exact-NaN-payload round trip; regime-independence (a bitcast inside
`wrapping {}`); and the `E0714` rejections (different-width, both-integer, both-float).
The 64-bit `trace` channel widens a 32-bit result with `conv i64 (bitcast u32 (..))`
so the widening zero-extends, matching host `f32::to_bits()` (a `u32`).


## 11. `sqrt` — the correctly-rounded square-root intrinsic — landed

Square root is the one float op that CANNOT be expressed bit-exactly in Candor
source: a Newton/Heron iteration converges to one ULP short of the correctly-rounded
IEEE result (see the `tests/floats.rs` note and the WASM float slice, where
`f*.sqrt` was deferred for exactly this reason). It therefore ships as a compiler
intrinsic that lowers to each backend's NATIVE square root, so every engine is
bit-identical to hardware and to IEEE.

### 11.1 Spelling — a builtin function `sqrt(x)`, overloaded by argument type

`sqrt` is spelled as an ordinary call `sqrt(x)` (a compiler-known builtin, like
`len`/`is_null`; no new keyword), OVERLOADED by the argument's float type: `f32 ->
f32` and `f64 -> f64`. A single name (not `sqrt_f32`/`sqrt_f64`) is possible because
the checker already resolves builtins by argument type — the `check_builtin` `"sqrt"`
arm reads the argument's scalar type and returns the same type, rejecting a
non-float argument. It shadows a same-named user function, exactly as the other
builtins do.

### 11.2 Semantics — total, non-faulting, correctly-rounded

`sqrt` is a pure, TOTAL unary float->float op. It NEVER faults: `sqrt` of a negative
is NaN (IEEE, not a fault), `sqrt(-0.0) == -0.0`, `sqrt(0.0) == 0.0`, `sqrt(+inf) ==
+inf`, `sqrt(NaN)` is NaN. Being non-faulting it carries no fault edge and is
regime-independent (like a float arithmetic op — §2). IEEE square root is
correctly-rounded and deterministic, so — unlike an arithmetic-generated NaN whose
sign is unspecified — a FINITE `sqrt` result is bit-identical across a
constant-folding compiler and runtime hardware.

### 11.3 Implementation — a new MIR op, NATIVE sqrt in both backends

A dedicated `Rvalue::Sqrt { ty, v }` (parallel to `Bitcast`), threaded through the
same layers a unary float op touches: lexer/parser unchanged (it is a call);
`check/expr.rs` (`check_builtin`), both interpreters (tree-walker `eval_builtin` +
`mir/interp.rs`, via a `float_sqrt` helper over `f{32,64}::sqrt` — Rust's are
correctly-rounded), `mir/build.rs` (`is_builtin` + `lower_builtin_value`),
`mir/serial.rs` (wire round-trip), `mir/opt.rs` (pure & non-faulting, so DCE-eligible
when dead). Both native backends emit the NATIVE square root — Cranelift's `sqrt`
instruction (`sqrtsd`/`sqrtss`) and LLVM's `@llvm.sqrt.f64`/`@llvm.sqrt.f32`
intrinsic — NOT a software approximation: the register's bit pattern is reinterpreted
as a float, the native sqrt is taken, and the result is reinterpreted back.

### 11.4 Cross-engine gate

`tests/sqrt.rs` + `tests/fixtures/run/{sqrt,sqrt_f32}.cnr` assert BIT-IDENTICAL
results across all five engines (tree-walker / MIR / Cranelift no-opt / -O2 / LLVM
-O2) for f32 and f64: known values (`sqrt(4.0) == 2.0`, `sqrt(2.0)` to its exact
bits, `sqrt(0.0) == 0.0`, `sqrt(-0.0) == -0.0`, `sqrt(1e100)`), a round-trip
`sqrt(x) * sqrt(x)`, and a negative argument yielding NaN — gated by IS-NAN
behaviour (the NaN sign is IEEE-unspecified across a folding compiler vs. runtime),
proving it is a value, not a fault. The intrinsic also closes the WASM interpreter's
`f32.sqrt` (0x91) / `f64.sqrt` (0x9f) deferral: those opcodes now bitcast the operand
slot to the float, call `sqrt`, and bitcast back, gated BIT-EXACT vs `wasmi` in
`tests/wasm.rs` (`diff_float_sqrt`, `diff_vector_length` — a `sqrt(x*x + y*y)` length).
