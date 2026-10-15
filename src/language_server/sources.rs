//! Give embedded PHP sources stable file URIs that editors can open.

use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::LazyLock;

use foldhash::HashMap;
use mago_database::file::{File, FileId};
use tempfile::NamedTempFile;
use tower_lsp_server::gen_lsp_types::Uri;

use crate::consts::{PRELUDE_BYTES, VERSION};
use crate::language_server::uri::UriFilePath;
use mago_server::workspace::paths::relative_source_name;

static PRELUDE_DIRECTORY: LazyLock<PathBuf> =
    LazyLock::new(|| Path::new(VERSION).join(FileId::new(PRELUDE_BYTES).to_string()));
const WORKSPACE_MARKER: &str = ".workspace-uri";

pub struct SourceFiles {
    root: PathBuf,
    workspace: PathBuf,
    published: HashMap<FileId, Uri>,
}

impl SourceFiles {
    pub fn new(workspace: &Path) -> Self {
        let workspace = workspace.canonicalize().unwrap_or_else(|_| workspace.to_path_buf());
        let workspace_id = FileId::new(workspace.as_os_str().as_encoded_bytes());
        Self {
            root: cache_root().join(&*PRELUDE_DIRECTORY).join(workspace_id.to_string()),
            workspace,
            published: HashMap::default(),
        }
    }

    pub fn uri(&mut self, file: &File) -> Option<Uri> {
        if let Some(uri) = file.path.as_ref().and_then(Uri::from_file_path) {
            return Some(uri);
        }
        if !file.file_type.is_builtin() {
            return None;
        }
        if let Some(uri) = self.published.get(&file.id) {
            return Some(uri.clone());
        }
        let name = Path::new(std::str::from_utf8(&file.name).ok()?);
        if !relative_source_name(name) {
            return None;
        }
        let path = self.root.join(name);
        let workspace_uri = Uri::from_file_path(&self.workspace)?;
        if let Err(error) = publish(&self.root.join(WORKSPACE_MARKER), workspace_uri.as_str().as_bytes())
            .and_then(|_| publish(&path, &file.contents))
        {
            tracing::warn!(path = %path.display(), %error, "cannot publish builtin source");
            return None;
        }
        let uri = Uri::from_file_path(path)?;
        self.published.insert(file.id, uri.clone());
        Some(uri)
    }

    /// Recover the project when an editor restarts from a cached builtin file or directory.
    pub fn workspace_for_cached_path(path: &Path) -> Result<Option<PathBuf>, String> {
        let cache = cache_root();
        let Ok(relative) = path.strip_prefix(&cache) else { return Ok(None) };
        cached_workspace(&cache, relative)
            .map(Some)
            .ok_or_else(|| format!("cannot recover the project for cached builtin source {}", path.display()))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

#[cfg(test)]
mod tests;

fn cache_root() -> PathBuf {
    let temporary = std::env::temp_dir();
    temporary.canonicalize().unwrap_or(temporary).join("mago-language-server")
}

fn cached_workspace(cache: &Path, relative: &Path) -> Option<PathBuf> {
    let mut parts = relative.strip_prefix(&*PRELUDE_DIRECTORY).ok()?.components();
    let Component::Normal(workspace_id) = parts.next()? else { return None };
    if parts.any(|part| !matches!(part, Component::Normal(_))) {
        return None;
    }
    let marker = cache.join(&*PRELUDE_DIRECTORY).join(workspace_id).join(WORKSPACE_MARKER);
    let metadata = std::fs::symlink_metadata(&marker).ok()?;
    if !metadata.file_type().is_file() || metadata.len() > 128 * 1024 {
        return None;
    }
    let uri = std::fs::read_to_string(marker).ok()?.parse::<Uri>().ok()?;
    let workspace = uri.to_file_path()?.into_owned();
    if !workspace.is_absolute()
        || workspace.starts_with(cache)
        || !workspace.is_dir()
        || workspace.canonicalize().ok()? != workspace
        || workspace_id != FileId::new(workspace.as_os_str().as_encoded_bytes()).to_string().as_str()
    {
        return None;
    }
    Some(workspace)
}

/// Publish complete, read-only bytes. Concurrent sessions may share the same source.
fn publish(path: &Path, contents: &[u8]) -> io::Result<()> {
    if let Ok(existing) = std::fs::read(path) {
        return if existing == contents { Ok(()) } else { Err(io::Error::other("cached builtin source differs")) };
    }
    let parent = path.parent().ok_or_else(|| io::Error::other("builtin source has no parent directory"))?;
    std::fs::create_dir_all(parent)?;
    let mut temporary = NamedTempFile::new_in(parent)?;
    temporary.write_all(contents)?;
    match temporary.persist_noclobber(path) {
        Ok(file) => {
            let mut permissions = file.metadata()?.permissions();
            permissions.set_readonly(true);
            file.set_permissions(permissions)
        }
        Err(error) if error.error.kind() == io::ErrorKind::AlreadyExists && std::fs::read(path)? == contents => Ok(()),
        Err(error) => Err(error.error),
    }
}
