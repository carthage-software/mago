use std::path::Path;

use ahash::HashSet;
use async_walkdir::Error;
use async_walkdir::Filtering;
use async_walkdir::WalkDir;
use futures::StreamExt;

use fennec_interner::ThreadedInterner;
use fennec_source::SourceManager;

use crate::source::config::SourceConfiguration;

pub mod config;

#[derive(Debug)]
pub struct SourceService {
    interner: ThreadedInterner,
    configuration: SourceConfiguration,
}

impl SourceService {
    pub fn new(interner: ThreadedInterner, configuration: SourceConfiguration) -> Self {
        Self { interner, configuration }
    }

    /// Load the source manager by scanning and processing the sources
    /// as per the given configuration.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new source manager or a `SourceError` if
    /// an error occurred during the build process.
    pub async fn load<'i>(&self) -> Result<SourceManager, Error> {
        let SourceConfiguration { root, paths, includes, excludes, extensions } = &self.configuration;

        let mut starting_paths = Vec::new();

        if paths.is_empty() {
            starting_paths.push((root.clone(), true));
        } else {
            for source in paths {
                starting_paths.push((source.clone(), true));
            }
        }

        for include in includes {
            starting_paths.push((include.clone(), false));
        }

        if paths.is_empty() && includes.is_empty() {
            starting_paths.push((root.clone(), true));
        }

        let excludes_set: HashSet<&String> = excludes.iter().collect();
        let extensions: HashSet<&String> = extensions.iter().collect();

        let manager = SourceManager::new(self.interner.clone());
        for (path, user_defined) in starting_paths.into_iter() {
            let mut entries = WalkDir::new(path)
                // filter out .git directories
                .filter(|entry| async move {
                    if entry.path().starts_with(".") {
                        Filtering::IgnoreDir
                    } else {
                        Filtering::Continue
                    }
                });

            // Check for errors after processing all entries in the current path
            while let Some(entry) = entries.next().await {
                let path = entry?.path();

                if is_excluded(&path, &excludes_set) {
                    continue;
                }

                if path.is_file() && is_accepted_file(&path, &extensions) {
                    let name = match path.strip_prefix(root) {
                        Ok(rel_path) => rel_path.to_path_buf(),
                        Err(_) => path.clone(),
                    };

                    let name_str = name.to_string_lossy().to_string();

                    manager.insert_path(name_str, path.clone(), user_defined);
                }
            }
        }

        Ok(manager)
    }
}

fn is_excluded(path: &Path, excludes: &HashSet<&String>) -> bool {
    excludes.iter().any(|ex| path.ends_with(ex) || glob_match::glob_match(ex, path.to_string_lossy().as_ref()))
}

fn is_accepted_file(path: &Path, extensions: &HashSet<&String>) -> bool {
    if extensions.is_empty() {
        path.extension().and_then(|s| s.to_str()).map(|ext| ext.eq_ignore_ascii_case("php")).unwrap_or(false)
    } else {
        path.extension().and_then(|s| s.to_str()).map(|ext| extensions.contains(&ext.to_string())).unwrap_or(false)
    }
}
