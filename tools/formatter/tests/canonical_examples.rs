//! Confirms `kyne_formatter::format` reproduces all six canonical
//! examples byte-for-byte - per this crate's acceptance criteria, they
//! are already hand-formatted to the canonical style, so this is a
//! direct conformance check.

fn assert_already_canonical(name: &str, source: &str) {
    let formatted = kyne_formatter::format(source)
        .unwrap_or_else(|d| panic!("{name}: expected a clean parse, got: {d:?}"));
    assert_eq!(
        formatted, source,
        "{name}: formatting an already-canonical file must be a no-op"
    );
}

#[test]
fn counter() {
    assert_already_canonical(
        "counter.kyn",
        include_str!("../../../examples/canonical/counter.kyn"),
    );
}

#[test]
fn token() {
    assert_already_canonical(
        "token.kyn",
        include_str!("../../../examples/canonical/token.kyn"),
    );
}

#[test]
fn escrow() {
    assert_already_canonical(
        "escrow.kyn",
        include_str!("../../../examples/canonical/escrow.kyn"),
    );
}

#[test]
fn marketplace() {
    assert_already_canonical(
        "marketplace.kyn",
        include_str!("../../../examples/canonical/marketplace.kyn"),
    );
}

#[test]
fn voting() {
    assert_already_canonical(
        "voting.kyn",
        include_str!("../../../examples/canonical/voting.kyn"),
    );
}

#[test]
fn multisig_wallet() {
    assert_already_canonical(
        "multisig_wallet.kyn",
        include_str!("../../../examples/canonical/multisig_wallet.kyn"),
    );
}
