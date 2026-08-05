//! `kyne_hir` - High-Level Intermediate Representation construction.
//!
//! Specified by docs/COMPILER_ARCHITECTURE.md §12. Desugars every
//! remaining piece of Kyne-level syntax sugar into a small, canonical
//! core.
//!
//! ## Scope (per issue #15)
//!
//! Scoped to what the Counter and Token canonical examples require, not
//! full desugaring coverage (a Phase 2 follow-up). In practice this
//! crate's two desugarings (`?` expansion, `if`/`match` unification -
//! see `src/nodes.rs`) are general, not example-specific; what's
//! narrower is validation, which only confirms Counter and Token lower
//! successfully rather than exhaustively testing every construct
//! `kyne_ast` can produce.
//!
//! ## Known limitations
//!
//! - HIR nodes carry no inline type annotations, unlike
//!   `COMPILER_ARCHITECTURE.md` §12's "fully-typed core" description.
//!   See docs/adr/ADR-0010-hir-implementation.md.

mod lower;
mod nodes;

pub use lower::lower;
pub use nodes::*;
