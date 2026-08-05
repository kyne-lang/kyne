//! Confirms `kyne_resolver::resolve` produces zero false-positive
//! unresolved-name errors for all six canonical example contracts, per
//! this crate's acceptance criteria.

fn assert_resolves_cleanly(name: &str, source: &str) {
    let (cst, diagnostics) = kyne_cst::Cst::parse(source, name);
    assert!(
        diagnostics.is_empty(),
        "{name}: expected a clean parse, got: {diagnostics:?}"
    );
    let program = kyne_ast::lower(&cst);
    let result = kyne_resolver::resolve(&program, name);
    assert!(
        result.diagnostics.is_empty(),
        "{name}: expected no name-resolution diagnostics, found {}: {:#?}",
        result.diagnostics.len(),
        result.diagnostics,
    );
}

#[test]
fn counter() {
    assert_resolves_cleanly(
        "counter.kyn",
        include_str!("../../../examples/canonical/counter.kyn"),
    );
}

#[test]
fn token() {
    assert_resolves_cleanly(
        "token.kyn",
        include_str!("../../../examples/canonical/token.kyn"),
    );
}

#[test]
fn escrow() {
    assert_resolves_cleanly(
        "escrow.kyn",
        include_str!("../../../examples/canonical/escrow.kyn"),
    );
}

#[test]
fn marketplace() {
    assert_resolves_cleanly(
        "marketplace.kyn",
        include_str!("../../../examples/canonical/marketplace.kyn"),
    );
}

#[test]
fn voting() {
    assert_resolves_cleanly(
        "voting.kyn",
        include_str!("../../../examples/canonical/voting.kyn"),
    );
}

#[test]
fn multisig_wallet() {
    assert_resolves_cleanly(
        "multisig_wallet.kyn",
        include_str!("../../../examples/canonical/multisig_wallet.kyn"),
    );
}
