# tests/golden/

Shared golden-file fixture methodology: an input fixture and its expected
output fixture are both checked in; the test suite diffs actual output
against the checked-in expectation on every run, per
[`COMPILER_ARCHITECTURE.md` §19](../../docs/COMPILER_ARCHITECTURE.md#19-testing-strategy).
Any intentional output change requires updating the golden file in the
same commit as the change that caused it.

## Status

Implemented (issue #20). This directory is a workspace member crate,
`kyne_golden`, providing one function — `assert_golden(fixture_path,
actual)` — used identically by every adopting crate's own test suite.
See [ADR-0015](../../docs/adr/ADR-0015-golden-file-testing.md) for why
this is a small, dependency-free helper rather than a crate like
`insta`.

## Using it in a crate's own tests

```toml
# Cargo.toml
[dev-dependencies]
kyne_golden = { path = "../../tests/golden" }
```

```rust
// tests/my_golden_test.rs
#[test]
fn some_output_matches_its_fixture() {
    let actual = /* whatever your stage produces */;
    kyne_golden::assert_golden("tests/snapshots/some_output.snap", &actual);
}
```

To create or intentionally update a fixture:

```sh
UPDATE_GOLDEN=1 cargo test -p <crate>
```

then review the diff with your version control tool and commit the
updated fixture **in the same commit** as the change that caused it —
see `CONTRIBUTING.md`.

## Real adopters

- `kyne_codegen` — `tests/snapshots/{counter,token}.rs.snap`, the
  generated-Rust-text golden corpus.
- `kyne_lexer` — `tests/snapshots/{basic,error_recovery}.tokens.snap`,
  a golden token-stream corpus including an error-recovery fixture.
