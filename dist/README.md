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
> from it). Read `LANG_PHYLOSOPHY.md` and `docs/design/` there for the why behind
> every decision, and `docs/BET5_CRITERION.md` for the frozen founding bet.

---

## 60-second getting started

Build the toolchain and install the `candor` command (full detail in
[INSTALL.md](INSTALL.md)):

```sh
# from the prototype crate:
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
- **[preview]** VS Code support: syntax highlighting + a diagnostics LSP.
- **[seed]** Standard library: a small `core`/`std` seed (`Opt`, `Res`, `Arena`,
  `List`, iteration, a bump allocator). Text handling and an I/O layer are
  designed-but-not-yet-shipped.

Nothing here is stable. See the lab's obligations ledger for what 1.0 still owes.
