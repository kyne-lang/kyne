# kyne_stdlib_test

## Purpose

Conceptual primitives for mocking execution context in an off-chain test
harness: `test.mock_ledger`, `test.mock_auth`, `test.mock_state`.

## Responsibilities

Implement the Test module, per
[`STANDARD_LIBRARY.md` §14](../../docs/STANDARD_LIBRARY.md#14-test-module).
Layer 3 (Utilities). Every function here MUST be rejected by the compiler
outside a recognized test-harness compilation context — a contract that
could call `test.mock_auth` in production would have a real authorization
bypass.

## Dependencies

None.

## Future work

Required for `kyne_test_runner` and the v0.5 MVP release gate, per
[`ROADMAP.md` §25](../../docs/ROADMAP.md#25-release-gates).
