# kyne_resolver

## Purpose

Name Resolution: binds every identifier reference in the AST to the
declaration it refers to.

## Responsibilities

- Perform the two-pass (collection, then binding) resolution [`COMPILER_ARCHITECTURE.md` §8](../../docs/COMPILER_ARCHITECTURE.md#8-name-resolution) requires, honoring [`LANGUAGE_SPEC.md` §2.1](../../docs/LANGUAGE_SPEC.md#21-name-resolution-is-order-independent)'s order-independence guarantee.
- Resolve `use` imports against the project's module graph.
- Enforce shadowing rules (a `let` may shadow another `let`; never a `state` field or `const`).

## Dependencies

- `kyne_ast` — consumes the AST.
- `kyne_diagnostics` — reports unresolved names.

## Status

Implemented. `symbol_table.rs` defines the collection pass's output
(`SymbolTable`: module scope, one contract's member scope, imported
names); `resolve.rs` implements both passes and produces `KY06xx`
diagnostics, per
[ADR-0007](../../docs/adr/ADR-0007-resolver-implementation.md), which
also records two scope limitations discovered while implementing this
crate against a real AST:

- `kyne_ast` carries no source-span information yet, so every diagnostic
  here uses a placeholder span rather than the real offending location.
- `use` import resolution is best-effort: no crate in this workspace
  loads a multi-file project yet, so an imported name is trusted rather
  than verified against another file's real declarations.

Covered by unit tests (order-independence, prelude constructors,
shadowing rules, duplicate declarations, loop/pattern-binding scoping,
imports) plus integration tests (`tests/canonical_examples.rs`)
confirming all six canonical examples resolve with zero false-positive
diagnostics.

## Future work

`kyne_types` (issue #13) is the next stage. Both limitations above are
inherited by it without blocking its own implementation.
