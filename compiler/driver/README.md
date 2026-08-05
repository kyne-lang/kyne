# kyne_driver

## Purpose

Pipeline orchestration: the only crate depending on every pipeline stage,
sequencing a `.kyn` project through the full [`COMPILER_ARCHITECTURE.md` §3](../../docs/COMPILER_ARCHITECTURE.md#3-compiler-pipeline)
pipeline.

## Responsibilities

- Compose every stage crate, in the fixed execution order `COMPILER_ARCHITECTURE.md` §3 defines, into one callable pipeline.
- Serve `kyne_cli`, `kyne_test_runner`, and `kyne_playground` as their shared entry point into the compiler, per [`ARCHITECTURE.md` §4](../../ARCHITECTURE.md#4-compiler-crates).

## Dependencies

Every other `compiler/` crate: `kyne_lexer`, `kyne_parser`, `kyne_cst`,
`kyne_ast`, `kyne_resolver`, `kyne_types`, `kyne_semantics`,
`kyne_security`, `kyne_hir`, `kyne_optimizer`, `kyne_rir`, `kyne_codegen`,
`kyne_diagnostics`, `kyne_cache`.

## Status

Partially implemented. `check(source, file) -> Vec<Diagnostic>` (used by
`kyne_cli`'s `kyne check`) currently runs only as much of the pipeline as
exists: lexing and parsing, via `kyne_cst`. Its signature is meant to
stay stable as later stages land — `kyne_resolver` (issue #12),
`kyne_types` (#13), `kyne_semantics` (#14), and eventually
`kyne_security` — each addition extends what `check` runs internally
without becoming a breaking change for callers.

## Future work

`build`, `run`, and `test` orchestration (the rest of the pipeline
through `kyne_codegen`, plus the external Cargo/Soroban toolchains)
begins once those stages exist, per
[`ROADMAP.md` §7](../../docs/ROADMAP.md#7-compiler-bootstrap-order).
