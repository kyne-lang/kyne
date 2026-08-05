//! Confirms the lexer produces zero `Error` tokens for any of the six
//! canonical example contracts from docs/LANGUAGE_SPEC.md §16, per this
//! crate's acceptance criteria: "Successfully tokenizes all six
//! canonical examples in `examples/canonical/`."

use kyne_lexer::{lex, TokenKind};

fn assert_no_lex_errors(name: &str, source: &str) {
    let tokens = lex(source);
    let errors: Vec<_> = tokens
        .iter()
        .filter(|t| t.kind == TokenKind::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "{name}: expected no Error tokens, found {} at {:?}",
        errors.len(),
        errors.iter().map(|t| t.span).collect::<Vec<_>>(),
    );
}

#[test]
fn counter() {
    assert_no_lex_errors(
        "counter.kyn",
        include_str!("../../../examples/canonical/counter.kyn"),
    );
}

#[test]
fn token() {
    assert_no_lex_errors(
        "token.kyn",
        include_str!("../../../examples/canonical/token.kyn"),
    );
}

#[test]
fn escrow() {
    assert_no_lex_errors(
        "escrow.kyn",
        include_str!("../../../examples/canonical/escrow.kyn"),
    );
}

#[test]
fn marketplace() {
    assert_no_lex_errors(
        "marketplace.kyn",
        include_str!("../../../examples/canonical/marketplace.kyn"),
    );
}

#[test]
fn voting() {
    assert_no_lex_errors(
        "voting.kyn",
        include_str!("../../../examples/canonical/voting.kyn"),
    );
}

#[test]
fn multisig_wallet() {
    assert_no_lex_errors(
        "multisig_wallet.kyn",
        include_str!("../../../examples/canonical/multisig_wallet.kyn"),
    );
}
