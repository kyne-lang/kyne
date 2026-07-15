//! `kyne_formatter` - the implementation behind `kyne fmt`.
//!
//! Specified by docs/TOOLCHAIN.md section 10 (Formatter) and
//! ARCHITECTURE.md section 5. Depends only on kyne_lexer, kyne_parser,
//! and kyne_cst - deliberately not kyne_types or any later stage, since
//! formatting a syntactically valid file never requires it to also
//! type-check, per ARCHITECTURE.md section 10.
//!
//! No implementation exists yet.
