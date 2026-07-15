# stdlib/

## Purpose

The Kyne standard library's implementation: one crate per module defined
in [`STANDARD_LIBRARY.md` §2](../docs/STANDARD_LIBRARY.md#2-library-organization),
organized into the same three layers that document fixes — Core,
Blockchain, and Utilities.

```
Layer 1 — Core:        core/  collections/  math/  convert/
Layer 2 — Blockchain:  storage/  auth/  ledger/  time/  events/  crypto/
Layer 3 — Utilities:   debug/  test/
```

## Ownership

Per [`GOVERNANCE.md` §3](../docs/GOVERNANCE.md#3-governance-structure), each
module crate has its own owning Maintainer. During Phase 1, ownership
defaults to the Founder, per [`ROADMAP.md` §26](../docs/ROADMAP.md#26-milestone-ownership).

## Relationship to the architecture

This is Kyne-facing runtime code that a generated contract links against —
**not** compiler logic, per [`ARCHITECTURE.md` §6](../ARCHITECTURE.md#6-standard-library-repository).
**No crate in this directory may depend on any `compiler/` crate**, per
[`ARCHITECTURE.md` §10](../ARCHITECTURE.md#10-dependency-rules). Dependency
flow between modules follows [`STANDARD_LIBRARY.md`'s own layering rule](../docs/STANDARD_LIBRARY.md#overall-library-architecture)
strictly: Core → Blockchain → Utilities, never in reverse. `stdlib/time`
depending on `stdlib/ledger` is the one cross-module dependency
`STANDARD_LIBRARY.md` documents explicitly; no other crate in this
directory depends on a sibling.

Every function this crate family will eventually expose is already fully
specified, behaviorally, in `STANDARD_LIBRARY.md` — no crate here may
implement a capability that document does not already describe.

No crate in this directory has any implementation yet.
