# kyne_diagnostics

## Purpose

The shared diagnostic type, rendering, and code namespace every other
compiler crate depends on.

## Responsibilities

- Define the required diagnostic shape: Error Code, Title, Explanation, Reason, Suggested Fix, Future Documentation Link, per [`COMPILER_ARCHITECTURE.md` §15](../../docs/COMPILER_ARCHITECTURE.md#15-diagnostics-engine).
- Own the `KY`/`KS` code-prefix namespace (`KY` = blocking Compiler Error; `KS` = non-blocking Security Analyzer finding).

## Dependencies

None — a foundation crate, per [`ARCHITECTURE.md` §4](../../ARCHITECTURE.md#4-compiler-crates). Depended on by every stage crate.

## Future work

Implementation begins early in Phase 1 — Milestone 2, immediately after
`kyne_ast`, per [`ROADMAP.md` §7](../../docs/ROADMAP.md#7-compiler-bootstrap-order),
since every later stage needs it to report errors.
