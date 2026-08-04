# ARCHITECTURE.md

# The Kyne Repository Architecture

**Status:** Operational — **not constitutional.** This document is not one of Kyne's nine constitutional documents ([FOUNDATION.md](docs/FOUNDATION.md), [LANGUAGE_PRINCIPLES.md](docs/LANGUAGE_PRINCIPLES.md), [LANGUAGE_SPEC.md](docs/LANGUAGE_SPEC.md), [RUNTIME_MODEL.md](docs/RUNTIME_MODEL.md), [COMPILER_ARCHITECTURE.md](docs/COMPILER_ARCHITECTURE.md), [MEMORY_MODEL.md](docs/MEMORY_MODEL.md), [STANDARD_LIBRARY.md](docs/STANDARD_LIBRARY.md), [TOOLCHAIN.md](docs/TOOLCHAIN.md), [GOVERNANCE.md](docs/GOVERNANCE.md)) and, per [GOVERNANCE.md §17](docs/GOVERNANCE.md#17-documentation-governance), it is an **informative** document — like [ROADMAP.md](docs/ROADMAP.md), it evolves through ordinary code review, not through [GOVERNANCE.md §25](docs/GOVERNANCE.md#25-constitutional-documents)'s elevated constitutional process. It describes how the Kyne repository is organized *today and as it grows*, and it MUST remain consistent with every constitutional document without ever amending one.

This document is the primary onboarding document for contributors after `README.md`. It answers **"how is the Kyne repository organized, and where should contributors make changes?"** — it does not answer "how does the compiler work?", which is [COMPILER_ARCHITECTURE.md](docs/COMPILER_ARCHITECTURE.md)'s question, already answered there in full.

This specification uses **RFC 2119** keywords normatively: **MUST**, **MUST NOT**, **SHALL**, **SHOULD**, **SHOULD NOT**, and **MAY** carry their standard meanings.

---

# 1. Purpose

**Why this document exists.** [COMPILER_ARCHITECTURE.md](docs/COMPILER_ARCHITECTURE.md) specifies what the compiler must do, in behavioral, RFC-2119 terms, deliberately independent of any particular repository layout. Someone still has to answer the ordinary engineering question that specification does not: which directory does a new contributor clone into, which crate do they open first, and which file do they touch to fix a specific class of bug. This document answers that question.

**Difference between repository architecture and compiler architecture.** [COMPILER_ARCHITECTURE.md](docs/COMPILER_ARCHITECTURE.md) is a **behavioral contract** — it is satisfied by any implementation that produces the right observable behavior, per its own [Non-Goals](docs/COMPILER_ARCHITECTURE.md#non-goals). This document is an **organizational map** of one specific, real implementation of that contract — the actual Kyne repository, its actual directories, and its actual crates. [COMPILER_ARCHITECTURE.md §20](docs/COMPILER_ARCHITECTURE.md#20-repository-architecture) already fixes the `compiler/` subdirectory's internal layout and its acyclic dependency rule as a MUST-level architectural requirement; this document does not repeat that specification's authority, it operates underneath it, filling in the repository-wide detail — crate ownership, tooling crates, the standard library's own repository home, testing and documentation layout — that COMPILER_ARCHITECTURE.md correctly left unspecified because it is an operational concern, not a behavioral one.

**Relationship to constitutional documents.** This document MUST NOT redefine language syntax, runtime semantics, compiler behavior, the memory model, standard library behavior, toolchain behavior, or governance — every one of those remains exclusively defined by its own constitutional document, per [Repository Scope](#repository-scope). Where this document names a crate or a directory, it is naming a place where already-specified behavior is implemented, never specifying new behavior of its own.

**Relationship to ROADMAP.md.** [ROADMAP.md](docs/ROADMAP.md) sequences *when* each part of this repository gets built, phase by phase and release by release. This document describes *where* that work lands once built. The two are companion operational documents: ROADMAP.md's [Repository Plan](docs/ROADMAP.md#8-repository-plan) names crates and their introduction phase; this document is that plan's permanent, standing reference once a crate exists, kept current independent of any single release's status.

---

# 2. Guiding Philosophy

**Library First.** Every major capability exists as a reusable library, and the CLI, formatter, Language Server, documentation generator, and playground are each only one consumer among several, never the sole home of any capability's logic. This exists because a capability implemented only inside a binary is a capability no other tool can reuse without duplicating it — and duplicated logic is exactly the drift risk [TOOLCHAIN.md §1](docs/TOOLCHAIN.md#1-toolchain-philosophy) identifies as an integrated toolchain's central problem to avoid. The compiler libraries, per [COMPILER_ARCHITECTURE.md §22](docs/COMPILER_ARCHITECTURE.md#22-library-first-compiler-architecture), remain the single source of truth for what a Kyne program means; this repository's entire crate structure exists to keep that true as the repository grows, not only at the moment COMPILER_ARCHITECTURE.md was written.

**Single Responsibility.** Every crate in this repository exists for exactly one reason, stated in one sentence a new contributor can read before opening a single file inside it. A crate accumulating a second, unrelated responsibility MUST be split before that accumulation becomes entangled enough that splitting it later requires a rewrite rather than a refactor. This mirrors [COMPILER_ARCHITECTURE.md §2](docs/COMPILER_ARCHITECTURE.md#2-compiler-overview)'s "one responsibility, one input, one output" pipeline-stage discipline, applied here to the repository's crate boundaries specifically, including the tooling and standard library crates COMPILER_ARCHITECTURE.md itself does not enumerate.

**Forward-Only Dependencies.** Dependencies flow in exactly one direction through this repository, with no circular dependency and no hidden dependency introduced through a side channel (a shared mutable global, an environment variable, an implicit file-system convention). [§10](#10-dependency-rules) and [§21](#21-dependency-graph) make this rule concrete at the level of specific crates. It exists because a dependency cycle between two crates means neither can be built, tested, or understood without the other — silently merging two crates that were deliberately kept separate, without the merge ever being a deliberate decision anyone reviewed.

**Small Crates.** This repository prefers many focused crates to a few enormous ones, because a crate a single contributor can hold in their head in full — its types, its invariants, its test suite — is a crate that can be reviewed, debugged, and safely modified by someone encountering it for the first time. A crate too large to read in one sitting is a crate whose true internal coupling is probably already violating Single Responsibility, whether or not it has been named as more than one crate yet.

**Explicit Ownership.** Every crate has a clearly identified owner, per [GOVERNANCE.md §3](docs/GOVERNANCE.md#3-governance-structure)'s Maintainer role, and every subsystem's ownership is discoverable without asking — per [§4](#4-compiler-crates) and [§5](#5-tooling-crates)'s per-crate tables. This exists for the same reason [ROADMAP.md §26](docs/ROADMAP.md#26-milestone-ownership) requires named milestone owners: an unowned crate is a crate anyone can assume someone else is maintaining, which in practice means no one is.

**Documentation as Architecture.** Every architectural decision made in this repository is documented somewhere a future contributor can find it — either in this document, in an Architecture Decision Record per [§11](#11-architecture-decision-records), or in a crate's own top-level documentation. The repository's own structure is expected to be self-explanatory to the degree this principle is actually honored: a contributor who can read directory and crate names and correctly guess their purpose is experiencing this principle working; a contributor who cannot is experiencing a gap this document, or an ADR, should be closing.

---

# 3. Workspace Layout

```
.
├── README.md
├── ARCHITECTURE.md
├── LICENSE
├── compiler/
├── stdlib/
├── tools/
├── docs/
├── examples/
├── tests/
├── website/
└── .github/
```

| Directory | Contents | Owned |
|---|---|---|
| `README.md` | The repository's entry point — project summary, quick start, links to this document and to `docs/`. | Source-owned. |
| `ARCHITECTURE.md` | This document. | Source-owned. |
| `LICENSE` | The project's license. | Source-owned. |
| `compiler/` | Every compiler crate, per [COMPILER_ARCHITECTURE.md §20](docs/COMPILER_ARCHITECTURE.md#20-repository-architecture) and [§4](#4-compiler-crates) below. | Source-owned. |
| `stdlib/` | The standard library's implementation, per [§6](#6-standard-library-repository). | Source-owned. |
| `tools/` | Every tooling crate — formatter, Language Server, documentation generator, playground, test runner — per [§5](#5-tooling-crates). | Source-owned. |
| `docs/` | Every constitutional document, [ROADMAP.md](docs/ROADMAP.md), this document's own copy is *not* here (it lives at the root, per this table's first two rows), and the ADR archive, per [§9](#9-documentation) and [§11](#11-architecture-decision-records). | Source-owned. |
| `examples/` | The canonical, tutorial, and regression example contracts, per [§7](#7-examples). | Source-owned. |
| `tests/` | Repository-wide, cross-crate testing assets, per [§8](#8-tests) — distinct from each crate's own internal `tests/`. | Source-owned. |
| `website/` | The documentation portal and hosted Playground's own source, per [TOOLCHAIN.md §17](docs/TOOLCHAIN.md#17-playground), introduced in Phase 3 per [ROADMAP.md §4](docs/ROADMAP.md#4-project-phases). | Source-owned. |
| `.github/` | CI configuration, issue templates, and other repository-operational files — explicitly out of this document's scope, per [Non-Goals](#non-goals). | Source-owned, operational detail. |

**Source-owned versus generated content.** Every directory in this table is source-owned: version-controlled, hand-authored or hand-organized, and never overwritten by a build. This mirrors, at the repository level, the distinction [TOOLCHAIN.md §4](docs/TOOLCHAIN.md#4-project-layout) already draws for an individual Kyne project between `src/`/`tests/`/`docs/` (user-owned) and `build/`/`.kyn/` (tool-generated) — this repository's own build output (compiled binaries, generated documentation sites, `target/`-equivalent build artifacts) is never checked in and has no corresponding row in this table, since [§9](#9-documentation)'s generated documentation and any compiled binaries are build products, not source.

**Note on the current repository.** All nine constitutional documents, including `FOUNDATION.md`, live under `docs/` — there is no separate `kyne/` directory; see [`docs/adr/ADR-0002-foundation-md-location.md`](docs/adr/ADR-0002-foundation-md-location.md) for the record of that correction. As of this document's original writing, the repository contained only Phase 0's constitutional documents; per [`WORKSPACE_BOOTSTRAP_REPORT.md`](WORKSPACE_BOOTSTRAP_REPORT.md), the layout above has since been realized as Phase 1 — Milestone 1's actual repository skeleton.

---

# 4. Compiler Crates

Every crate below lives under `compiler/`, per [COMPILER_ARCHITECTURE.md §20](docs/COMPILER_ARCHITECTURE.md#20-repository-architecture) and [§22](docs/COMPILER_ARCHITECTURE.md#22-library-first-compiler-architecture), which remain the sole authority on each crate's *behavior*; this table adds only the repository-operational detail those sections did not need to specify.

| Crate | Purpose | Public API (conceptual) | Depends on | Consumed by |
|---|---|---|---|---|
| `kyne_lexer` | Tokenization, per [COMPILER_ARCHITECTURE.md §4](docs/COMPILER_ARCHITECTURE.md#4-lexical-analysis). | A function from source text to a token stream. | — | `kyne_parser`, `kyne_formatter` |
| `kyne_parser` | Grammar enforcement and CST construction, per [COMPILER_ARCHITECTURE.md §5](docs/COMPILER_ARCHITECTURE.md#5-parsing). | A function from a token stream to a CST plus diagnostics. | `kyne_lexer`, `kyne_diagnostics` | `kyne_cst`, `kyne_formatter` |
| `kyne_cst` | The CST data structure and CST→AST lowering, per [COMPILER_ARCHITECTURE.md §6](docs/COMPILER_ARCHITECTURE.md#6-concrete-syntax-tree)–[§7](docs/COMPILER_ARCHITECTURE.md#7-abstract-syntax-tree). | The CST type; a lowering function to AST. | `kyne_parser` | `kyne_ast`, `kyne_formatter`, `kyne_docgen`, Language Server |
| `kyne_ast` | The AST data structure and shared visitor abstraction. | The AST type; visitor traits. | `kyne_cst` | `kyne_resolver`, `kyne_docgen` |
| `kyne_resolver` | Name Resolution, per [COMPILER_ARCHITECTURE.md §8](docs/COMPILER_ARCHITECTURE.md#8-name-resolution). | A function from AST to a resolved, annotated AST plus a symbol table. | `kyne_ast`, `kyne_diagnostics` | `kyne_types`, Language Server |
| `kyne_types` | The Type Checker, per [COMPILER_ARCHITECTURE.md §9](docs/COMPILER_ARCHITECTURE.md#9-type-checker). | A function from resolved AST to a typed AST. | `kyne_resolver`, `kyne_diagnostics` | `kyne_semantics`, Language Server |
| `kyne_semantics` | The Semantic Analyzer, per [COMPILER_ARCHITECTURE.md §10](docs/COMPILER_ARCHITECTURE.md#10-semantic-analysis). | A function that validates a typed AST, producing diagnostics. | `kyne_types`, `kyne_diagnostics` | `kyne_hir` |
| `kyne_security` | The Security Analyzer, per [COMPILER_ARCHITECTURE.md §11](docs/COMPILER_ARCHITECTURE.md#11-security-analyzer). | A function that analyzes a typed AST, producing non-blocking findings. | `kyne_types`, `kyne_diagnostics` | `kyne_driver` |
| `kyne_hir` | HIR construction, per [COMPILER_ARCHITECTURE.md §12](docs/COMPILER_ARCHITECTURE.md#12-intermediate-representations). | A function from a validated, typed AST to HIR. | `kyne_semantics` | `kyne_optimizer`, `kyne_rir` |
| `kyne_optimizer` | Optimization passes, per [COMPILER_ARCHITECTURE.md §13](docs/COMPILER_ARCHITECTURE.md#13-optimization). | A function from HIR to optimized HIR. | `kyne_hir` | `kyne_rir` |
| `kyne_rir` | RIR lowering — including the Ownership, Memory, and Storage Planners of [MEMORY_MODEL.md §21](docs/MEMORY_MODEL.md#21-memory-planning-architecture) — per [COMPILER_ARCHITECTURE.md §12](docs/COMPILER_ARCHITECTURE.md#12-intermediate-representations). | A function from (optimized) HIR to RIR. | `kyne_hir` (or `kyne_optimizer`'s output) | `kyne_codegen` |
| `kyne_codegen` | Rust Code Generation, per [COMPILER_ARCHITECTURE.md §14](docs/COMPILER_ARCHITECTURE.md#14-rust-code-generation). | A function from RIR to a formatted Rust source tree. | `kyne_rir` | `kyne_driver`, `kyne_playground` |
| `kyne_diagnostics` | The shared diagnostic type, formatting, and code namespace, per [COMPILER_ARCHITECTURE.md §15](docs/COMPILER_ARCHITECTURE.md#15-diagnostics-engine). | The `Diagnostic` type; rendering functions. | — (foundation crate) | Every stage crate above |
| `kyne_cache` | The compiler cache, per [COMPILER_ARCHITECTURE.md §17](docs/COMPILER_ARCHITECTURE.md#17-compiler-cache). | Content-hash-keyed get/put over each stage's serializable output. | — (foundation crate) | `kyne_driver` |
| `kyne_driver` | Pipeline orchestration — the only crate depending on every stage above, per [COMPILER_ARCHITECTURE.md §20](docs/COMPILER_ARCHITECTURE.md#20-repository-architecture). | A function from a project's `.kyn` files to a compiled artifact or diagnostics. | Every crate above | `kyne_cli`, `kyne_test_runner`, `kyne_playground` |
| `kyne_cli` | The `kyne` binary itself. | N/A — a binary, not a library. | `kyne_driver`, every `tools/` crate per [§5](#5-tooling-crates) | End users |

**Ownership.** Every crate in this table has exactly one owning Maintainer or Maintainer group, per [GOVERNANCE.md §3](docs/GOVERNANCE.md#3-governance-structure). In Phase 1, per [ROADMAP.md §26](docs/ROADMAP.md#26-milestone-ownership), ownership of every crate in this table MAY default to the Founder, diversifying as [GOVERNANCE.md §12](docs/GOVERNANCE.md#12-contributor-journey)'s contributor progression produces Maintainers ready to own a specific crate.

**Do not implement.** This table describes architecture, not code — no function signature above is a binding API contract; each is a conceptual description sufficient for a contributor to know what a crate is for and what it touches.

---

# 5. Tooling Crates

Every crate below lives under `tools/`. Per [TOOLCHAIN.md §2](docs/TOOLCHAIN.md#2-toolchain-components) and [COMPILER_ARCHITECTURE.md §22](docs/COMPILER_ARCHITECTURE.md#22-library-first-compiler-architecture), **every one of these crates reuses `compiler/` libraries and MUST NOT independently implement any language behavior** — no tool in this table contains its own parser, its own type checker, or its own formatting logic distinct from what [§4](#4-compiler-crates)'s crates already provide.

| Crate | Purpose | Depends on |
|---|---|---|
| `kyne_formatter` | `kyne fmt`, per [TOOLCHAIN.md §10](docs/TOOLCHAIN.md#10-formatter). | `kyne_lexer`, `kyne_parser`, `kyne_cst` only — deliberately not `kyne_types` or later stages, since formatting a syntactically valid file never requires it to also type-check. |
| `kyne_lsp` | The Language Server, per [TOOLCHAIN.md §16](docs/TOOLCHAIN.md#16-language-server), launched via a `kyne lsp` subcommand rather than a separate binary, per [§13's "One Tool" implication](docs/TOOLCHAIN.md#3-cli-philosophy). | `kyne_cst`, `kyne_ast`, `kyne_resolver`, `kyne_types`, `kyne_diagnostics`. |
| `kyne_docgen` | `kyne doc`, per [TOOLCHAIN.md §12](docs/TOOLCHAIN.md#12-documentation-generator). | `kyne_cst`, `kyne_ast`. |
| `kyne_test_runner` | `kyne test`'s test-discovery and mock-execution-context logic, per [TOOLCHAIN.md §13](docs/TOOLCHAIN.md#13-test-runner). | `kyne_driver`, the standard library's `test/` module implementation per [§6](#6-standard-library-repository). |
| `kyne_playground` | The browser/server-hosted Playground, per [TOOLCHAIN.md §17](docs/TOOLCHAIN.md#17-playground) — the one tool in this table not exposed as a `kyne` subcommand, since it is a hosted service rather than a local CLI invocation. | `kyne_driver`, `kyne_formatter`, `kyne_cache`'s sandboxing-relevant subset. |

**How every tool reuses compiler libraries.** Every "Depends on" column above names only `compiler/` crates from [§4](#4-compiler-crates) — no `tools/` crate depends on another `tools/` crate, and no `tools/` crate reimplements a capability a `compiler/` crate already provides. This is the concrete, per-crate enforcement of [Library First](#2-guiding-philosophy).

**Tooling MUST NOT independently implement language behavior.** This is a permanent, repository-wide rule, not a preference: a `tools/` crate found to contain its own copy of grammar-matching logic, type-compatibility logic, or formatting-rule logic — however small — is a defect to be corrected by routing that logic through the corresponding `compiler/` crate, per [§19](#19-repository-commandments)'s "no duplicated logic" commandment.

---

# 6. Standard Library Repository

The standard library, specified behaviorally by [STANDARD_LIBRARY.md](docs/STANDARD_LIBRARY.md), is implemented under `stdlib/` as its own crate family, separate from `compiler/`, since it is Kyne-facing runtime code a generated contract links against, not compiler logic.

```
stdlib/
  core/
  collections/
  math/
  convert/
  storage/
  auth/
  ledger/
  time/
  events/
  crypto/
  debug/
  test/
```

**Module layout.** Each subdirectory corresponds exactly to one [STANDARD_LIBRARY.md §2](docs/STANDARD_LIBRARY.md#2-library-organization) module, in the same layer grouping (Core: `core/`, `collections/`, `math/`, `convert/`; Blockchain: `storage/`, `auth/`, `ledger/`, `time/`, `events/`, `crypto/`; Utilities: `debug/`, `test/`) — this repository introduces no module STANDARD_LIBRARY.md did not already name, and no module here may depend on another in violation of [STANDARD_LIBRARY.md's own layering rule](docs/STANDARD_LIBRARY.md#overall-library-architecture).

**Compiler cooperation.** `kyne_codegen`, per [§4](#4-compiler-crates), recognizes a call to a standard library function and emits a corresponding call into this crate family's compiled Rust — this is the architectural resolution to a question [STANDARD_LIBRARY.md](docs/STANDARD_LIBRARY.md) deliberately leaves open (it specifies behavior, not implementation strategy, per its own front matter): standard library functions are realized as ordinary, real Rust functions in a real, versioned crate family, not as compiler-intrinsic special cases hand-coded into `kyne_codegen` one at a time, except where a module's own specified behavior requires compiler cooperation beyond an ordinary function call — `debug/`'s build-profile elision, per [STANDARD_LIBRARY.md §13](docs/STANDARD_LIBRARY.md#13-debug-module), and `test/`'s context-gating, per [STANDARD_LIBRARY.md §14](docs/STANDARD_LIBRARY.md#14-test-module), both require `kyne_optimizer` and `kyne_semantics` respectively to be aware of which crate a given call resolves into.

**Testing.** Every `stdlib/` module carries its own unit tests, per [§16](#16-testing-philosophy), and, per [COMPILER_ARCHITECTURE.md §19](docs/COMPILER_ARCHITECTURE.md#19-testing-strategy)'s end-to-end category, is additionally exercised by the same six canonical example contracts every other stage's tests already depend on, per [§7](#7-examples).

**Versioning.** The standard library is versioned alongside the compiler and [LANGUAGE_SPEC.md](docs/LANGUAGE_SPEC.md), per [STANDARD_LIBRARY.md §20](docs/STANDARD_LIBRARY.md#20-stability-guarantees) — `stdlib/` does not carry an independent version number a project could pin separately from its compiler version.

**Documentation.** Every `stdlib/` module's public functions carry Rust doc comments describing their implementation, distinct from, and secondary to, [STANDARD_LIBRARY.md](docs/STANDARD_LIBRARY.md)'s own behavioral specification — where the two would ever disagree, [STANDARD_LIBRARY.md](docs/STANDARD_LIBRARY.md) governs, per [Repository Scope](#repository-scope).

**Relationship to the compiler.** `stdlib/` crates are ordinary dependencies of every generated Kyne contract's own `Cargo.toml`, per [TOOLCHAIN.md §14](docs/TOOLCHAIN.md#14-build-artifacts)'s `build/rust/` output — they are not part of the `kyne` compiler binary itself, and a `stdlib/` change does not require recompiling `kyne_cli`.

**Relationship to constitutional specifications.** `stdlib/`'s implementation MUST conform exactly to [STANDARD_LIBRARY.md](docs/STANDARD_LIBRARY.md)'s behavioral specification for every function it provides — an implementation detail not dictated by that document (a specific hashing library dependency, a specific internal data structure) belongs in an ADR, per [§11](#11-architecture-decision-records), never as an undocumented deviation.

---

# 7. Examples

```
examples/
  canonical/
  tutorial/
  regression/
```

**Canonical examples.** `examples/canonical/` holds the six examples from [LANGUAGE_SPEC.md §16](docs/LANGUAGE_SPEC.md#16-examples) — Counter, Token, Escrow, Marketplace, Voting, Multisig Wallet — verbatim. These are not merely illustrative; per [COMPILER_ARCHITECTURE.md §19](docs/COMPILER_ARCHITECTURE.md#19-testing-strategy), they are the compiler's own golden-file corpus, and per [TOOLCHAIN.md §6](docs/TOOLCHAIN.md#6-project-templates), two of them are the literal content of the `hello` and `token` project templates. A change to any file in `examples/canonical/` is, definitionally, a change to [LANGUAGE_SPEC.md](docs/LANGUAGE_SPEC.md) itself and MUST go through [GOVERNANCE.md §25](docs/GOVERNANCE.md#25-constitutional-documents)'s constitutional process, never an ordinary pull request.

**Tutorial examples.** `examples/tutorial/` holds additional, smaller examples written to teach one concept at a time — a single-function contract demonstrating only `auth`, a single-function contract demonstrating only `match` — supporting [FOUNDATION.md](docs/FOUNDATION.md#success-metrics)'s "first contract within one hour" goal without inflating the canonical corpus with content that exists to teach rather than to specify.

**Regression examples.** `examples/regression/` holds one file per fixed compiler defect whose triggering pattern is not already covered by `canonical/` or `tutorial/` — the repository-wide instance of [ROADMAP.md §13](docs/ROADMAP.md#13-testing-roadmap)'s "every fixed defect gets a permanent regression test."

**Maintenance.** Every file in `examples/` MUST continue to compile successfully under the current compiler at all times — an example that stops compiling is treated as a build failure, not a documentation issue, and is fixed with the same urgency as a failing test, since [§16](#16-testing-philosophy) treats this directory as executable specification, not as static prose.

---

# 8. Tests

```
tests/
  integration/
  snapshot/
  regression/
  performance/
  fuzz/
```

Per-crate unit tests live inside each crate under `compiler/`, `stdlib/`, and `tools/`, per ordinary Rust convention, and are not duplicated here. `tests/` at the repository root holds only **cross-crate** testing assets:

| Directory | Purpose |
|---|---|
| `tests/integration/` | End-to-end tests exercising the full pipeline from `.kyn` source through a real Cargo/Soroban build, per [COMPILER_ARCHITECTURE.md §19](docs/COMPILER_ARCHITECTURE.md#19-testing-strategy)'s end-to-end category. |
| `tests/snapshot/` | Golden-file tests comparing generated Rust output against checked-in expectations, per [COMPILER_ARCHITECTURE.md §19](docs/COMPILER_ARCHITECTURE.md#19-testing-strategy). |
| `tests/regression/` | The test-side counterpart to `examples/regression/` — one test per fixed defect, asserting the specific failure no longer reproduces. |
| `tests/performance/` | Benchmarks tracking [ROADMAP.md §14](docs/ROADMAP.md#14-performance-roadmap)'s compilation-speed and memory-usage goals, introduced in Phase 2 per that roadmap section. |
| `tests/fuzz/` | Fuzz-testing harnesses for `kyne_lexer` and `kyne_parser`, per [ROADMAP.md §15](docs/ROADMAP.md#15-security-roadmap), introduced in Phase 1. |

**Testing philosophy.** Restated and expanded in [§16](#16-testing-philosophy).

---

# 9. Documentation

```
docs/
  FOUNDATION.md
  LANGUAGE_PRINCIPLES.md
  LANGUAGE_SPEC.md
  RUNTIME_MODEL.md
  COMPILER_ARCHITECTURE.md
  MEMORY_MODEL.md
  STANDARD_LIBRARY.md
  TOOLCHAIN.md
  GOVERNANCE.md
  ROADMAP.md
  adr/
  guides/
```

| Category | Location | Governed by |
|---|---|---|
| Constitutional documents | `docs/*.md`, the nine files this document is not one of. | [GOVERNANCE.md §25](docs/GOVERNANCE.md#25-constitutional-documents) — elevated review, KIP required. |
| Operational documents | `docs/ROADMAP.md`, this document (`ARCHITECTURE.md`, at the repository root, per [§3](#3-workspace-layout)). | [GOVERNANCE.md §17](docs/GOVERNANCE.md#17-documentation-governance) — ordinary review. |
| Generated documentation | Not checked into `docs/` at all — produced by `kyne_docgen` into a build output directory, per [TOOLCHAIN.md §14](docs/TOOLCHAIN.md#14-build-artifacts)'s `build/docs/` pattern, applied here to this repository's own `stdlib/` API surface. | Regenerated, never hand-edited. |
| Architecture documents | This document, plus `docs/adr/`'s individual records. | [§11](#11-architecture-decision-records). |
| Tutorials | `docs/guides/`. | Ordinary review, per [GOVERNANCE.md §17](docs/GOVERNANCE.md#17-documentation-governance). |
| Examples | `examples/`, per [§7](#7-examples), not `docs/` — kept separate since examples are executable and tested, while `docs/guides/` is prose. |

**Directory conventions.** `docs/` contains only Markdown content classified by the table above; it MUST NOT contain generated output, per [§3](#3-workspace-layout)'s source-owned-versus-generated distinction, and it MUST NOT contain example source code, which belongs in `examples/` specifically so that tooling scanning for compilable Kyne source does not need to also filter out documentation prose.

---

# 10. Dependency Rules

**Allowed** (the complete, correct chain, restated in full from [COMPILER_ARCHITECTURE.md §3](docs/COMPILER_ARCHITECTURE.md#3-compiler-pipeline) and [§20](docs/COMPILER_ARCHITECTURE.md#20-repository-architecture) — a superset of any illustrative partial chain, since omitting the Security Analyzer, Optimizer, or foundation crates from a dependency diagram would understate the real, locked pipeline):

```
kyne_lexer
    ↓
kyne_parser
    ↓
kyne_cst
    ↓
kyne_ast
    ↓
kyne_resolver
    ↓
kyne_types
    ↓
kyne_semantics ──→ kyne_security
    ↓
kyne_hir ──→ kyne_optimizer
    ↓
kyne_rir
    ↓
kyne_codegen
```

with `kyne_diagnostics` and `kyne_cache` available to every stage above as foundation crates (per [§4](#4-compiler-crates)'s table), and `kyne_driver` the sole crate permitted to depend on every stage.

**Forbidden**, restated and completed:

- A `compiler/` stage crate depending on a *later* stage crate (for example, `kyne_resolver` depending on `kyne_types`) — this would create a cycle the moment the later stage's own, already-legitimate dependency on the earlier one is considered.
- `kyne_formatter` depending on `kyne_codegen`, `kyne_hir`, `kyne_rir`, or any stage beyond `kyne_cst` — the formatter's entire justification for existing as a lightweight, fast tool depends on this boundary holding, per [§5](#5-tooling-crates).
- The Language Server (`kyne_lsp`) implementing its own parser, resolver, or type-checking logic instead of depending on `kyne_parser`/`kyne_resolver`/`kyne_types` — per [§5](#5-tooling-crates)'s "tooling MUST NOT independently implement language behavior."
- Any `tools/` crate depending on another `tools/` crate — per [§5](#5-tooling-crates), every tool is a sibling consumer of `compiler/`, never a consumer of another tool.
- Any `stdlib/` module depending on a `compiler/` crate — the standard library is Kyne-facing runtime code, not compiler logic, and a dependency in this direction would mean a generated contract's own dependency tree included compiler internals it has no reason to need.

**Why dependency direction matters.** A forward-only dependency graph is what makes [§16](#16-testing-philosophy)'s "every crate testable in isolation" claim actually true, what makes [§4](#4-compiler-crates)'s "one owner per crate" claim enforceable (an owner cannot be held accountable for a crate whose behavior depends on an unbounded, backward-reaching dependency), and what makes a future crate split or crate merge — per [§20](#20-future-growth) — a local change instead of a repository-wide one. Every rule in this section is a specific instance of [Forward-Only Dependencies](#2-guiding-philosophy), stated concretely enough that a build-system-level dependency check can enforce it automatically rather than relying on review discipline alone.

---

# 11. Architecture Decision Records

**Naming and numbering.** An ADR is named `ADR-NNN-short-title.md`, numbered sequentially starting from `001`, matching the format [GOVERNANCE.md §19](docs/GOVERNANCE.md#19-architecture-decision-records-adr) already establishes.

**Storage location.** `docs/adr/`, per [§9](#9-documentation)'s table.

**Review.** An ADR is proposed and accepted by the Maintainer(s) owning the affected crate or subsystem, per [GOVERNANCE.md §19](docs/GOVERNANCE.md#19-architecture-decision-records-adr) — it does not require the project-wide Core Maintainer consensus a KIP requires, since, by definition, an ADR does not change any constitutional document's specified behavior.

**Relationship to implementation.** An ADR is written at the point an implementation decision is made, not retroactively once a contributor asks why a crate is structured a certain way — [§6](#6-standard-library-repository)'s decision to realize the standard library as a real Rust crate family rather than as compiler intrinsics is exactly the kind of decision this document has already made and that a corresponding ADR should formally record.

**Relationship to GOVERNANCE.md.** This document does not redefine the ADR mechanism — [GOVERNANCE.md §19](docs/GOVERNANCE.md#19-architecture-decision-records-adr) and [§24](docs/GOVERNANCE.md#24-kips-vs-adrs) already fix what an ADR is and how it differs from a KIP. This section states only where, in this specific repository, that mechanism's output is stored and who, concretely, reviews it.

---

# 12. KIPs

This document distinguishes three kinds of repository change, each with a different required process:

| Change type | Example | Required process |
|---|---|---|
| Constitutional change | Adding a new type to [LANGUAGE_SPEC.md §6](docs/LANGUAGE_SPEC.md#6-types); changing [RUNTIME_MODEL.md](docs/RUNTIME_MODEL.md)'s failure model. | A KIP, per [GOVERNANCE.md §6](docs/GOVERNANCE.md#6-kip-lifecycle), through Accepted before any corresponding `compiler/` code is merged. |
| Implementation change | Choosing a specific parsing technique inside `kyne_parser`; restructuring `kyne_cache`'s internal storage format without changing its specified behavior. | An ADR, per [§11](#11-architecture-decision-records) — never a KIP. |
| Repository change | Splitting a crate in `compiler/` into two, per [§20](#20-future-growth); adding a new directory under `tools/`. | Ordinary pull request review, per [§18](#18-contributor-workflow) — an ADR is warranted only where the split or addition reflects a decision worth recording for future reference, per [§11](#11-architecture-decision-records)'s own judgment call. |

**Boundaries.** The test that separates the first row from the second and third, restated directly from [GOVERNANCE.md §25](docs/GOVERNANCE.md#25-constitutional-documents): does the change alter what a conforming implementation is required to do. If yes, it is constitutional, regardless of how it is initially framed by its author, and this document's own repository-organization authority does not extend to it.

---

# 13. Branch Strategy

**Main branch.** `main` is always in a buildable, passing-tests state, per [§16](#16-testing-philosophy) — no commit lands on `main` that fails the test suite `main` itself already requires passing.

**Feature branches.** Work in progress lives on a branch until it satisfies [§18](#18-contributor-workflow)'s pull request criteria; this document does not prescribe a specific naming convention beyond requiring that a feature branch's purpose be inferable from its associated pull request, not necessarily from its name alone.

**Release branches.** A release, per [ROADMAP.md §10](docs/ROADMAP.md#10-release-roadmap) and [GOVERNANCE.md §9](docs/GOVERNANCE.md#9-release-policy), is cut from `main` at the commit satisfying that release's gate, per [ROADMAP.md §25](docs/ROADMAP.md#25-release-gates) — a release branch exists to allow a patch, per [GOVERNANCE.md §9](docs/GOVERNANCE.md#9-release-policy)'s support-window policy, to be applied to an already-shipped major version without requiring `main`'s subsequent, unreleased work to also ship.

**Hotfix branches.** A [GOVERNANCE.md §11](docs/GOVERNANCE.md#11-security-governance) emergency security fix is branched directly from the affected release branch(es), fixed under embargo, and merged both there and forward into `main`, so the fix is never present in a release without also being present in every later one.

**Merge philosophy.** A branch merges into `main` only once it satisfies every applicable item in [§18](#18-contributor-workflow)'s pull request checklist — this document does not prescribe a specific Git workflow (rebase-and-merge versus squash-merge, for example) beyond this architectural requirement, leaving that specific mechanical choice to ordinary repository configuration outside this document's scope, per [Non-Goals](#non-goals).

---

# 14. Release Process

This document does not redefine release policy — [GOVERNANCE.md §9](docs/GOVERNANCE.md#9-release-policy) and [ROADMAP.md §10](docs/ROADMAP.md#10-release-roadmap) already fix versioning semantics, support windows, and the conceptual release sequence. This section states only the repository-level mechanics:

**Versioning.** Every crate under `compiler/` and `tools/` is versioned in lockstep with the `kyne` CLI's own release version, per [TOOLCHAIN.md §19](docs/TOOLCHAIN.md#19-version-management) — there is no independently versioned compiler crate a project could depend on at a different version than its installed `kyne` binary.

**Tags.** A release is marked by a single repository tag corresponding to the version satisfying [ROADMAP.md §25](docs/ROADMAP.md#25-release-gates)'s gate for that version.

**Release notes.** Every release's notes enumerate the KIPs Released in that version, per [GOVERNANCE.md §6](docs/GOVERNANCE.md#6-kip-lifecycle)'s lifecycle stage of the same name, plus any non-KIP-requiring fixes and improvements shipped alongside them.

**Documentation updates.** A release MUST NOT ship with `docs/` describing a different state than the release's actual behavior — per [§17](#17-documentation-philosophy), documentation currency is a release-gate item, not a follow-up task.

**Generated artifacts.** Compiled binaries and any generated documentation site content are produced by the release process itself and are never checked into the repository, per [§3](#3-workspace-layout)'s source-owned-versus-generated distinction.

**Relationship to ROADMAP.md.** [ROADMAP.md §10](docs/ROADMAP.md#10-release-roadmap) decides *what* each release contains and *when* it is ready; this section describes only the mechanical repository actions (tagging, crate version bumps, note generation) that turn that readiness into an actual, published release.

---

# 15. Coding Standards

**Naming.** Rust-level naming (crate names, module names, function names inside `compiler/`, `stdlib/`, and `tools/`) follows ordinary, idiomatic Rust convention — `snake_case` for functions and modules, `PascalCase` for types — consistent with [COMPILER_ARCHITECTURE.md §14.3](docs/COMPILER_ARCHITECTURE.md#143-reviewability)'s requirement that generated Rust, and by extension the compiler's own Rust, read as ordinary, idiomatic Rust a reviewer would recognize.

**Modules and files.** A Rust module's file boundary SHOULD correspond to a single, nameable responsibility within its crate, mirroring [Single Responsibility](#2-guiding-philosophy) one level down from the crate boundary.

**Comments.** Comments explain *why*, not *what* — restating [LANGUAGE_SPEC.md](docs/LANGUAGE_SPEC.md)'s own comment discipline (well-named identifiers already communicate what code does) applied to the compiler's own implementation, not only to Kyne source.

**Public APIs.** Every `pub` item in a `compiler/`, `stdlib/`, or `tools/` crate carries a doc comment stating its purpose and, where not obvious from its signature, its relationship to the constitutional document section it implements — a `pub fn` implementing a specific COMPILER_ARCHITECTURE.md rule SHOULD cite that section directly in its doc comment.

**Documentation.** Every crate carries a top-level `lib.rs` (or equivalent) doc comment summarizing its purpose, matching [§4](#4-compiler-crates) and [§5](#5-tooling-crates)'s "Purpose" column exactly, so the two never drift apart.

**Consistency.** Every crate under `compiler/` follows the same internal organization pattern (a small number of well-named modules, a `tests/` directory, a doc-commented public surface) so that familiarity with one crate transfers directly to the next.

**Language-specific formatting.** This section governs the repository's own Rust code. Formatting of `.kyn` source — in `examples/`, in generated test fixtures, anywhere — is governed exclusively by [TOOLCHAIN.md §10](docs/TOOLCHAIN.md#10-formatter), never by this document; this section makes no formatting rule for Kyne source of any kind.

---

# 16. Testing Philosophy

**Every crate is tested.** No crate under `compiler/`, `stdlib/`, or `tools/` merges its first implementation without an accompanying test suite, per [COMPILER_ARCHITECTURE.md §19](docs/COMPILER_ARCHITECTURE.md#19-testing-strategy)'s per-stage testing strategy, applied here as a repository-wide requirement with no exceptions carved out for a crate merely because it feels small.

**Regression tests prevent language regressions.** Every defect fixed anywhere in this repository gets a permanent test in `tests/regression/` or the relevant crate's own regression suite, per [§8](#8-tests) — a defect fixed without a regression test is a defect this repository has no protection against recurring.

**Snapshot tests preserve parser and codegen behavior.** `tests/snapshot/`, per [§8](#8-tests), is what makes [COMPILER_ARCHITECTURE.md §14.4](docs/COMPILER_ARCHITECTURE.md#144-deterministic-generation)'s byte-identical-output guarantee something this repository actually verifies on every change, not merely something the specification asserts.

**Performance tests protect compiler speed.** `tests/performance/`, introduced per [ROADMAP.md §14](docs/ROADMAP.md#14-performance-roadmap), exists so a change that regresses compile time is caught by CI rather than by a contributor noticing their own workflow has quietly gotten slower.

**Fuzz testing improves robustness.** `tests/fuzz/`, per [ROADMAP.md §15](docs/ROADMAP.md#15-security-roadmap), targets `kyne_lexer` and `kyne_parser` specifically, since both accept fully untrusted, adversarial-capable input the moment [TOOLCHAIN.md §17](docs/TOOLCHAIN.md#17-playground)'s Playground exists.

**Quality expectations.** A pull request touching `compiler/`, `stdlib/`, or `tools/` MUST include tests covering its change at the level [COMPILER_ARCHITECTURE.md §19](docs/COMPILER_ARCHITECTURE.md#19-testing-strategy) specifies for the crate it touches — per [§18](#18-contributor-workflow), this is a checklist item, not a suggestion, and a pull request lacking it is incomplete regardless of how correct its implementation otherwise appears.

---

# 17. Documentation Philosophy

**Every crate documented.** Per [§15](#15-coding-standards)'s "Documentation" rule — no exceptions for internal-only crates, since "internal" does not mean "unread by future contributors."

**Architecture documented.** This document, [COMPILER_ARCHITECTURE.md](docs/COMPILER_ARCHITECTURE.md), and `docs/adr/`'s growing record together constitute the full account of why the repository and the compiler are shaped the way they are.

**Examples maintained.** Per [§7](#7-examples)'s maintenance rule — every example is executable, tested specification, kept perpetually compiling.

**API documentation generated.** The standard library's Kyne-facing API surface is documented via `kyne_docgen`, per [§9](#9-documentation)'s "Generated documentation" row, never hand-maintained as a separate prose document that could drift from [STANDARD_LIBRARY.md](docs/STANDARD_LIBRARY.md)'s own specification.

**Implementation documentation kept current.** A crate's doc comments MUST describe its current behavior, not a past or aspirational one — [§27](#27-directory-rules) *(cross-referencing [ROADMAP.md §27](docs/ROADMAP.md#27-definition-of-done))* already requires this at the level of an individual milestone's completion criteria; this section restates it as a permanent, ongoing repository expectation rather than only a one-time completion gate.

**Documentation evolves with implementation.** Stated plainly, as its own rule: a pull request that changes a crate's public behavior without updating that crate's doc comments to match is an incomplete pull request, exactly as one missing tests would be, per [§16](#16-testing-philosophy).

---

# 18. Contributor Workflow

1. **Clone the repository.**
2. **Read documentation** — this document first, then the constitutional document(s) relevant to the intended contribution, then the specific crate's own doc comments.
3. **Create a branch**, per [§13](#13-branch-strategy).
4. **Implement the feature or fix**, respecting [§10](#10-dependency-rules)'s dependency boundaries and [§15](#15-coding-standards)'s coding standards.
5. **Add tests**, per [§16](#16-testing-philosophy), at the level appropriate to the crate touched.
6. **Update documentation**, per [§17](#17-documentation-philosophy) — doc comments, and, where the change is constitutional per [§12](#12-kips), the corresponding KIP process, not this workflow alone.
7. **Open a pull request**, referencing the KIP it implements where applicable, per [GOVERNANCE.md §6](docs/GOVERNANCE.md#6-kip-lifecycle)'s Implemented stage.
8. **Review**, per [GOVERNANCE.md §13](docs/GOVERNANCE.md#13-code-review-philosophy)'s correctness/documentation/testing/performance/security/maintainability/consistency criteria.
9. **Merge**, per [§13](#13-branch-strategy)'s merge philosophy, by the crate's owning Maintainer, per [§4](#4-compiler-crates)/[§5](#5-tooling-crates)'s ownership tables.

**Expected workflow.** Steps 1–2 are a one-time (or infrequent) orientation cost; steps 3–9 repeat for every contribution, small or large. A contribution that skips step 5 or step 6 is not eligible for step 9 regardless of the quality of step 4's implementation — per [GOVERNANCE.md §13](docs/GOVERNANCE.md#13-code-review-philosophy), a reviewer approving a change missing tests or documentation has performed an incomplete review, not a lenient one.

---

# 19. Repository Commandments

**Every crate earns its existence.** A crate is created only when [Single Responsibility](#2-guiding-philosophy) genuinely requires a new boundary, mirroring [LANGUAGE_SPEC.md's "every keyword must earn its place"](docs/LANGUAGE_SPEC.md#every-keyword-must-earn-its-place) applied to repository structure — a crate created "for organization" with no clear, singular responsibility is a crate this repository should not have created.

**No cyclic dependencies.** Restated permanently from [§10](#10-dependency-rules): the acyclic graph is enforced, not merely documented, and a proposed change that would introduce a cycle is rejected at review regardless of how small the cycle appears.

**No duplicated logic.** Restated permanently from [§5](#5-tooling-crates): if two crates independently implement the same rule, one of them is wrong by construction, even if both currently agree — the moment the underlying rule changes, only one of the two duplicated implementations will be updated, and the repository will silently disagree with itself.

**Documentation evolves with code.** Restated permanently from [§17](#17-documentation-philosophy).

**Tests are part of implementation.** Restated permanently from [§16](#16-testing-philosophy) — a change without tests is not a smaller version of a complete change, it is an incomplete change.

**Compiler libraries remain the single source of truth.** Restated permanently from [Library First](#2-guiding-philosophy) and [COMPILER_ARCHITECTURE.md §22](docs/COMPILER_ARCHITECTURE.md#22-library-first-compiler-architecture) — every tool, forever, is a consumer, never a second implementation.

**Performance never sacrifices correctness.** Restated permanently from [COMPILER_ARCHITECTURE.md's Principle 3](docs/COMPILER_ARCHITECTURE.md#principle-3--compiler-correctness-before-optimization) and [ROADMAP.md §14](docs/ROADMAP.md#14-performance-roadmap) — no repository change, at any layer, trades a correctness guarantee for a speed improvement.

**Security is everyone's responsibility.** Every contributor's pull request is expected to consider [GOVERNANCE.md §13](docs/GOVERNANCE.md#13-code-review-philosophy)'s security dimension, not only the contributor working in `kyne_security` — a defect in `kyne_lexer`'s error recovery is as much a security concern, per [ROADMAP.md §15](docs/ROADMAP.md#15-security-roadmap)'s fuzz-testing rationale, as a missing Security Analyzer check.

**Why these are permanent.** Every commandment above is what keeps this repository's actual, growing shape consistent with the principles in [§2](#2-guiding-philosophy) as it scales past what any single contributor can hold in their head at once — the same justification [GOVERNANCE.md §20](docs/GOVERNANCE.md#20-governance-invariants) gives for its own permanent invariants, applied here to the code, rather than the process, those invariants ultimately protect.

---

# 20. Future Growth

**Adding new crates.** A new crate is added when a genuinely new responsibility emerges that does not fit any existing crate's single responsibility, per [§19](#19-repository-commandments) — its dependency edges MUST be added to [§10](#10-dependency-rules) and [§21](#21-dependency-graph) in the same change that introduces it.

**Splitting crates.** A crate that has grown to hold more than one responsibility, per [Single Responsibility](#2-guiding-philosophy), is split — this is a repository-organization change, per [§12](#12-kips)'s table, not a constitutional one, provided the split preserves every crate's specified behavior exactly.

**Creating new tools.** A new `tools/` crate follows [§5](#5-tooling-crates)'s existing pattern exactly: it depends only on `compiler/` crates, never reimplements language behavior, and is exposed through `kyne_cli` as a subcommand unless it is a hosted service like the Playground.

**Expanding documentation.** `docs/guides/` grows without constraint, per [§9](#9-documentation) — informative documentation has no ceiling this document imposes.

**Workspace growth.** A new top-level directory beyond [§3](#3-workspace-layout)'s list requires the same justification a new crate does: a responsibility no existing directory already covers.

**Repository restructuring.** Any restructuring beyond ordinary crate addition or splitting — a change to `compiler/`'s own internal layout, for example — MUST preserve [§2](#2-guiding-philosophy)'s six principles in full; a restructuring proposal that would violate one of them (introducing a cycle to simplify a build step, for example) is not an acceptable restructuring regardless of its other merits.

**Preserving repository principles.** Every direction of growth named in this chapter is bounded by the same six principles [§2](#2-guiding-philosophy) states at the top of this document — this chapter grants no exception to any of them, only describes the shapes growth is expected to take while remaining inside them.

---

# 21. Dependency Graph

```
kyne_cli
    ↓
kyne_driver
    ↓
kyne_lexer
    ↓
kyne_parser
    ↓
kyne_cst
    ↓
kyne_ast
    ↓
kyne_resolver
    ↓
kyne_types
    ↓
kyne_semantics
    ↓
kyne_hir
    ↓
kyne_rir
    ↓
kyne_codegen
```

with `kyne_security` branching from `kyne_types`, `kyne_optimizer` branching from `kyne_hir`, and `kyne_diagnostics`/`kyne_cache` available as foundation crates to every node in this graph, exactly as detailed in [§10](#10-dependency-rules)'s fuller diagram — this chapter's diagram is the same graph presented top-down from `kyne_cli`, the entry point a new contributor actually runs, rather than bottom-up from `kyne_lexer`, the entry point the pipeline itself begins execution from.

**Dependency direction.** Every arrow in this graph points from a consumer to what it consumes — `kyne_cli` depends on `kyne_driver`, not the reverse, and `kyne_driver` depends on every stage crate beneath it, not the reverse. No arrow in this repository's real dependency graph ever points upward against this diagram.

**Why dependencies remain forward-only.** Restated once more, at the level of this specific, concrete diagram rather than the general principle in [§2](#2-guiding-philosophy) and [§10](#10-dependency-rules): a forward-only graph is what allows `kyne_lexer` to be built, tested, and understood on day one of Phase 1, per [ROADMAP.md §7](docs/ROADMAP.md#7-compiler-bootstrap-order), with zero knowledge of `kyne_codegen`, which will not exist until much later in that same phase — every stage's independence from the stages above it in this diagram is what makes [Roadmap Principle 3 (Vertical Slices)](docs/ROADMAP.md#roadmap-philosophy-principles) achievable as an actual engineering plan rather than only a nice-sounding intention.

---

# 22. Workspace Principles

**Every crate has one responsibility.** Restated permanently from [Single Responsibility](#2-guiding-philosophy) — the single most load-bearing principle in this document, since nearly every other rule here exists to protect it.

**Every crate has one owner.** Restated permanently from [Explicit Ownership](#2-guiding-philosophy) and enforced concretely by [§4](#4-compiler-crates)/[§5](#5-tooling-crates)'s ownership tables.

**No hidden dependencies.** A crate's `Cargo.toml` (or workspace-equivalent dependency declaration) MUST be the complete, accurate account of everything it depends on — no crate may reach another's internals through a file-system convention, an environment variable, or any channel [§10](#10-dependency-rules)'s explicit dependency graph does not already capture.

**Shared utilities belong in shared crates.** A helper needed by more than one crate is factored into a shared crate — `kyne_diagnostics` and `kyne_cache` are the two existing instances of this principle, per [§4](#4-compiler-crates) — rather than copied into each crate that needs it, which would immediately violate [§19](#19-repository-commandments)'s "no duplicated logic."

**No duplicate implementations.** Restated permanently, at the workspace level rather than only the tooling level [§5](#5-tooling-crates) states it at: any two crates found to implement the same rule independently are, by definition, in violation of this principle, regardless of which layer of the repository they live in.

**Compiler libraries remain authoritative.** Restated a final time, at the workspace-principle level: nothing in `tools/`, `stdlib/`, `examples/`, or `website/` is ever permitted to be the actual source of truth for what a Kyne program means — that authority belongs to `compiler/` alone, which itself answers only to the nine constitutional documents.

---

# 23. Directory Rules

| Directory | Contains | MUST NOT contain |
|---|---|---|
| `compiler/` | Compiler implementation only, per [§4](#4-compiler-crates). | Tooling logic, standard library implementation, documentation prose. |
| `tools/` | Tooling only, per [§5](#5-tooling-crates). | Any independent implementation of language behavior, per [§5](#5-tooling-crates)'s own rule. |
| `stdlib/` | Standard library implementation only, per [§6](#6-standard-library-repository). | Compiler logic; a `stdlib/` crate MUST NOT depend on any `compiler/` crate, per [§10](#10-dependency-rules). |
| `docs/` | Documentation only, per [§9](#9-documentation). | Generated output; example source code. |
| `examples/` | Example contracts only, per [§7](#7-examples). | Non-Kyne source code; documentation prose (belongs in `docs/guides/` instead). |
| `tests/` | Repository-wide, cross-crate testing assets only, per [§8](#8-tests). | Per-crate unit tests, which belong inside their own crate. |

**Why directory discipline matters.** A directory whose contents cannot be predicted from its name is a directory a new contributor cannot navigate without asking — the entire value of [§3](#3-workspace-layout)'s top-level layout depends on every directory actually containing only what this table says it contains, consistently, for as long as the repository exists. Directory discipline is [Documentation as Architecture](#2-guiding-philosophy) made literal: the repository's own file tree is the first piece of documentation any contributor reads, whether or not they open a single `.md` file first.

---

# Repository Scope

This document governs repository organization: where code lives, how crates depend on one another, how the repository is tested and documented, and how a contributor moves through it. It does **not** redefine:

- **Language syntax** — [LANGUAGE_SPEC.md](docs/LANGUAGE_SPEC.md).
- **Compiler architecture** — [COMPILER_ARCHITECTURE.md](docs/COMPILER_ARCHITECTURE.md).
- **Runtime semantics** — [RUNTIME_MODEL.md](docs/RUNTIME_MODEL.md).
- **Memory model** — [MEMORY_MODEL.md](docs/MEMORY_MODEL.md).
- **Toolchain behavior** — [TOOLCHAIN.md](docs/TOOLCHAIN.md).
- **Governance** — [GOVERNANCE.md](docs/GOVERNANCE.md).

Every one of these remains defined, exclusively and permanently, by its own constitutional document. Where this document names a crate that implements one of them, it is naming a location, never redefining the behavior located there.

---

# Non-Goals

This document MUST NOT define:

- **Compiler algorithms** — an implementation detail for the relevant crate's own code and, where worth recording, an ADR per [§11](#11-architecture-decision-records).
- **Language behavior** — [LANGUAGE_SPEC.md](docs/LANGUAGE_SPEC.md) and [RUNTIME_MODEL.md](docs/RUNTIME_MODEL.md).
- **Runtime implementation** — an implementation detail, per the same rule.
- **Memory algorithms** — [MEMORY_MODEL.md](docs/MEMORY_MODEL.md) fixes observable behavior; this document does not fix how `kyne_rir`'s planners achieve it.
- **Standard library APIs** — [STANDARD_LIBRARY.md](docs/STANDARD_LIBRARY.md).
- **Package manager architecture** — reserved for a future `PACKAGE_MANAGER.md`, per [TOOLCHAIN.md §22](docs/TOOLCHAIN.md#22-future-package-manager) and [ROADMAP.md](docs/ROADMAP.md#12-documentation-roadmap).
- **Compiler optimizations** — [COMPILER_ARCHITECTURE.md §13](docs/COMPILER_ARCHITECTURE.md#13-optimization).
- **Programming style guides beyond repository organization** — [§15](#15-coding-standards) covers only naming, module boundaries, comments, and documentation conventions relevant to navigating this repository; it is not a general Rust style guide.

---

# Cross References

This document is informed by, and MUST remain consistent with:

- [FOUNDATION.md](docs/FOUNDATION.md), [LANGUAGE_PRINCIPLES.md](docs/LANGUAGE_PRINCIPLES.md), [LANGUAGE_SPEC.md](docs/LANGUAGE_SPEC.md), [RUNTIME_MODEL.md](docs/RUNTIME_MODEL.md), [COMPILER_ARCHITECTURE.md](docs/COMPILER_ARCHITECTURE.md), [MEMORY_MODEL.md](docs/MEMORY_MODEL.md), [STANDARD_LIBRARY.md](docs/STANDARD_LIBRARY.md), [TOOLCHAIN.md](docs/TOOLCHAIN.md), [GOVERNANCE.md](docs/GOVERNANCE.md) — the nine constitutional documents this document implements the repository organization for, and never contradicts.
- [ROADMAP.md](docs/ROADMAP.md) — the companion operational document sequencing *when* this document's crates and directories come into being.

[`CONTRIBUTING.md`](CONTRIBUTING.md), at the repository root, now exists — see that document directly for contributor workflow guidance. The following documents remain anticipated but not yet written, and this document's [Non-Goals](#non-goals) section reserves their scope explicitly: `PACKAGE_MANAGER.md`, `ECOSYSTEM.md`, and `RELEASE_PROCESS.md`. Until each exists, this document does not speculate on their contents beyond the scope boundary already stated above.

---

# Closing

**ARCHITECTURE.md is an operational document, not a constitutional document.** It evolves with the implementation and the repository as Kyne grows — a crate split, a new tool, a restructured `tests/` directory are all changes this document expects to make, through ordinary review, for as long as the project exists. What does not change is its relationship to the nine constitutional documents: this document explains where and how their already-fixed behavior is built, and it earns no authority of its own to alter what that behavior is. A future contributor who finds this document out of date with the actual repository should correct it directly; a future contributor who finds it in tension with a constitutional document should trust the constitutional document and open a pull request fixing this one.
