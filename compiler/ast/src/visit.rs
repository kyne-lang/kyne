//! The shared visitor abstraction every later pass (Name Resolution, the
//! Type Checker, the Semantic Analyzer, the Security Analyzer) traverses
//! the AST through, per docs/COMPILER_ARCHITECTURE.md §7 - a new node
//! type's traversal behavior is defined once, here, not once per pass.
//!
//! Each `visit_*` method has a default implementation that walks the
//! node's children via the corresponding free `walk_*` function.
//! Implementors override only the methods relevant to their pass; the
//! rest fall through to the default walk.

use crate::nodes::*;

pub trait Visitor {
    fn visit_program(&mut self, node: &Program) {
        walk_program(self, node);
    }
    fn visit_top_level_decl(&mut self, node: &TopLevelDecl) {
        walk_top_level_decl(self, node);
    }
    fn visit_contract_decl(&mut self, node: &ContractDecl) {
        walk_contract_decl(self, node);
    }
    fn visit_contract_member(&mut self, node: &ContractMember) {
        walk_contract_member(self, node);
    }
    fn visit_state_decl(&mut self, node: &StateDecl) {
        walk_state_decl(self, node);
    }
    fn visit_const_decl(&mut self, node: &ConstDecl) {
        walk_const_decl(self, node);
    }
    fn visit_error_decl(&mut self, _node: &ErrorDecl) {}
    fn visit_event_decl(&mut self, _node: &EventDecl) {}
    fn visit_struct_decl(&mut self, _node: &StructDecl) {}
    fn visit_enum_decl(&mut self, _node: &EnumDecl) {}
    fn visit_fn_decl(&mut self, node: &FnDecl) {
        walk_fn_decl(self, node);
    }
    fn visit_block(&mut self, node: &Block) {
        walk_block(self, node);
    }
    fn visit_stmt(&mut self, node: &Statement) {
        walk_stmt(self, node);
    }
    fn visit_expr(&mut self, node: &Expr) {
        walk_expr(self, node);
    }
    fn visit_pattern(&mut self, _node: &Pattern) {}
}

pub fn walk_program<V: Visitor + ?Sized>(v: &mut V, node: &Program) {
    for item in &node.items {
        v.visit_top_level_decl(item);
    }
}

pub fn walk_top_level_decl<V: Visitor + ?Sized>(v: &mut V, node: &TopLevelDecl) {
    match node {
        TopLevelDecl::Contract(decl) => v.visit_contract_decl(decl),
        TopLevelDecl::Struct(decl) => v.visit_struct_decl(decl),
        TopLevelDecl::Enum(decl) => v.visit_enum_decl(decl),
        TopLevelDecl::Error(decl) => v.visit_error_decl(decl),
        TopLevelDecl::Event(decl) => v.visit_event_decl(decl),
        TopLevelDecl::Const(decl) => v.visit_const_decl(decl),
        TopLevelDecl::Fn(decl) => v.visit_fn_decl(decl),
    }
}

pub fn walk_contract_decl<V: Visitor + ?Sized>(v: &mut V, node: &ContractDecl) {
    for member in &node.members {
        v.visit_contract_member(member);
    }
}

pub fn walk_contract_member<V: Visitor + ?Sized>(v: &mut V, node: &ContractMember) {
    match node {
        ContractMember::State(decl) => v.visit_state_decl(decl),
        ContractMember::Const(decl) => v.visit_const_decl(decl),
        ContractMember::Error(decl) => v.visit_error_decl(decl),
        ContractMember::Event(decl) => v.visit_event_decl(decl),
        ContractMember::Struct(decl) => v.visit_struct_decl(decl),
        ContractMember::Enum(decl) => v.visit_enum_decl(decl),
        ContractMember::Fn(decl) => v.visit_fn_decl(decl),
    }
}

pub fn walk_state_decl<V: Visitor + ?Sized>(v: &mut V, node: &StateDecl) {
    if let Some(init) = &node.init {
        v.visit_expr(init);
    }
}

pub fn walk_const_decl<V: Visitor + ?Sized>(v: &mut V, node: &ConstDecl) {
    v.visit_expr(&node.value);
}

pub fn walk_fn_decl<V: Visitor + ?Sized>(v: &mut V, node: &FnDecl) {
    v.visit_block(&node.body);
}

pub fn walk_block<V: Visitor + ?Sized>(v: &mut V, node: &Block) {
    for stmt in &node.statements {
        v.visit_stmt(stmt);
    }
}

pub fn walk_stmt<V: Visitor + ?Sized>(v: &mut V, node: &Statement) {
    match node {
        Statement::Let(s) => v.visit_expr(&s.value),
        Statement::Assign(s) => {
            v.visit_expr(&s.target);
            v.visit_expr(&s.value);
        }
        Statement::Auth(e) | Statement::Throw(e) | Statement::Expr(e) => v.visit_expr(e),
        Statement::Emit(s) => {
            for arg in &s.args {
                v.visit_expr(arg);
            }
        }
        Statement::If(s) => walk_if_stmt(v, s),
        Statement::Match(s) => walk_match_stmt(v, s),
        Statement::For(s) => {
            v.visit_expr(&s.iterable);
            v.visit_block(&s.body);
        }
        Statement::While(s) => {
            v.visit_expr(&s.condition);
            v.visit_block(&s.body);
        }
        Statement::Return(Some(e)) => v.visit_expr(e),
        Statement::Return(None) | Statement::Break | Statement::Continue => {}
    }
}

pub fn walk_if_stmt<V: Visitor + ?Sized>(v: &mut V, node: &IfStmt) {
    v.visit_expr(&node.condition);
    v.visit_block(&node.then_block);
    match &node.else_branch {
        Some(ElseBranch::If(nested)) => walk_if_stmt(v, nested),
        Some(ElseBranch::Block(block)) => v.visit_block(block),
        None => {}
    }
}

pub fn walk_match_stmt<V: Visitor + ?Sized>(v: &mut V, node: &MatchStmt) {
    v.visit_expr(&node.subject);
    for arm in &node.arms {
        v.visit_pattern(&arm.pattern);
        if let Some(guard) = &arm.guard {
            v.visit_expr(guard);
        }
        match &arm.body {
            MatchArmBody::Block(block) => v.visit_block(block),
            MatchArmBody::Expr(e) | MatchArmBody::Throw(e) => v.visit_expr(e),
            MatchArmBody::Return(Some(e)) => v.visit_expr(e),
            MatchArmBody::Return(None) | MatchArmBody::Break | MatchArmBody::Continue => {}
        }
    }
}

pub fn walk_expr<V: Visitor + ?Sized>(v: &mut V, node: &Expr) {
    match node {
        Expr::Literal(_) | Expr::Path(_) => {}
        Expr::Binary { left, right, .. } => {
            v.visit_expr(left);
            v.visit_expr(right);
        }
        Expr::Unary { operand, .. } | Expr::Try { operand } => v.visit_expr(operand),
        Expr::Call { callee, args } => {
            v.visit_expr(callee);
            for arg in args {
                v.visit_expr(arg);
            }
        }
        Expr::Field { base, .. } => v.visit_expr(base),
        Expr::MethodCall { base, args, .. } => {
            v.visit_expr(base);
            for arg in args {
                v.visit_expr(arg);
            }
        }
        Expr::StructLiteral { fields, .. } => {
            for field in fields {
                v.visit_expr(&field.value);
            }
        }
        Expr::ListLiteral(items) => {
            for item in items {
                v.visit_expr(item);
            }
        }
        Expr::MapLiteral(entries) => {
            for entry in entries {
                v.visit_expr(&entry.key);
                v.visit_expr(&entry.value);
            }
        }
        Expr::If(if_stmt) => walk_if_stmt(v, if_stmt),
        Expr::Match(match_stmt) => walk_match_stmt(v, match_stmt),
    }
}
