# kyne_parser

## Purpose

Enforces the grammar defined in [`LANGUAGE_SPEC.md` §2](../../docs/LANGUAGE_SPEC.md#2-grammar)
against the token stream `kyne_lexer` produces, and builds a Concrete
Syntax Tree.

## Responsibilities

- Accept exactly the set of token sequences the EBNF grammar defines; reject every other sequence.
- Recover from a syntax error by synchronizing at the next safe token, per [`COMPILER_ARCHITECTURE.md` §5](../../docs/COMPILER_ARCHITECTURE.md#5-parsing).
- Produce a lossless CST (via `kyne_cst`) — never an AST directly.

## Dependencies

- `kyne_lexer` — consumes its token stream.
- `kyne_diagnostics` — reports syntax errors.

## Future work

Implementation begins in Phase 1 — Milestone 2, per
[`ROADMAP.md` §7](../../docs/ROADMAP.md#7-compiler-bootstrap-order).
