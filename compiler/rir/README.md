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

- `kyne_ast` — the shared node types (`Pattern`, `Literal`, `Type`, ...) RIR reuses directly rather than redefining.
- `kyne_hir` — consumes (optimized) HIR.

## Status

Implemented (scoped to Counter and Token, per issue #16). `src/nodes.rs`
defines the RIR node types - already Rust-shaped, not Kyne-shaped: a
`state` read/write is already `RExpr::StorageGet`/`RStmt::StorageSet`,
`auth(addr)` is already `RStmt::RequireAuth`, and every type is already
its Soroban SDK equivalent. `src/lower.rs` implements the lowering,
including expanding a compound `state` assignment (`count += 1`) into
an explicit read-modify-write, since `state` has no Rust place a native
`+=` could apply to.

See [ADR-0011](../../docs/adr/ADR-0011-rir-implementation.md) for the
full Kyne-to-Soroban type and storage-API mapping table this crate
targets, and an honest note that it is a best-effort target — not yet
verified against a real `cargo build` with a `soroban-sdk` dependency,
which no crate before issue #18 is able to attempt.

Covered by unit tests (implicit in the canonical-example assertions
below) and integration tests (`tests/canonical_examples.rs`) confirming
Counter and Token lower successfully, including structural checks for
the `address`→`Address` mapping, `map<K, V>`→`RType::Map` mapping,
`auth`→`RequireAuth`, compound `state`-assignment expansion, and the
implicit empty-map default `balances` (inherited from ADR-0009) carries
into its `StorageGet` occurrences.

## Future work

`kyne_codegen` (issue #17) is the next stage - it owns prepending the
synthetic `env: Env` parameter to every function signature and choosing
the literal Rust syntax that realizes `StorageGet`/`StorageSet`/
`RequireAuth`, per ADR-0011. Full RIR coverage beyond Counter/Token is a
Phase 2 follow-up, per
[`ROADMAP.md` §7](../../docs/ROADMAP.md#7-compiler-bootstrap-order).
