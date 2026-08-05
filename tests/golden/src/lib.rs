//! `kyne_golden` - the shared golden-file fixture methodology, per
//! [`COMPILER_ARCHITECTURE.md` §19](../../../docs/COMPILER_ARCHITECTURE.md#19-testing-strategy):
//! "an input fixture and its expected output fixture are both checked
//! into the repository; the test suite diffs actual output against the
//! checked-in expectation on every run. Any intentional change to
//! output requires updating the golden file in the same commit as the
//! compiler change that caused it."
//!
//! One function, [`assert_golden`], used the same way by every stage's
//! own test suite (lexer, parser, codegen, ...) - see
//! docs/adr/ADR-0015-golden-file-testing.md for why this is a small,
//! dependency-free helper rather than a crate like `insta`, and
//! `compiler/codegen/tests/canonical_examples.rs` /
//! `compiler/lexer/tests/canonical_examples.rs` for real adopters.

use std::env;
use std::fs;
use std::path::Path;

/// Compares `actual` against the checked-in fixture at `fixture_path`
/// (a path relative to the calling crate's own manifest directory -
/// `cargo test` sets the test binary's working directory to the
/// package root, so a literal `"tests/snapshots/counter.rs.snap"`-style
/// string, matching how the fixture is referred to elsewhere in that
/// crate, is exactly what every call site should pass).
///
/// Set the `UPDATE_GOLDEN` environment variable (to any value) to
/// write/overwrite the fixture with `actual` instead of comparing -
/// the update workflow for an intentional output change:
///
/// ```sh
/// UPDATE_GOLDEN=1 cargo test -p <crate>
/// ```
///
/// then review the resulting diff with your version control tool
/// before committing it, per this repository's convention (see
/// `CONTRIBUTING.md`) that a golden-file update MUST land in the same
/// commit as the change that caused it - a fixture is never written
/// automatically as a side effect of an ordinary test run, only when a
/// human explicitly opts in.
///
/// # Panics
///
/// Panics (failing the calling test) if `actual` does not match the
/// checked-in fixture, or if the fixture does not exist and
/// `UPDATE_GOLDEN` was not set.
pub fn assert_golden(fixture_path: &str, actual: &str) {
    let path = Path::new(fixture_path);

    if env::var_os("UPDATE_GOLDEN").is_some() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .unwrap_or_else(|e| panic!("failed to create {parent:?}: {e}"));
        }
        fs::write(path, actual).unwrap_or_else(|e| panic!("failed to write {path:?}: {e}"));
        return;
    }

    let expected = fs::read_to_string(path).unwrap_or_else(|e| {
        panic!(
            "golden fixture {path:?} does not exist or could not be read ({e}) - run with \
             UPDATE_GOLDEN=1 to create it, then review the diff and commit it deliberately"
        )
    });
    assert_eq!(
        actual, expected,
        "golden fixture {path:?} does not match actual output - if this is an intentional \
         change, rerun with UPDATE_GOLDEN=1, review the diff with your version control tool, \
         and commit the updated fixture in the same commit as the change that caused it"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch_path(name: &str) -> String {
        format!(
            "{}/kyne_golden_test_{name}_{}.snap",
            env::temp_dir().display(),
            std::process::id()
        )
    }

    #[test]
    fn matching_content_passes() {
        let path = scratch_path("matching");
        fs::write(&path, "hello\n").unwrap();
        assert_golden(&path, "hello\n");
        let _ = fs::remove_file(&path);
    }

    #[test]
    #[should_panic(expected = "does not match actual output")]
    fn mismatched_content_panics() {
        let path = scratch_path("mismatched");
        fs::write(&path, "hello\n").unwrap();
        assert_golden(&path, "goodbye\n");
        let _ = fs::remove_file(&path);
    }

    #[test]
    #[should_panic(expected = "does not exist")]
    fn missing_fixture_panics_without_update_golden() {
        let path = scratch_path("missing");
        let _ = fs::remove_file(&path);
        assert_golden(&path, "anything\n");
    }

    #[test]
    fn update_golden_writes_the_fixture() {
        let path = scratch_path("update");
        let _ = fs::remove_file(&path);

        env::set_var("UPDATE_GOLDEN", "1");
        assert_golden(&path, "written by the test\n");
        env::remove_var("UPDATE_GOLDEN");

        let written = fs::read_to_string(&path).unwrap();
        assert_eq!(written, "written by the test\n");

        // With UPDATE_GOLDEN unset, the freshly written fixture now
        // matches - this exercises the full round trip a real
        // intentional-change workflow goes through.
        assert_golden(&path, "written by the test\n");
        let _ = fs::remove_file(&path);
    }
}
