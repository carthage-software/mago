//! Diagnostic preparation and publication.

use tower_lsp_server::gen_lsp_types::Uri;

use crate::language_server::diagnostic_publication::PublishedDiagnostics;

use super::Backend;

impl Backend {
    #[cfg(test)]
    pub(in crate::language_server) async fn test_barrier(
        &self,
        params: tower_lsp_server::gen_lsp_types::LogMessageParams,
    ) {
        self.client.log_message(params.kind, params.message).await;
    }

    pub(super) async fn publish_initial_diagnostics(&self) {
        let diagnostics = self.with_registry_mut(|registry| registry.publication.take_initial()).unwrap_or_default();
        self.publish_diagnostics(diagnostics).await;
    }

    pub(super) async fn publish_diagnostics(&self, diagnostics: Vec<(Uri, PublishedDiagnostics)>) {
        let preferences = self.client_preferences().diagnostics;
        for (uri, mut published) in diagnostics {
            for diagnostic in &mut published.diagnostics {
                preferences.apply(diagnostic);
            }
            self.client.publish_diagnostics(uri, published.diagnostics, published.version).await;
        }
    }
}
