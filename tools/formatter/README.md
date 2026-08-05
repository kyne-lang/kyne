# kyne_formatter

## Purpose

The implementation behind `kyne fmt` — Kyne's one, unconfigurable,
canonical source formatter.

## Responsibilities

- Implement [`LANGUAGE_SPEC.md` §13](../../docs/LANGUAGE_SPEC.md#13-formatting-rules) exactly: indentation, brace style, line length, trailing commas, import ordering, canonical member order.
- Remain idempotent: formatting already-canonical source produces byte-identical output.

## Dependencies

- `kyne_lexer`, `kyne_parser`, `kyne_cst` only — deliberately not `kyne_types` or any later stage, per [`ARCHITECTURE.md` §5](../../ARCHITECTURE.md#5-tooling-crates), since formatting never requires type-checking to succeed.

## Status

Implemented. `src/printer.rs` walks `kyne_cst`'s raw tree directly (not
`kyne_ast`) and re-emits every construct in canonical form, per
[ADR-0006](../../docs/adr/ADR-0006-formatter-implementation.md), which
also records two rule interpretations the canonical examples forced
(struct/enum/error bodies are always multi-line; width-driven wrapping
for struct-literal and `if`-expression values) and a fixture correction
(`examples/canonical/escrow.kyn` had a 101-column line, one over §13's
own limit).

Covered by unit tests (idempotence, always-multiline declarations, width
wrapping and its "hug the last struct-literal argument" case, blank-line
handling, import sorting, malformed-input error return) plus integration
tests (`tests/canonical_examples.rs`) confirming all six canonical
examples format as a byte-exact no-op.

## Future work

None for this crate's scope per `LANGUAGE_SPEC.md` §13. The width-aware
wrapping algorithm covers exactly the two expression shapes the
canonical examples exercise (struct literals, `if`-expressions);
extending it to more shapes (list/map literals, binary expression
chains) is a natural, additive follow-up.
