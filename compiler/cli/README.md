# kyne_cli

## Purpose

The `kyne` binary — the single executable through which every Kyne
capability is accessed, per [`TOOLCHAIN.md` §3](../../docs/TOOLCHAIN.md#3-cli-philosophy).

## Responsibilities

- Compose `kyne_driver` and every `tools/` crate into the CLI subcommands specified in [`TOOLCHAIN.md` §9](../../docs/TOOLCHAIN.md#9-cli-commands) (`new`, `init`, `build`, `check`, `run`, `test`, `fmt`, `lint`, `doc`, `clean`, `version`, `doctor`, `explain`, `fix`).
- Contain no compiler or tooling logic of its own — every capability is implemented in a library crate this binary only composes, per [`COMPILER_ARCHITECTURE.md` §22](../../docs/COMPILER_ARCHITECTURE.md#22-library-first-compiler-architecture).

## Dependencies

- `kyne_driver`.

## Future work

**No subcommand is implemented yet** — `src/main.rs` intentionally does
nothing beyond compiling, per this milestone's scope. Subcommand dispatch
begins in Phase 1 — Milestone 2, per [`ROADMAP.md` §4](../../docs/ROADMAP.md#4-project-phases).
