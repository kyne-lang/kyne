//! RIR node types - "how is this realized in Rust?", per
//! docs/COMPILER_ARCHITECTURE.md §12.
//!
//! Unlike `kyne_hir` and `kyne_ast`, this tree is shaped like the Rust
//! it will become, not like Kyne source: `address` is already
//! `soroban_sdk::Address`, a `state` read/write is already an explicit
//! storage-API call, `auth(addr)` is already `addr.require_auth()`, and
//! every contract-member function already carries the synthetic `env:
//! Env` parameter Soroban's real calling convention requires but Kyne
//! source never writes. See docs/adr/ADR-0011-rir-implementation.md for
//! the exact Soroban SDK surface this crate targets and why it hasn't
//! been verified against a live `cargo build` (issue #18's job).
//!
//! Per LANGUAGE_SPEC.md §4.0 and MEMORY_MODEL.md §21, this is also
//! where every ownership/borrow/clone decision Kyne's value semantics
//! reserve to the compiler is made. This crate's scope (issue #16,
//! bounded to what Counter and Token require) never needs a value used
//! more than once in a way that would require cloning to satisfy
//! ownership - every RIR value below is passed by simple ownership
//! transfer or Rust's `Copy` semantics for primitives, which is already
//! the correct, idiomatic realization for both examples' actual method
//! bodies.

#[derive(Debug, Clone, PartialEq)]
pub struct RProgram {
    pub contract: RContract,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RContract {
    pub name: String,
    pub state_fields: Vec<RStateField>,
    pub consts: Vec<RConst>,
    pub errors: Vec<RErrorDecl>,
    pub events: Vec<REventDecl>,
    pub structs: Vec<RStructDecl>,
    pub enums: Vec<REnumDecl>,
    pub functions: Vec<RFnDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RStateField {
    pub name: String,
    pub ty: RType,
    /// The storage key this field is realized under - its own name, per
    /// LANGUAGE_SPEC.md §4.3 ("keyed by the field's name within the
    /// contract instance").
    pub key: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RConst {
    pub name: String,
    pub ty: RType,
    pub value: RExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RErrorDecl {
    pub name: String,
    pub variants: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct REventDecl {
    pub name: String,
    pub params: Vec<RParam>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RStructDecl {
    pub name: String,
    pub fields: Vec<RField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RField {
    pub name: String,
    pub ty: RType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct REnumDecl {
    pub name: String,
    pub variants: Vec<REnumVariant>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct REnumVariant {
    pub name: String,
    pub shape: REnumVariantShape,
}

#[derive(Debug, Clone, PartialEq)]
pub enum REnumVariantShape {
    Unit,
    Tuple(Vec<RType>),
    Struct(Vec<RField>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RParam {
    pub name: String,
    pub ty: RType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RFnDecl {
    pub visibility: kyne_ast::Visibility,
    pub name: String,
    /// Kyne source's declared parameters. The synthetic `env: Env`
    /// parameter every Soroban contract function needs is not
    /// represented here - `kyne_codegen` (issue #17) prepends it at
    /// print time, since it is a fixed, unconditional part of every
    /// function's signature and carries no information this stage
    /// would otherwise compute per-function.
    pub params: Vec<RParam>,
    pub return_type: Option<RType>,
    pub body: RBlock,
}

/// Kyne's primitive and intrinsic parametric types, already mapped onto
/// their Soroban SDK Rust equivalents. See
/// docs/adr/ADR-0011-rir-implementation.md for the full mapping table.
#[derive(Debug, Clone, PartialEq)]
pub enum RType {
    Bool,
    I32,
    I64,
    I128,
    U32,
    U64,
    U128,
    /// `soroban_sdk::Address`.
    Address,
    /// `soroban_sdk::Symbol`.
    Symbol,
    /// `soroban_sdk::String`.
    String,
    /// `soroban_sdk::Bytes`.
    Bytes,
    /// `soroban_sdk::BytesN<N>`.
    FixedBytes(u64),
    /// `soroban_sdk::Vec<T>` - distinct from `std::vec::Vec<T>`, per
    /// Soroban's own host-managed collection type.
    Vec(Box<RType>),
    /// `soroban_sdk::Map<K, V>`.
    Map(Box<RType>, Box<RType>),
    Option(Box<RType>),
    Result(Box<RType>, Box<RType>),
    /// A user-declared `struct`/`enum`/`error` name, unchanged.
    Named(String),
    Unit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RBlock {
    pub statements: Vec<RStmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RStmt {
    Let {
        name: String,
        ty: Option<RType>,
        value: RExpr,
    },
    /// A non-`state` assignment - ordinary Rust variable assignment.
    Assign {
        target: RExpr,
        value: RExpr,
    },
    /// A `state` field assignment, realized as an explicit
    /// persistent-storage write. A compound source assignment
    /// (`count += 1`) is already expanded here into its read-modify-
    /// write form (`value` is `count + 1`, fully formed) - `state` has
    /// no notion of a Rust-level place to apply `+=` to directly, since
    /// it is never a real Rust variable at all.
    StorageSet {
        key: String,
        value: RExpr,
    },
    /// `addr.require_auth();` - the direct realization of `auth(addr)`,
    /// per LANGUAGE_SPEC.md §10.1.
    RequireAuth(RExpr),
    Emit {
        event: String,
        args: Vec<RExpr>,
    },
    Match(RMatch),
    For {
        var: String,
        iterable: RExpr,
        body: RBlock,
    },
    While {
        condition: RExpr,
        body: RBlock,
    },
    Expr(RExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RMatch {
    pub subject: RExpr,
    pub arms: Vec<RMatchArm>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RMatchArm {
    pub pattern: kyne_ast::Pattern,
    pub guard: Option<RExpr>,
    pub body: RMatchArmBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RMatchArmBody {
    Block(RBlock),
    Expr(RExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RExpr {
    Literal(kyne_ast::Literal),
    /// A local variable, parameter, or the synthetic `env` reference -
    /// anything that is *not* a `state` field access, which is instead
    /// realized as [`RExpr::StorageGet`]/[`RExpr::StorageSet`] wherever
    /// it appears.
    Var(String),
    /// A `state` field read, realized as an explicit persistent-storage
    /// call - see docs/adr/ADR-0011-rir-implementation.md for the exact
    /// API shape. `default` is the field's own literal initializer,
    /// re-lowered here since Soroban's `get` returns `Option<T>` and a
    /// `state` field with a default must never observably read as
    /// unset, per LANGUAGE_SPEC.md §4.4's definite-assignment guarantee.
    StorageGet {
        key: String,
        default: Option<Box<RExpr>>,
    },
    Path(kyne_ast::Path),
    Binary {
        left: Box<RExpr>,
        op: kyne_ast::BinaryOp,
        right: Box<RExpr>,
    },
    Unary {
        op: kyne_ast::UnaryOp,
        operand: Box<RExpr>,
    },
    Call {
        callee: Box<RExpr>,
        args: Vec<RExpr>,
    },
    Field {
        base: Box<RExpr>,
        name: String,
    },
    MethodCall {
        base: Box<RExpr>,
        method: String,
        args: Vec<RExpr>,
    },
    StructLiteral {
        path: kyne_ast::Path,
        fields: Vec<RFieldInit>,
    },
    ListLiteral(Vec<RExpr>),
    MapLiteral(Vec<RMapEntry>),
    Match(Box<RMatch>),
    Return(Option<Box<RExpr>>),
    Throw(Box<RExpr>),
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RFieldInit {
    pub name: String,
    pub value: RExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RMapEntry {
    pub key: RExpr,
    pub value: RExpr,
}
