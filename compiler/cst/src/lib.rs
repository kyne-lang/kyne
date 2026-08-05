//! `kyne_cst` - the stable Concrete Syntax Tree type.
//!
//! Specified by docs/COMPILER_ARCHITECTURE.md §6. Wraps `kyne_parser`'s
//! raw, lossless tree together with the source text it was built from,
//! so consumers (the formatter, the Language Server, `kyne_ast`'s
//! CST→AST lowering) work against one stable type instead of threading a
//! `(SyntaxNode, &str)` pair around themselves.
//!
//! Per docs/ARCHITECTURE.md §4's dependency table, the CST→AST lowering
//! function itself lives in `kyne_ast` (which depends on this crate),
//! not here - `kyne_ast` owns its own node types, and lowering has to
//! construct them, so it cannot be implemented in a crate `kyne_ast`
//! depends on.

pub use kyne_parser::{NodeKind, SyntaxElement, SyntaxNode, TokenKind};

use kyne_diagnostics::Diagnostic;

/// The Concrete Syntax Tree for one source file: a lossless tree over
/// every token `kyne_lexer` produced for it, including whitespace and
/// comments, paired with the exact source text it was parsed from.
#[derive(Debug, Clone)]
pub struct Cst {
    root: SyntaxNode,
    source: String,
}

impl Cst {
    /// Parses `source` (from `file`, used only for diagnostic messages)
    /// into a `Cst` plus every syntax error found. Never aborts on
    /// malformed input - per docs/COMPILER_ARCHITECTURE.md §5, parsing
    /// always runs to completion and returns a full tree.
    pub fn parse(source: impl Into<String>, file: impl Into<String>) -> (Cst, Vec<Diagnostic>) {
        let source = source.into();
        let (root, diagnostics) = kyne_parser::parse(&source, file);
        (Cst { root, source }, diagnostics)
    }

    /// The root `Program` node.
    pub fn root(&self) -> &SyntaxNode {
        &self.root
    }

    /// The exact source text this tree was parsed from.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Reconstructs the exact original source text by reprinting every
    /// token reachable from the root, in order - the CST's losslessness
    /// guarantee, per docs/COMPILER_ARCHITECTURE.md §6.
    pub fn text(&self) -> String {
        self.root.text(&self.source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_source_and_reconstructs_it_losslessly() {
        let source = "contract C {\n    state n: i64 = 0;\n}\n";
        let (cst, diagnostics) = Cst::parse(source, "test.kyn");
        assert!(diagnostics.is_empty());
        assert_eq!(cst.source(), source);
        assert_eq!(cst.text(), source);
        assert_eq!(cst.root().kind, NodeKind::Program);
    }

    #[test]
    fn preserves_comments_and_whitespace_in_the_reconstruction() {
        let source = "// a leading comment\ncontract C {}\n\n/* trailing */\n";
        let (cst, diagnostics) = Cst::parse(source, "test.kyn");
        assert!(diagnostics.is_empty());
        assert_eq!(cst.text(), source);
    }

    #[test]
    fn malformed_input_still_yields_a_lossless_tree() {
        let source = "contract C { fn f() { let x = ; } }\n";
        let (cst, diagnostics) = Cst::parse(source, "test.kyn");
        assert!(!diagnostics.is_empty());
        assert_eq!(cst.text(), source);
    }
}
