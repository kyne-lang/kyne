#!/usr/bin/env bash
# Lint this repository's own Rust implementation.
set -euo pipefail
cd "$(dirname "$0")/.."

cargo clippy --workspace --all-targets -- -D warnings
