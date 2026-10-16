//! Route projects and keep buffer contents shared across their lifetimes.

use super::document_path::DocumentPath;
use super::{OpenDocument, Workspace};
use foldhash::HashMap;
use std::cmp::Reverse;
use std::hash::Hash;
use std::path::{Path, PathBuf};

mod buffers;
mod disk;
mod file;
mod updates;

pub use updates::{WorkspaceChange, WorkspaceError, WorkspaceUpdate};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileChange {
    Created,
    Changed,
    Deleted,
}

#[derive(Clone, Debug)]
pub struct FileEvent {
    pub path: PathBuf,
    pub kind: FileChange,
}

/// Route paths to the most specific root, then configured external sources.
///
/// `K` identifies a caller's buffer and is never parsed. `W` may be a workspace
/// or a caller's wrapper around one; attached client data moves with its workspace.
/// Updates finish analysis before returning affected file IDs to the caller.
pub struct Workspaces<K = PathBuf, W = Workspace<K>> {
    workspaces: Vec<W>,
    documents: HashMap<K, OpenDocument>,
}

impl<K, W> Workspaces<K, W>
where
    K: Eq + Hash + Clone,
    W: AsRef<Workspace<K>> + AsMut<Workspace<K>>,
{
    #[must_use]
    pub fn new(mut workspaces: Vec<W>) -> Self {
        sort_by_root_len(&mut workspaces);
        Self { workspaces, documents: HashMap::default() }
    }

    /// Replay accepted buffers before making a new workspace available.
    ///
    /// # Errors
    ///
    /// Returns an error if analysis fails after replaying the buffers.
    pub fn add(&mut self, mut entry: W) -> Result<usize, WorkspaceError> {
        let workspace = entry.as_mut();
        let changed = self.replay_buffers(workspace);
        workspace
            .server
            .refresh_changed_files(&changed)
            .map_err(|cause| WorkspaceError { root: workspace.root.clone(), cause })?;
        self.workspaces.push(entry);
        sort_by_root_len(&mut self.workspaces);
        self.rebind_buffer_snapshots();
        Ok(changed.len())
    }

    /// Swap in a workspace rebuilt from the same folder, replaying accepted buffers first.
    ///
    /// Returns `None`, and keeps nothing, when no workspace was loaded from that folder.
    ///
    /// # Errors
    ///
    /// Returns an error if analysis fails after replaying the buffers; the current workspace stays.
    pub fn replace(&mut self, mut entry: W) -> Result<Option<usize>, WorkspaceError> {
        let Some(index) = self.root_index(&entry.as_ref().folder) else {
            return Ok(None);
        };
        let workspace = entry.as_mut();
        let changed = self.replay_buffers(workspace);
        workspace
            .server
            .refresh_changed_files(&changed)
            .map_err(|cause| WorkspaceError { root: workspace.root.clone(), cause })?;
        self.workspaces[index] = entry;
        sort_by_root_len(&mut self.workspaces);
        self.rebind_buffer_snapshots();
        Ok(Some(changed.len()))
    }

    /// Buffers survive removal and can be replayed into a new owner.
    pub fn remove(&mut self, root: &Path) -> bool {
        let Some(index) = self.root_index(root) else {
            return false;
        };
        self.workspaces.remove(index);
        self.rebind_buffer_snapshots();
        true
    }

    #[must_use]
    pub fn contains_root(&self, root: &Path) -> bool {
        self.root_index(root).is_some()
    }

    fn root_index(&self, root: &Path) -> Option<usize> {
        self.workspaces
            .iter()
            .position(|ws| ws.as_ref().folder == root)
            .or_else(|| self.workspaces.iter().position(|ws| ws.as_ref().root == root))
    }

    pub fn for_path_mut(&mut self, path: &Path) -> Option<&mut W> {
        let index = self.owner_index(path)?;
        self.workspaces.get_mut(index)
    }

    #[must_use]
    pub fn owner_index(&self, path: &Path) -> Option<usize> {
        let path = DocumentPath::new(path);
        self.workspaces
            .iter()
            .position(|ws| path.requested().starts_with(&ws.as_ref().root))
            .or_else(|| self.workspaces.iter().position(|ws| ws.as_ref().builtin_file_at(path.requested()).is_some()))
            .or_else(|| self.workspaces.iter().position(|ws| path.resolved().starts_with(&ws.as_ref().root)))
            .or_else(|| {
                self.workspaces.iter().position(|ws| {
                    path.direct_file(ws.as_ref()).is_some() || ws.as_ref().matcher.contains(path.resolved())
                })
            })
            .or_else(|| self.workspaces.iter().position(|ws| path.file(ws.as_ref()).is_some()))
    }

    #[must_use]
    pub fn entries(&self) -> &[W] {
        &self.workspaces
    }
    pub fn entries_mut(&mut self) -> &mut [W] {
        &mut self.workspaces
    }
    #[must_use]
    pub fn documents(&self) -> &HashMap<K, OpenDocument> {
        &self.documents
    }
}

fn sort_by_root_len<K, W>(workspaces: &mut [W])
where
    W: AsRef<Workspace<K>>,
{
    workspaces.sort_by_key(|ws| Reverse(ws.as_ref().root.as_os_str().len()));
}

#[cfg(test)]
mod tests;
