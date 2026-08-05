# Architecture Decision Records

## What an ADR is

An ADR records an **implementation** decision — a choice made while
building the compiler, standard library, or toolchain that does not
change any constitutional document's specified behavior, but that future
contributors need the reasoning behind so they do not accidentally
reverse it or re-litigate it from scratch.

An ADR is proposed and accepted by the Maintainer(s) owning the affected
crate or subsystem — it does **not** require the project-wide Core
Maintainer consensus a KIP requires, since, by definition, an ADR never
changes what a conforming Kyne implementation is required to do.

## How ADRs differ from KIPs

| | ADR | KIP |
|---|---|---|
| Governs | Implementation decisions — *how* a specified behavior is realized. | Architectural evolution — *what* Kyne is. |
| Changes a constitutional document | Never. | Always. |
| Review | The affected crate's own Maintainer(s). | Core Maintainer consensus, per [`GOVERNANCE.md` §4](../../docs/GOVERNANCE.md#4-decision-making). |
| Examples | Choice of parser implementation strategy; choice of an incremental-compilation framework; choice of internal crate layout beyond what `COMPILER_ARCHITECTURE.md` §20 already fixes. | A new type in `LANGUAGE_SPEC.md`; a change to `RUNTIME_MODEL.md`'s failure model. |

See [`GOVERNANCE.md` §19](../../docs/GOVERNANCE.md#19-architecture-decision-records-adr)
and [`§24`](../../docs/GOVERNANCE.md#24-kips-vs-adrs) for the complete,
authoritative definition of this mechanism, and
[`ARCHITECTURE.md` §11](../../ARCHITECTURE.md#11-architecture-decision-records)
for this repository's specific storage and numbering convention. This
`README.md` restates them for a reader who lands in this directory
directly; it does not redefine either document.

## Naming and numbering

`ADR-NNNN-short-title.md`, numbered sequentially starting from `0001`.

## Index

| ADR | Title | Status |
|---|---|---|
| [ADR-0001](./ADR-0001-repository-structure.md) | Repository Structure | Accepted |
| [ADR-0002](./ADR-0002-foundation-md-location.md) | FOUNDATION.md Location Correction | Accepted |
| [ADR-0003](./ADR-0003-license-apache-2.0.md) | License Finalized as Apache License 2.0 | Accepted |
| [ADR-0004](./ADR-0004-msrv-policy.md) | Minimum Supported Rust Version (MSRV) Policy | Accepted |
| [ADR-0005](./ADR-0005-parser-implementation.md) | Parser Implementation Technique and Syntax-Error Code Range | Accepted |
| [ADR-0006](./ADR-0006-formatter-implementation.md) | Formatter Implementation Strategy and Two Rule Interpretations | Accepted |
