# Programs written *in* Candor

A catalog of the substantial programs authored **in the Candor language itself**
(not the Rust compiler), how to run them, and how each is verified. These are the
proof that Candor is a real, expressive systems language — and the best things to
show someone.

Every program here is verified the same way the whole project is: **differentially**
— run on multiple independent execution engines and checked to produce the
*byte-identical* result. Candor has five engines (a tree-walking interpreter, a MIR
interpreter, a Cranelift JIT, a Cranelift-optimized native path, and an LLVM `-O2`
native path); a program "passes" only when all of them agree, and where an external
reference exists (e.g. `wasmi` for WebAssembly), against that too.

> Paths below are relative to the repo root. The CLI is `candor` (build it with
> `cargo build --bin candor` inside `compiler/`, then `compiler/target/debug/candor`).
> Tests run with `cargo nextest run` inside `compiler/` (see `.config/nextest.toml`:
> full suite ~3 min; `--profile fast` ~2 s for the logic tests).

---

## 1. A WebAssembly runtime — in Candor  ⭐ the showpiece

**`compiler/tests/fixtures/wasm/interp.cnr`** (~1,900 lines). A from-scratch
WebAssembly interpreter: it decodes a real `.wasm` binary (magic/version, the
type/function/code/memory/data/global/table/element/import sections, LEB128) and
executes it on a value stack.

What it covers (essentially the WASM MVP):
- the full **integer ISA** (i32/i64 arithmetic with correct signed/unsigned and
  div/rem/shift traps, `clz`/`ctz`/`popcount`, `rotl`/`rotr`),
- the full **floating-point ISA** (f32/f64 arithmetic, comparisons, `min`/`max`,
  `sqrt`, `ceil`/`floor`/`trunc`/`nearest`, conversions with WASM's trapping
  semantics),
- **structured control flow** (`block`/`loop`/`if`/`br`/`br_table`/`return`) and
  `call` with **recursion** (via an explicit activation stack, not host recursion),
- **linear memory** (a `Vec[u8]`, little-endian load/store, bounds-trapped, data
  segments, `memory.grow`),
- **globals**, **tables + `call_indirect`** (with the runtime signature-check trap —
  how compiled languages do vtables/function pointers),
- **host imports**, including the real **WASI** `fd_write`/`proc_exit` ABI, so a
  wasm module produces real output.

**How it's verified:** `compiler/tests/wasm.rs` compares the Candor interpreter's
result against **`wasmi`** (an independent, spec-compliant Rust runtime) over ~350
modules — including trap-equivalence — *and* asserts the Candor interpreter itself
gives identical results on all five engines. The differential reference has already
paid off: it caught two bugs in the test harness's own encoder that a hand-written
expected value never would.

```
# run the whole WASM suite (decode, integer/float ISA, wasmi differential, WASI)
cd compiler && cargo nextest run -E 'binary(wasm)'
```

**Demo:** the interpreter reads the shipped `fib10.wasm` / `wasi_hello.wasm`
binaries. See §6 for the standalone "a Candor-compiled native executable that *is* a
WASM interpreter" demo.

---

## 1b. An HTTP/1.0 static-file server — in Candor  ⭐ the network showpiece

**`compiler/tests/fixtures/http_server/main.cnr`** (~600 lines, a boundary module).
A working web server written in Candor: it `listen`s on a TCP port, `accept`s
connections, parses the HTTP request line with the std string utilities
(`starts_with`/`split`/`trim`, plus a `contains "..'` path-traversal guard), maps
`GET /path` to a file under its root, reads it through the std_io wrappers, and
answers `200 OK` (with `Content-Length`, the body built as a `String`) or
`404 Not Found`. Value-first and allocator-explicit throughout (it runs over the
real free-list allocator); all I/O crosses the audited `foreign` boundary, so
`candor audit` lists exactly what it trusts (`sys_tcp_listen`/`accept`/`port` +
the shared read/write/close).

**How it's verified:** `compiler/tests/http_server.rs` runs it on the tree-walker
and the MIR interpreter (a real Rust `TcpStream` client connects and asserts the
exact status lines and body bytes) *and* as a compiled native binary spawned as a
process — same requests, same bytes, exit code = the server's sentinel.

```
cd compiler && cargo nextest run -E 'binary(http_server)'
```

**Demo:** compile it and point a real browser or `curl` at it — it is a genuine,
if deliberately tiny, web server (HTTP/1.0, serves N requests then exits).

---

## 2. The self-hosted compiler — Candor compiling Candor  ⭐

**`selfhost/*.cnr`** (~19,300 lines). The Candor toolchain, written in
Candor: `lexer`, `parser`, `checker`, `analyses` (move/init, the borrow checker's
XOR loans, alloc-effect partition, match exhaustiveness), `interp` (a tree-walking
interpreter), `lower` (AST → MIR), `mono` (monomorphizer), `codegen` (x86-64
assembly), and `layout`.

Four self-hosting tiers, each verified **byte-exact against the Rust reference**:
- **self-check** — the Candor-written checker/analyses check the compiler's own source;
- **self-interpret** — `interp.cnr` runs the systems corpus;
- **self-lower** — `lower.cnr` compiles the corpus to MIR;
- **self-compile** — `codegen.cnr` emits real x86-64 that assembles and runs, with
  **no Rust in the compile path** (just Candor and `as`/`ld`).

```
cd compiler && cargo nextest run -E 'binary(/selfhost/)'
```

---

## 3. The standard library — in Candor

**`compiler/tests/fixtures/corelib/`** and the `std_*` fixtures:
- `core/opt.cnr`, `core/res.cnr` — `Opt`/`Res` + combinators (`map`/`and_then`/`?`),
- `core/arena.cnr`, `std/list.cnr` — an arena and a cons list,
- `std/alloc.cnr`, `std/bump.cnr`, **`std/freelist.cnr`** — the allocator interface, a
  bump allocator, and a **reclaiming free-list allocator** (first-fit + splitting +
  forward/backward coalescing) written over the sanctioned `rawptr` valve,
- `std_fmt.cnr` — `fmt_i64` / `fmt_f64` (round-trip-verified float formatting) + the
  `Show` convention,
- `std_io` — file I/O over an **audited `foreign`/trust boundary**: whole-file
  read/write, `read_lines`, and a streaming `BufReader`/`BufWriter`.

```
cd compiler && cargo nextest run -E 'binary(corelib) | binary(fmt) | binary(std_io) | binary(freelist)'
```

Note: `String`/`Vec`/`Map` are compiler-provided but **compile to native code** on
both backends (allocate + grow + reclaim through the free-list allocator).

---

## 4. The systems corpus — the hardest programs

Five deliberately hard systems programs (`compiler/tests/fixtures/`): a bump
allocator, an intrusive-list scheduler, MMIO registers, a recursive-descent parser,
and a `Box [4096]Node` arena. These are what the self-hosting tiers and the native
backends are all validated against, byte-exact across engines.

```
cd compiler && cargo nextest run -E 'binary(stage_d) | binary(aot) | binary(llvm)'
```

---

## 5. How verification works (the differential story to tell)

The single most convincing thing to show: **the same program produces the
byte-identical result on five independently-built engines**, and for WebAssembly,
identical to `wasmi` too. That's not "the tests pass" — it's "five different
implementations of the language agree, so a bug would have to occur identically in
all of them." The obligations ledger `docs/spec/99-obligations.md` records each
proven property; adversarial design reviews live in `docs/reviews/`.

```
# the full suite: every program, every engine, ~3 minutes
cd compiler && cargo nextest run
```

---

## 6. Show-off demos (CLI)

The CLI runs a Candor program on the interpreter (`run`), or compiles it to a real
native executable (`compile`). Build it once: `cd compiler && cargo build --bin candor`.
**All commands below are verified working** (from the repo root).

```
CANDOR=compiler/target/debug/candor
WASM=compiler/tests/fixtures/wasm/interp.cnr

# (a) run the WASM interpreter on the tree-walker — its embedded module computes 40+2
$CANDOR run $WASM                            # prints 42

# (b) COMPILE the WASM interpreter to a native x86-64 executable and run THAT
$CANDOR compile $WASM -o /tmp/wasmvm && /tmp/wasmvm ; echo "exit = $?"   # -> 42
#   ^ default backend: Cranelift (fast compile, no optimization — the dev build)

# (b′) the RELEASE build: optimized native via the LLVM -O2 backend
$CANDOR compile --release $WASM -o /tmp/wasmvm_rel && /tmp/wasmvm_rel     # -> 42
#   (--backend=llvm is a synonym; Candor -> LLVM IR -> optimized native)

# (b″) THE showpiece: compile it FREESTANDING — a static, NO-libc native binary
$CANDOR compile $WASM -o /tmp/wasmvm_fs --freestanding && /tmp/wasmvm_fs ; echo "exit = $?"
ldd /tmp/wasmvm_fs                           # -> "not a dynamic executable"
#   ^ a no-runtime, no-libc native WebAssembly interpreter, written in Candor

# (c) Candor checks its own compiler source (whole self-host module tree, via tests)
cd compiler && cargo nextest run -E 'binary(/selfhost/)' ; cd ..
#   (a single self-host module can't be `candor check`ed alone — it `use`s the others;
#    the self-host tests load the module tree and verify byte-exact vs the Rust oracle.)

# (d) audit the I/O trust boundary — machine-readable JSON of every extern + its effect
$CANDOR audit compiler/tests/fixtures/std_io
```

---

## Where these live (quick reference)

| Program | Path | ~Lines of Candor |
|---|---|---|
| WebAssembly runtime | `compiler/tests/fixtures/wasm/interp.cnr` | 1,900 |
| HTTP/1.0 static-file server | `compiler/tests/fixtures/http_server/main.cnr` | 600 |
| Self-hosted compiler | `selfhost/*.cnr` | 19,300 |
| Standard library | `compiler/tests/fixtures/corelib/`, `std_*.cnr` | — |
| Systems corpus | `compiler/tests/fixtures/11_*.cnr` etc. | — |
| Verification harnesses (Rust) | `compiler/tests/*.rs` | — |
