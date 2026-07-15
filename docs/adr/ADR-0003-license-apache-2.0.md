# ADR-0003: License Finalized as Apache License 2.0

**Status:** Accepted
**Date:** 2026-07-15
**Deciders:** Founder (initial GitHub push)
**Type:** Implementation decision — does not amend any constitutional document's normative content.
**Supersedes:** The license choice recorded in [ADR-0001 §5](./ADR-0001-repository-structure.md), which selected MIT and explicitly flagged that choice as a "bootstrap-necessary default... revisitable by deliberate Core Maintainer decision."

## Context

The `https://github.com/kyne-lang/kyne` repository was created on GitHub
with an initial, auto-generated commit containing a single `LICENSE` file
under the Apache License 2.0 — established before this local workspace's
first push. ADR-0001 had independently chosen MIT as a bootstrap
placeholder, creating a conflict between the local repository's declared
license and the license already associated with the actual GitHub
repository.

## Decision

The project's license is **Apache License 2.0**, matching what was
already established on GitHub. The local `LICENSE` file, `Cargo.toml`'s
`workspace.package.license` field, and `README.md`'s License section are
all updated to Apache-2.0 accordingly.

`ADR-0001` itself is **not edited** — it accurately recorded the decision
made at bootstrap time, explicitly flagged as provisional. This ADR is
the record of that provision being exercised, consistent with
[`GOVERNANCE.md` §19](../GOVERNANCE.md#19-architecture-decision-records-adr)'s
treatment of ADRs as point-in-time decision records superseded by new
ADRs, never silently rewritten.

## Consequences

- Every crate in this workspace is licensed under Apache-2.0 via the
  workspace-level `license` field; no crate declares an independent
  license.
- The local repository's license now matches the GitHub repository's
  license from the moment of the very first successful push — there is
  no point in the project's public history where the two disagreed.
- Future license changes (for example, a future move to a dual
  MIT/Apache-2.0 license, common in the Rust ecosystem) remain possible
  through the same ADR process, or, if such a change would affect
  external contributors' existing rights in a way significant enough to
  warrant broader review, through a Governance-category KIP per
  [`GOVERNANCE.md` §7](../GOVERNANCE.md#7-kip-categories).
