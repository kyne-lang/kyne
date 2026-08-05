# kyne_diagnostics

## Purpose

The shared diagnostic type, rendering, and code namespace every other
compiler crate depends on.

## Responsibilities

- Define the required diagnostic shape: Error Code, Title, Explanation, Reason, Suggested Fix, Future Documentation Link, per [`COMPILER_ARCHITECTURE.md` §15](../../docs/COMPILER_ARCHITECTURE.md#15-diagnostics-engine).
- Own the `KY`/`KS` code-prefix namespace (`KY` = blocking Compiler Error; `KS` = non-blocking Security Analyzer finding).

## Dependencies

None — a foundation crate, per [`ARCHITECTURE.md` §4](../../ARCHITECTURE.md#4-compiler-crates). Depended on by every stage crate.

## Status

Implemented. `Diagnostic` (`src/diagnostic.rs`) is a builder — `error`,
`warning`, and `hint` construct one, each asserting its `code` matches the
`KY`/`KS` namespace its severity requires; `.note(...)`,
`.help(...)`/`.help_with_snippet(...)`, and `.doc_link(...)` attach the
remaining required fields. `render` produces the terminal format shown
throughout `COMPILER_ARCHITECTURE.md` §15.2.

Note on landing order: this crate landed ahead of `kyne_ast` rather than
immediately after it, since `kyne_parser`'s dependency on `kyne_diagnostics`
(fixed at workspace-bootstrap time) makes it a hard prerequisite for the
parser, not merely a nice-to-have — see the parser's own PR for the actual
sequencing this ended up following.

## Future work

None — this crate's scope per `COMPILER_ARCHITECTURE.md` §15 is complete.
Later stages will call into it, but this crate itself needs no further
work to support them.
