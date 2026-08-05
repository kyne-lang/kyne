# kyne_cst

## Purpose

The stable Concrete Syntax Tree type: a lossless tree recording every
token, including whitespace and comments, paired with the source text it
was parsed from.

## Responsibilities

- Preserve original source layout for the formatter, IDE tooling, diagnostics, and code navigation, per [`COMPILER_ARCHITECTURE.md` §6](../../docs/COMPILER_ARCHITECTURE.md#6-concrete-syntax-tree).
- Provide the `Cst` type other crates depend on, so they don't each thread a `(tree, source)` pair through their own APIs.

Note: the CST→AST lowering step itself is implemented in `kyne_ast`
(issue #8), not here — `kyne_ast` depends on this crate and owns the AST
node types lowering constructs, so the lowering function has to live in
the crate that defines its output type, not the crate that defines its
input type.

## Dependencies

- `kyne_parser` — wraps its raw syntax tree as the stable `Cst` type.
- `kyne_diagnostics` — `Cst::parse` returns the diagnostics `kyne_parser` produces.

## Status

Implemented. `Cst` (`src/lib.rs`) wraps `kyne_parser`'s `SyntaxNode` plus
the source text it was parsed from, with `text()` reconstructing the
exact original source as a direct test of losslessness. Covered by unit
tests plus integration tests (`tests/canonical_examples.rs`) confirming
all six canonical examples parse with zero diagnostics and losslessly
reconstruct their exact source text through this wrapper.

## Future work

`kyne_ast` (issue #8) is the next stage. This is also the crate
`kyne_formatter` depends on directly, per
[`ARCHITECTURE.md` §5](../../ARCHITECTURE.md#5-tooling-crates), since
formatting requires only lexing, parsing, and the CST.
