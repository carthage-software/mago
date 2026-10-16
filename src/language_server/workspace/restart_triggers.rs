//! Find the file changes that reload a workspace from scratch.

use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use mago_database::git::GitIgnores;

use crate::config::Configuration;

/// The configuration files, Git ignore rules, and `restart-on` paths a workspace loaded with.
#[derive(Default)]
pub struct RestartTriggers {
    files: Vec<PathBuf>,
    paths: Vec<PathBuf>,
    git: Option<Arc<GitIgnores>>,
}

impl RestartTriggers {
    /// Without `git` rules, `uses_git` means a new repository at the root would add them.
    pub fn new(root: &Path, configuration: &Configuration, git: Option<Arc<GitIgnores>>, uses_git: bool) -> Self {
        let mut files: Vec<_> = Configuration::project_config_candidates(root).map(|(file, _)| file).collect();
        files.extend(configuration.config_file.iter().chain(&configuration.config_layers).cloned());
        if uses_git && git.is_none() {
            files.push(root.join(".git"));
        }

        files.sort_unstable();
        files.dedup();
        let paths = configuration
            .language_server
            .restart_on
            .iter()
            .map(|entry| {
                let path = root.join(entry);
                path.canonicalize().unwrap_or(path)
            })
            .collect();

        Self { files, paths, git }
    }

    pub fn matches(&self, path: &Path) -> bool {
        self.files.iter().any(|file| file == path)
            || self.paths.iter().any(|trigger| path.starts_with(trigger))
            || self.git.as_ref().is_some_and(|git| git.depends_on(path))
    }

    /// Files to watch beside the workspace sources.
    pub fn watched_files(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.files.iter().cloned().chain(self.git.iter().flat_map(|git| git.sources_outside()))
    }

    /// Paths to watch with everything below them.
    pub fn watched_paths(&self) -> &[PathBuf] {
        &self.paths
    }
}
