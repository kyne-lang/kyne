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

## Future work

Implementation begins in Phase 1 — Milestone 2, per
[`ROADMAP.md` §7](../../docs/ROADMAP.md#7-compiler-bootstrap-order). No code
exists in this crate yet — see `src/lib.rs`.
