# kyne_stdlib_time

## Purpose

Ergonomic, derived helpers over `ledger.timestamp()` — `time.now()`,
`time.before()`, `time.after()`, `time.elapsed()`.

## Responsibilities

Implement the Time module, per
[`STANDARD_LIBRARY.md` §7](../../docs/STANDARD_LIBRARY.md#7-time-module).
Layer 2 (Blockchain). `time.now()` is defined to return exactly
`ledger.timestamp()`'s value — not a second, independent clock.

## Dependencies

- `kyne_stdlib_ledger` — the one documented cross-module stdlib dependency.

## Future work

Not yet scheduled to a specific Phase 1 milestone.
