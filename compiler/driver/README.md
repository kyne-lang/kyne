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

## Future work

Implementation begins once the individual stages it orchestrates exist,
per [`ROADMAP.md` §7](../../docs/ROADMAP.md#7-compiler-bootstrap-order).
