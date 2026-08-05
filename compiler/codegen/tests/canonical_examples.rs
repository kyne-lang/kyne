//! Confirms `kyne_codegen::generate` renders Counter and Token as
//! `rustfmt`-formatted Rust source text, per this crate's acceptance
//! criteria (issue #17 scopes this crate to exactly these two canonical
//! examples). Each generated file is compared against a checked-in
//! golden-file fixture under `tests/snapshots/`, via `kyne_golden`'s
//! shared `assert_golden` (issue #20) - a diff here means either a real
//! regression or an intentional codegen change; rerun with
//! `UPDATE_GOLDEN=1` to regenerate the fixture, then review the diff
//! and commit it deliberately, never blindly.

fn generate_source(name: &str, source: &str) -> String {
    let (cst, diagnostics) = kyne_cst::Cst::parse(source, name);
    assert!(
        diagnostics.is_empty(),
        "{name}: expected a clean parse, got: {diagnostics:?}"
    );
    let program = kyne_ast::lower(&cst);
    let resolved = kyne_resolver::resolve(&program, name);
    assert!(
        resolved.diagnostics.is_empty(),
        "{name}: expected clean name resolution, got: {:?}",
        resolved.diagnostics
    );
    let typed = kyne_types::check(&program, name);
    assert!(
        typed.diagnostics.is_empty(),
        "{name}: expected clean type checking, got: {:?}",
        typed.diagnostics
    );
    let hir = kyne_hir::lower(&program);
    let rir = kyne_rir::lower(&hir);
    kyne_codegen::generate(&rir).unwrap_or_else(|e| panic!("{name}: codegen failed: {e}"))
}

#[test]
fn counter_matches_golden_file() {
    let generated = generate_source(
        "counter.kyn",
        include_str!("../../../examples/canonical/counter.kyn"),
    );
    kyne_golden::assert_golden("tests/snapshots/counter.rs.snap", &generated);
}

#[test]
fn token_matches_golden_file() {
    let generated = generate_source(
        "token.kyn",
        include_str!("../../../examples/canonical/token.kyn"),
    );
    kyne_golden::assert_golden("tests/snapshots/token.rs.snap", &generated);
}

/// Deterministic generation, per docs/COMPILER_ARCHITECTURE.md §14.4:
/// identical RIR compiled twice MUST produce byte-identical Rust output.
#[test]
fn generation_is_deterministic() {
    let source = include_str!("../../../examples/canonical/token.kyn");
    let first = generate_source("token.kyn", source);
    let second = generate_source("token.kyn", source);
    assert_eq!(first, second);
}

/// Every generated function applies the standard Soroban SDK attribute
/// macros to the appropriate items.
#[test]
fn applies_soroban_attribute_macros() {
    let generated = generate_source(
        "token.kyn",
        include_str!("../../../examples/canonical/token.kyn"),
    );
    assert!(generated.contains("#[contract]"));
    assert!(generated.contains("#[contractimpl]"));
    assert!(generated.contains("#[contracterror]"));
}

/// `total_supply += amount;` must never be printed as a bare Rust `+=`
/// on a nonexistent local variable - it is a `state` field, realized as
/// an explicit storage write.
#[test]
fn compound_state_assignment_is_a_storage_write() {
    let generated = generate_source(
        "token.kyn",
        include_str!("../../../examples/canonical/token.kyn"),
    );
    assert!(!generated.contains("total_supply +="));
    assert!(generated.contains("persistent().set"));
}

/// `+`/`-`/`*`/`/`/`%` MUST never be printed as bare Rust operators,
/// per LANGUAGE_SPEC.md §7.2's checked-arithmetic requirement - a bare
/// operator silently wraps on overflow in a release build.
#[test]
fn arithmetic_is_checked_not_bare() {
    let generated = generate_source(
        "token.kyn",
        include_str!("../../../examples/canonical/token.kyn"),
    );
    assert!(generated.contains("checked_add"));
}

/// `soroban_sdk::Address` (and every other non-`Copy` SDK type) isn't
/// `Copy`, so a parameter used more than once - `to` in `mint`, read
/// once by `balances.set(to, ...)` and again by `emit Mint(to,
/// amount)` - MUST be `.clone()`d at each read after the first, or the
/// generated Rust fails to compile with "use of moved value". Verified
/// against a real `cargo build --target wasm32-unknown-unknown` with
/// the real `soroban-sdk` crate during issue #18's work - see
/// docs/adr/ADR-0012-codegen-implementation.md.
#[test]
fn non_copy_parameter_reused_more_than_once_is_cloned() {
    let generated = generate_source(
        "token.kyn",
        include_str!("../../../examples/canonical/token.kyn"),
    );
    assert!(
        generated.contains("to.clone()"),
        "expected `to` (reused in `mint`) to be cloned at its later reads, got:\n{generated}"
    );
}
