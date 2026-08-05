# tests/fuzz/

Fuzz-testing harnesses for `kyne_lexer` and `kyne_parser` specifically —
both accept fully untrusted input the moment the Playground exists, per
[`ROADMAP.md` §15](../../docs/ROADMAP.md#15-security-roadmap).

## Status

Implemented (issue #19). `kyne_fuzz` (this directory, a workspace
member crate) provides a small, dependency-free, seedable mutation
engine (`src/lib.rs`) driven against each target by
`tests/lexer_fuzz.rs` and `tests/parser_fuzz.rs`. See
[ADR-0014](../../docs/adr/ADR-0014-fuzz-harness-implementation.md) for
why this approach was chosen over `cargo-fuzz`/libFuzzer (no external
dependency, no nightly toolchain, and it actually runs on Windows,
where `cargo-fuzz` does not).

## Running locally

```sh
cargo test -p kyne_fuzz
```

Runs the crate's own unit tests (the PRNG, mutator, and corpus) plus
both fuzz harnesses — 20,000 mutated inputs each against
`kyne_lexer::lex` and `kyne_parser::parse`, a few seconds each. Every
run is deterministic (seeded), so a failure is always reproducible from
the exact input and seed printed in the panic message.

To fuzz for longer than the default 20,000-iteration baseline, edit the
iteration count in `tests/lexer_fuzz.rs`/`tests/parser_fuzz.rs`
directly — there is no separate configuration file, consistent with
this crate's whole design being a plain `cargo test` target rather than
a special-cased fuzzing tool with its own invocation surface.

## Running in CI

The same command, `cargo test -p kyne_fuzz`, is what CI runs — these
harnesses need no separate CI job, special runner image, or nightly
toolchain, since they are ordinary `#[test]` functions in an ordinary
workspace crate.

## Verified

Both harnesses were run for real during issue #19's own verification:
`fuzz_lexer_reports_no_panics` (20,000 iterations, ~6s) and
`fuzz_parser_reports_no_panics` (20,000 iterations, ~13s) both passed
with **no panics found** in either `kyne_lexer` or `kyne_parser` across
the fuzzed input space. The mutation engine's own
`mutate_never_panics_on_repeated_application` test caught a real bug in
`kyne_fuzz` itself (unbounded buffer growth causing an effective hang)
before either harness ever reached the real lexer/parser — see
ADR-0014 for the full account.

## Future work

A future issue could add real coverage-guided fuzzing (via
`cargo-fuzz`, gated to non-Windows CI runners specifically) as an
additive, opt-in campaign alongside this one, per ADR-0014's own
Consequences section.
