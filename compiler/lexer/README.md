# kyne_lexer

## Purpose

Tokenization for Kyne source: converts `.kyn` source text into a stream of
tokens carrying kind, literal text, and exact source span.

## Responsibilities

- Recognize every token category in [`LANGUAGE_SPEC.md` §1](../../docs/LANGUAGE_SPEC.md#1-lexical-structure): keywords, identifiers, literals, comments, punctuation.
- Enforce the ASCII-only identifier rule.
- Recover from invalid input without aborting the entire lexing pass, per [`COMPILER_ARCHITECTURE.md` §4](../../docs/COMPILER_ARCHITECTURE.md#4-lexical-analysis).

## Dependencies

None. `kyne_lexer` is the first stage of the pipeline and depends on no
other crate in this workspace.

## Status

Implemented. `Lexer` (`src/lexer.rs`) hand-scans `.kyn` source into a flat
`Vec<Token>` (`src/token.rs`), with keyword and reserved-word recognition
in `src/keyword.rs`. Malformed input never aborts the pass: an invalid or
incomplete lexeme becomes a single `TokenKind::Error` token and scanning
resumes immediately after it.

Covered by unit tests for every token category (keywords, reserved words,
identifiers, integer and string literals, comments, punctuation,
whitespace, spans) including malformed-input recovery, plus integration
tests (`tests/canonical_examples.rs`) confirming all six canonical
examples under `examples/canonical/` tokenize with zero `Error` tokens.

## Future work

Downstream crates (`kyne_cst`, `kyne_parser`) will consume this crate's
token stream, per
[`COMPILER_ARCHITECTURE.md` §3](../../docs/COMPILER_ARCHITECTURE.md#3-compiler-pipeline).
