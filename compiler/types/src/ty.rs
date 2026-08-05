//! The type representation this crate checks against, per
//! docs/LANGUAGE_SPEC.md §6.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Bool,
    I32,
    I64,
    I128,
    U32,
    U64,
    U128,
    Address,
    Symbol,
    String,
    Bytes,
    /// A function with no `-> Type` returns this, per LANGUAGE_SPEC.md
    /// §5.1. Not a primitive type a Kyne program can name.
    Unit,
    List(Box<Ty>),
    Map(Box<Ty>, Box<Ty>),
    /// `bytes<N>` - `N`'s parsed value. A `Type::Bytes(text)` whose text
    /// fails to parse as `u64` is a type error (`KY0201`), not silently
    /// defaulted.
    FixedBytes(u64),
    Option(Box<Ty>),
    Result(Box<Ty>, Box<Ty>),
    /// A user-declared `struct`, `enum`, or `error` name. This crate does
    /// not itself re-validate that the name refers to one of those three
    /// kinds beyond what `kyne_resolver` already confirmed exists.
    Named(String),
    /// A sentinel produced after an already-reported type error, so
    /// checking the rest of an expression tree doesn't cascade a single
    /// mistake into a flood of unrelated-looking diagnostics. Never a
    /// real program type, and never itself reported as a mismatch
    /// against another type (every comparison involving `Error` is
    /// treated as compatible).
    Error,
}

impl Ty {
    pub fn is_error(&self) -> bool {
        matches!(self, Ty::Error)
    }

    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Ty::I32 | Ty::I64 | Ty::I128 | Ty::U32 | Ty::U64 | Ty::U128
        )
    }

    /// Structural type equality treating [`Ty::Error`] as compatible
    /// with anything, per this type's own documentation - recursively,
    /// so a single already-reported mistake inside a `list<T>`/
    /// `map<K, V>`/`Option<T>`/`Result<T, E>` doesn't cascade into a
    /// second, unrelated-looking mismatch diagnostic every time that
    /// container type is compared against something else afterward.
    pub fn compatible(&self, other: &Ty) -> bool {
        match (self, other) {
            (Ty::Error, _) | (_, Ty::Error) => true,
            (Ty::List(a), Ty::List(b)) => a.compatible(b),
            (Ty::Map(ak, av), Ty::Map(bk, bv)) => ak.compatible(bk) && av.compatible(bv),
            (Ty::Option(a), Ty::Option(b)) => a.compatible(b),
            (Ty::Result(at, ae), Ty::Result(bt, be)) => at.compatible(bt) && ae.compatible(be),
            _ => self == other,
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Ty::Bool => "bool".to_string(),
            Ty::I32 => "i32".to_string(),
            Ty::I64 => "i64".to_string(),
            Ty::I128 => "i128".to_string(),
            Ty::U32 => "u32".to_string(),
            Ty::U64 => "u64".to_string(),
            Ty::U128 => "u128".to_string(),
            Ty::Address => "address".to_string(),
            Ty::Symbol => "symbol".to_string(),
            Ty::String => "string".to_string(),
            Ty::Bytes => "bytes".to_string(),
            Ty::Unit => "unit".to_string(),
            Ty::List(t) => format!("list<{}>", t.describe()),
            Ty::Map(k, v) => format!("map<{}, {}>", k.describe(), v.describe()),
            Ty::FixedBytes(n) => format!("bytes<{n}>"),
            Ty::Option(t) => format!("Option<{}>", t.describe()),
            Ty::Result(t, e) => format!("Result<{}, {}>", t.describe(), e.describe()),
            Ty::Named(n) => n.clone(),
            Ty::Error => "<error type>".to_string(),
        }
    }
}

/// Converts an AST type reference into a checked [`Ty`], per
/// LANGUAGE_SPEC.md §6.1's fixed primitive set and §6.7's five closed
/// intrinsic parametric types. `on_error` is called (and [`Ty::Error`]
/// returned for that position) when `bytes<N>`'s `N` fails to parse -
/// the one way this conversion can itself detect a malformed type.
pub fn lower_type(ty: &kyne_ast::Type, on_error: &mut impl FnMut(String)) -> Ty {
    match ty {
        kyne_ast::Type::Named(name) => match name.as_str() {
            "bool" => Ty::Bool,
            "i32" => Ty::I32,
            "i64" => Ty::I64,
            "i128" => Ty::I128,
            "u32" => Ty::U32,
            "u64" => Ty::U64,
            "u128" => Ty::U128,
            "address" => Ty::Address,
            "symbol" => Ty::Symbol,
            "string" => Ty::String,
            "bytes" => Ty::Bytes,
            other => Ty::Named(other.to_string()),
        },
        kyne_ast::Type::List(inner) => Ty::List(Box::new(lower_type(inner, on_error))),
        kyne_ast::Type::Map(k, v) => Ty::Map(
            Box::new(lower_type(k, on_error)),
            Box::new(lower_type(v, on_error)),
        ),
        kyne_ast::Type::Bytes(text) => match text.parse::<u64>() {
            Ok(n) => Ty::FixedBytes(n),
            Err(_) => {
                on_error(format!("`bytes<{text}>` is not a valid fixed length"));
                Ty::Error
            }
        },
        kyne_ast::Type::Option(inner) => Ty::Option(Box::new(lower_type(inner, on_error))),
        kyne_ast::Type::Result(t, e) => Ty::Result(
            Box::new(lower_type(t, on_error)),
            Box::new(lower_type(e, on_error)),
        ),
    }
}
