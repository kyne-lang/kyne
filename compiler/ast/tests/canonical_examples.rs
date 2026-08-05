//! Confirms `kyne_ast::lower` succeeds without panicking, and produces a
//! `Program` with exactly one `ContractDecl`, for all six canonical
//! example contracts from docs/LANGUAGE_SPEC.md §16.

use kyne_ast::{lower, TopLevelDecl};
use kyne_cst::Cst;

fn assert_lowers_to_one_contract(name: &str, source: &str) {
    let (cst, diagnostics) = Cst::parse(source, name);
    assert!(
        diagnostics.is_empty(),
        "{name}: expected a clean parse, got: {diagnostics:?}"
    );
    let program = lower(&cst);
    let contract_count = program
        .items
        .iter()
        .filter(|i| matches!(i, TopLevelDecl::Contract(_)))
        .count();
    assert_eq!(
        contract_count, 1,
        "{name}: expected exactly one ContractDecl, found {contract_count}"
    );
}

#[test]
fn counter() {
    assert_lowers_to_one_contract(
        "counter.kyn",
        include_str!("../../../examples/canonical/counter.kyn"),
    );
}

#[test]
fn token() {
    assert_lowers_to_one_contract(
        "token.kyn",
        include_str!("../../../examples/canonical/token.kyn"),
    );
}

#[test]
fn escrow() {
    assert_lowers_to_one_contract(
        "escrow.kyn",
        include_str!("../../../examples/canonical/escrow.kyn"),
    );
}

#[test]
fn marketplace() {
    assert_lowers_to_one_contract(
        "marketplace.kyn",
        include_str!("../../../examples/canonical/marketplace.kyn"),
    );
}

#[test]
fn voting() {
    assert_lowers_to_one_contract(
        "voting.kyn",
        include_str!("../../../examples/canonical/voting.kyn"),
    );
}

#[test]
fn multisig_wallet() {
    assert_lowers_to_one_contract(
        "multisig_wallet.kyn",
        include_str!("../../../examples/canonical/multisig_wallet.kyn"),
    );
}
