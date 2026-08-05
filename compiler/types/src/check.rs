//! The type checker itself, per docs/COMPILER_ARCHITECTURE.md §9.
//!
//! Because Kyne performs no implicit numeric widening and preserves
//! whatever parentheses the author wrote (per
//! docs/adr/ADR-0006-formatter-implementation.md's reasoning for the
//! formatter - the same absence of precedence-driven restructuring
//! applies here), this checker never needs a general unification
//! algorithm: every construct's type is computed bottom-up, with an
//! `expected` type threaded down only to resolve LANGUAGE_SPEC.md's two
//! genuinely contextual cases - an untyped integer literal (no default
//! "int" type exists; an integer literal with no surrounding context
//! defaults to `i64`) and a `string_literal` used where a `symbol` is
//! expected, per §1.6.

use std::collections::HashMap;

use kyne_ast::{
    Block, ContractMember, Expr, FnDecl, IfStmt, Literal, MatchArmBody, MatchStmt, Path, Pattern,
    Program, Statement, TopLevelDecl,
};
use kyne_diagnostics::{Diagnostic, Span};

use crate::env::{EnumVariantTys, TypeEnv};
use crate::ty::{lower_type, Ty};

/// See this crate's README for why every diagnostic below uses a
/// placeholder span: `kyne_ast` carries no source-span information yet,
/// the same limitation `kyne_resolver` documented in
/// docs/adr/ADR-0007-resolver-implementation.md.
fn placeholder_span() -> Span {
    Span::new(1, 1, 1)
}

pub struct CheckResult {
    pub diagnostics: Vec<Diagnostic>,
}

pub fn check(program: &Program, file: &str) -> CheckResult {
    let mut diagnostics = Vec::new();
    let env = TypeEnv::build(program, &mut |msg| {
        diagnostics.push(malformed_type_diagnostic(file, &msg))
    });
    let mut checker = Checker {
        env: &env,
        file: file.to_string(),
        diagnostics,
        scopes: Vec::new(),
        fn_return_ty: None,
    };
    checker.check_program(program);
    CheckResult {
        diagnostics: checker.diagnostics,
    }
}

struct Checker<'a> {
    env: &'a TypeEnv,
    file: String,
    diagnostics: Vec<Diagnostic>,
    scopes: Vec<HashMap<String, Ty>>,
    fn_return_ty: Option<Ty>,
}

impl Checker<'_> {
    fn check_program(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                TopLevelDecl::Contract(c) => {
                    for member in &c.members {
                        self.check_contract_member(member);
                    }
                }
                TopLevelDecl::Fn(f) => self.check_fn(f),
                TopLevelDecl::Const(c) => {
                    let expected = lower_type(&c.ty, &mut |_| {});
                    self.push_scope();
                    self.check_expr_expect(&c.value, &expected, "const");
                    self.pop_scope();
                }
                _ => {}
            }
        }
    }

    fn check_contract_member(&mut self, member: &ContractMember) {
        match member {
            ContractMember::Fn(f) => self.check_fn(f),
            ContractMember::State(s) => {
                if let Some(init) = &s.init {
                    let expected = lower_type(&s.ty, &mut |_| {});
                    self.push_scope();
                    self.check_expr_expect(init, &expected, "state");
                    self.pop_scope();
                }
            }
            ContractMember::Const(c) => {
                let expected = lower_type(&c.ty, &mut |_| {});
                self.push_scope();
                self.check_expr_expect(&c.value, &expected, "const");
                self.pop_scope();
            }
            _ => {}
        }
    }

    /// Checks `expr` against `expected`, emitting `KY0201` if the
    /// computed type doesn't match - used at every position
    /// LANGUAGE_SPEC.md requires an explicit type the checker must
    /// actually enforce (a `state`/`const` initializer, a struct-literal
    /// field), as opposed to positions where `expected` is only a
    /// defaulting *hint* for an untyped literal (e.g. a bare function
    /// call argument, where the mismatch is instead reported by the
    /// call site itself with a more specific message).
    fn check_expr_expect(&mut self, expr: &Expr, expected: &Ty, context: &str) -> Ty {
        let actual = self.check_expr(expr, Some(expected));
        if !expected.compatible(&actual) {
            self.error(
                "KY0201",
                format!(
                    "expected `{}`, found `{}`",
                    expected.describe(),
                    actual.describe()
                ),
                format!("{context} initializer type mismatch"),
            );
        }
        actual
    }

    fn check_fn(&mut self, f: &FnDecl) {
        self.push_scope();
        for p in &f.params {
            let ty = lower_type(&p.ty, &mut |_| {});
            self.declare_local(&p.name, ty);
        }
        let return_ty = f
            .return_type
            .as_ref()
            .map(|t| lower_type(t, &mut |_| {}))
            .unwrap_or(Ty::Unit);
        let previous_return_ty = self.fn_return_ty.replace(return_ty);
        self.check_block(&f.body, None);
        self.fn_return_ty = previous_return_ty;
        self.pop_scope();
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare_local(&mut self, name: &str, ty: Ty) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    fn lookup(&self, name: &str) -> Option<Ty> {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.get(name).cloned())
            .or_else(|| self.env.state.get(name).cloned())
            .or_else(|| self.env.consts.get(name).cloned())
    }

    fn error(
        &mut self,
        code: &'static str,
        title: impl Into<String>,
        label: impl Into<String>,
    ) -> Ty {
        self.diagnostics.push(
            Diagnostic::error(code, title, self.file.clone(), placeholder_span(), label)
                .note("kyne_ast does not yet carry source-span information, so this diagnostic cannot point at the exact offending location - see this crate's README")
        );
        Ty::Error
    }

    // ---- Statements ----

    fn check_block(&mut self, block: &Block, expected_value: Option<&Ty>) {
        self.push_scope();
        self.check_block_statements(&block.statements, expected_value);
        self.pop_scope();
    }

    /// Checks a block's statements in the *current* scope (no push/pop) -
    /// used both by `check_block` and directly wherever a scope was
    /// already pushed for another reason (a `for` loop's own variable, a
    /// `match` arm's pattern bindings).
    fn check_block_statements(&mut self, statements: &[Statement], expected_value: Option<&Ty>) {
        for (i, stmt) in statements.iter().enumerate() {
            let is_last = i + 1 == statements.len();
            if is_last && expected_value.is_some() {
                if let Statement::Expr(e) = stmt {
                    self.check_expr(e, expected_value);
                    continue;
                }
            }
            self.check_stmt(stmt);
        }
    }

    /// The value a block produces when used in expression position (an
    /// `if`/`match` arm) - the AST does not distinguish a semicolon-less
    /// trailing expression from an ordinary expression statement (see
    /// this crate's README), so the last `Statement::Expr` in the block
    /// is treated as that value whenever one is asked for. This is only
    /// ever invoked from `Expr::If`/`Expr::Match` handling, never from
    /// statement-position `if`/`match`, so the ambiguity cannot misfire
    /// on an ordinary block.
    fn block_value_ty(&mut self, block: &Block, expected: Option<&Ty>) -> Ty {
        self.push_scope();
        let statements = &block.statements;
        for stmt in &statements[..statements.len().saturating_sub(1)] {
            self.check_stmt(stmt);
        }
        let value_ty = match statements.last() {
            Some(Statement::Expr(e)) => self.check_expr(e, expected),
            Some(other) => {
                self.check_stmt(other);
                Ty::Unit
            }
            None => Ty::Unit,
        };
        self.pop_scope();
        value_ty
    }

    fn check_stmt(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let(s) => {
                let annotated = s.ty.as_ref().map(|t| lower_type(t, &mut |_| {}));
                let value_ty = self.check_expr(&s.value, annotated.as_ref());
                let final_ty = match &annotated {
                    Some(ann) if !ann.compatible(&value_ty) => self.error(
                        "KY0201",
                        format!(
                            "expected `{}`, found `{}`",
                            ann.describe(),
                            value_ty.describe()
                        ),
                        format!(
                            "`{}` does not match the declared type `{}`",
                            s.name,
                            ann.describe()
                        ),
                    ),
                    Some(ann) => ann.clone(),
                    None => value_ty,
                };
                self.declare_local(&s.name, final_ty);
            }
            Statement::Assign(s) => {
                let target_ty = self.check_expr(&s.target, None);
                let value_ty = self.check_expr(&s.value, Some(&target_ty));
                if !target_ty.compatible(&value_ty) {
                    self.error(
                        "KY0201",
                        format!(
                            "expected `{}`, found `{}`",
                            target_ty.describe(),
                            value_ty.describe()
                        ),
                        "assignment type mismatch",
                    );
                } else if !matches!(s.op, kyne_ast::AssignOp::Assign)
                    && !target_ty.is_numeric()
                    && !target_ty.is_error()
                {
                    self.error(
                        "KY0203",
                        format!(
                            "compound assignment is not defined for `{}`",
                            target_ty.describe()
                        ),
                        "only defined for numeric types",
                    );
                }
            }
            Statement::Auth(e) | Statement::Expr(e) => {
                self.check_expr(e, None);
            }
            Statement::Emit(s) => {
                for arg in &s.args {
                    self.check_expr(arg, None);
                }
            }
            Statement::Throw(e) => {
                let error_ty = self.current_error_ty();
                self.check_expr(e, error_ty.as_ref());
            }
            Statement::If(if_stmt) => self.check_if_stmt(if_stmt),
            Statement::Match(match_stmt) => {
                self.check_match(match_stmt, None);
            }
            Statement::For(s) => {
                let iterable_ty = self.check_expr(&s.iterable, None);
                let elem_ty = match iterable_ty {
                    Ty::List(elem) => *elem,
                    Ty::Error => Ty::Error,
                    other => self.error(
                        "KY0201",
                        format!("expected `list<T>`, found `{}`", other.describe()),
                        "`for` can only iterate a `list<T>`",
                    ),
                };
                self.push_scope();
                self.declare_local(&s.var, elem_ty);
                self.check_block_statements(&s.body.statements, None);
                self.pop_scope();
            }
            Statement::While(s) => {
                self.check_expr(&s.condition, Some(&Ty::Bool));
                self.check_block(&s.body, None);
            }
            Statement::Return(Some(e)) => {
                let expected = self.fn_return_ty.clone();
                let actual = self.check_expr(e, expected.as_ref());
                if let Some(expected) = &expected {
                    if !expected.compatible(&actual) {
                        self.error(
                            "KY0201",
                            format!(
                                "expected return type `{}`, found `{}`",
                                expected.describe(),
                                actual.describe()
                            ),
                            "return type mismatch",
                        );
                    }
                }
            }
            Statement::Return(None) | Statement::Break | Statement::Continue => {}
        }
    }

    fn current_error_ty(&self) -> Option<Ty> {
        match &self.fn_return_ty {
            Some(Ty::Result(_, e)) => Some((**e).clone()),
            _ => None,
        }
    }

    fn check_if_stmt(&mut self, if_stmt: &IfStmt) {
        self.check_expr(&if_stmt.condition, Some(&Ty::Bool));
        self.check_block(&if_stmt.then_block, None);
        match &if_stmt.else_branch {
            Some(kyne_ast::ElseBranch::If(nested)) => self.check_if_stmt(nested),
            Some(kyne_ast::ElseBranch::Block(block)) => self.check_block(block, None),
            None => {}
        }
    }

    fn check_match(&mut self, match_stmt: &MatchStmt, expected_value: Option<&Ty>) -> Ty {
        let subject_ty = self.check_expr(&match_stmt.subject, None);
        let mut result_ty: Option<Ty> = expected_value.cloned();
        for arm in &match_stmt.arms {
            self.push_scope();
            self.bind_pattern(&arm.pattern, &subject_ty);
            if let Some(guard) = &arm.guard {
                self.check_expr(guard, Some(&Ty::Bool));
            }
            let arm_ty = match &arm.body {
                MatchArmBody::Block(block) => {
                    if expected_value.is_some() {
                        Some(self.block_value_ty(block, expected_value))
                    } else {
                        self.check_block_statements(&block.statements, None);
                        None
                    }
                }
                MatchArmBody::Expr(e) => Some(self.check_expr(e, expected_value)),
                MatchArmBody::Throw(e) => {
                    let error_ty = self.current_error_ty();
                    self.check_expr(e, error_ty.as_ref());
                    None
                }
                MatchArmBody::Return(Some(e)) => {
                    let expected = self.fn_return_ty.clone();
                    self.check_expr(e, expected.as_ref());
                    None
                }
                MatchArmBody::Return(None) | MatchArmBody::Break | MatchArmBody::Continue => None,
            };
            self.pop_scope();
            if let Some(ty) = arm_ty {
                if result_ty.as_ref().is_none_or(|t| t.is_error()) {
                    result_ty = Some(ty);
                }
            }
        }
        result_ty.unwrap_or(Ty::Unit)
    }

    /// Binds the names a pattern introduces. `None` (bare, no
    /// parentheses) and a bare nullary enum variant name are treated as
    /// matching that variant, not as a fresh binding - see this crate's
    /// README for why the AST alone cannot distinguish the two syntactic
    /// shapes and this has to be resolved here, using `subject_ty`.
    fn bind_pattern(&mut self, pattern: &Pattern, subject_ty: &Ty) {
        match pattern {
            Pattern::Wildcard | Pattern::Literal(_) => {}
            Pattern::Ident(name) => {
                if name == "None" && matches!(subject_ty, Ty::Option(_)) {
                    return;
                }
                if let Ty::Named(enum_name) = subject_ty {
                    if let Some(EnumVariantTys::Unit) =
                        self.env.enums.get(enum_name).and_then(|v| v.get(name))
                    {
                        return;
                    }
                }
                self.declare_local(name, subject_ty.clone());
            }
            Pattern::Tuple { path, patterns } => match path {
                Path::Ident(name) if name == "Some" => {
                    self.bind_option_or_result_tuple(patterns, subject_ty, true)
                }
                Path::Ident(name) if name == "Ok" => {
                    self.bind_option_or_result_tuple(patterns, subject_ty, true)
                }
                Path::Ident(name) if name == "Err" => {
                    self.bind_option_or_result_tuple(patterns, subject_ty, false)
                }
                Path::Qualified(ty_name, variant) => {
                    let tys = match self.env.enums.get(ty_name).and_then(|v| v.get(variant)) {
                        Some(EnumVariantTys::Tuple(tys)) => tys.clone(),
                        _ => vec![Ty::Error; patterns.len()],
                    };
                    for (p, t) in patterns.iter().zip(tys.iter()) {
                        self.bind_pattern(p, t);
                    }
                }
                _ => {
                    for p in patterns {
                        self.bind_pattern(p, &Ty::Error);
                    }
                }
            },
            Pattern::Struct { path, fields } => {
                let field_tys = match path {
                    Path::Qualified(ty_name, variant) => {
                        match self.env.enums.get(ty_name).and_then(|v| v.get(variant)) {
                            Some(EnumVariantTys::Struct(fs)) => fs.clone(),
                            _ => Vec::new(),
                        }
                    }
                    _ => Vec::new(),
                };
                for field in fields {
                    let ty = field_tys
                        .iter()
                        .find(|(n, _)| n == field)
                        .map(|(_, t)| t.clone())
                        .unwrap_or(Ty::Error);
                    self.declare_local(field, ty);
                }
            }
        }
    }

    fn bind_option_or_result_tuple(
        &mut self,
        patterns: &[Pattern],
        subject_ty: &Ty,
        ok_slot: bool,
    ) {
        let inner = match (subject_ty, ok_slot) {
            (Ty::Option(t), true) => Some((**t).clone()),
            (Ty::Result(t, _), true) => Some((**t).clone()),
            (Ty::Result(_, e), false) => Some((**e).clone()),
            _ => None,
        };
        for p in patterns {
            self.bind_pattern(p, inner.as_ref().unwrap_or(&Ty::Error));
        }
    }

    // ---- Expressions ----

    fn check_expr(&mut self, expr: &Expr, expected: Option<&Ty>) -> Ty {
        match expr {
            Expr::Literal(Literal::Bool(_)) => Ty::Bool,
            Expr::Literal(Literal::Str(_)) => {
                if expected == Some(&Ty::Symbol) {
                    Ty::Symbol
                } else {
                    Ty::String
                }
            }
            Expr::Literal(Literal::Int(_)) => match expected {
                Some(t) if t.is_numeric() => t.clone(),
                _ => Ty::I64,
            },
            Expr::Path(path) => self.check_path(path, expected),
            Expr::Binary { left, op, right } => self.check_binary(left, *op, right),
            Expr::Unary { op, operand } => self.check_unary(*op, operand),
            Expr::Call { callee, args } => self.check_call(callee, args, expected),
            Expr::Field { base, name } => {
                let base_ty = self.check_expr(base, None);
                match &base_ty {
                    Ty::Named(struct_name) => match self.env.structs.get(struct_name) {
                        Some(fields) => match fields.iter().find(|(n, _)| n == name) {
                            Some((_, ty)) => ty.clone(),
                            None => self.error(
                                "KY0204",
                                format!("no field `{name}` on type `{struct_name}`"),
                                "unknown field",
                            ),
                        },
                        None => Ty::Error,
                    },
                    Ty::Error => Ty::Error,
                    other => self.error(
                        "KY0204",
                        format!("no field `{name}` on type `{}`", other.describe()),
                        "unknown field",
                    ),
                }
            }
            Expr::MethodCall { base, method, args } => self.check_method_call(base, method, args),
            Expr::Try { operand } => self.check_try(operand),
            Expr::StructLiteral { path, fields } => self.check_struct_literal(path, fields),
            Expr::ListLiteral(items) => self.check_list_literal(items, expected),
            Expr::MapLiteral(entries) => self.check_map_literal(entries, expected),
            Expr::If(if_stmt) => {
                self.check_expr(&if_stmt.condition, Some(&Ty::Bool));
                let then_ty = self.block_value_ty(&if_stmt.then_block, expected);
                let else_ty = match &if_stmt.else_branch {
                    Some(kyne_ast::ElseBranch::Block(block)) => {
                        Some(self.block_value_ty(block, expected.or(Some(&then_ty))))
                    }
                    Some(kyne_ast::ElseBranch::If(nested)) => Some(
                        self.check_expr(&Expr::If(nested.clone()), expected.or(Some(&then_ty))),
                    ),
                    None => None,
                };
                match else_ty {
                    Some(else_ty) if then_ty.compatible(&else_ty) => {
                        if then_ty.is_error() {
                            else_ty
                        } else {
                            then_ty
                        }
                    }
                    Some(else_ty) => self.error(
                        "KY0201",
                        format!(
                            "`if`/`else` branches have different types: `{}` and `{}`",
                            then_ty.describe(),
                            else_ty.describe()
                        ),
                        "branch type mismatch",
                    ),
                    None => then_ty,
                }
            }
            Expr::Match(match_stmt) => self.check_match(match_stmt, expected),
        }
    }

    fn check_path(&mut self, path: &Path, expected: Option<&Ty>) -> Ty {
        match path {
            Path::Ident(name) => {
                if name == "None" {
                    return expected
                        .cloned()
                        .unwrap_or_else(|| Ty::Option(Box::new(Ty::Error)));
                }
                self.lookup(name).unwrap_or(Ty::Error)
            }
            Path::Qualified(ty_name, _variant) => {
                if self.env.enums.contains_key(ty_name) || self.env.errors.contains_key(ty_name) {
                    Ty::Named(ty_name.clone())
                } else {
                    Ty::Error
                }
            }
        }
    }

    fn check_binary(&mut self, left: &Expr, op: kyne_ast::BinaryOp, right: &Expr) -> Ty {
        use kyne_ast::BinaryOp::*;
        let left_ty = self.check_expr(left, None);
        let right_ty = self.check_expr(right, Some(&left_ty));
        match op {
            Add | Sub | Mul | Div | Rem => {
                if left_ty.is_error() || right_ty.is_error() {
                    Ty::Error
                } else if left_ty.is_numeric() && left_ty == right_ty {
                    left_ty
                } else {
                    self.error(
                        "KY0203",
                        format!(
                            "cannot apply arithmetic to `{}` and `{}`",
                            left_ty.describe(),
                            right_ty.describe()
                        ),
                        "arithmetic requires two operands of the same numeric type",
                    )
                }
            }
            Eq | Ne => {
                if !left_ty.compatible(&right_ty) {
                    self.error(
                        "KY0201",
                        format!(
                            "cannot compare `{}` and `{}`",
                            left_ty.describe(),
                            right_ty.describe()
                        ),
                        "`==`/`!=` require both operands to have the same type",
                    );
                }
                Ty::Bool
            }
            Lt | Gt | Le | Ge => {
                if !left_ty.is_error()
                    && !right_ty.is_error()
                    && (!left_ty.is_numeric() || left_ty != right_ty)
                {
                    self.error(
                        "KY0203",
                        format!(
                            "cannot compare `{}` and `{}`",
                            left_ty.describe(),
                            right_ty.describe()
                        ),
                        "`<`/`>`/`<=`/`>=` require two operands of the same numeric type",
                    );
                }
                Ty::Bool
            }
            And | Or => {
                if !left_ty.compatible(&Ty::Bool) || !right_ty.compatible(&Ty::Bool) {
                    self.error(
                        "KY0201",
                        "`&&`/`||` require `bool` operands",
                        format!(
                            "found `{}` and `{}`",
                            left_ty.describe(),
                            right_ty.describe()
                        ),
                    );
                }
                Ty::Bool
            }
        }
    }

    fn check_unary(&mut self, op: kyne_ast::UnaryOp, operand: &Expr) -> Ty {
        let ty = self.check_expr(operand, None);
        match op {
            kyne_ast::UnaryOp::Neg if ty.is_numeric() || ty.is_error() => ty,
            kyne_ast::UnaryOp::Neg => self.error(
                "KY0203",
                format!("cannot negate `{}`", ty.describe()),
                "`-` requires a numeric operand",
            ),
            kyne_ast::UnaryOp::Not if ty.compatible(&Ty::Bool) => Ty::Bool,
            kyne_ast::UnaryOp::Not => self.error(
                "KY0201",
                format!("cannot apply `!` to `{}`", ty.describe()),
                "`!` requires a `bool` operand",
            ),
        }
    }

    fn check_call(&mut self, callee: &Expr, args: &[Expr], expected: Option<&Ty>) -> Ty {
        if let Expr::Path(Path::Ident(name)) = callee {
            match name.as_str() {
                "Some" => {
                    let inner_expected = match expected {
                        Some(Ty::Option(t)) => Some((**t).clone()),
                        _ => None,
                    };
                    let inner = args
                        .first()
                        .map(|a| self.check_expr(a, inner_expected.as_ref()))
                        .unwrap_or(Ty::Error);
                    return Ty::Option(Box::new(inner));
                }
                "Ok" => {
                    let (t_expected, e) = match expected {
                        Some(Ty::Result(t, e)) => (Some((**t).clone()), (**e).clone()),
                        _ => (None, Ty::Error),
                    };
                    let t = args
                        .first()
                        .map(|a| self.check_expr(a, t_expected.as_ref()))
                        .unwrap_or(Ty::Unit);
                    return Ty::Result(Box::new(t), Box::new(e));
                }
                "Err" => {
                    let (t, e_expected) = match expected {
                        Some(Ty::Result(t, e)) => ((**t).clone(), Some((**e).clone())),
                        _ => (Ty::Error, None),
                    };
                    let e = args
                        .first()
                        .map(|a| self.check_expr(a, e_expected.as_ref()))
                        .unwrap_or(Ty::Error);
                    return Ty::Result(Box::new(t), Box::new(e));
                }
                _ => {
                    if let Some(sig) = self.env.functions.get(name).cloned() {
                        self.check_call_args(name, &sig.params, args);
                        return sig.return_ty;
                    }
                }
            }
        }
        if let Expr::Path(Path::Qualified(ty_name, variant)) = callee {
            if let Some(EnumVariantTys::Tuple(param_tys)) = self
                .env
                .enums
                .get(ty_name)
                .and_then(|v| v.get(variant))
                .cloned()
            {
                self.check_call_args(variant, &param_tys, args);
                return Ty::Named(ty_name.clone());
            }
        }
        // Unknown callee shape - still visit every argument so nested
        // errors are not silently skipped.
        for arg in args {
            self.check_expr(arg, None);
        }
        self.check_expr(callee, None);
        Ty::Error
    }

    fn check_call_args(&mut self, name: &str, params: &[Ty], args: &[Expr]) {
        if params.len() != args.len() {
            self.error(
                "KY0202",
                format!(
                    "`{name}` expects {} argument(s), found {}",
                    params.len(),
                    args.len()
                ),
                "argument count mismatch",
            );
        }
        for (i, arg) in args.iter().enumerate() {
            let expected = params.get(i);
            let actual = self.check_expr(arg, expected);
            if let Some(expected) = expected {
                if !expected.compatible(&actual) {
                    self.error(
                        "KY0201",
                        format!(
                            "expected `{}`, found `{}`",
                            expected.describe(),
                            actual.describe()
                        ),
                        format!("argument {} to `{name}`", i + 1),
                    );
                }
            }
        }
    }

    fn check_method_call(&mut self, base: &Expr, method: &str, args: &[Expr]) -> Ty {
        let base_ty = self.check_expr(base, None);
        let arg_tys: Vec<Ty> = args.iter().map(|a| self.check_expr(a, None)).collect();
        match &base_ty {
            Ty::List(elem) => match method {
                "push" if arg_tys.len() == 1 => Ty::Unit,
                "get" if arg_tys.len() == 1 => Ty::Option(elem.clone()),
                "len" if arg_tys.is_empty() => Ty::U32,
                "remove" if arg_tys.len() == 1 => Ty::Unit,
                _ => self.unknown_method(method, &base_ty),
            },
            Ty::Map(_, v) => match method {
                "get" if arg_tys.len() == 1 => Ty::Option(v.clone()),
                "set" if arg_tys.len() == 2 => Ty::Unit,
                "has" if arg_tys.len() == 1 => Ty::Bool,
                "remove" if arg_tys.len() == 1 => Ty::Unit,
                "len" if arg_tys.is_empty() => Ty::U32,
                _ => self.unknown_method(method, &base_ty),
            },
            Ty::Option(t) => match method {
                "is_some" | "is_none" if arg_tys.is_empty() => Ty::Bool,
                "unwrap_or" if arg_tys.len() == 1 => (**t).clone(),
                _ => self.unknown_method(method, &base_ty),
            },
            Ty::Result(t, _) => match method {
                "is_ok" | "is_err" if arg_tys.is_empty() => Ty::Bool,
                "unwrap_or" if arg_tys.len() == 1 => (**t).clone(),
                _ => self.unknown_method(method, &base_ty),
            },
            Ty::Error => Ty::Error,
            other => self.unknown_method(method, other),
        }
    }

    fn unknown_method(&mut self, method: &str, base_ty: &Ty) -> Ty {
        self.error(
            "KY0204",
            format!("no method `{method}` on type `{}`", base_ty.describe()),
            "unknown method",
        )
    }

    fn check_try(&mut self, operand: &Expr) -> Ty {
        let operand_ty = self.check_expr(operand, None);
        match (&operand_ty, self.current_error_ty()) {
            (Ty::Result(t, e), Some(fn_error_ty)) => {
                if e.compatible(&fn_error_ty) {
                    (**t).clone()
                } else {
                    self.error(
                        "KY0201",
                        format!("`?` produces error type `{}`, but the enclosing function returns `Result<_, {}>`", e.describe(), fn_error_ty.describe()),
                        "mismatched error type",
                    )
                }
            }
            (Ty::Result(t, _), None) => self.error(
                "KY0201",
                "`?` is only valid inside a function returning `Result<T, E>`",
                format!("would produce `{}`", t.describe()),
            ),
            (Ty::Error, _) => Ty::Error,
            (other, _) => self.error(
                "KY0201",
                format!(
                    "`?` requires a `Result<T, E>`, found `{}`",
                    other.describe()
                ),
                "invalid `?`",
            ),
        }
    }

    fn check_struct_literal(&mut self, path: &Path, fields: &[kyne_ast::FieldInit]) -> Ty {
        let (result_ty, declared_fields) = match path {
            Path::Ident(name) => (Ty::Named(name.clone()), self.env.structs.get(name).cloned()),
            Path::Qualified(ty_name, variant) => {
                let shape = self.env.enums.get(ty_name).and_then(|v| v.get(variant));
                let declared = match shape {
                    Some(EnumVariantTys::Struct(fs)) => Some(fs.clone()),
                    _ => None,
                };
                (Ty::Named(ty_name.clone()), declared)
            }
        };
        for field in fields {
            let expected = declared_fields
                .as_ref()
                .and_then(|fs| fs.iter().find(|(n, _)| n == &field.name))
                .map(|(_, t)| t.clone());
            match expected {
                Some(expected) => {
                    self.check_expr_expect(&field.value, &expected, "struct field");
                }
                None => {
                    self.check_expr(&field.value, None);
                }
            }
        }
        result_ty
    }

    fn check_list_literal(&mut self, items: &[Expr], expected: Option<&Ty>) -> Ty {
        let elem_expected = match expected {
            Some(Ty::List(t)) => Some((**t).clone()),
            _ => None,
        };
        let mut elem_ty = elem_expected.clone();
        for item in items {
            let ty = self.check_expr(item, elem_expected.as_ref());
            if elem_ty.is_none() {
                elem_ty = Some(ty);
            }
        }
        Ty::List(Box::new(elem_ty.unwrap_or(Ty::Error)))
    }

    fn check_map_literal(&mut self, entries: &[kyne_ast::MapEntry], expected: Option<&Ty>) -> Ty {
        let (k_expected, v_expected) = match expected {
            Some(Ty::Map(k, v)) => (Some((**k).clone()), Some((**v).clone())),
            _ => (None, None),
        };
        let mut k_ty = k_expected.clone();
        let mut v_ty = v_expected.clone();
        for entry in entries {
            let k = self.check_expr(&entry.key, k_expected.as_ref());
            let v = self.check_expr(&entry.value, v_expected.as_ref());
            if k_ty.is_none() {
                k_ty = Some(k);
            }
            if v_ty.is_none() {
                v_ty = Some(v);
            }
        }
        Ty::Map(
            Box::new(k_ty.unwrap_or(Ty::Error)),
            Box::new(v_ty.unwrap_or(Ty::Error)),
        )
    }
}

fn malformed_type_diagnostic(file: &str, message: &str) -> Diagnostic {
    Diagnostic::error("KY0200", message.to_string(), file, placeholder_span(), "malformed type")
        .note("kyne_ast does not yet carry source-span information, so this diagnostic cannot point at the exact offending location - see this crate's README")
}

#[cfg(test)]
mod tests {
    use super::*;
    use kyne_cst::Cst;

    fn check_source(source: &str) -> CheckResult {
        let (cst, diagnostics) = Cst::parse(source, "test.kyn");
        assert!(
            diagnostics.is_empty(),
            "expected a clean parse, got: {diagnostics:?}"
        );
        let program = kyne_ast::lower(&cst);
        check(&program, "test.kyn")
    }

    fn codes(result: &CheckResult) -> Vec<&str> {
        result.diagnostics.iter().map(|d| d.code()).collect()
    }

    fn assert_clean(source: &str) {
        let result = check_source(source);
        assert!(
            result.diagnostics.is_empty(),
            "expected no diagnostics, got: {:?}",
            result.diagnostics
        );
    }

    // ---- Every primitive type ----

    #[test]
    fn every_primitive_type_checks() {
        assert_clean(
            "contract C {
                state a: bool = true;
                state b: i32 = 1;
                state c: i64 = 1;
                state d: i128 = 1;
                state e: u32 = 1;
                state f: u64 = 1;
                state g: u128 = 1;
                state h: string = \"x\";
                state i: symbol = \"x\";
            }\n",
        );
    }

    // ---- The five closed intrinsic parametric types ----

    #[test]
    fn list_type_and_methods() {
        assert_clean(
            "contract C {
                fn f(xs: list<i64>) -> u32 {
                    let y = xs.get(0);
                    xs.push(1);
                    return xs.len();
                }
            }\n",
        );
    }

    #[test]
    fn map_type_and_methods() {
        assert_clean(
            "contract C {
                state balances: map<address, i128>;
                fn f(a: address) -> i128 {
                    return balances.get(a).unwrap_or(0);
                }
            }\n",
        );
    }

    #[test]
    fn bytes_n_type() {
        assert_clean("contract C { state h: bytes<32>; }\n");
    }

    #[test]
    fn option_type_and_methods() {
        assert_clean(
            "contract C {
                fn f(x: Option<i64>) -> bool {
                    return x.is_some();
                }
            }\n",
        );
    }

    #[test]
    fn result_type_and_methods() {
        assert_clean(
            "error E { X }
             contract C {
                fn f(x: Result<i64, E>) -> bool {
                    return x.is_ok();
                }
            }\n",
        );
    }

    // ---- Inference boundary: exactly at `let` ----

    #[test]
    fn let_infers_type_from_initializer() {
        assert_clean("contract C { fn f() { let x = 1; let y: i64 = x; } }\n");
    }

    #[test]
    fn let_annotation_mismatch_is_an_error() {
        let result = check_source("contract C { fn f() { let x: address = 1; } }\n");
        assert_eq!(codes(&result), vec!["KY0201"]);
    }

    #[test]
    fn state_field_type_is_never_inferred_and_required_explicitly() {
        // `state` always carries an explicit type in the grammar itself
        // (LetStmt's optional annotation is a LetStmt-only production) -
        // this test instead confirms a state initializer is checked
        // against its declared type, not treated as an inference site.
        let result = check_source("contract C { state n: address = 1; }\n");
        assert_eq!(codes(&result), vec!["KY0201"]);
    }

    // ---- Arithmetic and comparison ----

    #[test]
    fn arithmetic_requires_matching_numeric_types() {
        let result = check_source("contract C { fn f(a: i64, b: u64) { let x = a + b; } }\n");
        assert_eq!(codes(&result), vec!["KY0203"]);
    }

    #[test]
    fn arithmetic_on_non_numeric_type_is_an_error() {
        let result =
            check_source("contract C { fn f(a: address, b: address) { let x = a + b; } }\n");
        assert_eq!(codes(&result), vec!["KY0203"]);
    }

    #[test]
    fn checked_arithmetic_on_matching_types_is_fine() {
        assert_clean("contract C { fn f(a: i64, b: i64) -> i64 { return a + b; } }\n");
    }

    #[test]
    fn integer_literal_defaults_to_i64_with_no_context() {
        assert_clean("contract C { fn f() -> i64 { return 1; } }\n");
    }

    #[test]
    fn integer_literal_takes_the_expected_numeric_type() {
        assert_clean("contract C { fn f() -> u32 { return 1; } }\n");
    }

    #[test]
    fn comparison_requires_matching_numeric_types() {
        let result =
            check_source("contract C { fn f(a: i64, b: u64) -> bool { return a < b; } }\n");
        assert_eq!(codes(&result), vec!["KY0203"]);
    }

    #[test]
    fn no_implicit_conversion_between_numeric_types() {
        let result = check_source("contract C { fn f(a: i32) -> i64 { return a; } }\n");
        assert_eq!(codes(&result), vec!["KY0201"]);
    }

    // ---- Assignment ----

    #[test]
    fn assignment_type_mismatch_is_an_error() {
        let result = check_source("contract C { state n: i64 = 0; fn f() { n = true; } }\n");
        assert_eq!(codes(&result), vec!["KY0201"]);
    }

    #[test]
    fn compound_assign_requires_a_numeric_target() {
        let result = check_source("contract C { state b: bool = true; fn f() { b += true; } }\n");
        // Compound assign to a non-numeric target is itself an error
        // (KY0203); the RHS `true` against `bool` is otherwise fine, so
        // exactly one diagnostic is expected.
        assert_eq!(codes(&result), vec!["KY0203"]);
    }

    // ---- Functions, calls, and `Result`/`?` ----

    #[test]
    fn call_argument_count_mismatch_is_an_error() {
        let result = check_source("contract C { fn g(a: i64) {} fn f() { g(1, 2); } }\n");
        assert_eq!(codes(&result), vec!["KY0202"]);
    }

    #[test]
    fn call_argument_type_mismatch_is_an_error() {
        let result = check_source("contract C { fn g(a: address) {} fn f() { g(1); } }\n");
        assert_eq!(codes(&result), vec!["KY0201"]);
    }

    #[test]
    fn try_operator_requires_matching_error_type() {
        assert_clean(
            "error E { X }
             contract C {
                fn g() -> Result<i64, E> { return Ok(1); }
                fn f() -> Result<i64, E> {
                    let x = g()?;
                    return Ok(x);
                }
            }\n",
        );
    }

    #[test]
    fn try_operator_with_mismatched_error_type_is_an_error() {
        let result = check_source(
            "error E1 { X }
             error E2 { Y }
             contract C {
                fn g() -> Result<i64, E1> { return Ok(1); }
                fn f() -> Result<i64, E2> {
                    let x = g()?;
                    return Ok(x);
                }
            }\n",
        );
        assert_eq!(codes(&result), vec!["KY0201"]);
    }

    #[test]
    fn throw_checks_against_the_enclosing_functions_error_type() {
        assert_clean(
            "error E { X }
             contract C {
                fn f() -> Result<i64, E> {
                    throw E::X;
                }
            }\n",
        );
    }

    // ---- Structs and enums ----

    #[test]
    fn struct_literal_field_types_are_checked() {
        let result = check_source(
            "struct Point { x: i64, y: i64 }
             contract C { fn f() { let p = Point { x: true, y: 1 }; } }\n",
        );
        assert_eq!(codes(&result), vec!["KY0201"]);
    }

    #[test]
    fn unknown_field_access_is_an_error() {
        let result = check_source(
            "struct Point { x: i64 }
             contract C { fn f(p: Point) -> i64 { return p.y; } }\n",
        );
        assert_eq!(codes(&result), vec!["KY0204"]);
    }

    #[test]
    fn enum_tuple_variant_construction_and_pattern_are_checked() {
        assert_clean(
            "enum Status { Approved(address) }
             contract C {
                fn f(status: Status, admin: address) -> bool {
                    match status {
                        Status::Approved(by) => { return by == admin; }
                        _ => { return false; }
                    }
                }
                fn g(admin: address) -> Status {
                    return Status::Approved(admin);
                }
            }\n",
        );
    }

    #[test]
    fn if_expression_branches_must_have_the_same_type() {
        let result = check_source("contract C { fn f(cond: bool) -> i64 { let x = if cond { 1 } else { true }; return x; } }\n");
        assert_eq!(codes(&result), vec!["KY0201"]);
    }

    #[test]
    fn if_expression_with_matching_branches_is_fine() {
        assert_clean("contract C { fn f(cond: bool) -> i64 { let x = if cond { 1 } else { 2 }; return x; } }\n");
    }
}
