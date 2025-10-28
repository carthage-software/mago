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
    configuration: &'a DatabaseConfiguration<'a>,
    memory_sources: Vec<(&'static str, &'static str, FileType)>,
}

impl<'a> DatabaseLoader<'a> {
    pub fn new(configuration: &'a DatabaseConfiguration<'a>) -> Self {
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

        let mut host_file_ids = HashSet::new();

        for file in host_files {
            host_file_ids.insert(file.id);
            db.add(file);
        }

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

                match read_file(self.configuration.workspace.as_ref(), &path, file_type) {
                    Ok(file) => Some(Ok(file)),
                    Err(e) => Some(Err(e)),
                }
            })
            .collect::<Result<Vec<File>, _>>()?;

        Ok(files)
    }
}
