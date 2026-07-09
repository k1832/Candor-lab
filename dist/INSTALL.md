# Installing the Candor 0.x preview

Three steps: build the toolchain, install the `candor` command, install the
editor extension. Everything here is preview-quality; see the maturity note at
the end.

## 1. Build the toolchain from the prototype

The compiler is a Rust crate. Building it produces the binary `candor-proto`
(the shim below turns that into the `candor` command).

```sh
# in the prototype crate directory:
cargo build --release
# -> target/release/candor-proto
```

A debug build (`cargo build`, at `target/debug/candor-proto`) works too; the
release build is faster to run.

Requirements:
- A recent stable Rust toolchain (`cargo`).
- A C compiler (`cc`) on PATH — `candor compile` links the emitted object into an
  ELF through it. (`--freestanding` links with `-nostdlib -static -no-pie` and
  needs no libc at runtime, but still uses `cc` to link.)

## 2. Install the `candor` command

The prototype's Cargo `[[bin]]` is still named `candor-proto`. The `bin/candor`
wrapper in this tree gives you the real `candor <command>` surface by forwarding
every argument to that binary. Point it at your build one of two ways:

```sh
# (a) tell the wrapper exactly where the binary is:
export CANDOR_PROTO=/absolute/path/to/target/release/candor-proto
# and put the wrapper on your PATH:
export PATH="/absolute/path/to/dist/bin:$PATH"

# (b) OR put the built binary on PATH under its own name and just use the wrapper:
cp /path/to/target/release/candor-proto ~/.local/bin/     # (a dir already on PATH)
export PATH="/absolute/path/to/dist/bin:$PATH"            # wrapper finds `candor-proto` on PATH
```

Verify:

```sh
candor run examples/01_hello.cnr     # -> 42
```

> **Packaging TODO (the real rename).** The shim exists only because renaming the
> crate binary mid-session is risky. The permanent fix is a one-line change in the
> prototype's `Cargo.toml` — `[[bin]] name = "candor-proto"` becomes
> `name = "candor"` — after which `cargo build` produces `candor` directly and
> `bin/candor` can be deleted. This is the intended first commit in the standalone
> distribution repo.

## 3. Install the VS Code extension

The editor support lives in the `vscode-candor` tool (TextMate grammar +
`language-configuration.json`) plus the `candor-lsp` diagnostics server.

```sh
# syntax highlighting: copy/symlink the extension into your VS Code extensions dir
ln -s /path/to/tools/vscode-candor ~/.vscode/extensions/candor
# reload VS Code; open any .cnr file to see highlighting.
```

For live diagnostics, build the LSP (`cargo build --release` in the `candor-lsp`
crate) and point the extension at it per that tool's own README. Diagnostics are
the P4 machine-readable JSON the compiler already emits, surfaced in the editor.

## Maturity

This is a 0.x preview with no stability promise. The command surface, the shim,
the standard-library seed, and the editor integration are all expected to change.
Build from source; there are no published binaries.
