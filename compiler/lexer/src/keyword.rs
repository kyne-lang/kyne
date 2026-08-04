//! Keyword and reserved-word recognition, per docs/LANGUAGE_SPEC.md §1.3
//! and §1.3.1.

use crate::token::TokenKind;

/// Maps an identifier-shaped word to its `TokenKind`, if it is one of
/// Kyne's primary keywords or a word reserved for a future language
/// version. Returns `None` for an ordinary identifier.
pub fn lookup(text: &str) -> Option<TokenKind> {
    use TokenKind::*;
    Some(match text {
        // Primary keywords — docs/LANGUAGE_SPEC.md §1.3.
        "contract" => Contract,
        "fn" => Fn,
        "state" => State,
        "let" => Let,
        "const" => Const,
        "event" => Event,
        "emit" => Emit,
        "use" => Use,
        "public" => Public,
        "internal" => Internal,
        "struct" => Struct,
        "enum" => Enum,
        "trait" => Trait,
        "match" => Match,
        "if" => If,
        "else" => Else,
        "for" => For,
        "while" => While,
        "return" => Return,
        "break" => Break,
        "continue" => Continue,
        "error" => ErrorKw,
        "throw" => Throw,
        "true" => True,
        "false" => False,
        "auth" => Auth,
        "in" => In,

        // Reserved for future language versions — docs/LANGUAGE_SPEC.md §1.3.1.
        "self" => KwSelfValue,
        "Self" => KwSelfType,
        "impl" => KwImpl,
        "mut" => KwMut,
        "async" => KwAsync,
        "await" => KwAwait,
        "unsafe" => KwUnsafe,
        "dyn" => KwDyn,
        "as" => KwAs,
        "move" => KwMove,

        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_every_primary_keyword() {
        let keywords = [
            "contract", "fn", "state", "let", "const", "event", "emit", "use", "public",
            "internal", "struct", "enum", "trait", "match", "if", "else", "for", "while", "return",
            "break", "continue", "error", "throw", "true", "false", "auth", "in",
        ];
        for kw in keywords {
            assert!(
                lookup(kw).is_some(),
                "{kw} should be recognized as a keyword"
            );
        }
    }

    #[test]
    fn recognizes_every_reserved_word() {
        let reserved = [
            "self", "Self", "impl", "mut", "async", "await", "unsafe", "dyn", "as", "move",
        ];
        for word in reserved {
            assert!(
                lookup(word).is_some(),
                "{word} should be recognized as reserved"
            );
        }
    }

    #[test]
    fn ordinary_identifiers_are_not_keywords() {
        for word in ["balance", "transfer", "TokenError", "amount_", "_private"] {
            assert!(lookup(word).is_none(), "{word} should not be a keyword");
        }
    }

    #[test]
    fn keyword_lookup_is_case_sensitive() {
        // "Self" is reserved, but "SELF" and "self_" are ordinary identifiers.
        assert!(lookup("SELF").is_none());
        assert!(lookup("self_").is_none());
        assert!(lookup("Contract").is_none());
    }
}
