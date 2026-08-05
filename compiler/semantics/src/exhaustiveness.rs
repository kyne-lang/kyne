//! Exhaustive `match` checking, per LANGUAGE_SPEC.md §7.7: a `match`
//! over an enum MUST cover every variant, unless a wildcard `_` arm (or
//! an unguarded bare-identifier binding, which matches anything) is
//! present.
//!
//! Determining which enum a `match`'s subject has requires re-deriving
//! type information `kyne_types` already computed once, since
//! `kyne_ast` carries no inline type annotations (see this crate's
//! README). Rather than duplicate a second type checker, this check
//! infers the subject's shape directly from the patterns actually
//! written - every real pattern for an enum variant names it via
//! `Type::Variant` (or the prelude's bare `Some`/`None`/`Ok`/`Err`),
//! which is enough to identify both the enum and its full variant set
//! without re-inferring the subject expression's type from scratch.

use std::collections::HashSet;

use kyne_ast::{FnDecl, MatchStmt, Path, Pattern, Program, Statement, TopLevelDecl};
use kyne_diagnostics::Diagnostic;
use kyne_types::TypeEnv;

use crate::placeholder_span;

pub fn check(program: &Program, env: &TypeEnv, file: &str, diagnostics: &mut Vec<Diagnostic>) {
    for item in &program.items {
        match item {
            TopLevelDecl::Contract(c) => {
                for member in &c.members {
                    if let kyne_ast::ContractMember::Fn(f) = member {
                        check_fn(f, env, file, diagnostics);
                    }
                }
            }
            TopLevelDecl::Fn(f) => check_fn(f, env, file, diagnostics),
            _ => {}
        }
    }
}

fn check_fn(f: &FnDecl, env: &TypeEnv, file: &str, diagnostics: &mut Vec<Diagnostic>) {
    walk_statements(&f.body.statements, env, file, diagnostics);
}

fn walk_statements(
    statements: &[Statement],
    env: &TypeEnv,
    file: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for stmt in statements {
        walk_stmt(stmt, env, file, diagnostics);
    }
}

fn walk_stmt(stmt: &Statement, env: &TypeEnv, file: &str, diagnostics: &mut Vec<Diagnostic>) {
    match stmt {
        Statement::Match(m) => {
            check_match(m, env, file, diagnostics);
            for arm in &m.arms {
                if let kyne_ast::MatchArmBody::Block(b) = &arm.body {
                    walk_statements(&b.statements, env, file, diagnostics);
                }
            }
        }
        Statement::If(if_stmt) => {
            walk_statements(&if_stmt.then_block.statements, env, file, diagnostics);
            match &if_stmt.else_branch {
                Some(kyne_ast::ElseBranch::Block(b)) => {
                    walk_statements(&b.statements, env, file, diagnostics)
                }
                Some(kyne_ast::ElseBranch::If(nested)) => {
                    walk_stmt(&Statement::If((**nested).clone()), env, file, diagnostics)
                }
                None => {}
            }
        }
        Statement::For(s) => walk_statements(&s.body.statements, env, file, diagnostics),
        Statement::While(s) => walk_statements(&s.body.statements, env, file, diagnostics),
        _ => {}
    }
}

fn check_match(m: &MatchStmt, env: &TypeEnv, file: &str, diagnostics: &mut Vec<Diagnostic>) {
    let mut has_catch_all = false;
    let mut covered: HashSet<String> = HashSet::new();
    let mut enum_name: Option<&str> = None;
    let mut is_option = false;
    let mut is_result = false;

    for arm in &m.arms {
        if arm.guard.is_some() {
            continue; // a guarded arm never counts toward exhaustiveness
        }
        match &arm.pattern {
            Pattern::Wildcard => has_catch_all = true,
            Pattern::Ident(_) => has_catch_all = true, // an unguarded bare binding matches anything
            Pattern::Tuple { path, .. } | Pattern::Struct { path, .. } => match path {
                Path::Ident(name) if name == "Some" => {
                    is_option = true;
                    covered.insert("Some".to_string());
                }
                Path::Ident(name) if name == "Ok" => {
                    is_result = true;
                    covered.insert("Ok".to_string());
                }
                Path::Ident(name) if name == "Err" => {
                    is_result = true;
                    covered.insert("Err".to_string());
                }
                Path::Qualified(ty_name, variant) => {
                    enum_name = Some(ty_name);
                    covered.insert(variant.clone());
                }
                _ => {}
            },
            Pattern::Literal(_) => {
                // Literal-pattern exhaustiveness over an unbounded domain
                // can never be proven without a wildcard; treat any
                // literal pattern's presence as requiring one.
                enum_name = None;
            }
        }
    }
    if let Some(Pattern::Ident(name)) = m
        .arms
        .iter()
        .find(|a| a.guard.is_none())
        .map(|a| &a.pattern)
    {
        if name == "None" {
            is_option = true;
            covered.insert("None".to_string());
        }
    }

    if has_catch_all {
        return;
    }

    if is_option {
        report_missing(&covered, &["Some", "None"], "Option<T>", file, diagnostics);
    } else if is_result {
        report_missing(&covered, &["Ok", "Err"], "Result<T, E>", file, diagnostics);
    } else if let Some(name) = enum_name {
        if let Some(variants) = env.enums.get(name) {
            let all: Vec<&str> = variants.keys().map(String::as_str).collect();
            report_missing(&covered, &all, name, file, diagnostics);
        }
    }
}

fn report_missing(
    covered: &HashSet<String>,
    required: &[&str],
    subject: &str,
    file: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let missing: Vec<&str> = required
        .iter()
        .filter(|v| !covered.contains(**v))
        .copied()
        .collect();
    if missing.is_empty() {
        return;
    }
    diagnostics.push(
        Diagnostic::error(
            "KY0701",
            "`match` is not exhaustive",
            file,
            placeholder_span(),
            format!("missing variant(s): {}", missing.join(", ")),
        )
        .note(format!("`{subject}` has a variant this `match` does not cover, and no wildcard `_` arm is present"))
        .help(format!("add an arm for {}, or a trailing `_ => ...` arm", missing.join(", "))),
    );
}
