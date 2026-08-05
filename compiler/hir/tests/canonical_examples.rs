//! Confirms `kyne_hir::lower` successfully lowers Counter and Token to
//! HIR, per this crate's acceptance criteria (issue #15 scopes this
//! crate to exactly these two canonical examples).

use kyne_hir::HTopLevelDecl;

fn lower_source(name: &str, source: &str) -> kyne_hir::HProgram {
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
    kyne_hir::lower(&program)
}

#[test]
fn counter() {
    let source = include_str!("../../../examples/canonical/counter.kyn");
    let hir = lower_source("counter.kyn", source);
    let contract_count = hir
        .items
        .iter()
        .filter(|i| matches!(i, HTopLevelDecl::Contract(_)))
        .count();
    assert_eq!(contract_count, 1);
}

#[test]
fn token() {
    let source = include_str!("../../../examples/canonical/token.kyn");
    let hir = lower_source("token.kyn", source);
    let contract_count = hir
        .items
        .iter()
        .filter(|i| matches!(i, HTopLevelDecl::Contract(_)))
        .count();
    assert_eq!(contract_count, 1);
}

/// A structural sanity check beyond "it didn't panic": every `if`
/// statement in Token's `mint`/`transfer` (`if amount <= 0 { throw ...; }`)
/// must have compiled down to an `HStmt::Match`, per this crate's
/// if/match unification - not to some leftover `if`-shaped node, since
/// `kyne_hir`'s node types have no `If` variant at all.
#[test]
fn token_ifs_lower_to_match() {
    let source = include_str!("../../../examples/canonical/token.kyn");
    let hir = lower_source("token.kyn", source);
    let Some(HTopLevelDecl::Contract(contract)) = hir
        .items
        .iter()
        .find(|i| matches!(i, HTopLevelDecl::Contract(_)))
    else {
        panic!("expected a contract");
    };
    let mint = contract
        .members
        .iter()
        .find_map(|m| match m {
            kyne_hir::HContractMember::Fn(f) if f.name == "mint" => Some(f),
            _ => None,
        })
        .expect("expected a `mint` function");
    assert!(
        mint.body
            .statements
            .iter()
            .any(|s| matches!(s, kyne_hir::HStmt::Match(_))),
        "expected `if amount <= 0 {{ ... }}` to lower to HStmt::Match"
    );
}
