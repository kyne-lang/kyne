# tools/

## Purpose

Every developer-facing tool built on top of the compiler libraries in
[`compiler/`](../compiler/): the formatter, the Language Server, the
documentation generator, the test runner, and the Playground backend, per
[`TOOLCHAIN.md`](../docs/TOOLCHAIN.md) and [`ARCHITECTURE.md` §5](../ARCHITECTURE.md#5-tooling-crates).

## Ownership

Per [`GOVERNANCE.md` §3](../docs/GOVERNANCE.md#3-governance-structure), each
crate here has its own owning Maintainer. During Phase 1, ownership
defaults to the Founder, per [`ROADMAP.md` §26](../docs/ROADMAP.md#26-milestone-ownership).

## Relationship to the architecture

**Every crate in this directory MUST depend only on `compiler/` crates and
MUST NOT independently implement any language behavior** — no tool here
may contain its own parser, type checker, or formatting logic distinct
from what `compiler/` already provides, per
[`ARCHITECTURE.md` §5](../ARCHITECTURE.md#5-tooling-crates). No crate in
this directory may depend on another crate in this directory.

Except for the Playground (a hosted service, not a CLI subcommand), every
tool here is exposed to developers exclusively through the single `kyne`
binary in `compiler/cli`, per [`TOOLCHAIN.md` §3](../docs/TOOLCHAIN.md#3-cli-philosophy)'s
"one executable" principle — none of these crates ships its own binary.

No crate in this directory has any implementation yet.
