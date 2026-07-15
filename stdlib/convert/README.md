# kyne_stdlib_convert

## Purpose

Explicit, closed-set conversions between primitive types and
`string`/`bytes`.

## Responsibilities

Implement one concretely-named function per type pair (`i64_to_string`,
`parse_i128`, `bytes_to_hex`, and so on), per
[`STANDARD_LIBRARY.md` §12](../../docs/STANDARD_LIBRARY.md#12-conversion-module).
Layer 1 (Core). No generic `to_string<T>()`/`parse<T>()` — Kyne has
neither generics nor overloading beyond the closed intrinsic set.

## Dependencies

None.

## Future work

Not yet scheduled to a specific Phase 1 milestone.
