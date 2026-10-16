//! Accept buffer versions once and copy their contents into every matching workspace.

use std::hash::Hash;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::super::document_path::DocumentPath;
use super::super::{OpenDocument, Workspace};
use super::file::FileTarget;
use super::updates::WorkspaceChange;
use super::{WorkspaceUpdate, Workspaces};
use foldhash::HashMap;
use mago_database::error::DatabaseError;
use mago_database::file::{File, FileId, FileType};

impl<K, W> Workspaces<K, W>
where
    K: Eq + Hash + Clone,
    W: AsRef<Workspace<K>> + AsMut<Workspace<K>>,
{
    /// Version accepted for this exact buffer key; closed documents have no version.
    #[must_use]
    pub fn document_version(&self, key: &K) -> Option<i32> {
        self.documents.get(key).map(|document| document.version)
    }

    /// Track a buffer even when no workspace admits its path, so a workspace that admits it later replays it.
    pub fn open_buffer(&mut self, key: K, path: PathBuf, contents: Vec<u8>, version: i32) -> WorkspaceUpdate {
        let owner = self.owner_index(&path);
        let path = DocumentPath::new(&path);
        if self.documents.contains_key(&key)
            || self.is_open_path(path.resolved())
            || owner.is_some_and(|owner| {
                path.file(self.workspaces[owner].as_ref()).is_some_and(|file| file.file_type.is_builtin())
            })
        {
            return WorkspaceUpdate::default();
        }
        self.store_buffer(key, path.requested().to_path_buf(), contents, version)
    }

    pub fn change_buffer(&mut self, key: K, contents: Vec<u8>, version: i32) -> WorkspaceUpdate {
        let Some(document) = self.documents.get(&key).filter(|document| version > document.version) else {
            return WorkspaceUpdate::default();
        };
        self.store_buffer(key.clone(), document.path.clone(), contents, version)
    }

    fn store_buffer(&mut self, key: K, path: PathBuf, contents: Vec<u8>, version: i32) -> WorkspaceUpdate {
        let path = DocumentPath::new(&path);
        let resolved_path = self
            .documents
            .get(&key)
            .map_or_else(|| path.resolved().to_path_buf(), |document| document.resolved_path.clone());
        let mut contents = contents;
        let targets: Vec<_> = self
            .workspaces
            .iter()
            .enumerate()
            .filter_map(|(index, workspace)| Some((index, FileTarget::resolve(workspace.as_ref(), &path)?)))
            .collect();
        let mut changes = Vec::with_capacity(targets.len());
        let mut snapshot = None;
        let count = targets.len();
        for (position, (index, target)) in targets.into_iter().enumerate() {
            let bytes = if position + 1 == count { std::mem::take(&mut contents) } else { contents.clone() };
            let workspace = self.workspaces[index].as_mut();
            let file = target.write(workspace, bytes.into());
            let id = file.id;
            workspace.open_file(id, key.clone(), path.requested().to_path_buf());
            snapshot = Some(file);
            changes.push(WorkspaceChange { index, files: vec![id] });
        }
        let snapshot = snapshot.unwrap_or_else(|| {
            Arc::new(File::new(
                path.resolved().to_string_lossy().as_bytes().to_vec().into(),
                FileType::Host,
                Some(path.resolved().to_path_buf()),
                contents.into(),
            ))
        });
        self.documents
            .insert(key, OpenDocument { path: path.requested().to_path_buf(), resolved_path, snapshot, version });
        self.refresh_changes(changes)
    }

    pub fn close_buffer(&mut self, key: &K) -> WorkspaceUpdate {
        let Some(document) = self.documents.remove(key) else { return WorkspaceUpdate::default() };
        let path = DocumentPath::new(&document.path);
        let mut projections = HashMap::<_, Vec<_>>::default();
        for (index, entry) in self.workspaces.iter_mut().enumerate() {
            let workspace = entry.as_mut();
            let Some(file) = path.file(workspace) else { continue };
            let disk_path = file.path.clone().unwrap_or_else(|| document.path.clone());
            projections.entry(disk_path).or_default().push((index, file.id));
            workspace.close_file(file.id);
        }
        let mut changes = Vec::new();
        let mut unreadable = Vec::new();
        // Restore each loaded path, sharing one read across workspaces that loaded the same path.
        for (disk_path, files) in projections {
            let contents = match File::read(Path::new(""), &disk_path, FileType::Host) {
                Ok(file) => Some(file.contents),
                Err(DatabaseError::IOError(error))
                    if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) =>
                {
                    None
                }
                Err(error) => {
                    tracing::warn!(path = %disk_path.display(), %error, "cannot read closed document");
                    unreadable.extend(files);
                    continue;
                }
            };
            for (index, id) in files {
                let workspace = self.workspaces[index].as_mut();
                let changed = match &contents {
                    Some(contents) => workspace.database_mut().update(id, contents.clone()),
                    None => workspace.database_mut().delete(id),
                };
                if changed || !workspace.server.is_initialized() {
                    changes.push(WorkspaceChange { index, files: if changed { vec![id] } else { Vec::new() } });
                }
            }
        }
        let mut update = self.refresh_changes(changes);
        for (index, id) in unreadable {
            update.affected.push(WorkspaceChange { index, files: vec![id] });
        }
        // Closing releases the accepted version even when the bytes did not change.
        update.queries_changed = true;
        update
    }

    pub(super) fn replay_buffers(&self, workspace: &mut Workspace<K>) -> Vec<FileId> {
        self.documents
            .iter()
            .filter_map(|(key, document)| {
                let path = DocumentPath::new(&document.path);
                let target = FileTarget::resolve(workspace, &path)?;
                let file = target.write(workspace, document.snapshot.contents.clone());
                workspace.open_file(file.id, key.clone(), path.requested().to_path_buf());
                Some(file.id)
            })
            .collect()
    }

    pub(super) fn rebind_buffer_snapshots(&mut self) {
        for document in self.documents.values_mut() {
            for workspace in &self.workspaces {
                let workspace = workspace.as_ref();
                if let Some(file) = DocumentPath::new(&document.path).file(workspace)
                    && file.contents == document.snapshot.contents
                {
                    document.snapshot = file;
                    break;
                }
            }
        }
    }

    pub(super) fn is_open_path(&self, path: &Path) -> bool {
        let path = DocumentPath::new(path);
        self.documents.values().any(|document| document.matches(&path))
    }
}
