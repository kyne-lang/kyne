# kyne_formatter

## Purpose

The implementation behind `kyne fmt` — Kyne's one, unconfigurable,
canonical source formatter.

## Responsibilities

- Implement [`LANGUAGE_SPEC.md` §13](../../docs/LANGUAGE_SPEC.md#13-formatting-rules) exactly: indentation, brace style, line length, trailing commas, import ordering, canonical member order.
- Remain idempotent: formatting already-canonical source produces byte-identical output.

## Dependencies

- `kyne_lexer`, `kyne_parser`, `kyne_cst` only — deliberately not `kyne_types` or any later stage, per [`ARCHITECTURE.md` §5](../../ARCHITECTURE.md#5-tooling-crates), since formatting never requires type-checking to succeed.

## Future work

Implementation begins in Phase 1 — Milestone 2. `kyne fmt` is expected to
be usable end-to-end before `kyne build` exists, per
[`ROADMAP.md` §7](../../docs/ROADMAP.md#7-compiler-bootstrap-order)'s
vertical-slice reasoning.
