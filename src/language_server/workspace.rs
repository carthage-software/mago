//! Workspace file walking and path/URI/FileId bridging.

use std::path::Path;
use std::path::PathBuf;

use mago_database::file::File as MagoFile;
use mago_database::file::FileId;
use mago_database::file::FileType;
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

/// Recursively walk `root` and collect every `.php` file as a [`MagoFile`].
///
/// Skips dotfiles, `vendor`, and `node_modules`. Returns the first IO error
/// encountered.
pub fn walk_php_files(root: &Path) -> Result<Vec<MagoFile>, String> {
    let mut files = Vec::new();
    walk_dir(root, root, &mut files)?;
    Ok(files)
}

fn walk_dir(workspace: &Path, dir: &Path, out: &mut Vec<MagoFile>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|err| format!("read_dir({}): {err}", dir.display()))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };

        if file_name.starts_with('.') || file_name == "vendor" || file_name == "node_modules" {
            continue;
        }

        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };

        if file_type.is_dir() {
            walk_dir(workspace, &path, out)?;
        } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "php") {
            match MagoFile::read(workspace, &path, FileType::Host) {
                Ok(file) => out.push(file),
                Err(err) => return Err(format!("read({}): {err}", path.display())),
            }
        }
    }

    Ok(())
}

/// Convert a filesystem path to an LSP `file://` URL.
#[allow(dead_code)]
pub fn url_for_path(path: &Path) -> Option<Url> {
    Url::from_file_path(path).ok()
}
