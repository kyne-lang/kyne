# ADR-0007: Resolver Diagnostic Code Range and Two Scope Limitations

**Status:** Accepted
**Date:** 2026-08-05
**Deciders:** Founder
**Type:** Implementation decision — does not amend any constitutional document's normative content.

## Context

Issue #12 required implementing `kyne_resolver`'s two-pass Name
Resolution, per
[`COMPILER_ARCHITECTURE.md` §8](../COMPILER_ARCHITECTURE.md#8-name-resolution).
Three gaps needed a documented decision, all discovered while
implementing against a real AST rather than only against the spec text.

## Decision

**Diagnostic code range.** Resolver diagnostics (unresolved name,
duplicate declaration, illegal `let`-shadowing of `state`/`const`) use
the `KY06xx` range, per the same reasoning
[ADR-0005](./ADR-0005-parser-implementation.md) applied to the parser's
`KY00xx`: `COMPILER_ARCHITECTURE.md` §15.1's table has no dedicated
"name resolution" range, and `KY06xx` ("Visibility / modules") is the
closest existing fit — `use` import resolution is explicitly a module
concern per that same range's name, and "which declaration does this
name refer to" is closely related to "is this name visible from here."
`KY0601` (unresolved name), `KY0602` (duplicate declaration), `KY0603`
(illegal shadow) are the three codes this crate produces today;
further `KY06xx` codes MAY be added without a new ADR.

**Known limitation: no source spans yet.** `kyne_ast` (issue #8) was
built with node types carrying no source-span information at all — a
scope decision made to keep that crate's first implementation focused on
structure. This crate is the first consumer that actually needs spans
for its diagnostics (per `COMPILER_ARCHITECTURE.md` §15's required
"Explanation... shown in the context of the offending source span"), and
retrofitting spans onto every `kyne_ast` node type is a larger,
cross-cutting change than fits this issue's scope. Every diagnostic this
crate produces therefore carries a placeholder span (line 1, column 1)
rather than the real offending location, with a `note` field saying so
explicitly rather than silently producing a misleading one. This is
recorded here as a real, tracked gap — not a hidden shortcut — to be
closed by a future, deliberate `kyne_ast` change (adding a `Span` to the
declaration and identifier-reference node shapes that need one) once a
consuming stage's own scope justifies that cross-cutting work.

**Known limitation: `use` import resolution is best-effort.** No crate
in this workspace yet loads a multi-file project (`kyne_driver::check`,
this compiler's only pipeline entry point so far, operates on one source
string). "Resolve `use` imports against the project's module graph," per
issue #12's acceptance criteria, therefore cannot mean anything stronger
than: record every name a `use` statement names as valid, on trust,
without verifying it against another file's real declarations. This
crate does exactly that (`SymbolTable::imported_names`) rather than
either failing to compile at all or inventing a premature multi-file
loader out of scope for this issue. Real cross-file verification is
deferred to whichever future issue introduces multi-file project
loading.

## Consequences

- Every `kyne_resolver` diagnostic's `-->` line will show `1:1`
  regardless of where the real problem is, until `kyne_ast` gains spans.
  Diagnostic *text* (title, note, help) remains accurate and useful in
  the meantime.
- A project with more than one file and real cross-file `use` imports
  will not get genuine "unknown import" errors from this crate yet —
  only single-file name resolution is verified end to end, matching
  every consumer of this compiler today (`kyne_driver::check` is
  single-file).
- `kyne_types` (issue #13) inherits both limitations as a dependent
  stage; neither blocks its own implementation, since type checking
  doesn't require resolver-level spans or multi-file imports to add
  value over a single file either.
