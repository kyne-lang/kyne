#!/usr/bin/env bash
# Verify the local environment can build the Kyne workspace.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> Toolchain"
rustup show

echo "==> Building workspace"
cargo build --workspace --all-targets
