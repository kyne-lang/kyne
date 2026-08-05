//! `kyne_driver` - pipeline orchestration.
//!
//! Specified by docs/COMPILER_ARCHITECTURE.md §3 (Compiler Pipeline) and
//! §20 (Repository Architecture). The only crate permitted to depend on
//! every pipeline stage; sequences a `.kyn` project through the pipeline
//! on behalf of `kyne_cli` and every other consumer named in
//! `ARCHITECTURE.md` §5.
//!
//! ## Status
//!
//! [`check`] runs `source` through the CST stage only (lexing and
//! parsing), matching `kyne_cli`'s current `kyne check` implementation
//! (issue #11) - kept intentionally minimal and CST-only for that
//! narrow, fast-path use.
//!
//! [`pipeline::compile`] runs the fuller pipeline (parse through
//! `kyne_codegen`) to generated Rust text, and
//! [`toolchain::build_project`] takes that the rest of the way per
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
//! so neither [`check`] nor [`pipeline::compile`] invokes them yet.
//! `kyne_cli`'s `check` subcommand and `pipeline::compile` are
//! currently two independent entry points rather than one sharing the
//! other's stages; unifying them (`check` reusing `pipeline::compile`'s
//! earlier stages once both need the same diagnostics) is natural
//! follow-up work, not done here.
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

use kyne_cst::Cst;
use kyne_diagnostics::Diagnostic;

pub use cargo_toml::{generate_cargo_toml, package_name};
pub use pipeline::{compile, CompileOutput, PipelineError};
pub use toolchain::{
    build_project, cargo_build, soroban_build, write_crate, BuildError, BuildOutput,
};

/// Runs `source` through the CST stage only (lexing and parsing) and
/// returns whatever diagnostics it produced - the minimal, fast-path
/// entry point `kyne_cli`'s `kyne check` subcommand (issue #11) uses.
/// An empty result means `source` parsed cleanly - not yet a full
/// guarantee of well-formedness; see [`pipeline::compile`] for the
/// fuller pipeline.
pub fn check(source: &str, file: &str) -> Vec<Diagnostic> {
    let (_cst, diagnostics) = Cst::parse(source, file);
    diagnostics
}
