//! Adapt shared workspace updates to LSP document keys and publication.

use crate::language_server::uri::UriFilePath;
use foldhash::HashMap;
use mago_server::workspace::registry::{FileChange, FileEvent as NativeFileEvent, Workspaces};
use std::path::Path;
use tower_lsp_server::gen_lsp_types::{FileChangeType, FileEvent, Uri};

use super::diagnostic_publication::{DiagnosticPublication, PublishedDiagnostics};
use super::state::WorkspaceState;

mod publication;
mod updates;

pub use updates::WorkspaceUpdate;

pub struct WorkspaceRegistry {
    core: Workspaces<Uri, WorkspaceState>,
    pub publication: DiagnosticPublication,
}

impl WorkspaceRegistry {
    pub fn new(workspaces: Vec<WorkspaceState>) -> Self {
        let mut registry =
            Self { core: Workspaces::new(workspaces), publication: DiagnosticPublication::new(HashMap::default()) };
        registry.reconcile_diagnostics(&[]);
        registry
    }

    pub fn add(&mut self, workspace: WorkspaceState) -> WorkspaceUpdate {
        let affected = self.diagnostic_uris();
        let root = workspace.core.root.clone();
        match self.core.add(workspace) {
            Ok(changed_files) => {
                tracing::info!(root = %root.display(), "workspace folder added");
                WorkspaceUpdate {
                    changed_files,
                    refresh_code_lenses: true,
                    diagnostics: self.reconcile_diagnostics(&affected),
                    ..WorkspaceUpdate::default()
                }
            }
            Err(error) => WorkspaceUpdate { errors: vec![error.to_string()], ..WorkspaceUpdate::default() },
        }
    }

    pub fn remove(&mut self, root: &Path) -> Vec<(Uri, PublishedDiagnostics)> {
        let affected = self.diagnostic_uris();
        if self.core.remove(root) { self.reconcile_diagnostics(&affected) } else { Vec::new() }
    }

    pub fn contains_root(&self, root: &Path) -> bool {
        self.core.contains_root(root)
    }
    pub fn for_path_mut(&mut self, path: &Path) -> Option<&mut WorkspaceState> {
        self.core.for_path_mut(path)
    }
    fn owner_index(&self, path: &Path) -> Option<usize> {
        self.core.owner_index(path)
    }
    pub fn iter(&self) -> impl Iterator<Item = &WorkspaceState> {
        self.core.entries().iter()
    }
    pub fn document_version(&self, uri: &Uri) -> Option<i32> {
        self.core.document_version(uri)
    }

    pub fn open_buffer(&mut self, uri: Uri, text: String, version: i32) -> WorkspaceUpdate {
        let Some(path) = uri.to_file_path().map(|path| path.into_owned()) else { return WorkspaceUpdate::default() };
        let update = self.core.open_buffer(uri, path, text.into_bytes(), version);
        self.adapt_update(update)
    }

    pub fn change_buffer(&mut self, uri: Uri, text: String, version: i32) -> WorkspaceUpdate {
        let update = self.core.change_buffer(uri, text.into_bytes(), version);
        self.adapt_update(update)
    }

    pub fn close_buffer(&mut self, uri: &Uri) -> WorkspaceUpdate {
        if self.core.document_version(uri).is_none() {
            return WorkspaceUpdate::default();
        }
        let update = self.core.close_buffer(uri);
        let mut update = self.adapt_update(update);
        update.diagnostics.extend(self.reconcile_diagnostics(std::slice::from_ref(uri)));
        update
    }

    pub fn change_disk_files(&mut self, events: Vec<FileEvent>) -> WorkspaceUpdate {
        let events = events
            .into_iter()
            .filter_map(|event| {
                let path = event.uri.to_file_path()?.into_owned();
                let kind = match event.kind {
                    FileChangeType::Created => FileChange::Created,
                    FileChangeType::Changed => FileChange::Changed,
                    FileChangeType::Deleted => FileChange::Deleted,
                    _ => return None,
                };
                Some(NativeFileEvent { path, kind })
            })
            .collect();
        let update = self.core.change_disk_files(events);
        self.adapt_update(update)
    }

    fn adapt_update(&mut self, update: mago_server::workspace::registry::WorkspaceUpdate) -> WorkspaceUpdate {
        let mut output = WorkspaceUpdate {
            changed_files: update.changed_files,
            refresh_code_lenses: update.queries_changed,
            errors: update.errors.into_iter().map(|error| error.to_string()).collect(),
            ..WorkspaceUpdate::default()
        };
        for change in update.affected {
            output.diagnostics.extend(self.changed_diagnostics(change.index, &change.files));
        }
        output
    }
}
