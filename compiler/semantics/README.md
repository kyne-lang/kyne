# kyne_semantics

## Purpose

The Semantic Analyzer: enforces every MUST-level static rule in
`LANGUAGE_SPEC.md` not already covered by parsing, resolution, or typing.

## Responsibilities

- Canonical contract member ordering (hard error), per [`LANGUAGE_SPEC.md` §3.2](../../docs/LANGUAGE_SPEC.md#32-canonical-member-order).
- Definite assignment of `state` fields, exhaustive `match`, event argument validation, `auth` placement, and every other rule in [`COMPILER_ARCHITECTURE.md` §10](../../docs/COMPILER_ARCHITECTURE.md#10-semantic-analysis).

## Dependencies

- `kyne_types` — consumes the typed AST.
- `kyne_diagnostics` — reports semantic errors.

## Future work

A minimal rule set (canonical order, definite assignment) is implemented
during Phase 1 — Milestone 2; full rule coverage arrives in Phase 2, per
[`ROADMAP.md` §7](../../docs/ROADMAP.md#7-compiler-bootstrap-order).
