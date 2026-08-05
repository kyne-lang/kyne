//! Confirms `kyne_semantics::analyze` produces zero false-positive
//! diagnostics for all six canonical example contracts.

fn assert_analyzes_cleanly(name: &str, source: &str) {
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
    let result = kyne_semantics::analyze(&program, name);
    assert!(
        result.diagnostics.is_empty(),
        "{name}: expected no semantic diagnostics, found {}: {:#?}",
        result.diagnostics.len(),
        result.diagnostics,
    );
}

#[test]
fn counter() {
    assert_analyzes_cleanly(
        "counter.kyn",
        include_str!("../../../examples/canonical/counter.kyn"),
    );
}

#[test]
fn token() {
    assert_analyzes_cleanly(
        "token.kyn",
        include_str!("../../../examples/canonical/token.kyn"),
    );
}

#[test]
fn escrow() {
    assert_analyzes_cleanly(
        "escrow.kyn",
        include_str!("../../../examples/canonical/escrow.kyn"),
    );
}

#[test]
fn marketplace() {
    assert_analyzes_cleanly(
        "marketplace.kyn",
        include_str!("../../../examples/canonical/marketplace.kyn"),
    );
}

#[test]
fn voting() {
    assert_analyzes_cleanly(
        "voting.kyn",
        include_str!("../../../examples/canonical/voting.kyn"),
    );
}

#[test]
fn multisig_wallet() {
    assert_analyzes_cleanly(
        "multisig_wallet.kyn",
        include_str!("../../../examples/canonical/multisig_wallet.kyn"),
    );
}
