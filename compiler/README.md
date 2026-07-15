# compiler/

## Purpose

The Kyne compiler: every crate that turns `.kyn` source into deployable
WASM, per [`COMPILER_ARCHITECTURE.md`](../docs/COMPILER_ARCHITECTURE.md).

## Ownership

Per [`GOVERNANCE.md` §3](../docs/GOVERNANCE.md#3-governance-structure), each
crate in this directory has its own owning Maintainer, listed in that
crate's own `README.md`. During Phase 1, ownership of every crate here
defaults to the Founder, per [`ROADMAP.md` §26](../docs/ROADMAP.md#26-milestone-ownership),
diversifying as Maintainers are promoted.

## Relationship to the architecture

This directory, and every crate within it, is a direct, one-to-one
realization of [`COMPILER_ARCHITECTURE.md` §20](../docs/COMPILER_ARCHITECTURE.md#20-repository-architecture)'s
repository layout and [`ARCHITECTURE.md` §4](../ARCHITECTURE.md#4-compiler-crates)'s
crate table. Dependencies between crates in this directory are forward-only,
per [`ARCHITECTURE.md` §10](../ARCHITECTURE.md#10-dependency-rules) — see
that section for the complete, authoritative dependency graph.

This directory contains **compiler implementation only**, per
[`ARCHITECTURE.md` §23](../ARCHITECTURE.md#23-directory-rules) — no tooling
logic and no standard library implementation belongs here.

No crate in this directory has any implementation yet — see
[`WORKSPACE_BOOTSTRAP_REPORT.md`](../WORKSPACE_BOOTSTRAP_REPORT.md) for the
current state and [`ROADMAP.md` §7](../docs/ROADMAP.md#7-compiler-bootstrap-order)
for build-out order.
