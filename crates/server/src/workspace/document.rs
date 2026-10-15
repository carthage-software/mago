//! Accepted caller contents, independent of workspace lifetime.

use std::path::PathBuf;
use std::sync::Arc;

use mago_database::file::File;

use super::document_path::DocumentPath;

/// State for a single caller-open document.
pub struct OpenDocument {
    /// Keep the requested path so a deleted symlink target cannot lose the loaded ID.
    pub path: PathBuf,
    pub resolved_path: PathBuf,
    /// Share one database allocation. Only the contents apply to other workspaces;
    /// each workspace derives its own file ID, logical name, and source type.
    pub snapshot: Arc<File>,
    /// Last accepted version; older or repeated changes are ignored.
    pub version: i32,
}

impl OpenDocument {
    pub fn matches(&self, path: &DocumentPath<'_>) -> bool {
        self.path == path.requested()
            || self.resolved_path == path.resolved()
            || DocumentPath::new(&self.path).resolved() == path.resolved()
    }
}
