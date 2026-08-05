//! `kyne_lexer` - tokenization for Kyne source.
//!
//! Specified by docs/COMPILER_ARCHITECTURE.md §4 (Lexical Analysis).
//! Converts `.kyn` source text into a token stream with source-span
//! information, per docs/LANGUAGE_SPEC.md §1.
//!
//! This crate has no dependencies on any other crate in this workspace,
//! per docs/ARCHITECTURE.md §4 - it is the first stage of the pipeline.

mod keyword;
mod lexer;
mod token;

pub use lexer::Lexer;
pub use token::{Span, Token, TokenKind};

/// Tokenizes `source` into a flat sequence of tokens, per
/// docs/COMPILER_ARCHITECTURE.md §4. The final token is always
/// [`TokenKind::Eof`].
///
/// Never panics on malformed input: an invalid or incomplete lexeme is
/// represented as a [`TokenKind::Error`] token, and lexing resumes
/// immediately afterward, per §4's error-recovery requirement - the
/// entire lexing pass always completes and returns a full token stream,
/// no matter how malformed the input is.
pub fn lex(source: &str) -> Vec<Token> {
    Lexer::new(source).tokenize()
}
