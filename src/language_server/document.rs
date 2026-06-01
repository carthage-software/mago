//! Open document tracking.
//!
//! When the editor opens a file via `textDocument/didOpen`, we mirror its
//! content into the workspace database (so the analyzer sees the buffer, not
//! the disk version). On close we restore from disk if the file existed
//! there, or delete it from the database if it was a virtual / new file.

use mago_database::file::FileId;

/// State for a single editor-open document.
pub struct OpenDocument {
    /// The mago file id used to look this document up in the database.
    pub file_id: FileId,
    /// `true` if the file did not exist on disk when first opened.
    /// On close we delete it from the database; otherwise we re-read disk.
    pub virtual_file: bool,
    /// Document version from the LSP client, used to discard out-of-order
    /// `didChange` notifications. Currently informational only.
    pub version: i32,
}
