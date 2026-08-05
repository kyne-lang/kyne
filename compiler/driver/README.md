# kyne_driver

## Purpose

Pipeline orchestration: the only crate depending on every pipeline stage,
sequencing a `.kyn` project through the full [`COMPILER_ARCHITECTURE.md` §3](../../docs/COMPILER_ARCHITECTURE.md#3-compiler-pipeline)
pipeline.

## Responsibilities

- Compose every stage crate, in the fixed execution order `COMPILER_ARCHITECTURE.md` §3 defines, into one callable pipeline.
- Serve `kyne_cli`, `kyne_test_runner`, and `kyne_playground` as their shared entry point into the compiler, per [`ARCHITECTURE.md` §4](../../ARCHITECTURE.md#4-compiler-crates).
- Own the generated crate's `Cargo.toml` and invoke the external `cargo`/Soroban toolchains against it (the "Cargo Build" and "Soroban Build" pipeline stages), per [`RUNTIME_MODEL.md` §3](../../docs/RUNTIME_MODEL.md#3-project-lifecycle).

## Dependencies

Every other `compiler/` crate: `kyne_lexer`, `kyne_parser`, `kyne_cst`,
`kyne_ast`, `kyne_resolver`, `kyne_types`, `kyne_semantics`,
`kyne_security`, `kyne_hir`, `kyne_optimizer`, `kyne_rir`, `kyne_codegen`,
`kyne_diagnostics`, `kyne_cache`.

## Status

Two independent entry points, not yet unified:

- `check(source, file) -> Vec<Diagnostic>` runs only the CST stage
  (lexing and parsing, via `kyne_cst`) — the minimal, fast-path
  function `kyne_cli`'s `kyne check` subcommand (issue #11) actually
  calls today.
- The fuller pipeline (three modules): `pipeline.rs` — `compile(source,
  file)` runs `.kyn` source through parse, name resolution, type
  checking, semantic analysis, and `kyne_codegen`, stopping at the
  first stage to report a diagnostic, per [`RUNTIME_MODEL.md`
  §3](../../docs/RUNTIME_MODEL.md#3-project-lifecycle)'s "a project
  that fails any check here never proceeds to a later stage."
  `cargo_toml.rs` — generates the output crate's real `Cargo.toml`,
  deferred here from `kyne_codegen` per
  [ADR-0012](../../docs/adr/ADR-0012-codegen-implementation.md).
  `toolchain.rs` — `cargo_build`/`soroban_build`/`build_project` write
  the generated crate to disk and invoke the real `cargo` and
  `stellar`/`soroban` CLIs against it, with no Kyne-specific
  modification to either.

`kyne_security` and `kyne_optimizer` have no implementation yet (Phase
2, per `ROADMAP.md` §7), so neither `check` nor `pipeline::compile`
invokes them.

See [ADR-0013](../../docs/adr/ADR-0013-driver-build-integration.md) for
the full set of print-time and integration decisions, and
[`docs/guides/deployment.md`](../../docs/guides/deployment.md) for the
end-to-end workflow from `.kyn` source to a deployed contract instance.

**Verified**: a real `cargo build --target wasm32-unknown-unknown
--release` against the real `soroban-sdk` produces a valid WASM binary
for both Counter and Token (`tests/build_integration.rs`'s
`#[ignore]`d `counter_builds_to_a_real_wasm_artifact` /
`token_builds_to_a_real_wasm_artifact` — opt in with `cargo test -p
kyne_driver -- --ignored`; ignored by default since they need a
populated `~/.cargo/registry` and the `wasm32-unknown-unknown` target,
neither guaranteed in every clone of this repository). **Not verified**:
the Soroban Build stage's success path, since no `stellar`/`soroban`
CLI was available in this environment — see ADR-0013.

## Future work

`kyne_cli`'s subcommand dispatch already covers `new`/`init`/`fmt`/
`check` (issue #11); a `kyne build` subcommand reaching
`pipeline::compile`/`toolchain::build_project`, per
[`TOOLCHAIN.md`](../../docs/TOOLCHAIN.md#kyne-build---release), is
still separate, in-progress work — this crate's public API is already
what it will call underneath. Unifying `check` with
`pipeline::compile`'s earlier stages (so `kyne check` gets resolver/
type-checker/semantic-analysis diagnostics too, not just parse
errors) is natural follow-up work, not done here.
`kyne_security`/`kyne_optimizer` join `pipeline::compile`'s pipeline
once their own implementations land, per [`ROADMAP.md`
§7](../../docs/ROADMAP.md#7-compiler-bootstrap-order).
