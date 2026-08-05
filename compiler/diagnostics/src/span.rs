//! The source-location type diagnostics point at.

/// A single-line source location a diagnostic renders a caret underline
/// under, per docs/COMPILER_ARCHITECTURE.md §15.2's examples.
///
/// Deliberately independent of `kyne_lexer::Span`: this crate has no
/// dependencies (per docs/ARCHITECTURE.md §4), so every producer converts
/// its own span representation into this one at the point it constructs a
/// [`crate::Diagnostic`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// 1-based line number.
    pub line: u32,
    /// 1-based, character-counted column of the first underlined character.
    pub column: u32,
    /// Number of characters the caret underline covers. Always at least 1.
    pub length: usize,
}

impl Span {
    pub fn new(line: u32, column: u32, length: usize) -> Self {
        Span {
            line,
            column,
            length: length.max(1),
        }
    }
}
