//! `kyne_semantics` - the Semantic Analyzer.
//!
//! Specified by docs/COMPILER_ARCHITECTURE.md §10. Enforces every
//! MUST-level static rule in docs/LANGUAGE_SPEC.md not already covered
//! by parsing, resolution, or typing.
//!
//! ## Scope (v0.2 gate minimum, per issue #14)
//!
//! Only four rules are implemented: canonical contract member ordering,
//! definite assignment of `state` fields, exhaustive `match` checking,
//! and `emit` argument validation. The full rule list in
//! `COMPILER_ARCHITECTURE.md` §10 (control-flow validation beyond
//! exhaustiveness, full contract/visibility validation, `auth`
//! placement and argument-type validation, and RUNTIME_MODEL.md's
//! undefined-behavior checkpoint) is explicitly deferred to a Phase 2
//! follow-up issue.
//!
//! ## Known limitations
//!
//! - `kyne_ast` carries no source-span information, so every diagnostic
//!   here uses a placeholder span - the same limitation
//!   `kyne_resolver`/`kyne_types` documented in
//!   docs/adr/ADR-0007-resolver-implementation.md.
//! - `emit` argument type-checking (`src/events.rs`) uses a small,
//!   local expression-type inference rather than `kyne_types`'s own
//!   (not exposed for reuse), covering only the directly-observable
//!   shapes real `emit` arguments take (names, literals, field access).

mod definite_assignment;
mod events;
mod exhaustiveness;
mod order;

use kyne_ast::{Program, TopLevelDecl};
use kyne_diagnostics::{Diagnostic, Span};
use kyne_types::TypeEnv;

pub struct AnalysisResult {
    pub diagnostics: Vec<Diagnostic>,
}

pub fn analyze(program: &Program, file: &str) -> AnalysisResult {
    let mut diagnostics = Vec::new();
    let env = TypeEnv::build(program, &mut |_| {});

    for item in &program.items {
        if let TopLevelDecl::Contract(c) = item {
            order::check(c, file, &mut diagnostics);
            definite_assignment::check(c, file, &mut diagnostics);
        }
    }
    exhaustiveness::check(program, &env, file, &mut diagnostics);
    events::check(program, &env, file, &mut diagnostics);

    AnalysisResult { diagnostics }
}

pub(crate) fn placeholder_span() -> Span {
    Span::new(1, 1, 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kyne_cst::Cst;

    fn analyze_source(source: &str) -> AnalysisResult {
        let (cst, diagnostics) = Cst::parse(source, "test.kyn");
        assert!(
            diagnostics.is_empty(),
            "expected a clean parse, got: {diagnostics:?}"
        );
        let program = kyne_ast::lower(&cst);
        analyze(&program, "test.kyn")
    }

    fn codes(result: &AnalysisResult) -> Vec<&str> {
        result.diagnostics.iter().map(|d| d.code()).collect()
    }

    fn assert_clean(source: &str) {
        let result = analyze_source(source);
        assert!(
            result.diagnostics.is_empty(),
            "expected no diagnostics, got: {:?}",
            result.diagnostics
        );
    }

    // ---- Canonical member order ----

    #[test]
    fn correct_canonical_order_is_clean() {
        assert_clean(
            "contract C {
                state a: i64 = 0;
                const M: i64 = 1;
                error E { X }
                event Ev(x: i64);
                struct S { x: i64 }
                public fn init() {}
                public fn a() {}
                internal fn b() {}
                fn c() {}
            }\n",
        );
    }

    #[test]
    fn const_before_state_is_out_of_order() {
        let result = analyze_source("contract C { const M: i64 = 1; state a: i64 = 0; }\n");
        assert_eq!(codes(&result), vec!["KY0401"]);
    }

    #[test]
    fn fn_before_event_is_out_of_order() {
        let result = analyze_source("contract C { public fn f() {} event Ev(x: i64); }\n");
        assert_eq!(codes(&result), vec!["KY0401"]);
    }

    #[test]
    fn one_violation_does_not_cascade_into_flagging_correct_members() {
        // `const` after `state` is correctly ordered; only the leading
        // `error` (before `state`) is actually out of place.
        let result =
            analyze_source("contract C { error E { X } state a: i64 = 0; const M: i64 = 1; }\n");
        assert_eq!(codes(&result), vec!["KY0401"]);
    }

    // ---- Definite assignment of `state` ----

    #[test]
    fn literal_initializer_satisfies_definite_assignment() {
        assert_clean("contract C { state n: i64 = 0; }\n");
    }

    #[test]
    fn unconditional_assignment_in_init_satisfies_definite_assignment() {
        assert_clean("contract C { state n: i64; public fn init() { n = 0; } }\n");
    }

    #[test]
    fn no_initializer_and_no_init_fn_is_an_error() {
        let result = analyze_source("contract C { state n: i64; }\n");
        assert_eq!(codes(&result), vec!["KY0104"]);
    }

    #[test]
    fn conditional_assignment_in_only_one_if_branch_is_an_error() {
        let result = analyze_source(
            "contract C { state n: i64; public fn init(cond: bool) { if cond { n = 1; } } }\n",
        );
        assert_eq!(codes(&result), vec!["KY0104"]);
    }

    #[test]
    fn assignment_in_both_if_and_else_branches_satisfies_definite_assignment() {
        assert_clean("contract C { state n: i64; public fn init(cond: bool) { if cond { n = 1; } else { n = 2; } } }\n");
    }

    #[test]
    fn assignment_in_one_branch_with_the_other_diverging_satisfies_definite_assignment() {
        assert_clean(
            "error E { X }
             contract C {
                state n: i64;
                public fn init(cond: bool) -> Result<bool, E> {
                    if cond {
                        throw E::X;
                    } else {
                        n = 1;
                    }
                    return Ok(true);
                }
            }\n",
        );
    }

    #[test]
    fn map_and_list_state_fields_are_exempt() {
        assert_clean(
            "contract C {
                state balances: map<address, i128>;
                state ids: list<u64>;
                public fn init() {}
            }\n",
        );
    }

    // ---- Exhaustive match ----

    #[test]
    fn match_with_wildcard_is_exhaustive() {
        assert_clean(
            "enum Status { Pending, Approved }
             contract C { fn f(s: Status) { match s { Status::Pending => {} _ => {} } } }\n",
        );
    }

    #[test]
    fn match_covering_every_variant_is_exhaustive() {
        assert_clean(
            "enum Status { Pending, Approved }
             contract C { fn f(s: Status) { match s { Status::Pending => {} Status::Approved => {} } } }\n",
        );
    }

    #[test]
    fn match_missing_a_variant_is_an_error() {
        let result = analyze_source(
            "enum Status { Pending, Approved }
             contract C { fn f(s: Status) { match s { Status::Pending => {} } } }\n",
        );
        assert_eq!(codes(&result), vec!["KY0701"]);
    }

    #[test]
    fn guarded_arm_does_not_count_toward_exhaustiveness() {
        let result = analyze_source(
            "enum Status { Pending, Approved }
             contract C {
                fn f(s: Status, cond: bool) {
                    match s {
                        Status::Pending if cond => {}
                        Status::Approved => {}
                    }
                }
            }\n",
        );
        assert_eq!(codes(&result), vec!["KY0701"]);
    }

    #[test]
    fn option_match_requires_both_some_and_none() {
        let result =
            analyze_source("contract C { fn f(x: Option<i64>) { match x { Some(v) => {} } } }\n");
        assert_eq!(codes(&result), vec!["KY0701"]);
    }

    #[test]
    fn option_match_with_both_arms_is_exhaustive() {
        assert_clean(
            "contract C { fn f(x: Option<i64>) { match x { Some(v) => {} None => {} } } }\n",
        );
    }

    // ---- `emit` argument validation ----

    #[test]
    fn emit_with_matching_arguments_is_clean() {
        assert_clean(
            "contract C {
                event Transfer(from: address, to: address, amount: i128);
                fn f(from: address, to: address, amount: i128) {
                    emit Transfer(from, to, amount);
                }
            }\n",
        );
    }

    #[test]
    fn emit_with_too_few_arguments_is_an_error() {
        let result = analyze_source(
            "contract C {
                event Transfer(from: address, to: address);
                fn f(from: address) { emit Transfer(from); }
            }\n",
        );
        assert_eq!(codes(&result), vec!["KY0301"]);
    }

    #[test]
    fn emit_with_wrong_argument_type_is_an_error() {
        let result = analyze_source(
            "contract C {
                event Transfer(amount: i128);
                fn f(ok: bool) { emit Transfer(ok); }
            }\n",
        );
        assert_eq!(codes(&result), vec!["KY0301"]);
    }
}
