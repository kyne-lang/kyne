//! HIR node types, per docs/COMPILER_ARCHITECTURE.md §12: "a small,
//! canonical core" with every remaining Kyne-level syntax sugar
//! desugared away. Two normalizations distinguish this tree from
//! `kyne_ast`'s:
//!
//! - The `?` operator is expanded into its explicit match-and-early-
//!   return form, per LANGUAGE_SPEC.md §7.8. See [`lower::lower_expr`]'s
//!   `Try` case for the exact desugaring.
//! - `if`/`match` used in expression position both compile down to the
//!   same single construct, [`HExpr::Match`] - an `if` desugars into a
//!   two-arm match over its condition (`true`/`false`), so HIR has
//!   exactly one conditional-value construct, not two.
//!
//! `return`/`throw`/`break`/`continue` are themselves [`HExpr`] variants
//! here (not statement-only, as in `kyne_ast`), mirroring
//! LANGUAGE_SPEC.md §7.7's own description of `return`/`throw` as
//! "the bottom type... compatible with whatever type the surrounding
//! `match` expression expects" - representing them as ordinary,
//! bottom-typed expressions is what makes normalizing every match arm
//! body down to one shape (`HExpr`) possible at all.
//!
//! Node types do not carry inline type annotations. `COMPILER_ARCHITECTURE.md`
//! §12 describes HIR as fully-typed; retrofitting that requires the same
//! kind of cross-cutting `kyne_ast` change the no-source-spans limitation
//! does (see docs/adr/ADR-0007-resolver-implementation.md) and is out of
//! scope for this issue - see docs/adr/ADR-0010-hir-implementation.md.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HProgram {
    pub items: Vec<HTopLevelDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HTopLevelDecl {
    Contract(HContractDecl),
    Struct(HStructDecl),
    Enum(HEnumDecl),
    Error(HErrorDecl),
    Event(HEventDecl),
    Const(HConstDecl),
    Fn(HFnDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HContractDecl {
    pub name: String,
    pub members: Vec<HContractMember>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HContractMember {
    State(HStateDecl),
    Const(HConstDecl),
    Error(HErrorDecl),
    Event(HEventDecl),
    Struct(HStructDecl),
    Enum(HEnumDecl),
    Fn(HFnDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HStateDecl {
    pub name: String,
    pub ty: kyne_ast::Type,
    pub init: Option<HExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HConstDecl {
    pub name: String,
    pub ty: kyne_ast::Type,
    pub value: HExpr,
}

pub use kyne_ast::{
    EnumVariant as HEnumVariant, ErrorDecl as HErrorDecl, Field as HField, Param as HParam,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HEventDecl {
    pub name: String,
    pub params: Vec<HParam>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HStructDecl {
    pub name: String,
    pub fields: Vec<HField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HEnumDecl {
    pub name: String,
    pub variants: Vec<HEnumVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HFnDecl {
    pub visibility: kyne_ast::Visibility,
    pub name: String,
    pub params: Vec<HParam>,
    pub return_type: Option<kyne_ast::Type>,
    pub body: HBlock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HBlock {
    pub statements: Vec<HStmt>,
}

/// Every statement kind that isn't itself desugared into an expression.
/// `if`/`match`/`return`/`throw`/`break`/`continue` used as *statements*
/// still appear here (as the ordinary top-level effect a statement
/// produces); the same constructs used in *expression* position appear
/// instead as [`HExpr`] variants - both routes share the same
/// [`HMatch`]/[`HExpr::Return`]-shaped machinery underneath, per this
/// module's own documentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HStmt {
    Let {
        name: String,
        ty: Option<kyne_ast::Type>,
        value: HExpr,
    },
    Assign {
        target: HExpr,
        op: kyne_ast::AssignOp,
        value: HExpr,
    },
    Auth(HExpr),
    Emit {
        event: String,
        args: Vec<HExpr>,
    },
    Match(HMatch),
    For {
        var: String,
        iterable: HExpr,
        body: HBlock,
    },
    While {
        condition: HExpr,
        body: HBlock,
    },
    Expr(HExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HMatch {
    pub subject: HExpr,
    pub arms: Vec<HMatchArm>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HMatchArm {
    pub pattern: kyne_ast::Pattern,
    pub guard: Option<HExpr>,
    pub body: HMatchArmBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HMatchArmBody {
    Block(HBlock),
    Expr(HExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HExpr {
    Literal(kyne_ast::Literal),
    Path(kyne_ast::Path),
    Binary {
        left: Box<HExpr>,
        op: kyne_ast::BinaryOp,
        right: Box<HExpr>,
    },
    Unary {
        op: kyne_ast::UnaryOp,
        operand: Box<HExpr>,
    },
    Call {
        callee: Box<HExpr>,
        args: Vec<HExpr>,
    },
    Field {
        base: Box<HExpr>,
        name: String,
    },
    MethodCall {
        base: Box<HExpr>,
        method: String,
        args: Vec<HExpr>,
    },
    StructLiteral {
        path: kyne_ast::Path,
        fields: Vec<HFieldInit>,
    },
    ListLiteral(Vec<HExpr>),
    MapLiteral(Vec<HMapEntry>),
    /// The single canonical conditional-value construct both `if` and
    /// `match` compile down to in expression position.
    Match(Box<HMatch>),
    /// Bottom-typed, per LANGUAGE_SPEC.md §7.7 - compatible with
    /// whatever type the surrounding context expects, since it never
    /// produces a value at all.
    Return(Option<Box<HExpr>>),
    Throw(Box<HExpr>),
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HFieldInit {
    pub name: String,
    pub value: HExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HMapEntry {
    pub key: HExpr,
    pub value: HExpr,
}
