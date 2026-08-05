//! Confirms `kyne_rir::lower` successfully lowers Counter and Token to
//! RIR, per this crate's acceptance criteria (issue #16 scopes this
//! crate to exactly these two canonical examples).

use kyne_rir::{RExpr, RStmt};

fn lower_source(name: &str, source: &str) -> kyne_rir::RProgram {
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
    kyne_rir::lower(&hir)
}

#[test]
fn counter() {
    let rir = lower_source(
        "counter.kyn",
        include_str!("../../../examples/canonical/counter.kyn"),
    );
    assert_eq!(rir.contract.name, "Counter");
    assert_eq!(rir.contract.state_fields.len(), 1);
    assert_eq!(rir.contract.state_fields[0].name, "count");
    assert_eq!(rir.contract.state_fields[0].ty, kyne_rir::RType::I64);
}

#[test]
fn token() {
    let rir = lower_source(
        "token.kyn",
        include_str!("../../../examples/canonical/token.kyn"),
    );
    assert_eq!(rir.contract.name, "Token");
    assert_eq!(rir.contract.state_fields.len(), 2);
    assert_eq!(rir.contract.consts.len(), 3);
    assert_eq!(rir.contract.errors.len(), 1);
    assert_eq!(rir.contract.events.len(), 2);
    assert_eq!(rir.contract.functions.len(), 4);
}

/// `address` parameters map to `soroban_sdk::Address`, per this crate's
/// documented mapping.
#[test]
fn token_address_params_map_to_r_type_address() {
    let rir = lower_source(
        "token.kyn",
        include_str!("../../../examples/canonical/token.kyn"),
    );
    let mint = rir
        .contract
        .functions
        .iter()
        .find(|f| f.name == "mint")
        .expect("expected a `mint` function");
    assert!(mint
        .params
        .iter()
        .any(|p| p.name == "to" && p.ty == kyne_rir::RType::Address));
}

/// `balances: map<address, i128>` maps to `soroban_sdk::Map<Address, i128>`.
#[test]
fn token_map_state_field_maps_to_r_type_map() {
    let rir = lower_source(
        "token.kyn",
        include_str!("../../../examples/canonical/token.kyn"),
    );
    let balances = rir
        .contract
        .state_fields
        .iter()
        .find(|f| f.name == "balances")
        .expect("expected `balances`");
    assert_eq!(
        balances.ty,
        kyne_rir::RType::Map(
            Box::new(kyne_rir::RType::Address),
            Box::new(kyne_rir::RType::I128)
        )
    );
}

/// `auth(admin)` in `mint` realizes as `RStmt::RequireAuth`.
#[test]
fn token_auth_becomes_require_auth() {
    let rir = lower_source(
        "token.kyn",
        include_str!("../../../examples/canonical/token.kyn"),
    );
    let mint = rir
        .contract
        .functions
        .iter()
        .find(|f| f.name == "mint")
        .expect("expected a `mint` function");
    assert!(mint
        .body
        .statements
        .iter()
        .any(|s| matches!(s, RStmt::RequireAuth(_))));
}

/// `total_supply += amount;` (a `state` field) realizes as a storage
/// write whose value is an explicit read-modify-write expression, not a
/// Rust-level `+=` on a nonexistent local variable.
#[test]
fn token_compound_state_assignment_expands_to_storage_set() {
    let rir = lower_source(
        "token.kyn",
        include_str!("../../../examples/canonical/token.kyn"),
    );
    let mint = rir
        .contract
        .functions
        .iter()
        .find(|f| f.name == "mint")
        .expect("expected a `mint` function");
    let storage_set = mint.body.statements.iter().find_map(|s| match s {
        RStmt::StorageSet { key, value } if key == "total_supply" => Some(value),
        _ => None,
    });
    let Some(RExpr::Binary { left, .. }) = storage_set else {
        panic!("expected `total_supply += amount` to lower to a StorageSet of a Binary expression");
    };
    assert!(matches!(left.as_ref(), RExpr::StorageGet { key, .. } if key == "total_supply"));
}

/// `balances: map<address, i128>` has no literal initializer in
/// `token.kyn`, so per ADR-0009 (inherited here) it gets an implicit
/// empty-map default, synthesized for its `StorageGet` occurrences.
#[test]
fn token_balances_get_calls_carry_an_empty_map_default() {
    let rir = lower_source(
        "token.kyn",
        include_str!("../../../examples/canonical/token.kyn"),
    );
    let balance_of = rir
        .contract
        .functions
        .iter()
        .find(|f| f.name == "balance_of")
        .expect("expected `balance_of`");
    let found_storage_get = find_storage_get(&balance_of.body, "balances");
    let Some(RExpr::StorageGet { default, .. }) = found_storage_get else {
        panic!("expected a StorageGet for `balances`");
    };
    assert!(matches!(default.as_deref(), Some(RExpr::MapLiteral(items)) if items.is_empty()));
}

fn find_storage_get<'a>(block: &'a kyne_rir::RBlock, key: &str) -> Option<&'a RExpr> {
    fn search_expr<'a>(expr: &'a RExpr, key: &str) -> Option<&'a RExpr> {
        if let RExpr::StorageGet { key: k, .. } = expr {
            if k == key {
                return Some(expr);
            }
        }
        match expr {
            RExpr::MethodCall { base, args, .. } => {
                search_expr(base, key).or_else(|| args.iter().find_map(|a| search_expr(a, key)))
            }
            RExpr::Call { callee, args } => {
                search_expr(callee, key).or_else(|| args.iter().find_map(|a| search_expr(a, key)))
            }
            RExpr::Binary { left, right, .. } => {
                search_expr(left, key).or_else(|| search_expr(right, key))
            }
            RExpr::Return(Some(e)) => search_expr(e, key),
            _ => None,
        }
    }
    for stmt in &block.statements {
        let found = match stmt {
            RStmt::Expr(e) => search_expr(e, key),
            RStmt::Let { value, .. } => search_expr(value, key),
            _ => None,
        };
        if found.is_some() {
            return found;
        }
    }
    None
}
