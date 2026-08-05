//! `kyne_driver` - pipeline orchestration.
//!
//! Specified by docs/COMPILER_ARCHITECTURE.md section 3 (Compiler
//! Pipeline) and section 20 (Repository Architecture). The only crate
//! permitted to depend on every pipeline stage; sequences a .kyn project
//! through the full pipeline on behalf of kyne_cli and every other
//! consumer named in ARCHITECTURE.md section 5.
//!
//! ## Scope (per issue #18)
//!
//! [`pipeline::compile`] runs `.kyn` source through the full compiler
//! pipeline to generated Rust text. [`toolchain::build_project`] takes
//! that Rust text the rest of the way per
//! [`RUNTIME_MODEL.md` §3](../../docs/RUNTIME_MODEL.md#3-project-lifecycle):
//! writes a real, buildable Cargo crate to disk, then invokes the
//! external `cargo` and `stellar`/`soroban` toolchains against it
//! exactly as a developer would from a terminal - the "Cargo Build" and
//! "Soroban Build" pipeline stages
//! ([`COMPILER_ARCHITECTURE.md` §3](../../docs/COMPILER_ARCHITECTURE.md#3-compiler-pipeline)),
//! introducing no Kyne-specific fork of either.
//!
//! `kyne_security` (the Security Analyzer) and `kyne_optimizer` have no
//! implementation yet - both are deferred to Phase 2, per
//! [`ROADMAP.md` §7](../../docs/ROADMAP.md#7-compiler-bootstrap-order) -
//! so [`pipeline::compile`] does not yet invoke them.
//!
//! Verified (issue #18): a real `cargo build --target
//! wasm32-unknown-unknown --release` of both Counter's and Token's
//! generated crate succeeds against the real `soroban-sdk`. The
//! `stellar`/`soroban` CLI itself was not available to verify the
//! Soroban Build stage in this environment - see
//! docs/adr/ADR-0013-driver-build-integration.md and
//! docs/guides/deployment.md.

mod cargo_toml;
mod pipeline;
mod toolchain;

pub use cargo_toml::{generate_cargo_toml, package_name};
pub use pipeline::{compile, CompileOutput, PipelineError};
pub use toolchain::{
    build_project, cargo_build, soroban_build, write_crate, BuildError, BuildOutput,
};
