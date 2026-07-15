# kyne_playground

## Purpose

The compilation backend for the browser/server-hosted Kyne Playground.

## Responsibilities

- Compile arbitrary, untrusted visitor-submitted source through `kyne_codegen`, per [`TOOLCHAIN.md` §17](../../docs/TOOLCHAIN.md#17-playground).
- Execute only within a sandboxed environment using the mock execution context from `STANDARD_LIBRARY.md` §14 — never real network or ledger access.
- Enforce resource limits (time, memory) on every request.
- Never provide any deployment mechanism.

## Dependencies

- `kyne_driver`, `kyne_formatter`, and `kyne_cache`'s sandboxing-relevant subset.

## Future work

**Not implemented in Phase 1.** Scheduled for Phase 3, per
[`ROADMAP.md` §16](../../docs/ROADMAP.md#16-tooling-roadmap), once
`kyne_codegen` is stable and sandboxable.
