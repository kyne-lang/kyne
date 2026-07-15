#!/usr/bin/env bash
# Format this repository's own Rust implementation.
# Does not touch Kyne (.kyn) source - that is `kyne fmt`'s job, per
# docs/TOOLCHAIN.md §10, once it exists.
set -euo pipefail
cd "$(dirname "$0")/.."

cargo fmt --all
