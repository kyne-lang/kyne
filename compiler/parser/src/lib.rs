//! `kyne_parser` - grammar enforcement and raw syntax tree construction.
//!
//! Specified by docs/COMPILER_ARCHITECTURE.md §5 (Parsing). Consumes the
//! token stream produced by `kyne_lexer` and enforces the grammar defined
//! in docs/LANGUAGE_SPEC.md §2, producing the lossless raw tree
//! `kyne_cst` wraps as the compiler's stable CST type.

mod parser;
mod tree;

pub use kyne_lexer::TokenKind;
pub use parser::Parser;
pub use tree::{NodeKind, SyntaxElement, SyntaxNode};

use kyne_diagnostics::Diagnostic;

/// Parses `source` (from `file`, used only for diagnostic messages) into
/// the raw syntax tree plus every syntax error found. Never aborts on
/// malformed input - per §5's recovery strategy, parsing always runs to
/// completion and returns a full tree.
pub fn parse(source: &str, file: impl Into<String>) -> (SyntaxNode, Vec<Diagnostic>) {
    Parser::new(source, file).parse()
}
