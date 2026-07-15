# examples/

## Purpose

Example Kyne contracts, per [`ARCHITECTURE.md` §7](../ARCHITECTURE.md#7-examples).
This directory contains **example contracts only** — no documentation
prose (that belongs in `docs/guides/`, once it exists) and no non-Kyne
source code.

## Ownership

Maintained collectively; `canonical/` specifically carries the same
elevated-review requirement as a constitutional document, per the note
in that subdirectory's own `README.md`.

## Relationship to the architecture

| Directory | Purpose |
|---|---|
| [`canonical/`](./canonical/) | The six examples from `LANGUAGE_SPEC.md` §16, verbatim — also the compiler's own golden-file test corpus. |
| [`tutorials/`](./tutorials/) | Small, single-concept examples for learning, distinct from the canonical corpus. |
| [`regression/`](./regression/) | One file per fixed compiler defect not already covered by `canonical/` or `tutorials/`. |

Every file in this directory MUST continue to compile successfully under
the current compiler at all times, per
[`ARCHITECTURE.md` §7](../ARCHITECTURE.md#7-examples) — an example that
stops compiling is a build failure, not a documentation issue. As of this
bootstrap milestone, no compiler exists yet to enforce this; it becomes
enforceable starting in Phase 1 — Milestone 2.
