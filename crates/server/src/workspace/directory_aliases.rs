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
            && !matcher.is_excluded(DocumentPath::new(path).resolved())
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
                || matcher.is_excluded(link)
            {
                return false;
            }
            let target = DocumentPath::new(link);
            if matcher.is_excluded(target.resolved()) {
                return false;
            }
            for document in &documents {
                let path = document.requested();
                if path.starts_with(link) {
                    continue;
                }
                let alias = if let Ok(relative) = document.resolved().strip_prefix(target.resolved()) {
                    link.join(relative)
                } else if target.resolved().starts_with(document.resolved()) {
                    link.clone()
                } else {
                    continue;
                };
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
}

#[cfg(all(test, unix))]
#[path = "directory_aliases/tests.rs"]
mod tests;
