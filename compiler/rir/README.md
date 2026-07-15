# kyne_rir

## Purpose

Rust Intermediate Representation lowering: makes every Rust-specific
decision — ownership, Soroban SDK type mapping, storage calls — that
`LANGUAGE_SPEC.md` §4.0 and `MEMORY_MODEL.md` reserve to the compiler's
discretion.

## Responsibilities

- Realize the Ownership Planner, Memory Planner, and Storage Planner conceptual subsystems of [`MEMORY_MODEL.md` §21](../../docs/MEMORY_MODEL.md#21-memory-planning-architecture), all within this one crate/stage — they are not separate pipeline stages.
- Map `address` → `soroban_sdk::Address`, `state` reads/writes → explicit storage API calls, `auth(addr)` → `require_auth()`.

## Dependencies

- `kyne_hir` — consumes (optimized) HIR.

## Future work

Implementation begins in Phase 1 — Milestone 2, per
[`ROADMAP.md` §7](../../docs/ROADMAP.md#7-compiler-bootstrap-order).
