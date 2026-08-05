//! Fuzz harness for `kyne_parser`, per issue #19 / `ROADMAP.md` §15.
//!
//! `kyne_parser::parse` MUST never panic on any input - malformed
//! syntax is always a `Diagnostic`, never a crash, per
//! `COMPILER_ARCHITECTURE.md` §5's parser contract. This harness
//! specifically targets the parser's own state machine (bracket/brace
//! matching, statement/expression recovery, the "always consume at
//! least one token" robustness rule already fixed once this session -
//! see `docs/adr/ADR-0005-parser-implementation.md`) rather than the
//! lexer's tokenization, though every input necessarily passes through
//! both.

#[test]
fn fuzz_parser_reports_no_panics() {
    // See lexer_fuzz.rs's harness for why 20_000 - the same rationale
    // applies here (each `parse` call is still well under a
    // millisecond).
    kyne_fuzz::run_fuzz_loop(0x5041_5253_4552_4653, 20_000, |text| {
        let _ = kyne_parser::parse(text, "fuzz.kyn");
    });
}
