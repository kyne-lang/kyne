//! The recursive-descent parser, per docs/COMPILER_ARCHITECTURE.md §5.
//!
//! Implementation technique and the `KY00xx` syntax-error code range are
//! recorded in docs/adr/ADR-0005-parser-implementation.md.

use kyne_diagnostics::{Diagnostic, Span as DiagSpan};
use kyne_lexer::{Token, TokenKind};

use crate::tree::{NodeKind, SyntaxElement, SyntaxNode};

fn is_trivia(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Whitespace
            | TokenKind::LineComment
            | TokenKind::BlockComment
            | TokenKind::DocComment
    )
}

pub struct Parser<'src> {
    source: &'src str,
    file: String,
    tokens: Vec<Token>,
    pos: usize,
    stack: Vec<SyntaxNode>,
    diagnostics: Vec<Diagnostic>,
    /// Disallows recognizing a bare `identifier { ... }` as a struct
    /// literal at the current nesting level. Set while parsing the head
    /// expression of `if`/`while`/`for`/`match`, where a trailing `{`
    /// unambiguously belongs to the following `Block`/arm-list rather
    /// than to the expression - mirroring the same restriction Rust
    /// applies to condition expressions, for the same reason.
    /// LANGUAGE_SPEC.md does not spell this rule out explicitly; it is
    /// forced by the six canonical examples (e.g. Escrow's
    /// `if released { ... }`, where `released` is a bare `bool` field,
    /// not a struct) failing to parse without it.
    no_struct_literal: bool,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str, file: impl Into<String>) -> Self {
        Parser {
            source,
            file: file.into(),
            tokens: kyne_lexer::lex(source),
            pos: 0,
            stack: Vec::new(),
            diagnostics: Vec::new(),
            no_struct_literal: false,
        }
    }

    pub fn parse(mut self) -> (SyntaxNode, Vec<Diagnostic>) {
        self.start_node(NodeKind::Program);
        self.parse_program_body();
        // Attach any trailing trivia, then the final `Eof` token, to the
        // root before closing it - every token in the stream, including
        // the last, must be reachable from the tree.
        self.bump();
        let diagnostics = std::mem::take(&mut self.diagnostics);
        let root = self.finish_root();
        (root, diagnostics)
    }

    // ---- Tree-building primitives ----

    fn start_node(&mut self, kind: NodeKind) {
        self.stack.push(SyntaxNode::new(kind));
    }

    fn finish_node(&mut self) {
        let node = self.stack.pop().expect("finish_node with empty stack");
        self.stack
            .last_mut()
            .expect("finish_node called on the root node - use finish_root instead")
            .children
            .push(SyntaxElement::Node(node));
    }

    fn finish_root(mut self) -> SyntaxNode {
        assert_eq!(
            self.stack.len(),
            1,
            "parse() must leave exactly the root node open"
        );
        self.stack.pop().unwrap()
    }

    /// Abandons the node currently being built, discarding it without
    /// attaching it to its parent - used only when a production turns out
    /// to have produced nothing valid at all.
    fn abandon_node(&mut self) -> SyntaxNode {
        self.stack.pop().expect("abandon_node with empty stack")
    }

    fn current_node_mut(&mut self) -> &mut SyntaxNode {
        self.stack.last_mut().expect("no open node")
    }

    /// Consumes and attaches the next token (draining any pending trivia
    /// into the currently-open node first). Never called past `Eof`.
    fn bump(&mut self) -> Token {
        while self.pos < self.tokens.len() - 1 && is_trivia(self.tokens[self.pos].kind) {
            let tok = self.tokens[self.pos];
            self.current_node_mut()
                .children
                .push(SyntaxElement::Token(tok));
            self.pos += 1;
        }
        let tok = self.tokens[self.pos];
        self.current_node_mut()
            .children
            .push(SyntaxElement::Token(tok));
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        tok
    }

    /// The kind of the next significant (non-trivia) token, without
    /// consuming anything.
    fn peek(&self) -> TokenKind {
        let mut i = self.pos;
        while i < self.tokens.len() - 1 && is_trivia(self.tokens[i].kind) {
            i += 1;
        }
        self.tokens[i].kind
    }

    fn peek_token(&self) -> Token {
        let mut i = self.pos;
        while i < self.tokens.len() - 1 && is_trivia(self.tokens[i].kind) {
            i += 1;
        }
        self.tokens[i]
    }

    /// The kind of the significant token one past the next one, without
    /// consuming anything.
    fn peek2(&self) -> TokenKind {
        let mut i = self.pos;
        let mut seen = 0;
        while i < self.tokens.len() - 1 {
            if !is_trivia(self.tokens[i].kind) {
                seen += 1;
                if seen == 2 {
                    return self.tokens[i].kind;
                }
            }
            i += 1;
        }
        TokenKind::Eof
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.peek() == kind
    }

    fn eat(&mut self, kind: TokenKind) -> Option<Token> {
        if self.at(kind) {
            Some(self.bump())
        } else {
            None
        }
    }

    fn diag_span(&self, token: Token) -> DiagSpan {
        let len = token.text(self.source).chars().count().max(1);
        DiagSpan::new(token.span.line, token.span.column, len)
    }

    fn error_unexpected(&mut self, expected: &str) {
        let token = self.peek_token();
        let found = token.text(self.source);
        let found_desc = if token.kind == TokenKind::Eof {
            "end of file".to_string()
        } else {
            format!("`{found}`")
        };
        let diagnostic = Diagnostic::error(
            "KY0001",
            format!("expected {expected}, found {found_desc}"),
            self.file.clone(),
            self.diag_span(token),
            format!("expected {expected} here"),
        )
        .note("the parser could not match this token against any valid grammar production at this position, per LANGUAGE_SPEC.md §2")
        .help(format!("replace this with {expected}, or remove it if it was left over from an edit"));
        self.diagnostics.push(diagnostic);
    }

    /// Consumes `kind`, or records a syntax error and leaves the token
    /// stream untouched so recovery can resynchronize.
    fn expect(&mut self, kind: TokenKind, expected: &str) -> Option<Token> {
        if let Some(tok) = self.eat(kind) {
            Some(tok)
        } else {
            self.error_unexpected(expected);
            None
        }
    }

    /// Error-recovery synchronization, per docs/COMPILER_ARCHITECTURE.md
    /// §5: discards tokens up to and including the next `;` or `}` at the
    /// current nesting depth, so a single malformed construct does not
    /// abort the rest of the file.
    fn synchronize(&mut self) {
        self.start_node(NodeKind::ErrorNode);
        let mut depth: i32 = 0;
        loop {
            match self.peek() {
                TokenKind::Eof => break,
                TokenKind::LBrace => {
                    depth += 1;
                    self.bump();
                }
                TokenKind::RBrace => {
                    if depth == 0 {
                        self.bump();
                        break;
                    }
                    depth -= 1;
                    self.bump();
                    if depth == 0 {
                        break;
                    }
                }
                TokenKind::Semicolon if depth == 0 => {
                    self.bump();
                    break;
                }
                _ => {
                    self.bump();
                }
            }
        }
        self.finish_node();
    }

    // ---- §2 Program ----

    fn parse_program_body(&mut self) {
        while self.at(TokenKind::Use) {
            self.parse_import();
        }
        while !self.at(TokenKind::Eof) {
            self.parse_top_level_decl();
        }
    }

    fn parse_import(&mut self) {
        self.start_node(NodeKind::Import);
        self.bump(); // "use"
        self.parse_import_path();
        if self.eat(TokenKind::Dot).is_some() && self.expect(TokenKind::LBrace, "`{`").is_some() {
            self.parse_identifier_list();
            self.expect(TokenKind::RBrace, "`}`");
        }
        self.expect(TokenKind::Semicolon, "`;`");
        self.finish_node();
    }

    fn parse_import_path(&mut self) {
        self.start_node(NodeKind::ImportPath);
        self.expect(TokenKind::Identifier, "an identifier");
        while self.at(TokenKind::Dot) && self.peek2() == TokenKind::Identifier {
            self.bump(); // "."
            self.bump(); // identifier
        }
        self.finish_node();
    }

    fn parse_identifier_list(&mut self) {
        self.start_node(NodeKind::IdentifierList);
        self.expect(TokenKind::Identifier, "an identifier");
        while self.eat(TokenKind::Comma).is_some() {
            if self.at(TokenKind::RBrace) {
                break;
            }
            self.expect(TokenKind::Identifier, "an identifier");
        }
        self.finish_node();
    }

    fn parse_top_level_decl(&mut self) {
        match self.peek() {
            TokenKind::Contract => self.parse_contract_decl(),
            TokenKind::Struct => self.parse_struct_decl(),
            TokenKind::Enum => self.parse_enum_decl(),
            TokenKind::ErrorKw => self.parse_error_decl(),
            TokenKind::Event => self.parse_event_decl(),
            TokenKind::Const => self.parse_const_decl(),
            TokenKind::Public | TokenKind::Internal | TokenKind::Fn => self.parse_fn_decl(),
            _ => {
                self.error_unexpected("a top-level declaration");
                self.synchronize();
            }
        }
    }

    // ---- §3 Contracts ----

    fn parse_contract_decl(&mut self) {
        self.start_node(NodeKind::ContractDecl);
        self.bump(); // "contract"
        self.expect(TokenKind::Identifier, "a contract name");
        if self.expect(TokenKind::LBrace, "`{`").is_some() {
            while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
                self.parse_contract_member();
            }
            self.expect(TokenKind::RBrace, "`}`");
        }
        self.finish_node();
    }

    fn parse_contract_member(&mut self) {
        match self.peek() {
            TokenKind::State => self.parse_state_decl(),
            TokenKind::Const => self.parse_const_decl(),
            TokenKind::ErrorKw => self.parse_error_decl(),
            TokenKind::Event => self.parse_event_decl(),
            TokenKind::Struct => self.parse_struct_decl(),
            TokenKind::Enum => self.parse_enum_decl(),
            TokenKind::Public | TokenKind::Internal | TokenKind::Fn => self.parse_fn_decl(),
            _ => {
                self.error_unexpected("a contract member");
                self.synchronize();
            }
        }
    }

    fn parse_state_decl(&mut self) {
        self.start_node(NodeKind::StateDecl);
        self.bump(); // "state"
        self.expect(TokenKind::Identifier, "a field name");
        self.expect(TokenKind::Colon, "`:`");
        self.parse_type();
        if self.eat(TokenKind::Eq).is_some() {
            self.parse_expression();
        }
        self.expect(TokenKind::Semicolon, "`;`");
        self.finish_node();
    }

    fn parse_const_decl(&mut self) {
        self.start_node(NodeKind::ConstDecl);
        self.bump(); // "const"
        self.expect(TokenKind::Identifier, "a constant name");
        self.expect(TokenKind::Colon, "`:`");
        self.parse_type();
        self.expect(TokenKind::Eq, "`=`");
        self.parse_expression();
        self.expect(TokenKind::Semicolon, "`;`");
        self.finish_node();
    }

    fn parse_error_decl(&mut self) {
        self.start_node(NodeKind::ErrorDecl);
        self.bump(); // "error"
        self.expect(TokenKind::Identifier, "an error name");
        if self.at(TokenKind::Semicolon) {
            self.bump();
        } else if self.expect(TokenKind::LBrace, "`{` or `;`").is_some() {
            self.start_node(NodeKind::ErrorVariants);
            self.expect(TokenKind::Identifier, "an error variant name");
            while self.eat(TokenKind::Comma).is_some() {
                if self.at(TokenKind::RBrace) {
                    break;
                }
                self.expect(TokenKind::Identifier, "an error variant name");
            }
            self.finish_node();
            self.expect(TokenKind::RBrace, "`}`");
        }
        self.finish_node();
    }

    fn parse_event_decl(&mut self) {
        self.start_node(NodeKind::EventDecl);
        self.bump(); // "event"
        self.expect(TokenKind::Identifier, "an event name");
        if self.expect(TokenKind::LParen, "`(`").is_some() {
            if !self.at(TokenKind::RParen) {
                self.parse_param_list();
            }
            self.expect(TokenKind::RParen, "`)`");
        }
        self.expect(TokenKind::Semicolon, "`;`");
        self.finish_node();
    }

    fn parse_struct_decl(&mut self) {
        self.start_node(NodeKind::StructDecl);
        self.bump(); // "struct"
        self.expect(TokenKind::Identifier, "a struct name");
        if self.expect(TokenKind::LBrace, "`{`").is_some() {
            if !self.at(TokenKind::RBrace) {
                self.parse_field_list();
            }
            self.expect(TokenKind::RBrace, "`}`");
        }
        self.finish_node();
    }

    fn parse_field_list(&mut self) {
        self.start_node(NodeKind::FieldList);
        self.parse_field();
        while self.eat(TokenKind::Comma).is_some() {
            if self.at(TokenKind::RBrace) || self.at(TokenKind::RParen) {
                break;
            }
            self.parse_field();
        }
        self.finish_node();
    }

    fn parse_field(&mut self) {
        self.start_node(NodeKind::Field);
        self.expect(TokenKind::Identifier, "a field name");
        self.expect(TokenKind::Colon, "`:`");
        self.parse_type();
        self.finish_node();
    }

    fn parse_enum_decl(&mut self) {
        self.start_node(NodeKind::EnumDecl);
        self.bump(); // "enum"
        self.expect(TokenKind::Identifier, "an enum name");
        if self.expect(TokenKind::LBrace, "`{`").is_some() {
            self.parse_enum_variant();
            while self.eat(TokenKind::Comma).is_some() {
                if self.at(TokenKind::RBrace) {
                    break;
                }
                self.parse_enum_variant();
            }
            self.expect(TokenKind::RBrace, "`}`");
        }
        self.finish_node();
    }

    fn parse_enum_variant(&mut self) {
        self.start_node(NodeKind::EnumVariant);
        self.expect(TokenKind::Identifier, "a variant name");
        if self.eat(TokenKind::LParen).is_some() {
            self.parse_type_list();
            self.expect(TokenKind::RParen, "`)`");
        } else if self.eat(TokenKind::LBrace).is_some() {
            if !self.at(TokenKind::RBrace) {
                self.parse_field_list();
            }
            self.expect(TokenKind::RBrace, "`}`");
        }
        self.finish_node();
    }

    fn parse_type_list(&mut self) {
        self.start_node(NodeKind::TypeList);
        self.parse_type();
        while self.eat(TokenKind::Comma).is_some() {
            self.parse_type();
        }
        self.finish_node();
    }

    // ---- §5 Functions ----

    fn parse_fn_decl(&mut self) {
        self.start_node(NodeKind::FnDecl);
        if self.at(TokenKind::Public) || self.at(TokenKind::Internal) {
            self.bump();
        }
        self.expect(TokenKind::Fn, "`fn`");
        self.expect(TokenKind::Identifier, "a function name");
        if self.expect(TokenKind::LParen, "`(`").is_some() {
            if !self.at(TokenKind::RParen) {
                self.parse_param_list();
            }
            self.expect(TokenKind::RParen, "`)`");
        }
        if self.eat(TokenKind::Arrow).is_some() {
            self.parse_type();
        }
        self.parse_block();
        self.finish_node();
    }

    fn parse_param_list(&mut self) {
        self.start_node(NodeKind::ParamList);
        self.parse_param();
        while self.eat(TokenKind::Comma).is_some() {
            if self.at(TokenKind::RParen) {
                break;
            }
            self.parse_param();
        }
        self.finish_node();
    }

    fn parse_param(&mut self) {
        self.start_node(NodeKind::Param);
        self.expect(TokenKind::Identifier, "a parameter name");
        self.expect(TokenKind::Colon, "`:`");
        self.parse_type();
        self.finish_node();
    }

    // ---- §6 Types ----
    //
    // "bool", "i32", "address", etc. are not their own keywords in
    // kyne_lexer's TokenKind set - all primitive and parametric type
    // names lex as ordinary `Identifier` tokens. The parser accepts any
    // identifier here and does not itself validate it against
    // LANGUAGE_SPEC.md §6.1's fixed, closed set; that validation is the
    // Type Checker's job (`kyne_types`), consistent with the parser only
    // enforcing grammar shape, not semantic legality, per
    // docs/COMPILER_ARCHITECTURE.md §5.

    fn parse_type(&mut self) {
        self.start_node(NodeKind::Type);
        match self.peek() {
            TokenKind::Identifier => {
                let text = self.peek_token().text(self.source).to_string();
                match text.as_str() {
                    "list" | "map" | "bytes" | "Option" | "Result"
                        if self.peek2() == TokenKind::LAngle =>
                    {
                        self.parse_parametric_type(&text);
                    }
                    _ => {
                        self.bump();
                    }
                }
            }
            _ => {
                self.error_unexpected("a type");
            }
        }
        self.finish_node();
    }

    fn parse_parametric_type(&mut self, name: &str) {
        self.bump(); // the type name identifier
        self.expect(TokenKind::LAngle, "`<`");
        match name {
            "list" | "Option" => {
                self.parse_type();
            }
            "map" | "Result" => {
                self.parse_type();
                self.expect(TokenKind::Comma, "`,`");
                self.parse_type();
            }
            "bytes" => {
                self.expect(TokenKind::IntLiteral, "an integer literal");
            }
            _ => unreachable!("caller only dispatches known parametric type names"),
        }
        self.expect(TokenKind::RAngle, "`>`");
    }

    // ---- §8 Statements ----

    fn parse_block(&mut self) {
        self.start_node(NodeKind::Block);
        if self.expect(TokenKind::LBrace, "`{`").is_some() {
            while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
                self.parse_statement();
            }
            self.expect(TokenKind::RBrace, "`}`");
        }
        self.finish_node();
    }

    fn parse_statement(&mut self) {
        match self.peek() {
            TokenKind::Let => self.parse_let_stmt(),
            TokenKind::Auth => self.parse_auth_stmt(),
            TokenKind::Emit => self.parse_emit_stmt(),
            TokenKind::Throw => self.parse_throw_stmt(),
            TokenKind::If => {
                self.parse_if(NodeKind::IfStmt);
            }
            TokenKind::Match => {
                self.parse_match(NodeKind::MatchStmt);
            }
            TokenKind::For => self.parse_for_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::Return => self.parse_return_stmt(),
            TokenKind::Break => self.parse_break_stmt(),
            TokenKind::Continue => self.parse_continue_stmt(),
            _ => self.parse_expr_or_assign_stmt(),
        }
    }

    fn parse_let_stmt(&mut self) {
        self.start_node(NodeKind::LetStmt);
        self.bump(); // "let"
        self.expect(TokenKind::Identifier, "a variable name");
        if self.eat(TokenKind::Colon).is_some() {
            self.parse_type();
        }
        self.expect(TokenKind::Eq, "`=`");
        self.parse_expression();
        self.expect(TokenKind::Semicolon, "`;`");
        self.finish_node();
    }

    fn parse_auth_stmt(&mut self) {
        self.start_node(NodeKind::AuthStmt);
        self.bump(); // "auth"
        self.expect(TokenKind::LParen, "`(`");
        self.parse_expression();
        self.expect(TokenKind::RParen, "`)`");
        self.expect(TokenKind::Semicolon, "`;`");
        self.finish_node();
    }

    fn parse_emit_stmt(&mut self) {
        self.start_node(NodeKind::EmitStmt);
        self.bump(); // "emit"
        self.expect(TokenKind::Identifier, "an event name");
        if self.expect(TokenKind::LParen, "`(`").is_some() {
            if !self.at(TokenKind::RParen) {
                self.parse_arg_list();
            }
            self.expect(TokenKind::RParen, "`)`");
        }
        self.expect(TokenKind::Semicolon, "`;`");
        self.finish_node();
    }

    fn parse_throw_stmt(&mut self) {
        self.start_node(NodeKind::ThrowStmt);
        self.bump(); // "throw"
        self.parse_expression();
        self.expect(TokenKind::Semicolon, "`;`");
        self.finish_node();
    }

    fn parse_for_stmt(&mut self) {
        self.start_node(NodeKind::ForStmt);
        self.bump(); // "for"
        self.expect(TokenKind::Identifier, "a loop variable name");
        self.expect(TokenKind::In, "`in`");
        self.with_no_struct_literal(Self::parse_expression);
        self.parse_block();
        self.finish_node();
    }

    fn parse_while_stmt(&mut self) {
        self.start_node(NodeKind::WhileStmt);
        self.bump(); // "while"
        self.with_no_struct_literal(Self::parse_expression);
        self.parse_block();
        self.finish_node();
    }

    fn parse_return_stmt(&mut self) {
        self.start_node(NodeKind::ReturnStmt);
        self.bump(); // "return"
        if !self.at(TokenKind::Semicolon) {
            self.parse_expression();
        }
        self.expect(TokenKind::Semicolon, "`;`");
        self.finish_node();
    }

    fn parse_break_stmt(&mut self) {
        self.start_node(NodeKind::BreakStmt);
        self.bump();
        self.expect(TokenKind::Semicolon, "`;`");
        self.finish_node();
    }

    fn parse_continue_stmt(&mut self) {
        self.start_node(NodeKind::ContinueStmt);
        self.bump();
        self.expect(TokenKind::Semicolon, "`;`");
        self.finish_node();
    }

    fn is_assign_op(kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Eq
                | TokenKind::PlusEq
                | TokenKind::MinusEq
                | TokenKind::StarEq
                | TokenKind::SlashEq
                | TokenKind::PercentEq
        )
    }

    fn parse_expr_or_assign_stmt(&mut self) {
        self.start_node(NodeKind::ExprStmt);
        self.parse_expression();
        if Self::is_assign_op(self.peek()) {
            // Retroactively reinterpret as an AssignStmt: pull the
            // already-built expression back out and re-wrap it.
            let lhs_holder = self.abandon_node();
            self.start_node(NodeKind::AssignStmt);
            self.current_node_mut().children.extend(lhs_holder.children);
            self.bump(); // the assignment operator
            self.parse_expression();
            self.expect(TokenKind::Semicolon, "`;`");
            self.finish_node();
        } else if self.at(TokenKind::RBrace) {
            // A semicolon-less trailing expression, immediately followed
            // by the block's closing `}`, is the block's value - required
            // for `if`/`match` used in expression position, per
            // LANGUAGE_SPEC.md §7.6's own example
            // (`if amount > 1000 { amount / 100 } else { 10 }`), even
            // though `Block = "{" { Statement } "}"` in §2's formal
            // grammar does not show this as a distinct production. Not
            // consuming a `;` here is unambiguous: nothing else in the
            // grammar can immediately follow an expression except a
            // `;`-requiring construct or the block's own closing brace.
            self.finish_node();
        } else {
            self.expect(TokenKind::Semicolon, "`;`");
            self.finish_node();
        }
    }

    fn with_no_struct_literal<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let previous = self.no_struct_literal;
        self.no_struct_literal = true;
        let result = f(self);
        self.no_struct_literal = previous;
        result
    }

    fn with_struct_literal_allowed<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let previous = self.no_struct_literal;
        self.no_struct_literal = false;
        let result = f(self);
        self.no_struct_literal = previous;
        result
    }

    // ---- if / match, shared between statement and expression position ----

    fn parse_if(&mut self, kind: NodeKind) {
        self.start_node(kind);
        self.bump(); // "if"
        self.with_no_struct_literal(Self::parse_expression);
        self.parse_block();
        if self.eat(TokenKind::Else).is_some() {
            if self.at(TokenKind::If) {
                self.parse_if(NodeKind::IfStmt);
            } else {
                self.parse_block();
            }
        }
        self.finish_node();
    }

    fn parse_match(&mut self, kind: NodeKind) {
        self.start_node(kind);
        self.bump(); // "match"
        self.with_no_struct_literal(Self::parse_expression);
        if self.expect(TokenKind::LBrace, "`{`").is_some() {
            while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
                self.parse_match_arm();
            }
            self.expect(TokenKind::RBrace, "`}`");
        }
        self.finish_node();
    }

    fn parse_match_arm(&mut self) {
        self.start_node(NodeKind::MatchArm);
        self.parse_pattern();
        if self.at(TokenKind::If) {
            self.bump();
            self.with_no_struct_literal(Self::parse_expression);
        }
        self.expect(TokenKind::FatArrow, "`=>`");
        if self.at(TokenKind::LBrace) {
            self.parse_block();
        } else {
            // A non-block arm body is `Expression` per LANGUAGE_SPEC.md
            // §2's `MatchArm` production, but §7.7's own worked example
            // uses bare `return`/`throw` directly as an arm body (e.g.
            // `Status::Approved(by) if by == admin => return true,`) -
            // neither is a grammar-level `Expression`. §7.7 confirms this
            // is intentional: "`throw` and `return` used as a match arm's
            // body have the bottom type." `break`/`continue` are accepted
            // on the same footing, for the same reason a loop body can
            // use them as its final statement.
            match self.peek() {
                TokenKind::Return => self.parse_return_arm_body(),
                TokenKind::Throw => self.parse_throw_arm_body(),
                TokenKind::Break => self.parse_bare_keyword_arm_body(NodeKind::BreakStmt),
                TokenKind::Continue => self.parse_bare_keyword_arm_body(NodeKind::ContinueStmt),
                _ => self.parse_expression(),
            }
            if !self.at(TokenKind::RBrace) {
                self.expect(TokenKind::Comma, "`,`");
            } else {
                self.eat(TokenKind::Comma);
            }
        }
        self.finish_node();
    }

    /// `return [Expression]` as a match-arm body: no trailing `;` - the
    /// arm's `,` (or the closing `}`) terminates it instead.
    fn parse_return_arm_body(&mut self) {
        self.start_node(NodeKind::ReturnStmt);
        self.bump(); // "return"
        if !self.at(TokenKind::Comma) && !self.at(TokenKind::RBrace) {
            self.parse_expression();
        }
        self.finish_node();
    }

    /// `throw Expression` as a match-arm body: no trailing `;`.
    fn parse_throw_arm_body(&mut self) {
        self.start_node(NodeKind::ThrowStmt);
        self.bump(); // "throw"
        self.parse_expression();
        self.finish_node();
    }

    /// `break` or `continue` as a match-arm body: no trailing `;`.
    fn parse_bare_keyword_arm_body(&mut self, kind: NodeKind) {
        self.start_node(kind);
        self.bump();
        self.finish_node();
    }

    fn parse_pattern(&mut self) {
        self.start_node(NodeKind::Pattern);
        match self.peek() {
            TokenKind::IntLiteral
            | TokenKind::StringLiteral
            | TokenKind::True
            | TokenKind::False => {
                self.bump();
            }
            TokenKind::Identifier => {
                let text = self.peek_token().text(self.source).to_string();
                if text == "_" {
                    self.bump();
                } else {
                    self.bump();
                    if self.eat(TokenKind::ColonColon).is_some() {
                        self.expect(TokenKind::Identifier, "a variant name");
                    }
                    if self.eat(TokenKind::LParen).is_some() {
                        self.start_node(NodeKind::PatternList);
                        if !self.at(TokenKind::RParen) {
                            self.parse_pattern();
                            while self.eat(TokenKind::Comma).is_some() {
                                if self.at(TokenKind::RParen) {
                                    break;
                                }
                                self.parse_pattern();
                            }
                        }
                        self.finish_node();
                        self.expect(TokenKind::RParen, "`)`");
                    } else if self.eat(TokenKind::LBrace).is_some() {
                        self.start_node(NodeKind::FieldPatternList);
                        if !self.at(TokenKind::RBrace) {
                            self.expect(TokenKind::Identifier, "a field name");
                            while self.eat(TokenKind::Comma).is_some() {
                                if self.at(TokenKind::RBrace) {
                                    break;
                                }
                                self.expect(TokenKind::Identifier, "a field name");
                            }
                        }
                        self.finish_node();
                        self.expect(TokenKind::RBrace, "`}`");
                    }
                }
            }
            TokenKind::Minus => {
                // A negative integer literal pattern, e.g. `-1`.
                self.bump();
                self.expect(TokenKind::IntLiteral, "an integer literal");
            }
            _ => {
                // No valid pattern production starts here, and nothing
                // else in this position can make progress either - the
                // offending token MUST be consumed so the enclosing
                // `while` loop over match arms cannot spin forever, per
                // the same forward-progress requirement as
                // `parse_primary_expr`'s fallback below.
                self.error_unexpected("a pattern");
                if !self.at(TokenKind::Eof) {
                    self.bump();
                }
            }
        }
        self.finish_node();
    }

    // ---- §7 Expressions ----

    fn parse_expression(&mut self) {
        self.parse_or_expr();
    }

    fn parse_or_expr(&mut self) {
        self.parse_binary_level(&[TokenKind::PipePipe], Self::parse_and_expr);
    }

    fn parse_and_expr(&mut self) {
        self.parse_binary_level(&[TokenKind::AmpAmp], Self::parse_equality_expr);
    }

    fn parse_equality_expr(&mut self) {
        self.parse_binary_level(
            &[TokenKind::EqEq, TokenKind::NotEq],
            Self::parse_comparison_expr,
        );
    }

    fn parse_comparison_expr(&mut self) {
        self.parse_binary_level(
            &[
                TokenKind::LAngle,
                TokenKind::RAngle,
                TokenKind::LtEq,
                TokenKind::GtEq,
            ],
            Self::parse_additive_expr,
        );
    }

    fn parse_additive_expr(&mut self) {
        self.parse_binary_level(
            &[TokenKind::Plus, TokenKind::Minus],
            Self::parse_multiplicative_expr,
        );
    }

    fn parse_multiplicative_expr(&mut self) {
        self.parse_binary_level(
            &[TokenKind::Star, TokenKind::Slash, TokenKind::Percent],
            Self::parse_unary_expr,
        );
    }

    fn parse_binary_level(&mut self, ops: &[TokenKind], mut next: impl FnMut(&mut Self)) {
        next(self);
        while ops.contains(&self.peek()) {
            let lhs = self.pop_last_child();
            self.start_node(NodeKind::BinaryExpr);
            self.current_node_mut().children.extend(lhs);
            self.bump(); // the operator
            next(self);
            self.finish_node();
        }
    }

    /// Pops whatever node the most recently finished sub-expression left
    /// as the last child of the current node, so a binary operator can
    /// re-wrap it as the left-hand side of a new `BinaryExpr`.
    fn pop_last_child(&mut self) -> Vec<SyntaxElement> {
        let last = self
            .current_node_mut()
            .children
            .pop()
            .expect("expected a preceding expression");
        vec![last]
    }

    fn parse_unary_expr(&mut self) {
        if matches!(self.peek(), TokenKind::Minus | TokenKind::Bang) {
            self.start_node(NodeKind::UnaryExpr);
            self.bump();
            self.parse_unary_expr();
            self.finish_node();
        } else {
            self.parse_postfix_expr();
        }
    }

    fn parse_postfix_expr(&mut self) {
        self.parse_primary_expr();
        loop {
            match self.peek() {
                TokenKind::Dot => {
                    let lhs = self.pop_last_child();
                    // Tentatively a field access; upgraded to a method
                    // call below if a `(` follows the name.
                    self.start_node(NodeKind::FieldExpr);
                    self.current_node_mut().children.extend(lhs);
                    self.bump(); // "."
                    self.expect(TokenKind::Identifier, "a field or method name");
                    if self.at(TokenKind::LParen) {
                        self.current_node_mut().kind = NodeKind::MethodCallExpr;
                        self.bump(); // "("
                        if !self.at(TokenKind::RParen) {
                            self.parse_arg_list();
                        }
                        self.expect(TokenKind::RParen, "`)`");
                    }
                    self.finish_node();
                }
                TokenKind::LParen => {
                    let lhs = self.pop_last_child();
                    self.start_node(NodeKind::CallExpr);
                    self.current_node_mut().children.extend(lhs);
                    self.bump(); // "("
                    if !self.at(TokenKind::RParen) {
                        self.parse_arg_list();
                    }
                    self.expect(TokenKind::RParen, "`)`");
                    self.finish_node();
                }
                TokenKind::Question => {
                    let lhs = self.pop_last_child();
                    self.start_node(NodeKind::TryExpr);
                    self.current_node_mut().children.extend(lhs);
                    self.bump(); // "?"
                    self.finish_node();
                }
                _ => break,
            }
        }
    }

    fn parse_arg_list(&mut self) {
        self.start_node(NodeKind::ArgList);
        self.with_struct_literal_allowed(Self::parse_expression);
        while self.eat(TokenKind::Comma).is_some() {
            if self.at(TokenKind::RParen) || self.at(TokenKind::RBracket) {
                break;
            }
            self.with_struct_literal_allowed(Self::parse_expression);
        }
        self.finish_node();
    }

    fn parse_primary_expr(&mut self) {
        match self.peek() {
            TokenKind::IntLiteral
            | TokenKind::StringLiteral
            | TokenKind::True
            | TokenKind::False => {
                self.start_node(NodeKind::LiteralExpr);
                self.bump();
                self.finish_node();
            }
            TokenKind::LParen => {
                self.start_node(NodeKind::ParenExpr);
                self.bump();
                self.with_struct_literal_allowed(Self::parse_expression);
                self.expect(TokenKind::RParen, "`)`");
                self.finish_node();
            }
            TokenKind::LBracket => {
                self.start_node(NodeKind::ListLiteral);
                self.bump();
                if !self.at(TokenKind::RBracket) {
                    self.with_struct_literal_allowed(Self::parse_expression);
                    while self.eat(TokenKind::Comma).is_some() {
                        if self.at(TokenKind::RBracket) {
                            break;
                        }
                        self.with_struct_literal_allowed(Self::parse_expression);
                    }
                }
                self.expect(TokenKind::RBracket, "`]`");
                self.finish_node();
            }
            TokenKind::LBrace => {
                self.parse_map_literal();
            }
            TokenKind::If => self.parse_if(NodeKind::IfStmt),
            TokenKind::Match => self.parse_match(NodeKind::MatchStmt),
            TokenKind::Identifier => self.parse_identifier_led_expr(),
            _ => {
                // No expression production starts here. The offending
                // token MUST be consumed - every caller in this file
                // reachable through `parse_expression` sits inside a
                // `while` loop bounded only by a closing delimiter or
                // EOF, so a zero-progress fallback here would spin
                // forever on malformed input instead of recovering.
                self.error_unexpected("an expression");
                self.start_node(NodeKind::ErrorNode);
                if !self.at(TokenKind::Eof) {
                    self.bump();
                }
                self.finish_node();
            }
        }
    }

    fn parse_map_literal(&mut self) {
        self.start_node(NodeKind::MapLiteral);
        self.bump(); // "{"
        if !self.at(TokenKind::RBrace) {
            self.parse_map_entry();
            while self.eat(TokenKind::Comma).is_some() {
                if self.at(TokenKind::RBrace) {
                    break;
                }
                self.parse_map_entry();
            }
        }
        self.expect(TokenKind::RBrace, "`}`");
        self.finish_node();
    }

    fn parse_map_entry(&mut self) {
        self.start_node(NodeKind::MapEntry);
        self.with_struct_literal_allowed(Self::parse_expression);
        self.expect(TokenKind::Colon, "`:`");
        self.with_struct_literal_allowed(Self::parse_expression);
        self.finish_node();
    }

    /// Handles every primary form that starts with a bare `identifier`:
    /// a plain reference, `path::variant[(...)|{...}]` enum construction,
    /// or `Name { ... }` struct-literal construction.
    fn parse_identifier_led_expr(&mut self) {
        if self.peek2() == TokenKind::ColonColon {
            self.start_node(NodeKind::PathExpr);
            self.bump(); // type/enum name
            self.bump(); // "::"
            self.expect(TokenKind::Identifier, "a variant name");
            self.finish_node();
        } else {
            self.start_node(NodeKind::IdentExpr);
            self.bump();
            self.finish_node();
        }

        if self.at(TokenKind::LBrace) && !self.no_struct_literal {
            let lhs = self.pop_last_child();
            self.start_node(NodeKind::StructLiteral);
            self.current_node_mut().children.extend(lhs);
            self.bump(); // "{"
            if !self.at(TokenKind::RBrace) {
                self.parse_field_init();
                while self.eat(TokenKind::Comma).is_some() {
                    if self.at(TokenKind::RBrace) {
                        break;
                    }
                    self.parse_field_init();
                }
            }
            self.expect(TokenKind::RBrace, "`}`");
            self.finish_node();
        }
    }

    fn parse_field_init(&mut self) {
        self.start_node(NodeKind::FieldInit);
        self.expect(TokenKind::Identifier, "a field name");
        if self.eat(TokenKind::Colon).is_some() {
            self.with_struct_literal_allowed(Self::parse_expression);
        }
        self.finish_node();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    /// Parses `source`, asserts there were no diagnostics and that the
    /// tree reconstructs `source` exactly (the CST's losslessness
    /// guarantee), and returns the tree for further inspection.
    fn parse_ok(source: &str) -> SyntaxNode {
        let (tree, diagnostics) = parse(source, "test.kyn");
        assert!(
            diagnostics.is_empty(),
            "expected no diagnostics, got: {diagnostics:?}"
        );
        assert_eq!(
            tree.text(source),
            source,
            "CST did not losslessly reconstruct the source"
        );
        tree
    }

    /// Parses `source` and asserts at least one diagnostic was produced,
    /// without panicking - confirming error recovery, not just rejection.
    fn parse_err(source: &str) -> Vec<Diagnostic> {
        let (tree, diagnostics) = parse(source, "test.kyn");
        assert!(
            !diagnostics.is_empty(),
            "expected at least one diagnostic for: {source:?}"
        );
        assert_eq!(
            tree.text(source),
            source,
            "CST did not losslessly reconstruct malformed source either"
        );
        diagnostics
    }

    fn first_decl_kind(tree: &SyntaxNode) -> NodeKind {
        tree.nodes()
            .next()
            .expect("expected at least one top-level declaration")
            .kind
    }

    // ---- §2 Program / imports ----

    #[test]
    fn empty_program() {
        let tree = parse_ok("");
        assert_eq!(tree.kind, NodeKind::Program);
        assert!(tree.nodes().next().is_none());
    }

    #[test]
    fn simple_import() {
        let tree = parse_ok("use math.safe_math;\n");
        assert_eq!(first_decl_kind(&tree), NodeKind::Import);
    }

    #[test]
    fn import_with_identifier_list() {
        let tree = parse_ok("use collections.{List, Map};\n");
        let import = tree.child(NodeKind::Import).unwrap();
        assert!(import.child(NodeKind::IdentifierList).is_some());
    }

    #[test]
    fn import_path_with_multiple_segments() {
        let tree = parse_ok("use a.b.c;\n");
        let path = tree
            .child(NodeKind::Import)
            .unwrap()
            .child(NodeKind::ImportPath)
            .unwrap();
        assert_eq!(
            path.tokens()
                .filter(|t| t.kind == TokenKind::Identifier)
                .count(),
            3
        );
    }

    // ---- §3 Contracts ----

    #[test]
    fn empty_contract() {
        let tree = parse_ok("contract Counter {}\n");
        assert_eq!(first_decl_kind(&tree), NodeKind::ContractDecl);
    }

    #[test]
    fn contract_with_every_member_kind() {
        let source = "\
contract Demo {
    state count: i64 = 0;
    const MAX: i64 = 100;
    error DemoError { Bad }
    event Did(who: address);
    struct Inner { x: i64 }
    enum Status { On, Off }

    public fn init() {
        count = 0;
    }
}
";
        let tree = parse_ok(source);
        let contract = tree.child(NodeKind::ContractDecl).unwrap();
        for kind in [
            NodeKind::StateDecl,
            NodeKind::ConstDecl,
            NodeKind::ErrorDecl,
            NodeKind::EventDecl,
            NodeKind::StructDecl,
            NodeKind::EnumDecl,
            NodeKind::FnDecl,
        ] {
            assert!(
                contract.child(kind).is_some(),
                "missing {kind:?} among contract members"
            );
        }
    }

    // ---- §4 Variables ----

    #[test]
    fn state_decl_without_initializer() {
        let tree = parse_ok("contract C { state buyer: address; }\n");
        let decl = tree
            .child(NodeKind::ContractDecl)
            .unwrap()
            .child(NodeKind::StateDecl)
            .unwrap();
        assert!(decl.child(NodeKind::Type).is_some());
    }

    #[test]
    fn const_decl_requires_initializer() {
        parse_err("contract C { const MAX: i64; }\n");
    }

    #[test]
    fn let_stmt_with_and_without_annotation() {
        parse_ok("contract C { fn f() { let x = 1; let y: i64 = 2; } }\n");
    }

    // ---- §5 Functions ----

    #[test]
    fn fn_visibility_modifiers() {
        let source = "\
contract C {
    public fn a() {}
    internal fn b() {}
    fn c() {}
}
";
        parse_ok(source);
    }

    #[test]
    fn fn_with_params_and_return_type() {
        parse_ok("contract C { fn f(a: i64, b: address) -> bool { return true; } }\n");
    }

    // ---- §6 Types ----

    #[test]
    fn parametric_types() {
        let source = "\
contract C {
    state a: list<i64>;
    state b: map<address, i128>;
    state c: bytes<32>;
    state d: Option<i64>;
    state e: Result<i64, DemoError>;
}
";
        parse_ok(source);
    }

    // ---- §6.4 / §6.5 Enums and errors ----

    #[test]
    fn enum_with_all_variant_shapes() {
        let source = "\
enum Status {
    Pending,
    Approved(address),
    Rejected { reason: string },
}
";
        parse_ok(source);
    }

    #[test]
    fn error_decl_with_no_variants_body() {
        parse_ok("error Empty;\n");
    }

    // ---- §7 Expressions ----

    #[test]
    fn binary_precedence_is_left_associative_and_nests_by_level() {
        // `1 + 2 * 3` must parse as `1 + (2 * 3)`: the outermost node is
        // the lowest-precedence operator (`+`), and its right-hand side
        // is itself a `BinaryExpr` for the higher-precedence `*`.
        let tree = parse_ok("contract C { fn f() { let x = 1 + 2 * 3; } }\n");
        let let_stmt = find_first(&tree, NodeKind::LetStmt).unwrap();
        let outer = find_first(let_stmt, NodeKind::BinaryExpr).unwrap();
        let inner = outer.nodes().find(|n| n.kind == NodeKind::BinaryExpr);
        assert!(
            inner.is_some(),
            "expected `2 * 3` to nest inside the outer `+`"
        );
    }

    #[test]
    fn unary_and_call_and_field_and_method_and_try() {
        let source = "\
contract C {
    fn f() -> Result<i64, E> {
        let a = -x;
        let b = !ok;
        let c = f(1, 2);
        let d = a.field;
        let e = a.method(1);
        let g = risky()?;
        return Ok(g);
    }
}
";
        parse_ok(source);
    }

    #[test]
    fn struct_literal_and_field_shorthand() {
        parse_ok("contract C { fn f() { let x = Listing { seller, price, sold: false }; } }\n");
    }

    #[test]
    fn enum_construction_tuple_and_struct_shapes() {
        let source = "\
contract C {
    fn f() {
        let a = Status::Approved(admin);
        let b = Status::Rejected { reason: \"x\" };
    }
}
";
        parse_ok(source);
    }

    #[test]
    fn list_and_map_literals_including_empty() {
        let source = "\
contract C {
    fn f() {
        let a: list<u64> = [1, 2, 3];
        let b: list<u64> = [];
        let c: map<address, i128> = { alice: 100 };
        let d: map<address, i128> = {};
    }
}
";
        parse_ok(source);
    }

    #[test]
    fn if_as_expression_requires_else() {
        parse_ok("contract C { fn f() { let x = if a { 1 } else { 2 }; } }\n");
    }

    #[test]
    fn bare_identifier_condition_is_not_a_struct_literal() {
        // The exact ambiguity forced by the Escrow canonical example:
        // `released` is a bare `bool` state field, not a struct type.
        let source = "\
contract C {
    state released: bool = false;
    fn f() {
        if released {
            return;
        }
    }
}
";
        let tree = parse_ok(source);
        let if_stmt = find_first(&tree, NodeKind::IfStmt).unwrap();
        assert!(
            find_first(if_stmt, NodeKind::StructLiteral).is_none(),
            "`released {{` must not be parsed as a struct literal in an `if` condition"
        );
        assert!(find_first(if_stmt, NodeKind::Block).is_some());
    }

    #[test]
    fn struct_literal_still_allowed_inside_parens_in_a_condition() {
        let source = "contract C { fn f() { if (Point { x: 1, y: 2 }).x == 1 { return; } } }\n";
        let tree = parse_ok(source);
        let if_stmt = find_first(&tree, NodeKind::IfStmt).unwrap();
        assert!(find_first(if_stmt, NodeKind::StructLiteral).is_some());
    }

    // ---- §8 Statements ----

    #[test]
    fn assign_and_compound_assign_ops() {
        let source = "\
contract C {
    state n: i64 = 0;
    fn f() {
        n = 1;
        n += 1;
        n -= 1;
        n *= 1;
        n /= 1;
        n %= 1;
    }
}
";
        parse_ok(source);
    }

    #[test]
    fn for_and_while_loops() {
        let source = "\
contract C {
    fn f(owners: list<address>) {
        for owner in owners {
            break;
        }
        while true {
            continue;
        }
    }
}
";
        parse_ok(source);
    }

    #[test]
    fn match_statement_and_expression_with_guard() {
        let source = "\
contract C {
    fn f(status: Status, admin: address) -> bool {
        match status {
            Status::Approved(by) if by == admin => return true,
            Status::Approved(_) => return false,
            Status::Rejected { reason } => throw ProcessError::Rejected,
            _ => return false,
        }
    }
}
";
        parse_ok(source);
    }

    #[test]
    fn throw_and_emit_and_auth() {
        let source = "\
contract C {
    event Transfer(from: address, to: address, amount: i128);
    fn f(from: address, to: address, amount: i128) -> Result<bool, TokenError> {
        if amount <= 0 {
            throw TokenError::InvalidAmount;
        }
        auth(from);
        emit Transfer(from, to, amount);
        return Ok(true);
    }
}
";
        parse_ok(source);
    }

    // ---- Error recovery ----

    #[test]
    fn missing_semicolon_recovers_and_parses_the_next_declaration() {
        let source = "contract C { const A: i64 = 1 const B: i64 = 2; }\n";
        let (tree, diagnostics) = parse(source, "test.kyn");
        assert!(!diagnostics.is_empty());
        // Recovery must not devour the rest of the file: the second,
        // well-formed declaration should still be found.
        let contract = tree.child(NodeKind::ContractDecl).unwrap();
        assert_eq!(contract.children_of_kind(NodeKind::ConstDecl).count(), 2);
    }

    #[test]
    fn garbage_top_level_input_does_not_panic() {
        parse_err("@@@ not kyne at all ###\n");
    }

    #[test]
    fn unclosed_brace_does_not_panic() {
        parse_err("contract C { fn f() { let x = 1;\n");
    }

    #[test]
    fn two_syntax_errors_in_one_file_both_reported() {
        // Per COMPILER_ARCHITECTURE.md §5's error-recovery rationale: a
        // single `kyne build` should surface every syntax error it can,
        // not stop after the first.
        let source = "contract C { const A i64 = 1; const B: i64 2; }\n";
        let diagnostics = parse_err(source);
        assert!(
            diagnostics.len() >= 2,
            "expected at least 2 diagnostics, got {}",
            diagnostics.len()
        );
    }

    // ---- Helpers ----

    fn find_first(node: &SyntaxNode, kind: NodeKind) -> Option<&SyntaxNode> {
        if node.kind == kind {
            return Some(node);
        }
        for child in node.nodes() {
            if let Some(found) = find_first(child, kind) {
                return Some(found);
            }
        }
        None
    }
}
