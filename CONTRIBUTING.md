# Contributing to Kyne

Thank you for your interest in Kyne. This document is the practical
companion to [`ARCHITECTURE.md`](./ARCHITECTURE.md) — read that first if
you have not already; it is the primary onboarding document for this
repository, and this file assumes it. Everything here is informative,
per [`GOVERNANCE.md` §17](./docs/GOVERNANCE.md#17-documentation-governance) —
it describes how to work in this repository, not what Kyne is.

---

## Before you start

Kyne is currently in **Phase 1 — Compiler Bootstrap**
(see [`ROADMAP.md`](./docs/ROADMAP.md)). Nine constitutional documents
under [`docs/`](./docs/) fully specify the language; almost none of it is
implemented yet. If you are looking for what to build, the open issues on
this repository are the current source of truth for what is actively
being worked on, sequenced per
[`ROADMAP.md` §7](./docs/ROADMAP.md#7-compiler-bootstrap-order).

Every constitutional document is locked: a contribution MUST NOT
introduce behavior that contradicts `LANGUAGE_SPEC.md`, `RUNTIME_MODEL.md`,
`COMPILER_ARCHITECTURE.md`, `MEMORY_MODEL.md`, `STANDARD_LIBRARY.md`,
`TOOLCHAIN.md`, or `GOVERNANCE.md`. If you believe one of them is wrong,
that is a [KIP](./docs/kip/README.md), not a pull request.

---

## Getting started

```sh
git clone https://github.com/kyne-lang/kyne.git
cd kyne
scripts/bootstrap.sh   # verifies your toolchain and builds the workspace
```

Day-to-day scripts, all thin wrappers over `cargo` (see
[`scripts/README.md`](./scripts/README.md) for what each does and does
not cover):

```sh
scripts/format.sh   # cargo fmt --all
scripts/lint.sh      # cargo clippy --workspace --all-targets -- -D warnings
scripts/test.sh      # cargo test --workspace
scripts/docs.sh       # cargo doc --workspace --no-deps
```

These format, lint, test, and document **this repository's own Rust
implementation** — the compiler, standard library, and tooling crates.
They are not the same as the eventual `kyne fmt` / `kyne test` / `kyne doc`
subcommands, which will operate on Kyne (`.kyn`) source once they exist;
see `scripts/README.md`'s note on this distinction.

---

## Where to work

| You want to... | Look at |
|---|---|
| Understand the repository layout and crate boundaries | [`ARCHITECTURE.md`](./ARCHITECTURE.md) |
| Find a specific compiler crate's purpose, responsibilities, and current status | That crate's own `README.md`, per [`ARCHITECTURE.md` §4](./ARCHITECTURE.md#4-compiler-crates)–[§6](./ARCHITECTURE.md#6-standard-library-repository) |
| Understand a language, runtime, or compiler behavior in depth | The relevant constitutional document under [`docs/`](./docs/) |
| See what's planned and in what order | [`ROADMAP.md`](./docs/ROADMAP.md) |
| Pick up a specific, scoped piece of work | The repository's [open issues](https://github.com/kyne-lang/kyne/issues) |

---

## The contributor journey

Kyne's governance defines a deliberate progression —
Community Member → Contributor → Reviewer → Maintainer → Core Maintainer
— explained fully in
[`GOVERNANCE.md` §12](./docs/GOVERNANCE.md#12-contributor-journey). You do
not need to ask permission to become a Contributor: submitting a merged,
non-trivial change is what makes you one. Later roles are earned through
demonstrated judgment, observed over time, never by seniority elsewhere
or self-nomination.

---

## KIPs and ADRs

Kyne distinguishes two kinds of change, and getting this distinction
right matters:

- **A [KIP](./docs/kip/README.md)** (Kyne Improvement Proposal) changes
  *what Kyne is* — language syntax, runtime semantics, compiler
  architecture, the memory model, standard library behavior, toolchain
  behavior, or governance itself. It amends a constitutional document and
  requires Core Maintainer consensus, per
  [`GOVERNANCE.md` §6](./docs/GOVERNANCE.md#6-kip-lifecycle).
- **An [ADR](./docs/adr/README.md)** (Architecture Decision Record)
  records *how something is implemented* — a choice that does not change
  any constitutional document's specified behavior, such as an internal
  data structure or a crate-naming convention. It is reviewed by the
  affected crate's own Maintainer(s), per
  [`GOVERNANCE.md` §19](./docs/GOVERNANCE.md#19-architecture-decision-records-adr).

If you are unsure which applies, ask in your issue or pull request before
writing code — the test, per
[`GOVERNANCE.md` §25](./docs/GOVERNANCE.md#25-constitutional-documents), is
whether the change could alter what a conforming implementation is
required to do.

Most day-to-day contributions — bug fixes, an individual compiler stage's
implementation against an already-accepted specification, refactoring,
documentation, tests — require **neither** a KIP nor an ADR, per
[`GOVERNANCE.md` §26](./docs/GOVERNANCE.md#26-governance-scope). Ordinary
review is the right process for most of what you will work on.

---

## Making a change

1. Fork or branch, and make your change.
2. Match the crate or module's existing conventions — see
   [`ARCHITECTURE.md` §15](./ARCHITECTURE.md#15-coding-standards).
3. Add or update tests at the level appropriate to what you touched, per
   [`ARCHITECTURE.md` §16](./ARCHITECTURE.md#16-testing-philosophy). A
   change without a test is an incomplete change, not a smaller one.
4. Update documentation in the same change — a crate's `README.md`, doc
   comments, or a constitutional document, as applicable. Documentation
   evolves with implementation; it is not a follow-up task.
5. Run the local checks before opening a pull request:

   ```sh
   scripts/format.sh
   scripts/lint.sh
   scripts/test.sh
   ```

6. Open a pull request. Fill out every item in the checklist in
   [`.github/PULL_REQUEST_TEMPLATE.md`](./.github/PULL_REQUEST_TEMPLATE.md) —
   it is not a formality, it is the same checklist a reviewer will hold
   your change against, per
   [`GOVERNANCE.md` §13](./docs/GOVERNANCE.md#13-code-review-philosophy).
7. Reference the issue your pull request addresses (for example,
   `Closes #6`), if one exists.

`main` is protected: changes land through pull request review, not
direct pushes.

---

## Code review

A reviewer evaluates a change against seven dimensions, per
[`GOVERNANCE.md` §13](./docs/GOVERNANCE.md#13-code-review-philosophy):
correctness, documentation, testing, performance, security,
maintainability, and consistency. Correctness is always non-negotiable;
the rest are evaluated as applicable to the change. A review that only
checks correctness while ignoring a missing test is an incomplete review
— reviewers are expected to check the whole list, not only the part
closest to their own expertise.

---

## Reporting a security issue

**Do not open a public issue for a suspected vulnerability.** See
[`.github/SECURITY.md`](./.github/SECURITY.md) for the private reporting
channel and what happens afterward.

---

## License

By contributing to Kyne, you agree that your contributions will be
licensed under the [Apache License 2.0](./LICENSE), matching every other
file in this repository.
