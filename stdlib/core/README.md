# kyne_stdlib_core

## Purpose

Assertions, panics, and unconditional-failure markers — the language-level
facilities every contract implicitly relies on.

## Responsibilities

Implement `core.panic`, `core.assert`, `core.assert_eq`, `core.todo`,
`core.unreachable`, per [`STANDARD_LIBRARY.md` §3](../../docs/STANDARD_LIBRARY.md#3-core-module).
Layer 1 (Core) — no dependency on any Blockchain-layer module.

## Dependencies

None.

## Future work

Implementation begins once `kyne_codegen` can emit calls into standard
library crates, per [`ARCHITECTURE.md` §6](../../ARCHITECTURE.md#6-standard-library-repository).
Not required for the Phase 1 MVP's core compile pipeline, but early in the
overall standard-library implementation sequence given how foundational it is.
