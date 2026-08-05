# ADR-0013: Driver's Cargo Build / Soroban Build Integration

**Status:** Accepted
**Date:** 2026-08-05
**Deciders:** Founder
**Type:** Implementation decision — does not amend any constitutional document's normative content.

## Context

Issue #18 required `kyne_driver` to invoke the external `cargo` and
Soroban build tools against the generated crate with no Kyne-specific
modification to either, per
[`RUNTIME_MODEL.md` §3](../RUNTIME_MODEL.md#3-project-lifecycle) and
[`COMPILER_ARCHITECTURE.md` §3](../COMPILER_ARCHITECTURE.md#3-compiler-pipeline)'s
"Cargo Build" / "Soroban Build" stages, and to produce a real, valid
WASM artifact for Counter and Token.

## Decisions

**`kyne_driver` gained three modules**, matching the pipeline's own
stage boundaries rather than one monolithic function: `pipeline.rs`
(`.kyn` source through generated Rust — everything `kyne_codegen` and
earlier stages already do), `cargo_toml.rs` (the generated crate's own
manifest — deferred here from `kyne_codegen` per
[ADR-0012](./ADR-0012-codegen-implementation.md)), and `toolchain.rs`
(the external `cargo`/`stellar`/`soroban` invocations).

**The generated `Cargo.toml`'s release profile matches the standard
Soroban contract scaffold** (`stellar contract init`/`soroban contract
init`'s own generated `Cargo.toml`, a widely-published, stable
convention): `opt-level = "z"`, `lto = true`, `codegen-units = 1`,
`panic = "abort"`, `strip = "symbols"` (minimizing deployed WASM size,
since Soroban bills resource usage partly by contract code size), and
`overflow-checks = true` — a second, build-profile-level guarantee
alongside (not a replacement for) `kyne_codegen`'s own explicit
`checked_*` calls for LANGUAGE_SPEC.md §7.2's requirement. This
specific profile was not independently re-verified against a live
`stellar contract init` invocation in this environment (no `stellar`/
`soroban` CLI was available — see Verification below), so it carries
the same "best-effort, not independently re-derived from a live tool"
caveat ADR-0011/ADR-0012 already established for the Soroban SDK
surface itself.

**Cargo Build and Soroban Build are two distinct function calls**,
matching `COMPILER_ARCHITECTURE.md` §3's own stage split exactly rather
than one `stellar contract build` invocation that happens to do both
internally: `toolchain::cargo_build` runs a plain, ordinary `cargo
build --target wasm32-unknown-unknown [--release]` (the "Cargo Build"
stage — WASM optimization and metadata embedding is explicitly *not*
this step's job); `toolchain::soroban_build` then runs `stellar
contract optimize --wasm <cargo output> --wasm-out <path>` (falling
back to the older `soroban` tool name) on that output (the "Soroban
Build" stage). Both are literal, unmodified invocations of the
external tools — `kyne_driver` passes no Kyne-specific flag to either.

**A missing `stellar`/`soroban` CLI does not fail the whole build.**
`toolchain::build_project` treats `BuildError::SorobanToolNotFound`
specially: `soroban_wasm` in the returned `BuildOutput` is simply
`None`, and the plain Cargo Build artifact (`cargo_wasm`) is still
returned. A real optimize failure (the tool ran and rejected the
input) is still a hard error (`BuildError::SorobanBuildFailed`) — only
the tool's absence is tolerated, since that reflects an environment
fact (is Stellar CLI installed?), not a defect in the generated crate.

## Verification

**A real `cargo build --target wasm32-unknown-unknown --release`
against the real `soroban-sdk` was run for both Counter and Token,
through `kyne_driver`'s own public API** (`tests/build_integration.rs`'s
`counter_builds_to_a_real_wasm_artifact` /
`token_builds_to_a_real_wasm_artifact`, `cargo test -p kyne_driver --
--ignored`), and **both produced a real, valid WASM binary** (confirmed
by reading the `\0asm` magic number from the produced file). This
machine's `~/.cargo/registry` already had `soroban-sdk 23.5.3` and its
full dependency closure cached from prior use, and `rustup target add
wasm32-unknown-unknown` succeeded (a different host,
`static.rust-lang.org`, than the one blocking fresh crate downloads —
see below) — neither is guaranteed in every environment this repository
is cloned into, which is why these two tests are `#[ignore]`d by
default rather than part of the standard suite (`cargo test -p
kyne_driver -- --ignored` to opt in).

**The `stellar`/`soroban` CLI itself was not available in this
environment** and could not be installed (this machine's network
egress allows `index.crates.io` and `github.com` but blocks
`static.crates.io`, the actual crate-download host, with an HTTP 403 —
`cargo install stellar-cli` needs to download crates not already
cached, and a prebuilt binary release was not fetched either). The
Soroban Build stage (`toolchain::soroban_build`) is therefore
implemented and exercised for its "tool not found" path only — its
success path (a real `stellar contract optimize` invocation) is
unverified in this environment. This is a real, current environment
gap, not a design uncertainty; the fix, when this repository is built
somewhere with the Stellar CLI installed, needs no code change, only a
real invocation to confirm the exact `--wasm`/`--wasm-out` flag
spelling this ADR assumed.

## Consequences

- `docs/guides/deployment.md` documents the resulting workflow
  end-to-end, including this environment's specific gap, so a reader
  building somewhere with full tooling knows exactly which parts have
  already been verified and which parts are new to their own run.
- If `stellar contract optimize`'s real flags differ from what
  `toolchain::soroban_build` assumed, the fix is local to that one
  function.
