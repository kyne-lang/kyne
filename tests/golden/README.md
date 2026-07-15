# tests/golden/

Shared golden-file fixture methodology: an input fixture and its expected
output fixture are both checked in; the test suite diffs actual output
against the checked-in expectation on every run, per
[`COMPILER_ARCHITECTURE.md` §19](../../docs/COMPILER_ARCHITECTURE.md#19-testing-strategy).
Any intentional output change requires updating the golden file in the
same commit as the change that caused it.

No fixtures exist yet.
