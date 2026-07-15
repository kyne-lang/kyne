//! `kyne_security` - the Security Analyzer.
//!
//! Specified by docs/COMPILER_ARCHITECTURE.md section 11 (Security
//! Analyzer). Detects known-dangerous smart-contract patterns (missing
//! auth(), unbounded storage growth, and more) and reports them as
//! non-blocking, KS-prefixed findings, per section 11.2's severity model.
//!
//! No implementation exists yet. Deferred to Phase 2, per
//! docs/ROADMAP.md section 7.
