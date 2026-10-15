//! Pass editor buffer events to the registry's version authority.

use tower_lsp_server::gen_lsp_types::Uri;

use super::Backend;

impl Backend {
    pub(super) async fn apply_buffer_open(&self, uri: Uri, text: String, version: i32) {
        self.apply_registry_update(move |registry| registry.open_buffer(uri, text, version)).await;
    }

    pub(super) async fn apply_buffer_change(&self, uri: Uri, text: String, version: i32) {
        self.apply_registry_update(move |registry| registry.change_buffer(uri, text, version)).await;
    }

    pub(super) async fn apply_buffer_close(&self, uri: Uri) {
        self.apply_registry_update(move |registry| registry.close_buffer(&uri)).await;
    }
}
