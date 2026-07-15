# TOOLCHAIN.md

# The Kyne Toolchain & Developer Experience Specification

**Phase:** 0 — Foundations
**Milestone:** 8 — Toolchain & Developer Experience Specification
**Version:** 0.1 (Draft, targeting Kyne v1.0)
**Status:** Normative — this document is binding on every future tooling contributor.

This document defines how developers interact with Kyne, from creating a project through building, testing, formatting, documenting, and deploying it. It describes **developer-facing behavior**, not implementation: every command, workflow, and guarantee stated here is a contract a conforming toolchain implementation MUST uphold, not a prescription for how that implementation is written.

This document implements — and MUST NOT contradict — [FOUNDATION.md](./FOUNDATION.md), [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md), [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md), [RUNTIME_MODEL.md](./RUNTIME_MODEL.md), [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md), [MEMORY_MODEL.md](./MEMORY_MODEL.md), and [STANDARD_LIBRARY.md](./STANDARD_LIBRARY.md), all of which are locked, constitutional documents with respect to this one. Every tool this document describes is, without exception, a composition of the library crates [COMPILER_ARCHITECTURE.md §22](./COMPILER_ARCHITECTURE.md#22-library-first-compiler-architecture) already names — this document introduces no new compiler capability, only the developer-facing surface built on top of the capability that already exists. Where this document appears to conflict with any of those seven, this document is wrong and MUST be corrected by KIP.

This specification uses **RFC 2119** keywords normatively: **MUST**, **MUST NOT**, **SHALL**, **SHOULD**, **SHOULD NOT**, and **MAY** carry their standard meanings.

**Exit code convention**, used throughout [§9](#9-cli-commands) and [§26](#26-developer-assistance-commands): every `kyne` subcommand MUST exit `0` on success, `1` on a failure the command itself detected (a compile error, a failing test, a formatting violation under `--check`), and `2` on a usage error (invalid arguments or flags). No command in this specification uses any other exit code.

---

# 1. Toolchain Philosophy

**Purpose.** The Kyne toolchain exists to make the complete lifecycle of a Kyne contract — creation, checking, testing, formatting, documentation, and building — a single, coherent, fast experience, so that a developer's attention stays on their contract's logic rather than on assembling and reconciling a set of independent tools.

**Goals.** Every goal in this document traces back to the six principles below: one tool, convention over configuration, fast feedback, one command per job, compiler-first reuse, and deterministic builds. A proposed toolchain feature that cannot be justified against at least one of these, per the same discipline [LANGUAGE_SPEC.md's Design Philosophy](./LANGUAGE_SPEC.md#every-keyword-must-earn-its-place) already applies to language keywords, does not belong here.

**Relationship to the compiler.** The toolchain is not a separate engineering effort from the compiler — it is the compiler's public face. Every capability described in this document is realized by composing the library crates [COMPILER_ARCHITECTURE.md §22](./COMPILER_ARCHITECTURE.md#22-library-first-compiler-architecture) defines; this document assumes that architecture as a fixed foundation and builds the developer experience directly on top of it.

**Relationship to the language.** The toolchain enforces [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md)'s canonical formatting and canonical member ordering ([§10](#10-formatter), [§26](#26-developer-assistance-commands)) as a matter of course, not as an optional add-on — a Kyne developer's daily experience of "what does correct Kyne code look like" is inseparable from what `kyne fmt` produces.

**Relationship to the runtime.** [§13](#13-test-runner)'s test runner and [§9](#9-cli-commands)'s `kyne run` both execute contract code against the exact failure and rollback semantics [RUNTIME_MODEL.md](./RUNTIME_MODEL.md) defines — a test's pass/fail outcome is not a toolchain-invented concept, it is [RUNTIME_MODEL.md §9](./RUNTIME_MODEL.md#9-failure-model)'s existing Success/Recoverable-Error/Runtime-Failure taxonomy, read directly.

**Relationship to the standard library.** [§13](#13-test-runner)'s mock execution context and [§17](#17-playground)'s sandboxed execution are both built directly on [STANDARD_LIBRARY.md §14](./STANDARD_LIBRARY.md#14-test-module)'s `test/` module primitives — the toolchain introduces no second, competing mocking mechanism.

**Relationship to future package management.** [§22](#22-future-package-manager) reserves command names and conceptual shape for a future package manager without designing one, consistent with [LANGUAGE_SPEC.md's Future Ecosystem section](./LANGUAGE_SPEC.md#future-ecosystem) and [STANDARD_LIBRARY.md's own deferral](./STANDARD_LIBRARY.md#non-goals) of the same concern.

**Why Kyne provides an integrated developer experience.** A language whose compiler, formatter, linter, documentation generator, and test runner are independently maintained tools inevitably drifts: the formatter's understanding of valid syntax lags the compiler's, the linter disagrees with the compiler about a diagnostic's wording, the documentation generator handles a language feature the formatter does not yet know about. Kyne avoids this category of drift structurally, per [COMPILER_ARCHITECTURE.md §22](./COMPILER_ARCHITECTURE.md#22-library-first-compiler-architecture): every tool is built from the same libraries the compiler itself is built from, so there is exactly one implementation of "what is valid Kyne" for every tool to agree with, by construction.

---

# 2. Toolchain Components

| Component | Responsibility | Built from |
|---|---|---|
| Compiler | Turns `.kyn` source into deployable WASM, per [COMPILER_ARCHITECTURE.md §3](./COMPILER_ARCHITECTURE.md#3-compiler-pipeline). | `kyne_lexer` through `kyne_codegen`, orchestrated by `kyne_driver`. |
| CLI | The single `kyne` executable; the entry point for every other component. | `kyne_driver`, plus thin per-command wiring. |
| Formatter | Produces Kyne's one canonical source rendering. | `kyne_lexer`, `kyne_parser`, `kyne_cst`. |
| Language Server | Provides editor intelligence over LSP. | `kyne_cst`, `kyne_ast`, `kyne_resolver`, `kyne_types`, `kyne_diagnostics`. |
| Documentation Generator | Produces API documentation from source and doc comments. | `kyne_cst`, `kyne_ast`. |
| Test Runner | Executes a project's `tests/` suite against the runtime failure model. | `kyne_driver` (full pipeline) plus the mocked execution context from [STANDARD_LIBRARY.md §14](./STANDARD_LIBRARY.md#14-test-module). |
| Project Generator | Scaffolds a new project from a template. | Static templates plus `Kyne.toml` generation logic; no compiler libraries required. |
| Build System | Orchestrates the full pipeline through WASM, including the external Cargo and Soroban toolchains. | `kyne_driver`, `kyne_cache`, plus the external `cargo` and Soroban build tools. |

**Interaction.** The CLI is the sole entry point a developer invokes directly; every other component in this table is reached exclusively through a `kyne` subcommand ([§9](#9-cli-commands)), never through a separate executable, per [§3](#3-cli-philosophy). The Language Server and Playground are the two components that run outside a direct CLI invocation (as a long-lived editor-integrated process and a browser-hosted service, respectively), but both are still built from the identical libraries, per [Toolchain Principle 5](#5-compiler-first).

---

# 3. CLI Philosophy

Kyne exposes exactly one executable:

```text
kyne
```

There is no separate formatter binary, no separate linter binary, no separate documentation generator binary, and no separate test runner binary — every capability in [§2](#2-toolchain-components) is a subcommand of `kyne`.

**Simplicity.** A developer setting up Kyne installs one thing and never needs to reason about which of several tools a given task requires — every task is `kyne <verb>`.

**Discoverability.** `kyne --help` (and `kyne <command> --help`) enumerates the complete surface of what Kyne can do in one place. A capability that lived in a separate binary would not be discoverable this way, and would require a developer to already know it existed and what it was called before they could find it.

**Maintainability.** A single executable with a shared internal architecture ([§2](#2-toolchain-components)) has one build, one release process, and one version number — there is no possibility of a formatter binary and a compiler binary drifting to incompatible versions on a developer's machine, since they are, in fact, the same binary.

**Consistency.** Every subcommand shares the same flag conventions, the same diagnostic rendering (via `kyne_diagnostics`, per [COMPILER_ARCHITECTURE.md §15](./COMPILER_ARCHITECTURE.md#15-diagnostics-engine)), and the same exit code convention (per this document's front matter) — a developer who has learned one command's interface has learned the shape of every command's interface.

---

# 4. Project Layout

```
project/
  Kyne.toml
  src/
    contract.kyn
  tests/
  docs/
  build/
    wasm/
    rust/
    docs/
  .kyn/
    cache/
```

| Path | Owner | Purpose |
|---|---|---|
| `Kyne.toml` | Developer | The project manifest, per [§5](#5-kynetoml). |
| `src/` | Developer | The project's `.kyn` source, per [LANGUAGE_SPEC.md §12](./LANGUAGE_SPEC.md#12-modules) — exactly one file in this tree contains the project's single `ContractDecl` if this is a contract project. |
| `tests/` | Developer | Test source, per [§13](#13-test-runner) — compiled separately from `src/` and never included in a deployable build. |
| `docs/` | Developer | Hand-written supplementary documentation (guides, design notes) — distinct from `kyne doc`'s generated output. |
| `build/` | Tool-generated | Every artifact `kyne build` and `kyne doc` produce, per [§14](#14-build-artifacts). Safe to delete at any time; regenerated in full by the next build. |
| `.kyn/cache/` | Tool-generated | The compiler cache defined in [COMPILER_ARCHITECTURE.md §17](./COMPILER_ARCHITECTURE.md#17-compiler-cache); this document introduces no changes to its layout or invalidation rules. |

**Generated versus user-owned.** `Kyne.toml`, `src/`, `tests/`, and `docs/` are hand-authored and belong in version control. `build/` and `.kyn/` are tool-owned, MUST NOT be hand-edited, and SHOULD be excluded from version control by convention (a generated project, per [§6](#6-project-templates), includes an appropriate ignore file by default).

**Why `docs/` and `build/docs/` are distinct.** Top-level `docs/` holds content a developer writes themselves — a README-adjacent guide, an architecture note specific to the project. `build/docs/` holds `kyne doc`'s generated API reference, per [§12](#12-documentation-generator). Collapsing the two would mean a `kyne doc` invocation could silently overwrite hand-written content, which [§21](#21-toolchain-invariants) rules out categorically.

---

# 5. Kyne.toml

The project manifest, using TOML.

```toml
[package]
name = "my-token"
version = "0.1.0"
edition = "v1"
authors = ["Jane Developer <jane@example.com>"]
description = "A fungible token contract."

[build]
# Reserved for build-relevant configuration, per §7.

[dependencies]
# Reserved for a future package manager, per §22.
```

| Field | Purpose |
|---|---|
| `package.name` | The project's name; SHOULD match the deployable contract's conceptual identity. |
| `package.version` | The project's own version, independent of the Kyne compiler's version. |
| `package.edition` | The language edition this project targets, per [§19](#19-version-management) — `"v1"` is the only edition Kyne v1 defines. |
| `package.authors` | Optional attribution metadata. |
| `package.description` | A one-line summary, surfaced by [§12](#12-documentation-generator)'s generated documentation. |
| `[build]` | Reserved for build-relevant configuration — per [Toolchain Principle 2](#2-convention-over-configuration), this section SHOULD remain empty for the overwhelming majority of projects, since sensible defaults cover ordinary use. |
| `[dependencies]` | Reserved, unused in v1, per [§22](#22-future-package-manager). |

**Why TOML.** TOML is human-readable, supports comments (unlike JSON), has an unambiguous, well-specified grammar (unlike YAML, whose implicit-typing and indentation-sensitivity rules are a well-documented source of surprising parses), and is already the manifest format of the Rust ecosystem Kyne's generated code depends on — a Kyne developer who ever opens the generated crate's own `Cargo.toml` ([§14](#14-build-artifacts)) encounters a familiar format, not a second one to learn.

---

# 6. Project Templates

```text
kyne new hello
kyne new token
kyne new nft
kyne new oracle
```

Each template provides a complete, buildable `Kyne.toml` and `src/` tree demonstrating one common contract shape. `kyne new hello` and `kyne new token` MUST use the **Counter** and **Token** contracts from [LANGUAGE_SPEC.md §16](./LANGUAGE_SPEC.md#16-examples) verbatim as their generated content — these are already the language's own normative, canonical examples, and generating a second, independently written "hello world" would create two contracts claiming to demonstrate the same idioms, in tension with [One Obvious Way](./LANGUAGE_SPEC.md#every-keyword-must-earn-its-place). `nft` and `oracle` are additional templates whose specific contract content is not fixed by this document — their design is an implementation detail of the templates themselves, not a language or runtime concern this specification governs.

**Extensibility.** Until a package manager exists (per [§22](#22-future-package-manager)), the set of official templates is fixed and shipped with the compiler itself. A future package manager MAY extend `kyne new` to accept a community-provided template source (for example, `kyne new my-project --template <registry-reference>`); this document reserves the flag shape conceptually but does not design the mechanism.

---

# 7. Build System

```
Source
    ↓
Check
    ↓
Compile
    ↓
Optimize
    ↓
Generate Rust
    ↓
Cargo Build
    ↓
Soroban Build
    ↓
WASM
```

This is a developer-facing, coarser-grained view of the pipeline [COMPILER_ARCHITECTURE.md §3](./COMPILER_ARCHITECTURE.md#3-compiler-pipeline) already fixes in full detail — this document does not redefine that pipeline, only names the groupings a developer reasons about day to day:

| Build System stage | Corresponds to (COMPILER_ARCHITECTURE.md §3) |
|---|---|
| Source | `.kyn` Source |
| Check | Lexer → Parser → CST → AST → Name Resolution → Type Checker → Semantic Analyzer → Security Analyzer |
| Compile | HIR lowering |
| Optimize | Optimization Passes |
| Generate Rust | RIR lowering → Rust Code Generator |
| Cargo Build | Cargo Build |
| Soroban Build | Soroban Build |
| WASM | WASM |

**Ownership.** Every stage through Generate Rust is owned by the Kyne compiler itself; Cargo Build and Soroban Build are ordinary, unmodified invocations of the external Rust and Soroban toolchains, per [RUNTIME_MODEL.md §3](./RUNTIME_MODEL.md#3-project-lifecycle) and [COMPILER_ARCHITECTURE.md §3](./COMPILER_ARCHITECTURE.md#3-compiler-pipeline) — the build system's job at that point is orchestration (invoking the right external commands with the right arguments), not compilation.

**Reproducibility.** Every stage in this table is deterministic for a fixed compiler version and fixed external toolchain versions, per [§20](#20-reproducible-builds).

---

# 8. Incremental Builds

This document does not redefine incremental compilation — [COMPILER_ARCHITECTURE.md §16](./COMPILER_ARCHITECTURE.md#16-incremental-compilation) and [§17](./COMPILER_ARCHITECTURE.md#17-compiler-cache) already fix the dependency graph (file granularity), change detection (content hashing), partial recompilation, and cache invalidation rules in full. This chapter states only the toolchain-level consequence: **`kyne build` and `kyne check` MUST consult and populate `.kyn/cache/` automatically and transparently, with no separate command required to opt in.** There is no `kyne build --incremental` flag — incrementality is not an opt-in performance mode, it is the only way `kyne build` and `kyne check` operate, per [Toolchain Principle 3 (Fast Feedback)](#3-fast-feedback).

**Performance philosophy.** A developer iterating on one file in a multi-file project SHOULD experience a build time proportional to that one file's own compilation cost plus whatever transitively depends on it, per [COMPILER_ARCHITECTURE.md §16](./COMPILER_ARCHITECTURE.md#16-incremental-compilation) — never proportional to the whole project's size, regardless of project size.

---

# 9. CLI Commands

Every command below follows the exit-code convention stated in this document's front matter. "Interaction with the compiler" names the specific compiler stages or libraries, per [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md), each command invokes.

### `kyne new <name> [--template <template>]`

| Field | Specification |
|---|---|
| Purpose | Create a new project in a new directory named `<name>`. |
| Arguments | `<name>` — required, the new directory and default package name. `--template` — optional, one of the templates in [§6](#6-project-templates); defaults to `hello`. |
| Behavior | Creates `<name>/` with the full [Project Layout](#4-project-layout) skeleton for the chosen template. |
| Exit codes | `0` on success; `2` if `<name>` already exists as a non-empty directory or `--template` names an unknown template. |
| Examples | `kyne new my-token --template token` |
| Interaction with the compiler | None — pure scaffolding, no compilation occurs. |

### `kyne init [--template <template>]`

| Field | Specification |
|---|---|
| Purpose | Initialize a project in the current directory, in place. |
| Arguments | `--template` — as above; defaults to `hello`. |
| Behavior | Identical to `kyne new`, targeting the current directory instead of a new one. |
| Exit codes | `0` on success; `2` if the current directory already contains a `Kyne.toml`. |
| Examples | `kyne init --template token` |
| Interaction with the compiler | None. |

### `kyne build [--release]`

| Field | Specification |
|---|---|
| Purpose | Run the full [Build System](#7-build-system) pipeline through WASM. |
| Arguments | `--release` — enables Optimization Passes ([COMPILER_ARCHITECTURE.md §13](./COMPILER_ARCHITECTURE.md#13-optimization)) and elides every [`debug.*`](./STANDARD_LIBRARY.md#13-debug-module) call. Without it, a development build skips optimization for faster turnaround and keeps `debug.*` calls live. |
| Behavior | Produces `build/wasm/`, `build/rust/`, per [§14](#14-build-artifacts). |
| Exit codes | `0` on success; `1` if any pipeline stage reports a Compiler Error. |
| Examples | `kyne build --release` |
| Interaction with the compiler | The entire pipeline, `kyne_lexer` through `kyne_codegen`, plus the external Cargo and Soroban toolchains. |

### `kyne check`

| Field | Specification |
|---|---|
| Purpose | Run the pipeline through the Security Analyzer stage only, reporting every diagnostic without generating Rust or building WASM. |
| Arguments | None. |
| Behavior | Identical correctness and security feedback to `kyne build`, at a fraction of the cost, since HIR lowering, optimization, code generation, and the external toolchains are never invoked. |
| Exit codes | `0` if no Compiler Error was found (Security Analyzer Warnings/Hints do not affect exit code, per [COMPILER_ARCHITECTURE.md §11.2](./COMPILER_ARCHITECTURE.md#112-severity-levels)); `1` otherwise. |
| Examples | `kyne check` |
| Interaction with the compiler | `kyne_lexer` through `kyne_security`. |

### `kyne run <function> [--arg <name>=<value>]...`

| Field | Specification |
|---|---|
| Purpose | Invoke a single `public fn` of the project's contract directly, against a local mock execution context, for ad hoc manual exploration. |
| Arguments | `<function>` — the function name. `--arg name=value` — one per parameter, parsed per [STANDARD_LIBRARY.md §12](./STANDARD_LIBRARY.md#12-conversion-module)'s conversion rules. |
| Behavior | Builds the project (per `kyne build`, development profile), then executes `<function>` against a mock execution context seeded with sensible defaults ([STANDARD_LIBRARY.md §14](./STANDARD_LIBRARY.md#14-test-module)'s `test.mock_ledger`/`test.mock_auth` primitives, applied automatically rather than requiring a `tests/` file), printing the function's return value or the failure that occurred. |
| Exit codes | `0` if the invocation succeeded, per [RUNTIME_MODEL.md §9.1](./RUNTIME_MODEL.md#91-success); `1` if it produced a Recoverable Error or Runtime Failure, per [RUNTIME_MODEL.md §9.2](./RUNTIME_MODEL.md#92-recoverable-error)/[§9.4](./RUNTIME_MODEL.md#94-runtime-failure). |
| Examples | `kyne run transfer --arg from=GABC... --arg to=GXYZ... --arg amount=100` |
| Interaction with the compiler | The full build pipeline, plus a mock-context execution harness sharing its mechanism with [§13](#13-test-runner)'s test runner. |
| Distinction from `kyne test` | `kyne run` is for one-off, ad hoc exploration during development; it is not a substitute for a project's systematic `tests/` suite, and its invocations are not recorded or repeatable as part of a project's test coverage. |

### `kyne test [pattern]`

| Field | Specification |
|---|---|
| Purpose | Compile and run every test case in `tests/`. |
| Arguments | `pattern` — optional; if given, only test functions whose name contains `pattern` are run. |
| Behavior | Per [§13](#13-test-runner): every `public fn` in every `.kyn` file under `tests/` is a test case. Each is executed independently, against a fresh mock execution context. A test passes if its invocation resolves to Success ([RUNTIME_MODEL.md §9.1](./RUNTIME_MODEL.md#91-success)); it fails otherwise. |
| Exit codes | `0` if every selected test passed; `1` if any failed. |
| Examples | `kyne test transfer` |
| Interaction with the compiler | The full build pipeline for both `src/` and `tests/`, plus the mock-context execution harness. |

### `kyne fmt [--check]`

| Field | Specification |
|---|---|
| Purpose | Rewrite every `.kyn` file in the project to Kyne's canonical formatting, per [LANGUAGE_SPEC.md §13](./LANGUAGE_SPEC.md#13-formatting-rules). |
| Arguments | `--check` — report whether any file would be reformatted, without writing changes. |
| Behavior | Deterministic, idempotent, and unconfigurable, per [§10](#10-formatter). |
| Exit codes | `0` if no changes were needed (or, without `--check`, once formatting completes); `1` under `--check` if any file would be reformatted. |
| Examples | `kyne fmt --check` (suitable for CI). |
| Interaction with the compiler | `kyne_lexer`, `kyne_parser`, `kyne_cst`. |

### `kyne lint`

| Field | Specification |
|---|---|
| Purpose | Surface Security Analyzer findings specifically. |
| Arguments | None. |
| Behavior | Performs **exactly the same analysis as `kyne check`** — the Security Analyzer already runs as a default stage of every `kyne check`/`kyne build` invocation, per [COMPILER_ARCHITECTURE.md §11](./COMPILER_ARCHITECTURE.md#11-security-analyzer). `kyne lint` introduces no new analysis pass; it exists solely as a discoverability convenience, filtering its output to `KS`-prefixed findings for a developer specifically looking for them. This is a direct consequence of [Toolchain Principle 1](#1-one-tool)'s "no separate linter executable": there is no linter distinct from the compiler's own Security Analyzer for this command to invoke. |
| Exit codes | `0` unconditionally, since `KS`-prefixed findings never affect exit code, per [COMPILER_ARCHITECTURE.md §11.2](./COMPILER_ARCHITECTURE.md#112-severity-levels) — a nonzero exit here would contradict that severity model. |
| Examples | `kyne lint` |
| Interaction with the compiler | Identical to `kyne check`. |

### `kyne doc [--open]`

| Field | Specification |
|---|---|
| Purpose | Generate API documentation for the project. |
| Arguments | `--open` — open the generated documentation in the developer's default browser after generation. |
| Behavior | Per [§12](#12-documentation-generator); writes to `build/docs/`. |
| Exit codes | `0` on success; `1` if generation fails (for example, due to a project that does not pass `kyne check`). |
| Examples | `kyne doc --open` |
| Interaction with the compiler | `kyne_cst`, `kyne_ast`. |

### `kyne clean`

| Field | Specification |
|---|---|
| Purpose | Remove every generated artifact. |
| Arguments | None. |
| Behavior | Deletes `build/` and `.kyn/cache/` in full. Per [COMPILER_ARCHITECTURE.md §17.3](./COMPILER_ARCHITECTURE.md#173-cleanup), this is always safe: both directories are strictly derived from `Kyne.toml` and `src/`/`tests/`, and deleting either can only cost time on the next build, never correctness. |
| Exit codes | `0` unconditionally, unless the deletion itself fails for an operating-system-level reason (permissions), in which case `1`. |
| Examples | `kyne clean` |
| Interaction with the compiler | None directly — this is a filesystem operation, not a compilation. |

### `kyne version`

| Field | Specification |
|---|---|
| Purpose | Report version information. |
| Arguments | None. |
| Behavior | Prints the `kyne` CLI version, the compiler version, and the current project's `Kyne.toml` edition, if run inside a project. |
| Exit codes | `0` unconditionally. |
| Examples | `kyne version` |
| Interaction with the compiler | None beyond reading its own build metadata. |

### `kyne doctor`

Specified fully in [§26](#26-developer-assistance-commands).

### `kyne explain <code>`

Specified fully in [§26](#26-developer-assistance-commands).

### `kyne fix`

Specified fully in [§26](#26-developer-assistance-commands).

---

# 10. Formatter

`kyne fmt` produces Kyne's **one** canonical rendering of any syntactically valid project, per [LANGUAGE_SPEC.md §13](./LANGUAGE_SPEC.md#13-formatting-rules) — this document does not restate those formatting rules, only the toolchain guarantees around applying them.

**No configuration.** `kyne fmt` accepts no style-affecting flags and reads no formatter-specific configuration file. There is no `.kyne-fmt.toml`, no per-project override of indentation width, brace style, or line length. Every Kyne project, everywhere, formats identically.

**Single formatting style.** `kyne fmt` is idempotent: formatting already-canonical source produces byte-identical output. This is a direct consequence of the formatting rules themselves being a total, deterministic function of the CST, per [COMPILER_ARCHITECTURE.md §6](./COMPILER_ARCHITECTURE.md#6-concrete-syntax-tree).

**Relationship to LANGUAGE_SPEC.md.** `kyne fmt` is the sole, authoritative implementation of [LANGUAGE_SPEC.md §13](./LANGUAGE_SPEC.md#13-formatting-rules) — there is no second, competing description of "correctly formatted Kyne" anywhere in the toolchain; the formatter's output *is* the definition, operationally, of what that section means.

**Why formatting is intentionally opinionated.** A configurable formatter reintroduces exactly the stylistic bikeshedding [Consistency Above Preference](./LANGUAGE_SPEC.md#consistency-above-preference) exists to eliminate — every configuration option is a decision some team, somewhere, will spend time debating, and a difference some future reader will need to notice and adjust to when moving between projects. Kyne removes the option entirely rather than defaulting it, because a default that can be overridden is not actually a guarantee.

---

# 11. Linter

Kyne has no linter distinct from the compiler. What a conventional toolchain would call "the linter" is, in Kyne, the Semantic Analyzer and Security Analyzer stages already fully specified in [COMPILER_ARCHITECTURE.md §10](./COMPILER_ARCHITECTURE.md#10-semantic-analysis) and [§11](./COMPILER_ARCHITECTURE.md#11-security-analyzer). This chapter states only the toolchain-level framing:

**Relationship to compiler diagnostics.** `KY`-prefixed diagnostics (Semantic Analyzer, Type Checker, Parser) are always Compiler Errors — blocking, certain, never heuristic — per [COMPILER_ARCHITECTURE.md §15.1](./COMPILER_ARCHITECTURE.md#151-diagnostic-code-namespace).

**Relationship to the Security Analyzer.** `KS`-prefixed diagnostics are always non-blocking Warnings or Hints, per [COMPILER_ARCHITECTURE.md §11.2](./COMPILER_ARCHITECTURE.md#112-severity-levels) — style, performance, and security observations all live in this single, already-specified severity model. This document introduces no third category.

**Style diagnostics.** Kyne has none beyond formatting itself: because [§10](#10-formatter)'s formatter is unconfigurable and canonical, there is no style question left for a linter to adjudicate — a file either matches `kyne fmt`'s output or it does not, checkable directly via `kyne fmt --check`.

**Performance diagnostics.** Covered by the Security Analyzer's existing checks — for example, [COMPILER_ARCHITECTURE.md §11.1](./COMPILER_ARCHITECTURE.md#111-analyses)'s "dangerous storage growth" and "unsafe runtime assumptions" findings are, in effect, performance-relevant warnings, already fully specified there.

**Security hints and warnings.** Identical to [COMPILER_ARCHITECTURE.md §11.1](./COMPILER_ARCHITECTURE.md#111-analyses)'s full list — this document adds no new check.

**Severity levels.** Identical to [COMPILER_ARCHITECTURE.md §11.2](./COMPILER_ARCHITECTURE.md#112-severity-levels)'s three levels (Compiler Error, Warning, Hint) — restated, not redefined.

---

# 12. Documentation Generator

`kyne doc` produces API documentation for a project's `src/` from `kyne_cst` (for `///` doc comments, per [LANGUAGE_SPEC.md §1.4](./LANGUAGE_SPEC.md#14-comments)) and `kyne_ast` (for declaration shapes).

**Generated output.** The primary output is a static, self-contained HTML site written to `build/docs/`, listing every `public` and `internal` declaration — contracts, structs, enums, errors, events, and functions — with its signature and attached doc comment. A secondary, flat Markdown export MAY also be produced, for embedding in an external documentation pipeline; the HTML site remains the primary, authoritative form.

**API documentation.** Every documented item's page includes its full signature exactly as declared, its doc comment rendered as prose, and, for a function, its parameter and return types.

**Examples.** A `///` doc comment MAY include a fenced ```` ```kyne ```` code block; the documentation generator MUST render it as a syntax-highlighted example using the same tokenization `kyne_lexer` itself performs, guaranteeing the highlighting is never inconsistent with the language's actual grammar.

**Cross references.** A type named in a signature (a parameter type, a return type, an `error` type) MUST be rendered as a link to that type's own documentation page, where one exists within the same project — a reader looking at `transfer`'s signature and its `TokenError` return type should be able to navigate directly to `TokenError`'s own documented variants.

**Documentation philosophy.** `kyne doc` treats documentation as a direct extension of the source it documents, never as a separately authored artifact that can drift from it — every fact `kyne doc` states is derived mechanically from the same `.kyn` source `kyne build` compiles, so a project's documentation can never describe a function that does not exist or a signature that has since changed.

---

# 13. Test Runner

**Testing philosophy.** A Kyne test is an ordinary Kyne function, executed against a mocked execution context, whose pass/fail outcome is [RUNTIME_MODEL.md](./RUNTIME_MODEL.md)'s own existing Success/Failure distinction — Kyne introduces no separate assertion framework beyond [STANDARD_LIBRARY.md §3](./STANDARD_LIBRARY.md#3-core-module)'s `core.assert`/`core.assert_eq`, and no separate pass/fail concept beyond the runtime's own.

**Convention, not annotation.** Kyne has no attribute or annotation syntax (per [LANGUAGE_SPEC.md §1.3](./LANGUAGE_SPEC.md#13-keywords)'s closed keyword set, which includes no macro- or attribute-introducing token). A test case is therefore identified by **location, not marking**: every `public fn` declared in a `.kyn` file under `tests/` is a test case, run independently by `kyne test`. This is a direct application of [Convention Over Configuration](#2-convention-over-configuration): the directory itself is the marker.

**Unit tests.** A test file under `tests/` imports the project's contract (per [LANGUAGE_SPEC.md §12.2](./LANGUAGE_SPEC.md#122-imports)) and exercises individual `public`/`internal` functions directly.

**Integration tests.** A test file MAY exercise a sequence of calls across multiple functions — `init`, then `transfer`, then a query — within a single test case, exactly as a real transaction sequence would, since each test case runs against its own freshly constructed mock execution context spanning its full body.

**Mock runtime.** Every test case executes against [STANDARD_LIBRARY.md §14](./STANDARD_LIBRARY.md#14-test-module)'s `test/` module: `test.mock_ledger` fixes the ledger context, `test.mock_auth` grants authorization without a real signature, and `test.mock_state` pre-seeds `state` for setup convenience. Per [STANDARD_LIBRARY.md §14](./STANDARD_LIBRARY.md#14-test-module), these functions are available only within this test-harness compilation context — `kyne test`'s compilation of `tests/` is exactly that context, and `kyne build`'s compilation of `src/` never permits them.

**Determinism.** Every test case runs against a context fully isolated from every other test case, per [STANDARD_LIBRARY.md §14](./STANDARD_LIBRARY.md#14-test-module)'s isolation guarantee — test execution order MUST NOT affect any test's outcome, and a conforming test runner MUST NOT share mutable state across test cases.

**Future coverage.** Code coverage reporting is reserved for future work, per [§23](#23-future-tooling).

---

# 14. Build Artifacts

```
build/
  wasm/
  rust/
  docs/
```

| Directory | Contents | Produced by |
|---|---|---|
| `build/wasm/` | The final, deployable WASM binary. | `kyne build`, final stage. |
| `build/rust/` | The generated Rust crate — the complete, buildable, reviewable output of [COMPILER_ARCHITECTURE.md §14](./COMPILER_ARCHITECTURE.md#14-rust-code-generation), including its own `Cargo.toml`. | `kyne build`, Generate Rust stage. |
| `build/docs/` | Generated API documentation. | `kyne doc`. |

**Ownership.** Every path under `build/` is tool-generated and MUST NOT be hand-edited — an edit to `build/rust/` is silently discarded the next time `kyne build` runs, since it is regenerated in full, not patched incrementally at the file level.

**Cleanup.** `kyne clean` ([§9](#9-cli-commands)) removes `build/` entirely; this is always safe, per [COMPILER_ARCHITECTURE.md §17.3](./COMPILER_ARCHITECTURE.md#173-cleanup)'s disposability guarantee, extended here to cover `build/` alongside `.kyn/cache/`.

**Why `build/rust/` is retained, not discarded.** Unlike `.kyn/cache/`'s internal, largely-opaque cache entries, `build/rust/` is specifically meant to be opened and read: [Transparent Abstraction](./LANGUAGE_PRINCIPLES.md#transparent-abstraction) requires generated Rust to remain reviewable, and a developer or auditor cannot review what the toolchain does not persist somewhere they can find it.

---

# 15. Compiler Cache

This document does not redefine the compiler cache — [COMPILER_ARCHITECTURE.md §17](./COMPILER_ARCHITECTURE.md#17-compiler-cache) already fixes its layout (`.kyn/cache/<compiler-version>/{cst,ast,hir,rir,diagnostics}/`), its invalidation rules (content hash plus compiler version), its cleanup guarantee (strictly disposable), and its reproducibility guarantee (path- and machine-independent keys).

**Reuse.** `kyne build` and `kyne check` consult it automatically, per [§8](#8-incremental-builds).

**Invalidation, cleaning, and reproducibility.** Identical to [COMPILER_ARCHITECTURE.md §17.2](./COMPILER_ARCHITECTURE.md#172-invalidation), [§17.3](./COMPILER_ARCHITECTURE.md#173-cleanup), and [§17.4](./COMPILER_ARCHITECTURE.md#174-reproducibility) respectively. The only toolchain-level addition is `kyne clean`'s explicit, on-demand invocation of the cleanup guarantee, per [§9](#9-cli-commands).

**Relationship to incremental compilation.** The cache is the storage mechanism; incremental compilation ([§8](#8-incremental-builds)) is the policy that decides what to read from and write to it. Neither concept is redefined here.

---

# 16. Language Server

| Capability | Built from |
|---|---|
| Autocomplete | `kyne_resolver`, `kyne_types` — suggestions are exactly the set of names Name Resolution would consider in scope at the cursor position, filtered by the Type Checker's expected type where one is known. |
| Hover | `kyne_types`, `kyne_ast` — the same type and declaration information the Type Checker itself computed, rendered for a human reader. |
| Rename | `kyne_resolver` — a rename is valid only where every reference the resolver's own symbol table identifies can be updated consistently; there is no separate, approximate "find similar text" fallback. |
| Go-to-definition | `kyne_resolver`, `kyne_cst` — resolves a reference to its declaration's exact CST span, per [COMPILER_ARCHITECTURE.md §6](./COMPILER_ARCHITECTURE.md#6-concrete-syntax-tree). |
| Diagnostics | `kyne_diagnostics`, surfaced incrementally as `kyne_types`/`kyne_semantics`/`kyne_security` run against the file being edited. |
| Formatting | `kyne_cst` — invoking "format document" in an editor runs the identical formatter [§10](#10-formatter) specifies, not an editor-approximated version of it. |
| Semantic highlighting | `kyne_cst`, `kyne_ast` — token classification (a `state` field reference versus a `let` local, for example) uses the same resolved information Name Resolution computed, not a regex-based heuristic. |

**Why every capability reuses compiler libraries.** An editor feature implemented independently of the compiler — a regex-based "go to definition," a heuristic autocomplete — will eventually disagree with what `kyne build` actually accepts, producing the exact drift [§1](#1-toolchain-philosophy) identifies as the core problem an integrated toolchain avoids. Every capability in this table is, by construction, unable to disagree with the compiler, because it is not a second implementation of the same analysis — it is the compiler's own analysis, incrementally re-run and exposed over LSP.

---

# 17. Playground

A browser-hosted environment for writing and exploring Kyne without a local installation.

**Compilation.** The playground runs the full pipeline through `kyne_codegen`, letting a visitor read the generated Rust for whatever they have written — the same transparency guarantee [§14](#14-build-artifacts) provides locally.

**Formatting.** The playground formats input using the identical `kyne_cst`-based formatter specified in [§10](#10-formatter).

**Examples.** The playground SHOULD ship pre-loaded with the six canonical examples from [LANGUAGE_SPEC.md §16](./LANGUAGE_SPEC.md#16-examples), as the fastest available on-ramp for a new visitor, per [FOUNDATION.md](./FOUNDATION.md#success-metrics)'s "first contract within one hour" success metric.

**Sharing.** A visitor MAY generate a shareable link to a specific piece of source; this is a persistence and URL-generation concern for the hosting implementation, not a compiler capability, and is not further specified here.

**Learning.** The playground's core purpose is learning and demonstration, not production contract development — it is explicitly not a substitute for a local `kyne new` project once a visitor moves past initial exploration.

**No deployment.** The playground MUST NOT provide any mechanism to deploy compiled output to a real network, sign a real transaction, or interact with real keys. This is an absolute boundary, not a configuration option.

**Security considerations.** Because the playground compiles and, where it offers execution, runs arbitrary visitor-submitted source, it MUST execute exclusively within a sandboxed environment using [STANDARD_LIBRARY.md §14](./STANDARD_LIBRARY.md#14-test-module)'s mock execution context — never against real network or ledger access — and MUST enforce resource limits (time and memory) on every compilation and execution request, since it is, by nature, processing untrusted input from the public internet.

---

# 18. IDE Integration

Kyne's editor support is **editor-independent by construction**: [§16](#16-language-server)'s Language Server implements the standard Language Server Protocol (LSP), and every capability it exposes is available to any editor with an LSP client, with no editor-specific reimplementation of language intelligence.

| Editor | Integration shape |
|---|---|
| VS Code | A thin extension wrapping the standard LSP client, plus syntax-highlighting grammar generated from `kyne_lexer`'s token definitions. |
| Cursor, Windsurf | Both are VS Code-compatible; the same extension applies without modification. |
| Zed | A thin LSP client configuration, per Zed's own extension mechanism. |
| Neovim | Configured via Neovim's built-in LSP client against the `kyne` language server binary directly — no Kyne-specific plugin logic beyond registering the server. |

**Editor independence.** No editor integration in this table contains language-intelligence logic of its own — every one is a thin adapter between the editor's own extension surface and the single, shared Language Server from [§16](#16-language-server). A new editor gains full Kyne support by implementing an LSP client, which is normal editor infrastructure unrelated to Kyne specifically, not by anyone writing Kyne-specific tooling for that editor.

**Protocol usage.** The Language Server MUST communicate exclusively over standard LSP — there is no Kyne-proprietary protocol extension required for any capability in [§16](#16-language-server)'s table, keeping every general-purpose LSP client fully compatible.

---

# 19. Version Management

**Compiler versions.** The `kyne` compiler is versioned with ordinary semantic versioning. Per [COMPILER_ARCHITECTURE.md's Principle 5](./COMPILER_ARCHITECTURE.md#principle-5--predictable-output), byte-identical output is guaranteed only for a fixed compiler version — this document does not weaken that guarantee, only names the version number a developer sees via `kyne version`.

**Project editions.** A project's `Kyne.toml` `edition` field, per [§5](#5-kynetoml), names the language edition it targets. Kyne v1 defines exactly one edition, `"v1"`.

**Compatibility.** A given compiler version MUST be able to build every project declaring an edition that compiler version supports; a compiler is never required to support an edition newer than itself, and a project declaring an edition unknown to the installed compiler MUST fail with a clear diagnostic identifying the mismatch, not a confusing downstream parse error.

**Migration.** A future edition MAY change default behavior that is not itself a MUST-level rule of [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) or [RUNTIME_MODEL.md](./RUNTIME_MODEL.md) — for example, a future canonical formatting refinement. A future edition MUST NOT silently change the meaning of an existing edition's programs; a project that does not update its `edition` field continues to build with its original edition's behavior indefinitely, mirroring the edition system precedent established by mature toolchains this specification is held to the quality bar of.

**Future edition policy.** A new edition is introduced only through the KIP process, per [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md#language-evolution), and is always opt-in via the `edition` field — never a forced migration for existing projects.

---

# 20. Reproducible Builds

**Deterministic build guarantee.** For a fixed set of inputs — `Kyne.toml`, the complete `src/` tree, the compiler version, and the external Cargo/Soroban toolchain versions — `kyne build` MUST always produce byte-identical generated Rust ([COMPILER_ARCHITECTURE.md §14.4](./COMPILER_ARCHITECTURE.md#144-deterministic-generation)) and, given identical external toolchains, byte-identical WASM.

**Compiler version.** As in [§19](#19-version-management), reproducibility across the Kyne-owned stages of the pipeline is guaranteed only for a fixed compiler version.

**Manifest and configuration.** `Kyne.toml`'s contents are part of the build's input surface — two builds with different manifest contents are not expected to produce identical output, and this is not a reproducibility violation, since the inputs genuinely differed.

**Hashes.** `.kyn/cache/`'s content-hash-keyed entries, per [COMPILER_ARCHITECTURE.md §17](./COMPILER_ARCHITECTURE.md#17-compiler-cache), are the same hashes that make reproducibility checkable in practice: two independent builds of identical inputs produce identical cache keys, per [COMPILER_ARCHITECTURE.md §17.4](./COMPILER_ARCHITECTURE.md#174-reproducibility).

**Generated Rust.** Guaranteed byte-identical for identical Kyne-owned inputs, per [COMPILER_ARCHITECTURE.md §14.4](./COMPILER_ARCHITECTURE.md#144-deterministic-generation).

**Generated WASM.** Guaranteed byte-identical only when the external Rust and Soroban toolchain versions are additionally held fixed, since those stages are outside Kyne's own determinism guarantee — a project requiring full, end-to-end WASM reproducibility SHOULD pin exact external toolchain versions in `Kyne.toml`'s `[build]` section, per [§5](#5-kynetoml).

**Why reproducibility matters.** A contract's generated Rust and final WASM are what gets audited and what gets deployed; if two builds of the identical source could produce different output, an auditor's review of one build would not actually certify the artifact a different build produces, silently breaking the audit's own premise. Reproducibility is what makes "I reviewed this generated Rust" and "this is the WASM that got deployed" the same claim.

---

# 21. Toolchain Invariants

**The formatter always produces canonical output.** `kyne fmt` run twice in succession on any input MUST produce identical output the second time as the first — idempotence is not a nice-to-have, it is the operational definition of "canonical," per [§10](#10-formatter). A formatter change that broke idempotence for any valid input would be a defect requiring an immediate fix, not a follow-up improvement.

**The compiler owns diagnostics.** No toolchain component outside `kyne_diagnostics` may originate a diagnostic message a developer sees — [§16](#16-language-server)'s Language Server and [§9](#9-cli-commands)'s CLI both render diagnostics `kyne_diagnostics` produces; neither is permitted to have its own, separately worded diagnostic for the same underlying condition, which would risk the two disagreeing about the same error.

**The CLI never rewrites business logic unexpectedly.** Every command in [§9](#9-cli-commands) that modifies source files — `kyne fmt` and `kyne fix` — is scoped exactly to the transformations [§10](#10-formatter) and [§26](#26-developer-assistance-commands) respectively define, both of which are proven, not merely believed, to preserve program meaning. No command in this specification, now or in the future, may silently alter a contract's runtime behavior as a side effect of a convenience feature — this is [Security Before Convenience](./LANGUAGE_PRINCIPLES.md#security-before-convenience) applied to tooling.

**Every tool reuses compiler libraries.** Restated permanently from [Toolchain Principle 5](#5-compiler-first): no future toolchain component may implement its own parser, its own type checker, or its own formatter — every one MUST be built from the `kyne_*` crates [COMPILER_ARCHITECTURE.md §22](./COMPILER_ARCHITECTURE.md#22-library-first-compiler-architecture) already defines.

**Builds are deterministic.** Restated permanently from [§20](#20-reproducible-builds) — no future toolchain feature (a build cache, a parallelization strategy, a future incremental-compilation refinement) may be shipped if it introduces any variance into build output for fixed inputs.

**Why contributors must preserve them.** Every invariant above protects the same underlying promise: a Kyne developer's trust that the tool in front of them behaves exactly as this document says, every time, with no tool-specific exception quietly carved out for convenience. A toolchain contributor who violates one of these invariants — even for a feature that seems purely additive — has broken that promise for every developer relying on it, often silently, since a toolchain defect of this kind is precisely the kind a developer has no independent way to detect.

---

# 22. Future Package Manager

Package management is **not designed by this document**. [LANGUAGE_SPEC.md's Future Ecosystem section](./LANGUAGE_SPEC.md#future-ecosystem) and [STANDARD_LIBRARY.md's Non-Goals](./STANDARD_LIBRARY.md#non-goals) already reserve this scope; this chapter additionally reserves the conceptual CLI surface a future package manager would occupy, so that its eventual design has an expected home without this document attempting to anticipate its shape:

```text
kyne add
kyne remove
kyne update
```

These commands do not exist in Kyne v1. Their eventual purpose — respectively, adding, removing, and updating an entry in `Kyne.toml`'s `[dependencies]` section, per [§5](#5-kynetoml) — is named here only to reserve the verbs, consistent with [§20 of LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md#language-evolution)-style forward reservation. Package management belongs to a future `PACKAGE_MANAGER.md`, to be produced through the KIP process, addressing registry design, dependency resolution, version resolution, and package signing — none of which this document takes a position on.

---

# 23. Future Tooling

The following are intentionally **outside the scope of Kyne v1** and MUST NOT be introduced without a KIP:

- **Profiler** — execution-cost profiling for contract functions, useful once real-world resource-metering data from deployed contracts exists to inform its design.
- **Benchmarking** — a `kyne bench`-style harness for comparing implementation strategies, reserved pending demonstrated need.
- **Debugger** — interactive, step-through debugging of contract execution; a substantial undertaking given [RUNTIME_MODEL.md](./RUNTIME_MODEL.md)'s staged-execution model, reserved for a dedicated future specification.
- **Coverage** — test coverage reporting for `kyne test`, per [§13](#13-test-runner)'s own deferral.
- **Package registry** — the hosted service a future package manager ([§22](#22-future-package-manager)) would depend on.
- **Cloud compiler** — a hosted build service beyond [§17](#17-playground)'s playground, for CI or team-shared build acceleration.
- **Additional IDE integrations** — editors beyond [§18](#18-ide-integration)'s initial list; new integrations remain thin LSP clients per that chapter's existing architecture and require no new specification to add.

**These are intentionally outside Kyne v1.** Each is deferred for the same reason [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md), [RUNTIME_MODEL.md](./RUNTIME_MODEL.md), [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md), [MEMORY_MODEL.md](./MEMORY_MODEL.md), and [STANDARD_LIBRARY.md](./STANDARD_LIBRARY.md) defer their own reserved features: real usage should inform design, not precede it. Any future addition MUST go through the KIP process in [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md#language-evolution).

---

# 24. Toolchain Commandments

1. **One executable.** Everything is `kyne <subcommand>`; there is no second binary a developer must separately install, discover, or keep in version-sync, per [§3](#3-cli-philosophy).

2. **One formatter.** `kyne fmt` is unconfigurable and canonical; there is no second, alternative formatting style a project can opt into, per [§10](#10-formatter).

3. **One compiler.** Every tool in [§2](#2-toolchain-components) is built from the identical `kyne_*` libraries the compiler itself is built from; no tool may embed a second, independent understanding of the language, per [Toolchain Principle 5](#5-compiler-first).

4. **Convention over configuration.** [Project Layout](#4-project-layout), [test discovery](#13-test-runner), and [formatting](#10-formatter) all work from sensible, fixed defaults; a new project requires no configuration decisions to build, check, format, and test successfully on its first `kyne build`.

5. **No hidden configuration.** Every setting that affects build output lives in `Kyne.toml`, visible and version-controlled, per [§5](#5-kynetoml) — there is no environment variable, no user-global config file, and no implicit machine-specific state capable of changing what a project builds to, since any of those would directly threaten [§20](#20-reproducible-builds)'s guarantee.

6. **Fast by default.** [§8](#8-incremental-builds)'s incremental compilation is not an opt-in mode; it is simply how `kyne build` and `kyne check` behave, always, per [Toolchain Principle 3](#3-fast-feedback).

7. **Deterministic by default.** [§20](#20-reproducible-builds)'s reproducibility guarantee requires no special flag to obtain — it is the ordinary behavior of an ordinary `kyne build`.

8. **Compiler libraries are the single source of truth.** Restated from commandment 3, at the data level rather than the tool level: [§16](#16-language-server)'s Language Server, [§10](#10-formatter)'s formatter, and [§12](#12-documentation-generator)'s documentation generator all derive their understanding of a program from the identical `kyne_cst`/`kyne_ast`/`kyne_resolver`/`kyne_types` data structures — never from independently parsed or independently inferred information that could, even briefly, disagree with what `kyne build` itself would conclude.

**Why these principles are permanent.** Every commandment above exists to prevent a specific, well-understood failure mode of mature toolchains: fragmentation into multiple binaries with drifting versions, configuration sprawl that makes "why did this build differently on my machine" an ordinary support question, and tool-specific reimplementations of language understanding that quietly diverge from the compiler's own. Kyne's toolchain is young enough, at this milestone, to design these failure modes out structurally rather than discover them empirically — these commandments are the record of that decision, binding on every future contributor exactly because the alternative has already played out, repeatedly, in other ecosystems this specification's authors have observed.

---

# 25. Developer Journey

```
Install Kyne
    ↓
kyne new
    ↓
Write contract
    ↓
kyne check
    ↓
kyne test
    ↓
kyne fmt
    ↓
kyne build
    ↓
Deploy WASM
    ↓
Generate documentation
    ↓
Publish package (future)
```

**Install Kyne.** A developer installs exactly one executable, per [§3](#3-cli-philosophy) — there is no second installation step for a formatter, linter, or test runner.

**`kyne new`.** The developer scaffolds a project from a template, per [§6](#6-project-templates), receiving a complete, immediately buildable [Project Layout](#4-project-layout) with no further setup required, per [Convention Over Configuration](#2-convention-over-configuration).

**Write contract.** The developer edits `.kyn` source under `src/`, with [§16](#16-language-server)'s Language Server providing autocomplete, hover, and inline diagnostics throughout — feedback begins before the developer ever runs a CLI command.

**`kyne check`.** The developer runs a fast, full-project correctness and security pass, per [§9](#9-cli-commands), without paying the cost of code generation or an external toolchain invocation — the fastest available feedback loop short of the editor itself.

**`kyne test`.** The developer runs the project's `tests/` suite, per [§13](#13-test-runner), exercising contract logic against a deterministic mock execution context before ever considering deployment.

**`kyne fmt`.** The developer reformats their source to Kyne's canonical style, per [§10](#10-formatter) — a step so mechanical it is frequently automated via editor-on-save integration rather than run explicitly, though it remains available as an explicit command for CI enforcement via `kyne fmt --check`.

**`kyne build`.** The developer produces a deployable WASM artifact, per [§7](#7-build-system), with `--release` for the fully optimized, `debug.*`-elided production build.

**Deploy WASM.** The developer takes `build/wasm/`'s output and deploys it via the standard Soroban deployment tooling, per [RUNTIME_MODEL.md §3](./RUNTIME_MODEL.md#3-project-lifecycle) — deployment itself is Soroban's own concern, outside this document's scope, per [Non-Goals](#non-goals).

**Generate documentation.** The developer runs `kyne doc`, per [§12](#12-documentation-generator), producing a reference for their contract's public API — a step available at any point in the journey, not only at the end, since documentation generation has no dependency on a successful deployment.

**Publish package (future).** Once a package manager exists, per [§22](#22-future-package-manager), a developer would share reusable Kyne libraries through it. This step is included in the journey diagram to show where that future capability will sit, not because it exists today.

**Philosophy behind this workflow.** Every stage in this journey is reachable through a single, memorable command, per [Toolchain Principle 4](#4-one-command-per-job), and every stage's feedback arrives faster than the stage after it — checking is faster than testing, testing is faster than building, per [Toolchain Principle 3](#3-fast-feedback) — so a developer is never forced to pay a slow feedback cost earlier in their workflow than the task actually requires.

---

# 26. Developer Assistance Commands

## `kyne doctor`

| Field | Specification |
|---|---|
| Purpose | Verify that a developer's local environment is correctly configured to build and deploy Kyne contracts. |
| Arguments | None. |
| Behavior | Performs a read-only sequence of environment checks — see table below — and reports each as OK, a Warning (functional but not recommended), or a Problem (will prevent some command from working), with a suggested remediation for every Warning and Problem. `kyne doctor` MUST NOT modify the developer's environment; it only inspects and reports. |
| Exit codes | `0` if every check reported OK or Warning; `1` if any check reported a Problem. |
| Examples | `kyne doctor` |
| Interaction with the compiler | Reads the installed `kyne` binary's own version metadata; otherwise inspects the surrounding environment only. |

| Checked | Why it matters |
|---|---|
| Compiler version | Confirms the installed `kyne` matches the project's `Kyne.toml` edition requirement, per [§19](#19-version-management). |
| Rust installation | Required for [§7](#7-build-system)'s Cargo Build stage. |
| Cargo | As above. |
| Soroban CLI | Required for the Soroban Build stage and for deployment. |
| Stellar CLI | Required for network interaction beyond compilation. |
| Environment variables | Confirms any variables the external toolchains require are set. |
| `PATH` | Confirms every required external tool is actually reachable, not merely installed somewhere. |
| Build dependencies | Confirms the external toolchain versions are compatible with what [§20](#20-reproducible-builds) expects for reproducible output. |

**Diagnostics and recommendations.** `kyne doctor`'s report follows the same shape as [COMPILER_ARCHITECTURE.md §15](./COMPILER_ARCHITECTURE.md#15-diagnostics-engine)'s diagnostic engine — a specific finding, why it matters, and a concrete suggested fix — extending [Compiler as a Teacher](./COMPILER_ARCHITECTURE.md#principle-1--the-compiler-is-a-teacher) from source-code diagnostics to environment diagnostics: a developer's first encounter with Kyne tooling should be exactly as pedagogically supported as their first compiler error.

## `kyne explain <code>`

```text
kyne explain KY0031
```

| Field | Specification |
|---|---|
| Purpose | Explain a compiler diagnostic code in full, independent of encountering it in a real build. |
| Arguments | `<code>` — a `KY`- or `KS`-prefixed diagnostic code, per [COMPILER_ARCHITECTURE.md §15.1](./COMPILER_ARCHITECTURE.md#151-diagnostic-code-namespace). |
| Behavior | Prints the diagnostic's full metadata: its meaning, common causes, one or more illustrative examples of code that triggers it, suggested fixes, a link to the relevant section of [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md), [RUNTIME_MODEL.md](./RUNTIME_MODEL.md), or [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md), and, where relevant, a reference to the KIP that introduced or last amended the rule. |
| Exit codes | `0` if `<code>` is a recognized diagnostic; `2` otherwise. |
| Examples | `kyne explain KY0031`, `kyne explain KS0101` |
| Interaction with the compiler | `kyne_diagnostics` — `kyne explain` draws from the **identical** diagnostic metadata table [COMPILER_ARCHITECTURE.md §15](./COMPILER_ARCHITECTURE.md#15-diagnostics-engine) already requires every diagnostic to carry (Error Code, Title, Explanation, Reason, Suggested Fix, Future Documentation Link); it is not a second, separately maintained documentation source, and MUST NOT be permitted to drift from the messages a real build actually produces. |

**Reinforcing "The Compiler Is a Teacher."** `kyne explain` exists because a developer does not always encounter a diagnostic at the moment they are best positioned to learn from it — sometimes the fastest path forward is to suppress a build error under time pressure and return to understand it properly later. `kyne explain` makes the compiler's pedagogical content available on the developer's own schedule, not only at the moment of failure, which is a direct, structural extension of [COMPILER_ARCHITECTURE.md's Principle 1](./COMPILER_ARCHITECTURE.md#principle-1--the-compiler-is-a-teacher) into the toolchain layer.

## `kyne fix`

| Field | Specification |
|---|---|
| Purpose | Apply automatic, semantics-preserving fixes to a project's source. |
| Arguments | None. |
| Behavior | Applies exactly the transformations in the table below — nothing else. |
| Exit codes | `0` on success (whether or not any fix was applied); `1` if a fix could not be safely applied due to a remaining Compiler Error unrelated to the fixable set. |
| Examples | `kyne fix` |
| Interaction with the compiler | `kyne_cst` (formatting, import ordering), `kyne_resolver` (unused-import detection), `kyne_semantics` (canonical member reordering). |

| Permitted fix | Why it is safe |
|---|---|
| Applying `kyne fmt`'s canonical formatting | Formatting has no effect on program meaning by definition, per [§10](#10-formatter). |
| Reordering `use` imports per [LANGUAGE_SPEC.md §13](./LANGUAGE_SPEC.md#13-formatting-rules)'s import-ordering rule | Import order has no effect on name resolution outcomes, only on source presentation. |
| Removing an unused `use` import | Provably safe, not merely likely safe: an import with zero references anywhere in the resolved AST — a fact `kyne_resolver` determines with certainty, not heuristically — cannot affect compiled output by definition. |
| Reordering contract members into [canonical order](./LANGUAGE_SPEC.md#32-canonical-member-order) | Provably safe: [LANGUAGE_SPEC.md §2.1](./LANGUAGE_SPEC.md#21-name-resolution-is-order-independent) guarantees declaration order has zero effect on a program's resolved meaning — canonical ordering is a presentation rule, not a semantic one, so reordering to satisfy it changes nothing a running contract could observe. |

**This command MUST NEVER rewrite business logic.** `kyne fix` MUST NOT remove a function, struct, enum, or any other declaration flagged by the Security Analyzer's "dead code" check ([COMPILER_ARCHITECTURE.md §11.1](./COMPILER_ARCHITECTURE.md#111-analyses)), and MUST NOT alter any expression, statement, or control-flow structure anywhere in a project. The critical distinction is **certainty**: every permitted fix in the table above is proven safe by a compiler analysis with no possibility of a false positive (import usage is a fact about the resolved AST; declaration order is proven irrelevant by [LANGUAGE_SPEC.md §2.1](./LANGUAGE_SPEC.md#21-name-resolution-is-order-independent)). Dead-code removal, by contrast, is explicitly classified as a **heuristic** Security Analyzer finding, per [COMPILER_ARCHITECTURE.md §11.2](./COMPILER_ARCHITECTURE.md#112-severity-levels) — capable of a false positive — and applying a heuristic finding automatically risks silently deleting code a developer retained intentionally. `kyne fix` is bound by [§21](#21-toolchain-invariants)'s "CLI never rewrites business logic unexpectedly" invariant, and this boundary — provable transformations only, never heuristic ones — is how that invariant is enforced concretely for this specific command.

---

# Non-Goals

This document does **not** define, and explicitly defers to other documents:

- **Language syntax** — defined in [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md).
- **Runtime semantics** — defined in [RUNTIME_MODEL.md](./RUNTIME_MODEL.md).
- **Compiler implementation** — defined in [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md).
- **Memory algorithms** — defined in [MEMORY_MODEL.md](./MEMORY_MODEL.md).
- **Standard library APIs** — defined in [STANDARD_LIBRARY.md](./STANDARD_LIBRARY.md).
- **Package manager implementation** — reserved per [§22](#22-future-package-manager), to be defined in a future `PACKAGE_MANAGER.md`.
- **Cloud deployment** — deploying a built WASM artifact to a live network is Soroban's own deployment tooling's concern, outside this document's scope.
- **IDE implementation details** — this document specifies the Language Server's capabilities ([§16](#16-language-server)) and the editor-independence guarantee ([§18](#18-ide-integration)); it does not specify any individual editor extension's internal implementation.
- **Operating system APIs** — not applicable to a toolchain whose compiled output has no OS-level presence; the toolchain itself runs on a developer's machine using ordinary OS facilities not specific to Kyne.

---

# Cross References

This document is normatively dependent on:

- [FOUNDATION.md](./FOUNDATION.md) — the mission and success metrics ([§17](#17-playground), [§25](#25-developer-journey)) this toolchain exists to serve.
- [LANGUAGE_PRINCIPLES.md](./LANGUAGE_PRINCIPLES.md) — the design philosophy and KIP governance process this document's own evolution follows.
- [LANGUAGE_SPEC.md](./LANGUAGE_SPEC.md) — the formatting rules, canonical ordering, module system, and canonical examples this toolchain implements and reuses directly.
- [RUNTIME_MODEL.md](./RUNTIME_MODEL.md) — the failure model this document's test runner and `kyne run` command read their pass/fail semantics from directly.
- [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md) — the pipeline, library crates, diagnostic engine, and Security Analyzer every tool in this document is built from.
- [MEMORY_MODEL.md](./MEMORY_MODEL.md) — referenced for consistency; this document introduces no toolchain feature that touches memory behavior directly.
- [STANDARD_LIBRARY.md](./STANDARD_LIBRARY.md) — the `test/` module primitives this document's test runner and playground both build on.

The following documents are anticipated but not yet written, and this document's [Non-Goals](#non-goals) section reserves their scope explicitly: `GOVERNANCE.md`, `ROADMAP.md`, `PACKAGE_MANAGER.md`, and `ECOSYSTEM.md`. Until each exists, this document does not speculate on their contents beyond the scope boundary already stated above.

---

# Closing

This document is the single authoritative specification for the Kyne developer experience. A future contributor implementing any part of the toolchain — the CLI, the formatter, the language server, the documentation generator, the test runner — should be able to determine, from this document alone, exactly what that component MUST do and exactly which compiler libraries it MUST be built from. Every design choice in this document that departed from an intuitive first guess is recorded here with its reasoning, so that reasoning does not need to be rediscovered, and is not casually reversed without being engaged with directly. Where a future engineer finds a genuine gap this document does not address, that is a gap to be closed by a KIP — not a decision to be made silently in an implementation.
