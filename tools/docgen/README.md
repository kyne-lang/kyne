# kyne_docgen

## Purpose

The implementation behind `kyne doc` — generates API documentation from
source and doc comments.

## Responsibilities

- Produce a static HTML site (and optional Markdown export), per [`TOOLCHAIN.md` §12](../../docs/TOOLCHAIN.md#12-documentation-generator).
- Cross-reference types named in signatures to their own documentation pages.
- Render fenced ` ```kyne ` code blocks using the same tokenization `kyne_lexer` performs.

## Dependencies

- `kyne_cst` (doc comments), `kyne_ast` (declaration shapes) — no semantic analysis dependency at all.

## Future work

**Not implemented in Phase 1.** Scheduled for Phase 3, per
[`ROADMAP.md` §12](../../docs/ROADMAP.md#12-documentation-roadmap), though
its generation is "automatic and continuous" once built — no separate
authoring effort.
