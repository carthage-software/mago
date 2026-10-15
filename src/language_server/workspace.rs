//! Workspace loading and path helpers.

use crate::language_server::uri::UriFilePath;
use std::path::PathBuf;

use tower_lsp_server::gen_lsp_types::{InitializeParams, RootPath, WorkspaceFolders};

mod loading;

pub use loading::build_workspace;

/// Resolve every workspace root from `initialize` params, in the order the
/// LSP spec recommends: all `workspaceFolders`, then the deprecated single
/// `rootUri`, then the deprecated `rootPath`. Returns one entry per open
/// project so the server can drive a [`Server`](mago_server::Server) per root.
#[must_use]
pub fn workspace_roots(params: &InitializeParams) -> Vec<PathBuf> {
    if let Some(WorkspaceFolders::WorkspaceFolderList(folders)) =
        &params.workspace_folders_initialize_params.workspace_folders
        && !folders.is_empty()
    {
        let roots: Vec<PathBuf> = folders.iter().filter_map(|f| f.uri.to_file_path().map(|p| p.into_owned())).collect();
        if !roots.is_empty() {
            return roots;
        }
    }

    #[allow(deprecated)]
    if let Some(uri) = &params.root_uri
        && let Some(path) = uri.to_file_path()
    {
        return vec![path.into_owned()];
    }

    #[allow(deprecated)]
    if let Some(RootPath::String(root)) = &params.root_path {
        return vec![PathBuf::from(root)];
    }

    Vec::new()
}
