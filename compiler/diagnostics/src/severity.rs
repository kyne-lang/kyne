//! Diagnostic severity, per docs/COMPILER_ARCHITECTURE.md §15.1 and §11.2.

/// How serious a [`crate::Diagnostic`] is, and whether it blocks
/// compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// A `KY`-prefixed Compiler Error. Blocks compilation. Produced by the
    /// Parser, Type Checker, and Semantic Analyzer.
    Error,
    /// A `KS`-prefixed Security Analyzer finding. Does not block
    /// compilation, but SHOULD be surfaced prominently.
    Warning,
    /// A `KS`-prefixed Security Analyzer finding. Lower-confidence or
    /// purely stylistic; best surfaced in an IDE.
    Hint,
}

impl Severity {
    /// The lowercase tag this severity renders as (`error`, `warning`,
    /// `hint`), matching COMPILER_ARCHITECTURE.md §15.2's examples.
    pub fn tag(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Hint => "hint",
        }
    }

    /// Whether a diagnostic of this severity blocks compilation, per
    /// docs/COMPILER_ARCHITECTURE.md §15.1's `KY`/`KS` namespace split.
    pub fn blocks_compilation(self) -> bool {
        matches!(self, Severity::Error)
    }
}
