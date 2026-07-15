# kyne_stdlib_collections

## Purpose

Ergonomic helpers over `list<T>` and `map<K, V>` beyond their intrinsic
method tables.

## Responsibilities

Implement `contains`, `index_of`, `reverse`, `is_empty`,
`sort_ascending`/`sort_descending` (numeric element types only), per
[`STANDARD_LIBRARY.md` §10](../../docs/STANDARD_LIBRARY.md#10-collections-module).
Layer 1 (Core). Deliberately offers no `filter`/`map`/`sort_by` (Kyne has
no function-value type) and no map key/value enumeration (reserved by
`LANGUAGE_SPEC.md` §6.2.2 for a future version).

## Dependencies

None.

## Future work

Not yet scheduled to a specific Phase 1 milestone; see
[`ROADMAP.md` §11](../../docs/ROADMAP.md#11-contributor-roadmap) for
standard-library implementation as a general contribution category.
