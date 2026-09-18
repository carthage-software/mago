//! Workspace path/URI/FileId bridging. The actual file discovery is
//! driven by [`mago_database::loader::DatabaseLoader`] (see
//! [`super::state::build_workspace`]) so the LSP honours the same
//! `[source]` configuration as `mago lint` / `mago analyze`.

use std::path::Path;
use std::path::PathBuf;

use mago_database::file::FileId;
use tower_lsp_server::ls_types::InitializeParams;

/// Resolve every workspace root from `initialize` params, in the order the
/// LSP spec recommends: all `workspaceFolders`, then the deprecated single
/// `rootUri`, then the deprecated `rootPath`. Returns one entry per open
/// project so the server can drive a [`Server`](mago_server::Server) per root.
#[must_use]
pub fn workspace_roots(params: &InitializeParams) -> Vec<PathBuf> {
    if let Some(folders) = &params.workspace_folders
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
    if let Some(root) = &params.root_path {
        return vec![PathBuf::from(root)];
    }

    Vec::new()
}

/// Compute the workspace-relative logical name used by `mago_database` to
/// derive [`FileId`]s. Mirrors `mago_database::file::read_file`.
#[must_use]
pub fn logical_name_for(workspace: &Path, path: &Path) -> String {
    path.strip_prefix(workspace).unwrap_or(path).to_string_lossy().into_owned()
}

/// Compute the [`FileId`] mago will use for a file at `path` in `workspace`.
#[must_use]
pub fn file_id_for(workspace: &Path, path: &Path) -> FileId {
    FileId::new(logical_name_for(workspace, path).as_bytes())
}
