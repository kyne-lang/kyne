//! `kyne_optimizer` - behavior-preserving HIR optimization passes.
//!
//! Specified by docs/COMPILER_ARCHITECTURE.md section 13 (Optimization).
//! Every pass in this crate MUST NEVER change observable behavior - this
//! is an absolute precondition, not a target, per section 13's own
//! wording.
//!
//! No implementation exists yet. Deferred to Phase 2, per
//! docs/ROADMAP.md section 7.
