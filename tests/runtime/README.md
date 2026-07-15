# tests/runtime/

Tests asserting Kyne programs actually exhibit the guarantees
[`RUNTIME_MODEL.md`](../../docs/RUNTIME_MODEL.md) specifies — determinism,
atomicity, scoped rollback on a Recoverable Error versus full-transaction
rollback on a Runtime Failure, per
[`RUNTIME_MODEL.md` §9.5](../../docs/RUNTIME_MODEL.md#95-scoped-rollback-and-catching).

No tests exist yet — this category requires a working `kyne build` and a
real or mocked Soroban execution environment, neither of which exists
yet. Introduced once `kyne_codegen` and `kyne_stdlib_test` reach a usable
state, per [`ROADMAP.md` §13](../../docs/ROADMAP.md#13-testing-roadmap).
