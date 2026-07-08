// Candor VS Code extension — activation glue.
//
// Two layers, one extension:
//   Layer 1 (syntax highlighting) is fully declarative: the grammar in
//   syntaxes/candor.tmLanguage.json + language-configuration.json. It needs no
//   JavaScript and works even if the LSP is disabled or the server binary is
//   missing.
//   Layer 2 (diagnostics) is this file: the standard vscode-languageclient glue
//   that launches the candor-lsp binary over stdio and lets it push
//   publishDiagnostics into the editor.
//
// Scope is intentionally narrow (P16): the server does diagnostics only — no
// hover, completion, or go-to-definition. Those await the real Candor toolchain.

const fs = require("fs");
const path = require("path");
const { workspace, window } = require("vscode");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

let client;

/**
 * Resolve the candor-lsp binary path:
 *   1. the `candor.lsp.serverPath` setting, if set;
 *   2. the release build next to this extension in the repo
 *      (tools/candor-lsp/target/release/candor-lsp);
 *   3. otherwise `candor-lsp`, trusting the user's PATH.
 */
function resolveServerPath(configuredPath) {
  if (configuredPath && configuredPath.trim().length > 0) {
    return configuredPath.trim();
  }
  const exe = process.platform === "win32" ? "candor-lsp.exe" : "candor-lsp";
  // this extension lives at tools/vscode-candor; the server at tools/candor-lsp.
  const local = path.join(
    __dirname,
    "..",
    "candor-lsp",
    "target",
    "release",
    exe
  );
  if (fs.existsSync(local)) {
    return local;
  }
  return exe; // fall back to PATH
}

function activate(context) {
  const config = workspace.getConfiguration("candor");
  if (config.get("lsp.enabled") === false) {
    return;
  }

  const serverPath = resolveServerPath(config.get("lsp.serverPath"));

  const serverOptions = {
    command: serverPath,
    args: [],
    transport: TransportKind.stdio,
  };

  const clientOptions = {
    documentSelector: [{ scheme: "file", language: "candor" }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/*.{cnr,cn}"),
    },
    diagnosticCollectionName: "candor",
  };

  client = new LanguageClient(
    "candorLsp",
    "Candor LSP",
    serverOptions,
    clientOptions
  );

  client.start().catch((err) => {
    window.showWarningMessage(
      "Candor: could not start candor-lsp (" +
        serverPath +
        "). Syntax highlighting still works; set candor.lsp.serverPath to enable diagnostics. " +
        String(err)
    );
  });
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
