# kyne_optimizer

## Purpose

Behavior-preserving HIR optimization passes: dead code elimination,
constant folding, inline expansion.

## Responsibilities

- Guarantee, absolutely, that no pass changes observable behavior, per [`COMPILER_ARCHITECTURE.md` §13](../../docs/COMPILER_ARCHITECTURE.md#13-optimization).
- Remain fully disableable, with the compiler's test suite passing identically with optimization on or off.

## Dependencies

- `kyne_hir` — consumes and produces HIR.

## Future work

**Not implemented in Phase 1.** Deferred to Phase 2 in full, per
[`ROADMAP.md` §7](../../docs/ROADMAP.md#7-compiler-bootstrap-order) and
[`COMPILER_ARCHITECTURE.md`'s Principle 3](../../docs/COMPILER_ARCHITECTURE.md#principle-3--compiler-correctness-before-optimization)
(correctness before optimization).
