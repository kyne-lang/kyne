# kyne_ast

## Purpose

The Abstract Syntax Tree: Kyne's canonical representation of a program's
language semantics, independent of formatting or comments.

## Responsibilities

- Mirror [`LANGUAGE_SPEC.md` §2](../../docs/LANGUAGE_SPEC.md#2-grammar)'s grammar productions directly, one AST node type per production.
- Provide the shared visitor abstraction every later pass (Name Resolution, Type Checker, Semantic Analyzer, Security Analyzer) traverses through.
- Remain immutable once constructed, per [`COMPILER_ARCHITECTURE.md` §7](../../docs/COMPILER_ARCHITECTURE.md#7-abstract-syntax-tree).

## Dependencies

- `kyne_cst` — consumes its CST→AST lowering output.

## Future work

Implementation begins in Phase 1 — Milestone 2, per
[`ROADMAP.md` §7](../../docs/ROADMAP.md#7-compiler-bootstrap-order).
