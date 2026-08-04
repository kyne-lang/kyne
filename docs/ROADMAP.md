# ROADMAP.md

# The Kyne Roadmap & Implementation Specification

**Phase:** 0 — Foundations (this document; subsequent phases defined within)
**Milestone:** 10 — Roadmap & Implementation Specification
**Version:** 0.1 (Draft)
**Status:** Informative — **not constitutional.** Per [GOVERNANCE.md §17](./GOVERNANCE.md#17-documentation-governance), this document is an informative document, not a normative one: it describes the intended execution plan for building what the nine constitutional documents already specify, and it is expected to evolve, through ordinary review rather than [GOVERNANCE.md §25](./GOVERNANCE.md#25-constitutional-documents)'s elevated constitutional process, as implementation experience surfaces new information. The **destination** — [FOUNDATION.md](./FOUNDATION.md) through [GOVERNANCE.md](./GOVERNANCE.md) — is fixed. The **route** described here is not, and revising this document's sequencing, priorities, or release boundaries MUST NOT be read as reopening any constitutional decision.

This document implements — and MUST NOT contradict — [FOUNDATION.md](./FOUNDATION.md), [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md), [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md), [RUNTIME_MODEL.md](./RUNTIME_MODEL.md), [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md), [MEMORY_MODEL.md](./MEMORY_MODEL.md), [STANDARD_LIBRARY.md](./STANDARD_LIBRARY.md), [TOOLCHAIN.md](./TOOLCHAIN.md), and [GOVERNANCE.md](./GOVERNANCE.md). Where a sequencing choice in this document appears to conflict with any of the nine, this document is wrong and MUST be corrected — not the reverse.

This specification uses **RFC 2119** keywords normatively: **MUST**, **MUST NOT**, **SHALL**, **SHOULD**, **SHOULD NOT**, and **MAY** carry their standard meanings. Consistent with the brief governing this milestone, this document contains **no calendar dates, no team-size estimates, and no marketing timelines** — every sequencing statement is expressed as an ordering and a dependency, never a date.

---

# 1. Vision

**Current state.** Kyne is, as of this document, a fully specified language with no implementation. Nine constitutional documents define, exhaustively, what Kyne is: its philosophy, its grammar and static semantics, its runtime behavior, its compiler architecture, its memory model, its standard library, its developer tooling, and its governance. No compiler exists yet.

**Future vision.** Kyne becomes the reference language for writing Soroban smart contracts — not by displacing Rust, but by giving developers who are not already Rust experts a direct, secure, auditable path into Soroban development, per [FOUNDATION.md](./FOUNDATION.md#vision)'s own stated vision: "the simplest, safest, and most enjoyable way to build smart contracts on Stellar."

**Five-year direction.** Within its first several years, Kyne SHOULD move through four phases, defined fully in [§4](#4-project-phases): a working compiler bootstrap, a feature-complete language implementation, a production-ready developer experience, and a healthy, self-sustaining ecosystem. This document does not attach a timeline to that progression — per [Roadmap Principle 1](#2-roadmap-philosophy), the constraint this roadmap plans against is a small, possibly single-founder team with limited funding, not a fixed calendar.

**Long-term goals.** Kyne aims to be judged, over its lifetime, against the measures [§20](#20-success-metrics) defines: a compiler trustworthy enough that its own correctness is rarely in question, a contributor base large enough that no single person is a point of failure, and a body of deployed contracts large enough that Kyne's promises — determinism, atomicity, explicit authorization — have been tested by real, adversarial conditions, not only by design review.

**What Kyne aims to become.** Not a general-purpose language, per [Non-Goals of LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md#non-goals) — a durable, boring-in-the-best-sense piece of infrastructure that a Soroban contract author reaches for by default, the way a systems programmer today reaches for an established, trusted toolchain without a second thought.

---

# 2. Roadmap Philosophy

**Why phased development exists.** A specification this complete cannot be implemented as a single undertaking without enormous risk: a multi-year effort with no working software to show partway through is a project that is difficult to sustain, difficult to attract contributors to, and difficult to course-correct if an early architectural assumption turns out to be wrong. Phased development, per [Roadmap Principle 2 (Incremental Value)](#roadmap-philosophy-principles) and [Principle 3 (Vertical Slices)](#roadmap-philosophy-principles), exists to keep working software in front of real users at every stage, so that risk is discovered early, when it is cheap to correct, rather than late.

**Why implementation follows specification.** Restated directly from [GOVERNANCE.md §23](./GOVERNANCE.md#23-specification-first-development): every phase in this roadmap builds toward behavior the nine constitutional documents already define. This roadmap sequences *when* a piece of already-specified behavior gets built; it never decides *what* that behavior is.

**How priorities are chosen.** A roadmap item is prioritized higher when it unblocks the largest number of other items (per [Compiler Bootstrap Order](#7-compiler-bootstrap-order)'s dependency-driven sequencing), when it produces the earliest possible complete vertical slice (per Principle 3), and when it protects work already done from needing to be redone (per [Principle 4 (Stable Foundations)](#roadmap-philosophy-principles)). Where these three considerations conflict, dependency correctness — an item that is a genuine prerequisite for another — always wins, since no amount of prioritization can make an item buildable before its dependency exists.

**Success philosophy.** Success, at every phase boundary in this document, is defined objectively per [§25](#25-release-gates) and [§27](#27-definition-of-done) — never as a subjective sense that "enough" has been built. A phase is complete when its release gate's checklist is satisfied, not when it feels complete.

## Roadmap Philosophy Principles

**1. Reality Over Optimism.** This roadmap assumes one founder, a small number of contributors, and limited funding, and it MUST remain achievable under those constraints — a roadmap that is only achievable with a large, well-funded team is not a roadmap for the project Kyne actually is at this stage. Every phase in [§4](#4-project-phases) is scoped so that a single dedicated contributor could, in principle, make meaningful progress on it alone, even though most phases will benefit from more.

**2. Incremental Value.** Every phase MUST produce something a real user can actually use, even if narrow — [§9](#9-mvp-definition)'s minimum viable compiler exists specifically so that Phase 1 ends with a working, if limited, tool rather than a pile of unintegrated components. A long stretch with no usable output is a failure of phasing, not an acceptable cost of thoroughness.

**3. Vertical Slices.** This roadmap consistently prefers building one complete, end-to-end path — lexing through a deployed WASM artifact for a single simple contract — before broadening that path's feature coverage, over building every subsystem to full completeness in isolation before any of them are connected. [§7](#7-compiler-bootstrap-order) is organized around this principle directly.

**4. Stable Foundations.** [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md)'s pipeline and crate boundaries are not re-litigated by this roadmap — every phase builds within them, never around them, and a roadmap item that would require redesigning core compiler architecture already decided by that constitutional document is out of this roadmap's scope entirely, since this document implements constitutional decisions rather than revisiting them.

**5. Contributors First.** Every phase in [§4](#4-project-phases) MUST expose work suitable for a Good First Issue, an intermediate contribution, and advanced compiler engineering, per [§11](#11-contributor-roadmap) — a phase with work only a project's most experienced engineer can meaningfully touch is a phase that fails to grow the contributor base [§19](#19-risks)'s knowledge-concentration risk depends on growing.

**6. Ship Often.** Every phase, and every meaningful milestone within a phase, SHOULD end with a real, versioned release, per [§10](#10-release-roadmap) — frequent, working releases are how a small team demonstrates progress, attracts contributors, and catches integration defects before they compound across multiple unshipped changes.

---

# 3. Current State

Phase 0 has produced nine constitutional documents, each locked and normative:

| Document | Establishes |
|---|---|
| [FOUNDATION.md](./FOUNDATION.md) | Kyne's mission, values, and the problem it exists to solve. |
| [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md) | Kyne's design philosophy, its Ten Commandments, and the KIP governance concept. |
| [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) | Complete lexical structure, grammar, and static semantics, including six canonical example contracts. |
| [RUNTIME_MODEL.md](./RUNTIME_MODEL.md) | Deterministic, transactional, transparent execution semantics — the state model, execution pipeline, failure model, and every runtime guarantee. |
| [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md) | The complete compiler pipeline, its library-first crate structure, its diagnostic engine, and its testing strategy. |
| [MEMORY_MODEL.md](./MEMORY_MODEL.md) | Kyne's value-semantics memory model and the compiler's ownership, allocation, and storage-mapping responsibilities. |
| [STANDARD_LIBRARY.md](./STANDARD_LIBRARY.md) | Every module and API Kyne ships with, specified behaviorally. |
| [TOOLCHAIN.md](./TOOLCHAIN.md) | The complete developer experience — CLI, formatter, language server, documentation generator, test runner. |
| [GOVERNANCE.md](./GOVERNANCE.md) | How Kyne evolves: the KIP process, roles, release policy, and constitutional protection for all nine documents. |

**What Phase 0 achieved.** A complete, internally consistent, implementation-independent specification of a smart contract language — every construct a Kyne program can contain, every guarantee its execution provides, and every process by which it may change, all fixed before a single line of compiler code exists. Phase 0's output is this document's entire input: every phase that follows exists to realize what is already written, not to discover what Kyne should be.

---

# 4. Project Phases

| Phase | Name | Status | Deliverable |
|---|---|---|---|
| 0 | Design & Specification | Complete | Nine constitutional documents, per [§3](#3-current-state). |
| 1 | Compiler Bootstrap | Not started | A minimal but functioning compiler. |
| 2 | Language Completion | Not started | A feature-complete compiler. |
| 3 | Developer Experience | Not started | A production-ready developer experience. |
| 4 | Ecosystem | Not started | A healthy long-term ecosystem. |

## Phase 0 — Design & Specification

Already completed, per [§3](#3-current-state) and [§5](#5-phase-0-retrospective).

## Phase 1 — Compiler Bootstrap

**Objective.** Prove the complete pipeline works end to end for simple contracts. Primary goals: CLI, Lexer, Parser, CST, AST, Diagnostics, Formatter, basic Rust generation, Cargo integration, basic WASM generation, and the ability to compile simple contracts successfully. Full detail in [§6](#6-phase-1-goals) and [§7](#7-compiler-bootstrap-order).

## Phase 2 — Language Completion

**Objective.** Bring the compiler to full conformance with every constitutional document. Primary goals: Semantic Analyzer, Type Checker, HIR, RIR, the Ownership/Memory/Storage Planners of [MEMORY_MODEL.md §21](./MEMORY_MODEL.md#21-memory-planning-architecture), incremental compilation, the Language Server, expanded diagnostics, and comprehensive testing across every category [COMPILER_ARCHITECTURE.md §19](./COMPILER_ARCHITECTURE.md#19-testing-strategy) defines.

## Phase 3 — Developer Experience

**Objective.** Make Kyne pleasant and safe to adopt for real projects. Primary goals: Playground, Package Manager, Registry, documentation website, examples, templates, debugger, profiler, coverage, and improved IDE support across every editor [TOOLCHAIN.md §18](./TOOLCHAIN.md#18-ide-integration) names.

## Phase 4 — Ecosystem

**Objective.** Grow a self-sustaining community around a stable language. Primary goals: official libraries, community ecosystem, education, certification, enterprise adoption, governance evolution per [GOVERNANCE.md §21](./GOVERNANCE.md#21-future-governance), and a stable v1.0 release maintained per [GOVERNANCE.md §9](./GOVERNANCE.md#9-release-policy).

---

# 5. Phase 0 Retrospective

**What Phase 0 accomplished.** Every question a compiler engineer, a contract author, or a future governance participant could ask about what Kyne *is* now has a written, normative answer. This includes questions that are easy to defer accidentally — what happens when a cross-contract call fails partway through, whether a `map<K, V>` used as `state` is stored as one blob or many entries, whether the standard library may ever expose `MD5` — all of which were resolved explicitly during Phase 0 rather than left for an implementer to decide unilaterally later.

**Why specification-first development was chosen.** Per [GOVERNANCE.md §23](./GOVERNANCE.md#23-specification-first-development), a specification written after an implementation tends to describe what the implementation happens to do, not what it should do — and a language whose actual contract with its users is "whatever the current compiler build does" cannot make the durability promises [RUNTIME_MODEL.md §4](./RUNTIME_MODEL.md#4-contract-lifecycle) requires for immutable, value-holding deployed contracts. Writing nine complete constitutional documents before Phase 1 begins is the concrete expression of that choice.

**Summary of the constitutional foundation.** Phase 0's nine documents divide cleanly along the boundary [GOVERNANCE.md §24](./GOVERNANCE.md#24-kips-vs-adrs) draws between specification and implementation: everything Phase 0 produced is specification — behavior, guarantees, and process. Nothing in Phase 0 constitutes or constrains a specific Rust data structure, parser-generator choice, or caching algorithm; those remain Phase 1 and Phase 2 engineering decisions, recorded as ADRs per [GOVERNANCE.md §19](./GOVERNANCE.md#19-architecture-decision-records-adr), not as amendments to any constitutional document.

---

# 6. Phase 1 Goals

**Success criteria.** Phase 1 is complete when the [MVP Definition](#9-mvp-definition) is satisfied in full and the [v0.5 release gate](#25-release-gates) is met.

**Dependencies.** Phase 1 depends on nothing outside Phase 0's completed constitutional documents — it is the first phase with no upstream dependency other than the specification itself.

**Expected outcomes.** A working `kyne` binary capable of taking the Counter and Token contracts from [LANGUAGE_SPEC.md §16](./LANGUAGE_SPEC.md#16-examples) from `.kyn` source through to a real, deployable WASM artifact; a functioning `kyne fmt`; and a diagnostic engine producing at least the illustrative examples [LANGUAGE_SPEC.md §14](./LANGUAGE_SPEC.md#14-error-philosophy) and [COMPILER_ARCHITECTURE.md §15](./COMPILER_ARCHITECTURE.md#15-diagnostics-engine) already specify.

**Definition of completion.** Per [§27](#27-definition-of-done)'s general template, applied to Phase 1 as a whole: every stage named in [§7](#7-compiler-bootstrap-order) is implemented to at least the depth [§9](#9-mvp-definition) requires, has passing tests at the level [COMPILER_ARCHITECTURE.md §19](./COMPILER_ARCHITECTURE.md#19-testing-strategy) specifies for that stage, and is documented sufficiently that a new contributor can locate and understand it from [COMPILER_ARCHITECTURE.md §20](./COMPILER_ARCHITECTURE.md#20-repository-architecture)'s repository layout alone.

---

# 7. Compiler Bootstrap Order

```
Lexer
    ↓
Parser + CST
    ↓
AST
    ↓
Diagnostics
    ↓
Formatter
    ↓
Name Resolution
    ↓
Type Checker
    ↓
Semantic Analyzer (minimal rule set)
    ↓
HIR (minimal desugaring)
    ↓
RIR
    ↓
Rust Code Generator
    ↓
Cargo Build integration
    ↓
Soroban Build integration
    ↓
WASM
```

with the `kyne` CLI's basic subcommand scaffolding (per [TOOLCHAIN.md §9](./TOOLCHAIN.md#9-cli-commands)) built alongside the Lexer, since even the earliest usable tool needs an entry point.

**This order is deliberately aligned to [COMPILER_ARCHITECTURE.md §3](./COMPILER_ARCHITECTURE.md#3-compiler-pipeline)'s fixed execution pipeline, not a simplified or reordered variant of it.** Name Resolution, the Type Checker, and the Semantic Analyzer appear here in the same relative order that pipeline already fixes, and HIR lowering appears only after them — never before, since [COMPILER_ARCHITECTURE.md §3](./COMPILER_ARCHITECTURE.md#3-compiler-pipeline) requires semantic validity to be established before HIR lowering ever begins. What this bootstrap order *does* permit, consistent with [Vertical Slices](#roadmap-philosophy-principles), is building an early stage in **minimal, stubbed form** — a Semantic Analyzer initially checking only canonical member order and definite assignment, or an HIR lowering pass handling only the desugaring the Counter and Token examples actually require — specifically so a complete, working vertical slice exists sooner, with fuller rule coverage arriving in Phase 2. A stubbed stage still executes in its constitutionally fixed position; only its completeness, never its position, is reduced during bootstrap.

**The Security Analyzer and Optimization Passes are deliberately absent from this bootstrap order.** Per [COMPILER_ARCHITECTURE.md §3](./COMPILER_ARCHITECTURE.md#3-compiler-pipeline), both stages exist in the fixed execution pipeline, but neither is required to produce a correct, deployable WASM artifact — the Security Analyzer's findings are non-blocking by design ([COMPILER_ARCHITECTURE.md §11.2](./COMPILER_ARCHITECTURE.md#112-severity-levels)), and Optimization Passes are explicitly optional, behavior-preserving transformations ([COMPILER_ARCHITECTURE.md §13](./COMPILER_ARCHITECTURE.md#13-optimization)) whose absence changes nothing about correctness. Deferring both to Phase 2 is a direct application of [Compiler Correctness Before Optimization](./COMPILER_ARCHITECTURE.md#principle-3--compiler-correctness-before-optimization): an MVP compiler that never optimizes is fully conforming; an MVP compiler with an incomplete Semantic Analyzer is not.

**Why this order minimizes risk.** Each stage in this list depends only on stages above it, exactly mirroring [COMPILER_ARCHITECTURE.md §20](./COMPILER_ARCHITECTURE.md#20-repository-architecture)'s acyclic, forward-only dependency rule — there is no point in this sequence where later work forces earlier work to be redesigned, which is precisely [Stable Foundations](#roadmap-philosophy-principles) in practice. Placing Diagnostics immediately after AST, ahead of every semantic stage, is deliberate: every later stage needs to report errors, and building the reporting mechanism once, early, means no later stage improvises its own ad hoc error handling that must be reconciled afterward. Placing the Formatter immediately after Diagnostics — ahead of Name Resolution, the Type Checker, and everything semantic — is likewise deliberate: per [TOOLCHAIN.md §22](./TOOLCHAIN.md#22-library-first-compiler-architecture) *(cross-referencing [COMPILER_ARCHITECTURE.md §22](./COMPILER_ARCHITECTURE.md#22-library-first-compiler-architecture))*, the formatter needs only `kyne_lexer`, `kyne_parser`, and `kyne_cst`, so it can ship as a genuinely useful, complete tool — `kyne fmt` working on real projects — well before `kyne build` exists at all, giving Phase 1 an early, real vertical slice per [Incremental Value](#roadmap-philosophy-principles).

---

# 8. Repository Plan

| Crate | Phase introduced | Depends on | Corresponds to |
|---|---|---|---|
| `kyne_cli` | 1 | `kyne_driver` | The `kyne` binary, per [TOOLCHAIN.md §2](./TOOLCHAIN.md#2-toolchain-components). |
| `kyne_driver` | 1 (minimal) | every stage crate, incrementally | Pipeline orchestration, per [COMPILER_ARCHITECTURE.md §20](./COMPILER_ARCHITECTURE.md#20-repository-architecture). |
| `kyne_lexer` | 1 | — | [COMPILER_ARCHITECTURE.md §4](./COMPILER_ARCHITECTURE.md#4-lexical-analysis). |
| `kyne_parser` | 1 | `kyne_lexer` | [COMPILER_ARCHITECTURE.md §5](./COMPILER_ARCHITECTURE.md#5-parsing). |
| `kyne_cst` | 1 | `kyne_parser` | [COMPILER_ARCHITECTURE.md §6](./COMPILER_ARCHITECTURE.md#6-concrete-syntax-tree). |
| `kyne_ast` | 1 | `kyne_cst` | [COMPILER_ARCHITECTURE.md §7](./COMPILER_ARCHITECTURE.md#7-abstract-syntax-tree). |
| `kyne_diagnostics` | 1 | — (foundation crate) | [COMPILER_ARCHITECTURE.md §15](./COMPILER_ARCHITECTURE.md#15-diagnostics-engine). |
| `kyne_formatter` | 1 | `kyne_lexer`, `kyne_parser`, `kyne_cst` | [TOOLCHAIN.md §10](./TOOLCHAIN.md#10-formatter) — an implementation-level crate boundary not separately enumerated in [COMPILER_ARCHITECTURE.md §22](./COMPILER_ARCHITECTURE.md#22-library-first-compiler-architecture)'s original list; naming it explicitly here is the kind of implementation decision [GOVERNANCE.md §19](./GOVERNANCE.md#19-architecture-decision-records-adr) records as an ADR, not a KIP, since it adds detail without changing any specified behavior. |
| `kyne_resolver` | 1 | `kyne_ast` | [COMPILER_ARCHITECTURE.md §8](./COMPILER_ARCHITECTURE.md#8-name-resolution). |
| `kyne_types` | 1 | `kyne_resolver` | [COMPILER_ARCHITECTURE.md §9](./COMPILER_ARCHITECTURE.md#9-type-checker). |
| `kyne_semantics` | 1 (minimal), completed in 2 | `kyne_types` | [COMPILER_ARCHITECTURE.md §10](./COMPILER_ARCHITECTURE.md#10-semantic-analysis). |
| `kyne_hir` | 1 (minimal), completed in 2 | `kyne_semantics` | [COMPILER_ARCHITECTURE.md §12](./COMPILER_ARCHITECTURE.md#12-intermediate-representations). |
| `kyne_rir` | 1 | `kyne_hir` | As above, plus [MEMORY_MODEL.md §21](./MEMORY_MODEL.md#21-memory-planning-architecture)'s planners, completed in Phase 2. |
| `kyne_codegen` | 1 | `kyne_rir` | [COMPILER_ARCHITECTURE.md §14](./COMPILER_ARCHITECTURE.md#14-rust-code-generation). |
| `kyne_security` | 2 | `kyne_types` | [COMPILER_ARCHITECTURE.md §11](./COMPILER_ARCHITECTURE.md#11-security-analyzer). |
| `kyne_optimizer` | 2 | `kyne_hir` | [COMPILER_ARCHITECTURE.md §13](./COMPILER_ARCHITECTURE.md#13-optimization). |
| `kyne_cache` | 2 | — (foundation crate) | [COMPILER_ARCHITECTURE.md §17](./COMPILER_ARCHITECTURE.md#17-compiler-cache). |

**Dependencies.** This table's "Depends on" column is a restriction of [COMPILER_ARCHITECTURE.md §20](./COMPILER_ARCHITECTURE.md#20-repository-architecture)'s full, locked dependency graph to only the edges relevant during initial construction — it does not redefine that graph, and every edge here is already implied by it.

**Implementation priorities.** Within Phase 1, crates are built in the order [§7](#7-compiler-bootstrap-order) fixes. `kyne_security`, `kyne_optimizer`, and `kyne_cache` are deferred to Phase 2 in full, per [§7](#7-compiler-bootstrap-order)'s reasoning — they are not partially started during Phase 1.

---

# 9. MVP Definition

**The minimum viable compiler MUST:**

- Accept `.kyn` source for a single-contract project, per [LANGUAGE_SPEC.md §12](./LANGUAGE_SPEC.md#12-modules).
- Fully lex and parse Kyne's grammar, per [LANGUAGE_SPEC.md §1](./LANGUAGE_SPEC.md#1-lexical-structure)–[§2](./LANGUAGE_SPEC.md#2-grammar), with functioning error recovery, per [COMPILER_ARCHITECTURE.md §5](./COMPILER_ARCHITECTURE.md#5-parsing).
- Perform Name Resolution, Type Checking, and a Semantic Analyzer rule set sufficient to correctly accept or reject every construct used in the **Counter** and **Token** contracts from [LANGUAGE_SPEC.md §16](./LANGUAGE_SPEC.md#16-examples) — canonical member ordering, definite assignment, exhaustive `match`, event argument validation, and `auth` placement, at minimum.
- Lower through HIR and RIR and generate readable, idiomatic Rust, per [COMPILER_ARCHITECTURE.md §14](./COMPILER_ARCHITECTURE.md#14-rust-code-generation), for both example contracts.
- Successfully invoke Cargo Build and Soroban Build to produce a real, valid WASM artifact, per [RUNTIME_MODEL.md §3](./RUNTIME_MODEL.md#3-project-lifecycle).
- Provide a functioning `kyne fmt`, `kyne check`, and `kyne build`, per [TOOLCHAIN.md §9](./TOOLCHAIN.md#9-cli-commands).
- Produce diagnostics matching the shape [COMPILER_ARCHITECTURE.md §15](./COMPILER_ARCHITECTURE.md#15-diagnostics-engine) requires (Error Code, Title, Explanation, Reason, Suggested Fix) for every error the MVP's rule set can detect.

**The minimum viable compiler intentionally excludes:**

| Excluded | Why |
|---|---|
| Optimization Passes | Non-load-bearing for correctness, per [§7](#7-compiler-bootstrap-order)'s reasoning; deferred to Phase 2. |
| Security Analyzer | Non-blocking by design, per [COMPILER_ARCHITECTURE.md §11.2](./COMPILER_ARCHITECTURE.md#112-severity-levels); an MVP with zero Security Analyzer findings is still fully conforming, since every finding is advisory. |
| Incremental compilation / `kyne_cache` | An MVP recompiling from scratch on every invocation is slower but not incorrect; incrementality is a [Fast Feedback](./TOOLCHAIN.md#3-fast-feedback) quality-of-life property, not a correctness one. |
| Language Server | Requires the same underlying stages the MVP already builds, but as a separate, additional consumer per [TOOLCHAIN.md §16](./TOOLCHAIN.md#16-language-server) — deferred so Phase 1 effort stays focused on the compiler itself. |
| `kyne doc`, `kyne explain`, `kyne fix`, `kyne doctor` | Valuable developer-assistance commands, per [TOOLCHAIN.md §26](./TOOLCHAIN.md#26-developer-assistance-commands), none of which are required for the core edit-check-build-deploy loop an MVP must prove works. |
| Playground, package manager | Phase 3 and Phase 4 concerns respectively, per [§4](#4-project-phases); neither is reachable before a working compiler exists to build them on. |
| Full coverage of all six canonical examples | Escrow, Marketplace, Voting, and Multisig Wallet exercise the same constructs Counter and Token already exercise, per [LANGUAGE_SPEC.md §16](./LANGUAGE_SPEC.md#16-examples) — compiling all six is a Phase 1 stretch goal, not an MVP requirement, since two contracts already prove the pipeline end to end. |

**Why.** Per [Vertical Slices](#roadmap-philosophy-principles), the MVP's entire purpose is proving the pipeline is *connected and correct end to end*, not proving it is *feature-complete* — feature completeness, per [§4](#4-project-phases), is explicitly Phase 2's deliverable. An MVP that tried to be both would delay Phase 1's first release far past the point [Incremental Value](#roadmap-philosophy-principles) and [Ship Often](#roadmap-philosophy-principles) require.

---

# 10. Release Roadmap

| Release | Goals | Capabilities | Major deliverables |
|---|---|---|---|
| **v0.1** | Lexer, parser, diagnostics, and formatter functional. | `kyne fmt` works on real projects; parse errors are reported with correct spans. | Working CLI skeleton; `kyne_lexer`, `kyne_parser`, `kyne_cst`, `kyne_ast`, `kyne_diagnostics`, `kyne_formatter`. |
| **v0.2** | Type Checker, Semantic Analyzer (minimal), HIR (minimal). | `kyne check` correctly accepts or rejects the Counter and Token examples. | `kyne_resolver`, `kyne_types`, `kyne_semantics` (minimal), `kyne_hir` (minimal). |
| **v0.3** | Rust generation, Cargo integration, WASM generation. | `kyne build` produces a real, deployable WASM artifact for Counter and Token. | `kyne_rir`, `kyne_codegen`; a basic, documented deployment workflow. |
| **v0.5** | MVP complete, per [§9](#9-mvp-definition). | Full edit–check–build loop for simple contracts; `kyne test` functional against [STANDARD_LIBRARY.md §14](./STANDARD_LIBRARY.md#14-test-module)'s mock context. | Phase 1 complete; the first release suitable for early adopters willing to work with a minimal toolchain. |
| **v0.8** | Feature-complete compiler, per Phase 2's deliverable. | Every construct in [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) fully supported; all six canonical examples compile; Security Analyzer at full coverage per [COMPILER_ARCHITECTURE.md §11.1](./COMPILER_ARCHITECTURE.md#111-analyses); incremental compilation functional; Language Server functional. | `kyne_security`, `kyne_optimizer`, `kyne_cache` complete; Phase 2 complete. |
| **v1.0** | Production-ready developer experience and stable release, per Phase 3's deliverable. | Playground, initial package manager, documentation website, debugger, and full IDE support across [TOOLCHAIN.md §18](./TOOLCHAIN.md#18-ide-integration)'s editor list. | The first release under [GOVERNANCE.md §10](./GOVERNANCE.md#10-compatibility-policy)'s full compatibility guarantees — the point at which a deployed contract's compiler dependency is a genuinely stable one. |

**Completion expectations.** Each release's goals column MUST be fully satisfied, per its corresponding [§25](#25-release-gates) entry, before that version number is tagged — a release is never shipped "mostly" meeting its gate, per [§27](#27-definition-of-done).

**Relationship to phases.** v0.1 through v0.5 fall within Phase 1; v0.8 marks Phase 2's completion; v1.0 marks Phase 3's completion. Phase 4 — ecosystem growth — is explicitly **not** gated behind a single release number: per [§18](#18-governance-roadmap), ecosystem and governance maturation are ongoing, evidence-driven processes that continue throughout the v1.x series and beyond, not a milestone with a fixed version boundary.

---

# 11. Contributor Roadmap

| Category | Examples | Suitable for |
|---|---|---|
| Good First Issues | Adding a diagnostic's Suggested Fix text; adding a golden-file test case for an existing grammar production; fixing a formatter edge case against [LANGUAGE_SPEC.md §13](./LANGUAGE_SPEC.md#13-formatting-rules); documentation corrections. | New contributors, per [GOVERNANCE.md §12](./GOVERNANCE.md#12-contributor-journey)'s Community-Member-to-Contributor step. |
| Intermediate Issues | Implementing one Semantic Analyzer rule from [COMPILER_ARCHITECTURE.md §10](./COMPILER_ARCHITECTURE.md#10-semantic-analysis); adding one Security Analyzer heuristic from [COMPILER_ARCHITECTURE.md §11.1](./COMPILER_ARCHITECTURE.md#111-analyses); implementing one Standard Library module's conceptual API from [STANDARD_LIBRARY.md](./STANDARD_LIBRARY.md). | Contributors building toward Reviewer status. |
| Advanced Compiler Work | RIR ownership-decision logic, per [MEMORY_MODEL.md §21](./MEMORY_MODEL.md#21-memory-planning-architecture); incremental compilation's cache-invalidation correctness, per [COMPILER_ARCHITECTURE.md §16](./COMPILER_ARCHITECTURE.md#16-incremental-compilation); Language Server incremental re-analysis. | Maintainer- and Core-Maintainer-track contributors. |
| Research Topics | Future memory optimizations per [MEMORY_MODEL.md §17](./MEMORY_MODEL.md#17-future-memory-evolution); formal verification exploration per [§22](#22-future-research). | Contributors pursuing [§22](#22-future-research)'s exploratory work — never blocking a phase's completion. |
| Documentation | Populating [§12](#12-documentation-roadmap)'s remaining documents; expanding generated API documentation examples. | Any contributor, regardless of compiler-engineering background. |
| Testing | Expanding golden-file coverage; writing fuzz-testing harnesses per [§15](#15-security-roadmap). | Any contributor; a strong on-ramp into compiler-internals familiarity without requiring it up front. |

**Contributor progression.** This table's rightmost column tracks [GOVERNANCE.md §12](./GOVERNANCE.md#12-contributor-journey)'s progression directly — a contributor is not expected to start with Advanced Compiler Work, and every phase in [§4](#4-project-phases) is required, per [Contributors First](#roadmap-philosophy-principles), to expose issues across this entire table, not only its advanced end.

---

# 12. Documentation Roadmap

| Document | Priority | When needed |
|---|---|---|
| [`CONTRIBUTING.md`](../CONTRIBUTING.md) | Done | Written early in Phase 1, per [GOVERNANCE.md §3](./GOVERNANCE.md#3-governance-structure)'s role definitions requiring somewhere to explain how to begin. |
| Informal architecture guides (supplementing [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md)) | High | Phase 1–2, growing alongside the compiler itself. |
| Tutorials and examples | High | Phase 1 onward, growing through Phase 3 per [TOOLCHAIN.md §6](./TOOLCHAIN.md#6-project-templates)'s template system. |
| API documentation | High | Automatic and continuous, per [TOOLCHAIN.md §12](./TOOLCHAIN.md#12-documentation-generator) — this is generated, not separately authored, so its "roadmap" is simply `kyne doc` existing and being run in CI. |
| `PACKAGE_MANAGER.md` | Medium | Needed once Phase 3's package manager work begins, per [TOOLCHAIN.md §22](./TOOLCHAIN.md#22-future-package-manager). |
| `ECOSYSTEM.md` | Medium | Needed once Phase 4 begins, per [GOVERNANCE.md §18](./GOVERNANCE.md#18-ecosystem-governance). |
| Migration guides | Reactive, not scheduled | Needed only when a breaking edition change is actually proposed, per [GOVERNANCE.md §10](./GOVERNANCE.md#10-compatibility-policy) — there is nothing to migrate from until then. |

**Priorities.** With `CONTRIBUTING.md` complete, the informal architecture guides and tutorials/examples rows above are the next-highest-priority remaining documents, since every later item in this table depends on a contributor base those two continue to grow.

---

# 13. Testing Roadmap

| Test category | Introduced | Matures through |
|---|---|---|
| Lexer / Parser tests | Phase 1, from the first commit | Continuous — every grammar production gets a golden-file test as it is implemented, per [COMPILER_ARCHITECTURE.md §19](./COMPILER_ARCHITECTURE.md#19-testing-strategy). |
| Compiler snapshot tests | Phase 1 | The six canonical examples from [LANGUAGE_SPEC.md §16](./LANGUAGE_SPEC.md#16-examples) become the golden corpus for code-generation tests, per [COMPILER_ARCHITECTURE.md §19](./COMPILER_ARCHITECTURE.md#19-testing-strategy), as each becomes compilable. |
| Semantic tests | Phase 1 (minimal rule set), Phase 2 (full coverage) | Grows in lockstep with [COMPILER_ARCHITECTURE.md §10](./COMPILER_ARCHITECTURE.md#10-semantic-analysis)'s rule set. |
| Runtime / integration tests | Phase 1, from v0.3 onward | Requires a real Soroban toolchain invocation, per [COMPILER_ARCHITECTURE.md §19](./COMPILER_ARCHITECTURE.md#19-testing-strategy)'s end-to-end category — only possible once WASM generation exists. |
| Performance tests | Phase 2 onward | Deferred until there is a stable, feature-complete baseline worth benchmarking against, per [§14](#14-performance-roadmap). |
| Regression tests | Continuous from Phase 1 onward | Every fixed defect gets a permanent regression test, without exception, for the lifetime of the project. |

**Maturity progression.** Testing rigor is expected to increase monotonically — a category never regresses from "covered" to "uncovered" as the project matures, and [§27](#27-definition-of-done)'s completion template requires passing tests at every phase boundary, not only at v1.0.

---

# 14. Performance Roadmap

**Compilation speed.** Not a Phase 1 priority beyond "usable" — per [Compiler Correctness Before Optimization](./COMPILER_ARCHITECTURE.md#principle-3--compiler-correctness-before-optimization), Phase 1's compiler MAY be slow, provided it is correct; deliberate speed work begins in Phase 2.

**Memory usage.** Compiler-process memory usage (distinct from [MEMORY_MODEL.md](./MEMORY_MODEL.md)'s subject, which is *generated program* memory behavior) is monitored from Phase 1 onward but not actively optimized until real, multi-file projects in Phase 2 make it a measurable concern.

**Incremental compilation and caching.** Phase 2 deliverables, per [§4](#4-project-phases) and [COMPILER_ARCHITECTURE.md §16](./COMPILER_ARCHITECTURE.md#16-incremental-compilation)–[§17](./COMPILER_ARCHITECTURE.md#17-compiler-cache) — not attempted in Phase 1, consistent with [§9](#9-mvp-definition)'s explicit MVP exclusion.

**Parallel compilation** across independent files is exploratory, not committed to any specific phase in [§4](#4-project-phases) — it is noted in [§22](#22-future-research) as a direction worth investigating once Phase 2's incremental compilation provides the dependency graph such parallelism would need to be built on safely.

**Optimization philosophy.** Restated permanently from [COMPILER_ARCHITECTURE.md §13](./COMPILER_ARCHITECTURE.md#13-optimization): every performance improvement at every phase MUST satisfy "never changes observable behavior" before it is eligible to ship — this roadmap does not create, and cannot create, an exception to that rule for the sake of hitting a performance target sooner.

---

# 15. Security Roadmap

**Security Analyzer evolution.** Begins with zero or one check during Phase 1's MVP (per [§9](#9-mvp-definition)'s exclusion), grows to full coverage of [COMPILER_ARCHITECTURE.md §11.1](./COMPILER_ARCHITECTURE.md#111-analyses)'s complete list by v0.8, and continues to grow afterward as new heuristics are proposed via Compiler-category KIPs, per [GOVERNANCE.md §7](./GOVERNANCE.md#7-kip-categories).

**Fuzz testing.** Introduced in Phase 1 for the Lexer and Parser specifically — both are classic, high-value fuzz targets, and fuzzing them early catches crash-class defects (as opposed to correctness-class defects, which golden-file testing already covers) before later stages are built on top of a potentially fragile foundation.

**Regression testing.** Continuous from Phase 1 onward, per [§13](#13-testing-roadmap).

**Compiler hardening.** An ongoing Phase 2–4 concern, growing as the compiler's real-world attack surface (a compiler that processes arbitrary, potentially adversarial `.kyn` source, per [TOOLCHAIN.md §17](./TOOLCHAIN.md#17-playground)'s Playground use case specifically) becomes concrete rather than theoretical.

**Formal verification research.** Exploratory only, per [§22](#22-future-research) — not committed to any phase, pending research partnerships or contributor expertise that does not yet exist.

**Security audits.** A professional security audit of the compiler itself SHOULD occur before v1.0 ships, given that every contract compiled by a defective compiler inherits that defect regardless of how carefully the contract's own `.kyn` source was written — this is an explicit v1.0 release gate, per [§25](#25-release-gates).

**Priorities.** Fuzzing and Security Analyzer coverage are Phase 1–2 priorities because they are cheap relative to their risk reduction; a professional audit is deferred to immediately before v1.0 because it is expensive and most valuable against a feature-complete, stable target rather than a rapidly changing Phase 1–2 codebase.

---

# 16. Tooling Roadmap

**Language Server.** Phase 2, per [§4](#4-project-phases) — it depends on `kyne_resolver`, `kyne_types`, and `kyne_diagnostics` all reaching the maturity Phase 2 targets.

**VS Code.** The first editor integration, per [TOOLCHAIN.md §18](./TOOLCHAIN.md#18-ide-integration), given its install-base size; built immediately once the Language Server exists.

**Cursor, Windsurf.** Effectively simultaneous with VS Code, since both are VS Code-compatible per [TOOLCHAIN.md §18](./TOOLCHAIN.md#18-ide-integration) and require no additional integration work.

**Zed, Neovim.** Phase 2–3, lower priority than VS Code but low marginal cost, since both are thin LSP clients per [TOOLCHAIN.md §18](./TOOLCHAIN.md#18-ide-integration)'s existing design.

**Playground.** Phase 3, per [§4](#4-project-phases) — depends on a stable, sandboxable `kyne_codegen`, which Phase 2 provides.

**Formatter improvements.** Continuous from Phase 1 onward, tracking [LANGUAGE_SPEC.md §13](./LANGUAGE_SPEC.md#13-formatting-rules) exactly; not phase-gated since the formatter is complete in scope from v0.1, only refined afterward.

**Debugger.** Phase 3–4, per [TOOLCHAIN.md §23](./TOOLCHAIN.md#23-future-tooling)'s own deferral — a substantial undertaking this roadmap does not commit to a specific phase boundary beyond "after the compiler and runtime model it must instrument are both stable."

**Dependencies.** Every item in this section depends on the compiler stage(s) it instruments reaching at least the maturity level [§4](#4-project-phases)'s corresponding phase requires — no tooling item in this roadmap is scheduled ahead of the compiler capability it needs to function correctly.

---

# 17. Ecosystem Roadmap

**Official libraries.** Phase 4, once [TOOLCHAIN.md §22](./TOOLCHAIN.md#22-future-package-manager)'s package manager exists to distribute them through.

**Templates and examples.** Begin in Phase 1 with the two MVP examples (Counter, Token) and expand through Phase 3 to cover all six canonical examples from [LANGUAGE_SPEC.md §16](./LANGUAGE_SPEC.md#16-examples), per [TOOLCHAIN.md §6](./TOOLCHAIN.md#6-project-templates).

**Registry.** Phase 3–4, tied directly to the package manager's own timeline, per [TOOLCHAIN.md §22](./TOOLCHAIN.md#22-future-package-manager) and [GOVERNANCE.md §18](./GOVERNANCE.md#18-ecosystem-governance).

**Community packages.** Phase 4 onward, ongoing indefinitely — this is not a milestone with a completion state, only a beginning.

**Documentation portal.** Phase 3, combining [TOOLCHAIN.md §12](./TOOLCHAIN.md#12-documentation-generator)'s generated output with hand-written guides into a hosted site.

**Education.** Phase 4 onward, ongoing — tutorials, courses, and workshop material building on Phase 3's Playground and documentation portal.

**Long-term priorities.** Ecosystem work is sequenced last, deliberately: an ecosystem built around a compiler that is not yet feature-complete or stable (Phase 1–2) risks accumulating content and tooling that must be reworked once the underlying compiler catches up — [Stable Foundations](#roadmap-philosophy-principles) applies to ecosystem investment exactly as it applies to compiler architecture.

---

# 18. Governance Roadmap

This chapter does not redefine [GOVERNANCE.md](./GOVERNANCE.md) — it states only how that document's own evolution provisions are expected to play out in practice as this roadmap's phases proceed.

**Core maintainers.** Grow organically through Phase 1–2 as real contributors demonstrate the judgment [GOVERNANCE.md §12](./GOVERNANCE.md#12-contributor-journey) requires — this roadmap does not target a specific Core Maintainer count at any phase boundary, since [GOVERNANCE.md §3](./GOVERNANCE.md#3-governance-structure) ties promotion to demonstrated trust, not to a project-planning need for headcount.

**Working groups.** Plausible once KIP volume within a single category, per [GOVERNANCE.md §21](./GOVERNANCE.md#21-future-governance), justifies dedicated review capacity — most likely to first become relevant in Phase 2 or later, once Language- and Compiler-category KIPs are flowing at a real, sustained rate.

**Possible future Steering Committee.** This roadmap explicitly does **not** schedule this transition to any phase or version. [GOVERNANCE.md's own Governance Model](./GOVERNANCE.md#governance-model) ties it to evidence of sufficient maturity and community size, not to a calendar or a release number, and this roadmap preserves that deliberately — stating "Steering Committee by Phase 4" here would quietly override a decision GOVERNANCE.md made intentionally open-ended, which this document has no authority to do.

**Community growth.** Tracked via [§20](#20-success-metrics)'s contributor-growth metric, informing — but never dictating — the working-group and Steering-Committee questions above.

**Relationship to GOVERNANCE.md.** Every statement in this chapter is descriptive, not prescriptive: it predicts how GOVERNANCE.md's already-fixed mechanisms are likely to engage as the roadmap's phases unfold, and it creates no new governance obligation GOVERNANCE.md does not already establish.

---

# 19. Risks

| Risk | Description | Mitigation |
|---|---|---|
| Technical | A correctness defect in the compiler — especially one touching determinism or atomicity — is more severe than an ordinary software bug, since [RUNTIME_MODEL.md §4](./RUNTIME_MODEL.md#4-contract-lifecycle) means a defect can be silently baked into an immutable, deployed, value-holding contract. | [§13](#13-testing-roadmap)'s golden-file and snapshot testing from Phase 1 onward; a pre-v1.0 security audit, per [§15](#15-security-roadmap). |
| Community | Per [Reality Over Optimism](#roadmap-philosophy-principles), a small early contributor base may not grow quickly, slowing every phase after Phase 1. | [Contributors First](#roadmap-philosophy-principles)'s requirement that every phase expose Good First Issues, per [§11](#11-contributor-roadmap), lowering the barrier to the project's first external contributions. |
| Funding | Limited funding could stall Phase 2–4 work, particularly ecosystem and tooling investment that does not directly advance compiler correctness. | [Ship Often](#roadmap-philosophy-principles)'s frequent releases, per [§10](#10-release-roadmap), which demonstrate credible progress to potential future sponsors without requiring funding to have already been secured. |
| Maintenance | Nine constitutional documents plus a growing compiler codebase is a substantial ongoing synchronization burden — a KIP that updates [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) but is never fully implemented leaves the specification and the compiler disagreeing. | [GOVERNANCE.md §6](./GOVERNANCE.md#6-kip-lifecycle)'s explicit Implemented and Released stages, distinct from Accepted, make an unimplemented-but-accepted KIP a visible, trackable state rather than a silent gap. |
| Architectural | An early Phase 1 design mistake, discovered only once Phase 2 has built heavily on top of it, would be costly to correct. | [Stable Foundations](#roadmap-philosophy-principles) plus [COMPILER_ARCHITECTURE.md §20](./COMPILER_ARCHITECTURE.md#20-repository-architecture)'s acyclic crate boundaries, which limit how far a defect in one stage's implementation can propagate into another's. |
| Knowledge concentration | A single founder or small core team holding most of the project's context is a classic bus-factor risk, especially acute given [Reality Over Optimism](#roadmap-philosophy-principles)'s explicit single-founder assumption. | [§11](#11-contributor-roadmap)'s full contribution ladder and [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md)'s own extensive documentation, both designed specifically to make context transferable rather than tacit. |
| Contributor burnout | A single founder executing a ten-chapter roadmap alone is a genuine sustainability risk this document must not understate. | [Ship Often](#roadmap-philosophy-principles) deliberately breaks the work into small, completable releases rather than one continuous push to v1.0, so that progress and rest are both structurally possible within the plan itself, not only in spite of it. |

---

# 20. Success Metrics

| Metric | What it measures | Why it matters |
|---|---|---|
| Compiler quality | Defect density; golden-file and snapshot test pass rate, per [§13](#13-testing-roadmap). | Directly protects [RUNTIME_MODEL.md](./RUNTIME_MODEL.md)'s guarantees for every contract the compiler produces. |
| Compiler performance | Compile time and memory usage per project size, per [§14](#14-performance-roadmap). | Directly affects [Fast Feedback](./TOOLCHAIN.md#3-fast-feedback), a stated toolchain principle. |
| Contributor growth | Count of individuals at each [GOVERNANCE.md §3](./GOVERNANCE.md#3-governance-structure) role over time. | The clearest available signal against [§19](#19-risks)'s knowledge-concentration risk. |
| Issue resolution | Median time to a substantive first response (not necessarily to a fix, per [Reality Over Optimism](#roadmap-philosophy-principles)'s resourcing constraint). | A responsive project retains the contributors [§11](#11-contributor-roadmap) works to attract. |
| Documentation quality | Generated API documentation coverage, per [TOOLCHAIN.md §12](./TOOLCHAIN.md#12-documentation-generator); completeness of [§12](#12-documentation-roadmap)'s remaining documents. | Directly affects [FOUNDATION.md](./FOUNDATION.md#success-metrics)'s "first contract within one hour" success metric. |
| Language stability | Frequency of breaking changes post-v1.0, per [GOVERNANCE.md §10](./GOVERNANCE.md#10-compatibility-policy). | The measure of whether [RUNTIME_MODEL.md §4](./RUNTIME_MODEL.md#4-contract-lifecycle)'s durability promise is being honored in practice, not only on paper. |
| Developer adoption | Real, deployed Soroban contracts written in Kyne. | The ultimate measure of whether [FOUNDATION.md's mission](./FOUNDATION.md#mission) is being achieved at all — every other metric in this table is a leading indicator toward this one. |
| Testing coverage | Breadth across every category [COMPILER_ARCHITECTURE.md §19](./COMPILER_ARCHITECTURE.md#19-testing-strategy) defines, not a single blanket percentage. | A single coverage percentage can be misleading; category breadth verifies no entire class of defect (for example, code-generation regressions) is going unchecked. |

**Why metrics matter.** Per [§2](#2-roadmap-philosophy)'s success philosophy, this roadmap rejects subjective, felt-sense judgments of progress in favor of the objective gates in [§25](#25-release-gates) — this table is the longer-horizon counterpart to those gates, tracking whether the project is healthy in ways a single release's checklist cannot capture on its own.

---

# 21. Out of Scope

The following are excluded from this roadmap because they are already excluded by a locked constitutional document — this roadmap does not schedule work on any of them, at any phase:

| Item | Excluded by |
|---|---|
| Reflection | [LANGUAGE_PRINCIPLES.md's Non-Goals](./LANGUAGE_PRINCIPLES.md#non-goals); [LANGUAGE_SPEC.md's Language Commandments](./LANGUAGE_SPEC.md#15-language-commandments). |
| Macros | As above. |
| Async | [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md)'s enumerated v1 exclusions; [MEMORY_MODEL.md §17](./MEMORY_MODEL.md#17-future-memory-evolution). |
| Advanced (user-facing) generics | [LANGUAGE_SPEC.md §6.7](./LANGUAGE_SPEC.md#67-generics-a-closed-not-an-open-feature). |
| FFI | Not named in any constitutional document as a Kyne capability at all — out of scope by omission, consistent with [Contract-Oriented Design](./LANGUAGE_PRINCIPLES.md#why-contracts-are-the-unit-of-everything)'s narrow focus. |
| Alternative compiler backends | [COMPILER_ARCHITECTURE.md §18](./COMPILER_ARCHITECTURE.md#18-plugin-architecture)'s "no plugins in v1" decision; a second backend is explicitly speculative even as a plugin-architecture future possibility. |
| Multi-contract projects | [LANGUAGE_SPEC.md §3.1](./LANGUAGE_SPEC.md#31-declaration-and-members)'s one-project-one-contract rule. |
| Experimental features generally | [LANGUAGE_SPEC.md's Small Language Philosophy](./LANGUAGE_SPEC.md#every-keyword-must-earn-its-place) — nothing enters Kyne without first being a KIP, and this roadmap does not pre-allocate implementation effort to a feature that does not yet, and may never, exist as an accepted specification. |

**Why these remain outside the roadmap.** A roadmap that scheduled implementation work for a feature the constitutional documents have already excluded would itself be a quiet act of redefining those documents — exactly what this document's front matter forbids. Every item above stays out of scope for as long as, and only for as long as, its excluding document says so; a future KIP reversing one of those exclusions would, at that point, also update this roadmap to schedule the resulting work, but this roadmap does not anticipate that reversal.

---

# 22. Future Research

The following are exploratory only — genuinely open questions this roadmap does not commit to any phase, distinguished from [§21](#21-out-of-scope)'s permanently excluded items by the fact that none of these has been decided against:

- **AI-assisted development** — tooling that assists Kyne contract authors using language models, an application layer on top of Kyne rather than a change to Kyne itself, and therefore not obviously even a Kyne-project concern rather than a third-party one.
- **Formal verification** — proving properties of Kyne contracts beyond what [COMPILER_ARCHITECTURE.md §11](./COMPILER_ARCHITECTURE.md#11-security-analyzer)'s heuristic Security Analyzer can offer; a substantial, open research area in its own right.
- **Zero-knowledge integration** — Soroban-ecosystem-specific cryptographic capability that would, if pursued, require its own [STANDARD_LIBRARY.md](./STANDARD_LIBRARY.md) KIP once concretely scoped.
- **Alternative compiler backends** — as noted in [§21](#21-out-of-scope), currently foreclosed by [COMPILER_ARCHITECTURE.md §18](./COMPILER_ARCHITECTURE.md#18-plugin-architecture)'s v1 decision, but worth tracking as a long-horizon research question distinct from a near-term roadmap commitment.
- **Advanced optimization** — beyond [COMPILER_ARCHITECTURE.md §13](./COMPILER_ARCHITECTURE.md#13-optimization)'s specified passes, contingent on real-world compiled-contract evidence of where it would matter.
- **Research partnerships** — collaboration with academic or industry research groups on any of the above, contingent on relationships that do not yet exist.

**These are exploratory only.** None of the items in this chapter is scheduled to a phase in [§4](#4-project-phases), none is a release gate in [§25](#25-release-gates), and none should be read as a commitment this roadmap is making on the project's behalf — they are recorded here so that future interest in one of them has a documented starting point, consistent with [Transparent Abstraction](#1-vision)'s "nothing important should happen behind closed doors" applied to research direction as much as to implementation decisions.

---

# 23. Roadmap Invariants

**No implementation before specification.** Restated permanently from [GOVERNANCE.md §23](./GOVERNANCE.md#23-specification-first-development): every phase in this roadmap builds toward behavior a constitutional document already defines; no phase is permitted to implement behavior first and specify it afterward, with the sole exception GOVERNANCE.md §11 itself carves out for security emergencies.

**Ship incrementally.** Restated permanently from [Incremental Value](#roadmap-philosophy-principles) and [Ship Often](#roadmap-philosophy-principles): a phase that has produced no usable release in [§10](#10-release-roadmap)'s sequence for an extended stretch is a phase that has drifted from this roadmap's own discipline, regardless of how much unreleased progress has been made internally.

**Backward compatibility remains important.** Restated permanently from [GOVERNANCE.md §10](./GOVERNANCE.md#10-compatibility-policy): this roadmap's pre-v1.0 releases (v0.1 through v0.8) are explicitly **not** yet bound by full compatibility guarantees — early breaking changes within Phase 1–2 are expected and acceptable — but from v1.0 onward, every release this roadmap schedules MUST respect those guarantees in full, with no roadmap-driven exception.

**Compiler libraries remain the single source of truth.** Restated permanently from [COMPILER_ARCHITECTURE.md §22](./COMPILER_ARCHITECTURE.md#22-library-first-compiler-architecture) and [TOOLCHAIN.md's Toolchain Commandments](./TOOLCHAIN.md#24-toolchain-commandments): no phase in this roadmap schedules a tool (Language Server, formatter, documentation generator, Playground) with its own independent parser or checker — every tool, at whatever phase it is introduced, is built from the `kyne_*` crates already named in [§8](#8-repository-plan).

**Constitutional documents remain authoritative.** Restated permanently from this document's own front matter: no phase, release, or milestone in this roadmap may be used to justify a de facto change to any constitutional document's specified behavior — a roadmap item that seems to require one is a signal that a KIP is needed first, per [GOVERNANCE.md §5](./GOVERNANCE.md#5-language-evolution), not a signal that the roadmap should proceed around it.

**Why future contributors must preserve them.** Every invariant above exists to keep this roadmap's own evolution — which [GOVERNANCE.md §17](./GOVERNANCE.md#17-documentation-governance) permits to happen through ordinary review, unlike the nine constitutional documents — from becoming a backdoor through which constitutional decisions are effectively revised without ever passing through [GOVERNANCE.md](./GOVERNANCE.md)'s actual amendment process. A roadmap is supposed to be the flexible document in this project's documentation set; these invariants are what keep that flexibility from quietly leaking into the documents that are not supposed to be flexible at all.

---

# 24. Long-Term Vision

Kyne's long-term destination, restated from [FOUNDATION.md](./FOUNDATION.md) and given a roadmap-level accounting of how success is measured over many years:

**The easiest smart contract language for Soroban.** Measured by [§20](#20-success-metrics)'s developer-adoption metric and by [FOUNDATION.md's own "first contract within one hour"](./FOUNDATION.md#success-metrics) success criterion, revisited periodically as the language and tooling mature well past v1.0.

**The reference compiler for Stellar.** Measured by whether Kyne's generated Rust becomes a pattern Soroban developers recognize and trust even when they did not write it themselves, per [Transparent Abstraction](./LANGUAGE_PRINCIPLES.md#transparent-abstraction)'s promise that generated code remains genuinely reviewable.

**The best onboarding experience into Soroban.** Measured by how effectively Kyne serves as a stepping stone into the broader Soroban and Rust ecosystem for developers who started with no blockchain-specific background, per [FOUNDATION.md's Target Audience](./FOUNDATION.md#target-audience).

**A mature educational ecosystem.** Measured by [§17](#17-ecosystem-roadmap)'s education and documentation-portal work reaching a state where a newcomer's path from curiosity to a deployed contract is well-trodden and well-documented, not dependent on direct help from a core maintainer.

**Enterprise-grade tooling.** Measured by whether [§16](#16-tooling-roadmap)'s completed tooling roadmap — Language Server, debugger, profiler, coverage, full IDE support — meets the bar a team adopting Kyne for a serious, funded project would expect from any mature language toolchain.

**How success is measured over many years.** Not by this roadmap's own completion — a roadmap that reaches its final release and stops evolving has not succeeded, it has stalled, per [GOVERNANCE.md §9](./GOVERNANCE.md#9-release-policy)'s ongoing release cadence beyond v1.0. Long-term success is measured by whether Kyne, years after this document is written, is still being developed under the same specification-first discipline [§5](#5-phase-0-retrospective) and [GOVERNANCE.md §23](./GOVERNANCE.md#23-specification-first-development) established from the very beginning — the destination matters less than whether the route there, and every route after it, was walked the same disciplined way.

---

# 25. Release Gates

Every release MUST define objective completion criteria — no release in [§10](#10-release-roadmap) ships on a subjective sense of readiness.

**v0.1**
- Lexer complete: full tokenization of [LANGUAGE_SPEC.md §1](./LANGUAGE_SPEC.md#1-lexical-structure), with error recovery.
- Parser complete: full grammar coverage of [LANGUAGE_SPEC.md §2](./LANGUAGE_SPEC.md#2-grammar), with error recovery.
- Diagnostics functional: every parser and lexer error follows [COMPILER_ARCHITECTURE.md §15](./COMPILER_ARCHITECTURE.md#15-diagnostics-engine)'s required shape.
- Formatter functional: `kyne fmt` is idempotent and matches [LANGUAGE_SPEC.md §13](./LANGUAGE_SPEC.md#13-formatting-rules) exactly.
- Basic CLI operational: `kyne new`, `kyne fmt`, `kyne check` (parse-only at this stage) all function per [TOOLCHAIN.md §9](./TOOLCHAIN.md#9-cli-commands).

**v0.2**
- Type Checker complete for every type in [LANGUAGE_SPEC.md §6](./LANGUAGE_SPEC.md#6-types).
- Semantic Analyzer covers, at minimum, canonical ordering, definite assignment, exhaustive `match`, and event validation.
- HIR lowering functional for the Counter and Token examples.
- Diagnostics improved to cover every new error class the above stages introduce.

**v0.3**
- Rust generation functional and passing [COMPILER_ARCHITECTURE.md §14.3](./COMPILER_ARCHITECTURE.md#143-reviewability)'s reviewability bar for Counter and Token.
- Cargo integration functional.
- WASM generation functional.
- A documented, repeatable basic deployment workflow exists.

**v0.5**
- [§9](#9-mvp-definition)'s MVP Definition satisfied in full.
- `kyne test` functional against [STANDARD_LIBRARY.md §14](./STANDARD_LIBRARY.md#14-test-module)'s mock context.
- All v0.1–v0.3 gates remain satisfied (no regression).

**v0.8**
- Every construct in [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) supported; all six canonical examples from [LANGUAGE_SPEC.md §16](./LANGUAGE_SPEC.md#16-examples) compile and pass their golden-file tests.
- Security Analyzer implements every check in [COMPILER_ARCHITECTURE.md §11.1](./COMPILER_ARCHITECTURE.md#111-analyses).
- Incremental compilation and `.kyn/cache/` functional per [COMPILER_ARCHITECTURE.md §16](./COMPILER_ARCHITECTURE.md#16-incremental-compilation)–[§17](./COMPILER_ARCHITECTURE.md#17-compiler-cache).
- Language Server functional for autocomplete, hover, diagnostics, and go-to-definition, per [TOOLCHAIN.md §16](./TOOLCHAIN.md#16-language-server).
- [COMPILER_ARCHITECTURE.md §19](./COMPILER_ARCHITECTURE.md#19-testing-strategy)'s full test category list has non-trivial coverage.

**v1.0**
- Playground, initial package manager, and documentation website all functional, per Phase 3's deliverable.
- Full IDE support across [TOOLCHAIN.md §18](./TOOLCHAIN.md#18-ide-integration)'s editor list.
- A professional security audit, per [§15](#15-security-roadmap), has been completed and its findings resolved.
- [GOVERNANCE.md §10](./GOVERNANCE.md#10-compatibility-policy)'s full compatibility guarantees are formally in effect from this release forward.

**Why measurable release gates exist.** A gate expressed as "the compiler feels ready" cannot be verified by anyone other than the person who felt it, and cannot be handed to a new contributor as something concrete to complete. Every gate in this chapter is phrased so that a Contributor with no prior context could, in principle, verify it directly against a constitutional document's own section — which is the same discipline [§27](#27-definition-of-done) applies at the level of an individual stage, applied here at the level of an entire release.

---

# 26. Milestone Ownership

Every roadmap milestone SHOULD define:

| Field | Purpose |
|---|---|
| Owner | The individual or Maintainer-area accountable for the milestone's completion, per [GOVERNANCE.md §3](./GOVERNANCE.md#3-governance-structure). |
| Dependencies | Which earlier milestones, per [§7](#7-compiler-bootstrap-order) and [§8](#8-repository-plan), must be complete first. |
| Success criteria | The specific [§25](#25-release-gates) or [§27](#27-definition-of-done) checklist the milestone satisfies. |
| Blocking risks | Which of [§19](#19-risks)'s named risks most threaten this specific milestone. |
| Estimated implementation complexity | A relative, non-numeric assessment (for example, "comparable to the Parser" or "comparable to the Type Checker") — consistent with this document's front matter, no date or team-size estimate is attached. |

**Initial ownership.** Every Phase 1 milestone's Owner field MAY initially default to the Founder, per [GOVERNANCE.md §15](./GOVERNANCE.md#15-leadership), consistent with [Reality Over Optimism](#roadmap-philosophy-principles)'s single-founder assumption — this is expected to diversify across Maintainers as Phase 1 and Phase 2 proceed and [§11](#11-contributor-roadmap)'s contribution ladder produces contributors ready for larger ownership.

**Why ownership matters.** A milestone with no named owner is a milestone anyone can assume someone else is handling — per [§19](#19-risks)'s knowledge-concentration and burnout risks, explicit ownership is what makes a milestone's actual status legible to the rest of the project, rather than a question that only gets answered when the milestone is already overdue.

---

# 27. Definition of Done

Every implementation milestone MUST define completion objectively before work on it begins, not retroactively once work feels finished. The template, illustrated for the Parser:

**"Parser complete" means:**
- Grammar implemented: every production in [LANGUAGE_SPEC.md §2](./LANGUAGE_SPEC.md#2-grammar) is accepted and every non-production is rejected.
- Error recovery implemented: per [COMPILER_ARCHITECTURE.md §5](./COMPILER_ARCHITECTURE.md#5-parsing)'s synchronization strategy.
- Unit tests passing: per-production tests, per [COMPILER_ARCHITECTURE.md §19](./COMPILER_ARCHITECTURE.md#19-testing-strategy).
- Snapshot tests passing: CST golden-file tests for representative real source.
- Documentation updated: the Parser's own section of the project's internal architecture guide, per [§12](#12-documentation-roadmap), reflects its actual, current behavior.
- Examples included: the Parser correctly handles all six canonical examples from [LANGUAGE_SPEC.md §16](./LANGUAGE_SPEC.md#16-examples), or, during Phase 1, at minimum the two MVP examples per [§9](#9-mvp-definition).

The same six-part template — grammar/rules implemented, error handling implemented, unit tests passing, snapshot tests passing, documentation updated, examples included — applies to every other stage in [§7](#7-compiler-bootstrap-order), substituting that stage's own constitutional document section for "grammar" and its own relevant error category for "error recovery."

**Why "almost complete" is not an acceptable project state.** An "almost complete" Semantic Analyzer is not 90% as safe as a complete one — per [COMPILER_ARCHITECTURE.md's own invariant](./COMPILER_ARCHITECTURE.md#the-compiler-never-generates-undefined-runtime-behavior), a single missing rule means the compiler can accept a program whose behavior [RUNTIME_MODEL.md](./RUNTIME_MODEL.md) does not actually define, which is a correctness gap, not a partial success. This document's bootstrap strategy already accounts for staged completeness honestly — a stage MAY be scoped to a *deliberately reduced* rule set during Phase 1, per [§7](#7-compiler-bootstrap-order), and that reduced scope, once fully implemented, is itself "done" for that phase. What this section forbids is a stage that is neither fully implemented against its *stated* scope nor honestly re-scoped — an unacknowledged gap between what a stage claims to do and what it actually does.

---

# Roadmap Commandments

1. **Specifications come before implementation.** Restated permanently from [§23](#23-roadmap-invariants) and [GOVERNANCE.md §23](./GOVERNANCE.md#23-specification-first-development) — the one rule every other commandment in this list assumes as already settled.

2. **Small releases are better than large rewrites.** Restated permanently from [Ship Often](#roadmap-philosophy-principles): this roadmap's six-release sequence in [§10](#10-release-roadmap) exists specifically so that no single release is large enough to be a "rewrite" in disguise.

3. **Every phase delivers value.** Restated permanently from [Incremental Value](#roadmap-philosophy-principles): a phase in [§4](#4-project-phases) that produced nothing a real user could touch would be a phase this roadmap failed to scope correctly.

4. **Contributors build on stable foundations.** Restated permanently from [Stable Foundations](#roadmap-philosophy-principles): [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md)'s architecture is not re-opened by roadmap pressure, ever, regardless of how appealing a shortcut around it might seem at a given phase.

5. **Security work is never optional.** Restated permanently from [§15](#15-security-roadmap) and [GOVERNANCE.md §1](./GOVERNANCE.md#1-governance-philosophy): fuzzing, Security Analyzer coverage, and the pre-v1.0 audit are release-gated requirements, per [§25](#25-release-gates), not aspirational extras a busy phase can quietly drop.

6. **Documentation evolves with implementation.** Restated permanently from [§27](#27-definition-of-done)'s own completion template: a stage is not "done" while its documentation still describes an earlier, less complete version of it.

7. **Backward compatibility is respected.** Restated permanently from [§23](#23-roadmap-invariants): explicitly relaxed before v1.0, per that same invariant, and non-negotiable from v1.0 onward.

8. **Compiler correctness outweighs premature optimization.** Restated permanently from [§14](#14-performance-roadmap) and [COMPILER_ARCHITECTURE.md's Principle 3](./COMPILER_ARCHITECTURE.md#principle-3--compiler-correctness-before-optimization): no phase in this roadmap trades a correctness guarantee for a performance gain, at any point, for any reason.

**Why these are permanent.** Every commandment above is a restatement, at the roadmap level, of a rule a constitutional document already fixed permanently — they are gathered here specifically so that a contributor planning a specific phase's work can consult one list rather than re-deriving each rule's roadmap-level consequence from nine separate source documents every time. Their permanence is inherited, not independently asserted: they remain permanent for exactly as long as the constitutional documents they restate remain constitutional, per [GOVERNANCE.md §25](./GOVERNANCE.md#25-constitutional-documents).

---

# Non-Goals

This document does **not** define:

- **Language syntax** — defined in [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md).
- **Runtime semantics** — defined in [RUNTIME_MODEL.md](./RUNTIME_MODEL.md).
- **Compiler architecture** — defined in [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md); this roadmap sequences its construction but does not redesign it.
- **Memory model** — defined in [MEMORY_MODEL.md](./MEMORY_MODEL.md).
- **Standard library behavior** — defined in [STANDARD_LIBRARY.md](./STANDARD_LIBRARY.md).
- **Governance policy** — defined in [GOVERNANCE.md](./GOVERNANCE.md); this roadmap describes how that policy is expected to engage over time, per [§18](#18-governance-roadmap), without altering it.
- **Implementation details of specific compiler algorithms** — a matter for future Architecture Decision Records, per [GOVERNANCE.md §19](./GOVERNANCE.md#19-architecture-decision-records-adr), not for this roadmap.
- **Package manager specification** — reserved for a future `PACKAGE_MANAGER.md`, per [§12](#12-documentation-roadmap).

---

# Cross References

This document is informed by, and MUST remain consistent with:

- [FOUNDATION.md](./FOUNDATION.md) — the mission this roadmap's every phase ultimately serves.
- [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md) — the design philosophy this roadmap's prioritization decisions must never override.
- [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) — the complete language surface [§9](#9-mvp-definition) and [§25](#25-release-gates) measure implementation completeness against.
- [RUNTIME_MODEL.md](./RUNTIME_MODEL.md) — the execution guarantees every release gate in [§25](#25-release-gates) implicitly protects.
- [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md) — the pipeline and crate structure [§7](#7-compiler-bootstrap-order) and [§8](#8-repository-plan) sequence the construction of.
- [MEMORY_MODEL.md](./MEMORY_MODEL.md) — the memory guarantees Phase 2's Ownership/Memory/Storage Planner work realizes.
- [STANDARD_LIBRARY.md](./STANDARD_LIBRARY.md) — the API surface Phase 1–2 implement incrementally.
- [TOOLCHAIN.md](./TOOLCHAIN.md) — the developer experience Phase 3 completes.
- [GOVERNANCE.md](./GOVERNANCE.md) — the process governing every KIP this roadmap's implementation work will eventually require, and the constitutional protection this document itself is explicitly excluded from, per this document's own front matter.

[`CONTRIBUTING.md`](../CONTRIBUTING.md) now exists. The following documents remain anticipated but not yet written, and this document's [Non-Goals](#non-goals) section reserves their scope explicitly: `PACKAGE_MANAGER.md`, `ECOSYSTEM.md`, and `RELEASE_PROCESS.md`. Until each exists, this document does not speculate on their contents beyond the scope boundary already stated above.

---

# Closing

This document is Kyne's engineering execution plan, not its constitution. The nine documents it implements define, permanently, what Kyne is; this document defines, provisionally and revisably, the order in which that already-settled destination gets built. A future contributor should be able to open this roadmap at any point in Kyne's history, find the phase currently in progress, and understand exactly what remains before the next release gate is met — and should feel free to propose a better sequencing, a reprioritized phase, or a revised release boundary through ordinary review, without that proposal ever needing to touch, question, or reopen a single sentence of the nine documents this roadmap exists only to carry out.
