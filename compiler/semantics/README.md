# kyne_semantics

## Purpose

The Semantic Analyzer: enforces every MUST-level static rule in
`LANGUAGE_SPEC.md` not already covered by parsing, resolution, or typing.

## Responsibilities

- Canonical contract member ordering (hard error), per [`LANGUAGE_SPEC.md` §3.2](../../docs/LANGUAGE_SPEC.md#32-canonical-member-order).
- Definite assignment of `state` fields, exhaustive `match`, event argument validation, `auth` placement, and every other rule in [`COMPILER_ARCHITECTURE.md` §10](../../docs/COMPILER_ARCHITECTURE.md#10-semantic-analysis).

## Dependencies

- `kyne_ast` — walks the AST directly.
- `kyne_types` — reuses `TypeEnv` for exhaustiveness and `emit`-argument checks.
- `kyne_diagnostics` — reports semantic errors.

## Status

Implemented (v0.2 gate minimum, per issue #14): canonical member order
(`src/order.rs`, `KY0401`), definite assignment of `state`
(`src/definite_assignment.rs`, `KY0104` — a real control-flow analysis,
not a syntactic check: it tracks which fields are assigned on every
non-diverging path through `init`, handling `if`/`else`, `match`, and
early `return`/`throw`/`break`/`continue`), exhaustive `match`
(`src/exhaustiveness.rs`, `KY0701`), and `emit` argument validation
(`src/events.rs`, `KY0301`).

See [ADR-0009](../../docs/adr/ADR-0009-semantics-implementation.md) for
two decisions found while testing this against the six canonical
examples: `list<T>`/`map<K, V>`-typed `state` fields are exempt from
definite assignment (a real, unanimous pattern across four of the six
examples, not stated in `LANGUAGE_SPEC.md` §4.4's literal text), and the
member-order check compares each member against the one immediately
before it, not the historical maximum, so one out-of-place member is
reported once rather than cascading into flagging everything after it.

Covered by unit tests for all four rules (positive and negative cases,
including branch-diverging definite assignment and guarded-arm
exhaustiveness) plus integration tests (`tests/canonical_examples.rs`)
confirming all six canonical examples analyze with zero false-positive
diagnostics.

## Future work

Full rule coverage (the remaining rules in
[`COMPILER_ARCHITECTURE.md` §10](../../docs/COMPILER_ARCHITECTURE.md#10-semantic-analysis) —
control-flow validation beyond exhaustiveness, full contract/visibility
validation, `auth` placement and argument-type validation, and
RUNTIME_MODEL.md's undefined-behavior checkpoint) is a Phase 2
follow-up, per [`ROADMAP.md` §7](../../docs/ROADMAP.md#7-compiler-bootstrap-order).
