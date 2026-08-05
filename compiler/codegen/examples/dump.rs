//! Scratch tool - not part of the crate's public surface. Prints
//! generated Rust for a canonical example to stdout for manual review.
//! Usage: `cargo run -p kyne_codegen --example dump -- counter`

fn main() {
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "counter".to_string());
    let path = format!("../../examples/canonical/{name}.kyn");
    let source = std::fs::read_to_string(&path).expect("read source");
    let (cst, diagnostics) = kyne_cst::Cst::parse(&source, &name);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let program = kyne_ast::lower(&cst);
    let resolved = kyne_resolver::resolve(&program, &name);
    assert!(
        resolved.diagnostics.is_empty(),
        "{:?}",
        resolved.diagnostics
    );
    let typed = kyne_types::check(&program, &name);
    assert!(typed.diagnostics.is_empty(), "{:?}", typed.diagnostics);
    let hir = kyne_hir::lower(&program);
    let rir = kyne_rir::lower(&hir);
    match kyne_codegen::generate(&rir) {
        Ok(rust) => print!("{rust}"),
        Err(e) => eprintln!("ERROR: {e}"),
    }
}
