//! Database loader for scanning and loading project files.

use std::borrow::Cow;
use std::ffi::OsString;
use std::path::Path;

use ahash::HashMap;
use ahash::HashSet;
use globset::Glob;
use globset::GlobSet;
use globset::GlobSetBuilder;
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::Database;
use crate::DatabaseConfiguration;
use crate::error::DatabaseError;
use crate::exclusion::Exclusion;
use crate::file::File;
use crate::file::FileId;
use crate::file::FileType;
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
pub struct DatabaseLoader<'a> {
    database: Option<Database<'a>>,
    configuration: DatabaseConfiguration<'a>,
    memory_sources: Vec<(&'static str, &'static str, FileType)>,
}

impl<'a> DatabaseLoader<'a> {
    pub fn new(configuration: DatabaseConfiguration<'a>) -> Self {
        Self { configuration, memory_sources: vec![], database: None }
    }

    pub fn with_database(mut self, database: Database<'a>) -> Self {
        self.database = Some(database);
        self
    }

    pub fn add_memory_source(&mut self, name: &'static str, contents: &'static str, file_type: FileType) {
        self.memory_sources.push((name, contents, file_type));
    }

    pub fn load(mut self) -> Result<Database<'a>, DatabaseError> {
        let mut db = self.database.take().unwrap_or_else(|| Database::new(self.configuration.clone()));

        // Update database configuration to use the loader's configuration
        // (fixes workspace path when merging with prelude database)
        db.configuration = self.configuration.clone();

        let extensions_set: HashSet<OsString> =
            self.configuration.extensions.iter().map(|s| OsString::from(s.as_ref())).collect();

        let mut glob_builder = GlobSetBuilder::new();
        for ex in &self.configuration.excludes {
            if let Exclusion::Pattern(pat) = ex {
                glob_builder.add(Glob::new(pat)?);
            }
        }

        let glob_excludes = glob_builder.build()?;

        let path_excludes: HashSet<_> = self
            .configuration
            .excludes
            .iter()
            .filter_map(|ex| match ex {
                Exclusion::Path(p) => Some(p),
                _ => None,
            })
            .collect();

        let host_files_with_spec = self.load_paths(
            &self.configuration.paths,
            FileType::Host,
            &extensions_set,
            &glob_excludes,
            &path_excludes,
        )?;
        let vendored_files_with_spec = self.load_paths(
            &self.configuration.includes,
            FileType::Vendored,
            &extensions_set,
            &glob_excludes,
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
        roots: &[Cow<'a, str>],
        file_type: FileType,
        extensions: &HashSet<OsString>,
        glob_excludes: &GlobSet,
        path_excludes: &HashSet<&Cow<'a, Path>>,
    ) -> Result<Vec<FileWithSpecificity>, DatabaseError> {
        let mut paths_to_process: Vec<(std::path::PathBuf, String, usize)> = Vec::new();

        for root in roots {
            // Check if this is a glob pattern (contains glob metacharacters)
            let is_glob_pattern = root.contains('*') || root.contains('?') || root.contains('[') || root.contains('{');

            let specificity = Self::calculate_pattern_specificity(root.as_ref());
            if is_glob_pattern {
                // Handle as glob pattern
                let pattern = if Path::new(root.as_ref()).is_absolute() {
                    root.to_string()
                } else {
                    // Make relative patterns absolute by prepending workspace
                    self.configuration.workspace.join(root.as_ref()).to_string_lossy().to_string()
                };

                match glob::glob(&pattern) {
                    Ok(entries) => {
                        for entry in entries {
                            match entry {
                                Ok(path) => {
                                    if path.is_file() {
                                        paths_to_process.push((path, root.to_string(), specificity));
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
                // Handle as directory path (existing logic)
                let dir_path = if Path::new(root.as_ref()).is_absolute() {
                    Path::new(root.as_ref()).to_path_buf()
                } else {
                    self.configuration.workspace.join(root.as_ref())
                };

                for entry in WalkDir::new(&dir_path).into_iter().filter_map(Result::ok) {
                    if entry.file_type().is_file() {
                        paths_to_process.push((entry.into_path(), root.to_string(), specificity));
                    }
                }
            }
        }

        let files: Vec<FileWithSpecificity> = paths_to_process
            .into_par_iter()
            .filter_map(|(path, _pattern, specificity)| {
                if glob_excludes.is_match(&path) {
                    return None;
                }

                if let Ok(canonical_path) = path.canonicalize()
                    && path_excludes.iter().any(|excluded| canonical_path.starts_with(excluded))
                {
                    return None;
                }

                if let Some(ext) = path.extension() {
                    if !extensions.contains(ext) {
                        return None;
                    }
                } else {
                    return None;
                }

                match read_file(self.configuration.workspace.as_ref(), &path, file_type) {
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
    fn calculate_pattern_specificity(pattern: &str) -> usize {
        let pattern_path = Path::new(pattern);

        let component_count = pattern_path.components().count();
        let is_glob = pattern.contains('*') || pattern.contains('?') || pattern.contains('[') || pattern.contains('{');

        if is_glob {
            let non_wildcard_components = pattern_path
                .components()
                .filter(|c| {
                    let s = c.as_os_str().to_string_lossy();
                    !s.contains('*') && !s.contains('?') && !s.contains('[') && !s.contains('{')
                })
                .count();
            non_wildcard_components * 10
        } else if pattern_path.is_file() || pattern_path.extension().is_some() || pattern.ends_with(".php") {
            component_count * 1000
        } else {
            component_count * 100
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DatabaseReader;
    use std::borrow::Cow;
    use tempfile::TempDir;

    fn create_test_config(temp_dir: &TempDir, paths: Vec<&str>, includes: Vec<&str>) -> DatabaseConfiguration<'static> {
        // Normalize path separators to platform-specific separators
        let normalize = |s: &str| s.replace('/', std::path::MAIN_SEPARATOR_STR);

        DatabaseConfiguration {
            workspace: Cow::Owned(temp_dir.path().to_path_buf()),
            paths: paths.into_iter().map(|s| Cow::Owned(normalize(s))).collect(),
            includes: includes.into_iter().map(|s| Cow::Owned(normalize(s))).collect(),
            excludes: vec![],
            extensions: vec![Cow::Borrowed("php")],
        }
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
        let spec = DatabaseLoader::calculate_pattern_specificity("src/b.php");
        assert!(spec >= 2000, "Exact file should have high specificity, got {}", spec);
    }

    #[test]
    fn test_specificity_calculation_directory() {
        let spec = DatabaseLoader::calculate_pattern_specificity("src/");
        assert!((100..1000).contains(&spec), "Directory should have moderate specificity, got {}", spec);
    }

    #[test]
    fn test_specificity_calculation_glob() {
        let spec = DatabaseLoader::calculate_pattern_specificity("src/*.php");
        assert!(spec < 100, "Glob pattern should have low specificity, got {}", spec);
    }

    #[test]
    fn test_specificity_calculation_deeper_path() {
        let shallow_spec = DatabaseLoader::calculate_pattern_specificity("src/");
        let deep_spec = DatabaseLoader::calculate_pattern_specificity("src/foo/bar/");
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

        let b_file = db.files().find(|f| f.name.contains("b.php")).unwrap();
        assert_eq!(b_file.file_type, FileType::Host, "src/b.php should be Host (exact file beats directory)");

        let a_file = db.files().find(|f| f.name.contains("a.php")).unwrap();
        assert_eq!(a_file.file_type, FileType::Vendored, "src/a.php should be Vendored");
    }

    #[test]
    fn test_deeper_vs_shallower_directory() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(&temp_dir, "src/foo/bar.php", "<?php");

        let config = create_test_config(&temp_dir, vec!["src/foo/"], vec!["src/"]);
        let loader = DatabaseLoader::new(config);
        let db = loader.load().unwrap();

        let file = db.files().find(|f| f.name.contains("bar.php")).unwrap();
        assert_eq!(file.file_type, FileType::Host, "Deeper directory pattern should win");
    }

    #[test]
    fn test_exact_file_vs_glob() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(&temp_dir, "src/b.php", "<?php");

        let config = create_test_config(&temp_dir, vec!["src/b.php"], vec!["src/*.php"]);
        let loader = DatabaseLoader::new(config);
        let db = loader.load().unwrap();

        let file = db.files().find(|f| f.name.contains("b.php")).unwrap();
        assert_eq!(file.file_type, FileType::Host, "Exact file should beat glob pattern");
    }

    #[test]
    fn test_equal_specificity_includes_wins() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(&temp_dir, "src/a.php", "<?php");

        let config = create_test_config(&temp_dir, vec!["src/"], vec!["src/"]);
        let loader = DatabaseLoader::new(config);
        let db = loader.load().unwrap();

        let file = db.files().find(|f| f.name.contains("a.php")).unwrap();
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

        let b_file = db.files().find(|f| f.name.contains("src/b.php") || f.name.ends_with("b.php")).unwrap();
        assert_eq!(b_file.file_type, FileType::Host, "src/b.php should be Host in bug scenario");

        let d_file = db.files().find(|f| f.name.contains("d.php")).unwrap();
        assert_eq!(d_file.file_type, FileType::Vendored, "src/c/d.php should be Vendored");

        let lib_file = db.files().find(|f| f.name.contains("lib1.php")).unwrap();
        assert_eq!(lib_file.file_type, FileType::Vendored, "vendor/lib1.php should be Vendored");
    }

    #[test]
    fn test_files_only_in_paths() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(&temp_dir, "src/a.php", "<?php");

        let config = create_test_config(&temp_dir, vec!["src/"], vec![]);
        let loader = DatabaseLoader::new(config);
        let db = loader.load().unwrap();

        let file = db.files().find(|f| f.name.contains("a.php")).unwrap();
        assert_eq!(file.file_type, FileType::Host, "File only in paths should be Host");
    }

    #[test]
    fn test_files_only_in_includes() {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(&temp_dir, "vendor/lib.php", "<?php");

        let config = create_test_config(&temp_dir, vec![], vec!["vendor/"]);
        let loader = DatabaseLoader::new(config);
        let db = loader.load().unwrap();

        let file = db.files().find(|f| f.name.contains("lib.php")).unwrap();
        assert_eq!(file.file_type, FileType::Vendored, "File only in includes should be Vendored");
    }
}
