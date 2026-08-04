//! The `Lexer`: converts `.kyn` source text into a flat token stream, per
//! docs/COMPILER_ARCHITECTURE.md §4 and docs/LANGUAGE_SPEC.md §1.

use crate::keyword;
use crate::token::{Span, Token, TokenKind};

/// Scans a source string into tokens.
///
/// Construct with [`Lexer::new`] and consume with [`Lexer::tokenize`], or
/// use the crate-level [`crate::lex`] convenience function.
pub struct Lexer<'src> {
    source: &'src str,
    /// Byte offset of the next unconsumed character.
    pos: usize,
    /// 1-based line number of `pos`.
    line: u32,
    /// 1-based column number (in characters) of `pos`.
    column: u32,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Lexer {
            source,
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    /// Scans the entire source and returns its complete token stream.
    /// The final token is always [`TokenKind::Eof`].
    ///
    /// Never panics on malformed input, per
    /// docs/COMPILER_ARCHITECTURE.md §4's error-recovery requirement: an
    /// invalid byte sequence becomes an [`TokenKind::Error`] token, and
    /// scanning resumes immediately afterward.
    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let done = token.kind == TokenKind::Eof;
            tokens.push(token);
            if done {
                return tokens;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn peek_char_at(&self, n: usize) -> Option<char> {
        self.source[self.pos..].chars().nth(n)
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek_char()?;
        self.pos += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(c)
    }

    fn make_token(&self, kind: TokenKind, start: usize, line: u32, column: u32) -> Token {
        Token {
            kind,
            span: Span {
                start,
                end: self.pos,
                line,
                column,
            },
        }
    }

    fn next_token(&mut self) -> Token {
        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        let Some(c) = self.peek_char() else {
            return self.make_token(TokenKind::Eof, start_pos, start_line, start_col);
        };

        let kind = if c.is_whitespace() {
            self.scan_whitespace()
        } else if c == '/' && self.peek_char_at(1) == Some('/') {
            self.scan_line_comment()
        } else if c == '/' && self.peek_char_at(1) == Some('*') {
            self.scan_block_comment()
        } else if is_ident_start(c) {
            self.scan_identifier_or_keyword()
        } else if c.is_ascii_digit() {
            self.scan_number()
        } else if c == '"' {
            self.scan_string()
        } else {
            self.scan_operator_or_error()
        };

        self.make_token(kind, start_pos, start_line, start_col)
    }

    fn scan_whitespace(&mut self) -> TokenKind {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.bump();
            } else {
                break;
            }
        }
        TokenKind::Whitespace
    }

    fn scan_line_comment(&mut self) -> TokenKind {
        self.bump(); // first '/'
        self.bump(); // second '/'

        let mut is_doc = false;
        if self.peek_char() == Some('/') {
            // Three slashes so far. `///x` (or `///` at EOF/newline) is a
            // doc comment; a fourth slash (`////`) is treated as an
            // ordinary comment instead, matching the common convention
            // that `////`-style dividers are not documentation.
            // LANGUAGE_SPEC.md §1.4 does not spell out this specific
            // edge case; this is the lexer's documented interpretation.
            if self.peek_char_at(1) != Some('/') {
                is_doc = true;
            }
            self.bump(); // consume the third '/' either way
        }

        while let Some(c) = self.peek_char() {
            if c == '\n' {
                break;
            }
            self.bump();
        }

        if is_doc {
            TokenKind::DocComment
        } else {
            TokenKind::LineComment
        }
    }

    fn scan_block_comment(&mut self) -> TokenKind {
        self.bump(); // '/'
        self.bump(); // '*'
        let mut depth: u32 = 1;
        loop {
            match (self.peek_char(), self.peek_char_at(1)) {
                (Some('/'), Some('*')) => {
                    self.bump();
                    self.bump();
                    depth += 1;
                }
                (Some('*'), Some('/')) => {
                    self.bump();
                    self.bump();
                    depth -= 1;
                    if depth == 0 {
                        return TokenKind::BlockComment;
                    }
                }
                (Some(_), _) => {
                    self.bump();
                }
                (None, _) => {
                    // Unterminated block comment: recover by returning an
                    // Error token covering everything consumed so far.
                    return TokenKind::Error;
                }
            }
        }
    }

    fn scan_identifier_or_keyword(&mut self) -> TokenKind {
        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if is_ident_continue(c) {
                self.bump();
            } else {
                break;
            }
        }
        let text = &self.source[start..self.pos];
        keyword::lookup(text).unwrap_or(TokenKind::Identifier)
    }

    fn scan_number(&mut self) -> TokenKind {
        // Hex literal: "0x" hex_digit { hex_digit | "_" }, per
        // docs/LANGUAGE_SPEC.md §1.6.
        if self.peek_char() == Some('0') && self.peek_char_at(1) == Some('x') {
            self.bump(); // '0'
            self.bump(); // 'x'
            let digits_start = self.pos;
            while let Some(c) = self.peek_char() {
                if c.is_ascii_hexdigit() || c == '_' {
                    self.bump();
                } else {
                    break;
                }
            }
            if self.pos == digits_start {
                // "0x" with no following hex digit is malformed. Consume
                // any trailing identifier-like characters too, so the
                // whole malformed lexeme (e.g. "0xZZ") becomes one Error
                // token instead of splitting into an Error for "0x"
                // followed by a separate, confusing Identifier token for
                // whatever comes after it.
                while let Some(c) = self.peek_char() {
                    if is_ident_continue(c) {
                        self.bump();
                    } else {
                        break;
                    }
                }
                return TokenKind::Error;
            }
            return TokenKind::IntLiteral;
        }

        // Decimal literal: digit { digit | "_" }.
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() || c == '_' {
                self.bump();
            } else {
                break;
            }
        }
        TokenKind::IntLiteral
    }

    fn scan_string(&mut self) -> TokenKind {
        self.bump(); // opening '"'
        let mut ok = true;
        loop {
            match self.peek_char() {
                None => {
                    // Unterminated string literal at end of input.
                    return TokenKind::Error;
                }
                Some('"') => {
                    self.bump();
                    return if ok {
                        TokenKind::StringLiteral
                    } else {
                        TokenKind::Error
                    };
                }
                Some('\\') => {
                    self.bump();
                    if !self.scan_escape() {
                        ok = false;
                    }
                }
                Some(_) => {
                    self.bump();
                }
            }
        }
    }

    /// Scans one escape sequence's payload, after the backslash has
    /// already been consumed. Returns `false`, without aborting the
    /// surrounding scan, if the escape was invalid — the caller keeps
    /// scanning to the string's closing quote so a malformed string
    /// literal still becomes one `Error` token rather than fragmenting
    /// into several confusing follow-on tokens.
    fn scan_escape(&mut self) -> bool {
        match self.peek_char() {
            Some('n') | Some('t') | Some('\\') | Some('"') => {
                self.bump();
                true
            }
            Some('u') => {
                self.bump(); // 'u'
                if self.peek_char() != Some('{') {
                    return false;
                }
                self.bump(); // '{'
                let digits_start = self.pos;
                while let Some(c) = self.peek_char() {
                    if c.is_ascii_hexdigit() {
                        self.bump();
                    } else {
                        break;
                    }
                }
                let has_digits = self.pos > digits_start;
                let hex = &self.source[digits_start..self.pos];
                let closed = self.peek_char() == Some('}');
                if closed {
                    self.bump();
                }
                if !has_digits || !closed {
                    return false;
                }
                u32::from_str_radix(hex, 16)
                    .ok()
                    .and_then(char::from_u32)
                    .is_some()
            }
            Some(_) => {
                // Unrecognized escape character - consume it so scanning
                // can continue toward the closing quote.
                self.bump();
                false
            }
            None => false,
        }
    }

    fn scan_operator_or_error(&mut self) -> TokenKind {
        let c = self
            .bump()
            .expect("caller already confirmed a character is present");
        match c {
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            ';' => TokenKind::Semicolon,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            '?' => TokenKind::Question,
            ':' => self.two_char(':', TokenKind::ColonColon, TokenKind::Colon),
            '+' => self.two_char('=', TokenKind::PlusEq, TokenKind::Plus),
            '*' => self.two_char('=', TokenKind::StarEq, TokenKind::Star),
            '/' => self.two_char('=', TokenKind::SlashEq, TokenKind::Slash),
            '%' => self.two_char('=', TokenKind::PercentEq, TokenKind::Percent),
            '!' => self.two_char('=', TokenKind::NotEq, TokenKind::Bang),
            '<' => self.two_char('=', TokenKind::LtEq, TokenKind::LAngle),
            '>' => self.two_char('=', TokenKind::GtEq, TokenKind::RAngle),
            '&' => self.two_char('&', TokenKind::AmpAmp, TokenKind::Error),
            '|' => self.two_char('|', TokenKind::PipePipe, TokenKind::Error),
            '=' => {
                if self.peek_char() == Some('=') {
                    self.bump();
                    TokenKind::EqEq
                } else if self.peek_char() == Some('>') {
                    self.bump();
                    TokenKind::FatArrow
                } else {
                    TokenKind::Eq
                }
            }
            '-' => {
                if self.peek_char() == Some('=') {
                    self.bump();
                    TokenKind::MinusEq
                } else if self.peek_char() == Some('>') {
                    self.bump();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            _ => TokenKind::Error,
        }
    }

    /// Consumes `expected` if it follows immediately, producing `then`;
    /// otherwise produces `otherwise` without consuming anything further.
    fn two_char(&mut self, expected: char, then: TokenKind, otherwise: TokenKind) -> TokenKind {
        if self.peek_char() == Some(expected) {
            self.bump();
            then
        } else {
            otherwise
        }
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<TokenKind> {
        Lexer::new(source)
            .tokenize()
            .into_iter()
            .map(|t| t.kind)
            .collect()
    }

    fn kinds_no_trivia(source: &str) -> Vec<TokenKind> {
        kinds(source)
            .into_iter()
            .filter(|k| {
                !matches!(
                    k,
                    TokenKind::Whitespace
                        | TokenKind::LineComment
                        | TokenKind::BlockComment
                        | TokenKind::DocComment
                )
            })
            .collect()
    }

    // -- Keywords and identifiers --

    #[test]
    fn keyword_vs_identifier() {
        assert_eq!(
            kinds_no_trivia("contract"),
            vec![TokenKind::Contract, TokenKind::Eof]
        );
        assert_eq!(
            kinds_no_trivia("contractX"),
            vec![TokenKind::Identifier, TokenKind::Eof]
        );
    }

    #[test]
    fn identifier_with_underscore_and_digits() {
        assert_eq!(
            kinds_no_trivia("_balance_1"),
            vec![TokenKind::Identifier, TokenKind::Eof]
        );
    }

    #[test]
    fn underscore_alone_is_an_identifier() {
        // The wildcard-pattern meaning of `_` is a parser concern, not a
        // lexical one - see docs/LANGUAGE_SPEC.md §7.7.
        assert_eq!(
            kinds_no_trivia("_"),
            vec![TokenKind::Identifier, TokenKind::Eof]
        );
    }

    #[test]
    fn reserved_word_is_lexically_distinct_from_identifier() {
        assert_eq!(
            kinds_no_trivia("self"),
            vec![TokenKind::KwSelfValue, TokenKind::Eof]
        );
        assert_eq!(
            kinds_no_trivia("Self"),
            vec![TokenKind::KwSelfType, TokenKind::Eof]
        );
    }

    #[test]
    fn non_ascii_character_is_a_lex_error_not_an_identifier() {
        // ASCII-only identifiers, per docs/LANGUAGE_SPEC.md §1.1: a
        // standalone non-ASCII character can never start or continue an
        // identifier, so it becomes its own Error token.
        assert_eq!(
            kinds_no_trivia("café"),
            vec![
                TokenKind::Identifier, // "caf"
                TokenKind::Error,      // "é"
                TokenKind::Eof,
            ]
        );
    }

    // -- Integer literals --

    #[test]
    fn decimal_literal_with_separators() {
        assert_eq!(
            kinds_no_trivia("1_000_000"),
            vec![TokenKind::IntLiteral, TokenKind::Eof]
        );
    }

    #[test]
    fn hex_literal_with_separators() {
        assert_eq!(
            kinds_no_trivia("0xFF_FF"),
            vec![TokenKind::IntLiteral, TokenKind::Eof]
        );
    }

    #[test]
    fn malformed_hex_literal_is_an_error() {
        assert_eq!(
            kinds_no_trivia("0x"),
            vec![TokenKind::Error, TokenKind::Eof]
        );
        assert_eq!(
            kinds_no_trivia("0xZZ"),
            vec![TokenKind::Error, TokenKind::Eof]
        );
    }

    #[test]
    fn zero_x_prefix_of_decimal_is_hex_not_two_tokens() {
        let tokens = Lexer::new("0x1").tokenize();
        assert_eq!(tokens[0].kind, TokenKind::IntLiteral);
        assert_eq!(tokens[0].span.start, 0);
        assert_eq!(tokens[0].span.end, 3);
    }

    // -- String literals --

    #[test]
    fn simple_string_literal() {
        let tokens = kinds_no_trivia(r#""hello""#);
        assert_eq!(tokens, vec![TokenKind::StringLiteral, TokenKind::Eof]);
    }

    #[test]
    fn string_with_valid_escapes() {
        let tokens = kinds_no_trivia(r#""a\nb\tc\\d\"e""#);
        assert_eq!(tokens, vec![TokenKind::StringLiteral, TokenKind::Eof]);
    }

    #[test]
    fn string_with_valid_unicode_escape() {
        let tokens = kinds_no_trivia(r#""\u{48}\u{1F600}""#);
        assert_eq!(tokens, vec![TokenKind::StringLiteral, TokenKind::Eof]);
    }

    #[test]
    fn string_with_invalid_escape_is_one_error_token() {
        let tokens = Lexer::new(r#""bad\qend""#).tokenize();
        let non_eof: Vec<_> = tokens.iter().filter(|t| t.kind != TokenKind::Eof).collect();
        assert_eq!(
            non_eof.len(),
            1,
            "malformed string should be a single token, got {non_eof:?}"
        );
        assert_eq!(non_eof[0].kind, TokenKind::Error);
    }

    #[test]
    fn string_with_invalid_unicode_codepoint_is_an_error() {
        // 0x110000 exceeds the maximum valid Unicode scalar value.
        let tokens = kinds_no_trivia(r#""\u{110000}""#);
        assert_eq!(tokens, vec![TokenKind::Error, TokenKind::Eof]);
    }

    #[test]
    fn unterminated_string_is_an_error() {
        assert_eq!(
            kinds_no_trivia("\"unterminated"),
            vec![TokenKind::Error, TokenKind::Eof]
        );
    }

    #[test]
    fn string_may_contain_a_raw_newline() {
        // docs/LANGUAGE_SPEC.md §1.6's string_char grammar excludes only
        // `"` and unescaped backslash - a raw newline is permitted.
        assert_eq!(
            kinds_no_trivia("\"line1\nline2\""),
            vec![TokenKind::StringLiteral, TokenKind::Eof]
        );
    }

    // -- Comments --

    #[test]
    fn line_comment_vs_doc_comment() {
        assert_eq!(
            kinds("// plain"),
            vec![TokenKind::LineComment, TokenKind::Eof]
        );
        assert_eq!(
            kinds("/// doc"),
            vec![TokenKind::DocComment, TokenKind::Eof]
        );
        assert_eq!(
            kinds("//// divider"),
            vec![TokenKind::LineComment, TokenKind::Eof]
        );
    }

    #[test]
    fn line_comment_stops_at_newline() {
        let tokens = kinds_no_trivia("// a\nfn");
        assert_eq!(tokens, vec![TokenKind::Fn, TokenKind::Eof]);
    }

    #[test]
    fn block_comment_simple() {
        assert_eq!(
            kinds("/* hello */"),
            vec![TokenKind::BlockComment, TokenKind::Eof]
        );
    }

    #[test]
    fn block_comment_nests() {
        assert_eq!(
            kinds("/* outer /* inner */ still outer */"),
            vec![TokenKind::BlockComment, TokenKind::Eof]
        );
    }

    #[test]
    fn unterminated_block_comment_is_an_error() {
        assert_eq!(
            kinds("/* never closed"),
            vec![TokenKind::Error, TokenKind::Eof]
        );
    }

    #[test]
    fn unbalanced_nested_block_comment_is_an_error() {
        // Opens two, closes only one.
        assert_eq!(
            kinds("/* /* nested */ trailing"),
            vec![TokenKind::Error, TokenKind::Eof]
        );
    }

    // -- Punctuation and operators --

    #[test]
    fn every_single_char_punctuation() {
        let source = "{ } ( ) ; , . ? : ! < >";
        assert_eq!(
            kinds_no_trivia(source),
            vec![
                TokenKind::LBrace,
                TokenKind::RBrace,
                TokenKind::LParen,
                TokenKind::RParen,
                TokenKind::Semicolon,
                TokenKind::Comma,
                TokenKind::Dot,
                TokenKind::Question,
                TokenKind::Colon,
                TokenKind::Bang,
                TokenKind::LAngle,
                TokenKind::RAngle,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn every_two_char_operator() {
        let source = ":: == != <= >= && || += -= *= /= %= -> =>";
        assert_eq!(
            kinds_no_trivia(source),
            vec![
                TokenKind::ColonColon,
                TokenKind::EqEq,
                TokenKind::NotEq,
                TokenKind::LtEq,
                TokenKind::GtEq,
                TokenKind::AmpAmp,
                TokenKind::PipePipe,
                TokenKind::PlusEq,
                TokenKind::MinusEq,
                TokenKind::StarEq,
                TokenKind::SlashEq,
                TokenKind::PercentEq,
                TokenKind::Arrow,
                TokenKind::FatArrow,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn arithmetic_operators() {
        assert_eq!(
            kinds_no_trivia("+ - * / %"),
            vec![
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::Percent,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn bare_ampersand_or_pipe_is_an_error() {
        // No bitwise operators in Kyne - a single `&` or `|` is invalid.
        assert_eq!(kinds_no_trivia("&"), vec![TokenKind::Error, TokenKind::Eof]);
        assert_eq!(kinds_no_trivia("|"), vec![TokenKind::Error, TokenKind::Eof]);
    }

    #[test]
    fn unrecognized_character_is_an_error_and_lexing_resumes() {
        assert_eq!(
            kinds_no_trivia("let x = 1 @ 2;"),
            vec![
                TokenKind::Let,
                TokenKind::Identifier,
                TokenKind::Eq,
                TokenKind::IntLiteral,
                TokenKind::Error,
                TokenKind::IntLiteral,
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    // -- Whitespace and spans --

    #[test]
    fn whitespace_is_a_single_token_between_others() {
        assert_eq!(
            kinds("fn  x"),
            vec![
                TokenKind::Fn,
                TokenKind::Whitespace,
                TokenKind::Identifier,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn empty_source_is_just_eof() {
        assert_eq!(kinds(""), vec![TokenKind::Eof]);
    }

    #[test]
    fn line_and_column_tracking() {
        let tokens = Lexer::new("fn\nx").tokenize();
        // "fn" at line 1, col 1.
        assert_eq!(tokens[0].span.line, 1);
        assert_eq!(tokens[0].span.column, 1);
        // "x" is after the newline: line 2, col 1.
        let ident = tokens
            .iter()
            .find(|t| t.kind == TokenKind::Identifier)
            .unwrap();
        assert_eq!(ident.span.line, 2);
        assert_eq!(ident.span.column, 1);
    }

    #[test]
    fn token_text_round_trips_via_span() {
        let source = "contract Counter";
        let tokens = Lexer::new(source).tokenize();
        assert_eq!(tokens[0].text(source), "contract");
        assert_eq!(tokens[2].text(source), "Counter");
    }

    #[test]
    fn realistic_snippet_tokenizes_with_no_errors() {
        let source = "public fn transfer(from: address, to: address, amount: i128) -> Result<bool, TokenError> {";
        let tokens = kinds(source);
        assert!(
            !tokens.contains(&TokenKind::Error),
            "expected no Error tokens in a valid snippet, got {tokens:?}"
        );
    }
}
