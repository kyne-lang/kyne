//! Confirms `kyne_types::check` produces zero false-positive type errors
//! for all six canonical example contracts, per this crate's acceptance
//! criteria.

fn assert_type_checks_cleanly(name: &str, source: &str) {
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
    let result = kyne_types::check(&program, name);
    assert!(
        result.diagnostics.is_empty(),
        "{name}: expected no type-check diagnostics, found {}: {:#?}",
        result.diagnostics.len(),
        result.diagnostics,
    );
}

#[test]
fn counter() {
    assert_type_checks_cleanly(
        "counter.kyn",
        include_str!("../../../examples/canonical/counter.kyn"),
    );
}

#[test]
fn token() {
    assert_type_checks_cleanly(
        "token.kyn",
        include_str!("../../../examples/canonical/token.kyn"),
    );
}

#[test]
fn escrow() {
    assert_type_checks_cleanly(
        "escrow.kyn",
        include_str!("../../../examples/canonical/escrow.kyn"),
    );
}

#[test]
fn marketplace() {
    assert_type_checks_cleanly(
        "marketplace.kyn",
        include_str!("../../../examples/canonical/marketplace.kyn"),
    );
}

#[test]
fn voting() {
    assert_type_checks_cleanly(
        "voting.kyn",
        include_str!("../../../examples/canonical/voting.kyn"),
    );
}

#[test]
fn multisig_wallet() {
    assert_type_checks_cleanly(
        "multisig_wallet.kyn",
        include_str!("../../../examples/canonical/multisig_wallet.kyn"),
    );
}
