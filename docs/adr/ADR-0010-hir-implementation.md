# ADR-0010: HIR Nodes Carry No Inline Type Annotations

**Status:** Accepted
**Date:** 2026-08-05
**Deciders:** Founder
**Type:** Implementation decision — does not amend any constitutional document's normative content.

## Context

Issue #15 required implementing `kyne_hir`'s two required desugarings
(expanding `?`; unifying `if`/`match` in expression position into one
canonical construct), scoped to what the Counter and Token canonical
examples require, per
[`COMPILER_ARCHITECTURE.md` §12](../COMPILER_ARCHITECTURE.md#12-intermediate-representations).
That section describes HIR as "fully-typed" — every node carrying its
resolved type, which the [Optimization Passes](../COMPILER_ARCHITECTURE.md#13-optimization)
stage (operating on HIR next) would presumably rely on.

## Decision

**HIR nodes carry no inline type annotations in this implementation.**
`kyne_ast` (issue #8) was built without a mechanism for attaching
per-node metadata (no `NodeId`, no side-table-keyed annotation scheme,
and no source spans either, per
[ADR-0007](./ADR-0007-resolver-implementation.md)). Retrofitting real
type annotations onto HIR nodes now would mean designing and building
that missing infrastructure as a side effect of this issue, rather than
as its own deliberate, tested piece of work — the same reasoning
[ADR-0007](./ADR-0007-resolver-implementation.md) and
[ADR-0008](./ADR-0008-types-implementation.md) already applied to the
no-spans gap. `kyne_hir`'s two desugarings do not themselves need type
information to be correct (`?`'s expansion and the `if`/`match` unifying
match-over-`true`/`false` transform are both purely structural), so
nothing about *this* issue's scope requires solving the annotation
problem to ship it.

## Consequences

- `kyne_optimizer` (a later, currently-unscoped stage) will need type
  information to implement type-aware optimizations. When that crate is
  built, it can either re-derive types on demand by walking HIR
  alongside `kyne_types`'s `TypeEnv` (the same pattern
  `kyne_semantics`'s `src/events.rs` already uses for a narrower
  purpose), or trigger the deliberate `kyne_ast`/HIR annotation
  infrastructure this ADR defers. That choice belongs to whoever
  implements `kyne_optimizer`, informed by what that crate actually
  needs — not decided speculatively here.
- `kyne_rir` (issue #16, the next stage) inherits the same gap. Nothing
  currently scoped for that issue is known to require inline types
  either, since RIR's job (SDK type mapping, ownership decisions,
  storage-call lowering) is likewise structural given the source AST's
  already-declared types (`kyne_ast::Type` on every relevant node,
  unaffected by this decision).
