//! `kyne_types` - the Type Checker.
//!
//! Specified by docs/COMPILER_ARCHITECTURE.md §9. Assigns and validates
//! a type for every typed construct in the AST, per
//! docs/LANGUAGE_SPEC.md §6.
//!
//! ## Known limitations
//!
//! - `kyne_ast` carries no source-span information, so every diagnostic
//!   here uses a placeholder span rather than the real offending
//!   location - the same limitation `kyne_resolver` documented in
//!   docs/adr/ADR-0007-resolver-implementation.md.
//! - `kyne_ast::Statement::Expr` does not distinguish a semicolon-less
//!   trailing expression from an ordinary expression statement (both
//!   lower to the same variant). This crate resolves the ambiguity by
//!   treating the last `Statement::Expr` in a block as that block's
//!   value whenever one is needed for an `if`/`match` used in expression
//!   position - the only context where a block's "value" is ever asked
//!   for, so the ambiguity cannot misfire on an ordinary block.
//! - `emit`'s argument list is type-checked per-argument but not
//!   cross-validated against its target event's declared parameter list
//!   (count/order match) - per docs/COMPILER_ARCHITECTURE.md §10, that
//!   specific validation is the Semantic Analyzer's job (`kyne_semantics`,
//!   issue #14), not the Type Checker's.

mod check;
mod env;
mod ty;

pub use check::{check, CheckResult};
pub use env::{EnumVariantTys, FnSig, TypeEnv};
pub use ty::{lower_type, Ty};
