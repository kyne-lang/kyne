//! `kyne_rir` - Rust Intermediate Representation lowering.
//!
//! Specified by docs/COMPILER_ARCHITECTURE.md §12 and docs/MEMORY_MODEL.md
//! §21. This is the stage at which every ownership, allocation, and
//! storage-mapping decision docs/MEMORY_MODEL.md reserves to the
//! compiler's discretion is actually made.
//!
//! ## Scope (per issue #16)
//!
//! Scoped to what Counter and Token require - see
//! docs/adr/ADR-0011-rir-implementation.md for the exact Soroban SDK
//! type and storage-API mapping this crate targets, and an honest note
//! that it has not been verified against a live `cargo build` with a
//! real `soroban-sdk` dependency (issue #18's job).

mod lower;
mod nodes;

pub use lower::lower;
pub use nodes::*;
