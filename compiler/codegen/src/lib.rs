//! `kyne_codegen` - Rust Code Generation.
//!
//! Specified by docs/COMPILER_ARCHITECTURE.md section 14 (Rust Code
//! Generation). Renders RIR as formatted, idiomatic Rust source text
//! forming a complete, buildable Cargo crate - the final stage before the
//! external Cargo and Soroban toolchains take over, per
//! docs/RUNTIME_MODEL.md section 3.
//!
//! Per section 14: every ownership, borrowing, and SDK-type-mapping
//! decision was already made during RIR lowering ([`kyne_rir`]), so this
//! stage is a nearly mechanical pretty-printer with no remaining semantic
//! decisions of its own - it walks the RIR tree and emits the
//! corresponding Rust syntax. The handful of print-time choices this
//! crate does still have to make (which literal Rust syntax realizes
//! `StorageGet`/`StorageSet`/`StorageMutate`/`RequireAuth`, checked-
//! arithmetic method names, import selection) are documented in
//! docs/adr/ADR-0012-codegen-implementation.md.
//!
//! ## Scope (per issue #17)
//!
//! Scoped to what Counter and Token require - see
//! docs/adr/ADR-0012-codegen-implementation.md for the full list of
//! deliberate, documented gaps (multi-file module mirroring per section
//! 14.1, `Cargo.toml` generation deferred to issue #18, etc.).

mod print;

pub use print::{generate, CodegenError};
