# Installing the Candor 0.x preview

Three steps: build the toolchain, install the `candor` command, install the
editor extension. Everything here is preview-quality; see the maturity note at
the end.

## 1. Build the toolchain from the prototype

The compiler is a Rust crate. Building it produces the `candor` binary.

```sh
# in the toolchain/ directory (the Candor crate at the repo root):
cargo build --release
# -> target/release/candor
```

A debug build (`cargo build`, at `target/debug/candor`) works too; the
release build is faster to run.

Requirements:
- A recent stable Rust toolchain (`cargo`).
- A C compiler (`cc`) on PATH — `candor compile` links the emitted object into an
  ELF through it. (`--freestanding` links with `-nostdlib -static -no-pie` and
  needs no libc at runtime, but still uses `cc` to link.)

## 2. Install the `candor` command

`cargo build --release` produces the `candor` binary directly. Put it on your
PATH:

```sh
cp toolchain/target/release/candor ~/.local/bin/     # (a dir already on PATH)
```

Verify:

```sh
candor run examples/01_hello.cnr     # -> 42
```

## 3. Install the VS Code extension

The editor support lives in `editor/vscode/` (TextMate grammar +
`language-configuration.json`) plus the `editor/lsp/` diagnostics server.

```sh
# syntax highlighting: copy/symlink the extension into your VS Code extensions dir
ln -s "$PWD/editor/vscode" ~/.vscode/extensions/candor
# reload VS Code; open any .cnr file to see highlighting.
```

For live diagnostics, build the LSP (`cargo build --release` in `editor/lsp/`)
and point the extension at it per that tool's own README. Diagnostics are
the P4 machine-readable JSON the compiler already emits, surfaced in the editor.

## Maturity

This is a 0.x preview with no stability promise. The command surface, the
standard-library seed, and the editor integration are all expected to change.
Build from source; there are no published binaries.
