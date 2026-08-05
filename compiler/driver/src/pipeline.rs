//! The `.kyn` source through generated-Rust half of the pipeline, per
//! docs/COMPILER_ARCHITECTURE.md §3 and docs/RUNTIME_MODEL.md §3's
//! "Compiler" stage: "A project that fails any check here never
//! proceeds to a later stage; there is no such thing as a Kyne program
//! that 'mostly' compiles." Each stage below runs in the fixed order
//! §3 specifies, and this function stops at the first stage to report
//! any diagnostic.
//!
//! `kyne_security` (the Security Analyzer) and `kyne_optimizer` have no
//! implementation yet - both are explicitly deferred to Phase 2, per
//! docs/ROADMAP.md §7 - so this pipeline does not yet invoke them; when
//! they are implemented, they slot in here (Security Analyzer between
//! semantic analysis and HIR lowering, Optimizer between HIR lowering
//! and RIR lowering) with no change to this function's shape.

use kyne_diagnostics::Diagnostic;

/// Everything [`compile`] produced on success: the generated Rust source
/// text, and the contract's own name (needed downstream to name the
/// generated crate/package and locate its build artifact).
#[derive(Debug)]
pub struct CompileOutput {
    pub rust_source: String,
    pub contract_name: String,
}

/// Every way [`compile`] can fail: either a real pipeline stage reported
/// one or more diagnostics (per RUNTIME_MODEL.md §3, this is the normal,
/// expected failure mode - an invalid `.kyn` project), or `kyne_codegen`
/// itself defects (per docs/adr/ADR-0012-codegen-implementation.md,
/// this should never happen for RIR `kyne_rir` produces - it is not a
/// user-facing diagnostic).
#[derive(Debug)]
pub enum PipelineError {
    Diagnostics {
        stage: &'static str,
        diagnostics: Vec<Diagnostic>,
    },
    Codegen(kyne_codegen::CodegenError),
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineError::Diagnostics { stage, diagnostics } => {
                write!(
                    f,
                    "{} stage reported {} diagnostic(s)",
                    stage,
                    diagnostics.len()
                )
            }
            PipelineError::Codegen(e) => write!(f, "codegen error: {e}"),
        }
    }
}

impl std::error::Error for PipelineError {}

/// Runs `.kyn` source through the full pipeline (parse, name
/// resolution, type checking, semantic analysis, HIR lowering, RIR
/// lowering, Rust code generation), returning the generated Rust
/// source text on success.
pub fn compile(source: &str, file: &str) -> Result<CompileOutput, PipelineError> {
    let (cst, diagnostics) = kyne_cst::Cst::parse(source, file);
    if !diagnostics.is_empty() {
        return Err(PipelineError::Diagnostics {
            stage: "parse",
            diagnostics,
        });
    }
    let program = kyne_ast::lower(&cst);

    let resolved = kyne_resolver::resolve(&program, file);
    if !resolved.diagnostics.is_empty() {
        return Err(PipelineError::Diagnostics {
            stage: "name-resolution",
            diagnostics: resolved.diagnostics,
        });
    }

    let typed = kyne_types::check(&program, file);
    if !typed.diagnostics.is_empty() {
        return Err(PipelineError::Diagnostics {
            stage: "type-check",
            diagnostics: typed.diagnostics,
        });
    }

    let analyzed = kyne_semantics::analyze(&program, file);
    if !analyzed.diagnostics.is_empty() {
        return Err(PipelineError::Diagnostics {
            stage: "semantic-analysis",
            diagnostics: analyzed.diagnostics,
        });
    }

    let hir = kyne_hir::lower(&program);
    let rir = kyne_rir::lower(&hir);
    let contract_name = rir.contract.name.clone();
    let rust_source = kyne_codegen::generate(&rir).map_err(PipelineError::Codegen)?;

    Ok(CompileOutput {
        rust_source,
        contract_name,
    })
}
