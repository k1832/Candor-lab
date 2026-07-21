#!/bin/sh
# seed.sh — assemble a standalone Candor 0.x distribution repository.
#
# This IMPLEMENTS the operator actions in MANIFEST.md ("## Operator actions"):
# it copies only the SHIPS rows out of the lab and lays them out as the root of
# a self-contained repo that builds the `candor` toolchain and runs the examples
# and the stdlib seed with nothing pointing back at the lab.
#
# Usage:  ./seed.sh [target-dir]        (default: ./candor-0.x)
# Run it from the lab's dist/ directory (its own location); it locates the lab
# root as dist/'s parent, so an absolute or relative target both work.
#
# The target must not already exist (refuses rather than clobbering).

set -eu

# --- locate the lab (this script lives in <lab>/dist/) -----------------------
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
DIST_DIR=$SCRIPT_DIR
LAB_ROOT=$(CDPATH= cd -- "$DIST_DIR/.." && pwd)

TARGET=${1:-./candor-0.x}
if [ -e "$TARGET" ]; then
    echo "seed.sh: target '$TARGET' already exists; remove it or pick another path." >&2
    exit 1
fi
mkdir -p "$TARGET"
TARGET=$(CDPATH= cd -- "$TARGET" && pwd)

echo "Seeding a standalone Candor 0.x distribution"
echo "  lab    : $LAB_ROOT"
echo "  target : $TARGET"
echo

# --- 1. toolchain crate: compiler/ -> toolchain/ ----------------------------
# Ship src/, Cargo.toml, Cargo.lock, benches/, .gitignore (MANIFEST SHIPS row).
# EXCLUDE tests/ and selfhost/: neither is a Cargo target the manifest declares
# (no [[test]]/[[example]], no build.rs), so the crate builds `candor` without
# them and they only bloat the distribution. `target/` is a build artifact.
echo "-> toolchain/  (from compiler/: src, Cargo.toml, Cargo.lock, benches)"
mkdir -p "$TARGET/toolchain"
cp -R "$LAB_ROOT/compiler/src"        "$TARGET/toolchain/src"
cp -R "$LAB_ROOT/compiler/benches"    "$TARGET/toolchain/benches"
cp    "$LAB_ROOT/compiler/Cargo.toml" "$TARGET/toolchain/Cargo.toml"
cp    "$LAB_ROOT/compiler/Cargo.lock" "$TARGET/toolchain/Cargo.lock"
cp    "$LAB_ROOT/compiler/.gitignore" "$TARGET/toolchain/.gitignore"

# --- 2. normative spec + spec-pack -------------------------------------------
echo "-> spec/       (from docs/spec/: chapters 00-12 + 99-obligations)"
mkdir -p "$TARGET/spec"
# The reference chapters only; lab drafts/ stay lab-only.
for f in "$LAB_ROOT"/docs/spec/*.md; do
    cp "$f" "$TARGET/spec/"
done
echo "-> specpack/   (from docs/specpack/)"
cp -R "$LAB_ROOT/docs/specpack" "$TARGET/specpack"

# --- 3. stdlib seed: relocate corelib fixture to a first-class stdlib/ --------
# MANIFEST operator action 3: out of tests/fixtures/, into a real library path.
echo "-> stdlib/     (from compiler/tests/fixtures/corelib/: core, std, main.cnr)"
cp -R "$LAB_ROOT/compiler/tests/fixtures/corelib" "$TARGET/stdlib"

# --- 4. editor tools ---------------------------------------------------------
echo "-> editor/vscode/  (from tools/vscode-candor/)"
mkdir -p "$TARGET/editor"
cp -R "$LAB_ROOT/tools/vscode-candor" "$TARGET/editor/vscode"
echo "-> editor/lsp/     (from tools/candor-lsp/: Cargo.toml, Cargo.lock, src, README)"
mkdir -p "$TARGET/editor/lsp"
cp -R "$LAB_ROOT/tools/candor-lsp/src"        "$TARGET/editor/lsp/src"
cp    "$LAB_ROOT/tools/candor-lsp/Cargo.lock" "$TARGET/editor/lsp/Cargo.lock"
# The LSP crate depends on the toolchain library by relative path. In the lab it
# is ../../compiler; in the seeded layout the toolchain lives at ../../toolchain.
sed -e 's#path = "../../compiler"#path = "../../toolchain"#' \
    "$LAB_ROOT/tools/candor-lsp/Cargo.toml" > "$TARGET/editor/lsp/Cargo.toml"
sed -e 's#`../../compiler`#`../../toolchain`#g' \
    "$LAB_ROOT/tools/candor-lsp/README.md" > "$TARGET/editor/lsp/README.md"

# --- 5. dist/ contents become the repo root ----------------------------------
echo "-> (root)      README.md, INSTALL.md, LANGUAGE-TOUR.md, MANIFEST.md, VERSIONING.md, examples/"
cp "$DIST_DIR/README.md"        "$TARGET/README.md"
cp "$DIST_DIR/INSTALL.md"       "$TARGET/INSTALL.md"
cp "$DIST_DIR/LANGUAGE-TOUR.md" "$TARGET/LANGUAGE-TOUR.md"
cp "$DIST_DIR/MANIFEST.md"      "$TARGET/MANIFEST.md"
# VERSIONING.md is a lab-root policy doc (peer to GOVERNANCE.md); ship it so
# preview users know what 0.x/1.0 and package semver mean.
cp "$LAB_ROOT/VERSIONING.md"    "$TARGET/VERSIONING.md"
# Dual license (MIT OR Apache-2.0), shipped verbatim from the lab root.
cp "$LAB_ROOT/LICENSE-MIT"      "$TARGET/LICENSE-MIT"
cp "$LAB_ROOT/LICENSE-APACHE"   "$TARGET/LICENSE-APACHE"
# The preview's own CI (the user-journey check: build, examples, stdlib, HTTP).
# Maintained in the lab as dist/prod-ci.yml; installed here so the published
# repo verifies itself on push + weekly (toolchain bit-rot).
mkdir -p "$TARGET/.github/workflows"
cp "$DIST_DIR/prod-ci.yml"      "$TARGET/.github/workflows/ci.yml"
cp -R "$DIST_DIR/examples"      "$TARGET/examples"

# --- 6. strip build caches -----------------------------------------------
# The lab working tree may carry untracked .candor-cache/ dirs (incremental-
# build caches from verification runs); cp -R copies the working tree, so
# strip them from the seed — a distribution ships sources, never caches.
find "$TARGET" -type d -name '.candor-cache' -exec rm -rf {} +

echo
echo "Done. The standalone repo is assembled at:"
echo "  $TARGET"
echo
echo "next:"
echo "  cd \"$TARGET/toolchain\" && cargo build --release   # -> target/release/candor"
echo "  ./target/release/candor run ../examples/01_hello.cnr   # -> 42"
echo "  ./target/release/candor run ../stdlib                  # runs the stdlib seed tree -> 380"
