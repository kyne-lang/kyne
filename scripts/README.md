# scripts/

## Purpose

Minimal wrappers establishing the intended developer workflow, per
[`ARCHITECTURE.md` §18](../ARCHITECTURE.md#18-contributor-workflow). Each
script is a thin shell wrapper over `cargo` — none contains build logic of
its own, consistent with [Compiler First](../ARCHITECTURE.md#2-guiding-philosophy):
these scripts compose the toolchain, they do not reimplement it.

| Script | Purpose |
|---|---|
| [`bootstrap.sh`](./bootstrap.sh) | Verify the local environment can build the workspace (`rustup show`, `cargo build --workspace`). |
| [`format.sh`](./format.sh) | `cargo fmt --all`. |
| [`lint.sh`](./lint.sh) | `cargo clippy --workspace --all-targets -- -D warnings`. |
| [`test.sh`](./test.sh) | `cargo test --workspace`. |
| [`docs.sh`](./docs.sh) | `cargo doc --workspace --no-deps`. |

These scripts operate on **this repository's own Rust implementation**
(the compiler, standard library, and tooling crates). They are distinct
from, and predate, the `kyne` CLI's own eventual subcommands (`kyne fmt`,
`kyne test`, `kyne doc`, per
[`TOOLCHAIN.md` §9](../docs/TOOLCHAIN.md#9-cli-commands)), which format,
test, and document **Kyne source code**, not this repository's Rust code.
The two are easy to conflate by name; they are not the same tool.
