//! Confirms `kyne_codegen::generate` renders Counter and Token as
//! `rustfmt`-formatted Rust source text, per this crate's acceptance
//! criteria (issue #17 scopes this crate to exactly these two canonical
//! examples). Each generated file is compared against a checked-in
//! golden-file fixture under `tests/snapshots/` - a diff here means
//! either a real regression or an intentional codegen change, in which
//! case the fixture must be reviewed and updated deliberately, never
//! regenerated blindly.

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
    let golden = include_str!("snapshots/counter.rs.snap");
    assert_eq!(
        generated, golden,
        "generated Rust for Counter no longer matches tests/snapshots/counter.rs.snap - \
         if this is an intentional codegen change, review the new output and update the \
         snapshot deliberately"
    );
}

#[test]
fn token_matches_golden_file() {
    let generated = generate_source(
        "token.kyn",
        include_str!("../../../examples/canonical/token.kyn"),
    );
    let golden = include_str!("snapshots/token.rs.snap");
    assert_eq!(
        generated, golden,
        "generated Rust for Token no longer matches tests/snapshots/token.rs.snap - \
         if this is an intentional codegen change, review the new output and update the \
         snapshot deliberately"
    );
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
