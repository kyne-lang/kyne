# ADR-0014: Fuzz-Harness Framework Choice

**Status:** Accepted
**Date:** 2026-08-05
**Deciders:** Founder
**Type:** Implementation decision — does not amend any constitutional document's normative content.

## Context

Issue #19 required setting up fuzz-testing harnesses for `kyne_lexer`
and `kyne_parser`, per
[`ROADMAP.md` §15](../ROADMAP.md#15-security-roadmap): "introduced in
Phase 1 for the Lexer and Parser specifically — both are classic,
high-value fuzz targets, and fuzzing them early catches crash-class
defects ... before later stages are built on top of a potentially
fragile foundation." The acceptance criteria require choosing a
fuzzing framework and recording that choice here.

## Decision

**A small, dependency-free, `cargo test`-integrated mutation harness
was chosen over `cargo-fuzz`/libFuzzer** (the conventional choice for
Rust fuzzing). Two reasons, both concrete rather than a general
preference:

1. **Every crate in this workspace has zero dependencies outside the
   workspace itself**, per the root `Cargo.toml`'s "no crate outside
   this workspace's `members` list is part of the Kyne build." Every
   crate implemented so far (`kyne_lexer`, `kyne_parser`, `kyne_cst`,
   `kyne_ast`, and further pipeline stages as they land) holds to this
   with no exception — `cargo-fuzz` requires `libfuzzer-sys`, an
   external dependency, and would have been the first break in that
   pattern anywhere in this codebase.
2. **`cargo-fuzz` needs a nightly Rust toolchain and libFuzzer's
   sanitizer-coverage instrumentation, which has poor-to-nonexistent
   support on `x86_64-pc-windows-msvc`** — the actual host this
   repository's compiler is developed on. A harness that cannot run on
   every contributor's machine fails the acceptance criterion "run
   cleanly for a reasonable baseline duration" in the most direct way
   possible: it cannot run at all.

Instead, `tests/fuzz/` is a new workspace member crate (`kyne_fuzz`)
providing:

- **`Rng`**: a tiny, seedable, non-cryptographic PRNG (xorshift64*).
  Every fuzz run is fully reproducible from its seed — a discovered
  crash can always be reproduced by rerunning with the same seed,
  which a true-random source could not guarantee.
- **`mutate`**: a classic byte-level mutator (flip / insert / delete /
  duplicate a slice), capped at 64 KiB (`MAX_LEN`) so repeated
  application cannot grow a buffer without bound — see the real bug
  this caught, below.
- **`seed_corpus`**: the six canonical example contracts plus a handful
  of hand-picked edge cases (empty input, a lone multi-byte UTF-8
  character, unclosed delimiters, an unterminated string, a NUL byte).
- **`run_fuzz_loop`**: drives a target function against many mutated
  corpus entries, wrapping each call in `std::panic::catch_unwind` and
  panicking with the exact failing input if the target ever panics.

`tests/lexer_fuzz.rs` and `tests/parser_fuzz.rs` each run 20,000
mutated inputs (a few seconds per run — a "reasonable baseline
duration" for a harness that runs on every `cargo test`, not a one-off
exhaustive campaign) against `kyne_lexer::lex`/`kyne_parser::parse`
respectively.

## A real bug this approach caught, in itself

`mutate`'s duplicate operation had no size cap in its first draft: it
could roughly double `bytes`' length on every hit. `run_fuzz_loop`
applies one-to-eight mutations per iteration across many iterations, so
repeated doubling reached gigabytes within a few dozen hits — in
practice, this presented as a hang (huge allocations and O(n)
`Vec::insert` shifting), not a clean out-of-memory abort. This was
caught by this crate's own `mutate_never_panics_on_repeated_application`
test hanging past a 20-second timeout during this issue's own
verification — before either fuzz harness had even reached the real
lexer/parser. Fixed by capping growth at `MAX_LEN = 64 * 1024`: above
that size, only the two non-growing operations (flip, delete) are
offered.

This is exactly the class of defect fuzzing itself is meant to catch,
just found one layer earlier than intended (in the fuzzer's own
mutator, rather than in `kyne_lexer`/`kyne_parser`) — left in this ADR
as a concrete illustration of why the harness includes real unit tests
for its own mutation logic, not only the two end-to-end lexer/parser
runs.

## Verification

Both harnesses were run for real: `fuzz_lexer_reports_no_panics`
(20,000 iterations, ~6s) and `fuzz_parser_reports_no_panics` (20,000
iterations, ~13s) both passed with **no panics found** in either
`kyne_lexer` or `kyne_parser` across the fuzzed input space.

## Consequences

- Running the harnesses locally or in CI is exactly `cargo test -p
  kyne_fuzz` — no additional toolchain component, external tool
  installation, or nightly Rust required, unlike `cargo-fuzz`.
- The mutator is a random, non-coverage-guided search, not a
  sophisticated one — it will not find as deep a set of crashes as a
  coverage-guided fuzzer (AFL/libFuzzer-style) would given equal
  wall-clock time. This is an accepted tradeoff for portability and
  zero dependencies; a future issue could add real coverage-guided
  fuzzing (via `cargo-fuzz`, gated to non-Windows CI runners only) as
  an additive, opt-in campaign without removing this one.
- If a future fuzz run does find a crash, the failing input and its
  seed are both printed in the panic message, making it directly
  reproducible.
