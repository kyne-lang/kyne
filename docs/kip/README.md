# Kyne Improvement Proposals (KIPs)

## What a KIP is

A KIP is the formal mechanism by which any change to the language
grammar, type system, standard library, compiler architecture, runtime
semantics, memory model, toolchain behavior, or governance itself is
proposed, debated, and either accepted or rejected — in public, with a
permanent, recorded rationale. Every one of Kyne's nine constitutional
documents may be amended only through an Accepted KIP; no constitutional
document is ever changed by an ordinary pull request, per
[`GOVERNANCE.md` §25](../../docs/GOVERNANCE.md#25-constitutional-documents).

## How KIPs differ from ADRs

| | KIP | ADR |
|---|---|---|
| Governs | Architectural evolution — *what* Kyne is. | Implementation decisions — *how* a specified behavior is realized. |
| Changes a constitutional document | Always. | Never. |
| Review | Core Maintainer consensus, or Founder tie-break where consensus cannot be reached. | The affected crate's own Maintainer(s). |
| Examples | A new type in `LANGUAGE_SPEC.md`; a change to `RUNTIME_MODEL.md`'s failure model; a new Security Analyzer check. | Choice of parser implementation strategy; choice of internal crate layout. |

See [`GOVERNANCE.md` §5](../../docs/GOVERNANCE.md#5-language-evolution)
through [`§8`](../../docs/GOVERNANCE.md#8-kip-acceptance-criteria) for the
complete, authoritative KIP lifecycle, categories, and acceptance
criteria, and [`§24`](../../docs/GOVERNANCE.md#24-kips-vs-adrs) for the
full KIP-versus-ADR distinction. This `README.md` restates the essentials
for a reader who lands in this directory directly; `GOVERNANCE.md` remains
the sole normative source.

## Lifecycle (summary)

```
Idea → Draft → Discussion → Accepted → Implemented → Released → Archived
```

Full definitions of every stage, its entry/exit criteria, and its
responsibilities: [`GOVERNANCE.md` §6](../../docs/GOVERNANCE.md#6-kip-lifecycle).

## Categories

Language, Compiler, Runtime, Memory, Toolchain, Standard Library,
Documentation, Governance, Ecosystem, Package Manager — see
[`GOVERNANCE.md` §7](../../docs/GOVERNANCE.md#7-kip-categories) for which
constitutional document each category amends.

## Current status

**No KIP has been filed yet.** This directory exists to hold them once
the project begins accepting proposed changes to its own constitutional
documents — expected no earlier than Phase 1 implementation work
surfacing a real need for one, per
[`WORKSPACE_BOOTSTRAP_REPORT.md`](../../WORKSPACE_BOOTSTRAP_REPORT.md).
