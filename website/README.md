# website/

## Purpose

The future home of Kyne's documentation portal and the hosted Playground's
frontend, per [`TOOLCHAIN.md` §17](../docs/TOOLCHAIN.md#17-playground) and
[`ARCHITECTURE.md` §3](../ARCHITECTURE.md#3-workspace-layout).

## Ownership

Not yet assigned — this directory has no content to own yet.

## Relationship to the architecture

Per [`ROADMAP.md` §4](../docs/ROADMAP.md#4-project-phases) and
[`§16`](../docs/ROADMAP.md#16-tooling-roadmap), the Playground and
documentation portal are **Phase 3** deliverables, well after the
compiler (`compiler/`) and standard library (`stdlib/`) this website will
eventually depend on. This directory is created empty during Phase 1 —
Milestone 1 to reserve its place in the workspace layout, per
[`ARCHITECTURE.md` §3](../ARCHITECTURE.md#3-workspace-layout), not because
any implementation work is expected here soon.

When work begins, this directory's frontend will consume the
`tools/playground` crate's compilation backend (per
[`ARCHITECTURE.md` §5](../ARCHITECTURE.md#5-tooling-crates)) and the
static output of `tools/docgen` (per
[`TOOLCHAIN.md` §12](../docs/TOOLCHAIN.md#12-documentation-generator)) —
it will not implement any language-processing logic of its own, per
[Library First](../ARCHITECTURE.md#2-guiding-philosophy).

## Current status

No implementation exists. No framework, static site generator, or hosting
target has been chosen — that choice is deferred to a future ADR, made
closer to Phase 3, once the `tools/playground` and `tools/docgen`
backends it will depend on actually exist to build against.
