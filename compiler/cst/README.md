# kyne_cst

## Purpose

The Concrete Syntax Tree: a lossless tree recording every token, including
whitespace and comments, plus the CST→AST lowering step.

## Responsibilities

- Preserve original source layout for the formatter, IDE tooling, diagnostics, and code navigation, per [`COMPILER_ARCHITECTURE.md` §6](../../docs/COMPILER_ARCHITECTURE.md#6-concrete-syntax-tree).
- Provide the one lowering function from CST to AST, discarding trivia and syntax sugar not needed for semantic analysis.

## Dependencies

- `kyne_parser` — consumes its CST output.

## Future work

Implementation begins in Phase 1 — Milestone 2, per
[`ROADMAP.md` §7](../../docs/ROADMAP.md#7-compiler-bootstrap-order). This is
also the crate `kyne_formatter` depends on directly, per
[`ARCHITECTURE.md` §5](../../ARCHITECTURE.md#5-tooling-crates), since
formatting requires only lexing, parsing, and the CST.
