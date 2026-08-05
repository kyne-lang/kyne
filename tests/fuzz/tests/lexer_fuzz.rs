//! Fuzz harness for `kyne_lexer`, per issue #19 / `ROADMAP.md` §15.
//!
//! `kyne_lexer::lex` MUST never panic on any input, including invalid
//! UTF-8-recovered text, unterminated tokens, or pathological nesting -
//! it is the first stage to see fully untrusted source text once the
//! Playground exists (`ARCHITECTURE.md` §8's "both accept fully
//! untrusted, adversarial-capable input"), and a lexer panic on
//! malformed input would be a denial-of-service-class defect distinct
//! from (and more severe than) an ordinary diagnostic.

#[test]
fn fuzz_lexer_reports_no_panics() {
    // 20_000 iterations keeps this well under a second per run (each
    // `lex` call is microseconds) while still exercising thousands of
    // distinct mutated inputs per `cargo test` invocation - a
    // "reasonable baseline duration" for a harness that runs on every
    // CI build, not a one-off exhaustive campaign.
    kyne_fuzz::run_fuzz_loop(0x004B_594E_454C_4558, 20_000, |text| {
        let _ = kyne_lexer::lex(text);
    });
}
