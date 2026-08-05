# kyne_types

## Purpose

The Type Checker: assigns and validates a type for every typed construct
in the resolved AST.

## Responsibilities

- Type-check the closed primitive set and the five closed intrinsic parametric types (`list<T>`, `map<K,V>`, `bytes<N>`, `Option<T>`, `Result<T,E>`), per [`LANGUAGE_SPEC.md` §6.7](../../docs/LANGUAGE_SPEC.md#67-generics-a-closed-not-an-open-feature).
- Draw the inference boundary exactly at `let` (inferred) versus every other position (never inferred), per [`COMPILER_ARCHITECTURE.md` §9](../../docs/COMPILER_ARCHITECTURE.md#9-type-checker).

## Dependencies

- `kyne_ast` — walks the AST directly (`kyne_resolver`'s output doesn't re-export the AST types this crate needs to consume).
- `kyne_resolver` — the resolved-AST stage this crate follows in the pipeline.
- `kyne_diagnostics` — reports type errors.

## Status

Implemented. `src/ty.rs` defines `Ty` and the AST-type-to-`Ty` conversion;
`src/env.rs` builds a flat type environment (structs, enums, errors,
events, consts, state, function signatures) from one scan of the
`Program`; `src/check.rs` implements the checker itself: bottom-up type
computation with an `expected` type threaded down only for the two
genuinely contextual cases LANGUAGE_SPEC.md defines (an untyped integer
literal defaults to `i64`; a string literal coerces to `symbol` where one
is expected). See
[ADR-0008](../../docs/adr/ADR-0008-types-implementation.md) for the
diagnostic codes used and a real gap found while implementing this crate
against a real AST: `kyne_ast::Statement::Expr` can't distinguish a
block's trailing value expression from an ordinary discarded expression
statement, worked around locally rather than by reopening `kyne_ast`.

Covered by unit tests for every primitive type, all five intrinsic
parametric types and their method tables, the `let`-only inference
boundary, arithmetic/comparison type rules, assignment, call-argument
checking, `throw`/`?` against the enclosing function's `Result` error
type, struct-literal field types, and `if`-expression branch matching —
plus integration tests (`tests/canonical_examples.rs`) confirming all six
canonical examples type-check with zero false-positive diagnostics.

## Future work

`kyne_semantics` (issue #14) is the next stage. Per
[`COMPILER_ARCHITECTURE.md` §10](../../docs/COMPILER_ARCHITECTURE.md#10-semantic-analysis),
`emit`'s argument list is deliberately *not* cross-validated against its
event's declared parameters here (each argument is still type-checked
individually) — that specific count/order/type match is the Semantic
Analyzer's responsibility, not the Type Checker's.
