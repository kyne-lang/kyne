#!/usr/bin/env bash
# Run this repository's own Rust test suite (per-crate unit tests plus
# tests/ - the repository-wide, cross-crate assets described in
# tests/README.md). Does not run Kyne's own `kyne test` - that exercises
# Kyne source projects, not this repository's Rust implementation.
set -euo pipefail
cd "$(dirname "$0")/.."

cargo test --workspace
