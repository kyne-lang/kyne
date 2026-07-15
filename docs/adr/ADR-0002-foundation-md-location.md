# ADR-0002: FOUNDATION.md Location Correction

**Status:** Accepted
**Date:** 2026-07-15
**Deciders:** Founder (repository correction, post Phase 1 — Milestone 1)
**Type:** Implementation decision — does not amend any constitutional document's normative content.

## Context

`Foundation.md` (Milestone 1's output) was originally created at
`kyne/Foundation.md` — the only constitutional document not located under
`docs/`, since it predates the `docs/` convention established starting
with Milestone 2. [`KYNE_WORKSPACE_AUDIT.md` §7](../../KYNE_WORKSPACE_AUDIT.md#7-workspace-consistency)
flagged this as a structural inconsistency, and
[`ARCHITECTURE.md` §9](../../ARCHITECTURE.md#9-documentation)'s own
documentation table already specified the intended, correct location as
`docs/FOUNDATION.md`, noting (at the time) that no such file actually
existed there.

The nested `kyne/` directory this produced was also easy to misread as an
intentional second project root, particularly against
[`ARCHITECTURE.md` §3](../../ARCHITECTURE.md#3-workspace-layout)'s own
workspace-layout diagram, which used a bare `kyne/` line to label the
repository root itself (a common tree-diagram convention) — a label that,
juxtaposed with a real, pre-existing `kyne/` subdirectory, was a
foreseeable source of confusion about which one was authoritative.

## Decision

`FOUNDATION.md` is moved to `docs/FOUNDATION.md`, matching every other
constitutional document's location and `ARCHITECTURE.md` §9's own
already-specified target. The `kyne/` directory is removed entirely — it
now holds nothing, and no part of the intended architecture calls for a
directory by that name anywhere in the tree.

Every Markdown cross-reference to the file, across every constitutional
and operational document, is updated to the new path:

- From within `docs/*.md`: `../kyne/Foundation.md` → `./FOUNDATION.md`.
- From the repository root (`ARCHITECTURE.md`, `README.md`):
  `kyne/Foundation.md` → `docs/FOUNDATION.md`.
- From `examples/tutorials/README.md`: `../../kyne/Foundation.md` →
  `../../docs/FOUNDATION.md`.

`ARCHITECTURE.md`'s workspace-layout tree diagram ([§3](../../ARCHITECTURE.md#3-workspace-layout))
and the equivalent diagram in the root `README.md` are both revised to
use an explicit `.` root marker instead of a bare `kyne/` label, removing
the ambiguity noted above.

## Files deliberately left unchanged

[`KYNE_WORKSPACE_AUDIT.md`](../../KYNE_WORKSPACE_AUDIT.md) and
[`WORKSPACE_BOOTSTRAP_REPORT.md`](../../WORKSPACE_BOOTSTRAP_REPORT.md)
are dated, point-in-time snapshot reports describing repository state as
it verifiably existed on their respective audit dates, prior to this
correction. Their references to `kyne/Foundation.md` and the `kyne/`
directory are accurate historical record, not live cross-references —
editing them to describe the corrected state would misrepresent what was
actually found at the time each report was produced. Both remain
unmodified; this ADR, and the accompanying
[`REPOSITORY_STRUCTURE_FIX_REPORT.md`](../../REPOSITORY_STRUCTURE_FIX_REPORT.md),
are the record of what changed afterward.

## Consequences

- There is exactly one `FOUNDATION.md` in the repository, at
  `docs/FOUNDATION.md`, matching `ARCHITECTURE.md`'s already-specified
  target layout exactly.
- No file outside the two historical reports named above references the
  old `kyne/Foundation.md` path.
- A repository-wide relative-link check (every `](path)` target in every
  `.md` file resolved against its containing file's directory) found zero
  broken links after this correction, per
  [`REPOSITORY_STRUCTURE_FIX_REPORT.md`](../../REPOSITORY_STRUCTURE_FIX_REPORT.md).
- This ADR does not amend any constitutional document's specified
  behavior — it corrects an implementation-level filesystem location
  question `ARCHITECTURE.md` had already answered but that had not yet
  been realized on disk.
