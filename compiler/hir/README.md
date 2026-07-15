# kyne_hir

## Purpose

High-Level Intermediate Representation construction: desugars all
remaining Kyne-level syntax sugar into a small, canonical, fully-typed
core.

## Responsibilities

- Expand the `?` operator into its explicit match-and-early-return form.
- Normalize `if`- and `match`-as-expressions into one canonical conditional-value construct.
- Serve as the representation `kyne_optimizer` operates on, per [`COMPILER_ARCHITECTURE.md` §12](../../docs/COMPILER_ARCHITECTURE.md#12-intermediate-representations).

## Dependencies

- `kyne_semantics` — consumes the validated, typed AST.

## Future work

A minimal desugaring pass (sufficient for the Counter and Token examples)
is implemented during Phase 1 — Milestone 2; full coverage arrives in
Phase 2, per [`ROADMAP.md` §7](../../docs/ROADMAP.md#7-compiler-bootstrap-order).
