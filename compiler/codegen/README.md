# kyne_codegen

## Purpose

Rust Code Generation: renders RIR as formatted, idiomatic Rust source text
forming a complete, buildable Cargo crate.

## Responsibilities

- Pretty-print RIR with no remaining semantic decisions of its own — every ownership/mapping decision was already made in `kyne_rir`, per [`COMPILER_ARCHITECTURE.md` §14](../../docs/COMPILER_ARCHITECTURE.md#14-rust-code-generation).
- Apply the standard Soroban SDK attribute macros (`#[contract]`, `#[contractimpl]`, `#[contracttype]`, `#[contracterror]`).
- Guarantee deterministic, byte-identical output for identical RIR input.

## Dependencies

- `kyne_rir` — consumes RIR.

## Future work

Implementation begins in Phase 1 — Milestone 2 and is required for the
v0.3 release gate, per [`ROADMAP.md` §10](../../docs/ROADMAP.md#10-release-roadmap)/[§25](../../docs/ROADMAP.md#25-release-gates).
