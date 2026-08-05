//! `kyne_formatter` - the implementation behind `kyne fmt`.
//!
//! Specified by docs/LANGUAGE_SPEC.md §13 (canonical rules) and
//! docs/TOOLCHAIN.md §10 (no configuration, idempotence). Depends only
//! on `kyne_lexer`, `kyne_parser`, and `kyne_cst` - deliberately not
//! `kyne_types` or any later stage, since formatting a syntactically
//! valid file never requires it to also type-check.

mod printer;

use kyne_cst::Cst;
use kyne_diagnostics::Diagnostic;

/// Formats `source` into Kyne's one canonical rendering, per
/// `LANGUAGE_SPEC.md` §13. Returns the parser's diagnostics, unformatted,
/// if `source` does not parse - formatting is defined only for
/// syntactically valid input.
pub fn format(source: &str) -> Result<String, Vec<Diagnostic>> {
    let (cst, diagnostics) = Cst::parse(source, "<source>");
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    Ok(printer::print(&cst))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_idempotent(source: &str) {
        let once = format(source).expect("expected a clean parse");
        let twice = format(&once).expect("formatted output must itself be valid Kyne");
        assert_eq!(once, twice, "formatting is not idempotent");
    }

    #[test]
    fn simple_contract_is_idempotent_and_matches_canonical_style() {
        let source = "\
contract Counter {
    state count: i64 = 0;

    event Incremented(by: address, new_value: i64);

    public fn init() {
        count = 0;
    }

    public fn increment(by: address) -> i64 {
        auth(by);

        count += 1;
        emit Incremented(by, count);
        return count;
    }

    public fn get() -> i64 {
        return count;
    }
}
";
        let formatted = format(source).unwrap();
        assert_eq!(formatted, source);
        assert_idempotent(source);
    }

    #[test]
    fn error_returns_parser_diagnostics() {
        let result = format("contract C { fn f() { let x = ; } }\n");
        assert!(result.is_err());
    }

    #[test]
    fn blank_line_forced_between_different_categories_removed_within_same() {
        let source = "contract C {\n    state a: i64 = 0;\n\n\n\n    state b: i64 = 0;\n    const M: i64 = 1;\n}\n";
        let formatted = format(source).unwrap();
        // Between the two `state` members (same category): collapsed to
        // no blank line, since the source had none-forced-but-collapsed
        // multiple blanks reduced to the source's own choice (blank
        // present -> kept, but capped at one).
        assert!(formatted.contains("state a: i64 = 0;\n\n    state b"));
        // Between `state` and `const` (different categories): forced.
        assert!(formatted.contains("state b: i64 = 0;\n\n    const M"));
    }

    #[test]
    fn long_param_list_wraps_with_trailing_comma() {
        let source = "contract C { public fn init(buyer_addr: address, seller_addr: address, arbiter_addr: address, deposit: i128) {} }\n";
        let formatted = format(source).unwrap();
        assert!(formatted.contains("public fn init(\n        buyer_addr: address,\n"));
        assert!(formatted.contains("        deposit: i128,\n    ) {"));
        assert_idempotent(source);
    }

    #[test]
    fn imports_are_sorted_alphabetically() {
        let source = "use z.mod;\nuse a.mod;\n";
        let formatted = format(source).unwrap();
        let a_pos = formatted.find("use a.mod").unwrap();
        let z_pos = formatted.find("use z.mod").unwrap();
        assert!(a_pos < z_pos);
    }

    #[test]
    fn struct_and_enum_and_error_declarations_are_always_multiline() {
        // Per docs/adr/ADR-0006-formatter-implementation.md: unlike a
        // call's argument list, these always expand one member per line
        // even when the flat form would easily fit under 100 columns.
        let source = "error E { A, B }\nstruct S { x: i64 }\nenum En { A, B }\n";
        let formatted = format(source).unwrap();
        assert!(formatted.contains("error E {\n    A,\n    B,\n}"));
        assert!(formatted.contains("struct S {\n    x: i64,\n}"));
        assert!(formatted.contains("enum En {\n    A,\n    B,\n}"));
    }

    #[test]
    fn overlong_struct_literal_argument_wraps_and_hugs_the_closing_paren() {
        let source = "\
contract C {
    fn f(m: map<u64, Transaction>, id: u64, tx: Transaction) {
        m.set(id, Transaction { to: tx.to, amount: tx.amount, approvals: tx.approvals, executed: true });
    }
}
";
        let formatted = format(source).unwrap();
        assert!(formatted.contains("m.set(id, Transaction {\n"));
        assert!(formatted.contains("            to: tx.to,\n"));
        assert!(formatted.contains("        });\n"));
        assert_idempotent(source);
    }

    #[test]
    fn overlong_if_expression_wraps_each_branch() {
        let source = "\
contract C {
    fn f(approve: bool, p: Proposal) {
        let updated = if approve { Proposal { description: p.description, yes_votes: p.yes_votes + 1, no_votes: p.no_votes } } else { Proposal { description: p.description, yes_votes: p.yes_votes, no_votes: p.no_votes + 1 } };
    }
}
";
        let formatted = format(source).unwrap();
        assert!(formatted.contains("let updated = if approve {\n"));
        assert!(formatted.contains("} else {\n"));
        assert!(formatted.lines().all(|l| l.chars().count() <= 100));
        assert_idempotent(source);
    }
}
