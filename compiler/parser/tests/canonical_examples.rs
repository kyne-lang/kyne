//! Confirms the parser produces zero diagnostics and a byte-perfect
//! lossless reconstruction for all six canonical example contracts from
//! docs/LANGUAGE_SPEC.md §16, per this crate's acceptance criteria:
//! "Parses all six canonical examples in `examples/canonical/` into a CST
//! with no diagnostics."

fn assert_parses_cleanly(name: &str, source: &str) {
    let (tree, diagnostics) = kyne_parser::parse(source, name);
    assert!(
        diagnostics.is_empty(),
        "{name}: expected no diagnostics, found {}: {:#?}",
        diagnostics.len(),
        diagnostics,
    );
    assert_eq!(
        tree.text(source),
        source,
        "{name}: CST did not losslessly reconstruct the source"
    );
}

#[test]
fn counter() {
    assert_parses_cleanly(
        "counter.kyn",
        include_str!("../../../examples/canonical/counter.kyn"),
    );
}

#[test]
fn token() {
    assert_parses_cleanly(
        "token.kyn",
        include_str!("../../../examples/canonical/token.kyn"),
    );
}

#[test]
fn escrow() {
    assert_parses_cleanly(
        "escrow.kyn",
        include_str!("../../../examples/canonical/escrow.kyn"),
    );
}

#[test]
fn marketplace() {
    assert_parses_cleanly(
        "marketplace.kyn",
        include_str!("../../../examples/canonical/marketplace.kyn"),
    );
}

#[test]
fn voting() {
    assert_parses_cleanly(
        "voting.kyn",
        include_str!("../../../examples/canonical/voting.kyn"),
    );
}

#[test]
fn multisig_wallet() {
    assert_parses_cleanly(
        "multisig_wallet.kyn",
        include_str!("../../../examples/canonical/multisig_wallet.kyn"),
    );
}
