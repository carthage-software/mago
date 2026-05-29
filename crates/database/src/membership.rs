//! Single-path membership test: would a given file be part of the database
//! under a [`DatabaseConfiguration`]?
//!
//! The loader discovers files by walking the configured `paths` / `includes`
//! roots and filtering by extension and `excludes`.
//!
//! The decision mirrors [`crate::loader`] exactly: a path is tracked iff it lies
//! under a configured root, carries a configured extension, and is not excluded.
//! Excludes apply to `includes` roots too, just as they do during a full scan.

use std::borrow::Cow;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use foldhash::HashSet;
use globset::GlobSet;

use crate::DatabaseConfiguration;
use crate::error::DatabaseError;
use crate::exclusion::Exclusion;
use crate::file::FileType;
use crate::loader::calculate_pattern_specificity;
use crate::loader::resolve_file_type;
use crate::matcher::build_glob_set;
use crate::utils::bytes_to_os_str;
use crate::utils::bytes_to_path;

/// Resolves whether a single path belongs to the workspace database and, if so,
/// as which [`FileType`]. Built once from a [`DatabaseConfiguration`]; cheap to
/// query per path.
#[derive(Debug, Clone)]
pub struct WorkspaceMatcher {
    workspace: PathBuf,
    extensions: HashSet<OsString>,
    glob_excludes: GlobSet,
    path_excludes: Vec<PathBuf>,
    host_bases: Vec<(PathBuf, usize)>,
    include_bases: Vec<(PathBuf, usize)>,
}

impl WorkspaceMatcher {
    /// Builds a matcher from a database configuration.
    ///
    /// # Errors
    ///
    /// Returns a [`DatabaseError`] if an exclude glob pattern fails to compile.
    pub fn from_configuration(configuration: &DatabaseConfiguration<'_>) -> Result<Self, DatabaseError> {
        let workspace = configuration.workspace.as_ref();
        let workspace = workspace.canonicalize().unwrap_or_else(|_| workspace.to_path_buf());

        let extensions: HashSet<OsString> = if configuration.extensions.is_empty() {
            std::iter::once(OsString::from("php")).collect()
        } else {
            configuration.extensions.iter().map(|s| bytes_to_os_str(s.as_ref()).into_owned()).collect()
        };

        let glob_patterns: Vec<&str> = configuration
            .excludes
            .iter()
            .filter_map(|ex| match ex {
                Exclusion::Pattern(pattern) => Some(pattern.as_ref()),
                Exclusion::Path(_) => None,
            })
            .collect();

        let glob_excludes = build_glob_set(glob_patterns.iter().copied(), configuration.glob)?;
        let path_excludes: Vec<PathBuf> = configuration
            .excludes
            .iter()
            .filter_map(|ex| match ex {
                Exclusion::Path(path) => {
                    let path =
                        if path.is_absolute() { path.as_ref().to_path_buf() } else { workspace.join(path.as_ref()) };
                    Some(path.canonicalize().unwrap_or(path))
                }
                Exclusion::Pattern(_) => None,
            })
            .collect();

        let make_bases = |patterns: &[Cow<'_, [u8]>]| -> Vec<(PathBuf, usize)> {
            patterns
                .iter()
                .map(|pattern| {
                    let specificity = calculate_pattern_specificity(pattern.as_ref());
                    let base = extract_base_path(pattern.as_ref());
                    let absolute = if base.is_absolute() { base } else { workspace.join(base) };
                    (absolute.canonicalize().unwrap_or(absolute), specificity)
                })
                .collect()
        };

        // An empty `paths` means "scan the whole workspace", matching the loader.
        let host_bases = if configuration.paths.is_empty() {
            vec![(workspace.clone(), calculate_pattern_specificity(workspace.to_string_lossy().as_bytes()))]
        } else {
            make_bases(&configuration.paths)
        };

        let include_bases = make_bases(&configuration.includes);

        Ok(Self { workspace, extensions, glob_excludes, path_excludes, host_bases, include_bases })
    }

    /// Returns the [`FileType`] for `path` if it would be part of the database,
    /// or `None` if the path lies outside every configured root or is excluded.
    #[must_use]
    pub fn classify(&self, path: &Path) -> Option<FileType> {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        let host = max_specificity(&self.host_bases, &canonical);
        let include = max_specificity(&self.include_bases, &canonical);
        if host.is_none() && include.is_none() {
            return None;
        }

        if self.is_excluded(&canonical) {
            return None;
        }

        let extension_ok = canonical.extension().is_some_and(|ext| self.extensions.contains(ext));
        let exact_base = self.host_bases.iter().chain(self.include_bases.iter()).any(|(base, _)| base == &canonical);
        if !extension_ok && !exact_base {
            return None;
        }

        Some(resolve_file_type((host, include)))
    }

    /// Returns `true` if `path` would be part of the database.
    #[must_use]
    pub fn contains(&self, path: &Path) -> bool {
        self.classify(path).is_some()
    }

    fn is_excluded(&self, canonical: &Path) -> bool {
        if !self.glob_excludes.is_empty() {
            if self.glob_excludes.is_match(canonical) {
                return true;
            }

            if let Ok(relative) = canonical.strip_prefix(&self.workspace)
                && self.glob_excludes.is_match(relative)
            {
                return true;
            }
        }

        self.path_excludes.iter().any(|excluded| canonical.starts_with(excluded))
    }
}

fn max_specificity(bases: &[(PathBuf, usize)], path: &Path) -> Option<usize> {
    bases.iter().filter(|(base, _)| path.starts_with(base)).map(|(_, specificity)| *specificity).max()
}

/// Extracts the literal directory portion of a path pattern, dropping any
/// trailing glob segment. Mirrors the loader's view of a configured root: a
/// glob like `src/**/*.php` is rooted at `src`, while a plain `tests/fixtures`
/// is returned unchanged.
fn extract_base_path(pattern: &[u8]) -> PathBuf {
    let is_glob = pattern.iter().any(|&b| matches!(b, b'*' | b'?' | b'[' | b'{'));
    if !is_glob {
        return bytes_to_path(pattern).into_owned();
    }

    let first_glob = pattern.iter().position(|&b| matches!(b, b'*' | b'?' | b'[' | b'{')).unwrap_or(pattern.len());
    let mut end = first_glob;
    while end > 0 && matches!(pattern[end - 1], b'/' | b'\\') {
        end -= 1;
    }

    let base = &pattern[..end];
    if base.is_empty() { PathBuf::from(".") } else { bytes_to_path(base).into_owned() }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::borrow::Cow;

    use tempfile::TempDir;

    use crate::GlobSettings;

    use super::*;

    fn touch(dir: &TempDir, relative: &str) {
        let path = dir.path().join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, "<?php\n").unwrap();
    }

    fn config(
        dir: &TempDir,
        paths: &[&str],
        includes: &[&str],
        excludes: Vec<Exclusion<'static>>,
    ) -> DatabaseConfiguration<'static> {
        DatabaseConfiguration {
            workspace: Cow::Owned(dir.path().to_path_buf()),
            paths: paths.iter().map(|s| Cow::Owned(s.as_bytes().to_vec())).collect(),
            includes: includes.iter().map(|s| Cow::Owned(s.as_bytes().to_vec())).collect(),
            excludes,
            extensions: vec![Cow::Borrowed(b"php")],
            glob: GlobSettings::default(),
        }
    }

    fn matcher(configuration: &DatabaseConfiguration<'_>) -> WorkspaceMatcher {
        WorkspaceMatcher::from_configuration(configuration).unwrap()
    }

    #[test]
    fn file_under_a_configured_path_is_tracked_as_host() {
        let dir = TempDir::new().unwrap();
        touch(&dir, "src/Foo.php");
        let matcher = matcher(&config(&dir, &["src"], &[], vec![]));

        assert_eq!(matcher.classify(&dir.path().join("src/Foo.php")), Some(FileType::Host));
    }

    #[test]
    fn file_outside_every_configured_path_is_not_tracked() {
        let dir = TempDir::new().unwrap();
        touch(&dir, "src/Foo.php");
        touch(&dir, "scripts/build.php");
        let matcher = matcher(&config(&dir, &["src"], &[], vec![]));

        assert_eq!(matcher.classify(&dir.path().join("scripts/build.php")), None);
    }

    #[test]
    fn glob_excluded_file_is_not_tracked() {
        let dir = TempDir::new().unwrap();
        touch(&dir, "src/Foo.php");
        touch(&dir, "src/generated/Bar.php");
        let excludes = vec![Exclusion::Pattern(Cow::Borrowed("src/generated/**"))];
        let matcher = matcher(&config(&dir, &["src"], &[], excludes));

        assert_eq!(matcher.classify(&dir.path().join("src/Foo.php")), Some(FileType::Host));
        assert_eq!(matcher.classify(&dir.path().join("src/generated/Bar.php")), None);
    }

    #[test]
    fn path_excluded_directory_is_not_tracked() {
        let dir = TempDir::new().unwrap();
        touch(&dir, "app/Foo.php");
        touch(&dir, "app/cache/Bar.php");
        let excludes = vec![Exclusion::Path(Cow::Owned(dir.path().join("app/cache")))];
        let matcher = matcher(&config(&dir, &["app"], &[], excludes));

        assert_eq!(matcher.classify(&dir.path().join("app/cache/Bar.php")), None);
    }

    #[test]
    fn included_path_is_tracked_as_vendored() {
        let dir = TempDir::new().unwrap();
        touch(&dir, "vendor/foo/Lib.php");
        let matcher = matcher(&config(&dir, &["src"], &["vendor/foo"], vec![]));

        assert_eq!(matcher.classify(&dir.path().join("vendor/foo/Lib.php")), Some(FileType::Vendored));
    }

    #[test]
    fn excludes_apply_to_included_paths_too() {
        // Matches the loader: an `includes` root does not override `excludes`.
        let dir = TempDir::new().unwrap();
        touch(&dir, "vendor/foo/Lib.php");
        let excludes = vec![Exclusion::Pattern(Cow::Borrowed("vendor/**"))];
        let matcher = matcher(&config(&dir, &["src"], &["vendor/foo"], excludes));

        assert_eq!(matcher.classify(&dir.path().join("vendor/foo/Lib.php")), None);
    }

    #[test]
    fn wrong_extension_is_not_tracked() {
        let dir = TempDir::new().unwrap();
        touch(&dir, "src/notes.txt");
        let matcher = matcher(&config(&dir, &["src"], &[], vec![]));

        assert_eq!(matcher.classify(&dir.path().join("src/notes.txt")), None);
    }

    #[test]
    fn empty_paths_track_the_whole_workspace() {
        let dir = TempDir::new().unwrap();
        touch(&dir, "anywhere/Foo.php");
        let matcher = matcher(&config(&dir, &[], &[], vec![]));

        assert_eq!(matcher.classify(&dir.path().join("anywhere/Foo.php")), Some(FileType::Host));
    }
}
