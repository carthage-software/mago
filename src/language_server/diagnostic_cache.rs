//! Map changed diagnostics to document URIs and clear old addresses.

use foldhash::HashMap;
use foldhash::HashSet;
use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::file::FileId;
use mago_reporting::CompiledIgnoreSet;
use mago_server::Server;
use tower_lsp_server::gen_lsp_types::Diagnostic;
use tower_lsp_server::gen_lsp_types::Uri;

use super::diagnostics::build_diagnostics;
use super::uri::UriFilePath;

#[cfg(test)]
mod tests;

#[derive(Default)]
pub struct DiagnosticCache {
    issues: mago_server::diagnostics::DiagnosticCache,
    /// Keep URIs with diagnostics until their next clear or path change.
    uris: HashMap<FileId, Uri>,
}

impl DiagnosticCache {
    pub fn uris(&self) -> impl Iterator<Item = &Uri> {
        self.uris.values()
    }

    pub fn refresh(
        &mut self,
        server: &Server,
        ignores: &CompiledIgnoreSet,
        changed: &[FileId],
    ) -> HashMap<Uri, Vec<Diagnostic>> {
        let update = self.issues.refresh(server, changed);
        let database = server.database();
        let mut diagnostics = build_diagnostics(database, update.issues(database, ignores));
        self.update_document_uris(database, &update.files, &mut diagnostics);
        diagnostics
    }

    /// Include empty payloads when issues disappear or a file loses its old URI.
    fn update_document_uris(
        &mut self,
        database: &Database<'_>,
        dirty: &HashSet<FileId>,
        diagnostics: &mut HashMap<Uri, Vec<Diagnostic>>,
    ) {
        let previous_capacity = self.uris.capacity();
        for &file in dirty {
            if let Ok(file_data) = database.get_ref(&file)
                && let Some(uri) = file_data.path.as_ref().and_then(Uri::from_file_path)
            {
                let old = if diagnostics.get(&uri).is_some_and(|issues| !issues.is_empty()) {
                    self.uris.insert(file, uri.clone())
                } else {
                    self.uris.remove(&file)
                };
                if let Some(old) = old
                    && old != uri
                {
                    diagnostics.entry(old).or_default();
                }
                diagnostics.entry(uri).or_default();
            } else if let Some(uri) = self.uris.remove(&file) {
                diagnostics.entry(uri).or_default();
            }
        }
        if previous_capacity.max(self.uris.capacity()) > self.uris.len().saturating_mul(4).max(64) {
            self.uris.shrink_to_fit();
        }
    }
}
