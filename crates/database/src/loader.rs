//! Database loader for scanning and loading project files.

use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use foldhash::HashMap;
use foldhash::HashSet;
use globset::GlobSet;
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::Database;
use crate::DatabaseConfiguration;
use crate::error::DatabaseError;
use crate::exclusion::Exclusion;
use crate::file::File;
use crate::file::FileId;
use crate::file::FileType;
use crate::matcher::build_glob_set;
use crate::utils::bytes_to_os_str;
use crate::utils::bytes_to_path;
use crate::utils::bytes_to_string_lossy;
use crate::utils::read_file;

/// Holds a file along with the specificity of the pattern that matched it.
///
/// Specificity is used to resolve conflicts when a file matches both `paths` and `includes`.
/// Higher specificity values indicate more specific matches (e.g., exact file paths have higher
/// specificity than directory patterns).
#[derive(Debug)]
struct FileWithSpecificity {
    file: File,
    specificity: usize,
}

/// Builder for loading files into a Database from the filesystem and memory.
pub struct DatabaseLoader<'config> {
    database: Option<Database<'config>>,
    configuration: DatabaseConfiguration<'config>,
    memory_sources: Vec<(&'static [u8], &'static [u8], FileType)>,
    stdin_override: Option<(Cow<'config, [u8]>, Vec<u8>)>,
}

impl<'config> DatabaseLoader<'config> {
    #[inline]
    #[must_use]
    pub fn new(configuration: DatabaseConfiguration<'config>) -> Self {
        Self { configuration, memory_sources: vec![], database: None, stdin_override: None }
    }

    #[inline]
    #[must_use]
    pub fn with_database(mut self, database: Database<'config>) -> Self {
        self.database = Some(database);
        self
    }

    /// When set, the file with this logical name (workspace-relative path) will use the given
    /// content instead of being read from disk. The logical name is used for baseline and reporting.
    ///
    /// `content` is raw bytes: PHP source is binary-safe, so a buffer piped in via `--stdin-input`
    /// may not be valid UTF-8.
    #[inline]
    #[must_use]
    pub fn with_stdin_override(mut self, logical_name: impl AsRef<[u8]>, content: Vec<u8>) -> Self {
        self.stdin_override = Some((Cow::Owned(logical_name.as_ref().to_vec()), content));
        self
    }

    #[inline]
    pub fn add_memory_source(&mut self, name: &'static str, contents: &'static str, file_type: FileType) {
        self.memory_sources.push((name.as_bytes(), contents.as_bytes(), file_type));
    }

    /// Loads files from disk into the database.
    ///
    /// # Errors
    ///
    /// Returns a [`DatabaseError`] if:
    /// - A glob pattern is invalid
    /// - File system operations fail (reading directories, files)
    /// - A file exceeds the maximum supported size
    #[inline]
    pub fn load(mut self) -> Result<Database<'config>, DatabaseError> {
        let mut db = self.database.take().unwrap_or_else(|| Database::new(self.configuration.clone()));

        // Update database configuration to use the loader's configuration
        // (fixes workspace path when merging with prelude database)
        db.configuration = self.configuration.clone();

        let extensions_set: HashSet<OsString> =
            self.configuration.extensions.iter().map(|s| bytes_to_os_str(s.as_ref()).into_owned()).collect();

        let glob_exclude_patterns: Vec<&str> = self
            .configuration
            .excludes
            .iter()
            .filter_map(|ex| match ex {
                Exclusion::Pattern(pat) => Some(pat.as_ref()),
                Exclusion::Path(_) => None,
            })
            .collect();

        let glob_excludes = build_glob_set(glob_exclude_patterns.iter().copied(), self.configuration.glob)?;
        let dir_prune_patterns: Vec<&str> = glob_exclude_patterns
            .iter()
            .filter_map(|pat| {
                let stripped =
                    pat.strip_suffix("/**/*").or_else(|| pat.strip_suffix("/**")).or_else(|| pat.strip_suffix("/*"))?;
                if stripped.is_empty() || stripped == "*" || stripped == "**" {
                    return None;
                }
                Some(stripped)
            })
            .collect();

        let dir_prune_globs = build_glob_set(dir_prune_patterns.iter().copied(), self.configuration.glob)?;

        let path_excludes: HashSet<_> = self
            .configuration
            .excludes
            .iter()
            .filter_map(|ex| match ex {
                Exclusion::Path(p) => Some(p),
                Exclusion::Pattern(_) => None,
            })
            .collect();

        let host_files_with_spec = self.load_paths(
            &self.configuration.paths,
            FileType::Host,
            &extensions_set,
            &glob_excludes,
            &dir_prune_globs,
            &path_excludes,
        )?;

        let vendored_files_with_spec = self.load_paths(
            &self.configuration.includes,
            FileType::Vendored,
            &extensions_set,
            &glob_excludes,
            &dir_prune_globs,
            &path_excludes,
        )?;

        let patch_files_with_spec = self.load_paths(
            &self.configuration.patches,
            FileType::Patch,
            &extensions_set,
            &glob_excludes,
            &dir_prune_globs,
            &path_excludes,
        )?;

        let mut all_files: HashMap<FileId, File> = HashMap::default();
        let mut file_decisions: HashMap<FileId, (FileType, usize)> = HashMap::default();

        // Process host files (from paths)
        for file_with_spec in host_files_with_spec {
            let file_id = file_with_spec.file.id;
            let specificity = file_with_spec.specificity;

            all_files.insert(file_id, file_with_spec.file);
            file_decisions.insert(file_id, (FileType::Host, specificity));
        }

        // When stdin override is set, ensure that the file is in the database
        // (covers new/unsaved files, not on disk). Excluded paths are skipped
        // so that editor integrations using `--stdin-input` honor the same
        // exclude rules as a regular filesystem scan.
        if let Some((name, content)) = &self.stdin_override {
            let virtual_path = self.configuration.workspace.join(bytes_to_path(name.as_ref()).as_ref());
            let virtual_path_canonical = virtual_path.canonicalize().unwrap_or_else(|_| virtual_path.clone());
            let virtual_path_str = virtual_path_canonical.to_string_lossy();

            let matched_glob = !glob_excludes.is_empty()
                && (glob_excludes.is_match(virtual_path_canonical.as_path())
                    || glob_excludes.is_match(bytes_to_path(name.as_ref()).as_ref()));

            let matched_path = path_excludes.iter().any(|excl| {
                let canonical = if Path::new(excl.as_ref()).is_absolute() {
                    excl.as_ref().to_path_buf()
                } else {
                    self.configuration.workspace.join(excl.as_ref())
                };
                let canonical = canonical.canonicalize().unwrap_or(canonical);
                let canonical_str = canonical.to_string_lossy();

                virtual_path_str.starts_with(canonical_str.as_ref())
                    && matches!(virtual_path_str.as_bytes().get(canonical_str.len()), None | Some(&b'/' | &b'\\'))
            });

            if !matched_glob && !matched_path {
                let file = File::ephemeral(Cow::Owned(name.as_ref().to_vec()), Cow::Owned(content.clone()));
                let file_id = file.id;
                if let Entry::Vacant(e) = all_files.entry(file_id) {
                    e.insert(file);

                    file_decisions.insert(file_id, (FileType::Host, usize::MAX));
                }
            }
        }

        for file_with_spec in vendored_files_with_spec {
            let file_id = file_with_spec.file.id;
            let vendored_specificity = file_with_spec.specificity;

            all_files.entry(file_id).or_insert(file_with_spec.file);

            match file_decisions.get(&file_id) {
                Some((FileType::Host, host_specificity)) if vendored_specificity < *host_specificity => {
                    // Keep Host
                }
                _ => {
                    file_decisions.insert(file_id, (FileType::Vendored, vendored_specificity));
                }
            }
        }

        // Patches beat vendored at equal specificity; host beats patches unless patch is strictly more specific.
        for file_with_spec in patch_files_with_spec {
            let file_id = file_with_spec.file.id;
            let patch_specificity = file_with_spec.specificity;

            all_files.entry(file_id).or_insert(file_with_spec.file);

            match file_decisions.get(&file_id) {
                Some((FileType::Host | FileType::Patch, existing_specificity))
                    if patch_specificity <= *existing_specificity =>
                {
                    // Keep existing: it is equally or more specific than the patch.
                }
                _ => {
                    file_decisions.insert(file_id, (FileType::Patch, patch_specificity));
                }
            }
        }

        db.reserve(file_decisions.len() + self.memory_sources.len());

        for (file_id, (final_type, _)) in file_decisions {
            if let Some(mut file) = all_files.remove(&file_id) {
                file.file_type = final_type;
                db.add(file);
            }
        }

        for (name, contents, file_type) in self.memory_sources {
            let file = File::new(Cow::Borrowed(name), file_type, None, Cow::Borrowed(contents));

            db.add(file);
        }

        Ok(db)
    }

    /// Discovers and reads all files from a set of root paths or glob patterns in parallel.
    ///
    /// Supports both:
    /// - Directory paths (e.g., "src", "tests") - recursively walks all files
    /// - Glob patterns (e.g., "src/**/*.php", "tests/Unit/*Test.php") - matches files using glob syntax
    ///
    /// Returns files along with their pattern specificity for conflict resolution.
    fn load_paths(
        &self,
        roots: &[Cow<'config, [u8]>],
        file_type: FileType,
        extensions: &HashSet<OsString>,
        glob_excludes: &GlobSet,
        dir_prune_globs: &GlobSet,
        path_excludes: &HashSet<&Cow<'config, Path>>,
    ) -> Result<Vec<FileWithSpecificity>, DatabaseError> {
        // Canonicalize the workspace once.  All WalkDir roots are canonicalized
        // before traversal so their paths inherit the canonical prefix without
        // any per-file syscalls.
        let canonical_workspace =
            self.configuration.workspace.canonicalize().unwrap_or_else(|_| self.configuration.workspace.to_path_buf());

        // Pre-canonicalize path excludes once as strings.  A plain byte-string
        // prefix check is then sufficient in the parallel section, replacing the
        // per-file canonicalize() + Path::starts_with (Components iteration).
        let canonical_excludes: Vec<String> = path_excludes
            .iter()
            .filter_map(|ex| {
                let p = if Path::new(ex.as_ref()).is_absolute() {
                    ex.as_ref().to_path_buf()
                } else {
                    self.configuration.workspace.join(ex.as_ref())
                };

                p.canonicalize().ok()?.into_os_string().into_string().ok()
            })
            .collect();

        let workspace_relative_str = |path: &Path| -> String {
            let rel = path.strip_prefix(canonical_workspace.as_path()).unwrap_or(path);
            let s = rel.to_string_lossy();
            #[cfg(windows)]
            {
                s.replace('\\', "/")
            }
            #[cfg(not(windows))]
            {
                s.into_owned()
            }
        };

        let mut paths_to_process: Vec<(PathBuf, usize)> = Vec::new();

        for root in roots {
            // Check if this is a glob pattern (contains glob metacharacters).
            // First check if it's an actual file/directory on disk. if so, treat it
            // as a literal path even if the name contains glob metacharacters like `[]`.
            let root_path = bytes_to_path(root.as_ref());
            let resolved_path = if root_path.is_absolute() {
                root_path.as_ref().to_path_buf()
            } else {
                self.configuration.workspace.join(root_path.as_ref())
            };

            let is_glob_pattern = !resolved_path.exists()
                && (root.contains(&b'*') || root.contains(&b'?') || root.contains(&b'[') || root.contains(&b'{'));

            let specificity = Self::calculate_pattern_specificity(root.as_ref());
            if is_glob_pattern {
                // Handle as glob pattern
                let pattern = if root_path.is_absolute() {
                    bytes_to_string_lossy(root.as_ref()).into_owned()
                } else {
                    // Make relative patterns absolute by prepending workspace
                    self.configuration.workspace.join(root_path.as_ref()).to_string_lossy().to_string()
                };

                match glob::glob(&pattern) {
                    Ok(entries) => {
                        for entry in entries {
                            match entry {
                                Ok(path) => {
                                    if path.is_file() {
                                        // Canonicalize so the path shares the same prefix as
                                        // `canonical_workspace` (important on macOS where
                                        // TempDir / glob return /var/… but canonicalize gives
                                        // /private/var/…).  Fall back to the original on error.
                                        let canonical = path.canonicalize().unwrap_or(path);
                                        paths_to_process.push((canonical, specificity));
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to read glob entry: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        return Err(DatabaseError::Glob(e.to_string()));
                    }
                }
            } else {
                let canonical_root = resolved_path.canonicalize().unwrap_or(resolved_path);
                let has_dir_prunes = !dir_prune_globs.is_empty();
                let has_path_prunes = !canonical_excludes.is_empty();
                let walker = WalkDir::new(&canonical_root).follow_links(true).into_iter().filter_entry(|entry| {
                    if entry.depth() == 0 || !entry.file_type().is_dir() {
                        return true;
                    }

                    let path = entry.path();

                    if has_path_prunes
                        && let Some(p) = path.to_str()
                        && canonical_excludes.iter().any(|excl| {
                            p.starts_with(excl.as_str())
                                && matches!(p.as_bytes().get(excl.len()), None | Some(&b'/' | &b'\\'))
                        })
                    {
                        return false;
                    }

                    if has_dir_prunes
                        && (dir_prune_globs.is_match(path) || dir_prune_globs.is_match(workspace_relative_str(path)))
                    {
                        return false;
                    }

                    true
                });

                for entry in walker {
                    match entry {
                        Ok(entry) => {
                            if !entry.file_type().is_dir() {
                                paths_to_process.push((entry.into_path(), specificity));
                            }
                        }
                        Err(err) => {
                            let path = err.path().unwrap_or(canonical_root.as_path()).display();
                            if let Some(ancestor) = err.loop_ancestor() {
                                tracing::warn!(
                                    "Skipping symlink loop at `{path}`: link cycles back to `{}`.",
                                    ancestor.display(),
                                );
                            } else {
                                tracing::warn!("Failed to walk `{path}`: {err}. Entry will be skipped.");
                            }
                        }
                    }
                }
            }
        }

        let has_path_excludes = !canonical_excludes.is_empty();
        let has_glob_excludes = !glob_excludes.is_empty();
        let files: Vec<FileWithSpecificity> = paths_to_process
            .into_par_iter()
            .filter_map(|(path, specificity)| {
                if has_glob_excludes
                    && (glob_excludes.is_match(&path) || glob_excludes.is_match(workspace_relative_str(&path)))
                {
                    return None;
                }

                let ext = path.extension()?;
                if !extensions.contains(ext) {
                    return None;
                }

                if has_path_excludes {
                    let excluded = path.to_str().is_some_and(|s| {
                        canonical_excludes.iter().any(|excl| {
                            s.starts_with(excl.as_str())
                                && matches!(s.as_bytes().get(excl.len()), None | Some(&b'/' | &b'\\'))
                        })
                    });

                    if excluded {
                        return None;
                    }
                }

                let workspace = canonical_workspace.as_path();
                #[cfg(windows)]
                let logical_name =
                    path.strip_prefix(workspace).unwrap_or(path.as_path()).to_string_lossy().replace('\\', "/");
                #[cfg(not(windows))]
                let logical_name =
                    path.strip_prefix(workspace).unwrap_or(path.as_path()).to_string_lossy().into_owned();

                if let Some((override_name, override_content)) = &self.stdin_override
                    && override_name.as_ref() == logical_name.as_bytes()
                {
                    let file = File::new(
                        Cow::Owned(logical_name.into_bytes()),
                        file_type,
                        Some(path.clone()),
                        Cow::Owned(override_content.clone()),
                    );

                    return Some(Ok(FileWithSpecificity { file, specificity }));
                }

                match read_file(workspace, &path, file_type) {
                    Ok(file) => Some(Ok(FileWithSpecificity { file, specificity })),
                    Err(e) => Some(Err(e)),
                }
            })
            .collect::<Result<Vec<FileWithSpecificity>, _>>()?;

        Ok(files)
    }

    /// Calculates how specific a pattern is for a given file path.
    ///
    /// Examples:
    ///
    /// - "src/b.php" matching src/b.php: ~2000 (exact file, 2 components)
    /// - "src/" matching src/b.php: ~100 (directory, 1 component)
    /// - "src" matching src/b.php: ~100 (directory, 1 component)
    fn calculate_pattern_specificity(pattern: &[u8]) -> usize {
        let pattern_path = bytes_to_path(pattern);

        let component_count = pattern_path.components().count();
        let is_glob =
            pattern.contains(&b'*') || pattern.contains(&b'?') || pattern.contains(&b'[') || pattern.contains(&b'{');

        if is_glob {
            let non_wildcard_components = pattern_path
                .components()
                .filter(|c| {
                    let s = c.as_os_str().to_string_lossy();
                    !s.contains('*') && !s.contains('?') && !s.contains('[') && !s.contains('{')
                })
                .count();
            non_wildcard_components * 10
        } else if pattern_path.is_file()
            || pattern_path.extension().is_some()
            || pattern.rsplit(|&b| b == b'.').next().is_some_and(|ext| ext.eq_ignore_ascii_case(b"php"))
        {
            component_count * 1000
        } else {
            component_count * 100
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::DatabaseReader;
    use crate::GlobSettings;
    use std::borrow::Cow;
    use tempfile::TempDir;

    fn create_test_config(temp_dir: &TempDir, paths: Vec<&str>, includes: Vec<&str>) -> DatabaseConfiguration<'static> {
        create_test_config_with_patches(temp_dir, paths, includes, vec![])
    }

    fn create_test_config_with_patches(
        temp_dir: &TempDir,
        paths: Vec<&str>,
        includes: Vec<&str>,
        patches: Vec<&str>,
    ) -> DatabaseConfiguration<'static> {
        // Normalize path separators to platform-specific separators
        let normalize = |s: &str| s.replace('/', std::path::MAIN_SEPARATOR_STR);

        DatabaseConfiguration {
            workspace: Cow::Owned(temp_dir.path().to_path_buf()),
            paths: paths.into_iter().map(|s| Cow::Owned(normalize(s).into_bytes())).collect(),
            includes: includes.into_iter().map(|s| Cow::Owned(normalize(s).into_bytes())).collect(),
            patches: patches.into_iter().map(|s| Cow::Owned(normalize(s).into_bytes())).collect(),
            excludes: vec![],
            extensions: vec![Cow::Borrowed(b"php")],
            glob: GlobSettings::default(),
        }
    }

    /// Returns the file's logical name as a lossy UTF-8 string for assertion matching.
    fn name_str(name: &[u8]) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(name)
    }

    fn create_test_file(temp_dir: &TempDir, relative_path: &str, content: &str) {
        let file_path = temp_dir.path().join(relative_path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(file_path, content).unwrap();
    }

    #[test]
    fn test_specificity_calculation_exact_file() {
        let spec = DatabaseLoader::calculate_pattern_specificity(b"src/b.php");
        assert!(spec >= 2000, "Exact file should have high specificity, got {spec}");
    }

    #[test]
    fn test_specificity_calculation_directory() {
        let spec = DatabaseLoader::calculate_pattern_specificity(b"src/");
        assert!((100..1000).contains(&spec), "Directory should have moderate specificity, got {spec}");
    }

    #[test]
    fn test_specificity_calculation_glob() {
        let spec = DatabaseLoader::calculate_pattern_specificity(b"src/*.php");
        assert!(spec < 100, "Glob pattern should have low specificity, got {spec}");
    }

    #[test]
    fn test_specificity_calculation_deeper_path() {
        let shallow_spec = DatabaseLoader::calculate_pattern_specificity(b"src/");
        let deep_spec = DatabaseLoader::calculate_pattern_specificity(b"src/foo/bar/");
        assert!(deep_spec > shallow_spec, "Deeper path should have higher specificity");
    }

    #[test]
    fn test_exact_file_vs_directory() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(&temp_dir, "src/b.php", "<?php");
        create_test_file(&temp_dir, "src/a.php", "<?php");

        let config = create_test_config(&temp_dir, vec!["src/b.php"], vec!["src/"]);
        let loader = DatabaseLoader::new(config);
        let db = loader.load().unwrap();

        let b_file = db.files().find(|f| name_str(&f.name).contains("b.php")).unwrap();
        assert_eq!(b_file.file_type, FileType::Host, "src/b.php should be Host (exact file beats directory)");

        let a_file = db.files().find(|f| name_str(&f.name).contains("a.php")).unwrap();
        assert_eq!(a_file.file_type, FileType::Vendored, "src/a.php should be Vendored");
    }

    #[test]
    fn test_deeper_vs_shallower_directory() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(&temp_dir, "src/foo/bar.php", "<?php");

        let config = create_test_config(&temp_dir, vec!["src/foo/"], vec!["src/"]);
        let loader = DatabaseLoader::new(config);
        let db = loader.load().unwrap();

        let file = db.files().find(|f| name_str(&f.name).contains("bar.php")).unwrap();
        assert_eq!(file.file_type, FileType::Host, "Deeper directory pattern should win");
    }

    #[test]
    fn test_exact_file_vs_glob() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(&temp_dir, "src/b.php", "<?php");

        let config = create_test_config(&temp_dir, vec!["src/b.php"], vec!["src/*.php"]);
        let loader = DatabaseLoader::new(config);
        let db = loader.load().unwrap();

        let file = db.files().find(|f| name_str(&f.name).contains("b.php")).unwrap();
        assert_eq!(file.file_type, FileType::Host, "Exact file should beat glob pattern");
    }

    #[test]
    fn test_equal_specificity_includes_wins() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(&temp_dir, "src/a.php", "<?php");

        let config = create_test_config(&temp_dir, vec!["src/"], vec!["src/"]);
        let loader = DatabaseLoader::new(config);
        let db = loader.load().unwrap();

        let file = db.files().find(|f| name_str(&f.name).contains("a.php")).unwrap();
        assert_eq!(file.file_type, FileType::Vendored, "Equal specificity: includes should win");
    }

    #[test]
    fn test_complex_scenario_from_bug_report() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(&temp_dir, "src/a.php", "<?php");
        create_test_file(&temp_dir, "src/b.php", "<?php");
        create_test_file(&temp_dir, "src/c/d.php", "<?php");
        create_test_file(&temp_dir, "src/c/e.php", "<?php");
        create_test_file(&temp_dir, "vendor/lib1.php", "<?php");
        create_test_file(&temp_dir, "vendor/lib2.php", "<?php");

        let config = create_test_config(&temp_dir, vec!["src/b.php"], vec!["vendor", "src/c", "src/"]);
        let loader = DatabaseLoader::new(config);
        let db = loader.load().unwrap();

        let b_file = db
            .files()
            .find(|f| name_str(&f.name).contains("src/b.php") || name_str(&f.name).ends_with("b.php"))
            .unwrap();
        assert_eq!(b_file.file_type, FileType::Host, "src/b.php should be Host in bug scenario");

        let d_file = db.files().find(|f| name_str(&f.name).contains("d.php")).unwrap();
        assert_eq!(d_file.file_type, FileType::Vendored, "src/c/d.php should be Vendored");

        let lib_file = db.files().find(|f| name_str(&f.name).contains("lib1.php")).unwrap();
        assert_eq!(lib_file.file_type, FileType::Vendored, "vendor/lib1.php should be Vendored");
    }

    #[test]
    fn test_files_only_in_paths() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(&temp_dir, "src/a.php", "<?php");

        let config = create_test_config(&temp_dir, vec!["src/"], vec![]);
        let loader = DatabaseLoader::new(config);
        let db = loader.load().unwrap();

        let file = db.files().find(|f| name_str(&f.name).contains("a.php")).unwrap();
        assert_eq!(file.file_type, FileType::Host, "File only in paths should be Host");
    }

    #[test]
    fn test_files_only_in_includes() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(&temp_dir, "vendor/lib.php", "<?php");

        let config = create_test_config(&temp_dir, vec![], vec!["vendor/"]);
        let loader = DatabaseLoader::new(config);
        let db = loader.load().unwrap();

        let file = db.files().find(|f| name_str(&f.name).contains("lib.php")).unwrap();
        assert_eq!(file.file_type, FileType::Vendored, "File only in includes should be Vendored");
    }

    #[test]
    fn test_stdin_override_replaces_file_content() {
        let temp_dir = TempDir::new().unwrap();
        create_test_file(&temp_dir, "src/foo.php", "<?php\n// on disk");

        let config = create_test_config(&temp_dir, vec!["src/"], vec![]);
        let loader = DatabaseLoader::new(config).with_stdin_override("src/foo.php", b"<?php\n// from stdin".to_vec());
        let db = loader.load().unwrap();

        let file = db.files().find(|f| name_str(&f.name).contains("foo.php")).unwrap();
        assert_eq!(
            file.contents.as_ref(),
            b"<?php\n// from stdin",
            "stdin override content should be used instead of disk"
        );
    }

    #[test]
    fn test_glob_excludes_match_workspace_relative_paths() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(&temp_dir, "src/Absences/Foo/Foo.php", "<?php");
        create_test_file(&temp_dir, "src/Absences/Test/Faker/Provider/AbsencesProvider.php", "<?php");
        create_test_file(&temp_dir, "src/Calendar/Test/Helper.php", "<?php");

        let mut config = create_test_config(&temp_dir, vec!["src"], vec![]);
        config.excludes = vec![Exclusion::Pattern(Cow::Borrowed("src/*/Test/**"))];

        let loader = DatabaseLoader::new(config);
        let db = loader.load().unwrap();

        let names: Vec<String> = db.files().map(|f| name_str(&f.name).into_owned()).collect();
        assert!(names.iter().any(|n| n.ends_with("src/Absences/Foo/Foo.php")), "non-Test file should be loaded");
        assert!(
            !names.iter().any(|n| n.contains("src/Absences/Test/")),
            "files under src/*/Test/** should be excluded, got {names:?}"
        );
        assert!(
            !names.iter().any(|n| n.contains("src/Calendar/Test/")),
            "files under src/*/Test/** should be excluded, got {names:?}"
        );
    }

    #[test]
    fn test_glob_excludes_match_legacy_absolute_prefix_patterns() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(&temp_dir, "packages/foo/src/main.php", "<?php");
        create_test_file(&temp_dir, "packages/foo/vendor/lib.php", "<?php");

        let mut config = create_test_config(&temp_dir, vec!["packages"], vec![]);
        config.excludes = vec![Exclusion::Pattern(Cow::Borrowed("*/packages/**/vendor/*"))];

        let loader = DatabaseLoader::new(config);
        let db = loader.load().unwrap();

        let names: Vec<String> = db.files().map(|f| name_str(&f.name).into_owned()).collect();
        assert!(names.iter().any(|n| n.ends_with("packages/foo/src/main.php")));
        assert!(
            !names.iter().any(|n| n.contains("/vendor/")),
            "legacy `*/packages/**/vendor/*` style should still exclude vendor files, got {names:?}"
        );
    }

    #[test]
    fn test_glob_dir_prune_skips_relative_directories() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(&temp_dir, "vendor/slevomat/coding-standard/main.php", "<?php");
        create_test_file(&temp_dir, "vendor/slevomat/coding-standard/tests/Sniffs/Foo.php", "<?php");
        create_test_file(&temp_dir, "vendor/another/lib.php", "<?php");

        let mut config = create_test_config(&temp_dir, vec![], vec!["vendor"]);
        config.excludes = vec![Exclusion::Pattern(Cow::Borrowed("vendor/**/tests/**"))];

        let loader = DatabaseLoader::new(config);
        let db = loader.load().unwrap();

        let names: Vec<String> = db.files().map(|f| name_str(&f.name).into_owned()).collect();
        assert!(names.iter().any(|n| n.ends_with("vendor/slevomat/coding-standard/main.php")));
        assert!(names.iter().any(|n| n.ends_with("vendor/another/lib.php")));
        assert!(
            !names.iter().any(|n| n.contains("/tests/")),
            "files under vendor/**/tests/** should be pruned, got {names:?}"
        );
    }

    #[test]
    fn test_stdin_override_adds_file_when_not_on_disk() {
        let temp_dir = TempDir::new().unwrap();
        create_test_file(&temp_dir, "src/.gitkeep", "");

        let config = create_test_config(&temp_dir, vec!["src/"], vec![]);
        let loader =
            DatabaseLoader::new(config).with_stdin_override("src/unsaved.php", b"<?php\n// unsaved buffer".to_vec());
        let db = loader.load().unwrap();

        let file = db.files().find(|f| name_str(&f.name).contains("unsaved.php")).unwrap();
        assert_eq!(file.file_type, FileType::Host);
        assert_eq!(file.contents.as_ref(), b"<?php\n// unsaved buffer");
    }

    #[test]
    fn test_stdin_override_accepts_non_utf8_content() {
        let temp_dir = TempDir::new().unwrap();
        create_test_file(&temp_dir, "src/.gitkeep", "");

        let config = create_test_config(&temp_dir, vec!["src/"], vec![]);
        // PHP identifiers are binary-safe, so a buffer piped in via `--stdin-input` may not
        // be valid UTF-8. The loaded file must carry those bytes through verbatim.
        let content = b"<?php\n\nfunction f\xC9\xFF(): void {}\n".to_vec();
        assert!(std::str::from_utf8(&content).is_err(), "test buffer must contain non-UTF-8 bytes");

        let loader = DatabaseLoader::new(config).with_stdin_override("src/buffer.php", content.clone());
        let db = loader.load().unwrap();

        let file = db.files().find(|f| name_str(&f.name).contains("buffer.php")).unwrap();
        assert_eq!(file.contents.as_ref(), content.as_slice());
    }

    #[cfg(unix)]
    #[test]
    fn test_symlinked_file_under_include_is_loaded() {
        let temp_dir = TempDir::new().unwrap();
        let external = TempDir::new().unwrap();

        create_test_file(&external, "Bar.php", "<?php class Bar {}\n");
        std::fs::create_dir_all(temp_dir.path().join("vendor")).unwrap();
        std::os::unix::fs::symlink(external.path().join("Bar.php"), temp_dir.path().join("vendor/Bar.php")).unwrap();

        let config = create_test_config(&temp_dir, vec![], vec!["vendor/"]);
        let db = DatabaseLoader::new(config).load().unwrap();

        let bar = db.files().find(|f| name_str(&f.name).contains("Bar.php"));
        assert!(bar.is_some(), "symlinked Bar.php should be loaded via include = ['vendor/']");
    }

    #[cfg(unix)]
    #[test]
    fn test_symlinked_directory_under_include_is_descended() {
        let temp_dir = TempDir::new().unwrap();
        let external = TempDir::new().unwrap();

        create_test_file(&external, "src/Foo.php", "<?php class Foo {}\n");
        create_test_file(&external, "src/Bar.php", "<?php class Bar {}\n");

        std::fs::create_dir_all(temp_dir.path().join("vendor")).unwrap();
        std::os::unix::fs::symlink(external.path(), temp_dir.path().join("vendor/example-package")).unwrap();

        let config = create_test_config(&temp_dir, vec![], vec!["vendor/"]);
        let db = DatabaseLoader::new(config).load().unwrap();

        assert!(db.files().any(|f| name_str(&f.name).contains("Foo.php")), "Foo.php inside symlinked dir not found");
        assert!(db.files().any(|f| name_str(&f.name).contains("Bar.php")), "Bar.php inside symlinked dir not found");
    }

    #[cfg(unix)]
    #[test]
    fn test_symlink_cycle_is_warned_and_skipped() {
        let temp_dir = TempDir::new().unwrap();
        create_test_file(&temp_dir, "src/Real.php", "<?php class Real {}\n");
        std::os::unix::fs::symlink(temp_dir.path().join("src"), temp_dir.path().join("src/loop")).unwrap();

        let config = create_test_config(&temp_dir, vec![], vec!["src/"]);
        let db = DatabaseLoader::new(config).load().expect("symlink cycle should not abort the load");

        assert!(
            db.files().any(|f| name_str(&f.name).contains("Real.php")),
            "Real.php still reachable despite the loop"
        );
    }

    #[test]
    fn test_patch_beats_vendored_at_equal_specificity() {
        // A file covered by both patches and includes at the same directory-level specificity
        // should be classified as Patch, not Vendored.
        let temp_dir = TempDir::new().unwrap();
        create_test_file(&temp_dir, "lib/Foo.php", "<?php");

        let config = create_test_config_with_patches(&temp_dir, vec![], vec!["lib/"], vec!["lib/"]);
        let db = DatabaseLoader::new(config).load().unwrap();

        let file = db.files().find(|f| String::from_utf8_lossy(&f.name).contains("Foo.php")).unwrap();
        assert_eq!(file.file_type, FileType::Patch, "patch should beat vendored at equal specificity");
    }

    #[test]
    fn test_host_beats_patch_at_equal_specificity() {
        // When a file is covered by both paths and patches at the same directory-level specificity,
        // the host (paths) classification wins.  Patches only override host when strictly more specific.
        let temp_dir = TempDir::new().unwrap();
        create_test_file(&temp_dir, "src/Foo.php", "<?php");

        let config = create_test_config_with_patches(&temp_dir, vec!["src/"], vec![], vec!["src/"]);
        let db = DatabaseLoader::new(config).load().unwrap();

        let file = db.files().find(|f| String::from_utf8_lossy(&f.name).contains("Foo.php")).unwrap();
        assert_eq!(file.file_type, FileType::Host, "host should beat patch at equal specificity");
    }

    #[test]
    fn test_patch_beats_host_when_strictly_more_specific() {
        // An exact-file patch pattern has higher specificity than a directory paths pattern,
        // so the patch wins and the file is treated as Patch rather than Host.
        let temp_dir = TempDir::new().unwrap();
        create_test_file(&temp_dir, "src/Foo.php", "<?php");
        create_test_file(&temp_dir, "src/Bar.php", "<?php");

        // Patch covers only Foo.php exactly; paths covers the whole directory.
        let config = create_test_config_with_patches(&temp_dir, vec!["src/"], vec![], vec!["src/Foo.php"]);
        let db = DatabaseLoader::new(config).load().unwrap();

        let foo = db.files().find(|f| String::from_utf8_lossy(&f.name).contains("Foo.php")).unwrap();
        assert_eq!(foo.file_type, FileType::Patch, "exact-file patch should beat directory-level host pattern");

        let bar = db.files().find(|f| String::from_utf8_lossy(&f.name).contains("Bar.php")).unwrap();
        assert_eq!(bar.file_type, FileType::Host, "file not covered by patch should remain Host");
    }
}
