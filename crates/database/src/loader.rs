//! Database loader for scanning and loading project files.

use std::borrow::Cow;
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::Path;

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
use crate::file::FileType;
use crate::utils::read_file;

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

        let host_files = self.load_paths(
            &self.configuration.paths,
            FileType::Host,
            &extensions_set,
            &glob_excludes,
            &path_excludes,
        )?;
        let vendored_files = self.load_paths(
            &self.configuration.includes,
            FileType::Vendored,
            &extensions_set,
            &glob_excludes,
            &path_excludes,
        )?;

        let mut vendored_file_ids = HashSet::new();

        // Add vendored files first - 'includes' takes precedence over 'paths'
        // This ensures that files explicitly marked as includes are treated as vendored,
        // even if they're within a directory specified in paths
        for file in vendored_files {
            vendored_file_ids.insert(file.id);
            db.add(file);
        }

        // Add host files only if not already added as vendored
        for file in host_files {
            if !vendored_file_ids.contains(&file.id) {
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
    fn load_paths(
        &self,
        roots: &[Cow<'a, str>],
        file_type: FileType,
        extensions: &HashSet<OsString>,
        glob_excludes: &GlobSet,
        path_excludes: &HashSet<&Cow<'a, Path>>,
    ) -> Result<Vec<File>, DatabaseError> {
        let mut paths_to_process = Vec::new();

        for root in roots {
            // Check if this is a glob pattern (contains glob metacharacters)
            let is_glob_pattern = root.contains('*') || root.contains('?') || root.contains('[') || root.contains('{');

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
                                        paths_to_process.push(path);
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
                        paths_to_process.push(entry.into_path());
                    }
                }
            }
        }

        let files: Vec<File> = paths_to_process
            .into_par_iter()
            .filter_map(|path| {
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
                    Ok(file) => Some(Ok(file)),
                    Err(e) => Some(Err(e)),
                }
            })
            .collect::<Result<Vec<File>, _>>()?;

        Ok(files)
    }
}
