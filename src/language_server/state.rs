//! Per-workspace analysis state, settings, source URIs, and diagnostics.
//!
//! The backend session owns the registry and controls access to each workspace.
//! The transport-independent [`mago_server::Server`] owns analysis state and caches.

use crate::language_server::uri::UriFilePath;
use tower_lsp_server::gen_lsp_types::Location;

use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::file::File;
use mago_server::workspace::Workspace;
use tower_lsp_server::gen_lsp_types::Uri;

use crate::config::Configuration;
use crate::language_server::diagnostic_cache::DiagnosticCache;
use crate::language_server::position::range_at_offsets;
use crate::language_server::sources::SourceFiles;

pub struct WorkspaceState {
    pub core: Workspace<Uri>,
    pub sources: SourceFiles,
    /// This workspace's own configuration, discovered from `{root}/mago.*` on
    /// `initialize` (or inherited from the launch config when the folder has
    /// none). Drives ignore patterns, globs, and formatter settings.
    pub configuration: Configuration,
    pub diagnostics: DiagnosticCache,
}

impl WorkspaceState {
    pub fn document_uri(&self, file: &File) -> Option<Uri> {
        self.core.opened_key(file.id).cloned().or_else(|| file.path.as_ref().and_then(Uri::from_file_path))
    }

    pub fn uri(&mut self, file: &File) -> Option<Uri> {
        self.core.opened_key(file.id).cloned().or_else(|| self.sources.uri(file))
    }

    pub fn locations(&mut self, locations: &[mago_server::SymbolLocation]) -> Vec<Location> {
        locations
            .iter()
            .filter_map(|location| {
                let file = self.database().get(&location.file).ok()?;
                Some(Location {
                    uri: self.uri(&file)?,
                    range: range_at_offsets(&file, location.range.start, location.range.end),
                })
            })
            .collect()
    }

    /// Borrow the workspace file database.
    #[must_use]
    pub fn database(&self) -> &Database<'static> {
        self.core.server.database()
    }
}

impl AsRef<Workspace<Uri>> for WorkspaceState {
    fn as_ref(&self) -> &Workspace<Uri> {
        &self.core
    }
}

impl AsMut<Workspace<Uri>> for WorkspaceState {
    fn as_mut(&mut self) -> &mut Workspace<Uri> {
        &mut self.core
    }
}
