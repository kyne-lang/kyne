# Kyne

**Kyne is a contract-oriented programming language for writing smart
contracts on the Stellar Soroban ecosystem.** It compiles into readable,
idiomatic, Soroban-compatible Rust — Kyne is not a Rust replacement, it is
a purpose-built authoring layer on top of it.

> **Status: Phase 1 has begun.** Kyne's complete language, runtime,
> compiler, memory, standard library, toolchain, and governance
> specification is finished — see [Documentation](#documentation) below.
> **No compiler exists yet.** This repository currently contains a
> bootstrapped, empty Cargo workspace skeleton and no implemented
> functionality. Do not expect to compile a Kyne contract with anything
> in this repository today. See
> [`WORKSPACE_BOOTSTRAP_REPORT.md`](./WORKSPACE_BOOTSTRAP_REPORT.md) for
> the exact current state.

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

Per Kyne's constitutional specification (not yet implemented — see
[Current Status](#current-status)):

- **Transparent abstraction** — generated Rust remains readable and reviewable, never hidden behind a black box.
- **Security by construction** — checked-by-default arithmetic, explicit authorization (`auth(...)`), explicit persistent state, and a compiler that treats every diagnostic as a teaching opportunity.
- **Contract-oriented design** — one project, one contract, one deployment artifact.
- **Deterministic, atomic execution** — every guarantee `RUNTIME_MODEL.md` makes is designed to hold for a contract's entire, immutable, on-chain lifetime.
- **A unified toolchain** — one `kyne` executable for building, checking, formatting, testing, and documenting a project; no separate formatter, linter, or test-runner binary.

## Current status

**Phase 0 — Design & Specification: complete.** Nine constitutional
documents fully specify the language, its runtime, its compiler
architecture, its memory model, its standard library, its toolchain, and
its governance.

**Phase 1 — Compiler Bootstrap: in progress.** This repository's Cargo
workspace, crate skeletons, examples, and repository documentation have
just been bootstrapped (Phase 1 — Milestone 1: Workspace Initialization).
Every crate in [`compiler/`](./compiler/), [`stdlib/`](./stdlib/), and
[`tools/`](./tools/) is an empty skeleton — see each crate's own
`README.md` for its specified purpose and current (nonexistent)
implementation status.

For the complete, honestly-scored breakdown of what exists and what does
not, see [`KYNE_WORKSPACE_AUDIT.md`](./KYNE_WORKSPACE_AUDIT.md) and
[`WORKSPACE_BOOTSTRAP_REPORT.md`](./WORKSPACE_BOOTSTRAP_REPORT.md).

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
- [`KYNE_WORKSPACE_AUDIT.md`](./KYNE_WORKSPACE_AUDIT.md) — the pre-Phase-1 baseline engineering audit.
- [`WORKSPACE_BOOTSTRAP_REPORT.md`](./WORKSPACE_BOOTSTRAP_REPORT.md) — this bootstrap milestone's own report.

## Roadmap summary

Per [`ROADMAP.md` §4](./docs/ROADMAP.md#4-project-phases):

1. **Phase 0 — Design & Specification** ✅ complete.
2. **Phase 1 — Compiler Bootstrap** 🚧 in progress — a minimal but functioning compiler, formatter, and CLI for simple contracts.
3. **Phase 2 — Language Completion** — a feature-complete compiler: full semantic analysis, the Security Analyzer, incremental compilation, and the Language Server.
4. **Phase 3 — Developer Experience** — the Playground, package manager, documentation website, and full IDE support.
5. **Phase 4 — Ecosystem** — official libraries, community growth, education, and a stable v1.0 release.

Full detail, including release gates and no-guessing sequencing: [`ROADMAP.md`](./docs/ROADMAP.md).

## Contributing

`CONTRIBUTING.md` does not exist yet — it is the single highest-priority
remaining document, per [`ROADMAP.md` §12](./docs/ROADMAP.md#12-documentation-roadmap),
scheduled for early Phase 1. Until it exists:

1. Read [`ARCHITECTURE.md`](./ARCHITECTURE.md) first.
2. Read [`GOVERNANCE.md`](./docs/GOVERNANCE.md) to understand how a
   change becomes part of Kyne — in particular, the distinction between a
   [KIP](./docs/kip/README.md) (changes what Kyne is) and an
   [ADR](./docs/adr/README.md) (changes how it's implemented).
3. Every crate's own `README.md` states its purpose, responsibilities,
   dependencies, and current status — start there before writing code in
   it.

Security vulnerabilities should be reported privately — see
[`.github/SECURITY.md`](./.github/SECURITY.md). Do not open a public
issue for a suspected vulnerability.

## License

[Apache License 2.0](./LICENSE).
