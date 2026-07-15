# kyne_types

## Purpose

The Type Checker: assigns and validates a type for every typed construct
in the resolved AST.

## Responsibilities

- Type-check the closed primitive set and the five closed intrinsic parametric types (`list<T>`, `map<K,V>`, `bytes<N>`, `Option<T>`, `Result<T,E>`), per [`LANGUAGE_SPEC.md` §6.7](../../docs/LANGUAGE_SPEC.md#67-generics-a-closed-not-an-open-feature).
- Draw the inference boundary exactly at `let` (inferred) versus every other position (never inferred), per [`COMPILER_ARCHITECTURE.md` §9](../../docs/COMPILER_ARCHITECTURE.md#9-type-checker).

## Dependencies

- `kyne_resolver` — consumes the resolved AST.
- `kyne_diagnostics` — reports type errors.

## Future work

Implementation begins in Phase 1 — Milestone 2, per
[`ROADMAP.md` §7](../../docs/ROADMAP.md#7-compiler-bootstrap-order).
