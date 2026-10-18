//! Single-path membership test: would a given file be part of the database
//! under a [`DatabaseConfiguration`]?
//!
//! Source directories, exact paths, and globs retain the loader's source types,
//! extension rules, and exclusions. Watch roots remain separate from membership.

use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use foldhash::HashSet;
use globset::GlobSet;

use crate::DatabaseConfiguration;
use crate::error::DatabaseError;
use crate::exclusion::Exclusion;
use crate::file::FileType;
use crate::loader::resolve_file_type;
use crate::matcher::build_glob_set;
use crate::utils::bytes_to_os_str;

mod source;

use source::SourcePattern;
use source::canonical_path;

/// Resolves whether a single path belongs to the workspace database and, if so,
/// as which [`FileType`]. Built once from a [`DatabaseConfiguration`]; cheap to
/// query per path.
#[derive(Debug, Clone)]
pub struct WorkspaceMatcher {
    workspace: PathBuf,
    extensions: HashSet<OsString>,
    glob_excludes: GlobSet,
    path_excludes: Vec<PathBuf>,
    host_sources: Vec<SourcePattern>,
    include_sources: Vec<SourcePattern>,
    patch_sources: Vec<SourcePattern>,
}

impl WorkspaceMatcher {
    /// Builds a matcher from a database configuration.
    ///
    /// # Errors
    ///
    /// Returns a [`DatabaseError`] if a source or exclude glob fails to compile.
    pub fn from_configuration(configuration: &DatabaseConfiguration<'_>) -> Result<Self, DatabaseError> {
        let workspace = configuration.workspace.as_ref();
        let workspace = canonical_path(workspace);

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
                    Some(canonical_path(&path))
                }
                Exclusion::Pattern(_) => None,
            })
            .collect();

        let make_sources = |patterns: &[Cow<'_, [u8]>]| -> Result<Vec<SourcePattern>, DatabaseError> {
            patterns.iter().map(|pattern| SourcePattern::new(&workspace, pattern)).collect()
        };

        let host_sources = if configuration.paths.is_empty() {
            vec![SourcePattern::new(&workspace, workspace.to_string_lossy().as_bytes())?]
        } else {
            make_sources(&configuration.paths)?
        };

        let include_sources = make_sources(&configuration.includes)?;
        let patch_sources = make_sources(&configuration.patches)?;

        Ok(Self { workspace, extensions, glob_excludes, path_excludes, host_sources, include_sources, patch_sources })
    }

    /// Returns the [`FileType`] for `file` if it would be part of the database,
    /// or `None` if the path lies outside every configured root or is excluded.
    #[must_use]
    pub fn classify(&self, file: &Path) -> Option<FileType> {
        let canonical = canonical_path(file);

        let specificity = |sources: &[SourcePattern]| {
            let extension_ok = file.extension().is_some_and(|extension| self.extensions.contains(extension));
            let canonical_extension_ok =
                canonical.extension().is_some_and(|extension| self.extensions.contains(extension));

            let lexical = (!file.components().any(|part| part == std::path::Component::ParentDir))
                .then(|| max_specificity(sources, file, extension_ok))
                .flatten();

            lexical.max(max_specificity(sources, &canonical, canonical_extension_ok))
        };

        let host = specificity(&self.host_sources);
        let include = specificity(&self.include_sources);
        let patch = specificity(&self.patch_sources);
        if host.is_none() && include.is_none() && patch.is_none() {
            return None;
        }

        if self.is_excluded(file) || self.is_excluded(&canonical) {
            return None;
        }

        Some(resolve_file_type(host, include, patch))
    }

    /// Returns `true` if `path` would be part of the database.
    #[must_use]
    pub fn contains(&self, path: &Path) -> bool {
        self.classify(path).is_some()
    }

    /// Roots for filesystem watchers, filtered by exclusions rather than file membership.
    pub fn watch_roots(&self) -> impl Iterator<Item = &Path> {
        self.host_sources
            .iter()
            .chain(&self.include_sources)
            .chain(&self.patch_sources)
            .map(|source| source.root.as_path())
            .filter(|root| !self.is_excluded(root))
    }

    /// Whether a directory can contain files from any configured source.
    #[must_use]
    pub fn can_contain_files(&self, directory: &Path) -> bool {
        let canonical = canonical_path(directory);
        !self.is_excluded(directory)
            && !self.is_excluded(&canonical)
            && self.watch_roots().any(|root| {
                directory.starts_with(root)
                    || root.starts_with(directory)
                    || canonical.starts_with(root)
                    || root.starts_with(&canonical)
            })
    }

    /// File extensions accepted below the configured roots.
    pub fn extensions(&self) -> impl Iterator<Item = &OsStr> {
        self.extensions.iter().map(OsString::as_os_str)
    }

    /// Check an absolute source path against the configured exclusions.
    #[must_use]
    pub fn is_excluded(&self, canonical: &Path) -> bool {
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

fn max_specificity(sources: &[SourcePattern], path: &Path, extension_ok: bool) -> Option<usize> {
    sources.iter().filter(|source| source.matches(path, extension_ok)).map(|source| source.specificity).max()
}

#[cfg(test)]
#[path = "membership/tests.rs"]
mod parity_tests;

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
            patches: Vec::new(),
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
    fn patch_path_is_tracked_as_patch() {
        let dir = TempDir::new().unwrap();
        touch(&dir, "stubs/Override.php");
        let mut configuration = config(&dir, &["src"], &[], vec![]);
        configuration.patches = vec![Cow::Owned(b"stubs".to_vec())];
        let matcher = matcher(&configuration);

        assert_eq!(matcher.classify(&dir.path().join("stubs/Override.php")), Some(FileType::Patch));
    }

    #[test]
    fn patch_beats_vendored_at_equal_specificity() {
        // Mirrors the loader's USER_DEFINED > PATCH > BUILT_IN > VENDORED tier order: a file
        // covered by both `includes` and `patches` at the same specificity resolves to Patch.
        let dir = TempDir::new().unwrap();
        touch(&dir, "lib/Lib.php");
        let mut configuration = config(&dir, &["src"], &["lib"], vec![]);
        configuration.patches = vec![Cow::Owned(b"lib".to_vec())];
        let matcher = matcher(&configuration);

        assert_eq!(matcher.classify(&dir.path().join("lib/Lib.php")), Some(FileType::Patch));
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
