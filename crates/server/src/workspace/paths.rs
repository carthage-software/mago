//! Keep native paths and database file names in sync.

use mago_database::file::{File, FileId};
use mago_database::{Database, DatabaseReader};
use std::path::{Component, Path};
use std::sync::Arc;

/// Compute the workspace-relative logical name used by `mago_database` to
/// derive [`FileId`]s. Matches the name bytes from `mago_database::file::File::read`.
#[must_use]
pub fn logical_name_for(workspace: &Path, path: &Path) -> Vec<u8> {
    let name = path.strip_prefix(workspace).unwrap_or(path).as_os_str().as_encoded_bytes();
    #[cfg(windows)]
    let name = name.iter().map(|byte| if *byte == b'\\' { b'/' } else { *byte }).collect();
    #[cfg(not(windows))]
    let name = name.to_vec();
    name
}

/// Compute the [`FileId`] mago will use for a file at `path` in `workspace`.
#[must_use]
pub fn file_id_for(workspace: &Path, path: &Path) -> FileId {
    FileId::new(&logical_name_for(workspace, path))
}

/// Resolve a published builtin path without accepting paths outside its source directory.
#[must_use]
pub fn builtin_file_at(database: &Database<'_>, root: &Path, path: &Path) -> Option<Arc<File>> {
    let name = path.strip_prefix(root).ok()?;
    if !relative_source_name(name) {
        return None;
    }
    let name = name.to_str()?.replace(std::path::MAIN_SEPARATOR, "/");
    let file = database.get(&FileId::new(name.as_bytes())).ok()?;
    (file.file_type.is_builtin() && file.path.is_none()).then_some(file)
}

/// Source names may only contain normal, relative path components.
#[must_use]
pub fn relative_source_name(path: &Path) -> bool {
    !path.as_os_str().is_empty() && path.components().all(|component| matches!(component, Component::Normal(_)))
}

#[cfg(test)]
#[path = "paths/tests.rs"]
mod tests;
