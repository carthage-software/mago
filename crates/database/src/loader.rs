//! Database loader for scanning and loading project files.
//!
//! This module provides [`DatabaseLoader`], a builder for constructing a [`Database`]
//! by scanning directories on the filesystem and optionally including in-memory sources.
//!
//! The loader handles:
//! - Recursive directory traversal
//! - File extension filtering
//! - Exclusion pattern matching (paths and globs)
//! - Parallel file I/O for performance
//! - Separation of host and vendored files
//! - In-memory file injection

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
use crate::error::DatabaseError;
use crate::exclusion::Exclusion;
use crate::file::File;
use crate::file::FileType;
use crate::utils::read_file;

/// A builder for loading files into a [`Database`] from the filesystem and memory.
///
/// This struct provides a flexible interface for discovering, filtering, and loading
/// source files into a database. It supports recursive directory scanning with configurable
/// inclusion/exclusion rules, parallel file I/O, and mixing of filesystem and in-memory sources.
///
/// # Workflow
///
/// 1. Create a loader with [`new`](Self::new), specifying workspace, paths, and exclusions
/// 2. Optionally call [`with_database`](Self::with_database) to populate an existing database
/// 3. Optionally call [`add_memory_source`](Self::add_memory_source) to inject in-memory files
/// 4. Call [`load`](Self::load) to execute the scan and build the database
///
/// # File Classification
///
/// Files are classified into two types during loading:
///
/// - **Host Files**: Primary source code from paths specified in the `paths` parameter
/// - **Vendored Files**: Dependencies from paths specified in the `includes` parameter
///
/// If a file appears in both host and vendored paths, the host version takes precedence.
///
/// # Filtering
///
/// Files are included only if they:
/// 1. Have an extension matching one in the `extensions` list
/// 2. Are not excluded by path-based exclusions
/// 3. Do not match any glob-based exclusion patterns
///
/// # Performance
///
/// File discovery is performed sequentially using `walkdir`, but file I/O is parallelized
/// using Rayon for significantly faster loading of large projects.
///
/// # Lifetime
///
/// The `'a` lifetime allows the loader to borrow paths and exclusions from configuration
/// or other long-lived structures, minimizing allocations.
pub struct DatabaseLoader<'a> {
    /// Optional pre-existing database to populate. If `None`, a new database is created.
    database: Option<Database>,

    /// The workspace root directory, used to resolve relative paths and canonicalize exclusions.
    workspace: &'a Path,

    /// Primary source paths to scan for host files.
    ///
    /// These paths are searched for files with matching extensions, and discovered files
    /// are marked as `FileType::Host`.
    paths: Vec<Cow<'a, Path>>,

    /// Additional paths to scan for vendored/dependency files.
    ///
    /// Files found in these paths are marked as `FileType::Vendored`. If a file appears
    /// in both `paths` and `includes`, the host version takes precedence.
    includes: Vec<Cow<'a, Path>>,

    /// Exclusion rules for filtering discovered files.
    ///
    /// These can be exact paths or glob patterns. Paths are canonicalized relative to
    /// the workspace during loader construction.
    excludes: Vec<Exclusion<'a>>,

    /// In-memory sources to inject into the database.
    ///
    /// These files don't exist on the filesystem but are included in the database as if
    /// they did. Useful for built-in stubs or dynamically generated code.
    memory_sources: Vec<(&'static str, &'static str, FileType)>,

    /// File extensions to include during the scan (e.g., `["php", "phtml"]`).
    ///
    /// Only files with these extensions are loaded into the database. Extensions should
    /// be specified without the leading dot.
    extensions: Vec<&'a str>,
}

impl<'a> DatabaseLoader<'a> {
    /// Creates a new loader with the given configuration.
    ///
    /// All provided exclusion paths are canonicalized relative to the workspace
    /// upon creation to ensure they are matched correctly.
    pub fn new(
        workspace: &'a Path,
        paths: Vec<&'a Path>,
        includes: Vec<&'a Path>,
        excludes: Vec<Exclusion<'a>>,
        extensions: Vec<&'a str>,
    ) -> Self {
        let paths = canonicalize_paths(workspace, paths);
        let includes = canonicalize_paths(workspace, includes);

        let excludes = excludes
            .into_iter()
            .filter_map(|exclusion| match exclusion {
                Exclusion::Path(p) => Some(if p.is_absolute() {
                    Exclusion::Path(p)
                } else {
                    workspace.join(p).canonicalize().ok().map(Cow::Owned).map(Exclusion::Path)?
                }),
                Exclusion::Pattern(pat) => Some(Exclusion::Pattern(pat)),
            })
            .collect();

        Self { workspace, paths, includes, excludes, memory_sources: vec![], extensions, database: None }
    }

    /// Sets a pre-existing database to populate.
    pub fn with_database(mut self, database: Database) -> Self {
        self.database = Some(database);
        self
    }

    /// Adds a memory source to the loader.
    ///
    /// This allows you to include files that are not on the filesystem but should be part of the database.
    ///
    /// # Arguments
    ///
    /// * `name` - The logical name of the file, typically its path relative to the workspace.
    /// * `contents` - The contents of the file as a string.
    /// * `file_type` - The type of the file, indicating whether it's a host file or a vendored file.
    pub fn add_memory_source(&mut self, name: &'static str, contents: &'static str, file_type: FileType) {
        self.memory_sources.push((name, contents, file_type));
    }

    /// Scans sources according to the configuration and builds a `Database`.
    pub fn load(mut self) -> Result<Database, DatabaseError> {
        let mut db = self.database.take().unwrap_or_default();
        let extensions_set: HashSet<OsString> = self.extensions.iter().map(OsString::from).collect();

        let mut glob_builder = GlobSetBuilder::new();
        for ex in &self.excludes {
            if let Exclusion::Pattern(pat) = ex {
                glob_builder.add(Glob::new(pat)?);
            }
        }

        let glob_excludes = glob_builder.build()?;

        let path_excludes: HashSet<_> = self
            .excludes
            .iter()
            .filter_map(|ex| match ex {
                Exclusion::Path(p) => Some(p),
                _ => None,
            })
            .collect();

        let host_files =
            self.load_paths(&self.paths, FileType::Host, &extensions_set, &glob_excludes, &path_excludes)?;
        let vendored_files =
            self.load_paths(&self.includes, FileType::Vendored, &extensions_set, &glob_excludes, &path_excludes)?;

        // Track host file IDs to prevent them from being overwritten by vendored files
        let mut host_file_ids = HashSet::new();

        for file in host_files {
            host_file_ids.insert(file.id);
            db.add(file);
        }

        // Only add vendored files if they weren't already added as host files
        for file in vendored_files {
            if !host_file_ids.contains(&file.id) {
                db.add(file);
            }
        }

        for (name, contents, file_type) in self.memory_sources {
            let file = File::new(Cow::Borrowed(name), file_type, None, Cow::Borrowed(contents));

            db.add(file);
        }

        Ok(db)
    }

    /// Discovers and reads all files from a set of root paths in parallel.
    fn load_paths(
        &self,
        roots: &[Cow<'a, Path>],
        file_type: FileType,
        extensions: &HashSet<OsString>,
        glob_excludes: &GlobSet,
        path_excludes: &HashSet<&Cow<'a, Path>>,
    ) -> Result<Vec<File>, DatabaseError> {
        let mut paths_to_process = Vec::new();
        for root in roots {
            for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
                if entry.file_type().is_file() {
                    paths_to_process.push(entry.into_path());
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

                match read_file(self.workspace, &path, file_type) {
                    Ok(file) => Some(Ok(file)),
                    Err(e) => Some(Err(e)),
                }
            })
            .collect::<Result<Vec<File>, _>>()?;

        Ok(files)
    }
}

/// A helper function to canonicalize a vector of paths relative to a workspace.
///
/// It handles both absolute and relative paths and logs a warning for any
/// path that cannot be resolved, filtering it out from the final result.
fn canonicalize_paths<'a>(workspace: &'a Path, paths: Vec<&'a Path>) -> Vec<Cow<'a, Path>> {
    paths
        .into_iter()
        .filter_map(|p| {
            Some(if p.is_absolute() {
                Cow::Borrowed(p)
            } else {
                workspace
                    .join(p)
                    .canonicalize()
                    .inspect_err(|_| tracing::warn!("Ignoring invalid or non-existent path `{}`", p.display()))
                    .ok()
                    .map(Cow::Owned)?
            })
        })
        .collect()
}
