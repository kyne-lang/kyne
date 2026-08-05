//! `kyne_resolver` - Name Resolution.
//!
//! Specified by docs/COMPILER_ARCHITECTURE.md §8. Binds every identifier
//! reference in the AST to its declaration, per docs/LANGUAGE_SPEC.md
//! §2.1's order-independence guarantee.
//!
//! ## Known limitations
//!
//! - `kyne_ast` carries no source-span information (a scope decision
//!   made in issue #8), so every diagnostic this crate produces uses a
//!   placeholder span rather than pointing at the exact offending
//!   location. See docs/adr/ADR-0007-resolver-implementation.md.
//! - `use` import resolution is best-effort: this compiler has no
//!   multi-file project loader yet, so an imported name is recorded as
//!   valid on trust rather than verified against another file's real
//!   declarations. See [`symbol_table::SymbolTable::imported_names`]'s
//!   documentation.
//! - A struct/enum-variant pattern's or struct-literal's field *names*
//!   are not validated against the type's real shape here - that
//!   requires type information and is `kyne_types`'s job.

mod resolve;
mod symbol_table;

pub use resolve::{resolve, ResolveResult};
pub use symbol_table::{DeclKind, Scope, SymbolTable};
