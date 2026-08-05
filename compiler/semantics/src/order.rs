//! Canonical contract member ordering, per LANGUAGE_SPEC.md §3.2 - a hard
//! compiler error, not a formatter convention.

use kyne_ast::{ContractDecl, ContractMember};
use kyne_diagnostics::Diagnostic;

use crate::placeholder_span;

/// The nine canonical categories, in required order, per §3.2.
fn category(member: &ContractMember) -> (u8, &'static str) {
    match member {
        ContractMember::State(_) => (1, "state"),
        ContractMember::Const(_) => (2, "const"),
        ContractMember::Error(_) => (3, "error"),
        ContractMember::Event(_) => (4, "event"),
        ContractMember::Struct(_) | ContractMember::Enum(_) => (5, "local struct/enum"),
        ContractMember::Fn(f) if f.name == "init" => (6, "`fn init`"),
        ContractMember::Fn(f) if f.visibility == kyne_ast::Visibility::Public => (7, "`public fn`"),
        ContractMember::Fn(f) if f.visibility == kyne_ast::Visibility::Internal => {
            (8, "`internal fn`")
        }
        ContractMember::Fn(_) => (9, "unmarked `fn`"),
    }
}

pub fn check(contract: &ContractDecl, file: &str, diagnostics: &mut Vec<Diagnostic>) {
    let mut previous: Option<(u8, &'static str)> = None;
    for member in &contract.members {
        let (cat, label) = category(member);
        if let Some((prev_cat, prev_label)) = previous {
            if cat < prev_cat {
                diagnostics.push(out_of_order_diagnostic(file, label, prev_label));
            }
        }
        // Always advance the baseline to the category just seen, even
        // when it was itself flagged as a violation - comparing against
        // the *immediately preceding* member (not the historical
        // maximum) means one out-of-place member is reported once, not
        // repeated against every correctly-ordered member that follows
        // it too.
        previous = Some((cat, label));
    }
}

fn out_of_order_diagnostic(file: &str, this_label: &str, prev_label: &str) -> Diagnostic {
    Diagnostic::error(
        "KY0401",
        "contract members are out of canonical order",
        file,
        placeholder_span(),
        format!("this {this_label} appears after a {prev_label}"),
    )
    .note("LANGUAGE_SPEC.md §3.2 requires: state, const, error, event, struct/enum, init, public fn, internal fn, unmarked fn")
    .help(format!("move this {this_label} above the {prev_label}, or move the {prev_label} below every {this_label}"))
}
