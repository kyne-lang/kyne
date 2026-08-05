# tests/snapshot/

Golden-file tests comparing generated Rust text against checked-in
expected output, per
[`COMPILER_ARCHITECTURE.md` §19](../../docs/COMPILER_ARCHITECTURE.md#19-testing-strategy).
The six contracts in [`examples/canonical/`](../../examples/canonical/)
are the canonical corpus for this category.

## Status

`kyne_codegen`'s own per-crate test suite
(`compiler/codegen/tests/canonical_examples.rs` +
`compiler/codegen/tests/snapshots/{counter,token}.rs.snap`) already
satisfies this category today, using the shared
[`kyne_golden`](../golden/README.md) helper (issue #20) — see
[ADR-0015](../../docs/adr/ADR-0015-golden-file-testing.md).

This repository-root directory remains reserved specifically for a
*cross-crate*, end-to-end generated-Rust golden corpus (exercising the
full `.kyn`-source-to-generated-Rust pipeline through `kyne_driver`,
rather than `kyne_codegen` in isolation) once `kyne_cli`'s `kyne build`
subcommand exists, per the v0.3 release gate
([`ROADMAP.md` §25](../../docs/ROADMAP.md#25-release-gates)) — whether
a separate fixture set here is still warranted at that point, given
`kyne_codegen`'s own coverage, is a decision for whoever picks that up.
