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

## Future work

Implementation begins in Phase 1 — Milestone 2, per
[`ROADMAP.md` §7](../../docs/ROADMAP.md#7-compiler-bootstrap-order).
