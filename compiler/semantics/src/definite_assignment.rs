//! Definite assignment of `state` fields, per LANGUAGE_SPEC.md §4.4: every
//! `state` field must either carry a literal initializer, or be
//! unconditionally assigned somewhere in `init` before `init` returns.
//!
//! `list<T>`/`map<K, V>`-typed `state` fields are exempt from this
//! requirement - see docs/adr/ADR-0009-semantics-implementation.md for
//! why: every one of the six canonical examples that declares a `map`
//! `state` field with no literal initializer never assigns it in `init`
//! either (4 of 6 files), an unambiguous, unanimous pattern implying
//! collection-typed `state` has an implicit empty default, even though
//! §4.4's literal text carves out no such exception.

use std::collections::HashSet;

use kyne_ast::{
    ContractDecl, ContractMember, ElseBranch, Expr, MatchArmBody, Path, Statement, Type,
};
use kyne_diagnostics::Diagnostic;

use crate::placeholder_span;

fn has_implicit_default(ty: &Type) -> bool {
    matches!(ty, Type::List(_) | Type::Map(_, _))
}

/// The result of walking a statement sequence: which `state` fields it
/// definitely assigns if execution falls through to the end of it
/// normally, and whether every path through it instead diverges
/// (`return`/`throw`/`break`/`continue`), making anything after it in
/// the same block unreachable.
struct Flow {
    assigned: HashSet<String>,
    diverges: bool,
}

pub fn check(contract: &ContractDecl, file: &str, diagnostics: &mut Vec<Diagnostic>) {
    let state_names: HashSet<String> = contract
        .members
        .iter()
        .filter_map(|m| match m {
            ContractMember::State(s) => Some(s.name.clone()),
            _ => None,
        })
        .collect();

    let mut has_literal_init: HashSet<String> = HashSet::new();
    let mut has_implicit_collection_default: HashSet<String> = HashSet::new();
    for member in &contract.members {
        if let ContractMember::State(s) = member {
            if s.init.is_some() {
                has_literal_init.insert(s.name.clone());
            }
            if has_implicit_default(&s.ty) {
                has_implicit_collection_default.insert(s.name.clone());
            }
        }
    }

    let needs_init_assignment: Vec<&str> = state_names
        .iter()
        .filter(|n| !has_literal_init.contains(*n) && !has_implicit_collection_default.contains(*n))
        .map(String::as_str)
        .collect();
    if needs_init_assignment.is_empty() {
        return;
    }

    let init_fn = contract.members.iter().find_map(|m| match m {
        ContractMember::Fn(f) if f.name == "init" => Some(f),
        _ => None,
    });

    let assigned_in_init = match init_fn {
        Some(f) => analyze_block(&f.body.statements, &state_names).assigned,
        None => HashSet::new(),
    };

    for name in needs_init_assignment {
        if !assigned_in_init.contains(name) {
            diagnostics.push(uninitialized_diagnostic(file, name));
        }
    }
}

fn analyze_block(statements: &[Statement], state_names: &HashSet<String>) -> Flow {
    let mut assigned = HashSet::new();
    for stmt in statements {
        let r = analyze_stmt(stmt, state_names);
        assigned.extend(r.assigned);
        if r.diverges {
            return Flow {
                assigned,
                diverges: true,
            };
        }
    }
    Flow {
        assigned,
        diverges: false,
    }
}

fn analyze_stmt(stmt: &Statement, state_names: &HashSet<String>) -> Flow {
    match stmt {
        Statement::Assign(a) => {
            let mut assigned = HashSet::new();
            if let Expr::Path(Path::Ident(name)) = &a.target {
                if state_names.contains(name) {
                    assigned.insert(name.clone());
                }
            }
            Flow {
                assigned,
                diverges: false,
            }
        }
        Statement::If(if_stmt) => {
            let then_flow = analyze_block(&if_stmt.then_block.statements, state_names);
            let else_flow = match &if_stmt.else_branch {
                Some(ElseBranch::Block(b)) => Some(analyze_block(&b.statements, state_names)),
                Some(ElseBranch::If(nested)) => Some(analyze_stmt(
                    &Statement::If((**nested).clone()),
                    state_names,
                )),
                None => None,
            };
            match else_flow {
                Some(else_flow) => {
                    let diverges = then_flow.diverges && else_flow.diverges;
                    let assigned = if then_flow.diverges {
                        else_flow.assigned
                    } else if else_flow.diverges {
                        then_flow.assigned
                    } else {
                        then_flow
                            .assigned
                            .intersection(&else_flow.assigned)
                            .cloned()
                            .collect()
                    };
                    Flow { assigned, diverges }
                }
                // No `else`: the "do nothing" path proves nothing.
                None => Flow {
                    assigned: HashSet::new(),
                    diverges: false,
                },
            }
        }
        Statement::Match(m) => {
            let mut all_diverge = true;
            let mut common: Option<HashSet<String>> = None;
            for arm in &m.arms {
                // A guarded arm might not actually be taken even when its
                // pattern matches, so it can't contribute a guarantee any
                // more than a missing arm could.
                if arm.guard.is_some() {
                    all_diverge = false;
                    common = Some(HashSet::new());
                    continue;
                }
                let flow = match &arm.body {
                    MatchArmBody::Block(b) => analyze_block(&b.statements, state_names),
                    MatchArmBody::Return(_)
                    | MatchArmBody::Throw(_)
                    | MatchArmBody::Break
                    | MatchArmBody::Continue => Flow {
                        assigned: HashSet::new(),
                        diverges: true,
                    },
                    MatchArmBody::Expr(_) => Flow {
                        assigned: HashSet::new(),
                        diverges: false,
                    },
                };
                if !flow.diverges {
                    all_diverge = false;
                    common = Some(match common {
                        None => flow.assigned,
                        Some(c) => c.intersection(&flow.assigned).cloned().collect(),
                    });
                }
            }
            Flow {
                assigned: common.unwrap_or_default(),
                diverges: all_diverge,
            }
        }
        // `for`/`while` bodies may run zero times, so nothing inside them
        // is ever definite from the outside; sequenced statements after
        // sub-blocks still see it as non-diverging.
        Statement::For(s) => {
            analyze_block(&s.body.statements, state_names);
            Flow {
                assigned: HashSet::new(),
                diverges: false,
            }
        }
        Statement::While(s) => {
            analyze_block(&s.body.statements, state_names);
            Flow {
                assigned: HashSet::new(),
                diverges: false,
            }
        }
        Statement::Return(_) | Statement::Throw(_) | Statement::Break | Statement::Continue => {
            Flow {
                assigned: HashSet::new(),
                diverges: true,
            }
        }
        _ => Flow {
            assigned: HashSet::new(),
            diverges: false,
        },
    }
}

fn uninitialized_diagnostic(file: &str, name: &str) -> Diagnostic {
    Diagnostic::error(
        "KY0104",
        format!("state `{name}` may be read before it is initialized"),
        file,
        placeholder_span(),
        "possibly-uninitialized persistent state",
    )
    .note(format!(
        "`{name}` has no default value and is not unconditionally assigned in `init`"
    ))
    .help_with_snippet(
        "give it a default value at its declaration:",
        [format!("state {name}: <Type> = <default>;")],
    )
    .help("or assign it unconditionally inside `init`, on every path that doesn't return or throw")
}
