# Repository Structure Fix Report

**Date:** 2026-07-15
**Type:** Repository correction — not a language, runtime, compiler, memory, standard library, toolchain, or governance change.
**Full decision record:** [`docs/adr/ADR-0002-foundation-md-location.md`](./docs/adr/ADR-0002-foundation-md-location.md)

---

## 1. Summary

An accidental nested `kyne/` directory, containing the misplaced
`Foundation.md`, has been removed. `Foundation.md` now lives at its
architecturally correct location, `docs/FOUNDATION.md`, alongside every
other constitutional document. Every live cross-reference to it, across
the entire repository, has been repaired and verified to resolve. No
constitutional document's normative content was changed — only file
locations and the links pointing to them.

---

## 2. Files Moved

| From | To |
|---|---|
| `kyne/Foundation.md` | `docs/FOUNDATION.md` |

The move was performed as a direct filesystem move (not a copy-then-delete
via re-authored content), guaranteeing byte-for-byte fidelity and exactly
one authoritative copy. The now-empty `kyne/` directory was removed
immediately afterward. Confirmed by direct filesystem check: exactly one
file named `FOUNDATION.md` (case-insensitive) exists anywhere in the
repository, and `kyne/` no longer exists at all.

---

## 3. Files Updated

Every file below had one or more relative-link targets corrected from a
`kyne/Foundation.md`-relative path to a `docs/FOUNDATION.md`-relative
path. No other content in any of these files was changed.

| File | References fixed |
|---|---|
| `docs/LANGUAGE_PRINCIPLES.md` | 7 |
| `docs/LANGUAGE_SPEC.md` | 2 |
| `docs/RUNTIME_MODEL.md` | 3 |
| `docs/COMPILER_ARCHITECTURE.md` | 3 |
| `docs/MEMORY_MODEL.md` | 4 |
| `docs/STANDARD_LIBRARY.md` | 2 |
| `docs/TOOLCHAIN.md` | 3 |
| `docs/GOVERNANCE.md` | 5 |
| `docs/ROADMAP.md` | 10 |
| `ARCHITECTURE.md` | 3 link targets, plus the workspace-layout tree diagram, the `docs/` layout tree's stale "symlinked or referenced from" annotation, and the "Note on the current repository" paragraph |
| `README.md` | 3 link targets, plus the repository-structure tree diagram and the documentation table's "except `FOUNDATION.md`" caveat |
| `examples/tutorials/README.md` | 1 |

New files created as part of this correction:

- `docs/adr/ADR-0002-foundation-md-location.md` — the decision record for this fix.
- `docs/adr/README.md` — index updated to list ADR-0002.
- `REPOSITORY_STRUCTURE_FIX_REPORT.md` — this report.

---

## 4. Documentation Links Repaired

Nine constitutional documents under `docs/` used the pattern
`[FOUNDATION.md](../kyne/Foundation.md)` (and anchored variants such as
`../kyne/Foundation.md#why-kyne-exists`) — every instance was rewritten to
`[FOUNDATION.md](./FOUNDATION.md)`, since `FOUNDATION.md` now lives
alongside them in the same directory. `ARCHITECTURE.md` (at the
repository root) and `README.md` (also at the root) used
`kyne/Foundation.md` directly — rewritten to `docs/FOUNDATION.md`.
`examples/tutorials/README.md` (two directories deep) used
`../../kyne/Foundation.md` — rewritten to `../../docs/FOUNDATION.md`.

Two structural diagrams were additionally corrected, beyond simple link
retargeting:

- **`ARCHITECTURE.md` §3**'s workspace-layout tree and the equivalent
  tree in `README.md` both originally opened with a bare `kyne/` line
  intended to label the repository root itself (an ordinary tree-diagram
  convention) — but, juxtaposed with a real, literal `kyne/` subdirectory
  elsewhere in the repository, this was a plausible contributing source
  of the original confusion. Both diagrams now open with an explicit `.`
  root marker instead.
- **`ARCHITECTURE.md` §9**'s `docs/` layout diagram described
  `FOUNDATION.md` as "symlinked or referenced from `kyne/Foundation.md`,"
  acknowledging a mismatch between specification and reality that no
  longer exists — this annotation was removed, since `FOUNDATION.md` is
  now simply, directly present in `docs/`.

---

## 5. References Corrected

A full-repository grep for `kyne/Foundation`, `kyne/`, and every
equivalent relative-path form specified in the task (`FOUNDATION.md`,
`docs/FOUNDATION.md`, `../FOUNDATION.md`, `../../FOUNDATION.md`,
`kyne/Foundation.md`) was performed before and after the fix. After the
fix, the only remaining occurrences of the string `kyne/` anywhere in the
repository (excluding `target/` build output and `kyne_`-prefixed crate
names) are:

1. `.github/ISSUE_TEMPLATE/config.yml` — `github.com/kyne-lang/kyne`, a
   GitHub organization/repository URL, unrelated to the filesystem
   structure this fix addresses.
2. `ARCHITECTURE.md` §3's note — the sentence stating, correctly, that no
   separate `kyne/` directory exists and pointing to ADR-0002.
3. `KYNE_WORKSPACE_AUDIT.md` — a dated, point-in-time snapshot report.
   **Deliberately left unmodified** — see §6.
4. `WORKSPACE_BOOTSTRAP_REPORT.md` — a dated report of the bootstrap
   milestone's pre-existing state. **Deliberately left unmodified** — see
   §6.

No other reference to the old path, in any form, was found anywhere in
the repository.

---

## 6. Files Deliberately Left Unchanged

`KYNE_WORKSPACE_AUDIT.md` and `WORKSPACE_BOOTSTRAP_REPORT.md` are both
explicitly dated, point-in-time snapshot documents describing repository
state as it verifiably existed on their respective audit dates — *before*
this correction. Their descriptions of `kyne/Foundation.md` and the
`kyne/` directory are accurate historical record, not live, currently-true
cross-references. Editing them to describe the corrected state would
misrepresent what those audits actually found at the time, undermining
their value as a historical baseline. Both are left fully intact; this
report and ADR-0002 are the record of what changed after them. This is
consistent with the task's instruction not to rewrite documentation
beyond repairing broken links and paths — these two files contain no
broken links (they describe a past state accurately) and were therefore
out of scope for editing.

---

## 7. Validation Results

| Check | Result |
|---|---|
| Exactly one `FOUNDATION.md` exists in the repository | **Pass** — confirmed via case-insensitive filesystem search; only `docs/FOUNDATION.md` found. |
| It lives at `docs/FOUNDATION.md` | **Pass.** |
| All Markdown links resolve correctly | **Pass** — a repository-wide script resolved every relative `](path)` link target in every `.md` file (excluding `target/` and `.git/`) against its containing file's directory; zero broken links found after this fix (one false-positive match on a backtick-quoted code example in ADR-0002's own prose, not a real link). |
| No broken relative links | **Pass**, per above. |
| No architectural document contradicts the repository layout | **Pass** — `ARCHITECTURE.md` now states, and the filesystem confirms, that all nine constitutional documents live under `docs/` with no separate `kyne/` directory. |
| No remaining reference to `kyne/Foundation.md` outside historical reports | **Pass**, per §5. |
| Cargo workspace still builds | **Pass** — `cargo build --workspace --all-targets` re-run after the move, unaffected, zero errors. |

---

## 8. Final Repository Tree

```
.
├── .editorconfig
├── .gitattributes
├── .github/
│   ├── ISSUE_TEMPLATE/
│   ├── PULL_REQUEST_TEMPLATE.md
│   ├── SECURITY.md
│   └── workflows/
├── .gitignore
├── ARCHITECTURE.md
├── Cargo.lock
├── Cargo.toml
├── KYNE_WORKSPACE_AUDIT.md
├── LICENSE
├── README.md
├── REPOSITORY_STRUCTURE_FIX_REPORT.md
├── WORKSPACE_BOOTSTRAP_REPORT.md
├── clippy.toml
├── compiler/            (16 crates)
├── docs/
│   ├── FOUNDATION.md
│   ├── LANGUAGE_PRINCIPLES.md
│   ├── LANGUAGE_SPEC.md
│   ├── RUNTIME_MODEL.md
│   ├── COMPILER_ARCHITECTURE.md
│   ├── MEMORY_MODEL.md
│   ├── STANDARD_LIBRARY.md
│   ├── TOOLCHAIN.md
│   ├── GOVERNANCE.md
│   ├── ROADMAP.md
│   ├── adr/
│   │   ├── README.md
│   │   ├── ADR-0001-repository-structure.md
│   │   └── ADR-0002-foundation-md-location.md
│   └── kip/
│       └── README.md
├── examples/
│   ├── canonical/       (6 .kyn files)
│   ├── tutorials/
│   └── regression/
├── rust-toolchain.toml
├── rustfmt.toml
├── scripts/             (5 workflow scripts)
├── stdlib/              (12 crates)
├── tests/               (9 category directories)
├── tools/               (5 crates)
└── website/
```

**There is no nested project root and no `kyne/` directory anywhere in
this tree.**

---

## 9. Confirmation of Conformance to ARCHITECTURE.md

The repository root now matches
[`ARCHITECTURE.md` §3](./ARCHITECTURE.md#3-workspace-layout)'s specified
workspace layout exactly, including the specific detail that document's
[§9](./ARCHITECTURE.md#9-documentation) already fixed but the filesystem
had not yet realized: `FOUNDATION.md` at `docs/FOUNDATION.md`,
indistinguishable in location from the other eight constitutional
documents it stands alongside. No architectural document — `ARCHITECTURE.md`
itself, or any of the nine constitutional documents — describes a
different, contradictory location for `FOUNDATION.md` or references a
`kyne/` subdirectory as part of the intended structure.
