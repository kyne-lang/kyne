//! The "Cargo Build" and "Soroban Build" pipeline stages, per
//! docs/COMPILER_ARCHITECTURE.md §3: "ordinary, unmodified invocations
//! of the unmodified standard Rust and Soroban toolchains ... external
//! tools Kyne depends on rather than components Kyne implements." This
//! module's only job is to invoke `cargo` and the `stellar`/`soroban`
//! CLI exactly as a developer would from a terminal - it introduces no
//! Kyne-specific flag, patch, or workaround into either.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::cargo_toml::{generate_cargo_toml, package_name};
use crate::pipeline::{compile, PipelineError};

/// Every way the build half of the pipeline can fail.
#[derive(Debug)]
pub enum BuildError {
    Pipeline(PipelineError),
    Io(std::io::Error),
    /// `cargo` itself could not be launched (not on `PATH`, or similar),
    /// distinct from [`BuildError::CargoBuildFailed`], which means
    /// `cargo` ran and reported a real build failure.
    CargoNotInvokable(std::io::Error),
    CargoBuildFailed {
        stderr: String,
    },
    /// Neither `stellar` nor `soroban` was found on `PATH`. Per
    /// RUNTIME_MODEL.md §3, the Soroban Build stage is an ordinary
    /// invocation of an external tool this crate does not implement -
    /// this variant exists so a caller can distinguish "the tool isn't
    /// installed" from "the tool ran and rejected the input"
    /// ([`BuildError::SorobanBuildFailed`]), since only the latter
    /// reflects a real defect in the generated crate.
    SorobanToolNotFound,
    SorobanBuildFailed {
        tool: &'static str,
        stderr: String,
    },
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::Pipeline(e) => write!(f, "{e}"),
            BuildError::Io(e) => write!(f, "I/O error writing the generated crate: {e}"),
            BuildError::CargoNotInvokable(e) => write!(f, "could not run `cargo`: {e}"),
            BuildError::CargoBuildFailed { stderr } => {
                write!(f, "`cargo build` failed:\n{stderr}")
            }
            BuildError::SorobanToolNotFound => write!(
                f,
                "neither `stellar` nor `soroban` was found on PATH - install the Stellar CLI \
                 to run the Soroban Build stage (WASM optimization and contract metadata \
                 embedding); the plain `cargo build` WASM is still available"
            ),
            BuildError::SorobanBuildFailed { tool, stderr } => {
                write!(f, "`{tool} contract optimize` failed:\n{stderr}")
            }
        }
    }
}

impl std::error::Error for BuildError {}

/// Everything a successful [`build_project`] run produced.
pub struct BuildOutput {
    pub crate_dir: PathBuf,
    /// The WASM produced by the plain `cargo build --target
    /// wasm32-unknown-unknown` step - the "Cargo Build" stage.
    pub cargo_wasm: PathBuf,
    /// The WASM produced by the "Soroban Build" stage (optimization and
    /// contract metadata embedding) on top of `cargo_wasm`, if the
    /// `stellar`/`soroban` CLI was available to run it. `None` means
    /// only the plain Cargo Build artifact exists - not a build
    /// failure, since neither tool being installed is an environment
    /// fact, not a defect in the generated crate.
    pub soroban_wasm: Option<PathBuf>,
}

/// Writes the generated crate (a real `Cargo.toml` plus `src/lib.rs`)
/// to `out_dir`, per docs/COMPILER_ARCHITECTURE.md §14's "complete,
/// buildable Cargo crate."
pub fn write_crate(
    rust_source: &str,
    contract_name: &str,
    out_dir: &Path,
) -> Result<(), std::io::Error> {
    let src_dir = out_dir.join("src");
    fs::create_dir_all(&src_dir)?;
    fs::write(
        out_dir.join("Cargo.toml"),
        generate_cargo_toml(contract_name),
    )?;
    fs::write(src_dir.join("lib.rs"), rust_source)?;
    Ok(())
}

/// Runs the "Cargo Build" pipeline stage: `cargo build --target
/// wasm32-unknown-unknown [--release]` against the crate at
/// `crate_dir`, and returns the path to the `.wasm` it produced.
pub fn cargo_build(
    crate_dir: &Path,
    contract_name: &str,
    release: bool,
) -> Result<PathBuf, BuildError> {
    let manifest_path = crate_dir.join("Cargo.toml");
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .arg("--manifest-path")
        .arg(&manifest_path);
    if release {
        cmd.arg("--release");
    }
    let output = cmd.output().map_err(BuildError::CargoNotInvokable)?;
    if !output.status.success() {
        return Err(BuildError::CargoBuildFailed {
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }

    let profile_dir = if release { "release" } else { "debug" };
    let wasm_path = crate_dir
        .join("target")
        .join("wasm32-unknown-unknown")
        .join(profile_dir)
        .join(format!("{}.wasm", package_name(contract_name)));
    Ok(wasm_path)
}

/// Runs the "Soroban Build" pipeline stage: WASM optimization and
/// contract metadata embedding on top of the Cargo Build output, via
/// the real `stellar`/`soroban` CLI. Tries `stellar` first (the
/// current official tool name; `soroban-cli` was renamed to
/// `stellar-cli`), falling back to `soroban` for an older toolchain
/// installation.
pub fn soroban_build(cargo_wasm: &Path, out_wasm: &Path) -> Result<PathBuf, BuildError> {
    for tool in ["stellar", "soroban"] {
        let result = Command::new(tool)
            .arg("contract")
            .arg("optimize")
            .arg("--wasm")
            .arg(cargo_wasm)
            .arg("--wasm-out")
            .arg(out_wasm)
            .output();
        match result {
            Ok(output) if output.status.success() => return Ok(out_wasm.to_path_buf()),
            Ok(output) => {
                return Err(BuildError::SorobanBuildFailed {
                    tool,
                    stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                })
            }
            // Tool not on PATH - try the next candidate name rather
            // than failing outright.
            Err(_) => continue,
        }
    }
    Err(BuildError::SorobanToolNotFound)
}

/// Runs the full `.kyn`-source-to-WASM pipeline: [`compile`], write the
/// generated crate to `out_dir`, Cargo Build, then Soroban Build (best-
/// effort - see [`BuildOutput::soroban_wasm`]).
pub fn build_project(
    source: &str,
    file: &str,
    out_dir: &Path,
    release: bool,
) -> Result<BuildOutput, BuildError> {
    let compiled = compile(source, file).map_err(BuildError::Pipeline)?;
    write_crate(&compiled.rust_source, &compiled.contract_name, out_dir).map_err(BuildError::Io)?;
    let cargo_wasm = cargo_build(out_dir, &compiled.contract_name, release)?;

    let soroban_out = out_dir.join(format!(
        "{}.optimized.wasm",
        package_name(&compiled.contract_name)
    ));
    let soroban_wasm = match soroban_build(&cargo_wasm, &soroban_out) {
        Ok(path) => Some(path),
        Err(BuildError::SorobanToolNotFound) => None,
        Err(e) => return Err(e),
    };

    Ok(BuildOutput {
        crate_dir: out_dir.to_path_buf(),
        cargo_wasm,
        soroban_wasm,
    })
}
