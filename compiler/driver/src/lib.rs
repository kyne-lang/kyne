//! `kyne_driver` - pipeline orchestration.
//!
//! Specified by docs/COMPILER_ARCHITECTURE.md §3 (Compiler Pipeline) and
//! §20 (Repository Architecture). The only crate permitted to depend on
//! every pipeline stage; sequences a `.kyn` project through the pipeline
//! on behalf of `kyne_cli` and every other consumer named in
//! `ARCHITECTURE.md` §5.
//!
//! ## Status
//!
//! Only [`check`] is implemented so far, and only through the CST, per
//! docs/TOOLCHAIN.md §9's `kyne check` entry: "Interaction with the
//! compiler: `kyne_lexer` through `kyne_security`" describes the
//! pipeline's eventual full scope, but `kyne_resolver`, `kyne_types`,
//! `kyne_semantics`, and `kyne_security` do not exist yet (issues
//! #12-#14 and later). `check` runs exactly as much of the pipeline as
//! currently exists - lexing and parsing - and will grow to cover each
//! later stage as its crate is implemented, without changing its public
//! signature.

use kyne_cst::Cst;
use kyne_diagnostics::Diagnostic;

/// Runs `source` through every pipeline stage currently implemented and
/// returns whatever diagnostics they produced. An empty result means
/// `source` passed every check this function currently performs - not
/// yet a full guarantee of well-formedness, since later stages
/// (name resolution, type checking, semantic analysis, security
/// analysis) aren't wired in yet.
pub fn check(source: &str, file: &str) -> Vec<Diagnostic> {
    let (_cst, diagnostics) = Cst::parse(source, file);
    diagnostics
}
