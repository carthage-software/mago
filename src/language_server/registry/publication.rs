//! Choose one diagnostic owner before comparing output sent to the client.

use foldhash::{HashMap, HashSet};
use tower_lsp_server::gen_lsp_types::Uri;

use mago_database::file::FileId;
use mago_server::workspace::OpenDocument;
use mago_server::workspace::document_path::DocumentPath;
use mago_server::workspace::paths::file_id_for;

use super::WorkspaceRegistry;
use crate::language_server::diagnostic_publication::PublishedDiagnostics;
use crate::language_server::state::WorkspaceState;
use crate::language_server::uri::UriFilePath;

/// A workspace may publish its own file or its open buffer, but not another owner's issues.
enum DiagnosticRoute<'a> {
    File,
    Buffer { uri: &'a Uri, version: i32 },
    ClearAlias,
    Ignore,
}

impl WorkspaceRegistry {
    pub(super) fn diagnostic_uris(&self) -> Vec<Uri> {
        self.publication
            .uris()
            .chain(self.core.entries().iter().flat_map(|workspace| workspace.diagnostics.uris()))
            .chain(self.core.documents().keys())
            .cloned()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    pub(super) fn changed_diagnostics(&mut self, index: usize, changed: &[FileId]) -> Vec<(Uri, PublishedDiagnostics)> {
        let diagnostics = diagnostics_for(&mut self.core.entries_mut()[index], changed);
        let diagnostics = self.route_diagnostics(index, diagnostics);
        self.publication.update(diagnostics)
    }

    /// Force current payloads when adding or removing roots, even if their issues are unchanged.
    pub(super) fn reconcile_diagnostics(&mut self, affected: &[Uri]) -> Vec<(Uri, PublishedDiagnostics)> {
        let mut diagnostics = HashMap::default();
        for index in 0..self.core.entries().len() {
            let changed = affected
                .iter()
                .filter(|uri| {
                    matches!(self.diagnostic_route(index, uri), DiagnosticRoute::File | DiagnosticRoute::Buffer { .. })
                })
                .filter_map(|uri| {
                    let path = uri.to_file_path()?;
                    let path = DocumentPath::new(&path);
                    let workspace = &self.core.entries()[index];
                    Some(
                        path.file(&workspace.core)
                            .map_or_else(|| file_id_for(&workspace.core.root, path.resolved()), |file| file.id),
                    )
                })
                .collect::<Vec<_>>();
            let current = diagnostics_for(&mut self.core.entries_mut()[index], &changed);
            diagnostics.extend(self.route_diagnostics(index, current));
        }
        for uri in affected {
            diagnostics.entry(uri.clone()).or_insert_with(|| PublishedDiagnostics {
                diagnostics: Vec::new(),
                version: self.core.documents().get(uri).map(|document| document.version),
            });
        }
        self.publication.update(diagnostics)
    }

    fn diagnostic_route(&self, index: usize, uri: &Uri) -> DiagnosticRoute<'_> {
        let open = self.open_document(index, uri);
        let owner_uri = open.map_or(uri, |(uri, _)| uri);
        if owner_uri.to_file_path().is_some_and(|path| self.owner_index(&path) == Some(index)) {
            match open {
                Some((uri, document)) => DiagnosticRoute::Buffer { uri, version: document.version },
                None => DiagnosticRoute::File,
            }
        } else if open.is_some_and(|(open_uri, _)| *open_uri != *uri) {
            // Another workspace owns the buffer. Clear only this workspace's old URI.
            DiagnosticRoute::ClearAlias
        } else {
            DiagnosticRoute::Ignore
        }
    }

    fn open_document(&self, index: usize, uri: &Uri) -> Option<(&Uri, &OpenDocument)> {
        self.core.documents().get_key_value(uri).or_else(|| {
            let path = uri.to_file_path()?;
            let workspace = &self.core.entries()[index];
            let file = file_id_for(&workspace.core.root, &path);
            self.core.documents().get_key_value(workspace.core.opened_key(file)?)
        })
    }

    fn route_diagnostics(
        &self,
        index: usize,
        diagnostics: HashMap<Uri, PublishedDiagnostics>,
    ) -> HashMap<Uri, PublishedDiagnostics> {
        let mut routed = HashMap::default();
        let mut clears = Vec::new();
        for (uri, mut payload) in diagnostics {
            match self.diagnostic_route(index, &uri) {
                DiagnosticRoute::File => {
                    routed.insert(uri, payload);
                }
                DiagnosticRoute::Buffer { uri: open_uri, version } => {
                    payload.version = Some(version);
                    if *open_uri != uri {
                        routed.insert(uri, PublishedDiagnostics { diagnostics: Vec::new(), version: None });
                    }
                    // A retained alias may contribute a clear beside the current source payload.
                    // Keep the current issues regardless of hash-map iteration order.
                    match routed.entry(open_uri.clone()) {
                        std::collections::hash_map::Entry::Occupied(mut entry) if !payload.diagnostics.is_empty() => {
                            entry.insert(payload);
                        }
                        std::collections::hash_map::Entry::Vacant(entry) => {
                            entry.insert(payload);
                        }
                        _ => {}
                    }
                }
                DiagnosticRoute::ClearAlias => clears.push(uri),
                DiagnosticRoute::Ignore => {}
            }
        }
        for uri in clears {
            routed.insert(uri, PublishedDiagnostics { diagnostics: Vec::new(), version: None });
        }
        routed
    }
}

fn diagnostics_for(workspace: &mut WorkspaceState, changed: &[FileId]) -> HashMap<Uri, PublishedDiagnostics> {
    workspace
        .diagnostics
        .refresh(&workspace.core.server, &workspace.core.analyzer_ignores, changed)
        .into_iter()
        .map(|(uri, diagnostics)| (uri, PublishedDiagnostics { diagnostics, version: None }))
        .collect()
}
