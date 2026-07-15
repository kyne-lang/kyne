# kyne_cache

## Purpose

The compiler cache: content-hash-keyed storage for each pipeline stage's
output, underpinning incremental compilation.

## Responsibilities

- Implement the `.kyn/cache/<compiler-version>/{cst,ast,hir,rir,diagnostics}/` layout, per [`COMPILER_ARCHITECTURE.md` §17](../../docs/COMPILER_ARCHITECTURE.md#17-compiler-cache).
- Guarantee the cache is strictly disposable: deleting any subset must never affect build correctness, only speed.

## Dependencies

None — a foundation crate, per [`ARCHITECTURE.md` §4](../../ARCHITECTURE.md#4-compiler-crates).

## Future work

**Not implemented in Phase 1.** Deferred to Phase 2 in full, per
[`ROADMAP.md` §8](../../docs/ROADMAP.md#8-repository-plan), since an MVP
compiler recompiling from scratch on every invocation is slower but not
incorrect.
