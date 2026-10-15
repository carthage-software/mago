//! LSP messages ready to publish after a shared workspace update.

use tower_lsp_server::gen_lsp_types::Uri;

use crate::language_server::diagnostic_publication::PublishedDiagnostics;

#[derive(Default)]
pub struct WorkspaceUpdate {
    pub changed_files: usize,
    pub refresh_code_lenses: bool,
    pub diagnostics: Vec<(Uri, PublishedDiagnostics)>,
    pub errors: Vec<String>,
}
