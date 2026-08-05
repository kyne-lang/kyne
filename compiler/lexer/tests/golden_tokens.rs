//! Golden token-stream tests, per
//! [`COMPILER_ARCHITECTURE.md` §19](../../../docs/COMPILER_ARCHITECTURE.md#19-testing-strategy):
//! "given a source text fixture, assert the exact resulting token
//! sequence, including span information. Include fixtures specifically
//! exercising error recovery." This is `kyne_golden`'s (issue #20)
//! second real adopter, alongside `kyne_codegen`'s Rust-text snapshots -
//! demonstrating the same shared helper covering a completely different
//! kind of golden output (a token stream, not generated source text).
//!
//! Deliberately small, hand-written fixtures rather than the full six
//! canonical examples: a token-stream dump of an entire canonical
//! contract would be hundreds of lines and hard for a reviewer to
//! actually read in a diff, defeating the point of a golden file being
//! reviewable. `tests/canonical_examples.rs` already covers the full
//! six examples at the coarser "zero Error tokens" granularity this
//! finer-grained format doesn't need to duplicate.

use std::fmt::Write as _;

use kyne_lexer::lex;

/// Renders a token stream as one line per token: kind, exact source
/// text, and 1-based line:column - everything
/// [`COMPILER_ARCHITECTURE.md` §19`]'s "including span information"
/// asks for, in a format a human reviewer can actually read in a diff.
fn render_tokens(source: &str) -> String {
    let tokens = lex(source);
    let mut out = String::new();
    for t in &tokens {
        writeln!(
            out,
            "{:?} {:?} @ {}:{}",
            t.kind,
            t.text(source),
            t.span.line,
            t.span.column
        )
        .expect("writing to a String never fails");
    }
    out
}

#[test]
fn basic_fixture_matches_golden_token_stream() {
    let source = include_str!("fixtures/basic.kyn");
    let rendered = render_tokens(source);
    kyne_golden::assert_golden("tests/snapshots/basic.tokens.snap", &rendered);
}

/// `$` is not a valid token anywhere in `LANGUAGE_SPEC.md`'s grammar -
/// this fixture confirms it becomes a single `Error` token and lexing
/// resumes cleanly afterward, rather than the whole rest of the file
/// being swallowed or the lexer stopping early.
#[test]
fn error_recovery_fixture_matches_golden_token_stream() {
    let source = include_str!("fixtures/error_recovery.kyn");
    let rendered = render_tokens(source);
    kyne_golden::assert_golden("tests/snapshots/error_recovery.tokens.snap", &rendered);
}
