//! `kyne_diagnostics` - the shared diagnostic type, formatting, and code
//! namespace.
//!
//! Specified by docs/COMPILER_ARCHITECTURE.md §15 (Diagnostics Engine). A
//! foundation crate: every other compiler stage depends on it, and it
//! depends on nothing else in this workspace, per docs/ARCHITECTURE.md §4.
//!
//! ## Code namespace
//!
//! Per §15.1:
//!
//! - `KYxxxx` - Compiler Errors. Blocking. Produced by the Parser, Type
//!   Checker, and Semantic Analyzer. Grouped by concern: `KY00xx`
//!   (lexical), `KY01xx` (storage/state), `KY02xx` (types/arithmetic),
//!   `KY03xx` (events), `KY04xx` (contract structure/canonical ordering),
//!   `KY05xx` (authorization), `KY06xx` (visibility/modules), `KY07xx`
//!   (control flow/semantic).
//! - `KSxxxx` - Security Analyzer findings. Non-blocking (Warning or
//!   Hint). Produced only by the Security Analyzer.
//!
//! [`Diagnostic::error`], [`Diagnostic::warning`], and [`Diagnostic::hint`]
//! each assert their `code` matches the namespace their severity requires.

mod diagnostic;
mod severity;
mod span;

pub use diagnostic::Diagnostic;
pub use severity::Severity;
pub use span::Span;

#[cfg(test)]
mod tests {
    use super::*;

    // The three illustrative diagnostics reproduced from
    // docs/LANGUAGE_SPEC.md §14, confirming `render` matches the format
    // that document specifies as the expected quality bar. Title, source
    // text, note, and help text are verbatim; the `-->` column number is
    // recomputed to genuinely match the shown source line's indentation
    // (the doc's own illustrative "line:column" text does not always
    // correspond to the char offset its own trimmed snippet shows, since
    // these examples are hand-authored prose, not real compiler output).

    #[test]
    fn language_spec_example_state_read_before_init() {
        let source = "\n".repeat(13) + "    let x = balance + 1;\n";
        let diagnostic = Diagnostic::error(
            "KY0104",
            "state `balance` may be read before it is initialized",
            "src/token.kyn",
            Span::new(14, 13, 7),
            "possibly-uninitialized persistent state",
        )
        .note("`balance` has no default value and is not assigned in `init`")
        .help_with_snippet(
            "give `balance` a default value at its declaration:",
            ["state balance: i128 = 0;"],
        )
        .help_with_snippet(
            "or assign it unconditionally inside `init`:",
            ["public fn init() {", "    balance = 0;", "}"],
        );

        let expected = "\
error[KY0104]: state `balance` may be read before it is initialized
  --> src/token.kyn:14:13
   |
14 |     let x = balance + 1;
   |             ^^^^^^^ possibly-uninitialized persistent state
   |
   = note: `balance` has no default value and is not assigned in `init`
   = help: give `balance` a default value at its declaration:
   |
   |     state balance: i128 = 0;
   |
   = help: or assign it unconditionally inside `init`:
   |
   |     public fn init() {
   |         balance = 0;
   |     }";

        assert_eq!(diagnostic.render(&source), expected);
    }

    #[test]
    fn language_spec_example_overflowing_constant() {
        let source = "\n".repeat(8) + "    const MAX: u32 = 4_294_967_295 + 1;\n";
        let diagnostic = Diagnostic::error(
            "KY0210",
            "this operation always overflows `u32`",
            "src/token.kyn",
            Span::new(9, 22, 17),
            "`u32` cannot represent this value",
        )
        .note("Kyne arithmetic is checked by default and never wraps silently — see LANGUAGE_SPEC.md §7.2")
        .help("if you intended wraparound, call it explicitly: `MAX_U32.wrapping_add(1)`");

        let expected = "\
error[KY0210]: this operation always overflows `u32`
  --> src/token.kyn:9:22
   |
 9 |     const MAX: u32 = 4_294_967_295 + 1;
   |                      ^^^^^^^^^^^^^^^^^ `u32` cannot represent this value
   |
   = note: Kyne arithmetic is checked by default and never wraps silently — see LANGUAGE_SPEC.md §7.2
   = help: if you intended wraparound, call it explicitly: `MAX_U32.wrapping_add(1)`";

        assert_eq!(diagnostic.render(&source), expected);
    }

    #[test]
    fn language_spec_example_mismatched_emit_arguments() {
        let source = "\n".repeat(46) + "    emit Transfer(from, to);\n";
        let diagnostic = Diagnostic::error(
            "KY0311",
            "`emit Transfer` does not match its declared event",
            "src/token.kyn",
            Span::new(47, 10, 17),
            "expected 3 arguments, found 2",
        )
        .note("`event Transfer(from: address, to: address, amount: i128);` declared at src/token.kyn:6")
        .help("did you forget to pass `amount`?");

        let expected = "\
error[KY0311]: `emit Transfer` does not match its declared event
  --> src/token.kyn:47:10
   |
47 |     emit Transfer(from, to);
   |          ^^^^^^^^^^^^^^^^^ expected 3 arguments, found 2
   |
   = note: `event Transfer(from: address, to: address, amount: i128);` declared at src/token.kyn:6
   = help: did you forget to pass `amount`?";

        assert_eq!(diagnostic.render(&source), expected);
    }

    // Reproduced from docs/COMPILER_ARCHITECTURE.md §15.2.

    #[test]
    fn compiler_architecture_example_canonical_order() {
        let source = "\n".repeat(21) + "    public fn transfer(...) { ... }\n";
        let diagnostic = Diagnostic::error(
            "KY0401",
            "contract members are out of canonical order",
            "src/token.kyn",
            Span::new(22, 5, 23),
            "this `public fn` appears before an `event` declaration",
        )
        .note("LANGUAGE_SPEC.md §3.2 requires: state, const, error, event, struct/enum, init, public fn, internal fn, unmarked fn")
        .help("move `event Transfer(...)` above this function, or move this function below all `event` declarations");

        let expected = "\
error[KY0401]: contract members are out of canonical order
  --> src/token.kyn:22:5
   |
22 |     public fn transfer(...) { ... }
   |     ^^^^^^^^^^^^^^^^^^^^^^^ this `public fn` appears before an `event` declaration
   |
   = note: LANGUAGE_SPEC.md §3.2 requires: state, const, error, event, struct/enum, init, public fn, internal fn, unmarked fn
   = help: move `event Transfer(...)` above this function, or move this function below all `event` declarations";

        assert_eq!(diagnostic.render(&source), expected);
    }

    #[test]
    fn compiler_architecture_example_missing_auth_is_a_warning() {
        let source = "\n".repeat(30) + "public fn set_admin(new_admin: address) {\n";
        let diagnostic = Diagnostic::warning(
            "KS0101",
            "this function mutates `state` but calls no `auth(...)`",
            "src/token.kyn",
            Span::new(31, 1, 39),
            "mutates `state.admin` with no authorization check found in this function",
        )
        .note("this is a heuristic finding — some public mutating functions are intentionally unauthenticated")
        .help("if `new_admin` should require the current admin's approval, add: auth(admin);");

        let expected = "\
warning[KS0101]: this function mutates `state` but calls no `auth(...)`
  --> src/token.kyn:31:1
   |
31 | public fn set_admin(new_admin: address) {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutates `state.admin` with no authorization check found in this function
   |
   = note: this is a heuristic finding — some public mutating functions are intentionally unauthenticated
   = help: if `new_admin` should require the current admin's approval, add: auth(admin);";

        assert_eq!(diagnostic.render(&source), expected);
    }

    #[test]
    fn ky_code_required_for_error_severity() {
        let result = std::panic::catch_unwind(|| {
            Diagnostic::error(
                "KS9999",
                "wrong namespace",
                "f.kyn",
                Span::new(1, 1, 1),
                "x",
            )
        });
        assert!(result.is_err());
    }

    #[test]
    fn ks_code_required_for_warning_and_hint_severity() {
        let warning = std::panic::catch_unwind(|| {
            Diagnostic::warning(
                "KY9999",
                "wrong namespace",
                "f.kyn",
                Span::new(1, 1, 1),
                "x",
            )
        });
        assert!(warning.is_err());

        let hint = std::panic::catch_unwind(|| {
            Diagnostic::hint(
                "KY9999",
                "wrong namespace",
                "f.kyn",
                Span::new(1, 1, 1),
                "x",
            )
        });
        assert!(hint.is_err());
    }

    #[test]
    fn severity_blocks_compilation_matches_namespace() {
        assert!(Severity::Error.blocks_compilation());
        assert!(!Severity::Warning.blocks_compilation());
        assert!(!Severity::Hint.blocks_compilation());
    }
}
