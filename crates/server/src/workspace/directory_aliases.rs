//! Route physical watcher paths through live directory symlinks.

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use foldhash::HashSet;
use mago_database::membership::WorkspaceMatcher;

use super::document_path::DocumentPath;

#[derive(Default)]
pub struct DirectoryAliases {
    links: HashSet<PathBuf>,
}

impl DirectoryAliases {
    #[must_use]
    pub fn new(links: Vec<PathBuf>, matcher: &WorkspaceMatcher) -> Self {
        let mut aliases = Self::default();
        for link in links {
            aliases.observe_directory(&link, matcher);
        }
        aliases
    }

    pub fn targets(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.links.iter().map(|link| DocumentPath::new(link).resolved().to_path_buf())
    }

    #[allow(clippy::set_contains_or_insert, reason = "A duplicate insert can grow a full set.")]
    pub fn observe_directory(&mut self, path: &Path, matcher: &WorkspaceMatcher) {
        if !self.links.contains(path)
            && path.is_symlink()
            && matcher.watch_roots().any(|root| path.starts_with(root))
            && matcher.can_contain_files(path)
            && !matcher.excludes_directory(DocumentPath::new(path).resolved())
        {
            self.links.insert(path.to_path_buf());
        }
    }

    pub fn translate_events(
        &mut self,
        matcher: &WorkspaceMatcher,
        paths: &mut Cow<'_, [PathBuf]>,
        deleted: &mut Cow<'_, HashSet<PathBuf>>,
    ) {
        if self.links.is_empty() {
            return;
        }
        let documents: Vec<_> = paths.iter().map(|path| DocumentPath::new(path)).collect();
        let mut translated = Vec::new();
        self.links.retain(|link| {
            if !link.is_symlink()
                || !matcher.watch_roots().any(|root| link.starts_with(root))
                || !matcher.can_contain_files(link)
            {
                return false;
            }
            let target = DocumentPath::new(link);
            if matcher.excludes_directory(target.resolved()) {
                return false;
            }
            for document in &documents {
                let path = document.requested();
                if path.starts_with(link) {
                    continue;
                }
                let Some(alias) = alias(link, target.resolved(), document.resolved()) else { continue };
                if deleted.contains(path) && !deleted.contains(&alias) {
                    deleted.to_mut().insert(alias.clone());
                }
                translated.push(alias);
            }
            true
        });
        if !translated.is_empty() {
            paths.to_mut().extend(translated);
            paths.to_mut().sort_unstable_by_key(|path| path.components().count());
        }
    }

    /// Add the paths that reach each root through a live link.
    pub fn translate_roots(&self, roots: &mut HashSet<PathBuf>) {
        let mut aliases = Vec::new();
        for link in &self.links {
            let target = DocumentPath::new(link);
            aliases.extend(roots.iter().filter_map(|root| alias(link, target.resolved(), root)));
        }
        roots.extend(aliases);
    }
}

fn alias(link: &Path, target: &Path, path: &Path) -> Option<PathBuf> {
    if let Ok(relative) = path.strip_prefix(target) {
        Some(link.join(relative))
    } else if target.starts_with(path) {
        Some(link.to_path_buf())
    } else {
        None
    }
}

#[cfg(all(test, unix))]
#[path = "directory_aliases/tests.rs"]
mod tests;
