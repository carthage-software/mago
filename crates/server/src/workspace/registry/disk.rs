//! Apply file changes without overwriting accepted buffers.

use std::borrow::Cow;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::super::Workspace;
use super::super::document_path::DocumentPath;
use super::file::FileTarget;
use super::updates::WorkspaceChange;
use super::{FileChange, FileEvent, WorkspaceUpdate, Workspaces};
use foldhash::{HashMap, HashSet};
use mago_database::DatabaseReader;
use mago_database::file::{File, FileId, FileType};

impl<K, W> Workspaces<K, W>
where
    K: Eq + Hash + Clone,
    W: AsRef<Workspace<K>> + AsMut<Workspace<K>>,
{
    pub fn change_disk_files(&mut self, events: Vec<FileEvent>) -> WorkspaceUpdate {
        let deleted: HashSet<_> =
            events.iter().filter(|event| event.kind == FileChange::Deleted).map(|event| event.path.clone()).collect();
        let mut paths: Vec<_> = events.into_iter().map(|event| event.path).collect();
        // A directory event covers its children, regardless of the client's event order.
        paths.sort_unstable_by_key(|path| path.components().count());
        let open: HashSet<_> = self
            .documents
            .values()
            .flat_map(|document| {
                [
                    document.path.clone(),
                    document.resolved_path.clone(),
                    DocumentPath::new(&document.path).resolved().to_path_buf(),
                ]
            })
            .collect();
        let mut changes = Vec::new();
        for (index, entry) in self.workspaces.iter_mut().enumerate() {
            let workspace = entry.as_mut();
            let mut paths = Cow::Borrowed(paths.as_slice());
            let mut deleted = Cow::Borrowed(&deleted);
            workspace.directory_aliases.translate_events(&workspace.matcher, &mut paths, &mut deleted);
            let mut changed = Vec::new();
            let readable = prepare_paths(workspace, &paths, &deleted, &open, &mut changed);
            let retry = !workspace.server.is_initialized()
                && (!readable.is_empty()
                    || paths.iter().any(|path| DocumentPath::new(path).direct_file(workspace).is_some()));
            let mut loaded = None;
            let mut visited = HashSet::default();
            for path in readable {
                read_path(workspace, &path, &open, &mut loaded, &mut visited, &mut changed);
            }
            if !changed.is_empty() || retry {
                changed.sort_unstable();
                changed.dedup();
                changes.push(WorkspaceChange { index, files: changed });
            }
        }
        self.refresh_changes(changes)
    }
}

fn prepare_paths<K>(
    workspace: &mut Workspace<K>,
    paths: &[PathBuf],
    deleted: &HashSet<PathBuf>,
    open: &HashSet<PathBuf>,
    changed: &mut Vec<FileId>,
) -> Vec<PathBuf> {
    let mut covered = HashSet::default();
    let mut missing_roots = HashSet::default();
    let mut matched = HashSet::default();
    let mut readable = Vec::new();
    for path in paths {
        let document = DocumentPath::new(path);
        if path.ancestors().chain(document.resolved().ancestors()).any(|ancestor| covered.contains(ancestor)) {
            continue;
        }
        let loaded = document.direct_file(workspace);
        if is_missing(path) {
            if let Some(file) = loaded {
                remove_file(workspace, &file, open, changed);
            } else {
                match missing_path(workspace, path, document.resolved()) {
                    MissingPath::Root => {
                        // A missing external target may still belong to a loaded package symlink.
                        missing_roots.insert(path.clone());
                        missing_roots.insert(document.resolved().to_path_buf());
                    }
                    MissingPath::Matched => {
                        matched.insert(path.clone());
                        matched.insert(document.resolved().to_path_buf());
                        continue;
                    }
                    MissingPath::Excluded => continue,
                }
            }
        } else if path.is_dir() {
            if !workspace.matcher.can_contain_files(path) {
                continue;
            }
            missing_roots.insert(path.clone());
            missing_roots.insert(document.resolved().to_path_buf());
            readable.push(path.clone());
        } else {
            if loaded.is_none() && deleted.contains(path) && workspace.matcher.can_contain_files(path) {
                // A deleted directory may already have been replaced by a regular file.
                missing_roots.insert(path.clone());
                missing_roots.insert(document.resolved().to_path_buf());
            }
            if loaded.is_none() && !workspace.matcher.contains(path) {
                continue;
            }
            readable.push(path.clone());
        }
        covered.insert(path.clone());
        covered.insert(document.resolved().to_path_buf());
    }
    if !matched.is_empty() {
        for file in workspace.database().files() {
            let Some(path) = file.path.as_deref() else { continue };
            missing_roots
                .extend(path.ancestors().skip(1).filter(|ancestor| matched.contains(*ancestor)).map(Path::to_path_buf));
        }
    }
    if !missing_roots.is_empty() {
        // Deleted unknown paths may be directories. Check loaded children once for the whole batch.
        let missing: Vec<_> = workspace
            .database()
            .files()
            .filter(|file| {
                !file.file_type.is_builtin()
                    && file.path.as_ref().is_some_and(|path| {
                        is_missing(path)
                            && (path.ancestors().any(|ancestor| missing_roots.contains(ancestor))
                                || DocumentPath::new(path)
                                    .resolved()
                                    .ancestors()
                                    .any(|ancestor| missing_roots.contains(ancestor)))
                    })
            })
            .collect();
        for file in missing {
            remove_file(workspace, &file, open, changed);
        }
    }
    readable
}

/// How a missing path that was not loaded itself takes part in cleanup.
enum MissingPath {
    /// It may have been a directory or a package symlink target with loaded files.
    Root,
    /// A file glob matches it. A glob never excludes a directory, so it was one only
    /// if loaded files lie below it, which one pass over loaded paths decides per batch.
    Matched,
    /// A path exclusion covers it, so nothing below it was loaded.
    Excluded,
}

fn missing_path<K>(workspace: &Workspace<K>, path: &Path, resolved: &Path) -> MissingPath {
    let matcher = &workspace.matcher;
    if matcher.excludes_directory(path) || matcher.excludes_directory(resolved) {
        MissingPath::Excluded
    } else if matcher.is_excluded(path) || matcher.is_excluded(resolved) {
        MissingPath::Matched
    } else {
        MissingPath::Root
    }
}

fn is_missing(path: &Path) -> bool {
    match path.try_exists() {
        Ok(exists) => !exists,
        Err(error) => matches!(error.kind(), std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory),
    }
}

fn remove_file<K>(workspace: &mut Workspace<K>, file: &File, open: &HashSet<PathBuf>, changed: &mut Vec<FileId>) {
    if !file.file_type.is_builtin()
        && workspace.opened_key(file.id).is_none()
        && file.path.as_ref().is_none_or(|path| !open.contains(path))
        && workspace.database_mut().delete(file.id)
    {
        changed.push(file.id);
    }
}

fn read_path<K>(
    workspace: &mut Workspace<K>,
    path: &Path,
    open: &HashSet<PathBuf>,
    loaded: &mut Option<HashMap<PathBuf, Arc<File>>>,
    visited: &mut HashSet<PathBuf>,
    changed: &mut Vec<FileId>,
) {
    let mut paths = vec![path.to_path_buf()];
    while let Some(path) = paths.pop() {
        let document = DocumentPath::new(&path);
        let is_directory = path.is_dir();
        let included_directory = is_directory && workspace.matcher.can_contain_files(&path);
        if included_directory {
            workspace.directory_aliases.observe_directory(&path, &workspace.matcher);
        }
        if !visited.insert(document.resolved().to_path_buf()) {
            continue;
        }
        if is_directory {
            if included_directory && let Ok(entries) = std::fs::read_dir(&path) {
                paths.extend(entries.filter_map(Result::ok).map(|entry| entry.path()));
            }
            continue;
        }
        if open.contains(&path) || open.contains(document.resolved()) {
            continue;
        }
        let mut file = document.direct_file(workspace);
        if file.is_none() {
            if !workspace.matcher.contains(&path) {
                continue;
            }
            let loaded = loaded.get_or_insert_with(|| {
                workspace
                    .database()
                    .files()
                    .filter_map(|file| {
                        let path = file.path.as_ref()?;
                        Some((DocumentPath::new(path).resolved().to_path_buf(), file))
                    })
                    .collect()
            });
            file = loaded.get(document.resolved()).and_then(|file| workspace.database().get(&file.id).ok());
        }
        if file.as_ref().is_some_and(|file| workspace.opened_key(file.id).is_some()) {
            continue;
        }
        let Some(target) = FileTarget::resolve_loaded(workspace, &document, file) else { continue };
        let Ok(file) = File::read(Path::new(""), &path, FileType::Host) else { continue };
        if !target.unchanged(&file.contents) {
            changed.push(target.write(workspace, file.contents).id);
        }
    }
}
