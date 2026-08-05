//! AST node types, mirroring docs/LANGUAGE_SPEC.md §2's grammar
//! productions one-to-one, per docs/COMPILER_ARCHITECTURE.md §7.
//!
//! Immutable once constructed: every field is a plain, owned value with
//! no interior mutability, and nothing in this crate exposes a `&mut`
//! accessor into an already-built tree.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub imports: Vec<Import>,
    pub items: Vec<TopLevelDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    pub path: Vec<String>,
    /// The `.{a, b}` selective-import list, per LANGUAGE_SPEC.md §12.2.
    /// Empty when the import names no specific identifiers.
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopLevelDecl {
    Contract(ContractDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Error(ErrorDecl),
    Event(EventDecl),
    Const(ConstDecl),
    Fn(FnDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractDecl {
    pub name: String,
    pub members: Vec<ContractMember>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractMember {
    State(StateDecl),
    Const(ConstDecl),
    Error(ErrorDecl),
    Event(EventDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Fn(FnDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateDecl {
    pub name: String,
    pub ty: Type,
    pub init: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstDecl {
    pub name: String,
    pub ty: Type,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorDecl {
    pub name: String,
    pub variants: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventDecl {
    pub name: String,
    pub params: Vec<Param>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDecl {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDecl {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariant {
    pub name: String,
    pub shape: EnumVariantShape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumVariantShape {
    Unit,
    Tuple(Vec<Type>),
    Struct(Vec<Field>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnDecl {
    pub visibility: Visibility,
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Internal,
    /// No modifier - file-private, per LANGUAGE_SPEC.md §5.3.
    Private,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

/// A type reference, per LANGUAGE_SPEC.md §6. This crate does not
/// validate that a `Named` type actually names a known primitive or
/// declared type - per docs/COMPILER_ARCHITECTURE.md §5, the parser (and
/// this lowering step) only enforce grammar shape; legality is
/// `kyne_types`'s job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// A primitive type name or a user-declared `struct`/`enum`/`error`
    /// name.
    Named(String),
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    /// `bytes<N>` - `N`'s literal text, unparsed for the same reason
    /// `Literal::Int` is, per that variant's documentation.
    Bytes(String),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Let(LetStmt),
    Assign(AssignStmt),
    Auth(Expr),
    Emit(EmitStmt),
    Throw(Expr),
    If(IfStmt),
    Match(MatchStmt),
    For(ForStmt),
    While(WhileStmt),
    Return(Option<Expr>),
    Break,
    Continue,
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetStmt {
    pub name: String,
    pub ty: Option<Type>,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignStmt {
    pub target: Expr,
    pub op: AssignOp,
    pub value: Expr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Assign,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitStmt {
    pub event: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_block: Block,
    pub else_branch: Option<ElseBranch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElseBranch {
    If(Box<IfStmt>),
    Block(Block),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchStmt {
    pub subject: Expr,
    pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: MatchArmBody,
}

/// A `match` arm's body. `Return`/`Throw`/`Break`/`Continue` are accepted
/// directly (no trailing `;`) alongside `Expr` and `Block`, per
/// LANGUAGE_SPEC.md §7.7's own worked example - see
/// docs/adr/ADR-0005-parser-implementation.md for why the grammar admits
/// this beyond its formal `MatchArm` production.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchArmBody {
    Block(Block),
    Expr(Expr),
    Return(Option<Expr>),
    Throw(Expr),
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    Wildcard,
    Literal(Literal),
    /// A bare binding identifier, or `_` already normalized to
    /// [`Pattern::Wildcard`].
    Ident(String),
    /// `identifier "(" PatternList ")"` - enum tuple-variant
    /// destructuring. `variant` is `None` when the pattern names a bare
    /// type/constructor with no `::` (not used by any variant shape in
    /// v1, but grammar-permitted).
    Tuple {
        path: Path,
        patterns: Vec<Pattern>,
    },
    /// `identifier "{" FieldPatternList "}"` - enum/struct field
    /// destructuring by name.
    Struct {
        path: Path,
        fields: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForStmt {
    pub var: String,
    pub iterable: Expr,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Block,
}

/// A name reference: a bare identifier, or a `Type::variant` qualified
/// path, per LANGUAGE_SPEC.md §7.5's enum-construction syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Path {
    Ident(String),
    Qualified(String, String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Literal(Literal),
    Path(Path),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Field {
        base: Box<Expr>,
        name: String,
    },
    MethodCall {
        base: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    /// The postfix `?` operator, per LANGUAGE_SPEC.md §7.8.
    Try {
        operand: Box<Expr>,
    },
    StructLiteral {
        path: Path,
        fields: Vec<FieldInit>,
    },
    ListLiteral(Vec<Expr>),
    MapLiteral(Vec<MapEntry>),
    /// `if` used in expression position, per LANGUAGE_SPEC.md §7.6 - the
    /// same [`IfStmt`] shape, since the grammar has no separate `IfExpr`
    /// production (only the position it appears in differs).
    If(Box<IfStmt>),
    /// `match` used in expression position, per LANGUAGE_SPEC.md §7.7 -
    /// the same [`MatchStmt`] shape, for the same reason as `If` above.
    Match(Box<MatchStmt>),
}

/// A struct-literal field initializer. Field-init shorthand (`{ seller }`
/// instead of `{ seller: seller }`) is expanded during lowering, per
/// docs/COMPILER_ARCHITECTURE.md §7 - by the time an AST exists, every
/// `FieldInit` has an explicit value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldInit {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapEntry {
    pub key: Expr,
    pub value: Expr,
}

/// A literal value. Integer literal text is kept as written (underscores
/// and all) rather than eagerly parsed to a fixed-width integer: a
/// lexically valid literal can still be out of range for every Kyne
/// integer type (e.g. a 40-digit decimal), and range-checking against a
/// concrete target type is `kyne_types`'s job, not this crate's - eagerly
/// parsing here would make lowering fallible, violating
/// docs/COMPILER_ARCHITECTURE.md §7's "MUST NOT be able to fail on any
/// CST the parser produced successfully" requirement. String literal
/// escapes, by contrast, are decoded eagerly: `kyne_lexer` already
/// guarantees a successfully-lexed `StringLiteral` token contains only
/// well-formed escapes, so decoding here cannot fail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Int(String),
    Str(String),
    Bool(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}
