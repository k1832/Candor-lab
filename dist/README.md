# Candor

Candor is a systems programming language built for a world where code is written
by a human and an LLM working as one author. It is memory-safe, explicit where
meaning lives, and locally verifiable: you can check a function by reading it and
its signatures, not the whole program. Its semantics are declared in the source —
the same program means the same thing in every build mode — and the compiler is
built to be a conversation partner (machine-readable diagnostics, one canonical
formatting, a grammar that parses without a symbol table) rather than a
gatekeeper. It targets the systems core — allocators, drivers, runtimes,
codecs — and competes with C, C++, Rust, and Zig on their home ground.

---

## ⚠️ 0.x — UNSTABLE PREVIEW

**This is a preview. There is no stability promise of any kind.** Syntax,
semantics, diagnostics, the standard library, and the command-line surface can
all change without notice or migration path before 1.0. Do not build anything
load-bearing on it yet.

The design record — the philosophy, twelve adversarially-reviewed designs, the
normative spec, and the frozen **Bet 5** memory-model experiment that this
language is founded on — lives in the lab repository. This preview ships *from*
that lab; the lab is the authority. 1.0 is a stability *gate*, not a date.

> Lab / design record: **the Candor lab repository** (this preview was staged
> from it). Read `LANG_PHILOSOPHY.md` and `docs/design/` there for the why behind
> every decision, and `docs/BET5_CRITERION.md` for the frozen founding bet.

---

## 60-second getting started

Build the toolchain and install the `candor` command (full detail in
[INSTALL.md](INSTALL.md)):

```sh
# from the toolchain/ directory:
cargo build --release            # produces target/release/candor
# then put the `candor` command on your PATH (see INSTALL.md):
cp target/release/candor ~/.local/bin/   # (a dir already on PATH)
```

Write `hello.cnr`:

```candor
fn main() -> i64 {
    trace(1);
    trace(2);
    return 42;
}
```

Run it and explore the toolchain:

```sh
candor run hello.cnr             # runs it; prints the return value (42)
candor compile hello.cnr -o hello   # AOT-compile to a native ELF; ./hello prints the trace, exits 42
candor fmt hello.cnr --check     # the one canonical format (no options, no bikeshedding)
candor build examples/05_modules # incremental, per-module build of a module tree
candor audit examples/07_boundary   # report every FFI boundary + its trust predicates
```

**Freestanding, in one line** — the same compiler links a static, `no-libc`
binary (`-nostdlib -static -no-pie`, raw syscalls, no runtime):

```sh
candor compile --freestanding hello.cnr -o hello && ldd hello   # => "not a dynamic executable"
```

---

## Showcase — two real programs, written in Candor

Not snippets — substantial programs you can read, run, and point at:

**A WebAssembly interpreter** (`examples/11_wasm_interp.cnr`, ~2,000 lines of
Candor): decodes a real `.wasm` binary (LEB128, the section format) and executes
it — the integer and float ISA, structured control flow, linear memory,
`call_indirect`, WASI — running its embedded module to 42:

```sh
candor run examples/11_wasm_interp.cnr        # -> 42
candor compile examples/11_wasm_interp.cnr -o wasmvm && ./wasmvm   # exits 42
```

**An HTTP/1.0 static-file server** (`examples/12_http_server/`, ~600 lines):
listens, parses requests with the std string utilities, serves files (with a
path-traversal guard), answers 200/404 — then curl it:

```sh
cd examples/12_http_server
candor run main.cnr        # serving on http://127.0.0.1:8080 (8 requests)
# from another terminal:
curl http://127.0.0.1:8080/hello.txt          # -> Hello, Candor!
curl http://127.0.0.1:8080/                   # -> the index page
```

It serves 8 requests and exits itself. (A compiled `candor compile main.cnr -o
httpd && ./httpd` serves identically. If you relaunch immediately and port 8080
is still in TIME_WAIT, wait a moment and retry.)

And the parts too big to ship as examples, on the public record in the
[lab repository](https://github.com/k1832/Candor-lab): the **self-hosted
compiler** (~19,300 lines of Candor that check, interpret, lower, and compile
Candor, verified byte-exact against the reference), the **five-engine
differential verification** every program here passes, and the frozen **Bet 5**
memory-model experiment the language is founded on.

---

## Examples

Runnable, formatted programs in [examples/](examples/) — each verified against
this toolchain:

| Example | Run with | Shows |
|---|---|---|
| `01_hello.cnr` | `candor run` | `trace`, the `i64` sentinel return |
| `02_checked_arith.cnr` | `candor run` | checked arithmetic + `wrapping`; **faults by design** (overflow) |
| `03_errors.cnr` | `candor run` | result-shaped `enum` + `match` + `?` + `From` widening |
| `04_generic_container.cnr` | `candor run` | a generic container and the `for` loop |
| `05_modules/` | `candor run` / `candor build` | a small multi-file module tree |
| `06_concurrency.cnr` | `candor run` | `scope`/`spawn` with race-free disjoint writes |
| `07_boundary/` | `candor run` / `candor audit` | an FFI boundary module + the audit surface |
| `08_ordering.cnr` | `candor run` | `Ord` on scalars, `[T: Ord]` bounds, `min`/`max`/`sort_ord` |
| `09_strings.cnr` | `candor run` | string utilities — `split`/`join`/`contains`/`trim` |
| `10_iterators.cnr` | `candor run` | iterator adapters + terminals composing (`take_n`/`enumerate`/`fold`/`find`) |
| `11_wasm_interp.cnr` | `candor run` / `candor compile` | ⭐ a from-scratch WebAssembly interpreter (see Showcase) |
| `12_http_server/` | `candor run` (from its dir) | ⭐ an HTTP/1.0 static-file server over the audited TCP boundary (see Showcase) |

---

## Features (with honest maturity tags)

Tags: **[working]** runs today · **[preview]** works but rough/partial ·
**[seed]** a starting sketch, expected to grow.

- **[working]** Value-first, memory-safe core — moves, `read`/`write`/`out`/`take`
  borrow modes, drop, no GC. (The founding Bet 5, *provisionally confirmed* on the
  lab's public record.)
- **[working]** Checked arithmetic by default; explicit `wrapping` blocks for
  modular arithmetic.
- **[working]** A deterministic fault model: the same fault identity (kind + span)
  across the interpreter, native JIT, and AOT binary.
- **[working]** Result-shaped enums, `match`, and `?` with `From`-based error
  widening.
- **[working]** Generics with bounds (`copy`, `portable`), monomorphized;
  interfaces + associated types; `for` over two iteration protocols.
- **[working]** Multi-file modules with an incremental, per-module build.
- **[working]** Structured concurrency: `scope`/`spawn` on real OS threads with
  compile-time race freedom.
- **[working]** An FFI boundary module (`extern`/`export`/`trust`) with a
  `candor audit` surface, and an effect system (`alloc`, `foreign`).
- **[working]** Contracts: `requires`/`ensures`/`assert`, enforced as faults.
- **[working]** AOT native compilation (x86-64 via Cranelift) and a
  `--freestanding` no-libc profile.
- **[working]** One canonical `fmt`; machine-readable JSON diagnostics.
- **[preview]** VS Code support: syntax highlighting + a diagnostics LSP, in
  `editor/` (`editor/vscode/` grammar, `editor/lsp/` server).
- **[preview]** Standard library in `stdlib/` (`candor run stdlib` drives the
  whole surface): `Opt`/`Res`/`Arena`/`List`; allocators (a bump allocator and a
  reclaiming free-list); a comparison interface (`Ord` implemented for the scalar
  types, `[T: Ord]` bounds) with `min`/`max`/`sort`/`sort_ord`; a composable
  iterator surface (adapters `take_n`/`skip_n`/`enumerate`/`zip`/`take_while`/
  `skip_while` + terminals `find`/`nth`/`collect`/`count`/`fold`/`any`/`all`); and
  string utilities (`split`/`join`/`trim`/`contains`/`find`/`starts_with`). The
  builtin collections `Vec`/`Map`/`String` compile to native code on both backends.
- **[preview]** I/O over the audited foreign boundary (`candor audit` lists the
  trust surface): files (whole-file read/write + a buffered `BufReader`/
  `BufWriter`), directory listing, and TCP client sockets — real I/O under both
  `candor run` (the interpreter) and `candor compile`d native binaries.

Nothing here is stable. See `VERSIONING.md` for what 0.x and 1.0 mean, and the
lab's obligations ledger for what 1.0 still owes.

---

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
