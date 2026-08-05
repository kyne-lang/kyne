# kyne_ast

## Purpose

The Abstract Syntax Tree: Kyne's canonical representation of a program's
language semantics, independent of formatting or comments.

## Responsibilities

- Mirror [`LANGUAGE_SPEC.md` §2](../../docs/LANGUAGE_SPEC.md#2-grammar)'s grammar productions directly, one AST node type per production.
- Provide the shared visitor abstraction every later pass (Name Resolution, Type Checker, Semantic Analyzer, Security Analyzer) traverses through.
- Remain immutable once constructed, per [`COMPILER_ARCHITECTURE.md` §7](../../docs/COMPILER_ARCHITECTURE.md#7-abstract-syntax-tree).

## Dependencies

- `kyne_cst` — this crate owns the CST→AST lowering function itself (`lower`), consuming `kyne_cst`'s tree as input.

## Status

Implemented. `src/nodes.rs` defines the AST types (one per `LANGUAGE_SPEC.md`
§2 production); `src/lower.rs` implements the total CST→AST lowering pass
(redundant parens collapsed, field-init shorthand expanded, string
escapes decoded); `src/visit.rs` provides the shared `Visitor` trait with
default per-node-kind walking.

Covered by unit tests per major construct (types, visibility, paren
collapsing, shorthand expansion, escape decoding, match-arm bodies,
if/else-if chains, every binary operator, the visitor abstraction) plus
integration tests (`tests/canonical_examples.rs`) confirming all six
canonical examples lower without panicking.

## Future work

`kyne_diagnostics` (issue #9) and `kyne_resolver` (issue #12) are the
next stages to consume this crate's output.
