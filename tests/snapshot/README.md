# tests/snapshot/

Golden-file tests comparing generated Rust text against checked-in
expected output, per
[`COMPILER_ARCHITECTURE.md` §19](../../docs/COMPILER_ARCHITECTURE.md#19-testing-strategy).
The six contracts in [`examples/canonical/`](../../examples/canonical/)
are the canonical corpus for this category.

No snapshots exist yet — requires `kyne_codegen`. Introduced starting at
the v0.3 release gate, per
[`ROADMAP.md` §25](../../docs/ROADMAP.md#25-release-gates).
