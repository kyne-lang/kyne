# kyne_lsp

## Purpose

The Kyne Language Server: autocomplete, hover, rename, go-to-definition,
diagnostics, formatting, and semantic highlighting, all reused directly
from compiler libraries.

## Responsibilities

- Implement every capability in [`TOOLCHAIN.md` §16](../../docs/TOOLCHAIN.md#16-language-server) without any independent parser, resolver, or type-checker of its own.
- Communicate exclusively over standard LSP, per [`TOOLCHAIN.md` §18](../../docs/TOOLCHAIN.md#18-ide-integration).
- Launch via a `kyne lsp` subcommand, not a separate binary.

## Dependencies

- `kyne_cst`, `kyne_ast`, `kyne_resolver`, `kyne_types`, `kyne_diagnostics`.

## Future work

**Not implemented in Phase 1.** Scheduled for Phase 2, per
[`ROADMAP.md` §16](../../docs/ROADMAP.md#16-tooling-roadmap), once
`kyne_resolver`/`kyne_types`/`kyne_diagnostics` reach Phase 2 maturity.
