//! `kyne_ast` - the Abstract Syntax Tree and its shared visitor abstraction.
//!
//! Specified by docs/COMPILER_ARCHITECTURE.md §7. A syntax-shaped,
//! trivia-free tree mirroring docs/LANGUAGE_SPEC.md §2's grammar
//! productions directly, produced by a total lowering from `kyne_cst`'s
//! tree and consumed by every semantic stage.

mod lower;
mod nodes;
mod visit;

pub use lower::lower;
pub use nodes::*;
pub use visit::{
    walk_block, walk_const_decl, walk_contract_decl, walk_contract_member, walk_expr, walk_fn_decl,
    walk_if_stmt, walk_match_stmt, walk_program, walk_state_decl, walk_stmt, walk_top_level_decl,
    Visitor,
};

#[cfg(test)]
mod tests {
    use super::*;
    use kyne_cst::Cst;

    fn lower_source(source: &str) -> Program {
        let (cst, diagnostics) = Cst::parse(source, "test.kyn");
        assert!(
            diagnostics.is_empty(),
            "expected clean parse, got: {diagnostics:?}"
        );
        lower(&cst)
    }

    fn only_contract(program: &Program) -> &ContractDecl {
        match program.items.as_slice() {
            [TopLevelDecl::Contract(c)] => c,
            other => panic!("expected exactly one ContractDecl, got {other:?}"),
        }
    }

    #[test]
    fn lowers_imports() {
        let program = lower_source("use math.safe_math;\nuse collections.{List, Map};\n");
        assert_eq!(program.imports.len(), 2);
        assert_eq!(program.imports[0].path, vec!["math", "safe_math"]);
        assert!(program.imports[0].names.is_empty());
        assert_eq!(program.imports[1].path, vec!["collections"]);
        assert_eq!(program.imports[1].names, vec!["List", "Map"]);
    }

    #[test]
    fn lowers_state_const_and_types() {
        let source = "\
contract C {
    state a: i64 = 0;
    state b: list<address>;
    state c: map<address, i128>;
    state d: bytes<32>;
    state e: Option<i64>;
    state f: Result<i64, E>;
    const MAX: i64 = 100;
}
";
        let program = lower_source(source);
        let contract = only_contract(&program);
        let states: Vec<&StateDecl> = contract
            .members
            .iter()
            .filter_map(|m| match m {
                ContractMember::State(s) => Some(s),
                _ => None,
            })
            .collect();
        assert_eq!(states.len(), 6);
        assert_eq!(states[0].name, "a");
        assert_eq!(states[0].ty, Type::Named("i64".into()));
        assert!(matches!(&states[0].init, Some(Expr::Literal(Literal::Int(n))) if n == "0"));
        assert_eq!(
            states[1].ty,
            Type::List(Box::new(Type::Named("address".into())))
        );
        assert_eq!(
            states[2].ty,
            Type::Map(
                Box::new(Type::Named("address".into())),
                Box::new(Type::Named("i128".into()))
            )
        );
        assert_eq!(states[3].ty, Type::Bytes("32".into()));
        assert_eq!(
            states[4].ty,
            Type::Option(Box::new(Type::Named("i64".into())))
        );
        assert_eq!(
            states[5].ty,
            Type::Result(
                Box::new(Type::Named("i64".into())),
                Box::new(Type::Named("E".into()))
            )
        );

        let ConstDecl { name, value, .. } = contract
            .members
            .iter()
            .find_map(|m| match m {
                ContractMember::Const(c) => Some(c),
                _ => None,
            })
            .unwrap();
        assert_eq!(name, "MAX");
        assert!(matches!(value, Expr::Literal(Literal::Int(n)) if n == "100"));
    }

    #[test]
    fn lowers_fn_visibility() {
        let source = "contract C { public fn a() {} internal fn b() {} fn c() {} }\n";
        let program = lower_source(source);
        let contract = only_contract(&program);
        let visibilities: Vec<Visibility> = contract
            .members
            .iter()
            .filter_map(|m| match m {
                ContractMember::Fn(f) => Some(f.visibility),
                _ => None,
            })
            .collect();
        assert_eq!(
            visibilities,
            vec![
                Visibility::Public,
                Visibility::Internal,
                Visibility::Private
            ]
        );
    }

    #[test]
    fn redundant_parens_collapse_during_lowering() {
        let with_parens = lower_source("contract C { fn f() { let x = (1 + 2); } }\n");
        let without_parens = lower_source("contract C { fn f() { let x = 1 + 2; } }\n");
        assert_eq!(
            with_parens, without_parens,
            "parenthesized and unparenthesized expressions must lower identically"
        );
    }

    #[test]
    fn field_init_shorthand_is_expanded() {
        let program = lower_source(
            "contract C { fn f(seller: address) { let x = Listing { seller, price: 1 }; } }\n",
        );
        let contract = only_contract(&program);
        let body = &contract
            .members
            .iter()
            .find_map(|m| match m {
                ContractMember::Fn(f) => Some(f),
                _ => None,
            })
            .unwrap()
            .body;
        let Statement::Let(let_stmt) = &body.statements[0] else {
            panic!("expected a let statement")
        };
        let Expr::StructLiteral { fields, .. } = &let_stmt.value else {
            panic!("expected a struct literal")
        };
        assert_eq!(fields[0].name, "seller");
        // Shorthand `seller` must expand to the explicit `seller: seller` form.
        assert_eq!(fields[0].value, Expr::Path(Path::Ident("seller".into())));
    }

    #[test]
    fn string_escapes_are_decoded() {
        let program = lower_source(r#"contract C { const S: string = "a\nb\t\u{41}"; }"#);
        let contract = only_contract(&program);
        let ConstDecl { value, .. } = contract
            .members
            .iter()
            .find_map(|m| match m {
                ContractMember::Const(c) => Some(c),
                _ => None,
            })
            .unwrap();
        assert_eq!(value, &Expr::Literal(Literal::Str("a\nb\tA".to_string())));
    }

    #[test]
    fn match_arm_bare_return_and_throw_lower_correctly() {
        let source = "\
contract C {
    fn f(status: Status, admin: address) -> bool {
        match status {
            Status::Approved(by) if by == admin => return true,
            Status::Rejected { reason } => throw ProcessError::Rejected,
            _ => return false,
        }
    }
}
";
        let program = lower_source(source);
        let contract = only_contract(&program);
        let body = &contract
            .members
            .iter()
            .find_map(|m| match m {
                ContractMember::Fn(f) => Some(f),
                _ => None,
            })
            .unwrap()
            .body;
        let Statement::Match(match_stmt) = &body.statements[0] else {
            panic!("expected a match statement")
        };
        assert_eq!(match_stmt.arms.len(), 3);
        assert!(matches!(
            &match_stmt.arms[0].body,
            MatchArmBody::Return(Some(_))
        ));
        assert!(match_stmt.arms[0].guard.is_some());
        assert!(matches!(&match_stmt.arms[1].body, MatchArmBody::Throw(_)));
        assert!(matches!(&match_stmt.arms[2].pattern, Pattern::Wildcard));
        assert!(matches!(
            &match_stmt.arms[2].body,
            MatchArmBody::Return(Some(_))
        ));
    }

    #[test]
    fn if_and_else_if_chain_lowers_correctly() {
        let source = "contract C { fn f(x: i64) { if x == 1 { } else if x == 2 { } else { } } }\n";
        let program = lower_source(source);
        let contract = only_contract(&program);
        let body = &contract
            .members
            .iter()
            .find_map(|m| match m {
                ContractMember::Fn(f) => Some(f),
                _ => None,
            })
            .unwrap()
            .body;
        let Statement::If(if_stmt) = &body.statements[0] else {
            panic!("expected an if statement")
        };
        match &if_stmt.else_branch {
            Some(ElseBranch::If(nested)) => {
                assert!(matches!(nested.else_branch, Some(ElseBranch::Block(_))))
            }
            other => panic!("expected an else-if chain, got {other:?}"),
        }
    }

    #[test]
    fn binary_operators_lower_to_the_right_variant() {
        let cases: &[(&str, BinaryOp)] = &[
            ("a + b", BinaryOp::Add),
            ("a - b", BinaryOp::Sub),
            ("a * b", BinaryOp::Mul),
            ("a / b", BinaryOp::Div),
            ("a % b", BinaryOp::Rem),
            ("a == b", BinaryOp::Eq),
            ("a != b", BinaryOp::Ne),
            ("a < b", BinaryOp::Lt),
            ("a > b", BinaryOp::Gt),
            ("a <= b", BinaryOp::Le),
            ("a >= b", BinaryOp::Ge),
            ("a && b", BinaryOp::And),
            ("a || b", BinaryOp::Or),
        ];
        for (expr_src, expected_op) in cases {
            let source =
                format!("contract C {{ fn f(a: bool, b: bool) {{ let x = {expr_src}; }} }}\n");
            let program = lower_source(&source);
            let contract = only_contract(&program);
            let body = &contract
                .members
                .iter()
                .find_map(|m| match m {
                    ContractMember::Fn(f) => Some(f),
                    _ => None,
                })
                .unwrap()
                .body;
            let Statement::Let(let_stmt) = &body.statements[0] else {
                panic!("expected a let statement")
            };
            let Expr::Binary { op, .. } = &let_stmt.value else {
                panic!("expected a binary expression for {expr_src}")
            };
            assert_eq!(op, expected_op, "wrong operator lowered for `{expr_src}`");
        }
    }

    #[test]
    fn lowering_never_panics_on_a_cst_with_parser_errors() {
        // Deliberately malformed input - lowering degrades gracefully
        // rather than panicking, per this crate's widened totality
        // guarantee (see src/lower.rs's module documentation).
        let (cst, diagnostics) = Cst::parse("contract C { fn f() { let x = ; } }\n", "bad.kyn");
        assert!(!diagnostics.is_empty());
        let _ = lower(&cst);
    }

    #[test]
    fn visitor_counts_every_binary_expression() {
        struct CountBinary(usize);
        impl Visitor for CountBinary {
            fn visit_expr(&mut self, node: &Expr) {
                if matches!(node, Expr::Binary { .. }) {
                    self.0 += 1;
                }
                walk_expr(self, node);
            }
        }
        let program = lower_source("contract C { fn f() { let x = 1 + 2 * 3 - 4; } }\n");
        let mut counter = CountBinary(0);
        counter.visit_program(&program);
        // `1 + 2 * 3 - 4` is two `-`/`+`-level binary nodes plus one `*`.
        assert_eq!(counter.0, 3);
    }
}
