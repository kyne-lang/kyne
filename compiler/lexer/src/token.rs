//! Token and span types produced by the lexer, per
//! docs/COMPILER_ARCHITECTURE.md §4.

/// A byte-offset span into the original source text, with 1-based line
/// and column information for diagnostic reporting.
///
/// `column` counts characters, not bytes, so it remains meaningful for
/// source containing multi-byte UTF-8 sequences in comments or string
/// literals, per docs/LANGUAGE_SPEC.md §1.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Byte offset of the first byte of this span.
    pub start: usize,
    /// Byte offset one past the last byte of this span.
    pub end: usize,
    /// 1-based line number of the first character of this span.
    pub line: u32,
    /// 1-based column number of the first character of this span.
    pub column: u32,
}

impl Span {
    /// The length of this span, in bytes.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Whether this span covers zero bytes.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// The kind of a lexical token, per docs/LANGUAGE_SPEC.md §1 and §2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // -- Primary keywords — docs/LANGUAGE_SPEC.md §1.3. --
    Contract,
    Fn,
    State,
    Let,
    Const,
    Event,
    Emit,
    Use,
    Public,
    Internal,
    Struct,
    Enum,
    Trait,
    Match,
    If,
    Else,
    For,
    While,
    Return,
    Break,
    Continue,
    /// The `error` declaration keyword, per docs/LANGUAGE_SPEC.md §6.5 -
    /// not to be confused with [`TokenKind::Error`], the lex-failure
    /// token kind.
    ErrorKw,
    Throw,
    True,
    False,
    Auth,
    In,

    // -- Reserved for future language versions — docs/LANGUAGE_SPEC.md §1.3.1. --
    //
    // Lexically distinct from `Identifier` now, rather than deferred to a
    // later stage, so that a future version granting one of these real
    // meaning is not a breaking lexer change, and so that "used a
    // reserved word as an identifier" is caught as an ordinary
    // unexpected-token parse error, not deferred to semantic analysis.
    KwSelfValue, // self
    KwSelfType,  // Self
    KwImpl,
    KwMut,
    KwAsync,
    KwAwait,
    KwUnsafe,
    KwDyn,
    KwAs,
    KwMove,

    // -- Literals and identifiers. --
    Identifier,
    IntLiteral,
    StringLiteral,

    // -- Punctuation and operators — docs/LANGUAGE_SPEC.md §2, §7.1. --
    LBrace,     // {
    RBrace,     // }
    LParen,     // (
    RParen,     // )
    LBracket,   // [
    RBracket,   // ]
    LAngle,     // < (also comparison `<`; disambiguated by the parser)
    RAngle,     // > (also comparison `>`; disambiguated by the parser)
    Semicolon,  // ;
    Comma,      // ,
    Dot,        // .
    Colon,      // :
    ColonColon, // :: (enum/error variant access, per §7.5)
    Eq,         // =
    PlusEq,     // +=
    MinusEq,    // -=
    StarEq,     // *=
    SlashEq,    // /=
    PercentEq,  // %=
    Arrow,      // ->
    FatArrow,   // =>
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Percent,    // %
    EqEq,       // ==
    NotEq,      // !=
    LtEq,       // <=
    GtEq,       // >=
    AmpAmp,     // &&
    PipePipe,   // ||
    Bang,       // !
    Question,   // ?

    // -- Trivia — retained so kyne_cst can reconstruct source exactly,
    //    per docs/COMPILER_ARCHITECTURE.md §6. --
    Whitespace,
    LineComment,
    BlockComment,
    DocComment,

    /// A byte sequence that could not begin, or complete, any valid
    /// token. The lexer recovers by emitting this and resuming
    /// immediately afterward, per docs/COMPILER_ARCHITECTURE.md §4.
    Error,

    /// End of input. Always the final token `tokenize` produces.
    Eof,
}

/// A single lexical token: its kind and its location in the source.
///
/// A `Token` does not own its text — call [`Token::text`] with the
/// original source to slice it out, avoiding an allocation per token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    /// The exact source text this token covers.
    ///
    /// `source` MUST be the same string `lex` was called with — this
    /// method does not validate that the span is in bounds for a
    /// different string.
    pub fn text<'src>(&self, source: &'src str) -> &'src str {
        &source[self.span.start..self.span.end]
    }
}
