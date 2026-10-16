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
use std::sync::Arc;

use foldhash::HashSet;
use globset::GlobSet;

use crate::DatabaseConfiguration;
use crate::GlobSettings;
use crate::error::DatabaseError;
use crate::exclusion::Exclusion;
use crate::file::File;
use crate::file::FileType;
use crate::git::GitIgnores;
use crate::loader::resolve_file_type;
use crate::matcher::build_glob_set;
use crate::matcher::directory_patterns;
use crate::utils::bytes_to_os_str;
use crate::utils::bytes_to_path;

mod source;

use source::SourcePattern;
use source::canonical_path;

/// Resolves whether a single path belongs to the workspace database and, if so,
/// as which [`FileType`]. Built once from a [`DatabaseConfiguration`]; cheap to
/// query per path.
#[derive(Debug, Clone)]
pub struct WorkspaceMatcher {
    extensions: HashSet<OsString>,
    exclusions: PathExclusions,
    host_sources: Vec<SourcePattern>,
    include_sources: Vec<SourcePattern>,
    patch_sources: Vec<SourcePattern>,
    git_ignores: Option<Arc<GitIgnores>>,
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

        let exclusions = PathExclusions::new(&workspace, &configuration.excludes, configuration.glob)?;

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

        Ok(Self { extensions, exclusions, host_sources, include_sources, patch_sources, git_ignores: None })
    }

    /// Classify source files that Git ignores as vendored, as the loader does.
    #[must_use]
    pub fn with_git_ignores(mut self, ignores: Arc<GitIgnores>) -> Self {
        self.git_ignores = Some(ignores);
        self
    }

    /// Returns the [`FileType`] for `file` if it would be part of the database,
    /// or `None` if the path lies outside every configured root or is excluded.
    #[must_use]
    pub fn classify(&self, file: &Path) -> Option<FileType> {
        let canonical = canonical_path(file);
        let lexical = !file.components().any(|part| part == std::path::Component::ParentDir);

        let specificity = |sources: &[SourcePattern]| {
            let extension_ok = file.extension().is_some_and(|extension| self.extensions.contains(extension));
            let canonical_extension_ok =
                canonical.extension().is_some_and(|extension| self.extensions.contains(extension));

            let requested = lexical.then(|| self.max_specificity(sources, file, extension_ok)).flatten();

            requested.max(self.max_specificity(sources, &canonical, canonical_extension_ok))
        };

        let mut host = specificity(&self.host_sources);
        let mut include = specificity(&self.include_sources);
        let patch = specificity(&self.patch_sources);
        if host.is_none() && include.is_none() && patch.is_none() {
            return None;
        }

        if self.is_excluded(file) || self.is_excluded(&canonical) {
            return None;
        }

        if host.is_some()
            && let Some(ignores) = &self.git_ignores
            && ignores.is_ignored(self.source_path(file, &canonical), false)
        {
            include = include.max(host.take());
        }

        Some(resolve_file_type(host, include, patch))
    }

    /// The path a file opened as `requested` is kept under: the requested path below a
    /// source root, which keeps the loader's symlinked directories, or else `resolved`.
    #[must_use]
    pub fn source_path<'path>(&self, requested: &'path Path, resolved: &'path Path) -> &'path Path {
        if !requested.components().any(|part| part == std::path::Component::ParentDir)
            && self.watch_roots().any(|root| requested.starts_with(root))
            && self.admits(requested)
        {
            requested
        } else {
            resolved
        }
    }

    /// Whether some source loads `path` itself, before symlinks resolve.
    fn admits(&self, path: &Path) -> bool {
        let extension_ok = path.extension().is_some_and(|extension| self.extensions.contains(extension));
        !self.exclusions.matches(path)
            && [&self.host_sources, &self.include_sources, &self.patch_sources]
                .into_iter()
                .any(|sources| self.max_specificity(sources, path, extension_ok).is_some())
    }

    /// `exclusions` on top of these sources, matched the way the loader applies a
    /// command's `excludes` while it walks them.
    ///
    /// # Errors
    ///
    /// Returns a [`DatabaseError`] if a glob fails to compile.
    pub fn exclusions(
        &self,
        exclusions: &[Exclusion<'_>],
        glob: GlobSettings,
    ) -> Result<PathExclusions, DatabaseError> {
        let mut compiled = PathExclusions::new(&self.exclusions.workspace, exclusions, glob)?;
        compiled.sources = [self.host_sources.clone(), self.include_sources.clone(), self.patch_sources.clone()];
        Ok(compiled)
    }

    fn max_specificity(&self, sources: &[SourcePattern], path: &Path, extension_ok: bool) -> Option<usize> {
        sources
            .iter()
            .filter(|source| source.matches(path, extension_ok) && !self.exclusions.prunes(source, path))
            .map(|source| source.specificity)
            .max()
    }

    /// Returns `true` if `path` would be part of the database.
    #[must_use]
    pub fn contains(&self, path: &Path) -> bool {
        self.classify(path).is_some()
    }

    /// Roots for filesystem watchers. The loader walks a root unless a path exclusion covers it.
    pub fn watch_roots(&self) -> impl Iterator<Item = &Path> {
        self.sources().map(|source| source.root.as_path()).filter(|root| !self.exclusions.covers(root))
    }

    /// Whether a directory can contain files from any configured source, as the loader walks them.
    #[must_use]
    pub fn can_contain_files(&self, directory: &Path) -> bool {
        let lexical = !directory.components().any(|part| part == std::path::Component::ParentDir);
        (lexical && self.reaches(directory)) || self.reaches(&canonical_path(directory))
    }

    fn reaches(&self, directory: &Path) -> bool {
        !self.exclusions.covers(directory)
            && self.sources().any(|source| {
                source.root.starts_with(directory)
                    || (directory.starts_with(&source.root) && !self.exclusions.prunes_directory(source, directory))
            })
    }

    fn sources(&self) -> impl Iterator<Item = &SourcePattern> {
        self.host_sources.iter().chain(&self.include_sources).chain(&self.patch_sources)
    }

    /// File extensions accepted below the configured roots.
    pub fn extensions(&self) -> impl Iterator<Item = &OsStr> {
        self.extensions.iter().map(OsString::as_os_str)
    }

    /// Check an absolute source path against the configured exclusions.
    #[must_use]
    pub fn is_excluded(&self, canonical: &Path) -> bool {
        self.exclusions.matches(canonical)
    }

    /// Whether a path exclusion covers `directory`. File globs never exclude a
    /// directory: the loader still walks one that a glob names.
    #[must_use]
    pub fn excludes_directory(&self, directory: &Path) -> bool {
        self.exclusions.covers(directory)
    }
}

/// Compiled [`Exclusion`]s, matched the way the loader skips files.
#[derive(Debug, Clone, Default)]
pub struct PathExclusions {
    workspace: PathBuf,
    globs: GlobSet,
    /// Directories the loader prunes, with everything below them, while it walks a source directory.
    directories: GlobSet,
    paths: Vec<PathBuf>,
    /// The host, include, and patch sources a command's own exclusions apply to.
    sources: [Vec<SourcePattern>; 3],
}

impl PathExclusions {
    fn new(workspace: &Path, exclusions: &[Exclusion<'_>], glob: GlobSettings) -> Result<Self, DatabaseError> {
        let workspace = canonical_path(workspace);
        let patterns = || {
            exclusions.iter().filter_map(|exclusion| match exclusion {
                Exclusion::Pattern(pattern) => Some(pattern.as_ref()),
                Exclusion::Path(_) => None,
            })
        };
        let globs = build_glob_set(patterns(), glob)?;
        let directories = build_glob_set(directory_patterns(patterns()), glob)?;
        let paths = exclusions
            .iter()
            .filter_map(|exclusion| match exclusion {
                Exclusion::Path(path) => {
                    Some(canonical_path(&if path.is_absolute() { path.to_path_buf() } else { workspace.join(path) }))
                }
                Exclusion::Pattern(_) => None,
            })
            .collect();

        Ok(Self { workspace, globs, directories, paths, sources: <[Vec<SourcePattern>; 3]>::default() })
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.globs.is_empty() && self.paths.is_empty()
    }

    /// Check a database file by its path, or by its logical name when it has none.
    #[must_use]
    pub fn excludes_file(&self, file: &File) -> bool {
        !self.is_empty()
            && match &file.path {
                Some(path) => self.is_excluded(path, file.file_type),
                None => self.is_excluded(&self.workspace.join(bytes_to_path(&file.name)), file.file_type),
            }
    }

    /// Check an absolute source path loaded as `file_type`. A command's own exclusions
    /// also skip a file when every source that can supply it in that tier prunes a
    /// directory above it. Vendored files include host files Git ignores.
    #[must_use]
    pub fn is_excluded(&self, canonical: &Path, file_type: FileType) -> bool {
        if self.matches(canonical) {
            return true;
        }

        let (first, second): (&[SourcePattern], &[SourcePattern]) = match file_type {
            FileType::Host => (&self.sources[0], &[]),
            FileType::Vendored => (&self.sources[0], &self.sources[1]),
            FileType::Patch => (&self.sources[2], &[]),
            FileType::Builtin | FileType::External => (&[], &[]),
        };
        let mut reaching = first.iter().chain(second).filter(|source| source.matches(canonical, true)).peekable();
        reaching.peek().is_some() && reaching.all(|source| self.prunes(source, canonical))
    }

    /// Whether a path exclusion covers `path`.
    fn covers(&self, path: &Path) -> bool {
        self.paths.iter().any(|excluded| path.starts_with(excluded))
    }

    fn matches(&self, canonical: &Path) -> bool {
        if !self.globs.is_empty() {
            if self.globs.is_match(canonical) {
                return true;
            }

            if let Ok(relative) = canonical.strip_prefix(&self.workspace)
                && self.globs.is_match(relative)
            {
                return true;
            }
        }

        self.covers(canonical)
    }

    /// Whether walking `source` prunes a directory between its root and the file at `path`.
    fn prunes(&self, source: &SourcePattern, path: &Path) -> bool {
        path.parent().is_some_and(|directory| self.prunes_directory(source, directory))
    }

    /// Whether walking `source` prunes `directory` or a directory between it and the root.
    fn prunes_directory(&self, source: &SourcePattern, directory: &Path) -> bool {
        source.walks()
            && !self.directories.is_empty()
            && directory
                .ancestors()
                .take_while(|directory| *directory != source.root && directory.starts_with(&source.root))
                .any(|directory| {
                    self.directories.is_match(directory)
                        || self.directories.is_match(directory.strip_prefix(&self.workspace).unwrap_or(directory))
                })
    }
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
    fn git_ignored_paths_are_vendored_even_before_they_exist() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        std::fs::write(dir.path().join(".gitignore"), ".claude/worktrees/\n").unwrap();
        touch(&dir, "src/Foo.php");
        let ignores = Arc::new(GitIgnores::discover_with_global(dir.path(), None).unwrap());
        let matcher = matcher(&config(&dir, &[], &[], vec![])).with_git_ignores(ignores);

        assert_eq!(matcher.classify(&dir.path().join("src/Foo.php")), Some(FileType::Host));
        assert_eq!(matcher.classify(&dir.path().join(".claude/worktrees/new/src/Foo.php")), Some(FileType::Vendored));
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
