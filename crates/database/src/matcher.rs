//! Path exclusion matching with mixed glob + prefix semantics.
//!
//! This module provides two reusable helpers used by the rest of the workspace:
//!
//! - [`build_glob_set`] compiles a list of glob-pattern strings into a
//!   [`globset::GlobSet`] using the project's [`GlobSettings`]. The file
//!   loader ([`crate::loader`]) uses this for `source.excludes` entries that
//!   are pure glob patterns.
//! - [`ExclusionMatcher`] layers on top of `build_glob_set`, splitting an
//!   input list into glob patterns and plain prefixes.
//!
//! # Pattern semantics used by [`ExclusionMatcher`]
//!
//! Each pattern in the input list is classified as either a **glob pattern**
//! or a **plain prefix** based on whether it contains any glob metacharacters
//! (`*`, `?`, `[`, `{`):
//!
//! - Glob patterns (e.g. `src/**/*.php`, `tests/fixtures/*`) are compiled via
//!   the [`globset`] crate using the supplied [`GlobSettings`] and matched
//!   against the full file path.
//! - Plain prefixes (e.g. `src`, `tests/fixtures/`) use the pre-existing
//!   directory-prefix semantics: a pattern `X` matches `X`, `X/anything`, or
//!   (if the pattern already ends with `/`) any path starting with `X/`.
//!
//! This split keeps existing `exclude = ["src/foo"]` configurations working
//! exactly as before while adding full glob support for anyone who needs it.

use globset::GlobBuilder;
use globset::GlobSet;
use globset::GlobSetBuilder;

use crate::GlobSettings;
use crate::error::DatabaseError;

/// Returns `true` if the string contains any character that would make it a
/// glob pattern rather than a literal path.
#[inline]
#[must_use]
pub fn contains_glob_metacharacters<T: AsRef<str>>(pattern: T) -> bool {
    pattern.as_ref().chars().any(|c| matches!(c, '*' | '?' | '[' | '{'))
}

/// Compiles a list of glob patterns into a [`GlobSet`] using the given
/// [`GlobSettings`].
///
/// This is the single place in the workspace that applies `GlobSettings` to
/// `GlobBuilder`; callers should prefer this over rebuilding the same
/// configuration ad-hoc.
///
/// Returns an error if any individual pattern fails to compile.
pub(crate) fn build_glob_set<I, S>(patterns: I, glob_settings: GlobSettings) -> Result<GlobSet, DatabaseError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = GlobBuilder::new(pattern.as_ref())
            .case_insensitive(glob_settings.case_insensitive)
            .literal_separator(glob_settings.literal_separator)
            .backslash_escape(glob_settings.backslash_escape)
            .empty_alternates(glob_settings.empty_alternates)
            .build()?;

        builder.add(glob);
    }

    Ok(builder.build()?)
}

/// Compiled matcher for a list of exclusion patterns.
///
/// Split patterns are stored once at construction time so that per-file
/// matching is cheap. See the module-level documentation for the exact
/// semantics.
#[derive(Debug, Clone)]
pub struct ExclusionMatcher<S: Clone + AsRef<str>> {
    globs: GlobSet,
    prefixes: Vec<S>,
}

impl<S: Clone + AsRef<str>> ExclusionMatcher<S> {
    /// Builds a matcher from a list of patterns and the project's glob
    /// settings.
    ///
    /// # Errors
    ///
    /// Returns a [`DatabaseError::InvalidGlobSet`] if any glob pattern fails to compile.
    #[inline]
    pub fn compile<I>(patterns: I, glob_settings: GlobSettings) -> Result<Self, DatabaseError>
    where
        I: IntoIterator<Item = S>,
    {
        let mut globs = Vec::new();
        let mut prefixes = Vec::new();

        for pattern in patterns {
            if contains_glob_metacharacters(&pattern) {
                globs.push(pattern);
            } else {
                prefixes.push(pattern);
            }
        }

        Ok(Self { globs: build_glob_set(&globs, glob_settings)?, prefixes })
    }

    /// Returns `true` if there are no patterns at all.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.globs.is_empty() && self.prefixes.is_empty()
    }

    /// Returns `true` if `file` matches any of the configured patterns.
    ///
    /// `file` is expected to be a forward-slash-separated path string
    /// relative to whatever root the caller considers meaningful (typically
    /// the workspace root for source excludes, or the logical
    /// `File::name` for per-rule excludes).
    #[inline]
    #[must_use]
    pub fn is_match(&self, file: &str) -> bool {
        if !self.globs.is_empty() && self.globs.is_match(file) {
            return true;
        }

        self.prefixes.iter().any(|pattern| prefix_matches(file, pattern.as_ref()))
    }
}

/// Checks whether `file` is covered by the plain-prefix pattern `pattern`.
///
/// - A pattern ending in `/` must be a path prefix of `file`.
/// - Otherwise the pattern must equal `file` or be a proper directory prefix
///   of it (i.e. `file` starts with `pattern` followed by `/`).
fn prefix_matches(file: &str, pattern: &str) -> bool {
    if pattern.ends_with('/') {
        return file.starts_with(pattern);
    }

    if file == pattern {
        return true;
    }

    let rest = file.strip_prefix(pattern);
    matches!(rest, Some(rest) if rest.starts_with('/'))
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn matcher<'pat>(patterns: &[&'pat str]) -> ExclusionMatcher<&'pat str> {
        ExclusionMatcher::compile(patterns.iter().copied(), GlobSettings::default()).expect("compile")
    }

    #[test]
    fn empty_matcher_matches_nothing() {
        let m = matcher(&[]);
        assert!(m.is_empty());
        assert!(!m.is_match("any/path.php"));
    }

    #[test]
    fn plain_directory_prefix_matches_descendants() {
        let m = matcher(&["src/vendor"]);
        assert!(m.is_match("src/vendor"));
        assert!(m.is_match("src/vendor/foo.php"));
        assert!(m.is_match("src/vendor/deep/nested.php"));
        assert!(!m.is_match("src/vendored.php"));
        assert!(!m.is_match("src/other.php"));
    }

    #[test]
    fn plain_trailing_slash_prefix_is_respected() {
        let m = matcher(&["src/tests/"]);
        assert!(m.is_match("src/tests/foo.php"));
        assert!(!m.is_match("src/tests"));
    }

    #[test]
    fn plain_file_matches_exactly() {
        let m = matcher(&["src/skip.php"]);
        assert!(m.is_match("src/skip.php"));
        assert!(!m.is_match("src/skipped.php"));
    }

    #[test]
    fn glob_double_star_matches_nested() {
        let m = matcher(&["src/**/*.php"]);
        assert!(m.is_match("src/a.php"));
        assert!(m.is_match("src/dir/a.php"));
        assert!(m.is_match("src/a/b/c.php"));
        assert!(!m.is_match("tests/a.php"));
    }

    #[test]
    fn glob_star_matches_flat_and_nested() {
        let m = matcher(&["tests/fixtures/*"]);
        assert!(m.is_match("tests/fixtures/a.php"));
        assert!(m.is_match("tests/fixtures/dir/a.php"));
    }

    #[test]
    fn mixed_patterns_combine_correctly() {
        let m = matcher(&["src/legacy", "tests/**/*Test.php"]);
        assert!(m.is_match("src/legacy/foo.php"));
        assert!(m.is_match("tests/Unit/FooTest.php"));
        assert!(!m.is_match("src/modern/foo.php"));
        assert!(!m.is_match("tests/Unit/Helper.php"));
    }

    #[test]
    fn contains_glob_metacharacters_detects_patterns() {
        assert!(contains_glob_metacharacters("src/**/*.php"));
        assert!(contains_glob_metacharacters("a?.php"));
        assert!(contains_glob_metacharacters("[abc]"));
        assert!(contains_glob_metacharacters("{a,b}"));
        assert!(!contains_glob_metacharacters("src"));
        assert!(!contains_glob_metacharacters("src/foo.php"));
        assert!(!contains_glob_metacharacters("vendor/"));
    }
}
