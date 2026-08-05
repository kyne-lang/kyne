//! Generates the output crate's `Cargo.toml`, per
//! docs/COMPILER_ARCHITECTURE.md §14: "its `Cargo.toml` declares the
//! same dependencies any hand-written Soroban contract crate would."
//! Deferred from `kyne_codegen` (issue #17) to this crate (issue #18)
//! per docs/adr/ADR-0012-codegen-implementation.md, since this is the
//! first stage that actually needs a real, buildable manifest to
//! invoke `cargo`/the Soroban toolchain against.

/// The generated crate's package name: the contract's own name,
/// lowercased. Used both here and by [`crate::toolchain`] to locate the
/// `.wasm` artifact `cargo` produces (which uses the package name,
/// `-` replaced with `_`, as its filename stem) - since a Kyne contract
/// name is a `PascalCase` identifier with no `-` in it
/// (LANGUAGE_SPEC.md §3.1), that substitution never actually applies
/// here, but the two call sites still share this one function so they
/// can never disagree.
pub fn package_name(contract_name: &str) -> String {
    contract_name.to_lowercase()
}

/// The `soroban-sdk` version this crate targets, per
/// docs/adr/ADR-0011-rir-implementation.md's Soroban SDK mapping - kept
/// in one place so it is easy to bump deliberately rather than having
/// call sites disagree.
const SOROBAN_SDK_VERSION: &str = "23.5.3";

/// The release profile every official Soroban contract scaffold
/// (`stellar contract init`/`soroban contract init`) generates: `opt-
/// level = "z"` and `lto = true` minimize the deployed WASM's size
/// (Soroban bills resource usage partly by contract code size),
/// `panic = "abort"` avoids linking in Rust's unwinding machinery
/// (meaningless in a WASM guest that has no caller to unwind to
/// anyway), and `overflow-checks = true` is a second, build-profile-
/// level guarantee alongside (not a replacement for -
/// see docs/adr/ADR-0012-codegen-implementation.md) `kyne_codegen`'s
/// own explicit `checked_*` calls for LANGUAGE_SPEC.md §7.2's
/// no-silent-wraparound requirement.
fn release_profile() -> &'static str {
    "[profile.release]\nopt-level = \"z\"\noverflow-checks = true\ndebug = 0\nstrip = \"symbols\"\ndebug-assertions = false\npanic = \"abort\"\ncodegen-units = 1\nlto = true\n"
}

/// Renders the generated crate's `Cargo.toml`. `contract_name` is the
/// contract's own declared name (`RContract.name`) - used verbatim as
/// the crate-type struct name in generated Rust ([`kyne_codegen`]) and,
/// lowercased, as this package's own name.
pub fn generate_cargo_toml(contract_name: &str) -> String {
    let package = package_name(contract_name);
    format!(
        "[package]\nname = \"{package}\"\nversion = \"0.0.0\"\nedition = \"2021\"\npublish = false\n\n[lib]\ncrate-type = [\"cdylib\"]\n\n[dependencies]\nsoroban-sdk = \"{SOROBAN_SDK_VERSION}\"\n\n{}",
        release_profile()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_name_is_lowercased() {
        assert_eq!(package_name("Counter"), "counter");
        assert_eq!(package_name("Token"), "token");
    }

    #[test]
    fn generated_manifest_is_valid_toml_shape() {
        let manifest = generate_cargo_toml("Counter");
        assert!(manifest.contains("name = \"counter\""));
        assert!(manifest.contains("soroban-sdk"));
        assert!(manifest.contains("crate-type = [\"cdylib\"]"));
        assert!(manifest.contains("[profile.release]"));
    }
}
