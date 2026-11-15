//! Resolve document aliases while keeping the loader's file identities.

use std::cell::OnceCell;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use mago_database::DatabaseReader;
use mago_database::file::File;

use super::Workspace;
use super::paths::file_id_for;

pub struct DocumentPath<'a> {
    requested: &'a Path,
    resolved: OnceCell<PathBuf>,
}

impl<'a> DocumentPath<'a> {
    #[must_use]
    pub fn new(requested: &'a Path) -> Self {
        Self { requested, resolved: OnceCell::new() }
    }

    pub fn requested(&self) -> &Path {
        self.requested
    }

    pub fn resolved(&self) -> &Path {
        self.resolved.get_or_init(|| resolve_path(self.requested))
    }

    pub fn direct_file<K>(&self, workspace: &Workspace<K>) -> Option<Arc<File>> {
        let database = workspace.database();
        let direct = |path: &Path| {
            workspace
                .builtin_file_at(path)
                .or_else(|| database.get(&file_id_for(&workspace.root, path)).ok().filter(|file| file.path.is_some()))
                .or_else(|| database.get(&workspace.opened_file_at(path)?).ok())
        };
        if let Some(file) = direct(self.requested()) {
            return Some(file);
        }
        direct(self.resolved())
    }

    pub fn file<K>(&self, workspace: &Workspace<K>) -> Option<Arc<File>> {
        if let Some(file) = self.direct_file(workspace) {
            return Some(file);
        }
        let resolved = self.resolved();
        // Missing files cannot match another live symlink target. Avoid scanning on new buffers.
        if !resolved.exists() {
            return None;
        }
        let database = workspace.database();

        // Directory walks keep symlink paths. Check likely names before renamed links.
        for same_name in [true, false] {
            if let Some(file) = database.files().find(|file| {
                let Some(path) = file.path.as_deref() else { return false };
                let matches_name =
                    path.file_name() == self.requested.file_name() || path.file_name() == resolved.file_name();
                matches_name == same_name && resolve_path(path) == resolved
            }) {
                return Some(file);
            }
        }
        None
    }
}

fn resolve_path(path: &Path) -> PathBuf {
    resolve_path_with_limit(path, 32)
}

fn resolve_path_with_limit(path: &Path, remaining_links: usize) -> PathBuf {
    if let Ok(canonical) = path.canonicalize() {
        return canonical;
    }
    if remaining_links == 0 {
        return path.to_path_buf();
    }
    for ancestor in path.ancestors() {
        if let Ok(target) = ancestor.read_link() {
            let target = if target.is_absolute() { target } else { ancestor.parent().unwrap_or(ancestor).join(target) };
            let target = resolve_path_with_limit(&target, remaining_links - 1);
            let suffix = path.strip_prefix(ancestor).unwrap_or(Path::new(""));
            return if suffix.as_os_str().is_empty() { target } else { target.join(suffix) };
        }
        if let Ok(canonical) = ancestor.canonicalize() {
            return canonical.join(path.strip_prefix(ancestor).unwrap_or(Path::new("")));
        }
    }
    path.to_path_buf()
}
