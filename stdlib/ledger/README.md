# kyne_stdlib_ledger

## Purpose

Direct, named accessors for the execution context
[`RUNTIME_MODEL.md` §5](../../docs/RUNTIME_MODEL.md#5-execution-context)
defines but does not name.

## Responsibilities

Implement `ledger.sequence()`, `ledger.timestamp()`, `ledger.network()`,
`ledger.protocol()`, per
[`STANDARD_LIBRARY.md` §6](../../docs/STANDARD_LIBRARY.md#6-ledger-module).
Layer 2 (Blockchain).

## Dependencies

None.

## Future work

Not yet scheduled to a specific Phase 1 milestone. `kyne_stdlib_time`
depends on this crate once both are implemented.
