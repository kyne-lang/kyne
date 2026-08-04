# ADR-0004: Minimum Supported Rust Version (MSRV) Policy

**Status:** Accepted
**Date:** 2026-07-15
**Deciders:** Founder
**Type:** Implementation decision — does not amend any constitutional document's normative content.
**Resolves:** The placeholder flagged in [ADR-0001 §6](./ADR-0001-repository-structure.md#6-toolchain-pin-rust-1960-msrv-target-175): *"Neither number is a deliberate MSRV policy decision — both are noted explicitly... as values the Core Maintainers should revisit deliberately."*

## Context

At bootstrap time, `rust-toolchain.toml` pinned the exact stable
toolchain verified to build the workspace (1.96.0), while `Cargo.toml`'s
`workspace.package.rust-version` and `clippy.toml`'s `msrv` were both set
to a lower, placeholder value (1.75) with no reasoning behind the
specific gap between the two. This left two different, unexplained Rust
version numbers in the repository with no documented relationship between
them, and no MSRV policy for future contributors or CI to enforce.

## Decision

**Kyne's MSRV is 1.96.0, exactly matching the pinned development
toolchain.** There is no separate, lower MSRV promise: `rust-toolchain.toml`'s
`channel`, `Cargo.toml`'s `workspace.package.rust-version`, and
`clippy.toml`'s `msrv` are all set to the same version and are kept in
agreement going forward.

This is a deliberate, minimal policy for the project's current stage, not
a permanent one:

- Kyne has not shipped a release yet, per
  [`ROADMAP.md` §10](../ROADMAP.md#10-release-roadmap) — there are no
  external consumers whose build environments this decision could break,
  and no compatibility promise has been made to break.
- Committing to an MSRV meaningfully older than the toolchain the project
  itself develops against, with no evidence yet of who would need it,
  would be exactly the kind of unjustified-ahead-of-need decision
  [`LANGUAGE_SPEC.md`'s Design Philosophy](../LANGUAGE_SPEC.md#every-keyword-must-earn-its-place)
  warns against applied to language decisions — the same discipline
  applies here, to a toolchain decision.
- A single tracked version number is simpler to keep consistent across
  three files than a policy requiring `rust-toolchain.toml` and
  `Cargo.toml`/`clippy.toml` to intentionally diverge and stay
  synchronized at an offset.

## Future revision

This policy SHOULD be revisited once Kyne approaches a stable release
(per [`ROADMAP.md`'s v1.0 gate](../ROADMAP.md#25-release-gates)) and
has real evidence of what MSRV its actual users need — for example,
adopting a rolling N-2-stable-releases policy, common among mature Rust
projects, once there is a real user base whose environments that would
serve. That future decision belongs in a new ADR superseding this one,
per the convention established in
[ADR-0002](./ADR-0002-foundation-md-location.md) and
[ADR-0003](./ADR-0003-license-apache-2.0.md) — this ADR is not
edited to reflect it.

## Consequences

- `rust-toolchain.toml`, `Cargo.toml`, and `clippy.toml` now agree
  exactly (1.96.0 / "1.96" / "1.96") with no unexplained gap between them.
- `cargo build --workspace --all-targets` continues to succeed under the
  pinned toolchain.
- A future contributor changing the toolchain version has one clear rule
  to follow: update all three files together, per the comment now present
  in each.
