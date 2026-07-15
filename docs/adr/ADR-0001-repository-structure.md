# ADR-0001: Repository Structure

**Status:** Accepted
**Date:** 2026-07-15
**Deciders:** Founder (Phase 1 — Milestone 1: Workspace Initialization)
**Type:** Implementation decision — does not amend any constitutional document.

## Context

[`ARCHITECTURE.md`](../../ARCHITECTURE.md) and
[`COMPILER_ARCHITECTURE.md` §20](../../docs/COMPILER_ARCHITECTURE.md#20-repository-architecture)
fix the *behavioral* boundaries and dependency rules of the Kyne
repository — which crate depends on which, and what each is responsible
for — but neither document, correctly, prescribes every literal directory
name, crate name, or file present on disk; that is an implementation
concern reserved for an ADR, per
[`GOVERNANCE.md` §24](../../docs/GOVERNANCE.md#24-kips-vs-adrs). Bootstrapping
a real, buildable Cargo workspace (Phase 1 — Milestone 1) required
resolving several concrete questions those documents left open. This ADR
records those decisions.

## Decisions

### 1. `kyne_cli`'s directory is `compiler/cli/`

`COMPILER_ARCHITECTURE.md` §20's original repository layout enumerates
`compiler/{lexer,parser,cst,ast,resolver,types,semantics,security,hir,
optimizer,rir,codegen,diagnostics,cache,driver,tests}/` without a `cli/`
entry, since `kyne_cli` was introduced later, in `ARCHITECTURE.md` §4.
**Decision:** `kyne_cli` lives at `compiler/cli/`, alongside `driver/`,
since `ARCHITECTURE.md` §22 groups the CLI binary and `kyne_driver`
together as the pipeline's composition layer, and `kyne_cli` has no
tooling-specific behavior that would justify placing it under `tools/`
instead.

### 2. Top-level `tests/` supersedes `compiler/tests/` for cross-crate testing

`COMPILER_ARCHITECTURE.md` §20 lists a `tests/` subdirectory under
`compiler/` for "cross-stage integration and end-to-end tests."
`ARCHITECTURE.md` §8, written later with full awareness of the whole
repository, instead specifies a **top-level** `tests/{integration,
snapshot,regression,performance,fuzz}/` for exactly this purpose, with
per-crate unit tests living inside each crate under ordinary Rust
convention. **Decision:** the top-level `tests/` directory, as specified
in `ARCHITECTURE.md` §8, is what this repository implements; no
`compiler/tests/` subdirectory is created, since it would duplicate the
top-level one. This is a clarification of a design that visibly evolved
between the two documents, not a contradiction — `ARCHITECTURE.md` is the
later, more repository-complete word on this specific question, and
neither document's *normative* content is affected by this ADR's
implementation choice.

### 3. Standard library crates are named `kyne_stdlib_<module>`

`STANDARD_LIBRARY.md` and `ARCHITECTURE.md` §6 name the standard
library's *modules* (`core`, `storage`, `math`, and so on) but not their
eventual Rust crate names. **Decision:** each is named
`kyne_stdlib_<module>` (for example, `kyne_stdlib_core`,
`kyne_stdlib_math`), to avoid a future namespace collision with a
same-named compiler crate (there is no `kyne_core` compiler crate today,
but there is no guarantee one will never be proposed) and to make every
standard-library crate's origin unambiguous at a glance, including once
published independently, per `ARCHITECTURE.md` §6's note that `stdlib/`
crates are ordinary dependencies of a generated contract's own
`Cargo.toml`, ordinarily versioned but conceptually separate from the
compiler's own crates.

### 4. The standard library is a real, linked Rust crate family, not compiler intrinsics

Already decided in `ARCHITECTURE.md` §6 itself (not newly decided by this
ADR): `kyne_codegen` will recognize a standard library call and emit a
call into the corresponding `stdlib/` crate's compiled Rust, rather than
every standard library function being hand-coded as a compiler intrinsic
one at a time. This ADR reaffirms that decision as the one this bootstrap
implements, since it is the reason `stdlib/` crates have **no** dependency
on any `compiler/` crate (per `ARCHITECTURE.md` §10) — they are ordinary,
independently compilable Rust libraries.

### 5. License: MIT

No constitutional document specifies a license. **Decision:** MIT,
attributed to "The Kyne Authors" (a collective attribution, following the
convention of comparable projects such as Go's "The Go Authors," rather
than naming a specific individual). This is a placeholder-appropriate,
conventional, permissive choice for a young open-source language project
and is explicitly not a decision this ADR claims special authority over —
a future Governance-category KIP or Core Maintainer decision MAY revisit
it (for example, toward a dual MIT/Apache-2.0 license, common in the Rust
ecosystem) before the project has significant external contributions
depending on the current choice.

### 6. Toolchain pin: Rust 1.96.0, MSRV target 1.75

`rust-toolchain.toml` pins the exact stable toolchain verified to build
this workspace at bootstrap time, consistent with
[`TOOLCHAIN.md` §20](../../docs/TOOLCHAIN.md#20-reproducible-builds)'s
reproducibility expectations. The workspace's declared `rust-version`
(MSRV) is set lower, at 1.75, as a conservative placeholder. **Neither
number is a deliberate MSRV policy decision** — both are noted explicitly
in `rust-toolchain.toml` and this ADR as values the Core Maintainers
should revisit deliberately once real implementation work and real CI
infrastructure exist to inform an actual MSRV policy.

### 7. Cargo workspace dependencies are wired to match `ARCHITECTURE.md` §4/§5/§10 exactly

Every `path` dependency between crates in this workspace mirrors the
dependency graph `ARCHITECTURE.md` §10 and §21 already specify — no
edge in this repository's actual `Cargo.toml` files was invented; each
was read directly from those two sections.

## Consequences

- A future contributor reading `ARCHITECTURE.md` should find this
  repository's actual directory names and dependency edges match it
  exactly, with the seven clarifications above as the only points where a
  literal name or file had to be chosen and was not already dictated.
- Decisions 5 and 6 are flagged as revisitable by design; they are
  bootstrap-necessary defaults, not considered positions of the project.
- This ADR does not, and cannot, amend `ARCHITECTURE.md`,
  `COMPILER_ARCHITECTURE.md`, or `STANDARD_LIBRARY.md` — where a future
  KIP changes any of the specifications this ADR implements against, this
  ADR's decisions are superseded to the extent of that conflict, and a
  new ADR should record the resulting implementation change.
