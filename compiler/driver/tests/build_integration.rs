//! Confirms `kyne_driver`'s pipeline orchestration and build integration,
//! per issue #18. Two tiers:
//!
//! - Fast, hermetic tests (`compile`, `write_crate`) that always run.
//! - `#[ignore]`d tests that invoke the real `cargo`/`wasm32-unknown-
//!   unknown` toolchain and the real `soroban-sdk` crate - these need a
//!   populated `~/.cargo/registry` and the `wasm32-unknown-unknown`
//!   target installed, which is not guaranteed in every environment
//!   this repository is cloned into, so they are opt-in
//!   (`cargo test -p kyne_driver -- --ignored`) rather than part of the
//!   default suite. They were run and passed during issue #18's own
//!   verification - see docs/adr/ADR-0013-driver-build-integration.md.

use std::fs;
use std::path::PathBuf;

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kyne_driver_test_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

#[test]
fn compile_counter_produces_rust_source() {
    let out = kyne_driver::compile(
        include_str!("../../../examples/canonical/counter.kyn"),
        "counter.kyn",
    )
    .unwrap_or_else(|e| panic!("expected Counter to compile cleanly, got: {e}"));
    assert_eq!(out.contract_name, "Counter");
    assert!(out.rust_source.contains("#[contract]"));
}

#[test]
fn compile_token_produces_rust_source() {
    let out = kyne_driver::compile(
        include_str!("../../../examples/canonical/token.kyn"),
        "token.kyn",
    )
    .unwrap_or_else(|e| panic!("expected Token to compile cleanly, got: {e}"));
    assert_eq!(out.contract_name, "Token");
    assert!(out.rust_source.contains("#[contract]"));
}

/// Per RUNTIME_MODEL.md §3: "A project that fails any check here never
/// proceeds to a later stage." A parse error must stop at the `parse`
/// stage, never reach codegen.
#[test]
fn compile_stops_at_the_first_failing_stage() {
    let err = kyne_driver::compile("contract {{{ not valid kyne", "bad.kyn")
        .expect_err("expected a syntax error, not a successful compile");
    match err {
        kyne_driver::PipelineError::Diagnostics { stage, diagnostics } => {
            assert_eq!(stage, "parse");
            assert!(!diagnostics.is_empty());
        }
        other => panic!("expected a Diagnostics failure at the parse stage, got: {other}"),
    }
}

#[test]
fn write_crate_writes_a_real_cargo_toml_and_lib_rs() {
    let dir = scratch_dir("write_crate");
    kyne_driver::write_crate("#![no_std]\n", "Counter", &dir).expect("write_crate failed");

    let manifest = fs::read_to_string(dir.join("Cargo.toml")).expect("Cargo.toml missing");
    assert!(manifest.contains("name = \"counter\""));
    assert!(manifest.contains("soroban-sdk"));

    let lib_rs = fs::read_to_string(dir.join("src").join("lib.rs")).expect("src/lib.rs missing");
    assert_eq!(lib_rs, "#![no_std]\n");

    let _ = fs::remove_dir_all(&dir);
}

/// Real, verified `cargo build --target wasm32-unknown-unknown
/// --release` of Counter's generated crate against the real
/// `soroban-sdk`, run and passed during issue #18's own verification.
/// Ignored by default - see this file's module doc comment.
#[test]
#[ignore = "requires a populated cargo registry cache and the wasm32-unknown-unknown target; run with `cargo test -p kyne_driver -- --ignored`"]
fn counter_builds_to_a_real_wasm_artifact() {
    let dir = scratch_dir("counter_build");
    let output = kyne_driver::build_project(
        include_str!("../../../examples/canonical/counter.kyn"),
        "counter.kyn",
        &dir,
        true,
    )
    .unwrap_or_else(|e| panic!("expected a real cargo build to succeed, got: {e}"));

    let wasm_bytes = fs::read(&output.cargo_wasm).expect("expected a .wasm file to exist");
    assert_eq!(
        &wasm_bytes[0..4],
        b"\0asm",
        "expected a valid WASM magic number"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Same as `counter_builds_to_a_real_wasm_artifact`, for Token - the
/// contract that exercises `StorageMutate`, checked arithmetic, and
/// non-`Copy` value cloning, all of which needed real fixes during
/// issue #17/#18's own verification.
#[test]
#[ignore = "requires a populated cargo registry cache and the wasm32-unknown-unknown target; run with `cargo test -p kyne_driver -- --ignored`"]
fn token_builds_to_a_real_wasm_artifact() {
    let dir = scratch_dir("token_build");
    let output = kyne_driver::build_project(
        include_str!("../../../examples/canonical/token.kyn"),
        "token.kyn",
        &dir,
        true,
    )
    .unwrap_or_else(|e| panic!("expected a real cargo build to succeed, got: {e}"));

    let wasm_bytes = fs::read(&output.cargo_wasm).expect("expected a .wasm file to exist");
    assert_eq!(
        &wasm_bytes[0..4],
        b"\0asm",
        "expected a valid WASM magic number"
    );

    let _ = fs::remove_dir_all(&dir);
}
