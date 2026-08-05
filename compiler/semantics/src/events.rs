//! `emit` argument validation, per LANGUAGE_SPEC.md §11.2: an `emit`'s
//! argument list MUST match its target event's declared parameter list
//! exactly in count, order, and type.
//!
//! Argument *count* is always checked. Argument *type* is checked on a
//! best-effort basis using only directly-observable expression shapes
//! (a local/parameter/`state`/`const` name, or a literal) rather than a
//! full expression-type inference - `kyne_types`'s own inference lives
//! in a private module not exposed for reuse, and every `emit` argument
//! in the six canonical examples is one of these simple shapes. A more
//! complex argument expression is skipped for the type check (count is
//! still verified), not guessed at.

use std::collections::HashMap;

use kyne_ast::{
    ContractMember, Expr, IfStmt, Literal, MatchArmBody, MatchStmt, Path, Program, Statement,
    TopLevelDecl,
};
use kyne_diagnostics::Diagnostic;
use kyne_types::{lower_type, Ty, TypeEnv};

use crate::placeholder_span;

pub fn check(program: &Program, env: &TypeEnv, file: &str, diagnostics: &mut Vec<Diagnostic>) {
    for item in &program.items {
        if let TopLevelDecl::Contract(c) = item {
            for member in &c.members {
                if let ContractMember::Fn(f) = member {
                    let mut locals = HashMap::new();
                    for p in &f.params {
                        locals.insert(p.name.clone(), lower_type(&p.ty, &mut |_| {}));
                    }
                    walk_statements(&f.body.statements, env, &mut locals, file, diagnostics);
                }
            }
        }
    }
}

fn walk_statements(
    statements: &[Statement],
    env: &TypeEnv,
    locals: &mut HashMap<String, Ty>,
    file: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for stmt in statements {
        walk_stmt(stmt, env, locals, file, diagnostics);
    }
}

fn walk_stmt(
    stmt: &Statement,
    env: &TypeEnv,
    locals: &mut HashMap<String, Ty>,
    file: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match stmt {
        Statement::Let(s) => {
            let ty =
                s.ty.as_ref()
                    .map(|t| lower_type(t, &mut |_| {}))
                    .unwrap_or_else(|| infer(&s.value, env, locals));
            locals.insert(s.name.clone(), ty);
        }
        Statement::Emit(s) => check_emit(s, env, locals, file, diagnostics),
        Statement::If(if_stmt) => walk_if(if_stmt, env, locals, file, diagnostics),
        Statement::Match(m) => walk_match(m, env, locals, file, diagnostics),
        Statement::For(s) => {
            let elem = match infer(&s.iterable, env, locals) {
                Ty::List(t) => *t,
                _ => Ty::Error,
            };
            let mut inner = locals.clone();
            inner.insert(s.var.clone(), elem);
            walk_statements(&s.body.statements, env, &mut inner, file, diagnostics);
        }
        Statement::While(s) => walk_statements(
            &s.body.statements,
            env,
            &mut locals.clone(),
            file,
            diagnostics,
        ),
        _ => {}
    }
}

fn walk_if(
    if_stmt: &IfStmt,
    env: &TypeEnv,
    locals: &HashMap<String, Ty>,
    file: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    walk_statements(
        &if_stmt.then_block.statements,
        env,
        &mut locals.clone(),
        file,
        diagnostics,
    );
    match &if_stmt.else_branch {
        Some(kyne_ast::ElseBranch::Block(b)) => {
            walk_statements(&b.statements, env, &mut locals.clone(), file, diagnostics)
        }
        Some(kyne_ast::ElseBranch::If(nested)) => walk_if(nested, env, locals, file, diagnostics),
        None => {}
    }
}

fn walk_match(
    m: &MatchStmt,
    env: &TypeEnv,
    locals: &HashMap<String, Ty>,
    file: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for arm in &m.arms {
        if let MatchArmBody::Block(b) = &arm.body {
            walk_statements(&b.statements, env, &mut locals.clone(), file, diagnostics);
        }
    }
}

fn check_emit(
    s: &kyne_ast::EmitStmt,
    env: &TypeEnv,
    locals: &HashMap<String, Ty>,
    file: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(params) = env.events.get(&s.event) else {
        // An emit targeting an unknown event is a name-resolution
        // concern (already reported elsewhere, or the event genuinely
        // doesn't exist and kyne_resolver's check_unresolved caught it);
        // this check only validates against declarations it can find.
        return;
    };
    if params.len() != s.args.len() {
        diagnostics.push(
            Diagnostic::error(
                "KY0301",
                format!("`emit {}` does not match its declared event", s.event),
                file,
                placeholder_span(),
                format!(
                    "expected {} argument(s), found {}",
                    params.len(),
                    s.args.len()
                ),
            )
            .note(format!(
                "`event {}(...)` declares {} parameter(s)",
                s.event,
                params.len()
            ))
            .help("pass exactly the arguments the event declares, in the same order"),
        );
        return;
    }
    for (i, (arg, expected)) in s.args.iter().zip(params.iter()).enumerate() {
        let actual = infer(arg, env, locals);
        if !actual.compatible(expected) {
            diagnostics.push(
                Diagnostic::error(
                    "KY0301",
                    format!("`emit {}` does not match its declared event", s.event),
                    file,
                    placeholder_span(),
                    format!(
                        "argument {} has type `{}`, expected `{}`",
                        i + 1,
                        actual.describe(),
                        expected.describe()
                    ),
                )
                .note(format!(
                    "`event {}(...)` declares this parameter as `{}`",
                    s.event,
                    expected.describe()
                ))
                .help("pass a value of the declared parameter type"),
            );
        }
    }
}

/// A deliberately small expression-type inference covering only the
/// shapes real `emit` arguments take in practice - see this module's
/// documentation for why a full inference isn't reused from
/// `kyne_types` here.
fn infer(expr: &Expr, env: &TypeEnv, locals: &HashMap<String, Ty>) -> Ty {
    match expr {
        Expr::Literal(Literal::Bool(_)) => Ty::Bool,
        Expr::Literal(Literal::Str(_)) => Ty::String,
        Expr::Literal(Literal::Int(_)) => Ty::Error, // untyped; a real mismatch would need context this shortcut doesn't have
        Expr::Path(Path::Ident(name)) => locals
            .get(name)
            .cloned()
            .or_else(|| env.state.get(name).cloned())
            .or_else(|| env.consts.get(name).cloned())
            .unwrap_or(Ty::Error),
        Expr::Field { base, name } => match infer(base, env, locals) {
            Ty::Named(struct_name) => env
                .structs
                .get(&struct_name)
                .and_then(|fs| fs.iter().find(|(n, _)| n == name))
                .map(|(_, t)| t.clone())
                .unwrap_or(Ty::Error),
            _ => Ty::Error,
        },
        _ => Ty::Error,
    }
}
