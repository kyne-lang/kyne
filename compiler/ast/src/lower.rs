//! CST→AST lowering, per docs/COMPILER_ARCHITECTURE.md §7.
//!
//! Total over any CST the parser produced without error: every function
//! here degrades to a placeholder rather than panicking when a node
//! shape doesn't match what a successful parse would have produced
//! (which can only happen for a CST containing a parser-recovery
//! `ErrorNode`, or - per §7's own framing - would indicate a parser
//! defect). This is a deliberate widening of §7's literal requirement
//! ("MUST NOT be able to fail on any CST the parser produced
//! successfully") to also degrade gracefully on CSTs that did *not*
//! parse successfully, so tooling built on this crate (e.g. a future
//! Language Server) can still get a best-effort AST for a file the user
//! is actively editing, mid-error, rather than nothing at all.

use kyne_cst::{Cst, NodeKind, SyntaxNode, Token, TokenKind};

use crate::nodes::*;

/// Lowers a parsed [`Cst`] into a [`Program`], per
/// docs/COMPILER_ARCHITECTURE.md §7.
pub fn lower(cst: &Cst) -> Program {
    Lowerer {
        source: cst.source(),
    }
    .lower_program(cst.root())
}

struct Lowerer<'a> {
    source: &'a str,
}

fn is_expr_kind(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::BinaryExpr
            | NodeKind::UnaryExpr
            | NodeKind::CallExpr
            | NodeKind::FieldExpr
            | NodeKind::MethodCallExpr
            | NodeKind::TryExpr
            | NodeKind::PathExpr
            | NodeKind::IdentExpr
            | NodeKind::LiteralExpr
            | NodeKind::ParenExpr
            | NodeKind::StructLiteral
            | NodeKind::ListLiteral
            | NodeKind::MapLiteral
            | NodeKind::IfStmt
            | NodeKind::MatchStmt
    )
}

fn expr_children(node: &SyntaxNode) -> impl Iterator<Item = &SyntaxNode> {
    node.nodes().filter(|n| is_expr_kind(n.kind))
}

fn identifiers<'a>(node: &'a SyntaxNode, source: &'a str) -> impl Iterator<Item = String> + 'a {
    node.tokens()
        .filter(|t| t.kind == TokenKind::Identifier)
        .map(|t| t.text(source).to_string())
}

fn first_identifier(node: &SyntaxNode, source: &str) -> String {
    identifiers(node, source).next().unwrap_or_default()
}

impl<'a> Lowerer<'a> {
    fn lower_program(&self, node: &SyntaxNode) -> Program {
        let mut imports = Vec::new();
        let mut items = Vec::new();
        for child in node.nodes() {
            match child.kind {
                NodeKind::Import => imports.push(self.lower_import(child)),
                _ => {
                    if let Some(decl) = self.lower_top_level_decl(child) {
                        items.push(decl);
                    }
                }
            }
        }
        Program { imports, items }
    }

    fn lower_import(&self, node: &SyntaxNode) -> Import {
        let path = node
            .child(NodeKind::ImportPath)
            .map(|p| identifiers(p, self.source).collect())
            .unwrap_or_default();
        let names = node
            .child(NodeKind::IdentifierList)
            .map(|l| identifiers(l, self.source).collect())
            .unwrap_or_default();
        Import { path, names }
    }

    fn lower_top_level_decl(&self, node: &SyntaxNode) -> Option<TopLevelDecl> {
        Some(match node.kind {
            NodeKind::ContractDecl => TopLevelDecl::Contract(self.lower_contract_decl(node)),
            NodeKind::StructDecl => TopLevelDecl::Struct(self.lower_struct_decl(node)),
            NodeKind::EnumDecl => TopLevelDecl::Enum(self.lower_enum_decl(node)),
            NodeKind::ErrorDecl => TopLevelDecl::Error(self.lower_error_decl(node)),
            NodeKind::EventDecl => TopLevelDecl::Event(self.lower_event_decl(node)),
            NodeKind::ConstDecl => TopLevelDecl::Const(self.lower_const_decl(node)),
            NodeKind::FnDecl => TopLevelDecl::Fn(self.lower_fn_decl(node)),
            _ => return None,
        })
    }

    fn lower_contract_decl(&self, node: &SyntaxNode) -> ContractDecl {
        let name = first_identifier(node, self.source);
        let members = node
            .nodes()
            .filter_map(|c| self.lower_contract_member(c))
            .collect();
        ContractDecl { name, members }
    }

    fn lower_contract_member(&self, node: &SyntaxNode) -> Option<ContractMember> {
        Some(match node.kind {
            NodeKind::StateDecl => ContractMember::State(self.lower_state_decl(node)),
            NodeKind::ConstDecl => ContractMember::Const(self.lower_const_decl(node)),
            NodeKind::ErrorDecl => ContractMember::Error(self.lower_error_decl(node)),
            NodeKind::EventDecl => ContractMember::Event(self.lower_event_decl(node)),
            NodeKind::StructDecl => ContractMember::Struct(self.lower_struct_decl(node)),
            NodeKind::EnumDecl => ContractMember::Enum(self.lower_enum_decl(node)),
            NodeKind::FnDecl => ContractMember::Fn(self.lower_fn_decl(node)),
            _ => return None,
        })
    }

    fn lower_state_decl(&self, node: &SyntaxNode) -> StateDecl {
        let name = first_identifier(node, self.source);
        let ty = node
            .child(NodeKind::Type)
            .map(|t| self.lower_type(t))
            .unwrap_or(Type::Named(String::new()));
        let init = expr_children(node).next().map(|e| self.lower_expr(e));
        StateDecl { name, ty, init }
    }

    fn lower_const_decl(&self, node: &SyntaxNode) -> ConstDecl {
        let name = first_identifier(node, self.source);
        let ty = node
            .child(NodeKind::Type)
            .map(|t| self.lower_type(t))
            .unwrap_or(Type::Named(String::new()));
        let value = expr_children(node)
            .next()
            .map(|e| self.lower_expr(e))
            .unwrap_or(Expr::Path(Path::Ident(String::new())));
        ConstDecl { name, ty, value }
    }

    fn lower_error_decl(&self, node: &SyntaxNode) -> ErrorDecl {
        let name = first_identifier(node, self.source);
        let variants = node
            .child(NodeKind::ErrorVariants)
            .map(|v| identifiers(v, self.source).collect())
            .unwrap_or_default();
        ErrorDecl { name, variants }
    }

    fn lower_event_decl(&self, node: &SyntaxNode) -> EventDecl {
        let name = first_identifier(node, self.source);
        let params = node
            .child(NodeKind::ParamList)
            .map(|p| {
                p.children_of_kind(NodeKind::Param)
                    .map(|c| self.lower_param(c))
                    .collect()
            })
            .unwrap_or_default();
        EventDecl { name, params }
    }

    fn lower_struct_decl(&self, node: &SyntaxNode) -> StructDecl {
        let name = first_identifier(node, self.source);
        let fields = node
            .child(NodeKind::FieldList)
            .map(|f| {
                f.children_of_kind(NodeKind::Field)
                    .map(|c| self.lower_field(c))
                    .collect()
            })
            .unwrap_or_default();
        StructDecl { name, fields }
    }

    fn lower_field(&self, node: &SyntaxNode) -> Field {
        let name = first_identifier(node, self.source);
        let ty = node
            .child(NodeKind::Type)
            .map(|t| self.lower_type(t))
            .unwrap_or(Type::Named(String::new()));
        Field { name, ty }
    }

    fn lower_param(&self, node: &SyntaxNode) -> Param {
        let name = first_identifier(node, self.source);
        let ty = node
            .child(NodeKind::Type)
            .map(|t| self.lower_type(t))
            .unwrap_or(Type::Named(String::new()));
        Param { name, ty }
    }

    fn lower_enum_decl(&self, node: &SyntaxNode) -> EnumDecl {
        let name = first_identifier(node, self.source);
        let variants = node
            .children_of_kind(NodeKind::EnumVariant)
            .map(|c| self.lower_enum_variant(c))
            .collect();
        EnumDecl { name, variants }
    }

    fn lower_enum_variant(&self, node: &SyntaxNode) -> EnumVariant {
        let name = first_identifier(node, self.source);
        let shape = if let Some(types) = node.child(NodeKind::TypeList) {
            EnumVariantShape::Tuple(
                types
                    .children_of_kind(NodeKind::Type)
                    .map(|t| self.lower_type(t))
                    .collect(),
            )
        } else if let Some(fields) = node.child(NodeKind::FieldList) {
            EnumVariantShape::Struct(
                fields
                    .children_of_kind(NodeKind::Field)
                    .map(|f| self.lower_field(f))
                    .collect(),
            )
        } else {
            EnumVariantShape::Unit
        };
        EnumVariant { name, shape }
    }

    fn lower_fn_decl(&self, node: &SyntaxNode) -> FnDecl {
        let visibility = if node.tokens().any(|t| t.kind == TokenKind::Public) {
            Visibility::Public
        } else if node.tokens().any(|t| t.kind == TokenKind::Internal) {
            Visibility::Internal
        } else {
            Visibility::Private
        };
        let name = first_identifier(node, self.source);
        let params = node
            .child(NodeKind::ParamList)
            .map(|p| {
                p.children_of_kind(NodeKind::Param)
                    .map(|c| self.lower_param(c))
                    .collect()
            })
            .unwrap_or_default();
        let return_type = node.child(NodeKind::Type).map(|t| self.lower_type(t));
        let body = node
            .child(NodeKind::Block)
            .map(|b| self.lower_block(b))
            .unwrap_or(Block {
                statements: Vec::new(),
            });
        FnDecl {
            visibility,
            name,
            params,
            return_type,
            body,
        }
    }

    fn lower_type(&self, node: &SyntaxNode) -> Type {
        let name = node
            .tokens()
            .find(|t| t.kind == TokenKind::Identifier)
            .map(|t| t.text(self.source).to_string())
            .unwrap_or_default();
        let mut type_args = node.children_of_kind(NodeKind::Type);
        match name.as_str() {
            "list" => Type::List(Box::new(
                type_args
                    .next()
                    .map(|t| self.lower_type(t))
                    .unwrap_or(Type::Named(String::new())),
            )),
            "Option" => Type::Option(Box::new(
                type_args
                    .next()
                    .map(|t| self.lower_type(t))
                    .unwrap_or(Type::Named(String::new())),
            )),
            "map" => {
                let k = type_args
                    .next()
                    .map(|t| self.lower_type(t))
                    .unwrap_or(Type::Named(String::new()));
                let v = type_args
                    .next()
                    .map(|t| self.lower_type(t))
                    .unwrap_or(Type::Named(String::new()));
                Type::Map(Box::new(k), Box::new(v))
            }
            "Result" => {
                let k = type_args
                    .next()
                    .map(|t| self.lower_type(t))
                    .unwrap_or(Type::Named(String::new()));
                let v = type_args
                    .next()
                    .map(|t| self.lower_type(t))
                    .unwrap_or(Type::Named(String::new()));
                Type::Result(Box::new(k), Box::new(v))
            }
            "bytes" if node.tokens().any(|t| t.kind == TokenKind::IntLiteral) => {
                let n = node
                    .tokens()
                    .find(|t| t.kind == TokenKind::IntLiteral)
                    .map(|t| t.text(self.source).to_string())
                    .unwrap_or_default();
                Type::Bytes(n)
            }
            _ => Type::Named(name),
        }
    }

    fn lower_block(&self, node: &SyntaxNode) -> Block {
        Block {
            statements: node.nodes().filter_map(|c| self.lower_stmt(c)).collect(),
        }
    }

    fn lower_stmt(&self, node: &SyntaxNode) -> Option<Statement> {
        Some(match node.kind {
            NodeKind::LetStmt => Statement::Let(self.lower_let_stmt(node)),
            NodeKind::AssignStmt => Statement::Assign(self.lower_assign_stmt(node)),
            NodeKind::AuthStmt => Statement::Auth(
                expr_children(node)
                    .next()
                    .map(|e| self.lower_expr(e))
                    .unwrap_or(Expr::Path(Path::Ident(String::new()))),
            ),
            NodeKind::EmitStmt => Statement::Emit(self.lower_emit_stmt(node)),
            NodeKind::ThrowStmt => Statement::Throw(
                expr_children(node)
                    .next()
                    .map(|e| self.lower_expr(e))
                    .unwrap_or(Expr::Path(Path::Ident(String::new()))),
            ),
            NodeKind::IfStmt => Statement::If(self.lower_if_stmt(node)),
            NodeKind::MatchStmt => Statement::Match(self.lower_match_stmt(node)),
            NodeKind::ForStmt => Statement::For(self.lower_for_stmt(node)),
            NodeKind::WhileStmt => Statement::While(self.lower_while_stmt(node)),
            NodeKind::ReturnStmt => {
                Statement::Return(expr_children(node).next().map(|e| self.lower_expr(e)))
            }
            NodeKind::BreakStmt => Statement::Break,
            NodeKind::ContinueStmt => Statement::Continue,
            NodeKind::ExprStmt => Statement::Expr(
                expr_children(node)
                    .next()
                    .map(|e| self.lower_expr(e))
                    .unwrap_or(Expr::Path(Path::Ident(String::new()))),
            ),
            _ => return None,
        })
    }

    fn lower_let_stmt(&self, node: &SyntaxNode) -> LetStmt {
        let name = first_identifier(node, self.source);
        let ty = node.child(NodeKind::Type).map(|t| self.lower_type(t));
        let value = expr_children(node)
            .next()
            .map(|e| self.lower_expr(e))
            .unwrap_or(Expr::Path(Path::Ident(String::new())));
        LetStmt { name, ty, value }
    }

    fn lower_assign_stmt(&self, node: &SyntaxNode) -> AssignStmt {
        let mut exprs = expr_children(node);
        let target = exprs
            .next()
            .map(|e| self.lower_expr(e))
            .unwrap_or(Expr::Path(Path::Ident(String::new())));
        let value = exprs
            .next()
            .map(|e| self.lower_expr(e))
            .unwrap_or(Expr::Path(Path::Ident(String::new())));
        let op = match node.tokens().find(|t| {
            matches!(
                t.kind,
                TokenKind::Eq
                    | TokenKind::PlusEq
                    | TokenKind::MinusEq
                    | TokenKind::StarEq
                    | TokenKind::SlashEq
                    | TokenKind::PercentEq
            )
        }) {
            Some(t) => match t.kind {
                TokenKind::PlusEq => AssignOp::Add,
                TokenKind::MinusEq => AssignOp::Sub,
                TokenKind::StarEq => AssignOp::Mul,
                TokenKind::SlashEq => AssignOp::Div,
                TokenKind::PercentEq => AssignOp::Rem,
                _ => AssignOp::Assign,
            },
            None => AssignOp::Assign,
        };
        AssignStmt { target, op, value }
    }

    fn lower_emit_stmt(&self, node: &SyntaxNode) -> EmitStmt {
        let event = first_identifier(node, self.source);
        let args = node
            .child(NodeKind::ArgList)
            .map(|l| expr_children(l).map(|e| self.lower_expr(e)).collect())
            .unwrap_or_default();
        EmitStmt { event, args }
    }

    fn lower_if_stmt(&self, node: &SyntaxNode) -> IfStmt {
        let condition = expr_children(node)
            .next()
            .map(|e| self.lower_expr(e))
            .unwrap_or(Expr::Path(Path::Ident(String::new())));
        let blocks: Vec<&SyntaxNode> = node.children_of_kind(NodeKind::Block).collect();
        let then_block = blocks
            .first()
            .map(|b| self.lower_block(b))
            .unwrap_or(Block {
                statements: Vec::new(),
            });
        let else_branch = if let Some(nested) = node.child(NodeKind::IfStmt) {
            Some(ElseBranch::If(Box::new(self.lower_if_stmt(nested))))
        } else {
            blocks
                .get(1)
                .map(|b| ElseBranch::Block(self.lower_block(b)))
        };
        IfStmt {
            condition,
            then_block,
            else_branch,
        }
    }

    fn lower_match_stmt(&self, node: &SyntaxNode) -> MatchStmt {
        let subject = expr_children(node)
            .next()
            .map(|e| self.lower_expr(e))
            .unwrap_or(Expr::Path(Path::Ident(String::new())));
        let arms = node
            .children_of_kind(NodeKind::MatchArm)
            .map(|a| self.lower_match_arm(a))
            .collect();
        MatchStmt { subject, arms }
    }

    fn lower_match_arm(&self, node: &SyntaxNode) -> MatchArm {
        let pattern = node
            .child(NodeKind::Pattern)
            .map(|p| self.lower_pattern(p))
            .unwrap_or(Pattern::Wildcard);
        let has_guard = node.tokens().any(|t| t.kind == TokenKind::If);
        let mut exprs = expr_children(node);
        let guard = if has_guard {
            exprs.next().map(|e| self.lower_expr(e))
        } else {
            None
        };

        let body = if let Some(block) = node.child(NodeKind::Block) {
            MatchArmBody::Block(self.lower_block(block))
        } else if let Some(ret) = node.child(NodeKind::ReturnStmt) {
            MatchArmBody::Return(expr_children(ret).next().map(|e| self.lower_expr(e)))
        } else if let Some(thr) = node.child(NodeKind::ThrowStmt) {
            MatchArmBody::Throw(
                expr_children(thr)
                    .next()
                    .map(|e| self.lower_expr(e))
                    .unwrap_or(Expr::Path(Path::Ident(String::new()))),
            )
        } else if node.child(NodeKind::BreakStmt).is_some() {
            MatchArmBody::Break
        } else if node.child(NodeKind::ContinueStmt).is_some() {
            MatchArmBody::Continue
        } else {
            MatchArmBody::Expr(
                exprs
                    .next()
                    .map(|e| self.lower_expr(e))
                    .unwrap_or(Expr::Path(Path::Ident(String::new()))),
            )
        };
        MatchArm {
            pattern,
            guard,
            body,
        }
    }

    fn lower_pattern(&self, node: &SyntaxNode) -> Pattern {
        if let Some(lit) = node.tokens().find(|t| {
            matches!(
                t.kind,
                TokenKind::IntLiteral
                    | TokenKind::StringLiteral
                    | TokenKind::True
                    | TokenKind::False
            )
        }) {
            let has_minus = node.tokens().any(|t| t.kind == TokenKind::Minus);
            return Pattern::Literal(self.lower_literal_token(lit, has_minus));
        }
        let names: Vec<String> = identifiers(node, self.source).collect();
        if names.first().map(String::as_str) == Some("_") {
            return Pattern::Wildcard;
        }
        let path = match names.as_slice() {
            [single] => Path::Ident(single.clone()),
            [ty, variant, ..] => Path::Qualified(ty.clone(), variant.clone()),
            [] => Path::Ident(String::new()),
        };
        if let Some(list) = node.child(NodeKind::PatternList) {
            Pattern::Tuple {
                path,
                patterns: list
                    .children_of_kind(NodeKind::Pattern)
                    .map(|p| self.lower_pattern(p))
                    .collect(),
            }
        } else if let Some(fields) = node.child(NodeKind::FieldPatternList) {
            Pattern::Struct {
                path,
                fields: identifiers(fields, self.source).collect(),
            }
        } else if names.len() == 1 {
            Pattern::Ident(names[0].clone())
        } else {
            Pattern::Tuple {
                path,
                patterns: Vec::new(),
            }
        }
    }

    fn lower_for_stmt(&self, node: &SyntaxNode) -> ForStmt {
        let var = first_identifier(node, self.source);
        let iterable = expr_children(node)
            .next()
            .map(|e| self.lower_expr(e))
            .unwrap_or(Expr::Path(Path::Ident(String::new())));
        let body = node
            .child(NodeKind::Block)
            .map(|b| self.lower_block(b))
            .unwrap_or(Block {
                statements: Vec::new(),
            });
        ForStmt {
            var,
            iterable,
            body,
        }
    }

    fn lower_while_stmt(&self, node: &SyntaxNode) -> WhileStmt {
        let condition = expr_children(node)
            .next()
            .map(|e| self.lower_expr(e))
            .unwrap_or(Expr::Path(Path::Ident(String::new())));
        let body = node
            .child(NodeKind::Block)
            .map(|b| self.lower_block(b))
            .unwrap_or(Block {
                statements: Vec::new(),
            });
        WhileStmt { condition, body }
    }

    fn lower_expr(&self, node: &SyntaxNode) -> Expr {
        match node.kind {
            NodeKind::ParenExpr => expr_children(node)
                .next()
                .map(|e| self.lower_expr(e))
                .unwrap_or(Expr::Path(Path::Ident(String::new()))),
            NodeKind::LiteralExpr => {
                let has_minus = false;
                let token = node.tokens().find(|t| {
                    matches!(
                        t.kind,
                        TokenKind::IntLiteral
                            | TokenKind::StringLiteral
                            | TokenKind::True
                            | TokenKind::False
                    )
                });
                match token {
                    Some(t) => Expr::Literal(self.lower_literal_token(t, has_minus)),
                    None => Expr::Literal(Literal::Bool(false)),
                }
            }
            NodeKind::IdentExpr => Expr::Path(Path::Ident(first_identifier(node, self.source))),
            NodeKind::PathExpr => {
                let names: Vec<String> = identifiers(node, self.source).collect();
                match names.as_slice() {
                    [ty, variant, ..] => Expr::Path(Path::Qualified(ty.clone(), variant.clone())),
                    [single] => Expr::Path(Path::Ident(single.clone())),
                    [] => Expr::Path(Path::Ident(String::new())),
                }
            }
            NodeKind::BinaryExpr => {
                let mut exprs = expr_children(node);
                let left = exprs
                    .next()
                    .map(|e| self.lower_expr(e))
                    .unwrap_or(Expr::Path(Path::Ident(String::new())));
                let right = exprs
                    .next()
                    .map(|e| self.lower_expr(e))
                    .unwrap_or(Expr::Path(Path::Ident(String::new())));
                let op = self.lower_binary_op(node);
                Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                }
            }
            NodeKind::UnaryExpr => {
                let operand = expr_children(node)
                    .next()
                    .map(|e| self.lower_expr(e))
                    .unwrap_or(Expr::Path(Path::Ident(String::new())));
                let op = if node.tokens().any(|t| t.kind == TokenKind::Bang) {
                    UnaryOp::Not
                } else {
                    UnaryOp::Neg
                };
                Expr::Unary {
                    op,
                    operand: Box::new(operand),
                }
            }
            NodeKind::CallExpr => {
                let callee = expr_children(node)
                    .next()
                    .map(|e| self.lower_expr(e))
                    .unwrap_or(Expr::Path(Path::Ident(String::new())));
                let args = node
                    .child(NodeKind::ArgList)
                    .map(|l| expr_children(l).map(|e| self.lower_expr(e)).collect())
                    .unwrap_or_default();
                Expr::Call {
                    callee: Box::new(callee),
                    args,
                }
            }
            NodeKind::FieldExpr => {
                let base = expr_children(node)
                    .next()
                    .map(|e| self.lower_expr(e))
                    .unwrap_or(Expr::Path(Path::Ident(String::new())));
                let name = identifiers(node, self.source).last().unwrap_or_default();
                Expr::Field {
                    base: Box::new(base),
                    name,
                }
            }
            NodeKind::MethodCallExpr => {
                let base = expr_children(node)
                    .next()
                    .map(|e| self.lower_expr(e))
                    .unwrap_or(Expr::Path(Path::Ident(String::new())));
                let method = identifiers(node, self.source).last().unwrap_or_default();
                let args = node
                    .child(NodeKind::ArgList)
                    .map(|l| expr_children(l).map(|e| self.lower_expr(e)).collect())
                    .unwrap_or_default();
                Expr::MethodCall {
                    base: Box::new(base),
                    method,
                    args,
                }
            }
            NodeKind::TryExpr => {
                let operand = expr_children(node)
                    .next()
                    .map(|e| self.lower_expr(e))
                    .unwrap_or(Expr::Path(Path::Ident(String::new())));
                Expr::Try {
                    operand: Box::new(operand),
                }
            }
            NodeKind::StructLiteral => {
                let path_node = expr_children(node).next();
                let path = match path_node {
                    Some(p) if p.kind == NodeKind::PathExpr => {
                        let names: Vec<String> = identifiers(p, self.source).collect();
                        match names.as_slice() {
                            [ty, variant, ..] => Path::Qualified(ty.clone(), variant.clone()),
                            [single] => Path::Ident(single.clone()),
                            [] => Path::Ident(String::new()),
                        }
                    }
                    Some(p) => Path::Ident(first_identifier(p, self.source)),
                    None => Path::Ident(String::new()),
                };
                let fields = node
                    .children_of_kind(NodeKind::FieldInit)
                    .map(|f| self.lower_field_init(f))
                    .collect();
                Expr::StructLiteral { path, fields }
            }
            NodeKind::ListLiteral => {
                Expr::ListLiteral(expr_children(node).map(|e| self.lower_expr(e)).collect())
            }
            NodeKind::MapLiteral => Expr::MapLiteral(
                node.children_of_kind(NodeKind::MapEntry)
                    .map(|e| self.lower_map_entry(e))
                    .collect(),
            ),
            NodeKind::IfStmt => Expr::If(Box::new(self.lower_if_stmt(node))),
            NodeKind::MatchStmt => Expr::Match(Box::new(self.lower_match_stmt(node))),
            _ => Expr::Path(Path::Ident(String::new())),
        }
    }

    fn lower_binary_op(&self, node: &SyntaxNode) -> BinaryOp {
        for token in node.tokens() {
            let op = match token.kind {
                TokenKind::Plus => Some(BinaryOp::Add),
                TokenKind::Minus => Some(BinaryOp::Sub),
                TokenKind::Star => Some(BinaryOp::Mul),
                TokenKind::Slash => Some(BinaryOp::Div),
                TokenKind::Percent => Some(BinaryOp::Rem),
                TokenKind::EqEq => Some(BinaryOp::Eq),
                TokenKind::NotEq => Some(BinaryOp::Ne),
                TokenKind::LAngle => Some(BinaryOp::Lt),
                TokenKind::RAngle => Some(BinaryOp::Gt),
                TokenKind::LtEq => Some(BinaryOp::Le),
                TokenKind::GtEq => Some(BinaryOp::Ge),
                TokenKind::AmpAmp => Some(BinaryOp::And),
                TokenKind::PipePipe => Some(BinaryOp::Or),
                _ => None,
            };
            if let Some(op) = op {
                return op;
            }
        }
        BinaryOp::Add
    }

    fn lower_field_init(&self, node: &SyntaxNode) -> FieldInit {
        let name = first_identifier(node, self.source);
        let value = expr_children(node)
            .next()
            .map(|e| self.lower_expr(e))
            .unwrap_or_else(|| Expr::Path(Path::Ident(name.clone())));
        FieldInit { name, value }
    }

    fn lower_map_entry(&self, node: &SyntaxNode) -> MapEntry {
        let mut exprs = expr_children(node);
        let key = exprs
            .next()
            .map(|e| self.lower_expr(e))
            .unwrap_or(Expr::Path(Path::Ident(String::new())));
        let value = exprs
            .next()
            .map(|e| self.lower_expr(e))
            .unwrap_or(Expr::Path(Path::Ident(String::new())));
        MapEntry { key, value }
    }

    fn lower_literal_token(&self, token: &Token, negate: bool) -> Literal {
        match token.kind {
            TokenKind::IntLiteral => {
                let text = token.text(self.source);
                Literal::Int(if negate {
                    format!("-{text}")
                } else {
                    text.to_string()
                })
            }
            TokenKind::StringLiteral => {
                Literal::Str(decode_string_literal(token.text(self.source)))
            }
            TokenKind::True => Literal::Bool(true),
            TokenKind::False => Literal::Bool(false),
            _ => Literal::Bool(false),
        }
    }
}

/// Decodes a string literal's escape sequences into its runtime value,
/// per LANGUAGE_SPEC.md §1.6. Safe to assume every escape is well-formed:
/// `kyne_lexer` only produces a `StringLiteral` token (as opposed to an
/// `Error` token) for a string containing exclusively valid escapes.
fn decode_string_literal(text: &str) -> String {
    let inner = text
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(text);
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('u') => {
                if chars.next() == Some('{') {
                    let hex: String = chars.by_ref().take_while(|c| *c != '}').collect();
                    if let Some(ch) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                        out.push(ch);
                    }
                }
            }
            Some(other) => out.push(other),
            None => {}
        }
    }
    out
}
