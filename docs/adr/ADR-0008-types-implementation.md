# ADR-0008: Type Checker Diagnostic Codes and Block-Value Ambiguity Workaround

**Status:** Accepted
**Date:** 2026-08-05
**Deciders:** Founder
**Type:** Implementation decision — does not amend any constitutional document's normative content.

## Context

Issue #13 required implementing `kyne_types`, per
[`COMPILER_ARCHITECTURE.md` §9](../COMPILER_ARCHITECTURE.md#9-type-checker).
Unlike the parser and resolver, this crate's diagnostics fit an existing
documented range (`KY02xx`, "Types / arithmetic", per
[`COMPILER_ARCHITECTURE.md` §15.1](../COMPILER_ARCHITECTURE.md#151-diagnostic-code-namespace)),
so no new range decision was needed. One real implementation gap was
found and had to be worked around rather than fixed at the source,
because fixing it at the source means reopening `kyne_ast` (issue #8),
which is out of scope here.

## Decision

**Diagnostic codes.** This crate uses `KY0200` (a malformed type, e.g.
`bytes<N>` whose `N` isn't a valid integer), `KY0201` (type mismatch),
`KY0202` (call argument count mismatch), `KY0203` (invalid arithmetic/
comparison operand type), and `KY0204` (unknown field or method) — all
within the existing `KY02xx` range, no extension needed.

**Block-value ambiguity workaround.** `kyne_ast::Statement::Expr` does
not distinguish a semicolon-less trailing expression (a block's *value*,
required when `if`/`match` is used in expression position, per
[`LANGUAGE_SPEC.md` §7.6](../LANGUAGE_SPEC.md#76-if-as-an-expression))
from an ordinary expression statement whose value is discarded — both
lower to the identical AST shape, since `kyne_ast`'s lowering (issue #8)
was not designed with this distinction in mind. This crate resolves the
ambiguity locally: whenever it needs a block's value (only when checking
an `Expr::If`/`Expr::Match` — never for a statement-position
`Statement::If`/`Statement::Match`, which never ask for a block's value
at all), it treats the block's *last* `Statement::Expr`, if present, as
that value. Because the ambiguity is only ever resolved in a context
that genuinely needs a value, an ordinary block ending in a
semicolon-terminated call statement is never misread — no such block is
ever asked for its "value" in the first place.

This is recorded here, rather than silently coded around, because a
future contributor implementing `kyne_hir` (issue #15) will hit the same
ambiguity lowering `if`/`match` expressions to HIR's canonical
conditional-value construct, and needs to know this is a pre-existing,
tracked gap in `kyne_ast` — not something to re-diagnose from scratch.

## Consequences

- A future, deliberate `kyne_ast` change adding a real
  `Block { statements: Vec<Statement>, trailing: Option<Box<Expr>> }`
  shape (or equivalent) would let both this crate and `kyne_hir` drop
  their last-statement heuristic in favor of reading `trailing` directly.
  Until then, every consumer of a block's "value" MUST apply the same
  heuristic this crate does, to stay consistent.
- This crate inherits `kyne_resolver`'s no-source-spans limitation
  (per [ADR-0007](./ADR-0007-resolver-implementation.md)) without
  re-documenting it here; every diagnostic uses a placeholder span for
  the same reason.
