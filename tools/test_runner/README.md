# kyne_test_runner

## Purpose

The implementation behind `kyne test` — discovers and runs a project's
test suite against a deterministic mock execution context.

## Responsibilities

- Treat every `public fn` under a project's `tests/` directory as a test case, per [`TOOLCHAIN.md` §13](../../docs/TOOLCHAIN.md#13-test-runner) (Kyne has no attribute syntax to mark tests explicitly).
- Determine pass/fail using `RUNTIME_MODEL.md`'s own Success/Recoverable-Error/Runtime-Failure outcomes — no separate assertion framework.
- Isolate every test case's mock context fully from every other's.

## Dependencies

- `kyne_driver` (full pipeline), plus the mock-context primitives from `kyne_stdlib_test` once that crate is implemented.

## Future work

**Not implemented in Phase 1.** `kyne test` is required for the v0.5 MVP
release gate, per [`ROADMAP.md` §25](../../docs/ROADMAP.md#25-release-gates).
