//! Describe a completed workspace update without protocol payloads.

use super::{Workspace, Workspaces};
use crate::ServerError;
use mago_database::file::FileId;
use std::hash::Hash;
use std::path::PathBuf;

#[derive(Default)]
pub struct WorkspaceUpdate {
    pub changed_files: usize,
    pub queries_changed: bool,
    pub affected: Vec<WorkspaceChange>,
    pub errors: Vec<WorkspaceError>,
}

pub struct WorkspaceChange {
    pub index: usize,
    pub files: Vec<FileId>,
}

/// The workspace that failed and its analysis error.
#[derive(Debug)]
pub struct WorkspaceError {
    pub root: PathBuf,
    pub cause: ServerError,
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.root.display(), self.cause)
    }
}

impl std::error::Error for WorkspaceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}

impl<K, W> Workspaces<K, W>
where
    K: Eq + Hash + Clone,
    W: AsRef<Workspace<K>> + AsMut<Workspace<K>>,
{
    pub(super) fn refresh_changes(&mut self, changes: Vec<WorkspaceChange>) -> WorkspaceUpdate {
        let mut update = WorkspaceUpdate::default();
        for WorkspaceChange { index, files } in changes {
            let workspace = self.workspaces[index].as_mut();
            if files.is_empty() && workspace.server.is_initialized() {
                continue;
            }
            update.changed_files += files.len();
            update.queries_changed = true;
            if let Err(cause) = workspace.server.refresh_changed_files(&files) {
                update.errors.push(WorkspaceError { root: workspace.root.clone(), cause });
            }
            update.affected.push(WorkspaceChange { index, files });
        }
        update
    }
}
