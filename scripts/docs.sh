#!/usr/bin/env bash
# Build this repository's own Rust API documentation (rustdoc).
# Does not run `kyne doc` - that generates documentation for Kyne source
# projects via tools/docgen, per docs/TOOLCHAIN.md §12, once it exists.
set -euo pipefail
cd "$(dirname "$0")/.."

cargo doc --workspace --no-deps
