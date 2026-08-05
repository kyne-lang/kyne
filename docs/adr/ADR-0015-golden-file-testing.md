# ADR-0015: Golden-File / Snapshot-Testing Infrastructure

**Status:** Accepted
**Date:** 2026-08-05
**Deciders:** Founder
**Type:** Implementation decision — does not amend any constitutional document's normative content.

## Context

Issue #20 required choosing and wiring up golden-file/snapshot-testing
infrastructure, per
[`COMPILER_ARCHITECTURE.md` §19](../COMPILER_ARCHITECTURE.md#19-testing-strategy):
"Golden file tests. A general methodology used across several of the
categories above: an input fixture and its expected output fixture are
both checked into the repository; the test suite diffs actual output
against the checked-in expectation on every run. Any intentional change
to output requires updating the golden file in the same commit as the
compiler change that caused it." `tests/golden/` and `tests/snapshot/`
existed only as placeholder directories with a README each.

Notably, `kyne_codegen` (issue #17) had already organically grown its
own hand-rolled golden-file mechanism (`tests/snapshots/*.rs.snap`,
compared via `include_str!` + `assert_eq!`) before this issue was
picked up, since it needed one immediately and no shared infrastructure
existed yet. This ADR's job is to choose the *shared* mechanism every
stage should use going forward, and bring that existing ad hoc instance
in line with it.

## Decision

**A small, dependency-free helper crate (`kyne_golden`, at
`tests/golden/`) was chosen over a dedicated snapshot-testing crate**
like [`insta`](https://insta.rs/) (the conventional choice for Rust
snapshot testing). This is the same reasoning already applied twice in
this codebase — [ADR-0012](./ADR-0012-codegen-implementation.md)
shelling out to `rustfmt` rather than linking a formatting crate, and
[ADR-0014](./ADR-0014-fuzz-harness-implementation.md) hand-rolling a
fuzz-mutation engine rather than depending on `cargo-fuzz` — restated
here for the same reason: every crate in this workspace has zero
dependencies outside itself, per the root `Cargo.toml`'s "no crate
outside this workspace's `members` list is part of the Kyne build."
`insta` would have been the first exception anywhere in this codebase,
for a need (compare-and-optionally-update a checked-in text fixture)
`kyne_codegen` had already shown is fully satisfiable in about sixty
lines of `std`-only Rust.

**One function, `kyne_golden::assert_golden(fixture_path, actual)`**,
used identically by every adopting crate's own test suite:

- Compares `actual` against the checked-in file at `fixture_path` (a
  path relative to the calling crate's own manifest directory — `cargo
  test` sets that as the test binary's working directory).
- If the `UPDATE_GOLDEN` environment variable is set, writes `actual`
  to `fixture_path` instead of comparing — the explicit, human-driven
  update workflow (`UPDATE_GOLDEN=1 cargo test -p <crate>`, then review
  the diff, then commit) rather than an automatic rewrite. A fixture is
  never written as a side effect of an ordinary test run.
- Panics (failing the test) with a message pointing at this workflow
  if the fixture doesn't match, or doesn't exist and `UPDATE_GOLDEN`
  wasn't set.

**Two real adopters demonstrate the shared helper covers genuinely
different golden-output shapes**, per issue #20's acceptance criterion
("demonstrated with at least one real fixture"):

1. **`kyne_codegen`** (retrofitted): `tests/canonical_examples.rs`'s
   `counter_matches_golden_file`/`token_matches_golden_file` now call
   `kyne_golden::assert_golden` instead of their original hand-rolled
   `include_str!` + `assert_eq!` — same fixtures
   (`tests/snapshots/{counter,token}.rs.snap`), same behavior, now
   sharing the common helper instead of a one-off implementation of it.
2. **`kyne_lexer`** (new): `tests/golden_tokens.rs` adds two small,
   hand-written fixtures — `tests/fixtures/basic.kyn` (ordinary
   tokenization) and `tests/fixtures/error_recovery.kyn` (an invalid
   `$` character, confirming it becomes a single `Error` token and
   lexing resumes cleanly afterward, satisfying
   [`COMPILER_ARCHITECTURE.md` §19](../COMPILER_ARCHITECTURE.md#19-testing-strategy)'s
   explicit "Include fixtures specifically exercising error recovery")
   — rendered as one line per token (kind, exact source text, 1-based
   line:column) into `tests/snapshots/*.tokens.snap`.

Fixtures are deliberately small and hand-written rather than reusing
the full six canonical examples for this finer-grained, per-token
format — a token-stream dump of an entire canonical contract would be
hundreds of lines and hard for a reviewer to actually read in a diff,
defeating the point of a golden file being reviewable.
`tests/canonical_examples.rs` (in most stage crates) already covers all
six canonical examples at whatever coarser granularity that stage
needs; golden files are for the cases specifically worth pinning down
byte-for-byte.

## Consequences

- **`CONTRIBUTING.md`** documents the update convention directly (an
  intentional golden-file change lands in the same commit as the
  change that caused it), per this issue's own acceptance criterion.
- Adopting `kyne_golden` in a new crate's test suite is exactly:
  a dev-dependency on `kyne_golden`, then one call per fixture instead
  of hand-rolling comparison logic — the pattern `kyne_codegen` and
  `kyne_lexer` both now demonstrate.
- `tests/golden/` holds the shared methodology (this crate);
  `tests/snapshot/` (per `ARCHITECTURE.md` §8) remains reserved
  specifically for `kyne_codegen`'s cross-crate, end-to-end generated-
  Rust golden corpus at the v0.3 release gate, per
  [`ROADMAP.md` §25](../ROADMAP.md#25-release-gates) — `kyne_codegen`'s
  own per-crate fixtures under `compiler/codegen/tests/snapshots/`
  satisfy this issue's demonstration requirement today; whether a
  *separate*, repository-root-level end-to-end fixture set under
  `tests/snapshot/` is still needed once `kyne_cli`'s `kyne build`
  subcommand exists is a decision for whoever picks that up, not
  answered here.
