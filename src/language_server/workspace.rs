//! Workspace path/URI/FileId bridging. The actual file discovery is
//! driven by [`mago_database::loader::DatabaseLoader`] (see
//! [`super::state::build_workspace`]) so the LSP honours the same
//! `[source]` configuration as `mago lint` / `mago analyze`.

use std::path::Path;
use std::path::PathBuf;

use mago_database::file::FileId;
use tower_lsp::lsp_types::InitializeParams;
use tower_lsp::lsp_types::Url;

/// Resolve the workspace root from `initialize` params, in the order the
/// LSP spec recommends: `workspaceFolders`, then deprecated `rootUri`, then
/// deprecated `rootPath`.
#[must_use]
pub fn workspace_root(params: &InitializeParams) -> Option<PathBuf> {
    if let Some(folders) = &params.workspace_folders
        && let Some(first) = folders.first()
        && let Ok(path) = first.uri.to_file_path()
    {
        return Some(path);
    }

    #[allow(deprecated)]
    if let Some(uri) = &params.root_uri
        && let Ok(path) = uri.to_file_path()
    {
        return Some(path);
    }

    #[allow(deprecated)]
    if let Some(root) = &params.root_path {
        return Some(PathBuf::from(root));
    }

    None
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
    FileId::new(&logical_name_for(workspace, path))
}

/// Convert a filesystem path to an LSP `file://` URL.
#[allow(dead_code)]
pub fn url_for_path(path: &Path) -> Option<Url> {
    Url::from_file_path(path).ok()
}
