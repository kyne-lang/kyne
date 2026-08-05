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

- `kyne_ast` — walks the AST directly, since `kyne_semantics` re-exports diagnostics, not the AST types this crate lowers.
- `kyne_semantics` — the validated, typed-and-analyzed stage this crate follows in the pipeline.

## Status

Implemented. `src/nodes.rs` defines the HIR node types; `src/lower.rs`
implements both required desugarings: `?` expands into an explicit
`Ok`/`Err` match with an early `HExpr::Return` in the `Err` arm, and
`if` (statement or expression position) compiles into a two-arm
`HMatch` over its condition (`true` first, then a wildcard fallback) —
so `if` and `match` share exactly one conditional construct in HIR, not
two. `return`/`throw`/`break`/`continue` are themselves `HExpr`
variants (bottom-typed, per LANGUAGE_SPEC.md §7.7), which is what makes
normalizing every match-arm body down to one shape possible.

See [ADR-0010](../../docs/adr/ADR-0010-hir-implementation.md): HIR nodes
carry no inline type annotations in this implementation, unlike
`COMPILER_ARCHITECTURE.md` §12's "fully-typed core" description — a gap
inherited from `kyne_ast` having no per-node annotation mechanism at all
(the same limitation behind the no-source-spans gap in
[ADR-0007](../../docs/adr/ADR-0007-resolver-implementation.md)).

Covered by unit tests for both desugarings (including else-if chains and
bare `return`/`throw` match-arm bodies) plus integration tests
(`tests/canonical_examples.rs`) confirming Counter and Token — the two
examples this issue scopes lowering to — succeed.

## Future work

Full desugaring coverage beyond Counter/Token (structs/enums/lists/maps
already lower structurally with no special handling needed, but are
untested against this crate directly) is a Phase 2 follow-up, per
[`ROADMAP.md` §7](../../docs/ROADMAP.md#7-compiler-bootstrap-order).
`kyne_rir` (issue #16) is the next stage.
