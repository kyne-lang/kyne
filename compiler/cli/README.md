# kyne_cli

## Purpose

The `kyne` binary — the single executable through which every Kyne
capability is accessed, per [`TOOLCHAIN.md` §3](../../docs/TOOLCHAIN.md#3-cli-philosophy).

## Responsibilities

- Compose `kyne_driver` and every `tools/` crate into the CLI subcommands specified in [`TOOLCHAIN.md` §9](../../docs/TOOLCHAIN.md#9-cli-commands) (`new`, `init`, `build`, `check`, `run`, `test`, `fmt`, `lint`, `doc`, `clean`, `version`, `doctor`, `explain`, `fix`).
- Contain no compiler or tooling logic of its own — every capability is implemented in a library crate this binary only composes, per [`COMPILER_ARCHITECTURE.md` §22](../../docs/COMPILER_ARCHITECTURE.md#22-library-first-compiler-architecture).

## Dependencies

- `kyne_driver` — `check`.
- `kyne_formatter` — `fmt`.
- `kyne_diagnostics` — rendering diagnostics from both.

## Status

Implemented: `new`, `init`, `fmt`, and `check` — the four commands the
v0.1 release gate requires ("Basic CLI operational", per
[`ROADMAP.md` §25](../../docs/ROADMAP.md#25-release-gates)).
`src/scaffold.rs` owns project generation (pure filesystem work, no
compiler crate involved, per `TOOLCHAIN.md` §9's `new`/`init` entries);
`src/main.rs` is argument parsing and dispatch only. `new`/`init` support
four templates (`hello`, `token`, `nft`, `oracle`); `hello`/`token` are
the Counter/Token canonical examples verbatim, per
[`TOOLCHAIN.md` §6](../../docs/TOOLCHAIN.md#6-project-templates).

Every other subcommand in `TOOLCHAIN.md` §9 (`build`, `run`, `test`,
`lint`, `doc`, ...) depends on a compiler stage that doesn't exist yet
and is out of scope for this issue.

## Future work

Each remaining subcommand lands once its underlying stage does:
`build`/`run`/`test` need the full pipeline through `kyne_codegen` and
the external Cargo/Soroban toolchains; `lint` needs `kyne_security`;
`doc` needs `kyne_docgen`.
