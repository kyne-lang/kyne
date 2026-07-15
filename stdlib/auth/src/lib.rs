//! `kyne_stdlib_auth` - a non-consuming, read-only authorization query
//! complementing the `auth(...)` language keyword.
//!
//! Specified by docs/STANDARD_LIBRARY.md section 5 (Authentication
//! Module). Layer 2 (Blockchain). Provides `auth.has(addr)` only - never
//! a `require`/`verify` duplicate of the `auth(...)` keyword itself,
//! which cannot be wrapped by library code at all, per
//! docs/LANGUAGE_SPEC.md section 10.1.
//!
//! No implementation exists yet.
