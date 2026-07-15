//! `kyne_cli` - the `kyne` binary.
//!
//! Specified by docs/TOOLCHAIN.md section 3 (CLI Philosophy) and section 9
//! (CLI Commands). Kyne exposes exactly one executable; every capability -
//! build, check, fmt, test, doc, and more - is a subcommand of this
//! binary, composed entirely from kyne_driver and the tools/ crates, per
//! docs/COMPILER_ARCHITECTURE.md section 22's library-first architecture.
//!
//! No subcommand is implemented yet. This binary intentionally does
//! nothing beyond existing and compiling, per Phase 1 - Milestone 1's
//! scope: workspace initialization only, no CLI implementation.

fn main() {
    // Intentionally empty. Subcommand dispatch begins in Phase 1 -
    // Milestone 2, per docs/ROADMAP.md section 4.
}
