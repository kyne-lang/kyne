# kyne_security

## Purpose

The Security Analyzer: detects known-dangerous smart-contract patterns and
reports them as non-blocking findings.

## Responsibilities

- Implement the full analysis list in [`COMPILER_ARCHITECTURE.md` §11.1](../../docs/COMPILER_ARCHITECTURE.md#111-analyses) (missing `auth()`, unauthorized state mutation, arithmetic safety, unreachable/dead code, infinite recursion, dangerous storage growth, invalid state transitions, unsafe runtime assumptions).
- Never emit a blocking Compiler Error — every finding is `KS`-prefixed Warning or Hint, per [§11.2](../../docs/COMPILER_ARCHITECTURE.md#112-severity-levels).

## Dependencies

- `kyne_types` — consumes the typed AST.
- `kyne_diagnostics` — reports findings.

## Future work

**Not implemented in Phase 1.** Deferred to Phase 2 in full, per
[`ROADMAP.md` §7](../../docs/ROADMAP.md#7-compiler-bootstrap-order), since
its findings are non-blocking and therefore not required for a correct,
deployable MVP artifact.
