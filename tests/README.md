# tests/

## Purpose

Repository-wide, **cross-crate** testing assets only, per
[`ARCHITECTURE.md` §8](../ARCHITECTURE.md#8-tests). Per-crate unit tests
live inside each crate under `compiler/`, `stdlib/`, and `tools/`, per
ordinary Rust convention, and are **not** duplicated here.

## Ownership

Maintained collectively across the crates each subdirectory exercises.

## Relationship to the architecture

| Directory | Purpose |
|---|---|
| [`lexer/`](./lexer/) | Cross-cutting lexer test fixtures beyond `compiler/lexer`'s own unit tests. |
| [`parser/`](./parser/) | Cross-cutting parser test fixtures. |
| [`semantic/`](./semantic/) | Cross-cutting semantic-analysis test fixtures. |
| [`runtime/`](./runtime/) | Tests exercising `RUNTIME_MODEL.md`'s execution guarantees end to end. |
| [`integration/`](./integration/) | Full pipeline: `.kyn` source through a real Cargo/Soroban build. |
| [`golden/`](./golden/) | Golden-file fixtures shared across multiple test categories. |
| [`snapshot/`](./snapshot/) | Generated-Rust snapshot comparisons. |
| [`performance/`](./performance/) | Compile-time and memory-usage benchmarks. |
| [`fuzz/`](./fuzz/) | Fuzz-testing harnesses for `kyne_lexer` and `kyne_parser`. |

This directory contains **repository-wide testing assets only**, per
[`ARCHITECTURE.md` §23](../ARCHITECTURE.md#23-directory-rules) — no
per-crate unit test belongs here.

No test exists yet in any category — see
[`WORKSPACE_BOOTSTRAP_REPORT.md`](../WORKSPACE_BOOTSTRAP_REPORT.md).
