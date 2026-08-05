//! AST-to-HIR lowering: the desugaring pass itself.

use kyne_ast::{
    ContractMember, Expr, IfStmt, Literal, MatchArmBody, MatchStmt, Path, Program, Statement,
    TopLevelDecl,
};

use crate::nodes::*;

/// Lowers a (resolved, typed, semantically valid) [`Program`] into HIR,
/// per docs/COMPILER_ARCHITECTURE.md §12. Total over any AST a prior
/// stage accepted - the same totality guarantee `kyne_ast`'s own
/// lowering makes, for the same reason: nothing here can fail on input
/// that already passed parsing, resolution, type checking, and semantic
/// analysis.
pub fn lower(program: &Program) -> HProgram {
    HProgram {
        items: program.items.iter().map(lower_top_level_decl).collect(),
    }
}

fn lower_top_level_decl(item: &TopLevelDecl) -> HTopLevelDecl {
    match item {
        TopLevelDecl::Contract(c) => HTopLevelDecl::Contract(HContractDecl {
            name: c.name.clone(),
            members: c.members.iter().map(lower_contract_member).collect(),
        }),
        TopLevelDecl::Struct(s) => HTopLevelDecl::Struct(HStructDecl {
            name: s.name.clone(),
            fields: s.fields.clone(),
        }),
        TopLevelDecl::Enum(e) => HTopLevelDecl::Enum(HEnumDecl {
            name: e.name.clone(),
            variants: e.variants.clone(),
        }),
        TopLevelDecl::Error(e) => HTopLevelDecl::Error(e.clone()),
        TopLevelDecl::Event(e) => HTopLevelDecl::Event(HEventDecl {
            name: e.name.clone(),
            params: e.params.clone(),
        }),
        TopLevelDecl::Const(c) => HTopLevelDecl::Const(HConstDecl {
            name: c.name.clone(),
            ty: c.ty.clone(),
            value: lower_expr(&c.value),
        }),
        TopLevelDecl::Fn(f) => HTopLevelDecl::Fn(lower_fn(f)),
    }
}

fn lower_contract_member(member: &ContractMember) -> HContractMember {
    match member {
        ContractMember::State(s) => HContractMember::State(HStateDecl {
            name: s.name.clone(),
            ty: s.ty.clone(),
            init: s.init.as_ref().map(lower_expr),
        }),
        ContractMember::Const(c) => HContractMember::Const(HConstDecl {
            name: c.name.clone(),
            ty: c.ty.clone(),
            value: lower_expr(&c.value),
        }),
        ContractMember::Error(e) => HContractMember::Error(e.clone()),
        ContractMember::Event(e) => HContractMember::Event(HEventDecl {
            name: e.name.clone(),
            params: e.params.clone(),
        }),
        ContractMember::Struct(s) => HContractMember::Struct(HStructDecl {
            name: s.name.clone(),
            fields: s.fields.clone(),
        }),
        ContractMember::Enum(e) => HContractMember::Enum(HEnumDecl {
            name: e.name.clone(),
            variants: e.variants.clone(),
        }),
        ContractMember::Fn(f) => HContractMember::Fn(lower_fn(f)),
    }
}

fn lower_fn(f: &kyne_ast::FnDecl) -> HFnDecl {
    HFnDecl {
        visibility: f.visibility,
        name: f.name.clone(),
        params: f.params.clone(),
        return_type: f.return_type.clone(),
        body: lower_block(&f.body),
    }
}

fn lower_block(block: &kyne_ast::Block) -> HBlock {
    HBlock {
        statements: block.statements.iter().map(lower_stmt).collect(),
    }
}

fn lower_stmt(stmt: &Statement) -> HStmt {
    match stmt {
        Statement::Let(s) => HStmt::Let {
            name: s.name.clone(),
            ty: s.ty.clone(),
            value: lower_expr(&s.value),
        },
        Statement::Assign(s) => HStmt::Assign {
            target: lower_expr(&s.target),
            op: s.op,
            value: lower_expr(&s.value),
        },
        Statement::Auth(e) => HStmt::Auth(lower_expr(e)),
        Statement::Emit(s) => HStmt::Emit {
            event: s.event.clone(),
            args: s.args.iter().map(lower_expr).collect(),
        },
        Statement::Throw(e) => HStmt::Expr(HExpr::Throw(Box::new(lower_expr(e)))),
        Statement::If(if_stmt) => HStmt::Match(lower_if_to_match(if_stmt)),
        Statement::Match(m) => HStmt::Match(lower_match(m)),
        Statement::For(s) => HStmt::For {
            var: s.var.clone(),
            iterable: lower_expr(&s.iterable),
            body: lower_block(&s.body),
        },
        Statement::While(s) => HStmt::While {
            condition: lower_expr(&s.condition),
            body: lower_block(&s.body),
        },
        Statement::Return(e) => {
            HStmt::Expr(HExpr::Return(e.as_ref().map(lower_expr).map(Box::new)))
        }
        Statement::Break => HStmt::Expr(HExpr::Break),
        Statement::Continue => HStmt::Expr(HExpr::Continue),
        Statement::Expr(e) => HStmt::Expr(lower_expr(e)),
    }
}

/// The single desugaring `if` needs: a two-arm match over its boolean
/// condition, `true` first then `false`, so `if`/`match` share one HIR
/// construct instead of two - per this crate's own module documentation.
fn lower_if_to_match(if_stmt: &IfStmt) -> HMatch {
    let else_body: HMatchArmBody = match &if_stmt.else_branch {
        Some(kyne_ast::ElseBranch::Block(b)) => HMatchArmBody::Block(lower_block(b)),
        Some(kyne_ast::ElseBranch::If(nested)) => {
            HMatchArmBody::Expr(HExpr::Match(Box::new(lower_if_to_match(nested))))
        }
        None => HMatchArmBody::Block(HBlock {
            statements: Vec::new(),
        }),
    };
    HMatch {
        subject: lower_expr(&if_stmt.condition),
        arms: vec![
            HMatchArm {
                pattern: kyne_ast::Pattern::Literal(Literal::Bool(true)),
                guard: None,
                body: HMatchArmBody::Block(lower_block(&if_stmt.then_block)),
            },
            HMatchArm {
                pattern: kyne_ast::Pattern::Wildcard,
                guard: None,
                body: else_body,
            },
        ],
    }
}

fn lower_match(m: &MatchStmt) -> HMatch {
    HMatch {
        subject: lower_expr(&m.subject),
        arms: m.arms.iter().map(lower_match_arm).collect(),
    }
}

fn lower_match_arm(arm: &kyne_ast::MatchArm) -> HMatchArm {
    let body = match &arm.body {
        MatchArmBody::Block(b) => HMatchArmBody::Block(lower_block(b)),
        MatchArmBody::Expr(e) => HMatchArmBody::Expr(lower_expr(e)),
        MatchArmBody::Return(e) => {
            HMatchArmBody::Expr(HExpr::Return(e.as_ref().map(lower_expr).map(Box::new)))
        }
        MatchArmBody::Throw(e) => HMatchArmBody::Expr(HExpr::Throw(Box::new(lower_expr(e)))),
        MatchArmBody::Break => HMatchArmBody::Expr(HExpr::Break),
        MatchArmBody::Continue => HMatchArmBody::Expr(HExpr::Continue),
    };
    HMatchArm {
        pattern: arm.pattern.clone(),
        guard: arm.guard.as_ref().map(lower_expr),
        body,
    }
}

fn lower_expr(expr: &Expr) -> HExpr {
    match expr {
        Expr::Literal(l) => HExpr::Literal(l.clone()),
        Expr::Path(p) => HExpr::Path(p.clone()),
        Expr::Binary { left, op, right } => HExpr::Binary {
            left: Box::new(lower_expr(left)),
            op: *op,
            right: Box::new(lower_expr(right)),
        },
        Expr::Unary { op, operand } => HExpr::Unary {
            op: *op,
            operand: Box::new(lower_expr(operand)),
        },
        Expr::Call { callee, args } => HExpr::Call {
            callee: Box::new(lower_expr(callee)),
            args: args.iter().map(lower_expr).collect(),
        },
        Expr::Field { base, name } => HExpr::Field {
            base: Box::new(lower_expr(base)),
            name: name.clone(),
        },
        Expr::MethodCall { base, method, args } => HExpr::MethodCall {
            base: Box::new(lower_expr(base)),
            method: method.clone(),
            args: args.iter().map(lower_expr).collect(),
        },
        Expr::StructLiteral { path, fields } => HExpr::StructLiteral {
            path: path.clone(),
            fields: fields
                .iter()
                .map(|f| HFieldInit {
                    name: f.name.clone(),
                    value: lower_expr(&f.value),
                })
                .collect(),
        },
        Expr::ListLiteral(items) => HExpr::ListLiteral(items.iter().map(lower_expr).collect()),
        Expr::MapLiteral(entries) => HExpr::MapLiteral(
            entries
                .iter()
                .map(|e| HMapEntry {
                    key: lower_expr(&e.key),
                    value: lower_expr(&e.value),
                })
                .collect(),
        ),
        Expr::If(if_stmt) => HExpr::Match(Box::new(lower_if_to_match(if_stmt))),
        Expr::Match(m) => HExpr::Match(Box::new(lower_match(m))),
        Expr::Try { operand } => lower_try(operand),
    }
}

/// Expands `operand?` into its explicit match-and-early-return form, per
/// LANGUAGE_SPEC.md §7.8: evaluate `operand`; if `Ok(v)`, the whole
/// expression's value is `v`; if `Err(e)`, immediately `return Err(e)`
/// from the enclosing function.
fn lower_try(operand: &Expr) -> HExpr {
    let ok_binding = "__kyne_try_value".to_string();
    let err_binding = "__kyne_try_error".to_string();
    HExpr::Match(Box::new(HMatch {
        subject: lower_expr(operand),
        arms: vec![
            HMatchArm {
                pattern: kyne_ast::Pattern::Tuple {
                    path: Path::Ident("Ok".to_string()),
                    patterns: vec![kyne_ast::Pattern::Ident(ok_binding.clone())],
                },
                guard: None,
                body: HMatchArmBody::Expr(HExpr::Path(Path::Ident(ok_binding))),
            },
            HMatchArm {
                pattern: kyne_ast::Pattern::Tuple {
                    path: Path::Ident("Err".to_string()),
                    patterns: vec![kyne_ast::Pattern::Ident(err_binding.clone())],
                },
                guard: None,
                body: HMatchArmBody::Expr(HExpr::Return(Some(Box::new(HExpr::Call {
                    callee: Box::new(HExpr::Path(Path::Ident("Err".to_string()))),
                    args: vec![HExpr::Path(Path::Ident(err_binding))],
                })))),
            },
        ],
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kyne_ast::Program;

    fn lower_source(source: &str) -> HProgram {
        let (cst, diagnostics) = kyne_cst::Cst::parse(source, "test.kyn");
        assert!(
            diagnostics.is_empty(),
            "expected a clean parse, got: {diagnostics:?}"
        );
        let program: Program = kyne_ast::lower(&cst);
        let resolved = kyne_resolver::resolve(&program, "test.kyn");
        assert!(
            resolved.diagnostics.is_empty(),
            "expected clean name resolution, got: {:?}",
            resolved.diagnostics
        );
        lower(&program)
    }

    fn only_fn<'a>(hir: &'a HProgram, name: &str) -> &'a HFnDecl {
        hir.items
            .iter()
            .find_map(|item| match item {
                HTopLevelDecl::Contract(c) => c.members.iter().find_map(|m| match m {
                    HContractMember::Fn(f) if f.name == name => Some(f),
                    _ => None,
                }),
                _ => None,
            })
            .unwrap_or_else(|| panic!("expected a fn named `{name}`"))
    }

    #[test]
    fn if_without_else_lowers_to_a_two_arm_match_with_an_empty_fallback() {
        let hir = lower_source("contract C { fn f(cond: bool) { if cond { auth(cond); } } }\n");
        let f = only_fn(&hir, "f");
        let HStmt::Match(m) = &f.body.statements[0] else {
            panic!("expected a Match statement")
        };
        assert_eq!(m.arms.len(), 2);
        assert_eq!(
            m.arms[0].pattern,
            kyne_ast::Pattern::Literal(kyne_ast::Literal::Bool(true))
        );
        assert_eq!(m.arms[1].pattern, kyne_ast::Pattern::Wildcard);
        let HMatchArmBody::Block(else_block) = &m.arms[1].body else {
            panic!("expected a block")
        };
        assert!(else_block.statements.is_empty());
    }

    #[test]
    fn if_else_if_chain_lowers_to_nested_matches() {
        let hir = lower_source(
            "contract C { fn f(a: bool, b: bool) { if a { } else if b { } else { } } }\n",
        );
        let f = only_fn(&hir, "f");
        let HStmt::Match(outer) = &f.body.statements[0] else {
            panic!("expected a Match statement")
        };
        let HMatchArmBody::Expr(HExpr::Match(inner)) = &outer.arms[1].body else {
            panic!("expected the else-if to lower to a nested HExpr::Match")
        };
        assert_eq!(inner.arms.len(), 2);
    }

    #[test]
    fn if_expression_lowers_to_match_expression() {
        let hir = lower_source("contract C { fn f(cond: bool) -> i64 { let x = if cond { 1 } else { 2 }; return x; } }\n");
        let f = only_fn(&hir, "f");
        let HStmt::Let { value, .. } = &f.body.statements[0] else {
            panic!("expected a Let statement")
        };
        assert!(
            matches!(value, HExpr::Match(_)),
            "expected the if-expression to lower to HExpr::Match, got {value:?}"
        );
    }

    #[test]
    fn try_operator_expands_to_ok_err_match_with_early_return() {
        let hir = lower_source(
            "error E { X }
             contract C {
                fn g() -> Result<i64, E> { return Ok(1); }
                fn f() -> Result<i64, E> {
                    let x = g()?;
                    return Ok(x);
                }
            }\n",
        );
        let f = only_fn(&hir, "f");
        let HStmt::Let { value, .. } = &f.body.statements[0] else {
            panic!("expected a Let statement")
        };
        let HExpr::Match(m) = value else {
            panic!("expected `?` to expand to HExpr::Match, got {value:?}")
        };
        assert_eq!(m.arms.len(), 2);

        let kyne_ast::Pattern::Tuple {
            path: kyne_ast::Path::Ident(ok_name),
            ..
        } = &m.arms[0].pattern
        else {
            panic!("expected the first arm to match `Ok(...)`")
        };
        assert_eq!(ok_name, "Ok");
        assert!(matches!(
            &m.arms[0].body,
            HMatchArmBody::Expr(HExpr::Path(kyne_ast::Path::Ident(_)))
        ));

        let kyne_ast::Pattern::Tuple {
            path: kyne_ast::Path::Ident(err_name),
            ..
        } = &m.arms[1].pattern
        else {
            panic!("expected the second arm to match `Err(...)`")
        };
        assert_eq!(err_name, "Err");
        let HMatchArmBody::Expr(HExpr::Return(Some(returned))) = &m.arms[1].body else {
            panic!("expected the Err arm to be an early HExpr::Return")
        };
        assert!(
            matches!(returned.as_ref(), HExpr::Call { .. }),
            "expected `return Err(e)`, got {returned:?}"
        );
    }

    #[test]
    fn match_arm_bare_return_and_throw_still_lower_to_expressions() {
        let hir = lower_source(
            "enum Status { Approved(address) }
             error E { X }
             contract C {
                fn f(s: Status) -> Result<bool, E> {
                    match s {
                        Status::Approved(_) => return Ok(true),
                        _ => throw E::X,
                    }
                }
            }\n",
        );
        let f = only_fn(&hir, "f");
        let HStmt::Match(m) = &f.body.statements[0] else {
            panic!("expected a Match statement")
        };
        assert!(matches!(
            &m.arms[0].body,
            HMatchArmBody::Expr(HExpr::Return(Some(_)))
        ));
        assert!(matches!(
            &m.arms[1].body,
            HMatchArmBody::Expr(HExpr::Throw(_))
        ));
    }
}
