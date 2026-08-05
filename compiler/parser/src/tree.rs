//! The raw, lossless syntax tree the parser builds directly, per
//! docs/COMPILER_ARCHITECTURE.md §5's "Creation of the CST": every token
//! from the lexer's stream, including whitespace and comments, is
//! recoverable from this tree, attached to the grammar node it
//! syntactically belongs to.
//!
//! `kyne_cst` wraps this raw tree with the stable, documented `Cst` API
//! consumers outside the parser use, per that crate's own charter.

use kyne_lexer::Token;

/// A grammar production this node represents, mirroring
/// docs/LANGUAGE_SPEC.md §2's EBNF one-to-one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    Program,
    Import,
    ImportPath,
    IdentifierList,

    ContractDecl,
    StateDecl,
    ConstDecl,
    ErrorDecl,
    ErrorVariants,
    EventDecl,
    StructDecl,
    FieldList,
    Field,
    EnumDecl,
    EnumVariant,
    TypeList,
    FnDecl,
    ParamList,
    Param,

    Type,
    ParametricType,

    Block,
    LetStmt,
    AssignStmt,
    AuthStmt,
    EmitStmt,
    ThrowStmt,
    IfStmt,
    MatchStmt,
    MatchArm,
    Pattern,
    PatternList,
    FieldPatternList,
    ForStmt,
    WhileStmt,
    ReturnStmt,
    BreakStmt,
    ContinueStmt,
    ExprStmt,
    ArgList,

    // Expressions.
    BinaryExpr,
    UnaryExpr,
    CallExpr,
    FieldExpr,
    MethodCallExpr,
    TryExpr,
    PathExpr,
    IdentExpr,
    LiteralExpr,
    ParenExpr,
    StructLiteral,
    FieldInit,
    ListLiteral,
    MapLiteral,
    MapEntry,

    /// A span of tokens the parser could not assemble into any valid
    /// production; wraps whatever was skipped during error recovery, per
    /// docs/COMPILER_ARCHITECTURE.md §5's recovery strategy.
    ErrorNode,
}

/// One child of a [`SyntaxNode`]: either a nested node, or a single token
/// (significant or trivia) straight from `kyne_lexer`.
#[derive(Debug, Clone)]
pub enum SyntaxElement {
    Node(SyntaxNode),
    Token(Token),
}

/// A node in the raw syntax tree: a grammar-production kind plus its
/// children in source order. Reprinting every [`Token`] reachable from a
/// node, in order, reproduces that node's exact source text.
#[derive(Debug, Clone)]
pub struct SyntaxNode {
    pub kind: NodeKind,
    pub children: Vec<SyntaxElement>,
}

impl SyntaxNode {
    pub fn new(kind: NodeKind) -> Self {
        SyntaxNode {
            kind,
            children: Vec::new(),
        }
    }

    /// Reconstructs this node's exact source text by concatenating every
    /// token reachable from it, in order - the CST's losslessness
    /// guarantee, per docs/COMPILER_ARCHITECTURE.md §5.
    pub fn text(&self, source: &str) -> String {
        let mut out = String::new();
        self.write_text(source, &mut out);
        out
    }

    fn write_text(&self, source: &str, out: &mut String) {
        for child in &self.children {
            match child {
                SyntaxElement::Node(node) => node.write_text(source, out),
                SyntaxElement::Token(token) => out.push_str(token.text(source)),
            }
        }
    }

    /// Direct child nodes of the given kind, in order. Not recursive.
    pub fn children_of_kind(&self, kind: NodeKind) -> impl Iterator<Item = &SyntaxNode> {
        self.children.iter().filter_map(move |c| match c {
            SyntaxElement::Node(n) if n.kind == kind => Some(n),
            _ => None,
        })
    }

    /// The first direct child node of the given kind, if any.
    pub fn child(&self, kind: NodeKind) -> Option<&SyntaxNode> {
        self.children_of_kind(kind).next()
    }

    /// All direct child nodes, in order.
    pub fn nodes(&self) -> impl Iterator<Item = &SyntaxNode> {
        self.children.iter().filter_map(|c| match c {
            SyntaxElement::Node(n) => Some(n),
            _ => None,
        })
    }

    /// All direct child tokens (excludes tokens nested inside child
    /// nodes), in order.
    pub fn tokens(&self) -> impl Iterator<Item = &Token> {
        self.children.iter().filter_map(|c| match c {
            SyntaxElement::Token(t) => Some(t),
            _ => None,
        })
    }

    /// Every token reachable from this node, recursively, in source order.
    pub fn all_tokens(&self) -> Vec<&Token> {
        let mut out = Vec::new();
        self.collect_tokens(&mut out);
        out
    }

    fn collect_tokens<'a>(&'a self, out: &mut Vec<&'a Token>) {
        for child in &self.children {
            match child {
                SyntaxElement::Node(n) => n.collect_tokens(out),
                SyntaxElement::Token(t) => out.push(t),
            }
        }
    }
}
