# Candor for VS Code

Editor support for the **Candor** language, in two layers:

- **Layer 1 — syntax highlighting** (declarative, no runtime): a TextMate
  grammar (`syntaxes/candor.tmLanguage.json`) plus `language-configuration.json`.
  Works with no server and no `npm install`.
- **Layer 2 — diagnostics** (optional): a thin
  [`vscode-languageclient`](https://www.npmjs.com/package/vscode-languageclient)
  client (`extension.js`) that launches the `candor-lsp` binary and shows the
  prototype checker's errors inline.

## Scope (honest, and narrow)

This is **not** a semantic language server. There is **no hover, no completion,
and no go-to-definition** — by design. Those are semantic-analysis features that
belong to the real Candor toolchain, and are deferred per **P16**. Layer 2 does
exactly one thing: it runs the `candor_proto` check pipeline and publishes
diagnostics (parse + resolve + Stage-2 check).

## File extensions

- `.cnr` — the **real** toolchain surface syntax (spec 01/02). This is what the
  grammar targets and what the LSP checks with `check_source_real`.
- `.cn` — the **throwaway** prototype syntax. It is registered as the *same*
  language for highlighting convenience. **Caveat:** the grammar is faithful to
  the `.cnr` real syntax; `.cn` uses retired spellings (`slice`, `borrow`,
  `deref`, `case`, `slice_mut`, …) that this grammar does **not** special-case,
  so some `.cn`-only tokens will highlight as plain identifiers. The LSP checks
  `.cn` with the throwaway `check_source` pipeline. Highlighting for `.cn` is
  "close enough"; author real code in `.cnr`.

## Install

### Quick dev install (symlink)

Highlighting only, no build step:

```sh
ln -s "$PWD/tools/vscode-candor" ~/.vscode/extensions/vscode-candor
```

Reload VS Code. Open `sample.cnr` to see the grammar in action.

### Package + install (recommended)

For the diagnostics client you need its one npm dependency, then package:

```sh
cd tools/vscode-candor
npm install                 # pulls vscode-languageclient
npx @vscode/vsce package    # produces vscode-candor-0.1.0.vsix
code --install-extension vscode-candor-0.1.0.vsix
```

### Build the LSP server

```sh
cd tools/candor-lsp
cargo build --release       # -> tools/candor-lsp/target/release/candor-lsp
```

The extension looks for the server in this order:
1. the `candor.lsp.serverPath` setting, if set;
2. `tools/candor-lsp/target/release/candor-lsp` next to the extension;
3. `candor-lsp` on `PATH`.

If the binary is missing, highlighting still works and the client shows a
one-time warning.

## Settings

| Setting | Default | Meaning |
|---|---|---|
| `candor.lsp.enabled` | `true` | Start the diagnostics server on activation. |
| `candor.lsp.serverPath` | `""` | Absolute path to `candor-lsp` (empty = auto-detect). |
| `candor.lsp.trace.server` | `off` | Trace JSON-RPC traffic. |

## The unsafe justification string is the audit surface

`unsafe "<justification>" { … }` requires a mandatory justification string
(spec 02 §6.9). That string is the human audit surface, so the grammar gives it
its **own** scope — distinct from ordinary strings — following P13's
*rare-dangerous-loud* principle:

```
string.quoted.double.unsafe-justification.candor
```

TextMate grammars can pick the *scope* but cannot force a theme to render it
boldly. To make it truly loud, drop this into your `settings.json`:

```jsonc
"editor.tokenColorCustomizations": {
  "textMateRules": [
    {
      "scope": "string.quoted.double.unsafe-justification.candor",
      "settings": { "foreground": "#ff5555", "fontStyle": "bold underline" }
    },
    {
      "scope": "keyword.control.unsafe.candor",
      "settings": { "foreground": "#ff5555", "fontStyle": "bold" }
    }
  ]
}
```

## Grammar scope reference (for theming)

| Construct | Scope |
|---|---|
| Control flow (`if else match loop while break continue return`) | `keyword.control.candor` |
| Contracts (`requires ensures assert panic`) | `keyword.control.contract.candor` |
| `unsafe` keyword | `keyword.control.unsafe.candor` |
| unsafe justification string | `string.quoted.double.unsafe-justification.candor` |
| Regimes (`wrapping saturating`) | `keyword.control.regime.candor` |
| Declarations (`fn struct enum static copy drop let`) | `keyword.declaration*.candor` |
| Declared names | `entity.name.function.candor`, `entity.name.type.candor` |
| Passing modes / borrows (`take read write out`) | `storage.modifier.mode.candor` |
| `mut` | `storage.modifier.mut.candor` |
| Contextual (`alloc`, `pub use region`) | `keyword.other.effect.candor`, `keyword.other.contextual.candor` |
| `ok` variant marker | `keyword.other.ok.candor` |
| Word operators (`conv clone`) | `keyword.operator.word.candor` |
| Scalar types (`i8..usize bool unit`) | `support.type.builtin.candor` |
| Type constructors (`rawptr Box BoxResult`) | `support.type.candor` |
| Intrinsics (`sizeof alignof min_of max_of offsetof field_ptr cast_ptr addr_to_ptr ptr_null`) | `support.function.builtin.candor` |
| `.*` deref / `?` propagate / `::` path | `keyword.operator.dereference|propagation|path.candor` |
| Bitwise / shift / logical / comparison / arithmetic | `keyword.operator.*.candor` |
| Int literals (+ suffix) | `constant.numeric.*.candor` (suffix: `keyword.other.numeric.suffix.candor`) |
| Booleans | `constant.language.boolean.candor` |
| Enum variant members | `variable.other.enummember.candor` |
| `SCREAMING_SNAKE` statics | `constant.other.candor` |

### Known highlighting limitations (reported, not worked around)

- **`alloc` / `pub` / `use` / `region` / `impl` / `for`** are *contextual*
  keywords (spec 01 §2.5; design 0007/0008): they are keywords only in a
  grammatical slot and identifiers elsewhere. A TextMate grammar cannot decide
  position, so these are colored wherever they appear — a deliberate
  over-approximation. `ok` is handled precisely (only when it leads a variant:
  `ok Name`).
- The **negative-literal fold** (`-9223...i64`) is grammatical, not lexical
  (spec 01 §3.4), so the `-` highlights as the arithmetic operator and the
  digits as a numeric literal — which is exactly the token structure.
- The `[T]` / `[N]T` / `Foo[T]` brackets share the one `[ ]` bracket pair with
  index/array-literal `[ ]` (there is no `<…>` in Candor — `OBL-GENERIC-BRACKET`),
  so bracket highlighting does not distinguish type brackets from index brackets.
