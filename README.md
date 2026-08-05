# Kyne

**Kyne is a contract-oriented programming language for writing smart
contracts on the Stellar Soroban ecosystem.** It compiles into readable,
idiomatic, Soroban-compatible Rust — Kyne is not a Rust replacement, it is
a purpose-built authoring layer on top of it.

> **Status: Phase 1's core pipeline works end to end.** Kyne's complete
> language, runtime, compiler, memory, standard library, toolchain, and
> governance specification is finished — see
> [Documentation](#documentation) below. The compiler itself now takes a
> `.kyn` source file all the way to a real, valid WASM binary: lexer,
> parser, CST, AST, name resolution, type checker, semantic analysis,
> HIR, RIR, and Rust code generation are all implemented, and the
> generated Rust for the `Counter` and `Token` canonical examples has
> been verified to compile against the real `soroban-sdk` via a real
> `cargo build --target wasm32-unknown-unknown`. See
> [Current status](#current-status) below for exactly what is and is not
> implemented yet — the Security Analyzer, Optimizer, incremental
> compilation, and every `tools/` crate besides the formatter are still
> unimplemented stubs.

## Why Kyne exists

Soroban smart contract development today requires simultaneously learning
Rust, the Soroban SDK, smart contract architecture, authorization models,
persistent storage, and deployment workflows. For experienced Rust
developers this is manageable; for everyone else — including the
TypeScript, Go, and backend developers Soroban most needs to reach — it is
a steep, avoidable barrier. Kyne exists to remove that barrier without
reducing what a contract author can express, by compiling into the exact
same Rust and Soroban ecosystem developers already trust.

See [`docs/FOUNDATION.md`](./docs/FOUNDATION.md) for Kyne's complete
mission, vision, and values.

## Features

Per Kyne's constitutional specification — see
[Current status](#current-status) for exactly which of these are
already implemented versus still specification-only:

- **Transparent abstraction** — generated Rust remains readable and reviewable, never hidden behind a black box. Implemented: `kyne_codegen`'s output for `Counter`/`Token` is real, `rustfmt`-formatted, idiomatic Rust, checked into `compiler/codegen/tests/snapshots/` as a golden-file fixture.
- **Security by construction** — checked-by-default arithmetic, explicit authorization (`auth(...)`), explicit persistent state, and a compiler that treats every diagnostic as a teaching opportunity. Arithmetic and storage handling are implemented in codegen; the Security Analyzer itself (`kyne_security`) is not yet.
- **Contract-oriented design** — one project, one contract, one deployment artifact.
- **Deterministic, atomic execution** — every guarantee `RUNTIME_MODEL.md` makes is designed to hold for a contract's entire, immutable, on-chain lifetime.
- **A unified toolchain** — one `kyne` executable for building, checking, formatting, testing, and documenting a project; no separate formatter, linter, or test-runner binary. `kyne new`/`init`/`fmt`/`check` are wired up today; `kyne build`/`test`/`doc` are not yet dispatched from the CLI, though the library code they will call (`kyne_driver`) already works.

## Current status

**Phase 0 — Design & Specification: complete.** Nine constitutional
documents fully specify the language, its runtime, its compiler
architecture, its memory model, its standard library, its toolchain, and
its governance.

**Phase 1 — Compiler Bootstrap: core pipeline complete for `Counter` and
`Token`.** Per [`ROADMAP.md` §6](./docs/ROADMAP.md#6-phase-1-goals)'s
objective — "prove the complete pipeline works end to end for simple
contracts" — every pipeline stage through code generation is
implemented and covered by real tests, and the result has been verified
against the real, external Soroban/Rust toolchain, not merely inspected:

| Stage | Crate | Status |
|---|---|---|
| Lexing | `kyne_lexer` | Implemented, fuzz-tested |
| Parsing / CST | `kyne_parser`, `kyne_cst` | Implemented, fuzz-tested |
| CST→AST lowering | `kyne_ast` | Implemented |
| Name resolution | `kyne_resolver` | Implemented |
| Type checking | `kyne_types` | Implemented |
| Semantic analysis | `kyne_semantics` | Implemented (minimal v0.2 rule set) |
| HIR / RIR lowering | `kyne_hir`, `kyne_rir` | Implemented, scoped to `Counter`/`Token` |
| Rust code generation | `kyne_codegen` | Implemented, scoped to `Counter`/`Token` |
| Diagnostics | `kyne_diagnostics` | Implemented |
| Formatting | `tools/formatter` | Implemented (`kyne fmt`) |
| Pipeline orchestration / build | `kyne_driver` | Implemented — real `cargo build --target wasm32-unknown-unknown` against the real `soroban-sdk` verified to produce a valid WASM binary for `Counter` and `Token` |
| CLI | `kyne_cli` | `new`/`init`/`fmt`/`check` wired; `build`/`run`/`test`/`doc` not yet dispatched |

**Not yet implemented** (still doc-comment-only stubs): the Security
Analyzer (`kyne_security`), the Optimizer (`kyne_optimizer`), the
compiler cache / incremental compilation (`kyne_cache`), the Language
Server, the documentation generator, the test runner, the Playground,
and every `stdlib/` module. Deploying a compiled contract to a real
Soroban network has not been verified end-to-end either — see
[`docs/guides/deployment.md`](./docs/guides/deployment.md) for exactly
which steps have and have not been run for real.

Every implementation decision made along the way that doesn't change
Kyne's specified behavior is recorded as a numbered
[ADR](./docs/adr/README.md) (fifteen so far) — the fastest way to see
*why* something works the way it does, beyond what a crate's own
`README.md` states.

[`KYNE_WORKSPACE_AUDIT.md`](./KYNE_WORKSPACE_AUDIT.md) and
[`WORKSPACE_BOOTSTRAP_REPORT.md`](./WORKSPACE_BOOTSTRAP_REPORT.md) are
dated snapshots of this repository's pre-Phase-1 and Milestone-1 states
respectively (2026-07-15) — useful as a historical baseline, not as a
description of the repository's current state, which this section and
each crate's own `README.md` track instead.

## Repository structure

```
.
├── compiler/     # The Kyne compiler — see compiler/README.md
├── stdlib/       # The Kyne standard library — see stdlib/README.md
├── tools/        # Formatter, Language Server, docgen, test runner, Playground
├── docs/         # The nine constitutional documents, plus ADRs and KIPs
├── examples/     # Canonical, tutorial, and regression example contracts
├── tests/        # Repository-wide, cross-crate tests
├── scripts/      # Developer workflow wrappers (format, lint, test, docs)
└── website/      # Future documentation portal and Playground frontend
```

Full detail: [`ARCHITECTURE.md`](./ARCHITECTURE.md), the primary
onboarding document for contributors after this one.

## Documentation

Kyne's nine constitutional documents (all in [`docs/`](./docs/)) are the
authoritative source of truth for what Kyne is — this README does not
attempt to summarize them beyond the links below:

| Document | Defines |
|---|---|
| [`FOUNDATION.md`](./docs/FOUNDATION.md) | Mission, vision, and values. |
| [`LANGUAGE_PRINCIPLES.md`](./docs/LANGUAGE_PRINCIPLES.md) | Design philosophy and the Ten Commandments. |
| [`LANGUAGE_SPEC.md`](./docs/LANGUAGE_SPEC.md) | Complete grammar and static semantics. |
| [`RUNTIME_MODEL.md`](./docs/RUNTIME_MODEL.md) | Execution, determinism, and failure guarantees. |
| [`COMPILER_ARCHITECTURE.md`](./docs/COMPILER_ARCHITECTURE.md) | The compiler pipeline and crate architecture. |
| [`MEMORY_MODEL.md`](./docs/MEMORY_MODEL.md) | Value semantics and compiler-managed memory. |
| [`STANDARD_LIBRARY.md`](./docs/STANDARD_LIBRARY.md) | Every module and API Kyne ships with. |
| [`TOOLCHAIN.md`](./docs/TOOLCHAIN.md) | The developer experience — CLI, formatter, LSP, and more. |
| [`GOVERNANCE.md`](./docs/GOVERNANCE.md) | How Kyne evolves: the KIP process, roles, and release policy. |

Operational (non-constitutional, evolving) documents:

- [`ROADMAP.md`](./docs/ROADMAP.md) — the phased implementation plan.
- [`ARCHITECTURE.md`](./ARCHITECTURE.md) — this repository's own organization.
- [`KYNE_WORKSPACE_AUDIT.md`](./KYNE_WORKSPACE_AUDIT.md) — the pre-Phase-1 baseline engineering audit (dated).
- [`WORKSPACE_BOOTSTRAP_REPORT.md`](./WORKSPACE_BOOTSTRAP_REPORT.md) — Phase 1 — Milestone 1's own bootstrap report (dated).

## Roadmap summary

Per [`ROADMAP.md` §4](./docs/ROADMAP.md#4-project-phases):

1. **Phase 0 — Design & Specification** ✅ complete.
2. **Phase 1 — Compiler Bootstrap** 🚧 in progress — the core pipeline (lexer through code generation) and a real, verified Cargo/Soroban build work end to end for the `Counter` and `Token` canonical examples; the Security Analyzer, incremental compilation, and full CLI subcommand coverage remain.
3. **Phase 2 — Language Completion** — a feature-complete compiler: full semantic analysis, the Security Analyzer, incremental compilation, and the Language Server.
4. **Phase 3 — Developer Experience** — the Playground, package manager, documentation website, and full IDE support.
5. **Phase 4 — Ecosystem** — official libraries, community growth, education, and a stable v1.0 release.

Full detail, including release gates and no-guessing sequencing: [`ROADMAP.md`](./docs/ROADMAP.md).

## Contributing

See [`CONTRIBUTING.md`](./CONTRIBUTING.md) for how to get set up, where to
work, the KIP-versus-ADR distinction, and what a pull request is expected
to include.

Security vulnerabilities should be reported privately — see
[`.github/SECURITY.md`](./.github/SECURITY.md). Do not open a public
issue for a suspected vulnerability.

## License

[Apache License 2.0](./LICENSE).
