# kyne_codegen

## Purpose

Rust Code Generation: renders RIR as formatted, idiomatic Rust source text
forming a complete, buildable Cargo crate.

## Responsibilities

- Pretty-print RIR with no remaining semantic decisions of its own — every ownership/mapping decision was already made in `kyne_rir`, per [`COMPILER_ARCHITECTURE.md` §14](../../docs/COMPILER_ARCHITECTURE.md#14-rust-code-generation).
- Apply the standard Soroban SDK attribute macros (`#[contract]`, `#[contractimpl]`, `#[contracttype]`, `#[contracterror]`).
- Guarantee deterministic, byte-identical output for identical RIR input.

## Dependencies

- `kyne_ast` — reuses `Literal`/`Path`/`Pattern`/`BinaryOp`/`UnaryOp`/`Visibility` directly, the same node types RIR itself reuses rather than redefining.
- `kyne_rir` — consumes RIR.

## Status

Implemented (scoped to Counter and Token, per issue #17). `src/print.rs`
walks RIR and emits Rust source text, then pipes it through the system
`rustfmt` binary (`std::process::Command`, not a linked dependency —
this workspace has none outside its own members) as a mandatory final
step, per §14's "generated Rust MUST NOT ever be presented ...
unformatted." A handful of purely mechanical print-time choices this
crate does still have to make — checked-arithmetic method names, the
literal storage-API call shape, reconstructing `if`/`else` from HIR's
canonical `if`-desugared `match` form, same-contract call resolution,
import selection — are documented in
[ADR-0012](../../docs/adr/ADR-0012-codegen-implementation.md), alongside
this issue's explicit scope boundary (no `Cargo.toml` generation yet,
no multi-file module mirroring, no `#[contractevent]` type).

Covered by `tests/canonical_examples.rs`: golden-file snapshot tests
(`tests/snapshots/counter.rs.snap`, `tests/snapshots/token.rs.snap`)
confirming Counter and Token render byte-for-byte as expected, plus
tests for determinism (§14.4), Soroban attribute macro application,
compound `state` assignment never appearing as a bare Rust `+=`, and
arithmetic never appearing as a bare Rust operator (§7.2's checked-
arithmetic requirement). `examples/dump.rs` is a small manual-review
tool (`cargo run -p kyne_codegen --example dump -- counter`), not part
of the crate's public API.

## Future work

`Cargo.toml` generation and the real `cargo`/Soroban build invocation
that would let ADR-0011's and ADR-0012's Soroban SDK assumptions
finally be checked against a live build are issue #18's job. Full RIR
coverage beyond Counter/Token is a Phase 2 follow-up, per
[`ROADMAP.md` §7](../../docs/ROADMAP.md#7-compiler-bootstrap-order).
